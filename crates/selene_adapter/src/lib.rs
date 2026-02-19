#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use selene_engines::ph1_voice_id::VoiceIdObservation as EngineVoiceIdObservation;
use selene_engines::ph1pattern::{Ph1PatternConfig as EnginePatternConfig, Ph1PatternRuntime};
use selene_engines::ph1rll::{Ph1RllConfig as EngineRllConfig, Ph1RllRuntime};
use selene_kernel_contracts::ph1_voice_id::{
    DeviceTrustLevel, Ph1VoiceIdRequest, Ph1VoiceIdResponse, UserId,
};
use selene_kernel_contracts::ph1art::{
    ArtifactScopeType, ArtifactStatus, ArtifactType, ArtifactVersion,
};
use selene_kernel_contracts::ph1j::{CorrelationId, DeviceId, TurnId};
use selene_kernel_contracts::ph1k::{
    AudioDeviceId, AudioFormat, AudioStreamId, AudioStreamKind, AudioStreamRef, ChannelCount,
    Confidence, FrameDurationMs, SampleFormat, SampleRateHz, SpeechLikeness, VadEvent,
};
use selene_kernel_contracts::ph1l::{NextAllowedActions, SessionId, SessionSnapshot};
use selene_kernel_contracts::ph1link::AppPlatform;
use selene_kernel_contracts::ph1os::{OsOutcomeActionClass, OsOutcomeUtilizationEntry};
use selene_kernel_contracts::ph1pattern::{Ph1PatternRequest, Ph1PatternResponse};
use selene_kernel_contracts::ph1rll::{Ph1RllRequest, Ph1RllResponse};
use selene_kernel_contracts::{MonotonicTimeNs, SchemaVersion, SessionState};
use selene_os::app_ingress::{AppServerIngressRuntime, AppVoiceIngressRequest};
use selene_os::device_artifact_sync::DeviceArtifactSyncWorkerPassMetrics;
use selene_os::ph1_voice_id::{
    Ph1VoiceIdLiveConfig, VoiceIdContractMigrationConfig, VoiceIdentityEmbeddingGateGovernedConfig,
    VoiceIdentityEmbeddingGateProfile, VoiceIdentityEmbeddingGateProfiles,
};
use selene_os::ph1builder::{
    BuilderOfflineInput, BuilderOrchestrationOutcome, DeterministicBuilderSandboxValidator,
    Ph1BuilderConfig, Ph1BuilderOrchestrator,
};
use selene_os::ph1os::{OsVoiceLiveTurnOutcome, OsVoiceTrigger};
use selene_os::ph1pattern::Ph1PatternEngine;
use selene_os::ph1rll::Ph1RllEngine;
use selene_os::simulation_executor::SimulationExecutor;
use selene_storage::ph1f::{
    DeviceRecord, IdentityRecord, IdentityStatus, MobileArtifactSyncKind, MobileArtifactSyncState,
    OutcomeUtilizationLedgerRowInput, Ph1fStore, StorageError,
};

pub mod grpc_api {
    tonic::include_proto!("selene.adapter.v1");
}

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    pub const ADAPTER_SYNC_RETRY: ReasonCodeId = ReasonCodeId(0xAD70_0001);
    pub const ADAPTER_SYNC_DEADLETTER: ReasonCodeId = ReasonCodeId(0xAD70_0002);
    pub const ADAPTER_SYNC_REPLAY_DUE: ReasonCodeId = ReasonCodeId(0xAD70_0003);
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VoiceTurnAdapterRequest {
    pub correlation_id: u64,
    pub turn_id: u64,
    pub app_platform: String,
    pub trigger: String,
    pub actor_user_id: String,
    pub tenant_id: Option<String>,
    pub device_id: Option<String>,
    pub now_ns: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VoiceTurnAdapterResponse {
    pub status: String,
    pub outcome: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct AdapterSyncWorkerCounters {
    pub pass_count: u64,
    pub dequeued_total: u64,
    pub acked_total: u64,
    pub retry_scheduled_total: u64,
    pub dead_lettered_total: u64,
    pub last_pass_at_ns: Option<u64>,
    pub last_dequeued_count: u16,
    pub last_acked_count: u16,
    pub last_retry_scheduled_count: u16,
    pub last_dead_lettered_count: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct AdapterSyncQueueCounters {
    pub queued_count: u32,
    pub in_flight_count: u32,
    pub acked_count: u32,
    pub dead_letter_count: u32,
    pub replay_due_count: u32,
    pub retry_pending_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct AdapterSyncHealth {
    pub worker: AdapterSyncWorkerCounters,
    pub queue: AdapterSyncQueueCounters,
    pub improvement: AdapterImprovementCounters,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct AdapterImprovementCounters {
    pub feedback_events_emitted_total: u64,
    pub learn_artifacts_emitted_total: u64,
    pub builder_runs_total: u64,
    pub builder_completed_total: u64,
    pub builder_refused_total: u64,
    pub builder_not_invoked_total: u64,
    pub builder_errors_total: u64,
    pub last_builder_status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AdapterHealthResponse {
    pub status: String,
    pub outcome: String,
    pub reason: Option<String>,
    pub sync: AdapterSyncHealth,
}

#[derive(Debug, Clone)]
pub struct AdapterRuntime {
    ingress: AppServerIngressRuntime,
    store: Arc<Mutex<Ph1fStore>>,
    sync_worker_counters: Arc<Mutex<AdapterSyncWorkerCounters>>,
    improvement_counters: Arc<Mutex<AdapterImprovementCounters>>,
    auto_builder_enabled: bool,
    persistence: Option<AdapterPersistenceConfig>,
}

#[derive(Debug, Clone)]
struct AdapterPersistenceConfig {
    journal_path: PathBuf,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct AdapterJournalEntry {
    schema_version: u8,
    request: VoiceTurnAdapterRequest,
}

impl AdapterJournalEntry {
    fn v1(request: VoiceTurnAdapterRequest) -> Self {
        Self {
            schema_version: 1,
            request,
        }
    }
}

#[derive(Debug, Clone)]
struct AdapterPatternEngineRuntime {
    runtime: Ph1PatternRuntime,
}

impl AdapterPatternEngineRuntime {
    fn new() -> Self {
        Self {
            runtime: Ph1PatternRuntime::new(EnginePatternConfig::mvp_v1()),
        }
    }
}

impl Ph1PatternEngine for AdapterPatternEngineRuntime {
    fn run(&self, req: &Ph1PatternRequest) -> Ph1PatternResponse {
        self.runtime.run(req)
    }
}

#[derive(Debug, Clone)]
struct AdapterRllEngineRuntime {
    runtime: Ph1RllRuntime,
}

impl AdapterRllEngineRuntime {
    fn new() -> Self {
        Self {
            runtime: Ph1RllRuntime::new(EngineRllConfig::mvp_v1()),
        }
    }
}

impl Ph1RllEngine for AdapterRllEngineRuntime {
    fn run(&self, req: &Ph1RllRequest) -> Ph1RllResponse {
        self.runtime.run(req)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SyncIssueKind {
    Retry,
    DeadLetter,
    ReplayDue,
}

#[derive(Debug, Clone)]
struct SyncIssueRecord {
    issue_kind: SyncIssueKind,
    sync_job_id: String,
    sync_kind: MobileArtifactSyncKind,
    attempt_count: u16,
    last_error: Option<String>,
    user_id: Option<UserId>,
    device_id: DeviceId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SyncImprovementEmissionResult {
    feedback_events_emitted: u64,
    learn_artifacts_emitted: u64,
    builder_input_entries: Vec<OsOutcomeUtilizationEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BuilderStatusKind {
    RunStarted,
    Completed,
    Refused,
    NotInvoked,
    Error,
}

impl Default for AdapterRuntime {
    fn default() -> Self {
        Self {
            ingress: AppServerIngressRuntime::default(),
            store: Arc::new(Mutex::new(Ph1fStore::new_in_memory())),
            sync_worker_counters: Arc::new(Mutex::new(AdapterSyncWorkerCounters::default())),
            improvement_counters: Arc::new(Mutex::new(AdapterImprovementCounters::default())),
            auto_builder_enabled: true,
            persistence: None,
        }
    }
}

impl AdapterRuntime {
    pub fn new(ingress: AppServerIngressRuntime, store: Arc<Mutex<Ph1fStore>>) -> Self {
        Self {
            ingress,
            store,
            sync_worker_counters: Arc::new(Mutex::new(AdapterSyncWorkerCounters::default())),
            improvement_counters: Arc::new(Mutex::new(AdapterImprovementCounters::default())),
            auto_builder_enabled: true,
            persistence: None,
        }
    }

    pub fn new_with_persistence(
        ingress: AppServerIngressRuntime,
        store: Arc<Mutex<Ph1fStore>>,
        journal_path: PathBuf,
        auto_builder_enabled: bool,
    ) -> Result<Self, String> {
        let runtime = Self {
            ingress,
            store,
            sync_worker_counters: Arc::new(Mutex::new(AdapterSyncWorkerCounters::default())),
            improvement_counters: Arc::new(Mutex::new(AdapterImprovementCounters::default())),
            auto_builder_enabled,
            persistence: Some(AdapterPersistenceConfig { journal_path }),
        };
        runtime.ensure_persistence_ready()?;
        runtime.replay_journal_into_store()?;
        Ok(runtime)
    }

    pub fn run_voice_turn(
        &self,
        request: VoiceTurnAdapterRequest,
    ) -> Result<VoiceTurnAdapterResponse, String> {
        self.run_voice_turn_internal(request, true)
    }

    pub fn run_device_artifact_sync_worker_pass(&self, now_ns: Option<u64>) -> Result<(), String> {
        let now_ns = now_ns.unwrap_or_else(system_time_now_ns).max(1);
        let _ = self.run_device_artifact_sync_worker_pass_internal(now_ns)?;
        Ok(())
    }

    pub fn health_report(&self, now_ns: Option<u64>) -> Result<AdapterHealthResponse, String> {
        let now_ns = now_ns.unwrap_or_else(system_time_now_ns).max(1);
        let now = MonotonicTimeNs(now_ns);
        let store = self
            .store
            .lock()
            .map_err(|_| "adapter store lock poisoned".to_string())?;
        let queue = snapshot_sync_queue_counters(&store, now);
        drop(store);
        let worker = self
            .sync_worker_counters
            .lock()
            .map_err(|_| "adapter sync worker counters lock poisoned".to_string())?
            .clone();
        let improvement = self
            .improvement_counters
            .lock()
            .map_err(|_| "adapter improvement counters lock poisoned".to_string())?
            .clone();

        Ok(AdapterHealthResponse {
            status: "ok".to_string(),
            outcome: "HEALTHY".to_string(),
            reason: None,
            sync: AdapterSyncHealth {
                worker,
                queue,
                improvement,
            },
        })
    }

    fn run_device_artifact_sync_worker_pass_internal(
        &self,
        now_ns: u64,
    ) -> Result<DeviceArtifactSyncWorkerPassMetrics, String> {
        let correlation_id = CorrelationId(now_ns as u128);
        let turn_id = TurnId(now_ns);
        let now = MonotonicTimeNs(now_ns);
        let mut store = self
            .store
            .lock()
            .map_err(|_| "adapter store lock poisoned".to_string())?;
        let metrics = self
            .ingress
            .run_device_artifact_sync_worker_pass_with_metrics(
                &mut store,
                now,
                correlation_id,
                turn_id,
            )
            .map_err(storage_error_to_string)?;
        let queue_after = snapshot_sync_queue_counters(&store, now);
        let improvement = match self.emit_sync_improvement_events(
            &mut store,
            now,
            correlation_id,
            turn_id,
            &metrics,
            &queue_after,
        ) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("selene_adapter sync improvement emit failed: {err}");
                SyncImprovementEmissionResult {
                    feedback_events_emitted: 0,
                    learn_artifacts_emitted: 0,
                    builder_input_entries: Vec::new(),
                }
            }
        };
        if let Err(err) = self.maybe_run_builder_for_sync_improvements(
            &mut store,
            now,
            correlation_id,
            turn_id,
            &metrics,
            &queue_after,
            &improvement.builder_input_entries,
        ) {
            eprintln!("selene_adapter builder auto-run failed: {err}");
        }
        drop(store);
        self.record_sync_worker_metrics(now_ns, &metrics)?;
        if let Err(err) = self.record_sync_improvement_metrics(&improvement) {
            eprintln!("selene_adapter sync improvement metrics update failed: {err}");
        }
        Ok(metrics)
    }

    fn record_sync_worker_metrics(
        &self,
        now_ns: u64,
        metrics: &DeviceArtifactSyncWorkerPassMetrics,
    ) -> Result<(), String> {
        let mut counters = self
            .sync_worker_counters
            .lock()
            .map_err(|_| "adapter sync worker counters lock poisoned".to_string())?;
        counters.pass_count = counters.pass_count.saturating_add(1);
        counters.dequeued_total = counters
            .dequeued_total
            .saturating_add(metrics.dequeued_count as u64);
        counters.acked_total = counters
            .acked_total
            .saturating_add(metrics.acked_count as u64);
        counters.retry_scheduled_total = counters
            .retry_scheduled_total
            .saturating_add(metrics.retry_scheduled_count as u64);
        counters.dead_lettered_total = counters
            .dead_lettered_total
            .saturating_add(metrics.dead_lettered_count as u64);
        counters.last_pass_at_ns = Some(now_ns);
        counters.last_dequeued_count = metrics.dequeued_count;
        counters.last_acked_count = metrics.acked_count;
        counters.last_retry_scheduled_count = metrics.retry_scheduled_count;
        counters.last_dead_lettered_count = metrics.dead_lettered_count;
        Ok(())
    }

    fn record_sync_improvement_metrics(
        &self,
        emitted: &SyncImprovementEmissionResult,
    ) -> Result<(), String> {
        let mut counters = self
            .improvement_counters
            .lock()
            .map_err(|_| "adapter improvement counters lock poisoned".to_string())?;
        counters.feedback_events_emitted_total = counters
            .feedback_events_emitted_total
            .saturating_add(emitted.feedback_events_emitted);
        counters.learn_artifacts_emitted_total = counters
            .learn_artifacts_emitted_total
            .saturating_add(emitted.learn_artifacts_emitted);
        Ok(())
    }

    fn emit_sync_improvement_events(
        &self,
        store: &mut Ph1fStore,
        now: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        _metrics: &DeviceArtifactSyncWorkerPassMetrics,
        queue_after: &AdapterSyncQueueCounters,
    ) -> Result<SyncImprovementEmissionResult, String> {
        let issue_records = collect_sync_issue_records_for_pass(store, now, queue_after);
        let mut feedback_events_emitted = 0u64;
        let mut learn_artifacts_emitted = 0u64;
        let mut builder_input_entries = Vec::new();
        let mut next_version_by_scope: BTreeMap<(String, ArtifactType), u32> = BTreeMap::new();

        for issue in issue_records {
            let (outcome_type, reason_code) = match issue.issue_kind {
                SyncIssueKind::Retry => ("VOICE_SYNC_RETRY", reason_codes::ADAPTER_SYNC_RETRY),
                SyncIssueKind::DeadLetter => (
                    "VOICE_SYNC_DEADLETTER",
                    reason_codes::ADAPTER_SYNC_DEADLETTER,
                ),
                SyncIssueKind::ReplayDue => (
                    "VOICE_SYNC_REPLAY_DUE",
                    reason_codes::ADAPTER_SYNC_REPLAY_DUE,
                ),
            };
            let issue_tag = sync_issue_tag(issue.issue_kind);
            let issue_idem = sanitize_idempotency_token(&format!(
                "sync_issue:{}:{}:{}:{}",
                issue_tag, issue.sync_job_id, issue.attempt_count, now.0
            ));

            let outcome_entry = match OsOutcomeUtilizationEntry::v1(
                "PH1.FEEDBACK".to_string(),
                outcome_type.to_string(),
                correlation_id,
                turn_id,
                OsOutcomeActionClass::QueueLearn,
                "PH1.LEARN".to_string(),
                sync_issue_latency_ms(issue.issue_kind),
                true,
                reason_code,
            ) {
                Ok(entry) => entry,
                Err(err) => {
                    eprintln!("selene_adapter outcome entry build failed: {err:?}");
                    continue;
                }
            };
            if let Err(err) =
                store.append_outcome_utilization_ledger_row(OutcomeUtilizationLedgerRowInput {
                    created_at: now,
                    correlation_id,
                    turn_id,
                    engine_id: "PH1.FEEDBACK".to_string(),
                    outcome_type: outcome_type.to_string(),
                    action_class: OsOutcomeActionClass::QueueLearn,
                    consumed_by: "PH1.LEARN".to_string(),
                    latency_cost_ms: sync_issue_latency_ms(issue.issue_kind),
                    decision_delta: true,
                    reason_code,
                    idempotency_key: Some(issue_idem.clone()),
                })
            {
                eprintln!(
                    "selene_adapter outcome utilization append failed: {}",
                    storage_error_to_string(err)
                );
                continue;
            }
            builder_input_entries.push(outcome_entry);

            let Some(user_id) = issue.user_id.clone() else {
                continue;
            };
            let Some(tenant_id) = tenant_scope_from_user_id(&user_id).map(str::to_string) else {
                continue;
            };

            let (feedback_event_type, learn_signal_type) = match issue.issue_kind {
                SyncIssueKind::Retry => ("VoiceIdReauthFriction", "VoiceIdReauthFriction"),
                SyncIssueKind::DeadLetter => ("VoiceIdDriftAlert", "VoiceIdDriftAlert"),
                SyncIssueKind::ReplayDue => ("VoiceIdDriftAlert", "VoiceIdDriftAlert"),
            };
            match store.ph1feedback_event_commit(
                now,
                tenant_id.clone(),
                correlation_id,
                turn_id,
                None,
                user_id.clone(),
                issue.device_id.clone(),
                feedback_event_type.to_string(),
                learn_signal_type.to_string(),
                reason_code,
                issue_idem.clone(),
            ) {
                Ok(_) => {
                    feedback_events_emitted = feedback_events_emitted.saturating_add(1);
                }
                Err(err) => {
                    eprintln!(
                        "selene_adapter feedback emit failed: {}",
                        storage_error_to_string(err)
                    );
                }
            }

            let artifact_type = artifact_type_for_sync_issue(issue.issue_kind);
            let version_key = (tenant_id.clone(), artifact_type);
            let next_version = if let Some(existing) = next_version_by_scope.get_mut(&version_key) {
                *existing = existing.saturating_add(1);
                *existing
            } else {
                let rows = store.ph1learn_artifact_rows(
                    ArtifactScopeType::Tenant,
                    &tenant_id,
                    artifact_type,
                );
                let base = rows
                    .iter()
                    .map(|row| row.artifact_version.0)
                    .max()
                    .unwrap_or(0)
                    .saturating_add(1);
                next_version_by_scope.insert(version_key, base);
                base
            };
            let package_hash = stable_hash_hex_16(&format!(
                "{tenant}:{job}:{kind:?}:{issue}:{attempt}:{err}",
                tenant = tenant_id,
                job = issue.sync_job_id,
                kind = issue.sync_kind,
                issue = issue_tag,
                attempt = issue.attempt_count,
                err = issue.last_error.as_deref().unwrap_or("none"),
            ));
            let payload_ref = truncate_ascii(
                &format!(
                    "learn:voice_sync:{issue}:{job}:attempt:{attempt}:kind:{kind:?}",
                    issue = issue_tag,
                    job = issue.sync_job_id,
                    attempt = issue.attempt_count,
                    kind = issue.sync_kind
                ),
                256,
            );
            let provenance_ref = truncate_ascii(
                &format!(
                    "sync_feedback:{issue}:{job}",
                    issue = issue_tag,
                    job = issue.sync_job_id
                ),
                128,
            );
            let learn_idem = sanitize_idempotency_token(&format!(
                "learn_sync:{}:{}:{}",
                issue_tag, issue.sync_job_id, issue.attempt_count
            ));
            match store.ph1learn_artifact_commit(
                now,
                tenant_id.clone(),
                ArtifactScopeType::Tenant,
                tenant_id,
                artifact_type,
                ArtifactVersion(next_version),
                package_hash,
                payload_ref,
                provenance_ref,
                ArtifactStatus::Active,
                learn_idem,
            ) {
                Ok(_) => {
                    learn_artifacts_emitted = learn_artifacts_emitted.saturating_add(1);
                }
                Err(err) => {
                    eprintln!(
                        "selene_adapter learn artifact emit failed: {}",
                        storage_error_to_string(err)
                    );
                }
            }
        }

        Ok(SyncImprovementEmissionResult {
            feedback_events_emitted,
            learn_artifacts_emitted,
            builder_input_entries,
        })
    }

    fn maybe_run_builder_for_sync_improvements(
        &self,
        store: &mut Ph1fStore,
        now: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        metrics: &DeviceArtifactSyncWorkerPassMetrics,
        queue_after: &AdapterSyncQueueCounters,
        outcome_entries: &[OsOutcomeUtilizationEntry],
    ) -> Result<(), String> {
        if !self.auto_builder_enabled {
            self.record_builder_status("DISABLED", BuilderStatusKind::NotInvoked)?;
            return Ok(());
        }
        if outcome_entries.is_empty() {
            self.record_builder_status("NO_SYNC_ISSUES", BuilderStatusKind::NotInvoked)?;
            return Ok(());
        }
        let severe = metrics.dead_lettered_count > 0 || queue_after.replay_due_count > 0;
        if !severe {
            self.record_builder_status("SKIPPED_NON_SEVERE", BuilderStatusKind::NotInvoked)?;
            return Ok(());
        }

        self.record_builder_status("RUNNING", BuilderStatusKind::RunStarted)?;
        let orchestrator = Ph1BuilderOrchestrator::new(
            Ph1BuilderConfig::mvp_v1(true),
            AdapterPatternEngineRuntime::new(),
            AdapterRllEngineRuntime::new(),
            DeterministicBuilderSandboxValidator,
        )
        .map_err(|err| format!("failed to initialize builder orchestrator: {err:?}"))?;
        let window_start = MonotonicTimeNs(now.0.saturating_sub(60_000_000_000));
        let builder_input = BuilderOfflineInput::v1(
            correlation_id,
            turn_id,
            window_start,
            now,
            now,
            outcome_entries.to_vec(),
            None,
            None,
            None,
            None,
            None,
            None,
            true,
        )
        .map_err(|err| format!("failed to build builder offline input: {err:?}"))?;

        match orchestrator.run_offline(store, &builder_input) {
            Ok(BuilderOrchestrationOutcome::Completed(_)) => {
                self.record_builder_status("COMPLETED", BuilderStatusKind::Completed)?;
            }
            Ok(BuilderOrchestrationOutcome::Refused(refuse)) => {
                self.record_builder_status(
                    &format!("REFUSED:{}:{}", refuse.stage, refuse.reason_code.0),
                    BuilderStatusKind::Refused,
                )?;
            }
            Ok(BuilderOrchestrationOutcome::NotInvokedDisabled) => {
                self.record_builder_status("NOT_INVOKED_DISABLED", BuilderStatusKind::NotInvoked)?;
            }
            Ok(BuilderOrchestrationOutcome::NotInvokedNoSignals) => {
                self.record_builder_status(
                    "NOT_INVOKED_NO_SIGNALS",
                    BuilderStatusKind::NotInvoked,
                )?;
            }
            Err(err) => {
                self.record_builder_status(&format!("ERROR:{err:?}"), BuilderStatusKind::Error)?;
            }
        }
        Ok(())
    }

    fn record_builder_status(&self, status: &str, kind: BuilderStatusKind) -> Result<(), String> {
        let mut counters = self
            .improvement_counters
            .lock()
            .map_err(|_| "adapter improvement counters lock poisoned".to_string())?;
        match kind {
            BuilderStatusKind::RunStarted => {
                counters.builder_runs_total = counters.builder_runs_total.saturating_add(1);
            }
            BuilderStatusKind::Completed => {
                counters.builder_completed_total =
                    counters.builder_completed_total.saturating_add(1);
            }
            BuilderStatusKind::Refused => {
                counters.builder_refused_total = counters.builder_refused_total.saturating_add(1);
            }
            BuilderStatusKind::NotInvoked => {
                counters.builder_not_invoked_total =
                    counters.builder_not_invoked_total.saturating_add(1);
            }
            BuilderStatusKind::Error => {
                counters.builder_errors_total = counters.builder_errors_total.saturating_add(1);
            }
        }
        counters.last_builder_status = Some(truncate_ascii(status, 256));
        Ok(())
    }

    fn run_voice_turn_internal(
        &self,
        request: VoiceTurnAdapterRequest,
        persist_on_success: bool,
    ) -> Result<VoiceTurnAdapterResponse, String> {
        let request_for_journal = request.clone();
        let app_platform = parse_app_platform(&request.app_platform)?;
        let trigger = parse_trigger(&request.trigger)?;
        let actor_user_id = UserId::new(request.actor_user_id.clone())
            .map_err(|err| format!("invalid actor_user_id: {err:?}"))?;
        let device_id = request
            .device_id
            .as_ref()
            .map(|id| {
                DeviceId::new(id.clone()).map_err(|err| format!("invalid device_id: {err:?}"))
            })
            .transpose()?;
        let correlation_id = CorrelationId(request.correlation_id.into());
        let turn_id = TurnId(request.turn_id);
        let now = MonotonicTimeNs(request.now_ns.unwrap_or(1));
        let voice_id_request = build_default_voice_id_request(now, actor_user_id.clone())
            .map_err(|err| format!("voice request build failed: {err:?}"))?;

        let mut store = self
            .store
            .lock()
            .map_err(|_| "adapter store lock poisoned".to_string())?;
        ensure_actor_identity_and_device(
            &mut store,
            &actor_user_id,
            device_id.as_ref(),
            app_platform,
            now,
        )?;

        let ingress_request = AppVoiceIngressRequest::v1(
            correlation_id,
            turn_id,
            app_platform,
            trigger,
            voice_id_request,
            actor_user_id,
            request.tenant_id,
            device_id,
            Vec::new(),
            empty_observation(),
        )
        .map_err(|err| format!("invalid ingress request: {err:?}"))?;
        let outcome = self
            .ingress
            .run_voice_turn(&mut store, ingress_request)
            .map_err(storage_error_to_string)?;
        let response = outcome_to_adapter_response(outcome);
        if persist_on_success {
            self.append_journal_entry(request_for_journal)?;
        }
        Ok(response)
    }

    pub fn default_from_env() -> Result<Self, String> {
        let mut executor = SimulationExecutor::default();
        if let Some(global_profiles) =
            build_embedding_gate_profiles_from_env_var_map(|key| env::var(key).ok())?
        {
            let config = Ph1VoiceIdLiveConfig {
                embedding_gate_profiles: VoiceIdentityEmbeddingGateGovernedConfig {
                    global_profiles,
                    tenant_overrides: BTreeMap::new(),
                },
                contract_migration: VoiceIdContractMigrationConfig::mvp_default(),
            };
            executor.set_voice_id_live_config(config);
        }
        let ingress = AppServerIngressRuntime::new(executor);
        let store = Arc::new(Mutex::new(Ph1fStore::new_in_memory()));
        let journal_path = env::var("SELENE_ADAPTER_STORE_PATH")
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".selene/adapter/voice_turns.jsonl"));
        let auto_builder_enabled = parse_auto_builder_enabled_from_env();

        Self::new_with_persistence(ingress, store, journal_path, auto_builder_enabled)
    }

    fn ensure_persistence_ready(&self) -> Result<(), String> {
        let Some(persistence) = self.persistence.as_ref() else {
            return Ok(());
        };
        let path = &persistence.journal_path;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                format!(
                    "failed to create adapter store directory '{}': {}",
                    parent.display(),
                    err
                )
            })?;
        }
        if !path.exists() {
            File::create(path).map_err(|err| {
                format!(
                    "failed to create adapter store journal '{}': {}",
                    path.display(),
                    err
                )
            })?;
        }
        Ok(())
    }

    fn replay_journal_into_store(&self) -> Result<(), String> {
        let Some(persistence) = self.persistence.as_ref() else {
            return Ok(());
        };
        let file = File::open(&persistence.journal_path).map_err(|err| {
            format!(
                "failed to open adapter store journal '{}': {}",
                persistence.journal_path.display(),
                err
            )
        })?;
        for (line_no, line_result) in BufReader::new(file).lines().enumerate() {
            let line = line_result.map_err(|err| {
                format!(
                    "failed reading adapter store journal '{}' at line {}: {}",
                    persistence.journal_path.display(),
                    line_no + 1,
                    err
                )
            })?;
            if line.trim().is_empty() {
                continue;
            }
            let entry: AdapterJournalEntry = serde_json::from_str(&line).map_err(|err| {
                format!(
                    "failed parsing adapter store journal '{}' at line {}: {}",
                    persistence.journal_path.display(),
                    line_no + 1,
                    err
                )
            })?;
            if entry.schema_version != 1 {
                return Err(format!(
                    "unsupported adapter store journal schema_version={} at line {}",
                    entry.schema_version,
                    line_no + 1
                ));
            }
            self.run_voice_turn_internal(entry.request, false)
                .map_err(|err| format!("journal replay failed at line {}: {}", line_no + 1, err))?;
        }
        Ok(())
    }

    fn append_journal_entry(&self, request: VoiceTurnAdapterRequest) -> Result<(), String> {
        let Some(persistence) = self.persistence.as_ref() else {
            return Ok(());
        };
        let entry = AdapterJournalEntry::v1(request);
        let json = serde_json::to_string(&entry)
            .map_err(|err| format!("failed to encode adapter journal entry: {err}"))?;
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&persistence.journal_path)
            .map_err(|err| {
                format!(
                    "failed opening adapter store journal '{}' for append: {}",
                    persistence.journal_path.display(),
                    err
                )
            })?;
        file.write_all(json.as_bytes())
            .and_then(|_| file.write_all(b"\n"))
            .and_then(|_| file.sync_data())
            .map_err(|err| {
                format!(
                    "failed writing adapter store journal '{}': {}",
                    persistence.journal_path.display(),
                    err
                )
            })?;
        Ok(())
    }
}

fn build_embedding_gate_profiles_from_env_var_map<F>(
    mut env_getter: F,
) -> Result<Option<VoiceIdentityEmbeddingGateProfiles>, String>
where
    F: FnMut(&str) -> Option<String>,
{
    let mut profiles = VoiceIdentityEmbeddingGateProfiles::mvp_v1_phone_first();
    let mut has_override = false;

    if let Some(v) = env_getter("SELENE_VID_GATE_GLOBAL_DEFAULT") {
        profiles.global_default =
            parse_embedding_gate_profile("SELENE_VID_GATE_GLOBAL_DEFAULT", &v)?;
        has_override = true;
    }
    if let Some(v) = env_getter("SELENE_VID_GATE_IOS_EXPLICIT") {
        profiles.ios_explicit = parse_embedding_gate_profile("SELENE_VID_GATE_IOS_EXPLICIT", &v)?;
        has_override = true;
    }
    if let Some(v) = env_getter("SELENE_VID_GATE_IOS_WAKE") {
        profiles.ios_wake = parse_embedding_gate_profile("SELENE_VID_GATE_IOS_WAKE", &v)?;
        has_override = true;
    }
    if let Some(v) = env_getter("SELENE_VID_GATE_ANDROID_EXPLICIT") {
        profiles.android_explicit =
            parse_embedding_gate_profile("SELENE_VID_GATE_ANDROID_EXPLICIT", &v)?;
        has_override = true;
    }
    if let Some(v) = env_getter("SELENE_VID_GATE_ANDROID_WAKE") {
        profiles.android_wake = parse_embedding_gate_profile("SELENE_VID_GATE_ANDROID_WAKE", &v)?;
        has_override = true;
    }
    if let Some(v) = env_getter("SELENE_VID_GATE_DESKTOP_EXPLICIT") {
        profiles.desktop_explicit =
            parse_embedding_gate_profile("SELENE_VID_GATE_DESKTOP_EXPLICIT", &v)?;
        has_override = true;
    }
    if let Some(v) = env_getter("SELENE_VID_GATE_DESKTOP_WAKE") {
        profiles.desktop_wake = parse_embedding_gate_profile("SELENE_VID_GATE_DESKTOP_WAKE", &v)?;
        has_override = true;
    }

    if has_override {
        Ok(Some(profiles))
    } else {
        Ok(None)
    }
}

fn parse_embedding_gate_profile(
    key: &'static str,
    value: &str,
) -> Result<VoiceIdentityEmbeddingGateProfile, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "required" => Ok(VoiceIdentityEmbeddingGateProfile::required()),
        "optional" => Ok(VoiceIdentityEmbeddingGateProfile::optional()),
        _ => Err(format!("{key} must be 'required' or 'optional'")),
    }
}

fn parse_app_platform(value: &str) -> Result<AppPlatform, String> {
    let normalized = value.trim().to_ascii_uppercase();
    match normalized.as_str() {
        "IOS" => Ok(AppPlatform::Ios),
        "ANDROID" => Ok(AppPlatform::Android),
        "DESKTOP" => Ok(AppPlatform::Desktop),
        _ => Err(format!(
            "invalid app_platform '{}'; expected IOS|ANDROID|DESKTOP",
            value
        )),
    }
}

fn parse_trigger(value: &str) -> Result<OsVoiceTrigger, String> {
    let normalized = value.trim().to_ascii_uppercase();
    match normalized.as_str() {
        "EXPLICIT" => Ok(OsVoiceTrigger::Explicit),
        "WAKE_WORD" => Ok(OsVoiceTrigger::WakeWord),
        _ => Err(format!(
            "invalid trigger '{}'; expected EXPLICIT|WAKE_WORD",
            value
        )),
    }
}

fn build_default_voice_id_request(
    now: MonotonicTimeNs,
    actor_user_id: UserId,
) -> Result<Ph1VoiceIdRequest, selene_kernel_contracts::ContractViolation> {
    let stream_id = AudioStreamId(1);
    let processed_stream_ref = AudioStreamRef::v1(
        stream_id,
        AudioStreamKind::MicProcessed,
        AudioFormat {
            sample_rate_hz: SampleRateHz(16_000),
            channels: ChannelCount(1),
            sample_format: SampleFormat::PcmS16LE,
        },
        FrameDurationMs::Ms20,
    );
    let vad_events = vec![VadEvent::v1(
        stream_id,
        MonotonicTimeNs(now.0.saturating_sub(2_000_000)),
        now,
        Confidence::new(0.95)?,
        SpeechLikeness::new(0.95)?,
    )];
    let session_snapshot = SessionSnapshot {
        schema_version: SchemaVersion(1),
        session_state: SessionState::Active,
        session_id: Some(SessionId(1)),
        next_allowed_actions: NextAllowedActions {
            may_speak: true,
            must_wait: false,
            must_rewake: false,
        },
    };
    let device_id = AudioDeviceId::new("adapter_mic_device_1".to_string())?;
    Ph1VoiceIdRequest::v1(
        now,
        processed_stream_ref,
        vad_events,
        device_id,
        session_snapshot,
        None,
        false,
        DeviceTrustLevel::Trusted,
        Some(actor_user_id),
    )
}

fn ensure_actor_identity_and_device(
    store: &mut Ph1fStore,
    actor_user_id: &UserId,
    device_id: Option<&DeviceId>,
    app_platform: AppPlatform,
    now: MonotonicTimeNs,
) -> Result<(), String> {
    if store.get_identity(actor_user_id).is_none() {
        store
            .insert_identity(IdentityRecord::v1(
                actor_user_id.clone(),
                None,
                None,
                now,
                IdentityStatus::Active,
            ))
            .map_err(storage_error_to_string)?;
    }
    if let Some(device_id) = device_id {
        if store.get_device(device_id).is_none() {
            store
                .insert_device(
                    DeviceRecord::v1(
                        device_id.clone(),
                        actor_user_id.clone(),
                        default_device_type(app_platform).to_string(),
                        now,
                        None,
                    )
                    .map_err(|err| format!("invalid device record: {err:?}"))?,
                )
                .map_err(storage_error_to_string)?;
        }
    }
    Ok(())
}

fn default_device_type(app_platform: AppPlatform) -> &'static str {
    match app_platform {
        AppPlatform::Ios | AppPlatform::Android => "phone",
        AppPlatform::Desktop => "desktop",
    }
}

fn empty_observation() -> EngineVoiceIdObservation {
    EngineVoiceIdObservation {
        primary_fingerprint: None,
        secondary_fingerprint: None,
        primary_embedding: None,
        secondary_embedding: None,
        spoof_risk: false,
    }
}

fn outcome_to_adapter_response(outcome: OsVoiceLiveTurnOutcome) -> VoiceTurnAdapterResponse {
    match outcome {
        OsVoiceLiveTurnOutcome::NotInvokedDisabled => VoiceTurnAdapterResponse {
            status: "ok".to_string(),
            outcome: "NOT_INVOKED_DISABLED".to_string(),
            reason: None,
        },
        OsVoiceLiveTurnOutcome::Refused(refuse) => VoiceTurnAdapterResponse {
            status: "ok".to_string(),
            outcome: "REFUSED".to_string(),
            reason: Some(format!(
                "os_refuse reason_code={} message={}",
                refuse.reason_code.0, refuse.message
            )),
        },
        OsVoiceLiveTurnOutcome::Forwarded(bundle) => {
            let reason = match bundle.voice_identity_assertion {
                Ph1VoiceIdResponse::SpeakerAssertionOk(ok) => Some(format!(
                    "voice_identity=OK score_bp={} user_id={}",
                    ok.score_bp,
                    ok.user_id.as_ref().map(UserId::as_str).unwrap_or("none")
                )),
                Ph1VoiceIdResponse::SpeakerAssertionUnknown(u) => Some(format!(
                    "voice_identity=UNKNOWN reason_code={} score_bp={}",
                    u.reason_code.0, u.score_bp
                )),
            };
            VoiceTurnAdapterResponse {
                status: "ok".to_string(),
                outcome: "FORWARDED".to_string(),
                reason,
            }
        }
    }
}

fn storage_error_to_string(err: StorageError) -> String {
    format!("{err:?}")
}

fn snapshot_sync_queue_counters(
    store: &Ph1fStore,
    now: MonotonicTimeNs,
) -> AdapterSyncQueueCounters {
    let mut counters = AdapterSyncQueueCounters::default();
    for row in store.device_artifact_sync_queue_rows() {
        match row.state {
            MobileArtifactSyncState::Queued => {
                counters.queued_count = counters.queued_count.saturating_add(1);
            }
            MobileArtifactSyncState::InFlight => {
                counters.in_flight_count = counters.in_flight_count.saturating_add(1);
                if row.last_error.is_some() {
                    counters.retry_pending_count = counters.retry_pending_count.saturating_add(1);
                }
            }
            MobileArtifactSyncState::Acked => {
                counters.acked_count = counters.acked_count.saturating_add(1);
            }
            MobileArtifactSyncState::DeadLetter => {
                counters.dead_letter_count = counters.dead_letter_count.saturating_add(1);
            }
        }
    }
    counters.replay_due_count = store.device_artifact_sync_replay_due_rows(now).len() as u32;
    counters
}

fn collect_sync_issue_records_for_pass(
    store: &Ph1fStore,
    now: MonotonicTimeNs,
    queue_after: &AdapterSyncQueueCounters,
) -> Vec<SyncIssueRecord> {
    let mut out = Vec::new();
    for row in store.device_artifact_sync_queue_rows() {
        let attempted_this_pass = row.last_attempted_at == Some(now);
        if attempted_this_pass && row.state == MobileArtifactSyncState::DeadLetter {
            out.push(SyncIssueRecord {
                issue_kind: SyncIssueKind::DeadLetter,
                sync_job_id: row.sync_job_id.clone(),
                sync_kind: row.sync_kind,
                attempt_count: row.attempt_count,
                last_error: row.last_error.clone(),
                user_id: row.user_id.clone(),
                device_id: row.device_id.clone(),
            });
            continue;
        }
        if attempted_this_pass && row.last_error.is_some() {
            out.push(SyncIssueRecord {
                issue_kind: SyncIssueKind::Retry,
                sync_job_id: row.sync_job_id.clone(),
                sync_kind: row.sync_kind,
                attempt_count: row.attempt_count,
                last_error: row.last_error.clone(),
                user_id: row.user_id.clone(),
                device_id: row.device_id.clone(),
            });
        }
    }
    if queue_after.replay_due_count > 0 {
        out.push(SyncIssueRecord {
            issue_kind: SyncIssueKind::ReplayDue,
            sync_job_id: "queue_replay_due".to_string(),
            sync_kind: MobileArtifactSyncKind::VoiceProfile,
            attempt_count: queue_after.replay_due_count as u16,
            last_error: Some("replay_due".to_string()),
            user_id: None,
            device_id: DeviceId::new("adapter_sync_aggregate_device")
                .expect("static aggregate device id must be valid"),
        });
    }
    out
}

fn sync_issue_tag(kind: SyncIssueKind) -> &'static str {
    match kind {
        SyncIssueKind::Retry => "RETRY",
        SyncIssueKind::DeadLetter => "DEADLETTER",
        SyncIssueKind::ReplayDue => "REPLAY_DUE",
    }
}

fn sync_issue_latency_ms(kind: SyncIssueKind) -> u32 {
    match kind {
        SyncIssueKind::Retry => 100,
        SyncIssueKind::DeadLetter => 250,
        SyncIssueKind::ReplayDue => 500,
    }
}

fn artifact_type_for_sync_issue(kind: SyncIssueKind) -> ArtifactType {
    match kind {
        SyncIssueKind::Retry => ArtifactType::VoiceIdProfileDeltaPack,
        SyncIssueKind::DeadLetter => ArtifactType::VoiceIdProfileDeltaPack,
        SyncIssueKind::ReplayDue => ArtifactType::VoiceIdThresholdPack,
    }
}

fn tenant_scope_from_user_id(user_id: &UserId) -> Option<&str> {
    let (tenant_scope, _) = user_id.as_str().split_once(':')?;
    if tenant_scope.trim().is_empty() {
        return None;
    }
    Some(tenant_scope)
}

fn sanitize_idempotency_token(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.') {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "sync_idem".to_string()
    } else {
        truncate_ascii(&out, 128)
    }
}

fn truncate_ascii(value: &str, max_len: usize) -> String {
    value.chars().take(max_len).collect::<String>()
}

fn stable_hash_hex_16(value: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn parse_auto_builder_enabled_from_env() -> bool {
    match env::var("SELENE_ADAPTER_AUTO_BUILDER_ENABLED") {
        Ok(v) => !matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "0" | "false" | "off" | "no"
        ),
        Err(_) => true,
    }
}

fn system_time_now_ns() -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(1);
    if nanos > u64::MAX as u128 {
        u64::MAX
    } else {
        nanos as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn base_request() -> VoiceTurnAdapterRequest {
        VoiceTurnAdapterRequest {
            correlation_id: 10_001,
            turn_id: 20_001,
            app_platform: "IOS".to_string(),
            trigger: "EXPLICIT".to_string(),
            actor_user_id: "tenant_a:user_adapter_test".to_string(),
            tenant_id: Some("tenant_a".to_string()),
            device_id: Some("adapter_device_1".to_string()),
            now_ns: Some(3),
        }
    }

    #[test]
    fn at_adapter_01_valid_ios_request_forwards() {
        let runtime = AdapterRuntime::default();
        let out = runtime
            .run_voice_turn(base_request())
            .expect("valid request must succeed");
        assert_eq!(out.status, "ok");
        assert_eq!(out.outcome, "FORWARDED");
        assert!(out
            .reason
            .as_deref()
            .unwrap_or_default()
            .contains("voice_identity="));
    }

    #[test]
    fn at_adapter_02_invalid_platform_fails_fast() {
        let runtime = AdapterRuntime::default();
        let mut req = base_request();
        req.app_platform = "CONSOLE".to_string();
        let err = runtime
            .run_voice_turn(req)
            .expect_err("invalid platform must fail");
        assert!(err.contains("invalid app_platform"));
    }

    #[test]
    fn at_adapter_03_desktop_explicit_is_supported() {
        let runtime = AdapterRuntime::default();
        let mut req = base_request();
        req.app_platform = "DESKTOP".to_string();
        req.device_id = Some("adapter_desktop_device_1".to_string());
        let out = runtime
            .run_voice_turn(req)
            .expect("desktop request must succeed");
        assert_eq!(out.status, "ok");
        assert_eq!(out.outcome, "FORWARDED");
    }

    #[test]
    fn at_adapter_04_invalid_trigger_fails_fast() {
        let runtime = AdapterRuntime::default();
        let mut req = base_request();
        req.trigger = "PUSH_TO_TALK".to_string();
        let err = runtime
            .run_voice_turn(req)
            .expect_err("invalid trigger must fail");
        assert!(err.contains("invalid trigger"));
    }

    #[test]
    fn at_adapter_05_embedding_gate_env_overrides_parse() {
        let profiles = build_embedding_gate_profiles_from_env_var_map(|key| match key {
            "SELENE_VID_GATE_GLOBAL_DEFAULT" => Some("optional".to_string()),
            "SELENE_VID_GATE_IOS_EXPLICIT" => Some("required".to_string()),
            "SELENE_VID_GATE_IOS_WAKE" => Some("required".to_string()),
            "SELENE_VID_GATE_ANDROID_EXPLICIT" => Some("required".to_string()),
            "SELENE_VID_GATE_ANDROID_WAKE" => Some("required".to_string()),
            "SELENE_VID_GATE_DESKTOP_EXPLICIT" => Some("optional".to_string()),
            "SELENE_VID_GATE_DESKTOP_WAKE" => Some("optional".to_string()),
            _ => None,
        })
        .expect("profiles must parse")
        .expect("override should be present");

        assert_eq!(
            profiles.global_default,
            VoiceIdentityEmbeddingGateProfile::optional()
        );
        assert_eq!(
            profiles.ios_explicit,
            VoiceIdentityEmbeddingGateProfile::required()
        );
        assert_eq!(
            profiles.desktop_explicit,
            VoiceIdentityEmbeddingGateProfile::optional()
        );
    }

    #[test]
    fn at_adapter_06_embedding_gate_env_rejects_invalid_mode() {
        let err = build_embedding_gate_profiles_from_env_var_map(|key| {
            if key == "SELENE_VID_GATE_ANDROID_WAKE" {
                Some("maybe".to_string())
            } else {
                None
            }
        })
        .expect_err("invalid override must fail");
        assert!(err.contains("SELENE_VID_GATE_ANDROID_WAKE"));
    }

    #[test]
    fn at_adapter_07_journal_persists_and_replays_voice_turns() {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be >= unix epoch")
            .as_nanos();
        let journal_path =
            std::env::temp_dir().join(format!("selene_adapter_journal_{seed}.jsonl"));

        let runtime_one = AdapterRuntime::new_with_persistence(
            AppServerIngressRuntime::default(),
            Arc::new(Mutex::new(Ph1fStore::new_in_memory())),
            journal_path.clone(),
            true,
        )
        .expect("runtime with persistence must construct");
        let out = runtime_one
            .run_voice_turn(base_request())
            .expect("first runtime request must succeed");
        assert_eq!(out.status, "ok");

        let lines_after_first = std::fs::read_to_string(&journal_path)
            .expect("journal should be readable")
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count();
        assert_eq!(lines_after_first, 1);

        let runtime_two = AdapterRuntime::new_with_persistence(
            AppServerIngressRuntime::default(),
            Arc::new(Mutex::new(Ph1fStore::new_in_memory())),
            journal_path.clone(),
            true,
        )
        .expect("second runtime should replay prior journal");
        let mut req = base_request();
        req.turn_id = 20_002;
        req.now_ns = Some(4);
        let out_two = runtime_two
            .run_voice_turn(req)
            .expect("second runtime request must succeed");
        assert_eq!(out_two.status, "ok");

        let lines_after_second = std::fs::read_to_string(&journal_path)
            .expect("journal should still be readable")
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count();
        assert_eq!(lines_after_second, 2);

        let _ = std::fs::remove_file(journal_path);
    }

    #[test]
    fn at_adapter_08_sync_worker_pass_runs_after_multi_platform_turns() {
        let runtime = AdapterRuntime::default();

        let ios = base_request();
        runtime
            .run_voice_turn(ios)
            .expect("ios voice turn should succeed");

        let mut android = base_request();
        android.turn_id = 20_010;
        android.now_ns = Some(10);
        android.app_platform = "ANDROID".to_string();
        android.trigger = "WAKE_WORD".to_string();
        android.device_id = Some("adapter_android_device_1".to_string());
        runtime
            .run_voice_turn(android)
            .expect("android voice turn should succeed");

        let mut desktop = base_request();
        desktop.turn_id = 20_011;
        desktop.now_ns = Some(11);
        desktop.app_platform = "DESKTOP".to_string();
        desktop.trigger = "EXPLICIT".to_string();
        desktop.device_id = Some("adapter_desktop_device_2".to_string());
        runtime
            .run_voice_turn(desktop)
            .expect("desktop voice turn should succeed");

        runtime
            .run_device_artifact_sync_worker_pass(Some(99))
            .expect("sync worker pass should succeed");
    }

    #[test]
    fn at_adapter_09_health_report_exposes_sync_counters() {
        let runtime = AdapterRuntime::default();
        runtime
            .run_device_artifact_sync_worker_pass(Some(101))
            .expect("sync worker pass should succeed");
        let health = runtime
            .health_report(Some(101))
            .expect("health report should succeed");
        assert_eq!(health.status, "ok");
        assert_eq!(health.outcome, "HEALTHY");
        assert!(health.sync.worker.pass_count >= 1);
        assert!(health.sync.worker.last_pass_at_ns.is_some());
    }
}
