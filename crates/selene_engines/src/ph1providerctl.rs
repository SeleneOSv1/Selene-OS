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

pub const WEB_ADMIN_DISABLED: &str = "WEB_ADMIN_DISABLED";
pub const PROVIDER_DISABLED: &str = "PROVIDER_DISABLED";
pub const PAID_PROVIDER_DISABLED: &str = "PAID_PROVIDER_DISABLED";
pub const URL_FETCH_DISABLED: &str = "URL_FETCH_DISABLED";
pub const DEEP_RESEARCH_DISABLED: &str = "DEEP_RESEARCH_DISABLED";
pub const NEWS_SEARCH_DISABLED: &str = "NEWS_SEARCH_DISABLED";
pub const PROVIDER_BUDGET_EXHAUSTED: &str = "PROVIDER_BUDGET_EXHAUSTED";
pub const STARTUP_PROVIDER_PROBES_DISABLED: &str = "STARTUP_PROVIDER_PROBES_DISABLED";
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
}

impl Default for ProviderNetworkPolicy {
    fn default() -> Self {
        let mut provider_specific_enabled = BTreeMap::new();
        provider_specific_enabled.insert("brave".to_string(), false);
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
        }
    }

    pub fn fake_test_allowing(max_calls: u32) -> Self {
        let mut provider_specific_enabled = BTreeMap::new();
        provider_specific_enabled.insert("brave".to_string(), true);
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
        }
    }

    pub fn is_provider_enabled(&self, provider: ProviderControlProvider) -> bool {
        if provider.is_brave() {
            return self
                .provider_specific_enabled
                .get("brave")
                .copied()
                .unwrap_or(false);
        }
        true
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderCallCounter {
    pub logical_search_intent_count: u32,
    pub provider_blocked_count: u32,
    pub provider_call_attempt_count: u32,
    pub provider_network_dispatch_count: u32,
    pub provider_success_count: u32,
    pub provider_failure_count: u32,
    pub provider_retry_count: u32,
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

pub fn evaluate_provider_gate(
    policy: &ProviderNetworkPolicy,
    context: ProviderUsageContext,
    mode: ProviderControlMode,
    mut counter: ProviderCallCounter,
) -> ProviderGateDecision {
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
        return if policy.max_calls_this_turn > 0
            && counter.provider_call_attempt_count >= policy.max_calls_this_turn
        {
            Some(PROVIDER_BUDGET_EXHAUSTED.to_string())
        } else {
            None
        };
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
    if context.provider.is_brave() && !policy.is_provider_enabled(context.provider) {
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
}
