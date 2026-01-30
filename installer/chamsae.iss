; Chamsae Hangul IME - InnoSetup スクリプト
;
; ビルド手順:
;   1. make release
;   2. InnoSetup Compiler で本スクリプトをコンパイル
;      iscc /DMyAppVersion=X.Y.Z installer/chamsae.iss
;
; アップグレード時:
;   1. 旧DLLのTSF登録解除 (regsvr32 /u)
;   2. 旧DLLを .old にリネーム (ロック中でも成功)
;   3. 新ファイル配置
;   4. regsvr32 chamsae.dll (再登録)
;   5. .old ファイル削除 (失敗時は次回クリーンアップ)
;
; アンインストール時: regsvr32 /u chamsae.dll → ファイル削除

#define MyAppName "Chamsae Hangul IME"
; /DMyAppVersion=X.Y.Z で上書き可能。未指定時はデフォルト値を使用。
#ifndef MyAppVersion
  #define MyAppVersion "0.6.5"
#endif
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

[Code]
/// インストール準備: 旧DLLリネーム方式でアップグレード。
///
/// 1. 旧 .old ファイルを削除 (前回の残り)
/// 2. 旧DLLのTSF登録を解除 (regsvr32 /u)
/// 3. 旧DLLを .old にリネーム (ロック中でも成功)
///
/// リネーム後、Inno Setup が新DLLを配置。
/// [Run] セクションで regsvr32 再登録。
function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  ResultCode: Integer;
  DllPath: String;
  OldPath: String;
begin
  Result := '';
  DllPath := ExpandConstant('{app}\chamsae.dll');
  OldPath := ExpandConstant('{app}\chamsae.dll.old');

  // 前回の .old ファイルを削除。
  if FileExists(OldPath) then
    DeleteFile(OldPath);

  if FileExists(DllPath) then
  begin
    // 旧DLLのTSF登録を解除。
    Exec('regsvr32.exe', '/s /u "' + DllPath + '"',
      '', SW_HIDE, ewWaitUntilTerminated, ResultCode);

    // 旧DLLをリネーム (ロック中でも成功)。
    RenameFile(DllPath, OldPath);
  end;
end;

/// インストール完了後: .old ファイルの削除を試みる。
procedure CurStepChanged(CurStep: TSetupStep);
var
  OldPath: String;
begin
  if CurStep = ssPostInstall then
  begin
    OldPath := ExpandConstant('{app}\chamsae.dll.old');
    if FileExists(OldPath) then
      DeleteFile(OldPath);
  end;
end;
