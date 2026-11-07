//! Prayer schedule math: the five prayers of a day and the next or last one, offline.

use chrono::{DateTime, Duration, NaiveDate, Utc};
use salah::prelude::{Configuration, Coordinates, HighLatitudeRule, Method, PrayerTimes};

use crate::{config::Config, hijri};

/// Calculation latitude cap. Past ~66 deg the sun never sets or rises around the solstices and
/// the library panics; the nearest latitude where times exist is used instead (aqrab al-bilad).
pub const MAX_LAT: f64 = 65.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prayer {
    Fajr,
    Dhuhr,
    Asr,
    Maghrib,
    Isha,
}

pub const PRAYERS: [Prayer; 5] = [Prayer::Fajr, Prayer::Dhuhr, Prayer::Asr, Prayer::Maghrib, Prayer::Isha];

impl Prayer {
    /// English name shown in the tray and toasts.
    pub fn name(self) -> &'static str {
        ["Fajr", "Dhuhr", "Asr", "Maghrib", "Isha"][self as usize]
    }

    /// The matching salah library prayer.
    fn salah(self) -> salah::prelude::Prayer {
        use salah::prelude::Prayer as P;
        [P::Fajr, P::Dhuhr, P::Asr, P::Maghrib, P::Isha][self as usize]
    }
}

/// One prayer instant; `iftar` marks Maghrib on a Ramadan day.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slot {
    pub prayer: Prayer,
    pub at: DateTime<Utc>,
    pub iftar: bool,
}

impl Slot {
    /// What the user reads: "Iftar" in Ramadan, else the prayer name.
    pub fn label(&self) -> &'static str {
        if self.iftar { "Iftar" } else { self.prayer.name() }
    }
}

/// The five prayers of one date at the configured place.
pub fn day(cfg: &Config, date: NaiveDate) -> [Slot; 5] {
    let ramadan = hijri::month(date, cfg.hijri_offset) == 9;
    let coords = Coordinates::new(cfg.lat.clamp(-MAX_LAT, MAX_LAT), cfg.lon);
    let mut params = Configuration::with(cfg.method, cfg.madhab);
    params.high_latitude_rule = HighLatitudeRule::recommended(Coordinates::new(coords.latitude.abs(), cfg.lon));
    if ramadan && cfg.method == Method::UmmAlQura {
        params.adjustments.isha += 30; // Umm al-Qura's Ramadan rule: Isha 120 min after Maghrib.
    }
    let t = PrayerTimes::new(date, coords, params);
    PRAYERS.map(|p| Slot { prayer: p, at: t.time(p.salah()), iftar: ramadan && p == Prayer::Maghrib })
}

/// The place's own calendar date at `now` (mean solar time), independent of the PC's time zone.
pub fn place_date(cfg: &Config, now: DateTime<Utc>) -> NaiveDate {
    (now + Duration::seconds((cfg.lon * 240.0) as i64)).date_naive()
}

/// The date and five prayers that `slot` belongs to (a late Isha can fall after midnight).
pub fn day_of(cfg: &Config, slot: Slot) -> (NaiveDate, [Slot; 5]) {
    let d = place_date(cfg, slot.at);
    [d, d - Duration::days(1), d + Duration::days(1)]
        .into_iter()
        .map(|d| (d, day(cfg, d)))
        .find(|(_, s)| s.contains(&slot))
        .expect("a slot comes from one of the days around its own date")
}

/// Prayers of the place's yesterday, today and tomorrow: always brackets `now`.
fn around(cfg: &Config, now: DateTime<Utc>) -> impl Iterator<Item = Slot> {
    let d = place_date(cfg, now);
    [d - Duration::days(1), d, d + Duration::days(1)].into_iter().flat_map(move |d| day(cfg, d))
}

/// First prayer strictly after `now`.
pub fn next(cfg: &Config, now: DateTime<Utc>) -> Slot {
    around(cfg, now).filter(|s| s.at > now).min_by_key(|s| s.at).expect("tomorrow always has a prayer after now")
}

/// Latest prayer at or before `now`.
pub fn last(cfg: &Config, now: DateTime<Utc>) -> Slot {
    around(cfg, now).filter(|s| s.at <= now).max_by_key(|s| s.at).expect("yesterday always has a prayer before now")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;
    use salah::prelude::Madhab;

    fn cfg(lat: f64, lon: f64, method: Method, madhab: Madhab) -> Config {
        Config {
            place: String::new(), lat, lon, method, madhab, hijri_offset: 0,
            adhan: [true; 5], volume: 80, autostart: true,
        }
    }

    fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    /// Asserts each prayer's local "HH:MM" (fixed UTC offset in hours) is within `tol` minutes.
    fn golden(c: &Config, date: NaiveDate, utc_offset: i64, want: [&str; 5], tol: i64) {
        for (slot, w) in day(c, date).iter().zip(want) {
            let got = (slot.at + Duration::hours(utc_offset)).time();
            let off = (got - NaiveTime::parse_from_str(w, "%H:%M").unwrap()).num_minutes();
            assert!(off.abs() <= tol, "{date} {:?}: got {got} want {w}", slot.prayer);
        }
    }

    // Published timetables, as collected in Batoul Apps adhan-js `Shared/Times` (MIT):
    // Makkah: ummulqura.org.sa; Singapore: muis.gov.sg; Kuwait: Kuwait prayer times app.
    #[test]
    fn makkah_umm_al_qura_matches_official_timetable() {
        let c = cfg(21.427009, 39.828685, Method::UmmAlQura, Madhab::Shafi);
        golden(&c, ymd(2016, 1, 5), 3, ["05:38", "12:26", "15:31", "17:52", "19:22"], 1);
        golden(&c, ymd(2016, 7, 8), 3, ["04:18", "12:26", "15:43", "19:07", "20:37"], 1);
    }

    #[test]
    fn singapore_muis_matches_official_timetable() {
        let c = cfg(1.370844612058886, 103.80145644060552, Method::Singapore, Madhab::Shafi);
        golden(&c, ymd(2020, 1, 1), 8, ["05:44", "13:10", "16:34", "19:10", "20:25"], 1);
        golden(&c, ymd(2020, 6, 29), 8, ["05:39", "13:10", "16:35", "19:15", "20:30"], 1);
    }

    #[test]
    fn kuwait_matches_official_timetable_within_its_2_min_variance() {
        let c = cfg(29.370865, 47.979139, Method::Kuwait, Madhab::Shafi);
        golden(&c, ymd(2016, 1, 1), 3, ["05:18", "11:51", "14:42", "17:01", "18:23"], 2);
    }

    // Reference implementation: Batoul Apps adhan-js 4.4.6, times in UTC.
    #[test]
    fn southern_moonsighting_committee_matches_adhan_js() {
        let c = cfg(-33.92, 18.42, Method::MoonsightingCommittee, Madhab::Shafi); // Cape Town
        golden(&c, ymd(2026, 1, 15), 0, ["02:11", "11:01", "14:40", "18:03", "19:14"], 1);
        golden(&c, ymd(2026, 7, 15), 0, ["04:21", "10:57", "13:37", "15:59", "17:22"], 1);
        golden(&c, ymd(2026, 9, 30), 0, ["03:01", "10:41", "14:08", "16:51", "18:03"], 1);
    }

    #[test]
    fn times_rounding_to_midnight_utc_match_adhan_js() {
        let mexico = cfg(19.43, -99.13, Method::MuslimWorldLeague, Madhab::Shafi);
        golden(&mexico, ymd(2026, 11, 7), 0, ["11:25", "18:21", "21:35", "00:00", "01:11"], 1);
        let new_york = cfg(40.71, -74.01, Method::MuslimWorldLeague, Madhab::Shafi);
        golden(&new_york, ymd(2026, 5, 7), 0, ["07:59", "16:54", "20:47", "23:58", "01:40"], 1);
    }

    #[test]
    fn method_and_asr_choice_change_times() {
        let date = ymd(2026, 11, 7);
        let mwl = day(&cfg(40.7128, -74.0059, Method::MuslimWorldLeague, Madhab::Shafi), date);
        let isna = day(&cfg(40.7128, -74.0059, Method::NorthAmerica, Madhab::Shafi), date);
        let hanafi = day(&cfg(40.7128, -74.0059, Method::MuslimWorldLeague, Madhab::Hanafi), date);
        assert!(isna[0].at - mwl[0].at >= Duration::minutes(10), "ISNA 15 deg Fajr is later than MWL 18 deg");
        assert!(hanafi[2].at - mwl[2].at >= Duration::minutes(30), "Hanafi Asr is later");
        assert_eq!(hanafi[1].at, mwl[1].at, "Dhuhr does not depend on madhab");
    }

    #[test]
    fn after_isha_next_is_tomorrows_fajr_and_last_is_isha() {
        let c = cfg(21.427009, 39.828685, Method::UmmAlQura, Madhab::Shafi);
        let today = day(&c, ymd(2026, 11, 7));
        let after_isha = today[4].at + Duration::minutes(5);
        assert_eq!(next(&c, after_isha), day(&c, ymd(2026, 11, 8))[0]);
        assert_eq!(last(&c, after_isha), today[4]);
        assert_eq!(next(&c, today[2].at), today[3], "exactly at Asr the next is Maghrib");
        assert_eq!(last(&c, today[2].at), today[2]);
    }

    #[test]
    fn ramadan_marks_iftar_and_umm_al_qura_isha_moves_to_120_min() {
        let c = cfg(21.427009, 39.828685, Method::UmmAlQura, Madhab::Shafi);
        let ramadan = day(&c, ymd(2026, 3, 1));
        assert!(ramadan[3].iftar && ramadan[3].label() == "Iftar");
        assert!(ramadan.iter().filter(|s| s.iftar).count() == 1);
        assert_eq!(ramadan[4].at - ramadan[3].at, Duration::minutes(120));
        let normal = day(&c, ymd(2026, 4, 1));
        assert!(!normal[3].iftar && normal[3].label() == "Maghrib");
        assert_eq!(normal[4].at - normal[3].at, Duration::minutes(90));
    }

    #[test]
    fn every_method_at_every_latitude_never_panics_and_stays_ordered() {
        use Method::*;
        assert!(cfg(0.0, 0.0, Other, Madhab::Shafi).validate_math().is_err(), "0 deg angles are rejected");
        let methods = [
            MuslimWorldLeague, Egyptian, Karachi, UmmAlQura, Dubai, MoonsightingCommittee,
            NorthAmerica, Kuwait, Qatar, Singapore, Tehran, Turkey,
        ];
        for lat in [-90.0, -65.0, -60.0, -48.5, 0.0, 21.4, 48.5, 51.5, 60.0, 65.0, 69.65, 90.0] {
            for lon in [-180.0, 0.0, 179.9] {
                for month in 1..=12 {
                    for (m, h) in methods.iter().flat_map(|&m| [(m, Madhab::Shafi), (m, Madhab::Hanafi)]) {
                        let c = cfg(lat, lon, m, h);
                        if c.validate_math().is_err() {
                            continue; // rejected at entry, never computed
                        }
                        let s = day(&c, ymd(2026, month, 21));
                        assert!(s.windows(2).all(|w| w[0].at <= w[1].at), "{lat},{lon} month {month} {m:?} {h:?} out of order");
                    }
                }
            }
        }
    }
}
