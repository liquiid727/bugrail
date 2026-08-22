//! Engine-level fixtures for the 2026-08-23 SPECOS-001 verification gaps.
//!
//! The Axum suites in `specos_spec_binding.rs` cover the transport contract;
//! the three scenarios here need to drive the engine/command-core directly:
//!
//! - `T17`: a contract-bound task with a required preflight gate and no
//!   configured producer must persist a `blocked` gate result carrying the
//!   `producer_unavailable` reason (Feature Spec §5.2) and keep the merge
//!   path rejected.
//! - `T20`: a real, unchanged Git worktree must observe a rejected completion
//!   (unmet gate) with `deleteWorktree=true` and prove there is no cleanup
//!   side effect — directory, folder row and task pointers all survive.
//! - `T27`: `work_task_contract_bind_core` must detect a Spec change that
//!   lands between validation and persistence atomically — a stale draft
//!   writes no contract, no gate rows and no events — and a successful bind
//!   persists one internally consistent (id, version, hash) snapshot.

use std::path::Path;
use std::process::Command;

use axum_test::TestServer;
use codeg_lib::app_state::AppState;
use codeg_lib::commands::work_task::{
    work_task_contract_bind_core, work_task_contract_preview_core,
};
use codeg_lib::db::entities::{folder, work_task};
use codeg_lib::db::service::work_task_service;
use codeg_lib::db::test_helpers::fresh_in_memory_db;
use codeg_lib::db::AppDatabase;
use codeg_lib::models::{WorkTaskContractDraft, WorkTaskDraft, WorkTaskStatus};
use codeg_lib::web::router::build_router;
use codeg_lib::web::shutdown::ShutdownSignal;
use codeg_lib::work_task::engine;
use codeg_lib::work_task::spec_reader;
use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::{json, Value};
use std::sync::Arc;

const TEST_TOKEN: &str = "spec-engine-gaps-test-token";
const SPEC_PATH: &str = ".features/BUGRAIL-SPECOS-001-work-task-quality/spec.md";

/// Mirrors the valid front matter + AC table shape used by the binding suites.
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
";

// ────────────────────────────────────────────────────────────────────────────
// Setup helpers (same idioms as specos_spec_binding.rs)
// ────────────────────────────────────────────────────────────────────────────

fn write_spec(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().expect("spec parent")).expect("create dirs");
    std::fs::write(&path, body).expect("write spec");
}

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

async fn build_server(db: AppDatabase) -> (TestServer, Arc<AppState>) {
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
    (server, state)
}

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

fn required_preflight_gate() -> Value {
    json!({
        "gates": [{
            "id": "preflight",
            "type": "preflight",
            "required": true,
            "reusable": false,
            "allow_waiver": false,
        }]
    })
}

/// Preview + bind a contract with one required preflight gate through the real
/// Axum router, leaving the task in `review` with the gate permanently unmet.
async fn bind_required_preflight_and_enter_review(server: &TestServer, task_id: i32) {
    let resp = server
        .post("/api/work_task_contract_preview")
        .add_header("authorization", format!("Bearer {TEST_TOKEN}"))
        .json(&json!({ "taskId": task_id, "sourceSpecPath": SPEC_PATH }))
        .await;
    assert_eq!(resp.status_code(), 200, "preview body={}", resp.text());
    let hash = resp.json::<Value>()["source_spec_hash"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = server
        .post("/api/work_task_contract_bind")
        .add_header("authorization", format!("Bearer {TEST_TOKEN}"))
        .json(&json!({
            "taskId": task_id,
            "draft": {
                "source_spec_path": SPEC_PATH,
                "expected_source_spec_hash": hash,
                "selected_acceptance_criteria_ids": ["BUGRAIL-SPECOS-001.AC01"],
                "gate_policy": required_preflight_gate(),
            }
        }))
        .await;
    assert_eq!(resp.status_code(), 200, "bind body={}", resp.text());
}

fn run_git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .output()
        .expect("spawn git");
    assert!(
        out.status.success(),
        "git {:?} in {} failed: {}{}",
        args,
        dir.display(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}

// ────────────────────────────────────────────────────────────────────────────
// T17 — required preflight with no configured producer
// ────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn t17_required_preflight_without_producer_persists_blocked_result() {
    let db = fresh_in_memory_db().await;
    let project = tempfile::tempdir().expect("project root");
    write_spec(project.path(), SPEC_PATH, SPEC_BODY);
    let task_id = seed_task(&db, project.path().to_str().unwrap()).await;
    let (server, state) = build_server(db).await;
    bind_required_preflight_and_enter_review(&server, task_id).await;
    set_status(&state.db.conn, task_id, WorkTaskStatus::Review).await;

    // Default folder settings carry no free-form preflight command and no
    // command reference: the producer is unavailable. Drive the engine's own
    // post-review preflight path (not the HTTP shortcut).
    let data_dir = tempfile::tempdir().expect("engine data dir");
    let engine = engine::new_for_test(
        AppDatabase {
            conn: state.db.conn.clone(),
        },
        codeg_lib::web::event_bridge::EventEmitter::Noop,
        data_dir.path().to_path_buf(),
    );
    engine.run_preflight(task_id, 0).await;

    let results = work_task_service::list_gate_results(&state.db.conn, task_id, Some(0))
        .await
        .expect("list gate results");
    let row = results
        .iter()
        .find(|r| r.gate_id == "preflight")
        .expect("preflight gate result persisted")
        .clone();
    assert_eq!(row.status, "blocked");
    assert_eq!(row.reason.as_deref(), Some("producer_unavailable"));
    assert_eq!(row.actor, "engine");
    assert!(
        row.finished_at.is_some(),
        "blocked is terminal — finished_at must be set"
    );

    // The persisted blocked state keeps the merge path rejected (T16 parity).
    let resp = server
        .post("/api/work_task_merge")
        .add_header("authorization", format!("Bearer {TEST_TOKEN}"))
        .json(&json!({ "id": task_id, "message": null, "deleteWorktree": false }))
        .await;
    assert_eq!(resp.status_code(), 500, "body={}", resp.text());
    let body: Value = resp.json();
    assert_eq!(body["i18n_key"], "workTask.qualityGate.unmet");
    assert_eq!(body["i18n_params"]["gates"], "preflight");
    let task = work_task_service::get_model(&state.db.conn, task_id)
        .await
        .expect("task remains");
    assert_eq!(task.status, WorkTaskStatus::Review);
}

// ────────────────────────────────────────────────────────────────────────────
// T20 — rejected completion on an unchanged real worktree, deleteWorktree=true
// ────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn t20_rejected_complete_with_delete_worktree_has_no_cleanup_side_effect() {
    let db = fresh_in_memory_db().await;
    let project = tempfile::tempdir().expect("project root");
    let repo = project.path().join("repo");
    std::fs::create_dir_all(&repo).expect("create repo dir");
    run_git(&repo, &["init", "-q"]);
    run_git(&repo, &["config", "user.email", "t20@bugrail.test"]);
    run_git(&repo, &["config", "user.name", "Bugrail T20"]);
    std::fs::write(repo.join("README.md"), "base\n").expect("seed file");
    run_git(&repo, &["add", "-A"]);
    run_git(&repo, &["commit", "-q", "-m", "base"]);

    // A real linked worktree on an untouched branch: `has_landable_changes`
    // would find nothing, so without the gate this completion (with
    // deleteWorktree) would proceed to cleanup.
    let wt_root = tempfile::tempdir().expect("worktree root");
    let wt_path = wt_root.path().join("wt");
    run_git(
        &repo,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "work-branch",
            wt_path.to_str().unwrap(),
        ],
    );

    write_spec(&repo, SPEC_PATH, SPEC_BODY);
    let task_id = seed_task(&db, repo.to_str().unwrap()).await;
    let wt_folder_id =
        codeg_lib::db::service::folder_service::add_folder(&db.conn, wt_path.to_str().unwrap())
            .await
            .expect("add worktree folder")
            .id;
    work_task::Entity::update_many()
        .col_expr(
            work_task::Column::WorktreeFolderId,
            Expr::value(wt_folder_id),
        )
        .col_expr(work_task::Column::WorkBranch, Expr::value("work-branch"))
        .col_expr(
            work_task::Column::UpdatedAt,
            Expr::value(chrono::Utc::now()),
        )
        .filter(work_task::Column::Id.eq(task_id))
        .exec(&db.conn)
        .await
        .expect("attach worktree to task");

    let (server, state) = build_server(db).await;
    bind_required_preflight_and_enter_review(&server, task_id).await;
    set_status(&state.db.conn, task_id, WorkTaskStatus::Review).await;
    let events_before = work_task_service::list_events(&state.db.conn, task_id, 500)
        .await
        .expect("list events before")
        .len();

    let resp = server
        .post("/api/work_task_complete")
        .add_header("authorization", format!("Bearer {TEST_TOKEN}"))
        .json(&json!({ "id": task_id, "deleteWorktree": true }))
        .await;
    assert_eq!(resp.status_code(), 500, "body={}", resp.text());
    let body: Value = resp.json();
    assert_eq!(body["i18n_key"], "workTask.qualityGate.unmet");
    assert_eq!(body["i18n_params"]["gates"], "preflight");

    // No state side effect: the task never left review.
    let task = work_task_service::get_model(&state.db.conn, task_id)
        .await
        .expect("task remains");
    assert_eq!(task.status, WorkTaskStatus::Review);
    assert_eq!(task.worktree_folder_id, Some(wt_folder_id));
    assert_eq!(task.work_branch.as_deref(), Some("work-branch"));
    assert!(task.cleanup_state.is_none(), "no cleanup bookkeeping");

    // No cleanup side effect on disk or in the folder registry: the worktree
    // directory survives and its folder row is still live.
    assert!(wt_path.exists(), "worktree directory must survive");
    let wt_folder = folder::Entity::find_by_id(wt_folder_id)
        .one(&state.db.conn)
        .await
        .expect("find worktree folder")
        .expect("worktree folder row survives");
    assert!(wt_folder.deleted_at.is_none());

    // The only observable write is the auditable block event.
    let events = work_task_service::list_events(&state.db.conn, task_id, 500)
        .await
        .expect("list events after");
    let new_events: Vec<_> = events.iter().skip(events_before).collect();
    assert_eq!(new_events.len(), 1, "exactly the block event");
    assert_eq!(new_events[0].kind, "quality_gate_blocked");
}

// ────────────────────────────────────────────────────────────────────────────
// T27 — atomic spec-change behavior at the command core
// ────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn t27_bind_core_rejects_mid_window_spec_change_and_persists_one_snapshot() {
    let db = fresh_in_memory_db().await;
    let project = tempfile::tempdir().expect("project root");
    write_spec(project.path(), SPEC_PATH, SPEC_BODY);
    let task_id = seed_task(&db, project.path().to_str().unwrap()).await;

    let preview = work_task_contract_preview_core(&db, task_id, SPEC_PATH.to_string())
        .await
        .expect("preview");

    // The Spec changes between validation and persistence: the draft still
    // carries the preview hash, so the core must reject atomically.
    let edited = SPEC_BODY.replace("exact metadata", "edited metadata");
    write_spec(project.path(), SPEC_PATH, &edited);
    let stale_draft: WorkTaskContractDraft = serde_json::from_value(json!({
        "source_spec_path": SPEC_PATH,
        "expected_source_spec_hash": preview.source_spec_hash,
        "selected_acceptance_criteria_ids": ["BUGRAIL-SPECOS-001.AC01"],
        "gate_policy": required_preflight_gate(),
    }))
    .expect("draft deserializes");
    let err = work_task_contract_bind_core(
        &codeg_lib::web::event_bridge::EventEmitter::Noop,
        &db,
        task_id,
        stale_draft,
    )
    .await
    .expect_err("stale draft must be rejected");
    assert_eq!(
        err.i18n_key.as_deref(),
        Some(spec_reader::SPEC_CONTRACT_STALE_I18N)
    );

    // Atomicity: no contract, no gate rows, no bind event survived the reject.
    assert!(work_task_service::get_contract(&db.conn, task_id)
        .await
        .expect("get contract")
        .is_none());
    assert!(
        work_task_service::list_gate_results(&db.conn, task_id, None)
            .await
            .expect("gate results")
            .is_empty()
    );
    let bound_events = work_task_service::list_events(&db.conn, task_id, 500)
        .await
        .expect("list events")
        .iter()
        .filter(|e| e.kind == "spec_contract_bound")
        .count();
    assert_eq!(bound_events, 0);

    // A fresh preview of the edited Spec binds, and the persisted row is one
    // internally consistent snapshot: (id, version, hash) all come from the
    // same validated read, never mixed with the stale draft.
    let fresh = work_task_contract_preview_core(&db, task_id, SPEC_PATH.to_string())
        .await
        .expect("re-preview");
    assert_ne!(fresh.source_spec_hash, preview.source_spec_hash);
    let draft: WorkTaskContractDraft = serde_json::from_value(json!({
        "source_spec_path": SPEC_PATH,
        "expected_source_spec_hash": fresh.source_spec_hash,
        "selected_acceptance_criteria_ids": ["BUGRAIL-SPECOS-001.AC01"],
        "gate_policy": required_preflight_gate(),
    }))
    .expect("draft deserializes");
    let info = work_task_contract_bind_core(
        &codeg_lib::web::event_bridge::EventEmitter::Noop,
        &db,
        task_id,
        draft,
    )
    .await
    .expect("fresh bind succeeds");

    let reread = spec_reader::read_spec_reference(project.path(), SPEC_PATH).expect("re-read spec");
    assert_eq!(info.source_spec_hash, reread.sha256);
    assert_eq!(info.source_spec_id, reread.id);
    assert_eq!(info.source_spec_version, reread.version);
    assert_ne!(info.source_spec_hash, preview.source_spec_hash);
}
