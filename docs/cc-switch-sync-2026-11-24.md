# CC Switch stable selective sync — 2026-11-24

**This is a selective maintenance sync, not full v4.0.40 parity. No version bump, tag, release, push or main merge.**

## Exact anchors

- Fork baseline: `b96f9b53bfcd56c9d76ace783ae4cb19caa00b0b` (v4.0.37 label; prior selective ports, not an upstream merge).
- Comparison boundary: upstream v4.0.37 commit `a8ca5e83a28c806463c03d239f7d667260a4ec4c` (annotated tag object `903a1622662aab2c8f55302c6427594ca223d0df`).
- Stable target: v4.0.40 commit `f19a9d4f4f08bda03106a7be87c77cbcdf55b019` (annotated tag object `3da554daf7d5db5feb530f975181371b3ce41133`; 110 subsequent commits).
- Observed main: `e6fd4accd257406bf9141baa402caaa78f975ba1`; eight post-release commits excluded.
- Privacy prerequisite: original `93920f87c313952904f43a166ef73195b04987c0`, cherry-picked as `76012afc6` with source trailer.
- Historical v3.18 sync document is not the current anchor. Cached cc-switch-ref checkout is July; its remote was Nov19. Those references are not evidence of current upstream completeness.

## Previous Nov20 wave reconciliation

The 15 upstream-subject matches below identify what was already ported. Exact matching patches are marked; adapted patches need semantic review rather than SHA equality. There is no single upstream commit through which this fork contains every change. Earlier omitted features are not silently marked synchronized.

| Fork commit | Upstream commit | Patch identity | Subject |
|---|---|---|---|
| `3be8a03d6` | `d5dfa8305` | adapted | fix(proxy): normalize null required in Codex tool schemas (#7023) |
| `51d97cc42` | `5e9812e07` | identical | fix(claude): omit reasoning_content for Mistral tool history (#9388) |
| `3ef45a1d3` | `fd9794213` | identical | fix(proxy): merge Gemini tool results split across user messages |
| `e69473751` | `7d5fb6574` | adapted | fix(usage): keep Gemini streaming model attribution and accept Vertex token counts (#7740) |
| `bc74a7b9c` | `6489c5d82` | adapted | fix(env): preserve colons in Unix environment source paths (#7170) |
| `8cb439fc9` | `d39705271` | identical | fix(webdav): tolerate MKCOL 400 to unblock full sync chain (#5623) |
| `5fe9587df` | `0ec969ca2` | identical | fix(sync): report WebDAV/S3 send failures with their real cause |
| `118a0144b` | `168556b45` | identical | fix(xai): preserve valid default OAuth account when non-default expires (#9548) |
| `fe34ffbce` | `a062485ae` | identical | fix(xai): propagate transient OAuth token refresh errors in quota queries (#9526) |
| `55caae7f6` | `98d2781c6` | identical | fix(s3): use virtual-hosted URLs for Alibaba OSS, Tencent COS and Huawei OBS |
| `17ff73ad7` | `86aeb47a6` | identical | fix(skills): include the OS error when creating a skill symlink fails |
| `a0ffda425` | `f94b702af` | identical | fix(sync): skip skill links that do not resolve instead of failing the sync |
| `b7bf3a005` | `fb453fb5d` | identical | feat(pricing): add GPT-6.2 Sol and Astra prices |
| `3caae2911` | `210103c6c` | identical | feat(pricing): seed Gemini 4 Carbon (#9421) |
| `f5dbe1119` | `6daff81b6` | identical | feat(pricing): seed Qwen4 Max (#9612) |

## Stable target disposition ledger

Skipped = deliberately excluded non-functional maintenance; Deferred = not synchronized, no claim of functional equivalence. Ported = code/tests carried into this fork with adaptations described above.

| Upstream SHA | Decision | Subject |
|---|---|---|
| `b989bd6e0` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(provider): report additive providers that fail a batch live sync (#9747) |
| `5968c5023` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(skill): report apps that fail to relink after a storage move (#9748) |
| `09d2f116d` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | test: give each integration test process its own test home (#9752) |
| `33ee6d668` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | refactor(proxy): split the Responses→Anthropic SSE stream into per-event handlers (#9658) |
| `25c2e6b5e` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | feat(agy): support Antigravity account switching on Linux (#9774) |
| `2e2f3bbfa` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(sessions): filter empty/internal Antigravity sessions and parse structured SessionBlocks (#9266) |
| `b13ee2694` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(sessions): make the omp resume command pasteable on Windows (#9153) |
| `c73d8a48c` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(proxy): disambiguate colliding flattened tool names in Codex Chat bridge (#9554) |
| `c131f68e9` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(usage): dedupe proxy and session logs with provider model prefixes (#9249) |
| `82c29d09a` | PORTED: Adapted msg_ response item IDs; preserve fork Chat/Anthropic plumbing. | fix(proxy): generate msg_-prefixed IDs for converted Responses message items (#9756) |
| `9431a6bc9` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(copilot): retain pending legacy v1 auth migration on transient errors (#9549) |
| `49e82664a` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(providers): isolate pending saves between editor sessions (#9593) |
| `63798b896` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | feat(deeplink): 供应商一键导入链接支持 WorkBuddy (#9768) |
| `ee36a25b3` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(tray): restore macOS fullscreen windows when reopened from the tray (#9770) |
| `fc665e818` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(usage): prevent Gemini session double-counting after rollup and allow in-place updates (#9527) |
| `d15cb087c` | PORTED: Drop OpenAI-only strict tool flag on Anthropic bridge. | fix(proxy): do not forward OpenAI strict flag in Codex->Anthropic tool conversion (#8659) |
| `5a5111b0a` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(window): recover undersized persisted main window state (#9496) (#9504) |
| `52f78ba1d` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | perf(sessions): replace substring indexing with lazy metadata scans (#9823) |
| `3b08b1162` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | perf(usage): index filters and select pooled medians in one scan (#9825) |
| `37e3eabfe` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(providers): match vendor hosts instead of URL substrings (#9648) |
| `3b63dcabd` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | refactor(proxy): move the transform_* tests into their own files (#9802) |
| `81a5973e0` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | chore: stop re-exporting crate-root names nothing outside the crate uses (#9799) |
| `03e93b711` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | chore(format): check tests/ with Prettier too (#9792) |
| `5d6ed73bf` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | test(i18n): fail on CJK text written straight into JSX (#9791) |
| `3a7495bf8` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix: opencode v2 session edge cases (#7982) |
| `908a83eda` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(sessions): report a text index failure instead of empty hits (#9512) |
| `a489233e3` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(usage): compute Antigravity speed from native streaming duration (#9788) |
| `9455eb319` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(linux): open links and folders with the host xdg-open inside an AppImage (#9113) |
| `a18d91c56` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(codex): honor reasoning overrides for official model names (#9017) |
| `1159383db` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(usage): show Codex cache writes in the usage summary and request log (#8191) |
| `40f4bc900` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(agy): drop redundant Google prefix from Google AI plan names (#9834) |
| `a3a85a500` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | chore(ci): remove the unused Claude review workflow |
| `18744a32d` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(dsh): normalize reasoningEfforts keys before writing cordis.patch.yml |
| `49e5d9a00` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(tools): give the install enumeration probe the login-shell PATH |
| `a5fd82b6a` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(skills): report a non-ZIP repo archive instead of "0 skills found" |
| `78d2daf76` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | test(sessions): gate the Windows-only Oh My Pi test env helper |
| `6006b663d` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | chore(release): v4.0.38 |
| `2c084ec75` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | docs(release): add v4.0.38 release notes |
| `b7d2ad94b` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(proxy): improve GPT-6.2 prompt caching on the Codex OAuth route (#9848) |
| `3c115dd1f` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | chore(deps): drop the unused objc2 0.5 / objc2-app-kit 0.2 pair (#9805) |
| `0a8feb794` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | chore(lint): drop lint allows that suppress nothing (#9798) |
| `ff3d1d899` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | ci: run clippy on test targets too (#9796) |
| `477b06d97` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | refactor(update): drop the never-written legacy dismissed-version key (#9793) |
| `400d37d71` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | refactor(deeplink): drop import-result fallbacks for a backend that never existed (#9794) |
| `574dc5754` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | refactor(provider): one exhaustive remove-from-live for additive apps (#9749) |
| `0f5c767d6` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | refactor(skills): read installed skills through one SELECT and one row decoder (#9797) |
| `ea420cc07` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | refactor(codex): split Codex TOML editing out of providerConfigUtils (#9795) |
| `01e68a44b` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | refactor(skill): split the SkillService impl into per-concern files (#9804) |
| `5fd8300f2` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | refactor(services): move the skill, usage_stats and session_usage_codex tests into their own files (#9801) |
| `e19640fb6` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | refactor(mode): move controller.rs tests into their own files (#9800) |
| `c8b5481d8` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | refactor: move the codex_config and tray tests into their own files (#9803) |
| `06e7d4d56` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | refactor: move the tool_discovery and database backup tests into their own files (#9806) |
| `6c86107fb` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | refactor(proxy): move the streaming_responses tests into their own file (#9815) |
| `4ef1490e3` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | refactor(proxy): move the claude and codex provider tests into their own files (#9836) |
| `18a72b84b` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(ui): prevent overlay flicker during rapid toggles (#9176) |
| `43a825fc9` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(proxy): bound streaming header wait with the first-byte timeout on hyper (#9754) |
| `f346fb64e` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(backups): add an opt-in size limit for the backups folder (#9011) |
| `698b1b7f4` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | feat(providers): add a copy button to the API key input (#9232) |
| `32773e88c` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | perf(privacy): avoid Unicode copies and prune redundant scans (#9826) |
| `30958c041` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(deeplink): do not open import dialog when config merge fails (#9844) |
| `3feb336f3` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix: route home-dir lookups through get_home_dir (#9845) |
| `f2b3c12eb` | REVERTED/DEFERRED: ring evicts semantically important Claude usage corrections. Requires usage reducer before bounded retention. | fix(proxy): bound SSE buffering in usage/log collectors (#9846) |
| `4a1da640e` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(tools): fall back to the desktop app's bundled Codex CLI for runtime commands |
| `dcba4bb05` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(plugins): don't create config dirs for apps whose CLI isn't installed |
| `897232901` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(presets): offer only low/high/max efforts for OpenCode Go DeepSeek V4.1 Flash |
| `87c39ee1a` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | chore(release): v4.0.39 |
| `768eec9ca` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | docs(release): add v4.0.39 release notes |
| `a87a30691` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(usage): include first-token time in Antigravity output speed (#9858) |
| `fea70d7b6` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(tray): show managed official accounts in provider subtitles (#9863) |
| `c842b9930` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | refactor(provider): drop the unreachable current-pointer write in switch_normal (#9878) |
| `62c49e732` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | refactor(env): make backup_dir infallible and drop its is_ok test (#9880) |
| `eaa1fffa9` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | test(provider): cover switch guards and side effects no test pins (#9873) |
| `22015c9da` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | test: fold 105 same-shape tests into 25 table tests (#9874) |
| `b4b1fb465` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | test(providers): drop delegation tests covered by integration flows and the unused isLoading (#9875) |
| `bb36087ff` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | test: make four weak assertions pin the behavior they are named for (#9876) |
| `4ad2c4b17` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | test: drop 20 mutation-supported cases and two policy-only contracts (#9879) |
| `b1d7598d7` | PORTED: MCP import validation and complete confirmation fields; preserve fork color tokens and four existing locales. | fix(security): expose hidden MCP deep-link fields in confirmation UI and block credential-exfil keys (#9831) |
| `fcff029cf` | DEFERRED: body-copy/privacy hot-path reordering interacts with fork trace capture and vision/project routing. | perf(proxy): avoid redundant body copies and privacy passes (#9821) |
| `9b8de59a5` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | feat: enable plugins feature and add Antigravity support (#9882) |
| `d9ebf421d` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | perf(ci): speed up Rust and Vitest pipelines without reducing coverage (#9872) |
| `f14d142b9` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(plugins): check remote version before agy update and guide on in-use errors (#9885) |
| `4666b8f18` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(workbuddy): query enterprise usage for enterprise accounts |
| `d32c47360` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(plugins): ask to reinstall Antigravity plugins in legacy numeric dirs |
| `8e043d9ea` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(skills): list one skill per install name within a repository |
| `188836260` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | feat(auth-center): order and hide account cards like the sidebar apps |
| `e809b9fdc` | DEFERRED: Xiaomi MiMo balance/Token Plan feature not synchronized. | feat(usage): add Xiaomi MiMo balance and Token Plan quota queries |
| `a9b43b892` | DEFERRED: provider/model cache-hit-rate UI feature not synchronized. | feat(usage): show cache hit rate per provider and per model |
| `92f61db17` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(agy): discover T3 Code and standalone ACP sessions (#9857) |
| `5adff00c1` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | perf(apps): probe tool versions concurrently (#9418) |
| `2ccbfe2b3` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(apps): show installation status before version checks finish (#9886) |
| `d910a92d8` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(apps): don't offer npm upgrade for OpenCode Desktop CLI (#9099) |
| `77592e142` | DEFERRED: Grok official account-binding feature not synchronized. | feat(grokbuild): bind Grok Official cards to auth center xAI accounts (#9321) |
| `84e4612c7` | REVERTED/DEFERRED with #9846; final SSE files identical to fork baseline. | fix(proxy): enforce SSE pending limit per event instead of per chunk (#9867) |
| `a7b9343ec` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | refactor(prompts): make get_base_dir_with_fallback infallible (#9868) |
| `3da2c3cab` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | perf(sessions): index short queries and isolate WAL readers (#9827) |
| `1fba4e16c` | DEFERRED: native Responses regrouping depends on fork-missing Kimi history module. | fix(proxy): regroup interleaved items out of parallel tool-call batches (native Responses) (#8825) |
| `e7915ecfa` | DEFERRED: blocking-worker/once-only JS evaluation changes fork runtime semantics; network safety port does not claim this change. | perf(usage): run QuickJS once on a blocking worker (#9817) |
| `23b937b20` | DEFERRED: parallel cursor scans need cross-source usage dedup regression. | perf(usage): share cursors across parallel session scanners (#9818) |
| `4e10bd857` | DEFERRED: WAL/cache changes require backup/trace/project-router invalidation and migration regressions. | perf(database): enable WAL and cache proxy hot-path reads (#9820) |
| `31575905b` | DEFERRED: new trigger/view rejection and 512MiB/30s limits change manual+sync import contract; validate large full backups and local trace preservation together first. | fix(security): block triggers/views and bound size and execution time in SQL import (#9832) |
| `dce5608be` | PORTED: Bound usage HTTP responses, deny redirects/self-forward, escape template substitutions; retain fork JS runtime architecture. | fix(security): harden usage_script against redirect SSRF, self-forwarding, OOM, and JS injection (#9829) |
| `a09a99d7a` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(tests): compile sync_agy_usage helper on all platforms |
| `2372b1134` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | perf(sessions): cancel obsolete content searches across IPC (#9896) |
| `9f1c1bb54` | PORTED: Per-connection Host guard; fork has no configurable client-host allowlist, existing None behavior retained. | fix(security): enforce per-connection Host check on non-loopback proxy listeners (#9833) |
| `f08abf96f` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(kimi): support protocol 1.5 usage accounting (#9505) |
| `0e8a584b6` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | fix(tray): show xAI OAuth quota in the tray, including Codex stack members |
| `0c1121ce3` | DEFERRED: chunked Mistral thinking/text requires earlier transform/module refactors and stream regressions. | fix(proxy): parse chunked thinking/text content from Chat upstreams |
| `f81715e5d` | DEFERRED: outside selected safety/protocol maintenance scope; not implemented or accepted in this pass. | feat(presets): add StringAI as a full sponsor |
| `08f505b1d` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | chore(release): v4.0.40 |
| `f19a9d4f4` | SKIPPED: upstream-only maintenance/refactor; not needed for the selected fork fixes. | docs(release): add v4.0.40 release notes |

Counts: 5 ported, 37 skipped, 68 deferred.

## Fork invariants and acceptance boundary

- Preserve Session Traces capture/redaction and local-only cloud-sync behavior; manual full backup remains full.
- Preserve schema version and fork v11/v17 migration lineage. No schema upgrade in this sync.
- Preserve Claude/Codex per-project provider routing, Vision Model overrides, proxy takeover/restore and triple-channel logs.
- Preserve Hermes/OpenClaw/OpenCode integration, app identifiers/config dirs, deep links, updater URLs and CC Gateway Pro branding.
- Do not import upstream release workflows, sponsor assets, locale additions or missing app ecosystems.
- MCP invalid imported configs are rejected before persisted/live mutation; ordinary manually configured MCP remains outside this new gate.
- Host guard narrows local-peer requests even on wildcard listeners; fork has no custom host allowlist UI. `.localhost/.local/.internal` and IP policy remain as already supported.
- No actual user cloud account, real API key, installed app, live config, production DB or external proxy request used for acceptance.

## Validation

- `cargo test --lib --offline`: 2636 passed, 3 ignored, 0 failures (includes privacy and fork proxy/vision/project tests).
- `pnpm test:unit`: 92 files / 632 tests passed, including rendered MCP confirmation and risk classifier tests.
- `pnpm typecheck`, `pnpm format:check`, renderer production build: passed. Build retains existing >500 kB bundle warning.
- `git diff --check`: passed; no net changes to SSE parser/response collector after reverting the unsafe ring port.
- `cargo clippy --all-targets --offline -- -D warnings` and `cargo fmt --all -- --check`: passed.
- Integration binaries (`cargo test --tests --offline`) still building at this checkpoint; final result appended below.
- Tested code commit: `a1bc7a0bf870295aff0288ff9ba04d9cb3aaaa18`. Later ledger-only changes do not alter tested code.
- Real desktop packaged app, OS permission behavior on Windows/Linux, actual cloud sync services, OAuth accounts, and production API routes not exercised.

### Adaptation evidence and limits

- Initial SSE candidate failed a newly imported Images-API usage fixture: this fork lacks that upstream parser feature. More importantly independent review showed the bounded event ring can discard mid-stream Claude usage corrections and change billing. The entire SSE pair was reverted, not merely its failing test.
- The upstream full DeepLinkImportDialog test file depends on absent pending-link/custom-icon test infrastructure. It was not transplanted wholesale. MCP UI has rendered tests; new usage URL warning has type/build coverage and existing endpoint-risk unit coverage, not an end-to-end dialog test.
- The standalone trace-privacy fix is present as an explicit cherry-pick. Final branch is a sibling of main, not a deployment or replacement of the installed app.
