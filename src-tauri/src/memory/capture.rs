//! Capture policy: eligibility, filtering, caps, secret exclusion and outbox
//! staging (BUGRAIL-SPECOS-017 §5).
//!
//! Only non-empty `user` and `assistant` text of a settled `review`/`failed`
//! run generation is staged. System prompts, Context Packages, tool
//! inputs/results, terminal bytes and attachments never enter the payload; a
//! message matching a secret rule is excluded whole, before hashing or
//! enqueue. Messages are never truncated — anything over a cap is excluded
//! with an explicit reason count.
//!
//! Enqueue performs no network access and resolves no env references: the
//! staged payload carries the deterministic message ids, while
//! team/agent/user/session identity is resolved by the worker at delivery
//! time (fresh config wins for undelivered rows).

use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::db::entities::work_task_run;
use crate::db::service::{folder_service, memory_capture_service, work_task_service};
use crate::db::AppDatabase;
use crate::models::{ContextProviderConfig, TurnRole};

use super::config::{binding_for_folder, CAP_CAPTURE, MEMORY_KIND};
use super::identity;

/// Message-count cap (spec §5 default; byte caps are per-provider config).
pub const MAX_CAPTURE_MESSAGES: usize = 100;

/// Context Activity kind for capture enqueue/delivery evidence.
pub const ACTIVITY_KIND: &str = "memory.capture";

// ── Secret rules ────────────────────────────────────────────────────────────

/// A message matching ANY rule is excluded whole, before hashing or enqueue
/// (spec §5). Rules are deterministic pattern checks; counts are logged,
/// content never is.
static SECRET_RULES: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    let patterns = [
        // PEM private key material.
        r"-----BEGIN [A-Z0-9 ]*PRIVATE KEY-----",
        // AWS access key ids.
        r"\bAKIA[0-9A-Z]{16}\b",
        // GitHub tokens (classic + fine-grained).
        r"\bgh[pousr]_[A-Za-z0-9]{30,}\b",
        r"\bgithub_pat_[A-Za-z0-9_]{20,}\b",
        // Slack tokens.
        r"\bxox[baprs]-[A-Za-z0-9-]{10,}\b",
        // Vendor API keys (OpenAI/Anthropic-style).
        r"\bsk-[A-Za-z0-9_-]{20,}\b",
        // JWTs.
        r"\beyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b",
        // Bearer tokens in text.
        r"(?i)\bbearer\s+[A-Za-z0-9._~+/-]{16,}",
        // Secret-like assignments: `password = ...`, `api_key: ...`.
        r"(?i)\b(api[_-]?key|secret|access[_-]?token|auth[_-]?token|password|passwd|pwd|client[_-]?secret)\b\s*[:=]\s*[^\s,;]{6,}",
        // URLs carrying credentials.
        r"://[^/\s:@]+:[^/\s@]+@",
    ];
    patterns
        .iter()
        .map(|p| Regex::new(p).expect("static secret rule compiles"))
        .collect()
});

pub fn contains_secret(text: &str) -> bool {
    SECRET_RULES.iter().any(|rule| rule.is_match(text))
}

// ── Staged payload ──────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct StagedPayload {
    version: u32,
    messages: Vec<StagedMessage>,
}

#[derive(Serialize, Deserialize)]
struct StagedMessage {
    id: String,
    role: String,
    content: String,
}

/// Outcome of one run's filtering pass: the staged JSON, its integrity hash,
/// the stable message ids and bounded exclusion counts (reasons only — never
/// content).
pub struct StagedCapture {
    pub payload_json: String,
    pub payload_hash: String,
    pub source_message_ids: Vec<String>,
    pub included: usize,
    pub excluded_secret: usize,
    pub excluded_message_over_cap: usize,
    pub excluded_batch_over_cap: usize,
    pub excluded_count_over_cap: usize,
}

/// Filter one durable conversation into a staged capture payload.
///
/// `binding` is the project binding of the task's folder; message ids are
/// `message_id(binding, conversation_id, turn_id)` — deterministic, so
/// at-least-once retries upsert instead of duplicating L0 rows.
pub fn stage_capture(
    turns: &[crate::models::MessageTurn],
    binding: &str,
    conversation_id: i32,
    provider: &ContextProviderConfig,
) -> Option<StagedCapture> {
    // Eligibility inside the transcript: the FINAL assistant turn must carry
    // complete, non-empty text that itself passes the secret rules — a run
    // whose last word is missing or secret-bearing has nothing capturable.
    let final_assistant_ok = turns
        .iter()
        .rev()
        .find(|turn| turn.role == TurnRole::Assistant)
        .is_some_and(|turn| {
            let text = turn_text(turn);
            !text.trim().is_empty() && !contains_secret(&text)
        });
    if !final_assistant_ok {
        return None;
    }

    let mut candidates: Vec<(String, String, String)> = Vec::new(); // (id, role, content)
    let mut excluded_secret = 0usize;
    for turn in turns {
        let role = match turn.role {
            TurnRole::User => "user",
            TurnRole::Assistant => "assistant",
            // System turns are never captured.
            TurnRole::System => continue,
        };
        let text = turn_text(turn);
        let text = text.trim();
        if text.is_empty() {
            continue;
        }
        if contains_secret(text) {
            excluded_secret += 1;
            continue;
        }
        let id = identity::message_id(binding, conversation_id, &turn.id);
        candidates.push((id, role.to_string(), text.to_string()));
    }

    let max_message = provider.max_capture_message_bytes;
    let max_batch = provider.max_capture_batch_bytes;
    let mut messages = Vec::new();
    let mut batch_bytes = 0usize;
    let (mut over_message, mut over_batch, mut over_count) = (0usize, 0usize, 0usize);
    for (id, role, content) in candidates {
        if content.len() > max_message {
            over_message += 1;
            continue;
        }
        if messages.len() >= MAX_CAPTURE_MESSAGES {
            over_count += 1;
            continue;
        }
        if batch_bytes.saturating_add(content.len()) > max_batch {
            over_batch += 1;
            continue;
        }
        batch_bytes += content.len();
        messages.push(StagedMessage { id, role, content });
    }
    if messages.is_empty() {
        return None;
    }

    let source_message_ids: Vec<String> = messages.iter().map(|m| m.id.clone()).collect();
    let payload = StagedPayload {
        version: 1,
        messages,
    };
    // Deterministic field order via the derived Serialize impl — the hash is
    // an integrity check over exactly this staged string.
    let payload_json = serde_json::to_string(&payload).expect("staged capture payload serializes");
    let payload_hash = format!("{:x}", Sha256::digest(payload_json.as_bytes()));
    Some(StagedCapture {
        included: source_message_ids.len(),
        payload_json,
        payload_hash,
        source_message_ids,
        excluded_secret,
        excluded_message_over_cap: over_message,
        excluded_batch_over_cap: over_batch,
        excluded_count_over_cap: over_count,
    })
}

/// Concatenate the `Text` blocks of a turn. Tool use/results, images,
/// terminal bytes and attachments contribute nothing — only plain text is
/// capturable (spec §5).
fn turn_text(turn: &crate::models::MessageTurn) -> String {
    let mut out = String::new();
    for block in &turn.blocks {
        if let crate::models::ContentBlock::Text { text } = block {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(text);
        }
    }
    out
}

// ── Enqueue and reconciliation ──────────────────────────────────────────────

/// Providers of this folder's context config that may capture. Static config
/// only — no env resolution happens at enqueue time.
fn capture_providers(config: &crate::models::ContextConfig) -> Vec<&ContextProviderConfig> {
    config
        .providers
        .iter()
        .filter(|p| {
            p.kind == MEMORY_KIND
                && p.enabled
                && p.capture_enabled
                && p.capabilities.iter().any(|c| c == CAP_CAPTURE)
        })
        .collect()
}

/// Stage capture deliveries for one settled run generation (settle hook and
/// reconciliation share this path). Idempotent per
/// `(provider_id, task_id, run_seq)`; never touches the network, never
/// resolves env references, and never fails the settle — callers log the
/// result and move on.
pub async fn enqueue_for_run(db: &AppDatabase, task_id: i32, run_seq: i32) -> Result<u32, String> {
    // The run row is authoritative for the settled generation: outcome must
    // be review/failed with a durable conversation. Cancelled and merge-only
    // generations never qualify (merge runs have no conversation).
    let Some(run) = work_task_run::Entity::find_by_id((task_id, run_seq))
        .one(&db.conn)
        .await
        .map_err(|e| e.to_string())?
    else {
        return Ok(0);
    };
    if run.status != "review" && run.status != "failed" {
        return Ok(0);
    }
    let Some(conversation_id) = run.conversation_id else {
        return Ok(0);
    };

    let task = work_task_service::get_model(&db.conn, task_id)
        .await
        .map_err(|e| e.to_string())?;
    if task.deleted_at.is_some() {
        return Ok(0);
    }
    let folder_id = task.folder_id;
    let Some(folder) = folder_service::get_folder_by_id(&db.conn, folder_id)
        .await
        .map_err(|e| e.to_string())?
    else {
        return Ok(0);
    };

    let config =
        crate::specos_control::load_context(Path::new(&folder.path)).map_err(|e| e.to_string())?;
    let providers = capture_providers(&config);
    if providers.is_empty() {
        // Legacy projects: zero Memory behavior, zero rows (spec §9, AC08).
        return Ok(0);
    }

    // Skip the transcript parse entirely when every provider already owns a
    // row for this run generation (the common steady-state path).
    let mut missing = Vec::new();
    for provider in &providers {
        let exists = memory_capture_service::find_for_run(&db.conn, &provider.id, task_id, run_seq)
            .await
            .map_err(|e| e.to_string())?
            .is_some();
        if !exists {
            missing.push(*provider);
        }
    }
    if missing.is_empty() {
        return Ok(0);
    }

    let (detail, _) =
        crate::commands::conversations::get_folder_conversation_core(&db.conn, conversation_id)
            .await
            .map_err(|e| e.to_string())?;
    let binding = binding_for_folder(Path::new(&folder.path));
    let mut staged_total = 0u32;
    for provider in missing {
        let Some(staged) = stage_capture(&detail.turns, &binding, conversation_id, provider) else {
            continue;
        };
        let ids_json =
            serde_json::to_string(&staged.source_message_ids).unwrap_or_else(|_| "[]".to_string());
        memory_capture_service::enqueue(
            &db.conn,
            memory_capture_service::NewDelivery {
                provider_id: provider.id.clone(),
                folder_id,
                task_id,
                run_seq,
                conversation_id,
                payload: staged.payload_json,
                payload_hash: staged.payload_hash,
                source_message_ids: ids_json,
            },
        )
        .await
        .map_err(|e| e.to_string())?;
        staged_total += 1;
        let exclusions = staged.excluded_secret
            + staged.excluded_message_over_cap
            + staged.excluded_batch_over_cap
            + staged.excluded_count_over_cap;
        let message = format!(
            "staged {} messages for task {} run {} ({} excluded)",
            staged.included, task_id, run_seq, exclusions
        );
        // IDs and counts only — never payload content (spec §9).
        if let Err(e) = crate::context::record_activity(
            &db.conn,
            folder_id,
            None,
            Some(&provider.id),
            ACTIVITY_KIND,
            "queued",
            Some(&message),
        )
        .await
        {
            tracing::warn!(provider = %provider.id, "[memory] capture activity failed: {e}");
        }
        tracing::info!(
            provider = %provider.id,
            task_id,
            run_seq,
            included = staged.included,
            excluded_secret = staged.excluded_secret,
            excluded_over_cap = staged.excluded_message_over_cap
                + staged.excluded_batch_over_cap
                + staged.excluded_count_over_cap,
            "[memory] capture staged"
        );
    }
    Ok(staged_total)
}

/// Startup reconciliation: settled runs of the recent window missing a
/// delivery row get staged, covering the crash window after settlement
/// (spec §5). Bounded by recency and count; `enqueue_for_run` is idempotent.
pub async fn reconcile_missing_deliveries(db: &AppDatabase) -> Result<u32, String> {
    use sea_orm::{ColumnTrait, QueryFilter, QueryOrder, QuerySelect};
    let window = chrono::Utc::now() - chrono::Duration::days(7);
    let runs = work_task_run::Entity::find()
        .filter(work_task_run::Column::Status.is_in(["review", "failed"]))
        .filter(work_task_run::Column::ConversationId.is_not_null())
        .filter(work_task_run::Column::UpdatedAt.gte(window))
        .order_by_desc(work_task_run::Column::UpdatedAt)
        .limit(100)
        .all(&db.conn)
        .await
        .map_err(|e| e.to_string())?;
    let mut staged = 0u32;
    for run in runs {
        match enqueue_for_run(db, run.task_id, run.run_seq).await {
            Ok(count) => staged += count,
            Err(e) => {
                tracing::warn!(
                    task_id = run.task_id,
                    run_seq = run.run_seq,
                    "[memory] capture reconciliation skipped a run: {e}"
                );
            }
        }
    }
    if staged > 0 {
        tracing::info!(
            staged,
            "[memory] capture reconciliation staged missing deliveries"
        );
    }
    Ok(staged)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ContentBlock, MessageTurn};
    use chrono::Utc;

    fn turn(id: &str, role: TurnRole, text: &str) -> MessageTurn {
        MessageTurn {
            id: id.into(),
            role,
            blocks: vec![ContentBlock::Text { text: text.into() }],
            timestamp: Utc::now(),
            usage: None,
            duration_ms: None,
            model: None,
            completed_at: None,
        }
    }

    fn provider(message_cap: usize, batch_cap: usize) -> ContextProviderConfig {
        ContextProviderConfig {
            max_capture_message_bytes: message_cap,
            max_capture_batch_bytes: batch_cap,
            ..Default::default()
        }
    }

    #[test]
    fn stages_only_user_and_assistant_text() {
        let turns = vec![
            turn("t1", TurnRole::User, "please fix the login bug"),
            turn("t2", TurnRole::System, "you are an agent"),
            turn("t3", TurnRole::Assistant, "fixed it in auth.rs"),
        ];
        let staged = stage_capture(&turns, "binding", 7, &provider(8192, 262144)).unwrap();
        assert_eq!(staged.included, 2);
        let payload: serde_json::Value = serde_json::from_str(&staged.payload_json).unwrap();
        let messages = payload["messages"].as_array().unwrap();
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[1]["role"], "assistant");
        // Deterministic ids: same input → same staged ids and hash.
        let again = stage_capture(&turns, "binding", 7, &provider(8192, 262144)).unwrap();
        assert_eq!(again.payload_hash, staged.payload_hash);
        assert_eq!(again.source_message_ids, staged.source_message_ids);
    }

    #[test]
    fn secret_rule_excludes_the_whole_message_before_hashing() {
        let turns = vec![
            turn(
                "t1",
                TurnRole::User,
                "use this key: sk-liveabc123abc123abc123abc",
            ),
            turn("t2", TurnRole::Assistant, "done"),
        ];
        let staged = stage_capture(&turns, "binding", 7, &provider(8192, 262144)).unwrap();
        assert_eq!(staged.included, 1);
        assert_eq!(staged.excluded_secret, 1);
        assert!(!staged.payload_json.contains("sk-liveabc"));

        // A secret-bearing FINAL assistant turn makes the run ineligible.
        let turns = vec![
            turn("t1", TurnRole::User, "hello"),
            turn(
                "t2",
                TurnRole::Assistant,
                "sure, my password = hunter2secret",
            ),
        ];
        assert!(stage_capture(&turns, "binding", 7, &provider(8192, 262144)).is_none());
    }

    #[test]
    fn caps_exclude_with_reasons_and_never_truncate() {
        let big = "x".repeat(9000); // over the 8 KiB message cap
        let turns = vec![
            turn("t1", TurnRole::User, &big),
            turn("t2", TurnRole::Assistant, "small reply"),
        ];
        let staged = stage_capture(&turns, "binding", 7, &provider(8192, 262144)).unwrap();
        assert_eq!(staged.included, 1);
        assert_eq!(staged.excluded_message_over_cap, 1);
        assert!(!staged.payload_json.contains(&"x".repeat(100)));

        // Batch cap: two 200 KiB messages cannot share a 256 KiB batch.
        let half = "y".repeat(200 * 1024);
        let turns = vec![
            turn("t1", TurnRole::User, &half),
            turn("t2", TurnRole::Assistant, &half),
        ];
        let staged =
            stage_capture(&turns, "binding", 7, &provider(256 * 1024, 256 * 1024)).unwrap();
        assert_eq!(staged.included, 1);
        assert_eq!(staged.excluded_batch_over_cap, 1);
    }

    #[test]
    fn count_cap_limits_messages_to_one_hundred() {
        let mut turns = Vec::new();
        for i in 0..120 {
            let role = if i % 2 == 0 {
                TurnRole::User
            } else {
                TurnRole::Assistant
            };
            turns.push(turn(&format!("t{i}"), role, &format!("message {i}")));
        }
        // Final turn must be assistant with text: t119 is assistant. OK.
        let staged = stage_capture(&turns, "binding", 7, &provider(8192, 1024 * 1024)).unwrap();
        assert_eq!(staged.included, MAX_CAPTURE_MESSAGES);
        assert_eq!(staged.excluded_count_over_cap, 20);
    }

    #[test]
    fn empty_or_system_only_transcripts_stage_nothing() {
        assert!(stage_capture(&[], "binding", 7, &provider(8192, 262144)).is_none());
        let turns = vec![turn("t1", TurnRole::System, "system only")];
        assert!(stage_capture(&turns, "binding", 7, &provider(8192, 262144)).is_none());
        // Assistant turn with empty text → no complete final assistant text.
        let turns = vec![
            turn("t1", TurnRole::User, "hello"),
            turn("t2", TurnRole::Assistant, "   "),
        ];
        assert!(stage_capture(&turns, "binding", 7, &provider(8192, 262144)).is_none());
    }

    #[test]
    fn secret_patterns_cover_the_documented_shapes() {
        assert!(contains_secret("-----BEGIN RSA PRIVATE KEY-----"));
        assert!(contains_secret("key AKIAIOSFODNN7EXAMPLE here"));
        assert!(contains_secret(
            "token ghp_abcdefghijklmnopqrstuvwxyz0123456789"
        ));
        assert!(contains_secret("xoxb-1234567890-abcdef"));
        assert!(contains_secret("sk-proj-abcdefghijklmnopqrstuvwx"));
        assert!(contains_secret(
            "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.dBjftJeZ4CVP"
        ));
        assert!(contains_secret(
            "Authorization: Bearer abcdef1234567890abcd"
        ));
        assert!(contains_secret("password = hunter2secret"));
        assert!(contains_secret("https://user:pass@example.com/api"));
        assert!(!contains_secret("the password field is validated"));
        assert!(!contains_secret("fix the api key rotation docs"));
    }
}
