//! What the panel receives for each listening port. Mirrors `src/types.ts`.

use serde::Serialize;

// Variants are filled in by detection (steps 7 and 8).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProcessType {
    Front,
    Back,
}

// Variants are filled in by detection (steps 7 and 8).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TypeSource {
    Detected,
    Manual,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Origin {
    ClaudeCode,
    Cursor,
    Vscode,
    Terminal,
    System,
    Unknown,
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
    pub origin: Origin,
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
            pid: Some(42),
            port: 3000,
            name: "node".into(),
            command: "node vite".into(),
            cwd: None,
            worktree: None,
            kind: ProcessType::Front,
            type_source: TypeSource::Detected,
            origin: Origin::ClaudeCode,
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
