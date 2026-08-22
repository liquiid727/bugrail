use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(WorkTaskHandoff::Table)
                    .add_column(
                        ColumnDef::new(WorkTaskHandoff::Actor)
                            .string()
                            .not_null()
                            .default("human"),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(WorkTaskHandoff::Table)
                    .add_column(
                        ColumnDef::new(WorkTaskHandoff::SourceBranch)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(WorkTaskHandoff::Table)
                    .add_column(ColumnDef::new(WorkTaskHandoff::SourceHead).string().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(WorkTaskHandoff::Table)
                    .drop_column(WorkTaskHandoff::SourceHead)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(WorkTaskHandoff::Table)
                    .drop_column(WorkTaskHandoff::SourceBranch)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(WorkTaskHandoff::Table)
                    .drop_column(WorkTaskHandoff::Actor)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum WorkTaskHandoff {
    Table,
    Actor,
    SourceBranch,
    SourceHead,
}
