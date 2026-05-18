#!/bin/bash
# cc-gateway-pro 本地构建脚本
# 用法: ./build.sh [--dev|--release|--dmg]

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

# Node.js 环境
export PATH="$HOME/.local/node-v22/bin:$PATH"
PNPM="$HOME/.local/node-v22-global/lib/node_modules/pnpm/bin/pnpm.cjs"

# 颜色
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log() { echo -e "${GREEN}[cc-gateway-pro]${NC} $1"; }
warn() { echo -e "${YELLOW}[warn]${NC} $1"; }
err() { echo -e "${RED}[error]${NC} $1"; exit 1; }

# 检查依赖
check_deps() {
    command -v node >/dev/null 2>&1 || err "Node.js not found"
    command -v cargo >/dev/null 2>&1 || err "Rust/Cargo not found"
    command -v rustc >/dev/null 2>&1 || err "Rustc not found"
    log "Node $(node --version), Rust $(rustc --version | cut -d' ' -f2)"
}

# 安装前端依赖
install_deps() {
    log "Installing frontend dependencies..."
    node $PNPM install
}

# 开发模式构建
build_dev() {
    log "Building in debug mode..."
    node $PNPM tauri dev
}

# Release 构建
build_release() {
    log "Building in release mode (this may take 5-10 minutes)..."
    node $PNPM tauri build
    log "Build complete! Check src-tauri/target/release/bundle/"
}

# 仅打包 DMG（需要先 build）
build_dmg() {
    log "Building DMG..."
    node $PNPM tauri build --target dmg 2>/dev/null || node $PNPM tauri build
    DMG_PATH=$(find src-tauri/target/release/bundle/dmg -name '*.dmg' 2>/dev/null | head -1)
    if [ -n "$DMG_PATH" ]; then
        log "DMG created: $DMG_PATH"
        log "Size: $(du -h "$DMG_PATH" | cut -f1)"
    else
        warn "DMG not found, check src-tauri/target/release/bundle/"
    fi
}

# 仅前端构建（测试 Vite）
build_frontend() {
    log "Building frontend only..."
    node $PNPM run build:renderer
    log "Frontend built to dist/"
}

# 计算 SHA256（用于 Homebrew）
calc_sha() {
    DMG_PATH=$(find src-tauri/target/release/bundle/dmg -name '*.dmg' 2>/dev/null | head -1)
    if [ -n "$DMG_PATH" ]; then
        SHA=$(shasum -a 256 "$DMG_PATH" | cut -d' ' -f1)
        log "SHA256: $SHA"
        log "Update homebrew/Casks/cc-gateway-pro.rb with this hash"
    else
        err "No DMG found. Run './build.sh --dmg' first."
    fi
}

# 显示帮助
show_help() {
    cat << EOF
cc-gateway-pro 构建脚本

用法: ./build.sh [选项]

选项:
  --dev         开发模式（热重载，调试用）
  --release     Release 构建（产出 .dmg + .app）
  --dmg         打包 DMG 安装包
  --frontend    仅构建前端（测试 Vite）
  --sha         计算 DMG 的 SHA256（更新 Homebrew 用）
  --deps        仅安装前端依赖
  --help        显示此帮助

示例:
  ./build.sh --dev           # 启动开发模式
  ./build.sh --release       # 完整 release 构建
  ./build.sh --dmg           # 打包 DMG
  ./build.sh --sha           # 获取 SHA256

产出目录:
  src-tauri/target/release/bundle/dmg/    # macOS DMG
  src-tauri/target/release/bundle/macos/  # macOS .app
  src-tauri/target/release/bundle/nsis/   # Windows installer
EOF
}

# 主逻辑
check_deps

case "${1:-}" in
    --dev)      install_deps; build_dev ;;
    --release)  install_deps; build_release ;;
    --dmg)      install_deps; build_dmg ;;
    --frontend) install_deps; build_frontend ;;
    --sha)      calc_sha ;;
    --deps)     install_deps ;;
    --help|-h)  show_help ;;
    *)          show_help ;;
esac
