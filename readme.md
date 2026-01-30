# Chamsae - Hangul IME (ハングルIME)

ローマ字入力から韓国語ハングル文字への変換を行うIME (Input Method Editor)。

Rust学習を兼ねた自作IMEプロジェクト。

## インストール

### インストーラー (推奨)

1. [最新リリース](https://github.com/hatolife/chamsae/releases/latest) から `chamsae-vX.Y.Z-setup.exe` をダウンロード
2. セットアップウィザードを実行 (管理者権限が必要)

インストーラーはDLL登録・設定テンプレート生成を自動で行う。
アップグレード時はDLLをリネームして新バージョンを配置するため、再起動不要。

### 手動インストール (ZIP)

1. [最新リリース](https://github.com/hatolife/chamsae/releases/latest) からzipファイルをダウンロード
2. zipを展開し、任意のフォルダに配置（例: `C:\Program Files\Chamsae\`）
3. `install.bat` をダブルクリックして実行（UAC昇格ダイアログが表示される）

登録後、Windowsの「設定 > 時刻と言語 > 言語と地域」で「Chamsae Hangul IME」が表示される。

## アンインストール

### インストーラー版

「設定 > アプリ > インストールされているアプリ」から「Chamsae Hangul IME」をアンインストール。

### 手動インストール版

1. `uninstall.bat` をダブルクリックして実行（UAC昇格ダイアログが表示される）
2. 配置したフォルダを削除

## 現在のステータス

| 項目 | 状態 |
|------|------|
| バージョン | v0.6.5 |
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

## ドキュメント

| ドキュメント | 内容 |
|-------------|------|
| [開発ガイド](./docs/development.md) | 開発環境・セットアップ・ディレクトリ構成・技術スタック |
| [設定・登録ガイド](./docs/configuration.md) | IME登録手順・設定ファイル・ユーザー辞書・トラブルシューティング |
| [ロードマップ](./docs/roadmap.md) | Phase 1〜9 の開発計画・進捗 |

---

## バージョン履歴

| バージョン | 日付 | 内容 |
|-----------|------|------|
| v0.6.5 | 2026-01-30 | インストーラー: リネーム方式DLLアップグレード (再起動不要) |
| v0.6.4 | 2026-01-30 | 候補ウィンドウ: 確定・キャンセル・修飾キー・フォーカス喪失時の非表示修正 |
| v0.6.1 | 2026-01-30 | 設定GUI: コンソール非表示・背景色統一, 設定ファイルを%APPDATA%に移動 |
| v0.6.0 | 2026-01-30 | ログ出力, ナビゲーションキー対応, 設定ホットリロード, DPI/マルチモニター対応 |
| v0.5.0 | 2026-01-30 | ユーザー辞書, 候補ウィンドウ, トレイアイコン, 設定GUI, インストーラー, CI/CD |
| v0.4.0 | 2026-01-30 | IMEトグル, 設定ファイル, 言語プロファイル設定, 連音化 |
| v0.3.0 | 2026-01-30 | TSF IME実装, キーイベント処理, コンポジション |
| v0.2.0 | 2026-01-30 | Windows DLL構造, COM基礎, Win32ウィンドウ |
| v0.0.1 | 2025-01-30 | 変換エンジン完成、CLIツール |

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
