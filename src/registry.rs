//! レジストリ登録。
//!
//! DLLをWindowsシステムに登録/登録解除する機能。
//! regsvr32コマンドがDllRegisterServer/DllUnregisterServerを呼び、
//! ここで定義した関数がレジストリを操作する。
//!
//! ## レジストリ構成
//!
//! ```text
//! HKEY_CLASSES_ROOT
//! └── CLSID
//!     └── {D4A5B8E1-7C2F-4A3D-9E6B-1F8C0D2A5E7B}
//!         ├── (Default) = "Chamsae Hangul IME"
//!         └── InprocServer32
//!             ├── (Default) = "C:\path\to\hangul_ime.dll"
//!             └── ThreadingModel = "Apartment"
//! ```
//!
//! ## 使い方
//!
//! ```bat
//! REM 登録
//! regsvr32 hangul_ime.dll
//!
//! REM 登録解除
//! regsvr32 /u hangul_ime.dll
//! ```

use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::{ERROR_SUCCESS, MAX_PATH, WIN32_ERROR};
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows::Win32::System::Registry::{
    RegCreateKeyExW, RegDeleteTreeW, RegSetValueExW,
    HKEY, HKEY_CLASSES_ROOT, KEY_WRITE, REG_OPTION_NON_VOLATILE, REG_SZ,
};

use crate::com::dll_module;
use crate::guid;

/// WIN32_ERRORをResult<()>に変換する。
fn win32_error_to_result(err: WIN32_ERROR) -> Result<()> {
    if err == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(windows::core::Error::from(err.to_hresult()))
    }
}

/// DLLをレジストリに登録する。
///
/// CLSIDキーとInprocServer32サブキーを作成し、
/// DLLパスとスレッドモデルを設定する。
pub fn register_server() -> Result<()> {
    unsafe {
        // DLLのファイルパスを取得。
        let dll_path = get_dll_path()?;

        // CLSID キーを作成。
        // HKCR\CLSID\{guid}
        let clsid_key_path = format!(
            "CLSID\\{}",
            guid::CLSID_CHAMSAE_TEXT_SERVICE_STR
        );
        let clsid_key = create_reg_key(HKEY_CLASSES_ROOT, &clsid_key_path)?;

        // デフォルト値 = 表示名。
        set_reg_string_value(&clsid_key, None, guid::IME_DISPLAY_NAME)?;

        // InprocServer32 サブキーを作成。
        // HKCR\CLSID\{guid}\InprocServer32
        let inproc_key_path = format!(
            "CLSID\\{}\\InprocServer32",
            guid::CLSID_CHAMSAE_TEXT_SERVICE_STR
        );
        let inproc_key = create_reg_key(HKEY_CLASSES_ROOT, &inproc_key_path)?;

        // デフォルト値 = DLLパス。
        set_reg_string_value(&inproc_key, None, &dll_path)?;

        // ThreadingModel = "Apartment"。
        // TSF IMEはSTA (Single-Threaded Apartment) で動作する。
        set_reg_string_value(&inproc_key, Some("ThreadingModel"), "Apartment")?;
    }

    Ok(())
}

/// DLLのレジストリ登録を解除する。
///
/// CLSIDキー以下のサブキーを全て削除する。
pub fn unregister_server() -> Result<()> {
    unsafe {
        let clsid_key_path = format!(
            "CLSID\\{}",
            guid::CLSID_CHAMSAE_TEXT_SERVICE_STR
        );
        let wide_path: Vec<u16> = clsid_key_path.encode_utf16().chain(Some(0)).collect();

        // サブキーを含めて全て削除。
        let result = RegDeleteTreeW(HKEY_CLASSES_ROOT, PCWSTR(wide_path.as_ptr()));
        win32_error_to_result(result)?;
    }

    Ok(())
}

/// DLLのファイルパスを取得する。
///
/// DllMainで保存したモジュールハンドルを使って、
/// DLLの完全パスを取得する。
unsafe fn get_dll_path() -> Result<String> {
    let hmodule = dll_module::get_module_handle();
    let mut path_buf = [0u16; MAX_PATH as usize];
    let len = GetModuleFileNameW(hmodule, &mut path_buf);

    if len == 0 {
        return Err(windows::core::Error::from_win32());
    }

    Ok(String::from_utf16_lossy(&path_buf[..len as usize]))
}

/// レジストリキーを作成する (既存の場合は開く)。
unsafe fn create_reg_key(parent: HKEY, subkey: &str) -> Result<HKEY> {
    let wide_subkey: Vec<u16> = subkey.encode_utf16().chain(Some(0)).collect();
    let mut hkey = HKEY::default();

    let result = RegCreateKeyExW(
        parent,
        PCWSTR(wide_subkey.as_ptr()),
        0,
        None,
        REG_OPTION_NON_VOLATILE,
        KEY_WRITE,
        None,
        &mut hkey,
        None,
    );
    win32_error_to_result(result)?;

    Ok(hkey)
}

/// レジストリの文字列値を設定する。
unsafe fn set_reg_string_value(
    hkey: &HKEY,
    name: Option<&str>,
    value: &str,
) -> Result<()> {
    let wide_name: Option<Vec<u16>> =
        name.map(|n| n.encode_utf16().chain(Some(0)).collect());
    let wide_value: Vec<u16> = value.encode_utf16().chain(Some(0)).collect();
    let value_bytes: &[u8] = core::slice::from_raw_parts(
        wide_value.as_ptr() as *const u8,
        wide_value.len() * 2,
    );

    let name_pcwstr = wide_name
        .as_ref()
        .map(|n| PCWSTR(n.as_ptr()))
        .unwrap_or(PCWSTR::null());

    let result = RegSetValueExW(
        *hkey,
        name_pcwstr,
        0,
        REG_SZ,
        Some(value_bytes),
    );
    win32_error_to_result(result)?;

    Ok(())
}
