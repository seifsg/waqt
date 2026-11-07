#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod alert;
mod clock;
mod config;
mod hijri;
mod icon;
mod settings;
mod ticker;
mod times;
mod tray;

use std::sync::RwLock;

use config::Config;
use tauri::{AppHandle, Manager, RunEvent};

/// State shared by the ticker, the tray and the settings commands.
pub struct Shared {
    cfg: RwLock<Option<Config>>,
}

impl Shared {
    /// A snapshot of the current config.
    pub fn config(&self) -> Option<Config> {
        self.cfg.read().expect("config lock is never poisoned (panic = abort)").clone()
    }

    /// Replaces the config the ticker and tray use.
    pub fn set(&self, cfg: Config) {
        *self.cfg.write().expect("config lock is never poisoned (panic = abort)") = Some(cfg);
    }
}

fn on_menu(app: &AppHandle, id: &str) {
    match id {
        "settings" => settings::open(app).unwrap_or_else(|e| eprintln!("waqt: settings window: {e}")),
        "quit" => app.exit(0),
        _ => {}
    }
}

/// Loads the saved config; an unreadable file is reported with a toast and treated as first run.
fn load_config(app: &AppHandle) -> Option<Config> {
    config::load(&config::path(app)).unwrap_or_else(|e| {
        alert::notify(app, "Waqt settings unreadable", &format!("{}. Please set Waqt up again.", e.trim_end_matches('.')), true);
        None
    })
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .invoke_handler(tauri::generate_handler![settings::get_config, settings::save_config])
        .setup(|app| {
            app.manage(tray::build(app.handle(), on_menu)?);
            let cfg = load_config(app.handle());
            let first_run = cfg.is_none();
            app.manage(Shared { cfg: RwLock::new(cfg) });
            if first_run {
                settings::open(app.handle())?;
            }
            ticker::spawn(app.handle().clone());
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("tauri app")
        .run(|_, e| {
            // Closing the settings window must not quit a tray app; only "Quit" (code Some) does.
            if let RunEvent::ExitRequested { code: None, api, .. } = e {
                api.prevent_exit();
            }
        });
}
