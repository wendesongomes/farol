//! Launching GUI apps (terminals) without blocking or leaving zombies.

use std::process::Command;

use super::{PlatformError, Result};

/// Starts the command and reaps it in the background when it exits.
pub fn detached(mut command: Command) -> Result<()> {
    let mut child = command
        .spawn()
        .map_err(|error| PlatformError::Other(error.to_string()))?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

/// Whether an executable with this name is in `PATH`.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn in_path(program: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(program).is_file()))
        .unwrap_or(false)
}
