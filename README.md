<div align="center">

# CC-Gateway-Pro

### AI Provider Gateway with Vision Routing & Project-Level Provider Binding

[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey.svg)](https://github.com/KeaneFeng/cc-gateway-pro)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-orange.svg)](https://tauri.app/)
[![Fork of cc-switch](https://img.shields.io/badge/fork%20of-cc--switch-blue.svg)](https://github.com/KeaneFeng/cc-switch)

English | [中文](#中文说明)

</div>

## What is CC-Gateway-Pro?

CC-Gateway-Pro is a fork of [cc-switch](https://github.com/KeaneFeng/cc-switch) that adds two powerful features for AI provider management:

### 🎯 Vision Model Auto-Routing
Automatically detects image content in requests and routes to a vision-capable model. Configure a `vision_model` per provider, and requests with images will automatically use it — no manual switching needed.

### 📁 Project-Level Provider Binding
Bind different AI providers to different project directories. When Claude Code sends a request from a specific project, it automatically uses the configured provider for that project.

## Features (inherited from cc-switch)

- **Multi-App Support**: Claude Code, Codex, Gemini CLI, OpenCode, Hermes Agent
- **50+ Provider Presets**: One-click import for popular providers
- **Visual Provider Management**: Card-based UI with drag-and-drop sorting
- **Proxy Server**: Built-in axum proxy with format conversion (Anthropic ↔ OpenAI ↔ Gemini)
- **Auto Failover**: Circuit breaker + failover queue for reliability
- **Usage Statistics**: Token tracking, cost estimation, balance queries
- **MCP & Skills Management**: Unified management across tools
- **i18n**: English and Chinese
- **System Tray**: Quick provider switching from menu bar

## Download & Installation

### System Requirements

- **macOS**: macOS 12 (Monterey) and above
- **Windows**: Windows 10 and above
- **Linux**: Ubuntu 22.04+ / Debian 11+ / Fedora 34+ and other mainstream distributions

### Windows Users

Download the latest `CC-Gateway-Pro-v{version}-Windows-x64-setup.exe` from the [Releases](https://github.com/KeaneFeng/cc-gateway-pro/releases) page.

### macOS Users

**Method 1: Install via Homebrew (Recommended)**

```bash
brew tap KeaneFeng/cc-gateway-pro
brew install --cask cc-gateway-pro
```

Update:

```bash
brew upgrade --cask cc-gateway-pro
```

**Method 2: Manual Download**

Download `CC-Gateway-Pro-v{version}-macOS.dmg` from the [Releases](https://github.com/KeaneFeng/cc-gateway-pro/releases) page.

### Linux Users

Download the latest Linux build from the [Releases](https://github.com/KeaneFeng/cc-gateway-pro/releases) page:

- `CC-Gateway-Pro-v{version}-Linux.deb` (Debian/Ubuntu)
- `CC-Gateway-Pro-v{version}-Linux.AppImage` (Universal)

## Build from Source (Local Testing)

```bash
# Prerequisites: Node.js 22+, Rust stable
./build.sh --dev          # Development mode (hot reload)
./build.sh --release      # Release build
./build.sh --dmg          # Package DMG installer
./build.sh --sha          # Calculate DMG SHA256 (for Homebrew update)
```

**Build outputs:**
- `src-tauri/target/release/bundle/dmg/` — macOS DMG
- `src-tauri/target/release/bundle/macos/` — macOS .app
- `src-tauri/target/release/bundle/nsis/` — Windows installer
- `src-tauri/target/release/bundle/deb/` — Linux .deb

**Test DMG locally and update Homebrew:**
```bash
# 1. Build DMG
./build.sh --dmg

# 2. Get SHA256
./build.sh --sha
# Output: SHA256: xxxxxxx...

# 3. Update homebrew tap
cd ~/www/homebrew-cc-gateway-pro
# Edit Casks/cc-gateway-pro.rb: replace PLACEHOLDER_SHA256 with actual hash
# Update version if needed

# 4. Commit and push tap
git add -A && git commit -m "update to v0.1.0" && git push

# 5. Test local install
brew tap KeaneFeng/cc-gateway-pro
brew install --cask cc-gateway-pro
```

## Architecture

```
Claude Code → CC-Gateway-Pro Proxy (port 16789) → AI Provider
                    ↓
            Vision Detection (image block?)
                    ↓ Yes              ↓ No
            vision_model          default model
                    ↓                  ↓
            Provider with         Provider with
            vision support        text-only support
```

## Project-Level Routing

```
~/projects/frontend → Provider A (Claude Sonnet)
~/projects/backend  → Provider B (DeepSeek)
~/projects/data     → Provider C (Gemini)
```

## Credits

- Based on [cc-switch](https://github.com/KeaneFeng/cc-switch) by Jason Young
- Original cc-gateway Rust CLI by [Keane Feng](https://github.com/KeaneFeng)

## License

MIT

---

<a id="中文说明"></a>

## 中文说明

CC-Gateway-Pro 是基于 [cc-switch](https://github.com/KeaneFeng/cc-switch) 的增强版本，新增两大核心功能：

**🎯 Vision Model 自动路由** — 检测请求中的图片内容，自动切换到支持视觉的模型，无需手动操作。

**📁 项目级 Provider 绑定** — 不同项目目录绑定不同的 AI 供应商，Claude Code 发送请求时自动使用对应项目的 Provider。

继承了 cc-switch 全部功能：多 App 支持（Claude Code/Codex/Gemini CLI/OpenCode/Hermes）、50+ 预设、可视化 Provider 管理、代理服务器、自动故障转移、用量统计、MCP/Skills 管理、中英双语。

### 安装方式

**macOS (Homebrew)**

```bash
brew tap KeaneFeng/cc-gateway-pro
brew install --cask cc-gateway-pro
```

**手动下载**

从 [GitHub Releases](https://github.com/KeaneFeng/cc-gateway-pro/releases) 下载对应平台的安装包。
