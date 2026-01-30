//! システムトレイアイコン管理。
//!
//! `Shell_NotifyIconW` APIでシステムトレイにアイコンを表示する。
//! 隠しウィンドウでトレイアイコンメッセージを受信し、
//! コンテキストメニューを表示する。
//!
//! ## コンテキストメニュー
//!
//! - IME ON/OFF
//! - 設定...  → chamsae_settings.exe を起動
//! - バージョン情報
//! - 区切り線
//! - 終了 (Deactivate)

use std::cell::Cell;
use std::sync::Mutex;

use windows::core::{w, Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NOTIFYICONDATAW, NIF_ICON, NIF_MESSAGE, NIF_TIP,
    NIM_ADD, NIM_DELETE, NIM_MODIFY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW,
    DestroyMenu, DestroyWindow, GetCursorPos, RegisterClassW,
    SetForegroundWindow, TrackPopupMenu,
    CS_HREDRAW, CS_VREDRAW, HMENU,
    MF_SEPARATOR, MF_STRING,
    SW_HIDE, WINDOW_EX_STYLE,
    WM_COMMAND, WM_DESTROY, WM_LBUTTONUP, WM_RBUTTONUP, WM_USER,
    WNDCLASSW, WS_OVERLAPPEDWINDOW,
    TPM_BOTTOMALIGN, TPM_LEFTALIGN,
};

use super::icon;

/// トレイアイコンのコールバックメッセージ。
const WM_TRAYICON: u32 = WM_USER + 1;

/// トレイアイコンID。
const TRAY_ICON_ID: u32 = 1;

/// メニュー項目ID。
const IDM_TOGGLE: u16 = 1001;
const IDM_SETTINGS: u16 = 1002;
const IDM_ABOUT: u16 = 1003;

/// トレイアイコンのウィンドウクラス名。
const TRAY_CLASS_NAME: PCWSTR = w!("ChamsaeTrayIconClass");

/// トレイアイコンのツールチップ。
const TOOLTIP_ON: &str = "Chamsae IME (ON)";
const TOOLTIP_OFF: &str = "Chamsae IME (OFF)";

/// トレイアイコンのコールバック結果。
pub enum TrayAction {
    /// アクションなし。
    None,
    /// IME ON/OFF トグル。
    Toggle,
}

/// グローバルなトレイアイコンコールバック結果。
static TRAY_RESULT: Mutex<TrayAction> = Mutex::new(TrayAction::None);

/// グローバルなIME有効状態 (メニュー表示用)。
static IME_ENABLED: Mutex<bool> = Mutex::new(true);

/// システムトレイアイコン。
pub struct TrayIcon {
    hwnd: Cell<HWND>,
    class_registered: Cell<bool>,
    added: Cell<bool>,
}

impl TrayIcon {
    /// 新しいトレイアイコンを作成する。
    pub fn new() -> Self {
        Self {
            hwnd: Cell::new(HWND::default()),
            class_registered: Cell::new(false),
            added: Cell::new(false),
        }
    }

    /// ウィンドウクラスを登録する (初回のみ)。
    fn ensure_class_registered(&self) -> Result<()> {
        if self.class_registered.get() {
            return Ok(());
        }

        unsafe {
            let wc = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(tray_window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: windows::Win32::System::LibraryLoader::GetModuleHandleW(None)?
                    .into(),
                hIcon: Default::default(),
                hCursor: Default::default(),
                hbrBackground: Default::default(),
                lpszMenuName: PCWSTR::null(),
                lpszClassName: TRAY_CLASS_NAME,
            };

            let atom = RegisterClassW(&wc);
            if atom != 0 {
                self.class_registered.set(true);
            }
        }

        Ok(())
    }

    /// 隠しウィンドウを作成する (初回のみ)。
    fn ensure_window_created(&self) -> Result<()> {
        if !self.hwnd.get().0.is_null() {
            return Ok(());
        }

        self.ensure_class_registered()?;

        unsafe {
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                TRAY_CLASS_NAME,
                w!("Chamsae Tray"),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                1,
                1,
                None,
                None,
                windows::Win32::System::LibraryLoader::GetModuleHandleW(None)?,
                None,
            )?;

            // ウィンドウは非表示のまま。
            let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(hwnd, SW_HIDE);
            self.hwnd.set(hwnd);
        }

        Ok(())
    }

    /// トレイアイコンを追加する。
    pub fn add(&self, enabled: bool) -> Result<()> {
        self.ensure_window_created()?;

        let hwnd = self.hwnd.get();
        if hwnd.0.is_null() {
            return Ok(());
        }

        *IME_ENABLED.lock().unwrap() = enabled;

        let hicon = if enabled {
            icon::create_on_icon()
        } else {
            icon::create_off_icon()
        };

        let tooltip = if enabled { TOOLTIP_ON } else { TOOLTIP_OFF };

        unsafe {
            let mut nid = NOTIFYICONDATAW::default();
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd;
            nid.uID = TRAY_ICON_ID;
            nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
            nid.uCallbackMessage = WM_TRAYICON;
            nid.hIcon = hicon;

            // ツールチップ設定。
            let tip: Vec<u16> = tooltip.encode_utf16().collect();
            for (i, &ch) in tip.iter().enumerate() {
                if i < nid.szTip.len() - 1 {
                    nid.szTip[i] = ch;
                }
            }

            let _ = Shell_NotifyIconW(NIM_ADD, &nid);
            self.added.set(true);

            icon::destroy_icon(hicon);
        }

        Ok(())
    }

    /// トレイアイコンを更新する (ON/OFF状態変更時)。
    pub fn update(&self, enabled: bool) {
        if !self.added.get() {
            return;
        }

        *IME_ENABLED.lock().unwrap() = enabled;

        let hwnd = self.hwnd.get();
        if hwnd.0.is_null() {
            return;
        }

        let hicon = if enabled {
            icon::create_on_icon()
        } else {
            icon::create_off_icon()
        };

        let tooltip = if enabled { TOOLTIP_ON } else { TOOLTIP_OFF };

        unsafe {
            let mut nid = NOTIFYICONDATAW::default();
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd;
            nid.uID = TRAY_ICON_ID;
            nid.uFlags = NIF_ICON | NIF_TIP;
            nid.hIcon = hicon;

            let tip: Vec<u16> = tooltip.encode_utf16().collect();
            for (i, &ch) in tip.iter().enumerate() {
                if i < nid.szTip.len() - 1 {
                    nid.szTip[i] = ch;
                }
            }

            let _ = Shell_NotifyIconW(NIM_MODIFY, &nid);

            icon::destroy_icon(hicon);
        }
    }

    /// トレイアイコンを削除する。
    pub fn remove(&self) {
        if !self.added.get() {
            return;
        }

        let hwnd = self.hwnd.get();
        if hwnd.0.is_null() {
            return;
        }

        unsafe {
            let mut nid = NOTIFYICONDATAW::default();
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd;
            nid.uID = TRAY_ICON_ID;

            let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
        }

        self.added.set(false);
    }

    /// トレイアイコンのアクションをポーリングする。
    ///
    /// TextServiceが定期的に呼び出して、
    /// ユーザーがメニューから選択したアクションを取得する。
    pub fn poll_action(&self) -> TrayAction {
        let mut result = TRAY_RESULT.lock().unwrap();
        std::mem::replace(&mut *result, TrayAction::None)
    }

    /// 隠しウィンドウを破棄する。
    pub fn destroy(&self) {
        self.remove();

        let hwnd = self.hwnd.get();
        if !hwnd.0.is_null() {
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
            self.hwnd.set(HWND::default());
        }
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        self.destroy();
    }
}

/// コンテキストメニューを表示する。
unsafe fn show_context_menu(hwnd: HWND) {
    let hmenu = CreatePopupMenu().unwrap_or(HMENU::default());
    if hmenu.0.is_null() {
        return;
    }

    let enabled = *IME_ENABLED.lock().unwrap();
    let toggle_text = if enabled {
        w!("IME OFF")
    } else {
        w!("IME ON")
    };

    let _ = AppendMenuW(hmenu, MF_STRING, IDM_TOGGLE as usize, toggle_text);
    let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, None);
    let _ = AppendMenuW(hmenu, MF_STRING, IDM_SETTINGS as usize, w!("設定..."));
    let _ = AppendMenuW(hmenu, MF_STRING, IDM_ABOUT as usize, w!("バージョン情報"));

    let mut pt = windows::Win32::Foundation::POINT::default();
    let _ = GetCursorPos(&mut pt);

    let _ = SetForegroundWindow(hwnd);
    let _ = TrackPopupMenu(hmenu, TPM_LEFTALIGN | TPM_BOTTOMALIGN, pt.x, pt.y, 0, hwnd, None);
    let _ = DestroyMenu(hmenu);
}

/// 設定画面を起動する。
unsafe fn launch_settings() {
    // DLLと同じディレクトリの chamsae_settings.exe を起動。
    if let Some(dir) = crate::config::get_dll_directory() {
        let exe_path = dir.join("chamsae_settings.exe");
        if exe_path.exists() {
            let path_wide: Vec<u16> = exe_path.to_string_lossy()
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            windows::Win32::System::Threading::CreateProcessW(
                windows::core::PCWSTR(path_wide.as_ptr()),
                windows::core::PWSTR::null(),
                None,
                None,
                false,
                windows::Win32::System::Threading::PROCESS_CREATION_FLAGS(0),
                None,
                None,
                &windows::Win32::System::Threading::STARTUPINFOW::default(),
                &mut windows::Win32::System::Threading::PROCESS_INFORMATION::default(),
            ).ok();
        }
    }
}

/// バージョン情報ダイアログを表示する。
unsafe fn show_about() {
    let version = env!("CARGO_PKG_VERSION");
    let text = format!("Chamsae Hangul IME v{}\n\nローマ字→ハングル変換IME", version);
    let wide_text: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let wide_caption: Vec<u16> = "Chamsae IME\0".encode_utf16().collect();

    windows::Win32::UI::WindowsAndMessaging::MessageBoxW(
        None,
        windows::core::PCWSTR(wide_text.as_ptr()),
        windows::core::PCWSTR(wide_caption.as_ptr()),
        windows::Win32::UI::WindowsAndMessaging::MB_OK
            | windows::Win32::UI::WindowsAndMessaging::MB_ICONINFORMATION,
    );
}

/// トレイアイコン用隠しウィンドウのウィンドウプロシージャ。
extern "system" fn tray_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_TRAYICON => {
                let event = lparam.0 as u32;
                match event {
                    // 左クリック: トグル。
                    x if x == WM_LBUTTONUP => {
                        *TRAY_RESULT.lock().unwrap() = TrayAction::Toggle;
                    }
                    // 右クリック: コンテキストメニュー。
                    x if x == WM_RBUTTONUP => {
                        show_context_menu(hwnd);
                    }
                    _ => {}
                }
                LRESULT(0)
            }
            WM_COMMAND => {
                let id = (wparam.0 & 0xFFFF) as u16;
                match id {
                    IDM_TOGGLE => {
                        *TRAY_RESULT.lock().unwrap() = TrayAction::Toggle;
                    }
                    IDM_SETTINGS => {
                        launch_settings();
                    }
                    IDM_ABOUT => {
                        show_about();
                    }
                    _ => {}
                }
                LRESULT(0)
            }
            WM_DESTROY => LRESULT(0),
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
