//! Context control plane owned by CodeG.
//!
//! Providers contribute assets, but selection, budgeting, provenance and the
//! immutable package bound to a WorkTask generation remain local authority.

use chrono::Utc;
use reqwest::Client;
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::Path;
use std::time::Duration;

use crate::db::entities::{
    context_activity, work_task, work_task_context_item, work_task_context_pack,
};
use crate::db::error::DbError;
use crate::db::service::specos_runtime_service;
use crate::models::{
    ContextActivityInfo, ContextItemInfo, ContextOverview, ContextPackageInfo,
    ContextProviderConfig, ContextProviderHealth,
};

pub struct PreparedContext {
    pub package: ContextPackageInfo,
    pub prompt: String,
}

/// An engine-computed extra Context Pack item (currently: the Code
/// Intelligence snapshot for agents that cannot query the index live via
/// MCP). Appended after the loadout's file sources; budget-aware — an item
/// that doesn't fit the loadout's remaining byte/token budget is skipped
/// rather than failing the run (a degraded snapshot never blocks work).
pub struct EngineContextItem {
    pub kind: String,
    pub source: String,
    pub title: String,
    pub content: String,
    pub provenance: serde_json::Value,
}

/// One collected Context Pack item, pre-insertion. Local loadout sources and
/// engine-computed extras (Code Intelligence snapshots) share this shape so
/// the package hash and the DB insert stay uniform.
struct PackCandidate {
    kind: String,
    source: String,
    title: String,
    content: String,
    hash: String,
    required: bool,
    provenance: serde_json::Value,
}

/// One Memory recall candidate awaiting package-budget adjudication. Carries
/// the evidence slot plus the bounded metadata needed to record an
/// included/excluded decision without retaining remote content.
struct MemoryCandidate {
    candidate: PackCandidate,
    evidence: usize,
    layer: &'static str,
    remote_id: String,
    score: Option<f64>,
}

/// Per-provider recall evidence persisted into
/// `work_task_context_pack.memory_evidence`. Holds hashes and bounded
/// metadata only — never remote content, query text or credentials
/// (BUGRAIL-SPECOS-017 R05/R07).
struct MemoryProviderEvidence {
    provider_id: String,
    adapter: String,
    query_hash: String,
    included: Vec<serde_json::Value>,
    excluded: Vec<serde_json::Value>,
}

/// Untrusted-data bounds for persisted recall evidence (R07): fixed-set
/// reasons, capped remote ids and capped array sizes keep the column small
/// and deterministic for identical recall inputs.
const EVIDENCE_MAX_REMOTE_ID_CHARS: usize = 128;
const EVIDENCE_MAX_REASON_CHARS: usize = 64;
const EVIDENCE_MAX_ENTRIES: usize = 64;

fn bound_reason(reason: &str) -> String {
    reason
        .chars()
        .take(EVIDENCE_MAX_REASON_CHARS)
        .collect::<String>()
}

fn bound_remote_id(remote_id: &str) -> String {
    remote_id
        .chars()
        .take(EVIDENCE_MAX_REMOTE_ID_CHARS)
        .collect::<String>()
}

impl MemoryProviderEvidence {
    fn entry(candidate: &MemoryCandidate, reason: Option<&str>) -> serde_json::Value {
        let mut value = serde_json::json!({
            "contentHash": candidate.candidate.hash,
            "layer": candidate.layer,
            "remoteId": bound_remote_id(&candidate.remote_id),
        });
        if let Some(score) = candidate.score {
            value["score"] = serde_json::json!(score);
        }
        if let Some(reason) = reason {
            value["reason"] = serde_json::json!(bound_reason(reason));
        }
        value
    }

    fn record_included(&mut self, candidate: &MemoryCandidate) {
        if self.included.len() < EVIDENCE_MAX_ENTRIES {
            self.included.push(Self::entry(candidate, None));
        }
    }

    fn record_excluded(&mut self, candidate: &MemoryCandidate, reason: &str) {
        if self.excluded.len() < EVIDENCE_MAX_ENTRIES {
            self.excluded.push(Self::entry(candidate, Some(reason)));
        }
    }

    fn record_plain_exclusion(&mut self, reason: &str) {
        if self.excluded.len() < EVIDENCE_MAX_ENTRIES {
            self.excluded.push(serde_json::json!({
                "reason": bound_reason(reason),
            }));
        }
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "provider": self.provider_id,
            "adapter": self.adapter,
            "queryHash": self.query_hash,
            "included": self.included,
            "excluded": self.excluded,
        })
    }
}
#[allow(clippy::too_many_arguments)]
pub async fn prepare_run(
    conn: &DatabaseConnection,
    folder_id: i32,
    config_root: &Path,
    source_root: &Path,
    task_id: i32,
    run_seq: i32,
    requested_loadout: Option<&str>,
    engine_items: Vec<EngineContextItem>,
    memory: &crate::memory::MemoryService,
) -> Result<PreparedContext, DbError> {
    if let Some(existing) = package_for_run(conn, task_id, run_seq).await? {
        return Ok(PreparedContext {
            prompt: render_prompt(&existing),
            package: existing,
        });
    }
    let config = crate::specos_control::load_context(config_root)?;
    if !config.validation_errors.is_empty() {
        return Err(DbError::Validation(config.validation_errors.join("; ")));
    }
    let loadout_id = requested_loadout.unwrap_or(&config.default_loadout_id);
    let loadout = config
        .loadouts
        .iter()
        .find(|l| l.id == loadout_id)
        .ok_or_else(|| {
            DbError::Validation(format!("context loadout '{loadout_id}' does not exist"))
        })?;

    let selected = loadout.provider_ids.iter().collect::<BTreeSet<_>>();
    let selected_providers = config
        .providers
        .iter()
        .filter(|p| selected.contains(&p.id))
        .cloned()
        .collect::<Vec<_>>();
    let health = check_provider_health(&selected_providers, memory, folder_id).await;
    for provider in config
        .providers
        .iter()
        .filter(|p| p.enabled && p.required && selected.contains(&p.id))
    {
        if health
            .iter()
            .find(|h| h.id == provider.id)
            .is_some_and(|h| h.status != "healthy")
        {
            record_activity(
                conn,
                folder_id,
                None,
                Some(&provider.id),
                "provider",
                "blocked",
                Some("required provider is unavailable"),
            )
            .await?;
            return Err(DbError::Validation(format!(
                "required context provider '{}' is unavailable",
                provider.id
            )));
        }
    }

    // Recall uses only the bounded task title. It never sends the compiled
    // prompt, repository files, tool output or credentials to the provider.
    let memory_query = work_task::Entity::find_by_id(task_id)
        .one(conn)
        .await?
        .map(|task| task.title.trim().chars().take(2_000).collect::<String>())
        .unwrap_or_default();
    let memory_query_hash = digest(memory_query.as_bytes());
    let mut memory_candidates: Vec<MemoryCandidate> = Vec::new();
    let mut evidence: Vec<MemoryProviderEvidence> = Vec::new();
    if !memory_query.is_empty() {
        for provider in selected_providers
            .iter()
            .filter(|provider| provider.kind == crate::memory::MEMORY_KIND && provider.enabled)
        {
            let recall_capable = provider.recall_enabled
                && provider
                    .capabilities
                    .iter()
                    .any(|capability| capability == crate::memory::CAP_RECALL_L1);
            let slot = evidence.len();
            evidence.push(MemoryProviderEvidence {
                provider_id: provider.id.clone(),
                adapter: provider.adapter.clone().unwrap_or_default(),
                query_hash: memory_query_hash.clone(),
                included: Vec::new(),
                excluded: Vec::new(),
            });
            if !recall_capable {
                evidence[slot].record_plain_exclusion("not_configured");
                continue;
            }
            let (resolved, adapter) = match memory.adapter_for(provider, None) {
                Ok(pair) => pair,
                Err(error) => {
                    record_activity(
                        conn,
                        folder_id,
                        None,
                        Some(&provider.id),
                        "memory.recall",
                        "degraded",
                        Some(error.class.key()),
                    )
                    .await?;
                    let reason = match error.class {
                        crate::memory::MemoryErrorClass::ConfigInvalid
                        | crate::memory::MemoryErrorClass::IdentityMissing => {
                            "not_configured".to_string()
                        }
                        class => format!("adapter_error:{}", class.key()),
                    };
                    evidence[slot].record_plain_exclusion(&reason);
                    if provider.required {
                        return Err(DbError::Validation(format!(
                            "required Memory provider '{}' cannot be resolved",
                            provider.id
                        )));
                    }
                    continue;
                }
            };
            let request = crate::memory::MemoryRecallRequest {
                team_id: resolved.team_id.clone(),
                agent_id: resolved.agent_id.clone(),
                user_id: resolved.user_id.clone(),
                query: memory_query.clone(),
                limit: resolved.recall_limit,
                include_core: resolved.can_recall_core(),
            };
            match adapter.recall(&request, resolved.timeout).await {
                Ok(result) => {
                    // Fixed package order (spec §6): L3 Core first, then L1
                    // ordered by score descending with remote ID as the
                    // deterministic tie-break.
                    let mut l1 = result.l1;
                    l1.sort_by(|a, b| {
                        b.score
                            .unwrap_or(0.0)
                            .partial_cmp(&a.score.unwrap_or(0.0))
                            .unwrap_or(std::cmp::Ordering::Equal)
                            .then_with(|| a.remote_id.cmp(&b.remote_id))
                    });
                    let mut hits = Vec::new();
                    if resolved.can_recall_core() {
                        hits.extend(result.l3);
                    }
                    hits.extend(l1);
                    if hits.is_empty() {
                        evidence[slot].record_plain_exclusion("empty");
                    }
                    let mut included_count = 0usize;
                    for hit in hits {
                        let layer = match hit.layer {
                            crate::memory::MemoryLayer::L1 => "l1",
                            crate::memory::MemoryLayer::L3 => "l3",
                        };
                        let wrapper = MemoryCandidate {
                            candidate: PackCandidate {
                                kind: "memory".into(),
                                source: format!(
                                    "{}/{}",
                                    crate::memory::ADAPTER_TENCENTDB_V3,
                                    layer
                                ),
                                title: format!("TencentDB Memory {layer}"),
                                hash: digest(hit.content.as_bytes()),
                                content: hit.content,
                                required: false,
                                provenance: serde_json::json!({
                                    "provider": provider.id,
                                    "adapter": crate::memory::ADAPTER_TENCENTDB_V3,
                                    "layer": layer,
                                    "remoteId": hit.remote_id,
                                    "score": hit.score,
                                    "capturedAt": hit.captured_at,
                                    "queryHash": memory_query_hash.clone(),
                                }),
                            },
                            evidence: slot,
                            layer,
                            remote_id: hit.remote_id,
                            score: hit.score,
                        };
                        if wrapper.candidate.content.trim().is_empty() {
                            // The candidate keeps remote id/layer metadata so
                            // the exclusion stays attributable; content
                            // itself is dropped.
                            evidence[slot].record_excluded(&wrapper, "empty");
                            continue;
                        }
                        included_count += 1;
                        memory_candidates.push(wrapper);
                    }
                    record_activity(
                        conn,
                        folder_id,
                        None,
                        Some(&provider.id),
                        "memory.recall",
                        "ready",
                        Some(&format!("{included_count} memory hits")),
                    )
                    .await?;
                }
                Err(error) => {
                    record_activity(
                        conn,
                        folder_id,
                        None,
                        Some(&provider.id),
                        "memory.recall",
                        "degraded",
                        Some(error.class.key()),
                    )
                    .await?;
                    evidence[slot]
                        .record_plain_exclusion(&format!("adapter_error:{}", error.class.key()));
                    if provider.required {
                        return Err(DbError::Validation(format!(
                            "required Memory provider '{}' recall failed",
                            provider.id
                        )));
                    }
                }
            }
        }
    }

    let canonical_root = std::fs::canonicalize(source_root).map_err(DbError::Io)?;
    let now = Utc::now();
    let mut candidates: Vec<PackCandidate> = Vec::new();
    let mut total = 0usize;
    let mut seen = BTreeSet::new();
    for source in &loadout.sources {
        if candidates.len() >= loadout.max_items {
            break;
        }
        let relative = Path::new(&source.path);
        let path = canonical_root.join(relative);
        let canonical = match std::fs::canonicalize(&path) {
            Ok(value) if value.starts_with(&canonical_root) => value,
            Ok(_) => {
                if source.required {
                    return Err(DbError::Validation(format!(
                        "required context source '{}' escapes the project",
                        source.path
                    )));
                }
                continue;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound && !source.required => continue,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(DbError::Validation(format!(
                    "required context source '{}' is missing",
                    source.path
                )))
            }
            Err(e) => return Err(DbError::Io(e)),
        };
        let meta = std::fs::metadata(&canonical)?;
        if !meta.is_file() {
            continue;
        }
        let remaining = loadout.max_bytes.saturating_sub(total);
        if remaining == 0 {
            break;
        }
        let bytes = std::fs::read(&canonical)?;
        if bytes.len() > remaining {
            if source.required {
                return Err(DbError::Validation(format!(
                    "required context source '{}' exceeds the remaining budget",
                    source.path
                )));
            }
            continue;
        }
        let content = match String::from_utf8(bytes) {
            Ok(value) => value,
            Err(_) if source.required => {
                return Err(DbError::Validation(format!(
                    "required context source '{}' is not UTF-8 text",
                    source.path
                )))
            }
            Err(_) => continue,
        };
        let hash = digest(content.as_bytes());
        if !seen.insert(hash.clone()) {
            continue;
        }
        total += content.len();
        candidates.push(PackCandidate {
            kind: source.kind.clone(),
            source: source.path.clone(),
            title: source.path.clone(),
            content,
            hash,
            required: source.required,
            provenance: serde_json::json!({
                "provider": "local",
                "path": source.path,
                "capturedAt": now,
            }),
        });
    }

    // Engine-computed extras (currently the Code Intelligence snapshot for
    // agents that cannot query the index live over MCP). Appended after the
    // file sources and subject to the same budgets: an item that does not fit
    // the remaining byte/token budget is skipped, never an error — a degraded
    // snapshot must not block the run.
    for item in engine_items {
        if candidates.len() >= loadout.max_items {
            break;
        }
        let remaining = loadout.max_bytes.saturating_sub(total);
        if item.content.len() > remaining {
            continue;
        }
        if (total + item.content.len()).div_ceil(4) > loadout.max_tokens {
            continue;
        }
        let hash = digest(item.content.as_bytes());
        if !seen.insert(hash.clone()) {
            continue;
        }
        total += item.content.len();
        candidates.push(PackCandidate {
            kind: item.kind,
            source: item.source,
            title: item.title,
            content: item.content,
            hash,
            required: false,
            provenance: item.provenance,
        });
    }

    // Remote Memory is appended after local files and engine extras, sharing
    // the same loadout budgets and deduplication rules. Every adjudication
    // is recorded as included/excluded evidence (R05): local content wins
    // dedup, and budget rejects carry a bounded reason.
    for wrapper in memory_candidates {
        if candidates.len() >= loadout.max_items {
            evidence[wrapper.evidence].record_excluded(&wrapper, "budget_items");
            continue;
        }
        let content_len = wrapper.candidate.content.len();
        if content_len > loadout.max_bytes {
            evidence[wrapper.evidence].record_excluded(&wrapper, "oversized");
            continue;
        }
        let remaining = loadout.max_bytes.saturating_sub(total);
        if content_len > remaining {
            evidence[wrapper.evidence].record_excluded(&wrapper, "budget_bytes");
            continue;
        }
        if (total + content_len).div_ceil(4) > loadout.max_tokens {
            evidence[wrapper.evidence].record_excluded(&wrapper, "budget_tokens");
            continue;
        }
        if !seen.insert(wrapper.candidate.hash.clone()) {
            evidence[wrapper.evidence].record_excluded(&wrapper, "duplicate");
            continue;
        }
        total += content_len;
        evidence[wrapper.evidence].record_included(&wrapper);
        candidates.push(wrapper.candidate);
    }

    let estimated_tokens = total.div_ceil(4);
    if estimated_tokens > loadout.max_tokens {
        return Err(DbError::Validation(format!(
            "context package requires about {estimated_tokens} tokens, budget is {}",
            loadout.max_tokens
        )));
    }

    let package_id = format!("ctx-{}", uuid::Uuid::new_v4().simple());
    let mut package_hasher = Sha256::new();
    for candidate in &candidates {
        package_hasher.update(candidate.hash.as_bytes());
    }
    let content_hash = format!("{:x}", package_hasher.finalize());
    let provider_status = serde_json::to_value(&health).unwrap_or_else(|_| serde_json::json!([]));
    // Recall evidence: deterministic (config provider order, fixed hit
    // order, capped arrays) so identical recall inputs keep the package
    // stable. `None` only when no Memory provider participated.
    let memory_evidence = (!evidence.is_empty()).then(|| {
        serde_json::json!({ "providers": evidence.iter().map(MemoryProviderEvidence::to_json).collect::<Vec<_>>() }).to_string()
    });
    // Memory is an optional enhancement by default: an unavailable remote
    // Memory service must not make the local file Context Package unusable.
    // Required providers and legacy remote providers retain the existing
    // degraded status behavior.
    let status = if health.iter().any(|h| {
        h.status == "degraded"
            && selected_providers
                .iter()
                .find(|provider| provider.id == h.id)
                .is_none_or(|provider| provider.kind != crate::memory::MEMORY_KIND)
    }) {
        "degraded"
    } else {
        "ready"
    };
    let txn = conn.begin().await?;
    work_task_context_pack::ActiveModel {
        id: Set(package_id.clone()),
        task_id: Set(task_id),
        run_seq: Set(run_seq),
        loadout_id: Set(loadout.id.clone()),
        status: Set(status.into()),
        content_hash: Set(content_hash.clone()),
        estimated_tokens: Set(estimated_tokens as i32),
        total_bytes: Set(total as i32),
        provider_status: Set(provider_status.to_string()),
        memory_evidence: Set(memory_evidence),
        created_at: Set(now),
    }
    .insert(&txn)
    .await?;
    let mut items = Vec::new();
    for (ordinal, candidate) in candidates.into_iter().enumerate() {
        let id = format!("ctxi-{}", uuid::Uuid::new_v4().simple());
        let provenance = candidate.provenance;
        work_task_context_item::ActiveModel {
            id: Set(id.clone()),
            package_id: Set(package_id.clone()),
            ordinal: Set(ordinal as i32),
            kind: Set(candidate.kind.clone()),
            source: Set(candidate.source.clone()),
            title: Set(candidate.title.clone()),
            content: Set(candidate.content.clone()),
            content_hash: Set(candidate.hash.clone()),
            required: Set(candidate.required),
            provenance: Set(provenance.to_string()),
        }
        .insert(&txn)
        .await?;
        items.push(ContextItemInfo {
            id,
            ordinal: ordinal as i32,
            kind: candidate.kind,
            source: candidate.source,
            title: candidate.title,
            content: candidate.content,
            content_hash: candidate.hash,
            required: candidate.required,
            provenance,
        });
    }
    specos_runtime_service::bind_context_package(&txn, task_id, run_seq, &package_id).await?;
    txn.commit().await?;
    record_activity(
        conn,
        folder_id,
        Some(&package_id),
        None,
        "package",
        status,
        Some(&format!("{} items, {} bytes", items.len(), total)),
    )
    .await?;
    let package = ContextPackageInfo {
        id: package_id,
        task_id,
        run_seq,
        loadout_id: loadout.id.clone(),
        status: status.into(),
        content_hash,
        estimated_tokens: estimated_tokens as i32,
        total_bytes: total as i32,
        provider_status,
        items,
        created_at: now,
    };
    Ok(PreparedContext {
        prompt: render_prompt(&package),
        package,
    })
}

pub async fn package_for_run(
    conn: &DatabaseConnection,
    task_id: i32,
    run_seq: i32,
) -> Result<Option<ContextPackageInfo>, DbError> {
    let row = work_task_context_pack::Entity::find()
        .filter(work_task_context_pack::Column::TaskId.eq(task_id))
        .filter(work_task_context_pack::Column::RunSeq.eq(run_seq))
        .one(conn)
        .await?;
    match row {
        Some(row) => Ok(Some(package_info(conn, row).await?)),
        None => Ok(None),
    }
}

pub async fn package_get(
    conn: &DatabaseConnection,
    id: &str,
) -> Result<ContextPackageInfo, DbError> {
    let row = work_task_context_pack::Entity::find_by_id(id)
        .one(conn)
        .await?
        .ok_or_else(|| DbError::NotFound(format!("context package {id}")))?;
    package_info(conn, row).await
}

async fn package_info(
    conn: &DatabaseConnection,
    row: work_task_context_pack::Model,
) -> Result<ContextPackageInfo, DbError> {
    let items = work_task_context_item::Entity::find()
        .filter(work_task_context_item::Column::PackageId.eq(&row.id))
        .order_by_asc(work_task_context_item::Column::Ordinal)
        .all(conn)
        .await?
        .into_iter()
        .map(|item| ContextItemInfo {
            id: item.id,
            ordinal: item.ordinal,
            kind: item.kind,
            source: item.source,
            title: item.title,
            content: item.content,
            content_hash: item.content_hash,
            required: item.required,
            provenance: serde_json::from_str(&item.provenance)
                .unwrap_or_else(|_| serde_json::json!({})),
        })
        .collect();
    Ok(ContextPackageInfo {
        id: row.id,
        task_id: row.task_id,
        run_seq: row.run_seq,
        loadout_id: row.loadout_id,
        status: row.status,
        content_hash: row.content_hash,
        estimated_tokens: row.estimated_tokens,
        total_bytes: row.total_bytes,
        provider_status: serde_json::from_str(&row.provider_status)
            .unwrap_or_else(|_| serde_json::json!([])),
        items,
        created_at: row.created_at,
    })
}

pub async fn overview(
    conn: &DatabaseConnection,
    folder_id: i32,
    root: &Path,
    memory: &crate::memory::MemoryService,
) -> Result<ContextOverview, DbError> {
    let config = crate::specos_control::load_context(root)?;
    let provider_health = check_provider_health(&config.providers, memory, folder_id).await;
    let rows = work_task_context_pack::Entity::find()
        .inner_join(work_task::Entity)
        .filter(work_task::Column::FolderId.eq(folder_id))
        .filter(work_task::Column::DeletedAt.is_null())
        .order_by_desc(work_task_context_pack::Column::CreatedAt)
        .limit(20)
        .all(conn)
        .await?;
    let mut packages = Vec::new();
    for row in rows {
        packages.push(package_info(conn, row).await?);
    }
    let activity = context_activity::Entity::find()
        .filter(context_activity::Column::FolderId.eq(folder_id))
        .order_by_desc(context_activity::Column::Id)
        .limit(100)
        .all(conn)
        .await?
        .into_iter()
        .map(activity_info)
        .collect();
    Ok(ContextOverview {
        config,
        provider_health,
        packages,
        activity,
    })
}

/// Provider health gate. Memory providers delegate to the Memory module
/// (`GET /health` on the pinned Adapter plus the version/writability gate,
/// shared 30s cache). Other non-local providers are probed with the public
/// MemoryCore-style `GET /health` — `GET /v3/tools/list` was never a health
/// probe: it belongs to the Knowledge service and is a POST there
/// (issue-054).
pub async fn check_provider_health(
    providers: &[ContextProviderConfig],
    memory: &crate::memory::MemoryService,
    folder_id: i32,
) -> Vec<ContextProviderHealth> {
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .ok();
    let mut out = Vec::new();
    for provider in providers {
        let checked_at = Utc::now();
        if !provider.enabled {
            out.push(ContextProviderHealth {
                id: provider.id.clone(),
                kind: provider.kind.clone(),
                status: "disabled".into(),
                message: None,
                checked_at,
            });
            continue;
        }
        if provider.kind == "local" {
            out.push(ContextProviderHealth {
                id: provider.id.clone(),
                kind: provider.kind.clone(),
                status: "healthy".into(),
                message: None,
                checked_at,
            });
            continue;
        }
        if provider.kind == "code-intelligence" {
            // Locally managed adapter provider. Validation forbids
            // `required: true` for this kind, so availability never gates a
            // run; real index state is surfaced through the Code
            // Intelligence state API and Context Pack summary items rather
            // than by probing the adapter here (health checks must not
            // spawn processes or trigger downloads).
            out.push(ContextProviderHealth {
                id: provider.id.clone(),
                kind: provider.kind.clone(),
                status: "healthy".into(),
                message: None,
                checked_at,
            });
            continue;
        }
        if provider.kind == crate::memory::MEMORY_KIND {
            out.push(memory.provider_health(folder_id, provider, false).await);
            continue;
        }
        let Some(endpoint) = provider.endpoint.as_deref() else {
            out.push(ContextProviderHealth {
                id: provider.id.clone(),
                kind: provider.kind.clone(),
                status: "degraded".into(),
                message: Some("endpoint is not configured".into()),
                checked_at,
            });
            continue;
        };
        let Some(client) = &client else {
            out.push(ContextProviderHealth {
                id: provider.id.clone(),
                kind: provider.kind.clone(),
                status: "degraded".into(),
                message: Some("HTTP client is unavailable".into()),
                checked_at,
            });
            continue;
        };
        let url = format!("{}/health", endpoint.trim_end_matches('/'));
        let mut request = client.get(url);
        if let Some(env_name) = &provider.secret_env {
            if let Ok(secret) = std::env::var(env_name) {
                request = request.bearer_auth(secret);
            }
        }
        match request.send().await {
            Ok(response) if response.status().is_success() => out.push(ContextProviderHealth {
                id: provider.id.clone(),
                kind: provider.kind.clone(),
                status: "healthy".into(),
                message: None,
                checked_at,
            }),
            Ok(response) => out.push(ContextProviderHealth {
                id: provider.id.clone(),
                kind: provider.kind.clone(),
                status: "degraded".into(),
                message: Some(format!("health endpoint returned {}", response.status())),
                checked_at,
            }),
            Err(error) => out.push(ContextProviderHealth {
                id: provider.id.clone(),
                kind: provider.kind.clone(),
                status: "degraded".into(),
                message: Some(error.to_string()),
                checked_at,
            }),
        }
    }
    out
}

fn render_prompt(package: &ContextPackageInfo) -> String {
    let mut out = format!(
        "SpecOS Context Package {} (loadout {}, immutable for this run):\n",
        package.id, package.loadout_id
    );
    for item in &package.items {
        out.push_str(&format!(
            "\n--- {} [{}] ---\n{}\n",
            item.title, item.content_hash, item.content
        ));
    }
    out
}

/// Context Activity row shared by the Context plane and the Memory capture
/// worker. Messages must stay safe: IDs, counts and error classes only —
/// never payload content or credentials (BUGRAIL-SPECOS-017 §9).
pub async fn record_activity(
    conn: &DatabaseConnection,
    folder_id: i32,
    package_id: Option<&str>,
    provider_id: Option<&str>,
    kind: &str,
    status: &str,
    message: Option<&str>,
) -> Result<(), DbError> {
    context_activity::ActiveModel {
        id: NotSet,
        folder_id: Set(folder_id),
        package_id: Set(package_id.map(str::to_string)),
        provider_id: Set(provider_id.map(str::to_string)),
        kind: Set(kind.into()),
        status: Set(status.into()),
        message: Set(message.map(str::to_string)),
        created_at: Set(Utc::now()),
    }
    .insert(conn)
    .await?;
    Ok(())
}

fn activity_info(row: context_activity::Model) -> ContextActivityInfo {
    ContextActivityInfo {
        id: row.id,
        folder_id: row.folder_id,
        package_id: row.package_id,
        provider_id: row.provider_id,
        kind: row.kind,
        status: row.status,
        message: row.message,
        created_at: row.created_at,
    }
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
