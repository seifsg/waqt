//! Prayer-time alerts: a toast, and the adhan when it is on for that prayer.

use std::{
    io::Cursor,
    sync::{Arc, Mutex},
    thread,
};

use chrono::Duration;
use rodio::{DeviceSinkBuilder, Player};
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::{config::Config, times::Slot, tray};

/// "Beautiful adhan", CC0, Wikimedia Commons (see CREDITS.md).
const ADHAN: &[u8] = include_bytes!("../assets/adhan.ogg");

/// A crossing seen later than this (sleep, suspend) gets a quiet "began at" toast and no adhan.
const ON_TIME: Duration = Duration::minutes(2);

/// The adhan playing now, if any; shared so the tray and settings can stop it.
#[derive(Default)]
pub struct Adhan(Mutex<Option<Arc<Player>>>);

/// Shows a Windows toast; `chime` plays the system sound, else it is silent.
pub fn notify(app: &AppHandle, title: &str, body: &str, chime: bool) {
    let toast = app.notification().builder().title(title).body(body);
    let toast = if chime { toast.sound("Default") } else { toast };
    if let Err(e) = toast.show() {
        eprintln!("waqt: toast failed: {e}");
    }
}

/// Announces a prayer that began `late` ago.
pub fn fire(app: &AppHandle, cfg: &Config, slot: Slot, late: Duration) {
    let time = tray::clock(slot.at);
    if late > ON_TIME {
        return notify(app, &format!("{} began at {time}", slot.label()), &cfg.place, false);
    }
    let adhan = cfg.adhan[slot.prayer as usize] && cfg.volume > 0;
    let title = if slot.iftar { "Iftar time · Maghrib".to_owned() } else { format!("Time for {}", slot.label()) };
    notify(app, &title, &format!("{time} · {}", cfg.place), !adhan);
    if adhan {
        play(app, cfg.volume);
    }
}

/// Plays the adhan at `volume` (0..=100) on its own thread; the audio device is open only meanwhile.
pub fn play(app: &AppHandle, volume: u8) {
    stop(app);
    let app = app.clone();
    thread::spawn(move || {
        let mut sink = match DeviceSinkBuilder::open_default_sink() {
            Ok(sink) => sink,
            Err(e) => return eprintln!("waqt: no audio output, adhan skipped: {e}"),
        };
        sink.log_on_drop(false); // dropping it after the adhan is the normal end
        let player = Arc::new(rodio::play(sink.mixer(), Cursor::new(ADHAN)).expect("bundled adhan.ogg decodes"));
        player.set_volume(f32::from(volume.min(100)) / 100.0);
        *current(&app) = Some(player.clone());
        tray::set_playing(&app, true);
        player.sleep_until_end();
        let mut now = current(&app);
        if now.as_ref().is_some_and(|p| Arc::ptr_eq(p, &player)) {
            *now = None;
            tray::set_playing(&app, false);
        }
    });
}

/// The playing adhan slot, locked.
fn current(app: &AppHandle) -> std::sync::MutexGuard<'_, Option<Arc<Player>>> {
    app.state::<Adhan>().inner().0.lock().expect("adhan lock is never poisoned (panic = abort)")
}

/// Stops the adhan if it is playing.
pub fn stop(app: &AppHandle) {
    if let Some(p) = current(app).take() {
        p.stop();
        tray::set_playing(app, false);
    }
}
