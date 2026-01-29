//! DLLモジュール管理。
//!
//! DLLのグローバル状態を管理する:
//! - モジュールハンドル (HMODULE): DLLファイルパス取得に使う
//! - 参照カウント: COMオブジェクトの生存数を追跡
//! - サーバーロック: IClassFactory::LockServerによるロック数
//!
//! これらの状態はDllCanUnloadNowでDLLアンロード可否を判定するために使う。

use std::sync::atomic::{AtomicIsize, AtomicU32, Ordering};
use windows::Win32::Foundation::HMODULE;

/// DLLモジュールハンドル。
///
/// DllMainのDLL_PROCESS_ATTACHで設定される。
/// DllRegisterServerでDLLのファイルパスを取得する際に使用。
static MODULE_HANDLE: AtomicIsize = AtomicIsize::new(0);

/// COMオブジェクトの参照カウント。
///
/// ClassFactory経由で作成されたオブジェクト数を追跡。
/// 0になるとDllCanUnloadNowがS_OKを返す。
static OBJECT_COUNT: AtomicU32 = AtomicU32::new(0);

/// サーバーロックカウント。
///
/// IClassFactory::LockServerで増減する。
/// クライアントがサーバーを明示的にロックしている間はアンロード不可。
static SERVER_LOCK_COUNT: AtomicU32 = AtomicU32::new(0);

/// モジュールハンドルを設定する (DllMainから呼ばれる)。
pub fn set_module_handle(hmodule: HMODULE) {
    MODULE_HANDLE.store(hmodule.0 as isize, Ordering::SeqCst);
}

/// モジュールハンドルを取得する。
pub fn get_module_handle() -> HMODULE {
    HMODULE(MODULE_HANDLE.load(Ordering::SeqCst) as *mut core::ffi::c_void)
}

/// COMオブジェクトの生成を通知 (参照カウント+1)。
pub fn increment_object_count() {
    OBJECT_COUNT.fetch_add(1, Ordering::SeqCst);
}

/// COMオブジェクトの破棄を通知 (参照カウント-1)。
pub fn decrement_object_count() {
    OBJECT_COUNT.fetch_sub(1, Ordering::SeqCst);
}

/// サーバーロックを追加。
pub fn lock_server() {
    SERVER_LOCK_COUNT.fetch_add(1, Ordering::SeqCst);
}

/// サーバーロックを解除。
pub fn unlock_server() {
    SERVER_LOCK_COUNT.fetch_sub(1, Ordering::SeqCst);
}

/// DLLがアンロード可能か判定する。
///
/// オブジェクト参照カウントとサーバーロックカウントが両方0のとき、
/// DLLは安全にアンロードできる。
pub fn can_unload() -> bool {
    OBJECT_COUNT.load(Ordering::SeqCst) == 0
        && SERVER_LOCK_COUNT.load(Ordering::SeqCst) == 0
}

/// 現在のオブジェクト数を取得 (デバッグ用)。
pub fn object_count() -> u32 {
    OBJECT_COUNT.load(Ordering::SeqCst)
}

/// 現在のサーバーロック数を取得 (デバッグ用)。
pub fn server_lock_count() -> u32 {
    SERVER_LOCK_COUNT.load(Ordering::SeqCst)
}
