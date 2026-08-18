use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "work_task_dependency")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub parent_task_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub child_task_id: i32,
    pub kind: String,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
