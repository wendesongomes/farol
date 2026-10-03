mod actions;
mod classify;
mod command;
mod commands;
mod i18n;
mod model;
mod origin;
mod panel;
mod platform;
mod ports;
mod prefs;
mod process;
mod rules;
mod scan;
mod tray;
mod worktree;

use std::sync::Arc;

use tauri::{Manager, RunEvent};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};

/// Opens the panel from anywhere: Ctrl+Alt+P, or Cmd+Option+P on macOS. It is
/// the only way to toggle the panel with one action on Linux trays that don't
/// report clicks.
fn panel_shortcut() -> Shortcut {
    #[cfg(target_os = "macos")]
    let modifiers = Modifiers::SUPER | Modifiers::ALT;
    #[cfg(not(target_os = "macos"))]
    let modifiers = Modifiers::CONTROL | Modifiers::ALT;
    Shortcut::new(Some(modifiers), Code::KeyP)
}

pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state == ShortcutState::Pressed && *shortcut == panel_shortcut() {
                        panel::toggle(app, None);
                    }
                })
                .build(),
        )
        .manage(panel::PanelState::default())
        .manage(Arc::new(scan::Scanner::default()))
        .invoke_handler(tauri::generate_handler![
            commands::list_processes,
            commands::kill_process,
            commands::open_in_browser,
            commands::open_terminal,
            commands::set_type_override,
            commands::toggle_pin
        ])
        .setup(|app| {
            // Menu bar app: no Dock icon and no app menu on macOS.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            app.manage(prefs::PrefsState::load(app.handle()));
            panel::init(app.handle());
            tray::create(app.handle())?;

            // Another app may already own the shortcut. Farol still works
            // through the tray icon, so this is not fatal.
            use tauri_plugin_global_shortcut::GlobalShortcutExt;
            if let Err(error) = app.global_shortcut().register(panel_shortcut()) {
                eprintln!("farol: could not register the global shortcut: {error}");
            }

            // In development, open the panel right away so changes are visible.
            if cfg!(debug_assertions) {
                panel::show(app.handle(), None);
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building Farol");

    app.run(|_app, event| {
        // The app lives in the tray: it only quits from the tray menu, which
        // calls `exit` with an explicit code.
        if let RunEvent::ExitRequested {
            code: None, api, ..
        } = event
        {
            api.prevent_exit();
        }
    });
}
