# Maintenance priorities

Proposed priorities from the October 23, 2026 repository audit. These are open work items, not delivery promises or claims of completed work.

- [x] Fix shared replay-cache test isolation without changing the nanosecond or cache-size assertions; local validation and [hosted CI](https://github.com/KeaneFeng/cc-gateway-pro/actions/runs/39893831322) pass (609 frontend tests; 2644 Linux backend tests, 3 ignored).
- [x] Align CI/release Node with `.node-version`, pin Rust 1.95, and pin pnpm 10.12.3. CI runner verification passes. The changed release workflow still awaits a new production release run.
- [x] Inventory locked dependency licenses and asset provenance, collect original notices and configure Tauri notice resources.
- [x] Resolve 19 of the original 34 missing dependency notices, identify 49 file/49 inline Lobe icon matches, and verify notice resources in a local macOS debug .app.
- [x] Enable GitHub private vulnerability reporting with maintainer authorization; verify `enabled: true` and the public reporting button. No report was submitted; delivery remains untested.
- [ ] Resolve the 15 specifically documented notice gaps and unconfirmed brand permissions; inspect Windows/Linux production packages. See [license audit](legal/third-party-license-audit.md).
- [ ] Test clean installation and upgrade paths on macOS arm64, Windows x64 and Linux x64; record OS/runtime details.
- [x] Reconstruct all 13 published release summaries after v3.19.1 with dated comparisons and revert exclusions; see [history](release-notes/published-history.md). Online release bodies remain unchanged.
- [x] Verify Homebrew tap/cask existence, v4.0.17 arm64 URL and digest against the release API.
- [ ] Perform actual Homebrew installation/upgrade and platform acceptance; static channel verification is not an installation test.
- [ ] Gather voluntary, public user feedback through existing issue templates; distinguish external contributions from upstream ports and automated dependency PRs.

Completed in the documentation preparation: provenance clarification, contribution evidence, README download corrections, contribution-command corrections, latest-release notes and an application draft. See the [audit](jetbrains-application-audit.md) for limitations.
