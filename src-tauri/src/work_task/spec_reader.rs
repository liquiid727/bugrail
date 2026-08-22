//! Internal repository-local Feature Spec reader (BUGRAIL-SPECOS-001 issue-002).
//!
//! Reads one Git-tracked Feature Spec from inside a task's live project folder
//! and returns a validated immutable reference: identity (`id`/`version` from
//! YAML front matter), the raw-file SHA-256, and the acceptance-criteria table
//! from the `## N. Acceptance Criteria` section. This is the only module that
//! knows how to turn a spec path into the wire `WorkTaskContractPreview` facts;
//! the command core never parses front matter itself.
//!
//! Security (Feature Spec §6): paths are repository-relative, canonicalized,
//! and confined to the project root. A symlink that resolves outside the root
//! is rejected, as is an absolute or parent-traversing path. Files larger than
//! [`MAX_SPEC_BYTES`] are rejected before parsing.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::app_error::AppCommandError;
use crate::models::AcceptanceCriterionSnapshot;
use crate::work_task::gate_decision::MAX_SPEC_BYTES;

/// Wire i18n keys stamped on spec-contract errors (Feature Spec §4.3). The
/// frontend consumes these under `Tasks.specos.*` once issue-004 lands; until
/// then a missing locale key falls back to the English `message`.
pub const SPEC_CONTRACT_INVALID_I18N: &str = "workTask.specContract.invalid";
pub const SPEC_CONTRACT_STALE_I18N: &str = "workTask.specContract.stale";

/// One validated reference to a repository-local Feature Spec.
#[derive(Debug, Clone)]
pub struct SpecReference {
    /// Front-matter `id` (e.g. `BUGRAIL-SPECOS-001`).
    pub id: String,
    /// Front-matter `version` (e.g. `0.3`).
    pub version: String,
    /// Repository-relative path as submitted by the client.
    pub path: String,
    /// Lowercase-hex SHA-256 of the raw file bytes.
    pub sha256: String,
    /// Acceptance criteria parsed from the `Acceptance Criteria` table.
    pub acceptance_criteria: Vec<AcceptanceCriterionSnapshot>,
}

/// `InvalidInput` + `workTask.specContract.invalid` — the generic spec
/// validation error for invalid path, metadata, AC, or gate policy.
fn invalid(message: impl Into<String>) -> AppCommandError {
    AppCommandError::invalid_input(message).with_i18n(SPEC_CONTRACT_INVALID_I18N, BTreeMap::new())
}

/// Resolve `rel_path` inside `project_root` and return the canonical absolute
/// path. Rejects absolute paths, lexical parent traversal, and any resolution
/// (including through a symlink) that lands outside `project_root`.
pub fn resolve_in_project(project_root: &Path, rel_path: &str) -> Result<PathBuf, AppCommandError> {
    let rel = Path::new(rel_path);
    if rel.is_absolute() {
        return Err(invalid("spec path must be repository-relative"));
    }
    for c in rel.components() {
        match c {
            Component::ParentDir | Component::Prefix(_) | Component::RootDir => {
                return Err(invalid("spec path escapes project root"));
            }
            _ => {}
        }
    }
    let canonical_root = std::fs::canonicalize(project_root)
        .map_err(|e| invalid("project root is not readable").with_detail(e.to_string()))?;
    let candidate = project_root.join(rel_path);
    let canonical_target = std::fs::canonicalize(&candidate)
        .map_err(|e| invalid("spec file is missing or not readable").with_detail(e.to_string()))?;
    if !canonical_target.starts_with(&canonical_root) {
        return Err(invalid("spec path escapes project root"));
    }
    Ok(canonical_target)
}

/// Read a Feature Spec from the task's project root: resolve + size-check +
/// parse. The full read path used by preview and bind.
pub fn read_spec_reference(
    project_root: &Path,
    rel_path: &str,
) -> Result<SpecReference, AppCommandError> {
    let resolved = resolve_in_project(project_root, rel_path)?;
    let bytes = std::fs::read(&resolved)
        .map_err(|e| invalid("spec file is not readable").with_detail(e.to_string()))?;
    if bytes.len() > MAX_SPEC_BYTES {
        return Err(invalid("spec file exceeds 1 MiB")
            .with_detail(format!("size={}, limit={MAX_SPEC_BYTES}", bytes.len())));
    }
    parse_spec_body(rel_path, &bytes)
}

/// Parse a Feature Spec body (front matter + acceptance-criteria table). Pure:
/// no filesystem access, deterministic on the byte content. The SHA-256 is of
/// the exact raw bytes, matching the `sourceSpecHash` convention.
pub fn parse_spec_body(rel_path: &str, bytes: &[u8]) -> Result<SpecReference, AppCommandError> {
    let sha256 = sha256_hex(bytes);
    let text = String::from_utf8_lossy(bytes);
    let (id, version) = parse_frontmatter(&text)?;
    let acceptance_criteria = parse_acceptance_criteria(&text);
    Ok(SpecReference {
        id,
        version,
        path: rel_path.trim().to_string(),
        sha256,
        acceptance_criteria,
    })
}

/// Whether the currently bound spec file is stale: re-reading the file at
/// `source_spec_path` must still hash to `bound_hash`. A file that can no
/// longer be read (moved, deleted, now escapes) is treated as stale — merge
/// gating must never proceed on an unverifiable reference.
pub fn spec_stale(
    project_root: &Path,
    source_spec_path: &str,
    bound_hash: &str,
) -> Result<bool, AppCommandError> {
    match read_spec_reference(project_root, source_spec_path) {
        Ok(spec) => Ok(spec.sha256 != bound_hash),
        Err(_) => Ok(true),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// YAML front-matter block between the leading `---` and the closing `---`.
fn extract_frontmatter(text: &str) -> Option<&str> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let text = text.strip_prefix('\n').unwrap_or(text);
    let rest = text.strip_prefix("---")?;
    let rest = rest
        .strip_prefix("\r\n")
        .or_else(|| rest.strip_prefix('\n'))?;
    let end = rest.find("\n---")?;
    Some(&rest[..end])
}

#[derive(serde::Deserialize)]
struct Frontmatter {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    version: Option<serde_yaml::Value>,
}

fn scalar_to_string(v: &serde_yaml::Value) -> Option<String> {
    match v {
        serde_yaml::Value::String(s) => Some(s.clone()),
        serde_yaml::Value::Number(n) => Some(n.to_string()),
        serde_yaml::Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// Extract `id` and `version` from the YAML front matter. Absent, empty, or
/// non-scalar values are rejected (`T06`).
fn parse_frontmatter(text: &str) -> Result<(String, String), AppCommandError> {
    let block = extract_frontmatter(text).ok_or_else(|| invalid("spec has no front matter"))?;
    let fm: Frontmatter = serde_yaml::from_str(block)
        .map_err(|e| invalid("spec front matter is malformed").with_detail(e.to_string()))?;
    let id = fm
        .id
        .ok_or_else(|| invalid("spec front matter missing id"))?;
    let id = id.trim().to_string();
    if id.is_empty() {
        return Err(invalid("spec front matter id is empty"));
    }
    let version = fm
        .version
        .as_ref()
        .ok_or_else(|| invalid("spec front matter missing version"))?;
    let version =
        scalar_to_string(version).ok_or_else(|| invalid("spec version must be a scalar"))?;
    let version = version.trim().to_string();
    if version.is_empty() {
        return Err(invalid("spec front matter version is empty"));
    }
    Ok((id, version))
}

fn strip_backticks(s: &str) -> String {
    s.trim().trim_matches('`').trim().to_string()
}

/// Parse the acceptance-criteria table from the `## N. Acceptance Criteria`
/// section. Deterministic on the controlled Feature Spec format: a markdown
/// table whose first column is the full AC id and second column the criterion.
fn parse_acceptance_criteria(text: &str) -> Vec<AcceptanceCriterionSnapshot> {
    let lines: Vec<&str> = text.lines().collect();
    let mut start = None;
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        let lower = t.to_ascii_lowercase();
        if lower.starts_with("##") && lower.contains("acceptance criteria") {
            start = Some(i + 1);
            break;
        }
    }
    let Some(start) = start else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for line in &lines[start..] {
        let t = line.trim();
        if t.starts_with("##") {
            break;
        }
        if !t.starts_with('|') {
            continue;
        }
        let trimmed = t.trim_matches(|c| c == '|' || c == ' ');
        let cells: Vec<&str> = trimmed.split('|').map(|c| c.trim()).collect();
        if cells.len() < 2 {
            continue;
        }
        let id = strip_backticks(cells[0]);
        let criterion = cells[1].trim();
        // Header row (`| ID | Criterion | ... |`) and the `|---|---|` separator.
        if id.eq_ignore_ascii_case("ID") || id.contains("---") {
            continue;
        }
        if id.is_empty() || criterion.is_empty() {
            continue;
        }
        let title = id.rsplit('.').next().unwrap_or(&id).to_string();
        out.push(AcceptanceCriterionSnapshot {
            id,
            title,
            text: criterion.to_string(),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_SPEC: &str = "---\n\
id: BUGRAIL-SPECOS-001\n\
version: \"0.3\"\n\
title: \"Spec-Linked WorkTask Quality\"\n\
---\n\
# BUGRAIL-SPECOS-001\n\
## 1. Summary\n\
Nothing here.\n\
## 8. Acceptance Criteria\n\
| ID | Criterion | Requirements |\n\
|---|---|---|\n\
| `BUGRAIL-SPECOS-001.AC01` | Preview then bind stores exact metadata. | `R01-R03` |\n\
| `BUGRAIL-SPECOS-001.AC02` | Invalid input is rejected. | `R02`, `R03` |\n\
";

    #[test]
    fn parses_frontmatter_and_ac_table() {
        let spec = parse_spec_body(".features/x/spec.md", VALID_SPEC.as_bytes()).unwrap();
        assert_eq!(spec.id, "BUGRAIL-SPECOS-001");
        assert_eq!(spec.version, "0.3");
        assert_eq!(spec.path, ".features/x/spec.md");
        assert_eq!(spec.acceptance_criteria.len(), 2);
        assert_eq!(spec.acceptance_criteria[0].id, "BUGRAIL-SPECOS-001.AC01");
        assert_eq!(spec.acceptance_criteria[0].title, "AC01");
        assert!(spec.acceptance_criteria[0].text.contains("exact metadata"));
        assert_eq!(spec.acceptance_criteria[1].id, "BUGRAIL-SPECOS-001.AC02");
    }

    #[test]
    fn sha256_is_hex_of_raw_bytes() {
        // SHA-256("hello") is a well-known constant.
        assert_eq!(
            sha256_hex(b"hello"),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn missing_or_malformed_frontmatter_rejected() {
        let err = parse_spec_body("s.md", b"# no front matter").unwrap_err();
        assert!(matches!(
            err.code,
            crate::app_error::AppErrorCode::InvalidInput
        ));
        assert!(err.message.contains("front matter"));

        let missing_id = "---\nversion: \"1.0\"\n---\n# x\n".replace("id", "nope");
        let err = parse_spec_body("s.md", missing_id.as_bytes()).unwrap_err();
        assert!(err.message.contains("id"));

        let list_version = "---\nid: X\nversion: [1, 2]\n---\n".to_string();
        let err = parse_spec_body("s.md", list_version.as_bytes()).unwrap_err();
        assert!(err.message.contains("scalar") || err.message.contains("malformed"));
    }

    #[test]
    fn numeric_version_is_accepted_as_string() {
        let spec = "---\nid: X\nversion: 1.5\n---\n# x\n";
        let spec = parse_spec_body("s.md", spec.as_bytes()).unwrap();
        assert_eq!(spec.version, "1.5");
    }

    #[test]
    fn empty_ac_table_yields_empty_snapshot() {
        let spec = "---\nid: X\nversion: \"1\"\n---\n# x\n## 1. Stuff\n";
        let spec = parse_spec_body("s.md", spec.as_bytes()).unwrap();
        assert!(spec.acceptance_criteria.is_empty());
    }
}
