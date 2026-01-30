//! 候補ウィンドウ。
//!
//! コンポジション中にローマ字入力と変換結果を表示する
//! カスタムWin32ポップアップウィンドウ。
//! キャレット位置に追従し、コンポジション終了時に非表示になる。
//!
//! ## 表示内容
//!
//! ```text
//! ┌──────────┐
//! │ 한국어    │  ← 変換結果 (大きめフォント)
//! │ han gug eo│  ← ローマ字入力 (小さめフォント)
//! └──────────┘
//! ```

use std::cell::Cell;

use windows::core::{w, Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontIndirectW, CreateSolidBrush, DeleteObject,
    EndPaint, FillRect, GetMonitorInfoW, GetTextExtentPoint32W,
    MonitorFromPoint, SelectObject, SetBkMode, SetTextColor, TextOutW,
    HBRUSH, HFONT, LOGFONTW, MONITOR_DEFAULTTONEAREST, MONITORINFO,
    PAINTSTRUCT, TRANSPARENT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow,
    MoveWindow, RegisterClassW, ShowWindow,
    CS_DROPSHADOW, CS_HREDRAW, CS_VREDRAW,
    SW_HIDE, SW_SHOWNOACTIVATE,
    WM_DESTROY, WM_PAINT,
    WNDCLASSW, WS_POPUP,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
};

use super::dpi;

/// 候補ウィンドウのクラス名。
const CANDIDATE_CLASS_NAME: PCWSTR = w!("ChamsaeCandidateWindow");

/// 背景色 (白)。
const BG_COLOR: u32 = 0x00FFFFFF;

/// 変換結果テキスト色 (黒)。
const HANGUL_TEXT_COLOR: u32 = 0x00000000;

/// ローマ字テキスト色 (グレー)。
const ROMAN_TEXT_COLOR: u32 = 0x00808080;

/// 変換結果フォントサイズ。
const HANGUL_FONT_SIZE: i32 = -18;

/// ローマ字フォントサイズ。
const ROMAN_FONT_SIZE: i32 = -13;

/// ウィンドウ内パディング。
const PADDING: i32 = 6;

/// ウィンドウの枠色 (青)。
const BORDER_COLOR: u32 = 0x00C08000;

/// 候補ウィンドウ。
///
/// コンポジション中に変換結果とローマ字入力を表示する。
pub struct CandidateWindow {
    hwnd: Cell<HWND>,
    class_registered: Cell<bool>,
}

/// ウィンドウに関連付けるテキストデータ (WM_PAINTで使用)。
///
/// SetWindowLongPtrで保存し、WM_PAINTで取得する。
static HANGUL_TEXT: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());
static ROMAN_TEXT: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());
/// WM_PAINT用のDPI値。
static PAINT_DPI: std::sync::Mutex<u32> = std::sync::Mutex::new(96);

impl CandidateWindow {
    /// 新しい候補ウィンドウを作成する。
    pub fn new() -> Self {
        Self {
            hwnd: Cell::new(HWND::default()),
            class_registered: Cell::new(false),
        }
    }

    /// ウィンドウクラスを登録する (初回のみ)。
    fn ensure_class_registered(&self) -> Result<()> {
        if self.class_registered.get() {
            return Ok(());
        }

        unsafe {
            let wc = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW | CS_DROPSHADOW,
                lpfnWndProc: Some(candidate_window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: windows::Win32::System::LibraryLoader::GetModuleHandleW(None)?
                    .into(),
                hIcon: Default::default(),
                hCursor: Default::default(),
                hbrBackground: HBRUSH::default(),
                lpszMenuName: PCWSTR::null(),
                lpszClassName: CANDIDATE_CLASS_NAME,
            };

            let atom = RegisterClassW(&wc);
            if atom != 0 {
                self.class_registered.set(true);
            }
        }

        Ok(())
    }

    /// ウィンドウを作成する (初回のみ)。
    fn ensure_window_created(&self) -> Result<()> {
        if !self.hwnd.get().0.is_null() {
            return Ok(());
        }

        self.ensure_class_registered()?;

        unsafe {
            let hwnd = CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE,
                CANDIDATE_CLASS_NAME,
                w!(""),
                WS_POPUP,
                0,
                0,
                1,
                1,
                None,
                None,
                windows::Win32::System::LibraryLoader::GetModuleHandleW(None)?,
                None,
            )?;

            self.hwnd.set(hwnd);
        }

        Ok(())
    }

    /// 候補ウィンドウを表示・更新する。
    ///
    /// `hangul`: 変換結果テキスト。
    /// `roman`: ローマ字入力テキスト。
    /// `x`, `y`: キャレット位置 (スクリーン座標)。
    pub fn show(&self, hangul: &str, roman: &str, x: i32, y: i32) -> Result<()> {
        self.ensure_window_created()?;

        let hwnd = self.hwnd.get();
        if hwnd.0.is_null() {
            return Ok(());
        }

        // テキストを静的変数に保存 (WM_PAINTで使用)。
        *HANGUL_TEXT.lock().unwrap() = hangul.to_string();
        *ROMAN_TEXT.lock().unwrap() = roman.to_string();
        *PAINT_DPI.lock().unwrap() = dpi::get_dpi_for_window(hwnd);

        // ウィンドウサイズを計算。
        let (width, height) = self.calculate_size(hangul, roman);

        // モニター領域内にクランプ。
        let (cx, cy) = Self::clamp_to_monitor(x, y, width, height);

        unsafe {
            let _ = MoveWindow(hwnd, cx, cy, width, height, true);
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);

            // 再描画を要求。
            let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, true);
        }

        Ok(())
    }

    /// 候補ウィンドウを非表示にする。
    pub fn hide(&self) {
        let hwnd = self.hwnd.get();
        if !hwnd.0.is_null() {
            unsafe {
                let _ = ShowWindow(hwnd, SW_HIDE);
            }
        }
    }

    /// ウィンドウを破棄する。
    pub fn destroy(&self) {
        let hwnd = self.hwnd.get();
        if !hwnd.0.is_null() {
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
            self.hwnd.set(HWND::default());
        }
    }

    /// テキストに基づいてウィンドウサイズを計算する。
    fn calculate_size(&self, hangul: &str, roman: &str) -> (i32, i32) {
        let hwnd = self.hwnd.get();
        if hwnd.0.is_null() {
            return (100, 50);
        }

        let window_dpi = dpi::get_dpi_for_window(hwnd);
        let scaled_padding = dpi::scale(PADDING, window_dpi);

        unsafe {
            let hdc = windows::Win32::Graphics::Gdi::GetDC(hwnd);

            // DPIスケーリングしたフォントでテキスト幅を計算。
            let hangul_font = create_font(dpi::scale(HANGUL_FONT_SIZE, window_dpi));
            let old_font = SelectObject(hdc, hangul_font);
            let hangul_wide: Vec<u16> = hangul.encode_utf16().collect();
            let mut hangul_size = windows::Win32::Foundation::SIZE::default();
            let _ = GetTextExtentPoint32W(hdc, &hangul_wide, &mut hangul_size);

            // ローマ字フォントでテキスト幅を計算。
            let roman_font = create_font(dpi::scale(ROMAN_FONT_SIZE, window_dpi));
            let _ = SelectObject(hdc, roman_font);
            let roman_wide: Vec<u16> = roman.encode_utf16().collect();
            let mut roman_size = windows::Win32::Foundation::SIZE::default();
            let _ = GetTextExtentPoint32W(hdc, &roman_wide, &mut roman_size);

            let _ = SelectObject(hdc, old_font);
            let _ = DeleteObject(hangul_font);
            let _ = DeleteObject(roman_font);
            let _ = windows::Win32::Graphics::Gdi::ReleaseDC(hwnd, hdc);

            let width = std::cmp::max(hangul_size.cx, roman_size.cx) + scaled_padding * 2 + 2;
            let height = hangul_size.cy + roman_size.cy + scaled_padding * 2 + 4;

            // 最小サイズ (DPIスケーリング済み)。
            let width = std::cmp::max(width, dpi::scale(60, window_dpi));
            let height = std::cmp::max(height, dpi::scale(40, window_dpi));

            (width, height)
        }
    }

    /// ウィンドウ位置をモニター領域内にクランプする。
    fn clamp_to_monitor(x: i32, y: i32, width: i32, height: i32) -> (i32, i32) {
        unsafe {
            let pt = POINT { x, y };
            let hmonitor = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
            let mut mi = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            if GetMonitorInfoW(hmonitor, &mut mi).as_bool() {
                let work = mi.rcWork;
                let mut cx = x;
                let mut cy = y;

                // 右端を超える場合は左にずらす。
                if cx + width > work.right {
                    cx = work.right - width;
                }
                // 下端を超える場合は上にずらす。
                if cy + height > work.bottom {
                    cy = work.bottom - height;
                }
                // 左端・上端を超える場合はクランプ。
                if cx < work.left {
                    cx = work.left;
                }
                if cy < work.top {
                    cy = work.top;
                }
                (cx, cy)
            } else {
                (x, y)
            }
        }
    }
}

impl Drop for CandidateWindow {
    fn drop(&mut self) {
        self.destroy();
    }
}

/// フォントを作成する。
unsafe fn create_font(height: i32) -> HFONT {
    let mut lf = LOGFONTW::default();
    lf.lfHeight = height;
    lf.lfWeight = 400; // FW_NORMAL
    lf.lfCharSet = windows::Win32::Graphics::Gdi::FONT_CHARSET(1); // DEFAULT_CHARSET
    lf.lfQuality = windows::Win32::Graphics::Gdi::FONT_QUALITY(5); // CLEARTYPE_QUALITY

    // "Meiryo UI" フォント名を設定。
    let face_name: Vec<u16> = "Meiryo UI\0".encode_utf16().collect();
    for (i, &ch) in face_name.iter().enumerate() {
        if i < lf.lfFaceName.len() {
            lf.lfFaceName[i] = ch;
        }
    }

    CreateFontIndirectW(&lf)
}

/// 候補ウィンドウのウィンドウプロシージャ。
extern "system" fn candidate_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut ps);

                let paint_dpi = *PAINT_DPI.lock().unwrap();
                let scaled_padding = dpi::scale(PADDING, paint_dpi);

                // 背景を塗る。
                let mut rc = RECT::default();
                let _ = windows::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &mut rc);
                let bg_brush = CreateSolidBrush(
                    windows::Win32::Foundation::COLORREF(BG_COLOR),
                );
                let _ = FillRect(hdc, &rc, bg_brush);
                let _ = DeleteObject(bg_brush);

                // 枠線を描画。
                let border_brush = CreateSolidBrush(
                    windows::Win32::Foundation::COLORREF(BORDER_COLOR),
                );
                let _ = windows::Win32::Graphics::Gdi::FrameRect(hdc, &rc, border_brush);
                let _ = DeleteObject(border_brush);

                SetBkMode(hdc, TRANSPARENT);

                // 変換結果テキスト (大きめフォント、DPIスケーリング済み)。
                let hangul_font = create_font(dpi::scale(HANGUL_FONT_SIZE, paint_dpi));
                let old_font = SelectObject(hdc, hangul_font);
                SetTextColor(hdc, windows::Win32::Foundation::COLORREF(HANGUL_TEXT_COLOR));
                let hangul_text = HANGUL_TEXT.lock().unwrap();
                let hangul_wide: Vec<u16> = hangul_text.encode_utf16().collect();
                drop(hangul_text);
                let _ = TextOutW(hdc, scaled_padding + 1, scaled_padding + 1, &hangul_wide);

                // ハングルテキストの高さを取得。
                let mut hangul_size = windows::Win32::Foundation::SIZE::default();
                let _ = GetTextExtentPoint32W(hdc, &hangul_wide, &mut hangul_size);

                // ローマ字テキスト (小さめフォント、DPIスケーリング済み)。
                let roman_font = create_font(dpi::scale(ROMAN_FONT_SIZE, paint_dpi));
                let _ = SelectObject(hdc, roman_font);
                SetTextColor(hdc, windows::Win32::Foundation::COLORREF(ROMAN_TEXT_COLOR));
                let roman_text = ROMAN_TEXT.lock().unwrap();
                let roman_wide: Vec<u16> = roman_text.encode_utf16().collect();
                drop(roman_text);
                let _ = TextOutW(
                    hdc,
                    scaled_padding + 1,
                    scaled_padding + 1 + hangul_size.cy + 2,
                    &roman_wide,
                );

                let _ = SelectObject(hdc, old_font);
                let _ = DeleteObject(hangul_font);
                let _ = DeleteObject(roman_font);

                let _ = EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_DESTROY => LRESULT(0),
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
