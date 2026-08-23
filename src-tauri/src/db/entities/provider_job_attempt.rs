use sea_orm::entity::prelude::*;

/// Bounded, append-only attempt evidence for a provider job. Error fields are
/// safe, capped diagnostics; provider payloads and credentials are not stored.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "provider_job_attempt")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub job_id: i32,
    pub attempt_no: i32,
    pub lease_token_hash: String,
    /// `running` | `succeeded` | `retryable_failed` | `failed` | `interrupted`.
    pub status: String,
    pub started_at: DateTimeUtc,
    pub finished_at: Option<DateTimeUtc>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::provider_job::Entity",
        from = "Column::JobId",
        to = "super::provider_job::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Job,
}

impl Related<super::provider_job::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Job.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
