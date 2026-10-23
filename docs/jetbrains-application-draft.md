# JetBrains open-source support application — draft

Updated October 23, 2026 against public revision `24a6275e853a91a67623f90c01fa94c541e6dd55`. Not submitted. The applicant has confirmed the account KeaneFeng and requested six All Products Pack subscriptions, stating that six people participate. The complete recipient list and evidence of each recipient’s regular contributions remain unverified. Non-commercial use, personal contact fields, funding/employment context and intended IDE use still require confirmation. See the [final checklist](jetbrains-final-checklist.md).

## 1. Project Name / Repository / License

- **Name:** CC-Gateway-Pro
- **Repository:** https://github.com/KeaneFeng/cc-gateway-pro
- **License:** MIT, retaining Copyright (c) 2025 Jason Young; [license](../LICENSE).

## 2. Project Overview

CC-Gateway-Pro is an open-source desktop control plane and local provider gateway for AI coding tools. Built with Tauri, Rust, React and TypeScript, it manages provider configurations and routes requests through a local proxy. It builds on cc-switch and retains upstream infrastructure for provider management, protocol adaptation, failover and usage analytics. Project-specific work includes project-level provider binding, image-aware Vision Model routing, terminal request-log viewing and local Session Traces improvements. These capabilities help developers inspect requests and choose providers for different projects while keeping operational data locally. The repository publishes packages for macOS, Windows and Linux and documents its architecture and development workflow. It continues to integrate upstream fixes while maintaining its own routing and observability extensions.

## 3. Maintainer Role

The repository owner is KeaneFeng. GitHub attributes 193 commits to this account at the audited revision. Public history records project routing and vision routing development, logging improvements, upstream integration and version maintenance. This count includes imports and maintenance work; it is not a claim of 193 independent inventions. The visible project history begins on May 18, 2026. The applicant has confirmed that they are KeaneFeng; the public history supports this maintenance role. This does not verify the identities or eligibility of the other five proposed recipients.

## 4. Original Contributions

- Project-level provider routing: session-to-directory lookup and provider bindings ([f93b21eb8](https://github.com/KeaneFeng/cc-gateway-pro/commit/f93b21eb8), [Codex routing PR](https://github.com/KeaneFeng/cc-gateway-pro/pull/3)).
- Vision Model routing: image-aware selection and synchronization of the forwarded request model ([initial change](https://github.com/KeaneFeng/cc-gateway-pro/commit/f93b21eb8), [module extraction](https://github.com/KeaneFeng/cc-gateway-pro/commit/84b0552df)).
- Daily proxy text logs and preferred-terminal viewing ([123624bf9](https://github.com/KeaneFeng/cc-gateway-pro/commit/123624bf9)); the underlying usage analytics is inherited.
- Session Traces improvements: bounded payload storage, retention/pruning controls and bounded queries ([672e1e1d8](https://github.com/KeaneFeng/cc-gateway-pro/commit/672e1e1d8)).

See [the contribution matrix](original-contributions.md) for implementation paths and scope. The core gateway and failover are not presented as original work.

## 5. Project Activity & Community Impact

As of October 23, 2026, the repository is public, with 8 stars, 3 forks and 43 non-draft releases. The latest release is v4.0.17, published October 20. Public release assets total 529 download events; installer/archive assets account for 233 of those events. These are not unique users or successful installations. One external proposal, [PR #8](https://github.com/KeaneFeng/cc-gateway-pro/pull/8), was closed without merging. The 49 GitHub contributor accounts include upstream authors and automation, so they are not evidence of 49 direct community contributors. Broad ecosystem adoption has not been established.

The v4.0.17 packaging workflow succeeded. Its historical backend CI failure was addressed by [shared-cache test isolation](https://github.com/KeaneFeng/cc-gateway-pro/commit/19de2600b). [Hosted CI on 24a6275e8](https://github.com/KeaneFeng/cc-gateway-pro/actions/runs/39893831322) now passes: 609 frontend tests and 2644 Linux backend tests, with zero failures and three ignored backend tests. Type checking, formatting, Clippy and license-inventory freshness checks also pass. These maintenance changes are on main after v4.0.17; no updated release package has been published. Documentation-only commits are not offered as evidence of active code development.

## 6. Why JetBrains All Products Pack

Proposed use, subject to maintainer confirmation: RustRover for Rust proxy, persistence and test development; WebStorm for the React/TypeScript renderer and frontend tests; DataGrip for inspecting SQLite schemas and migrations using sanitized local fixtures. A multi-IDE workflow would support changes spanning native request processing, database state and the desktop UI. These are intended uses based on the repository's actual stack, not claims about existing subscriptions or IDE usage.

## 7. Upstream Attribution

CC-Gateway-Pro builds on cc-switch. It imported an upstream code snapshot into a standalone repository and continues to integrate upstream changes. The original MIT copyright and permission notice remain in LICENSE. Project-specific extensions do not erase upstream authorship. See [project history](project-history.md). A lockfile-based dependency inventory and notice collection is available in the [license audit](legal/third-party-license-audit.md); remaining notice gaps, asset permissions and final distribution checks are explicitly recorded.

## 8. Links to Evidence

- [Source and history](https://github.com/KeaneFeng/cc-gateway-pro/commits/main/)
- [Published v4.0.17](https://github.com/KeaneFeng/cc-gateway-pro/releases/tag/v4.0.17)
- [Successful release build](https://github.com/KeaneFeng/cc-gateway-pro/actions/runs/39409186505)
- [Successful current CI](https://github.com/KeaneFeng/cc-gateway-pro/actions/runs/39893831322)
- [Historical v4.0.17 CI failure](https://github.com/KeaneFeng/cc-gateway-pro/actions/runs/39408533954), retained for transparency
- [Reconstructed published release history](release-notes/published-history.md)
- [Architecture](architecture-and-flows-zh.md), [contributing](../CONTRIBUTING.md), [security](../SECURITY.md)
- [Official program information](https://www.jetbrains.com/community/opensource/)

The contribution matrix, provenance documents and license evidence are public at `24a6275e8`. This final revision of the application draft is local and has not been submitted. Eligibility and acceptance remain subject to JetBrains review.

## 9. Mapping to the current application form

Checked against the [official form](https://www.jetbrains.com/shop/eform/opensource) on October 23, 2026. This is a local preparation table; nothing has been entered into or submitted through the form.

| Form field                            | Prepared value / unresolved item                                                               |
| ------------------------------------- | ---------------------------------------------------------------------------------------------- |
| Previous JetBrains OSS licenses       | Applicant to confirm; do not assume this is a first application                                |
| Project website                       | https://github.com/KeaneFeng/cc-gateway-pro (public project page; no separate website claimed) |
| Repository URL                        | https://github.com/KeaneFeng/cc-gateway-pro                                                    |
| License URL                           | https://github.com/KeaneFeng/cc-gateway-pro/blob/main/LICENSE                                  |
| Country / region                      | Applicant to supply; not inferred from language or timezone                                    |
| Number of required licenses           | **6 requested by applicant; recipient eligibility not yet established**                        |
| Project description                   | Sections 2–7 above, or the compact draft below                                                 |
| Email / first and last name           | Applicant to supply privately when applying; not inferred from Git author metadata             |
| Contributor profile                   | https://github.com/KeaneFeng                                                                   |
| Non-commercial OSS use confirmation   | Pending personal confirmation; do not pre-check                                                |
| Sharing only with active contributors | Pending six-person recipient/evidence list; upstream author attribution is insufficient        |
| Account agreement / other consent     | Applicant must review and accept personally                                                    |

The form restricts seats to regularly active contributors and excludes non-code commits as active development evidence. Six subscriptions mean six recipients, not six IDEs for one person. The applicant’s stated team size is retained without treating the repository’s contributor count as proof.

## 10. Compact project-description draft

CC-Gateway-Pro is a public MIT-licensed desktop application for managing AI coding-tool providers and routing requests through a local gateway. It uses Tauri/Rust and React/TypeScript, with SQLite persistence. It builds on cc-switch and retains upstream attribution. KeaneFeng maintains project-specific provider binding, image-aware Vision Model routing, terminal request-log viewing and Session Traces improvements; linked commits in the contribution matrix document this work. Published packages cover macOS arm64, Windows x64 and Linux x64. Current hosted CI passes frontend and backend checks. The repository has 8 stars and 3 forks as of October 23, 2026; broad adoption is not claimed. All Products Pack would support work across the native backend, renderer and database. Six subscriptions are requested, subject to documenting each recipient’s regular contributions and confirming non-commercial use. Dependency-notice and asset-permission gaps remain disclosed in the license audit.

## 11. Items to resolve before submitting

- Confirm all six recipients, their profiles and regular project-specific code contribution links. Only KeaneFeng is presently identified as an applicant; do not add upstream authors without their actual participation and consent.
- Confirm non-commercial OSS usage, intended tools, funding/employment context, prior OSS licenses and contact fields. GitHub evidence cannot answer these personal declarations.
- Review the disclosed 15 notice gaps, unconfirmed asset permissions and missing platform installation/distribution checks; do not claim full clearance.
- Private vulnerability reporting has been enabled and verified through the repository API and public Security page. End-to-end report delivery has not been tested; no synthetic report was submitted.
- Refresh metrics and apply the live form’s final wording. The official program evaluates applications individually; this material does not establish approval or a guaranteed eligibility threshold.
