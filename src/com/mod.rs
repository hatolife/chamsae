//! COMモジュール。
//!
//! Windows COM (Component Object Model) の基礎実装。
//! DLLとしてCOMオブジェクトを公開するために必要な機能を提供する。
//!
//! ## 構成
//!
//! - `dll_module`: DLLのモジュールハンドルと参照カウント管理
//! - `class_factory`: IClassFactory実装 (COMオブジェクト生成)

pub mod class_factory;
pub mod dll_module;
