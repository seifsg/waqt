//! The settings window and the commands it calls. The webview is untrusted: Rust validates.

use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_autostart::ManagerExt;

use serde::Serialize;

use crate::{Shared, alert, clock, config::{self, Config}, hijri, times, tray};

const LABEL: &str = "settings";

/// Opens the settings window, or focuses it when already open.
pub fn open(app: &AppHandle) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window(LABEL) {
        w.unminimize()?;
        return w.set_focus();
    }
    WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html".into()))
        .title("Waqt settings")
        .inner_size(480.0, 780.0)
        .min_inner_size(400.0, 560.0)
        .center()
        .build()
        .map(drop)
}

/// The saved config, or `None` before first setup.
#[tauri::command]
pub fn get_config(state: State<'_, Shared>) -> Option<Config> {
    state.config()
}

/// One prayer line in the preview.
#[derive(Serialize)]
pub struct Line {
    label: &'static str,
    time: String,
}

/// What the form would give: the next prayer's day, which line is next, its Hijri date.
#[derive(Serialize)]
pub struct Preview {
    times: Vec<Line>,
    next: usize,
    hijri: String,
    polar: bool,
}

/// Times for a config being edited, so method and Asr choices show their effect at once.
#[tauri::command]
pub fn preview(cfg: Config) -> Result<Preview, String> {
    cfg.validate_math()?;
    let next = times::next(&cfg, clock::now());
    let (date, day) = times::day_of(&cfg, next);
    Ok(Preview {
        times: day.iter().map(|s| Line { label: s.label(), time: tray::clock(s.at) }).collect(),
        next: day.iter().position(|s| *s == next).expect("day_of returns the day holding next"),
        hijri: hijri::label(date, cfg.hijri_offset),
        polar: cfg.lat.abs() > times::MAX_LAT,
    })
}

/// Validates, saves and applies a config, then closes the window.
#[tauri::command]
pub fn save_config(app: AppHandle, cfg: Config) -> Result<(), String> {
    cfg.validate()?;
    config::save(&config::path(&app), &cfg)?;
    let launch = app.autolaunch();
    if cfg.autostart { launch.enable() } else { launch.disable() }.map_err(|e| format!("Start with Windows: {e}"))?;
    app.state::<Shared>().set(cfg.clone());
    tray::show(&app, Some(&cfg), clock::now()).map_err(|e| e.to_string())?;
    if let Some(w) = app.get_webview_window(LABEL) {
        w.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Plays the adhan at `volume` so the user can hear it before saving.
#[tauri::command]
pub fn test_adhan(app: AppHandle, volume: u8) {
    alert::play(&app, volume.min(100));
}

/// Stops a playing adhan.
#[tauri::command]
pub fn stop_adhan(app: AppHandle) {
    alert::stop(&app);
}
