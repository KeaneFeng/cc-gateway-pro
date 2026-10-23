# JetBrains 申请前最终核查

核查日期：2026-10-23（Asia/Shanghai）。对照用户提供的《CC-Gateway-Pro：JetBrains All Products Pack 开源申请前优化清单》，逐项复核公开代码 `24a6275e853a91a67623f90c01fa94c541e6dd55`。原清单作为验收依据，不把其中“仅执行第一阶段”的旧指令当作本轮任务；本轮按用户要求核查并完善材料。未提交申请，仅按用户授权启用 GitHub 私密漏洞报告；未改发行资产或分支历史，本轮文档修改未 commit/push。

**结论：工程修复和申请草案已完成，申请仍有事实待确认及许可/发行缺口，不能标记为全部完成。**

## 逐项对照原清单

| 原清单              | 状态                           | 已完成及证据                                                                                                                                                                       | 剩余缺口 / 下一步                                                                                |
| ------------------- | ------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| §1 核心原则         | 已落实                         | 保留根 MIT 版权、上游归属及真实历史；未虚构指标或用户；[来源说明](project-history.md)                                                                                              | 继续避免“完全原创”“全面合规”“保证获批”等表述                                                     |
| §2 仓库事实审计     | 已完成当前快照                 | [审计](jetbrains-application-audit.md)与下方 API/命令；公开、main、独立仓库但含上游代码                                                                                            | 上游导入的精确 revision 未完全建立；逐行原创性不做无证据计数                                     |
| §3 P0 来源与许可证  | 部分完成                       | 五语言介绍使用“参考”及对应译文；代码复用历史和 MIT 原文保留；1305 个依赖的元数据、1290 项声明/文件头已收集；19 项缺口补齐                                                          | 15 项声明未补；其余品牌来源及使用权限、完整分发合规仍未结案                                      |
| §4 P1 独立成果      | 已完成证据表                   | [贡献矩阵](original-contributions.md)覆盖原清单六类功能及 Session Traces；区分上游基础能力与项目改进                                                                               | 不把全部贡献者/提交数当作本项目独创；未声称逐模块完全重写                                        |
| §5 P1 README / 展示 | 已完成文档优化                 | 五语言 README、真实仓库图示、安装/开发入口、历史/许可证/贡献链接                                                                                                                   | 工具×系统兼容矩阵未逐组合实测；Homebrew 渠道未实机验收                                           |
| §6 P1 发布与维护    | 部分完成                       | 43 个真实发行；[最新版本](https://github.com/KeaneFeng/cc-gateway-pro/releases/tag/v4.0.17)有三平台资产；贡献/安全文档、Issue 模板、Roadmap 及补充版本说明存在                     | 中间版本摘要已补齐（见下节）；线上 release body 仍通用；新资源尚未正式发行；三平台安装升级未验收 |
| §6 CI 与基础测试    | 已完成当前代码验证             | [托管 CI](https://github.com/KeaneFeng/cc-gateway-pro/actions/runs/39893831322)两项 job 均成功；609 前端、2644 Linux 后端测试通过，0 失败，3 忽略；类型/格式/Clippy/许可摘要均通过 | 新版 release workflow 尚未正式运行；CI 不是完整应用端到端或生产签名验证                          |
| §6 SECURITY 可用性  | 已启用并核验                   | API 回读 `enabled: true`，公开页面有报告按钮；SECURITY 已更新                                                                                                                      | 未创建测试报告，报告送达/通知未做端到端验证                                                      |
| §7 P2 社区影响      | 已完成真实证据整理             | 8 stars、3 forks、529 次资产下载事件、233 次安装包/归档下载；外部 PR #8 关闭未合并                                                                                                 | 无独立用户/成功安装数，无已核实下游案例；49 个贡献账户包含上游/机器人，不能当作本项目团队规模    |
| §8 英文申请草案     | 已完善，待本人确认             | [英文草案](jetbrains-application-draft.md)：100–150 词概览、职责、4 类改进、指标、IDE 用途、署名、证据；另附实际表单映射和精简描述                                                 | 6 人席位证据、个人联系方式、非商业用途、既往许可经历和使用需求待补；未在远程表单中填写           |
| §9 交付与验收       | 文档与工程证据完成，整体有缺口 | 审计、贡献表、草案、版本说明与改动核验记录齐全；已有三笔提交公开                                                                                                                   | 不能把尚未结案的许可证/资产/平台项标为完成；申请未提交                                           |
| §10 维护者信息      | 部分确认                       | 用户确认身份 KeaneFeng、希望 6 份 All Products Pack、称有 6 人参与；可见维护从 2026-05-18 开始，存在持续上游整合及独立改进                                                         | 六人完整名单/持续直接贡献证据、仅非商业用途、拟用 IDE、资助/商业雇佣背景仍未核实                 |

## 当前可核验数据

- 仓库 API：公开、`fork: false`、默认分支 `main`，创建于 `2026-05-17T15:56:52Z`；最新推送 `2026-10-23T03:13:12Z`。可见代码历史始于北京时间 2026-05-18；最新三笔维护提交为 2026-10-23。
- `git rev-list --count HEAD`：304；GitHub 将 193 个贡献归于 KeaneFeng，含导入/维护等混合工作，不是 193 项原创发明。API contributors 为 49 个账户，不是 49 位本项目直接参与者。
- 非草稿发行 43、预发行 0；最新 v4.0.17 于 `2026-10-20T10:46:44Z` 发布。资产累计下载 529，按 dmg/exe/msi/deb/rpm/AppImage/zip/app.tar.gz 筛选为 233，均非独立用户数。
- 全部分页 issues API 返回 37 条，均为 PR：35 个 dependabot、1 个 KeaneFeng、1 个 onepaperbox。外部 [PR #8](https://github.com/KeaneFeng/cc-gateway-pro/pull/8) 已关闭、未合并，不能当作稳定参与团队证明。
- [CI 39893831322](https://github.com/KeaneFeng/cc-gateway-pro/actions/runs/39893831322) 对应 `24a6275e8`，结论 success。此前 v4.0.17 的失败记录保留，后续修复没有改写其历史结果。

## 六个席位的证据边界

用户确认希望申请 6 份全家桶，称参与者共 6 人。申请草案保留该需求，**没有自行改成 1 份，也没有自动从贡献排行榜凑齐 6 人**。

[官方表单](https://www.jetbrains.com/shop/eform/opensource)明确要求席位接收者具有持续、可见的活跃贡献，并排除纯非代码提交作为活跃开发证明；还要求仅用于非商业开源开发。一个 All Products Pack 席位已包含多款 IDE，席位数不是 IDE 数量。

| 证据对象              | 已核实事实                                                                                                            | 是否可直接支持本次席位名单                                 |
| --------------------- | --------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------- |
| KeaneFeng             | 用户确认申请身份；项目路由、Vision、日志及 Traces 提交见贡献表                                                        | 可列为已确认申请人；用途/个人声明仍需本人确认              |
| farion1231 等上游作者 | 保留在本仓库历史的上游作者贡献                                                                                        | 不自动等于本次参与申请或实际使用许可证                     |
| hulkbig               | 本地 `a0b863f16` 标题/作者对应 [cc-switch PR #8322](https://github.com/farion1231/cc-switch/pull/8322)，由 Keane 移植 | 该证据支持上游归属，不足以证明本项目直接持续参与及席位需求 |
| xue-ding-e            | 本地 `471cf6d02` 对应 [cc-switch PR #8281](https://github.com/farion1231/cc-switch/pull/8281)，由 Keane 移植          | 同上                                                       |
| onepaperbox           | 仅核实本项目外部 PR #8，关闭未合并                                                                                    | 不能只据此认定为持续活跃贡献者                             |
| 其余拟申请人          | 用户尚未给出对应账号清单与贡献链接                                                                                    | 待补充；不以未知姓名、机器人或上游身份代填                 |

未据此否认用户所说的六人团队；结论仅是公共记录还不能把这六个席位逐一对应到具名活跃贡献者。建议补充每人：GitHub 账号、近期项目代码提交/PR、持续职责、实际用证需求和参与申请确认。不在公开仓库存放个人邮箱等不必要信息。

## 尚未关闭的来源与发行缺口

- 15 项依赖：`@open-draft/deferred-promise`、`dlv`、`html-parse-stringify`、`is-node-process`、`react-remove-scroll-bar`、`stackback`、`strict-event-emitter`、`tr46`、`convert_case`、`fax`、`fax_derive`、`mac`、`rustls-platform-verifier-android`、`seahash`、`tao-macros`。确切版本及缺失原文、403/404、版本映射不足等原因见[逐项记录](legal/dependency-follow-up.md)。本轮复核记录与摘要，未将所有历史请求重做，也未宣称远端状态永远不变。
- 品牌素材：49 个文件和 49 个内联 SVG 已与 Lobe Icons 精确匹配并收集 MIT 声明；其余 51 个品牌文件和 21 个内联条目仅确认 Git 沿革。来源库的版权许可不自动证明独立商标授权，详见[来源表](legal/brand-asset-sources.md)。
- 平台分发：本地 macOS debug 包的 10 个资源已核对；Windows/Linux 包内声明、系统依赖义务、正式签名及三平台安装升级未做最终验收。现有发布资产不会因 main 的配置更新而自动改变。

## 申请判断与后续动作

当前可以审阅和继续完善申请材料；不能据此承诺获批或勾选尚未确认的个人声明。[官方项目页面](https://www.jetbrains.com/community/opensource/)强调成熟项目、活跃维护及开发者社区影响，按个案审查。现有公开指标支持“仍处于较小规模”，不支持“广泛采用”。这不是据 Star 数自行设定硬性资格门槛。

下一步优先补齐六人席位依据及用途声明，私密安全报告渠道已启用并核验；继续解决原始许可和品牌来源、平台分发缺口。可通过回应真实问题、发布具体版本说明和收集自愿公开案例完善社区证据；不制造数据或重写历史。本次仅按授权启用私密漏洞报告；没有联系第三方或提交申请。

## 复核与材料验证

```bash
gh api repos/KeaneFeng/cc-gateway-pro
gh api 'repos/KeaneFeng/cc-gateway-pro/contributors?per_page=100' --paginate
gh api 'repos/KeaneFeng/cc-gateway-pro/commits?per_page=100' --paginate
gh api repos/KeaneFeng/cc-gateway-pro/releases --paginate --slurp
gh api 'repos/KeaneFeng/cc-gateway-pro/issues?state=all&per_page=100' --paginate
gh api repos/KeaneFeng/cc-gateway-pro/private-vulnerability-reporting
gh run view 39893831322 --json headSha,status,conclusion,jobs
git rev-list --count HEAD
python3 scripts/audit-third-party-licenses.py --check
```

本轮只改文档，未改源码、依赖锁文件或生成许可证数据；不把先前 CI 成功说成本轮未提交文档已通过新的 CI。最终本地检查覆盖 Markdown 格式、相对文件链接、英文概览词数、许可证摘要及 `git diff --check`。

## 本轮可关闭缺口的补充结果

- 私密漏洞报告：启用操作成功；两次 API 回读均为 `enabled: true`；匿名访问公开 Security 页面可见报告按钮，目标 `/KeaneFeng/cc-gateway-pro/security/advisories/new` 正确。未填写或提交漏洞报告。
- 中间发行记录：[历史摘要](release-notes/published-history.md)覆盖 v3.19.1 后所有 13 个正式发行，按发布时间排序并核验相邻 tag 祖先关系；v4.0.5 的版本号变更、v4.0.11/v4.0.16 的撤销操作均明确保留。未补造未发布版本。
- Homebrew：tap 存在，固定 revision 的 cask 指向 v4.0.17 arm64 DMG，SHA-256 与 release API 的 digest 一致。未执行安装、升级或移除 quarantine 的 postflight；Intel 与其他平台安装体验未据此认证。
- 15 项许可和未知品牌来源仍需对应版本原文或权利证据；本轮未获得能够改变既有结论的新证据，不通过改写声明关闭缺口。6 人名单及非商业用途仍不能由代理代为确认。
