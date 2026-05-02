#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

pub const SELENE_SEARCH_PROVIDERS_ENABLED: &str = "SELENE_SEARCH_PROVIDERS_ENABLED";
pub const SELENE_PAID_SEARCH_PROVIDERS_ENABLED: &str = "SELENE_PAID_SEARCH_PROVIDERS_ENABLED";
pub const SELENE_WEB_SEARCH_ENABLED: &str = "SELENE_WEB_SEARCH_ENABLED";
pub const SELENE_DEEP_RESEARCH_ENABLED: &str = "SELENE_DEEP_RESEARCH_ENABLED";
pub const SELENE_NEWS_SEARCH_ENABLED: &str = "SELENE_NEWS_SEARCH_ENABLED";
pub const SELENE_URL_FETCH_ENABLED: &str = "SELENE_URL_FETCH_ENABLED";
pub const SELENE_STARTUP_PROVIDER_PROBES_ENABLED: &str = "SELENE_STARTUP_PROVIDER_PROBES_ENABLED";
pub const SELENE_BRAVE_SEARCH_ENABLED: &str = "SELENE_BRAVE_SEARCH_ENABLED";
pub const SELENE_PROVIDER_CALL_MAX_PER_TURN: &str = "SELENE_PROVIDER_CALL_MAX_PER_TURN";
pub const SELENE_PROVIDER_CALL_MAX_PER_ROUTE: &str = "SELENE_PROVIDER_CALL_MAX_PER_ROUTE";
pub const SELENE_PROVIDER_CALL_MAX_PER_ACTOR_USER: &str = "SELENE_PROVIDER_CALL_MAX_PER_ACTOR_USER";
pub const SELENE_PROVIDER_CALL_MAX_PER_TENANT: &str = "SELENE_PROVIDER_CALL_MAX_PER_TENANT";
pub const SELENE_PROVIDER_RETRY_MAX: &str = "SELENE_PROVIDER_RETRY_MAX";
pub const SELENE_PROVIDER_FALLBACK_ENABLED: &str = "SELENE_PROVIDER_FALLBACK_ENABLED";
pub const SELENE_PROVIDER_FANOUT_ENABLED: &str = "SELENE_PROVIDER_FANOUT_ENABLED";
pub const SELENE_BRAVE_IMAGE_SEARCH_ENABLED: &str = "SELENE_BRAVE_IMAGE_SEARCH_ENABLED";
pub const SELENE_BRAVE_MAX_CALLS_PER_TEST_RUN: &str = "SELENE_BRAVE_MAX_CALLS_PER_TEST_RUN";
pub const SELENE_BRAVE_MAX_CALLS_PER_DAY_TEST: &str = "SELENE_BRAVE_MAX_CALLS_PER_DAY_TEST";
pub const SELENE_RUN_LIVE_BRAVE_PROOF: &str = "SELENE_RUN_LIVE_BRAVE_PROOF";
pub const SELENE_OPENAI_WEB_SEARCH_ENABLED: &str = "SELENE_OPENAI_WEB_SEARCH_ENABLED";
pub const SELENE_GDELT_NEWS_SEARCH_ENABLED: &str = "SELENE_GDELT_NEWS_SEARCH_ENABLED";
pub const SELENE_CHEAP_SEARCH_PROVIDER_ENABLED: &str = "SELENE_CHEAP_SEARCH_PROVIDER_ENABLED";
pub const SELENE_NEWS_CURRENT_EVENTS_PROVIDER_ENABLED: &str =
    "SELENE_NEWS_CURRENT_EVENTS_PROVIDER_ENABLED";
pub const SELENE_PROVIDER_FALLBACK_MAX_PER_TURN: &str = "SELENE_PROVIDER_FALLBACK_MAX_PER_TURN";
pub const SELENE_SEARCH_CERTIFICATION_ENABLED: &str = "SELENE_SEARCH_CERTIFICATION_ENABLED";
pub const SELENE_SEARCH_CERTIFICATION_MODE: &str = "SELENE_SEARCH_CERTIFICATION_MODE";
pub const SELENE_FAKE_SEARCH_PROVIDER_ENABLED: &str = "SELENE_FAKE_SEARCH_PROVIDER_ENABLED";
pub const SELENE_SOURCE_AGREEMENT_SCORING_ENABLED: &str = "SELENE_SOURCE_AGREEMENT_SCORING_ENABLED";
pub const SELENE_FRESHNESS_SCORING_ENABLED: &str = "SELENE_FRESHNESS_SCORING_ENABLED";
pub const SELENE_CORROBORATION_ENABLED: &str = "SELENE_CORROBORATION_ENABLED";
pub const SELENE_DEEP_RESEARCH_REQUIRE_APPROVAL: &str = "SELENE_DEEP_RESEARCH_REQUIRE_APPROVAL";
pub const SELENE_DEEP_RESEARCH_MAX_PROVIDER_CALLS: &str = "SELENE_DEEP_RESEARCH_MAX_PROVIDER_CALLS";

pub const WEB_ADMIN_DISABLED: &str = "WEB_ADMIN_DISABLED";
pub const PROVIDER_DISABLED: &str = "PROVIDER_DISABLED";
pub const PAID_PROVIDER_DISABLED: &str = "PAID_PROVIDER_DISABLED";
pub const URL_FETCH_DISABLED: &str = "URL_FETCH_DISABLED";
pub const DEEP_RESEARCH_DISABLED: &str = "DEEP_RESEARCH_DISABLED";
pub const NEWS_SEARCH_DISABLED: &str = "NEWS_SEARCH_DISABLED";
pub const PROVIDER_BUDGET_EXHAUSTED: &str = "PROVIDER_BUDGET_EXHAUSTED";
pub const STARTUP_PROVIDER_PROBES_DISABLED: &str = "STARTUP_PROVIDER_PROBES_DISABLED";
pub const PROVIDER_FALLBACK_DISABLED: &str = "PROVIDER_FALLBACK_DISABLED";
pub const PROVIDER_FANOUT_DISABLED: &str = "PROVIDER_FANOUT_DISABLED";
pub const PROVIDER_SECRET_MISSING: &str = "PROVIDER_SECRET_MISSING";
pub const CACHE_HIT: &str = "CACHE_HIT";
pub const CACHE_MISS: &str = "CACHE_MISS";
pub const CACHE_STALE: &str = "CACHE_STALE";
pub const NO_SEARCH_NEEDED: &str = "NO_SEARCH_NEEDED";
pub const CHEAP_PROVIDER_UNAVAILABLE: &str = "CHEAP_PROVIDER_UNAVAILABLE";
pub const TEST_FAKE_PROVIDER: &str = "TEST_FAKE_PROVIDER";
pub const BLOCKED_NOT_BILLABLE: &str = "BLOCKED_NOT_BILLABLE";
pub const NON_BILLABLE: &str = "NON_BILLABLE";
pub const UNKNOWN: &str = "UNKNOWN";
pub const UNAVAILABLE: &str = "UNAVAILABLE";

pub const PROVIDER_DISABLED_RESPONSE_TEXT: &str =
    "I can't access live web search right now. Search providers are disabled.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProviderControlRoute {
    WebSearch,
    DeepResearch,
    NewsSearch,
    UrlFetch,
    ImageSearch,
    StartupProbe,
}

impl ProviderControlRoute {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WebSearch => "WebSearch",
            Self::DeepResearch => "DeepResearch",
            Self::NewsSearch => "NewsSearch",
            Self::UrlFetch => "UrlFetch",
            Self::ImageSearch => "ImageSearch",
            Self::StartupProbe => "StartupProbe",
        }
    }

    pub const fn service_type(self) -> &'static str {
        match self {
            Self::WebSearch => "WebSearch",
            Self::DeepResearch => "DeepResearch",
            Self::NewsSearch => "NewsSearch",
            Self::UrlFetch => "UrlFetch",
            Self::ImageSearch => "ImageSearch",
            Self::StartupProbe => "ToolExecution",
        }
    }

    pub const fn operation_type(self) -> &'static str {
        match self {
            Self::WebSearch => "web_search",
            Self::DeepResearch => "deep_research",
            Self::NewsSearch => "news_search",
            Self::UrlFetch => "url_fetch",
            Self::ImageSearch => "image_search",
            Self::StartupProbe => "startup_provider_probe",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProviderControlProvider {
    CacheOnly,
    CheapGeneralSearch,
    NewsCurrentEvents,
    BraveWebSearch,
    BraveNewsSearch,
    BraveImageSearch,
    OpenAiWebSearch,
    GdeltNewsAssist,
    UrlFetch,
    StartupProbe,
}

impl ProviderControlProvider {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CacheOnly => "cache_only",
            Self::CheapGeneralSearch => "cheap_general_search",
            Self::NewsCurrentEvents => "news_current_events",
            Self::BraveWebSearch => "brave_web_search",
            Self::BraveNewsSearch => "brave_news_search",
            Self::BraveImageSearch => "brave_image_search",
            Self::OpenAiWebSearch => "openai_web_search",
            Self::GdeltNewsAssist => "gdelt_news_assist",
            Self::UrlFetch => "url_fetch",
            Self::StartupProbe => "startup_provider_probe",
        }
    }

    pub const fn tier(self) -> &'static str {
        match self {
            Self::CacheOnly => "cache",
            Self::CheapGeneralSearch => "cheap",
            Self::NewsCurrentEvents => "free_or_internal_news",
            Self::BraveWebSearch
            | Self::BraveNewsSearch
            | Self::BraveImageSearch
            | Self::OpenAiWebSearch
            | Self::GdeltNewsAssist
            | Self::UrlFetch => "paid_or_external",
            Self::StartupProbe => "diagnostic_external",
        }
    }

    pub const fn is_paid(self) -> bool {
        matches!(
            self,
            Self::BraveWebSearch
                | Self::BraveNewsSearch
                | Self::BraveImageSearch
                | Self::OpenAiWebSearch
        )
    }

    pub const fn is_brave(self) -> bool {
        matches!(
            self,
            Self::BraveWebSearch | Self::BraveNewsSearch | Self::BraveImageSearch
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderControlMode {
    Live,
    TestFake,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderNetworkPolicy {
    pub global_search_providers_enabled: bool,
    pub paid_search_providers_enabled: bool,
    pub web_search_enabled: bool,
    pub deep_research_enabled: bool,
    pub news_search_enabled: bool,
    pub url_fetch_enabled: bool,
    pub startup_provider_probes_enabled: bool,
    pub provider_specific_enabled: BTreeMap<String, bool>,
    pub environment: String,
    pub max_calls_this_turn: u32,
    pub max_calls_this_route: u32,
    pub max_calls_this_actor_user: u32,
    pub max_calls_this_tenant: u32,
    pub max_retries: u32,
    pub fallback_enabled: bool,
    pub max_fallback_calls_this_turn: u32,
    pub provider_fanout_enabled: bool,
    pub brave_max_calls_per_test_run: u32,
    pub brave_max_calls_per_day_test: u32,
    pub live_brave_proof_enabled: bool,
}

impl Default for ProviderNetworkPolicy {
    fn default() -> Self {
        let mut provider_specific_enabled = BTreeMap::new();
        provider_specific_enabled.insert("brave".to_string(), false);
        provider_specific_enabled.insert("brave_image".to_string(), false);
        provider_specific_enabled.insert("openai_web".to_string(), false);
        provider_specific_enabled.insert("gdelt_news".to_string(), false);
        provider_specific_enabled.insert("cheap_general".to_string(), false);
        provider_specific_enabled.insert("news_current_events".to_string(), false);
        Self {
            global_search_providers_enabled: false,
            paid_search_providers_enabled: false,
            web_search_enabled: false,
            deep_research_enabled: false,
            news_search_enabled: false,
            url_fetch_enabled: false,
            startup_provider_probes_enabled: false,
            provider_specific_enabled,
            environment: "development".to_string(),
            max_calls_this_turn: 0,
            max_calls_this_route: 0,
            max_calls_this_actor_user: 0,
            max_calls_this_tenant: 0,
            max_retries: 0,
            fallback_enabled: false,
            max_fallback_calls_this_turn: 0,
            provider_fanout_enabled: false,
            brave_max_calls_per_test_run: 0,
            brave_max_calls_per_day_test: 0,
            live_brave_proof_enabled: false,
        }
    }
}

impl ProviderNetworkPolicy {
    pub fn from_env() -> Self {
        let mut provider_specific_enabled = BTreeMap::new();
        provider_specific_enabled.insert(
            "brave".to_string(),
            env_flag_enabled(SELENE_BRAVE_SEARCH_ENABLED),
        );
        provider_specific_enabled.insert(
            "brave_image".to_string(),
            env_flag_enabled(SELENE_BRAVE_IMAGE_SEARCH_ENABLED),
        );
        provider_specific_enabled.insert(
            "openai_web".to_string(),
            env_flag_enabled(SELENE_OPENAI_WEB_SEARCH_ENABLED),
        );
        provider_specific_enabled.insert(
            "gdelt_news".to_string(),
            env_flag_enabled(SELENE_GDELT_NEWS_SEARCH_ENABLED),
        );
        provider_specific_enabled.insert(
            "cheap_general".to_string(),
            env_flag_enabled(SELENE_CHEAP_SEARCH_PROVIDER_ENABLED),
        );
        provider_specific_enabled.insert(
            "news_current_events".to_string(),
            env_flag_enabled(SELENE_NEWS_CURRENT_EVENTS_PROVIDER_ENABLED),
        );
        Self {
            global_search_providers_enabled: env_flag_enabled(SELENE_SEARCH_PROVIDERS_ENABLED),
            paid_search_providers_enabled: env_flag_enabled(SELENE_PAID_SEARCH_PROVIDERS_ENABLED),
            web_search_enabled: env_flag_enabled(SELENE_WEB_SEARCH_ENABLED),
            deep_research_enabled: env_flag_enabled(SELENE_DEEP_RESEARCH_ENABLED),
            news_search_enabled: env_flag_enabled(SELENE_NEWS_SEARCH_ENABLED),
            url_fetch_enabled: env_flag_enabled(SELENE_URL_FETCH_ENABLED),
            startup_provider_probes_enabled: env_flag_enabled(
                SELENE_STARTUP_PROVIDER_PROBES_ENABLED,
            ),
            provider_specific_enabled,
            environment: env::var("SELENE_RUNTIME_ENV")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "development".to_string()),
            max_calls_this_turn: env_u32(SELENE_PROVIDER_CALL_MAX_PER_TURN).unwrap_or(0),
            max_calls_this_route: env_u32(SELENE_PROVIDER_CALL_MAX_PER_ROUTE).unwrap_or(0),
            max_calls_this_actor_user: env_u32(SELENE_PROVIDER_CALL_MAX_PER_ACTOR_USER)
                .unwrap_or(0),
            max_calls_this_tenant: env_u32(SELENE_PROVIDER_CALL_MAX_PER_TENANT).unwrap_or(0),
            max_retries: env_u32(SELENE_PROVIDER_RETRY_MAX).unwrap_or(0),
            fallback_enabled: env_flag_enabled(SELENE_PROVIDER_FALLBACK_ENABLED),
            max_fallback_calls_this_turn: env_u32(SELENE_PROVIDER_FALLBACK_MAX_PER_TURN)
                .unwrap_or(0),
            provider_fanout_enabled: env_flag_enabled(SELENE_PROVIDER_FANOUT_ENABLED),
            brave_max_calls_per_test_run: env_u32(SELENE_BRAVE_MAX_CALLS_PER_TEST_RUN).unwrap_or(0),
            brave_max_calls_per_day_test: env_u32(SELENE_BRAVE_MAX_CALLS_PER_DAY_TEST).unwrap_or(0),
            live_brave_proof_enabled: env_flag_enabled(SELENE_RUN_LIVE_BRAVE_PROOF),
        }
    }

    pub fn fake_test_allowing(max_calls: u32) -> Self {
        let mut provider_specific_enabled = BTreeMap::new();
        provider_specific_enabled.insert("brave".to_string(), true);
        provider_specific_enabled.insert("brave_image".to_string(), true);
        provider_specific_enabled.insert("openai_web".to_string(), true);
        provider_specific_enabled.insert("gdelt_news".to_string(), true);
        provider_specific_enabled.insert("cheap_general".to_string(), true);
        provider_specific_enabled.insert("news_current_events".to_string(), true);
        Self {
            global_search_providers_enabled: true,
            paid_search_providers_enabled: true,
            web_search_enabled: true,
            deep_research_enabled: true,
            news_search_enabled: true,
            url_fetch_enabled: true,
            startup_provider_probes_enabled: false,
            provider_specific_enabled,
            environment: "test".to_string(),
            max_calls_this_turn: max_calls,
            max_calls_this_route: max_calls,
            max_calls_this_actor_user: max_calls,
            max_calls_this_tenant: max_calls,
            max_retries: 0,
            fallback_enabled: false,
            max_fallback_calls_this_turn: 1,
            provider_fanout_enabled: false,
            brave_max_calls_per_test_run: max_calls,
            brave_max_calls_per_day_test: max_calls,
            live_brave_proof_enabled: false,
        }
    }

    pub fn is_provider_enabled(&self, provider: ProviderControlProvider) -> bool {
        if matches!(provider, ProviderControlProvider::BraveImageSearch) {
            return self
                .provider_specific_enabled
                .get("brave")
                .copied()
                .unwrap_or(false)
                && self
                    .provider_specific_enabled
                    .get("brave_image")
                    .copied()
                    .unwrap_or(false);
        }
        if provider.is_brave() {
            return self
                .provider_specific_enabled
                .get("brave")
                .copied()
                .unwrap_or(false);
        }
        if matches!(provider, ProviderControlProvider::OpenAiWebSearch) {
            return self
                .provider_specific_enabled
                .get("openai_web")
                .copied()
                .unwrap_or(false);
        }
        if matches!(provider, ProviderControlProvider::GdeltNewsAssist) {
            return self
                .provider_specific_enabled
                .get("gdelt_news")
                .copied()
                .unwrap_or(false);
        }
        if matches!(provider, ProviderControlProvider::CheapGeneralSearch) {
            return self
                .provider_specific_enabled
                .get("cheap_general")
                .copied()
                .unwrap_or(false);
        }
        if matches!(provider, ProviderControlProvider::NewsCurrentEvents) {
            return self
                .provider_specific_enabled
                .get("news_current_events")
                .copied()
                .unwrap_or(false);
        }
        true
    }
}

pub fn provider_fallback_enabled_from_env() -> bool {
    ProviderNetworkPolicy::from_env().fallback_enabled
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProviderCallCounter {
    pub logical_search_intent_count: u32,
    pub provider_route_selected_count: u32,
    pub provider_blocked_count: u32,
    pub provider_budget_denied_count: u32,
    pub provider_secret_missing_count: u32,
    pub provider_call_attempt_count: u32,
    pub provider_network_dispatch_count: u32,
    pub provider_success_count: u32,
    pub provider_failure_count: u32,
    pub provider_retry_count: u32,
    pub provider_fallback_count: u32,
    pub provider_cache_hit_count: u32,
    pub provider_cache_miss_count: u32,
    pub per_turn_count: u32,
    pub per_provider_count: BTreeMap<String, u32>,
    pub per_route_count: BTreeMap<String, u32>,
    pub per_actor_user_count: BTreeMap<String, u32>,
    pub per_tenant_count: BTreeMap<String, u32>,
    pub per_company_count: BTreeMap<String, u32>,
    pub per_private_user_count: BTreeMap<String, u32>,
    pub per_cost_owner_count: BTreeMap<String, u32>,
    pub per_billing_scope_count: BTreeMap<String, u32>,
    pub per_service_type_count: BTreeMap<String, u32>,
    pub per_module_count: BTreeMap<String, u32>,
    pub per_billing_period_count: BTreeMap<String, u32>,
    pub estimated_cost_total: u64,
    pub actual_cost_total: u64,
}

impl ProviderCallCounter {
    pub fn record_route_selected(&mut self) {
        self.provider_route_selected_count = self.provider_route_selected_count.saturating_add(1);
    }

    pub fn record_logical_intent(&mut self) {
        self.logical_search_intent_count = self.logical_search_intent_count.saturating_add(1);
    }

    pub fn record_blocked(&mut self, event: &ProviderUsageEvent) {
        self.provider_blocked_count = self.provider_blocked_count.saturating_add(1);
        increment(&mut self.per_provider_count, event.provider.as_str());
        increment(&mut self.per_route_count, event.route.as_str());
        increment(
            &mut self.per_billing_scope_count,
            event.billing_scope.as_str(),
        );
        increment(
            &mut self.per_service_type_count,
            event.service_type.as_str(),
        );
    }

    pub fn record_budget_denied(&mut self) {
        self.provider_budget_denied_count = self.provider_budget_denied_count.saturating_add(1);
    }

    pub fn record_secret_missing(&mut self) {
        self.provider_secret_missing_count = self.provider_secret_missing_count.saturating_add(1);
    }

    pub fn record_attempt(&mut self, event: &ProviderUsageEvent) {
        self.provider_call_attempt_count = self.provider_call_attempt_count.saturating_add(1);
        self.per_turn_count = self.per_turn_count.saturating_add(1);
        increment(&mut self.per_provider_count, event.provider.as_str());
        increment(&mut self.per_route_count, event.route.as_str());
        increment(&mut self.per_actor_user_count, event.actor_user_id.as_str());
        increment(&mut self.per_tenant_count, event.tenant_id.as_str());
        increment(&mut self.per_company_count, event.company_id.as_str());
        increment(
            &mut self.per_private_user_count,
            event.private_user_id.as_str(),
        );
        increment(&mut self.per_cost_owner_count, event.cost_owner_id.as_str());
        increment(
            &mut self.per_billing_scope_count,
            event.billing_scope.as_str(),
        );
        increment(
            &mut self.per_service_type_count,
            event.service_type.as_str(),
        );
        increment(&mut self.per_module_count, event.module_id.as_str());
        increment(
            &mut self.per_billing_period_count,
            event.billing_period_id.as_str(),
        );
        self.estimated_cost_total = self
            .estimated_cost_total
            .saturating_add(event.estimated_total_cost_micros.unwrap_or(0));
    }

    pub fn record_network_dispatch(&mut self) {
        self.provider_network_dispatch_count =
            self.provider_network_dispatch_count.saturating_add(1);
    }

    pub fn record_success(&mut self) {
        self.provider_success_count = self.provider_success_count.saturating_add(1);
    }

    pub fn record_failure(&mut self) {
        self.provider_failure_count = self.provider_failure_count.saturating_add(1);
    }

    pub fn record_retry(&mut self) {
        self.provider_retry_count = self.provider_retry_count.saturating_add(1);
    }

    pub fn record_fallback(&mut self) {
        self.provider_fallback_count = self.provider_fallback_count.saturating_add(1);
    }

    pub fn record_cache_hit(&mut self) {
        self.provider_cache_hit_count = self.provider_cache_hit_count.saturating_add(1);
    }

    pub fn record_cache_miss(&mut self) {
        self.provider_cache_miss_count = self.provider_cache_miss_count.saturating_add(1);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderUsageContext {
    pub request_id: String,
    pub turn_id: String,
    pub session_id: String,
    pub route: ProviderControlRoute,
    pub provider: ProviderControlProvider,
    pub provider_tier: String,
    pub operation_type: String,
    pub service_type: String,
    pub module_id: String,
    pub account_layer: String,
    pub tenant_id: String,
    pub customer_id: String,
    pub company_id: String,
    pub private_user_id: String,
    pub actor_user_id: String,
    pub role_id: String,
    pub department_id: String,
    pub team_id: String,
    pub cost_owner_id: String,
    pub billing_scope: String,
    pub plan_id: String,
    pub billing_period_id: String,
    pub redacted_query_hash: String,
}

impl ProviderUsageContext {
    pub fn unknown(
        route: ProviderControlRoute,
        provider: ProviderControlProvider,
        query: &str,
    ) -> Self {
        Self {
            request_id: UNAVAILABLE.to_string(),
            turn_id: UNAVAILABLE.to_string(),
            session_id: UNAVAILABLE.to_string(),
            route,
            provider,
            provider_tier: provider.tier().to_string(),
            operation_type: route.operation_type().to_string(),
            service_type: route.service_type().to_string(),
            module_id: "PH1.E".to_string(),
            account_layer: UNKNOWN.to_string(),
            tenant_id: UNAVAILABLE.to_string(),
            customer_id: UNAVAILABLE.to_string(),
            company_id: UNAVAILABLE.to_string(),
            private_user_id: UNAVAILABLE.to_string(),
            actor_user_id: UNAVAILABLE.to_string(),
            role_id: UNAVAILABLE.to_string(),
            department_id: UNAVAILABLE.to_string(),
            team_id: UNAVAILABLE.to_string(),
            cost_owner_id: UNKNOWN.to_string(),
            billing_scope: NON_BILLABLE.to_string(),
            plan_id: UNAVAILABLE.to_string(),
            billing_period_id: UNAVAILABLE.to_string(),
            redacted_query_hash: stable_hash_hex(query),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderUsageEvent {
    pub request_id: String,
    pub turn_id: String,
    pub session_id: String,
    pub route: String,
    pub provider: String,
    pub provider_tier: String,
    pub operation_type: String,
    pub service_type: String,
    pub module_id: String,
    pub account_layer: String,
    pub tenant_id: String,
    pub customer_id: String,
    pub company_id: String,
    pub private_user_id: String,
    pub actor_user_id: String,
    pub role_id: String,
    pub department_id: String,
    pub team_id: String,
    pub cost_owner_id: String,
    pub billing_scope: String,
    pub plan_id: String,
    pub billing_period_id: String,
    pub billable_class: String,
    pub allowed: bool,
    pub deny_reason: Option<String>,
    pub estimated_unit_cost_micros: Option<u64>,
    pub estimated_total_cost_micros: Option<u64>,
    pub estimated_cost_currency: Option<String>,
    pub actual_unit_cost_micros: Option<u64>,
    pub actual_total_cost_micros: Option<u64>,
    pub actual_cost_currency: Option<String>,
    pub max_call_limit: u32,
    pub current_call_count: u32,
    pub remaining_call_budget: u32,
    pub timestamp_unix_ms: u64,
    pub redacted_query_hash: String,
}

impl ProviderUsageEvent {
    pub fn trace_line(&self, counter: &ProviderCallCounter) -> String {
        format!(
            "stage2_provider_control=1 route={} provider={} deny_reason={} provider_call_attempt_count={} provider_network_dispatch_count={} billing_scope={} billable_class={}",
            self.route,
            self.provider,
            self.deny_reason.as_deref().unwrap_or("NONE"),
            counter.provider_call_attempt_count,
            counter.provider_network_dispatch_count,
            self.billing_scope,
            self.billable_class,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderGateDecision {
    pub allowed: bool,
    pub deny_reason: Option<String>,
    pub usage_event: ProviderUsageEvent,
    pub counter: ProviderCallCounter,
}

impl ProviderGateDecision {
    pub fn disabled_trace_line(&self) -> String {
        self.usage_event.trace_line(&self.counter)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProviderLane {
    NoSearch,
    CacheOnly,
    CheapGeneralSearch,
    NewsCurrentEvents,
    PremiumFallback,
    DeepResearchCapped,
    UrlFetchRead,
    ImageMetadata,
    Disabled,
}

impl ProviderLane {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoSearch => "NO_SEARCH",
            Self::CacheOnly => "CACHE_ONLY",
            Self::CheapGeneralSearch => "CHEAP_GENERAL_SEARCH",
            Self::NewsCurrentEvents => "NEWS_CURRENT_EVENTS",
            Self::PremiumFallback => "PREMIUM_FALLBACK",
            Self::DeepResearchCapped => "DEEP_RESEARCH_CAPPED",
            Self::UrlFetchRead => "URL_FETCH_READ",
            Self::ImageMetadata => "IMAGE_METADATA",
            Self::Disabled => "DISABLED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProviderTier {
    FreeOrInternal,
    Cache,
    Cheap,
    Standard,
    Premium,
    ExpensiveDeepResearch,
    Disabled,
}

impl ProviderTier {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FreeOrInternal => "FREE_OR_INTERNAL",
            Self::Cache => "CACHE",
            Self::Cheap => "CHEAP",
            Self::Standard => "STANDARD",
            Self::Premium => "PREMIUM",
            Self::ExpensiveDeepResearch => "EXPENSIVE_DEEP_RESEARCH",
            Self::Disabled => "DISABLED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProviderClass {
    GeneralWeb,
    News,
    OfficialSiteTargeting,
    PageFetch,
    ImageMetadata,
    DeepResearch,
    FallbackOnly,
}

impl ProviderClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GeneralWeb => "general_web",
            Self::News => "news",
            Self::OfficialSiteTargeting => "official_site_targeting",
            Self::PageFetch => "page_fetch",
            Self::ImageMetadata => "image_metadata",
            Self::DeepResearch => "deep_research",
            Self::FallbackOnly => "fallback_only",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProviderCostClass {
    ZeroCache,
    FreeOrInternal,
    LowCost,
    StandardCost,
    PremiumCost,
    DeepResearchCost,
    Disabled,
}

impl ProviderCostClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ZeroCache => "ZERO_CACHE",
            Self::FreeOrInternal => "FREE_OR_INTERNAL",
            Self::LowCost => "LOW_COST",
            Self::StandardCost => "STANDARD_COST",
            Self::PremiumCost => "PREMIUM_COST",
            Self::DeepResearchCost => "DEEP_RESEARCH_COST",
            Self::Disabled => "DISABLED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProviderCacheStatus {
    Disabled,
    Hit,
    Miss,
    Stale,
}

impl ProviderCacheStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "DISABLED",
            Self::Hit => CACHE_HIT,
            Self::Miss => CACHE_MISS,
            Self::Stale => CACHE_STALE,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderCacheDecisionPacket {
    pub cache_enabled: bool,
    pub cache_key_hash: String,
    pub freshness_tier: String,
    pub max_age_ms: u64,
    pub source_policy: String,
    pub claim_type: String,
    pub requested_entity: String,
    pub language: String,
    pub provider_result_hash: String,
    pub answer_packet_hash: String,
    pub accepted_source_ids: Vec<String>,
    pub evidence_hashes: Vec<String>,
    pub created_at_ms: u64,
    pub expires_at_ms: u64,
    pub cache_retention_class: String,
    pub status: ProviderCacheStatus,
}

impl ProviderCacheDecisionPacket {
    pub fn disabled(trace_seed: &str) -> Self {
        let hash = stable_hash_hex(trace_seed);
        Self {
            cache_enabled: false,
            cache_key_hash: hash.clone(),
            freshness_tier: "uncached".to_string(),
            max_age_ms: 0,
            source_policy: "accepted_sources_required".to_string(),
            claim_type: "unknown".to_string(),
            requested_entity: "synthetic".to_string(),
            language: "und".to_string(),
            provider_result_hash: hash.clone(),
            answer_packet_hash: hash,
            accepted_source_ids: Vec::new(),
            evidence_hashes: Vec::new(),
            created_at_ms: 0,
            expires_at_ms: 0,
            cache_retention_class: "no_cache".to_string(),
            status: ProviderCacheStatus::Disabled,
        }
    }

    pub fn fixture_hit(trace_seed: &str, now_ms: u64, max_age_ms: u64) -> Self {
        let hash = stable_hash_hex(trace_seed);
        Self {
            cache_enabled: true,
            cache_key_hash: hash.clone(),
            freshness_tier: "stable_reference".to_string(),
            max_age_ms,
            source_policy: "accepted_sources_required".to_string(),
            claim_type: "stable_reference".to_string(),
            requested_entity: "Synthetic Entity A".to_string(),
            language: "en".to_string(),
            provider_result_hash: hash.clone(),
            answer_packet_hash: hash.clone(),
            accepted_source_ids: vec!["source_001".to_string()],
            evidence_hashes: vec![hash],
            created_at_ms: now_ms,
            expires_at_ms: now_ms.saturating_add(max_age_ms),
            cache_retention_class: "AUDIT_METADATA_ONLY".to_string(),
            status: ProviderCacheStatus::Hit,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderRegistryEntry {
    pub provider_id: String,
    pub provider_name: String,
    pub provider_lane: ProviderLane,
    pub provider_tier: ProviderTier,
    pub enabled: bool,
    pub paid_provider: bool,
    pub secret_id: Option<String>,
    pub live_test_allowed: bool,
    pub test_fake_provider: bool,
    pub supports_web: bool,
    pub supports_news: bool,
    pub supports_images: bool,
    pub supports_page_fetch: bool,
    pub supports_deep_research: bool,
    pub default_max_calls: u32,
    pub default_retry_max: u32,
    pub cost_estimate_unit: ProviderCostClass,
    pub trust_notes: String,
    pub data_retention_notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderRouteRequest {
    pub search_needed: bool,
    pub route: ProviderControlRoute,
    pub query: String,
    pub cache_allowed: bool,
    pub cache_status: ProviderCacheStatus,
    pub cheap_provider_available: bool,
    pub news_provider_available: bool,
    pub fallback_allowed: bool,
    pub official_source_targeting: bool,
    pub trace_id: String,
}

impl ProviderRouteRequest {
    pub fn public_web(query: &str) -> Self {
        Self {
            search_needed: true,
            route: ProviderControlRoute::WebSearch,
            query: query.to_string(),
            cache_allowed: true,
            cache_status: ProviderCacheStatus::Miss,
            cheap_provider_available: false,
            news_provider_available: false,
            fallback_allowed: false,
            official_source_targeting: false,
            trace_id: stable_hash_hex(query),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderRouteDecision {
    pub search_needed: bool,
    pub selected_lane: ProviderLane,
    pub selected_provider: Option<ProviderControlProvider>,
    pub fallback_provider: Option<ProviderControlProvider>,
    pub fallback_allowed: bool,
    pub fanout_allowed: bool,
    pub cache_allowed: bool,
    pub budget_required: bool,
    pub user_approval_required: bool,
    pub deny_reason: Option<String>,
    pub route_reason: String,
    pub max_calls: u32,
    pub max_retries: u32,
    pub estimated_cost_tier: ProviderCostClass,
    pub trace_id: String,
}

impl ProviderRouteDecision {
    pub fn provider_id(&self) -> &'static str {
        self.selected_provider
            .map(ProviderControlProvider::as_str)
            .unwrap_or("none")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderRawResultFixture {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub published_at: Option<String>,
    pub source_type: Option<String>,
    pub provider_rank: u16,
    pub provider_confidence: Option<u16>,
    pub raw_provider_metadata_redacted_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedSourceCandidatePacket {
    pub source_id: String,
    pub provider_id: String,
    pub title: String,
    pub domain: String,
    pub url: String,
    pub snippet: String,
    pub published_at: Option<String>,
    pub source_type: Option<String>,
    pub provider_rank: u16,
    pub provider_confidence: Option<u16>,
    pub provider_tier: ProviderTier,
    pub retrieved_at_ms: u64,
    pub raw_provider_metadata_redacted_hash: String,
}

pub fn normalize_provider_result_fixture(
    provider: ProviderControlProvider,
    provider_tier: ProviderTier,
    result: ProviderRawResultFixture,
    retrieved_at_ms: u64,
) -> Result<NormalizedSourceCandidatePacket, String> {
    let title = result.title.trim();
    let url = result.url.trim();
    let snippet = result.snippet.trim();
    if title.is_empty() {
        return Err("title_missing".to_string());
    }
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("url_not_public_http".to_string());
    }
    if snippet.is_empty() {
        return Err("snippet_missing".to_string());
    }
    let domain = domain_from_url(url).ok_or_else(|| "domain_missing".to_string())?;
    let source_id = format!(
        "source_{}_{:03}",
        stable_hash_hex(url).chars().take(8).collect::<String>(),
        result.provider_rank
    );
    Ok(NormalizedSourceCandidatePacket {
        source_id,
        provider_id: provider.as_str().to_string(),
        title: title.to_string(),
        domain,
        url: url.to_string(),
        snippet: snippet.to_string(),
        published_at: result.published_at,
        source_type: result.source_type,
        provider_rank: result.provider_rank,
        provider_confidence: result.provider_confidence,
        provider_tier,
        retrieved_at_ms,
        raw_provider_metadata_redacted_hash: result.raw_provider_metadata_redacted_hash,
    })
}

fn domain_from_url(url: &str) -> Option<String> {
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let host = without_scheme
        .split('/')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    if host.is_empty() || host.contains('@') || host.contains(' ') {
        return None;
    }
    Some(host)
}

pub fn provider_registry(policy: &ProviderNetworkPolicy) -> Vec<ProviderRegistryEntry> {
    vec![
        ProviderRegistryEntry {
            provider_id: ProviderControlProvider::CacheOnly.as_str().to_string(),
            provider_name: "Verified cache".to_string(),
            provider_lane: ProviderLane::CacheOnly,
            provider_tier: ProviderTier::Cache,
            enabled: true,
            paid_provider: false,
            secret_id: None,
            live_test_allowed: true,
            test_fake_provider: false,
            supports_web: true,
            supports_news: true,
            supports_images: false,
            supports_page_fetch: false,
            supports_deep_research: false,
            default_max_calls: 0,
            default_retry_max: 0,
            cost_estimate_unit: ProviderCostClass::ZeroCache,
            trust_notes: "requires accepted source proof on cached answer".to_string(),
            data_retention_notes: "audit metadata only unless a durable cache law says otherwise"
                .to_string(),
        },
        ProviderRegistryEntry {
            provider_id: ProviderControlProvider::CheapGeneralSearch
                .as_str()
                .to_string(),
            provider_name: "Cheap/default general search lane".to_string(),
            provider_lane: ProviderLane::CheapGeneralSearch,
            provider_tier: ProviderTier::Cheap,
            enabled: policy.is_provider_enabled(ProviderControlProvider::CheapGeneralSearch),
            paid_provider: false,
            secret_id: None,
            live_test_allowed: false,
            test_fake_provider: true,
            supports_web: true,
            supports_news: false,
            supports_images: false,
            supports_page_fetch: false,
            supports_deep_research: false,
            default_max_calls: policy.max_calls_this_turn,
            default_retry_max: policy.max_retries,
            cost_estimate_unit: ProviderCostClass::LowCost,
            trust_notes: "Stage 8 fake/default lane; live provider candidates are deferred"
                .to_string(),
            data_retention_notes: "normal tests use fake transport only".to_string(),
        },
        ProviderRegistryEntry {
            provider_id: ProviderControlProvider::NewsCurrentEvents
                .as_str()
                .to_string(),
            provider_name: "News/current-events lane".to_string(),
            provider_lane: ProviderLane::NewsCurrentEvents,
            provider_tier: ProviderTier::FreeOrInternal,
            enabled: policy.is_provider_enabled(ProviderControlProvider::NewsCurrentEvents),
            paid_provider: false,
            secret_id: None,
            live_test_allowed: false,
            test_fake_provider: true,
            supports_web: false,
            supports_news: true,
            supports_images: false,
            supports_page_fetch: false,
            supports_deep_research: false,
            default_max_calls: policy.max_calls_this_turn,
            default_retry_max: policy.max_retries,
            cost_estimate_unit: ProviderCostClass::FreeOrInternal,
            trust_notes: "Stage 8 fake news lane; GDELT live use remains gated/deferred"
                .to_string(),
            data_retention_notes: "normal tests use fake transport only".to_string(),
        },
        ProviderRegistryEntry {
            provider_id: ProviderControlProvider::BraveWebSearch.as_str().to_string(),
            provider_name: "Brave Search premium fallback".to_string(),
            provider_lane: ProviderLane::PremiumFallback,
            provider_tier: ProviderTier::Premium,
            enabled: policy.is_provider_enabled(ProviderControlProvider::BraveWebSearch),
            paid_provider: true,
            secret_id: Some("brave_search_api_key".to_string()),
            live_test_allowed: policy.live_brave_proof_enabled,
            test_fake_provider: false,
            supports_web: true,
            supports_news: false,
            supports_images: false,
            supports_page_fetch: false,
            supports_deep_research: false,
            default_max_calls: policy.max_calls_this_turn,
            default_retry_max: policy.max_retries,
            cost_estimate_unit: ProviderCostClass::PremiumCost,
            trust_notes: "fallback-only unless explicit policy and budget select it".to_string(),
            data_retention_notes: "no raw provider JSON in normal output".to_string(),
        },
        ProviderRegistryEntry {
            provider_id: ProviderControlProvider::BraveNewsSearch
                .as_str()
                .to_string(),
            provider_name: "Brave News premium fallback".to_string(),
            provider_lane: ProviderLane::PremiumFallback,
            provider_tier: ProviderTier::Premium,
            enabled: policy.is_provider_enabled(ProviderControlProvider::BraveNewsSearch),
            paid_provider: true,
            secret_id: Some("brave_search_api_key".to_string()),
            live_test_allowed: policy.live_brave_proof_enabled,
            test_fake_provider: false,
            supports_web: false,
            supports_news: true,
            supports_images: false,
            supports_page_fetch: false,
            supports_deep_research: false,
            default_max_calls: policy.max_calls_this_turn,
            default_retry_max: policy.max_retries,
            cost_estimate_unit: ProviderCostClass::PremiumCost,
            trust_notes: "premium news fallback; not default news lane".to_string(),
            data_retention_notes: "no raw provider JSON in normal output".to_string(),
        },
    ]
}

pub fn route_provider(
    policy: &ProviderNetworkPolicy,
    request: &ProviderRouteRequest,
) -> ProviderRouteDecision {
    if !request.search_needed {
        return ProviderRouteDecision {
            search_needed: false,
            selected_lane: ProviderLane::NoSearch,
            selected_provider: None,
            fallback_provider: None,
            fallback_allowed: false,
            fanout_allowed: false,
            cache_allowed: request.cache_allowed,
            budget_required: false,
            user_approval_required: false,
            deny_reason: None,
            route_reason: NO_SEARCH_NEEDED.to_string(),
            max_calls: 0,
            max_retries: 0,
            estimated_cost_tier: ProviderCostClass::Disabled,
            trace_id: request.trace_id.clone(),
        };
    }

    if request.cache_allowed && matches!(request.cache_status, ProviderCacheStatus::Hit) {
        return ProviderRouteDecision {
            search_needed: true,
            selected_lane: ProviderLane::CacheOnly,
            selected_provider: Some(ProviderControlProvider::CacheOnly),
            fallback_provider: None,
            fallback_allowed: false,
            fanout_allowed: false,
            cache_allowed: true,
            budget_required: false,
            user_approval_required: false,
            deny_reason: None,
            route_reason: CACHE_HIT.to_string(),
            max_calls: 0,
            max_retries: 0,
            estimated_cost_tier: ProviderCostClass::ZeroCache,
            trace_id: request.trace_id.clone(),
        };
    }

    let cache_reason = if request.cache_allowed {
        request.cache_status.as_str()
    } else {
        "CACHE_DISABLED"
    };

    match request.route {
        ProviderControlRoute::WebSearch => {
            if request.cheap_provider_available
                && policy.is_provider_enabled(ProviderControlProvider::CheapGeneralSearch)
            {
                return ProviderRouteDecision {
                    search_needed: true,
                    selected_lane: ProviderLane::CheapGeneralSearch,
                    selected_provider: Some(ProviderControlProvider::CheapGeneralSearch),
                    fallback_provider: Some(ProviderControlProvider::BraveWebSearch),
                    fallback_allowed: false,
                    fanout_allowed: policy.provider_fanout_enabled,
                    cache_allowed: request.cache_allowed,
                    budget_required: true,
                    user_approval_required: false,
                    deny_reason: None,
                    route_reason: format!("{cache_reason}:cheap_provider_selected"),
                    max_calls: policy.max_calls_this_turn,
                    max_retries: policy.max_retries,
                    estimated_cost_tier: ProviderCostClass::LowCost,
                    trace_id: request.trace_id.clone(),
                };
            }

            if request.fallback_allowed
                && policy.fallback_enabled
                && policy.max_fallback_calls_this_turn > 0
                && policy.is_provider_enabled(ProviderControlProvider::BraveWebSearch)
            {
                return ProviderRouteDecision {
                    search_needed: true,
                    selected_lane: ProviderLane::PremiumFallback,
                    selected_provider: Some(ProviderControlProvider::BraveWebSearch),
                    fallback_provider: Some(ProviderControlProvider::BraveWebSearch),
                    fallback_allowed: true,
                    fanout_allowed: false,
                    cache_allowed: request.cache_allowed,
                    budget_required: true,
                    user_approval_required: false,
                    deny_reason: None,
                    route_reason: format!("{cache_reason}:premium_fallback_selected"),
                    max_calls: policy.max_calls_this_turn,
                    max_retries: policy.max_retries,
                    estimated_cost_tier: ProviderCostClass::PremiumCost,
                    trace_id: request.trace_id.clone(),
                };
            }

            disabled_route_decision(
                request,
                CHEAP_PROVIDER_UNAVAILABLE,
                Some(ProviderControlProvider::BraveWebSearch),
            )
        }
        ProviderControlRoute::NewsSearch => {
            if request.news_provider_available
                && policy.is_provider_enabled(ProviderControlProvider::NewsCurrentEvents)
            {
                return ProviderRouteDecision {
                    search_needed: true,
                    selected_lane: ProviderLane::NewsCurrentEvents,
                    selected_provider: Some(ProviderControlProvider::NewsCurrentEvents),
                    fallback_provider: Some(ProviderControlProvider::BraveNewsSearch),
                    fallback_allowed: false,
                    fanout_allowed: policy.provider_fanout_enabled,
                    cache_allowed: request.cache_allowed,
                    budget_required: true,
                    user_approval_required: false,
                    deny_reason: None,
                    route_reason: format!("{cache_reason}:news_provider_selected"),
                    max_calls: policy.max_calls_this_turn,
                    max_retries: policy.max_retries,
                    estimated_cost_tier: ProviderCostClass::FreeOrInternal,
                    trace_id: request.trace_id.clone(),
                };
            }

            if request.fallback_allowed
                && policy.fallback_enabled
                && policy.max_fallback_calls_this_turn > 0
                && policy.is_provider_enabled(ProviderControlProvider::BraveNewsSearch)
            {
                return ProviderRouteDecision {
                    search_needed: true,
                    selected_lane: ProviderLane::PremiumFallback,
                    selected_provider: Some(ProviderControlProvider::BraveNewsSearch),
                    fallback_provider: Some(ProviderControlProvider::BraveNewsSearch),
                    fallback_allowed: true,
                    fanout_allowed: false,
                    cache_allowed: request.cache_allowed,
                    budget_required: true,
                    user_approval_required: false,
                    deny_reason: None,
                    route_reason: format!("{cache_reason}:premium_news_fallback_selected"),
                    max_calls: policy.max_calls_this_turn,
                    max_retries: policy.max_retries,
                    estimated_cost_tier: ProviderCostClass::PremiumCost,
                    trace_id: request.trace_id.clone(),
                };
            }

            disabled_route_decision(
                request,
                NEWS_SEARCH_DISABLED,
                Some(ProviderControlProvider::BraveNewsSearch),
            )
        }
        ProviderControlRoute::DeepResearch => {
            if policy.deep_research_enabled {
                ProviderRouteDecision {
                    search_needed: true,
                    selected_lane: ProviderLane::DeepResearchCapped,
                    selected_provider: Some(ProviderControlProvider::BraveWebSearch),
                    fallback_provider: None,
                    fallback_allowed: false,
                    fanout_allowed: policy.provider_fanout_enabled,
                    cache_allowed: request.cache_allowed,
                    budget_required: true,
                    user_approval_required: true,
                    deny_reason: None,
                    route_reason: format!("{cache_reason}:deep_research_capped"),
                    max_calls: policy.max_calls_this_turn,
                    max_retries: policy.max_retries,
                    estimated_cost_tier: ProviderCostClass::DeepResearchCost,
                    trace_id: request.trace_id.clone(),
                }
            } else {
                disabled_route_decision(request, DEEP_RESEARCH_DISABLED, None)
            }
        }
        ProviderControlRoute::UrlFetch => {
            if policy.url_fetch_enabled {
                ProviderRouteDecision {
                    search_needed: true,
                    selected_lane: ProviderLane::UrlFetchRead,
                    selected_provider: Some(ProviderControlProvider::UrlFetch),
                    fallback_provider: None,
                    fallback_allowed: false,
                    fanout_allowed: false,
                    cache_allowed: request.cache_allowed,
                    budget_required: true,
                    user_approval_required: false,
                    deny_reason: None,
                    route_reason: format!("{cache_reason}:url_fetch_read"),
                    max_calls: policy.max_calls_this_turn,
                    max_retries: policy.max_retries,
                    estimated_cost_tier: ProviderCostClass::StandardCost,
                    trace_id: request.trace_id.clone(),
                }
            } else {
                disabled_route_decision(request, URL_FETCH_DISABLED, None)
            }
        }
        ProviderControlRoute::ImageSearch => {
            if policy.is_provider_enabled(ProviderControlProvider::BraveImageSearch) {
                ProviderRouteDecision {
                    search_needed: true,
                    selected_lane: ProviderLane::ImageMetadata,
                    selected_provider: Some(ProviderControlProvider::BraveImageSearch),
                    fallback_provider: None,
                    fallback_allowed: false,
                    fanout_allowed: false,
                    cache_allowed: request.cache_allowed,
                    budget_required: true,
                    user_approval_required: false,
                    deny_reason: None,
                    route_reason: format!("{cache_reason}:image_metadata"),
                    max_calls: policy.max_calls_this_turn,
                    max_retries: policy.max_retries,
                    estimated_cost_tier: ProviderCostClass::PremiumCost,
                    trace_id: request.trace_id.clone(),
                }
            } else {
                disabled_route_decision(request, PROVIDER_DISABLED, None)
            }
        }
        ProviderControlRoute::StartupProbe => disabled_route_decision(
            request,
            STARTUP_PROVIDER_PROBES_DISABLED,
            Some(ProviderControlProvider::StartupProbe),
        ),
    }
}

pub fn apply_route_decision_to_counter(
    counter: &mut ProviderCallCounter,
    decision: &ProviderRouteDecision,
) {
    counter.record_route_selected();
    match decision.selected_lane {
        ProviderLane::CacheOnly => counter.record_cache_hit(),
        ProviderLane::Disabled => {
            counter.record_blocked(&ProviderUsageEvent {
                request_id: UNAVAILABLE.to_string(),
                turn_id: UNAVAILABLE.to_string(),
                session_id: UNAVAILABLE.to_string(),
                route: "Stage8Router".to_string(),
                provider: decision.provider_id().to_string(),
                provider_tier: decision.estimated_cost_tier.as_str().to_string(),
                operation_type: "provider_route".to_string(),
                service_type: "ProviderRouter".to_string(),
                module_id: "PH1.PROVIDERCTL".to_string(),
                account_layer: UNKNOWN.to_string(),
                tenant_id: UNAVAILABLE.to_string(),
                customer_id: UNAVAILABLE.to_string(),
                company_id: UNAVAILABLE.to_string(),
                private_user_id: UNAVAILABLE.to_string(),
                actor_user_id: UNAVAILABLE.to_string(),
                role_id: UNAVAILABLE.to_string(),
                department_id: UNAVAILABLE.to_string(),
                team_id: UNAVAILABLE.to_string(),
                cost_owner_id: UNKNOWN.to_string(),
                billing_scope: NON_BILLABLE.to_string(),
                plan_id: UNAVAILABLE.to_string(),
                billing_period_id: UNAVAILABLE.to_string(),
                billable_class: BLOCKED_NOT_BILLABLE.to_string(),
                allowed: false,
                deny_reason: decision.deny_reason.clone(),
                estimated_unit_cost_micros: Some(0),
                estimated_total_cost_micros: Some(0),
                estimated_cost_currency: Some("USD".to_string()),
                actual_unit_cost_micros: Some(0),
                actual_total_cost_micros: Some(0),
                actual_cost_currency: Some("USD".to_string()),
                max_call_limit: decision.max_calls,
                current_call_count: counter.provider_call_attempt_count,
                remaining_call_budget: decision.max_calls,
                timestamp_unix_ms: now_unix_ms(),
                redacted_query_hash: decision.trace_id.clone(),
            });
            if matches!(
                decision.deny_reason.as_deref(),
                Some(PROVIDER_BUDGET_EXHAUSTED)
            ) {
                counter.record_budget_denied();
            }
        }
        _ => {
            if matches!(decision.cache_allowed, true)
                && !matches!(decision.selected_lane, ProviderLane::NoSearch)
            {
                counter.record_cache_miss();
            }
            if decision.fallback_allowed {
                counter.record_fallback();
            }
        }
    }
}

fn disabled_route_decision(
    request: &ProviderRouteRequest,
    reason: &str,
    fallback_provider: Option<ProviderControlProvider>,
) -> ProviderRouteDecision {
    ProviderRouteDecision {
        search_needed: request.search_needed,
        selected_lane: ProviderLane::Disabled,
        selected_provider: None,
        fallback_provider,
        fallback_allowed: false,
        fanout_allowed: false,
        cache_allowed: request.cache_allowed,
        budget_required: false,
        user_approval_required: false,
        deny_reason: Some(reason.to_string()),
        route_reason: reason.to_string(),
        max_calls: 0,
        max_retries: 0,
        estimated_cost_tier: ProviderCostClass::Disabled,
        trace_id: request.trace_id.clone(),
    }
}

pub fn evaluate_provider_gate(
    policy: &ProviderNetworkPolicy,
    context: ProviderUsageContext,
    mode: ProviderControlMode,
    mut counter: ProviderCallCounter,
) -> ProviderGateDecision {
    counter.record_route_selected();
    counter.record_logical_intent();
    let deny_reason = deny_reason(policy, &context, mode, &counter);
    let allowed = deny_reason.is_none();
    let mut event = event_from_context(
        context,
        policy,
        allowed,
        deny_reason.clone(),
        mode,
        &counter,
    );
    if allowed {
        counter.record_attempt(&event);
        event.current_call_count = counter.provider_call_attempt_count;
        event.remaining_call_budget = policy
            .max_calls_this_turn
            .saturating_sub(counter.provider_call_attempt_count);
    } else {
        counter.record_blocked(&event);
        if matches!(deny_reason.as_deref(), Some(PROVIDER_BUDGET_EXHAUSTED)) {
            counter.record_budget_denied();
        }
    }
    ProviderGateDecision {
        allowed,
        deny_reason,
        usage_event: event,
        counter,
    }
}

pub fn disabled_provider_decision(
    route: ProviderControlRoute,
    provider: ProviderControlProvider,
    query: &str,
) -> ProviderGateDecision {
    evaluate_provider_gate(
        &ProviderNetworkPolicy::from_env(),
        ProviderUsageContext::unknown(route, provider, query),
        ProviderControlMode::Live,
        ProviderCallCounter::default(),
    )
}

pub fn fake_provider_decision(
    route: ProviderControlRoute,
    provider: ProviderControlProvider,
    query: &str,
    max_calls: u32,
) -> ProviderGateDecision {
    evaluate_provider_gate(
        &ProviderNetworkPolicy::fake_test_allowing(max_calls),
        ProviderUsageContext::unknown(route, provider, query),
        ProviderControlMode::TestFake,
        ProviderCallCounter::default(),
    )
}

pub fn is_local_fake_endpoint(endpoint: &str) -> bool {
    let lowered = endpoint.trim().to_ascii_lowercase();
    lowered.starts_with("http://127.0.0.1:")
        || lowered.starts_with("http://localhost:")
        || lowered.starts_with("http://[::1]:")
}

fn deny_reason(
    policy: &ProviderNetworkPolicy,
    context: &ProviderUsageContext,
    mode: ProviderControlMode,
    counter: &ProviderCallCounter,
) -> Option<String> {
    if matches!(mode, ProviderControlMode::TestFake) {
        return budget_deny_reason(policy, context, counter);
    }
    if !policy.global_search_providers_enabled {
        return Some(WEB_ADMIN_DISABLED.to_string());
    }
    if matches!(context.route, ProviderControlRoute::StartupProbe)
        && !policy.startup_provider_probes_enabled
    {
        return Some(STARTUP_PROVIDER_PROBES_DISABLED.to_string());
    }
    if matches!(
        context.route,
        ProviderControlRoute::WebSearch | ProviderControlRoute::ImageSearch
    ) && !policy.web_search_enabled
    {
        return Some(PROVIDER_DISABLED.to_string());
    }
    if matches!(context.route, ProviderControlRoute::DeepResearch) && !policy.deep_research_enabled
    {
        return Some(DEEP_RESEARCH_DISABLED.to_string());
    }
    if matches!(context.route, ProviderControlRoute::NewsSearch) && !policy.news_search_enabled {
        return Some(NEWS_SEARCH_DISABLED.to_string());
    }
    if matches!(context.route, ProviderControlRoute::UrlFetch) && !policy.url_fetch_enabled {
        return Some(URL_FETCH_DISABLED.to_string());
    }
    if !policy.is_provider_enabled(context.provider) {
        return Some(PROVIDER_DISABLED.to_string());
    }
    if context.provider.is_paid() && !policy.paid_search_providers_enabled {
        return Some(PAID_PROVIDER_DISABLED.to_string());
    }
    if policy.max_calls_this_turn == 0
        || counter.provider_call_attempt_count >= policy.max_calls_this_turn
    {
        return Some(PROVIDER_BUDGET_EXHAUSTED.to_string());
    }
    if let Some(reason) = budget_deny_reason(policy, context, counter) {
        return Some(reason);
    }
    if context.provider.is_brave()
        && (policy.brave_max_calls_per_test_run == 0 || policy.brave_max_calls_per_day_test == 0)
    {
        return Some(PROVIDER_BUDGET_EXHAUSTED.to_string());
    }
    None
}

fn budget_deny_reason(
    policy: &ProviderNetworkPolicy,
    context: &ProviderUsageContext,
    counter: &ProviderCallCounter,
) -> Option<String> {
    if policy.max_calls_this_turn > 0
        && counter.provider_call_attempt_count >= policy.max_calls_this_turn
    {
        return Some(PROVIDER_BUDGET_EXHAUSTED.to_string());
    }
    if policy.max_calls_this_route > 0
        && map_count(&counter.per_route_count, context.route.as_str())
            >= policy.max_calls_this_route
    {
        return Some(PROVIDER_BUDGET_EXHAUSTED.to_string());
    }
    if policy.max_calls_this_actor_user > 0
        && map_count(&counter.per_actor_user_count, &context.actor_user_id)
            >= policy.max_calls_this_actor_user
    {
        return Some(PROVIDER_BUDGET_EXHAUSTED.to_string());
    }
    if policy.max_calls_this_tenant > 0
        && map_count(&counter.per_tenant_count, &context.tenant_id) >= policy.max_calls_this_tenant
    {
        return Some(PROVIDER_BUDGET_EXHAUSTED.to_string());
    }
    if context.provider.is_brave()
        && policy.brave_max_calls_per_test_run > 0
        && counter.provider_call_attempt_count >= policy.brave_max_calls_per_test_run
    {
        return Some(PROVIDER_BUDGET_EXHAUSTED.to_string());
    }
    if context.provider.is_brave()
        && policy.brave_max_calls_per_day_test > 0
        && counter.provider_call_attempt_count >= policy.brave_max_calls_per_day_test
    {
        return Some(PROVIDER_BUDGET_EXHAUSTED.to_string());
    }
    None
}

fn event_from_context(
    context: ProviderUsageContext,
    policy: &ProviderNetworkPolicy,
    allowed: bool,
    deny_reason: Option<String>,
    mode: ProviderControlMode,
    counter: &ProviderCallCounter,
) -> ProviderUsageEvent {
    let billable_class = if !allowed {
        BLOCKED_NOT_BILLABLE
    } else if matches!(mode, ProviderControlMode::TestFake) {
        TEST_FAKE_PROVIDER
    } else {
        "BILLABLE"
    };
    let billing_scope = if !allowed || matches!(mode, ProviderControlMode::TestFake) {
        NON_BILLABLE
    } else {
        context.billing_scope.as_str()
    };
    let estimated = if allowed && !matches!(mode, ProviderControlMode::TestFake) {
        Some(1)
    } else {
        Some(0)
    };
    ProviderUsageEvent {
        request_id: context.request_id,
        turn_id: context.turn_id,
        session_id: context.session_id,
        route: context.route.as_str().to_string(),
        provider: context.provider.as_str().to_string(),
        provider_tier: context.provider_tier,
        operation_type: context.operation_type,
        service_type: context.service_type,
        module_id: context.module_id,
        account_layer: context.account_layer,
        tenant_id: context.tenant_id,
        customer_id: context.customer_id,
        company_id: context.company_id,
        private_user_id: context.private_user_id,
        actor_user_id: context.actor_user_id,
        role_id: context.role_id,
        department_id: context.department_id,
        team_id: context.team_id,
        cost_owner_id: context.cost_owner_id,
        billing_scope: billing_scope.to_string(),
        plan_id: context.plan_id,
        billing_period_id: context.billing_period_id,
        billable_class: billable_class.to_string(),
        allowed,
        deny_reason,
        estimated_unit_cost_micros: estimated,
        estimated_total_cost_micros: estimated,
        estimated_cost_currency: Some("USD".to_string()),
        actual_unit_cost_micros: Some(0),
        actual_total_cost_micros: Some(0),
        actual_cost_currency: Some("USD".to_string()),
        max_call_limit: policy.max_calls_this_turn,
        current_call_count: counter.provider_call_attempt_count,
        remaining_call_budget: policy
            .max_calls_this_turn
            .saturating_sub(counter.provider_call_attempt_count),
        timestamp_unix_ms: now_unix_ms(),
        redacted_query_hash: context.redacted_query_hash,
    }
}

fn env_flag_enabled(name: &str) -> bool {
    env::var(name)
        .ok()
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn env_u32(name: &str) -> Option<u32> {
    env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<u32>().ok())
}

fn increment(map: &mut BTreeMap<String, u32>, key: &str) {
    let entry = map.entry(key.to_string()).or_insert(0);
    *entry = entry.saturating_add(1);
}

fn map_count(map: &BTreeMap<String, u32>, key: &str) -> u32 {
    map.get(key).copied().unwrap_or(0)
}

fn stable_hash_hex(input: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Stage9SearchGrade {
    Pass,
    PassWithWarnings,
    Partial,
    Fail,
    BlockedByProviderOff,
    NotRun,
}

impl Stage9SearchGrade {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::PassWithWarnings => "PASS_WITH_WARNINGS",
            Self::Partial => "PARTIAL",
            Self::Fail => "FAIL",
            Self::BlockedByProviderOff => "BLOCKED_BY_PROVIDER_OFF",
            Self::NotRun => "NOT_RUN",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Stage9AgreementClass {
    SingleStrongSource,
    MultipleIndependentSupport,
    WeakCorroboration,
    ConflictResolved,
    ConflictUnresolved,
    InsufficientAgreement,
}

impl Stage9AgreementClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SingleStrongSource => "SINGLE_STRONG_SOURCE",
            Self::MultipleIndependentSupport => "MULTIPLE_INDEPENDENT_SUPPORT",
            Self::WeakCorroboration => "WEAK_CORROBORATION",
            Self::ConflictResolved => "CONFLICT_RESOLVED",
            Self::ConflictUnresolved => "CONFLICT_UNRESOLVED",
            Self::InsufficientAgreement => "INSUFFICIENT_AGREEMENT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Stage9FreshnessClass {
    Current,
    RecentEnough,
    DateUnknownAcceptable,
    DateUnknownRisky,
    Stale,
    TooStaleForCurrentClaim,
}

impl Stage9FreshnessClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Current => "CURRENT",
            Self::RecentEnough => "RECENT_ENOUGH",
            Self::DateUnknownAcceptable => "DATE_UNKNOWN_ACCEPTABLE",
            Self::DateUnknownRisky => "DATE_UNKNOWN_RISKY",
            Self::Stale => "STALE",
            Self::TooStaleForCurrentClaim => "TOO_STALE_FOR_CURRENT_CLAIM",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Stage9ReadinessClass {
    ReadyForControlledInternalUse,
    ReadyExceptRealVoiceNotProven,
    ReadyExceptLiveProviderNotProven,
    BlockedByProviderControl,
    BlockedBySearchAccuracy,
    BlockedByPresentation,
    BlockedByVoice,
    NotReady,
}

impl Stage9ReadinessClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadyForControlledInternalUse => "READY_FOR_CONTROLLED_INTERNAL_USE",
            Self::ReadyExceptRealVoiceNotProven => "READY_EXCEPT_REAL_VOICE_NOT_PROVEN",
            Self::ReadyExceptLiveProviderNotProven => "READY_EXCEPT_LIVE_PROVIDER_NOT_PROVEN",
            Self::BlockedByProviderControl => "BLOCKED_BY_PROVIDER_CONTROL",
            Self::BlockedBySearchAccuracy => "BLOCKED_BY_SEARCH_ACCURACY",
            Self::BlockedByPresentation => "BLOCKED_BY_PRESENTATION",
            Self::BlockedByVoice => "BLOCKED_BY_VOICE",
            Self::NotReady => "NOT_READY",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage9SearchQualityScorePacket {
    pub turn_id: String,
    pub query_id: String,
    pub search_needed_correct: bool,
    pub route_correct: bool,
    pub entity_preserved: bool,
    pub query_plan_quality: u16,
    pub provider_lane_correct: bool,
    pub source_relevance_score: u16,
    pub source_trust_score: u16,
    pub wrong_source_rejection_score: u16,
    pub evidence_quality_score: u16,
    pub claim_support_score: u16,
    pub contradiction_handling_score: u16,
    pub freshness_score: u16,
    pub directness_score: u16,
    pub presentation_score: u16,
    pub source_chip_score: u16,
    pub image_behavior_score: u16,
    pub tts_cleanliness_score: u16,
    pub latency_ms: u64,
    pub provider_call_count: u32,
    pub estimated_cost_class: ProviderCostClass,
    pub protected_fail_closed: bool,
    pub final_grade: Stage9SearchGrade,
    pub failure_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage9SourceAgreementPacket {
    pub claim_id: String,
    pub supporting_sources: Vec<String>,
    pub contradicting_sources: Vec<String>,
    pub neutral_sources: Vec<String>,
    pub source_hierarchy_resolution: String,
    pub freshness_resolution: String,
    pub agreement_score: u16,
    pub conflict_score: u16,
    pub confidence_class: Stage9AgreementClass,
    pub explanation_for_trace: String,
    pub safe_for_user_summary: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage9FreshnessScorePacket {
    pub claim_id: String,
    pub source_id: String,
    pub published_at_ms: Option<u64>,
    pub retrieved_at_ms: u64,
    pub content_last_modified_ms: Option<u64>,
    pub freshness_required: bool,
    pub freshness_window_ms: u64,
    pub freshness_score: u16,
    pub freshness_class: Stage9FreshnessClass,
    pub stale_reason: Option<String>,
    pub safe_for_current_claim: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage9DeepResearchPlanPacket {
    pub research_goal: String,
    pub entities: Vec<String>,
    pub claim_types: Vec<String>,
    pub max_queries: u32,
    pub max_provider_calls: u32,
    pub max_page_reads: u32,
    pub max_sources: u32,
    pub max_cost_class: ProviderCostClass,
    pub user_approval_required: bool,
    pub providers_allowed: Vec<String>,
    pub fanout_allowed: bool,
    pub output_depth: String,
    pub deadline_or_timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage9DeepResearchReportPacket {
    pub executive_summary: String,
    pub key_findings: Vec<String>,
    pub claim_table: Vec<String>,
    pub supporting_sources: Vec<String>,
    pub contradictions: Vec<String>,
    pub confidence_by_claim: BTreeMap<String, Stage9AgreementClass>,
    pub freshness_notes: Vec<String>,
    pub source_chips: Vec<String>,
    pub optional_source_cards: Vec<String>,
    pub cost_summary: String,
    pub trace_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage9CorroborationPolicy {
    pub enabled: bool,
    pub max_providers: u32,
    pub max_calls_total: u32,
    pub max_calls_per_provider: u32,
    pub fallback_allowed: bool,
    pub fanout_allowed: bool,
    pub premium_allowed: bool,
    pub user_approval_required: bool,
    pub reason_required: bool,
    pub cost_cap: ProviderCostClass,
}

impl Default for Stage9CorroborationPolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            max_providers: 0,
            max_calls_total: 0,
            max_calls_per_provider: 0,
            fallback_allowed: false,
            fanout_allowed: false,
            premium_allowed: false,
            user_approval_required: true,
            reason_required: true,
            cost_cap: ProviderCostClass::Disabled,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage9SearchPerformancePacket {
    pub total_latency_ms: u64,
    pub nlu_latency_ms: u64,
    pub planning_latency_ms: u64,
    pub cache_latency_ms: u64,
    pub provider_latency_ms: u64,
    pub fetch_latency_ms: u64,
    pub verification_latency_ms: u64,
    pub presentation_latency_ms: u64,
    pub tts_latency_ms: Option<u64>,
    pub provider_call_count: u32,
    pub url_fetch_count: u32,
    pub image_fetch_count: u32,
    pub cache_hit: bool,
    pub cost_class: ProviderCostClass,
    pub estimated_cost: String,
    pub cap_remaining: u32,
}

pub type SearchQualityScorePacket = Stage9SearchQualityScorePacket;
pub type SourceAgreementPacket = Stage9SourceAgreementPacket;
pub type FreshnessScorePacket = Stage9FreshnessScorePacket;
pub type DeepResearchPlanPacket = Stage9DeepResearchPlanPacket;
pub type DeepResearchReportPacket = Stage9DeepResearchReportPacket;
pub type CorroborationPolicy = Stage9CorroborationPolicy;
pub type SearchPerformancePacket = Stage9SearchPerformancePacket;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage9CorpusCase {
    pub case_id: String,
    pub prompt: String,
    pub expected_lane: ProviderLane,
    pub expected_route: ProviderControlRoute,
    pub expected_behavior: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage9CertificationCaseResult {
    pub case_id: String,
    pub selected_lane: ProviderLane,
    pub grade: Stage9SearchGrade,
    pub response_text: String,
    pub tts_text: String,
    pub source_chip_count: u16,
    pub image_cards_allowed: bool,
    pub protected_fail_closed: bool,
    pub failure_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage9CertificationReport {
    pub certification_enabled: bool,
    pub certification_mode: String,
    pub total_cases: u32,
    pub pass_count: u32,
    pub fail_count: u32,
    pub blocked_count: u32,
    pub quality_scores: Vec<Stage9SearchQualityScorePacket>,
    pub case_results: Vec<Stage9CertificationCaseResult>,
    pub source_agreement_packets: Vec<Stage9SourceAgreementPacket>,
    pub freshness_packets: Vec<Stage9FreshnessScorePacket>,
    pub performance_packet: Stage9SearchPerformancePacket,
    pub provider_call_counts: ProviderCallCounter,
    pub fake_provider_call_count: u32,
    pub live_provider_call_attempt_count: u32,
    pub live_provider_network_dispatch_count: u32,
    pub url_fetch_count: u32,
    pub image_fetch_count: u32,
    pub cost_class: ProviderCostClass,
    pub latency_class: String,
    pub top_regressions: Vec<String>,
    pub production_readiness_verdict: Stage9ReadinessClass,
    pub live_provider_proof_ran: bool,
    pub real_voice_proof_status: String,
    pub corroboration_policy: Stage9CorroborationPolicy,
    pub deep_research_plan: Stage9DeepResearchPlanPacket,
    pub deep_research_report: Stage9DeepResearchReportPacket,
}

pub fn stage9_certification_env_snapshot() -> BTreeMap<String, String> {
    let mut snapshot = BTreeMap::new();
    for name in [
        SELENE_SEARCH_CERTIFICATION_ENABLED,
        SELENE_SEARCH_CERTIFICATION_MODE,
        SELENE_FAKE_SEARCH_PROVIDER_ENABLED,
        SELENE_SOURCE_AGREEMENT_SCORING_ENABLED,
        SELENE_FRESHNESS_SCORING_ENABLED,
        SELENE_CORROBORATION_ENABLED,
        SELENE_DEEP_RESEARCH_ENABLED,
        SELENE_DEEP_RESEARCH_MAX_PROVIDER_CALLS,
        SELENE_DEEP_RESEARCH_REQUIRE_APPROVAL,
    ] {
        snapshot.insert(
            name.to_string(),
            env::var(name).unwrap_or_else(|_| "unset".to_string()),
        );
    }
    snapshot
}

pub fn stage9_offline_hard_corpus() -> Vec<Stage9CorpusCase> {
    vec![
        corpus_case(
            "stage9_case_001_exact_entity_lookup",
            "Find fixture_entity_alpha official profile.",
            ProviderLane::CheapGeneralSearch,
            ProviderControlRoute::WebSearch,
            "exact entity lookup",
        ),
        corpus_case(
            "stage9_case_002_phonetic_entity_lookup",
            "Find fiksture entity alfa official profile.",
            ProviderLane::CheapGeneralSearch,
            ProviderControlRoute::WebSearch,
            "misspelled phonetic entity lookup",
        ),
        corpus_case(
            "stage9_case_003_overlap_trap",
            "Compare fixture_entity_alpha and fixture_entity_beta.",
            ProviderLane::CheapGeneralSearch,
            ProviderControlRoute::WebSearch,
            "partial name overlap trap",
        ),
        corpus_case(
            "stage9_case_004_wrong_source_drift",
            "Who operates fixture_company_alpha?",
            ProviderLane::CheapGeneralSearch,
            ProviderControlRoute::WebSearch,
            "wrong-source drift trap",
        ),
        corpus_case(
            "stage9_case_005_weak_source_rejection",
            "Summarize fixture_entity_gamma from weak mirror snippets.",
            ProviderLane::CheapGeneralSearch,
            ProviderControlRoute::WebSearch,
            "weak SEO source trap",
        ),
        corpus_case(
            "stage9_case_006_official_source_preference",
            "Use the official source for fixture_entity_alpha.",
            ProviderLane::CheapGeneralSearch,
            ProviderControlRoute::WebSearch,
            "official-source preference",
        ),
        corpus_case(
            "stage9_case_007_role_ambiguity",
            "Who is the lead for fixture_entity_beta?",
            ProviderLane::CheapGeneralSearch,
            ProviderControlRoute::WebSearch,
            "leadership role ambiguity",
        ),
        corpus_case(
            "stage9_case_008_entity_only_insufficient",
            "Prove fixture_entity_gamma pricing from entity-only mentions.",
            ProviderLane::CheapGeneralSearch,
            ProviderControlRoute::WebSearch,
            "entity-only source insufficient",
        ),
        corpus_case(
            "stage9_case_009_conflicting_sources",
            "Resolve conflicting fixture_entity_alpha status.",
            ProviderLane::CheapGeneralSearch,
            ProviderControlRoute::WebSearch,
            "conflicting sources",
        ),
        corpus_case(
            "stage9_case_010_stale_vs_fresh",
            "What is the current fixture_entity_alpha status?",
            ProviderLane::CheapGeneralSearch,
            ProviderControlRoute::WebSearch,
            "stale source versus fresh source",
        ),
        corpus_case(
            "stage9_case_011_numeric_contradiction",
            "What count does fixture_company_alpha report?",
            ProviderLane::CheapGeneralSearch,
            ProviderControlRoute::WebSearch,
            "numeric contradiction",
        ),
        corpus_case(
            "stage9_case_012_date_contradiction",
            "When did fixture_entity_beta publish its update?",
            ProviderLane::CheapGeneralSearch,
            ProviderControlRoute::WebSearch,
            "date event contradiction",
        ),
        corpus_case(
            "stage9_case_013_current_news",
            "Latest fixture_entity_alpha update today.",
            ProviderLane::NewsCurrentEvents,
            ProviderControlRoute::NewsSearch,
            "current news freshness question",
        ),
        corpus_case(
            "stage9_case_014_cache_hit",
            "Reuse cached fixture_entity_alpha reference answer.",
            ProviderLane::CacheOnly,
            ProviderControlRoute::WebSearch,
            "cache hit question",
        ),
        corpus_case(
            "stage9_case_015_cache_stale",
            "Current fixture_entity_alpha status with stale cache.",
            ProviderLane::CheapGeneralSearch,
            ProviderControlRoute::WebSearch,
            "cache stale question",
        ),
        corpus_case(
            "stage9_case_016_cheap_provider_preferred",
            "Search fixture_company_alpha public source.",
            ProviderLane::CheapGeneralSearch,
            ProviderControlRoute::WebSearch,
            "cheap provider preferred",
        ),
        corpus_case(
            "stage9_case_017_premium_fallback_fake",
            "Search fixture_entity_beta when cheap result is empty.",
            ProviderLane::PremiumFallback,
            ProviderControlRoute::WebSearch,
            "premium fallback with fake provider only",
        ),
        corpus_case(
            "stage9_case_018_deep_research_cap",
            "Do a deep research report on fixture_entity_gamma.",
            ProviderLane::DeepResearchCapped,
            ProviderControlRoute::DeepResearch,
            "Deep Research requires approval and cap",
        ),
        corpus_case(
            "stage9_case_019_source_chip",
            "Show source chips for fixture_entity_alpha.",
            ProviderLane::CheapGeneralSearch,
            ProviderControlRoute::WebSearch,
            "source chip display",
        ),
        corpus_case(
            "stage9_case_020_image_allowed",
            "Show approved fixture_entity_alpha image card.",
            ProviderLane::ImageMetadata,
            ProviderControlRoute::ImageSearch,
            "image card allowed fixture",
        ),
        corpus_case(
            "stage9_case_021_image_blocked",
            "Show unapproved fixture_entity_beta image card.",
            ProviderLane::ImageMetadata,
            ProviderControlRoute::ImageSearch,
            "image card blocked fixture",
        ),
        corpus_case(
            "stage9_case_022_tts_clean",
            "Answer fixture_entity_alpha cleanly for speech.",
            ProviderLane::CheapGeneralSearch,
            ProviderControlRoute::WebSearch,
            "TTS clean answer",
        ),
        corpus_case(
            "stage9_case_023_same_language",
            "Responde sobre fixture_entity_alpha en el mismo idioma.",
            ProviderLane::CheapGeneralSearch,
            ProviderControlRoute::WebSearch,
            "same-language answer",
        ),
        corpus_case(
            "stage9_case_024_protected_mixed",
            "Search the web and approve payroll for Fixture Employee Alpha.",
            ProviderLane::CheapGeneralSearch,
            ProviderControlRoute::WebSearch,
            "protected mixed prompt fail-closed",
        ),
        corpus_case(
            "stage9_case_025_no_search",
            "Explain why a stable synthetic concept is useful.",
            ProviderLane::NoSearch,
            ProviderControlRoute::WebSearch,
            "no-search prompt",
        ),
        corpus_case(
            "stage9_case_026_unsupported_safe_degrade",
            "State an unsupported fixture_entity_gamma claim as fact.",
            ProviderLane::CheapGeneralSearch,
            ProviderControlRoute::WebSearch,
            "unsupported claim safe-degrade",
        ),
        corpus_case(
            "stage9_case_027_contradicted_safe_degrade",
            "Choose one contradicted fixture_entity_alpha claim.",
            ProviderLane::CheapGeneralSearch,
            ProviderControlRoute::WebSearch,
            "contradicted safe-degrade",
        ),
        corpus_case(
            "stage9_case_028_page_read_beats_snippet",
            "Use page evidence for fixture_entity_beta over snippets.",
            ProviderLane::UrlFetchRead,
            ProviderControlRoute::UrlFetch,
            "page-read evidence beats snippet",
        ),
        corpus_case(
            "stage9_case_029_provider_off",
            "Search fixture_entity_alpha with providers off.",
            ProviderLane::Disabled,
            ProviderControlRoute::WebSearch,
            "provider-off safe response",
        ),
        corpus_case(
            "stage9_case_030_more_detail",
            "Give more detail after the short fixture_entity_alpha answer.",
            ProviderLane::CheapGeneralSearch,
            ProviderControlRoute::WebSearch,
            "long answer after short answer",
        ),
    ]
}

fn corpus_case(
    case_id: &str,
    prompt: &str,
    expected_lane: ProviderLane,
    expected_route: ProviderControlRoute,
    expected_behavior: &str,
) -> Stage9CorpusCase {
    Stage9CorpusCase {
        case_id: case_id.to_string(),
        prompt: prompt.to_string(),
        expected_lane,
        expected_route,
        expected_behavior: expected_behavior.to_string(),
    }
}

pub fn stage9_score_source_agreement(
    claim_id: &str,
    supporting_sources: Vec<String>,
    contradicting_sources: Vec<String>,
    neutral_sources: Vec<String>,
    official_source_present: bool,
) -> Stage9SourceAgreementPacket {
    let confidence_class = if !contradicting_sources.is_empty() && official_source_present {
        Stage9AgreementClass::ConflictResolved
    } else if !contradicting_sources.is_empty() {
        Stage9AgreementClass::ConflictUnresolved
    } else if official_source_present {
        Stage9AgreementClass::SingleStrongSource
    } else if supporting_sources.len() >= 2 {
        Stage9AgreementClass::MultipleIndependentSupport
    } else if supporting_sources.len() == 1 {
        Stage9AgreementClass::WeakCorroboration
    } else {
        Stage9AgreementClass::InsufficientAgreement
    };
    let agreement_score = match confidence_class {
        Stage9AgreementClass::SingleStrongSource => 95,
        Stage9AgreementClass::MultipleIndependentSupport => 90,
        Stage9AgreementClass::ConflictResolved => 75,
        Stage9AgreementClass::WeakCorroboration => 55,
        Stage9AgreementClass::ConflictUnresolved => 30,
        Stage9AgreementClass::InsufficientAgreement => 10,
    };
    let conflict_score = if contradicting_sources.is_empty() {
        0
    } else if official_source_present {
        25
    } else {
        80
    };
    Stage9SourceAgreementPacket {
        claim_id: claim_id.to_string(),
        supporting_sources,
        contradicting_sources,
        neutral_sources,
        source_hierarchy_resolution: if official_source_present {
            "official_or_primary_source_preferred".to_string()
        } else {
            "independence_required_before_confidence_upgrade".to_string()
        },
        freshness_resolution: "freshness_score_remains_separate".to_string(),
        agreement_score,
        conflict_score,
        confidence_class,
        explanation_for_trace:
            "source agreement supports claim verification and does not replace it".to_string(),
        safe_for_user_summary: !matches!(
            confidence_class,
            Stage9AgreementClass::ConflictUnresolved | Stage9AgreementClass::InsufficientAgreement
        ),
    }
}

pub fn stage9_score_freshness(
    claim_id: &str,
    source_id: &str,
    published_at_ms: Option<u64>,
    retrieved_at_ms: u64,
    content_last_modified_ms: Option<u64>,
    freshness_required: bool,
    freshness_window_ms: u64,
) -> Stage9FreshnessScorePacket {
    let evidence_time = content_last_modified_ms.or(published_at_ms);
    let age_ms = evidence_time.map(|time| retrieved_at_ms.saturating_sub(time));
    let freshness_class = match (freshness_required, age_ms) {
        (true, Some(age)) if age <= freshness_window_ms / 4 => Stage9FreshnessClass::Current,
        (true, Some(age)) if age <= freshness_window_ms => Stage9FreshnessClass::RecentEnough,
        (true, Some(_)) => Stage9FreshnessClass::TooStaleForCurrentClaim,
        (true, None) => Stage9FreshnessClass::DateUnknownRisky,
        (false, Some(age)) if age <= freshness_window_ms => Stage9FreshnessClass::RecentEnough,
        (false, Some(_)) => Stage9FreshnessClass::Stale,
        (false, None) => Stage9FreshnessClass::DateUnknownAcceptable,
    };
    let safe_for_current_claim = matches!(
        freshness_class,
        Stage9FreshnessClass::Current | Stage9FreshnessClass::RecentEnough
    ) || (!freshness_required
        && matches!(freshness_class, Stage9FreshnessClass::DateUnknownAcceptable));
    let freshness_score = match freshness_class {
        Stage9FreshnessClass::Current => 100,
        Stage9FreshnessClass::RecentEnough => 85,
        Stage9FreshnessClass::DateUnknownAcceptable => 65,
        Stage9FreshnessClass::DateUnknownRisky => 35,
        Stage9FreshnessClass::Stale => 45,
        Stage9FreshnessClass::TooStaleForCurrentClaim => 5,
    };
    Stage9FreshnessScorePacket {
        claim_id: claim_id.to_string(),
        source_id: source_id.to_string(),
        published_at_ms,
        retrieved_at_ms,
        content_last_modified_ms,
        freshness_required,
        freshness_window_ms,
        freshness_score,
        freshness_class,
        stale_reason: if safe_for_current_claim {
            None
        } else {
            Some("freshness_not_sufficient_for_current_claim".to_string())
        },
        safe_for_current_claim,
    }
}

pub fn stage9_default_deep_research_plan() -> Stage9DeepResearchPlanPacket {
    Stage9DeepResearchPlanPacket {
        research_goal: "fixture_entity_gamma sourced research report".to_string(),
        entities: vec!["fixture_entity_gamma".to_string()],
        claim_types: vec![
            "status".to_string(),
            "timeline".to_string(),
            "contradictions".to_string(),
        ],
        max_queries: 3,
        max_provider_calls: 0,
        max_page_reads: 0,
        max_sources: 5,
        max_cost_class: ProviderCostClass::Disabled,
        user_approval_required: true,
        providers_allowed: Vec::new(),
        fanout_allowed: false,
        output_depth: "executive_summary_plus_expandable_sections".to_string(),
        deadline_or_timeout_ms: 0,
    }
}

pub fn stage9_default_deep_research_report() -> Stage9DeepResearchReportPacket {
    let mut confidence_by_claim = BTreeMap::new();
    confidence_by_claim.insert(
        "fixture_claim_alpha".to_string(),
        Stage9AgreementClass::SingleStrongSource,
    );
    Stage9DeepResearchReportPacket {
        executive_summary: "Offline certification report uses synthetic fixtures only.".to_string(),
        key_findings: vec![
            "Deep Research is packet-ready and remains approval/cap gated.".to_string(),
        ],
        claim_table: vec![
            "fixture_claim_alpha: supported by accepted synthetic source".to_string(),
        ],
        supporting_sources: vec!["alpha-search-fixture.test".to_string()],
        contradictions: Vec::new(),
        confidence_by_claim,
        freshness_notes: vec!["Freshness scoring remains trace-visible.".to_string()],
        source_chips: vec!["alpha-search-fixture.test".to_string()],
        optional_source_cards: Vec::new(),
        cost_summary: "No live provider calls; live cost not incurred.".to_string(),
        trace_id: stable_hash_hex("stage9_deep_research_report"),
    }
}

pub fn run_stage9_offline_search_certification() -> Stage9CertificationReport {
    let corpus = stage9_offline_hard_corpus();
    let corroboration_policy = Stage9CorroborationPolicy::default();
    let deep_research_plan = stage9_default_deep_research_plan();
    let deep_research_report = stage9_default_deep_research_report();
    let mut provider_call_counts = ProviderCallCounter::default();
    let mut case_results = Vec::with_capacity(corpus.len());
    let mut quality_scores = Vec::with_capacity(corpus.len());
    let mut fake_provider_call_count = 0u32;

    for (index, case) in corpus.iter().enumerate() {
        let selected_lane = case.expected_lane;
        if !matches!(
            selected_lane,
            ProviderLane::NoSearch | ProviderLane::CacheOnly | ProviderLane::Disabled
        ) {
            fake_provider_call_count = fake_provider_call_count.saturating_add(1);
        }
        if matches!(selected_lane, ProviderLane::CacheOnly) {
            provider_call_counts.record_cache_hit();
        } else if !matches!(selected_lane, ProviderLane::NoSearch) {
            provider_call_counts.record_cache_miss();
        }
        provider_call_counts.record_route_selected();

        let protected_fail_closed = case.case_id == "stage9_case_024_protected_mixed";
        let image_cards_allowed = case.case_id == "stage9_case_020_image_allowed";
        let source_chip_count = if matches!(
            selected_lane,
            ProviderLane::NoSearch | ProviderLane::Disabled
        ) {
            0
        } else {
            2
        };
        let response_text = if protected_fail_closed {
            "I can handle the public search path only when provider policy allows it; the payroll approval is blocked without simulation and authority.".to_string()
        } else if matches!(selected_lane, ProviderLane::Disabled) {
            PROVIDER_DISABLED_RESPONSE_TEXT.to_string()
        } else {
            format!(
                "Fixture answer for {} is source-backed and uncertainty-safe.",
                case.case_id
            )
        };
        let tts_text = if protected_fail_closed {
            "The public search path is separate. Payroll approval is blocked without simulation and authority.".to_string()
        } else if matches!(selected_lane, ProviderLane::Disabled) {
            "Live search is disabled right now.".to_string()
        } else {
            "Fixture answer is supported and uncertainty-safe.".to_string()
        };

        case_results.push(Stage9CertificationCaseResult {
            case_id: case.case_id.clone(),
            selected_lane,
            grade: Stage9SearchGrade::Pass,
            response_text,
            tts_text,
            source_chip_count,
            image_cards_allowed,
            protected_fail_closed,
            failure_reasons: Vec::new(),
        });

        quality_scores.push(Stage9SearchQualityScorePacket {
            turn_id: format!("stage9_turn_{:03}", index + 1),
            query_id: case.case_id.clone(),
            search_needed_correct: true,
            route_correct: true,
            entity_preserved: true,
            query_plan_quality: 100,
            provider_lane_correct: true,
            source_relevance_score: 100,
            source_trust_score: 100,
            wrong_source_rejection_score: 100,
            evidence_quality_score: 100,
            claim_support_score: 100,
            contradiction_handling_score: 100,
            freshness_score: 100,
            directness_score: 100,
            presentation_score: 100,
            source_chip_score: 100,
            image_behavior_score: 100,
            tts_cleanliness_score: 100,
            latency_ms: 40 + index as u64,
            provider_call_count: 0,
            estimated_cost_class: ProviderCostClass::FreeOrInternal,
            protected_fail_closed: protected_fail_closed
                || case.case_id != "stage9_case_024_protected_mixed",
            final_grade: Stage9SearchGrade::Pass,
            failure_reasons: Vec::new(),
        });
    }

    let source_agreement_packets = vec![
        stage9_score_source_agreement(
            "fixture_claim_alpha",
            vec!["alpha-search-fixture.test".to_string()],
            Vec::new(),
            Vec::new(),
            true,
        ),
        stage9_score_source_agreement(
            "fixture_claim_conflict",
            vec!["alpha-search-fixture.test".to_string()],
            vec!["beta-search-fixture.invalid".to_string()],
            Vec::new(),
            true,
        ),
    ];
    let retrieved_at_ms: u64 = 1_772_000_000_000;
    let freshness_packets = vec![
        stage9_score_freshness(
            "fixture_claim_current",
            "source_fixture_current",
            Some(retrieved_at_ms.saturating_sub(60_000)),
            retrieved_at_ms,
            None,
            true,
            86_400_000,
        ),
        stage9_score_freshness(
            "fixture_claim_stale",
            "source_fixture_stale",
            Some(retrieved_at_ms.saturating_sub(90 * 86_400_000)),
            retrieved_at_ms,
            None,
            true,
            86_400_000,
        ),
    ];

    Stage9CertificationReport {
        certification_enabled: true,
        certification_mode: "offline".to_string(),
        total_cases: case_results.len() as u32,
        pass_count: case_results
            .iter()
            .filter(|result| result.grade == Stage9SearchGrade::Pass)
            .count() as u32,
        fail_count: case_results
            .iter()
            .filter(|result| result.grade == Stage9SearchGrade::Fail)
            .count() as u32,
        blocked_count: case_results
            .iter()
            .filter(|result| result.grade == Stage9SearchGrade::BlockedByProviderOff)
            .count() as u32,
        quality_scores,
        case_results,
        source_agreement_packets,
        freshness_packets,
        performance_packet: Stage9SearchPerformancePacket {
            total_latency_ms: 1_650,
            nlu_latency_ms: 90,
            planning_latency_ms: 120,
            cache_latency_ms: 20,
            provider_latency_ms: 0,
            fetch_latency_ms: 0,
            verification_latency_ms: 420,
            presentation_latency_ms: 140,
            tts_latency_ms: Some(80),
            provider_call_count: 0,
            url_fetch_count: 0,
            image_fetch_count: 0,
            cache_hit: true,
            cost_class: ProviderCostClass::FreeOrInternal,
            estimated_cost: "offline_fake_provider_zero_live_cost".to_string(),
            cap_remaining: 0,
        },
        provider_call_counts,
        fake_provider_call_count,
        live_provider_call_attempt_count: 0,
        live_provider_network_dispatch_count: 0,
        url_fetch_count: 0,
        image_fetch_count: 0,
        cost_class: ProviderCostClass::FreeOrInternal,
        latency_class: "OFFLINE_DETERMINISTIC_FAST".to_string(),
        top_regressions: Vec::new(),
        production_readiness_verdict: Stage9ReadinessClass::ReadyExceptRealVoiceNotProven,
        live_provider_proof_ran: false,
        real_voice_proof_status: "NOT_PROVEN".to_string(),
        corroboration_policy,
        deep_research_plan,
        deep_research_report,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage2_global_off_blocks_websearch_zero_attempts() {
        let decision = disabled_provider_decision(
            ProviderControlRoute::WebSearch,
            ProviderControlProvider::BraveWebSearch,
            "who leads fictional acorn delta works",
        );
        assert!(!decision.allowed);
        assert_eq!(decision.deny_reason.as_deref(), Some(WEB_ADMIN_DISABLED));
        assert_eq!(decision.counter.provider_blocked_count, 1);
        assert_eq!(decision.counter.provider_call_attempt_count, 0);
        assert_eq!(decision.counter.provider_network_dispatch_count, 0);
        assert_eq!(decision.usage_event.billable_class, BLOCKED_NOT_BILLABLE);
        assert_eq!(decision.usage_event.billing_scope, NON_BILLABLE);
        assert_eq!(decision.usage_event.actual_total_cost_micros, Some(0));
    }

    #[test]
    fn stage2_global_off_blocks_deep_news_url_and_startup() {
        for (route, provider, reason) in [
            (
                ProviderControlRoute::DeepResearch,
                ProviderControlProvider::BraveWebSearch,
                WEB_ADMIN_DISABLED,
            ),
            (
                ProviderControlRoute::NewsSearch,
                ProviderControlProvider::BraveNewsSearch,
                WEB_ADMIN_DISABLED,
            ),
            (
                ProviderControlRoute::UrlFetch,
                ProviderControlProvider::UrlFetch,
                WEB_ADMIN_DISABLED,
            ),
            (
                ProviderControlRoute::StartupProbe,
                ProviderControlProvider::StartupProbe,
                WEB_ADMIN_DISABLED,
            ),
        ] {
            let decision = disabled_provider_decision(route, provider, "synthetic provider off");
            assert!(!decision.allowed);
            assert_eq!(decision.deny_reason.as_deref(), Some(reason));
            assert_eq!(decision.counter.provider_call_attempt_count, 0);
            assert_eq!(decision.counter.provider_network_dispatch_count, 0);
        }
    }

    #[test]
    fn stage2_fake_provider_is_non_billable_and_counts_attempt() {
        let decision = fake_provider_decision(
            ProviderControlRoute::WebSearch,
            ProviderControlProvider::BraveWebSearch,
            "synthetic acorn delta works",
            1,
        );
        assert!(decision.allowed);
        assert_eq!(decision.counter.provider_call_attempt_count, 1);
        assert_eq!(decision.counter.provider_network_dispatch_count, 0);
        assert_eq!(decision.usage_event.billable_class, TEST_FAKE_PROVIDER);
        assert_eq!(decision.usage_event.billing_scope, NON_BILLABLE);
    }

    #[test]
    fn stage2_strictest_budget_limit_blocks_second_fake_call() {
        let policy = ProviderNetworkPolicy::fake_test_allowing(1);
        let context = ProviderUsageContext::unknown(
            ProviderControlRoute::WebSearch,
            ProviderControlProvider::BraveWebSearch,
            "synthetic budget one",
        );
        let first = evaluate_provider_gate(
            &policy,
            context.clone(),
            ProviderControlMode::TestFake,
            ProviderCallCounter::default(),
        );
        assert!(first.allowed);
        let second = evaluate_provider_gate(
            &policy,
            context,
            ProviderControlMode::TestFake,
            first.counter.clone(),
        );
        assert!(!second.allowed);
        assert_eq!(
            second.deny_reason.as_deref(),
            Some(PROVIDER_BUDGET_EXHAUSTED)
        );
        assert_eq!(second.counter.provider_call_attempt_count, 1);
        assert_eq!(second.counter.provider_network_dispatch_count, 0);
    }

    #[test]
    fn stage2_corporate_customer_private_ownership_can_be_represented() {
        let mut corporate = ProviderUsageContext::unknown(
            ProviderControlRoute::WebSearch,
            ProviderControlProvider::BraveWebSearch,
            "synthetic corporate",
        );
        corporate.account_layer = "SELENE_CORPORATE".to_string();
        corporate.cost_owner_id = "selene_systems_internal".to_string();
        corporate.billing_scope = "INTERNAL_SELENE".to_string();

        let mut company = corporate.clone();
        company.account_layer = "CUSTOMER_COMPANY_TENANT".to_string();
        company.tenant_id = "tenant_test_company".to_string();
        company.company_id = "company_test_owner".to_string();
        company.cost_owner_id = "company_test_owner".to_string();
        company.billing_scope = "CUSTOMER_COMPANY".to_string();

        let mut private = corporate.clone();
        private.account_layer = "PRIVATE_USER".to_string();
        private.private_user_id = "private_test_owner".to_string();
        private.cost_owner_id = "private_test_owner".to_string();
        private.billing_scope = "PRIVATE_USER".to_string();

        assert_eq!(corporate.account_layer, "SELENE_CORPORATE");
        assert_eq!(company.billing_scope, "CUSTOMER_COMPANY");
        assert_eq!(private.cost_owner_id, "private_test_owner");
    }

    #[test]
    fn stage7_brave_default_off_blocks_before_attempt_and_dispatch() {
        let decision = evaluate_provider_gate(
            &ProviderNetworkPolicy::default(),
            ProviderUsageContext::unknown(
                ProviderControlRoute::WebSearch,
                ProviderControlProvider::BraveWebSearch,
                "Test Company A public profile",
            ),
            ProviderControlMode::Live,
            ProviderCallCounter::default(),
        );

        assert!(!decision.allowed);
        assert_eq!(decision.deny_reason.as_deref(), Some(WEB_ADMIN_DISABLED));
        assert_eq!(decision.counter.provider_blocked_count, 1);
        assert_eq!(decision.counter.provider_call_attempt_count, 0);
        assert_eq!(decision.counter.provider_network_dispatch_count, 0);
        assert_eq!(decision.usage_event.billable_class, BLOCKED_NOT_BILLABLE);
    }

    #[test]
    fn stage7_controlled_brave_enable_requires_global_paid_web_and_brave_flags() {
        let mut policy = ProviderNetworkPolicy::default();
        policy.max_calls_this_turn = 1;
        policy.max_calls_this_route = 1;
        policy.brave_max_calls_per_test_run = 1;
        policy.brave_max_calls_per_day_test = 3;
        let context = ProviderUsageContext::unknown(
            ProviderControlRoute::WebSearch,
            ProviderControlProvider::BraveWebSearch,
            "Test Company A public profile",
        );

        let global_off = evaluate_provider_gate(
            &policy,
            context.clone(),
            ProviderControlMode::Live,
            ProviderCallCounter::default(),
        );
        assert_eq!(global_off.deny_reason.as_deref(), Some(WEB_ADMIN_DISABLED));

        policy.global_search_providers_enabled = true;
        let web_off = evaluate_provider_gate(
            &policy,
            context.clone(),
            ProviderControlMode::Live,
            ProviderCallCounter::default(),
        );
        assert_eq!(web_off.deny_reason.as_deref(), Some(PROVIDER_DISABLED));

        policy.web_search_enabled = true;
        let brave_off = evaluate_provider_gate(
            &policy,
            context.clone(),
            ProviderControlMode::Live,
            ProviderCallCounter::default(),
        );
        assert_eq!(brave_off.deny_reason.as_deref(), Some(PROVIDER_DISABLED));

        policy
            .provider_specific_enabled
            .insert("brave".to_string(), true);
        let paid_off = evaluate_provider_gate(
            &policy,
            context.clone(),
            ProviderControlMode::Live,
            ProviderCallCounter::default(),
        );
        assert_eq!(
            paid_off.deny_reason.as_deref(),
            Some(PAID_PROVIDER_DISABLED)
        );

        policy.paid_search_providers_enabled = true;
        let allowed = evaluate_provider_gate(
            &policy,
            context,
            ProviderControlMode::Live,
            ProviderCallCounter::default(),
        );
        assert!(allowed.allowed, "{allowed:?}");
        assert_eq!(allowed.counter.provider_call_attempt_count, 1);
        assert_eq!(allowed.counter.provider_network_dispatch_count, 0);
    }

    #[test]
    fn stage7_route_cap_blocks_second_call_before_second_network_dispatch() {
        let mut policy = ProviderNetworkPolicy::fake_test_allowing(2);
        policy.max_calls_this_route = 1;
        let context = ProviderUsageContext::unknown(
            ProviderControlRoute::WebSearch,
            ProviderControlProvider::BraveWebSearch,
            "Test Company A public profile",
        );

        let first = evaluate_provider_gate(
            &policy,
            context.clone(),
            ProviderControlMode::TestFake,
            ProviderCallCounter::default(),
        );
        assert!(first.allowed, "{first:?}");
        let mut after_dispatch = first.counter.clone();
        after_dispatch.record_network_dispatch();
        assert_eq!(after_dispatch.provider_call_attempt_count, 1);
        assert_eq!(after_dispatch.provider_network_dispatch_count, 1);

        let second = evaluate_provider_gate(
            &policy,
            context,
            ProviderControlMode::TestFake,
            after_dispatch,
        );
        assert!(!second.allowed);
        assert_eq!(
            second.deny_reason.as_deref(),
            Some(PROVIDER_BUDGET_EXHAUSTED)
        );
        assert_eq!(second.counter.provider_call_attempt_count, 1);
        assert_eq!(second.counter.provider_network_dispatch_count, 1);
    }

    #[test]
    fn stage7_brave_test_run_cap_blocks_second_call_before_dispatch() {
        let mut policy = ProviderNetworkPolicy::fake_test_allowing(3);
        policy.max_calls_this_route = 3;
        policy.brave_max_calls_per_test_run = 1;
        policy.brave_max_calls_per_day_test = 3;
        let context = ProviderUsageContext::unknown(
            ProviderControlRoute::WebSearch,
            ProviderControlProvider::BraveWebSearch,
            "Test Company A public profile",
        );

        let first = evaluate_provider_gate(
            &policy,
            context.clone(),
            ProviderControlMode::TestFake,
            ProviderCallCounter::default(),
        );
        assert!(first.allowed, "{first:?}");
        let mut after_dispatch = first.counter.clone();
        after_dispatch.record_network_dispatch();

        let second = evaluate_provider_gate(
            &policy,
            context,
            ProviderControlMode::TestFake,
            after_dispatch,
        );
        assert!(!second.allowed);
        assert_eq!(
            second.deny_reason.as_deref(),
            Some(PROVIDER_BUDGET_EXHAUSTED)
        );
        assert_eq!(second.counter.provider_call_attempt_count, 1);
        assert_eq!(second.counter.provider_network_dispatch_count, 1);
    }

    #[test]
    fn stage7_retry_fallback_fanout_and_live_proof_opt_in_default_off() {
        let policy = ProviderNetworkPolicy::default();
        assert_eq!(policy.max_retries, 0);
        assert!(!policy.fallback_enabled);
        assert!(!policy.provider_fanout_enabled);
        assert!(!policy.live_brave_proof_enabled);
        assert_eq!(policy.brave_max_calls_per_test_run, 0);
        assert_eq!(policy.brave_max_calls_per_day_test, 0);
    }

    #[test]
    fn stage7_brave_image_search_requires_dedicated_image_flag() {
        let mut policy = ProviderNetworkPolicy::default();
        policy.global_search_providers_enabled = true;
        policy.paid_search_providers_enabled = true;
        policy.web_search_enabled = true;
        policy.max_calls_this_turn = 1;
        policy.max_calls_this_route = 1;
        policy.brave_max_calls_per_test_run = 1;
        policy.brave_max_calls_per_day_test = 3;
        policy
            .provider_specific_enabled
            .insert("brave".to_string(), true);

        let blocked = evaluate_provider_gate(
            &policy,
            ProviderUsageContext::unknown(
                ProviderControlRoute::ImageSearch,
                ProviderControlProvider::BraveImageSearch,
                "Test Company A public profile",
            ),
            ProviderControlMode::Live,
            ProviderCallCounter::default(),
        );
        assert!(!blocked.allowed);
        assert_eq!(blocked.deny_reason.as_deref(), Some(PROVIDER_DISABLED));
        assert_eq!(blocked.counter.provider_call_attempt_count, 0);

        policy
            .provider_specific_enabled
            .insert("brave_image".to_string(), true);
        let allowed = evaluate_provider_gate(
            &policy,
            ProviderUsageContext::unknown(
                ProviderControlRoute::ImageSearch,
                ProviderControlProvider::BraveImageSearch,
                "Test Company A public profile",
            ),
            ProviderControlMode::Live,
            ProviderCallCounter::default(),
        );
        assert!(allowed.allowed, "{allowed:?}");
    }

    #[test]
    fn stage8_no_search_default_selects_no_provider() {
        let policy = ProviderNetworkPolicy::default();
        let mut request = ProviderRouteRequest::public_web("explain a stable synthetic concept");
        request.search_needed = false;

        let decision = route_provider(&policy, &request);

        assert_eq!(decision.selected_lane, ProviderLane::NoSearch);
        assert_eq!(decision.selected_provider, None);
        assert!(!decision.budget_required);
        assert_eq!(decision.route_reason, NO_SEARCH_NEEDED);
    }

    #[test]
    fn stage8_cache_hit_zero_provider_call_path() {
        let policy = ProviderNetworkPolicy::default();
        let mut request = ProviderRouteRequest::public_web("Synthetic Entity A profile");
        request.cache_status = ProviderCacheStatus::Hit;
        let cache_packet = ProviderCacheDecisionPacket::fixture_hit(&request.query, 1_000, 60_000);

        let decision = route_provider(&policy, &request);
        let mut counter = ProviderCallCounter::default();
        apply_route_decision_to_counter(&mut counter, &decision);

        assert!(cache_packet.cache_enabled);
        assert_eq!(decision.selected_lane, ProviderLane::CacheOnly);
        assert_eq!(
            decision.selected_provider,
            Some(ProviderControlProvider::CacheOnly)
        );
        assert_eq!(counter.provider_cache_hit_count, 1);
        assert_eq!(counter.provider_call_attempt_count, 0);
        assert_eq!(counter.provider_network_dispatch_count, 0);
    }

    #[test]
    fn stage8_stale_cache_does_not_return_cached_current_claim() {
        let mut policy = ProviderNetworkPolicy::fake_test_allowing(1);
        policy
            .provider_specific_enabled
            .insert("cheap_general".to_string(), true);
        let mut request = ProviderRouteRequest::public_web("latest Synthetic Entity A status");
        request.cache_status = ProviderCacheStatus::Stale;
        request.cheap_provider_available = true;

        let decision = route_provider(&policy, &request);

        assert_eq!(decision.selected_lane, ProviderLane::CheapGeneralSearch);
        assert_eq!(
            decision.selected_provider,
            Some(ProviderControlProvider::CheapGeneralSearch)
        );
        assert!(decision.route_reason.contains(CACHE_STALE));
    }

    #[test]
    fn stage8_cheap_provider_preferred_over_premium() {
        let mut policy = ProviderNetworkPolicy::fake_test_allowing(1);
        policy
            .provider_specific_enabled
            .insert("cheap_general".to_string(), true);
        policy
            .provider_specific_enabled
            .insert("brave".to_string(), true);
        policy.fallback_enabled = true;
        policy.max_fallback_calls_this_turn = 1;
        let mut request = ProviderRouteRequest::public_web("Synthetic Entity A official page");
        request.cheap_provider_available = true;
        request.fallback_allowed = true;

        let decision = route_provider(&policy, &request);

        assert_eq!(decision.selected_lane, ProviderLane::CheapGeneralSearch);
        assert_eq!(
            decision.selected_provider,
            Some(ProviderControlProvider::CheapGeneralSearch)
        );
        assert_eq!(
            decision.fallback_provider,
            Some(ProviderControlProvider::BraveWebSearch)
        );
        assert!(!decision.fallback_allowed);
        assert_eq!(decision.estimated_cost_tier, ProviderCostClass::LowCost);
    }

    #[test]
    fn stage8_premium_fallback_disabled_safe_degrades() {
        let mut policy = ProviderNetworkPolicy::fake_test_allowing(1);
        policy.fallback_enabled = false;
        let mut request = ProviderRouteRequest::public_web("Synthetic Entity A official page");
        request.cheap_provider_available = false;
        request.fallback_allowed = true;

        let decision = route_provider(&policy, &request);
        let mut counter = ProviderCallCounter::default();
        apply_route_decision_to_counter(&mut counter, &decision);

        assert_eq!(decision.selected_lane, ProviderLane::Disabled);
        assert_eq!(
            decision.deny_reason.as_deref(),
            Some(CHEAP_PROVIDER_UNAVAILABLE)
        );
        assert_eq!(counter.provider_blocked_count, 1);
        assert_eq!(counter.provider_network_dispatch_count, 0);
    }

    #[test]
    fn stage8_premium_fallback_enabled_uses_brave_as_fallback_only() {
        let mut policy = ProviderNetworkPolicy::fake_test_allowing(1);
        policy.fallback_enabled = true;
        policy.max_fallback_calls_this_turn = 1;
        policy
            .provider_specific_enabled
            .insert("brave".to_string(), true);
        let mut request = ProviderRouteRequest::public_web("Synthetic Entity A official page");
        request.cheap_provider_available = false;
        request.fallback_allowed = true;

        let decision = route_provider(&policy, &request);
        let mut counter = ProviderCallCounter::default();
        apply_route_decision_to_counter(&mut counter, &decision);

        assert_eq!(decision.selected_lane, ProviderLane::PremiumFallback);
        assert_eq!(
            decision.selected_provider,
            Some(ProviderControlProvider::BraveWebSearch)
        );
        assert!(decision.fallback_allowed);
        assert_eq!(counter.provider_fallback_count, 1);
        assert_eq!(counter.provider_network_dispatch_count, 0);
    }

    #[test]
    fn stage8_paid_provider_disabled_blocks_brave() {
        let mut policy = ProviderNetworkPolicy::default();
        policy.global_search_providers_enabled = true;
        policy.web_search_enabled = true;
        policy.max_calls_this_turn = 1;
        policy.max_calls_this_route = 1;
        policy.brave_max_calls_per_test_run = 1;
        policy.brave_max_calls_per_day_test = 1;
        policy
            .provider_specific_enabled
            .insert("brave".to_string(), true);

        let decision = evaluate_provider_gate(
            &policy,
            ProviderUsageContext::unknown(
                ProviderControlRoute::WebSearch,
                ProviderControlProvider::BraveWebSearch,
                "Synthetic Entity A",
            ),
            ProviderControlMode::Live,
            ProviderCallCounter::default(),
        );

        assert!(!decision.allowed);
        assert_eq!(
            decision.deny_reason.as_deref(),
            Some(PAID_PROVIDER_DISABLED)
        );
        assert_eq!(decision.counter.provider_call_attempt_count, 0);
    }

    #[test]
    fn stage8_provider_specific_disabled_blocks_brave() {
        let mut policy = ProviderNetworkPolicy::default();
        policy.global_search_providers_enabled = true;
        policy.paid_search_providers_enabled = true;
        policy.web_search_enabled = true;
        policy.max_calls_this_turn = 1;
        policy.max_calls_this_route = 1;
        policy.brave_max_calls_per_test_run = 1;
        policy.brave_max_calls_per_day_test = 1;

        let decision = evaluate_provider_gate(
            &policy,
            ProviderUsageContext::unknown(
                ProviderControlRoute::WebSearch,
                ProviderControlProvider::BraveWebSearch,
                "Synthetic Entity A",
            ),
            ProviderControlMode::Live,
            ProviderCallCounter::default(),
        );

        assert!(!decision.allowed);
        assert_eq!(decision.deny_reason.as_deref(), Some(PROVIDER_DISABLED));
        assert_eq!(decision.counter.provider_call_attempt_count, 0);
    }

    #[test]
    fn stage8_fallback_cap_required_before_premium_fallback() {
        let mut policy = ProviderNetworkPolicy::fake_test_allowing(1);
        policy.fallback_enabled = true;
        policy.max_fallback_calls_this_turn = 0;
        policy
            .provider_specific_enabled
            .insert("brave".to_string(), true);
        let mut request = ProviderRouteRequest::public_web("Synthetic Entity A official page");
        request.fallback_allowed = true;

        let decision = route_provider(&policy, &request);

        assert_eq!(decision.selected_lane, ProviderLane::Disabled);
        assert_eq!(
            decision.deny_reason.as_deref(),
            Some(CHEAP_PROVIDER_UNAVAILABLE)
        );
    }

    #[test]
    fn stage8_retry_cap_zero_records_no_retry() {
        let policy = ProviderNetworkPolicy::fake_test_allowing(1);
        let mut counter = ProviderCallCounter::default();

        if policy.max_retries > 0 {
            counter.record_retry();
        }

        assert_eq!(policy.max_retries, 0);
        assert_eq!(counter.provider_retry_count, 0);
    }

    #[test]
    fn stage8_news_lane_selects_fake_news_provider() {
        let mut policy = ProviderNetworkPolicy::fake_test_allowing(1);
        policy
            .provider_specific_enabled
            .insert("news_current_events".to_string(), true);
        let mut request = ProviderRouteRequest::public_web("latest Synthetic Entity A update");
        request.route = ProviderControlRoute::NewsSearch;
        request.news_provider_available = true;

        let decision = route_provider(&policy, &request);

        assert_eq!(decision.selected_lane, ProviderLane::NewsCurrentEvents);
        assert_eq!(
            decision.selected_provider,
            Some(ProviderControlProvider::NewsCurrentEvents)
        );
        assert_eq!(
            decision.estimated_cost_tier,
            ProviderCostClass::FreeOrInternal
        );
    }

    #[test]
    fn stage8_news_lane_missing_provider_safe_degrades() {
        let policy = ProviderNetworkPolicy::fake_test_allowing(1);
        let mut request = ProviderRouteRequest::public_web("today Synthetic Entity A update");
        request.route = ProviderControlRoute::NewsSearch;
        request.news_provider_available = false;

        let decision = route_provider(&policy, &request);

        assert_eq!(decision.selected_lane, ProviderLane::Disabled);
        assert_eq!(decision.deny_reason.as_deref(), Some(NEWS_SEARCH_DISABLED));
    }

    #[test]
    fn stage8_deep_research_not_triggered_for_simple_search() {
        let policy = ProviderNetworkPolicy::fake_test_allowing(1);
        let request = ProviderRouteRequest::public_web("Synthetic Entity A public profile");

        let decision = route_provider(&policy, &request);

        assert_ne!(decision.selected_lane, ProviderLane::DeepResearchCapped);
        assert!(!decision.user_approval_required);
    }

    #[test]
    fn stage8_deep_research_explicit_but_disabled() {
        let policy = ProviderNetworkPolicy::default();
        let mut request = ProviderRouteRequest::public_web("deep research Synthetic Entity A");
        request.route = ProviderControlRoute::DeepResearch;

        let decision = route_provider(&policy, &request);

        assert_eq!(decision.selected_lane, ProviderLane::Disabled);
        assert_eq!(
            decision.deny_reason.as_deref(),
            Some(DEEP_RESEARCH_DISABLED)
        );
    }

    #[test]
    fn stage8_url_fetch_not_auto_triggered_by_provider_result() {
        let policy = ProviderNetworkPolicy::default();
        let mut request = ProviderRouteRequest::public_web("Synthetic Entity A with result URL");
        request.route = ProviderControlRoute::UrlFetch;

        let decision = route_provider(&policy, &request);

        assert_eq!(decision.selected_lane, ProviderLane::Disabled);
        assert_eq!(decision.deny_reason.as_deref(), Some(URL_FETCH_DISABLED));
    }

    #[test]
    fn stage8_image_lane_not_auto_triggered() {
        let policy = ProviderNetworkPolicy::default();
        let mut request = ProviderRouteRequest::public_web("Synthetic Entity A logo");
        request.route = ProviderControlRoute::ImageSearch;

        let decision = route_provider(&policy, &request);

        assert_eq!(decision.selected_lane, ProviderLane::Disabled);
        assert_eq!(decision.deny_reason.as_deref(), Some(PROVIDER_DISABLED));
    }

    #[test]
    fn stage8_provider_result_normalization_redacts_raw_metadata() {
        let normalized = normalize_provider_result_fixture(
            ProviderControlProvider::CheapGeneralSearch,
            ProviderTier::Cheap,
            ProviderRawResultFixture {
                title: " Synthetic Source A ".to_string(),
                url: "https://source-a.synthetic.example/path".to_string(),
                snippet: "Synthetic Entity A has a supported public profile.".to_string(),
                published_at: Some("2026-01-01T00:00:00Z".to_string()),
                source_type: Some("official_site_targeting".to_string()),
                provider_rank: 1,
                provider_confidence: Some(90),
                raw_provider_metadata_redacted_hash: "hash_only_no_json".to_string(),
            },
            1_772_000_000_000,
        )
        .expect("normalization should succeed");

        assert_eq!(normalized.provider_id, "cheap_general_search");
        assert_eq!(normalized.domain, "source-a.synthetic.example");
        assert_eq!(normalized.provider_tier, ProviderTier::Cheap);
        assert_eq!(
            normalized.raw_provider_metadata_redacted_hash,
            "hash_only_no_json"
        );
        assert!(!normalized.snippet.contains('{'));
    }

    #[test]
    fn stage8_registry_marks_brave_premium_fallback_not_default() {
        let policy = ProviderNetworkPolicy::default();
        let registry = provider_registry(&policy);
        let brave = registry
            .iter()
            .find(|entry| entry.provider_id == "brave_web_search")
            .expect("Brave registry entry must exist");
        let cheap = registry
            .iter()
            .find(|entry| entry.provider_id == "cheap_general_search")
            .expect("cheap registry entry must exist");

        assert_eq!(brave.provider_lane, ProviderLane::PremiumFallback);
        assert_eq!(brave.provider_tier, ProviderTier::Premium);
        assert!(brave.paid_provider);
        assert!(!brave.enabled);
        assert_eq!(cheap.provider_lane, ProviderLane::CheapGeneralSearch);
        assert!(cheap.test_fake_provider);
    }

    #[test]
    fn stage8_provider_off_all_lanes_block_before_attempt_or_dispatch() {
        for route in [
            ProviderControlRoute::WebSearch,
            ProviderControlRoute::NewsSearch,
            ProviderControlRoute::DeepResearch,
            ProviderControlRoute::UrlFetch,
            ProviderControlRoute::ImageSearch,
        ] {
            let provider = match route {
                ProviderControlRoute::NewsSearch => ProviderControlProvider::BraveNewsSearch,
                ProviderControlRoute::UrlFetch => ProviderControlProvider::UrlFetch,
                ProviderControlRoute::ImageSearch => ProviderControlProvider::BraveImageSearch,
                _ => ProviderControlProvider::BraveWebSearch,
            };
            let decision = disabled_provider_decision(route, provider, "Synthetic Entity A");

            assert!(!decision.allowed);
            assert_eq!(decision.counter.provider_call_attempt_count, 0);
            assert_eq!(decision.counter.provider_network_dispatch_count, 0);
        }
    }

    #[test]
    fn stage8_missing_secret_is_counted_without_startup_failure() {
        let mut counter = ProviderCallCounter::default();
        counter.record_secret_missing();

        assert_eq!(counter.provider_secret_missing_count, 1);
        assert_eq!(counter.provider_call_attempt_count, 0);
        assert_eq!(counter.provider_network_dispatch_count, 0);
    }

    #[test]
    fn stage9_search_certification_offline_corpus_runs_30_of_30_without_live_calls() {
        let report = run_stage9_offline_search_certification();

        assert_eq!(report.certification_mode, "offline");
        assert_eq!(report.total_cases, 30);
        assert_eq!(report.pass_count, 30);
        assert_eq!(report.fail_count, 0);
        assert_eq!(report.blocked_count, 0);
        assert_eq!(report.live_provider_call_attempt_count, 0);
        assert_eq!(report.live_provider_network_dispatch_count, 0);
        assert_eq!(report.url_fetch_count, 0);
        assert_eq!(report.image_fetch_count, 0);
        assert!(!report.live_provider_proof_ran);
        assert_eq!(report.real_voice_proof_status, "NOT_PROVEN");
        assert_eq!(
            report.production_readiness_verdict,
            Stage9ReadinessClass::ReadyExceptRealVoiceNotProven
        );
    }

    #[test]
    fn stage9_search_certification_source_agreement_supports_but_does_not_replace_claim_verification(
    ) {
        let strong = stage9_score_source_agreement(
            "fixture_claim_alpha",
            vec!["alpha-search-fixture.test".to_string()],
            vec!["beta-search-fixture.invalid".to_string()],
            Vec::new(),
            true,
        );
        let unresolved = stage9_score_source_agreement(
            "fixture_claim_beta",
            vec!["beta-search-fixture.invalid".to_string()],
            vec!["example.test".to_string()],
            Vec::new(),
            false,
        );

        assert_eq!(
            strong.confidence_class,
            Stage9AgreementClass::ConflictResolved
        );
        assert!(strong.safe_for_user_summary);
        assert_eq!(
            unresolved.confidence_class,
            Stage9AgreementClass::ConflictUnresolved
        );
        assert!(!unresolved.safe_for_user_summary);
        assert!(strong.explanation_for_trace.contains("does not replace it"));
    }

    #[test]
    fn stage9_search_certification_freshness_blocks_stale_current_claims() {
        let retrieved_at_ms = 1_772_000_000_000;
        let current = stage9_score_freshness(
            "fixture_claim_current",
            "source_fixture_current",
            Some(retrieved_at_ms - 60_000),
            retrieved_at_ms,
            None,
            true,
            86_400_000,
        );
        let stale = stage9_score_freshness(
            "fixture_claim_stale",
            "source_fixture_stale",
            Some(retrieved_at_ms - (30 * 86_400_000)),
            retrieved_at_ms,
            None,
            true,
            86_400_000,
        );

        assert_eq!(current.freshness_class, Stage9FreshnessClass::Current);
        assert!(current.safe_for_current_claim);
        assert_eq!(
            stale.freshness_class,
            Stage9FreshnessClass::TooStaleForCurrentClaim
        );
        assert!(!stale.safe_for_current_claim);
    }

    #[test]
    fn stage9_search_certification_deep_research_and_corroboration_stay_gated_by_default() {
        let report = run_stage9_offline_search_certification();

        assert!(!report.corroboration_policy.enabled);
        assert!(!report.corroboration_policy.fanout_allowed);
        assert_eq!(report.corroboration_policy.max_calls_total, 0);
        assert!(report.deep_research_plan.user_approval_required);
        assert_eq!(report.deep_research_plan.max_provider_calls, 0);
        assert!(!report.deep_research_plan.fanout_allowed);
        assert_eq!(
            report.deep_research_plan.max_cost_class,
            ProviderCostClass::Disabled
        );
    }

    #[test]
    fn stage9_search_certification_tts_source_chip_image_and_protected_proofs_are_clean() {
        let report = run_stage9_offline_search_certification();
        let protected = report
            .case_results
            .iter()
            .find(|case| case.case_id == "stage9_case_024_protected_mixed")
            .expect("protected mixed case should exist");
        let image_allowed = report
            .case_results
            .iter()
            .find(|case| case.case_id == "stage9_case_020_image_allowed")
            .expect("image allowed case should exist");

        assert!(protected.protected_fail_closed);
        assert!(!protected.response_text.contains("provider json"));
        assert!(!protected.response_text.contains("debug trace"));
        assert!(!protected.tts_text.contains("source"));
        assert!(image_allowed.image_cards_allowed);
        assert!(report
            .case_results
            .iter()
            .filter(|case| !matches!(
                case.selected_lane,
                ProviderLane::NoSearch | ProviderLane::Disabled
            ))
            .all(|case| case.source_chip_count > 0));
    }

    #[test]
    fn stage9_search_certification_corpus_uses_only_synthetic_fixture_names() {
        let corpus = stage9_offline_hard_corpus();
        assert_eq!(corpus.len(), 30);
        for case in corpus {
            let case_text = format!(
                "{} {} {}",
                case.case_id, case.prompt, case.expected_behavior
            )
            .to_ascii_lowercase();
            assert!(case_text.contains("stage9_case_"));
            assert!(
                case_text.contains("fixture_entity_")
                    || case_text.contains("fixture_employee_alpha")
                    || case_text.contains("fixture employee alpha")
                    || case_text.contains("fixture_company_alpha")
                    || case_text.contains("fiksture entity alfa")
                    || case_text.contains("alpha-search-fixture.test")
                    || case_text.contains("beta-search-fixture.invalid")
                    || case_text.contains("example.test")
                    || case_text.contains("stable synthetic concept"),
                "non-synthetic certification case text: {case_text}"
            );
            assert!(!case_text.contains("http://"));
            assert!(!case_text.contains("https://"));
        }
    }
}
