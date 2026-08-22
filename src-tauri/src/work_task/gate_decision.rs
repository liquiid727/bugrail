//! Pure gate-policy evaluation and limit validation for SpecOS contract-bound
//! WorkTasks. No DB, no transport, no engine state: given a snapshotted policy
//! and the persisted attempt rows, compute the explainable merge/complete
//! decision (Feature Spec §5.3). The WorkTask module owns loading attempts and
//! applying the decision; correctness never depends on live event delivery.

use std::collections::HashMap;

use crate::models::{
    AcceptanceCriterionSnapshot, GateDecisionItem, GateRequirement, WorkTaskGateDecision,
    WorkTaskGatePolicy, WorkTaskGateStatus, WorkTaskGateType,
};

// Security and limits (Feature Spec §6). Binding validates before any row is
// written; `evaluate` re-checks the rules that can change with runtime facts
// (reusable-head validity, waiver policy).
pub const MAX_SPEC_BYTES: usize = 1024 * 1024;
pub const MAX_AC_ITEMS: usize = 64;
pub const MAX_AC_SNAPSHOT_BYTES: usize = 64 * 1024;
pub const MAX_GATE_COUNT: usize = 32;
pub const MAX_GATE_POLICY_BYTES: usize = 32 * 1024;
pub const MAX_EVIDENCE_BYTES: usize = 64 * 1024;

/// A gate attempt as seen by the evaluator. Structurally mirrors a
/// `work_task_gate_result` row; the service converts rows into this.
#[derive(Debug, Clone)]
pub struct GateAttemptInput {
    /// Row id — the tie-breaker for "latest attempt" within a run.
    pub id: i64,
    pub run_seq: i32,
    pub gate_id: String,
    pub gate_type: WorkTaskGateType,
    pub status: WorkTaskGateStatus,
    pub required: bool,
    pub reusable: bool,
    pub actor: String,
    pub reason: Option<String>,
    pub evidence: Option<serde_json::Value>,
}

impl GateAttemptInput {
    /// Evidence `verified_head` (the Worktree HEAD a reusable preflight ran on).
    fn verified_head(&self) -> Option<&str> {
        self.evidence
            .as_ref()
            .and_then(|e| e.get("verified_head"))
            .and_then(|v| v.as_str())
    }
}

/// Validates a gate policy snapshot before persistence. Errors carry the reason
/// surfaced through `InvalidInput` / `workTask.specContract.invalid`.
pub fn validate_gate_policy(policy: &WorkTaskGatePolicy) -> Result<(), String> {
    if policy.gates.len() > MAX_GATE_COUNT {
        return Err(format!(
            "gate policy exceeds {MAX_GATE_COUNT} gates ({} given)",
            policy.gates.len()
        ));
    }
    let mut seen = std::collections::HashSet::new();
    for gate in &policy.gates {
        if gate.id.trim().is_empty() {
            return Err("gate id cannot be empty".to_string());
        }
        if !seen.insert(gate.id.as_str()) {
            return Err(format!("duplicate gate id: {}", gate.id));
        }
        // `WorkTaskGateType` is exhaustive for the first slice: an unsupported
        // type cannot be deserialized into it, so no extra check is possible.
        if gate.gate_type == WorkTaskGateType::HumanApproval && gate.reusable {
            return Err(format!(
                "human_approval gate {} cannot be reusable",
                gate.id
            ));
        }
    }
    let size = serde_json::to_vec(policy).map_err(|e| format!("serialize gate policy: {e}"))?;
    if size.len() > MAX_GATE_POLICY_BYTES {
        return Err(format!(
            "gate policy exceeds {} bytes ({} serialized)",
            MAX_GATE_POLICY_BYTES,
            size.len()
        ));
    }
    Ok(())
}

/// Validates an acceptance-criteria snapshot before persistence.
pub fn validate_acceptance_criteria_snapshot(
    items: &[AcceptanceCriterionSnapshot],
) -> Result<(), String> {
    if items.len() > MAX_AC_ITEMS {
        return Err(format!(
            "acceptance criteria exceed {MAX_AC_ITEMS} items ({} given)",
            items.len()
        ));
    }
    for item in items {
        if item.id.trim().is_empty() {
            return Err("acceptance criterion id cannot be empty".to_string());
        }
    }
    let size = serde_json::to_vec(items).map_err(|e| format!("serialize AC snapshot: {e}"))?;
    if size.len() > MAX_AC_SNAPSHOT_BYTES {
        return Err(format!(
            "acceptance criteria snapshot exceeds {} bytes ({} serialized)",
            MAX_AC_SNAPSHOT_BYTES,
            size.len()
        ));
    }
    Ok(())
}

/// Validates an evidence payload before it is persisted.
pub fn validate_evidence(evidence: &serde_json::Value) -> Result<(), String> {
    let size = serde_json::to_vec(evidence).map_err(|e| format!("serialize evidence: {e}"))?;
    if size.len() > MAX_EVIDENCE_BYTES {
        return Err(format!(
            "gate evidence exceeds {} bytes ({} serialized)",
            MAX_EVIDENCE_BYTES,
            size.len()
        ));
    }
    Ok(())
}

/// Compute the merge/complete decision from persisted facts.
///
/// Eligibility (Feature Spec §5.3):
/// - the bound Spec still matches the current file (`spec_stale` false);
/// - every required gate has a latest applicable attempt for `current_run_seq`,
///   or a reusable passing attempt allowed by policy;
/// - every applicable attempt is `passed` or validly `waived`;
/// - no required gate is `running`, `failed`, or `blocked`.
///
/// A reusable preflight result is applicable only while its evidence
/// `verified_head` equals `current_head` and the bound Spec hash is unchanged.
pub fn evaluate(
    policy: &WorkTaskGatePolicy,
    attempts: &[GateAttemptInput],
    current_run_seq: i32,
    spec_stale: bool,
    current_head: Option<&str>,
) -> WorkTaskGateDecision {
    // Index attempts by gate_id, newest first (id desc within run_seq desc).
    let mut by_gate: HashMap<&str, Vec<&GateAttemptInput>> = HashMap::new();
    for attempt in attempts {
        by_gate
            .entry(attempt.gate_id.as_str())
            .or_default()
            .push(attempt);
    }
    for list in by_gate.values_mut() {
        list.sort_by(|a, b| b.run_seq.cmp(&a.run_seq).then(b.id.cmp(&a.id)));
    }

    let mut required = Vec::new();
    let mut unmet = Vec::new();
    let mut waived = Vec::new();

    for gate in &policy.gates {
        if !gate.required {
            continue;
        }
        let (item, ok) = evaluate_gate(
            gate,
            by_gate.get(gate.id.as_str()).map(|v| v.as_slice()),
            current_run_seq,
            spec_stale,
            current_head,
        );
        let is_waived = item.status == Some(WorkTaskGateStatus::Waived);
        if is_waived && ok {
            waived.push(item.clone());
        }
        if !ok {
            unmet.push(item.clone());
        }
        required.push(item);
    }

    WorkTaskGateDecision {
        eligible: !spec_stale && unmet.is_empty(),
        stale_spec: spec_stale,
        required,
        unmet,
        waived,
    }
}

/// Evaluate one required gate. Returns the decision item and whether it is
/// currently eligible.
fn evaluate_gate(
    gate: &GateRequirement,
    attempts: Option<&[&GateAttemptInput]>,
    current_run_seq: i32,
    spec_stale: bool,
    current_head: Option<&str>,
) -> (GateDecisionItem, bool) {
    let attempts = attempts.unwrap_or(&[]);
    // Newest attempt of this gate in the current run.
    let current = attempts.iter().find(|a| a.run_seq == current_run_seq);

    match current {
        Some(attempt) => {
            let item = GateDecisionItem {
                gate_id: gate.id.clone(),
                gate_type: gate.gate_type,
                status: Some(attempt.status),
                reason: attempt
                    .reason
                    .clone()
                    .unwrap_or_else(|| attempt_status_text(attempt.status)),
            };
            let ok = match attempt.status {
                WorkTaskGateStatus::Passed => {
                    // Reusable preflight still must satisfy the head/spec rule
                    // even in the current run (a return/rebase can move HEAD).
                    if attempt.gate_type == WorkTaskGateType::Preflight && attempt.reusable {
                        reusable_applicable(attempt, current_head) && !spec_stale
                    } else {
                        true
                    }
                }
                WorkTaskGateStatus::Waived => gate.allow_waiver,
                WorkTaskGateStatus::Running
                | WorkTaskGateStatus::Failed
                | WorkTaskGateStatus::Blocked => false,
            };
            (item, ok)
        }
        None => {
            // No attempt for the current run: fall back to a reusable passing
            // result from an earlier run, when policy allows.
            let reusable = attempts
                .iter()
                .filter(|a| a.status == WorkTaskGateStatus::Passed && a.reusable)
                .max_by_key(|a| (a.run_seq, a.id));

            if let Some(result) = reusable {
                if reusable_applicable(result, current_head) && !spec_stale {
                    (
                        GateDecisionItem {
                            gate_id: gate.id.clone(),
                            gate_type: gate.gate_type,
                            status: Some(WorkTaskGateStatus::Passed),
                            reason: format!(
                                "reusable result from run {} (verified_head matches)",
                                result.run_seq
                            ),
                        },
                        true,
                    )
                } else {
                    (
                        GateDecisionItem {
                            gate_id: gate.id.clone(),
                            gate_type: gate.gate_type,
                            status: None,
                            reason: if spec_stale {
                                "spec changed since the reusable result; rebind required"
                                    .to_string()
                            } else {
                                "reusable preflight no longer applicable (HEAD changed)".to_string()
                            },
                        },
                        false,
                    )
                }
            } else {
                (
                    GateDecisionItem {
                        gate_id: gate.id.clone(),
                        gate_type: gate.gate_type,
                        status: None,
                        reason: format!("no attempt for run {current_run_seq}"),
                    },
                    false,
                )
            }
        }
    }
}

fn attempt_status_text(status: WorkTaskGateStatus) -> String {
    status.as_str().to_string()
}

/// A reusable preflight result is applicable only while its evidence
/// `verified_head` equals the current Worktree HEAD.
fn reusable_applicable(attempt: &GateAttemptInput, current_head: Option<&str>) -> bool {
    if attempt.gate_type != WorkTaskGateType::Preflight {
        return false;
    }
    match (attempt.verified_head(), current_head) {
        (Some(verified), Some(head)) => verified == head,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(
        id: &str,
        gate_type: WorkTaskGateType,
        required: bool,
        reusable: bool,
        allow_waiver: bool,
    ) -> GateRequirement {
        GateRequirement {
            id: id.to_string(),
            gate_type,
            required,
            reusable,
            allow_waiver,
        }
    }

    fn attempt(
        id: i64,
        run_seq: i32,
        gate_id: &str,
        status: WorkTaskGateStatus,
        reusable: bool,
    ) -> GateAttemptInput {
        GateAttemptInput {
            id,
            run_seq,
            gate_id: gate_id.to_string(),
            gate_type: WorkTaskGateType::Preflight,
            status,
            required: true,
            reusable,
            actor: "engine".to_string(),
            reason: None,
            evidence: None,
        }
    }

    fn preflight(verified_head: &str) -> Option<serde_json::Value> {
        Some(serde_json::json!({ "verified_head": verified_head }))
    }

    #[test]
    fn policy_validation_rejects_duplicate_and_oversize_and_reusable_human() {
        let dup = WorkTaskGatePolicy {
            gates: vec![
                req("g", WorkTaskGateType::Preflight, true, false, false),
                req("g", WorkTaskGateType::Preflight, true, false, false),
            ],
        };
        assert!(validate_gate_policy(&dup)
            .unwrap_err()
            .contains("duplicate"));

        let reusable_human = WorkTaskGatePolicy {
            gates: vec![req("h", WorkTaskGateType::HumanApproval, true, true, false)],
        };
        assert!(validate_gate_policy(&reusable_human)
            .unwrap_err()
            .contains("cannot be reusable"));

        let too_many = WorkTaskGatePolicy {
            gates: (0..=MAX_GATE_COUNT as i32)
                .map(|i| {
                    req(
                        &format!("g{i}"),
                        WorkTaskGateType::Preflight,
                        true,
                        false,
                        false,
                    )
                })
                .collect(),
        };
        assert!(validate_gate_policy(&too_many)
            .unwrap_err()
            .contains("exceeds"));
    }

    #[test]
    fn ac_snapshot_validation_rejects_oversize() {
        let ok = vec![AcceptanceCriterionSnapshot {
            id: "AC01".into(),
            title: "t".into(),
            text: "x".into(),
        }];
        assert!(validate_acceptance_criteria_snapshot(&ok).is_ok());

        let many = (0..=MAX_AC_ITEMS as i32)
            .map(|i| AcceptanceCriterionSnapshot {
                id: format!("AC{i}"),
                title: "t".into(),
                text: "x".into(),
            })
            .collect::<Vec<_>>();
        assert!(validate_acceptance_criteria_snapshot(&many)
            .unwrap_err()
            .contains("exceed"));
    }

    #[test]
    fn missing_required_attempt_is_unmet() {
        let policy = WorkTaskGatePolicy {
            gates: vec![req(
                "preflight",
                WorkTaskGateType::Preflight,
                true,
                false,
                false,
            )],
        };
        let d = evaluate(&policy, &[], 3, false, None);
        assert!(!d.eligible);
        assert_eq!(d.unmet.len(), 1);
        assert!(d.unmet[0].reason.contains("no attempt for run 3"));
    }

    #[test]
    fn passed_current_run_is_eligible() {
        let policy = WorkTaskGatePolicy {
            gates: vec![req(
                "preflight",
                WorkTaskGateType::Preflight,
                true,
                false,
                false,
            )],
        };
        let a = attempt(1, 3, "preflight", WorkTaskGateStatus::Passed, false);
        let d = evaluate(&policy, &[a], 3, false, None);
        assert!(d.eligible);
        assert!(d.unmet.is_empty());
    }

    #[test]
    fn failed_and_blocked_and_running_are_unmet() {
        let policy = WorkTaskGatePolicy {
            gates: vec![req("g", WorkTaskGateType::Preflight, true, false, false)],
        };
        for status in [
            WorkTaskGateStatus::Failed,
            WorkTaskGateStatus::Blocked,
            WorkTaskGateStatus::Running,
        ] {
            let a = attempt(1, 3, "g", status, false);
            let d = evaluate(&policy, &[a], 3, false, None);
            assert!(!d.eligible);
            assert_eq!(d.unmet.len(), 1);
        }
    }

    #[test]
    fn retry_does_not_reuse_non_reusable_passing_result() {
        let policy = WorkTaskGatePolicy {
            gates: vec![req("g", WorkTaskGateType::Preflight, true, false, false)],
        };
        let old = attempt(1, 2, "g", WorkTaskGateStatus::Passed, false);
        let d = evaluate(&policy, &[old], 3, false, Some("head1"));
        assert!(!d.eligible);
        assert!(d.unmet[0].reason.contains("no attempt for run 3"));
    }

    #[test]
    fn reusable_preflight_applies_only_when_heads_and_spec_match() {
        let policy = WorkTaskGatePolicy {
            gates: vec![req("g", WorkTaskGateType::Preflight, true, true, false)],
        };
        let old = attempt(1, 2, "g", WorkTaskGateStatus::Passed, true);
        // Head matches + spec unchanged → eligible.
        let mut with_evidence = old.clone();
        with_evidence.evidence = preflight("head1");
        let d = evaluate(&policy, &[with_evidence.clone()], 3, false, Some("head1"));
        assert!(d.eligible, "head match should reuse");

        // Head changed → not applicable.
        let d = evaluate(&policy, &[with_evidence.clone()], 3, false, Some("head2"));
        assert!(!d.eligible);
        assert!(d.unmet[0].reason.contains("HEAD changed"));

        // Spec changed → not applicable, even with matching head.
        let d = evaluate(&policy, &[with_evidence], 3, true, Some("head1"));
        assert!(!d.eligible);
        assert!(d.unmet[0].reason.contains("rebind required"));
    }

    #[test]
    fn reusable_preflight_without_evidence_is_not_applicable() {
        let policy = WorkTaskGatePolicy {
            gates: vec![req("g", WorkTaskGateType::Preflight, true, true, false)],
        };
        let old = attempt(1, 2, "g", WorkTaskGateStatus::Passed, true); // no evidence
        let d = evaluate(&policy, &[old], 3, false, Some("head1"));
        assert!(!d.eligible);
    }

    #[test]
    fn stale_spec_marks_ineligible_even_when_gates_pass() {
        let policy = WorkTaskGatePolicy {
            gates: vec![req("g", WorkTaskGateType::Preflight, true, false, false)],
        };
        let a = attempt(1, 3, "g", WorkTaskGateStatus::Passed, false);
        let d = evaluate(&policy, &[a], 3, true, Some("head1"));
        assert!(!d.eligible);
        assert!(d.stale_spec);
        assert!(d.unmet.is_empty(), "gates pass but the spec is stale");
    }

    #[test]
    fn waiver_valid_only_when_policy_allows() {
        let allowed = WorkTaskGatePolicy {
            gates: vec![req("h", WorkTaskGateType::HumanApproval, true, false, true)],
        };
        let mut waived_attempt = attempt(1, 3, "h", WorkTaskGateStatus::Waived, false);
        waived_attempt.gate_type = WorkTaskGateType::HumanApproval;
        let d = evaluate(&allowed, &[waived_attempt.clone()], 3, false, None);
        assert!(d.eligible, "policy-allowed waiver passes");
        assert_eq!(d.waived.len(), 1);
        assert!(d.unmet.is_empty());

        let forbidden = WorkTaskGatePolicy {
            gates: vec![req(
                "h",
                WorkTaskGateType::HumanApproval,
                true,
                false,
                false,
            )],
        };
        let d = evaluate(&forbidden, &[waived_attempt], 3, false, None);
        assert!(!d.eligible, "waiver without policy allowance is unmet");
    }

    #[test]
    fn latest_attempt_wins_within_a_run() {
        let policy = WorkTaskGatePolicy {
            gates: vec![req("g", WorkTaskGateType::Preflight, true, false, false)],
        };
        let mut running = attempt(2, 3, "g", WorkTaskGateStatus::Running, false);
        running.evidence = None;
        let passed = attempt(3, 3, "g", WorkTaskGateStatus::Passed, false);
        // Later id wins even if listed first.
        let d = evaluate(&policy, &[running, passed], 3, false, None);
        assert!(d.eligible);
    }
}
