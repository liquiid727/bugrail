use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "work_task_context_pack")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub task_id: i32,
    pub run_seq: i32,
    pub loadout_id: String,
    pub status: String,
    pub content_hash: String,
    pub estimated_tokens: i32,
    pub total_bytes: i32,
    #[sea_orm(column_type = "Text")]
    pub provider_status: String,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::work_task::Entity",
        from = "Column::TaskId",
        to = "super::work_task::Column::Id"
    )]
    Task,
    #[sea_orm(has_many = "super::work_task_context_item::Entity")]
    Items,
}
impl Related<super::work_task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}
impl Related<super::work_task_context_item::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Items.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}
