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

fn check(ok: bool, msg: &str) -> Result<(), String> {
    if ok { Ok(()) } else { Err(msg.to_owned()) }
}

impl Config {
    /// Rejects values the math cannot handle; the message is shown to the user as is.
    pub fn validate_math(&self) -> Result<(), String> {
        check(self.lat.is_finite() && self.lat.abs() <= 90.0, "Latitude must be a number from -90 to 90.")?;
        check(self.lon.is_finite() && self.lon.abs() <= 180.0, "Longitude must be a number from -180 to 180.")?;
        check((-2..=2).contains(&self.hijri_offset), "Hijri adjustment must be from -2 to +2 days.")?;
        check(self.volume <= 100, "Volume must be from 0 to 100.")
    }

    /// Everything a saved config must satisfy.
    pub fn validate(&self) -> Result<(), String> {
        self.validate_math()?;
        check((1..=120).contains(&self.place.trim().chars().count()), "Name the place (up to 120 characters).")
    }
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
        Ok(text) => {
            let cfg: Config = serde_json::from_str(&text).map_err(|e| format!("{} is not valid: {e}", path.display()))?;
            cfg.validate().map(|()| Some(cfg)).map_err(|e| format!("{}: {e}", path.display()))
        }
    }
}

/// Writes the config atomically (temp file, then rename) so a crash never leaves half a file.
pub fn save(path: &std::path::Path, cfg: &Config) -> Result<(), String> {
    let fail = |e: std::io::Error| format!("cannot save {}: {e}", path.display());
    std::fs::create_dir_all(path.parent().expect("config path has a folder")).map_err(fail)?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(cfg).expect("Config always serializes")).map_err(fail)?;
    std::fs::rename(&tmp, path).map_err(fail)
}
