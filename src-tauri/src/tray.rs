//! The tray / menu bar icon and its menu.

use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::AppHandle;

use crate::{i18n, panel};

const MENU_OPEN: &str = "open";
const MENU_QUIT: &str = "quit";

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let strings = i18n::system_tray_strings();
    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, MENU_OPEN, strings.open_panel, true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, MENU_QUIT, strings.quit, true, None::<&str>)?,
        ],
    )?;

    TrayIconBuilder::with_id("main")
        .icon(icon()?)
        .icon_as_template(cfg!(target_os = "macos"))
        .tooltip("Farol")
        .menu(&menu)
        // On macOS and Windows a left click toggles the panel and the menu is
        // on the right click. Most Linux trays (AppIndicator) never send click
        // events and always show the menu, so it holds "Open panel" there.
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            MENU_OPEN => panel::show(app, None),
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
