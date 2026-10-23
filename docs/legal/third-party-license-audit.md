# Third-party license and asset audit

Snapshot: October 23, 2026. This is a source/dependency inventory and notice collection, not a claim that every shipped artifact or brand asset has completed legal clearance.

## Coverage results

- 1,305 locked/resolved package versions: 556 npm and 749 Cargo; all have declared license metadata.
- 1290 package versions have collected original notice text or file-level notices; 15 still require original notice collection. Missing entries are marked `review_required` in the JSON inventory.
- 653 distinct notice texts (including the Lobe Icons asset library) are included, with original copyrights retained. Supplemental source records are pinned to upstream commits.
- Eight audit-tool tests pass, including nested notice collection, lockfile parsing and stale/tampered document rejection. Inventory freshness verification passes locally.

## Scope and evidence

- [Dependency inventory](dependency-licenses.json): exact name/version/license metadata for every package entry in `pnpm-lock.yaml` and the resolved Cargo metadata graph, including build, development, optional and foreign-platform dependencies. It is a conservative superset, not an installed-binary SBOM.
- [Collected notices](../../THIRD_PARTY_NOTICES.txt): original project MIT text, package source URLs, individual notice filenames and SHA-256 references, then deduplicated verbatim notice texts. Copyright notices are retained.
- [Supplemental notices](supplemental-notices.json): notices omitted from published packages, recovered from GitHub source revisions identified by npm `gitHead` or crate `.cargo_vcs_info.json`. Each record includes its immutable source URL and covered package identities. These supplement package metadata; source provenance is recorded rather than silently substituting a generic license template.
- [Asset inventory](asset-provenance.json): 204 tracked image/icon files plus 70 inline SVG entries. Of these, 49 files and 49 inline SVGs match the integrity-verified historical Lobe Icons 1.73.0 archive byte-for-byte. [Detailed source investigation](brand-asset-sources.md) records each brand file and inline key. No tracked font files were found in the inspected asset paths.
- [Dependency follow-up](dependency-follow-up.md): all 34 original gaps have individual findings; 19 were resolved and 15 remain, with specific unavailable revisions, missing texts or broken source links.
- [Asset license notice](asset-license-notices.json): original Copyright (c) 2023 LobeHub and MIT text from the package gitHead, included in the generated notice bundle.

The tool does not execute dependency code or change lockfiles. npm metadata for packages absent on this platform is queried by exact version from registry.npmjs.org. Cargo registry packages were downloaded using `cargo metadata --locked`; the normal Cargo registry/cache integrity checks apply. No vendored dependency source or license was modified.

## License findings and distribution obligations

| Finding                                                         | Practical treatment                                                                                                                                                                                                                                                                                                         |
| --------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| MIT / ISC / BSD / Zlib / 0BSD and other permissive declarations | Retain applicable original notices; the root project's MIT label does not replace dependency licenses. Inspect compound expressions rather than dropping the second license.                                                                                                                                                |
| Apache-2.0 and LLVM exception combinations                      | Preserve included license and NOTICE texts; record modifications when applicable. See [Apache section 4](https://www.apache.org/licenses/LICENSE-2.0). The collection includes nested notices found in package sources.                                                                                                     |
| Seven MPL-2.0 crates                                            | `cssparser` 0.29.6 and 0.36.0; `cssparser-macros` 0.6.1; `dtoa-short` 0.3.5; `option-ext` 0.2.0; `selectors` 0.24.0 and 0.35.0. The notices index supplies exact-version source archive URLs and retains available MPL texts. No local modifications were made to these crate sources.                                      |
| MPL executable distribution                                     | Recipients must be told where to obtain the MPL-covered source; inclusion does not by itself require relicensing every independent project file. See [Mozilla's FAQ, questions 8–11](https://www.mozilla.org/en-US/MPL/2.0/FAQ/). Recheck any future local patches and ship their source as required.                       |
| `r-efi` license alternatives                                    | Metadata declares `MIT OR Apache-2.0 OR LGPL-2.1-or-later`; this is an alternative-license expression, not evidence that LGPL is the sole mandatory choice. Preserve the expression and applicable selected-license notices.                                                                                                |
| Unicode-3.0 / CDLA-Permissive-2.0 / CC-BY-4.0                   | Keep their distinct data and attribution conditions. `caniuse-lite` declares CC-BY-4.0; it is build-tool data, not grounds to relabel all application code.                                                                                                                                                                 |
| Compound ISC/OpenSSL and BSD/MIT expressions                    | Collected raw license metadata and nested notices remain authoritative inputs; this audit does not flatten `AND` into `OR`.                                                                                                                                                                                                 |
| Provider logos and screenshots                                  | 49 SVG files and 49 inline SVGs have a byte-exact Lobe library source and collected MIT notice. The other 51 brand files and 21 inline entries have only repository history evidence; specific reasons are listed in the brand-source report. Library copyright terms do not establish separate brand/trademark permission. |

## What has been implemented

The Tauri resource configuration includes the root LICENSE, THIRD_PARTY_NOTICES.txt, dependency inventory and this report in future bundles. CI checks that the inventory's input hashes match the lockfiles, original LICENSE and supplemental sources, and that the notices file matches its recorded hash. This detects stale/tampered generated documents; it does not certify legal compliance.

No existing release asset has been replaced. A local macOS debug .app was built with ad-hoc signing, updater artifact creation disabled and notarization credentials removed. Its configured license resources were compared by SHA-256 with the workspace files. It was not installed or published. Windows/Linux packages and production signing remain unverified; this does not retroactively fix already published binaries.

## Reproduce and update

Use the pinned pnpm version and Python 3.11+ (stdlib `tomllib`):

```bash
npx --yes pnpm@10.12.3 install --frozen-lockfile
python3 scripts/audit-third-party-licenses.py
python3 scripts/audit-third-party-licenses.py --check
python3 -m unittest discover -s scripts -p test_license_audit.py
```

`--offline` omits npm registry lookups. If metadata is missing, generation fails before overwriting existing documents. To reuse previously reviewed npm metadata, combine `--offline --reuse-npm-metadata`; the tool requires an identical pnpm lockfile hash and never copies notices from unrelated versions. Supplied Cargo metadata must match all registry name/version/source identities in Cargo.lock. The latest regeneration reused the unchanged npm lock snapshot after transient registry timeouts; dependency versions were not changed. To reuse Cargo metadata: `cargo metadata --locked --format-version 1 --manifest-path src-tauri/Cargo.toml > /tmp/cargo-metadata.json`, then pass `--cargo-metadata /tmp/cargo-metadata.json`.

`review_required` means the declared license or collected notice text is missing; false only means those inventory fields are present, not legal approval. Review newly introduced license expressions, changed versions, missing text, copied assets and platform packaging whenever dependencies change. Supplemental notices must be rechecked at the source revision for new versions.

## Remaining maintainer/release checks

- Obtain missing original notice texts identified in the machine-readable inventory; some packages publish only a license identifier or omit source-revision metadata. Do not fabricate a copyright holder to fill the gap.
- Confirm the actual distribution graph and system-library notices for each target; audit local patches if any are introduced.
- Establish the source and usage permission for each brand asset, including whether brand guidelines permit redistribution. The maintainer may have permission evidence not stored in Git.
- Inspect Windows/Linux installers and production releases for notice resources and source availability. Local macOS debug bundle verification is complete; no release package was published or installed.
