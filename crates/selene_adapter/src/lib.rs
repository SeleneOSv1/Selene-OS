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
use selene_engines::ph1health::{
    reason_codes as health_reason_codes, Ph1HealthConfig as EngineHealthConfig,
    Ph1HealthRuntime as EngineHealthRuntime,
};
use selene_engines::ph1pattern::{Ph1PatternConfig as EnginePatternConfig, Ph1PatternRuntime};
use selene_engines::ph1rll::{Ph1RllConfig as EngineRllConfig, Ph1RllRuntime};
use selene_kernel_contracts::ph1health::{
    HealthAckState, HealthActionResult, HealthCompanyScope, HealthDisplayTarget,
    HealthIssueEvent, HealthIssueStatus, HealthPageAction, HealthReadEnvelope, HealthReportKind,
    HealthReportQueryReadRequest, HealthReportQueryReadOk, HealthReportTimeRange, Ph1HealthRequest,
    Ph1HealthResponse, HealthSeverity,
};
use selene_kernel_contracts::ph1_voice_id::{
    DeviceTrustLevel, Ph1VoiceIdRequest, Ph1VoiceIdResponse, UserId,
};
use selene_kernel_contracts::ph1art::{
    ArtifactScopeType, ArtifactStatus, ArtifactType, ArtifactVersion,
};
use selene_kernel_contracts::ph1f::{
    ConversationRole, ConversationSource, ConversationTurnInput, PrivacyScope,
};
use selene_kernel_contracts::ph1j::{CorrelationId, DeviceId, TurnId};
use selene_kernel_contracts::ph1k::{
    AudioDeviceId, AudioFormat, AudioStreamId, AudioStreamKind, AudioStreamRef, ChannelCount,
    Confidence, FrameDurationMs, SampleFormat, SampleRateHz, SpeechLikeness, VadEvent,
};
use selene_kernel_contracts::ph1l::{NextAllowedActions, SessionId, SessionSnapshot};
use selene_kernel_contracts::ph1link::AppPlatform;
use selene_kernel_contracts::ph1position::TenantId;
use selene_kernel_contracts::ph1os::{OsOutcomeActionClass, OsOutcomeUtilizationEntry};
use selene_kernel_contracts::ph1pattern::{Ph1PatternRequest, Ph1PatternResponse};
use selene_kernel_contracts::ph1rll::{Ph1RllRequest, Ph1RllResponse};
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId, SchemaVersion, SessionState};
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
use selene_os::ph1x::{resolve_report_display_target, ReportDisplayResolution};
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

pub mod app_ui_assets {
    pub const APP_HTML: &str = include_str!("web/app.html");
    pub const APP_CSS: &str = include_str!("web/app.css");
    pub const APP_JS: &str = include_str!("web/app.js");
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
    pub user_text_partial: Option<String>,
    pub user_text_final: Option<String>,
    pub selene_text_partial: Option<String>,
    pub selene_text_final: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UiHealthCheckRow {
    pub check_id: String,
    pub label: String,
    pub status: String,
    pub open_issue_count: u32,
    pub last_event_at_ns: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UiHealthChecksResponse {
    pub status: String,
    pub generated_at_ns: u64,
    pub checks: Vec<UiHealthCheckRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UiHealthSummary {
    pub open_issues: u32,
    pub critical_open_count: u32,
    pub auto_resolved_24h_count: u32,
    pub escalated_24h_count: u32,
    pub mttr_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UiHealthIssueRow {
    pub issue_id: String,
    pub severity: String,
    pub issue_type: String,
    pub engine_owner: String,
    pub first_seen_at_ns: Option<u64>,
    pub last_update_at_ns: Option<u64>,
    pub status: String,
    pub resolution_state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UiHealthTimelineEntry {
    pub issue_id: String,
    pub at_ns: Option<u64>,
    pub action_id: String,
    pub result: String,
    pub reason_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct UiHealthDetailFilter {
    pub issue_query: Option<String>,
    pub engine_owner: Option<String>,
    pub open_only: bool,
    pub critical_only: bool,
    pub escalated_only: bool,
    pub from_utc_ns: Option<u64>,
    pub to_utc_ns: Option<u64>,
    pub selected_issue_id: Option<String>,
    pub timeline_page_size: Option<u16>,
    pub timeline_cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UiHealthTimelinePaging {
    pub has_next: bool,
    pub next_cursor: Option<String>,
    pub total_entries: u32,
    pub visible_entries: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UiHealthDetailResponse {
    pub status: String,
    pub generated_at_ns: u64,
    pub selected_check_id: String,
    pub selected_check_label: String,
    pub summary: UiHealthSummary,
    pub issues: Vec<UiHealthIssueRow>,
    pub active_issue_id: Option<String>,
    pub timeline: Vec<UiHealthTimelineEntry>,
    pub timeline_paging: UiHealthTimelinePaging,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UiHealthReportQueryRequest {
    pub correlation_id: Option<u64>,
    pub turn_id: Option<u64>,
    pub tenant_id: Option<String>,
    pub viewer_user_id: Option<String>,
    pub report_kind: Option<String>,
    pub from_utc_ns: Option<u64>,
    pub to_utc_ns: Option<u64>,
    pub engine_owner_filter: Option<String>,
    pub company_scope: Option<String>,
    pub company_ids: Option<Vec<String>>,
    pub country_codes: Option<Vec<String>>,
    pub escalated_only: Option<bool>,
    pub unresolved_only: Option<bool>,
    pub display_target: Option<String>,
    pub page_action: Option<String>,
    pub page_cursor: Option<String>,
    pub report_context_id: Option<String>,
    pub page_size: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UiHealthReportRow {
    pub tenant_id: String,
    pub issue_id: String,
    pub owner_engine_id: String,
    pub severity: String,
    pub status: String,
    pub latest_reason_code: String,
    pub last_seen_at_ns: u64,
    pub bcast_id: Option<String>,
    pub ack_state: Option<String>,
    pub issue_fingerprint: Option<String>,
    pub recurrence_observed: bool,
    pub impact_summary: Option<String>,
    pub attempted_fix_actions: Vec<String>,
    pub current_monitoring_evidence: Option<String>,
    pub unresolved_reason_exact: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UiHealthReportPaging {
    pub has_next: bool,
    pub has_prev: bool,
    pub next_cursor: Option<String>,
    pub prev_cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UiHealthReportQueryResponse {
    pub status: String,
    pub generated_at_ns: u64,
    pub reason_code: String,
    pub report_context_id: Option<String>,
    pub report_revision: Option<u64>,
    pub normalized_query: Option<String>,
    pub rows: Vec<UiHealthReportRow>,
    pub paging: UiHealthReportPaging,
    pub display_target_applied: Option<String>,
    pub remembered_display_target: Option<String>,
    pub requires_clarification: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UiTranscriptMessage {
    pub role: String,
    pub source: String,
    pub finalized: bool,
    pub text: String,
    pub timestamp_ns: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UiChatTranscriptResponse {
    pub status: String,
    pub generated_at_ns: u64,
    pub note: Option<String>,
    pub messages: Vec<UiTranscriptMessage>,
}

#[derive(Debug, Clone)]
pub struct AdapterRuntime {
    ingress: AppServerIngressRuntime,
    store: Arc<Mutex<Ph1fStore>>,
    sync_worker_counters: Arc<Mutex<AdapterSyncWorkerCounters>>,
    improvement_counters: Arc<Mutex<AdapterImprovementCounters>>,
    transcript_state: Arc<Mutex<AdapterTranscriptState>>,
    report_display_target_defaults: Arc<Mutex<BTreeMap<String, String>>>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum AdapterTranscriptRole {
    User,
    Selene,
}

impl AdapterTranscriptRole {
    fn as_str(self) -> &'static str {
        match self {
            AdapterTranscriptRole::User => "USER",
            AdapterTranscriptRole::Selene => "SELENE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum AdapterTranscriptSource {
    Ph1C,
    Ph1Write,
    UiText,
}

impl AdapterTranscriptSource {
    fn as_str(self) -> &'static str {
        match self {
            AdapterTranscriptSource::Ph1C => "PH1.C",
            AdapterTranscriptSource::Ph1Write => "PH1.WRITE",
            AdapterTranscriptSource::UiText => "UI.TEXT",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AdapterTranscriptEvent {
    seq: u64,
    correlation_id: CorrelationId,
    turn_id: TurnId,
    role: AdapterTranscriptRole,
    source: AdapterTranscriptSource,
    finalized: bool,
    text: String,
    timestamp_ns: u64,
}

impl AdapterTranscriptEvent {
    fn key(&self) -> AdapterTranscriptKey {
        AdapterTranscriptKey {
            correlation_id: self.correlation_id.0,
            turn_id: self.turn_id.0,
            role: self.role,
            source: self.source,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct AdapterTranscriptKey {
    correlation_id: u128,
    turn_id: u64,
    role: AdapterTranscriptRole,
    source: AdapterTranscriptSource,
}

#[derive(Debug, Clone)]
struct AdapterTranscriptState {
    next_seq: u64,
    events: Vec<AdapterTranscriptEvent>,
}

impl Default for AdapterTranscriptState {
    fn default() -> Self {
        Self {
            next_seq: 1,
            events: Vec::new(),
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
            transcript_state: Arc::new(Mutex::new(AdapterTranscriptState::default())),
            report_display_target_defaults: Arc::new(Mutex::new(BTreeMap::new())),
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
            transcript_state: Arc::new(Mutex::new(AdapterTranscriptState::default())),
            report_display_target_defaults: Arc::new(Mutex::new(BTreeMap::new())),
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
            transcript_state: Arc::new(Mutex::new(AdapterTranscriptState::default())),
            report_display_target_defaults: Arc::new(Mutex::new(BTreeMap::new())),
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

    pub fn ui_health_checks_report(
        &self,
        now_ns: Option<u64>,
    ) -> Result<UiHealthChecksResponse, String> {
        let now_ns = now_ns.unwrap_or_else(system_time_now_ns).max(1);
        let health = self.health_report(Some(now_ns))?;
        Ok(build_ui_health_checks_response(&health, now_ns))
    }

    pub fn ui_health_detail_report(
        &self,
        check_id: &str,
        now_ns: Option<u64>,
    ) -> Result<UiHealthDetailResponse, String> {
        self.ui_health_detail_report_filtered(check_id, UiHealthDetailFilter::default(), now_ns)
    }

    pub fn ui_health_detail_report_filtered(
        &self,
        check_id: &str,
        filter: UiHealthDetailFilter,
        now_ns: Option<u64>,
    ) -> Result<UiHealthDetailResponse, String> {
        if let (Some(from), Some(to)) = (filter.from_utc_ns, filter.to_utc_ns) {
            if from > to {
                return Err("invalid health detail date range: from_utc_ns is after to_utc_ns".to_string());
            }
        }
        let now_ns = now_ns.unwrap_or_else(system_time_now_ns).max(1);
        let health = self.health_report(Some(now_ns))?;
        let mut detail = build_ui_health_detail_response(&health, check_id, now_ns)?;
        detail.issues = filter_health_issues(&detail.issues, &filter);
        detail.active_issue_id = select_active_issue_id(&detail.issues, filter.selected_issue_id.as_deref());
        let active_issue = detail.active_issue_id.as_deref();
        let filtered_timeline = filter_timeline_for_issue(&detail.timeline, active_issue, &filter);
        let (timeline, timeline_paging) = page_timeline_entries(
            filtered_timeline,
            filter.timeline_page_size.unwrap_or(20),
            filter.timeline_cursor.as_deref(),
        )?;
        detail.timeline = timeline;
        detail.timeline_paging = timeline_paging;
        Ok(detail)
    }

    pub fn ui_chat_transcript_report(&self, now_ns: Option<u64>) -> UiChatTranscriptResponse {
        let now_ns = now_ns.unwrap_or_else(system_time_now_ns).max(1);
        let final_events = match self.store.lock() {
            Ok(store) => store
                .conversation_ledger()
                .iter()
                .filter_map(adapter_transcript_event_from_record)
                .collect::<Vec<_>>(),
            Err(_) => {
                return UiChatTranscriptResponse {
                    status: "error".to_string(),
                    generated_at_ns: now_ns,
                    note: Some("adapter store lock poisoned".to_string()),
                    messages: Vec::new(),
                };
            }
        };
        let partial_events = match self.transcript_state.lock() {
            Ok(state) => state.events.clone(),
            Err(_) => {
                return UiChatTranscriptResponse {
                    status: "error".to_string(),
                    generated_at_ns: now_ns,
                    note: Some("adapter transcript lock poisoned".to_string()),
                    messages: Vec::new(),
                };
            }
        };

        let mut final_by_key: BTreeMap<AdapterTranscriptKey, AdapterTranscriptEvent> =
            BTreeMap::new();
        for event in final_events {
            let key = event.key();
            if let Some(existing) = final_by_key.get(&key) {
                if existing.timestamp_ns >= event.timestamp_ns {
                    continue;
                }
            }
            final_by_key.insert(key, event);
        }

        let mut partial_by_key: BTreeMap<AdapterTranscriptKey, AdapterTranscriptEvent> =
            BTreeMap::new();
        for event in partial_events {
            if event.finalized {
                continue;
            }
            let key = event.key();
            if final_by_key.contains_key(&key) {
                continue;
            }
            if let Some(existing) = partial_by_key.get(&key) {
                if existing.seq >= event.seq {
                    continue;
                }
            }
            partial_by_key.insert(key, event);
        }

        let mut ordered = Vec::new();
        for (_, event) in final_by_key {
            ordered.push((
                event.timestamp_ns,
                event.correlation_id.0,
                event.turn_id.0,
                event.role,
                UiTranscriptMessage {
                    role: event.role.as_str().to_string(),
                    source: event.source.as_str().to_string(),
                    finalized: true,
                    text: event.text,
                    timestamp_ns: event.timestamp_ns,
                },
            ));
        }
        for (_, event) in partial_by_key {
            ordered.push((
                event.timestamp_ns,
                event.correlation_id.0,
                event.turn_id.0,
                event.role,
                UiTranscriptMessage {
                    role: event.role.as_str().to_string(),
                    source: event.source.as_str().to_string(),
                    finalized: false,
                    text: event.text,
                    timestamp_ns: event.timestamp_ns,
                },
            ));
        }

        ordered.sort_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then_with(|| left.1.cmp(&right.1))
                .then_with(|| left.2.cmp(&right.2))
                .then_with(|| left.3.cmp(&right.3))
        });
        let messages = ordered
            .into_iter()
            .map(|(_, _, _, _, msg)| msg)
            .collect::<Vec<_>>();
        let note = if messages.is_empty() {
            Some("No transcript messages yet.".to_string())
        } else {
            None
        };

        UiChatTranscriptResponse {
            status: "ok".to_string(),
            generated_at_ns: now_ns,
            note,
            messages,
        }
    }

    pub fn ui_health_report_query(
        &self,
        request: UiHealthReportQueryRequest,
        now_ns: Option<u64>,
    ) -> UiHealthReportQueryResponse {
        let now_ns = now_ns.unwrap_or_else(system_time_now_ns).max(1);
        let viewer_user_id = request
            .viewer_user_id
            .clone()
            .unwrap_or_else(|| "viewer_01".to_string());

        let remembered_target = self
            .report_display_target_defaults
            .lock()
            .ok()
            .and_then(|m| m.get(&viewer_user_id).cloned());

        let display_resolution =
            resolve_report_display_target(request.display_target.as_deref(), remembered_target.as_deref());
        let display_target_applied = match display_resolution {
            ReportDisplayResolution::Resolved(target) => target.as_str().to_string(),
            ReportDisplayResolution::Clarify(question) => {
                return UiHealthReportQueryResponse {
                    status: "ok".to_string(),
                    generated_at_ns: now_ns,
                    reason_code: health_reason_codes::PH1_HEALTH_DISPLAY_TARGET_REQUIRED
                        .0
                        .to_string(),
                    report_context_id: None,
                    report_revision: None,
                    normalized_query: None,
                    rows: Vec::new(),
                    paging: UiHealthReportPaging {
                        has_next: false,
                        has_prev: false,
                        next_cursor: None,
                        prev_cursor: None,
                    },
                    display_target_applied: None,
                    remembered_display_target: remembered_target,
                    requires_clarification: Some(question),
                };
            }
        };

        if let Ok(mut remembered) = self.report_display_target_defaults.lock() {
            remembered.insert(viewer_user_id, display_target_applied.clone());
        }

        let health = match self.health_report(Some(now_ns)) {
            Ok(v) => v,
            Err(err) => {
                return UiHealthReportQueryResponse {
                    status: "error".to_string(),
                    generated_at_ns: now_ns,
                    reason_code: health_reason_codes::PH1_HEALTH_INTERNAL_PIPELINE_ERROR
                        .0
                        .to_string(),
                    report_context_id: None,
                    report_revision: None,
                    normalized_query: None,
                    rows: Vec::new(),
                    paging: UiHealthReportPaging {
                        has_next: false,
                        has_prev: false,
                        next_cursor: None,
                        prev_cursor: None,
                    },
                    display_target_applied: Some(display_target_applied),
                    remembered_display_target: remembered_target,
                    requires_clarification: Some(err),
                };
            }
        };

        let tenant_id = match parse_tenant_id(request.tenant_id.as_deref()) {
            Ok(v) => v,
            Err(reason) => {
                return UiHealthReportQueryResponse {
                    status: "error".to_string(),
                    generated_at_ns: now_ns,
                    reason_code: health_reason_codes::PH1_HEALTH_INPUT_SCHEMA_INVALID
                        .0
                        .to_string(),
                    report_context_id: None,
                    report_revision: None,
                    normalized_query: None,
                    rows: Vec::new(),
                    paging: UiHealthReportPaging {
                        has_next: false,
                        has_prev: false,
                        next_cursor: None,
                        prev_cursor: None,
                    },
                    display_target_applied: Some(display_target_applied),
                    remembered_display_target: remembered_target,
                    requires_clarification: Some(reason),
                };
            }
        };

        let from_ns = request
            .from_utc_ns
            .unwrap_or(now_ns.saturating_sub(30 * 24 * 60 * 60 * 1_000_000_000));
        let to_ns = request.to_utc_ns.unwrap_or(now_ns);
        let time_range = match HealthReportTimeRange::v1(MonotonicTimeNs(from_ns), MonotonicTimeNs(to_ns)) {
            Ok(v) => v,
            Err(_) => {
                return UiHealthReportQueryResponse {
                    status: "error".to_string(),
                    generated_at_ns: now_ns,
                    reason_code: health_reason_codes::PH1_HEALTH_DATE_RANGE_INVALID
                        .0
                        .to_string(),
                    report_context_id: None,
                    report_revision: None,
                    normalized_query: None,
                    rows: Vec::new(),
                    paging: UiHealthReportPaging {
                        has_next: false,
                        has_prev: false,
                        next_cursor: None,
                        prev_cursor: None,
                    },
                    display_target_applied: Some(display_target_applied),
                    remembered_display_target: remembered_target,
                    requires_clarification: Some("Invalid date range.".to_string()),
                };
            }
        };

        let envelope = match HealthReadEnvelope::v1(
            CorrelationId(request.correlation_id.unwrap_or(now_ns) as u128),
            TurnId(request.turn_id.unwrap_or(1)),
            MonotonicTimeNs(now_ns),
        ) {
            Ok(v) => v,
            Err(_) => {
                return UiHealthReportQueryResponse {
                    status: "error".to_string(),
                    generated_at_ns: now_ns,
                    reason_code: health_reason_codes::PH1_HEALTH_INPUT_SCHEMA_INVALID
                        .0
                        .to_string(),
                    report_context_id: None,
                    report_revision: None,
                    normalized_query: None,
                    rows: Vec::new(),
                    paging: UiHealthReportPaging {
                        has_next: false,
                        has_prev: false,
                        next_cursor: None,
                        prev_cursor: None,
                    },
                    display_target_applied: Some(display_target_applied),
                    remembered_display_target: remembered_target,
                    requires_clarification: Some("Invalid report envelope.".to_string()),
                };
            }
        };

        let issue_events = synth_health_issue_events(&health, &tenant_id, now_ns);
        let report_request = HealthReportQueryReadRequest::v1(
            envelope,
            tenant_id,
            request
                .viewer_user_id
                .clone()
                .unwrap_or_else(|| "viewer_01".to_string()),
            parse_report_kind(request.report_kind.as_deref()),
            time_range,
            request.engine_owner_filter.clone(),
            parse_company_scope(request.company_scope.as_deref()),
            parse_company_ids(request.company_ids.as_ref()),
            parse_country_codes(request.country_codes.as_ref()),
            request.escalated_only.unwrap_or(false),
            request.unresolved_only.unwrap_or(false),
            Some(parse_health_display_target(&display_target_applied)),
            parse_page_action(request.page_action.as_deref()),
            request.page_cursor.clone(),
            request.report_context_id.clone(),
            request.page_size.unwrap_or(25),
            issue_events,
        );

        let report_request = match report_request {
            Ok(v) => v,
            Err(err) => {
                return UiHealthReportQueryResponse {
                    status: "error".to_string(),
                    generated_at_ns: now_ns,
                    reason_code: health_reason_codes::PH1_HEALTH_INPUT_SCHEMA_INVALID
                        .0
                        .to_string(),
                    report_context_id: None,
                    report_revision: None,
                    normalized_query: None,
                    rows: Vec::new(),
                    paging: UiHealthReportPaging {
                        has_next: false,
                        has_prev: false,
                        next_cursor: None,
                        prev_cursor: None,
                    },
                    display_target_applied: Some(display_target_applied),
                    remembered_display_target: remembered_target,
                    requires_clarification: Some(format!("Invalid report request: {err:?}")),
                };
            }
        };

        let engine = EngineHealthRuntime::new(EngineHealthConfig::mvp_v1());
        let outcome = engine.run(&Ph1HealthRequest::HealthReportQueryRead(report_request));
        match outcome {
            Ph1HealthResponse::HealthReportQueryReadOk(ok) => {
                map_health_report_ok(ok, now_ns, remembered_target)
            }
            Ph1HealthResponse::Refuse(refuse) => UiHealthReportQueryResponse {
                status: "error".to_string(),
                generated_at_ns: now_ns,
                reason_code: refuse.reason_code.0.to_string(),
                report_context_id: None,
                report_revision: None,
                normalized_query: None,
                rows: Vec::new(),
                paging: UiHealthReportPaging {
                    has_next: false,
                    has_prev: false,
                    next_cursor: None,
                    prev_cursor: None,
                },
                display_target_applied: Some(display_target_applied),
                remembered_display_target: remembered_target,
                requires_clarification: Some(refuse.message),
            },
            _ => UiHealthReportQueryResponse {
                status: "error".to_string(),
                generated_at_ns: now_ns,
                reason_code: health_reason_codes::PH1_HEALTH_INTERNAL_PIPELINE_ERROR
                    .0
                    .to_string(),
                report_context_id: None,
                report_revision: None,
                normalized_query: None,
                rows: Vec::new(),
                paging: UiHealthReportPaging {
                    has_next: false,
                    has_prev: false,
                    next_cursor: None,
                    prev_cursor: None,
                },
                display_target_applied: Some(display_target_applied),
                remembered_display_target: remembered_target,
                requires_clarification: Some("Unexpected health report response.".to_string()),
            },
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn record_transcript_updates(
        &self,
        store: &mut Ph1fStore,
        now: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        actor_user_id: &UserId,
        device_id: Option<&DeviceId>,
        user_text_partial: Option<String>,
        user_text_final: Option<String>,
        selene_text_partial: Option<String>,
        selene_text_final: Option<String>,
    ) -> Result<(), String> {
        if let Some(text) = user_text_partial {
            self.push_transcript_partial_event(
                correlation_id,
                turn_id,
                AdapterTranscriptRole::User,
                AdapterTranscriptSource::Ph1C,
                text,
                now.0,
            )?;
        }
        if let Some(text) = selene_text_partial {
            self.push_transcript_partial_event(
                correlation_id,
                turn_id,
                AdapterTranscriptRole::Selene,
                AdapterTranscriptSource::Ph1Write,
                text,
                now.0,
            )?;
        }
        if let Some(text) = user_text_final {
            append_transcript_final_conversation_turn(
                store,
                now,
                correlation_id,
                turn_id,
                actor_user_id,
                device_id,
                ConversationRole::User,
                ConversationSource::VoiceTranscript,
                &text,
            )?;
            self.clear_transcript_partials_for_key(
                correlation_id,
                turn_id,
                AdapterTranscriptRole::User,
                AdapterTranscriptSource::Ph1C,
            )?;
        }
        if let Some(text) = selene_text_final {
            append_transcript_final_conversation_turn(
                store,
                now,
                correlation_id,
                turn_id,
                actor_user_id,
                device_id,
                ConversationRole::Selene,
                ConversationSource::SeleneOutput,
                &text,
            )?;
            self.clear_transcript_partials_for_key(
                correlation_id,
                turn_id,
                AdapterTranscriptRole::Selene,
                AdapterTranscriptSource::Ph1Write,
            )?;
        }
        Ok(())
    }

    fn push_transcript_partial_event(
        &self,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        role: AdapterTranscriptRole,
        source: AdapterTranscriptSource,
        text: String,
        timestamp_ns: u64,
    ) -> Result<(), String> {
        let mut state = self
            .transcript_state
            .lock()
            .map_err(|_| "adapter transcript lock poisoned".to_string())?;
        let seq = state.next_seq;
        state.next_seq = state.next_seq.saturating_add(1);
        state.events.push(AdapterTranscriptEvent {
            seq,
            correlation_id,
            turn_id,
            role,
            source,
            finalized: false,
            text,
            timestamp_ns,
        });
        const MAX_TRANSCRIPT_EVENTS: usize = 4096;
        if state.events.len() > MAX_TRANSCRIPT_EVENTS {
            let drop_count = state.events.len().saturating_sub(MAX_TRANSCRIPT_EVENTS);
            state.events.drain(0..drop_count);
        }
        Ok(())
    }

    fn clear_transcript_partials_for_key(
        &self,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        role: AdapterTranscriptRole,
        source: AdapterTranscriptSource,
    ) -> Result<(), String> {
        let key = AdapterTranscriptKey {
            correlation_id: correlation_id.0,
            turn_id: turn_id.0,
            role,
            source,
        };
        let mut state = self
            .transcript_state
            .lock()
            .map_err(|_| "adapter transcript lock poisoned".to_string())?;
        state
            .events
            .retain(|event| event.finalized || event.key() != key);
        Ok(())
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
        let user_text_partial = sanitize_transcript_text_option(request.user_text_partial.clone());
        let user_text_final = sanitize_transcript_text_option(request.user_text_final.clone());
        let selene_text_partial =
            sanitize_transcript_text_option(request.selene_text_partial.clone());
        let selene_text_final = sanitize_transcript_text_option(request.selene_text_final.clone());
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
            actor_user_id.clone(),
            request.tenant_id.clone(),
            device_id.clone(),
            Vec::new(),
            empty_observation(),
        )
        .map_err(|err| format!("invalid ingress request: {err:?}"))?;
        let outcome = self
            .ingress
            .run_voice_turn(&mut store, ingress_request)
            .map_err(storage_error_to_string)?;
        self.record_transcript_updates(
            &mut store,
            now,
            correlation_id,
            turn_id,
            &actor_user_id,
            device_id.as_ref(),
            user_text_partial,
            user_text_final,
            selene_text_partial,
            selene_text_final,
        )?;
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
            .unwrap_or_else(default_adapter_store_path);
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

fn sanitize_transcript_text_option(value: Option<String>) -> Option<String> {
    value
        .map(|v| truncate_ascii(v.trim(), 8192))
        .filter(|v| !v.trim().is_empty())
}

fn adapter_transcript_role_from_storage(role: ConversationRole) -> AdapterTranscriptRole {
    match role {
        ConversationRole::User => AdapterTranscriptRole::User,
        ConversationRole::Selene => AdapterTranscriptRole::Selene,
    }
}

fn adapter_transcript_source_from_storage(
    source: ConversationSource,
) -> Option<AdapterTranscriptSource> {
    match source {
        ConversationSource::VoiceTranscript => Some(AdapterTranscriptSource::Ph1C),
        ConversationSource::SeleneOutput => Some(AdapterTranscriptSource::Ph1Write),
        ConversationSource::TypedText => Some(AdapterTranscriptSource::UiText),
        ConversationSource::Tombstone => None,
    }
}

fn adapter_transcript_event_from_record(
    record: &selene_kernel_contracts::ph1f::ConversationTurnRecord,
) -> Option<AdapterTranscriptEvent> {
    let source = adapter_transcript_source_from_storage(record.source)?;
    Some(AdapterTranscriptEvent {
        seq: record.conversation_turn_id.0,
        correlation_id: record.correlation_id,
        turn_id: record.turn_id,
        role: adapter_transcript_role_from_storage(record.role),
        source,
        finalized: true,
        text: record.text.clone(),
        timestamp_ns: record.created_at.0,
    })
}

#[allow(clippy::too_many_arguments)]
fn append_transcript_final_conversation_turn(
    store: &mut Ph1fStore,
    now: MonotonicTimeNs,
    correlation_id: CorrelationId,
    turn_id: TurnId,
    actor_user_id: &UserId,
    device_id: Option<&DeviceId>,
    role: ConversationRole,
    source: ConversationSource,
    text: &str,
) -> Result<(), String> {
    let text = truncate_ascii(text.trim(), 8192);
    if text.is_empty() {
        return Ok(());
    }
    let idempotency_key = sanitize_idempotency_token(&format!(
        "adapter_transcript:{}:{}:{}:{}",
        correlation_id.0,
        turn_id.0,
        match role {
            ConversationRole::User => "USER",
            ConversationRole::Selene => "SELENE",
        },
        match source {
            ConversationSource::VoiceTranscript => "PH1.C",
            ConversationSource::TypedText => "UI.TEXT",
            ConversationSource::SeleneOutput => "PH1.WRITE",
            ConversationSource::Tombstone => "TOMBSTONE",
        }
    ));
    let input = ConversationTurnInput::v1(
        now,
        correlation_id,
        turn_id,
        None,
        actor_user_id.clone(),
        device_id.cloned(),
        role,
        source,
        text.clone(),
        stable_hash_hex_16(&text),
        PrivacyScope::PublicChat,
        Some(idempotency_key),
        None,
        None,
    )
    .map_err(|err| format!("invalid transcript conversation input: {err:?}"))?;
    let _ = store
        .append_conversation_turn(input)
        .map_err(storage_error_to_string)?;
    Ok(())
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

fn default_adapter_store_path() -> PathBuf {
    if let Ok(home) = env::var("HOME") {
        let home = home.trim();
        if !home.is_empty() {
            return PathBuf::from(home).join(".selene/adapter/voice_turns.jsonl");
        }
    }
    PathBuf::from(".selene/adapter/voice_turns.jsonl")
}

fn parse_tenant_id(raw: Option<&str>) -> Result<TenantId, String> {
    let tenant = raw.unwrap_or("tenant_a").trim();
    TenantId::new(tenant.to_string()).map_err(|err| format!("invalid tenant_id: {err:?}"))
}

fn parse_report_kind(raw: Option<&str>) -> HealthReportKind {
    match raw.unwrap_or("UNRESOLVED_ESCALATED").trim().to_ascii_uppercase().as_str() {
        "MISSED_STT" => HealthReportKind::MissedStt,
        "ISSUE_STATUS" => HealthReportKind::IssueStatus,
        _ => HealthReportKind::UnresolvedEscalated,
    }
}

fn parse_company_scope(raw: Option<&str>) -> HealthCompanyScope {
    match raw
        .unwrap_or("TENANT_ONLY")
        .trim()
        .to_ascii_uppercase()
        .as_str()
    {
        "CROSS_TENANT_TENANT_ROWS" => HealthCompanyScope::CrossTenantTenantRows,
        _ => HealthCompanyScope::TenantOnly,
    }
}

fn parse_page_action(raw: Option<&str>) -> HealthPageAction {
    match raw.unwrap_or("FIRST").trim().to_ascii_uppercase().as_str() {
        "NEXT" => HealthPageAction::Next,
        "PREV" => HealthPageAction::Prev,
        "REFRESH" => HealthPageAction::Refresh,
        _ => HealthPageAction::First,
    }
}

fn parse_health_display_target(raw: &str) -> HealthDisplayTarget {
    match raw.trim().to_ascii_lowercase().as_str() {
        "phone" => HealthDisplayTarget::Phone,
        _ => HealthDisplayTarget::Desktop,
    }
}

fn parse_company_ids(raw: Option<&Vec<String>>) -> Vec<TenantId> {
    let Some(values) = raw else {
        return Vec::new();
    };
    values
        .iter()
        .filter_map(|tenant| TenantId::new(tenant.trim().to_string()).ok())
        .collect()
}

fn parse_country_codes(raw: Option<&Vec<String>>) -> Vec<String> {
    let Some(values) = raw else {
        return Vec::new();
    };
    values
        .iter()
        .map(|code| code.trim().to_ascii_uppercase())
        .filter(|code| !code.is_empty())
        .collect()
}

fn ack_state_label(state: Option<HealthAckState>) -> Option<String> {
    state.map(|value| match value {
        HealthAckState::Waiting => "WAITING".to_string(),
        HealthAckState::Acknowledged => "ACKNOWLEDGED".to_string(),
        HealthAckState::FollowupPending => "FOLLOWUP_PENDING".to_string(),
    })
}

fn map_health_report_ok(
    ok: HealthReportQueryReadOk,
    generated_at_ns: u64,
    remembered_display_target: Option<String>,
) -> UiHealthReportQueryResponse {
    let rows = ok
        .rows
        .into_iter()
        .map(|row| UiHealthReportRow {
            tenant_id: row.tenant_id.as_str().to_string(),
            issue_id: row.issue_id,
            owner_engine_id: row.owner_engine_id,
            severity: format!("{:?}", row.severity).to_ascii_uppercase(),
            status: format!("{:?}", row.status).to_ascii_uppercase(),
            latest_reason_code: row.latest_reason_code.0.to_string(),
            last_seen_at_ns: row.last_seen_at.0,
            bcast_id: row.bcast_id,
            ack_state: ack_state_label(row.ack_state),
            issue_fingerprint: row.issue_fingerprint,
            recurrence_observed: row.recurrence_observed,
            impact_summary: row.impact_summary,
            attempted_fix_actions: row.attempted_fix_actions,
            current_monitoring_evidence: row.current_monitoring_evidence,
            unresolved_reason_exact: row.unresolved_reason_exact,
        })
        .collect::<Vec<_>>();
    let display_target_applied = ok.display_target_applied.map(|target| match target {
        HealthDisplayTarget::Desktop => "desktop".to_string(),
        HealthDisplayTarget::Phone => "phone".to_string(),
    });

    UiHealthReportQueryResponse {
        status: "ok".to_string(),
        generated_at_ns,
        reason_code: ok.reason_code.0.to_string(),
        report_context_id: Some(ok.report_context_id),
        report_revision: Some(ok.report_revision),
        normalized_query: Some(ok.normalized_query),
        rows,
        paging: UiHealthReportPaging {
            has_next: ok.paging.has_next,
            has_prev: ok.paging.has_prev,
            next_cursor: ok.paging.next_cursor,
            prev_cursor: ok.paging.prev_cursor,
        },
        display_target_applied,
        remembered_display_target,
        requires_clarification: ok.requires_clarification,
    }
}

fn synth_health_issue_events(
    health: &AdapterHealthResponse,
    tenant: &TenantId,
    now_ns: u64,
) -> Vec<HealthIssueEvent> {
    let mut out = Vec::new();

    fn add_event(
        out: &mut Vec<HealthIssueEvent>,
        tenant: &TenantId,
        now_ns: u64,
        issue_id: &str,
        engine_owner_id: &str,
        severity: HealthSeverity,
        status: HealthIssueStatus,
        reason_code: ReasonCodeId,
        bcast_id: Option<String>,
        ack_state: Option<HealthAckState>,
        impact_summary: Option<String>,
        attempted_fix_actions: Vec<String>,
        current_monitoring_evidence: Option<String>,
        unresolved_reason_exact: Option<String>,
        issue_fingerprint: Option<String>,
        recurrence_observed: Option<bool>,
    ) {
        let seed_status = if status == HealthIssueStatus::Escalated {
            HealthIssueStatus::Open
        } else {
            status
        };
        let base = HealthIssueEvent::v1(
            tenant.clone(),
            issue_id.to_string(),
            engine_owner_id.to_string(),
            severity,
            seed_status,
            format!("ACTION_{}", issue_id.to_ascii_uppercase()),
            if status == HealthIssueStatus::Resolved {
                HealthActionResult::Pass
            } else {
                HealthActionResult::Retry
            },
            1,
            reason_code,
            MonotonicTimeNs(now_ns.saturating_sub(1_000_000_000)),
            if status == HealthIssueStatus::Resolved {
                Some(MonotonicTimeNs(now_ns))
            } else {
                None
            },
            Some(MonotonicTimeNs(now_ns.saturating_add(15 * 60 * 1_000_000_000))),
            None,
            None,
        );
        let Ok(mut base) = base else {
            return;
        };
        base.status = status;
        base.bcast_id = bcast_id;
        base.ack_state = ack_state;
        let with_payload = base
            .clone()
            .with_escalation_payload(
                impact_summary,
                attempted_fix_actions,
                current_monitoring_evidence,
                unresolved_reason_exact,
            )
            .unwrap_or(base);
        let full = with_payload
            .clone()
            .with_resolution_proof(
                issue_fingerprint,
                Some(MonotonicTimeNs(now_ns.saturating_sub(5 * 60 * 1_000_000_000))),
                Some(MonotonicTimeNs(now_ns)),
                recurrence_observed,
            )
            .unwrap_or(with_payload);
        out.push(full);
    }

    if health.sync.queue.dead_letter_count > 0 {
        add_event(
            &mut out,
            tenant,
            now_ns,
            "sync_dead_letter",
            "PH1.OS",
            HealthSeverity::Critical,
            HealthIssueStatus::Escalated,
            reason_codes::ADAPTER_SYNC_DEADLETTER,
            Some("bcast_sync_dead_letter".to_string()),
            Some(HealthAckState::Waiting),
            Some("Critical sync dead letters are blocking artifact continuity.".to_string()),
            vec!["retry queued artifacts".to_string()],
            Some(format!(
                "dead_letter_count={} replay_due_count={}",
                health.sync.queue.dead_letter_count, health.sync.queue.replay_due_count
            )),
            Some("dead letters remain after retry budget".to_string()),
            Some("sync_dead_letter_fingerprint".to_string()),
            Some(true),
        );
    }

    if health.sync.queue.retry_pending_count > 0 {
        add_event(
            &mut out,
            tenant,
            now_ns,
            "sync_retry_backlog",
            "PH1.OS",
            HealthSeverity::Warn,
            HealthIssueStatus::Open,
            reason_codes::ADAPTER_SYNC_RETRY,
            None,
            None,
            Some("Retry queue backlog is above zero.".to_string()),
            vec!["retry worker pass".to_string()],
            Some(format!("retry_pending_count={}", health.sync.queue.retry_pending_count)),
            Some("retry queue has not drained yet".to_string()),
            Some("sync_retry_fingerprint".to_string()),
            Some(true),
        );
    }

    if health.sync.queue.replay_due_count > 0 {
        add_event(
            &mut out,
            tenant,
            now_ns,
            "sync_replay_due",
            "PH1.OS",
            HealthSeverity::Critical,
            HealthIssueStatus::Open,
            reason_codes::ADAPTER_SYNC_REPLAY_DUE,
            None,
            None,
            Some("Replay-due jobs exceeded threshold.".to_string()),
            vec!["replay scan".to_string()],
            Some(format!("replay_due_count={}", health.sync.queue.replay_due_count)),
            Some("replay-due jobs remain unresolved".to_string()),
            Some("sync_replay_due_fingerprint".to_string()),
            Some(true),
        );
    }

    if out.is_empty() {
        add_event(
            &mut out,
            tenant,
            now_ns,
            "health_nominal",
            "PH1.HEALTH",
            HealthSeverity::Info,
            HealthIssueStatus::Resolved,
            ReasonCodeId(health_reason_codes::PH1_HEALTH_OK_REPORT_QUERY_READ.0),
            None,
            Some(HealthAckState::Acknowledged),
            Some("No active unresolved health issues.".to_string()),
            vec!["daily health scan".to_string()],
            Some("no recurrence detected".to_string()),
            Some("resolved by live verification".to_string()),
            Some("health_nominal_fingerprint".to_string()),
            Some(false),
        );
    }

    out
}

const UI_HEALTH_CHECKS: [(&str, &str); 8] = [
    ("VOICE", "Voice"),
    ("WAKE", "Wake"),
    ("SYNC", "Sync"),
    ("STT", "STT"),
    ("TTS", "TTS"),
    ("DELIVERY", "Delivery"),
    ("BUILDER", "Builder"),
    ("MEMORY", "Memory"),
];

fn build_ui_health_checks_response(
    health: &AdapterHealthResponse,
    generated_at_ns: u64,
) -> UiHealthChecksResponse {
    let sync_open = health
        .sync
        .queue
        .retry_pending_count
        .saturating_add(health.sync.queue.dead_letter_count)
        .saturating_add(health.sync.queue.replay_due_count);
    let sync_status =
        if health.sync.queue.dead_letter_count > 0 || health.sync.queue.replay_due_count > 0 {
            "CRITICAL"
        } else if sync_open > 0
            || health.sync.queue.in_flight_count > 0
            || health.sync.queue.queued_count > 0
        {
            "AT_RISK"
        } else {
            "HEALTHY"
        };
    let builder_status = builder_health_status(health);
    let builder_open = if builder_status == "HEALTHY" { 0 } else { 1 };

    let checks = UI_HEALTH_CHECKS
        .iter()
        .map(|(check_id, label)| {
            let (status, open_issue_count, last_event_at_ns) = match *check_id {
                "SYNC" => (
                    sync_status.to_string(),
                    sync_open,
                    health.sync.worker.last_pass_at_ns,
                ),
                "BUILDER" => (
                    builder_status.to_string(),
                    builder_open,
                    health.sync.worker.last_pass_at_ns,
                ),
                _ => ("HEALTHY".to_string(), 0, health.sync.worker.last_pass_at_ns),
            };
            UiHealthCheckRow {
                check_id: (*check_id).to_string(),
                label: (*label).to_string(),
                status,
                open_issue_count,
                last_event_at_ns,
            }
        })
        .collect::<Vec<_>>();

    UiHealthChecksResponse {
        status: "ok".to_string(),
        generated_at_ns,
        checks,
    }
}

fn build_ui_health_detail_response(
    health: &AdapterHealthResponse,
    check_id: &str,
    generated_at_ns: u64,
) -> Result<UiHealthDetailResponse, String> {
    let Some((normalized, label)) = normalize_ui_health_check_id(check_id) else {
        return Err(format!(
            "invalid health check id '{}'; expected one of VOICE|WAKE|SYNC|STT|TTS|DELIVERY|BUILDER|MEMORY",
            check_id
        ));
    };
    let (summary, issues, timeline) = match normalized {
        "SYNC" => build_sync_detail(health),
        "BUILDER" => build_builder_detail(health),
        _ => (
            UiHealthSummary {
                open_issues: 0,
                critical_open_count: 0,
                auto_resolved_24h_count: 0,
                escalated_24h_count: 0,
                mttr_ms: None,
            },
            Vec::new(),
            Vec::new(),
        ),
    };

    let active_issue_id = issues.first().map(|issue| issue.issue_id.clone());
    let timeline_count = timeline.len().min(u32::MAX as usize) as u32;
    Ok(UiHealthDetailResponse {
        status: "ok".to_string(),
        generated_at_ns,
        selected_check_id: normalized.to_string(),
        selected_check_label: label.to_string(),
        summary,
        issues,
        active_issue_id,
        timeline,
        timeline_paging: UiHealthTimelinePaging {
            has_next: false,
            next_cursor: None,
            total_entries: timeline_count,
            visible_entries: timeline_count,
        },
    })
}

fn filter_health_issues(issues: &[UiHealthIssueRow], filter: &UiHealthDetailFilter) -> Vec<UiHealthIssueRow> {
    let query = filter
        .issue_query
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase());
    let owner = filter
        .engine_owner
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase());

    issues
        .iter()
        .filter(|issue| {
            if filter.open_only {
                let is_open = issue.status == "OPEN"
                    || issue.status == "ESCALATED"
                    || issue.resolution_state == "UNRESOLVED";
                if !is_open {
                    return false;
                }
            }
            if filter.critical_only && issue.severity != "CRITICAL" {
                return false;
            }
            if filter.escalated_only && issue.status != "ESCALATED" {
                return false;
            }
            if let Some(owner_filter) = owner.as_deref() {
                if !issue.engine_owner.to_ascii_lowercase().contains(owner_filter) {
                    return false;
                }
            }
            if let Some(query_filter) = query.as_deref() {
                let haystack = format!(
                    "{} {} {}",
                    issue.issue_id, issue.issue_type, issue.engine_owner
                )
                .to_ascii_lowercase();
                if !haystack.contains(query_filter) {
                    return false;
                }
            }
            let issue_time = issue.last_update_at_ns.unwrap_or(0);
            if let Some(from) = filter.from_utc_ns {
                if issue_time < from {
                    return false;
                }
            }
            if let Some(to) = filter.to_utc_ns {
                if issue_time > to {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect::<Vec<_>>()
}

fn select_active_issue_id(
    issues: &[UiHealthIssueRow],
    requested_issue_id: Option<&str>,
) -> Option<String> {
    if let Some(requested) = requested_issue_id {
        if issues.iter().any(|issue| issue.issue_id == requested) {
            return Some(requested.to_string());
        }
    }
    issues.first().map(|issue| issue.issue_id.clone())
}

fn filter_timeline_for_issue(
    timeline: &[UiHealthTimelineEntry],
    active_issue_id: Option<&str>,
    filter: &UiHealthDetailFilter,
) -> Vec<UiHealthTimelineEntry> {
    let mut out = timeline
        .iter()
        .filter(|entry| {
            if let Some(issue_id) = active_issue_id {
                if entry.issue_id != issue_id {
                    return false;
                }
            }
            let at_ns = entry.at_ns.unwrap_or(0);
            if let Some(from) = filter.from_utc_ns {
                if at_ns < from {
                    return false;
                }
            }
            if let Some(to) = filter.to_utc_ns {
                if at_ns > to {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect::<Vec<_>>();
    out.sort_by(|left, right| {
        right
            .at_ns
            .unwrap_or(0)
            .cmp(&left.at_ns.unwrap_or(0))
            .then_with(|| left.issue_id.cmp(&right.issue_id))
            .then_with(|| left.action_id.cmp(&right.action_id))
    });
    out
}

fn parse_timeline_cursor(cursor: Option<&str>) -> Result<usize, String> {
    let Some(cursor) = cursor else {
        return Ok(0);
    };
    let (prefix, value) = cursor
        .split_once(':')
        .ok_or_else(|| "invalid health detail timeline cursor format".to_string())?;
    if prefix != "idx" {
        return Err("invalid health detail timeline cursor prefix".to_string());
    }
    value
        .parse::<usize>()
        .map_err(|_| "invalid health detail timeline cursor value".to_string())
}

fn page_timeline_entries(
    timeline: Vec<UiHealthTimelineEntry>,
    page_size: u16,
    cursor: Option<&str>,
) -> Result<(Vec<UiHealthTimelineEntry>, UiHealthTimelinePaging), String> {
    let total = timeline.len();
    let page_size = page_size.clamp(1, 200) as usize;
    let start = parse_timeline_cursor(cursor)?;
    let start = start.min(total);
    let end = start.saturating_add(page_size).min(total);
    let slice = timeline[start..end].to_vec();
    let has_next = end < total;
    let next_cursor = if has_next {
        Some(format!("idx:{end}"))
    } else {
        None
    };
    Ok((
        slice,
        UiHealthTimelinePaging {
            has_next,
            next_cursor,
            total_entries: total.min(u32::MAX as usize) as u32,
            visible_entries: end.saturating_sub(start).min(u32::MAX as usize) as u32,
        },
    ))
}

fn normalize_ui_health_check_id(raw: &str) -> Option<(&'static str, &'static str)> {
    let normalized = raw.trim().to_ascii_uppercase();
    UI_HEALTH_CHECKS
        .iter()
        .find(|(check_id, _)| *check_id == normalized)
        .copied()
}

fn builder_health_status(health: &AdapterHealthResponse) -> &'static str {
    let last = health
        .sync
        .improvement
        .last_builder_status
        .as_deref()
        .unwrap_or("");
    if last.starts_with("ERROR") {
        "CRITICAL"
    } else if last.starts_with("REFUSED") {
        "AT_RISK"
    } else {
        "HEALTHY"
    }
}

fn build_sync_detail(
    health: &AdapterHealthResponse,
) -> (
    UiHealthSummary,
    Vec<UiHealthIssueRow>,
    Vec<UiHealthTimelineEntry>,
) {
    let mut issues = Vec::new();
    let mut timeline = Vec::new();
    let at_ns = health.sync.worker.last_pass_at_ns;

    if health.sync.queue.retry_pending_count > 0 {
        issues.push(UiHealthIssueRow {
            issue_id: "sync_retry_backlog".to_string(),
            severity: "MEDIUM".to_string(),
            issue_type: "SYNC_RETRY_BACKLOG".to_string(),
            engine_owner: "PH1.OS".to_string(),
            first_seen_at_ns: at_ns,
            last_update_at_ns: at_ns,
            status: "OPEN".to_string(),
            resolution_state: "UNRESOLVED".to_string(),
        });
        timeline.push(UiHealthTimelineEntry {
            issue_id: "sync_retry_backlog".to_string(),
            at_ns,
            action_id: "SYNC_WORKER_RETRY_PASS".to_string(),
            result: format!("retry_pending={}", health.sync.queue.retry_pending_count),
            reason_code: reason_codes::ADAPTER_SYNC_RETRY.0.to_string(),
        });
    }
    if health.sync.queue.dead_letter_count > 0 {
        issues.push(UiHealthIssueRow {
            issue_id: "sync_dead_letter".to_string(),
            severity: "CRITICAL".to_string(),
            issue_type: "SYNC_DEAD_LETTER".to_string(),
            engine_owner: "PH1.OS".to_string(),
            first_seen_at_ns: at_ns,
            last_update_at_ns: at_ns,
            status: "ESCALATED".to_string(),
            resolution_state: "UNRESOLVED".to_string(),
        });
        timeline.push(UiHealthTimelineEntry {
            issue_id: "sync_dead_letter".to_string(),
            at_ns,
            action_id: "SYNC_WORKER_DEADLETTER".to_string(),
            result: format!("dead_lettered={}", health.sync.queue.dead_letter_count),
            reason_code: reason_codes::ADAPTER_SYNC_DEADLETTER.0.to_string(),
        });
    }
    if health.sync.queue.replay_due_count > 0 {
        issues.push(UiHealthIssueRow {
            issue_id: "sync_replay_due".to_string(),
            severity: "CRITICAL".to_string(),
            issue_type: "SYNC_REPLAY_DUE".to_string(),
            engine_owner: "PH1.OS".to_string(),
            first_seen_at_ns: at_ns,
            last_update_at_ns: at_ns,
            status: "OPEN".to_string(),
            resolution_state: "UNRESOLVED".to_string(),
        });
        timeline.push(UiHealthTimelineEntry {
            issue_id: "sync_replay_due".to_string(),
            at_ns,
            action_id: "SYNC_WORKER_REPLAY_DUE_SCAN".to_string(),
            result: format!("replay_due={}", health.sync.queue.replay_due_count),
            reason_code: reason_codes::ADAPTER_SYNC_REPLAY_DUE.0.to_string(),
        });
    }

    if timeline.is_empty() {
        timeline.push(UiHealthTimelineEntry {
            issue_id: "sync_nominal".to_string(),
            at_ns,
            action_id: "SYNC_WORKER_PASS".to_string(),
            result: "NO_OPEN_ISSUES".to_string(),
            reason_code: "0".to_string(),
        });
    }

    let critical_open = issues
        .iter()
        .filter(|issue| issue.severity == "CRITICAL")
        .count() as u32;
    let summary = UiHealthSummary {
        open_issues: issues.len() as u32,
        critical_open_count: critical_open,
        auto_resolved_24h_count: health.sync.worker.acked_total.min(u16::MAX as u64) as u32,
        escalated_24h_count: health.sync.queue.dead_letter_count,
        mttr_ms: None,
    };
    (summary, issues, timeline)
}

fn build_builder_detail(
    health: &AdapterHealthResponse,
) -> (
    UiHealthSummary,
    Vec<UiHealthIssueRow>,
    Vec<UiHealthTimelineEntry>,
) {
    let at_ns = health.sync.worker.last_pass_at_ns;
    let mut issues = Vec::new();
    let mut timeline = Vec::new();
    let builder_status = health
        .sync
        .improvement
        .last_builder_status
        .clone()
        .unwrap_or_else(|| "NO_BUILDER_ACTIVITY".to_string());
    let status = builder_health_status(health);

    if status != "HEALTHY" {
        issues.push(UiHealthIssueRow {
            issue_id: "builder_health".to_string(),
            severity: if status == "CRITICAL" {
                "CRITICAL".to_string()
            } else {
                "HIGH".to_string()
            },
            issue_type: "BUILDER_STATUS".to_string(),
            engine_owner: "PH1.BUILDER".to_string(),
            first_seen_at_ns: at_ns,
            last_update_at_ns: at_ns,
            status: "OPEN".to_string(),
            resolution_state: "UNRESOLVED".to_string(),
        });
    }

    timeline.push(UiHealthTimelineEntry {
        issue_id: "builder_health".to_string(),
        at_ns,
        action_id: "BUILDER_STATUS_TRACK".to_string(),
        result: builder_status,
        reason_code: "0".to_string(),
    });
    let summary = UiHealthSummary {
        open_issues: issues.len() as u32,
        critical_open_count: issues
            .iter()
            .filter(|issue| issue.severity == "CRITICAL")
            .count() as u32,
        auto_resolved_24h_count: health
            .sync
            .improvement
            .builder_completed_total
            .min(u16::MAX as u64) as u32,
        escalated_24h_count: 0,
        mttr_ms: None,
    };
    (summary, issues, timeline)
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
            user_text_partial: None,
            user_text_final: None,
            selene_text_partial: None,
            selene_text_final: None,
        }
    }

    fn base_report_query_request() -> UiHealthReportQueryRequest {
        UiHealthReportQueryRequest {
            correlation_id: Some(10_001),
            turn_id: Some(20_001),
            tenant_id: Some("tenant_a".to_string()),
            viewer_user_id: Some("viewer_01".to_string()),
            report_kind: Some("UNRESOLVED_ESCALATED".to_string()),
            from_utc_ns: Some(1),
            to_utc_ns: Some(5_000_000_000),
            engine_owner_filter: None,
            company_scope: Some("TENANT_ONLY".to_string()),
            company_ids: None,
            country_codes: None,
            escalated_only: Some(false),
            unresolved_only: Some(false),
            display_target: Some("desktop".to_string()),
            page_action: Some("FIRST".to_string()),
            page_cursor: None,
            report_context_id: None,
            page_size: Some(20),
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

    #[test]
    fn at_adapter_10_ui_health_checks_order_is_locked() {
        let runtime = AdapterRuntime::default();
        let response = runtime
            .ui_health_checks_report(Some(111))
            .expect("ui health checks should succeed");
        let order = response
            .checks
            .iter()
            .map(|check| check.check_id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            order,
            vec!["VOICE", "WAKE", "SYNC", "STT", "TTS", "DELIVERY", "BUILDER", "MEMORY"]
        );
    }

    #[test]
    fn at_adapter_11_ui_health_detail_rejects_unknown_check() {
        let runtime = AdapterRuntime::default();
        let err = runtime
            .ui_health_detail_report("NOT_A_CHECK", Some(111))
            .expect_err("unknown check id must fail");
        assert!(err.contains("invalid health check id"));
    }

    #[test]
    fn at_adapter_12_ui_chat_transcript_maps_user_and_selene_final_rows() {
        let runtime = AdapterRuntime::default();
        let mut req = base_request();
        req.user_text_final = Some("book payroll for Friday".to_string());
        req.selene_text_final = Some("Done. Payroll reminder is prepared.".to_string());
        runtime
            .run_voice_turn(req)
            .expect("voice turn with transcript finals must succeed");
        let response = runtime.ui_chat_transcript_report(Some(222));
        assert_eq!(response.status, "ok");
        assert!(response.note.is_none());
        assert!(response.messages.iter().any(|message| {
            message.role == "USER"
                && message.source == "PH1.C"
                && message.finalized
                && message.text == "book payroll for Friday"
        }));
        assert!(response.messages.iter().any(|message| {
            message.role == "SELENE"
                && message.source == "PH1.WRITE"
                && message.finalized
                && message.text == "Done. Payroll reminder is prepared."
        }));
    }

    #[test]
    fn at_adapter_13_partial_replaced_by_final_without_ghost_line() {
        let runtime = AdapterRuntime::default();
        let mut req_partial = base_request();
        req_partial.user_text_partial = Some("book pay".to_string());
        runtime
            .run_voice_turn(req_partial)
            .expect("partial transcript turn must succeed");
        let before = runtime.ui_chat_transcript_report(Some(333));
        assert!(before
            .messages
            .iter()
            .any(|message| !message.finalized && message.text == "book pay"));

        let mut req_final = base_request();
        req_final.user_text_final = Some("book payroll for Friday".to_string());
        req_final.now_ns = Some(4);
        runtime
            .run_voice_turn(req_final)
            .expect("final transcript turn must succeed");
        let after = runtime.ui_chat_transcript_report(Some(444));
        assert!(after.messages.iter().any(|message| {
            message.finalized && message.text == "book payroll for Friday" && message.role == "USER"
        }));
        assert!(!after
            .messages
            .iter()
            .any(|message| !message.finalized && message.text == "book pay"));
    }

    #[test]
    fn at_adapter_14_partial_rows_visible_when_final_not_present() {
        let runtime = AdapterRuntime::default();
        let mut req = base_request();
        req.user_text_partial = Some("checking".to_string());
        req.selene_text_partial = Some("working on it".to_string());
        runtime
            .run_voice_turn(req)
            .expect("partial-only turn must succeed");
        let response = runtime.ui_chat_transcript_report(Some(555));
        assert!(response.messages.iter().any(|message| {
            !message.finalized
                && message.role == "USER"
                && message.source == "PH1.C"
                && message.text == "checking"
        }));
        assert!(response.messages.iter().any(|message| {
            !message.finalized
                && message.role == "SELENE"
                && message.source == "PH1.WRITE"
                && message.text == "working on it"
        }));
    }

    #[test]
    fn at_adapter_15_report_query_clarify_then_remember_display_target() {
        let runtime = AdapterRuntime::default();
        let mut clarify_req = base_report_query_request();
        clarify_req.display_target = None;
        let clarify = runtime.ui_health_report_query(clarify_req, Some(5_000_000_000));
        assert_eq!(
            clarify.reason_code,
            health_reason_codes::PH1_HEALTH_DISPLAY_TARGET_REQUIRED
                .0
                .to_string()
        );
        assert!(clarify.requires_clarification.is_some());
        assert!(clarify.display_target_applied.is_none());

        let set_target = runtime.ui_health_report_query(base_report_query_request(), Some(5_000_000_001));
        assert_eq!(set_target.status, "ok");
        assert_eq!(set_target.display_target_applied.as_deref(), Some("desktop"));
        assert!(set_target.requires_clarification.is_none());

        let mut remembered_req = base_report_query_request();
        remembered_req.display_target = None;
        let remembered = runtime.ui_health_report_query(remembered_req, Some(5_000_000_002));
        assert_eq!(remembered.status, "ok");
        assert_eq!(remembered.display_target_applied.as_deref(), Some("desktop"));
        assert!(remembered.requires_clarification.is_none());
    }

    #[test]
    fn at_adapter_16_report_query_context_supports_follow_up_patch() {
        let runtime = AdapterRuntime::default();
        let first = runtime.ui_health_report_query(base_report_query_request(), Some(5_000_000_100));
        assert_eq!(first.status, "ok");
        let context_id = first
            .report_context_id
            .clone()
            .expect("report context id should be present");

        let mut follow_up = base_report_query_request();
        follow_up.report_context_id = Some(context_id.clone());
        follow_up.country_codes = Some(vec!["CN".to_string()]);
        let second = runtime.ui_health_report_query(follow_up, Some(5_000_000_101));
        assert_eq!(second.status, "ok");
        assert_eq!(second.report_context_id.as_deref(), Some(context_id.as_str()));
    }

    fn sample_health_issues_for_filters() -> Vec<UiHealthIssueRow> {
        vec![
            UiHealthIssueRow {
                issue_id: "sync_retry_backlog".to_string(),
                severity: "MEDIUM".to_string(),
                issue_type: "SYNC_RETRY_BACKLOG".to_string(),
                engine_owner: "PH1.OS".to_string(),
                first_seen_at_ns: Some(100),
                last_update_at_ns: Some(200),
                status: "OPEN".to_string(),
                resolution_state: "UNRESOLVED".to_string(),
            },
            UiHealthIssueRow {
                issue_id: "sync_dead_letter".to_string(),
                severity: "CRITICAL".to_string(),
                issue_type: "SYNC_DEAD_LETTER".to_string(),
                engine_owner: "PH1.OS".to_string(),
                first_seen_at_ns: Some(101),
                last_update_at_ns: Some(201),
                status: "ESCALATED".to_string(),
                resolution_state: "UNRESOLVED".to_string(),
            },
            UiHealthIssueRow {
                issue_id: "sync_replay_due".to_string(),
                severity: "CRITICAL".to_string(),
                issue_type: "SYNC_REPLAY_DUE".to_string(),
                engine_owner: "PH1.OS".to_string(),
                first_seen_at_ns: Some(102),
                last_update_at_ns: Some(202),
                status: "OPEN".to_string(),
                resolution_state: "UNRESOLVED".to_string(),
            },
        ]
    }

    fn sample_timeline_for_filters() -> Vec<UiHealthTimelineEntry> {
        vec![
            UiHealthTimelineEntry {
                issue_id: "sync_dead_letter".to_string(),
                at_ns: Some(400),
                action_id: "A4".to_string(),
                result: "r4".to_string(),
                reason_code: "4".to_string(),
            },
            UiHealthTimelineEntry {
                issue_id: "sync_dead_letter".to_string(),
                at_ns: Some(300),
                action_id: "A3".to_string(),
                result: "r3".to_string(),
                reason_code: "3".to_string(),
            },
            UiHealthTimelineEntry {
                issue_id: "sync_dead_letter".to_string(),
                at_ns: Some(200),
                action_id: "A2".to_string(),
                result: "r2".to_string(),
                reason_code: "2".to_string(),
            },
            UiHealthTimelineEntry {
                issue_id: "sync_dead_letter".to_string(),
                at_ns: Some(100),
                action_id: "A1".to_string(),
                result: "r1".to_string(),
                reason_code: "1".to_string(),
            },
        ]
    }

    #[test]
    fn at_adapter_17_health_detail_filters_open_critical_escalated() {
        let issues = sample_health_issues_for_filters();
        let filter = UiHealthDetailFilter {
            open_only: true,
            critical_only: true,
            escalated_only: true,
            ..UiHealthDetailFilter::default()
        };
        let filtered = filter_health_issues(&issues, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].issue_id, "sync_dead_letter");
    }

    #[test]
    fn at_adapter_18_health_detail_timeline_cursor_paging_is_deterministic() {
        let timeline = sample_timeline_for_filters();
        let filter = UiHealthDetailFilter {
            selected_issue_id: Some("sync_dead_letter".to_string()),
            ..UiHealthDetailFilter::default()
        };
        let filtered = filter_timeline_for_issue(&timeline, Some("sync_dead_letter"), &filter);
        let (page_one, paging_one) = page_timeline_entries(filtered.clone(), 2, None).unwrap();
        assert_eq!(page_one.len(), 2);
        assert_eq!(page_one[0].action_id, "A4");
        assert!(paging_one.has_next);
        assert_eq!(paging_one.next_cursor.as_deref(), Some("idx:2"));

        let (page_two, paging_two) =
            page_timeline_entries(filtered, 2, paging_one.next_cursor.as_deref()).unwrap();
        assert_eq!(page_two.len(), 2);
        assert_eq!(page_two[0].action_id, "A2");
        assert!(!paging_two.has_next);
    }

    #[test]
    fn at_adapter_19_health_detail_filter_rejects_invalid_date_range() {
        let runtime = AdapterRuntime::default();
        let filter = UiHealthDetailFilter {
            from_utc_ns: Some(20),
            to_utc_ns: Some(10),
            ..UiHealthDetailFilter::default()
        };
        let err = runtime
            .ui_health_detail_report_filtered("SYNC", filter, Some(100))
            .expect_err("invalid date range must fail");
        assert!(err.contains("invalid health detail date range"));
    }

    #[test]
    fn at_adapter_20_fail_closed_ui_state_markers_are_present() {
        assert!(app_ui_assets::APP_HTML.contains("health-state-banner"));
        assert!(app_ui_assets::APP_HTML.contains("report-state-chip"));
        assert!(app_ui_assets::APP_HTML.contains("voice-wave-state"));
        assert!(app_ui_assets::APP_CSS.contains(".state-error"));
        assert!(app_ui_assets::APP_CSS.contains(".voice-wave.wave-degraded"));
        assert!(app_ui_assets::APP_JS.contains("setHealthViewState(\"error\""));
        assert!(app_ui_assets::APP_JS.contains("setReportViewState(\"error\""));
        assert!(app_ui_assets::APP_JS.contains("setVoiceWaveState(\"degraded\""));
    }

    #[test]
    fn at_adapter_21_ios_android_desktop_contract_parity_is_locked() {
        let runtime = AdapterRuntime::default();
        let mut expected_outcome: Option<String> = None;
        for (idx, platform, trigger, device_id) in [
            (1_u64, "IOS", "WAKE_WORD", "ios_1"),
            (2_u64, "ANDROID", "WAKE_WORD", "android_1"),
            (3_u64, "DESKTOP", "EXPLICIT", "desktop_1"),
        ] {
            let mut req = base_request();
            req.turn_id = 20_100 + idx;
            req.now_ns = Some(10_000 + idx);
            req.app_platform = platform.to_string();
            req.trigger = trigger.to_string();
            req.device_id = Some(device_id.to_string());
            let out = runtime
                .run_voice_turn(req)
                .expect("platform turn should succeed");
            assert_eq!(out.status, "ok");
            if let Some(expected) = &expected_outcome {
                assert_eq!(&out.outcome, expected);
            } else {
                expected_outcome = Some(out.outcome.clone());
            }
        }

        let checks = runtime
            .ui_health_checks_report(Some(10_100))
            .expect("health checks should succeed");
        let order = checks
            .checks
            .iter()
            .map(|row| row.check_id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            order,
            vec!["VOICE", "WAKE", "SYNC", "STT", "TTS", "DELIVERY", "BUILDER", "MEMORY"]
        );
        assert!(checks
            .checks
            .iter()
            .all(|row| !row.label.trim().is_empty() && !row.status.trim().is_empty()));
    }

    #[test]
    fn at_adapter_22_voice_text_bidirectional_transcript_parity_is_locked() {
        let runtime = AdapterRuntime::default();

        let mut voice_turn = base_request();
        voice_turn.turn_id = 30_001;
        voice_turn.now_ns = Some(20_001);
        voice_turn.trigger = "WAKE_WORD".to_string();
        voice_turn.user_text_final = Some("show missed stt report for june".to_string());
        voice_turn.selene_text_final = Some("Opening the report on desktop.".to_string());
        runtime
            .run_voice_turn(voice_turn)
            .expect("voice turn should succeed");

        let mut text_turn = base_request();
        text_turn.turn_id = 30_002;
        text_turn.now_ns = Some(20_002);
        text_turn.trigger = "EXPLICIT".to_string();
        text_turn.app_platform = "DESKTOP".to_string();
        text_turn.user_text_final = Some("same report for all customers in china".to_string());
        text_turn.selene_text_final = Some("Updated report now shown for China scope.".to_string());
        runtime
            .run_voice_turn(text_turn)
            .expect("text turn should succeed");

        let transcript = runtime.ui_chat_transcript_report(Some(20_003));
        assert_eq!(transcript.status, "ok");
        let user_final_count = transcript
            .messages
            .iter()
            .filter(|message| {
                message.role == "USER" && message.source == "PH1.C" && message.finalized
            })
            .count();
        let selene_final_count = transcript
            .messages
            .iter()
            .filter(|message| {
                message.role == "SELENE" && message.source == "PH1.WRITE" && message.finalized
            })
            .count();
        assert!(user_final_count >= 2);
        assert!(selene_final_count >= 2);
    }

    #[test]
    fn at_health_10_display_target_clarify_then_memory_reuse() {
        at_adapter_15_report_query_clarify_then_remember_display_target();
    }

    #[test]
    fn at_health_11_follow_up_report_patch_reuses_context() {
        let runtime = AdapterRuntime::default();
        let mut first_req = base_report_query_request();
        first_req.from_utc_ns = Some(8_000_000_000);
        first_req.to_utc_ns = Some(9_000_000_200);
        let first = runtime.ui_health_report_query(first_req, Some(9_000_000_100));
        assert_eq!(first.status, "ok");
        let first_context = first
            .report_context_id
            .clone()
            .expect("first context id must be present");
        let first_revision = first.report_revision;
        let first_rows = first.rows;
        assert!(!first_rows.is_empty());

        let mut follow_up = base_report_query_request();
        follow_up.from_utc_ns = Some(8_000_000_000);
        follow_up.to_utc_ns = Some(9_000_000_200);
        follow_up.report_context_id = Some(first_context.clone());
        follow_up.country_codes = Some(vec!["CN".to_string()]);
        let second = runtime.ui_health_report_query(follow_up, Some(9_000_000_101));
        assert_eq!(second.status, "ok");
        assert_eq!(second.report_context_id.as_deref(), Some(first_context.as_str()));
        assert_ne!(second.report_revision, first_revision);
        assert_ne!(second.rows, first_rows);
    }

    #[test]
    fn at_health_12_voice_wave_degraded_marker_is_wired() {
        assert!(app_ui_assets::APP_HTML.contains("voice-wave-state"));
        assert!(app_ui_assets::APP_CSS.contains(".voice-wave.wave-degraded"));
        assert!(app_ui_assets::APP_JS.contains("degraded (transcript sync failed)"));
    }
}
