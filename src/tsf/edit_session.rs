//! EditSession実装。
//!
//! TSFではテキストの変更はEditSessionを通じて行う。
//! RequestEditSessionでセッションを要求し、
//! TSFが許可するとDoEditSessionが呼ばれる。

use std::sync::{Arc, Mutex};

use windows::core::{implement, Interface, Result};
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::TextServices::{
    ITfComposition, ITfCompositionSink, ITfContext, ITfContextComposition,
    ITfEditSession, ITfEditSession_Impl, ITfInsertAtSelection,
    TF_IAS_QUERYONLY,
};

/// EditSessionのアクション。
pub enum EditAction {
    /// コンポジションを更新 (開始 or テキスト変更)。
    Update(Vec<u16>),
    /// コンポジションを確定 (テキスト確定して終了)。
    Commit,
    /// コンポジションをキャンセル (テキスト削除して終了)。
    Cancel,
}

/// キャレット位置情報 (スクリーン座標)。
#[derive(Clone, Default)]
pub struct CaretPos {
    pub x: i32,
    pub y: i32,
    pub height: i32,
}

/// TSF EditSession。
///
/// DoEditSessionでコンポジションの開始・更新・終了を行う。
/// TextServiceとコンポジション状態を共有するためArc<Mutex>を使用。
#[implement(ITfEditSession)]
pub struct EditSession {
    context: ITfContext,
    composition: Arc<Mutex<Option<ITfComposition>>>,
    composition_sink: ITfCompositionSink,
    action: EditAction,
    /// キャレット位置 (Update時にEditSessionが書き込む)。
    caret_pos: Arc<Mutex<CaretPos>>,
}

impl EditSession {
    /// 新しいEditSessionを作成する。
    pub fn new(
        context: ITfContext,
        composition: Arc<Mutex<Option<ITfComposition>>>,
        composition_sink: ITfCompositionSink,
        action: EditAction,
        caret_pos: Arc<Mutex<CaretPos>>,
    ) -> Self {
        Self {
            context,
            composition,
            composition_sink,
            action,
            caret_pos,
        }
    }

    /// EditSessionをITfEditSessionインターフェースに変換する。
    pub fn into_interface(self) -> ITfEditSession {
        self.into()
    }
}

impl ITfEditSession_Impl for EditSession_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        match &self.action {
            EditAction::Update(text) => self.do_update(ec, text),
            EditAction::Commit => self.do_end_composition(ec),
            EditAction::Cancel => self.do_cancel(ec),
        }
    }
}

impl EditSession_Impl {
    /// コンポジションを開始または更新する。
    fn do_update(&self, ec: u32, text: &[u16]) -> Result<()> {
        let mut comp_guard = self.composition.lock().unwrap();

        if comp_guard.is_none() {
            // 新しいコンポジションを開始。
            let insert: ITfInsertAtSelection = self.context.cast()?;
            let range = unsafe {
                insert.InsertTextAtSelection(ec, TF_IAS_QUERYONLY, &[])?
            };

            let ctx_comp: ITfContextComposition = self.context.cast()?;
            let new_comp = unsafe {
                ctx_comp.StartComposition(ec, &range, &self.composition_sink)?
            };
            *comp_guard = Some(new_comp);
        }

        // コンポジションテキストを更新。
        if let Some(comp) = comp_guard.as_ref() {
            unsafe {
                let range = comp.GetRange()?;
                range.SetText(ec, 0, text)?;

                // キャレット位置を取得。
                if let Ok(view) = self.context.GetActiveView() {
                    let mut rc = RECT::default();
                    let mut clipped = windows::Win32::Foundation::BOOL::default();
                    if view.GetTextExt(ec, &range, &mut rc, &mut clipped).is_ok() {
                        let mut pos = self.caret_pos.lock().unwrap();
                        pos.x = rc.left;
                        pos.y = rc.bottom;
                        pos.height = rc.bottom - rc.top;
                    }
                }
            }
        }

        Ok(())
    }

    /// コンポジションを確定する (テキストはそのまま残す)。
    fn do_end_composition(&self, ec: u32) -> Result<()> {
        let mut comp_guard = self.composition.lock().unwrap();
        if let Some(comp) = comp_guard.take() {
            unsafe {
                comp.EndComposition(ec)?;
            }
        }
        Ok(())
    }

    /// コンポジションをキャンセルする (テキストを削除)。
    fn do_cancel(&self, ec: u32) -> Result<()> {
        let mut comp_guard = self.composition.lock().unwrap();
        if let Some(comp) = comp_guard.take() {
            unsafe {
                let range = comp.GetRange()?;
                range.SetText(ec, 0, &[])?;
                comp.EndComposition(ec)?;
            }
        }
        Ok(())
    }
}
