<div align="center">

# CC-Gateway-Pro

### Claude Code、Claude Desktop、Codex、Gemini CLI、OpenCode、OpenClaw、Hermes 向けローカル AI Provider Gateway

[![Version](https://img.shields.io/github/v/release/KeaneFeng/cc-gateway-pro?color=blue&label=version)](https://github.com/KeaneFeng/cc-gateway-pro/releases)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/KeaneFeng/cc-gateway-pro/releases)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-orange.svg)](https://tauri.app/)
[![Fork of cc-switch](https://img.shields.io/badge/fork%20of-farion1231%2Fcc--switch-blue.svg)](https://github.com/farion1231/cc-switch)

[English](README.md) | [中文](README_ZH.md) | 日本語 | [Changelog](CHANGELOG.md)

</div>

## CC-Gateway-Pro とは？

CC-Gateway-Pro は、AI コーディングツール向けのデスクトップ管理画面兼ローカル AI Provider Gateway です。プロバイダープロファイルを管理し、各ツールのネイティブ設定ファイルを書き込み、ローカルプロキシ経由で実リクエストをルーティングして、ログ、フェイルオーバー、モデル変換、プロジェクト単位の Provider バインド、Vision Model ルーティングを実現します。

このプロジェクトは [farion1231/cc-switch](https://github.com/farion1231/cc-switch) から fork して開発されています。オリジナルのビジュアルなプロバイダー切り替えを引き継ぎつつ、Rust/Tauri のローカルゲートウェイ、複数アプリの設定管理、リクエストルーティング、使用量分析、同期機能、プロバイダー別の連携を追加しています。

## 対応アプリ

| アプリ         | 主な機能                                                                                                                     |
| -------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| Claude Code    | Provider 切り替え、ローカルプロキシ接管、プロジェクトルーティング、Vision Model ルーティング、MCP、Prompts、Skills、Sessions |
| Claude Desktop | 公式/サードパーティプロファイル、直接接続モード、ローカルルーティングモード、モデルマッピング                                |
| Codex          | Provider 切り替え、ローカルプロキシ接管、プロジェクトルーティング、OAuth/Copilot 補助、MCP、Prompts、Skills、Sessions        |
| Gemini CLI     | Provider 切り替え、ローカルプロキシ接管、MCP、Prompts、Skills、Sessions                                                      |
| OpenCode       | Provider プリセット、共通設定スニペット、MCP、Prompts、Skills、Sessions                                                      |
| OpenClaw       | Provider プリセット、ワークスペースファイル、Agent 既定値、ツール/環境変数パネル                                             |
| Hermes Agent   | Provider プリセット、Memory パネル、MCP と Skills 管理                                                                       |

## 主な機能

- **Provider 管理**：50 以上のプリセット、カスタム Provider、ドラッグ並び替え、インポート/エクスポート、エンドポイント速度テスト、残高とクォータ補助。
- **ローカルプロキシゲートウェイ**：既定で `127.0.0.1:15721` を使用し、Tauri/Rust + Axum により Claude、Claude Desktop、Codex、Gemini のアプリ単位プロキシ接管を提供。
- **API 形式の変換**：Anthropic、OpenAI Chat Completions、OpenAI Responses、Gemini、プロバイダー固有の互換パスに対応。
- **プロジェクト単位ルーティング**：Claude/Codex のローカルセッションファイルからプロジェクトディレクトリを識別し、プロジェクトに紐づく Provider へルーティング。
- **Vision Model ルーティング**：画像ブロックを含むリクエストを、現在の Provider に設定された `vision_model` へ自動切り替え。
- **高可用性**：アプリ単位のフェイルオーバーキュー、サーキットブレーカー、リトライ、ヘルス状態、ストリーム/非ストリームのタイムアウト制御。
- **使用量分析**：リクエストログ、Token 使用量、コスト見積もり、トレンド、Provider/モデル別集計、カスタム価格。
- **MCP、Prompts、Skills**：対応アプリ間で統一管理と同期、Deep Link、リポジトリ/ZIP からの Skill インストール。
- **データ保護**：SQLite ストレージ、アトミック書き込み、自動バックアップ、移動可能なデータディレクトリ、WebDAV 同期。

## アーキテクチャ図

### ローカル AI Provider Gateway

![ローカル AI Provider Gateway アーキテクチャ](docs/assets/diagrams/cc-gateway-pro-architecture-zh.svg)

### Vision Model プロキシの仕組み

![Vision Model プロキシの仕組み](docs/assets/diagrams/vision-model-proxy-flow-zh.svg)

### Project Provider プロキシの仕組み

![Project Provider プロキシの仕組み](docs/assets/diagrams/project-provider-proxy-flow-zh.svg)

フェイルオーバー、使用量ログ、MCP/Prompts/Skills 同期、完全なリクエストチェーンについては [Architecture and Flows](docs/architecture-and-flows-zh.md) を参照してください。

## 画面プレビュー

|                  メイン画面                   |                  Provider 追加                  |
| :-------------------------------------------: | :---------------------------------------------: |
| ![メイン画面](assets/screenshots/main-ja.png) | ![Provider 追加](assets/screenshots/add-ja.png) |

|                              プロジェクトルーティング                              |                                Vision Model 設定                                |
| :--------------------------------------------------------------------------------: | :-----------------------------------------------------------------------------: |
| ![プロジェクトルーティング画面](assets/screenshots/project-routing-preview-zh.svg) | ![Vision Model 設定画面](assets/screenshots/vision-model-config-preview-zh.svg) |

## ダウンロードとインストール

最新ビルドは [GitHub Releases](https://github.com/KeaneFeng/cc-gateway-pro/releases) からダウンロードできます。

システム要件：

- macOS 12 以降
- Windows 10 以降
- Ubuntu 22.04+、Debian 11+、Fedora 34+ など、最新の WebKitGTK ランタイムを備えた Linux ディストリビューション

### macOS Homebrew

cask が見つからない、または古いバージョンが入る場合があるため、先に Homebrew のメタデータを更新してください。

```bash
brew update
brew tap KeaneFeng/cc-gateway-pro
brew install --cask cc-gateway-pro
```

アップグレード：

```bash
brew update
brew upgrade --cask cc-gateway-pro
```

### 手動ダウンロード

- Windows：[Releases](https://github.com/KeaneFeng/cc-gateway-pro/releases) から `.msi` インストーラーまたはポータブル `.zip` をダウンロード。
- macOS：[Releases](https://github.com/KeaneFeng/cc-gateway-pro/releases) から `.dmg` をダウンロード。
- Linux：[Releases](https://github.com/KeaneFeng/cc-gateway-pro/releases) から `.deb`、`.rpm`、`.AppImage` をダウンロード。
- Arch Linux：`paru -S cc-gateway-pro-bin` でインストール。

## ソースからビルド

必要なもの：Node.js 22+、pnpm、Rust stable。

```bash
pnpm install
pnpm tauri dev
pnpm tauri build
```

リポジトリには補助スクリプトも含まれています。

```bash
./build.sh --dev
./build.sh --release
./build.sh --dmg
./build.sh --sha
```

## ドキュメント

- [English README](README.md)
- [中文 README](README_ZH.md)
- [ユーザーマニュアル](docs/user-manual/ja/README.md)
- [Architecture and flows](docs/architecture-and-flows-zh.md)
- [Proxy guide](docs/proxy-guide-zh.md)
- [Release notes](docs/release-notes/v3.15.0-ja.md)

## データ保存場所

- メインデータベース：`~/.cc-gateway-pro/cc-gateway-pro.db`
- デバイス設定：`~/.cc-gateway-pro/settings.json`
- バックアップ：`~/.cc-gateway-pro/backups/`
- Skills：`~/.cc-gateway-pro/skills/`

設定画面から、データディレクトリを Dropbox、OneDrive、iCloud、NAS、または他の同期フォルダーへ移動できます。

## クレジット

- [farion1231/cc-switch](https://github.com/farion1231/cc-switch) から fork して開発
- Tauri、Rust、React、TypeScript、Tailwind CSS で構築

## License

MIT © Keane Feng
