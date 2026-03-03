#![forbid(unsafe_code)]

pub mod citation_renderer;
pub mod formatter;
pub mod localization;
pub mod style_guard;
pub mod voice_renderer;

use crate::web_search_plan::write::formatter::format_synthesis_packet;
use crate::web_search_plan::write::voice_renderer::render_voice_output;
use crate::web_search_plan::diag::{
    default_failed_transitions, try_build_debug_packet, DebugPacketContext, DebugStatus,
};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

pub const WRITE_ENGINE_ID: &str = "PH1.WRITE";
pub const WRITE_SCHEMA_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteFormatMode {
    Brief,
    Standard,
    Deep,
}

impl WriteFormatMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Brief => "brief",
            Self::Standard => "standard",
            Self::Deep => "deep",
        }
    }

    pub const fn max_direct_answer_sentences(self) -> usize {
        match self {
            Self::Brief => 2,
            Self::Standard => 3,
            Self::Deep => 4,
        }
    }
}

impl Default for WriteFormatMode {
    fn default() -> Self {
        Self::Standard
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteError {
    InvalidSynthesis(String),
    CitationMismatch(String),
    UnsupportedClaim(String),
    StyleGuardViolation(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteAuditMetrics {
    pub formatted_text_hash: String,
    pub citation_count: usize,
    pub bullet_count: usize,
    pub format_mode: String,
    pub language_tag: String,
    pub style_guard_passed: bool,
}

#[derive(Debug, Clone)]
pub struct WriteRenderResult {
    pub write_packet: Value,
    pub voice_text: String,
    pub audit_metrics: WriteAuditMetrics,
}

pub fn render_write_packet(
    synthesis_packet: &Value,
    created_at_ms: i64,
    trace_id: &str,
    format_mode: WriteFormatMode,
) -> Result<WriteRenderResult, WriteError> {
    let formatted = format_synthesis_packet(synthesis_packet, format_mode).map_err(|err| {
        emit_write_debug_packet(trace_id, created_at_ms, &err);
        err
    })?;

    let write_packet = json!({
        "schema_version": WRITE_SCHEMA_VERSION,
        "produced_by": WRITE_ENGINE_ID,
        "intended_consumers": ["API", "PH1.TTS", "PH1.J"],
        "created_at_ms": created_at_ms,
        "trace_id": trace_id,
        "formatted_text": formatted.formatted_text,
        "format_mode": format_mode.as_str(),
        "voice_safe": true,
        "citation_map": Value::Object(formatted.citation_map.clone()),
    });

    let voice_text = render_voice_output(
        write_packet
            .get("formatted_text")
            .and_then(Value::as_str)
            .unwrap_or_default(),
    );

    let formatted_text = write_packet
        .get("formatted_text")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let audit_metrics = WriteAuditMetrics {
        formatted_text_hash: sha256_hex(formatted_text),
        citation_count: formatted.citation_count,
        bullet_count: formatted.bullet_count,
        format_mode: format_mode.as_str().to_string(),
        language_tag: formatted.language_tag,
        style_guard_passed: formatted.style_guard_passed,
    };

    Ok(WriteRenderResult {
        write_packet,
        voice_text,
        audit_metrics,
    })
}

pub fn append_write_audit_fields(
    audit_packet: &mut Value,
    metrics: &WriteAuditMetrics,
) -> Result<(), String> {
    let obj = audit_packet
        .as_object_mut()
        .ok_or_else(|| "audit packet must be object".to_string())?;

    let transition_value = obj
        .entry("turn_state_transition".to_string())
        .or_insert_with(|| Value::Object(Map::new()));

    let transition_obj = if transition_value.is_object() {
        transition_value
            .as_object_mut()
            .ok_or_else(|| "turn_state_transition must be object".to_string())?
    } else if let Some(state) = transition_value.as_str() {
        *transition_value = json!({"state": state});
        transition_value
            .as_object_mut()
            .ok_or_else(|| "turn_state_transition conversion failed".to_string())?
    } else {
        return Err("turn_state_transition must be string or object".to_string());
    };

    transition_obj.insert(
        "write_audit".to_string(),
        json!({
            "formatted_text_hash": metrics.formatted_text_hash,
            "citation_count": metrics.citation_count,
            "bullet_count": metrics.bullet_count,
            "format_mode": metrics.format_mode,
            "language_tag": metrics.language_tag,
            "style_guard_passed": metrics.style_guard_passed,
        }),
    );

    Ok(())
}

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn emit_write_debug_packet(trace_id: &str, created_at_ms: i64, error: &WriteError) {
    let (error_kind, reason_code, hint) = match error {
        WriteError::InvalidSynthesis(message) => {
            ("input_unparseable", "input_unparseable", message.as_str())
        }
        WriteError::CitationMismatch(message) => {
            ("citation_mismatch", "citation_mismatch", message.as_str())
        }
        WriteError::UnsupportedClaim(message) => {
            ("unsupported_claim", "unsupported_claim", message.as_str())
        }
        WriteError::StyleGuardViolation(message) => {
            ("policy_violation", "policy_violation", message.as_str())
        }
    };

    let transitions = default_failed_transitions(created_at_ms);
    let _ = try_build_debug_packet(DebugPacketContext {
        trace_id,
        status: DebugStatus::Failed,
        provider: "Write",
        error_kind,
        reason_code,
        proxy_mode: None,
        source_url: None,
        created_at_ms,
        turn_state_transitions: &transitions,
        debug_hint: Some(hint),
        fallback_used: None,
        health_status_before_fallback: None,
    });
}

#[cfg(test)]
pub mod write_tests;
