//! Shared by macOS and Linux: both stop processes with POSIX signals.

use super::{PlatformError, Result};

pub fn signal(pid: u32, signal: libc::c_int) -> Result<()> {
    let pid = libc::pid_t::try_from(pid).map_err(|_| PlatformError::NotFound)?;
    // pid 0 and negative pids address process groups; never send those.
    if pid <= 0 {
        return Err(PlatformError::NotFound);
    }
    // SAFETY: kill(2) has no memory-safety requirements.
    if unsafe { libc::kill(pid, signal) } == 0 {
        return Ok(());
    }
    let error = std::io::Error::last_os_error();
    Err(match error.raw_os_error() {
        Some(libc::ESRCH) => PlatformError::NotFound,
        Some(libc::EPERM) => PlatformError::PermissionDenied,
        _ => PlatformError::Other(error.to_string()),
    })
}

pub fn terminate(pid: u32) -> Result<()> {
    signal(pid, libc::SIGTERM)
}

pub fn force_kill(pid: u32) -> Result<()> {
    signal(pid, libc::SIGKILL)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn terminates_a_child_process() {
        let mut child = Command::new("sleep").arg("30").spawn().unwrap();
        terminate(child.id()).unwrap();
        let status = child.wait().unwrap();
        assert!(!status.success());
    }

    #[test]
    fn reports_a_missing_process() {
        // PIDs are capped well below this on every supported system.
        assert_eq!(terminate(i32::MAX as u32), Err(PlatformError::NotFound));
    }

    #[test]
    fn reports_permission_denied_for_other_users() {
        // PID 1 belongs to root; only meaningful when not running as root.
        if unsafe { libc::geteuid() } != 0 {
            assert_eq!(signal(1, 0), Err(PlatformError::PermissionDenied));
        }
    }
}
