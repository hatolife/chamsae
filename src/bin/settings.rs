#![cfg_attr(windows, windows_subsystem = "windows")]
//! Chamsae IME 設定画面。
//!
//! Win32ダイアログベースの設定GUI。
//! chamsae.jsonを読み込み、UIで設定を変更して保存する。
//!
//! ## UI構成
//!
//! - トグルキー設定 (キー選択 + 修飾キーチェックボックス)
//! - 言語プロファイル (日本語/韓国語)
//! - ユーザー辞書パス
//! - 保存/キャンセルボタン

#[cfg(not(windows))]
fn main() {
    eprintln!("chamsae_settings: Windowsでのみ実行可能です。");
    std::process::exit(1);
}

#[cfg(windows)]
fn main() {
    if let Err(e) = run_settings() {
        let msg = format!("エラー: {}", e);
        let wide: Vec<u16> = msg.encode_utf16().chain(std::iter::once(0)).collect();
        let caption: Vec<u16> = "Chamsae 設定\0".encode_utf16().collect();
        unsafe {
            windows::Win32::UI::WindowsAndMessaging::MessageBoxW(
                None,
                windows::core::PCWSTR(wide.as_ptr()),
                windows::core::PCWSTR(caption.as_ptr()),
                windows::Win32::UI::WindowsAndMessaging::MB_OK
                    | windows::Win32::UI::WindowsAndMessaging::MB_ICONERROR,
            );
        }
        std::process::exit(1);
    }
}

#[cfg(windows)]
fn run_settings() -> anyhow::Result<()> {
    use std::cell::RefCell;
    use std::rc::Rc;

    use windows::core::{w, PCWSTR};
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::Graphics::Gdi::{GetStockObject, HBRUSH, WHITE_BRUSH};
    use windows::Win32::UI::WindowsAndMessaging::*;

    // Per-Monitor DPI Awareness v2を有効化。
    unsafe {
        let _ = windows::Win32::UI::HiDpi::SetProcessDpiAwarenessContext(
            windows::Win32::UI::HiDpi::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        );
    }

    /// 設定ファイルJSONの構造。
    #[derive(serde::Serialize, serde::Deserialize)]
    struct ConfigJson {
        toggle_key: ToggleKeyJson,
        #[serde(default)]
        languages: LanguagesJson,
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        user_dict_path: Option<String>,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct ToggleKeyJson {
        key: String,
        shift: bool,
        ctrl: bool,
        alt: bool,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct LanguagesJson {
        japanese: bool,
        korean: bool,
    }

    impl Default for LanguagesJson {
        fn default() -> Self {
            Self {
                japanese: true,
                korean: false,
            }
        }
    }

    impl Default for ConfigJson {
        fn default() -> Self {
            Self {
                toggle_key: ToggleKeyJson {
                    key: "Space".to_string(),
                    shift: true,
                    ctrl: false,
                    alt: false,
                },
                languages: LanguagesJson::default(),
                user_dict_path: None,
            }
        }
    }

    // コントロールID。
    const IDC_KEY_COMBO: i32 = 1001;
    const IDC_SHIFT_CHECK: i32 = 1002;
    const IDC_CTRL_CHECK: i32 = 1003;
    const IDC_ALT_CHECK: i32 = 1004;
    const IDC_JA_CHECK: i32 = 1005;
    const IDC_KO_CHECK: i32 = 1006;
    const IDC_DICT_EDIT: i32 = 1007;
    const IDC_DICT_BROWSE: i32 = 1008;
    const IDC_DICT_OPEN: i32 = 1009;
    const IDC_SAVE: i32 = 1010;
    const IDC_CANCEL: i32 = 1011;

    const WINDOW_CLASS: PCWSTR = w!("ChamsaeSettingsClass");
    const WINDOW_TITLE: PCWSTR = w!("Chamsae 設定");

    // キー選択肢。
    const KEY_OPTIONS: &[&str] = &[
        "Space", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L",
        "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z",
        "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
    ];

    // 設定ファイルのパス (%APPDATA%\Chamsae\)。
    let config_dir = chamsae::config::get_config_directory()
        .unwrap_or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.to_path_buf()))
                .unwrap_or_else(|| std::path::PathBuf::from("."))
        });
    let config_path = config_dir.join("chamsae.json");

    // 設定を読み込み。
    let config: ConfigJson = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        ConfigJson::default()
    };

    // Rcで共有 (ウィンドウプロシージャからアクセス)。
    let config = Rc::new(RefCell::new(config));
    let config_path = Rc::new(config_path);

    // グローバルステート (WndProcから参照するため)。
    struct AppState {
        controls: Controls,
        config_path: std::path::PathBuf,
    }

    struct Controls {
        key_combo: HWND,
        shift_check: HWND,
        ctrl_check: HWND,
        alt_check: HWND,
        ja_check: HWND,
        ko_check: HWND,
        dict_edit: HWND,
    }

    // スタティック変数でウィンドウプロシージャにデータを渡す。
    static APP_STATE: std::sync::atomic::AtomicPtr<AppState> =
        std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());

    unsafe fn get_app_state() -> Option<&'static AppState> {
        let ptr = APP_STATE.load(std::sync::atomic::Ordering::Relaxed);
        if ptr.is_null() { None } else { Some(&*ptr) }
    }

    unsafe extern "system" fn settings_wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_COMMAND => {
                let id = (wparam.0 & 0xFFFF) as i32;
                match id {
                    IDC_SAVE => {
                        if let Some(state) = get_app_state() {
                            let controls = &state.controls;

                            // UIから値を読み取り。
                            let key = get_combo_text(controls.key_combo);
                            let shift = is_checked(controls.shift_check);
                            let ctrl = is_checked(controls.ctrl_check);
                            let alt = is_checked(controls.alt_check);
                            let japanese = is_checked(controls.ja_check);
                            let korean = is_checked(controls.ko_check);
                            let dict_path = get_window_text(controls.dict_edit);

                            let config = ConfigJson {
                                toggle_key: ToggleKeyJson {
                                    key,
                                    shift,
                                    ctrl,
                                    alt,
                                },
                                languages: LanguagesJson {
                                    japanese,
                                    korean,
                                },
                                user_dict_path: if dict_path.is_empty() {
                                    None
                                } else {
                                    Some(dict_path)
                                },
                            };

                            // JSONに保存。
                            if let Ok(json) = serde_json::to_string_pretty(&config) {
                                if std::fs::write(&state.config_path, json).is_ok() {
                                    MessageBoxW(
                                        hwnd,
                                        w!("設定を保存しました。\nトレイメニューの「設定の再読み込み」で反映できます。"),
                                        w!("Chamsae 設定"),
                                        MB_OK | MB_ICONINFORMATION,
                                    );
                                    PostQuitMessage(0);
                                } else {
                                    MessageBoxW(
                                        hwnd,
                                        w!("設定ファイルの保存に失敗しました。"),
                                        w!("エラー"),
                                        MB_OK | MB_ICONERROR,
                                    );
                                }
                            }
                        }
                        LRESULT(0)
                    }
                    IDC_CANCEL => {
                        PostQuitMessage(0);
                        LRESULT(0)
                    }
                    IDC_DICT_BROWSE => {
                        // ファイル選択ダイアログ。
                        if let Some(state) = get_app_state() {
                            if let Some(path) = open_file_dialog(hwnd) {
                                set_window_text(state.controls.dict_edit, &path);
                            }
                        }
                        LRESULT(0)
                    }
                    IDC_DICT_OPEN => {
                        // ユーザー辞書をメモ帳で開く。
                        if let Some(state) = get_app_state() {
                            let path = get_window_text(state.controls.dict_edit);
                            if !path.is_empty() {
                                open_in_notepad(&path);
                            } else {
                                // デフォルトパスで新規作成して開く。
                                if let Some(dir) = chamsae::config::get_config_directory() {
                                    let dict_path = dir.join("user_dict.json");
                                    if !dict_path.exists() {
                                        let default_json = r#"{
  "entries": {
  }
}"#;
                                        let _ = std::fs::write(&dict_path, default_json);
                                    }
                                    open_in_notepad(&dict_path.to_string_lossy());
                                }
                            }
                        }
                        LRESULT(0)
                    }
                    _ => DefWindowProcW(hwnd, msg, wparam, lparam),
                }
            }
            WM_CTLCOLORSTATIC | WM_CTLCOLORBTN => {
                // 子コントロールの背景色をウィンドウ背景と統一。
                LRESULT(GetStockObject(WHITE_BRUSH).0 as isize)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    unsafe fn is_checked(hwnd: HWND) -> bool {
        SendMessageW(hwnd, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 != 0
    }

    unsafe fn get_combo_text(hwnd: HWND) -> String {
        let idx = SendMessageW(hwnd, CB_GETCURSEL, WPARAM(0), LPARAM(0)).0;
        if idx < 0 {
            return "Space".to_string();
        }
        let len = SendMessageW(hwnd, CB_GETLBTEXTLEN, WPARAM(idx as usize), LPARAM(0)).0;
        if len <= 0 {
            return "Space".to_string();
        }
        let mut buf = vec![0u16; (len + 1) as usize];
        SendMessageW(
            hwnd,
            CB_GETLBTEXT,
            WPARAM(idx as usize),
            LPARAM(buf.as_mut_ptr() as isize),
        );
        String::from_utf16_lossy(&buf[..len as usize])
    }

    unsafe fn get_window_text(hwnd: HWND) -> String {
        let len = GetWindowTextLengthW(hwnd);
        if len == 0 {
            return String::new();
        }
        let mut buf = vec![0u16; (len + 1) as usize];
        GetWindowTextW(hwnd, &mut buf);
        String::from_utf16_lossy(&buf[..len as usize])
    }

    unsafe fn set_window_text(hwnd: HWND, text: &str) {
        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let _ = SetWindowTextW(hwnd, PCWSTR(wide.as_ptr()));
    }

    unsafe fn open_file_dialog(owner: HWND) -> Option<String> {
        use windows::Win32::UI::Controls::Dialogs::*;

        let mut file_buf = [0u16; 260];
        let mut ofn = OPENFILENAMEW::default();
        ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
        ofn.hwndOwner = owner;
        let filter: Vec<u16> = "JSON Files (*.json)\0*.json\0All Files (*.*)\0*.*\0\0"
            .encode_utf16()
            .collect();
        ofn.lpstrFilter = PCWSTR(filter.as_ptr());
        ofn.lpstrFile = windows::core::PWSTR(file_buf.as_mut_ptr());
        ofn.nMaxFile = file_buf.len() as u32;
        ofn.Flags = OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST;

        if GetOpenFileNameW(&mut ofn).as_bool() {
            let len = file_buf.iter().position(|&c| c == 0).unwrap_or(file_buf.len());
            Some(String::from_utf16_lossy(&file_buf[..len]))
        } else {
            None
        }
    }

    unsafe fn open_in_notepad(path: &str) {
        let cmd = format!("notepad.exe \"{}\"\0", path);
        let wide: Vec<u16> = cmd.encode_utf16().collect();
        let mut si = windows::Win32::System::Threading::STARTUPINFOW::default();
        si.cb = std::mem::size_of::<windows::Win32::System::Threading::STARTUPINFOW>() as u32;
        let mut pi = windows::Win32::System::Threading::PROCESS_INFORMATION::default();

        let _ = windows::Win32::System::Threading::CreateProcessW(
            None,
            windows::core::PWSTR(wide.as_ptr() as *mut _),
            None,
            None,
            false,
            windows::Win32::System::Threading::PROCESS_CREATION_FLAGS(0),
            None,
            None,
            &si,
            &mut pi,
        );
    }

    // メインウィンドウ作成。
    unsafe {
        let hinstance = windows::Win32::System::LibraryLoader::GetModuleHandleW(None)?;

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(settings_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance.into(),
            hIcon: Default::default(),
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hbrBackground: HBRUSH(GetStockObject(WHITE_BRUSH).0),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: WINDOW_CLASS,
        };

        let atom = RegisterClassW(&wc);
        if atom == 0 {
            anyhow::bail!("ウィンドウクラスの登録に失敗しました。");
        }

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            WINDOW_CLASS,
            WINDOW_TITLE,
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            420,
            380,
            None,
            None,
            hinstance,
            None,
        )?;

        let cfg = config.borrow();

        // === トグルキー設定 ===
        // ラベル: トグルキー設定
        create_static(hwnd, hinstance.into(), "トグルキー設定", 15, 15, 200, 20);

        // キー: [ComboBox]
        create_static(hwnd, hinstance.into(), "キー:", 25, 40, 40, 20);
        let key_combo = create_combo(hwnd, hinstance.into(), IDC_KEY_COMBO, 70, 38, 100, 200);

        // ComboBoxにキーオプションを追加。
        for &key in KEY_OPTIONS {
            let wide: Vec<u16> = key.encode_utf16().chain(std::iter::once(0)).collect();
            SendMessageW(
                key_combo,
                CB_ADDSTRING,
                WPARAM(0),
                LPARAM(wide.as_ptr() as isize),
            );
        }

        // 現在の設定を選択。
        let current_key = &cfg.toggle_key.key;
        if let Some(idx) = KEY_OPTIONS.iter().position(|&k| k == current_key) {
            SendMessageW(key_combo, CB_SETCURSEL, WPARAM(idx), LPARAM(0));
        }

        // 修飾キーチェックボックス。
        let shift_check = create_checkbox(
            hwnd,
            hinstance.into(),
            IDC_SHIFT_CHECK,
            "Shift",
            25,
            68,
            70,
            20,
        );
        let ctrl_check = create_checkbox(
            hwnd,
            hinstance.into(),
            IDC_CTRL_CHECK,
            "Ctrl",
            100,
            68,
            70,
            20,
        );
        let alt_check = create_checkbox(
            hwnd,
            hinstance.into(),
            IDC_ALT_CHECK,
            "Alt",
            175,
            68,
            70,
            20,
        );

        if cfg.toggle_key.shift {
            SendMessageW(shift_check, BM_SETCHECK, WPARAM(1), LPARAM(0));
        }
        if cfg.toggle_key.ctrl {
            SendMessageW(ctrl_check, BM_SETCHECK, WPARAM(1), LPARAM(0));
        }
        if cfg.toggle_key.alt {
            SendMessageW(alt_check, BM_SETCHECK, WPARAM(1), LPARAM(0));
        }

        // === 言語プロファイル ===
        create_static(hwnd, hinstance.into(), "言語プロファイル", 15, 105, 200, 20);

        let ja_check = create_checkbox(
            hwnd,
            hinstance.into(),
            IDC_JA_CHECK,
            "日本語",
            25,
            130,
            100,
            20,
        );
        let ko_check = create_checkbox(
            hwnd,
            hinstance.into(),
            IDC_KO_CHECK,
            "韓国語",
            130,
            130,
            100,
            20,
        );

        if cfg.languages.japanese {
            SendMessageW(ja_check, BM_SETCHECK, WPARAM(1), LPARAM(0));
        }
        if cfg.languages.korean {
            SendMessageW(ko_check, BM_SETCHECK, WPARAM(1), LPARAM(0));
        }

        // === ユーザー辞書 ===
        create_static(hwnd, hinstance.into(), "ユーザー辞書", 15, 170, 200, 20);

        create_static(hwnd, hinstance.into(), "パス:", 25, 197, 40, 20);
        let dict_edit = create_edit(
            hwnd,
            hinstance.into(),
            IDC_DICT_EDIT,
            70,
            195,
            220,
            22,
        );
        if let Some(ref path) = cfg.user_dict_path {
            set_window_text(dict_edit, path);
        }

        create_button(hwnd, hinstance.into(), IDC_DICT_BROWSE, "参照...", 298, 195, 75, 24);
        create_button(hwnd, hinstance.into(), IDC_DICT_OPEN, "辞書を編集...", 25, 225, 120, 26);

        // === 保存/キャンセルボタン ===
        create_button(hwnd, hinstance.into(), IDC_SAVE, "保存", 210, 300, 85, 30);
        create_button(hwnd, hinstance.into(), IDC_CANCEL, "キャンセル", 303, 300, 85, 30);

        drop(cfg);

        // AppStateを作成しグローバルに設定。
        let state = Box::new(AppState {
            controls: Controls {
                key_combo,
                shift_check,
                ctrl_check,
                alt_check,
                ja_check,
                ko_check,
                dict_edit,
            },
            config_path: (*config_path).clone(),
        });
        APP_STATE.store(Box::into_raw(state), std::sync::atomic::Ordering::Relaxed);

        let _ = ShowWindow(hwnd, SW_SHOW);

        // メッセージループ。
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // クリーンアップ。
        let ptr = APP_STATE.swap(std::ptr::null_mut(), std::sync::atomic::Ordering::Relaxed);
        if !ptr.is_null() {
            let _ = Box::from_raw(ptr);
        }
    }

    Ok(())
}

// === Win32コントロール作成ヘルパー (Windows専用) ===

#[cfg(windows)]
unsafe fn create_static(
    parent: windows::Win32::Foundation::HWND,
    hinstance: windows::Win32::Foundation::HINSTANCE,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> windows::Win32::Foundation::HWND {
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    windows::Win32::UI::WindowsAndMessaging::CreateWindowExW(
        windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE::default(),
        windows::core::w!("STATIC"),
        windows::core::PCWSTR(wide.as_ptr()),
        windows::Win32::UI::WindowsAndMessaging::WS_CHILD
            | windows::Win32::UI::WindowsAndMessaging::WS_VISIBLE,
        x,
        y,
        width,
        height,
        parent,
        None,
        hinstance,
        None,
    )
    .unwrap_or_default()
}

#[cfg(windows)]
unsafe fn create_combo(
    parent: windows::Win32::Foundation::HWND,
    hinstance: windows::Win32::Foundation::HINSTANCE,
    id: i32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> windows::Win32::Foundation::HWND {
    windows::Win32::UI::WindowsAndMessaging::CreateWindowExW(
        windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE::default(),
        windows::core::w!("COMBOBOX"),
        windows::core::w!(""),
        windows::Win32::UI::WindowsAndMessaging::WS_CHILD
            | windows::Win32::UI::WindowsAndMessaging::WS_VISIBLE
            | windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE(
                windows::Win32::UI::WindowsAndMessaging::CBS_DROPDOWNLIST as u32,
            ),
        x,
        y,
        width,
        height,
        parent,
        windows::Win32::UI::WindowsAndMessaging::HMENU(id as *mut _),
        hinstance,
        None,
    )
    .unwrap_or_default()
}

#[cfg(windows)]
unsafe fn create_checkbox(
    parent: windows::Win32::Foundation::HWND,
    hinstance: windows::Win32::Foundation::HINSTANCE,
    id: i32,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> windows::Win32::Foundation::HWND {
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    windows::Win32::UI::WindowsAndMessaging::CreateWindowExW(
        windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE::default(),
        windows::core::w!("BUTTON"),
        windows::core::PCWSTR(wide.as_ptr()),
        windows::Win32::UI::WindowsAndMessaging::WS_CHILD
            | windows::Win32::UI::WindowsAndMessaging::WS_VISIBLE
            | windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE(
                windows::Win32::UI::WindowsAndMessaging::BS_AUTOCHECKBOX as u32,
            ),
        x,
        y,
        width,
        height,
        parent,
        windows::Win32::UI::WindowsAndMessaging::HMENU(id as *mut _),
        hinstance,
        None,
    )
    .unwrap_or_default()
}

#[cfg(windows)]
unsafe fn create_edit(
    parent: windows::Win32::Foundation::HWND,
    hinstance: windows::Win32::Foundation::HINSTANCE,
    id: i32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> windows::Win32::Foundation::HWND {
    windows::Win32::UI::WindowsAndMessaging::CreateWindowExW(
        windows::Win32::UI::WindowsAndMessaging::WS_EX_CLIENTEDGE,
        windows::core::w!("EDIT"),
        windows::core::w!(""),
        windows::Win32::UI::WindowsAndMessaging::WS_CHILD
            | windows::Win32::UI::WindowsAndMessaging::WS_VISIBLE
            | windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE(
                windows::Win32::UI::WindowsAndMessaging::ES_AUTOHSCROLL as u32,
            ),
        x,
        y,
        width,
        height,
        parent,
        windows::Win32::UI::WindowsAndMessaging::HMENU(id as *mut _),
        hinstance,
        None,
    )
    .unwrap_or_default()
}

#[cfg(windows)]
unsafe fn create_button(
    parent: windows::Win32::Foundation::HWND,
    hinstance: windows::Win32::Foundation::HINSTANCE,
    id: i32,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> windows::Win32::Foundation::HWND {
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    windows::Win32::UI::WindowsAndMessaging::CreateWindowExW(
        windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE::default(),
        windows::core::w!("BUTTON"),
        windows::core::PCWSTR(wide.as_ptr()),
        windows::Win32::UI::WindowsAndMessaging::WS_CHILD
            | windows::Win32::UI::WindowsAndMessaging::WS_VISIBLE
            | windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE(
                windows::Win32::UI::WindowsAndMessaging::BS_PUSHBUTTON as u32,
            ),
        x,
        y,
        width,
        height,
        parent,
        windows::Win32::UI::WindowsAndMessaging::HMENU(id as *mut _),
        hinstance,
        None,
    )
    .unwrap_or_default()
}
