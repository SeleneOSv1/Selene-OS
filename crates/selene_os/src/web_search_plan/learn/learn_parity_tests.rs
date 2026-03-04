#![forbid(unsafe_code)]

use crate::web_search_plan::learn::failure_signature::{
    compute_signature_id, FailureEvent, FailureSignature, LearningLane,
};
use crate::web_search_plan::learn::promotion_gate::{
    approve_proposal, PolicySnapshotReference, PromotionApprovalInput,
};
use crate::web_search_plan::learn::proposal_artifact::{
    generate_proposal_if_threshold, ProposalStatus, ProposedChangeType,
};
use crate::web_search_plan::learn::rollback::PromotionState;
use crate::web_search_plan::learn::session_adaptation::{
    apply_bounded_adaptation, AdaptationRequest, SessionBaselinePolicy,
};
use crate::web_search_plan::perf_cost::tiers::ImportanceTier;
use serde_json::json;

#[test]
fn test_parity_failure_signature_id_determinism() {
    let event = FailureEvent {
        lane: LearningLane::News,
        provider_id: Some("Brave_News_Search".to_string()),
        error_kind: "Timeout_Exceeded".to_string(),
        reason_code_id: "timeout_exceeded".to_string(),
        importance_tier: ImportanceTier::High,
        canonical_url: Some("HTTPS://EXAMPLE.COM/A".to_string()),
        occurred_at_ms: 1_772_496_000_000,
        ttl_ms: 60_000,
    };

    let id_a = compute_signature_id(&event);
    let id_b = compute_signature_id(&event);
    assert_eq!(id_a, id_b);
    assert_eq!(id_a.len(), 64);
}

#[test]
fn test_parity_adaptation_bounds() {
    let baseline = SessionBaselinePolicy::for_tier(
        "session-1",
        ImportanceTier::Medium,
        Some("brave_web_search".to_string()),
        vec![
            "brave_web_search".to_string(),
            "openai_web_search".to_string(),
        ],
    );

    let invalid = apply_bounded_adaptation(
        "sig-1",
        &baseline,
        &AdaptationRequest {
            reordered_fallback_priority: None,
            retry_attempts: Some(3),
            cooldown_failures_before: None,
            skip_lead_provider: false,
            reduce_open_budget_per_query: None,
            force_snippet_only: false,
            provider_fan_out_override: None,
        },
        1_772_496_000_000,
        60_000,
    );
    assert!(invalid.is_err());

    let valid = apply_bounded_adaptation(
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
        1_772_496_000_000,
        60_000,
    )
    .expect("bounded adaptation should pass");

    assert_eq!(
        valid.policy.fallback_priority,
        vec![
            "openai_web_search".to_string(),
            "brave_web_search".to_string(),
        ]
    );
    assert_eq!(valid.policy.retry_attempts, 1);
    assert_eq!(valid.policy.cooldown_failures_before, 1);
    assert_eq!(valid.policy.open_budget_per_query, 1);
    assert!(valid.policy.snippet_only_mode);
}

#[test]
fn test_parity_proposal_generation_threshold() {
    let signature = FailureSignature {
        signature_id: "sig-new".to_string(),
        lane: "news".to_string(),
        provider_id: Some("brave_news_search".to_string()),
        error_kind: "timeout_exceeded".to_string(),
        reason_code_id: "timeout_exceeded".to_string(),
        importance_tier: "high".to_string(),
        canonical_url: Some("https://example.com/a".to_string()),
        created_at_ms: 1_772_496_000_000,
        last_seen_at_ms: 1_772_496_100_000,
        count: 4,
        ttl_ms: 60_000,
        schema_version: "1.0.0".to_string(),
    };

    let proposal = generate_proposal_if_threshold(
        &signature,
        3,
        ProposedChangeType::TimeoutTune,
        &json!({"timeout_ms": 1500}),
        1_772_496_200_000,
    )
    .expect("proposal should be generated");

    assert_eq!(proposal.status, ProposalStatus::Proposed);
    assert_eq!(proposal.evidence.occurrence_count, 4);
}

#[test]
fn test_parity_promotion_gate_and_rollback() {
    let signature = FailureSignature {
        signature_id: "sig-new".to_string(),
        lane: "news".to_string(),
        provider_id: Some("brave_news_search".to_string()),
        error_kind: "timeout_exceeded".to_string(),
        reason_code_id: "timeout_exceeded".to_string(),
        importance_tier: "high".to_string(),
        canonical_url: Some("https://example.com/a".to_string()),
        created_at_ms: 1_772_496_000_000,
        last_seen_at_ms: 1_772_496_100_000,
        count: 4,
        ttl_ms: 60_000,
        schema_version: "1.0.0".to_string(),
    };
    let proposal = generate_proposal_if_threshold(
        &signature,
        3,
        ProposedChangeType::TimeoutTune,
        &json!({"timeout_ms": 1500}),
        1_772_496_200_000,
    )
    .expect("proposal should be generated");

    let current = PolicySnapshotReference {
        policy_snapshot_version: "v1".to_string(),
        fallback_priority: vec!["brave_web_search".to_string()],
        timeout_per_provider_ms: 2_000,
        open_budget_per_query: 2,
    };
    let next = PolicySnapshotReference {
        policy_snapshot_version: "v2".to_string(),
        fallback_priority: vec!["brave_web_search".to_string()],
        timeout_per_provider_ms: 1_500,
        open_budget_per_query: 2,
    };
    let approval = PromotionApprovalInput {
        approver_engine_id: "PH1.GOV".to_string(),
        proposal_id: proposal.proposal_id.clone(),
        approved_at_ms: 1_772_496_300_000,
    };
    let (approved, record) = approve_proposal(&proposal, &approval, &current, &next)
        .expect("promotion should pass");
    assert_eq!(approved.status, ProposalStatus::Approved);

    let mut state = PromotionState::new(current);
    state.apply_promotion(record, next);
    let rollback_record = state
        .rollback(1_772_496_400_000, "PH1.GOV")
        .expect("rollback should pass");
    assert_eq!(rollback_record.restored_policy_snapshot_version, "v1");
}
