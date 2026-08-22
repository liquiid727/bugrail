//! Real-binary integration test for the Code Intelligence module.
//!
//! Gated behind `#[ignore]` and an explicit env var so default / CI runs
//! stay hermetic. To run it, place the pinned codebase-memory-mcp v0.10.6
//! binary somewhere and point the test at it:
//!
//! ```sh
//! CODEG_CBM_TEST_BIN=/tmp/cbm-probe/codebase-memory-mcp \
//!   cargo test --features test-utils --test code_intelligence_real -- \
//!   --ignored --nocapture
//! ```
//!
//! Covers: env-override binary resolution, base-repo indexing, bound
//! read-only queries, Context Pack summary shape, worktree isolation
//! (sibling worktree gets its own index, never resolves to the base),
//! task-worktree index drop, and write-side tool unreachability.
#![cfg(feature = "test-utils")]

use std::path::Path;

use codeg_lib::code_intelligence::{self as ci, manifest, CodeQuery};
use serde_json::json;

fn test_binary() -> Option<String> {
    std::env::var("CODEG_CBM_TEST_BIN")
        .ok()
        .filter(|p| !p.trim().is_empty())
}

fn run_git(dir: &Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to spawn git");
    assert!(
        out.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
}

fn seed_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, content).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires CODEG_CBM_TEST_BIN pointing at the pinned codebase-memory-mcp v0.10.6 binary"]
async fn real_adapter_full_lifecycle() {
    let Some(binary) = test_binary() else {
        panic!("set CODEG_CBM_TEST_BIN to the pinned codebase-memory-mcp binary to run this test");
    };

    // Env override is the first resolution tier; set it before any resolve.
    // (2021 edition: plain set_var is fine.)
    std::env::set_var(manifest::BINARY_OVERRIDE_ENV, &binary);

    // Sandboxed data dir: registry + binary cache + upstream store all live
    // under this temp root and disappear with it.
    let data_dir = tempfile::tempdir().expect("data dir");
    let rt = ci::init_runtime(data_dir.path()).expect("runtime init");

    // ── binary resolution: env override wins, reports the pinned version ──
    let resolved = rt
        .resolve_binary()
        .await
        .expect("resolve_binary")
        .expect("override must resolve");
    assert_eq!(
        resolved.path,
        Path::new(&binary),
        "env override must resolve to the exact path"
    );
    assert!(
        matches!(resolved.source, ci::BinarySource::EnvOverride),
        "expected EnvOverride, got {:?}",
        resolved.source
    );
    assert!(
        resolved.version.contains("0.10.6"),
        "expected the pinned 0.10.6 line, got {:?}",
        resolved.version
    );

    // ── fixture repository ────────────────────────────────────────────────
    let fixture = tempfile::tempdir().expect("fixture dir");
    let repo = fixture.path().join("repo");
    seed_file(
        &repo.join("src/main.rs"),
        "fn main() { println!(\"hello index\"); }\n\nfn helper() -> i32 { 7 }\n",
    );
    seed_file(&repo.join("README.md"), "# probe repo\n");
    run_git(&repo, &["init", "-q"]);
    run_git(&repo, &["add", "."]);
    run_git(
        &repo,
        &[
            "-c",
            "user.email=probe@example.com",
            "-c",
            "user.name=Probe",
            "commit",
            "-qm",
            "seed",
        ],
    );

    // ── enable (index) the base repo ──────────────────────────────────────
    let record = rt
        .enable_project(&repo, false, None)
        .await
        .expect("enable_project must index the fixture repo");
    assert!(record.enabled);
    assert!(!record.worktree);
    assert_eq!(record.task_id, None);
    let canonical = ci::canonicalize_dir(&repo).expect("canonicalize");
    assert_eq!(record.project, ci::derive_project_key(&canonical));
    assert_eq!(record.repo_path, canonical);

    // Enabling must record the root in the shared store's allow-list — the
    // upstream daemon only indexes recorded roots once one exists.
    let allowed_roots_file = ci::store_dir(rt.root()).join("allowed_roots");
    let allowed = std::fs::read_to_string(&allowed_roots_file).expect("allowed_roots file");
    assert!(
        allowed.lines().any(|line| line.trim() == canonical),
        "base repo root must be recorded, got: {allowed:?}"
    );

    // Status is bound and enabled for anything inside the repo.
    let status = rt.status(&repo).await.expect("status");
    assert!(status.bound, "status must be bound after enable");
    assert!(status.enabled);
    assert_eq!(status.project.as_deref(), Some(record.project.as_str()));

    // ── read-only queries bind to the base repo key ───────────────────────
    let search = rt
        .query(&repo, CodeQuery::Search, json!({ "query": "helper" }))
        .await
        .expect("search");
    assert!(!search.is_error, "search errored: {}", search.text);
    assert_eq!(search.project, record.project);

    let text = rt
        .query(
            &repo,
            CodeQuery::TextSearch,
            json!({ "pattern": "hello index" }),
        )
        .await
        .expect("text search");
    assert!(!text.is_error, "text search errored: {}", text.text);
    assert_eq!(text.project, record.project);

    // Querying from a subdirectory still binds to the same index.
    let sub = rt
        .query(&repo.join("src"), CodeQuery::Architecture, json!({}))
        .await
        .expect("architecture from subdir");
    assert_eq!(sub.project, record.project);

    // ── Context Pack summary for non-MCP agents ───────────────────────────
    let summary = rt
        .context_summary(&repo)
        .await
        .expect("context_summary")
        .expect("bound repo must yield a summary");
    assert_eq!(summary["schema"], "bugrail.code-intelligence.summary");
    assert_eq!(summary["project"], record.project);
    assert_eq!(summary["adapter"], manifest::ADAPTER_ID);

    // ── worktree isolation ────────────────────────────────────────────────
    // Worktrees are SIBLINGS of the base repo in Bugrail, so the ancestor
    // fallback must never resolve a worktree cwd to the base index.
    let wt = fixture.path().join("repo-task-42");
    seed_file(
        &wt.join("src/main.rs"),
        "fn main() { println!(\"worktree copy\"); }\n",
    );
    run_git(&wt, &["init", "-q"]);
    run_git(&wt, &["add", "."]);
    run_git(
        &wt,
        &[
            "-c",
            "user.email=probe@example.com",
            "-c",
            "user.name=Probe",
            "commit",
            "-qm",
            "worktree seed",
        ],
    );

    let wt_record = rt
        .enable_project(&wt, true, Some(42))
        .await
        .expect("worktree enable_project");
    assert!(wt_record.worktree);
    assert_eq!(wt_record.task_id, Some(42));
    assert_ne!(
        wt_record.project, record.project,
        "worktree must have its own project key"
    );

    // The worktree root is recorded too — required because the daemon was
    // already running for the base repo when this second root got enabled.
    let wt_canonical = ci::canonicalize_dir(&wt).expect("canonicalize worktree");
    let allowed = std::fs::read_to_string(&allowed_roots_file).expect("allowed_roots file");
    assert!(
        allowed.lines().any(|line| line.trim() == wt_canonical),
        "worktree root must be recorded alongside the base root, got: {allowed:?}"
    );

    // Queries from inside the worktree bind to the WORKTREE key, never the
    // base repo key.
    let wt_query = rt
        .query(
            &wt.join("src"),
            CodeQuery::TextSearch,
            json!({ "pattern": "worktree copy" }),
        )
        .await
        .expect("worktree query");
    assert_eq!(wt_query.project, wt_record.project);
    assert_ne!(wt_query.project, record.project);

    // Both records are visible in the registry.
    let all = rt.registry().all();
    assert!(all
        .iter()
        .any(|r| r.project == record.project && !r.worktree));
    assert!(all
        .iter()
        .any(|r| r.project == wt_record.project && r.worktree));

    // ── task worktree cleanup drops the temporary index ───────────────────
    let dropped = rt.drop_task_worktree_indexes(42).await;
    assert_eq!(dropped, 1, "exactly one worktree index for task 42");

    // The worktree root leaves the allow-list; the base root stays.
    let allowed = std::fs::read_to_string(&allowed_roots_file).expect("allowed_roots file");
    assert!(
        !allowed.lines().any(|line| line.trim() == wt_canonical),
        "worktree root must be un-recorded after drop, got: {allowed:?}"
    );
    assert!(
        allowed.lines().any(|line| line.trim() == canonical),
        "base repo root must survive the worktree drop, got: {allowed:?}"
    );

    // The worktree no longer resolves (siblings → no ancestor fallback).
    let err = rt
        .query(&wt, CodeQuery::TextSearch, json!({ "pattern": "x" }))
        .await;
    assert!(
        matches!(err, Err(ci::CodeIntelError::NotFound(_))),
        "worktree query must be unbound after drop, got {err:?}"
    );

    // The base repo index is untouched.
    let still = rt
        .query(
            &repo,
            CodeQuery::TextSearch,
            json!({ "pattern": "hello index" }),
        )
        .await
        .expect("base repo must stay queryable");
    assert_eq!(still.project, record.project);

    // ── write-side tools are never agent-addressable ──────────────────────
    for forbidden in [
        "index_repository",
        "delete_project",
        "manage_adr",
        "ingest_traces",
    ] {
        assert!(
            CodeQuery::from_tool_name(forbidden).is_none(),
            "{forbidden} must never map onto the read-only CodeQuery enum"
        );
    }

    rt.shutdown().await;
}
