# Maintenance priorities

Proposed priorities from the October 23, 2026 repository audit. These are open work items, not delivery promises or claims of completed work.

- [x] Fix shared replay-cache test isolation without changing the nanosecond or cache-size assertions; full local backend suite passes. Hosted CI rerun remains pending publication.
- [x] Align CI/release Node with `.node-version`, pin Rust 1.95, and pin pnpm 10.12.3. Local checks pass; hosted runner verification remains pending.
- [x] Inventory locked dependency licenses and asset provenance, collect original notices and configure Tauri notice resources.
- [x] Resolve 19 of the original 34 missing dependency notices, identify 49 file/49 inline Lobe icon matches, and verify notice resources in a local macOS debug .app.
- [ ] Resolve the 15 specifically documented notice gaps and unconfirmed brand permissions; inspect Windows/Linux production packages. See [license audit](legal/third-party-license-audit.md).
- [ ] Test clean installation and upgrade paths on macOS arm64, Windows x64 and Linux x64; record OS/runtime details.
- [ ] Keep per-release notes tied to commit comparisons; update older post-3.19.1 gaps without inventing historical entries.
- [ ] Verify Homebrew and any community packaging channels before promising availability or support.
- [ ] Gather voluntary, public user feedback through existing issue templates; distinguish external contributions from upstream ports and automated dependency PRs.

Completed in the documentation preparation: provenance clarification, contribution evidence, README download corrections, contribution-command corrections, latest-release notes and an application draft. See the [audit](jetbrains-application-audit.md) for limitations.
