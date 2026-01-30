//! アイコンリソース (プログラマティック生成)。
//!
//! GDIを使ってIMEのトレイアイコンを動的に生成する。
//! リソースファイル不要でアイコンを作成できる。
//!
//! - IME ON: 緑色の背景に「韓」
//! - IME OFF: グレーの背景に「A」

use windows::Win32::Foundation::COLORREF;
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleBitmap, CreateCompatibleDC, CreateFontIndirectW, CreateSolidBrush,
    DeleteDC, DeleteObject, FillRect, GetDC, ReleaseDC, SelectObject,
    SetBkMode, SetTextColor, TextOutW,
    HDC, HFONT, LOGFONTW, TRANSPARENT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateIconIndirect, DestroyIcon, HICON, ICONINFO,
};
use windows::Win32::Foundation::RECT;

/// アイコンサイズ (16x16)。
const ICON_SIZE: i32 = 16;

/// IME ON時の背景色 (緑)。
const ON_BG_COLOR: u32 = 0x0040A040;

/// IME OFF時の背景色 (グレー)。
const OFF_BG_COLOR: u32 = 0x00808080;

/// テキスト色 (白)。
const TEXT_COLOR: u32 = 0x00FFFFFF;

/// IME ON用アイコンを作成する (緑の「韓」)。
pub fn create_on_icon() -> HICON {
    create_text_icon("韓", ON_BG_COLOR)
}

/// IME OFF用アイコンを作成する (グレーの「A」)。
pub fn create_off_icon() -> HICON {
    create_text_icon("A", OFF_BG_COLOR)
}

/// アイコンを破棄する。
pub fn destroy_icon(icon: HICON) {
    if !icon.0.is_null() {
        unsafe {
            let _ = DestroyIcon(icon);
        }
    }
}

/// テキスト付きアイコンを作成する。
fn create_text_icon(text: &str, bg_color: u32) -> HICON {
    unsafe {
        let hdc_screen = GetDC(None);
        let hdc_mem = CreateCompatibleDC(hdc_screen);
        let hbm_color = CreateCompatibleBitmap(hdc_screen, ICON_SIZE, ICON_SIZE);
        let hbm_mask = CreateCompatibleBitmap(hdc_screen, ICON_SIZE, ICON_SIZE);

        // カラービットマップに描画。
        let old_bmp = SelectObject(hdc_mem, hbm_color);
        draw_icon_content(hdc_mem, text, bg_color);
        let _ = SelectObject(hdc_mem, old_bmp);

        // マスクビットマップ (すべて0 = 不透明)。
        let old_bmp = SelectObject(hdc_mem, hbm_mask);
        let mask_brush = CreateSolidBrush(COLORREF(0x00000000));
        let rc = RECT {
            left: 0,
            top: 0,
            right: ICON_SIZE,
            bottom: ICON_SIZE,
        };
        let _ = FillRect(hdc_mem, &rc, mask_brush);
        let _ = DeleteObject(mask_brush);
        let _ = SelectObject(hdc_mem, old_bmp);

        // ICONINFO からアイコンを作成。
        let icon_info = ICONINFO {
            fIcon: true.into(),
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: hbm_mask,
            hbmColor: hbm_color,
        };

        let hicon = CreateIconIndirect(&icon_info).unwrap_or_default();

        // リソース解放。
        let _ = DeleteObject(hbm_color);
        let _ = DeleteObject(hbm_mask);
        let _ = DeleteDC(hdc_mem);
        ReleaseDC(None, hdc_screen);

        hicon
    }
}

/// アイコンの内容をDCに描画する。
unsafe fn draw_icon_content(hdc: HDC, text: &str, bg_color: u32) {
    // 背景を塗りつぶし。
    let bg_brush = CreateSolidBrush(COLORREF(bg_color));
    let rc = RECT {
        left: 0,
        top: 0,
        right: ICON_SIZE,
        bottom: ICON_SIZE,
    };
    let _ = FillRect(hdc, &rc, bg_brush);
    let _ = DeleteObject(bg_brush);

    // テキスト描画。
    let font = create_icon_font(text);
    let old_font = SelectObject(hdc, font);
    SetBkMode(hdc, TRANSPARENT);
    SetTextColor(hdc, COLORREF(TEXT_COLOR));

    let wide: Vec<u16> = text.encode_utf16().collect();

    // テキストを中央に配置。
    let mut text_size = windows::Win32::Foundation::SIZE::default();
    let _ = windows::Win32::Graphics::Gdi::GetTextExtentPoint32W(hdc, &wide, &mut text_size);
    let x = (ICON_SIZE - text_size.cx) / 2;
    let y = (ICON_SIZE - text_size.cy) / 2;

    let _ = TextOutW(hdc, x, y, &wide);

    let _ = SelectObject(hdc, old_font);
    let _ = DeleteObject(font);
}

/// アイコン用フォントを作成する。
unsafe fn create_icon_font(text: &str) -> HFONT {
    let mut lf = LOGFONTW::default();
    // 漢字は大きめ、アルファベットはやや小さめ。
    lf.lfHeight = if text.chars().any(|c| c > '\u{7F}') {
        -12
    } else {
        -11
    };
    lf.lfWeight = 700; // FW_BOLD
    lf.lfCharSet = windows::Win32::Graphics::Gdi::FONT_CHARSET(1); // DEFAULT_CHARSET
    lf.lfQuality = windows::Win32::Graphics::Gdi::FONT_QUALITY(5); // CLEARTYPE_QUALITY

    let face_name: Vec<u16> = "Meiryo UI\0".encode_utf16().collect();
    for (i, &ch) in face_name.iter().enumerate() {
        if i < lf.lfFaceName.len() {
            lf.lfFaceName[i] = ch;
        }
    }

    CreateFontIndirectW(&lf)
}
