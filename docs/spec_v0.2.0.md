# Chamsae IME 仕様書 - Phase 2: Windows API入門

## 概要

Phase 1の変換エンジンをWindows DLLとして構成し、COM基礎とWin32ウィンドウ作成を実装する。

### 目標

- windows-rs クレートによるWindows API呼び出し
- COM基礎 (IUnknown, IClassFactory) の理解と実装
- Win32ウィンドウ作成の基礎習得
- DLL作成とレジストリ登録

### 対象環境

- OS: Windows 11 (優先), Linux (クロスコンパイル)
- 言語: Rust
- ターゲット: `x86_64-pc-windows-gnu`
- 主要依存: `windows` 0.58, `windows-core` 0.58

---

## DLL構造

### エントリポイント

DLLは以下の4つの標準COMエクスポート関数を公開する。

| 関数 | 役割 |
|------|------|
| `DllMain` | DLLロード/アンロード時の初期化・終了処理 |
| `DllGetClassObject` | COMクラスファクトリの提供 |
| `DllCanUnloadNow` | DLLアンロード可否の判定 |
| `DllRegisterServer` | レジストリへのCLSID登録 |
| `DllUnregisterServer` | レジストリからのCLSID削除 |

### DllMain処理

```
DLL_PROCESS_ATTACH:
  1. モジュールハンドル (HMODULE) を保存
  2. DisableThreadLibraryCalls でスレッド通知を無効化
```

### クレートタイプ

```toml
[lib]
name = "chamsae"
crate-type = ["cdylib", "rlib"]
```

- `cdylib`: Windows DLL (.dll) として出力
- `rlib`: CLIバイナリからライブラリとして参照

---

## GUID定義

COMオブジェクトとTSFプロファイルの識別に使用するGUID。

| 名前 | 値 | 用途 |
|------|-----|------|
| `CLSID_CHAMSAE_TEXT_SERVICE` | `{D4A5B8E1-7C2F-4A3D-9E6B-1F8C0D2A5E7B}` | COMクラスID |
| `GUID_CHAMSAE_PROFILE` | `{A2C4E6F8-1B3D-5A7C-9E0F-2D4B6A8C0E1F}` | TSF言語プロファイル (Phase 3用) |

### 定数

| 名前 | 値 |
|------|-----|
| `IME_DISPLAY_NAME` | `"Chamsae Hangul IME"` |
| `IME_DESCRIPTION` | `"Chamsae - Romanji to Hangul Input Method Editor"` |

---

## COM基礎

### IClassFactory実装

COMクライアントがオブジェクトを作成するためのファクトリパターン。

```
CoCreateInstance呼び出しフロー:
1. クライアントが CoCreateInstance を呼ぶ
2. COMランタイムが DllGetClassObject を呼ぶ
3. DllGetClassObject が ClassFactory を返す
4. COMランタイムが IClassFactory::CreateInstance を呼ぶ
5. ClassFactory が TextService オブジェクトを作成して返す
```

#### メソッド

| メソッド | 実装状態 | 説明 |
|----------|---------|------|
| `CreateInstance` | スタブ (E_NOINTERFACE) | Phase 3でTextServiceを返す |
| `LockServer` | 実装済み | サーバーロックカウントの増減 |

### DLLモジュール管理

グローバル状態をAtomicで安全に管理する。

| 状態 | 型 | 用途 |
|------|-----|------|
| `MODULE_HANDLE` | `AtomicIsize` | DLLファイルパス取得用 |
| `OBJECT_COUNT` | `AtomicU32` | COMオブジェクト生存数 |
| `SERVER_LOCK_COUNT` | `AtomicU32` | IClassFactory::LockServerによるロック数 |

#### アンロード判定

```
DllCanUnloadNow:
  OBJECT_COUNT == 0 かつ SERVER_LOCK_COUNT == 0 → S_OK (アンロード可)
  それ以外 → S_FALSE (アンロード不可)
```

---

## レジストリ登録

### レジストリ構成

```
HKEY_CLASSES_ROOT
└── CLSID
    └── {D4A5B8E1-7C2F-4A3D-9E6B-1F8C0D2A5E7B}
        ├── (Default) = "Chamsae Hangul IME"
        └── InprocServer32
            ├── (Default) = "C:\path\to\chamsae.dll"
            └── ThreadingModel = "Apartment"
```

### 登録/解除

```bat
REM 登録
regsvr32 chamsae.dll

REM 登録解除
regsvr32 /u chamsae.dll
```

### 使用API

| API | 用途 |
|-----|------|
| `RegCreateKeyExW` | レジストリキーの作成/オープン |
| `RegSetValueExW` | レジストリ値の設定 |
| `RegDeleteTreeW` | レジストリキーの再帰削除 |
| `GetModuleFileNameW` | DLLの完全パス取得 |

---

## Win32ウィンドウ

### ウィンドウ作成フロー

```
1. WNDCLASSW 構造体でウィンドウクラスを定義
2. RegisterClassW でクラスを登録
3. CreateWindowExW でウィンドウを作成
4. ShowWindow で表示
5. メッセージループ (GetMessage → TranslateMessage → DispatchMessage)
6. WM_DESTROY で PostQuitMessage を呼び終了
```

### ウィンドウクラス

| プロパティ | 値 |
|-----------|-----|
| クラス名 | `ChamsaeWindowClass` |
| タイトル | `Chamsae IME Test Window` |
| サイズ | 640 x 480 |
| スタイル | `WS_OVERLAPPEDWINDOW` |
| 背景 | `WHITE_BRUSH` |
| カーソル | `IDC_ARROW` |

### メッセージ処理

| メッセージ | 処理 |
|-----------|------|
| `WM_DESTROY` | `PostQuitMessage(0)` でループ終了 |
| その他 | `DefWindowProcW` に委譲 |

---

## ファイル構成

```
chamsae/
├── Cargo.toml
├── makefile
├── readme.md
├── spec_v0.1.0.md       # Phase 1 仕様書
├── spec_v0.2.0.md       # 本仕様書
└── src/
    ├── lib.rs            # ライブラリルート + DLLエクスポート
    ├── hangul.rs         # 変換ロジック + テスト
    ├── guid.rs           # GUID/CLSID定義
    ├── registry.rs       # レジストリ登録
    ├── com/
    │   ├── mod.rs        # COMモジュール
    │   ├── class_factory.rs  # IClassFactory実装
    │   └── dll_module.rs     # DLLモジュール管理
    ├── win32/
    │   ├── mod.rs        # Win32モジュール
    │   └── window.rs     # 基本ウィンドウ作成
    └── bin/
        ├── cli.rs        # CLIツール
        └── window_test.rs    # ウィンドウテスト
```

---

## ビルド生成物

| 種別 | ファイル名 | 説明 |
|------|-----------|------|
| DLLライブラリ | `chamsae.dll` | COM DLL (cdylib) |
| CLIバイナリ | `chamsae.exe` | ローマ字→ハングル変換CLI |
| テストバイナリ | `chamsae_window_test.exe` | Win32ウィンドウ動作確認 |

### ビルドコマンド

```bash
# Windowsターゲット (クロスコンパイル)
cargo build --target x86_64-pc-windows-gnu

# テスト (ホスト環境)
cargo test
```

---

## 技術スタック

| 用途 | ライブラリ/バージョン |
|------|---------------------|
| 引数解析 | clap 4 |
| エラー処理 | anyhow 1 |
| Windows API | windows 0.58 |
| Windows Core | windows-core 0.58 |
| テスト | 標準 (cargo test) |

### windows クレート features

| feature | 用途 |
|---------|------|
| `implement` | COM `#[implement]` マクロ |
| `Win32_Foundation` | 基本型 (BOOL, HMODULE, HRESULT等) |
| `Win32_Security` | `RegCreateKeyExW` のセキュリティ属性 |
| `Win32_System_Com` | IClassFactory等COMインターフェース |
| `Win32_System_LibraryLoader` | GetModuleFileNameW, DisableThreadLibraryCalls |
| `Win32_System_Registry` | レジストリ操作API |
| `Win32_UI_WindowsAndMessaging` | ウィンドウ作成・メッセージ処理 |
| `Win32_Graphics_Gdi` | GDIオブジェクト (ブラシ等) |

---

## 制限事項

### Phase 2の制限

1. **ClassFactory::CreateInstance**: TextService未実装のため E_NOINTERFACE を返す
2. **ウィンドウ**: テスト用のみ。IME候補ウィンドウとしては未使用
3. **レジストリ登録**: CLSID登録のみ。TSF言語プロファイル登録は Phase 3

### Phase 3への引き継ぎ

1. `ClassFactory::CreateInstance` で `TextService` オブジェクトを返す
2. `ITfTextInputProcessorEx` インターフェースの実装
3. TSF言語プロファイルのレジストリ登録 (`GUID_CHAMSAE_PROFILE` 使用)
4. キーイベント処理と変換ロジックの統合
