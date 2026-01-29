//! TextService実装。
//!
//! TSFフレームワークのテキスト入力プロセッサ。
//! ITfTextInputProcessorExを実装し、COMオブジェクトとしてTSFに提供される。
//!
//! ## インターフェース
//!
//! - ITfTextInputProcessorEx: TSFへの登録/解除 (Activate/Deactivate)
//! - ITfKeyEventSink: キーイベントの受信
//! - ITfCompositionSink: コンポジションの終了通知

use std::cell::{Cell, RefCell};
use std::sync::{Arc, Mutex};

use windows::core::{implement, Interface, IUnknownImpl, Result, GUID};
use windows::Win32::Foundation::{BOOL, FALSE, LPARAM, TRUE, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState;
use windows::Win32::UI::TextServices::{
    ITfComposition, ITfCompositionSink, ITfCompositionSink_Impl, ITfContext,
    ITfEditSession, ITfKeyEventSink, ITfKeyEventSink_Impl, ITfKeystrokeMgr,
    ITfTextInputProcessor_Impl, ITfTextInputProcessorEx,
    ITfTextInputProcessorEx_Impl, ITfThreadMgr,
    TF_ES_READWRITE, TF_ES_SYNC,
};

use crate::com::dll_module;
use crate::config::Config;
use crate::hangul::HangulConverter;
use crate::tsf::edit_session::{EditAction, EditSession};
use crate::tsf::key_handler;

/// Chamsae TextService。
///
/// TSFのテキスト入力プロセッサとして機能する。
/// STA (Single-Threaded Apartment) で動作するため、RefCellを使用。
#[implement(ITfTextInputProcessorEx, ITfKeyEventSink, ITfCompositionSink)]
pub struct TextService {
    /// TSFスレッドマネージャへの参照。
    thread_mgr: RefCell<Option<ITfThreadMgr>>,
    /// TSFから割り当てられたクライアントID。
    client_id: Cell<u32>,
    /// ローマ字入力バッファ。
    roman_buffer: RefCell<String>,
    /// アクティブなコンポジション (TextServiceとEditSessionで共有)。
    composition: Arc<Mutex<Option<ITfComposition>>>,
    /// ハングル変換器。
    converter: HangulConverter,
    /// IME設定。
    config: Config,
    /// IME有効状態。falseの場合はすべてのキーをパススルーする。
    enabled: Cell<bool>,
}

impl TextService {
    /// 新しいTextServiceを作成する。
    pub fn new() -> Self {
        dll_module::increment_object_count();
        Self {
            thread_mgr: RefCell::new(None),
            client_id: Cell::new(0),
            roman_buffer: RefCell::new(String::new()),
            composition: Arc::new(Mutex::new(None)),
            converter: HangulConverter::new(),
            config: Config::load_from_dll(),
            enabled: Cell::new(true),
        }
    }
}

impl Drop for TextService {
    fn drop(&mut self) {
        dll_module::decrement_object_count();
    }
}

// === ITfTextInputProcessor ===

impl ITfTextInputProcessor_Impl for TextService_Impl {
    fn Activate(&self, ptim: Option<&ITfThreadMgr>, tid: u32) -> Result<()> {
        self.ActivateEx(ptim, tid, 0)
    }

    fn Deactivate(&self) -> Result<()> {
        // キーイベントシンクの登録解除。
        let thread_mgr = self.thread_mgr.borrow();
        if let Some(mgr) = thread_mgr.as_ref() {
            unsafe {
                let keystroke_mgr: ITfKeystrokeMgr = mgr.cast()?;
                let _ = keystroke_mgr.UnadviseKeyEventSink(self.client_id.get());
            }
        }
        drop(thread_mgr);

        // 状態をクリア。
        self.roman_buffer.borrow_mut().clear();
        *self.composition.lock().unwrap() = None;
        *self.thread_mgr.borrow_mut() = None;
        self.client_id.set(0);
        self.enabled.set(true);

        Ok(())
    }
}

// === ITfTextInputProcessorEx ===

impl ITfTextInputProcessorEx_Impl for TextService_Impl {
    fn ActivateEx(
        &self,
        ptim: Option<&ITfThreadMgr>,
        tid: u32,
        _dwflags: u32,
    ) -> Result<()> {
        let mgr = match ptim {
            Some(m) => m,
            None => return Ok(()),
        };

        self.client_id.set(tid);
        *self.thread_mgr.borrow_mut() = Some(mgr.clone());

        // キーイベントシンクを登録。
        unsafe {
            let keystroke_mgr: ITfKeystrokeMgr = mgr.cast()?;
            let sink: ITfKeyEventSink = self.to_interface();
            keystroke_mgr.AdviseKeyEventSink(tid, &sink, TRUE)?;
        }

        Ok(())
    }
}

// === ITfKeyEventSink ===

impl ITfKeyEventSink_Impl for TextService_Impl {
    fn OnSetFocus(&self, _fforeground: BOOL) -> Result<()> {
        Ok(())
    }

    fn OnTestKeyDown(
        &self,
        _pic: Option<&ITfContext>,
        wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Result<BOOL> {
        let vk = wparam.0 as u32;

        // トグルキー (Shift+Space) は常に横取りする。
        if self.is_toggle_key(vk) {
            return Ok(TRUE);
        }

        // IME無効時はすべてのキーをパススルー。
        if !self.enabled.get() {
            return Ok(FALSE);
        }

        // Ctrl/Alt押下中はハングルキーを捕捉しない。
        if self.is_modifier_held() {
            // バッファ非空なら自動確定用に横取り。
            if !self.roman_buffer.borrow().is_empty() {
                return Ok(TRUE);
            }
            return Ok(FALSE);
        }

        if key_handler::is_hangul_key(vk) {
            return Ok(TRUE);
        }

        if key_handler::is_control_key(vk) && !self.roman_buffer.borrow().is_empty() {
            return Ok(TRUE);
        }

        // バッファ非空時、未対応キーも自動確定用に横取り。
        if !self.roman_buffer.borrow().is_empty() {
            return Ok(TRUE);
        }

        Ok(FALSE)
    }

    fn OnTestKeyUp(
        &self,
        _pic: Option<&ITfContext>,
        _wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Result<BOOL> {
        Ok(FALSE)
    }

    fn OnKeyDown(
        &self,
        pic: Option<&ITfContext>,
        wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Result<BOOL> {
        let vk = wparam.0 as u32;
        let context = match pic {
            Some(c) => c,
            None => return Ok(FALSE),
        };

        // トグルキー (Shift+Space) でIMEのON/OFFを切り替え。
        if self.is_toggle_key(vk) {
            // コンポジション中なら確定してからトグル。
            if !self.roman_buffer.borrow().is_empty() {
                self.request_edit_session(context, EditAction::Commit)?;
                self.roman_buffer.borrow_mut().clear();
            }
            self.enabled.set(!self.enabled.get());
            return Ok(TRUE);
        }

        // IME無効時はすべてのキーをパススルー。
        if !self.enabled.get() {
            return Ok(FALSE);
        }

        // Ctrl/Alt押下中はハングルキーを処理しない。
        if self.is_modifier_held() {
            // バッファ非空なら自動確定してパススルー。
            if !self.roman_buffer.borrow().is_empty() {
                self.request_edit_session(context, EditAction::Commit)?;
                self.roman_buffer.borrow_mut().clear();
            }
            return Ok(FALSE);
        }

        // ローマ字キー → バッファに追加してコンポジション更新。
        if let Some(ch) = key_handler::vk_to_char(vk) {
            self.roman_buffer.borrow_mut().push(ch);
            self.update_composition(context)?;
            return Ok(TRUE);
        }

        // 制御キーの処理 (バッファが空でない場合のみ)。
        if !self.roman_buffer.borrow().is_empty() {
            match vk {
                key_handler::VK_BACK => {
                    self.roman_buffer.borrow_mut().pop();
                    if self.roman_buffer.borrow().is_empty() {
                        self.request_edit_session(context, EditAction::Cancel)?;
                    } else {
                        self.update_composition(context)?;
                    }
                }
                key_handler::VK_RETURN => {
                    self.request_edit_session(context, EditAction::Commit)?;
                    self.roman_buffer.borrow_mut().clear();
                }
                key_handler::VK_ESCAPE => {
                    self.request_edit_session(context, EditAction::Cancel)?;
                    self.roman_buffer.borrow_mut().clear();
                }
                key_handler::VK_SPACE => {
                    self.roman_buffer.borrow_mut().push(' ');
                    self.update_composition(context)?;
                }
                // 未対応キー → 自動確定してパススルー。
                _ => {
                    self.request_edit_session(context, EditAction::Commit)?;
                    self.roman_buffer.borrow_mut().clear();
                    return Ok(FALSE);
                }
            }
            return Ok(TRUE);
        }

        Ok(FALSE)
    }

    fn OnKeyUp(
        &self,
        _pic: Option<&ITfContext>,
        _wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Result<BOOL> {
        Ok(FALSE)
    }

    fn OnPreservedKey(
        &self,
        _pic: Option<&ITfContext>,
        _rguid: *const GUID,
    ) -> Result<BOOL> {
        Ok(FALSE)
    }
}

// === ITfCompositionSink ===

impl ITfCompositionSink_Impl for TextService_Impl {
    fn OnCompositionTerminated(
        &self,
        _ecwrite: u32,
        _pcomposition: Option<&ITfComposition>,
    ) -> Result<()> {
        self.roman_buffer.borrow_mut().clear();
        *self.composition.lock().unwrap() = None;
        Ok(())
    }
}

// === ヘルパーメソッド ===

impl TextService_Impl {
    /// 設定されたトグルキーか判定する。
    fn is_toggle_key(&self, vk: u32) -> bool {
        let tk = &self.config.toggle_key;
        if vk != tk.vk {
            return false;
        }
        let shift_held = unsafe { GetKeyState(key_handler::VK_SHIFT as i32) } < 0;
        let ctrl_held = unsafe { GetKeyState(key_handler::VK_CONTROL as i32) } < 0;
        let alt_held = unsafe { GetKeyState(key_handler::VK_MENU as i32) } < 0;
        shift_held == tk.shift && ctrl_held == tk.ctrl && alt_held == tk.alt
    }

    /// Ctrl/Altが押下されているか判定する。
    fn is_modifier_held(&self) -> bool {
        unsafe {
            GetKeyState(key_handler::VK_CONTROL as i32) < 0
                || GetKeyState(key_handler::VK_MENU as i32) < 0
        }
    }

    /// バッファの内容をハングルに変換してコンポジションを更新する。
    fn update_composition(&self, context: &ITfContext) -> Result<()> {
        let buffer = self.roman_buffer.borrow();
        let hangul = self.converter.convert(&buffer);
        let text: Vec<u16> = hangul.encode_utf16().collect();
        self.request_edit_session(context, EditAction::Update(text))
    }

    /// EditSessionを作成してTSFに要求する。
    fn request_edit_session(&self, context: &ITfContext, action: EditAction) -> Result<()> {
        let session = EditSession::new(
            context.clone(),
            self.composition.clone(),
            self.to_interface::<ITfCompositionSink>(),
            action,
        );

        let session_iface: ITfEditSession = session.into_interface();
        unsafe {
            let _hr = context.RequestEditSession(
                self.client_id.get(),
                &session_iface,
                TF_ES_SYNC | TF_ES_READWRITE,
            )?;
        }

        Ok(())
    }
}
