use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "work_task_run")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub task_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub run_seq: i32,
    pub status: String,
    pub agent_profile_id: Option<String>,
    pub model_profile_id: Option<String>,
    pub agent_type: Option<String>,
    pub model: Option<String>,
    pub mode_id: Option<String>,
    pub reasoning: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub resolution: Option<String>,
    pub conversation_id: Option<i32>,
    pub worktree_folder_id: Option<i32>,
    pub context_package_id: Option<String>,
    pub created_at: DateTimeUtc,
    pub started_at: Option<DateTimeUtc>,
    pub finished_at: Option<DateTimeUtc>,
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
