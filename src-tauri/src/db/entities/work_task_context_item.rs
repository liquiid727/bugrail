use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "work_task_context_item")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub package_id: String,
    pub ordinal: i32,
    pub kind: String,
    pub source: String,
    pub title: String,
    #[sea_orm(column_type = "Text")]
    pub content: String,
    pub content_hash: String,
    pub required: bool,
    #[sea_orm(column_type = "Text")]
    pub provenance: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::work_task_context_pack::Entity",
        from = "Column::PackageId",
        to = "super::work_task_context_pack::Column::Id"
    )]
    Package,
}
impl Related<super::work_task_context_pack::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Package.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}
