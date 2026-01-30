# Chamsae IME 仕様書 - Phase 6: 実用性向上

## 概要

Phase 5の機能をベースに、ログ出力、ナビゲーションキー対応、設定ホットリロード、
DPI対応・マルチモニター対応を追加する。

### 目標

- ファイルベースログ出力 (DLLデバッグ・トラブルシューティング)
- ナビゲーションキー (矢印/Home/End/Delete/Tab) の明示的対応
- 設定ホットリロード (IME再起動不要)
- DPIスケーリング・マルチモニター対応 (候補ウィンドウ・設定画面)

### 対象環境

- OS: Windows 11 (優先), Linux (クロスコンパイル・テスト)
- 言語: Rust
- ビルド: MSVC (CI), MinGW (ローカル・クロスビルド)

---

## 6.6 ログ出力

### 仕様

DLLはstdout/stderrが使えないため、ファイルベースのロガーを実装。
`log` クレートの `Log` トレイトを実装し、DLLと同じディレクトリの `chamsae.log` に追記する。

### ファイル

- `src/logger.rs`: `FileLogger` 構造体 (`log::Log` トレイト実装)
- `Cargo.toml`: `log = "0.4"` 追加

### 動作

| タイミング | ログ内容 |
|-----------|---------|
| DLL_PROCESS_ATTACH | `Chamsae IME DLL loaded` |
| Activate | `TextService::Activate (tid=N)` |
| Deactivate | `TextService::Deactivate` |
| IMEトグル | `IME toggled: ON/OFF` |
| トレイ操作 | `Tray: launching settings` / `Tray: showing about dialog` |
| 設定読み込み | 成功時: キー設定情報 / 失敗時: 警告メッセージ |
| ナビゲーションキー | `Navigation key (vk=0xNN): auto-commit and passthrough` |
| 設定再読み込み | `Reloading config and user dictionary` |

### ログフォーマット

```
[UNIX_TIMESTAMP LEVEL] メッセージ
```

例:
```
[1706600000  INFO] Chamsae IME DLL loaded
[1706600001  INFO] Config loaded: toggle=Space(0x20) shift=true ctrl=false alt=false
[1706600002  INFO] TextService::Activate (tid=1)
```

---

## 6.1 ナビゲーションキー対応

### 仕様

コンポジション中に以下のキーが押された場合、コンポジションを自動確定してからキーをパススルーする。
候補ウィンドウも非表示にする。

### 対象キー

| キー | VK定数 | コード |
|------|--------|--------|
| Tab | VK_TAB | 0x09 |
| Left Arrow | VK_LEFT | 0x25 |
| Up Arrow | VK_UP | 0x26 |
| Right Arrow | VK_RIGHT | 0x27 |
| Down Arrow | VK_DOWN | 0x28 |
| Home | VK_HOME | 0x24 |
| End | VK_END | 0x23 |
| Delete | VK_DELETE | 0x2E |

### 関数

- `key_handler::is_navigation_key(vk: u32) -> bool`

### 動作フロー

1. ナビゲーションキー押下を検出
2. コンポジション確定 (`EditAction::Commit`)
3. ローマ字バッファクリア
4. 候補ウィンドウ非表示
5. キーをアプリケーションにパススルー (`Ok(FALSE)`)

### テスト

- `test_is_navigation_key_arrows`: 矢印キー4つ
- `test_is_navigation_key_home_end_delete_tab`: Home/End/Delete/Tab
- `test_is_navigation_key_not_navigation`: 非ナビゲーションキーが`false`
- `test_vk_to_char`: VK→文字変換
- `test_is_hangul_key`: ハングルキー判定
- `test_is_control_key`: 制御キー判定

---

## 6.3 設定ホットリロード

### 仕様

トレイメニューに「設定の再読み込み」を追加。
`config` と `user_dict` を `RefCell` でラップし、実行中の再読み込みを可能にする。

### トレイメニュー

| 項目 | ID | アクション |
|------|-----|----------|
| IME ON/OFF | IDM_TOGGLE (1001) | トグル |
| (区切り線) | - | - |
| 設定の再読み込み | IDM_RELOAD (1004) | 設定再読み込み |
| 設定... | IDM_SETTINGS (1002) | 設定画面起動 |
| バージョン情報 | IDM_ABOUT (1003) | バージョンダイアログ |

### TrayAction列挙型

```rust
pub enum TrayAction {
    None,
    Toggle,
    Reload,  // 新規追加
}
```

### TextService変更

| フィールド | 旧型 | 新型 |
|-----------|------|------|
| config | `Config` | `RefCell<Config>` |
| user_dict | `UserDict` | `RefCell<UserDict>` |

### メソッド

- `reload_config()`: `Config::load_from_dll()` で再読み込み、ユーザー辞書も再読み込み
- `check_tray_action()`: `OnKeyDown` 先頭で呼び出し、トレイメニューのアクションを処理

### 設定画面メッセージ変更

保存成功メッセージを変更:

旧: `設定を保存しました。\nIMEを再起動すると反映されます。`

新: `設定を保存しました。\nトレイメニューの「設定の再読み込み」で反映できます。`

---

## 6.4 DPI対応・マルチモニター

### 仕様

候補ウィンドウのフォント・パディングをDPIスケーリングし、
マルチモニター環境で画面端からはみ出さないようクランプする。

### DPIヘルパー (`src/tsf/dpi.rs`)

```rust
pub fn get_dpi_for_window(hwnd: HWND) -> u32
pub fn scale(value: i32, dpi: u32) -> i32
```

- `get_dpi_for_window`: `GetDpiForWindow` APIでウィンドウのDPIを取得 (デフォルト96)
- `scale`: `value * dpi / 96` で物理ピクセルに変換

### スケーリング対象

| 要素 | 基準値 (96dpi) | 150% (144dpi) | 200% (192dpi) |
|------|---------------|---------------|---------------|
| ハングルフォント | -18 | -27 | -36 |
| ローマ字フォント | -13 | -19 | -26 |
| パディング | 6 | 9 | 12 |
| 最小幅 | 60 | 90 | 120 |
| 最小高さ | 40 | 60 | 80 |

### モニタークランプ

`CandidateWindow::clamp_to_monitor(x, y, width, height) -> (i32, i32)`

- `MonitorFromPoint` で最寄りモニターを取得
- `GetMonitorInfoW` で作業領域 (`rcWork`) を取得
- ウィンドウが作業領域をはみ出す場合にクランプ:
  - 右端超過: 左にずらす
  - 下端超過: 上にずらす
  - 左端/上端: 最小座標にクランプ

### 設定画面 (`settings.rs`)

- `SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)` 呼び出し
- Per-Monitor DPI Awareness v2を有効化

### テスト

- `test_scale_100_percent`: 96dpi (100%) のスケーリング
- `test_scale_150_percent`: 144dpi (150%) のスケーリング
- `test_scale_200_percent`: 192dpi (200%) のスケーリング
- `test_scale_zero`: ゼロ値のスケーリング

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---------|---------|
| `Cargo.toml` | `log = "0.4"` 追加、`Win32_UI_HiDpi` feature追加 |
| `src/logger.rs` | **新規** ファイルベースロガー |
| `src/lib.rs` | `pub mod logger` 追加、DllMainでロガー初期化 |
| `src/config.rs` | 設定読み込み成功/失敗ログ追加 |
| `src/tsf/mod.rs` | `pub mod dpi` 追加 |
| `src/tsf/dpi.rs` | **新規** DPIスケーリングヘルパー |
| `src/tsf/key_handler.rs` | VK定数追加、`is_navigation_key()` 追加、テスト追加 |
| `src/tsf/text_service.rs` | RefCell化、ナビゲーションキーアーム、reload/check_tray_action |
| `src/tsf/tray_icon.rs` | `TrayAction::Reload`、`IDM_RELOAD` メニュー追加、ログ追加 |
| `src/tsf/candidate_window.rs` | DPIスケーリング適用、モニタークランプ追加 |
| `src/bin/settings.rs` | DPI Awareness設定、保存メッセージ変更 |

---

## v0.6.1 設定GUI改善・設定ディレクトリ移行

### 設定GUIコンソール非表示

`settings.rs` に `#![cfg_attr(windows, windows_subsystem = "windows")]` を追加し、
設定画面起動時にコマンドプロンプトウィンドウが表示されないようにした。

### 背景色統一

`WM_CTLCOLORSTATIC` / `WM_CTLCOLORBTN` メッセージのハンドラを追加し、
子コントロールの背景色をウィンドウ背景色 (白) と統一した。

### 設定ディレクトリ移行

設定ファイル・ユーザー辞書・ログファイルの保存先を DLLディレクトリから `%APPDATA%\Chamsae\` に移行。
`Program Files` 以下ではユーザー権限でファイル書き込みができないため。

| ファイル | 旧パス | 新パス |
|---------|--------|--------|
| 設定 | `<DLL_DIR>\chamsae.json` | `%APPDATA%\Chamsae\chamsae.json` |
| ユーザー辞書 | `<DLL_DIR>\user_dict.json` | `%APPDATA%\Chamsae\user_dict.json` |
| ログ | `<DLL_DIR>\chamsae.log` | `%APPDATA%\Chamsae\chamsae.log` |

### 変更ファイル

| ファイル | 変更内容 |
|---------|---------|
| `src/config.rs` | `get_config_directory()` 追加 (`%APPDATA%\Chamsae\`、なければ作成) |
| `src/lib.rs` | ロガー初期化を `get_config_directory()` に変更 |
| `src/tsf/text_service.rs` | ユーザー辞書パスを `get_config_directory()` に変更 |
| `src/bin/settings.rs` | コンソール非表示、背景色統一、設定/辞書パスを `get_config_directory()` に変更 |

---

## v0.6.4 候補ウィンドウ表示修正

### フォーカス喪失時

`OnSetFocus(FALSE)` で変換バッファをクリアし候補ウィンドウを非表示にする。
以前は `OnSetFocus` が空実装だったため、フォーカス移動後も候補ウィンドウが残っていた。

### 確定・キャンセル時

以下のすべての確定パスに `candidate_window.hide()` を追加:

| パス | キー/条件 |
|------|----------|
| 確定 | Enter |
| キャンセル | Escape |
| 修飾キー自動確定 | Ctrl+X / Alt+X など |
| 未対応キー自動確定 | ハングルキー以外 |

### 変更ファイル

| ファイル | 変更内容 |
|---------|---------|
| `src/tsf/text_service.rs` | `OnSetFocus(FALSE)` 実装、全確定パスに `candidate_window.hide()` 追加 |

---

## v0.6.5 インストーラーアップグレード改善

### 課題

IMEのDLLはWindowsが常にロードしているため、上書きインストールが失敗する。
再起動を要求せずにアップグレードする方法が必要。

### リネーム方式

Windowsではロック中のファイルのリネームは成功する。この特性を利用して再起動不要のアップグレードを実現。

### アップグレードフロー

```
PrepareToInstall:
  1. 前回の .old ファイルを削除 (残っていれば)
  2. 旧DLLのTSF登録を解除 (regsvr32 /u)
  3. 旧DLLを chamsae.dll.old にリネーム (ロック中でも成功)

[Files] セクション:
  4. 新DLLを chamsae.dll として配置

[Run] セクション:
  5. 新DLLをTSF登録 (regsvr32)

CurStepChanged(ssPostInstall):
  6. .old ファイルの削除を試みる (ロック中なら次回インストール時に削除)
```

### アンインストールフロー

```
[UninstallRun]:
  1. regsvr32 /u chamsae.dll (TSF登録解除)

[Files] 自動削除:
  2. インストール済みファイルを削除
```

### InnoSetup [Code] セクション

```pascal
function PrepareToInstall(var NeedsRestart: Boolean): String;
  // .old 削除 → regsvr32 /u → リネーム

procedure CurStepChanged(CurStep: TSetupStep);
  // ssPostInstall: .old 削除試行
```

### 変更ファイル

| ファイル | 変更内容 |
|---------|---------|
| `installer/chamsae.iss` | `[Code]` セクション追加 (PrepareToInstall, CurStepChanged) |

---

## 未実装 (Phase 7へ移行)

| 項目 | 内容 |
|------|------|
| 6.2 | 候補ウィンドウの複数候補表示・選択 (ユーザー辞書の前方一致) |
| 6.5 | コンポジションプレビュー (入力中の字母を逐次表示) |
