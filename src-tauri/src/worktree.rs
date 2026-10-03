//! Which git worktree (and branch) a folder belongs to.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::command;
use crate::model::Worktree;

/// A process' working directory almost never changes, so each folder is
/// asked to git once. Entries still expire so a `git checkout` in that
/// worktree shows the new branch after a while.
const TTL: Duration = Duration::from_secs(60);

#[derive(Default)]
pub struct WorktreeCache {
    entries: Mutex<HashMap<PathBuf, (Instant, Option<Worktree>)>>,
}

impl WorktreeCache {
    /// Resolves every folder, asking git in parallel for the ones not in the
    /// cache. Returns the worktree of each folder (`None` outside a repo).
    pub fn resolve_all(&self, folders: &[PathBuf]) -> HashMap<PathBuf, Option<Worktree>> {
        let mut result = HashMap::new();
        let mut missing = Vec::new();
        {
            let entries = self.entries.lock().unwrap();
            for folder in folders {
                match entries.get(folder) {
                    Some((at, worktree)) if at.elapsed() < TTL => {
                        result.insert(folder.clone(), worktree.clone());
                    }
                    _ if !missing.contains(folder) => missing.push(folder.clone()),
                    _ => {}
                }
            }
        }

        let fresh: Vec<(PathBuf, Option<Worktree>)> = std::thread::scope(|scope| {
            let handles: Vec<_> = missing
                .iter()
                .map(|folder| scope.spawn(move || (folder.clone(), lookup(folder))))
                .collect();
            handles.into_iter().filter_map(|h| h.join().ok()).collect()
        });

        let mut entries = self.entries.lock().unwrap();
        let now = Instant::now();
        for (folder, worktree) in fresh {
            entries.insert(folder.clone(), (now, worktree.clone()));
            result.insert(folder, worktree);
        }
        result
    }
}

/// Asks git once. If git isn't installed or the folder isn't in a
/// repository, the answer is simply `None`: no error is shown.
fn lookup(folder: &Path) -> Option<Worktree> {
    let folder = folder.to_str()?;
    let output = command::output(
        "git",
        &[
            "-C",
            folder,
            "rev-parse",
            "--show-toplevel",
            "--abbrev-ref",
            "HEAD",
        ],
    )?;
    parse_rev_parse(&output)
}

/// Parses `git rev-parse --show-toplevel --abbrev-ref HEAD`: the worktree
/// root on the first line, the branch on the second ("HEAD" when detached).
pub fn parse_rev_parse(output: &str) -> Option<Worktree> {
    let mut lines = output.lines().map(str::trim);
    let root = lines.next().filter(|l| !l.is_empty())?;
    let branch = lines
        .next()
        .filter(|b| !b.is_empty() && *b != "HEAD")
        .map(str::to_string);
    let name = Path::new(root)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| root.to_string());
    Some(Worktree {
        root: root.to_string(),
        branch,
        name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_root_and_branch() {
        let worktree = parse_rev_parse("/Users/me/code/shop-checkout\nfeat/checkout\n").unwrap();
        assert_eq!(worktree.root, "/Users/me/code/shop-checkout");
        assert_eq!(worktree.name, "shop-checkout");
        assert_eq!(worktree.branch.as_deref(), Some("feat/checkout"));
    }

    #[test]
    fn detached_head_has_no_branch() {
        let worktree = parse_rev_parse("/work/app\nHEAD\n").unwrap();
        assert_eq!(worktree.branch, None);
    }

    #[test]
    fn windows_paths_from_git() {
        let worktree = parse_rev_parse("C:/Users/me/code/api\r\nmain\r\n").unwrap();
        assert_eq!(worktree.name, "api");
        assert_eq!(worktree.branch.as_deref(), Some("main"));
    }

    #[test]
    fn empty_output_is_no_worktree() {
        assert_eq!(parse_rev_parse(""), None);
    }

    fn git(dir: &Path, args: &[&str]) -> bool {
        command::quiet("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[test]
    fn resolves_real_repositories_and_worktrees() {
        let base = std::env::temp_dir().join(format!("farol-worktree-test-{}", std::process::id()));
        let main = base.join("main");
        let feature = base.join("feature");
        std::fs::create_dir_all(main.join("src")).unwrap();

        let ready = git(&main, &["init", "-q", "-b", "main"])
            && git(
                &main,
                &[
                    "-c",
                    "user.name=t",
                    "-c",
                    "user.email=t@t",
                    "commit",
                    "-q",
                    "--allow-empty",
                    "-m",
                    "init",
                ],
            )
            && git(
                &main,
                &[
                    "worktree",
                    "add",
                    "-q",
                    "-b",
                    "feature",
                    feature.to_str().unwrap(),
                ],
            );
        if !ready {
            eprintln!("git is not available; skipping");
            let _ = std::fs::remove_dir_all(&base);
            return;
        }

        let cache = WorktreeCache::default();
        let outside = std::env::temp_dir();
        let result = cache.resolve_all(&[main.join("src"), feature.clone(), outside.clone()]);

        let in_main = result[&main.join("src")].clone().unwrap();
        assert_eq!(in_main.name, "main");
        assert_eq!(in_main.branch.as_deref(), Some("main"));
        let in_feature = result[&feature].clone().unwrap();
        assert_eq!(in_feature.name, "feature");
        assert_eq!(in_feature.branch.as_deref(), Some("feature"));
        assert_eq!(result[&outside], None);

        let _ = std::fs::remove_dir_all(&base);
    }
}
