use std::path::Path;

use super::{spawn, unix, Platform, PlatformError, Result};
use crate::process::ProcessInfo;

pub struct Linux;

// Filled in by step 8: recognizing system services.
impl Platform for Linux {
    fn terminate(pid: u32) -> Result<()> {
        unix::terminate(pid)
    }

    fn force_kill(pid: u32) -> Result<()> {
        unix::force_kill(pid)
    }

    fn open_terminal(cwd: &Path) -> Result<()> {
        let (program, args) =
            terminal_command(cwd, spawn::in_path).ok_or(PlatformError::NoTerminal)?;
        let mut command = std::process::Command::new(program);
        command.args(args).current_dir(cwd);
        spawn::detached(command)
    }

    fn is_system_service(_process: &ProcessInfo) -> bool {
        false
    }
}

/// The first installed terminal, in order of preference, with the arguments
/// that make it start in `cwd`. `x-terminal-emulator` (Debian's configurable
/// default) takes no folder argument: it inherits the working directory.
fn terminal_command(
    cwd: &Path,
    installed: impl Fn(&str) -> bool,
) -> Option<(&'static str, Vec<String>)> {
    let cwd = cwd.display().to_string();
    let candidates: [(&str, Vec<String>); 6] = [
        ("x-terminal-emulator", vec![]),
        ("gnome-terminal", vec![format!("--working-directory={cwd}")]),
        ("konsole", vec!["--workdir".into(), cwd.clone()]),
        ("xfce4-terminal", vec![format!("--working-directory={cwd}")]),
        ("kitty", vec!["--directory".into(), cwd.clone()]),
        ("alacritty", vec!["--working-directory".into(), cwd.clone()]),
    ];
    candidates
        .into_iter()
        .find(|(program, _)| installed(program))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_the_system_default_terminal() {
        let (program, args) = terminal_command(Path::new("/w"), |_| true).unwrap();
        assert_eq!(program, "x-terminal-emulator");
        assert!(args.is_empty());
    }

    #[test]
    fn falls_back_in_order() {
        let only = |name: &'static str| move |p: &str| p == name;
        assert_eq!(
            terminal_command(Path::new("/w"), only("konsole")).unwrap(),
            ("konsole", vec!["--workdir".to_string(), "/w".to_string()])
        );
        assert_eq!(
            terminal_command(Path::new("/w"), only("gnome-terminal"))
                .unwrap()
                .1,
            vec!["--working-directory=/w".to_string()]
        );
    }

    #[test]
    fn no_terminal_installed() {
        assert_eq!(terminal_command(Path::new("/w"), |_| false), None);
    }
}
