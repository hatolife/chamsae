//! Win32ウィンドウ作成。
//!
//! Win32 APIを使った基本的なウィンドウ作成機能。
//! IMEの候補ウィンドウや設定画面の基礎となる。
//!
//! ## Win32ウィンドウ作成の流れ
//!
//! 1. WNDCLASSW構造体でウィンドウクラスを定義
//! 2. RegisterClassWでクラスを登録
//! 3. CreateWindowExWでウィンドウを作成
//! 4. ShowWindowで表示
//! 5. メッセージループ (GetMessage/TranslateMessage/DispatchMessage)
//! 6. WM_DESTROYでPostQuitMessageを呼び終了

use windows::core::{w, Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{GetStockObject, WHITE_BRUSH};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW,
    LoadCursorW, PostQuitMessage, RegisterClassW, ShowWindow, TranslateMessage,
    CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, IDC_ARROW, MSG,
    SW_SHOW, WINDOW_EX_STYLE, WM_DESTROY, WNDCLASSW, WS_OVERLAPPEDWINDOW,
};

/// ウィンドウクラス名。
const WINDOW_CLASS_NAME: PCWSTR = w!("ChamsaeWindowClass");

/// デフォルトウィンドウタイトル。
const WINDOW_TITLE: PCWSTR = w!("Chamsae IME Test Window");

/// テスト用ウィンドウを作成して表示する。
///
/// Win32 APIの基本的なウィンドウ作成フローを実行する。
/// ウィンドウクラスの登録→ウィンドウ作成→表示→メッセージループ。
pub fn create_test_window() -> Result<()> {
    unsafe {
        // 1. ウィンドウクラスの登録。
        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: windows::Win32::System::LibraryLoader::GetModuleHandleW(None)?
                .into(),
            hIcon: Default::default(),
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hbrBackground: GetStockObject(WHITE_BRUSH).into(),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: WINDOW_CLASS_NAME,
        };

        let atom = RegisterClassW(&wc);
        if atom == 0 {
            return Err(windows::core::Error::from_win32());
        }

        // 2. ウィンドウの作成。
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            WINDOW_CLASS_NAME,
            WINDOW_TITLE,
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            640,
            480,
            None,
            None,
            wc.hInstance,
            None,
        )?;

        // 3. ウィンドウの表示。
        let _ = ShowWindow(hwnd, SW_SHOW);

        // 4. メッセージループ。
        run_message_loop();
    }

    Ok(())
}

/// メッセージループを実行する。
///
/// Win32アプリケーションの心臓部。
/// ウィンドウメッセージを取得し、翻訳・ディスパッチする。
/// WM_QUITメッセージを受け取るまでループする。
fn run_message_loop() {
    unsafe {
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

/// ウィンドウプロシージャ (コールバック)。
///
/// ウィンドウに送られるメッセージを処理する。
/// 処理しないメッセージはDefWindowProcWに委譲する。
///
/// ## 処理するメッセージ
///
/// - WM_DESTROY: ウィンドウ破棄時にPostQuitMessageでメッセージループを終了
extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
