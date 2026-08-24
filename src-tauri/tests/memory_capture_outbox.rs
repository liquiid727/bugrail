//! Capture outbox integration oracles (BUGRAIL-SPECOS-017 Test Spec T03/T05).
//!
//! Settles WorkTask runs against a real Claude session fixture, stages the
//! outbox through `memory::capture::enqueue_for_run` and delivers through
//! `memory::capture_worker::deliver_due` into the deterministic in-memory
//! Adapter. No network is involved; the oracles are SQLite rows plus the
//! Adapter's captured batches.
//!
//! Tests in this file serialize on `ENV_LOCK` because they steer two
//! process-global environment seams: `CLAUDE_CONFIG_DIR` (session fixture
//! discovery) and the provider credential references.

use std::path::Path;
use std::sync::{LazyLock, Mutex, MutexGuard};

use chrono::Utc;
use codeg_lib::db::entities::{memory_capture_delivery as delivery, work_task};
use codeg_lib::db::service::{
    conversation_service, memory_capture_service, specos_runtime_service, work_task_service,
};
use codeg_lib::db::test_helpers::{fresh_in_memory_db, seed_folder};
use codeg_lib::memory::capture::{enqueue_for_run, MAX_CAPTURE_MESSAGES};
use codeg_lib::memory::capture_worker::{deliver_due, recover_and_reconcile};
use codeg_lib::memory::test_adapter::DeterministicMemoryAdapter;
use codeg_lib::memory::{AdapterRegistry, MemoryErrorClass, MemoryService};
use codeg_lib::models::{
    AgentType, ContextConfig, ContextLoadout, ContextProviderConfig, WorkTaskDraft,
};
use codeg_lib::specos_control;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, Set};
use serde_json::json;

use std::sync::Arc;

static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

// ── fixtures ────────────────────────────────────────────────────────────────

struct Scenario {
    _lock: MutexGuard<'static, ()>,
    db: codeg_lib::db::AppDatabase,
    memory: Arc<MemoryService>,
    adapter: DeterministicMemoryAdapter,
    folder_id: i32,
    _temp: tempfile::TempDir,
}

fn provider_config(env_suffix: &str) -> ContextProviderConfig {
    ContextProviderConfig {
        id: "project-memory".into(),
        kind: "memory".into(),
        adapter: Some("tencentdb-agent-memory-v3".into()),
        endpoint: Some("http://127.0.0.1:9".into()), // never dialed at enqueue
        secret_env: Some(format!("MEM_OUTBOX_SECRET_{env_suffix}")),
        service_id_env: Some(format!("MEM_OUTBOX_SERVICE_{env_suffix}")),
        team_id: Some(format!("team-{env_suffix}")),
        user_id_env: Some(format!("MEM_OUTBOX_USER_{env_suffix}")),
        default_agent_id: Some("bugrail-agent".into()),
        capabilities: vec!["memory.capture".into(), "memory.recall.l1".into()],
        enabled: true,
        ..Default::default()
    }
}

fn set_provider_env(env_suffix: &str) {
    // SAFETY: unique per-scenario names, guarded by ENV_LOCK.
    std::env::set_var(format!("MEM_OUTBOX_SECRET_{env_suffix}"), "s");
    std::env::set_var(format!("MEM_OUTBOX_SERVICE_{env_suffix}"), "svc");
    std::env::set_var(format!("MEM_OUTBOX_USER_{env_suffix}"), "user");
}

// The ENV_LOCK guard must span every await of the scenario: it serializes
// process-wide env mutation (CLAUDE_CONFIG_DIR + provider env references).
#[allow(clippy::await_holding_lock)]
async fn setup_scenario(env_suffix: &str, providers: Vec<ContextProviderConfig>) -> Scenario {
    let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    set_provider_env(env_suffix);

    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    std::fs::create_dir_all(root).expect("create project root");

    // Session fixture under a private CLAUDE_CONFIG_DIR.
    let claude_dir = root.join("claude-config");
    std::env::set_var("CLAUDE_CONFIG_DIR", &claude_dir);
    write_session_fixture(&claude_dir, "sess-outbox-1");

    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, root.to_str().unwrap()).await;

    let config = ContextConfig {
        version: 1,
        default_loadout_id: "default".into(),
        providers,
        loadouts: vec![ContextLoadout {
            id: "default".into(),
            name: "Default".into(),
            sources: vec![],
            provider_ids: vec![],
            max_items: 16,
            max_bytes: 32 * 1024,
            max_tokens: 8_000,
        }],
        validation_errors: vec![],
    };
    specos_control::save_context(root, config).expect("save context config");

    let adapter = DeterministicMemoryAdapter::new();
    let memory = Arc::new(MemoryService::new_with_registry(
        codeg_lib::db::AppDatabase {
            conn: db.conn.clone(),
        },
        AdapterRegistry::deterministic(Arc::new(adapter.clone())),
    ));

    Scenario {
        _lock: lock,
        db,
        memory,
        adapter,
        folder_id,
        _temp: temp,
    }
}

/// Mixed transcript: user/assistant text, tool use/result blocks and one
/// secret-bearing user message. Only the eligible text may reach the Adapter.
fn write_session_fixture(claude_dir: &Path, session_id: &str) {
    let project_dir = claude_dir.join("projects").join("-tmp-outbox");
    std::fs::create_dir_all(&project_dir).expect("create project dir");
    let lines = [
        json!({
            "type": "user",
            "sessionId": session_id,
            "timestamp": "2026-08-18T10:00:00Z",
            "uuid": "u1",
            "cwd": "/tmp/outbox",
            "gitBranch": "main",
            "message": { "content": [{ "type": "text", "text": "please fix the login bug" }] }
        }),
        json!({
            "type": "assistant",
            "sessionId": session_id,
            "timestamp": "2026-08-18T10:00:05Z",
            "uuid": "a1",
            "message": {
                "model": "claude-sonnet-4-6",
                "content": [
                    { "type": "text", "text": "looking at auth.rs now" },
                    {
                        "type": "tool_use",
                        "id": "toolu_1",
                        "name": "Read",
                        "input": { "file_path": "/tmp/outbox/auth.rs" }
                    }
                ],
                "usage": { "input_tokens": 100, "output_tokens": 20 }
            }
        }),
        json!({
            "type": "user",
            "sessionId": session_id,
            "timestamp": "2026-08-18T10:00:06Z",
            "uuid": "u2",
            "message": {
                "content": [{
                    "type": "tool_result",
                    "tool_use_id": "toolu_1",
                    "content": "fn login() { todo!() }"
                }]
            }
        }),
        json!({
            "type": "user",
            "sessionId": session_id,
            "timestamp": "2026-08-18T10:00:07Z",
            "uuid": "u3",
            "message": {
                "content": [{
                    "type": "text",
                    "text": "deploy key: sk-liveabc123abc123abc123abc"
                }]
            }
        }),
        json!({
            "type": "assistant",
            "sessionId": session_id,
            "timestamp": "2026-08-18T10:00:10Z",
            "uuid": "a2",
            "message": {
                "model": "claude-sonnet-4-6",
                "content": [{ "type": "text", "text": "fixed the session check in auth.rs" }],
                "usage": { "input_tokens": 120, "output_tokens": 30 }
            }
        }),
    ];
    let body = lines
        .iter()
        .map(serde_json::Value::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(project_dir.join(format!("{session_id}.jsonl")), body)
        .expect("write session fixture");
}

/// Create a task + claimed run and settle it to `review` (or `failed`) with
/// the session fixture bound as its conversation.
async fn settle_run(s: &Scenario, session_id: &str, status: &str) -> (i32, i32, i32) {
    let draft = WorkTaskDraft {
        folder_id: s.folder_id,
        title: "capture me".into(),
        config: serde_json::json!({
            "display_text": "capture me",
            "prompt_blocks": [{"type":"text","text":"capture me"}],
        }),
        task_kind: Default::default(),
    };
    let created = work_task_service::create(&s.db.conn, draft)
        .await
        .expect("create task");
    let run_seq = work_task_service::claim_for_run(
        &s.db.conn,
        created.id,
        work_task::WorkTaskStatus::Todo,
        "test",
    )
    .await
    .expect("claim run")
    .expect("claim seq");
    let conv =
        conversation_service::create(&s.db.conn, s.folder_id, AgentType::ClaudeCode, None, None)
            .await
            .expect("create conversation");
    conversation_service::bind_external_id(&s.db.conn, conv.id, session_id, &[])
        .await
        .expect("bind external id");
    specos_runtime_service::update_run_state(
        &s.db.conn,
        created.id,
        run_seq,
        "running",
        Some(conv.id),
        false,
    )
    .await
    .expect("bind conversation to run");
    specos_runtime_service::update_run_state(&s.db.conn, created.id, run_seq, status, None, true)
        .await
        .expect("settle run");
    let task_status = if status == "review" {
        work_task::WorkTaskStatus::Review
    } else {
        work_task::WorkTaskStatus::Failed
    };
    let row = work_task::Entity::find_by_id(created.id)
        .one(&s.db.conn)
        .await
        .unwrap()
        .unwrap();
    let mut active = row.into_active_model();
    active.status = Set(task_status);
    active.update(&s.db.conn).await.unwrap();
    (created.id, run_seq, conv.id)
}

fn captured_texts(adapter: &DeterministicMemoryAdapter) -> Vec<(String, String)> {
    let state = adapter.state();
    state
        .captured
        .iter()
        .flat_map(|batch| {
            batch.messages.iter().map(|message| {
                (
                    match message.role {
                        codeg_lib::memory::MemoryRole::User => "user".to_string(),
                        codeg_lib::memory::MemoryRole::Assistant => "assistant".to_string(),
                    },
                    message.content.clone(),
                )
            })
        })
        .collect()
}

async fn delivery_rows(db: &codeg_lib::db::AppDatabase) -> Vec<delivery::Model> {
    delivery::Entity::find().all(&db.conn).await.expect("rows")
}

// ── oracles ─────────────────────────────────────────────────────────────────

/// T03: only eligible bounded text reaches the Adapter; one row per
/// `(provider_id, task_id, run_seq)`; delivered rows retain hash/IDs only,
/// not the payload body.
#[tokio::test]
async fn t03_filtered_capture_delivers_once_and_clears_payload() {
    let s = setup_scenario("T03A", vec![provider_config("T03A")]).await;
    let (task_id, run_seq, _) = settle_run(&s, "sess-outbox-1", "review").await;

    let staged = enqueue_for_run(&s.db, task_id, run_seq)
        .await
        .expect("enqueue");
    assert_eq!(staged, 1, "one provider ⇒ one staged delivery");

    // Idempotent re-enqueue (the settle hook can race reconciliation).
    let again = enqueue_for_run(&s.db, task_id, run_seq)
        .await
        .expect("re-enqueue");
    assert_eq!(again, 0, "the unique key prevents duplicate rows");
    assert_eq!(delivery_rows(&s.db).await.len(), 1);

    let delivered = deliver_due(&s.memory).await.expect("deliver");
    assert_eq!(delivered, 1);

    // Tool inputs/results, the secret-bearing message and every non-text
    // block are absent; exactly the two eligible texts arrived, in order.
    let texts = captured_texts(&s.adapter);
    assert_eq!(
        texts,
        vec![
            ("user".into(), "please fix the login bug".into()),
            ("assistant".into(), "looking at auth.rs now".into()),
            (
                "assistant".into(),
                "fixed the session check in auth.rs".into()
            ),
        ],
        "assistant mid-turn text belongs to the assistant role"
    );
    let joined = texts
        .iter()
        .map(|(_, content)| content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !joined.contains("sk-liveabc"),
        "secret never leaves the host"
    );
    assert!(!joined.contains("tool_use"), "tool inputs are excluded");
    assert!(!joined.contains("fn login()"), "tool results are excluded");

    let row = &delivery_rows(&s.db).await[0];
    assert_eq!(row.status, "delivered");
    assert_eq!(row.attempts, 1);
    assert!(
        row.payload.is_none(),
        "payload body is cleared after delivery"
    );
    assert!(!row.payload_hash.is_empty(), "hash survives delivery");
    assert!(row.delivered_at.is_some());
    let accepted: Vec<String> =
        serde_json::from_str(row.upstream_accepted_ids.as_deref().unwrap_or("[]")).unwrap();
    let source: Vec<String> = serde_json::from_str(&row.source_message_ids).unwrap();
    assert_eq!(accepted, source, "patched contract echoes the caller ids");
    assert!(row.safe_error_code.is_none());
}

/// T03: restart (sending recovery + reconciliation) never duplicates rows or
/// L0 content; replaying the same batch is a no-op under the patch contract.
#[tokio::test]
async fn t03_restart_and_replay_never_duplicate_l0() {
    let s = setup_scenario("T03B", vec![provider_config("T03B")]).await;
    let (task_id, run_seq, _) = settle_run(&s, "sess-outbox-1", "review").await;
    enqueue_for_run(&s.db, task_id, run_seq)
        .await
        .expect("enqueue");
    deliver_due(&s.memory).await.expect("deliver");
    let l0_before = s.adapter.l0_count();
    assert!(l0_before > 0);

    // Simulated restart: recover sending rows + reconcile settled runs.
    recover_and_reconcile(&s.memory).await;
    assert_eq!(delivery_rows(&s.db).await.len(), 1, "still exactly one row");
    deliver_due(&s.memory).await.expect("post-restart pass");
    assert_eq!(
        s.adapter.l0_count(),
        l0_before,
        "replay under the patched upsert contract adds no L0 rows"
    );
}

/// T03: a vanilla `v2.0.0` Gateway is detected as unsupported; capture fails
/// terminal without reaching `conversation/add`.
#[tokio::test]
async fn t03_vanilla_gateway_blocks_capture_as_unsupported() {
    let s = setup_scenario("T03C", vec![provider_config("T03C")]).await;
    s.adapter.state().version = "v2.0.0".to_string();
    let (task_id, run_seq, _) = settle_run(&s, "sess-outbox-1", "failed").await;
    enqueue_for_run(&s.db, task_id, run_seq)
        .await
        .expect("enqueue");

    deliver_due(&s.memory).await.expect("deliver");
    let row = &delivery_rows(&s.db).await[0];
    assert_eq!(row.status, "failed");
    assert_eq!(
        row.safe_error_code.as_deref(),
        Some(MemoryErrorClass::UpstreamUnsupported.key())
    );
    assert!(!row.retryable);
    assert!(
        s.adapter.state().captured.is_empty(),
        "no batch may reach a vanilla gateway"
    );
    // A terminal row stays terminal across restarts.
    recover_and_reconcile(&s.memory).await;
    assert_eq!(delivery_rows(&s.db).await[0].status, "failed");
}

/// T05: retryable failure returns to `queued` under exponential backoff and
/// delivers once the upstream recovers.
#[tokio::test]
async fn t05_retryable_failure_requeues_with_backoff_then_delivers() {
    let s = setup_scenario("T05A", vec![provider_config("T05A")]).await;
    let (task_id, run_seq, _) = settle_run(&s, "sess-outbox-1", "review").await;
    enqueue_for_run(&s.db, task_id, run_seq)
        .await
        .expect("enqueue");

    s.adapter.state().fail_capture = Some(MemoryErrorClass::Unavailable);
    deliver_due(&s.memory).await.expect("first pass");
    let row = &delivery_rows(&s.db).await[0];
    assert_eq!(row.status, "queued", "retryable failures requeue");
    assert_eq!(row.attempts, 1);
    assert!(row.retryable);
    assert_eq!(
        row.safe_error_code.as_deref(),
        Some(MemoryErrorClass::Unavailable.key())
    );

    // Backoff: the row is not due immediately after attempt 1.
    s.adapter.state().fail_capture = None;
    assert_eq!(deliver_due(&s.memory).await.expect("second pass"), 0);

    // Elapse the backoff window by rewinding `updated_at`.
    let row = delivery_rows(&s.db).await.pop().unwrap();
    let mut active = row.into_active_model();
    active.updated_at = Set(Utc::now() - chrono::Duration::seconds(10));
    active.update(&s.db.conn).await.unwrap();

    assert_eq!(deliver_due(&s.memory).await.expect("third pass"), 1);
    let row = &delivery_rows(&s.db).await[0];
    assert_eq!(row.status, "delivered");
    assert_eq!(row.attempts, 2);
}

/// T05: non-retryable failure is terminal until an explicit manual retry.
#[tokio::test]
async fn t05_non_retryable_failure_is_terminal_until_manual_retry() {
    let s = setup_scenario("T05B", vec![provider_config("T05B")]).await;
    let (task_id, run_seq, _) = settle_run(&s, "sess-outbox-1", "review").await;
    enqueue_for_run(&s.db, task_id, run_seq)
        .await
        .expect("enqueue");

    s.adapter.state().fail_capture = Some(MemoryErrorClass::Unauthorized);
    deliver_due(&s.memory).await.expect("first pass");
    let row = &delivery_rows(&s.db).await[0];
    assert_eq!(row.status, "failed");
    assert!(!row.retryable);

    // The worker leaves terminal rows alone.
    s.adapter.state().fail_capture = None;
    assert_eq!(deliver_due(&s.memory).await.expect("second pass"), 0);

    // Manual retry (Context UI command) resets the attempt budget.
    assert!(
        memory_capture_service::requeue_for_retry(&s.db.conn, row.id)
            .await
            .expect("manual retry")
    );
    assert_eq!(deliver_due(&s.memory).await.expect("third pass"), 1);
    let row = &delivery_rows(&s.db).await[0];
    assert_eq!(row.status, "delivered");
    assert_eq!(row.attempts, 1);

    // Retrying a delivered row is rejected.
    assert!(
        !memory_capture_service::requeue_for_retry(&s.db.conn, row.id)
            .await
            .expect("retry delivered")
    );
}

/// T05: the automatic attempt budget caps at five.
#[tokio::test]
async fn t05_attempt_budget_caps_at_five() {
    let s = setup_scenario("T05C", vec![provider_config("T05C")]).await;
    let (task_id, run_seq, _) = settle_run(&s, "sess-outbox-1", "review").await;
    enqueue_for_run(&s.db, task_id, run_seq)
        .await
        .expect("enqueue");
    s.adapter.state().fail_capture = Some(MemoryErrorClass::Timeout);

    for attempt in 1..=memory_capture_service::MAX_ATTEMPTS {
        let row = delivery_rows(&s.db).await.pop().unwrap();
        let mut active = row.into_active_model();
        active.updated_at = Set(Utc::now() - chrono::Duration::seconds(120));
        active.update(&s.db.conn).await.unwrap();
        deliver_due(&s.memory).await.expect("attempt pass");
        let row = &delivery_rows(&s.db).await[0];
        assert_eq!(row.attempts, attempt);
        if attempt < memory_capture_service::MAX_ATTEMPTS {
            assert_eq!(row.status, "queued", "attempt {attempt} requeues");
        } else {
            assert_eq!(row.status, "failed", "the fifth attempt is terminal");
        }
    }
}

/// T05: crash recovery — rows stuck in `sending` after a restart go back to
/// `queued` and finish delivering.
#[tokio::test]
async fn startup_recovery_resets_sending_rows() {
    let s = setup_scenario("T05D", vec![provider_config("T05D")]).await;
    let (task_id, run_seq, _) = settle_run(&s, "sess-outbox-1", "review").await;
    enqueue_for_run(&s.db, task_id, run_seq)
        .await
        .expect("enqueue");

    // Simulate a crash mid-delivery: claim the row, then restart.
    assert!(
        memory_capture_service::mark_sending(&s.db.conn, delivery_rows(&s.db).await[0].id)
            .await
            .expect("claim")
    );
    assert_eq!(delivery_rows(&s.db).await[0].status, "sending");

    recover_and_reconcile(&s.memory).await;
    assert_eq!(delivery_rows(&s.db).await[0].status, "queued");
    assert_eq!(deliver_due(&s.memory).await.expect("deliver"), 1);
    assert_eq!(delivery_rows(&s.db).await[0].status, "delivered");
}

/// T05: reconciliation stages deliveries for settled runs that lost their
/// enqueue to a crash after settlement.
#[tokio::test]
async fn reconciliation_stages_missing_deliveries() {
    let s = setup_scenario("T05E", vec![provider_config("T05E")]).await;
    let (task_id, run_seq, _) = settle_run(&s, "sess-outbox-1", "review").await;
    // Deliberately no enqueue_for_run — the crash window after settlement.
    assert!(delivery_rows(&s.db).await.is_empty());

    recover_and_reconcile(&s.memory).await;
    assert_eq!(
        delivery_rows(&s.db).await.len(),
        1,
        "missing row reconciled"
    );
    assert_eq!(delivery_rows(&s.db).await[0].task_id, task_id);
    assert_eq!(delivery_rows(&s.db).await[0].run_seq, run_seq);
}

/// T03/AC08: cancelled runs, merge-only generations (no conversation) and
/// legacy projects without a memory provider stage nothing.
#[tokio::test]
async fn ineligible_runs_and_legacy_projects_stage_nothing() {
    let s = setup_scenario("T03D", vec![provider_config("T03D")]).await;

    // Cancelled run generation.
    let (task_id, run_seq, _) = settle_run(&s, "sess-outbox-1", "canceled").await;
    assert_eq!(
        enqueue_for_run(&s.db, task_id, run_seq)
            .await
            .expect("enqueue"),
        0
    );

    // Merge-only generation: settled without a conversation.
    let draft = WorkTaskDraft {
        folder_id: s.folder_id,
        title: "merge only".into(),
        config: serde_json::json!({
            "display_text": "merge only",
            "prompt_blocks": [{"type":"text","text":"merge only"}],
        }),
        task_kind: Default::default(),
    };
    let created = work_task_service::create(&s.db.conn, draft)
        .await
        .expect("create");
    let seq = work_task_service::claim_for_run(
        &s.db.conn,
        created.id,
        work_task::WorkTaskStatus::Todo,
        "test",
    )
    .await
    .expect("claim")
    .expect("seq");
    specos_runtime_service::update_run_state(&s.db.conn, created.id, seq, "review", None, true)
        .await
        .expect("settle");
    assert_eq!(
        enqueue_for_run(&s.db, created.id, seq)
            .await
            .expect("enqueue"),
        0
    );
    assert!(delivery_rows(&s.db).await.is_empty());
}

#[tokio::test]
async fn legacy_project_without_memory_provider_stages_nothing() {
    let s = setup_scenario("T03E", vec![]).await; // no providers at all
    let (task_id, run_seq, _) = settle_run(&s, "sess-outbox-1", "review").await;
    assert_eq!(
        enqueue_for_run(&s.db, task_id, run_seq)
            .await
            .expect("enqueue"),
        0
    );
    assert!(delivery_rows(&s.db).await.is_empty());
    assert_eq!(deliver_due(&s.memory).await.expect("deliver"), 0);
    assert!(
        s.adapter.state().captured.is_empty(),
        "zero Memory network behavior for legacy projects"
    );
}

/// Migration: `work_task_context_pack.memory_evidence` exists and is NULL
/// for legacy packages (the column the recall evidence persists into).
#[tokio::test]
#[allow(clippy::await_holding_lock)] // see setup_scenario: env serialization
async fn migration_adds_memory_evidence_column() {
    let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let db = fresh_in_memory_db().await;
    let value: Option<String> = sea_orm::ConnectionTrait::query_one(
        &db.conn,
        sea_orm::Statement::from_string(
            sea_orm::DbBackend::Sqlite,
            "SELECT memory_evidence FROM work_task_context_pack LIMIT 1".to_owned(),
        ),
    )
    .await
    .expect("memory_evidence column exists")
    .and_then(|row| row.try_get("", "memory_evidence").ok());
    assert!(value.is_none());
    drop(lock);
}

/// Cap sanity through the DB path: staged rows never exceed the message cap.
#[tokio::test]
async fn staged_payload_respects_the_message_cap() {
    let s = setup_scenario("T03F", vec![provider_config("T03F")]).await;
    let (task_id, run_seq, _) = settle_run(&s, "sess-outbox-1", "review").await;
    enqueue_for_run(&s.db, task_id, run_seq)
        .await
        .expect("enqueue");
    let row = &delivery_rows(&s.db).await[0];
    let payload = row.payload.as_deref().expect("staged payload");
    let messages = serde_json::from_str::<serde_json::Value>(payload).unwrap()["messages"]
        .as_array()
        .unwrap()
        .len();
    assert!(messages <= MAX_CAPTURE_MESSAGES);
    assert!(
        !payload.contains("sk-liveabc"),
        "the staged payload itself never holds secret-bearing messages"
    );
}
