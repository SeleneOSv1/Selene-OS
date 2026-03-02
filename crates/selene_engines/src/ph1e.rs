#![forbid(unsafe_code)]

use std::env;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::device_vault;
use selene_kernel_contracts::ph1e::{
    CacheStatus, SourceMetadata, SourceRef, StrictBudget, ToolName, ToolRequest, ToolResponse,
    ToolResult, ToolStructuredField, ToolTextSnippet,
};
use selene_kernel_contracts::provider_secrets::ProviderSecretId;
use selene_kernel_contracts::{ReasonCodeId, Validate};
use serde_json::Value;

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.E reason-code namespace. Values are placeholders until global registry lock.
    pub const E_OK_TOOL_RESULT: ReasonCodeId = ReasonCodeId(0x4500_0001);

    pub const E_FAIL_TIMEOUT: ReasonCodeId = ReasonCodeId(0x4500_00F1);
    pub const E_FAIL_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x4500_00F2);
    pub const E_FAIL_FORBIDDEN_TOOL: ReasonCodeId = ReasonCodeId(0x4500_00F3);
    pub const E_FAIL_POLICY_BLOCK: ReasonCodeId = ReasonCodeId(0x4500_00F4);
    pub const E_FAIL_FORBIDDEN_DOMAIN: ReasonCodeId = ReasonCodeId(0x4500_00F5);
    pub const E_FAIL_PROVIDER_MISSING_CONFIG: ReasonCodeId = ReasonCodeId(0x4500_00F6);
    pub const E_FAIL_PROVIDER_UPSTREAM: ReasonCodeId = ReasonCodeId(0x4500_00F7);
    pub const E_FAIL_QUERY_PARSE: ReasonCodeId = ReasonCodeId(0x4500_00F8);
    pub const E_FAIL_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4500_00FF);
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ToolFailPayload {
    reason_code: ReasonCodeId,
    fail_detail: Option<String>,
}

impl ToolFailPayload {
    fn new(reason_code: ReasonCodeId) -> Self {
        Self {
            reason_code,
            fail_detail: None,
        }
    }

    fn with_detail(reason_code: ReasonCodeId, detail: String) -> Self {
        Self {
            reason_code,
            fail_detail: Some(detail),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProviderCallError {
    provider: &'static str,
    http_status: Option<u16>,
    error_kind: &'static str,
}

impl ProviderCallError {
    fn new(provider: &'static str, error_kind: &'static str, http_status: Option<u16>) -> Self {
        Self {
            provider,
            http_status,
            error_kind,
        }
    }

    fn safe_detail(&self) -> String {
        match self.http_status {
            Some(status) => format!(
                "provider={} error={} status={}",
                self.provider, self.error_kind, status
            ),
            None => format!("provider={} error={}", self.provider, self.error_kind),
        }
    }
}

const STARTUP_CONNECTIVITY_TIMEOUT_MS: u32 = 2_000;
const BRAVE_CONNECTIVITY_PROBE_URL: &str = "https://api.search.brave.com/";
const OPENAI_CONNECTIVITY_PROBE_URL: &str = "https://api.openai.com/";
const CLASH_EXPLICIT_HINT: &str =
    "If using Clash, set SELENE_PROXY_MODE=explicit and SELENE_HTTP_PROXY_URL=http://127.0.0.1:<port>";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ph1eProxyMode {
    Off,
    Env,
    Explicit,
}

impl Ph1eProxyMode {
    fn from_env_value(raw: Option<String>) -> Self {
        match raw
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("off") => Self::Off,
            Some("explicit") => Self::Explicit,
            _ => Self::Env,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Env => "env",
            Self::Explicit => "explicit",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1eProxyConfig {
    pub mode: Ph1eProxyMode,
    pub http_proxy_url: Option<String>,
    pub https_proxy_url: Option<String>,
}

impl Ph1eProxyConfig {
    pub fn from_env() -> Self {
        Self {
            mode: Ph1eProxyMode::from_env_value(env::var("SELENE_PROXY_MODE").ok()),
            http_proxy_url: env::var("SELENE_HTTP_PROXY_URL").ok(),
            https_proxy_url: env::var("SELENE_HTTPS_PROXY_URL").ok(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedProxyConfig {
    mode: Ph1eProxyMode,
    http_proxy_url: Option<String>,
    https_proxy_url: Option<String>,
    effective_proxy_url: Option<String>,
}

impl ResolvedProxyConfig {
    fn safe_proxy_host_port(&self) -> Option<String> {
        self.effective_proxy_url
            .as_ref()
            .and_then(|url| proxy_host_port_hint(url))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboundSelfCheckFailure {
    pub provider: &'static str,
    pub endpoint: &'static str,
    pub proxy_mode: Ph1eProxyMode,
    pub proxy_host_port: Option<String>,
    pub error_kind: &'static str,
}

impl OutboundSelfCheckFailure {
    pub fn safe_log_line(&self) -> String {
        let mut out = format!(
            "selene_adapter_http outbound self-check failed provider={} endpoint={} proxy_mode={} error={}",
            self.provider,
            self.endpoint,
            self.proxy_mode.as_str(),
            self.error_kind,
        );
        if let Some(proxy) = self.proxy_host_port.as_deref() {
            out.push_str(&format!(" proxy={proxy}"));
        }
        out
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1eProviderConfig {
    pub brave_api_key: Option<String>,
    pub brave_web_url: String,
    pub brave_news_url: String,
    pub openai_api_key: Option<String>,
    pub openai_responses_url: String,
    pub openai_model: String,
    pub user_agent: String,
    pub proxy_config: Ph1eProxyConfig,
    pub brave_web_fixture_json: Option<String>,
    pub brave_news_fixture_json: Option<String>,
    pub url_fetch_fixture_html: Option<String>,
}

impl Ph1eProviderConfig {
    pub fn from_env() -> Self {
        Self {
            // Secrets are resolved from the encrypted local device vault at runtime.
            brave_api_key: None,
            brave_web_url: env::var("BRAVE_SEARCH_WEB_URL")
                .unwrap_or_else(|_| "https://api.search.brave.com/res/v1/web/search".to_string()),
            brave_news_url: env::var("BRAVE_SEARCH_NEWS_URL")
                .unwrap_or_else(|_| "https://api.search.brave.com/res/v1/news/search".to_string()),
            openai_api_key: None,
            openai_responses_url: env::var("OPENAI_RESPONSES_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1/responses".to_string()),
            openai_model: env::var("OPENAI_WEB_FALLBACK_MODEL")
                .unwrap_or_else(|_| "gpt-4o-mini".to_string()),
            user_agent: env::var("PH1E_HTTP_USER_AGENT")
                .unwrap_or_else(|_| "selene-ph1e/1.0".to_string()),
            proxy_config: Ph1eProxyConfig::from_env(),
            brave_web_fixture_json: None,
            brave_news_fixture_json: None,
            url_fetch_fixture_html: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1eRuntime {
    config: Ph1eConfig,
    provider_config: Ph1eProviderConfig,
}

impl Ph1eRuntime {
    pub fn new(config: Ph1eConfig) -> Self {
        Self::new_with_provider_config(config, Ph1eProviderConfig::from_env())
    }

    pub fn new_with_provider_config(
        config: Ph1eConfig,
        provider_config: Ph1eProviderConfig,
    ) -> Self {
        Self {
            config,
            provider_config,
        }
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
        let (tool_result, source_metadata) = match &req.tool_name {
            ToolName::Time => ToolResult::Time {
                local_time_iso: "2026-01-01T00:00:00Z".to_string(),
            }
            .with_default_source_metadata(
                &req.tool_name,
                &req.query,
                req.strict_budget.max_results.min(self.config.max_results),
            ),
            ToolName::Weather => ToolResult::Weather {
                summary: format!("Weather snapshot for {}", req.query),
            }
            .with_default_source_metadata(
                &req.tool_name,
                &req.query,
                req.strict_budget.max_results.min(self.config.max_results),
            ),
            ToolName::WebSearch => match self.run_web_search(req) {
                Ok(ok) => ok,
                Err(fail) => {
                    return fail_response_with_detail(
                        req,
                        fail.reason_code,
                        cache_status,
                        fail.fail_detail.as_deref(),
                    )
                }
            },
            ToolName::News => match self.run_news_search(req) {
                Ok(ok) => ok,
                Err(fail) => {
                    return fail_response_with_detail(
                        req,
                        fail.reason_code,
                        cache_status,
                        fail.fail_detail.as_deref(),
                    )
                }
            },
            ToolName::UrlFetchAndCite => match self.run_url_fetch_and_cite(req) {
                Ok(ok) => ok,
                Err(code) => return fail_response(req, code, cache_status),
            },
            ToolName::DocumentUnderstand => ToolResult::DocumentUnderstand {
                summary: format!("Document summary for '{}'", truncate_ascii(&req.query, 80)),
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
                    url: "https://docs.selene.ai/document-citation".to_string(),
                }],
            }
            .with_default_source_metadata(
                &req.tool_name,
                &req.query,
                req.strict_budget.max_results.min(self.config.max_results),
            ),
            ToolName::PhotoUnderstand => ToolResult::PhotoUnderstand {
                summary: format!("Photo summary for '{}'", truncate_ascii(&req.query, 80)),
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
                    url: "https://docs.selene.ai/photo-citation".to_string(),
                }],
            }
            .with_default_source_metadata(
                &req.tool_name,
                &req.query,
                req.strict_budget.max_results.min(self.config.max_results),
            ),
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
                    url: "https://docs.selene.ai/data-analysis-citation".to_string(),
                }],
            }
            .with_default_source_metadata(
                &req.tool_name,
                &req.query,
                req.strict_budget.max_results.min(self.config.max_results),
            ),
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
                        url: "https://research.selene.ai/source-a".to_string(),
                    },
                    ToolTextSnippet {
                        title: "Primary source B".to_string(),
                        snippet: "Cross-check finding from source B".to_string(),
                        url: "https://research.selene.ai/source-b".to_string(),
                    },
                ],
            }
            .with_default_source_metadata(
                &req.tool_name,
                &req.query,
                req.strict_budget.max_results.min(self.config.max_results),
            ),
            ToolName::RecordMode => ToolResult::RecordMode {
                summary: format!("Recording summary for '{}'", truncate_ascii(&req.query, 80)),
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
            }
            .with_default_source_metadata(
                &req.tool_name,
                &req.query,
                req.strict_budget.max_results.min(self.config.max_results),
            ),
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
                .with_default_source_metadata(
                    &req.tool_name,
                    &req.query,
                    req.strict_budget.max_results.min(self.config.max_results),
                )
            }
            ToolName::Other(_) => {
                return fail_response(
                    req,
                    reason_codes::E_FAIL_FORBIDDEN_TOOL,
                    CacheStatus::Bypassed,
                )
            }
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

    fn run_web_search(
        &self,
        req: &ToolRequest,
    ) -> Result<(ToolResult, SourceMetadata), ToolFailPayload> {
        self.run_live_search(req, ToolName::WebSearch)
    }

    fn run_news_search(
        &self,
        req: &ToolRequest,
    ) -> Result<(ToolResult, SourceMetadata), ToolFailPayload> {
        self.run_live_search(req, ToolName::News)
    }

    fn resolve_secret_from_vault(&self, secret_id: ProviderSecretId) -> Option<String> {
        match device_vault::resolve_secret(secret_id.as_str()) {
            Ok(Some(secret)) => trim_non_empty(secret),
            _ => None,
        }
    }

    fn resolve_brave_api_key(&self) -> Option<String> {
        self.provider_config
            .brave_api_key
            .clone()
            .and_then(trim_non_empty)
            .or_else(|| self.resolve_secret_from_vault(ProviderSecretId::BraveSearchApiKey))
    }

    fn resolve_openai_api_key(&self) -> Option<String> {
        self.provider_config
            .openai_api_key
            .clone()
            .and_then(trim_non_empty)
            .or_else(|| self.resolve_secret_from_vault(ProviderSecretId::OpenAIApiKey))
    }

    fn brave_fixture_json_for(&self, is_news: bool) -> Option<&str> {
        if is_news {
            self.provider_config.brave_news_fixture_json.as_deref()
        } else {
            self.provider_config.brave_web_fixture_json.as_deref()
        }
    }

    fn url_fetch_fixture_html(&self) -> Option<&str> {
        self.provider_config.url_fetch_fixture_html.as_deref()
    }

    fn run_live_search(
        &self,
        req: &ToolRequest,
        kind: ToolName,
    ) -> Result<(ToolResult, SourceMetadata), ToolFailPayload> {
        let max_results = req.strict_budget.max_results.min(self.config.max_results);
        let brave_api_key = self.resolve_brave_api_key();
        let openai_api_key = self.resolve_openai_api_key();
        if brave_api_key.is_none() && openai_api_key.is_none() {
            return Err(ToolFailPayload::new(
                reason_codes::E_FAIL_PROVIDER_MISSING_CONFIG,
            ));
        }

        let mut brave_failure: Option<ProviderCallError> = None;
        if let Some(brave_key) = brave_api_key.as_deref() {
            let url = if matches!(kind, ToolName::News) {
                &self.provider_config.brave_news_url
            } else {
                &self.provider_config.brave_web_url
            };
            let brave_response = run_brave_search(
                url,
                brave_key,
                &req.query,
                max_results,
                req.strict_budget.timeout_ms,
                &self.provider_config.user_agent,
                &self.provider_config.proxy_config,
                matches!(kind, ToolName::News),
                self.brave_fixture_json_for(matches!(kind, ToolName::News)),
            );

            match brave_response {
                Ok((items, sources)) => {
                    return Ok((
                        if matches!(kind, ToolName::News) {
                            ToolResult::News { items }
                        } else {
                            ToolResult::WebSearch { items }
                        },
                        source_metadata_from_live(
                            Some("ph1search_brave".to_string()),
                            now_unix_ms(),
                            sources,
                        ),
                    ));
                }
                Err(err) => {
                    brave_failure = Some(err);
                }
            }
        }

        if let Some(openai_key) = openai_api_key.as_deref() {
            match run_openai_search_fallback(
                &self.provider_config.openai_responses_url,
                openai_key,
                &self.provider_config.openai_model,
                &req.query,
                max_results,
                req.strict_budget.timeout_ms,
                &self.provider_config.user_agent,
                &self.provider_config.proxy_config,
                matches!(kind, ToolName::News),
            ) {
                Ok((items, sources)) => Ok((
                    if matches!(kind, ToolName::News) {
                        ToolResult::News { items }
                    } else {
                        ToolResult::WebSearch { items }
                    },
                    source_metadata_from_live(
                        Some("ph1search_openai_fallback".to_string()),
                        now_unix_ms(),
                        sources,
                    ),
                )),
                Err(openai_failure) => Err(ToolFailPayload::with_detail(
                    reason_codes::E_FAIL_PROVIDER_UPSTREAM,
                    combine_live_search_failure_detail(
                        brave_failure.as_ref(),
                        Some(&openai_failure),
                        &self.provider_config.proxy_config,
                    ),
                )),
            }
        } else if let Some(brave_failure) = brave_failure {
            Err(ToolFailPayload::with_detail(
                reason_codes::E_FAIL_PROVIDER_UPSTREAM,
                combine_live_search_failure_detail(
                    Some(&brave_failure),
                    None,
                    &self.provider_config.proxy_config,
                ),
            ))
        } else {
            Err(ToolFailPayload::new(reason_codes::E_FAIL_PROVIDER_UPSTREAM))
        }
    }

    fn run_url_fetch_and_cite(
        &self,
        req: &ToolRequest,
    ) -> Result<(ToolResult, SourceMetadata), ReasonCodeId> {
        let url = first_http_url_in_text(&req.query).ok_or(reason_codes::E_FAIL_QUERY_PARSE)?;
        let fetched = run_url_fetch_citation(
            &url,
            req.strict_budget.max_results.min(self.config.max_results),
            req.strict_budget.timeout_ms,
            &self.provider_config.user_agent,
            &self.provider_config.proxy_config,
            self.url_fetch_fixture_html(),
        );
        let (citations, sources) = fetched.map_err(|_| reason_codes::E_FAIL_PROVIDER_UPSTREAM)?;
        Ok((
            ToolResult::UrlFetchAndCite { citations },
            source_metadata_from_live(
                Some("ph1search_url_fetch".to_string()),
                now_unix_ms(),
                sources,
            ),
        ))
    }
}

pub fn startup_outbound_self_check_logs() -> Vec<String> {
    let provider_config = Ph1eProviderConfig::from_env();
    run_startup_outbound_self_check_with_probe(
        &provider_config,
        probe_provider_connectivity,
    )
    .into_iter()
    .map(|failure| failure.safe_log_line())
    .collect()
}

fn run_startup_outbound_self_check_with_probe<F>(
    provider_config: &Ph1eProviderConfig,
    mut probe: F,
) -> Vec<OutboundSelfCheckFailure>
where
    F: FnMut(
        &'static str,
        &'static str,
        u32,
        &str,
        &Ph1eProxyConfig,
    ) -> Result<(), ProviderCallError>,
{
    let mut failures = Vec::new();
    for (provider, endpoint) in [
        ("brave", BRAVE_CONNECTIVITY_PROBE_URL),
        ("openai", OPENAI_CONNECTIVITY_PROBE_URL),
    ] {
        if let Err(err) = probe(
            provider,
            endpoint,
            STARTUP_CONNECTIVITY_TIMEOUT_MS,
            &provider_config.user_agent,
            &provider_config.proxy_config,
        ) {
            failures.push(OutboundSelfCheckFailure {
                provider,
                endpoint,
                proxy_mode: provider_config.proxy_config.mode,
                proxy_host_port: resolve_proxy_config(&provider_config.proxy_config)
                    .ok()
                    .and_then(|resolved| resolved.safe_proxy_host_port()),
                error_kind: err.error_kind,
            });
        }
    }
    failures
}

fn probe_provider_connectivity(
    provider: &'static str,
    endpoint: &'static str,
    timeout_ms: u32,
    user_agent: &str,
    proxy_config: &Ph1eProxyConfig,
) -> Result<(), ProviderCallError> {
    let agent = build_http_agent(timeout_ms, user_agent, proxy_config)
        .map_err(|_| ProviderCallError::new(provider, "config_invalid", None))?;
    match agent.head(endpoint).call() {
        Ok(_) => Ok(()),
        Err(ureq::Error::Status(_, _)) => Ok(()),
        Err(ureq::Error::Transport(transport)) => Err(provider_error_from_transport(
            provider, transport,
        )),
    }
}

trait ToolResultWithSource {
    fn with_default_source_metadata(
        self,
        tool_name: &ToolName,
        query: &str,
        max_results: u8,
    ) -> (ToolResult, SourceMetadata);
}

impl ToolResultWithSource for ToolResult {
    fn with_default_source_metadata(
        self,
        tool_name: &ToolName,
        query: &str,
        max_results: u8,
    ) -> (ToolResult, SourceMetadata) {
        (
            self,
            SourceMetadata {
                schema_version: selene_kernel_contracts::ph1e::PH1E_CONTRACT_VERSION,
                provider_hint: Some("ph1e_builtin".to_string()),
                retrieved_at_unix_ms: now_unix_ms(),
                sources: source_refs_for_tool(tool_name, query, max_results),
            },
        )
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
    ) && (req.policy_context_ref.privacy_mode
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
        ToolName::Time => "https://worldtimeapi.org/",
        ToolName::Weather => "https://api.open-meteo.com/",
        ToolName::WebSearch => "https://search.brave.com/",
        ToolName::News => "https://search.brave.com/news",
        ToolName::UrlFetchAndCite => "https://www.iana.org/domains/reserved",
        ToolName::DocumentUnderstand => "https://docs.selene.local/document-understand",
        ToolName::PhotoUnderstand => "https://docs.selene.local/photo-understand",
        ToolName::DataAnalysis => "https://docs.selene.local/data-analysis",
        ToolName::DeepResearch => "https://docs.selene.local/deep-research",
        ToolName::RecordMode => "recording://session/demo/chunk_001",
        ToolName::ConnectorQuery => "https://workspace.selene.local/connectors",
        ToolName::Other(_) => "https://docs.selene.local/tool",
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
                url: "https://workspace.selene.local/connectors".to_string(),
            });
        }
        return refs;
    }
    vec![SourceRef {
        title: "Deterministic PH1.E source".to_string(),
        url: source_url_for_tool(tool_name).to_string(),
    }]
}

fn source_metadata_from_live(
    provider_hint: Option<String>,
    retrieved_at_unix_ms: u64,
    sources: Vec<SourceRef>,
) -> SourceMetadata {
    SourceMetadata {
        schema_version: selene_kernel_contracts::ph1e::PH1E_CONTRACT_VERSION,
        provider_hint,
        retrieved_at_unix_ms,
        sources,
    }
}

fn resolve_proxy_config(proxy_config: &Ph1eProxyConfig) -> Result<ResolvedProxyConfig, String> {
    let mode = proxy_config.mode;
    let (http_proxy_url, https_proxy_url) = match mode {
        Ph1eProxyMode::Off => (None, None),
        Ph1eProxyMode::Env => (
            env::var("HTTP_PROXY").ok().and_then(trim_non_empty),
            env::var("HTTPS_PROXY").ok().and_then(trim_non_empty),
        ),
        Ph1eProxyMode::Explicit => (
            proxy_config
                .http_proxy_url
                .clone()
                .and_then(trim_non_empty),
            proxy_config
                .https_proxy_url
                .clone()
                .and_then(trim_non_empty),
        ),
    };

    if matches!(mode, Ph1eProxyMode::Explicit) && (http_proxy_url.is_none() || https_proxy_url.is_none()) {
        return Err(
            "explicit proxy mode requires SELENE_HTTP_PROXY_URL and SELENE_HTTPS_PROXY_URL"
                .to_string(),
        );
    }

    let effective_proxy_url = https_proxy_url.clone().or_else(|| http_proxy_url.clone());
    Ok(ResolvedProxyConfig {
        mode,
        http_proxy_url,
        https_proxy_url,
        effective_proxy_url,
    })
}

fn proxy_host_port_hint(raw_proxy_url: &str) -> Option<String> {
    let trimmed = raw_proxy_url.trim();
    if trimmed.is_empty() {
        return None;
    }
    let without_scheme = trimmed
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(trimmed);
    let without_auth = without_scheme
        .rsplit_once('@')
        .map(|(_, rest)| rest)
        .unwrap_or(without_scheme);
    let host_port = without_auth
        .split(['/', '?', '#'])
        .next()
        .map(str::trim)
        .filter(|v| !v.is_empty())?;
    Some(host_port.to_string())
}

fn proxy_hint_for_failures(
    brave_failure: Option<&ProviderCallError>,
    openai_failure: Option<&ProviderCallError>,
    proxy_config: &Ph1eProxyConfig,
) -> Option<String> {
    let kind = brave_failure
        .map(|err| err.error_kind)
        .or_else(|| openai_failure.map(|err| err.error_kind))?;
    if !matches!(kind, "connection" | "tls" | "dns" | "config_invalid") {
        return None;
    }

    let mode = proxy_config.mode.as_str();
    let proxy_host_port = resolve_proxy_config(proxy_config)
        .ok()
        .and_then(|resolved| resolved.safe_proxy_host_port());
    let mut out = format!("proxy_mode={mode}");
    if let Some(proxy) = proxy_host_port.as_deref() {
        out.push_str(&format!(" proxy={proxy}"));
    }
    out.push_str(&format!(" hint={CLASH_EXPLICIT_HINT}"));
    Some(out)
}

fn run_brave_search(
    endpoint: &str,
    api_key: &str,
    query: &str,
    max_results: u8,
    timeout_ms: u32,
    user_agent: &str,
    proxy_config: &Ph1eProxyConfig,
    is_news: bool,
    fixture_json: Option<&str>,
) -> Result<(Vec<ToolTextSnippet>, Vec<SourceRef>), ProviderCallError> {
    let body: Value = if let Some(fixture) = fixture_json {
        serde_json::from_str(fixture)
            .map_err(|_| ProviderCallError::new("brave", "json_parse", None))?
    } else {
        let agent = build_http_agent(timeout_ms, user_agent, proxy_config)
            .map_err(|_| ProviderCallError::new("brave", "config_invalid", None))?;
        let response = agent
            .get(endpoint)
            .set("Accept", "application/json")
            .set("X-Subscription-Token", api_key)
            .query("q", query)
            .query("count", &max_results.to_string())
            .call()
            .map_err(|e| provider_error_from_ureq("brave", e))?;
        serde_json::from_reader(response.into_reader())
            .map_err(|_| ProviderCallError::new("brave", "json_parse", None))?
    };

    let items = extract_tool_snippets(&body, usize::from(max_results), is_news)
        .ok_or_else(|| ProviderCallError::new("brave", "empty_results", None))?;
    let sources = items_to_sources(&items);
    Ok((items, sources))
}

fn run_openai_search_fallback(
    endpoint: &str,
    api_key: &str,
    model: &str,
    query: &str,
    max_results: u8,
    timeout_ms: u32,
    user_agent: &str,
    proxy_config: &Ph1eProxyConfig,
    is_news: bool,
) -> Result<(Vec<ToolTextSnippet>, Vec<SourceRef>), ProviderCallError> {
    let prompt = if is_news {
        format!(
            "Return JSON with key 'results' as an array of objects (title,url,snippet) for the latest news query: {query}. Limit {max_results} results."
        )
    } else {
        format!(
            "Return JSON with key 'results' as an array of objects (title,url,snippet) for web search query: {query}. Limit {max_results} results."
        )
    };

    let mut payload = serde_json::json!({
        "model": model,
        "input": prompt,
        "temperature": 0,
        "max_output_tokens": 800,
        "tools": [{"type": "web_search"}],
    });
    let agent = build_http_agent(timeout_ms, user_agent, proxy_config)
        .map_err(|_| ProviderCallError::new("openai", "config_invalid", None))?;
    let response = post_json(agent.clone(), endpoint, api_key, &payload).or_else(|_| {
        // Fallback when the account/model does not support tool invocation.
        if let Some(obj) = payload.as_object_mut() {
            obj.remove("tools");
        }
        post_json(agent, endpoint, api_key, &payload)
    })?;

    let items = extract_tool_snippets(&response, usize::from(max_results), false)
        .or_else(|| extract_openai_results(&response, usize::from(max_results)))
        .ok_or_else(|| ProviderCallError::new("openai", "empty_results", None))?;
    if items.is_empty() {
        return Err(ProviderCallError::new("openai", "empty_results", None));
    }
    let sources = items_to_sources(&items);
    Ok((items, sources))
}

fn post_json(
    agent: ureq::Agent,
    endpoint: &str,
    api_key: &str,
    payload: &Value,
) -> Result<Value, ProviderCallError> {
    let response = agent
        .post(endpoint)
        .set("Content-Type", "application/json")
        .set("Authorization", &format!("Bearer {api_key}"))
        .set("Accept", "application/json")
        .send_json(payload.clone())
        .map_err(|e| provider_error_from_ureq("openai", e))?;
    serde_json::from_reader(response.into_reader())
        .map_err(|_| ProviderCallError::new("openai", "json_parse", None))
}

fn build_http_agent(
    timeout_ms: u32,
    user_agent: &str,
    proxy_config: &Ph1eProxyConfig,
) -> Result<ureq::Agent, String> {
    if timeout_ms == 0 {
        return Err("timeout must be > 0".to_string());
    }
    let timeout = Duration::from_millis(u64::from(timeout_ms).max(100));
    let mut builder = ureq::AgentBuilder::new()
        .timeout_connect(timeout)
        .timeout_read(timeout)
        .timeout_write(timeout)
        .user_agent(user_agent)
        .try_proxy_from_env(false);
    let resolved_proxy = resolve_proxy_config(proxy_config)?;
    if let Some(proxy_url) = resolved_proxy.effective_proxy_url.as_deref() {
        let proxy = ureq::Proxy::new(proxy_url)
            .map_err(|_| format!("invalid proxy url for mode={}", proxy_config.mode.as_str()))?;
        builder = builder.proxy(proxy);
    }
    Ok(builder.build())
}

fn provider_error_from_ureq(provider: &'static str, err: ureq::Error) -> ProviderCallError {
    match err {
        ureq::Error::Status(status, _) => {
            ProviderCallError::new(provider, "http_non_200", Some(status as u16))
        }
        ureq::Error::Transport(transport) => provider_error_from_transport(provider, transport),
    }
}

fn provider_error_from_transport(
    provider: &'static str,
    transport: ureq::Transport,
) -> ProviderCallError {
    let combined = format!("{:?} {}", transport.kind(), transport);
    let error_kind = classify_transport_error_kind(&combined);
    ProviderCallError::new(provider, error_kind, None)
}

fn classify_transport_error_kind(raw: &str) -> &'static str {
    let lower = raw.to_ascii_lowercase();
    if lower.contains("timeout") {
        "timeout"
    } else if lower.contains("tls") || lower.contains("ssl") {
        "tls"
    } else if lower.contains("dns") {
        "dns"
    } else if lower.contains("connection") || lower.contains("connect") {
        "connection"
    } else {
        "transport"
    }
}

fn combine_live_search_failure_detail(
    brave_failure: Option<&ProviderCallError>,
    openai_failure: Option<&ProviderCallError>,
    proxy_config: &Ph1eProxyConfig,
) -> String {
    let mut detail = match (brave_failure, openai_failure) {
        (Some(brave), Some(openai)) => format!(
            "primary({}) fallback({})",
            brave.safe_detail(),
            openai.safe_detail()
        ),
        (Some(brave), None) => brave.safe_detail(),
        (None, Some(openai)) => openai.safe_detail(),
        (None, None) => "provider=unknown error=upstream".to_string(),
    };
    if let Some(proxy_hint) =
        proxy_hint_for_failures(brave_failure, openai_failure, proxy_config)
    {
        detail.push(' ');
        detail.push_str(&proxy_hint);
    }
    detail
}

fn extract_tool_snippets(
    root: &Value,
    max_results: usize,
    is_news: bool,
) -> Option<Vec<ToolTextSnippet>> {
    let mut candidates: Vec<&Value> = Vec::new();
    if is_news {
        if let Some(v) = root.pointer("/results").and_then(Value::as_array) {
            candidates.extend(v.iter());
        }
        if let Some(v) = root.pointer("/news/results").and_then(Value::as_array) {
            candidates.extend(v.iter());
        }
    } else {
        if let Some(v) = root.pointer("/web/results").and_then(Value::as_array) {
            candidates.extend(v.iter());
        }
        if let Some(v) = root.pointer("/results").and_then(Value::as_array) {
            candidates.extend(v.iter());
        }
        if let Some(v) = root.pointer("/output").and_then(Value::as_array) {
            candidates.extend(v.iter());
        }
    }

    let mut out: Vec<ToolTextSnippet> = Vec::new();
    for item in candidates {
        if out.len() >= max_results {
            break;
        }
        if let Some(snippet) = value_to_tool_text_snippet(item) {
            out.push(snippet);
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn extract_openai_results(root: &Value, max_results: usize) -> Option<Vec<ToolTextSnippet>> {
    if let Some(v) = root.get("results").and_then(Value::as_array) {
        let mut out = Vec::new();
        for item in v {
            if out.len() >= max_results {
                break;
            }
            if let Some(snippet) = value_to_tool_text_snippet(item) {
                out.push(snippet);
            }
        }
        if !out.is_empty() {
            return Some(out);
        }
    }

    let output_text = root
        .get("output_text")
        .and_then(Value::as_str)
        .or_else(|| {
            root.pointer("/output/0/content/0/text")
                .and_then(Value::as_str)
        })?;
    let json_candidate = output_text
        .split_once('{')
        .map(|(_, rest)| format!("{{{rest}"))
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())?;
    extract_openai_results(&json_candidate, max_results)
}

fn value_to_tool_text_snippet(value: &Value) -> Option<ToolTextSnippet> {
    let url = value
        .get("url")
        .or_else(|| value.get("link"))
        .and_then(Value::as_str)?
        .trim();
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return None;
    }

    let title = value
        .get("title")
        .or_else(|| value.get("name"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("Result");

    let snippet = value
        .get("description")
        .or_else(|| value.get("snippet"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("No snippet available");

    Some(ToolTextSnippet {
        title: truncate_ascii(title, 256),
        snippet: truncate_ascii(snippet, 2048),
        url: truncate_ascii(url, 2048),
    })
}

fn run_url_fetch_citation(
    url: &str,
    max_results: u8,
    timeout_ms: u32,
    user_agent: &str,
    proxy_config: &Ph1eProxyConfig,
    fixture_html: Option<&str>,
) -> Result<(Vec<ToolTextSnippet>, Vec<SourceRef>), String> {
    let body_text = if let Some(fixture) = fixture_html {
        fixture.to_string()
    } else {
        let agent = build_http_agent(timeout_ms, user_agent, proxy_config)?;
        let response = agent
            .get(url)
            .set("Accept", "text/html,text/plain;q=0.9,*/*;q=0.1")
            .call()
            .map_err(|e| format!("url fetch failed: {e}"))?;
        let mut reader = response.into_reader().take(256 * 1024);
        let mut body = Vec::new();
        reader
            .read_to_end(&mut body)
            .map_err(|e| format!("failed to read fetched response: {e}"))?;
        String::from_utf8_lossy(&body).to_string()
    };

    let normalized = normalize_text_for_citation(&body_text);
    if normalized.is_empty() {
        return Err("fetched page had no extractable text".to_string());
    }

    let chunks = split_text_chunks(&normalized, 450, usize::from(max_results).max(1));
    let mut citations = Vec::new();
    let mut sources = Vec::new();
    for (idx, chunk) in chunks.into_iter().enumerate() {
        let hash = stable_content_hash_hex(&chunk);
        let suffix = &hash[..12];
        let chunk_url = format!("{url}#chunk-{:02}-{suffix}", idx + 1);
        citations.push(ToolTextSnippet {
            title: format!("Citation chunk {}", idx + 1),
            snippet: truncate_ascii(&format!("{chunk} [content_hash:{hash}]"), 2048),
            url: chunk_url.clone(),
        });
        sources.push(SourceRef {
            title: truncate_ascii(
                &format!("URL citation chunk {} (hash:{})", idx + 1, &hash[..16]),
                256,
            ),
            url: chunk_url,
        });
    }
    if citations.is_empty() {
        return Err("no citations were generated".to_string());
    }
    Ok((citations, sources))
}

fn first_http_url_in_text(input: &str) -> Option<String> {
    for token in input.split_whitespace() {
        let candidate = token.trim_matches(|c: char| {
            c == '"'
                || c == '\''
                || c == '('
                || c == ')'
                || c == '['
                || c == ']'
                || c == '{'
                || c == '}'
                || c == '<'
                || c == '>'
                || c == ','
                || c == ';'
        });
        if candidate.starts_with("https://") || candidate.starts_with("http://") {
            return Some(candidate.to_string());
        }
    }
    None
}

fn normalize_text_for_citation(input: &str) -> String {
    let stripped = strip_html_tags(input);
    collapse_ws(stripped.trim())
}

fn strip_html_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => {
                in_tag = true;
                out.push(' ');
            }
            '>' => {
                in_tag = false;
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

fn split_text_chunks(input: &str, chunk_size_chars: usize, max_chunks: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut current_chars = 0usize;
    for ch in input.chars() {
        current.push(ch);
        current_chars += 1;
        if current_chars >= chunk_size_chars {
            let compact = collapse_ws(current.trim());
            if !compact.is_empty() {
                out.push(compact);
            }
            if out.len() >= max_chunks {
                return out;
            }
            current.clear();
            current_chars = 0;
        }
    }
    if !current.trim().is_empty() && out.len() < max_chunks {
        out.push(collapse_ws(current.trim()));
    }
    out
}

fn items_to_sources(items: &[ToolTextSnippet]) -> Vec<SourceRef> {
    items
        .iter()
        .map(|item| SourceRef {
            title: truncate_ascii(&item.title, 256),
            url: item.url.clone(),
        })
        .collect()
}

fn stable_content_hash_hex(input: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(1)
        .max(1)
}

fn trim_non_empty(raw: String) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn connector_scope_for_query(query: &str) -> (Vec<&'static str>, bool) {
    let lower = query.to_ascii_lowercase();
    let mut out = Vec::new();
    for connector in [
        "gmail", "outlook", "calendar", "drive", "dropbox", "slack", "notion", "onedrive",
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
            url: "https://workspace.selene.local/gmail".to_string(),
        },
        "outlook" => SourceRef {
            title: "Connector source: Outlook".to_string(),
            url: "https://workspace.selene.local/outlook".to_string(),
        },
        "calendar" => SourceRef {
            title: "Connector source: Calendar".to_string(),
            url: "https://workspace.selene.local/calendar".to_string(),
        },
        "drive" => SourceRef {
            title: "Connector source: Drive".to_string(),
            url: "https://workspace.selene.local/drive".to_string(),
        },
        "dropbox" => SourceRef {
            title: "Connector source: Dropbox".to_string(),
            url: "https://workspace.selene.local/dropbox".to_string(),
        },
        "slack" => SourceRef {
            title: "Connector source: Slack".to_string(),
            url: "https://workspace.selene.local/slack".to_string(),
        },
        "notion" => SourceRef {
            title: "Connector source: Notion".to_string(),
            url: "https://workspace.selene.local/notion".to_string(),
        },
        "onedrive" => SourceRef {
            title: "Connector source: OneDrive".to_string(),
            url: "https://workspace.selene.local/onedrive".to_string(),
        },
        _ => SourceRef {
            title: "Connector source".to_string(),
            url: "https://workspace.selene.local/connectors".to_string(),
        },
    }
}

fn connector_citation(connector: &'static str, query: &str, idx: usize) -> ToolTextSnippet {
    let compact_query = truncate_ascii(query, 60);
    match connector {
        "gmail" => ToolTextSnippet {
            title: "Gmail thread result".to_string(),
            snippet: format!("Gmail match for '{compact_query}'"),
            url: format!("https://workspace.selene.local/gmail/thread_{:03}", idx + 1),
        },
        "outlook" => ToolTextSnippet {
            title: "Outlook message result".to_string(),
            snippet: format!("Outlook match for '{compact_query}'"),
            url: format!(
                "https://workspace.selene.local/outlook/message_{:03}",
                idx + 1
            ),
        },
        "calendar" => ToolTextSnippet {
            title: "Calendar event result".to_string(),
            snippet: format!("Calendar match for '{compact_query}'"),
            url: format!(
                "https://workspace.selene.local/calendar/event_{:03}",
                idx + 1
            ),
        },
        "drive" => ToolTextSnippet {
            title: "Drive doc result".to_string(),
            snippet: format!("Drive match for '{compact_query}'"),
            url: format!("https://workspace.selene.local/drive/doc_{:03}", idx + 1),
        },
        "dropbox" => ToolTextSnippet {
            title: "Dropbox file result".to_string(),
            snippet: format!("Dropbox match for '{compact_query}'"),
            url: format!("https://workspace.selene.local/dropbox/file_{:03}", idx + 1),
        },
        "slack" => ToolTextSnippet {
            title: "Slack message result".to_string(),
            snippet: format!("Slack match for '{compact_query}'"),
            url: format!(
                "https://workspace.selene.local/slack/message_{:03}",
                idx + 1
            ),
        },
        "notion" => ToolTextSnippet {
            title: "Notion page result".to_string(),
            snippet: format!("Notion match for '{compact_query}'"),
            url: format!("https://workspace.selene.local/notion/page_{:03}", idx + 1),
        },
        "onedrive" => ToolTextSnippet {
            title: "OneDrive file result".to_string(),
            snippet: format!("OneDrive match for '{compact_query}'"),
            url: format!(
                "https://workspace.selene.local/onedrive/file_{:03}",
                idx + 1
            ),
        },
        _ => ToolTextSnippet {
            title: "Connector result".to_string(),
            snippet: format!("Connector match for '{compact_query}'"),
            url: format!(
                "https://workspace.selene.local/connectors/item_{:03}",
                idx + 1
            ),
        },
    }
}

fn truncate_ascii(input: &str, max_len: usize) -> String {
    input.chars().take(max_len).collect()
}

fn collapse_ws(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut prev_space = false;
    for ch in input.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out.trim().to_string()
}

fn fail_response(req: &ToolRequest, code: ReasonCodeId, cache_status: CacheStatus) -> ToolResponse {
    fail_response_with_detail(req, code, cache_status, None)
}

fn fail_response_with_detail(
    req: &ToolRequest,
    code: ReasonCodeId,
    cache_status: CacheStatus,
    fail_detail: Option<&str>,
) -> ToolResponse {
    let safe_detail = fail_detail
        .map(sanitize_fail_detail_text)
        .filter(|detail| !detail.is_empty());
    ToolResponse::fail_with_detail_v1(req.request_id, req.query_hash, code, safe_detail, cache_status)
        .expect("ToolResponse::fail_v1 must construct for bounded PH1.E failure output")
}

fn sanitize_fail_detail_text(detail: &str) -> String {
    truncate_ascii(&collapse_ws(detail), 256)
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1d::{PolicyContextRef, SafetyTier};
    use selene_kernel_contracts::ph1e::{ToolRequestOrigin, ToolStatus};
    use std::ffi::OsString;
    use std::sync::{Mutex, OnceLock};
    use std::{env, fs};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct ScopedEnvVar {
        key: &'static str,
        previous: Option<OsString>,
    }

    impl ScopedEnvVar {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = env::var_os(key);
            env::set_var(key, value);
            Self { key, previous }
        }

        fn unset(key: &'static str) -> Self {
            let previous = env::var_os(key);
            env::remove_var(key);
            Self { key, previous }
        }
    }

    impl Drop for ScopedEnvVar {
        fn drop(&mut self) {
            if let Some(value) = self.previous.as_ref() {
                env::set_var(self.key, value);
            } else {
                env::remove_var(self.key);
            }
        }
    }

    fn with_isolated_empty_device_vault<T>(label: &str, f: impl FnOnce() -> T) -> T {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let env_lock = ENV_LOCK.get_or_init(|| Mutex::new(()));
        let _guard = env_lock.lock().expect("env lock poisoned");
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time must be monotonic for tests")
            .as_nanos();
        let path = env::temp_dir().join(format!("selene-empty-vault-{label}-{nanos}.json"));
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(path.with_extension("master.key"));
        let path_text = path
            .to_str()
            .expect("temp path should be valid UTF-8 for test env var")
            .to_string();
        let _scope = ScopedEnvVar::set("SELENE_DEVICE_VAULT_PATH", &path_text);
        let out = f();
        let _ = fs::remove_file(path);
        out
    }

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

    #[derive(Debug, Clone)]
    struct TestHttpFixture {
        base_url: String,
        brave_web_url: String,
        brave_news_url: String,
        brave_web_fixture_json: String,
        brave_news_fixture_json: String,
        url_fetch_fixture_html: String,
    }

    fn spawn_test_http_fixture() -> TestHttpFixture {
        let base = "https://docs.selene.ai".to_string();
        TestHttpFixture {
            base_url: base.clone(),
            brave_web_url: format!("{base}/res/v1/web/search"),
            brave_news_url: format!("{base}/res/v1/news/search"),
            brave_web_fixture_json:
                r#"{"web":{"results":[{"title":"Selene web result","url":"https://docs.selene.ai/search/1","description":"Provider-backed web snippet"}]}}"#
                    .to_string(),
            brave_news_fixture_json:
                r#"{"results":[{"title":"Selene news result","url":"https://news.selene.ai/item/1","description":"Provider-backed news snippet"}]}"#
                    .to_string(),
            url_fetch_fixture_html:
                "<html><body><h1>Selene spec</h1><p>This page proves URL fetch and citation chunking behavior with deterministic evidence text.</p></body></html>"
                    .to_string(),
        }
    }

    fn runtime_with_live_fixture(fixture: &TestHttpFixture) -> Ph1eRuntime {
        Ph1eRuntime::new_with_provider_config(
            Ph1eConfig::mvp_v1(),
            Ph1eProviderConfig {
                brave_api_key: Some("fixture_brave_key".to_string()),
                brave_web_url: fixture.brave_web_url.clone(),
                brave_news_url: fixture.brave_news_url.clone(),
                brave_web_fixture_json: Some(fixture.brave_web_fixture_json.clone()),
                brave_news_fixture_json: Some(fixture.brave_news_fixture_json.clone()),
                openai_api_key: None,
                openai_responses_url: "https://api.openai.com/v1/responses".to_string(),
                openai_model: "gpt-4o-mini".to_string(),
                user_agent: "selene-ph1e-test/1.0".to_string(),
                proxy_config: Ph1eProxyConfig {
                    mode: Ph1eProxyMode::Off,
                    http_proxy_url: None,
                    https_proxy_url: None,
                },
                url_fetch_fixture_html: Some(fixture.url_fetch_fixture_html.clone()),
            },
        )
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
        let fixture = spawn_test_http_fixture();
        let rt = runtime_with_live_fixture(&fixture);
        let page_url = format!("{}/page", fixture.base_url);
        let out = rt.run(&req(
            ToolName::UrlFetchAndCite,
            &format!("open this URL and cite it: {page_url}"),
            false,
            false,
        ));
        assert_eq!(out.tool_status, ToolStatus::Ok);
        match out
            .tool_result
            .as_ref()
            .expect("tool result required for ok")
        {
            ToolResult::UrlFetchAndCite { citations } => assert!(!citations.is_empty()),
            other => panic!("expected UrlFetchAndCite result, got {other:?}"),
        }
        let meta = out
            .source_metadata
            .as_ref()
            .expect("source metadata required");
        assert!(!meta.sources.is_empty());
        assert!(meta.sources[0].url.contains("#chunk-"));
        assert!(!meta.sources[0].url.contains("example.invalid"));
        assert_eq!(meta.provider_hint.as_deref(), Some("ph1search_url_fetch"));
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
        match out
            .tool_result
            .as_ref()
            .expect("tool result required for ok")
        {
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
        let meta = out
            .source_metadata
            .as_ref()
            .expect("source metadata required");
        assert!(!meta.sources.is_empty());
        assert!(!meta.sources[0].url.contains("example.invalid"));
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
        match out
            .tool_result
            .as_ref()
            .expect("tool result required for ok")
        {
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
        let meta = out
            .source_metadata
            .as_ref()
            .expect("source metadata required");
        assert!(!meta.sources.is_empty());
        assert!(!meta.sources[0].url.contains("example.invalid"));
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
        match out
            .tool_result
            .as_ref()
            .expect("tool result required for ok")
        {
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
        let meta = out
            .source_metadata
            .as_ref()
            .expect("source metadata required");
        assert!(!meta.sources.is_empty());
        assert!(!meta.sources[0].url.contains("example.invalid"));
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
        match out
            .tool_result
            .as_ref()
            .expect("tool result required for ok")
        {
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
        let meta = out
            .source_metadata
            .as_ref()
            .expect("source metadata required");
        assert!(!meta.sources.is_empty());
        assert!(!meta.sources[0].url.contains("example.invalid"));
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
        match out
            .tool_result
            .as_ref()
            .expect("tool result required for ok")
        {
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
        let meta = out
            .source_metadata
            .as_ref()
            .expect("source metadata required");
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
        match out
            .tool_result
            .as_ref()
            .expect("tool result required for ok")
        {
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
                assert!(citations
                    .iter()
                    .all(|item| { item.url.contains("/gmail/") || item.url.contains("/drive/") }));
            }
            other => panic!("expected ConnectorQuery result, got {other:?}"),
        }
        let meta = out
            .source_metadata
            .as_ref()
            .expect("source metadata required");
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
        match out
            .tool_result
            .as_ref()
            .expect("tool result required for ok")
        {
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
        match out
            .tool_result
            .as_ref()
            .expect("tool result required for ok")
        {
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

    #[test]
    fn at_e_16_web_search_fails_closed_when_provider_keys_missing() {
        with_isolated_empty_device_vault("at_e_16", || {
            let rt = Ph1eRuntime::new_with_provider_config(
                Ph1eConfig::mvp_v1(),
                Ph1eProviderConfig {
                    brave_api_key: None,
                    brave_web_url: "https://api.search.brave.com/res/v1/web/search".to_string(),
                    brave_news_url: "https://api.search.brave.com/res/v1/news/search".to_string(),
                    brave_web_fixture_json: None,
                    brave_news_fixture_json: None,
                    openai_api_key: None,
                    openai_responses_url: "https://api.openai.com/v1/responses".to_string(),
                    openai_model: "gpt-4o-mini".to_string(),
                    user_agent: "selene-ph1e-test/1.0".to_string(),
                    proxy_config: Ph1eProxyConfig {
                        mode: Ph1eProxyMode::Off,
                        http_proxy_url: None,
                        https_proxy_url: None,
                    },
                    url_fetch_fixture_html: None,
                },
            );
            let out = rt.run(&req(
                ToolName::WebSearch,
                "search the web for selene release notes",
                false,
                false,
            ));
            assert_eq!(out.tool_status, ToolStatus::Fail);
            assert_eq!(
                out.reason_code,
                reason_codes::E_FAIL_PROVIDER_MISSING_CONFIG
            );
        });
    }

    #[test]
    fn at_e_17_live_web_and_news_search_do_not_emit_mock_urls() {
        let fixture = spawn_test_http_fixture();
        let rt = runtime_with_live_fixture(&fixture);
        let web = rt.run(&req(
            ToolName::WebSearch,
            "search the web for selene test fixture",
            false,
            false,
        ));
        assert_eq!(web.tool_status, ToolStatus::Ok);
        if let ToolResult::WebSearch { items } = web
            .tool_result
            .as_ref()
            .expect("tool result required for web search")
        {
            assert!(!items.is_empty());
            assert!(!items[0].url.contains("example.invalid"));
        } else {
            panic!("expected web search tool result");
        }

        let news = rt.run(&req(
            ToolName::News,
            "latest selene project updates",
            false,
            false,
        ));
        assert_eq!(news.tool_status, ToolStatus::Ok);
        if let ToolResult::News { items } = news
            .tool_result
            .as_ref()
            .expect("tool result required for news search")
        {
            assert!(!items.is_empty());
            assert!(!items[0].url.contains("example.invalid"));
        } else {
            panic!("expected news tool result");
        }
    }

    #[test]
    fn at_e_18_web_search_brave_failure_surfaces_safe_fail_detail() {
        with_isolated_empty_device_vault("at_e_18", || {
            let rt = Ph1eRuntime::new_with_provider_config(
                Ph1eConfig::mvp_v1(),
                Ph1eProviderConfig {
                    brave_api_key: Some("test_brave_key".to_string()),
                    brave_web_url: "http://127.0.0.1:9/res/v1/web/search".to_string(),
                    brave_news_url: "http://127.0.0.1:9/res/v1/news/search".to_string(),
                    brave_web_fixture_json: None,
                    brave_news_fixture_json: None,
                    openai_api_key: None,
                    openai_responses_url: "https://api.openai.com/v1/responses".to_string(),
                    openai_model: "gpt-4o-mini".to_string(),
                    user_agent: "selene-ph1e-test/1.0".to_string(),
                    proxy_config: Ph1eProxyConfig {
                        mode: Ph1eProxyMode::Off,
                        http_proxy_url: None,
                        https_proxy_url: None,
                    },
                    url_fetch_fixture_html: None,
                },
            );
            let out = rt.run(&req(
                ToolName::WebSearch,
                "search the web for selene release notes",
                false,
                false,
            ));
            assert_eq!(out.tool_status, ToolStatus::Fail);
            assert_eq!(out.reason_code, reason_codes::E_FAIL_PROVIDER_UPSTREAM);
            let detail = out
                .fail_detail
                .as_deref()
                .expect("fail detail must be present for provider upstream failures")
                .to_ascii_lowercase();
            assert!(detail.contains("provider=brave"));
            assert!(detail.contains("error="));
        });
    }

    #[test]
    fn at_e_19_web_search_openai_failure_surfaces_safe_fail_detail() {
        with_isolated_empty_device_vault("at_e_19", || {
            let rt = Ph1eRuntime::new_with_provider_config(
                Ph1eConfig::mvp_v1(),
                Ph1eProviderConfig {
                    brave_api_key: None,
                    brave_web_url: "https://api.search.brave.com/res/v1/web/search".to_string(),
                    brave_news_url: "https://api.search.brave.com/res/v1/news/search".to_string(),
                    brave_web_fixture_json: None,
                    brave_news_fixture_json: None,
                    openai_api_key: Some("test_openai_key".to_string()),
                    openai_responses_url: "http://127.0.0.1:9/v1/responses".to_string(),
                    openai_model: "gpt-4o-mini".to_string(),
                    user_agent: "selene-ph1e-test/1.0".to_string(),
                    proxy_config: Ph1eProxyConfig {
                        mode: Ph1eProxyMode::Off,
                        http_proxy_url: None,
                        https_proxy_url: None,
                    },
                    url_fetch_fixture_html: None,
                },
            );
            let out = rt.run(&req(
                ToolName::WebSearch,
                "search the web for selene release notes",
                false,
                false,
            ));
            assert_eq!(out.tool_status, ToolStatus::Fail);
            assert_eq!(out.reason_code, reason_codes::E_FAIL_PROVIDER_UPSTREAM);
            let detail = out
                .fail_detail
                .as_deref()
                .expect("fail detail must be present for provider upstream failures")
                .to_ascii_lowercase();
            assert!(detail.contains("provider=openai"));
            assert!(detail.contains("error="));
        });
    }

    #[test]
    fn proxy_mode_off_builds_client_no_proxy() {
        let proxy_config = Ph1eProxyConfig {
            mode: Ph1eProxyMode::Off,
            http_proxy_url: Some("http://127.0.0.1:7897".to_string()),
            https_proxy_url: Some("http://127.0.0.1:7898".to_string()),
        };
        let resolved = resolve_proxy_config(&proxy_config).expect("proxy config must resolve");
        assert_eq!(resolved.mode, Ph1eProxyMode::Off);
        assert!(resolved.effective_proxy_url.is_none());
        assert!(build_http_agent(1_000, "selene-test", &proxy_config).is_ok());
    }

    #[test]
    fn proxy_mode_env_does_not_set_all_proxy() {
        let _all_proxy = ScopedEnvVar::set("ALL_PROXY", "socks5://127.0.0.1:9050");
        let _http_proxy = ScopedEnvVar::set("HTTP_PROXY", "http://127.0.0.1:7001");
        let _https_proxy = ScopedEnvVar::unset("HTTPS_PROXY");
        let proxy_config = Ph1eProxyConfig {
            mode: Ph1eProxyMode::Env,
            http_proxy_url: None,
            https_proxy_url: None,
        };
        let resolved = resolve_proxy_config(&proxy_config).expect("proxy config must resolve");
        assert_eq!(
            resolved.effective_proxy_url.as_deref(),
            Some("http://127.0.0.1:7001")
        );
        let effective = resolved.effective_proxy_url.as_deref().unwrap_or("");
        assert!(!effective.starts_with("socks5://"));
    }

    #[test]
    fn proxy_mode_explicit_sets_http_https_proxy() {
        let proxy_config = Ph1eProxyConfig {
            mode: Ph1eProxyMode::Explicit,
            http_proxy_url: Some("http://127.0.0.1:7897".to_string()),
            https_proxy_url: Some("http://127.0.0.1:7898".to_string()),
        };
        let resolved = resolve_proxy_config(&proxy_config).expect("proxy config must resolve");
        assert_eq!(
            resolved.http_proxy_url.as_deref(),
            Some("http://127.0.0.1:7897")
        );
        assert_eq!(
            resolved.https_proxy_url.as_deref(),
            Some("http://127.0.0.1:7898")
        );
        assert_eq!(
            resolved.effective_proxy_url.as_deref(),
            Some("http://127.0.0.1:7898")
        );
    }

    #[test]
    fn startup_self_check_failure_produces_safe_diagnostic() {
        let provider_config = Ph1eProviderConfig {
            brave_api_key: None,
            brave_web_url: "https://api.search.brave.com/res/v1/web/search".to_string(),
            brave_news_url: "https://api.search.brave.com/res/v1/news/search".to_string(),
            openai_api_key: None,
            openai_responses_url: "https://api.openai.com/v1/responses".to_string(),
            openai_model: "gpt-4o-mini".to_string(),
            user_agent: "selene-ph1e-test/1.0".to_string(),
            proxy_config: Ph1eProxyConfig {
                mode: Ph1eProxyMode::Explicit,
                http_proxy_url: Some("http://user:password@127.0.0.1:7897".to_string()),
                https_proxy_url: Some("http://127.0.0.1:7898".to_string()),
            },
            brave_web_fixture_json: None,
            brave_news_fixture_json: None,
            url_fetch_fixture_html: None,
        };

        let failures =
            run_startup_outbound_self_check_with_probe(&provider_config, |provider, _, _, _, _| {
                Err(ProviderCallError::new(provider, "connection", None))
            });
        assert_eq!(failures.len(), 2);
        for failure in failures {
            let line = failure.safe_log_line();
            assert!(line.contains("proxy_mode=explicit"));
            assert!(line.contains("error=connection"));
            assert!(line.contains("provider="));
            assert!(!line.contains("user:password"));
        }
    }
}
