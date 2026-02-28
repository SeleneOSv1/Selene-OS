#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1e::{
    CacheStatus, SourceMetadata, SourceRef, StrictBudget, ToolName, ToolRequest, ToolResponse,
    ToolResult, ToolStructuredField, ToolTextSnippet,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.E reason-code namespace. Values are placeholders until global registry lock.
    pub const E_OK_TOOL_RESULT: ReasonCodeId = ReasonCodeId(0x4500_0001);

    pub const E_FAIL_TIMEOUT: ReasonCodeId = ReasonCodeId(0x4500_00F1);
    pub const E_FAIL_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x4500_00F2);
    pub const E_FAIL_FORBIDDEN_TOOL: ReasonCodeId = ReasonCodeId(0x4500_00F3);
    pub const E_FAIL_POLICY_BLOCK: ReasonCodeId = ReasonCodeId(0x4500_00F4);
    pub const E_FAIL_FORBIDDEN_DOMAIN: ReasonCodeId = ReasonCodeId(0x4500_00F5);
    pub const E_FAIL_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4500_00FF);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1eConfig {
    pub max_timeout_ms: u32,
    pub max_results: u8,
}

impl Ph1eConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_timeout_ms: 2_000,
            max_results: 5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1eRuntime {
    config: Ph1eConfig,
}

impl Ph1eRuntime {
    pub fn new(config: Ph1eConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &ToolRequest) -> ToolResponse {
        if req.validate().is_err() {
            return fail_response(
                req,
                reason_codes::E_FAIL_FORBIDDEN_TOOL,
                CacheStatus::Bypassed,
            );
        }

        if budget_exceeded(req.strict_budget, self.config) {
            return fail_response(
                req,
                reason_codes::E_FAIL_BUDGET_EXCEEDED,
                CacheStatus::Bypassed,
            );
        }

        if policy_blocks(req) {
            return fail_response(
                req,
                reason_codes::E_FAIL_POLICY_BLOCK,
                CacheStatus::Bypassed,
            );
        }

        if connector_scope_policy_block(req) {
            return fail_response(
                req,
                reason_codes::E_FAIL_POLICY_BLOCK,
                CacheStatus::Bypassed,
            );
        }

        if forbidden_domain(req) {
            return fail_response(
                req,
                reason_codes::E_FAIL_FORBIDDEN_DOMAIN,
                CacheStatus::Bypassed,
            );
        }

        if deterministic_timeout(req) {
            return fail_response(req, reason_codes::E_FAIL_TIMEOUT, CacheStatus::Miss);
        }

        let cache_status = cache_status_for_request(req);
        let tool_result = match &req.tool_name {
            ToolName::Time => ToolResult::Time {
                local_time_iso: "2026-01-01T00:00:00Z".to_string(),
            },
            ToolName::Weather => ToolResult::Weather {
                summary: format!("Weather snapshot for {}", req.query),
            },
            ToolName::WebSearch => ToolResult::WebSearch {
                items: vec![ToolTextSnippet {
                    title: format!("Search: {}", truncate_ascii(&req.query, 40)),
                    snippet: format!("Result for query '{}'", truncate_ascii(&req.query, 80)),
                    url: "https://example.com/search-result".to_string(),
                }],
            },
            ToolName::News => ToolResult::News {
                items: vec![ToolTextSnippet {
                    title: format!("News: {}", truncate_ascii(&req.query, 40)),
                    snippet: format!("News item for '{}'", truncate_ascii(&req.query, 80)),
                    url: "https://example.com/news-item".to_string(),
                }],
            },
            ToolName::UrlFetchAndCite => ToolResult::UrlFetchAndCite {
                citations: vec![ToolTextSnippet {
                    title: format!("Source page: {}", truncate_ascii(&req.query, 40)),
                    snippet: format!("Citation extracted from '{}'", truncate_ascii(&req.query, 80)),
                    url: "https://example.com/url-fetch-citation".to_string(),
                }],
            },
            ToolName::DocumentUnderstand => ToolResult::DocumentUnderstand {
                summary: format!(
                    "Document summary for '{}'",
                    truncate_ascii(&req.query, 80)
                ),
                extracted_fields: vec![
                    ToolStructuredField {
                        key: "document_type".to_string(),
                        value: "pdf".to_string(),
                    },
                    ToolStructuredField {
                        key: "key_point".to_string(),
                        value: "Deterministic extracted statement".to_string(),
                    },
                ],
                citations: vec![ToolTextSnippet {
                    title: "Document citation".to_string(),
                    snippet: "Extracted from uploaded document segment".to_string(),
                    url: "https://example.com/document-citation".to_string(),
                }],
            },
            ToolName::PhotoUnderstand => ToolResult::PhotoUnderstand {
                summary: format!(
                    "Photo summary for '{}'",
                    truncate_ascii(&req.query, 80)
                ),
                extracted_fields: vec![
                    ToolStructuredField {
                        key: "visible_text".to_string(),
                        value: "Detected text fragment".to_string(),
                    },
                    ToolStructuredField {
                        key: "chart_signal".to_string(),
                        value: "Upward trend".to_string(),
                    },
                ],
                citations: vec![ToolTextSnippet {
                    title: "Image region citation".to_string(),
                    snippet: "Extracted from visible image region".to_string(),
                    url: "https://example.com/photo-citation".to_string(),
                }],
            },
            ToolName::DataAnalysis => ToolResult::DataAnalysis {
                summary: format!(
                    "Data analysis summary for '{}'",
                    truncate_ascii(&req.query, 80)
                ),
                extracted_fields: vec![
                    ToolStructuredField {
                        key: "rows_analyzed".to_string(),
                        value: "128".to_string(),
                    },
                    ToolStructuredField {
                        key: "chart_hint".to_string(),
                        value: "line: revenue_over_time".to_string(),
                    },
                ],
                citations: vec![ToolTextSnippet {
                    title: "Data source segment".to_string(),
                    snippet: "Derived from uploaded table rows 1-128".to_string(),
                    url: "https://example.com/data-analysis-citation".to_string(),
                }],
            },
            ToolName::DeepResearch => ToolResult::DeepResearch {
                summary: format!(
                    "Deep research synthesis for '{}'",
                    truncate_ascii(&req.query, 80)
                ),
                extracted_fields: vec![
                    ToolStructuredField {
                        key: "scope".to_string(),
                        value: "multi-source synthesis".to_string(),
                    },
                    ToolStructuredField {
                        key: "confidence".to_string(),
                        value: "high".to_string(),
                    },
                ],
                citations: vec![
                    ToolTextSnippet {
                        title: "Primary source A".to_string(),
                        snippet: "Key finding from source A".to_string(),
                        url: "https://example.com/research-source-a".to_string(),
                    },
                    ToolTextSnippet {
                        title: "Primary source B".to_string(),
                        snippet: "Cross-check finding from source B".to_string(),
                        url: "https://example.com/research-source-b".to_string(),
                    },
                ],
            },
            ToolName::RecordMode => ToolResult::RecordMode {
                summary: format!(
                    "Recording summary for '{}'",
                    truncate_ascii(&req.query, 80)
                ),
                action_items: vec![
                    ToolStructuredField {
                        key: "action_item_1".to_string(),
                        value: "Draft follow-up summary by EOD".to_string(),
                    },
                    ToolStructuredField {
                        key: "action_item_2".to_string(),
                        value: "Share meeting decisions with finance".to_string(),
                    },
                ],
                evidence_refs: vec![
                    ToolStructuredField {
                        key: "chunk_001".to_string(),
                        value: "speaker=PM timecode=00:02:10-00:02:38".to_string(),
                    },
                    ToolStructuredField {
                        key: "chunk_004".to_string(),
                        value: "speaker=Ops timecode=00:11:05-00:11:42".to_string(),
                    },
                ],
            },
            ToolName::ConnectorQuery => {
                let (requested_scope, explicit_scope) = connector_scope_for_query(&req.query);
                let max_results =
                    usize::from(req.strict_budget.max_results.min(self.config.max_results));
                let returned_scope: Vec<&'static str> =
                    requested_scope.iter().copied().take(max_results).collect();
                let requested_scope_csv = requested_scope.join(",");
                let returned_scope_csv = returned_scope.join(",");
                let citations: Vec<ToolTextSnippet> = returned_scope
                    .iter()
                    .copied()
                    .enumerate()
                    .map(|(idx, connector)| connector_citation(connector, &req.query, idx))
                    .collect();

                ToolResult::ConnectorQuery {
                    summary: format!(
                        "Connector search summary for '{}' ({})",
                        truncate_ascii(&req.query, 80),
                        if explicit_scope {
                            "explicit connector scope"
                        } else {
                            "default connector scope"
                        }
                    ),
                    extracted_fields: vec![
                        ToolStructuredField {
                            key: "connector_scope".to_string(),
                            value: returned_scope_csv,
                        },
                        ToolStructuredField {
                            key: "connector_scope_requested".to_string(),
                            value: requested_scope_csv,
                        },
                        ToolStructuredField {
                            key: "scope_mode".to_string(),
                            value: if explicit_scope {
                                "explicit".to_string()
                            } else {
                                "default".to_string()
                            },
                        },
                        ToolStructuredField {
                            key: "matched_items".to_string(),
                            value: citations.len().to_string(),
                        },
                    ],
                    citations,
                }
            }
            ToolName::Other(_) => {
                return fail_response(
                    req,
                    reason_codes::E_FAIL_FORBIDDEN_TOOL,
                    CacheStatus::Bypassed,
                )
            }
        };

        let source_metadata = SourceMetadata {
            schema_version: selene_kernel_contracts::ph1e::PH1E_CONTRACT_VERSION,
            provider_hint: Some("ph1e_mock".to_string()),
            retrieved_at_unix_ms: 1_700_000_000_000,
            sources: source_refs_for_tool(
                &req.tool_name,
                &req.query,
                req.strict_budget.max_results.min(self.config.max_results),
            ),
        };

        match ToolResponse::ok_v1(
            req.request_id,
            req.query_hash,
            tool_result,
            source_metadata,
            None,
            reason_codes::E_OK_TOOL_RESULT,
            cache_status,
        ) {
            Ok(r) => r,
            Err(_) => fail_response(
                req,
                reason_codes::E_FAIL_INTERNAL_PIPELINE_ERROR,
                CacheStatus::Bypassed,
            ),
        }
    }
}

fn budget_exceeded(request_budget: StrictBudget, config: Ph1eConfig) -> bool {
    request_budget.timeout_ms > config.max_timeout_ms
        || request_budget.max_results > config.max_results
}

fn policy_blocks(req: &ToolRequest) -> bool {
    matches!(
        req.tool_name,
        ToolName::WebSearch | ToolName::News | ToolName::UrlFetchAndCite | ToolName::DeepResearch
    )
        && (req.policy_context_ref.privacy_mode
            || matches!(
                req.policy_context_ref.safety_tier,
                selene_kernel_contracts::ph1d::SafetyTier::Strict
            ))
}

fn connector_scope_policy_block(req: &ToolRequest) -> bool {
    if !matches!(req.tool_name, ToolName::ConnectorQuery) {
        return false;
    }
    let lower = req.query.to_ascii_lowercase();
    const UNSUPPORTED_CONNECTORS: &[&str] = &[
        "salesforce",
        "servicenow",
        "zendesk",
        "hubspot",
        "atlassian compass",
        "workday",
    ];
    UNSUPPORTED_CONNECTORS
        .iter()
        .any(|token| lower.contains(token))
}

fn forbidden_domain(req: &ToolRequest) -> bool {
    req.query.to_ascii_lowercase().contains("forbidden.example")
}

fn deterministic_timeout(req: &ToolRequest) -> bool {
    req.query.to_ascii_lowercase().contains("timeout")
}

fn cache_status_for_request(req: &ToolRequest) -> CacheStatus {
    match req.query_hash.0 % 3 {
        0 => CacheStatus::Hit,
        1 => CacheStatus::Miss,
        _ => CacheStatus::Bypassed,
    }
}

fn source_url_for_tool(tool_name: &ToolName) -> &'static str {
    match tool_name {
        ToolName::Time => "https://example.com/time",
        ToolName::Weather => "https://example.com/weather",
        ToolName::WebSearch => "https://example.com/search",
        ToolName::News => "https://example.com/news",
        ToolName::UrlFetchAndCite => "https://example.com/url-fetch",
        ToolName::DocumentUnderstand => "https://example.com/document",
        ToolName::PhotoUnderstand => "https://example.com/photo",
        ToolName::DataAnalysis => "https://example.com/data-analysis",
        ToolName::DeepResearch => "https://example.com/deep-research",
        ToolName::RecordMode => "recording://session/demo/chunk_001",
        ToolName::ConnectorQuery => "https://workspace.example.com/connectors",
        ToolName::Other(_) => "https://example.com",
    }
}

fn source_refs_for_tool(tool_name: &ToolName, query: &str, max_results: u8) -> Vec<SourceRef> {
    if matches!(tool_name, ToolName::ConnectorQuery) {
        let (scope, _) = connector_scope_for_query(query);
        let mut refs: Vec<SourceRef> = scope
            .into_iter()
            .take(usize::from(max_results))
            .map(connector_source_ref)
            .collect();
        if refs.is_empty() {
            refs.push(SourceRef {
                title: "Connector source".to_string(),
                url: "https://workspace.example.com/connectors".to_string(),
            });
        }
        return refs;
    }
    vec![SourceRef {
        title: "Deterministic PH1.E source".to_string(),
        url: source_url_for_tool(tool_name).to_string(),
    }]
}

fn connector_scope_for_query(query: &str) -> (Vec<&'static str>, bool) {
    let lower = query.to_ascii_lowercase();
    let mut out = Vec::new();
    for connector in [
        "gmail",
        "outlook",
        "calendar",
        "drive",
        "dropbox",
        "slack",
        "notion",
        "onedrive",
    ] {
        if connector_aliases(connector)
            .iter()
            .any(|alias| lower.contains(alias))
        {
            out.push(connector);
        }
    }
    let explicit_scope = !out.is_empty();
    if out.is_empty() {
        out.extend(["gmail", "calendar", "drive"]);
    }
    (out, explicit_scope)
}

fn connector_aliases(connector: &str) -> &'static [&'static str] {
    match connector {
        "gmail" => &["gmail", "google mail"],
        "outlook" => &["outlook", "exchange mail"],
        "calendar" => &["calendar", "gcal", "google calendar", "outlook calendar"],
        "drive" => &["drive", "google drive", "google docs", "google sheets"],
        "dropbox" => &["dropbox"],
        "slack" => &["slack"],
        "notion" => &["notion"],
        "onedrive" => &["onedrive", "one drive"],
        _ => &[],
    }
}

fn connector_source_ref(connector: &'static str) -> SourceRef {
    match connector {
        "gmail" => SourceRef {
            title: "Connector source: Gmail".to_string(),
            url: "https://workspace.example.com/gmail".to_string(),
        },
        "outlook" => SourceRef {
            title: "Connector source: Outlook".to_string(),
            url: "https://workspace.example.com/outlook".to_string(),
        },
        "calendar" => SourceRef {
            title: "Connector source: Calendar".to_string(),
            url: "https://workspace.example.com/calendar".to_string(),
        },
        "drive" => SourceRef {
            title: "Connector source: Drive".to_string(),
            url: "https://workspace.example.com/drive".to_string(),
        },
        "dropbox" => SourceRef {
            title: "Connector source: Dropbox".to_string(),
            url: "https://workspace.example.com/dropbox".to_string(),
        },
        "slack" => SourceRef {
            title: "Connector source: Slack".to_string(),
            url: "https://workspace.example.com/slack".to_string(),
        },
        "notion" => SourceRef {
            title: "Connector source: Notion".to_string(),
            url: "https://workspace.example.com/notion".to_string(),
        },
        "onedrive" => SourceRef {
            title: "Connector source: OneDrive".to_string(),
            url: "https://workspace.example.com/onedrive".to_string(),
        },
        _ => SourceRef {
            title: "Connector source".to_string(),
            url: "https://workspace.example.com/connectors".to_string(),
        },
    }
}

fn connector_citation(connector: &'static str, query: &str, idx: usize) -> ToolTextSnippet {
    let compact_query = truncate_ascii(query, 60);
    match connector {
        "gmail" => ToolTextSnippet {
            title: "Gmail thread result".to_string(),
            snippet: format!("Gmail match for '{compact_query}'"),
            url: format!("https://workspace.example.com/gmail/thread_{:03}", idx + 1),
        },
        "outlook" => ToolTextSnippet {
            title: "Outlook message result".to_string(),
            snippet: format!("Outlook match for '{compact_query}'"),
            url: format!("https://workspace.example.com/outlook/message_{:03}", idx + 1),
        },
        "calendar" => ToolTextSnippet {
            title: "Calendar event result".to_string(),
            snippet: format!("Calendar match for '{compact_query}'"),
            url: format!("https://workspace.example.com/calendar/event_{:03}", idx + 1),
        },
        "drive" => ToolTextSnippet {
            title: "Drive doc result".to_string(),
            snippet: format!("Drive match for '{compact_query}'"),
            url: format!("https://workspace.example.com/drive/doc_{:03}", idx + 1),
        },
        "dropbox" => ToolTextSnippet {
            title: "Dropbox file result".to_string(),
            snippet: format!("Dropbox match for '{compact_query}'"),
            url: format!("https://workspace.example.com/dropbox/file_{:03}", idx + 1),
        },
        "slack" => ToolTextSnippet {
            title: "Slack message result".to_string(),
            snippet: format!("Slack match for '{compact_query}'"),
            url: format!("https://workspace.example.com/slack/message_{:03}", idx + 1),
        },
        "notion" => ToolTextSnippet {
            title: "Notion page result".to_string(),
            snippet: format!("Notion match for '{compact_query}'"),
            url: format!("https://workspace.example.com/notion/page_{:03}", idx + 1),
        },
        "onedrive" => ToolTextSnippet {
            title: "OneDrive file result".to_string(),
            snippet: format!("OneDrive match for '{compact_query}'"),
            url: format!("https://workspace.example.com/onedrive/file_{:03}", idx + 1),
        },
        _ => ToolTextSnippet {
            title: "Connector result".to_string(),
            snippet: format!("Connector match for '{compact_query}'"),
            url: format!("https://workspace.example.com/connectors/item_{:03}", idx + 1),
        },
    }
}

fn truncate_ascii(input: &str, max_len: usize) -> String {
    input.chars().take(max_len).collect()
}

fn fail_response(req: &ToolRequest, code: ReasonCodeId, cache_status: CacheStatus) -> ToolResponse {
    ToolResponse::fail_v1(req.request_id, req.query_hash, code, cache_status)
        .expect("ToolResponse::fail_v1 must construct for bounded PH1.E failure output")
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1d::{PolicyContextRef, SafetyTier};
    use selene_kernel_contracts::ph1e::{ToolRequestOrigin, ToolStatus};

    fn req(tool_name: ToolName, query: &str, privacy_mode: bool, strict: bool) -> ToolRequest {
        req_with_budget(tool_name, query, privacy_mode, strict, 3)
    }

    fn req_with_budget(
        tool_name: ToolName,
        query: &str,
        privacy_mode: bool,
        strict: bool,
        max_results: u8,
    ) -> ToolRequest {
        ToolRequest::v1(
            ToolRequestOrigin::Ph1X,
            tool_name,
            query.to_string(),
            Some("en-US".to_string()),
            StrictBudget::new(1000, max_results).unwrap(),
            PolicyContextRef::v1(
                privacy_mode,
                false,
                if strict {
                    SafetyTier::Strict
                } else {
                    SafetyTier::Standard
                },
            ),
        )
        .unwrap()
    }

    #[test]
    fn at_e_01_policy_blocks_web_search_in_privacy_mode() {
        let rt = Ph1eRuntime::new(Ph1eConfig::mvp_v1());
        let out = rt.run(&req(ToolName::WebSearch, "selene", true, false));
        assert_eq!(out.tool_status, ToolStatus::Fail);
        assert_eq!(out.reason_code, reason_codes::E_FAIL_POLICY_BLOCK);
    }

    #[test]
    fn at_e_02_time_request_returns_ok_result() {
        let rt = Ph1eRuntime::new(Ph1eConfig::mvp_v1());
        let out = rt.run(&req(ToolName::Time, "what time", false, false));
        assert_eq!(out.tool_status, ToolStatus::Ok);
        assert!(out.tool_result.is_some());
        assert!(out.source_metadata.is_some());
        assert_eq!(out.reason_code, reason_codes::E_OK_TOOL_RESULT);
    }

    #[test]
    fn at_e_03_timeout_query_fails_closed() {
        let rt = Ph1eRuntime::new(Ph1eConfig::mvp_v1());
        let out = rt.run(&req(ToolName::Weather, "timeout in upstream", false, false));
        assert_eq!(out.tool_status, ToolStatus::Fail);
        assert_eq!(out.reason_code, reason_codes::E_FAIL_TIMEOUT);
    }

    #[test]
    fn at_e_04_forbidden_domain_fails_closed() {
        let rt = Ph1eRuntime::new(Ph1eConfig::mvp_v1());
        let out = rt.run(&req(
            ToolName::News,
            "site:forbidden.example update",
            false,
            false,
        ));
        assert_eq!(out.tool_status, ToolStatus::Fail);
        assert_eq!(out.reason_code, reason_codes::E_FAIL_FORBIDDEN_DOMAIN);
    }

    #[test]
    fn at_e_05_budget_exceeded_fails_closed() {
        let rt = Ph1eRuntime::new(Ph1eConfig {
            max_timeout_ms: 500,
            max_results: 2,
        });
        let req = ToolRequest::v1(
            ToolRequestOrigin::Ph1X,
            ToolName::Time,
            "now".to_string(),
            None,
            StrictBudget::new(1000, 3).unwrap(),
            PolicyContextRef::v1(false, false, SafetyTier::Standard),
        )
        .unwrap();
        let out = rt.run(&req);
        assert_eq!(out.tool_status, ToolStatus::Fail);
        assert_eq!(out.reason_code, reason_codes::E_FAIL_BUDGET_EXCEEDED);
    }

    #[test]
    fn at_e_06_url_fetch_and_cite_returns_citations_with_provenance() {
        let rt = Ph1eRuntime::new(Ph1eConfig::mvp_v1());
        let out = rt.run(&req(
            ToolName::UrlFetchAndCite,
            "open this URL and cite it: https://example.com/spec",
            false,
            false,
        ));
        assert_eq!(out.tool_status, ToolStatus::Ok);
        match out.tool_result.as_ref().expect("tool result required for ok") {
            ToolResult::UrlFetchAndCite { citations } => assert!(!citations.is_empty()),
            other => panic!("expected UrlFetchAndCite result, got {other:?}"),
        }
        let meta = out.source_metadata.as_ref().expect("source metadata required");
        assert!(!meta.sources.is_empty());
        assert!(meta.sources[0].url.contains("example.com"));
    }

    #[test]
    fn at_e_07_document_understand_returns_structured_fields_with_provenance() {
        let rt = Ph1eRuntime::new(Ph1eConfig::mvp_v1());
        let out = rt.run(&req(
            ToolName::DocumentUnderstand,
            "read this PDF and summarize it",
            false,
            false,
        ));
        assert_eq!(out.tool_status, ToolStatus::Ok);
        match out.tool_result.as_ref().expect("tool result required for ok") {
            ToolResult::DocumentUnderstand {
                summary,
                extracted_fields,
                citations,
            } => {
                assert!(!summary.trim().is_empty());
                assert!(!extracted_fields.is_empty());
                assert!(!citations.is_empty());
            }
            other => panic!("expected DocumentUnderstand result, got {other:?}"),
        }
        let meta = out.source_metadata.as_ref().expect("source metadata required");
        assert!(!meta.sources.is_empty());
        assert!(meta.sources[0].url.contains("example.com"));
    }

    #[test]
    fn at_e_08_photo_understand_returns_structured_fields_with_provenance() {
        let rt = Ph1eRuntime::new(Ph1eConfig::mvp_v1());
        let out = rt.run(&req(
            ToolName::PhotoUnderstand,
            "what does this screenshot say?",
            false,
            false,
        ));
        assert_eq!(out.tool_status, ToolStatus::Ok);
        match out.tool_result.as_ref().expect("tool result required for ok") {
            ToolResult::PhotoUnderstand {
                summary,
                extracted_fields,
                citations,
            } => {
                assert!(!summary.trim().is_empty());
                assert!(!extracted_fields.is_empty());
                assert!(!citations.is_empty());
            }
            other => panic!("expected PhotoUnderstand result, got {other:?}"),
        }
        let meta = out.source_metadata.as_ref().expect("source metadata required");
        assert!(!meta.sources.is_empty());
        assert!(meta.sources[0].url.contains("example.com"));
    }

    #[test]
    fn at_e_09_data_analysis_returns_structured_fields_with_provenance() {
        let rt = Ph1eRuntime::new(Ph1eConfig::mvp_v1());
        let out = rt.run(&req(
            ToolName::DataAnalysis,
            "analyze this csv and show summary stats",
            false,
            false,
        ));
        assert_eq!(out.tool_status, ToolStatus::Ok);
        match out.tool_result.as_ref().expect("tool result required for ok") {
            ToolResult::DataAnalysis {
                summary,
                extracted_fields,
                citations,
            } => {
                assert!(!summary.trim().is_empty());
                assert!(!extracted_fields.is_empty());
                assert!(!citations.is_empty());
            }
            other => panic!("expected DataAnalysis result, got {other:?}"),
        }
        let meta = out.source_metadata.as_ref().expect("source metadata required");
        assert!(!meta.sources.is_empty());
        assert!(meta.sources[0].url.contains("example.com"));
    }

    #[test]
    fn at_e_10_deep_research_returns_structured_fields_with_provenance() {
        let rt = Ph1eRuntime::new(Ph1eConfig::mvp_v1());
        let out = rt.run(&req(
            ToolName::DeepResearch,
            "deep research AI chip policy changes with citations",
            false,
            false,
        ));
        assert_eq!(out.tool_status, ToolStatus::Ok);
        match out.tool_result.as_ref().expect("tool result required for ok") {
            ToolResult::DeepResearch {
                summary,
                extracted_fields,
                citations,
            } => {
                assert!(!summary.trim().is_empty());
                assert!(!extracted_fields.is_empty());
                assert!(!citations.is_empty());
            }
            other => panic!("expected DeepResearch result, got {other:?}"),
        }
        let meta = out.source_metadata.as_ref().expect("source metadata required");
        assert!(!meta.sources.is_empty());
        assert!(meta.sources[0].url.contains("example.com"));
    }

    #[test]
    fn at_e_11_record_mode_returns_recording_evidence_with_provenance() {
        let rt = Ph1eRuntime::new(Ph1eConfig::mvp_v1());
        let out = rt.run(&req(
            ToolName::RecordMode,
            "summarize this meeting recording and list action items",
            false,
            false,
        ));
        assert_eq!(out.tool_status, ToolStatus::Ok);
        match out.tool_result.as_ref().expect("tool result required for ok") {
            ToolResult::RecordMode {
                summary,
                action_items,
                evidence_refs,
            } => {
                assert!(!summary.trim().is_empty());
                assert!(!action_items.is_empty());
                assert!(!evidence_refs.is_empty());
            }
            other => panic!("expected RecordMode result, got {other:?}"),
        }
        let meta = out.source_metadata.as_ref().expect("source metadata required");
        assert!(!meta.sources.is_empty());
        assert!(meta.sources[0].url.starts_with("recording://"));
    }

    #[test]
    fn at_e_12_connector_query_returns_structured_fields_with_provenance() {
        let rt = Ph1eRuntime::new(Ph1eConfig::mvp_v1());
        let out = rt.run(&req(
            ToolName::ConnectorQuery,
            "search connectors for q3 roadmap notes in gmail and drive",
            false,
            false,
        ));
        assert_eq!(out.tool_status, ToolStatus::Ok);
        match out.tool_result.as_ref().expect("tool result required for ok") {
            ToolResult::ConnectorQuery {
                summary,
                extracted_fields,
                citations,
            } => {
                assert!(!summary.trim().is_empty());
                assert!(!extracted_fields.is_empty());
                assert!(!citations.is_empty());
                let scope = extracted_fields
                    .iter()
                    .find(|field| field.key == "connector_scope")
                    .map(|field| field.value.as_str())
                    .expect("connector_scope field missing");
                assert!(scope.contains("gmail"));
                assert!(scope.contains("drive"));
                assert!(!scope.contains("calendar"));
                assert!(citations.iter().all(|item| {
                    item.url.contains("/gmail/") || item.url.contains("/drive/")
                }));
            }
            other => panic!("expected ConnectorQuery result, got {other:?}"),
        }
        let meta = out.source_metadata.as_ref().expect("source metadata required");
        assert!(!meta.sources.is_empty());
        assert!(meta.sources.iter().any(|s| s.url.contains("/gmail")));
        assert!(meta.sources.iter().any(|s| s.url.contains("/drive")));
    }

    #[test]
    fn at_e_13_connector_query_defaults_scope_when_none_is_requested() {
        let rt = Ph1eRuntime::new(Ph1eConfig::mvp_v1());
        let out = rt.run(&req(
            ToolName::ConnectorQuery,
            "search connectors for onboarding checklist notes",
            false,
            false,
        ));
        assert_eq!(out.tool_status, ToolStatus::Ok);
        match out.tool_result.as_ref().expect("tool result required for ok") {
            ToolResult::ConnectorQuery {
                extracted_fields,
                citations,
                ..
            } => {
                let scope = extracted_fields
                    .iter()
                    .find(|field| field.key == "connector_scope")
                    .map(|field| field.value.as_str())
                    .expect("connector_scope field missing");
                assert_eq!(scope, "gmail,calendar,drive");
                let mode = extracted_fields
                    .iter()
                    .find(|field| field.key == "scope_mode")
                    .map(|field| field.value.as_str())
                    .expect("scope_mode field missing");
                assert_eq!(mode, "default");
                assert_eq!(citations.len(), 3);
            }
            other => panic!("expected ConnectorQuery result, got {other:?}"),
        }
    }

    #[test]
    fn at_e_14_connector_query_respects_budget_and_scope_limit() {
        let rt = Ph1eRuntime::new(Ph1eConfig::mvp_v1());
        let out = rt.run(&req_with_budget(
            ToolName::ConnectorQuery,
            "search slack and notion for infra postmortems",
            false,
            false,
            1,
        ));
        assert_eq!(out.tool_status, ToolStatus::Ok);
        match out.tool_result.as_ref().expect("tool result required for ok") {
            ToolResult::ConnectorQuery {
                extracted_fields,
                citations,
                ..
            } => {
                assert_eq!(citations.len(), 1);
                let requested = extracted_fields
                    .iter()
                    .find(|field| field.key == "connector_scope_requested")
                    .map(|field| field.value.as_str())
                    .expect("connector_scope_requested field missing");
                assert!(requested.contains("slack"));
                assert!(requested.contains("notion"));
                let returned = extracted_fields
                    .iter()
                    .find(|field| field.key == "connector_scope")
                    .map(|field| field.value.as_str())
                    .expect("connector_scope field missing");
                assert_eq!(returned, "slack");
                assert!(citations[0].url.contains("/slack/"));
            }
            other => panic!("expected ConnectorQuery result, got {other:?}"),
        }
    }

    #[test]
    fn at_e_15_connector_query_unsupported_scope_fails_closed() {
        let rt = Ph1eRuntime::new(Ph1eConfig::mvp_v1());
        let out = rt.run(&req(
            ToolName::ConnectorQuery,
            "search salesforce for renewal notes",
            false,
            false,
        ));
        assert_eq!(out.tool_status, ToolStatus::Fail);
        assert_eq!(out.reason_code, reason_codes::E_FAIL_POLICY_BLOCK);
    }
}
