//! Independent Context plugin contracts (BUGRAIL-SPECOS-028).
//!
//! The Context plane owns the registry and the vendor-neutral envelope. Each
//! plugin kind has its own trait and result type; there is deliberately no
//! catch-all asset trait for Adapters to implement. Runtime Memory transport
//! remains owned by [`crate::memory::MemoryProvider`]. This module supplies
//! the foundation contract and static construction boundary used by future
//! Wiki, CodeGraph and Skill implementations.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::models::{ContextConfig, ContextProviderConfig};

pub const ADAPTER_DETERMINISTIC_MEMORY: &str = "deterministic-memory";
pub const ADAPTER_DETERMINISTIC_WIKI: &str = "deterministic-wiki";
pub const ADAPTER_DETERMINISTIC_CODEGRAPH: &str = "deterministic-codegraph";
pub const ADAPTER_DETERMINISTIC_SKILL: &str = "deterministic-skill";

/// The only primary kinds accepted by the Context plugin foundation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PluginKind {
    Memory,
    Wiki,
    #[serde(rename = "codegraph")]
    CodeGraph,
    Skill,
}

impl PluginKind {
    pub const ALL: [Self; 4] = [Self::Memory, Self::Wiki, Self::CodeGraph, Self::Skill];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Memory => "memory",
            Self::Wiki => "wiki",
            Self::CodeGraph => "codegraph",
            Self::Skill => "skill",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "memory" => Some(Self::Memory),
            "wiki" => Some(Self::Wiki),
            "codegraph" => Some(Self::CodeGraph),
            "skill" => Some(Self::Skill),
            _ => None,
        }
    }
}

/// Safe, vendor-neutral scope for an asset. Values are identifiers only;
/// credentials and provider payloads never belong in this envelope.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetScope {
    pub project_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_profile_id: Option<String>,
}

/// Safe provenance retained with an AssetRef. It intentionally has no free
/// form metadata map, so vendor DTOs cannot leak through this boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetProvenance {
    pub source: String,
}

/// Stable identity and lineage envelope shared by all plugin results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetRef {
    pub id: String,
    pub kind: PluginKind,
    pub scope: AssetScope,
    pub provider: String,
    pub revision: String,
    pub provenance: AssetProvenance,
}

impl AssetRef {
    pub fn validate(&self) -> Result<(), PluginContractError> {
        if self.id.trim().is_empty()
            || self.id.chars().count() > 256
            || self.scope.project_id.trim().is_empty()
            || self.scope.project_id.chars().count() > 256
            || self.provider.trim().is_empty()
            || self.provider.chars().count() > 128
            || self.revision.trim().is_empty()
            || self.revision.chars().count() > 128
            || self.provenance.source.trim().is_empty()
            || self.provenance.source.chars().count() > 256
        {
            return Err(PluginContractError::InvalidAssetRef);
        }
        for value in [
            self.scope.folder_id.as_deref(),
            self.scope.agent_profile_id.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            if value.trim().is_empty() || value.chars().count() > 256 {
                return Err(PluginContractError::InvalidAssetRef);
            }
        }
        Ok(())
    }
}

/// Normalized content that can enter the existing Context compiler. The
/// source plugin result is converted to this shape before it reaches
/// `EngineContextItem`; vendor-specific response types stay inside Adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextCandidate {
    pub asset: AssetRef,
    pub title: String,
    pub content: String,
}

impl ContextCandidate {
    pub fn into_engine_item(self) -> Result<super::EngineContextItem, PluginContractError> {
        self.asset.validate()?;
        if self.title.trim().is_empty() || self.title.chars().count() > 512 {
            return Err(PluginContractError::InvalidContextCandidate);
        }
        Ok(super::EngineContextItem {
            kind: self.asset.kind.as_str().to_string(),
            source: format!("plugin/{}/{}", self.asset.kind.as_str(), self.asset.id),
            title: self.title,
            content: self.content,
            provenance: serde_json::json!({ "assetRef": self.asset }),
        })
    }
}

/// Convert normalized candidates into the shape consumed by the existing
/// budget, deduplication and persistence pipeline. Input order is preserved;
/// the compiler remains the authority for exact budget adjudication.
pub fn to_context_items<I>(
    candidates: I,
) -> Result<Vec<super::EngineContextItem>, PluginContractError>
where
    I: IntoIterator<Item = ContextCandidate>,
{
    candidates
        .into_iter()
        .map(ContextCandidate::into_engine_item)
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginManifest {
    pub id: String,
    pub kind: PluginKind,
    pub adapter: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    /// Environment variable name only. The value is never read by this
    /// module and is never copied into a health projection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret_env: Option<String>,
}

impl PluginManifest {
    pub fn from_provider_config(provider: &ContextProviderConfig) -> Option<Self> {
        Some(Self {
            id: provider.id.clone(),
            kind: PluginKind::parse(&provider.kind)?,
            adapter: provider.adapter.clone().unwrap_or_default(),
            enabled: provider.enabled,
            required: provider.required,
            capabilities: provider.capabilities.clone(),
            endpoint: provider.endpoint.clone(),
            secret_env: provider.secret_env.clone(),
        })
    }
}

/// Return a renderer-safe copy of persisted Context configuration. The file
/// remains the authoritative editable source, but malformed endpoint values
/// (including embedded credentials) must not be reflected into transport
/// payloads or an operational UI snapshot.
pub fn redact_context_config(config: &ContextConfig) -> ContextConfig {
    let mut safe = config.clone();
    safe.providers = config
        .providers
        .iter()
        .map(redact_provider_config)
        .collect();
    safe.validation_errors = config
        .validation_errors
        .iter()
        .map(|value| redact_diagnostic(value))
        .collect();
    safe
}

pub fn redact_provider_config(provider: &ContextProviderConfig) -> ContextProviderConfig {
    let mut safe = provider.clone();
    safe.id = safe_identifier(&provider.id, 128);
    safe.kind = safe_identifier(&provider.kind, 64);
    safe.adapter = provider
        .adapter
        .as_deref()
        .map(|value| safe_identifier(value, 128));
    safe.endpoint = provider.endpoint.as_deref().and_then(safe_endpoint);
    safe.secret_env = provider.secret_env.as_deref().and_then(safe_env_name);
    safe.service_id_env = provider.service_id_env.as_deref().and_then(safe_env_name);
    safe.team_id = provider.team_id.as_deref().map(redact_diagnostic);
    safe.user_id_env = provider.user_id_env.as_deref().and_then(safe_env_name);
    safe.default_agent_id = provider.default_agent_id.as_deref().map(redact_diagnostic);
    safe.agent_id_map = provider
        .agent_id_map
        .iter()
        .map(|(key, value)| (redact_diagnostic(key), redact_diagnostic(value)))
        .collect();
    safe.capabilities = provider
        .capabilities
        .iter()
        .map(|value| safe_identifier(value, 128))
        .collect();
    safe
}

/// Strip controls, cap untrusted diagnostics and replace values that look like
/// runtime credentials with a stable generic message.
pub fn redact_diagnostic(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .filter(|ch| !ch.is_control())
        .take(256)
        .collect();
    let lower = sanitized.to_ascii_lowercase();
    if [
        "authorization",
        "api_key",
        "apikey",
        "credential",
        "password",
        "secret",
        "token",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
    {
        "provider diagnostic redacted".into()
    } else {
        sanitized
    }
}

fn safe_identifier(value: &str, max: usize) -> String {
    let sanitized: String = value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
        .take(max)
        .collect();
    if sanitized.is_empty() {
        "<invalid>".into()
    } else {
        sanitized
    }
}

fn safe_env_name(value: &str) -> Option<String> {
    (valid_env_name(value) && value.len() <= 128).then(|| value.to_string())
}

fn safe_endpoint(value: &str) -> Option<String> {
    let url = reqwest::Url::parse(value).ok()?;
    if !(url.scheme() == "http" || url.scheme() == "https")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || value.chars().any(char::is_control)
    {
        return None;
    }
    Some(url.to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PluginHealthStatus {
    Healthy,
    Degraded,
    Disabled,
}

/// Registry-owned, client-safe health projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginHealth {
    pub id: String,
    pub kind: PluginKind,
    pub status: PluginHealthStatus,
    pub capabilities: Vec<String>,
    pub message: Option<String>,
}

// ── Independent plugin contracts ──────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryAsset {
    pub asset: AssetRef,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiAsset {
    pub asset: AssetRef,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeGraphAsset {
    pub asset: AssetRef,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillAsset {
    pub asset: AssetRef,
    pub title: String,
    pub content: String,
}

pub trait MemoryPluginAdapter: Send + Sync {
    fn manifest(&self) -> &PluginManifest;
    fn health(&self) -> PluginHealth;
    fn recall(&self, query: &str, limit: usize) -> Vec<MemoryAsset>;
}

pub trait WikiPluginAdapter: Send + Sync {
    fn manifest(&self) -> &PluginManifest;
    fn health(&self) -> PluginHealth;
    fn search(&self, query: &str, limit: usize) -> Vec<WikiAsset>;
}

pub trait CodeGraphPluginAdapter: Send + Sync {
    fn manifest(&self) -> &PluginManifest;
    fn health(&self) -> PluginHealth;
    fn query(&self, query: &str, limit: usize) -> Vec<CodeGraphAsset>;
}

pub trait SkillPluginAdapter: Send + Sync {
    fn manifest(&self) -> &PluginManifest;
    fn health(&self) -> PluginHealth;
    fn discover(&self, query: &str, limit: usize) -> Vec<SkillAsset>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginRegistryError {
    InvalidManifest,
    DuplicateId,
    UnsupportedAdapter,
    ConstructionFailed,
}

impl fmt::Display for PluginRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidManifest => "plugin manifest is invalid",
            Self::DuplicateId => "duplicate plugin id",
            Self::UnsupportedAdapter => "plugin adapter is not in the static allowlist",
            Self::ConstructionFailed => "plugin adapter construction failed",
        })
    }
}

impl std::error::Error for PluginRegistryError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginContractError {
    InvalidAssetRef,
    InvalidContextCandidate,
    AssetKindMismatch,
}

impl fmt::Display for PluginContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidAssetRef => "asset reference is invalid",
            Self::InvalidContextCandidate => "context candidate is invalid",
            Self::AssetKindMismatch => "asset kind does not match plugin contract",
        })
    }
}

impl std::error::Error for PluginContractError {}

struct RegisteredMemory {
    manifest: PluginManifest,
    adapter: Option<Arc<dyn MemoryPluginAdapter>>,
}

struct RegisteredWiki {
    manifest: PluginManifest,
    adapter: Option<Arc<dyn WikiPluginAdapter>>,
}

struct RegisteredCodeGraph {
    manifest: PluginManifest,
    adapter: Option<Arc<dyn CodeGraphPluginAdapter>>,
}

struct RegisteredSkill {
    manifest: PluginManifest,
    adapter: Option<Arc<dyn SkillPluginAdapter>>,
}

/// Static, kind-separated backend registry. There is no public registration
/// hook: callers can inspect the typed adapter for a configured id, but cannot
/// inject an arbitrary implementation or call another kind's methods.
pub struct PluginRegistry {
    memory: BTreeMap<String, RegisteredMemory>,
    wiki: BTreeMap<String, RegisteredWiki>,
    codegraph: BTreeMap<String, RegisteredCodeGraph>,
    skill: BTreeMap<String, RegisteredSkill>,
}

impl PluginRegistry {
    pub fn production() -> Self {
        Self {
            memory: BTreeMap::new(),
            wiki: BTreeMap::new(),
            codegraph: BTreeMap::new(),
            skill: BTreeMap::new(),
        }
    }

    /// Four deterministic adapters are the contract fixture for T01/T02.
    /// They are constructed by this module's static match, not by a dynamic
    /// loader or a caller-provided factory.
    pub fn deterministic() -> Self {
        let manifests = [
            manifest(
                "memory-fixture",
                PluginKind::Memory,
                ADAPTER_DETERMINISTIC_MEMORY,
            ),
            manifest("wiki-fixture", PluginKind::Wiki, ADAPTER_DETERMINISTIC_WIKI),
            manifest(
                "codegraph-fixture",
                PluginKind::CodeGraph,
                ADAPTER_DETERMINISTIC_CODEGRAPH,
            ),
            manifest(
                "skill-fixture",
                PluginKind::Skill,
                ADAPTER_DETERMINISTIC_SKILL,
            ),
        ];
        Self::from_manifests(&manifests).expect("deterministic manifests are static and valid")
    }

    pub fn from_manifests(manifests: &[PluginManifest]) -> Result<Self, PluginRegistryError> {
        let registry = Self::production();
        let mut ids = BTreeSet::new();
        for manifest in manifests {
            registry.validate_manifest(manifest)?;
            if !ids.insert(manifest.id.clone()) {
                return Err(PluginRegistryError::DuplicateId);
            }
        }

        let mut registry = registry;
        for manifest in manifests {
            match manifest.kind {
                PluginKind::Memory => {
                    let adapter = if manifest.enabled {
                        Some(build_memory(manifest)?)
                    } else {
                        None
                    };
                    registry.memory.insert(
                        manifest.id.clone(),
                        RegisteredMemory {
                            manifest: manifest.clone(),
                            adapter,
                        },
                    );
                }
                PluginKind::Wiki => {
                    let adapter = if manifest.enabled {
                        Some(build_wiki(manifest)?)
                    } else {
                        None
                    };
                    registry.wiki.insert(
                        manifest.id.clone(),
                        RegisteredWiki {
                            manifest: manifest.clone(),
                            adapter,
                        },
                    );
                }
                PluginKind::CodeGraph => {
                    let adapter = if manifest.enabled {
                        Some(build_codegraph(manifest)?)
                    } else {
                        None
                    };
                    registry.codegraph.insert(
                        manifest.id.clone(),
                        RegisteredCodeGraph {
                            manifest: manifest.clone(),
                            adapter,
                        },
                    );
                }
                PluginKind::Skill => {
                    let adapter = if manifest.enabled {
                        Some(build_skill(manifest)?)
                    } else {
                        None
                    };
                    registry.skill.insert(
                        manifest.id.clone(),
                        RegisteredSkill {
                            manifest: manifest.clone(),
                            adapter,
                        },
                    );
                }
            }
        }
        Ok(registry)
    }

    /// Validate without reading environment variables or constructing an
    /// Adapter. This is the configuration gate used by `validate_context`.
    pub fn validate_manifest(&self, manifest: &PluginManifest) -> Result<(), PluginRegistryError> {
        if !valid_name(&manifest.id, 128)
            || !valid_name(&manifest.adapter, 128)
            || manifest.required && !manifest.enabled
            || manifest.capabilities.iter().any(|cap| {
                !valid_name(cap, 128)
                    || !capability_allowlist(manifest.kind).contains(&cap.as_str())
            })
        {
            return Err(PluginRegistryError::InvalidManifest);
        }
        if let Some(endpoint) = &manifest.endpoint {
            let Ok(url) = reqwest::Url::parse(endpoint) else {
                return Err(PluginRegistryError::InvalidManifest);
            };
            if !(url.scheme() == "http" || url.scheme() == "https")
                || !url.username().is_empty()
                || url.password().is_some()
                || url.query().is_some()
                || url.fragment().is_some()
            {
                return Err(PluginRegistryError::InvalidManifest);
            }
        }
        if let Some(secret_env) = &manifest.secret_env {
            if !valid_env_name(secret_env) {
                return Err(PluginRegistryError::InvalidManifest);
            }
        }
        if !supported_adapter(manifest.kind, &manifest.adapter) {
            return Err(PluginRegistryError::UnsupportedAdapter);
        }
        Ok(())
    }

    pub fn memory(&self, id: &str) -> Option<&dyn MemoryPluginAdapter> {
        self.memory.get(id)?.adapter.as_deref()
    }

    pub fn wiki(&self, id: &str) -> Option<&dyn WikiPluginAdapter> {
        self.wiki.get(id)?.adapter.as_deref()
    }

    pub fn codegraph(&self, id: &str) -> Option<&dyn CodeGraphPluginAdapter> {
        self.codegraph.get(id)?.adapter.as_deref()
    }

    pub fn skill(&self, id: &str) -> Option<&dyn SkillPluginAdapter> {
        self.skill.get(id)?.adapter.as_deref()
    }

    pub fn health_for(&self, kind: PluginKind, id: &str) -> Option<PluginHealth> {
        match kind {
            PluginKind::Memory => self.memory.get(id).map(|entry| {
                project_health(
                    &entry.manifest,
                    entry.adapter.as_deref().map(|adapter| adapter.health()),
                )
            }),
            PluginKind::Wiki => self.wiki.get(id).map(|entry| {
                project_health(
                    &entry.manifest,
                    entry.adapter.as_deref().map(|adapter| adapter.health()),
                )
            }),
            PluginKind::CodeGraph => self.codegraph.get(id).map(|entry| {
                project_health(
                    &entry.manifest,
                    entry.adapter.as_deref().map(|adapter| adapter.health()),
                )
            }),
            PluginKind::Skill => self.skill.get(id).map(|entry| {
                project_health(
                    &entry.manifest,
                    entry.adapter.as_deref().map(|adapter| adapter.health()),
                )
            }),
        }
    }

    pub fn health_projection(&self) -> Vec<PluginHealth> {
        let mut health = Vec::new();
        for entry in self.memory.values() {
            health.push(project_health(
                &entry.manifest,
                entry.adapter.as_deref().map(|adapter| adapter.health()),
            ));
        }
        for entry in self.wiki.values() {
            health.push(project_health(
                &entry.manifest,
                entry.adapter.as_deref().map(|adapter| adapter.health()),
            ));
        }
        for entry in self.codegraph.values() {
            health.push(project_health(
                &entry.manifest,
                entry.adapter.as_deref().map(|adapter| adapter.health()),
            ));
        }
        for entry in self.skill.values() {
            health.push(project_health(
                &entry.manifest,
                entry.adapter.as_deref().map(|adapter| adapter.health()),
            ));
        }
        health.sort_by_key(|entry| (entry.kind, entry.id.clone()));
        health
    }
}

fn project_health(manifest: &PluginManifest, adapter_health: Option<PluginHealth>) -> PluginHealth {
    if !manifest.enabled {
        return PluginHealth {
            id: manifest.id.clone(),
            kind: manifest.kind,
            status: PluginHealthStatus::Disabled,
            capabilities: manifest.capabilities.clone(),
            message: None,
        };
    }
    adapter_health.unwrap_or_else(|| PluginHealth {
        id: manifest.id.clone(),
        kind: manifest.kind,
        status: PluginHealthStatus::Degraded,
        capabilities: manifest.capabilities.clone(),
        message: Some("adapter is unavailable".into()),
    })
}

fn default_true() -> bool {
    true
}

fn manifest(id: &str, kind: PluginKind, adapter: &str) -> PluginManifest {
    PluginManifest {
        id: id.into(),
        kind,
        adapter: adapter.into(),
        enabled: true,
        required: false,
        capabilities: capability_allowlist(kind)
            .iter()
            .take(2)
            .map(|capability| (*capability).into())
            .collect(),
        endpoint: None,
        secret_env: None,
    }
}

fn valid_name(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.chars().count() <= max
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
}

fn valid_env_name(value: &str) -> bool {
    !value.is_empty()
        && value.chars().count() <= 128
        && value
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
}

fn capability_allowlist(kind: PluginKind) -> &'static [&'static str] {
    match kind {
        PluginKind::Memory => &["health", "recall"],
        PluginKind::Wiki => &["health", "search", "sync", "page"],
        PluginKind::CodeGraph => &["health", "query", "index", "symbol", "references", "impact"],
        PluginKind::Skill => &[
            "health",
            "discover",
            "candidate",
            "validate",
            "publish",
            "rollback",
        ],
    }
}

fn supported_adapter(kind: PluginKind, adapter: &str) -> bool {
    matches!(
        (kind, adapter),
        (PluginKind::Memory, ADAPTER_DETERMINISTIC_MEMORY)
            | (PluginKind::Wiki, ADAPTER_DETERMINISTIC_WIKI)
            | (PluginKind::CodeGraph, ADAPTER_DETERMINISTIC_CODEGRAPH)
            | (PluginKind::Skill, ADAPTER_DETERMINISTIC_SKILL)
    )
}

fn build_memory(
    manifest: &PluginManifest,
) -> Result<Arc<dyn MemoryPluginAdapter>, PluginRegistryError> {
    match manifest.adapter.as_str() {
        ADAPTER_DETERMINISTIC_MEMORY => Ok(Arc::new(DeterministicMemory {
            manifest: manifest.clone(),
        })),
        _ => Err(PluginRegistryError::UnsupportedAdapter),
    }
}

fn build_wiki(
    manifest: &PluginManifest,
) -> Result<Arc<dyn WikiPluginAdapter>, PluginRegistryError> {
    match manifest.adapter.as_str() {
        ADAPTER_DETERMINISTIC_WIKI => Ok(Arc::new(DeterministicWiki {
            manifest: manifest.clone(),
        })),
        _ => Err(PluginRegistryError::UnsupportedAdapter),
    }
}

fn build_codegraph(
    manifest: &PluginManifest,
) -> Result<Arc<dyn CodeGraphPluginAdapter>, PluginRegistryError> {
    match manifest.adapter.as_str() {
        ADAPTER_DETERMINISTIC_CODEGRAPH => Ok(Arc::new(DeterministicCodeGraph {
            manifest: manifest.clone(),
        })),
        _ => Err(PluginRegistryError::UnsupportedAdapter),
    }
}

fn build_skill(
    manifest: &PluginManifest,
) -> Result<Arc<dyn SkillPluginAdapter>, PluginRegistryError> {
    match manifest.adapter.as_str() {
        ADAPTER_DETERMINISTIC_SKILL => Ok(Arc::new(DeterministicSkill {
            manifest: manifest.clone(),
        })),
        _ => Err(PluginRegistryError::UnsupportedAdapter),
    }
}

fn fixture_ref(manifest: &PluginManifest, suffix: &str) -> AssetRef {
    AssetRef {
        id: format!("{}-{suffix}", manifest.id),
        kind: manifest.kind,
        scope: AssetScope {
            project_id: "fixture-project".into(),
            folder_id: Some("fixture-folder".into()),
            agent_profile_id: None,
        },
        provider: "deterministic".into(),
        revision: "fixture-v1".into(),
        provenance: AssetProvenance {
            source: format!("fixture/{}", manifest.kind.as_str()),
        },
    }
}

fn fixture_health(manifest: &PluginManifest) -> PluginHealth {
    PluginHealth {
        id: manifest.id.clone(),
        kind: manifest.kind,
        status: PluginHealthStatus::Healthy,
        capabilities: manifest.capabilities.clone(),
        message: None,
    }
}

struct DeterministicMemory {
    manifest: PluginManifest,
}

impl MemoryPluginAdapter for DeterministicMemory {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    fn health(&self) -> PluginHealth {
        fixture_health(&self.manifest)
    }

    fn recall(&self, query: &str, limit: usize) -> Vec<MemoryAsset> {
        if query.is_empty() || limit == 0 {
            return Vec::new();
        }
        vec![MemoryAsset {
            asset: fixture_ref(&self.manifest, "memory"),
            title: "Memory fixture".into(),
            content: query.into(),
        }]
    }
}

struct DeterministicWiki {
    manifest: PluginManifest,
}

impl WikiPluginAdapter for DeterministicWiki {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    fn health(&self) -> PluginHealth {
        fixture_health(&self.manifest)
    }

    fn search(&self, query: &str, limit: usize) -> Vec<WikiAsset> {
        if query.is_empty() || limit == 0 {
            return Vec::new();
        }
        vec![WikiAsset {
            asset: fixture_ref(&self.manifest, "wiki"),
            title: "Wiki fixture".into(),
            content: query.into(),
        }]
    }
}

struct DeterministicCodeGraph {
    manifest: PluginManifest,
}

impl CodeGraphPluginAdapter for DeterministicCodeGraph {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    fn health(&self) -> PluginHealth {
        fixture_health(&self.manifest)
    }

    fn query(&self, query: &str, limit: usize) -> Vec<CodeGraphAsset> {
        if query.is_empty() || limit == 0 {
            return Vec::new();
        }
        vec![CodeGraphAsset {
            asset: fixture_ref(&self.manifest, "codegraph"),
            title: "CodeGraph fixture".into(),
            content: query.into(),
        }]
    }
}

struct DeterministicSkill {
    manifest: PluginManifest,
}

impl SkillPluginAdapter for DeterministicSkill {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    fn health(&self) -> PluginHealth {
        fixture_health(&self.manifest)
    }

    fn discover(&self, query: &str, limit: usize) -> Vec<SkillAsset> {
        if query.is_empty() || limit == 0 {
            return Vec::new();
        }
        vec![SkillAsset {
            asset: fixture_ref(&self.manifest, "skill"),
            title: "Skill fixture".into(),
            content: query.into(),
        }]
    }
}

impl TryFrom<MemoryAsset> for ContextCandidate {
    type Error = PluginContractError;

    fn try_from(asset: MemoryAsset) -> Result<Self, Self::Error> {
        if asset.asset.kind != PluginKind::Memory {
            return Err(PluginContractError::AssetKindMismatch);
        }
        Ok(Self {
            asset: asset.asset,
            title: asset.title,
            content: asset.content,
        })
    }
}

impl TryFrom<WikiAsset> for ContextCandidate {
    type Error = PluginContractError;

    fn try_from(asset: WikiAsset) -> Result<Self, Self::Error> {
        if asset.asset.kind != PluginKind::Wiki {
            return Err(PluginContractError::AssetKindMismatch);
        }
        Ok(Self {
            asset: asset.asset,
            title: asset.title,
            content: asset.content,
        })
    }
}

impl TryFrom<CodeGraphAsset> for ContextCandidate {
    type Error = PluginContractError;

    fn try_from(asset: CodeGraphAsset) -> Result<Self, Self::Error> {
        if asset.asset.kind != PluginKind::CodeGraph {
            return Err(PluginContractError::AssetKindMismatch);
        }
        Ok(Self {
            asset: asset.asset,
            title: asset.title,
            content: asset.content,
        })
    }
}

impl TryFrom<SkillAsset> for ContextCandidate {
    type Error = PluginContractError;

    fn try_from(asset: SkillAsset) -> Result<Self, Self::Error> {
        if asset.asset.kind != PluginKind::Skill {
            return Err(PluginContractError::AssetKindMismatch);
        }
        Ok(Self {
            asset: asset.asset,
            title: asset.title,
            content: asset.content,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_registry_exposes_four_isolated_projections() {
        let registry = PluginRegistry::deterministic();
        let health = registry.health_projection();
        assert_eq!(health.len(), 4);
        assert_eq!(
            health.iter().map(|entry| entry.kind).collect::<Vec<_>>(),
            PluginKind::ALL
        );
        assert!(registry.memory("memory-fixture").is_some());
        assert!(registry.wiki("wiki-fixture").is_some());
        assert!(registry.codegraph("codegraph-fixture").is_some());
        assert!(registry.skill("skill-fixture").is_some());

        let memory = registry.memory("memory-fixture").unwrap().recall("same", 1);
        let wiki = registry.wiki("wiki-fixture").unwrap().search("same", 1);
        let graph = registry
            .codegraph("codegraph-fixture")
            .unwrap()
            .query("same", 1);
        let skill = registry.skill("skill-fixture").unwrap().discover("same", 1);
        assert_eq!(memory[0].asset.kind, PluginKind::Memory);
        assert_eq!(wiki[0].asset.kind, PluginKind::Wiki);
        assert_eq!(graph[0].asset.kind, PluginKind::CodeGraph);
        assert_eq!(skill[0].asset.kind, PluginKind::Skill);
    }

    #[test]
    fn invalid_config_is_rejected_before_construction() {
        let registry = PluginRegistry::production();
        let mut invalid = manifest("invalid", PluginKind::Wiki, "unknown");
        assert_eq!(
            registry.validate_manifest(&invalid),
            Err(PluginRegistryError::UnsupportedAdapter)
        );
        invalid.adapter = ADAPTER_DETERMINISTIC_WIKI.into();
        invalid.endpoint = Some("https://user:secret@example.invalid".into());
        assert_eq!(
            registry.validate_manifest(&invalid),
            Err(PluginRegistryError::InvalidManifest)
        );
        invalid.endpoint = Some("https://example.invalid/search?token=secret".into());
        assert_eq!(
            registry.validate_manifest(&invalid),
            Err(PluginRegistryError::InvalidManifest)
        );
        invalid.endpoint = None;
        invalid.secret_env = Some("not-a-secret-env".into());
        assert_eq!(
            registry.validate_manifest(&invalid),
            Err(PluginRegistryError::InvalidManifest)
        );
        invalid.secret_env = None;
        invalid.capabilities = vec!["memory.capture".into()];
        assert_eq!(
            registry.validate_manifest(&invalid),
            Err(PluginRegistryError::InvalidManifest)
        );
        assert!(PluginRegistry::from_manifests(&[invalid]).is_err());
    }

    #[test]
    fn disabled_manifest_has_no_adapter_and_projects_disabled() {
        let mut disabled = manifest(
            "wiki-disabled",
            PluginKind::Wiki,
            ADAPTER_DETERMINISTIC_WIKI,
        );
        disabled.enabled = false;
        let registry = PluginRegistry::from_manifests(&[disabled]).unwrap();
        assert!(registry.wiki("wiki-disabled").is_none());
        let health = registry.health_projection();
        assert_eq!(health[0].status, PluginHealthStatus::Disabled);
    }

    #[test]
    fn all_kinds_normalize_to_existing_context_items_with_asset_ref() {
        let registry = PluginRegistry::deterministic();
        let candidates = vec![
            ContextCandidate::try_from(
                registry.memory("memory-fixture").unwrap().recall("m", 1)[0].clone(),
            ),
            ContextCandidate::try_from(
                registry.wiki("wiki-fixture").unwrap().search("w", 1)[0].clone(),
            ),
            ContextCandidate::try_from(
                registry
                    .codegraph("codegraph-fixture")
                    .unwrap()
                    .query("g", 1)[0]
                    .clone(),
            ),
            ContextCandidate::try_from(
                registry.skill("skill-fixture").unwrap().discover("s", 1)[0].clone(),
            ),
        ];
        let candidates = candidates
            .into_iter()
            .map(Result::unwrap)
            .collect::<Vec<_>>();
        let items = to_context_items(candidates).unwrap();
        assert_eq!(items.len(), 4);
        for item in items {
            assert!(item.provenance.get("assetRef").is_some());
            assert!(item.provenance.get("vendor").is_none());
        }
    }

    #[test]
    fn duplicate_ids_are_rejected_across_kinds() {
        let manifests = [
            manifest("same-id", PluginKind::Wiki, ADAPTER_DETERMINISTIC_WIKI),
            manifest("same-id", PluginKind::Skill, ADAPTER_DETERMINISTIC_SKILL),
        ];
        assert!(matches!(
            PluginRegistry::from_manifests(&manifests),
            Err(PluginRegistryError::DuplicateId)
        ));
    }

    #[test]
    fn wrong_kind_asset_is_rejected_before_context_mapping() {
        let registry = PluginRegistry::deterministic();
        let mut asset = registry.wiki("wiki-fixture").unwrap().search("w", 1)[0].clone();
        asset.asset.kind = PluginKind::Memory;
        assert_eq!(
            ContextCandidate::try_from(asset),
            Err(PluginContractError::AssetKindMismatch)
        );
    }
}
