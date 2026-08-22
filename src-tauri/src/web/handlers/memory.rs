//! Axum handlers for the Memory Provider operations (issue-080).
//!
//! Thin JSON wrappers over `crate::commands::memory` core functions; the
//! JSON shapes are identical to the Tauri command returns so transport
//! parity holds.

use axum::{extract::Extension, Json};
use serde::Deserialize;
use std::sync::Arc;

use crate::app_error::AppCommandError;
use crate::app_state::AppState;
use crate::commands::memory as core;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderParams {
    pub folder_id: i32,
    pub provider_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryListParams {
    pub folder_id: i32,
    #[serde(default)]
    pub task_id: Option<i32>,
    #[serde(default)]
    pub limit: Option<u32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryRetryParams {
    #[serde(default)]
    pub delivery_id: Option<i32>,
    #[serde(default)]
    pub provider_id: Option<String>,
    #[serde(default)]
    pub task_id: Option<i32>,
    #[serde(default)]
    pub run_seq: Option<i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecallPreviewParams {
    pub folder_id: i32,
    pub provider_id: String,
    pub query: String,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub include_core: Option<bool>,
}

pub async fn provider_test(
    Extension(s): Extension<Arc<AppState>>,
    Json(p): Json<ProviderParams>,
) -> Result<Json<core::MemoryProviderTestResult>, AppCommandError> {
    Ok(Json(
        core::memory_provider_test_core(&s.db, &s.memory_service, p.folder_id, &p.provider_id)
            .await?,
    ))
}

pub async fn delivery_list(
    Extension(s): Extension<Arc<AppState>>,
    Json(p): Json<DeliveryListParams>,
) -> Result<Json<Vec<core::MemoryDeliveryInfo>>, AppCommandError> {
    Ok(Json(
        core::memory_delivery_list_core(&s.db, p.folder_id, p.task_id, p.limit).await?,
    ))
}

pub async fn delivery_retry(
    Extension(s): Extension<Arc<AppState>>,
    Json(p): Json<DeliveryRetryParams>,
) -> Result<Json<core::MemoryDeliveryInfo>, AppCommandError> {
    Ok(Json(
        core::memory_delivery_retry_core(
            &s.db,
            p.delivery_id,
            p.provider_id.as_deref(),
            p.task_id,
            p.run_seq,
        )
        .await?,
    ))
}

pub async fn recall_preview(
    Extension(s): Extension<Arc<AppState>>,
    Json(p): Json<RecallPreviewParams>,
) -> Result<Json<core::MemoryRecallPreview>, AppCommandError> {
    Ok(Json(
        core::memory_recall_preview_core(
            &s.db,
            &s.memory_service,
            p.folder_id,
            &p.provider_id,
            &p.query,
            p.limit,
            p.include_core,
        )
        .await?,
    ))
}
