use sea_orm::entity::prelude::*;

/// Memory capture outbox row (BUGRAIL-SPECOS-017 §5). One delivery per
/// `(provider_id, task_id, run_seq)` settled run generation. The staged
/// `payload` body is cleared after a successful delivery; `payload_hash`,
/// source IDs, upstream accepted IDs and safe error fields remain as the
/// durable evidence.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "memory_capture_delivery")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub provider_id: String,
    pub folder_id: i32,
    pub task_id: i32,
    pub run_seq: i32,
    pub conversation_id: i32,
    /// Staged filtered send payload (JSON). `None` after delivery.
    #[sea_orm(column_type = "Text", nullable)]
    pub payload: Option<String>,
    pub payload_hash: String,
    /// JSON array of the stable upstream message ids in the payload.
    #[sea_orm(column_type = "Text")]
    pub source_message_ids: String,
    /// `queued` | `sending` | `delivered` | `failed`.
    pub status: String,
    pub attempts: i32,
    /// Whether the last failure class allows an automatic retry.
    pub retryable: bool,
    /// JSON array echoed by the patched Gateway under the upsert contract.
    #[sea_orm(column_type = "Text", nullable)]
    pub upstream_accepted_ids: Option<String>,
    #[sea_orm(column_name = "safe_error_code")]
    pub safe_error_code: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub safe_error_message: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub delivered_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
