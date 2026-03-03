#![forbid(unsafe_code)]

pub mod canonical;
pub mod charset;
pub mod decompress;
pub mod extract;
pub mod fetch;
pub mod mime;
pub mod quality_gate;
pub mod redirect;

pub use fetch::fetch_url_to_evidence_packet;

use crate::web_search_plan::proxy::proxy_config::ProxyConfig;
use crate::web_search_plan::proxy::ProxyMode;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlFetchRequest {
    pub trace_id: String,
    pub query: String,
    pub requested_url: String,
    pub importance_tier: String,
    pub url_open_ordinal: usize,
    pub url_open_cap: Option<usize>,
    pub created_at_ms: i64,
    pub retrieved_at_ms: i64,
    pub produced_by: String,
    pub intended_consumers: Vec<String>,
    pub proxy_config: ProxyConfig,
    pub policy: UrlFetchPolicy,
}

impl UrlFetchRequest {
    pub fn new(
        trace_id: &str,
        query: &str,
        requested_url: &str,
        now_ms: i64,
        produced_by: &str,
        intended_consumers: Vec<String>,
        proxy_mode: ProxyMode,
    ) -> Self {
        Self {
            trace_id: trace_id.to_string(),
            query: query.to_string(),
            requested_url: requested_url.to_string(),
            importance_tier: "medium".to_string(),
            url_open_ordinal: 0,
            url_open_cap: None,
            created_at_ms: now_ms,
            retrieved_at_ms: now_ms,
            produced_by: produced_by.to_string(),
            intended_consumers,
            proxy_config: ProxyConfig {
                mode: proxy_mode,
                http_proxy_url: None,
                https_proxy_url: None,
            },
            policy: UrlFetchPolicy::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlFetchPolicy {
    pub connect_timeout_ms: u64,
    pub read_timeout_ms: u64,
    pub total_timeout_ms: u64,
    pub max_redirect_depth: usize,
    pub allow_scheme_downgrade: bool,
    pub allow_non_200: bool,
    pub max_response_bytes: usize,
    pub max_decompressed_bytes: usize,
    pub max_extracted_chars: usize,
    pub min_text_length: usize,
    pub max_noise_ratio_bp: u32,
}

impl Default for UrlFetchPolicy {
    fn default() -> Self {
        Self {
            connect_timeout_ms: 2_000,
            read_timeout_ms: 4_000,
            total_timeout_ms: 8_000,
            max_redirect_depth: 5,
            allow_scheme_downgrade: false,
            allow_non_200: false,
            max_response_bytes: 2 * 1024 * 1024,
            max_decompressed_bytes: 8 * 1024 * 1024,
            max_extracted_chars: 120_000,
            min_text_length: 64,
            max_noise_ratio_bp: 3_500,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UrlFetchErrorKind {
    BudgetExhausted,
    UnsupportedScheme,
    InvalidUrl,
    HttpNon200,
    RedirectLoopDetected,
    RedirectDepthExceeded,
    RedirectDowngradeBlocked,
    RedirectMissingLocation,
    MimeNotAllowed,
    MimeAmbiguous,
    UnsupportedContentEncoding,
    ResponseTooLarge,
    DecompressedTooLarge,
    ExtractionTooLarge,
    DecompressionFailed,
    CharsetDecodeFailed,
    HashCollisionDetected,
    ExtractionQualityLow,
    EmptyExtraction,
    TimeoutExceeded,
    ProxyMisconfigured,
    ProxyAuthFailed,
    ProxyConnectFailed,
    ProxyTlsFailed,
    ProxyDnsFailed,
    ProxyTimeout,
    TransportFailed,
}

impl UrlFetchErrorKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BudgetExhausted => "budget_exhausted",
            Self::UnsupportedScheme => "unsupported_scheme",
            Self::InvalidUrl => "invalid_url",
            Self::HttpNon200 => "http_non_200",
            Self::RedirectLoopDetected => "redirect_loop_detected",
            Self::RedirectDepthExceeded => "redirect_depth_exceeded",
            Self::RedirectDowngradeBlocked => "redirect_scheme_downgrade_blocked",
            Self::RedirectMissingLocation => "redirect_missing_location",
            Self::MimeNotAllowed => "mime_not_allowed",
            Self::MimeAmbiguous => "mime_ambiguous",
            Self::UnsupportedContentEncoding => "unsupported_content_encoding",
            Self::ResponseTooLarge => "response_bytes_exceeded",
            Self::DecompressedTooLarge => "decompressed_bytes_exceeded",
            Self::ExtractionTooLarge => "extraction_chars_exceeded",
            Self::DecompressionFailed => "decompression_failed",
            Self::CharsetDecodeFailed => "charset_decode_failed",
            Self::HashCollisionDetected => "hash_collision_detected",
            Self::ExtractionQualityLow => "extraction_quality_low",
            Self::EmptyExtraction => "empty_extraction",
            Self::TimeoutExceeded => "timeout_exceeded",
            Self::ProxyMisconfigured => "proxy_misconfigured",
            Self::ProxyAuthFailed => "proxy_auth_failed",
            Self::ProxyConnectFailed => "proxy_connect_failed",
            Self::ProxyTlsFailed => "proxy_tls_failed",
            Self::ProxyDnsFailed => "proxy_dns_failed",
            Self::ProxyTimeout => "proxy_timeout",
            Self::TransportFailed => "transport_failed",
        }
    }

    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::BudgetExhausted => "budget_exhausted",
            Self::TimeoutExceeded | Self::ProxyTimeout => "timeout_exceeded",
            Self::ProxyMisconfigured
            | Self::ProxyAuthFailed
            | Self::ProxyConnectFailed
            | Self::ProxyTlsFailed
            | Self::ProxyDnsFailed => "proxy_misconfigured",
            Self::ExtractionQualityLow | Self::EmptyExtraction => "empty_results",
            Self::HashCollisionDetected => "hash_collision_detected",
            _ => "provider_upstream_failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlFetchAudit {
    pub canonical_url: String,
    pub final_url: Option<String>,
    pub status_code: Option<u16>,
    pub bytes_read: usize,
    pub bytes_decompressed: usize,
    pub extraction_chars: usize,
    pub timeout_hit: bool,
    pub reason_code: Option<String>,
    pub error_kind: Option<String>,
    pub latency_ms: u64,
    pub proxy_mode: String,
    pub proxy_redacted_endpoint: Option<String>,
    pub proxy_error_kind: Option<String>,
}

impl UrlFetchAudit {
    pub fn new(canonical_url: String, proxy_mode: ProxyMode) -> Self {
        Self {
            canonical_url,
            final_url: None,
            status_code: None,
            bytes_read: 0,
            bytes_decompressed: 0,
            extraction_chars: 0,
            timeout_hit: false,
            reason_code: None,
            error_kind: None,
            latency_ms: 0,
            proxy_mode: proxy_mode.as_str().to_string(),
            proxy_redacted_endpoint: None,
            proxy_error_kind: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UrlFetchSuccess {
    pub evidence_packet: Value,
    pub title: String,
    pub body_text: String,
    pub media_type: String,
    pub audit: UrlFetchAudit,
}

#[derive(Debug, Clone)]
pub struct UrlFetchFailure {
    pub error_kind: UrlFetchErrorKind,
    pub reason_code: &'static str,
    pub message: String,
    pub audit: UrlFetchAudit,
    pub evidence_packet: Value,
}

#[cfg(test)]
pub mod url_tests;
