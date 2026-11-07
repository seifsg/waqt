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

use crate::{config::Config, log, times::Slot, tray};

/// "Beautiful adhan", CC0, Wikimedia Commons (see CREDITS.md).
const ADHAN: &[u8] = include_bytes!("../assets/adhan.ogg");

/// A crossing seen later than this (sleep, suspend) gets a quiet "began at" toast and no adhan.
const ON_TIME: Duration = Duration::minutes(2);

/// The adhan now playing, if any, and a generation that every play and stop bumps, so a
/// player that registers after it was stopped or replaced never plays. The lock is never
/// held across a tray call (off the main thread, those wait for it).
#[derive(Default)]
pub struct Adhan(Mutex<(u64, Option<Arc<Player>>)>);

/// Shows a Windows toast; `chime` plays the system sound, else it is silent.
pub fn notify(app: &AppHandle, title: &str, body: &str, chime: bool) {
    let toast = app.notification().builder().title(title).body(body);
    let toast = if chime { toast.sound("Default") } else { toast };
    if let Err(e) = toast.show() {
        log::error(&format!("toast failed: {e}"));
    }
}

/// Announces a prayer that began `late` ago; the toast is silent, the adhan is the only sound.
pub fn fire(app: &AppHandle, cfg: &Config, slot: Slot, late: Duration) {
    let time = tray::clock(slot.at);
    if late > ON_TIME {
        return notify(app, &format!("{} began at {time}", slot.label()), &cfg.place, false);
    }
    let title = if slot.iftar { "Iftar time · Maghrib".to_owned() } else { format!("Time for {}", slot.label()) };
    notify(app, &title, &format!("{time} · {}", cfg.place), false);
    if cfg.adhan[slot.prayer as usize] && cfg.volume > 0 {
        play(app, cfg.volume);
    }
}

/// Plays the adhan at `volume` (0..=100) on its own thread; the audio device is open only meanwhile.
pub fn play(app: &AppHandle, volume: u8) {
    let me = stop(app);
    let app = app.clone();
    thread::spawn(move || {
        let mut sink = match DeviceSinkBuilder::open_default_sink() {
            Ok(sink) => sink,
            Err(e) => return notify(&app, "Adhan skipped", &format!("No audio output: {e}"), false),
        };
        sink.log_on_drop(false); // dropping it after the adhan is the normal end
        let player = Arc::new(rodio::play(sink.mixer(), Cursor::new(ADHAN)).expect("bundled adhan.ogg decodes"));
        player.set_volume(f32::from(volume.min(100)) / 100.0);
        if !swap(&app, me, Some(player.clone())) {
            return player.stop(); // stopped or replaced while the device opened
        }
        sync_row(&app);
        player.sleep_until_end();
        swap(&app, me, None);
        sync_row(&app);
    });
}

/// Stops the adhan if it is playing and returns the generation a new play may use.
pub fn stop(app: &AppHandle) -> u64 {
    let (generation, old) = {
        let mut slot = lock(app);
        slot.0 += 1;
        (slot.0, slot.1.take())
    };
    if let Some(p) = old {
        p.stop();
        sync_row(app);
    }
    generation
}

/// Enables "Stop adhan" exactly while one plays. It reads the slot on the main thread, where
/// these run one by one, so the last sync after any change shows the final state.
fn sync_row(app: &AppHandle) {
    let main = app.clone();
    let sync = move || {
        let playing = lock(&main).1.is_some();
        tray::set_playing(&main, playing);
    };
    if let Err(e) = app.run_on_main_thread(sync) {
        log::error(&format!("tray menu update failed: {e}"));
    }
}

/// Sets the playing slot if `generation` is still current; returns whether it was.
fn swap(app: &AppHandle, generation: u64, player: Option<Arc<Player>>) -> bool {
    let mut slot = lock(app);
    let current = slot.0 == generation;
    if current {
        slot.1 = player;
    }
    current
}

/// The adhan slot, locked.
fn lock(app: &AppHandle) -> std::sync::MutexGuard<'_, (u64, Option<Arc<Player>>)> {
    app.state::<Adhan>().inner().0.lock().expect("adhan lock is never poisoned (panic = abort)")
}
