//! Shared Tauri/Axum commands for the Bugrail Code Intelligence module.
//!
//! Every write-side operation (install / index / enable / rebuild / binary
//! override / Graph UI) lives here and is reachable only from the Bugrail UI
//! or backend lifecycle — agents never get these commands. Their only access
//! to the index is the read-only `codebase_*` companion tool set, which the
//! listener binds to the caller's working directory.

use std::path::Path;

use serde::Serialize;

use crate::code_intelligence::{self as ci, manifest};
use crate::db::error::DbError;
use crate::db::AppDatabase;
use crate::models::ContextProviderConfig;

/// Stable provider id used in `.codeg/context.yaml`.
pub const PROVIDER_ID: &str = "codebase";
const PROVIDER_KIND: &str = "code-intelligence";
const PROVIDER_CAPABILITIES: [&str; 5] = [
    "code.search",
    "code.trace",
    "code.architecture",
    "code.impact",
    "code.coverage",
];

fn ci_err(e: ci::CodeIntelError) -> DbError {
    match e {
        ci::CodeIntelError::NotFound(msg) => DbError::NotFound(msg),
        other => DbError::Validation(other.to_string()),
    }
}

fn runtime_or_disabled() -> Result<&'static ci::CodeIntelRuntime, DbError> {
    ci::runtime().ok_or_else(|| {
        DbError::Validation("Code Intelligence is not available in this process".into())
    })
}

/// State view model for the Context page's Codebase section.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeIntelState {
    /// `false` when the runtime failed to initialize at startup — the whole
    /// section renders as unavailable.
    pub runtime_available: bool,
    /// Binary install state; `None` when it cannot be resolved right now.
    pub install: Option<ci::InstallState>,
    /// Index state bound to this folder's project root (`phase:
    /// "not_indexed"` when nothing covers it).
    pub project: ci::ProjectStatus,
    /// Every registered index (base repos + WorkTask worktrees).
    pub records: Vec<ci::registry::ProjectRecord>,
}

pub async fn get_state_core(db: &AppDatabase, folder_id: i32) -> Result<CodeIntelState, DbError> {
    let root = crate::specos_control::project_root(&db.conn, folder_id).await?;
    let Some(rt) = ci::runtime() else {
        return Ok(CodeIntelState {
            runtime_available: false,
            install: None,
            project: ci::ProjectStatus::unbound(),
            records: Vec::new(),
        });
    };
    let install = rt.install_state().await.ok();
    let project = rt.status(&root).await.map_err(ci_err)?;
    let records = rt.registry().all();
    Ok(CodeIntelState {
        runtime_available: true,
        install,
        project,
        records,
    })
}

/// Install (or re-verify) the managed pinned adapter binary. Never touches
/// an override — overrides are fixed via `set_binary_override`.
pub async fn install_core() -> Result<ci::InstallState, DbError> {
    runtime_or_disabled()?.install().await.map_err(ci_err)
}

/// Enable or disable Code Intelligence for a project folder.
///
/// Enabling indexes the repository when there is no record yet (this is the
/// only UI path that creates one) and joins the `codebase` provider into
/// `.codeg/context.yaml` + the default loadout so non-MCP agents get the
/// summary item. Disabling only toggles the registry record — the provider
/// entry and index data stay put.
pub async fn set_enabled_core(
    db: &AppDatabase,
    folder_id: i32,
    enabled: bool,
) -> Result<CodeIntelState, DbError> {
    let root = crate::specos_control::project_root(&db.conn, folder_id).await?;
    let rt = runtime_or_disabled()?;
    let canonical = ci::canonicalize_dir(&root).map_err(ci_err)?;
    if enabled {
        if rt.registry().get(&canonical).is_none() {
            rt.enable_project(&root, false, None)
                .await
                .map_err(ci_err)?;
        } else {
            rt.set_enabled(&canonical, true).map_err(ci_err)?;
        }
        join_context_provider(&root)?;
    } else {
        rt.set_enabled(&canonical, false).map_err(ci_err)?;
    }
    get_state_core(db, folder_id).await
}

/// Force a full re-index of an already-enabled project.
pub async fn reindex_core(db: &AppDatabase, folder_id: i32) -> Result<CodeIntelState, DbError> {
    let root = crate::specos_control::project_root(&db.conn, folder_id).await?;
    runtime_or_disabled()?
        .sync(&root, true)
        .await
        .map_err(ci_err)?;
    get_state_core(db, folder_id).await
}

/// Point the module at a user-provided binary (`Some`) or clear the override
/// back to the managed pinned binary (`None`).
pub async fn set_binary_override_core(path: Option<String>) -> Result<ci::InstallState, DbError> {
    runtime_or_disabled()?
        .set_binary_override(path)
        .await
        .map_err(ci_err)
}

/// Ensure the `codebase` provider exists in `.codeg/context.yaml` and the
/// default loadout selects it. Idempotent; never touches other providers.
fn join_context_provider(root: &Path) -> Result<(), DbError> {
    let mut config = crate::specos_control::load_context(root)?;
    if !config.providers.iter().any(|p| p.id == PROVIDER_ID) {
        config.providers.push(ContextProviderConfig {
            id: PROVIDER_ID.into(),
            kind: PROVIDER_KIND.into(),
            adapter: Some(manifest::ADAPTER_ID.into()),
            endpoint: None,
            secret_env: None,
            enabled: true,
            required: false,
            capabilities: PROVIDER_CAPABILITIES
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            ..Default::default()
        });
    }
    for loadout in config
        .loadouts
        .iter_mut()
        .filter(|l| l.id == config.default_loadout_id)
    {
        if !loadout.provider_ids.iter().any(|id| id == PROVIDER_ID) {
            loadout.provider_ids.push(PROVIDER_ID.into());
        }
    }
    crate::specos_control::save_context(root, config)?;
    Ok(())
}

// ── Tauri command wrappers (desktop only) ───────────────────────────────

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
pub async fn code_intelligence_get_state(
    db: tauri::State<'_, AppDatabase>,
    folder_id: i32,
) -> Result<CodeIntelState, DbError> {
    get_state_core(&db, folder_id).await
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
pub async fn code_intelligence_install() -> Result<ci::InstallState, DbError> {
    install_core().await
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
pub async fn code_intelligence_set_enabled(
    db: tauri::State<'_, AppDatabase>,
    folder_id: i32,
    enabled: bool,
) -> Result<CodeIntelState, DbError> {
    set_enabled_core(&db, folder_id, enabled).await
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
pub async fn code_intelligence_reindex(
    db: tauri::State<'_, AppDatabase>,
    folder_id: i32,
) -> Result<CodeIntelState, DbError> {
    reindex_core(&db, folder_id).await
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
pub async fn code_intelligence_set_binary_override(
    path: Option<String>,
) -> Result<ci::InstallState, DbError> {
    set_binary_override_core(path).await
}

/// Desktop-only: bring up the upstream Graph UI on a loopback port and open
/// it in the default browser. Server mode has no local browser to open and
/// is refused by the handler instead.
#[cfg(feature = "tauri-runtime")]
#[tauri::command]
pub async fn code_intelligence_open_graph(app: tauri::AppHandle) -> Result<String, DbError> {
    use tauri_plugin_opener::OpenerExt;
    let url = open_graph_core().await?;
    let _ = app.opener().open_url(url.clone(), None::<&str>);
    Ok(url)
}

/// Shared part of the Graph UI command (also used by the desktop wrapper).
#[cfg(feature = "tauri-runtime")]
pub async fn open_graph_core() -> Result<String, DbError> {
    let rt = runtime_or_disabled()?;
    rt.enable_graph_ui().await.map_err(ci_err)
}
