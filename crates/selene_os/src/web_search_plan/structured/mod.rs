#![forbid(unsafe_code)]

pub mod adapters;
pub mod normalize;
pub mod registry;
pub mod schema;
pub mod types;
pub mod validator;

use crate::web_search_plan::proxy::proxy_config::{ProxyConfig, SystemEnvProvider};
use crate::web_search_plan::proxy::ProxyMode;
use crate::web_search_plan::structured::normalize::{normalize_row, sort_rows_deterministically};
use crate::web_search_plan::structured::registry::{extract_domain_hint, route_adapter};
use crate::web_search_plan::structured::types::{
    StructuredAdapterRequest, StructuredConnectorError, StructuredErrorKind, StructuredExtraction,
    StructuredRuntimeConfig, STRUCTURED_ENGINE_ID,
};
use crate::web_search_plan::structured::validator::validate_extraction;
use serde_json::{json, Map, Value};

#[derive(Debug, Clone)]
pub struct StructuredConnectorResult {
    pub extraction: StructuredExtraction,
    pub evidence_packet: Value,
}

pub fn execute_structured_from_tool_request(
    tool_request_packet: &Value,
    now_ms: i64,
    config: &StructuredRuntimeConfig,
) -> Result<StructuredConnectorResult, StructuredConnectorError> {
    let request = parse_tool_request_packet(tool_request_packet, now_ms)?;
    let selection =
        route_adapter(request.query.as_str(), request.domain_hint).map_err(|message| {
            StructuredConnectorError::new(
                "structured_registry",
                StructuredErrorKind::InsufficientEvidence,
                None,
                message,
                0,
            )
        })?;

    let output = adapters::run_adapter(selection, &request, config)?;

    let mut rows = output
        .rows
        .into_iter()
        .map(normalize_row)
        .collect::<Vec<_>>();
    sort_rows_deterministically(&mut rows);

    let extraction = StructuredExtraction {
        query: request.query.clone(),
        rows,
        schema_id: output.schema_id,
        extracted_at_ms: now_ms,
        provider_runs: output.provider_runs,
        sources: output.sources,
        errors: output.errors,
    };

    validate_extraction(&extraction).map_err(|error| {
        StructuredConnectorError::new(
            selection.as_str(),
            StructuredErrorKind::PolicyViolation,
            None,
            format!("structured schema validation failed: {}", error),
            0,
        )
    })?;

    let structured_rows_json = serde_json::to_value(&extraction.rows).map_err(|error| {
        StructuredConnectorError::new(
            selection.as_str(),
            StructuredErrorKind::PolicyViolation,
            None,
            format!("failed serializing structured rows: {}", error),
            0,
        )
    })?;

    let evidence_packet = json!({
        "schema_version": "1.0.0",
        "produced_by": STRUCTURED_ENGINE_ID,
        "intended_consumers": request.intended_consumers,
        "created_at_ms": request.created_at_ms,
        "trace_id": request.trace_id,
        "query": request.query,
        "retrieved_at_ms": now_ms,
        "provider_runs": extraction.provider_runs,
        "sources": extraction.sources,
        "content_chunks": [],
        "trust_metadata": {
            "structured": {
                "adapter_id": selection.as_str(),
                "schema_id": extraction.schema_id,
                "rows": structured_rows_json,
                "errors": extraction.errors,
                "domain_hint": request.domain_hint.map(|hint| hint.as_str()),
            }
        }
    });

    Ok(StructuredConnectorResult {
        extraction,
        evidence_packet,
    })
}

pub fn append_structured_audit_fields(
    audit_packet: &mut Value,
    extraction: &StructuredExtraction,
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
        *transition_value = json!({ "state": state });
        transition_value
            .as_object_mut()
            .ok_or_else(|| "turn_state_transition conversion failed".to_string())?
    } else {
        return Err("turn_state_transition must be string or object".to_string());
    };

    transition_obj.insert(
        "structured_audit".to_string(),
        json!({
            "schema_id": extraction.schema_id,
            "row_count": extraction.rows.len(),
            "error_count": extraction.errors.len(),
        }),
    );
    Ok(())
}

fn parse_tool_request_packet(
    tool_request_packet: &Value,
    now_ms: i64,
) -> Result<StructuredAdapterRequest, StructuredConnectorError> {
    let obj = tool_request_packet.as_object().ok_or_else(|| {
        StructuredConnectorError::new(
            "structured_parser",
            StructuredErrorKind::ParseFailed,
            None,
            "tool request packet must be object",
            0,
        )
    })?;

    let mode = obj.get("mode").and_then(Value::as_str).unwrap_or_default();
    if mode != "structured" {
        return Err(StructuredConnectorError::new(
            "structured_parser",
            StructuredErrorKind::ParseFailed,
            None,
            format!("tool request mode must be structured, got {}", mode),
            0,
        ));
    }

    let query = obj
        .get("query")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            StructuredConnectorError::new(
                "structured_parser",
                StructuredErrorKind::ParseFailed,
                None,
                "tool request query missing",
                0,
            )
        })?
        .to_string();

    let trace_id = obj
        .get("trace_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            StructuredConnectorError::new(
                "structured_parser",
                StructuredErrorKind::ParseFailed,
                None,
                "tool request trace_id missing",
                0,
            )
        })?
        .to_string();

    let created_at_ms = obj
        .get("created_at_ms")
        .and_then(Value::as_i64)
        .unwrap_or(now_ms);

    let intended_consumers = obj
        .get("intended_consumers")
        .and_then(Value::as_array)
        .map(|array| {
            array
                .iter()
                .filter_map(Value::as_str)
                .map(|entry| entry.trim().to_string())
                .filter(|entry| !entry.is_empty())
                .collect::<Vec<String>>()
        })
        .filter(|array| !array.is_empty())
        .unwrap_or_else(|| {
            vec![
                "PH1.D".to_string(),
                "PH1.WRITE".to_string(),
                "PH1.J".to_string(),
            ]
        });

    let importance_tier = obj
        .get("importance_tier")
        .and_then(Value::as_str)
        .unwrap_or("medium")
        .to_string();

    let budgets = obj.get("budgets").cloned().unwrap_or_else(|| json!({}));
    let domain_hint = extract_domain_hint(tool_request_packet);

    let env = SystemEnvProvider;
    let proxy_mode_raw =
        std::env::var("SELENE_STRUCTURED_PROXY_MODE").unwrap_or_else(|_| "off".to_string());
    let proxy_mode = ProxyMode::parse(&proxy_mode_raw).unwrap_or(ProxyMode::Off);
    let proxy_config = ProxyConfig::from_env(proxy_mode, &env);

    Ok(StructuredAdapterRequest {
        trace_id,
        query,
        created_at_ms,
        now_ms,
        intended_consumers,
        importance_tier,
        domain_hint,
        budgets,
        proxy_config,
    })
}

#[cfg(test)]
pub mod structured_tests;
