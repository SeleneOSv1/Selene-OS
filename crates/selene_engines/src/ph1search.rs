#![forbid(unsafe_code)]

use std::cmp::min;
use std::collections::BTreeSet;

use selene_kernel_contracts::ph1search::{
    Ph1SearchRequest, Ph1SearchResponse, SearchCapabilityId, SearchPlanBuildOk,
    SearchPlanBuildRequest, SearchPlanQuery, SearchQueryId, SearchQueryRewriteOk,
    SearchQueryRewriteRequest, SearchRefuse, SearchValidationStatus,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.SEARCH reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_SEARCH_OK_PLAN_BUILD: ReasonCodeId = ReasonCodeId(0x5348_0001);
    pub const PH1_SEARCH_OK_QUERY_REWRITE: ReasonCodeId = ReasonCodeId(0x5348_0002);

    pub const PH1_SEARCH_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x5348_00F1);
    pub const PH1_SEARCH_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x5348_00F2);
    pub const PH1_SEARCH_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x5348_00F3);
    pub const PH1_SEARCH_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5348_00F4);
    pub const PH1_SEARCH_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x5348_00F5);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1SearchConfig {
    pub max_raw_query_chars: usize,
    pub max_output_queries: u8,
    pub max_diagnostics: u8,
}

impl Ph1SearchConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_raw_query_chars: 512,
            max_output_queries: 4,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1SearchRuntime {
    config: Ph1SearchConfig,
}

impl Ph1SearchRuntime {
    pub fn new(config: Ph1SearchConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1SearchRequest) -> Ph1SearchResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_SEARCH_INPUT_SCHEMA_INVALID,
                "search request failed contract validation",
            );
        }

        match req {
            Ph1SearchRequest::SearchPlanBuild(r) => self.run_plan_build(r),
            Ph1SearchRequest::SearchQueryRewrite(r) => self.run_query_rewrite(r),
        }
    }

    fn run_plan_build(&self, req: &SearchPlanBuildRequest) -> Ph1SearchResponse {
        if req.raw_query.trim().is_empty() {
            return self.refuse(
                SearchCapabilityId::SearchPlanBuild,
                reason_codes::PH1_SEARCH_UPSTREAM_INPUT_MISSING,
                "raw query is empty",
            );
        }

        if req.raw_query.len() > self.config.max_raw_query_chars {
            return self.refuse(
                SearchCapabilityId::SearchPlanBuild,
                reason_codes::PH1_SEARCH_BUDGET_EXCEEDED,
                "raw query exceeds budget",
            );
        }

        let query_budget = min(
            req.envelope.max_plan_queries,
            self.config.max_output_queries,
        ) as usize;
        let mut candidates: Vec<String> = Vec::new();
        let normalized = collapse_ws(req.raw_query.trim());

        if !normalized.is_empty() {
            candidates.push(normalized.clone());
        }

        let without_assistant_prefix = strip_assistant_prefix(&normalized);
        if !without_assistant_prefix.is_empty() {
            candidates.push(without_assistant_prefix);
        }

        let without_polite_prefix = strip_polite_prefix(&normalized);
        if !without_polite_prefix.is_empty() {
            candidates.push(without_polite_prefix);
        }

        let punctuation_light = strip_terminal_punctuation(&normalized);
        if !punctuation_light.is_empty() {
            candidates.push(punctuation_light);
        }

        let mut seen: BTreeSet<String> = BTreeSet::new();
        let mut planned_queries: Vec<SearchPlanQuery> = Vec::new();
        for candidate in candidates {
            if planned_queries.len() >= query_budget {
                break;
            }
            let canonical = candidate.to_ascii_lowercase();
            if canonical.is_empty() || !seen.insert(canonical) {
                continue;
            }

            let query_id =
                match SearchQueryId::new(format!("search_q_{:03}", planned_queries.len())) {
                    Ok(id) => id,
                    Err(_) => {
                        return self.refuse(
                            SearchCapabilityId::SearchPlanBuild,
                            reason_codes::PH1_SEARCH_INTERNAL_PIPELINE_ERROR,
                            "failed to build deterministic query id",
                        )
                    }
                };

            let planned = match SearchPlanQuery::v1(
                query_id,
                truncate_to_char_boundary(&candidate, 256),
                req.language_hint.clone(),
            ) {
                Ok(q) => q,
                Err(_) => {
                    return self.refuse(
                        SearchCapabilityId::SearchPlanBuild,
                        reason_codes::PH1_SEARCH_INTERNAL_PIPELINE_ERROR,
                        "failed to build search plan query",
                    )
                }
            };
            planned_queries.push(planned);
        }

        if planned_queries.is_empty() {
            return self.refuse(
                SearchCapabilityId::SearchPlanBuild,
                reason_codes::PH1_SEARCH_UPSTREAM_INPUT_MISSING,
                "no search plan query could be produced",
            );
        }

        match SearchPlanBuildOk::v1(
            reason_codes::PH1_SEARCH_OK_PLAN_BUILD,
            planned_queries,
            true,
        ) {
            Ok(ok) => Ph1SearchResponse::SearchPlanBuildOk(ok),
            Err(_) => self.refuse(
                SearchCapabilityId::SearchPlanBuild,
                reason_codes::PH1_SEARCH_INTERNAL_PIPELINE_ERROR,
                "failed to construct plan build output",
            ),
        }
    }

    fn run_query_rewrite(&self, req: &SearchQueryRewriteRequest) -> Ph1SearchResponse {
        if req.planned_queries.is_empty() {
            return self.refuse(
                SearchCapabilityId::SearchQueryRewrite,
                reason_codes::PH1_SEARCH_UPSTREAM_INPUT_MISSING,
                "no planned queries were provided",
            );
        }

        if req.planned_queries.len() > req.envelope.max_plan_queries as usize {
            return self.refuse(
                SearchCapabilityId::SearchQueryRewrite,
                reason_codes::PH1_SEARCH_BUDGET_EXCEEDED,
                "planned query budget exceeded",
            );
        }

        let raw_tokens = token_set(&req.raw_query);
        let mut diagnostics: Vec<String> = Vec::new();
        let mut rewritten_queries: Vec<SearchPlanQuery> = Vec::new();

        for (idx, planned) in req.planned_queries.iter().enumerate() {
            let rewritten_text = rewrite_query_text(&planned.query_text);
            if rewritten_text.trim().is_empty() {
                diagnostics.push(format!("query_{idx}_rewrite_empty"));
            } else if !has_any_shared_token(&raw_tokens, &token_set(&rewritten_text)) {
                diagnostics.push(format!("query_{idx}_not_intent_anchored"));
            }

            if diagnostics.len() >= self.config.max_diagnostics as usize {
                break;
            }

            let rewritten = match SearchPlanQuery::v1(
                planned.query_id.clone(),
                truncate_to_char_boundary(&rewritten_text, 256),
                planned.language_hint.clone(),
            ) {
                Ok(q) => q,
                Err(_) => {
                    return self.refuse(
                        SearchCapabilityId::SearchQueryRewrite,
                        reason_codes::PH1_SEARCH_INTERNAL_PIPELINE_ERROR,
                        "failed to build rewritten query",
                    )
                }
            };
            rewritten_queries.push(rewritten);
        }

        if rewritten_queries.is_empty() {
            return self.refuse(
                SearchCapabilityId::SearchQueryRewrite,
                reason_codes::PH1_SEARCH_UPSTREAM_INPUT_MISSING,
                "no rewritten query could be produced",
            );
        }

        let (status, reason_code) = if diagnostics.is_empty() {
            (
                SearchValidationStatus::Ok,
                reason_codes::PH1_SEARCH_OK_QUERY_REWRITE,
            )
        } else {
            (
                SearchValidationStatus::Fail,
                reason_codes::PH1_SEARCH_VALIDATION_FAILED,
            )
        };

        match SearchQueryRewriteOk::v1(reason_code, status, rewritten_queries, diagnostics, true) {
            Ok(ok) => Ph1SearchResponse::SearchQueryRewriteOk(ok),
            Err(_) => self.refuse(
                SearchCapabilityId::SearchQueryRewrite,
                reason_codes::PH1_SEARCH_INTERNAL_PIPELINE_ERROR,
                "failed to construct query rewrite output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: SearchCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1SearchResponse {
        let r = SearchRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("SearchRefuse::v1 must construct for static message");
        Ph1SearchResponse::Refuse(r)
    }
}

fn capability_from_request(req: &Ph1SearchRequest) -> SearchCapabilityId {
    match req {
        Ph1SearchRequest::SearchPlanBuild(_) => SearchCapabilityId::SearchPlanBuild,
        Ph1SearchRequest::SearchQueryRewrite(_) => SearchCapabilityId::SearchQueryRewrite,
    }
}

fn collapse_ws(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn strip_assistant_prefix(input: &str) -> String {
    let prefixes = ["selene ", "hey selene "];
    let lower = input.to_ascii_lowercase();
    for prefix in prefixes {
        if lower.starts_with(prefix) {
            return collapse_ws(&input[prefix.len()..]);
        }
    }
    input.to_string()
}

fn strip_polite_prefix(input: &str) -> String {
    let prefixes = [
        "please ",
        "can you ",
        "could you ",
        "would you ",
        "help me ",
    ];
    let lower = input.to_ascii_lowercase();
    for prefix in prefixes {
        if lower.starts_with(prefix) {
            return collapse_ws(&input[prefix.len()..]);
        }
    }
    input.to_string()
}

fn strip_terminal_punctuation(input: &str) -> String {
    input
        .trim_end_matches(|c: char| matches!(c, '?' | '!' | '.' | ','))
        .trim()
        .to_string()
}

fn rewrite_query_text(input: &str) -> String {
    let collapsed = collapse_ws(input.trim());
    strip_terminal_punctuation(&collapsed)
}

fn token_set(input: &str) -> BTreeSet<String> {
    input
        .to_ascii_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect::<BTreeSet<_>>()
}

fn has_any_shared_token(left: &BTreeSet<String>, right: &BTreeSet<String>) -> bool {
    if left.is_empty() || right.is_empty() {
        return false;
    }
    left.iter().any(|token| right.contains(token))
}

fn truncate_to_char_boundary(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }
    input.chars().take(max_chars).collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1search::{
        SearchPlanBuildRequest, SearchQueryId, SearchQueryRewriteRequest, SearchRequestEnvelope,
    };

    fn runtime() -> Ph1SearchRuntime {
        Ph1SearchRuntime::new(Ph1SearchConfig::mvp_v1())
    }

    fn envelope(max_queries: u8) -> SearchRequestEnvelope {
        SearchRequestEnvelope::v1(CorrelationId(1001), TurnId(61), max_queries).unwrap()
    }

    fn planned(id: &str, text: &str) -> SearchPlanQuery {
        SearchPlanQuery::v1(
            SearchQueryId::new(id).unwrap(),
            text.to_string(),
            Some("en".to_string()),
        )
        .unwrap()
    }

    #[test]
    fn at_search_01_plan_build_output_is_schema_valid() {
        let req = Ph1SearchRequest::SearchPlanBuild(
            SearchPlanBuildRequest::v1(
                envelope(4),
                "Selene please weather in San Francisco tomorrow?".to_string(),
                Some("en".to_string()),
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        assert!(out.validate().is_ok());
        match out {
            Ph1SearchResponse::SearchPlanBuildOk(ok) => {
                assert_eq!(ok.capability_id, SearchCapabilityId::SearchPlanBuild);
                assert!(ok.no_intent_drift);
                assert!(!ok.planned_queries.is_empty());
            }
            _ => panic!("expected SearchPlanBuildOk"),
        }
    }

    #[test]
    fn at_search_02_plan_build_order_is_deterministic() {
        let req = Ph1SearchRequest::SearchPlanBuild(
            SearchPlanBuildRequest::v1(
                envelope(4),
                "Selene can you weather in SF tomorrow???".to_string(),
                Some("en".to_string()),
            )
            .unwrap(),
        );

        let runtime = runtime();
        let out1 = runtime.run(&req);
        let out2 = runtime.run(&req);

        let list1 = match out1 {
            Ph1SearchResponse::SearchPlanBuildOk(ok) => ok
                .planned_queries
                .into_iter()
                .map(|q| q.query_text)
                .collect::<Vec<_>>(),
            _ => panic!("expected SearchPlanBuildOk"),
        };
        let list2 = match out2 {
            Ph1SearchResponse::SearchPlanBuildOk(ok) => ok
                .planned_queries
                .into_iter()
                .map(|q| q.query_text)
                .collect::<Vec<_>>(),
            _ => panic!("expected SearchPlanBuildOk"),
        };

        assert_eq!(list1, list2);
    }

    #[test]
    fn at_search_03_budget_bound_is_enforced() {
        let req = Ph1SearchRequest::SearchPlanBuild(
            SearchPlanBuildRequest::v1(
                envelope(2),
                "Selene please could you weather in New York tomorrow".to_string(),
                Some("en".to_string()),
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        match out {
            Ph1SearchResponse::SearchPlanBuildOk(ok) => {
                assert!(ok.planned_queries.len() <= 2);
            }
            _ => panic!("expected SearchPlanBuildOk"),
        }
    }

    #[test]
    fn at_search_04_query_rewrite_fails_for_intent_drift() {
        let req = Ph1SearchRequest::SearchQueryRewrite(
            SearchQueryRewriteRequest::v1(
                envelope(4),
                "weather in singapore tomorrow".to_string(),
                vec![planned("q1", "stock market today")],
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        match out {
            Ph1SearchResponse::SearchQueryRewriteOk(ok) => {
                assert_eq!(ok.validation_status, SearchValidationStatus::Fail);
                assert_eq!(ok.reason_code, reason_codes::PH1_SEARCH_VALIDATION_FAILED);
                assert!(ok
                    .diagnostics
                    .iter()
                    .any(|d| d == "query_0_not_intent_anchored"));
            }
            _ => panic!("expected SearchQueryRewriteOk"),
        }
    }
}
