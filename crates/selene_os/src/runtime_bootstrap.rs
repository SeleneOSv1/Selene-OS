#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::panic::{catch_unwind, UnwindSafe};
use std::time::{SystemTime, UNIX_EPOCH};

use selene_kernel_contracts::provider_secrets::ProviderSecretId;

pub const LIVENESS_ENDPOINT_PATH: &str = "/livez";
pub const READINESS_ENDPOINT_PATH: &str = "/readyz";
pub const STARTUP_ENDPOINT_PATH: &str = "/startupz";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RuntimeLifecycleState {
    Starting,
    Warming,
    Ready,
    Degraded,
    Draining,
    ShuttingDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeStartupStatus {
    Pending,
    RunningPreflight,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimePreflightStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeBootstrapErrorKind {
    MissingConfiguration,
    InvalidConfiguration,
    MissingRequiredSecret,
    DependencyGraphInvalid,
    InvariantViolation,
    StartupBlocked,
    InvalidStateTransition,
    PanicIsolated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBootstrapError {
    pub kind: RuntimeBootstrapErrorKind,
    pub reason_code: &'static str,
    pub message: String,
}

impl RuntimeBootstrapError {
    fn missing_configuration(field: &str) -> Self {
        Self {
            kind: RuntimeBootstrapErrorKind::MissingConfiguration,
            reason_code: "runtime_missing_configuration",
            message: format!("missing required startup configuration: {field}"),
        }
    }

    fn invalid_configuration(field: &str, reason: &str) -> Self {
        Self {
            kind: RuntimeBootstrapErrorKind::InvalidConfiguration,
            reason_code: "runtime_invalid_configuration",
            message: format!("invalid startup configuration for {field}: {reason}"),
        }
    }

    fn missing_required_secret(secret_id: ProviderSecretId) -> Self {
        Self {
            kind: RuntimeBootstrapErrorKind::MissingRequiredSecret,
            reason_code: "runtime_missing_required_secret",
            message: format!(
                "startup blocked because required secret {} is unavailable",
                secret_id.as_str()
            ),
        }
    }

    fn dependency_graph_invalid(message: String) -> Self {
        Self {
            kind: RuntimeBootstrapErrorKind::DependencyGraphInvalid,
            reason_code: "runtime_dependency_graph_invalid",
            message,
        }
    }

    fn invariant_violation(invariant_id: &str, detail: &str) -> Self {
        Self {
            kind: RuntimeBootstrapErrorKind::InvariantViolation,
            reason_code: "runtime_startup_invariant_failed",
            message: format!("startup invariant {invariant_id} failed: {detail}"),
        }
    }

    fn invalid_state_transition(from: RuntimeLifecycleState, to: RuntimeLifecycleState) -> Self {
        Self {
            kind: RuntimeBootstrapErrorKind::InvalidStateTransition,
            reason_code: "runtime_invalid_state_transition",
            message: format!("runtime state transition {from:?} -> {to:?} is not allowed"),
        }
    }

    fn panic_isolated() -> Self {
        Self {
            kind: RuntimeBootstrapErrorKind::PanicIsolated,
            reason_code: "runtime_panic_isolated",
            message: "runtime operation panicked and was isolated".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBuildMetadata {
    pub node_id: String,
    pub runtime_instance_identity: String,
    pub environment_identity: String,
    pub build_version: String,
    pub git_commit: String,
}

impl RuntimeBuildMetadata {
    pub fn load(source: &impl RuntimeConfigSource) -> Result<Self, RuntimeBootstrapError> {
        Ok(Self {
            node_id: required_config(source, "SELENE_RUNTIME_NODE_ID")?,
            runtime_instance_identity: required_config(source, "SELENE_RUNTIME_INSTANCE_ID")?,
            environment_identity: required_config(source, "SELENE_ENVIRONMENT_IDENTITY")?,
            build_version: required_config(source, "SELENE_BUILD_VERSION")?,
            git_commit: required_config(source, "SELENE_GIT_COMMIT")?,
        })
    }

    fn validate(&self) -> Result<(), RuntimeBootstrapError> {
        require_non_empty("runtime_build_metadata.node_id", &self.node_id)?;
        require_non_empty(
            "runtime_build_metadata.runtime_instance_identity",
            &self.runtime_instance_identity,
        )?;
        require_non_empty(
            "runtime_build_metadata.environment_identity",
            &self.environment_identity,
        )?;
        require_non_empty("runtime_build_metadata.build_version", &self.build_version)?;
        require_non_empty("runtime_build_metadata.git_commit", &self.git_commit)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBootstrapConfig {
    pub service_name: String,
    pub shutdown_grace_period_ms: u64,
    pub required_secrets: Vec<ProviderSecretId>,
    pub build_metadata: RuntimeBuildMetadata,
}

impl RuntimeBootstrapConfig {
    pub fn validate(&self) -> Result<(), RuntimeBootstrapError> {
        require_non_empty("runtime_bootstrap_config.service_name", &self.service_name)?;
        if self.shutdown_grace_period_ms == 0 {
            return Err(RuntimeBootstrapError::invalid_configuration(
                "runtime_bootstrap_config.shutdown_grace_period_ms",
                "must be greater than zero",
            ));
        }
        let mut seen = BTreeSet::new();
        for secret_id in &self.required_secrets {
            if !seen.insert(*secret_id) {
                return Err(RuntimeBootstrapError::invalid_configuration(
                    "runtime_bootstrap_config.required_secrets",
                    "duplicate required secret ids are not allowed",
                ));
            }
        }
        self.build_metadata.validate()
    }
}

pub trait RuntimeConfigSource {
    fn get(&self, key: &str) -> Option<String>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ProcessEnvRuntimeConfigSource;

impl RuntimeConfigSource for ProcessEnvRuntimeConfigSource {
    fn get(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeServiceRegistration {
    pub service_id: String,
    pub dependencies: Vec<String>,
}

pub trait RuntimeClock {
    fn now_unix_ms(&self) -> i64;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemRuntimeClock;

impl RuntimeClock for SystemRuntimeClock {
    fn now_unix_ms(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis() as i64)
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSecretValue {
    value: String,
}

impl RuntimeSecretValue {
    pub fn new(value: String) -> Result<Self, RuntimeBootstrapError> {
        if value.trim().is_empty() {
            return Err(RuntimeBootstrapError::invalid_configuration(
                "runtime_secret_value",
                "secret values must not be empty",
            ));
        }
        Ok(Self { value })
    }

    pub fn expose_for_runtime(&self) -> &str {
        &self.value
    }
}

pub trait RuntimeSecretsProvider {
    fn get_secret(&self, secret_id: ProviderSecretId) -> Option<RuntimeSecretValue>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeInvariantHook {
    pub invariant_id: String,
    pub holds: bool,
    pub detail: String,
}

impl RuntimeInvariantHook {
    pub fn passing(invariant_id: impl Into<String>) -> Self {
        Self {
            invariant_id: invariant_id.into(),
            holds: true,
            detail: "ok".to_string(),
        }
    }

    pub fn failing(invariant_id: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            invariant_id: invariant_id.into(),
            holds: false,
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePreflightCheckResult {
    pub check_id: String,
    pub passed: bool,
    pub reason_code: Option<&'static str>,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLifecycleTransition {
    pub from: RuntimeLifecycleState,
    pub to: RuntimeLifecycleState,
    pub at_unix_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLogEvent {
    pub at_unix_ms: i64,
    pub level: &'static str,
    pub state: RuntimeLifecycleState,
    pub service_name: String,
    pub node_id: String,
    pub runtime_instance_identity: String,
    pub environment_identity: String,
    pub build_version: String,
    pub git_commit: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLivenessEndpoint {
    pub path: &'static str,
    pub live: bool,
    pub state: RuntimeLifecycleState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeReadinessEndpoint {
    pub path: &'static str,
    pub ready: bool,
    pub state: RuntimeLifecycleState,
    pub reason_code: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeStartupEndpoint {
    pub path: &'static str,
    pub complete: bool,
    pub succeeded: bool,
    pub startup_status: RuntimeStartupStatus,
    pub preflight_status: RuntimePreflightStatus,
    pub state: RuntimeLifecycleState,
    pub reason_code: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeStartupSummary {
    pub state: RuntimeLifecycleState,
    pub startup_status: RuntimeStartupStatus,
    pub preflight_status: RuntimePreflightStatus,
}

#[derive(Debug)]
pub struct RuntimeServiceContainer<C, S> {
    clock: C,
    secrets_provider: S,
    registrations: BTreeMap<String, RuntimeServiceRegistration>,
}

impl<C, S> RuntimeServiceContainer<C, S>
where
    C: RuntimeClock,
    S: RuntimeSecretsProvider,
{
    pub fn with_startup_foundation(
        clock: C,
        secrets_provider: S,
    ) -> Result<Self, RuntimeBootstrapError> {
        let mut container = Self {
            clock,
            secrets_provider,
            registrations: BTreeMap::new(),
        };
        container.register_service("runtime_clock", &[])?;
        container.register_service("secrets_provider", &[])?;
        container.register_service("config_loader", &[])?;
        container.register_service(
            "startup_preflight_runner",
            &["runtime_clock", "secrets_provider", "config_loader"],
        )?;
        container.register_service("health_endpoint_registry", &["startup_preflight_runner"])?;
        container.register_service("shutdown_coordinator", &["runtime_clock"])?;
        container.register_service("startup_logger", &["runtime_clock"])?;
        Ok(container)
    }

    pub fn register_service(
        &mut self,
        service_id: impl Into<String>,
        dependencies: &[&str],
    ) -> Result<(), RuntimeBootstrapError> {
        let service_id = service_id.into();
        require_non_empty("runtime_service_registration.service_id", &service_id)?;
        if self.registrations.contains_key(&service_id) {
            return Err(RuntimeBootstrapError::invalid_configuration(
                "runtime_service_registration.service_id",
                "duplicate service registration",
            ));
        }

        let mut normalized_dependencies = Vec::with_capacity(dependencies.len());
        for dependency in dependencies {
            require_non_empty("runtime_service_registration.dependencies", dependency)?;
            if *dependency == service_id {
                return Err(RuntimeBootstrapError::invalid_configuration(
                    "runtime_service_registration.dependencies",
                    "a service cannot depend on itself",
                ));
            }
            normalized_dependencies.push((*dependency).to_string());
        }

        self.registrations.insert(
            service_id.clone(),
            RuntimeServiceRegistration {
                service_id,
                dependencies: normalized_dependencies,
            },
        );
        Ok(())
    }

    pub fn clock(&self) -> &C {
        &self.clock
    }

    pub fn service_ids(&self) -> Vec<&str> {
        self.registrations.keys().map(String::as_str).collect()
    }

    fn validate_dependency_graph(&self) -> Result<(), RuntimeBootstrapError> {
        for registration in self.registrations.values() {
            for dependency in &registration.dependencies {
                if !self.registrations.contains_key(dependency) {
                    return Err(RuntimeBootstrapError::dependency_graph_invalid(format!(
                        "service {} depends on missing service {}",
                        registration.service_id, dependency
                    )));
                }
            }
        }

        let mut permanent = BTreeSet::new();
        let mut visiting = BTreeSet::new();
        for service_id in self.registrations.keys() {
            self.visit_service(service_id, &mut visiting, &mut permanent)?;
        }
        Ok(())
    }

    fn visit_service(
        &self,
        service_id: &str,
        visiting: &mut BTreeSet<String>,
        permanent: &mut BTreeSet<String>,
    ) -> Result<(), RuntimeBootstrapError> {
        if permanent.contains(service_id) {
            return Ok(());
        }
        if !visiting.insert(service_id.to_string()) {
            return Err(RuntimeBootstrapError::dependency_graph_invalid(format!(
                "cyclic startup dependency detected at service {}",
                service_id
            )));
        }

        let registration = self.registrations.get(service_id).ok_or_else(|| {
            RuntimeBootstrapError::dependency_graph_invalid(format!(
                "missing registration while traversing dependency graph: {}",
                service_id
            ))
        })?;

        for dependency in &registration.dependencies {
            self.visit_service(dependency, visiting, permanent)?;
        }

        visiting.remove(service_id);
        permanent.insert(service_id.to_string());
        Ok(())
    }
}

#[derive(Debug)]
pub struct RuntimeProcess<C, S> {
    config: RuntimeBootstrapConfig,
    services: RuntimeServiceContainer<C, S>,
    state: RuntimeLifecycleState,
    startup_status: RuntimeStartupStatus,
    preflight_status: RuntimePreflightStatus,
    preflight_results: Vec<RuntimePreflightCheckResult>,
    invariant_hooks: Vec<RuntimeInvariantHook>,
    transition_history: Vec<RuntimeLifecycleTransition>,
    log_events: Vec<RuntimeLogEvent>,
    last_error: Option<RuntimeBootstrapError>,
}

impl<C, S> RuntimeProcess<C, S>
where
    C: RuntimeClock,
    S: RuntimeSecretsProvider,
{
    pub fn new(config: RuntimeBootstrapConfig, services: RuntimeServiceContainer<C, S>) -> Self {
        Self {
            config,
            services,
            state: RuntimeLifecycleState::Starting,
            startup_status: RuntimeStartupStatus::Pending,
            preflight_status: RuntimePreflightStatus::Pending,
            preflight_results: Vec::new(),
            invariant_hooks: Vec::new(),
            transition_history: Vec::new(),
            log_events: Vec::new(),
            last_error: None,
        }
    }

    pub fn register_invariant_hook(&mut self, hook: RuntimeInvariantHook) {
        self.invariant_hooks.push(hook);
    }

    pub fn state(&self) -> RuntimeLifecycleState {
        self.state
    }

    pub fn transition_history(&self) -> &[RuntimeLifecycleTransition] {
        &self.transition_history
    }

    pub fn log_events(&self) -> &[RuntimeLogEvent] {
        &self.log_events
    }

    pub fn preflight_results(&self) -> &[RuntimePreflightCheckResult] {
        &self.preflight_results
    }

    pub fn last_error(&self) -> Option<&RuntimeBootstrapError> {
        self.last_error.as_ref()
    }

    pub fn clock(&self) -> &C {
        self.services.clock()
    }

    pub fn service_ids(&self) -> Vec<&str> {
        self.services.service_ids()
    }

    pub fn start(&mut self) -> Result<RuntimeStartupSummary, RuntimeBootstrapError> {
        self.startup_status = RuntimeStartupStatus::RunningPreflight;
        self.preflight_status = RuntimePreflightStatus::Running;
        self.log("INFO", "runtime bootstrap starting");
        self.transition_to(RuntimeLifecycleState::Warming)?;

        if let Err(error) = self.run_preflight() {
            self.preflight_status = RuntimePreflightStatus::Failed;
            self.startup_status = RuntimeStartupStatus::Failed;
            self.last_error = Some(error.clone());
            self.transition_to(RuntimeLifecycleState::Degraded)?;
            self.log(
                "ERROR",
                &format!("runtime startup failed: {}", error.message),
            );
            return Err(error);
        }

        self.preflight_status = RuntimePreflightStatus::Succeeded;
        self.startup_status = RuntimeStartupStatus::Succeeded;
        self.transition_to(RuntimeLifecycleState::Ready)?;
        self.log("INFO", "runtime startup reached READY after preflight");
        Ok(RuntimeStartupSummary {
            state: self.state,
            startup_status: self.startup_status,
            preflight_status: self.preflight_status,
        })
    }

    pub fn begin_shutdown(&mut self) -> Result<(), RuntimeBootstrapError> {
        self.transition_to(RuntimeLifecycleState::Draining)?;
        self.log(
            "WARN",
            "runtime readiness disabled; draining before shutdown",
        );
        Ok(())
    }

    pub fn finish_shutdown(&mut self) -> Result<(), RuntimeBootstrapError> {
        self.transition_to(RuntimeLifecycleState::ShuttingDown)?;
        self.log("WARN", "runtime entered shutting down state");
        Ok(())
    }

    pub fn run_panic_isolated<T, F>(&mut self, operation: F) -> Result<T, RuntimeBootstrapError>
    where
        F: FnOnce() -> T + UnwindSafe,
    {
        match catch_unwind(operation) {
            Ok(value) => Ok(value),
            Err(_) => {
                let error = RuntimeBootstrapError::panic_isolated();
                self.last_error = Some(error.clone());
                if matches!(
                    self.state,
                    RuntimeLifecycleState::Ready | RuntimeLifecycleState::Warming
                ) {
                    self.transition_to(RuntimeLifecycleState::Degraded)?;
                }
                self.log("ERROR", "runtime operation panic isolated");
                Err(error)
            }
        }
    }

    pub fn liveness_endpoint(&self) -> RuntimeLivenessEndpoint {
        RuntimeLivenessEndpoint {
            path: LIVENESS_ENDPOINT_PATH,
            live: true,
            state: self.state,
        }
    }

    pub fn readiness_endpoint(&self) -> RuntimeReadinessEndpoint {
        RuntimeReadinessEndpoint {
            path: READINESS_ENDPOINT_PATH,
            ready: self.state == RuntimeLifecycleState::Ready
                && self.startup_status == RuntimeStartupStatus::Succeeded
                && self.preflight_status == RuntimePreflightStatus::Succeeded,
            state: self.state,
            reason_code: self.last_error.as_ref().map(|error| error.reason_code),
        }
    }

    pub fn startup_endpoint(&self) -> RuntimeStartupEndpoint {
        RuntimeStartupEndpoint {
            path: STARTUP_ENDPOINT_PATH,
            complete: matches!(
                self.startup_status,
                RuntimeStartupStatus::Succeeded | RuntimeStartupStatus::Failed
            ),
            succeeded: self.startup_status == RuntimeStartupStatus::Succeeded,
            startup_status: self.startup_status,
            preflight_status: self.preflight_status,
            state: self.state,
            reason_code: self.last_error.as_ref().map(|error| error.reason_code),
        }
    }

    fn run_preflight(&mut self) -> Result<(), RuntimeBootstrapError> {
        self.preflight_results.clear();

        self.config.validate().map_err(|error| {
            self.record_failure("config_validation", &error);
            error
        })?;
        self.record_success(
            "config_validation",
            "startup configuration passed validation",
        );

        self.services.validate_dependency_graph().map_err(|error| {
            self.record_failure("dependency_graph_validation", &error);
            error
        })?;
        self.record_success(
            "dependency_graph_validation",
            "startup dependency graph is acyclic and complete",
        );

        for secret_id in self.config.required_secrets.clone() {
            match self.services.secrets_provider.get_secret(secret_id) {
                Some(secret) if !secret.expose_for_runtime().trim().is_empty() => self
                    .record_success(
                        &format!("required_secret_presence:{}", secret_id.as_str()),
                        &format!("required secret {} is available", secret_id.as_str()),
                    ),
                _ => {
                    let error = RuntimeBootstrapError::missing_required_secret(secret_id);
                    self.record_failure(
                        &format!("required_secret_presence:{}", secret_id.as_str()),
                        &error,
                    );
                    return Err(error);
                }
            }
        }

        for hook in self.invariant_hooks.clone() {
            if let Err(error) =
                require_non_empty("runtime_invariant_hook.invariant_id", &hook.invariant_id)
            {
                self.record_failure("startup_invariant_validation", &error);
                return Err(error);
            }
            if !hook.holds {
                let error =
                    RuntimeBootstrapError::invariant_violation(&hook.invariant_id, &hook.detail);
                self.record_failure(&format!("startup_invariant:{}", hook.invariant_id), &error);
                return Err(error);
            }
            self.record_success(
                &format!("startup_invariant:{}", hook.invariant_id),
                &hook.detail,
            );
        }

        Ok(())
    }

    fn transition_to(
        &mut self,
        next_state: RuntimeLifecycleState,
    ) -> Result<(), RuntimeBootstrapError> {
        if self.state == next_state {
            return Ok(());
        }
        if !state_transition_allowed(self.state, next_state) {
            return Err(RuntimeBootstrapError::invalid_state_transition(
                self.state, next_state,
            ));
        }
        let transition = RuntimeLifecycleTransition {
            from: self.state,
            to: next_state,
            at_unix_ms: self.services.clock.now_unix_ms(),
        };
        self.transition_history.push(transition);
        self.state = next_state;
        Ok(())
    }

    fn log(&mut self, level: &'static str, message: &str) {
        self.log_events.push(RuntimeLogEvent {
            at_unix_ms: self.services.clock.now_unix_ms(),
            level,
            state: self.state,
            service_name: self.config.service_name.clone(),
            node_id: self.config.build_metadata.node_id.clone(),
            runtime_instance_identity: self.config.build_metadata.runtime_instance_identity.clone(),
            environment_identity: self.config.build_metadata.environment_identity.clone(),
            build_version: self.config.build_metadata.build_version.clone(),
            git_commit: self.config.build_metadata.git_commit.clone(),
            message: message.to_string(),
        });
    }

    fn record_success(&mut self, check_id: &str, detail: &str) {
        self.preflight_results.push(RuntimePreflightCheckResult {
            check_id: check_id.to_string(),
            passed: true,
            reason_code: None,
            detail: detail.to_string(),
        });
    }

    fn record_failure(&mut self, check_id: &str, error: &RuntimeBootstrapError) {
        self.preflight_results.push(RuntimePreflightCheckResult {
            check_id: check_id.to_string(),
            passed: false,
            reason_code: Some(error.reason_code),
            detail: error.message.clone(),
        });
    }
}

fn required_config(
    source: &impl RuntimeConfigSource,
    key: &str,
) -> Result<String, RuntimeBootstrapError> {
    let value = source
        .get(key)
        .ok_or_else(|| RuntimeBootstrapError::missing_configuration(key))?;
    require_non_empty(key, &value)?;
    Ok(value)
}

fn require_non_empty(field: &str, value: &str) -> Result<(), RuntimeBootstrapError> {
    if value.trim().is_empty() {
        return Err(RuntimeBootstrapError::invalid_configuration(
            field,
            "must not be empty",
        ));
    }
    Ok(())
}

fn state_transition_allowed(current: RuntimeLifecycleState, next: RuntimeLifecycleState) -> bool {
    matches!(
        (current, next),
        (
            RuntimeLifecycleState::Starting,
            RuntimeLifecycleState::Warming
        ) | (
            RuntimeLifecycleState::Starting,
            RuntimeLifecycleState::Degraded
        ) | (
            RuntimeLifecycleState::Starting,
            RuntimeLifecycleState::Draining
        ) | (RuntimeLifecycleState::Warming, RuntimeLifecycleState::Ready)
            | (
                RuntimeLifecycleState::Warming,
                RuntimeLifecycleState::Degraded
            )
            | (
                RuntimeLifecycleState::Warming,
                RuntimeLifecycleState::Draining
            )
            | (
                RuntimeLifecycleState::Ready,
                RuntimeLifecycleState::Degraded
            )
            | (
                RuntimeLifecycleState::Ready,
                RuntimeLifecycleState::Draining
            )
            | (
                RuntimeLifecycleState::Degraded,
                RuntimeLifecycleState::Warming
            )
            | (
                RuntimeLifecycleState::Degraded,
                RuntimeLifecycleState::Draining
            )
            | (
                RuntimeLifecycleState::Draining,
                RuntimeLifecycleState::ShuttingDown
            )
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

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

    #[derive(Debug, Default)]
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

    #[derive(Debug, Default)]
    struct MapConfigSource {
        values: BTreeMap<String, String>,
    }

    impl MapConfigSource {
        fn with(mut self, key: &str, value: &str) -> Self {
            self.values.insert(key.to_string(), value.to_string());
            self
        }
    }

    impl RuntimeConfigSource for MapConfigSource {
        fn get(&self, key: &str) -> Option<String> {
            self.values.get(key).cloned()
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

    fn config(required_secrets: Vec<ProviderSecretId>) -> RuntimeBootstrapConfig {
        RuntimeBootstrapConfig {
            service_name: "selene_runtime".to_string(),
            shutdown_grace_period_ms: 5000,
            required_secrets,
            build_metadata: build_metadata(),
        }
    }

    #[test]
    fn slice_1a_runtime_reaches_ready_only_after_preflight_succeeds() {
        let clock = FixedClock::new(100);
        let secrets = StaticSecretsProvider::default()
            .with_secret(ProviderSecretId::OpenAIApiKey, "secret")
            .expect("secret should be valid");
        let services =
            RuntimeServiceContainer::with_startup_foundation(clock, secrets).expect("services");
        let mut runtime =
            RuntimeProcess::new(config(vec![ProviderSecretId::OpenAIApiKey]), services);

        assert_eq!(runtime.state(), RuntimeLifecycleState::Starting);
        assert!(!runtime.readiness_endpoint().ready);
        assert!(!runtime.startup_endpoint().complete);

        let summary = runtime.start().expect("startup should succeed");
        assert_eq!(summary.state, RuntimeLifecycleState::Ready);
        assert_eq!(runtime.state(), RuntimeLifecycleState::Ready);
        assert_eq!(
            runtime.transition_history(),
            &[
                RuntimeLifecycleTransition {
                    from: RuntimeLifecycleState::Starting,
                    to: RuntimeLifecycleState::Warming,
                    at_unix_ms: 101,
                },
                RuntimeLifecycleTransition {
                    from: RuntimeLifecycleState::Warming,
                    to: RuntimeLifecycleState::Ready,
                    at_unix_ms: 102,
                }
            ]
        );
        assert!(runtime.readiness_endpoint().ready);
        assert!(runtime.liveness_endpoint().live);
        assert!(runtime.startup_endpoint().complete);
        assert!(runtime.startup_endpoint().succeeded);
        assert_eq!(runtime.log_events().len(), 2);
        assert_eq!(runtime.log_events()[0].node_id, "node-a");
        assert_eq!(runtime.log_events()[0].build_version, "build-1");
    }

    #[test]
    fn slice_1a_failed_preflight_blocks_ready_and_reports_health_posture() {
        let clock = FixedClock::new(200);
        let services = RuntimeServiceContainer::with_startup_foundation(
            clock,
            StaticSecretsProvider::default(),
        )
        .expect("services");
        let mut runtime =
            RuntimeProcess::new(config(vec![ProviderSecretId::OpenAIApiKey]), services);

        let error = runtime.start().expect_err("startup must fail closed");
        assert_eq!(error.reason_code, "runtime_missing_required_secret");
        assert_eq!(runtime.state(), RuntimeLifecycleState::Degraded);
        assert!(!runtime.readiness_endpoint().ready);
        assert!(runtime.liveness_endpoint().live);
        assert!(runtime.startup_endpoint().complete);
        assert!(!runtime.startup_endpoint().succeeded);
        assert_eq!(
            runtime.startup_endpoint().reason_code,
            Some("runtime_missing_required_secret")
        );
    }

    #[test]
    fn slice_1a_liveness_is_independent_from_readiness_posture() {
        let clock = FixedClock::new(300);
        let services = RuntimeServiceContainer::with_startup_foundation(
            clock,
            StaticSecretsProvider::default(),
        )
        .expect("services");
        let mut runtime =
            RuntimeProcess::new(config(vec![ProviderSecretId::OpenAIApiKey]), services);

        let _ = runtime.start().expect_err("startup must fail closed");
        let liveness = runtime.liveness_endpoint();
        let readiness = runtime.readiness_endpoint();
        assert!(liveness.live);
        assert!(!readiness.ready);
        assert_eq!(liveness.state, RuntimeLifecycleState::Degraded);
        assert_eq!(readiness.state, RuntimeLifecycleState::Degraded);
    }

    #[test]
    fn slice_1a_startup_endpoint_tracks_preflight_status() {
        let clock = FixedClock::new(400);
        let secrets = StaticSecretsProvider::default()
            .with_secret(ProviderSecretId::OpenAIApiKey, "secret")
            .expect("secret should be valid");
        let services =
            RuntimeServiceContainer::with_startup_foundation(clock, secrets).expect("services");
        let mut runtime =
            RuntimeProcess::new(config(vec![ProviderSecretId::OpenAIApiKey]), services);

        let initial = runtime.startup_endpoint();
        assert_eq!(initial.startup_status, RuntimeStartupStatus::Pending);
        assert_eq!(initial.preflight_status, RuntimePreflightStatus::Pending);
        assert!(!initial.complete);

        runtime.start().expect("startup should succeed");

        let startup = runtime.startup_endpoint();
        assert_eq!(startup.startup_status, RuntimeStartupStatus::Succeeded);
        assert_eq!(startup.preflight_status, RuntimePreflightStatus::Succeeded);
        assert!(startup.complete);
        assert!(startup.succeeded);
    }

    #[test]
    fn slice_1a_shutdown_disables_readiness_before_termination_path() {
        let clock = FixedClock::new(500);
        let secrets = StaticSecretsProvider::default()
            .with_secret(ProviderSecretId::OpenAIApiKey, "secret")
            .expect("secret should be valid");
        let services =
            RuntimeServiceContainer::with_startup_foundation(clock, secrets).expect("services");
        let mut runtime =
            RuntimeProcess::new(config(vec![ProviderSecretId::OpenAIApiKey]), services);

        runtime.start().expect("startup should succeed");
        assert!(runtime.readiness_endpoint().ready);

        runtime.begin_shutdown().expect("drain should succeed");
        assert_eq!(runtime.state(), RuntimeLifecycleState::Draining);
        assert!(!runtime.readiness_endpoint().ready);
        assert!(runtime.liveness_endpoint().live);

        runtime.finish_shutdown().expect("shutdown should succeed");
        assert_eq!(runtime.state(), RuntimeLifecycleState::ShuttingDown);
        assert!(!runtime.readiness_endpoint().ready);
    }

    #[test]
    fn slice_1a_runtime_clock_service_is_injectable() {
        let clock = FixedClock::new(600);
        let services = RuntimeServiceContainer::with_startup_foundation(
            clock,
            StaticSecretsProvider::default(),
        )
        .expect("services");
        let runtime = RuntimeProcess::new(config(Vec::new()), services);

        assert!(runtime.service_ids().contains(&"runtime_clock"));
        assert_eq!(runtime.clock().now_unix_ms(), 600);
        assert_eq!(runtime.clock().now_unix_ms(), 601);
    }

    #[test]
    fn slice_1a_global_error_model_maps_failures_deterministically() {
        let missing_config = RuntimeBuildMetadata::load(
            &MapConfigSource::default()
                .with("SELENE_RUNTIME_NODE_ID", "node")
                .with("SELENE_RUNTIME_INSTANCE_ID", "instance"),
        )
        .expect_err("build metadata should fail closed");
        assert_eq!(missing_config.reason_code, "runtime_missing_configuration");

        let clock = FixedClock::new(700);
        let services = RuntimeServiceContainer::with_startup_foundation(
            clock,
            StaticSecretsProvider::default(),
        )
        .expect("services");
        let mut runtime =
            RuntimeProcess::new(config(vec![ProviderSecretId::OpenAIApiKey]), services);
        let missing_secret = runtime
            .start()
            .expect_err("secret failure must be explicit");
        assert_eq!(
            missing_secret.reason_code,
            "runtime_missing_required_secret"
        );

        let clock = FixedClock::new(800);
        let secrets = StaticSecretsProvider::default()
            .with_secret(ProviderSecretId::OpenAIApiKey, "secret")
            .expect("secret should be valid");
        let services =
            RuntimeServiceContainer::with_startup_foundation(clock, secrets).expect("services");
        let mut runtime =
            RuntimeProcess::new(config(vec![ProviderSecretId::OpenAIApiKey]), services);
        runtime.start().expect("startup should succeed");
        let panic_error = runtime
            .run_panic_isolated::<(), _>(|| panic!("panic must be isolated"))
            .expect_err("panic must be mapped");
        assert_eq!(panic_error.reason_code, "runtime_panic_isolated");
    }

    #[test]
    fn slice_1a_required_secret_and_config_failures_are_fail_closed() {
        let clock = FixedClock::new(900);
        let services = RuntimeServiceContainer::with_startup_foundation(
            clock,
            StaticSecretsProvider::default(),
        )
        .expect("services");
        let mut invalid_runtime = RuntimeProcess::new(
            RuntimeBootstrapConfig {
                service_name: "".to_string(),
                shutdown_grace_period_ms: 5000,
                required_secrets: Vec::new(),
                build_metadata: build_metadata(),
            },
            services,
        );
        let config_error = invalid_runtime
            .start()
            .expect_err("invalid config must fail closed");
        assert_eq!(config_error.reason_code, "runtime_invalid_configuration");
        assert_eq!(invalid_runtime.state(), RuntimeLifecycleState::Degraded);
        assert!(!invalid_runtime.readiness_endpoint().ready);

        let clock = FixedClock::new(1000);
        let services = RuntimeServiceContainer::with_startup_foundation(
            clock,
            StaticSecretsProvider::default(),
        )
        .expect("services");
        let mut secret_runtime =
            RuntimeProcess::new(config(vec![ProviderSecretId::OpenAIApiKey]), services);
        let secret_error = secret_runtime
            .start()
            .expect_err("missing secret must fail closed");
        assert_eq!(secret_error.reason_code, "runtime_missing_required_secret");
        assert_eq!(secret_runtime.state(), RuntimeLifecycleState::Degraded);
        assert!(!secret_runtime.readiness_endpoint().ready);
    }

    #[test]
    fn slice_1a_startup_invariants_and_dependency_graph_fail_closed() {
        let clock = FixedClock::new(1100);
        let secrets = StaticSecretsProvider::default()
            .with_secret(ProviderSecretId::OpenAIApiKey, "secret")
            .expect("secret should be valid");
        let mut services =
            RuntimeServiceContainer::with_startup_foundation(clock, secrets).expect("services");
        services
            .register_service("bad_service", &["missing_dep"])
            .expect("registration shape itself is valid");
        let mut runtime =
            RuntimeProcess::new(config(vec![ProviderSecretId::OpenAIApiKey]), services);
        let graph_error = runtime
            .start()
            .expect_err("missing dependency must fail closed");
        assert_eq!(graph_error.reason_code, "runtime_dependency_graph_invalid");

        let clock = FixedClock::new(1200);
        let secrets = StaticSecretsProvider::default()
            .with_secret(ProviderSecretId::OpenAIApiKey, "secret")
            .expect("secret should be valid");
        let services =
            RuntimeServiceContainer::with_startup_foundation(clock, secrets).expect("services");
        let mut runtime =
            RuntimeProcess::new(config(vec![ProviderSecretId::OpenAIApiKey]), services);
        runtime.register_invariant_hook(RuntimeInvariantHook::failing(
            "runtime_identity_present",
            "identity manifest not loaded",
        ));
        let invariant_error = runtime
            .start()
            .expect_err("invariant failure must block READY");
        assert_eq!(
            invariant_error.reason_code,
            "runtime_startup_invariant_failed"
        );
        assert_eq!(runtime.state(), RuntimeLifecycleState::Degraded);
    }
}
