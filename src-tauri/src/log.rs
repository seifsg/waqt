//! Waqt's error log. A release build has no console, so errors and panics go to
//! `%LOCALAPPDATA%\dev.seif.waqt\logs\waqt.log`, local only, capped at two small files.

use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::OnceLock,
};

use tauri::{AppHandle, Manager};

/// Size past which the log moves to `waqt.old.log`; the two never pass about twice this.
const CAP: u64 = 256 * 1024;

static DIR: OnceLock<PathBuf> = OnceLock::new();

/// Points the log at the app's log folder and logs panics too (panic = abort skips unwinding).
pub fn init(app: &AppHandle) {
    DIR.get_or_init(|| app.path().app_log_dir().expect("the OS has a per-user data folder"));
    std::panic::set_hook(Box::new(|info| error(&format!("panic: {info}"))));
}

/// Writes one timestamped line to stderr and to the log file.
pub fn error(msg: &str) {
    let line = format!("{} {msg}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
    eprint!("waqt: {line}");
    if let Some(dir) = DIR.get()
        && let Err(e) = append(dir, &line)
    {
        eprintln!("waqt: log file not written: {e}"); // the log is the last place to report to
    }
}

/// Appends `line`, first moving a full log aside.
fn append(dir: &Path, line: &str) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let path = dir.join("waqt.log");
    if fs::metadata(&path).is_ok_and(|m| m.len() > CAP) {
        fs::rename(&path, dir.join("waqt.old.log"))?;
    }
    OpenOptions::new().create(true).append(true).open(path)?.write_all(line.as_bytes())
}
