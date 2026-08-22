//! Bugrail Code Intelligence module — the only door between agents/UI and
//! the `codebase-memory-mcp` adapter.
//!
//! Data flow (approved plan):
//!
//! ```text
//! Repository/Worktree → Code Intelligence Module → CodebaseMemory Adapter
//!   → stdio MCP → codebase-memory-mcp → SQLite / Watcher / Graph UI
//! ```
//!
//! Guarantees this module enforces:
//!
//! - Agents never choose projects: every query is bound to the connected
//!   working dir (exact canonical match, else nearest indexed ancestor).
//! - Indexing, rebuilding and cleanup happen only through Bugrail (UI
//!   commands / WorkTask lifecycle); agent-facing tools are read-only.
//! - Confinement is two-layered: every spawned adapter session carries
//!   `CBM_ALLOWED_ROOT` for the one repository it serves, and before any
//!   indexing call Bugrail records the root in the shared store's
//!   `allowed_roots` file (`allow-root`). All sessions share one upstream
//!   daemon per cache dir, and that daemon only honours the env var until
//!   the first root is recorded — afterwards indexing is confined to the
//!   recorded roots, i.e. exactly the paths Bugrail enabled. Everything is
//!   stored under Bugrail's private cache root
//!   `<data_dir>/code-intelligence/codebase-memory-mcp`.
//! - The binary is the pinned v0.10.6 release (or a user override of the
//!   same major.minor); incompatible versions are refused, never silently
//!   substituted.

pub mod adapter;
pub mod binary_cache;
pub mod manifest;
pub mod registry;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::Mutex;

use adapter::{SessionManager, INDEX_TIMEOUT, QUERY_TIMEOUT};
use manifest::BINARY_OVERRIDE_ENV;
use registry::{ProjectRecord, Registry};

use crate::app_error::AppCommandError;

/// Maximum bytes of tool text forwarded to agents/UI; longer answers are
/// truncated with a visible marker.
pub const MAX_RESPONSE_BYTES: usize = 256 * 1024;
/// Architecture text kept inside Context Pack summaries.
pub const MAX_SUMMARY_ARCH_BYTES: usize = 32 * 1024;

// ─── errors ─────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum CodeIntelError {
    #[error("io error: {0}")]
    Io(String),
    #[error("download failed: {0}")]
    Download(String),
    #[error("archive checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },
    #[error("archive extraction failed: {0}")]
    Extract(String),
    #[error("failed to spawn adapter: {0}")]
    Spawn(String),
    #[error("adapter error: {0}")]
    Adapter(String),
    #[error("timeout: {0}")]
    Timeout(String),
    #[error("unsupported platform: {0}")]
    UnsupportedPlatform(String),
    #[error("binary override rejected: {0}")]
    BinaryInvalid(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("{0}")]
    Internal(String),
}

impl CodeIntelError {
    pub fn io(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<CodeIntelError> for AppCommandError {
    fn from(err: CodeIntelError) -> Self {
        match &err {
            CodeIntelError::NotFound(_) => AppCommandError::not_found(err.to_string()),
            CodeIntelError::Download(_) => AppCommandError::network(err.to_string()),
            CodeIntelError::BinaryInvalid(_)
            | CodeIntelError::UnsupportedPlatform(_)
            | CodeIntelError::Extract(_)
            | CodeIntelError::ChecksumMismatch { .. } => {
                AppCommandError::invalid_input(err.to_string())
            }
            CodeIntelError::Io(_) => AppCommandError::io_error(err.to_string()),
            CodeIntelError::Spawn(_)
            | CodeIntelError::Adapter(_)
            | CodeIntelError::Timeout(_)
            | CodeIntelError::Internal(_) => {
                AppCommandError::task_execution_failed(err.to_string())
            }
        }
    }
}

// ─── layout ─────────────────────────────────────────────────────────────

/// Unique Bugrail cache root: `<data_dir>/code-intelligence/codebase-memory-mcp`.
pub fn code_intel_root(data_dir: &Path) -> PathBuf {
    data_dir
        .join("code-intelligence")
        .join(manifest::ADAPTER_ID)
}

/// Shared upstream cache dir (`CBM_CACHE_DIR` for every session).
pub fn store_dir(root: &Path) -> PathBuf {
    root.join("store")
}

// ─── binary resolution ──────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BinarySource {
    /// Absolute path from `CODEG_CBM_BIN`.
    EnvOverride,
    /// Absolute path from the Bugrail preferences file.
    PreferencesOverride,
    /// The managed pinned-version binary in the cache root.
    Managed,
}

#[derive(Debug, Clone)]
pub struct ResolvedBinary {
    pub path: PathBuf,
    pub source: BinarySource,
    /// Trimmed `--version` output of the binary.
    pub version: String,
}

/// Which tier wins, given the two override inputs. Pure helper so the
/// priority rule is unit-testable without touching the environment.
pub fn pick_override<'a>(
    env_override: Option<&'a str>,
    preferences_override: Option<&'a str>,
) -> Option<(BinarySource, &'a str)> {
    if let Some(path) = env_override.map(str::trim).filter(|p| !p.is_empty()) {
        return Some((BinarySource::EnvOverride, path));
    }
    if let Some(path) = preferences_override
        .map(str::trim)
        .filter(|p| !p.is_empty())
    {
        return Some((BinarySource::PreferencesOverride, path));
    }
    None
}

/// Run `binary --version` and require a compatible (same major.minor as the
/// pinned version) report. Refusing here is deliberate: an override that is
/// silently replaced by the managed copy would surprise users more than a
/// loud error.
async fn probe_version(binary: &Path) -> Result<String, CodeIntelError> {
    if !binary.is_file() {
        return Err(CodeIntelError::BinaryInvalid(format!(
            "{} does not exist or is not a file",
            binary.display()
        )));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let meta = std::fs::metadata(binary).map_err(CodeIntelError::io)?;
        if meta.permissions().mode() & 0o111 == 0 {
            return Err(CodeIntelError::BinaryInvalid(format!(
                "{} is not executable",
                binary.display()
            )));
        }
    }
    let output = tokio::time::timeout(
        Duration::from_secs(15),
        tokio::process::Command::new(binary)
            .arg("--version")
            .output(),
    )
    .await
    .map_err(|_| {
        CodeIntelError::BinaryInvalid(format!("{} --version timed out", binary.display()))
    })?
    .map_err(|err| {
        CodeIntelError::BinaryInvalid(format!("{} --version failed: {err}", binary.display()))
    })?;
    let mut combined = String::from_utf8_lossy(&output.stdout).to_string();
    combined.push_str(&String::from_utf8_lossy(&output.stderr));
    let combined = combined.trim().to_string();
    if !manifest::is_compatible_version(&combined) {
        return Err(CodeIntelError::BinaryInvalid(format!(
            "{} reports an incompatible version: '{combined}' (need {})",
            binary.display(),
            manifest::PINNED_VERSION
        )));
    }
    Ok(combined)
}

// ─── query model ────────────────────────────────────────────────────────

/// Closed set of read-only queries Bugrail exposes. Agents and the UI can
/// only pick one of these; write/indexing tools of the upstream server are
/// never reachable through this module's query path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CodeQuery {
    Status,
    Search,
    Trace,
    Query,
    Architecture,
    Impact,
    Snippet,
    Coverage,
    TextSearch,
}

impl CodeQuery {
    pub fn from_tool_name(name: &str) -> Option<CodeQuery> {
        match name {
            "codebase_status" | "status" => Some(CodeQuery::Status),
            "codebase_search" | "search" => Some(CodeQuery::Search),
            "codebase_trace" | "trace" => Some(CodeQuery::Trace),
            "codebase_query" | "query" => Some(CodeQuery::Query),
            "codebase_architecture" | "architecture" => Some(CodeQuery::Architecture),
            "codebase_impact" | "impact" => Some(CodeQuery::Impact),
            "codebase_snippet" | "snippet" => Some(CodeQuery::Snippet),
            "codebase_coverage" | "coverage" => Some(CodeQuery::Coverage),
            "codebase_text_search" | "text_search" => Some(CodeQuery::TextSearch),
            _ => None,
        }
    }

    /// The upstream MCP tool each Bugrail query maps onto.
    pub fn upstream_tool(self) -> &'static str {
        match self {
            CodeQuery::Status => "index_status",
            CodeQuery::Search => "search_graph",
            CodeQuery::Trace => "trace_path",
            CodeQuery::Query => "query_graph",
            CodeQuery::Architecture => "get_architecture",
            CodeQuery::Impact => "detect_changes",
            CodeQuery::Snippet => "get_code_snippet",
            CodeQuery::Coverage => "check_index_coverage",
            CodeQuery::TextSearch => "search_code",
        }
    }
}

/// Clamp agent-supplied pagination/depth/size arguments to Bugrail caps.
/// Absent limits get an explicit bounded default so a missing argument can
/// never select unbounded output.
pub fn enforce_caps(query: CodeQuery, mut args: Value) -> Value {
    if !args.is_object() {
        args = json!({});
    }
    fn clamp_int(
        args: &mut Value,
        key: &str,
        min: i64,
        max: i64,
        default_when_absent: Option<i64>,
    ) {
        let obj = args.as_object_mut().unwrap();
        match obj.get(key).and_then(Value::as_i64) {
            Some(v) => {
                obj.insert(key.into(), json!(v.clamp(min, max)));
            }
            None => {
                if let Some(d) = default_when_absent {
                    obj.insert(key.into(), json!(d));
                }
            }
        }
    }
    fn cap_array_len(args: &mut Value, key: &str, max: usize) {
        let obj = args.as_object_mut().unwrap();
        if let Some(arr) = obj.get_mut(key).and_then(Value::as_array_mut) {
            arr.truncate(max);
        }
    }
    match query {
        CodeQuery::Search => clamp_int(&mut args, "limit", 1, 50, Some(20)),
        CodeQuery::Trace => clamp_int(&mut args, "depth", 1, 5, None),
        CodeQuery::Query => clamp_int(&mut args, "max_rows", 1, 200, Some(100)),
        CodeQuery::Impact => {
            clamp_int(&mut args, "limit", 1, 200, Some(50));
            clamp_int(&mut args, "depth", 1, 10, None);
        }
        CodeQuery::TextSearch => clamp_int(&mut args, "limit", 1, 100, Some(50)),
        CodeQuery::Coverage => {
            cap_array_len(&mut args, "paths", 128);
            cap_array_len(&mut args, "scopes", 32);
        }
        CodeQuery::Status | CodeQuery::Architecture | CodeQuery::Snippet => {}
    }
    args
}

/// Truncate tool text to `max` bytes on a char boundary, appending a marker
/// so consumers know the answer was cut.
pub fn truncate_text(text: String, max: usize) -> String {
    if text.len() <= max {
        return text;
    }
    let mut cut = max;
    while cut > 0 && !text.is_char_boundary(cut) {
        cut -= 1;
    }
    let mut out = text[..cut].to_string();
    out.push_str("\n…[truncated by Bugrail Code Intelligence]");
    out
}

/// Derive the upstream project key from a canonical repo path. Upstream
/// replaces path separators with `-` (`/private/tmp/repo` →
/// `private-tmp-repo`); verified against the pinned binary.
pub fn derive_project_key(canonical_path: &str) -> String {
    let mut key: String = canonical_path
        .chars()
        .map(|c| match c {
            '/' | '\\' => '-',
            ':' => '-',
            other => other,
        })
        .collect();
    while key.starts_with('-') {
        key.remove(0);
    }
    while key.ends_with('-') {
        key.pop();
    }
    key
}

/// Canonicalize a directory path to the string form used as registry key
/// and `CBM_ALLOWED_ROOT`.
pub fn canonicalize_dir(path: &Path) -> Result<String, CodeIntelError> {
    let canonical = std::fs::canonicalize(path)
        .map_err(|err| CodeIntelError::NotFound(format!("{}: {err}", path.display())))?;
    if !canonical.is_dir() {
        return Err(CodeIntelError::NotFound(format!(
            "{} is not a directory",
            canonical.display()
        )));
    }
    let mut s = canonical.to_string_lossy().to_string();
    while s.len() > 1 && s.ends_with('/') {
        s.pop();
    }
    Ok(s)
}

// ─── status / summary models ────────────────────────────────────────────

/// Index state for one repository as shown in the UI / state API.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectStatus {
    /// Whether any enabled index is bound to the queried directory.
    pub bound: bool,
    pub project: Option<String>,
    pub repo_path: Option<String>,
    pub worktree: bool,
    pub task_id: Option<i32>,
    pub enabled: bool,
    /// Index phase reported by `index_status` ("ready", "indexing", …);
    /// "unknown" when the adapter could not answer.
    pub phase: String,
    pub revision: Option<String>,
    pub node_count: Option<u64>,
    pub edge_count: Option<u64>,
    pub file_count: Option<u64>,
    pub indexed_at: Option<String>,
    pub last_synced_at: Option<String>,
}

impl ProjectStatus {
    pub fn unbound() -> Self {
        Self {
            bound: false,
            project: None,
            repo_path: None,
            worktree: false,
            task_id: None,
            enabled: false,
            phase: "not_indexed".into(),
            revision: None,
            node_count: None,
            edge_count: None,
            file_count: None,
            indexed_at: None,
            last_synced_at: None,
        }
    }
}

/// Result of a bound query.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryOutcome {
    pub text: String,
    pub is_error: bool,
    /// The project key the query was bound to (agents never choose this).
    pub project: String,
}

/// Install/binary state for the UI.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallState {
    pub pinned_version: &'static str,
    pub installed: bool,
    pub binary_path: Option<String>,
    pub source: Option<BinarySource>,
    pub reported_version: Option<String>,
    pub cache_root: String,
}

// ─── runtime ────────────────────────────────────────────────────────────

static RUNTIME: OnceLock<CodeIntelRuntime> = OnceLock::new();

/// Initialize the process-global runtime once, at startup (desktop setup /
/// server bootstrap). Synchronous: only creates directories and loads the
/// registry — no download, no process spawn.
pub fn init_runtime(data_dir: &Path) -> Result<&'static CodeIntelRuntime, CodeIntelError> {
    if let Some(existing) = RUNTIME.get() {
        return Ok(existing);
    }
    let root = code_intel_root(data_dir);
    std::fs::create_dir_all(store_dir(&root)).map_err(CodeIntelError::io)?;
    std::fs::create_dir_all(root.join("state")).map_err(CodeIntelError::io)?;
    let registry = Registry::load(&root)?;
    let runtime = CodeIntelRuntime {
        root,
        registry,
        binary: Mutex::new(None),
        session_manager: Mutex::new(None),
    };
    match RUNTIME.set(runtime) {
        Ok(()) => Ok(RUNTIME.get().unwrap()),
        Err(_) => Ok(RUNTIME.get().unwrap()),
    }
}

/// The initialized runtime, when Code Intelligence is available in this
/// process. `None` before init (or in test harnesses that skip it) — all
/// call sites must treat that as "feature disabled".
pub fn runtime() -> Option<&'static CodeIntelRuntime> {
    RUNTIME.get()
}

pub struct CodeIntelRuntime {
    root: PathBuf,
    registry: Registry,
    binary: Mutex<Option<ResolvedBinary>>,
    session_manager: Mutex<Option<ArcSessionManager>>,
}

type ArcSessionManager = std::sync::Arc<SessionManager>;

impl CodeIntelRuntime {
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    // ── binary lifecycle ──

    /// Resolve the binary without downloading: env override → preferences
    /// override → managed pinned copy. Overrides are validated (exists,
    /// executable, compatible version) and refused when invalid; the managed
    /// copy missing is reported as `Ok(None)`.
    pub async fn resolve_binary(&self) -> Result<Option<ResolvedBinary>, CodeIntelError> {
        let mut cache = self.binary.lock().await;
        if let Some(resolved) = cache.as_ref() {
            return Ok(Some(resolved.clone()));
        }
        let env_override = std::env::var(BINARY_OVERRIDE_ENV).ok();
        let prefs_override = crate::preferences::load().codebase_memory_mcp_path;
        let resolved = match pick_override(env_override.as_deref(), prefs_override.as_deref()) {
            Some((source, path)) => {
                let path_buf = PathBuf::from(path);
                let version = probe_version(&path_buf).await?;
                ResolvedBinary {
                    path: path_buf,
                    source,
                    version,
                }
            }
            None => match binary_cache::find_installed(&self.root) {
                Some(path) => {
                    let version = probe_version(&path).await?;
                    ResolvedBinary {
                        path,
                        source: BinarySource::Managed,
                        version,
                    }
                }
                None => return Ok(None),
            },
        };
        *cache = Some(resolved.clone());
        Ok(Some(resolved))
    }

    /// Resolve, downloading the managed pinned binary when it is missing.
    /// Never downloads over an override — overrides must be fixed by the
    /// user instead.
    pub async fn ensure_binary(&self) -> Result<ResolvedBinary, CodeIntelError> {
        if let Some(resolved) = self.resolve_binary().await? {
            return Ok(resolved);
        }
        let path = binary_cache::ensure_installed(&self.root).await?;
        let version = probe_version(&path).await?;
        let resolved = ResolvedBinary {
            path,
            source: BinarySource::Managed,
            version,
        };
        *self.binary.lock().await = Some(resolved.clone());
        Ok(resolved)
    }

    /// Explicit install command: ensure the managed binary exists and report
    /// the resulting state.
    pub async fn install(&self) -> Result<InstallState, CodeIntelError> {
        self.ensure_binary().await?;
        self.install_state().await
    }

    pub async fn install_state(&self) -> Result<InstallState, CodeIntelError> {
        let resolved = self.resolve_binary().await?;
        Ok(InstallState {
            pinned_version: manifest::PINNED_VERSION,
            installed: resolved.is_some(),
            binary_path: resolved.as_ref().map(|r| r.path.display().to_string()),
            source: resolved.as_ref().map(|r| r.source),
            reported_version: resolved.map(|r| r.version),
            cache_root: self.root.display().to_string(),
        })
    }

    /// Apply a new preferences-file override (or clear it with `None`).
    /// Drops the cached resolution and all live sessions so the next call
    /// re-resolves.
    pub async fn set_binary_override(
        &self,
        path: Option<String>,
    ) -> Result<InstallState, CodeIntelError> {
        let mut prefs = crate::preferences::load();
        prefs.codebase_memory_mcp_path = path.filter(|p| !p.trim().is_empty());
        crate::preferences::save(&prefs).map_err(CodeIntelError::io)?;
        self.reset_binary_cache().await;
        // Surface validation errors (invalid override) to the caller.
        self.resolve_binary().await?;
        self.install_state().await
    }

    async fn reset_binary_cache(&self) {
        self.binary.lock().await.take();
        if let Some(manager) = self.session_manager.lock().await.take() {
            manager.shutdown().await;
        }
    }

    async fn session_for(
        &self,
        canonical_path: &str,
    ) -> Result<std::sync::Arc<adapter::AdapterSession>, CodeIntelError> {
        // Read paths never trigger a download: a missing binary is reported,
        // and only the lifecycle commands (install/enable/sync) fetch it.
        let Some(resolved) = self.resolve_binary().await? else {
            return Err(CodeIntelError::NotFound(
                "codebase-memory-mcp binary is not installed — install it from the Context page"
                    .into(),
            ));
        };
        let mut guard = self.session_manager.lock().await;
        let needs_new = match guard.as_ref() {
            None => true,
            Some(manager) => manager.binary_path() != resolved.path,
        };
        if needs_new {
            if let Some(old) = guard.take() {
                old.shutdown().await;
            }
            *guard = Some(std::sync::Arc::new(SessionManager::new(
                resolved.path.clone(),
                store_dir(&self.root),
            )));
        }
        let manager = guard.as_ref().unwrap().clone();
        drop(guard);
        manager.session_for(canonical_path).await
    }

    // ── upstream root confinement ──

    /// Record `canonical` in the shared store's `allowed_roots` file so the
    /// upstream daemon accepts `index_repository` for it. All Bugrail
    /// sessions share one daemon per cache dir; that daemon only honours a
    /// session's `CBM_ALLOWED_ROOT` env until the first root is recorded,
    /// then confines indexing to the recorded roots. The daemon re-reads
    /// the file on every indexing attempt, so recording works while a
    /// daemon started for another root is already running.
    async fn allow_root(&self, canonical: &str) -> Result<(), CodeIntelError> {
        let Some(resolved) = self.resolve_binary().await? else {
            return Err(CodeIntelError::NotFound(
                "codebase-memory-mcp binary is not installed — install it from the Context page"
                    .into(),
            ));
        };
        let output = tokio::process::Command::new(&resolved.path)
            .arg("allow-root")
            .arg(canonical)
            .env("CBM_CACHE_DIR", store_dir(&self.root))
            .env("CBM_LOG_LEVEL", "warn")
            .output()
            .await
            .map_err(CodeIntelError::io)?;
        if !output.status.success() {
            let detail = String::from_utf8_lossy(&output.stderr);
            return Err(CodeIntelError::Adapter(truncate_text(
                format!("allow-root refused {canonical}: {detail}"),
                2048,
            )));
        }
        Ok(())
    }

    /// Best-effort counterpart of [`allow_root`]: drop a root from the
    /// store's `allowed_roots` file when its index is deleted, keeping the
    /// confinement surface minimal. The upstream binary has no removal
    /// subcommand; Bugrail owns the cache root, so trimming the file
    /// directly is safe.
    fn disallow_root(&self, canonical: &str) {
        let file = store_dir(&self.root).join("allowed_roots");
        let Ok(content) = std::fs::read_to_string(&file) else {
            return;
        };
        let kept: Vec<&str> = content
            .lines()
            .filter(|line| line.trim() != canonical)
            .collect();
        let mut out = kept.join("\n");
        if !out.is_empty() {
            out.push('\n');
        }
        if content != out {
            let _ = std::fs::write(&file, out);
        }
    }

    // ── index lifecycle (UI / backend only — never agent-callable) ──

    /// Index (or re-index) a repository. The only entry point that creates
    /// an index; `worktree` marks temporary WorkTask indexes.
    pub async fn enable_project(
        &self,
        repo_path: &Path,
        worktree: bool,
        task_id: Option<i32>,
    ) -> Result<ProjectRecord, CodeIntelError> {
        let canonical = canonicalize_dir(repo_path)?;
        self.ensure_binary().await?; // lifecycle commands may download
        self.allow_root(&canonical).await?;
        let session = self.session_for(&canonical).await?;
        let outcome = session
            .call_tool(
                "index_repository",
                json!({ "repo_path": canonical }),
                INDEX_TIMEOUT,
            )
            .await?;
        if outcome.is_error {
            return Err(CodeIntelError::Adapter(truncate_text(outcome.text, 2048)));
        }
        let now = chrono::Utc::now().to_rfc3339();
        let record = ProjectRecord {
            project: derive_project_key(&canonical),
            repo_path: canonical,
            worktree,
            task_id,
            enabled: true,
            indexed_at: now.clone(),
            last_synced_at: Some(now),
            revision: extract_first_str(&outcome.text, &["revision", "indexed_revision", "commit"]),
        };
        self.registry.upsert(record.clone())?;
        Ok(record)
    }

    /// Re-sync an already-enabled repository. `force` requests a full
    /// re-index; otherwise the upstream default (incremental) is used.
    pub async fn sync(
        &self,
        repo_path: &Path,
        force: bool,
    ) -> Result<ProjectRecord, CodeIntelError> {
        let canonical = canonicalize_dir(repo_path)?;
        let mut record = self.registry.get(&canonical).ok_or_else(|| {
            CodeIntelError::NotFound("repository is not enabled — enable it first".into())
        })?;
        self.ensure_binary().await?; // lifecycle commands may download
                                     // Idempotent re-record: covers a store whose allowed_roots file was
                                     // wiped after the repository was enabled.
        self.allow_root(&canonical).await?;
        let session = self.session_for(&canonical).await?;
        let mut args = json!({ "repo_path": canonical });
        if force {
            args["mode"] = json!("full");
        }
        let outcome = session
            .call_tool("index_repository", args, INDEX_TIMEOUT)
            .await?;
        if outcome.is_error {
            return Err(CodeIntelError::Adapter(truncate_text(outcome.text, 2048)));
        }
        record.last_synced_at = Some(chrono::Utc::now().to_rfc3339());
        if let Some(rev) =
            extract_first_str(&outcome.text, &["revision", "indexed_revision", "commit"])
        {
            record.revision = Some(rev);
        }
        self.registry.upsert(record.clone())?;
        Ok(record)
    }

    /// Toggle a record without touching index data.
    pub fn set_enabled(
        &self,
        repo_path: &str,
        enabled: bool,
    ) -> Result<ProjectRecord, CodeIntelError> {
        if !self.registry.set_enabled(repo_path, enabled)? {
            return Err(CodeIntelError::NotFound(
                "no index registered for this path".into(),
            ));
        }
        Ok(self
            .registry
            .get(repo_path)
            .expect("record exists after set_enabled"))
    }

    /// Delete one index (upstream `delete_project`) and forget it. Used for
    /// worktree cleanup; errors from the adapter are logged but the
    /// registry record is always removed — a leftover DB with no record is
    /// invisible to Bugrail, a leftover record with no DB is not.
    pub async fn delete_index(&self, record: &ProjectRecord) {
        match self.session_for(&record.repo_path).await {
            Ok(session) => {
                let result = session
                    .call_tool(
                        "delete_project",
                        json!({ "project": record.project }),
                        QUERY_TIMEOUT,
                    )
                    .await;
                if let Err(err) = result {
                    tracing::warn!(
                        "[CodeIntel] delete_project failed for {}: {err}",
                        record.project
                    );
                }
            }
            Err(err) => {
                tracing::warn!(
                    "[CodeIntel] could not start session to delete {}: {err}",
                    record.project
                );
            }
        }
        if let Some(manager) = self.session_manager.lock().await.as_ref() {
            manager.drop_session(&record.repo_path).await;
        }
        let _ = self.registry.remove(&record.repo_path);
        self.disallow_root(&record.repo_path);
    }

    /// Drop all temporary indexes owned by a WorkTask — called from the
    /// worktree cleanup path. Best-effort per index; returns how many were
    /// processed.
    pub async fn drop_task_worktree_indexes(&self, task_id: i32) -> usize {
        let records = self.registry.worktree_records_for_task(task_id);
        let count = records.len();
        for record in records {
            self.delete_index(&record).await;
        }
        count
    }

    // ── read-only queries (agent + UI path) ──

    /// Bind `working_dir` to an enabled index and run one read-only query.
    /// The `project` argument is always overwritten with the bound key —
    /// callers (including agents) cannot address arbitrary projects.
    pub async fn query(
        &self,
        working_dir: &Path,
        query: CodeQuery,
        arguments: Value,
    ) -> Result<QueryOutcome, CodeIntelError> {
        let canonical = canonicalize_dir(working_dir)?;
        let record = self.registry.resolve(&canonical).ok_or_else(|| {
            CodeIntelError::NotFound("no enabled code index covers this directory".into())
        })?;
        let session = self.session_for(&record.repo_path).await?;
        let mut args = enforce_caps(query, arguments);
        if let Some(obj) = args.as_object_mut() {
            obj.insert("project".into(), json!(record.project));
        }
        let outcome = session
            .call_tool(query.upstream_tool(), args, QUERY_TIMEOUT)
            .await?;
        Ok(QueryOutcome {
            text: truncate_text(outcome.text, MAX_RESPONSE_BYTES),
            is_error: outcome.is_error,
            project: record.project,
        })
    }

    /// Status of the index bound to `working_dir` (best-effort: an adapter
    /// failure yields phase "unknown" rather than an error so the UI keeps
    /// working).
    pub async fn status(&self, working_dir: &Path) -> Result<ProjectStatus, CodeIntelError> {
        let canonical = canonicalize_dir(working_dir)?;
        let Some(record) = self.registry.resolve(&canonical) else {
            return Ok(ProjectStatus::unbound());
        };
        let mut status = ProjectStatus {
            bound: true,
            project: Some(record.project.clone()),
            repo_path: Some(record.repo_path.clone()),
            worktree: record.worktree,
            task_id: record.task_id,
            enabled: record.enabled,
            phase: "unknown".into(),
            revision: record.revision.clone(),
            node_count: None,
            edge_count: None,
            file_count: None,
            indexed_at: Some(record.indexed_at.clone()),
            last_synced_at: record.last_synced_at.clone(),
        };
        let Ok(session) = self.session_for(&record.repo_path).await else {
            return Ok(status);
        };
        let Ok(outcome) = session
            .call_tool(
                "index_status",
                json!({ "project": record.project }),
                QUERY_TIMEOUT,
            )
            .await
        else {
            return Ok(status);
        };
        if let Some(json) = parse_json_lenient(&outcome.text) {
            status.phase =
                extract_first_str_from(&json, &["phase", "status", "state", "index_status"])
                    .unwrap_or_else(|| status.phase.clone());
            if let Some(rev) =
                extract_first_str_from(&json, &["revision", "indexed_revision", "commit"])
            {
                status.revision = Some(rev);
            }
            status.node_count =
                extract_first_u64_from(&json, &["nodes", "node_count", "total_nodes"]);
            status.edge_count =
                extract_first_u64_from(&json, &["edges", "edge_count", "total_edges"]);
            status.file_count = extract_first_u64_from(
                &json,
                &["files", "file_count", "indexed_files", "total_files"],
            );
        }
        Ok(status)
    }

    /// Normalized, bounded summary for Context Packs (non-MCP agents).
    /// `None` when no index covers the directory — degraded, never
    /// blocking.
    pub async fn context_summary(
        &self,
        working_dir: &Path,
    ) -> Result<Option<Value>, CodeIntelError> {
        let canonical = canonicalize_dir(working_dir)?;
        let Some(record) = self.registry.resolve(&canonical) else {
            return Ok(None);
        };
        let Ok(session) = self.session_for(&record.repo_path).await else {
            return Ok(None);
        };

        let architecture = session
            .call_tool(
                "get_architecture",
                json!({ "project": record.project }),
                QUERY_TIMEOUT,
            )
            .await
            .map(|o| truncate_text(o.text, MAX_SUMMARY_ARCH_BYTES))
            .unwrap_or_default();

        let mut revision = record.revision.clone();
        let mut phase = String::from("unknown");
        let mut files: Option<u64> = None;
        if let Ok(status_outcome) = session
            .call_tool(
                "index_status",
                json!({ "project": record.project }),
                QUERY_TIMEOUT,
            )
            .await
        {
            if let Some(json) = parse_json_lenient(&status_outcome.text) {
                if let Some(rev) =
                    extract_first_str_from(&json, &["revision", "indexed_revision", "commit"])
                {
                    revision = Some(rev);
                }
                if let Some(p) = extract_first_str_from(&json, &["phase", "status", "state"]) {
                    phase = p;
                }
                files = extract_first_u64_from(
                    &json,
                    &["files", "file_count", "indexed_files", "total_files"],
                );
            }
        }

        let resolved = self.resolve_binary().await?;
        Ok(Some(json!({
            "schema": "bugrail.code-intelligence.summary",
            "version": 1,
            "adapter": manifest::ADAPTER_ID,
            "adapter_version": resolved.map(|r| r.version),
            "project": record.project,
            "repo_path": record.repo_path,
            "revision": revision,
            "phase": phase,
            "files_indexed": files,
            "architecture": architecture,
            "generated_at": chrono::Utc::now().to_rfc3339(),
        })))
    }

    // ── Graph UI (desktop only; the command layer gates availability) ──

    /// Enable the upstream Graph UI on a fresh loopback port, bounce the
    /// shared daemon so it picks the config up, and wait until it answers
    /// HTTP. Returns the URL for the opener. Requires at least one enabled
    /// index (the daemon only runs while a session lives).
    pub async fn enable_graph_ui(&self) -> Result<String, CodeIntelError> {
        let Some(record) = self.registry.enabled().first().cloned() else {
            return Err(CodeIntelError::NotFound(
                "enable a code index before opening the graph UI".into(),
            ));
        };

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(CodeIntelError::io)?;
        let port = listener.local_addr().map_err(CodeIntelError::io)?.port();
        drop(listener);

        // Merge into the daemon config inside the shared store.
        let config_path = store_dir(&self.root).join("config.json");
        let mut config: Value = std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_else(|| json!({}));
        if let Some(obj) = config.as_object_mut() {
            obj.insert("ui_enabled".into(), json!(true));
            obj.insert("ui_port".into(), json!(port));
        }
        std::fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap())
            .map_err(CodeIntelError::io)?;

        // Bounce: ending every session ends the shared daemon; the next
        // session relaunches it with ui_enabled in effect.
        if let Some(manager) = self.session_manager.lock().await.as_ref() {
            manager.shutdown().await;
        }
        let session = self.session_for(&record.repo_path).await?;
        // The session stays cached inside the SessionManager, keeping the
        // shared daemon (and its Graph UI) alive past this call.
        drop(session);

        let url = format!("http://127.0.0.1:{port}");
        let client = reqwest::Client::new();
        let deadline = std::time::Instant::now() + Duration::from_secs(20);
        loop {
            if std::time::Instant::now() > deadline {
                return Err(CodeIntelError::Timeout(
                    "graph UI did not start within 20s".into(),
                ));
            }
            match client.get(&url).send().await {
                Ok(resp) if resp.status().is_success() || resp.status().is_redirection() => {
                    return Ok(url)
                }
                _ => tokio::time::sleep(Duration::from_millis(500)).await,
            }
        }
    }

    /// End all adapter sessions — app shutdown hook. Ending the last
    /// session also stops the shared daemon and its watchers.
    pub async fn shutdown(&self) {
        if let Some(manager) = self.session_manager.lock().await.as_ref() {
            manager.shutdown().await;
        }
    }
}

// ─── lenient JSON helpers ───────────────────────────────────────────────

fn parse_json_lenient(text: &str) -> Option<Value> {
    if let Ok(v) = serde_json::from_str::<Value>(text) {
        return Some(v);
    }
    // Tools sometimes wrap JSON in prose; take the first {...} span.
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if end <= start {
        return None;
    }
    serde_json::from_str(&text[start..=end]).ok()
}

fn extract_first_str(text: &str, keys: &[&str]) -> Option<String> {
    parse_json_lenient(text).and_then(|json| extract_first_str_from(&json, keys))
}

fn extract_first_str_from(json: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(s) = json.get(key).and_then(Value::as_str) {
            if !s.is_empty() {
                return Some(s.to_string());
            }
        }
    }
    None
}

fn extract_first_u64_from(json: &Value, keys: &[&str]) -> Option<u64> {
    for key in keys {
        if let Some(n) = json.get(key).and_then(Value::as_u64) {
            return Some(n);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_key_derivation_matches_upstream() {
        assert_eq!(
            derive_project_key("/private/tmp/cbm-probe/fixrepo"),
            "private-tmp-cbm-probe-fixrepo"
        );
        assert_eq!(derive_project_key("/repo"), "repo");
        assert_eq!(derive_project_key("/a/b/"), "a-b");
    }

    #[test]
    fn override_priority_env_then_preferences() {
        assert_eq!(
            pick_override(Some("/env/bin"), Some("/prefs/bin")),
            Some((BinarySource::EnvOverride, "/env/bin"))
        );
        assert_eq!(
            pick_override(None, Some("/prefs/bin")),
            Some((BinarySource::PreferencesOverride, "/prefs/bin"))
        );
        assert_eq!(
            pick_override(Some("  "), Some("/prefs/bin")).unwrap().0,
            BinarySource::PreferencesOverride
        );
        assert_eq!(pick_override(None, None), None);
        assert_eq!(pick_override(Some(""), Some("")), None);
    }

    #[test]
    fn caps_clamp_agent_arguments() {
        let args = enforce_caps(
            CodeQuery::Search,
            json!({ "limit": 9999, "offset": 4, "pattern": "x" }),
        );
        assert_eq!(args["limit"], 50); // 9999 clamps to the cap
        let args = enforce_caps(CodeQuery::Search, json!({}));
        assert_eq!(args["limit"], 20); // bounded default when absent

        let args = enforce_caps(
            CodeQuery::Trace,
            json!({ "depth": 99, "function_name": "f" }),
        );
        assert_eq!(args["depth"], 5);

        let args = enforce_caps(
            CodeQuery::Query,
            json!({ "max_rows": 10_000, "query": "MATCH (n) RETURN n" }),
        );
        assert_eq!(args["max_rows"], 200);

        let args = enforce_caps(
            CodeQuery::TextSearch,
            json!({ "pattern": "x", "limit": 10_000 }),
        );
        assert_eq!(args["limit"], 100);

        let paths: Vec<Value> = (0..200).map(|i| json!(format!("f{i}.rs"))).collect();
        let scopes: Vec<Value> = (0..50).map(|i| json!(format!("s{i}"))).collect();
        let args = enforce_caps(
            CodeQuery::Coverage,
            json!({ "paths": paths, "scopes": scopes }),
        );
        assert_eq!(args["paths"].as_array().unwrap().len(), 128);
        assert_eq!(args["scopes"].as_array().unwrap().len(), 32);

        // Non-object arguments become a bounded empty object.
        let args = enforce_caps(CodeQuery::Architecture, json!("garbage"));
        assert!(args.is_object());
    }

    #[test]
    fn query_tool_mapping_is_closed_and_read_only() {
        let all = [
            (CodeQuery::Status, "index_status"),
            (CodeQuery::Search, "search_graph"),
            (CodeQuery::Trace, "trace_path"),
            (CodeQuery::Query, "query_graph"),
            (CodeQuery::Architecture, "get_architecture"),
            (CodeQuery::Impact, "detect_changes"),
            (CodeQuery::Snippet, "get_code_snippet"),
            (CodeQuery::Coverage, "check_index_coverage"),
            (CodeQuery::TextSearch, "search_code"),
        ];
        for (query, upstream) in all {
            assert_eq!(query.upstream_tool(), upstream);
        }
        // Write/admin tools must NOT be reachable through CodeQuery.
        for forbidden in [
            "index_repository",
            "delete_project",
            "manage_adr",
            "ingest_traces",
        ] {
            assert!(CodeQuery::from_tool_name(forbidden).is_none());
            assert!(CodeQuery::from_tool_name(&format!("codebase_{forbidden}")).is_none());
        }
    }

    #[test]
    fn truncation_respects_char_boundaries() {
        let text = "中".repeat(100); // 3 bytes each
        let cut = truncate_text(text.clone(), 10);
        assert!(cut.len() <= 10 + "\n…[truncated by Bugrail Code Intelligence]".len());
        assert!(cut.ends_with("[truncated by Bugrail Code Intelligence]"));
        assert!(text.len() > 10);
        // Short text passes through untouched.
        assert_eq!(truncate_text("ok".into(), 10), "ok");
    }

    #[test]
    fn lenient_json_extraction() {
        assert_eq!(
            extract_first_str(r#"{"phase": "ready", "nodes": 5}"#, &["phase", "status"]),
            Some("ready".into())
        );
        assert_eq!(
            extract_first_str(r#"noise {"status":"indexing"} noise"#, &["phase", "status"]),
            Some("indexing".into())
        );
        assert_eq!(extract_first_str("not json", &["phase"]), None);
        let json = parse_json_lenient(r#"{"files": 42}"#).unwrap();
        assert_eq!(
            extract_first_u64_from(&json, &["files", "file_count"]),
            Some(42)
        );
    }

    #[test]
    fn canonicalize_rejects_missing_paths() {
        let err = canonicalize_dir(Path::new("/definitely/not/here/xyz")).unwrap_err();
        assert!(matches!(err, CodeIntelError::NotFound(_)));
    }

    #[test]
    fn canonicalize_dir_strips_trailing_slash() {
        let tmp = tempfile::tempdir().unwrap();
        let with_slash = format!("{}/", tmp.path().display());
        let canonical = canonicalize_dir(Path::new(&with_slash)).unwrap();
        assert!(!canonical.ends_with('/'));
        assert_eq!(canonical, canonicalize_dir(tmp.path()).unwrap());
    }
}
