# CC-Gateway-Pro 架构与核心流程

本文根据当前代码实现整理 CC-Gateway-Pro 的运行方式，重点说明本项目相对原始 cc-switch 的增强：本地代理网关、项目级 Provider 绑定、Vision Model 自动路由、故障转移、用量统计、Session Traces 和多应用配置管理。

> 项目来源说明：CC-Gateway-Pro 基于 [farion1231/cc-switch](https://github.com/farion1231/cc-switch) fork 后继续开发。原项目的核心价值是可视化管理和切换 AI 编程工具的供应商配置；本项目在此基础上扩展了本地网关、代理接管、请求转换、高可用、用量分析以及更多应用集成。

## 总体架构

![本地 AI Provider 网关架构原理图](assets/diagrams/cc-gateway-pro-architecture-zh.svg)

```mermaid
flowchart TB
    subgraph Clients["客户端 / AI 编程工具"]
        Claude["Claude Code"]
        ClaudeDesktop["Claude Desktop"]
        Codex["Codex CLI"]
        Gemini["Gemini CLI"]
        OpenCode["OpenCode"]
        OpenClaw["OpenClaw"]
        Hermes["Hermes Agent"]
    end

    subgraph Gateway["本地网关"]
        direction TB

        subgraph UI["管理界面层"]
            ProviderUI["Provider 管理"]
            ProxyUI["代理 / 故障转移"]
            UsageUI["用量仪表盘"]
            ExtUI["MCP / Prompts / Skills"]
            SessionUI["Sessions / Workspace"]
        end

        subgraph Core["核心能力层"]
            ConfigWrite["配置写入\nJSON / TOML / env"]
            ProviderSelect["Provider 选择\n当前 / 队列"]
            ProjectRoute["项目级路由\nsession -> cwd -> provider"]
            VisionRoute["Vision Model 路由\nimage -> vision_model"]
            Transform["协议转换\nAnthropic / OpenAI / Gemini"]
            Failover["高可用\nRetry / Circuit Breaker"]
        end

        subgraph Governance["治理与观测层"]
            UsageLog["请求日志 / Token / 成本 / Trace"]
            Health["Provider 健康状态"]
            Backup["原子写入 / 自动备份"]
            Sync["目录迁移 / WebDAV 同步"]
        end
    end

    subgraph Foundation["基础支撑层"]
        DB[("SQLite\nproviders / settings / logs / health")]
        Settings["settings.json\n设备级设置"]
        AppFiles["应用配置文件\n~/.claude / ~/.codex / ~/.gemini / ..."]
        SessionFiles["会话文件\nClaude JSONL / Codex sessions"]
    end

    subgraph Services["上游 AI 服务"]
        Official["官方 API\nAnthropic / OpenAI / Google"]
        Relay["第三方中转\nOpenRouter / API 聚合 / 私有网关"]
        OAuth["OAuth / Copilot / Codex Auth"]
    end

    Clients -->|配置读取或本地代理请求| Gateway
    UI --> Core
    Core --> Governance
    Core --> Foundation
    Governance --> Foundation
    Core -->|HTTP 转发| Services
    Services -->|响应 / SSE| Core
    Governance -->|状态展示| UI
```

### 架构特点

| 特点       | 说明                                                                  |
| ---------- | --------------------------------------------------------------------- |
| 统一接入   | 多个 AI 编程工具在同一个桌面应用中管理 Provider、MCP、Prompts、Skills |
| 智能路由   | 代理链路中按故障转移、项目绑定、Vision Model 和模型映射决定最终上游   |
| 安全可恢复 | 接管前备份原配置，写入使用原子流程，关闭接管时恢复                    |
| 高可用     | 每个应用有独立故障转移队列、熔断器、重试与超时配置                    |
| 可观测     | 记录请求日志、Token、成本、延迟、错误、Provider 健康状态和会话 Trace |
| 可扩展     | 支持 Provider 预设、Deep Link、WebDAV、Skills 仓库和多应用同步        |

## 关键流程图索引

| 流程图                                                            | 解决的问题                                |
| ----------------------------------------------------------------- | ----------------------------------------- |
| [本地代理请求流程](#本地代理请求流程)                             | 说明一次请求如何从客户端进入代理并转发    |
| [Vision Model 代理工作原理](#vision-model-代理工作原理)           | 说明图片请求如何自动切换到视觉模型        |
| [Project Provider 代理工作原理](#project-provider-代理工作原理)   | 说明 Claude/Codex 如何按项目绑定 Provider |
| [故障转移与熔断](#故障转移与熔断)                                 | 说明高可用队列、熔断和恢复机制            |
| [MCP / Prompts / Skills 同步流程](#mcp--prompts--skills-同步流程) | 说明扩展能力如何统一管理并同步到各应用    |
| [用量统计流程](#用量统计流程)                                     | 说明 Token、成本和日志如何沉淀到仪表盘    |
| [Session Traces](#session-traces)                                 | 说明上下文 Trace 如何按用户显式开启采集   |

### 主要模块

| 模块           | 代码位置                                                                              | 作用                                                       |
| -------------- | ------------------------------------------------------------------------------------- | ---------------------------------------------------------- |
| 前端界面       | `src/components`, `src/hooks`, `src/lib/api`                                          | 管理供应商、代理、故障转移、用量、MCP、Prompts、Skills     |
| Tauri 命令     | `src-tauri/src/commands`                                                              | 暴露 Provider、Proxy、Project Routing、Settings 等后端能力 |
| 数据库         | `src-tauri/src/database`                                                              | 保存供应商、设置、请求日志、Session Traces、健康状态、备份信息 |
| 代理服务       | `src-tauri/src/proxy`                                                                 | 接收本地请求、选择 Provider、转换格式、转发、记录日志与 Trace |
| Provider 配置  | `src-tauri/src/services/provider`, `src/config/*ProviderPresets.ts`                   | 维护不同应用的预设和配置写入逻辑                           |
| 会话与项目路由 | `src-tauri/src/proxy/session_project_router.rs`, `src-tauri/src/proxy/project_router` | 从 Claude/Codex 会话文件识别项目路径并匹配 Provider        |

## Provider 配置流程

```mermaid
sequenceDiagram
    participant User as 用户
    participant UI as 前端 Provider 表单
    participant Cmd as Tauri Provider 命令
    participant DB as SQLite
    participant Writer as 配置写入器
    participant App as AI CLI / Desktop App

    User->>UI: 选择预设或自定义 Provider
    UI->>Cmd: add/update provider
    Cmd->>DB: 保存 Provider JSON、meta、排序和当前状态
    User->>UI: 点击启用
    UI->>Cmd: switch provider
    Cmd->>DB: 更新当前 Provider
    Cmd->>Writer: 按应用生成配置
    Writer->>App: 原子写入 JSON/TOML/.env 等配置文件
```

非代理模式下，切换 Provider 的本质是改写对应应用自己的配置文件。正在运行的 CLI 是否立即生效取决于该 CLI 是否会重新读取配置；代理接管模式下，Provider 切换由本地代理立即生效。

## 本地代理请求流程

```mermaid
flowchart TD
    Req["客户端请求\n/v1/messages, /v1/chat/completions, Gemini endpoint"] --> Ctx["创建 RequestContext\n读取应用代理配置、超时、整流器"]
    Ctx --> Select["ProviderRouter.select_providers"]
    Select --> Failover{"自动故障转移开启?"}
    Failover -- 否 --> Current["使用当前 Provider"]
    Failover -- 是 --> Queue["按故障转移队列过滤\n跳过熔断 Provider"]
    Current --> Project["项目级 Provider 检查"]
    Queue --> Project
    Project --> Vision["Vision Model 检查"]
    Vision --> Forward["Forwarder 重试转发"]
    Forward --> Transform{"Provider 需要格式转换?"}
    Transform -- 是 --> Convert["Anthropic / OpenAI / Responses / Gemini 转换"]
    Transform -- 否 --> Pass["透传响应"]
    Convert --> Usage["解析 Token、记录用量、更新健康状态"]
    Pass --> Usage
    Usage --> Resp["返回客户端"]
```

代理服务默认监听 `127.0.0.1:15721`。启动后可按应用启用接管，CC-Gateway-Pro 会将 Claude、Claude Desktop、Codex 或 Gemini 的端点指向本地代理，并在关闭接管时恢复原配置。

## Project Provider 代理工作原理

项目级绑定目前用于 Claude Code 和 Codex。它的核心不是猜当前目录，而是从各应用生成的会话记录中反查 `session_id -> cwd`，再用 `cwd -> provider_id` 的用户绑定决定实际 Provider。

![Project Provider 代理工作原理图](assets/diagrams/project-provider-proxy-flow-zh.svg)

```mermaid
flowchart TB
    subgraph Client["客户端请求"]
        Req["Claude / Codex 请求"]
        Header["headers / body 中提取 session_id"]
    end

    subgraph Router["Project Router"]
        Cache{"session -> cwd\n缓存命中?"}
        Scan["增量扫描会话文件"]
        Cwd["得到项目路径 cwd"]
        Bind["读取项目绑定表\nClaude: project_providers\nCodex: project_providers_codex"]
        Match{"路径匹配成功?"}
        Direct["精确匹配"]
        Canon["canonical 匹配"]
        Prefix["父子目录前缀匹配"]
    end

    subgraph Provider["Provider 决策"]
        Base["基础 Provider\n当前 Provider 或故障转移队列首个"]
        Override["项目绑定 Provider 覆盖本次请求"]
        Fallback["未命中则保持基础 Provider"]
    end

    subgraph Forward["代理转发"]
        Vision["继续执行 Vision 路由"]
        Transform["协议转换 / 模型映射"]
        Upstream["发送到上游 Provider"]
    end

    Req --> Header --> Cache
    Cache -- 是 --> Cwd
    Cache -- 否 --> Scan --> Cwd
    Cwd --> Bind --> Match
    Match -- 是 --> Direct --> Override
    Match -- 是 --> Canon --> Override
    Match -- 是 --> Prefix --> Override
    Match -- 否 --> Fallback
    Base --> Fallback
    Override --> Vision
    Fallback --> Vision
    Vision --> Transform --> Upstream
```

### 匹配规则

1. 精确匹配保存的项目路径。
2. 对项目路径做 canonicalize 后再匹配。
3. 使用前缀匹配处理父目录/子目录绑定。

如果没有匹配到项目绑定，请求会继续使用当前 Provider 或故障转移队列选择的 Provider。项目绑定发生在 Vision Model 检查之前，因此不同项目可以绑定不同 Provider，并分别使用自己的 `vision_model`、模型映射、API 格式和密钥。

### Project Provider 工作流

```mermaid
sequenceDiagram
    participant CLI as Claude/Codex
    participant Proxy as 本地代理
    participant Router as Project Router
    participant DB as SQLite Settings
    participant Provider as 上游 Provider

    CLI->>Proxy: 请求携带 session_id
    Proxy->>Router: 查询 session 对应项目
    Router->>Router: 缓存未命中则扫描会话文件
    Router->>DB: 读取项目路径与 Provider 绑定
    DB-->>Router: provider_id 或空
    Router-->>Proxy: 返回项目 Provider 或未命中
    Proxy->>Proxy: 项目 Provider 覆盖基础 Provider
    Proxy->>Provider: 转发请求
    Provider-->>Proxy: 返回响应
    Proxy-->>CLI: 返回结果
```

## Vision Model 代理工作原理

Provider 的 `meta.vision_model` 是 CC-Gateway-Pro 扩展字段。代理会递归检查请求体中的图片内容，支持 Anthropic、OpenAI Chat、OpenAI Responses 以及嵌套 tool_result 中的图片块。

![Vision Model 代理工作原理图](assets/diagrams/vision-model-proxy-flow-zh.svg)

```mermaid
flowchart TD
    subgraph Input["客户端输入"]
        User["用户请求\n文本 / 图片 / 工具结果"]
        Body["请求体 JSON\nmessages 或 input"]
    end

    subgraph Detect["图片识别层"]
        Walk["递归扫描 content"]
        Anthropic["Anthropic image"]
        Chat["OpenAI image_url"]
        Responses["Responses input_image"]
        Tool["tool_result 嵌套图片"]
    end

    subgraph Route["Vision 路由层"]
        HasImage{"发现图片内容?"}
        HasVision{"Provider 配置 vision_model?"}
        Replace["替换 body.model\nmodel -> vision_model"]
        Keep["保持原模型"]
        Preserve["后续模型映射跳过 vision_model\n避免被默认模型覆盖"]
    end

    subgraph Upstream["上游调用"]
        Transform["必要时转换 API 格式"]
        Send["发送到最终 Provider"]
        Log["记录原始模型 / 实际模型 / 用量"]
    end

    User --> Body --> Walk
    Walk --> Anthropic --> HasImage
    Walk --> Chat --> HasImage
    Walk --> Responses --> HasImage
    Walk --> Tool --> HasImage
    HasImage -- 否 --> Keep
    HasImage -- 是 --> HasVision
    HasVision -- 否 --> Keep
    HasVision -- 是 --> Replace --> Preserve
    Keep --> Transform
    Preserve --> Transform
    Transform --> Send --> Log
```

这意味着同一个 Provider 可以默认使用更便宜或更快的文本模型，而在用户粘贴图片、截图或工具返回图片时自动切换到视觉模型。

### Vision Model 工作流

```mermaid
sequenceDiagram
    participant Client as Claude/Codex 请求
    participant Proxy as 本地代理
    participant Mapper as ModelMapping
    participant Provider as 上游 Provider
    participant Log as Usage Logger

    Client->>Proxy: 请求 model=默认文本模型，content 含图片
    Proxy->>Mapper: 检查 messages/input/content
    Mapper-->>Proxy: has_images=true
    Proxy->>Proxy: 读取当前 Provider meta.vision_model
    alt 已配置 vision_model
        Proxy->>Proxy: body.model 替换为 vision_model
        Proxy->>Provider: 使用视觉模型转发请求
    else 未配置 vision_model
        Proxy->>Provider: 保持原模型转发
    end
    Provider-->>Proxy: 返回响应
    Proxy->>Log: 记录请求模型、实际模型、Token、成本
    Proxy-->>Client: 返回响应
```

## 故障转移与熔断

![故障转移与用量观测工作流](assets/diagrams/failover-usage-flow-zh.svg)

```mermaid
stateDiagram-v2
    [*] --> Closed: Provider 初始可用
    Closed --> Open: 连续失败或错误率达到阈值
    Open --> HalfOpen: 等待恢复时间到期
    HalfOpen --> Closed: 探测请求成功达到恢复阈值
    HalfOpen --> Open: 探测请求失败
```

```mermaid
flowchart TD
    Start["请求开始"] --> Enabled{"应用自动故障转移开启?"}
    Enabled -- 否 --> One["只使用当前 Provider"]
    Enabled -- 是 --> Queue["读取该应用故障转移队列"]
    Queue --> Filter["过滤 Open/HalfOpen 不可用 Provider"]
    Filter --> Try["按顺序尝试 Provider"]
    Try --> OK{"成功?"}
    OK -- 是 --> Success["记录成功和健康状态"]
    OK -- 否 --> Fail["记录失败、更新熔断器"]
    Fail --> Next{"还有可用 Provider?"}
    Next -- 是 --> Try
    Next -- 否 --> Error["返回代理错误"]
```

故障转移是按应用独立配置的。开启后，代理只使用故障转移队列中的 Provider，并按队列顺序尝试；关闭时只使用当前 Provider。

## MCP / Prompts / Skills 同步流程

CC-Gateway-Pro 不只管理 API Provider，也统一管理 MCP、Prompts 和 Skills。它们的共同点是：先进入数据库或统一存储，再同步到各应用自己的 live 配置文件或目录。

![MCP / Prompts / Skills 统一同步原理图](assets/diagrams/extensions-sync-flow-zh.svg)

```mermaid
flowchart TB
    subgraph Sources["输入来源"]
        UI["用户在界面创建 / 编辑"]
        DeepLink["ccgatewaypro:// Deep Link 导入"]
        Repo["GitHub / ZIP / skills.sh 仓库"]
        Existing["从现有应用配置回填"]
    end

    subgraph Center["统一管理层"]
        DB[("SQLite\nMCP / Prompts / Skills 元数据")]
        SkillStore["~/.cc-gateway-pro/skills\n统一 Skills 存储"]
        Policy["应用同步开关\nClaude / Codex / Gemini / OpenCode / Hermes"]
        Backup["卸载前备份 / 回填保护"]
    end

    subgraph Apps["应用侧配置"]
        Claude["Claude\nMCP / CLAUDE.md / Skills"]
        Codex["Codex\nMCP / AGENTS.md / Skills"]
        Gemini["Gemini\nMCP / GEMINI.md / Skills"]
        OpenCode["OpenCode\nMCP / AGENTS.md / Skills"]
        Hermes["Hermes\nMCP / Skills"]
    end

    Sources --> DB
    Repo --> SkillStore
    DB --> Policy
    SkillStore --> Policy
    Policy --> Backup
    Backup --> Claude
    Backup --> Codex
    Backup --> Gemini
    Backup --> OpenCode
    Backup --> Hermes
    Apps -->|回填 / 导入| Existing
```

### 扩展同步要点

| 能力      | 同步方式                                                        |
| --------- | --------------------------------------------------------------- |
| MCP       | 保存统一定义后，按应用写入对应 MCP 配置                         |
| Prompts   | Markdown 预设激活后同步到 `CLAUDE.md`、`AGENTS.md`、`GEMINI.md` |
| Skills    | 默认集中存储到 `~/.cc-gateway-pro/skills`，再按应用软链或复制   |
| Deep Link | 通过 `ccgatewaypro://` 导入 Provider、MCP、Prompts、Skills      |

## 用量统计流程

```mermaid
sequenceDiagram
    participant Proxy as 本地代理
    participant Upstream as 上游 Provider
    participant Parser as Usage Parser
    participant DB as SQLite
    participant UI as 用量仪表盘

    Proxy->>Upstream: 转发流式或非流式请求
    Upstream-->>Proxy: 返回响应 / SSE
    Proxy->>Parser: 解析 usage、模型、状态码、延迟
    Parser->>DB: 写入 request log 和 rollup
    UI->>DB: 查询趋势、模型统计、Provider 统计、请求详情
```

代理会尽量从不同 Provider 的响应结构中提取 token 用量；无法直接获取时，会保留请求日志和状态信息，供后续统计或定价补录。

## Session Traces

Session Traces 是独立于普通 usage 统计的上下文观察能力。它默认关闭，只有用户在「设置 → 高级 → Session Traces」或独立 Session Traces 页面显式开启后，才会记录新的会话上下文摘要、工具调用和每轮 usage。

```mermaid
flowchart TD
    Req["代理请求进入"] --> Enabled{"Session Traces 已开启?"}
    Enabled -- 否 --> UsageOnly["仅保留普通 usage / request log"]
    Enabled -- 是 --> Mode{"采集模式"}
    Mode -- Summary --> Summary["提取字段\ncontext 分类 / 工具调用 / usage / 响应预览"]
    Mode -- Full --> Full["保存脱敏 JSON\nrequest / response"]
    Summary --> Redact["敏感 key 脱敏"]
    Full --> Redact
    Redact --> DB[("session_traces")]
    DB --> UI["Session Traces 页面\nOverview / Context / Traces / Usage"]
```

| 项目     | 行为                                                                 |
| -------- | -------------------------------------------------------------------- |
| 默认状态 | 关闭；历史 usage 仍可查看，但不会新增上下文 Trace                    |
| Summary  | 保存提取后的字段、上下文分类、工具/Skills/MCP 统计和响应预览         |
| Full     | 在脱敏后保存 request/response JSON，适合可信设备上的深度排查         |
| 保留策略 | 默认保留 14 天，响应预览默认截断到 2000 字符                         |
| 数据边界 | 数据保存在本机 SQLite；Session Traces 开关独立于普通代理日志开关     |

## 数据与备份

| 数据        | 默认位置                              |
| ----------- | ------------------------------------- |
| 主数据库    | `~/.cc-gateway-pro/cc-gateway-pro.db` |
| 设备级设置  | `~/.cc-gateway-pro/settings.json`     |
| 自动备份    | `~/.cc-gateway-pro/backups/`          |
| Skills 存储 | `~/.cc-gateway-pro/skills/`           |
| Skills 备份 | `~/.cc-gateway-pro/skill-backups/`    |

应用配置写入尽量采用原子写入和备份恢复机制。代理接管会记录接管前配置，关闭代理或关闭对应应用接管时恢复。

## 当前实现边界

- 项目级 Provider 绑定主要覆盖 Claude Code 和 Codex。
- Vision Model 自动路由依赖 Provider 配置中的 `vision_model`，未配置时不会自动猜测视觉模型。
- 故障转移只在代理模式下工作；非代理模式仍然是直接写入应用配置文件。
- OpenCode、OpenClaw、Hermes 目前主要由配置管理、扩展管理和会话/工作区能力支撑，不走完整的本地代理接管链路。
