# CC-Gateway-Pro 开发计划

> 基于 cc-gateway-pro v3.15.0 fork，实现 cc-gateway 的差异化功能

## 背景

cc-gateway（Rust CLI proxy）的核心价值：Vision Model 自动路由 + 项目级 Provider 绑定。
但 TUI (crossterm raw mode) 体验极差，表单编辑、复制粘贴都无法正常使用。

cc-gateway-pro 已经是成熟的 Tauri 2 + React 桌面应用，覆盖了：
- Provider CRUD + 卡片式 UI + 拖拽排序
- 多 App 支持（Claude Code / Codex / Gemini CLI / OpenCode / Hermes）
- 代理服务器（axum + hyper）+ 格式转换（Anthropic ↔ OpenAI ↔ Gemini）
- 故障转移 + 熔断器
- 用量统计 + 余额查询
- 预设导入 + 会话管理
- i18n 中英文

**结论：直接 fork cc-gateway-pro，加 2 个差异化功能即可。**

---

## 现有能力对比

| 功能 | cc-gateway | cc-gateway-pro | 迁移策略 |
|------|-----------|-----------|---------|
| Provider CRUD | ✅ TUI 表单 | ✅ React UI | ✅ 直接用 cc-gateway-pro |
| Provider 切换 | ✅ CLI/API | ✅ UI + 热切换 | ✅ 直接用 |
| 代理服务器 | ✅ axum | ✅ axum + hyper | ✅ 直接用 |
| 格式转换 | ✅ Anthropic↔OpenAI | ✅ 全格式 | ✅ 直接用 |
| SSE 流式转发 | ✅ | ✅ | ✅ 直接用 |
| 健康检查 | ✅ pbcopy CLI | ✅ UI 实时状态 | ✅ 直接用 |
| 余额查询 | ✅ CLI | ✅ UI 卡片内嵌 | ✅ 直接用 |
| 用量统计 | ✅ SQLite | ✅ SQLite + UI 图表 | ✅ 直接用 |
| 预设导入 | ✅ CLI presets | ✅ UI 对话框 | ✅ 直接用 |
| effort_level | ✅ provider 级 | ✅ thinking_optimizer + resolve_reasoning_effort | ✅ 已有 |
| 模型映射 | ✅ display_name | ✅ model_mapper (haiku/sonnet/opus) | ✅ 已有 |
| 熔断/故障转移 | ❌ | ✅ circuit_breaker + failover | ✅ 新增能力 |
| 拖拽排序 | ❌ | ✅ dnd-kit | ✅ 新增能力 |
| 多 App 支持 | ❌ | ✅ 6 种 App | ✅ 新增能力 |
| **Vision Model 路由** | **✅ 核心差异** | ❌ | **需迁移** |
| **项目级 Provider 绑定** | **✅ 核心差异** | ❌ (有 session.project_dir 但无路由) | **需迁移** |

---

## 需迁移的 2 个核心功能

### Feature 1: Vision Model 自动路由

**cc-gateway 现有逻辑** (src/proxy/transform.rs):
```
请求 → 解析 body → 检测 messages[].content[].type == "image"
  → 如果有图片 && provider 有 vision_model → 用 vision_model 替代 model
```

**cc-gateway-pro 需要的改动**:

#### 1.1 数据模型扩展
- 文件: `src-tauri/src/provider.rs` → ProviderMeta
- 新增字段: `vision_model: Option<String>`
- 前端: `src/types.ts` → Provider type 同步加 vision_model

#### 1.2 Provider 编辑 UI
- 文件: `src/components/providers/forms/` 编辑表单
- 新增 "Vision Model" 输入框（可选，在 Model 字段下方）
- 提示："留空则所有请求使用同一模型"

#### 1.3 代理路由层 Vision 检测
- 文件: `src-tauri/src/proxy/handler_context.rs` → build_context_for_claude()
- 在 `request_model` 提取后，解析 body 检测 image content block
- 如果检测到图片 && 当前 provider 配置了 vision_model → 替换 request_model
- 新增函数: `fn has_image_content(body: &Value) -> bool`
  - 遍历 body["messages"] → content 数组 → 检查 type == "image"
  - 同时检查 tool_result 中的 image block

#### 1.4 模型映射联动
- 文件: `src-tauri/src/proxy/model_mapper.rs`
- ModelMapping 新增 vision_model 字段
- map_model() 逻辑: 如果 has_image 且有 vision_model → 返回 vision_model

---

### Feature 2: 项目级 Provider 绑定

**cc-gateway 现有逻辑**:
```
路由优先级: session > project > model > default
1. x-claude-code-session-id → SessionRouter → 匹配 project_providers[path]
2. project_dirs 扫描 JSONL 文件 → session → cwd → provider
3. config.toml 中 project_providers: HashMap<path, provider_id>
```

**cc-gateway-pro 需要的改动**:

#### 2.1 数据模型
- 文件: `src-tauri/src/proxy/types.rs` → ProxyConfig
- 新增字段:
  ```rust
  project_providers: HashMap<String, String>,  // project_path -> provider_id
  project_dirs: Vec<String>,                    // 要扫描的项目目录
  ```

#### 2.2 配置 UI
- 新增 Tab: "项目路由" (Project Routing)
  - 项目目录列表（可添加/删除）
  - 项目 → Provider 映射表（路径 + Provider 下拉选择）
  - 自动发现: 扫描 project_dirs 下的 .claude/ 目录

#### 2.3 路由层集成
- 文件: `src-tauri/src/proxy/handler_context.rs`
- 在 select_providers() 后，检查 session 的 project_dir
- 如果 project_dir 在 project_providers 中有映射 → 切换到指定 provider
- 路由优先级: project_mapping > failover_queue > current_provider

#### 2.4 Session Manager 扩展
- 文件: `src-tauri/src/session_manager/mod.rs`
- SessionMeta 已有 project_dir 字段
- 新增: 从 session 的 project_dir 查找 project_providers 映射

---

## 品牌改造

| 改动 | 文件 |
|------|------|
| 项目名 | package.json → name: "cc-gateway-pro" |
| 窗口标题 | src-tauri/tauri.conf.json → title |
| App 名称 | src-tauri/tauri.conf.json → identifier |
| 图标 | src-tauri/icons/ → 替换 |
| README | 新写 README.md |
| 数据目录 | ~/.cc-gateway-pro/ → ~/.cc-gateway-pro/ (可选，保持兼容也可) |

---

## 实施阶段

### Phase 0: 品牌改造 + 编译验证 (0.5天)
- [ ] 修改 package.json、tauri.conf.json
- [ ] 确认 `pnpm install && pnpm tauri dev` 能跑
- [ ] 确认基本功能正常（Provider 列表、切换、代理）

### Phase 1: Vision Model 路由 (1-1.5天)
- [ ] 1.1 ProviderMeta 加 vision_model 字段
- [ ] 1.2 前端编辑表单加 Vision Model 输入框
- [ ] 1.3 handler_context 加 image 检测 + 模型替换
- [ ] 1.4 model_mapper 联动
- [ ] 测试: 带 image 的请求自动路由到 vision_model

### Phase 2: 项目级 Provider 绑定 (1-1.5天)
- [ ] 2.1 ProxyConfig 加 project_providers / project_dirs
- [ ] 2.2 前端 "项目路由" Tab
- [ ] 2.3 handler_context 加 project 路由优先级
- [ ] 2.4 session_manager 扩展
- [ ] 测试: 不同项目目录使用不同 provider

### Phase 3: 集成测试 + 打包 (1天)
- [ ] Claude Code 连接 cc-gateway-pro 代理
- [ ] Vision 请求自动路由验证
- [ ] 项目路由验证
- [ ] 故障转移 + 熔断验证
- [ ] macOS build + dmg 打包

---

## 技术要点

### Vision 检测实现
```rust
fn has_image_content(body: &serde_json::Value) -> bool {
    let messages = match body.get("messages").and_then(|m| m.as_array()) {
        Some(msgs) => msgs,
        None => return false,
    };
    for msg in messages {
        if let Some(content) = msg.get("content").and_then(|c| c.as_array()) {
            for block in content {
                if block.get("type").and_then(|t| t.as_str()) == Some("image") {
                    return true;
                }
            }
        }
    }
    false
}
```

### 路由优先级
```
cc-gateway 原始: session > project > model > default
cc-gateway-pro:  session_project > failover > current
```
- session 通过 JSONL 扫描关联 project_dir
- project_dir 查 project_providers 映射
- 无映射则走原有 failover/current 逻辑

### 数据兼容性
- cc-gateway 用 TOML 配置，cc-gateway-pro 用 JSON
- 两者都用 SQLite（WAL mode），schema 兼容
- 迁移路径: cc-gateway TOML → cc-gateway-pro JSON（一次性导入，UI 操作）

---

## 风险与注意事项

1. **不要破坏 cc-gateway-pro 已有功能** — 所有改动都是新增，不修改核心路由/转换逻辑
2. **Vision 检测性能** — body 解析在转发前，确保不增加显著延迟
3. **项目路径标准化** — macOS 路径有 symlink，需要 canonicalize
4. **pnpm 依赖** — cc-gateway-pro 用 pnpm workspace，需要 `corepack enable`
5. **Tauri 2 环境** — 需要 Rust nightly 或 stable + Tauri CLI

---

## 最终产物

一个 Tauri 2 桌面应用（cc-gateway-pro），具备：
- cc-gateway-pro 全部功能（Provider 管理、多 App、故障转移、用量统计）
- ✨ Vision Model 自动路由（检测图片 → 切换模型）
- ✨ 项目级 Provider 绑定（不同项目用不同 AI）
- 完全替代 cc-gateway CLI + TUI
