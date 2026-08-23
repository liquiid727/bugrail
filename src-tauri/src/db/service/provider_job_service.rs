//! Durable repository for asynchronous Context-plugin provider work.
//!
//! This is a facts-only repository, not another task scheduler: it stores a
//! safe request hash, provider idempotency key, bounded lease/attempt state
//! and redacted diagnostics. Provider adapters remain responsible for using
//! the idempotency key when they perform external mutation. Existing refresh
//! events are hints; callers can always reconstruct the projection here.

use chrono::{DateTime, Duration, Utc};
use sea_orm::{
    sea_query::{Expr, OnConflict},
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
    TransactionTrait,
};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

use crate::db::entities::{provider_job, provider_job_attempt};

pub const STATUS_QUEUED: &str = "queued";
pub const STATUS_RUNNING: &str = "running";
pub const STATUS_SUCCEEDED: &str = "succeeded";
pub const STATUS_FAILED: &str = "failed";

pub const ATTEMPT_RUNNING: &str = "running";
pub const ATTEMPT_SUCCEEDED: &str = "succeeded";
pub const ATTEMPT_RETRYABLE_FAILED: &str = "retryable_failed";
pub const ATTEMPT_FAILED: &str = "failed";
pub const ATTEMPT_INTERRUPTED: &str = "interrupted";

pub const MAX_ATTEMPTS: i32 = 5;
const MAX_CLAIMS_PER_BATCH: u64 = 32;
const BUSY_RETRY_LIMIT: u32 = 32;
const BUSY_RETRY_BASE_MILLIS: u64 = 5;
const MAX_LEASE_SECONDS: i64 = 60 * 60;
const MAX_COMPONENT_LENGTH: usize = 128;
const MAX_ERROR_CODE_LENGTH: usize = 64;
const MAX_ERROR_MESSAGE_LENGTH: usize = 500;

#[derive(Debug, Error)]
pub enum ProviderJobError {
    #[error("database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("invalid provider job: {0}")]
    Validation(String),
    #[error("provider job idempotency conflict")]
    IdempotencyConflict,
    #[error("provider job lease is no longer valid")]
    LeaseLost,
    #[error("provider job not found")]
    NotFound,
}

/// No request body is accepted by this repository. request_hash is the
/// stable digest of provider-owned input and is used only for conflict checks.
#[derive(Clone, Debug)]
pub struct ProviderJobSpec {
    pub provider_kind: String,
    pub provider_id: String,
    pub operation: String,
    pub idempotency_key: String,
    pub request_hash: String,
    pub max_attempts: i32,
}

#[derive(Clone, Debug)]
pub struct ClaimedProviderJob {
    pub job: provider_job::Model,
    /// Raw bearer token kept only by the worker; the database stores its hash.
    pub lease_token: String,
}

enum ClaimOutcome {
    NoDueJob,
    LostRace,
    Claimed(Box<ClaimedProviderJob>),
}

fn validate_component(name: &str, value: &str) -> Result<(), ProviderJobError> {
    let length = value.chars().count();
    if length == 0 || length > MAX_COMPONENT_LENGTH {
        return Err(ProviderJobError::Validation(format!(
            "{name} must contain 1-{MAX_COMPONENT_LENGTH} characters"
        )));
    }
    if value.chars().any(char::is_control) {
        return Err(ProviderJobError::Validation(format!(
            "{name} contains a control character"
        )));
    }
    Ok(())
}

fn validate_spec(spec: &ProviderJobSpec) -> Result<(), ProviderJobError> {
    validate_component("provider kind", &spec.provider_kind)?;
    validate_component("provider id", &spec.provider_id)?;
    validate_component("operation", &spec.operation)?;
    validate_component("idempotency key", &spec.idempotency_key)?;
    if spec.request_hash.len() != 64
        || !spec
            .request_hash
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(ProviderJobError::Validation(
            "request hash must be a 64-character hex digest".into(),
        ));
    }
    if !(1..=MAX_ATTEMPTS).contains(&spec.max_attempts) {
        return Err(ProviderJobError::Validation(format!(
            "max attempts must be between 1 and {MAX_ATTEMPTS}"
        )));
    }
    Ok(())
}

fn validate_lease(lease_for: Duration) -> Result<(), ProviderJobError> {
    if lease_for <= Duration::zero() || lease_for > Duration::seconds(MAX_LEASE_SECONDS) {
        return Err(ProviderJobError::Validation(format!(
            "lease must be between 1 second and {MAX_LEASE_SECONDS} seconds"
        )));
    }
    Ok(())
}

fn lease_hash(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}

fn safe_error_code(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .filter(|ch| !ch.is_control())
        .take(MAX_ERROR_CODE_LENGTH)
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
        "provider_error".to_string()
    } else {
        sanitized
    }
}

fn safe_error_message(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .filter(|ch| !ch.is_control())
        .take(MAX_ERROR_MESSAGE_LENGTH)
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
        "provider operation failed".to_string()
    } else {
        sanitized
    }
}

fn retry_delay(attempt_count: i32) -> Duration {
    let shift = attempt_count.saturating_sub(1).clamp(0, 5) as u32;
    Duration::seconds(1_i64 << shift)
}

fn is_busy_error(error: &ProviderJobError) -> bool {
    matches!(
        error,
        ProviderJobError::Database(db_error)
            if db_error.to_string().contains("database is locked")
    )
}

async fn recover_expired_with_busy_retry(
    conn: &DatabaseConnection,
    now: DateTime<Utc>,
) -> Result<u64, ProviderJobError> {
    for retry in 0..=BUSY_RETRY_LIMIT {
        match recover_expired(conn, now).await {
            Ok(recovered) => return Ok(recovered),
            Err(error) if is_busy_error(&error) && retry < BUSY_RETRY_LIMIT => {
                tokio::time::sleep(std::time::Duration::from_millis(
                    BUSY_RETRY_BASE_MILLIS * u64::from(retry + 1),
                ))
                .await;
            }
            Err(error) => return Err(error),
        }
    }
    unreachable!("busy retry loop returns or errors")
}

/// Submit one idempotent job. A duplicate key with the same immutable facts
/// returns the original row; a duplicate key with different facts is rejected.
pub async fn submit(
    conn: &DatabaseConnection,
    spec: ProviderJobSpec,
    now: DateTime<Utc>,
) -> Result<provider_job::Model, ProviderJobError> {
    validate_spec(&spec)?;
    for retry in 0..=BUSY_RETRY_LIMIT {
        match submit_once(conn, &spec, now).await {
            Ok(row) => return Ok(row),
            Err(error) if is_busy_error(&error) && retry < BUSY_RETRY_LIMIT => {
                tokio::time::sleep(std::time::Duration::from_millis(
                    BUSY_RETRY_BASE_MILLIS * u64::from(retry + 1),
                ))
                .await;
            }
            Err(error) => return Err(error),
        }
    }
    unreachable!("busy retry loop returns or errors")
}

async fn submit_once(
    conn: &DatabaseConnection,
    spec: &ProviderJobSpec,
    now: DateTime<Utc>,
) -> Result<provider_job::Model, ProviderJobError> {
    let model = provider_job::ActiveModel {
        provider_kind: Set(spec.provider_kind.clone()),
        provider_id: Set(spec.provider_id.clone()),
        operation: Set(spec.operation.clone()),
        idempotency_key: Set(spec.idempotency_key.clone()),
        request_hash: Set(spec.request_hash.clone()),
        status: Set(STATUS_QUEUED.to_string()),
        attempt_count: Set(0),
        max_attempts: Set(spec.max_attempts),
        next_run_at: Set(now),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    let insert = provider_job::Entity::insert(model)
        .on_conflict(
            OnConflict::columns([
                provider_job::Column::ProviderKind,
                provider_job::Column::ProviderId,
                provider_job::Column::IdempotencyKey,
            ])
            .do_nothing()
            .to_owned(),
        )
        .exec(conn)
        .await;
    if let Err(error) = insert {
        if !matches!(error, sea_orm::DbErr::RecordNotInserted) {
            return Err(ProviderJobError::Database(error));
        }
    }
    let row = provider_job::Entity::find()
        .filter(provider_job::Column::ProviderKind.eq(spec.provider_kind.as_str()))
        .filter(provider_job::Column::ProviderId.eq(spec.provider_id.as_str()))
        .filter(provider_job::Column::IdempotencyKey.eq(spec.idempotency_key.as_str()))
        .one(conn)
        .await?
        .ok_or(ProviderJobError::NotFound)?;
    if row.operation != spec.operation
        || row.request_hash != spec.request_hash
        || row.max_attempts != spec.max_attempts
    {
        return Err(ProviderJobError::IdempotencyConflict);
    }
    Ok(row)
}

pub async fn get(
    conn: &DatabaseConnection,
    id: i32,
) -> Result<Option<provider_job::Model>, ProviderJobError> {
    Ok(provider_job::Entity::find_by_id(id).one(conn).await?)
}

/// Reconcile expired leases after startup or before a worker claims work.
pub async fn recover_expired(
    conn: &DatabaseConnection,
    now: DateTime<Utc>,
) -> Result<u64, ProviderJobError> {
    let expired = provider_job::Entity::find()
        .filter(provider_job::Column::Status.eq(STATUS_RUNNING))
        .filter(provider_job::Column::LeaseUntil.lte(now))
        .all(conn)
        .await?;
    let mut recovered = 0;
    for candidate in expired {
        let txn = conn.begin().await?;
        let Some(row) = provider_job::Entity::find_by_id(candidate.id)
            .one(&txn)
            .await?
        else {
            txn.rollback().await?;
            continue;
        };
        if row.status != STATUS_RUNNING || row.lease_until.is_none_or(|until| until > now) {
            txn.rollback().await?;
            continue;
        }
        let terminal = row.attempt_count >= row.max_attempts;
        let status = if terminal {
            STATUS_FAILED
        } else {
            STATUS_QUEUED
        };
        let update = provider_job::Entity::update_many()
            .filter(provider_job::Column::Id.eq(row.id))
            .filter(provider_job::Column::Status.eq(STATUS_RUNNING))
            .filter(provider_job::Column::LeaseUntil.lte(now))
            .col_expr(provider_job::Column::Status, Expr::value(status))
            .col_expr(provider_job::Column::NextRunAt, Expr::value(now))
            .col_expr(
                provider_job::Column::LeaseTokenHash,
                Expr::value(None::<String>),
            )
            .col_expr(
                provider_job::Column::LeaseUntil,
                Expr::value(None::<DateTime<Utc>>),
            )
            .col_expr(
                provider_job::Column::LastErrorCode,
                Expr::value(Some("lease_expired".to_string())),
            )
            .col_expr(
                provider_job::Column::LastErrorMessage,
                Expr::value(Some(
                    "worker lease expired during restart recovery".to_string(),
                )),
            )
            .col_expr(
                provider_job::Column::CompletedAt,
                Expr::value(terminal.then_some(now)),
            )
            .col_expr(provider_job::Column::UpdatedAt, Expr::value(now))
            .exec(&txn)
            .await?;
        if update.rows_affected != 1 {
            txn.rollback().await?;
            continue;
        }
        provider_job_attempt::Entity::update_many()
            .filter(provider_job_attempt::Column::JobId.eq(row.id))
            .filter(provider_job_attempt::Column::AttemptNo.eq(row.attempt_count))
            .filter(provider_job_attempt::Column::Status.eq(ATTEMPT_RUNNING))
            .col_expr(
                provider_job_attempt::Column::Status,
                Expr::value(if terminal {
                    ATTEMPT_FAILED
                } else {
                    ATTEMPT_INTERRUPTED
                }),
            )
            .col_expr(
                provider_job_attempt::Column::FinishedAt,
                Expr::value(Some(now)),
            )
            .col_expr(
                provider_job_attempt::Column::ErrorCode,
                Expr::value(Some("lease_expired".to_string())),
            )
            .col_expr(
                provider_job_attempt::Column::ErrorMessage,
                Expr::value(Some(
                    "worker lease expired during restart recovery".to_string(),
                )),
            )
            .exec(&txn)
            .await?;
        txn.commit().await?;
        recovered += 1;
    }
    Ok(recovered)
}

/// Claim up to limit due jobs. Each claim and its attempt row are committed
/// together, so two workers cannot own the same lease or attempt number.
pub async fn claim_due(
    conn: &DatabaseConnection,
    now: DateTime<Utc>,
    lease_for: Duration,
    limit: u64,
) -> Result<Vec<ClaimedProviderJob>, ProviderJobError> {
    validate_lease(lease_for)?;
    if limit == 0 {
        return Ok(Vec::new());
    }
    recover_expired_with_busy_retry(conn, now).await?;
    let mut claimed = Vec::new();
    let target = limit.min(MAX_CLAIMS_PER_BATCH);
    while claimed.len() < target as usize {
        match claim_one_with_busy_retry(conn, now, lease_for).await? {
            ClaimOutcome::NoDueJob => break,
            ClaimOutcome::LostRace => continue,
            ClaimOutcome::Claimed(job) => claimed.push(*job),
        }
    }
    Ok(claimed)
}

async fn claim_one_with_busy_retry(
    conn: &DatabaseConnection,
    now: DateTime<Utc>,
    lease_for: Duration,
) -> Result<ClaimOutcome, ProviderJobError> {
    for retry in 0..=BUSY_RETRY_LIMIT {
        match claim_one(conn, now, lease_for).await {
            Ok(outcome) => return Ok(outcome),
            Err(error) if is_busy_error(&error) && retry < BUSY_RETRY_LIMIT => {
                tokio::time::sleep(std::time::Duration::from_millis(
                    BUSY_RETRY_BASE_MILLIS * u64::from(retry + 1),
                ))
                .await;
            }
            Err(error) => return Err(error),
        }
    }
    unreachable!("busy retry loop returns or errors")
}

async fn claim_one(
    conn: &DatabaseConnection,
    now: DateTime<Utc>,
    lease_for: Duration,
) -> Result<ClaimOutcome, ProviderJobError> {
    let txn = conn.begin().await?;
    let Some(row) = provider_job::Entity::find()
        .filter(provider_job::Column::Status.eq(STATUS_QUEUED))
        .filter(provider_job::Column::NextRunAt.lte(now))
        .filter(provider_job::Column::AttemptCount.lt(MAX_ATTEMPTS))
        .order_by_asc(provider_job::Column::NextRunAt)
        .order_by_asc(provider_job::Column::Id)
        .limit(1)
        .one(&txn)
        .await?
    else {
        txn.rollback().await?;
        return Ok(ClaimOutcome::NoDueJob);
    };
    let token = Uuid::new_v4().to_string();
    let token_hash = lease_hash(&token);
    let lease_until = now + lease_for;
    let attempt_no = row.attempt_count + 1;
    let update = provider_job::Entity::update_many()
        .filter(provider_job::Column::Id.eq(row.id))
        .filter(provider_job::Column::Status.eq(STATUS_QUEUED))
        .filter(provider_job::Column::AttemptCount.lt(MAX_ATTEMPTS))
        .col_expr(provider_job::Column::Status, Expr::value(STATUS_RUNNING))
        .col_expr(
            provider_job::Column::AttemptCount,
            Expr::col(provider_job::Column::AttemptCount).add(1),
        )
        .col_expr(
            provider_job::Column::LeaseTokenHash,
            Expr::value(Some(token_hash.clone())),
        )
        .col_expr(
            provider_job::Column::LeaseUntil,
            Expr::value(Some(lease_until)),
        )
        .col_expr(provider_job::Column::UpdatedAt, Expr::value(now))
        .exec(&txn)
        .await?;
    if update.rows_affected != 1 {
        txn.rollback().await?;
        return Ok(ClaimOutcome::LostRace);
    }
    provider_job_attempt::Entity::insert(provider_job_attempt::ActiveModel {
        job_id: Set(row.id),
        attempt_no: Set(attempt_no),
        lease_token_hash: Set(token_hash.clone()),
        status: Set(ATTEMPT_RUNNING.to_string()),
        started_at: Set(now),
        ..Default::default()
    })
    .exec(&txn)
    .await?;
    txn.commit().await?;
    let mut claimed_row = row;
    claimed_row.status = STATUS_RUNNING.to_string();
    claimed_row.attempt_count = attempt_no;
    claimed_row.lease_token_hash = Some(token_hash);
    claimed_row.lease_until = Some(lease_until);
    claimed_row.updated_at = now;
    Ok(ClaimOutcome::Claimed(Box::new(ClaimedProviderJob {
        job: claimed_row,
        lease_token: token,
    })))
}

pub async fn mark_succeeded(
    conn: &DatabaseConnection,
    id: i32,
    lease_token: &str,
    now: DateTime<Utc>,
) -> Result<(), ProviderJobError> {
    settle(conn, id, lease_token, now, None).await.map(|_| ())
}

pub async fn mark_failed(
    conn: &DatabaseConnection,
    id: i32,
    lease_token: &str,
    error_code: &str,
    error_message: &str,
    retryable: bool,
    now: DateTime<Utc>,
) -> Result<String, ProviderJobError> {
    settle(
        conn,
        id,
        lease_token,
        now,
        Some((error_code, error_message, retryable)),
    )
    .await
    .map(|status| status.unwrap_or_else(|| STATUS_SUCCEEDED.to_string()))
}

async fn settle(
    conn: &DatabaseConnection,
    id: i32,
    lease_token: &str,
    now: DateTime<Utc>,
    failure: Option<(&str, &str, bool)>,
) -> Result<Option<String>, ProviderJobError> {
    if lease_token.is_empty() {
        return Err(ProviderJobError::LeaseLost);
    }
    let token_hash = lease_hash(lease_token);
    let txn = conn.begin().await?;
    let Some(row) = provider_job::Entity::find_by_id(id).one(&txn).await? else {
        txn.rollback().await?;
        return Err(ProviderJobError::NotFound);
    };
    if row.status != STATUS_RUNNING || row.lease_token_hash.as_deref() != Some(token_hash.as_str())
    {
        txn.rollback().await?;
        return Err(ProviderJobError::LeaseLost);
    }
    let (status, attempt_status, completed_at, next_run_at, safe_code, safe_message) =
        if let Some((code, message, retryable)) = failure {
            let should_retry = retryable && row.attempt_count < row.max_attempts;
            let status = if should_retry {
                STATUS_QUEUED
            } else {
                STATUS_FAILED
            };
            (
                status,
                if should_retry {
                    ATTEMPT_RETRYABLE_FAILED
                } else {
                    ATTEMPT_FAILED
                },
                (!should_retry).then_some(now),
                if should_retry {
                    now + retry_delay(row.attempt_count)
                } else {
                    now
                },
                Some(safe_error_code(code)),
                Some(safe_error_message(message)),
            )
        } else {
            (
                STATUS_SUCCEEDED,
                ATTEMPT_SUCCEEDED,
                Some(now),
                now,
                None,
                None,
            )
        };
    let update = provider_job::Entity::update_many()
        .filter(provider_job::Column::Id.eq(id))
        .filter(provider_job::Column::Status.eq(STATUS_RUNNING))
        .filter(provider_job::Column::LeaseTokenHash.eq(token_hash))
        .col_expr(provider_job::Column::Status, Expr::value(status))
        .col_expr(provider_job::Column::NextRunAt, Expr::value(next_run_at))
        .col_expr(
            provider_job::Column::LeaseTokenHash,
            Expr::value(None::<String>),
        )
        .col_expr(
            provider_job::Column::LeaseUntil,
            Expr::value(None::<DateTime<Utc>>),
        )
        .col_expr(
            provider_job::Column::LastErrorCode,
            Expr::value(safe_code.clone()),
        )
        .col_expr(
            provider_job::Column::LastErrorMessage,
            Expr::value(safe_message.clone()),
        )
        .col_expr(provider_job::Column::CompletedAt, Expr::value(completed_at))
        .col_expr(provider_job::Column::UpdatedAt, Expr::value(now))
        .exec(&txn)
        .await?;
    if update.rows_affected != 1 {
        txn.rollback().await?;
        return Err(ProviderJobError::LeaseLost);
    }
    provider_job_attempt::Entity::update_many()
        .filter(provider_job_attempt::Column::JobId.eq(id))
        .filter(provider_job_attempt::Column::AttemptNo.eq(row.attempt_count))
        .filter(provider_job_attempt::Column::Status.eq(ATTEMPT_RUNNING))
        .col_expr(
            provider_job_attempt::Column::Status,
            Expr::value(attempt_status),
        )
        .col_expr(
            provider_job_attempt::Column::FinishedAt,
            Expr::value(Some(now)),
        )
        .col_expr(
            provider_job_attempt::Column::ErrorCode,
            Expr::value(safe_code),
        )
        .col_expr(
            provider_job_attempt::Column::ErrorMessage,
            Expr::value(safe_message),
        )
        .exec(&txn)
        .await?;
    txn.commit().await?;
    if failure.is_some() {
        Ok(Some(status.to_string()))
    } else {
        Ok(None)
    }
}

pub async fn attempt_history(
    conn: &DatabaseConnection,
    job_id: i32,
    limit: u64,
) -> Result<Vec<provider_job_attempt::Model>, ProviderJobError> {
    Ok(provider_job_attempt::Entity::find()
        .filter(provider_job_attempt::Column::JobId.eq(job_id))
        .order_by_asc(provider_job_attempt::Column::AttemptNo)
        .limit(limit.min(MAX_ATTEMPTS as u64))
        .all(conn)
        .await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_are_bounded_and_redacted() {
        assert_eq!(
            safe_error_message("Authorization: bearer secret"),
            "provider operation failed"
        );
        assert!(safe_error_message(&"x".repeat(900)).len() <= MAX_ERROR_MESSAGE_LENGTH);
    }

    #[test]
    fn retry_delay_is_bounded() {
        assert_eq!(retry_delay(1), Duration::seconds(1));
        assert_eq!(retry_delay(6), Duration::seconds(32));
    }

    #[test]
    fn request_hash_rejects_raw_payloads() {
        let error = validate_spec(&ProviderJobSpec {
            provider_kind: "wiki".into(),
            provider_id: "main".into(),
            operation: "sync".into(),
            idempotency_key: "key".into(),
            request_hash: "raw-payload".into(),
            max_attempts: 3,
        })
        .unwrap_err();
        assert!(matches!(error, ProviderJobError::Validation(_)));
    }
}
