#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use selene_kernel_contracts::ph1link::AppPlatform;
use selene_kernel_contracts::runtime_execution::{
    AdmissionState, FailureClass, RuntimeEntryTrigger,
};

use crate::runtime_bootstrap::{
    RuntimeBootstrapError, RuntimeBuildMetadata, RuntimeClock, RuntimeLifecycleState,
    RuntimeLivenessEndpoint, RuntimePreflightCheckResult, RuntimeProcess, RuntimeReadinessEndpoint,
    RuntimeSecretsProvider, RuntimeStartupEndpoint,
};

pub const FOUNDATION_STATUS_ENDPOINT_PATH: &str = "/runtime/foundation/status";

const MAX_TOKEN_LEN: usize = 128;

mod reason_codes {
    pub const ROUTE_DUPLICATE: &str = "runtime_route_duplicate";
    pub const ROUTE_SCOPE_VIOLATION: &str = "runtime_route_scope_violation";
    pub const ROUTE_MIDDLEWARE_INVALID: &str = "runtime_route_middleware_invalid";
    pub const ROUTE_NOT_FOUND: &str = "runtime_route_not_found";
    pub const ENVELOPE_REQUIRED: &str = "runtime_envelope_required";
    pub const ENVELOPE_INVALID: &str = "runtime_envelope_invalid";
    pub const SECURITY_HEADER_INVALID: &str = "runtime_security_header_invalid";
    pub const REPLAY_REJECTED: &str = "runtime_replay_rejected";
    pub const REQUEST_NOT_READY: &str = "runtime_request_not_ready";
    pub const REQUEST_OVERLOADED: &str = "runtime_request_overloaded";
    pub const REQUEST_SHUTTING_DOWN: &str = "runtime_request_shutting_down";
    pub const FEATURE_DISABLED: &str = "runtime_feature_disabled";
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RuntimeHttpMethod {
    Get,
    Post,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RuntimeRequestClass {
    Health,
    Startup,
    System,
}

impl RuntimeRequestClass {
    pub const fn default_budget_ms(self) -> u32 {
        match self {
            RuntimeRequestClass::Health => 250,
            RuntimeRequestClass::Startup => 500,
            RuntimeRequestClass::System => 1_500,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            RuntimeRequestClass::Health => "HEALTH",
            RuntimeRequestClass::Startup => "STARTUP",
            RuntimeRequestClass::System => "SYSTEM",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RuntimeRouteHandlerKind {
    Liveness,
    Readiness,
    Startup,
    FoundationStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RuntimeRouteMiddlewareKind {
    EnvelopeFoundation,
    RequestSecurity,
    AdmissionControl,
    FeatureFlags,
    InvariantValidation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RuntimeFeatureFlag {
    FoundationStatusRoute,
    DiagnosticMode,
    EmitRuntimeMetrics,
    EmitRuntimeEvents,
}

impl RuntimeFeatureFlag {
    pub const fn as_str(self) -> &'static str {
        match self {
            RuntimeFeatureFlag::FoundationStatusRoute => "FOUNDATION_STATUS_ROUTE",
            RuntimeFeatureFlag::DiagnosticMode => "DIAGNOSTIC_MODE",
            RuntimeFeatureFlag::EmitRuntimeMetrics => "EMIT_RUNTIME_METRICS",
            RuntimeFeatureFlag::EmitRuntimeEvents => "EMIT_RUNTIME_EVENTS",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RuntimeRequestStage {
    Received,
    EnvelopeCreated,
    SecurityValidated,
    Admitted,
    Routed,
    Responded,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeAdmissionPolicy {
    AlwaysAllow,
    RequireReady,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeAdmissionOutcome {
    Admitted,
    RejectedNotReady,
    RejectedOverloaded,
    RejectedShuttingDown,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeRouteKey {
    pub method: RuntimeHttpMethod,
    pub path: String,
}

impl RuntimeRouteKey {
    pub fn new(
        method: RuntimeHttpMethod,
        path: impl Into<String>,
    ) -> Result<Self, RuntimeRequestFoundationError> {
        let key = Self {
            method,
            path: path.into(),
        };
        validate_route_path(&key.path)?;
        Ok(key)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeRouteDefinition {
    pub key: RuntimeRouteKey,
    pub handler: RuntimeRouteHandlerKind,
    pub request_class: RuntimeRequestClass,
    pub admission_policy: RuntimeAdmissionPolicy,
    pub required_middleware: BTreeSet<RuntimeRouteMiddlewareKind>,
    pub description: &'static str,
}

impl RuntimeRouteDefinition {
    pub fn liveness() -> Result<Self, RuntimeRequestFoundationError> {
        Ok(Self {
            key: RuntimeRouteKey::new(
                RuntimeHttpMethod::Get,
                crate::runtime_bootstrap::LIVENESS_ENDPOINT_PATH,
            )?,
            handler: RuntimeRouteHandlerKind::Liveness,
            request_class: RuntimeRequestClass::Health,
            admission_policy: RuntimeAdmissionPolicy::AlwaysAllow,
            required_middleware: BTreeSet::new(),
            description: "Slice 1A liveness surface through Slice 1B routing substrate",
        })
    }

    pub fn readiness() -> Result<Self, RuntimeRequestFoundationError> {
        Ok(Self {
            key: RuntimeRouteKey::new(
                RuntimeHttpMethod::Get,
                crate::runtime_bootstrap::READINESS_ENDPOINT_PATH,
            )?,
            handler: RuntimeRouteHandlerKind::Readiness,
            request_class: RuntimeRequestClass::Health,
            admission_policy: RuntimeAdmissionPolicy::AlwaysAllow,
            required_middleware: BTreeSet::new(),
            description: "Slice 1A readiness surface through Slice 1B routing substrate",
        })
    }

    pub fn startup() -> Result<Self, RuntimeRequestFoundationError> {
        Ok(Self {
            key: RuntimeRouteKey::new(
                RuntimeHttpMethod::Get,
                crate::runtime_bootstrap::STARTUP_ENDPOINT_PATH,
            )?,
            handler: RuntimeRouteHandlerKind::Startup,
            request_class: RuntimeRequestClass::Startup,
            admission_policy: RuntimeAdmissionPolicy::AlwaysAllow,
            required_middleware: BTreeSet::new(),
            description: "Slice 1A startup surface through Slice 1B routing substrate",
        })
    }

    pub fn foundation_status() -> Result<Self, RuntimeRequestFoundationError> {
        Ok(Self {
            key: RuntimeRouteKey::new(RuntimeHttpMethod::Get, FOUNDATION_STATUS_ENDPOINT_PATH)?,
            handler: RuntimeRouteHandlerKind::FoundationStatus,
            request_class: RuntimeRequestClass::System,
            admission_policy: RuntimeAdmissionPolicy::RequireReady,
            required_middleware: BTreeSet::from([
                RuntimeRouteMiddlewareKind::EnvelopeFoundation,
                RuntimeRouteMiddlewareKind::RequestSecurity,
                RuntimeRouteMiddlewareKind::AdmissionControl,
                RuntimeRouteMiddlewareKind::FeatureFlags,
                RuntimeRouteMiddlewareKind::InvariantValidation,
            ]),
            description: "Slice 1B bounded runtime-foundation status route",
        })
    }

    fn requires_envelope(&self) -> bool {
        self.required_middleware
            .contains(&RuntimeRouteMiddlewareKind::EnvelopeFoundation)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFeatureFlagDefinition {
    pub flag: RuntimeFeatureFlag,
    pub default_enabled: bool,
    pub description: &'static str,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimeFeatureFlagOverrides {
    values: BTreeMap<RuntimeFeatureFlag, bool>,
}

impl RuntimeFeatureFlagOverrides {
    pub fn set(&mut self, flag: RuntimeFeatureFlag, enabled: bool) {
        self.values.insert(flag, enabled);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFeatureFlagSnapshot {
    values: BTreeMap<RuntimeFeatureFlag, bool>,
}

impl RuntimeFeatureFlagSnapshot {
    pub fn is_enabled(&self, flag: RuntimeFeatureFlag) -> bool {
        self.values.get(&flag).copied().unwrap_or(false)
    }

    pub fn active_flags(&self) -> Vec<RuntimeFeatureFlag> {
        self.values
            .iter()
            .filter_map(|(flag, enabled)| enabled.then_some(*flag))
            .collect()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimeFeatureFlagRegistry {
    definitions: BTreeMap<RuntimeFeatureFlag, RuntimeFeatureFlagDefinition>,
}

impl RuntimeFeatureFlagRegistry {
    pub fn with_slice_1b_defaults() -> Result<Self, RuntimeRequestFoundationError> {
        let mut registry = Self::default();
        registry.register(RuntimeFeatureFlagDefinition {
            flag: RuntimeFeatureFlag::FoundationStatusRoute,
            default_enabled: true,
            description: "Enables the bounded Slice 1B runtime foundation status route",
        })?;
        registry.register(RuntimeFeatureFlagDefinition {
            flag: RuntimeFeatureFlag::DiagnosticMode,
            default_enabled: false,
            description: "Enables verbose runtime foundation diagnostics on the status route",
        })?;
        registry.register(RuntimeFeatureFlagDefinition {
            flag: RuntimeFeatureFlag::EmitRuntimeMetrics,
            default_enabled: true,
            description: "Emits standardized routing and admission metrics",
        })?;
        registry.register(RuntimeFeatureFlagDefinition {
            flag: RuntimeFeatureFlag::EmitRuntimeEvents,
            default_enabled: true,
            description: "Publishes bounded routing and admission events",
        })?;
        Ok(registry)
    }

    pub fn register(
        &mut self,
        definition: RuntimeFeatureFlagDefinition,
    ) -> Result<(), RuntimeRequestFoundationError> {
        if self.definitions.contains_key(&definition.flag) {
            return Err(RuntimeRequestFoundationError::invalid_route_middleware(
                "duplicate feature flag definition",
            ));
        }
        self.definitions.insert(definition.flag, definition);
        Ok(())
    }

    pub fn snapshot(
        &self,
        overrides: &RuntimeFeatureFlagOverrides,
    ) -> Result<RuntimeFeatureFlagSnapshot, RuntimeRequestFoundationError> {
        let mut values = BTreeMap::new();
        for (flag, definition) in &self.definitions {
            let enabled = overrides
                .values
                .get(flag)
                .copied()
                .unwrap_or(definition.default_enabled);
            values.insert(*flag, enabled);
        }
        for flag in overrides.values.keys() {
            if !self.definitions.contains_key(flag) {
                return Err(RuntimeRequestFoundationError::feature_disabled(
                    "feature override references an unregistered flag",
                ));
            }
        }
        Ok(RuntimeFeatureFlagSnapshot { values })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeRequestFoundationConfig {
    pub service_name: String,
    pub build_metadata: RuntimeBuildMetadata,
    pub max_clock_skew_ms: i64,
}

impl RuntimeRequestFoundationConfig {
    pub fn validate(&self) -> Result<(), RuntimeRequestFoundationError> {
        validate_ascii_token(
            "runtime_request_foundation_config.service_name",
            &self.service_name,
        )?;
        validate_ascii_token(
            "runtime_request_foundation_config.node_id",
            &self.build_metadata.node_id,
        )?;
        validate_ascii_token(
            "runtime_request_foundation_config.runtime_instance_identity",
            &self.build_metadata.runtime_instance_identity,
        )?;
        validate_ascii_token(
            "runtime_request_foundation_config.environment_identity",
            &self.build_metadata.environment_identity,
        )?;
        validate_ascii_token(
            "runtime_request_foundation_config.build_version",
            &self.build_metadata.build_version,
        )?;
        validate_ascii_token(
            "runtime_request_foundation_config.git_commit",
            &self.build_metadata.git_commit,
        )?;
        if self.max_clock_skew_ms <= 0 {
            return Err(RuntimeRequestFoundationError::invalid_envelope(
                "runtime_request_foundation_config.max_clock_skew_ms must be positive",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeRequestOrigin {
    pub platform: AppPlatform,
    pub trigger: RuntimeEntryTrigger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeRequestEnvelopeInput {
    pub request_id: String,
    pub trace_id: String,
    pub idempotency_key: String,
    pub timestamp_ms: i64,
    pub nonce: String,
    pub origin: RuntimeRequestOrigin,
}

impl RuntimeRequestEnvelopeInput {
    fn validate(&self) -> Result<(), RuntimeRequestFoundationError> {
        validate_ascii_token(
            "runtime_request_envelope_input.request_id",
            &self.request_id,
        )?;
        validate_ascii_token("runtime_request_envelope_input.trace_id", &self.trace_id)?;
        validate_ascii_token(
            "runtime_request_envelope_input.idempotency_key",
            &self.idempotency_key,
        )?;
        validate_ascii_token("runtime_request_envelope_input.nonce", &self.nonce)?;
        if self.timestamp_ms <= 0 {
            return Err(RuntimeRequestFoundationError::invalid_security_header(
                "runtime_request_envelope_input.timestamp_ms must be positive",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFoundationRequest {
    pub key: RuntimeRouteKey,
    pub envelope_input: Option<RuntimeRequestEnvelopeInput>,
    pub overload_active: bool,
    pub feature_flag_overrides: RuntimeFeatureFlagOverrides,
}

impl RuntimeFoundationRequest {
    pub fn health(path: &str) -> Result<Self, RuntimeRequestFoundationError> {
        Ok(Self {
            key: RuntimeRouteKey::new(RuntimeHttpMethod::Get, path)?,
            envelope_input: None,
            overload_active: false,
            feature_flag_overrides: RuntimeFeatureFlagOverrides::default(),
        })
    }

    pub fn system_status(
        envelope_input: RuntimeRequestEnvelopeInput,
        feature_flag_overrides: RuntimeFeatureFlagOverrides,
    ) -> Result<Self, RuntimeRequestFoundationError> {
        Ok(Self {
            key: RuntimeRouteKey::new(RuntimeHttpMethod::Get, FOUNDATION_STATUS_ENDPOINT_PATH)?,
            envelope_input: Some(envelope_input),
            overload_active: false,
            feature_flag_overrides,
        })
    }

    pub fn with_overload_active(mut self, overload_active: bool) -> Self {
        self.overload_active = overload_active;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeExecutionBudget {
    pub total_budget_ms: u32,
    pub remaining_budget_ms: u32,
}

impl RuntimeExecutionBudget {
    pub fn new(total_budget_ms: u32) -> Result<Self, RuntimeRequestFoundationError> {
        if total_budget_ms == 0 {
            return Err(RuntimeRequestFoundationError::invalid_envelope(
                "runtime execution budget must be greater than zero",
            ));
        }
        Ok(Self {
            total_budget_ms,
            remaining_budget_ms: total_budget_ms,
        })
    }

    fn default_for_request_class(
        request_class: RuntimeRequestClass,
    ) -> Result<Self, RuntimeRequestFoundationError> {
        Self::new(request_class.default_budget_ms())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeRequestEnvelopeHeader {
    pub request_id: String,
    pub trace_id: String,
    pub idempotency_key: String,
    pub received_at_ms: i64,
    pub node_id: String,
    pub runtime_instance_identity: String,
    pub build_version: String,
    pub git_commit: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeRequestStageRecord {
    pub stage: RuntimeRequestStage,
    pub at_unix_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeRequestEnvelopeFoundation {
    header: RuntimeRequestEnvelopeHeader,
    route_key: RuntimeRouteKey,
    request_class: RuntimeRequestClass,
    origin: RuntimeRequestOrigin,
    feature_flags: RuntimeFeatureFlagSnapshot,
    execution_budget: RuntimeExecutionBudget,
    admission_state: AdmissionState,
    stage: RuntimeRequestStage,
    stage_history: Vec<RuntimeRequestStageRecord>,
}

impl RuntimeRequestEnvelopeFoundation {
    fn create(
        config: &RuntimeRequestFoundationConfig,
        request_class: RuntimeRequestClass,
        route_key: RuntimeRouteKey,
        input: RuntimeRequestEnvelopeInput,
        received_at_ms: i64,
        feature_flags: RuntimeFeatureFlagSnapshot,
    ) -> Result<Self, RuntimeRequestFoundationError> {
        input.validate()?;
        let header = RuntimeRequestEnvelopeHeader {
            request_id: input.request_id,
            trace_id: input.trace_id,
            idempotency_key: input.idempotency_key,
            received_at_ms,
            node_id: config.build_metadata.node_id.clone(),
            runtime_instance_identity: config.build_metadata.runtime_instance_identity.clone(),
            build_version: config.build_metadata.build_version.clone(),
            git_commit: config.build_metadata.git_commit.clone(),
        };
        Ok(Self {
            header,
            route_key,
            request_class,
            origin: input.origin,
            feature_flags,
            execution_budget: RuntimeExecutionBudget::default_for_request_class(request_class)?,
            admission_state: AdmissionState::IngressValidated,
            stage: RuntimeRequestStage::EnvelopeCreated,
            stage_history: vec![
                RuntimeRequestStageRecord {
                    stage: RuntimeRequestStage::Received,
                    at_unix_ms: received_at_ms,
                },
                RuntimeRequestStageRecord {
                    stage: RuntimeRequestStage::EnvelopeCreated,
                    at_unix_ms: received_at_ms,
                },
            ],
        })
    }

    pub fn header(&self) -> &RuntimeRequestEnvelopeHeader {
        &self.header
    }

    pub fn route_key(&self) -> &RuntimeRouteKey {
        &self.route_key
    }

    pub fn request_class(&self) -> RuntimeRequestClass {
        self.request_class
    }

    pub fn current_stage(&self) -> RuntimeRequestStage {
        self.stage
    }

    pub fn stage_history(&self) -> &[RuntimeRequestStageRecord] {
        &self.stage_history
    }

    pub fn feature_flags(&self) -> &RuntimeFeatureFlagSnapshot {
        &self.feature_flags
    }

    pub fn execution_budget(&self) -> RuntimeExecutionBudget {
        self.execution_budget
    }

    pub fn admission_state(&self) -> AdmissionState {
        self.admission_state
    }

    pub fn origin(&self) -> &RuntimeRequestOrigin {
        &self.origin
    }

    fn advance_to(mut self, stage: RuntimeRequestStage, at_unix_ms: i64) -> Self {
        self.stage = stage;
        self.stage_history
            .push(RuntimeRequestStageRecord { stage, at_unix_ms });
        self
    }

    fn with_admission_state(mut self, admission_state: AdmissionState) -> Self {
        self.admission_state = admission_state;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeAdmissionDecision {
    pub admission_state: AdmissionState,
    pub outcome: RuntimeAdmissionOutcome,
    pub reason_code: Option<&'static str>,
}

impl RuntimeAdmissionDecision {
    fn admitted() -> Self {
        Self {
            admission_state: AdmissionState::ExecutionAdmitted,
            outcome: RuntimeAdmissionOutcome::Admitted,
            reason_code: None,
        }
    }

    fn rejected(outcome: RuntimeAdmissionOutcome, reason_code: &'static str) -> Self {
        Self {
            admission_state: AdmissionState::Rejected,
            outcome,
            reason_code: Some(reason_code),
        }
    }

    fn is_allowed(&self) -> bool {
        self.outcome == RuntimeAdmissionOutcome::Admitted
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RuntimeReplayProtectionState {
    seen_pairs: BTreeSet<(String, String)>,
}

impl RuntimeReplayProtectionState {
    fn check_and_record(
        &mut self,
        request_id: &str,
        nonce: &str,
    ) -> Result<(), RuntimeRequestFoundationError> {
        let key = (request_id.to_string(), nonce.to_string());
        if !self.seen_pairs.insert(key) {
            return Err(RuntimeRequestFoundationError::replay_rejected(
                "duplicate request_id/nonce pair was rejected",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RuntimeEventBus {
    events: Vec<RuntimeFoundationEvent>,
}

impl RuntimeEventBus {
    fn publish(&mut self, event: RuntimeFoundationEvent) {
        self.events.push(event);
    }

    pub fn events(&self) -> &[RuntimeFoundationEvent] {
        &self.events
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeFoundationEvent {
    RouteRegistered {
        method: RuntimeHttpMethod,
        path: String,
    },
    RequestDispatched {
        request_id: Option<String>,
        path: String,
        request_class: RuntimeRequestClass,
    },
    AdmissionEvaluated {
        request_id: Option<String>,
        path: String,
        outcome: RuntimeAdmissionOutcome,
    },
    RequestRejected {
        request_id: Option<String>,
        path: String,
        reason_code: &'static str,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RuntimeMetricName {
    RouteDispatchTotal,
    AdmissionDecisionTotal,
    RequestRejectedTotal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeMetricSample {
    pub metric_name: RuntimeMetricName,
    pub service_name: String,
    pub node_id: String,
    pub route_path: String,
    pub request_class: RuntimeRequestClass,
    pub outcome: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RuntimeMetricsCollector {
    samples: Vec<RuntimeMetricSample>,
}

impl RuntimeMetricsCollector {
    fn emit(&mut self, sample: RuntimeMetricSample) {
        self.samples.push(sample);
    }

    pub fn samples(&self) -> &[RuntimeMetricSample] {
        &self.samples
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeCapabilityManifest {
    pub service_name: String,
    pub node_id: String,
    pub runtime_instance_identity: String,
    pub environment_identity: String,
    pub build_version: String,
    pub git_commit: String,
    pub routes: Vec<String>,
    pub active_feature_flags: Vec<RuntimeFeatureFlag>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFoundationStatusResponse {
    pub request_id: String,
    pub trace_id: String,
    pub received_at_ms: i64,
    pub service_name: String,
    pub node_id: String,
    pub runtime_instance_identity: String,
    pub environment_identity: String,
    pub build_version: String,
    pub git_commit: String,
    pub runtime_state: RuntimeLifecycleState,
    pub liveness: RuntimeLivenessEndpoint,
    pub readiness: RuntimeReadinessEndpoint,
    pub startup: RuntimeStartupEndpoint,
    pub capability_manifest: RuntimeCapabilityManifest,
    pub diagnostic_mode_enabled: bool,
    pub execution_budget: RuntimeExecutionBudget,
    pub preflight_results: Option<Vec<RuntimePreflightCheckResult>>,
    pub service_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeRouteResponse {
    Liveness(RuntimeLivenessEndpoint),
    Readiness(RuntimeReadinessEndpoint),
    Startup(RuntimeStartupEndpoint),
    FoundationStatus(Box<RuntimeFoundationStatusResponse>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeRouteDispatchResult {
    pub response: RuntimeRouteResponse,
    pub envelope: Option<RuntimeRequestEnvelopeFoundation>,
    pub admission: RuntimeAdmissionDecision,
}

#[derive(Debug, Clone)]
pub struct RuntimeRouter {
    config: RuntimeRequestFoundationConfig,
    feature_flags: RuntimeFeatureFlagRegistry,
    routes: BTreeMap<RuntimeRouteKey, RuntimeRouteDefinition>,
    replay_protection: RuntimeReplayProtectionState,
    event_bus: RuntimeEventBus,
    metrics: RuntimeMetricsCollector,
}

impl RuntimeRouter {
    pub fn with_slice_1b_foundation_defaults(
        config: RuntimeRequestFoundationConfig,
    ) -> Result<Self, RuntimeRequestFoundationError> {
        config.validate()?;
        let mut router = Self {
            config,
            feature_flags: RuntimeFeatureFlagRegistry::with_slice_1b_defaults()?,
            routes: BTreeMap::new(),
            replay_protection: RuntimeReplayProtectionState::default(),
            event_bus: RuntimeEventBus::default(),
            metrics: RuntimeMetricsCollector::default(),
        };
        router.register_route(RuntimeRouteDefinition::liveness()?)?;
        router.register_route(RuntimeRouteDefinition::readiness()?)?;
        router.register_route(RuntimeRouteDefinition::startup()?)?;
        router.register_route(RuntimeRouteDefinition::foundation_status()?)?;
        Ok(router)
    }

    pub fn register_route(
        &mut self,
        definition: RuntimeRouteDefinition,
    ) -> Result<(), RuntimeRequestFoundationError> {
        validate_route_definition(&definition)?;
        if self.routes.contains_key(&definition.key) {
            return Err(RuntimeRequestFoundationError::duplicate_route(
                definition.key.path.as_str(),
            ));
        }
        self.event_bus
            .publish(RuntimeFoundationEvent::RouteRegistered {
                method: definition.key.method,
                path: definition.key.path.clone(),
            });
        self.routes.insert(definition.key.clone(), definition);
        Ok(())
    }

    pub fn route_keys(&self) -> Vec<&RuntimeRouteKey> {
        self.routes.keys().collect()
    }

    pub fn event_bus(&self) -> &RuntimeEventBus {
        &self.event_bus
    }

    pub fn metrics(&self) -> &RuntimeMetricsCollector {
        &self.metrics
    }

    pub fn capability_manifest(
        &self,
        snapshot: &RuntimeFeatureFlagSnapshot,
    ) -> RuntimeCapabilityManifest {
        RuntimeCapabilityManifest {
            service_name: self.config.service_name.clone(),
            node_id: self.config.build_metadata.node_id.clone(),
            runtime_instance_identity: self.config.build_metadata.runtime_instance_identity.clone(),
            environment_identity: self.config.build_metadata.environment_identity.clone(),
            build_version: self.config.build_metadata.build_version.clone(),
            git_commit: self.config.build_metadata.git_commit.clone(),
            routes: self.routes.keys().map(|key| key.path.clone()).collect(),
            active_feature_flags: snapshot.active_flags(),
        }
    }

    pub fn dispatch<C, S>(
        &mut self,
        runtime: &RuntimeProcess<C, S>,
        request: RuntimeFoundationRequest,
    ) -> Result<RuntimeRouteDispatchResult, RuntimeRequestFoundationError>
    where
        C: RuntimeClock,
        S: RuntimeSecretsProvider,
    {
        let definition = self.routes.get(&request.key).cloned().ok_or_else(|| {
            RuntimeRequestFoundationError::route_not_found(request.key.path.as_str())
        })?;
        let snapshot = self
            .feature_flags
            .snapshot(&request.feature_flag_overrides)?;
        let now_unix_ms = runtime.clock().now_unix_ms();

        self.publish_dispatch_event(&definition, request.envelope_input.as_ref(), &snapshot);

        let mut envelope = if definition.requires_envelope() {
            let input = request
                .envelope_input
                .clone()
                .ok_or_else(RuntimeRequestFoundationError::envelope_required)?;
            Some(RuntimeRequestEnvelopeFoundation::create(
                &self.config,
                definition.request_class,
                definition.key.clone(),
                input,
                now_unix_ms,
                snapshot.clone(),
            )?)
        } else {
            None
        };

        if definition
            .required_middleware
            .contains(&RuntimeRouteMiddlewareKind::RequestSecurity)
        {
            let validated = self.validate_request_security(
                envelope
                    .take()
                    .expect("envelope must exist when request security is required"),
                request
                    .envelope_input
                    .as_ref()
                    .expect("envelope input must exist when request security is required"),
                now_unix_ms,
            )?;
            envelope = Some(validated);
        }

        let admission = evaluate_admission(runtime, &definition, request.overload_active);
        self.publish_admission_event(&definition, envelope.as_ref(), &admission, &snapshot);
        if !admission.is_allowed() {
            let mut rejected_envelope = envelope.map(|existing| {
                existing
                    .with_admission_state(admission.admission_state)
                    .advance_to(RuntimeRequestStage::Rejected, now_unix_ms)
            });
            self.emit_rejection_metric(
                &definition,
                definition.request_class,
                admission
                    .reason_code
                    .expect("rejected admission must carry reason code"),
                &snapshot,
            );
            self.event_bus
                .publish(RuntimeFoundationEvent::RequestRejected {
                    request_id: rejected_envelope
                        .as_ref()
                        .map(|env| env.header().request_id.clone()),
                    path: definition.key.path.clone(),
                    reason_code: admission
                        .reason_code
                        .expect("rejected admission must carry reason code"),
                });
            let message = format!(
                "request for {} rejected by admission control",
                definition.key.path
            );
            let error = RuntimeRequestFoundationError::admission_rejected(
                admission
                    .reason_code
                    .expect("rejected admission must carry reason code"),
                message,
            );
            if let Some(rejected_envelope) = rejected_envelope.take() {
                return Err(error.with_envelope(rejected_envelope));
            }
            return Err(error);
        }

        if definition.handler == RuntimeRouteHandlerKind::FoundationStatus
            && !snapshot.is_enabled(RuntimeFeatureFlag::FoundationStatusRoute)
        {
            let reason_code = reason_codes::FEATURE_DISABLED;
            self.emit_rejection_metric(
                &definition,
                definition.request_class,
                reason_code,
                &snapshot,
            );
            self.event_bus
                .publish(RuntimeFoundationEvent::RequestRejected {
                    request_id: envelope.as_ref().map(|env| env.header().request_id.clone()),
                    path: definition.key.path.clone(),
                    reason_code,
                });
            return Err(RuntimeRequestFoundationError::feature_disabled(
                "foundation status route is disabled by feature flag",
            ));
        }

        let response = match definition.handler {
            RuntimeRouteHandlerKind::Liveness => {
                emit_success_metrics(
                    &self.config,
                    &mut self.metrics,
                    &definition,
                    &snapshot,
                    "ok",
                );
                RuntimeRouteResponse::Liveness(runtime.liveness_endpoint())
            }
            RuntimeRouteHandlerKind::Readiness => {
                emit_success_metrics(
                    &self.config,
                    &mut self.metrics,
                    &definition,
                    &snapshot,
                    "ok",
                );
                RuntimeRouteResponse::Readiness(runtime.readiness_endpoint())
            }
            RuntimeRouteHandlerKind::Startup => {
                emit_success_metrics(
                    &self.config,
                    &mut self.metrics,
                    &definition,
                    &snapshot,
                    "ok",
                );
                RuntimeRouteResponse::Startup(runtime.startup_endpoint())
            }
            RuntimeRouteHandlerKind::FoundationStatus => {
                let routed_envelope = envelope
                    .take()
                    .expect("foundation status route requires an envelope")
                    .with_admission_state(admission.admission_state)
                    .advance_to(RuntimeRequestStage::Admitted, now_unix_ms)
                    .advance_to(RuntimeRequestStage::Routed, now_unix_ms)
                    .advance_to(RuntimeRequestStage::Responded, now_unix_ms);
                let response = RuntimeRouteResponse::FoundationStatus(Box::new(
                    RuntimeFoundationStatusResponse {
                        request_id: routed_envelope.header().request_id.clone(),
                        trace_id: routed_envelope.header().trace_id.clone(),
                        received_at_ms: routed_envelope.header().received_at_ms,
                        service_name: self.config.service_name.clone(),
                        node_id: self.config.build_metadata.node_id.clone(),
                        runtime_instance_identity: self
                            .config
                            .build_metadata
                            .runtime_instance_identity
                            .clone(),
                        environment_identity: self
                            .config
                            .build_metadata
                            .environment_identity
                            .clone(),
                        build_version: self.config.build_metadata.build_version.clone(),
                        git_commit: self.config.build_metadata.git_commit.clone(),
                        runtime_state: runtime.state(),
                        liveness: runtime.liveness_endpoint(),
                        readiness: runtime.readiness_endpoint(),
                        startup: runtime.startup_endpoint(),
                        capability_manifest: self.capability_manifest(&snapshot),
                        diagnostic_mode_enabled: snapshot
                            .is_enabled(RuntimeFeatureFlag::DiagnosticMode),
                        execution_budget: routed_envelope.execution_budget(),
                        preflight_results: snapshot
                            .is_enabled(RuntimeFeatureFlag::DiagnosticMode)
                            .then(|| runtime.preflight_results().to_vec()),
                        service_ids: snapshot
                            .is_enabled(RuntimeFeatureFlag::DiagnosticMode)
                            .then(|| {
                                runtime
                                    .service_ids()
                                    .into_iter()
                                    .map(str::to_string)
                                    .collect()
                            }),
                    },
                ));
                emit_success_metrics(
                    &self.config,
                    &mut self.metrics,
                    &definition,
                    &snapshot,
                    "ok",
                );
                envelope = Some(routed_envelope);
                response
            }
        };

        Ok(RuntimeRouteDispatchResult {
            response,
            envelope,
            admission,
        })
    }

    pub fn register_slice_1b_foundation_services<C, S>(
        container: &mut crate::runtime_bootstrap::RuntimeServiceContainer<C, S>,
    ) -> Result<(), RuntimeBootstrapError>
    where
        C: RuntimeClock,
        S: RuntimeSecretsProvider,
    {
        container.register_service(
            "runtime_route_registry",
            &["runtime_clock", "health_endpoint_registry"],
        )?;
        container.register_service(
            "runtime_request_security_foundation",
            &["runtime_clock", "runtime_route_registry"],
        )?;
        container.register_service(
            "runtime_admission_controller",
            &["startup_preflight_runner", "health_endpoint_registry"],
        )?;
        container.register_service("runtime_feature_flag_registry", &["config_loader"])?;
        container.register_service(
            "runtime_metrics_collector",
            &["runtime_clock", "startup_logger"],
        )?;
        container.register_service(
            "runtime_event_bus",
            &["runtime_clock", "runtime_feature_flag_registry"],
        )?;
        container.register_service(
            "runtime_router",
            &[
                "runtime_route_registry",
                "runtime_request_security_foundation",
                "runtime_admission_controller",
                "runtime_feature_flag_registry",
                "runtime_metrics_collector",
                "runtime_event_bus",
            ],
        )?;
        Ok(())
    }

    fn validate_request_security(
        &mut self,
        envelope: RuntimeRequestEnvelopeFoundation,
        input: &RuntimeRequestEnvelopeInput,
        now_unix_ms: i64,
    ) -> Result<RuntimeRequestEnvelopeFoundation, RuntimeRequestFoundationError> {
        input.validate()?;
        let skew = (now_unix_ms - input.timestamp_ms).abs();
        if skew > self.config.max_clock_skew_ms {
            return Err(RuntimeRequestFoundationError::invalid_security_header(
                "request timestamp fell outside the allowed runtime skew window",
            ));
        }
        self.replay_protection
            .check_and_record(&input.request_id, &input.nonce)?;
        Ok(envelope.advance_to(RuntimeRequestStage::SecurityValidated, now_unix_ms))
    }

    fn publish_dispatch_event(
        &mut self,
        definition: &RuntimeRouteDefinition,
        input: Option<&RuntimeRequestEnvelopeInput>,
        snapshot: &RuntimeFeatureFlagSnapshot,
    ) {
        if snapshot.is_enabled(RuntimeFeatureFlag::EmitRuntimeEvents) {
            self.event_bus
                .publish(RuntimeFoundationEvent::RequestDispatched {
                    request_id: input.map(|value| value.request_id.clone()),
                    path: definition.key.path.clone(),
                    request_class: definition.request_class,
                });
        }
    }

    fn publish_admission_event(
        &mut self,
        definition: &RuntimeRouteDefinition,
        envelope: Option<&RuntimeRequestEnvelopeFoundation>,
        admission: &RuntimeAdmissionDecision,
        snapshot: &RuntimeFeatureFlagSnapshot,
    ) {
        if snapshot.is_enabled(RuntimeFeatureFlag::EmitRuntimeEvents) {
            self.event_bus
                .publish(RuntimeFoundationEvent::AdmissionEvaluated {
                    request_id: envelope.map(|value| value.header().request_id.clone()),
                    path: definition.key.path.clone(),
                    outcome: admission.outcome,
                });
        }
    }

    fn emit_rejection_metric(
        &mut self,
        definition: &RuntimeRouteDefinition,
        request_class: RuntimeRequestClass,
        reason_code: &'static str,
        snapshot: &RuntimeFeatureFlagSnapshot,
    ) {
        if snapshot.is_enabled(RuntimeFeatureFlag::EmitRuntimeMetrics) {
            self.metrics.emit(RuntimeMetricSample {
                metric_name: RuntimeMetricName::RequestRejectedTotal,
                service_name: self.config.service_name.clone(),
                node_id: self.config.build_metadata.node_id.clone(),
                route_path: definition.key.path.clone(),
                request_class,
                outcome: reason_code.to_string(),
            });
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeRequestFoundationError {
    pub kind: RuntimeRequestFoundationErrorKind,
    pub reason_code: &'static str,
    pub failure_class: FailureClass,
    pub message: String,
    pub rejected_envelope: Option<Box<RuntimeRequestEnvelopeFoundation>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeRequestFoundationErrorKind {
    DuplicateRoute,
    RouteScopeViolation,
    InvalidRouteMiddleware,
    RouteNotFound,
    EnvelopeRequired,
    InvalidEnvelope,
    InvalidSecurityHeader,
    ReplayRejected,
    AdmissionRejected,
    FeatureDisabled,
}

impl RuntimeRequestFoundationError {
    fn duplicate_route(path: &str) -> Self {
        Self {
            kind: RuntimeRequestFoundationErrorKind::DuplicateRoute,
            reason_code: reason_codes::ROUTE_DUPLICATE,
            failure_class: FailureClass::InvalidPayload,
            message: format!("duplicate route registration rejected for path {path}"),
            rejected_envelope: None,
        }
    }

    fn scope_violation(message: &str) -> Self {
        Self {
            kind: RuntimeRequestFoundationErrorKind::RouteScopeViolation,
            reason_code: reason_codes::ROUTE_SCOPE_VIOLATION,
            failure_class: FailureClass::PolicyViolation,
            message: message.to_string(),
            rejected_envelope: None,
        }
    }

    fn invalid_route_middleware(message: &str) -> Self {
        Self {
            kind: RuntimeRequestFoundationErrorKind::InvalidRouteMiddleware,
            reason_code: reason_codes::ROUTE_MIDDLEWARE_INVALID,
            failure_class: FailureClass::InvalidPayload,
            message: message.to_string(),
            rejected_envelope: None,
        }
    }

    fn route_not_found(path: &str) -> Self {
        Self {
            kind: RuntimeRequestFoundationErrorKind::RouteNotFound,
            reason_code: reason_codes::ROUTE_NOT_FOUND,
            failure_class: FailureClass::InvalidPayload,
            message: format!("no runtime foundation route is registered for path {path}"),
            rejected_envelope: None,
        }
    }

    fn envelope_required() -> Self {
        Self {
            kind: RuntimeRequestFoundationErrorKind::EnvelopeRequired,
            reason_code: reason_codes::ENVELOPE_REQUIRED,
            failure_class: FailureClass::InvalidPayload,
            message: "route requires a runtime request envelope foundation".to_string(),
            rejected_envelope: None,
        }
    }

    fn invalid_envelope(message: &str) -> Self {
        Self {
            kind: RuntimeRequestFoundationErrorKind::InvalidEnvelope,
            reason_code: reason_codes::ENVELOPE_INVALID,
            failure_class: FailureClass::InvalidPayload,
            message: message.to_string(),
            rejected_envelope: None,
        }
    }

    fn invalid_security_header(message: &str) -> Self {
        Self {
            kind: RuntimeRequestFoundationErrorKind::InvalidSecurityHeader,
            reason_code: reason_codes::SECURITY_HEADER_INVALID,
            failure_class: FailureClass::InvalidPayload,
            message: message.to_string(),
            rejected_envelope: None,
        }
    }

    fn replay_rejected(message: &str) -> Self {
        Self {
            kind: RuntimeRequestFoundationErrorKind::ReplayRejected,
            reason_code: reason_codes::REPLAY_REJECTED,
            failure_class: FailureClass::ReplayRejected,
            message: message.to_string(),
            rejected_envelope: None,
        }
    }

    fn admission_rejected(reason_code: &'static str, message: String) -> Self {
        Self {
            kind: RuntimeRequestFoundationErrorKind::AdmissionRejected,
            reason_code,
            failure_class: FailureClass::RetryableRuntime,
            message,
            rejected_envelope: None,
        }
    }

    fn feature_disabled(message: &str) -> Self {
        Self {
            kind: RuntimeRequestFoundationErrorKind::FeatureDisabled,
            reason_code: reason_codes::FEATURE_DISABLED,
            failure_class: FailureClass::PolicyViolation,
            message: message.to_string(),
            rejected_envelope: None,
        }
    }

    fn with_envelope(mut self, envelope: RuntimeRequestEnvelopeFoundation) -> Self {
        self.rejected_envelope = Some(Box::new(envelope));
        self
    }
}

fn emit_success_metrics(
    config: &RuntimeRequestFoundationConfig,
    metrics: &mut RuntimeMetricsCollector,
    definition: &RuntimeRouteDefinition,
    snapshot: &RuntimeFeatureFlagSnapshot,
    outcome: &str,
) {
    if snapshot.is_enabled(RuntimeFeatureFlag::EmitRuntimeMetrics) {
        metrics.emit(RuntimeMetricSample {
            metric_name: RuntimeMetricName::RouteDispatchTotal,
            service_name: config.service_name.clone(),
            node_id: config.build_metadata.node_id.clone(),
            route_path: definition.key.path.clone(),
            request_class: definition.request_class,
            outcome: outcome.to_string(),
        });
        metrics.emit(RuntimeMetricSample {
            metric_name: RuntimeMetricName::AdmissionDecisionTotal,
            service_name: config.service_name.clone(),
            node_id: config.build_metadata.node_id.clone(),
            route_path: definition.key.path.clone(),
            request_class: definition.request_class,
            outcome: outcome.to_string(),
        });
    }
}

fn evaluate_admission<C, S>(
    runtime: &RuntimeProcess<C, S>,
    definition: &RuntimeRouteDefinition,
    overload_active: bool,
) -> RuntimeAdmissionDecision
where
    C: RuntimeClock,
    S: RuntimeSecretsProvider,
{
    match definition.admission_policy {
        RuntimeAdmissionPolicy::AlwaysAllow => RuntimeAdmissionDecision::admitted(),
        RuntimeAdmissionPolicy::RequireReady => {
            if matches!(
                runtime.state(),
                RuntimeLifecycleState::Draining | RuntimeLifecycleState::ShuttingDown
            ) {
                return RuntimeAdmissionDecision::rejected(
                    RuntimeAdmissionOutcome::RejectedShuttingDown,
                    reason_codes::REQUEST_SHUTTING_DOWN,
                );
            }
            if overload_active {
                return RuntimeAdmissionDecision::rejected(
                    RuntimeAdmissionOutcome::RejectedOverloaded,
                    reason_codes::REQUEST_OVERLOADED,
                );
            }
            if !runtime.readiness_endpoint().ready {
                return RuntimeAdmissionDecision::rejected(
                    RuntimeAdmissionOutcome::RejectedNotReady,
                    reason_codes::REQUEST_NOT_READY,
                );
            }
            RuntimeAdmissionDecision::admitted()
        }
    }
}

fn validate_ascii_token(
    field: &'static str,
    value: &str,
) -> Result<(), RuntimeRequestFoundationError> {
    if value.trim().is_empty() {
        return Err(RuntimeRequestFoundationError::invalid_envelope(&format!(
            "{field} must not be empty"
        )));
    }
    if value.len() > MAX_TOKEN_LEN {
        return Err(RuntimeRequestFoundationError::invalid_envelope(&format!(
            "{field} exceeds max length"
        )));
    }
    if !value.is_ascii() {
        return Err(RuntimeRequestFoundationError::invalid_envelope(&format!(
            "{field} must be ASCII"
        )));
    }
    Ok(())
}

fn validate_route_path(path: &str) -> Result<(), RuntimeRequestFoundationError> {
    validate_ascii_token("runtime_route_key.path", path)?;
    if !path.starts_with('/') {
        return Err(RuntimeRequestFoundationError::scope_violation(
            "runtime route paths must start with '/'",
        ));
    }
    Ok(())
}

fn validate_route_definition(
    definition: &RuntimeRouteDefinition,
) -> Result<(), RuntimeRequestFoundationError> {
    validate_route_path(&definition.key.path)?;
    if definition.key.path.starts_with("/v1/") {
        return Err(RuntimeRequestFoundationError::scope_violation(
            "Slice 1B may not register canonical Section 03 ingress routes",
        ));
    }
    if definition.key.path.contains("/session") {
        return Err(RuntimeRequestFoundationError::scope_violation(
            "Slice 1B may not register Section 02 session routes",
        ));
    }

    let health_classes = matches!(
        definition.request_class,
        RuntimeRequestClass::Health | RuntimeRequestClass::Startup
    );
    if health_classes && !definition.required_middleware.is_empty() {
        return Err(RuntimeRequestFoundationError::invalid_route_middleware(
            "health and startup routes must remain free of Slice 1B system-route middleware",
        ));
    }

    if definition.handler == RuntimeRouteHandlerKind::FoundationStatus {
        let required = BTreeSet::from([
            RuntimeRouteMiddlewareKind::EnvelopeFoundation,
            RuntimeRouteMiddlewareKind::RequestSecurity,
            RuntimeRouteMiddlewareKind::AdmissionControl,
            RuntimeRouteMiddlewareKind::FeatureFlags,
            RuntimeRouteMiddlewareKind::InvariantValidation,
        ]);
        if definition.required_middleware != required {
            return Err(RuntimeRequestFoundationError::invalid_route_middleware(
                "foundation status route must carry the complete Slice 1B middleware foundation set",
            ));
        }
        if definition.admission_policy != RuntimeAdmissionPolicy::RequireReady {
            return Err(RuntimeRequestFoundationError::invalid_route_middleware(
                "foundation status route must be readiness-gated",
            ));
        }
    }

    if definition.request_class == RuntimeRequestClass::System
        && !definition
            .required_middleware
            .contains(&RuntimeRouteMiddlewareKind::InvariantValidation)
    {
        return Err(RuntimeRequestFoundationError::invalid_route_middleware(
            "system routes must carry invariant validation middleware",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    use selene_kernel_contracts::provider_secrets::ProviderSecretId;

    use crate::runtime_bootstrap::{
        RuntimeBootstrapConfig, RuntimeBootstrapErrorKind, RuntimeSecretValue,
        RuntimeSecretsProvider, RuntimeServiceContainer,
    };

    #[derive(Debug, Default)]
    struct FixedClock {
        now_ms: Cell<i64>,
    }

    impl FixedClock {
        fn new(start_ms: i64) -> Self {
            Self {
                now_ms: Cell::new(start_ms),
            }
        }
    }

    impl RuntimeClock for FixedClock {
        fn now_unix_ms(&self) -> i64 {
            let current = self.now_ms.get();
            self.now_ms.set(current + 1);
            current
        }
    }

    #[derive(Debug, Default, Clone)]
    struct StaticSecretsProvider {
        secrets: BTreeMap<ProviderSecretId, RuntimeSecretValue>,
    }

    impl StaticSecretsProvider {
        fn with_secret(
            mut self,
            secret_id: ProviderSecretId,
            value: &str,
        ) -> Result<Self, RuntimeBootstrapError> {
            self.secrets
                .insert(secret_id, RuntimeSecretValue::new(value.to_string())?);
            Ok(self)
        }
    }

    impl RuntimeSecretsProvider for StaticSecretsProvider {
        fn get_secret(&self, secret_id: ProviderSecretId) -> Option<RuntimeSecretValue> {
            self.secrets.get(&secret_id).cloned()
        }
    }

    fn build_metadata() -> RuntimeBuildMetadata {
        RuntimeBuildMetadata {
            node_id: "node-a".to_string(),
            runtime_instance_identity: "instance-a".to_string(),
            environment_identity: "test".to_string(),
            build_version: "build-1".to_string(),
            git_commit: "abcdef".to_string(),
        }
    }

    fn bootstrap_config(required_secrets: Vec<ProviderSecretId>) -> RuntimeBootstrapConfig {
        RuntimeBootstrapConfig {
            service_name: "selene_runtime".to_string(),
            shutdown_grace_period_ms: 5_000,
            required_secrets,
            build_metadata: build_metadata(),
        }
    }

    fn request_foundation_config() -> RuntimeRequestFoundationConfig {
        RuntimeRequestFoundationConfig {
            service_name: "selene_runtime".to_string(),
            build_metadata: build_metadata(),
            max_clock_skew_ms: 30_000,
        }
    }

    fn ready_runtime() -> RuntimeProcess<FixedClock, StaticSecretsProvider> {
        let clock = FixedClock::new(1_000);
        let secrets = StaticSecretsProvider::default()
            .with_secret(ProviderSecretId::OpenAIApiKey, "secret")
            .expect("secret should be valid");
        let services =
            RuntimeServiceContainer::with_startup_foundation(clock, secrets).expect("services");
        let mut runtime = RuntimeProcess::new(
            bootstrap_config(vec![ProviderSecretId::OpenAIApiKey]),
            services,
        );
        runtime
            .start()
            .expect("Slice 1A runtime should reach READY");
        runtime
    }

    fn degraded_runtime() -> RuntimeProcess<FixedClock, StaticSecretsProvider> {
        let clock = FixedClock::new(2_000);
        let services = RuntimeServiceContainer::with_startup_foundation(
            clock,
            StaticSecretsProvider::default(),
        )
        .expect("services");
        let mut runtime = RuntimeProcess::new(
            bootstrap_config(vec![ProviderSecretId::OpenAIApiKey]),
            services,
        );
        let error = runtime
            .start()
            .expect_err("missing secret must fail closed");
        assert_eq!(error.kind, RuntimeBootstrapErrorKind::MissingRequiredSecret);
        runtime
    }

    fn status_envelope_input(timestamp_ms: i64) -> RuntimeRequestEnvelopeInput {
        RuntimeRequestEnvelopeInput {
            request_id: "req-1".to_string(),
            trace_id: "trace-1".to_string(),
            idempotency_key: "idem-1".to_string(),
            timestamp_ms,
            nonce: "nonce-1".to_string(),
            origin: RuntimeRequestOrigin {
                platform: AppPlatform::Ios,
                trigger: RuntimeEntryTrigger::Explicit,
            },
        }
    }

    #[test]
    fn slice_1b_route_registration_detects_duplicates_deterministically() {
        let mut router =
            RuntimeRouter::with_slice_1b_foundation_defaults(request_foundation_config())
                .expect("router");

        let duplicate = router
            .register_route(RuntimeRouteDefinition::liveness().expect("route"))
            .expect_err("duplicate route must fail closed");
        assert_eq!(duplicate.reason_code, reason_codes::ROUTE_DUPLICATE);
        assert_eq!(
            duplicate.kind,
            RuntimeRequestFoundationErrorKind::DuplicateRoute
        );
    }

    #[test]
    fn slice_1b_health_surfaces_remain_coherent_through_routing_substrate() {
        let mut router =
            RuntimeRouter::with_slice_1b_foundation_defaults(request_foundation_config())
                .expect("router");
        let mut runtime = ready_runtime();

        let live = router
            .dispatch(
                &runtime,
                RuntimeFoundationRequest::health(crate::runtime_bootstrap::LIVENESS_ENDPOINT_PATH)
                    .expect("request"),
            )
            .expect("live route should dispatch");
        let ready = router
            .dispatch(
                &runtime,
                RuntimeFoundationRequest::health(crate::runtime_bootstrap::READINESS_ENDPOINT_PATH)
                    .expect("request"),
            )
            .expect("ready route should dispatch");
        let startup = router
            .dispatch(
                &runtime,
                RuntimeFoundationRequest::health(crate::runtime_bootstrap::STARTUP_ENDPOINT_PATH)
                    .expect("request"),
            )
            .expect("startup route should dispatch");

        match live.response {
            RuntimeRouteResponse::Liveness(ref endpoint) => {
                assert_eq!(endpoint, &runtime.liveness_endpoint());
            }
            _ => panic!("expected liveness response"),
        }
        match ready.response {
            RuntimeRouteResponse::Readiness(ref endpoint) => {
                assert_eq!(endpoint, &runtime.readiness_endpoint());
            }
            _ => panic!("expected readiness response"),
        }
        match startup.response {
            RuntimeRouteResponse::Startup(ref endpoint) => {
                assert_eq!(endpoint, &runtime.startup_endpoint());
            }
            _ => panic!("expected startup response"),
        }

        runtime.begin_shutdown().expect("drain should succeed");
        let ready_after = router
            .dispatch(
                &runtime,
                RuntimeFoundationRequest::health(crate::runtime_bootstrap::READINESS_ENDPOINT_PATH)
                    .expect("request"),
            )
            .expect("ready route should still dispatch while draining");
        match ready_after.response {
            RuntimeRouteResponse::Readiness(endpoint) => {
                assert!(!endpoint.ready);
                assert_eq!(endpoint.state, RuntimeLifecycleState::Draining);
            }
            _ => panic!("expected readiness response"),
        }
    }

    #[test]
    fn slice_1b_runtime_envelope_base_assigns_immutable_core_fields() {
        let mut router =
            RuntimeRouter::with_slice_1b_foundation_defaults(request_foundation_config())
                .expect("router");
        let runtime = ready_runtime();

        let result = router
            .dispatch(
                &runtime,
                RuntimeFoundationRequest::system_status(
                    status_envelope_input(1_005),
                    RuntimeFeatureFlagOverrides::default(),
                )
                .expect("request"),
            )
            .expect("status route should dispatch");

        let envelope = result
            .envelope
            .expect("status route must create an envelope");
        assert_eq!(envelope.header().request_id, "req-1");
        assert_eq!(envelope.header().trace_id, "trace-1");
        assert_eq!(envelope.header().idempotency_key, "idem-1");
        assert_eq!(envelope.header().node_id, "node-a");
        assert_eq!(envelope.header().runtime_instance_identity, "instance-a");
        assert_eq!(envelope.header().build_version, "build-1");
        assert_eq!(envelope.header().git_commit, "abcdef");
        assert_eq!(envelope.request_class(), RuntimeRequestClass::System);
        assert_eq!(envelope.route_key().path, FOUNDATION_STATUS_ENDPOINT_PATH);
        assert_eq!(envelope.current_stage(), RuntimeRequestStage::Responded);
        assert_eq!(
            envelope.admission_state(),
            AdmissionState::ExecutionAdmitted
        );
        assert_eq!(envelope.execution_budget().total_budget_ms, 1_500);
        assert_eq!(
            envelope
                .stage_history()
                .iter()
                .map(|record| record.stage)
                .collect::<Vec<_>>(),
            vec![
                RuntimeRequestStage::Received,
                RuntimeRequestStage::EnvelopeCreated,
                RuntimeRequestStage::SecurityValidated,
                RuntimeRequestStage::Admitted,
                RuntimeRequestStage::Routed,
                RuntimeRequestStage::Responded,
            ]
        );
    }

    #[test]
    fn slice_1b_invalid_request_routing_prerequisites_fail_closed() {
        let mut router =
            RuntimeRouter::with_slice_1b_foundation_defaults(request_foundation_config())
                .expect("router");
        let runtime = ready_runtime();

        let invalid_route = RuntimeRouteDefinition {
            key: RuntimeRouteKey::new(RuntimeHttpMethod::Get, "/runtime/foundation/bad")
                .expect("key"),
            handler: RuntimeRouteHandlerKind::FoundationStatus,
            request_class: RuntimeRequestClass::System,
            admission_policy: RuntimeAdmissionPolicy::RequireReady,
            required_middleware: BTreeSet::from([
                RuntimeRouteMiddlewareKind::EnvelopeFoundation,
                RuntimeRouteMiddlewareKind::RequestSecurity,
            ]),
            description: "invalid route",
        };
        let invalid_route_error = router
            .register_route(invalid_route)
            .expect_err("missing middleware invariants must fail closed");
        assert_eq!(
            invalid_route_error.reason_code,
            reason_codes::ROUTE_MIDDLEWARE_INVALID
        );

        let missing_envelope_error = router
            .dispatch(
                &runtime,
                RuntimeFoundationRequest {
                    key: RuntimeRouteKey::new(
                        RuntimeHttpMethod::Get,
                        FOUNDATION_STATUS_ENDPOINT_PATH,
                    )
                    .expect("key"),
                    envelope_input: None,
                    overload_active: false,
                    feature_flag_overrides: RuntimeFeatureFlagOverrides::default(),
                },
            )
            .expect_err("status route must require an envelope");
        assert_eq!(
            missing_envelope_error.reason_code,
            reason_codes::ENVELOPE_REQUIRED
        );
    }

    #[test]
    fn slice_1b_request_classification_and_admission_are_deterministic() {
        let mut router =
            RuntimeRouter::with_slice_1b_foundation_defaults(request_foundation_config())
                .expect("router");
        let ready_runtime = ready_runtime();
        let degraded_runtime = degraded_runtime();

        let allowed = router
            .dispatch(
                &ready_runtime,
                RuntimeFoundationRequest::system_status(
                    status_envelope_input(1_005),
                    RuntimeFeatureFlagOverrides::default(),
                )
                .expect("request"),
            )
            .expect("ready runtime should admit system foundation route");
        assert!(allowed.admission.is_allowed());

        let rejected_not_ready = router
            .dispatch(
                &degraded_runtime,
                RuntimeFoundationRequest::system_status(
                    RuntimeRequestEnvelopeInput {
                        request_id: "req-not-ready".to_string(),
                        trace_id: "trace-not-ready".to_string(),
                        idempotency_key: "idem-not-ready".to_string(),
                        timestamp_ms: 2_005,
                        nonce: "nonce-not-ready".to_string(),
                        origin: RuntimeRequestOrigin {
                            platform: AppPlatform::Ios,
                            trigger: RuntimeEntryTrigger::Explicit,
                        },
                    },
                    RuntimeFeatureFlagOverrides::default(),
                )
                .expect("request"),
            )
            .expect_err("degraded runtime must reject ready-only route");
        assert_eq!(
            rejected_not_ready.reason_code,
            reason_codes::REQUEST_NOT_READY
        );

        let rejected_overloaded = router
            .dispatch(
                &ready_runtime,
                RuntimeFoundationRequest::system_status(
                    RuntimeRequestEnvelopeInput {
                        request_id: "req-2".to_string(),
                        trace_id: "trace-2".to_string(),
                        idempotency_key: "idem-2".to_string(),
                        timestamp_ms: 1_010,
                        nonce: "nonce-2".to_string(),
                        origin: RuntimeRequestOrigin {
                            platform: AppPlatform::Ios,
                            trigger: RuntimeEntryTrigger::Explicit,
                        },
                    },
                    RuntimeFeatureFlagOverrides::default(),
                )
                .expect("request")
                .with_overload_active(true),
            )
            .expect_err("overload hook must reject system route deterministically");
        assert_eq!(
            rejected_overloaded.reason_code,
            reason_codes::REQUEST_OVERLOADED
        );
    }

    #[test]
    fn slice_1b_feature_flag_snapshot_fallback_is_deterministic() {
        let mut router =
            RuntimeRouter::with_slice_1b_foundation_defaults(request_foundation_config())
                .expect("router");
        let runtime = ready_runtime();

        let base = router
            .dispatch(
                &runtime,
                RuntimeFoundationRequest::system_status(
                    status_envelope_input(1_005),
                    RuntimeFeatureFlagOverrides::default(),
                )
                .expect("request"),
            )
            .expect("status route should dispatch");

        let mut overrides = RuntimeFeatureFlagOverrides::default();
        overrides.set(RuntimeFeatureFlag::DiagnosticMode, true);
        let with_diag = router
            .dispatch(
                &runtime,
                RuntimeFoundationRequest::system_status(
                    RuntimeRequestEnvelopeInput {
                        request_id: "req-3".to_string(),
                        trace_id: "trace-3".to_string(),
                        idempotency_key: "idem-3".to_string(),
                        timestamp_ms: 1_020,
                        nonce: "nonce-3".to_string(),
                        origin: RuntimeRequestOrigin {
                            platform: AppPlatform::Ios,
                            trigger: RuntimeEntryTrigger::Explicit,
                        },
                    },
                    overrides,
                )
                .expect("request"),
            )
            .expect("status route should dispatch with overrides");

        match base.response {
            RuntimeRouteResponse::FoundationStatus(status) => {
                assert!(!status.diagnostic_mode_enabled);
                assert!(status.preflight_results.is_none());
                assert!(status.service_ids.is_none());
            }
            _ => panic!("expected foundation status response"),
        }
        match with_diag.response {
            RuntimeRouteResponse::FoundationStatus(status) => {
                assert!(status.diagnostic_mode_enabled);
                assert!(status.preflight_results.is_some());
                assert!(status.service_ids.is_some());
            }
            _ => panic!("expected foundation status response"),
        }
    }

    #[test]
    fn slice_1b_request_security_rejects_malformed_missing_and_replayed_inputs() {
        let mut router =
            RuntimeRouter::with_slice_1b_foundation_defaults(request_foundation_config())
                .expect("router");
        let runtime = ready_runtime();

        let invalid_ascii = router
            .dispatch(
                &runtime,
                RuntimeFoundationRequest::system_status(
                    RuntimeRequestEnvelopeInput {
                        request_id: "req-\u{2603}".to_string(),
                        trace_id: "trace-4".to_string(),
                        idempotency_key: "idem-4".to_string(),
                        timestamp_ms: 1_030,
                        nonce: "nonce-4".to_string(),
                        origin: RuntimeRequestOrigin {
                            platform: AppPlatform::Ios,
                            trigger: RuntimeEntryTrigger::Explicit,
                        },
                    },
                    RuntimeFeatureFlagOverrides::default(),
                )
                .expect("request"),
            )
            .expect_err("invalid ASCII request_id must fail closed");
        assert_eq!(invalid_ascii.reason_code, reason_codes::ENVELOPE_INVALID);

        let invalid_skew_timestamp = router
            .dispatch(
                &runtime,
                RuntimeFoundationRequest::system_status(
                    RuntimeRequestEnvelopeInput {
                        request_id: "req-5".to_string(),
                        trace_id: "trace-5".to_string(),
                        idempotency_key: "idem-5".to_string(),
                        timestamp_ms: 100_000,
                        nonce: "nonce-5".to_string(),
                        origin: RuntimeRequestOrigin {
                            platform: AppPlatform::Ios,
                            trigger: RuntimeEntryTrigger::Explicit,
                        },
                    },
                    RuntimeFeatureFlagOverrides::default(),
                )
                .expect("request"),
            )
            .expect_err("out-of-skew timestamp must fail closed");
        assert_eq!(
            invalid_skew_timestamp.reason_code,
            reason_codes::SECURITY_HEADER_INVALID
        );

        let first = RuntimeFoundationRequest::system_status(
            RuntimeRequestEnvelopeInput {
                request_id: "req-6".to_string(),
                trace_id: "trace-6".to_string(),
                idempotency_key: "idem-6".to_string(),
                timestamp_ms: 1_040,
                nonce: "nonce-6".to_string(),
                origin: RuntimeRequestOrigin {
                    platform: AppPlatform::Ios,
                    trigger: RuntimeEntryTrigger::Explicit,
                },
            },
            RuntimeFeatureFlagOverrides::default(),
        )
        .expect("request");
        router
            .dispatch(&runtime, first)
            .expect("first request should pass security");
        let replay = RuntimeFoundationRequest::system_status(
            RuntimeRequestEnvelopeInput {
                request_id: "req-6".to_string(),
                trace_id: "trace-6b".to_string(),
                idempotency_key: "idem-6b".to_string(),
                timestamp_ms: 1_041,
                nonce: "nonce-6".to_string(),
                origin: RuntimeRequestOrigin {
                    platform: AppPlatform::Ios,
                    trigger: RuntimeEntryTrigger::Explicit,
                },
            },
            RuntimeFeatureFlagOverrides::default(),
        )
        .expect("request");
        let replay_error = router
            .dispatch(&runtime, replay)
            .expect_err("duplicate request_id/nonce pair must be rejected");
        assert_eq!(replay_error.reason_code, reason_codes::REPLAY_REJECTED);
    }

    #[test]
    fn slice_1b_registers_foundation_services_in_slice_1a_container() {
        let clock = FixedClock::new(4_000);
        let services = StaticSecretsProvider::default();
        let mut container =
            RuntimeServiceContainer::with_startup_foundation(clock, services).expect("services");
        RuntimeRouter::register_slice_1b_foundation_services(&mut container)
            .expect("Slice 1B services should register");
        let service_ids = container.service_ids();
        assert!(service_ids.contains(&"runtime_router"));
        assert!(service_ids.contains(&"runtime_request_security_foundation"));
        assert!(service_ids.contains(&"runtime_feature_flag_registry"));
        assert!(service_ids.contains(&"runtime_event_bus"));
    }

    #[test]
    fn slice_1b_no_section_02_or_section_03_routes_can_be_registered() {
        let mut router =
            RuntimeRouter::with_slice_1b_foundation_defaults(request_foundation_config())
                .expect("router");
        let section_03 = RuntimeRouteDefinition {
            key: RuntimeRouteKey::new(RuntimeHttpMethod::Post, "/v1/voice/turn").expect("key"),
            handler: RuntimeRouteHandlerKind::FoundationStatus,
            request_class: RuntimeRequestClass::System,
            admission_policy: RuntimeAdmissionPolicy::RequireReady,
            required_middleware: BTreeSet::from([
                RuntimeRouteMiddlewareKind::EnvelopeFoundation,
                RuntimeRouteMiddlewareKind::RequestSecurity,
                RuntimeRouteMiddlewareKind::AdmissionControl,
                RuntimeRouteMiddlewareKind::FeatureFlags,
                RuntimeRouteMiddlewareKind::InvariantValidation,
            ]),
            description: "illegal section 03 route",
        };
        let section_03_error = router
            .register_route(section_03)
            .expect_err("Section 03 routes are out of scope");
        assert_eq!(
            section_03_error.reason_code,
            reason_codes::ROUTE_SCOPE_VIOLATION
        );

        let section_02 = RuntimeRouteDefinition {
            key: RuntimeRouteKey::new(RuntimeHttpMethod::Post, "/runtime/session/attach")
                .expect("key"),
            handler: RuntimeRouteHandlerKind::FoundationStatus,
            request_class: RuntimeRequestClass::System,
            admission_policy: RuntimeAdmissionPolicy::RequireReady,
            required_middleware: BTreeSet::from([
                RuntimeRouteMiddlewareKind::EnvelopeFoundation,
                RuntimeRouteMiddlewareKind::RequestSecurity,
                RuntimeRouteMiddlewareKind::AdmissionControl,
                RuntimeRouteMiddlewareKind::FeatureFlags,
                RuntimeRouteMiddlewareKind::InvariantValidation,
            ]),
            description: "illegal section 02 route",
        };
        let section_02_error = router
            .register_route(section_02)
            .expect_err("Section 02 routes are out of scope");
        assert_eq!(
            section_02_error.reason_code,
            reason_codes::ROUTE_SCOPE_VIOLATION
        );

        let manifest = router.capability_manifest(
            &router
                .feature_flags
                .snapshot(&RuntimeFeatureFlagOverrides::default())
                .expect("snapshot"),
        );
        assert_eq!(
            manifest.routes,
            vec![
                crate::runtime_bootstrap::LIVENESS_ENDPOINT_PATH.to_string(),
                crate::runtime_bootstrap::READINESS_ENDPOINT_PATH.to_string(),
                FOUNDATION_STATUS_ENDPOINT_PATH.to_string(),
                crate::runtime_bootstrap::STARTUP_ENDPOINT_PATH.to_string(),
            ]
        );
    }
}
