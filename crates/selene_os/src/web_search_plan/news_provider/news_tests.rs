#![forbid(unsafe_code)]

use crate::web_search_plan::news_provider::recency_policy::normalize_published_at;
use crate::web_search_plan::news_provider::{
    append_news_provider_audit_fields, execute_news_provider_ladder_from_tool_request,
    NewsRuntimeConfig,
};
use crate::web_search_plan::proxy::proxy_config::ProxyConfig;
use crate::web_search_plan::proxy::ProxyMode;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
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

fn tool_request_packet(query: &str, importance_tier: &str) -> Value {
    json!({
        "schema_version": "1.0.0",
        "produced_by": "PH1.X",
        "intended_consumers": ["PH1.D", "PH1.WRITE", "PH1.J"],
        "created_at_ms": 1_772_496_000_000i64,
        "trace_id": "trace-run8",
        "mode": "news",
        "query": query,
        "importance_tier": importance_tier,
        "budgets": {}
    })
}

fn base_runtime(brave_endpoint: &str, gdelt_endpoint: &str) -> NewsRuntimeConfig {
    NewsRuntimeConfig {
        brave_news_endpoint: brave_endpoint.to_string(),
        gdelt_endpoint: gdelt_endpoint.to_string(),
        max_results: 4,
        timeout_ms: 1_500,
        user_agent: "selene-news-provider-ladder/test".to_string(),
        proxy_config: ProxyConfig {
            mode: ProxyMode::Off,
            http_proxy_url: None,
            https_proxy_url: None,
        },
        brave_api_key_override: Some("test_brave_key".to_string()),
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

#[test]
fn test_t1_brave_success_no_assist() {
    let gdelt_hits = Arc::new(AtomicUsize::new(0));
    let gdelt_hits_ref = gdelt_hits.clone();

    let (base, join) = spawn_server(
        move |_, path| {
            if path.starts_with("/res/v1/news/search") {
                MockResponse::json(
                    200,
                    brave_news_payload(&[
                        (
                            "Alpha",
                            "https://news.example.com/a",
                            "Alpha snippet",
                            "2026-03-02T00:00:00Z",
                        ),
                        (
                            "Beta",
                            "https://world.example.com/b",
                            "Beta snippet",
                            "2026-03-02T01:00:00Z",
                        ),
                    ]),
                )
            } else if path.starts_with("/api/v2/doc/doc") {
                gdelt_hits_ref.fetch_add(1, Ordering::SeqCst);
                MockResponse::json(
                    200,
                    gdelt_payload(&[(
                        "Assist",
                        "https://assist.example.com/x",
                        "Assist snippet",
                        "20260302T020000Z",
                    )]),
                )
            } else {
                MockResponse::json(500, json!({"error": "unexpected path"}))
            }
        },
        2,
    );

    let config = base_runtime(
        &format!("{}/res/v1/news/search", base),
        &format!("{}/api/v2/doc/doc", base),
    );

    let result = execute_news_provider_ladder_from_tool_request(
        &tool_request_packet("latest market news", "medium"),
        1_772_496_000_000i64,
        &config,
    )
    .expect("brave success should pass");

    assert!(!result.audit_metrics.assist_used);
    assert_eq!(gdelt_hits.load(Ordering::SeqCst), 0);

    let provider_runs = result
        .evidence_packet
        .get("provider_runs")
        .and_then(Value::as_array)
        .expect("provider_runs must exist");
    assert_eq!(provider_runs.len(), 1);
    assert_eq!(
        provider_runs[0].get("provider_id").and_then(Value::as_str),
        Some("brave_news_search")
    );

    let _ = join.join();
}

#[test]
fn test_t2_brave_empty_assist_executed() {
    let (base, join) = spawn_server(
        move |_, path| {
            if path.starts_with("/res/v1/news/search") {
                MockResponse::json(200, json!({"results": []}))
            } else if path.starts_with("/api/v2/doc/doc") {
                MockResponse::json(
                    200,
                    gdelt_payload(&[(
                        "Fallback Item",
                        "https://gdelt.example.org/item",
                        "Fallback snippet",
                        "20260303T010203Z",
                    )]),
                )
            } else {
                MockResponse::json(500, json!({"error": "unexpected path"}))
            }
        },
        2,
    );

    let config = base_runtime(
        &format!("{}/res/v1/news/search", base),
        &format!("{}/api/v2/doc/doc", base),
    );

    let result = execute_news_provider_ladder_from_tool_request(
        &tool_request_packet("earthquake updates", "medium"),
        1_772_496_000_000i64,
        &config,
    )
    .expect("gdelt assist path should pass");

    assert!(result.audit_metrics.assist_used);

    let provider_runs = result
        .evidence_packet
        .get("provider_runs")
        .and_then(Value::as_array)
        .expect("provider_runs must exist");
    assert_eq!(provider_runs.len(), 2);
    assert_eq!(
        provider_runs[0]
            .pointer("/error/reason_code")
            .and_then(Value::as_str),
        Some("empty_results")
    );
    assert_eq!(
        provider_runs[1].get("provider_id").and_then(Value::as_str),
        Some("gdelt_news_assist")
    );

    let _ = join.join();
}

#[test]
fn test_t3_recency_filtering_removes_stale_items() {
    let (base, join) = spawn_server(
        move |_, path| {
            if path.starts_with("/res/v1/news/search") {
                MockResponse::json(
                    200,
                    brave_news_payload(&[
                        (
                            "Fresh",
                            "https://fresh.example.com/story",
                            "Fresh snippet",
                            "2026-03-02T00:00:00Z",
                        ),
                        (
                            "Stale",
                            "https://stale.example.com/story",
                            "Stale snippet",
                            "2025-01-01T00:00:00Z",
                        ),
                    ]),
                )
            } else {
                MockResponse::json(
                    200,
                    gdelt_payload(&[(
                        "Unused",
                        "https://unused.example.com/story",
                        "Unused snippet",
                        "20260302T000000Z",
                    )]),
                )
            }
        },
        2,
    );

    let config = base_runtime(
        &format!("{}/res/v1/news/search", base),
        &format!("{}/api/v2/doc/doc", base),
    );

    let result = execute_news_provider_ladder_from_tool_request(
        &tool_request_packet("policy updates", "medium"),
        1_772_496_000_000i64,
        &config,
    )
    .expect("recency filtering should pass");

    assert_eq!(result.audit_metrics.filtered_by_recency_count, 1);

    let sources = result
        .evidence_packet
        .get("sources")
        .and_then(Value::as_array)
        .expect("sources must exist");
    assert_eq!(sources.len(), 1);
    assert_eq!(
        sources[0].get("url").and_then(Value::as_str),
        Some("https://fresh.example.com/story")
    );

    let _ = join.join();
}

#[test]
fn test_t4_publish_date_normalization_deterministic() {
    let a =
        normalize_published_at("2026-03-03T10:20:30+08:00").expect("offset timestamp should parse");
    let b =
        normalize_published_at("2026-03-03T10:20:30+08:00").expect("offset timestamp should parse");
    assert_eq!(a, b);
    assert_eq!(a.utc_rfc3339, "2026-03-03T02:20:30Z");

    let compact =
        normalize_published_at("20260303T022030Z").expect("compact UTC timestamp should parse");
    assert_eq!(compact.utc_rfc3339, "2026-03-03T02:20:30Z");
}

#[test]
fn test_t5_dedup_preserves_brave_precedence() {
    let (base, join) = spawn_server(
        move |_, path| {
            if path.starts_with("/res/v1/news/search") {
                MockResponse::json(
                    200,
                    brave_news_payload(&[
                        (
                            "Dup",
                            "https://dup.example.com/story?utm_source=ads",
                            "Brave version",
                            "2026-03-02T00:00:00Z",
                        ),
                        (
                            "Second",
                            "https://dup.example.com/second",
                            "Second brave item",
                            "2026-03-02T00:10:00Z",
                        ),
                    ]),
                )
            } else if path.starts_with("/api/v2/doc/doc") {
                MockResponse::json(
                    200,
                    gdelt_payload(&[
                        (
                            "Dup GDELT",
                            "https://dup.example.com/story",
                            "GDELT corroboration",
                            "20260302T020203Z",
                        ),
                        (
                            "Unique GDELT",
                            "https://other.example.net/unique",
                            "Unique gdelt item",
                            "20260302T030203Z",
                        ),
                    ]),
                )
            } else {
                MockResponse::json(500, json!({"error": "unexpected path"}))
            }
        },
        2,
    );

    let mut config = base_runtime(
        &format!("{}/res/v1/news/search", base),
        &format!("{}/api/v2/doc/doc", base),
    );
    config.max_results = 5;

    let result = execute_news_provider_ladder_from_tool_request(
        &tool_request_packet("merger rumors", "high"),
        1_772_496_000_000i64,
        &config,
    )
    .expect("dedup + assist should pass");

    let sources = result
        .evidence_packet
        .get("sources")
        .and_then(Value::as_array)
        .expect("sources must exist");
    assert_eq!(sources.len(), 3);
    assert_eq!(
        sources[0].get("provider_id").and_then(Value::as_str),
        Some("brave_news_search")
    );
    assert_eq!(
        sources[0].get("canonical_url").and_then(Value::as_str),
        Some("https://dup.example.com/story")
    );
    assert_eq!(
        sources[0]
            .get("corroborated_by_assist")
            .and_then(Value::as_bool),
        Some(true)
    );

    let dedup_count = result
        .evidence_packet
        .pointer("/trust_metadata/news_provider_ladder/dedup_count")
        .and_then(Value::as_u64)
        .expect("dedup_count must exist");
    assert_eq!(dedup_count, 1);

    let _ = join.join();
}

#[test]
fn test_t6_diversity_threshold_deterministic() {
    let (base, join) = spawn_server(
        move |_, path| {
            if path.starts_with("/res/v1/news/search") {
                MockResponse::json(
                    200,
                    brave_news_payload(&[(
                        "Lead",
                        "https://solo.example.com/lead",
                        "Lead snippet",
                        "2026-03-02T00:00:00Z",
                    )]),
                )
            } else if path.starts_with("/api/v2/doc/doc") {
                MockResponse::json(
                    200,
                    gdelt_payload(&[(
                        "Assist",
                        "https://other.example.org/assist",
                        "Assist snippet",
                        "20260302T010203Z",
                    )]),
                )
            } else {
                MockResponse::json(500, json!({"error": "unexpected path"}))
            }
        },
        4,
    );

    let config = base_runtime(
        &format!("{}/res/v1/news/search", base),
        &format!("{}/api/v2/doc/doc", base),
    );

    let result_a = execute_news_provider_ladder_from_tool_request(
        &tool_request_packet("supply chain", "high"),
        1_772_496_000_000i64,
        &config,
    )
    .expect("run A should pass");

    let result_b = execute_news_provider_ladder_from_tool_request(
        &tool_request_packet("supply chain", "high"),
        1_772_496_000_000i64,
        &config,
    )
    .expect("run B should pass");

    assert_eq!(
        result_a.audit_metrics.diversity_threshold_met,
        result_b.audit_metrics.diversity_threshold_met
    );
    assert_eq!(
        result_a.audit_metrics.distinct_domain_count,
        result_b.audit_metrics.distinct_domain_count
    );
    assert!(result_a.audit_metrics.assist_used);
    assert!(result_a.audit_metrics.diversity_threshold_met);

    let _ = join.join();
}

#[test]
fn test_t7_conflict_clustering_reproducible() {
    let (base, join) = spawn_server(
        move |_, path| {
            if path.starts_with("/res/v1/news/search") {
                MockResponse::json(
                    200,
                    brave_news_payload(&[(
                        "Acme merger update today",
                        "https://finance.example.com/acme",
                        "Officials confirmed merger completed",
                        "2026-03-02T00:00:00Z",
                    )]),
                )
            } else if path.starts_with("/api/v2/doc/doc") {
                MockResponse::json(
                    200,
                    gdelt_payload(&[(
                        "Acme merger update today",
                        "https://wire.example.net/acme",
                        "Officials deny merger completed",
                        "20260302T010203Z",
                    )]),
                )
            } else {
                MockResponse::json(500, json!({"error": "unexpected path"}))
            }
        },
        2,
    );

    let config = base_runtime(
        &format!("{}/res/v1/news/search", base),
        &format!("{}/api/v2/doc/doc", base),
    );

    let result = execute_news_provider_ladder_from_tool_request(
        &tool_request_packet("acme merger", "high"),
        1_772_496_000_000i64,
        &config,
    )
    .expect("conflict clustering path should pass");

    assert_eq!(result.audit_metrics.contradiction_clusters_detected, 1);

    let clusters = result
        .evidence_packet
        .pointer("/trust_metadata/news_provider_ladder/contradiction_clusters")
        .and_then(Value::as_array)
        .expect("contradiction clusters must exist");
    assert_eq!(clusters.len(), 1);
    assert_eq!(
        clusters[0].get("group_id").and_then(Value::as_str),
        Some("news_conflict_001")
    );

    let content_chunks = result
        .evidence_packet
        .get("content_chunks")
        .and_then(Value::as_array)
        .expect("content_chunks must exist");
    assert!(content_chunks.iter().all(|chunk| {
        chunk
            .get("contradiction_group_id")
            .and_then(Value::as_str)
            .is_some()
    }));

    let _ = join.join();
}

#[test]
fn test_t8_evidence_packet_completeness_news_mode() {
    let (base, join) = spawn_server(
        move |_, path| {
            if path.starts_with("/res/v1/news/search") {
                MockResponse::json(
                    200,
                    brave_news_payload(&[(
                        "Completeness",
                        "https://complete.example.com/a",
                        "Complete snippet",
                        "2026-03-02T00:00:00Z",
                    )]),
                )
            } else {
                MockResponse::json(
                    200,
                    gdelt_payload(&[(
                        "Unused",
                        "https://unused.example.com/a",
                        "Unused snippet",
                        "20260302T010203Z",
                    )]),
                )
            }
        },
        2,
    );

    let config = base_runtime(
        &format!("{}/res/v1/news/search", base),
        &format!("{}/api/v2/doc/doc", base),
    );

    let result = execute_news_provider_ladder_from_tool_request(
        &tool_request_packet("completeness", "medium"),
        1_772_496_000_000i64,
        &config,
    )
    .expect("completeness path should pass");

    let provider_runs = result
        .evidence_packet
        .get("provider_runs")
        .and_then(Value::as_array)
        .expect("provider_runs must exist");
    assert!(!provider_runs.is_empty());
    assert!(provider_runs[0].get("provider_id").is_some());
    assert_eq!(
        provider_runs[0].get("endpoint").and_then(Value::as_str),
        Some("news")
    );
    assert!(provider_runs[0].get("latency_ms").is_some());
    assert!(provider_runs[0].get("results_count").is_some());
    assert!(provider_runs[0].get("error").is_some());

    let sources = result
        .evidence_packet
        .get("sources")
        .and_then(Value::as_array)
        .expect("sources must exist");
    assert!(!sources.is_empty());
    let source = &sources[0];
    assert!(source.get("title").is_some());
    assert!(source.get("url").is_some());
    assert!(source.get("snippet").is_some());
    assert!(source.get("published_at").is_some());
    assert_eq!(
        source.get("media_type").and_then(Value::as_str),
        Some("news")
    );
    assert!(source.get("provider_id").is_some());
    assert!(source.get("rank").is_some());
    assert!(source.get("canonical_url").is_some());
    assert!(source.get("freshness_score").is_some());

    let mut audit_packet = json!({
        "schema_version": "1.0.0",
        "produced_by": "PH1.OS",
        "intended_consumers": ["PH1.J"],
        "created_at_ms": 1_772_496_000_000i64,
        "trace_id": "trace-run8-audit",
        "turn_state_transition": "AUDIT_COMMITTED",
        "packet_hashes": {},
        "evidence_hash": "evidence-hash",
        "response_hash": "response-hash",
        "reason_codes": [],
        "policy_snapshot_id": "policy-1"
    });
    append_news_provider_audit_fields(&mut audit_packet, &result.audit_metrics)
        .expect("audit append should pass");
    assert!(audit_packet
        .pointer("/turn_state_transition/news_provider_audit/assist_used")
        .is_some());

    let _ = join.join();
}
