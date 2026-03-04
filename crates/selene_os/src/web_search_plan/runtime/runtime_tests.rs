#![forbid(unsafe_code)]

use crate::web_search_plan::news::NewsRuntimeConfig;
use crate::web_search_plan::proxy::proxy_config::ProxyConfig;
use crate::web_search_plan::proxy::ProxyMode;
use crate::web_search_plan::replay::snapshot::hash_canonical_json;
use crate::web_search_plan::runtime::orchestrator::{
    execute_web_search_turn_with_dependencies, RuntimeDependencies, RuntimeServiceTrace,
};
use crate::web_search_plan::web_provider::WebProviderRuntimeConfig;
use crate::web_search_plan::write::WriteFormatMode;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
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

    fn html(status: u16, body: &str) -> Self {
        Self {
            status,
            headers: BTreeMap::from([(
                "Content-Type".to_string(),
                "text/html; charset=utf-8".to_string(),
            )]),
            body: body.as_bytes().to_vec(),
        }
    }
}

fn spawn_server<F>(handler: F, max_requests: usize) -> (String, thread::JoinHandle<()>)
where
    F: Fn(&str, &str, &str) -> MockResponse + Send + Sync + 'static,
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
    let base_for_thread = base.clone();

    let join = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut served = 0usize;
        while served < max_requests && Instant::now() < deadline {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let (method, path) = read_request_head(&mut stream);
                    let response = handler(&method, &path, base_for_thread.as_str());
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

fn replay_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/web_search_plan/replay_fixtures")
        .join(name)
}

fn load_replay_fixture(name: &str) -> Value {
    let text = fs::read_to_string(replay_fixture_path(name)).expect("fixture should load");
    serde_json::from_str::<Value>(&text).expect("fixture should parse")
}

fn make_turn_input(trace_id: &str, created_at_ms: i64, query: &str) -> Value {
    json!({
        "schema_version": "1.0.0",
        "produced_by": "API",
        "intended_consumers": ["PH1.X"],
        "created_at_ms": created_at_ms,
        "trace_id": trace_id,
        "transcript": query,
        "identity_scope": "user",
        "language": "en",
        "session_id": "session-runtime-tests"
    })
}

fn make_search_assist(trace_id: &str, created_at_ms: i64, search_required: bool) -> Value {
    json!({
        "schema_version": "1.0.0",
        "produced_by": "PH1.X",
        "intended_consumers": ["PH1.SEARCH"],
        "created_at_ms": created_at_ms,
        "trace_id": trace_id,
        "intent_class": "information_request",
        "search_required": search_required,
        "confidence": 0.9,
        "missing_fields": [],
        "risk_flags": []
    })
}

fn make_tool_request(
    trace_id: &str,
    created_at_ms: i64,
    mode: &str,
    query: &str,
    importance_tier: &str,
) -> Value {
    json!({
        "schema_version": "1.0.0",
        "produced_by": "PH1.X",
        "intended_consumers": ["PH1.SEARCH", "PH1.E", "PH1.D", "PH1.WRITE", "PH1.J"],
        "created_at_ms": created_at_ms,
        "trace_id": trace_id,
        "mode": mode,
        "query": query,
        "importance_tier": importance_tier,
        "budgets": {}
    })
}

fn runtime_deps_for_base(base: &str) -> RuntimeDependencies {
    RuntimeDependencies {
        web_runtime_config: WebProviderRuntimeConfig {
            brave_endpoint: format!("{}/res/v1/web/search", base),
            openai_endpoint: format!("{}/v1/responses", base),
            openai_model: "gpt-4o-mini".to_string(),
            max_results: 3,
            timeout_ms: 1_500,
            user_agent: "selene-runtime-tests".to_string(),
            proxy_config: ProxyConfig {
                mode: ProxyMode::Off,
                http_proxy_url: None,
                https_proxy_url: None,
            },
            health_policy:
                crate::web_search_plan::web_provider::health_state::HealthPolicy::default(),
            brave_api_key_override: Some("test_brave_key".to_string()),
            openai_api_key_override: Some("test_openai_key".to_string()),
        },
        news_runtime_config: NewsRuntimeConfig {
            brave_news_endpoint: format!("{}/res/v1/news/search", base),
            gdelt_endpoint: format!("{}/api/v2/doc/doc", base),
            max_results: 4,
            timeout_ms: 1_500,
            user_agent: "selene-runtime-tests".to_string(),
            proxy_config: ProxyConfig {
                mode: ProxyMode::Off,
                http_proxy_url: None,
                https_proxy_url: None,
            },
            health_policy:
                crate::web_search_plan::web_provider::health_state::HealthPolicy::default(),
            brave_api_key_override: Some("test_brave_key".to_string()),
        },
        planning_policy: crate::web_search_plan::planning::PlanningPolicy::default(),
        write_format_mode: WriteFormatMode::Standard,
        learn_observation_enabled: false,
        service_trace: None,
    }
}

fn assert_success_state_path(audit_packet: &Value) {
    let transitions = audit_packet
        .pointer("/turn_state_transition/transitions")
        .and_then(Value::as_array)
        .expect("audit transitions should exist");
    let path = transitions
        .iter()
        .map(|entry| entry.get("to").and_then(Value::as_str).unwrap_or_default())
        .collect::<Vec<&str>>();
    assert_eq!(
        path,
        vec![
            "INPUT_PARSED",
            "INTENT_CLASSIFIED",
            "PLAN_SELECTED",
            "RETRIEVAL_EXECUTED",
            "EVIDENCE_LOCKED",
            "SYNTHESIS_READY",
            "OUTPUT_RENDERED",
            "AUDIT_COMMITTED",
            "TURN_COMPLETED",
        ]
    );
}

fn sanitized_provider_latency(packet: &Value) -> Value {
    let mut cloned = packet.clone();
    if let Some(runs) = cloned
        .get_mut("provider_runs")
        .and_then(Value::as_array_mut)
    {
        for run in runs {
            if let Some(object) = run.as_object_mut() {
                object.insert("latency_ms".to_string(), json!(0));
            }
        }
    }
    cloned
}

#[test]
fn test_t1_web_happy_path_returns_all_packets_with_citations_and_stable_hashes() {
    let fixture = load_replay_fixture("web_answer_basic.json");
    let trace_id = fixture["evidence_packet"]["trace_id"]
        .as_str()
        .expect("trace id should exist");
    let query = fixture["evidence_packet"]["query"]
        .as_str()
        .expect("query should exist");
    let created_at_ms = fixture["evidence_packet"]["created_at_ms"]
        .as_i64()
        .expect("created_at_ms should exist");

    let (base, join) = spawn_server(
        move |method, path, base_url| {
            if method == "GET" && path.starts_with("/res/v1/web/search") {
                MockResponse::json(
                    200,
                    json!({
                        "web": {
                            "results": [
                                {
                                    "title": "Selene Determinism Notes",
                                    "url": format!("{}/page/determinism", base_url),
                                    "description": "Selene enforces deterministic ordering and fail-closed behavior."
                                },
                                {
                                    "title": "Selene Reliability",
                                    "url": format!("{}/page/reliability", base_url),
                                    "description": "Reliability controls are evidence-bound and citation-backed."
                                }
                            ]
                        }
                    }),
                )
            } else if method == "GET" && path.starts_with("/page/determinism") {
                MockResponse::html(
                    200,
                    "<html><body><h1>Determinism</h1><p>Selene enforces deterministic ordering and fail-closed behavior with citation requirements for every answer case.</p></body></html>",
                )
            } else if method == "GET" && path.starts_with("/page/reliability") {
                MockResponse::html(
                    200,
                    "<html><body><h1>Reliability</h1><p>The runtime path is evidence-bound and auditable, and every claim must map to a source citation.</p></body></html>",
                )
            } else {
                MockResponse::json(500, json!({"error": "unexpected path"}))
            }
        },
        8,
    );

    let mut deps_a = runtime_deps_for_base(&base);
    let mut deps_b = runtime_deps_for_base(&base);

    let run_a = execute_web_search_turn_with_dependencies(
        make_turn_input(trace_id, created_at_ms, query),
        make_search_assist(trace_id, created_at_ms.saturating_add(1), true),
        make_tool_request(
            trace_id,
            created_at_ms.saturating_add(2),
            "web",
            query,
            "medium",
        ),
        "policy-snapshot-runtime-v1".to_string(),
        &mut deps_a,
    )
    .expect("web runtime should succeed");
    let run_b = execute_web_search_turn_with_dependencies(
        make_turn_input(trace_id, created_at_ms, query),
        make_search_assist(trace_id, created_at_ms.saturating_add(1), true),
        make_tool_request(
            trace_id,
            created_at_ms.saturating_add(2),
            "web",
            query,
            "medium",
        ),
        "policy-snapshot-runtime-v1".to_string(),
        &mut deps_b,
    )
    .expect("web runtime should be deterministic");

    assert_success_state_path(&run_a.audit_packet);
    assert_eq!(
        run_a
            .synthesis_packet
            .get("citations")
            .and_then(Value::as_array)
            .map(|entries| entries.len())
            .unwrap_or(0),
        4
    );
    assert!(run_a
        .write_packet
        .get("citation_map")
        .and_then(Value::as_object)
        .map(|map| !map.is_empty())
        .unwrap_or(false));

    assert_eq!(
        hash_canonical_json(&run_a.evidence_packet).expect("hash should compute"),
        hash_canonical_json(&run_b.evidence_packet).expect("hash should compute")
    );
    assert_eq!(
        hash_canonical_json(&run_a.synthesis_packet).expect("hash should compute"),
        hash_canonical_json(&run_b.synthesis_packet).expect("hash should compute")
    );
    assert_eq!(
        hash_canonical_json(&run_a.write_packet).expect("hash should compute"),
        hash_canonical_json(&run_b.write_packet).expect("hash should compute")
    );
    assert_eq!(
        hash_canonical_json(&run_a.audit_packet).expect("hash should compute"),
        hash_canonical_json(&run_b.audit_packet).expect("hash should compute")
    );

    let _ = join.join();
}

#[test]
fn test_t2_refusal_path_insufficient_evidence_fails_closed_with_reason_code() {
    let fixture = load_replay_fixture("web_refusal_insufficient.json");
    let trace_id = fixture["evidence_packet"]["trace_id"]
        .as_str()
        .expect("trace id should exist");
    let query = fixture["evidence_packet"]["query"]
        .as_str()
        .expect("query should exist");
    let created_at_ms = fixture["evidence_packet"]["created_at_ms"]
        .as_i64()
        .expect("created_at_ms should exist");

    let (base, _join) = spawn_server(
        move |_, _, _| MockResponse::json(500, json!({"error": "unused in refusal test"})),
        1,
    );
    let mut deps = runtime_deps_for_base(&base);

    let error = execute_web_search_turn_with_dependencies(
        make_turn_input(trace_id, created_at_ms, query),
        make_search_assist(trace_id, created_at_ms.saturating_add(1), false),
        make_tool_request(
            trace_id,
            created_at_ms.saturating_add(2),
            "web",
            query,
            "medium",
        ),
        "policy-snapshot-runtime-v1".to_string(),
        &mut deps,
    )
    .expect_err("search_required=false should fail closed");

    assert_eq!(error.reason_code, "insufficient_evidence");
    assert_eq!(
        error
            .transitions
            .last()
            .map(|entry| entry.to.as_str())
            .unwrap_or(""),
        "TURN_FAILED_CLOSED"
    );
    assert!(error
        .debug_packet
        .as_ref()
        .map(|packet| packet.get("reason_code").is_some())
        .unwrap_or(true));
}

#[test]
fn test_t3_news_path_is_deterministic_and_citation_backed() {
    let fixture = load_replay_fixture("news_answer_basic.json");
    let trace_id = fixture["evidence_packet"]["trace_id"]
        .as_str()
        .expect("trace id should exist");
    let query = fixture["evidence_packet"]["query"]
        .as_str()
        .expect("query should exist");
    let created_at_ms = fixture["evidence_packet"]["created_at_ms"]
        .as_i64()
        .expect("created_at_ms should exist");

    let (base, join) = spawn_server(
        move |method, path, _| {
            if method == "GET" && path.starts_with("/res/v1/news/search") {
                MockResponse::json(
                    200,
                    json!({
                        "results": [
                            {
                                "title": "Selene Runtime Launch",
                                "url": "https://news.example.com/selene-launch",
                                "description": "Selene runtime launch emphasizes deterministic evidence handling.",
                                "published": "2026-03-03T10:00:00Z"
                            },
                            {
                                "title": "Selene Audit Enhancements",
                                "url": "https://analysis.example.org/selene-audit",
                                "description": "Audit trail now captures state transitions and reason codes.",
                                "published": "2026-03-03T11:00:00Z"
                            }
                        ]
                    }),
                )
            } else if method == "GET" && path.starts_with("/api/v2/doc/doc") {
                MockResponse::json(200, json!({"articles": []}))
            } else {
                MockResponse::json(500, json!({"error": "unexpected path"}))
            }
        },
        6,
    );

    let mut deps_a = runtime_deps_for_base(&base);
    let mut deps_b = runtime_deps_for_base(&base);

    let run_a = execute_web_search_turn_with_dependencies(
        make_turn_input(trace_id, created_at_ms, query),
        make_search_assist(trace_id, created_at_ms.saturating_add(1), true),
        make_tool_request(
            trace_id,
            created_at_ms.saturating_add(2),
            "news",
            query,
            "medium",
        ),
        "policy-snapshot-runtime-v1".to_string(),
        &mut deps_a,
    )
    .expect("news runtime should succeed");
    let run_b = execute_web_search_turn_with_dependencies(
        make_turn_input(trace_id, created_at_ms, query),
        make_search_assist(trace_id, created_at_ms.saturating_add(1), true),
        make_tool_request(
            trace_id,
            created_at_ms.saturating_add(2),
            "news",
            query,
            "medium",
        ),
        "policy-snapshot-runtime-v1".to_string(),
        &mut deps_b,
    )
    .expect("news runtime should be deterministic");

    assert_success_state_path(&run_a.audit_packet);
    assert!(run_a
        .synthesis_packet
        .get("citations")
        .and_then(Value::as_array)
        .map(|entries| !entries.is_empty())
        .unwrap_or(false));
    assert!(run_a
        .write_packet
        .get("citation_map")
        .and_then(Value::as_object)
        .map(|map| !map.is_empty())
        .unwrap_or(false));

    assert_eq!(
        hash_canonical_json(&sanitized_provider_latency(&run_a.evidence_packet))
            .expect("hash should compute"),
        hash_canonical_json(&sanitized_provider_latency(&run_b.evidence_packet))
            .expect("hash should compute")
    );
    assert_eq!(
        hash_canonical_json(&run_a.synthesis_packet).expect("hash should compute"),
        hash_canonical_json(&run_b.synthesis_packet).expect("hash should compute")
    );
    assert_eq!(
        hash_canonical_json(&run_a.write_packet).expect("hash should compute"),
        hash_canonical_json(&run_b.write_packet).expect("hash should compute")
    );

    let _ = join.join();
}

#[test]
fn test_all_modes_route_through_orchestrator() {
    let trace_id = "trace-runtime-mode-route";
    let created_at_ms = 1_703_100_000_000_i64;

    let (base, join) = spawn_server(
        move |method, path, base_url| {
            if method == "GET" && path.starts_with("/res/v1/web/search") {
                MockResponse::json(
                    200,
                    json!({
                        "web": {
                            "results": [
                                {
                                    "title": "Runtime Mode Routing",
                                    "url": format!("{}/page/mode-routing", base_url),
                                    "description": "Mode routing remains deterministic and evidence-bound."
                                },
                                {
                                    "title": "Runtime Mode Routing Companion",
                                    "url": format!("{}/page/mode-routing-2", base_url),
                                    "description": "Companion source for deterministic synthesis sufficiency."
                                }
                            ]
                        }
                    }),
                )
            } else if method == "GET" && path.starts_with("/res/v1/news/search") {
                MockResponse::json(
                    200,
                    json!({
                        "results": [
                            {
                                "title": "Runtime News Routing",
                                "url": "https://news.example.com/runtime-routing",
                                "description": "News lane remains deterministic.",
                                "published": "2026-03-03T10:00:00Z"
                            },
                            {
                                "title": "Runtime News Routing Companion",
                                "url": "https://analysis.example.org/runtime-routing",
                                "description": "Companion source for deterministic news synthesis.",
                                "published": "2026-03-03T11:00:00Z"
                            }
                        ]
                    }),
                )
            } else if method == "GET" && path.starts_with("/api/v2/doc/doc") {
                MockResponse::json(200, json!({"articles": []}))
            } else if method == "GET"
                && (path.starts_with("/page/mode-routing")
                    || path.starts_with("/page/mode-routing-2")
                    || path.starts_with("/page/url-fetch"))
            {
                MockResponse::html(
                    200,
                    "<html><body><p>Deterministic mode routing content for runtime tests.</p></body></html>",
                )
            } else {
                MockResponse::json(500, json!({"error": "unexpected path"}))
            }
        },
        16,
    );

    let mut deps = runtime_deps_for_base(&base);

    for mode in ["web", "news"] {
        let query = "runtime orchestrator route check".to_string();
        let result = execute_web_search_turn_with_dependencies(
            make_turn_input(trace_id, created_at_ms, query.as_str()),
            make_search_assist(trace_id, created_at_ms.saturating_add(1), true),
            make_tool_request(
                trace_id,
                created_at_ms.saturating_add(2),
                mode,
                query.as_str(),
                "medium",
            ),
            "policy-snapshot-runtime-v1".to_string(),
            &mut deps,
        )
        .unwrap_or_else(|error| {
            panic!(
                "mode {} should succeed but failed: {}",
                mode, error.reason_code
            )
        });
        assert_success_state_path(&result.audit_packet);
    }

    let url_fetch_query = format!("{}/page/url-fetch", base);
    let url_fetch_error = execute_web_search_turn_with_dependencies(
        make_turn_input(trace_id, created_at_ms, url_fetch_query.as_str()),
        make_search_assist(trace_id, created_at_ms.saturating_add(1), true),
        make_tool_request(
            trace_id,
            created_at_ms.saturating_add(2),
            "url_fetch",
            url_fetch_query.as_str(),
            "medium",
        ),
        "policy-snapshot-runtime-v1".to_string(),
        &mut deps,
    )
    .expect_err("url_fetch mode should fail closed when evidence is insufficient");
    assert!(
        matches!(
            url_fetch_error.reason_code.as_str(),
            "empty_results" | "insufficient_evidence" | "provider_upstream_failed"
        ),
        "unexpected url_fetch reason code {}",
        url_fetch_error.reason_code
    );

    for mode in ["structured", "images", "video"] {
        let error = execute_web_search_turn_with_dependencies(
            make_turn_input(trace_id, created_at_ms, "unsupported mode route check"),
            make_search_assist(trace_id, created_at_ms.saturating_add(1), true),
            make_tool_request(
                trace_id,
                created_at_ms.saturating_add(2),
                mode,
                "unsupported mode route check",
                "medium",
            ),
            "policy-snapshot-runtime-v1".to_string(),
            &mut deps,
        )
        .expect_err("unsupported mode should fail closed");
        assert_eq!(error.reason_code, "policy_violation");
        assert_eq!(
            error
                .transitions
                .last()
                .map(|transition| transition.to.as_str())
                .unwrap_or(""),
            "TURN_FAILED_CLOSED"
        );
    }

    let _ = join.join();
}

#[test]
fn test_t5_parallel_service_trace_invoked_on_web_execution() {
    let trace = Arc::new(RuntimeServiceTrace::default());
    let created_at_ms = 1_703_200_000_000_i64;
    let trace_id = "trace-runtime-parallel-hook";
    let query = "parallel hook invocation check";

    let (base, join) = spawn_server(
        move |method, path, base_url| {
            if method == "GET" && path.starts_with("/res/v1/web/search") {
                MockResponse::json(
                    200,
                    json!({
                        "web": {
                            "results": [
                                {
                                    "title": "Parallel Hook Source A",
                                    "url": format!("{}/page/parallel-a", base_url),
                                    "description": "Source A for runtime parallel hook trace."
                                },
                                {
                                    "title": "Parallel Hook Source B",
                                    "url": format!("{}/page/parallel-b", base_url),
                                    "description": "Source B for runtime parallel hook trace."
                                }
                            ]
                        }
                    }),
                )
            } else if method == "GET"
                && (path.starts_with("/page/parallel-a") || path.starts_with("/page/parallel-b"))
            {
                MockResponse::html(
                    200,
                    "<html><body><p>parallel hook deterministic content</p></body></html>",
                )
            } else {
                MockResponse::json(500, json!({"error": "unexpected path"}))
            }
        },
        8,
    );

    let mut deps = runtime_deps_for_base(&base);
    deps.service_trace = Some(trace.clone());

    execute_web_search_turn_with_dependencies(
        make_turn_input(trace_id, created_at_ms, query),
        make_search_assist(trace_id, created_at_ms.saturating_add(1), true),
        make_tool_request(
            trace_id,
            created_at_ms.saturating_add(2),
            "web",
            query,
            "medium",
        ),
        "policy-snapshot-runtime-v1".to_string(),
        &mut deps,
    )
    .expect("web runtime should succeed");

    assert!(
        trace.parallel_plan_calls() >= 1,
        "parallel plan trace counter should increment"
    );

    let _ = join.join();
}

#[test]
fn test_t6_learn_observer_trace_invoked_on_fail_closed_path() {
    let trace = Arc::new(RuntimeServiceTrace::default());
    let trace_id = "trace-runtime-learn-hook";
    let created_at_ms = 1_703_200_000_100_i64;
    let query = "learn hook fail-closed check";

    let (base, _join) = spawn_server(
        move |_, _, _| MockResponse::json(500, json!({"error": "unused"})),
        1,
    );
    let mut deps = runtime_deps_for_base(&base);
    deps.learn_observation_enabled = true;
    deps.service_trace = Some(trace.clone());

    let error = execute_web_search_turn_with_dependencies(
        make_turn_input(trace_id, created_at_ms, query),
        make_search_assist(trace_id, created_at_ms.saturating_add(1), false),
        make_tool_request(
            trace_id,
            created_at_ms.saturating_add(2),
            "web",
            query,
            "medium",
        ),
        "policy-snapshot-runtime-v1".to_string(),
        &mut deps,
    )
    .expect_err("search_required=false should fail closed");

    assert_eq!(error.reason_code, "insufficient_evidence");
    assert!(
        trace.learn_observe_calls() >= 1,
        "learn observation trace counter should increment on fail-closed path"
    );
}
