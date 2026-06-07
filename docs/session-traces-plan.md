# session_traces 规划文档

> 目标：在 cc-gateway-pro 的现有代理链路中，增加一条用户显式开启后才生效的会话上下文记录旁路，用于查看每个 session / turn 的系统提示词、消息摘要、工具定义、工具调用、响应内容、token usage 与请求差异。

前端入口与交互原型见：`docs/session-traces-frontend-prototype.md`。

## 1. Hermes 会话内容总结

本规划来自 Hermes session `20260604_110118_ff7651`，标题为“claude-tap API流量截获原理分析”。该会话先分析 `liaohch3/claude-tap` 如何捕获真实 API 流量，然后把它映射到本仓库的 proxy 架构中。

Hermes 得出的核心判断是：

- claude-tap 的关键不是“懂 Claude Code 内部状态”，而是在 API 边界截获已经展开后的请求与响应。
- 系统提示词来自请求 body 的 `system` 字段。
- 对话历史来自请求 body 的 `messages[]`。
- 工具模式 / 工具定义来自请求 body 的 `tools[]`。
- 工具调用与响应文本来自响应 body，流式场景需要逐个 SSE event 累积重建。
- token usage 来自响应的 `usage` 字段或流式事件里的 usage delta。
- cc-gateway-pro 已经处在 reverse proxy 位置，天然能看到 request body、response body、SSE chunk 和 session_id。
- 当前项目已经把 token usage 写进 `proxy_request_logs`，但请求上下文、工具定义、响应文本等没有持久化。

Hermes 推荐的实现方式是：不重写代理管道，在现有 `spawn_log_usage` 旁边加一个 `spawn_log_session_trace`，同时扩展流式 collector，让它除了 usage 以外也能累积 response text / tool calls。

## 2. 当前代码锚点

### 2.1 请求侧

入口在 `src-tauri/src/proxy/handlers.rs`。

`handle_messages_for_app` 已经读取并解析完整请求体：

```rust
let body_bytes = body.collect().await?.to_bytes();
let mut body: Value = serde_json::from_slice(&body_bytes)?;
let mut ctx = RequestContext::new(&state, &mut body, &headers, ...).await?;
```

随后通过 `forward_with_retry(..., body.clone(), ...)` 转发。也就是说，请求侧的 `system`、`messages`、`tools`、`model`、`stream` 在转发前都已经是 `serde_json::Value`，不需要二次抓包。

### 2.2 请求上下文

`src-tauri/src/proxy/handler_context.rs` 的 `RequestContext` 已经包含 trace 所需的关键元数据：

- `start_time`
- `provider`
- `current_provider_id`
- `request_model`
- `original_model`
- `app_type_str`
- `session_id`
- `session_client_provided`

这说明 session_traces 不需要单独发明 session 识别机制，应该复用 `RequestContext.session_id`。

### 2.3 响应侧

`src-tauri/src/proxy/response_processor.rs` 是最核心落点：

- `handle_non_streaming` 会读取完整响应 body，并在 logging 开启时 parse JSON。
- `handle_streaming` 会创建 `SseUsageCollector`，在 `create_logged_passthrough_stream` 中逐 chunk 解析 SSE。
- `create_usage_collector` 当前只为 token usage 收集 `Vec<Value>`，且受 `stream_event_filter` 影响，可能只保留 usage 相关事件。
- `spawn_log_usage` / `log_usage_internal` 最终通过 `UsageLogger` 写入 `proxy_request_logs`。

session_traces 应该复用这里的生命周期，但不要把 trace 写入逻辑混进 usage 计费逻辑。

### 2.4 数据库

数据库 schema 在 `src-tauri/src/database/schema.rs`，当前 `SCHEMA_VERSION = 10`。

已有 `proxy_request_logs` 字段包括：

- `request_id`
- `provider_id`
- `app_type`
- `model`
- `request_model`
- token 四件套
- cost
- `latency_ms`
- `first_token_ms`
- `status_code`
- `session_id`
- `is_streaming`
- `created_at`
- `data_source`

session_traces 应该作为新表存在，并用 `proxy_request_id` 或 `request_id` 与 `proxy_request_logs` 做弱关联，而不是直接扩展 `proxy_request_logs` 存大文本。

## 3. 目标与非目标

### 3.1 Goals

- 按 session + turn 记录一次真实 API 调用的上下文摘要。
- 支持非流式和流式响应。
- 支持 Claude / Codex / Gemini 等现有 app_type 的统一存储模型。
- 能展示系统提示词、消息摘要、工具定义摘要、工具调用、响应文本、stop reason、token usage 与基础耗时。
- 默认保护隐私：Session Traces 默认关闭，用户显式开启后才记录。
- 完整 request / response body 必须二次确认后才能开启。
- 不影响现有 proxy 透传性能、故障转移、usage 统计和格式转换。

### 3.2 Non-Goals

- 不做 TLS MITM 或 forward proxy，cc-gateway-pro 当前场景是 reverse proxy。
- 不上传 trace 到云端。
- v1 不要求完整复刻 claude-tap 的 compact 回放格式。
- v1 不做复杂 diff UI，只存必要字段，后续再基于数据做“请求差异”视图。

## 4. 推荐数据模型

新增表：`session_traces`。

```sql
CREATE TABLE IF NOT EXISTS session_traces (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    trace_id TEXT NOT NULL UNIQUE,
    proxy_request_id TEXT,

    session_id TEXT NOT NULL,
    turn_index INTEGER,
    app_type TEXT NOT NULL,
    provider_id TEXT,
    model TEXT,
    request_model TEXT,
    is_streaming INTEGER NOT NULL DEFAULT 0,
    status_code INTEGER,

    system_prompt_preview TEXT,
    system_prompt_hash TEXT,
    message_count INTEGER NOT NULL DEFAULT 0,
    tool_count INTEGER NOT NULL DEFAULT 0,
    request_summary_json TEXT NOT NULL DEFAULT '{}',
    context_stats_json TEXT NOT NULL DEFAULT '{}',
    context_window_tokens INTEGER,
    context_used_tokens INTEGER,
    context_usage_ratio REAL,
    request_json TEXT,

    response_text_preview TEXT,
    response_text TEXT,
    response_json TEXT,
    tool_calls_json TEXT NOT NULL DEFAULT '[]',
    stop_reason TEXT,

    input_tokens INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    cache_read_tokens INTEGER NOT NULL DEFAULT 0,
    cache_creation_tokens INTEGER NOT NULL DEFAULT 0,
    latency_ms INTEGER,
    first_token_ms INTEGER,

    trace_mode TEXT NOT NULL DEFAULT 'summary',
    redaction_version INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_session_traces_session
    ON session_traces(session_id, turn_index);

CREATE INDEX IF NOT EXISTS idx_session_traces_created_at
    ON session_traces(created_at);

CREATE INDEX IF NOT EXISTS idx_session_traces_app
    ON session_traces(app_type, provider_id);
```

字段说明：

- `trace_id`：trace 自己的 UUID。不要依赖 usage 的 `request_id`，因为 usage request id 可能来自响应 message id，也可能是随机生成。
- `proxy_request_id`：可选关联 `proxy_request_logs.request_id`。如果 usage parser 生成了 request id，就写入；拿不到时保持 NULL。
- `turn_index`：v1 可以插入时按 `session_id` 计算 `MAX(turn_index) + 1`；如果担心并发，查询侧也可以用 `ROW_NUMBER() OVER (PARTITION BY session_id ORDER BY created_at, id)` 动态展示。
- `request_summary_json`：结构化摘要，保留 `messages[].role`、content 类型、文本 preview、图片/文件/tool_result 计数等。
- `context_stats_json`：当前 turn 的 context 统计快照，包含分类 token、已加载 tools、skills、agents、memory files、模型窗口、剩余空间等。
- `request_json` / `response_json`：仅在完整模式启用时保存，且必须先脱敏。
- `response_text_preview`：默认保存，长度建议 1,000 到 2,000 字符。
- `response_text`：可配置保存完整文本，默认可为空。
- 表内 `trace_mode DEFAULT 'summary'` 只表示“已写入 trace 行的默认采集级别”；全局设置仍必须默认 `off`，关闭时不插入 trace 行。

## 5. Trace Mode 策略

建议新增三个级别，先实现 `summary`：

| 模式 | 保存内容 | 默认 | 用途 |
| --- | --- | --- | --- |
| `off` | 不保存 trace | 是 | 用户完全关闭上下文记录 |
| `summary` | system preview、消息摘要、工具摘要、响应 preview、usage | 否 | 低风险、低空间占用的会话质量分析 |
| `full` | 脱敏后的完整 request / response JSON、完整响应文本 | 否 | 精确排障、回放、请求差异分析 |

配置建议：

- 不复用 `proxy_config.enable_logging` 作为上下文记录总开关。
- 新增独立设置 `session_traces_enabled` 与 `session_trace_mode`。
- `proxy_config.enable_logging` 只控制 usage / proxy 日志；`session_traces_enabled` 控制上下文 trace。
- 当 `session_traces_enabled=false` 或 `session_trace_mode=off` 时，流式热路径不做额外 trace 解析。
- 新增长度限制：`system_preview_limit`、`message_preview_limit`、`response_preview_limit`、`full_text_limit`。
- Full mode 需要弹窗二次确认，提示可能保存 system prompt、消息 preview、工具 input、响应 preview 等本地数据。

## 6. Session Context 统计与质量分析

用户截图里的 `/context` 信息可以作为 session_traces 的上层目标：不仅记录每轮请求，还要能回答“当前会话质量如何、context 被什么占用了、哪些工具/skills/MCP 被加载或调用、每轮用了哪些模型和 token”。

### 6.1 能收集什么

| 信息 | 来源 | 准确度 | 说明 |
| --- | --- | --- | --- |
| 当前模型 | request body `model`、response body `model`、`RequestContext.request_model` | 高 | 可同时记录请求模型与供应商实际返回模型 |
| context window | 模型元数据 / 内置映射 / provider metadata | 中到高 | 需要维护 `model -> context_window` 映射，未知模型可为空 |
| 总 input/output/cache tokens | response `usage` / SSE usage events / `proxy_request_logs` | 高 | 这是现有 usage 管道已经在做的事实数据 |
| 每轮 usage | `session_traces` + `proxy_request_logs` 按 `session_id` 聚合 | 高 | 可展示 turn-by-turn token/cost/latency/model |
| system prompt token | request body `system` | 中到高 | token 数建议用本地 tokenizer 估算，或用字符数近似 |
| messages token | request body `messages[]` | 中到高 | 包括历史对话、tool_result、文件内容等已进入 API 的上下文 |
| tools/MCP schema token | request body `tools[]` | 中到高 | 可按 tool name 分类，MCP tool 通常能从命名前缀识别 |
| 实际工具调用 | response tool calls / SSE tool_use events | 高 | 可统计每轮调用了哪些 tool、调用次数、失败率 |
| loaded MCP tools | request body `tools[]` | 高 | “加载了哪些工具定义”可以准确看到；是否来自 MCP 需按命名或 metadata 识别 |
| skills / agents / memory files | system prompt、messages、session 文件、CLI metadata | 中 | 只靠 API 请求可推断；要做到截图级来源标签，建议补本地 session 文件解析 |
| free space / usage ratio | `context_window - context_used_tokens` | 中 | context_used 可用 usage input tokens 或本地估算值 |

结论：用于“分析当前会话质量”的核心数据可以通过 proxy 收集。截图里那种精细来源分组也能做，但要分两档：v1 先做 API 侧可验证统计，v2 再叠加 Claude/Codex/Hermes 本地 session metadata，让 memory files、skills、custom agents 的来源更准。

### 6.2 每轮 Context Snapshot

每一条 `session_traces` 除了请求/响应摘要，还应保存当前 turn 的 context 快照：

```json
{
  "model": "claude-opus-4-8",
  "contextWindowTokens": 1000000,
  "contextUsedTokens": 66800,
  "contextUsageRatio": 0.067,
  "freeTokens": 933200,
  "categories": [
    { "key": "system_prompt", "label": "System prompt", "tokens": 1900, "ratio": 0.002 },
    { "key": "tools", "label": "MCP/tools schema", "tokens": 3300, "ratio": 0.003 },
    { "key": "memory_files", "label": "Memory files", "tokens": 2900, "ratio": 0.003 },
    { "key": "skills", "label": "Skills", "tokens": 3300, "ratio": 0.003 },
    { "key": "messages", "label": "Messages", "tokens": 55400, "ratio": 0.055 }
  ],
  "loadedTools": [
    { "name": "mcp_mysql_list_tables", "source": "mcp", "estimatedTokens": 0 },
    { "name": "mcp_mysql_run_select_query", "source": "mcp", "estimatedTokens": 0 }
  ],
  "calledTools": [
    { "name": "mcp_mysql_run_select_query", "count": 2, "turns": [3, 5] }
  ],
  "skills": [
    { "name": "docx", "scope": "user", "estimatedTokens": 120 },
    { "name": "source-command-e2e", "scope": "user", "estimatedTokens": 60 }
  ],
  "memoryFiles": [
    { "path": "~/.claude/CLAUDE.md", "estimatedTokens": 610 }
  ]
}
```

v1 可以把它作为 `context_stats_json` 存在 `session_traces` 中；后续如果查询压力变大，再拆成 `session_context_snapshots`、`session_context_items` 两张规范化表。

### 6.3 Session 聚合统计

基于多轮 `session_traces` + `proxy_request_logs`，可以为一个 session 生成聚合报告：

- Context 水位：最后一轮 used/free/ratio、峰值 context、增长速度。
- 分类占比：system、messages、tools、skills、memory、agents 的估算 token 占比。
- 模型使用：每个模型的 turn 数、input/output/cache tokens、cost、平均 latency。
- 工具加载：加载了多少 tool schema，哪些 MCP server 占用最多 schema token。
- 工具调用：实际调用了哪些 tools/MCP，调用次数、失败次数、平均耗时。
- Skills/agents：识别到哪些 skills/custom agents，估算 token 占用。
- 质量指标：cache hit rate、output/input ratio、tool-call density、error rate、stop reason 分布、context 接近上限告警。

建议新增后端 service：`src-tauri/src/services/session_context_stats.rs`。

建议新增命令：

- `get_session_context_summary(session_id: string)`
- `list_session_turn_usage(session_id: string)`
- `list_session_context_items(session_id: string, turn_index?: number)`

前端使用独立 `Session Traces` 页面展示，不嵌入现有 Session Manager 详情页。入口放在 Session Manager 顶栏图标旁边，展示类似截图的结构：

- 顶部：context usage bar、模型、窗口、used/free。
- 中部：按 category 的 token 占比。
- 下方：MCP tools、custom agents、memory files、skills 清单。
- 右侧或底部：每轮 usage 表格，按 model/provider/turn 展示。

### 6.4 质量评分建议

可以先不做单一分数，先给可解释指标。若后续需要“会话质量评分”，建议由这些指标组成：

- Context pressure：`context_usage_ratio` 越高风险越大，超过 70% 提醒 compact。
- Tool efficiency：实际调用工具数 / 已加载工具数，长期过低说明 tool schema 过重。
- Message growth：最近 N 轮 messages token 增速，判断是否需要总结历史。
- Cache efficiency：cache read / cacheable input，评估 prompt cache 利用率。
- Model stability：同 session 内频繁切模型可能导致质量波动。
- Error/stop health：错误率、超时、`max_tokens` stop reason。

## 7. 后端实现规划

### Phase 1：数据库与 DAO

1. 将 `SCHEMA_VERSION` 从 10 升到 11。
2. 在 `create_tables_on_conn` 中创建 `session_traces` 与索引。
3. 在 `apply_schema_migrations_on_conn` 增加 `migrate_v10_to_v11`。
4. 新增独立设置存储，至少包含：
   - `session_traces_enabled`
   - `session_trace_mode`
   - `session_trace_retention_days`
   - `session_trace_response_preview_limit`
5. 新增 `src-tauri/src/database/dao/session_traces.rs`，提供：
   - `insert_session_trace`
   - `list_session_traces`
   - `get_session_trace`
   - `delete_session_traces_for_session`
   - `prune_session_traces`
6. 让 WebDAV auto sync 知道这张表是否需要同步；建议 v1 不自动同步完整 trace，除非 trace mode 明确允许。

### Phase 2：请求摘要提取

新增模块：`src-tauri/src/proxy/session_trace.rs`。

建议职责：

- `build_request_snapshot(ctx, request_body, trace_mode) -> RequestTraceSnapshot`
- `extract_system_prompt(value) -> String`
- `summarize_messages(value) -> Value`
- `summarize_tools(value) -> Value`
- `redact_json(value) -> Value`
- `hash_text(value) -> String`

脱敏规则至少覆盖：

- `api_key`
- `authorization`
- `x-api-key`
- `cookie`
- `set-cookie`
- `access_token`
- `refresh_token`
- `password`
- `secret`
- `private_key`

注意：请求 body 中也可能出现环境变量、MCP 参数、tool input 的密钥，脱敏逻辑要递归处理 key 名。

### Phase 3：非流式响应记录

修改 `process_response` / `handle_non_streaming` 的签名，让它能拿到 request snapshot 或原始 request body。

推荐不要在 `handle_non_streaming` 内直接重新读请求，而是在 handler 中构造 snapshot：

```rust
let trace_snapshot = SessionTraceSnapshot::from_request(&ctx, &body, trace_mode);
process_response(response, &ctx, &state, &PARSER_CONFIG, connection_guard, Some(trace_snapshot)).await
```

非流式记录点：

- parse `body_bytes` 为 JSON。
- 复用 `parser_config.response_parser` 得到 `TokenUsage`。
- 从响应 JSON 提取 `response_text`、`tool_calls`、`stop_reason`。
- 调用 `spawn_log_session_trace` 写库。

### Phase 4：流式响应重建

当前 `SseUsageCollector` 为 usage 服务，并且会被 `stream_event_filter` 过滤。session trace 需要更完整的事件序列，因此建议不要直接复用它，而是抽象出一个更通用的 `SseTraceCollector`：

```rust
struct SseTraceCollector {
    request_snapshot: SessionTraceSnapshot,
    events: Vec<Value>,
    response_text: String,
    tool_calls: Vec<ToolCallSummary>,
    stop_reason: Option<String>,
}
```

接入方式：

- `create_logged_passthrough_stream` 接收 `usage_collector` 和 `trace_collector` 两个 collector。
- 每个 SSE `data:` JSON 先进入 trace collector，再根据 filter 决定是否进入 usage collector。
- 流结束时分别 finish：usage 写 `proxy_request_logs`，trace 写 `session_traces`。

各格式提取策略：

- Claude Anthropic SSE：
  - `content_block_delta.delta.type == "text_delta"` 累积文本。
  - `content_block_start.content_block.type == "tool_use"` 记录 tool name / id。
  - `input_json_delta.partial_json` 累积 tool input preview。
  - `message_delta.delta.stop_reason` 记录 stop reason。
- OpenAI Chat SSE：
  - `choices[].delta.content` 累积文本。
  - `choices[].delta.tool_calls[]` 累积工具调用。
  - `choices[].finish_reason` 记录 stop reason。
- OpenAI Responses SSE：
  - `response.output_text.delta` 累积文本。
  - `response.output_item.added` / `response.function_call_arguments.delta` 记录工具调用。
- Gemini SSE / transformed SSE：
  - 优先在转换后的 Anthropic SSE 上记录，降低格式分支数量。

### Phase 5：命令与前端 API

后端命令建议放在 `src-tauri/src/commands/session_traces.rs`：

- `list_session_traces(session_id?: string, app_type?: string, page?: number, page_size?: number)`
- `get_session_trace(trace_id: string)`
- `delete_session_trace(trace_id: string)`
- `delete_session_traces_for_session(session_id: string)`
- `prune_session_traces(before: i64)`

前端 API 建议新增：

- `src/lib/api/session-traces.ts`
- `src/lib/query/session-traces.ts`

UI 初版使用独立页面，不改现有 Session Manager：

- `App.tsx` 增加 `sessionTraces` view。
- 顶栏 Session Manager 的 `History` 图标旁新增 Session Traces 图标入口。
- 独立页面左侧为 trace session list。
- 独立页面右侧显示 Overview / Context / Traces / Usage。
- full 模式下显示 JSON viewer，并提供复制按钮。

## 8. 与现有 usage 统计的关系

`proxy_request_logs` 继续作为计费和统计事实表。

`session_traces` 是上下文观察表，应该满足：

- usage 写入失败不阻止 trace 写入。
- trace 写入失败不阻止 usage 写入。
- 两者都不能阻塞响应透传。
- 后续可以通过 `proxy_request_id`、`session_id`、`created_at` 做弱关联。

在代码上建议把 `spawn_log_usage` 与 `spawn_log_session_trace` 并列放置，但内部使用不同 service / DAO。

## 9. 测试计划

### Rust 单元测试

- `extract_system_prompt` 支持 string 与 content block array。
- `summarize_messages` 正确处理 text、image、tool_result、unknown content block。
- `summarize_tools` 只保留 name、description preview、input_schema keys。
- `redact_json` 能递归脱敏敏感 key。
- `extract_response_text` 覆盖 Claude / OpenAI Chat / Responses / Gemini 典型响应。
- `SseTraceCollector` 能从模拟 SSE events 重建文本与 tool calls。
- context category estimator 能从 request body 产出 system/messages/tools 分类 token。
- session context summary 能正确聚合多轮模型、usage、tools、skills。

### 集成测试

- 非流式 Claude 请求：写入一条 session_trace，字段完整。
- 流式 Claude 请求：SSE 结束后写入 response preview 和 usage。
- usage logging 关闭时：确认 trace mode 与 usage 开关的预期行为。
- full 模式：确认 `request_json` 已脱敏。
- 并发同 session 请求：turn_index 不重复或查询侧展示稳定。
- 多轮同 session：聚合 usage 与最后一轮 context 水位正确。

### 前端测试

- API 类型与命令参数序列化。
- 独立 Session Traces 页面空态、列表态、详情态。
- 长文本截断和复制操作。

## 10. 风险与处理

| 风险 | 影响 | 处理 |
| --- | --- | --- |
| 保存系统提示词和消息可能包含敏感数据 | 高 | 默认关闭，summary 需用户开启，full 模式二次确认 |
| 流式热路径额外 JSON parse | 中 | trace mode 为 off 时完全不解析；summary 限制保存长度 |
| SSE event filter 当前只保留 usage 事件 | 中 | trace collector 独立于 usage filter |
| 转换链路存在多种格式 | 中 | 优先记录“客户端收到的格式”，先覆盖 Anthropic / OpenAI 主路径 |
| 数据库膨胀 | 中 | preview limit、full text limit、prune 命令、未来可加 retention days |
| request_id 与 usage 去重逻辑不稳定 | 低 | trace 使用独立 trace_id，proxy_request_id 只做可选关联 |
| skills/memory/agents 来源标签不一定在 API body 中保留 | 中 | v1 标记为 estimated，v2 叠加本地 session 文件 / CLI metadata 解析 |
| token 分类估算与供应商真实 tokenizer 有差异 | 中 | UI 标注 estimated；关键 billing usage 仍以响应 usage 为准 |
| 用户误以为 Session Manager 会自动记录上下文 | 中 | Session Traces 独立入口 + 设置开关明确显示 Off/Summary/Full |

## 11. 推荐落地顺序

1. 先做 schema + DAO + request snapshot helper。
2. 接入非流式 trace，完成最小闭环。
3. 接入 Anthropic SSE trace collector。
4. 接入 OpenAI Chat / Responses SSE。
5. 增加 context category estimator 与 session 聚合 summary。
6. 增加 Tauri commands 与独立 Session Traces 页面。
7. 增加 pruning 与 full mode 设置。
8. 最后再做请求差异 UI。

最小可交付版本只需要完成 1 到 3：能在数据库中看到每个 session turn 的请求摘要、响应 preview、usage 和耗时，就已经覆盖 Hermes 会话中提出的核心价值。
