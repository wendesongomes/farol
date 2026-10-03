//! The user's choices: pinned processes and front/back corrections.
//!
//! They are keyed by worktree, port and executable name instead of PID,
//! because the PID changes every time a server restarts and the choice
//! should survive that.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreExt;

use crate::model::ProcessType;

const STORE_FILE: &str = "prefs.json";
const STORE_KEY: &str = "prefs";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Prefs {
    pub type_overrides: BTreeMap<String, ProcessType>,
    pub pins: BTreeSet<String>,
}

/// `"<worktree root or 'none'>:<port>:<name>"`.
pub fn key(worktree_root: Option<&str>, port: u16, name: &str) -> String {
    format!("{}:{}:{}", worktree_root.unwrap_or("none"), port, name)
}

impl Prefs {
    pub fn set_type(&mut self, key: &str, kind: ProcessType) {
        self.type_overrides.insert(key.to_string(), kind);
    }

    /// Returns whether the key is pinned after toggling.
    pub fn toggle_pin(&mut self, key: &str) -> bool {
        if self.pins.remove(key) {
            false
        } else {
            self.pins.insert(key.to_string());
            true
        }
    }
}

/// Prefs in memory, written to the store file on every change.
#[derive(Clone, Default)]
pub struct PrefsState(pub Arc<Mutex<Prefs>>);

impl PrefsState {
    pub fn load<R: Runtime>(app: &AppHandle<R>) -> Self {
        let prefs = app
            .store(STORE_FILE)
            .ok()
            .and_then(|store| store.get(STORE_KEY))
            .and_then(|value| serde_json::from_value(value).ok())
            .unwrap_or_default();
        Self(Arc::new(Mutex::new(prefs)))
    }

    pub fn snapshot(&self) -> Prefs {
        self.0.lock().unwrap().clone()
    }

    pub fn update<R: Runtime, T>(
        &self,
        app: &AppHandle<R>,
        change: impl FnOnce(&mut Prefs) -> T,
    ) -> T {
        let mut prefs = self.0.lock().unwrap();
        let result = change(&mut prefs);
        if let Ok(store) = app.store(STORE_FILE) {
            if let Ok(value) = serde_json::to_value(&*prefs) {
                store.set(STORE_KEY, value);
                if let Err(error) = store.save() {
                    eprintln!("farol: could not save preferences: {error}");
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_survives_restarts() {
        // Same server restarted: new PID, same key.
        assert_eq!(key(Some("/code/web"), 3000, "node"), "/code/web:3000:node");
        assert_eq!(key(None, 5432, "postgres"), "none:5432:postgres");
    }

    #[test]
    fn key_separates_worktrees() {
        assert_ne!(
            key(Some("/code/web"), 3000, "node"),
            key(Some("/code/web-2"), 3000, "node")
        );
    }

    #[test]
    fn toggling_a_pin_twice_unpins() {
        let mut prefs = Prefs::default();
        assert!(prefs.toggle_pin("k"));
        assert!(prefs.pins.contains("k"));
        assert!(!prefs.toggle_pin("k"));
        assert!(prefs.pins.is_empty());
    }

    #[test]
    fn round_trips_through_json() {
        let mut prefs = Prefs::default();
        prefs.set_type("none:3000:node", ProcessType::Front);
        prefs.toggle_pin("none:5432:postgres");
        let json = serde_json::to_value(&prefs).unwrap();
        assert_eq!(json["typeOverrides"]["none:3000:node"], "front");
        assert_eq!(serde_json::from_value::<Prefs>(json).unwrap(), prefs);
    }

    #[test]
    fn tolerates_missing_fields() {
        let prefs: Prefs = serde_json::from_str("{}").unwrap();
        assert_eq!(prefs, Prefs::default());
    }
}
