#![forbid(unsafe_code)]

use crate::web_search_plan::reason_code_validator::validate_reason_code_registry;
use crate::web_search_plan::registry_loader::load_reason_code_registry;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::OnceLock;

static REGISTERED_REASON_CODES: OnceLock<BTreeSet<String>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorTaxonomyMapping {
    pub provider: String,
    pub error_kind: String,
    pub reason_code: String,
}

pub fn map_internal_failure(
    provider: &str,
    error_kind: &str,
    reason_code_override: Option<&str>,
) -> Result<ErrorTaxonomyMapping, String> {
    let canonical_provider = canonicalize_provider(provider)?;
    let canonical_error_kind = canonicalize_error_kind(error_kind)?;

    let reason_code = if let Some(override_code) = reason_code_override {
        let normalized = override_code.trim().to_ascii_lowercase();
        if normalized.is_empty() {
            return Err("reason_code must not be empty".to_string());
        }
        normalized
    } else {
        default_reason_code_for_error_kind(&canonical_error_kind)
            .ok_or_else(|| format!("unknown error_kind taxonomy mapping: {}", canonical_error_kind))?
            .to_string()
    };

    ensure_reason_code_registered(&reason_code)?;

    Ok(ErrorTaxonomyMapping {
        provider: canonical_provider,
        error_kind: canonical_error_kind,
        reason_code,
    })
}

pub fn ensure_reason_code_registered(reason_code: &str) -> Result<(), String> {
    let reason_codes = registered_reason_codes()?;
    if reason_codes.contains(reason_code) {
        Ok(())
    } else {
        Err(format!("reason_code {} is not in REASON_CODES.json", reason_code))
    }
}

fn registered_reason_codes() -> Result<&'static BTreeSet<String>, String> {
    if let Some(existing) = REGISTERED_REASON_CODES.get() {
        return Ok(existing);
    }

    let registry = load_reason_code_registry()?;
    validate_reason_code_registry(&registry)?;
    let mapped: BTreeSet<String> = registry
        .reason_codes
        .iter()
        .map(|entry| entry.code_id.trim().to_ascii_lowercase())
        .collect();

    let _ = REGISTERED_REASON_CODES.set(mapped);
    REGISTERED_REASON_CODES
        .get()
        .ok_or_else(|| "failed to initialize reason code registry cache".to_string())
}

fn canonicalize_provider(provider: &str) -> Result<String, String> {
    match provider.trim().to_ascii_lowercase().as_str() {
        "proxy" => Ok("Proxy".to_string()),
        "urlfetch" | "url_fetch" | "url-fetch" => Ok("UrlFetch".to_string()),
        "bravewebsearch" | "brave_web_search" => Ok("BraveWebSearch".to_string()),
        "openai_websearch" | "openai_web_search" => Ok("OpenAI_WebSearch".to_string()),
        "gdelt" => Ok("GDELT".to_string()),
        "chunkhash" | "chunk_hash" => Ok("ChunkHash".to_string()),
        "planning" => Ok("Planning".to_string()),
        "synthesis" => Ok("Synthesis".to_string()),
        "write" => Ok("Write".to_string()),
        other => Err(format!("unsupported debug provider {}", other)),
    }
}

fn canonicalize_error_kind(error_kind: &str) -> Result<String, String> {
    let normalized = error_kind.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Err("error_kind must not be empty".to_string());
    }

    if default_reason_code_for_error_kind(&normalized).is_some() {
        return Ok(normalized);
    }

    Err(format!(
        "unknown error_kind taxonomy mapping: {}",
        normalized
    ))
}

fn default_reason_code_for_error_kind(error_kind: &str) -> Option<&'static str> {
    match error_kind {
        "provider_unconfigured" => Some("provider_unconfigured"),
        "timeout_exceeded" | "proxy_timeout" => Some("timeout_exceeded"),
        "proxy_misconfigured"
        | "proxy_auth_failed"
        | "proxy_connect_failed"
        | "proxy_tls_failed"
        | "proxy_dns_failed" => Some("proxy_misconfigured"),
        "empty_results" | "extraction_quality_low" | "empty_extraction" => Some("empty_results"),
        "insufficient_evidence" => Some("insufficient_evidence"),
        "citation_mismatch" => Some("citation_mismatch"),
        "unsupported_claim" => Some("unsupported_claim"),
        "budget_exhausted" => Some("budget_exhausted"),
        "policy_violation" => Some("policy_violation"),
        "hash_collision_detected" => Some("hash_collision_detected"),
        "invalid_session" => Some("invalid_session"),
        "input_unparseable" => Some("input_unparseable"),
        "dns_failed"
        | "tls_failed"
        | "connect_failed"
        | "health_cooldown"
        | "http_non_200"
        | "parse_failed"
        | "transport_failed"
        | "unsupported_scheme"
        | "invalid_url"
        | "redirect_loop_detected"
        | "redirect_depth_exceeded"
        | "redirect_scheme_downgrade_blocked"
        | "redirect_missing_location"
        | "mime_not_allowed"
        | "mime_ambiguous"
        | "unsupported_content_encoding"
        | "response_bytes_exceeded"
        | "decompressed_bytes_exceeded"
        | "extraction_chars_exceeded"
        | "decompression_failed"
        | "charset_decode_failed" => Some("provider_upstream_failed"),
        _ => None,
    }
}
