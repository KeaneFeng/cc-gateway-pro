# JetBrains 申请前事实审计

核验日期：2026-10-23（Asia/Shanghai）。源代码基准：`8b68d91b8f74a4db271076829273f18dce7a8ec0`，本地 `main`、`origin/main` 与 v4.0.17 对应。数字是当日快照，不是持续增长承诺。第一轮为文档审计；第二轮增加测试隔离修复、工具链和许可证资源配置，详见下节。未提交、推送、重写历史、替换许可证或发布申请。已有 `.mimosa/` 未跟踪目录未改动。

## 第二轮优化与本地验证（2026-10-23）

以下结果更新第一轮审计中的待办状态；后文 GitHub CI 和发行数字仍保留为历史快照。

- **CI 根因**：原线上失败位于 `session_usage_codex.rs:3384`，即 `parent_timelines.len() == 1`，并非纳秒过滤结果。分页基线恢复测试未加入现有 serial 锁，却会填充全局缓存；其他同步测试也会读写共享状态。补齐这些测试的 `#[serial_test::serial]`，保留所有原始断言，不改变生产统计逻辑。
- **复现边界**：原测试单独通过；修复前本地 20 次并发模块执行也通过，未把概率失败冒充为本地必现。根因依据线上明确失败行及共享状态调用链。修复后模块以 32 个测试线程执行 20 轮全部通过。
- **完整验证**：`cargo test --locked` 合计 2657 passed / 0 failed / 3 ignored（15 个测试 binary）；`cargo clippy --locked -- -D warnings`、`cargo fmt --check` 和配置变更后的 `cargo check --locked` 通过。前端 89 文件 / 609 测试通过，TypeScript、源码格式检查与 renderer 构建通过。构建仍有大 chunk 提示，没有为了此任务改动打包策略。
- **工具链**：CI/Release 读取 `.node-version`，Rust 固定 1.95，packageManager 固定 pnpm 10.12.3；冻结锁文件安装完成。未升级系统全局 pnpm。
- **许可证**：新增锁文件级依赖清单、原始声明合集、补充来源及 204 个资产的来源状态清单；在 Tauri 中配置未来发行包携带声明，并加入 CI 新鲜度校验。具体覆盖和剩余原始声明缺口见 [许可证报告](legal/third-party-license-audit.md)。不能把元数据清点等同于最终授权清查。
- **表述**：五语言 README 改为“参考 cc-switch”的产品介绍，移除首屏来源徽章；详细代码复用来源、MIT 版权和贡献证据仍保留。
- **未执行**：没有 commit/push，因而没有新的远程 CI 结果；未签名、发布或实机安装三平台包。现有发布包不会因本地资源配置变化自动补入声明。

## 第三轮：来源补证与提交准备（2026-10-23）

- 此前 34 个依赖原文缺口补齐 19 个，剩余 15 个；[逐项记录](legal/dependency-follow-up.md)说明了缺失文件、不可访问 revision、版本映射不足等具体原因。
- 历史锁定的 Lobe Icons 1.73.0 包通过 SHA-512 验证，49 个文件与 49 个内联 SVG 逐字节匹配；原 MIT/版权声明已纳入分发资源。其余品牌素材的 Git 沿革与证据边界见 [品牌来源表](legal/brand-asset-sources.md)。
- 扫描器补齐 AUTHORS 与 LICENSES 目录识别；缺元数据时拒绝覆盖旧报告，旧 Cargo metadata 不匹配锁文件时拒绝生成；8 项审计工具测试通过。
- 本地 macOS 调试 .app 构建及资源 SHA-256 核对通过，未安装、正式签名、公证或发布。原有 2657 个后端测试和 609 个前端测试结果仍适用于未继续修改的运行/测试代码；本轮新增验证集中在审计工具与实际资源打包。
- 本轮只判断提交准备度，未 stage、commit 或 push。现有 `.mimosa/` 仍不属于提交范围。提交审计改进不等于宣称剩余授权问题解决。

## 第一轮结论（历史快照）

项目具有可验证的功能开发和三平台发行记录，但不能据此宣称完全原创、全部 CI 通过或已有广泛社区影响。最重要的申请前缺口为后端测试失败、完整第三方许可证清查及维护者申请信息。本文和英文草案不保证审核结果。

## 仓库与维护周期

| 项目             | 核验结果                                                                     | 证据                                                                                                                                                                  |
| ---------------- | ---------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 可见性与仓库类型 | public；GitHub `fork=false`，无 parent；代码来源为 cc-switch 导入            | [仓库 API](https://api.github.com/repos/KeaneFeng/cc-gateway-pro)；[根提交](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb085)                             |
| 默认分支         | main                                                                         | 同上                                                                                                                                                                  |
| 创建时间         | 2026-05-17 15:56:52 UTC（北京时间 05-17 23:56:52）                           | API `created_at`                                                                                                                                                      |
| 首个可见本地提交 | 2026-05-18 00:01:56 +08:00，导入 cc-switch v3.15.0                           | `git log --reverse --format=fuller`                                                                                                                                   |
| 最近维护         | 10-20 18:31:09 +08:00 版本提交；此前 10-19 有功能修复提交                    | [提交历史](https://github.com/KeaneFeng/cc-gateway-pro/commits/main/)                                                                                                 |
| 可观察维护窗口   | 2026-05-18 至 2026-10-20，约五个月；不是继承上游的项目年龄，也不证明每日投入 | Git 作者时间和 GitHub 发布时间                                                                                                                                        |
| Stars / Forks    | 8 / 3                                                                        | 仓库 API；两者均不等于活跃用户数                                                                                                                                      |
| Releases         | 43 个非草稿发行、0 个预发行；最新 v4.0.17 发布于 2026-10-20 10:46:44 UTC     | [发行 API](https://api.github.com/repos/KeaneFeng/cc-gateway-pro/releases?per_page=100)、[最新发行](https://github.com/KeaneFeng/cc-gateway-pro/releases/tag/v4.0.17) |
| 下载             | 非草稿资产累计 529 次；安装包/归档后缀筛选累计 233 次                        | API assets.download_count；筛选 dmg/exe/msi/deb/rpm/AppImage/zip/app.tar.gz，排除签名和 latest.json；不是独立用户或成功安装数                                         |
| Contributors     | API 返回 49 个账户，包含上游作者及机器人                                     | [contributors API](https://api.github.com/repos/KeaneFeng/cc-gateway-pro/contributors?per_page=100)；不能当作 49 位本项目外部贡献者                                   |

## Git 贡献统计与边界

- `git rev-list --count HEAD`：301 个可达提交。
- `git rev-list --max-parents=0 HEAD`：仅根提交 `0f69cb08506238d89bae17d9fbf23a70eb92e25f`；该提交是大规模代码导入，不能算作全部独立开发成果。
- `git shortlog -sn HEAD`：作者名 Keane 对应 190 个提交；其余 111 个保留其他作者。GitHub API 将 190 个贡献关联到 KeaneFeng，但作者字段不证明每行代码的原创性。
- 全历史 63 个提交正文含 `cherry picked from commit`，均不在 Keane 的 190 个中。这是显式移植的下限，不是全部上游移植数量。
- Keane 的提交标题中，45 个包含 `sync`、`upstream` 或 `port `（不区分大小写）。这是可复现的粗筛标签，包含同步产品功能等潜在误报，不能机械扣除后声称剩余都是原创。
- 无共同祖先：`git merge-base HEAD cc-switch-ref/main` 无结果。上游 Git 历史没有作为本项目祖先导入，但上游代码确实存在。
- 不给出“独立原创提交总数”：需逐提交判定代码来源、混合改动及合并内容。已证实的具体成果列于 [original-contributions.md](original-contributions.md)。

本地 upstream 参考 `cc-switch-ref/main=0cda8d46c7f74718c46c2ef47335bf1a8e50f7af` 的作者日期为 2026-07-02，不能用作 10-23 最新上游功能全集。未为了此审计更新分支或同步代码。

## P0：来源与许可证

当前 LICENSE、导入根提交 LICENSE 均为 MIT，版权行为 `Copyright (c) 2025 Jason Young`，SHA-256 均为 `912b6a597d10c43b40a0909349ed95b052b17efb6502b4898e1b35dafb896755`。本地 upstream LICENSE 相同，在线 [上游 license API](https://api.github.com/repos/farion1231/cc-switch/license) 识别为 MIT。

原五语言 README 只写 `MIT © Keane Feng`，与 LICENSE 不一致；已改为链接原许可证并将维护者身份单独说明。LICENSE 原文未改。原 README 把本地网关、用量分析等笼统描述为后续扩展，已明确它们属于继承能力。

风险及未完成项：

1. 保留完整 MIT 版权、许可与免责声明；不能以更改 README 代替许可证保留。参阅 [项目历史](project-history.md)。
2. `git ls-files '*LICENSE*' '*NOTICE*'` 只返回根 LICENSE；没有发现 tracked NOTICE，不等于不存在依赖或资产署名义务。
3. 未完成所有 Rust/npm 传递依赖、图标、字体和发布包的许可证兼容性审查，不能签署“所有依赖均为 MIT/完全合规”的声明。
4. 尚未证明精确导入的 upstream commit，也未逐文件判定所有代码归属；不得使用“完全重写”“完全原创”。
5. package.json 的 Jason Young author 信息未移除，不为了突出维护者而擦除来源。

## 功能与项目展示

[贡献表](original-contributions.md)列出清单中的六类能力及 Session Traces，分别指向源文件、改动和发行。项目路由与 Vision Model 路由可追溯到 `f93b21eb8`；终端日志查看可追溯到 `123624bf9`；Session Traces 保留/裁剪改进可追溯到 `672e1e1d8`。网关、failover 和 usage logger 在根提交已经存在。

已发布状态仅指代码进入发布标签、对应资产存在；本次未进行三平台安装、设备级操作或登录供应商后的完整功能验收。没有把历史计划文件等同于新功能；后续工作集中于 [roadmap](roadmap.md)。保留 README 已有工具矩阵和架构图，其全部排列组合未做实时集成测试。SVG 预览明确标注为演示素材，不当作真实用户使用证据。

## 工程、发行与社区

| 检查项                    | 结果与改进                                                                                                                                                                                                                                                                                                                                  |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| CI                        | [39408533954](https://github.com/KeaneFeng/cc-gateway-pro/actions/runs/39408533954)：Frontend Checks 成功；Backend Checks 的 Run tests 失败。2522 passed / 1 failed / 3 ignored；失败项 `services::session_usage_codex::tests::test_parent_rollout_nanosecond_cutoffs_are_exact`，断言 2 != 1。当时未修复；第二轮已修复本地测试隔离，见上节 |
| 发行构建                  | [39409186505](https://github.com/KeaneFeng/cc-gateway-pro/actions/runs/39409186505) 成功；最新资产有 macOS arm64、Windows x64、Linux x64；未实机验收                                                                                                                                                                                        |
| 工具链                    | 本地 Node 22.12.0、Rust 1.95；CI 请求 Node 20 / Rust stable，Release 请求 Node 22 / Rust stable，仍需统一并通过托管验证                                                                                                                                                                                                                     |
| 安装指南                  | 改正最新版不存在的 Windows portable ZIP；不再承诺未经确认的 AUR 包；Linux 改为 WebKitGTK 4.1/libsoup 3 与实际构建平台说明；Homebrew 渠道保留原说明但本次未实机验证                                                                                                                                                                          |
| CONTRIBUTING              | 修复旧 Node/Rust/pnpm 要求、无实际脚本的 pnpm lint、错误 locale 路径；四个实际 UI locale；添加锁文件及本地/原生验证边界                                                                                                                                                                                                                     |
| SECURITY                  | 保留私密 GitHub Advisory 报告渠道，修正旧产品名及 Latest 3.x；原文 v3.18.0 威胁模型的历史验证范围没有伪装成最新安全审计；报告入口的实际启用情况待维护者确认                                                                                                                                                                                 |
| CHANGELOG / Release Notes | 原 CHANGELOG 最新条目为 3.19.1，最新 GitHub release body 只有通用安装句；已按真实 tag diff 补写 4.0.17，并明确中间版本记录仍不完整                                                                                                                                                                                                          |
| 架构、贡献、模板          | 已有架构/代理/用户手册，CONTRIBUTING、SECURITY、行为准则和 bug/feature/doc/question 模板；新增材料提供入口，不复制创建占位文件                                                                                                                                                                                                              |
| 社区反馈                  | issues API 的 37 条记录均为 PR，其中 35 个 dependabot、1 个维护者 PR、1 个外部 PR；[onepaperbox 的 #8](https://github.com/KeaneFeng/cc-gateway-pro/pull/8) 已关闭且 merged_at=null；这是真实外部提议，不是已合并贡献。没有查到普通 Issue，不证明没有用户                                                                                    |
| 引用与下游案例            | 本次无可验证证据；待人工确认，不虚构                                                                                                                                                                                                                                                                                                        |

## 按优先级排序的修改计划与落实

| 优先级 | 文件                                                                                      | 原因与状态                                                                                   |
| ------ | ----------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- |
| P0     | README.md / README_ZH.md / README_JA.md / README_ES.md / README_SV.md；project-history.md | 已纠正原创性及版权表述，保留上游来源                                                         |
| P1     | original-contributions.md；jetbrains-application-draft.md                                 | 已创建证据表和英文草案；维护者身份声明需本人最终核实                                         |
| P1     | CONTRIBUTING.md；SECURITY.md                                                              | 已修复可核实的过期说明                                                                       |
| P1     | CHANGELOG.md；release-notes/v4.0.17-en.md                                                 | 已补真实版本差异与 CI 限制；未修改 GitHub 线上 release                                       |
| P1     | roadmap.md                                                                                | 已记录 CI 修复、工具链、完整许可证清查和平台验收；均未假称完成                               |
| P2     | 公开反馈与申请提交                                                                        | 可自然完善文档、响应真实问题、发布具体版本说明；不制造 Star、下载、PR 或历史；未代为提交申请 |

## 维护者需要确认的事实

- 首次可见维护为 2026-05-18；是否有更早且可公开验证的投入？不要追溯编造。
- 申请主体是否确为 KeaneFeng；哪些职责由本人承担、是否存在资助或商业雇佣关系？
- 拟使用哪些 JetBrains IDE？草案只按 Rust/TypeScript/SQLite 工作流提出 RustRover、WebStorm、DataGrip 用途，不声称已有使用记录。
- 是否有可公开引用的实际用户、渠道或反馈？是否允许引用？
- 私密漏洞报告入口是否启用？第三方资产/依赖许可证审查由谁完成？

[JetBrains 当前官方页面](https://www.jetbrains.com/community/opensource/)将 All Products Pack 支持描述为面向有活跃维护及较广开发者影响的成熟开源项目，按个案审核。不能把旧清单或过去的固定月份、Star 门槛当作当前保证。申请草案不代表资格已获确认；实际表单字段须提交时再次核对。

## 复核命令

```bash
gh api repos/KeaneFeng/cc-gateway-pro
gh api repos/KeaneFeng/cc-gateway-pro/releases --paginate --slurp
gh api 'repos/KeaneFeng/cc-gateway-pro/contributors?per_page=100'
gh api 'repos/KeaneFeng/cc-gateway-pro/issues?state=all&per_page=100'
gh api repos/KeaneFeng/cc-gateway-pro/pulls/8
gh run view 39408533954 --json jobs
gh run view 39408533954 --log-failed
git rev-list --count HEAD
git shortlog -sn HEAD
git log --format='%H %an %s%n%b'
git show 0f69cb085:LICENSE
git log --format=fuller v4.0.16..v4.0.17
```

计数时排除 draft releases，分页到底；下载不含用户去重；GitHub contributors 与 Git 作者统计口径不同。README 链接到本地新增文档，在提交并推送前不会出现在远程默认分支。

## 第一轮文档验证

- 14 个新增或修改 Markdown 文件通过仓库内 Prettier 格式检查；相对文件链接检查未发现缺失目标。
- 英文 Project Overview 为 115 词，满足 100–150 词要求。
- `git diff --check` 通过；LICENSE 与运行代码未改动。
- 本机全局 pnpm 因要求 Node >=22.13 而拒绝在 Node 22.12.0 启动；本次直接执行仓库内 `node node_modules/prettier/bin/prettier.cjs` 完成格式校验。开发指南明确采用 CI 的 pnpm 10.12.3，未升级本机环境。
- 第一轮文档变更未重跑应用测试；CI 结果来自上文真实线上记录，不等于当前修改后的新 CI 验证。
