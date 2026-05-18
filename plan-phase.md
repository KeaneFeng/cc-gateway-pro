# CC-Gateway-Pro Phase 追踪

## Phase 0: 品牌改造 + 编译验证 ✅
- **状态**: 完成
- **完成时间**: 2026-05-17
- **内容**: fork cc-switch v3.15.0 → cc-gateway-pro，品牌替换101个文件，新icon，GitHub推送
- **仓库**: https://github.com/KeaneFeng/cc-gateway-pro

## Phase 1: Vision Model 自动路由 ✅
- **状态**: 完成 + 实际测试验证通过
- **完成时间**: 2026-05-17 → 2026-05-18 验证
- **改动清单**:
  - `src-tauri/src/provider.rs` — ProviderMeta 加 `vision_model` 字段
  - `src-tauri/src/proxy/model_mapper.rs` — ModelMapping 加 `vision_model` + `has_image_content()` 检测函数
  - `src-tauri/src/proxy/handler_context.rs` — build_context_for_claude() 加 vision routing 逻辑
  - `src-tauri/src/proxy/handlers.rs` — ctx 创建后同步 body model（防止 model_mapper 覆盖）
  - `src/types.ts` — ProviderMeta 加 `visionModel` 类型
  - `src/components/providers/forms/ProviderAdvancedConfig.tsx` — 新增 Vision Model 输入框
  - `src/components/providers/forms/ProviderForm.tsx` — visionModel 状态管理 + nextMeta 提交
- **实际测试结果**:
  ```
  Vision routing: detected image content, switching model claude-opus-4-7 -> mimo-v2.5
  Vision routing: updating body model claude-opus-4-7 -> mimo-v2.5
  ```
  ✅ 路由生效，body model 正确同步

## Phase 2: 项目级 Provider 绑定 ✅
- **状态**: 完成 + 实际测试验证通过
- **完成时间**: 2026-05-17 → 2026-05-18 验证
- **改动清单**:
  - `src-tauri/src/proxy/session_project_router.rs` — 新模块，扫描 ~/.claude/projects/ JSONL
  - `src-tauri/src/proxy/mod.rs` — 注册新模块
  - `src-tauri/src/proxy/server.rs` — ProxyState 加 session_project_router（Arc<Database>）
  - `src-tauri/src/proxy/handler_context.rs` — session→project→provider 查找+切换
  - `src-tauri/src/proxy/handler_context.rs` — get_providers 确保 project-routed provider 在列表首位
  - `src-tauri/src/provider.rs` — ProviderMeta 加 `project_providers` + `project_dirs`
  - `src-tauri/src/commands/project_routing.rs` — Tauri 命令 + UI 数据
  - `src-tauri/src/components/projects/ProjectRoutingPage.tsx` — 项目管理页面
  - `src-tauri/src/components/providers/forms/ProviderAdvancedConfig.tsx` — 移除项目路由部分
  - `src/App.tsx` — 工具栏 FolderTree 入口
  - `src/types.ts` — ProviderMeta 加 `projectProviders` + `projectDirs`
- **实际测试结果**:
  ```
  [ProjectRouter] session 18343e0f -> project /Users/keane/www/ayd_company/apd
  [ProjectRouter] Direct match: /Users/keane/www/ayd_company/apd -> 29061b48 (Volcengine)
  [handler_context] Project routing: session ... -> provider Volcengine
  [FO-001] 切换: claude → Volcengine
  ```
  响应模型：`glm-5.1`（Volcengine）✅

## Phase 3: 编译验证 + 修复 + 提交 ✅
- **状态**: 完成
- **完成时间**: 2026-05-18
- **修复的问题**:
  1. **空 cwd bug**: JSONL 首行 `type=permission-mode` 有 sessionId 但 cwd 为空，`or_insert_with` 缓存空值 → 只在 cwd 非空时插入
  2. **vision body 不同步**: ctx.request_model 改了但 body["model"] 没改 → handlers.rs 中同步
  3. **get_providers 不包含 project-routed provider**: forwarder 迭代 providers 列表找不到 → 插入到首位
  4. **handler_context body 引用不可变**: 改为在 handlers.rs 中处理
- **编译**: cargo build --release ✅ (3m53s)
- **测试**: curl 模拟请求 ✅
- **推送**: git push ✅

## Bug 修复记录

| Bug | 根因 | 修复 |
|-----|------|------|
| 项目路由不生效 | JSONL 首行 cwd 为空被缓存 | 只在 cwd 非空时插入映射 |
| project-routed provider 不在 forwarder 列表 | get_providers 只返回当前+故障转移 | 插入 effective_provider 到首位 |
| Vision model 被覆盖回默认模型 | body["model"] 未同步，model_mapper 重新映射 | handlers.rs 中同步 body model |
| 空 cwd 导致 session→project 全空 | 3处扫描逻辑都有此 bug | 统一修复 |
