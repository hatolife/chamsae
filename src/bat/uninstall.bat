@echo off

rem 管理者権限でコマンドプロンプトを開きなおす
for /f "tokens=3 delims=\ " %%A in ('whoami /groups^|find "Mandatory"') do set LEVEL=%%A
if not "%LEVEL%"=="High" (
	powershell -NoProfile -ExecutionPolicy Unrestricted -Command "Start-Process \"%~f0\" -Verb runas"
	exit
)

pushd "%~dp0"

regsvr32 /u chamsae.dll

pause
