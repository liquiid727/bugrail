//! Durable provider-job oracles for BUGRAIL-SPECOS-028.T03.
//!
//! SQLite facts, not refresh-event delivery, are the source of truth. The
//! tests cover idempotency, exclusive leases, crash recovery and bounded
//! retry history without involving a WorkTask or Memory capture delivery.

use chrono::{Duration, TimeZone, Utc};
use codeg_lib::db::entities::provider_job;
use codeg_lib::db::migration::Migrator;
use codeg_lib::db::service::provider_job_service::{
    self, ProviderJobError, ProviderJobSpec, ATTEMPT_FAILED, ATTEMPT_INTERRUPTED,
    ATTEMPT_SUCCEEDED, STATUS_FAILED, STATUS_QUEUED, STATUS_SUCCEEDED,
};
use codeg_lib::db::test_helpers::{fresh_disk_db, fresh_in_memory_db};
use sea_orm::{ConnectOptions, Database, EntityTrait, PaginatorTrait};
use sea_orm_migration::MigratorTrait;
use sha2::Digest;
use std::sync::Arc;
use tokio::sync::Barrier;

fn at(seconds: i64) -> chrono::DateTime<Utc> {
    Utc.timestamp_opt(1_750_000_000 + seconds, 0)
        .single()
        .expect("valid fixture timestamp")
}

fn spec(key: &str, hash: &str, max_attempts: i32) -> ProviderJobSpec {
    ProviderJobSpec {
        provider_kind: "wiki".into(),
        provider_id: "wiki-main".into(),
        operation: "sync".into(),
        idempotency_key: key.into(),
        request_hash: if hash.len() == 64 {
            hash.into()
        } else {
            format!("{:x}", sha2::Sha256::digest(hash.as_bytes()))
        },
        max_attempts,
    }
}

#[tokio::test]
async fn t03_duplicate_submission_is_one_idempotency_fact() {
    let db = fresh_in_memory_db().await;
    let first = provider_job_service::submit(&db.conn, spec("same-key", "hash-a", 3), at(0))
        .await
        .expect("first submission");
    let duplicate = provider_job_service::submit(&db.conn, spec("same-key", "hash-a", 3), at(1))
        .await
        .expect("duplicate is idempotent");
    assert_eq!(first.id, duplicate.id);
    assert_eq!(duplicate.created_at, at(0));

    let conflict =
        provider_job_service::submit(&db.conn, spec("same-key", "hash-b", 3), at(2)).await;
    assert!(matches!(
        conflict,
        Err(ProviderJobError::IdempotencyConflict)
    ));
    let count = provider_job::Entity::find().count(&db.conn).await.unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn t03_crash_recovery_reclaims_lease_without_parallel_claim() {
    let db = fresh_in_memory_db().await;
    let row = provider_job_service::submit(&db.conn, spec("crash-key", "hash-a", 3), at(0))
        .await
        .unwrap();
    let first = provider_job_service::claim_due(&db.conn, at(0), Duration::seconds(30), 1)
        .await
        .unwrap();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].job.id, row.id);
    assert!(
        provider_job_service::claim_due(&db.conn, at(0), Duration::seconds(30), 1)
            .await
            .unwrap()
            .is_empty()
    );

    assert_eq!(
        provider_job_service::recover_expired(&db.conn, at(31))
            .await
            .unwrap(),
        1
    );
    let history = provider_job_service::attempt_history(&db.conn, row.id, 99)
        .await
        .unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].status, ATTEMPT_INTERRUPTED);
    assert!(matches!(
        provider_job_service::mark_succeeded(&db.conn, row.id, &first[0].lease_token, at(31)).await,
        Err(ProviderJobError::LeaseLost)
    ));

    let second = provider_job_service::claim_due(&db.conn, at(31), Duration::seconds(30), 1)
        .await
        .unwrap();
    assert_eq!(second.len(), 1);
    assert_eq!(second[0].job.attempt_count, 2);
    provider_job_service::mark_succeeded(&db.conn, row.id, &second[0].lease_token, at(32))
        .await
        .unwrap();
    let settled = provider_job_service::get(&db.conn, row.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(settled.status, STATUS_SUCCEEDED);
    assert_eq!(
        provider_job_service::attempt_history(&db.conn, row.id, 99)
            .await
            .unwrap()
            .iter()
            .map(|attempt| attempt.status.as_str())
            .collect::<Vec<_>>(),
        vec![ATTEMPT_INTERRUPTED, ATTEMPT_SUCCEEDED]
    );
}

#[tokio::test]
async fn t03_retry_exhaustion_is_bounded_and_redacts_diagnostics() {
    let db = fresh_in_memory_db().await;
    let row = provider_job_service::submit(&db.conn, spec("retry-key", "hash-a", 2), at(0))
        .await
        .unwrap();

    let first = provider_job_service::claim_due(&db.conn, at(0), Duration::seconds(30), 1)
        .await
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(
        provider_job_service::mark_failed(
            &db.conn,
            row.id,
            &first.lease_token,
            "rate_limited",
            "temporary provider outage",
            true,
            at(0),
        )
        .await
        .unwrap(),
        STATUS_QUEUED
    );
    let second = provider_job_service::claim_due(&db.conn, at(2), Duration::seconds(30), 1)
        .await
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(
        provider_job_service::mark_failed(
            &db.conn,
            row.id,
            &second.lease_token,
            "authorization",
            "Authorization: bearer secret-value",
            true,
            at(2),
        )
        .await
        .unwrap(),
        STATUS_FAILED
    );
    assert!(
        provider_job_service::claim_due(&db.conn, at(100), Duration::seconds(30), 1)
            .await
            .unwrap()
            .is_empty()
    );
    let settled = provider_job_service::get(&db.conn, row.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(settled.status, STATUS_FAILED);
    assert_eq!(settled.attempt_count, 2);
    assert_eq!(
        settled.last_error_message.as_deref(),
        Some("provider operation failed")
    );
    assert_eq!(settled.last_error_code.as_deref(), Some("provider_error"));
    let history = provider_job_service::attempt_history(&db.conn, row.id, 99)
        .await
        .unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[1].status, ATTEMPT_FAILED);
}

#[tokio::test]
async fn t03_restart_reconstructs_facts_from_disk() {
    let temp = tempfile::tempdir().unwrap();
    let db = fresh_disk_db(temp.path()).await;
    let row = provider_job_service::submit(&db.conn, spec("restart-key", "hash-a", 3), at(0))
        .await
        .unwrap();
    provider_job_service::claim_due(&db.conn, at(0), Duration::seconds(10), 1)
        .await
        .unwrap();
    db.conn.close().await.unwrap();

    let reopened = fresh_disk_db(temp.path()).await;
    assert_eq!(
        provider_job_service::recover_expired(&reopened.conn, at(11))
            .await
            .unwrap(),
        1
    );
    let recovered = provider_job_service::get(&reopened.conn, row.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(recovered.status, STATUS_QUEUED);
    assert_eq!(recovered.attempt_count, 1);
    assert!(recovered.lease_token_hash.is_none());
    assert_eq!(
        provider_job_service::attempt_history(&reopened.conn, row.id, 99)
            .await
            .unwrap()[0]
            .status,
        ATTEMPT_INTERRUPTED
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn t03_concurrent_workers_retry_sqlite_busy_without_dropping_jobs() {
    let temp = tempfile::tempdir().unwrap();
    let url = format!(
        "sqlite:{}?mode=rwc",
        temp.path().join("provider-jobs.db").display()
    );
    let mut migrate_options = ConnectOptions::new(url.clone());
    migrate_options.max_connections(1).min_connections(1);
    let migrate_conn = Database::connect(migrate_options).await.unwrap();
    Migrator::up(&migrate_conn, None).await.unwrap();
    migrate_conn.close().await.unwrap();

    let mut options = ConnectOptions::new(url);
    options.max_connections(5).min_connections(5);
    let conn = Database::connect(options).await.unwrap();
    let now = at(0);
    for index in 0..20 {
        provider_job_service::submit(&conn, spec(&format!("parallel-{index}"), "hash-a", 1), now)
            .await
            .unwrap();
    }

    let barrier = Arc::new(Barrier::new(21));
    let mut tasks = Vec::new();
    for _ in 0..20 {
        let worker_conn = conn.clone();
        let worker_barrier = barrier.clone();
        tasks.push(tokio::spawn(async move {
            worker_barrier.wait().await;
            provider_job_service::claim_due(&worker_conn, now, Duration::seconds(30), 1).await
        }));
    }
    barrier.wait().await;

    let mut claimed = 0;
    for task in tasks {
        claimed += task.await.unwrap().unwrap().len();
    }
    assert_eq!(claimed, 20);
}
