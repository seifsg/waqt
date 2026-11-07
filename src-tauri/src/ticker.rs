//! The heartbeat: wakes at each minute and at each prayer instant, announces prayers that
//! began since the last wake, then refreshes the tray.

use std::{thread, time::Duration as StdDuration};

use chrono::{DateTime, Utc};
use tauri::{AppHandle, Manager};

use crate::{Shared, alert, clock, log, times, tray};

/// Starts the background loop for the app's lifetime.
pub fn spawn(app: AppHandle) {
    thread::spawn(move || {
        // The previous wake with a config; a prayer between it and now just began. A Settings
        // save between wakes keeps it, so a prayer right after a save is still announced.
        let mut prev: Option<DateTime<Utc>> = None;
        loop {
            let now = clock::now();
            let cfg = app.state::<Shared>().config();
            if let (Some(c), Some(then)) = (&cfg, prev) {
                let last = times::last(c, now);
                if last.at > then {
                    alert::fire(&app, c, last, now - last.at);
                }
            }
            let next = tray::show(&app, cfg.as_ref(), now).unwrap_or_else(|e| {
                log::error(&format!("tray update failed: {e}"));
                None
            });
            prev = cfg.map(|_| now);
            thread::sleep(until_next_wake(next, now));
        }
    });
}

/// Time to the next minute boundary or the next prayer, whichever comes first (never past 60 s).
fn until_next_wake(prayer: Option<DateTime<Utc>>, now: DateTime<Utc>) -> StdDuration {
    let to_minute = 60_000 - now.timestamp_millis().rem_euclid(60_000);
    let to_prayer = prayer.map_or(i64::MAX, |at| (at - now).num_milliseconds());
    StdDuration::from_millis(to_minute.min(to_prayer).max(20) as u64)
}
