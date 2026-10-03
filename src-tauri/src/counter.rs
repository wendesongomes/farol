//! Keeps the number of open ports next to the tray icon up to date.
//!
//! It only counts listening sockets, which is cheap, so it can keep running
//! while the panel is closed. The panel itself refreshes the full list.

use std::time::Duration;

use tauri::AppHandle;

use crate::{i18n, panel, ports};

const OPEN_INTERVAL: Duration = Duration::from_secs(3);
const CLOSED_INTERVAL: Duration = Duration::from_secs(15);

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let strings = i18n::system_tray_strings();
        let mut last = None;
        loop {
            let count = ports::listening().len();
            if last != Some(count) {
                update_tray(&app, count, strings);
                last = Some(count);
            }
            std::thread::sleep(if panel::is_open(&app) {
                OPEN_INTERVAL
            } else {
                CLOSED_INTERVAL
            });
        }
    });
}

fn update_tray(app: &AppHandle, count: usize, strings: &i18n::TrayStrings) {
    let Some(tray) = app.tray_by_id(crate::tray::ID) else {
        return;
    };
    // The tooltip works everywhere; Windows can't show text next to the icon.
    let _ = tray.set_tooltip(Some((strings.port_count)(count)));
    // macOS menu bar and Linux (AppIndicator label, where the panel shows it).
    #[cfg(not(windows))]
    let _ = tray.set_title(Some(count.to_string()));
}
