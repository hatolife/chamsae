//! TSFプロファイル・カテゴリ登録。
//!
//! TSFフレームワークにIMEとして登録/登録解除する。
//! DllRegisterServer/DllUnregisterServerから呼ばれる。
//!
//! ## 登録内容
//!
//! 1. ITfInputProcessorProfiles: テキストサービスの登録
//! 2. ITfInputProcessorProfileMgr: 言語プロファイルの登録 (韓国語・日本語)
//! 3. ITfCategoryMgr: キーボードTIPカテゴリの登録

use windows::core::{Interface, Result};
use windows::Win32::Foundation::TRUE;
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows::Win32::UI::Input::KeyboardAndMouse::HKL;
use windows::Win32::UI::TextServices::{
    CLSID_TF_CategoryMgr, CLSID_TF_InputProcessorProfiles, GUID_TFCAT_TIP_KEYBOARD,
    ITfCategoryMgr, ITfInputProcessorProfileMgr, ITfInputProcessorProfiles,
};

use crate::guid;

/// TSFプロファイルとカテゴリを登録する。
///
/// 1. テキストサービスをCLSIDで登録
/// 2. ITfInputProcessorProfileMgrで韓国語・日本語の言語プロファイルを追加
/// 3. キーボードTIPカテゴリに登録
pub fn register_tsf_profile() -> Result<()> {
    unsafe {
        // テキストサービスの登録。
        let profiles: ITfInputProcessorProfiles = CoCreateInstance(
            &CLSID_TF_InputProcessorProfiles,
            None,
            CLSCTX_INPROC_SERVER,
        )?;

        profiles.Register(&guid::CLSID_CHAMSAE_TEXT_SERVICE)?;

        // ITfInputProcessorProfileMgrを取得。
        // 同じCOMオブジェクトが両方のインターフェースを実装している。
        let profile_mgr: ITfInputProcessorProfileMgr = profiles.cast()?;

        let desc: Vec<u16> = guid::IME_DISPLAY_NAME.encode_utf16().collect();

        // 韓国語言語プロファイルの追加。(現在は日本語のみ登録)
        // profile_mgr.RegisterProfile(
        //     &guid::CLSID_CHAMSAE_TEXT_SERVICE,
        //     guid::LANGID_KOREAN,
        //     &guid::GUID_CHAMSAE_PROFILE,
        //     &desc,
        //     &[],
        //     0,
        //     HKL::default(),
        //     0,
        //     TRUE,
        //     0,
        // )?;

        // 日本語言語プロファイルの追加。
        // 日本語入力の状態からChamsaeに切り替えられるようにする。
        profile_mgr.RegisterProfile(
            &guid::CLSID_CHAMSAE_TEXT_SERVICE,
            guid::LANGID_JAPANESE,
            &guid::GUID_CHAMSAE_PROFILE_JA,
            &desc,
            &[],
            0,
            HKL::default(),
            0,
            TRUE,
            0,
        )?;

        // キーボードTIPカテゴリに登録。
        let category_mgr: ITfCategoryMgr = CoCreateInstance(
            &CLSID_TF_CategoryMgr,
            None,
            CLSCTX_INPROC_SERVER,
        )?;

        category_mgr.RegisterCategory(
            &guid::CLSID_CHAMSAE_TEXT_SERVICE,
            &GUID_TFCAT_TIP_KEYBOARD,
            &guid::CLSID_CHAMSAE_TEXT_SERVICE,
        )?;
    }

    Ok(())
}

/// TSFプロファイルとカテゴリの登録を解除する。
///
/// 登録と逆の順序で解除する:
/// 1. カテゴリの登録解除
/// 2. テキストサービスの登録解除
pub fn unregister_tsf_profile() -> Result<()> {
    unsafe {
        // カテゴリの登録解除。
        let category_mgr: ITfCategoryMgr = CoCreateInstance(
            &CLSID_TF_CategoryMgr,
            None,
            CLSCTX_INPROC_SERVER,
        )?;

        category_mgr.UnregisterCategory(
            &guid::CLSID_CHAMSAE_TEXT_SERVICE,
            &GUID_TFCAT_TIP_KEYBOARD,
            &guid::CLSID_CHAMSAE_TEXT_SERVICE,
        )?;

        // テキストサービスの登録解除。
        let profiles: ITfInputProcessorProfiles = CoCreateInstance(
            &CLSID_TF_InputProcessorProfiles,
            None,
            CLSCTX_INPROC_SERVER,
        )?;

        profiles.Unregister(&guid::CLSID_CHAMSAE_TEXT_SERVICE)?;
    }

    Ok(())
}
