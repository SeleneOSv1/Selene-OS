#![forbid(unsafe_code)]

use crate::web_search_plan::enterprise::consistency::{validate_cross_mode_consistency, ConsistencyInputs};
use crate::web_search_plan::enterprise::enterprise_pipeline::run_enterprise_pipeline;
use crate::web_search_plan::enterprise::enterprise_request::{EnterpriseConstraints, EnterpriseRequest};
use crate::web_search_plan::enterprise::mode_router::EnterpriseMode;
use crate::web_search_plan::merge::{InternalContext, InternalSourceType};
use crate::web_search_plan::replay::snapshot::hash_canonical_json;
use crate::web_search_plan::structured::types::StructuredRow;
use serde::Deserialize;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
struct RowsFixture {
    rows: Vec<StructuredRow>,
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/web_search_plan/enterprise_fixtures")
}

fn load_json(path: &PathBuf) -> Value {
    let text = fs::read_to_string(path).expect("fixture should load");
    serde_json::from_str(&text).expect("fixture should parse")
}

fn load_enterprise_fixture(name: &str) -> Value {
    load_json(&fixture_dir().join(name))
}

fn load_competitive_fixture(name: &str) -> Value {
    load_json(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/web_search_plan/competitive_fixtures")
            .join(name),
    )
}

fn load_temporal_rows(name: &str) -> Vec<StructuredRow> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/web_search_plan/temporal_fixtures")
        .join(name);
    let text = fs::read_to_string(path).expect("temporal fixture should load");
    let parsed: RowsFixture = serde_json::from_str(&text).expect("temporal fixture should parse");
    parsed.rows
}

fn load_temporal_missing_fixture() -> Value {
    load_json(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/web_search_plan/temporal_fixtures/missing_timestamps.json"),
    )
}

fn merge_context() -> InternalContext {
    InternalContext {
        prior_summary: "Acme previously reported steady uptime metrics.".to_string(),
        prior_key_points: vec![
            "Status page reports 99.99% uptime SLA.".to_string(),
            "Starter plan costs $10 monthly.".to_string(),
        ],
        prior_timestamp_ms: 1_702_000_000_000,
        internal_source_type: InternalSourceType::PriorReport,
    }
}

fn base_request(mode: EnterpriseMode, trace_id: &str, query: &str) -> EnterpriseRequest {
    EnterpriseRequest {
        trace_id: trace_id.to_string(),
        query: query.to_string(),
        mode,
        importance_tier: "high".to_string(),
        created_at_ms: 1_703_000_000_000,
        policy_snapshot_id: "policy-snapshot-default".to_string(),
        jurisdiction: None,
        as_of_from_ms: None,
        as_of_to_ms: None,
        constraints: EnterpriseConstraints::default(),
        target_entity: None,
        tool_request_packet: None,
        evidence_packet: None,
        structured_rows: None,
        computation_packet: None,
        internal_context: None,
    }
}

fn hash_from_expected_fixture(name: &str, field: &str) -> String {
    load_enterprise_fixture(name)
        .get(field)
        .and_then(Value::as_str)
        .expect("expected hash field should exist")
        .to_string()
}

#[test]
fn test_t1_enterprise_request_routing_is_deterministic() {
    let request_json = json!({
        "trace_id": "trace-enterprise-route",
        "query": "compare acme pricing against competitors",
        "created_at_ms": 1703000000000_i64,
        "importance_tier": "high"
    });
    let first = EnterpriseRequest::parse_from_tool_request(&request_json)
        .expect("first parse should pass");
    let second = EnterpriseRequest::parse_from_tool_request(&request_json)
        .expect("second parse should pass");
    assert_eq!(first.mode, EnterpriseMode::Competitive);
    assert_eq!(first.mode, second.mode);

    let explicit_json = json!({
        "trace_id": "trace-enterprise-route-explicit",
        "query": "compare acme pricing against competitors",
        "created_at_ms": 1703000000000_i64,
        "enterprise_mode": "risk",
        "importance_tier": "medium"
    });
    let explicit = EnterpriseRequest::parse_from_tool_request(&explicit_json)
        .expect("explicit parse should pass");
    assert_eq!(explicit.mode, EnterpriseMode::Risk);
}

#[test]
fn test_t2_pipeline_composes_in_deterministic_order() {
    let mut request = base_request(
        EnterpriseMode::Report,
        "trace-enterprise-order",
        "generate enterprise report for acme",
    );
    request.evidence_packet = Some(load_enterprise_fixture("enterprise_evidence_packet.json"));
    request.computation_packet = Some(load_enterprise_fixture("enterprise_computation_packet.json"));
    request.internal_context = Some(merge_context());

    let output = run_enterprise_pipeline(&request, 1_703_000_000_000)
        .expect("enterprise report should build");
    assert_eq!(
        output.stage_trace,
        vec![
            "base_evidence:provided".to_string(),
            "trust_enrichment".to_string(),
            "computation:provided".to_string(),
            "mode_output:report.compose".to_string(),
            "consistency_validation".to_string(),
        ]
    );
    assert!(output.risk_packet.is_some());
    assert!(output.merge_packet.is_some());
    assert!(output.report_packet.is_some());
}

#[test]
fn test_t3_consistency_validator_fails_on_uncited_claim() {
    let evidence = json!({
        "sources": [{"url": "https://example.com/a"}],
        "content_chunks": []
    });
    let report = json!({
        "claims": [
            {
                "text": "This is an uncited claim",
                "citations": []
            }
        ]
    });
    let error = validate_cross_mode_consistency(ConsistencyInputs {
        evidence_packet: &evidence,
        competitive_packet: None,
        temporal_packet: None,
        risk_packet: None,
        merge_packet: None,
        report_packet: Some(&report),
    })
    .expect_err("uncited report claim must fail");
    assert_eq!(error.reason_code, "policy_violation");
}

#[test]
fn test_t4_consistency_validator_fails_on_missing_evidence_ref() {
    let evidence = json!({
        "sources": [{"url": "https://example.com/a"}],
        "content_chunks": []
    });
    let risk = json!({
        "factor_breakdown": [
            {"factor_id": "financial_stress", "evidence_refs": ["https://missing.example/ref"]}
        ],
        "evidence_refs": ["https://missing.example/ref"]
    });
    let error = validate_cross_mode_consistency(ConsistencyInputs {
        evidence_packet: &evidence,
        competitive_packet: None,
        temporal_packet: None,
        risk_packet: Some(&risk),
        merge_packet: None,
        report_packet: None,
    })
    .expect_err("missing evidence refs must fail");
    assert_eq!(error.reason_code, "policy_violation");
}

#[test]
fn test_t5_same_evidence_produces_identical_outputs_and_hashes() {
    let mut request = base_request(
        EnterpriseMode::Merge,
        "trace-enterprise-replay",
        "what changed since last report for acme",
    );
    request.evidence_packet = Some(load_enterprise_fixture("enterprise_evidence_packet.json"));
    request.internal_context = Some(merge_context());

    let first = run_enterprise_pipeline(&request, 1_703_000_000_000)
        .expect("first merge run should pass");
    let second = run_enterprise_pipeline(&request, 1_703_000_000_000)
        .expect("second merge run should pass");
    assert_eq!(first, second);

    let first_merge_hash = hash_canonical_json(
        first
            .merge_packet
            .as_ref()
            .expect("merge packet should be present"),
    )
    .expect("first merge hash should compute");
    let second_merge_hash = hash_canonical_json(
        second
            .merge_packet
            .as_ref()
            .expect("merge packet should be present"),
    )
    .expect("second merge hash should compute");
    assert_eq!(first_merge_hash, second_merge_hash);
}

#[test]
fn test_t6_provenance_record_stable_and_complete() {
    let mut request = base_request(
        EnterpriseMode::Report,
        "trace-enterprise-provenance",
        "generate enterprise report for acme",
    );
    request.evidence_packet = Some(load_enterprise_fixture("enterprise_evidence_packet.json"));
    request.computation_packet = Some(load_enterprise_fixture("enterprise_computation_packet.json"));
    request.internal_context = Some(merge_context());

    let output = run_enterprise_pipeline(&request, 1_703_000_000_000)
        .expect("report run should succeed");
    assert_eq!(output.provenance.mode_selected, "report".to_string());
    assert_eq!(output.provenance.evidence_hash.len(), 64);
    assert!(output.provenance.output_packet_hashes.contains_key("risk"));
    assert!(output.provenance.output_packet_hashes.contains_key("merge"));
    assert!(output.provenance.output_packet_hashes.contains_key("report"));

    let provenance_value = serde_json::to_value(&output.provenance)
        .expect("provenance should serialize");
    let provenance_hash = hash_canonical_json(&provenance_value)
        .expect("provenance hash should compute");
    let expected_hash = hash_from_expected_fixture("expected_provenance.json", "provenance_hash");
    assert_eq!(provenance_hash, expected_hash);
}

#[test]
fn test_snapshot_hashes_match_expected_packets() {
    let competitive_fixture = load_competitive_fixture("pricing_competitors.json");
    let mut competitive_request = base_request(
        EnterpriseMode::Competitive,
        "trace-enterprise-competitive",
        "compare selene os against competitors",
    );
    competitive_request.target_entity = Some(
        competitive_fixture
            .get("target_entity")
            .and_then(Value::as_str)
            .expect("target_entity should exist")
            .to_string(),
    );
    competitive_request.evidence_packet = Some(
        competitive_fixture
            .get("evidence_packet")
            .cloned()
            .expect("evidence_packet should exist"),
    );
    competitive_request.structured_rows = Some(
        serde_json::from_value(
            competitive_fixture
                .get("structured_rows")
                .cloned()
                .expect("structured_rows should exist"),
        )
        .expect("structured rows should deserialize"),
    );
    competitive_request.computation_packet = Some(
        competitive_fixture
            .get("computation_packet")
            .cloned()
            .expect("computation_packet should exist"),
    );
    let competitive_output = run_enterprise_pipeline(&competitive_request, 1_703_000_000_000)
        .expect("competitive pipeline should succeed");
    let competitive_hash = hash_canonical_json(
        competitive_output
            .competitive_packet
            .as_ref()
            .expect("competitive packet should exist"),
    )
    .expect("competitive hash should compute");
    let expected_competitive =
        hash_from_expected_fixture("expected_competitive.json", "packet_hash");

    let temporal_missing = load_temporal_missing_fixture();
    let mut temporal_rows = load_temporal_rows("baseline_rows.json");
    temporal_rows.extend(load_temporal_rows("compare_rows.json"));
    let allowed_temporal_sources = temporal_missing
        .pointer("/evidence_packet/sources")
        .and_then(Value::as_array)
        .map(|sources| {
            sources
                .iter()
                .filter_map(|source| source.get("url").and_then(Value::as_str))
                .map(ToString::to_string)
                .collect::<std::collections::BTreeSet<String>>()
        })
        .unwrap_or_default();
    temporal_rows.retain(|row| allowed_temporal_sources.contains(row.source_url.as_str()));
    for row in &mut temporal_rows {
        row.source_ref = row.source_url.clone();
    }
    let mut temporal_request = base_request(
        EnterpriseMode::Temporal,
        "trace-enterprise-temporal",
        "show acme timeline changes",
    );
    temporal_request.evidence_packet = Some(
        temporal_missing
            .get("evidence_packet")
            .cloned()
            .expect("temporal evidence should exist"),
    );
    temporal_request.structured_rows = Some(temporal_rows);
    temporal_request.as_of_from_ms = None;
    temporal_request.as_of_to_ms = None;
    let temporal_output = run_enterprise_pipeline(&temporal_request, 1_703_000_000_000)
        .expect("temporal pipeline should succeed");
    let temporal_hash = hash_canonical_json(
        temporal_output
            .temporal_packet
            .as_ref()
            .expect("temporal packet should exist"),
    )
    .expect("temporal hash should compute");
    let expected_temporal = hash_from_expected_fixture("expected_temporal.json", "packet_hash");

    let mut risk_request = base_request(
        EnterpriseMode::Risk,
        "trace-enterprise-risk",
        "assess acme risk",
    );
    risk_request.evidence_packet = Some(load_enterprise_fixture("enterprise_evidence_packet.json"));
    risk_request.computation_packet = Some(load_enterprise_fixture("enterprise_computation_packet.json"));
    let risk_output = run_enterprise_pipeline(&risk_request, 1_703_000_000_000)
        .expect("risk pipeline should succeed");
    let risk_hash = hash_canonical_json(
        risk_output
            .risk_packet
            .as_ref()
            .expect("risk packet should exist"),
    )
    .expect("risk hash should compute");
    let expected_risk = hash_from_expected_fixture("expected_risk.json", "packet_hash");

    let mut merge_request = base_request(
        EnterpriseMode::Merge,
        "trace-enterprise-merge",
        "what changed since last report for acme",
    );
    merge_request.evidence_packet = Some(load_enterprise_fixture("enterprise_evidence_packet.json"));
    merge_request.internal_context = Some(merge_context());
    let merge_output = run_enterprise_pipeline(&merge_request, 1_703_000_000_000)
        .expect("merge pipeline should succeed");
    let merge_hash = hash_canonical_json(
        merge_output
            .merge_packet
            .as_ref()
            .expect("merge packet should exist"),
    )
    .expect("merge hash should compute");
    let expected_merge = hash_from_expected_fixture("expected_merge.json", "packet_hash");

    let mut report_request = base_request(
        EnterpriseMode::Report,
        "trace-enterprise-report",
        "generate enterprise report for acme",
    );
    report_request.evidence_packet = Some(load_enterprise_fixture("enterprise_evidence_packet.json"));
    report_request.computation_packet = Some(load_enterprise_fixture("enterprise_computation_packet.json"));
    report_request.internal_context = Some(merge_context());
    let report_output = run_enterprise_pipeline(&report_request, 1_703_000_000_000)
        .expect("report pipeline should succeed");
    let report_hash = hash_canonical_json(
        report_output
            .report_packet
            .as_ref()
            .expect("report packet should exist"),
    )
    .expect("report hash should compute");
    let expected_report = hash_from_expected_fixture("expected_report.json", "packet_hash");

    assert_eq!(
        competitive_hash, expected_competitive,
        "competitive packet hash mismatch"
    );
    assert_eq!(temporal_hash, expected_temporal, "temporal packet hash mismatch");
    assert_eq!(risk_hash, expected_risk, "risk packet hash mismatch");
    assert_eq!(merge_hash, expected_merge, "merge packet hash mismatch");
    assert_eq!(
        report_hash, expected_report,
        "report packet hash mismatch"
    );
}
