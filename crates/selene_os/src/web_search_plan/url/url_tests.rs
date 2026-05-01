#![forbid(unsafe_code)]

use crate::web_search_plan::packet_validator::{validate_packet, validate_packet_schema_registry};
use crate::web_search_plan::proxy::ProxyMode;
use crate::web_search_plan::registry_loader::load_packet_schema_registry;
use crate::web_search_plan::url::canonical::canonicalize_url;
use crate::web_search_plan::url::extract::extract_document;
use crate::web_search_plan::url::fetch_url_to_evidence_packet;
use crate::web_search_plan::url::mime::AllowedMime;
use crate::web_search_plan::url::{UrlFetchErrorKind, UrlFetchPolicy, UrlFetchRequest};
use flate2::write::GzEncoder;
use flate2::Compression;
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
    fn new(status: u16, body: Vec<u8>) -> Self {
        Self {
            status,
            headers: BTreeMap::new(),
            body,
        }
    }

    fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }
}

fn spawn_server<F>(handler: F, max_requests: usize) -> (String, thread::JoinHandle<()>)
where
    F: Fn(&str) -> MockResponse + Send + Sync + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").expect("test server bind should succeed");
    listener
        .set_nonblocking(true)
        .expect("nonblocking setup should succeed");
    let addr = format!(
        "http://{}",
        listener.local_addr().expect("local addr exists")
    );
    let handler = Arc::new(handler);

    let join = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut served = 0usize;
        while served < max_requests && Instant::now() < deadline {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let path = read_request_path(&mut stream);
                    let response = handler(&path);
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

    (addr, join)
}

fn read_request_path(stream: &mut TcpStream) -> String {
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

    first_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("/")
        .to_string()
}

fn write_http_response(stream: &mut TcpStream, response: &MockResponse) {
    let status_text = match response.status {
        200 => "OK",
        301 => "Moved Permanently",
        302 => "Found",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        404 => "Not Found",
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

fn base_request(url: &str) -> UrlFetchRequest {
    UrlFetchRequest {
        trace_id: "trace-url-1".to_string(),
        query: "fetch this page".to_string(),
        requested_url: url.to_string(),
        importance_tier: "medium".to_string(),
        url_open_ordinal: 0,
        url_open_cap: None,
        created_at_ms: 1_700_000_000_000,
        retrieved_at_ms: 1_700_000_000_500,
        produced_by: "PH1.E".to_string(),
        intended_consumers: vec![
            "PH1.D".to_string(),
            "PH1.WRITE".to_string(),
            "PH1.J".to_string(),
        ],
        proxy_config: crate::web_search_plan::proxy::proxy_config::ProxyConfig {
            mode: ProxyMode::Off,
            http_proxy_url: None,
            https_proxy_url: None,
        },
        policy: UrlFetchPolicy::default(),
    }
}

#[test]
fn stage2_url_fetch_global_off_blocks_external_before_network() {
    let err = fetch_url_to_evidence_packet(&base_request(
        "https://synthetic-acorn-delta.invalid/report",
    ))
    .expect_err("default provider-off policy must block external URL fetch");

    assert_eq!(err.error_kind, UrlFetchErrorKind::ProviderDisabled);
    assert_eq!(err.reason_code, "provider_disabled");
    assert!(err.message.contains("stage2_provider_control=1"));
    assert!(err.message.contains("provider_call_attempt_count=0"));
    assert!(err.message.contains("provider_network_dispatch_count=0"));
    assert!(err.message.contains("billable_class=BLOCKED_NOT_BILLABLE"));
    assert!(err.message.contains("billing_scope=NON_BILLABLE"));
}

#[test]
fn test_canonicalization_determinism() {
    let canonical =
        canonicalize_url("HTTPS://Example.COM:443/path/?utm_source=a&gclid=abc&keep=1#section")
            .expect("canonicalization should succeed");
    assert_eq!(canonical.canonical_url, "https://example.com/path?keep=1");

    let again = canonicalize_url(&canonical.canonical_url).expect("idempotent canonicalization");
    assert_eq!(canonical.canonical_url, again.canonical_url);
}

#[test]
fn test_redirect_loop_detection() {
    let (base, join) = spawn_server(
        |path| match path {
            "/a" => MockResponse::new(302, Vec::new()).with_header("Location", "/b"),
            "/b" => MockResponse::new(302, Vec::new()).with_header("Location", "/a"),
            _ => MockResponse::new(404, b"missing".to_vec()),
        },
        8,
    );

    let req = base_request(&format!("{}/a", base));
    let err = fetch_url_to_evidence_packet(&req).expect_err("redirect loop must fail");
    assert_eq!(err.error_kind, UrlFetchErrorKind::RedirectLoopDetected);

    let _ = join.join();
}

#[test]
fn test_mime_allowlist_enforcement() {
    let (base, join) = spawn_server(
        |_| {
            MockResponse::new(200, vec![0, 1, 2, 3, 4])
                .with_header("Content-Type", "application/octet-stream")
        },
        2,
    );

    let req = base_request(&base);
    let err = fetch_url_to_evidence_packet(&req).expect_err("binary mime should fail");
    assert_eq!(err.error_kind, UrlFetchErrorKind::MimeNotAllowed);

    let _ = join.join();
}

#[test]
fn test_streaming_response_cap_abort() {
    let body = vec![b'a'; 512];
    let (base, join) = spawn_server(
        move |_| MockResponse::new(200, body.clone()).with_header("Content-Type", "text/plain"),
        2,
    );

    let mut req = base_request(&base);
    req.policy.max_response_bytes = 64;
    let err = fetch_url_to_evidence_packet(&req).expect_err("response cap should fail");
    assert_eq!(err.error_kind, UrlFetchErrorKind::ResponseTooLarge);

    let _ = join.join();
}

#[test]
fn test_decompression_cap_abort() {
    let text = "lorem ipsum dolor sit amet ".repeat(2000);
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(text.as_bytes())
        .expect("gzip write should succeed");
    let compressed = encoder.finish().expect("gzip finish should succeed");

    let (base, join) = spawn_server(
        move |_| {
            MockResponse::new(200, compressed.clone())
                .with_header("Content-Type", "text/plain")
                .with_header("Content-Encoding", "gzip")
        },
        2,
    );

    let mut req = base_request(&base);
    req.policy.max_decompressed_bytes = 256;
    req.policy.max_extracted_chars = 20_000;
    req.policy.min_text_length = 5;
    let err = fetch_url_to_evidence_packet(&req).expect_err("decompression cap should fail");
    assert_eq!(err.error_kind, UrlFetchErrorKind::DecompressedTooLarge);

    let _ = join.join();
}

#[test]
fn test_charset_normalization_stability() {
    let html_iso_bytes: Vec<u8> = vec![
        0x3c, 0x68, 0x74, 0x6d, 0x6c, 0x3e, 0x3c, 0x68, 0x65, 0x61, 0x64, 0x3e, 0x3c, 0x6d, 0x65,
        0x74, 0x61, 0x20, 0x63, 0x68, 0x61, 0x72, 0x73, 0x65, 0x74, 0x3d, 0x22, 0x69, 0x73, 0x6f,
        0x2d, 0x38, 0x38, 0x35, 0x39, 0x2d, 0x31, 0x22, 0x3e, 0x3c, 0x74, 0x69, 0x74, 0x6c, 0x65,
        0x3e, 0x43, 0x61, 0x66, 0xe9, 0x3c, 0x2f, 0x74, 0x69, 0x74, 0x6c, 0x65, 0x3e, 0x3c, 0x2f,
        0x68, 0x65, 0x61, 0x64, 0x3e, 0x3c, 0x62, 0x6f, 0x64, 0x79, 0x3e, 0x43, 0x61, 0x66, 0xe9,
        0x20, 0x74, 0x65, 0x78, 0x74, 0x20, 0x77, 0x69, 0x74, 0x68, 0x20, 0x73, 0x74, 0x61, 0x62,
        0x6c, 0x65, 0x20, 0x6e, 0x6f, 0x72, 0x6d, 0x61, 0x6c, 0x69, 0x7a, 0x61, 0x74, 0x69, 0x6f,
        0x6e, 0x2e, 0x3c, 0x2f, 0x62, 0x6f, 0x64, 0x79, 0x3e, 0x3c, 0x2f, 0x68, 0x74, 0x6d, 0x6c,
        0x3e,
    ];

    let (base, join) = spawn_server(
        move |_| {
            MockResponse::new(200, html_iso_bytes.clone()).with_header("Content-Type", "text/html")
        },
        3,
    );

    let mut req = base_request(&base);
    req.policy.min_text_length = 10;

    let first = fetch_url_to_evidence_packet(&req).expect("first fetch should pass");
    let second = fetch_url_to_evidence_packet(&req).expect("second fetch should pass");

    assert_eq!(first.body_text, second.body_text);
    assert!(first.body_text.contains("Café"));

    let _ = join.join();
}

#[test]
fn test_extraction_quality_low_fail() {
    let body = b"<html><body>ok</body></html>".to_vec();
    let (base, join) = spawn_server(
        move |_| MockResponse::new(200, body.clone()).with_header("Content-Type", "text/html"),
        2,
    );

    let mut req = base_request(&base);
    req.policy.min_text_length = 25;
    let err = fetch_url_to_evidence_packet(&req).expect_err("quality gate must fail");
    assert_eq!(err.error_kind, UrlFetchErrorKind::ExtractionQualityLow);

    let _ = join.join();
}

#[test]
fn test_evidence_packet_completeness() {
    let html = b"<html><head><title>Deterministic Title</title></head><body>This is deterministic body text with enough content for quality gate and extraction checks.</body></html>".to_vec();
    let (base, join) = spawn_server(
        move |_| MockResponse::new(200, html.clone()).with_header("Content-Type", "text/html"),
        2,
    );

    let mut req = base_request(&base);
    req.policy.min_text_length = 30;
    let success = fetch_url_to_evidence_packet(&req).expect("fetch should succeed");

    let packet_registry = load_packet_schema_registry().expect("packet schema should load");
    validate_packet_schema_registry(&packet_registry).expect("packet schema should be valid");
    validate_packet("EvidencePacket", &success.evidence_packet, &packet_registry)
        .expect("evidence packet should satisfy run1 schema");

    let provider_runs = success
        .evidence_packet
        .get("provider_runs")
        .and_then(|v| v.as_array())
        .expect("provider_runs should be array");
    assert_eq!(provider_runs.len(), 1);
    assert_eq!(provider_runs[0]["endpoint"], "url_fetch");

    let _ = join.join();
}

#[test]
fn test_html_extraction_produces_text() {
    let html = "<html><head><title>Deterministic Title</title></head><body>This is deterministic body text with enough content for quality gate and extraction checks.</body></html>";
    let extracted =
        extract_document(AllowedMime::Html, html, 200_000).expect("extraction should succeed");
    assert!(extracted.extraction_chars > 30);
    assert!(extracted.body_text.contains("deterministic"));
}

#[test]
fn test_fetch_html_returns_non_empty_body() {
    let html = b"<html><head><title>T</title></head><body>Body text that should survive extraction and quality.</body></html>".to_vec();
    let (base, join) = spawn_server(
        move |_| MockResponse::new(200, html.clone()).with_header("Content-Type", "text/html"),
        2,
    );

    let mut req = base_request(&base);
    req.policy.min_text_length = 10;
    let out = fetch_url_to_evidence_packet(&req).expect("fetch should succeed");
    assert!(out.body_text.len() > 10);

    let _ = join.join();
}
