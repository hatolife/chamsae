//! ファイルベースロガー。
//!
//! DLLはstdout/stderrが使えないため、ファイルに出力する。
//! `DLL_DIR/chamsae.log` に追記モードで書き込む。
//! `log` クレートの `Log` トレイトを実装し、グローバルロガーとして設定する。

use log::{Level, Log, Metadata, Record};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// ファイルロガー。
struct FileLogger {
    path: Mutex<PathBuf>,
}

impl Log for FileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let path = self.path.lock().unwrap();
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&*path) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let _ = writeln!(
                file,
                "[{} {:5}] {}",
                now,
                record.level(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

/// ロガーを初期化する。
///
/// `log_dir` 内の `chamsae.log` にログを出力する。
/// 二重初期化時は何もしない。
pub fn init(log_dir: &std::path::Path) {
    let path = log_dir.join("chamsae.log");
    let logger = FileLogger {
        path: Mutex::new(path),
    };

    // Box::leak で 'static 参照を作成。DLLライフタイム中有効。
    let logger: &'static FileLogger = Box::leak(Box::new(logger));
    if log::set_logger(logger).is_ok() {
        log::set_max_level(log::LevelFilter::Info);
    }
}
