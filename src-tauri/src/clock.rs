//! The one wall clock Waqt reads. Debug builds can shift it with `WAQT_SHIFT_SECS`
//! to demo alerts and Ramadan without touching the system clock; release builds cannot.

use chrono::{DateTime, Duration, Utc};

pub fn now() -> DateTime<Utc> {
    Utc::now() + shift()
}

#[cfg(debug_assertions)]
fn shift() -> Duration {
    static SHIFT: std::sync::OnceLock<Duration> = std::sync::OnceLock::new();
    *SHIFT.get_or_init(|| {
        let secs = std::env::var("WAQT_SHIFT_SECS").map_or(0, |v| v.parse().expect("WAQT_SHIFT_SECS must be whole seconds"));
        Duration::seconds(secs)
    })
}

#[cfg(not(debug_assertions))]
fn shift() -> Duration {
    Duration::zero()
}
