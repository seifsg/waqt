//! User settings: where, how to calculate, how to alert. Saved once as JSON.

use salah::prelude::{Madhab, Method};
use serde::{Deserialize, Serialize};

/// Everything Waqt needs to compute and announce prayer times.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub place: String,
    pub lat: f64,
    pub lon: f64,
    pub method: Method,
    pub madhab: Madhab,
    /// Days added to the Umm al-Qura date for local moon sighting, -2..=2.
    pub hijri_offset: i8,
    /// Adhan on or off per prayer, in `times::PRAYERS` order.
    pub adhan: [bool; 5],
    /// Adhan volume, 0..=100.
    pub volume: u8,
    pub autostart: bool,
}
