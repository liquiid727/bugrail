//! Migration + repository integration tests for the SpecOS contract and gate
//! tables (BUGRAIL-SPECOS-001 issue-001). Verifies migration up/down, indexes,
//! FK cascade, and that a rollback of the new tables leaves the legacy
//! `work_task` schema readable.

use codeg_lib::db::entities::{work_task, work_task_contract, work_task_gate_result};
use codeg_lib::db::migration::Migrator;
use codeg_lib::db::service::work_task_service;
use codeg_lib::db::test_helpers::{fresh_in_memory_db, seed_folder};
use sea_orm::{
    ActiveValue, ColumnTrait, ConnectionTrait, Database, DatabaseConnection, DbBackend,
    EntityTrait, PaginatorTrait, QueryFilter, Statement,
};
use sea_orm_migration::{prelude::MigrationTrait, MigratorTrait, SchemaManager};

struct PreSpecMigrator;

#[async_trait::async_trait]
impl MigratorTrait for PreSpecMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        Migrator::migrations()
            .into_iter()
            .take_while(|migration| migration.name() != "m20260809_000001_spec_contract")
            .collect()
    }
}

async fn table_names_for_connection(conn: &DatabaseConnection) -> Vec<String> {
    let rows = conn
        .query_all(Statement::from_string(
            DbBackend::Sqlite,
            "SELECT name FROM sqlite_master WHERE type='table'".to_string(),
        ))
        .await
        .unwrap();
    rows.iter()
        .filter_map(|r| r.try_get("", "name").ok())
        .collect()
}

async fn table_names(conn: &codeg_lib::db::AppDatabase) -> Vec<String> {
    table_names_for_connection(&conn.conn).await
}

async fn index_names(conn: &codeg_lib::db::AppDatabase) -> Vec<String> {
    let rows = conn
        .conn
        .query_all(Statement::from_string(
            DbBackend::Sqlite,
            "SELECT name FROM sqlite_master WHERE type='index' AND tbl_name IN \
             ('work_task_contract', 'work_task_gate_result')"
                .to_string(),
        ))
        .await
        .unwrap();
    rows.iter()
        .filter_map(|r| r.try_get("", "name").ok())
        .collect()
}

fn contract_active(task_id: i32) -> work_task_contract::ActiveModel {
    work_task_contract::ActiveModel {
        task_id: ActiveValue::Set(task_id),
        source_spec_id: ActiveValue::Set("S".to_string()),
        source_spec_version: ActiveValue::Set("1".to_string()),
        source_spec_path: ActiveValue::Set("spec.md".to_string()),
        source_spec_hash: ActiveValue::Set("h".to_string()),
        acceptance_criteria: ActiveValue::Set("[]".to_string()),
        gate_policy: ActiveValue::Set("{\"gates\":[]}".to_string()),
        created_at: ActiveValue::Set(chrono::Utc::now()),
        updated_at: ActiveValue::Set(chrono::Utc::now()),
    }
}

#[tokio::test]
async fn migration_creates_tables_and_indexes() {
    let db = fresh_in_memory_db().await;
    let names = table_names(&db).await;
    assert!(names.contains(&"work_task_contract".to_string()));
    assert!(names.contains(&"work_task_gate_result".to_string()));

    let idx = index_names(&db).await;
    assert!(idx.contains(&"idx_work_task_contract_spec_id".to_string()));
    assert!(idx.contains(&"idx_work_task_gate_task_run".to_string()));
}

#[tokio::test]
async fn interrupted_migration_keeps_a_complete_schema_prefix() {
    let conn = Database::connect("sqlite::memory:").await.unwrap();
    PreSpecMigrator::up(&conn, None).await.unwrap();
    conn.execute_unprepared(
        "CREATE TABLE specos_index_blocker (source_spec_id TEXT); \
         CREATE INDEX idx_work_task_contract_spec_id \
         ON specos_index_blocker(source_spec_id);",
    )
    .await
    .unwrap();

    let spec_contract = Migrator::migrations()
        .into_iter()
        .find(|migration| migration.name() == "m20260809_000001_spec_contract")
        .expect("Spec contract migration");
    let error = spec_contract
        .up(&SchemaManager::new(&conn))
        .await
        .expect_err("the conflicting index must interrupt the SpecOS migration");
    assert!(error.to_string().contains("idx_work_task_contract_spec_id"));

    let names = table_names_for_connection(&conn).await;
    let has_contract = names.contains(&"work_task_contract".to_string());
    let has_gates = names.contains(&"work_task_gate_result".to_string());
    assert_eq!(
        has_contract, has_gates,
        "SpecOS contract schema must be entirely pre- or post-feature"
    );
    assert!(
        !has_contract,
        "the interrupted migration must restore the pre-feature schema"
    );

    let applied = conn
        .query_one(Statement::from_string(
            DbBackend::Sqlite,
            "SELECT COUNT(*) AS count FROM seaql_migrations".to_string(),
        ))
        .await
        .unwrap()
        .expect("migration ledger row");
    assert_eq!(
        applied.try_get::<i64>("", "count").unwrap(),
        PreSpecMigrator::migrations().len() as i64,
        "the failed migration must not enter the migration ledger"
    );
}

#[tokio::test]
async fn down_drops_only_the_specos_tables() {
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, "/tmp/wt-specos-down").await;
    let draft = codeg_lib::models::WorkTaskDraft {
        folder_id,
        title: "legacy".to_string(),
        config: serde_json::json!({
            "display_text": "legacy",
            "prompt_blocks": [{ "type": "text", "text": "legacy" }],
        }),
        task_kind: Default::default(),
    };
    let t = work_task_service::create(&db.conn, draft).await.unwrap();
    work_task_service::upsert_contract(&db.conn, contract_active(t.id))
        .await
        .unwrap();

    // Revert the migrations from the current tail through the Spec contract,
    // leaving the pre-feature WorkTask schema intact.
    Migrator::down(&db.conn, Some(10))
        .await
        .expect("down migrations");

    let names = table_names(&db).await;
    assert!(!names.contains(&"work_task_contract".to_string()));
    assert!(!names.contains(&"work_task_gate_result".to_string()));
    for name in [
        "provider_job",
        "provider_job_attempt",
        "work_task_run",
        "work_task_dependency",
        "work_task_handoff",
        "team_run",
        "team_run_task",
        "work_task_context_pack",
        "work_task_context_item",
        "context_activity",
    ] {
        assert!(!names.contains(&name.to_string()), "{name} must roll back");
    }
    assert!(names.contains(&"work_task".to_string()));

    // The legacy task row is still readable after the rollback.
    let row = db
        .conn
        .query_one(Statement::from_string(
            DbBackend::Sqlite,
            format!("SELECT id, title FROM work_task WHERE id = {}", t.id),
        ))
        .await
        .unwrap()
        .expect("legacy task readable");
    assert_eq!(row.try_get::<i32>("", "id").unwrap(), t.id);
    assert_eq!(row.try_get::<String>("", "title").unwrap(), "legacy");
}

#[tokio::test]
async fn fk_cascade_removes_contract_and_gate_rows_with_the_task() {
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, "/tmp/wt-specos-fk").await;
    let draft = codeg_lib::models::WorkTaskDraft {
        folder_id,
        title: "fk".to_string(),
        config: serde_json::json!({
            "display_text": "fk",
            "prompt_blocks": [{ "type": "text", "text": "fk" }],
        }),
        task_kind: Default::default(),
    };
    let t = work_task_service::create(&db.conn, draft).await.unwrap();

    work_task_service::upsert_contract(&db.conn, contract_active(t.id))
        .await
        .unwrap();
    work_task_service::insert_gate_result(
        &db.conn,
        work_task_gate_result::ActiveModel {
            id: ActiveValue::NotSet,
            task_id: ActiveValue::Set(t.id),
            run_seq: ActiveValue::Set(0),
            gate_id: ActiveValue::Set("preflight".to_string()),
            gate_type: ActiveValue::Set("preflight".to_string()),
            status: ActiveValue::Set("passed".to_string()),
            required: ActiveValue::Set(true),
            reusable: ActiveValue::Set(false),
            actor: ActiveValue::Set("engine".to_string()),
            evidence: ActiveValue::Set(None),
            reason: ActiveValue::Set(None),
            started_at: ActiveValue::Set(chrono::Utc::now()),
            finished_at: ActiveValue::Set(Some(chrono::Utc::now())),
        },
    )
    .await
    .unwrap();

    // Foreign keys are ON in the test harness; deleting the task must cascade.
    let del = work_task::Entity::delete_by_id(t.id)
        .exec(&db.conn)
        .await
        .unwrap();
    assert_eq!(del.rows_affected, 1);

    let contract_count = work_task_contract::Entity::find()
        .filter(work_task_contract::Column::TaskId.eq(t.id))
        .count(&db.conn)
        .await
        .unwrap();
    assert_eq!(contract_count, 0, "contract cascades");

    let gate_count = work_task_gate_result::Entity::find()
        .filter(work_task_gate_result::Column::TaskId.eq(t.id))
        .count(&db.conn)
        .await
        .unwrap();
    assert_eq!(gate_count, 0, "gate results cascade");
}

/// The composite index is what makes latest-attempt lookups cheap; make sure it
/// actually serves the query plan.
#[tokio::test]
async fn gate_index_is_used_by_latest_lookup() {
    let db = fresh_in_memory_db().await;
    let plan = db
        .conn
        .query_all(Statement::from_string(
            DbBackend::Sqlite,
            "EXPLAIN QUERY PLAN SELECT * FROM work_task_gate_result \
             WHERE task_id = 1 AND run_seq = 0 AND gate_id = 'g' \
             ORDER BY id DESC LIMIT 1",
        ))
        .await
        .unwrap();
    let joined = plan
        .iter()
        .filter_map(|r| r.try_get::<String>("", "detail").ok())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        joined.contains("idx_work_task_gate_task_run"),
        "expected index usage, got: {joined}"
    );
}
