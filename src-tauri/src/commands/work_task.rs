//! Work-task CRUD + engine-dispatched commands. The `*_core` fns are
//! mode-agnostic and shared by the Tauri wrappers and the Axum handlers.
//! Anything that launches, cancels, merges, or touches a worktree routes
//! through the process-global task engine (per-folder git mutex + run_seq
//! generations live there); a process that does not hold the engine lock gets
//! a clean "engine not running" error.

use crate::app_error::AppCommandError;
use crate::commands::folders::{get_folder_core, git_diff_with_branch};
use crate::db::entities::work_task::WorkTaskStatus;
use crate::db::entities::{work_task_contract, work_task_gate_result};
use crate::db::error::DbError;
use crate::db::service::work_task_service;
use crate::db::AppDatabase;
use crate::models::{
    AcceptanceCriterionSnapshot, FollowUpIntent, WorkTaskChangedFile, WorkTaskContractDraft,
    WorkTaskContractInfo, WorkTaskContractPreview, WorkTaskDraft, WorkTaskEventInfo,
    WorkTaskFolderSettings, WorkTaskGateDecision, WorkTaskGatePolicy, WorkTaskGateResultInfo,
    WorkTaskGateStatus, WorkTaskGateType, WorkTaskInfo, WorkTaskTemplateDraft,
    WorkTaskTemplateInfo,
};
use crate::web::event_bridge::{
    emit_event, EventEmitter, WorkTaskChange, WORK_TASK_CHANGED_EVENT,
};
use crate::work_task::{gate_decision, spec_reader};
use sea_orm::ActiveValue::{NotSet, Set};
use sea_orm::TransactionTrait;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

fn engine() -> Result<std::sync::Arc<crate::work_task::TaskEngine>, DbError> {
    crate::work_task::engine()
        .ok_or_else(|| DbError::Validation("task engine not running".to_string()))
}

/// Best-effort pump nudge so an auto_process folder reacts to creates, edits,
/// requeues and settings changes without waiting for the reconcile tick. A
/// process not holding the engine lock skips it — the owning process's tick
/// picks the change up from the DB.
fn nudge_pump(folder_id: i32) {
    if let Some(engine) = crate::work_task::engine() {
        tokio::spawn(async move { engine.pump_folder(folder_id).await });
    }
}

/// Best-effort sweep of planned starts, for the one case the 15s tick handles
/// visibly late: a time that is already in the past when it is set.
fn nudge_schedule() {
    if let Some(engine) = crate::work_task::engine() {
        tokio::spawn(async move { engine.claim_due_scheduled().await });
    }
}

/// Best-effort auto-merge nudge after a settings change: switching auto-merge
/// on should drain the review backlog now, not at the next reconcile tick.
/// Scope 0 is the global row, which any folder without its own row follows —
/// that one sweeps every folder holding reviewed tasks.
fn nudge_auto_merge(folder_id: i32) {
    if let Some(engine) = crate::work_task::engine() {
        let scope = (folder_id != 0).then_some(folder_id);
        tokio::spawn(async move { engine.sweep_auto_merge_backlog(scope).await });
    }
}

// ── shared business logic (both modes) ──────────────────────────────────────

pub async fn work_task_list_core(
    db: &AppDatabase,
    folder_id: Option<i32>,
) -> Result<Vec<WorkTaskInfo>, DbError> {
    let mut infos = work_task_service::list(&db.conn, folder_id).await?;
    annotate_worktree_missing(db, &mut infos).await?;
    Ok(infos)
}

pub async fn work_task_get_core(db: &AppDatabase, id: i32) -> Result<WorkTaskInfo, DbError> {
    let mut infos = vec![work_task_service::get(&db.conn, id).await?];
    annotate_worktree_missing(db, &mut infos).await?;
    Ok(infos.pop().expect("annotated the one row"))
}

/// Stamp `worktree_missing` on every row whose recorded worktree can no longer
/// serve a merge: its folder row was removed, or its directory is gone from
/// disk. One batched folder query plus a stat per distinct worktree — cheap
/// enough for every list, and the board needs it live: a reviewed task whose
/// worktree vanished must offer "complete" instead of a merge that can only
/// fail.
async fn annotate_worktree_missing(
    db: &AppDatabase,
    infos: &mut [WorkTaskInfo],
) -> Result<(), DbError> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    let ids: std::collections::BTreeSet<i32> =
        infos.iter().filter_map(|t| t.worktree_folder_id).collect();
    if ids.is_empty() {
        return Ok(());
    }
    let live_paths: std::collections::HashMap<i32, String> =
        crate::db::entities::folder::Entity::find()
            .filter(crate::db::entities::folder::Column::Id.is_in(ids.iter().copied()))
            .filter(crate::db::entities::folder::Column::DeletedAt.is_null())
            .all(&db.conn)
            .await?
            .into_iter()
            .map(|f| (f.id, f.path))
            .collect();
    let on_disk: std::collections::HashMap<i32, bool> = ids
        .iter()
        .map(|id| {
            let present = live_paths
                .get(id)
                .is_some_and(|path| std::path::Path::new(path).exists());
            (*id, present)
        })
        .collect();
    for info in infos.iter_mut() {
        if let Some(wt_id) = info.worktree_folder_id {
            info.worktree_missing = !on_disk.get(&wt_id).copied().unwrap_or(false);
        }
    }
    Ok(())
}

pub async fn work_task_events_core(
    db: &AppDatabase,
    task_id: i32,
    limit: u64,
) -> Result<Vec<WorkTaskEventInfo>, DbError> {
    work_task_service::list_events(&db.conn, task_id, limit).await
}

pub async fn work_task_attention_count_core(db: &AppDatabase) -> Result<u64, DbError> {
    work_task_service::attention_count(&db.conn).await
}

pub async fn work_task_create_core(
    emitter: &EventEmitter,
    db: &AppDatabase,
    draft: WorkTaskDraft,
) -> Result<WorkTaskInfo, DbError> {
    let info = work_task_service::create(&db.conn, draft).await?;
    emit_event(
        emitter,
        WORK_TASK_CHANGED_EVENT,
        WorkTaskChange::Upsert { id: info.id },
    );
    nudge_pump(info.folder_id);
    Ok(info)
}

pub async fn work_task_update_core(
    emitter: &EventEmitter,
    db: &AppDatabase,
    id: i32,
    draft: WorkTaskDraft,
) -> Result<WorkTaskInfo, DbError> {
    let info = work_task_service::update(&db.conn, id, draft).await?;
    emit_event(
        emitter,
        WORK_TASK_CHANGED_EVENT,
        WorkTaskChange::Upsert { id },
    );
    nudge_pump(info.folder_id);
    Ok(info)
}

/// How many times a delete re-reads before giving up. A retry only happens when
/// something claimed the task mid-delete; more than a couple in a row means the
/// board is fighting the user, and an error is a better answer than a loop.
const DELETE_ATTEMPTS: usize = 4;

/// Delete a task. An active run is canceled first; `delete_worktree` also
/// removes its worktree (best-effort — a cleanup failure does not block the
/// delete, the worktree just stays on disk). Refused while merging.
///
/// The whole thing runs as converge-then-tombstone rather than
/// decide-once-then-write: three arms can claim a `todo` task out from under
/// this call (the user, the folder's auto-processor, a planned start coming
/// due), and a tombstone written over a generation that just started would
/// leave its freshly minted worktree — and possibly its agent process — behind,
/// with the row that knows about them gone. So the final `soft_delete` is
/// guarded on the status we validated, and losing that guard sends us round
/// again to cancel whatever claimed it.
pub async fn work_task_delete_core(
    emitter: &EventEmitter,
    db: &AppDatabase,
    id: i32,
    delete_worktree: bool,
) -> Result<(), DbError> {
    // Kept only to report the reason if we run out of attempts.
    let mut last_conflict: Option<String> = None;
    for _ in 0..DELETE_ATTEMPTS {
        let task = work_task_service::get_model(&db.conn, id).await?;
        if task.status == WorkTaskStatus::Merging {
            return Err(DbError::Validation(
                "task is merging — wait for it to finish".to_string(),
            ));
        }
        if matches!(
            task.status,
            WorkTaskStatus::Queued
                | WorkTaskStatus::Preparing
                | WorkTaskStatus::Running
                | WorkTaskStatus::AwaitingInput
        ) {
            // `cancel` waits on the engine's per-task lock, which a launch holds
            // across its whole setup — so when it returns, that generation has
            // stopped touching the worktree. A cancel that loses its own CAS
            // just means the task settled by itself; re-read and decide again
            // instead of failing the delete.
            if let Err(e) = engine()?.cancel(id, None).await {
                last_conflict = Some(e);
            }
            continue;
        }
        // Read from THIS pass, not from a stale first look: a run that started
        // and was cancelled above has a worktree the first snapshot never saw.
        if delete_worktree && task.worktree_folder_id.is_some() {
            if let Err(e) = engine()?.cleanup_task(id).await {
                tracing::warn!("[work_task] cleanup during delete of task {id}: {e}");
            }
        }
        if work_task_service::soft_delete(&db.conn, id, task.status).await? {
            emit_event(
                emitter,
                WORK_TASK_CHANGED_EVENT,
                WorkTaskChange::Deleted { id },
            );
            return Ok(());
        }
        last_conflict = Some("task was claimed while being deleted".to_string());
    }
    Err(DbError::Validation(last_conflict.unwrap_or_else(|| {
        "task kept changing while being deleted — try again".to_string()
    })))
}

/// Persist the pending column's drag order. `sort_order` also drives the
/// engine's claim/launch order, so reordering queued tasks re-prioritizes them.
pub async fn work_task_reorder_core(
    emitter: &EventEmitter,
    db: &AppDatabase,
    folder_id: i32,
    ordered_ids: Vec<i32>,
) -> Result<(), DbError> {
    work_task_service::reorder(&db.conn, folder_id, &ordered_ids).await?;
    emit_event(emitter, WORK_TASK_CHANGED_EVENT, WorkTaskChange::Refresh);
    nudge_pump(folder_id);
    Ok(())
}

pub async fn work_task_start_core(id: i32) -> Result<(), DbError> {
    engine()?.start(id).await.map_err(DbError::Validation)
}

/// `folder_id: None` = the global sweep — every folder holding todos.
pub async fn work_task_start_all_core(folder_id: Option<i32>) -> Result<u32, DbError> {
    engine()?
        .start_all(folder_id)
        .await
        .map_err(DbError::Validation)
}

/// failed → queued, optionally with a note explaining what to do differently.
/// `blocks` carries whatever the note box attached out of band (images, pasted
/// bytes) as raw prompt blocks.
pub async fn work_task_retry_core(
    id: i32,
    note: Option<String>,
    blocks: Vec<serde_json::Value>,
) -> Result<(), DbError> {
    engine()?
        .retry(id, note, blocks)
        .await
        .map_err(DbError::Validation)
}

/// canceled → todo. Pure DB (no engine needed) — the user starts it again
/// explicitly. A cancel usually had a reason; the optional note carries it into
/// the next run's prompt.
pub async fn work_task_requeue_core(
    emitter: &EventEmitter,
    db: &AppDatabase,
    id: i32,
    note: Option<String>,
    blocks: Vec<serde_json::Value>,
) -> Result<(), DbError> {
    if !work_task_service::requeue_canceled(&db.conn, id, note.as_deref(), &blocks).await? {
        return Err(DbError::Validation("task is not canceled".to_string()));
    }
    emit_event(
        emitter,
        WORK_TASK_CHANGED_EVENT,
        WorkTaskChange::Upsert { id },
    );
    if let Ok(task) = work_task_service::get_model(&db.conn, id).await {
        nudge_pump(task.folder_id);
    }
    Ok(())
}

/// Plan when a to-do task starts. `scheduled_at` is an RFC 3339 instant (the
/// client sends the time the user picked, converted from its own zone);
/// `None` clears the plan. Pure DB — the engine's schedule tick claims the task
/// when its time comes, and the nudge below covers a time already in the past.
pub async fn work_task_schedule_core(
    emitter: &EventEmitter,
    db: &AppDatabase,
    id: i32,
    scheduled_at: Option<String>,
) -> Result<(), DbError> {
    let at = match scheduled_at.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(raw) => Some(
            chrono::DateTime::parse_from_rfc3339(raw)
                .map_err(|e| DbError::Validation(format!("invalid scheduled_at: {e}")))?
                .with_timezone(&chrono::Utc),
        ),
        None => None,
    };
    if !work_task_service::set_schedule(&db.conn, id, at).await? {
        return Err(DbError::Validation(
            "only to-do tasks can be scheduled".to_string(),
        ));
    }
    emit_event(
        emitter,
        WORK_TASK_CHANGED_EVENT,
        WorkTaskChange::Upsert { id },
    );
    if at.is_some() {
        nudge_schedule();
    }
    Ok(())
}

/// Follow up on a reviewed task. `intent` picks the wording the agent receives;
/// absent means `revise`, the historical "returned with feedback" behaviour.
pub async fn work_task_return_core(
    id: i32,
    feedback: String,
    intent: Option<String>,
    blocks: Vec<serde_json::Value>,
) -> Result<(), DbError> {
    let intent = FollowUpIntent::from_wire(intent.as_deref()).map_err(DbError::Validation)?;
    let feedback = feedback.trim().to_string();
    // A self-check is a complete instruction on its own, and so is an attached
    // screenshot; everything else is only as good as what the user typed.
    if feedback.is_empty() && blocks.is_empty() && !intent.allows_empty() {
        return Err(DbError::Validation("feedback is required".to_string()));
    }
    engine()?
        .return_task(id, intent, feedback, blocks)
        .await
        .map_err(DbError::Validation)
}

/// Stop a task. `reason` is the user's optional note about WHY — it lands on
/// the `canceled` entry of the progress timeline and nowhere else.
pub async fn work_task_cancel_core(id: i32, reason: Option<String>) -> Result<(), DbError> {
    engine()?
        .cancel(id, reason)
        .await
        .map_err(DbError::Validation)
}

/// Dispatch the merge generation: the agent lands the task in its session and
/// the outcome rides the `task://changed` events (merging → done, or back to
/// review with a readable error). This awaits only the dispatch (validation +
/// agent spawn), so refused merges surface directly in the dialog.
/// `message: None` = the agent writes the commit message itself.
///
/// A contract-bound task must first be gate-eligible (Spec not stale, every
/// required gate passed or validly waived). A blocked merge keeps `review` and
/// records a `quality_gate_blocked` timeline event.
pub async fn work_task_merge_core(
    emitter: &EventEmitter,
    db: &AppDatabase,
    id: i32,
    message: Option<String>,
    delete_worktree: bool,
) -> Result<(), AppCommandError> {
    enforce_gate_eligibility(emitter, db, id).await?;
    engine()?
        .merge_task(id, message, delete_worktree, false)
        .await
        .map_err(|e| AppCommandError::from(DbError::Validation(e)))?;
    Ok(())
}

/// Finish a reviewed task that has nothing to land (review → done, no merge),
/// optionally removing its worktree. Refused when the worktree turns out to
/// hold changes after all — that task belongs on the merge path. Same
/// gate-eligibility precondition as merge; a blocked completion stays `review`.
pub async fn work_task_complete_core(
    emitter: &EventEmitter,
    db: &AppDatabase,
    id: i32,
    delete_worktree: bool,
) -> Result<(), AppCommandError> {
    enforce_gate_eligibility(emitter, db, id).await?;
    engine()?
        .complete_task(id, delete_worktree)
        .await
        .map_err(|e| AppCommandError::from(DbError::Validation(e)))?;
    Ok(())
}

const QUALITY_GATE_UNMET_I18N: &str = "workTask.qualityGate.unmet";
const QUALITY_GATE_INVALID_WAIVER_I18N: &str = "workTask.qualityGate.invalidWaiver";

/// Current gate decision for a contract-bound task (`None` for legacy tasks,
/// which nothing gates). Computed fresh from persisted facts + the current Spec
/// file hash + the worktree HEAD — never from a cached or client-supplied
/// result (Feature Spec §5.3, AC05).
async fn task_gate_decision(
    db: &AppDatabase,
    task_id: i32,
) -> Result<Option<WorkTaskGateDecision>, AppCommandError> {
    if work_task_service::get_contract(&db.conn, task_id)
        .await
        .map_err(AppCommandError::from)?
        .is_none()
    {
        return Ok(None);
    }
    let stale = work_task_spec_staleness_core(db, task_id).await?;
    let current_head = task_worktree_head(db, task_id).await?;
    work_task_service::gate_decision(&db.conn, task_id, current_head.as_deref(), stale)
        .await
        .map(Some)
        .map_err(AppCommandError::from)
}

/// The task worktree's current `HEAD`, when it has one. Used to validate a
/// reusable preflight result (`verified_head` must still match).
async fn task_worktree_head(
    db: &AppDatabase,
    task_id: i32,
) -> Result<Option<String>, AppCommandError> {
    let task = work_task_service::get_model(&db.conn, task_id)
        .await
        .map_err(AppCommandError::from)?;
    let Some(wt_id) = task.worktree_folder_id else {
        return Ok(None);
    };
    let wt = get_folder_core(db, wt_id).await.map_err(AppCommandError::from)?;
    match crate::work_task::git::rev_parse(&wt.path, "HEAD").await {
        Ok(sha) => Ok(Some(sha)),
        Err(_) => Ok(None),
    }
}

/// Reject merge/complete for a contract-bound, reviewed task whose gate
/// decision is not eligible. Records the block so it is auditable. A task not
/// in `review` is left to the engine's own status error (no gate block event
/// for a task that could not have merged anyway).
async fn enforce_gate_eligibility(
    emitter: &EventEmitter,
    db: &AppDatabase,
    task_id: i32,
) -> Result<(), AppCommandError> {
    let task = work_task_service::get_model(&db.conn, task_id)
        .await
        .map_err(AppCommandError::from)?;
    if task.status != WorkTaskStatus::Review {
        return Ok(());
    }
    let Some(decision) = task_gate_decision(db, task_id).await? else {
        return Ok(());
    };
    if decision.eligible {
        return Ok(());
    }
    record_gate_blocked_event(db, task_id, &decision).await;
    emit_event(emitter, WORK_TASK_CHANGED_EVENT, WorkTaskChange::Upsert { id: task_id });
    Err(gate_block_error(task_id, &decision))
}

/// Typed merge/complete block: a stale bound Spec reads as
/// `workTask.specContract.stale`; otherwise `workTask.qualityGate.unmet` with
/// the unmet gate IDs (Feature Spec §4.3). Both are `TaskExecutionFailed` and
/// leave the task untouched in `review`.
fn gate_block_error(task_id: i32, decision: &WorkTaskGateDecision) -> AppCommandError {
    if decision.stale_spec {
        return AppCommandError::task_execution_failed(format!(
            "task {task_id}'s bound spec changed since it was bound — re-preview and rebind"
        ))
        .with_i18n(spec_reader::SPEC_CONTRACT_STALE_I18N, {
            let mut m = BTreeMap::new();
            m.insert("taskId".to_string(), task_id.to_string());
            m
        });
    }
    let gates = decision
        .unmet
        .iter()
        .map(|g| g.gate_id.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    AppCommandError::task_execution_failed(format!(
        "task {task_id} has unmet quality gates: {gates}"
    ))
    .with_i18n(QUALITY_GATE_UNMET_I18N, {
        let mut m = BTreeMap::new();
        m.insert("taskId".to_string(), task_id.to_string());
        m.insert("gates".to_string(), gates);
        m
    })
}

async fn record_gate_blocked_event(
    db: &AppDatabase,
    task_id: i32,
    decision: &WorkTaskGateDecision,
) {
    let payload = serde_json::json!({
        "eligible": decision.eligible,
        "stale_spec": decision.stale_spec,
        "unmet": decision.unmet.iter().map(|g| serde_json::json!({
            "gate_id": g.gate_id,
            "status": g.status.map(|s| s.as_str()),
            "reason": g.reason,
        })).collect::<Vec<_>>(),
    });
    let _ = work_task_service::record_event(&db.conn, task_id, "quality_gate_blocked", "user", Some(payload))
        .await;
}

/// List persisted gate attempts for a task, optionally scoped to one run.
pub async fn work_task_gate_list_core(
    db: &AppDatabase,
    task_id: i32,
    run_seq: Option<i32>,
) -> Result<Vec<WorkTaskGateResultInfo>, DbError> {
    let rows = work_task_service::list_gate_results(&db.conn, task_id, run_seq).await?;
    rows.into_iter()
        .map(work_task_service::gate_result_info)
        .collect()
}

/// The current explainable gate decision. Legacy (unbound) tasks are always
/// eligible.
pub async fn work_task_gate_decision_core(
    db: &AppDatabase,
    task_id: i32,
) -> Result<WorkTaskGateDecision, AppCommandError> {
    Ok(task_gate_decision(db, task_id)
        .await?
        .unwrap_or(WorkTaskGateDecision {
            eligible: true,
            stale_spec: false,
            required: vec![],
            unmet: vec![],
            waived: vec![],
        }))
}

/// Record a trusted-user gate decision (`approve` → `passed`, `waive` →
/// `waived`). No generic client-controlled gate-record path exists: the gate
/// must be in the task's snapshotted policy, `approve` applies only to
/// `human_approval` gates, `waive` only when `allow_waiver` is true, and actor
/// is always derived from the authenticated command context (never request
/// JSON). A reason is required, and a waiver requires a non-empty one.
pub async fn work_task_gate_human_decide_core(
    emitter: &EventEmitter,
    db: &AppDatabase,
    task_id: i32,
    gate_id: String,
    decision: String,
    reason: Option<String>,
) -> Result<WorkTaskGateResultInfo, AppCommandError> {
    let contract = work_task_service::get_contract(&db.conn, task_id)
        .await
        .map_err(AppCommandError::from)?
        .ok_or_else(|| {
            AppCommandError::invalid_input(format!("task {task_id} has no spec contract"))
        })?;
    let policy: WorkTaskGatePolicy = serde_json::from_str(&contract.gate_policy).map_err(|e| {
        AppCommandError::invalid_input(format!("stored gate policy invalid: {e}"))
    })?;
    let gate = policy
        .gates
        .iter()
        .find(|g| g.id == gate_id)
        .ok_or_else(|| {
            AppCommandError::invalid_input(format!("gate {gate_id} is not in the task's policy"))
        })?;

    let reason = reason.map(|r| r.trim().to_string()).filter(|r| !r.is_empty());
    let (status, is_waiver) = match decision.as_str() {
        "approve" => {
            if gate.gate_type != WorkTaskGateType::HumanApproval {
                return Err(AppCommandError::invalid_input(format!(
                    "gate {gate_id} is engine-owned preflight; only a waiver can be recorded here"
                )));
            }
            (WorkTaskGateStatus::Passed, false)
        }
        "waive" => {
            if !gate.allow_waiver {
                return Err(AppCommandError::permission_denied(format!(
                    "gate {gate_id} does not allow waiver"
                ))
                .with_i18n(QUALITY_GATE_INVALID_WAIVER_I18N, BTreeMap::new()));
            }
            (WorkTaskGateStatus::Waived, true)
        }
        other => {
            return Err(AppCommandError::invalid_input(format!(
                "unknown gate decision: {other}"
            )))
        }
    };
    let Some(reason) = reason else {
        if is_waiver {
            return Err(AppCommandError::permission_denied(
                "a waiver requires a non-empty reason",
            )
            .with_i18n(QUALITY_GATE_INVALID_WAIVER_I18N, BTreeMap::new()));
        }
        return Err(AppCommandError::invalid_input(
            "a gate decision requires a non-empty reason",
        ));
    };

    let task = work_task_service::get_model(&db.conn, task_id)
        .await
        .map_err(AppCommandError::from)?;
    let now = chrono::Utc::now();
    let txn = db
        .conn
        .begin()
        .await
        .map_err(|e| AppCommandError::from(DbError::from(e)))?;
    let model = work_task_gate_result::ActiveModel {
        id: NotSet,
        task_id: Set(task_id),
        run_seq: Set(task.run_seq),
        gate_id: Set(gate_id.clone()),
        gate_type: Set(gate.gate_type.as_str().to_string()),
        status: Set(status.as_str().to_string()),
        required: Set(gate.required),
        reusable: Set(gate.reusable),
        actor: Set("user".to_string()),
        evidence: Set(None),
        reason: Set(Some(reason)),
        started_at: Set(now),
        finished_at: Set(Some(now)),
    };
    let row = work_task_service::insert_gate_result(&txn, model)
        .await
        .map_err(AppCommandError::from)?;
    work_task_service::record_event(
        &txn,
        task_id,
        "gate_result",
        "user",
        Some(serde_json::json!({
            "gate_id": gate_id,
            "gate_type": gate.gate_type.as_str(),
            "status": status.as_str(),
            "decision": decision,
        })),
    )
    .await
    .map_err(AppCommandError::from)?;
    txn.commit()
        .await
        .map_err(|e| AppCommandError::from(DbError::from(e)))?;

    emit_event(emitter, WORK_TASK_CHANGED_EVENT, WorkTaskChange::Upsert { id: task_id });
    work_task_service::gate_result_info(row).map_err(AppCommandError::from)
}

/// Archive / unarchive a terminal task (pure DB; no engine needed). Archived
/// tasks leave the default board view and the attention badge.
pub async fn work_task_archive_core(
    emitter: &EventEmitter,
    db: &AppDatabase,
    id: i32,
    archived: bool,
) -> Result<(), DbError> {
    if !work_task_service::set_archived(&db.conn, id, archived).await? {
        return Err(DbError::Validation(if archived {
            "only finished tasks can be archived".to_string()
        } else {
            "task is not archived".to_string()
        }));
    }
    emit_event(
        emitter,
        WORK_TASK_CHANGED_EVENT,
        WorkTaskChange::Upsert { id },
    );
    Ok(())
}

pub async fn work_task_cleanup_core(id: i32) -> Result<(), DbError> {
    engine()?.cleanup_task(id).await.map_err(DbError::Validation)
}

/// Diff of the task worktree vs. its recorded base (`base_sha`, so the view is
/// stable even when the base branch advances). `file = None` → full patch.
pub async fn work_task_diff_core(
    db: &AppDatabase,
    id: i32,
    file: Option<String>,
) -> Result<String, AppCommandError> {
    let task = work_task_service::get_model(&db.conn, id)
        .await
        .map_err(AppCommandError::from)?;
    let wt_id = task
        .worktree_folder_id
        .ok_or_else(|| AppCommandError::not_found("task has no worktree"))?;
    let base = task
        .base_sha
        .clone()
        .or(task.base_branch.clone())
        .ok_or_else(|| AppCommandError::not_found("task has no recorded base"))?;
    let wt = get_folder_core(db, wt_id)
        .await
        .map_err(AppCommandError::from)?;
    git_diff_with_branch(wt.path, base, file).await
}

pub async fn work_task_changed_files_core(
    db: &AppDatabase,
    id: i32,
) -> Result<Vec<WorkTaskChangedFile>, AppCommandError> {
    let task = work_task_service::get_model(&db.conn, id)
        .await
        .map_err(AppCommandError::from)?;
    let Some(wt_id) = task.worktree_folder_id else {
        return Ok(vec![]);
    };
    let Some(base) = task.base_sha.clone().or(task.base_branch.clone()) else {
        return Ok(vec![]);
    };
    let wt = get_folder_core(db, wt_id)
        .await
        .map_err(AppCommandError::from)?;
    crate::work_task::git::diff_numstat(&wt.path, &base).await
}

// ── SpecOS contract commands (BUGRAIL-SPECOS-001 issue-002) ─────────────────

fn spec_invalid(message: impl Into<String>) -> AppCommandError {
    AppCommandError::invalid_input(message)
        .with_i18n(spec_reader::SPEC_CONTRACT_INVALID_I18N, BTreeMap::new())
}

/// Current Spec file differs from the preview hash / bound hash (stale).
fn spec_stale(message: impl Into<String>) -> AppCommandError {
    AppCommandError::task_execution_failed(message)
        .with_i18n(spec_reader::SPEC_CONTRACT_STALE_I18N, BTreeMap::new())
}

/// The live project folder a task runs in — the root every repository-relative
/// Spec path resolves against (Feature Spec §5.1 step 1).
async fn task_project_root(db: &AppDatabase, task_id: i32) -> Result<PathBuf, AppCommandError> {
    let task = work_task_service::get_model(&db.conn, task_id)
        .await
        .map_err(AppCommandError::from)?;
    let folder = get_folder_core(db, task.folder_id)
        .await
        .map_err(AppCommandError::from)?;
    Ok(PathBuf::from(folder.path))
}

/// Parse a repository-local Feature Spec and return its exact identity, hash,
/// and available AC, plus the task's current binding hash (rebind context).
/// Read-only: never mutates the task or its contract.
pub async fn work_task_contract_preview_core(
    db: &AppDatabase,
    task_id: i32,
    source_spec_path: String,
) -> Result<WorkTaskContractPreview, AppCommandError> {
    let project_root = task_project_root(db, task_id).await?;
    let spec = spec_reader::read_spec_reference(&project_root, &source_spec_path)?;
    let current_binding_hash = work_task_service::get_contract(&db.conn, task_id)
        .await
        .map_err(AppCommandError::from)?
        .map(|m| m.source_spec_hash);
    Ok(WorkTaskContractPreview {
        source_spec_id: spec.id,
        source_spec_version: spec.version,
        source_spec_path: spec.path,
        source_spec_hash: spec.sha256,
        acceptance_criteria: spec.acceptance_criteria,
        current_binding_hash,
    })
}

/// Bind (or explicitly rebind) a task to the repository-local Feature Spec in
/// the draft. Validates the canonical path, file size, identity, hash (the
/// preview's optimistic-concurrency token), selected AC, and gate policy before
/// any write; then upserts the contract and records a `spec_contract_bound`
/// timeline event in ONE transaction. A rebind preserves old gate attempts and
/// records old/new hashes in the timeline (Feature Spec §5.1).
pub async fn work_task_contract_bind_core(
    emitter: &EventEmitter,
    db: &AppDatabase,
    task_id: i32,
    draft: WorkTaskContractDraft,
) -> Result<WorkTaskContractInfo, AppCommandError> {
    let task = work_task_service::get_model(&db.conn, task_id)
        .await
        .map_err(AppCommandError::from)?;
    let existing = work_task_service::get_contract(&db.conn, task_id)
        .await
        .map_err(AppCommandError::from)?;

    // An explicit rebind may only happen while no execution generation is in
    // flight: todo / review / failed / canceled. Everything else would let one
    // generation change its acceptance contract mid-flight (Feature Spec §5.1).
    if existing.is_some()
        && !matches!(
            task.status,
            WorkTaskStatus::Todo
                | WorkTaskStatus::Review
                | WorkTaskStatus::Failed
                | WorkTaskStatus::Canceled
        )
    {
        return Err(AppCommandError::task_execution_failed(format!(
            "task {task_id} cannot rebind in state {}",
            work_task_service::status_str(task.status)
        )));
    }

    // Revalidate the file and hash. The hash is the optimistic-concurrency
    // token from the preview: if the file changed since preview, bind is
    // rejected and no implicit rebind happens (T07).
    let project_root = task_project_root(db, task_id).await?;
    let spec = spec_reader::read_spec_reference(&project_root, &draft.source_spec_path)?;
    if spec.sha256 != draft.expected_source_spec_hash {
        return Err(spec_stale(
            "spec changed since preview; re-preview and retry",
        ));
    }

    // Resolve selected AC identifiers server-side; the client never submits AC
    // text (T08).
    let by_id: HashMap<&str, &AcceptanceCriterionSnapshot> = spec
        .acceptance_criteria
        .iter()
        .map(|ac| (ac.id.as_str(), ac))
        .collect();
    let mut selected = Vec::with_capacity(draft.selected_acceptance_criteria_ids.len());
    for id in &draft.selected_acceptance_criteria_ids {
        let ac = by_id.get(id.as_str()).ok_or_else(|| {
            spec_invalid(format!("unknown acceptance criterion: {id}"))
        })?;
        selected.push((*ac).clone());
    }

    // Gate policy + snapshot limits (T09). `WorkTaskGateType` is exhaustive for
    // the first slice, so an unsupported type fails to deserialize.
    gate_decision::validate_gate_policy(&draft.gate_policy).map_err(spec_invalid)?;
    gate_decision::validate_acceptance_criteria_snapshot(&selected).map_err(spec_invalid)?;

    let now = chrono::Utc::now();
    let txn = db
        .conn
        .begin()
        .await
        .map_err(|e| AppCommandError::from(DbError::from(e)))?;
    let model = work_task_contract::ActiveModel {
        task_id: Set(task_id),
        source_spec_id: Set(spec.id.clone()),
        source_spec_version: Set(spec.version.clone()),
        source_spec_path: Set(spec.path.clone()),
        source_spec_hash: Set(spec.sha256.clone()),
        acceptance_criteria: Set(serde_json::to_string(&selected).map_err(|e| {
            AppCommandError::invalid_input("serialize acceptance criteria").with_detail(e.to_string())
        })?),
        gate_policy: Set(serde_json::to_string(&draft.gate_policy).map_err(|e| {
            AppCommandError::invalid_input("serialize gate policy").with_detail(e.to_string())
        })?),
        // A rebind keeps the original creation time; only the reference moves.
        created_at: Set(existing.as_ref().map(|m| m.created_at).unwrap_or(now)),
        updated_at: Set(now),
    };
    work_task_service::upsert_contract(&txn, model)
        .await
        .map_err(AppCommandError::from)?;

    let mut payload = serde_json::json!({
        "source_spec_id": spec.id,
        "source_spec_version": spec.version,
        "source_spec_hash": spec.sha256,
    });
    if let Some(old) = existing.as_ref() {
        payload["rebind"] = serde_json::json!(true);
        payload["previous_source_spec_hash"] = serde_json::json!(old.source_spec_hash);
    }
    work_task_service::record_event(&txn, task_id, "spec_contract_bound", "user", Some(payload))
        .await
        .map_err(AppCommandError::from)?;
    txn.commit()
        .await
        .map_err(|e| AppCommandError::from(DbError::from(e)))?;

    emit_event(
        emitter,
        WORK_TASK_CHANGED_EVENT,
        WorkTaskChange::Upsert { id: task_id },
    );

    work_task_service::contract_get(&db.conn, task_id)
        .await
        .map_err(AppCommandError::from)?
        .ok_or_else(|| AppCommandError::not_found(format!("contract for task {task_id}")))
}

/// Read a task's stored contract (`None` for legacy/unbound tasks).
pub async fn work_task_contract_get_core(
    db: &AppDatabase,
    task_id: i32,
) -> Result<Option<WorkTaskContractInfo>, DbError> {
    work_task_service::contract_get(&db.conn, task_id).await
}

/// Whether the bound Spec file still hashes to the bound hash. Internal helper
/// feeding the merge/complete decision (issue-003); not a public command.
pub async fn work_task_spec_staleness_core(
    db: &AppDatabase,
    task_id: i32,
) -> Result<bool, AppCommandError> {
    let Some(contract) = work_task_service::get_contract(&db.conn, task_id)
        .await
        .map_err(AppCommandError::from)?
    else {
        return Ok(false);
    };
    let project_root = task_project_root(db, task_id).await?;
    spec_reader::spec_stale(&project_root, &contract.source_spec_path, &contract.source_spec_hash)
}

pub async fn work_task_settings_get_core(
    db: &AppDatabase,
    folder_id: i32,
) -> Result<WorkTaskFolderSettings, DbError> {
    work_task_service::settings_get(&db.conn, folder_id).await
}

/// Effective settings after the folder → global → built-in fallback — what
/// the engine will actually use for this folder (editor prefill, merge dialog
/// seeding).
pub async fn work_task_settings_effective_core(
    db: &AppDatabase,
    folder_id: i32,
) -> Result<WorkTaskFolderSettings, DbError> {
    work_task_service::settings_get_effective(&db.conn, folder_id).await
}

/// The folder's own settings row, or `None` when it follows the global
/// defaults — the settings dialog's source-of-truth probe.
pub async fn work_task_settings_get_own_core(
    db: &AppDatabase,
    folder_id: i32,
) -> Result<Option<WorkTaskFolderSettings>, DbError> {
    work_task_service::settings_get_own(&db.conn, folder_id).await
}

pub async fn work_task_settings_set_core(
    emitter: &EventEmitter,
    db: &AppDatabase,
    folder_id: i32,
    settings: WorkTaskFolderSettings,
) -> Result<(), DbError> {
    work_task_service::settings_set(&db.conn, folder_id, &settings).await?;
    emit_event(
        emitter,
        WORK_TASK_CHANGED_EVENT,
        WorkTaskChange::Settings { folder_id },
    );
    nudge_pump(folder_id);
    nudge_auto_merge(folder_id);
    Ok(())
}

/// Remove the folder's own settings row — it reverts to the global defaults.
/// Same nudges as a set: auto-process/concurrency may effectively change.
pub async fn work_task_settings_delete_core(
    emitter: &EventEmitter,
    db: &AppDatabase,
    folder_id: i32,
) -> Result<(), DbError> {
    work_task_service::settings_delete(&db.conn, folder_id).await?;
    emit_event(
        emitter,
        WORK_TASK_CHANGED_EVENT,
        WorkTaskChange::Settings { folder_id },
    );
    nudge_pump(folder_id);
    // Reverting to the global row can also switch auto-merge ON for this
    // folder (the global row may carry it) — same drain-now semantics.
    nudge_auto_merge(folder_id);
    Ok(())
}

pub async fn work_task_template_list_core(
    db: &AppDatabase,
) -> Result<Vec<WorkTaskTemplateInfo>, DbError> {
    work_task_service::template_list(&db.conn).await
}

pub async fn work_task_template_save_core(
    db: &AppDatabase,
    draft: WorkTaskTemplateDraft,
) -> Result<WorkTaskTemplateInfo, DbError> {
    work_task_service::template_save(&db.conn, &draft).await
}

pub async fn work_task_template_delete_core(db: &AppDatabase, id: i32) -> Result<(), DbError> {
    work_task_service::template_delete(&db.conn, id).await
}

// ── Tauri command wrappers (desktop only) ───────────────────────────────────

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_list(
    db: tauri::State<'_, AppDatabase>,
    folder_id: Option<i32>,
) -> Result<Vec<WorkTaskInfo>, DbError> {
    work_task_list_core(&db, folder_id).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_get(
    db: tauri::State<'_, AppDatabase>,
    id: i32,
) -> Result<WorkTaskInfo, DbError> {
    work_task_get_core(&db, id).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_events(
    db: tauri::State<'_, AppDatabase>,
    task_id: i32,
    limit: Option<u64>,
) -> Result<Vec<WorkTaskEventInfo>, DbError> {
    work_task_events_core(&db, task_id, limit.unwrap_or(500)).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_attention_count(
    db: tauri::State<'_, AppDatabase>,
) -> Result<u64, DbError> {
    work_task_attention_count_core(&db).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_create(
    app: tauri::AppHandle,
    db: tauri::State<'_, AppDatabase>,
    draft: WorkTaskDraft,
) -> Result<WorkTaskInfo, DbError> {
    work_task_create_core(&EventEmitter::Tauri(app), &db, draft).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_update(
    app: tauri::AppHandle,
    db: tauri::State<'_, AppDatabase>,
    id: i32,
    draft: WorkTaskDraft,
) -> Result<WorkTaskInfo, DbError> {
    work_task_update_core(&EventEmitter::Tauri(app), &db, id, draft).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_reorder(
    app: tauri::AppHandle,
    db: tauri::State<'_, AppDatabase>,
    folder_id: i32,
    ordered_ids: Vec<i32>,
) -> Result<(), DbError> {
    work_task_reorder_core(&EventEmitter::Tauri(app), &db, folder_id, ordered_ids).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_delete(
    app: tauri::AppHandle,
    db: tauri::State<'_, AppDatabase>,
    id: i32,
    delete_worktree: Option<bool>,
) -> Result<(), DbError> {
    work_task_delete_core(
        &EventEmitter::Tauri(app),
        &db,
        id,
        delete_worktree.unwrap_or(false),
    )
    .await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_start(id: i32) -> Result<(), DbError> {
    work_task_start_core(id).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_start_all(folder_id: Option<i32>) -> Result<u32, DbError> {
    work_task_start_all_core(folder_id).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_retry(
    id: i32,
    note: Option<String>,
    blocks: Option<Vec<serde_json::Value>>,
) -> Result<(), DbError> {
    work_task_retry_core(id, note, blocks.unwrap_or_default()).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_requeue(
    app: tauri::AppHandle,
    db: tauri::State<'_, AppDatabase>,
    id: i32,
    note: Option<String>,
    blocks: Option<Vec<serde_json::Value>>,
) -> Result<(), DbError> {
    work_task_requeue_core(
        &EventEmitter::Tauri(app),
        &db,
        id,
        note,
        blocks.unwrap_or_default(),
    )
    .await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_schedule(
    app: tauri::AppHandle,
    db: tauri::State<'_, AppDatabase>,
    id: i32,
    scheduled_at: Option<String>,
) -> Result<(), DbError> {
    work_task_schedule_core(&EventEmitter::Tauri(app), &db, id, scheduled_at).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_return(
    id: i32,
    feedback: String,
    intent: Option<String>,
    blocks: Option<Vec<serde_json::Value>>,
) -> Result<(), DbError> {
    work_task_return_core(id, feedback, intent, blocks.unwrap_or_default()).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_cancel(id: i32, reason: Option<String>) -> Result<(), DbError> {
    work_task_cancel_core(id, reason).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_merge(
    app: tauri::AppHandle,
    db: tauri::State<'_, AppDatabase>,
    id: i32,
    message: Option<String>,
    delete_worktree: bool,
) -> Result<(), AppCommandError> {
    work_task_merge_core(&EventEmitter::Tauri(app), &db, id, message, delete_worktree).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_complete(
    app: tauri::AppHandle,
    db: tauri::State<'_, AppDatabase>,
    id: i32,
    delete_worktree: bool,
) -> Result<(), AppCommandError> {
    work_task_complete_core(&EventEmitter::Tauri(app), &db, id, delete_worktree).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_gate_list(
    db: tauri::State<'_, AppDatabase>,
    task_id: i32,
    run_seq: Option<i32>,
) -> Result<Vec<WorkTaskGateResultInfo>, DbError> {
    work_task_gate_list_core(&db, task_id, run_seq).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_gate_decision(
    db: tauri::State<'_, AppDatabase>,
    task_id: i32,
) -> Result<WorkTaskGateDecision, AppCommandError> {
    work_task_gate_decision_core(&db, task_id).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_gate_human_decide(
    app: tauri::AppHandle,
    db: tauri::State<'_, AppDatabase>,
    task_id: i32,
    gate_id: String,
    decision: String,
    reason: Option<String>,
) -> Result<WorkTaskGateResultInfo, AppCommandError> {
    work_task_gate_human_decide_core(
        &EventEmitter::Tauri(app),
        &db,
        task_id,
        gate_id,
        decision,
        reason,
    )
    .await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_archive(
    app: tauri::AppHandle,
    db: tauri::State<'_, AppDatabase>,
    id: i32,
    archived: bool,
) -> Result<(), DbError> {
    work_task_archive_core(&EventEmitter::Tauri(app), &db, id, archived).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_cleanup(id: i32) -> Result<(), DbError> {
    work_task_cleanup_core(id).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_diff(
    db: tauri::State<'_, AppDatabase>,
    id: i32,
    file: Option<String>,
) -> Result<String, AppCommandError> {
    work_task_diff_core(&db, id, file).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_changed_files(
    db: tauri::State<'_, AppDatabase>,
    id: i32,
) -> Result<Vec<WorkTaskChangedFile>, AppCommandError> {
    work_task_changed_files_core(&db, id).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_settings_get(
    db: tauri::State<'_, AppDatabase>,
    folder_id: i32,
) -> Result<WorkTaskFolderSettings, DbError> {
    work_task_settings_get_core(&db, folder_id).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_settings_effective(
    db: tauri::State<'_, AppDatabase>,
    folder_id: i32,
) -> Result<WorkTaskFolderSettings, DbError> {
    work_task_settings_effective_core(&db, folder_id).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_settings_get_own(
    db: tauri::State<'_, AppDatabase>,
    folder_id: i32,
) -> Result<Option<WorkTaskFolderSettings>, DbError> {
    work_task_settings_get_own_core(&db, folder_id).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_settings_set(
    app: tauri::AppHandle,
    db: tauri::State<'_, AppDatabase>,
    folder_id: i32,
    settings: WorkTaskFolderSettings,
) -> Result<(), DbError> {
    work_task_settings_set_core(&EventEmitter::Tauri(app), &db, folder_id, settings).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_settings_delete(
    app: tauri::AppHandle,
    db: tauri::State<'_, AppDatabase>,
    folder_id: i32,
) -> Result<(), DbError> {
    work_task_settings_delete_core(&EventEmitter::Tauri(app), &db, folder_id).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_template_list(
    db: tauri::State<'_, AppDatabase>,
) -> Result<Vec<WorkTaskTemplateInfo>, DbError> {
    work_task_template_list_core(&db).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_template_save(
    db: tauri::State<'_, AppDatabase>,
    draft: WorkTaskTemplateDraft,
) -> Result<WorkTaskTemplateInfo, DbError> {
    work_task_template_save_core(&db, draft).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_template_delete(
    db: tauri::State<'_, AppDatabase>,
    id: i32,
) -> Result<(), DbError> {
    work_task_template_delete_core(&db, id).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_contract_preview(
    db: tauri::State<'_, AppDatabase>,
    task_id: i32,
    source_spec_path: String,
) -> Result<WorkTaskContractPreview, AppCommandError> {
    work_task_contract_preview_core(&db, task_id, source_spec_path).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_contract_bind(
    app: tauri::AppHandle,
    db: tauri::State<'_, AppDatabase>,
    task_id: i32,
    draft: WorkTaskContractDraft,
) -> Result<WorkTaskContractInfo, AppCommandError> {
    work_task_contract_bind_core(&EventEmitter::Tauri(app), &db, task_id, draft).await
}

#[cfg(feature = "tauri-runtime")]
#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub async fn work_task_contract_get(
    db: tauri::State<'_, AppDatabase>,
    task_id: i32,
) -> Result<Option<WorkTaskContractInfo>, DbError> {
    work_task_contract_get_core(&db, task_id).await
}
