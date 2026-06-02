# Upstream Commit Analysis: 20 Commits Since Fork Base

Fork base: eaec3f66
Upstream HEAD: 8bf16602 (cc-switch v3.16.0)
Analysis date: 2026-06-01

## Classification Table

| # | Hash | Title | Classification | Affected Files | Fork Status | Conflict Risk |
|---|------|-------|----------------|----------------|-------------|---------------|
| 1 | 8bf16602 | fix(codex): always update model catalog JSON on provider switch | PORT | services/proxy.rs | MISSING | HIGH - proxy.rs heavily diverged (fork: 3317 vs upstream: 4474 lines) |
| 2 | afa09e12 | fix(usage): resolve per-app credentials for native balance/coding-plan queries | PORT | codex_config.rs, commands/provider.rs, provider.rs, proxy/providers/codex.rs, UsageScriptModal.tsx | MISSING | HIGH - provider.rs diverged (fork: 1121 vs upstream: 1447 lines) |
| 3 | 0960fd71 | fix: Claude Desktop official provider add error #3402 | PORT | claude_desktop_config.rs, commands/provider.rs, lib.rs, AddProviderDialog.tsx, useProviderActions.ts, api/providers.ts, query/mutations.ts, test | MISSING | MEDIUM - claude_desktop_config.rs same size but form components diverged |
| 4 | 5ef72a20 | fix(codex): multi-platform CLI discovery + gpt-5.5 template fallback | PORT | codex_config.rs, resources/gpt5_5_template.json (new) | MISSING | MEDIUM - codex_config.rs heavily diverged (fork: 1041 vs upstream: 2008 lines) |
| 5 | e02a2763 | fix: add kimi/moonshot to Anthropic tool thinking history normalizer | PORT | proxy/providers/claude.rs | ALREADY PORTED | NONE - fork has REASONING_VENDOR_HINTS and is_reasoning_vendor_identifier |
| 6 | c9cadd6e | Fix Codex OAuth auth being cleared during preserve-mode takeover | PORT | codex_config.rs, services/proxy.rs, SettingsPage.tsx, i18n (4 locales) | MISSING | HIGH - depends on 2683af57 + 3f59ab37, proxy.rs diverged |
| 7 | 60a9b330 | Refactor Codex live-write routing and cover default auth overwrite | PORT | codex_config.rs, tests/* (4 files) | MISSING | MEDIUM - depends on 2683af57, affects test infrastructure |
| 8 | f4e2c28a | Enrich Codex proxy forwarding-error responses with context | PORT | proxy/error_mapper.rs, proxy/handlers.rs | PARTIAL - handlers.rs has build_codex_proxy_error_response, error_mapper.rs MISSING mappings | LOW - error types already in fork's error.rs, just need mapping updates |
| 9 | 0e6f2b39 | Swap Shengsuanyun and AICodeMirror sponsor ads | SKIP | README*.md (4 files) | N/A | N/A - cosmetic only |
| 10 | 41433cfa | Add Codex restart hint after provider switch | PORT | useProviderActions.ts, i18n (4 locales), test | MISSING | LOW - simple additive change to switch notification logic |
| 11 | 3f59ab37 | Default Codex auth preservation to off (opt-in) | PORT | services/proxy.rs, settings.rs, CodexAuthSettings.tsx, useSettingsForm.ts | MISSING | HIGH - depends on 2683af57 being ported first |
| 12 | ee69c836 | Fix garbled output and false "not runnable" in Windows version probe | PORT | Cargo.lock, Cargo.toml, commands/misc.rs | MISSING | MEDIUM - misc.rs heavily diverged (fork: 2519 vs upstream: 4722 lines) |
| 13 | 2683af57 | Add Codex auth preservation setting | PORT | codex_config.rs, services/proxy.rs, settings.rs, CodexAuthSettings.tsx (new), SettingsPage.tsx, useSettingsForm.ts, i18n (4), schemas/settings.ts, types.ts | MISSING | HIGH - introduces new UI component and setting, many files |
| 14 | 8f83fa20 | docs: add Codex DeepSeek routing guides | SKIP | docs/* (11 files) | N/A | N/A - docs only |
| 15 | 47232cb0 | chore(release): bump version to 3.16.0 | SKIP | package.json, Cargo.toml, Cargo.lock, tauri.conf.json, release notes | N/A | N/A - version bump, covered by previous sync analysis |
| 16 | fe3eb7e6 | docs(changelog): add 3.16.0 release notes | SKIP | CHANGELOG.md | N/A | N/A - docs only |
| 17 | d905ed16 | Add UTM params to Atlas Cloud partner link | SKIP | README*.md (4 files) | N/A | N/A - cosmetic only |
| 18 | 94cc3d10 | Align Claude Desktop model mapping with Claude Code three-role tiers | PORT | claude_desktop_config.rs, ClaudeDesktopProviderForm.tsx, docs/*, i18n (4), test | LIKELY CONFLICT | HIGH - claude_desktop_config.rs same line count but form.tsx diverged (fork: 1193 vs upstream: 1141) |
| 19 | 058c9fb8 | Rename OpenCode Go preset to drop model suffix | PORT | claudeDesktopProviderPresets.ts, claudeProviderPresets.ts | UNKNOWN | LOW - simple preset rename |
| 20 | 85552cf4 | Add referral param to ShengSuanYun website links | SKIP | 7 provider preset config files | N/A | N/A - cosmetic/referral only |

## Summary

**SKIP (6 commits):** 0e6f2b39, 8f83fa20, 47232cb0, fe3eb7e6, d905ed16, 85552cf4
- Sponsor swaps, docs, version bumps, referral params - no code changes needed

**ALREADY PORTED (1 commit):** e02a2763
- kimi/moonshot normalizer already present in fork's claude.rs

**PARTIALLY PORTED (1 commit):** f4e2c28a
- handlers.rs has build_codex_proxy_error_response already
- error_mapper.rs still needs: AlreadyRunning/NotRunning/StreamIdleTimeout/ConfigError/InvalidRequest/AuthError status mappings

**NEEDS PORTING (12 commits):** 8bf16602, afa09e12, 0960fd71, 5ef72a20, c9cadd6e, 60a9b330, 41433cfa, 3f59ab37, ee69c836, 2683af57, 94cc3d10, 058c9fb8

## Porting Priority & Dependency Order

### Tier 1: Independent, low-risk ports (start here)
1. **41433cfa** - Codex restart hint (useProviderActions.ts, i18n) - LOW conflict risk
2. **058c9fb8** - OpenCode Go preset rename - trivial, 2 files
3. **f4e2c28a** (partial) - error_mapper.rs status mappings only - error types exist, just add mappings

### Tier 2: Bug fixes, moderate risk
4. **0960fd71** - Claude Desktop official provider fix - touches lib.rs, commands/provider.rs, AddProviderDialog.tsx
5. **afa09e12** - Per-app credentials for usage queries - new functions: extract_codex_base_url (fork has), resolve_usage_credentials, resolve_native_credentials

### Tier 3: Codex auth preservation feature stack (MUST be ported in order)
6. **2683af57** - Add Codex auth preservation setting (base feature)
7. **3f59ab37** - Default auth preservation to off (depends on #6)
8. **60a9b330** - Refactor live-write routing (depends on #6)
9. **c9cadd6e** - Fix OAuth auth clearing (depends on #6, #8)
10. **8bf16602** - Update model catalog on provider switch (depends on #6-#9)

### Tier 4: High-divergence ports (careful manual merge needed)
11. **5ef72a20** - Multi-platform CLI discovery + gpt-5.5 template - codex_config.rs is 2x larger upstream
12. **ee69c836** - Windows version probe fix - misc.rs is 2x larger upstream
13. **94cc3d10** - Claude Desktop model mapping alignment - form component has fork-specific changes

## Conflict Hotspots

1. **services/proxy.rs** (fork: 3317 vs upstream: 4474 lines)
   - 1157 lines of new upstream code, mostly Codex-related
   - Affected by commits: 8bf16602, c9cadd6e, 2683af57, 3f59ab37

2. **codex_config.rs** (fork: 1041 vs upstream: 2008 lines)
   - 967 lines of new upstream code
   - Affected by commits: afa09e12, 5ef72a20, c9cadd6e, 60a9b330, 2683af57, 8bf16602

3. **commands/misc.rs** (fork: 2519 vs upstream: 4722 lines)
   - 2203 lines of divergence
   - Affected by: ee69c836

4. **ClaudeDesktopProviderForm.tsx** (fork: 1193 vs upstream: 1141 lines)
   - Fork is 52 lines larger, indicating fork-specific additions
   - Affected by: 94cc3d10

## Fork-Specific Features at Risk

The fork (cc-gateway-pro) adds gateway proxy layer, multi-provider routing, and API key management on top of cc-switch. The following fork features could be impacted by porting:

- **Gateway proxy architecture**: proxy.rs changes must preserve gateway routing logic
- **Per-app credential resolution**: afa09e12 changes credential extraction - verify compatibility with gateway auth flow
- **Error handling**: f4e2c28a adds new error status mappings - verify gateway error types align
- **Claude Desktop form customizations**: 94cc3d10 restructures the form - fork-specific UI additions must be preserved
