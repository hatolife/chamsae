# Chamsae - Hangul IME (ハングルIME)

ローマ字入力から韓国語ハングル文字への変換を行うIME (Input Method Editor)。

Rust学習を兼ねた自作IMEプロジェクト。

## インストール

1. [最新リリース](https://github.com/hatolife/chamsae/releases/latest) からzipファイルをダウンロード
2. zipを展開し、任意のフォルダに配置（例: `C:\Program Files\Chamsae\`）
3. `chamsae.json` を必要に応じて編集（トグルキーや言語プロファイルの設定）
4. `install.bat` をダブルクリックして実行（UAC昇格ダイアログが表示される）

登録後、Windowsの「設定 > 時刻と言語 > 言語と地域」で「Chamsae Hangul IME」が表示される。

## アンインストール

1. `uninstall.bat` をダブルクリックして実行（UAC昇格ダイアログが表示される）
2. 配置したフォルダを削除

## 現在のステータス

| 項目 | 状態 |
|------|------|
| バージョン | v0.6.0 |
| フェーズ | Phase 6 完了 |
| テスト | 67テスト通過 |
| CI/CD | GitHub Actions (テスト・ビルド・リリース自動化) |
| 対応OS | CLI: Windows/Linux, DLL/IME: Windows |

## クイックスタート

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

## 入力規則

| 入力 | 意味 | 例 |
|------|------|-----|
| 半角スペース1つ | 音節区切り | `han gug` → 한국 |
| 半角スペース2つ | 実際のスペース | `an nyeong  ha se yo` → 안녕 하세요 |

詳細は [spec_v0.1.0.md](./docs/spec_v0.1.0.md) / [spec_v0.2.0.md](./docs/spec_v0.2.0.md) / [spec_v0.3.0.md](./docs/spec_v0.3.0.md) / [spec_v0.4.0.md](./docs/spec_v0.4.0.md) / [spec_v0.5.0.md](./docs/spec_v0.5.0.md) / [spec_v0.6.0.md](./docs/spec_v0.6.0.md) を参照。

---

## ロードマップ

### Phase 1: 変換エンジン ✅ 完了

| 内容 | 状態 |
|------|------|
| Rust環境構築・基礎学習 | ✅ |
| CLIツール (clap, anyhow) | ✅ |
| ハングル変換ロジック | ✅ |

### Phase 2: Windows API入門 ✅ 完了

| マイルストーン | 内容 | 状態 |
|---------------|------|------|
| 2.1 | windows-rs セットアップ | ✅ |
| 2.2 | Win32 基本ウィンドウ作成 | ✅ |
| 2.3 | COM基礎 (IUnknown, ClassFactory) | ✅ |
| 2.4 | DLL作成・レジストリ登録 | ✅ |

### Phase 3: TSF IME実装 ✅ 完了

| マイルストーン | 内容 | 状態 |
|---------------|------|------|
| 3.1 | TSF最小スケルトン | ✅ |
| 3.2 | ITfTextInputProcessorEx実装 | ✅ |
| 3.3 | キーイベント処理 | ✅ |
| 3.4 | 変換ロジック統合 | ✅ |
| 3.5 | コンポジション下線表示 | ✅ |

### Phase 4: リアルタイム変換・設定 ✅ 完了

| マイルストーン | 内容 | 状態 |
|---------------|------|------|
| 4.1 | 終声の自動移動 (連音化) | ✅ |
| 4.2 | IME ON/OFF トグル | ✅ |
| 4.3 | 非対応キー入力時の自動確定 | ✅ |
| 4.4 | エッジケース修正 | ✅ |
| 4.5 | 設定ファイル (トグルキー変更) | ✅ |
| 4.6 | 言語プロファイル設定 (日本語/韓国語) | ✅ |

### Phase 5: 改善・拡張 ✅ 完了

| マイルストーン | 内容 | 状態 |
|---------------|------|------|
| 5.1 | ユーザー辞書 | ✅ |
| 5.2 | 候補ウィンドウ (変換結果リアルタイム表示) | ✅ |
| 5.3 | システムトレイアイコン (IME状態・メニュー) | ✅ |
| 5.4 | 設定画面 (Win32 GUI) | ✅ |
| 5.5 | InnoSetupインストーラー定義 | ✅ |
| 5.6 | CI/CD (GitHub Actions) | ✅ |

### Phase 6: 実用性向上 ✅ 完了

| マイルストーン | 内容 | 状態 |
|---------------|------|------|
| 6.1 | 矢印キー・Home/End/Delete/Tab対応 (コンポジション中の自動確定) | ✅ |
| 6.3 | 設定ホットリロード (トレイメニューから再読み込み) | ✅ |
| 6.4 | DPI対応・マルチモニター (候補ウィンドウ・設定画面) | ✅ |
| 6.6 | ログ出力 (ファイルベースロガー) | ✅ |

### Phase 7: 入力機能強化

| マイルストーン | 内容 | 優先度 |
|---------------|------|--------|
| 7.1 | 候補ウィンドウの複数候補表示・選択 (ユーザー辞書の前方一致) | 高 |
| 7.2 | コンポジションプレビュー (入力中の字母を逐次表示) | 高 |
| 7.3 | 漢字 (한자) 変換 | 中 |
| 7.4 | 2ボル式キーボードレイアウト対応 (標準韓国語配列) | 中 |
| 7.5 | 入力履歴・頻度学習辞書 | 低 |

**目標**: 候補選択による入力効率向上、複数入力方式サポート

### Phase 8: 高度な機能

| マイルストーン | 内容 | 優先度 |
|---------------|------|--------|
| 8.1 | 記号・特殊文字入力モード (ㄱ+漢字キー方式) | 中 |
| 8.2 | 辞書前方一致・予測変換 | 中 |
| 8.3 | アプリケーション別設定プロファイル | 低 |

**目標**: 韓国語ネイティブユーザーにも使える入力方式

### Phase 9: 品質・配布

| マイルストーン | 内容 | 優先度 |
|---------------|------|--------|
| 9.1 | Windows統合テスト (CI上でのIME動作検証) | 高 |
| 9.2 | コード署名 (SmartScreen警告の回避) | 高 |
| 9.3 | winget / Microsoft Store 配布 | 中 |
| 9.4 | ユーザーガイド (使い方ドキュメント) | 低 |
| 9.5 | パフォーマンス最適化 (変換エンジン・メモリ) | 低 |

**目標**: v1.0.0 リリース、一般配布可能な品質

---

## バージョン履歴

| バージョン | 日付 | 内容 |
|-----------|------|------|
| v0.6.0 | 2026-01-30 | ログ出力, ナビゲーションキー対応, 設定ホットリロード, DPI/マルチモニター対応 |
| v0.5.0 | 2026-01-30 | ユーザー辞書, 候補ウィンドウ, トレイアイコン, 設定GUI, インストーラー, CI/CD |
| v0.4.0 | 2026-01-30 | IMEトグル, 設定ファイル, 言語プロファイル設定, 連音化 |
| v0.3.0 | 2026-01-30 | TSF IME実装, キーイベント処理, コンポジション |
| v0.2.0 | 2026-01-29 | Windows DLL構造, COM基礎, Win32ウィンドウ |
| v0.0.1 | 2025-01-30 | 変換エンジン完成、CLIツール |

---

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

### ディレクトリ構成

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

---

## 技術スタック

| 用途 | ライブラリ |
|------|-----------|
| 引数解析 | clap |
| エラー処理 | anyhow |
| ログ出力 | log |
| Windows API | windows-rs 0.58 |
| JSON設定 | serde + serde_json |
| テスト | 標準 (cargo test) + tempfile |

---

## IME登録 (Windows)

DLLをWindowsに登録してIMEとして使用する手順。

`regsvr32` を実行すると以下が行われる:
1. CLSID/InprocServer32 のレジストリ登録
2. 設定ファイル (`chamsae.json`) の読み込み (なければデフォルトで新規作成)
3. TSFプロファイル登録 (設定に基づき日本語/韓国語キーボードとして登録)
4. TSFカテゴリ登録 (キーボードTIPとして登録)

### ビルド

```bash
# Linux (WSL) でクロスコンパイル → build/ に成果物配置
make release
```

### 登録・解除

`build/` フォルダ内のバッチファイルをダブルクリックで実行する。
管理者権限が必要な場合は自動でUAC昇格ダイアログが表示される。

| バッチファイル | 説明 |
|---------------|------|
| `install.bat` | `regsvr32 chamsae.dll` を実行してIMEを登録 |
| `uninstall.bat` | `regsvr32 /u chamsae.dll` を実行してIME登録を解除 |

手動で実行する場合 (**管理者権限のコマンドプロンプト**):

```bat
REM DLLの登録 (IMEとしてシステムに追加)
regsvr32 chamsae.dll

REM DLLの登録解除
regsvr32 /u chamsae.dll
```

### 登録の確認

レジストリエディタ (`regedit`) で以下のキーが作成されていれば成功です。
このGUIDは `src/guid.rs` で固定定義された値で、ビルドごとに変わりません。

```
HKEY_CLASSES_ROOT\CLSID\{D4A5B8E1-7C2F-4A3D-9E6B-1F8C0D2A5E7B}
├── (Default) = "Chamsae Hangul IME"
└── InprocServer32
    ├── (Default) = "<DLLの絶対パス>"
    └── ThreadingModel = "Apartment"
```

登録後、Windowsの「設定 > 時刻と言語 > 言語と地域」で「Chamsae Hangul IME」が表示される。
デフォルトでは日本語キーボードとして登録される。韓国語としても登録するには設定ファイルを変更する (下記参照)。

### 設定ファイル

DLLと同じディレクトリに `chamsae.json` が配置される (初回登録時に自動作成)。

```json
{
  "toggle_key": {
    "key": "Space",
    "shift": true,
    "ctrl": false,
    "alt": false
  },
  "languages": {
    "japanese": true,
    "korean": false
  },
  "user_dict_path": null
}
```

#### toggle_key

IME ON/OFF の切り替えキー。デフォルトは Shift+Space。

| フィールド | 説明 | 指定可能な値 |
|-----------|------|-------------|
| `key` | キー名 | `"A"`〜`"Z"`, `"0"`〜`"9"`, `"Space"` |
| `shift` | Shift同時押し | `true` / `false` |
| `ctrl` | Ctrl同時押し | `true` / `false` |
| `alt` | Alt同時押し | `true` / `false` |

例: Alt+S に変更する場合:
```json
"toggle_key": { "key": "S", "shift": false, "ctrl": false, "alt": true }
```

#### languages

`regsvr32` 実行時にどの言語プロファイルを登録するかを制御する。
設定変更後は `regsvr32 /u` で登録解除してから再登録する。

| フィールド | 説明 | デフォルト |
|-----------|------|----------|
| `japanese` | 日本語キーボードとして登録 | `true` |
| `korean` | 韓国語キーボードとして登録 | `false` |

#### user_dict_path

ユーザー辞書ファイルのパス。`null` または未指定の場合、DLLと同じディレクトリの `user_dict.json` を自動検索する。

```json
"user_dict_path": "C:\\Users\\user\\my_dict.json"
```

### ユーザー辞書

DLLと同じディレクトリに `user_dict.json` を配置すると、カスタム変換が使用できる。
設定ファイルの `user_dict_path` で別のパスを指定することも可能。

```json
{
  "entries": {
    "addr": "서울시 강남구",
    "name": "김철수"
  }
}
```

変換時にユーザー辞書を完全一致検索し、一致すれば辞書の値を使用する。

### トラブルシューティング

| 症状 | 原因・対処 |
|------|-----------|
| `regsvr32` でアクセス拒否 | 管理者権限で実行していない。コマンドプロンプトを「管理者として実行」で開く |
| モジュールが見つからない | DLLパスが間違っている。絶対パスで指定するか、DLLのあるディレクトリで実行する |
| エントリポイントが見つからない | ビルドターゲットが正しくない。`--target x86_64-pc-windows-gnu` を確認する |
| IMEが言語設定に表示されない | TSFプロファイル登録に失敗している可能性。`regsvr32` の出力を確認する |

---

## 参考資料

### ハングル

- [Unicode Hangul Syllables](https://en.wikipedia.org/wiki/Korean_language_and_computers#Hangul_in_Unicode)
- [Revised Romanization of Korean](https://en.wikipedia.org/wiki/Revised_Romanization_of_Korean)

### Windows TSF

- [Text Services Framework (Microsoft Docs)](https://docs.microsoft.com/en-us/windows/win32/tsf/text-services-framework)
- [Windows Classic Samples - IME](https://github.com/microsoft/Windows-classic-samples/tree/main/Samples/IME)
- [windows-rs](https://github.com/microsoft/windows-rs)

### Rust IME実装例

- [rust-ime](https://github.com/pxsta/rust-ime) - Rust TSF IMEの参考実装

---

## ライセンス

CC0 1.0 Universal

---

## 作者メモ

このプロジェクトはRust学習を目的としています。
実用IMEとしての完成度より、学習過程を重視しています。
