# 開発ガイド

## 開発環境

### 必要なもの

| 項目 | バージョン |
|------|-----------|
| Rust | 1.75+ |
| Windows SDK | 10.0+ (Phase 2以降) |
| Visual Studio Build Tools | 2022 (Phase 2以降) |

### セットアップ

```bash
# Rustインストール
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windowsクロスコンパイル (Linux上で開発する場合)
rustup target add x86_64-pc-windows-gnu
```

### make ターゲット

| ターゲット | 説明 |
|-----------|------|
| `make` / `make all` | デバッグビルド (Windows向けクロスコンパイル) |
| `make release` | クリーン → テスト → リリースビルド → `build/` に成果物コピー |
| `make build-debug` | デバッグビルド |
| `make build-release` | リリースビルド |
| `make build-dll` | DLLのみリリースビルド |
| `make cp-release` | リリース成果物を `build/` にコピー (exe, dll, bat, chamsae.json) |
| `make zip-release` | `build/` をZIP化 (`chamsae-v{VERSION}.zip`) |
| `make test` | `cargo test` 実行 |
| `make clean` | `cargo clean` 実行 |
| `make installer` | InnoSetup手動コンパイル手順の表示 |

`make release` を実行すると、`build/` ディレクトリに以下が配置される:

```
build/
├── chamsae.exe              # CLIツール
├── chamsae_settings.exe     # 設定GUI
├── chamsae.dll              # IME DLL
├── chamsae.json             # 設定ファイル (テンプレート自動生成)
├── install.bat              # IME登録バッチ
└── uninstall.bat            # IME登録解除バッチ
```

この `build/` フォルダをWindows環境にコピーすればすぐに使える。

## ディレクトリ構成

```
chamsae/
├── Cargo.toml
├── makefile               # ビルド・テスト・リリース
├── chamsae.json           # 設定ファイルテンプレート
├── readme.md              # 本ファイル
├── docs/                  # 仕様書
│   ├── spec_v0.1.0.md     # Phase 1
│   ├── spec_v0.2.0.md     # Phase 2
│   ├── spec_v0.3.0.md     # Phase 3
│   ├── spec_v0.4.0.md     # Phase 4
│   ├── spec_v0.5.0.md     # Phase 5
│   └── spec_v0.6.0.md     # Phase 6
├── installer/
│   └── chamsae.iss        # InnoSetup定義
├── .github/workflows/
│   ├── ci.yml             # CI (テスト + ビルド)
│   └── release.yml        # CD (リリース自動化)
├── build/                 # make release の出力先 (gitignore)
└── src/
    ├── lib.rs             # ライブラリルート + DLLエクスポート
    ├── hangul.rs          # 変換ロジック + テスト
    ├── config.rs          # 設定ファイル読み込み (chamsae.json)
    ├── logger.rs          # ファイルベースロガー
    ├── user_dict.rs       # ユーザー辞書
    ├── guid.rs            # GUID/CLSID定義
    ├── registry.rs        # レジストリ登録 + TSF登録
    ├── bat/
    │   ├── install.bat     # IME登録 (UAC自動昇格)
    │   └── uninstall.bat   # IME登録解除 (UAC自動昇格)
    ├── com/
    │   ├── mod.rs         # COMモジュール
    │   ├── class_factory.rs   # IClassFactory実装
    │   └── dll_module.rs      # DLLモジュール管理
    ├── tsf/
    │   ├── mod.rs             # TSFモジュール
    │   ├── text_service.rs    # TextService (ITfTextInputProcessorEx)
    │   ├── key_handler.rs     # キーイベント判定
    │   ├── edit_session.rs    # EditSession (コンポジション操作)
    │   ├── registration.rs    # TSFプロファイル・カテゴリ登録
    │   ├── candidate_window.rs # 候補ウィンドウ (DPI対応)
    │   ├── dpi.rs             # DPIスケーリングヘルパー
    │   ├── tray_icon.rs       # システムトレイアイコン (設定再読み込み対応)
    │   └── icon.rs            # GDI動的アイコン生成
    ├── win32/
    │   ├── mod.rs         # Win32モジュール
    │   └── window.rs      # 基本ウィンドウ作成
    └── bin/
        ├── cli.rs         # CLIツール
        ├── settings.rs    # 設定GUI
        └── window_test.rs # ウィンドウテスト
```

## 技術スタック

| 用途 | ライブラリ |
|------|-----------|
| 引数解析 | clap |
| エラー処理 | anyhow |
| ログ出力 | log |
| Windows API | windows-rs 0.58 |
| JSON設定 | serde + serde_json |
| テスト | 標準 (cargo test) + tempfile |
