//! TencentDB Agent Memory v3 Adapter (BUGRAIL-SPECOS-017 §4).
//!
//! Pinned compatibility: upstream tag `v2.0.0`, commit
//! `0aff21a2d9f2b8a0354aaa80a2e586aab4054562`, plus the minimal
//! `v2.0.0+bugrail.1` patch that makes `conversation/add` accept caller
//! message ids and upsert by them (spec §4.1). The patched version string
//! is the capture writability gate.
//!
//! Transport rules:
//! - `Authorization: Bearer` + `x-tdai-service-id` on every call.
//! - Health is the public MemoryCore `GET /health` — never `/v3/tools/list`
//!   (Knowledge service, POST-only there).
//! - HTTP non-success, non-JSON, timeouts and business `code != 0` are
//!   errors with stable classes; request/trace ids may be retained, response
//!   bodies and credentials never are.
//! - Redirects are never followed (`Policy::none`): a redirecting endpoint
//!   is a misconfiguration, and cross-scheme redirects cannot happen.
//! - No TLS-verification bypass, no URL credentials, HTTPS except loopback
//!   (enforced in `config::validate_memory_endpoint`).
//! - Oversized upstream responses are aborted at their bound.

use std::time::{Duration, Instant};

use serde::Deserialize;

use super::config::ResolvedMemoryProvider;
use super::{
    MemoryCaptureBatch, MemoryCaptureReceipt, MemoryError, MemoryErrorClass, MemoryHealthReport,
    MemoryHealthStatus, MemoryLayer, MemoryProvider, MemoryRecallHit, MemoryRecallRequest,
    MemoryRecallResult,
};

pub const ADAPTER_ID: &str = "tencentdb-agent-memory-v3";
/// Exact patched version required for capture writability (spec §4.1).
pub const EXPECTED_UPSTREAM_VERSION: &str = "v2.0.0+bugrail.1";
/// Upstream pin: TencentCloud/TencentDB-Agent-Memory tag v2.0.0.
pub const UPSTREAM_PIN_COMMIT: &str = "0aff21a2d9f2b8a0354aaa80a2e586aab4054562";

/// Bound for control-plane responses (`/health`, envelopes).
const MAX_CONTROL_BODY_BYTES: usize = 1024 * 1024;
/// Bound for recall payloads.
const MAX_RECALL_BODY_BYTES: usize = 4 * 1024 * 1024;

pub struct TencentDbMemoryAdapter {
    provider: ResolvedMemoryProvider,
    client: reqwest::Client,
}

impl TencentDbMemoryAdapter {
    pub fn new(provider: ResolvedMemoryProvider) -> Self {
        // Redirects are never followed: a 3xx surfaces as a misconfigured
        // endpoint instead of silently crossing schemes or hosts.
        let client = reqwest::Client::builder()
            .timeout(provider.timeout)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap_or_default();
        Self { provider, client }
    }

    fn endpoint(&self, path: &str) -> String {
        format!("{}{}", self.provider.endpoint, path)
    }

    fn authorize(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        request
            .bearer_auth(&self.provider.secret)
            .header("x-tdai-service-id", &self.provider.service_id)
    }

    async fn health(&self) -> Result<MemoryHealthReport, MemoryError> {
        let started = Instant::now();
        let request = self.authorize(self.client.get(self.endpoint("/health")));
        let response = send(request, &self.provider).await?;
        let status = response.status();
        let trace_id = header_trace_id(&response);
        if !status.is_success() {
            return Err(classify_status(status, &self.provider, trace_id));
        }
        let body = read_limited(response, MAX_CONTROL_BODY_BYTES, &self.provider).await?;
        let value: serde_json::Value = serde_json::from_slice(&body).map_err(|_| {
            memory_error(
                MemoryErrorClass::InvalidResponse,
                "health response is not valid JSON",
                &self.provider,
                None,
            )
        })?;
        let version = value
            .get("version")
            .or_else(|| value.get("data").and_then(|data| data.get("version")))
            .and_then(|version| version.as_str())
            .map(str::to_string);
        let writable = version.as_deref() == Some(EXPECTED_UPSTREAM_VERSION);
        Ok(MemoryHealthReport {
            status: MemoryHealthStatus::Healthy,
            version,
            writable,
            error_class: None,
            message: None,
            latency_ms: Some(started.elapsed().as_millis() as u64),
            trace_id,
        })
    }

    async fn post_envelope(
        &self,
        path: &str,
        body: &serde_json::Value,
        max_response: usize,
    ) -> Result<serde_json::Value, MemoryError> {
        let request = self
            .authorize(self.client.post(self.endpoint(path)))
            .json(body);
        let response = send(request, &self.provider).await?;
        let status = response.status();
        let trace_id = header_trace_id(&response);
        if !status.is_success() {
            return Err(classify_status(status, &self.provider, trace_id));
        }
        let raw = read_limited(response, max_response, &self.provider).await?;
        let envelope: Envelope = serde_json::from_slice(&raw).map_err(|_| {
            memory_error(
                MemoryErrorClass::InvalidResponse,
                "response envelope is not valid JSON",
                &self.provider,
                trace_id.clone(),
            )
        })?;
        let trace_id = envelope.request_id.clone().or(trace_id);
        if envelope.code != 0 {
            // Safe message only: the upstream message text is discarded.
            return Err(memory_error(
                MemoryErrorClass::Upstream,
                format!("upstream business error code {}", envelope.code),
                &self.provider,
                trace_id,
            ));
        }
        Ok(envelope.data)
    }
}

#[async_trait::async_trait]
impl MemoryProvider for TencentDbMemoryAdapter {
    fn adapter_id(&self) -> &str {
        ADAPTER_ID
    }

    async fn health(&self) -> Result<MemoryHealthReport, MemoryError> {
        self.health().await
    }

    async fn capture(
        &self,
        batch: &MemoryCaptureBatch,
    ) -> Result<MemoryCaptureReceipt, MemoryError> {
        let body = serde_json::json!({
            "team_id": batch.team_id,
            "agent_id": batch.agent_id,
            "user_id": batch.user_id,
            "session_id": batch.session_id,
            "task_id": batch.task_id,
            "messages": batch
                .messages
                .iter()
                .map(|message| {
                    serde_json::json!({
                        "id": message.id,
                        "role": match message.role {
                            super::MemoryRole::User => "user",
                            super::MemoryRole::Assistant => "assistant",
                        },
                        "content": message.content,
                    })
                })
                .collect::<Vec<_>>(),
        });
        let data = self
            .post_envelope("/v3/conversation/add", &body, MAX_CONTROL_BODY_BYTES)
            .await?;
        // Patched contract: accepted ids echo the caller ids. Their absence
        // means the Gateway does not implement the upsert contract (vanilla
        // regenerates ids), so no delivery receipt may be recorded.
        let accepted = data
            .get("accepted_ids")
            .and_then(|ids| ids.as_array())
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| id.as_str().map(str::to_string))
                    .collect::<Vec<_>>()
            })
            .ok_or_else(|| {
                memory_error(
                    MemoryErrorClass::InvalidResponse,
                    "capture response is missing accepted message ids",
                    &self.provider,
                    None,
                )
            })?;
        Ok(MemoryCaptureReceipt {
            accepted_ids: accepted,
            trace_id: None,
        })
    }

    async fn recall(
        &self,
        request: &MemoryRecallRequest,
        deadline: Duration,
    ) -> Result<MemoryRecallResult, MemoryError> {
        let l1_body = serde_json::json!({
            "team_id": request.team_id,
            "agent_id": request.agent_id,
            "user_id": request.user_id,
            "query": request.query,
            "limit": request.limit,
        });
        let l3_body = serde_json::json!({
            "team_id": request.team_id,
            "agent_id": request.agent_id,
            "user_id": request.user_id,
            "query": request.query,
        });
        let include_core = request.include_core;

        // One unified deadline across both layers — L1 and L3 run in
        // parallel, never sequential round trips (spec §6).
        let joined = tokio::time::timeout(deadline, async {
            let l1 = self.post_envelope("/v3/atomic/search", &l1_body, MAX_RECALL_BODY_BYTES);
            let l3 = include_core
                .then(|| self.post_envelope("/v3/core/read", &l3_body, MAX_RECALL_BODY_BYTES));
            match l3 {
                Some(l3) => {
                    let (l1_result, l3_result) = tokio::join!(l1, l3);
                    (l1_result, Some(l3_result))
                }
                None => (l1.await, None),
            }
        })
        .await;

        let (l1_result, l3_result) = joined.map_err(|_| {
            memory_error(
                MemoryErrorClass::Timeout,
                "recall exceeded the unified deadline",
                &self.provider,
                None,
            )
        })?;

        let l1_data = l1_result?;
        let l1 = parse_hits(&l1_data, MemoryLayer::L1, &self.provider)?;
        let l3 = match l3_result {
            Some(Ok(data)) => parse_hits(&data, MemoryLayer::L3, &self.provider)?,
            // A soft L3 failure degrades to "no Core memory" — L1 results
            // stay usable. Hard L1 failures above already returned.
            Some(Err(err)) => {
                tracing::warn!(
                    provider = %self.provider.provider_id,
                    class = err.class.key(),
                    "[memory] L3 core recall degraded"
                );
                Vec::new()
            }
            None => Vec::new(),
        };
        Ok(MemoryRecallResult { l1, l3 })
    }

    async fn probe_read(&self, request: &MemoryRecallRequest) -> Result<(), MemoryError> {
        let body = serde_json::json!({
            "team_id": request.team_id,
            "agent_id": request.agent_id,
            "user_id": request.user_id,
            "query": request.query,
        });
        // Credentials + isolation check only; the response body is discarded.
        self.post_envelope("/v3/core/read", &body, MAX_CONTROL_BODY_BYTES)
            .await?;
        Ok(())
    }
}

// ── Transport helpers ───────────────────────────────────────────────────────

#[derive(Deserialize)]
struct Envelope {
    #[serde(default)]
    code: i64,
    #[serde(default, rename = "request_id", alias = "requestId")]
    request_id: Option<String>,
    #[serde(default)]
    data: serde_json::Value,
}

fn memory_error(
    class: MemoryErrorClass,
    message: impl Into<String>,
    provider: &ResolvedMemoryProvider,
    trace_id: Option<String>,
) -> MemoryError {
    let mut error = MemoryError::new(class, message).with_provider_id(provider.provider_id.clone());
    if let Some(trace_id) = trace_id {
        error = error.with_trace_id(trace_id);
    }
    error
}

async fn send(
    request: reqwest::RequestBuilder,
    provider: &ResolvedMemoryProvider,
) -> Result<reqwest::Response, MemoryError> {
    request.send().await.map_err(|error| {
        let class = if error.is_timeout() {
            MemoryErrorClass::Timeout
        } else if error.is_connect() || error.is_request() {
            MemoryErrorClass::Unavailable
        } else {
            MemoryErrorClass::InvalidResponse
        };
        // Transport detail only — never bodies or credentials.
        let message = if error.is_timeout() {
            "request timed out".to_string()
        } else {
            "connection failed".to_string()
        };
        memory_error(class, message, provider, None)
    })
}

fn header_trace_id(response: &reqwest::Response) -> Option<String> {
    ["x-tdai-request-id", "x-request-id", "x-tdai-trace-id"]
        .iter()
        .find_map(|name| response.headers().get(*name))
        .and_then(|value| value.to_str().ok())
        .map(|value| value.chars().take(128).collect())
}

/// Read a response body with a hard bound; exceeding it aborts the read
/// (spec §9 "oversized upstream responses are aborted at their bound").
async fn read_limited(
    response: reqwest::Response,
    limit: usize,
    provider: &ResolvedMemoryProvider,
) -> Result<Vec<u8>, MemoryError> {
    let mut body = Vec::new();
    let mut stream = response;
    loop {
        let chunk = stream.chunk().await.map_err(|error| {
            let class = if error.is_timeout() {
                MemoryErrorClass::Timeout
            } else {
                MemoryErrorClass::Unavailable
            };
            memory_error(class, "reading the response failed", provider, None)
        })?;
        match chunk {
            Some(bytes) => {
                if body.len() + bytes.len() > limit {
                    return Err(memory_error(
                        MemoryErrorClass::InvalidResponse,
                        "upstream response exceeds the size bound",
                        provider,
                        None,
                    ));
                }
                body.extend_from_slice(&bytes);
            }
            None => return Ok(body),
        }
    }
}

fn classify_status(
    status: reqwest::StatusCode,
    provider: &ResolvedMemoryProvider,
    trace_id: Option<String>,
) -> MemoryError {
    let class = match status.as_u16() {
        401 | 403 => MemoryErrorClass::Unauthorized,
        408 => MemoryErrorClass::Timeout,
        429 => MemoryErrorClass::RateLimited,
        300..=399 => {
            // Redirects are never followed; a redirecting endpoint is a
            // misconfiguration (and cross-scheme redirects stay impossible).
            MemoryErrorClass::ConfigInvalid
        }
        500..=599 => MemoryErrorClass::Unavailable,
        _ => MemoryErrorClass::InvalidResponse,
    };
    memory_error(
        class,
        format!("upstream returned HTTP {}", status.as_u16()),
        provider,
        trace_id,
    )
}

fn parse_hits(
    data: &serde_json::Value,
    layer: MemoryLayer,
    provider: &ResolvedMemoryProvider,
) -> Result<Vec<MemoryRecallHit>, MemoryError> {
    // L1: the pinned v3 contract returns `data.items` (sdk-v3.yaml
    // AtomicSearchData / AtomicQueryData); `hits` is accepted as a legacy
    // alias. L3: either an items/hits array or a single `content`
    // document — normalized to the same vendor-neutral hit.
    let hits = data
        .get("items")
        .or_else(|| data.get("hits"))
        .and_then(|hits| hits.as_array());
    if let Some(hits) = hits {
        let mut out = Vec::new();
        for hit in hits {
            let content = hit
                .get("content")
                .and_then(|content| content.as_str())
                .unwrap_or_default()
                .to_string();
            let remote_id = hit
                .get("id")
                .and_then(|id| id.as_str())
                .unwrap_or_default()
                .to_string();
            if content.trim().is_empty() {
                continue;
            }
            out.push(MemoryRecallHit {
                remote_id,
                layer,
                score: hit.get("score").and_then(|score| score.as_f64()),
                content,
                captured_at: hit
                    .get("captured_at")
                    .or_else(|| hit.get("created_at"))
                    .and_then(|value| value.as_str())
                    .map(str::to_string),
            });
        }
        return Ok(out);
    }
    if layer == MemoryLayer::L3 {
        if let Some(content) = data.get("content").and_then(|content| content.as_str()) {
            if !content.trim().is_empty() {
                return Ok(vec![MemoryRecallHit {
                    remote_id: "core".into(),
                    layer,
                    score: None,
                    content: content.to_string(),
                    captured_at: data
                        .get("updated_at")
                        .and_then(|value| value.as_str())
                        .map(str::to_string),
                }]);
            }
        }
        // Empty Core memory is a successful recall with zero candidates.
        return Ok(Vec::new());
    }
    Err(memory_error(
        MemoryErrorClass::InvalidResponse,
        "recall response is missing a hits array",
        provider,
        None,
    ))
}
