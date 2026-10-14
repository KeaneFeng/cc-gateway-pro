<div align="center">

# CC-Gateway-Pro

### Lokal AI-providergateway för Claude Code, Claude Desktop, Codex, Gemini CLI, OpenCode, OpenClaw och Hermes

[![Version](https://img.shields.io/github/v/release/KeaneFeng/cc-gateway-pro?color=blue&label=version)](https://github.com/KeaneFeng/cc-gateway-pro/releases)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/KeaneFeng/cc-gateway-pro/releases)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-orange.svg)](https://tauri.app/)
[![Fork of cc-switch](https://img.shields.io/badge/fork%20of-farion1231%2Fcc--switch-blue.svg)](https://github.com/farion1231/cc-switch)

[English](README.md) | [中文](README_ZH.md) | [日本語](README_JA.md) | Svenska | [Español](README_ES.md) | [Ändringslogg](CHANGELOG.md)

</div>

## Vad är CC-Gateway-Pro?

CC-Gateway-Pro är en skrivbordsbaserad kontrollpanel och lokal AI-providergateway för AI-verktyg för kodning. Den hanterar providerprofiler, skriver varje verktygs inbyggda konfigurationsfiler och kan dirigera aktiva förfrågningar genom en lokal proxy för förfrågningsloggning, sessionsspårningar, redundansväxling, modelltransformering, providerbindning på projektnivå och Vision Model-routning.

Projektet är en fork av [farion1231/cc-switch](https://github.com/farion1231/cc-switch). Det behåller den ursprungliga idén med visuellt providersbyte och bygger sedan ut den med en lokal gateway i Rust/Tauri, konfigurationshantering för flera appar, förfrågningsroutning, användningsanalys, synkronisering och providerspecifika integrationer.

## Appar som stöds

| App            | Huvudsakliga funktioner                                                                                                                    |
| -------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| Claude Code    | Providersbyte, övertagande via lokal proxy, projektroutning, Vision Model-routning, MCP, prompts, skills, sessioner                       |
| Claude Desktop | Officiella och tredjepartsprofiler, direktläge, lokalt dirigerat läge, mappning av modellrutter                                             |
| Codex          | Providersbyte, övertagande via lokal proxy, projektroutning, OAuth-/Copilot-hjälp, MCP, prompts, skills, sessioner                         |
| Gemini CLI     | Providersbyte, övertagande via lokal proxy, MCP, prompts, skills, sessioner                                                                 |
| Grok Build     | Providersbyte, övertagande via lokal proxy, kvoter, redundansväxling, MCP, prompts, skills, sessioner                                      |
| OpenCode       | Providerförinställningar, vanliga konfigurationsutdrag, MCP, prompts, skills, sessioner                                                     |
| OpenClaw       | Providerförinställningar, arbetsytans filer, agentstandarder, paneler för verktyg och miljövariabler                                        |
| Hermes Agent   | Providerförinställningar, minnespanel, hantering av MCP och skills                                                                          |

## Huvudfunktioner

- **Providerhantering**: över 50 förinställningar, anpassade providers, sortering med dra och släpp, import/export, hastighetstester för endpoints, hjälpmedel för saldo och kvoter.
- **Lokal proxygateway**: Tauri/Rust + Axum-proxy på `127.0.0.1:15721` som standard, med övertagande per app för Claude, Claude Desktop, Codex och Gemini.
- **Anpassning av API-format**: Anthropic, OpenAI Chat Completions, OpenAI Responses, Gemini och providerspecifika kompatibilitetsvägar.
- **Routning på projektnivå**: sessioner i Claude och Codex kan mappas från lokala sessionsfiler till projektkataloger och sedan dirigeras till projektbundna providers.
- **Vision Model-routning**: förfrågningar som innehåller bildblock kan automatiskt växla till providerns konfigurerade `vision_model`.
- **Hög tillgänglighet**: köer för redundansväxling per app, kretsbrytare, återförsök, hälsostatus samt styrning av tidsgränser för strömmade och icke-strömmade svar.
- **Proxyförfrågningsloggar**: visa de senaste sju dagarnas proxyförfrågningsloggar i önskad terminal med rullning i realtid.
- **Användningsanalys**: förfrågningsloggar, tokenanvändning, kostnadsuppskattning, trender, uppdelning per provider/modell och anpassad prissättning.
- **Session Traces**: valfri lokal kontextanalys för nya förfrågningar, med Summary-insamling för utdragna fält och Full-insamling för maskerad JSON för förfrågan/svar.
- **MCP, prompts och skills**: enhetlig hantering, synkronisering mellan appar som stöds, deep links samt installation av skills från repository eller ZIP.
- **Datasäkerhet**: SQLite-lagring, atomiska konfigurationsskrivningar, automatiska säkerhetskopior, konfigurerbar lagringskatalog och WebDAV-synkronisering.

## Arkitekturdiagram

### Lokal AI-providergateway

![Arkitektur för lokal AI-providergateway](docs/assets/diagrams/cc-gateway-pro-architecture-zh.svg)

### Arbetsflöde för Vision Model-proxyn

![Arbetsflöde för Vision Model-proxyn](docs/assets/diagrams/vision-model-proxy-flow-zh.svg)

### Arbetsflöde för Project Provider-proxyn

![Arbetsflöde för Project Provider-proxyn](docs/assets/diagrams/project-provider-proxy-flow-zh.svg)

Fler diagram för redundansväxling, användningsloggning, synkronisering av MCP/Prompts/Skills och hela förfrågningskedjan finns i [Arkitektur och flöden](docs/architecture-and-flows-zh.md).

## Förhandsvisning av gränssnittet

|                 Huvudskärm                 |                 Lägg till provider                  |
| :----------------------------------------: | :-------------------------------------------------: |
| ![Huvudskärm](assets/screenshots/main-en.png) | ![Lägg till provider](assets/screenshots/add-en.png) |

|                            Projektroutning                            |                          Konfiguration av Vision Model                          |
| :-------------------------------------------------------------------: | :---------------------------------------------------------------------------: |
| ![Förhandsvisning av projektroutning](assets/screenshots/project-routing-preview-zh.svg) | ![Förhandsvisning av Vision Model-konfiguration](assets/screenshots/vision-model-config-preview-zh.svg) |

|                          Inställningar för Session Traces                          |                          Analys av Session Traces                          |
| :-------------------------------------------------------------------------------: | :-----------------------------------------------------------------------: |
| ![Förhandsvisning av Session Traces-inställningar](assets/screenshots/session-traces-settings-preview.svg) | ![Förhandsvisning av Session Traces-analys](assets/screenshots/session-traces-analysis-preview.svg) |

## Installation

Hämta den senaste versionen från [GitHub Releases](https://github.com/KeaneFeng/cc-gateway-pro/releases).

Systemkrav:

- macOS 12 eller senare
- Windows 10 eller senare
- Linux-distributioner med en modern WebKitGTK-runtime, till exempel Ubuntu 22.04+, Debian 11+ eller Fedora 34+

### macOS Homebrew

Många användare behöver uppdatera Homebrew-metadata innan casken kan hittas eller uppdateras korrekt:

```bash
brew update
brew tap KeaneFeng/cc-gateway-pro
brew install --cask cc-gateway-pro
```

Uppgradering:

```bash
brew update
brew upgrade --cask cc-gateway-pro
```

### Manuell nedladdning

- Windows: ladda ner `.msi`-installationsprogrammet eller den portabla `.zip`-filen från [Releases](https://github.com/KeaneFeng/cc-gateway-pro/releases).
- macOS: ladda ner `.dmg` från [Releases](https://github.com/KeaneFeng/cc-gateway-pro/releases).
- Linux: ladda ner `.deb`, `.rpm` eller `.AppImage` från [Releases](https://github.com/KeaneFeng/cc-gateway-pro/releases).
- Arch Linux: installera med `paru -S cc-gateway-pro-bin`.

## Bygg från källkod

Förutsättningar: Node.js 22+, pnpm och Rust stable.

```bash
pnpm install
pnpm tauri dev
pnpm tauri build
```

Repositoriet innehåller även ett hjälpskript:

```bash
./build.sh --dev
./build.sh --release
./build.sh --dmg
./build.sh --sha
```

## Dokumentation

- [Engelsk README](README.md)
- [Kinesisk README](README_ZH.md)
- [Japansk README](README_JA.md)
- [Spansk README](README_ES.md)
- [Användarmanual](docs/user-manual/zh/README.md)
- [Arkitektur och flöden](docs/architecture-and-flows-zh.md)
- [Proxyguide](docs/proxy-guide-zh.md)
- [Versionsinformation](docs/release-notes/v3.19.1-en.md)

## Datalagringsplatser

- Huvuddatabas: `~/.cc-gateway-pro/cc-gateway-pro.db`
- Enhetsinställningar: `~/.cc-gateway-pro/settings.json`
- Säkerhetskopior: `~/.cc-gateway-pro/backups/`
- Skills: `~/.cc-gateway-pro/skills/`

Lagringskatalogen kan flyttas till Dropbox, OneDrive, iCloud, NAS eller en annan synkroniserad mapp från inställningssidan.

## Erkännanden

- Forkad från [farion1231/cc-switch](https://github.com/farion1231/cc-switch)
- Byggd med Tauri, Rust, React, TypeScript och Tailwind CSS

## Licens

MIT © Keane Feng
