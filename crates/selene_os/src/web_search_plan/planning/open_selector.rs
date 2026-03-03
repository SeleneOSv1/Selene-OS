#![forbid(unsafe_code)]

use crate::web_search_plan::cache::cache_key::{CacheKey, CacheMode};
use crate::web_search_plan::cache::l1::L1Cache;
use crate::web_search_plan::cache::ttl::ttl_ms_for;
use crate::web_search_plan::cache::{lookup_typed, store_typed};
use crate::web_search_plan::planning::{OpenFailure, OpenSuccess, SearchCandidate};
use crate::web_search_plan::perf_cost::tiers::ImportanceTier;
use crate::web_search_plan::proxy::proxy_config::ProxyConfig;
use crate::web_search_plan::url::{fetch_url_to_evidence_packet, UrlFetchPolicy, UrlFetchRequest};

const CACHE_SCHEMA_VERSION: &str = "1.0.0";
const URL_FETCH_PROVIDER_ID: &str = "url_fetch_open";

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
    pub cache_enabled: bool,
    pub cache_policy_snapshot_id: String,
}

pub fn open_candidate_with_url_fetch(
    context: &UrlOpenContext,
    candidate: &SearchCandidate,
    url_open_ordinal: usize,
) -> Result<OpenSuccess, OpenFailure> {
    let tier = ImportanceTier::parse_or_default(context.importance_tier.as_str());
    let mut l1_cache = L1Cache::default();
    let cache_key = CacheKey::new(
        CacheMode::UrlFetch,
        context.query.as_str(),
        Some(candidate.canonical_url.as_str()),
        Some(URL_FETCH_PROVIDER_ID),
        tier,
        Some(context.cache_policy_snapshot_id.as_str()),
    );

    if context.cache_enabled {
        match lookup_typed::<OpenSuccess>(
            &mut l1_cache,
            &cache_key,
            context.retrieved_at_ms,
            CACHE_SCHEMA_VERSION,
            context.cache_policy_snapshot_id.as_str(),
        ) {
            Ok(Some(hit)) => return Ok(hit.value),
            Ok(None) => {}
            Err(err) => {
                return Err(OpenFailure {
                    canonical_url: candidate.canonical_url.clone(),
                    reason_code: "policy_violation".to_string(),
                    error_kind: "cache_policy_violation".to_string(),
                    message: format!("cache lookup rejected: {}", err),
                })
            }
        }
    }

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
        Ok(success) => {
            let open_success = OpenSuccess {
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
            };

            if context.cache_enabled {
                let ttl_ms = ttl_ms_for(CacheMode::UrlFetch, tier);
                if let Err(err) = store_typed(
                    &mut l1_cache,
                    &cache_key,
                    &open_success,
                    CACHE_SCHEMA_VERSION,
                    context.retrieved_at_ms,
                    ttl_ms,
                    context.cache_policy_snapshot_id.as_str(),
                    context.retrieved_at_ms,
                ) {
                    return Err(OpenFailure {
                        canonical_url: candidate.canonical_url.clone(),
                        reason_code: "policy_violation".to_string(),
                        error_kind: "cache_policy_violation".to_string(),
                        message: format!("cache store rejected: {}", err),
                    });
                }
            }

            Ok(open_success)
        }
        Err(failure) => Err(OpenFailure {
            canonical_url: candidate.canonical_url.clone(),
            reason_code: failure.reason_code.to_string(),
            error_kind: failure.error_kind.as_str().to_string(),
            message: failure.message,
        }),
    }
}
