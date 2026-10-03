//! Plain data about a running process, collected once from the operating
//! system and then handed to the classification code. Keeping this a dumb
//! struct is what lets the detection logic be tested on any system.

use std::path::PathBuf;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProcessInfo {
    pub pid: u32,
    /// Executable name, e.g. `node` or `node.exe`.
    pub name: String,
    /// Full path of the executable, when the system tells us.
    pub exe: Option<PathBuf>,
    /// Full command line, arguments joined by spaces.
    pub command: String,
    /// Working directory, when the system tells us.
    pub cwd: Option<PathBuf>,
    pub parent: Option<ParentInfo>,
    /// Start time in seconds since the Unix epoch. Together with the PID it
    /// identifies a process even after the system reuses the PID.
    pub start_time: u64,
}

/// The direct parent of a process, enough to recognize service managers
/// (`launchd`, `services.exe`, `systemd`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParentInfo {
    pub pid: u32,
    pub name: String,
    pub command: String,
}
