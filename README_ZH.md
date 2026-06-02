<div align="center">

# CC-Gateway-Pro

### 面向 Claude Code、Claude Desktop、Codex、Gemini CLI、OpenCode、OpenClaw 和 Hermes 的本地 AI Provider 网关

[![Version](https://img.shields.io/github/v/release/KeaneFeng/cc-gateway-pro?color=blue&label=version)](https://github.com/KeaneFeng/cc-gateway-pro/releases)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/KeaneFeng/cc-gateway-pro/releases)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-orange.svg)](https://tauri.app/)
[![Fork of cc-switch](https://img.shields.io/badge/fork%20of-farion1231%2Fcc--switch-blue.svg)](https://github.com/farion1231/cc-switch)

[English](README.md) | 中文 | [日本語](README_JA.md) | [更新日志](CHANGELOG.md)

</div>

## CC-Gateway-Pro 是什么？

CC-Gateway-Pro 是一个桌面控制台和本地 AI Provider Gateway。它可以管理不同 AI 编程工具的 Provider 配置，写入各工具原生配置文件，也可以通过本地代理接管实时请求，实现日志记录、故障转移、模型转换、项目级 Provider 绑定和 Vision Model 自动路由。

本项目基于 [farion1231/cc-switch](https://github.com/farion1231/cc-switch) fork 后继续开发。CC-Gateway-Pro 保留了原项目“可视化切换供应商”的核心思路，并扩展出 Rust/Tauri 本地网关、多应用配置管理、请求路由、用量统计、同步能力和更多 Provider 适配。

## 支持的应用

| 应用           | 主要能力                                                                                  |
| -------------- | ----------------------------------------------------------------------------------------- |
| Claude Code    | Provider 切换、本地代理接管、项目路由、Vision Model 路由、MCP、Prompts、Skills、Sessions  |
| Claude Desktop | 官方与第三方配置、直连模式、本地路由模式、模型映射                                        |
| Codex          | Provider 切换、本地代理接管、项目路由、OAuth/Copilot 辅助、MCP、Prompts、Skills、Sessions |
| Gemini CLI     | Provider 切换、本地代理接管、MCP、Prompts、Skills、Sessions                               |
| OpenCode       | Provider 预设、通用配置片段、MCP、Prompts、Skills、Sessions                               |
| OpenClaw       | Provider 预设、工作区文件、Agent 默认配置、工具和环境变量面板                             |
| Hermes Agent   | Provider 预设、Memory 面板、MCP 与 Skills 管理                                            |

## 核心能力

- **Provider 管理**：50+ 预设、自定义 Provider、拖拽排序、导入导出、端点测速、余额与配额辅助。
- **本地代理网关**：默认监听 `127.0.0.1:15721`，基于 Tauri/Rust + Axum，为 Claude、Claude Desktop、Codex、Gemini 提供应用级代理接管。
- **API 格式适配**：支持 Anthropic、OpenAI Chat Completions、OpenAI Responses、Gemini 以及供应商专有兼容路径。
- **项目级路由**：从 Claude/Codex 本地会话文件识别项目目录，将指定项目绑定到指定 Provider。
- **Vision Model 路由**：当请求包含图片内容时，自动切换到当前 Provider 配置的 `vision_model`。
- **高可用策略**：应用级故障转移队列、熔断、重试、健康状态、流式与非流式超时控制。
- **用量统计**：请求日志、Token 用量、成本估算、趋势、Provider/模型拆分和自定义计价。
- **MCP、Prompts、Skills**：统一管理并同步到支持的应用，支持 Deep Link、仓库和 ZIP 技能安装。
- **数据安全**：SQLite 存储、原子写入、自动备份、可迁移数据目录和 WebDAV 同步。

## 原理图

### 本地 AI Provider 网关架构

![本地 AI Provider 网关架构原理图](docs/assets/diagrams/cc-gateway-pro-architecture-zh.svg)

### Vision Model 代理工作原理

![Vision Model 代理工作原理图](docs/assets/diagrams/vision-model-proxy-flow-zh.svg)

### Project Provider 代理工作原理

![Project Provider 代理工作原理图](docs/assets/diagrams/project-provider-proxy-flow-zh.svg)

故障转移、用量统计、MCP/Prompts/Skills 同步和完整代理链路说明见 [架构与核心流程](docs/architecture-and-flows-zh.md)。

## 界面预览

|                  主界面                   |                  添加供应商                  |
| :---------------------------------------: | :------------------------------------------: |
| ![主界面](assets/screenshots/main-zh.png) | ![添加供应商](assets/screenshots/add-zh.png) |

|                                项目路由                                |                                  Vision Model 配置                                  |
| :--------------------------------------------------------------------: | :---------------------------------------------------------------------------------: |
| ![项目路由界面预览](assets/screenshots/project-routing-preview-zh.svg) | ![Vision Model 配置界面预览](assets/screenshots/vision-model-config-preview-zh.svg) |

## 下载安装

从 [GitHub Releases](https://github.com/KeaneFeng/cc-gateway-pro/releases) 下载最新版本。

系统要求：

- macOS 12 及以上
- Windows 10 及以上
- Linux 需要较新的 WebKitGTK 运行时，例如 Ubuntu 22.04+、Debian 11+ 或 Fedora 34+

### macOS Homebrew

不少用户反馈 Homebrew 未刷新时会找不到 cask 或安装到旧版本，建议先执行 `brew update`：

```bash
brew update
brew tap KeaneFeng/cc-gateway-pro
brew install --cask cc-gateway-pro
```

更新：

```bash
brew update
brew upgrade --cask cc-gateway-pro
```

### 手动下载

- Windows：从 [Releases](https://github.com/KeaneFeng/cc-gateway-pro/releases) 下载 `.msi` 安装包或便携版 `.zip`。
- macOS：从 [Releases](https://github.com/KeaneFeng/cc-gateway-pro/releases) 下载 `.dmg`。
- Linux：从 [Releases](https://github.com/KeaneFeng/cc-gateway-pro/releases) 下载 `.deb`、`.rpm` 或 `.AppImage`。
- Arch Linux：使用 `paru -S cc-gateway-pro-bin` 安装。

## 从源码构建

需要 Node.js 22+、pnpm 和 Rust stable。

```bash
pnpm install
pnpm tauri dev
pnpm tauri build
```

也可以使用仓库内的构建脚本：

```bash
./build.sh --dev
./build.sh --release
./build.sh --dmg
./build.sh --sha
```

## 文档

- [English README](README.md)
- [日本語 README](README_JA.md)
- [用户手册](docs/user-manual/zh/README.md)
- [架构与核心流程](docs/architecture-and-flows-zh.md)
- [代理使用指南](docs/proxy-guide-zh.md)
- [发布说明](docs/release-notes/v3.15.0-zh.md)

## 数据位置

- 主数据库：`~/.cc-gateway-pro/cc-gateway-pro.db`
- 设备设置：`~/.cc-gateway-pro/settings.json`
- 自动备份：`~/.cc-gateway-pro/backups/`
- Skills：`~/.cc-gateway-pro/skills/`

可以在设置页把数据目录迁移到 Dropbox、OneDrive、iCloud、NAS 或其他同步目录。

## 致谢

- 基于 [farion1231/cc-switch](https://github.com/farion1231/cc-switch) fork 后继续开发
- 使用 Tauri、Rust、React、TypeScript 和 Tailwind CSS 构建

## License

MIT © Keane Feng
