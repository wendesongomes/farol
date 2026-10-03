//! Builds the list the panel shows: listening ports joined with what the
//! system knows about the process behind each one.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, Uid, UpdateKind};

use crate::model::{Origin, PortProcess, ProcessType, TypeSource, Worktree};
use crate::ports;
use crate::process::{ParentInfo, ProcessInfo};
use crate::worktree::WorktreeCache;

pub struct Scanner {
    /// Kept between scans so sysinfo only reads new processes' details
    /// (command line, working directory) once.
    system: Mutex<System>,
    worktrees: WorktreeCache,
}

/// A process as seen during one scan, plus who owns it.
struct Snapshot {
    info: ProcessInfo,
    mine: bool,
}

impl Default for Scanner {
    fn default() -> Self {
        Self {
            system: Mutex::new(System::new()),
            worktrees: WorktreeCache::default(),
        }
    }
}

impl Scanner {
    pub fn scan(&self) -> Vec<PortProcess> {
        let listeners = ports::listening();
        let processes = self.snapshot();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or_default();

        let snapshots: Vec<_> = listeners
            .into_iter()
            .map(|listener| (listener, listener.pid.and_then(|pid| processes.get(&pid))))
            .collect();

        let folders: Vec<_> = snapshots
            .iter()
            .filter_map(|(_, s)| s.and_then(|s| s.info.cwd.clone()))
            .collect();
        let worktrees = self.worktrees.resolve_all(&folders);

        let mut list: Vec<PortProcess> = snapshots
            .into_iter()
            .map(|(listener, snapshot)| {
                let worktree = snapshot
                    .and_then(|s| s.info.cwd.as_ref())
                    .and_then(|cwd| worktrees.get(cwd).cloned().flatten());
                build(listener, snapshot, worktree, now)
            })
            .collect();
        list.sort_by_key(|p| (p.port, p.pid));
        list
    }

    fn snapshot(&self) -> HashMap<u32, Snapshot> {
        let mut system = self.system.lock().unwrap();
        system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing()
                .with_cmd(UpdateKind::OnlyIfNotSet)
                .with_exe(UpdateKind::OnlyIfNotSet)
                .with_cwd(UpdateKind::OnlyIfNotSet)
                .with_user(UpdateKind::OnlyIfNotSet),
        );

        let me: Option<Uid> = sysinfo::get_current_pid()
            .ok()
            .and_then(|pid| system.process(pid))
            .and_then(|p| p.user_id().cloned());

        system
            .processes()
            .iter()
            .map(|(pid, process)| {
                let parent = process.parent().and_then(|ppid| {
                    system.process(ppid).map(|p| ParentInfo {
                        pid: ppid.as_u32(),
                        name: p.name().to_string_lossy().into_owned(),
                        command: join_command(p),
                    })
                });
                let info = ProcessInfo {
                    pid: pid.as_u32(),
                    name: process.name().to_string_lossy().into_owned(),
                    exe: process.exe().map(Into::into),
                    command: join_command(process),
                    cwd: process.cwd().map(Into::into),
                    parent,
                    start_time: process.start_time(),
                };
                let mine = me.is_some() && process.user_id() == me.as_ref();
                (pid.as_u32(), Snapshot { info, mine })
            })
            .collect()
    }
}

fn join_command(process: &sysinfo::Process) -> String {
    process
        .cmd()
        .iter()
        .map(|arg| arg.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ")
}

fn build(
    listener: ports::Listener,
    snapshot: Option<&Snapshot>,
    worktree: Option<Worktree>,
    now: u64,
) -> PortProcess {
    let (name, command, cwd, uptime, can_kill) = match snapshot {
        Some(Snapshot { info, mine }) => {
            // Without permission the command line comes back empty; the
            // executable path or name is the next best thing.
            let command = if info.command.is_empty() {
                info.exe
                    .as_ref()
                    .map(|exe| exe.display().to_string())
                    .unwrap_or_else(|| info.name.clone())
            } else {
                info.command.clone()
            };
            (
                info.name.clone(),
                command,
                info.cwd.as_ref().map(|cwd| cwd.display().to_string()),
                now.saturating_sub(info.start_time),
                *mine,
            )
        }
        None => (String::new(), String::new(), None, 0, false),
    };

    PortProcess {
        pid: listener.pid,
        port: listener.port,
        name,
        command,
        cwd,
        worktree,
        kind: ProcessType::Back,
        type_source: TypeSource::Detected,
        origin: Origin::Unknown,
        uptime_seconds: uptime,
        pinned: false,
        can_kill,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_a_port_opened_by_this_test() {
        let socket = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = socket.local_addr().unwrap().port();

        let list = Scanner::default().scan();
        let entry = list
            .iter()
            .find(|p| p.port == port && p.pid == Some(std::process::id()))
            .expect("the test's own port is listed");
        assert!(!entry.name.is_empty());
        assert!(
            entry.can_kill,
            "a process of the current user can be killed"
        );
        assert!(entry.cwd.is_some());
    }

    #[test]
    fn unknown_owner_cannot_be_killed() {
        let listener = ports::Listener {
            port: 631,
            pid: None,
        };
        let entry = build(listener, None, None, 100);
        assert!(!entry.can_kill);
        assert_eq!(entry.name, "");
    }
}
