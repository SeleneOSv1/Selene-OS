#![forbid(unsafe_code)]

use crate::web_search_plan::merge::merge_packet::{build_merge_packet, MergeRequest};
use crate::web_search_plan::merge::{InternalContext, MergeBuildError};
use crate::web_search_plan::packet_validator::validate_packet;
use crate::web_search_plan::registry_loader::load_packet_schema_registry;
use crate::web_search_plan::replay::snapshot::hash_canonical_json;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/web_search_plan/merge_fixtures")
}

fn load_fixture(name: &str) -> Value {
    let text = fs::read_to_string(fixture_dir().join(name)).expect("fixture should load");
    serde_json::from_str(&text).expect("fixture should parse as JSON")
}

fn request_from_fixture(name: &str) -> MergeRequest {
    let fixture = load_fixture(name);
    let internal_context = fixture.get("internal_context").cloned().and_then(|value| {
        if value.is_null() {
            None
        } else {
            Some(
                serde_json::from_value::<InternalContext>(value)
                    .expect("internal_context fixture should deserialize"),
            )
        }
    });

    MergeRequest {
        trace_id: fixture
            .get("trace_id")
            .and_then(Value::as_str)
            .expect("trace_id should exist")
            .to_string(),
        created_at_ms: fixture
            .get("created_at_ms")
            .and_then(Value::as_i64)
            .expect("created_at_ms should exist"),
        intended_consumers: fixture
            .get("intended_consumers")
            .and_then(Value::as_array)
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default(),
        policy_snapshot_id: fixture
            .get("policy_snapshot_id")
            .and_then(Value::as_str)
            .expect("policy_snapshot_id should exist")
            .to_string(),
        evidence_packet: fixture
            .get("evidence_packet")
            .cloned()
            .expect("evidence_packet should exist"),
        internal_context,
    }
}

#[test]
fn test_t1_merge_packet_validates_against_schema() {
    let output = build_merge_packet(request_from_fixture("external_only.json"))
        .expect("external-only merge should build");
    let packet_json = serde_json::to_value(&output.packet).expect("packet should serialize");
    let registry = load_packet_schema_registry().expect("packet schema registry should load");
    validate_packet("MergePacket", &packet_json, &registry)
        .expect("merge packet should validate against schema");
}

#[test]
fn test_t2_hashes_are_deterministic() {
    let first = build_merge_packet(request_from_fixture("consistent_case.json"))
        .expect("first merge build should pass");
    let second = build_merge_packet(request_from_fixture("consistent_case.json"))
        .expect("second merge build should pass");

    assert_eq!(first.packet.evidence_hash, second.packet.evidence_hash);
    assert_eq!(
        first.packet.internal_context_hash,
        second.packet.internal_context_hash
    );
}

#[test]
fn test_t3_delta_ordering_is_deterministic() {
    let output = build_merge_packet(request_from_fixture("external_only.json"))
        .expect("merge build should pass");
    let topic_keys = output
        .packet
        .delta
        .changes_since_last_time
        .iter()
        .map(|change| change.topic_key.clone())
        .collect::<Vec<String>>();
    let mut sorted = topic_keys.clone();
    sorted.sort();
    assert_eq!(topic_keys, sorted);
}

#[test]
fn test_t4_conflict_report_emitted_when_conflict_exists() {
    let output = build_merge_packet(request_from_fixture("conflict_case.json"))
        .expect("conflict fixture should build");
    let conflict_report = output
        .packet
        .conflict_report
        .as_ref()
        .expect("conflict report should be present");
    assert!(!conflict_report.conflicts.is_empty());
}

#[test]
fn test_t5_memory_only_citation_is_rejected() {
    let mut request = request_from_fixture("external_only.json");
    request.evidence_packet = load_fixture("external_only.json")
        .get("evidence_packet")
        .cloned()
        .expect("evidence packet exists");
    request
        .evidence_packet
        .pointer_mut("/trust_metadata/merge/top_findings/0/citations")
        .expect("fixture path should exist")
        .as_array_mut()
        .expect("citations should be array")
        .splice(.., vec![Value::String("memory_only_ref".to_string())]);

    let err = build_merge_packet(request).expect_err("memory-only citations must fail");
    assert_eq!(err.reason_code, "policy_violation");
}

#[test]
fn test_t6_insufficient_evidence_behavior_is_deterministic() {
    let first = build_merge_packet(request_from_fixture("insufficient_evidence_case.json"))
        .expect("insufficient evidence fixture should still build");
    let second = build_merge_packet(request_from_fixture("insufficient_evidence_case.json"))
        .expect("insufficient evidence fixture replay should build");

    assert_eq!(first.packet, second.packet);
    assert!(first
        .packet
        .reason_codes
        .iter()
        .any(|reason_code| reason_code == "insufficient_evidence"));
    assert!(first.packet.delta.changes_since_last_time.is_empty());
}

#[test]
fn test_t7_same_inputs_produce_identical_merge_packet() {
    let first = build_merge_packet(request_from_fixture("external_only.json"))
        .expect("first merge build should pass");
    let second = build_merge_packet(request_from_fixture("external_only.json"))
        .expect("second merge build should pass");
    assert_eq!(first.packet, second.packet);
}

#[test]
fn test_snapshot_hashes_match_expected_fixture() {
    let cases = [
        ("internal_only", "internal_only.json"),
        ("external_only", "external_only.json"),
        ("consistent_case", "consistent_case.json"),
        ("conflict_case", "conflict_case.json"),
        ("insufficient_evidence_case", "insufficient_evidence_case.json"),
    ];
    let expected = load_fixture("expected_merge_packet.json")
        .as_object()
        .cloned()
        .expect("expected hash fixture must be object");

    let mut actual = BTreeMap::new();
    for (key, fixture_name) in cases {
        let output = build_merge_packet(request_from_fixture(fixture_name))
            .unwrap_or_else(|error: MergeBuildError| {
                panic!("merge fixture {} should build: {}", fixture_name, error.message)
            });
        let packet_value =
            serde_json::to_value(output.packet).expect("merge packet should serialize");
        let hash = hash_canonical_json(&packet_value).expect("hash should compute");
        actual.insert(key.to_string(), hash);
    }

    for (key, hash) in actual {
        let expected_hash = expected
            .get(key.as_str())
            .and_then(Value::as_str)
            .expect("expected hash should exist for case");
        assert_eq!(hash, expected_hash, "snapshot hash mismatch for {}", key);
    }
}
