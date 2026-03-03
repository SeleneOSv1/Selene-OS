#![forbid(unsafe_code)]

use crate::web_search_plan::learn::failure_signature::{
    compute_signature_id, FailureEvent, FailureLedger, LearningLane,
};
use crate::web_search_plan::learn::promotion_gate::{
    approve_proposal, PolicySnapshotReference, PromotionApprovalInput, PromotionError,
};
use crate::web_search_plan::learn::proposal_artifact::{
    generate_proposal_if_threshold, ProposalStatus, ProposedChangeType,
};
use crate::web_search_plan::learn::rollback::{PromotionState, RollbackError};
use crate::web_search_plan::learn::session_adaptation::{
    apply_bounded_adaptation, AdaptationError, AdaptationRequest, SessionBaselinePolicy,
};
use crate::web_search_plan::learn::{
    assert_packet_immutable, enforce_non_authoritative_boundary, BoundaryTarget, LearnBoundaryError,
};
use crate::web_search_plan::perf_cost::tiers::ImportanceTier;
use serde_json::json;

fn failure_event(occurred_at_ms: i64) -> FailureEvent {
    FailureEvent {
        lane: LearningLane::Web,
        provider_id: Some("Brave_Web_Search".to_string()),
        error_kind: "timeout_exceeded".to_string(),
        reason_code_id: "timeout_exceeded".to_string(),
        importance_tier: ImportanceTier::High,
        canonical_url: Some("https://Example.com/Path".to_string()),
        occurred_at_ms,
        ttl_ms: 200,
    }
}

fn baseline_policy() -> SessionBaselinePolicy {
    SessionBaselinePolicy {
        session_id: "session-1".to_string(),
        importance_tier: ImportanceTier::High,
        lead_provider: Some("brave_web_search".to_string()),
        fallback_priority: vec![
            "brave_web_search".to_string(),
            "openai_web_search".to_string(),
        ],
        retry_attempts: 1,
        cooldown_failures_before: 3,
        open_budget_per_query: 3,
        max_provider_fan_out: 2,
    }
}

fn policy(
    version: &str,
    timeout_per_provider_ms: u64,
    open_budget_per_query: usize,
) -> PolicySnapshotReference {
    PolicySnapshotReference {
        policy_snapshot_version: version.to_string(),
        fallback_priority: vec![
            "brave_web_search".to_string(),
            "openai_web_search".to_string(),
        ],
        timeout_per_provider_ms,
        open_budget_per_query,
    }
}

#[test]
fn test_t1_signature_id_determinism_same_event_same_id() {
    let event_a = failure_event(1_000);
    let event_b = failure_event(1_500);

    let id_a = compute_signature_id(&event_a);
    let id_b = compute_signature_id(&event_b);

    assert_eq!(id_a, id_b);
    assert_eq!(id_a.len(), 64);
}

#[test]
fn test_t2_count_increments_deterministically() {
    let mut ledger = FailureLedger::new_from_registry_file()
        .expect("reason registry should load for failure ledger");

    let first = ledger
        .record_failure(&failure_event(1_000), "policy-v1")
        .expect("first failure should record");
    let second = ledger
        .record_failure(&failure_event(1_100), "policy-v1")
        .expect("second failure should record");

    assert_eq!(first.signature_id, second.signature_id);
    assert_eq!(first.count, 1);
    assert_eq!(second.count, 2);
    assert_eq!(ledger.entries().len(), 2);
}

#[test]
fn test_t3_ttl_expiry_deterministic() {
    let mut ledger = FailureLedger::new_from_registry_file()
        .expect("reason registry should load for failure ledger");

    let first = ledger
        .record_failure(&failure_event(1_000), "policy-v1")
        .expect("first failure should record");
    let second = ledger
        .record_failure(&failure_event(1_100), "policy-v1")
        .expect("second failure should record");
    let third = ledger
        .record_failure(&failure_event(1_401), "policy-v1")
        .expect("post-expiry failure should record");

    assert_eq!(first.count, 1);
    assert_eq!(second.count, 2);
    assert_eq!(third.count, 1);
    assert_eq!(third.created_at_ms, 1_401);
}

#[test]
fn test_t4_session_adaptations_stay_within_allowed_set() {
    let baseline = baseline_policy();
    let adaptation = apply_bounded_adaptation(
        "sig-1",
        &baseline,
        &AdaptationRequest {
            reordered_fallback_priority: Some(vec![
                "openai_web_search".to_string(),
                "brave_web_search".to_string(),
            ]),
            retry_attempts: Some(1),
            cooldown_failures_before: Some(1),
            skip_lead_provider: true,
            reduce_open_budget_per_query: Some(1),
            force_snippet_only: true,
            provider_fan_out_override: None,
        },
        10_000,
        5_000,
    )
    .expect("bounded adaptation should pass");

    assert_eq!(adaptation.policy.retry_attempts, 1);
    assert_eq!(adaptation.policy.cooldown_failures_before, 1);
    assert_eq!(adaptation.policy.open_budget_per_query, 1);
    assert!(adaptation.policy.lead_provider.is_none());
    assert!(adaptation.policy.snippet_only_mode);
    assert_eq!(
        adaptation.policy.fallback_priority,
        vec![
            "openai_web_search".to_string(),
            "brave_web_search".to_string()
        ]
    );
}

#[test]
fn test_t5_adaptations_expire_correctly() {
    let adaptation = apply_bounded_adaptation(
        "sig-2",
        &baseline_policy(),
        &AdaptationRequest {
            reordered_fallback_priority: None,
            retry_attempts: Some(0),
            cooldown_failures_before: Some(1),
            skip_lead_provider: false,
            reduce_open_budget_per_query: Some(2),
            force_snippet_only: false,
            provider_fan_out_override: None,
        },
        2_000,
        500,
    )
    .expect("adaptation should be created");

    assert!(adaptation.is_active(2_250, true));
    assert!(!adaptation.is_active(2_501, true));
    assert!(!adaptation.is_active(2_250, false));
}

#[test]
fn test_t6_proposal_artifact_created_only_after_threshold() {
    let mut ledger = FailureLedger::new_from_registry_file()
        .expect("reason registry should load for failure ledger");

    ledger
        .record_failure(&failure_event(1_000), "policy-v1")
        .expect("first failure should record");
    let second = ledger
        .record_failure(&failure_event(1_100), "policy-v1")
        .expect("second failure should record");
    let third = ledger
        .record_failure(&failure_event(1_200), "policy-v1")
        .expect("third failure should record");

    let payload = json!({
        "cooldown_hint": "early",
        "lane": "web"
    });

    let not_yet = generate_proposal_if_threshold(
        &second,
        3,
        ProposedChangeType::CooldownTune,
        &payload,
        1_300,
    );
    assert!(not_yet.is_none());

    let proposal = generate_proposal_if_threshold(
        &third,
        3,
        ProposedChangeType::CooldownTune,
        &payload,
        1_300,
    )
    .expect("proposal should be created at threshold");
    assert_eq!(proposal.status, ProposalStatus::Proposed);
    assert_eq!(proposal.evidence.occurrence_count, 3);
}

#[test]
fn test_t7_promotion_requires_explicit_approval_input() {
    let mut ledger = FailureLedger::new_from_registry_file()
        .expect("reason registry should load for failure ledger");
    let signature = ledger
        .record_failure(&failure_event(1_000), "policy-v1")
        .expect("failure should record");

    let proposal = generate_proposal_if_threshold(
        &signature,
        1,
        ProposedChangeType::RoutingHint,
        &json!({ "preferred_lead": "openai_web_search" }),
        1_200,
    )
    .expect("proposal should exist");

    let current = policy("policy-v1", 2000, 3);
    let promoted = policy("policy-v2", 1500, 2);

    let invalid = approve_proposal(
        &proposal,
        &PromotionApprovalInput {
            approver_engine_id: "PH1.X".to_string(),
            proposal_id: proposal.proposal_id.clone(),
            approved_at_ms: 1_300,
        },
        &current,
        &promoted,
    );
    assert_eq!(invalid, Err(PromotionError::InvalidApprover));

    let mismatch = approve_proposal(
        &proposal,
        &PromotionApprovalInput {
            approver_engine_id: "PH1.GOV".to_string(),
            proposal_id: "wrong-id".to_string(),
            approved_at_ms: 1_300,
        },
        &current,
        &promoted,
    );
    assert_eq!(mismatch, Err(PromotionError::ProposalIdMismatch));

    let approved = approve_proposal(
        &proposal,
        &PromotionApprovalInput {
            approver_engine_id: "PH1.GOV".to_string(),
            proposal_id: proposal.proposal_id.clone(),
            approved_at_ms: 1_300,
        },
        &current,
        &promoted,
    )
    .expect("approval with PH1.GOV should succeed");
    assert_eq!(approved.0.status, ProposalStatus::Approved);
    assert_eq!(approved.1.activation_state, "activation_pending");
}

#[test]
fn test_t8_rollback_restores_prior_state_deterministically() {
    let current = policy("policy-v1", 2000, 3);
    let promoted = policy("policy-v2", 1500, 1);

    let mut ledger = FailureLedger::new_from_registry_file()
        .expect("reason registry should load for failure ledger");
    let signature = ledger
        .record_failure(&failure_event(1_000), "policy-v1")
        .expect("failure should record");
    let proposal = generate_proposal_if_threshold(
        &signature,
        1,
        ProposedChangeType::BudgetTune,
        &json!({ "open_budget_per_query": 1 }),
        1_100,
    )
    .expect("proposal should exist");

    let (_, promotion_record) = approve_proposal(
        &proposal,
        &PromotionApprovalInput {
            approver_engine_id: "PH1.GOV".to_string(),
            proposal_id: proposal.proposal_id.clone(),
            approved_at_ms: 1_200,
        },
        &current,
        &promoted,
    )
    .expect("promotion should be approved");

    let mut state = PromotionState::new(current.clone());
    state.apply_promotion(promotion_record, promoted.clone());
    assert_eq!(state.current().policy_snapshot_version, "policy-v2");

    let rollback = state
        .rollback(1_250, "PH1.GOV")
        .expect("rollback should restore prior policy");
    assert_eq!(rollback.from_policy_snapshot_version, "policy-v2");
    assert_eq!(rollback.restored_policy_snapshot_version, "policy-v1");
    assert_eq!(state.current(), &current);
    assert_eq!(state.previous(), None);

    let second = state.rollback(1_300, "PH1.GOV");
    assert_eq!(second, Err(RollbackError::PriorSnapshotMissing));
}

#[test]
fn test_t9_learning_cannot_change_evidence_synthesis_write_outputs() {
    let evidence_original = json!({
        "schema_version": "1.0.0",
        "query": "q",
        "provider_runs": [],
        "sources": [],
        "content_chunks": [],
        "trust_metadata": {}
    });
    let mut evidence_mutated = evidence_original.clone();
    evidence_mutated["query"] = json!("mutated");

    let blocked = assert_packet_immutable(&evidence_original, &evidence_mutated, "EvidencePacket");
    assert_eq!(
        blocked,
        Err(LearnBoundaryError::PacketMutationBlocked(
            "EvidencePacket".to_string()
        ))
    );

    assert_eq!(
        enforce_non_authoritative_boundary(BoundaryTarget::EvidencePacketWrite),
        Err(LearnBoundaryError::BoundaryViolation(
            "evidence_packet_write"
        ))
    );
    assert_eq!(
        enforce_non_authoritative_boundary(BoundaryTarget::SynthesisPacketWrite),
        Err(LearnBoundaryError::BoundaryViolation(
            "synthesis_packet_write"
        ))
    );
    assert_eq!(
        enforce_non_authoritative_boundary(BoundaryTarget::WritePacketWrite),
        Err(LearnBoundaryError::BoundaryViolation("write_packet_write"))
    );
}

#[test]
fn test_t10_learning_cannot_increase_budgets_or_fanout() {
    let baseline = baseline_policy();

    let retry_increase = apply_bounded_adaptation(
        "sig-budget",
        &baseline,
        &AdaptationRequest {
            reordered_fallback_priority: None,
            retry_attempts: Some(2),
            cooldown_failures_before: None,
            skip_lead_provider: false,
            reduce_open_budget_per_query: None,
            force_snippet_only: false,
            provider_fan_out_override: None,
        },
        1_000,
        500,
    );
    assert_eq!(retry_increase, Err(AdaptationError::RetryIncreaseBlocked));

    let open_budget_increase = apply_bounded_adaptation(
        "sig-budget",
        &baseline,
        &AdaptationRequest {
            reordered_fallback_priority: None,
            retry_attempts: None,
            cooldown_failures_before: None,
            skip_lead_provider: false,
            reduce_open_budget_per_query: Some(5),
            force_snippet_only: false,
            provider_fan_out_override: None,
        },
        1_000,
        500,
    );
    assert_eq!(
        open_budget_increase,
        Err(AdaptationError::OpenBudgetIncreaseBlocked)
    );

    let fanout_change = apply_bounded_adaptation(
        "sig-budget",
        &baseline,
        &AdaptationRequest {
            reordered_fallback_priority: None,
            retry_attempts: None,
            cooldown_failures_before: None,
            skip_lead_provider: false,
            reduce_open_budget_per_query: None,
            force_snippet_only: false,
            provider_fan_out_override: Some(3),
        },
        1_000,
        500,
    );
    assert_eq!(
        fanout_change,
        Err(AdaptationError::ProviderFanoutChangeBlocked)
    );
}
