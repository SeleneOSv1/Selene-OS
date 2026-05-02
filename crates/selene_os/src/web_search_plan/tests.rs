#![forbid(unsafe_code)]

use crate::web_search_plan::contract_hash::{computed_manifest, verify_contract_hash_manifest};
use crate::web_search_plan::idempotency_validator::validate_idempotency_registry;
use crate::web_search_plan::packet_validator::{
    packet_name_from_fixture_filename, required_fixture_pair, validate_packet,
    validate_packet_schema_registry,
};
use crate::web_search_plan::reason_code_validator::{
    validate_reason_code_registry, validate_reason_codes_registered,
};
use crate::web_search_plan::registry_loader::{
    fixtures_dir, load_handoff_map, load_idempotency_registry, load_ownership_matrix,
    load_packet_schema_registry, load_reason_code_registry, load_turn_state_machine,
};
use crate::web_search_plan::turn_state_machine_validator::{
    validate_fail_closed_reason_codes, validate_transition_sequence,
    validate_turn_state_machine_spec,
};
use selene_engines::ph1providerctl::{
    run_stage9_offline_search_certification, ProviderCostClass, ProviderLane, Stage9ReadinessClass,
};
use serde_json::Value;
use std::fs;

fn load_fixture_json(kind: &str, name: &str) -> Result<Value, String> {
    let path = fixtures_dir(kind).join(name);
    let text = fs::read_to_string(&path)
        .map_err(|e| format!("failed reading fixture {}: {}", path.display(), e))?;
    serde_json::from_str::<Value>(&text)
        .map_err(|e| format!("invalid fixture JSON {}: {}", path.display(), e))
}

fn load_core() -> Result<
    (
        crate::web_search_plan::registry_loader::PacketSchemaRegistry,
        crate::web_search_plan::registry_loader::ReasonCodeRegistry,
    ),
    String,
> {
    let packet_registry = load_packet_schema_registry()?;
    validate_packet_schema_registry(&packet_registry)?;

    let reason_registry = load_reason_code_registry()?;
    validate_reason_code_registry(&reason_registry)?;

    Ok((packet_registry, reason_registry))
}

fn validate_single_fixture(kind: &str, name: &str) -> Result<(), String> {
    let (packet_registry, reason_registry) = load_core()?;
    let packet_name = packet_name_from_fixture_filename(name)
        .ok_or_else(|| format!("unknown fixture file mapping for {}", name))?;
    let value = load_fixture_json(kind, name)?;
    validate_packet(packet_name, &value, &packet_registry)?;

    if let Some(reason_codes) = value.get("reason_codes").and_then(Value::as_array) {
        let parsed: Vec<String> = reason_codes
            .iter()
            .map(|v| {
                v.as_str()
                    .ok_or_else(|| format!("reason_codes entry in {} must be string", name))
                    .map(ToString::to_string)
            })
            .collect::<Result<Vec<String>, String>>()?;
        validate_reason_codes_registered(&parsed, &reason_registry)?;
    }

    Ok(())
}

#[test]
fn test_valid_fixtures_pass() {
    let valid_names = [
        "turn_input.json",
        "search_assist.json",
        "tool_request.json",
        "evidence.json",
        "synthesis.json",
        "write.json",
        "temporal_comparison.json",
        "competitive_comparison.json",
        "risk.json",
        "enterprise_report.json",
        "computation.json",
        "vision_tool_request.json",
        "vision_evidence.json",
        "merge.json",
        "audit.json",
    ];

    for name in valid_names {
        validate_single_fixture("valid", name)
            .unwrap_or_else(|e| panic!("valid fixture {} failed validation: {}", name, e));
    }
}

#[test]
fn test_invalid_fixtures_fail() {
    let invalid_names = [
        "turn_input_missing_required.json",
        "search_assist_missing_required.json",
        "tool_request_missing_required.json",
        "tool_request_bad_mode.json",
        "evidence_missing_required.json",
        "evidence_bad_schema_version.json",
        "synthesis_missing_required.json",
        "write_missing_required.json",
        "audit_missing_required.json",
        "audit_missing_hashes.json",
        "temporal_comparison_missing_required.json",
        "competitive_comparison_missing_required.json",
        "risk_missing_required.json",
        "enterprise_report_missing_required.json",
        "computation_missing_required.json",
        "vision_tool_request_missing_required.json",
        "vision_tool_request_missing_asset_ref.json",
        "vision_evidence_missing_required.json",
        "vision_evidence_missing_outputs.json",
        "merge_missing_required.json",
        "unknown_reason_code.json",
    ];

    for name in invalid_names {
        let result = validate_single_fixture("invalid", name);
        assert!(
            result.is_err(),
            "invalid fixture {} unexpectedly passed validation",
            name
        );
    }
}

#[test]
fn test_registered_packets_have_required_fixture_pairs() {
    let packet_registry = load_packet_schema_registry().expect("packet registry must load");

    for packet in &packet_registry.packets {
        let (valid_name, invalid_name) = required_fixture_pair(packet.packet_name.as_str())
            .unwrap_or_else(|| panic!("missing fixture naming rule for {}", packet.packet_name));

        let valid_path = fixtures_dir("valid").join(valid_name.as_str());
        assert!(
            valid_path.exists(),
            "missing valid fixture for {} at {}",
            packet.packet_name,
            valid_path.display()
        );
        let invalid_path = fixtures_dir("invalid").join(invalid_name.as_str());
        assert!(
            invalid_path.exists(),
            "missing invalid fixture for {} at {}",
            packet.packet_name,
            invalid_path.display()
        );
    }
}

#[test]
fn test_fixture_files_map_to_registered_packets() {
    let packet_registry = load_packet_schema_registry().expect("packet registry must load");
    let packet_names = packet_registry
        .packets
        .iter()
        .map(|entry| entry.packet_name.as_str())
        .collect::<std::collections::BTreeSet<&str>>();

    for kind in ["valid", "invalid"] {
        let dir = fixtures_dir(kind);
        let entries = fs::read_dir(&dir)
            .unwrap_or_else(|err| panic!("failed reading fixture dir {}: {}", dir.display(), err));
        for entry in entries {
            let entry = entry.expect("fixture entry should be readable");
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let packet_name = packet_name_from_fixture_filename(file_name)
                .unwrap_or_else(|| panic!("fixture {} has no packet mapping rule", path.display()));
            assert!(
                packet_names.contains(packet_name),
                "fixture {} maps to unregistered packet {}",
                path.display(),
                packet_name
            );
        }
    }
}

#[test]
fn test_unknown_reason_code_fails() {
    let (packet_registry, reason_registry) = load_core().expect("core registries must load");
    let value = load_fixture_json("invalid", "unknown_reason_code.json")
        .expect("fixture should be readable JSON");
    validate_packet("AuditPacket", &value, &packet_registry)
        .expect("packet shape should be valid before reason-code check");
    let reason_codes = value
        .get("reason_codes")
        .and_then(Value::as_array)
        .expect("reason_codes must exist for fixture");
    let parsed: Vec<String> = reason_codes
        .iter()
        .map(|v| {
            v.as_str()
                .expect("reason code should be string")
                .to_string()
        })
        .collect();
    let err = validate_reason_codes_registered(&parsed, &reason_registry)
        .expect_err("unknown reason code must fail validation");
    assert!(err.contains("unknown reason code"));
}

#[test]
fn test_turn_state_machine_valid_path() {
    let machine = load_turn_state_machine().expect("state machine should load");
    validate_turn_state_machine_spec(&machine).expect("state machine spec must be valid");

    let path = vec![
        "TURN_ACCEPTED".to_string(),
        "INPUT_PARSED".to_string(),
        "INTENT_CLASSIFIED".to_string(),
        "PLAN_SELECTED".to_string(),
        "RETRIEVAL_EXECUTED".to_string(),
        "EVIDENCE_LOCKED".to_string(),
        "SYNTHESIS_READY".to_string(),
        "OUTPUT_RENDERED".to_string(),
        "AUDIT_COMMITTED".to_string(),
        "TURN_COMPLETED".to_string(),
    ];
    validate_transition_sequence(&machine, &path).expect("valid path must pass");
}

#[test]
fn test_turn_state_machine_fail_closed_requires_reason() {
    let machine = load_turn_state_machine().expect("state machine should load");
    let fail_path = vec![
        "TURN_ACCEPTED".to_string(),
        "INPUT_PARSED".to_string(),
        "TURN_FAILED_CLOSED".to_string(),
    ];
    validate_transition_sequence(&machine, &fail_path)
        .expect("fail-closed path transition should be legal");
    let err = validate_fail_closed_reason_codes(&machine, &fail_path, &[])
        .expect_err("fail-closed without reason code must fail");
    assert!(err.contains("requires at least one reason code"));
}

#[test]
fn test_contract_hash_manifest_matches() {
    verify_contract_hash_manifest().expect("contract hash manifest must match computed values");
}

#[test]
fn test_idempotency_registry_foundation_entries_present() {
    let registry = load_idempotency_registry().expect("idempotency registry must load");
    validate_idempotency_registry(&registry).expect("idempotency registry must be valid");
}

#[test]
fn test_handoff_map_packet_refs_exist() {
    let packet_registry = load_packet_schema_registry().expect("packet registry must load");
    let packet_names: std::collections::BTreeSet<&str> = packet_registry
        .packets
        .iter()
        .map(|packet| packet.packet_name.as_str())
        .collect();

    let handoff_map = load_handoff_map().expect("handoff map must load");
    assert!(!handoff_map.handoffs.is_empty());
    for handoff in handoff_map.handoffs {
        assert!(
            packet_names.contains(handoff.packet_type.as_str()),
            "handoff references unknown packet {}",
            handoff.packet_type
        );
        assert!(
            handoff.authority == "authoritative" || handoff.authority == "non_authoritative",
            "handoff has invalid authority {}",
            handoff.authority
        );
    }
}

#[test]
fn test_ownership_matrix_engine_ids_are_well_formed() {
    let ownership = load_ownership_matrix().expect("ownership matrix must load");
    assert!(!ownership.engines.is_empty());
    for engine in ownership.engines {
        assert!(
            is_valid_engine_id(engine.engine_id.as_str()),
            "invalid engine id format {}",
            engine.engine_id
        );
        assert!(
            engine.authority == "authoritative" || engine.authority == "non_authoritative",
            "invalid authority {}",
            engine.authority
        );
    }
}

fn is_valid_engine_id(engine_id: &str) -> bool {
    if engine_id == "API" {
        return true;
    }
    if !engine_id.starts_with("PH1.") {
        return false;
    }
    engine_id.split('.').skip(1).all(|segment| {
        !segment.is_empty()
            && segment
                .chars()
                .all(|ch| ch.is_ascii_uppercase() || ch == '_' || ch.is_ascii_digit())
    })
}

#[test]
fn test_contract_hash_manifest_can_be_computed() {
    let manifest = computed_manifest().expect("computed manifest should build");
    assert_eq!(manifest.manifest_version, "1.0.0");
    assert_eq!(manifest.hash_algorithm, "sha256");
    assert_eq!(manifest.hashes.packet_schema_hash.len(), 64);
}

#[test]
fn test_computation_valid_fixture_passes() {
    validate_single_fixture("valid", "computation.json")
        .expect("valid computation fixture must pass");
}

#[test]
fn test_computation_invalid_fixture_fails() {
    let result = validate_single_fixture("invalid", "computation_missing_required.json");
    assert!(
        result.is_err(),
        "invalid computation fixture unexpectedly passed"
    );
}

#[test]
fn test_temporal_comparison_valid_fixture_passes() {
    validate_single_fixture("valid", "temporal_comparison.json")
        .expect("valid temporal comparison fixture must pass");
}

#[test]
fn test_temporal_comparison_invalid_fixture_fails() {
    let result = validate_single_fixture("invalid", "temporal_comparison_missing_required.json");
    assert!(
        result.is_err(),
        "invalid temporal comparison fixture unexpectedly passed"
    );
}

#[test]
fn test_competitive_comparison_valid_fixture_passes() {
    validate_single_fixture("valid", "competitive_comparison.json")
        .expect("valid competitive comparison fixture must pass");
}

#[test]
fn test_competitive_comparison_invalid_fixture_fails() {
    let result = validate_single_fixture("invalid", "competitive_comparison_missing_required.json");
    assert!(
        result.is_err(),
        "invalid competitive comparison fixture unexpectedly passed"
    );
}

#[test]
fn test_enterprise_report_valid_fixture_passes() {
    validate_single_fixture("valid", "enterprise_report.json")
        .expect("valid enterprise report fixture must pass");
}

#[test]
fn test_enterprise_report_invalid_fixture_fails() {
    let result = validate_single_fixture("invalid", "enterprise_report_missing_required.json");
    assert!(
        result.is_err(),
        "invalid enterprise report fixture unexpectedly passed"
    );
}

#[test]
fn test_vision_tool_request_valid_fixture_passes() {
    validate_single_fixture("valid", "vision_tool_request.json")
        .expect("valid vision tool request fixture must pass");
}

#[test]
fn test_vision_evidence_valid_fixture_passes() {
    validate_single_fixture("valid", "vision_evidence.json")
        .expect("valid vision evidence fixture must pass");
}

#[test]
fn test_vision_tool_request_invalid_fixture_fails() {
    let result = validate_single_fixture("invalid", "vision_tool_request_missing_asset_ref.json");
    assert!(
        result.is_err(),
        "invalid vision tool request fixture unexpectedly passed"
    );
}

#[test]
fn test_vision_evidence_invalid_fixture_fails() {
    let result = validate_single_fixture("invalid", "vision_evidence_missing_outputs.json");
    assert!(
        result.is_err(),
        "invalid vision evidence fixture unexpectedly passed"
    );
}

#[test]
fn test_merge_valid_fixture_passes() {
    validate_single_fixture("valid", "merge.json").expect("valid merge fixture must pass");
}

#[test]
fn test_merge_invalid_fixture_fails() {
    let result = validate_single_fixture("invalid", "merge_missing_required.json");
    assert!(result.is_err(), "invalid merge fixture unexpectedly passed");
}

#[test]
fn stage9_search_certification_os_accepts_offline_report_without_live_provider_calls() {
    let report = run_stage9_offline_search_certification();

    assert_eq!(report.total_cases, 30);
    assert_eq!(report.pass_count, 30);
    assert_eq!(report.fail_count, 0);
    assert_eq!(report.live_provider_call_attempt_count, 0);
    assert_eq!(report.live_provider_network_dispatch_count, 0);
    assert_eq!(report.performance_packet.provider_call_count, 0);
    assert_eq!(report.performance_packet.url_fetch_count, 0);
    assert_eq!(report.performance_packet.image_fetch_count, 0);
    assert_eq!(
        report.production_readiness_verdict,
        Stage9ReadinessClass::ReadyExceptRealVoiceNotProven
    );
}

#[test]
fn stage9_search_certification_os_preserves_provider_lane_cache_cost_and_source_chip_proof() {
    let report = run_stage9_offline_search_certification();

    assert!(report
        .case_results
        .iter()
        .any(|case| case.selected_lane == ProviderLane::CacheOnly));
    assert!(report
        .case_results
        .iter()
        .any(|case| case.selected_lane == ProviderLane::CheapGeneralSearch));
    assert!(report
        .case_results
        .iter()
        .any(|case| case.selected_lane == ProviderLane::NewsCurrentEvents));
    assert!(report
        .case_results
        .iter()
        .any(|case| case.selected_lane == ProviderLane::PremiumFallback));
    assert_eq!(report.cost_class, ProviderCostClass::FreeOrInternal);
    assert!(report
        .case_results
        .iter()
        .filter(|case| {
            !matches!(
                case.selected_lane,
                ProviderLane::NoSearch | ProviderLane::Disabled
            )
        })
        .all(|case| case.source_chip_count > 0));
}
