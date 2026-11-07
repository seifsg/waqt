# Waqt's patches to salah

Vendored from [insha/salah](https://github.com/insha/salah) `main` at `adc8f592c4` (MIT, see `LICENSE` and `AUTHORS`),
because crates.io 0.7.6 panics between 48 and 66.5 degrees and lacks serde. Two further fixes, each with a test in
`src-tauri/src/times.rs` that fails without it:

| File | Bug | Fix | Test |
|---|---|---|---|
| `src/astronomy/solar.rs` `setting_hour` | A solar time between 23:59:30 and 24:00 UTC rounded up to hour 24 and `unwrap()` panicked (New York, Chicago, Toronto, Mexico City, Suva: 3 to 14 days a year) | Add the rounded minutes to the date's midnight with chrono | `times_rounding_to_midnight_utc_match_adhan_js` |
| `src/astronomy/ops.rs` `days_since_solstice` | Southern hemisphere: `u32` underflow before day 172, 365 days too many after it, so Moonsighting Committee used the wrong season | Wrap like Adhan's `daysSinceSolstice` | `southern_moonsighting_committee_matches_adhan_js` |

Still open upstream: polar day or night (beyond about 66 degrees) panics; Waqt calculates at 65 degrees there.
Drop this copy once a salah release carries these fixes.
