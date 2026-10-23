# Project history and attribution

CC-Gateway-Pro builds on [farion1231/cc-switch](https://github.com/farion1231/cc-switch). Its gateway, provider management, failover, usage analytics, and substantial UI/backend infrastructure have upstream origins. They are not claimed as original CC-Gateway-Pro inventions.

GitHub reports this repository as a standalone repository (`fork: false`). Its root commit, [0f69cb085](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb085), dated May 18, 2026 in Asia/Shanghai, explicitly imports cc-switch v3.15.0. This is a new Git history containing imported code, not evidence that the imported code was written by this repository's maintainer. The exact upstream snapshot identity has not been independently established.

[KeaneFeng](https://github.com/KeaneFeng) maintains this repository. Project-specific development includes project provider binding, Vision Model routing, terminal request-log viewing, and Session Traces improvements. See the [contribution evidence](original-contributions.md). Upstream ports continue: the October 2026 history retains upstream authors and cherry-pick references. The [v4.0.17 notes](release-notes/v4.0.17-en.md) distinguish those fixes from original additions.

## License and third-party materials

The root [MIT license](../LICENSE) retains `Copyright (c) 2025 Jason Young`, including the permission notice and warranty disclaimer. The imported root commit and current LICENSE have identical SHA-256 hashes. This notice must remain with copies or substantial portions of the software. Maintainer identity does not replace upstream copyright.

The local upstream reference also contains that MIT notice; GitHub's upstream license API identifies MIT. The initial audit found no tracked NOTICE file; the project now includes THIRD_PARTY_NOTICES.txt. This does not establish that every dependency, icon, font, asset, or distribution bundle is covered solely by MIT. Dependency metadata and original notices have now been inventoried against the lockfiles; see the [license audit](legal/third-party-license-audit.md). 11 missing original notices, individual brand permissions and Windows/Linux release resources still require verification before making a comprehensive compliance claim.

No license replacement, copyright removal, or Git history rewrite was performed for the application preparation. This document records provenance evidence; it is not a completed third-party license clearance.
