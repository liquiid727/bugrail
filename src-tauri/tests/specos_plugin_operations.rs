//! Command-core and transport oracles for BUGRAIL-SPECOS-028.T04-T05.
//!
//! The tests inspect persisted/config-derived projections rather than event
//! delivery. They also serialize the returned values to prove that credentials,
//! request bodies and lease material do not cross the frontend boundary.

use axum_test::TestServer;
use chrono::Utc;
use codeg_lib::app_state::AppState;
use codeg_lib::commands::specos_control::context_plugin_operations_core;
use codeg_lib::db::service::provider_job_service;
use codeg_lib::db::test_helpers::{fresh_in_memory_db, seed_folder};
use codeg_lib::models::{ContextConfig, ContextLoadout, ContextProviderConfig};
use codeg_lib::web::router::build_router;
use codeg_lib::web::shutdown::ShutdownSignal;
use sha2::{Digest, Sha256};
use std::sync::Arc;

const TOKEN: &str = "plugin-operations-test-token";

fn config() -> ContextConfig {
    let providers = vec![
        provider("wiki", "wiki", "deterministic-wiki"),
        provider("graph", "codegraph", "deterministic-codegraph"),
        provider("skills", "skill", "deterministic-skill"),
    ];
    ContextConfig {
        version: 1,
        default_loadout_id: "default".into(),
        loadouts: vec![ContextLoadout {
            id: "default".into(),
            name: "Default".into(),
            provider_ids: providers.iter().map(|item| item.id.clone()).collect(),
            max_items: 16,
            max_bytes: 32 * 1024,
            max_tokens: 8_000,
            ..Default::default()
        }],
        providers,
        validation_errors: Vec::new(),
    }
}

fn provider(id: &str, kind: &str, adapter: &str) -> ContextProviderConfig {
    ContextProviderConfig {
        id: id.into(),
        kind: kind.into(),
        adapter: Some(adapter.into()),
        capabilities: vec!["health".into()],
        ..Default::default()
    }
}

fn request_hash(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

#[tokio::test]
async fn t04_operations_projection_redacts_config_and_job_secrets() {
    let db = fresh_in_memory_db().await;
    let project = tempfile::tempdir().unwrap();
    let folder_id = seed_folder(&db, project.path().to_str().unwrap()).await;
    codeg_lib::specos_control::save_context(project.path(), config()).unwrap();

    let row = provider_job_service::submit(
        &db.conn,
        provider_job_service::ProviderJobSpec {
            provider_kind: "wiki".into(),
            provider_id: "wiki".into(),
            operation: "sync".into(),
            idempotency_key: "Authorization bearer super-secret".into(),
            request_hash: request_hash("provider payload"),
            max_attempts: 3,
        },
        Utc::now(),
    )
    .await
    .unwrap();
    let memory = codeg_lib::memory::MemoryService::new(codeg_lib::db::AppDatabase {
        conn: db.conn.clone(),
    });
    let projection = context_plugin_operations_core(
        &db,
        &memory,
        folder_id,
        None,
        None,
        Some(20),
    )
    .await
    .unwrap();
    assert_eq!(projection.config.len(), 3);
    assert_eq!(projection.health.len(), 3);
    assert_eq!(projection.jobs.len(), 1);
    assert_eq!(projection.jobs[0].id, row.id);
    assert!(projection.jobs[0].attempts.is_empty());

    let encoded = serde_json::to_string(&projection).unwrap();
    assert!(!encoded.contains("super-secret"));
    assert!(!encoded.contains("Authorization"));
    assert!(!encoded.contains("lease_token_hash"));
    assert!(!encoded.contains("request body"));
    assert!(encoded.contains(&request_hash("provider payload")));
}

#[tokio::test]
async fn t05_axum_operation_projection_reconstructs_without_events() {
    let db = fresh_in_memory_db().await;
    let project = tempfile::tempdir().unwrap();
    let folder_id = seed_folder(&db, project.path().to_str().unwrap()).await;
    codeg_lib::specos_control::save_context(project.path(), config()).unwrap();
    provider_job_service::submit(
        &db.conn,
        provider_job_service::ProviderJobSpec {
            provider_kind: "codegraph".into(),
            provider_id: "graph".into(),
            operation: "index".into(),
            idempotency_key: "stable-operation-key".into(),
            request_hash: request_hash("index request"),
            max_attempts: 2,
        },
        Utc::now(),
    )
    .await
    .unwrap();

    let data_dir = tempfile::tempdir().unwrap();
    let static_dir = tempfile::tempdir().unwrap();
    let state = Arc::new(AppState::new_for_test(
        db,
        data_dir.path().to_path_buf(),
    ));
    let router = build_router(
        state,
        TOKEN.to_string(),
        static_dir.path().to_path_buf(),
        Arc::new(ShutdownSignal::new()),
    );
    let server = TestServer::new(router).unwrap();
    let response = server
        .post("/api/specos_context_plugin_operations_get")
        .add_header("authorization", format!("Bearer {TOKEN}"))
        .json(&serde_json::json!({
            "folderId": folder_id,
            "limit": 20
        }))
        .await;
    assert_eq!(response.status_code(), 200);
    let body: serde_json::Value = response.json();
    assert_eq!(body["config"].as_array().unwrap().len(), 3);
    assert_eq!(body["jobs"].as_array().unwrap().len(), 1);
    assert_eq!(body["jobs"][0]["providerId"], "graph");
    assert!(body["jobs"][0].get("idempotencyKey").is_none());
    assert!(body["jobs"][0].get("leaseTokenHash").is_none());
    assert!(!body.to_string().contains("stable-operation-key"));
}

#[tokio::test]
async fn malformed_endpoint_is_not_reflected_in_operations_or_overview() {
    let db = fresh_in_memory_db().await;
    let project = tempfile::tempdir().unwrap();
    let folder_id = seed_folder(&db, project.path().to_str().unwrap()).await;
    let dir = project.path().join(".codeg");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("context.yaml"),
        "version: 1\ndefaultLoadoutId: default\nproviders:\n  - id: wiki\n    kind: wiki\n    adapter: deterministic-wiki\n    enabled: true\n    endpoint: \\\"https://operator:credential@example.invalid\\\"\nloadouts:\n  - id: default\n    name: Default\n    providerIds: [wiki]\n    maxItems: 16\n    maxBytes: 32768\n    maxTokens: 8000\n",
    )
    .unwrap();
    let memory = codeg_lib::memory::MemoryService::new(codeg_lib::db::AppDatabase {
        conn: db.conn.clone(),
    });
    let projection = context_plugin_operations_core(
        &db,
        &memory,
        folder_id,
        None,
        None,
        None,
    )
    .await
    .unwrap();
    let encoded = serde_json::to_string(&projection).unwrap();
    assert!(!encoded.contains("credential"));
    assert!(!encoded.contains("operator"));
    assert_eq!(projection.config[0].endpoint, None);
    assert_eq!(projection.health[0].status, "degraded");
}
