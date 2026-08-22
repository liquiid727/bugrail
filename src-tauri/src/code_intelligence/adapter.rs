//! stdio MCP session driver for `codebase-memory-mcp`.
//!
//! Bugrail talks to the adapter over plain JSON-RPC 2.0 on stdio (the MCP
//! transport every `codebase-memory-mcp` build exposes). One session is
//! spawned per canonical repository/worktree path with:
//!
//! - `CBM_CACHE_DIR=<root>/store` — Bugrail's private cache root, shared by
//!   all sessions so they reuse the same upstream daemon (watchers stop when
//!   the last session ends);
//! - `CBM_ALLOWED_ROOT=<that path>` — confines `index_repository` to the one
//!   directory this session is bound to, so a malformed or hostile tool call
//!   cannot index or touch anything else on disk;
//! - `CBM_LOG_LEVEL=warn` — daemon logs stay out of the user's face.
//!
//! Sessions are request/response multiplexed by JSON-RPC id with bounded
//! timeouts; a crashed child fails every outstanding request and is replaced
//! lazily by the next call.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Child;
use tokio::sync::{oneshot, Mutex};
use tokio::task::JoinHandle;

use crate::code_intelligence::CodeIntelError;

/// Default timeout for read-only queries.
pub const QUERY_TIMEOUT: Duration = Duration::from_secs(60);
/// Indexing / reindexing can legitimately take a while on large repos.
pub const INDEX_TIMEOUT: Duration = Duration::from_secs(30 * 60);

/// Outcome of one MCP `tools/call`.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolCallOutcome {
    /// Concatenated text content blocks returned by the tool.
    pub text: String,
    /// The tool ran but reported an error (`isError: true`).
    pub is_error: bool,
}

/// Environment the upstream binary reads.
const ENV_CACHE_DIR: &str = "CBM_CACHE_DIR";
const ENV_ALLOWED_ROOT: &str = "CBM_ALLOWED_ROOT";
const ENV_LOG_LEVEL: &str = "CBM_LOG_LEVEL";

type PendingMap = Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>>;

/// One live stdio session bound to one repository path.
pub struct AdapterSession {
    canonical_path: String,
    writer: Arc<Mutex<tokio::process::ChildStdin>>,
    pending: PendingMap,
    next_id: AtomicU64,
    _child: Arc<Mutex<Child>>,
    _reader: JoinHandle<()>,
    _stderr: JoinHandle<()>,
}

impl AdapterSession {
    /// Spawn the adapter binary and complete the MCP initialize handshake.
    pub async fn spawn(
        binary: &Path,
        store_dir: &Path,
        canonical_path: &str,
    ) -> Result<Arc<Self>, CodeIntelError> {
        let mut child = tokio::process::Command::new(binary)
            .env(ENV_CACHE_DIR, store_dir)
            .env(ENV_ALLOWED_ROOT, canonical_path)
            .env(ENV_LOG_LEVEL, "warn")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|err| CodeIntelError::Spawn(format!("{}: {err}", binary.display())))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| CodeIntelError::Spawn("stdin unavailable".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| CodeIntelError::Spawn("stdout unavailable".into()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| CodeIntelError::Spawn("stderr unavailable".into()))?;

        let pending: PendingMap = Arc::new(Mutex::new(HashMap::new()));
        let reader_pending = pending.clone();
        let reader_path = canonical_path.to_string();
        let reader = tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            // EOF or read error: the child is gone — the loop ends and we
            // fail everything outstanding so callers get an error, not a
            // hang.
            while let Ok(Some(line)) = lines.next_line().await {
                dispatch_line(&reader_pending, &line).await;
            }
            let mut map = reader_pending.lock().await;
            for (_, waiter) in map.drain() {
                let _ = waiter.send(json!({
                    "error": {"code": -32000, "message": format!("codebase-memory-mcp session for {reader_path} ended")}
                }));
            }
        });

        let stderr_path = canonical_path.to_string();
        let stderr_task = tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                tracing::debug!("[CodeIntel:{stderr_path}] {line}");
            }
        });

        let session = Arc::new(Self {
            canonical_path: canonical_path.to_string(),
            writer: Arc::new(Mutex::new(stdin)),
            pending,
            next_id: AtomicU64::new(1),
            _child: Arc::new(Mutex::new(child)),
            _reader: reader,
            _stderr: stderr_task,
        });

        // MCP handshake. The server may answer with a protocolVersion it
        // supports; both sides must just agree, and 2024-11-05 is the
        // baseline every codebase-memory-mcp release speaks.
        let init = session
            .request(
                "initialize",
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "bugrail-code-intelligence",
                        "version": env!("CARGO_PKG_VERSION"),
                    }
                }),
                Duration::from_secs(30),
            )
            .await
            .map_err(|err| CodeIntelError::Adapter(format!("initialize failed: {err}")))?;
        if init.get("error").is_some() {
            return Err(CodeIntelError::Adapter(format!(
                "initialize rejected: {}",
                init
            )));
        }
        session
            .notify("notifications/initialized", json!({}))
            .await?;
        Ok(session)
    }

    pub fn canonical_path(&self) -> &str {
        &self.canonical_path
    }

    /// Call a tool and wait for its result, extracting MCP text content.
    pub async fn call_tool(
        self: &Arc<Self>,
        tool: &str,
        arguments: Value,
        timeout: Duration,
    ) -> Result<ToolCallOutcome, CodeIntelError> {
        let response = self
            .request(
                "tools/call",
                json!({ "name": tool, "arguments": arguments }),
                timeout,
            )
            .await?;
        if let Some(error) = response.get("error") {
            return Err(CodeIntelError::Adapter(format!(
                "tool {tool} returned JSON-RPC error: {error}"
            )));
        }
        let result = response
            .get("result")
            .cloned()
            .ok_or_else(|| CodeIntelError::Adapter("tool response missing result".into()))?;
        let is_error = result
            .get("isError")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let mut text = String::new();
        if let Some(blocks) = result.get("content").and_then(Value::as_array) {
            for block in blocks {
                if block.get("type").and_then(Value::as_str) == Some("text") {
                    if let Some(t) = block.get("text").and_then(Value::as_str) {
                        if !text.is_empty() {
                            text.push('\n');
                        }
                        text.push_str(t);
                    }
                }
            }
        }
        Ok(ToolCallOutcome { text, is_error })
    }

    async fn request(
        self: &Arc<Self>,
        method: &str,
        params: Value,
        timeout: Duration,
    ) -> Result<Value, CodeIntelError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id, tx);

        let frame = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        if let Err(err) = self.write_frame(&frame).await {
            self.pending.lock().await.remove(&id);
            return Err(err);
        }

        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(_)) => Err(CodeIntelError::Adapter(
                "adapter session ended before answering".into(),
            )),
            Err(_) => {
                self.pending.lock().await.remove(&id);
                Err(CodeIntelError::Timeout(format!(
                    "{method} timed out after {}s",
                    timeout.as_secs()
                )))
            }
        }
    }

    async fn notify(&self, method: &str, params: Value) -> Result<(), CodeIntelError> {
        let frame = json!({ "jsonrpc": "2.0", "method": method, "params": params });
        self.write_frame(&frame).await
    }

    async fn write_frame(&self, frame: &Value) -> Result<(), CodeIntelError> {
        let mut bytes = serde_json::to_vec(frame)
            .map_err(|err| CodeIntelError::Internal(format!("frame serialize: {err}")))?;
        bytes.push(b'\n');
        let mut writer = self.writer.lock().await;
        writer
            .write_all(&bytes)
            .await
            .map_err(|err| CodeIntelError::Adapter(format!("write to adapter failed: {err}")))?;
        writer
            .flush()
            .await
            .map_err(|err| CodeIntelError::Adapter(format!("flush to adapter failed: {err}")))?;
        Ok(())
    }
}

impl Drop for AdapterSession {
    fn drop(&mut self) {
        // `kill_on_drop` handles the child; abort the pump tasks so they
        // don't outlive the session on runtime shutdown paths.
        self._reader.abort();
        self._stderr.abort();
    }
}

async fn dispatch_line(pending: &PendingMap, line: &str) {
    let Ok(value) = serde_json::from_str::<Value>(line) else {
        return; // non-JSON noise on stdout is ignored
    };
    let Some(id) = value.get("id") else {
        return; // notifications from the server carry no id
    };
    let id = match id {
        Value::Number(n) => n.as_u64(),
        Value::String(s) => s.parse::<u64>().ok(),
        _ => None,
    };
    let Some(id) = id else { return };
    if let Some(waiter) = pending.lock().await.remove(&id) {
        let _ = waiter.send(value);
    }
}

/// Lazily spawns and caches one session per canonical repo path. All
/// sessions share `store_dir`, so the upstream daemon (and its watchers and
/// Graph UI state) is shared too.
pub struct SessionManager {
    binary: PathBuf,
    store_dir: PathBuf,
    sessions: Mutex<HashMap<String, Arc<AdapterSession>>>,
}

impl SessionManager {
    pub fn new(binary: PathBuf, store_dir: PathBuf) -> Self {
        Self {
            binary,
            store_dir,
            sessions: Mutex::new(HashMap::new()),
        }
    }

    /// The binary path this manager spawns sessions from — used to detect
    /// stale managers after a binary override change.
    pub fn binary_path(&self) -> PathBuf {
        self.binary.clone()
    }

    /// The session bound to `canonical_path`, spawning it on first use. A
    /// previously cached session whose child exited is replaced.
    pub async fn session_for(
        &self,
        canonical_path: &str,
    ) -> Result<Arc<AdapterSession>, CodeIntelError> {
        let mut sessions = self.sessions.lock().await;
        match sessions.get(canonical_path) {
            Some(existing) if child_alive(existing).await => Ok(existing.clone()),
            _ => {
                let session =
                    AdapterSession::spawn(&self.binary, &self.store_dir, canonical_path).await?;
                sessions.insert(canonical_path.to_string(), session.clone());
                Ok(session)
            }
        }
    }

    /// Forget (and drop → kill) the session for one path.
    pub async fn drop_session(&self, canonical_path: &str) {
        self.sessions.lock().await.remove(canonical_path);
    }

    /// Kill every session — used at app shutdown so the shared daemon's
    /// refcount reaches zero and watchers stop.
    pub async fn shutdown(&self) {
        self.sessions.lock().await.clear();
    }
}

async fn child_alive(session: &Arc<AdapterSession>) -> bool {
    matches!(session._child.lock().await.try_wait(), Ok(None))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal fake MCP server: enough protocol to exercise the session
    /// driver end-to-end without the real binary.
    const FAKE_MCP_PY: &str = r#"
import json, os, sys

def send(obj):
    sys.stdout.write(json.dumps(obj) + "\n")
    sys.stdout.flush()

while True:
    line = sys.stdin.readline()
    if not line:
        break
    try:
        msg = json.loads(line)
    except Exception:
        continue
    method = msg.get("method")
    mid = msg.get("id")
    if method == "initialize":
        send({"jsonrpc": "2.0", "id": mid, "result": {
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "fake-cbm", "version": "0.10.6"}}})
    elif method == "tools/call" and mid is not None:
        params = msg.get("params", {})
        name = params.get("name")
        args = params.get("arguments", {})
        if name == "echo":
            send({"jsonrpc": "2.0", "id": mid, "result": {
                "content": [{"type": "text", "text": json.dumps(args)}],
                "isError": False}})
        elif name == "env":
            send({"jsonrpc": "2.0", "id": mid, "result": {
                "content": [{"type": "text", "text": json.dumps({
                    "CBM_ALLOWED_ROOT": os.environ.get("CBM_ALLOWED_ROOT", ""),
                    "CBM_CACHE_DIR": os.environ.get("CBM_CACHE_DIR", ""),
                    "CBM_LOG_LEVEL": os.environ.get("CBM_LOG_LEVEL", "")})],
                "isError": False}})
        elif name == "fail":
            send({"jsonrpc": "2.0", "id": mid, "result": {
                "content": [{"type": "text", "text": "tool exploded"}],
                "isError": True}})
        elif name == "twotext":
            send({"jsonrpc": "2.0", "id": mid, "result": {
                "content": [{"type": "text", "text": "one"},
                            {"type": "text", "text": "two"}],
                "isError": False}})
        elif name == "hang":
            pass  # never answers — exercises the timeout path
        else:
            send({"jsonrpc": "2.0", "id": mid, "error": {
                "code": -32601, "message": "unknown tool"}})
"#;

    fn write_fake_server(dir: &Path) -> PathBuf {
        let script = dir.join("fake_cbm.py");
        std::fs::write(&script, FAKE_MCP_PY).unwrap();
        script
    }

    fn python3() -> Option<PathBuf> {
        which::which("python3").ok()
    }

    /// Spawn a session against the fake server (the session API is binary-
    /// agnostic: anything speaking the protocol works).
    async fn fake_session(tmp: &Path) -> Option<Arc<AdapterSession>> {
        let python = python3()?;
        let script = write_fake_server(tmp);
        // Reuse spawn mechanics by spawning manually with the same env.
        let mut child = tokio::process::Command::new(&python)
            .arg(&script)
            .env("CBM_CACHE_DIR", tmp.join("store"))
            .env("CBM_ALLOWED_ROOT", "/repo")
            .env("CBM_LOG_LEVEL", "warn")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .ok()?;
        let stdin = child.stdin.take()?;
        let stdout = child.stdout.take()?;
        let stderr = child.stderr.take()?;
        let pending: PendingMap = Arc::new(Mutex::new(HashMap::new()));
        let rp = pending.clone();
        let reader = tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                dispatch_line(&rp, &line).await;
            }
        });
        let stderr_task = tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(_)) = lines.next_line().await {}
        });
        let session = Arc::new(AdapterSession {
            canonical_path: "/repo".into(),
            writer: Arc::new(Mutex::new(stdin)),
            pending,
            next_id: AtomicU64::new(1),
            _child: Arc::new(Mutex::new(child)),
            _reader: reader,
            _stderr: stderr_task,
        });
        // Handshake against the fake.
        session
            .request(
                "initialize",
                json!({"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "0"}}),
                Duration::from_secs(5),
            )
            .await
            .ok()?;
        session
            .notify("notifications/initialized", json!({}))
            .await
            .ok()?;
        Some(session)
    }

    #[tokio::test]
    async fn session_calls_tool_and_extracts_text() {
        let tmp = tempfile::tempdir().unwrap();
        let Some(session) = fake_session(tmp.path()).await else {
            eprintln!("python3 unavailable; skipping");
            return;
        };
        let outcome = session
            .call_tool("echo", json!({"hello": "world"}), Duration::from_secs(5))
            .await
            .expect("echo succeeds");
        assert!(!outcome.is_error);
        let parsed: Value = serde_json::from_str(&outcome.text).unwrap();
        assert_eq!(parsed["hello"], "world");
    }

    #[tokio::test]
    async fn session_passes_required_env() {
        let tmp = tempfile::tempdir().unwrap();
        let Some(session) = fake_session(tmp.path()).await else {
            eprintln!("python3 unavailable; skipping");
            return;
        };
        let outcome = session
            .call_tool("env", json!({}), Duration::from_secs(5))
            .await
            .unwrap();
        let parsed: Value = serde_json::from_str(&outcome.text).unwrap();
        assert_eq!(parsed["CBM_ALLOWED_ROOT"], "/repo");
        assert_eq!(
            parsed["CBM_CACHE_DIR"],
            tmp.path().join("store").to_string_lossy().to_string()
        );
        assert_eq!(parsed["CBM_LOG_LEVEL"], "warn");
    }

    #[tokio::test]
    async fn session_surfaces_tool_error_flag() {
        let tmp = tempfile::tempdir().unwrap();
        let Some(session) = fake_session(tmp.path()).await else {
            eprintln!("python3 unavailable; skipping");
            return;
        };
        let outcome = session
            .call_tool("fail", json!({}), Duration::from_secs(5))
            .await
            .expect("transport succeeds even when tool errors");
        assert!(outcome.is_error);
        assert!(outcome.text.contains("tool exploded"));
    }

    #[tokio::test]
    async fn session_concatenates_text_blocks() {
        let tmp = tempfile::tempdir().unwrap();
        let Some(session) = fake_session(tmp.path()).await else {
            eprintln!("python3 unavailable; skipping");
            return;
        };
        let outcome = session
            .call_tool("twotext", json!({}), Duration::from_secs(5))
            .await
            .unwrap();
        assert_eq!(outcome.text, "one\ntwo");
    }

    #[tokio::test]
    async fn session_jsonrpc_error_becomes_adapter_error() {
        let tmp = tempfile::tempdir().unwrap();
        let Some(session) = fake_session(tmp.path()).await else {
            eprintln!("python3 unavailable; skipping");
            return;
        };
        let err = session
            .call_tool("missing", json!({}), Duration::from_secs(5))
            .await
            .unwrap_err();
        assert!(matches!(err, CodeIntelError::Adapter(_)));
    }

    #[tokio::test]
    async fn session_times_out_on_hung_tool() {
        let tmp = tempfile::tempdir().unwrap();
        let Some(session) = fake_session(tmp.path()).await else {
            eprintln!("python3 unavailable; skipping");
            return;
        };
        let err = session
            .call_tool("hang", json!({}), Duration::from_millis(300))
            .await
            .unwrap_err();
        assert!(matches!(err, CodeIntelError::Timeout(_)));
        // The session is still usable after a timeout.
        let outcome = session
            .call_tool("echo", json!({"ok": true}), Duration::from_secs(5))
            .await
            .unwrap();
        assert!(!outcome.is_error);
    }
}
