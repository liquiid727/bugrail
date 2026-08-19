use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Memory capture outbox (BUGRAIL-SPECOS-017 §5). One delivery per
        // `(provider_id, task_id, run_seq)`; the staged payload body is
        // cleared after a successful delivery and only hash/IDs/safe error
        // fields survive.
        manager
            .create_table(
                Table::create()
                    .table(MemoryCaptureDelivery::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(MemoryCaptureDelivery::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(MemoryCaptureDelivery::ProviderId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MemoryCaptureDelivery::FolderId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MemoryCaptureDelivery::TaskId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MemoryCaptureDelivery::RunSeq)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MemoryCaptureDelivery::ConversationId)
                            .integer()
                            .not_null(),
                    )
                    // Staged filtered send payload (JSON). Cleared after a
                    // successful delivery — retention is hash + IDs only.
                    .col(ColumnDef::new(MemoryCaptureDelivery::Payload).text().null())
                    .col(
                        ColumnDef::new(MemoryCaptureDelivery::PayloadHash)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MemoryCaptureDelivery::SourceMessageIds)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MemoryCaptureDelivery::Status)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MemoryCaptureDelivery::Attempts)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(MemoryCaptureDelivery::Retryable)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(MemoryCaptureDelivery::UpstreamAcceptedIds)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(MemoryCaptureDelivery::SafeErrorCode)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(MemoryCaptureDelivery::SafeErrorMessage)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(MemoryCaptureDelivery::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MemoryCaptureDelivery::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MemoryCaptureDelivery::DeliveredAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("uq_memory_capture_delivery_run")
                    .table(MemoryCaptureDelivery::Table)
                    .col(MemoryCaptureDelivery::ProviderId)
                    .col(MemoryCaptureDelivery::TaskId)
                    .col(MemoryCaptureDelivery::RunSeq)
                    .unique()
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_memory_capture_delivery_folder_status")
                    .table(MemoryCaptureDelivery::Table)
                    .col(MemoryCaptureDelivery::FolderId)
                    .col(MemoryCaptureDelivery::Status)
                    .to_owned(),
            )
            .await?;

        // Recall evidence for the run's Context Package (BUGRAIL-SPECOS-017
        // §6): JSON array of included/excluded candidate facts. NULL for
        // packages compiled without Memory candidates.
        manager
            .alter_table(
                Table::alter()
                    .table(WorkTaskContextPack::Table)
                    .add_column(
                        ColumnDef::new(WorkTaskContextPack::MemoryEvidence)
                            .text()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(WorkTaskContextPack::Table)
                    .drop_column(WorkTaskContextPack::MemoryEvidence)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(MemoryCaptureDelivery::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum MemoryCaptureDelivery {
    Table,
    Id,
    ProviderId,
    FolderId,
    TaskId,
    RunSeq,
    ConversationId,
    Payload,
    PayloadHash,
    SourceMessageIds,
    Status,
    Attempts,
    Retryable,
    UpstreamAcceptedIds,
    SafeErrorCode,
    SafeErrorMessage,
    CreatedAt,
    UpdatedAt,
    DeliveredAt,
}
#[derive(DeriveIden)]
enum WorkTaskContextPack {
    Table,
    MemoryEvidence,
}
