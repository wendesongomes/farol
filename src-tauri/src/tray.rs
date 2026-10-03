//! The tray / menu bar icon and its menu.

use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

use crate::{i18n, panel};

pub const ID: &str = "main";

const MENU_OPEN: &str = "open";
const MENU_AUTOSTART: &str = "autostart";
const MENU_QUIT: &str = "quit";

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let strings = i18n::system_tray_strings();
    let autostart_enabled = app.autolaunch().is_enabled().unwrap_or(false);
    let autostart = CheckMenuItem::with_id(
        app,
        MENU_AUTOSTART,
        strings.start_at_login,
        true,
        autostart_enabled,
        None::<&str>,
    )?;
    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, MENU_OPEN, strings.open_panel, true, None::<&str>)?,
            &autostart,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, MENU_QUIT, strings.quit, true, None::<&str>)?,
        ],
    )?;

    TrayIconBuilder::with_id(ID)
        .icon(icon()?)
        .icon_as_template(cfg!(target_os = "macos"))
        .tooltip("Farol")
        .menu(&menu)
        // On macOS and Windows a left click toggles the panel and the menu is
        // on the right click. Most Linux trays (AppIndicator) never send click
        // events and always show the menu, so it holds "Open panel" there.
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            MENU_OPEN => panel::show(app, None),
            MENU_AUTOSTART => toggle_autostart(app, &autostart),
            MENU_QUIT => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            {
                panel::toggle(tray.app_handle(), Some(rect));
            }
        })
        .build(app)?;
    Ok(())
}

/// Flips "start at login" and makes the checkmark reflect what really
/// happened, in case the system refused the change.
fn toggle_autostart(app: &AppHandle, item: &CheckMenuItem<tauri::Wry>) {
    let manager = app.autolaunch();
    let enabled = manager.is_enabled().unwrap_or(false);
    let result = if enabled {
        manager.disable()
    } else {
        manager.enable()
    };
    if let Err(error) = result {
        eprintln!("farol: could not change start at login: {error}");
    }
    let _ = item.set_checked(manager.is_enabled().unwrap_or(enabled));
}

/// macOS gets a black glyph that the system tints for light and dark menu
/// bars ("template image"). Windows and Linux can't do that and their trays
/// can be light or dark, so they get the colored app icon, visible on both.
fn icon() -> tauri::Result<Image<'static>> {
    #[cfg(target_os = "macos")]
    let bytes = include_bytes!("../icons/tray/template.png");
    #[cfg(not(target_os = "macos"))]
    let bytes = include_bytes!("../icons/tray/color.png");
    Image::from_bytes(bytes)
}
