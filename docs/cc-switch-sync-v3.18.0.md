# cc-switch v3.18.0 同步说明

## 同步范围

本次以 `/Users/keane/www/cc-switch-ref` 的 `v3.16.5..v3.18.0` 为上游差异基线，
把 v3.17.0 与 v3.18.0 的产品功能、代理协议修复、用量修复和测试迁移到
CC Gateway Pro。上游文档、发布工作流和 CC Switch 品牌资源不直接覆盖本分叉。

主要同步内容：

- Grok Build 第八个受管应用，包括供应商、MCP、Skills、会话、用量与代理接管。
- xAI Grok OAuth 设备授权、多账号管理，以及 Claude/Codex 的受管路由。
- Codex 原生 ChatGPT 接管、Anthropic Messages 上游、Responses/Chat/Anthropic
  协议桥、模型目录兼容和供应商默认模型。
- Project Profiles、Claude Desktop 独立 profile scope 和切换后的状态刷新。
- Codex fork/子代理用量去重、稳定响应键、单飞同步、手动/自动重建用量。
- 缓存写入 token 语义、prompt cache 路由与 breakpoint、严格工具 schema、
  多轮推理和并行工具调用修复。
- 持久诊断日志、前端错误边界和敏感信息脱敏。
- Kimi K3、Grok 4.5、GPT-5.6/Hunyuan 定价与供应商预设更新。
- Windows 无黑窗切换、Node 版本管理器下的工具更新、托盘首启语言等修复。

## 分叉兼容处理

以下 CC Gateway Pro 能力继续保留，并接入上游的新应用与协议路径：

- Session Traces 旁路记录、7 天默认保留、启动/每日清理和大字段限界。
- Claude/Codex 项目路由与 Project Profiles 并存。
- Vision Model 独立路由与供应商 `meta.visionModel` 持久化。
- SQLite 用量日志、每日代理文本日志和应用诊断日志三套职责并存。
- Hermes、OpenClaw、OpenCode 的配置目录、供应商同步和专属界面。
- CC Gateway Pro 品牌、配置目录、deep link、更新器和本地存储键。
- 原版 cc-switch 供应商只读导入入口，并扩展支持 Grok Build、OpenClaw。
- Claude/Codex 项目路由使用的按项目会话查询命令。

## 数据库迁移

CC Gateway Pro 历史 schema v11 用于创建 `session_traces`，而上游 schema v11
用于增加用量计价字段和重建 daily rollup。直接采用上游 v16 会让现有分叉用户
跳过上游的 v10→v11 迁移。

本次将分叉 schema 提升到 v17，并把冲突迁移改成幂等兼容路径：

- 从现有 CC Gateway Pro v11 升级时保留 Session Trace 数据；
- 补齐 `pricing_model`、`request_model` 和 input token semantics；
- 继续创建 Profiles、Grok Build proxy/MCP/Skills 字段；
- 只重建 Codex session 来源的用量，不影响代理来源和 Session Traces；
- v16→v17 再次校准分叉专属表，兼容从上游数据库导入的场景。
- 分别覆盖 fork v11 与 upstream v11 的迁移测试，验证 Trace 和 rollup 数据保留。
- v15→v16 破坏性用量重建前若备份失败，停止迁移并保留原库。

## 安全收口

- Unix 配置目录、日志目录和备份目录强制为 `0700`，数据库、备份和日志文件
  强制为 `0600`，启动时同时修复已有日志权限。
- 上游错误正文、自由文本诊断、Session Trace 的 system/response/tool call
  在持久化前统一限长并脱敏。
- xAI OAuth 仅允许在 `127.0.0.1`、`::1` 或 `localhost` 监听时使用，避免把
  本机订阅令牌暴露给局域网请求。

仍保留的上游行为：

- 普通 API Key 代理的非回环监听仍没有独立的客户端认证层。需要局域网使用时，
  应由用户自行配置可信网络边界；后续若引入网关令牌，需要同时迁移所有受管客户端。
- Grok Build 官方一键安装仍调用 xAI 官方远程安装脚本，未固定脚本哈希。
- Windows 自定义共享配置目录的 ACL 仍依赖操作系统继承规则。

## 验证

已完成：

- Rust 格式检查、全目标 Clippy（warnings as errors）与全量测试；
- Rust 库测试 `2128 passed / 2 ignored`，集成测试 `118 passed`；
- TypeScript 类型检查与 Prettier；
- Vitest `80` 个测试文件、`500` 项测试全部通过；
- JSON 解析、冲突标记、前后端 invoke 注册和 `git diff --check`；
- Session Trace、Vision Model、项目路由、日志入口、同步命名空间和品牌配置静态回归。
