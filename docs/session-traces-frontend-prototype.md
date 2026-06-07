# Session Traces 独立页面原型与交互入口

> 目标：不影响现有 Session Manager 页面，在其入口旁边新增独立的 Session Traces 入口。Session Traces 作为单独分析页面，用于查看单个会话的 Context 质量、每轮请求 trace、usage、工具/Skills/MCP 统计；采集功能默认关闭，只有用户显式开启后才记录。

## 1. 当前客户端 UI 参考

现有入口是 `src/components/sessions/SessionManagerPage.tsx`，它的定位是本地会话浏览、搜索、恢复与删除。Session Traces 不应该改动这个页面的核心交互，而是借鉴它的布局密度与列表/详情模式，独立实现新页面。

- 左侧：会话列表、搜索、Provider 筛选、批量管理。
- 右侧：会话详情 header、恢复命令、消息时间线。
- 右侧大屏还有 `SessionTocSidebar`，移动端用浮动目录按钮。
- 消息区使用 `@tanstack/react-virtual` 做虚拟滚动，适合继续处理长会话。
- Usage 页面已有指标卡、request log table、request detail dialog，可复用信息密度和表格风格。
- 顶栏入口集中在 `src/App.tsx`，Session Manager 使用 `History` 图标按钮；Session Traces 可以放在它旁边，使用 `Activity`、`ChartSpline`、`FileSearch` 或 `ScanSearch` 这类分析含义的图标。

结论：Session Manager 保持不变；Session Traces 新增独立 view 与独立页面组件。

## 2. 信息架构

新增独立页面：`SessionTracesPage`。

```text
App Top Bar
├─ Session Manager 入口：History
└─ Session Traces 入口：Activity / ChartSpline

Session Traces Page
├─ Page Header
│  ├─ 标题 / 状态 / 开关提示
│  ├─ Refresh / Export / Settings
│  └─ Trace collection switch 状态
├─ 左侧 Trace Session List
│  ├─ 搜索
│  ├─ Provider / 时间 / 风险过滤
│  └─ 会话质量摘要
└─ 右侧 Analysis Detail
   ├─ Summary Header：session、project、model、context usage
   ├─ Tabs
   │  ├─ Overview      总览与质量信号
   │  ├─ Context       context 分类与加载项
   │  ├─ Traces        每轮 request / response trace
   │  └─ Usage         每轮模型 / token / cost / latency
   └─ Tab Content
```

页面默认选中最近有 trace 的会话；如果没有任何 trace 数据，显示开关引导和隐私说明。

## 3. 交互入口设计

### 3.1 顶栏入口

在现有 Session Manager 的 `History` 图标旁新增 `Session Traces` 图标。

建议接入点：

- `src/App.tsx` 的 `View` 类型新增 `sessionTraces`。
- `VALID_VIEWS` 新增 `"sessionTraces"`。
- `renderContent` 新增 `case "sessionTraces"`。
- 顶栏 session 入口旁新增按钮。

原型：

```text
[History]  Session Manager
[Activity] Session Traces
```

可见性：

- 与 Session Manager 同样只在支持会话的 app 上显示：Claude、Codex、OpenCode、OpenClaw、Gemini、Hermes。
- 如果 trace collection 关闭，入口仍可见，但页面显示“未开启采集”的空态。
- 不建议在 Session Manager 页面内插入跳转卡片，避免影响现有用户路径。

### 3.2 独立页面左侧列表

Session Traces 页面有自己的左侧列表，不复用 `SessionItem`，因为它展示的是“分析状态”而不是“恢复会话”：

```text
┌────────────────────────────────────┐
│ Claude icon  ay-d_front...   7%    │
│ 18 turns · $0.18 · 12 tools        │
│ Peak 74.1k · last 14:34 · healthy  │
└────────────────────────────────────┘
```

建议字段：

- `turns`：trace turn 数。
- `contextUsageRatio`：最后一轮 context 使用率。
- `totalCost`：session 聚合 cost。
- `toolCallCount`：实际工具调用次数。
- `peakContextUsedTokens`：峰值 context。
- `qualityState`：healthy / warning / critical。

展示策略：

- context 超过阈值：`>70%` 用 amber，`>90%` 用 red。
- 无 trace 数据：不进入列表，除非开启“显示无 trace 会话”过滤。
- 点击列表项只影响 Session Traces 页面内部选中态，不跳转 Session Manager。

### 3.3 Page Header 与采集状态

```text
Session Traces
Analyze context pressure, tool usage, and per-turn model usage.

[Trace collection: Off] [Enable]                       [Refresh] [Export] [Settings]
```

开关状态：

- `Off`：不记录新的 traces；页面只显示历史 traces。
- `Summary`：记录摘要、context stats、usage、tool calls。
- `Full`：记录脱敏后的完整 request/response JSON，需要二次确认。

快捷操作：

- `RefreshCw`：刷新 traces / context summary。
- `Download`：导出当前 session trace JSON。
- `Shield`：trace 隐私模式状态，点击打开设置或说明。
- `Settings`：打开设置页的 Session Traces 配置区。

## 4. 采集开关与权限设计

Session Traces 必须是 opt-in，默认关闭。

### 4.1 设置项

建议新增设置结构：

```ts
export type SessionTraceMode = "off" | "summary" | "full";

export interface SessionTraceSettings {
  enabled: boolean;
  mode: SessionTraceMode;
  retentionDays: number;
  maxResponseTextChars: number;
  captureRequestJson: boolean;
  captureResponseJson: boolean;
  redactSensitiveValues: boolean;
}
```

默认值：

```json
{
  "enabled": false,
  "mode": "off",
  "retentionDays": 14,
  "maxResponseTextChars": 2000,
  "captureRequestJson": false,
  "captureResponseJson": false,
  "redactSensitiveValues": true
}
```

### 4.2 设置入口

建议放在 Settings 的 Advanced tab，靠近 `LogConfigPanel`，但独立为 `SessionTraceConfigPanel`。

UI：

```text
Session Traces
Record per-session context and request traces for quality analysis.

[ ] Enable Session Traces

Mode
( ) Summary   Stores extracted fields, token estimates, tool calls, previews
( ) Full      Stores redacted request/response JSON for debugging

Retention: [14 days]
Response preview limit: [2000 chars]
[Prune old traces] [Open Session Traces]
```

### 4.3 开启确认

第一次开启时弹确认框：

```text
Enable Session Traces?

Session Traces may store system prompts, message previews, tool names,
tool inputs, and response previews locally on this device.

Sensitive keys are redacted, but you should only enable this on trusted devices.

[Cancel] [Enable Summary Mode]
```

切换到 Full mode 时再弹更强确认：

```text
Enable Full Trace Mode?

Full mode stores redacted request and response JSON. This can include
large prompts and tool payloads. Use it only for debugging.

[Cancel] [Enable Full Mode]
```

## 5. 桌面端布局原型

### 5.1 独立页面 Overview Tab

```text
┌────────────────────────────────────────────────────────────────────────────┐
│ Session Traces                                             Refresh Export   │
│ Trace collection: Summary · Retention 14d · 128 sessions                    │
├────────────────────────────────────────────────────────────────────────────┤
│ Search sessions...  Provider: All  Risk: All  Time: 7d                     │
├───────────────────────┬────────────────────────────────────────────────────┤
│ Trace Sessions         │ ay-d_front...                         Claude Code │
│ ┌───────────────────┐ │ Project ay-d_front · 18 turns · Last 14:34         │
│ │ ay-d_front  7%    │ │ [Overview] [Context] [Traces 18] [Usage]           │
│ │ 18 turns $0.18    │ ├────────────────────────────────────────────────────┤
│ │ healthy           │ │ Session Quality                                    │
│ └───────────────────┘ │ Context pressure  Low       Tool efficiency 8/61   │
│ ┌───────────────────┐ │ Cache hit rate    41%       Errors          0      │
│ │ cc-gateway 72%    │ │ Model stability   Stable    Stop max_tokens 0      │
│ │ 31 turns $0.42    │ │                                                    │
│ │ warning           │ │ Recent growth                                     │
│ └───────────────────┘ │ Turn 16  58.7k                                     │
│                       │ Turn 17  61.0k                                     │
│                       │ Turn 18  66.8k                                     │
└───────────────────────┴────────────────────────────────────────────────────┘
```

### 5.2 Context Tab

这是给用户判断“当前会话质量”的主视图。

```text
┌────────────────────────────────────────────────────────────────────────────┐
│ [Overview] [Context] [Traces 18] [Usage]                         Refresh   │
├────────────────────────────────────────────────────────────────────────────┤
│ Context Usage                                                             │
│ ┌──────────────────────────────────────────────────────────────────────┐   │
│ │ Opus 4.8 · claude-opus-4-8[1M]                     66.8k / 1M  7%   │   │
│ │ ███░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │   │
│ │ Free space 933.2k · Peak 74.1k · +5.8k last turn                     │   │
│ └──────────────────────────────────────────────────────────────────────┘   │
│                                                                            │
│ ┌─────────────────────────────┐ ┌──────────────────────────────────────┐   │
│ │ Estimated by category        │ │ Quality Signals                      │   │
│ │ System prompt     1.9k  0.2% │ │ Context pressure       Low           │   │
│ │ Skills            3.3k  0.3% │ │ Tool efficiency        8 / 61 used   │   │
│ │ Memory files      2.9k  0.3% │ │ Cache hit rate         41%           │   │
│ │ Tools schema      3.3k  0.3% │ │ Stop reasons           end_turn 17   │   │
│ │ Messages         55.4k  5.5% │ │ Errors                 0             │   │
│ └─────────────────────────────┘ └──────────────────────────────────────┘   │
│                                                                            │
│ Loaded Context Items                                                        │
│ [MCP tools 61] [Skills 28] [Agents 14] [Memory 7]                           │
│ ┌──────────────────────────────────────────────────────────────────────┐   │
│ │ mcp_mysql_list_tables                 mcp        0 tokens     loaded │   │
│ │ mcp_mysql_run_select_query             mcp        0 tokens     called │   │
│ │ docx                                  skill    ~120 tokens     loaded │   │
│ │ ~/.claude/CLAUDE.md                   memory   ~610 tokens     loaded │   │
│ └──────────────────────────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────────────────────────┘
```

交互：

- 点击 category 行：过滤下方 `Loaded Context Items`。
- 点击 `called` tool：跳到 `Traces` tab 并过滤相关 turn。
- 点击 `+5.8k last turn`：跳到 Usage tab 对比最近两轮。
- `Estimated` 标签始终可见，因为分类 token 是本地估算，不等于供应商计费 usage。

### 5.3 Traces Tab

用于查看每一轮真实 API request / response 摘要。

```text
┌────────────────────────────────────────────────────────────────────────────┐
│ [Overview] [Context] [Traces 18] [Usage]                                   │
├───────────────────────┬────────────────────────────────────────────────────┤
│ Turn List             │ Trace Detail                                       │
│ ┌───────────────────┐ │ Turn 12 · 14:28:11 · claude-opus-4-8 · 200 · SSE   │
│ │ #18 66.8k  200    │ │ Input 8.4k · Output 1.2k · Cache read 38.1k        │
│ │ #17 61.0k  200    │ ├────────────────────────────────────────────────────┤
│ │ #16 58.7k  tool   │ │ System Prompt                                      │
│ │ #15 52.1k  200    │ │ You are Claude Code...                             │
│ └───────────────────┘ │                                                    │
│                       │ Messages Summary                                   │
│ Filter: all/tool/error│ user: 1.2k · assistant history: 43.2k · tools: 3    │
│                       │                                                    │
│                       │ Tools                                               │
│                       │ mcp_mysql_run_select_query · mcp_context7_resolve   │
│                       │                                                    │
│                       │ Assistant Response                                  │
│                       │ 已完成。我分析了...                                 │
│                       │                                                    │
│                       │ [Summary] [Request JSON] [Response JSON] [Copy]     │
└───────────────────────┴────────────────────────────────────────────────────┘
```

交互：

- 左侧 turn list 复用虚拟列表，支持按 `tool / error / model / status` 过滤。
- 默认打开最新 turn。
- 详情内部用二级 tabs：`Summary / Request JSON / Response JSON / Tool Calls`。
- full mode 未开启时，JSON tab 显示空态和隐私说明。
- 点击 tool call 可展开 input preview / result preview。

### 5.4 Usage Tab

这是 session 维度的 request log table。

```text
┌────────────────────────────────────────────────────────────────────────────┐
│ Session Usage                                                              │
│ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────────┐  │
│ │ 18 turns      │ │ 412.8k input │ │ 38.4k output │ │ $0.184 total      │  │
│ └──────────────┘ └──────────────┘ └──────────────┘ └──────────────────┘  │
│                                                                            │
│ Model Breakdown                                                             │
│ claude-opus-4-8       12 turns    310k in    28k out   $0.14   7.2s avg    │
│ claude-sonnet-4-5      6 turns    102k in    10k out   $0.04   3.1s avg    │
│                                                                            │
│ Turn Usage                                                                  │
│ Turn  Time    Model              Input  Cache  Output  Cost   Latency  OK   │
│ 18    14:34   claude-opus-4-8    8.4k   38.1k  1.2k    $0.01  5.1s     200 │
│ 17    14:30   claude-opus-4-8    7.9k   35.8k  920     $0.01  4.8s     200 │
└────────────────────────────────────────────────────────────────────────────┘
```

交互：

- 点击 turn 行跳到 `Traces` tab 的对应 turn。
- 点击 model breakdown 行过滤 turn table。
- 支持复制 session usage summary。
- 支持导出 CSV。

## 6. 移动端布局

移动端不做左右分栏嵌套，使用独立页面纵向结构：

```text
Session Traces Header
Trace collection status
Search / Filters
Session list
Selected session summary
[Overview] [Context] [Traces] [Usage]

Context Usage compact card
Category list
Accordion: MCP tools
Accordion: Skills
Accordion: Memory
```

原则：

- Context tab 的 loaded items 用 accordion 分组。
- Traces tab 的 turn list 先显示，点 turn 后进入 detail panel 或 dialog。
- Usage tab 保留核心指标，表格横向滚动。

## 7. 数据类型原型

前端建议新增 `src/types/session-traces.ts`：

```ts
export interface SessionTraceSummary {
  traceId: string;
  sessionId: string;
  turnIndex: number;
  appType: string;
  providerId?: string;
  model?: string;
  requestModel?: string;
  statusCode?: number;
  isStreaming: boolean;
  createdAt: number;
  inputTokens: number;
  outputTokens: number;
  cacheReadTokens: number;
  cacheCreationTokens: number;
  latencyMs?: number;
  firstTokenMs?: number;
  toolCallCount: number;
  contextUsedTokens?: number;
  contextWindowTokens?: number;
  contextUsageRatio?: number;
}

export interface SessionContextSummary {
  sessionId: string;
  turnCount: number;
  lastModel?: string;
  contextWindowTokens?: number;
  contextUsedTokens?: number;
  contextUsageRatio?: number;
  peakContextUsedTokens?: number;
  totalInputTokens: number;
  totalOutputTokens: number;
  totalCacheReadTokens: number;
  totalCacheCreationTokens: number;
  totalCostUsd?: string;
  categories: ContextCategoryStat[];
  loadedItems: ContextItem[];
  modelBreakdown: SessionModelUsage[];
  qualitySignals: SessionQualitySignal[];
}

export interface ContextCategoryStat {
  key: "system_prompt" | "messages" | "tools" | "skills" | "memory_files" | "agents" | "other";
  label: string;
  tokens: number;
  ratio?: number;
  estimated: boolean;
}

export interface ContextItem {
  id: string;
  name: string;
  kind: "mcp_tool" | "skill" | "agent" | "memory_file" | "tool" | "other";
  source?: string;
  estimatedTokens?: number;
  loaded: boolean;
  calledCount?: number;
  lastTurnIndex?: number;
}

export interface SessionModelUsage {
  model: string;
  providerId?: string;
  turnCount: number;
  inputTokens: number;
  outputTokens: number;
  cacheReadTokens: number;
  cacheCreationTokens: number;
  totalCostUsd?: string;
  avgLatencyMs?: number;
}
```

## 8. 组件拆分建议

新增目录：`src/components/session-traces/`。

```text
session-traces/
├─ SessionTracesPage.tsx
├─ SessionTraceConfigPanel.tsx
├─ TraceSessionList.tsx
├─ TraceSessionItem.tsx
├─ TracePageHeader.tsx
├─ TraceAnalysisTabs.tsx
├─ SessionOverviewPanel.tsx
├─ SessionContextPanel.tsx
├─ ContextUsageBar.tsx
├─ ContextCategoryList.tsx
├─ ContextItemTable.tsx
├─ SessionTracePanel.tsx
├─ TraceTurnList.tsx
├─ TraceDetail.tsx
├─ TraceJsonViewer.tsx
├─ SessionUsagePanel.tsx
├─ SessionModelBreakdown.tsx
└─ SessionTurnUsageTable.tsx
```

集成方式：

- `SessionManagerPage.tsx` 不改。
- `App.tsx` 增加 `sessionTraces` view 和顶栏入口。
- `SessionTracesPage.tsx` 自己维护 selected trace session、filters、active tab。
- `SessionTraceConfigPanel.tsx` 放 Settings Advanced tab，管理采集开关。

## 9. Query 与 API 原型

新增 `src/lib/api/session-traces.ts`：

```ts
export const sessionTracesApi = {
  getSettings(): Promise<SessionTraceSettings>,
  setSettings(settings: SessionTraceSettings): Promise<void>,
  listSessions(filters: TraceSessionFilters): Promise<TraceSessionSummary[]>,
  list(sessionId: string): Promise<SessionTraceSummary[]>,
  get(traceId: string): Promise<SessionTraceDetail>,
  getContextSummary(sessionId: string): Promise<SessionContextSummary>,
  listTurnUsage(sessionId: string): Promise<SessionTurnUsage[]>,
  exportSession(sessionId: string): Promise<string>,
};
```

新增 query key：

- `["sessionTraceSettings"]`
- `["traceSessions", filters]`
- `["sessionTraces", sessionId]`
- `["sessionTrace", traceId]`
- `["sessionContextSummary", sessionId]`
- `["sessionTurnUsage", sessionId]`

刷新策略：

- 进入 `SessionTracesPage` 后先拉 settings 和 trace session list。
- 如果 settings 为 off，仍拉历史 trace session list，但展示“新采集关闭”提示。
- 切到 `Overview` 时拉 summary。
- 切到 `Context` 时拉 summary。
- 切到 `Traces` 时拉 trace list，选中 turn 后拉 detail。
- `Usage` tab 拉 turn usage。

## 10. 空态与错误态

### 无 traces

```text
No session traces yet
Trace collection starts after Session Traces is enabled.
[Open proxy settings]
```

中文：

```text
暂无 Session Traces
开启 Session Traces 后，新请求会出现在这里。
[开启 Session Traces]
```

### 采集关闭

```text
Session Traces is off
Existing traces remain available, but new requests will not be recorded.

[Enable summary mode] [Open settings]
```

### 只有 usage，无 context snapshot

显示 Usage tab 可用，Context tab 显示：

```text
此会话只有用量记录，没有上下文快照。
可能来自历史数据、会话日志同步，或当时未开启 Session Traces。
```

### full JSON 不可用

```text
Full request body was not stored
Current trace mode is summary. You can still inspect extracted fields.
```

## 11. i18n Key 建议

新增命名空间建议使用 `sessionTraces`，因为它是独立页面：

```json
{
  "sessionTraces": {
    "title": "Session Traces",
    "subtitle": "Analyze context pressure, tool usage, and per-turn model usage",
    "enable": "Enable Session Traces",
    "collectionOff": "Session Traces is off",
    "modeSummary": "Summary",
    "modeFull": "Full",
    "tabsOverview": "Overview",
    "tabsContext": "Context",
    "tabsTraces": "Traces",
    "tabsUsage": "Usage",
    "contextUsage": "Context Usage",
    "estimatedByCategory": "Estimated by category",
    "loadedContextItems": "Loaded Context Items",
    "qualitySignals": "Quality Signals",
    "turnUsage": "Turn Usage",
    "modelBreakdown": "Model Breakdown",
    "noSessionTraces": "No session traces yet",
    "openProxySettings": "Open proxy settings"
  }
}
```

## 12. 实现优先级

1. 增加 `sessionTraces` view、顶栏入口和空页面，不改 `SessionManagerPage`。
2. 增加 `SessionTraceConfigPanel` 与默认关闭的开关。
3. 做独立页面的 trace session list、空态、采集关闭态。
4. 做 Overview / Context 静态骨架。
5. 接入 `get_session_context_summary`，展示 context bar、category、quality signals。
6. 做 Usage tab，先复用 `proxy_request_logs` 的 session_id 聚合。
7. 做 Traces tab，接入 trace list/detail。
8. 增加导出、过滤、JSON viewer。

这个顺序可以保证现有 Session Manager 零干扰，同时让用户明确知道：只有打开 Session Traces，应用才会开始记录新的上下文分析数据。
