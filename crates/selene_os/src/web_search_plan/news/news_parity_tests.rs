#![forbid(unsafe_code)]

use crate::web_search_plan::news::{
    execute_news_provider_ladder_from_tool_request as execute_news_from_tool_request,
    NewsProviderErrorKind, NewsRuntimeConfig, DEFAULT_BRAVE_NEWS_ENDPOINT, DEFAULT_GDELT_ENDPOINT,
};
use crate::web_search_plan::proxy::proxy_config::ProxyConfig;
use crate::web_search_plan::proxy::ProxyMode;
use crate::web_search_plan::web_provider::health_state::{HealthPolicy, ProviderHealthTracker};
use selene_engines::ph1providerctl::{
    route_provider, ProviderControlProvider, ProviderControlRoute, ProviderLane,
    ProviderNetworkPolicy, ProviderRouteRequest,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn stage2_news_provider_global_off_blocks_external_before_network() {
    let config = runtime(DEFAULT_BRAVE_NEWS_ENDPOINT, DEFAULT_GDELT_ENDPOINT);
    let request = tool_request_packet("synthetic acorn delta works update", "medium");

    let mut health = ProviderHealthTracker::default();
    let err = execute_news_from_tool_request(&request, 1_772_496_000_000i64, &mut health, &config)
        .expect_err("default provider-off policy must block external news providers");

    assert_eq!(err.kind, NewsProviderErrorKind::PolicyViolation);
    assert!(err.message.contains("stage2_provider_control=1"));
    assert!(err.message.contains("deny_reason=WEB_ADMIN_DISABLED"));
    assert!(err.message.contains("provider_call_attempt_count=0"));
    assert!(err.message.contains("provider_network_dispatch_count=0"));
    assert!(err.message.contains("billable_class=BLOCKED_NOT_BILLABLE"));
    assert!(err.message.contains("billing_scope=NON_BILLABLE"));
}

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

#[test]
fn stage8_os_news_lane_selects_fake_news_provider_before_premium_fallback() {
    let mut policy = ProviderNetworkPolicy::fake_test_allowing(1);
    policy
        .provider_specific_enabled
        .insert("news_current_events".to_string(), true);
    policy
        .provider_specific_enabled
        .insert("brave".to_string(), true);
    policy.fallback_enabled = true;
    policy.max_fallback_calls_this_turn = 1;

    let mut request = ProviderRouteRequest::public_web("latest Synthetic Entity C update");
    request.route = ProviderControlRoute::NewsSearch;
    request.news_provider_available = true;
    request.fallback_allowed = true;

    let decision = route_provider(&policy, &request);

    assert_eq!(decision.selected_lane, ProviderLane::NewsCurrentEvents);
    assert_eq!(
        decision.selected_provider,
        Some(ProviderControlProvider::NewsCurrentEvents)
    );
    assert_eq!(
        decision.fallback_provider,
        Some(ProviderControlProvider::BraveNewsSearch)
    );
    assert!(!decision.fallback_allowed);
}

fn tool_request_packet(query: &str, importance_tier: &str) -> Value {
    json!({
        "schema_version": "1.0.0",
        "produced_by": "PH1.X",
        "intended_consumers": ["PH1.D", "PH1.WRITE", "PH1.J"],
        "created_at_ms": 1_772_496_000_000i64,
        "trace_id": "trace-news-parity",
        "mode": "news",
        "query": query,
        "importance_tier": importance_tier,
        "budgets": {}
    })
}

fn runtime(brave_endpoint: &str, gdelt_endpoint: &str) -> NewsRuntimeConfig {
    NewsRuntimeConfig {
        brave_news_endpoint: brave_endpoint.to_string(),
        gdelt_endpoint: gdelt_endpoint.to_string(),
        max_results: 4,
        timeout_ms: 1_500,
        user_agent: "selene-news-parity/test".to_string(),
        proxy_config: ProxyConfig {
            mode: ProxyMode::Off,
            http_proxy_url: None,
            https_proxy_url: None,
        },
        health_policy: HealthPolicy::default(),
        brave_api_key_override: Some("test_brave_key".to_string()),
        brave_news_fixture_json: None,
        gdelt_fixture_json: None,
    }
}

fn brave_news_payload(items: &[(&str, &str, &str, &str)]) -> Value {
    json!({
        "results": items
            .iter()
            .map(|(title, url, snippet, published)| {
                json!({
                    "title": title,
                    "url": url,
                    "description": snippet,
                    "published": published,
                })
            })
            .collect::<Vec<Value>>()
    })
}

fn gdelt_payload(items: &[(&str, &str, &str, &str)]) -> Value {
    json!({
        "articles": items
            .iter()
            .map(|(title, url, snippet, seendate)| {
                json!({
                    "title": title,
                    "url": url,
                    "snippet": snippet,
                    "seendate": seendate,
                })
            })
            .collect::<Vec<Value>>()
    })
}

fn source_urls(packet: &Value) -> Vec<String> {
    packet
        .get("sources")
        .and_then(Value::as_array)
        .expect("sources should be array")
        .iter()
        .filter_map(|source| source.get("url").and_then(Value::as_str))
        .map(ToString::to_string)
        .collect()
}

#[test]
fn test_parity_sources_ordering_and_recency_filtering() {
    let (base, join) = spawn_server(
        move |_, path| {
            if path.starts_with("/res/v1/news/search") {
                MockResponse::json(
                    200,
                    brave_news_payload(&[
                        (
                            "Fresh Alpha",
                            "https://alpha.example.com/a",
                            "Alpha fresh snippet",
                            "2026-03-02T00:00:00Z",
                        ),
                        (
                            "Stale Beta",
                            "https://beta.example.com/b",
                            "Beta stale snippet",
                            "2025-01-02T00:00:00Z",
                        ),
                        (
                            "Fresh Gamma",
                            "https://gamma.example.com/c",
                            "Gamma fresh snippet",
                            "2026-03-01T09:00:00Z",
                        ),
                    ]),
                )
            } else if path.starts_with("/api/v2/doc/doc") {
                MockResponse::json(200, gdelt_payload(&[]))
            } else {
                MockResponse::json(500, json!({"error": "unexpected path"}))
            }
        },
        2,
    );

    let config = runtime(
        &format!("{}/res/v1/news/search", base),
        &format!("{}/api/v2/doc/doc", base),
    );
    let request = tool_request_packet("market update", "high");

    let mut health = ProviderHealthTracker::default();
    let result =
        execute_news_from_tool_request(&request, 1_772_496_000_000i64, &mut health, &config)
            .expect("news run should pass");

    assert_eq!(
        source_urls(&result.evidence_packet),
        vec![
            "https://alpha.example.com/a".to_string(),
            "https://gamma.example.com/c".to_string()
        ]
    );
    assert_eq!(result.audit_metrics.filtered_by_recency_count, 1);
    assert_eq!(result.audit_metrics.recency_window_applied, 7);

    let _ = join.join();
}

#[test]
fn test_parity_diversity_rule_and_assist_usage() {
    let (base, join) = spawn_server(
        move |_, path| {
            if path.starts_with("/res/v1/news/search") {
                MockResponse::json(
                    200,
                    brave_news_payload(&[(
                        "Single domain only",
                        "https://same.example.com/a",
                        "Lead source only",
                        "2026-03-03T00:00:00Z",
                    )]),
                )
            } else if path.starts_with("/api/v2/doc/doc") {
                MockResponse::json(
                    200,
                    gdelt_payload(&[(
                        "Assist source",
                        "https://other.example.org/b",
                        "Assist improves diversity",
                        "20260303T010203Z",
                    )]),
                )
            } else {
                MockResponse::json(500, json!({"error": "unexpected path"}))
            }
        },
        4,
    );

    let config = runtime(
        &format!("{}/res/v1/news/search", base),
        &format!("{}/api/v2/doc/doc", base),
    );
    let request = tool_request_packet("diversity check", "high");

    let mut health = ProviderHealthTracker::default();
    let result =
        execute_news_from_tool_request(&request, 1_772_496_000_000i64, &mut health, &config)
            .expect("news run should pass");

    assert_eq!(result.audit_metrics.distinct_domain_count, 2);
    assert!(result.audit_metrics.diversity_threshold_met);
    assert!(result.audit_metrics.assist_used);

    let _ = join.join();
}

#[test]
fn test_parity_conflict_clustering() {
    let (base, join) = spawn_server(
        move |_, path| {
            if path.starts_with("/res/v1/news/search") {
                MockResponse::json(
                    200,
                    brave_news_payload(&[
                        (
                            "Acme merger talks advance",
                            "https://source1.example.com/a",
                            "Acme confirms merger talks are active",
                            "2026-03-03T00:00:00Z",
                        ),
                        (
                            "Acme merger talks advance",
                            "https://source2.example.com/b",
                            "Acme denies merger talks are active",
                            "2026-03-03T01:00:00Z",
                        ),
                    ]),
                )
            } else if path.starts_with("/api/v2/doc/doc") {
                MockResponse::json(200, gdelt_payload(&[]))
            } else {
                MockResponse::json(500, json!({"error": "unexpected path"}))
            }
        },
        2,
    );

    let config = runtime(
        &format!("{}/res/v1/news/search", base),
        &format!("{}/api/v2/doc/doc", base),
    );
    let request = tool_request_packet("acme merger talks", "medium");

    let mut health = ProviderHealthTracker::default();
    let result =
        execute_news_from_tool_request(&request, 1_772_496_000_000i64, &mut health, &config)
            .expect("news run should pass");

    assert_eq!(result.audit_metrics.contradiction_clusters_detected, 1);
    let clusters = result
        .evidence_packet
        .pointer("/trust_metadata/news_provider_ladder/contradiction_clusters")
        .and_then(Value::as_array)
        .expect("clusters should exist");
    assert_eq!(clusters.len(), 1);
    assert_eq!(
        clusters[0].get("topic_key").and_then(Value::as_str),
        Some("acme merger talks advance")
    );

    let _ = join.join();
}
