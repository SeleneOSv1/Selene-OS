#![forbid(unsafe_code)]

use crate::web_search_plan::packet_validator::{validate_packet, validate_packet_schema_registry};
use crate::web_search_plan::reason_code_validator::{
    validate_reason_code_registry, validate_reason_codes_registered,
};
use crate::web_search_plan::registry_loader::{load_packet_schema_registry, load_reason_code_registry};
use crate::web_search_plan::replay::corpus::{
    load_replay_corpus, replay_fixture_path, validate_replay_corpus, ReplayCase,
};
use crate::web_search_plan::replay::metrics::{compute_quality_metrics, ReplayMetrics};
use crate::web_search_plan::replay::regressions::{evaluate_regressions, load_replay_expected};
use crate::web_search_plan::replay::snapshot::{build_snapshot, ReplaySnapshot};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayFixtureCase {
    pub case_id: String,
    pub stop_reason: String,
    pub simulated_latency_ms: u64,
    pub evidence_packet: Value,
    #[serde(default)]
    pub synthesis_packet: Option<Value>,
    #[serde(default)]
    pub write_packet: Option<Value>,
    #[serde(default)]
    pub audit_packet: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayCaseResult {
    pub case_id: String,
    pub snapshot: ReplaySnapshot,
    pub metrics: ReplayMetrics,
}

pub fn load_fixture_case(case_id: &str) -> Result<ReplayFixtureCase, String> {
    let full_path = replay_fixture_path(case_id);
    let text = fs::read_to_string(&full_path)
        .map_err(|e| format!("failed to read fixture {}: {}", full_path.display(), e))?;
    serde_json::from_str::<ReplayFixtureCase>(&text)
        .map_err(|e| format!("invalid replay fixture JSON {}: {}", full_path.display(), e))
}

pub fn run_replay_corpus() -> Result<Vec<ReplayCaseResult>, String> {
    let corpus = load_replay_corpus()?;
    validate_replay_corpus(&corpus)?;

    let packet_registry = load_packet_schema_registry()?;
    validate_packet_schema_registry(&packet_registry)?;

    let reason_registry = load_reason_code_registry()?;
    validate_reason_code_registry(&reason_registry)?;

    let mut results = Vec::new();

    for case in &corpus.cases {
        let fixture = load_fixture_case(case.case_id.as_str())?;
        validate_fixture(case, &fixture, &packet_registry, &reason_registry)?;

        let snapshot_first = build_snapshot(
            case.case_id.as_str(),
            fixture.stop_reason.as_str(),
            &fixture.evidence_packet,
            fixture.synthesis_packet.as_ref(),
            fixture.write_packet.as_ref(),
            fixture.audit_packet.as_ref(),
        )?;
        let snapshot_second = build_snapshot(
            case.case_id.as_str(),
            fixture.stop_reason.as_str(),
            &fixture.evidence_packet,
            fixture.synthesis_packet.as_ref(),
            fixture.write_packet.as_ref(),
            fixture.audit_packet.as_ref(),
        )?;
        let determinism_ok = snapshot_first == snapshot_second;

        let metrics = compute_quality_metrics(
            case,
            fixture.synthesis_packet.as_ref(),
            fixture.simulated_latency_ms,
            &snapshot_first,
            determinism_ok,
        )?;

        results.push(ReplayCaseResult {
            case_id: case.case_id.clone(),
            snapshot: snapshot_first,
            metrics,
        });
    }

    Ok(results)
}

pub fn run_replay_with_regression_gate() -> Result<Vec<ReplayCaseResult>, String> {
    let results = run_replay_corpus()?;
    let expected = load_replay_expected()?;
    evaluate_regressions(&results, &expected)?;
    Ok(results)
}

fn validate_fixture(
    case: &ReplayCase,
    fixture: &ReplayFixtureCase,
    packet_registry: &crate::web_search_plan::registry_loader::PacketSchemaRegistry,
    reason_registry: &crate::web_search_plan::registry_loader::ReasonCodeRegistry,
) -> Result<(), String> {
    if case.case_id != fixture.case_id {
        return Err(format!(
            "fixture case_id mismatch expected={} actual={}",
            case.case_id, fixture.case_id
        ));
    }

    validate_packet("EvidencePacket", &fixture.evidence_packet, packet_registry)?;
    if let Some(synthesis) = &fixture.synthesis_packet {
        validate_packet("SynthesisPacket", synthesis, packet_registry)?;
    }
    if let Some(write) = &fixture.write_packet {
        validate_packet("WritePacket", write, packet_registry)?;
    }
    if let Some(audit) = &fixture.audit_packet {
        validate_packet("AuditPacket", audit, packet_registry)?;
    }

    validate_reason_codes_in_packet(&fixture.evidence_packet, reason_registry)?;
    if let Some(synthesis) = &fixture.synthesis_packet {
        validate_reason_codes_in_packet(synthesis, reason_registry)?;
    }
    if let Some(write) = &fixture.write_packet {
        validate_reason_codes_in_packet(write, reason_registry)?;
    }
    if let Some(audit) = &fixture.audit_packet {
        validate_reason_codes_in_packet(audit, reason_registry)?;
    }

    Ok(())
}

fn validate_reason_codes_in_packet(
    packet: &Value,
    reason_registry: &crate::web_search_plan::registry_loader::ReasonCodeRegistry,
) -> Result<(), String> {
    let Some(raw_codes) = packet.get("reason_codes") else {
        return Ok(());
    };

    let entries = raw_codes
        .as_array()
        .ok_or_else(|| "reason_codes must be an array when present".to_string())?;

    let parsed = entries
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .ok_or_else(|| "reason_codes entry must be string".to_string())
                .map(ToString::to_string)
        })
        .collect::<Result<Vec<String>, String>>()?;

    validate_reason_codes_registered(&parsed, reason_registry)
}
