#![forbid(unsafe_code)]

use super::citation_validator::{
    build_evidence_citation_index, validate_claim_citation_coverage, CitationValidationError,
};
use super::claim_extractor::{AtomicClaim, CitationRef, CitationRefKind};
use super::{
    synthesize_evidence_bound, ExternalLookup, SynthesisError, SynthesisPolicy,
    SYNTHESIS_TEMPLATE_VERSION,
};
use crate::web_search_plan::packet_validator::{validate_packet, validate_packet_schema_registry};
use crate::web_search_plan::registry_loader::load_packet_schema_registry;
use serde_json::{json, Value};

fn base_evidence_packet() -> Value {
    json!({
        "schema_version": "1.0.0",
        "produced_by": "PH1.E",
        "intended_consumers": ["PH1.D", "PH1.WRITE", "PH1.J"],
        "created_at_ms": 1_700_000_500_000i64,
        "trace_id": "trace-run5",
        "query": "run5 synthesis question",
        "retrieved_at_ms": 1_700_000_500_100i64,
        "provider_runs": [
            {
                "endpoint": "url_fetch",
                "latency_ms": 42
            }
        ],
        "sources": [
            {
                "title": "Source A",
                "url": "https://example.com/a",
                "media_type": "web"
            },
            {
                "title": "Source B",
                "url": "https://example.com/b",
                "media_type": "web"
            }
        ],
        "content_chunks": [
            {
                "chunk_id": "chunk-a",
                "source_url": "https://example.com/a",
                "chunk_index": 0,
                "text_excerpt": "Claim from source A with deterministic wording.",
                "text_len_chars": 48
            },
            {
                "chunk_id": "chunk-b",
                "source_url": "https://example.com/b",
                "chunk_index": 1,
                "text_excerpt": "Claim from source B that corroborates the answer.",
                "text_len_chars": 50
            }
        ],
        "trust_metadata": {}
    })
}

fn malformed_uncited_evidence_packet() -> Value {
    json!({
        "schema_version": "1.0.0",
        "produced_by": "PH1.E",
        "intended_consumers": ["PH1.D", "PH1.WRITE", "PH1.J"],
        "created_at_ms": 1_700_000_500_000i64,
        "trace_id": "trace-run5-uncited",
        "query": "run5 synthesis question",
        "retrieved_at_ms": 1_700_000_500_100i64,
        "provider_runs": [{"endpoint": "url_fetch", "latency_ms": 1}],
        "sources": [
            {"title": "Source A", "url": "https://example.com/a", "media_type": "web"},
            {"title": "Source B", "url": "https://example.com/b", "media_type": "web"}
        ],
        "content_chunks": [
            {
                "chunk_id": "",
                "source_url": "",
                "chunk_index": 0,
                "text_excerpt": "Uncited statement from a malformed chunk.",
                "text_len_chars": 39
            },
            {
                "chunk_id": "",
                "source_url": "",
                "chunk_index": 1,
                "text_excerpt": "Another uncited statement from malformed chunk.",
                "text_len_chars": 45
            }
        ],
        "trust_metadata": {}
    })
}

#[test]
fn test_t1_claim_to_citation_coverage_is_strict() {
    let evidence = base_evidence_packet();
    let index = build_evidence_citation_index(&evidence).expect("evidence index should build");

    let claims = vec![
        AtomicClaim {
            claim_id: "claim_001".to_string(),
            text: "Claim with citation".to_string(),
            citations: vec![CitationRef {
                kind: CitationRefKind::ChunkId,
                value: "chunk-a".to_string(),
            }],
        },
        AtomicClaim {
            claim_id: "claim_002".to_string(),
            text: "Claim without citation".to_string(),
            citations: vec![],
        },
    ];

    let err = validate_claim_citation_coverage(&claims, &index)
        .expect_err("a claim without citation must fail coverage validation");
    match err {
        CitationValidationError::UnsupportedClaim { claim_id, .. } => {
            assert_eq!(claim_id, "claim_002");
        }
        other => panic!("unexpected error variant: {:?}", other),
    }
}

#[test]
fn test_t2_unsupported_claim_triggers_hard_fail() {
    let evidence = malformed_uncited_evidence_packet();
    let policy = SynthesisPolicy {
        // Bypass sufficiency so unsupported-claim validation is exercised directly.
        sufficiency: super::insufficiency_gate::EvidenceSufficiencyPolicy {
            min_distinct_sources: 0,
            min_chunk_support: 0,
        },
        max_claims: 8,
    };

    let err = synthesize_evidence_bound(
        "What does the evidence say?",
        &evidence,
        1_700_000_500_200i64,
        "trace-run5-t2",
        policy,
        None,
    )
    .expect_err("unsupported claim path must fail closed");

    assert!(matches!(err, SynthesisError::UnsupportedClaim(_)));
}

#[test]
fn test_t3_conflict_detection_surfaces_uncertainty() {
    let mut evidence = base_evidence_packet();
    let chunks = evidence
        .get_mut("content_chunks")
        .and_then(Value::as_array_mut)
        .expect("content_chunks should exist");
    chunks[0]["contradiction_group_id"] = json!("group-1");
    chunks[1]["contradiction_group_id"] = json!("group-1");
    chunks[1]["text_excerpt"] = json!("Conflicting statement from source B.");

    let result = synthesize_evidence_bound(
        "Do sources agree?",
        &evidence,
        1_700_000_500_200i64,
        "trace-run5-t3",
        SynthesisPolicy::default(),
        None,
    )
    .expect("conflict synthesis should still produce packet with uncertainty");

    let packet = result.synthesis_packet;
    let uncertainty = packet
        .get("uncertainty_flags")
        .and_then(Value::as_array)
        .expect("uncertainty flags should be array")
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<&str>>();
    assert!(uncertainty.contains(&"conflicting_evidence_detected"));

    let reason_codes = packet
        .get("reason_codes")
        .and_then(Value::as_array)
        .expect("reason codes should be array")
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<&str>>();
    assert!(reason_codes.contains(&"conflicting_evidence_detected"));

    let answer = packet
        .get("answer_text")
        .and_then(Value::as_str)
        .expect("answer text should exist");
    assert!(answer.contains("Uncertainty"));
    assert!(result.audit_metrics.conflict_detected);
}

#[test]
fn test_t4_insufficient_evidence_refusal_works() {
    let mut evidence = base_evidence_packet();
    evidence["sources"] = json!([{
        "title": "Source A",
        "url": "https://example.com/a",
        "media_type": "web"
    }]);
    evidence["content_chunks"] = json!([{
        "chunk_id": "chunk-a",
        "source_url": "https://example.com/a",
        "chunk_index": 0,
        "text_excerpt": "Only one chunk is available.",
        "text_len_chars": 28
    }]);

    let result = synthesize_evidence_bound(
        "Can you answer this fully?",
        &evidence,
        1_700_000_500_200i64,
        "trace-run5-t4",
        SynthesisPolicy::default(),
        None,
    )
    .expect("insufficient evidence should return deterministic refusal packet");

    let packet = result.synthesis_packet;
    let reason_codes = packet
        .get("reason_codes")
        .and_then(Value::as_array)
        .expect("reason codes must exist")
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<&str>>();
    assert_eq!(reason_codes, vec!["insufficient_evidence"]);

    let answer = packet
        .get("answer_text")
        .and_then(Value::as_str)
        .expect("answer text should exist");
    assert!(answer.contains("Insufficient evidence"));
    assert!(result.audit_metrics.insufficient_evidence_flag);
}

#[test]
fn test_t5_identical_evidence_produces_identical_synthesis_packet() {
    let evidence = base_evidence_packet();
    let first = synthesize_evidence_bound(
        "What is the grounded answer?",
        &evidence,
        1_700_000_500_200i64,
        "trace-run5-t5",
        SynthesisPolicy::default(),
        None,
    )
    .expect("first synthesis should pass");
    let second = synthesize_evidence_bound(
        "What is the grounded answer?",
        &evidence,
        1_700_000_500_200i64,
        "trace-run5-t5",
        SynthesisPolicy::default(),
        None,
    )
    .expect("second synthesis should pass");

    assert_eq!(first.synthesis_packet, second.synthesis_packet);
    assert_eq!(first.audit_metrics, second.audit_metrics);

    let packet_registry = load_packet_schema_registry().expect("packet schema should load");
    validate_packet_schema_registry(&packet_registry).expect("packet schema should validate");
    validate_packet("SynthesisPacket", &first.synthesis_packet, &packet_registry)
        .expect("output packet must satisfy SynthesisPacket schema");
    assert_eq!(
        first
            .synthesis_packet
            .get("synthesis_template_version")
            .and_then(Value::as_str),
        Some(SYNTHESIS_TEMPLATE_VERSION)
    );
}

struct PanicLookup;

impl ExternalLookup for PanicLookup {
    fn lookup(&self, _query: &str) -> Option<String> {
        panic!("external lookup must never be called for PH1.D synthesis");
    }
}

#[test]
fn test_t6_external_lookup_boundary_blocked() {
    let evidence = base_evidence_packet();
    let lookup = PanicLookup;
    let err = synthesize_evidence_bound(
        "Use only evidence",
        &evidence,
        1_700_000_500_200i64,
        "trace-run5-t6",
        SynthesisPolicy::default(),
        Some(&lookup),
    )
    .expect_err("external lookup injection must be blocked");

    match err {
        SynthesisError::EvidenceBoundaryViolation(message) => {
            assert!(message.contains("forbidden"));
        }
        other => panic!("unexpected error variant: {:?}", other),
    }
}

#[test]
fn test_t7_citation_references_valid_chunk_ids_only() {
    let evidence = base_evidence_packet();
    let index = build_evidence_citation_index(&evidence).expect("evidence index should build");

    let claims = vec![AtomicClaim {
        claim_id: "claim_001".to_string(),
        text: "Invalid chunk claim".to_string(),
        citations: vec![CitationRef {
            kind: CitationRefKind::ChunkId,
            value: "chunk-unknown".to_string(),
        }],
    }];

    let err = validate_claim_citation_coverage(&claims, &index)
        .expect_err("unknown chunk_id must fail citation validation");
    match err {
        CitationValidationError::CitationMismatch { message, .. } => {
            assert!(message.contains("unknown chunk_id"));
        }
        other => panic!("unexpected error variant: {:?}", other),
    }
}
