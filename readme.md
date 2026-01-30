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

## 入力規則

| 入力 | 意味 | 例 |
|------|------|-----|
| 半角スペース1つ | 音節区切り | `han gug` → 한국 |
| 半角スペース2つ | 実際のスペース | `an nyeong  ha se yo` → 안녕 하세요 |

詳細は [spec_v0.1.0.md](./docs/spec_v0.1.0.md) / [spec_v0.2.0.md](./docs/spec_v0.2.0.md) / [spec_v0.3.0.md](./docs/spec_v0.3.0.md) / [spec_v0.4.0.md](./docs/spec_v0.4.0.md) / [spec_v0.5.0.md](./docs/spec_v0.5.0.md) / [spec_v0.6.0.md](./docs/spec_v0.6.0.md) を参照。

## ドキュメント

| ドキュメント | 内容 |
|-------------|------|
| [開発ガイド](./docs/development.md) | 環境構築 (WSL)・クイックスタート・ビルド・ディレクトリ構成 |
| [設定・登録ガイド](./docs/configuration.md) | IME登録手順・設定ファイル・ユーザー辞書・トラブルシューティング |
| [ロードマップ](./docs/roadmap.md) | Phase 1〜9 の開発計画・進捗 |
| [バージョン履歴](./docs/changelog.md) | リリースごとの変更内容 |
| [参考資料](./docs/references.md) | ハングル・Windows TSF・Rust IME 関連リンク |

## ライセンス

CC0 1.0 Universal
