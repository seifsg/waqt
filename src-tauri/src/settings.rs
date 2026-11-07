//! The settings window and the commands it calls. The webview is untrusted: Rust validates.

use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_autostart::ManagerExt;

use crate::{Shared, clock, config::{self, Config}, tray};

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
