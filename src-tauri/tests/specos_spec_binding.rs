//! Focused integration tests for issue-002 (bind/validate a Feature Spec on a
//! WorkTask): Feature Test cases `T01-T09`, `T26`, and the Axum half of `T28`.
//!
//! Scope note: `T25`/`T27`/the decision half of `T28` depend on the gate
//! decision and merge/complete enforcement, which land in issue-003; the
//! staleness helper they build on (`work_task_spec_staleness_core`) is already
//! wired but not exercised here.
//!
//! Transport parity: every request is driven through the real Axum router
//! (`web::router::build_router`) with `authorization: Bearer <token>`. The
//! Tauri command wrappers call the same `commands::work_task::*_core`
//! functions, so the wire contract asserted here is the contract both
//! transports share.

use std::path::Path;
use std::sync::Arc;

use axum_test::{TestResponse, TestServer};
use codeg_lib::app_state::AppState;
use codeg_lib::commands::work_task::work_task_spec_staleness_core;
use codeg_lib::db::entities::work_task;
use codeg_lib::db::service::work_task_service;
use codeg_lib::db::test_helpers::fresh_in_memory_db;
use codeg_lib::db::AppDatabase;
use codeg_lib::models::{WorkTaskDraft, WorkTaskGateStatus, WorkTaskStatus};
use codeg_lib::web::router::build_router;
use codeg_lib::web::shutdown::ShutdownSignal;
use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::{json, Value};

const TEST_TOKEN: &str = "spec-binding-test-token";
const SPEC_PATH: &str = ".features/BUGRAIL-SPECOS-001-work-task-quality/spec.md";

/// A valid repository-local Feature Spec (mirrors the real
/// BUGRAIL-SPECOS-001 front matter + AC table format).
const SPEC_BODY: &str = "---\n\
id: BUGRAIL-SPECOS-001\n\
version: \"0.3\"\n\
title: \"Spec-Linked WorkTask Quality\"\n\
---\n\
# BUGRAIL-SPECOS-001\n\
## 8. Acceptance Criteria\n\
| ID | Criterion | Requirements |\n\
|---|---|---|\n\
| `BUGRAIL-SPECOS-001.AC01` | Preview then bind stores exact metadata. | `R01-R03` |\n\
| `BUGRAIL-SPECOS-001.AC02` | Invalid input is rejected. | `R02`, `R03` |\n\
";

// ────────────────────────────────────────────────────────────────────────────
// Setup helpers
// ────────────────────────────────────────────────────────────────────────────

async fn seed_task(db: &AppDatabase, project_root: &str) -> i32 {
    let folder_id = codeg_lib::db::test_helpers::seed_folder(db, project_root).await;
    let draft = WorkTaskDraft {
        folder_id,
        title: "spec-bound task".into(),
        config: json!({
            "display_text": "do the thing",
            "prompt_blocks": [{ "type": "text", "text": "do the thing" }],
        }),
        task_kind: Default::default(),
    };
    work_task_service::create(&db.conn, draft)
        .await
        .expect("create task")
        .id
}

/// Build the real Axum router over an in-memory DB. The caller keeps the
/// returned temp dirs alive so the on-disk project root / static dir persist.
async fn build_server(
    db: AppDatabase,
) -> (
    TestServer,
    Arc<AppState>,
    tempfile::TempDir,
    tempfile::TempDir,
) {
    let data_dir = tempfile::tempdir().expect("data dir");
    let static_dir = tempfile::tempdir().expect("static dir");
    let state = Arc::new(AppState::new_for_test(db, data_dir.path().to_path_buf()));
    let shutdown = Arc::new(ShutdownSignal::new());
    let router = build_router(
        state.clone(),
        TEST_TOKEN.to_string(),
        static_dir.path().to_path_buf(),
        shutdown,
    );
    let server = TestServer::new(router).expect("test server");
    (server, state, data_dir, static_dir)
}

fn write_spec(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().expect("spec parent")).expect("create dirs");
    std::fs::write(&path, body).expect("write spec");
}

async fn preview(server: &TestServer, task_id: i32, path: &str) -> TestResponse {
    server
        .post("/api/work_task_contract_preview")
        .add_header("authorization", format!("Bearer {TEST_TOKEN}"))
        .json(&json!({ "taskId": task_id, "sourceSpecPath": path }))
        .await
}

async fn bind(server: &TestServer, task_id: i32, draft: Value) -> TestResponse {
    server
        .post("/api/work_task_contract_bind")
        .add_header("authorization", format!("Bearer {TEST_TOKEN}"))
        .json(&json!({ "taskId": task_id, "draft": draft }))
        .await
}

async fn get_contract(server: &TestServer, task_id: i32) -> TestResponse {
    server
        .post("/api/work_task_contract_get")
        .add_header("authorization", format!("Bearer {TEST_TOKEN}"))
        .json(&json!({ "taskId": task_id }))
        .await
}

fn draft_body(path: &str, hash: &str, ac_ids: &[&str], gates: &Value) -> Value {
    json!({
        "source_spec_path": path,
        "expected_source_spec_hash": hash,
        "selected_acceptance_criteria_ids": ac_ids,
        "gate_policy": gates,
    })
}

fn gate_policy(gates: Vec<Value>) -> Value {
    json!({ "gates": gates })
}

fn preflight_gate(id: &str, required: bool, reusable: bool) -> Value {
    json!({
        "id": id,
        "type": "preflight",
        "required": required,
        "reusable": reusable,
        "allow_waiver": false,
    })
}

fn human_approval_gate(id: &str, reusable: bool) -> Value {
    json!({
        "id": id,
        "type": "human_approval",
        "required": true,
        "reusable": reusable,
        "allow_waiver": true,
    })
}

/// Assert the response is the InvalidInput/`workTask.specContract.invalid` spec
/// validation error and that the task still has no contract.
async fn assert_spec_invalid(
    resp: &TestResponse,
    state: &AppState,
    task_id: i32,
    events_before: usize,
) {
    assert_eq!(resp.status_code(), 400, "body={}", resp.text());
    let body: Value = resp.json();
    assert_eq!(body["code"], "invalid_input");
    assert_eq!(body["i18n_key"], "workTask.specContract.invalid");
    let events = work_task_service::list_events(&state.db.conn, task_id, 500)
        .await
        .expect("list events");
    assert_eq!(
        events.len(),
        events_before,
        "no event appended on rejection"
    );
    let contract = work_task_service::get_contract(&state.db.conn, task_id)
        .await
        .expect("get contract");
    assert!(contract.is_none(), "no contract written on rejection");
}

/// The count of `spec_contract_bound` events for a task (0 before first bind).
async fn bind_event_count(state: &AppState, task_id: i32) -> usize {
    work_task_service::list_events(&state.db.conn, task_id, 500)
        .await
        .expect("list events")
        .into_iter()
        .filter(|e| e.kind == "spec_contract_bound")
        .count()
}

/// Force the task's persisted status (test-only shortcut; the state machine's
/// own transitions are exercised elsewhere). The bind guard reads the stored
/// status, so this is the direct way to probe every forbidden state.
async fn set_status(db: &sea_orm::DatabaseConnection, task_id: i32, status: WorkTaskStatus) {
    work_task::Entity::update_many()
        .col_expr(
            work_task::Column::Status,
            Expr::value(work_task_service::status_str(status)),
        )
        .col_expr(
            work_task::Column::UpdatedAt,
            Expr::value(chrono::Utc::now()),
        )
        .filter(work_task::Column::Id.eq(task_id))
        .exec(db)
        .await
        .expect("set status");
}

// ────────────────────────────────────────────────────────────────────────────
// T01 / T02 — happy path: preview exactness, transactional bind, roundtrip
// ────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn t01_t02_preview_then_bind_stores_exact_metadata_and_roundtrips() {
    let db = fresh_in_memory_db().await;
    let project = tempfile::tempdir().expect("project root");
    write_spec(project.path(), SPEC_PATH, SPEC_BODY);
    let task_id = seed_task(&db, project.path().to_str().unwrap()).await;
    let (server, state, _data, _static) = build_server(db).await;

    // Preview returns the exact parsed identity/hash/AC without mutation.
    let resp = preview(&server, task_id, SPEC_PATH).await;
    assert_eq!(resp.status_code(), 200, "body={}", resp.text());
    let preview: Value = resp.json();
    assert_eq!(preview["source_spec_id"], "BUGRAIL-SPECOS-001");
    assert_eq!(preview["source_spec_version"], "0.3");
    assert_eq!(preview["source_spec_path"], SPEC_PATH);
    assert!(preview["source_spec_hash"].is_string());
    let hash = preview["source_spec_hash"].as_str().unwrap().to_string();
    let acs = preview["acceptance_criteria"].as_array().unwrap();
    assert_eq!(acs.len(), 2);
    assert_eq!(acs[0]["id"], "BUGRAIL-SPECOS-001.AC01");
    assert_eq!(acs[0]["title"], "AC01");
    assert!(acs[0]["text"].as_str().unwrap().contains("exact metadata"));
    assert_eq!(preview["current_binding_hash"], Value::Null);

    assert_eq!(bind_event_count(&state, task_id).await, 0);

    // Bind with the preview hash + server-resolved AC ids.
    let gates = gate_policy(vec![
        preflight_gate("ci", true, false),
        human_approval_gate("human", false),
    ]);
    let resp = bind(
        &server,
        task_id,
        draft_body(SPEC_PATH, &hash, &["BUGRAIL-SPECOS-001.AC01"], &gates),
    )
    .await;
    assert_eq!(resp.status_code(), 200, "body={}", resp.text());
    let bound: Value = resp.json();
    assert_eq!(bound["task_id"], task_id);
    assert_eq!(bound["source_spec_id"], "BUGRAIL-SPECOS-001");
    assert_eq!(bound["source_spec_version"], "0.3");
    assert_eq!(bound["source_spec_hash"], hash);
    let stored_acs = bound["acceptance_criteria"].as_array().unwrap();
    assert_eq!(stored_acs.len(), 1, "only the selected AC is snapshotted");
    assert_eq!(stored_acs[0]["id"], "BUGRAIL-SPECOS-001.AC01");
    assert_eq!(stored_acs[0]["title"], "AC01");
    assert!(stored_acs[0]["text"]
        .as_str()
        .unwrap()
        .contains("exact metadata"));

    // The bind event was recorded in the same transaction.
    let bound_events = bind_event_count(&state, task_id).await;
    assert_eq!(bound_events, 1);
    let events = work_task_service::list_events(&state.db.conn, task_id, 500)
        .await
        .expect("list events");
    let ev = events
        .iter()
        .find(|e| e.kind == "spec_contract_bound")
        .expect("bound event");
    assert_eq!(ev.actor, "user");
    let payload = ev.payload.as_ref().expect("payload");
    assert_eq!(payload["source_spec_id"], "BUGRAIL-SPECOS-001");
    assert_eq!(payload["source_spec_version"], "0.3");
    assert_eq!(payload["source_spec_hash"], hash);
    assert_ne!(payload.get("rebind"), Some(&json!(true)));

    // T02 — read the contract back; exact reference and snapshots unchanged.
    let resp = get_contract(&server, task_id).await;
    assert_eq!(resp.status_code(), 200, "body={}", resp.text());
    let got: Value = resp.json();
    assert_eq!(got["source_spec_hash"], hash);
    assert_eq!(got["acceptance_criteria"], bound["acceptance_criteria"]);
    assert_eq!(got["gate_policy"], bound["gate_policy"]);
    assert_eq!(got["created_at"], bound["created_at"]);
    assert_eq!(got["updated_at"], bound["updated_at"]);
}

// ────────────────────────────────────────────────────────────────────────────
// T03 / T04 / T05 — path confinement and size limits
// ────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn t03_path_escaping_the_project_root_is_rejected() {
    let db = fresh_in_memory_db().await;
    let project = tempfile::tempdir().expect("project root");
    write_spec(project.path(), SPEC_PATH, SPEC_BODY);
    let task_id = seed_task(&db, project.path().to_str().unwrap()).await;
    let (server, state, _data, _static) = build_server(db).await;
    let events_before = work_task_service::list_events(&state.db.conn, task_id, 500)
        .await
        .expect("list events")
        .len();

    for escaping in [
        "../outside.md",
        "../../etc/passwd",
        "nested/../../../outside.md",
        "/absolute/path/spec.md",
    ] {
        let resp = preview(&server, task_id, escaping).await;
        assert_spec_invalid(&resp, &state, task_id, events_before).await;
    }
}

#[cfg(unix)]
#[tokio::test]
async fn t04_symlink_resolving_outside_the_project_root_is_rejected() {
    let db = fresh_in_memory_db().await;
    let project = tempfile::tempdir().expect("project root");
    let outside = tempfile::tempdir().expect("outside root");
    write_spec(project.path(), SPEC_PATH, SPEC_BODY);
    // A spec inside the project that is a symlink to a file outside it.
    std::fs::write(outside.path().join("secret.md"), SPEC_BODY).expect("write secret");
    std::os::unix::fs::symlink(
        outside.path().join("secret.md"),
        project.path().join("link.md"),
    )
    .expect("symlink");
    let task_id = seed_task(&db, project.path().to_str().unwrap()).await;
    let (server, state, _data, _static) = build_server(db).await;
    let events_before = work_task_service::list_events(&state.db.conn, task_id, 500)
        .await
        .expect("list events")
        .len();

    let resp = preview(&server, task_id, "link.md").await;
    assert_spec_invalid(&resp, &state, task_id, events_before).await;
}

#[tokio::test]
async fn t05_missing_or_oversized_spec_is_rejected_before_mutation() {
    let db = fresh_in_memory_db().await;
    let project = tempfile::tempdir().expect("project root");
    write_spec(project.path(), SPEC_PATH, SPEC_BODY);
    let task_id = seed_task(&db, project.path().to_str().unwrap()).await;
    let (server, state, _data, _static) = build_server(db).await;
    let events_before = work_task_service::list_events(&state.db.conn, task_id, 500)
        .await
        .expect("list events")
        .len();

    // Missing file.
    let resp = preview(&server, task_id, ".features/does-not-exist.md").await;
    assert_spec_invalid(&resp, &state, task_id, events_before).await;

    // Over 1 MiB.
    let big =
        "---\nid: BIG\nversion: \"1\"\n---\n# x\n".to_string() + &"# padding\n".repeat(120_000);
    assert!(big.len() > 1024 * 1024, "test fixture must exceed 1 MiB");
    write_spec(project.path(), ".features/big.md", &big);
    let resp = preview(&server, task_id, ".features/big.md").await;
    assert_spec_invalid(&resp, &state, task_id, events_before).await;
}

// ────────────────────────────────────────────────────────────────────────────
// T06 — identity/version absent or malformed
// ────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn t06_spec_identity_or_version_absent_or_malformed_is_rejected() {
    let db = fresh_in_memory_db().await;
    let project = tempfile::tempdir().expect("project root");
    write_spec(project.path(), SPEC_PATH, SPEC_BODY);
    let task_id = seed_task(&db, project.path().to_str().unwrap()).await;
    let (server, state, _data, _static) = build_server(db).await;
    let events_before = work_task_service::list_events(&state.db.conn, task_id, 500)
        .await
        .expect("list events")
        .len();

    let no_front_matter = "# just a heading\n";
    let missing_id = "---\nversion: \"1.0\"\n---\n# x\n";
    let missing_version = "---\nid: X\n---\n# x\n";
    let non_scalar_version = "---\nid: X\nversion: [1, 2]\n---\n# x\n";

    for (i, body) in [
        no_front_matter,
        missing_id,
        missing_version,
        non_scalar_version,
    ]
    .into_iter()
    .enumerate()
    {
        let rel = format!(".features/bad-{i}.md");
        write_spec(project.path(), &rel, body);
        let resp = preview(&server, task_id, &rel).await;
        assert_spec_invalid(&resp, &state, task_id, events_before).await;
    }
}

// ────────────────────────────────────────────────────────────────────────────
// T07 — optimistic-concurrency: hash changes between preview and bind
// ────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn t07_spec_changed_after_preview_rejects_bind_with_no_implicit_rebind() {
    let db = fresh_in_memory_db().await;
    let project = tempfile::tempdir().expect("project root");
    write_spec(project.path(), SPEC_PATH, SPEC_BODY);
    let task_id = seed_task(&db, project.path().to_str().unwrap()).await;
    let (server, state, _data, _static) = build_server(db).await;

    let resp = preview(&server, task_id, SPEC_PATH).await;
    let hash = resp.json::<Value>()["source_spec_hash"]
        .as_str()
        .unwrap()
        .to_string();

    // The file changes after preview (author edits the spec before binding).
    let edited = SPEC_BODY.replace("exact metadata", "new metadata");
    assert_ne!(edited, SPEC_BODY);
    write_spec(project.path(), SPEC_PATH, &edited);

    let gates = gate_policy(vec![preflight_gate("ci", true, false)]);
    let resp = bind(
        &server,
        task_id,
        draft_body(SPEC_PATH, &hash, &["BUGRAIL-SPECOS-001.AC01"], &gates),
    )
    .await;
    // Stale → TaskExecutionFailed / workTask.specContract.stale (Feature Spec §7.1).
    assert_eq!(resp.status_code(), 500, "body={}", resp.text());
    let body: Value = resp.json();
    assert_eq!(body["code"], "task_execution_failed");
    assert_eq!(body["i18n_key"], "workTask.specContract.stale");

    // No implicit rebind: no contract, no event.
    assert_eq!(bind_event_count(&state, task_id).await, 0);
    let contract = work_task_service::get_contract(&state.db.conn, task_id)
        .await
        .expect("get contract");
    assert!(contract.is_none(), "stale bind must not write a contract");
}

// ────────────────────────────────────────────────────────────────────────────
// T08 — selected AC absent from source
// ────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn t08_unknown_acceptance_criterion_is_rejected() {
    let db = fresh_in_memory_db().await;
    let project = tempfile::tempdir().expect("project root");
    write_spec(project.path(), SPEC_PATH, SPEC_BODY);
    let task_id = seed_task(&db, project.path().to_str().unwrap()).await;
    let (server, state, _data, _static) = build_server(db).await;

    let resp = preview(&server, task_id, SPEC_PATH).await;
    let hash = resp.json::<Value>()["source_spec_hash"]
        .as_str()
        .unwrap()
        .to_string();
    let events_before = work_task_service::list_events(&state.db.conn, task_id, 500)
        .await
        .expect("list events")
        .len();

    let gates = gate_policy(vec![preflight_gate("ci", true, false)]);
    let resp = bind(
        &server,
        task_id,
        draft_body(SPEC_PATH, &hash, &["BUGRAIL-SPECOS-001.AC999"], &gates),
    )
    .await;
    assert_spec_invalid(&resp, &state, task_id, events_before).await;
}

// ────────────────────────────────────────────────────────────────────────────
// T09 — gate policy limits
// ────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn t09_gate_policy_limits_are_rejected() {
    let db = fresh_in_memory_db().await;
    let project = tempfile::tempdir().expect("project root");
    write_spec(project.path(), SPEC_PATH, SPEC_BODY);
    let task_id = seed_task(&db, project.path().to_str().unwrap()).await;
    let (server, state, _data, _static) = build_server(db).await;

    let resp = preview(&server, task_id, SPEC_PATH).await;
    let hash = resp.json::<Value>()["source_spec_hash"]
        .as_str()
        .unwrap()
        .to_string();
    let events_before = work_task_service::list_events(&state.db.conn, task_id, 500)
        .await
        .expect("list events")
        .len();
    let ac = "BUGRAIL-SPECOS-001.AC01";

    // Duplicate gate id.
    let dup = gate_policy(vec![
        preflight_gate("ci", true, false),
        preflight_gate("ci", true, false),
    ]);
    let resp = bind(&server, task_id, draft_body(SPEC_PATH, &hash, &[ac], &dup)).await;
    assert_spec_invalid(&resp, &state, task_id, events_before).await;

    // Reusable human approval.
    let reusable_human = gate_policy(vec![human_approval_gate("human", true)]);
    let resp = bind(
        &server,
        task_id,
        draft_body(SPEC_PATH, &hash, &[ac], &reusable_human),
    )
    .await;
    assert_spec_invalid(&resp, &state, task_id, events_before).await;

    // More than 32 gates.
    let too_many = gate_policy(
        (0..33)
            .map(|i| preflight_gate(&format!("g{i}"), true, false))
            .collect(),
    );
    let resp = bind(
        &server,
        task_id,
        draft_body(SPEC_PATH, &hash, &[ac], &too_many),
    )
    .await;
    assert_spec_invalid(&resp, &state, task_id, events_before).await;

    // Oversized policy: 32 gates each with a long id pushes past 32 KiB.
    let oversized = gate_policy(
        (0..32)
            .map(|i| preflight_gate(&format!("g{i}{}", "x".repeat(2000)), true, false))
            .collect(),
    );
    let resp = bind(
        &server,
        task_id,
        draft_body(SPEC_PATH, &hash, &[ac], &oversized),
    )
    .await;
    assert_spec_invalid(&resp, &state, task_id, events_before).await;

    // Oversized AC snapshot: one criterion text over 64 KiB, selected at bind.
    let giant_ac = format!(
        "---\nid: BIG\nversion: \"1\"\n---\n## 8. Acceptance Criteria\n\
         | ID | Criterion |\n|---|---|\n| `BIG.AC01` | {} |\n",
        "x".repeat(70 * 1024)
    );
    write_spec(project.path(), ".features/big-ac.md", &giant_ac);
    let resp = preview(&server, task_id, ".features/big-ac.md").await;
    assert_eq!(
        resp.status_code(),
        200,
        "preview must accept a sub-1MiB spec"
    );
    let big_hash = resp.json::<Value>()["source_spec_hash"]
        .as_str()
        .unwrap()
        .to_string();
    let gates = gate_policy(vec![preflight_gate("ci", true, false)]);
    let resp = bind(
        &server,
        task_id,
        draft_body(".features/big-ac.md", &big_hash, &["BIG.AC01"], &gates),
    )
    .await;
    assert_spec_invalid(&resp, &state, task_id, events_before).await;

    // Unsupported gate type fails at wire deserialization (422 from axum's Json
    // rejection — before the core runs, so nothing is written).
    let unsupported = json!({ "gates": [{ "id": "x", "type": "magic", "required": true, "reusable": false, "allow_waiver": false }] });
    let resp = bind(
        &server,
        task_id,
        draft_body(SPEC_PATH, &hash, &[ac], &unsupported),
    )
    .await;
    assert_eq!(resp.status_code(), 422, "body={}", resp.text());
    assert_eq!(
        bind_event_count(&state, task_id).await,
        0,
        "no event on deserialize failure"
    );
    let contract = work_task_service::get_contract(&state.db.conn, task_id)
        .await
        .expect("get contract");
    assert!(contract.is_none(), "no contract on deserialize failure");
}

// ────────────────────────────────────────────────────────────────────────────
// T25 foundation — staleness after binding (the merge/complete *decision* that
// consumes this fact lands in issue-003; the fact itself is issue-002)
// ────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn t25_staleness_detects_spec_change_after_binding() {
    let db = fresh_in_memory_db().await;
    let project = tempfile::tempdir().expect("project root");
    write_spec(project.path(), SPEC_PATH, SPEC_BODY);
    let task_id = seed_task(&db, project.path().to_str().unwrap()).await;
    let (server, state, _data, _static) = build_server(db).await;

    let resp = preview(&server, task_id, SPEC_PATH).await;
    let hash = resp.json::<Value>()["source_spec_hash"]
        .as_str()
        .unwrap()
        .to_string();
    let gates = gate_policy(vec![preflight_gate("ci", true, false)]);
    let resp = bind(
        &server,
        task_id,
        draft_body(SPEC_PATH, &hash, &["BUGRAIL-SPECOS-001.AC01"], &gates),
    )
    .await;
    assert_eq!(resp.status_code(), 200, "body={}", resp.text());

    // Unchanged file → not stale.
    assert!(!work_task_spec_staleness_core(&state.db, task_id)
        .await
        .expect("staleness"));

    // Author edits the spec → stale.
    write_spec(
        project.path(),
        SPEC_PATH,
        &SPEC_BODY.replace("exact metadata", "new metadata"),
    );
    assert!(work_task_spec_staleness_core(&state.db, task_id)
        .await
        .expect("staleness"));

    // Restore the exact bytes → not stale again.
    write_spec(project.path(), SPEC_PATH, SPEC_BODY);
    assert!(!work_task_spec_staleness_core(&state.db, task_id)
        .await
        .expect("staleness"));

    // The file goes away → stale (merge gating must never proceed on an
    // unverifiable reference).
    std::fs::remove_file(project.path().join(SPEC_PATH)).expect("remove spec");
    assert!(work_task_spec_staleness_core(&state.db, task_id)
        .await
        .expect("staleness"));
}

// ────────────────────────────────────────────────────────────────────────────
// T26 — rebind in `review` succeeds and preserves history; every forbidden
// state rejects without mutation
// ────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn t26_rebind_in_review_succeeds_forbidden_states_reject_without_mutation() {
    let db = fresh_in_memory_db().await;
    let project = tempfile::tempdir().expect("project root");
    write_spec(project.path(), SPEC_PATH, SPEC_BODY);
    let task_id = seed_task(&db, project.path().to_str().unwrap()).await;
    let (server, state, _data, _static) = build_server(db).await;

    // First bind (todo state).
    let resp = preview(&server, task_id, SPEC_PATH).await;
    let hash1 = resp.json::<Value>()["source_spec_hash"]
        .as_str()
        .unwrap()
        .to_string();
    let gates = gate_policy(vec![preflight_gate("ci", true, false)]);
    let resp = bind(
        &server,
        task_id,
        draft_body(SPEC_PATH, &hash1, &["BUGRAIL-SPECOS-001.AC01"], &gates),
    )
    .await;
    assert_eq!(resp.status_code(), 200, "body={}", resp.text());

    // A rebind with a different spec version (file unchanged, same hash is fine,
    // but simulate the author bumping the file to a new version).
    let v2 = SPEC_BODY.replace("version: \"0.3\"", "version: \"0.4\"");
    write_spec(project.path(), SPEC_PATH, &v2);
    let resp = preview(&server, task_id, SPEC_PATH).await;
    assert_eq!(resp.status_code(), 200, "body={}", resp.text());
    let hash2 = resp.json::<Value>()["source_spec_hash"]
        .as_str()
        .unwrap()
        .to_string();
    assert_ne!(hash1, hash2);

    // Forbidden states: a rebind while a generation is in flight is rejected
    // and leaves the contract + timeline untouched.
    for status in [
        WorkTaskStatus::Queued,
        WorkTaskStatus::Preparing,
        WorkTaskStatus::Running,
        WorkTaskStatus::AwaitingInput,
        WorkTaskStatus::Merging,
        WorkTaskStatus::Done,
    ] {
        set_status(&state.db.conn, task_id, status).await;
        let events_before = bind_event_count(&state, task_id).await;
        let resp = bind(
            &server,
            task_id,
            draft_body(SPEC_PATH, &hash2, &["BUGRAIL-SPECOS-001.AC01"], &gates),
        )
        .await;
        assert_eq!(
            resp.status_code(),
            500,
            "state {status:?} must reject rebind: body={}",
            resp.text()
        );
        let body: Value = resp.json();
        assert_eq!(body["code"], "task_execution_failed");
        assert_eq!(bind_event_count(&state, task_id).await, events_before);
        let contract = work_task_service::get_contract(&state.db.conn, task_id)
            .await
            .expect("get contract")
            .expect("bound contract");
        assert_eq!(contract.source_spec_hash, hash1, "no rebind in {status:?}");
    }

    // Rebind in review succeeds; the old hash is preserved in the timeline.
    set_status(&state.db.conn, task_id, WorkTaskStatus::Review).await;
    let events_before = bind_event_count(&state, task_id).await;
    let resp = bind(
        &server,
        task_id,
        draft_body(SPEC_PATH, &hash2, &["BUGRAIL-SPECOS-001.AC02"], &gates),
    )
    .await;
    assert_eq!(resp.status_code(), 200, "body={}", resp.text());
    let rebind: Value = resp.json();
    assert_eq!(rebind["source_spec_hash"], hash2);
    assert_eq!(rebind["source_spec_version"], "0.4");
    let stored_acs = rebind["acceptance_criteria"].as_array().unwrap();
    assert_eq!(stored_acs[0]["id"], "BUGRAIL-SPECOS-001.AC02");

    assert_eq!(bind_event_count(&state, task_id).await, events_before + 1);
    let events = work_task_service::list_events(&state.db.conn, task_id, 500)
        .await
        .expect("list events");
    let ev = events
        .iter()
        .rev()
        .find(|e| e.kind == "spec_contract_bound")
        .expect("rebind event");
    let payload = ev.payload.as_ref().expect("payload");
    assert_eq!(payload["rebind"], true);
    assert_eq!(payload["previous_source_spec_hash"], hash1);
    assert_eq!(payload["source_spec_hash"], hash2);

    // The rebind keeps the original creation time; only the reference moved.
    let contract = work_task_service::get_contract(&state.db.conn, task_id)
        .await
        .expect("get contract")
        .expect("rebound contract");
    assert_eq!(contract.source_spec_hash, hash2);
    assert!(contract.updated_at >= contract.created_at);
}

// T14 / T15 — the public transport exposes only policy-bound human decisions;
// engine-owned results and forbidden waivers cannot be forged by a client.
#[tokio::test]
async fn t14_t15_human_gate_transport_enforces_actor_policy_and_reason() {
    let db = fresh_in_memory_db().await;
    let project = tempfile::tempdir().expect("project root");
    write_spec(project.path(), SPEC_PATH, SPEC_BODY);
    let task_id = seed_task(&db, project.path().to_str().unwrap()).await;
    let (server, _state, _data, _static) = build_server(db).await;

    let resp = preview(&server, task_id, SPEC_PATH).await;
    let hash = resp.json::<Value>()["source_spec_hash"]
        .as_str()
        .unwrap()
        .to_string();
    let gates = gate_policy(vec![
        preflight_gate("preflight", true, false),
        human_approval_gate("human", false),
    ]);
    let resp = bind(
        &server,
        task_id,
        draft_body(SPEC_PATH, &hash, &["BUGRAIL-SPECOS-001.AC01"], &gates),
    )
    .await;
    assert_eq!(resp.status_code(), 200, "body={}", resp.text());

    let decide = |gate_id: &str, decision: &str, reason: Option<&str>| {
        server
            .post("/api/work_task_gate_human_decide")
            .add_header("authorization", format!("Bearer {TEST_TOKEN}"))
            .json(&json!({
                "taskId": task_id,
                "gateId": gate_id,
                "decision": decision,
                "reason": reason,
            }))
    };

    let resp = decide("preflight", "approve", Some("forged agent pass")).await;
    assert_eq!(resp.status_code(), 400, "body={}", resp.text());
    let resp = decide("preflight", "waive", Some("not allowed")).await;
    assert_eq!(resp.status_code(), 403, "body={}", resp.text());
    let resp = decide("human", "approve", Some("   ")).await;
    assert_eq!(resp.status_code(), 400, "body={}", resp.text());

    let resp = decide("human", "approve", Some("reviewed by owner")).await;
    assert_eq!(resp.status_code(), 200, "body={}", resp.text());
    let result: Value = resp.json();
    assert_eq!(result["actor"], "user");
    assert_eq!(result["status"], "passed");
    assert_eq!(result["reason"], "reviewed by owner");

    let resp = decide("human", "waive", Some("accepted residual risk")).await;
    assert_eq!(resp.status_code(), 200, "body={}", resp.text());
    let waived: Value = resp.json();
    assert_eq!(waived["actor"], "user");
    assert_eq!(waived["status"], "waived");
    assert_eq!(waived["reason"], "accepted residual risk");
    assert!(waived["started_at"].is_string());
    assert!(waived["finished_at"].is_string());

    let resp = server
        .post("/api/work_task_gate_list")
        .add_header("authorization", format!("Bearer {TEST_TOKEN}"))
        .json(&json!({ "taskId": task_id }))
        .await;
    assert_eq!(resp.status_code(), 200, "body={}", resp.text());
    let rows: Value = resp.json();
    let rows = rows.as_array().unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows.last().unwrap()["status"], "waived");
    assert_eq!(rows.last().unwrap()["actor"], "user");
    assert_eq!(rows.last().unwrap()["reason"], "accepted residual risk");

    let resp = server
        .post("/api/work_task_gate_record")
        .add_header("authorization", format!("Bearer {TEST_TOKEN}"))
        .json(&json!({
            "taskId": task_id,
            "runSeq": 0,
            "gateId": "preflight",
            "gateType": "preflight",
            "status": "passed",
            "required": true,
            "reusable": false,
            "actor": "agent",
            "evidence": { "summary": "forged" },
            "reason": "forged public writer"
        }))
        .await;
    assert_eq!(resp.status_code(), 501, "no generic gate-record route");
    let resp = server
        .post("/api/work_task_gate_list")
        .add_header("authorization", format!("Bearer {TEST_TOKEN}"))
        .json(&json!({ "taskId": task_id }))
        .await;
    let rows: Value = resp.json();
    assert_eq!(
        rows.as_array().unwrap().len(),
        2,
        "forged route wrote no row"
    );
}

// T16 and the status branches of T17-T19 — every ineligible gate state blocks
// the merge path before engine dispatch and leaves the reviewed task untouched.
// The T17 missing-producer branch and T20 real unchanged-worktree cleanup need
// engine-level fixtures, so this test intentionally does not claim them.
#[tokio::test]
async fn merge_rejects_missing_running_failed_and_blocked_gate_states() {
    let db = fresh_in_memory_db().await;
    let project = tempfile::tempdir().expect("project root");
    write_spec(project.path(), SPEC_PATH, SPEC_BODY);
    let task_id = seed_task(&db, project.path().to_str().unwrap()).await;
    let (server, state, _data, _static) = build_server(db).await;

    let resp = preview(&server, task_id, SPEC_PATH).await;
    let hash = resp.json::<Value>()["source_spec_hash"]
        .as_str()
        .unwrap()
        .to_string();
    let gates = gate_policy(vec![preflight_gate("preflight", true, false)]);
    let resp = bind(
        &server,
        task_id,
        draft_body(SPEC_PATH, &hash, &["BUGRAIL-SPECOS-001.AC01"], &gates),
    )
    .await;
    assert_eq!(resp.status_code(), 200, "body={}", resp.text());
    set_status(&state.db.conn, task_id, WorkTaskStatus::Review).await;

    for gate_status in [
        None,
        Some(WorkTaskGateStatus::Running),
        Some(WorkTaskGateStatus::Failed),
        Some(WorkTaskGateStatus::Blocked),
    ] {
        if let Some(status) = gate_status {
            let evidence = (status == WorkTaskGateStatus::Failed)
                .then(|| json!({ "summary": "preflight command failed" }));
            work_task_service::record_preflight_gates(
                &state.db.conn,
                task_id,
                0,
                status,
                (status != WorkTaskGateStatus::Running).then(|| "evidence failed".into()),
                evidence,
            )
            .await
            .expect("record engine-owned gate result");
        }

        let resp = server
            .post("/api/work_task_merge")
            .add_header("authorization", format!("Bearer {TEST_TOKEN}"))
            .json(&json!({
                "id": task_id,
                "message": null,
                "deleteWorktree": false
            }))
            .await;
        assert_eq!(resp.status_code(), 500, "body={}", resp.text());
        let body: Value = resp.json();
        assert_eq!(body["i18n_key"], "workTask.qualityGate.unmet");
        assert_eq!(body["i18n_params"]["gates"], "preflight");
        let task = work_task_service::get_model(&state.db.conn, task_id)
            .await
            .expect("task remains");
        assert_eq!(task.status, WorkTaskStatus::Review);

        if gate_status == Some(WorkTaskGateStatus::Failed) {
            let attempts = work_task_service::list_gate_results(&state.db.conn, task_id, Some(0))
                .await
                .expect("list persisted gate evidence");
            let failed = attempts.last().expect("failed attempt persisted");
            assert_eq!(failed.status, "failed");
            let evidence: Value = serde_json::from_str(
                failed
                    .evidence
                    .as_deref()
                    .expect("failed evidence persisted"),
            )
            .expect("failed evidence is valid JSON");
            assert_eq!(
                evidence["summary"].as_str(),
                Some("preflight command failed")
            );
        }
    }

    let events = work_task_service::list_events(&state.db.conn, task_id, 100)
        .await
        .expect("list events");
    assert_eq!(
        events
            .iter()
            .filter(|event| event.kind == "quality_gate_blocked")
            .count(),
        4
    );
}
