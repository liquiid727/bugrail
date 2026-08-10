//! Migration + repository integration tests for the SpecOS contract and gate
//! tables (BUGRAIL-SPECOS-001 issue-001). Verifies migration up/down, indexes,
//! FK cascade, and that a rollback of the new tables leaves the legacy
//! `work_task` schema readable.

use codeg_lib::db::entities::{work_task, work_task_contract, work_task_gate_result};
use codeg_lib::db::migration::Migrator;
use codeg_lib::db::service::work_task_service;
use codeg_lib::db::test_helpers::{fresh_in_memory_db, seed_folder};
use sea_orm::{
    ActiveValue, ColumnTrait, ConnectionTrait, DbBackend, EntityTrait, PaginatorTrait,
    QueryFilter, Statement,
};
use sea_orm_migration::MigratorTrait;

async fn table_names(conn: &codeg_lib::db::AppDatabase) -> Vec<String> {
    let rows = conn
        .conn
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

async fn index_names(conn: &codeg_lib::db::AppDatabase) -> Vec<String> {
    let rows = conn
        .conn
        .query_all(Statement::from_string(
            DbBackend::Sqlite,
            "SELECT name FROM sqlite_master WHERE type='index' AND tbl_name IN \
             ('work_task_contract', 'work_task_gate_result')".to_string(),
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
    };
    let t = work_task_service::create(&db.conn, draft).await.unwrap();
    work_task_service::upsert_contract(&db.conn, contract_active(t.id))
        .await
        .unwrap();

    // Revert exactly this migration (it is the newest).
    Migrator::down(&db.conn, Some(1)).await.expect("down migration");

    let names = table_names(&db).await;
    assert!(!names.contains(&"work_task_contract".to_string()));
    assert!(!names.contains(&"work_task_gate_result".to_string()));
    assert!(names.contains(&"work_task".to_string()));

    // The legacy task row is still readable after the rollback.
    let row = work_task::Entity::find()
        .filter(work_task::Column::Id.eq(t.id))
        .one(&db.conn)
        .await
        .unwrap()
        .expect("legacy task readable");
    assert_eq!(row.title, "legacy");
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
    let del = work_task::Entity::delete_by_id(t.id).exec(&db.conn).await.unwrap();
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
