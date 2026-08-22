//! Memory Plugin MVP01 (BUGRAIL-SPECOS-017).
//!
//! BugRail owns capture policy, identity mapping, delivery evidence, recall
//! selection, immutable Context Package injection and UI; the Adapter owns
//! vendor transport and L0-L3 calls. The external interface of this module
//! is the test surface:
//!
//! ```text
//! health(MemoryProviderRef) -> MemoryHealthReport
//! capture(MemoryCaptureBatch) -> MemoryCaptureReceipt
//! recall(MemoryRecallRequest) -> MemoryRecallResult
//! ```
//!
//! Adapter selection is a static allowlist keyed by `adapter`; MVP01 loads no
//! dynamic code and never exposes upstream DTOs to callers. Credentials stay
//! environment references in config and are resolved only inside
//! [`ResolvedMemoryProvider`], which is never serialized or logged.

pub mod capture;
pub mod capture_worker;
pub mod config;
pub mod identity;
mod tencentdb;

#[cfg(any(test, feature = "test-utils"))]
pub mod test_adapter;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::db::AppDatabase;
use crate::models::{ContextProviderConfig, ContextProviderHealth};

pub use config::{
    ResolvedMemoryProvider, ADAPTER_TENCENTDB_V3, CAP_CAPTURE, CAP_RECALL_L1, CAP_RECALL_L3,
    MEMORY_KIND,
};
pub use tencentdb::{EXPECTED_UPSTREAM_VERSION, UPSTREAM_PIN_COMMIT};

// ── Error contract (spec §8) ────────────────────────────────────────────────

/// Stable error classes shared by every Adapter. Errors expose the safe
/// reason, provider id, retryability and trace id only — never response
/// bodies or credentials.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MemoryErrorClass {
    ConfigInvalid,
    IdentityMissing,
    Unauthorized,
    Unavailable,
    Timeout,
    RateLimited,
    InvalidResponse,
    UpstreamUnsupported,
    DeliveryNotRetryable,
    Upstream,
}

impl MemoryErrorClass {
    /// i18n / API error key.
    pub fn key(self) -> &'static str {
        match self {
            MemoryErrorClass::ConfigInvalid => "memory.configInvalid",
            MemoryErrorClass::IdentityMissing => "memory.identityMissing",
            MemoryErrorClass::Unauthorized => "memory.unauthorized",
            MemoryErrorClass::Unavailable => "memory.unavailable",
            MemoryErrorClass::Timeout => "memory.timeout",
            MemoryErrorClass::RateLimited => "memory.rateLimited",
            MemoryErrorClass::InvalidResponse => "memory.invalidResponse",
            MemoryErrorClass::UpstreamUnsupported => "memory.upstreamUnsupported",
            MemoryErrorClass::DeliveryNotRetryable => "memory.deliveryNotRetryable",
            MemoryErrorClass::Upstream => "memory.upstream",
        }
    }

    /// Whether an automatic retry may follow this failure class.
    pub fn retryable(self) -> bool {
        matches!(
            self,
            MemoryErrorClass::Unavailable
                | MemoryErrorClass::Timeout
                | MemoryErrorClass::RateLimited
                | MemoryErrorClass::Upstream
        )
    }
}

#[derive(Debug, Clone)]
pub struct MemoryError {
    pub class: MemoryErrorClass,
    /// Safe, bounded reason text. Must not contain response bodies,
    /// transcript content or credentials.
    pub message: String,
    pub provider_id: String,
    pub trace_id: Option<String>,
}

impl MemoryError {
    pub fn new(class: MemoryErrorClass, message: impl Into<String>) -> Self {
        Self {
            class,
            message: message.into(),
            provider_id: String::new(),
            trace_id: None,
        }
    }

    pub fn with_provider_id(mut self, provider_id: impl Into<String>) -> Self {
        self.provider_id = provider_id.into();
        self
    }

    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.class.key(), self.message)
    }
}

impl std::error::Error for MemoryError {}

// ── Vendor-neutral request/result types ─────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MemoryHealthStatus {
    /// Gateway reachable; `/health` answered. [`MemoryHealthReport::writable`]
    /// separately gates capture on the exact pinned patch version.
    Healthy,
    /// Unreachable, unauthorized, malformed or otherwise unusable.
    Degraded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryHealthReport {
    pub status: MemoryHealthStatus,
    /// Upstream-reported version (e.g. `v2.0.0+bugrail.1`), when present.
    pub version: Option<String>,
    /// Capture may write only when the exact patched pin was recognized.
    pub writable: bool,
    pub error_class: Option<MemoryErrorClass>,
    pub message: Option<String>,
    pub latency_ms: Option<u64>,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MemoryRole {
    User,
    Assistant,
}

/// One filtered, bounded transcript message staged for L0 capture.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryCaptureMessage {
    /// Deterministic caller id (project binding + conversation/message
    /// identity). The patched Gateway upserts by this id.
    pub id: String,
    pub role: MemoryRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryCaptureBatch {
    pub team_id: String,
    pub agent_id: String,
    pub user_id: String,
    pub session_id: String,
    /// Stable upstream task id derived from the WorkTask identity. Recall
    /// does NOT filter by it.
    pub task_id: String,
    pub messages: Vec<MemoryCaptureMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryCaptureReceipt {
    /// Upstream-accepted message ids. Under the patch contract a replay with
    /// identical caller ids returns the same accepted ids.
    pub accepted_ids: Vec<String>,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MemoryLayer {
    /// Atomic semantic search hits.
    L1,
    /// Core (long-term) memory.
    L3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryRecallHit {
    pub remote_id: String,
    pub layer: MemoryLayer,
    /// Present for scored L1 hits; L3 Core has no score.
    pub score: Option<f64>,
    pub content: String,
    pub captured_at: Option<String>,
}

/// Recall request. Contains bounded task title/goal text only — never the
/// compiled prompt or repository contents — and no `task_id` filter, so
/// recall spans WorkTasks inside one project `team_id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryRecallRequest {
    pub team_id: String,
    pub agent_id: String,
    pub user_id: String,
    pub query: String,
    pub limit: u32,
    pub include_core: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryRecallResult {
    pub l1: Vec<MemoryRecallHit>,
    /// Empty when `include_core` is false or Core recall failed soft.
    pub l3: Vec<MemoryRecallHit>,
}

// ── Memory interface ────────────────────────────────────────────────────────

/// The deep Memory interface. Wiki, CodeGraph and Skill Evolution are NOT
/// methods here; MVP01 exposes exactly health / capture / recall (plus the
/// bounded credential probe used by the connection test).
#[async_trait::async_trait]
pub trait MemoryProvider: Send + Sync {
    /// Static adapter key (registry allowlist entry).
    fn adapter_id(&self) -> &str;

    async fn health(&self) -> Result<MemoryHealthReport, MemoryError>;

    async fn capture(
        &self,
        batch: &MemoryCaptureBatch,
    ) -> Result<MemoryCaptureReceipt, MemoryError>;

    /// L1 and (optionally) L3 run in parallel inside `deadline` — one
    /// unified recall bound, never sequential round trips.
    async fn recall(
        &self,
        request: &MemoryRecallRequest,
        deadline: Duration,
    ) -> Result<MemoryRecallResult, MemoryError>;

    /// Connection-test probe: validates credentials and isolation through a
    /// bounded read (`POST /v3/core/read`) whose response body is discarded.
    async fn probe_read(&self, request: &MemoryRecallRequest) -> Result<(), MemoryError>;
}

// ── Static Adapter registry ─────────────────────────────────────────────────

pub type AdapterFactory =
    Arc<dyn Fn(&ResolvedMemoryProvider) -> Arc<dyn MemoryProvider> + Send + Sync>;

/// Static allowlist keyed by `adapter`. MVP01 does not load dynamic code.
#[derive(Clone)]
pub struct AdapterRegistry {
    factories: Arc<HashMap<&'static str, AdapterFactory>>,
}

impl AdapterRegistry {
    pub fn production() -> Self {
        let mut factories: HashMap<&'static str, AdapterFactory> = HashMap::new();
        factories.insert(
            ADAPTER_TENCENTDB_V3,
            Arc::new(|provider| {
                Arc::new(tencentdb::TencentDbMemoryAdapter::new(provider.clone()))
                    as Arc<dyn MemoryProvider>
            }),
        );
        Self {
            factories: Arc::new(factories),
        }
    }

    /// Registry holding a shared deterministic in-memory Adapter. Contract
    /// and command-core tests build this to inspect captured batches and to
    /// seed recall hits without any HTTP transport.
    #[cfg(any(test, feature = "test-utils"))]
    pub fn deterministic(adapter: Arc<test_adapter::DeterministicMemoryAdapter>) -> Self {
        let mut factories: HashMap<&'static str, AdapterFactory> = HashMap::new();
        factories.insert(
            ADAPTER_TENCENTDB_V3,
            Arc::new(move |_| adapter.clone() as Arc<dyn MemoryProvider>),
        );
        Self {
            factories: Arc::new(factories),
        }
    }

    pub fn supported(&self, adapter: &str) -> bool {
        self.factories.contains_key(adapter)
    }

    pub fn build(
        &self,
        provider: &ResolvedMemoryProvider,
    ) -> Result<Arc<dyn MemoryProvider>, MemoryError> {
        let factory = self
            .factories
            .get(provider.adapter.as_str())
            .ok_or_else(|| {
                MemoryError::new(
                    MemoryErrorClass::ConfigInvalid,
                    format!(
                        "adapter '{}' is not in the static allowlist",
                        provider.adapter
                    ),
                )
                .with_provider_id(provider.provider_id.clone())
            })?;
        Ok(factory(provider))
    }
}

// ── Shared Memory service ───────────────────────────────────────────────────

/// Health cache time-to-live. Context compilation and the Context Overview
/// share one bounded probe window; the connection-test command forces a
/// fresh check.
pub const HEALTH_CACHE_TTL: Duration = Duration::from_secs(30);

struct CachedHealth {
    checked_at: Instant,
    health: ContextProviderHealth,
}

/// Process-shared Memory service: Adapter registry + health cache + capture
/// outbox. Shared by desktop, server and the WorkTask engine through
/// `AppState` / `TaskEngine`.
pub struct MemoryService {
    db: AppDatabase,
    registry: AdapterRegistry,
    health_cache: tokio::sync::Mutex<HashMap<(i32, String), CachedHealth>>,
}

impl MemoryService {
    pub fn new(db: AppDatabase) -> Self {
        Self::new_with_registry(db, AdapterRegistry::production())
    }

    pub fn new_with_registry(db: AppDatabase, registry: AdapterRegistry) -> Self {
        Self {
            db,
            registry,
            health_cache: tokio::sync::Mutex::new(HashMap::new()),
        }
    }

    pub fn db(&self) -> &AppDatabase {
        &self.db
    }

    pub fn registry(&self) -> &AdapterRegistry {
        &self.registry
    }

    /// Resolve config + identity and build the Adapter.
    /// `agent_profile_id` selects the exact `agent_id_map` entry; the
    /// fallback is `default_agent_id`.
    pub fn adapter_for(
        &self,
        provider: &ContextProviderConfig,
        agent_profile_id: Option<&str>,
    ) -> Result<(ResolvedMemoryProvider, Arc<dyn MemoryProvider>), MemoryError> {
        let resolved = config::resolve_memory_provider(provider, agent_profile_id)?;
        let adapter = self.registry.build(&resolved)?;
        Ok((resolved, adapter))
    }

    /// Provider health with the shared 30s TTL cache. Memory providers are
    /// probed through the Adapter (`GET /health` on the Memory Adapter);
    /// non-memory providers are not this service's concern.
    pub async fn provider_health(
        &self,
        folder_id: i32,
        provider: &ContextProviderConfig,
        force: bool,
    ) -> ContextProviderHealth {
        let key = (folder_id, provider.id.clone());
        if !force {
            let cache = self.health_cache.lock().await;
            if let Some(entry) = cache.get(&key) {
                if entry.checked_at.elapsed() < HEALTH_CACHE_TTL {
                    return entry.health.clone();
                }
            }
        }
        let health = self.check_provider_now(provider).await;
        let mut cache = self.health_cache.lock().await;
        cache.insert(
            key,
            CachedHealth {
                checked_at: Instant::now(),
                health: health.clone(),
            },
        );
        health
    }

    async fn check_provider_now(&self, provider: &ContextProviderConfig) -> ContextProviderHealth {
        let checked_at = chrono::Utc::now();
        let skeleton = |status: &str, message: Option<String>| ContextProviderHealth {
            id: provider.id.clone(),
            kind: provider.kind.clone(),
            status: status.into(),
            message,
            checked_at,
        };
        if !provider.enabled {
            return skeleton("disabled", None);
        }
        // Identity resolution first: an incomplete identity must never
        // issue a request (spec AC01). The resolved values (which include
        // the secret) are dropped here — only the Adapter needs them, and
        // no resolved value may enter a client-visible health fact.
        let (_, adapter) = match self.adapter_for(provider, None) {
            Ok(pair) => pair,
            Err(err) => {
                return skeleton("degraded", Some(err.class.key().to_string()));
            }
        };
        match adapter.health().await {
            Ok(report) => match report.status {
                MemoryHealthStatus::Healthy if report.writable => skeleton("healthy", None),
                MemoryHealthStatus::Healthy => skeleton(
                    "degraded",
                    Some(format!(
                        "{} (version {})",
                        MemoryErrorClass::UpstreamUnsupported.key(),
                        report.version.as_deref().unwrap_or("unknown")
                    )),
                ),
                MemoryHealthStatus::Degraded => skeleton(
                    "degraded",
                    Some(
                        report
                            .error_class
                            .map(|class| class.key().to_string())
                            .unwrap_or_else(|| MemoryErrorClass::Unavailable.key().to_string()),
                    ),
                ),
            },
            Err(err) => skeleton("degraded", Some(err.class.key().to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_keys_and_retryability_are_stable() {
        assert_eq!(
            MemoryErrorClass::ConfigInvalid.key(),
            "memory.configInvalid"
        );
        assert_eq!(
            MemoryErrorClass::IdentityMissing.key(),
            "memory.identityMissing"
        );
        assert_eq!(MemoryErrorClass::Unauthorized.key(), "memory.unauthorized");
        assert_eq!(MemoryErrorClass::Unavailable.key(), "memory.unavailable");
        assert_eq!(MemoryErrorClass::Timeout.key(), "memory.timeout");
        assert_eq!(MemoryErrorClass::RateLimited.key(), "memory.rateLimited");
        assert_eq!(
            MemoryErrorClass::InvalidResponse.key(),
            "memory.invalidResponse"
        );
        assert_eq!(
            MemoryErrorClass::UpstreamUnsupported.key(),
            "memory.upstreamUnsupported"
        );
        assert_eq!(
            MemoryErrorClass::DeliveryNotRetryable.key(),
            "memory.deliveryNotRetryable"
        );

        assert!(MemoryErrorClass::Unavailable.retryable());
        assert!(MemoryErrorClass::Timeout.retryable());
        assert!(MemoryErrorClass::RateLimited.retryable());
        assert!(MemoryErrorClass::Upstream.retryable());
        assert!(!MemoryErrorClass::Unauthorized.retryable());
        assert!(!MemoryErrorClass::ConfigInvalid.retryable());
        assert!(!MemoryErrorClass::IdentityMissing.retryable());
        assert!(!MemoryErrorClass::InvalidResponse.retryable());
        assert!(!MemoryErrorClass::UpstreamUnsupported.retryable());
    }

    #[test]
    fn production_registry_only_knows_the_pinned_adapter() {
        let registry = AdapterRegistry::production();
        assert!(registry.supported(ADAPTER_TENCENTDB_V3));
        assert!(!registry.supported("latest"));
        assert!(!registry.supported(""));
    }
}
