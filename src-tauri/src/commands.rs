//! The functions the panel calls with `invoke(...)`.

use std::sync::Arc;

use tauri::State;

use crate::model::PortProcess;
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
