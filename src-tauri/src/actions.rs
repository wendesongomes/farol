//! Things the user can do to a process from the panel.

use std::path::Path;

use crate::platform::{Current, Platform, PlatformError, Result};

pub fn kill(pid: u32, force: bool) -> Result<()> {
    // Never let the panel stop Farol itself.
    if pid == std::process::id() {
        return Err(PlatformError::PermissionDenied);
    }
    if force {
        Current::force_kill(pid)
    } else {
        Current::terminate(pid)
    }
}

pub fn open_terminal(cwd: &str) -> Result<()> {
    let cwd = Path::new(cwd);
    if !cwd.is_dir() {
        return Err(PlatformError::NotFound);
    }
    Current::open_terminal(cwd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_to_kill_itself() {
        assert_eq!(
            kill(std::process::id(), true),
            Err(PlatformError::PermissionDenied)
        );
    }
}
