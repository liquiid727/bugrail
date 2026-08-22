use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(WorkTask::Table)
                    .add_column(
                        ColumnDef::new(WorkTask::TaskKind)
                            .string()
                            .not_null()
                            .default("implementation"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(WorkTask::Table)
                    .drop_column(WorkTask::TaskKind)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum WorkTask {
    Table,
    TaskKind,
}
