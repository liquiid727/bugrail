//! Independent verification oracles for BUGRAIL-SPECOS-002..009 Test Specs.
//!
//! Each `t0n_*` function maps to a Test Spec scenario. Authoritative evidence
//! is SQLite after the call, YAML after validate-then-rename, and command-core
//! errors. Live events are never the only oracle.

use codeg_lib::agent_runtime;
use codeg_lib::context;
use codeg_lib::db::entities::{work_task, work_task_run};
use codeg_lib::db::service::{specos_runtime_service, work_task_service};
use codeg_lib::db::test_helpers::{fresh_in_memory_db, seed_folder};
use codeg_lib::models::{
    AgentCatalog, AgentProfile, ContextConfig, ContextLoadout, ContextProviderConfig,
    ContextSourceConfig, ModelProfile, ResolvedAgentRuntime, WorkTaskConfig, WorkTaskDraft,
    WorkTaskFolderSettings, WorkTaskHandoffDraft,
};
use codeg_lib::specos_control;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter, Set,
};
use std::collections::BTreeMap;
use std::path::Path;

/// Memory service over the same in-memory connection, for context surfaces
/// that now take `&MemoryService`. These tests never configure a memory
/// provider, so the service only supplies the production registry plumbing.
fn test_memory(db: &codeg_lib::db::AppDatabase) -> codeg_lib::memory::MemoryService {
    codeg_lib::memory::MemoryService::new(codeg_lib::db::AppDatabase {
        conn: db.conn.clone(),
    })
}

fn task(folder_id: i32, title: &str) -> WorkTaskDraft {
    WorkTaskDraft {
        folder_id,
        title: title.into(),
        config: serde_json::json!({
            "display_text": title,
            "prompt_blocks": [{"type":"text","text":title}],
        }),
    }
}

fn task_with_profile(folder_id: i32, title: &str, profile: &str) -> WorkTaskDraft {
    WorkTaskDraft {
        folder_id,
        title: title.into(),
        config: serde_json::json!({
            "display_text": title,
            "prompt_blocks": [{"type":"text","text":title}],
            "agent_profile_id": profile,
        }),
    }
}

fn shared_model_catalog() -> AgentCatalog {
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
        agent_profiles: vec![
            profile("planner", "codex", Some("shared")),
            profile("implementer", "codex", Some("shared")),
        ],
        validation_errors: vec![],
    }
}

fn profile(id: &str, adapter: &str, model: Option<&str>) -> AgentProfile {
    AgentProfile {
        id: id.into(),
        name: id.into(),
        runtime_adapter: adapter.into(),
        model_profile_id: model.map(str::to_string),
        mode_id: None,
        reasoning: None,
        context_loadout_id: None,
        skills: vec![],
        rules: vec![],
        tools: vec![],
        config_values: BTreeMap::new(),
        enabled: true,
    }
}

async fn set_status(
    conn: &sea_orm::DatabaseConnection,
    id: i32,
    status: work_task::WorkTaskStatus,
) {
    let row = work_task::Entity::find_by_id(id)
        .one(conn)
        .await
        .unwrap()
        .unwrap();
    let mut active = row.into_active_model();
    active.status = Set(status);
    active.update(conn).await.unwrap();
}

fn write_project(root: &Path, files: &[(&str, &str)]) {
    for (rel, body) in files {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, body).unwrap();
    }
}

fn source(path: &str, required: bool) -> ContextSourceConfig {
    ContextSourceConfig {
        path: path.into(),
        required,
        kind: "rules".into(),
    }
}

fn tiny_context(
    sources: Vec<ContextSourceConfig>,
    providers: Vec<ContextProviderConfig>,
) -> ContextConfig {
    let provider_ids = providers.iter().map(|p| p.id.clone()).collect();
    ContextConfig {
        version: 1,
        default_loadout_id: "default".into(),
        providers,
        loadouts: vec![ContextLoadout {
            id: "default".into(),
            name: "Default".into(),
            sources,
            provider_ids,
            max_items: 16,
            max_bytes: 32 * 1024,
            max_tokens: 8_000,
        }],
        validation_errors: vec![],
    }
}

async fn claim_task(conn: &sea_orm::DatabaseConnection, folder_id: i32, title: &str) -> (i32, i32) {
    let created = work_task_service::create(conn, task(folder_id, title))
        .await
        .unwrap();
    let seq =
        work_task_service::claim_for_run(conn, created.id, work_task::WorkTaskStatus::Todo, "test")
            .await
            .unwrap()
            .unwrap();
    (created.id, seq)
}

// ── BUGRAIL-SPECOS-002 ────────────────────────────────────────────────────

#[test]
fn t01_distinct_profile_same_model() {
    let catalog = shared_model_catalog();
    assert!(specos_control::validate_agents(&catalog).is_empty());
    let root = tempfile::tempdir().unwrap();
    specos_control::save_agents(root.path(), catalog).unwrap();
    let planner = agent_runtime::resolve(
        root.path(),
        &WorkTaskConfig {
            agent_profile_id: Some("planner".into()),
            ..Default::default()
        },
        &WorkTaskFolderSettings::default(),
        None,
    )
    .unwrap();
    let implementer = agent_runtime::resolve(
        root.path(),
        &WorkTaskConfig {
            agent_profile_id: Some("implementer".into()),
            ..Default::default()
        },
        &WorkTaskFolderSettings::default(),
        None,
    )
    .unwrap();
    assert_eq!(planner.agent_profile_id.as_deref(), Some("planner"));
    assert_eq!(implementer.agent_profile_id.as_deref(), Some("implementer"));
    assert_eq!(planner.model_profile_id.as_deref(), Some("shared"));
    assert_eq!(implementer.model_profile_id.as_deref(), Some("shared"));
    assert_ne!(planner.agent_profile_id, implementer.agent_profile_id);
}

#[test]
fn t02_resolution_precedence() {
    let root = tempfile::tempdir().unwrap();
    specos_control::save_agents(root.path(), shared_model_catalog()).unwrap();

    let explicit = agent_runtime::resolve(
        root.path(),
        &WorkTaskConfig {
            agent_profile_id: Some("implementer".into()),
            ..Default::default()
        },
        &WorkTaskFolderSettings::default(),
        None,
    )
    .unwrap();
    assert_eq!(explicit.reason_codes, vec!["explicit_task_profile"]);
    assert_eq!(explicit.agent_profile_id.as_deref(), Some("implementer"));

    let project_default = agent_runtime::resolve(
        root.path(),
        &WorkTaskConfig::default(),
        &WorkTaskFolderSettings::default(),
        None,
    )
    .unwrap();
    assert_eq!(
        project_default.reason_codes,
        vec!["project_default_profile"]
    );
    assert_eq!(project_default.agent_profile_id.as_deref(), Some("planner"));

    let empty = tempfile::tempdir().unwrap();
    specos_control::save_agents(
        empty.path(),
        AgentCatalog {
            version: 1,
            default_agent_profile_id: None,
            model_profiles: vec![],
            agent_profiles: vec![],
            validation_errors: vec![],
        },
    )
    .unwrap();
    let legacy = agent_runtime::resolve(
        empty.path(),
        &WorkTaskConfig {
            agent_type: Some("claude_code".into()),
            config_values: BTreeMap::from([("model".into(), "legacy-model".into())]),
            ..Default::default()
        },
        &WorkTaskFolderSettings::default(),
        None,
    )
    .unwrap();
    assert_eq!(legacy.reason_codes, vec!["explicit_legacy_agent"]);
    assert_eq!(legacy.agent_type, "claude_code");
    assert_eq!(legacy.model.as_deref(), Some("legacy-model"));
    assert!(legacy.agent_profile_id.is_none());
}

#[test]
fn t03_invalid_catalog_atomicity() {
    let root = tempfile::tempdir().unwrap();
    specos_control::save_agents(root.path(), shared_model_catalog()).unwrap();
    let mut invalid = shared_model_catalog();
    invalid.agent_profiles[0]
        .config_values
        .insert("api_key".into(), "secret".into());
    let err = specos_control::save_agents(root.path(), invalid).unwrap_err();
    assert!(err.to_string().contains("secret-like"));
    let reloaded = specos_control::load_agents(root.path()).unwrap();
    assert!(reloaded.validation_errors.is_empty());
    assert_eq!(reloaded.agent_profiles.len(), 2);
    assert!(reloaded.agent_profiles[0].config_values.is_empty());
}

#[test]
fn t04_secret_redaction_and_symlink_config() {
    let catalog = AgentCatalog {
        version: 1,
        default_agent_profile_id: Some("planner".into()),
        model_profiles: vec![ModelProfile {
            id: "shared".into(),
            name: "Shared".into(),
            provider_ref: None,
            model: String::new(),
            reasoning: None,
            fallback_profile_ids: vec![],
        }],
        agent_profiles: vec![AgentProfile {
            id: "planner".into(),
            name: "Planner".into(),
            runtime_adapter: "unknown-runtime".into(),
            model_profile_id: Some("shared".into()),
            mode_id: None,
            reasoning: None,
            context_loadout_id: None,
            skills: vec![],
            rules: vec![],
            tools: vec![],
            config_values: BTreeMap::from([("api_key".into(), "secret".into())]),
            enabled: true,
        }],
        validation_errors: vec![],
    };
    let errors = specos_control::validate_agents(&catalog).join("; ");
    assert!(errors.contains("unknown runtimeAdapter"));
    assert!(errors.contains("secret-like"));

    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(outside.path(), root.path().join(".codeg")).unwrap();
    let err = specos_control::save_agents(root.path(), shared_model_catalog()).unwrap_err();
    assert!(err.to_string().contains("must not be a symlink"));
}

#[test]
fn t05_legacy_fallback() {
    let root = tempfile::tempdir().unwrap();
    specos_control::save_agents(
        root.path(),
        AgentCatalog {
            version: 1,
            default_agent_profile_id: None,
            model_profiles: vec![],
            agent_profiles: vec![],
            validation_errors: vec![],
        },
    )
    .unwrap();
    let resolved = agent_runtime::resolve(
        root.path(),
        &WorkTaskConfig::default(),
        &WorkTaskFolderSettings {
            default_agent_type: Some("codex".into()),
            ..Default::default()
        },
        None,
    )
    .unwrap();
    assert_eq!(resolved.reason_codes, vec!["folder_default_agent"]);
    assert_eq!(resolved.agent_type, "codex");
    assert!(resolved.agent_profile_id.is_none());
}

#[test]
fn t05_missing_agents_yaml_falls_back_to_legacy() {
    let root = tempfile::tempdir().unwrap();
    let resolved = agent_runtime::resolve(
        root.path(),
        &WorkTaskConfig {
            agent_type: Some("codex".into()),
            ..Default::default()
        },
        &WorkTaskFolderSettings::default(),
        None,
    )
    .unwrap();
    assert_eq!(resolved.reason_codes, vec!["explicit_legacy_agent"]);
    assert_eq!(resolved.agent_type, "codex");
    assert!(resolved.agent_profile_id.is_none());
}

// ── BUGRAIL-SPECOS-003 ────────────────────────────────────────────────────

#[tokio::test]
async fn t01_one_row_per_generation() {
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, "/tmp/specos-run-t01").await;
    let a = work_task_service::create(&db.conn, task(folder_id, "A"))
        .await
        .unwrap();
    let first =
        work_task_service::claim_for_run(&db.conn, a.id, work_task::WorkTaskStatus::Todo, "test")
            .await
            .unwrap()
            .unwrap();
    assert_eq!(first, 1);
    set_status(&db.conn, a.id, work_task::WorkTaskStatus::Failed).await;
    let second =
        work_task_service::claim_for_run(&db.conn, a.id, work_task::WorkTaskStatus::Failed, "test")
            .await
            .unwrap()
            .unwrap();
    assert_eq!(second, 2);
    let runs = specos_runtime_service::list_runs(&db.conn, a.id)
        .await
        .unwrap();
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].run_seq, 2);
    assert_eq!(runs[1].run_seq, 1);
}

#[tokio::test]
async fn t02_claim_rollback() {
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, "/tmp/specos-run-t02").await;
    let a = work_task_service::create(&db.conn, task(folder_id, "A"))
        .await
        .unwrap();
    let lost =
        work_task_service::claim_for_run(&db.conn, a.id, work_task::WorkTaskStatus::Review, "test")
            .await
            .unwrap();
    assert!(lost.is_none());
    assert_eq!(
        work_task::Entity::find()
            .filter(work_task::Column::Id.eq(a.id))
            .one(&db.conn)
            .await
            .unwrap()
            .unwrap()
            .run_seq,
        0
    );
    assert_eq!(
        work_task_run::Entity::find()
            .filter(work_task_run::Column::TaskId.eq(a.id))
            .count(&db.conn)
            .await
            .unwrap(),
        0
    );
}

#[tokio::test]
async fn t03_retry_run_attribution() {
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, "/tmp/specos-run-t03").await;
    let a = work_task_service::create(&db.conn, task_with_profile(folder_id, "A", "planner"))
        .await
        .unwrap();
    let first =
        work_task_service::claim_for_run(&db.conn, a.id, work_task::WorkTaskStatus::Todo, "test")
            .await
            .unwrap()
            .unwrap();
    set_status(&db.conn, a.id, work_task::WorkTaskStatus::Failed).await;
    let retry = work_task_service::claim_for_run_with_action(
        &db.conn,
        a.id,
        work_task::WorkTaskStatus::Failed,
        "test",
        Some(serde_json::json!({"action":"retry","note":"again"})),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(first, 1);
    assert_eq!(retry, 2);
    let runs = specos_runtime_service::list_runs(&db.conn, a.id)
        .await
        .unwrap();
    assert_eq!(runs[0].run_seq, 2);
    assert_eq!(runs[0].agent_profile_id.as_deref(), Some("planner"));
    assert_eq!(runs[1].run_seq, 1);
}

#[tokio::test]
async fn t04_immutable_resolution() {
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, "/tmp/specos-run-t04").await;
    let a = work_task_service::create(&db.conn, task(folder_id, "A"))
        .await
        .unwrap();
    let seq =
        work_task_service::claim_for_run(&db.conn, a.id, work_task::WorkTaskStatus::Todo, "test")
            .await
            .unwrap()
            .unwrap();
    let runtime = ResolvedAgentRuntime {
        agent_profile_id: Some("planner".into()),
        model_profile_id: Some("shared".into()),
        agent_type: "codex".into(),
        model: Some("gpt-test".into()),
        mode_id: Some("default".into()),
        reasoning: Some("medium".into()),
        context_loadout_id: Some("default".into()),
        config_values: BTreeMap::new(),
        reason_codes: vec!["explicit_task_profile".into()],
    };
    specos_runtime_service::update_run_resolution(&db.conn, a.id, seq, &runtime)
        .await
        .unwrap();
    let runs = specos_runtime_service::list_runs(&db.conn, a.id)
        .await
        .unwrap();
    let resolution = runs[0].resolution.as_ref().unwrap();
    assert_eq!(resolution["agentProfileId"], "planner");
    assert_eq!(resolution["model"], "gpt-test");
    specos_runtime_service::update_run_state(&db.conn, a.id, seq, "running", None, false)
        .await
        .unwrap();
    let again = specos_runtime_service::list_runs(&db.conn, a.id)
        .await
        .unwrap();
    assert_eq!(
        again[0].resolution.as_ref().unwrap()["agentProfileId"],
        "planner"
    );
    assert_eq!(again[0].status, "running");
}

#[tokio::test]
async fn t05_restart_projection() {
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, "/tmp/specos-run-t05").await;
    let a = work_task_service::create(&db.conn, task(folder_id, "A"))
        .await
        .unwrap();
    work_task_service::claim_for_run(&db.conn, a.id, work_task::WorkTaskStatus::Todo, "test")
        .await
        .unwrap();
    let first = specos_runtime_service::list_runs(&db.conn, a.id)
        .await
        .unwrap();
    let second = specos_runtime_service::list_runs(&db.conn, a.id)
        .await
        .unwrap();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].run_seq, second[0].run_seq);
    assert_eq!(first[0].created_at, second[0].created_at);
}

#[tokio::test]
async fn t06_legacy_event_compatibility() {
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, "/tmp/specos-run-t06").await;
    let a = work_task_service::create(&db.conn, task(folder_id, "legacy"))
        .await
        .unwrap();
    let listed = specos_runtime_service::list_runs(&db.conn, a.id)
        .await
        .unwrap();
    assert!(listed.is_empty());
    let row = work_task_service::get(&db.conn, a.id).await.unwrap();
    assert_eq!(row.title, "legacy");
    assert_eq!(row.run_seq, 0);
}

// ── BUGRAIL-SPECOS-004 ────────────────────────────────────────────────────

#[tokio::test]
async fn t01_acyclic_edge_validation() {
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, "/tmp/specos-dep-t01").await;
    let a = work_task_service::create(&db.conn, task(folder_id, "A"))
        .await
        .unwrap();
    let b = work_task_service::create(&db.conn, task(folder_id, "B"))
        .await
        .unwrap();
    specos_runtime_service::add_dependency(&db.conn, a.id, b.id, "completion")
        .await
        .unwrap();
    let cycle = specos_runtime_service::add_dependency(&db.conn, b.id, a.id, "completion")
        .await
        .unwrap_err();
    assert!(cycle.to_string().contains("cycle"));
    let self_edge = specos_runtime_service::add_dependency(&db.conn, a.id, a.id, "completion")
        .await
        .unwrap_err();
    assert!(self_edge.to_string().contains("itself"));
}

#[tokio::test]
async fn t02_blocked_child_not_selected() {
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, "/tmp/specos-dep-t02").await;
    let parent = work_task_service::create(&db.conn, task(folder_id, "P"))
        .await
        .unwrap();
    let child = work_task_service::create(&db.conn, task(folder_id, "C"))
        .await
        .unwrap();
    specos_runtime_service::add_dependency(&db.conn, parent.id, child.id, "completion")
        .await
        .unwrap();
    let unmet = work_task_service::claim_for_run(
        &db.conn,
        child.id,
        work_task::WorkTaskStatus::Todo,
        "user",
    )
    .await
    .unwrap_err();
    assert!(specos_runtime_service::is_unmet_dependency(&unmet));
    set_status(&db.conn, child.id, work_task::WorkTaskStatus::Queued).await;
    assert!(
        !specos_runtime_service::dependencies_satisfied(&db.conn, child.id)
            .await
            .unwrap()
    );
    assert!(work_task_service::next_queued(&db.conn, folder_id, &[])
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn t03_parallel_ready_claims() {
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, "/tmp/specos-dep-t03").await;
    let parent = work_task_service::create(&db.conn, task(folder_id, "P"))
        .await
        .unwrap();
    let c1 = work_task_service::create(&db.conn, task(folder_id, "C1"))
        .await
        .unwrap();
    let c2 = work_task_service::create(&db.conn, task(folder_id, "C2"))
        .await
        .unwrap();
    specos_runtime_service::add_dependency(&db.conn, parent.id, c1.id, "completion")
        .await
        .unwrap();
    specos_runtime_service::add_dependency(&db.conn, parent.id, c2.id, "completion")
        .await
        .unwrap();
    set_status(&db.conn, parent.id, work_task::WorkTaskStatus::Done).await;
    set_status(&db.conn, c1.id, work_task::WorkTaskStatus::Queued).await;
    set_status(&db.conn, c2.id, work_task::WorkTaskStatus::Queued).await;
    let first = work_task_service::next_queued(&db.conn, folder_id, &[])
        .await
        .unwrap()
        .unwrap();
    let second = work_task_service::next_queued(&db.conn, folder_id, &[first.id])
        .await
        .unwrap()
        .unwrap();
    let ids = [first.id, second.id];
    assert!(ids.contains(&c1.id) && ids.contains(&c2.id));
}

#[tokio::test]
async fn t04_parent_failure_reason() {
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, "/tmp/specos-dep-t04").await;
    let parent = work_task_service::create(&db.conn, task(folder_id, "P"))
        .await
        .unwrap();
    let child = work_task_service::create(&db.conn, task(folder_id, "C"))
        .await
        .unwrap();
    specos_runtime_service::add_dependency(&db.conn, parent.id, child.id, "completion")
        .await
        .unwrap();
    set_status(&db.conn, parent.id, work_task::WorkTaskStatus::Failed).await;
    set_status(&db.conn, child.id, work_task::WorkTaskStatus::Queued).await;
    assert!(
        !specos_runtime_service::dependencies_satisfied(&db.conn, child.id)
            .await
            .unwrap()
    );
    assert!(work_task_service::next_queued(&db.conn, folder_id, &[])
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn t05_concurrency_race_cycle() {
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, "/tmp/specos-dep-t05").await;
    let a = work_task_service::create(&db.conn, task(folder_id, "A"))
        .await
        .unwrap();
    let b = work_task_service::create(&db.conn, task(folder_id, "B"))
        .await
        .unwrap();
    let (left, right) = tokio::join!(
        specos_runtime_service::add_dependency(&db.conn, a.id, b.id, "completion"),
        specos_runtime_service::add_dependency(&db.conn, b.id, a.id, "completion")
    );
    assert_eq!(left.is_ok() as u8 + right.is_ok() as u8, 1);
    let edges = specos_runtime_service::list_dependencies(&db.conn, a.id)
        .await
        .unwrap();
    assert_eq!(edges.len(), 1);
}

#[tokio::test]
async fn t06_legacy_task_readiness() {
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, "/tmp/specos-dep-t06").await;
    let a = work_task_service::create(&db.conn, task(folder_id, "legacy"))
        .await
        .unwrap();
    assert!(
        specos_runtime_service::dependencies_satisfied(&db.conn, a.id)
            .await
            .unwrap()
    );
    set_status(&db.conn, a.id, work_task::WorkTaskStatus::Queued).await;
    let next = work_task_service::next_queued(&db.conn, folder_id, &[])
        .await
        .unwrap()
        .unwrap();
    assert_eq!(next.id, a.id);
}

// ── BUGRAIL-SPECOS-005 ────────────────────────────────────────────────────

#[tokio::test]
async fn t01_handoff_roundtrip() {
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, "/tmp/specos-handoff-t01").await;
    let a = work_task_service::create(&db.conn, task(folder_id, "A"))
        .await
        .unwrap();
    work_task_service::claim_for_run(&db.conn, a.id, work_task::WorkTaskStatus::Todo, "test")
        .await
        .unwrap();
    let saved = specos_runtime_service::save_handoff(
        &db.conn,
        a.id,
        WorkTaskHandoffDraft {
            summary: "merged the payment adapter".into(),
            artifacts: vec!["src/pay.rs".into()],
            risks: vec!["no live charge test".into()],
            open_questions: vec!["rotate keys?".into()],
        },
    )
    .await
    .unwrap();
    assert_eq!(saved.run_seq, 1);
    let loaded = specos_runtime_service::get_handoff(&db.conn, a.id, Some(1))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(loaded.summary, "merged the payment adapter");
    assert_eq!(loaded.artifacts, vec!["src/pay.rs"]);
}

#[tokio::test]
async fn t02_missing_handoff_is_absent() {
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, "/tmp/specos-handoff-t02").await;
    let a = work_task_service::create(&db.conn, task(folder_id, "A"))
        .await
        .unwrap();
    assert!(specos_runtime_service::get_handoff(&db.conn, a.id, None)
        .await
        .unwrap()
        .is_none());
    let rejected = specos_runtime_service::save_handoff(
        &db.conn,
        a.id,
        WorkTaskHandoffDraft {
            summary: "   ".into(),
            ..Default::default()
        },
    )
    .await
    .unwrap_err();
    assert!(rejected.to_string().contains("required"));
    assert!(specos_runtime_service::get_handoff(&db.conn, a.id, None)
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn t06_legacy_summary_compatibility() {
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, "/tmp/specos-handoff-t06").await;
    let a = work_task_service::create(&db.conn, task(folder_id, "legacy"))
        .await
        .unwrap();
    assert!(specos_runtime_service::list_runs(&db.conn, a.id)
        .await
        .unwrap()
        .is_empty());
    assert!(specos_runtime_service::get_handoff(&db.conn, a.id, None)
        .await
        .unwrap()
        .is_none());
}

// ── BUGRAIL-SPECOS-006 / 007 / 008 ────────────────────────────────────────

#[tokio::test]
async fn t01_deterministic_order_hash() {
    let root = tempfile::tempdir().unwrap();
    write_project(
        root.path(),
        &[
            ("AGENTS.md", "# agents\n"),
            ("README.md", "# readme\n"),
            (".rules/project.md", "# rules\n"),
        ],
    );
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, root.path().to_str().unwrap()).await;
    let (task_id, seq) = claim_task(&db.conn, folder_id, "ctx").await;
    let first = context::prepare_run(
        &db.conn,
        folder_id,
        root.path(),
        root.path(),
        task_id,
        seq,
        None,
        Vec::new(),
        &test_memory(&db),
    )
    .await
    .unwrap();
    let second = context::prepare_run(
        &db.conn,
        folder_id,
        root.path(),
        root.path(),
        task_id,
        seq,
        None,
        Vec::new(),
        &test_memory(&db),
    )
    .await
    .unwrap();
    assert_eq!(first.package.id, second.package.id);
    assert_eq!(first.package.content_hash, second.package.content_hash);
    assert!(!first.package.content_hash.is_empty());
    assert!(first.prompt.contains(&first.package.id));
    let items: Vec<_> = first
        .package
        .items
        .iter()
        .map(|i| i.source.as_str())
        .collect();
    assert!(items.contains(&"AGENTS.md"));
}

#[tokio::test]
async fn t02_required_budget_block() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path(), &[("big.md", &"x".repeat(100))]);
    specos_control::save_context(
        root.path(),
        ContextConfig {
            version: 1,
            default_loadout_id: "default".into(),
            providers: vec![],
            loadouts: vec![ContextLoadout {
                id: "default".into(),
                name: "tiny".into(),
                sources: vec![source("big.md", true)],
                provider_ids: vec![],
                max_items: 8,
                max_bytes: 8,
                max_tokens: 8_000,
            }],
            validation_errors: vec![],
        },
    )
    .unwrap();
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, root.path().to_str().unwrap()).await;
    let (task_id, seq) = claim_task(&db.conn, folder_id, "ctx").await;
    let err = match context::prepare_run(
        &db.conn,
        folder_id,
        root.path(),
        root.path(),
        task_id,
        seq,
        None,
        Vec::new(),
        &test_memory(&db),
    )
    .await
    {
        Ok(_) => panic!("required oversize source must block"),
        Err(err) => err,
    };
    assert!(
        err.to_string().contains("budget") || err.to_string().contains("exceeds"),
        "{err}"
    );
}

#[tokio::test]
async fn t03_optional_source_absence() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path(), &[("AGENTS.md", "# agents\n")]);
    specos_control::save_context(
        root.path(),
        tiny_context(
            vec![source("AGENTS.md", true), source("missing.md", false)],
            vec![],
        ),
    )
    .unwrap();
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, root.path().to_str().unwrap()).await;
    let (task_id, seq) = claim_task(&db.conn, folder_id, "ctx").await;
    let prepared = context::prepare_run(
        &db.conn,
        folder_id,
        root.path(),
        root.path(),
        task_id,
        seq,
        None,
        Vec::new(),
        &test_memory(&db),
    )
    .await
    .unwrap();
    assert_eq!(prepared.package.items.len(), 1);
    assert_eq!(prepared.package.items[0].source, "AGENTS.md");
}

#[tokio::test]
async fn t04_path_and_symlink_escape() {
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(outside.path().join("secret.md"), "secret").unwrap();
    std::os::unix::fs::symlink(
        outside.path().join("secret.md"),
        root.path().join("link.md"),
    )
    .unwrap();
    specos_control::save_context(
        root.path(),
        tiny_context(vec![source("link.md", true)], vec![]),
    )
    .unwrap();
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, root.path().to_str().unwrap()).await;
    let (task_id, seq) = claim_task(&db.conn, folder_id, "ctx").await;
    let err = match context::prepare_run(
        &db.conn,
        folder_id,
        root.path(),
        root.path(),
        task_id,
        seq,
        None,
        Vec::new(),
        &test_memory(&db),
    )
    .await
    {
        Ok(_) => panic!("escaped symlink must block"),
        Err(err) => err,
    };
    assert!(err.to_string().contains("escapes"), "{err}");

    let invalid = tiny_context(vec![source("../outside.md", true)], vec![]);
    let errors = specos_control::validate_context(&invalid).join("; ");
    assert!(errors.contains("repository-relative"));
}

#[tokio::test]
async fn t05_retry_package_isolation() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path(), &[("AGENTS.md", "# agents\n")]);
    specos_control::save_context(
        root.path(),
        tiny_context(vec![source("AGENTS.md", true)], vec![]),
    )
    .unwrap();
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, root.path().to_str().unwrap()).await;
    let created = work_task_service::create(&db.conn, task(folder_id, "ctx"))
        .await
        .unwrap();
    let seq1 = work_task_service::claim_for_run(
        &db.conn,
        created.id,
        work_task::WorkTaskStatus::Todo,
        "t",
    )
    .await
    .unwrap()
    .unwrap();
    let first = context::prepare_run(
        &db.conn,
        folder_id,
        root.path(),
        root.path(),
        created.id,
        seq1,
        None,
        Vec::new(),
        &test_memory(&db),
    )
    .await
    .unwrap();
    set_status(&db.conn, created.id, work_task::WorkTaskStatus::Failed).await;
    let seq2 = work_task_service::claim_for_run(
        &db.conn,
        created.id,
        work_task::WorkTaskStatus::Failed,
        "t",
    )
    .await
    .unwrap()
    .unwrap();
    let second = context::prepare_run(
        &db.conn,
        folder_id,
        root.path(),
        root.path(),
        created.id,
        seq2,
        None,
        Vec::new(),
        &test_memory(&db),
    )
    .await
    .unwrap();
    assert_ne!(first.package.id, second.package.id);
    assert_eq!(first.package.run_seq, 1);
    assert_eq!(second.package.run_seq, 2);
    let bound = specos_runtime_service::list_runs(&db.conn, created.id)
        .await
        .unwrap();
    assert_eq!(
        bound[0].context_package_id.as_deref(),
        Some(second.package.id.as_str())
    );
    assert_eq!(
        bound[1].context_package_id.as_deref(),
        Some(first.package.id.as_str())
    );
}

#[tokio::test]
async fn t06_post_restart_inspection() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path(), &[("AGENTS.md", "# agents\n")]);
    specos_control::save_context(
        root.path(),
        tiny_context(vec![source("AGENTS.md", true)], vec![]),
    )
    .unwrap();
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, root.path().to_str().unwrap()).await;
    let (task_id, seq) = claim_task(&db.conn, folder_id, "ctx").await;
    let prepared = context::prepare_run(
        &db.conn,
        folder_id,
        root.path(),
        root.path(),
        task_id,
        seq,
        None,
        Vec::new(),
        &test_memory(&db),
    )
    .await
    .unwrap();
    let fetched = context::package_get(&db.conn, &prepared.package.id)
        .await
        .unwrap();
    assert_eq!(fetched.content_hash, prepared.package.content_hash);
    assert_eq!(fetched.items.len(), prepared.package.items.len());
}

#[tokio::test]
async fn provider_t01_local_health_and_t03_required_block() {
    let db = fresh_in_memory_db().await;
    let local = ContextProviderConfig {
        id: "local".into(),
        kind: "local".into(),
        adapter: None,
        endpoint: None,
        secret_env: None,
        enabled: true,
        required: false,
        capabilities: vec![],
        ..Default::default()
    };
    let health =
        context::check_provider_health(std::slice::from_ref(&local), &test_memory(&db), 0).await;
    assert_eq!(health[0].status, "healthy");

    let root = tempfile::tempdir().unwrap();
    write_project(root.path(), &[("AGENTS.md", "# agents\n")]);
    specos_control::save_context(
        root.path(),
        tiny_context(
            vec![source("AGENTS.md", true)],
            vec![ContextProviderConfig {
                id: "remote".into(),
                kind: "tencent-memory".into(),
                adapter: None,
                endpoint: Some("http://127.0.0.1:1".into()),
                secret_env: Some("TENCENT_TOKEN".into()),
                enabled: true,
                required: true,
                capabilities: vec!["memory".into()],
                ..Default::default()
            }],
        ),
    )
    .unwrap();
    let folder_id = seed_folder(&db, root.path().to_str().unwrap()).await;
    let (task_id, seq) = claim_task(&db.conn, folder_id, "ctx").await;
    let err = match context::prepare_run(
        &db.conn,
        folder_id,
        root.path(),
        root.path(),
        task_id,
        seq,
        None,
        Vec::new(),
        &test_memory(&db),
    )
    .await
    {
        Ok(_) => panic!("required unavailable provider must block"),
        Err(err) => err,
    };
    assert!(
        err.to_string().contains("required context provider"),
        "{err}"
    );
}

#[tokio::test]
async fn provider_t04_optional_degradation() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path(), &[("AGENTS.md", "# agents\n")]);
    specos_control::save_context(
        root.path(),
        tiny_context(
            vec![source("AGENTS.md", true)],
            vec![ContextProviderConfig {
                id: "remote".into(),
                kind: "tencent-memory".into(),
                adapter: None,
                endpoint: Some("http://127.0.0.1:1".into()),
                secret_env: Some("TENCENT_TOKEN".into()),
                enabled: true,
                required: false,
                capabilities: vec!["memory".into()],
                ..Default::default()
            }],
        ),
    )
    .unwrap();
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, root.path().to_str().unwrap()).await;
    let (task_id, seq) = claim_task(&db.conn, folder_id, "ctx").await;
    let prepared = context::prepare_run(
        &db.conn,
        folder_id,
        root.path(),
        root.path(),
        task_id,
        seq,
        None,
        Vec::new(),
        &test_memory(&db),
    )
    .await
    .unwrap();
    assert_eq!(prepared.package.status, "degraded");
}

#[tokio::test]
async fn provider_t05_timeout_and_redaction() {
    let db = fresh_in_memory_db().await;
    let disabled = ContextProviderConfig {
        id: "off".into(),
        kind: "tencent-memory".into(),
        adapter: None,
        endpoint: Some("http://127.0.0.1:1".into()),
        secret_env: Some("TENCENT_TOKEN".into()),
        enabled: false,
        required: false,
        capabilities: vec![],
        ..Default::default()
    };
    let health = context::check_provider_health(&[disabled], &test_memory(&db), 0).await;
    assert_eq!(health[0].status, "disabled");
    let dumped = serde_json::to_string(&health).unwrap();
    assert!(!dumped.contains("sk-"));
    assert!(!dumped.to_lowercase().contains("bearer"));

    let invalid = tiny_context(
        vec![],
        vec![ContextProviderConfig {
            id: "remote".into(),
            kind: "tencent-memory".into(),
            adapter: None,
            endpoint: Some("file:///tmp/socket".into()),
            secret_env: Some("not-safe".into()),
            enabled: false,
            required: true,
            capabilities: vec![],
            ..Default::default()
        }],
    );
    let errors = specos_control::validate_context(&invalid).join("; ");
    assert!(errors.contains("must be enabled"));
    assert!(errors.contains("http or https"));
    assert!(errors.contains("uppercase environment"));
}

fn codebase_provider(required: bool) -> ContextProviderConfig {
    ContextProviderConfig {
        id: "codebase".into(),
        kind: "code-intelligence".into(),
        adapter: Some("codebase-memory-mcp".into()),
        endpoint: None,
        secret_env: None,
        enabled: true,
        required,
        capabilities: vec![
            "code.search".into(),
            "code.trace".into(),
            "code.architecture".into(),
            "code.impact".into(),
            "code.coverage".into(),
        ],
        ..Default::default()
    }
}

#[tokio::test]
async fn provider_codebase_valid_config_is_accepted() {
    let config = tiny_context(vec![], vec![codebase_provider(false)]);
    assert!(
        specos_control::validate_context(&config).is_empty(),
        "valid code-intelligence provider must validate"
    );
    // Health is statically healthy — never blocks, never probes.
    let db = fresh_in_memory_db().await;
    let health =
        context::check_provider_health(&[codebase_provider(false)], &test_memory(&db), 0).await;
    assert_eq!(health[0].status, "healthy");
}

#[tokio::test]
async fn provider_codebase_required_is_forbidden() {
    let config = tiny_context(vec![], vec![codebase_provider(true)]);
    let errors = specos_control::validate_context(&config).join("; ");
    assert!(errors.contains("must not be required"), "{errors}");
}

#[tokio::test]
async fn provider_codebase_adapter_is_pinned_and_local_only() {
    let mut provider = codebase_provider(false);
    provider.adapter = Some("some-other-adapter".into());
    let errors = specos_control::validate_context(&tiny_context(vec![], vec![provider])).join("; ");
    assert!(errors.contains("unknown adapter"), "{errors}");

    let mut provider = codebase_provider(false);
    provider.adapter = None;
    let errors = specos_control::validate_context(&tiny_context(vec![], vec![provider])).join("; ");
    assert!(errors.contains("requires adapter"), "{errors}");

    let mut provider = codebase_provider(false);
    provider.endpoint = Some("http://127.0.0.1:9".into());
    let errors = specos_control::validate_context(&tiny_context(vec![], vec![provider])).join("; ");
    assert!(errors.contains("must not set endpoint"), "{errors}");
}

#[tokio::test]
async fn provider_codebase_yaml_compat_round_trip() {
    // A pre-existing context.yaml without the `adapter` field still loads —
    // existing provider kinds are unaffected by the new optional field.
    let legacy = r#"
version: 1
defaultLoadoutId: default
providers:
  - id: remote
    kind: tencent-memory
    endpoint: "https://example.com"
    enabled: true
    required: false
    capabilities: []
loadouts:
  - id: default
    name: Default
    sources: []
    providerIds: [remote]
    maxItems: 16
    maxBytes: 32768
    maxTokens: 8000
"#;
    let parsed: codeg_lib::models::ContextConfig = serde_yaml::from_str(legacy).unwrap();
    assert!(parsed.providers[0].adapter.is_none());
    assert!(specos_control::validate_context(&parsed).is_empty());

    // Serializing a provider without adapter must not emit the field.
    let dumped = serde_yaml::to_string(&parsed).unwrap();
    assert!(!dumped.contains("adapter"));

    // The approved code-intelligence provider shape parses and validates.
    let with_codebase = r#"
version: 1
defaultLoadoutId: default
providers:
  - id: codebase
    kind: code-intelligence
    adapter: codebase-memory-mcp
    enabled: true
    required: false
    capabilities: [code.search, code.trace, code.architecture, code.impact, code.coverage]
loadouts:
  - id: default
    name: Default
    sources: []
    providerIds: [codebase]
    maxItems: 16
    maxBytes: 32768
    maxTokens: 8000
"#;
    let parsed: codeg_lib::models::ContextConfig = serde_yaml::from_str(with_codebase).unwrap();
    assert!(specos_control::validate_context(&parsed).is_empty());
}

#[tokio::test]
async fn loadout_t01_precedence_and_t06_prompt_order() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path(), &[("a.md", "A"), ("b.md", "B"), ("c.md", "C")]);
    specos_control::save_context(
        root.path(),
        ContextConfig {
            version: 1,
            default_loadout_id: "default".into(),
            providers: vec![],
            loadouts: vec![
                ContextLoadout {
                    id: "default".into(),
                    name: "Default".into(),
                    sources: vec![source("a.md", true)],
                    provider_ids: vec![],
                    max_items: 8,
                    max_bytes: 1024,
                    max_tokens: 1000,
                },
                ContextLoadout {
                    id: "review".into(),
                    name: "Review".into(),
                    sources: vec![source("b.md", true), source("c.md", true)],
                    provider_ids: vec![],
                    max_items: 8,
                    max_bytes: 1024,
                    max_tokens: 1000,
                },
            ],
            validation_errors: vec![],
        },
    )
    .unwrap();
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, root.path().to_str().unwrap()).await;
    let created = work_task_service::create(&db.conn, task(folder_id, "ctx"))
        .await
        .unwrap();
    let seq1 = work_task_service::claim_for_run(
        &db.conn,
        created.id,
        work_task::WorkTaskStatus::Todo,
        "t",
    )
    .await
    .unwrap()
    .unwrap();
    let default_pkg = context::prepare_run(
        &db.conn,
        folder_id,
        root.path(),
        root.path(),
        created.id,
        seq1,
        None,
        Vec::new(),
        &test_memory(&db),
    )
    .await
    .unwrap();
    assert_eq!(default_pkg.package.loadout_id, "default");
    assert_eq!(default_pkg.package.items[0].source, "a.md");

    set_status(&db.conn, created.id, work_task::WorkTaskStatus::Failed).await;
    let seq2 = work_task_service::claim_for_run(
        &db.conn,
        created.id,
        work_task::WorkTaskStatus::Failed,
        "t",
    )
    .await
    .unwrap()
    .unwrap();
    let review = context::prepare_run(
        &db.conn,
        folder_id,
        root.path(),
        root.path(),
        created.id,
        seq2,
        Some("review"),
        Vec::new(),
        &test_memory(&db),
    )
    .await
    .unwrap();
    assert_eq!(review.package.loadout_id, "review");
    assert_eq!(review.package.items[0].source, "b.md");
    assert_eq!(review.package.items[1].source, "c.md");
    let b_pos = review.prompt.find("--- b.md").unwrap();
    let c_pos = review.prompt.find("--- c.md").unwrap();
    assert!(b_pos < c_pos);
}

#[tokio::test]
async fn loadout_t03_dedupe_and_budget() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path(), &[("same.md", "dup"), ("copy.md", "dup")]);
    specos_control::save_context(
        root.path(),
        tiny_context(
            vec![source("same.md", true), source("copy.md", true)],
            vec![],
        ),
    )
    .unwrap();
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, root.path().to_str().unwrap()).await;
    let (task_id, seq) = claim_task(&db.conn, folder_id, "ctx").await;
    let prepared = context::prepare_run(
        &db.conn,
        folder_id,
        root.path(),
        root.path(),
        task_id,
        seq,
        None,
        Vec::new(),
        &test_memory(&db),
    )
    .await
    .unwrap();
    assert_eq!(prepared.package.items.len(), 1);
}

#[tokio::test]
async fn inspector_t02_t03_t05_overview_join_and_activity() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path(), &[("AGENTS.md", "# agents\n")]);
    specos_control::save_context(
        root.path(),
        tiny_context(vec![source("AGENTS.md", true)], vec![]),
    )
    .unwrap();
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, root.path().to_str().unwrap()).await;
    let (task_id, seq) = claim_task(&db.conn, folder_id, "ctx").await;
    let prepared = context::prepare_run(
        &db.conn,
        folder_id,
        root.path(),
        root.path(),
        task_id,
        seq,
        None,
        Vec::new(),
        &test_memory(&db),
    )
    .await
    .unwrap();
    let overview = context::overview(&db.conn, folder_id, root.path(), &test_memory(&db))
        .await
        .unwrap();
    assert_eq!(overview.packages.len(), 1);
    assert_eq!(overview.packages[0].id, prepared.package.id);
    assert_eq!(overview.packages[0].task_id, task_id);
    assert_eq!(overview.packages[0].run_seq, seq);
    assert!(!overview.activity.is_empty());
    assert!(overview.packages[0].items[0]
        .provenance
        .get("path")
        .is_some());
}

// ── BUGRAIL-SPECOS-005 Git truth ──────────────────────────────────────────

fn git_run(dir: &std::path::Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@example.com")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@example.com")
        .output()
        .expect("spawn git");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn init_repo_with_sources() -> (tempfile::TempDir, String, String) {
    let dir = tempfile::tempdir().unwrap();
    git_run(dir.path(), &["init", "-q"]);
    git_run(dir.path(), &["checkout", "-qb", "main"]);
    std::fs::write(dir.path().join("base.txt"), "base\n").unwrap();
    git_run(dir.path(), &["add", "-A"]);
    git_run(dir.path(), &["commit", "-qm", "base"]);
    git_run(dir.path(), &["checkout", "-qb", "src-a"]);
    std::fs::write(dir.path().join("a.txt"), "A\n").unwrap();
    git_run(dir.path(), &["add", "-A"]);
    git_run(dir.path(), &["commit", "-qm", "source a"]);
    let head_a = String::from_utf8(
        std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(dir.path())
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string();
    git_run(dir.path(), &["checkout", "-q", "main"]);
    git_run(dir.path(), &["checkout", "-qb", "src-b"]);
    std::fs::write(dir.path().join("b.txt"), "B\n").unwrap();
    git_run(dir.path(), &["add", "-A"]);
    git_run(dir.path(), &["commit", "-qm", "source b"]);
    let head_b = String::from_utf8(
        std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(dir.path())
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string();
    git_run(dir.path(), &["checkout", "-q", "main"]);
    (dir, head_a, head_b)
}

async fn set_branch(conn: &sea_orm::DatabaseConnection, id: i32, branch: &str) {
    let row = work_task::Entity::find_by_id(id)
        .one(conn)
        .await
        .unwrap()
        .unwrap();
    let mut active = row.into_active_model();
    active.work_branch = Set(Some(branch.into()));
    active.update(conn).await.unwrap();
}

#[tokio::test]
async fn t02_missing_handoff_blocks_integration() {
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, "/tmp/specos-int-t02").await;
    let source = work_task_service::create(&db.conn, task(folder_id, "src"))
        .await
        .unwrap();
    let integ = work_task_service::create(&db.conn, task(folder_id, "integ"))
        .await
        .unwrap();
    specos_runtime_service::add_dependency(&db.conn, source.id, integ.id, "integration_source")
        .await
        .unwrap();
    set_status(&db.conn, source.id, work_task::WorkTaskStatus::Review).await;
    set_branch(&db.conn, source.id, "src-a").await;
    assert!(
        !specos_runtime_service::dependencies_satisfied(&db.conn, integ.id)
            .await
            .unwrap()
    );
    let plan = specos_runtime_service::integration_plan(&db.conn, integ.id, None)
        .await
        .unwrap();
    assert_eq!(plan.status, "waiting_source");
    assert!(!plan.sources[0].has_handoff);
}

#[tokio::test]
async fn t03_source_head_order() {
    let (repo, head_a, head_b) = init_repo_with_sources();
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, repo.path().to_str().unwrap()).await;
    let src_a = work_task_service::create(&db.conn, task(folder_id, "A"))
        .await
        .unwrap();
    let src_b = work_task_service::create(&db.conn, task(folder_id, "B"))
        .await
        .unwrap();
    let integ = work_task_service::create(&db.conn, task(folder_id, "I"))
        .await
        .unwrap();
    specos_runtime_service::add_dependency(&db.conn, src_a.id, integ.id, "integration_source")
        .await
        .unwrap();
    specos_runtime_service::add_dependency(&db.conn, src_b.id, integ.id, "integration_source")
        .await
        .unwrap();
    for (id, branch) in [(src_a.id, "src-a"), (src_b.id, "src-b")] {
        set_status(&db.conn, id, work_task::WorkTaskStatus::Review).await;
        set_branch(&db.conn, id, branch).await;
        work_task_service::claim_for_run(&db.conn, id, work_task::WorkTaskStatus::Review, "test")
            .await
            .ok();
    }
    // Sources must stay in review with a handoff on the live run_seq.
    for id in [src_a.id, src_b.id] {
        set_status(&db.conn, id, work_task::WorkTaskStatus::Review).await;
        specos_runtime_service::save_handoff(
            &db.conn,
            id,
            WorkTaskHandoffDraft {
                summary: format!("ready {id}"),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    }
    let plan = specos_runtime_service::refresh_integration_plan(
        &db.conn,
        integ.id,
        repo.path().to_str().unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(plan.status, "eligible");
    assert_eq!(plan.sources.len(), 2);
    assert!(plan.sources[0].task_id < plan.sources[1].task_id);
    assert_eq!(plan.sources[0].merge_order, 0);
    assert_eq!(plan.sources[1].merge_order, 1);
    let heads: Vec<_> = plan
        .sources
        .iter()
        .map(|s| s.current_head.clone().unwrap())
        .collect();
    assert!(heads.contains(&head_a));
    assert!(heads.contains(&head_b));
}

#[tokio::test]
async fn t04_conflict_recovery() {
    let dir = tempfile::tempdir().unwrap();
    git_run(dir.path(), &["init", "-q"]);
    git_run(dir.path(), &["checkout", "-qb", "main"]);
    std::fs::write(dir.path().join("f.txt"), "base\n").unwrap();
    git_run(dir.path(), &["add", "-A"]);
    git_run(dir.path(), &["commit", "-qm", "base"]);
    git_run(dir.path(), &["checkout", "-qb", "left"]);
    std::fs::write(dir.path().join("f.txt"), "left\n").unwrap();
    git_run(dir.path(), &["commit", "-qam", "left"]);
    git_run(dir.path(), &["checkout", "-q", "main"]);
    std::fs::write(dir.path().join("f.txt"), "right\n").unwrap();
    git_run(dir.path(), &["commit", "-qam", "right"]);
    let _ = std::process::Command::new("git")
        .args(["merge", "--no-commit", "left"])
        .current_dir(dir.path())
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@example.com")
        .output();
    assert!(
        codeg_lib::work_task::git::has_merge_head(dir.path().to_str().unwrap())
            .await
            .unwrap()
    );
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, dir.path().to_str().unwrap()).await;
    let integ = work_task_service::create(&db.conn, task(folder_id, "I"))
        .await
        .unwrap();
    let src = work_task_service::create(&db.conn, task(folder_id, "A"))
        .await
        .unwrap();
    specos_runtime_service::add_dependency(&db.conn, src.id, integ.id, "integration_source")
        .await
        .unwrap();
    let plan = specos_runtime_service::integration_plan(
        &db.conn,
        integ.id,
        Some(dir.path().to_str().unwrap()),
    )
    .await
    .unwrap();
    assert_eq!(plan.status, "conflict");
    assert_eq!(plan.conflicts, vec!["MERGE_HEAD".to_string()]);
}

#[tokio::test]
async fn t05_gated_integration_landing() {
    let (repo, _head_a, _) = init_repo_with_sources();
    git_run(repo.path(), &["merge", "--no-ff", "-m", "land a", "src-a"]);
    let landing = String::from_utf8(
        std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(repo.path())
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string();
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, repo.path().to_str().unwrap()).await;
    let src = work_task_service::create(&db.conn, task(folder_id, "A"))
        .await
        .unwrap();
    let integ = work_task_service::create(&db.conn, task(folder_id, "I"))
        .await
        .unwrap();
    specos_runtime_service::add_dependency(&db.conn, src.id, integ.id, "integration_source")
        .await
        .unwrap();
    set_status(&db.conn, src.id, work_task::WorkTaskStatus::Review).await;
    set_branch(&db.conn, src.id, "src-a").await;
    specos_runtime_service::save_handoff(
        &db.conn,
        src.id,
        WorkTaskHandoffDraft {
            summary: "ready".into(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    specos_runtime_service::refresh_integration_plan(
        &db.conn,
        integ.id,
        repo.path().to_str().unwrap(),
    )
    .await
    .unwrap();
    specos_runtime_service::assert_integration_landing(
        &db.conn,
        integ.id,
        repo.path().to_str().unwrap(),
        &landing,
    )
    .await
    .unwrap();
    git_run(repo.path(), &["checkout", "-q", "src-b"]);
    let only_b = String::from_utf8(
        std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(repo.path())
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string();
    let err = specos_runtime_service::assert_integration_landing(
        &db.conn,
        integ.id,
        repo.path().to_str().unwrap(),
        &only_b,
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("notContained"), "{err}");
}

// ── Code Intelligence engine items (non-MCP agents get a snapshot item) ──

fn code_intel_item(content: &str) -> context::EngineContextItem {
    context::EngineContextItem {
        kind: "code-intelligence".into(),
        source: "code-intelligence/codebase-memory-mcp".into(),
        title: "Code Intelligence summary".into(),
        content: content.into(),
        provenance: serde_json::json!({
            "provider": "code-intelligence",
            "adapter": "codebase-memory-mcp",
        }),
    }
}

#[tokio::test]
async fn engine_item_appended_after_sources_with_provenance() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path(), &[("AGENTS.md", "# agents\n")]);
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, root.path().to_str().unwrap()).await;
    let (task_id, seq) = claim_task(&db.conn, folder_id, "ctx").await;
    let summary = "{\"schema\":\"bugrail.code-intelligence.summary\",\"version\":1}";
    let prepared = context::prepare_run(
        &db.conn,
        folder_id,
        root.path(),
        root.path(),
        task_id,
        seq,
        None,
        vec![code_intel_item(summary)],
        &test_memory(&db),
    )
    .await
    .unwrap();
    let intel: Vec<_> = prepared
        .package
        .items
        .iter()
        .filter(|i| i.kind == "code-intelligence")
        .collect();
    assert_eq!(intel.len(), 1, "engine item must be appended exactly once");
    let item = intel[0];
    assert_eq!(item.source, "code-intelligence/codebase-memory-mcp");
    assert_eq!(item.title, "Code Intelligence summary");
    assert_eq!(item.content, summary);
    assert!(!item.required, "engine items never gate a run");
    assert_eq!(item.provenance["provider"], "code-intelligence");
    // Appended AFTER the file sources: every file item precedes it.
    let intel_ordinal = item.ordinal;
    assert!(prepared
        .package
        .items
        .iter()
        .filter(|i| i.kind != "code-intelligence")
        .all(|i| i.ordinal < intel_ordinal));
    // And it lands in the prompt the agent actually receives.
    assert!(prepared
        .prompt
        .contains("bugrail.code-intelligence.summary"));
}

#[tokio::test]
async fn engine_item_over_budget_is_skipped_not_fatal() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path(), &[("AGENTS.md", "# a\n")]);
    specos_control::save_context(
        root.path(),
        ContextConfig {
            version: 1,
            default_loadout_id: "default".into(),
            providers: vec![],
            loadouts: vec![ContextLoadout {
                id: "default".into(),
                name: "tiny".into(),
                sources: vec![source("AGENTS.md", false)],
                provider_ids: vec![],
                max_items: 8,
                max_bytes: 32,
                max_tokens: 8_000,
            }],
            validation_errors: vec![],
        },
    )
    .unwrap();
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, root.path().to_str().unwrap()).await;
    let (task_id, seq) = claim_task(&db.conn, folder_id, "ctx").await;
    let prepared = context::prepare_run(
        &db.conn,
        folder_id,
        root.path(),
        root.path(),
        task_id,
        seq,
        None,
        vec![code_intel_item(&"x".repeat(100))],
        &test_memory(&db),
    )
    .await
    .expect("a degraded snapshot must never block the run");
    assert!(
        prepared
            .package
            .items
            .iter()
            .all(|i| i.kind != "code-intelligence"),
        "oversize engine item must be skipped, not fail the package"
    );
    assert_eq!(prepared.package.status, "ready");
}

#[tokio::test]
async fn engine_item_duplicating_a_source_is_deduplicated() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path(), &[("AGENTS.md", "# agents\n")]);
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, root.path().to_str().unwrap()).await;
    let (task_id, seq) = claim_task(&db.conn, folder_id, "ctx").await;
    let prepared = context::prepare_run(
        &db.conn,
        folder_id,
        root.path(),
        root.path(),
        task_id,
        seq,
        None,
        vec![code_intel_item("# agents\n")],
        &test_memory(&db),
    )
    .await
    .unwrap();
    assert!(
        prepared
            .package
            .items
            .iter()
            .all(|i| i.kind != "code-intelligence"),
        "content-identical engine item must dedupe against the file source"
    );
}
