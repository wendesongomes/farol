//! What the panel receives for each listening port. Mirrors `src/types.ts`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProcessType {
    Front,
    Back,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TypeSource {
    Detected,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Worktree {
    pub root: String,
    pub branch: Option<String>,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortProcess {
    /// Identifies the process across restarts, for pins and type overrides:
    /// see `prefs::key`.
    pub key: String,
    /// `None` when the system doesn't say who owns the port.
    pub pid: Option<u32>,
    pub port: u16,
    pub name: String,
    pub command: String,
    pub cwd: Option<String>,
    pub worktree: Option<Worktree>,
    #[serde(rename = "type")]
    pub kind: ProcessType,
    pub type_source: TypeSource,
    /// "claude-code", "cursor", "vscode", any other agent id from
    /// rules/detection.toml, "terminal", "system" or "unknown".
    pub origin: String,
    /// Display name for agents that only exist in the rules file.
    pub origin_label: Option<String>,
    pub uptime_seconds: u64,
    pub pinned: bool,
    pub can_kill: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_like_the_typescript_model() {
        let process = PortProcess {
            key: "none:3000:node".into(),
            pid: Some(42),
            port: 3000,
            name: "node".into(),
            command: "node vite".into(),
            cwd: None,
            worktree: None,
            kind: ProcessType::Front,
            type_source: TypeSource::Detected,
            origin: "claude-code".into(),
            origin_label: None,
            uptime_seconds: 5,
            pinned: false,
            can_kill: true,
        };
        let json = serde_json::to_value(&process).unwrap();
        assert_eq!(json["type"], "front");
        assert_eq!(json["typeSource"], "detected");
        assert_eq!(json["origin"], "claude-code");
        assert_eq!(json["uptimeSeconds"], 5);
        assert_eq!(json["canKill"], true);
    }
}
