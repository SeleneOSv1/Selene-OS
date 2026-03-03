#![forbid(unsafe_code)]

use crate::web_search_plan::planning::{OpenFailure, OpenSuccess, SearchCandidate};
use crate::web_search_plan::proxy::proxy_config::ProxyConfig;
use crate::web_search_plan::url::{fetch_url_to_evidence_packet, UrlFetchPolicy, UrlFetchRequest};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlOpenContext {
    pub trace_id: String,
    pub query: String,
    pub importance_tier: String,
    pub created_at_ms: i64,
    pub retrieved_at_ms: i64,
    pub produced_by: String,
    pub intended_consumers: Vec<String>,
    pub proxy_config: ProxyConfig,
    pub url_fetch_policy: UrlFetchPolicy,
    pub url_open_cap: usize,
}

pub fn open_candidate_with_url_fetch(
    context: &UrlOpenContext,
    candidate: &SearchCandidate,
    url_open_ordinal: usize,
) -> Result<OpenSuccess, OpenFailure> {
    let request = UrlFetchRequest {
        trace_id: context.trace_id.clone(),
        query: context.query.clone(),
        requested_url: candidate.url.clone(),
        importance_tier: context.importance_tier.clone(),
        url_open_ordinal,
        url_open_cap: Some(context.url_open_cap),
        created_at_ms: context.created_at_ms,
        retrieved_at_ms: context.retrieved_at_ms,
        produced_by: context.produced_by.clone(),
        intended_consumers: context.intended_consumers.clone(),
        proxy_config: context.proxy_config.clone(),
        policy: context.url_fetch_policy.clone(),
    };

    match fetch_url_to_evidence_packet(&request) {
        Ok(success) => Ok(OpenSuccess {
            final_url: success
                .audit
                .final_url
                .clone()
                .unwrap_or_else(|| candidate.url.clone()),
            title: if success.title.trim().is_empty() {
                candidate.title.clone()
            } else {
                success.title
            },
            extracted_text: success.body_text,
            extracted_chars: success.audit.extraction_chars,
        }),
        Err(failure) => Err(OpenFailure {
            canonical_url: candidate.canonical_url.clone(),
            reason_code: failure.reason_code.to_string(),
            error_kind: failure.error_kind.as_str().to_string(),
            message: failure.message,
        }),
    }
}
