use axum::{extract::Extension, Json};
use serde::Deserialize;
use std::sync::Arc;

use crate::app_error::AppCommandError;
#[cfg(not(feature = "tauri-runtime"))]
use crate::app_error::AppErrorCode;
use crate::app_state::AppState;
use crate::code_intelligence::InstallState;
use crate::commands::code_intelligence as core;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderParams {
    pub folder_id: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetEnabledParams {
    pub folder_id: i32,
    pub enabled: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinaryOverrideParams {
    pub path: Option<String>,
}

pub async fn get_state(
    Extension(s): Extension<Arc<AppState>>,
    Json(p): Json<FolderParams>,
) -> Result<Json<core::CodeIntelState>, AppCommandError> {
    Ok(Json(core::get_state_core(&s.db, p.folder_id).await?))
}

pub async fn install(
    Extension(_s): Extension<Arc<AppState>>,
) -> Result<Json<InstallState>, AppCommandError> {
    Ok(Json(core::install_core().await?))
}

pub async fn set_enabled(
    Extension(s): Extension<Arc<AppState>>,
    Json(p): Json<SetEnabledParams>,
) -> Result<Json<core::CodeIntelState>, AppCommandError> {
    Ok(Json(
        core::set_enabled_core(&s.db, p.folder_id, p.enabled).await?,
    ))
}

pub async fn reindex(
    Extension(s): Extension<Arc<AppState>>,
    Json(p): Json<FolderParams>,
) -> Result<Json<core::CodeIntelState>, AppCommandError> {
    Ok(Json(core::reindex_core(&s.db, p.folder_id).await?))
}

pub async fn set_binary_override(
    Extension(_s): Extension<Arc<AppState>>,
    Json(p): Json<BinaryOverrideParams>,
) -> Result<Json<InstallState>, AppCommandError> {
    Ok(Json(core::set_binary_override_core(p.path).await?))
}

/// The upstream Graph UI runs on the host's loopback and is opened in a
/// local browser — that only exists in desktop mode. Server mode refuses
/// (v1: no remote Graph).
pub async fn open_graph(
    Extension(_s): Extension<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppCommandError> {
    #[cfg(feature = "tauri-runtime")]
    {
        let url = core::open_graph_core().await?;
        Ok(Json(serde_json::json!({ "url": url })))
    }
    #[cfg(not(feature = "tauri-runtime"))]
    {
        Err(AppCommandError::new(
            AppErrorCode::PermissionDenied,
            "the Code Intelligence Graph UI is only available in the desktop app",
        ))
    }
}
