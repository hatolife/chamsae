//! IClassFactory実装。
//!
//! COMのクラスファクトリパターンを実装する。
//! DllGetClassObjectから返され、COMクライアントがオブジェクトを作成するために使う。
//!
//! ## COMクラスファクトリの仕組み
//!
//! 1. クライアントがCoCreateInstanceを呼ぶ
//! 2. COMランタイムがDllGetClassObjectを呼ぶ
//! 3. DllGetClassObjectがClassFactoryを返す
//! 4. COMランタイムがIClassFactory::CreateInstanceを呼ぶ
//! 5. ClassFactoryがTextServiceオブジェクトを作成して返す

use windows::core::{implement, IUnknown, GUID};
use windows::Win32::Foundation::{BOOL, CLASS_E_NOAGGREGATION, E_NOINTERFACE};
use windows::Win32::System::Com::{IClassFactory, IClassFactory_Impl};

use super::dll_module;

/// COMクラスファクトリ。
///
/// IClassFactoryインターフェースを実装し、
/// TextServiceオブジェクトの生成を担当する。
#[implement(IClassFactory)]
pub struct ClassFactory;

impl ClassFactory {
    /// 新しいClassFactoryを作成する。
    pub fn new() -> Self {
        dll_module::increment_object_count();
        Self
    }
}

impl Drop for ClassFactory {
    fn drop(&mut self) {
        dll_module::decrement_object_count();
    }
}

impl IClassFactory_Impl for ClassFactory_Impl {
    /// COMオブジェクトのインスタンスを作成する。
    ///
    /// # 引数
    ///
    /// - `punkouter`: アグリゲーション用の外部IUnknown (未サポート、Noneであること)
    /// - `riid`: 要求するインターフェースのIID
    /// - `ppvobject`: 作成されたオブジェクトのポインタを受け取る
    ///
    /// # Phase 2 の制限
    ///
    /// 現在はTextServiceが未実装のため、E_NOINTERFACEを返す。
    /// Phase 3でITfTextInputProcessorExを実装後、ここでTextServiceを返す。
    fn CreateInstance(
        &self,
        punkouter: Option<&IUnknown>,
        _riid: *const GUID,
        ppvobject: *mut *mut core::ffi::c_void,
    ) -> windows::core::Result<()> {
        unsafe {
            if !ppvobject.is_null() {
                *ppvobject = core::ptr::null_mut();
            }

            // COMアグリゲーションは未サポート。
            if punkouter.is_some() {
                return Err(CLASS_E_NOAGGREGATION.into());
            }

            // Phase 3で実装予定: TextServiceオブジェクトの作成。
            // let text_service = TextService::new();
            // let unknown: IUnknown = text_service.into();
            // return unknown.query(riid, ppvobject).ok();

            // Phase 2: TextService未実装のためE_NOINTERFACE。
            Err(E_NOINTERFACE.into())
        }
    }

    /// サーバーロックの増減。
    ///
    /// COMクライアントがDLLをメモリに保持したい場合に使う。
    /// flock=TRUEでロック追加、FALSEでロック解除。
    fn LockServer(&self, flock: BOOL) -> windows::core::Result<()> {
        if flock.as_bool() {
            dll_module::lock_server();
        } else {
            dll_module::unlock_server();
        }
        Ok(())
    }
}
