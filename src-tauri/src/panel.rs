//! The panel window: showing, hiding and placing it next to the tray icon.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Manager, PhysicalPosition, Rect, WebviewWindow, WindowEvent};

pub const LABEL: &str = "panel";

/// Window size in logical pixels, from `tauri.conf.json`. Includes the
/// transparent margin that leaves room for the panel's CSS shadow.
const WINDOW_WIDTH: f64 = 396.0;
const WINDOW_HEIGHT: f64 = 536.0;

/// Clicking the tray icon while the panel is open first takes focus away from
/// the panel (which hides it) and only then delivers the click. Without this
/// grace period the click would immediately reopen the panel.
const REOPEN_GRACE: Duration = Duration::from_millis(300);

#[derive(Default)]
pub struct PanelState {
    hidden_by_blur_at: Mutex<Option<Instant>>,
}

/// A rectangle in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Bounds {
    fn center_x(&self) -> f64 {
        self.x + self.width / 2.0
    }

    fn center_y(&self) -> f64 {
        self.y + self.height / 2.0
    }
}

/// Places a window of `size` next to the tray icon: below it when the icon is
/// in the top half of the screen (macOS menu bar, top panels), above it
/// otherwise (Windows taskbar), always kept inside the monitor's work area.
pub fn place_near_icon(icon: Bounds, work_area: Bounds, size: (f64, f64)) -> (f64, f64) {
    let (width, height) = size;
    let x = icon.center_x() - width / 2.0;
    let y = if icon.center_y() < work_area.center_y() {
        icon.y + icon.height
    } else {
        icon.y - height
    };
    clamp_into(x, y, work_area, size)
}

/// Places a window of `size` at the top center of the work area. Used when
/// there is no icon position (Linux tray menus, global shortcut).
pub fn place_top_center(work_area: Bounds, size: (f64, f64)) -> (f64, f64) {
    let x = work_area.center_x() - size.0 / 2.0;
    clamp_into(x, work_area.y, work_area, size)
}

fn clamp_into(x: f64, y: f64, area: Bounds, (width, height): (f64, f64)) -> (f64, f64) {
    // `max` before `min` so a window larger than the area sticks to its top-left.
    let x = x.min(area.x + area.width - width).max(area.x);
    let y = y.min(area.y + area.height - height).max(area.y);
    (x, y)
}

fn window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(LABEL)
}

/// Hides the panel whenever it loses focus, and keeps it alive when the user
/// tries to close it (e.g. Alt+F4): the app only quits from the tray menu.
pub fn init(app: &AppHandle) {
    let Some(window) = window(app) else { return };
    let handle = app.clone();
    window.on_window_event(move |event| match event {
        WindowEvent::Focused(false) => {
            if let Some(window) = window_if_visible(&handle) {
                let _ = window.hide();
                *handle
                    .state::<PanelState>()
                    .hidden_by_blur_at
                    .lock()
                    .unwrap() = Some(Instant::now());
            }
        }
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            hide(&handle);
        }
        _ => {}
    });
}

pub fn is_open(app: &AppHandle) -> bool {
    window_if_visible(app).is_some()
}

fn window_if_visible(app: &AppHandle) -> Option<WebviewWindow> {
    window(app).filter(|w| w.is_visible().unwrap_or(false))
}

/// Opens the panel if it is closed and closes it if it is open. `icon` is the
/// tray icon's position, when the platform reports one.
pub fn toggle(app: &AppHandle, icon: Option<Rect>) {
    if window_if_visible(app).is_some() {
        hide(app);
        return;
    }
    let just_hidden = app
        .state::<PanelState>()
        .hidden_by_blur_at
        .lock()
        .unwrap()
        .take()
        .is_some_and(|at| at.elapsed() < REOPEN_GRACE);
    // Only a click on the icon steals focus from the panel before arriving;
    // the global shortcut doesn't, so it always opens.
    let clicked_icon = icon.is_some();
    if !(clicked_icon && just_hidden) {
        show(app, icon);
    }
}

pub fn show(app: &AppHandle, icon: Option<Rect>) {
    let Some(window) = window(app) else { return };
    if let Some(position) = target_position(app, icon) {
        let _ = window.set_position(position);
    }
    let _ = window.show();
    let _ = window.set_focus();
}

pub fn hide(app: &AppHandle) {
    if let Some(window) = window(app) {
        let _ = window.hide();
    }
}

fn target_position(app: &AppHandle, icon: Option<Rect>) -> Option<PhysicalPosition<i32>> {
    // The icon rect is reported in physical pixels; its scale factor only
    // matters if a platform reports logical ones, and then 1.0 is a safe guess.
    let icon = icon.map(|rect| {
        let position = rect.position.to_physical::<f64>(1.0);
        let size = rect.size.to_physical::<f64>(1.0);
        Bounds {
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
        }
    });

    let monitor = match icon {
        Some(icon) => app
            .monitor_from_point(icon.center_x(), icon.center_y())
            .ok()?,
        None => app
            .cursor_position()
            .ok()
            .and_then(|p| app.monitor_from_point(p.x, p.y).ok().flatten()),
    }
    .or_else(|| app.primary_monitor().ok().flatten())?;

    let area = monitor.work_area();
    let work_area = Bounds {
        x: area.position.x as f64,
        y: area.position.y as f64,
        width: area.size.width as f64,
        height: area.size.height as f64,
    };
    let scale = monitor.scale_factor();
    let size = (WINDOW_WIDTH * scale, WINDOW_HEIGHT * scale);

    let (x, y) = match icon {
        Some(icon) => place_near_icon(icon, work_area, size),
        None => place_top_center(work_area, size),
    };
    Some(PhysicalPosition::new(x.round() as i32, y.round() as i32))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIZE: (f64, f64) = (396.0, 536.0);

    fn bounds(x: f64, y: f64, width: f64, height: f64) -> Bounds {
        Bounds {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn below_icon_in_macos_menu_bar() {
        // 1440x900 screen, 25 px menu bar above the work area.
        let icon = bounds(1200.0, 0.0, 30.0, 25.0);
        let area = bounds(0.0, 25.0, 1440.0, 875.0);
        assert_eq!(place_near_icon(icon, area, SIZE), (1017.0, 25.0));
    }

    #[test]
    fn above_icon_in_windows_taskbar() {
        // 1920x1080 screen, 48 px taskbar at the bottom.
        let icon = bounds(1700.0, 1040.0, 32.0, 32.0);
        let area = bounds(0.0, 0.0, 1920.0, 1032.0);
        // Right above the icon would be y = 504, which overlaps the taskbar
        // by 8 px; the panel is pushed up to the edge of the work area.
        assert_eq!(place_near_icon(icon, area, SIZE), (1518.0, 496.0));
    }

    #[test]
    fn kept_inside_screen_when_icon_is_at_the_edge() {
        let icon = bounds(1900.0, 1040.0, 20.0, 32.0);
        let area = bounds(0.0, 0.0, 1920.0, 1032.0);
        let (x, _) = place_near_icon(icon, area, SIZE);
        assert_eq!(x, 1920.0 - SIZE.0);
    }

    #[test]
    fn beside_a_taskbar_on_the_left() {
        // Taskbar on the left: the work area starts at x = 48.
        let icon = bounds(8.0, 1000.0, 32.0, 32.0);
        let area = bounds(48.0, 0.0, 1872.0, 1080.0);
        assert_eq!(place_near_icon(icon, area, SIZE), (48.0, 464.0));
    }

    #[test]
    fn on_a_secondary_monitor() {
        let icon = bounds(3500.0, 0.0, 30.0, 25.0);
        let area = bounds(1920.0, 25.0, 1920.0, 1055.0);
        assert_eq!(place_near_icon(icon, area, SIZE), (3317.0, 25.0));
    }

    #[test]
    fn top_center_without_icon() {
        let area = bounds(0.0, 27.0, 1920.0, 1053.0);
        assert_eq!(place_top_center(area, SIZE), (762.0, 27.0));
    }
}
