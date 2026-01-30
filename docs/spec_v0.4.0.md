# Chamsae IME 仕様書 - Phase 4: リアルタイム変換・設定

## 概要

Phase 3のTSF IME基盤をベースに、IME ON/OFFトグル、非対応キーの自動確定、
連音化対応、設定ファイルによるカスタマイズ機能を追加する。

### 目標

- IME ON/OFF トグルキー (デフォルト: Shift+Space)
- 非対応キー入力時のコンポジション自動確定
- Ctrl/Alt押下中のパススルー (自動確定付き)
- 設定ファイル (`chamsae.json`) によるトグルキー変更
- 言語プロファイル登録の設定 (日本語/韓国語)

### 対象環境

- OS: Windows 11 (優先), Linux (クロスコンパイル・テスト)
- 言語: Rust
- 主要依存追加: `serde` 1.x, `serde_json` 1.x

---

## 設定ファイル

### 仕様

**パス**: `<DLLと同じディレクトリ>/chamsae.json`

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
  }
}
```

### 読み込み動作

| 状態 | 動作 |
|------|------|
| ファイルなし | デフォルト設定で新規作成し、デフォルト値を使用 |
| パース成功 | 設定値を使用 |
| パース失敗 (不正JSON) | デフォルト値にフォールバック |
| キー名不正 | デフォルト値にフォールバック |
| `languages` フィールドなし | デフォルト値 (japanese: true, korean: false) を使用 |

### toggle_key

IME ON/OFF 切り替えキーの設定。

#### key フィールド

| 値 | 仮想キーコード | 説明 |
|-----|---------------|------|
| `"A"`〜`"Z"` | 0x41〜0x5A | アルファベットキー |
| `"0"`〜`"9"` | 0x30〜0x39 | 数字キー |
| `"Space"` | 0x20 | スペースキー |

#### 修飾キー

| フィールド | 型 | デフォルト | 説明 |
|-----------|-----|----------|------|
| `shift` | bool | `true` | Shiftキー同時押し |
| `ctrl` | bool | `false` | Ctrlキー同時押し |
| `alt` | bool | `false` | Altキー同時押し |

#### 設定例

Alt+S でトグルする場合:
```json
{
  "toggle_key": {
    "key": "S",
    "shift": false,
    "ctrl": false,
    "alt": true
  }
}
```

### languages

`regsvr32` によるDLL登録時にどの言語プロファイルを登録するかを制御する。

| フィールド | 型 | デフォルト | 説明 |
|-----------|-----|----------|------|
| `japanese` | bool | `true` | 日本語キーボードプロファイルとして登録 |
| `korean` | bool | `false` | 韓国語キーボードプロファイルとして登録 |

設定変更後は `regsvr32 /u chamsae.dll` で登録解除してから `regsvr32 chamsae.dll` で再登録する。

---

## IME ON/OFF トグル

### 動作

- トグルキー押下で IME の有効/無効を切り替える
- IME無効時はすべてのキーをパススルー (アプリケーションに直接渡す)
- コンポジション中にトグルすると、まず確定してからOFF

### 判定ロジック

```
is_toggle_key(vk):
  1. vk == config.toggle_key.vk ?
  2. Shift状態 == config.toggle_key.shift ?
  3. Ctrl状態 == config.toggle_key.ctrl ?
  4. Alt状態 == config.toggle_key.alt ?
  すべて一致 → true
```

### キーイベントフロー

```
OnTestKeyDown:
  is_toggle_key? → TRUE (横取り)
  !enabled? → FALSE (パススルー)
  modifier_held? → バッファ非空なら TRUE
  is_hangul_key? → TRUE
  is_control_key && バッファ非空? → TRUE
  バッファ非空? → TRUE
  → FALSE

OnKeyDown:
  is_toggle_key? → 確定 (バッファ非空時) → toggle → TRUE
  !enabled? → FALSE
  modifier_held? → 確定 (バッファ非空時) → FALSE
  (以降Phase 3と同じ)
```

### Ctrl/Alt修飾キー

コンポジション中にCtrlまたはAltが押された場合:
1. バッファ非空なら自動確定
2. キーイベントをパススルー (ショートカットキーが効く)

---

## TSF登録

### プロファイル登録フロー

```
DllRegisterServer
  → register_server()
      → Config::load_from_dll()  ← 設定ファイル読み込み
      → register_tsf_profile(&config)
          → profiles.Register(CLSID)
          → config.languages.korean? → RegisterProfile(LANGID_KOREAN, GUID_CHAMSAE_PROFILE)
          → config.languages.japanese? → RegisterProfile(LANGID_JAPANESE, GUID_CHAMSAE_PROFILE_JA)
          → category_mgr.RegisterCategory(GUID_TFCAT_TIP_KEYBOARD)
```

### GUID一覧

| 名前 | 値 | 用途 |
|------|-----|------|
| `CLSID_CHAMSAE_TEXT_SERVICE` | `{D4A5B8E1-7C2F-4A3D-9E6B-1F8C0D2A5E7B}` | COMクラスID |
| `GUID_CHAMSAE_PROFILE` | `{A2C4E6F8-1B3D-5A7C-9E0F-2D4B6A8C0E1F}` | 韓国語言語プロファイル |
| `GUID_CHAMSAE_PROFILE_JA` | `{B3D5F7A9-2C4E-6B8D-AF10-3E5C7B9D1F20}` | 日本語言語プロファイル |
| `LANGID_KOREAN` | `0x0412` | 韓国語言語ID |
| `LANGID_JAPANESE` | `0x0411` | 日本語言語ID |

---

## ファイル構成

### 新規ファイル

| ファイル | 内容 |
|---------|------|
| `src/config.rs` | 設定ファイル読み込みモジュール |
| `spec_v0.4.0.md` | 本仕様書 |

### 変更ファイル

| ファイル | 変更内容 |
|---------|---------|
| `Cargo.toml` | `serde`, `serde_json` 依存追加、`tempfile` dev依存追加 |
| `src/lib.rs` | `pub mod config` 追加 |
| `src/tsf/text_service.rs` | `Config` フィールド追加、`is_toggle_key()` を設定値参照に変更 |
| `src/tsf/registration.rs` | `Config` 引数追加、言語プロファイル条件付き登録 |
| `src/registry.rs` | `Config::load_from_dll()` 呼び出し追加 |

---

## config.rs 設計

### 構造体

```rust
pub struct ToggleKey {
    pub vk: u32,       // 仮想キーコード
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

pub struct Languages {
    pub japanese: bool,
    pub korean: bool,
}

pub struct Config {
    pub toggle_key: ToggleKey,
    pub languages: Languages,
}
```

### JSON用構造体

```rust
#[derive(Serialize, Deserialize)]
struct ToggleKeyJson {
    key: String,
    shift: bool, ctrl: bool, alt: bool,
}

#[derive(Serialize, Deserialize)]
struct LanguagesJson {
    japanese: bool,
    korean: bool,
}

#[derive(Serialize, Deserialize)]
struct ConfigJson {
    toggle_key: ToggleKeyJson,
    #[serde(default)]           // 旧設定ファイル互換
    languages: LanguagesJson,
}
```

`languages` に `#[serde(default)]` を付与することで、
`languages` フィールドのない旧設定ファイルでもパースが成功する。

### 主要関数

| 関数 | 説明 |
|------|------|
| `Config::default()` | デフォルト設定 (Shift+Space, 日本語有効, 韓国語無効) |
| `Config::load(dll_dir)` | 設定ファイル読み込み (なければ新規作成) |
| `Config::load_from_dll()` | DLLパスから設定読み込み (Windows専用) |
| `key_name_to_vk(name)` | キー名 → 仮想キーコード変換 |
| `get_dll_directory()` | DLLモジュールの親ディレクトリ取得 (Windows専用) |

---

## テスト

### 追加テスト (config.rs)

| テスト名 | 内容 |
|---------|------|
| `test_default_config` | デフォルト値 (VK, 修飾キー, 言語) の確認 |
| `test_key_name_to_vk_alphabet` | A-Z → 0x41-0x5A 変換 |
| `test_key_name_to_vk_digits` | 0-9 → 0x30-0x39 変換 |
| `test_key_name_to_vk_space` | "Space" → 0x20 変換 |
| `test_key_name_to_vk_invalid` | 不正キー名 → None |
| `test_load_valid_json` | 正常JSON読み込み (トグルキー + 言語設定) |
| `test_load_json_without_languages_uses_default` | `languages` なしJSON → デフォルト言語設定 |
| `test_load_missing_file_creates_default` | ファイル新規作成 + デフォルト値確認 |
| `test_load_invalid_json_fallback` | 不正JSON → デフォルトフォールバック |
| `test_load_invalid_key_name_fallback` | 不正キー名 → デフォルトフォールバック |

---

## 依存関係

### 追加依存

| クレート | バージョン | 用途 |
|---------|-----------|------|
| `serde` | 1.x (features: derive) | JSON構造体の直列化/逆直列化 |
| `serde_json` | 1.x | JSONファイルの読み書き |
| `tempfile` | 3.x (dev) | テストでの一時ディレクトリ |
