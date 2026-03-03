#![forbid(unsafe_code)]

use crate::web_search_plan::packet_validator::{validate_packet, validate_packet_schema_registry};
use crate::web_search_plan::registry_loader::load_packet_schema_registry;
use crate::web_search_plan::write::{
    append_write_audit_fields, render_write_packet, WriteError, WriteFormatMode,
};
use serde_json::{json, Value};

fn base_synthesis_packet() -> Value {
    json!({
        "schema_version": "1.0.0",
        "produced_by": "PH1.D",
        "intended_consumers": ["PH1.WRITE"],
        "created_at_ms": 1_700_000_600_000i64,
        "trace_id": "trace-run6",
        "answer_text": "Direct Answer\nGrounded response for question: What happened.\n\nEvidence\n- Claim from source A. [chunk:chunk-a] [url:https://example.com/a]\n- Claim from source B. [chunk:chunk-b] [url:https://example.com/b]\n\nCitations\n- chunk:chunk-a\n- url:https://example.com/a\n- chunk:chunk-b\n- url:https://example.com/b",
        "bullet_evidence": [
            "Claim from source A.",
            "Claim from source B."
        ],
        "citations": [
            {"type": "chunk_id", "value": "chunk-a"},
            {"type": "source_url", "value": "https://example.com/a"},
            {"type": "chunk_id", "value": "chunk-b"},
            {"type": "source_url", "value": "https://example.com/b"}
        ],
        "uncertainty_flags": [],
        "reason_codes": [],
        "evidence_refs": ["chunk-a", "chunk-b"],
        "synthesis_template_version": "1.0.0"
    })
}

#[test]
fn test_t1_identical_synthesis_packet_produces_identical_formatted_text() {
    let synthesis = base_synthesis_packet();

    let first = render_write_packet(
        &synthesis,
        1_700_000_600_500i64,
        "trace-run6-t1",
        WriteFormatMode::Standard,
    )
    .expect("first write render should pass");

    let second = render_write_packet(
        &synthesis,
        1_700_000_600_500i64,
        "trace-run6-t1",
        WriteFormatMode::Standard,
    )
    .expect("second write render should pass");

    assert_eq!(
        first
            .write_packet
            .get("formatted_text")
            .and_then(Value::as_str),
        second
            .write_packet
            .get("formatted_text")
            .and_then(Value::as_str)
    );
    assert_eq!(first.write_packet, second.write_packet);
    assert_eq!(first.voice_text, second.voice_text);
    assert_eq!(first.audit_metrics.language_tag, second.audit_metrics.language_tag);
    assert_eq!(first.audit_metrics.language_tag, "en");

    let packet_registry = load_packet_schema_registry().expect("packet schema should load");
    validate_packet_schema_registry(&packet_registry).expect("packet schema should validate");
    validate_packet("WritePacket", &first.write_packet, &packet_registry)
        .expect("write packet should satisfy schema");
}

#[test]
fn test_t2_citation_order_stability() {
    let mut synthesis = base_synthesis_packet();
    synthesis["answer_text"] = json!("Direct Answer\nGrounded response for question: What happened.\n\nEvidence\n- Claim from source B. [chunk:chunk-b] [url:https://example.com/b]\n- Claim from source A. [chunk:chunk-a] [url:https://example.com/a]\n\nCitations\n- url:https://example.com/b\n- url:https://example.com/a");
    synthesis["bullet_evidence"] = json!(["Claim from source B.", "Claim from source A."]);
    synthesis["citations"] = json!([
        {"type": "source_url", "value": "https://example.com/a"},
        {"type": "source_url", "value": "https://example.com/b"},
        {"type": "source_url", "value": "https://example.com/a"}
    ]);

    let rendered = render_write_packet(
        &synthesis,
        1_700_000_600_500i64,
        "trace-run6-t2",
        WriteFormatMode::Standard,
    )
    .expect("write render should pass");

    let formatted = rendered
        .write_packet
        .get("formatted_text")
        .and_then(Value::as_str)
        .expect("formatted_text should exist");

    let citation_lines = lines_after_heading(formatted, "Citations:");
    assert_eq!(
        citation_lines,
        vec![
            "- [C1] https://example.com/b".to_string(),
            "- [C2] https://example.com/a".to_string()
        ]
    );
}

#[test]
fn test_t3_no_semantic_drift_from_synthesis_packet() {
    let synthesis = base_synthesis_packet();
    let rendered = render_write_packet(
        &synthesis,
        1_700_000_600_500i64,
        "trace-run6-t3",
        WriteFormatMode::Standard,
    )
    .expect("write render should pass");

    let formatted = rendered
        .write_packet
        .get("formatted_text")
        .and_then(Value::as_str)
        .expect("formatted_text should exist");

    let evidence_lines = lines_after_heading(formatted, "Evidence:");
    let evidence_claims: Vec<String> = evidence_lines
        .iter()
        .map(|line| {
            line.trim_start_matches("- ")
                .split('[')
                .next()
                .unwrap_or_default()
                .trim()
                .to_string()
        })
        .collect();

    assert_eq!(
        evidence_claims,
        vec![
            "Claim from source A.".to_string(),
            "Claim from source B.".to_string()
        ]
    );
}

#[test]
fn test_t4_missing_citations_are_rejected() {
    let mut synthesis = base_synthesis_packet();
    synthesis["answer_text"] = json!("Direct Answer\nGrounded response for question: What happened.\n\nEvidence\n- Claim from source A. [chunk:chunk-a]\n- Claim from source B. [chunk:chunk-b]\n\nCitations\n- chunk:chunk-a\n- chunk:chunk-b");

    let err = render_write_packet(
        &synthesis,
        1_700_000_600_500i64,
        "trace-run6-t4",
        WriteFormatMode::Standard,
    )
    .expect_err("missing source URL citations must fail");

    assert!(matches!(err, WriteError::CitationMismatch(_)));
}

#[test]
fn test_t5_style_guard_blocks_speculative_language() {
    let mut synthesis = base_synthesis_packet();
    synthesis["answer_text"] = json!("Direct Answer\nMaybe this happened perhaps.\n\nEvidence\n- Claim from source A. [chunk:chunk-a] [url:https://example.com/a]\n- Claim from source B. [chunk:chunk-b] [url:https://example.com/b]\n\nCitations\n- url:https://example.com/a\n- url:https://example.com/b");

    let err = render_write_packet(
        &synthesis,
        1_700_000_600_500i64,
        "trace-run6-t5",
        WriteFormatMode::Standard,
    )
    .expect_err("speculative language should fail style guard");

    assert!(matches!(err, WriteError::StyleGuardViolation(_)));
}

#[test]
fn test_t6_voice_output_matches_formatted_text_semantics() {
    let synthesis = base_synthesis_packet();
    let rendered = render_write_packet(
        &synthesis,
        1_700_000_600_500i64,
        "trace-run6-t6",
        WriteFormatMode::Standard,
    )
    .expect("write render should pass");

    assert!(rendered.voice_text.contains("Direct Answer:"));
    assert!(rendered.voice_text.contains("Evidence:"));
    assert!(rendered.voice_text.contains("Citations:"));
    assert!(rendered.voice_text.contains("Claim from source A."));
    assert!(rendered.voice_text.contains("Claim from source B."));
    assert!(!rendered.voice_text.contains("**"));
    assert!(!rendered.voice_text.contains("`"));
    assert!(rendered.voice_text.contains("[PAUSE_SHORT]"));
}

#[test]
fn test_t7_localization_contract_blocks_mid_response_switching() {
    let mut synthesis = base_synthesis_packet();
    synthesis["answer_text"] = json!("Direct Answer\nGrounded response in English.\n\nEvidence\n- 中文结论来自来源 A. [chunk:chunk-a] [url:https://example.com/a]\n- Claim from source B. [chunk:chunk-b] [url:https://example.com/b]\n\nCitations\n- url:https://example.com/a\n- url:https://example.com/b");
    synthesis["bullet_evidence"] = json!(["中文结论来自来源 A.", "Claim from source B."]);

    let err = render_write_packet(
        &synthesis,
        1_700_000_600_500i64,
        "trace-run13-t7",
        WriteFormatMode::Standard,
    )
    .expect_err("language switching should fail localization contract");

    assert!(matches!(err, WriteError::StyleGuardViolation(_)));
}

#[test]
fn test_t8_style_guard_blocks_mutation_attempts() {
    let mut synthesis = base_synthesis_packet();
    synthesis["answer_text"] = json!("Direct Answer\nGrounded response for question: What happened.\n\nEvidence\n- Different statement that mutates meaning. [chunk:chunk-a] [url:https://example.com/a]\n- Claim from source B. [chunk:chunk-b] [url:https://example.com/b]\n\nCitations\n- url:https://example.com/a\n- url:https://example.com/b");

    let err = render_write_packet(
        &synthesis,
        1_700_000_600_500i64,
        "trace-run6-t8",
        WriteFormatMode::Standard,
    )
    .expect_err("semantic mutation should fail style guard");

    assert!(matches!(err, WriteError::StyleGuardViolation(_)));
}

#[test]
fn test_append_write_audit_fields_is_replay_stable() {
    let synthesis = base_synthesis_packet();
    let rendered = render_write_packet(
        &synthesis,
        1_700_000_600_500i64,
        "trace-run6-audit",
        WriteFormatMode::Standard,
    )
    .expect("write render should pass");

    let mut audit_packet = json!({
        "schema_version": "1.0.0",
        "produced_by": "PH1.OS",
        "intended_consumers": ["PH1.J"],
        "created_at_ms": 1_700_000_600_800i64,
        "trace_id": "trace-run6-audit",
        "turn_state_transition": "OUTPUT_RENDERED",
        "packet_hashes": {"write": "abc"},
        "evidence_hash": "ev",
        "response_hash": "resp",
        "reason_codes": [],
        "policy_snapshot_id": "pol-1"
    });

    append_write_audit_fields(&mut audit_packet, &rendered.audit_metrics)
        .expect("audit append should pass");

    let write_audit = audit_packet
        .get("turn_state_transition")
        .and_then(Value::as_object)
        .and_then(|obj| obj.get("write_audit"))
        .expect("write_audit should exist");

    assert_eq!(
        write_audit
            .get("citation_count")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        rendered.audit_metrics.citation_count as u64
    );
    assert_eq!(
        write_audit
            .get("style_guard_passed")
            .and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        write_audit
            .get("language_tag")
            .and_then(Value::as_str),
        Some("en")
    );
}

fn lines_after_heading(formatted_text: &str, heading: &str) -> Vec<String> {
    let mut seen = false;
    let mut out = Vec::new();
    for line in formatted_text.lines() {
        let trimmed = line.trim();
        if !seen {
            if trimmed == heading {
                seen = true;
            }
            continue;
        }

        if trimmed.is_empty() {
            continue;
        }

        if trimmed.ends_with(':') && trimmed != heading {
            break;
        }

        out.push(trimmed.to_string());
    }

    out
}
