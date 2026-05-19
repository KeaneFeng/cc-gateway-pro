#!/bin/bash
# cc-gateway-pro 本地构建脚本
# 用法: ./build.sh [--dev|--release|--dmg|--brew|--bump|--version VERSION]

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

# Node.js 环境
export PATH="$HOME/.local/node-v22/bin:$PATH"
PNPM="$HOME/.local/node-v22-global/lib/node_modules/pnpm/bin/pnpm.cjs"

# 配置文件路径
TAURI_CONF="src-tauri/tauri.conf.json"
HOMEBREW_TAP="/opt/homebrew/Library/Taps/keanefeng/homebrew-cc-gateway-pro/Casks/cc-gateway-pro.rb"

# 颜色
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

log() { echo -e "${GREEN}[cc-gateway-pro]${NC} $1"; }
warn() { echo -e "${YELLOW}[warn]${NC} $1"; }
err() { echo -e "${RED}[error]${NC} $1"; exit 1; }
info() { echo -e "${BLUE}[info]${NC} $1"; }

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

# 获取当前版本号
get_version() {
    python3 -c "import json; print(json.load(open('$TAURI_CONF'))['version'])"
}

# 显示当前版本
show_version() {
    CURRENT_VERSION=$(get_version)
    echo -e "${BLUE}[版本]${NC} 当前版本: ${GREEN}${CURRENT_VERSION}${NC}"
}

# 递增版本号
# 用法: bump_version [major|minor|patch|VERSION]
# - patch: 自动递增第三位（默认）
# - minor: 自动递增第二位，第三位归零
# - major: 自动递增第一位，第二、三位归零
# - VERSION: 直接设置为指定版本
bump_version() {
    local mode="${1:-patch}"
    CURRENT_VERSION=$(get_version)
    
    echo -e "${BLUE}[版本]${NC} 当前版本: ${GREEN}${CURRENT_VERSION}${NC}"
    
    if [[ "$mode" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        NEW_VERSION="$mode"
        info "设置版本为: $NEW_VERSION"
    else
        IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"
        case "$mode" in
            major)  NEW_VERSION="$((MAJOR + 1)).0.0" ;;
            minor)  NEW_VERSION="${MAJOR}.$((MINOR + 1)).0" ;;
            patch)  NEW_VERSION="${MAJOR}.${MINOR}.$((PATCH + 1))" ;;
            *)      err "无效的版本模式: $mode (可选: major/minor/patch 或完整版本号如 3.16.0)" ;;
        esac
        info "新版本: ${GREEN}${NEW_VERSION}${NC}"
    fi
    
    # 更新 tauri.conf.json
    python3 -c "
import json
with open('$TAURI_CONF', 'r') as f:
    conf = json.load(f)
conf['version'] = '$NEW_VERSION'
with open('$TAURI_CONF', 'w') as f:
    json.dump(conf, f, indent=2)
    f.write('\n')
"
    
    log "已更新 $TAURI_CONF -> $NEW_VERSION"
    echo "$NEW_VERSION"
}

# 完整流程：递增版本 + 构建 DMG + 更新 Homebrew + 发布 Release
brew_release() {
    local skip_confirm=false
    local version_mode=""
    
    # 解析参数
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --yes|-y) skip_confirm=true; shift ;;
            --bump|-b) shift ;;
            major|minor|patch) version_mode="$1"; shift ;;
            *) if [[ "$1" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then version_mode="$1"; fi; shift ;;
        esac
    done
    
    if [ -z "$version_mode" ]; then
        version_mode="patch"
    fi
    
    CURRENT_VERSION=$(get_version)
    
    log "=== 开始 Homebrew 发布流程 ==="
    log "当前版本: $CURRENT_VERSION"
    
    if [[ "$version_mode" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        NEW_VERSION="$version_mode"
        info "设置版本为: $NEW_VERSION"
    else
        IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"
        case "$version_mode" in
            major)  NEW_VERSION="$((MAJOR + 1)).0.0" ;;
            minor)  NEW_VERSION="${MAJOR}.$((MINOR + 1)).0" ;;
            patch)  NEW_VERSION="${MAJOR}.${MINOR}.$((PATCH + 1))" ;;
            *)      err "无效的版本模式: $version_mode (可选: major/minor/patch 或直接指定版本号)" ;;
        esac
        info "新版本: ${GREEN}${NEW_VERSION}${NC}"
    fi
    
    # 更新 tauri.conf.json
    python3 -c "
import json
with open('$TAURI_CONF', 'r') as f:
    conf = json.load(f)
conf['version'] = '$NEW_VERSION'
with open('$TAURI_CONF', 'w') as f:
    json.dump(conf, f, indent=2)
    f.write('\\n')
"
    log "已更新 $TAURI_CONF -> $NEW_VERSION"
    
    # 确认
    if [ "$skip_confirm" = false ]; then
        read -p "$(echo -e ${YELLOW}是否继续构建并发布 $NEW_VERSION? [y/N] ${NC})" -r
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            warn "已取消"
            exit 0
        fi
    fi
    
    # 3. 构建 DMG
    install_deps
    build_dmg
    
    # 4. 计算 SHA256
    DMG_PATH=$(find src-tauri/target/release/bundle/dmg -name '*.dmg' 2>/dev/null | head -1)
    if [ -z "$DMG_PATH" ]; then
        err "DMG 构建失败"
    fi
    SHA=$(shasum -a 256 "$DMG_PATH" | cut -d' ' -f1)
    log "SHA256: $SHA"
    
    DMG_FILENAME=$(basename "$DMG_PATH")
    
    # 5. 创建 GitHub Release
    log "创建 GitHub Release v$NEW_VERSION..."
    if gh release view "v${NEW_VERSION}" &>/dev/null; then
        warn "Release v${NEW_VERSION} 已存在，尝试上传资产..."
        gh release upload "v${NEW_VERSION}" "$DMG_PATH" --clobber 2>&1 || warn "上传失败"
    else
        gh release create "v${NEW_VERSION}" \
            "$DMG_PATH" \
            --title "v${NEW_VERSION}" \
            --notes "CC-Gateway-Pro v${NEW_VERSION}
        
- Multi-provider aggregation gateway for Claude Code
- Tauri 2 desktop application
- macOS ARM64 (Apple Silicon)" 2>&1 || warn "Release 创建失败"
    fi
    log "Release: https://github.com/KeaneFeng/cc-gateway-pro/releases/tag/v${NEW_VERSION}"
    
    # 6. 更新 Homebrew Cask
    if [ -f "$HOMEBREW_TAP" ]; then
        log "更新 Homebrew Cask..."
        python3 -c "
content = open('$HOMEBREW_TAP').read()
import re
content = re.sub(r'version \"[^\"]+\"', 'version \"${NEW_VERSION}\"', content)
content = re.sub(r'sha256 \"[^\"]+\"', 'sha256 \"${SHA}\"', content)
content = re.sub(r'CC-Gateway-Pro_[^\"]+\.dmg', '${DMG_FILENAME}', content)
open('$HOMEBREW_TAP', 'w').write(content)
"
        log "Homebrew Cask 已更新"
        
        # 7. 提交并推送 tap 仓库
        cd "$(dirname "$HOMEBREW_TAP")"
        git add -A
        git commit -m "Update cc-gateway-pro to v${NEW_VERSION}" || warn "无变更需要提交"
        git push 2>&1
        log "Tap 仓库已推送"
    else
        warn "Homebrew Cask 文件不存在: $HOMEBREW_TAP"
    fi
    
    # 8. 提交 tauri.conf.json 变更
    cd "$SCRIPT_DIR"
    git add "$TAURI_CONF"
    git commit -m "Bump version to ${NEW_VERSION}" || warn "无变更需要提交"
    
    log "=== 发布完成 ==="
    echo ""
    echo -e "${GREEN}版本:${NC} $NEW_VERSION"
    echo -e "${GREEN}Release:${NC} https://github.com/KeaneFeng/cc-gateway-pro/releases/tag/v${NEW_VERSION}"
    echo -e "${GREEN}安装命令:${NC} brew tap KeaneFeng/cc-gateway-pro && brew install --cask cc-gateway-pro"
}

# 显示帮助
show_help() {
    cat << EOF
cc-gateway-pro 构建脚本

用法: ./build.sh [选项]

选项:
  --dev              开发模式（热重载，调试用）
  --release          Release 构建（产出 .dmg + .app）
  --dmg              打包 DMG 安装包
  --frontend         仅构建前端（测试 Vite）
  --sha              计算 DMG 的 SHA256（更新 Homebrew 用）
  --deps             仅安装前端依赖
  --version          显示当前版本号
  --bump [MODE]      递增版本号并更新配置
                     MODE: major/minor/patch(默认) 或直接指定版本号如 3.16.0
  --brew [MODE]      完整发布流程：递增版本 + 构建 DMG + 发布 Release + 更新 Homebrew
                     MODE: major/minor/patch(默认) 或直接指定版本号
                     --yes/-y 跳过确认，--bump 显示当前版本提示
  --help             显示此帮助

示例:
  ./build.sh --dev                  # 启动开发模式
  ./build.sh --release              # 完整 release 构建
  ./build.sh --dmg                  # 打包 DMG
  ./build.sh --sha                  # 获取 SHA256
  ./build.sh --version              # 查看当前版本
  ./build.sh --bump                 # 自动递增 patch 版本 (3.15.0 -> 3.15.1)
  ./build.sh --bump minor           # 递增 minor 版本 (3.15.0 -> 3.16.0)
  ./build.sh --bump 3.20.0          # 直接设置为指定版本
  ./build.sh --brew                 # 完整发布流程（自动 bump patch）
  ./build.sh --brew --yes           # 同上，跳过确认
  ./build.sh --brew --bump          # 显示当前版本并提示确认
  ./build.sh --brew minor           # 递增 minor 版本并发布
  ./build.sh --brew 4.0.0           # 直接设为指定版本并发布

产出目录:
  src-tauri/target/release/bundle/dmg/    # macOS DMG
  src-tauri/target/release/bundle/macos/  # macOS .app
  src-tauri/target/release/bundle/nsis/   # Windows installer
EOF
}

# 主逻辑
check_deps

case "${1:-}" in
    --dev)          install_deps; build_dev ;;
    --release)      install_deps; build_release ;;
    --dmg)          install_deps; build_dmg ;;
    --frontend)     install_deps; build_frontend ;;
    --sha)          calc_sha ;;
    --deps)         install_deps ;;
    --version|-v)   show_version ;;
    --bump)         bump_version "${2:-patch}" ;;
    --brew)         shift; brew_release "$@" ;;
    --help|-h)      show_help ;;
    *)              show_help ;;
esac
