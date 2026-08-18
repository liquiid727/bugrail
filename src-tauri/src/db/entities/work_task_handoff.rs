use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "work_task_handoff")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub task_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub run_seq: i32,
    #[sea_orm(column_type = "Text")]
    pub summary: String,
    #[sea_orm(column_type = "Text")]
    pub artifacts: String,
    #[sea_orm(column_type = "Text")]
    pub risks: String,
    #[sea_orm(column_type = "Text")]
    pub open_questions: String,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
