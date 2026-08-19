//! Persistent project registry for Code Intelligence indexes.
//!
//! Upstream `codebase-memory-mcp` derives project keys from canonical repo
//! paths (slashes → dashes) but never records *which* repo produced a key or
//! whether Bugrail considers the index live. The registry at
//! `<root>/state/projects.json` records that binding so:
//!
//! - queries can resolve the connected working dir to exactly one project
//!   (exact canonical match, else nearest indexed ancestor base repo);
//! - worktree indexes can be enumerated and dropped when their WorkTask
//!   worktree is cleaned up;
//! - the UI can list per-project state without asking the daemon.
//!
//! The file is rewritten atomically (temp file + rename) and read under a
//! mutex; records are the only mutable state shared across adapter sessions.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::code_intelligence::CodeIntelError;

/// One indexed repository (base repo or WorkTask worktree).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ProjectRecord {
    /// Upstream project key (canonical path with `/` replaced by `-`).
    pub project: String,
    /// Canonical absolute path of the indexed repository / worktree.
    pub repo_path: String,
    /// `true` when the record belongs to a WorkTask worktree whose index
    /// should be dropped together with the worktree.
    pub worktree: bool,
    /// Owning WorkTask id for worktree records.
    pub task_id: Option<i32>,
    /// Whether Bugrail currently treats this index as enabled. Disabled
    /// records keep their data but are skipped by query binding.
    pub enabled: bool,
    /// RFC 3339 timestamp of the first successful index.
    pub indexed_at: String,
    /// RFC 3339 timestamp of the most recent successful sync, if any.
    pub last_synced_at: Option<String>,
    /// Last observed index revision (upstream index revision / commit), if
    /// reported by the adapter.
    pub revision: Option<String>,
}

impl Default for ProjectRecord {
    fn default() -> Self {
        Self {
            project: String::new(),
            repo_path: String::new(),
            worktree: false,
            task_id: None,
            enabled: true,
            indexed_at: String::new(),
            last_synced_at: None,
            revision: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RegistryFile {
    /// Keyed by canonical repo path so a path resolves to one record.
    projects: BTreeMap<String, ProjectRecord>,
}

/// Process-wide registry handle bound to one cache root. The runtime creates
/// exactly one (at init) and hands clones to sessions/commands.
#[derive(Debug, Clone)]
pub struct Registry {
    file_path: PathBuf,
    state: std::sync::Arc<Mutex<RegistryFile>>,
}

impl Registry {
    /// Load (or initialize empty) the registry at `<root>/state/projects.json`.
    pub fn load(root: &Path) -> Result<Self, CodeIntelError> {
        let state_dir = root.join("state");
        fs::create_dir_all(&state_dir).map_err(CodeIntelError::io)?;
        let file_path = state_dir.join("projects.json");
        let file = match fs::read_to_string(&file_path) {
            Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
            Err(err) if err.kind() == io::ErrorKind::NotFound => RegistryFile::default(),
            Err(err) => return Err(CodeIntelError::io(err)),
        };
        Ok(Self {
            file_path,
            state: std::sync::Arc::new(Mutex::new(file)),
        })
    }

    fn persist(&self, file: &RegistryFile) -> Result<(), CodeIntelError> {
        let raw = serde_json::to_string_pretty(file)
            .map_err(|err| CodeIntelError::Internal(format!("registry serialize: {err}")))?;
        let tmp = self.file_path.with_extension("json.tmp");
        fs::write(&tmp, raw).map_err(CodeIntelError::io)?;
        fs::rename(&tmp, &self.file_path).map_err(CodeIntelError::io)?;
        Ok(())
    }

    /// Insert or replace the record for a canonical repo path, persisting.
    pub fn upsert(&self, record: ProjectRecord) -> Result<(), CodeIntelError> {
        let mut file = self.lock();
        file.projects.insert(record.repo_path.clone(), record);
        self.persist(&file)
    }

    /// Remove the record for a canonical repo path, persisting. Returns the
    /// removed record if present.
    pub fn remove(&self, repo_path: &str) -> Result<Option<ProjectRecord>, CodeIntelError> {
        let mut file = self.lock();
        let removed = file.projects.remove(repo_path);
        if removed.is_some() {
            self.persist(&file)?;
        }
        Ok(removed)
    }

    /// Flip `enabled` for a record, persisting. Returns `false` when the
    /// path is unknown.
    pub fn set_enabled(&self, repo_path: &str, enabled: bool) -> Result<bool, CodeIntelError> {
        let mut file = self.lock();
        let Some(record) = file.projects.get_mut(repo_path) else {
            return Ok(false);
        };
        record.enabled = enabled;
        self.persist(&file)?;
        Ok(true)
    }

    pub fn get(&self, repo_path: &str) -> Option<ProjectRecord> {
        self.lock().projects.get(repo_path).cloned()
    }

    /// All records, sorted by canonical path for stable UI ordering.
    pub fn all(&self) -> Vec<ProjectRecord> {
        self.lock().projects.values().cloned().collect()
    }

    /// Enabled records only.
    pub fn enabled(&self) -> Vec<ProjectRecord> {
        self.all().into_iter().filter(|r| r.enabled).collect()
    }

    /// Resolve a connected working dir to a project record:
    /// 1. exact canonical match (the dir itself is indexed), else
    /// 2. the longest indexed ancestor path (a worktree run resolves to its
    ///    base repo index).
    ///
    /// Disabled records are ignored, so disabling a base repo also stops
    /// its indexes serving worktree queries.
    pub fn resolve(&self, canonical_dir: &str) -> Option<ProjectRecord> {
        let file = self.lock();
        if let Some(record) = file.projects.get(canonical_dir) {
            if record.enabled {
                return Some(record.clone());
            }
        }
        let mut best: Option<&ProjectRecord> = None;
        for (path, record) in &file.projects {
            if !record.enabled {
                continue;
            }
            // Ancestor match on a path-component boundary.
            let is_ancestor = canonical_dir
                .strip_prefix(path.as_str())
                .is_some_and(|rest| rest.starts_with('/') || rest.is_empty());
            if is_ancestor {
                let is_longer = match best {
                    None => true,
                    Some(current) => path.len() > current.repo_path.len(),
                };
                if is_longer {
                    best = Some(record);
                }
            }
        }
        best.cloned()
    }

    /// Worktree records owned by a task — used when a WorkTask worktree is
    /// cleaned up and its temporary index must be dropped.
    pub fn worktree_records_for_task(&self, task_id: i32) -> Vec<ProjectRecord> {
        self.lock()
            .projects
            .values()
            .filter(|r| r.worktree && r.task_id == Some(task_id))
            .cloned()
            .collect()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, RegistryFile> {
        match self.state.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(path: &str, enabled: bool) -> ProjectRecord {
        ProjectRecord {
            project: path.trim_start_matches('/').replace('/', "-"),
            repo_path: path.to_string(),
            enabled,
            indexed_at: "2026-08-18T00:00:00Z".to_string(),
            ..Default::default()
        }
    }

    fn registry_in(dir: &Path) -> Registry {
        Registry::load(dir).expect("registry loads")
    }

    #[test]
    fn upsert_and_get_round_trip() {
        let tmp = tempfile::tempdir().unwrap();
        let registry = registry_in(tmp.path());
        registry.upsert(record("/repo/a", true)).unwrap();

        let got = registry.get("/repo/a").expect("record exists");
        assert_eq!(got.project, "repo-a");
        assert!(got.enabled);

        // File is valid JSON on disk.
        let raw = fs::read_to_string(tmp.path().join("state/projects.json")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert!(parsed["projects"]["/repo/a"].is_object());
    }

    #[test]
    fn persists_across_reopen() {
        let tmp = tempfile::tempdir().unwrap();
        registry_in(tmp.path())
            .upsert(record("/repo/a", true))
            .unwrap();
        let reopened = registry_in(tmp.path());
        assert!(reopened.get("/repo/a").is_some());
    }

    #[test]
    fn resolve_prefers_exact_then_longest_ancestor() {
        let tmp = tempfile::tempdir().unwrap();
        let registry = registry_in(tmp.path());
        registry.upsert(record("/repo", true)).unwrap();
        registry.upsert(record("/repo/nested", true)).unwrap();

        // Exact match wins over ancestor.
        assert_eq!(
            registry.resolve("/repo/nested").unwrap().repo_path,
            "/repo/nested"
        );
        // Subdirectory resolves to the longest indexed ancestor.
        assert_eq!(registry.resolve("/repo/nested/src/lib").unwrap().repo_path, "/repo/nested");
        assert_eq!(registry.resolve("/repo/src/lib").unwrap().repo_path, "/repo");
        // Worktrees are SIBLINGS of their base repo (`{base}-task-{id}`),
        // never children — a worktree dir only matches its own record
        // (exact) or falls back to the nearest true ancestor.
        assert_eq!(
            registry.resolve("/repo/nested-task-5").unwrap().repo_path,
            "/repo"
        );
        // Path-component boundary: `/rep` is not an ancestor of `/repo`.
        assert!(registry.resolve("/rep/other").is_none());
        assert!(registry.resolve("/elsewhere").is_none());
    }

    #[test]
    fn disabled_records_are_skipped_by_resolve() {
        let tmp = tempfile::tempdir().unwrap();
        let registry = registry_in(tmp.path());
        registry.upsert(record("/repo", false)).unwrap();
        assert!(registry.resolve("/repo").is_none());
        assert!(registry.resolve("/repo/sub").is_none());

        registry.set_enabled("/repo", true).unwrap();
        assert!(registry.resolve("/repo").is_some());
    }

    #[test]
    fn worktree_records_filter_by_task() {
        let tmp = tempfile::tempdir().unwrap();
        let registry = registry_in(tmp.path());
        let mut wt = record("/repo-task-5", true);
        wt.worktree = true;
        wt.task_id = Some(5);
        registry.upsert(wt).unwrap();
        registry.upsert(record("/repo", true)).unwrap();

        let for_task = registry.worktree_records_for_task(5);
        assert_eq!(for_task.len(), 1);
        assert_eq!(for_task[0].repo_path, "/repo-task-5");
        assert!(registry.worktree_records_for_task(6).is_empty());
    }

    #[test]
    fn remove_drops_record_and_persists() {
        let tmp = tempfile::tempdir().unwrap();
        let registry = registry_in(tmp.path());
        registry.upsert(record("/repo", true)).unwrap();
        assert!(registry.remove("/repo").unwrap().is_some());
        assert!(registry.get("/repo").is_none());

        let reopened = registry_in(tmp.path());
        assert!(reopened.get("/repo").is_none());
        assert!(registry.remove("/repo").unwrap().is_none());
    }
}
