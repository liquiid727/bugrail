use sea_orm::entity::prelude::*;

/// One optional Spec contract per WorkTask (BUGRAIL-SPECOS-001.R01-R03).
/// A legacy task has no row and keeps its current behavior. The row stores the
/// validated repository-relative reference plus a JSON snapshot of the selected
/// acceptance criteria and the required gate policy.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "work_task_contract")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub task_id: i32,
    pub source_spec_id: String,
    pub source_spec_version: String,
    /// Repository-relative path, canonicalized and confined to the project root.
    pub source_spec_path: String,
    /// SHA-256 of the approved source content at bind time.
    pub source_spec_hash: String,
    /// JSON array snapshot of selected `AcceptanceCriterionSnapshot`.
    #[sea_orm(column_type = "Text")]
    pub acceptance_criteria: String,
    /// JSON `WorkTaskGatePolicy` snapshot.
    #[sea_orm(column_type = "Text")]
    pub gate_policy: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::work_task::Entity",
        from = "Column::TaskId",
        to = "super::work_task::Column::Id"
    )]
    Task,
}

impl Related<super::work_task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
