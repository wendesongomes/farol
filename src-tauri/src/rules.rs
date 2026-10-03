//! The detection rules from `rules/detection.toml`, embedded at build time.
//!
//! Everything a contributor might want to tweak (keywords, agent and terminal
//! names) lives in that file; this module only loads it and provides the
//! matching helpers the rules' comments describe.

use std::sync::OnceLock;

use serde::Deserialize;

const DETECTION_TOML: &str = include_str!("../../rules/detection.toml");

#[derive(Debug, Deserialize)]
pub struct Rules {
    #[serde(rename = "type")]
    pub kind: TypeRules,
    pub origin: OriginRules,
    pub system_services: PerOs<ServiceRules>,
}

#[derive(Debug, Deserialize)]
pub struct TypeRules {
    pub front: Keywords,
    pub back: Keywords,
}

#[derive(Debug, Deserialize)]
pub struct Keywords {
    pub keywords: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct OriginRules {
    pub agents: Vec<AgentRule>,
    pub terminals: PerOs<Vec<String>>,
    pub shells: PerOs<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct AgentRule {
    pub id: String,
    pub names: Vec<String>,
    pub command_keywords: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ServiceRules {
    pub parent_names: Vec<String>,
    #[serde(default)]
    pub executable_prefixes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct PerOs<T> {
    pub macos: T,
    pub windows: T,
    pub linux: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Mac,
    Windows,
    Linux,
}

impl Os {
    pub const CURRENT: Os = if cfg!(target_os = "macos") {
        Os::Mac
    } else if cfg!(windows) {
        Os::Windows
    } else {
        Os::Linux
    };

    pub fn parse(name: &str) -> Option<Os> {
        match name {
            "macos" => Some(Os::Mac),
            "windows" => Some(Os::Windows),
            "linux" => Some(Os::Linux),
            _ => None,
        }
    }
}

impl<T> PerOs<T> {
    pub fn get(&self, os: Os) -> &T {
        match os {
            Os::Mac => &self.macos,
            Os::Windows => &self.windows,
            Os::Linux => &self.linux,
        }
    }
}

impl Rules {
    pub fn parse(text: &str) -> Result<Rules, toml::de::Error> {
        toml::from_str(text)
    }
}

/// The embedded rules. A broken `detection.toml` fails the tests (and CI)
/// long before it could reach a release.
pub fn rules() -> &'static Rules {
    static RULES: OnceLock<Rules> = OnceLock::new();
    RULES.get_or_init(|| Rules::parse(DETECTION_TOML).expect("rules/detection.toml is valid"))
}

/// Executable names are compared without case and without `.exe`.
pub fn normalize_name(name: &str) -> String {
    let lower = name.trim().to_lowercase();
    lower
        .strip_suffix(".exe")
        .map(str::to_string)
        .unwrap_or(lower)
}

pub fn name_matches(name: &str, candidates: &[String]) -> bool {
    let name = normalize_name(name);
    candidates.iter().any(|c| normalize_name(c) == name)
}

/// Whether `keyword` appears in `text` as a whole word or phrase: the
/// characters around it must not be letters or digits. Case-insensitive.
pub fn contains_keyword(text: &str, keyword: &str) -> bool {
    let text = text.to_lowercase();
    let keyword = keyword.to_lowercase();
    if keyword.is_empty() {
        return false;
    }
    let is_word = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric());
    let mut start = 0;
    while let Some(found) = text[start..].find(&keyword) {
        let at = start + found;
        let end = at + keyword.len();
        let before = text[..at].chars().next_back();
        let after = text[end..].chars().next();
        if !is_word(before) && !is_word(after) {
            return true;
        }
        // Advance by one character, staying on a char boundary.
        start = at + text[at..].chars().next().map_or(1, char::len_utf8);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_rules_parse() {
        let rules = rules();
        assert!(!rules.kind.front.keywords.is_empty());
        assert!(!rules.kind.back.keywords.is_empty());
        assert!(rules.origin.agents.iter().any(|a| a.id == "claude-code"));
    }

    #[test]
    fn keywords_match_whole_words_only() {
        assert!(contains_keyword("node_modules/.bin/nest start", "nest"));
        assert!(contains_keyword("NEXT DEV --port 3000", "next dev"));
        assert!(contains_keyword("python -m http.server", "http.server"));
        assert!(!contains_keyword("node honest.js", "nest"));
        assert!(!contains_keyword("nestjs", "nest"));
        assert!(contains_keyword("nestjs nest", "nest"));
    }

    #[test]
    fn keywords_handle_non_ascii_text() {
        assert!(contains_keyword("/Users/joão/código/vite", "vite"));
        assert!(!contains_keyword("évite", "vite"));
    }

    #[test]
    fn names_ignore_case_and_exe() {
        let names = vec!["WindowsTerminal".to_string()];
        assert!(name_matches("windowsterminal.EXE", &names));
        assert!(!name_matches("terminal", &names));
    }
}
