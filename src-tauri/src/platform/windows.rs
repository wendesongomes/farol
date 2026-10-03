use std::path::Path;

use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ACCESS_DENIED};
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_TERMINATE};

use super::{spawn, Platform, PlatformError, Result};
use crate::command;
use crate::origin;
use crate::process::ProcessInfo;
use crate::rules::{rules, Os};

pub struct Windows;

/// Checks that the process exists and that we may stop it. `taskkill`'s own
/// error messages are translated into the system language, so they can't be
/// used to tell these cases apart.
fn check_access(pid: u32) -> Result<()> {
    // SAFETY: plain Win32 calls; the handle is closed before returning.
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if handle.is_null() {
            return Err(match GetLastError() {
                ERROR_ACCESS_DENIED => PlatformError::PermissionDenied,
                // ERROR_INVALID_PARAMETER: no process with this PID.
                _ => PlatformError::NotFound,
            });
        }
        CloseHandle(handle);
    }
    Ok(())
}

fn taskkill(pid: u32, force: bool) -> std::io::Result<std::process::Output> {
    let pid = pid.to_string();
    let mut args = vec!["/PID", pid.as_str()];
    if force {
        args.insert(0, "/F");
    }
    command::quiet("taskkill").args(args).output()
}

impl Platform for Windows {
    /// `taskkill` without `/F` posts a close message to the process' windows.
    /// Console programs (node, python) usually have none and keep running;
    /// that isn't an error here: the panel notices the port is still open
    /// and offers to force it.
    fn terminate(pid: u32) -> Result<()> {
        check_access(pid)?;
        taskkill(pid, false).map_err(|e| PlatformError::Other(e.to_string()))?;
        Ok(())
    }

    fn force_kill(pid: u32) -> Result<()> {
        check_access(pid)?;
        let output = taskkill(pid, true).map_err(|e| PlatformError::Other(e.to_string()))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(PlatformError::Other(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ))
        }
    }

    /// Windows Terminal when installed, the classic console otherwise.
    fn open_terminal(cwd: &Path) -> Result<()> {
        let mut wt = command::quiet("wt");
        wt.arg("-d").arg(cwd);
        if spawn::detached(wt).is_ok() {
            return Ok(());
        }
        // `start` opens a new console window for the inner `cmd`.
        let mut cmd = command::quiet("cmd");
        cmd.args(["/c", "start", "", "cmd", "/K", "cd", "/d"])
            .arg(cwd);
        spawn::detached(cmd).map_err(|_| PlatformError::NoTerminal)
    }

    fn is_system_service(process: &ProcessInfo) -> bool {
        origin::is_service(process, rules().system_services.get(Os::Windows))
    }
}
