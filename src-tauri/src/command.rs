//! Running external programs (`git`, and the native port listings used as a
//! fallback).

use std::ffi::OsStr;
use std::process::{Command, Stdio};

/// A `Command` that never flashes a console window on Windows. Farol runs
/// commands every few seconds, so without this the screen would blink.
pub fn quiet<S: AsRef<OsStr>>(program: S) -> Command {
    #[allow(unused_mut)]
    let mut command = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command.stdin(Stdio::null());
    command
}

/// Runs a program and returns its standard output, or `None` if it could not
/// be started or exited with an error.
pub fn output<S: AsRef<OsStr>>(program: S, args: &[&str]) -> Option<String> {
    let output = quiet(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}
