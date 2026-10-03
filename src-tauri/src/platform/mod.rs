//! Everything that works differently on macOS, Windows and Linux.
//!
//! The rest of Farol only talks to the [`Platform`] trait through
//! [`Current`], the implementation for the system being compiled. Each
//! implementation lives in its own file and is compiled only on its system.

use std::fmt;
use std::path::Path;

use crate::process::ProcessInfo;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(target_os = "linux")]
pub use linux::Linux as Current;
#[cfg(target_os = "macos")]
pub use macos::MacOs as Current;
#[cfg(windows)]
pub use windows::Windows as Current;

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
compile_error!("Farol supports macOS, Windows and Linux.");

pub type Result<T> = std::result::Result<T, PlatformError>;

// `open_terminal` and `is_system_service` get their first callers in steps 7
// and 8.
#[allow(dead_code)]
pub trait Platform {
    /// Asks the process to exit, giving it a chance to clean up
    /// (`SIGTERM` on macOS and Linux, `taskkill` without `/F` on Windows).
    fn terminate(pid: u32) -> Result<()>;

    /// Ends the process immediately (`SIGKILL`, `taskkill /F`).
    fn force_kill(pid: u32) -> Result<()>;

    /// Opens the user's terminal in `cwd`.
    fn open_terminal(cwd: &Path) -> Result<()>;

    /// Whether the process was started by the OS service manager, such as
    /// a database installed as a service.
    fn is_system_service(process: &ProcessInfo) -> bool;
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformError {
    /// The process no longer exists.
    NotFound,
    /// The process belongs to another user or to the system.
    PermissionDenied,
    /// No terminal application could be found.
    NoTerminal,
    /// Not available yet on this system.
    NotImplemented,
    /// Anything else, with the system's own message.
    Other(String),
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "the process no longer exists"),
            Self::PermissionDenied => write!(f, "permission denied"),
            Self::NoTerminal => write!(f, "no terminal application found"),
            Self::NotImplemented => write!(f, "not implemented on this system yet"),
            Self::Other(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for PlatformError {}

impl PlatformError {
    /// Stable identifier the panel uses to pick a translated message.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::NotFound => "not-found",
            Self::PermissionDenied => "permission-denied",
            Self::NoTerminal => "no-terminal",
            Self::NotImplemented => "not-implemented",
            Self::Other(_) => "other",
        }
    }
}

/// Sent to the panel as `{ kind, message }`.
impl serde::Serialize for PlatformError {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("PlatformError", 2)?;
        s.serialize_field("kind", self.kind())?;
        s.serialize_field("message", &self.to_string())?;
        s.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_have_readable_messages() {
        assert_eq!(
            PlatformError::PermissionDenied.to_string(),
            "permission denied"
        );
        assert_eq!(PlatformError::Other("boom".into()).to_string(), "boom");
    }

    #[test]
    fn errors_serialize_with_a_kind() {
        let json = serde_json::to_value(PlatformError::NotFound).unwrap();
        assert_eq!(json["kind"], "not-found");
        assert_eq!(json["message"], "the process no longer exists");
    }

    #[test]
    fn current_platform_compiles_and_answers() {
        // Every implementation must at least answer for an unknown process.
        assert!(!Current::is_system_service(&ProcessInfo::default()));
    }
}
