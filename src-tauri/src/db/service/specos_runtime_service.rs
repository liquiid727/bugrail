//! Persistence/projection helpers for Agent Team and Context runs.

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, PaginatorTrait, QueryFilter, QueryOrder, Set,
};
use std::collections::{BTreeMap, BTreeSet};

use crate::db::entities::{
    folder, team_run, team_run_task, work_task, work_task_dependency, work_task_handoff,
    work_task_run,
};
use crate::db::error::DbError;
use crate::models::{
    IntegrationPlan, IntegrationSnapshot, IntegrationSourceCapture, IntegrationSourceInfo,
    ResolvedAgentRuntime, TeamRunInfo, TeamRunNodeInfo, WorkTaskConfig, WorkTaskDependencyInfo,
    WorkTaskHandoffDraft, WorkTaskHandoffInfo, WorkTaskRunInfo,
};

pub async fn insert_run_snapshot<C: ConnectionTrait>(
    conn: &C,
    task: &work_task::Model,
    run_seq: i32,
    status: &str,
) -> Result<(), DbError> {
    let now = Utc::now();
    let cfg = serde_json::from_str::<WorkTaskConfig>(&task.config).unwrap_or_default();
    work_task_run::ActiveModel {
        task_id: Set(task.id),
        run_seq: Set(run_seq),
        status: Set(status.to_string()),
        agent_profile_id: Set(cfg.agent_profile_id),
        model_profile_id: Set(cfg.model_profile_id),
        agent_type: Set(cfg.agent_type),
        model: Set(cfg.config_values.get("model").cloned()),
        mode_id: Set(cfg.mode_id),
        reasoning: Set(cfg.config_values.get("reasoning_effort").cloned()),
        resolution: Set(None),
        conversation_id: Set(None),
        worktree_folder_id: Set(task.worktree_folder_id),
        context_package_id: Set(None),
        created_at: Set(now),
        started_at: Set(None),
        finished_at: Set(None),
        updated_at: Set(now),
    }
    .insert(conn)
    .await?;
    Ok(())
}

pub async fn update_run_state<C: ConnectionTrait>(
    conn: &C,
    task_id: i32,
    run_seq: i32,
    status: &str,
    conversation_id: Option<i32>,
    terminal: bool,
) -> Result<(), DbError> {
    let Some(row) = work_task_run::Entity::find_by_id((task_id, run_seq))
        .one(conn)
        .await?
    else {
        return Ok(());
    };
    let now = Utc::now();
    let mut active = row.into_active_model();
    active.status = Set(status.to_string());
    active.updated_at = Set(now);
    if status == "running" && active.started_at.as_ref().is_none() {
        active.started_at = Set(Some(now));
    }
    if let Some(id) = conversation_id {
        active.conversation_id = Set(Some(id));
    }
    if terminal {
        active.finished_at = Set(Some(now));
    }
    active.update(conn).await?;
    Ok(())
}

pub async fn update_run_resolution(
    conn: &DatabaseConnection,
    task_id: i32,
    run_seq: i32,
    runtime: &ResolvedAgentRuntime,
) -> Result<(), DbError> {
    let Some(row) = work_task_run::Entity::find_by_id((task_id, run_seq))
        .one(conn)
        .await?
    else {
        return Ok(());
    };
    let mut active = row.into_active_model();
    active.agent_profile_id = Set(runtime.agent_profile_id.clone());
    active.model_profile_id = Set(runtime.model_profile_id.clone());
    active.agent_type = Set(Some(runtime.agent_type.clone()));
    active.model = Set(runtime.model.clone());
    active.mode_id = Set(runtime.mode_id.clone());
    active.reasoning = Set(runtime.reasoning.clone());
    active.resolution = Set(Some(
        serde_json::to_string(runtime).map_err(|e| DbError::Validation(e.to_string()))?,
    ));
    active.updated_at = Set(Utc::now());
    active.update(conn).await?;
    Ok(())
}

pub async fn bind_context_package<C: ConnectionTrait>(
    conn: &C,
    task_id: i32,
    run_seq: i32,
    package_id: &str,
) -> Result<(), DbError> {
    if let Some(row) = work_task_run::Entity::find_by_id((task_id, run_seq))
        .one(conn)
        .await?
    {
        let mut active = row.into_active_model();
        active.context_package_id = Set(Some(package_id.to_string()));
        active.updated_at = Set(Utc::now());
        active.update(conn).await?;
    }
    Ok(())
}

pub async fn list_runs(
    conn: &DatabaseConnection,
    task_id: i32,
) -> Result<Vec<WorkTaskRunInfo>, DbError> {
    Ok(work_task_run::Entity::find()
        .filter(work_task_run::Column::TaskId.eq(task_id))
        .order_by_desc(work_task_run::Column::RunSeq)
        .all(conn)
        .await?
        .into_iter()
        .map(run_info)
        .collect())
}

fn run_info(row: work_task_run::Model) -> WorkTaskRunInfo {
    WorkTaskRunInfo {
        task_id: row.task_id,
        run_seq: row.run_seq,
        status: row.status,
        agent_profile_id: row.agent_profile_id,
        model_profile_id: row.model_profile_id,
        agent_type: row.agent_type,
        model: row.model,
        mode_id: row.mode_id,
        reasoning: row.reasoning,
        resolution: row
            .resolution
            .as_deref()
            .and_then(|v| serde_json::from_str(v).ok()),
        conversation_id: row.conversation_id,
        worktree_folder_id: row.worktree_folder_id,
        context_package_id: row.context_package_id,
        created_at: row.created_at,
        started_at: row.started_at,
        finished_at: row.finished_at,
        updated_at: row.updated_at,
    }
}

pub async fn dependencies_satisfied<C: ConnectionTrait>(
    conn: &C,
    child_task_id: i32,
) -> Result<bool, DbError> {
    let edges = work_task_dependency::Entity::find()
        .filter(work_task_dependency::Column::ChildTaskId.eq(child_task_id))
        .all(conn)
        .await?;
    if edges.is_empty() {
        return Ok(true);
    }
    let parent_ids = edges.iter().map(|e| e.parent_task_id).collect::<Vec<_>>();
    let parents = work_task::Entity::find()
        .filter(work_task::Column::Id.is_in(parent_ids.iter().copied()))
        .all(conn)
        .await?;
    if parents.len() != parent_ids.len() {
        return Ok(false);
    }
    let parent_by_id = parents
        .into_iter()
        .map(|p| (p.id, p))
        .collect::<BTreeMap<_, _>>();
    for edge in edges {
        let Some(parent) = parent_by_id.get(&edge.parent_task_id) else {
            return Ok(false);
        };
        if parent.deleted_at.is_some() {
            return Ok(false);
        }
        match edge.kind.as_str() {
            "completion" => {
                if parent.status != crate::db::entities::work_task::WorkTaskStatus::Done {
                    return Ok(false);
                }
            }
            "integration_source" => {
                if !integration_source_ready(conn, parent).await? {
                    return Ok(false);
                }
            }
            _ => return Ok(false),
        }
    }
    Ok(true)
}

async fn integration_source_ready<C: ConnectionTrait>(
    conn: &C,
    parent: &work_task::Model,
) -> Result<bool, DbError> {
    if parent.status != crate::db::entities::work_task::WorkTaskStatus::Review {
        return Ok(false);
    }
    if parent
        .work_branch
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .is_empty()
    {
        return Ok(false);
    }
    Ok(work_task_handoff::Entity::find()
        .filter(work_task_handoff::Column::TaskId.eq(parent.id))
        .filter(work_task_handoff::Column::RunSeq.eq(parent.run_seq))
        .one(conn)
        .await?
        .is_some())
}

pub const UNMET_DEPENDENCY: &str = "workTask.dependency.unmet";

pub fn is_unmet_dependency(err: &DbError) -> bool {
    matches!(err, DbError::Validation(msg) if msg.starts_with(UNMET_DEPENDENCY))
}

pub async fn team_task_launch_allowed<C: ConnectionTrait>(
    conn: &C,
    task_id: i32,
) -> Result<bool, DbError> {
    let Some(binding) = team_run_task::Entity::find()
        .filter(team_run_task::Column::TaskId.eq(task_id))
        .one(conn)
        .await?
    else {
        return Ok(true);
    };
    let Some(run) = team_run::Entity::find_by_id(binding.team_run_id)
        .one(conn)
        .await?
    else {
        return Ok(false);
    };
    if run.control_state != "running" {
        return Ok(false);
    }
    if run.max_concurrent <= 0 {
        return Ok(true);
    }
    let task_ids = team_run_task::Entity::find()
        .filter(team_run_task::Column::TeamRunId.eq(&run.id))
        .all(conn)
        .await?
        .into_iter()
        .map(|b| b.task_id)
        .collect::<Vec<_>>();
    let active = work_task::Entity::find()
        .filter(work_task::Column::Id.is_in(task_ids))
        .filter(work_task::Column::Status.is_in([
            crate::db::entities::work_task::WorkTaskStatus::Preparing,
            crate::db::entities::work_task::WorkTaskStatus::Running,
            crate::db::entities::work_task::WorkTaskStatus::AwaitingInput,
            crate::db::entities::work_task::WorkTaskStatus::Merging,
        ]))
        .count(conn)
        .await?;
    Ok(active < run.max_concurrent as u64)
}

/// Serializes folder-graph writers so two concurrent opposite edges cannot
/// both pass a check-then-insert cycle test.
static DEPENDENCY_WRITE: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

pub async fn add_dependency<C: ConnectionTrait>(
    conn: &C,
    parent_task_id: i32,
    child_task_id: i32,
    kind: &str,
) -> Result<(), DbError> {
    let _guard = DEPENDENCY_WRITE.lock().await;
    add_dependency_locked(conn, parent_task_id, child_task_id, kind).await
}

async fn add_dependency_locked<C: ConnectionTrait>(
    conn: &C,
    parent_task_id: i32,
    child_task_id: i32,
    kind: &str,
) -> Result<(), DbError> {
    if parent_task_id == child_task_id {
        return Err(DbError::Validation("a task cannot depend on itself".into()));
    }
    if kind.trim().is_empty() || kind.len() > 64 {
        return Err(DbError::Validation(
            "dependency kind must be 1..64 bytes".into(),
        ));
    }
    if !matches!(kind, "completion" | "integration_source") {
        return Err(DbError::Validation(
            "dependency kind must be completion or integration_source".into(),
        ));
    }
    let parent = work_task::Entity::find_by_id(parent_task_id)
        .one(conn)
        .await?
        .ok_or_else(|| DbError::NotFound(format!("work task {parent_task_id}")))?;
    let child = work_task::Entity::find_by_id(child_task_id)
        .one(conn)
        .await?
        .ok_or_else(|| DbError::NotFound(format!("work task {child_task_id}")))?;
    if parent.deleted_at.is_some()
        || child.deleted_at.is_some()
        || parent.folder_id != child.folder_id
    {
        return Err(DbError::Validation(
            "dependencies require live tasks in the same project".into(),
        ));
    }
    let edges = work_task_dependency::Entity::find().all(conn).await?;
    if edges.iter().any(|e| {
        e.parent_task_id == parent_task_id && e.child_task_id == child_task_id
    }) {
        return Err(DbError::Validation("duplicate dependency".into()));
    }
    let mut children: BTreeMap<i32, Vec<i32>> = BTreeMap::new();
    for edge in &edges {
        children
            .entry(edge.parent_task_id)
            .or_default()
            .push(edge.child_task_id);
    }
    let mut pending = vec![child_task_id];
    let mut seen = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if id == parent_task_id {
            return Err(DbError::Validation(
                "dependency would create a cycle".into(),
            ));
        }
        if seen.insert(id) {
            pending.extend(children.get(&id).into_iter().flatten().copied());
        }
    }
    work_task_dependency::ActiveModel {
        parent_task_id: Set(parent_task_id),
        child_task_id: Set(child_task_id),
        kind: Set(kind.to_string()),
        created_at: Set(Utc::now()),
    }
    .insert(conn)
    .await?;
    Ok(())
}

pub async fn list_dependencies(
    conn: &DatabaseConnection,
    task_id: i32,
) -> Result<Vec<WorkTaskDependencyInfo>, DbError> {
    let rows = work_task_dependency::Entity::find()
        .filter(
            sea_orm::Condition::any()
                .add(work_task_dependency::Column::ParentTaskId.eq(task_id))
                .add(work_task_dependency::Column::ChildTaskId.eq(task_id)),
        )
        .all(conn)
        .await?;
    Ok(rows
        .into_iter()
        .map(|e| WorkTaskDependencyInfo {
            parent_task_id: e.parent_task_id,
            child_task_id: e.child_task_id,
            kind: e.kind,
        })
        .collect())
}

pub async fn save_handoff(
    conn: &DatabaseConnection,
    task_id: i32,
    draft: WorkTaskHandoffDraft,
) -> Result<WorkTaskHandoffInfo, DbError> {
    if draft.summary.trim().is_empty() {
        return Err(DbError::Validation("handoff summary is required".into()));
    }
    if draft.summary.len() > 16 * 1024 {
        return Err(DbError::Validation(
            "handoff summary exceeds 16384 bytes".into(),
        ));
    }
    for (name, values) in [
        ("artifacts", &draft.artifacts),
        ("risks", &draft.risks),
        ("openQuestions", &draft.open_questions),
    ] {
        if values.len() > 64 || values.iter().any(|value| value.len() > 4096) {
            return Err(DbError::Validation(format!(
                "handoff {name} exceeds 64 items or 4096 bytes per item"
            )));
        }
    }
    if serde_json::to_vec(&draft)
        .map_err(|e| DbError::Validation(e.to_string()))?
        .len()
        > 64 * 1024
    {
        return Err(DbError::Validation(
            "handoff exceeds 65536 serialized bytes".into(),
        ));
    }
    let task = work_task::Entity::find_by_id(task_id)
        .one(conn)
        .await?
        .ok_or_else(|| DbError::NotFound(format!("work task {task_id}")))?;
    let now = Utc::now();
    let active = work_task_handoff::ActiveModel {
        task_id: Set(task_id),
        run_seq: Set(task.run_seq),
        summary: Set(draft.summary.trim().to_string()),
        artifacts: Set(serde_json::to_string(&draft.artifacts).unwrap_or_else(|_| "[]".into())),
        risks: Set(serde_json::to_string(&draft.risks).unwrap_or_else(|_| "[]".into())),
        open_questions: Set(
            serde_json::to_string(&draft.open_questions).unwrap_or_else(|_| "[]".into())
        ),
        created_at: Set(now),
    };
    work_task_handoff::Entity::delete_by_id((task_id, task.run_seq))
        .exec(conn)
        .await?;
    let row = active.insert(conn).await?;
    Ok(handoff_info(row, Some(&task)))
}

pub async fn get_handoff(
    conn: &DatabaseConnection,
    task_id: i32,
    run_seq: Option<i32>,
) -> Result<Option<WorkTaskHandoffInfo>, DbError> {
    let mut query =
        work_task_handoff::Entity::find().filter(work_task_handoff::Column::TaskId.eq(task_id));
    if let Some(seq) = run_seq {
        query = query.filter(work_task_handoff::Column::RunSeq.eq(seq));
    }
    let row = query
        .order_by_desc(work_task_handoff::Column::RunSeq)
        .one(conn)
        .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    let task = work_task::Entity::find_by_id(task_id).one(conn).await?;
    Ok(Some(handoff_info(row, task.as_ref())))
}

fn handoff_info(row: work_task_handoff::Model, task: Option<&work_task::Model>) -> WorkTaskHandoffInfo {
    WorkTaskHandoffInfo {
        task_id: row.task_id,
        run_seq: row.run_seq,
        summary: row.summary,
        artifacts: serde_json::from_str(&row.artifacts).unwrap_or_default(),
        risks: serde_json::from_str(&row.risks).unwrap_or_default(),
        open_questions: serde_json::from_str(&row.open_questions).unwrap_or_default(),
        source_branch: task.and_then(|t| t.work_branch.clone()),
        source_head: None,
        created_at: row.created_at,
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn create_team_run<C: ConnectionTrait>(
    conn: &C,
    id: &str,
    folder_id: i32,
    team_id: &str,
    workflow_id: &str,
    workflow_version: i32,
    max_concurrent: i32,
    definition_hash: &str,
) -> Result<(), DbError> {
    let now = Utc::now();
    team_run::ActiveModel {
        id: Set(id.to_string()),
        folder_id: Set(folder_id),
        team_id: Set(team_id.to_string()),
        workflow_id: Set(workflow_id.to_string()),
        workflow_version: Set(workflow_version),
        max_concurrent: Set(max_concurrent),
        control_state: Set("running".into()),
        definition_hash: Set(definition_hash.to_string()),
        created_at: Set(now),
        updated_at: Set(now),
        finished_at: Set(None),
    }
    .insert(conn)
    .await?;
    Ok(())
}

pub async fn bind_team_task<C: ConnectionTrait>(
    conn: &C,
    run_id: &str,
    node_id: &str,
    task_id: i32,
) -> Result<(), DbError> {
    team_run_task::ActiveModel {
        team_run_id: Set(run_id.into()),
        node_id: Set(node_id.into()),
        task_id: Set(task_id),
        created_at: Set(Utc::now()),
    }
    .insert(conn)
    .await?;
    Ok(())
}

pub async fn set_team_control(
    conn: &DatabaseConnection,
    run_id: &str,
    state: &str,
) -> Result<(), DbError> {
    if !matches!(state, "running" | "paused" | "canceled") {
        return Err(DbError::Validation("invalid team control state".into()));
    }
    let row = team_run::Entity::find_by_id(run_id)
        .one(conn)
        .await?
        .ok_or_else(|| DbError::NotFound(format!("team run {run_id}")))?;
    let mut active = row.into_active_model();
    active.control_state = Set(state.into());
    active.updated_at = Set(Utc::now());
    if state == "canceled" {
        active.finished_at = Set(Some(Utc::now()));
    }
    active.update(conn).await?;
    Ok(())
}

pub async fn team_run_tasks(
    conn: &DatabaseConnection,
    run_id: &str,
) -> Result<Vec<team_run_task::Model>, DbError> {
    Ok(team_run_task::Entity::find()
        .filter(team_run_task::Column::TeamRunId.eq(run_id))
        .all(conn)
        .await?)
}

pub async fn list_team_runs(
    conn: &DatabaseConnection,
    folder_id: i32,
) -> Result<Vec<TeamRunInfo>, DbError> {
    let runs = team_run::Entity::find()
        .filter(team_run::Column::FolderId.eq(folder_id))
        .order_by_desc(team_run::Column::CreatedAt)
        .all(conn)
        .await?;
    let mut out = Vec::with_capacity(runs.len());
    for run in runs {
        let bindings = team_run_tasks(conn, &run.id).await?;
        let mut nodes = Vec::new();
        for binding in bindings {
            if let Some(task) = work_task::Entity::find_by_id(binding.task_id)
                .one(conn)
                .await?
            {
                nodes.push(TeamRunNodeInfo {
                    node_id: binding.node_id,
                    task_id: task.id,
                    title: task.title,
                    status: crate::db::service::work_task_service::status_str(task.status).into(),
                    run_seq: task.run_seq,
                });
            }
        }
        nodes.sort_by(|a, b| a.node_id.cmp(&b.node_id));
        let status = derive_team_status(&run.control_state, &nodes);
        out.push(TeamRunInfo {
            id: run.id,
            folder_id: run.folder_id,
            team_id: run.team_id,
            workflow_id: run.workflow_id,
            workflow_version: run.workflow_version,
            control_state: run.control_state,
            status,
            definition_hash: run.definition_hash,
            nodes,
            created_at: run.created_at,
            updated_at: run.updated_at,
            finished_at: run.finished_at,
        });
    }
    Ok(out)
}

fn derive_team_status(control: &str, nodes: &[TeamRunNodeInfo]) -> String {
    if control == "canceled" {
        return "canceled".into();
    }
    if nodes.iter().any(|n| n.status == "failed") {
        return "failed".into();
    }
    if !nodes.is_empty() && nodes.iter().all(|n| n.status == "done") {
        return "done".into();
    }
    if control == "paused" {
        return "paused".into();
    }
    if nodes.iter().any(|n| {
        matches!(
            n.status.as_str(),
            "running" | "preparing" | "awaiting_input" | "merging"
        )
    }) {
        return "running".into();
    }
    "queued".into()
}

pub async fn integration_plan(
    conn: &DatabaseConnection,
    task_id: i32,
    repo_path: Option<&str>,
) -> Result<IntegrationPlan, DbError> {
    let task = work_task::Entity::find_by_id(task_id)
        .one(conn)
        .await?
        .ok_or_else(|| DbError::NotFound(format!("work task {task_id}")))?;
    let cfg = serde_json::from_str::<WorkTaskConfig>(&task.config).unwrap_or_default();
    let snapshot = cfg.integration_snapshot.as_ref();
    let mut edges = work_task_dependency::Entity::find()
        .filter(work_task_dependency::Column::ChildTaskId.eq(task_id))
        .filter(work_task_dependency::Column::Kind.eq("integration_source"))
        .all(conn)
        .await?;
    edges.sort_by_key(|e| e.parent_task_id);
    let mut sources = Vec::new();
    for (order, edge) in edges.iter().enumerate() {
        let parent = work_task::Entity::find_by_id(edge.parent_task_id)
            .one(conn)
            .await?
            .ok_or_else(|| DbError::NotFound(format!("work task {}", edge.parent_task_id)))?;
        let has_handoff = work_task_handoff::Entity::find()
            .filter(work_task_handoff::Column::TaskId.eq(parent.id))
            .filter(work_task_handoff::Column::RunSeq.eq(parent.run_seq))
            .one(conn)
            .await?
            .is_some();
        let captured = snapshot
            .and_then(|s| s.sources.iter().find(|c| c.task_id == parent.id));
        let current_head = match (repo_path, parent.work_branch.as_deref()) {
            (Some(path), Some(branch)) if !branch.trim().is_empty() => {
                crate::work_task::git::rev_parse(path, branch).await.ok()
            }
            _ => None,
        };
        let stale = match captured {
            Some(c) => {
                c.run_seq != parent.run_seq
                    || current_head
                        .as_deref()
                        .is_some_and(|head| head != c.head)
            }
            None => false,
        };
        sources.push(IntegrationSourceInfo {
            task_id: parent.id,
            title: parent.title,
            status: crate::db::service::work_task_service::status_str(parent.status).into(),
            run_seq: parent.run_seq,
            branch: parent.work_branch,
            current_head,
            captured_head: captured.map(|c| c.head.clone()),
            captured_run_seq: captured.map(|c| c.run_seq),
            has_handoff,
            merge_order: order as i32,
            stale,
        });
    }
    let conflicts = match repo_path {
        Some(path) if crate::work_task::git::has_merge_head(path).await.unwrap_or(false) => {
            vec!["MERGE_HEAD".into()]
        }
        _ => Vec::new(),
    };
    let status = if task.status == work_task::WorkTaskStatus::Done {
        "landed"
    } else if sources.is_empty() {
        "no_sources"
    } else if !conflicts.is_empty() {
        "conflict"
    } else if sources.iter().any(|s| s.stale) {
        "stale"
    } else if sources.iter().any(|s| {
        s.status != "review" || !s.has_handoff || s.branch.as_deref().unwrap_or("").is_empty()
    }) {
        "waiting_source"
    } else {
        "eligible"
    };
    Ok(IntegrationPlan {
        task_id,
        status: status.into(),
        sources,
        conflicts,
    })
}

pub async fn refresh_integration_plan(
    conn: &DatabaseConnection,
    task_id: i32,
    repo_path: &str,
) -> Result<IntegrationPlan, DbError> {
    let plan = integration_plan(conn, task_id, Some(repo_path)).await?;
    let mut sources = Vec::new();
    for source in &plan.sources {
        let head = source
            .current_head
            .clone()
            .or(source.captured_head.clone())
            .ok_or_else(|| {
                DbError::Validation(format!(
                    "workTask.integration.invalidSource: source {} has no head",
                    source.task_id
                ))
            })?;
        let branch = source.branch.clone().ok_or_else(|| {
            DbError::Validation(format!(
                "workTask.integration.invalidSource: source {} has no branch",
                source.task_id
            ))
        })?;
        sources.push(IntegrationSourceCapture {
            task_id: source.task_id,
            run_seq: source.run_seq,
            branch,
            head,
            merge_order: source.merge_order,
        });
    }
    write_integration_snapshot(conn, task_id, IntegrationSnapshot { sources }).await?;
    integration_plan(conn, task_id, Some(repo_path)).await
}

async fn write_integration_snapshot(
    conn: &DatabaseConnection,
    task_id: i32,
    snapshot: IntegrationSnapshot,
) -> Result<(), DbError> {
    let row = work_task::Entity::find_by_id(task_id)
        .one(conn)
        .await?
        .ok_or_else(|| DbError::NotFound(format!("work task {task_id}")))?;
    let mut cfg = serde_json::from_str::<WorkTaskConfig>(&row.config).unwrap_or_default();
    cfg.integration_snapshot = Some(snapshot);
    let mut active = row.into_active_model();
    active.config = Set(serde_json::to_string(&cfg).map_err(|e| DbError::Validation(e.to_string()))?);
    active.update(conn).await?;
    Ok(())
}

pub async fn assert_integration_landing(
    conn: &DatabaseConnection,
    task_id: i32,
    repo_path: &str,
    landing_sha: &str,
) -> Result<(), DbError> {
    let edges = work_task_dependency::Entity::find()
        .filter(work_task_dependency::Column::ChildTaskId.eq(task_id))
        .filter(work_task_dependency::Column::Kind.eq("integration_source"))
        .all(conn)
        .await?;
    if edges.is_empty() {
        return Ok(());
    }
    let task = work_task::Entity::find_by_id(task_id)
        .one(conn)
        .await?
        .ok_or_else(|| DbError::NotFound(format!("work task {task_id}")))?;
    let cfg = serde_json::from_str::<WorkTaskConfig>(&task.config).unwrap_or_default();
    let snapshot = cfg.integration_snapshot.ok_or_else(|| {
        DbError::Validation("workTask.integration.stalePlan: no captured source heads".into())
    })?;
    if snapshot.sources.len() != edges.len() {
        return Err(DbError::Validation(
            "workTask.integration.stalePlan: captured source set does not match".into(),
        ));
    }
    for source in snapshot.sources {
        let contained =
            crate::work_task::git::is_ancestor(repo_path, &source.head, landing_sha)
                .await
                .map_err(|e| DbError::Validation(e.to_string()))?;
        if !contained {
            return Err(DbError::Validation(format!(
                "workTask.integration.notContained: {} is not in {landing_sha}",
                source.head
            )));
        }
    }
    Ok(())
}

pub async fn folder_path_for_task(
    conn: &DatabaseConnection,
    task_id: i32,
) -> Result<Option<String>, DbError> {
    let Some(task) = work_task::Entity::find_by_id(task_id).one(conn).await? else {
        return Ok(None);
    };
    Ok(folder::Entity::find_by_id(task.folder_id)
        .one(conn)
        .await?
        .map(|f| f.path))
}
