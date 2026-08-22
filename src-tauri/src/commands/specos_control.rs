//! Shared Tauri/Axum commands for Agent Team and Context control surfaces.

use sea_orm::TransactionTrait;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

use crate::db::entities::work_task::WorkTaskStatus;
use crate::db::error::DbError;
use crate::db::service::{specos_runtime_service, work_task_service};
use crate::db::AppDatabase;
use crate::models::*;
use crate::web::event_bridge::{emit_event, EventEmitter, WorkTaskChange, WORK_TASK_CHANGED_EVENT};

pub async fn agent_catalog_get_core(
    db: &AppDatabase,
    folder_id: i32,
) -> Result<AgentCatalog, DbError> {
    let root = crate::specos_control::project_root(&db.conn, folder_id).await?;
    crate::specos_control::load_agents(&root)
}

pub async fn agent_catalog_save_core(
    db: &AppDatabase,
    folder_id: i32,
    catalog: AgentCatalog,
) -> Result<AgentCatalog, DbError> {
    let root = crate::specos_control::project_root(&db.conn, folder_id).await?;
    let context = crate::specos_control::load_context(&root)?;
    let loadout_ids = context
        .loadouts
        .iter()
        .map(|loadout| loadout.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    for profile in &catalog.agent_profiles {
        if let Some(loadout_id) = &profile.context_loadout_id {
            if !loadout_ids.contains(loadout_id.as_str()) {
                return Err(DbError::Validation(format!(
                    "agent profile '{}' references missing Context Loadout '{loadout_id}'",
                    profile.id
                )));
            }
        }
    }
    crate::specos_control::save_agents(&root, catalog)
}

pub async fn team_catalog_get_core(
    db: &AppDatabase,
    folder_id: i32,
) -> Result<TeamCatalog, DbError> {
    let root = crate::specos_control::project_root(&db.conn, folder_id).await?;
    crate::specos_control::load_teams(&root)
}

pub async fn team_catalog_save_core(
    db: &AppDatabase,
    folder_id: i32,
    catalog: TeamCatalog,
) -> Result<TeamCatalog, DbError> {
    let root = crate::specos_control::project_root(&db.conn, folder_id).await?;
    let agents = crate::specos_control::load_agents(&root)?;
    let profile_ids = agents
        .agent_profiles
        .iter()
        .map(|p| p.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let model_ids = agents
        .model_profiles
        .iter()
        .map(|p| p.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let context = crate::specos_control::load_context(&root)?;
    let loadout_ids = context
        .loadouts
        .iter()
        .map(|loadout| loadout.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    for team in &catalog.teams {
        for member in &team.member_profile_ids {
            if !profile_ids.contains(member.as_str()) {
                return Err(DbError::Validation(format!(
                    "team '{}' references missing AgentProfile '{member}'",
                    team.id
                )));
            }
        }
    }
    for workflow in &catalog.workflows {
        let team = catalog
            .teams
            .iter()
            .find(|team| team.id == workflow.team_id)
            .ok_or_else(|| {
                DbError::Validation(format!(
                    "workflow '{}' references missing Team '{}'",
                    workflow.id, workflow.team_id
                ))
            })?;
        for node in &workflow.nodes {
            if !profile_ids.contains(node.agent_profile_id.as_str()) {
                return Err(DbError::Validation(format!(
                    "workflow node '{}' references missing AgentProfile '{}'",
                    node.id, node.agent_profile_id
                )));
            }
            if !team.member_profile_ids.contains(&node.agent_profile_id) {
                return Err(DbError::Validation(format!(
                    "workflow node '{}' AgentProfile '{}' is not a member of Team '{}'",
                    node.id, node.agent_profile_id, team.id
                )));
            }
            if node
                .model_profile_id
                .as_ref()
                .is_some_and(|id| !model_ids.contains(id.as_str()))
            {
                return Err(DbError::Validation(format!(
                    "workflow node '{}' references a missing Model Profile",
                    node.id
                )));
            }
            if node
                .context_loadout_id
                .as_ref()
                .is_some_and(|id| !loadout_ids.contains(id.as_str()))
            {
                return Err(DbError::Validation(format!(
                    "workflow node '{}' references a missing Context Loadout",
                    node.id
                )));
            }
        }
    }
    crate::specos_control::save_teams(&root, catalog)
}

pub async fn team_run_list_core(
    db: &AppDatabase,
    folder_id: i32,
) -> Result<Vec<TeamRunInfo>, DbError> {
    specos_runtime_service::list_team_runs(&db.conn, folder_id).await
}

pub async fn team_run_start_core(
    emitter: &EventEmitter,
    db: &AppDatabase,
    folder_id: i32,
    workflow_id: String,
) -> Result<TeamRunInfo, DbError> {
    let root = crate::specos_control::project_root(&db.conn, folder_id).await?;
    let catalog = crate::specos_control::load_teams(&root)?;
    if !catalog.validation_errors.is_empty() {
        return Err(DbError::Validation(catalog.validation_errors.join("; ")));
    }
    let agents = crate::specos_control::load_agents(&root)?;
    if !agents.validation_errors.is_empty() {
        return Err(DbError::Validation(agents.validation_errors.join("; ")));
    }
    let workflow = catalog
        .workflows
        .iter()
        .find(|w| w.id == workflow_id)
        .cloned()
        .ok_or_else(|| DbError::NotFound(format!("workflow {workflow_id}")))?;
    let team = catalog
        .teams
        .iter()
        .find(|t| t.id == workflow.team_id)
        .ok_or_else(|| DbError::Validation(format!("workflow '{}' has no Team", workflow.id)))?;
    let profile_ids = agents
        .agent_profiles
        .iter()
        .filter(|p| p.enabled)
        .map(|p| p.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    for node in &workflow.nodes {
        if !profile_ids.contains(node.agent_profile_id.as_str()) {
            return Err(DbError::Validation(format!(
                "AgentProfile '{}' is unavailable",
                node.agent_profile_id
            )));
        }
        if !team.member_profile_ids.contains(&node.agent_profile_id) {
            return Err(DbError::Validation(format!(
                "AgentProfile '{}' is not a member of Team '{}'",
                node.agent_profile_id, team.id
            )));
        }
    }
    let definition =
        serde_yaml::to_string(&workflow).map_err(|e| DbError::Validation(e.to_string()))?;
    let definition_hash = format!("{:x}", Sha256::digest(definition.as_bytes()));
    let run_id = format!("team-{}", uuid::Uuid::new_v4().simple());
    let txn = db.conn.begin().await?;
    specos_runtime_service::create_team_run(
        &txn,
        &run_id,
        folder_id,
        &team.id,
        &workflow.id,
        workflow.version,
        workflow.max_concurrent,
        &definition_hash,
    )
    .await?;
    let mut tasks = BTreeMap::new();
    for node in &workflow.nodes {
        let draft = WorkTaskDraft {
            folder_id,
            title: node.title.clone(),
            config: serde_json::to_value(WorkTaskConfig {
                prompt_blocks: vec![serde_json::json!({"type":"text","text":node.prompt})],
                display_text: node.prompt.clone(),
                agent_type: None,
                agent_profile_id: Some(node.agent_profile_id.clone()),
                model_profile_id: node.model_profile_id.clone(),
                context_loadout_id: node.context_loadout_id.clone(),
                team_run_id: Some(run_id.clone()),
                team_node_id: Some(node.id.clone()),
                mode_id: None,
                config_values: Default::default(),
                label_snapshot: None,
                integration_snapshot: None,
            })
            .map_err(|e| DbError::Validation(e.to_string()))?,
            task_kind: Default::default(),
        };
        let task = work_task_service::create_in_transaction(&txn, draft).await?;
        specos_runtime_service::bind_team_task(&txn, &run_id, &node.id, task.id).await?;
        tasks.insert(node.id.clone(), task.id);
    }
    for node in &workflow.nodes {
        for parent in &node.depends_on {
            specos_runtime_service::add_dependency(
                &txn,
                tasks[parent],
                tasks[&node.id],
                "completion",
            )
            .await?;
        }
    }
    txn.commit().await?;
    for task_id in tasks.values() {
        match work_task_service::claim_for_run(
            &db.conn,
            *task_id,
            WorkTaskStatus::Todo,
            "team_orchestrator",
        )
        .await
        {
            Ok(_) => {}
            Err(e) if specos_runtime_service::is_unmet_dependency(&e) => {}
            Err(e) => return Err(e),
        }
    }
    emit_event(emitter, WORK_TASK_CHANGED_EVENT, WorkTaskChange::Refresh);
    if let Some(engine) = crate::work_task::engine() {
        tokio::spawn(async move {
            engine.pump_folder(folder_id).await;
        });
    }
    specos_runtime_service::list_team_runs(&db.conn, folder_id)
        .await?
        .into_iter()
        .find(|r| r.id == run_id)
        .ok_or_else(|| DbError::NotFound("new team run projection".into()))
}

pub async fn team_run_control_core(
    db: &AppDatabase,
    run_id: String,
    action: String,
) -> Result<(), DbError> {
    let state = match action.as_str() {
        "pause" => "paused",
        "resume" => "running",
        "cancel" => "canceled",
        _ => {
            return Err(DbError::Validation(
                "action must be pause, resume, or cancel".into(),
            ))
        }
    };
    let cancel_engine = if action == "cancel" {
        Some(
            crate::work_task::engine()
                .ok_or_else(|| DbError::Validation("task engine not running".into()))?,
        )
    } else {
        None
    };
    specos_runtime_service::set_team_control(&db.conn, &run_id, state).await?;
    let bindings = specos_runtime_service::team_run_tasks(&db.conn, &run_id).await?;
    if let Some(engine) = cancel_engine {
        for binding in bindings {
            let _ = engine
                .cancel(binding.task_id, Some("team run canceled".into()))
                .await;
        }
    } else if action == "resume" {
        if let Some(first) = bindings.first() {
            if let Ok(task) = work_task_service::get_model(&db.conn, first.task_id).await {
                if let Some(engine) = crate::work_task::engine() {
                    tokio::spawn(async move {
                        engine.pump_folder(task.folder_id).await;
                    });
                }
            }
        }
    }
    Ok(())
}

pub async fn context_config_get_core(
    db: &AppDatabase,
    folder_id: i32,
) -> Result<ContextConfig, DbError> {
    let root = crate::specos_control::project_root(&db.conn, folder_id).await?;
    crate::specos_control::load_context(&root)
}

pub async fn context_config_save_core(
    db: &AppDatabase,
    folder_id: i32,
    config: ContextConfig,
) -> Result<ContextConfig, DbError> {
    let root = crate::specos_control::project_root(&db.conn, folder_id).await?;
    crate::specos_control::save_context(&root, config)
}

pub async fn context_overview_core(
    db: &AppDatabase,
    memory: &crate::memory::MemoryService,
    folder_id: i32,
) -> Result<ContextOverview, DbError> {
    let root = crate::specos_control::project_root(&db.conn, folder_id).await?;
    crate::context::overview(&db.conn, folder_id, &root, memory).await
}

pub async fn context_package_get_core(
    db: &AppDatabase,
    id: String,
) -> Result<ContextPackageInfo, DbError> {
    crate::context::package_get(&db.conn, &id).await
}
pub async fn work_task_runs_core(
    db: &AppDatabase,
    task_id: i32,
) -> Result<Vec<WorkTaskRunInfo>, DbError> {
    specos_runtime_service::list_runs(&db.conn, task_id).await
}
pub async fn work_task_dependencies_core(
    db: &AppDatabase,
    task_id: i32,
) -> Result<Vec<WorkTaskDependencyInfo>, DbError> {
    specos_runtime_service::list_dependencies(&db.conn, task_id).await
}
pub async fn work_task_handoff_get_core(
    db: &AppDatabase,
    task_id: i32,
    run_seq: Option<i32>,
) -> Result<Option<WorkTaskHandoffInfo>, DbError> {
    specos_runtime_service::get_handoff(&db.conn, task_id, run_seq).await
}
pub async fn work_task_handoff_save_core(
    db: &AppDatabase,
    task_id: i32,
    draft: WorkTaskHandoffDraft,
) -> Result<WorkTaskHandoffInfo, DbError> {
    specos_runtime_service::save_handoff(&db.conn, task_id, draft).await
}

pub async fn work_task_integration_plan_core(
    db: &AppDatabase,
    task_id: i32,
) -> Result<IntegrationPlan, DbError> {
    let path = specos_runtime_service::folder_path_for_task(&db.conn, task_id).await?;
    specos_runtime_service::integration_plan(&db.conn, task_id, path.as_deref()).await
}

pub async fn work_task_integration_refresh_core(
    db: &AppDatabase,
    task_id: i32,
) -> Result<IntegrationPlan, DbError> {
    let path = specos_runtime_service::folder_path_for_task(&db.conn, task_id)
        .await?
        .ok_or_else(|| DbError::NotFound(format!("folder for work task {task_id}")))?;
    specos_runtime_service::refresh_integration_plan(&db.conn, task_id, &path).await
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
pub async fn specos_agent_catalog_get(
    db: tauri::State<'_, AppDatabase>,
    folder_id: i32,
) -> Result<AgentCatalog, DbError> {
    agent_catalog_get_core(&db, folder_id).await
}
#[cfg(feature = "tauri-runtime")]
#[tauri::command]
pub async fn specos_agent_catalog_save(
    db: tauri::State<'_, AppDatabase>,
    folder_id: i32,
    catalog: AgentCatalog,
) -> Result<AgentCatalog, DbError> {
    agent_catalog_save_core(&db, folder_id, catalog).await
}
#[cfg(feature = "tauri-runtime")]
#[tauri::command]
pub async fn specos_team_catalog_get(
    db: tauri::State<'_, AppDatabase>,
    folder_id: i32,
) -> Result<TeamCatalog, DbError> {
    team_catalog_get_core(&db, folder_id).await
}
#[cfg(feature = "tauri-runtime")]
#[tauri::command]
pub async fn specos_team_catalog_save(
    db: tauri::State<'_, AppDatabase>,
    folder_id: i32,
    catalog: TeamCatalog,
) -> Result<TeamCatalog, DbError> {
    team_catalog_save_core(&db, folder_id, catalog).await
}
#[cfg(feature = "tauri-runtime")]
#[tauri::command]
pub async fn specos_team_run_list(
    db: tauri::State<'_, AppDatabase>,
    folder_id: i32,
) -> Result<Vec<TeamRunInfo>, DbError> {
    team_run_list_core(&db, folder_id).await
}
#[cfg(feature = "tauri-runtime")]
#[tauri::command]
pub async fn specos_team_run_start(
    app: tauri::AppHandle,
    db: tauri::State<'_, AppDatabase>,
    folder_id: i32,
    workflow_id: String,
) -> Result<TeamRunInfo, DbError> {
    team_run_start_core(&EventEmitter::Tauri(app), &db, folder_id, workflow_id).await
}
#[cfg(feature = "tauri-runtime")]
#[tauri::command]
pub async fn specos_team_run_control(
    db: tauri::State<'_, AppDatabase>,
    run_id: String,
    action: String,
) -> Result<(), DbError> {
    team_run_control_core(&db, run_id, action).await
}
#[cfg(feature = "tauri-runtime")]
#[tauri::command]
pub async fn specos_context_config_get(
    db: tauri::State<'_, AppDatabase>,
    folder_id: i32,
) -> Result<ContextConfig, DbError> {
    context_config_get_core(&db, folder_id).await
}
#[cfg(feature = "tauri-runtime")]
#[tauri::command]
pub async fn specos_context_config_save(
    db: tauri::State<'_, AppDatabase>,
    folder_id: i32,
    config: ContextConfig,
) -> Result<ContextConfig, DbError> {
    context_config_save_core(&db, folder_id, config).await
}
#[cfg(feature = "tauri-runtime")]
#[tauri::command]
pub async fn specos_context_overview(
    db: tauri::State<'_, AppDatabase>,
    memory: tauri::State<'_, std::sync::Arc<crate::memory::MemoryService>>,
    folder_id: i32,
) -> Result<ContextOverview, DbError> {
    context_overview_core(&db, &memory, folder_id).await
}
#[cfg(feature = "tauri-runtime")]
#[tauri::command]
pub async fn specos_context_package_get(
    db: tauri::State<'_, AppDatabase>,
    id: String,
) -> Result<ContextPackageInfo, DbError> {
    context_package_get_core(&db, id).await
}
#[cfg(feature = "tauri-runtime")]
#[tauri::command]
pub async fn specos_work_task_runs(
    db: tauri::State<'_, AppDatabase>,
    task_id: i32,
) -> Result<Vec<WorkTaskRunInfo>, DbError> {
    work_task_runs_core(&db, task_id).await
}
#[cfg(feature = "tauri-runtime")]
#[tauri::command]
pub async fn specos_work_task_dependencies(
    db: tauri::State<'_, AppDatabase>,
    task_id: i32,
) -> Result<Vec<WorkTaskDependencyInfo>, DbError> {
    work_task_dependencies_core(&db, task_id).await
}
#[cfg(feature = "tauri-runtime")]
#[tauri::command]
pub async fn specos_work_task_handoff_get(
    db: tauri::State<'_, AppDatabase>,
    task_id: i32,
    run_seq: Option<i32>,
) -> Result<Option<WorkTaskHandoffInfo>, DbError> {
    work_task_handoff_get_core(&db, task_id, run_seq).await
}
#[cfg(feature = "tauri-runtime")]
#[tauri::command]
pub async fn specos_work_task_handoff_save(
    db: tauri::State<'_, AppDatabase>,
    task_id: i32,
    draft: WorkTaskHandoffDraft,
) -> Result<WorkTaskHandoffInfo, DbError> {
    work_task_handoff_save_core(&db, task_id, draft).await
}
#[cfg(feature = "tauri-runtime")]
#[tauri::command]
pub async fn specos_work_task_integration_plan(
    db: tauri::State<'_, AppDatabase>,
    task_id: i32,
) -> Result<IntegrationPlan, DbError> {
    work_task_integration_plan_core(&db, task_id).await
}
#[cfg(feature = "tauri-runtime")]
#[tauri::command]
pub async fn specos_work_task_integration_refresh(
    db: tauri::State<'_, AppDatabase>,
    task_id: i32,
) -> Result<IntegrationPlan, DbError> {
    work_task_integration_refresh_core(&db, task_id).await
}
