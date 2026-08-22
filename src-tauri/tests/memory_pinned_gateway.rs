//! BUGRAIL-SPECOS-017 T08: pinned TencentDB Agent Memory `v2.0.0+bugrail.1`
//! integration verification.
//!
//! Gated behind `BUGRAIL_T08_URL` (e.g. `http://127.0.0.1:18420`). The
//! fixture is the pinned upstream commit
//! `0aff21a2d9f2b8a0354aaa80a2e586aab4054562` plus the `bugrail.1` patch
//! (caller-id upsert + `/health` version). Runs `#[ignore]` in CI; the
//! release evidence run executes it with `--ignored` while the fixture is
//! up and records commands, exit codes and redacted traces.
//!
//! Oracles (test-spec T08):
//! - `/health` reports the exact patched version and the provider is
//!   writable.
//! - Capture with stable message ids is idempotent: replay returns the same
//!   accepted ids and `/v3/conversation/count` does not grow.
//! - Two `team_id` values stay isolated in recall.
//! - A later recall (after BugRail restart, i.e. a fresh `MemoryService`
//!   over the same persistent fixture) returns matching L1 provenance,
//!   polled every 2s for at most 120s to absorb async L1 extraction.

use std::time::Duration;

use codeg_lib::memory::{
    MemoryCaptureBatch, MemoryCaptureMessage, MemoryLayer, MemoryRecallRequest, MemoryRole,
    MemoryService, ADAPTER_TENCENTDB_V3, CAP_CAPTURE, CAP_RECALL_L1,
};
use codeg_lib::models::ContextProviderConfig;

struct Fixture {
    url: String,
    secret: String,
    service_id: String,
    user_id: String,
}

fn fixture() -> Option<Fixture> {
    let url = std::env::var("BUGRAIL_T08_URL").ok()?;
    Some(Fixture {
        url: url.trim_end_matches('/').to_string(),
        secret: std::env::var("BUGRAIL_T08_SECRET").unwrap_or_else(|_| "t08-secret".into()),
        service_id: std::env::var("BUGRAIL_T08_SERVICE_ID")
            .unwrap_or_else(|_| "t08-service".into()),
        user_id: std::env::var("BUGRAIL_T08_USER_ID").unwrap_or_else(|_| "t08-user".into()),
    })
}

fn provider_config(fx: &Fixture, team_id: &str) -> ContextProviderConfig {
    ContextProviderConfig {
        id: format!("pinned-{team_id}"),
        kind: codeg_lib::memory::MEMORY_KIND.into(),
        adapter: Some(ADAPTER_TENCENTDB_V3.into()),
        endpoint: Some(fx.url.clone()),
        // Pinned fixture credentials enter through env references, exactly
        // like production config (values are loopback test secrets).
        secret_env: Some("BUGRAIL_T08_SECRET".into()),
        service_id_env: Some("BUGRAIL_T08_SERVICE_ID".into()),
        team_id: Some(team_id.into()),
        user_id_env: Some("BUGRAIL_T08_USER_ID".into()),
        default_agent_id: Some("bugrail-t08-agent".into()),
        capabilities: vec![CAP_CAPTURE.into(), CAP_RECALL_L1.into()],
        enabled: true,
        capture_enabled: true,
        recall_enabled: true,
        include_core: false,
        timeout_ms: 5_000,
        ..Default::default()
    }
}

async fn service() -> MemoryService {
    let db = codeg_lib::db::test_helpers::fresh_in_memory_db().await;
    MemoryService::new_with_registry(
        codeg_lib::db::AppDatabase { conn: db.conn },
        codeg_lib::memory::AdapterRegistry::production(),
    )
}

fn batch(team: &str, token: &str) -> MemoryCaptureBatch {
    MemoryCaptureBatch {
        team_id: team.into(),
        agent_id: "bugrail-t08-agent".into(),
        user_id: "t08-user".into(),
        session_id: format!("t08-session-{team}"),
        task_id: format!("t08-task-{team}"),
        messages: vec![
            MemoryCaptureMessage {
                id: format!("t08-{team}-m1"),
                role: MemoryRole::User,
                content: format!("captured fact T8TOK-{token} for later recall"),
            },
            MemoryCaptureMessage {
                id: format!("t08-{team}-m2"),
                role: MemoryRole::Assistant,
                content: format!("acknowledged T8TOK-{token}"),
            },
        ],
    }
}

fn recall_request(team: &str) -> MemoryRecallRequest {
    MemoryRecallRequest {
        team_id: team.into(),
        agent_id: "bugrail-t08-agent".into(),
        user_id: "t08-user".into(),
        query: "captured fact for later recall".into(),
        limit: 5,
        include_core: false,
    }
}

#[tokio::test]
#[ignore = "requires the pinned v2.0.0+bugrail.1 fixture (see tests/results memory evidence)"]
async fn t08_pinned_gateway_health_capture_idempotency_isolation_and_restart_recall() {
    let Some(fx) = fixture() else {
        panic!("BUGRAIL_T08_URL is not set; start the pinned fixture first");
    };
    let memory = service().await;

    // Phase 1 — health: exact pinned version, writable.
    let (resolved, adapter) = memory
        .adapter_for(&provider_config(&fx, "team-alpha"), None)
        .expect("resolve pinned provider");
    let health = adapter.health().await.expect("health");
    assert!(matches!(
        health.status,
        codeg_lib::memory::MemoryHealthStatus::Healthy
    ));
    assert_eq!(
        health.version.as_deref(),
        Some(codeg_lib::memory::EXPECTED_UPSTREAM_VERSION)
    );
    assert!(health.writable, "pinned version must gate capture writable");

    // Phase 2 — capture under team-alpha, then replay the identical batch.
    let alpha = batch("team-alpha", "alpha");
    let receipt = adapter.capture(&alpha).await.expect("capture");
    assert_eq!(
        receipt.accepted_ids,
        vec![
            "t08-team-alpha-m1".to_string(),
            "t08-team-alpha-m2".to_string()
        ]
    );
    let replay = adapter.capture(&alpha).await.expect("replay capture");
    assert_eq!(
        replay.accepted_ids, receipt.accepted_ids,
        "replay must return the identical accepted id set"
    );

    // L0 row count is stable across replay (patch upsert contract).
    let count = conversation_count(&fx, "team-alpha").await;
    assert_eq!(count, 2, "replay must not add L0 rows");

    // Phase 3 — identity isolation: a second team captures its own token.
    let beta = batch("team-beta", "beta");
    adapter.capture(&beta).await.expect("capture beta");
    assert_eq!(conversation_count(&fx, "team-beta").await, 2);

    // Phase 4 — restart BugRail: a fresh service over the same fixture
    // recalls L1 (poll every 2s, at most 120s) with matching provenance.
    let memory2 = service().await;
    let (_resolved2, adapter2) = memory2
        .adapter_for(&provider_config(&fx, "team-alpha"), None)
        .expect("resolve after restart");
    let mut alpha_hits = Vec::new();
    for _ in 0..60 {
        let result = adapter2
            .recall(&recall_request("team-alpha"), resolved.timeout)
            .await
            .expect("recall");
        if !result.l1.is_empty() {
            alpha_hits = result.l1;
            break;
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    assert!(
        !alpha_hits.is_empty(),
        "L1 extraction did not land within 120s"
    );
    for hit in &alpha_hits {
        assert_eq!(hit.layer, MemoryLayer::L1);
        assert!(!hit.remote_id.is_empty());
        assert!(
            hit.content.contains("memory-of:alpha"),
            "alpha recall must carry the alpha token, got: {}",
            hit.content
        );
    }

    let (_rb, adapter_b) = memory2
        .adapter_for(&provider_config(&fx, "team-beta"), None)
        .expect("resolve beta after restart");
    let mut beta_hits = Vec::new();
    for _ in 0..60 {
        let result = adapter_b
            .recall(&recall_request("team-beta"), resolved.timeout)
            .await
            .expect("recall beta");
        if !result.l1.is_empty() {
            beta_hits = result.l1;
            break;
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    assert!(!beta_hits.is_empty(), "beta L1 missing");
    for hit in &beta_hits {
        assert!(
            hit.content.contains("memory-of:beta"),
            "beta recall must carry the beta token, got: {}",
            hit.content
        );
    }
    // Isolation: no cross-team token ever appears.
    for hit in &alpha_hits {
        assert!(!hit.content.contains("memory-of:beta"));
    }
    for hit in &beta_hits {
        assert!(!hit.content.contains("memory-of:alpha"));
    }
}

/// `/v3/conversation/count` for one team. Bounded, auth'd, body discarded
/// except the count — used only as the no-duplicate-L0 oracle.
async fn conversation_count(fx: &Fixture, team: &str) -> u64 {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(5))
        .build()
        .expect("client");
    let response = client
        .post(format!("{}/v3/conversation/count", fx.url))
        .bearer_auth(&fx.secret)
        .header("x-tdai-service-id", &fx.service_id)
        .json(&serde_json::json!({
            "team_id": team,
            "agent_id": "bugrail-t08-agent",
            "user_id": fx.user_id,
        }))
        .send()
        .await
        .expect("count request");
    assert!(response.status().is_success(), "count endpoint failed");
    let value: serde_json::Value = response.json().await.expect("count json");
    value
        .get("data")
        .and_then(|d| d.get("total").or_else(|| d.get("count")))
        .and_then(|c| c.as_u64())
        .or_else(|| value.get("count").and_then(|c| c.as_u64()))
        .unwrap_or(0)
}
