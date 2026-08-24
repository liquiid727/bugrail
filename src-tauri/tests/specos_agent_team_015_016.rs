//! Independent verification oracles for BUGRAIL-SPECOS-015/016.
//!
//! These tests deliberately use the real SQLite schema, shared command core,
//! and Axum router. No test creates an alternate Team runtime or scheduler.

use axum_test::TestServer;
use codeg_lib::app_state::AppState;
use codeg_lib::commands::specos_control as core;
use codeg_lib::db::entities::work_task;
use codeg_lib::db::service::{specos_runtime_service, work_task_service};
use codeg_lib::db::test_helpers::{fresh_in_memory_db, seed_folder};
use codeg_lib::db::AppDatabase;
use codeg_lib::models::{
    AgentCatalog, AgentProfile, ModelProfile, TeamCatalog, TeamDefinition, TeamWorkflowDefinition,
    WorkflowNodeDefinition,
};
use codeg_lib::specos_control;
use codeg_lib::web::event_bridge::EventEmitter;
use codeg_lib::web::router::build_router;
use codeg_lib::web::shutdown::ShutdownSignal;
use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::Arc;
use tempfile::TempDir;

const TEST_TOKEN: &str = "specos-team-015-016-test-token";

fn profile(id: &str) -> AgentProfile {
    AgentProfile {
        id: id.into(),
        name: id.into(),
        runtime_adapter: "codex".into(),
        model_profile_id: Some("shared".into()),
        mode_id: None,
        reasoning: Some("medium".into()),
        context_loadout_id: None,
        skills: vec![],
        rules: vec![],
        tools: vec![],
        config_values: BTreeMap::new(),
        enabled: true,
    }
}

fn agents() -> AgentCatalog {
    AgentCatalog {
        version: 1,
        default_agent_profile_id: Some("planner".into()),
        model_profiles: vec![ModelProfile {
            id: "shared".into(),
            name: "Shared".into(),
            provider_ref: None,
            model: "gpt-test".into(),
            reasoning: Some("medium".into()),
            fallback_profile_ids: vec![],
        }],
        agent_profiles: vec![profile("planner"), profile("reviewer")],
        validation_errors: vec![],
    }
}

fn node(id: &str, depends_on: &[&str]) -> WorkflowNodeDefinition {
    WorkflowNodeDefinition {
        id: id.into(),
        title: id.into(),
        prompt: format!("Execute {id}"),
        agent_profile_id: if id == "review" {
            "reviewer".into()
        } else {
            "planner".into()
        },
        model_profile_id: None,
        context_loadout_id: None,
        depends_on: depends_on.iter().map(|dep| (*dep).into()).collect(),
    }
}

fn catalog(max_concurrent: i32) -> TeamCatalog {
    TeamCatalog {
        version: 1,
        teams: vec![TeamDefinition {
            id: "delivery".into(),
            name: "Delivery".into(),
            description: "A real WorkTask-backed team".into(),
            member_profile_ids: vec!["planner".into(), "reviewer".into()],
        }],
        workflows: vec![TeamWorkflowDefinition {
            id: "delivery-flow".into(),
            name: "Delivery flow".into(),
            version: 1,
            team_id: "delivery".into(),
            max_concurrent,
            nodes: vec![
                node("root-a", &[]),
                node("root-b", &[]),
                node("review", &["root-a"]),
            ],
        }],
        validation_errors: vec![],
    }
}

async fn fixture() -> (AppDatabase, TempDir, i32) {
    let root = tempfile::tempdir().expect("project root");
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, root.path().to_str().expect("utf8 project root")).await;
    specos_control::save_agents(root.path(), agents()).expect("save agents");
    (db, root, folder_id)
}

async fn set_status(db: &AppDatabase, task_id: i32, status: work_task::WorkTaskStatus) {
    work_task::Entity::update_many()
        .col_expr(work_task::Column::Status, Expr::value(status))
        .filter(work_task::Column::Id.eq(task_id))
        .exec(&db.conn)
        .await
        .expect("set task status");
}

fn task_ids(bindings: &[codeg_lib::db::entities::team_run_task::Model]) -> BTreeMap<String, i32> {
    bindings
        .iter()
        .map(|binding| (binding.node_id.clone(), binding.task_id))
        .collect()
}

async fn build_server(db: AppDatabase) -> (TestServer, Arc<AppState>, TempDir, TempDir) {
    let data_dir = tempfile::tempdir().expect("data dir");
    let static_dir = tempfile::tempdir().expect("static dir");
    let state = Arc::new(AppState::new_for_test(db, data_dir.path().to_path_buf()));
    let router = build_router(
        state.clone(),
        TEST_TOKEN.to_string(),
        static_dir.path().to_path_buf(),
        Arc::new(ShutdownSignal::new()),
    );
    (
        TestServer::new(router).expect("axum test server"),
        state,
        data_dir,
        static_dir,
    )
}

#[tokio::test]
async fn t015_real_start_orders_dag_and_reserves_team_concurrency() {
    let (db, _root, folder_id) = fixture().await;
    let saved = core::team_catalog_save_core(&db, folder_id, catalog(1))
        .await
        .expect("shared core accepts the valid catalog");
    assert_eq!(saved.workflows[0].max_concurrent, 1);

    let run =
        core::team_run_start_core(&EventEmitter::Noop, &db, folder_id, "delivery-flow".into())
            .await
            .expect("start through command core");
    assert_eq!(run.status, "queued");
    assert_eq!(run.nodes.len(), 3);
    assert!(run
        .nodes
        .iter()
        .all(|node| node.status == "queued" && node.run_seq == 1));

    let bindings = specos_runtime_service::team_run_tasks(&db.conn, &run.id)
        .await
        .expect("team bindings");
    let ids = task_ids(&bindings);
    let first = work_task_service::next_queued(&db.conn, folder_id, &[])
        .await
        .expect("first queue read")
        .expect("one root is runnable")
        .id;
    assert_eq!(first, ids["root-a"]);

    set_status(&db, ids["root-a"], work_task::WorkTaskStatus::Preparing).await;
    assert!(
        work_task_service::next_queued(&db.conn, folder_id, &[])
            .await
            .expect("concurrency queue read")
            .is_none(),
        "the second root must not exceed maxConcurrent=1"
    );

    set_status(&db, ids["root-a"], work_task::WorkTaskStatus::Done).await;
    let second = work_task_service::next_queued(&db.conn, folder_id, &[])
        .await
        .expect("parallel root queue read")
        .expect("the independent root is now runnable")
        .id;
    assert_eq!(second, ids["root-b"]);

    set_status(&db, ids["root-b"], work_task::WorkTaskStatus::Done).await;
    let dependent = work_task_service::next_queued(&db.conn, folder_id, &[])
        .await
        .expect("sequential queue read")
        .expect("the dependent node is now runnable")
        .id;
    assert_eq!(dependent, ids["review"]);
}

#[tokio::test]
async fn t016_controls_are_idempotent_cas_and_engine_absence_is_explicit() {
    let (db, _root, folder_id) = fixture().await;
    core::team_catalog_save_core(&db, folder_id, catalog(2))
        .await
        .expect("save catalog");
    let run =
        core::team_run_start_core(&EventEmitter::Noop, &db, folder_id, "delivery-flow".into())
            .await
            .expect("start run");

    core::team_run_control_core(&db, run.id.clone(), "pause".into())
        .await
        .expect("pause");
    core::team_run_control_core(&db, run.id.clone(), "pause".into())
        .await
        .expect("repeated pause is idempotent");
    assert_eq!(
        specos_runtime_service::team_run_control_state(&db.conn, &run.id)
            .await
            .unwrap(),
        "paused"
    );
    let bindings = specos_runtime_service::team_run_tasks(&db.conn, &run.id)
        .await
        .unwrap();
    let first_task = bindings[0].task_id;
    let task = work_task_service::get_model(&db.conn, first_task)
        .await
        .unwrap();
    assert!(
        !work_task_service::begin_setup(&db.conn, first_task, task.run_seq)
            .await
            .unwrap()
    );

    core::team_run_control_core(&db, run.id.clone(), "resume".into())
        .await
        .expect("resume");
    core::team_run_control_core(&db, run.id.clone(), "resume".into())
        .await
        .expect("repeated resume is idempotent");
    assert_eq!(
        specos_runtime_service::team_run_control_state(&db.conn, &run.id)
            .await
            .unwrap(),
        "running"
    );

    let cancel_without_engine = core::team_run_control_core(&db, run.id.clone(), "cancel".into())
        .await
        .expect_err("cancel must not claim success without TaskEngine");
    assert!(cancel_without_engine.to_string().contains("task engine"));
    assert_eq!(
        specos_runtime_service::team_run_control_state(&db.conn, &run.id)
            .await
            .unwrap(),
        "running"
    );

    specos_runtime_service::set_team_control(&db.conn, &run.id, "canceled")
        .await
        .expect("terminal transition");
    specos_runtime_service::set_team_control(&db.conn, &run.id, "canceled")
        .await
        .expect("repeated terminal transition is idempotent");
    let resume_terminal = specos_runtime_service::set_team_control(&db.conn, &run.id, "running")
        .await
        .expect_err("terminal Team runs cannot resume");
    assert!(resume_terminal.to_string().contains("canceled"));
}

#[tokio::test]
async fn t016_axum_requests_match_shared_command_core() {
    let (db, root, folder_id) = fixture().await;
    let catalog = catalog(2);
    let direct = core::team_catalog_save_core(&db, folder_id, catalog.clone())
        .await
        .expect("direct command-core save");
    let (server, state, _data, _static) = build_server(db).await;

    let response = server
        .post("/api/specos_team_catalog_save")
        .add_header("authorization", format!("Bearer {TEST_TOKEN}"))
        .json(&json!({"folderId": folder_id, "catalog": catalog}))
        .await;
    assert_eq!(response.status_code(), 200, "{}", response.text());
    let via_axum: Value = response.json();
    assert_eq!(via_axum, serde_json::to_value(&direct).unwrap());

    let response = server
        .post("/api/specos_team_run_start")
        .add_header("authorization", format!("Bearer {TEST_TOKEN}"))
        .json(&json!({"folderId": folder_id, "workflowId": "delivery-flow"}))
        .await;
    assert_eq!(response.status_code(), 200, "{}", response.text());
    let started: Value = response.json();
    assert_eq!(started["status"], "queued");
    assert_eq!(started["nodes"].as_array().unwrap().len(), 3);

    let project_root = state.db.conn.clone();
    let saved_on_disk = specos_control::load_teams(root.path()).unwrap();
    let listed = core::team_run_list_core(&AppDatabase { conn: project_root }, folder_id)
        .await
        .unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(saved_on_disk.workflows[0].id, "delivery-flow");
    assert_eq!(listed[0].status, "queued");
}

#[tokio::test]
async fn t015_catalog_rejects_cycles_oversized_prompts_and_unknown_profiles() {
    let (db, root, folder_id) = fixture().await;
    let mut oversized = catalog(1);
    oversized.workflows[0].nodes[0].prompt = "x".repeat(64 * 1024 + 1);
    let oversized_errors = specos_control::validate_teams(&oversized);
    assert!(oversized_errors
        .iter()
        .any(|error| error.contains("prompt") && error.contains("byte")));

    let mut cycle = catalog(1);
    cycle.workflows[0].nodes[0].depends_on = vec!["review".into()];
    let cycle_errors = specos_control::validate_teams(&cycle);
    assert!(cycle_errors
        .iter()
        .any(|error| error.contains("dependency cycle")));

    let mut unknown = catalog(1);
    unknown.workflows[0].nodes[0].agent_profile_id = "missing".into();
    specos_control::save_teams(root.path(), unknown).expect("internal DAG remains YAML-valid");
    let start_error =
        core::team_run_start_core(&EventEmitter::Noop, &db, folder_id, "delivery-flow".into())
            .await
            .expect_err("start must revalidate external catalog references");
    assert!(start_error.to_string().contains("missing AgentProfile"));
}
