//! Memory recall → Context Package integration oracles
//! (BUGRAIL-SPECOS-017 Test Spec T04/T05, R04/R05).
//!
//! All recall runs against the deterministic in-memory Adapter — zero
//! network. Authoritative evidence is SQLite: the immutable package rows,
//! their `memory_evidence` column and Context Activity. Tests serialize on
//! `ENV_LOCK` because provider credential references are process-global
//! environment variables.

use std::collections::BTreeSet;
use std::sync::{LazyLock, Mutex, MutexGuard};

use codeg_lib::context;
use codeg_lib::db::entities::{context_activity, work_task, work_task_context_pack};
use codeg_lib::db::service::work_task_service;
use codeg_lib::db::test_helpers::{fresh_in_memory_db, seed_folder};
use codeg_lib::memory::test_adapter::DeterministicMemoryAdapter;
use codeg_lib::memory::{AdapterRegistry, MemoryLayer, MemoryRecallHit, MemoryService};
use codeg_lib::models::{
    ContextConfig, ContextLoadout, ContextProviderConfig, ContextSourceConfig, WorkTaskDraft,
};
use codeg_lib::specos_control;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

// ── fixtures ────────────────────────────────────────────────────────────────

struct Scenario {
    _lock: MutexGuard<'static, ()>,
    db: codeg_lib::db::AppDatabase,
    memory: MemoryService,
    adapter: DeterministicMemoryAdapter,
    folder_id: i32,
    root: std::path::PathBuf,
    _temp: tempfile::TempDir,
}

fn provider_config(env_suffix: &str, required: bool, include_core: bool) -> ContextProviderConfig {
    let mut capabilities = vec![
        codeg_lib::memory::CAP_CAPTURE.into(),
        codeg_lib::memory::CAP_RECALL_L1.into(),
    ];
    if include_core {
        capabilities.push(codeg_lib::memory::CAP_RECALL_L3.into());
    }
    ContextProviderConfig {
        id: "project-memory".into(),
        kind: "memory".into(),
        adapter: Some(codeg_lib::memory::ADAPTER_TENCENTDB_V3.into()),
        endpoint: Some("http://127.0.0.1:9".into()), // never dialed
        secret_env: Some(format!("MEM_RECALL_SECRET_{env_suffix}")),
        service_id_env: Some(format!("MEM_RECALL_SERVICE_{env_suffix}")),
        team_id: Some(format!("team-{env_suffix}")),
        user_id_env: Some(format!("MEM_RECALL_USER_{env_suffix}")),
        default_agent_id: Some("bugrail-agent".into()),
        capabilities,
        enabled: true,
        required,
        include_core,
        ..Default::default()
    }
}

fn set_provider_env(env_suffix: &str) {
    // SAFETY: unique per-scenario names, guarded by ENV_LOCK.
    std::env::set_var(format!("MEM_RECALL_SECRET_{env_suffix}"), "s");
    std::env::set_var(format!("MEM_RECALL_SERVICE_{env_suffix}"), "svc");
    std::env::set_var(format!("MEM_RECALL_USER_{env_suffix}"), "user");
}

// The ENV_LOCK guard must span every await: it serializes process-wide env
// mutation (provider env references).
#[allow(clippy::await_holding_lock)]
async fn setup_scenario(
    env_suffix: &str,
    required: bool,
    include_core: bool,
    max_bytes: usize,
) -> Scenario {
    let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    set_provider_env(env_suffix);

    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create project root");
    std::fs::write(root.join("AGENTS.md"), "# a\n").expect("write local source");

    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, root.to_str().unwrap()).await;

    let config = ContextConfig {
        version: 1,
        default_loadout_id: "default".into(),
        providers: vec![provider_config(env_suffix, required, include_core)],
        loadouts: vec![ContextLoadout {
            id: "default".into(),
            name: "Default".into(),
            sources: vec![ContextSourceConfig {
                path: "AGENTS.md".into(),
                required: true,
                kind: "rules".into(),
            }],
            provider_ids: vec!["project-memory".into()],
            max_items: 16,
            max_bytes,
            max_tokens: 8_000,
        }],
        validation_errors: vec![],
    };
    specos_control::save_context(&root, config).expect("save context config");

    let adapter = DeterministicMemoryAdapter::new();
    let memory = MemoryService::new_with_registry(
        codeg_lib::db::AppDatabase {
            conn: db.conn.clone(),
        },
        AdapterRegistry::deterministic(std::sync::Arc::new(adapter.clone())),
    );

    Scenario {
        _lock: lock,
        db,
        memory,
        adapter,
        folder_id,
        root,
        _temp: temp,
    }
}

/// Create a task with a fixed title (the recall query) and claim run 1.
async fn claim(s: &Scenario, title: &str) -> (i32, i32) {
    let created = work_task_service::create(
        &s.db.conn,
        WorkTaskDraft {
            folder_id: s.folder_id,
            title: title.into(),
            config: serde_json::json!({
                "display_text": title,
                "prompt_blocks": [{"type":"text","text":title}],
            }),
            task_kind: Default::default(),
        },
    )
    .await
    .expect("create task");
    let seq = work_task_service::claim_for_run(
        &s.db.conn,
        created.id,
        work_task::WorkTaskStatus::Todo,
        "test",
    )
    .await
    .expect("claim run")
    .expect("claim seq");
    (created.id, seq)
}

async fn prepare(s: &Scenario, task_id: i32, run_seq: i32) -> codeg_lib::context::PreparedContext {
    context::prepare_run(
        &s.db.conn,
        s.folder_id,
        &s.root,
        &s.root,
        task_id,
        run_seq,
        None,
        Vec::new(),
        &s.memory,
    )
    .await
    .expect("prepare_run")
}

fn hit(remote_id: &str, layer: MemoryLayer, score: Option<f64>, content: &str) -> MemoryRecallHit {
    MemoryRecallHit {
        remote_id: remote_id.into(),
        layer,
        score,
        content: content.into(),
        captured_at: Some("2026-08-01T00:00:00Z".into()),
    }
}

/// The persisted memory_evidence JSON for one (task, run).
async fn evidence_json(s: &Scenario, task_id: i32) -> serde_json::Value {
    let row = work_task_context_pack::Entity::find()
        .filter(work_task_context_pack::Column::TaskId.eq(task_id))
        .one(&s.db.conn)
        .await
        .expect("pack row")
        .expect("pack row exists");
    let raw = row.memory_evidence.expect("memory_evidence is persisted");
    serde_json::from_str(&raw).expect("memory_evidence is valid JSON")
}

async fn activity_rows(s: &Scenario) -> Vec<context_activity::Model> {
    context_activity::Entity::find()
        .filter(context_activity::Column::FolderId.eq(s.folder_id))
        .filter(context_activity::Column::Kind.eq("memory.recall"))
        .all(&s.db.conn)
        .await
        .expect("activity rows")
}

async fn pack_count(s: &Scenario) -> u64 {
    use sea_orm::PaginatorTrait;
    work_task_context_pack::Entity::find()
        .count(&s.db.conn)
        .await
        .expect("pack count")
}

fn reasons(entries: &[serde_json::Value]) -> Vec<&str> {
    entries
        .iter()
        .filter_map(|entry| entry.get("reason").and_then(|r| r.as_str()))
        .collect()
}

// ── T04 ─────────────────────────────────────────────────────────────────────

/// T04: ordered L1 + optional L3 merged after local candidates; duplicate
/// content deduped (local and remote winners); oversized excluded with
/// reason; provenance carries provider/adapter/layer/remoteId/score/
/// queryHash; memory_evidence records included/excluded decisions; no remote
/// content leaks into the evidence column.
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn t04_ordered_merge_dedup_oversized_and_provenance_evidence() {
    let s = setup_scenario("T04A", false, true, 200).await;
    {
        let mut state = s.adapter.state();
        state.l1 = vec![
            hit("l1-big", MemoryLayer::L1, Some(0.95), &"x".repeat(300)),
            hit("l1-dup", MemoryLayer::L1, Some(0.9), "shared dup body"),
            hit("l1-low", MemoryLayer::L1, Some(0.1), "low score body"),
        ];
        state.l3 = vec![
            hit("core-a", MemoryLayer::L3, None, "core body"),
            hit("core-dup", MemoryLayer::L3, None, "shared dup body"),
        ];
    }
    let (task_id, seq) = claim(&s, "memory ctx").await;
    let prepared = prepare(&s, task_id, seq).await;

    // Fixed order: local file, then L3, then L1 by score desc (the oversized
    // 300-byte item is gone; the duplicate content survives exactly once via
    // its first (L3) occurrence).
    let sources: Vec<&str> = prepared
        .package
        .items
        .iter()
        .map(|item| item.source.as_str())
        .collect();
    assert_eq!(
        sources,
        vec![
            "AGENTS.md",
            "tencentdb-agent-memory-v3/l3",
            "tencentdb-agent-memory-v3/l3",
            "tencentdb-agent-memory-v3/l1",
        ],
        "L3 items precede L1 items; both follow local sources"
    );
    let memory_items: Vec<_> = prepared
        .package
        .items
        .iter()
        .filter(|item| item.kind == "memory")
        .collect();
    assert_eq!(memory_items[0].provenance["remoteId"], "core-a");
    // L3 duplicate wins over the later L1 duplicate (first occurrence).
    assert_eq!(memory_items[1].provenance["remoteId"], "core-dup");
    assert_eq!(memory_items[2].provenance["remoteId"], "l1-low");
    let dup_item = &memory_items[1];
    assert_eq!(dup_item.content, "shared dup body");

    // Safe provenance on every memory item (R05).
    for item in &memory_items {
        assert_eq!(item.provenance["provider"], "project-memory");
        assert_eq!(
            item.provenance["adapter"],
            codeg_lib::memory::ADAPTER_TENCENTDB_V3
        );
        assert!(item.provenance["layer"].is_string());
        assert!(item.provenance["remoteId"].is_string());
        assert!(item.provenance["queryHash"].is_string());
        assert!(!item.required, "memory items never gate a run");
    }
    assert_eq!(memory_items[2].provenance["score"], 0.1);

    // Evidence: 3 included (l3 core-a, l3 core-dup, l1-low), 2 excluded
    // (oversized l1-big, duplicate l1-dup).
    let evidence = evidence_json(&s, task_id).await;
    let provider = &evidence["providers"][0];
    assert_eq!(provider["provider"], "project-memory");
    assert_eq!(
        provider["queryHash"],
        prepared.package.items[1].provenance["queryHash"]
    );
    let included = provider["included"].as_array().expect("included array");
    assert_eq!(included.len(), 3);
    let included_remote: BTreeSet<_> = included
        .iter()
        .map(|e| e["remoteId"].as_str().expect("remoteId"))
        .collect();
    assert_eq!(
        included_remote,
        BTreeSet::from(["core-a", "core-dup", "l1-low"])
    );
    for entry in included {
        assert!(entry["contentHash"].is_string());
        assert!(entry["layer"].is_string());
    }
    let excluded = provider["excluded"].as_array().expect("excluded array");
    let mut excluded_reasons = reasons(excluded);
    excluded_reasons.sort_unstable();
    assert_eq!(excluded_reasons, vec!["duplicate", "oversized"]);
    let oversized = excluded
        .iter()
        .find(|e| e["reason"] == "oversized")
        .expect("oversized entry");
    assert_eq!(oversized["remoteId"], "l1-big");
    assert_eq!(oversized["layer"], "l1");

    // Untrusted-data handling: no remote content and no query text in the
    // persisted evidence.
    let raw = work_task_context_pack::Entity::find()
        .filter(work_task_context_pack::Column::TaskId.eq(task_id))
        .one(&s.db.conn)
        .await
        .unwrap()
        .unwrap()
        .memory_evidence
        .unwrap();
    assert!(!raw.contains("low score body"));
    assert!(!raw.contains("core body"));
    assert!(!raw.contains("memory ctx"));
    assert_eq!(prepared.package.status, "ready");
}

/// T04: exact budget edge — an item exactly filling the remaining byte
/// budget is included; the next item, however small, is excluded with
/// `budget_bytes`.
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn t04_exact_budget_edge_included_then_budget_bytes() {
    // AGENTS.md is 4 bytes; budget 16 leaves 12 for memory.
    let s = setup_scenario("T04B", false, false, 16).await;
    {
        let mut state = s.adapter.state();
        state.l1 = vec![
            hit("ten", MemoryLayer::L1, Some(0.9), &"a".repeat(10)),
            hit("exact", MemoryLayer::L1, Some(0.5), "bb"),
            hit("one", MemoryLayer::L1, Some(0.3), "c"),
        ];
    }
    let (task_id, seq) = claim(&s, "edge ctx").await;
    let prepared = prepare(&s, task_id, seq).await;

    let remote: Vec<&str> = prepared
        .package
        .items
        .iter()
        .filter(|item| item.kind == "memory")
        .map(|item| item.provenance["remoteId"].as_str().unwrap())
        .collect();
    assert_eq!(remote, vec!["ten", "exact"], "exact-fit item is included");
    assert_eq!(prepared.package.total_bytes, 16);

    let evidence = evidence_json(&s, task_id).await;
    let provider = &evidence["providers"][0];
    assert_eq!(provider["included"].as_array().unwrap().len(), 2);
    assert_eq!(
        reasons(provider["excluded"].as_array().unwrap()),
        vec!["budget_bytes"]
    );
}

/// T04: empty recall is a successful recall — no memory items, ready
/// package, and an evidence entry with reason `empty`.
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn t04_empty_recall_yields_ready_package_and_empty_evidence() {
    let s = setup_scenario("T04C", false, true, 4_096).await;
    let (task_id, seq) = claim(&s, "nothing to recall").await;
    let prepared = prepare(&s, task_id, seq).await;

    assert_eq!(prepared.package.status, "ready");
    assert!(prepared
        .package
        .items
        .iter()
        .all(|item| item.kind != "memory"));
    let evidence = evidence_json(&s, task_id).await;
    let provider = &evidence["providers"][0];
    assert!(provider["included"]
        .as_array()
        .expect("included")
        .is_empty());
    assert_eq!(
        reasons(provider["excluded"].as_array().expect("excluded")),
        vec!["empty"]
    );
    assert_eq!(s.adapter.state().recalls, 1, "empty recall still ran");
}

/// T04: the package content_hash is deterministic across identical
/// compilations and differs when recall hits differ.
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn t04_deterministic_content_hash_across_compilations() {
    let s = setup_scenario("T04D", false, false, 4_096).await;
    {
        let mut state = s.adapter.state();
        state.l1 = vec![
            hit("b", MemoryLayer::L1, Some(0.4), "body b"),
            hit("a", MemoryLayer::L1, Some(0.9), "body a"),
        ];
    }
    let (first_task, first_seq) = claim(&s, "same title").await;
    let (second_task, second_seq) = claim(&s, "same title").await;
    let first = prepare(&s, first_task, first_seq).await;
    let second = prepare(&s, second_task, second_seq).await;

    assert_ne!(first.package.id, second.package.id);
    assert_eq!(
        first.package.content_hash, second.package.content_hash,
        "identical inputs must compile to the same package hash"
    );
    // Fixed ordering inside one package: score desc with remote-id
    // tie-break.
    let remote: Vec<_> = first
        .package
        .items
        .iter()
        .filter(|item| item.kind == "memory")
        .map(|item| item.provenance["remoteId"].as_str().unwrap())
        .collect();
    assert_eq!(remote, vec!["a", "b"]);

    // Change the recall hits: same query, different package hash.
    s.adapter.state().l1 = vec![hit("a", MemoryLayer::L1, Some(0.9), "different body")];
    let (third_task, third_seq) = claim(&s, "same title").await;
    let third = prepare(&s, third_task, third_seq).await;
    assert_ne!(
        first.package.content_hash, third.package.content_hash,
        "different recall hits must change the package hash"
    );
}

// ── T05 ─────────────────────────────────────────────────────────────────────

/// T05: a required Memory provider failing recall blocks prepare with a
/// Validation error and records a degraded activity — before any package is
/// written.
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn t05_required_recall_failure_blocks_before_any_package() {
    let s = setup_scenario("T05A", true, true, 4_096).await;
    s.adapter.state().fail_recall = Some(codeg_lib::memory::MemoryErrorClass::Timeout);
    let (task_id, seq) = claim(&s, "must not launch").await;

    let err = context::prepare_run(
        &s.db.conn,
        s.folder_id,
        &s.root,
        &s.root,
        task_id,
        seq,
        None,
        Vec::new(),
        &s.memory,
    )
    .await
    .err()
    .expect("required recall failure must block prepare");
    assert!(
        err.to_string().contains("required Memory provider"),
        "{err}"
    );
    assert_eq!(
        pack_count(&s).await,
        0,
        "no package row may exist when a required provider blocks"
    );
    let activity = activity_rows(&s).await;
    assert!(activity.iter().any(|row| row.status == "degraded"
        && row.provider_id.as_deref() == Some("project-memory")
        && row.message.as_deref() == Some("memory.timeout")));
}

/// T05: a required Memory provider whose configuration cannot be resolved
/// (missing credential reference) blocks with no recall attempt and no
/// package.
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn t05_required_unresolved_config_blocks_without_recall() {
    use sea_orm::PaginatorTrait;
    let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().to_path_buf();
    std::fs::write(root.join("AGENTS.md"), "# a\n").unwrap();
    // MEM_RECALL_SECRET_T05B is deliberately never set.
    let db = fresh_in_memory_db().await;
    let folder_id = seed_folder(&db, root.to_str().unwrap()).await;
    let mut provider = provider_config("T05B", true, false);
    provider.secret_env = Some("MEM_RECALL_SECRET_T05B".into());
    specos_control::save_context(
        &root,
        ContextConfig {
            version: 1,
            default_loadout_id: "default".into(),
            providers: vec![provider],
            loadouts: vec![ContextLoadout {
                id: "default".into(),
                name: "Default".into(),
                sources: vec![ContextSourceConfig {
                    path: "AGENTS.md".into(),
                    required: true,
                    kind: "rules".into(),
                }],
                provider_ids: vec!["project-memory".into()],
                max_items: 16,
                max_bytes: 4_096,
                max_tokens: 8_000,
            }],
            validation_errors: vec![],
        },
    )
    .expect("save context");

    let adapter = DeterministicMemoryAdapter::new();
    let memory = MemoryService::new_with_registry(
        codeg_lib::db::AppDatabase {
            conn: db.conn.clone(),
        },
        AdapterRegistry::deterministic(std::sync::Arc::new(adapter.clone())),
    );

    let created = work_task_service::create(
        &db.conn,
        WorkTaskDraft {
            folder_id,
            title: "blocked".into(),
            config: serde_json::json!({
                "display_text": "blocked",
                "prompt_blocks": [{"type":"text","text":"blocked"}],
            }),
            task_kind: Default::default(),
        },
    )
    .await
    .unwrap();
    let seq = work_task_service::claim_for_run(
        &db.conn,
        created.id,
        work_task::WorkTaskStatus::Todo,
        "test",
    )
    .await
    .unwrap()
    .unwrap();

    let err = context::prepare_run(
        &db.conn,
        folder_id,
        &root,
        &root,
        created.id,
        seq,
        None,
        Vec::new(),
        &memory,
    )
    .await
    .err()
    .expect("unresolved required provider must block");
    // The required health gate fires first: identity resolution happens
    // inside health, so no request is issued and prepare blocks as an
    // unavailable required provider.
    assert!(
        err.to_string().contains("required context provider"),
        "{err}"
    );
    assert_eq!(
        work_task_context_pack::Entity::find()
            .count(&db.conn)
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        adapter.state().recalls,
        0,
        "no recall may be attempted on unresolved identity"
    );
    // A blocked activity is recorded for the required provider.
    let blocked = context_activity::Entity::find()
        .filter(context_activity::Column::FolderId.eq(folder_id))
        .filter(context_activity::Column::Status.eq("blocked"))
        .all(&db.conn)
        .await
        .expect("activity rows");
    assert!(blocked
        .iter()
        .any(|row| row.provider_id.as_deref() == Some("project-memory")));
    drop(lock);
}

/// T05: an optional provider failing recall yields a ready package with a
/// degraded activity, no memory items, and evidence recording the adapter
/// error class.
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn t05_optional_recall_failure_degrades_but_prepares() {
    let s = setup_scenario("T05C", false, true, 4_096).await;
    s.adapter.state().fail_recall = Some(codeg_lib::memory::MemoryErrorClass::Unavailable);
    let (task_id, seq) = claim(&s, "degraded but usable").await;
    let prepared = prepare(&s, task_id, seq).await;

    assert_eq!(prepared.package.status, "ready");
    assert!(prepared
        .package
        .items
        .iter()
        .all(|item| item.kind != "memory"));
    let activity = activity_rows(&s).await;
    assert!(activity.iter().any(
        |row| row.status == "degraded" && row.message.as_deref() == Some("memory.unavailable")
    ));
    let evidence = evidence_json(&s, task_id).await;
    let provider = &evidence["providers"][0];
    assert_eq!(
        reasons(provider["excluded"].as_array().expect("excluded")),
        vec!["adapter_error:memory.unavailable"]
    );
}

/// T05: restart/inspection — a second prepare for the same (task_id,
/// run_seq) returns the same immutable package (early return, no second
/// recall) with the memory evidence intact.
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn t05_second_prepare_returns_immutable_package_with_evidence() {
    let s = setup_scenario("T05D", false, true, 4_096).await;
    {
        let mut state = s.adapter.state();
        state.l1 = vec![hit("l1-a", MemoryLayer::L1, Some(0.8), " recalled body")];
        state.l3 = vec![hit("core-a", MemoryLayer::L3, None, "core body")];
    }
    let (task_id, seq) = claim(&s, "inspect me").await;
    let first = prepare(&s, task_id, seq).await;
    assert_eq!(s.adapter.state().recalls, 1);
    let evidence_first = evidence_json(&s, task_id).await;

    // Simulated restart re-inspection: the run package is immutable.
    let second = prepare(&s, task_id, seq).await;
    assert_eq!(first.package.id, second.package.id);
    assert_eq!(first.package.content_hash, second.package.content_hash);
    assert_eq!(s.adapter.state().recalls, 1, "no second recall on replay");
    assert_eq!(evidence_json(&s, task_id).await, evidence_first);

    // And package_get serves the same persisted facts.
    let fetched = context::package_get(&s.db.conn, &first.package.id)
        .await
        .expect("package_get");
    assert_eq!(fetched.content_hash, first.package.content_hash);
    assert_eq!(fetched.items.len(), first.package.items.len());
}
