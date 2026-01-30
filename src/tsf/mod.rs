//! TSF (Text Services Framework) モジュール。
//!
//! Windows TSFフレームワークのIME実装。
//! TextServiceがITfTextInputProcessorExとして動作し、
//! キー入力を受け取ってハングル変換を行う。
//!
//! ## 構成
//!
//! - `text_service`: TextService本体 (COM実装)
//! - `registration`: TSFプロファイル・カテゴリ登録
//! - `candidate_window`: 候補ウィンドウ (変換結果表示)
//! - `tray_icon`: システムトレイアイコン
//! - `icon`: アイコンリソース (GDI動的生成)

pub mod candidate_window;
pub mod edit_session;
pub mod icon;
pub mod key_handler;
pub mod registration;
pub mod text_service;
pub mod tray_icon;
