//! GUID/CLSID定義。
//!
//! Chamsae IMEで使用するGUIDを定義する。
//! COMオブジェクトの識別やTSFプロファイルに必要。

use windows::core::GUID;

/// Chamsae TextServiceのCLSID。
///
/// COMランタイムがTextServiceオブジェクトを特定するために使う。
/// regsvr32でDLLを登録する際、このCLSIDでレジストリに書き込まれる。
pub const CLSID_CHAMSAE_TEXT_SERVICE: GUID = GUID::from_u128(
    0xD4A5B8E1_7C2F_4A3D_9E6B_1F8C0D2A5E7B
);

/// Chamsae言語プロファイルのGUID。
///
/// TSFフレームワークで言語プロファイルを識別する。
/// Phase 3でITfInputProcessorProfileMgr::RegisterProfileに渡す。
pub const GUID_CHAMSAE_PROFILE: GUID = GUID::from_u128(
    0xA2C4E6F8_1B3D_5A7C_9E0F_2D4B6A8C0E1F
);

/// CLSID文字列 (レジストリ登録用)。
pub const CLSID_CHAMSAE_TEXT_SERVICE_STR: &str = "{D4A5B8E1-7C2F-4A3D-9E6B-1F8C0D2A5E7B}";

/// IME表示名。
pub const IME_DISPLAY_NAME: &str = "Chamsae Hangul IME";

/// IMEの説明。
pub const IME_DESCRIPTION: &str = "Chamsae - Romanji to Hangul Input Method Editor";

/// 韓国語のLANGID。
pub const LANGID_KOREAN: u16 = 0x0412;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clsid_not_zero() {
        let zero = GUID::from_u128(0);
        assert_ne!(CLSID_CHAMSAE_TEXT_SERVICE, zero);
        assert_ne!(GUID_CHAMSAE_PROFILE, zero);
    }

    #[test]
    fn test_clsid_different_from_profile() {
        assert_ne!(CLSID_CHAMSAE_TEXT_SERVICE, GUID_CHAMSAE_PROFILE);
    }

    #[test]
    fn test_clsid_string() {
        assert!(CLSID_CHAMSAE_TEXT_SERVICE_STR.starts_with('{'));
        assert!(CLSID_CHAMSAE_TEXT_SERVICE_STR.ends_with('}'));
    }
}
