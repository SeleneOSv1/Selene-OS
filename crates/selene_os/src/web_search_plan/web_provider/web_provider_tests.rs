#![forbid(unsafe_code)]

use crate::web_search_plan::packet_validator::{validate_packet, validate_packet_schema_registry};
use crate::web_search_plan::proxy::proxy_config::ProxyConfig;
use crate::web_search_plan::proxy::ProxyMode;
use crate::web_search_plan::registry_loader::load_packet_schema_registry;
use crate::web_search_plan::web_provider::health_state::{HealthPolicy, ProviderHealthTracker};
use crate::web_search_plan::web_provider::provider_merge::merge_results;
use crate::web_search_plan::web_provider::{
    append_web_provider_audit_fields, execute_web_provider_ladder_from_tool_request,
    NormalizedSearchResult, ProviderErrorKind, ProviderId, WebProviderRuntimeConfig,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct MockResponse {
    status: u16,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}

impl MockResponse {
    fn json(status: u16, body: Value) -> Self {
        Self {
            status,
            headers: BTreeMap::from([("Content-Type".to_string(), "application/json".to_string())]),
            body: serde_json::to_vec(&body).expect("json serialization should succeed"),
        }
    }
}

fn spawn_server<F>(handler: F, max_requests: usize) -> (String, thread::JoinHandle<()>)
where
    F: Fn(&str, &str) -> MockResponse + Send + Sync + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind should succeed");
    listener
        .set_nonblocking(true)
        .expect("nonblocking should succeed");

    let base = format!(
        "http://{}",
        listener.local_addr().expect("local addr should exist")
    );
    let handler = Arc::new(handler);

    let join = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut served = 0usize;
        while served < max_requests && Instant::now() < deadline {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let (method, path) = read_request_head(&mut stream);
                    let response = handler(&method, &path);
                    write_http_response(&mut stream, &response);
                    served = served.saturating_add(1);
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(_) => break,
            }
        }
    });

    (base, join)
}

fn read_request_head(stream: &mut TcpStream) -> (String, String) {
    let mut reader = BufReader::new(stream);
    let mut first_line = String::new();
    let _ = reader.read_line(&mut first_line);

    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).is_err() {
            break;
        }
        if line == "\r\n" || line.is_empty() {
            break;
        }
    }

    let mut parts = first_line.split_whitespace();
    let method = parts.next().unwrap_or("GET").to_string();
    let path = parts.next().unwrap_or("/").to_string();
    (method, path)
}

fn write_http_response(stream: &mut TcpStream, response: &MockResponse) {
    let status_text = match response.status {
        200 => "OK",
        500 => "Internal Server Error",
        _ => "Status",
    };

    let mut head = format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nConnection: close\r\n",
        response.status,
        status_text,
        response.body.len()
    );
    for (key, value) in &response.headers {
        head.push_str(&format!("{}: {}\r\n", key, value));
    }
    head.push_str("\r\n");

    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(&response.body);
    let _ = stream.flush();
}

fn tool_request_packet(query: &str) -> Value {
    json!({
        "schema_version": "1.0.0",
        "produced_by": "PH1.X",
        "intended_consumers": ["PH1.E"],
        "created_at_ms": 1_700_000_700_000i64,
        "trace_id": "trace-run7",
        "mode": "web",
        "query": query,
        "importance_tier": "medium",
        "budgets": {}
    })
}

fn base_runtime(brave_endpoint: &str, openai_endpoint: &str) -> WebProviderRuntimeConfig {
    WebProviderRuntimeConfig {
        brave_endpoint: brave_endpoint.to_string(),
        openai_endpoint: openai_endpoint.to_string(),
        openai_model: "gpt-4o-mini".to_string(),
        max_results: 3,
        timeout_ms: 1_500,
        user_agent: "selene-web-provider-ladder/test".to_string(),
        proxy_config: ProxyConfig {
            mode: ProxyMode::Off,
            http_proxy_url: None,
            https_proxy_url: None,
        },
        health_policy: HealthPolicy::default(),
        brave_api_key_override: Some("test_brave_key".to_string()),
        openai_api_key_override: Some("test_openai_key".to_string()),
    }
}

fn brave_success_payload(items: &[(&str, &str, &str)]) -> Value {
    json!({
        "web": {
            "results": items
                .iter()
                .map(|(title, url, snippet)| json!({
                    "title": title,
                    "url": url,
                    "description": snippet,
                }))
                .collect::<Vec<Value>>()
        }
    })
}

fn openai_success_payload(items: &[(&str, &str, &str)], citation_urls: &[&str]) -> Value {
    let results = items
        .iter()
        .map(|(title, url, snippet)| {
            json!({
                "title": title,
                "url": url,
                "snippet": snippet,
            })
        })
        .collect::<Vec<Value>>();

    let output_text = json!({"results": results}).to_string();
    json!({
        "output": [
            {
                "content": [
                    {
                        "text": output_text,
                        "annotations": citation_urls
                            .iter()
                            .map(|url| json!({"type": "url_citation", "url": url}))
                            .collect::<Vec<Value>>()
                    }
                ]
            }
        ]
    })
}

#[test]
fn test_t1_brave_success_no_fallback() {
    let (base, join) = spawn_server(
        move |_, path| {
            if path.starts_with("/res/v1/web/search") {
                MockResponse::json(
                    200,
                    brave_success_payload(&[
                        ("A", "https://example.com/a", "Snippet A."),
                        ("B", "https://example.com/b", "Snippet B."),
                    ]),
                )
            } else {
                MockResponse::json(500, json!({"error": "unexpected path"}))
            }
        },
        2,
    );

    let config = base_runtime(
        &format!("{}/res/v1/web/search", base),
        &format!("{}/v1/responses", base),
    );

    let mut health = ProviderHealthTracker::default();
    let result = execute_web_provider_ladder_from_tool_request(
        &tool_request_packet("selene web query"),
        1_700_000_700_111i64,
        &mut health,
        &config,
    )
    .expect("brave success path should pass");

    assert!(!result.audit_metrics.fallback_used);
    assert!(result.audit_metrics.lead_attempted);
    assert_eq!(result.audit_metrics.results_count_lead, 2);
    assert_eq!(result.audit_metrics.results_count_fallback, 0);

    let provider_runs = result
        .evidence_packet
        .get("provider_runs")
        .and_then(Value::as_array)
        .expect("provider_runs must exist");
    assert_eq!(provider_runs.len(), 1);
    assert_eq!(
        provider_runs[0].get("provider_id").and_then(Value::as_str),
        Some("brave_web_search")
    );

    let _ = join.join();
}

#[test]
fn test_t2_brave_transport_failure_triggers_fallback() {
    let (openai_base, join) = spawn_server(
        move |method, path| {
            if method == "POST" && path.starts_with("/v1/responses") {
                MockResponse::json(
                    200,
                    openai_success_payload(
                        &[("OA", "https://fallback.example.com/1", "Fallback snippet")],
                        &["https://fallback.example.com/1"],
                    ),
                )
            } else {
                MockResponse::json(500, json!({"error": "unexpected"}))
            }
        },
        2,
    );

    let config = base_runtime(
        "http://127.0.0.1:9/res/v1/web/search",
        &format!("{}/v1/responses", openai_base),
    );

    let mut health = ProviderHealthTracker::default();
    let result = execute_web_provider_ladder_from_tool_request(
        &tool_request_packet("selene web query"),
        1_700_000_700_222i64,
        &mut health,
        &config,
    )
    .expect("fallback path should pass");

    assert!(result.audit_metrics.fallback_used);
    assert_eq!(
        result.audit_metrics.fallback_reason_code.as_deref(),
        Some("provider_upstream_failed")
    );

    let provider_runs = result
        .evidence_packet
        .get("provider_runs")
        .and_then(Value::as_array)
        .expect("provider_runs must exist");
    assert_eq!(provider_runs.len(), 2);
    assert_eq!(
        provider_runs[0].get("provider_id").and_then(Value::as_str),
        Some("brave_web_search")
    );
    assert_eq!(
        provider_runs[1].get("provider_id").and_then(Value::as_str),
        Some("openai_web_search")
    );

    let _ = join.join();
}

#[test]
fn test_t3_brave_empty_results_triggers_fallback() {
    let (base, join) = spawn_server(
        move |method, path| {
            if method == "GET" && path.starts_with("/res/v1/web/search") {
                MockResponse::json(200, json!({"web": {"results": []}}))
            } else if method == "POST" && path.starts_with("/v1/responses") {
                MockResponse::json(
                    200,
                    openai_success_payload(
                        &[("OA", "https://fallback.example.com/2", "Fallback snippet")],
                        &["https://fallback.example.com/2"],
                    ),
                )
            } else {
                MockResponse::json(500, json!({"error": "unexpected"}))
            }
        },
        4,
    );

    let config = base_runtime(
        &format!("{}/res/v1/web/search", base),
        &format!("{}/v1/responses", base),
    );

    let mut health = ProviderHealthTracker::default();
    let result = execute_web_provider_ladder_from_tool_request(
        &tool_request_packet("selene web query"),
        1_700_000_700_333i64,
        &mut health,
        &config,
    )
    .expect("empty results fallback should pass");

    assert!(result.audit_metrics.fallback_used);
    assert_eq!(result.audit_metrics.results_count_lead, 0);
    assert_eq!(result.audit_metrics.results_count_fallback, 1);

    let _ = join.join();
}

#[test]
fn test_t4_health_cooldown_skips_brave_deterministically() {
    let (openai_base, join) = spawn_server(
        move |method, path| {
            if method == "POST" && path.starts_with("/v1/responses") {
                MockResponse::json(
                    200,
                    openai_success_payload(
                        &[("OA", "https://fallback.example.com/3", "Fallback snippet")],
                        &["https://fallback.example.com/3"],
                    ),
                )
            } else {
                MockResponse::json(500, json!({"error": "unexpected"}))
            }
        },
        6,
    );

    let mut config = base_runtime(
        "http://127.0.0.1:9/res/v1/web/search",
        &format!("{}/v1/responses", openai_base),
    );
    config.health_policy = HealthPolicy {
        failures_before_cooldown: 1,
        cooldown_ms: 60_000,
    };

    let mut health = ProviderHealthTracker::default();
    let first = execute_web_provider_ladder_from_tool_request(
        &tool_request_packet("selene web query"),
        1_700_000_700_444i64,
        &mut health,
        &config,
    )
    .expect("first call should fallback and set cooldown");
    assert!(first.audit_metrics.lead_attempted);

    let second = execute_web_provider_ladder_from_tool_request(
        &tool_request_packet("selene web query"),
        1_700_000_700_445i64,
        &mut health,
        &config,
    )
    .expect("second call should skip brave due cooldown");

    assert!(!second.audit_metrics.lead_attempted);
    assert!(second.audit_metrics.fallback_used);

    let provider_runs = second
        .evidence_packet
        .get("provider_runs")
        .and_then(Value::as_array)
        .expect("provider_runs must exist");
    assert_eq!(
        provider_runs[0]
            .get("skipped_due_to_cooldown")
            .and_then(Value::as_bool),
        Some(true)
    );

    let _ = join.join();
}

#[test]
fn test_t5_dedup_preserves_brave_precedence() {
    let brave = vec![
        NormalizedSearchResult {
            title: "A".to_string(),
            url: "https://example.com/a".to_string(),
            snippet: "Snippet A".to_string(),
            canonical_url: "https://example.com/a".to_string(),
            citation_url: "https://example.com/a".to_string(),
            provider_id: ProviderId::BraveWebSearch,
            provider_rank: 1,
        },
        NormalizedSearchResult {
            title: "B".to_string(),
            url: "https://example.com/b".to_string(),
            snippet: "Snippet B".to_string(),
            canonical_url: "https://example.com/b".to_string(),
            citation_url: "https://example.com/b".to_string(),
            provider_id: ProviderId::BraveWebSearch,
            provider_rank: 2,
        },
    ];
    let openai = vec![
        NormalizedSearchResult {
            title: "B-openai".to_string(),
            url: "https://example.com/b".to_string(),
            snippet: "Snippet B duplicate".to_string(),
            canonical_url: "https://example.com/b".to_string(),
            citation_url: "https://example.com/b".to_string(),
            provider_id: ProviderId::OpenAiWebSearch,
            provider_rank: 1,
        },
        NormalizedSearchResult {
            title: "C".to_string(),
            url: "https://example.com/c".to_string(),
            snippet: "Snippet C".to_string(),
            canonical_url: "https://example.com/c".to_string(),
            citation_url: "https://example.com/c".to_string(),
            provider_id: ProviderId::OpenAiWebSearch,
            provider_rank: 2,
        },
    ];

    let merged = merge_results(&brave, &openai);
    assert_eq!(merged.dedup_count, 1);
    let urls: Vec<String> = merged
        .merged_results
        .iter()
        .map(|result| result.url.clone())
        .collect();
    assert_eq!(
        urls,
        vec![
            "https://example.com/a".to_string(),
            "https://example.com/b".to_string(),
            "https://example.com/c".to_string(),
        ]
    );
    assert_eq!(
        merged.merged_results[1].provider_id,
        ProviderId::BraveWebSearch
    );
}

#[test]
fn test_t6_merge_order_stable_across_identical_runs() {
    let (base, join) = spawn_server(
        move |method, path| {
            if method == "GET" && path.starts_with("/res/v1/web/search") {
                MockResponse::json(
                    200,
                    brave_success_payload(&[
                        ("A", "https://stable.example.com/a", "Stable A"),
                        ("B", "https://stable.example.com/b", "Stable B"),
                    ]),
                )
            } else {
                MockResponse::json(500, json!({"error": "unexpected"}))
            }
        },
        4,
    );

    let config = base_runtime(
        &format!("{}/res/v1/web/search", base),
        &format!("{}/v1/responses", base),
    );

    let mut health = ProviderHealthTracker::default();
    let first = execute_web_provider_ladder_from_tool_request(
        &tool_request_packet("stable query"),
        1_700_000_700_666i64,
        &mut health,
        &config,
    )
    .expect("first run should pass");
    let second = execute_web_provider_ladder_from_tool_request(
        &tool_request_packet("stable query"),
        1_700_000_700_666i64,
        &mut health,
        &config,
    )
    .expect("second run should pass");

    let first_sources = first
        .evidence_packet
        .get("sources")
        .and_then(Value::as_array)
        .expect("first sources must exist");
    let second_sources = second
        .evidence_packet
        .get("sources")
        .and_then(Value::as_array)
        .expect("second sources must exist");
    assert_eq!(first_sources, second_sources);

    let first_chunks = first
        .evidence_packet
        .get("content_chunks")
        .and_then(Value::as_array)
        .expect("first content_chunks must exist");
    let second_chunks = second
        .evidence_packet
        .get("content_chunks")
        .and_then(Value::as_array)
        .expect("second content_chunks must exist");
    let first_chunk_ids: Vec<&str> = first_chunks
        .iter()
        .filter_map(|chunk| chunk.get("chunk_id").and_then(Value::as_str))
        .collect();
    let second_chunk_ids: Vec<&str> = second_chunks
        .iter()
        .filter_map(|chunk| chunk.get("chunk_id").and_then(Value::as_str))
        .collect();
    assert_eq!(first_chunk_ids, second_chunk_ids);

    let _ = join.join();
}

#[test]
fn test_t7_evidence_packet_contains_provenance_fields() {
    let (base, join) = spawn_server(
        move |_, path| {
            if path.starts_with("/res/v1/web/search") {
                MockResponse::json(
                    200,
                    brave_success_payload(&[(
                        "A",
                        "https://prov.example.com/a",
                        "Provenance snippet",
                    )]),
                )
            } else {
                MockResponse::json(500, json!({"error": "unexpected"}))
            }
        },
        2,
    );

    let config = base_runtime(
        &format!("{}/res/v1/web/search", base),
        &format!("{}/v1/responses", base),
    );

    let mut health = ProviderHealthTracker::default();
    let result = execute_web_provider_ladder_from_tool_request(
        &tool_request_packet("provenance query"),
        1_700_000_700_777i64,
        &mut health,
        &config,
    )
    .expect("provenance run should pass");

    let packet_registry = load_packet_schema_registry().expect("packet schema should load");
    validate_packet_schema_registry(&packet_registry).expect("packet schema should validate");
    validate_packet("EvidencePacket", &result.evidence_packet, &packet_registry)
        .expect("evidence packet must satisfy schema");

    let source = result
        .evidence_packet
        .get("sources")
        .and_then(Value::as_array)
        .and_then(|arr| arr.first())
        .expect("source should exist");
    assert!(source.get("provider_id").and_then(Value::as_str).is_some());
    assert!(source
        .get("canonical_url")
        .and_then(Value::as_str)
        .is_some());
    assert!(source.get("rank").and_then(Value::as_u64).is_some());

    let mut audit = json!({
        "schema_version": "1.0.0",
        "produced_by": "PH1.OS",
        "intended_consumers": ["PH1.J"],
        "created_at_ms": 1_700_000_700_888i64,
        "trace_id": "trace-run7",
        "turn_state_transition": "RETRIEVAL_EXECUTED",
        "packet_hashes": {"evidence": "hash"},
        "evidence_hash": "ev",
        "response_hash": "resp",
        "reason_codes": [],
        "policy_snapshot_id": "policy"
    });
    append_web_provider_audit_fields(&mut audit, &result.audit_metrics)
        .expect("append audit fields should pass");
    assert!(audit
        .get("turn_state_transition")
        .and_then(Value::as_object)
        .and_then(|obj| obj.get("web_provider_audit"))
        .is_some());

    let _ = join.join();
}

#[test]
fn test_t8_openai_requires_explicit_citations() {
    let (base, join) = spawn_server(
        move |method, path| {
            if method == "GET" && path.starts_with("/res/v1/web/search") {
                MockResponse::json(200, json!({"web": {"results": []}}))
            } else if method == "POST" && path.starts_with("/v1/responses") {
                MockResponse::json(
                    200,
                    openai_success_payload(
                        &[("OA", "https://no-citation.example.com/1", "Snippet")],
                        &["https://different.example.com/1"],
                    ),
                )
            } else {
                MockResponse::json(500, json!({"error": "unexpected"}))
            }
        },
        4,
    );

    let config = base_runtime(
        &format!("{}/res/v1/web/search", base),
        &format!("{}/v1/responses", base),
    );

    let mut health = ProviderHealthTracker::default();
    let err = execute_web_provider_ladder_from_tool_request(
        &tool_request_packet("citation strictness query"),
        1_700_000_700_999i64,
        &mut health,
        &config,
    )
    .expect_err("openai responses with invented citations must fail");

    assert_eq!(err.kind, ProviderErrorKind::ParseFailed);

    let _ = join.join();
}
