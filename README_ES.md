<div align="center">

# CC-Gateway-Pro

### Pasarela local de proveedores de IA para Claude Code, Claude Desktop, Codex, Gemini CLI, OpenCode, OpenClaw y Hermes

[![Version](https://img.shields.io/github/v/release/KeaneFeng/cc-gateway-pro?color=blue&label=version)](https://github.com/KeaneFeng/cc-gateway-pro/releases)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/KeaneFeng/cc-gateway-pro/releases)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-orange.svg)](https://tauri.app/)
[![Fork of cc-switch](https://img.shields.io/badge/fork%20of-farion1231%2Fcc--switch-blue.svg)](https://github.com/farion1231/cc-switch)

[English](README.md) | [中文](README_ZH.md) | [日本語](README_JA.md) | [Svenska](README_SV.md) | Español | [Changelog](CHANGELOG.md)

</div>

## ¿Qué es CC-Gateway-Pro?

CC-Gateway-Pro es un panel de control de escritorio y una pasarela local de proveedores de IA para herramientas de programación asistida por IA. Gestiona perfiles de proveedores, escribe los archivos de configuración nativos de cada herramienta y puede enrutar las solicitudes en vivo a través de un proxy local para registrar solicitudes, generar trazas de sesión, aplicar conmutación por error, transformar modelos, vincular proveedores a nivel de proyecto y enrutar el Vision Model.

Este proyecto es un fork de [farion1231/cc-switch](https://github.com/farion1231/cc-switch). Conserva la idea original del cambio visual de proveedores y la amplía con una pasarela local en Rust/Tauri, gestión de configuración de múltiples aplicaciones, enrutado de solicitudes, análisis de uso, sincronización e integraciones específicas de cada proveedor.

## Aplicaciones compatibles

| Aplicación     | Funciones principales                                                                                                                           |
| -------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| Claude Code    | Cambio de proveedor, toma de control mediante proxy local, enrutado por proyecto, enrutado de Vision Model, MCP, prompts, skills, sesiones       |
| Claude Desktop | Perfiles oficiales y de terceros, modo directo, modo enrutado local, mapeo de rutas de modelos                                                  |
| Codex          | Cambio de proveedor, toma de control mediante proxy local, enrutado por proyecto, asistentes de OAuth/Copilot, MCP, prompts, skills, sesiones    |
| Gemini CLI     | Cambio de proveedor, toma de control mediante proxy local, MCP, prompts, skills, sesiones                                                        |
| Grok Build     | Cambio de proveedor, toma de control mediante proxy local, cuotas, conmutación por error, MCP, prompts, skills, sesiones                         |
| OpenCode       | Preajustes de proveedores, fragmentos de configuración comunes, MCP, prompts, skills, sesiones                                                   |
| OpenClaw       | Preajustes de proveedores, archivos del espacio de trabajo, valores por defecto del agente, paneles de herramientas y variables de entorno       |
| Hermes Agent   | Preajustes de proveedores, panel de memoria, gestión de MCP y skills                                                                            |

## Funciones principales

- **Gestión de proveedores**: más de 50 preajustes, proveedores personalizados, ordenación por arrastrar y soltar, importación/exportación, pruebas de velocidad de endpoints y utilidades de saldo y cuota.
- **Pasarela proxy local**: proxy en Tauri/Rust + Axum en `127.0.0.1:15721` por defecto, con toma de control por aplicación para Claude, Claude Desktop, Codex y Gemini.
- **Adaptación de formato de API**: Anthropic, OpenAI Chat Completions, OpenAI Responses, Gemini y rutas de compatibilidad específicas de cada proveedor.
- **Enrutado a nivel de proyecto**: las sesiones de Claude y Codex pueden mapearse desde archivos de sesión locales a directorios de proyecto y, a continuación, enrutarse a los proveedores vinculados a cada proyecto.
- **Enrutado de Vision Model**: las solicitudes que contienen bloques de imagen pueden cambiar automáticamente al `vision_model` configurado en el proveedor.
- **Alta disponibilidad**: colas de conmutación por error por aplicación, disyuntores, reintentos, estado de salud y control de tiempos de espera para respuestas en streaming y sin streaming.
- **Registros de solicitudes del proxy**: consulta los registros de solicitudes del proxy de los últimos 7 días en tu terminal preferida con desplazamiento en tiempo real.
- **Análisis de uso**: registros de solicitudes, consumo de tokens, estimación de costes, tendencias, desgloses por proveedor y por modelo, y precios personalizados.
- **Session Traces**: análisis de contexto local opcional para solicitudes nuevas, con captura Summary para los campos extraídos y captura Full para el JSON anonimizado de solicitud y respuesta.
- **MCP, prompts y skills**: gestión unificada, sincronización entre las aplicaciones compatibles, deep links e instalación de skills desde repositorios o archivos ZIP.
- **Seguridad de los datos**: almacenamiento en SQLite, escrituras atómicas de configuración, copias de seguridad automáticas, directorio de almacenamiento configurable y sincronización WebDAV.

## Diagramas de arquitectura

### Pasarela local de proveedores de IA

![Arquitectura de la pasarela local de proveedores de IA](docs/assets/diagrams/cc-gateway-pro-architecture-zh.svg)

### Flujo de trabajo del proxy de Vision Model

![Flujo de trabajo del proxy de Vision Model](docs/assets/diagrams/vision-model-proxy-flow-zh.svg)

### Flujo de trabajo del proxy de Project Provider

![Flujo de trabajo del proxy de Project Provider](docs/assets/diagrams/project-provider-proxy-flow-zh.svg)

Hay más diagramas sobre la conmutación por error, el registro de uso, la sincronización de MCP/Prompts/Skills y la cadena completa de solicitudes en [Arquitectura y flujos](docs/architecture-and-flows-zh.md).

## Vista previa de la interfaz

|                  Pantalla principal                  |                  Añadir proveedor                  |
| :--------------------------------------------------: | :-----------------------------------------------: |
| ![Pantalla principal](assets/screenshots/main-en.png) | ![Añadir proveedor](assets/screenshots/add-en.png) |

|                            Enrutado por proyecto                            |                       Configuración de Vision Model                        |
| :------------------------------------------------------------------------: | :------------------------------------------------------------------------: |
| ![Vista previa del enrutado por proyecto](assets/screenshots/project-routing-preview-zh.svg) | ![Vista previa de la configuración de Vision Model](assets/screenshots/vision-model-config-preview-zh.svg) |

|                       Ajustes de Session Traces                        |                        Análisis de Session Traces                         |
| :--------------------------------------------------------------------: | :-----------------------------------------------------------------------: |
| ![Vista previa de los ajustes de Session Traces](assets/screenshots/session-traces-settings-preview.svg) | ![Vista previa del análisis de Session Traces](assets/screenshots/session-traces-analysis-preview.svg) |

## Instalación

Descarga la última compilación desde [GitHub Releases](https://github.com/KeaneFeng/cc-gateway-pro/releases).

Requisitos del sistema:

- macOS 12 o posterior
- Windows 10 o posterior
- Distribuciones de Linux con un runtime WebKitGTK moderno, como Ubuntu 22.04+, Debian 11+ o Fedora 34+

### macOS Homebrew

Muchos usuarios necesitan actualizar los metadatos de Homebrew para que el cask se pueda encontrar o actualizar correctamente:

```bash
brew update
brew tap KeaneFeng/cc-gateway-pro
brew install --cask cc-gateway-pro
```

Actualizar:

```bash
brew update
brew upgrade --cask cc-gateway-pro
```

### Descarga manual

- Windows: descarga el instalador `.msi` o el `.zip` portátil desde [Releases](https://github.com/KeaneFeng/cc-gateway-pro/releases).
- macOS: descarga el `.dmg` desde [Releases](https://github.com/KeaneFeng/cc-gateway-pro/releases).
- Linux: descarga el `.deb`, el `.rpm` o el `.AppImage` desde [Releases](https://github.com/KeaneFeng/cc-gateway-pro/releases).
- Arch Linux: instala con `paru -S cc-gateway-pro-bin`.

## Compilar desde el código fuente

Requisitos previos: Node.js 22+, pnpm y Rust stable.

```bash
pnpm install
pnpm tauri dev
pnpm tauri build
```

El repositorio también incluye un script auxiliar:

```bash
./build.sh --dev
./build.sh --release
./build.sh --dmg
./build.sh --sha
```

## Documentación

- [README en inglés](README.md)
- [README en chino](README_ZH.md)
- [README en japonés](README_JA.md)
- [README en sueco](README_SV.md)
- [Manual de usuario](docs/user-manual/zh/README.md)
- [Arquitectura y flujos](docs/architecture-and-flows-zh.md)
- [Guía del proxy](docs/proxy-guide-zh.md)
- [Notas de la versión](docs/release-notes/v3.19.1-en.md)

## Ubicación de los datos

- Base de datos principal: `~/.cc-gateway-pro/cc-gateway-pro.db`
- Ajustes del dispositivo: `~/.cc-gateway-pro/settings.json`
- Copias de seguridad: `~/.cc-gateway-pro/backups/`
- Skills: `~/.cc-gateway-pro/skills/`

El directorio de almacenamiento se puede mover a Dropbox, OneDrive, iCloud, un NAS u otra carpeta sincronizada desde la página de ajustes.

## Créditos

- Fork de [farion1231/cc-switch](https://github.com/farion1231/cc-switch)
- Construido con Tauri, Rust, React, TypeScript y Tailwind CSS

## Licencia

MIT © Keane Feng
