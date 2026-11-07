# Credits

Waqt stands on these works. Thank you.

## Bundled data and media

| Work | Author | License | Use in Waqt |
|---|---|---|---|
| [Beautiful adhan.ogg](https://commons.wikimedia.org/wiki/File:Beautiful_adhan.ogg) | Adam-synagda, Wikimedia Commons | [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/) | `src-tauri/assets/adhan.ogg`, unchanged (SHA-1 `a1fa4fd942401922c5c3301816384aae86956522`) |
| [GeoNames](https://www.geonames.org/) `cities1000` and `admin1CodesASCII` | GeoNames | [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) | `ui/public/cities.tsv`, modified: only name, region, country code and coordinates kept, coordinates rounded to 2 decimals, sorted by population |
| [Inter](https://rsms.me/inter/) via `@fontsource-variable/inter` | Rasmus Andersson | [SIL OFL 1.1](https://openfontlicense.org/) | Landing page font |

## Code

| Library | License | Use |
|---|---|---|
| [salah](https://github.com/insha/salah) (`main` at `adc8f592c4`, vendored in `src-tauri/vendor/salah` with two fixes, see its `PATCHES.md`) | MIT | Prayer-time calculation |
| [Adhan](https://github.com/batoulapps/adhan-js) test timetables and adhan-js 4.4.6 | MIT | Reference times in `src-tauri/src/times.rs` tests |
| [ICU4X `icu_calendar`](https://github.com/unicode-org/icu4x) | Unicode-3.0 | Umm al-Qura Hijri dates |
| [Tauri](https://tauri.app) and its notification, autostart and single-instance plugins | MIT or Apache-2.0 | Tray, window, notifications |
| [rodio](https://github.com/RustAudio/rodio) | MIT or Apache-2.0 | Adhan playback |
| [chrono](https://github.com/chronotope/chrono), [serde](https://serde.rs) | MIT or Apache-2.0 | Time and settings |
| [Vite](https://vite.dev), [TypeScript](https://www.typescriptlang.org) | MIT, Apache-2.0 | Settings window and landing page builds |

Published timetables behind the tests: Umm al-Qura University ([ummulqura.org.sa](http://www.ummulqura.org.sa/)), MUIS Singapore ([muis.gov.sg](https://www.muis.gov.sg/)), and the Kuwait prayer times app, as collected in the Adhan project.
