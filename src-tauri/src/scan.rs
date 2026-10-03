//! Builds the list the panel shows: listening ports joined with what the
//! system knows about the process behind each one.
//!
//! The slow parts (asking git, probing ports over HTTP) run in parallel and
//! are cached, so a refresh stays fast with dozens of processes.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, Uid, UpdateKind};

use crate::classify;
use crate::model::{PortProcess, ProcessType, TypeSource, Worktree};
use crate::origin;
use crate::platform::{Current, Platform};
use crate::ports::{self, Listener};
use crate::prefs::{self, Prefs};
use crate::process::{ParentInfo, ProcessInfo};
use crate::rules::{rules, Os};
use crate::worktree::WorktreeCache;

/// A PID plus its start time: still the same process after the system
/// reuses the PID for something else.
type ProcessId = (u32, u64);

pub struct Scanner {
    /// Kept between scans so sysinfo only reads new processes' details
    /// (command line, working directory) once.
    system: Mutex<System>,
    worktrees: WorktreeCache,
    /// Detected type per process: keywords and the HTTP probe run once.
    types: Mutex<HashMap<ProcessId, ProcessType>>,
    /// Origin per process: the tree above a process doesn't change.
    origins: Mutex<HashMap<ProcessId, String>>,
}

/// A process as seen during one scan, plus who owns it.
struct Snapshot {
    info: ProcessInfo,
    mine: bool,
}

impl Snapshot {
    fn id(&self) -> ProcessId {
        (self.info.pid, self.info.start_time)
    }
}

impl Default for Scanner {
    fn default() -> Self {
        Self {
            system: Mutex::new(System::new()),
            worktrees: WorktreeCache::default(),
            types: Mutex::new(HashMap::new()),
            origins: Mutex::new(HashMap::new()),
        }
    }
}

impl Scanner {
    pub fn scan(&self, prefs: &Prefs) -> Vec<PortProcess> {
        let listeners = ports::listening();
        let processes = self.snapshot();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or_default();

        let rows: Vec<(Listener, Option<&Snapshot>)> = listeners
            .into_iter()
            .map(|listener| (listener, listener.pid.and_then(|pid| processes.get(&pid))))
            .collect();

        let folders: Vec<_> = rows
            .iter()
            .filter_map(|(_, s)| s.and_then(|s| s.info.cwd.clone()))
            .collect();
        let worktrees = self.worktrees.resolve_all(&folders);
        let types = self.detect_types(&rows, &processes);
        let origins = self.detect_origins(&rows, &processes);

        let mut list: Vec<PortProcess> = rows
            .into_iter()
            .map(|(listener, snapshot)| {
                let worktree = snapshot
                    .and_then(|s| s.info.cwd.as_ref())
                    .and_then(|cwd| worktrees.get(cwd).cloned().flatten());
                let detected = snapshot
                    .and_then(|s| types.get(&s.id()).copied())
                    .unwrap_or(ProcessType::Back);
                let mut row = build(listener, snapshot, worktree, now);
                if let Some(origin) = snapshot.and_then(|s| origins.get(&s.id())) {
                    row.origin_label = agent_label(origin);
                    row.origin = origin.clone();
                }
                apply_prefs(&mut row, detected, prefs);
                row
            })
            .collect();
        list.sort_by_key(|p| (p.port, p.pid));
        list
    }

    /// Detects the type of every listening process not seen before:
    /// keywords first, then (in parallel) the HTTP probe.
    fn detect_types(
        &self,
        rows: &[(Listener, Option<&Snapshot>)],
        processes: &HashMap<u32, Snapshot>,
    ) -> HashMap<ProcessId, ProcessType> {
        let mut cache = self.types.lock().unwrap();
        // Forget processes that are gone.
        cache.retain(|(pid, start), _| {
            processes
                .get(pid)
                .is_some_and(|s| s.info.start_time == *start)
        });

        let mut to_probe: Vec<(ProcessId, u16)> = Vec::new();
        for (listener, snapshot) in rows {
            let Some(snapshot) = snapshot else { continue };
            let id = snapshot.id();
            if cache.contains_key(&id) || to_probe.iter().any(|(other, _)| *other == id) {
                continue;
            }
            match classify::from_keywords(&snapshot.info.command, &rules().kind) {
                Some(kind) => {
                    cache.insert(id, kind);
                }
                None => to_probe.push((id, listener.port)),
            }
        }

        let probed: Vec<(ProcessId, ProcessType)> = std::thread::scope(|scope| {
            let handles: Vec<_> = to_probe
                .iter()
                .map(|&(id, port)| scope.spawn(move || (id, classify::from_probe(port))))
                .collect();
            handles.into_iter().filter_map(|h| h.join().ok()).collect()
        });
        cache.extend(probed);
        cache.clone()
    }

    /// Finds who started each listening process not seen before.
    fn detect_origins(
        &self,
        rows: &[(Listener, Option<&Snapshot>)],
        processes: &HashMap<u32, Snapshot>,
    ) -> HashMap<ProcessId, String> {
        let mut cache = self.origins.lock().unwrap();
        cache.retain(|(pid, start), _| {
            processes
                .get(pid)
                .is_some_and(|s| s.info.start_time == *start)
        });
        for snapshot in rows.iter().filter_map(|(_, s)| *s) {
            cache.entry(snapshot.id()).or_insert_with(|| {
                let chain = origin::ancestors(snapshot.info.pid, |pid| {
                    processes.get(&pid).map(|s| &s.info)
                });
                origin::from_ancestors(&chain, Os::CURRENT, &rules().origin)
                    .or_else(|| {
                        Current::is_system_service(&snapshot.info).then(|| origin::SYSTEM.into())
                    })
                    // Daemons and `nohup` servers adopted by the root process
                    // have no recognizable ancestor: that is expected.
                    .unwrap_or_else(|| origin::UNKNOWN.into())
            });
        }
        cache.clone()
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
    listener: Listener,
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
        key: prefs::key(
            worktree.as_ref().map(|w| w.root.as_str()),
            listener.port,
            &name,
        ),
        pid: listener.pid,
        port: listener.port,
        name,
        command,
        cwd,
        worktree,
        kind: ProcessType::Back,
        type_source: TypeSource::Detected,
        origin: origin::UNKNOWN.into(),
        origin_label: None,
        uptime_seconds: uptime,
        pinned: false,
        can_kill,
    }
}

fn agent_label(id: &str) -> Option<String> {
    rules()
        .origin
        .agents
        .iter()
        .find(|agent| agent.id == id)
        .and_then(|agent| agent.label.clone())
}

/// The user's choices win over detection.
fn apply_prefs(row: &mut PortProcess, detected: ProcessType, prefs: &Prefs) {
    match prefs.type_overrides.get(&row.key) {
        Some(&kind) => {
            row.kind = kind;
            row.type_source = TypeSource::Manual;
        }
        None => {
            row.kind = detected;
            row.type_source = TypeSource::Detected;
        }
    }
    row.pinned = prefs.pins.contains(&row.key);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_a_port_opened_by_this_test() {
        let socket = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = socket.local_addr().unwrap().port();

        let list = Scanner::default().scan(&Prefs::default());
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
        let listener = Listener {
            port: 631,
            pid: None,
        };
        let entry = build(listener, None, None, 100);
        assert!(!entry.can_kill);
        assert_eq!(entry.name, "");
        assert_eq!(entry.key, "none:631:");
    }

    #[test]
    fn manual_type_and_pin_win() {
        let listener = Listener {
            port: 3000,
            pid: None,
        };
        let mut row = build(listener, None, None, 0);
        let mut prefs = Prefs::default();
        prefs.set_type(&row.key, ProcessType::Front);
        prefs.toggle_pin(&row.key);

        apply_prefs(&mut row, ProcessType::Back, &prefs);
        assert_eq!(row.kind, ProcessType::Front);
        assert_eq!(row.type_source, TypeSource::Manual);
        assert!(row.pinned);

        apply_prefs(&mut row, ProcessType::Back, &Prefs::default());
        assert_eq!(row.kind, ProcessType::Back);
        assert_eq!(row.type_source, TypeSource::Detected);
        assert!(!row.pinned);
    }
}
