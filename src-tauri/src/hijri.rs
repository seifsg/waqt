//! Offline Hijri dates (Umm al-Qura), shifted by the user's moon-sighting offset.

use chrono::{Datelike, Duration, NaiveDate};
use icu_calendar::{Date, cal::Hijri};

const MONTHS: [&str; 12] = [
    "Muharram", "Safar", "Rabi al-Awwal", "Rabi al-Thani", "Jumada al-Ula", "Jumada al-Akhirah",
    "Rajab", "Shaban", "Ramadan", "Shawwal", "Dhu al-Qadah", "Dhu al-Hijjah",
];

/// Hijri (day, month 1..=12, year) of a Gregorian date plus `offset` days.
fn parts(date: NaiveDate, offset: i8) -> (u8, u8, i32) {
    let d = date + Duration::days(offset.into());
    let h = Date::try_new_iso(d.year(), d.month() as u8, d.day() as u8)
        .expect("chrono dates are valid ISO dates")
        .to_calendar(Hijri::new_umm_al_qura());
    (h.day_of_month().0, h.month().ordinal, h.year().era_year_or_related_iso())
}

/// Hijri month number; 9 is Ramadan.
pub fn month(date: NaiveDate, offset: i8) -> u8 {
    parts(date, offset).1
}

/// Human date, e.g. "17 Ramadan 1447".
pub fn label(date: NaiveDate, offset: i8) -> String {
    let (d, m, y) = parts(date, offset);
    format!("{d} {} {y}", MONTHS[usize::from(m - 1)])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn ramadan_1447_bounds_and_offset() {
        assert_eq!(month(ymd(2026, 2, 17), 0), 8);
        assert_eq!(month(ymd(2026, 2, 18), 0), 9);
        assert_eq!(label(ymd(2026, 3, 19), 0), "30 Ramadan 1447");
        assert_eq!(month(ymd(2026, 3, 20), 0), 10);
        assert_eq!(month(ymd(2026, 2, 17), 1), 9, "+1 day pulls Ramadan earlier");
        assert_eq!(month(ymd(2026, 3, 19), 1), 10);
    }
}
