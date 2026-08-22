//! Typed Memory Provider configuration (BUGRAIL-SPECOS-017 §3, §9).
//!
//! Two layers:
//! - [`validate_memory_provider`] — static checks run while saving
//!   `.codeg/context.yaml` (no environment access, no network).
//! - [`resolve_memory_provider`] — runtime resolution in the backend only:
//!   environment references become values here and never travel back to
//!   config responses, errors, logs or the frontend.

use std::path::Path;
use std::time::Duration;

use crate::models::ContextProviderConfig;

use super::{MemoryError, MemoryErrorClass};

/// `ContextProviderConfig.kind` that activates the Memory module.
pub const MEMORY_KIND: &str = "memory";
/// The only Adapter in the MVP01 static allowlist.
pub const ADAPTER_TENCENTDB_V3: &str = "tencentdb-agent-memory-v3";

/// Provider capability allowlist (spec §3 rule 5).
pub const CAP_CAPTURE: &str = "memory.capture";
pub const CAP_RECALL_L1: &str = "memory.recall.l1";
pub const CAP_RECALL_L3: &str = "memory.recall.l3";
const KNOWN_CAPABILITIES: [&str; 3] = [CAP_CAPTURE, CAP_RECALL_L1, CAP_RECALL_L3];

pub const MIN_TIMEOUT_MS: u64 = 500;
pub const MAX_TIMEOUT_MS: u64 = 30_000;
pub const MIN_RECALL_LIMIT: u32 = 1;
pub const MAX_RECALL_LIMIT: u32 = 20;
/// Upper bound for configurable capture caps (spec defaults are 8 KiB /
/// 256 KiB; operators may lower or raise them within these absolute bounds).
pub const MAX_CAPTURE_MESSAGE_BYTES_BOUND: usize = 64 * 1024;
pub const MAX_CAPTURE_BATCH_BYTES_BOUND: usize = 1024 * 1024;

/// Runtime-resolved Memory Provider. Holds resolved secrets in memory only —
/// this type is never serialized, logged or returned to clients.
#[derive(Debug, Clone)]
pub struct ResolvedMemoryProvider {
    pub provider_id: String,
    pub adapter: String,
    pub endpoint: String,
    /// Bearer token resolved from `secret_env`.
    pub secret: String,
    /// Upstream service id resolved from `service_id_env`.
    pub service_id: String,
    pub team_id: String,
    /// Upstream user id resolved from `user_id_env`.
    pub user_id: String,
    pub agent_id: String,
    pub required: bool,
    pub capture_enabled: bool,
    pub recall_enabled: bool,
    pub recall_limit: u32,
    pub include_core: bool,
    pub timeout: Duration,
    pub max_capture_message_bytes: usize,
    pub max_capture_batch_bytes: usize,
    pub capabilities: Vec<String>,
}

impl ResolvedMemoryProvider {
    pub fn can_capture(&self) -> bool {
        self.capture_enabled && self.capabilities.iter().any(|c| c == CAP_CAPTURE)
    }

    pub fn can_recall(&self) -> bool {
        self.recall_enabled && self.capabilities.iter().any(|c| c == CAP_RECALL_L1)
    }

    pub fn can_recall_core(&self) -> bool {
        self.include_core && self.capabilities.iter().any(|c| c == CAP_RECALL_L3)
    }
}

/// Static validation for one `kind = "memory"` provider document. Returns
/// human-readable error strings in the same style as the rest of
/// `validate_context`. Never touches the environment.
pub fn validate_memory_provider(provider: &ContextProviderConfig) -> Vec<String> {
    let mut errors = Vec::new();
    let id = provider.id.as_str();

    match provider.adapter.as_deref() {
        Some(ADAPTER_TENCENTDB_V3) => {}
        Some(other) => errors.push(format!(
            "memory provider '{id}' adapter '{other}' is not in the static allowlist"
        )),
        None => errors.push(format!(
            "memory provider '{id}' requires adapter '{ADAPTER_TENCENTDB_V3}'"
        )),
    }

    match provider.endpoint.as_deref() {
        Some(endpoint) => {
            if let Err(reason) = validate_memory_endpoint(endpoint) {
                errors.push(format!("memory provider '{id}' {reason}"));
            }
        }
        None => errors.push(format!("memory provider '{id}' requires an endpoint")),
    }

    for (field, value) in [
        ("secretEnv", &provider.secret_env),
        ("serviceIdEnv", &provider.service_id_env),
        ("userIdEnv", &provider.user_id_env),
    ] {
        match value {
            Some(name) if is_env_name(name) => {}
            Some(_) => errors.push(format!(
                "memory provider '{id}' {field} must be an uppercase environment variable name"
            )),
            None => errors.push(format!("memory provider '{id}' requires {field}")),
        }
    }

    if provider
        .team_id
        .as_deref()
        .unwrap_or_default()
        .trim()
        .is_empty()
    {
        errors.push(format!(
            "memory provider '{id}' requires a project-specific teamId"
        ));
    }

    let has_map = provider
        .agent_id_map
        .values()
        .any(|value| !value.trim().is_empty());
    let has_default = provider
        .default_agent_id
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty());
    if !has_map && !has_default {
        errors.push(format!(
            "memory provider '{id}' requires defaultAgentId or an agentIdMap entry"
        ));
    }

    for capability in &provider.capabilities {
        if !KNOWN_CAPABILITIES.contains(&capability.as_str()) {
            errors.push(format!(
                "memory provider '{id}' capability '{capability}' is not supported by this adapter"
            ));
        }
    }

    if !(MIN_TIMEOUT_MS..=MAX_TIMEOUT_MS).contains(&provider.timeout_ms) {
        errors.push(format!(
            "memory provider '{id}' timeoutMs must be {MIN_TIMEOUT_MS}..={MAX_TIMEOUT_MS}"
        ));
    }
    if !(MIN_RECALL_LIMIT..=MAX_RECALL_LIMIT).contains(&provider.recall_limit) {
        errors.push(format!(
            "memory provider '{id}' recallLimit must be {MIN_RECALL_LIMIT}..={MAX_RECALL_LIMIT}"
        ));
    }
    if provider.max_capture_message_bytes == 0
        || provider.max_capture_message_bytes > MAX_CAPTURE_MESSAGE_BYTES_BOUND
    {
        errors.push(format!(
            "memory provider '{id}' maxCaptureMessageBytes must be 1..={MAX_CAPTURE_MESSAGE_BYTES_BOUND}"
        ));
    }
    if provider.max_capture_batch_bytes == 0
        || provider.max_capture_batch_bytes > MAX_CAPTURE_BATCH_BYTES_BOUND
    {
        errors.push(format!(
            "memory provider '{id}' maxCaptureBatchBytes must be 1..={MAX_CAPTURE_BATCH_BYTES_BOUND}"
        ));
    }
    if provider.max_capture_message_bytes > provider.max_capture_batch_bytes {
        errors.push(format!(
            "memory provider '{id}' maxCaptureMessageBytes cannot exceed maxCaptureBatchBytes"
        ));
    }

    errors
}

fn is_env_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
}

/// Endpoint security gate (spec §9): HTTPS everywhere except loopback HTTP;
/// URLs carrying credentials are rejected outright. Used by both static
/// validation and runtime resolution so a hand-edited config cannot drift
/// past the runtime path.
pub fn validate_memory_endpoint(endpoint: &str) -> Result<(), String> {
    let url =
        reqwest::Url::parse(endpoint).map_err(|e| format!("endpoint is not a valid URL: {}", e))?;
    if !url.username().is_empty() || url.password().is_some() {
        return Err("endpoint must not contain credentials".into());
    }
    match url.scheme() {
        "https" => {}
        "http" => {
            if !host_is_loopback(&url) {
                return Err("endpoint must use HTTPS except for loopback hosts".into());
            }
        }
        other => return Err(format!("endpoint scheme '{other}' is not supported")),
    }
    if url.host_str().is_none() {
        return Err("endpoint must have a host".into());
    }
    Ok(())
}

fn host_is_loopback(url: &reqwest::Url) -> bool {
    match url.host_str() {
        Some(host) if host.eq_ignore_ascii_case("localhost") => true,
        Some(host) => host
            .trim_start_matches('[')
            .trim_end_matches(']')
            .parse::<std::net::IpAddr>()
            .map(|ip| ip.is_loopback())
            .unwrap_or(false),
        None => false,
    }
}

/// Runtime resolution. `agent_profile_id` selects the exact `agent_id_map`
/// entry; the fallback is `default_agent_id`. Any missing identity fails
/// with a stable class and the caller must not issue a network request.
pub fn resolve_memory_provider(
    provider: &ContextProviderConfig,
    agent_profile_id: Option<&str>,
) -> Result<ResolvedMemoryProvider, MemoryError> {
    let provider_id = provider.id.clone();
    let fail = |class: MemoryErrorClass, message: String| {
        Err(MemoryError {
            class,
            message,
            provider_id: provider_id.clone(),
            trace_id: None,
        })
    };

    let adapter = match provider.adapter.as_deref() {
        Some(ADAPTER_TENCENTDB_V3) => ADAPTER_TENCENTDB_V3.to_string(),
        Some(other) => {
            return fail(
                MemoryErrorClass::ConfigInvalid,
                format!("adapter '{other}' is not in the static allowlist"),
            )
        }
        None => {
            return fail(
                MemoryErrorClass::ConfigInvalid,
                "memory provider requires an adapter".into(),
            )
        }
    };

    let endpoint = match provider.endpoint.as_deref() {
        Some(endpoint) => endpoint.trim_end_matches('/').to_string(),
        None => {
            return fail(
                MemoryErrorClass::ConfigInvalid,
                "memory provider requires an endpoint".into(),
            )
        }
    };
    if let Err(reason) = validate_memory_endpoint(&endpoint) {
        return fail(MemoryErrorClass::ConfigInvalid, reason);
    }

    let secret = read_env_reference(&provider_id, provider.secret_env.as_deref(), "secretEnv")?;
    let service_id = read_env_reference(
        &provider_id,
        provider.service_id_env.as_deref(),
        "serviceIdEnv",
    )?;
    let user_id = read_env_reference(&provider_id, provider.user_id_env.as_deref(), "userIdEnv")?;

    let team_id = provider
        .team_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| MemoryError {
            class: MemoryErrorClass::IdentityMissing,
            message: "teamId is not configured".into(),
            provider_id: provider_id.clone(),
            trace_id: None,
        })?
        .to_string();

    let agent_id = agent_profile_id
        .and_then(|profile_id| provider.agent_id_map.get(profile_id))
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            provider
                .default_agent_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .ok_or_else(|| MemoryError {
            class: MemoryErrorClass::IdentityMissing,
            message: "agent identity could not be resolved".into(),
            provider_id: provider_id.clone(),
            trace_id: None,
        })?
        .to_string();

    Ok(ResolvedMemoryProvider {
        provider_id,
        adapter,
        endpoint,
        secret,
        service_id,
        team_id,
        user_id,
        agent_id,
        required: provider.required,
        capture_enabled: provider.capture_enabled,
        recall_enabled: provider.recall_enabled,
        recall_limit: provider.recall_limit,
        include_core: provider.include_core,
        timeout: Duration::from_millis(provider.timeout_ms),
        max_capture_message_bytes: provider.max_capture_message_bytes,
        max_capture_batch_bytes: provider.max_capture_batch_bytes,
        capabilities: provider.capabilities.clone(),
    })
}

fn read_env_reference(
    provider_id: &str,
    env_name: Option<&str>,
    field: &str,
) -> Result<String, MemoryError> {
    let name = env_name
        .filter(|name| !name.trim().is_empty())
        .ok_or_else(|| MemoryError {
            class: MemoryErrorClass::ConfigInvalid,
            message: format!("{field} is not configured"),
            provider_id: provider_id.to_string(),
            trace_id: None,
        })?;
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| MemoryError {
            class: MemoryErrorClass::ConfigInvalid,
            // Names the REFERENCE, never a resolved value.
            message: format!("environment variable '{name}' referenced by {field} is not set"),
            provider_id: provider_id.to_string(),
            trace_id: None,
        })
}

/// True when the provider document activates Memory behavior.
pub fn is_memory_provider(provider: &ContextProviderConfig) -> bool {
    provider.kind == MEMORY_KIND
}

/// Project root helper kept local so Memory never widens folder access:
/// identity derivation only needs the folder's path.
pub fn binding_for_folder(folder_path: &Path) -> String {
    super::identity::project_binding(folder_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_provider() -> ContextProviderConfig {
        ContextProviderConfig {
            id: "project-memory".into(),
            kind: MEMORY_KIND.into(),
            adapter: Some(ADAPTER_TENCENTDB_V3.into()),
            endpoint: Some("https://memory.example.com".into()),
            secret_env: Some("TDAI_SECRET".into()),
            service_id_env: Some("TDAI_SERVICE_ID".into()),
            team_id: Some("team-alpha".into()),
            user_id_env: Some("TDAI_USER_ID".into()),
            default_agent_id: Some("bugrail-agent".into()),
            capabilities: vec![CAP_CAPTURE.into(), CAP_RECALL_L1.into()],
            enabled: true,
            ..Default::default()
        }
    }

    #[test]
    fn valid_provider_passes_static_validation() {
        assert!(validate_memory_provider(&base_provider()).is_empty());
    }

    #[test]
    fn adapter_allowlist_is_strict() {
        let mut provider = base_provider();
        provider.adapter = Some("other-memory".into());
        let errors = validate_memory_provider(&provider);
        assert!(errors.iter().any(|e| e.contains("allowlist")));

        provider.adapter = None;
        let errors = validate_memory_provider(&provider);
        assert!(errors.iter().any(|e| e.contains("requires adapter")));
    }

    #[test]
    fn endpoint_rules_enforce_https_loopback_and_no_credentials() {
        assert!(validate_memory_endpoint("https://memory.example.com").is_ok());
        assert!(validate_memory_endpoint("http://127.0.0.1:8420").is_ok());
        assert!(validate_memory_endpoint("http://localhost:8420").is_ok());
        assert!(validate_memory_endpoint("http://[::1]:8420").is_ok());
        assert!(validate_memory_endpoint("http://10.0.0.5:8420").is_err());
        assert!(validate_memory_endpoint("http://memory.example.com").is_err());
        assert!(validate_memory_endpoint("https://user:pass@memory.example.com").is_err());
        assert!(validate_memory_endpoint("ftp://memory.example.com").is_err());
        assert!(validate_memory_endpoint("not a url").is_err());
    }

    #[test]
    fn bounds_are_checked() {
        let mut provider = base_provider();
        provider.timeout_ms = 100;
        provider.recall_limit = 21;
        provider.max_capture_message_bytes = 0;
        provider.max_capture_batch_bytes = 10 * 1024 * 1024;
        let errors = validate_memory_provider(&provider);
        assert!(errors.iter().any(|e| e.contains("timeoutMs")));
        assert!(errors.iter().any(|e| e.contains("recallLimit")));
        assert!(errors.iter().any(|e| e.contains("maxCaptureMessageBytes")));
        assert!(errors.iter().any(|e| e.contains("maxCaptureBatchBytes")));
    }

    #[test]
    fn unknown_capabilities_are_rejected() {
        let mut provider = base_provider();
        provider.capabilities.push("wiki".into());
        let errors = validate_memory_provider(&provider);
        assert!(errors.iter().any(|e| e.contains("wiki")));
    }

    #[test]
    fn identity_requires_team_and_agent() {
        let mut provider = base_provider();
        provider.team_id = None;
        provider.default_agent_id = None;
        let errors = validate_memory_provider(&provider);
        assert!(errors.iter().any(|e| e.contains("teamId")));
        assert!(errors.iter().any(|e| e.contains("defaultAgentId")));
    }

    #[test]
    fn resolution_maps_agents_exactly_then_defaults() {
        let mut provider = base_provider();
        provider
            .agent_id_map
            .insert("claude".into(), "agent-claude".into());

        let resolved = resolve_memory_provider_with_env(&provider, Some("claude"));
        assert_eq!(resolved.agent_id, "agent-claude");

        let resolved = resolve_memory_provider_with_env(&provider, Some("codex"));
        assert_eq!(resolved.agent_id, "bugrail-agent");

        let resolved = resolve_memory_provider_with_env(&provider, None);
        assert_eq!(resolved.agent_id, "bugrail-agent");
    }

    #[test]
    fn missing_identity_never_builds_a_provider() {
        // SAFETY: same module-local env keys as resolve_memory_provider_with_env.
        std::env::set_var("TDAI_SECRET", "s");
        std::env::set_var("TDAI_SERVICE_ID", "svc");
        std::env::set_var("TDAI_USER_ID", "user");
        let mut provider = base_provider();
        provider.default_agent_id = None;
        let err = resolve_memory_provider(&provider, None).unwrap_err();
        assert_eq!(err.class, MemoryErrorClass::IdentityMissing);
    }

    /// Test helper: resolve with the three env references pinned.
    fn resolve_memory_provider_with_env(
        provider: &ContextProviderConfig,
        agent_profile_id: Option<&str>,
    ) -> ResolvedMemoryProvider {
        // SAFETY: test-local env keys unique to this module.
        std::env::set_var("TDAI_SECRET", "s");
        std::env::set_var("TDAI_SERVICE_ID", "svc");
        std::env::set_var("TDAI_USER_ID", "user");
        resolve_memory_provider(provider, agent_profile_id).expect("resolves")
    }
}
