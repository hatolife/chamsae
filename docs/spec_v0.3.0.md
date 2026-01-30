# Chamsae IME 仕様書 - Phase 3: TSF IME実装

## 概要

Phase 2のCOM基盤をベースに、TSF (Text Services Framework) のIMEとして動作する実装を行う。
ローマ字キー入力をリアルタイムでハングルに変換し、コンポジション文字列として表示する。

### 目標

- TSF TextServiceとしてWindowsに登録・動作
- キーイベントの受信とローマ字→ハングル変換
- EditSessionによるコンポジション操作
- TSFプロファイル・カテゴリ登録

### 対象環境

- OS: Windows 11 (優先), Linux (クロスコンパイル)
- 言語: Rust
- ターゲット: `x86_64-pc-windows-gnu`
- 主要依存: `windows` 0.58, `windows-core` 0.58

---

## アーキテクチャ

### 全体フロー

```
キー入力 (a-z)
    │
    ▼
OnTestKeyDown ─── TRUE (処理する) / FALSE (スキップ)
    │
    ▼
OnKeyDown
    ├── ローマ字バッファに蓄積
    ├── HangulConverter::convert() でハングル変換
    └── EditSession を RequestEditSession で要求
            │
            ▼
        DoEditSession (TSFが edit cookie を付与)
            ├── コンポジション開始 (StartComposition)
            ├── テキスト更新 (SetText)
            └── コンポジション終了 (EndComposition)
```

### コンポーネント構成

```
TextService (ITfTextInputProcessorEx + ITfKeyEventSink + ITfCompositionSink)
    ├── roman_buffer: RefCell<String>          # ローマ字入力バッファ
    ├── composition: Arc<Mutex<Option<ITfComposition>>>  # アクティブなコンポジション
    ├── converter: HangulConverter             # 変換エンジン (Phase 1)
    ├── thread_mgr: RefCell<Option<ITfThreadMgr>>       # TSFスレッドマネージャ
    └── client_id: Cell<u32>                   # TSFクライアントID

EditSession (ITfEditSession)
    ├── context: ITfContext                    # テキストコンテキスト
    ├── composition: Arc<Mutex<...>>           # TextServiceと共有
    ├── composition_sink: ITfCompositionSink   # コンポジション終了通知先
    └── action: EditAction                     # 操作種別

Registration
    ├── ITfInputProcessorProfiles              # プロファイル登録
    └── ITfCategoryMgr                         # カテゴリ登録
```

---

## TextService

### COMインターフェース

`#[implement]` マクロで3つのインターフェースを実装する。

| インターフェース | 継承元 | 役割 |
|-----------------|--------|------|
| `ITfTextInputProcessorEx` | `ITfTextInputProcessor` | TSFへの登録/解除 |
| `ITfKeyEventSink` | `IUnknown` | キーイベント処理 |
| `ITfCompositionSink` | `IUnknown` | コンポジション終了通知 |

### ライフサイクル

```
DllGetClassObject
    └── ClassFactory::CreateInstance
            └── TextService::new()
                    │
                    ▼
            ActivateEx (TSFがアクティベート)
                ├── thread_mgr, client_id を保存
                └── AdviseKeyEventSink (キーイベント受信開始)
                        │
                        ▼
                キー入力処理 (OnTestKeyDown → OnKeyDown)
                        │
                        ▼
            Deactivate (TSFがディアクティベート)
                ├── UnadviseKeyEventSink
                └── 状態クリア
```

### ITfTextInputProcessor

| メソッド | 実装 |
|----------|------|
| `Activate` | `ActivateEx` に委譲 |
| `Deactivate` | キーシンク解除、状態クリア |

### ITfTextInputProcessorEx

| メソッド | 実装 |
|----------|------|
| `ActivateEx` | thread_mgr/client_id保存、AdviseKeyEventSink |

### ITfKeyEventSink

| メソッド | 実装 |
|----------|------|
| `OnSetFocus` | 何もしない |
| `OnTestKeyDown` | a-z → TRUE、制御キー (バッファ非空時) → TRUE |
| `OnTestKeyUp` | FALSE (処理しない) |
| `OnKeyDown` | バッファ蓄積 + コンポジション更新/確定/キャンセル |
| `OnKeyUp` | FALSE |
| `OnPreservedKey` | FALSE |

### ITfCompositionSink

| メソッド | 実装 |
|----------|------|
| `OnCompositionTerminated` | バッファクリア、コンポジション参照解放 |

---

## キーイベント処理

### 処理対象キー

| キー | VK | バッファ空 | バッファ非空 |
|------|-----|-----------|-------------|
| a-z | 0x41-0x5A | 処理する (コンポジション開始) | 処理する (バッファ追加) |
| Backspace | 0x08 | スキップ | 処理する (1文字削除) |
| Enter | 0x0D | スキップ | 処理する (確定) |
| Escape | 0x1B | スキップ | 処理する (キャンセル) |
| Space | 0x20 | スキップ | 処理する (音節区切り) |
| その他 | - | スキップ | スキップ |

### OnKeyDown処理フロー

```
vk_to_char(vk) → Some(ch)?
    ├── YES: buffer.push(ch) → update_composition()
    └── NO: buffer.is_empty()?
            ├── YES: return FALSE
            └── NO: match vk
                    ├── VK_BACK → buffer.pop()
                    │       └── buffer.empty? → Cancel : Update
                    ├── VK_RETURN → Commit → buffer.clear()
                    ├── VK_ESCAPE → Cancel → buffer.clear()
                    └── VK_SPACE → buffer.push(' ') → Update
```

### 仮想キーコード変換

```
VK_A (0x41) → 'a'
VK_B (0x42) → 'b'
...
VK_Z (0x5A) → 'z'
```

大文字/小文字の区別はしない (常に小文字に変換)。

---

## EditSession

### 概要

TSFではテキストの変更は必ずEditSessionを通じて行う。
`RequestEditSession` でセッションを要求し、TSFがedit cookie (ec) を付与して
`DoEditSession` を呼び出す。

### EditAction

| アクション | 動作 |
|-----------|------|
| `Update(Vec<u16>)` | コンポジション開始 (未開始時) + テキスト更新 |
| `Commit` | コンポジション確定 (テキストはそのまま残る) |
| `Cancel` | コンポジションキャンセル (テキスト削除) |

### コンポジション操作

#### 開始 (Update時、コンポジション未開始)

```
1. ITfInsertAtSelection::InsertTextAtSelection(ec, TF_IAS_QUERYONLY, &[])
   → 挿入位置の ITfRange を取得
2. ITfContextComposition::StartComposition(ec, &range, &composition_sink)
   → ITfComposition を取得して Arc<Mutex> に保存
```

#### テキスト更新

```
1. ITfComposition::GetRange() → コンポジション範囲を取得
2. ITfRange::SetText(ec, 0, &utf16_text) → テキスト設定
```

#### 確定 (Commit)

```
1. ITfComposition::EndComposition(ec) → コンポジション終了
   (テキストはドキュメントにそのまま残る)
```

#### キャンセル (Cancel)

```
1. ITfRange::SetText(ec, 0, &[]) → テキストを空に
2. ITfComposition::EndComposition(ec) → コンポジション終了
```

### 共有状態

TextServiceとEditSessionは `Arc<Mutex<Option<ITfComposition>>>` でコンポジション状態を共有する。

- **TextService**: コンポジション生存管理、OnCompositionTerminatedでのクリア
- **EditSession**: コンポジション開始/更新/終了の実操作

STA環境のためMutexが実際に競合することはないが、`#[implement]` マクロが
Sendを要求するため `Arc<Mutex>` を使用。

### RequestEditSession

```rust
context.RequestEditSession(
    client_id,
    &session,
    TF_ES_SYNC | TF_ES_READWRITE,  // 同期 + 読み書き
)
```

`TF_ES_SYNC` により `DoEditSession` は `RequestEditSession` の呼び出し内で
同期的に実行される。

---

## TSF登録

### プロファイル登録

`DllRegisterServer` → `register_server()` → `register_tsf_profile()` の順で呼ばれる。

#### 手順

```
1. CoCreateInstance(CLSID_TF_InputProcessorProfiles) → ITfInputProcessorProfiles
2. profiles.Register(&CLSID_CHAMSAE_TEXT_SERVICE)
3. profiles.AddLanguageProfile(
       CLSID_CHAMSAE_TEXT_SERVICE,
       LANGID_KOREAN (0x0412),
       GUID_CHAMSAE_PROFILE,
       "Chamsae Hangul IME",  // 表示名
       "",                     // アイコンファイル (なし)
       0,                      // アイコンインデックス
   )
4. CoCreateInstance(CLSID_TF_CategoryMgr) → ITfCategoryMgr
5. category_mgr.RegisterCategory(
       CLSID_CHAMSAE_TEXT_SERVICE,
       GUID_TFCAT_TIP_KEYBOARD,      // キーボードTIPカテゴリ
       CLSID_CHAMSAE_TEXT_SERVICE,
   )
```

### プロファイル解除

`DllUnregisterServer` → `unregister_server()` の順で呼ばれる。

```
1. カテゴリ解除: UnregisterCategory(GUID_TFCAT_TIP_KEYBOARD)
2. プロファイル解除: Unregister(CLSID_CHAMSAE_TEXT_SERVICE)
3. レジストリ削除: CLSID キー削除 (Phase 2と同じ)
```

### GUID一覧

| 名前 | 値 | 用途 |
|------|-----|------|
| `CLSID_CHAMSAE_TEXT_SERVICE` | `{D4A5B8E1-...5E7B}` | COMクラスID |
| `GUID_CHAMSAE_PROFILE` | `{A2C4E6F8-...0E1F}` | 言語プロファイル |
| `LANGID_KOREAN` | `0x0412` | 韓国語言語ID |

---

## ClassFactory更新

Phase 2ではスタブ (E_NOINTERFACE) だった `CreateInstance` を実装。

```
CreateInstance:
  1. punkouter チェック (アグリゲーション非対応)
  2. TextService::new() でオブジェクト作成
  3. IUnknown に変換
  4. QueryInterface で要求されたインターフェースを返す
```

---

## ファイル構成

```
src/tsf/
├── mod.rs              # TSFモジュールルート
├── text_service.rs     # TextService (COMオブジェクト本体)
├── key_handler.rs      # 仮想キーコード判定・変換
├── edit_session.rs     # EditSession (コンポジション操作)
└── registration.rs     # TSFプロファイル・カテゴリ登録
```

### 変更ファイル

| ファイル | 変更内容 |
|----------|---------|
| `Cargo.toml` | `Win32_UI_TextServices` フィーチャー追加 |
| `src/lib.rs` | `pub mod tsf` 追加 |
| `src/guid.rs` | `LANGID_KOREAN` 追加 |
| `src/com/class_factory.rs` | CreateInstance で TextService 生成 |
| `src/registry.rs` | TSFプロファイル登録/解除の呼び出し追加 |

---

## windows クレート features

Phase 2に加えて追加したfeature:

| feature | 用途 |
|---------|------|
| `Win32_UI_TextServices` | TSFインターフェース (ITfTextInputProcessorEx, ITfKeyEventSink, ITfCompositionSink, ITfEditSession, ITfKeystrokeMgr, ITfContextComposition, ITfInsertAtSelection, ITfRange等) |

---

## 制限事項

### Phase 3の制限

1. **IME ON/OFF**: トグル機能がなく、登録すると常にハングル変換が有効
2. **候補ウィンドウ**: 未実装。コンポジション下線のみ表示
3. **自動確定**: a-z 以外のキー入力時にコンポジションが自動確定されない
4. **連音化**: 終声の次の音節への自動移動は未対応 (スペース区切りで明示的に分離)

### Phase 4への引き継ぎ

1. IME ON/OFF トグルキー (Shift+Space 等) の実装
2. 非対応キー入力時のコンポジション自動確定
3. 終声の自動移動 (連音化) 対応
4. コンポジション中の視覚フィードバック改善
