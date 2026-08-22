//! Capture outbox delivery worker (BUGRAIL-SPECOS-017 §5).
//!
//! The worker runs independently from the settlement transaction: settle
//! never waits on the network, and capture failure never changes a settled
//! WorkTask or gate outcome. Delivery semantics:
//!
//! - `queued -> sending -> delivered | failed`; retryable failures return to
//!   `queued` with exponential backoff, at most [`MAX_ATTEMPTS`] attempts;
//! - startup recovers `sending` rows (crash mid-flight) and reconciles
//!   settled runs missing a delivery row (crash after settlement);
//! - stable caller message ids plus the `v2.0.0+bugrail.1` upsert contract
//!   make at-least-once retries idempotent upstream;
//! - capture writes only when the Gateway reports the exact patched version
//!   (the writability gate, via the shared health cache); vanilla gateways
//!   fail terminal with `memory.upstreamUnsupported`.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use serde::Deserialize;

use crate::db::entities::memory_capture_delivery as delivery;
use crate::db::service::{folder_service, memory_capture_service};
use crate::memory::{MemoryCaptureBatch, MemoryCaptureMessage, MemoryError, MemoryService};

use super::capture::{reconcile_missing_deliveries, ACTIVITY_KIND};
use super::config::binding_for_folder;
use super::identity;

/// Rows claimed per tick — small on purpose; delivery is serialized.
const BATCH_PER_TICK: u64 = 8;
/// Worker poll interval.
const TICK: Duration = Duration::from_secs(5);

/// Spawn the worker for the process lifetime (server boot path; the desktop
/// boot path calls [`run`] through the Tauri async runtime). Startup
/// recovery and reconciliation run inside [`run`] before the first tick.
pub fn spawn(memory: Arc<MemoryService>) {
    tokio::spawn(run(memory));
}

pub async fn run(memory: Arc<MemoryService>) {
    recover_and_reconcile(&memory).await;
    let mut tick = tokio::time::interval(TICK);
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        tick.tick().await;
        if let Err(e) = deliver_due(&memory).await {
            tracing::warn!("[memory] capture delivery tick failed: {e}");
        }
    }
}

/// Startup recovery (`sending` rows back to `queued`) plus reconciliation of
/// settled runs missing a delivery row.
pub async fn recover_and_reconcile(memory: &MemoryService) {
    let conn = &memory.db().conn;
    match memory_capture_service::recover_sending(conn).await {
        Ok(0) => {}
        Ok(n) => tracing::info!(rows = n, "[memory] capture worker recovered sending rows"),
        Err(e) => tracing::warn!("[memory] capture worker recovery failed: {e}"),
    }
    if let Err(e) = reconcile_missing_deliveries(memory.db()).await {
        tracing::warn!("[memory] capture reconciliation failed: {e}");
    }
}

/// One delivery pass over all due rows. Returns the number of rows that
/// reached `delivered`.
pub async fn deliver_due(memory: &MemoryService) -> Result<u32, String> {
    let conn = &memory.db().conn;
    let rows = memory_capture_service::due(conn, chrono::Utc::now(), BATCH_PER_TICK)
        .await
        .map_err(|e| e.to_string())?;
    let mut delivered = 0u32;
    for row in rows {
        if deliver_row(memory, row).await? {
            delivered += 1;
        }
    }
    Ok(delivered)
}

/// Staged payload shape written by `memory::capture::stage_capture`.
#[derive(Deserialize)]
struct StagedPayload {
    #[serde(default)]
    messages: Vec<StagedMessage>,
}

#[derive(Deserialize)]
struct StagedMessage {
    id: String,
    role: String,
    content: String,
}

/// Deliver one row. Returns true when the row reached `delivered`.
async fn deliver_row(memory: &MemoryService, row: delivery::Model) -> Result<bool, String> {
    let conn = &memory.db().conn;
    let delivery_id = row.id;
    // CAS claim: another pass (or a parallel retry command) cannot deliver
    // the same row twice.
    if !memory_capture_service::mark_sending(conn, delivery_id)
        .await
        .map_err(|e| e.to_string())?
    {
        return Ok(false);
    }
    let attempts = row.attempts + 1;
    let outcome = attempt_delivery(memory, &row).await;
    match outcome {
        Ok(accepted_ids_json) => {
            memory_capture_service::mark_delivered(conn, delivery_id, attempts, &accepted_ids_json)
                .await
                .map_err(|e| e.to_string())?;
            activity(
                memory,
                &row,
                "delivered",
                &format!("delivery {delivery_id} accepted upstream (attempt {attempts})"),
            )
            .await;
            tracing::info!(
                delivery_id,
                provider = %row.provider_id,
                attempt = attempts,
                "[memory] capture delivered"
            );
            Ok(true)
        }
        Err(err) => {
            let status = memory_capture_service::mark_failed(
                conn,
                delivery_id,
                attempts,
                err.class.key(),
                &err.message,
                err.class.retryable(),
            )
            .await
            .map_err(|e| e.to_string())?;
            let message = if status == memory_capture_service::STATUS_QUEUED {
                format!(
                    "{} (attempt {attempts}/{}, retry scheduled)",
                    err.class.key(),
                    memory_capture_service::MAX_ATTEMPTS
                )
            } else {
                format!("{} (attempt {attempts})", err.class.key())
            };
            activity(memory, &row, &status, &message).await;
            tracing::warn!(
                delivery_id,
                provider = %row.provider_id,
                class = err.class.key(),
                attempt = attempts,
                status = %status,
                "[memory] capture delivery failed"
            );
            Ok(false)
        }
    }
}

/// One network attempt: resolve fresh config and identity, rebuild the batch
/// from the staged payload, call the Adapter. Errors carry safe classes and
/// messages only (the Adapter enforces this at the transport edge).
async fn attempt_delivery(
    memory: &MemoryService,
    row: &delivery::Model,
) -> Result<String, MemoryError> {
    use super::MemoryErrorClass;

    let conn = &memory.db().conn;
    let staged = row.payload.as_deref().ok_or_else(|| {
        MemoryError::new(
            MemoryErrorClass::DeliveryNotRetryable,
            "delivery row has no staged payload",
        )
        .with_provider_id(row.provider_id.clone())
    })?;
    let payload: StagedPayload = serde_json::from_str(staged).map_err(|_| {
        MemoryError::new(
            MemoryErrorClass::InvalidResponse,
            "staged capture payload is corrupt",
        )
        .with_provider_id(row.provider_id.clone())
    })?;

    let folder = folder_service::get_folder_by_id(conn, row.folder_id)
        .await
        .ok()
        .flatten()
        .ok_or_else(|| {
            MemoryError::new(
                MemoryErrorClass::ConfigInvalid,
                "folder for this delivery no longer exists",
            )
            .with_provider_id(row.provider_id.clone())
        })?;
    let config = crate::specos_control::load_context(Path::new(&folder.path)).map_err(|e| {
        MemoryError::new(
            MemoryErrorClass::ConfigInvalid,
            format!("context config could not be loaded: {e}"),
        )
        .with_provider_id(row.provider_id.clone())
    })?;
    let provider_config = config
        .providers
        .iter()
        .find(|p| p.id == row.provider_id && p.kind == super::MEMORY_KIND)
        .ok_or_else(|| {
            MemoryError::new(
                MemoryErrorClass::ConfigInvalid,
                "memory provider is no longer configured",
            )
            .with_provider_id(row.provider_id.clone())
        })?
        .clone();
    if !provider_config.enabled || !provider_config.capture_enabled {
        return Err(MemoryError::new(
            MemoryErrorClass::ConfigInvalid,
            "capture is disabled for this provider",
        )
        .with_provider_id(row.provider_id.clone()));
    }

    // Writability gate: the Gateway must report the exact patched pin before
    // capture may write. The shared 30s health cache bounds probe traffic; a
    // vanilla gateway fails terminal with `memory.upstreamUnsupported`.
    let health = memory
        .provider_health(row.folder_id, &provider_config, false)
        .await;
    if health.status != "healthy" {
        let unsupported = health
            .message
            .as_deref()
            .is_some_and(|m| m.starts_with(MemoryErrorClass::UpstreamUnsupported.key()));
        let class = if unsupported {
            MemoryErrorClass::UpstreamUnsupported
        } else {
            // Treat the degraded probe like a transport failure: retryable
            // classes retry, everything else fails terminal.
            health
                .message
                .as_deref()
                .and_then(|m| {
                    [
                        MemoryErrorClass::Unauthorized,
                        MemoryErrorClass::Unavailable,
                        MemoryErrorClass::Timeout,
                        MemoryErrorClass::RateLimited,
                        MemoryErrorClass::InvalidResponse,
                        MemoryErrorClass::ConfigInvalid,
                        MemoryErrorClass::IdentityMissing,
                    ]
                    .iter()
                    .find(|class| m.starts_with(class.key()))
                    .copied()
                })
                .unwrap_or(MemoryErrorClass::Unavailable)
        };
        return Err(
            MemoryError::new(class, "provider health gate did not pass before capture")
                .with_provider_id(row.provider_id.clone()),
        );
    }

    let (resolved, adapter) = memory.adapter_for(&provider_config, None)?;
    if !resolved.can_capture() {
        return Err(MemoryError::new(
            MemoryErrorClass::ConfigInvalid,
            "provider does not declare the memory.capture capability",
        )
        .with_provider_id(row.provider_id.clone()));
    }

    // Identity is derived fresh at delivery time from the durable row ids —
    // never stored on the row, so retries are deterministic.
    let binding = binding_for_folder(Path::new(&folder.path));
    let messages = payload
        .messages
        .into_iter()
        .filter_map(|message| {
            let role = match message.role.as_str() {
                "user" => super::MemoryRole::User,
                "assistant" => super::MemoryRole::Assistant,
                _ => return None,
            };
            Some(MemoryCaptureMessage {
                id: message.id,
                role,
                content: message.content,
            })
        })
        .collect::<Vec<_>>();
    if messages.is_empty() {
        return Err(MemoryError::new(
            MemoryErrorClass::InvalidResponse,
            "staged capture payload has no capturable messages",
        )
        .with_provider_id(row.provider_id.clone()));
    }
    let batch = MemoryCaptureBatch {
        team_id: resolved.team_id.clone(),
        agent_id: resolved.agent_id.clone(),
        user_id: resolved.user_id.clone(),
        session_id: identity::session_id(&binding, row.task_id, row.run_seq),
        task_id: identity::upstream_task_id(&binding, row.task_id),
        messages,
    };
    let receipt = adapter.capture(&batch).await?;
    serde_json::to_string(&receipt.accepted_ids).map_err(|_| {
        MemoryError::new(
            MemoryErrorClass::InvalidResponse,
            "accepted ids could not be serialized",
        )
        .with_provider_id(row.provider_id.clone())
    })
}

/// Context Activity evidence for one delivery state change. Failures to
/// record activity never fail the delivery itself.
async fn activity(memory: &MemoryService, row: &delivery::Model, status: &str, message: &str) {
    if let Err(e) = crate::context::record_activity(
        &memory.db().conn,
        row.folder_id,
        None,
        Some(&row.provider_id),
        ACTIVITY_KIND,
        status,
        Some(message),
    )
    .await
    {
        tracing::warn!(
            delivery_id = row.id,
            "[memory] capture activity record failed: {e}"
        );
    }
}
