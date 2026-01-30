//! キーイベント処理。
//!
//! 仮想キーコードの判定とASCII文字への変換。
//! OnTestKeyDown/OnKeyDownで使用する。

/// 仮想キーコード定数。
pub const VK_BACK: u32 = 0x08;
pub const VK_TAB: u32 = 0x09;
pub const VK_RETURN: u32 = 0x0D;
pub const VK_SHIFT: u32 = 0x10;
pub const VK_CONTROL: u32 = 0x11;
pub const VK_MENU: u32 = 0x12;
pub const VK_ESCAPE: u32 = 0x1B;
pub const VK_SPACE: u32 = 0x20;
pub const VK_END: u32 = 0x23;
pub const VK_HOME: u32 = 0x24;
pub const VK_LEFT: u32 = 0x25;
pub const VK_UP: u32 = 0x26;
pub const VK_RIGHT: u32 = 0x27;
pub const VK_DOWN: u32 = 0x28;
pub const VK_DELETE: u32 = 0x2E;

/// 仮想キーコードからASCII小文字への変換。
///
/// A-Z (0x41-0x5A) のキーコードを a-z に変換する。
/// それ以外のキーはNoneを返す。
pub fn vk_to_char(vk: u32) -> Option<char> {
    match vk {
        0x41..=0x5A => Some((vk as u8 - b'A' + b'a') as char),
        _ => None,
    }
}

/// ハングル変換で処理すべきキーか判定。
///
/// ローマ字入力キー (a-z) の場合にtrueを返す。
pub fn is_hangul_key(vk: u32) -> bool {
    vk_to_char(vk).is_some()
}

/// コンポジション中に処理すべき制御キーか判定。
///
/// バッファが空でない場合にこれらのキーを横取りする:
/// - Backspace: バッファから1文字削除
/// - Enter: コンポジション確定
/// - Escape: コンポジションキャンセル
/// - Space: 音節区切り
pub fn is_control_key(vk: u32) -> bool {
    matches!(vk, VK_BACK | VK_RETURN | VK_SPACE | VK_ESCAPE)
}

/// ナビゲーションキーか判定。
///
/// コンポジション中にこれらのキーが押されると、
/// コンポジションを自動確定してからキーをパススルーする:
/// - Tab: タブ
/// - 矢印キー: Left/Right/Up/Down
/// - Home/End: 行頭/行末
/// - Delete: 削除
pub fn is_navigation_key(vk: u32) -> bool {
    matches!(
        vk,
        VK_TAB | VK_LEFT | VK_RIGHT | VK_UP | VK_DOWN | VK_HOME | VK_END | VK_DELETE
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_navigation_key_arrows() {
        assert!(is_navigation_key(VK_LEFT));
        assert!(is_navigation_key(VK_RIGHT));
        assert!(is_navigation_key(VK_UP));
        assert!(is_navigation_key(VK_DOWN));
    }

    #[test]
    fn test_is_navigation_key_home_end_delete_tab() {
        assert!(is_navigation_key(VK_HOME));
        assert!(is_navigation_key(VK_END));
        assert!(is_navigation_key(VK_DELETE));
        assert!(is_navigation_key(VK_TAB));
    }

    #[test]
    fn test_is_navigation_key_not_navigation() {
        assert!(!is_navigation_key(VK_BACK));
        assert!(!is_navigation_key(VK_RETURN));
        assert!(!is_navigation_key(VK_SPACE));
        assert!(!is_navigation_key(VK_ESCAPE));
        assert!(!is_navigation_key(0x41)); // 'A'
    }

    #[test]
    fn test_vk_to_char() {
        assert_eq!(vk_to_char(0x41), Some('a'));
        assert_eq!(vk_to_char(0x5A), Some('z'));
        assert_eq!(vk_to_char(0x20), None); // Space
        assert_eq!(vk_to_char(0x08), None); // Backspace
    }

    #[test]
    fn test_is_hangul_key() {
        assert!(is_hangul_key(0x41)); // A
        assert!(is_hangul_key(0x5A)); // Z
        assert!(!is_hangul_key(0x20)); // Space
        assert!(!is_hangul_key(0x30)); // '0'
    }

    #[test]
    fn test_is_control_key() {
        assert!(is_control_key(VK_BACK));
        assert!(is_control_key(VK_RETURN));
        assert!(is_control_key(VK_SPACE));
        assert!(is_control_key(VK_ESCAPE));
        assert!(!is_control_key(VK_TAB));
        assert!(!is_control_key(0x41));
    }
}
