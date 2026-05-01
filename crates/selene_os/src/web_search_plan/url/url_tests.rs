#![forbid(unsafe_code)]

use crate::web_search_plan::packet_validator::{validate_packet, validate_packet_schema_registry};
use crate::web_search_plan::proxy::ProxyMode;
use crate::web_search_plan::registry_loader::load_packet_schema_registry;
use crate::web_search_plan::url::canonical::canonicalize_url;
use crate::web_search_plan::url::extract::extract_document;
use crate::web_search_plan::url::fetch_url_to_evidence_packet;
use crate::web_search_plan::url::mime::AllowedMime;
use crate::web_search_plan::url::{
    UrlFetchErrorKind, UrlFetchFixture, UrlFetchFixtureResponse, UrlFetchPolicy, UrlFetchRequest,
    STAGE3_MAX_EVIDENCE_CHUNKS_PER_SOURCE, STAGE3_MAX_EVIDENCE_EXCERPT_CHARS,
    STAGE3_MAX_TRACE_PREVIEW_CHARS,
};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::env;
use std::io::Write;
use std::sync::{Mutex, OnceLock};

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
        test_fixture: None,
    }
}

fn fixture_request(url: &str, fixture: UrlFetchFixture) -> UrlFetchRequest {
    let mut request = base_request(url);
    request.test_fixture = Some(fixture);
    request
}

fn html_response(body: &[u8]) -> UrlFetchFixtureResponse {
    UrlFetchFixtureResponse::new(200, body.to_vec()).with_header("Content-Type", "text/html")
}

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn with_url_fetch_enabled_env<F: FnOnce()>(body: F) {
    let _guard = env_lock().lock().expect("env lock should not be poisoned");
    let keys = [
        "SELENE_SEARCH_PROVIDERS_ENABLED",
        "SELENE_URL_FETCH_ENABLED",
        "SELENE_PROVIDER_CALL_MAX_PER_TURN",
    ];
    let previous = keys
        .iter()
        .map(|key| (*key, env::var(key).ok()))
        .collect::<Vec<(&str, Option<String>)>>();
    env::set_var("SELENE_SEARCH_PROVIDERS_ENABLED", "1");
    env::set_var("SELENE_URL_FETCH_ENABLED", "1");
    env::set_var("SELENE_PROVIDER_CALL_MAX_PER_TURN", "1");
    body();
    for (key, value) in previous {
        match value {
            Some(existing) => env::set_var(key, existing),
            None => env::remove_var(key),
        }
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
fn stage3_url_admission_blocks_private_internal_and_credential_urls() {
    for (url, expected) in [
        (
            "http://127.0.0.1/private",
            UrlFetchErrorKind::PrivateUrlBlocked,
        ),
        (
            "http://localhost/private",
            UrlFetchErrorKind::PrivateUrlBlocked,
        ),
        ("http://[::1]/private", UrlFetchErrorKind::PrivateUrlBlocked),
        (
            "http://10.1.2.3/private",
            UrlFetchErrorKind::PrivateUrlBlocked,
        ),
        (
            "http://169.254.169.254/latest",
            UrlFetchErrorKind::PrivateUrlBlocked,
        ),
        (
            "https://metadata.google.internal/compute",
            UrlFetchErrorKind::PrivateUrlBlocked,
        ),
        (
            "file:///tmp/not-allowed",
            UrlFetchErrorKind::UnsupportedScheme,
        ),
        (
            "https://user:pass@fixture.stage3.test/secret",
            UrlFetchErrorKind::UnsafeUrlBlocked,
        ),
    ] {
        let req = fixture_request(
            url,
            UrlFetchFixture::single(html_response(
                b"<html><body>This fixture must never be read.</body></html>",
            )),
        );
        let err = fetch_url_to_evidence_packet(&req).expect_err("unsafe URL must be blocked");
        assert_eq!(err.error_kind, expected, "url={url}");
        assert_eq!(err.audit.url_fetch_attempt_count, 0, "url={url}");
        assert_eq!(err.audit.url_fetch_network_dispatch_count, 0, "url={url}");
    }
}

#[test]
fn stage3_live_external_fetch_fails_closed_without_dns_private_ip_proof() {
    with_url_fetch_enabled_env(|| {
        let err = fetch_url_to_evidence_packet(&base_request(
            "https://fixture.stage3.test/live-disabled",
        ))
        .expect_err("live external fetch must stay disabled without DNS validation proof");
        assert_eq!(
            err.error_kind,
            UrlFetchErrorKind::DnsPrivateValidationUnavailable
        );
        assert_eq!(err.audit.url_fetch_attempt_count, 0);
        assert_eq!(err.audit.url_fetch_network_dispatch_count, 0);
        assert_eq!(
            err.evidence_packet
                .pointer("/trust_metadata/stage3_page_fetch/page_fetch_blocked_reason")
                .and_then(|value| value.as_str()),
            Some("live_external_dns_validation_unavailable")
        );
    });
}

#[test]
fn stage3_redirect_to_private_target_is_blocked_before_network() {
    let req = fixture_request(
        "https://fixture.stage3.test/start",
        UrlFetchFixture::chain(vec![UrlFetchFixtureResponse::new(302, Vec::new())
            .with_header("Location", "https://127.0.0.1/private")]),
    );
    let err = fetch_url_to_evidence_packet(&req).expect_err("redirect to private URL must block");
    assert_eq!(err.error_kind, UrlFetchErrorKind::PrivateUrlBlocked);
    assert_eq!(err.audit.url_fetch_network_dispatch_count, 0);
}

#[test]
fn stage3_fixture_fetch_extracts_evidence_with_hard_limits_and_no_network() {
    let long_text = "Fictional entity evidence paragraph with claim terms. ".repeat(200);
    let html = format!(
        "<html><head><title>Fixture Evidence</title><script>secret script text</script><style>hidden style text</style></head><body><nav>navigation junk</nav><main>{}</main><footer>footer junk</footer></body></html>",
        long_text
    );
    let mut req = fixture_request(
        "https://fixture.stage3.test/evidence-limits",
        UrlFetchFixture::single(html_response(html.as_bytes())),
    );
    req.policy.min_text_length = 30;
    let success = fetch_url_to_evidence_packet(&req).expect("fixture fetch should succeed");
    assert_eq!(success.audit.url_fetch_attempt_count, 1);
    assert_eq!(success.audit.url_fetch_network_dispatch_count, 0);
    assert_eq!(success.audit.url_fetch_fake_fixture_dispatch_count, 1);
    assert!(!success.body_text.contains("secret script text"));
    assert!(!success.body_text.contains("hidden style text"));
    assert!(!success.body_text.contains("navigation junk"));
    assert!(!success.body_text.contains("footer junk"));

    let stage3 = success
        .evidence_packet
        .pointer("/trust_metadata/stage3_page_fetch")
        .expect("stage3 fetch metadata must exist");
    assert_eq!(stage3["raw_page_stored"], false);
    assert_eq!(
        stage3["max_excerpt_chars"].as_u64(),
        Some(STAGE3_MAX_EVIDENCE_EXCERPT_CHARS as u64)
    );
    assert_eq!(
        stage3["max_evidence_chunks_per_source"].as_u64(),
        Some(STAGE3_MAX_EVIDENCE_CHUNKS_PER_SOURCE as u64)
    );
    assert_eq!(
        stage3["max_trace_preview_chars"].as_u64(),
        Some(STAGE3_MAX_TRACE_PREVIEW_CHARS as u64)
    );

    let chunks = success
        .evidence_packet
        .get("content_chunks")
        .and_then(|value| value.as_array())
        .expect("content chunks must exist");
    assert!(!chunks.is_empty());
    assert!(chunks.len() <= STAGE3_MAX_EVIDENCE_CHUNKS_PER_SOURCE);
    for chunk in chunks {
        let excerpt = chunk
            .get("text_excerpt")
            .and_then(|value| value.as_str())
            .expect("chunk excerpt must exist");
        assert!(excerpt.chars().count() <= STAGE3_MAX_EVIDENCE_EXCERPT_CHARS);
    }

    let extraction = success
        .evidence_packet
        .pointer("/trust_metadata/stage3_evidence_extraction")
        .expect("stage3 extraction metadata must exist");
    let preview = extraction["trace_preview"]
        .as_str()
        .expect("trace preview must exist");
    assert!(preview.chars().count() <= STAGE3_MAX_TRACE_PREVIEW_CHARS);
    assert_ne!(preview, success.body_text);
    assert_eq!(extraction["full_extracted_text_in_trace"], false);
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
    let req = fixture_request(
        "https://fixture.stage3.test/a",
        UrlFetchFixture::chain(vec![
            UrlFetchFixtureResponse::new(302, Vec::new()).with_header("Location", "/b"),
            UrlFetchFixtureResponse::new(302, Vec::new()).with_header("Location", "/a"),
        ]),
    );
    let err = fetch_url_to_evidence_packet(&req).expect_err("redirect loop must fail");
    assert_eq!(err.error_kind, UrlFetchErrorKind::RedirectLoopDetected);
}

#[test]
fn test_mime_allowlist_enforcement() {
    let req = fixture_request(
        "https://fixture.stage3.test/binary",
        UrlFetchFixture::single(
            UrlFetchFixtureResponse::new(200, vec![0, 1, 2, 3, 4])
                .with_header("Content-Type", "application/octet-stream"),
        ),
    );
    let err = fetch_url_to_evidence_packet(&req).expect_err("binary mime should fail");
    assert_eq!(err.error_kind, UrlFetchErrorKind::MimeNotAllowed);
}

#[test]
fn test_streaming_response_cap_abort() {
    let body = vec![b'a'; 512];
    let mut req = fixture_request(
        "https://fixture.stage3.test/large",
        UrlFetchFixture::single(
            UrlFetchFixtureResponse::new(200, body).with_header("Content-Type", "text/plain"),
        ),
    );
    req.policy.max_response_bytes = 64;
    let err = fetch_url_to_evidence_packet(&req).expect_err("response cap should fail");
    assert_eq!(err.error_kind, UrlFetchErrorKind::ResponseTooLarge);
}

#[test]
fn test_decompression_cap_abort() {
    let text = "lorem ipsum dolor sit amet ".repeat(2000);
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(text.as_bytes())
        .expect("gzip write should succeed");
    let compressed = encoder.finish().expect("gzip finish should succeed");

    let mut req = fixture_request(
        "https://fixture.stage3.test/gzip",
        UrlFetchFixture::single(
            UrlFetchFixtureResponse::new(200, compressed)
                .with_header("Content-Type", "text/plain")
                .with_header("Content-Encoding", "gzip"),
        ),
    );
    req.policy.max_decompressed_bytes = 256;
    req.policy.max_extracted_chars = 20_000;
    req.policy.min_text_length = 5;
    let err = fetch_url_to_evidence_packet(&req).expect_err("decompression cap should fail");
    assert_eq!(err.error_kind, UrlFetchErrorKind::DecompressedTooLarge);
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

    let mut req = fixture_request(
        "https://fixture.stage3.test/charset",
        UrlFetchFixture::single(html_response(&html_iso_bytes)),
    );
    req.policy.min_text_length = 10;

    let first = fetch_url_to_evidence_packet(&req).expect("first fetch should pass");
    let second = fetch_url_to_evidence_packet(&req).expect("second fetch should pass");

    assert_eq!(first.body_text, second.body_text);
    assert!(first.body_text.contains("Café"));
}

#[test]
fn test_extraction_quality_low_fail() {
    let body = b"<html><body>ok</body></html>".to_vec();
    let mut req = fixture_request(
        "https://fixture.stage3.test/short",
        UrlFetchFixture::single(html_response(&body)),
    );
    req.policy.min_text_length = 25;
    let err = fetch_url_to_evidence_packet(&req).expect_err("quality gate must fail");
    assert_eq!(err.error_kind, UrlFetchErrorKind::ExtractionQualityLow);
}

#[test]
fn test_evidence_packet_completeness() {
    let html = b"<html><head><title>Deterministic Title</title></head><body>This is deterministic body text with enough content for quality gate and extraction checks.</body></html>".to_vec();
    let mut req = fixture_request(
        "https://fixture.stage3.test/evidence",
        UrlFetchFixture::single(html_response(&html)),
    );
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
    let mut req = fixture_request(
        "https://fixture.stage3.test/html",
        UrlFetchFixture::single(html_response(&html)),
    );
    req.policy.min_text_length = 10;
    let out = fetch_url_to_evidence_packet(&req).expect("fetch should succeed");
    assert!(out.body_text.len() > 10);
}
