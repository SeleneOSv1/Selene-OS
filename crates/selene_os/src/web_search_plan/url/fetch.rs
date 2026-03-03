#![forbid(unsafe_code)]

use crate::web_search_plan::chunk::chunker::ChunkPolicy;
use crate::web_search_plan::chunk::citation::build_citation_anchors;
use crate::web_search_plan::chunk::{
    build_hashed_chunks_for_document, bounded_excerpt, ChunkBuildError,
    EVIDENCE_TRUNCATED_REASON_CODE, HASH_COLLISION_REASON_CODE,
};
use crate::web_search_plan::diag::{
    default_failed_transitions, try_build_debug_packet, DebugPacketContext, DebugStatus,
};
use crate::web_search_plan::proxy::proxy_redaction::redact_proxy_url;
use crate::web_search_plan::proxy::proxy_self_check::run_startup_self_check;
use crate::web_search_plan::proxy::{ProxyErrorKind, ProxyMode};
use crate::web_search_plan::url::canonical::{canonicalize_url, CANON_VERSION};
use crate::web_search_plan::url::charset::{
    charset_from_content_type, select_charset, CHARSET_SNIFF_LIMIT_BYTES, CHARSET_VERSION,
    NORMALIZATION_VERSION,
};
use crate::web_search_plan::url::decompress::{
    parse_content_encoding, wrap_decoder, ContentEncoding,
};
use crate::web_search_plan::url::extract::{extract_document, ExtractedDocument, EXTRACTION_VERSION};
use crate::web_search_plan::url::mime::{detect_allowed_mime, AllowedMime, MIME_SNIFF_PREFIX_BYTES};
use crate::web_search_plan::url::quality_gate::{
    evaluate_text_quality, QualityMetrics, QUALITY_GATE_VERSION,
};
use crate::web_search_plan::url::redirect::RedirectState;
use crate::web_search_plan::url::{
    UrlFetchAudit, UrlFetchErrorKind, UrlFetchFailure, UrlFetchPolicy, UrlFetchRequest,
    UrlFetchSuccess,
};
use serde_json::{json, Value};
use std::io::Read;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

const FIXED_USER_AGENT: &str = "SeleneWebSearch/1.0";
const FIXED_ACCEPT: &str = "text/html,application/xhtml+xml,text/plain,application/pdf";
const FIXED_ACCEPT_ENCODING: &str = "gzip, br, deflate";

#[derive(Debug)]
struct BodyOutcome {
    mime: AllowedMime,
    extracted: ExtractedDocument,
    quality: QualityMetrics,
    bytes_read: usize,
    bytes_decompressed: usize,
}

pub fn fetch_url_to_evidence_packet(
    request: &UrlFetchRequest,
) -> Result<UrlFetchSuccess, UrlFetchFailure> {
    let canonical = canonicalize_url(&request.requested_url)
        .map_err(|kind| build_failure_without_audit(request, kind, "failed to canonicalize url"))?;

    let mut audit = UrlFetchAudit::new(canonical.canonical_url.clone(), request.proxy_config.mode);
    let mut redirect = RedirectState::new(
        &canonical.canonical_url,
        request.policy.max_redirect_depth,
        request.policy.allow_scheme_downgrade,
    );

    let mut current_url = canonical.canonical_url.clone();
    loop {
        let step_start = Instant::now();
        let response = send_get_request(request, &current_url, &mut audit).map_err(|mut failure| {
            failure.audit.canonical_url = canonical.canonical_url.clone();
            failure
        })?;
        audit.latency_ms = audit
            .latency_ms
            .saturating_add(step_start.elapsed().as_millis() as u64);

        let status = response.status() as u16;
        audit.status_code = Some(status);

        if is_redirect_status(status) {
            let location = response
                .header("location")
                .ok_or_else(|| {
                    build_failure(
                        request,
                        &audit,
                        UrlFetchErrorKind::RedirectMissingLocation,
                        "redirect status without location header",
                    )
                })?;
            let next = redirect.resolve_next(&current_url, location).map_err(|kind| {
                build_failure(
                    request,
                    &audit,
                    kind,
                    "redirect validation failed during url fetch",
                )
            })?;
            current_url = next;
            continue;
        }

        audit.final_url = Some(current_url.clone());
        if status != 200 && !request.policy.allow_non_200 {
            return Err(build_failure(
                request,
                &audit,
                UrlFetchErrorKind::HttpNon200,
                &format!("http status {} rejected by policy", status),
            ));
        }

        let content_type_header = response.header("content-type").map(str::to_string);
        let content_encoding_header = response.header("content-encoding").map(str::to_string);

        let body = read_response_body(
            response,
            content_type_header.as_deref(),
            content_encoding_header.as_deref(),
            &request.policy,
        )
        .map_err(|kind| build_failure(request, &audit, kind, "failed while reading response body"))?;

        audit.bytes_read = body.bytes_read;
        audit.bytes_decompressed = body.bytes_decompressed;
        audit.extraction_chars = body.extracted.extraction_chars;

        let final_url = audit
            .final_url
            .clone()
            .unwrap_or_else(|| audit.canonical_url.clone());
        let chunk_output = build_hashed_chunks_for_document(
            &audit.canonical_url,
            &final_url,
            &body.extracted.body_text,
            ChunkPolicy::default(),
        )
        .map_err(|err| map_chunk_error_to_fetch_failure(request, &audit, err))?;
        let evidence_packet = build_success_evidence_packet(request, &audit, &body, &chunk_output);
        return Ok(UrlFetchSuccess {
            evidence_packet,
            title: body.extracted.title,
            body_text: body.extracted.body_text,
            media_type: body.mime.as_str().to_string(),
            audit,
        });
    }
}

fn send_get_request(
    request: &UrlFetchRequest,
    current_url: &str,
    audit: &mut UrlFetchAudit,
) -> Result<ureq::Response, UrlFetchFailure> {
    let mut builder = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_millis(request.policy.connect_timeout_ms))
        .timeout_read(Duration::from_millis(request.policy.read_timeout_ms))
        .timeout_write(Duration::from_millis(request.policy.read_timeout_ms))
        .user_agent(FIXED_USER_AGENT)
        .try_proxy_from_env(false)
        .redirects(0);

    if let Err(check) = run_startup_self_check(&request.proxy_config) {
        audit.proxy_error_kind = Some(check.error_kind.as_str().to_string());
        if check.severity.as_str() == "critical" {
            return Err(build_failure(
                request,
                audit,
                classify_proxy_error(check.error_kind),
                &check.details,
            ));
        }
    }

    let proxy_url = proxy_for_url(&request.proxy_config.mode, current_url, request)?;
    if let Some(proxy_raw) = proxy_url {
        let redacted = redact_proxy_url(proxy_raw).map_err(|kind| {
            build_failure(
                request,
                audit,
                classify_proxy_error(kind),
                "proxy URL redaction failed",
            )
        })?;
        audit.proxy_redacted_endpoint = Some(redacted);
        let proxy = ureq::Proxy::new(proxy_raw).map_err(|_| {
            build_failure(
                request,
                audit,
                UrlFetchErrorKind::ProxyMisconfigured,
                "invalid proxy URL",
            )
        })?;
        builder = builder.proxy(proxy);
    }

    let agent = builder.build();
    let req = agent
        .get(current_url)
        .set("Accept", FIXED_ACCEPT)
        .set("Accept-Encoding", FIXED_ACCEPT_ENCODING)
        .set("Cache-Control", "no-cache")
        .set("Pragma", "no-cache")
        .timeout(Duration::from_millis(request.policy.total_timeout_ms));

    match req.call() {
        Ok(resp) => Ok(resp),
        Err(ureq::Error::Status(_, resp)) => Ok(resp),
        Err(ureq::Error::Transport(transport)) => {
            let combined = format!("{:?} {}", transport.kind(), transport).to_ascii_lowercase();
            let kind = classify_transport_error(
                &combined,
                request.proxy_config.mode,
                audit.proxy_redacted_endpoint.as_ref(),
            );
            if matches!(kind, UrlFetchErrorKind::TimeoutExceeded | UrlFetchErrorKind::ProxyTimeout)
            {
                audit.timeout_hit = true;
            }
            audit.proxy_error_kind = proxy_error_kind_for_transport(kind).map(ToString::to_string);
            Err(build_failure(
                request,
                audit,
                kind,
                "transport failure during HTTP GET",
            ))
        }
    }
}

fn proxy_for_url<'a>(
    mode: &ProxyMode,
    current_url: &str,
    request: &'a UrlFetchRequest,
) -> Result<Option<&'a str>, UrlFetchFailure> {
    match mode {
        ProxyMode::Off => Ok(None),
        ProxyMode::Env | ProxyMode::Explicit => {
            let is_https = current_url
                .to_ascii_lowercase()
                .starts_with("https://");
            if is_https {
                Ok(request.proxy_config.https_proxy_url.as_deref())
            } else {
                Ok(request.proxy_config.http_proxy_url.as_deref())
            }
        }
    }
}

fn classify_proxy_error(kind: ProxyErrorKind) -> UrlFetchErrorKind {
    match kind {
        ProxyErrorKind::ProxyMisconfigured => UrlFetchErrorKind::ProxyMisconfigured,
        ProxyErrorKind::ProxyAuthFailed => UrlFetchErrorKind::ProxyAuthFailed,
        ProxyErrorKind::ProxyConnectFailed => UrlFetchErrorKind::ProxyConnectFailed,
        ProxyErrorKind::ProxyTlsFailed => UrlFetchErrorKind::ProxyTlsFailed,
        ProxyErrorKind::ProxyDnsFailed => UrlFetchErrorKind::ProxyDnsFailed,
        ProxyErrorKind::ProxyTimeout => UrlFetchErrorKind::ProxyTimeout,
    }
}

fn classify_transport_error(
    raw: &str,
    mode: ProxyMode,
    proxy_endpoint: Option<&String>,
) -> UrlFetchErrorKind {
    let proxy_active = mode != ProxyMode::Off && proxy_endpoint.is_some();
    if raw.contains("timeout") {
        return if proxy_active {
            UrlFetchErrorKind::ProxyTimeout
        } else {
            UrlFetchErrorKind::TimeoutExceeded
        };
    }
    if proxy_active {
        if raw.contains("auth") || raw.contains("407") {
            return UrlFetchErrorKind::ProxyAuthFailed;
        }
        if raw.contains("dns") {
            return UrlFetchErrorKind::ProxyDnsFailed;
        }
        if raw.contains("tls") || raw.contains("ssl") {
            return UrlFetchErrorKind::ProxyTlsFailed;
        }
        if raw.contains("connect") || raw.contains("connection") || raw.contains("proxy") {
            return UrlFetchErrorKind::ProxyConnectFailed;
        }
    }
    UrlFetchErrorKind::TransportFailed
}

fn proxy_error_kind_for_transport(kind: UrlFetchErrorKind) -> Option<&'static str> {
    match kind {
        UrlFetchErrorKind::ProxyMisconfigured
        | UrlFetchErrorKind::ProxyAuthFailed
        | UrlFetchErrorKind::ProxyConnectFailed
        | UrlFetchErrorKind::ProxyTlsFailed
        | UrlFetchErrorKind::ProxyDnsFailed
        | UrlFetchErrorKind::ProxyTimeout => Some(kind.as_str()),
        _ => None,
    }
}

fn read_response_body(
    response: ureq::Response,
    content_type_header: Option<&str>,
    content_encoding_header: Option<&str>,
    policy: &UrlFetchPolicy,
) -> Result<BodyOutcome, UrlFetchErrorKind> {
    let content_encoding = parse_content_encoding(content_encoding_header)?;
    let bytes_read = Arc::new(AtomicUsize::new(0));
    let limited = LimitedReader::new(
        response.into_reader(),
        policy.max_response_bytes,
        Arc::clone(&bytes_read),
    );
    let mut decoded_reader = wrap_decoder(limited, content_encoding);

    let mut decompressed_bytes = 0usize;
    let mut sniff_prefix: Vec<u8> = Vec::new();
    let mut charset_probe: Vec<u8> = Vec::new();
    let mut selected_charset = charset_from_content_type(content_type_header);
    let mut decoder = selected_charset.map(|enc| enc.new_decoder());

    let mut decoded_text = String::new();
    let mut decoded_chars = 0usize;
    let mut read_buf = [0u8; 8192];
    loop {
        let read = decoded_reader
            .read(&mut read_buf)
            .map_err(|err| classify_stream_read_error(err, content_encoding))?;
        if read == 0 {
            break;
        }

        decompressed_bytes = decompressed_bytes.saturating_add(read);
        if decompressed_bytes > policy.max_decompressed_bytes {
            return Err(UrlFetchErrorKind::DecompressedTooLarge);
        }

        if sniff_prefix.len() < MIME_SNIFF_PREFIX_BYTES {
            let remaining = MIME_SNIFF_PREFIX_BYTES - sniff_prefix.len();
            let take = remaining.min(read);
            sniff_prefix.extend_from_slice(&read_buf[..take]);
        }

        if decoder.is_none() {
            charset_probe.extend_from_slice(&read_buf[..read]);
            if charset_probe.len() >= CHARSET_SNIFF_LIMIT_BYTES {
                selected_charset = Some(select_charset(content_type_header, &charset_probe));
                decoder = selected_charset.map(|enc| enc.new_decoder());
                decode_append(
                    decoder.as_mut().expect("decoder initialized"),
                    &charset_probe,
                    false,
                    &mut decoded_text,
                    &mut decoded_chars,
                    policy.max_extracted_chars,
                )?;
                charset_probe.clear();
            }
            continue;
        }

        decode_append(
            decoder.as_mut().expect("decoder initialized"),
            &read_buf[..read],
            false,
            &mut decoded_text,
            &mut decoded_chars,
            policy.max_extracted_chars,
        )?;
    }

    if decoder.is_none() {
        let selected = select_charset(content_type_header, &charset_probe);
        decoder = Some(selected.new_decoder());
        decode_append(
            decoder.as_mut().expect("decoder initialized"),
            &charset_probe,
            true,
            &mut decoded_text,
            &mut decoded_chars,
            policy.max_extracted_chars,
        )?;
    } else {
        decode_append(
            decoder.as_mut().expect("decoder initialized"),
            &[],
            true,
            &mut decoded_text,
            &mut decoded_chars,
            policy.max_extracted_chars,
        )?;
    }

    let mime = detect_allowed_mime(content_type_header, &sniff_prefix)?;
    let extracted = extract_document(mime, &decoded_text, policy.max_extracted_chars)?;
    let quality = evaluate_text_quality(
        &extracted.body_text,
        policy.min_text_length,
        policy.max_noise_ratio_bp,
    )?;

    Ok(BodyOutcome {
        mime,
        extracted,
        quality,
        bytes_read: bytes_read.load(Ordering::Relaxed),
        bytes_decompressed: decompressed_bytes,
    })
}

fn decode_append(
    decoder: &mut encoding_rs::Decoder,
    input: &[u8],
    last: bool,
    output: &mut String,
    decoded_chars: &mut usize,
    max_extracted_chars: usize,
) -> Result<(), UrlFetchErrorKind> {
    let mut start = 0usize;
    let mut had_any_errors = false;
    let before_len = output.len();
    loop {
        output.reserve(input.len().saturating_sub(start).saturating_add(16));
        let (result, read, had_errors) = decoder.decode_to_string(&input[start..], output, last);
        had_any_errors |= had_errors;
        start = start.saturating_add(read);
        match result {
            encoding_rs::CoderResult::InputEmpty => break,
            encoding_rs::CoderResult::OutputFull => continue,
        }
    }
    if had_any_errors {
        return Err(UrlFetchErrorKind::CharsetDecodeFailed);
    }

    let appended = &output[before_len..];
    *decoded_chars = decoded_chars.saturating_add(appended.chars().count());
    if *decoded_chars > max_extracted_chars {
        return Err(UrlFetchErrorKind::ExtractionTooLarge);
    }
    Ok(())
}

fn classify_stream_read_error(error: std::io::Error, encoding: ContentEncoding) -> UrlFetchErrorKind {
    let lower = error.to_string().to_ascii_lowercase();
    if lower.contains("response byte cap exceeded") {
        return UrlFetchErrorKind::ResponseTooLarge;
    }
    if lower.contains("timed out") {
        return UrlFetchErrorKind::TimeoutExceeded;
    }
    if encoding != ContentEncoding::Identity {
        return UrlFetchErrorKind::DecompressionFailed;
    }
    UrlFetchErrorKind::TransportFailed
}

fn build_failure_without_audit(
    request: &UrlFetchRequest,
    kind: UrlFetchErrorKind,
    message: &str,
) -> UrlFetchFailure {
    let mut audit = UrlFetchAudit::new(request.requested_url.clone(), request.proxy_config.mode);
    audit.reason_code = Some(kind.reason_code().to_string());
    audit.error_kind = Some(kind.as_str().to_string());
    let evidence_packet = build_failure_evidence_packet(request, &audit, kind, message);
    UrlFetchFailure {
        error_kind: kind,
        reason_code: kind.reason_code(),
        message: message.to_string(),
        audit,
        evidence_packet,
    }
}

fn build_failure(
    request: &UrlFetchRequest,
    audit: &UrlFetchAudit,
    kind: UrlFetchErrorKind,
    message: &str,
) -> UrlFetchFailure {
    let mut next_audit = audit.clone();
    next_audit.reason_code = Some(kind.reason_code().to_string());
    next_audit.error_kind = Some(kind.as_str().to_string());
    if matches!(kind, UrlFetchErrorKind::TimeoutExceeded | UrlFetchErrorKind::ProxyTimeout) {
        next_audit.timeout_hit = true;
    }
    let evidence_packet = build_failure_evidence_packet(request, &next_audit, kind, message);
    UrlFetchFailure {
        error_kind: kind,
        reason_code: kind.reason_code(),
        message: message.to_string(),
        audit: next_audit,
        evidence_packet,
    }
}

fn build_success_evidence_packet(
    request: &UrlFetchRequest,
    audit: &UrlFetchAudit,
    body: &BodyOutcome,
    chunk_output: &crate::web_search_plan::chunk::ChunkBuildOutput,
) -> Value {
    let final_url = audit
        .final_url
        .clone()
        .unwrap_or_else(|| audit.canonical_url.clone());
    let citation_anchors = build_citation_anchors(&chunk_output.chunks);
    let content_chunks: Vec<Value> = chunk_output
        .chunks
        .iter()
        .zip(citation_anchors.iter())
        .map(|(chunk, citation)| {
            json!({
                "chunk_id": chunk.chunk_id,
                "hash_version": chunk.hash_version,
                "norm_version": chunk.norm_version,
                "chunk_version": chunk.chunk_version,
                "source_url": chunk.source_url,
                "canonical_url": chunk.canonical_url,
                "chunk_index": chunk.chunk_index,
                "text_excerpt": bounded_excerpt(&chunk.normalized_text, 320),
                "text_len_chars": chunk.text_len_chars,
                "citation": {
                    "chunk_id": citation.chunk_id,
                    "source_url": citation.source_url
                }
            })
        })
        .collect();
    let truncation_reason = if chunk_output
        .reason_codes
        .iter()
        .any(|code| *code == EVIDENCE_TRUNCATED_REASON_CODE)
    {
        Some(EVIDENCE_TRUNCATED_REASON_CODE)
    } else {
        None
    };

    json!({
        "schema_version": "1.0.0",
        "produced_by": request.produced_by,
        "intended_consumers": request.intended_consumers,
        "created_at_ms": request.created_at_ms,
        "trace_id": request.trace_id,
        "query": request.query,
        "retrieved_at_ms": request.retrieved_at_ms,
        "provider_runs": [
            {
                "endpoint": "url_fetch",
                "latency_ms": audit.latency_ms,
                "status_code": audit.status_code,
                "canonical_url": audit.canonical_url,
                "final_url": final_url,
                "reason_code": truncation_reason,
                "reason_codes": chunk_output.reason_codes,
                "timeout_hit": audit.timeout_hit,
                "proxy": {
                    "mode": audit.proxy_mode,
                    "redacted_endpoint": audit.proxy_redacted_endpoint,
                    "error_kind": audit.proxy_error_kind,
                },
                "audit": {
                    "bytes_read": audit.bytes_read,
                    "bytes_decompressed": audit.bytes_decompressed,
                    "extraction_chars": audit.extraction_chars
                }
            }
        ],
        "sources": [
            {
                "title": body.extracted.title,
                "url": final_url,
                "media_type": "web",
                "mime_type": body.mime.as_str(),
                "canonical_url": audit.canonical_url
            }
        ],
        "content_chunks": content_chunks,
        "trust_metadata": {
            "canon_version": CANON_VERSION,
            "charset_version": CHARSET_VERSION,
            "normalization_version": NORMALIZATION_VERSION,
            "extraction_version": EXTRACTION_VERSION,
            "quality_gate_version": QUALITY_GATE_VERSION,
            "chunking": {
                "chunk_count": chunk_output.chunks.len(),
                "truncated": chunk_output.truncated,
                "reason_codes": chunk_output.reason_codes
            },
            "quality": {
                "text_len": body.quality.text_len,
                "noise_ratio_bp": body.quality.noise_ratio_bp
            }
        }
    })
}

fn map_chunk_error_to_fetch_failure(
    request: &UrlFetchRequest,
    audit: &UrlFetchAudit,
    error: ChunkBuildError,
) -> UrlFetchFailure {
    match error {
        ChunkBuildError::HashCollisionDetected {
            chunk_id,
            first_index,
            second_index,
        } => build_failure(
            request,
            audit,
            UrlFetchErrorKind::HashCollisionDetected,
            &format!(
                "{} chunk_id={} first_index={} second_index={}",
                HASH_COLLISION_REASON_CODE, chunk_id, first_index, second_index
            ),
        ),
        ChunkBuildError::CitationAnchorInvalid(message) => {
            build_failure(request, audit, UrlFetchErrorKind::TransportFailed, &message)
        }
    }
}

fn build_failure_evidence_packet(
    request: &UrlFetchRequest,
    audit: &UrlFetchAudit,
    kind: UrlFetchErrorKind,
    message: &str,
) -> Value {
    let debug_packet_value =
        build_fetch_debug_packet_value(request, audit, kind, message).unwrap_or(Value::Null);

    json!({
        "schema_version": "1.0.0",
        "produced_by": request.produced_by,
        "intended_consumers": request.intended_consumers,
        "created_at_ms": request.created_at_ms,
        "trace_id": request.trace_id,
        "query": request.query,
        "retrieved_at_ms": request.retrieved_at_ms,
        "provider_runs": [
            {
                "endpoint": "url_fetch",
                "latency_ms": audit.latency_ms,
                "status_code": audit.status_code,
                "canonical_url": audit.canonical_url,
                "final_url": audit.final_url,
                "reason_code": kind.reason_code(),
                "timeout_hit": audit.timeout_hit,
                "proxy": {
                    "mode": audit.proxy_mode,
                    "redacted_endpoint": audit.proxy_redacted_endpoint,
                    "error_kind": audit.proxy_error_kind,
                },
                "audit": {
                    "bytes_read": audit.bytes_read,
                    "bytes_decompressed": audit.bytes_decompressed,
                    "extraction_chars": audit.extraction_chars
                },
                "error": {
                    "error_kind": kind.as_str(),
                    "reason_code": kind.reason_code(),
                    "message": message
                }
            }
        ],
        "sources": [],
        "content_chunks": [],
        "trust_metadata": {
            "canon_version": CANON_VERSION,
            "charset_version": CHARSET_VERSION,
            "normalization_version": NORMALIZATION_VERSION,
            "extraction_version": EXTRACTION_VERSION,
            "quality_gate_version": QUALITY_GATE_VERSION,
            "failure": {
                "error_kind": kind.as_str(),
                "reason_code": kind.reason_code(),
                "debug_packet": debug_packet_value,
            }
        }
    })
}

fn build_fetch_debug_packet_value(
    request: &UrlFetchRequest,
    audit: &UrlFetchAudit,
    kind: UrlFetchErrorKind,
    message: &str,
) -> Option<Value> {
    let provider = match kind {
        UrlFetchErrorKind::ProxyMisconfigured
        | UrlFetchErrorKind::ProxyAuthFailed
        | UrlFetchErrorKind::ProxyConnectFailed
        | UrlFetchErrorKind::ProxyTlsFailed
        | UrlFetchErrorKind::ProxyDnsFailed
        | UrlFetchErrorKind::ProxyTimeout => "Proxy",
        UrlFetchErrorKind::HashCollisionDetected => "ChunkHash",
        _ => "UrlFetch",
    };

    let source_url = audit
        .final_url
        .as_deref()
        .or(Some(request.requested_url.as_str()));

    let packet = try_build_debug_packet(DebugPacketContext {
        trace_id: request.trace_id.as_str(),
        status: DebugStatus::Failed,
        provider,
        error_kind: kind.as_str(),
        reason_code: kind.reason_code(),
        proxy_mode: Some(request.proxy_config.mode.as_str()),
        source_url,
        created_at_ms: request.created_at_ms,
        turn_state_transitions: &default_failed_transitions(request.created_at_ms),
        debug_hint: Some(message),
        fallback_used: None,
        health_status_before_fallback: None,
    })
    .ok()?;

    serde_json::to_value(packet).ok()
}

fn is_redirect_status(status: u16) -> bool {
    matches!(status, 301 | 302 | 303 | 307 | 308)
}

struct LimitedReader<R: Read> {
    inner: R,
    max_bytes: usize,
    byte_counter: Arc<AtomicUsize>,
}

impl<R: Read> LimitedReader<R> {
    fn new(inner: R, max_bytes: usize, byte_counter: Arc<AtomicUsize>) -> Self {
        Self {
            inner,
            max_bytes,
            byte_counter,
        }
    }
}

impl<R: Read> Read for LimitedReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let used = self.byte_counter.load(Ordering::Relaxed);
        if used >= self.max_bytes {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "response byte cap exceeded",
            ));
        }

        let remaining = self.max_bytes.saturating_sub(used);
        let read_len = remaining.min(buf.len());
        let n = self.inner.read(&mut buf[..read_len])?;
        if n == 0 {
            return Ok(0);
        }

        let new_total = used.saturating_add(n);
        self.byte_counter.store(new_total, Ordering::Relaxed);
        Ok(n)
    }
}
