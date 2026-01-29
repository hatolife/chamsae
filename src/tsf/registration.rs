//! TSFプロファイル・カテゴリ登録。
//!
//! TSFフレームワークにIMEとして登録/登録解除する。
//! DllRegisterServer/DllUnregisterServerから呼ばれる。
//!
//! ## 登録内容
//!
//! 1. ITfInputProcessorProfiles: テキストサービスと言語プロファイルの登録
//! 2. ITfCategoryMgr: キーボードTIPカテゴリの登録

use windows::core::Result;
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows::Win32::UI::TextServices::{
    CLSID_TF_CategoryMgr, CLSID_TF_InputProcessorProfiles, GUID_TFCAT_TIP_KEYBOARD,
    ITfCategoryMgr, ITfInputProcessorProfiles,
};

use crate::guid;

/// TSFプロファイルとカテゴリを登録する。
///
/// 1. テキストサービスをCLSIDで登録
/// 2. 韓国語の言語プロファイルを追加
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

        // 韓国語言語プロファイルの追加。
        let desc: Vec<u16> = guid::IME_DISPLAY_NAME.encode_utf16().collect();

        profiles.AddLanguageProfile(
            &guid::CLSID_CHAMSAE_TEXT_SERVICE,
            guid::LANGID_KOREAN,
            &guid::GUID_CHAMSAE_PROFILE,
            &desc,
            &[],
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
