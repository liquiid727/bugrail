//! Memory capture outbox repository (BUGRAIL-SPECOS-017 §5).
//!
//! Thin persistence helpers; capture policy lives in
//! `crate::memory::capture` and delivery in `crate::memory::capture_worker`.
//! One row per `(provider_id, task_id, run_seq)` — the unique index makes
//! enqueue and reconciliation idempotent.

use chrono::{DateTime, Utc};
use sea_orm::{
    sea_query::OnConflict, ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait,
    IntoActiveModel, QueryFilter, QueryOrder, QuerySelect, Set,
};

use crate::db::entities::memory_capture_delivery as delivery;
use crate::db::error::DbError;

pub const STATUS_QUEUED: &str = "queued";
pub const STATUS_SENDING: &str = "sending";
pub const STATUS_DELIVERED: &str = "delivered";
pub const STATUS_FAILED: &str = "failed";

/// Automatic delivery attempts per row (spec §5: at most 5).
pub const MAX_ATTEMPTS: i32 = 5;

/// Staged payload plus the durable identity columns of a new row.
pub struct NewDelivery {
    pub provider_id: String,
    pub folder_id: i32,
    pub task_id: i32,
    pub run_seq: i32,
    pub conversation_id: i32,
    pub payload: String,
    pub payload_hash: String,
    pub source_message_ids: String,
}

/// Insert one queued row; a row for the same `(provider_id, task_id,
/// run_seq)` wins without overwrite (idempotent enqueue/reconciliation).
/// Returns the row that now owns the delivery.
pub async fn enqueue<C: ConnectionTrait>(
    conn: &C,
    new: NewDelivery,
) -> Result<delivery::Model, DbError> {
    let now = Utc::now();
    let model = delivery::ActiveModel {
        provider_id: Set(new.provider_id.clone()),
        folder_id: Set(new.folder_id),
        task_id: Set(new.task_id),
        run_seq: Set(new.run_seq),
        conversation_id: Set(new.conversation_id),
        payload: Set(Some(new.payload)),
        payload_hash: Set(new.payload_hash),
        source_message_ids: Set(new.source_message_ids),
        status: Set(STATUS_QUEUED.into()),
        attempts: Set(0),
        retryable: Set(false),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    let inserted = delivery::Entity::insert(model)
        .on_conflict(
            OnConflict::columns([
                delivery::Column::ProviderId,
                delivery::Column::TaskId,
                delivery::Column::RunSeq,
            ])
            .do_nothing()
            .to_owned(),
        )
        .exec(conn)
        .await;
    match inserted {
        Ok(res) => {
            let id = res.last_insert_id;
            Ok(delivery::Entity::find_by_id(id)
                .one(conn)
                .await?
                .ok_or_else(|| {
                    DbError::Validation("capture delivery row vanished after insert".into())
                })?)
        }
        // The unique key already owns this run generation — return the
        // existing row instead of duplicating it.
        Err(_) => find_for_run(conn, &new.provider_id, new.task_id, new.run_seq)
            .await?
            .ok_or_else(|| {
                DbError::Validation("capture delivery insert conflicted without a row".into())
            }),
    }
}

pub async fn get<C: ConnectionTrait>(
    conn: &C,
    id: i32,
) -> Result<Option<delivery::Model>, DbError> {
    Ok(delivery::Entity::find_by_id(id).one(conn).await?)
}

pub async fn find_for_run<C: ConnectionTrait>(
    conn: &C,
    provider_id: &str,
    task_id: i32,
    run_seq: i32,
) -> Result<Option<delivery::Model>, DbError> {
    Ok(delivery::Entity::find()
        .filter(delivery::Column::ProviderId.eq(provider_id))
        .filter(delivery::Column::TaskId.eq(task_id))
        .filter(delivery::Column::RunSeq.eq(run_seq))
        .one(conn)
        .await?)
}

/// Queued rows whose exponential backoff has elapsed, oldest first.
pub async fn due<C: ConnectionTrait>(
    conn: &C,
    now: DateTime<Utc>,
    limit: u64,
) -> Result<Vec<delivery::Model>, DbError> {
    let rows = delivery::Entity::find()
        .filter(delivery::Column::Status.eq(STATUS_QUEUED))
        .order_by_asc(delivery::Column::UpdatedAt)
        .limit(limit)
        .all(conn)
        .await?;
    Ok(rows
        .into_iter()
        .filter(|row| backoff_due(row, now))
        .collect())
}

/// Exponential backoff: `2^attempts` seconds after the last state change
/// (attempts counts completed delivery attempts). Attempt 0 is due at once.
pub fn backoff_due(row: &delivery::Model, now: DateTime<Utc>) -> bool {
    if row.attempts <= 0 {
        return true;
    }
    let seconds = 1i64 << row.attempts.min(6);
    now.signed_duration_since(row.updated_at) >= chrono::Duration::seconds(seconds)
}

/// Crash recovery: rows stuck in `sending` go back to `queued` (their attempt
/// outcome is unknown — the at-least-once contract covers the replay).
pub async fn recover_sending<C: ConnectionTrait>(conn: &C) -> Result<u64, DbError> {
    let res = delivery::Entity::update_many()
        .filter(delivery::Column::Status.eq(STATUS_SENDING))
        .col_expr(
            delivery::Column::Status,
            sea_orm::sea_query::Expr::value(STATUS_QUEUED),
        )
        .col_expr(
            delivery::Column::UpdatedAt,
            sea_orm::sea_query::Expr::value(Utc::now()),
        )
        .exec(conn)
        .await?;
    Ok(res.rows_affected)
}

/// Claim the row for one delivery attempt (`queued -> sending`). Returns
/// false when the row was already claimed or settled.
pub async fn mark_sending<C: ConnectionTrait>(conn: &C, id: i32) -> Result<bool, DbError> {
    let res = delivery::Entity::update_many()
        .filter(delivery::Column::Id.eq(id))
        .filter(delivery::Column::Status.eq(STATUS_QUEUED))
        .col_expr(
            delivery::Column::Status,
            sea_orm::sea_query::Expr::value(STATUS_SENDING),
        )
        .col_expr(
            delivery::Column::UpdatedAt,
            sea_orm::sea_query::Expr::value(Utc::now()),
        )
        .exec(conn)
        .await?;
    Ok(res.rows_affected == 1)
}

/// Successful delivery: clear the staged payload body and keep hash, source
/// IDs, upstream accepted IDs (spec §5 retention rule). `attempts` records
/// how many delivery attempts the row consumed (1 for a first-pass success).
pub async fn mark_delivered<C: ConnectionTrait>(
    conn: &C,
    id: i32,
    attempts: i32,
    accepted_ids_json: &str,
) -> Result<(), DbError> {
    let Some(row) = delivery::Entity::find_by_id(id).one(conn).await? else {
        return Ok(());
    };
    let now = Utc::now();
    let mut active = row.into_active_model();
    active.status = Set(STATUS_DELIVERED.into());
    active.attempts = Set(attempts);
    active.payload = Set(None);
    active.upstream_accepted_ids = Set(Some(accepted_ids_json.to_string()));
    active.safe_error_code = Set(None);
    active.safe_error_message = Set(None);
    active.updated_at = Set(now);
    active.delivered_at = Set(Some(now));
    active.update(conn).await?;
    Ok(())
}

/// Failure handling. Retryable failures within the attempt budget return to
/// `queued` (exponential backoff via `updated_at`); everything else is
/// terminal `failed`.
pub async fn mark_failed<C: ConnectionTrait>(
    conn: &C,
    id: i32,
    attempts: i32,
    error_code: &str,
    error_message: &str,
    retryable: bool,
) -> Result<String, DbError> {
    let Some(row) = delivery::Entity::find_by_id(id).one(conn).await? else {
        return Ok(STATUS_FAILED.into());
    };
    let attempts = attempts.min(MAX_ATTEMPTS);
    let requeue = retryable && attempts < MAX_ATTEMPTS;
    let status = if requeue {
        STATUS_QUEUED
    } else {
        STATUS_FAILED
    };
    let mut active = row.into_active_model();
    active.status = Set(status.into());
    active.attempts = Set(attempts);
    active.retryable = Set(retryable);
    active.safe_error_code = Set(Some(error_code.to_string()));
    active.safe_error_message = Set(Some(error_message.chars().take(500).collect()));
    active.updated_at = Set(Utc::now());
    active.update(conn).await?;
    Ok(status.into())
}

/// Explicit manual retry (Context UI). Only `failed`/`queued` rows with a
/// staged payload may retry; the attempt counter resets so the worker gets a
/// fresh budget.
pub async fn requeue_for_retry<C: ConnectionTrait>(conn: &C, id: i32) -> Result<bool, DbError> {
    let res = delivery::Entity::update_many()
        .filter(delivery::Column::Id.eq(id))
        .filter(delivery::Column::Status.is_in([STATUS_FAILED, STATUS_QUEUED]))
        .filter(delivery::Column::Payload.is_not_null())
        .col_expr(
            delivery::Column::Status,
            sea_orm::sea_query::Expr::value(STATUS_QUEUED),
        )
        .col_expr(
            delivery::Column::Attempts,
            sea_orm::sea_query::Expr::value(0),
        )
        .col_expr(
            delivery::Column::Retryable,
            sea_orm::sea_query::Expr::value(true),
        )
        .col_expr(
            delivery::Column::UpdatedAt,
            sea_orm::sea_query::Expr::value(Utc::now()),
        )
        .exec(conn)
        .await?;
    Ok(res.rows_affected == 1)
}

/// Newest-first page for the Context UI delivery list. `cursor` is the id
/// strictly below the page start (`None` = latest page).
pub async fn list_by_folder<C: ConnectionTrait>(
    conn: &C,
    folder_id: i32,
    cursor: Option<i32>,
    limit: u64,
) -> Result<Vec<delivery::Model>, DbError> {
    let mut query = delivery::Entity::find()
        .filter(delivery::Column::FolderId.eq(folder_id))
        .order_by_desc(delivery::Column::Id)
        .limit(limit);
    if let Some(cursor) = cursor {
        query = query.filter(delivery::Column::Id.lt(cursor));
    }
    Ok(query.all(conn).await?)
}
