#![forbid(unsafe_code)]

use crate::web_search_plan::proxy::ProxyMode;
use crate::web_search_plan::structured::adapters::company_registry;
use crate::web_search_plan::structured::execute_structured_from_tool_request;
use crate::web_search_plan::structured::normalize::{
    row_ordering_key, sort_rows_deterministically,
};
use crate::web_search_plan::structured::registry::{route_adapter, AdapterSelection, DomainHint};
use crate::web_search_plan::structured::types::{
    StructuredAdapterRequest, StructuredErrorKind, StructuredExtraction, StructuredRow,
    StructuredRuntimeConfig, StructuredValue, STRUCTURED_SCHEMA_VERSION,
};
use crate::web_search_plan::structured::validator::{validate_extraction, validate_row};
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::TcpListener;

fn sample_row(
    entity: &str,
    attribute: &str,
    value: StructuredValue,
    source_url: &str,
) -> StructuredRow {
    StructuredRow {
        entity: entity.to_string(),
        attribute: attribute.to_string(),
        value,
        unit: None,
        as_of_ms: None,
        source_url: source_url.to_string(),
        source_ref: source_url.to_string(),
        confidence: None,
        schema_version: STRUCTURED_SCHEMA_VERSION.to_string(),
    }
}

fn sample_request(query: &str) -> StructuredAdapterRequest {
    StructuredAdapterRequest {
        trace_id: "trace-1".to_string(),
        query: query.to_string(),
        created_at_ms: 1_700_000_000_000,
        now_ms: 1_700_000_000_001,
        intended_consumers: vec!["PH1.D".to_string()],
        importance_tier: "medium".to_string(),
        domain_hint: None,
        budgets: json!({}),
        proxy_config: crate::web_search_plan::proxy::proxy_config::ProxyConfig {
            mode: ProxyMode::Off,
            http_proxy_url: None,
            https_proxy_url: None,
        },
    }
}

#[test]
fn test_t1_schema_validation_fail_closed_missing_required_field() {
    let extraction = StructuredExtraction {
        query: "school query".to_string(),
        schema_id: "gov_dataset_v1".to_string(),
        extracted_at_ms: 1_700_000_000_000,
        provider_runs: vec![],
        sources: vec![],
        errors: vec![],
        rows: vec![sample_row(
            "School A",
            "dataset_id",
            StructuredValue::Int { value: 42 },
            "https://example.com",
        )],
    };

    let result = validate_extraction(&extraction);
    assert!(result.is_err());
    assert!(result
        .err()
        .unwrap_or_default()
        .contains("missing required attribute value"));
}

#[test]
fn test_t2_typed_parsing_correctness_currency_percent_date() {
    let currency_row = sample_row(
        "Acme",
        "price",
        StructuredValue::Currency {
            amount: 19.99,
            currency_code: "USD".to_string(),
        },
        "https://example.com/price",
    );
    let percent_row = sample_row(
        "Acme",
        "margin",
        StructuredValue::Percent { value: 25.0 },
        "https://example.com/price",
    );
    let date_row = sample_row(
        "Acme",
        "as_of",
        StructuredValue::Date {
            value: "2025-01-01".to_string(),
        },
        "https://example.com/price",
    );

    assert!(validate_row(&currency_row).is_ok());
    assert!(validate_row(&percent_row).is_ok());
    assert!(validate_row(&date_row).is_ok());

    let invalid_percent = sample_row(
        "Acme",
        "margin",
        StructuredValue::Percent { value: 125.0 },
        "https://example.com/price",
    );
    assert!(validate_row(&invalid_percent).is_err());
}

#[test]
fn test_t3_deterministic_row_ordering() {
    let mut rows = vec![
        sample_row(
            "BEntity",
            "beta",
            StructuredValue::String {
                value: "x".to_string(),
            },
            "https://b.example.com",
        ),
        sample_row(
            "AEntity",
            "alpha",
            StructuredValue::String {
                value: "x".to_string(),
            },
            "https://a.example.com",
        ),
        sample_row(
            "AEntity",
            "alpha",
            StructuredValue::String {
                value: "a".to_string(),
            },
            "https://a.example.com",
        ),
    ];

    sort_rows_deterministically(&mut rows);
    let keys = rows.iter().map(row_ordering_key).collect::<Vec<_>>();
    assert_eq!(keys[0].0, "aentity");
    assert_eq!(keys[0].3, "string:a");
    assert_eq!(keys[1].0, "aentity");
    assert_eq!(keys[1].3, "string:x");
    assert_eq!(keys[2].0, "bentity");
}

#[test]
fn test_t4_adapter_registry_routing_deterministic() {
    assert_eq!(
        route_adapter("anything", Some(DomainHint::Patents)).expect("patents routing"),
        AdapterSelection::Patents
    );
    assert_eq!(
        route_adapter("https://data.example.com/payload.json", None).expect("url routing"),
        AdapterSelection::GenericHttpJson
    );

    let err = route_adapter("acme financials", None).expect_err("non-url should fail");
    assert!(err.contains("requires domain_hint or explicit URL"));
}

#[test]
fn test_t5_provider_unconfigured_deterministic_when_creds_absent() {
    let request = sample_request("query");
    let config = StructuredRuntimeConfig::default();

    let first = company_registry::execute(&request, &config).expect_err("must fail closed");
    let second = company_registry::execute(&request, &config).expect_err("must fail closed");
    assert_eq!(first.kind, StructuredErrorKind::ProviderUnconfigured);
    assert_eq!(second.kind, StructuredErrorKind::ProviderUnconfigured);
    assert_eq!(first.reason_code(), "provider_unconfigured");
    assert_eq!(first.message, second.message);
}

#[test]
fn test_t6_fixture_end_to_end_parsing_validation_path() {
    let payload = r#"{
      "company": {
        "name": "Acme",
        "revenue": "USD 123.45",
        "growth": "12%",
        "as_of": "2025-01-01"
      }
    }"#;

    let (url, handle) = spawn_json_server(payload, 2);
    let tool_request = json!({
        "schema_version": "1.0.0",
        "produced_by": "PH1.X",
        "intended_consumers": ["PH1.E"],
        "created_at_ms": 1_700_000_000_000i64,
        "trace_id": "trace-structured",
        "mode": "structured",
        "query": url,
        "importance_tier": "medium",
        "budgets": {}
    });
    let config = StructuredRuntimeConfig::default();

    let run_a = execute_structured_from_tool_request(&tool_request, 1_700_000_000_111, &config)
        .expect("structured run a should pass");
    let run_b = execute_structured_from_tool_request(&tool_request, 1_700_000_000_111, &config)
        .expect("structured run b should pass");
    handle.join().expect("server thread should join");

    assert_eq!(run_a.extraction.rows, run_b.extraction.rows);
    assert_eq!(
        run_a
            .evidence_packet
            .pointer("/trust_metadata/structured/schema_id")
            .and_then(Value::as_str),
        Some("generic_http_json_v1")
    );
    assert!(!run_a
        .evidence_packet
        .pointer("/trust_metadata/structured/rows")
        .and_then(Value::as_array)
        .expect("rows array expected")
        .is_empty());
}

#[test]
fn test_t6_live_gov_dataset_adapter_optional_e2e() {
    if std::env::var("SELENE_E2E").ok().as_deref() != Some("1") {
        return;
    }

    let api_key = std::env::var("SELENE_GOV_DATASET_API_KEY").ok();
    if api_key.as_deref().unwrap_or("").trim().is_empty() {
        return;
    }

    let tool_request = json!({
        "schema_version": "1.0.0",
        "produced_by": "PH1.X",
        "intended_consumers": ["PH1.E"],
        "created_at_ms": 1_700_000_000_000i64,
        "trace_id": "trace-structured-e2e",
        "mode": "structured",
        "query": "Stanford",
        "importance_tier": "medium",
        "budgets": {
            "domain_hint": "gov_dataset"
        }
    });

    let mut config = StructuredRuntimeConfig::default();
    config.gov_dataset_api_key_override = api_key;

    match execute_structured_from_tool_request(&tool_request, 1_700_000_000_222, &config) {
        Ok(result) => {
            assert!(!result.extraction.rows.is_empty());
        }
        Err(error) => {
            assert!(
                matches!(
                    error.kind,
                    StructuredErrorKind::ProviderUpstreamFailed
                        | StructuredErrorKind::TimeoutExceeded
                        | StructuredErrorKind::EmptyResults
                ),
                "unexpected live error {:?}",
                error
            );
        }
    }
}

fn spawn_json_server(payload: &str, requests: usize) -> (String, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let address = listener.local_addr().expect("server addr");
    let payload = payload.to_string();

    let handle = std::thread::spawn(move || {
        for _ in 0..requests {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buffer = [0u8; 4096];
                let _ = stream.read(&mut buffer);
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    payload.len(),
                    payload
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        }
    });

    (format!("http://{}", address), handle)
}
