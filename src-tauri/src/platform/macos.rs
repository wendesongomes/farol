use std::path::Path;

use super::{spawn, unix, Platform, Result};
use crate::process::ProcessInfo;

pub struct MacOs;

// Filled in by step 8: recognizing system services.
impl Platform for MacOs {
    fn terminate(pid: u32) -> Result<()> {
        unix::terminate(pid)
    }

    fn force_kill(pid: u32) -> Result<()> {
        unix::force_kill(pid)
    }

    fn open_terminal(cwd: &Path) -> Result<()> {
        let mut command = std::process::Command::new("open");
        command.args(["-a", "Terminal"]).arg(cwd);
        spawn::detached(command)
    }

    fn is_system_service(_process: &ProcessInfo) -> bool {
        false
    }
}
