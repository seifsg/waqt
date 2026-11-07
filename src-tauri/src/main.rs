#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clock;
mod config;
mod hijri;
mod icon;
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
}

fn on_menu(app: &AppHandle, id: &str) {
    if id == "quit" {
        app.exit(0);
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let cfg = config::load(&config::path(app.handle())).unwrap_or_else(|e| {
                eprintln!("waqt: {e}");
                None
            });
            app.manage(Shared { cfg: RwLock::new(cfg) });
            app.manage(tray::build(app.handle(), on_menu)?);
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
