//! Win32ウィンドウ作成テスト。
//!
//! Win32 APIの基本的なウィンドウ作成を確認するテストバイナリ。
//! Milestone 2.2の動作確認用。
//!
//! ## 使い方
//!
//! ```bat
//! cargo run --bin window_test
//! ```

#[cfg(not(windows))]
fn main() {
    eprintln!("window_test: Windowsでのみ実行可能です。");
    std::process::exit(1);
}

#[cfg(windows)]
fn main() {
    println!("Chamsae IME - Win32ウィンドウテスト");
    println!("ウィンドウを作成しています...");

    match hangul_ime::win32::window::create_test_window() {
        Ok(()) => {
            println!("ウィンドウが正常に閉じられました。");
        }
        Err(e) => {
            eprintln!("エラー: {}", e);
            std::process::exit(1);
        }
    }
}
