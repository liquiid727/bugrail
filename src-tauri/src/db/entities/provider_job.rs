use sea_orm::entity::prelude::*;

/// Durable facts for one external Context-plugin operation. The request body
/// is deliberately absent; callers persist only a stable request hash and
/// must use `idempotency_key` with the provider for external mutation safety.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "provider_job")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub provider_kind: String,
    pub provider_id: String,
    pub operation: String,
    pub idempotency_key: String,
    pub request_hash: String,
    /// `queued` | `running` | `succeeded` | `failed`.
    pub status: String,
    pub attempt_count: i32,
    pub max_attempts: i32,
    pub next_run_at: DateTimeUtc,
    /// SHA-256 of the in-memory lease token; the bearer token is never stored.
    pub lease_token_hash: Option<String>,
    pub lease_until: Option<DateTimeUtc>,
    pub last_error_code: Option<String>,
    pub last_error_message: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub completed_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::provider_job_attempt::Entity")]
    Attempts,
}

impl Related<super::provider_job_attempt::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Attempts.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
