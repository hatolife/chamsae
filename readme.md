# Chamsae - Hangul IME (ハングルIME)

ローマ字入力から韓国語ハングル文字への変換を行うIME (Input Method Editor)。

Rust学習を兼ねた自作IMEプロジェクト。

## 現在のステータス

| 項目 | 状態 |
|------|------|
| バージョン | v0.1.0 (開発中) |
| フェーズ | Phase 1 完了 |
| テスト | 47テスト通過 |
| 対応OS | CLI: Windows/Linux, IME: 未実装 |

## クイックスタート

```bash
# ビルド
cargo build --release

# 単一変換
./target/release/hangul_ime -i "an nyeong ha se yo"
# 出力: 안녕하세요

# インタラクティブモード
./target/release/hangul_ime -I
> han gug eo
  → 한국어
> exit
```

## 入力規則

| 入力 | 意味 | 例 |
|------|------|-----|
| 半角スペース1つ | 音節区切り | `han gug` → 한국 |
| 半角スペース2つ | 実際のスペース | `an nyeong  ha se yo` → 안녕 하세요 |

詳細は [spec_v0.1.0.md](./spec_v0.1.0.md) を参照。

---

## ロードマップ

### Phase 1: 変換エンジン ✅ 完了

| 内容 | 状態 |
|------|------|
| Rust環境構築・基礎学習 | ✅ |
| CLIツール (clap, anyhow) | ✅ |
| ハングル変換ロジック | ✅ |

### Phase 2: Windows API入門 🔜 次

| マイルストーン | 内容 | 予定 |
|---------------|------|------|
| 2.1 | windows-rs セットアップ | Week 1 |
| 2.2 | Win32 基本ウィンドウ作成 | Week 1 |
| 2.3 | COM基礎 (IUnknown, ClassFactory) | Week 2 |
| 2.4 | DLL作成・レジストリ登録 | Week 2 |

**目標**: Windowsアプリケーション開発の基礎を習得

### Phase 3: TSF IME実装

| マイルストーン | 内容 | 予定 |
|---------------|------|------|
| 3.1 | TSF最小スケルトン | Week 3 |
| 3.2 | ITfTextInputProcessorEx実装 | Week 3-4 |
| 3.3 | キーイベント処理 | Week 4 |
| 3.4 | 変換ロジック統合 | Week 5 |
| 3.5 | 候補ウィンドウ (基本) | Week 5-6 |

**目標**: システムIMEとして動作する最小実装

### Phase 4: リアルタイム変換

| マイルストーン | 内容 | 予定 |
|---------------|------|------|
| 4.1 | キー入力単位の処理 | Week 7 |
| 4.2 | 終声の自動移動 (連音化) | Week 7 |
| 4.3 | Composition文字列表示 | Week 8 |
| 4.4 | バックスペース処理 | Week 8 |

**目標**: 韓国IMEと同様のリアルタイム再構成

### Phase 5: 改善・拡張

| マイルストーン | 内容 | 優先度 |
|---------------|------|--------|
| 5.1 | 設定画面 | 中 |
| 5.2 | トレイアイコン | 中 |
| 5.3 | インストーラー作成 | 高 |
| 5.4 | 変換候補表示 | 低 |
| 5.5 | ユーザー辞書 | 低 |

---

## バージョン履歴

| バージョン | 日付 | 内容 |
|-----------|------|------|
| v0.0.1 | 2025-01-30 | 変換エンジン完成、CLIツール |

### 予定バージョン

| バージョン | 目標 | 主な機能 |
|-----------|------|----------|
| v0.1.0 | Phase 2完了 | Windows DLL作成 |
| v0.2.0 | Phase 3完了 | TSF IME動作 |
| v0.3.0 | Phase 4完了 | リアルタイム変換 |
| v1.0.0 | Phase 5完了 | 実用レベル |

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

# ビルド
cargo build

# テスト
cargo test

# リンター
cargo clippy
```

### ディレクトリ構成

```
hangul_ime/
├── Cargo.toml
├── Makefile
├── README.md           # 本ファイル
├── spec_v0.0.1.md      # 仕様書
└── src/
    ├── main.rs         # CLIエントリポイント
    └── hangul.rs       # 変換ロジック + テスト
```

### 将来の構成 (Phase 3以降)

```
hangul_ime/
├── Cargo.toml
├── README.md
├── spec_v*.md
├── src/
│   ├── lib.rs          # DLLエントリポイント
│   ├── hangul.rs       # 変換ロジック
│   ├── tsf/
│   │   ├── mod.rs
│   │   ├── text_service.rs
│   │   ├── key_handler.rs
│   │   └── composition.rs
│   └── bin/
│       └── cli.rs      # CLIツール (開発用)
└── installer/
    └── setup.iss       # Inno Setup スクリプト
```

---

## 技術スタック

| 用途 | ライブラリ |
|------|-----------|
| 引数解析 | clap |
| エラー処理 | anyhow |
| Windows API | windows-rs (Phase 2〜) |
| テスト | 標準 (cargo test) |

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

MIT License (予定)

---

## 作者メモ

このプロジェクトはRust学習を目的としています。
実用IMEとしての完成度より、学習過程を重視しています。
