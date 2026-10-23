<div align="center">

# CC-Gateway-Pro

### Local AI Provider Gateway for Claude Code, Claude Desktop, Codex, Gemini CLI, OpenCode, OpenClaw and Hermes

[![Version](https://img.shields.io/github/v/release/KeaneFeng/cc-gateway-pro?color=blue&label=version)](https://github.com/KeaneFeng/cc-gateway-pro/releases)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/KeaneFeng/cc-gateway-pro/releases)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-orange.svg)](https://tauri.app/)

English | [中文](README_ZH.md) | [日本語](README_JA.md) | [Svenska](README_SV.md) | [Español](README_ES.md) | [Changelog](CHANGELOG.md)

</div>

## What Is CC-Gateway-Pro?

CC-Gateway-Pro is a desktop control plane and local AI Provider Gateway for AI coding tools. It manages provider profiles, writes each tool's native configuration files, and can route live requests through a local proxy for request logging, session traces, failover, model transformation, project-level provider binding, and Vision Model routing.

CC-Gateway-Pro manages local providers, project-level routing, Vision Model routing and request observability for AI coding tools. It references [cc-switch](https://github.com/farion1231/cc-switch); reused code and upstream contributions are documented in the project history. See [Project history and attribution](docs/project-history.md) and [verified contributions](docs/original-contributions.md).

## Supported Apps

| App            | Main capabilities                                                                                                |
| -------------- | ---------------------------------------------------------------------------------------------------------------- |
| Claude Code    | Provider switching, local proxy takeover, project routing, Vision Model routing, MCP, prompts, skills, sessions  |
| Claude Desktop | Official and third-party profiles, direct mode, local routed mode, model route mapping                           |
| Codex          | Provider switching, local proxy takeover, project routing, OAuth/Copilot helpers, MCP, prompts, skills, sessions |
| Gemini CLI     | Provider switching, local proxy takeover, MCP, prompts, skills, sessions                                         |
| Grok Build     | Provider switching, local proxy takeover, quota, failover, MCP, prompts, skills, sessions                        |
| OpenCode       | Provider presets, common config snippets, MCP, prompts, skills, sessions                                         |
| OpenClaw       | Provider presets, workspace files, agent defaults, tool and environment panels                                   |
| Hermes Agent   | Provider presets, memory panel, MCP and skills management                                                        |

## Core Features

- **Provider management**: 50+ presets, custom providers, drag sorting, import/export, endpoint speed tests, balance and quota helpers.
- **Local proxy gateway**: Tauri/Rust + Axum proxy on `127.0.0.1:15721` by default, with per-app takeover for Claude, Claude Desktop, Codex and Gemini.
- **API format adaptation**: Anthropic, OpenAI Chat Completions, OpenAI Responses, Gemini and provider-specific compatibility paths.
- **Project-level routing**: Claude and Codex sessions can be mapped from local session files to project directories, then routed to project-bound providers.
- **Vision Model routing**: requests containing image blocks can automatically switch to the provider's configured `vision_model`.
- **High availability**: per-app failover queues, circuit breakers, retries, health state, stream and non-stream timeout controls.
- **Proxy request logs**: View the last 7 days of proxy request logs in your preferred terminal with real-time scrolling.
- **Usage analytics**: request logs, token usage, cost estimation, trends, provider/model breakdowns and custom pricing.
- **Session Traces**: opt-in local context analysis for new requests, with Summary capture for extracted fields and Full capture for redacted request/response JSON.
- **MCP, prompts and skills**: unified management, sync across supported apps, deep links, repository/ZIP skill installation.
- **Data safety**: SQLite storage, atomic config writes, automatic backups, configurable storage directory and WebDAV sync.

## Architecture Diagrams

### Local AI Provider Gateway

![Local AI provider gateway architecture](docs/assets/diagrams/cc-gateway-pro-architecture-zh.svg)

### Vision Model Proxy Workflow

![Vision Model proxy workflow](docs/assets/diagrams/vision-model-proxy-flow-zh.svg)

### Project Provider Proxy Workflow

![Project Provider proxy workflow](docs/assets/diagrams/project-provider-proxy-flow-zh.svg)

More diagrams for failover, usage logging, MCP/Prompts/Skills synchronization and the full request chain are available in [Architecture and Flows](docs/architecture-and-flows-zh.md).

## Interface Preview

|                  Main screen                   |                  Add provider                  |
| :--------------------------------------------: | :--------------------------------------------: |
| ![Main screen](assets/screenshots/main-en.png) | ![Add provider](assets/screenshots/add-en.png) |

|                                Project routing                                |                                  Vision Model configuration                                  |
| :---------------------------------------------------------------------------: | :------------------------------------------------------------------------------------------: |
| ![Project routing preview](assets/screenshots/project-routing-preview-zh.svg) | ![Vision Model configuration preview](assets/screenshots/vision-model-config-preview-zh.svg) |

|                                  Session Traces settings                                   |                                  Session Traces analysis                                   |
| :----------------------------------------------------------------------------------------: | :----------------------------------------------------------------------------------------: |
| ![Session Traces settings preview](assets/screenshots/session-traces-settings-preview.svg) | ![Session Traces analysis preview](assets/screenshots/session-traces-analysis-preview.svg) |

## Installation

Download the latest build from [GitHub Releases](https://github.com/KeaneFeng/cc-gateway-pro/releases).

System requirements:

- macOS 12 or later
- Windows 10 or later
- Linux: WebKitGTK 4.1 / libsoup 3; release builds use Ubuntu 22.04. Other distributions require local compatibility verification.

### macOS Homebrew

Many users need to refresh Homebrew metadata before the cask can be found or updated correctly:

```bash
brew update
brew tap KeaneFeng/cc-gateway-pro
brew install --cask cc-gateway-pro
```

Upgrade:

```bash
brew update
brew upgrade --cask cc-gateway-pro
```

### Manual Download

- Windows: `.msi` / `.exe` — [GitHub Releases](https://github.com/KeaneFeng/cc-gateway-pro/releases/latest).
- macOS: download the `.dmg` from [Releases](https://github.com/KeaneFeng/cc-gateway-pro/releases).
- Linux: download the `.deb`, `.rpm` or `.AppImage` from [Releases](https://github.com/KeaneFeng/cc-gateway-pro/releases).
- Additional package channels: verify availability with the distributor; GitHub Releases is the authoritative download source.

## Build From Source

Prerequisites: Node.js 22.12+, pnpm 10.12.3 and Rust 1.95 (rust-toolchain.toml).

```bash
pnpm install --frozen-lockfile
pnpm tauri dev
pnpm tauri build
```

The repository also includes a helper:

```bash
./build.sh --dev
./build.sh --release
./build.sh --dmg
./build.sh --sha
```

## Documentation

- [Chinese README](README_ZH.md)
- [Japanese README](README_JA.md)
- [Swedish README](README_SV.md)
- [Spanish README](README_ES.md)
- [User manual](docs/user-manual/zh/README.md)
- [Architecture and flows](docs/architecture-and-flows-zh.md)
- [Proxy guide](docs/proxy-guide-zh.md)
- [Release notes](https://github.com/KeaneFeng/cc-gateway-pro/releases/latest)

## Data Locations

- Main database: `~/.cc-gateway-pro/cc-gateway-pro.db`
- Device settings: `~/.cc-gateway-pro/settings.json`
- Backups: `~/.cc-gateway-pro/backups/`
- Skills: `~/.cc-gateway-pro/skills/`

The storage directory can be moved to Dropbox, OneDrive, iCloud, NAS or another synchronized folder from the settings page.

## Credits

- Reference: [farion1231/cc-switch](https://github.com/farion1231/cc-switch)
- Built with Tauri, Rust, React, TypeScript and Tailwind CSS

## License

[MIT License](LICENSE) — Copyright (c) 2025 Jason Young.

CC-Gateway-Pro is maintained by [KeaneFeng](https://github.com/KeaneFeng). Upstream copyright and permission notices are retained; see [project history](docs/project-history.md).

## Development and project evidence

- [Development, build and test commands](CONTRIBUTING.md)
- [Security reports](SECURITY.md) · [Issue templates](https://github.com/KeaneFeng/cc-gateway-pro/issues/new/choose)
- [Project history and attribution](docs/project-history.md)
- [Verified contributions and release status](docs/original-contributions.md)
- [Maintenance priorities](docs/roadmap.md)
- [Dated repository audit](docs/jetbrains-application-audit.md)

The SVG interface previews above use demonstration data; they are illustrations, not evidence of production usage.
