; Chamsae Hangul IME - InnoSetup スクリプト
;
; ビルド手順:
;   1. make release
;   2. InnoSetup Compiler で本スクリプトをコンパイル
;
; インストール時: ファイル配置 → regsvr32 chamsae.dll
; アンインストール時: regsvr32 /u chamsae.dll → ファイル削除

#define MyAppName "Chamsae Hangul IME"
#define MyAppVersion "0.5.0"
#define MyAppPublisher "Chamsae"
#define MyAppURL ""
#define MyAppExeName "chamsae_settings.exe"

[Setup]
AppId={{D4A5B8E1-7C2F-4A3D-9E6B-1F8C0D2A5E7B}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={autopf}\Chamsae
DefaultGroupName=Chamsae
AllowNoIcons=yes
OutputDir=..\build
OutputBaseFilename=chamsae-v{#MyAppVersion}-setup
Compression=lzma
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
; 管理者権限が必要 (regsvr32)
PrivilegesRequired=admin

[Languages]
Name: "japanese"; MessagesFile: "compiler:Languages\Japanese.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
; DLL
Source: "..\build\chamsae.dll"; DestDir: "{app}"; Flags: ignoreversion
; 設定GUI
Source: "..\build\chamsae_settings.exe"; DestDir: "{app}"; Flags: ignoreversion
; CLIツール
Source: "..\build\chamsae.exe"; DestDir: "{app}"; Flags: ignoreversion
; バッチファイル
Source: "..\build\install.bat"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\build\uninstall.bat"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
; スタートメニュー
Name: "{group}\Chamsae 設定"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\Chamsae アンインストール"; Filename: "{uninstallexe}"

[Run]
; インストール後にDLLを登録。
Filename: "regsvr32.exe"; Parameters: "/s ""{app}\chamsae.dll"""; \
  StatusMsg: "IMEを登録しています..."; Flags: runhidden waituntilterminated
; インストール後に設定テンプレートを生成。
Filename: "{app}\chamsae.exe"; Parameters: "-t"; \
  WorkingDir: "{app}"; Flags: runhidden waituntilterminated

[UninstallRun]
; アンインストール前にDLL登録を解除。
Filename: "regsvr32.exe"; Parameters: "/s /u ""{app}\chamsae.dll"""; \
  Flags: runhidden waituntilterminated
