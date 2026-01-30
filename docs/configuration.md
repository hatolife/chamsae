# 設定・登録ガイド

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

## 設定ファイル

`%APPDATA%\Chamsae\chamsae.json` に配置される (初回起動時に自動作成)。
設定GUIまたは手動でファイルを編集後、トレイメニューの「設定の再読み込み」で即時反映できる。

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

### toggle_key

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

### languages

`regsvr32` 実行時にどの言語プロファイルを登録するかを制御する。
設定変更後は `regsvr32 /u` で登録解除してから再登録する。

| フィールド | 説明 | デフォルト |
|-----------|------|----------|
| `japanese` | 日本語キーボードとして登録 | `true` |
| `korean` | 韓国語キーボードとして登録 | `false` |

### user_dict_path

ユーザー辞書ファイルのパス。`null` または未指定の場合、`%APPDATA%\Chamsae\user_dict.json` を自動検索する。

```json
"user_dict_path": "C:\\Users\\user\\my_dict.json"
```

## ユーザー辞書

`%APPDATA%\Chamsae\user_dict.json` を配置すると、カスタム変換が使用できる。
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

## トラブルシューティング

| 症状 | 原因・対処 |
|------|-----------|
| `regsvr32` でアクセス拒否 | 管理者権限で実行していない。コマンドプロンプトを「管理者として実行」で開く |
| モジュールが見つからない | DLLパスが間違っている。絶対パスで指定するか、DLLのあるディレクトリで実行する |
| エントリポイントが見つからない | ビルドターゲットが正しくない。`--target x86_64-pc-windows-gnu` を確認する |
| IMEが言語設定に表示されない | TSFプロファイル登録に失敗している可能性。`regsvr32` の出力を確認する |
