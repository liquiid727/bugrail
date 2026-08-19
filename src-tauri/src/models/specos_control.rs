use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelProfile {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub provider_ref: Option<String>,
    pub model: String,
    #[serde(default)]
    pub reasoning: Option<String>,
    #[serde(default)]
    pub fallback_profile_ids: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentProfile {
    pub id: String,
    pub name: String,
    pub runtime_adapter: String,
    #[serde(default)]
    pub model_profile_id: Option<String>,
    #[serde(default)]
    pub mode_id: Option<String>,
    #[serde(default)]
    pub reasoning: Option<String>,
    #[serde(default)]
    pub context_loadout_id: Option<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub rules: Vec<String>,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub config_values: BTreeMap<String, String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCatalog {
    #[serde(default = "default_version")]
    pub version: i32,
    #[serde(default)]
    pub default_agent_profile_id: Option<String>,
    #[serde(default)]
    pub model_profiles: Vec<ModelProfile>,
    #[serde(default)]
    pub agent_profiles: Vec<AgentProfile>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub validation_errors: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamDefinition {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub member_profile_ids: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowNodeDefinition {
    pub id: String,
    pub title: String,
    pub prompt: String,
    pub agent_profile_id: String,
    #[serde(default)]
    pub model_profile_id: Option<String>,
    #[serde(default)]
    pub context_loadout_id: Option<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamWorkflowDefinition {
    pub id: String,
    pub name: String,
    #[serde(default = "default_version")]
    pub version: i32,
    pub team_id: String,
    #[serde(default = "default_concurrency")]
    pub max_concurrent: i32,
    #[serde(default)]
    pub nodes: Vec<WorkflowNodeDefinition>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamCatalog {
    #[serde(default = "default_version")]
    pub version: i32,
    #[serde(default)]
    pub teams: Vec<TeamDefinition>,
    #[serde(default)]
    pub workflows: Vec<TeamWorkflowDefinition>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub validation_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextProviderConfig {
    pub id: String,
    pub kind: String,
    /// Adapter implementation for adapter-backed kinds. For
    /// `kind: code-intelligence` this must be `codebase-memory-mcp`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter: Option<String>,
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub secret_env: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub capabilities: Vec<String>,
    // ── Memory Provider settings (BUGRAIL-SPECOS-017, `kind = "memory"`) ──
    // Every field below is ignored for non-memory kinds, so legacy provider
    // documents deserialize unchanged. `adapter` above is shared with the
    // code-intelligence provider and selects `tencentdb-agent-memory-v3` here.
    /// Name of the environment variable holding the upstream service id
    /// (`x-tdai-service-id`). The value itself is resolved in the backend at
    /// call time only; it is never persisted or returned to clients.
    #[serde(default)]
    pub service_id_env: Option<String>,
    /// Project-specific upstream isolation space. Local folder database IDs
    /// are never used as cross-install identity.
    #[serde(default)]
    pub team_id: Option<String>,
    /// Name of the environment variable holding the upstream user id.
    #[serde(default)]
    pub user_id_env: Option<String>,
    /// Upstream Agent id used when `agent_id_map` has no exact match.
    #[serde(default)]
    pub default_agent_id: Option<String>,
    /// Exact AgentProfile id → upstream Agent id mapping.
    #[serde(default)]
    pub agent_id_map: BTreeMap<String, String>,
    #[serde(default = "default_true")]
    pub capture_enabled: bool,
    #[serde(default = "default_true")]
    pub recall_enabled: bool,
    /// L1 recall hit limit, bounded 1..=20.
    #[serde(default = "default_recall_limit")]
    pub recall_limit: u32,
    /// Also recall L3 Core memory (`POST /v3/core/read`).
    #[serde(default)]
    pub include_core: bool,
    /// Per-request timeout in milliseconds, bounded 500..=30000.
    #[serde(default = "default_memory_timeout_ms")]
    pub timeout_ms: u64,
    /// Capture cap per message in bytes (default 8 KiB). A message over the
    /// cap is excluded with an explicit reason, never truncated.
    #[serde(default = "default_max_capture_message_bytes")]
    pub max_capture_message_bytes: usize,
    /// Capture cap per batch in bytes (default 256 KiB).
    #[serde(default = "default_max_capture_batch_bytes")]
    pub max_capture_batch_bytes: usize,
}

/// Manual `Default` mirroring the serde defaults so struct-literal
/// construction with `..Default::default()` matches what a deserialized
/// config yields (bounded memory fields get their in-range defaults, not
/// zero).
impl Default for ContextProviderConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            kind: String::new(),
            endpoint: None,
            secret_env: None,
            enabled: true,
            required: false,
            capabilities: Vec::new(),
            adapter: None,
            service_id_env: None,
            team_id: None,
            user_id_env: None,
            default_agent_id: None,
            agent_id_map: BTreeMap::new(),
            capture_enabled: true,
            recall_enabled: true,
            recall_limit: default_recall_limit(),
            include_core: false,
            timeout_ms: default_memory_timeout_ms(),
            max_capture_message_bytes: default_max_capture_message_bytes(),
            max_capture_batch_bytes: default_max_capture_batch_bytes(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextSourceConfig {
    pub path: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default = "default_source_kind")]
    pub kind: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextLoadout {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub sources: Vec<ContextSourceConfig>,
    #[serde(default)]
    pub provider_ids: Vec<String>,
    #[serde(default = "default_max_items")]
    pub max_items: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextConfig {
    #[serde(default = "default_version")]
    pub version: i32,
    #[serde(default = "default_loadout_id")]
    pub default_loadout_id: String,
    #[serde(default)]
    pub providers: Vec<ContextProviderConfig>,
    #[serde(default)]
    pub loadouts: Vec<ContextLoadout>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub validation_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedAgentRuntime {
    pub agent_profile_id: Option<String>,
    pub model_profile_id: Option<String>,
    pub agent_type: String,
    pub model: Option<String>,
    pub mode_id: Option<String>,
    pub reasoning: Option<String>,
    pub context_loadout_id: Option<String>,
    pub config_values: BTreeMap<String, String>,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkTaskRunInfo {
    pub task_id: i32,
    pub run_seq: i32,
    pub status: String,
    pub agent_profile_id: Option<String>,
    pub model_profile_id: Option<String>,
    pub agent_type: Option<String>,
    pub model: Option<String>,
    pub mode_id: Option<String>,
    pub reasoning: Option<String>,
    pub resolution: Option<serde_json::Value>,
    pub conversation_id: Option<i32>,
    pub worktree_folder_id: Option<i32>,
    pub context_package_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkTaskDependencyInfo {
    pub parent_task_id: i32,
    pub child_task_id: i32,
    pub kind: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkTaskHandoffDraft {
    pub summary: String,
    #[serde(default)]
    pub artifacts: Vec<String>,
    #[serde(default)]
    pub risks: Vec<String>,
    #[serde(default)]
    pub open_questions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkTaskHandoffInfo {
    pub task_id: i32,
    pub run_seq: i32,
    pub summary: String,
    pub artifacts: Vec<String>,
    pub risks: Vec<String>,
    pub open_questions: Vec<String>,
    pub source_branch: Option<String>,
    pub source_head: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationSnapshot {
    pub sources: Vec<IntegrationSourceCapture>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationSourceCapture {
    pub task_id: i32,
    pub run_seq: i32,
    pub branch: String,
    pub head: String,
    pub merge_order: i32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationSourceInfo {
    pub task_id: i32,
    pub title: String,
    pub status: String,
    pub run_seq: i32,
    pub branch: Option<String>,
    pub current_head: Option<String>,
    pub captured_head: Option<String>,
    pub captured_run_seq: Option<i32>,
    pub has_handoff: bool,
    pub merge_order: i32,
    pub stale: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationPlan {
    pub task_id: i32,
    pub status: String,
    pub sources: Vec<IntegrationSourceInfo>,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamRunNodeInfo {
    pub node_id: String,
    pub task_id: i32,
    pub title: String,
    pub status: String,
    pub run_seq: i32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamRunInfo {
    pub id: String,
    pub folder_id: i32,
    pub team_id: String,
    pub workflow_id: String,
    pub workflow_version: i32,
    pub control_state: String,
    pub status: String,
    pub definition_hash: String,
    pub nodes: Vec<TeamRunNodeInfo>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextItemInfo {
    pub id: String,
    pub ordinal: i32,
    pub kind: String,
    pub source: String,
    pub title: String,
    pub content: String,
    pub content_hash: String,
    pub required: bool,
    pub provenance: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextPackageInfo {
    pub id: String,
    pub task_id: i32,
    pub run_seq: i32,
    pub loadout_id: String,
    pub status: String,
    pub content_hash: String,
    pub estimated_tokens: i32,
    pub total_bytes: i32,
    pub provider_status: serde_json::Value,
    pub items: Vec<ContextItemInfo>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextActivityInfo {
    pub id: i32,
    pub folder_id: i32,
    pub package_id: Option<String>,
    pub provider_id: Option<String>,
    pub kind: String,
    pub status: String,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextOverview {
    pub config: ContextConfig,
    pub provider_health: Vec<ContextProviderHealth>,
    pub packages: Vec<ContextPackageInfo>,
    pub activity: Vec<ContextActivityInfo>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextProviderHealth {
    pub id: String,
    pub kind: String,
    pub status: String,
    pub message: Option<String>,
    pub checked_at: DateTime<Utc>,
}

fn default_true() -> bool {
    true
}
fn default_version() -> i32 {
    1
}
fn default_concurrency() -> i32 {
    2
}
fn default_loadout_id() -> String {
    "default".to_string()
}
fn default_source_kind() -> String {
    "project".to_string()
}
fn default_max_items() -> usize {
    64
}
fn default_max_bytes() -> usize {
    512 * 1024
}
fn default_max_tokens() -> usize {
    32_000
}
fn default_recall_limit() -> u32 {
    5
}
fn default_memory_timeout_ms() -> u64 {
    5_000
}
fn default_max_capture_message_bytes() -> usize {
    8 * 1024
}
fn default_max_capture_batch_bytes() -> usize {
    256 * 1024
}
