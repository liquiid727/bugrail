//! Logical Agent/Model profile resolution over the existing ACP runtime.

use std::collections::BTreeMap;
use std::path::Path;

use crate::db::error::DbError;
use crate::models::{ResolvedAgentRuntime, WorkTaskConfig, WorkTaskFolderSettings};

pub fn resolve(
    root: &Path,
    cfg: &WorkTaskConfig,
    settings: &WorkTaskFolderSettings,
    folder_default_agent: Option<String>,
) -> Result<ResolvedAgentRuntime, DbError> {
    let catalog = crate::specos_control::load_agents(root)?;
    if !catalog.validation_errors.is_empty() {
        return Err(DbError::Validation(catalog.validation_errors.join("; ")));
    }
    let selected_id = cfg
        .agent_profile_id
        .as_ref()
        .or(catalog.default_agent_profile_id.as_ref());
    if let Some(id) = selected_id {
        let profile = catalog
            .agent_profiles
            .iter()
            .find(|p| p.id == *id && p.enabled)
            .ok_or_else(|| {
                DbError::Validation(format!("agent profile '{id}' is missing or disabled"))
            })?;
        let model_id = cfg
            .model_profile_id
            .as_ref()
            .or(profile.model_profile_id.as_ref());
        let model_profile =
            model_id.and_then(|mid| catalog.model_profiles.iter().find(|m| m.id == *mid));
        if let Some(model_id) = model_id {
            if model_profile.is_none() {
                return Err(DbError::Validation(format!(
                    "model profile '{model_id}' is missing"
                )));
            }
        }
        let mut config_values = profile.config_values.clone();
        for (key, value) in &cfg.config_values {
            config_values.insert(key.clone(), value.clone());
        }
        if let Some(model) = model_profile.filter(|model| !model.model.trim().is_empty()) {
            config_values.insert("model".into(), model.model.clone());
        }
        if let Some(reasoning) = profile
            .reasoning
            .as_ref()
            .or(model_profile.and_then(|m| m.reasoning.as_ref()))
        {
            config_values.insert("reasoning_effort".into(), reasoning.clone());
        }
        return Ok(ResolvedAgentRuntime {
            agent_profile_id: Some(profile.id.clone()),
            model_profile_id: model_profile.map(|m| m.id.clone()),
            agent_type: profile.runtime_adapter.clone(),
            model: model_profile
                .and_then(|m| (!m.model.trim().is_empty()).then(|| m.model.clone()))
                .or_else(|| config_values.get("model").cloned()),
            mode_id: cfg.mode_id.clone().or_else(|| profile.mode_id.clone()),
            reasoning: profile
                .reasoning
                .clone()
                .or_else(|| model_profile.and_then(|m| m.reasoning.clone())),
            context_loadout_id: cfg
                .context_loadout_id
                .clone()
                .or_else(|| profile.context_loadout_id.clone()),
            config_values,
            reason_codes: vec![if cfg.agent_profile_id.is_some() {
                "explicit_task_profile"
            } else {
                "project_default_profile"
            }
            .into()],
        });
    }

    let (agent_type, mode_id, config_values, reason) = if let Some(agent) = cfg.agent_type.clone() {
        (
            agent,
            cfg.mode_id.clone(),
            cfg.config_values.clone(),
            "explicit_legacy_agent",
        )
    } else if let Some(agent) = settings.default_agent_type.clone() {
        (
            agent,
            settings.mode_id.clone(),
            settings.config_values.clone(),
            "folder_default_agent",
        )
    } else if let Some(agent) = folder_default_agent {
        (
            agent,
            settings.mode_id.clone(),
            settings.config_values.clone(),
            "workspace_default_agent",
        )
    } else {
        return Err(DbError::Validation(
            "no AgentProfile or legacy agent is configured".into(),
        ));
    };
    Ok(ResolvedAgentRuntime {
        agent_profile_id: None,
        model_profile_id: None,
        agent_type,
        model: config_values.get("model").cloned(),
        mode_id,
        reasoning: config_values.get("reasoning_effort").cloned(),
        context_loadout_id: cfg.context_loadout_id.clone(),
        config_values,
        reason_codes: vec![reason.into()],
    })
}

pub fn config_snapshot(runtime: &ResolvedAgentRuntime) -> BTreeMap<String, String> {
    runtime.config_values.clone()
}
