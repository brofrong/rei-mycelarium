//! Panics go to `~/Library/Logs/Mycelarium Rust/panic.log` (appended: time, thread, message,
//! location, backtrace) as well as stderr: an app started from Finder has no stderr to read.

use std::io::Write;
use std::path::PathBuf;

/// `~/Library/Logs/Mycelarium Rust/panic.log`, or `MYCO_PANIC_LOG` when set.
pub fn log_path() -> PathBuf {
    if let Some(p) = std::env::var_os("MYCO_PANIC_LOG").filter(|p| !p.is_empty()) {
        return PathBuf::from(p);
    }
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    home.join("Library/Logs/Mycelarium Rust/panic.log")
}

/// The report written for one panic.
pub fn report(info: &std::panic::PanicHookInfo, backtrace: &std::backtrace::Backtrace) -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let thread = std::thread::current();
    let msg = info
        .payload()
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| info.payload().downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "(non-string panic payload)".into());
    let at = info
        .location()
        .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
        .unwrap_or_default();
    format!(
        "=== panic at unix {secs}, {} {}, thread {:?}\n{msg}\n  at {at}\n{backtrace}\n",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        thread.name().unwrap_or("?"),
    )
}

/// Install the hook (keeping the default one, which prints to stderr).
pub fn install() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let text = report(info, &std::backtrace::Backtrace::force_capture());
        let path = log_path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
            let _ = f.write_all(text.as_bytes());
        }
        default(info);
    }));
}
