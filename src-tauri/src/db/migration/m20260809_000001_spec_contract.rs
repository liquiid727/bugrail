use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // work_task_contract: one optional Spec contract per WorkTask (R01-R03).
        // A legacy task has no row and keeps its current behavior. task_id is the
        // primary key and a hard FK to work_task — the contract dies with its task.
        manager
            .create_table(
                Table::create()
                    .table(WorkTaskContract::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(WorkTaskContract::TaskId)
                            .integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(WorkTaskContract::SourceSpecId).string().not_null())
                    .col(ColumnDef::new(WorkTaskContract::SourceSpecVersion).string().not_null())
                    .col(ColumnDef::new(WorkTaskContract::SourceSpecPath).string().not_null())
                    .col(ColumnDef::new(WorkTaskContract::SourceSpecHash).string().not_null())
                    // JSON snapshot of selected acceptance criteria (id + text).
                    .col(
                        ColumnDef::new(WorkTaskContract::AcceptanceCriteria)
                            .text()
                            .not_null(),
                    )
                    // JSON snapshot of the required gate policy.
                    .col(ColumnDef::new(WorkTaskContract::GatePolicy).text().not_null())
                    .col(
                        ColumnDef::new(WorkTaskContract::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WorkTaskContract::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(WorkTaskContract::Table, WorkTaskContract::TaskId)
                            .to(WorkTask::Table, WorkTask::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Trace queries by Spec id (which tasks bound to which Feature Spec).
        manager
            .create_index(
                Index::create()
                    .name("idx_work_task_contract_spec_id")
                    .table(WorkTaskContract::Table)
                    .col(WorkTaskContract::SourceSpecId)
                    .to_owned(),
            )
            .await?;

        // work_task_gate_result: append-only gate attempts (R05, R09). One row
        // per attempt; the same (run_seq, gate_id) can appear many times. The
        // composite index makes "latest attempt for a run/gate" a cheap ordered
        // lookup, and keeps retry history auditable.
        manager
            .create_table(
                Table::create()
                    .table(WorkTaskGateResult::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(WorkTaskGateResult::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(WorkTaskGateResult::TaskId).integer().not_null())
                    .col(ColumnDef::new(WorkTaskGateResult::RunSeq).integer().not_null())
                    .col(ColumnDef::new(WorkTaskGateResult::GateId).string().not_null())
                    // preflight | human_approval
                    .col(ColumnDef::new(WorkTaskGateResult::GateType).string().not_null())
                    // running | passed | failed | blocked | waived
                    .col(ColumnDef::new(WorkTaskGateResult::Status).string().not_null())
                    .col(ColumnDef::new(WorkTaskGateResult::Required).boolean().not_null())
                    .col(
                        ColumnDef::new(WorkTaskGateResult::Reusable)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    // engine | user (derived from the command context; never request JSON).
                    .col(ColumnDef::new(WorkTaskGateResult::Actor).string().not_null())
                    // JSON references and capped summaries (never secrets/full output).
                    .col(ColumnDef::new(WorkTaskGateResult::Evidence).text().null())
                    // Required for failed / blocked / waived.
                    .col(ColumnDef::new(WorkTaskGateResult::Reason).text().null())
                    .col(
                        ColumnDef::new(WorkTaskGateResult::StartedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WorkTaskGateResult::FinishedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(WorkTaskGateResult::Table, WorkTaskGateResult::TaskId)
                            .to(WorkTask::Table, WorkTask::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_work_task_gate_task_run")
                    .table(WorkTaskGateResult::Table)
                    .col(WorkTaskGateResult::TaskId)
                    .col(WorkTaskGateResult::RunSeq)
                    .col(WorkTaskGateResult::GateId)
                    .col(WorkTaskGateResult::Id)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop gate results before contracts (child before parent is cosmetic
        // here since each has its own FK to work_task, but it mirrors the
        // dependency order the spec calls for).
        manager
            .drop_table(
                Table::drop()
                    .table(WorkTaskGateResult::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(WorkTaskContract::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum WorkTask {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum WorkTaskContract {
    Table,
    TaskId,
    SourceSpecId,
    SourceSpecVersion,
    SourceSpecPath,
    SourceSpecHash,
    AcceptanceCriteria,
    GatePolicy,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum WorkTaskGateResult {
    Table,
    Id,
    TaskId,
    RunSeq,
    GateId,
    GateType,
    Status,
    Required,
    Reusable,
    Actor,
    Evidence,
    Reason,
    StartedAt,
    FinishedAt,
}
