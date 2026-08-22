//! Deterministic Memory identity (BUGRAIL-SPECOS-017 §3).
//!
//! BugRail has no cross-install project binding id, so the project binding is
//! derived from the canonicalized folder path. Every upstream identity is a
//! pure function of that binding plus persisted WorkTask/conversation facts —
//! restarting codeg or retrying a delivery therefore produces byte-identical
//! ids, which is what makes at-least-once capture safe under the
//! `v2.0.0+bugrail.1` upsert contract.
//!
//! Ids are opaque hex digests: they carry no readable folder names, task
//! titles or transcript text to the upstream service.

use sha2::{Digest, Sha256};
use std::path::Path;

fn sha256_hex(input: &str) -> String {
    format!("{:x}", Sha256::digest(input.as_bytes()))
}

/// Project binding: sha256 of the canonicalized project folder path.
///
/// Canonicalization falls back to the verbatim path when the directory is
/// temporarily unresolvable (e.g. unmounted volume) — identity derivation
/// must never fail the caller; a changed binding only means the upstream
/// space starts fresh, it can never mix two projects because `team_id`
/// remains the authoritative isolation boundary.
pub fn project_binding(folder_path: &Path) -> String {
    let canonical =
        std::fs::canonicalize(folder_path).unwrap_or_else(|_| folder_path.to_path_buf());
    sha256_hex(&canonical.to_string_lossy())
}

/// Session id: project binding + task generation. One upstream session per
/// WorkTask run generation.
pub fn session_id(binding: &str, task_id: i32, run_seq: i32) -> String {
    sha256_hex(&format!("{binding}/session/{task_id}/{run_seq}"))
}

/// Stable upstream `task_id` carried by capture. Recall does NOT filter by
/// it — recall spans all WorkTasks inside one project `team_id`.
pub fn upstream_task_id(binding: &str, task_id: i32) -> String {
    sha256_hex(&format!("{binding}/task/{task_id}"))
}

/// Message id: project binding + persisted conversation id + parser turn id.
/// Replaying the same settled run yields the same ids, so the patched
/// Gateway upserts instead of duplicating L0 rows.
pub fn message_id(binding: &str, conversation_id: i32, turn_id: &str) -> String {
    sha256_hex(&format!("{binding}/message/{conversation_id}/{turn_id}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identities_are_deterministic_and_distinct() {
        let binding = project_binding(Path::new("/tmp/project-a"));
        assert_eq!(binding, project_binding(Path::new("/tmp/project-a")));
        assert_ne!(binding, project_binding(Path::new("/tmp/project-b")));

        assert_eq!(
            session_id(&binding, 1, 2),
            session_id(&binding, 1, 2),
            "session id must be stable across restarts"
        );
        assert_ne!(session_id(&binding, 1, 2), session_id(&binding, 1, 3));
        assert_ne!(session_id(&binding, 1, 2), session_id(&binding, 2, 2));

        assert_ne!(upstream_task_id(&binding, 1), upstream_task_id(&binding, 2));

        assert_eq!(
            message_id(&binding, 7, "turn-3"),
            message_id(&binding, 7, "turn-3")
        );
        assert_ne!(
            message_id(&binding, 7, "turn-3"),
            message_id(&binding, 8, "turn-3")
        );
    }

    #[test]
    fn identities_are_opaque_hex() {
        let binding = project_binding(Path::new("/tmp/project-a"));
        assert!(binding.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(binding.len(), 64);
        assert!(!binding.contains("project-a"));
    }
}
