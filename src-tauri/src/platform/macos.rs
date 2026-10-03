use std::path::Path;

use super::{spawn, unix, Platform, PlatformError, Result};
use crate::origin;
use crate::process::ProcessInfo;
use crate::rules::{rules, Os};

pub struct MacOs;

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
        spawn::detached(command).map_err(|_| PlatformError::NoTerminal)
    }

    fn is_system_service(process: &ProcessInfo) -> bool {
        origin::is_service(process, rules().system_services.get(Os::Mac))
    }
}
