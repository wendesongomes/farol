//! Who started a process: an AI agent, an editor, a terminal or the system.
//!
//! Farol walks up the process tree from the parent and stops at the first
//! ancestor that matches an agent or a terminal from `rules/detection.toml`.
//! Shells are skipped and only count as "terminal" when nothing else
//! matches. This module works on plain data so it can be tested with
//! simulated process trees for every system.

use crate::process::ProcessInfo;
use crate::rules::{contains_keyword, name_matches, OriginRules, Os, ServiceRules};

pub const TERMINAL: &str = "terminal";
pub const SYSTEM: &str = "system";
pub const UNKNOWN: &str = "unknown";

/// How far up the tree to look. Real chains are a handful of levels deep;
/// the limit only guards against loops.
const MAX_DEPTH: usize = 20;

/// One process in the chain above the server.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Ancestor {
    pub name: String,
    pub command: String,
}

/// The ancestors of `pid`, from its parent upwards.
pub fn ancestors<'a>(pid: u32, lookup: impl Fn(u32) -> Option<&'a ProcessInfo>) -> Vec<Ancestor> {
    let mut chain = Vec::new();
    let mut current = lookup(pid).and_then(|p| p.parent.as_ref()).map(|p| p.pid);
    while let Some(pid) = current {
        if chain.len() >= MAX_DEPTH || pid == 0 {
            break;
        }
        let Some(process) = lookup(pid) else { break };
        chain.push(Ancestor {
            name: process.name.clone(),
            command: process.command.clone(),
        });
        current = process
            .parent
            .as_ref()
            .map(|p| p.pid)
            .filter(|&next| next != pid);
    }
    chain
}

/// The agent id (e.g. "claude-code") or "terminal" of the first recognized
/// ancestor, or `None` when nothing in the chain is recognized.
pub fn from_ancestors(chain: &[Ancestor], os: Os, rules: &OriginRules) -> Option<String> {
    for ancestor in chain.iter().take(MAX_DEPTH) {
        let agent = rules.agents.iter().find(|agent| {
            name_matches(&ancestor.name, &agent.names)
                || agent
                    .command_keywords
                    .iter()
                    .any(|k| contains_keyword(&ancestor.command, k))
        });
        if let Some(agent) = agent {
            return Some(agent.id.clone());
        }
        if name_matches(&ancestor.name, rules.terminals.get(os)) {
            return Some(TERMINAL.to_string());
        }
    }
    let shells = rules.shells.get(os);
    chain
        .iter()
        .any(|a| name_matches(&a.name, shells))
        .then(|| TERMINAL.to_string())
}

/// Whether the OS service manager started the process: its direct parent is
/// one of `parent_names` (and, on macOS, the executable lives under one of
/// `executable_prefixes`, so apps re-parented to launchd don't count).
pub fn is_service(process: &ProcessInfo, rules: &ServiceRules) -> bool {
    let Some(parent) = &process.parent else {
        return false;
    };
    if !name_matches(&parent.name, &rules.parent_names) {
        return false;
    }
    if rules.executable_prefixes.is_empty() {
        return true;
    }
    process.exe.as_ref().is_some_and(|exe| {
        rules
            .executable_prefixes
            .iter()
            .any(|prefix| exe.starts_with(prefix))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::ParentInfo;
    use crate::rules::rules;
    use serde::Deserialize;
    use std::collections::HashMap;

    #[derive(Deserialize)]
    struct Cases {
        origin: Vec<OriginCase>,
    }

    #[derive(Deserialize)]
    struct OriginCase {
        os: String,
        ancestors: Vec<Ancestor>,
        expect: String,
    }

    /// Runs every `[[origin]]` case of rules/detection_tests.toml.
    #[test]
    fn detection_rules_cases() {
        let cases: Cases =
            toml::from_str(include_str!("../../rules/detection_tests.toml")).unwrap();
        assert!(!cases.origin.is_empty());
        for case in cases.origin {
            let os = Os::parse(&case.os).expect("os is macos, windows or linux");
            let got = from_ancestors(&case.ancestors, os, &rules().origin)
                .unwrap_or_else(|| "none".into());
            assert_eq!(got, case.expect, "{} chain: {:?}", case.os, case.ancestors);
        }
    }

    fn process(pid: u32, name: &str, command: &str, parent: Option<(u32, &str)>) -> ProcessInfo {
        ProcessInfo {
            pid,
            name: name.into(),
            command: command.into(),
            parent: parent.map(|(pid, name)| ParentInfo {
                pid,
                name: name.into(),
                command: String::new(),
            }),
            ..Default::default()
        }
    }

    #[test]
    fn walks_the_tree_from_the_parent() {
        let tree: HashMap<u32, ProcessInfo> = [
            process(1, "launchd", "/sbin/launchd", None),
            process(10, "iTerm2", "iTerm2", Some((1, "launchd"))),
            process(
                20,
                "node",
                "node /usr/local/bin/claude",
                Some((10, "iTerm2")),
            ),
            process(30, "zsh", "zsh -c npm run dev", Some((20, "node"))),
            process(40, "node", "node vite", Some((30, "zsh"))),
        ]
        .into_iter()
        .map(|p| (p.pid, p))
        .collect();

        let chain = ancestors(40, |pid| tree.get(&pid));
        let names: Vec<_> = chain.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, ["zsh", "node", "iTerm2", "launchd"]);
        assert_eq!(
            from_ancestors(&chain, Os::Mac, &rules().origin).as_deref(),
            Some("claude-code")
        );
    }

    #[test]
    fn survives_loops_in_the_tree() {
        let tree: HashMap<u32, ProcessInfo> = [
            process(5, "a", "a", Some((6, "b"))),
            process(6, "b", "b", Some((5, "a"))),
        ]
        .into_iter()
        .map(|p| (p.pid, p))
        .collect();
        assert_eq!(ancestors(5, |pid| tree.get(&pid)).len(), MAX_DEPTH);
    }

    fn service_rules(os: Os) -> &'static ServiceRules {
        rules().system_services.get(os)
    }

    #[test]
    fn macos_service_needs_launchd_and_a_system_path() {
        let mut postgres = process(
            80,
            "postgres",
            "postgres -D /opt/homebrew/var",
            Some((1, "launchd")),
        );
        postgres.exe = Some("/opt/homebrew/opt/postgresql@16/bin/postgres".into());
        assert!(is_service(&postgres, service_rules(Os::Mac)));

        // Started with nohup from a project folder, then adopted by launchd.
        let mut adopted = process(81, "node", "node server.js", Some((1, "launchd")));
        adopted.exe = Some("/Users/me/.nvm/versions/node/v20/bin/node".into());
        assert!(!is_service(&adopted, service_rules(Os::Mac)));
    }

    #[test]
    fn windows_service_is_a_child_of_services_exe() {
        let sql = process(
            900,
            "sqlservr.exe",
            "sqlservr.exe",
            Some((700, "services.exe")),
        );
        assert!(is_service(&sql, service_rules(Os::Windows)));
        let node = process(901, "node.exe", "node.exe", Some((800, "explorer.exe")));
        assert!(!is_service(&node, service_rules(Os::Windows)));
    }

    #[test]
    fn linux_service_is_a_child_of_systemd() {
        let redis = process(
            300,
            "redis-server",
            "redis-server *:6379",
            Some((1, "systemd")),
        );
        assert!(is_service(&redis, service_rules(Os::Linux)));
        // `systemd --user` instances have the same name.
        let user = process(301, "syncthing", "syncthing", Some((1200, "systemd")));
        assert!(is_service(&user, service_rules(Os::Linux)));
        let shell = process(
            302,
            "python3",
            "python3 -m http.server",
            Some((1300, "bash")),
        );
        assert!(!is_service(&shell, service_rules(Os::Linux)));
    }
}
