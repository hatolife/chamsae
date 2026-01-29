# Chamsae - Hangul IME (ハングルIME)

ローマ字入力から韓国語ハングル文字への変換を行うIME (Input Method Editor)。

Rust学習を兼ねた自作IMEプロジェクト。

## 現在のステータス

| 項目 | 状態 |
|------|------|
| バージョン | v0.3.0 (開発中) |
| フェーズ | Phase 3 完了 |
| テスト | 47テスト通過 |
| 対応OS | CLI: Windows/Linux, DLL/IME: Windows |

## クイックスタート

```bash
# ビルド
cargo build --release

# 単一変換
./target/release/chamsae -i "an nyeong ha se yo"
# 出力: 안녕하세요

# インタラクティブモード
./target/release/chamsae -I
> han gug eo
  → 한국어
> exit
```

## 入力規則

| 入力 | 意味 | 例 |
|------|------|-----|
| 半角スペース1つ | 音節区切り | `han gug` → 한국 |
| 半角スペース2つ | 実際のスペース | `an nyeong  ha se yo` → 안녕 하세요 |

詳細は [spec_v0.1.0.md](./spec_v0.1.0.md) / [spec_v0.2.0.md](./spec_v0.2.0.md) / [spec_v0.3.0.md](./spec_v0.3.0.md) を参照。

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

### Phase 4: リアルタイム変換 🔜 次

| マイルストーン | 内容 | 予定 |
|---------------|------|------|
| 4.1 | 終声の自動移動 (連音化) | - |
| 4.2 | IME ON/OFF トグル | - |
| 4.3 | 非対応キー入力時の自動確定 | - |
| 4.4 | エッジケース修正 | - |

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
| v0.3.0 | 2026-01-30 | TSF IME実装, キーイベント処理, コンポジション |
| v0.2.0 | 2026-01-29 | Windows DLL構造, COM基礎, Win32ウィンドウ |
| v0.0.1 | 2025-01-30 | 変換エンジン完成、CLIツール |

### 予定バージョン

| バージョン | 目標 | 主な機能 |
|-----------|------|----------|
| v0.4.0 | Phase 4完了 | リアルタイム変換, IMEトグル |
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
chamsae/
├── Cargo.toml
├── makefile
├── readme.md              # 本ファイル
├── spec_v0.1.0.md         # 仕様書 (Phase 1)
├── spec_v0.2.0.md         # 仕様書 (Phase 2)
├── spec_v0.3.0.md         # 仕様書 (Phase 3)
└── src/
    ├── lib.rs             # ライブラリルート + DLLエクスポート
    ├── hangul.rs          # 変換ロジック + テスト
    ├── guid.rs            # GUID/CLSID定義
    ├── registry.rs        # レジストリ登録 + TSF登録
    ├── com/
    │   ├── mod.rs         # COMモジュール
    │   ├── class_factory.rs   # IClassFactory実装
    │   └── dll_module.rs      # DLLモジュール管理
    ├── tsf/
    │   ├── mod.rs         # TSFモジュール
    │   ├── text_service.rs    # TextService (ITfTextInputProcessorEx)
    │   ├── key_handler.rs     # キーイベント判定
    │   ├── edit_session.rs    # EditSession (コンポジション操作)
    │   └── registration.rs    # TSFプロファイル・カテゴリ登録
    ├── win32/
    │   ├── mod.rs         # Win32モジュール
    │   └── window.rs      # 基本ウィンドウ作成
    └── bin/
        ├── cli.rs         # CLIツール
        └── window_test.rs # ウィンドウテスト
```

---

## 技術スタック

| 用途 | ライブラリ |
|------|-----------|
| 引数解析 | clap |
| エラー処理 | anyhow |
| Windows API | windows-rs 0.58 |
| テスト | 標準 (cargo test) |

---

## IME登録 (Windows)

DLLをWindowsに登録してIMEとして使用する手順。

`regsvr32` を実行すると以下が行われる:
1. CLSID/InprocServer32 のレジストリ登録
2. TSFプロファイル登録 (韓国語キーボードとして登録)
3. TSFカテゴリ登録 (キーボードTIPとして登録)

### ビルド

```bat
cargo build --release --target x86_64-pc-windows-gnu
```

### 登録・解除

**管理者権限のコマンドプロンプト**で実行してください。

```bat
REM DLLの登録 (IMEとしてシステムに追加)
regsvr32 target\x86_64-pc-windows-gnu\release\chamsae.dll

REM DLLの登録解除
regsvr32 /u target\x86_64-pc-windows-gnu\release\chamsae.dll
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

登録後、Windowsの「設定 > 時刻と言語 > 言語と地域」で韓国語キーボードとして「Chamsae Hangul IME」が表示される。

### 現在の制限

- IME ON/OFF のトグル機能がないため、登録すると常にローマ字→ハングル変換が有効になる
- 候補ウィンドウは未実装 (コンポジション下線のみ)
- a-z 以外のキー入力中にコンポジションの自動確定が行われない

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
