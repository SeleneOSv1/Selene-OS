#![forbid(unsafe_code)]

use crate::web_search_plan::learning::failure_signature::{
    compute_signature_id, FailureEventInput, FailureLedger, SearchMode,
};
use crate::web_search_plan::learning::promotion_gate::{
    approve_proposal, PolicySnapshot, PromotionError,
};
use crate::web_search_plan::learning::proposal::{
    generate_proposal_if_threshold, LearningProposalLedger, ProposalStatus,
};
use crate::web_search_plan::learning::rollback::{PolicyHistory, RollbackError};
use crate::web_search_plan::learning::session_adaptation::{
    apply_bounded_adaptation, AdaptationError, AdaptationRequest, BaselineExecutionPolicy,
};
use crate::web_search_plan::learning::{
    append_learning_audit_fields, assert_no_packet_mutation, enforce_learning_boundary,
    BoundaryTarget, LearningBoundaryError,
};
use serde_json::json;

fn failure_input() -> FailureEventInput {
    FailureEventInput {
        provider_id: "brave_web_search".to_string(),
        error_kind: "timeout_exceeded".to_string(),
        mode: SearchMode::Web,
        importance_tier: "high".to_string(),
    }
}

fn baseline_policy() -> BaselineExecutionPolicy {
    BaselineExecutionPolicy {
        fallback_priority: vec![
            "brave_web_search".to_string(),
            "openai_web_search".to_string(),
        ],
        retry_attempts: 3,
        cooldown_failures_before: 2,
        open_budget_per_query: 3,
    }
}

fn policy_snapshot(version: &str) -> PolicySnapshot {
    PolicySnapshot {
        policy_snapshot_version: version.to_string(),
        fallback_priority: vec![
            "brave_web_search".to_string(),
            "openai_web_search".to_string(),
        ],
        retry_attempts: 3,
        cooldown_failures_before: 2,
        open_budget_per_query: 3,
    }
}

#[test]
fn test_t1_failure_signatures_hash_deterministically() {
    let input_a = failure_input();
    let input_b = failure_input();

    let signature_a = compute_signature_id(&input_a);
    let signature_b = compute_signature_id(&input_b);

    assert_eq!(signature_a, signature_b);
    assert_eq!(signature_a.len(), 64);
}

#[test]
fn test_t2_repeated_identical_failures_increment_occurrence_count() {
    let mut ledger = FailureLedger::default();
    let input = failure_input();

    let first = ledger.record_failure(&input, 1_772_500_000_000, 60_000);
    let second = ledger.record_failure(&input, 1_772_500_000_100, 60_000);

    assert_eq!(first.signature_id, second.signature_id);
    assert_eq!(first.occurrence_count, 1);
    assert_eq!(second.occurrence_count, 2);
    assert_eq!(ledger.occurrence_count(&first.signature_id), 2);
    assert_eq!(ledger.len(), 2);
}

#[test]
fn test_t3_session_adaptation_does_not_exceed_allowed_scope() {
    let baseline = baseline_policy();

    let invalid_fallback = apply_bounded_adaptation(
        "session-1",
        "sig-1",
        &baseline,
        &AdaptationRequest {
            reordered_fallback_priority: Some(vec![
                "brave_web_search".to_string(),
                "openai_web_search".to_string(),
                "third_provider".to_string(),
            ]),
            retry_attempts: None,
            cooldown_failures_before: None,
            open_budget_per_query: None,
        },
        1000,
        5000,
    );
    assert_eq!(
        invalid_fallback,
        Err(AdaptationError::FallbackPriorityExpansion)
    );

    let invalid_retry = apply_bounded_adaptation(
        "session-1",
        "sig-1",
        &baseline,
        &AdaptationRequest {
            reordered_fallback_priority: None,
            retry_attempts: Some(4),
            cooldown_failures_before: None,
            open_budget_per_query: None,
        },
        1000,
        5000,
    );
    assert_eq!(invalid_retry, Err(AdaptationError::RetryIncreaseBlocked));

    let valid = apply_bounded_adaptation(
        "session-1",
        "sig-1",
        &baseline,
        &AdaptationRequest {
            reordered_fallback_priority: Some(vec![
                "openai_web_search".to_string(),
                "brave_web_search".to_string(),
            ]),
            retry_attempts: Some(2),
            cooldown_failures_before: Some(1),
            open_budget_per_query: Some(2),
        },
        1000,
        5000,
    )
    .expect("valid bounded adaptation should pass");

    assert_eq!(valid.policy.retry_attempts, 2);
    assert_eq!(valid.policy.cooldown_failures_before, 1);
    assert_eq!(valid.policy.open_budget_per_query, 2);
    assert_eq!(
        valid.policy.fallback_priority,
        vec![
            "openai_web_search".to_string(),
            "brave_web_search".to_string()
        ]
    );
}

#[test]
fn test_t4_adaptation_expires_after_session_end() {
    let adaptation = apply_bounded_adaptation(
        "session-2",
        "sig-2",
        &baseline_policy(),
        &AdaptationRequest {
            reordered_fallback_priority: None,
            retry_attempts: Some(2),
            cooldown_failures_before: Some(1),
            open_budget_per_query: Some(2),
        },
        1_000,
        2_000,
    )
    .expect("adaptation should be created");

    assert!(adaptation.is_active(1_500, true));
    assert!(!adaptation.is_active(3_001, true));
    assert!(!adaptation.is_active(1_500, false));
}

#[test]
fn test_t5_proposal_artifact_generated_after_threshold() {
    let mut ledger = FailureLedger::default();
    let input = failure_input();

    ledger.record_failure(&input, 10, 60_000);
    ledger.record_failure(&input, 20, 60_000);
    let third = ledger.record_failure(&input, 30, 60_000);

    let proposal = generate_proposal_if_threshold(
        &third,
        3,
        "reduce retries for timeout_exceeded in web mode",
        "reduces repeated timeout latency",
        40,
    )
    .expect("proposal should be generated at threshold");

    assert_eq!(proposal.status, ProposalStatus::Proposed);
    assert_eq!(proposal.related_signature_id, third.signature_id);
    assert_eq!(proposal.reproducibility_count, 3);

    let not_generated = generate_proposal_if_threshold(&third, 4, "same change", "same impact", 50);
    assert!(not_generated.is_none());
}

#[test]
fn test_t6_promotion_requires_explicit_approval() {
    let signature = FailureLedger::default().record_failure(&failure_input(), 10, 60_000);
    let proposal = generate_proposal_if_threshold(
        &signature,
        1,
        "reduce retries",
        "improve timeout resilience",
        20,
    )
    .expect("proposal should exist");

    let current = policy_snapshot("policy-v1");
    let mut promoted = policy_snapshot("policy-v2");
    promoted.retry_attempts = 2;

    let wrong_approver = approve_proposal(&proposal, "PH1.X", true, &current, &promoted, 30);
    assert_eq!(wrong_approver, Err(PromotionError::InvalidApprover));

    let no_replay = approve_proposal(&proposal, "PH1.GOV", false, &current, &promoted, 30);
    assert_eq!(no_replay, Err(PromotionError::ReplayValidationRequired));

    let approved = approve_proposal(&proposal, "PH1.GOV", true, &current, &promoted, 30)
        .expect("approved promotion should pass");

    assert_eq!(approved.0.status, ProposalStatus::Approved);
    assert_eq!(approved.1.prior_policy_snapshot_version, "policy-v1");
    assert_eq!(approved.1.new_policy_snapshot_version, "policy-v2");
}

#[test]
fn test_t7_rollback_restores_prior_policy_snapshot() {
    let current = policy_snapshot("policy-v1");
    let mut promoted = policy_snapshot("policy-v2");
    promoted.retry_attempts = 2;

    let mut history = PolicyHistory::new(current.clone());
    history.promote(promoted);

    assert_eq!(history.current().policy_snapshot_version, "policy-v2");
    assert!(history.prior().is_some());

    let rollback_record = history.rollback(999).expect("rollback should pass");
    assert_eq!(rollback_record.from_policy_snapshot_version, "policy-v2");
    assert_eq!(
        rollback_record.restored_policy_snapshot_version,
        "policy-v1"
    );
    assert_eq!(rollback_record.rollback_status, "rolled_back");
    assert_eq!(history.current().policy_snapshot_version, "policy-v1");

    let second = history.rollback(1000);
    assert_eq!(second, Err(RollbackError::PriorSnapshotMissing));
}

#[test]
fn test_t8_learning_cannot_alter_evidence_packet() {
    let original = json!({
        "schema_version": "1.0.0",
        "query": "q",
        "provider_runs": [],
        "sources": [],
        "content_chunks": [],
        "trust_metadata": {}
    });

    let mut mutated = original.clone();
    mutated["query"] = json!("different");

    let packet_guard = assert_no_packet_mutation(&original, &mutated, "EvidencePacket");
    assert_eq!(
        packet_guard,
        Err(LearningBoundaryError::PacketMutationBlocked(
            "EvidencePacket".to_string()
        ))
    );

    let boundary_guard = enforce_learning_boundary(BoundaryTarget::EvidencePacket);
    assert_eq!(
        boundary_guard,
        Err(LearningBoundaryError::BoundaryViolation("evidence_packet"))
    );
}

#[test]
fn test_t9_learning_cannot_alter_provider_ladder_logic_directly() {
    let guard = enforce_learning_boundary(BoundaryTarget::ProviderLadderCore);
    assert_eq!(
        guard,
        Err(LearningBoundaryError::BoundaryViolation(
            "provider_ladder_core"
        ))
    );

    let baseline = baseline_policy();
    let fanout_attempt = apply_bounded_adaptation(
        "session-3",
        "sig-3",
        &baseline,
        &AdaptationRequest {
            reordered_fallback_priority: Some(vec![
                "openai_web_search".to_string(),
                "brave_web_search".to_string(),
                "new_provider".to_string(),
            ]),
            retry_attempts: None,
            cooldown_failures_before: None,
            open_budget_per_query: None,
        },
        200,
        1000,
    );
    assert_eq!(
        fanout_attempt,
        Err(AdaptationError::FallbackPriorityExpansion)
    );
}

#[test]
fn test_t10_replay_determinism_preserved_after_adaptation_no_drift() {
    fn run_sequence() -> (String, String, String, String) {
        let mut failures = FailureLedger::default();
        let input = failure_input();
        failures.record_failure(&input, 1_000, 20_000);
        let signature = failures.record_failure(&input, 1_100, 20_000);

        let adaptation = apply_bounded_adaptation(
            "session-deterministic",
            &signature.signature_id,
            &baseline_policy(),
            &AdaptationRequest {
                reordered_fallback_priority: Some(vec![
                    "openai_web_search".to_string(),
                    "brave_web_search".to_string(),
                ]),
                retry_attempts: Some(2),
                cooldown_failures_before: Some(1),
                open_budget_per_query: Some(2),
            },
            1_200,
            10_000,
        )
        .expect("adaptation should pass");

        let proposal = generate_proposal_if_threshold(
            &signature,
            2,
            "promote openai first after repeated brave timeout",
            "reduce repeated timeout cycles",
            1_300,
        )
        .expect("proposal should generate");

        let mut ledger = LearningProposalLedger::default();
        ledger.append(proposal.clone());

        let current = policy_snapshot("policy-v1");
        let mut promoted = policy_snapshot("policy-v2");
        promoted.fallback_priority = adaptation.policy.fallback_priority.clone();
        promoted.retry_attempts = adaptation.policy.retry_attempts;
        promoted.cooldown_failures_before = adaptation.policy.cooldown_failures_before;
        promoted.open_budget_per_query = adaptation.policy.open_budget_per_query;

        let (_approved, promotion_record) = approve_proposal(
            ledger
                .latest_for_signature(&signature.signature_id)
                .expect("proposal exists"),
            "PH1.GOV",
            true,
            &current,
            &promoted,
            1_400,
        )
        .expect("promotion should pass");

        (
            signature.signature_id,
            adaptation.adaptation_id,
            proposal.proposal_id,
            promoted.fingerprint() + ":" + &promotion_record.promotion_status,
        )
    }

    let run_a = run_sequence();
    let run_b = run_sequence();

    assert_eq!(run_a, run_b);

    let mut audit_packet = json!({
        "schema_version": "1.0.0",
        "produced_by": "PH1.OS",
        "intended_consumers": ["PH1.J"],
        "created_at_ms": 1_777_500_000_000i64,
        "trace_id": "trace-run11-audit",
        "turn_state_transition": "AUDIT_COMMITTED",
        "packet_hashes": {"a": "b"},
        "evidence_hash": "eh",
        "response_hash": "rh",
        "reason_codes": ["timeout_exceeded"],
        "policy_snapshot_id": "policy-v2"
    });

    append_learning_audit_fields(
        &mut audit_packet,
        &crate::web_search_plan::learning::LearningAuditMetrics::new(
            run_a.0,
            true,
            true,
            Some(run_a.2),
            "approved",
            "not_requested",
            "policy-v2",
        ),
    )
    .expect("learning audit append should pass");

    assert!(audit_packet
        .pointer("/turn_state_transition/learning_audit/failure_signature_id")
        .is_some());
}
