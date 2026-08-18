use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "team_run")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub folder_id: i32,
    pub team_id: String,
    pub workflow_id: String,
    pub workflow_version: i32,
    pub max_concurrent: i32,
    pub control_state: String,
    pub definition_hash: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub finished_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::team_run_task::Entity")]
    Tasks,
}
impl Related<super::team_run_task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tasks.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}
