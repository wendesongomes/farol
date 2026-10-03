//! The functions the panel calls with `invoke(...)`.

use std::sync::Arc;

use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use crate::actions;
use crate::model::{PortProcess, ProcessType};
use crate::platform::PlatformError;
use crate::prefs::PrefsState;
use crate::scan::Scanner;

/// Reading every process takes a few milliseconds to a few hundred; it runs
/// on a worker thread so the UI never waits on it.
#[tauri::command]
pub async fn list_processes(
    scanner: State<'_, Arc<Scanner>>,
    prefs: State<'_, PrefsState>,
) -> Result<Vec<PortProcess>, String> {
    let scanner = Arc::clone(&scanner);
    let prefs = prefs.snapshot();
    tauri::async_runtime::spawn_blocking(move || scanner.scan(&prefs))
        .await
        .map_err(|error| error.to_string())
}

/// Asks the process to stop, or ends it immediately when `force` is set.
#[tauri::command]
pub async fn kill_process(pid: u32, force: bool) -> Result<(), PlatformError> {
    tauri::async_runtime::spawn_blocking(move || actions::kill(pid, force))
        .await
        .map_err(|error| PlatformError::Other(error.to_string()))?
}

/// Opens `http://localhost:<port>` in the default browser.
#[tauri::command]
pub fn open_in_browser(app: AppHandle, port: u16) -> Result<(), PlatformError> {
    app.opener()
        .open_url(format!("http://localhost:{port}"), None::<&str>)
        .map_err(|error| PlatformError::Other(error.to_string()))
}

#[tauri::command]
pub fn open_terminal(cwd: String) -> Result<(), PlatformError> {
    actions::open_terminal(&cwd)
}

/// Remembers the user's front/back choice for this process (see `prefs::key`).
#[tauri::command]
pub fn set_type_override(
    app: AppHandle,
    prefs: State<'_, PrefsState>,
    key: String,
    kind: ProcessType,
) {
    prefs.update(&app, |p| p.set_type(&key, kind));
}

/// Pins or unpins; returns whether it is now pinned.
#[tauri::command]
pub fn toggle_pin(app: AppHandle, prefs: State<'_, PrefsState>, key: String) -> bool {
    prefs.update(&app, |p| p.toggle_pin(&key))
}
