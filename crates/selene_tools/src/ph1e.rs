#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1d::SafetyTier;
use selene_kernel_contracts::ph1e::{
    CacheStatus, StrictBudget, StructuredAmbiguity, ToolName, ToolQueryHash, ToolRequest,
    ToolRequestId, ToolResponse, ToolResult,
};
use selene_kernel_contracts::{ContractViolation, ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.E reason-code namespace. Values are placeholders until the global registry is formalized.
    pub const E_OK_TIME: ReasonCodeId = ReasonCodeId(0x4500_1001);
    pub const E_OK_WEATHER: ReasonCodeId = ReasonCodeId(0x4500_1002);
    pub const E_OK_WEB_SEARCH: ReasonCodeId = ReasonCodeId(0x4500_1003);
    pub const E_OK_NEWS: ReasonCodeId = ReasonCodeId(0x4500_1004);
    pub const E_OK_URL_FETCH_AND_CITE: ReasonCodeId = ReasonCodeId(0x4500_1005);
    pub const E_OK_DOCUMENT_UNDERSTAND: ReasonCodeId = ReasonCodeId(0x4500_1006);
    pub const E_OK_PHOTO_UNDERSTAND: ReasonCodeId = ReasonCodeId(0x4500_1007);
    pub const E_OK_DATA_ANALYSIS: ReasonCodeId = ReasonCodeId(0x4500_1008);

    pub const E_FAIL_FORBIDDEN_TOOL: ReasonCodeId = ReasonCodeId(0x4500_0001);
    pub const E_FAIL_FORBIDDEN_ORIGIN: ReasonCodeId = ReasonCodeId(0x4500_0002);
    pub const E_FAIL_TIMEOUT: ReasonCodeId = ReasonCodeId(0x4500_0003);
    pub const E_FAIL_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x4500_0004);
    pub const E_FAIL_INTERNAL: ReasonCodeId = ReasonCodeId(0x4500_0005);
    pub const E_FAIL_POLICY_BLOCK: ReasonCodeId = ReasonCodeId(0x4500_0006);
    pub const E_FAIL_FORBIDDEN_DOMAIN: ReasonCodeId = ReasonCodeId(0x4500_0007);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolRouterConfig {
    pub max_timeout_ms: u32,
    pub max_results: u8,
    /// Empty => allow all domains.
    pub domain_allowlist: &'static [&'static str],
    /// Empty => deny none.
    pub domain_denylist: &'static [&'static str],
}

impl ToolRouterConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_timeout_ms: 10_000,
            max_results: 10,
            domain_allowlist: &[],
            domain_denylist: &[],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProviderOutcome {
    Ok {
        result: ToolResult,
        source_metadata: selene_kernel_contracts::ph1e::SourceMetadata,
        ambiguity: Option<StructuredAmbiguity>,
    },
    Timeout,
    BudgetExceeded,
    Fail {
        reason_code: ReasonCodeId,
    },
}

pub trait ToolProvider {
    fn call(
        &self,
        tool: &ToolName,
        query: &str,
        locale: Option<&str>,
        budget: StrictBudget,
    ) -> ProviderOutcome;
}

#[derive(Debug, Clone)]
pub struct ToolRouter {
    config: ToolRouterConfig,
}

impl ToolRouter {
    pub fn new(config: ToolRouterConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &ToolRequest, provider: &dyn ToolProvider) -> ToolResponse {
        let cache_status = CacheStatus::Bypassed;
        let (request_id, query_hash) = safe_ids(req);

        // Do not assume requests are well-formed; fail-closed.
        if let Err(v) = req.validate() {
            return map_contract_violation_to_fail(v, req, request_id, query_hash, cache_status);
        }

        // Defense-in-depth policy block (enterprise).
        if matches!(
            req.tool_name,
            ToolName::WebSearch | ToolName::News | ToolName::UrlFetchAndCite
        )
            && (req.policy_context_ref.privacy_mode
                || req.policy_context_ref.safety_tier == SafetyTier::Strict)
        {
            return ToolResponse::fail_v1(
                request_id,
                query_hash,
                reason_codes::E_FAIL_POLICY_BLOCK,
                cache_status,
            )
            .expect("ToolResponse::fail_v1 must construct");
        }

        if let ToolName::Other(_) = &req.tool_name {
            return ToolResponse::fail_v1(
                request_id,
                query_hash,
                reason_codes::E_FAIL_FORBIDDEN_TOOL,
                cache_status,
            )
            .expect("ToolResponse::fail_v1 must construct");
        }

        if req.strict_budget.timeout_ms > self.config.max_timeout_ms {
            return ToolResponse::fail_v1(
                request_id,
                query_hash,
                reason_codes::E_FAIL_BUDGET_EXCEEDED,
                cache_status,
            )
            .expect("ToolResponse::fail_v1 must construct");
        }

        let budget = StrictBudget {
            timeout_ms: req.strict_budget.timeout_ms,
            max_results: req.strict_budget.max_results.min(self.config.max_results),
        };

        match provider.call(&req.tool_name, &req.query, req.locale.as_deref(), budget) {
            ProviderOutcome::Ok {
                result,
                source_metadata,
                ambiguity,
            } => {
                let result = clamp_result(result, budget.max_results);

                if violates_domain_policy(
                    self.config.domain_allowlist,
                    self.config.domain_denylist,
                    &result,
                    &source_metadata,
                ) {
                    return ToolResponse::fail_v1(
                        request_id,
                        query_hash,
                        reason_codes::E_FAIL_FORBIDDEN_DOMAIN,
                        cache_status,
                    )
                    .expect("ToolResponse::fail_v1 must construct");
                }

                ToolResponse::ok_v1(
                    request_id,
                    query_hash,
                    result,
                    source_metadata,
                    ambiguity,
                    ok_reason_code(req.tool_name.as_str()),
                    cache_status,
                )
                .unwrap_or_else(|_| {
                    ToolResponse::fail_v1(
                        request_id,
                        query_hash,
                        reason_codes::E_FAIL_INTERNAL,
                        cache_status,
                    )
                    .expect("ToolResponse::fail_v1 must construct")
                })
            }
            ProviderOutcome::Timeout => ToolResponse::fail_v1(
                request_id,
                query_hash,
                reason_codes::E_FAIL_TIMEOUT,
                cache_status,
            )
            .expect("ToolResponse::fail_v1 must construct"),
            ProviderOutcome::BudgetExceeded => ToolResponse::fail_v1(
                request_id,
                query_hash,
                reason_codes::E_FAIL_BUDGET_EXCEEDED,
                cache_status,
            )
            .expect("ToolResponse::fail_v1 must construct"),
            ProviderOutcome::Fail { reason_code } => {
                ToolResponse::fail_v1(request_id, query_hash, reason_code, cache_status)
                    .expect("ToolResponse::fail_v1 must construct")
            }
        }
    }
}

fn safe_ids(req: &ToolRequest) -> (ToolRequestId, ToolQueryHash) {
    let request_id = if req.request_id.validate().is_ok() {
        req.request_id
    } else {
        ToolRequestId(1)
    };
    let query_hash = if req.query_hash.validate().is_ok() {
        req.query_hash
    } else {
        ToolQueryHash(1)
    };
    (request_id, query_hash)
}

fn ok_reason_code(tool_name: &str) -> ReasonCodeId {
    match tool_name {
        "time" => reason_codes::E_OK_TIME,
        "weather" => reason_codes::E_OK_WEATHER,
        "web_search" => reason_codes::E_OK_WEB_SEARCH,
        "news" => reason_codes::E_OK_NEWS,
        "url_fetch_and_cite" => reason_codes::E_OK_URL_FETCH_AND_CITE,
        "document_understand" => reason_codes::E_OK_DOCUMENT_UNDERSTAND,
        "photo_understand" => reason_codes::E_OK_PHOTO_UNDERSTAND,
        "data_analysis" => reason_codes::E_OK_DATA_ANALYSIS,
        _ => reason_codes::E_OK_WEB_SEARCH,
    }
}

fn clamp_result(mut result: ToolResult, max_results: u8) -> ToolResult {
    let n = max_results as usize;
    match &mut result {
        ToolResult::WebSearch { items } | ToolResult::News { items } => {
            if items.len() > n {
                items.truncate(n);
            }
        }
        ToolResult::UrlFetchAndCite { citations } => {
            if citations.len() > n {
                citations.truncate(n);
            }
        }
        ToolResult::DocumentUnderstand { citations, .. } => {
            if citations.len() > n {
                citations.truncate(n);
            }
        }
        ToolResult::PhotoUnderstand { citations, .. } => {
            if citations.len() > n {
                citations.truncate(n);
            }
        }
        ToolResult::DataAnalysis { citations, .. } => {
            if citations.len() > n {
                citations.truncate(n);
            }
        }
        ToolResult::Time { .. } | ToolResult::Weather { .. } => {}
    }
    result
}

fn violates_domain_policy(
    allowlist: &[&str],
    denylist: &[&str],
    result: &ToolResult,
    source_metadata: &selene_kernel_contracts::ph1e::SourceMetadata,
) -> bool {
    // Any forbidden URL => fail closed.
    match result {
        ToolResult::WebSearch { items } | ToolResult::News { items } => {
            for s in &source_metadata.sources {
                if !url_allowed(allowlist, denylist, &s.url) {
                    return true;
                }
            }
            for it in items {
                if !url_allowed(allowlist, denylist, &it.url) {
                    return true;
                }
            }
            false
        }
        ToolResult::UrlFetchAndCite { citations } => {
            for s in &source_metadata.sources {
                if !url_allowed(allowlist, denylist, &s.url) {
                    return true;
                }
            }
            for it in citations {
                if !url_allowed(allowlist, denylist, &it.url) {
                    return true;
                }
            }
            false
        }
        ToolResult::DocumentUnderstand { citations, .. } => {
            for s in &source_metadata.sources {
                if !url_allowed(allowlist, denylist, &s.url) {
                    return true;
                }
            }
            for it in citations {
                if !url_allowed(allowlist, denylist, &it.url) {
                    return true;
                }
            }
            false
        }
        ToolResult::PhotoUnderstand { citations, .. } => {
            for s in &source_metadata.sources {
                if !url_allowed(allowlist, denylist, &s.url) {
                    return true;
                }
            }
            for it in citations {
                if !url_allowed(allowlist, denylist, &it.url) {
                    return true;
                }
            }
            false
        }
        ToolResult::DataAnalysis { citations, .. } => {
            for s in &source_metadata.sources {
                if !url_allowed(allowlist, denylist, &s.url) {
                    return true;
                }
            }
            for it in citations {
                if !url_allowed(allowlist, denylist, &it.url) {
                    return true;
                }
            }
            false
        }
        ToolResult::Time { .. } | ToolResult::Weather { .. } => false,
    }
}

fn url_allowed(allowlist: &[&str], denylist: &[&str], url: &str) -> bool {
    let Some(host) = host_from_url(url) else {
        return false;
    };
    let host = host.to_ascii_lowercase();
    for d in denylist {
        if host_matches(&host, d) {
            return false;
        }
    }
    if allowlist.is_empty() {
        return true;
    }
    allowlist.iter().any(|a| host_matches(&host, a))
}

fn host_from_url(url: &str) -> Option<&str> {
    let url = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let host_port = url.split('/').next()?;
    Some(host_port.split(':').next().unwrap_or(host_port))
}

fn host_matches(host: &str, rule: &str) -> bool {
    let rule = rule.trim().trim_start_matches('.').to_ascii_lowercase();
    if rule.is_empty() {
        return false;
    }
    host == rule || host.ends_with(&format!(".{rule}"))
}

fn map_contract_violation_to_fail(
    v: ContractViolation,
    req: &ToolRequest,
    request_id: ToolRequestId,
    query_hash: ToolQueryHash,
    cache_status: CacheStatus,
) -> ToolResponse {
    // Provide a deterministic mapping to PH1.E reason codes.
    let rc = match (&req.origin, &req.tool_name, v) {
        (selene_kernel_contracts::ph1e::ToolRequestOrigin::Other(_), _, _) => {
            reason_codes::E_FAIL_FORBIDDEN_ORIGIN
        }
        (_, ToolName::Other(_), _) => reason_codes::E_FAIL_FORBIDDEN_TOOL,
        _ => reason_codes::E_FAIL_INTERNAL,
    };
    ToolResponse::fail_v1(request_id, query_hash, rc, cache_status)
        .expect("ToolResponse::fail_v1 must construct")
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1d::{PolicyContextRef, SafetyTier};
    use selene_kernel_contracts::ph1e::{
        SourceMetadata, SourceRef, ToolRequestOrigin, ToolStatus, ToolTextSnippet,
    };
    use selene_kernel_contracts::SchemaVersion;

    struct StubProvider {
        outcome: ProviderOutcome,
        last_locale: std::cell::RefCell<Option<String>>,
    }

    impl Default for StubProvider {
        fn default() -> Self {
            Self {
                outcome: ProviderOutcome::Fail {
                    reason_code: reason_codes::E_FAIL_INTERNAL,
                },
                last_locale: std::cell::RefCell::new(None),
            }
        }
    }

    impl ToolProvider for StubProvider {
        fn call(
            &self,
            _tool: &ToolName,
            _query: &str,
            locale: Option<&str>,
            _budget: StrictBudget,
        ) -> ProviderOutcome {
            *self.last_locale.borrow_mut() = locale.map(|s| s.to_string());
            self.outcome.clone()
        }
    }

    fn metadata(provider_hint: Option<&str>) -> SourceMetadata {
        SourceMetadata {
            schema_version: SchemaVersion(1),
            provider_hint: provider_hint.map(|s| s.to_string()),
            retrieved_at_unix_ms: 1,
            sources: vec![SourceRef {
                title: "t".to_string(),
                url: "https://example.com".to_string(),
            }],
        }
    }

    #[test]
    fn at_e_01_read_only_enforcement_rejects_other_tool() {
        let router = ToolRouter::new(ToolRouterConfig::mvp_v1());
        let provider = StubProvider::default();

        // Bypass the constructor to simulate a hostile/invalid request.
        let req = ToolRequest {
            schema_version: SchemaVersion(1),
            request_id: ToolRequestId(1),
            query_hash: ToolQueryHash(1),
            origin: ToolRequestOrigin::Ph1X,
            tool_name: ToolName::other("send_email").unwrap(),
            query: "hi".to_string(),
            locale: None,
            strict_budget: StrictBudget::new(1000, 3).unwrap(),
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
        };

        let out = router.run(&req, &provider);
        assert_eq!(out.tool_status, ToolStatus::Fail);
        assert_eq!(
            out.fail_reason_code,
            Some(reason_codes::E_FAIL_FORBIDDEN_TOOL)
        );
        assert!(out.tool_result.is_none());
    }

    #[test]
    fn at_e_02_deterministic_budget_timeout_returns_fail_timeout() {
        let router = ToolRouter::new(ToolRouterConfig::mvp_v1());
        let provider = StubProvider {
            outcome: ProviderOutcome::Timeout,
            ..Default::default()
        };

        let req = ToolRequest::v1(
            ToolRequestOrigin::Ph1X,
            ToolName::Time,
            "now".to_string(),
            None,
            StrictBudget::new(1000, 3).unwrap(),
            PolicyContextRef::v1(false, false, SafetyTier::Standard),
        )
        .unwrap();

        let out = router.run(&req, &provider);
        assert_eq!(out.tool_status, ToolStatus::Fail);
        assert_eq!(out.fail_reason_code, Some(reason_codes::E_FAIL_TIMEOUT));
        assert!(out.tool_result.is_none());
        assert_eq!(out.cache_status, CacheStatus::Bypassed);
    }

    #[test]
    fn at_e_03_provider_invisibility_provider_hint_not_required() {
        let router = ToolRouter::new(ToolRouterConfig::mvp_v1());
        let provider = StubProvider {
            outcome: ProviderOutcome::Ok {
                result: ToolResult::Time {
                    local_time_iso: "2026-02-08T12:00:00Z".to_string(),
                },
                source_metadata: metadata(None),
                ambiguity: None,
            },
            ..Default::default()
        };

        let req = ToolRequest::v1(
            ToolRequestOrigin::Ph1X,
            ToolName::Time,
            "now".to_string(),
            None,
            StrictBudget::new(1000, 3).unwrap(),
            PolicyContextRef::v1(false, false, SafetyTier::Standard),
        )
        .unwrap();

        let out = router.run(&req, &provider);
        assert_eq!(out.tool_status, ToolStatus::Ok);
        assert!(out.fail_reason_code.is_none());
        assert!(out
            .source_metadata
            .as_ref()
            .unwrap()
            .provider_hint
            .is_none());
        assert_eq!(out.cache_status, CacheStatus::Bypassed);
    }

    #[test]
    fn at_e_04_multilingual_query_passes_locale() {
        let router = ToolRouter::new(ToolRouterConfig::mvp_v1());
        let provider = StubProvider {
            outcome: ProviderOutcome::Ok {
                result: ToolResult::Weather {
                    summary: "clear".to_string(),
                },
                source_metadata: metadata(Some("weather")),
                ambiguity: None,
            },
            ..Default::default()
        };

        let req = ToolRequest::v1(
            ToolRequestOrigin::Ph1X,
            ToolName::Weather,
            "天气".to_string(),
            Some("zh-CN".to_string()),
            StrictBudget::new(1000, 3).unwrap(),
            PolicyContextRef::v1(false, false, SafetyTier::Standard),
        )
        .unwrap();

        let _out = router.run(&req, &provider);
        assert_eq!(provider.last_locale.borrow().as_deref(), Some("zh-CN"));
    }

    #[test]
    fn at_e_05_conflicting_sources_returns_structured_ambiguity() {
        let router = ToolRouter::new(ToolRouterConfig::mvp_v1());
        let provider = StubProvider {
            outcome: ProviderOutcome::Ok {
                result: ToolResult::News {
                    items: vec![ToolTextSnippet {
                        title: "headline".to_string(),
                        snippet: "one says A".to_string(),
                        url: "https://example.com/a".to_string(),
                    }],
                },
                source_metadata: metadata(Some("news")),
                ambiguity: Some(StructuredAmbiguity {
                    summary: "Sources disagree".to_string(),
                    alternatives: vec!["A".to_string(), "B".to_string()],
                }),
            },
            ..Default::default()
        };

        let req = ToolRequest::v1(
            ToolRequestOrigin::Ph1X,
            ToolName::News,
            "latest".to_string(),
            None,
            StrictBudget::new(1000, 3).unwrap(),
            PolicyContextRef::v1(false, false, SafetyTier::Standard),
        )
        .unwrap();

        let out = router.run(&req, &provider);
        assert_eq!(out.tool_status, ToolStatus::Ok);
        assert!(out.ambiguity.is_some());
    }

    #[test]
    fn at_e_06_reason_code_required_for_ok_outputs() {
        let router = ToolRouter::new(ToolRouterConfig::mvp_v1());
        let provider = StubProvider {
            outcome: ProviderOutcome::Ok {
                result: ToolResult::Time {
                    local_time_iso: "2026-02-08T12:00:00Z".to_string(),
                },
                source_metadata: metadata(None),
                ambiguity: None,
            },
            ..Default::default()
        };

        let req = ToolRequest::v1(
            ToolRequestOrigin::Ph1X,
            ToolName::Time,
            "now".to_string(),
            None,
            StrictBudget::new(1000, 3).unwrap(),
            PolicyContextRef::v1(false, false, SafetyTier::Standard),
        )
        .unwrap();

        let out = router.run(&req, &provider);
        assert_eq!(out.tool_status, ToolStatus::Ok);
        assert_eq!(out.reason_code, reason_codes::E_OK_TIME);
        assert_eq!(out.request_id, req.request_id);
        assert_eq!(out.query_hash, req.query_hash);
    }

    #[test]
    fn at_e_07_item_level_provenance_web_search_items_have_urls() {
        let router = ToolRouter::new(ToolRouterConfig::mvp_v1());
        let provider = StubProvider {
            outcome: ProviderOutcome::Ok {
                result: ToolResult::WebSearch {
                    items: vec![
                        ToolTextSnippet {
                            title: "t1".to_string(),
                            snippet: "s1".to_string(),
                            url: "https://example.com/1".to_string(),
                        },
                        ToolTextSnippet {
                            title: "t2".to_string(),
                            snippet: "s2".to_string(),
                            url: "https://example.com/2".to_string(),
                        },
                    ],
                },
                source_metadata: metadata(None),
                ambiguity: None,
            },
            ..Default::default()
        };

        let req = ToolRequest::v1(
            ToolRequestOrigin::Ph1X,
            ToolName::WebSearch,
            "query".to_string(),
            None,
            StrictBudget::new(1000, 3).unwrap(),
            PolicyContextRef::v1(false, false, SafetyTier::Standard),
        )
        .unwrap();

        let out = router.run(&req, &provider);
        assert_eq!(out.tool_status, ToolStatus::Ok);
        let ToolResult::WebSearch { items } = out.tool_result.unwrap() else {
            panic!("expected ToolResult::WebSearch");
        };
        assert!(items.iter().all(|i| !i.url.trim().is_empty()));
    }

    #[test]
    fn at_e_08_privacy_policy_blocks_web_search() {
        let router = ToolRouter::new(ToolRouterConfig::mvp_v1());
        let provider = StubProvider {
            outcome: ProviderOutcome::Ok {
                result: ToolResult::WebSearch {
                    items: vec![ToolTextSnippet {
                        title: "t".to_string(),
                        snippet: "s".to_string(),
                        url: "https://allowed.com".to_string(),
                    }],
                },
                source_metadata: metadata(None),
                ambiguity: None,
            },
            ..Default::default()
        };

        let req = ToolRequest::v1(
            ToolRequestOrigin::Ph1X,
            ToolName::WebSearch,
            "query".to_string(),
            None,
            StrictBudget::new(1000, 3).unwrap(),
            PolicyContextRef::v1(true, false, SafetyTier::Standard),
        )
        .unwrap();

        let out = router.run(&req, &provider);
        assert_eq!(out.tool_status, ToolStatus::Fail);
        assert_eq!(
            out.fail_reason_code,
            Some(reason_codes::E_FAIL_POLICY_BLOCK)
        );
    }

    #[test]
    fn at_e_09_domain_denylist_fails_closed() {
        let router = ToolRouter::new(ToolRouterConfig {
            domain_denylist: &["example.com"],
            ..ToolRouterConfig::mvp_v1()
        });
        let provider = StubProvider {
            outcome: ProviderOutcome::Ok {
                result: ToolResult::News {
                    items: vec![ToolTextSnippet {
                        title: "t".to_string(),
                        snippet: "s".to_string(),
                        url: "https://example.com/x".to_string(),
                    }],
                },
                source_metadata: metadata(None),
                ambiguity: None,
            },
            ..Default::default()
        };

        let req = ToolRequest::v1(
            ToolRequestOrigin::Ph1X,
            ToolName::News,
            "query".to_string(),
            None,
            StrictBudget::new(1000, 3).unwrap(),
            PolicyContextRef::v1(false, false, SafetyTier::Standard),
        )
        .unwrap();

        let out = router.run(&req, &provider);
        assert_eq!(out.tool_status, ToolStatus::Fail);
        assert_eq!(
            out.fail_reason_code,
            Some(reason_codes::E_FAIL_FORBIDDEN_DOMAIN)
        );
    }

    #[test]
    fn at_e_10_cache_status_is_always_present() {
        let router = ToolRouter::new(ToolRouterConfig::mvp_v1());
        let provider = StubProvider {
            outcome: ProviderOutcome::BudgetExceeded,
            ..Default::default()
        };

        let req = ToolRequest::v1(
            ToolRequestOrigin::Ph1X,
            ToolName::Time,
            "now".to_string(),
            None,
            StrictBudget::new(1000, 3).unwrap(),
            PolicyContextRef::v1(false, false, SafetyTier::Standard),
        )
        .unwrap();

        let out = router.run(&req, &provider);
        assert_eq!(out.cache_status, CacheStatus::Bypassed);
    }
}
