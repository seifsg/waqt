//! The tray icon and its menu: a live badge, a tooltip and today's prayer list.

use chrono::{DateTime, Local, Utc};
use tauri::{
    AppHandle, Manager,
    image::Image,
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIcon, TrayIconBuilder},
};

use crate::{
    config::Config,
    hijri,
    icon::{self, Look},
    times,
};

const TEAL: Look = Look { bg: [0x2B, 0xB3, 0xA3], fg: [0x0B, 0x16, 0x26] };
const GOLD: Look = Look { bg: [0xE8, 0xB8, 0x5C], fg: [0x0B, 0x16, 0x26] };
const NIGHT: Look = Look { bg: [0x1E, 0x35, 0x50], fg: [0xF7, 0xF3, 0xEA] };

/// Handles kept to update the tray each tick.
pub struct Tray {
    icon: TrayIcon,
    header: MenuItem<tauri::Wry>,
    lines: [CheckMenuItem<tauri::Wry>; 5],
    hijri: MenuItem<tauri::Wry>,
    stop: MenuItem<tauri::Wry>,
}

/// Creates the tray with its menu; menu clicks go to `on_menu`.
pub fn build(app: &AppHandle, on_menu: fn(&AppHandle, &str)) -> tauri::Result<Tray> {
    let info = |id: &str| MenuItem::with_id(app, id, "", true, None::<&str>);
    let header = info("header")?;
    let hijri = info("hijri")?;
    let lines = times::PRAYERS.map(|p| CheckMenuItem::with_id(app, p.name(), p.name(), true, false, None::<&str>));
    let [a, b, c, d, e] = lines;
    let lines = [a?, b?, c?, d?, e?];
    let sep = || PredefinedMenuItem::separator(app);
    let stop = MenuItem::with_id(app, "stop", "Stop adhan", false, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Waqt", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[&header, &sep()?, &lines[0], &lines[1], &lines[2], &lines[3], &lines[4], &hijri, &sep()?, &stop, &settings, &quit],
    )?;
    let icon = TrayIconBuilder::with_id("waqt")
        .icon(app.default_window_icon().expect("bundle icon is configured").clone())
        .tooltip("Waqt")
        .menu(&menu)
        .on_menu_event(move |app, e| on_menu(app, e.id().as_ref()))
        .build(app)?;
    Ok(Tray { icon, header, lines, hijri, stop })
}

/// Redraws badge, tooltip and menu for `now` and returns the next prayer instant.
/// `None` config shows the setup hint.
pub fn show(app: &AppHandle, cfg: Option<&Config>, now: DateTime<Utc>) -> tauri::Result<Option<DateTime<Utc>>> {
    let t = app.state::<Tray>();
    let Some(cfg) = cfg else {
        t.header.set_text("Set your location in Settings")?;
        t.icon.set_tooltip(Some("Waqt · set your location"))?;
        return Ok(None);
    };
    let next = times::next(cfg, now);
    let mins = ((next.at - now).num_seconds().max(0) as u64).div_ceil(60) as i64;
    let (text, look) = badge(mins);
    t.icon.set_icon(Some(Image::new_owned(icon::render(&text, look, tray_px()), tray_px(), tray_px())))?;
    let head = format!("{} in {} · {}", next.label(), left(mins), clock(next.at));
    t.icon.set_tooltip(Some(fit_tooltip(&format!("{head}\n{}", cfg.place))))?;
    t.header.set_text(&head)?;
    let (date, day) = times::day_of(cfg, next);
    for (item, slot) in t.lines.iter().zip(day) {
        item.set_text(format!("{}\t{}", slot.label(), clock(slot.at)))?;
        item.set_checked(slot == next)?;
    }
    t.hijri.set_text(format!("{} · {}", hijri::label(date, cfg.hijri_offset), cfg.place))?;
    Ok(Some(next.at))
}

/// Enables "Stop adhan" while the adhan plays.
pub fn set_playing(app: &AppHandle, playing: bool) {
    if let Err(e) = app.state::<Tray>().stop.set_enabled(playing) {
        eprintln!("waqt: tray menu update failed: {e}");
    }
}

/// Badge text and colors: minutes under 100 (gold in the last 15), else hours on night blue.
fn badge(mins: i64) -> (String, Look) {
    match mins {
        0..=15 => (mins.to_string(), GOLD),
        16..=99 => (mins.to_string(), TEAL),
        _ if mins < 600 => (format!("{}h", mins / 60), NIGHT),
        _ => ((mins / 60).to_string(), NIGHT),
    }
}

/// Cuts `text` to 120 UTF-16 units: Windows tooltips hold 127 plus a terminator.
fn fit_tooltip(text: &str) -> String {
    let mut units = 0;
    text.chars().take_while(|c| { units += c.len_utf16(); units <= 120 }).collect()
}

/// "45m" or "2h 05m".
fn left(mins: i64) -> String {
    if mins < 60 { format!("{mins}m") } else { format!("{}h {:02}m", mins / 60, mins % 60) }
}

/// Local wall-clock "HH:MM" of an instant.
pub fn clock(at: DateTime<Utc>) -> String {
    at.with_timezone(&Local).format("%H:%M").to_string()
}

/// Tray icon edge in pixels at the current DPI.
fn tray_px() -> u32 {
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSMICON};
        // SAFETY: plain metric query, no pointers.
        let px = unsafe { GetSystemMetrics(SM_CXSMICON) };
        // 0 means the query failed; 16 px is the 100% DPI tray size.
        u32::try_from(px).unwrap_or(0).max(16)
    }
    #[cfg(not(windows))]
    32
}
