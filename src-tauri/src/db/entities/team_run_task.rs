use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "team_run_task")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub team_run_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub node_id: String,
    pub task_id: i32,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::team_run::Entity",
        from = "Column::TeamRunId",
        to = "super::team_run::Column::Id"
    )]
    Run,
    #[sea_orm(
        belongs_to = "super::work_task::Entity",
        from = "Column::TaskId",
        to = "super::work_task::Column::Id"
    )]
    Task,
}
impl Related<super::team_run::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Run.def()
    }
}
impl Related<super::work_task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}
