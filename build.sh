#!/bin/bash
# cc-gateway-pro 构建脚本
# 用法: ./build.sh [--dev|--release|--dmg|--brew|--bump|--version]

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

# Node.js 环境
export PATH="$HOME/.local/node-v22/bin:$PATH"
PNPM="$HOME/.local/node-v22-global/lib/node_modules/pnpm/bin/pnpm.cjs"

# 配置文件路径
TAURI_CONF="src-tauri/tauri.conf.json"
CARGO_TOML="src-tauri/Cargo.toml"
PKG_JSON="package.json"
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

# ========== 基础函数 ==========

check_deps() {
    command -v node >/dev/null 2>&1 || err "Node.js not found"
    command -v cargo >/dev/null 2>&1 || err "Rust/Cargo not found"
    command -v rustc >/dev/null 2>&1 || err "Rustc not found"
    log "Node $(node --version), Rust $(rustc --version | cut -d' ' -f2)"
}

install_deps() {
    log "Installing frontend dependencies..."
    node $PNPM install
}

build_dev() {
    log "Building in debug mode..."
    node $PNPM tauri dev
}

build_release() {
    log "Building in release mode (this may take 5-10 minutes)..."
    node $PNPM tauri build
    log "Build complete! Check src-tauri/target/release/bundle/"
}

build_dmg() {
    log "Building DMG..."
    node $PNPM tauri build
    DMG_PATH=$(find src-tauri/target/release/bundle/dmg -name '*.dmg' 2>/dev/null | head -1)
    if [ -n "$DMG_PATH" ]; then
        log "DMG created: $DMG_PATH"
        log "Size: $(du -h "$DMG_PATH" | cut -f1)"
    else
        warn "DMG not found"
    fi
}

build_frontend() {
    log "Building frontend only..."
    node $PNPM run build:renderer
    log "Frontend built to dist/"
}

calc_sha() {
    DMG_PATH=$(find src-tauri/target/release/bundle/dmg -name '*.dmg' 2>/dev/null | head -1)
    if [ -n "$DMG_PATH" ]; then
        SHA=$(shasum -a 256 "$DMG_PATH" | cut -d' ' -f1)
        log "SHA256: $SHA"
    else
        err "No DMG found. Run './build.sh --dmg' first."
    fi
}

# ========== 版本管理 ==========

get_version() {
    python3 -c "import json; print(json.load(open('$TAURI_CONF'))['version'])"
}

show_version() {
    CURRENT_VERSION=$(get_version)
    echo -e "${BLUE}[版本]${NC} 当前版本: ${GREEN}${CURRENT_VERSION}${NC}"
}

# 同步更新三个版本文件
set_version() {
    local ver="$1"
    python3 -c "
import json, re

# tauri.conf.json
with open('$TAURI_CONF', 'r') as f:
    conf = json.load(f)
conf['version'] = '$ver'
with open('$TAURI_CONF', 'w') as f:
    json.dump(conf, f, indent=2)
    f.write('\n')

# Cargo.toml
with open('$CARGO_TOML', 'r') as f:
    content = f.read()
content = re.sub(r'^version = \".*\"', 'version = \"$ver\"', content, 1, re.MULTILINE)
with open('$CARGO_TOML', 'w') as f:
    f.write(content)

# package.json
with open('$PKG_JSON', 'r') as f:
    pkg = json.load(f)
pkg['version'] = '$ver'
with open('$PKG_JSON', 'w') as f:
    json.dump(pkg, f, indent=2)
    f.write('\n')
"
}

# 仅递增版本号（不构建不发布）
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
            *)      err "无效的版本模式: $mode (可选: major/minor/patch 或版本号如 3.16.0)" ;;
        esac
        info "新版本: ${GREEN}${NEW_VERSION}${NC}"
    fi

    set_version "$NEW_VERSION"
    log "已同步更新 tauri.conf.json + Cargo.toml + package.json -> $NEW_VERSION"
}

# ========== 发布流程（推送 tag → GitHub Actions 构建） ==========

brew_release() {
    local skip_confirm=false
    local version_mode=""

    # 解析参数
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --yes|-y)   skip_confirm=true; shift ;;
            --bump|-b)  shift ;;
            major|minor|patch) version_mode="$1"; shift ;;
            *)  if [[ "$1" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then version_mode="$1"; fi; shift ;;
        esac
    done
    [ -z "$version_mode" ] && version_mode="patch"

    CURRENT_VERSION=$(get_version)

    log "=== 发布流程 ==="
    log "当前版本: $CURRENT_VERSION"

    # 计算新版本号
    if [[ "$version_mode" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        NEW_VERSION="$version_mode"
        info "设置版本为: $NEW_VERSION"
    else
        IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"
        case "$version_mode" in
            major)  NEW_VERSION="$((MAJOR + 1)).0.0" ;;
            minor)  NEW_VERSION="${MAJOR}.$((MINOR + 1)).0" ;;
            patch)  NEW_VERSION="${MAJOR}.${MINOR}.$((PATCH + 1))" ;;
        esac
        info "新版本: ${GREEN}${NEW_VERSION}${NC}"
    fi

    # 更新版本文件
    set_version "$NEW_VERSION"
    log "已同步更新版本文件 -> $NEW_VERSION"

    # 确认
    if [ "$skip_confirm" = false ]; then
        read -p "$(echo -e ${YELLOW}确认发布 $NEW_VERSION? (推送 tag 触发 GitHub Actions) [y/N] ${NC})" -r
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            warn "已取消"
            exit 0
        fi
    fi

    # 提交版本变更
    git add -A
    git commit -m "Bump version to ${NEW_VERSION}" || warn "无变更需要提交"
    git push 2>&1
    log "版本变更已推送"

    # 创建并推送 tag → 触发 GitHub Actions
    TAG="v${NEW_VERSION}"
    log "推送 tag: $TAG → 触发 GitHub Actions"
    git tag -a "$TAG" -m "Release $TAG" 2>&1 || warn "Tag 已存在"
    git push origin "$TAG" 2>&1
    log "Tag 已推送"

    # 提示
    echo ""
    echo -e "${GREEN}=== 发布已触发 ===${NC}"
    echo -e "  版本: ${NEW_VERSION}"
    echo -e "  CI 进度: https://github.com/KeaneFeng/cc-gateway-pro/actions"
    echo ""
    echo -e "${BLUE}GitHub Actions 将自动:${NC}"
    echo "  1. 跨平台构建 (macOS/Windows/Linux)"
    echo "  2. Apple 签名 + 公证"
    echo "  3. 生成 latest.json (应用内更新检测)"
    echo "  4. 上传所有安装包"
    echo ""
    echo -e "${YELLOW}CI 完成后更新 Homebrew:${NC}"
    echo "  ./build.sh --update-sha $NEW_VERSION"
}

# ========== CI 完成后更新 Homebrew SHA ==========

update_sha() {
    local version="${1:-}"
    [ -z "$version" ] && err "请指定版本号: ./build.sh --update-sha 3.15.1"

    TAG="v${version}"
    log "从 GitHub Release $TAG 下载 DMG 计算 SHA256..."

    DMG_NAME="CC-Gateway-Pro_${version}_aarch64.dmg"
    DMG_URL="https://github.com/KeaneFeng/cc-gateway-pro/releases/download/${TAG}/${DMG_NAME}"
    TMP_DMG="/tmp/${DMG_NAME}"

    curl -L -o "$TMP_DMG" "$DMG_URL" 2>&1 || err "下载失败"
    SHA=$(shasum -a 256 "$TMP_DMG" | cut -d' ' -f1)
    rm -f "$TMP_DMG"
    log "SHA256: $SHA"

    if [ -f "$HOMEBREW_TAP" ]; then
        python3 -c "
import re
content = open('$HOMEBREW_TAP').read()
content = re.sub(r'version \"[^\"]+\"', 'version \"${version}\"', content)
content = re.sub(r'sha256 \"[^\"]+\"', 'sha256 \"${SHA}\"', content)
content = re.sub(r'CC-Gateway-Pro_[^\"]+\.dmg', '${DMG_NAME}', content)
open('$HOMEBREW_TAP', 'w').write(content)
"
        cd "$(dirname "$HOMEBREW_TAP")"
        git add -A
        git commit -m "Update cc-gateway-pro to v${version}" || warn "无变更"
        git push 2>&1
        log "Homebrew Cask 已更新并推送"
    else
        err "Homebrew Cask 不存在: $HOMEBREW_TAP"
    fi
}

# ========== 帮助 ==========

show_help() {
    cat << EOF
cc-gateway-pro 构建脚本

用法: ./build.sh [选项]

选项:
  --dev              开发模式（热重载）
  --release          Release 构建
  --dmg              打包 DMG
  --frontend         仅构建前端
  --sha              计算 DMG SHA256
  --deps             仅安装前端依赖
  --version          显示当前版本号
  --bump [MODE]      递增版本号 (major/minor/patch 或版本号)
  --brew [MODE]      发布: 推送 tag → GitHub Actions 自动构建+签名+生成latest.json
                     --yes/-y 跳过确认
  --update-sha VER   CI 完成后更新 Homebrew Cask SHA256
  --help             显示帮助

示例:
  ./build.sh --version              # 查看当前版本
  ./build.sh --bump                 # 递增 patch (3.15.1 → 3.15.2)
  ./build.sh --bump minor           # 递增 minor (3.15.1 → 3.16.0)
  ./build.sh --brew                 # 发布: bump + 推送 tag → CI 自动构建
  ./build.sh --brew --yes           # 同上，跳过确认
  ./build.sh --brew minor           # 递增 minor 并发布
  ./build.sh --update-sha 3.15.2    # CI 完成后更新 Homebrew SHA
EOF
}

# ========== 主逻辑 ==========

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
    --update-sha)   update_sha "${2:-}" ;;
    --help|-h)      show_help ;;
    *)              show_help ;;
esac
