//! Listener-facing access for the read-only `codebase_*` tools carried by
//! codeg-mcp (Bugrail Code Intelligence). The listener resolves the caller's
//! working directory from its per-launch token and hands the query here; the
//! production impl binds the query to the enabled index covering that
//! directory via [`crate::code_intelligence`] and never lets the caller pick
//! a project. Kept as a trait so the listener stays decoupled from the
//! runtime (and tests can stub it). Mirrors [`crate::acp::work_task_tools`].

use std::path::Path;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Outcome of one codebase query, ready to render into an MCP tool result.
/// Errors are carried as readable text (`is_error: true`) rather than
/// transport failures so the agent can react ("no index covers this repo —
/// ask the user to enable Code Intelligence") instead of aborting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseQueryOutcome {
    /// Tool-facing text (caps and truncation already applied).
    pub text: String,
    /// `true` when the text is an error/degraded message.
    pub is_error: bool,
    /// The project key the query was bound to, when one was found.
    pub project: Option<String>,
}

impl CodebaseQueryOutcome {
    pub fn error(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_error: true,
            project: None,
        }
    }
}

#[async_trait]
pub trait CodebaseToolAccess: Send + Sync {
    /// Run one read-only code-intelligence query bound to `working_dir`
    /// (the connected directory of the caller's parent session). `tool` is
    /// a Bugrail tool name (`codebase_search`, …); `arguments` is the raw
    /// MCP argument object — the impl clamps limits and injects the bound
    /// `project` key itself.
    async fn query(
        &self,
        working_dir: &Path,
        tool: &str,
        arguments: Value,
    ) -> CodebaseQueryOutcome;
}

/// Stub for processes/tests without Code Intelligence: every query degrades
/// to a readable error and never blocks.
pub struct NullCodebaseTools;

#[async_trait]
impl CodebaseToolAccess for NullCodebaseTools {
    async fn query(&self, _working_dir: &Path, _tool: &str, _arguments: Value) -> CodebaseQueryOutcome {
        CodebaseQueryOutcome::error(
            "Code Intelligence is not available in this process. Ask the user to enable it on \
             the Context page.",
        )
    }
}

/// Production impl: delegates to the process-global Code Intelligence
/// runtime. Tool names map through the closed [`crate::code_intelligence::CodeQuery`]
/// set — unknown or write-side names are rejected here, matching the
/// companion-side gating.
pub struct RuntimeCodebaseTools;

#[async_trait]
impl CodebaseToolAccess for RuntimeCodebaseTools {
    async fn query(&self, working_dir: &Path, tool: &str, arguments: Value) -> CodebaseQueryOutcome {
        let Some(runtime) = crate::code_intelligence::runtime() else {
            return CodebaseQueryOutcome::error(
                "Code Intelligence is not initialized in this process.",
            );
        };
        let Some(query) = crate::code_intelligence::CodeQuery::from_tool_name(tool) else {
            return CodebaseQueryOutcome::error(format!("unknown codebase tool: {tool}"));
        };
        match runtime.query(working_dir, query, arguments).await {
            Ok(outcome) => CodebaseQueryOutcome {
                text: outcome.text,
                is_error: outcome.is_error,
                project: Some(outcome.project),
            },
            Err(err) => CodebaseQueryOutcome::error(err.to_string()),
        }
    }
}
