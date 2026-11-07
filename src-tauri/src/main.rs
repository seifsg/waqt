#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{RunEvent, image::Image, menu::{Menu, MenuItem}, tray::TrayIconBuilder};

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let quit = MenuItem::with_id(app, "quit", "Quit Waqt", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit])?;
            TrayIconBuilder::with_id("waqt")
                .icon(Image::new_owned(vec![43, 179, 163, 255].repeat(16 * 16), 16, 16))
                .tooltip("Waqt spike")
                .menu(&menu)
                .on_menu_event(|app, e| if e.id() == "quit" { app.exit(0) })
                .build(app)?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("tauri app")
        .run(|_, e| if let RunEvent::ExitRequested { code: None, api, .. } = e { api.prevent_exit() });
}
