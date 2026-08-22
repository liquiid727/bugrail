//! Deterministic in-memory Memory Adapter (BUGRAIL-SPECOS-017 §2).
//!
//! Internal to contract and command-core tests: verifies the Memory
//! interface, capture/recall lifecycle and command parity without any HTTP
//! transport. State is shared through an `Arc` so tests can seed recall
//! hits, force error classes and inspect exactly what was captured.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::{
    MemoryCaptureBatch, MemoryCaptureReceipt, MemoryError, MemoryErrorClass, MemoryHealthReport,
    MemoryHealthStatus, MemoryLayer, MemoryProvider, MemoryRecallHit, MemoryRecallRequest,
    MemoryRecallResult, ADAPTER_TENCENTDB_V3,
};

#[derive(Debug, Default)]
pub struct DeterministicState {
    /// `/health` outcome.
    pub healthy: bool,
    /// Version reported through `/health`.
    pub version: String,
    /// Fail health with this class instead of answering.
    pub fail_health: Option<MemoryErrorClass>,
    /// Fail capture with this class.
    pub fail_capture: Option<MemoryErrorClass>,
    /// Fail recall with this class.
    pub fail_recall: Option<MemoryErrorClass>,
    /// Fail the connection-test probe with this class.
    pub fail_probe: Option<MemoryErrorClass>,
    /// Captured batches in arrival order — replayed message ids are
    /// upserted, mirroring the patched Gateway contract.
    pub captured: Vec<MemoryCaptureBatch>,
    /// Seed L1 hits.
    pub l1: Vec<MemoryRecallHit>,
    /// Seed L3 hits.
    pub l3: Vec<MemoryRecallHit>,
    /// Probe invocations (connection test).
    pub probes: u32,
    /// Recall invocations.
    pub recalls: u32,
}

impl DeterministicState {
    fn writable(&self) -> bool {
        self.version == super::EXPECTED_UPSTREAM_VERSION
    }
}

#[derive(Debug, Clone, Default)]
pub struct DeterministicMemoryAdapter {
    state: Arc<Mutex<DeterministicState>>,
}

impl DeterministicMemoryAdapter {
    pub fn new() -> Self {
        let state = DeterministicState {
            healthy: true,
            version: super::EXPECTED_UPSTREAM_VERSION.to_string(),
            ..Default::default()
        };
        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }

    pub fn state(&self) -> std::sync::MutexGuard<'_, DeterministicState> {
        self.state.lock().expect("test adapter state")
    }

    /// Total L0 message count stored across captured batches (replay-safe).
    pub fn l0_count(&self) -> usize {
        let state = self.state();
        state
            .captured
            .iter()
            .flat_map(|batch| batch.messages.iter().map(|message| message.id.clone()))
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    }
}

#[async_trait::async_trait]
impl MemoryProvider for DeterministicMemoryAdapter {
    fn adapter_id(&self) -> &str {
        ADAPTER_TENCENTDB_V3
    }

    async fn health(&self) -> Result<MemoryHealthReport, MemoryError> {
        let state = self.state.lock().expect("test adapter state");
        if let Some(class) = state.fail_health {
            return Err(MemoryError::new(class, "forced health failure"));
        }
        if !state.healthy {
            return Ok(MemoryHealthReport {
                status: MemoryHealthStatus::Degraded,
                version: None,
                writable: false,
                error_class: Some(MemoryErrorClass::Unavailable),
                message: Some("forced degraded health".into()),
                latency_ms: Some(0),
                trace_id: None,
            });
        }
        let writable = state.writable();
        Ok(MemoryHealthReport {
            status: MemoryHealthStatus::Healthy,
            version: Some(state.version.clone()),
            writable,
            error_class: None,
            message: None,
            latency_ms: Some(0),
            trace_id: None,
        })
    }

    async fn capture(
        &self,
        batch: &MemoryCaptureBatch,
    ) -> Result<MemoryCaptureReceipt, MemoryError> {
        let mut state = self.state.lock().expect("test adapter state");
        if let Some(class) = state.fail_capture {
            return Err(MemoryError::new(class, "forced capture failure"));
        }
        // Patched upsert contract: replaying identical message ids returns
        // the same accepted ids and adds no new L0 rows.
        let known = state
            .captured
            .iter()
            .flat_map(|known| known.messages.iter().map(|message| message.id.clone()))
            .collect::<std::collections::BTreeSet<_>>();
        let is_replay = batch
            .messages
            .iter()
            .all(|message| known.contains(&message.id));
        if !is_replay {
            state.captured.push(batch.clone());
        }
        Ok(MemoryCaptureReceipt {
            accepted_ids: batch
                .messages
                .iter()
                .map(|message| message.id.clone())
                .collect(),
            trace_id: Some("test-trace".into()),
        })
    }

    async fn recall(
        &self,
        request: &MemoryRecallRequest,
        _deadline: Duration,
    ) -> Result<MemoryRecallResult, MemoryError> {
        let mut state = self.state.lock().expect("test adapter state");
        state.recalls += 1;
        if let Some(class) = state.fail_recall {
            return Err(MemoryError::new(class, "forced recall failure"));
        }
        let limit = request.limit.max(1) as usize;
        let mut l1: Vec<MemoryRecallHit> = state
            .l1
            .iter()
            .filter(|hit| hit.layer == MemoryLayer::L1)
            .cloned()
            .collect();
        l1.sort_by(|a, b| {
            b.score
                .unwrap_or(0.0)
                .partial_cmp(&a.score.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.remote_id.cmp(&b.remote_id))
        });
        l1.truncate(limit);
        let l3 = if request.include_core {
            state.l3.clone()
        } else {
            Vec::new()
        };
        Ok(MemoryRecallResult { l1, l3 })
    }

    async fn probe_read(&self, _request: &MemoryRecallRequest) -> Result<(), MemoryError> {
        let mut state = self.state.lock().expect("test adapter state");
        state.probes += 1;
        if let Some(class) = state.fail_probe {
            return Err(MemoryError::new(class, "forced probe failure"));
        }
        Ok(())
    }
}
