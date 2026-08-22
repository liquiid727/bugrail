//! Shared Memory Provider operations for the Context UI
//! (BUGRAIL-SPECOS-017 R06, issue-080).
//!
//! Four command-core functions with equivalent Tauri/Axum wrappers:
//! provider test, delivery list, delivery retry and recall preview. Every
//! returned fact is bounded and redacted (R07): error class keys instead of
//! upstream bodies, hashes instead of payloads, and previews truncated to a
//! short untrusted-text window. Recall preview is a UI probe only — it never
//! creates packages or provenance rows.

use serde::{Deserialize, Serialize};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use sha2::{Digest, Sha256};

use crate::db::entities::memory_capture_delivery as delivery;
use crate::db::error::DbError;
use crate::db::service::memory_capture_service;
use crate::db::AppDatabase;
use crate::memory::{MemoryLayer, MemoryRecallRequest, MemoryService, MEMORY_KIND};
use crate::models::ContextProviderConfig;

/// Upper bound for the delivery list page.
const MAX_DELIVERY_LIMIT: u32 = 100;
/// Recall preview bounds (mirrors the adapter recall limit bound).
const MIN_PREVIEW_LIMIT: u32 = 1;
const MAX_PREVIEW_LIMIT: u32 = 20;
/// Preview text window; remote content beyond this is dropped, not shipped.
const PREVIEW_CHARS: usize = 200;
/// Remote id window shared with Context provenance evidence.
const REMOTE_ID_CHARS: usize = 128;
/// Query window shared with `context::prepare_run`.
const QUERY_CHARS: usize = 2_000;

// ── Redacted result types ───────────────────────────────────────────────────

/// Connection-test result. Carries the health status, the pinned-version
/// gate outcome, latency and an error class key — never an endpoint URL,
/// credential or upstream body.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryProviderTestResult {
    pub provider_id: String,
    /// `healthy` | `degraded` | `blocked` (`blocked` = disabled or
    /// unresolvable identity: no request was issued).
    pub status: String,
    pub version: Option<String>,
    /// True only when the Gateway reported the exact patched pin
    /// (`v2.0.0+bugrail.1`), the capture writability gate.
    pub version_match: bool,
    pub latency_ms: Option<u64>,
    /// Stable error class key (`memory.*`) when status is not healthy.
    pub error_key: Option<String>,
}

/// One capture delivery row for the Context UI list. Payload bodies are
/// never included; upstream accepted ids are reduced to a count.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryDeliveryInfo {
    pub id: i32,
    pub provider_id: String,
    pub folder_id: i32,
    pub task_id: i32,
    pub run_seq: i32,
    pub status: String,
    pub attempts: i32,
    pub retryable: bool,
    /// Number of message ids the patched Gateway accepted (post-delivery).
    pub accepted_count: Option<u32>,
    /// Number of staged source message ids.
    pub source_count: u32,
    pub payload_hash: String,
    pub safe_error_code: Option<String>,
    pub safe_error_message: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub delivered_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// One recall preview hit: bounded metadata plus a truncated untrusted-text
/// preview. No package or provenance is created.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryRecallPreviewHit {
    /// `l1` | `l3`.
    pub layer: String,
    pub remote_id: String,
    pub score: Option<f64>,
    pub content_hash: String,
    /// First ~200 chars of remote content, treated as untrusted text by
    /// the renderer.
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryRecallPreview {
    pub provider_id: String,
    pub query_hash: String,
    pub hits: Vec<MemoryRecallPreviewHit>,
}

// ── Helpers ─────────────────────────────────────────────────────────────────

async fn memory_provider_config(
    db: &AppDatabase,
    folder_id: i32,
    provider_id: &str,
) -> Result<ContextProviderConfig, DbError> {
    let root = crate::specos_control::project_root(&db.conn, folder_id).await?;
    let config = crate::specos_control::load_context(&root)?;
    config
        .providers
        .into_iter()
        .find(|provider| provider.id == provider_id && provider.kind == MEMORY_KIND)
        .ok_or_else(|| {
            DbError::NotFound(format!(
                "memory provider '{provider_id}' in folder {folder_id}"
            ))
        })
}

fn json_array_len(raw: &Option<String>) -> Option<u32> {
    raw.as_ref()
        .and_then(|value| serde_json::from_str::<serde_json::Value>(value).ok())
        .and_then(|value| value.as_array().map(|items| items.len() as u32))
}

fn delivery_info(row: delivery::Model) -> MemoryDeliveryInfo {
    MemoryDeliveryInfo {
        id: row.id,
        provider_id: row.provider_id,
        folder_id: row.folder_id,
        task_id: row.task_id,
        run_seq: row.run_seq,
        status: row.status,
        attempts: row.attempts,
        retryable: row.retryable,
        accepted_count: json_array_len(&row.upstream_accepted_ids),
        source_count: json_array_len(&Some(row.source_message_ids.clone())).unwrap_or_default(),
        payload_hash: row.payload_hash,
        safe_error_code: row.safe_error_code,
        safe_error_message: row.safe_error_message,
        created_at: row.created_at,
        updated_at: row.updated_at,
        delivered_at: row.delivered_at,
    }
}

// ── Command core ────────────────────────────────────────────────────────────

/// Provider connection test: `GET /health` through the pinned Adapter plus
/// the exact-version writability gate. Forces a fresh probe (no shared
/// cache) because the user explicitly asked.
pub async fn memory_provider_test_core(
    db: &AppDatabase,
    memory: &MemoryService,
    folder_id: i32,
    provider_id: &str,
) -> Result<MemoryProviderTestResult, DbError> {
    let provider = memory_provider_config(db, folder_id, provider_id).await?;
    if !provider.enabled {
        return Ok(MemoryProviderTestResult {
            provider_id: provider.id,
            status: "blocked".into(),
            version: None,
            version_match: false,
            latency_ms: None,
            error_key: Some("memory.configInvalid".into()),
        });
    }
    let (_, adapter) = memory.adapter_for(&provider, None).map_err(|error| {
        // Class key only — the (safe) message stays server-side.
        DbError::Validation(error.class.key().to_string())
    })?;
    match adapter.health().await {
        Ok(report) => {
            let version_match = report.writable;
            let status = if version_match { "healthy" } else { "degraded" };
            Ok(MemoryProviderTestResult {
                provider_id: provider.id,
                status: status.into(),
                version: report.version,
                version_match,
                latency_ms: report.latency_ms,
                error_key: (!version_match).then(|| {
                    crate::memory::MemoryErrorClass::UpstreamUnsupported
                        .key()
                        .to_string()
                }),
            })
        }
        Err(error) => Ok(MemoryProviderTestResult {
            provider_id: provider.id,
            status: "degraded".into(),
            version: None,
            version_match: false,
            latency_ms: None,
            error_key: Some(error.class.key().to_string()),
        }),
    }
}

/// Delivery rows for the Context UI, newest first, optionally filtered to
/// one WorkTask. Payload bodies never leave the backend.
pub async fn memory_delivery_list_core(
    db: &AppDatabase,
    folder_id: i32,
    task_id: Option<i32>,
    limit: Option<u32>,
) -> Result<Vec<MemoryDeliveryInfo>, DbError> {
    let limit = limit.unwrap_or(20).clamp(1, MAX_DELIVERY_LIMIT) as u64;
    let mut query = delivery::Entity::find()
        .filter(delivery::Column::FolderId.eq(folder_id))
        .order_by_desc(delivery::Column::Id)
        .limit(limit);
    if let Some(task_id) = task_id {
        query = query.filter(delivery::Column::TaskId.eq(task_id));
    }
    let rows = query.all(&db.conn).await?;
    Ok(rows.into_iter().map(delivery_info).collect())
}

/// Manual retry of a terminal failed (or still-queued) delivery row,
/// reusing the capture service's bounded `requeue_for_retry` semantics:
/// only `failed`/`queued` rows with a staged payload retry, and the attempt
/// counter resets so the worker gets a fresh budget.
pub async fn memory_delivery_retry_core(
    db: &AppDatabase,
    delivery_id: Option<i32>,
    provider_id: Option<&str>,
    task_id: Option<i32>,
    run_seq: Option<i32>,
) -> Result<MemoryDeliveryInfo, DbError> {
    let row = match delivery_id {
        Some(id) => memory_capture_service::get(&db.conn, id).await?,
        None => {
            let (provider_id, task_id, run_seq) = match (provider_id, task_id, run_seq) {
                (Some(provider_id), Some(task_id), Some(run_seq)) => {
                    (provider_id, task_id, run_seq)
                }
                _ => {
                    return Err(DbError::Validation(
                        "retry requires deliveryId or providerId+taskId+runSeq".into(),
                    ))
                }
            };
            memory_capture_service::find_for_run(&db.conn, provider_id, task_id, run_seq).await?
        }
    }
    .ok_or_else(|| DbError::NotFound("memory capture delivery".into()))?;
    if !memory_capture_service::requeue_for_retry(&db.conn, row.id).await? {
        return Err(DbError::Validation(
            crate::memory::MemoryErrorClass::DeliveryNotRetryable
                .key()
                .to_string(),
        ));
    }
    let updated = memory_capture_service::get(&db.conn, row.id)
        .await?
        .ok_or_else(|| DbError::NotFound("memory capture delivery".into()))?;
    crate::context::record_activity(
        &db.conn,
        updated.folder_id,
        None,
        Some(&updated.provider_id),
        "memory.delivery",
        "queued",
        Some("manual retry requested"),
    )
    .await?;
    Ok(delivery_info(updated))
}

/// Recall preview: resolves identity, calls `adapter.recall` directly and
/// returns bounded redacted hit metadata. UI probe only — no package, no
/// provenance, no persistence of remote content.
pub async fn memory_recall_preview_core(
    db: &AppDatabase,
    memory: &MemoryService,
    folder_id: i32,
    provider_id: &str,
    query: &str,
    limit: Option<u32>,
    include_core: Option<bool>,
) -> Result<MemoryRecallPreview, DbError> {
    let provider = memory_provider_config(db, folder_id, provider_id).await?;
    let query: String = query.trim().chars().take(QUERY_CHARS).collect();
    if query.is_empty() {
        return Err(DbError::Validation("query must not be empty".into()));
    }
    let limit = limit
        .unwrap_or(provider.recall_limit)
        .clamp(MIN_PREVIEW_LIMIT, MAX_PREVIEW_LIMIT);
    let (resolved, adapter) = memory
        .adapter_for(&provider, None)
        .map_err(|error| DbError::Validation(error.class.key().to_string()))?;
    let request = MemoryRecallRequest {
        team_id: resolved.team_id.clone(),
        agent_id: resolved.agent_id.clone(),
        user_id: resolved.user_id,
        query: query.clone(),
        limit,
        include_core: include_core.unwrap_or(resolved.include_core),
    };
    let result = adapter
        .recall(&request, resolved.timeout)
        .await
        .map_err(|error| DbError::Validation(error.class.key().to_string()))?;
    let query_hash = format!("{:x}", Sha256::digest(query.as_bytes()));
    // Fixed order shared with package compilation: L3 Core first, then L1
    // by score descending with remote id as the deterministic tie-break.
    let mut hits: Vec<MemoryRecallPreviewHit> = result
        .l3
        .into_iter()
        .chain(result.l1)
        .map(|hit| MemoryRecallPreviewHit {
            layer: match hit.layer {
                MemoryLayer::L1 => "l1".into(),
                MemoryLayer::L3 => "l3".into(),
            },
            remote_id: hit.remote_id.chars().take(REMOTE_ID_CHARS).collect(),
            score: hit.score,
            content_hash: format!("{:x}", Sha256::digest(hit.content.as_bytes())),
            preview: hit.content.chars().take(PREVIEW_CHARS).collect(),
        })
        .collect();
    let l1_start = hits.partition_point(|hit| hit.layer == "l3");
    hits[l1_start..].sort_by(|a, b| {
        b.score
            .unwrap_or(0.0)
            .partial_cmp(&a.score.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.remote_id.cmp(&b.remote_id))
    });
    Ok(MemoryRecallPreview {
        provider_id: provider.id,
        query_hash,
        hits,
    })
}

// ── Tauri command wrappers (desktop only) ───────────────────────────────────

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn specos_memory_provider_test(
    db: tauri::State<'_, AppDatabase>,
    memory: tauri::State<'_, std::sync::Arc<MemoryService>>,
    folder_id: i32,
    provider_id: String,
) -> Result<MemoryProviderTestResult, DbError> {
    memory_provider_test_core(&db, &memory, folder_id, &provider_id).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn specos_memory_delivery_list(
    db: tauri::State<'_, AppDatabase>,
    folder_id: i32,
    task_id: Option<i32>,
    limit: Option<u32>,
) -> Result<Vec<MemoryDeliveryInfo>, DbError> {
    memory_delivery_list_core(&db, folder_id, task_id, limit).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn specos_memory_delivery_retry(
    db: tauri::State<'_, AppDatabase>,
    delivery_id: Option<i32>,
    provider_id: Option<String>,
    task_id: Option<i32>,
    run_seq: Option<i32>,
) -> Result<MemoryDeliveryInfo, DbError> {
    memory_delivery_retry_core(&db, delivery_id, provider_id.as_deref(), task_id, run_seq).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn specos_memory_recall_preview(
    db: tauri::State<'_, AppDatabase>,
    memory: tauri::State<'_, std::sync::Arc<MemoryService>>,
    folder_id: i32,
    provider_id: String,
    query: String,
    limit: Option<u32>,
    include_core: Option<bool>,
) -> Result<MemoryRecallPreview, DbError> {
    memory_recall_preview_core(
        &db,
        &memory,
        folder_id,
        &provider_id,
        &query,
        limit,
        include_core,
    )
    .await
}
