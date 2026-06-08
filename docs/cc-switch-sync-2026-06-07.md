# cc-switch upstream sync - 2026-06-07

## Version anchors

- cc-gateway-pro before sync: `8db56659afea0cc5e3e0c6c885df8e7b7da73c14`
- cc-switch upstream reference: `5c36ae066bf70b0cd366ef6a7ad43d78c1a03aae`
- Previous sync date noted by maintainer: `2026-06-05`
- Reference directory: `/Users/demo/www/cc-switch-ref`

## Synced commits

| Upstream commit | Summary                                                         | cc-gateway-pro sync note                                                                         |
| --------------- | --------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `bda625a4`      | APINebula OpenCode preset uses OpenAI-compatible SDK            | Direct preset fix.                                                                               |
| `e96eab52`      | Update SSSAiCode domain and endpoint nodes                      | Synced URLs/endpoints; preserved Pro's Gemini default model.                                     |
| `2626eeeb`      | Normalize Windows skill path separators                         | Direct backend fix.                                                                              |
| `ab6266f7`      | Remove tray icon before Windows process exit                    | Direct Tauri lifecycle fix.                                                                      |
| `aa09c9cb`      | Normalize localhost listen address                              | Direct proxy UI fix.                                                                             |
| `5c36ae06`      | Only block explicit official providers under proxy takeover     | Synced provider switch rule; Pro keeps existing usage/OAuth card behavior.                       |
| `8e0e9ac3`      | Correct inflated Claude stream input tokens                     | Direct usage parser fix.                                                                         |
| `3cd9a0de`      | Normalize Anthropic system-role messages                        | Synced proxy normalizer and tests.                                                               |
| `2985ad2c`      | Resolve actual port for ephemeral `listen_port = 0`             | Synced runtime behavior; resolved test conflict in favor of Pro's existing fixed-port test path. |
| `ea6123ad`      | Cache reasoning across turns for Codex custom/tool-search calls | Direct Codex history fix.                                                                        |
| `6940a4b2`      | Distinguish truncated chat streams from normal completion       | Direct Codex streaming fix.                                                                      |
| `f59fab6c`      | Map `input_file` and `input_audio` Responses parts to Chat      | Direct Codex transform fix.                                                                      |
| `27c41f74`      | Add `GET /v1/models` for Codex CLI reachability                 | Direct proxy route/handler fix.                                                                  |
| `6716a4c4`      | Fix Codex VS Code session previews                              | Synced Rust session title parsing, frontend TOC preview, and tests.                              |

## Deferred or skipped commits

| Upstream commit | Summary                                                        | Decision                                                                                                                                                     |
| --------------- | -------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `473f2197`      | Official subscription quota template and opt-in tray rendering | Deferred. It changes current official provider quota behavior from automatic display to explicit opt-in. Needs separate product decision for cc-gateway-pro. |
| `03a9296c`      | Usage statistics UI polish                                     | Skipped for this pass; cosmetic and outside core sync scope.                                                                                                 |
| `1392ef62`      | README release note/sponsor markup                             | Skipped; docs-only upstream change.                                                                                                                          |

## Regression focus

- Provider switching: custom providers without explicit `category = official` must remain switchable during proxy takeover.
- Vision/model proxy path: Codex Responses to Chat transformation must preserve text, image, file, audio, and tool-call reasoning fields.
- Project/session manager: Codex VS Code injected context should show the actual last `My request for Codex` prompt in title/TOC.
- Log viewer and session traces: proxy stream completion, incomplete, and failed events must still pass through response processing and trace collection.
- Proxy takeover: `listen_port = 0` must persist the OS-assigned port and never write `:0` into live configs.
