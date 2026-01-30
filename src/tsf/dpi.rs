//! DPIスケーリングヘルパー。
//!
//! マルチモニターDPI対応のためのユーティリティ関数。
//! `GetDpiForWindow` APIでウィンドウのDPIを取得し、
//! 論理ピクセルをスケーリングする。

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::HiDpi::GetDpiForWindow;

/// デフォルトDPI (96dpi = 100%)。
const DEFAULT_DPI: u32 = 96;

/// ウィンドウのDPIを取得する。
///
/// `GetDpiForWindow` APIを使用。
/// ウィンドウが無効な場合はデフォルト値 (96) を返す。
pub fn get_dpi_for_window(hwnd: HWND) -> u32 {
    if hwnd.0.is_null() {
        return DEFAULT_DPI;
    }
    let dpi = unsafe { GetDpiForWindow(hwnd) };
    if dpi == 0 { DEFAULT_DPI } else { dpi }
}

/// 論理値をDPIスケーリングする。
///
/// `value * dpi / 96` で物理ピクセルに変換する。
pub fn scale(value: i32, dpi: u32) -> i32 {
    (value as i64 * dpi as i64 / DEFAULT_DPI as i64) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_100_percent() {
        assert_eq!(scale(10, 96), 10);
        assert_eq!(scale(-18, 96), -18);
        assert_eq!(scale(6, 96), 6);
    }

    #[test]
    fn test_scale_150_percent() {
        assert_eq!(scale(10, 144), 15);
        assert_eq!(scale(-18, 144), -27);
        assert_eq!(scale(6, 144), 9);
    }

    #[test]
    fn test_scale_200_percent() {
        assert_eq!(scale(10, 192), 20);
        assert_eq!(scale(-18, 192), -36);
        assert_eq!(scale(6, 192), 12);
    }

    #[test]
    fn test_scale_zero() {
        assert_eq!(scale(0, 96), 0);
        assert_eq!(scale(0, 192), 0);
    }
}
