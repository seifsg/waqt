//! User settings: where, how to calculate, how to alert. Saved once as JSON.

use salah::prelude::{Madhab, Method};
use serde::{Deserialize, Serialize};

/// Everything Waqt needs to compute and announce prayer times.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub place: String,
    pub lat: f64,
    pub lon: f64,
    pub method: Method,
    pub madhab: Madhab,
    /// Days added to the Umm al-Qura date for local moon sighting, -2..=2.
    pub hijri_offset: i8,
    /// Adhan on or off per prayer, in `times::PRAYERS` order.
    pub adhan: [bool; 5],
    /// Adhan volume, 0..=100.
    pub volume: u8,
    pub autostart: bool,
}

/// `config.json` in the app's config folder (%APPDATA%\dev.seif.waqt on Windows).
pub fn path(app: &tauri::AppHandle) -> std::path::PathBuf {
    use tauri::Manager;
    app.path().app_config_dir().expect("the OS has a per-user config folder").join("config.json")
}

/// Reads the saved config: `Ok(None)` before first setup, `Err` when the file is unreadable or invalid.
pub fn load(path: &std::path::Path) -> Result<Option<Config>, String> {
    match std::fs::read_to_string(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("cannot read {}: {e}", path.display())),
        Ok(text) => serde_json::from_str(&text).map(Some).map_err(|e| format!("{} is not valid: {e}", path.display())),
    }
}
