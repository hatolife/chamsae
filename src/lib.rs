//! Chamsae - ハングルIME ライブラリ。
//!
//! ローマ字入力から韓国語ハングル文字への変換を行うIME。
//! Phase 5: 改善・拡張 (ユーザー辞書、候補ウィンドウ、トレイアイコン、設定画面)。

pub mod hangul;
pub mod config;
pub mod user_dict;

// Windows専用モジュール。
#[cfg(windows)]
pub mod guid;
#[cfg(windows)]
pub mod com;
#[cfg(windows)]
pub mod win32;
#[cfg(windows)]
pub mod registry;
#[cfg(windows)]
pub mod tsf;

// === DLLエクスポート関数 (Windows専用) ===

/// DllMain - DLLのエントリポイント。
///
/// DLLがプロセスにロード/アンロードされる際に呼ばれる。
/// モジュールハンドルを保存し、スレッド通知を無効化する。
#[cfg(windows)]
#[no_mangle]
extern "system" fn DllMain(
    hinstance: windows::Win32::Foundation::HMODULE,
    reason: u32,
    _reserved: *mut core::ffi::c_void,
) -> windows::Win32::Foundation::BOOL {
    use windows::Win32::Foundation::TRUE;

    const DLL_PROCESS_ATTACH: u32 = 1;
    const DLL_PROCESS_DETACH: u32 = 0;

    match reason {
        DLL_PROCESS_ATTACH => {
            // モジュールハンドルを保存。
            com::dll_module::set_module_handle(hinstance);

            // スレッドアタッチ/デタッチ通知を無効化 (パフォーマンス最適化)。
            unsafe {
                windows::Win32::System::LibraryLoader::DisableThreadLibraryCalls(hinstance)
                    .ok();
            }

            TRUE
        }
        DLL_PROCESS_DETACH => {
            TRUE
        }
        _ => TRUE,
    }
}

/// DllGetClassObject - COMクラスファクトリを返す。
///
/// COMランタイムがオブジェクトを作成する際に呼ぶ。
/// CLSIDが一致すれば、ClassFactoryのIClassFactoryインターフェースを返す。
#[cfg(windows)]
#[no_mangle]
extern "system" fn DllGetClassObject(
    rclsid: *const windows::core::GUID,
    riid: *const windows::core::GUID,
    ppv: *mut *mut core::ffi::c_void,
) -> windows::core::HRESULT {
    use windows::core::Interface;

    unsafe {
        if ppv.is_null() {
            return windows::Win32::Foundation::E_POINTER;
        }
        *ppv = core::ptr::null_mut();

        let rclsid = &*rclsid;
        let riid = &*riid;

        // CLSIDの確認。
        if *rclsid != guid::CLSID_CHAMSAE_TEXT_SERVICE {
            return windows::Win32::Foundation::CLASS_E_CLASSNOTAVAILABLE;
        }

        // ClassFactoryを作成。
        let factory = com::class_factory::ClassFactory::new();
        let unknown: windows::core::IUnknown = factory.into();

        // 要求されたインターフェースを返す。
        unknown.query(riid, ppv)
    }
}

/// DllCanUnloadNow - DLLがアンロード可能か確認。
///
/// 全てのCOMオブジェクトが解放され、サーバーロックがなければS_OKを返す。
#[cfg(windows)]
#[no_mangle]
extern "system" fn DllCanUnloadNow() -> windows::core::HRESULT {
    if com::dll_module::can_unload() {
        windows::Win32::Foundation::S_OK
    } else {
        windows::Win32::Foundation::S_FALSE
    }
}

/// DllRegisterServer - DLLをシステムに登録。
///
/// レジストリにCLSID、InprocServer32パスを書き込む。
#[cfg(windows)]
#[no_mangle]
extern "system" fn DllRegisterServer() -> windows::core::HRESULT {
    match registry::register_server() {
        Ok(()) => windows::Win32::Foundation::S_OK,
        Err(_) => windows::Win32::Foundation::E_FAIL,
    }
}

/// DllUnregisterServer - DLLの登録を解除。
///
/// レジストリからCLSID関連のキーを削除する。
#[cfg(windows)]
#[no_mangle]
extern "system" fn DllUnregisterServer() -> windows::core::HRESULT {
    match registry::unregister_server() {
        Ok(()) => windows::Win32::Foundation::S_OK,
        Err(_) => windows::Win32::Foundation::E_FAIL,
    }
}
