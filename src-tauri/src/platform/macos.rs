use std::path::Path;

use super::{unix, Platform, PlatformError, Result};
use crate::process::ProcessInfo;

pub struct MacOs;

// Filled in by the following steps: opening a terminal (step 7) and
// recognizing system services (step 8).
impl Platform for MacOs {
    fn terminate(pid: u32) -> Result<()> {
        unix::terminate(pid)
    }

    fn force_kill(pid: u32) -> Result<()> {
        unix::force_kill(pid)
    }

    fn open_terminal(_cwd: &Path) -> Result<()> {
        Err(PlatformError::NotImplemented)
    }

    fn is_system_service(_process: &ProcessInfo) -> bool {
        false
    }
}
