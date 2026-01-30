# Chamsae IME 仕様書 - Phase 5: 改善・拡張

## 概要

Phase 4の設定・トグル機能をベースに、ユーザー辞書、候補ウィンドウ、
システムトレイアイコン、設定GUI、InnoSetupインストーラー、CI/CDを追加する。

### 目標

- ユーザー辞書によるカスタム変換
- 候補ウィンドウ (コンポジション中の変換結果リアルタイム表示)
- システムトレイアイコン (IME状態表示・コンテキストメニュー)
- GDI動的アイコン生成 (リソースファイル不要)
- Win32設定GUI (`chamsae_settings.exe`)
- InnoSetupインストーラー定義
- GitHub Actions CI/CD (テスト・ビルド・リリース自動化)

### 対象環境

- OS: Windows 11 (優先), Linux (クロスコンパイル・テスト)
- 言語: Rust
- ビルド: MSVC (CI), MinGW (ローカル・クロスビルド)

---

## ユーザー辞書

### 仕様

**パス**: 設定ファイルの `user_dict_path` で指定。未指定時は `<DLLと同じディレクトリ>/user_dict.json`。

```json
{
  "entries": {
    "addr": "서울시 강남구",
    "name": "김철수",
    "email": "이메일 주소"
  }
}
```

### 動作

| 状態 | 動作 |
|------|------|
| ファイルなし | 空の辞書 (辞書なしで動作) |
| パース成功 | エントリを使用 |
| パース失敗 (不正JSON) | 空の辞書にフォールバック |

### 変換優先度

1. ユーザー辞書を完全一致検索
2. 一致あり → 辞書の値を使用
3. 一致なし → ハングル変換エンジンで変換

### 設定ファイル拡張

`chamsae.json` に `user_dict_path` フィールドを追加。

```json
{
  "toggle_key": { ... },
  "languages": { ... },
  "user_dict_path": "C:\\path\\to\\user_dict.json"
}
```

`user_dict_path` が `null` または未指定の場合、DLLと同じディレクトリの `user_dict.json` を自動検索する。

---

## 候補ウィンドウ

### 仕様

コンポジション中にキャレット位置に追従するポップアップウィンドウ。

```text
┌──────────┐
│ 한국어    │  ← 変換結果 (18pt, 黒)
│ han gug eo│  ← ローマ字入力 (13pt, グレー)
└──────────┘
```

### ウィンドウスタイル

| 属性 | 値 | 説明 |
|------|-----|------|
| スタイル | `WS_POPUP` | 枠なしポップアップ |
| 拡張スタイル | `WS_EX_TOOLWINDOW \| WS_EX_TOPMOST \| WS_EX_NOACTIVATE` | タスクバー非表示・最前面・フォーカス奪取なし |
| フォント | Meiryo UI | ClearType品質 |
| 背景色 | 白 (`#FFFFFF`) | |
| 枠色 | 青 (`#0080C0`) | |
| パディング | 6px | |

### 表示タイミング

| イベント | 動作 |
|---------|------|
| ハングルキー入力 | 表示・更新 (キャレット位置に追従) |
| BackSpace | 表示更新 (バッファ空なら非表示) |
| Enter / 非対応キー | 非表示 (コンポジション確定) |
| IMEトグルOFF | 非表示 |

### キャレット位置取得

EditSessionの `ITfContextView::GetTextExt()` でコンポジション範囲の矩形を取得し、
`RECT.left`, `RECT.bottom` をウィンドウ位置として使用する。

---

## システムトレイアイコン

### 仕様

`Shell_NotifyIconW` APIでシステムトレイにIME状態を表示する。

### アイコン

| 状態 | 外観 | 説明 |
|------|------|------|
| IME ON | 緑背景に「韓」 | GDIで動的生成 (16x16) |
| IME OFF | グレー背景に「A」 | GDIで動的生成 (16x16) |

### コンテキストメニュー (右クリック)

| メニュー項目 | ID | 動作 |
|-------------|-----|------|
| IME ON/OFF | 1001 | IMEトグル (状態に応じて表示テキスト変更) |
| ─ (区切り線) | | |
| 設定... | 1002 | `chamsae_settings.exe` を起動 |
| バージョン情報 | 1003 | バージョンダイアログ表示 |

### 操作

| 操作 | 動作 |
|------|------|
| 左クリック | IME ON/OFF トグル |
| 右クリック | コンテキストメニュー表示 |

### 実装

隠しウィンドウ (`WS_OVERLAPPEDWINDOW`, `SW_HIDE`) でトレイアイコンメッセージ (`WM_USER+1`) を受信する。
`TrayAction` enumでTextServiceにアクション通知。

---

## アイコン生成 (GDI)

### 仕様

リソースファイル (`.ico`, `.rc`) を使わず、GDI APIで16x16アイコンを動的生成する。

### 生成手順

1. `CreateCompatibleDC` + `CreateCompatibleBitmap` でカラーDC/ビットマップ作成
2. `FillRect` で背景色塗りつぶし
3. `CreateFontIndirectW` + `TextOutW` でテキスト描画 (中央配置)
4. マスクビットマップを全不透明 (0) で作成
5. `ICONINFO` → `CreateIconIndirect` でHICON生成

### フォント

| 属性 | 値 |
|------|-----|
| フェイス | Meiryo UI |
| 高さ | 漢字: -12, アルファベット: -11 |
| ウェイト | 700 (Bold) |
| 品質 | ClearType |

---

## 設定画面 (chamsae_settings.exe)

### 仕様

Win32ウィンドウベースの設定GUI。`chamsae.json` を読み書きする。

### UI構成

```text
┌─ Chamsae 設定 ──────────────────┐
│                                  │
│  トグルキー設定                    │
│  キー: [Space ▼]                 │
│  ☑ Shift  ☐ Ctrl  ☐ Alt         │
│                                  │
│  言語プロファイル                   │
│  ☑ 日本語  ☐ 韓国語               │
│                                  │
│  ユーザー辞書                      │
│  パス: [_____________] [参照...]  │
│  [辞書を編集...]                  │
│                                  │
│            [保存] [キャンセル]      │
└──────────────────────────────────┘
```

### コントロールID

| ID | コントロール | 説明 |
|-----|-----------|------|
| 1001 | ComboBox | キー選択 (Space, A-Z, 0-9) |
| 1002 | CheckBox | Shift修飾 |
| 1003 | CheckBox | Ctrl修飾 |
| 1004 | CheckBox | Alt修飾 |
| 1005 | CheckBox | 日本語プロファイル |
| 1006 | CheckBox | 韓国語プロファイル |
| 1007 | Edit | 辞書パス |
| 1008 | Button | 参照... (ファイル選択ダイアログ) |
| 1009 | Button | 辞書を編集... (メモ帳で開く) |
| 1010 | Button | 保存 |
| 1011 | Button | キャンセル |

### 動作

- 起動時: `chamsae.json` を読み込みUIに反映
- 保存: UIの値を `chamsae.json` に書き込み
- `#[cfg(not(windows))]`: 非Windows環境ではエラーメッセージを表示して終了

---

## InnoSetupインストーラー

### 仕様

`installer/chamsae.iss` にInnoSetup定義。Windows向けGUIインストーラーを生成する。

### インストール内容

| ファイル | インストール先 |
|---------|-------------|
| `chamsae.dll` | `{app}\` |
| `chamsae.exe` | `{app}\` |
| `chamsae_settings.exe` | `{app}\` |

### 自動処理

- インストール時: `regsvr32 /s chamsae.dll` (DLL登録)
- インストール後: `chamsae.exe -t` (設定テンプレート生成)
- アンインストール時: `regsvr32 /u /s chamsae.dll` (DLL登録解除)

### スタートメニュー

| ショートカット | 対象 |
|-------------|------|
| Chamsae 設定 | `chamsae_settings.exe` |
| Chamsae をアンインストール | アンインストーラー |

---

## CI/CD (GitHub Actions)

### CI ワークフロー (`.github/workflows/ci.yml`)

#### トリガー

- `push` (全ブランチ)
- `pull_request` (masterブランチ)

#### ジョブ構成

| ジョブ | ランナー | 依存 | 内容 |
|-------|---------|------|------|
| `test` | ubuntu-latest | - | `cargo test` (67テスト) |
| `cross-build` | ubuntu-latest | test | mingw クロスコンパイル検証 |
| `build` | windows-latest | test | ネイティブビルド + ZIP作成 + インストーラー作成 + アーティファクト |

#### build ジョブの成果物

`make release` 相当のZIP + InnoSetupインストーラーをアーティファクトとして登録:

```
chamsae-v{VERSION}.zip
├── chamsae.exe
├── chamsae_settings.exe
├── chamsae.dll
├── chamsae.json   (テンプレート自動生成)
├── install.bat
└── uninstall.bat

chamsae-v{VERSION}-setup.exe   (InnoSetupインストーラー)
```

### Release ワークフロー (`.github/workflows/release.yml`)

#### トリガー

- `push` tags: `v*` (例: `v0.6.0`)

#### 処理

1. テスト実行
2. リリースビルド (Windows)
3. 成果物収集 + `chamsae.exe -t` (設定テンプレート生成)
4. `chamsae-v{VERSION}.zip` 作成
5. `iscc` でインストーラー (`chamsae-v{VERSION}-setup.exe`) 作成
6. GitHub Release作成 + ZIP・インストーラーアップロード (rcタグ時はpre-release)

---

## ファイル構成

### 新規ファイル

| ファイル | 内容 |
|---------|------|
| `src/user_dict.rs` | ユーザー辞書モジュール |
| `src/tsf/candidate_window.rs` | 候補ウィンドウ |
| `src/tsf/tray_icon.rs` | システムトレイアイコン |
| `src/tsf/icon.rs` | GDI動的アイコン生成 |
| `src/bin/settings.rs` | 設定画面GUI |
| `installer/chamsae.iss` | InnoSetup定義 |
| `.github/workflows/ci.yml` | CI (テスト + ビルド) |
| `.github/workflows/release.yml` | CD (リリース自動化) |

### 変更ファイル

| ファイル | 変更内容 |
|---------|---------|
| `Cargo.toml` | `chamsae_settings` バイナリ追加、`windows` crate features追加 |
| `src/lib.rs` | `pub mod user_dict` 追加 |
| `src/config.rs` | `user_dict_path` フィールド追加 |
| `src/tsf/mod.rs` | `candidate_window`, `tray_icon`, `icon` モジュール追加 |
| `src/tsf/text_service.rs` | ユーザー辞書・候補ウィンドウ・トレイアイコン統合 |
| `src/tsf/edit_session.rs` | `CaretPos` 構造体追加、キャレット位置取得 |
| `makefile` | `chamsae_settings.exe` コピー追加、`installer` ターゲット追加 |

---

## テスト

### 追加テスト (user_dict.rs)

| テスト名 | 内容 |
|---------|------|
| `test_load_valid_dict` | 正常JSON読み込み・完全一致検索 |
| `test_load_missing_file` | ファイルなし → 空辞書 |
| `test_load_invalid_json` | 不正JSON → 空辞書 |
| `test_empty_dict` | 空辞書のlookup → None |
| `test_empty_entries` | 空エントリのlookup → None |

### テスト合計: 67テスト

---

## 依存関係

### 追加 `windows` crate features

| Feature | 用途 |
|---------|------|
| `Win32_System_Threading` | プロセス起動 (設定画面・メモ帳) |
| `Win32_UI_Shell` | トレイアイコン (`Shell_NotifyIconW`) |
| `Win32_UI_Controls` | コンボボックス・チェックボックス |
| `Win32_UI_Controls_Dialogs` | ファイル選択ダイアログ (`GetOpenFileNameW`) |
