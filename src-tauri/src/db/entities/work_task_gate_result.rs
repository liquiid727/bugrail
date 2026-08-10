use sea_orm::entity::prelude::*;

/// One append-only gate attempt for a WorkTask (BUGRAIL-SPECOS-001.R05, R09).
/// Rows are scoped to `task_id` + `run_seq`; a retry starts a new `run_seq` and
/// previous attempts stay auditable but do not pass the new run unless the gate
/// policy marks them reusable. Written in the SAME transaction as the state
/// transition (or decision event) they record.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "work_task_gate_result")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub task_id: i32,
    /// Execution generation the attempt belongs to.
    pub run_seq: i32,
    pub gate_id: String,
    /// preflight | human_approval
    pub gate_type: String,
    /// running | passed | failed | blocked | waived
    pub status: String,
    /// Whether this gate participates in the merge/complete decision.
    pub required: bool,
    /// A passing preflight may be reused across runs only while its evidence
    /// `verified_head` matches the current Worktree HEAD and the bound Spec hash
    /// is unchanged.
    pub reusable: bool,
    /// engine | user — derived from the command context, never request JSON.
    pub actor: String,
    /// JSON references and capped summaries (never secrets/full command output).
    #[sea_orm(column_type = "Text")]
    pub evidence: Option<String>,
    /// Required for failed / blocked / waived.
    pub reason: Option<String>,
    pub started_at: DateTimeUtc,
    pub finished_at: Option<DateTimeUtc>,
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
