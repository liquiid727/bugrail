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
    AgentCatalog, ContextConfig, ContextLoadout, ContextProviderConfig, ContextSourceConfig,
    TeamCatalog,
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

/// The product's default Memory provider. TencentDB is currently the only
/// supported remote Memory adapter, so projects do not need to opt into a
/// provider just to get the default Memory behavior. Credentials remain
/// environment references and are resolved only when a request is sent.
pub fn default_memory_provider() -> ContextProviderConfig {
    ContextProviderConfig {
        id: "project-memory".into(),
        kind: crate::memory::MEMORY_KIND.into(),
        adapter: Some(crate::memory::ADAPTER_TENCENTDB_V3.into()),
        endpoint: Some("http://127.0.0.1:8420".into()),
        secret_env: Some("TENCENTDB_AGENT_MEMORY_API_KEY".into()),
        enabled: true,
        required: false,
        capabilities: vec![
            crate::memory::CAP_CAPTURE.into(),
            crate::memory::CAP_RECALL_L1.into(),
            crate::memory::CAP_RECALL_L3.into(),
        ],
        service_id_env: Some("TENCENTDB_AGENT_MEMORY_SERVICE_ID".into()),
        team_id: Some("team-example".into()),
        user_id_env: Some("TENCENTDB_AGENT_MEMORY_USER_ID".into()),
        default_agent_id: Some("agt-default".into()),
        agent_id_map: BTreeMap::new(),
        capture_enabled: true,
        recall_enabled: true,
        recall_limit: 5,
        include_core: false,
        timeout_ms: 5_000,
        max_capture_message_bytes: 8 * 1024,
        max_capture_batch_bytes: 256 * 1024,
    }
}

pub fn default_context_config() -> ContextConfig {
    let provider = default_memory_provider();
    let provider_id = provider.id.clone();
    ContextConfig {
        version: 1,
        default_loadout_id: "default".into(),
        providers: vec![provider],
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
            provider_ids: vec![provider_id],
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
    // No injection on load: a saved config without a Memory provider is a
    // deliberate legacy/no-Memory project and must stay zero-Memory (017 AC08).
    // New projects get the Memory provider through `default_context_config`.
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
    if value.teams.len() > 64 {
        errors.push("team catalog must contain at most 64 teams".into());
    }
    if value.workflows.len() > 128 {
        errors.push("team catalog must contain at most 128 workflows".into());
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
    for team in &value.teams {
        validate_text_length(&mut errors, "team", &team.id, 128);
        validate_text_length(&mut errors, "team name", &team.name, 512);
        validate_text_length(&mut errors, "team description", &team.description, 8 * 1024);
        if team.member_profile_ids.len() > 128 {
            errors.push(format!(
                "team '{}' may contain at most 128 member profiles",
                team.id
            ));
        }
    }
    for workflow in &value.workflows {
        validate_text_length(&mut errors, "workflow", &workflow.id, 128);
        validate_text_length(&mut errors, "workflow name", &workflow.name, 512);
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
            validate_text_length(&mut errors, "workflow node", &node.id, 128);
            validate_text_length(&mut errors, "workflow node title", &node.title, 512);
            validate_text_length(&mut errors, "workflow node prompt", &node.prompt, 64 * 1024);
            if node.depends_on.len() > 128 {
                errors.push(format!(
                    "workflow node '{}' may contain at most 128 dependencies",
                    node.id
                ));
            }
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

pub fn validate_teams_against_catalog(
    value: &TeamCatalog,
    agents: &AgentCatalog,
    context: &ContextConfig,
    require_enabled_profiles: bool,
) -> Vec<String> {
    let mut errors = validate_teams(value);
    errors.extend(agents.validation_errors.iter().cloned());
    errors.extend(context.validation_errors.iter().cloned());

    let profile_ids = agents
        .agent_profiles
        .iter()
        .map(|profile| profile.id.as_str())
        .collect::<BTreeSet<_>>();
    let enabled_profile_ids = agents
        .agent_profiles
        .iter()
        .filter(|profile| profile.enabled)
        .map(|profile| profile.id.as_str())
        .collect::<BTreeSet<_>>();
    let model_ids = agents
        .model_profiles
        .iter()
        .map(|profile| profile.id.as_str())
        .collect::<BTreeSet<_>>();
    let loadout_ids = context
        .loadouts
        .iter()
        .map(|loadout| loadout.id.as_str())
        .collect::<BTreeSet<_>>();

    for team in &value.teams {
        for profile_id in &team.member_profile_ids {
            if !profile_ids.contains(profile_id.as_str()) {
                errors.push(format!(
                    "team '{}' references missing AgentProfile '{profile_id}'",
                    team.id
                ));
            }
        }
    }
    for workflow in &value.workflows {
        let Some(team) = value.teams.iter().find(|team| team.id == workflow.team_id) else {
            continue;
        };
        for node in &workflow.nodes {
            if !profile_ids.contains(node.agent_profile_id.as_str()) {
                errors.push(format!(
                    "workflow node '{}' references missing AgentProfile '{}'",
                    node.id, node.agent_profile_id
                ));
            } else if require_enabled_profiles
                && !enabled_profile_ids.contains(node.agent_profile_id.as_str())
            {
                errors.push(format!(
                    "AgentProfile '{}' is unavailable",
                    node.agent_profile_id
                ));
            }
            if !team.member_profile_ids.contains(&node.agent_profile_id) {
                errors.push(format!(
                    "workflow node '{}' AgentProfile '{}' is not a member of Team '{}'",
                    node.id, node.agent_profile_id, team.id
                ));
            }
            if let Some(model_id) = &node.model_profile_id {
                if !model_ids.contains(model_id.as_str()) {
                    errors.push(format!(
                        "workflow node '{}' references a missing Model Profile",
                        node.id
                    ));
                }
            }
            if let Some(loadout_id) = &node.context_loadout_id {
                if !loadout_ids.contains(loadout_id.as_str()) {
                    errors.push(format!(
                        "workflow node '{}' references a missing Context Loadout",
                        node.id
                    ));
                }
            }
        }
    }
    errors
}

fn validate_text_length(errors: &mut Vec<String>, kind: &str, value: &str, max_bytes: usize) {
    if value.len() > max_bytes {
        errors.push(format!(
            "{kind} '{}' exceeds the {max_bytes}-byte limit",
            value.chars().take(32).collect::<String>()
        ));
    }
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
        if let Some(manifest) =
            crate::context::plugins::PluginManifest::from_provider_config(provider)
        {
            // Memory's production transport remains behind the existing
            // memory::AdapterRegistry and its typed validator. The shared
            // foundation registry validates the three new plugin kinds here.
            if manifest.kind != crate::context::plugins::PluginKind::Memory {
                if let Err(error) = crate::context::plugins::PluginRegistry::production()
                    .validate_manifest(&manifest)
                {
                    errors.push(format!(
                        "context provider '{}' plugin manifest: {error}",
                        safe_context_id(&provider.id)
                    ));
                }
            }
        }
        if provider.kind == "code-intelligence" {
            // Code Intelligence is a local adapter provider: it is always
            // optional (a degraded index never blocks a run) and never takes
            // a remote endpoint or secret.
            if provider.required {
                errors.push(format!(
                    "code-intelligence provider '{}' must not be required — degraded indexing never blocks runs",
                    provider.id
                ));
            }
            match provider.adapter.as_deref() {
                Some(crate::code_intelligence::manifest::ADAPTER_ID) => {}
                Some(other) => errors.push(format!(
                    "code-intelligence provider '{}' has unknown adapter '{other}' (expected '{}')",
                    provider.id,
                    crate::code_intelligence::manifest::ADAPTER_ID
                )),
                None => errors.push(format!(
                    "code-intelligence provider '{}' requires adapter: {}",
                    provider.id,
                    crate::code_intelligence::manifest::ADAPTER_ID
                )),
            }
            if provider.endpoint.is_some() || provider.secret_env.is_some() {
                errors.push(format!(
                    "code-intelligence provider '{}' must not set endpoint or secretEnv",
                    provider.id
                ));
            }
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
        // Typed Memory Provider rules (BUGRAIL-SPECOS-017 §3). Non-memory
        // providers keep the legacy behavior above and ignore the Memory
        // fields entirely.
        if provider.kind == crate::memory::MEMORY_KIND {
            errors.extend(crate::memory::config::validate_memory_provider(provider));
        } else if provider.kind != "code-intelligence"
            && crate::context::plugins::PluginKind::parse(&provider.kind).is_none()
            && provider.adapter.is_some()
        {
            errors.push(format!(
                "context provider '{}' adapter is only valid for kind '{}'",
                provider.id,
                crate::memory::MEMORY_KIND
            ));
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
        .into_iter()
        .map(|error| safe_diagnostic(&error))
        .collect()
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

fn safe_context_id(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
        .take(64)
        .collect();
    if sanitized.is_empty() {
        "<invalid>".into()
    } else {
        sanitized
    }
}

fn safe_diagnostic(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !ch.is_control())
        .take(256)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_context_loads_tencentdb_memory() {
        let config = default_context_config();
        let provider = config
            .providers
            .iter()
            .find(|provider| provider.kind == crate::memory::MEMORY_KIND)
            .expect("default Memory provider");

        assert_eq!(provider.id, "project-memory");
        assert_eq!(
            provider.adapter.as_deref(),
            Some(crate::memory::ADAPTER_TENCENTDB_V3)
        );
        assert_eq!(provider.endpoint.as_deref(), Some("http://127.0.0.1:8420"));
        assert!(config.loadouts[0].provider_ids.contains(&provider.id));
        assert!(validate_context(&config).is_empty());
    }

    #[test]
    fn plugin_manifest_validation_rejects_unknown_backend_without_resolution() {
        let config = ContextConfig {
            version: 1,
            default_loadout_id: "default".into(),
            providers: vec![ContextProviderConfig {
                id: "wiki-provider".into(),
                kind: "wiki".into(),
                adapter: Some("not-allowlisted".into()),
                ..ContextProviderConfig::default()
            }],
            loadouts: vec![ContextLoadout {
                id: "default".into(),
                provider_ids: vec!["wiki-provider".into()],
                ..ContextLoadout::default()
            }],
            validation_errors: Vec::new(),
        };
        let errors = validate_context(&config);
        assert!(errors.iter().any(|error| {
            error.contains("plugin manifest") && error.contains("static allowlist")
        }));
    }

    #[test]
    fn plugin_manifest_diagnostic_bounds_untrusted_provider_id() {
        let config = ContextConfig {
            version: 1,
            default_loadout_id: "default".into(),
            providers: vec![ContextProviderConfig {
                id: format!("bad\n{}", "x".repeat(300)),
                kind: "wiki".into(),
                adapter: Some("not-allowlisted".into()),
                ..ContextProviderConfig::default()
            }],
            loadouts: vec![ContextLoadout {
                id: "default".into(),
                provider_ids: vec![],
                max_items: 64,
                max_bytes: 512 * 1024,
                max_tokens: 32_000,
                ..ContextLoadout::default()
            }],
            validation_errors: Vec::new(),
        };
        let errors = validate_context(&config).join("; ");
        assert!(!errors.contains('\n'));
        assert!(errors.len() < 256);
    }

    #[test]
    fn independent_plugin_manifest_is_accepted_by_context_validation() {
        let mut config = default_context_config();
        config.providers = vec![ContextProviderConfig {
            id: "wiki-provider".into(),
            kind: "wiki".into(),
            adapter: Some(crate::context::plugins::ADAPTER_DETERMINISTIC_WIKI.into()),
            ..ContextProviderConfig::default()
        }];
        config.loadouts[0].provider_ids = vec!["wiki-provider".into()];
        assert!(validate_context(&config).is_empty());
    }

    #[test]
    fn legacy_context_without_memory_stays_zero_memory() {
        let root = tempfile::tempdir().expect("project root");
        let codeg = root.path().join(CONFIG_DIR);
        std::fs::create_dir_all(&codeg).expect(".codeg");
        let config = ContextConfig {
            version: 1,
            default_loadout_id: "default".into(),
            providers: Vec::new(),
            loadouts: vec![ContextLoadout {
                id: "default".into(),
                ..ContextLoadout::default()
            }],
            validation_errors: Vec::new(),
        };
        std::fs::write(
            codeg.join("context.yaml"),
            serde_yaml::to_string(&config).expect("serialize context"),
        )
        .expect("write context");

        let loaded = load_context(root.path()).expect("load context");
        assert_eq!(
            loaded
                .providers
                .iter()
                .filter(|provider| provider.kind == crate::memory::MEMORY_KIND)
                .count(),
            0,
            "saved no-Memory configs must stay zero-Memory (017 AC08)"
        );
        assert!(!loaded.loadouts[0]
            .provider_ids
            .contains(&"project-memory".into()));
    }

    #[test]
    fn existing_memory_provider_is_not_duplicated() {
        let root = tempfile::tempdir().expect("project root");
        let codeg = root.path().join(CONFIG_DIR);
        std::fs::create_dir_all(&codeg).expect(".codeg");
        let mut config = default_context_config();
        config.providers[0].id = "custom-memory".into();
        config.loadouts[0].provider_ids = vec!["custom-memory".into()];
        std::fs::write(
            codeg.join("context.yaml"),
            serde_yaml::to_string(&config).expect("serialize context"),
        )
        .expect("write context");

        let loaded = load_context(root.path()).expect("load context");
        assert_eq!(
            loaded
                .providers
                .iter()
                .filter(|provider| provider.kind == crate::memory::MEMORY_KIND)
                .count(),
            1
        );
        assert_eq!(loaded.providers[0].id, "custom-memory");
    }
}
