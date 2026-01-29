//! キーイベント処理。
//!
//! 仮想キーコードの判定とASCII文字への変換。
//! OnTestKeyDown/OnKeyDownで使用する。

/// 仮想キーコード定数。
pub const VK_BACK: u32 = 0x08;
pub const VK_RETURN: u32 = 0x0D;
pub const VK_SHIFT: u32 = 0x10;
pub const VK_CONTROL: u32 = 0x11;
pub const VK_MENU: u32 = 0x12;
pub const VK_ESCAPE: u32 = 0x1B;
pub const VK_SPACE: u32 = 0x20;

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
