//! The functions the panel calls with `invoke(...)`.

use std::sync::Arc;

use tauri::State;

use crate::actions;
use crate::model::PortProcess;
use crate::platform::PlatformError;
use crate::scan::Scanner;

/// Reading every process takes a few milliseconds to a few hundred; it runs
/// on a worker thread so the UI never waits on it.
#[tauri::command]
pub async fn list_processes(scanner: State<'_, Arc<Scanner>>) -> Result<Vec<PortProcess>, String> {
    let scanner = Arc::clone(&scanner);
    tauri::async_runtime::spawn_blocking(move || scanner.scan())
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
