//! Project-owned Agent/Team/Context configuration.
//!
//! These documents are intentionally separate from `.specos/workflows/`,
//! which describes the repository's delivery-document workflow rather than an
//! executable Agent Team. Runtime snapshots are persisted by WorkTask.

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::db::entities::folder;
use crate::db::error::DbError;
use crate::models::{
    AgentCatalog, ContextConfig, ContextLoadout, ContextSourceConfig, TeamCatalog,
};

const CONFIG_DIR: &str = ".codeg";

pub async fn project_root(conn: &DatabaseConnection, folder_id: i32) -> Result<PathBuf, DbError> {
    let row = folder::Entity::find_by_id(folder_id)
        .filter(folder::Column::DeletedAt.is_null())
        .one(conn)
        .await?
        .ok_or_else(|| DbError::NotFound(format!("folder {folder_id}")))?;
    if row.parent_id.is_some() {
        return Err(DbError::Validation(
            "SpecOS configuration requires a project folder, not a worktree".into(),
        ));
    }
    Ok(PathBuf::from(row.path))
}

fn config_path(root: &Path, file: &str) -> Result<PathBuf, DbError> {
    let dir = root.join(CONFIG_DIR);
    if let Ok(meta) = std::fs::symlink_metadata(&dir) {
        if meta.file_type().is_symlink() {
            return Err(DbError::Validation(".codeg must not be a symlink".into()));
        }
    }
    let path = dir.join(file);
    if let Ok(meta) = std::fs::symlink_metadata(&path) {
        if meta.file_type().is_symlink() {
            return Err(DbError::Validation(format!(
                "{CONFIG_DIR}/{file} must not be a symlink"
            )));
        }
    }
    Ok(path)
}

fn load_yaml<T: DeserializeOwned + Default>(root: &Path, file: &str) -> Result<T, DbError> {
    let path = config_path(root, file)?;
    match std::fs::read_to_string(&path) {
        Ok(raw) => serde_yaml::from_str(&raw)
            .map_err(|e| DbError::Validation(format!("invalid {CONFIG_DIR}/{file}: {e}"))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(T::default()),
        Err(e) => Err(DbError::Io(e)),
    }
}

fn save_yaml<T: Serialize>(root: &Path, file: &str, value: &T) -> Result<(), DbError> {
    let path = config_path(root, file)?;
    let parent = path.parent().expect("config path has parent");
    std::fs::create_dir_all(parent)?;
    let raw = serde_yaml::to_string(value)
        .map_err(|e| DbError::Validation(format!("cannot serialize {file}: {e}")))?;
    let temp = parent.join(format!(".{file}.{}.tmp", uuid::Uuid::new_v4().simple()));
    std::fs::write(&temp, raw)?;
    if let Err(e) = std::fs::rename(&temp, &path) {
        let _ = std::fs::remove_file(&temp);
        return Err(DbError::Io(e));
    }
    Ok(())
}

pub fn load_agents(root: &Path) -> Result<AgentCatalog, DbError> {
    let mut value: AgentCatalog = load_yaml(root, "agents.yaml")?;
    if value.version <= 0
        && value.agent_profiles.is_empty()
        && value.model_profiles.is_empty()
        && value.default_agent_profile_id.is_none()
    {
        value.version = 1;
    }
    value.validation_errors = validate_agents(&value);
    Ok(value)
}

pub fn save_agents(root: &Path, mut value: AgentCatalog) -> Result<AgentCatalog, DbError> {
    value.validation_errors = validate_agents(&value);
    if !value.validation_errors.is_empty() {
        return Err(DbError::Validation(value.validation_errors.join("; ")));
    }
    save_yaml(root, "agents.yaml", &value)?;
    Ok(value)
}

pub fn load_teams(root: &Path) -> Result<TeamCatalog, DbError> {
    let mut value: TeamCatalog = load_yaml(root, "teams.yaml")?;
    value.validation_errors = validate_teams(&value);
    Ok(value)
}

pub fn save_teams(root: &Path, mut value: TeamCatalog) -> Result<TeamCatalog, DbError> {
    value.validation_errors = validate_teams(&value);
    if !value.validation_errors.is_empty() {
        return Err(DbError::Validation(value.validation_errors.join("; ")));
    }
    save_yaml(root, "teams.yaml", &value)?;
    Ok(value)
}

pub fn default_context_config() -> ContextConfig {
    ContextConfig {
        version: 1,
        default_loadout_id: "default".into(),
        providers: Vec::new(),
        loadouts: vec![ContextLoadout {
            id: "default".into(),
            name: "Project essentials".into(),
            sources: vec![
                ContextSourceConfig {
                    path: "AGENTS.md".into(),
                    required: false,
                    kind: "rules".into(),
                },
                ContextSourceConfig {
                    path: "README.md".into(),
                    required: false,
                    kind: "project".into(),
                },
                ContextSourceConfig {
                    path: ".rules/project.md".into(),
                    required: false,
                    kind: "rules".into(),
                },
            ],
            provider_ids: Vec::new(),
            max_items: 64,
            max_bytes: 512 * 1024,
            max_tokens: 32_000,
        }],
        validation_errors: Vec::new(),
    }
}

pub fn load_context(root: &Path) -> Result<ContextConfig, DbError> {
    let path = config_path(root, "context.yaml")?;
    let mut value = if path.exists() {
        load_yaml(root, "context.yaml")?
    } else {
        default_context_config()
    };
    value.validation_errors = validate_context(&value);
    Ok(value)
}

pub fn save_context(root: &Path, mut value: ContextConfig) -> Result<ContextConfig, DbError> {
    value.validation_errors = validate_context(&value);
    if !value.validation_errors.is_empty() {
        return Err(DbError::Validation(value.validation_errors.join("; ")));
    }
    save_yaml(root, "context.yaml", &value)?;
    Ok(value)
}

pub fn validate_agents(value: &AgentCatalog) -> Vec<String> {
    let mut errors = Vec::new();
    if value.version <= 0 {
        errors.push("agent catalog version must be positive".into());
    }
    let model_ids = unique_ids(
        value.model_profiles.iter().map(|p| p.id.as_str()),
        "model profile",
        &mut errors,
    );
    let agent_ids = unique_ids(
        value.agent_profiles.iter().map(|p| p.id.as_str()),
        "agent profile",
        &mut errors,
    );
    for profile in &value.agent_profiles {
        if profile.runtime_adapter.trim().is_empty() {
            errors.push(format!(
                "agent profile '{}' requires runtimeAdapter",
                profile.id
            ));
        } else if serde_json::from_value::<crate::models::AgentType>(serde_json::Value::String(
            profile.runtime_adapter.clone(),
        ))
        .is_err()
        {
            errors.push(format!(
                "agent profile '{}' has unknown runtimeAdapter '{}'",
                profile.id, profile.runtime_adapter
            ));
        }
        if let Some(id) = &profile.model_profile_id {
            if !model_ids.contains(id) {
                errors.push(format!(
                    "agent profile '{}' references missing model profile '{id}'",
                    profile.id
                ));
            }
        }
        for key in profile.config_values.keys() {
            let lowered = key.to_ascii_lowercase();
            if [
                "secret",
                "password",
                "api_key",
                "apikey",
                "access_token",
                "credential",
            ]
            .iter()
            .any(|needle| lowered.contains(needle))
            {
                errors.push(format!(
                    "agent profile '{}' configValues must not contain secret-like key '{key}'",
                    profile.id
                ));
            }
        }
    }
    for profile in &value.model_profiles {
        for fallback in &profile.fallback_profile_ids {
            if !model_ids.contains(fallback) {
                errors.push(format!(
                    "model profile '{}' references missing fallback '{fallback}'",
                    profile.id
                ));
            }
        }
    }
    if let Some(id) = &value.default_agent_profile_id {
        if !agent_ids.contains(id) {
            errors.push(format!("default agent profile '{id}' does not exist"));
        }
    }
    errors
}

pub fn validate_teams(value: &TeamCatalog) -> Vec<String> {
    let mut errors = Vec::new();
    if value.version <= 0 {
        errors.push("team catalog version must be positive".into());
    }
    let team_ids = unique_ids(
        value.teams.iter().map(|t| t.id.as_str()),
        "team",
        &mut errors,
    );
    unique_ids(
        value.workflows.iter().map(|w| w.id.as_str()),
        "workflow",
        &mut errors,
    );
    for workflow in &value.workflows {
        if workflow.version <= 0 {
            errors.push(format!(
                "workflow '{}' version must be positive",
                workflow.id
            ));
        }
        if !team_ids.contains(&workflow.team_id) {
            errors.push(format!(
                "workflow '{}' references missing team '{}'",
                workflow.id, workflow.team_id
            ));
        }
        if !(1..=32).contains(&workflow.max_concurrent) {
            errors.push(format!(
                "workflow '{}' maxConcurrent must be 1..32",
                workflow.id
            ));
        }
        if workflow.nodes.is_empty() || workflow.nodes.len() > 128 {
            errors.push(format!(
                "workflow '{}' must contain 1..128 nodes",
                workflow.id
            ));
            continue;
        }
        let node_ids = unique_ids(
            workflow.nodes.iter().map(|n| n.id.as_str()),
            "workflow node",
            &mut errors,
        );
        let mut indegree: BTreeMap<&str, usize> =
            workflow.nodes.iter().map(|n| (n.id.as_str(), 0)).collect();
        let mut children: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for node in &workflow.nodes {
            if node.prompt.trim().is_empty() {
                errors.push(format!("workflow node '{}' requires a prompt", node.id));
            }
            for dep in &node.depends_on {
                if dep == &node.id {
                    errors.push(format!(
                        "workflow node '{}' cannot depend on itself",
                        node.id
                    ));
                } else if !node_ids.contains(dep) {
                    errors.push(format!(
                        "workflow node '{}' references missing dependency '{dep}'",
                        node.id
                    ));
                } else {
                    *indegree.get_mut(node.id.as_str()).expect("node exists") += 1;
                    children
                        .entry(dep.as_str())
                        .or_default()
                        .push(node.id.as_str());
                }
            }
        }
        let mut ready: Vec<&str> = indegree
            .iter()
            .filter_map(|(id, d)| (*d == 0).then_some(*id))
            .collect();
        let mut visited = 0;
        while let Some(id) = ready.pop() {
            visited += 1;
            for child in children.get(id).into_iter().flatten() {
                let entry = indegree.get_mut(child).expect("child exists");
                *entry -= 1;
                if *entry == 0 {
                    ready.push(child);
                }
            }
        }
        if visited != workflow.nodes.len() {
            errors.push(format!(
                "workflow '{}' contains a dependency cycle",
                workflow.id
            ));
        }
    }
    errors
}

pub fn validate_context(value: &ContextConfig) -> Vec<String> {
    let mut errors = Vec::new();
    if value.version <= 0 {
        errors.push("context config version must be positive".into());
    }
    let provider_ids = unique_ids(
        value.providers.iter().map(|p| p.id.as_str()),
        "context provider",
        &mut errors,
    );
    let loadout_ids = unique_ids(
        value.loadouts.iter().map(|p| p.id.as_str()),
        "context loadout",
        &mut errors,
    );
    if !loadout_ids.contains(&value.default_loadout_id) {
        errors.push(format!(
            "default context loadout '{}' does not exist",
            value.default_loadout_id
        ));
    }
    for provider in &value.providers {
        if provider.required && !provider.enabled {
            errors.push(format!(
                "required context provider '{}' must be enabled",
                provider.id
            ));
        }
        if let Some(endpoint) = &provider.endpoint {
            if !(endpoint.starts_with("http://") || endpoint.starts_with("https://")) {
                errors.push(format!(
                    "context provider '{}' endpoint must use http or https",
                    provider.id
                ));
            }
        }
        if let Some(secret_env) = &provider.secret_env {
            if secret_env.is_empty()
                || !secret_env
                    .chars()
                    .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
            {
                errors.push(format!("context provider '{}' secretEnv must be an uppercase environment variable name", provider.id));
            }
        }
    }
    for loadout in &value.loadouts {
        if loadout.max_items == 0 || loadout.max_items > 64 {
            errors.push(format!("loadout '{}' maxItems must be 1..64", loadout.id));
        }
        if loadout.max_bytes == 0 || loadout.max_bytes > 512 * 1024 {
            errors.push(format!(
                "loadout '{}' maxBytes must be 1..524288",
                loadout.id
            ));
        }
        if loadout.max_tokens == 0 || loadout.max_tokens > 32_000 {
            errors.push(format!(
                "loadout '{}' maxTokens must be 1..32000",
                loadout.id
            ));
        }
        for provider in &loadout.provider_ids {
            if !provider_ids.contains(provider) {
                errors.push(format!(
                    "loadout '{}' references missing provider '{provider}'",
                    loadout.id
                ));
            }
        }
        for source in &loadout.sources {
            let path = Path::new(&source.path);
            if path.is_absolute() || source.path.split('/').any(|p| p == "..") {
                errors.push(format!(
                    "loadout '{}' source '{}' must be repository-relative",
                    loadout.id, source.path
                ));
            }
        }
    }
    errors
}

fn unique_ids<'a>(
    ids: impl Iterator<Item = &'a str>,
    kind: &str,
    errors: &mut Vec<String>,
) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for id in ids {
        let id = id.trim();
        if id.is_empty() {
            errors.push(format!("{kind} id is required"));
        } else if !out.insert(id.to_string()) {
            errors.push(format!("duplicate {kind} id '{id}'"));
        }
    }
    out
}
