# 開発ガイド

## 開発環境

### 必要なもの

| 項目 | バージョン | 備考 |
|------|-----------|------|
| Rust | 1.75+ | `x86_64-pc-windows-gnu` ターゲットが必要 |
| MinGW-w64 | — | Windows向けクロスコンパイラ |
| jq | — | ビルドスクリプト (makefile) で使用 |
| zip | — | `make zip-release` で使用 |
| Windows SDK | 10.0+ | Windows上で直接ビルドする場合 |
| Visual Studio Build Tools | 2022 | Windows上で直接ビルドする場合 |

### WSL (Ubuntu) でのセットアップ

WSL2 + Ubuntu を使った開発環境の構築手順。

#### 1. システムパッケージのインストール

```bash
sudo apt update
sudo apt install -y build-essential mingw-w64 jq zip
```

- `build-essential` — make, gcc 等の基本ビルドツール
- `mingw-w64` — Windows向けクロスコンパイラ (`x86_64-w64-mingw32-gcc`)
- `jq` — makefile 内で cargo の JSON 出力をパースするために使用
- `zip` — `make zip-release` でリリース ZIP を作成するために使用

#### 2. Rust のインストール

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

インストール後、バージョンを確認:

```bash
rustc --version
cargo --version
```

#### 3. Windows クロスコンパイルターゲットの追加

```bash
rustup target add x86_64-pc-windows-gnu
```

このターゲットにより、Linux 上で `.exe` / `.dll` をビルドできる。

#### 4. リポジトリのクローンとビルド確認

```bash
git clone <repository-url>
cd chamsae

# テスト実行 (Linux ネイティブ)
make test

# リリースビルド (Windows 向けクロスコンパイル)
make release
```

`make release` が成功すると `build/` ディレクトリに成果物が生成される。

#### 5. Windows 環境へのデプロイ

WSL から Windows 側にビルド成果物をコピーする:

```bash
# Windows 側のフォルダにコピー (例: デスクトップ)
cp -r build/ /mnt/c/Users/<ユーザー名>/Desktop/chamsae/
```

Windows 側で `install.bat` を実行すれば IME が登録される。

### Windows でのセットアップ

Windows 上で直接ビルドする場合は Visual Studio Build Tools 2022 と Windows SDK が必要。

```powershell
# Rustインストール (PowerShell)
winget install Rustlang.Rustup

# ビルド
cargo build --lib
cargo build --bins
```

## クイックスタート

### CLIツール

```bash
# リリースビルド (テスト → ビルド → build/ にコピー)
make release

# 単一変換
./build/chamsae.exe -i "an nyeong ha se yo"
# 出力: 안녕하세요

# 標準入力から変換 (引数なしで起動)
echo "han gug eo" | ./build/chamsae.exe
# 出力: 한국어

# インタラクティブモード
./build/chamsae.exe -I
> han gug eo
  → 한국어
> exit

# 設定ファイルのテンプレート生成
./build/chamsae.exe -t
# カレントディレクトリに chamsae.json を生成
```

### IME として使う

1. `make release` でビルド
2. `build/` フォルダを Windows 側にコピー
3. `install.bat` を実行 (UAC 昇格ダイアログが表示される)
4. Windows の「設定 > 時刻と言語 > 言語と地域」で「Chamsae Hangul IME」を確認
5. Shift+Space で IME ON/OFF を切り替え

詳細な設定・登録手順は [設定・登録ガイド](./configuration.md) を参照。

## make ターゲット

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

### ビルド成果物

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
├── readme.md
├── docs/                  # ドキュメント・仕様書
│   ├── development.md     # 本ファイル
│   ├── configuration.md   # 設定・登録ガイド
│   ├── roadmap.md         # ロードマップ
│   ├── changelog.md       # バージョン履歴
│   ├── references.md      # 参考資料
│   ├── spec_v0.1.0.md     # Phase 1 仕様
│   ├── spec_v0.2.0.md     # Phase 2 仕様
│   ├── spec_v0.3.0.md     # Phase 3 仕様
│   ├── spec_v0.4.0.md     # Phase 4 仕様
│   ├── spec_v0.5.0.md     # Phase 5 仕様
│   └── spec_v0.6.0.md     # Phase 6 仕様
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
