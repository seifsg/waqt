//! Toasts (and, at prayer time, the adhan).

use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

/// Shows a Windows toast; `chime` plays the system sound, else it is silent.
pub fn notify(app: &AppHandle, title: &str, body: &str, chime: bool) {
    let toast = app.notification().builder().title(title).body(body);
    let toast = if chime { toast.sound("Default") } else { toast };
    if let Err(e) = toast.show() {
        eprintln!("waqt: toast failed: {e}");
    }
}
