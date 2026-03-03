#![forbid(unsafe_code)]

use crate::web_search_plan::document::filing::{financials_like, patent_like, sec_like};
use crate::web_search_plan::document::ocr::OcrOptions;
use crate::web_search_plan::document::pdf_tables::extract_table_rows;
use crate::web_search_plan::document::pdf_text::extract_text_from_pdf;
use crate::web_search_plan::document::{
    execute_document_pipeline_from_tool_request, DocumentErrorKind, DocumentRuntimeConfig,
};
use crate::web_search_plan::proxy::ProxyMode;
use crate::web_search_plan::structured::normalize::sort_rows_deterministically;
use crate::web_search_plan::structured::types::{
    StructuredExtraction, StructuredRow, StructuredValue, STRUCTURED_SCHEMA_VERSION,
};
use crate::web_search_plan::structured::validator::validate_extraction;
use serde_json::{json, Value};
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/web_search_plan/document_fixtures")
}

fn read_fixture_bytes(file_name: &str) -> Vec<u8> {
    fs::read(fixture_dir().join(file_name)).expect("fixture bytes should load")
}

fn read_fixture_json(file_name: &str) -> Value {
    let text = fs::read_to_string(fixture_dir().join(file_name)).expect("fixture json should load");
    serde_json::from_str(&text).expect("fixture json should parse")
}

#[test]
fn test_t1_text_pdf_extraction_deterministic() {
    let bytes = read_fixture_bytes("sample_text_pdf.pdf");
    let output = extract_text_from_pdf(&bytes, 50_000, 1_700_000_000_000)
        .expect("text extraction should pass");
    let expected = read_fixture_json("expected_text.json");

    assert_eq!(
        output.extracted_text,
        expected["normalized_text"]
            .as_str()
            .expect("expected normalized_text")
    );
    assert_eq!(output.page_count, 1);
}

#[test]
fn test_t2_table_extraction_deterministic_structured_rows() {
    let bytes = read_fixture_bytes("sample_table_pdf.pdf");
    let text = extract_text_from_pdf(&bytes, 50_000, 1_700_000_000_000)
        .expect("table extraction text should pass")
        .extracted_text;
    let mut actual = extract_table_rows(
        &text,
        "https://example.com/sample_table.pdf",
        "pdf_table_v1",
        256,
    )
    .expect("table row extraction should pass");
    sort_rows_deterministically(&mut actual);

    let expected_value = read_fixture_json("expected_tables.json");
    let mut expected: Vec<StructuredRow> =
        serde_json::from_value(expected_value).expect("expected tables should deserialize");
    sort_rows_deterministically(&mut expected);

    assert_eq!(actual, expected);
}

#[test]
fn test_t3_filing_packs_extract_deterministic_required_fields() {
    let bytes = read_fixture_bytes("sample_text_pdf.pdf");
    let text = extract_text_from_pdf(&bytes, 50_000, 1_700_000_000_000)
        .expect("text extraction should pass")
        .extracted_text;

    let mut sec_rows = sec_like::parse_rows(
        &text,
        "https://example.com/filing.pdf",
        "filing_sec_like_v1",
    )
    .expect("sec_like should parse");
    let mut financial_rows = financials_like::parse_rows(
        &text,
        "https://example.com/filing.pdf",
        "filing_financials_like_v1",
    )
    .expect("financials_like should parse");
    let mut patent_rows = patent_like::parse_rows(
        &text,
        "https://example.com/filing.pdf",
        "filing_patent_like_v1",
    )
    .expect("patent_like should parse");

    sort_rows_deterministically(&mut sec_rows);
    sort_rows_deterministically(&mut financial_rows);
    sort_rows_deterministically(&mut patent_rows);

    let mut expected_sec: Vec<StructuredRow> =
        serde_json::from_value(read_fixture_json("expected_filing_sec_like.json"))
            .expect("expected sec rows");
    let mut expected_financial: Vec<StructuredRow> =
        serde_json::from_value(read_fixture_json("expected_filing_financials_like.json"))
            .expect("expected financial rows");
    let mut expected_patent: Vec<StructuredRow> =
        serde_json::from_value(read_fixture_json("expected_filing_patent_like.json"))
            .expect("expected patent rows");

    sort_rows_deterministically(&mut expected_sec);
    sort_rows_deterministically(&mut expected_financial);
    sort_rows_deterministically(&mut expected_patent);

    assert_eq!(sec_rows, expected_sec);
    assert_eq!(financial_rows, expected_financial);
    assert_eq!(patent_rows, expected_patent);
}

#[test]
fn test_t4_ocr_missing_fail_closed_provider_unconfigured() {
    let bytes = read_fixture_bytes("sample_scanned_pdf.pdf");
    let options = OcrOptions {
        backend: None,
        language: "eng".to_string(),
        max_pages: 1,
    };
    let error = crate::web_search_plan::document::ocr::extract_text_from_pdf_with_ocr(
        &bytes, &options, 2_000,
    )
    .expect_err("missing OCR backend should fail closed");

    assert_eq!(error.kind, DocumentErrorKind::ProviderUnconfigured);
    assert_eq!(error.reason_code(), "provider_unconfigured");
}

#[test]
fn test_t5_schema_validation_rejects_malformed_rows() {
    let malformed = StructuredRow {
        entity: "Acme".to_string(),
        attribute: "margin".to_string(),
        value: StructuredValue::Percent { value: 150.0 },
        unit: None,
        as_of_ms: None,
        source_url: "https://example.com/table.pdf".to_string(),
        source_ref: "https://example.com/table.pdf#row".to_string(),
        confidence: None,
        schema_version: STRUCTURED_SCHEMA_VERSION.to_string(),
    };
    let extraction = StructuredExtraction {
        query: "table".to_string(),
        rows: vec![malformed],
        schema_id: "pdf_table_v1".to_string(),
        extracted_at_ms: 1_700_000_000_000,
        provider_runs: vec![],
        sources: vec![],
        errors: vec![],
    };
    let result = validate_extraction(&extraction);
    assert!(result.is_err());
    assert!(result
        .err()
        .unwrap_or_default()
        .contains("percent must be in [0,100]"));
}

#[test]
fn test_t6_evidence_packet_provenance_document_mode() {
    let bytes = read_fixture_bytes("sample_text_pdf.pdf");
    let (url, handle) = spawn_pdf_server(bytes, "application/pdf", 1);

    let tool_request = json!({
        "schema_version": "1.0.0",
        "produced_by": "PH1.X",
        "intended_consumers": ["PH1.E", "PH1.D"],
        "created_at_ms": 1_700_000_000_000i64,
        "trace_id": "trace-document",
        "mode": "structured",
        "query": url,
        "importance_tier": "medium",
        "budgets": {
            "document_schema": "filing_sec_like_v1"
        }
    });

    let mut config = DocumentRuntimeConfig::default();
    config.proxy_config.mode = ProxyMode::Off;
    config.ocr_options.backend = None;
    config.max_pdf_bytes = 2 * 1024 * 1024;

    let result =
        execute_document_pipeline_from_tool_request(&tool_request, 1_700_000_000_111, &config)
            .expect("document pipeline should pass");
    handle.join().expect("server thread should join");

    let provider_runs = result
        .evidence_packet
        .get("provider_runs")
        .and_then(Value::as_array)
        .expect("provider_runs must exist");
    let endpoints = provider_runs
        .iter()
        .filter_map(|entry| entry.get("endpoint").and_then(Value::as_str))
        .collect::<Vec<&str>>();

    assert!(endpoints.contains(&"document_pdf_fetch"));
    assert!(endpoints.contains(&"document_pdf_text"));
    assert!(endpoints.contains(&"document_filing_pack"));

    assert_eq!(
        result
            .evidence_packet
            .pointer("/sources/0/media_type")
            .and_then(Value::as_str),
        Some("document")
    );
    assert_eq!(
        result
            .evidence_packet
            .pointer("/trust_metadata/document/schema_id")
            .and_then(Value::as_str),
        Some("filing_sec_like_v1")
    );
    assert_eq!(
        result
            .evidence_packet
            .get("content_chunks")
            .and_then(Value::as_array)
            .map(|chunks| chunks.is_empty()),
        Some(true)
    );
}

fn spawn_pdf_server(
    body: Vec<u8>,
    content_type: &str,
    requests: usize,
) -> (String, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let address = listener.local_addr().expect("server addr");
    let content_type = content_type.to_string();

    let handle = std::thread::spawn(move || {
        for _ in 0..requests {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buffer = [0u8; 4096];
                let _ = stream.read(&mut buffer);
                let response_header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    content_type,
                    body.len()
                );
                let _ = stream.write_all(response_header.as_bytes());
                let _ = stream.write_all(&body);
                let _ = stream.flush();
            }
        }
    });

    (format!("http://{}/sample.pdf", address), handle)
}
