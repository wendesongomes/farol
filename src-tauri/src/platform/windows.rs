use std::path::Path;

use super::{Platform, PlatformError, Result};
use crate::process::ProcessInfo;

pub struct Windows;

// Filled in by the following steps: killing (step 4), opening a terminal
// (step 7) and recognizing system services (step 8).
impl Platform for Windows {
    fn terminate(_pid: u32) -> Result<()> {
        Err(PlatformError::NotImplemented)
    }

    fn force_kill(_pid: u32) -> Result<()> {
        Err(PlatformError::NotImplemented)
    }

    fn open_terminal(_cwd: &Path) -> Result<()> {
        Err(PlatformError::NotImplemented)
    }

    fn is_system_service(_process: &ProcessInfo) -> bool {
        false
    }
}
