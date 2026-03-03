#![forbid(unsafe_code)]

use crate::web_search_plan::chunk::build_hashed_chunks_with_custom_hasher;
use crate::web_search_plan::chunk::chunker::{chunk_document, ChunkPolicy};
use crate::web_search_plan::chunk::hasher::{derive_chunk_id, ChunkHasher, Sha256ChunkHasher};
use crate::web_search_plan::chunk::normalize::normalize_document_for_chunking;
use crate::web_search_plan::chunk::{
    build_hashed_chunks_for_document, to_text_chunks, ChunkBuildError, EVIDENCE_TRUNCATED_REASON_CODE,
};
use crate::web_search_plan::packet_validator::{validate_packet, validate_packet_schema_registry};
use crate::web_search_plan::registry_loader::load_packet_schema_registry;
use crate::web_search_plan::url::fetch_url_to_evidence_packet;
use crate::web_search_plan::url::UrlFetchRequest;
use crate::web_search_plan::proxy::ProxyMode;
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
    let addr = format!("http://{}", listener.local_addr().expect("local addr exists"));
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

fn base_url_fetch_request(url: &str) -> UrlFetchRequest {
    UrlFetchRequest::new(
        "trace-run4",
        "chunk foundation check",
        url,
        1_700_000_100_000,
        "PH1.E",
        vec!["PH1.D".to_string(), "PH1.WRITE".to_string(), "PH1.J".to_string()],
        ProxyMode::Off,
    )
}

#[test]
fn test_normalization_stability_and_idempotence() {
    let input = "  line 1\r\n\r\n[[EXTRACTOR_START]] line   2\t\n\nline 3  [[EXTRACTOR_END]] ";
    let normalized_once = normalize_document_for_chunking(input);
    let normalized_twice = normalize_document_for_chunking(&normalized_once);
    assert_eq!(normalized_once, normalized_twice);
    assert_eq!(normalized_once, "line 1\n\nline 2\n\nline 3");
}

#[test]
fn test_identical_document_has_stable_chunk_id_order() {
    let text = "Paragraph one has enough content to become a stable chunk for deterministic tests.\n\nParagraph two remains deterministic and includes additional words to exceed minimum chunk length.";
    let policy = ChunkPolicy {
        max_chunk_chars: 120,
        min_chunk_chars: 30,
        max_chunks_per_document: 10,
    };

    let first = build_hashed_chunks_for_document("https://example.com/a", "https://example.com/a", text, policy)
        .expect("first chunk build should pass");
    let second = build_hashed_chunks_for_document("https://example.com/a", "https://example.com/a", text, policy)
        .expect("second chunk build should pass");

    let first_ids: Vec<String> = first.chunks.iter().map(|chunk| chunk.chunk_id.clone()).collect();
    let second_ids: Vec<String> = second.chunks.iter().map(|chunk| chunk.chunk_id.clone()).collect();
    assert_eq!(first_ids, second_ids);
}

#[test]
fn test_truncation_determinism() {
    let text = (0..20)
        .map(|idx| format!("Paragraph {} with deterministic text for truncation behavior.", idx))
        .collect::<Vec<String>>()
        .join("\n\n");

    let policy = ChunkPolicy {
        max_chunk_chars: 80,
        min_chunk_chars: 20,
        max_chunks_per_document: 3,
    };

    let first = build_hashed_chunks_for_document("https://example.com/t", "https://example.com/t", &text, policy)
        .expect("first chunk build should pass");
    let second = build_hashed_chunks_for_document("https://example.com/t", "https://example.com/t", &text, policy)
        .expect("second chunk build should pass");

    assert!(first.truncated);
    assert!(second.truncated);
    assert_eq!(first.reason_codes, vec![EVIDENCE_TRUNCATED_REASON_CODE]);
    assert_eq!(first.chunks.len(), 3);
    let first_ids: Vec<String> = first.chunks.iter().map(|chunk| chunk.chunk_id.clone()).collect();
    let second_ids: Vec<String> = second.chunks.iter().map(|chunk| chunk.chunk_id.clone()).collect();
    assert_eq!(first_ids, second_ids);
}

#[derive(Debug, Default)]
struct ConstantHasher;

impl ChunkHasher for ConstantHasher {
    fn hash_hex(&self, _input: &[u8]) -> String {
        "collision-fixed-id".to_string()
    }
}

#[test]
fn test_collision_detection_fails_closed() {
    let text = "One paragraph with substantial text.\n\nAnother paragraph with different text to force collision check.";
    let policy = ChunkPolicy {
        max_chunk_chars: 60,
        min_chunk_chars: 10,
        max_chunks_per_document: 10,
    };

    let err = build_hashed_chunks_with_custom_hasher(
        "https://example.com/collision",
        "https://example.com/collision",
        text,
        policy,
        &ConstantHasher,
    )
    .expect_err("forced hash collisions must fail closed");

    match err {
        ChunkBuildError::HashCollisionDetected { chunk_id, .. } => {
            assert_eq!(chunk_id, "collision-fixed-id");
        }
        other => panic!("unexpected error variant: {:?}", other),
    }
}

#[test]
fn test_replay_stability_across_runs() {
    let text = "Replay paragraph one with deterministic content and punctuation.\n\nReplay paragraph two with deterministic content and punctuation.";
    let policy = ChunkPolicy::default();

    let mut previous_ids = Vec::new();
    for _ in 0..3 {
        let output = build_hashed_chunks_for_document(
            "https://example.com/replay",
            "https://example.com/replay",
            text,
            policy,
        )
        .expect("chunk build should pass");
        let ids: Vec<String> = output.chunks.iter().map(|chunk| chunk.chunk_id.clone()).collect();
        if previous_ids.is_empty() {
            previous_ids = ids;
        } else {
            assert_eq!(previous_ids, ids);
        }
    }
}

#[test]
fn test_chunk_id_uses_pinned_materialization() {
    let normalized = normalize_document_for_chunking("A deterministic paragraph.");
    let doc = chunk_document(&normalized, ChunkPolicy::default());
    let first_chunk = doc.chunks.first().expect("at least one chunk");
    let derived = derive_chunk_id("https://example.com/hash", first_chunk, &Sha256ChunkHasher);
    let derived_again = derive_chunk_id("https://example.com/hash", first_chunk, &Sha256ChunkHasher);
    assert_eq!(derived, derived_again);
}

#[test]
fn test_evidence_packet_contains_chunk_fields() {
    let html = b"<html><head><title>Chunk Test</title></head><body>Paragraph one has deterministic content for chunk fields validation.\n\nParagraph two continues to ensure multiple chunks can appear with stable ids and citations.</body></html>".to_vec();
    let (base, join) = spawn_server(
        move |_| MockResponse::new(200, html.clone()).with_header("Content-Type", "text/html"),
        2,
    );

    let mut req = base_url_fetch_request(&base);
    req.policy.min_text_length = 20;
    let result = fetch_url_to_evidence_packet(&req).expect("fetch should succeed");

    let packet_registry = load_packet_schema_registry().expect("packet schema should load");
    validate_packet_schema_registry(&packet_registry).expect("packet schema should validate");
    validate_packet("EvidencePacket", &result.evidence_packet, &packet_registry)
        .expect("evidence packet shape should remain valid");

    let chunks = result
        .evidence_packet
        .get("content_chunks")
        .and_then(|v| v.as_array())
        .expect("content_chunks must be array");
    assert!(!chunks.is_empty());

    for chunk in chunks {
        assert!(chunk.get("chunk_id").and_then(|v| v.as_str()).is_some());
        assert!(chunk.get("hash_version").and_then(|v| v.as_str()).is_some());
        assert!(chunk.get("norm_version").and_then(|v| v.as_str()).is_some());
        assert!(chunk.get("chunk_version").and_then(|v| v.as_str()).is_some());
        assert!(chunk.get("source_url").and_then(|v| v.as_str()).is_some());
        assert!(chunk.get("canonical_url").and_then(|v| v.as_str()).is_some());
        assert!(chunk.get("chunk_index").and_then(|v| v.as_u64()).is_some());
        assert!(chunk.get("text_excerpt").and_then(|v| v.as_str()).is_some());
        assert!(chunk.get("text_len_chars").and_then(|v| v.as_u64()).is_some());
        let citation = chunk.get("citation").expect("citation field exists");
        assert!(citation.get("chunk_id").and_then(|v| v.as_str()).is_some());
        assert!(citation.get("source_url").and_then(|v| v.as_str()).is_some());
    }

    let _ = join.join();
}

#[test]
fn test_to_text_chunks_round_trip() {
    let output = build_hashed_chunks_for_document(
        "https://example.com/round",
        "https://example.com/round",
        "Round trip paragraph for text chunk conversion path.",
        ChunkPolicy::default(),
    )
    .expect("chunk build should pass");
    let round_trip = to_text_chunks(&output);
    assert_eq!(round_trip.len(), output.chunks.len());
}
