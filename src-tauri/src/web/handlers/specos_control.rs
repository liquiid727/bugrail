use axum::{extract::Extension, Json};
use serde::Deserialize;
use std::sync::Arc;

use crate::app_error::AppCommandError;
use crate::app_state::AppState;
use crate::commands::specos_control as core;
use crate::models::*;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderParams {
    pub folder_id: i32,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSaveParams {
    pub folder_id: i32,
    pub catalog: AgentCatalog,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamSaveParams {
    pub folder_id: i32,
    pub catalog: TeamCatalog,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamStartParams {
    pub folder_id: i32,
    pub workflow_id: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamControlParams {
    pub run_id: String,
    pub action: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextSaveParams {
    pub folder_id: i32,
    pub config: ContextConfig,
}
#[derive(Deserialize)]
pub struct StringIdParams {
    pub id: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskParams {
    pub task_id: i32,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandoffGetParams {
    pub task_id: i32,
    #[serde(default)]
    pub run_seq: Option<i32>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandoffSaveParams {
    pub task_id: i32,
    pub draft: WorkTaskHandoffDraft,
}

pub async fn agent_get(
    Extension(s): Extension<Arc<AppState>>,
    Json(p): Json<FolderParams>,
) -> Result<Json<AgentCatalog>, AppCommandError> {
    Ok(Json(
        core::agent_catalog_get_core(&s.db, p.folder_id).await?,
    ))
}
pub async fn agent_save(
    Extension(s): Extension<Arc<AppState>>,
    Json(p): Json<AgentSaveParams>,
) -> Result<Json<AgentCatalog>, AppCommandError> {
    Ok(Json(
        core::agent_catalog_save_core(&s.db, p.folder_id, p.catalog).await?,
    ))
}
pub async fn team_get(
    Extension(s): Extension<Arc<AppState>>,
    Json(p): Json<FolderParams>,
) -> Result<Json<TeamCatalog>, AppCommandError> {
    Ok(Json(core::team_catalog_get_core(&s.db, p.folder_id).await?))
}
pub async fn team_save(
    Extension(s): Extension<Arc<AppState>>,
    Json(p): Json<TeamSaveParams>,
) -> Result<Json<TeamCatalog>, AppCommandError> {
    Ok(Json(
        core::team_catalog_save_core(&s.db, p.folder_id, p.catalog).await?,
    ))
}
pub async fn team_run_list(
    Extension(s): Extension<Arc<AppState>>,
    Json(p): Json<FolderParams>,
) -> Result<Json<Vec<TeamRunInfo>>, AppCommandError> {
    Ok(Json(core::team_run_list_core(&s.db, p.folder_id).await?))
}
pub async fn team_run_start(
    Extension(s): Extension<Arc<AppState>>,
    Json(p): Json<TeamStartParams>,
) -> Result<Json<TeamRunInfo>, AppCommandError> {
    Ok(Json(
        core::team_run_start_core(&s.emitter, &s.db, p.folder_id, p.workflow_id).await?,
    ))
}
pub async fn team_run_control(
    Extension(s): Extension<Arc<AppState>>,
    Json(p): Json<TeamControlParams>,
) -> Result<Json<()>, AppCommandError> {
    core::team_run_control_core(&s.db, p.run_id, p.action).await?;
    Ok(Json(()))
}
pub async fn context_get(
    Extension(s): Extension<Arc<AppState>>,
    Json(p): Json<FolderParams>,
) -> Result<Json<ContextConfig>, AppCommandError> {
    Ok(Json(
        core::context_config_get_core(&s.db, p.folder_id).await?,
    ))
}
pub async fn context_save(
    Extension(s): Extension<Arc<AppState>>,
    Json(p): Json<ContextSaveParams>,
) -> Result<Json<ContextConfig>, AppCommandError> {
    Ok(Json(
        core::context_config_save_core(&s.db, p.folder_id, p.config).await?,
    ))
}
pub async fn context_overview(
    Extension(s): Extension<Arc<AppState>>,
    Json(p): Json<FolderParams>,
) -> Result<Json<ContextOverview>, AppCommandError> {
    Ok(Json(
        core::context_overview_core(&s.db, &s.memory_service, p.folder_id).await?,
    ))
}
pub async fn context_package_get(
    Extension(s): Extension<Arc<AppState>>,
    Json(p): Json<StringIdParams>,
) -> Result<Json<ContextPackageInfo>, AppCommandError> {
    Ok(Json(core::context_package_get_core(&s.db, p.id).await?))
}
pub async fn task_runs(
    Extension(s): Extension<Arc<AppState>>,
    Json(p): Json<TaskParams>,
) -> Result<Json<Vec<WorkTaskRunInfo>>, AppCommandError> {
    Ok(Json(core::work_task_runs_core(&s.db, p.task_id).await?))
}
pub async fn task_dependencies(
    Extension(s): Extension<Arc<AppState>>,
    Json(p): Json<TaskParams>,
) -> Result<Json<Vec<WorkTaskDependencyInfo>>, AppCommandError> {
    Ok(Json(
        core::work_task_dependencies_core(&s.db, p.task_id).await?,
    ))
}
pub async fn handoff_get(
    Extension(s): Extension<Arc<AppState>>,
    Json(p): Json<HandoffGetParams>,
) -> Result<Json<Option<WorkTaskHandoffInfo>>, AppCommandError> {
    Ok(Json(
        core::work_task_handoff_get_core(&s.db, p.task_id, p.run_seq).await?,
    ))
}
pub async fn handoff_save(
    Extension(s): Extension<Arc<AppState>>,
    Json(p): Json<HandoffSaveParams>,
) -> Result<Json<WorkTaskHandoffInfo>, AppCommandError> {
    Ok(Json(
        core::work_task_handoff_save_core(&s.db, p.task_id, p.draft).await?,
    ))
}
pub async fn integration_plan(
    Extension(s): Extension<Arc<AppState>>,
    Json(p): Json<TaskParams>,
) -> Result<Json<IntegrationPlan>, AppCommandError> {
    Ok(Json(
        core::work_task_integration_plan_core(&s.db, p.task_id).await?,
    ))
}
pub async fn integration_refresh(
    Extension(s): Extension<Arc<AppState>>,
    Json(p): Json<TaskParams>,
) -> Result<Json<IntegrationPlan>, AppCommandError> {
    Ok(Json(
        core::work_task_integration_refresh_core(&s.db, p.task_id).await?,
    ))
}
