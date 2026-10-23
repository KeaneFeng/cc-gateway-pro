# JetBrains open-source support application — draft

Prepared October 23, 2026. Not submitted. The maintainer must confirm personal statements, intended IDE use, funding/commercial context and the live application fields before submission. Statistics are dated snapshots; refresh them before applying. See the [audit](jetbrains-application-audit.md) for known gaps.

## 1. Project Name / Repository / License

- **Name:** CC-Gateway-Pro
- **Repository:** https://github.com/KeaneFeng/cc-gateway-pro
- **License:** MIT, retaining Copyright (c) 2025 Jason Young; [license](../LICENSE).

## 2. Project Overview

CC-Gateway-Pro is an open-source desktop control plane and local provider gateway for AI coding tools. Built with Tauri, Rust, React and TypeScript, it manages provider configurations and routes requests through a local proxy. It builds on cc-switch and retains upstream infrastructure for provider management, protocol adaptation, failover and usage analytics. Project-specific work includes project-level provider binding, image-aware Vision Model routing, terminal request-log viewing and local Session Traces improvements. These capabilities help developers inspect requests and choose providers for different projects while keeping operational data locally. The repository publishes packages for macOS, Windows and Linux and documents its architecture and development workflow. It continues to integrate upstream fixes while maintaining its own routing and observability extensions.

## 3. Maintainer Role

The repository owner is KeaneFeng. GitHub attributes 190 commits to this account at the audited revision. Public history records project routing and vision routing development, logging improvements, upstream integration and version maintenance. This count includes imports and maintenance work; it is not a claim of 190 independent inventions. The visible project history begins on May 18, 2026. The applicant should confirm that this describes their own role before using it in the first person.

## 4. Original Contributions

- Project-level provider routing: session-to-directory lookup and provider bindings ([f93b21eb8](https://github.com/KeaneFeng/cc-gateway-pro/commit/f93b21eb8), [Codex routing PR](https://github.com/KeaneFeng/cc-gateway-pro/pull/3)).
- Vision Model routing: image-aware selection and synchronization of the forwarded request model ([initial change](https://github.com/KeaneFeng/cc-gateway-pro/commit/f93b21eb8), [module extraction](https://github.com/KeaneFeng/cc-gateway-pro/commit/84b0552df)).
- Daily proxy text logs and preferred-terminal viewing ([123624bf9](https://github.com/KeaneFeng/cc-gateway-pro/commit/123624bf9)); the underlying usage analytics is inherited.
- Session Traces improvements: bounded payload storage, retention/pruning controls and bounded queries ([672e1e1d8](https://github.com/KeaneFeng/cc-gateway-pro/commit/672e1e1d8)).

See [the contribution matrix](original-contributions.md) for implementation paths and scope. The core gateway and failover are not presented as original work.

## 5. Project Activity & Community Impact

As of October 23, 2026, the repository is public, with 8 stars, 3 forks and 43 non-draft releases. The latest release is v4.0.17, published October 20. Public release assets total 529 download events; installer/archive assets account for 233 of those events. These are not unique users or successful installations. One external proposal, [PR #8](https://github.com/KeaneFeng/cc-gateway-pro/pull/8), was closed without merging. The 49 GitHub contributor accounts include upstream authors and automation, so they are not evidence of 49 direct community contributors. Broad ecosystem adoption has not been established.

The v4.0.17 packaging workflow succeeded. Its separate CI run passed frontend checks but failed one backend test; the [audit](jetbrains-application-audit.md) records the historical failure and the subsequent local test-isolation fix. Local validation now passes; the fix has not been committed or run on hosted CI. No claim of a new successful public CI run is made.

## 6. Why JetBrains All Products Pack

Proposed use, subject to maintainer confirmation: RustRover for Rust proxy, persistence and test development; WebStorm for the React/TypeScript renderer and frontend tests; DataGrip for inspecting SQLite schemas and migrations using sanitized local fixtures. A multi-IDE workflow would support changes spanning native request processing, database state and the desktop UI. These are intended uses based on the repository's actual stack, not claims about existing subscriptions or IDE usage.

## 7. Upstream Attribution

CC-Gateway-Pro builds on cc-switch. It imported an upstream code snapshot into a standalone repository and continues to integrate upstream changes. The original MIT copyright and permission notice remain in LICENSE. Project-specific extensions do not erase upstream authorship. See [project history](project-history.md). A lockfile-based dependency inventory and notice collection is available in the [license audit](legal/third-party-license-audit.md); remaining notice gaps, asset permissions and final distribution checks are explicitly recorded.

## 8. Links to Evidence

- [Source and history](https://github.com/KeaneFeng/cc-gateway-pro/commits/main/)
- [Published v4.0.17](https://github.com/KeaneFeng/cc-gateway-pro/releases/tag/v4.0.17)
- [Successful release build](https://github.com/KeaneFeng/cc-gateway-pro/actions/runs/39409186505)
- [CI results, including outstanding failure](https://github.com/KeaneFeng/cc-gateway-pro/actions/runs/39408533954)
- [Architecture](architecture-and-flows-zh.md), [contributing](../CONTRIBUTING.md), [security](../SECURITY.md)
- [Official program information](https://www.jetbrains.com/community/opensource/)

New local documents will only become public evidence after the maintainer publishes them. Eligibility and acceptance remain subject to JetBrains review.
