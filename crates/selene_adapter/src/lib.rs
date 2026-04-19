#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use selene_engines::ph1_voice_id::VoiceIdObservation as EngineVoiceIdObservation;
use selene_engines::ph1c::{
    reason_codes as ph1c_reason_codes, Ph1cConfig as EnginePh1cConfig, Ph1cLiveProviderContext,
    Ph1cRuntime as EnginePh1cRuntime, Ph1cStreamCommit,
};
use selene_engines::ph1context::{
    Ph1ContextConfig as EnginePh1ContextConfig, Ph1ContextRuntime as EnginePh1ContextRuntime,
};
use selene_engines::ph1d::{
    reason_codes as ph1d_reason_codes, ModelCallOutcome as Ph1dModelCallOutcome,
    Ph1dProviderAdapter, Ph1dProviderAdapterError, Ph1dRuntime as EnginePh1dRuntime,
};
use selene_engines::ph1health::{
    reason_codes as health_reason_codes, Ph1HealthConfig as EngineHealthConfig,
    Ph1HealthRuntime as EngineHealthRuntime,
};
use selene_engines::ph1k::{
    build_interrupt_feedback_signal, build_ph1k_to_ph1c_handoff, default_adaptive_policy_input,
    evaluate_interrupt_candidate, InterruptFeedbackSignalKind, InterruptInput, InterruptNoiseClass,
    InterruptPhraseMatcher, PhraseDetection,
};
use selene_engines::ph1n::{Ph1nConfig as EnginePh1nConfig, Ph1nRuntime as EnginePh1nRuntime};
use selene_engines::ph1pattern::{Ph1PatternConfig as EnginePatternConfig, Ph1PatternRuntime};
use selene_engines::ph1rll::{Ph1RllConfig as EngineRllConfig, Ph1RllRuntime};
use selene_engines::ph1vision::{
    Ph1VisionConfig as EnginePh1VisionConfig, Ph1VisionRuntime as EnginePh1VisionRuntime,
};
use selene_engines::ph1w::{
    reason_codes as ph1w_reason_codes, Ph1wOutputEvent, Ph1wRuntime, SourceLivenessHint,
    WakeConfig as EngineWakeConfig, WakeStepInput,
};
use selene_kernel_contracts::ph1_voice_id::{
    DeviceTrustLevel, Ph1VoiceIdRequest, Ph1VoiceIdResponse, UserId,
};
use selene_kernel_contracts::ph1art::{
    ArtifactScopeType, ArtifactStatus, ArtifactType, ArtifactVersion,
};
use selene_kernel_contracts::ph1c::{
    ConfidenceBucket as Ph1cConfidenceBucket, LanguageHint, LanguageHintConfidence, LanguageTag,
    NoiseLevelHint, Ph1cRequest, Ph1cResponse, Ph1kToPh1cHandoff, RetryAdvice as Ph1cRetryAdvice,
    SessionStateRef as Ph1cSessionStateRef, SpeakerOverlapClass, SpeakerOverlapHint,
    TranscriptOk as Ph1cTranscriptOk, VadQualityHint,
};
use selene_kernel_contracts::ph1context::{Ph1ContextRequest, Ph1ContextResponse};
use selene_kernel_contracts::ph1d::{
    Ph1dFailureKind, Ph1dOk, Ph1dProviderCallRequest, Ph1dProviderCallResponse, Ph1dResponse,
    PolicyContextRef, SafetyTier,
};
use selene_kernel_contracts::ph1e::{
    CacheStatus, ToolCatalogRef, ToolName, ToolResponse, ToolStatus,
};
use selene_kernel_contracts::ph1f::{
    ConversationRole, ConversationSource, ConversationTurnInput, PrivacyScope,
};
use selene_kernel_contracts::ph1feedback::FeedbackEventType;
use selene_kernel_contracts::ph1health::{
    HealthAckState, HealthActionResult, HealthCompanyScope, HealthDisplayTarget, HealthIssueEvent,
    HealthIssueStatus, HealthPageAction, HealthReadEnvelope, HealthReportKind,
    HealthReportQueryReadOk, HealthReportQueryReadRequest, HealthReportTimeRange, HealthSeverity,
    Ph1HealthRequest, Ph1HealthResponse,
};
use selene_kernel_contracts::ph1j::{CorrelationId, DeviceId, TurnId};
use selene_kernel_contracts::ph1k::{
    AdvancedAudioQualityMetrics, AudioDeviceId, AudioFormat, AudioStreamId, AudioStreamKind,
    AudioStreamRef, ChannelCount, Confidence, DeviceHealth, DeviceReliabilityScoreInput,
    DeviceRoute, DeviceState, FrameDurationMs, InterruptLexiconPolicyBinding, InterruptLocaleTag,
    PreRollBufferId, PreRollBufferRef, SampleFormat, SampleRateHz, SpeechLikeness,
    TimingStats as Ph1kTimingStats, TtsPlaybackActiveEvent, VadEvent,
};
use selene_kernel_contracts::ph1l::{
    Ph1lInput, SessionId, SessionSnapshot, TtsPlaybackState, UserActivitySignals,
};
use selene_kernel_contracts::ph1learn::{LearnSignalType, WakeLearnSignalV1, WakeLearnTrigger};
use selene_kernel_contracts::ph1link::{AppPlatform, TokenId};
use selene_kernel_contracts::ph1n::{Chat as Ph1nChat, Ph1nRequest, Ph1nResponse};
use selene_kernel_contracts::ph1onb::{
    OnboardingNextStep, OnboardingSessionId, SenderVerifyDecision,
};
use selene_kernel_contracts::ph1os::{OsNextMove, OsOutcomeActionClass, OsOutcomeUtilizationEntry};
use selene_kernel_contracts::ph1pattern::{Ph1PatternRequest, Ph1PatternResponse};
use selene_kernel_contracts::ph1position::TenantId;
use selene_kernel_contracts::ph1rll::{Ph1RllRequest, Ph1RllResponse};
use selene_kernel_contracts::ph1vision::{
    BoundingBoxPx, Ph1VisionRequest, Ph1VisionResponse, VisualSourceId, VisualSourceKind,
    VisualSourceRef, VisualToken,
};
use selene_kernel_contracts::ph1w::{
    BoundedAudioSegmentRef, SessionState as WakeSessionState, WakeDecision, WakeGateResults,
    WakePolicyContext, PH1W_IMPLEMENTATION_ID,
};
use selene_kernel_contracts::ph1x::{
    ConfirmAnswer, PendingState, Ph1xDirective, ThreadPolicyFlags, ThreadState,
};
use selene_kernel_contracts::runtime_execution::{
    default_device_class_for_platform, default_hardware_capability_profile,
    supported_capabilities_for_platform, AdmissionState, ClientCompatibilityStatus,
    ClientIntegrityStatus, DeviceCapability, DeviceClass, DeviceTrustClass, FailureClass,
    NetworkProfile, PersistenceAcknowledgementState, PersistenceConflictSeverity,
    PersistenceConvergenceState, PersistenceExecutionState, PersistenceRecoveryMode,
    PlatformRuntimeContext, PlatformTriggerPolicy, ReconciliationDecision, RuntimeEntryTrigger,
    RuntimeExecutionEnvelope, SessionAttachOutcome,
};
use selene_kernel_contracts::runtime_governance::GovernanceProtectedActionClass;
use selene_kernel_contracts::{
    ContractViolation, MonotonicTimeNs, ReasonCodeId, SessionState, Validate,
};
use selene_os::app_ingress::{
    AppInviteLinkOpenRequest, AppOnboardingContinueAction, AppOnboardingContinueNextStep,
    AppOnboardingContinueRequest, AppServerIngressRuntime,
    AppSessionRecoverRequest, AppSessionResumeRequest, AppVoiceIngressRequest,
    AppVoicePh1xBuildInput, AppVoiceTurnExecutionOutcome, AppVoiceTurnNextMove,
    AppWakeProfileAvailabilityRefreshRequest,
};
use selene_os::device_artifact_sync::DeviceArtifactSyncWorkerPassMetrics;
use selene_os::ph1_voice_id::{
    Ph1VoiceIdLiveConfig, VoiceIdContractMigrationConfig, VoiceIdentityEmbeddingGateGovernedConfig,
    VoiceIdentityEmbeddingGateProfile, VoiceIdentityEmbeddingGateProfiles,
};
use selene_os::ph1builder::{
    BuilderOfflineInput, BuilderOrchestrationOutcome, DeterministicBuilderSandboxValidator,
    Ph1BuilderConfig, Ph1BuilderOrchestrator,
};
use selene_os::ph1context::{Ph1ContextEngine, Ph1ContextWiring, Ph1ContextWiringConfig};
use selene_os::ph1l::{
    ph1l_step_voice_turn, trigger_requires_session_open_step, Ph1lConfig, Ph1lRuntime,
    Ph1lTurnTrigger,
};
use selene_os::ph1n::{Ph1nEngine, Ph1nWiring, Ph1nWiringConfig};
use selene_os::ph1os::{
    OsOcrAnalyzerForwardBundle, OsOcrContextNlpOutcome, OsOcrRouteOutcome, OsVoiceLiveTurnOutcome,
    OsVoiceTrigger, Ph1OsOcrContextNlpConfig, Ph1OsOcrContextNlpWiring, Ph1OsOcrRouteConfig,
    Ph1OsOcrRouteWiring,
};
use selene_os::ph1pattern::Ph1PatternEngine;
use selene_os::ph1rll::Ph1RllEngine;
use selene_os::ph1vision::{
    Ph1VisionEngine, Ph1VisionWiring, Ph1VisionWiringConfig, VisionTurnInput, VisionWiringOutcome,
};
use selene_os::ph1x::{resolve_report_display_target, ReportDisplayResolution};
use selene_os::runtime_governance::{
    governance_failure_class_for_response, governance_reason_to_session_state,
    governance_runtime_reason, RuntimeGovernanceDecision,
};
use selene_os::simulation_executor::SimulationExecutor;
use selene_storage::ph1f::{
    DeviceRecord, IdentityRecord, IdentityStatus, MobileArtifactSyncKind, MobileArtifactSyncState,
    OutcomeUtilizationLedgerRowInput, Ph1fStore, Ph1kDeviceHealth, Ph1kFeedbackCaptureInput,
    Ph1kFeedbackIssueKind, Ph1kInterruptCandidateExtendedFields, Ph1kRuntimeEventKind,
    Ph1kRuntimeEventRecord, SessionRecord, StorageError,
};
pub mod grpc_api {
    tonic::include_proto!("selene.adapter.v1");
}

pub mod desktop_mic_producer;

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
    pub const ADAPTER_READ_ONLY_TOOL_FAIL_INCIDENT: ReasonCodeId = ReasonCodeId(0xAD70_0011);
    pub const ADAPTER_READ_ONLY_CLARIFY_LOOP_INCIDENT: ReasonCodeId = ReasonCodeId(0xAD70_0012);
    pub const ADAPTER_READ_ONLY_USER_CORRECTION_INCIDENT: ReasonCodeId = ReasonCodeId(0xAD70_0013);
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct VoiceTurnAudioCaptureRef {
    pub stream_id: u128,
    pub pre_roll_buffer_id: u64,
    pub t_start_ns: u64,
    pub t_end_ns: u64,
    pub t_candidate_start_ns: u64,
    pub t_confirmed_ns: u64,
    pub locale_tag: Option<String>,
    pub device_route: Option<String>,
    pub selected_mic: Option<String>,
    pub selected_speaker: Option<String>,
    pub tts_playback_active: Option<bool>,
    pub detection_text: Option<String>,
    pub detection_confidence_bp: Option<u16>,
    pub vad_confidence_bp: Option<u16>,
    pub acoustic_confidence_bp: Option<u16>,
    pub prosody_confidence_bp: Option<u16>,
    pub speech_likeness_bp: Option<u16>,
    pub echo_safe_confidence_bp: Option<u16>,
    pub nearfield_confidence_bp: Option<u16>,
    pub capture_degraded: Option<bool>,
    pub stream_gap_detected: Option<bool>,
    pub aec_unstable: Option<bool>,
    pub device_changed: Option<bool>,
    pub snr_db_milli: Option<i32>,
    pub clipping_ratio_bp: Option<u16>,
    pub echo_delay_ms_milli: Option<u32>,
    pub packet_loss_bp: Option<u16>,
    pub double_talk_bp: Option<u16>,
    pub erle_db_milli: Option<i32>,
    pub device_failures_24h: Option<u32>,
    pub device_recoveries_24h: Option<u32>,
    pub device_mean_recovery_ms: Option<u32>,
    pub device_reliability_bp: Option<u16>,
    pub timing_jitter_ms_milli: Option<u32>,
    pub timing_drift_ppm_milli: Option<u32>,
    pub timing_buffer_depth_ms_milli: Option<u32>,
    pub timing_underruns: Option<u64>,
    pub timing_overruns: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct VoiceTurnVisualTokenRef {
    pub token: String,
    pub x: Option<u32>,
    pub y: Option<u32>,
    pub w: Option<u32>,
    pub h: Option<u32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct VoiceTurnVisualInputRef {
    pub turn_opt_in_enabled: bool,
    pub source_id: Option<String>,
    pub source_kind: Option<String>,
    pub image_ref: Option<String>,
    pub blob_ref: Option<String>,
    pub visible_tokens: Vec<VoiceTurnVisualTokenRef>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct VoiceTurnThreadPolicyFlags {
    pub privacy_mode: bool,
    pub do_not_disturb: bool,
    pub strict_safety: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VoiceTurnAdapterRequest {
    pub correlation_id: u64,
    pub turn_id: u64,
    pub device_turn_sequence: Option<u64>,
    pub app_platform: String,
    pub platform_version: Option<String>,
    pub device_class: Option<String>,
    pub runtime_client_version: Option<String>,
    pub hardware_capability_profile: Option<String>,
    pub network_profile: Option<String>,
    pub claimed_capabilities: Option<Vec<String>>,
    pub integrity_status: Option<String>,
    pub attestation_ref: Option<String>,
    pub trigger: String,
    pub actor_user_id: String,
    pub tenant_id: Option<String>,
    pub device_id: Option<String>,
    pub now_ns: Option<u64>,
    pub thread_key: Option<String>,
    pub project_id: Option<String>,
    pub pinned_context_refs: Option<Vec<String>>,
    pub thread_policy_flags: Option<VoiceTurnThreadPolicyFlags>,
    pub user_text_partial: Option<String>,
    pub user_text_final: Option<String>,
    pub selene_text_partial: Option<String>,
    pub selene_text_final: Option<String>,
    pub audio_capture_ref: Option<VoiceTurnAudioCaptureRef>,
    pub visual_input_ref: Option<VoiceTurnVisualInputRef>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VoiceTurnAdapterResponse {
    pub status: String,
    pub outcome: String,
    pub session_id: Option<String>,
    pub turn_id: Option<u64>,
    pub session_state: Option<String>,
    pub session_attach_outcome: Option<SessionAttachOutcome>,
    pub failure_class: Option<FailureClass>,
    pub reason: Option<String>,
    pub next_move: String,
    pub response_text: String,
    pub reason_code: String,
    pub provenance: Option<VoiceTurnProvenance>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VoiceTurnProvenanceSource {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VoiceTurnProvenance {
    pub sources: Vec<VoiceTurnProvenanceSource>,
    pub retrieved_at: u64,
    pub cache_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct VoiceTurnIngressError {
    pub failure_class: FailureClass,
    pub reason_code: String,
    pub reason: Option<String>,
    pub session_id: Option<String>,
    pub turn_id: Option<u64>,
    pub session_state: Option<String>,
}

impl VoiceTurnIngressError {
    pub fn to_runtime_reason(&self) -> String {
        self.reason
            .clone()
            .unwrap_or_else(|| self.reason_code.clone())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct AdapterPersistenceRuntime {
    legacy_journal_path: PathBuf,
    state_path: PathBuf,
    #[serde(skip)]
    state: Arc<Mutex<AdapterPersistenceState>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct AdapterPersistenceState {
    schema_version: u8,
    recovery_mode: PersistenceRecoveryMode,
    outbox_records: BTreeMap<String, AdapterOutboxRecord>,
    operation_journal: Vec<AdapterOperationJournalEntry>,
    authoritative_outcomes: BTreeMap<String, AdapterAuthoritativeOutcomeRecord>,
    audit_trail: Vec<AdapterPersistenceAuditEntry>,
    next_journal_sequence: u64,
    next_audit_sequence: u64,
    last_reconciled_at_ns: Option<u64>,
}

impl Default for AdapterPersistenceState {
    fn default() -> Self {
        Self {
            schema_version: 2,
            recovery_mode: PersistenceRecoveryMode::Normal,
            outbox_records: BTreeMap::new(),
            operation_journal: Vec::new(),
            authoritative_outcomes: BTreeMap::new(),
            audit_trail: Vec::new(),
            next_journal_sequence: 1,
            next_audit_sequence: 1,
            last_reconciled_at_ns: None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct AdapterOutboxRecord {
    operation_id: String,
    request_id: String,
    trace_id: String,
    actor_user_id: String,
    idempotency_key: String,
    session_id: Option<String>,
    turn_id: Option<u64>,
    device_id: String,
    device_turn_sequence: Option<u64>,
    submission_timestamp_ns: u64,
    retry_counter: u32,
    acknowledgement_state: PersistenceAcknowledgementState,
    cleared_from_active_outbox_at_ns: Option<u64>,
    last_reconciliation_decision: Option<ReconciliationDecision>,
    last_conflict_severity: Option<PersistenceConflictSeverity>,
    request: VoiceTurnAdapterRequest,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct AdapterOperationJournalEntry {
    sequence: u64,
    operation_id: String,
    request_id: String,
    trace_id: String,
    actor_user_id: String,
    idempotency_key: String,
    session_id: Option<String>,
    turn_id: Option<u64>,
    device_id: String,
    device_turn_sequence: Option<u64>,
    at_ns: u64,
    acknowledgement_state: PersistenceAcknowledgementState,
    recovery_mode: PersistenceRecoveryMode,
    reconciliation_decision: Option<ReconciliationDecision>,
    conflict_severity: Option<PersistenceConflictSeverity>,
    event_kind: AdapterOperationJournalEventKind,
    detail: Option<String>,
    runtime_node_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum AdapterOperationJournalEventKind {
    Created,
    SubmissionAttempted,
    RetryAttempted,
    Acknowledged,
    TerminalFailure,
    Quarantined,
    RecoveryModeTransition,
    DedupeReused,
    FreshSessionStateRequested,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct AdapterAuthoritativeOutcomeRecord {
    actor_user_id: String,
    idempotency_key: String,
    session_id: Option<String>,
    turn_id: Option<u64>,
    session_state: Option<String>,
    device_id: String,
    device_turn_sequence: Option<u64>,
    acknowledged_at_ns: u64,
    runtime_node_id: String,
    #[serde(default)]
    governance_policy_version: Option<String>,
    conflict_severity: Option<PersistenceConflictSeverity>,
    result: AdapterPersistedAuthoritativeResult,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
enum AdapterPersistedAuthoritativeResult {
    Success(VoiceTurnAdapterResponse),
    Failure(VoiceTurnIngressError),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct AdapterPersistenceAuditEntry {
    sequence: u64,
    at_ns: u64,
    operation_id: Option<String>,
    actor_user_id: Option<String>,
    idempotency_key: Option<String>,
    session_id: Option<String>,
    turn_id: Option<u64>,
    decision: AdapterPersistenceAuditDecision,
    recovery_mode: PersistenceRecoveryMode,
    conflict_severity: Option<PersistenceConflictSeverity>,
    runtime_node_id: String,
    note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum AdapterPersistenceAuditDecision {
    ReconciliationDecision,
    DedupeOutcome,
    QuarantineAction,
    RecoveryModeTransition,
    AcknowledgementOutcome,
    LegacyJournalReplay,
}

#[derive(Debug, Clone)]
struct PreparedPersistenceOperation {
    operation_id: String,
    persistence_state: PersistenceExecutionState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PersistenceInvocationMode {
    Standard,
    ExistingOutboxReplay,
    LegacyJournalReplay,
}

#[derive(Clone, Copy)]
struct SyncImprovementBuilderContext<'a> {
    now: MonotonicTimeNs,
    correlation_id: CorrelationId,
    turn_id: TurnId,
    metrics: &'a DeviceArtifactSyncWorkerPassMetrics,
    queue_after: &'a AdapterSyncQueueCounters,
    outcome_entries: &'a [OsOutcomeUtilizationEntry],
}

struct ReusedResponseTranscriptUpdate<'a> {
    now: MonotonicTimeNs,
    correlation_id: CorrelationId,
    turn_id: TurnId,
    actor_user_id: &'a UserId,
    device_id: &'a DeviceId,
    response: &'a VoiceTurnAdapterResponse,
    user_text_final: Option<&'a str>,
    selene_text_final: Option<&'a str>,
}

struct FallbackRuntimeExecutionEnvelopeParams<'a> {
    correlation_id: CorrelationId,
    turn_id: TurnId,
    app_platform: AppPlatform,
    actor_user_id: &'a UserId,
    device_id: &'a DeviceId,
    device_turn_sequence: Option<u64>,
    session_attach_outcome: Option<SessionAttachOutcome>,
    platform_context: Option<PlatformRuntimeContext>,
}

#[derive(Debug, Default)]
struct OnboardingContinueActionInput {
    field_value: Option<String>,
    receipt_kind: Option<String>,
    receipt_ref: Option<String>,
    signer: Option<String>,
    payload_hash: Option<String>,
    terms_version_id: Option<String>,
    accepted: Option<bool>,
    device_id: Option<String>,
    proof_ok: Option<bool>,
    sample_seed: Option<String>,
    photo_blob_ref: Option<String>,
    sender_decision: Option<String>,
}

struct ResolveSessionTurnStateInput<'a> {
    actor_user_id: &'a UserId,
    device_id: &'a DeviceId,
    trigger: OsVoiceTrigger,
    ph1k: &'a Ph1kLiveSignalBundle,
    wake_event: Option<WakeDecision>,
    idempotency_key: &'a str,
    device_turn_sequence: u64,
    runtime_node_id: &'a str,
    session_lease_ttl_ms: u64,
    session_retry_cache: &'a Arc<Mutex<BTreeMap<AdapterRetryCacheKey, VoiceTurnAdapterResponse>>>,
}

struct PersistPh1xThreadStateInput<'a> {
    actor_user_id: &'a UserId,
    thread_key: &'a str,
    thread_state: ThreadState,
    reason_code: ReasonCodeId,
    correlation_id: CorrelationId,
    turn_id: TurnId,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InviteLinkOpenAdapterRequest {
    pub correlation_id: u64,
    pub idempotency_key: String,
    pub token_id: String,
    pub token_signature: String,
    pub tenant_id: Option<String>,
    pub app_platform: String,
    pub device_fingerprint: String,
    pub app_instance_id: String,
    pub deep_link_nonce: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InviteLinkOpenAdapterResponse {
    pub status: String,
    pub outcome: String,
    pub reason: Option<String>,
    pub onboarding_session_id: Option<String>,
    pub next_step: Option<String>,
    pub required_fields: Vec<String>,
    pub required_verification_gates: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OnboardingContinueAdapterRequest {
    pub correlation_id: u64,
    pub onboarding_session_id: String,
    pub idempotency_key: String,
    pub tenant_id: Option<String>,
    pub action: String,
    pub field_value: Option<String>,
    pub receipt_kind: Option<String>,
    pub receipt_ref: Option<String>,
    pub signer: Option<String>,
    pub payload_hash: Option<String>,
    pub terms_version_id: Option<String>,
    pub accepted: Option<bool>,
    pub device_id: Option<String>,
    pub proof_ok: Option<bool>,
    pub sample_seed: Option<String>,
    pub photo_blob_ref: Option<String>,
    pub sender_decision: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OnboardingContinueAdapterResponse {
    pub status: String,
    pub outcome: String,
    pub reason: Option<String>,
    pub onboarding_session_id: Option<String>,
    pub next_step: Option<String>,
    pub blocking_field: Option<String>,
    pub blocking_question: Option<String>,
    pub remaining_missing_fields: Vec<String>,
    pub remaining_platform_receipt_kinds: Vec<String>,
    pub voice_artifact_sync_receipt_ref: Option<String>,
    pub access_engine_instance_id: Option<String>,
    pub onboarding_status: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionResumeAdapterRequest {
    pub correlation_id: u64,
    pub idempotency_key: String,
    pub session_id: String,
    pub device_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionResumeAdapterResponse {
    pub status: String,
    pub outcome: String,
    pub reason: Option<String>,
    pub session_id: Option<String>,
    pub session_state: Option<String>,
    pub session_attach_outcome: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionRecoverAdapterRequest {
    pub correlation_id: u64,
    pub idempotency_key: String,
    pub session_id: String,
    pub device_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionRecoverAdapterResponse {
    pub status: String,
    pub outcome: String,
    pub reason: Option<String>,
    pub session_id: Option<String>,
    pub session_state: Option<String>,
    pub session_attach_outcome: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WakeProfileAvailabilityRefreshAdapterRequest {
    pub correlation_id: u64,
    pub idempotency_key: String,
    pub device_id: String,
    pub expected_wake_profile_id: String,
    pub voice_artifact_sync_receipt_ref: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WakeProfileAvailabilityRefreshAdapterResponse {
    pub status: String,
    pub outcome: String,
    pub reason: Option<String>,
    pub device_id: Option<String>,
    pub wake_profile_id: Option<String>,
    pub active_wake_artifact_version: Option<String>,
    pub activated_count: u64,
    pub noop_count: u64,
    pub pull_error_count: u64,
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
    pub blocker: Option<String>,
    pub unresolved_deadline_at_ns: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UiHealthTimelineEntry {
    pub issue_id: String,
    pub at_ns: Option<u64>,
    pub action_id: String,
    pub result: String,
    pub reason_code: String,
    pub evidence_ref: Option<String>,
    pub blocker: Option<String>,
    pub unresolved_deadline_at_ns: Option<u64>,
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
    session_retry_cache: Arc<Mutex<BTreeMap<AdapterRetryCacheKey, VoiceTurnAdapterResponse>>>,
    sync_worker_counters: Arc<Mutex<AdapterSyncWorkerCounters>>,
    improvement_counters: Arc<Mutex<AdapterImprovementCounters>>,
    transcript_state: Arc<Mutex<AdapterTranscriptState>>,
    report_display_target_defaults: Arc<Mutex<BTreeMap<String, String>>>,
    auto_builder_enabled: bool,
    ph1c_live_enabled: bool,
    ph1c_streaming_enabled: bool,
    ph1c_runtime: EnginePh1cRuntime,
    ph1d_runtime: EnginePh1dRuntime,
    ph1d_live_adapter: Option<EnvPh1dLiveAdapter>,
    persistence: Option<AdapterPersistenceRuntime>,
    runtime_node_id: String,
    session_lease_ttl_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct AdapterRetryCacheKey {
    actor_user_id: UserId,
    idempotency_key: String,
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

#[derive(Debug, Clone)]
struct AdapterVisionEngineRuntime {
    runtime: EnginePh1VisionRuntime,
}

impl AdapterVisionEngineRuntime {
    fn new() -> Self {
        Self {
            runtime: EnginePh1VisionRuntime::new(EnginePh1VisionConfig::mvp_v1()),
        }
    }
}

impl Ph1VisionEngine for AdapterVisionEngineRuntime {
    fn run(&self, req: &Ph1VisionRequest) -> Ph1VisionResponse {
        self.runtime.run(req)
    }
}

#[derive(Debug, Clone)]
struct AdapterContextEngineRuntime {
    runtime: EnginePh1ContextRuntime,
}

impl AdapterContextEngineRuntime {
    fn new() -> Self {
        Self {
            runtime: EnginePh1ContextRuntime::new(EnginePh1ContextConfig::mvp_v1()),
        }
    }
}

impl Ph1ContextEngine for AdapterContextEngineRuntime {
    fn run(&self, req: &Ph1ContextRequest) -> Ph1ContextResponse {
        self.runtime.run(req)
    }
}

#[derive(Debug, Clone)]
struct AdapterNlpEngineRuntime {
    runtime: EnginePh1nRuntime,
}

impl AdapterNlpEngineRuntime {
    fn new() -> Self {
        Self {
            runtime: EnginePh1nRuntime::new(EnginePh1nConfig::mvp_v1()),
        }
    }
}

impl Ph1nEngine for AdapterNlpEngineRuntime {
    fn run(&self, req: &Ph1nRequest) -> Result<Ph1nResponse, ContractViolation> {
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
enum ReadOnlyIncidentKind {
    ToolFail,
    ClarifyLoop,
    UserCorrection,
}

impl ReadOnlyIncidentKind {
    fn outcome_type(self) -> &'static str {
        match self {
            ReadOnlyIncidentKind::ToolFail => "READ_ONLY_TOOL_FAIL",
            ReadOnlyIncidentKind::ClarifyLoop => "READ_ONLY_CLARIFY_LOOP",
            ReadOnlyIncidentKind::UserCorrection => "READ_ONLY_USER_CORRECTION",
        }
    }

    fn tag(self) -> &'static str {
        match self {
            ReadOnlyIncidentKind::ToolFail => "tool_fail",
            ReadOnlyIncidentKind::ClarifyLoop => "clarify_loop",
            ReadOnlyIncidentKind::UserCorrection => "user_correction",
        }
    }

    fn feedback_event_type(self) -> FeedbackEventType {
        match self {
            ReadOnlyIncidentKind::ToolFail => FeedbackEventType::ToolFail,
            ReadOnlyIncidentKind::ClarifyLoop => FeedbackEventType::ClarifyLoop,
            ReadOnlyIncidentKind::UserCorrection => FeedbackEventType::UserCorrection,
        }
    }

    fn learn_signal_type(self) -> LearnSignalType {
        match self {
            ReadOnlyIncidentKind::ToolFail => LearnSignalType::ToolFail,
            ReadOnlyIncidentKind::ClarifyLoop => LearnSignalType::ClarifyLoop,
            ReadOnlyIncidentKind::UserCorrection => LearnSignalType::UserCorrection,
        }
    }

    fn severe(self) -> bool {
        matches!(
            self,
            ReadOnlyIncidentKind::ToolFail | ReadOnlyIncidentKind::ClarifyLoop
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReadOnlyIncidentRecord {
    kind: ReadOnlyIncidentKind,
    reason_code: ReasonCodeId,
    evidence_ref: String,
    provenance_ref: String,
}

#[derive(Debug, Clone)]
struct Ph1cLiveTurnOutcomeSummary {
    response: Ph1cResponse,
    partial_text: Option<String>,
    final_text: Option<String>,
    finalized: bool,
    low_latency_commit: bool,
    provider_call_trace: Vec<Ph1dProviderCallResponse>,
}

#[derive(Debug, Clone)]
struct EnvPh1dLiveAdapter {
    provider_id: String,
    model_id: String,
}

impl EnvPh1dLiveAdapter {
    fn from_env() -> Result<Self, String> {
        let provider_id = env::var("SELENE_PH1D_LIVE_PROVIDER_ID")
            .ok()
            .map(|v| truncate_ascii(v.trim(), 64))
            .filter(|v| !v.is_empty())
            .ok_or_else(|| "missing SELENE_PH1D_LIVE_PROVIDER_ID".to_string())?;
        let model_id = env::var("SELENE_PH1D_LIVE_MODEL_ID")
            .ok()
            .map(|v| truncate_ascii(v.trim(), 128))
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| "ph1d_live_model_default".to_string());
        Ok(Self {
            provider_id,
            model_id,
        })
    }
}

impl Ph1dProviderAdapter for EnvPh1dLiveAdapter {
    fn execute(
        &self,
        _req: &Ph1dProviderCallRequest,
    ) -> Result<Ph1dProviderCallResponse, Ph1dProviderAdapterError> {
        Err(Ph1dProviderAdapterError::terminal(format!(
            "ph1d live provider adapter unavailable for provider={} model={}",
            self.provider_id, self.model_id
        )))
    }
}

#[derive(Debug, Clone)]
struct RecordingPh1dProviderAdapter<'a, A>
where
    A: Ph1dProviderAdapter,
{
    inner: &'a A,
    records: Arc<Mutex<Vec<Ph1dProviderCallResponse>>>,
}

impl<'a, A> RecordingPh1dProviderAdapter<'a, A>
where
    A: Ph1dProviderAdapter,
{
    fn new(inner: &'a A, records: Arc<Mutex<Vec<Ph1dProviderCallResponse>>>) -> Self {
        Self { inner, records }
    }
}

impl<A> Ph1dProviderAdapter for RecordingPh1dProviderAdapter<'_, A>
where
    A: Ph1dProviderAdapter,
{
    fn execute(
        &self,
        req: &Ph1dProviderCallRequest,
    ) -> Result<Ph1dProviderCallResponse, Ph1dProviderAdapterError> {
        let out = self.inner.execute(req)?;
        if let Ok(mut records) = self.records.lock() {
            records.push(out.clone());
        }
        Ok(out)
    }
}

#[derive(Debug, Clone)]
struct Ph1kLiveSignalBundle {
    locale_tag: InterruptLocaleTag,
    processed_stream_ref: AudioStreamRef,
    pre_roll_buffer_ref: PreRollBufferRef,
    vad_events: Vec<VadEvent>,
    device_state: DeviceState,
    timing_stats: Ph1kTimingStats,
    tts_playback: TtsPlaybackActiveEvent,
    interrupt_input: InterruptInput,
    interrupt_decision: selene_engines::ph1k::InterruptDecisionTrace,
    ph1c_handoff: Ph1kToPh1cHandoff,
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
        if !cfg!(test) {
            return Self::default_from_env().unwrap_or_else(|err| {
                panic!("selene_adapter persistent bootstrap required for runtime: {err}")
            });
        }
        let ph1d_live_adapter = build_ph1d_live_adapter_from_env();
        Self {
            ingress: AppServerIngressRuntime::default(),
            store: Arc::new(Mutex::new(Ph1fStore::new_in_memory())),
            session_retry_cache: Arc::new(Mutex::new(BTreeMap::new())),
            sync_worker_counters: Arc::new(Mutex::new(AdapterSyncWorkerCounters::default())),
            improvement_counters: Arc::new(Mutex::new(AdapterImprovementCounters::default())),
            transcript_state: Arc::new(Mutex::new(AdapterTranscriptState::default())),
            report_display_target_defaults: Arc::new(Mutex::new(BTreeMap::new())),
            auto_builder_enabled: true,
            ph1c_live_enabled: parse_bool_env("SELENE_PH1C_LIVE_ENABLED", true),
            ph1c_streaming_enabled: parse_bool_env("SELENE_PH1C_STREAMING_ENABLED", true),
            ph1c_runtime: EnginePh1cRuntime::new(EnginePh1cConfig::mvp_desktop_v1()),
            ph1d_runtime: EnginePh1dRuntime::new(selene_engines::ph1d::Ph1dConfig::mvp_v1()),
            ph1d_live_adapter,
            persistence: None,
            runtime_node_id: runtime_node_id_from_env(),
            session_lease_ttl_ms: parse_u64_env("SELENE_SESSION_LEASE_TTL_MS", 30_000),
        }
    }
}

impl AdapterRuntime {
    pub fn new(ingress: AppServerIngressRuntime, store: Arc<Mutex<Ph1fStore>>) -> Self {
        if !cfg!(test) {
            let journal_path = env::var("SELENE_ADAPTER_STORE_PATH")
                .ok()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
                .unwrap_or_else(default_adapter_store_path);
            let auto_builder_enabled = parse_auto_builder_enabled_from_env();
            return Self::new_with_persistence(ingress, store, journal_path, auto_builder_enabled)
                .unwrap_or_else(|err| {
                    panic!("selene_adapter persistent bootstrap required for runtime: {err}")
                });
        }
        let ph1d_live_adapter = build_ph1d_live_adapter_from_env();
        Self {
            ingress,
            store,
            session_retry_cache: Arc::new(Mutex::new(BTreeMap::new())),
            sync_worker_counters: Arc::new(Mutex::new(AdapterSyncWorkerCounters::default())),
            improvement_counters: Arc::new(Mutex::new(AdapterImprovementCounters::default())),
            transcript_state: Arc::new(Mutex::new(AdapterTranscriptState::default())),
            report_display_target_defaults: Arc::new(Mutex::new(BTreeMap::new())),
            auto_builder_enabled: true,
            ph1c_live_enabled: parse_bool_env("SELENE_PH1C_LIVE_ENABLED", true),
            ph1c_streaming_enabled: parse_bool_env("SELENE_PH1C_STREAMING_ENABLED", true),
            ph1c_runtime: EnginePh1cRuntime::new(EnginePh1cConfig::mvp_desktop_v1()),
            ph1d_runtime: EnginePh1dRuntime::new(selene_engines::ph1d::Ph1dConfig::mvp_v1()),
            ph1d_live_adapter,
            persistence: None,
            runtime_node_id: runtime_node_id_from_env(),
            session_lease_ttl_ms: parse_u64_env("SELENE_SESSION_LEASE_TTL_MS", 30_000),
        }
    }

    pub fn new_with_persistence(
        ingress: AppServerIngressRuntime,
        store: Arc<Mutex<Ph1fStore>>,
        journal_path: PathBuf,
        auto_builder_enabled: bool,
    ) -> Result<Self, String> {
        let ph1d_live_adapter = build_ph1d_live_adapter_from_env();
        let persistence = AdapterPersistenceRuntime {
            legacy_journal_path: journal_path.clone(),
            state_path: adapter_persistence_state_path(&journal_path),
            state: Arc::new(Mutex::new(AdapterPersistenceState::default())),
        };
        let runtime = Self {
            ingress,
            store,
            session_retry_cache: Arc::new(Mutex::new(BTreeMap::new())),
            sync_worker_counters: Arc::new(Mutex::new(AdapterSyncWorkerCounters::default())),
            improvement_counters: Arc::new(Mutex::new(AdapterImprovementCounters::default())),
            transcript_state: Arc::new(Mutex::new(AdapterTranscriptState::default())),
            report_display_target_defaults: Arc::new(Mutex::new(BTreeMap::new())),
            auto_builder_enabled,
            ph1c_live_enabled: parse_bool_env("SELENE_PH1C_LIVE_ENABLED", true),
            ph1c_streaming_enabled: parse_bool_env("SELENE_PH1C_STREAMING_ENABLED", true),
            ph1c_runtime: EnginePh1cRuntime::new(EnginePh1cConfig::mvp_desktop_v1()),
            ph1d_runtime: EnginePh1dRuntime::new(selene_engines::ph1d::Ph1dConfig::mvp_v1()),
            ph1d_live_adapter,
            persistence: Some(persistence),
            runtime_node_id: runtime_node_id_from_env(),
            session_lease_ttl_ms: parse_u64_env("SELENE_SESSION_LEASE_TTL_MS", 30_000),
        };
        runtime.ensure_persistence_ready()?;
        runtime.bootstrap_persistence_runtime()?;
        Ok(runtime)
    }

    pub fn run_voice_turn(
        &self,
        request: VoiceTurnAdapterRequest,
    ) -> Result<VoiceTurnAdapterResponse, String> {
        self.run_voice_turn_internal(
            request,
            None,
            true,
            true,
            PersistenceInvocationMode::Standard,
        )
        .map_err(|err| err.to_runtime_reason())
    }

    pub fn run_voice_turn_ingress(
        &self,
        request: VoiceTurnAdapterRequest,
    ) -> Result<VoiceTurnAdapterResponse, VoiceTurnIngressError> {
        let runtime_execution_envelope =
            fallback_runtime_execution_envelope_for_voice_turn_request(&request).map_err(
                |err| {
                    voice_turn_ingress_error(
                        FailureClass::InvalidPayload,
                        "INVALID_RUNTIME_EXECUTION_ENVELOPE".to_string(),
                        Some(format!("invalid runtime_execution_envelope: {err:?}")),
                        None,
                        Some(request.turn_id),
                        None,
                    )
                },
            )?;
        self.run_voice_turn_internal(
            request,
            Some(runtime_execution_envelope),
            true,
            false,
            PersistenceInvocationMode::Standard,
        )
    }

    pub fn run_voice_turn_ingress_with_execution_envelope(
        &self,
        request: VoiceTurnAdapterRequest,
        runtime_execution_envelope: RuntimeExecutionEnvelope,
    ) -> Result<VoiceTurnAdapterResponse, VoiceTurnIngressError> {
        self.run_voice_turn_internal(
            request,
            Some(runtime_execution_envelope),
            true,
            false,
            PersistenceInvocationMode::Standard,
        )
    }

    pub fn run_invite_link_open_and_start_onboarding(
        &self,
        request: InviteLinkOpenAdapterRequest,
    ) -> Result<InviteLinkOpenAdapterResponse, String> {
        let correlation_id = CorrelationId(u128::from(request.correlation_id));
        let app_platform = parse_app_platform(&request.app_platform)?;
        let token_id = TokenId::new(request.token_id.clone())
            .map_err(|err| format!("invalid token_id: {err:?}"))?;
        let ingress_request = AppInviteLinkOpenRequest::v1(
            correlation_id,
            request.idempotency_key,
            token_id,
            request.token_signature,
            request.tenant_id,
            app_platform,
            request.device_fingerprint,
            request.app_instance_id,
            request.deep_link_nonce,
        )
        .map_err(|err| format!("invalid invite_link_open request: {err:?}"))?;

        let mut store = self
            .store
            .lock()
            .map_err(|_| "adapter store lock poisoned".to_string())?;
        let now = MonotonicTimeNs(system_time_now_ns().max(1));
        let outcome = self
            .ingress
            .run_invite_link_open_and_start_onboarding(&mut store, ingress_request, now)
            .map_err(storage_error_to_string)?;

        Ok(InviteLinkOpenAdapterResponse {
            status: "ok".to_string(),
            outcome: "ONBOARDING_STARTED".to_string(),
            reason: None,
            onboarding_session_id: Some(outcome.onboarding_session_id),
            next_step: Some(onboarding_next_step_to_api_value(outcome.next_step)),
            required_fields: outcome.required_fields,
            required_verification_gates: outcome.required_verification_gates,
        })
    }

    pub fn run_onboarding_continue(
        &self,
        request: OnboardingContinueAdapterRequest,
    ) -> Result<OnboardingContinueAdapterResponse, String> {
        let correlation_id = CorrelationId(u128::from(request.correlation_id));
        let onboarding_session_id = OnboardingSessionId::new(request.onboarding_session_id)
            .map_err(|err| format!("invalid onboarding_session_id: {err:?}"))?;
        let action = parse_onboarding_continue_action(
            &request.action,
            OnboardingContinueActionInput {
                field_value: request.field_value,
                receipt_kind: request.receipt_kind,
                receipt_ref: request.receipt_ref,
                signer: request.signer,
                payload_hash: request.payload_hash,
                terms_version_id: request.terms_version_id,
                accepted: request.accepted,
                device_id: request.device_id,
                proof_ok: request.proof_ok,
                sample_seed: request.sample_seed,
                photo_blob_ref: request.photo_blob_ref,
                sender_decision: request.sender_decision,
            },
        )?;
        let ingress_request = AppOnboardingContinueRequest::v1(
            correlation_id,
            onboarding_session_id,
            request.idempotency_key,
            request.tenant_id,
            action,
        )
        .map_err(|err| format!("invalid onboarding_continue request: {err:?}"))?;

        let mut store = self
            .store
            .lock()
            .map_err(|_| "adapter store lock poisoned".to_string())?;
        let now = MonotonicTimeNs(system_time_now_ns().max(1));
        let outcome = self
            .ingress
            .run_onboarding_continue(&mut store, ingress_request, now)
            .map_err(storage_error_to_string)?;

        Ok(OnboardingContinueAdapterResponse {
            status: "ok".to_string(),
            outcome: "ONBOARDING_CONTINUED".to_string(),
            reason: None,
            onboarding_session_id: Some(outcome.onboarding_session_id),
            next_step: Some(onboarding_continue_next_step_to_api_value(
                outcome.next_step,
            )),
            blocking_field: outcome.blocking_field,
            blocking_question: outcome.blocking_question,
            remaining_missing_fields: outcome.remaining_missing_fields,
            remaining_platform_receipt_kinds: outcome.remaining_platform_receipt_kinds,
            voice_artifact_sync_receipt_ref: outcome.voice_artifact_sync_receipt_ref,
            access_engine_instance_id: outcome.access_engine_instance_id,
            onboarding_status: outcome
                .onboarding_status
                .map(|status| format!("{status:?}").to_ascii_uppercase()),
        })
    }

    pub fn run_session_resume(
        &self,
        request: SessionResumeAdapterRequest,
    ) -> Result<SessionResumeAdapterResponse, String> {
        let correlation_id = CorrelationId(u128::from(request.correlation_id));
        let session_id = request
            .session_id
            .parse::<u128>()
            .map(SessionId)
            .map_err(|err| format!("invalid session_id: {err}"))?;
        let device_id = DeviceId::new(request.device_id.clone())
            .map_err(|err| format!("invalid device_id: {err:?}"))?;
        let ingress_request = AppSessionResumeRequest::v1(
            correlation_id,
            request.idempotency_key,
            session_id,
            device_id,
        )
        .map_err(|err| format!("invalid session_resume request: {err:?}"))?;

        let mut store = self
            .store
            .lock()
            .map_err(|_| "adapter store lock poisoned".to_string())?;
        let now = MonotonicTimeNs(system_time_now_ns().max(1));
        let outcome = self
            .ingress
            .run_session_resume(&mut store, ingress_request, now)
            .map_err(storage_error_to_string)?;

        Ok(SessionResumeAdapterResponse {
            status: "ok".to_string(),
            outcome: "SESSION_RESUMED".to_string(),
            reason: None,
            session_id: Some(outcome.session_id),
            session_state: Some(session_state_to_api_value(outcome.session_state)),
            session_attach_outcome: Some(session_attach_outcome_to_api_value(
                outcome.session_attach_outcome,
            )),
        })
    }

    pub fn run_session_recover(
        &self,
        request: SessionRecoverAdapterRequest,
    ) -> Result<SessionRecoverAdapterResponse, String> {
        let correlation_id = CorrelationId(u128::from(request.correlation_id));
        let session_id = request
            .session_id
            .parse::<u128>()
            .map(SessionId)
            .map_err(|err| format!("invalid session_id: {err}"))?;
        let device_id = DeviceId::new(request.device_id.clone())
            .map_err(|err| format!("invalid device_id: {err:?}"))?;
        let ingress_request = AppSessionRecoverRequest::v1(
            correlation_id,
            request.idempotency_key,
            session_id,
            device_id,
        )
        .map_err(|err| format!("invalid session_recover request: {err:?}"))?;

        let mut store = self
            .store
            .lock()
            .map_err(|_| "adapter store lock poisoned".to_string())?;
        let now = MonotonicTimeNs(system_time_now_ns().max(1));
        let outcome = self
            .ingress
            .run_session_recover(&mut store, ingress_request, now)
            .map_err(storage_error_to_string)?;

        Ok(SessionRecoverAdapterResponse {
            status: "ok".to_string(),
            outcome: "SESSION_RECOVERED".to_string(),
            reason: None,
            session_id: Some(outcome.session_id),
            session_state: Some(session_state_to_api_value(outcome.session_state)),
            session_attach_outcome: Some(session_attach_outcome_to_api_value(
                outcome.session_attach_outcome,
            )),
        })
    }

    pub fn run_wake_profile_availability_refresh(
        &self,
        request: WakeProfileAvailabilityRefreshAdapterRequest,
    ) -> Result<WakeProfileAvailabilityRefreshAdapterResponse, String> {
        let correlation_id = CorrelationId(u128::from(request.correlation_id));
        let device_id = DeviceId::new(request.device_id.clone())
            .map_err(|err| format!("invalid device_id: {err:?}"))?;
        let ingress_request = AppWakeProfileAvailabilityRefreshRequest::v1(
            correlation_id,
            request.idempotency_key,
            device_id,
            request.expected_wake_profile_id,
            request.voice_artifact_sync_receipt_ref,
        )
        .map_err(|err| format!("invalid wake_profile_availability request: {err:?}"))?;

        let mut store = self
            .store
            .lock()
            .map_err(|_| "adapter store lock poisoned".to_string())?;
        let now = MonotonicTimeNs(system_time_now_ns().max(1));
        let outcome = self
            .ingress
            .run_wake_profile_availability_refresh(&mut store, ingress_request, now)
            .map_err(storage_error_to_string)?;

        Ok(WakeProfileAvailabilityRefreshAdapterResponse {
            status: "ok".to_string(),
            outcome: outcome.outcome,
            reason: outcome.reason,
            device_id: Some(outcome.device_id),
            wake_profile_id: Some(outcome.wake_profile_id),
            active_wake_artifact_version: outcome.active_wake_artifact_version,
            activated_count: outcome.activated_count,
            noop_count: outcome.noop_count,
            pull_error_count: outcome.pull_error_count,
        })
    }

    pub fn debug_last_agent_packet_voice_identity_assertion(&self) -> Option<String> {
        self.ingress
            .debug_last_agent_input_packet()
            .and_then(|packet| packet.runtime_execution_envelope)
            .and_then(|envelope| envelope.voice_identity_assertion)
            .map(|assertion| format!("{assertion:?}"))
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
                return Err(
                    "invalid health detail date range: from_utc_ns is after to_utc_ns".to_string(),
                );
            }
        }
        let now_ns = now_ns.unwrap_or_else(system_time_now_ns).max(1);
        let health = self.health_report(Some(now_ns))?;
        let mut detail = build_ui_health_detail_response(&health, check_id, now_ns)?;
        detail.issues = filter_health_issues(&detail.issues, &filter);
        detail.active_issue_id =
            select_active_issue_id(&detail.issues, filter.selected_issue_id.as_deref());
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

        let display_resolution = resolve_report_display_target(
            request.display_target.as_deref(),
            remembered_target.as_deref(),
        );
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
        let time_range =
            match HealthReportTimeRange::v1(MonotonicTimeNs(from_ns), MonotonicTimeNs(to_ns)) {
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
        session_id: Option<SessionId>,
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
                session_id,
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
                session_id,
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

    fn record_reused_response_transcript_finals(
        &self,
        update: ReusedResponseTranscriptUpdate<'_>,
    ) -> Result<(), String> {
        if update.user_text_final.is_none() && update.selene_text_final.is_none() {
            return Ok(());
        }
        let mut store = self
            .store
            .lock()
            .map_err(|_| "adapter store lock poisoned".to_string())?;
        self.record_transcript_updates(
            &mut store,
            update.now,
            update.correlation_id,
            update.turn_id,
            update.actor_user_id,
            Some(update.device_id),
            adapter_response_session_id(update.response)?,
            None,
            update.user_text_final.map(str::to_string),
            None,
            update.selene_text_final.map(str::to_string),
        )
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
            SyncImprovementBuilderContext {
                now,
                correlation_id,
                turn_id,
                metrics: &metrics,
                queue_after: &queue_after,
                outcome_entries: &improvement.builder_input_entries,
            },
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
                // PH1.LEARN appends downstream artifacts; PH1.BUILDER owns ACTIVE promotion.
                ArtifactStatus::Deprecated,
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
        ctx: SyncImprovementBuilderContext<'_>,
    ) -> Result<(), String> {
        if !self.auto_builder_enabled {
            self.record_builder_status("DISABLED", BuilderStatusKind::NotInvoked)?;
            return Ok(());
        }
        if ctx.outcome_entries.is_empty() {
            self.record_builder_status("NO_SYNC_ISSUES", BuilderStatusKind::NotInvoked)?;
            return Ok(());
        }
        let severe = ctx.metrics.dead_lettered_count > 0 || ctx.queue_after.replay_due_count > 0;
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
        let window_start = MonotonicTimeNs(ctx.now.0.saturating_sub(60_000_000_000));
        let builder_input = BuilderOfflineInput::v1(
            ctx.correlation_id,
            ctx.turn_id,
            window_start,
            ctx.now,
            ctx.now,
            ctx.outcome_entries.to_vec(),
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

    #[allow(clippy::too_many_arguments)]
    fn emit_read_only_lane_incidents_and_maybe_run_builder(
        &self,
        store: &mut Ph1fStore,
        now: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        actor_user_id: &UserId,
        tenant_id: Option<&str>,
        device_id: &DeviceId,
        user_text_final: Option<&str>,
        execution_outcome: &AppVoiceTurnExecutionOutcome,
    ) -> Result<(), String> {
        let Some(tenant_id) = tenant_id else {
            return Ok(());
        };
        let incidents = detect_read_only_turn_incidents(user_text_final, execution_outcome);
        if incidents.is_empty() {
            return Ok(());
        }

        let mut feedback_events_emitted = 0u64;
        let mut learn_artifacts_emitted = 0u64;
        let mut builder_input_entries = Vec::new();
        let mut severe_incident_observed = false;

        for incident in incidents {
            severe_incident_observed |= incident.kind.severe();
            let feedback_event_type = feedback_event_type_str(incident.kind.feedback_event_type());
            let learn_signal_type = learn_signal_type_str(incident.kind.learn_signal_type());
            let issue_tag = incident.kind.tag();

            let feedback_idem = sanitize_idempotency_token(&format!(
                "ro_feedback_{}_{}_{}",
                issue_tag, correlation_id.0, turn_id.0
            ));
            match store.ph1feedback_event_commit(
                now,
                tenant_id.to_string(),
                correlation_id,
                turn_id,
                None,
                actor_user_id.clone(),
                device_id.clone(),
                feedback_event_type.to_string(),
                learn_signal_type.to_string(),
                incident.reason_code,
                feedback_idem,
            ) {
                Ok(_) => {
                    feedback_events_emitted = feedback_events_emitted.saturating_add(1);
                }
                Err(err) => {
                    eprintln!(
                        "selene_adapter read-only feedback emit failed: {}",
                        storage_error_to_string(err)
                    );
                }
            }

            let learn_idem = sanitize_idempotency_token(&format!(
                "ro_learn_{}_{}_{}",
                issue_tag, correlation_id.0, turn_id.0
            ));
            match store.ph1feedback_learn_signal_bundle_commit(
                now,
                tenant_id.to_string(),
                correlation_id,
                turn_id,
                None,
                actor_user_id.clone(),
                device_id.clone(),
                feedback_event_type.to_string(),
                learn_signal_type.to_string(),
                incident.reason_code,
                incident.evidence_ref.clone(),
                incident.provenance_ref.clone(),
                0,
                learn_idem,
            ) {
                Ok(_) => {
                    learn_artifacts_emitted = learn_artifacts_emitted.saturating_add(1);
                }
                Err(err) => {
                    eprintln!(
                        "selene_adapter read-only learn bundle emit failed: {}",
                        storage_error_to_string(err)
                    );
                }
            }

            let outcome_idem = sanitize_idempotency_token(&format!(
                "ro_outcome_{}_{}_{}",
                issue_tag, correlation_id.0, turn_id.0
            ));
            if let Err(err) =
                store.append_outcome_utilization_ledger_row(OutcomeUtilizationLedgerRowInput {
                    created_at: now,
                    correlation_id,
                    turn_id,
                    engine_id: "PH1.FEEDBACK".to_string(),
                    outcome_type: incident.kind.outcome_type().to_string(),
                    action_class: OsOutcomeActionClass::QueueLearn,
                    consumed_by: "PH1.LEARN".to_string(),
                    latency_cost_ms: 120,
                    decision_delta: true,
                    reason_code: incident.reason_code,
                    idempotency_key: Some(outcome_idem.clone()),
                })
            {
                eprintln!(
                    "selene_adapter read-only outcome utilization append failed: {}",
                    storage_error_to_string(err)
                );
                continue;
            }

            match OsOutcomeUtilizationEntry::v1(
                "PH1.FEEDBACK".to_string(),
                incident.kind.outcome_type().to_string(),
                correlation_id,
                turn_id,
                OsOutcomeActionClass::QueueLearn,
                "PH1.LEARN".to_string(),
                120,
                true,
                incident.reason_code,
            ) {
                Ok(entry) => builder_input_entries.push(entry),
                Err(err) => {
                    eprintln!("selene_adapter read-only outcome entry build failed: {err:?}");
                }
            }
        }

        let emission = SyncImprovementEmissionResult {
            feedback_events_emitted,
            learn_artifacts_emitted,
            builder_input_entries,
        };
        if let Err(err) = self.record_sync_improvement_metrics(&emission) {
            eprintln!(
                "selene_adapter read-only incident metrics update failed: {}",
                err
            );
        }
        self.maybe_run_builder_for_read_only_incidents(
            store,
            now,
            correlation_id,
            turn_id,
            severe_incident_observed,
            &emission.builder_input_entries,
        )?;

        Ok(())
    }

    fn maybe_run_builder_for_read_only_incidents(
        &self,
        store: &mut Ph1fStore,
        now: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        severe_incident_observed: bool,
        outcome_entries: &[OsOutcomeUtilizationEntry],
    ) -> Result<(), String> {
        if !self.auto_builder_enabled {
            self.record_builder_status("DISABLED", BuilderStatusKind::NotInvoked)?;
            return Ok(());
        }
        if outcome_entries.is_empty() {
            self.record_builder_status("NO_READ_ONLY_INCIDENTS", BuilderStatusKind::NotInvoked)?;
            return Ok(());
        }
        if !severe_incident_observed {
            self.record_builder_status(
                "SKIPPED_NON_SEVERE_READ_ONLY",
                BuilderStatusKind::NotInvoked,
            )?;
            return Ok(());
        }

        self.record_builder_status("RUNNING_READ_ONLY", BuilderStatusKind::RunStarted)?;
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
                self.record_builder_status("COMPLETED_READ_ONLY", BuilderStatusKind::Completed)?;
            }
            Ok(BuilderOrchestrationOutcome::Refused(refuse)) => {
                self.record_builder_status(
                    &format!(
                        "REFUSED_READ_ONLY:{}:{}",
                        refuse.stage, refuse.reason_code.0
                    ),
                    BuilderStatusKind::Refused,
                )?;
            }
            Ok(BuilderOrchestrationOutcome::NotInvokedDisabled) => {
                self.record_builder_status(
                    "NOT_INVOKED_DISABLED_READ_ONLY",
                    BuilderStatusKind::NotInvoked,
                )?;
            }
            Ok(BuilderOrchestrationOutcome::NotInvokedNoSignals) => {
                self.record_builder_status(
                    "NOT_INVOKED_NO_SIGNALS_READ_ONLY",
                    BuilderStatusKind::NotInvoked,
                )?;
            }
            Err(err) => {
                self.record_builder_status(
                    &format!("ERROR_READ_ONLY:{err:?}"),
                    BuilderStatusKind::Error,
                )?;
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

    fn run_ph1c_live_turn(
        &self,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        actor_user_id: &UserId,
        tenant_id: Option<&str>,
        session_state: SessionState,
        ph1k: &Ph1kLiveSignalBundle,
    ) -> Option<Ph1cLiveTurnOutcomeSummary> {
        if !self.ph1c_live_enabled {
            return None;
        }
        let Some(adapter) = self.ph1d_live_adapter.as_ref() else {
            return Some(ph1c_live_reject_summary(
                ph1c_reason_codes::STT_FAIL_PROVIDER_CIRCUIT_OPEN,
                Ph1cRetryAdvice::SwitchToText,
            ));
        };
        let tenant_id = tenant_id.unwrap_or("tenant_default");
        let ph1c_request = match build_ph1c_live_request(ph1k, session_state) {
            Ok(req) => req,
            Err(_) => {
                return Some(ph1c_live_reject_summary(
                    ph1c_reason_codes::STT_FAIL_POLICY_RESTRICTED,
                    Ph1cRetryAdvice::SwitchToText,
                ));
            }
        };
        let mut live = Ph1cLiveProviderContext::mvp_openai_google_v1(
            correlation_id_to_u64(correlation_id),
            turn_id.0.max(1),
            truncate_ascii(tenant_id, 64),
        );
        live.idempotency_key =
            sanitize_idempotency_token(&format!("ph1c_live_{}_{}", correlation_id.0, turn_id.0));
        live.tenant_vocabulary_pack_id =
            Some(format!("tenant_vocab_{}", truncate_ascii(tenant_id, 48)));
        live.user_vocabulary_pack_id = Some(format!(
            "user_vocab_{}",
            truncate_ascii(actor_user_id.as_str(), 48)
        ));
        let provider_records = Arc::new(Mutex::new(Vec::<Ph1dProviderCallResponse>::new()));
        let recording_adapter =
            RecordingPh1dProviderAdapter::new(adapter, Arc::clone(&provider_records));

        if self.ph1c_streaming_enabled {
            let stream_commit = self.ph1c_runtime.run_stream_via_live_provider_adapter(
                &ph1c_request,
                &live,
                &recording_adapter,
            );
            return Some(summarize_ph1c_stream_commit(
                stream_commit,
                snapshot_provider_calls(&provider_records),
            ));
        }

        let response = self.ph1c_runtime.run_via_live_provider_adapter(
            &ph1c_request,
            &live,
            &recording_adapter,
        );
        let final_text = match &response {
            Ph1cResponse::TranscriptOk(ok) => Some(ok.transcript_text.clone()),
            Ph1cResponse::TranscriptReject(_) => None,
        };
        Some(Ph1cLiveTurnOutcomeSummary {
            response,
            partial_text: None,
            final_text,
            finalized: true,
            low_latency_commit: false,
            provider_call_trace: snapshot_provider_calls(&provider_records),
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn commit_ph1c_live_outcome(
        &self,
        store: &mut Ph1fStore,
        now: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        actor_user_id: &UserId,
        tenant_id: Option<&str>,
        device_id: Option<&DeviceId>,
        session_id: Option<SessionId>,
        ph1c: &Ph1cLiveTurnOutcomeSummary,
    ) -> Result<(), String> {
        let (Some(tenant_id), Some(device_id)) = (tenant_id, device_id) else {
            return Ok(());
        };

        match &ph1c.response {
            Ph1cResponse::TranscriptOk(ok) => {
                let idempotency_key = sanitize_idempotency_token(&format!(
                    "ph1c_ok_{}_{}",
                    correlation_id.0, turn_id.0
                ));
                store
                    .ph1c_transcript_ok_commit(
                        now,
                        tenant_id.to_string(),
                        correlation_id,
                        turn_id,
                        session_id,
                        actor_user_id.clone(),
                        device_id.clone(),
                        ok.transcript_text.clone(),
                        stable_hash_hex_16(&ok.transcript_text),
                        ok.language_tag.clone(),
                        ok.confidence_bucket,
                        idempotency_key,
                    )
                    .map_err(storage_error_to_string)?;
            }
            Ph1cResponse::TranscriptReject(reject) => {
                let idempotency_key = sanitize_idempotency_token(&format!(
                    "ph1c_reject_{}_{}",
                    correlation_id.0, turn_id.0
                ));
                let transcript_hash = ph1c.final_text.as_ref().map(|v| stable_hash_hex_16(v));
                store
                    .ph1c_transcript_reject_commit(
                        now,
                        tenant_id.to_string(),
                        correlation_id,
                        turn_id,
                        session_id,
                        actor_user_id.clone(),
                        device_id.clone(),
                        reject.reason_code,
                        reject.retry_advice,
                        transcript_hash,
                        idempotency_key,
                    )
                    .map_err(storage_error_to_string)?;
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_ph1c_gold_capture_and_learning(
        &self,
        store: &mut Ph1fStore,
        now: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        actor_user_id: &UserId,
        tenant_id: Option<&str>,
        device_id: Option<&DeviceId>,
        session_id: Option<SessionId>,
        ph1c: &Ph1cLiveTurnOutcomeSummary,
    ) -> Result<(), String> {
        let (Some(tenant_id), Some(device_id)) = (tenant_id, device_id) else {
            return Ok(());
        };
        let (feedback_event_type, reason_code) = match &ph1c.response {
            Ph1cResponse::TranscriptReject(reject) => {
                (FeedbackEventType::SttReject, reject.reason_code)
            }
            Ph1cResponse::TranscriptOk(_) => return Ok(()),
        };
        let Some((feedback_event_type, learn_signal_type)) =
            feedback_learn_pair_for_ph1c_capture(feedback_event_type)
        else {
            return Ok(());
        };
        let feedback_idem = sanitize_idempotency_token(&format!(
            "ph1c_feedback_{}_{}",
            correlation_id.0, turn_id.0
        ));
        store
            .ph1feedback_event_commit(
                now,
                tenant_id.to_string(),
                correlation_id,
                turn_id,
                session_id,
                actor_user_id.clone(),
                device_id.clone(),
                feedback_event_type.to_string(),
                learn_signal_type.to_string(),
                reason_code,
                feedback_idem,
            )
            .map_err(storage_error_to_string)?;

        let ingest_latency_ms = match &ph1c.response {
            Ph1cResponse::TranscriptOk(ok) => ok
                .audit_meta
                .as_ref()
                .map(|meta| meta.total_latency_ms.min(2_000))
                .unwrap_or(0),
            Ph1cResponse::TranscriptReject(reject) => reject
                .audit_meta
                .as_ref()
                .map(|meta| meta.total_latency_ms.min(2_000))
                .unwrap_or(0),
        };
        let learn_idem =
            sanitize_idempotency_token(&format!("ph1c_learn_{}_{}", correlation_id.0, turn_id.0));
        let evidence_ref = truncate_ascii(
            ph1c.final_text
                .as_deref()
                .unwrap_or("ph1c_transcript_unavailable"),
            128,
        );
        store
            .ph1feedback_learn_signal_bundle_commit(
                now,
                tenant_id.to_string(),
                correlation_id,
                turn_id,
                session_id,
                actor_user_id.clone(),
                device_id.clone(),
                feedback_event_type.to_string(),
                learn_signal_type.to_string(),
                reason_code,
                evidence_ref.clone(),
                truncate_ascii(&format!("ph1c_capture:{evidence_ref}"), 128),
                ingest_latency_ms,
                learn_idem,
            )
            .map_err(storage_error_to_string)?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn commit_ph1d_runtime_outcome(
        &self,
        store: &mut Ph1fStore,
        now: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        actor_user_id: &UserId,
        tenant_id: Option<&str>,
        device_id: Option<&DeviceId>,
        session_id: Option<SessionId>,
        session_state: SessionState,
        transcript_text: Option<&str>,
        os_outcome: &OsVoiceLiveTurnOutcome,
    ) -> Result<(), String> {
        let (Some(tenant_id), Some(device_id)) = (tenant_id, device_id) else {
            return Ok(());
        };

        let transcript_text = transcript_text
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .unwrap_or("transcript_unavailable");
        let transcript_ok = Ph1cTranscriptOk::v1(
            transcript_text.to_string(),
            LanguageTag::new("en").map_err(|err| format!("invalid ph1d language tag: {err:?}"))?,
            if transcript_text == "transcript_unavailable" {
                Ph1cConfidenceBucket::Low
            } else {
                Ph1cConfidenceBucket::High
            },
        )
        .map_err(|err| format!("ph1d transcript envelope build failed: {err:?}"))?;
        let nlp_output = Ph1nResponse::Chat(
            Ph1nChat::v1(
                transcript_text.to_string(),
                ph1d_reason_codes::D_PROVIDER_OK,
            )
            .map_err(|err| format!("ph1d nlp envelope build failed: {err:?}"))?,
        );
        let request = selene_kernel_contracts::ph1d::Ph1dRequest::v1(
            transcript_ok,
            nlp_output,
            Ph1cSessionStateRef::v1(session_state, false),
            PolicyContextRef::v1(false, false, SafetyTier::Standard),
            ToolCatalogRef::v1(vec![
                ToolName::Time,
                ToolName::Weather,
                ToolName::WebSearch,
                ToolName::News,
                ToolName::UrlFetchAndCite,
                ToolName::DocumentUnderstand,
                ToolName::PhotoUnderstand,
                ToolName::DataAnalysis,
                ToolName::DeepResearch,
                ToolName::RecordMode,
            ])
            .map_err(|err| format!("ph1d tool catalog build failed: {err:?}"))?,
        )
        .map_err(|err| format!("ph1d request build failed: {err:?}"))?;
        let response = self
            .ph1d_runtime
            .run(&request, ph1d_model_outcome_from_os_outcome(os_outcome));

        match response {
            Ph1dResponse::Ok(Ph1dOk::Chat(chat)) => {
                store
                    .ph1d_chat_commit(
                        now,
                        tenant_id.to_string(),
                        correlation_id,
                        turn_id,
                        session_id,
                        actor_user_id.clone(),
                        device_id.clone(),
                        chat.reason_code,
                        sanitize_idempotency_token(&format!(
                            "ph1d_chat:{}:{}",
                            correlation_id.0, turn_id.0
                        )),
                    )
                    .map_err(storage_error_to_string)?;
            }
            Ph1dResponse::Ok(Ph1dOk::Intent(intent)) => {
                store
                    .ph1d_intent_commit(
                        now,
                        tenant_id.to_string(),
                        correlation_id,
                        turn_id,
                        session_id,
                        actor_user_id.clone(),
                        device_id.clone(),
                        truncate_ascii(&format!("{:?}", intent.refined_intent_type), 64),
                        intent.reason_code,
                        sanitize_idempotency_token(&format!(
                            "ph1d_intent:{}:{}",
                            correlation_id.0, turn_id.0
                        )),
                    )
                    .map_err(storage_error_to_string)?;
            }
            Ph1dResponse::Ok(Ph1dOk::Clarify(clarify)) => {
                let missing = clarify
                    .what_is_missing
                    .first()
                    .map(|field| format!("{field:?}"))
                    .unwrap_or_else(|| "Task".to_string());
                store
                    .ph1d_clarify_commit(
                        now,
                        tenant_id.to_string(),
                        correlation_id,
                        turn_id,
                        session_id,
                        actor_user_id.clone(),
                        device_id.clone(),
                        truncate_ascii(&missing, 64),
                        clarify.reason_code,
                        sanitize_idempotency_token(&format!(
                            "ph1d_clarify:{}:{}",
                            correlation_id.0, turn_id.0
                        )),
                    )
                    .map_err(storage_error_to_string)?;
            }
            Ph1dResponse::Ok(Ph1dOk::Analysis(analysis)) => {
                store
                    .ph1d_analysis_commit(
                        now,
                        tenant_id.to_string(),
                        correlation_id,
                        turn_id,
                        session_id,
                        actor_user_id.clone(),
                        device_id.clone(),
                        truncate_ascii(&analysis.short_analysis, 64),
                        analysis.reason_code,
                        sanitize_idempotency_token(&format!(
                            "ph1d_analysis:{}:{}",
                            correlation_id.0, turn_id.0
                        )),
                    )
                    .map_err(storage_error_to_string)?;
            }
            Ph1dResponse::Fail(fail) => {
                store
                    .ph1d_fail_closed_commit(
                        now,
                        tenant_id.to_string(),
                        correlation_id,
                        turn_id,
                        session_id,
                        actor_user_id.clone(),
                        device_id.clone(),
                        ph1d_fail_code(fail.kind).to_string(),
                        fail.reason_code,
                        sanitize_idempotency_token(&format!(
                            "ph1d_fail:{}:{}",
                            correlation_id.0, turn_id.0
                        )),
                    )
                    .map_err(storage_error_to_string)?;
            }
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_ph1d_gold_capture_and_learning(
        &self,
        store: &mut Ph1fStore,
        now: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        actor_user_id: &UserId,
        tenant_id: Option<&str>,
        device_id: Option<&DeviceId>,
        session_id: Option<SessionId>,
        provider_calls: &[Ph1dProviderCallResponse],
        final_transcript: Option<String>,
        language_locale: Option<String>,
    ) -> Result<(), String> {
        let (Some(tenant_id), Some(device_id)) = (tenant_id, device_id) else {
            return Ok(());
        };
        for (idx, provider_call) in provider_calls.iter().enumerate() {
            if provider_call.provider_status
                == selene_kernel_contracts::ph1d::Ph1dProviderStatus::Ok
                && provider_call.validation_status
                    == selene_kernel_contracts::ph1d::Ph1dProviderValidationStatus::SchemaOk
            {
                continue;
            }
            let feedback_event_type = FeedbackEventType::SttReject;
            let Some((feedback_event_type, learn_signal_type)) =
                feedback_learn_pair_for_ph1d_capture(feedback_event_type)
            else {
                continue;
            };
            let feedback_idem = sanitize_idempotency_token(&format!(
                "ph1d_feedback_{}_{}_{}",
                correlation_id.0, turn_id.0, idx
            ));
            store
                .ph1feedback_event_commit(
                    now,
                    tenant_id.to_string(),
                    correlation_id,
                    turn_id,
                    session_id,
                    actor_user_id.clone(),
                    device_id.clone(),
                    feedback_event_type.to_string(),
                    learn_signal_type.to_string(),
                    provider_call.reason_code,
                    feedback_idem,
                )
                .map_err(storage_error_to_string)?;
            let learn_idem = sanitize_idempotency_token(&format!(
                "ph1d_learn_{}_{}_{}",
                correlation_id.0, turn_id.0, idx
            ));
            let evidence = truncate_ascii(
                final_transcript
                    .as_deref()
                    .or(language_locale.as_deref())
                    .unwrap_or("ph1d_provider_error"),
                128,
            );
            store
                .ph1feedback_learn_signal_bundle_commit(
                    now,
                    tenant_id.to_string(),
                    correlation_id,
                    turn_id,
                    session_id,
                    actor_user_id.clone(),
                    device_id.clone(),
                    feedback_event_type.to_string(),
                    learn_signal_type.to_string(),
                    provider_call.reason_code,
                    evidence,
                    truncate_ascii(
                        &format!(
                            "ph1d_provider:{}:{}",
                            provider_call.provider_id, provider_call.model_id
                        ),
                        128,
                    ),
                    provider_call.provider_latency_ms.min(2_000),
                    learn_idem,
                )
                .map_err(storage_error_to_string)?;
        }
        Ok(())
    }

    fn emit_ph1c_live_telemetry(
        &self,
        store: &mut Ph1fStore,
        now: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        ph1c: &Ph1cLiveTurnOutcomeSummary,
        tenant_id: Option<&str>,
    ) -> Result<(), String> {
        let tenant_id = tenant_id.unwrap_or("tenant_default");
        let (outcome_type, reason_code, latency_ms, decision_delta) = match &ph1c.response {
            Ph1cResponse::TranscriptOk(ok) => (
                if ph1c.low_latency_commit {
                    "PH1C_LIVE_TRANSCRIPT_OK_LOW_LATENCY"
                } else if ph1c.finalized {
                    "PH1C_LIVE_TRANSCRIPT_OK_FINAL"
                } else {
                    "PH1C_LIVE_TRANSCRIPT_OK_PARTIAL"
                },
                ReasonCodeId(0x4300_5101),
                ok.audit_meta
                    .as_ref()
                    .map(|meta| meta.total_latency_ms)
                    .unwrap_or(0),
                true,
            ),
            Ph1cResponse::TranscriptReject(reject) => (
                "PH1C_LIVE_TRANSCRIPT_REJECT",
                reject.reason_code,
                reject
                    .audit_meta
                    .as_ref()
                    .map(|meta| meta.total_latency_ms)
                    .unwrap_or(0),
                false,
            ),
        };
        let idempotency_key = sanitize_idempotency_token(&format!(
            "ph1c_live_telemetry:{}:{}:{}:{}",
            tenant_id, correlation_id.0, turn_id.0, outcome_type
        ));
        store
            .append_outcome_utilization_ledger_row(OutcomeUtilizationLedgerRowInput {
                created_at: now,
                correlation_id,
                turn_id,
                engine_id: "PH1.C".to_string(),
                outcome_type: outcome_type.to_string(),
                action_class: OsOutcomeActionClass::AuditOnly,
                consumed_by: "PH1.C.SUPERIORITY".to_string(),
                latency_cost_ms: latency_ms,
                decision_delta,
                reason_code,
                idempotency_key: Some(idempotency_key),
            })
            .map_err(storage_error_to_string)?;
        if let Err(err) = append_ph1c_live_telemetry_csv(
            now,
            correlation_id,
            turn_id,
            tenant_id,
            outcome_type,
            reason_code,
            latency_ms,
            decision_delta,
            ph1c.finalized,
            ph1c.low_latency_commit,
        ) {
            eprintln!("selene_adapter ph1c live telemetry csv append failed: {err}");
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn commit_ph1k_live_runtime_events(
        &self,
        store: &mut Ph1fStore,
        now: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        tenant_id: Option<&str>,
        device_id: &DeviceId,
        session_id: Option<SessionId>,
        bundle: &Ph1kLiveSignalBundle,
    ) -> Result<(), String> {
        let tenant_id = truncate_ascii(tenant_id.unwrap_or("tenant_default"), 64);
        let processed_stream_id = Some(bundle.processed_stream_ref.stream_id.0);
        let pre_roll_buffer_id = Some(bundle.pre_roll_buffer_ref.buffer_id.0);
        let device_health = storage_device_health_from_bundle(bundle);

        store
            .ph1k_runtime_event_commit(
                now,
                tenant_id.clone(),
                device_id.clone(),
                session_id,
                Ph1kRuntimeEventKind::StreamRefs,
                processed_stream_id,
                None,
                pre_roll_buffer_id,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                sanitize_idempotency_token(&format!(
                    "ph1k_runtime:{}:{}:stream_refs",
                    correlation_id.0, turn_id.0
                )),
            )
            .map_err(storage_error_to_string)?;

        for (idx, vad) in bundle.vad_events.iter().enumerate() {
            store
                .ph1k_runtime_event_commit(
                    now,
                    tenant_id.clone(),
                    device_id.clone(),
                    session_id,
                    Ph1kRuntimeEventKind::VadEvent,
                    Some(vad.stream_id.0),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    sanitize_idempotency_token(&format!(
                        "ph1k_runtime:{}:{}:vad:{}",
                        correlation_id.0, turn_id.0, idx
                    )),
                )
                .map_err(storage_error_to_string)?;
        }

        store
            .ph1k_runtime_event_commit(
                now,
                tenant_id.clone(),
                device_id.clone(),
                session_id,
                Ph1kRuntimeEventKind::DeviceState,
                None,
                None,
                None,
                Some(bundle.device_state.selected_mic.as_str().to_string()),
                Some(bundle.device_state.selected_speaker.as_str().to_string()),
                Some(device_health),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                sanitize_idempotency_token(&format!(
                    "ph1k_runtime:{}:{}:device_state",
                    correlation_id.0, turn_id.0
                )),
            )
            .map_err(storage_error_to_string)?;

        store
            .ph1k_runtime_event_commit(
                now,
                tenant_id.clone(),
                device_id.clone(),
                session_id,
                Ph1kRuntimeEventKind::TimingStats,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(bundle.timing_stats.jitter_ms),
                Some(bundle.timing_stats.drift_ppm),
                Some(bundle.timing_stats.buffer_depth_ms),
                Some(bundle.timing_stats.underruns),
                Some(bundle.timing_stats.overruns),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                sanitize_idempotency_token(&format!(
                    "ph1k_runtime:{}:{}:timing_stats",
                    correlation_id.0, turn_id.0
                )),
            )
            .map_err(storage_error_to_string)?;

        store
            .ph1k_runtime_event_commit(
                now,
                tenant_id.clone(),
                device_id.clone(),
                session_id,
                Ph1kRuntimeEventKind::DegradationFlags,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(bundle.interrupt_input.capture_degraded),
                Some(bundle.interrupt_input.aec_unstable),
                Some(bundle.interrupt_input.device_changed),
                Some(bundle.interrupt_input.stream_gap_detected),
                sanitize_idempotency_token(&format!(
                    "ph1k_runtime:{}:{}:degradation",
                    correlation_id.0, turn_id.0
                )),
            )
            .map_err(storage_error_to_string)?;

        store
            .ph1k_runtime_event_commit(
                now,
                tenant_id.clone(),
                device_id.clone(),
                session_id,
                Ph1kRuntimeEventKind::TtsPlaybackActive,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(bundle.tts_playback.active),
                None,
                None,
                None,
                None,
                sanitize_idempotency_token(&format!(
                    "ph1k_runtime:{}:{}:tts_active",
                    correlation_id.0, turn_id.0
                )),
            )
            .map_err(storage_error_to_string)?;

        if let Some(candidate) = bundle.interrupt_decision.candidate.as_ref() {
            let interrupt_extended = Ph1kInterruptCandidateExtendedFields {
                trigger_phrase_id: candidate.trigger_phrase_id.0,
                trigger_locale: candidate.trigger_locale.as_str().to_string(),
                candidate_confidence_band: candidate.candidate_confidence_band,
                vad_decision_confidence_band: bundle.ph1c_handoff.vad_confidence_band,
                risk_context_class: candidate.risk_context_class,
                gate_confidences: candidate.gate_confidences,
                degradation_context: candidate.degradation_context,
                quality_metrics: bundle.interrupt_input.adaptive_policy_input.quality_metrics,
                timing_markers: candidate.timing_markers,
                speech_window_metrics: candidate.speech_window_metrics,
                subject_relation_confidence_bundle: candidate.subject_relation_confidence_bundle,
                interrupt_policy_profile_id: bundle
                    .interrupt_input
                    .lexicon_policy_binding
                    .policy_profile_id
                    .as_str()
                    .to_string(),
                interrupt_tenant_profile_id: bundle
                    .interrupt_input
                    .lexicon_policy_binding
                    .tenant_profile_id
                    .as_str()
                    .to_string(),
                interrupt_locale_tag: bundle
                    .interrupt_input
                    .lexicon_policy_binding
                    .locale_tag
                    .as_str()
                    .to_string(),
                adaptive_device_route: bundle.interrupt_input.adaptive_policy_input.device_route,
                adaptive_noise_class: interrupt_noise_class_label(
                    bundle.interrupt_decision.adaptive_noise_class,
                )
                .to_string(),
                adaptive_capture_to_handoff_latency_ms: bundle
                    .interrupt_input
                    .adaptive_policy_input
                    .capture_to_handoff_latency_ms,
                adaptive_timing_jitter_ms: bundle
                    .interrupt_input
                    .adaptive_policy_input
                    .timing_stats
                    .jitter_ms,
                adaptive_timing_drift_ppm: bundle
                    .interrupt_input
                    .adaptive_policy_input
                    .timing_stats
                    .drift_ppm,
                adaptive_device_reliability_score: bundle
                    .interrupt_input
                    .adaptive_policy_input
                    .device_reliability
                    .reliability_score
                    .0,
            };

            store
                .ph1k_runtime_event_commit_extended(
                    now,
                    tenant_id,
                    device_id.clone(),
                    session_id,
                    Ph1kRuntimeEventKind::InterruptCandidate,
                    processed_stream_id,
                    None,
                    pre_roll_buffer_id,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(candidate.phrase_id.0),
                    Some(candidate.phrase_text.clone()),
                    Some(candidate.reason_code),
                    Some(interrupt_extended),
                    None,
                    None,
                    None,
                    None,
                    None,
                    sanitize_idempotency_token(&format!(
                        "ph1k_runtime:{}:{}:interrupt_candidate:{}",
                        correlation_id.0, turn_id.0, candidate.phrase_id.0
                    )),
                )
                .map_err(storage_error_to_string)?;
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_ph1k_feedback_capture(
        &self,
        store: &mut Ph1fStore,
        now: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        tenant_id: Option<&str>,
        actor_user_id: &UserId,
        device_id: &DeviceId,
        session_id: Option<SessionId>,
        bundle: &Ph1kLiveSignalBundle,
    ) -> Result<(), String> {
        let feedback_kind = if let Some(candidate) = bundle.interrupt_decision.candidate.as_ref() {
            if !bundle.tts_playback.active {
                Some(InterruptFeedbackSignalKind::FalseLexicalTrigger)
            } else if matches!(
                candidate.candidate_confidence_band,
                selene_kernel_contracts::ph1k::InterruptCandidateConfidenceBand::Low
            ) {
                Some(InterruptFeedbackSignalKind::WrongConfidenceBand)
            } else {
                None
            }
        } else if bundle.tts_playback.active && bundle.interrupt_input.detection.is_some() {
            Some(InterruptFeedbackSignalKind::MissedLexicalTrigger)
        } else {
            None
        };

        let Some(feedback_kind) = feedback_kind else {
            return Ok(());
        };

        let candidate_confidence_band = bundle
            .interrupt_decision
            .candidate
            .as_ref()
            .map(|candidate| candidate.candidate_confidence_band);
        let _signal = build_interrupt_feedback_signal(feedback_kind, candidate_confidence_band);
        let issue_kind = match feedback_kind {
            InterruptFeedbackSignalKind::FalseLexicalTrigger => {
                Ph1kFeedbackIssueKind::FalseInterrupt
            }
            InterruptFeedbackSignalKind::MissedLexicalTrigger => {
                Ph1kFeedbackIssueKind::MissedInterrupt
            }
            InterruptFeedbackSignalKind::WrongConfidenceBand => {
                Ph1kFeedbackIssueKind::WrongDegradationClassification
            }
        };
        let capture_input = Ph1kFeedbackCaptureInput {
            issue_kind,
            candidate_confidence_band,
            risk_context_class: bundle
                .interrupt_decision
                .candidate
                .as_ref()
                .map(|candidate| candidate.risk_context_class),
            adaptive_device_route: Some(bundle.interrupt_input.adaptive_policy_input.device_route),
            adaptive_noise_class: Some(
                interrupt_noise_class_label(bundle.interrupt_decision.adaptive_noise_class)
                    .to_string(),
            ),
            capture_degraded: Some(bundle.interrupt_input.capture_degraded),
            aec_unstable: Some(bundle.interrupt_input.aec_unstable),
            device_changed: Some(bundle.interrupt_input.device_changed),
            stream_gap_detected: Some(bundle.interrupt_input.stream_gap_detected),
            failover_from_device: None,
            failover_to_device: None,
        };
        let tenant_id = truncate_ascii(tenant_id.unwrap_or("tenant_default"), 64);
        store
            .ph1k_feedback_capture_commit(
                now,
                tenant_id,
                correlation_id,
                turn_id,
                session_id,
                actor_user_id.clone(),
                device_id.clone(),
                capture_input,
                sanitize_idempotency_token(&format!(
                    "ph1k_feedback:{}:{}:{}",
                    correlation_id.0,
                    turn_id.0,
                    interrupt_feedback_kind_label(feedback_kind)
                )),
            )
            .map_err(storage_error_to_string)?;
        Ok(())
    }

    fn run_ph1vision_os_orchestration_step(
        &self,
        request: &VoiceTurnAdapterRequest,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        tenant_scope: Option<&str>,
        base_transcript_text: Option<&str>,
    ) -> Result<(), String> {
        let Some(vision_turn_input) =
            build_vision_turn_input_from_adapter_request(request, correlation_id, turn_id)?
        else {
            return Ok(());
        };

        let vision_wiring = Ph1VisionWiring::new(
            Ph1VisionWiringConfig::mvp_v1(true),
            AdapterVisionEngineRuntime::new(),
        )
        .map_err(|err| format!("ph1vision wiring bootstrap failed: {err:?}"))?;
        let vision_outcome = vision_wiring
            .run_turn(&vision_turn_input)
            .map_err(|err| format!("ph1vision run_turn failed: {err:?}"))?;
        let analyzer_bundle = match vision_outcome {
            VisionWiringOutcome::NotInvokedOptOut => {
                return Err("PH1.VISION not invoked: opt-in disabled".to_string())
            }
            VisionWiringOutcome::NotInvokedNoVisualInput => {
                return Err("PH1.VISION not invoked: no visual input available".to_string())
            }
            VisionWiringOutcome::Refused(refuse) => {
                return Err(format!(
                    "ph1vision_refuse reason_code={} message={}",
                    refuse.reason_code.0, refuse.message
                ))
            }
            VisionWiringOutcome::Forwarded { bundle, .. } => {
                OsOcrAnalyzerForwardBundle::Vision(bundle)
            }
        };

        let live_adapter = self.ph1d_live_adapter.clone().ok_or_else(|| {
            "PH1.D live provider adapter unavailable for PH1.VISION OCR path".to_string()
        })?;
        let mut ocr_route_config = Ph1OsOcrRouteConfig::openai_default();
        if let Some(tenant_scope) = tenant_scope {
            ocr_route_config.tenant_id = truncate_ascii(tenant_scope, 64);
        }
        let ocr_route_wiring = Ph1OsOcrRouteWiring::new(ocr_route_config, live_adapter)
            .map_err(|err| format!("ocr route wiring bootstrap failed: {err:?}"))?;
        let ocr_route_outcome = ocr_route_wiring
            .run_handoff(&analyzer_bundle)
            .map_err(|err| format!("ocr route handoff failed: {err:?}"))?;
        let ocr_bundle = match ocr_route_outcome {
            OsOcrRouteOutcome::NotInvokedDisabled => {
                return Err("PH1.OS OCR route disabled for PH1.VISION handoff".to_string())
            }
            OsOcrRouteOutcome::Refused(refuse) => {
                return Err(format!(
                    "ph1os_ocr_refuse reason_code={} message={}",
                    refuse.reason_code.0, refuse.message
                ))
            }
            OsOcrRouteOutcome::Forwarded(bundle) => bundle,
        };

        let context_wiring = Ph1ContextWiring::new(
            Ph1ContextWiringConfig::mvp_v1(true),
            AdapterContextEngineRuntime::new(),
        )
        .map_err(|err| format!("ph1context wiring bootstrap failed: {err:?}"))?;
        let nlp_wiring = Ph1nWiring::new(
            Ph1nWiringConfig::mvp_v1(true),
            AdapterNlpEngineRuntime::new(),
        )
        .map_err(|err| format!("ph1n wiring bootstrap failed: {err:?}"))?;
        let bridge = Ph1OsOcrContextNlpWiring::new(
            Ph1OsOcrContextNlpConfig::mvp_v1(),
            context_wiring,
            nlp_wiring,
        )
        .map_err(|err| format!("ocr->context/nlp wiring bootstrap failed: {err:?}"))?;
        let base_nlp_request =
            build_base_nlp_request_for_vision_handoff(request, base_transcript_text, tenant_scope)?;
        let bridge_outcome = bridge
            .run_handoff(&ocr_bundle, &base_nlp_request)
            .map_err(|err| format!("ocr->context/nlp handoff failed: {err:?}"))?;
        match bridge_outcome {
            OsOcrContextNlpOutcome::NotInvokedDisabled => {
                Err("PH1.OS OCR->CONTEXT/NLP bridge disabled".to_string())
            }
            OsOcrContextNlpOutcome::Refused(refuse) => Err(format!(
                "ph1os_ocr_context_refuse reason_code={} message={}",
                refuse.reason_code.0, refuse.message
            )),
            OsOcrContextNlpOutcome::Forwarded(_) => Ok(()),
        }
    }

    fn run_voice_turn_internal(
        &self,
        request: VoiceTurnAdapterRequest,
        runtime_execution_envelope: Option<RuntimeExecutionEnvelope>,
        persist_on_success: bool,
        allow_identity_auto_provision: bool,
        persistence_mode: PersistenceInvocationMode,
    ) -> Result<VoiceTurnAdapterResponse, VoiceTurnIngressError> {
        let request_for_journal = request.clone();
        let response_turn_id = Some(request.turn_id);
        let mut user_text_partial =
            sanitize_transcript_text_option(request.user_text_partial.clone());
        let mut user_text_final = sanitize_transcript_text_option(request.user_text_final.clone());
        let upstream_transcript_supplied = user_text_final.is_some();
        let selene_text_partial =
            sanitize_transcript_text_option(request.selene_text_partial.clone());
        let selene_text_final = sanitize_transcript_text_option(request.selene_text_final.clone());
        let pre_session_error = |reason: String| {
            classify_voice_turn_runtime_error(&reason, response_turn_id, None, None)
        };
        let app_platform = parse_app_platform(&request.app_platform).map_err(pre_session_error)?;
        let trigger = parse_trigger(&request.trigger).map_err(pre_session_error)?;
        let actor_user_id = UserId::new(request.actor_user_id.clone())
            .map_err(|err| pre_session_error(format!("invalid actor_user_id: {err:?}")))?;
        let request_device_id = request
            .device_id
            .as_ref()
            .map(|id| {
                DeviceId::new(id.clone())
                    .map_err(|err| pre_session_error(format!("invalid device_id: {err:?}")))
            })
            .transpose()?;
        let correlation_id = CorrelationId(request.correlation_id.into());
        let turn_id = TurnId(request.turn_id);
        let device_turn_sequence = request
            .device_turn_sequence
            .unwrap_or(request.turn_id)
            .max(1);
        let platform_context = normalize_platform_runtime_context(&request, app_platform, trigger)
            .map_err(pre_session_error)?;
        let now = MonotonicTimeNs(request.now_ns.unwrap_or(1));
        let runtime_device_id = match request_device_id {
            Some(id) => id,
            None => DeviceId::new(format!(
                "adapter_auto_{}",
                stable_hash_hex_16(actor_user_id.as_str())
            ))
            .map_err(|err| {
                pre_session_error(format!("invalid generated runtime device_id: {err:?}"))
            })?,
        };
        let mut runtime_execution_envelope = match runtime_execution_envelope {
            Some(envelope) => {
                RuntimeExecutionEnvelope::v1_with_platform_context_device_turn_sequence_and_attach_outcome(
                    envelope.request_id,
                    envelope.trace_id,
                    envelope.idempotency_key,
                    actor_user_id.clone(),
                    runtime_device_id.clone(),
                    app_platform,
                    platform_context.clone(),
                    envelope.session_id,
                    turn_id,
                    envelope.device_turn_sequence.or(Some(device_turn_sequence)),
                    AdmissionState::IngressValidated,
                    envelope.session_attach_outcome,
                )
                .map_err(|err| {
                    pre_session_error(format!("invalid runtime_execution_envelope: {err:?}"))
                })?
            }
            None => fallback_runtime_execution_envelope_for_voice_turn_request_with_identities(
                FallbackRuntimeExecutionEnvelopeParams {
                    correlation_id,
                    turn_id,
                    app_platform,
                    actor_user_id: &actor_user_id,
                    device_id: &runtime_device_id,
                    device_turn_sequence: Some(device_turn_sequence),
                    session_attach_outcome: None,
                    platform_context: Some(platform_context.clone()),
                },
            )
            .map_err(|err| {
                pre_session_error(format!("invalid runtime_execution_envelope: {err:?}"))
            })?,
        };
        if persistence_mode != PersistenceInvocationMode::LegacyJournalReplay {
            if let Some(reused_result) = self
                .try_reuse_persisted_authoritative_result(
                    now,
                    &actor_user_id,
                    &runtime_execution_envelope.idempotency_key,
                )
                .map_err(pre_session_error)?
            {
                let reused_result = match reused_result {
                    Ok(response) => {
                        self.record_reused_response_transcript_finals(
                            ReusedResponseTranscriptUpdate {
                                now,
                                correlation_id,
                                turn_id,
                                actor_user_id: &actor_user_id,
                                device_id: &runtime_device_id,
                                response: &response,
                                user_text_final: user_text_final.as_deref(),
                                selene_text_final: selene_text_final.as_deref(),
                            },
                        )
                        .map_err(pre_session_error)?;
                        Ok(response)
                    }
                    Err(error) => Err(error),
                };
                return reused_result;
            }
        }
        let prepared_persistence = self
            .prepare_persistence_operation(
                &request,
                &runtime_execution_envelope,
                now,
                persistence_mode,
            )
            .map_err(pre_session_error)?;
        if let Some(prepared) = prepared_persistence.as_ref() {
            runtime_execution_envelope = runtime_execution_envelope
                .with_persistence_state(Some(prepared.persistence_state.clone()))
                .map_err(|err| {
                    pre_session_error(format!(
                        "invalid runtime_execution_envelope persistence_state: {err:?}"
                    ))
                })?;
        }
        if let Some(response) = cached_retry_response_for_actor(
            &self.session_retry_cache,
            &actor_user_id,
            &runtime_execution_envelope.idempotency_key,
        )
        .map_err(pre_session_error)?
        {
            let result = match self.record_reused_response_transcript_finals(
                ReusedResponseTranscriptUpdate {
                    now,
                    correlation_id,
                    turn_id,
                    actor_user_id: &actor_user_id,
                    device_id: &runtime_device_id,
                    response: &response,
                    user_text_final: user_text_final.as_deref(),
                    selene_text_final: selene_text_final.as_deref(),
                },
            ) {
                Ok(()) => Ok(response),
                Err(err) => Err(pre_session_error(err)),
            };
            if let Some(prepared) = prepared_persistence.as_ref() {
                self.finalize_persistence_operation(
                    prepared,
                    now,
                    &runtime_device_id,
                    &runtime_execution_envelope.idempotency_key,
                    &result,
                )
                .map_err(pre_session_error)?;
            }
            return result;
        }

        let execution_result = (|| {
            let mut store = self
                .store
                .lock()
                .map_err(|_| pre_session_error("adapter store lock poisoned".to_string()))?;
            ensure_actor_identity_and_device(
                &mut store,
                &actor_user_id,
                Some(&runtime_device_id),
                app_platform,
                now,
                allow_identity_auto_provision,
            )
            .map_err(pre_session_error)?;
            let tenant_id_for_ph1c = resolve_tenant_scope(
                request.tenant_id.clone(),
                &actor_user_id,
                Some(&runtime_device_id),
            );
            let ph1k_bundle = build_ph1k_live_signal_bundle(
                &store,
                &request,
                now,
                tenant_id_for_ph1c.as_deref(),
                Some(&runtime_device_id),
            )
            .map_err(pre_session_error)?;
            let wake_evaluation = evaluate_wake_for_turn(
                &store,
                now,
                &actor_user_id,
                &runtime_device_id,
                app_platform,
                trigger,
                &ph1k_bundle,
            )
            .map_err(pre_session_error)?;
            if let Some(wake_eval) = wake_evaluation.as_ref() {
                if !wake_eval.decision.accepted {
                    let session_id_for_reject =
                        latest_canonical_session_for_actor(&store, &actor_user_id)
                            .map_err(pre_session_error)?
                            .map(|row| row.session_id);
                    commit_wake_runtime_event(
                        &mut store,
                        now,
                        correlation_id,
                        turn_id,
                        &actor_user_id,
                        &runtime_device_id,
                        session_id_for_reject,
                        &ph1k_bundle,
                        wake_eval,
                    )
                    .map_err(pre_session_error)?;
                    commit_wake_learn_signal(
                        &mut store,
                        now,
                        correlation_id,
                        turn_id,
                        &runtime_device_id,
                        session_id_for_reject,
                        trigger,
                        &ph1k_bundle,
                        wake_eval,
                    )
                    .map_err(pre_session_error)?;
                    return Err(voice_turn_ingress_error(
                        FailureClass::PolicyViolation,
                        format!("WAKE_REJECTED_{}", wake_eval.decision.reason_code.0),
                        Some(format!(
                            "wake_rejected reason_code={}",
                            wake_eval.decision.reason_code.0
                        )),
                        session_id_for_reject.map(session_id_to_string),
                        response_turn_id,
                        None,
                    ));
                }
            }
            let session_turn_state = match resolve_session_turn_state(
                &mut store,
                now,
                correlation_id,
                turn_id,
                ResolveSessionTurnStateInput {
                    actor_user_id: &actor_user_id,
                    device_id: &runtime_device_id,
                    trigger,
                    ph1k: &ph1k_bundle,
                    wake_event: wake_evaluation.as_ref().map(|wake| wake.decision.clone()),
                    idempotency_key: &runtime_execution_envelope.idempotency_key,
                    device_turn_sequence,
                    runtime_node_id: &self.runtime_node_id,
                    session_lease_ttl_ms: self.session_lease_ttl_ms,
                    session_retry_cache: &self.session_retry_cache,
                },
            )
            .map_err(pre_session_error)?
            {
                AdapterSessionResolution::Proceed(state) => state,
                AdapterSessionResolution::Retry(response) => return Ok(response),
            };
            let response_session_id = session_turn_state
                .session_snapshot
                .session_id
                .map(session_id_to_string);
            let response_session_state = session_turn_state.session_snapshot.session_state;
            let post_session_error = |reason: String| {
                classify_voice_turn_runtime_error(
                    &reason,
                    response_turn_id,
                    response_session_id.clone(),
                    Some(response_session_state),
                )
            };
            runtime_execution_envelope = runtime_execution_envelope
                .with_session_device_turn_and_attach_outcome(
                    session_turn_state.session_snapshot.session_id,
                    AdmissionState::SessionResolved,
                    Some(session_turn_state.device_turn_sequence),
                    Some(session_turn_state.session_attach_outcome),
                )
                .map_err(|err| {
                    post_session_error(format!(
                        "invalid runtime_execution_envelope after session resolve: {err:?}"
                    ))
                })?;
            if let Some(wake_eval) = wake_evaluation.as_ref() {
                commit_wake_runtime_event(
                    &mut store,
                    now,
                    correlation_id,
                    turn_id,
                    &actor_user_id,
                    &runtime_device_id,
                    session_turn_state.session_id_for_commits,
                    &ph1k_bundle,
                    wake_eval,
                )
                .map_err(post_session_error)?;
                commit_wake_learn_signal(
                    &mut store,
                    now,
                    correlation_id,
                    turn_id,
                    &runtime_device_id,
                    session_turn_state.session_id_for_commits,
                    trigger,
                    &ph1k_bundle,
                    wake_eval,
                )
                .map_err(post_session_error)?;
            }
            let voice_id_request = build_voice_id_request_from_ph1k_bundle(
                now,
                &ph1k_bundle,
                session_turn_state.session_snapshot,
                session_turn_state.wake_event.clone(),
            )
            .map_err(|err| post_session_error(format!("voice request build failed: {err:?}")))?;
            let ph1c_live_outcome = if upstream_transcript_supplied {
                None
            } else {
                self.run_ph1c_live_turn(
                    correlation_id,
                    turn_id,
                    &actor_user_id,
                    tenant_id_for_ph1c.as_deref(),
                    session_turn_state.session_snapshot.session_state,
                    &ph1k_bundle,
                )
            };
            if let Some(ph1c) = ph1c_live_outcome.as_ref() {
                if user_text_partial.is_none() {
                    user_text_partial = ph1c.partial_text.clone();
                }
                if user_text_final.is_none() {
                    user_text_final = ph1c.final_text.clone();
                }
                self.commit_ph1c_live_outcome(
                    &mut store,
                    now,
                    correlation_id,
                    turn_id,
                    &actor_user_id,
                    tenant_id_for_ph1c.as_deref(),
                    Some(&runtime_device_id),
                    session_turn_state.session_id_for_commits,
                    ph1c,
                )
                .map_err(post_session_error)?;
            }
            self.run_ph1vision_os_orchestration_step(
                &request,
                correlation_id,
                turn_id,
                tenant_id_for_ph1c.as_deref(),
                user_text_final.as_deref(),
            )
            .map_err(post_session_error)?;

            self.commit_ph1k_live_runtime_events(
                &mut store,
                now,
                correlation_id,
                turn_id,
                tenant_id_for_ph1c.as_deref(),
                &runtime_device_id,
                session_turn_state.session_id_for_commits,
                &ph1k_bundle,
            )
            .map_err(post_session_error)?;
            self.emit_ph1k_feedback_capture(
                &mut store,
                now,
                correlation_id,
                turn_id,
                tenant_id_for_ph1c.as_deref(),
                &actor_user_id,
                &runtime_device_id,
                session_turn_state.session_id_for_commits,
                &ph1k_bundle,
            )
            .map_err(post_session_error)?;
            if let Err(err) = append_ph1k_live_eval_snapshot_csv(
                &store,
                now,
                correlation_id,
                turn_id,
                tenant_id_for_ph1c.as_deref().unwrap_or("tenant_default"),
                &ph1k_bundle,
            ) {
                eprintln!("selene_adapter ph1k live eval csv append failed: {err}");
            }

            let ingress_request = AppVoiceIngressRequest::v1_with_runtime_execution_envelope(
                correlation_id,
                turn_id,
                runtime_execution_envelope.clone(),
                app_platform,
                trigger,
                voice_id_request,
                actor_user_id.clone(),
                tenant_id_for_ph1c.clone(),
                Some(runtime_device_id.clone()),
                Vec::new(),
                empty_observation(),
            )
            .map_err(|err| post_session_error(format!("invalid ingress request: {err:?}")))?;
            let nlp_output = build_nlp_output_for_voice_turn(
                &request,
                user_text_final.as_deref(),
                tenant_id_for_ph1c.as_deref(),
            )
            .map_err(post_session_error)?;
            let thread_key = resolve_adapter_thread_key(request.thread_key.as_deref());
            let mut base_thread_state = load_ph1x_thread_state(&store, &actor_user_id, &thread_key);
            if request.project_id.is_some() || request.pinned_context_refs.is_some() {
                let project_id = resolve_adapter_project_id(request.project_id.as_deref());
                let pinned_context_refs =
                    resolve_adapter_pinned_context_refs(request.pinned_context_refs.as_deref());
                base_thread_state = base_thread_state
                    .with_project_context(project_id, pinned_context_refs)
                    .map_err(|err| {
                        post_session_error(format!("invalid thread project context: {err:?}"))
                    })?;
            }
            if let Some(flags) = request.thread_policy_flags.as_ref() {
                let kernel_flags = ThreadPolicyFlags::v1(
                    flags.privacy_mode,
                    flags.do_not_disturb,
                    flags.strict_safety,
                )
                .map_err(|err| {
                    post_session_error(format!("invalid thread policy flags: {err:?}"))
                })?;
                base_thread_state = base_thread_state
                    .with_thread_policy_flags(Some(kernel_flags))
                    .map_err(|err| {
                        post_session_error(format!("invalid thread policy flags: {err:?}"))
                    })?;
            }
            let confirm_answer =
                infer_confirm_answer_from_user_text(&base_thread_state, user_text_final.as_deref());
            let locale = request
                .audio_capture_ref
                .as_ref()
                .and_then(|capture| capture.locale_tag.as_deref())
                .map(|raw| truncate_ascii(raw.trim(), 16))
                .filter(|value| !value.is_empty());
            let x_build = AppVoicePh1xBuildInput {
                now,
                thread_key: Some(thread_key.clone()),
                thread_state: base_thread_state,
                session_state: session_turn_state.session_snapshot.session_state,
                policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
                memory_candidates: Vec::new(),
                confirm_answer,
                nlp_output: Some(nlp_output),
                tool_response: None,
                interruption: None,
                locale,
                last_failure_reason_code: None,
            };
            let execution_outcome = self
                .ingress
                .run_voice_turn_end_to_end(&mut store, ingress_request, x_build)
                .map_err(|err| post_session_error(storage_error_to_string(err)))?;
            if let Some(ph1x_response) = execution_outcome.ph1x_response.as_ref() {
                persist_ph1x_thread_state(
                    &mut store,
                    now,
                    PersistPh1xThreadStateInput {
                        actor_user_id: &actor_user_id,
                        thread_key: &thread_key,
                        thread_state: ph1x_response.thread_state.clone(),
                        reason_code: ph1x_response.reason_code,
                        correlation_id,
                        turn_id,
                    },
                )
                .map_err(post_session_error)?;
            }
            self.commit_ph1d_runtime_outcome(
                &mut store,
                now,
                correlation_id,
                turn_id,
                &actor_user_id,
                tenant_id_for_ph1c.as_deref(),
                Some(&runtime_device_id),
                session_turn_state.session_id_for_commits,
                session_turn_state.session_snapshot.session_state,
                user_text_final.as_deref(),
                &execution_outcome.voice_outcome,
            )
            .map_err(post_session_error)?;
            if let Err(err) = self.emit_read_only_lane_incidents_and_maybe_run_builder(
                &mut store,
                now,
                correlation_id,
                turn_id,
                &actor_user_id,
                tenant_id_for_ph1c.as_deref(),
                &runtime_device_id,
                user_text_final.as_deref(),
                &execution_outcome,
            ) {
                eprintln!("selene_adapter read-only incident emission failed: {err}");
            }
            self.record_transcript_updates(
                &mut store,
                now,
                correlation_id,
                turn_id,
                &actor_user_id,
                Some(&runtime_device_id),
                session_turn_state.session_id_for_commits,
                user_text_partial,
                user_text_final,
                selene_text_partial,
                selene_text_final,
            )
            .map_err(post_session_error)?;
            if let Some(ph1c) = ph1c_live_outcome.as_ref() {
                self.emit_ph1c_gold_capture_and_learning(
                    &mut store,
                    now,
                    correlation_id,
                    turn_id,
                    &actor_user_id,
                    tenant_id_for_ph1c.as_deref(),
                    Some(&runtime_device_id),
                    session_turn_state.session_id_for_commits,
                    ph1c,
                )
                .map_err(post_session_error)?;
                self.emit_ph1d_gold_capture_and_learning(
                    &mut store,
                    now,
                    correlation_id,
                    turn_id,
                    &actor_user_id,
                    tenant_id_for_ph1c.as_deref(),
                    Some(&runtime_device_id),
                    session_turn_state.session_id_for_commits,
                    &ph1c.provider_call_trace,
                    ph1c.final_text.clone(),
                    ph1c_language_locale(&ph1c.response),
                )
                .map_err(post_session_error)?;
                self.emit_ph1c_live_telemetry(
                    &mut store,
                    now,
                    correlation_id,
                    turn_id,
                    ph1c,
                    tenant_id_for_ph1c.as_deref(),
                )
                .map_err(post_session_error)?;
            }
            finalize_session_turn_record(
                &mut store,
                now,
                correlation_id,
                turn_id,
                &runtime_device_id,
                session_turn_state.session_id_for_commits,
                session_turn_state.device_turn_sequence,
                &runtime_execution_envelope.idempotency_key,
                &self.runtime_node_id,
                self.session_lease_ttl_ms,
            )
            .map_err(post_session_error)?;
            let response = execution_outcome_to_adapter_response(execution_outcome);
            cache_authoritative_turn_response(
                &self.session_retry_cache,
                &actor_user_id,
                &runtime_execution_envelope.idempotency_key,
                &response,
            )
            .map_err(post_session_error)?;
            if persist_on_success {
                self.append_legacy_journal_entry(request_for_journal.clone())
                    .map_err(post_session_error)?;
            }
            Ok(response)
        })();

        if let Some(prepared) = prepared_persistence.as_ref() {
            self.finalize_persistence_operation(
                prepared,
                now,
                &runtime_device_id,
                &runtime_execution_envelope.idempotency_key,
                &execution_result,
            )
            .map_err(pre_session_error)?;
        }

        execution_result
    }

    fn try_reuse_persisted_authoritative_result(
        &self,
        now: MonotonicTimeNs,
        actor_user_id: &UserId,
        idempotency_key: &str,
    ) -> Result<Option<Result<VoiceTurnAdapterResponse, VoiceTurnIngressError>>, String> {
        let Some(persistence) = self.persistence.as_ref() else {
            return Ok(None);
        };
        let lookup_key = actor_idempotency_lookup_key(actor_user_id.as_str(), idempotency_key);
        let operation_id = derived_operation_id(actor_user_id.as_str(), idempotency_key);
        let mut guard = persistence
            .state
            .lock()
            .map_err(|_| "adapter persistence state lock poisoned".to_string())?;
        validate_persistence_state_integrity(&guard).map_err(|err| {
            format!(
                "persistence state integrity check failed before authoritative outcome reuse: {err}"
            )
        })?;
        let Some(outcome) = guard.authoritative_outcomes.get(&lookup_key).cloned() else {
            return Ok(None);
        };
        if let Some(record) = guard.outbox_records.get_mut(&operation_id) {
            record.session_id = outcome.session_id.clone();
            record.turn_id = outcome.turn_id;
            record.acknowledgement_state =
                PersistenceAcknowledgementState::AuthoritativelyAcknowledged;
            record.cleared_from_active_outbox_at_ns = Some(now.0);
            record.last_reconciliation_decision =
                Some(ReconciliationDecision::ReusePriorAuthoritativeOutcome);
            record.last_conflict_severity = Some(PersistenceConflictSeverity::Info);
        }
        let recovery_mode = guard.recovery_mode;
        append_persistence_operation_journal_locked(
            &mut guard,
            now,
            AdapterOperationJournalEventKind::DedupeReused,
            &operation_id,
            actor_user_id.as_str(),
            idempotency_key,
            outcome.session_id.clone(),
            outcome.turn_id,
            &outcome.device_id,
            outcome.device_turn_sequence,
            PersistenceAcknowledgementState::AuthoritativelyAcknowledged,
            recovery_mode,
            Some(ReconciliationDecision::ReusePriorAuthoritativeOutcome),
            Some(PersistenceConflictSeverity::Info),
            None,
            &self.runtime_node_id,
        );
        let cross_node = outcome.runtime_node_id != self.runtime_node_id;
        append_persistence_audit_locked(
            &mut guard,
            now,
            AdapterPersistenceAuditDecision::DedupeOutcome,
            Some(operation_id.clone()),
            Some(actor_user_id.as_str().to_string()),
            Some(idempotency_key.to_string()),
            outcome.session_id.clone(),
            outcome.turn_id,
            Some(ReconciliationDecision::ReusePriorAuthoritativeOutcome),
            Some(PersistenceConflictSeverity::Info),
            &self.runtime_node_id,
            Some(if cross_node {
                format!(
                    "cross-node dedupe reused authoritative outcome from node {}",
                    outcome.runtime_node_id
                )
            } else {
                "dedupe reused authoritative outcome on same node".to_string()
            }),
        );
        let observed_policy_version = outcome.governance_policy_version.clone();
        let next_recovery_mode = recompute_persistence_recovery_mode_locked(&guard);
        set_persistence_recovery_mode_locked(
            &mut guard,
            now,
            next_recovery_mode,
            &self.runtime_node_id,
            Some("dedupe reuse completed".to_string()),
        );
        self.save_persistence_state_to_disk_locked(&guard)?;
        drop(guard);
        if cross_node {
            let _ = self.ingress.observe_runtime_governance_node_policy_version(
                &outcome.runtime_node_id,
                observed_policy_version.as_deref(),
            );
        }
        Ok(Some(match outcome.result {
            AdapterPersistedAuthoritativeResult::Success(mut response) => {
                response.session_attach_outcome = Some(SessionAttachOutcome::RetryReusedResult);
                Ok(response)
            }
            AdapterPersistedAuthoritativeResult::Failure(error) => Err(error),
        }))
    }

    fn prepare_persistence_operation(
        &self,
        request: &VoiceTurnAdapterRequest,
        runtime_execution_envelope: &RuntimeExecutionEnvelope,
        now: MonotonicTimeNs,
        persistence_mode: PersistenceInvocationMode,
    ) -> Result<Option<PreparedPersistenceOperation>, String> {
        let Some(persistence) = self.persistence.as_ref() else {
            return Ok(None);
        };
        if persistence_mode == PersistenceInvocationMode::LegacyJournalReplay {
            return Ok(None);
        }
        let operation_id = derived_operation_id(
            request.actor_user_id.as_str(),
            &runtime_execution_envelope.idempotency_key,
        );
        let mut guard = persistence
            .state
            .lock()
            .map_err(|_| "adapter persistence state lock poisoned".to_string())?;
        validate_persistence_state_integrity(&guard).map_err(|err| {
            format!("persistence state integrity check failed before prepare: {err}")
        })?;
        let mut reconciliation_decision = None;
        let mut conflict_severity = None;
        match persistence_mode {
            PersistenceInvocationMode::Standard => {
                if let Some(existing_snapshot) = guard.outbox_records.get(&operation_id).cloned() {
                    validate_outbox_record(&existing_snapshot)?;
                    let recovery_mode = guard.recovery_mode;
                    if !outbox_record_request_matches(
                        &existing_snapshot,
                        request,
                        runtime_execution_envelope,
                    ) {
                        {
                            let existing = guard
                                .outbox_records
                                .get_mut(&operation_id)
                                .expect("existing outbox row must still exist");
                            existing.acknowledgement_state =
                                PersistenceAcknowledgementState::QuarantinedLocalState;
                            existing.cleared_from_active_outbox_at_ns = Some(now.0);
                            existing.last_reconciliation_decision =
                                Some(ReconciliationDecision::QuarantineLocalState);
                            existing.last_conflict_severity =
                                Some(PersistenceConflictSeverity::QuarantineRequired);
                        }
                        append_persistence_operation_journal_locked(
                            &mut guard,
                            now,
                            AdapterOperationJournalEventKind::Quarantined,
                            &operation_id,
                            &existing_snapshot.actor_user_id,
                            &existing_snapshot.idempotency_key,
                            existing_snapshot.session_id.clone(),
                            existing_snapshot.turn_id,
                            &existing_snapshot.device_id,
                            existing_snapshot.device_turn_sequence,
                            PersistenceAcknowledgementState::QuarantinedLocalState,
                            recovery_mode,
                            Some(ReconciliationDecision::QuarantineLocalState),
                            Some(PersistenceConflictSeverity::QuarantineRequired),
                            Some("outbox_request_mismatch".to_string()),
                            &self.runtime_node_id,
                        );
                        append_persistence_audit_locked(
                            &mut guard,
                            now,
                            AdapterPersistenceAuditDecision::QuarantineAction,
                            Some(operation_id.clone()),
                            Some(existing_snapshot.actor_user_id.clone()),
                            Some(existing_snapshot.idempotency_key.clone()),
                            existing_snapshot.session_id.clone(),
                            existing_snapshot.turn_id,
                            Some(ReconciliationDecision::QuarantineLocalState),
                            Some(PersistenceConflictSeverity::QuarantineRequired),
                            &self.runtime_node_id,
                            Some("outbox request mismatch quarantined local state".to_string()),
                        );
                        set_persistence_recovery_mode_locked(
                            &mut guard,
                            now,
                            PersistenceRecoveryMode::QuarantinedLocalState,
                            &self.runtime_node_id,
                            Some("outbox request mismatch".to_string()),
                        );
                        self.save_persistence_state_to_disk_locked(&guard)?;
                        let decision = self.ingress.govern_persistence_signal(
                            Some(runtime_execution_envelope),
                            GovernanceProtectedActionClass::PersistenceReplay,
                            "persistence_quarantine_required outbox_request_mismatch",
                            Some("outbox request mismatch quarantined local state".to_string()),
                        );
                        return Err(governance_runtime_reason(&decision));
                    }
                    append_persistence_operation_journal_locked(
                        &mut guard,
                        now,
                        AdapterOperationJournalEventKind::SubmissionAttempted,
                        &operation_id,
                        &existing_snapshot.actor_user_id,
                        &existing_snapshot.idempotency_key,
                        existing_snapshot.session_id.clone(),
                        existing_snapshot.turn_id,
                        &existing_snapshot.device_id,
                        existing_snapshot.device_turn_sequence,
                        existing_snapshot.acknowledgement_state,
                        recovery_mode,
                        None,
                        None,
                        Some("resubmitted_pending_operation".to_string()),
                        &self.runtime_node_id,
                    );
                } else {
                    let record = AdapterOutboxRecord {
                        operation_id: operation_id.clone(),
                        request_id: runtime_execution_envelope.request_id.clone(),
                        trace_id: runtime_execution_envelope.trace_id.clone(),
                        actor_user_id: request.actor_user_id.clone(),
                        idempotency_key: runtime_execution_envelope.idempotency_key.clone(),
                        session_id: runtime_execution_envelope
                            .session_id
                            .map(session_id_to_string),
                        turn_id: Some(runtime_execution_envelope.turn_id.0),
                        device_id: runtime_execution_envelope
                            .device_identity
                            .as_str()
                            .to_string(),
                        device_turn_sequence: runtime_execution_envelope.device_turn_sequence,
                        submission_timestamp_ns: now.0,
                        retry_counter: 0,
                        acknowledgement_state:
                            PersistenceAcknowledgementState::PendingCloudAcknowledgement,
                        cleared_from_active_outbox_at_ns: None,
                        last_reconciliation_decision: None,
                        last_conflict_severity: None,
                        request: request.clone(),
                    };
                    validate_outbox_record(&record)?;
                    guard
                        .outbox_records
                        .insert(operation_id.clone(), record.clone());
                    let recovery_mode = guard.recovery_mode;
                    append_persistence_operation_journal_locked(
                        &mut guard,
                        now,
                        AdapterOperationJournalEventKind::Created,
                        &operation_id,
                        &record.actor_user_id,
                        &record.idempotency_key,
                        record.session_id.clone(),
                        record.turn_id,
                        &record.device_id,
                        record.device_turn_sequence,
                        record.acknowledgement_state,
                        recovery_mode,
                        None,
                        None,
                        None,
                        &self.runtime_node_id,
                    );
                    append_persistence_operation_journal_locked(
                        &mut guard,
                        now,
                        AdapterOperationJournalEventKind::SubmissionAttempted,
                        &operation_id,
                        &record.actor_user_id,
                        &record.idempotency_key,
                        record.session_id.clone(),
                        record.turn_id,
                        &record.device_id,
                        record.device_turn_sequence,
                        record.acknowledgement_state,
                        recovery_mode,
                        None,
                        None,
                        None,
                        &self.runtime_node_id,
                    );
                }
            }
            PersistenceInvocationMode::ExistingOutboxReplay => {
                let existing_snapshot = guard
                    .outbox_records
                    .get(&operation_id)
                    .cloned()
                    .ok_or_else(|| "persistence_outbox_missing_for_replay".to_string())?;
                validate_outbox_record(&existing_snapshot)?;
                let recovery_mode = guard.recovery_mode;
                if !outbox_record_request_matches(
                    &existing_snapshot,
                    request,
                    runtime_execution_envelope,
                ) {
                    {
                        let existing = guard
                            .outbox_records
                            .get_mut(&operation_id)
                            .expect("existing outbox row must still exist");
                        existing.acknowledgement_state =
                            PersistenceAcknowledgementState::QuarantinedLocalState;
                        existing.cleared_from_active_outbox_at_ns = Some(now.0);
                        existing.last_reconciliation_decision =
                            Some(ReconciliationDecision::QuarantineLocalState);
                        existing.last_conflict_severity =
                            Some(PersistenceConflictSeverity::QuarantineRequired);
                    }
                    append_persistence_operation_journal_locked(
                        &mut guard,
                        now,
                        AdapterOperationJournalEventKind::Quarantined,
                        &operation_id,
                        &existing_snapshot.actor_user_id,
                        &existing_snapshot.idempotency_key,
                        existing_snapshot.session_id.clone(),
                        existing_snapshot.turn_id,
                        &existing_snapshot.device_id,
                        existing_snapshot.device_turn_sequence,
                        PersistenceAcknowledgementState::QuarantinedLocalState,
                        recovery_mode,
                        Some(ReconciliationDecision::QuarantineLocalState),
                        Some(PersistenceConflictSeverity::QuarantineRequired),
                        Some("replay_request_mismatch".to_string()),
                        &self.runtime_node_id,
                    );
                    append_persistence_audit_locked(
                        &mut guard,
                        now,
                        AdapterPersistenceAuditDecision::QuarantineAction,
                        Some(operation_id.clone()),
                        Some(existing_snapshot.actor_user_id.clone()),
                        Some(existing_snapshot.idempotency_key.clone()),
                        existing_snapshot.session_id.clone(),
                        existing_snapshot.turn_id,
                        Some(ReconciliationDecision::QuarantineLocalState),
                        Some(PersistenceConflictSeverity::QuarantineRequired),
                        &self.runtime_node_id,
                        Some("replay request mismatch quarantined local persistence".to_string()),
                    );
                    set_persistence_recovery_mode_locked(
                        &mut guard,
                        now,
                        PersistenceRecoveryMode::QuarantinedLocalState,
                        &self.runtime_node_id,
                        Some("replay request mismatch".to_string()),
                    );
                    self.save_persistence_state_to_disk_locked(&guard)?;
                    let decision = self.ingress.govern_persistence_signal(
                        Some(runtime_execution_envelope),
                        GovernanceProtectedActionClass::PersistenceReplay,
                        "persistence_quarantine_required replay_request_mismatch",
                        Some("replay request mismatch quarantined local persistence".to_string()),
                    );
                    return Err(governance_runtime_reason(&decision));
                }
                {
                    let existing = guard
                        .outbox_records
                        .get_mut(&operation_id)
                        .expect("existing outbox row must still exist");
                    existing.retry_counter = existing.retry_counter.saturating_add(1);
                }
                reconciliation_decision = Some(ReconciliationDecision::RetrySameOperation);
                conflict_severity = Some(PersistenceConflictSeverity::Retryable);
                if existing_snapshot.session_id.is_some() {
                    reconciliation_decision =
                        Some(ReconciliationDecision::RequestFreshSessionState);
                    conflict_severity = None;
                    append_persistence_operation_journal_locked(
                        &mut guard,
                        now,
                        AdapterOperationJournalEventKind::FreshSessionStateRequested,
                        &operation_id,
                        &existing_snapshot.actor_user_id,
                        &existing_snapshot.idempotency_key,
                        existing_snapshot.session_id.clone(),
                        existing_snapshot.turn_id,
                        &existing_snapshot.device_id,
                        existing_snapshot.device_turn_sequence,
                        existing_snapshot.acknowledgement_state,
                        recovery_mode,
                        reconciliation_decision,
                        conflict_severity,
                        Some("session snapshot refresh requested before replay".to_string()),
                        &self.runtime_node_id,
                    );
                }
                append_persistence_operation_journal_locked(
                    &mut guard,
                    now,
                    AdapterOperationJournalEventKind::RetryAttempted,
                    &operation_id,
                    &existing_snapshot.actor_user_id,
                    &existing_snapshot.idempotency_key,
                    existing_snapshot.session_id.clone(),
                    existing_snapshot.turn_id,
                    &existing_snapshot.device_id,
                    existing_snapshot.device_turn_sequence,
                    existing_snapshot.acknowledgement_state,
                    recovery_mode,
                    reconciliation_decision,
                    conflict_severity,
                    None,
                    &self.runtime_node_id,
                );
                set_persistence_recovery_mode_locked(
                    &mut guard,
                    now,
                    PersistenceRecoveryMode::Recovering,
                    &self.runtime_node_id,
                    Some("reconciling pending outbox operation".to_string()),
                );
            }
            PersistenceInvocationMode::LegacyJournalReplay => unreachable!(),
        }
        let audit_ref = if reconciliation_decision.is_some() {
            Some(
                append_persistence_audit_locked(
                    &mut guard,
                    now,
                    AdapterPersistenceAuditDecision::ReconciliationDecision,
                    Some(operation_id.clone()),
                    Some(request.actor_user_id.clone()),
                    Some(runtime_execution_envelope.idempotency_key.clone()),
                    runtime_execution_envelope
                        .session_id
                        .map(session_id_to_string),
                    Some(runtime_execution_envelope.turn_id.0),
                    reconciliation_decision,
                    conflict_severity,
                    &self.runtime_node_id,
                    Some("reconciliation policy decision recorded".to_string()),
                )
                .to_string(),
            )
        } else {
            None
        };
        self.save_persistence_state_to_disk_locked(&guard)?;
        Ok(Some(PreparedPersistenceOperation {
            operation_id,
            persistence_state: build_persistence_execution_state(
                guard.recovery_mode,
                if reconciliation_decision == Some(ReconciliationDecision::RejectStaleOperation) {
                    PersistenceAcknowledgementState::StaleRejected
                } else {
                    PersistenceAcknowledgementState::PendingCloudAcknowledgement
                },
                reconciliation_decision,
                conflict_severity,
                false,
                audit_ref,
            )?,
        }))
    }

    fn finalize_persistence_operation(
        &self,
        prepared: &PreparedPersistenceOperation,
        now: MonotonicTimeNs,
        runtime_device_id: &DeviceId,
        idempotency_key: &str,
        result: &Result<VoiceTurnAdapterResponse, VoiceTurnIngressError>,
    ) -> Result<(), String> {
        let Some(persistence) = self.persistence.as_ref() else {
            return Ok(());
        };
        let mut guard = persistence
            .state
            .lock()
            .map_err(|_| "adapter persistence state lock poisoned".to_string())?;
        validate_persistence_state_integrity(&guard).map_err(|err| {
            format!("persistence state integrity check failed before finalize: {err}")
        })?;
        let record_snapshot = guard
            .outbox_records
            .get(&prepared.operation_id)
            .cloned()
            .ok_or_else(|| "persistence_outbox_missing_for_finalize".to_string())?;
        match result {
            Ok(response) => {
                {
                    let record = guard
                        .outbox_records
                        .get_mut(&prepared.operation_id)
                        .expect("outbox record must still exist");
                    record.session_id = response.session_id.clone();
                    record.turn_id = response.turn_id;
                    record.acknowledgement_state =
                        PersistenceAcknowledgementState::AuthoritativelyAcknowledged;
                    record.cleared_from_active_outbox_at_ns = Some(now.0);
                    record.last_reconciliation_decision =
                        prepared.persistence_state.reconciliation_decision;
                    record.last_conflict_severity = Some(PersistenceConflictSeverity::Info);
                }
                guard.authoritative_outcomes.insert(
                    actor_idempotency_lookup_key(&record_snapshot.actor_user_id, idempotency_key),
                    AdapterAuthoritativeOutcomeRecord {
                        actor_user_id: record_snapshot.actor_user_id.clone(),
                        idempotency_key: idempotency_key.to_string(),
                        session_id: response.session_id.clone(),
                        turn_id: response.turn_id,
                        session_state: response.session_state.clone(),
                        device_id: runtime_device_id.as_str().to_string(),
                        device_turn_sequence: record_snapshot.device_turn_sequence,
                        acknowledged_at_ns: now.0,
                        runtime_node_id: self.runtime_node_id.clone(),
                        governance_policy_version: Some(
                            self.ingress.runtime_governance_policy_version().to_string(),
                        ),
                        conflict_severity: Some(PersistenceConflictSeverity::Info),
                        result: AdapterPersistedAuthoritativeResult::Success(response.clone()),
                    },
                );
                let recovery_mode = guard.recovery_mode;
                append_persistence_operation_journal_locked(
                    &mut guard,
                    now,
                    AdapterOperationJournalEventKind::Acknowledged,
                    &prepared.operation_id,
                    &record_snapshot.actor_user_id,
                    idempotency_key,
                    response.session_id.clone(),
                    response.turn_id,
                    &record_snapshot.device_id,
                    record_snapshot.device_turn_sequence,
                    PersistenceAcknowledgementState::AuthoritativelyAcknowledged,
                    recovery_mode,
                    prepared.persistence_state.reconciliation_decision,
                    Some(PersistenceConflictSeverity::Info),
                    None,
                    &self.runtime_node_id,
                );
                append_persistence_audit_locked(
                    &mut guard,
                    now,
                    AdapterPersistenceAuditDecision::AcknowledgementOutcome,
                    Some(prepared.operation_id.clone()),
                    Some(record_snapshot.actor_user_id.clone()),
                    Some(idempotency_key.to_string()),
                    response.session_id.clone(),
                    response.turn_id,
                    prepared.persistence_state.reconciliation_decision,
                    Some(PersistenceConflictSeverity::Info),
                    &self.runtime_node_id,
                    Some("authoritative acknowledgement recorded".to_string()),
                );
            }
            Err(error) => {
                let retryable = error.failure_class == FailureClass::RetryableRuntime
                    || (error.failure_class == FailureClass::SessionConflict
                        && !is_stale_conflict_reason(&error.reason_code));
                if retryable {
                    let (record_session_id, record_turn_id, acknowledgement_state) = {
                        let record = guard
                            .outbox_records
                            .get_mut(&prepared.operation_id)
                            .expect("outbox record must still exist");
                        if error.session_id.is_some() {
                            record.session_id = error.session_id.clone();
                        }
                        if let Some(turn_id) = error.turn_id {
                            record.turn_id = Some(turn_id);
                        }
                        record.retry_counter = record.retry_counter.saturating_add(1);
                        record.last_reconciliation_decision =
                            Some(ReconciliationDecision::RetrySameOperation);
                        record.last_conflict_severity =
                            Some(PersistenceConflictSeverity::Retryable);
                        (
                            record.session_id.clone(),
                            record.turn_id,
                            record.acknowledgement_state,
                        )
                    };
                    let recovery_mode = guard.recovery_mode;
                    append_persistence_operation_journal_locked(
                        &mut guard,
                        now,
                        AdapterOperationJournalEventKind::RetryAttempted,
                        &prepared.operation_id,
                        &record_snapshot.actor_user_id,
                        idempotency_key,
                        record_session_id,
                        record_turn_id,
                        &record_snapshot.device_id,
                        record_snapshot.device_turn_sequence,
                        acknowledgement_state,
                        recovery_mode,
                        Some(ReconciliationDecision::RetrySameOperation),
                        Some(PersistenceConflictSeverity::Retryable),
                        error.reason.clone(),
                        &self.runtime_node_id,
                    );
                    set_persistence_recovery_mode_locked(
                        &mut guard,
                        now,
                        PersistenceRecoveryMode::DegradedRecovery,
                        &self.runtime_node_id,
                        Some("retryable persistence failure retained in outbox".to_string()),
                    );
                } else {
                    let ack_state = terminal_persistence_failure_state(error);
                    let conflict_severity = terminal_persistence_failure_severity(error);
                    {
                        let record = guard
                            .outbox_records
                            .get_mut(&prepared.operation_id)
                            .expect("outbox record must still exist");
                        record.session_id = error.session_id.clone();
                        record.turn_id = error.turn_id;
                        record.acknowledgement_state = ack_state;
                        record.cleared_from_active_outbox_at_ns = Some(now.0);
                        record.last_reconciliation_decision =
                            prepared.persistence_state.reconciliation_decision;
                        record.last_conflict_severity = Some(conflict_severity);
                    }
                    guard.authoritative_outcomes.insert(
                        actor_idempotency_lookup_key(
                            &record_snapshot.actor_user_id,
                            idempotency_key,
                        ),
                        AdapterAuthoritativeOutcomeRecord {
                            actor_user_id: record_snapshot.actor_user_id.clone(),
                            idempotency_key: idempotency_key.to_string(),
                            session_id: error.session_id.clone(),
                            turn_id: error.turn_id,
                            session_state: error.session_state.clone(),
                            device_id: runtime_device_id.as_str().to_string(),
                            device_turn_sequence: record_snapshot.device_turn_sequence,
                            acknowledged_at_ns: now.0,
                            runtime_node_id: self.runtime_node_id.clone(),
                            governance_policy_version: Some(
                                self.ingress.runtime_governance_policy_version().to_string(),
                            ),
                            conflict_severity: Some(conflict_severity),
                            result: AdapterPersistedAuthoritativeResult::Failure(error.clone()),
                        },
                    );
                    let recovery_mode = guard.recovery_mode;
                    append_persistence_operation_journal_locked(
                        &mut guard,
                        now,
                        AdapterOperationJournalEventKind::TerminalFailure,
                        &prepared.operation_id,
                        &record_snapshot.actor_user_id,
                        idempotency_key,
                        error.session_id.clone(),
                        error.turn_id,
                        &record_snapshot.device_id,
                        record_snapshot.device_turn_sequence,
                        ack_state,
                        recovery_mode,
                        prepared.persistence_state.reconciliation_decision,
                        Some(conflict_severity),
                        error.reason.clone(),
                        &self.runtime_node_id,
                    );
                    append_persistence_audit_locked(
                        &mut guard,
                        now,
                        AdapterPersistenceAuditDecision::AcknowledgementOutcome,
                        Some(prepared.operation_id.clone()),
                        Some(record_snapshot.actor_user_id.clone()),
                        Some(idempotency_key.to_string()),
                        error.session_id.clone(),
                        error.turn_id,
                        prepared.persistence_state.reconciliation_decision,
                        Some(conflict_severity),
                        &self.runtime_node_id,
                        Some("terminal authoritative failure recorded".to_string()),
                    );
                }
            }
        }
        let next_recovery_mode = recompute_persistence_recovery_mode_locked(&guard);
        set_persistence_recovery_mode_locked(
            &mut guard,
            now,
            next_recovery_mode,
            &self.runtime_node_id,
            Some("persistence finalization completed".to_string()),
        );
        self.save_persistence_state_to_disk_locked(&guard)
    }

    fn sync_authoritative_outcomes_into_retry_cache(
        &self,
        state: &AdapterPersistenceState,
    ) -> Result<(), String> {
        let mut cache = self
            .session_retry_cache
            .lock()
            .map_err(|_| "adapter retry cache lock poisoned".to_string())?;
        cache.clear();
        for outcome in state.authoritative_outcomes.values() {
            if let AdapterPersistedAuthoritativeResult::Success(response) = &outcome.result {
                cache.insert(
                    AdapterRetryCacheKey {
                        actor_user_id: UserId::new(outcome.actor_user_id.clone())
                            .map_err(|err| format!("invalid persisted actor_user_id: {err:?}"))?,
                        idempotency_key: outcome.idempotency_key.clone(),
                    },
                    response.clone(),
                );
            }
        }
        Ok(())
    }

    fn reconcile_pending_outbox_records(&self) -> Result<(), String> {
        let Some(persistence) = self.persistence.as_ref() else {
            return Ok(());
        };
        let pending_operation_ids = {
            let mut guard = persistence
                .state
                .lock()
                .map_err(|_| "adapter persistence state lock poisoned".to_string())?;
            validate_persistence_state_integrity(&guard).map_err(|err| {
                format!(
                    "persistence state integrity check failed before reconciliation replay: {err}"
                )
            })?;
            let pending = sorted_pending_outbox_operation_ids(&guard);
            if !pending.is_empty() {
                set_persistence_recovery_mode_locked(
                    &mut guard,
                    MonotonicTimeNs(1),
                    PersistenceRecoveryMode::Recovering,
                    &self.runtime_node_id,
                    Some(format!("reconciling {} pending operations", pending.len())),
                );
                self.save_persistence_state_to_disk_locked(&guard)?;
            }
            pending
        };
        for operation_id in pending_operation_ids {
            let (record, maybe_stale_error, prepared) = {
                let mut guard = persistence
                    .state
                    .lock()
                    .map_err(|_| "adapter persistence state lock poisoned".to_string())?;
                let Some(record) = guard.outbox_records.get(&operation_id).cloned() else {
                    continue;
                };
                if let Err(err) = validate_outbox_record(&record) {
                    if let Some(row) = guard.outbox_records.get_mut(&operation_id) {
                        row.acknowledgement_state =
                            PersistenceAcknowledgementState::QuarantinedLocalState;
                        row.cleared_from_active_outbox_at_ns = Some(1);
                        row.last_reconciliation_decision =
                            Some(ReconciliationDecision::QuarantineLocalState);
                        row.last_conflict_severity =
                            Some(PersistenceConflictSeverity::QuarantineRequired);
                    }
                    let recovery_mode = guard.recovery_mode;
                    append_persistence_operation_journal_locked(
                        &mut guard,
                        MonotonicTimeNs(1),
                        AdapterOperationJournalEventKind::Quarantined,
                        &operation_id,
                        &record.actor_user_id,
                        &record.idempotency_key,
                        record.session_id.clone(),
                        record.turn_id,
                        &record.device_id,
                        record.device_turn_sequence,
                        PersistenceAcknowledgementState::QuarantinedLocalState,
                        recovery_mode,
                        Some(ReconciliationDecision::QuarantineLocalState),
                        Some(PersistenceConflictSeverity::QuarantineRequired),
                        Some(err),
                        &self.runtime_node_id,
                    );
                    append_persistence_audit_locked(
                        &mut guard,
                        MonotonicTimeNs(1),
                        AdapterPersistenceAuditDecision::QuarantineAction,
                        Some(operation_id.clone()),
                        Some(record.actor_user_id.clone()),
                        Some(record.idempotency_key.clone()),
                        record.session_id.clone(),
                        record.turn_id,
                        Some(ReconciliationDecision::QuarantineLocalState),
                        Some(PersistenceConflictSeverity::QuarantineRequired),
                        &self.runtime_node_id,
                        Some("invalid durable outbox record quarantined".to_string()),
                    );
                    set_persistence_recovery_mode_locked(
                        &mut guard,
                        MonotonicTimeNs(1),
                        PersistenceRecoveryMode::QuarantinedLocalState,
                        &self.runtime_node_id,
                        Some("invalid durable outbox record".to_string()),
                    );
                    self.save_persistence_state_to_disk_locked(&guard)?;
                    let _ = self.ingress.govern_persistence_signal(
                        None,
                        GovernanceProtectedActionClass::PersistenceRecovery,
                        "persistence_quarantine_required invalid_outbox_record",
                        Some("invalid durable outbox record quarantined".to_string()),
                    );
                    continue;
                }
                let stale_decision =
                    stale_replay_error_for_record(&self.ingress, &guard, &record, None)
                        .map(|_| ReconciliationDecision::RejectStaleOperation);
                let stale_conflict =
                    stale_decision.map(|_| PersistenceConflictSeverity::StaleRejected);
                let stale_audit_ref = if stale_decision.is_some() {
                    Some(
                        append_persistence_audit_locked(
                            &mut guard,
                            MonotonicTimeNs(1),
                            AdapterPersistenceAuditDecision::ReconciliationDecision,
                            Some(operation_id.clone()),
                            Some(record.actor_user_id.clone()),
                            Some(record.idempotency_key.clone()),
                            record.session_id.clone(),
                            record.turn_id,
                            stale_decision,
                            stale_conflict,
                            &self.runtime_node_id,
                            Some("stale replay policy decision recorded".to_string()),
                        )
                        .to_string(),
                    )
                } else {
                    None
                };
                let prepared = PreparedPersistenceOperation {
                    operation_id: operation_id.clone(),
                    persistence_state: build_persistence_execution_state(
                        guard.recovery_mode,
                        if stale_decision == Some(ReconciliationDecision::RejectStaleOperation) {
                            PersistenceAcknowledgementState::StaleRejected
                        } else {
                            PersistenceAcknowledgementState::PendingCloudAcknowledgement
                        },
                        stale_decision,
                        stale_conflict,
                        false,
                        stale_audit_ref,
                    )?,
                };
                let stale_error = stale_replay_error_for_record(
                    &self.ingress,
                    &guard,
                    &record,
                    Some(prepared.persistence_state.clone()),
                );
                (record, stale_error, prepared)
            };
            if let Some(error) = maybe_stale_error {
                self.finalize_persistence_operation(
                    &prepared,
                    MonotonicTimeNs(record.submission_timestamp_ns.max(1)),
                    &DeviceId::new(record.device_id.clone())
                        .map_err(|err| format!("invalid persisted device_id: {err:?}"))?,
                    &record.idempotency_key,
                    &Err(error),
                )?;
                continue;
            }
            let _ = self.run_voice_turn_internal(
                record.request,
                None,
                true,
                true,
                PersistenceInvocationMode::ExistingOutboxReplay,
            );
        }
        let mut guard = persistence
            .state
            .lock()
            .map_err(|_| "adapter persistence state lock poisoned".to_string())?;
        guard.last_reconciled_at_ns = Some(1);
        let next_recovery_mode = recompute_persistence_recovery_mode_locked(&guard);
        set_persistence_recovery_mode_locked(
            &mut guard,
            MonotonicTimeNs(1),
            next_recovery_mode,
            &self.runtime_node_id,
            Some("reconciliation pass completed".to_string()),
        );
        self.save_persistence_state_to_disk_locked(&guard)
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
        for path in [&persistence.legacy_journal_path, &persistence.state_path] {
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
                        "failed to create adapter persistence file '{}': {}",
                        path.display(),
                        err
                    )
                })?;
            }
        }
        Ok(())
    }

    fn bootstrap_persistence_runtime(&self) -> Result<(), String> {
        let Some(persistence) = self.persistence.as_ref() else {
            return Ok(());
        };
        let mut state = self.load_persistence_state_from_disk()?;
        if state.recovery_mode == PersistenceRecoveryMode::QuarantinedLocalState {
            let _ = self.ingress.govern_persistence_signal(
                None,
                GovernanceProtectedActionClass::PersistenceRecovery,
                "persistence_quarantine_required state_bootstrap_quarantined",
                Some("persistence bootstrap entered quarantined local state".to_string()),
            );
        }
        self.replay_legacy_journal_into_store(&mut state)?;
        {
            let mut guard = persistence
                .state
                .lock()
                .map_err(|_| "adapter persistence state lock poisoned".to_string())?;
            *guard = state;
            self.sync_authoritative_outcomes_into_retry_cache(&guard)?;
            self.save_persistence_state_to_disk_locked(&guard)?;
        }
        self.reconcile_pending_outbox_records()?;
        Ok(())
    }

    fn load_persistence_state_from_disk(&self) -> Result<AdapterPersistenceState, String> {
        let Some(persistence) = self.persistence.as_ref() else {
            return Ok(AdapterPersistenceState::default());
        };
        let quarantine_state_with_note =
            |suffix: &str, note: String| -> Result<AdapterPersistenceState, String> {
                let mut quarantined = AdapterPersistenceState {
                    recovery_mode: PersistenceRecoveryMode::QuarantinedLocalState,
                    ..AdapterPersistenceState::default()
                };
                append_persistence_audit_locked(
                    &mut quarantined,
                    MonotonicTimeNs(1),
                    AdapterPersistenceAuditDecision::QuarantineAction,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(PersistenceConflictSeverity::QuarantineRequired),
                    &self.runtime_node_id,
                    Some(note),
                );
                let quarantine_path = quarantined_persistence_path(&persistence.state_path, suffix);
                fs::rename(&persistence.state_path, &quarantine_path).map_err(|rename_err| {
                    format!(
                        "failed quarantining adapter persistence state '{}' -> '{}': {}",
                        persistence.state_path.display(),
                        quarantine_path.display(),
                        rename_err
                    )
                })?;
                File::create(&persistence.state_path).map_err(|create_err| {
                    format!(
                        "failed recreating adapter persistence state '{}': {}",
                        persistence.state_path.display(),
                        create_err
                    )
                })?;
                Ok(quarantined)
            };
        let raw = fs::read_to_string(&persistence.state_path).map_err(|err| {
            format!(
                "failed reading adapter persistence state '{}': {}",
                persistence.state_path.display(),
                err
            )
        })?;
        if raw.trim().is_empty() {
            return Ok(AdapterPersistenceState::default());
        }
        match serde_json::from_str::<AdapterPersistenceState>(&raw) {
            Ok(state) if state.schema_version == 2 => {
                if let Err(err) = validate_persistence_state_integrity(&state) {
                    quarantine_state_with_note(
                        "state_integrity",
                        format!(
                            "persistence state integrity validation quarantined: {} ({err})",
                            persistence.state_path.display()
                        ),
                    )
                } else {
                    Ok(state)
                }
            }
            Ok(state) => Err(format!(
                "unsupported adapter persistence schema_version={}",
                state.schema_version
            )),
            Err(err) => quarantine_state_with_note(
                "state_corrupt",
                format!(
                    "corrupted persistence state quarantined: {} ({err})",
                    persistence.state_path.display()
                ),
            ),
        }
    }

    fn save_persistence_state_to_disk_locked(
        &self,
        state: &AdapterPersistenceState,
    ) -> Result<(), String> {
        let Some(persistence) = self.persistence.as_ref() else {
            return Ok(());
        };
        validate_persistence_state_integrity(state).map_err(|err| {
            format!("refusing to persist adapter state with integrity violation: {err}")
        })?;
        let json = serde_json::to_vec_pretty(state)
            .map_err(|err| format!("failed encoding adapter persistence state: {err}"))?;
        let tmp_path = persistence.state_path.with_extension("json.tmp");
        let mut file = File::create(&tmp_path).map_err(|err| {
            format!(
                "failed creating adapter persistence temp file '{}': {}",
                tmp_path.display(),
                err
            )
        })?;
        file.write_all(&json)
            .and_then(|_| file.sync_data())
            .map_err(|err| {
                format!(
                    "failed writing adapter persistence temp file '{}': {}",
                    tmp_path.display(),
                    err
                )
            })?;
        fs::rename(&tmp_path, &persistence.state_path).map_err(|err| {
            format!(
                "failed replacing adapter persistence state '{}': {}",
                persistence.state_path.display(),
                err
            )
        })?;
        Ok(())
    }

    fn replay_legacy_journal_into_store(
        &self,
        state: &mut AdapterPersistenceState,
    ) -> Result<(), String> {
        let Some(persistence) = self.persistence.as_ref() else {
            return Ok(());
        };
        let file = File::open(&persistence.legacy_journal_path).map_err(|err| {
            format!(
                "failed to open adapter store journal '{}': {}",
                persistence.legacy_journal_path.display(),
                err
            )
        })?;
        for (line_no, line_result) in BufReader::new(file).lines().enumerate() {
            let line = line_result.map_err(|err| {
                format!(
                    "failed reading adapter store journal '{}' at line {}: {}",
                    persistence.legacy_journal_path.display(),
                    line_no + 1,
                    err
                )
            })?;
            if line.trim().is_empty() {
                continue;
            }
            let entry: AdapterJournalEntry = match serde_json::from_str(&line) {
                Ok(entry) => entry,
                Err(err) => {
                    state.recovery_mode = PersistenceRecoveryMode::QuarantinedLocalState;
                    append_persistence_audit_locked(
                        state,
                        MonotonicTimeNs((line_no as u64).saturating_add(1)),
                        AdapterPersistenceAuditDecision::QuarantineAction,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        Some(PersistenceConflictSeverity::QuarantineRequired),
                        &self.runtime_node_id,
                        Some(format!(
                            "legacy journal replay quarantined at line {}: {err}",
                            line_no + 1
                        )),
                    );
                    let quarantine_path = quarantined_persistence_path(
                        &persistence.legacy_journal_path,
                        "legacy_journal_corrupt",
                    );
                    fs::rename(&persistence.legacy_journal_path, &quarantine_path).map_err(
                        |rename_err| {
                            format!(
                                "failed quarantining legacy journal '{}' -> '{}': {}",
                                persistence.legacy_journal_path.display(),
                                quarantine_path.display(),
                                rename_err
                            )
                        },
                    )?;
                    File::create(&persistence.legacy_journal_path).map_err(|create_err| {
                        format!(
                            "failed recreating legacy journal '{}': {}",
                            persistence.legacy_journal_path.display(),
                            create_err
                        )
                    })?;
                    return Ok(());
                }
            };
            if entry.schema_version != 1 {
                return Err(format!(
                    "unsupported adapter store journal schema_version={} at line {}",
                    entry.schema_version,
                    line_no + 1
                ));
            }
            self.run_voice_turn_internal(
                entry.request,
                None,
                false,
                true,
                PersistenceInvocationMode::LegacyJournalReplay,
            )
            .map_err(|err| {
                format!(
                    "journal replay failed at line {}: {}",
                    line_no + 1,
                    err.to_runtime_reason()
                )
            })?;
            append_persistence_audit_locked(
                state,
                MonotonicTimeNs((line_no as u64).saturating_add(1)),
                AdapterPersistenceAuditDecision::LegacyJournalReplay,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                &self.runtime_node_id,
                Some(format!("replayed legacy journal line {}", line_no + 1)),
            );
        }
        Ok(())
    }

    fn append_legacy_journal_entry(&self, request: VoiceTurnAdapterRequest) -> Result<(), String> {
        let Some(persistence) = self.persistence.as_ref() else {
            return Ok(());
        };
        let entry = AdapterJournalEntry::v1(request);
        let json = serde_json::to_string(&entry)
            .map_err(|err| format!("failed to encode adapter journal entry: {err}"))?;
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&persistence.legacy_journal_path)
            .map_err(|err| {
                format!(
                    "failed opening adapter store journal '{}' for append: {}",
                    persistence.legacy_journal_path.display(),
                    err
                )
            })?;
        file.write_all(json.as_bytes())
            .and_then(|_| file.write_all(b"\n"))
            .and_then(|_| file.sync_data())
            .map_err(|err| {
                format!(
                    "failed writing adapter store journal '{}': {}",
                    persistence.legacy_journal_path.display(),
                    err
                )
            })?;
        Ok(())
    }
}

fn adapter_persistence_state_path(journal_path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.state.json", journal_path.display()))
}

fn quarantined_persistence_path(path: &Path, suffix: &str) -> PathBuf {
    PathBuf::from(format!("{}.{}.quarantined", path.display(), suffix))
}

fn actor_idempotency_lookup_key(actor_user_id: &str, idempotency_key: &str) -> String {
    format!("{actor_user_id}::{idempotency_key}")
}

fn derived_operation_id(actor_user_id: &str, idempotency_key: &str) -> String {
    format!(
        "op_{}",
        stable_hash_hex_16(&format!("{actor_user_id}::{idempotency_key}"))
    )
}

fn validate_persistence_ascii_token(
    field: &str,
    value: &str,
    max_len: usize,
) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    if value.len() > max_len {
        return Err(format!("{field} exceeds max length {max_len}"));
    }
    if !value.is_ascii() {
        return Err(format!("{field} must be ASCII"));
    }
    Ok(())
}

fn validate_optional_persistence_ascii_token(
    field: &str,
    value: &Option<String>,
    max_len: usize,
) -> Result<(), String> {
    if let Some(value) = value.as_ref() {
        validate_persistence_ascii_token(field, value, max_len)?;
    }
    Ok(())
}

fn validate_persistence_state_integrity(state: &AdapterPersistenceState) -> Result<(), String> {
    if state.schema_version != 2 {
        return Err(format!(
            "persistence schema_version={} is not supported for integrity validation",
            state.schema_version
        ));
    }

    for (operation_key, record) in &state.outbox_records {
        let expected_operation_id =
            derived_operation_id(&record.actor_user_id, &record.idempotency_key);
        if operation_key != &record.operation_id {
            return Err(format!(
                "outbox key '{operation_key}' must match outbox.operation_id '{}'",
                record.operation_id
            ));
        }
        if record.operation_id != expected_operation_id {
            return Err(format!(
                "outbox.operation_id '{}' must match derived operation_id '{}'",
                record.operation_id, expected_operation_id
            ));
        }
        if record.acknowledgement_state
            == PersistenceAcknowledgementState::PendingCloudAcknowledgement
            && record.cleared_from_active_outbox_at_ns.is_some()
        {
            return Err(format!(
                "pending outbox record '{}' must not already be cleared from the active outbox",
                record.operation_id
            ));
        }
        if record.acknowledgement_state
            != PersistenceAcknowledgementState::PendingCloudAcknowledgement
            && record.cleared_from_active_outbox_at_ns.is_none()
        {
            return Err(format!(
                "non-pending outbox record '{}' must record cleared_from_active_outbox_at_ns",
                record.operation_id
            ));
        }
    }

    let mut expected_journal_sequence = 1_u64;
    for entry in &state.operation_journal {
        if entry.sequence != expected_journal_sequence {
            return Err(format!(
                "operation_journal sequence {} must equal expected contiguous sequence {}",
                entry.sequence, expected_journal_sequence
            ));
        }
        expected_journal_sequence = expected_journal_sequence.saturating_add(1);
        validate_persistence_ascii_token(
            "operation_journal.operation_id",
            &entry.operation_id,
            128,
        )?;
        validate_persistence_ascii_token("operation_journal.request_id", &entry.request_id, 256)?;
        validate_persistence_ascii_token("operation_journal.trace_id", &entry.trace_id, 256)?;
        validate_persistence_ascii_token(
            "operation_journal.actor_user_id",
            &entry.actor_user_id,
            128,
        )?;
        validate_persistence_ascii_token(
            "operation_journal.idempotency_key",
            &entry.idempotency_key,
            256,
        )?;
        validate_optional_persistence_ascii_token(
            "operation_journal.session_id",
            &entry.session_id,
            128,
        )?;
        validate_persistence_ascii_token(
            "operation_journal.runtime_node_id",
            &entry.runtime_node_id,
            128,
        )?;
        if matches!(entry.turn_id, Some(0)) {
            return Err(format!(
                "operation_journal entry '{}' must not record turn_id=0",
                entry.operation_id
            ));
        }
        if matches!(entry.device_turn_sequence, Some(0)) {
            return Err(format!(
                "operation_journal entry '{}' must not record device_turn_sequence=0",
                entry.operation_id
            ));
        }
        if entry.request_id == "unknown_request" || entry.trace_id == "unknown_trace" {
            return Err(format!(
                "operation_journal entry '{}' must not rely on placeholder request or trace identity",
                entry.operation_id
            ));
        }
        let record = state
            .outbox_records
            .get(&entry.operation_id)
            .ok_or_else(|| {
                format!(
                    "operation_journal entry '{}' must reference a live durable outbox record",
                    entry.operation_id
                )
            })?;
        if entry.request_id != record.request_id {
            return Err(format!(
                "operation_journal entry '{}' request_id '{}' must match outbox request_id '{}'",
                entry.operation_id, entry.request_id, record.request_id
            ));
        }
        if entry.trace_id != record.trace_id {
            return Err(format!(
                "operation_journal entry '{}' trace_id '{}' must match outbox trace_id '{}'",
                entry.operation_id, entry.trace_id, record.trace_id
            ));
        }
        if entry.actor_user_id != record.actor_user_id {
            return Err(format!(
                "operation_journal entry '{}' actor_user_id '{}' must match outbox actor_user_id '{}'",
                entry.operation_id, entry.actor_user_id, record.actor_user_id
            ));
        }
        if entry.idempotency_key != record.idempotency_key {
            return Err(format!(
                "operation_journal entry '{}' idempotency_key '{}' must match outbox idempotency_key '{}'",
                entry.operation_id, entry.idempotency_key, record.idempotency_key
            ));
        }
    }
    if state.next_journal_sequence != expected_journal_sequence {
        return Err(format!(
            "next_journal_sequence={} must equal contiguous next sequence {}",
            state.next_journal_sequence, expected_journal_sequence
        ));
    }

    let mut expected_audit_sequence = 1_u64;
    for entry in &state.audit_trail {
        if entry.sequence != expected_audit_sequence {
            return Err(format!(
                "audit_trail sequence {} must equal expected contiguous sequence {}",
                entry.sequence, expected_audit_sequence
            ));
        }
        expected_audit_sequence = expected_audit_sequence.saturating_add(1);
        validate_optional_persistence_ascii_token(
            "audit_trail.operation_id",
            &entry.operation_id,
            128,
        )?;
        validate_optional_persistence_ascii_token(
            "audit_trail.actor_user_id",
            &entry.actor_user_id,
            128,
        )?;
        validate_optional_persistence_ascii_token(
            "audit_trail.idempotency_key",
            &entry.idempotency_key,
            256,
        )?;
        validate_optional_persistence_ascii_token(
            "audit_trail.session_id",
            &entry.session_id,
            128,
        )?;
        validate_persistence_ascii_token(
            "audit_trail.runtime_node_id",
            &entry.runtime_node_id,
            128,
        )?;
        if matches!(entry.turn_id, Some(0)) {
            return Err("audit_trail turn_id must not be 0".to_string());
        }
        if let Some(operation_id) = entry.operation_id.as_ref() {
            let record = state.outbox_records.get(operation_id).ok_or_else(|| {
                format!(
                    "audit_trail entry '{}' must reference a live durable outbox record",
                    operation_id
                )
            })?;
            if entry.actor_user_id.as_deref() != Some(record.actor_user_id.as_str()) {
                return Err(format!(
                    "audit_trail entry '{}' actor_user_id must match durable outbox actor_user_id",
                    operation_id
                ));
            }
            if entry.idempotency_key.as_deref() != Some(record.idempotency_key.as_str()) {
                return Err(format!(
                    "audit_trail entry '{}' idempotency_key must match durable outbox idempotency_key",
                    operation_id
                ));
            }
        }
    }
    if state.next_audit_sequence != expected_audit_sequence {
        return Err(format!(
            "next_audit_sequence={} must equal contiguous next sequence {}",
            state.next_audit_sequence, expected_audit_sequence
        ));
    }

    for (lookup_key, outcome) in &state.authoritative_outcomes {
        validate_persistence_ascii_token(
            "authoritative_outcome.actor_user_id",
            &outcome.actor_user_id,
            128,
        )?;
        validate_persistence_ascii_token(
            "authoritative_outcome.idempotency_key",
            &outcome.idempotency_key,
            256,
        )?;
        validate_optional_persistence_ascii_token(
            "authoritative_outcome.session_id",
            &outcome.session_id,
            128,
        )?;
        validate_persistence_ascii_token(
            "authoritative_outcome.device_id",
            &outcome.device_id,
            128,
        )?;
        validate_persistence_ascii_token(
            "authoritative_outcome.runtime_node_id",
            &outcome.runtime_node_id,
            128,
        )?;
        let expected_lookup_key =
            actor_idempotency_lookup_key(&outcome.actor_user_id, &outcome.idempotency_key);
        if lookup_key != &expected_lookup_key {
            return Err(format!(
                "authoritative_outcome key '{lookup_key}' must match actor-idempotency lookup key '{expected_lookup_key}'"
            ));
        }
        let operation_id = derived_operation_id(&outcome.actor_user_id, &outcome.idempotency_key);
        let record = state.outbox_records.get(&operation_id).ok_or_else(|| {
            format!(
                "authoritative_outcome '{}' must reference a live durable outbox record",
                operation_id
            )
        })?;
        if record.actor_user_id != outcome.actor_user_id {
            return Err(format!(
                "authoritative_outcome '{}' actor_user_id must match durable outbox actor_user_id",
                operation_id
            ));
        }
        if record.idempotency_key != outcome.idempotency_key {
            return Err(format!(
                "authoritative_outcome '{}' idempotency_key must match durable outbox idempotency_key",
                operation_id
            ));
        }
        match &outcome.result {
            AdapterPersistedAuthoritativeResult::Success(response) => {
                if response.session_id != outcome.session_id {
                    return Err(format!(
                        "authoritative_outcome '{}' success session_id must match stored outcome session_id",
                        operation_id
                    ));
                }
                if response.turn_id != outcome.turn_id {
                    return Err(format!(
                        "authoritative_outcome '{}' success turn_id must match stored outcome turn_id",
                        operation_id
                    ));
                }
            }
            AdapterPersistedAuthoritativeResult::Failure(error) => {
                if error.session_id != outcome.session_id {
                    return Err(format!(
                        "authoritative_outcome '{}' failure session_id must match stored outcome session_id",
                        operation_id
                    ));
                }
                if error.turn_id != outcome.turn_id {
                    return Err(format!(
                        "authoritative_outcome '{}' failure turn_id must match stored outcome turn_id",
                        operation_id
                    ));
                }
            }
        }
    }

    Ok(())
}

fn persistence_convergence_state_for(
    acknowledgement_state: PersistenceAcknowledgementState,
    reconciliation_decision: Option<ReconciliationDecision>,
) -> PersistenceConvergenceState {
    match reconciliation_decision {
        Some(ReconciliationDecision::ReusePriorAuthoritativeOutcome) => {
            PersistenceConvergenceState::ConvergedToCloudTruth
        }
        Some(ReconciliationDecision::RejectStaleOperation) => {
            PersistenceConvergenceState::CloudTruthPreserved
        }
        Some(ReconciliationDecision::QuarantineLocalState) => {
            PersistenceConvergenceState::QuarantinedLocalState
        }
        Some(ReconciliationDecision::RetrySameOperation)
        | Some(ReconciliationDecision::RequestFreshSessionState)
        | None => match acknowledgement_state {
            PersistenceAcknowledgementState::PendingCloudAcknowledgement => {
                PersistenceConvergenceState::PendingCloudTruth
            }
            PersistenceAcknowledgementState::AuthoritativelyAcknowledged => {
                PersistenceConvergenceState::ConvergedToCloudTruth
            }
            PersistenceAcknowledgementState::StaleRejected => {
                PersistenceConvergenceState::CloudTruthPreserved
            }
            PersistenceAcknowledgementState::QuarantinedLocalState => {
                PersistenceConvergenceState::QuarantinedLocalState
            }
        },
    }
}

fn build_persistence_execution_state(
    recovery_mode: PersistenceRecoveryMode,
    acknowledgement_state: PersistenceAcknowledgementState,
    reconciliation_decision: Option<ReconciliationDecision>,
    conflict_severity: Option<PersistenceConflictSeverity>,
    cross_node_dedupe_applied: bool,
    audit_ref: Option<String>,
) -> Result<PersistenceExecutionState, String> {
    PersistenceExecutionState::v1(
        recovery_mode,
        acknowledgement_state,
        reconciliation_decision,
        conflict_severity,
        cross_node_dedupe_applied,
        persistence_convergence_state_for(acknowledgement_state, reconciliation_decision),
        audit_ref,
    )
    .map_err(|err| format!("invalid persistence execution state: {err:?}"))
}

fn validate_outbox_record(record: &AdapterOutboxRecord) -> Result<(), String> {
    validate_persistence_ascii_token("outbox.operation_id", &record.operation_id, 128)?;
    validate_persistence_ascii_token("outbox.request_id", &record.request_id, 256)?;
    validate_persistence_ascii_token("outbox.trace_id", &record.trace_id, 256)?;
    validate_persistence_ascii_token("outbox.actor_user_id", &record.actor_user_id, 128)?;
    validate_persistence_ascii_token("outbox.idempotency_key", &record.idempotency_key, 256)?;
    validate_persistence_ascii_token("outbox.device_id", &record.device_id, 128)?;
    if let Some(session_id) = record.session_id.as_ref() {
        validate_persistence_ascii_token("outbox.session_id", session_id, 128)?;
    }
    if matches!(record.turn_id, Some(0)) {
        return Err("outbox.turn_id must be > 0 when provided".to_string());
    }
    if matches!(record.device_turn_sequence, Some(0)) {
        return Err("outbox.device_turn_sequence must be > 0 when provided".to_string());
    }
    if record.request.actor_user_id != record.actor_user_id {
        return Err("outbox.request.actor_user_id must match outbox.actor_user_id".to_string());
    }
    if record.request.turn_id != record.turn_id.unwrap_or(record.request.turn_id) {
        return Err("outbox.request.turn_id must match outbox.turn_id".to_string());
    }
    if record
        .request
        .device_id
        .as_deref()
        .unwrap_or(&record.device_id)
        != record.device_id
    {
        return Err("outbox.request.device_id must match outbox.device_id".to_string());
    }
    if record.request.device_turn_sequence.unwrap_or(
        record
            .device_turn_sequence
            .unwrap_or(record.request.turn_id),
    ) != record.device_turn_sequence.unwrap_or(
        record
            .request
            .device_turn_sequence
            .unwrap_or(record.request.turn_id),
    ) {
        return Err(
            "outbox.request.device_turn_sequence must match outbox.device_turn_sequence"
                .to_string(),
        );
    }
    if record.request.app_platform.trim().is_empty() {
        return Err("outbox.request.app_platform must not be empty".to_string());
    }
    if record.request.trigger.trim().is_empty() {
        return Err("outbox.request.trigger must not be empty".to_string());
    }
    Ok(())
}

fn outbox_record_request_matches(
    record: &AdapterOutboxRecord,
    request: &VoiceTurnAdapterRequest,
    runtime_execution_envelope: &RuntimeExecutionEnvelope,
) -> bool {
    record.actor_user_id == request.actor_user_id
        && record.idempotency_key == runtime_execution_envelope.idempotency_key
        && record.request_id == runtime_execution_envelope.request_id
        && record.trace_id == runtime_execution_envelope.trace_id
        && record.device_id == runtime_execution_envelope.device_identity.as_str()
        && record.turn_id == Some(request.turn_id)
        && record.device_turn_sequence == runtime_execution_envelope.device_turn_sequence
        && record.request.device_id == request.device_id
        && record.request.device_turn_sequence == request.device_turn_sequence
}

#[allow(clippy::too_many_arguments)]
fn append_persistence_operation_journal_locked(
    state: &mut AdapterPersistenceState,
    now: MonotonicTimeNs,
    event_kind: AdapterOperationJournalEventKind,
    operation_id: &str,
    actor_user_id: &str,
    idempotency_key: &str,
    session_id: Option<String>,
    turn_id: Option<u64>,
    device_id: &str,
    device_turn_sequence: Option<u64>,
    acknowledgement_state: PersistenceAcknowledgementState,
    recovery_mode: PersistenceRecoveryMode,
    reconciliation_decision: Option<ReconciliationDecision>,
    conflict_severity: Option<PersistenceConflictSeverity>,
    detail: Option<String>,
    runtime_node_id: &str,
) {
    let sequence = state.next_journal_sequence;
    state.next_journal_sequence = state.next_journal_sequence.saturating_add(1);
    state.operation_journal.push(AdapterOperationJournalEntry {
        sequence,
        operation_id: operation_id.to_string(),
        request_id: state
            .outbox_records
            .get(operation_id)
            .map(|row| row.request_id.clone())
            .unwrap_or_else(|| "unknown_request".to_string()),
        trace_id: state
            .outbox_records
            .get(operation_id)
            .map(|row| row.trace_id.clone())
            .unwrap_or_else(|| "unknown_trace".to_string()),
        actor_user_id: actor_user_id.to_string(),
        idempotency_key: idempotency_key.to_string(),
        session_id,
        turn_id,
        device_id: device_id.to_string(),
        device_turn_sequence,
        at_ns: now.0,
        acknowledgement_state,
        recovery_mode,
        reconciliation_decision,
        conflict_severity,
        event_kind,
        detail,
        runtime_node_id: runtime_node_id.to_string(),
    });
}

#[allow(clippy::too_many_arguments)]
fn append_persistence_audit_locked(
    state: &mut AdapterPersistenceState,
    now: MonotonicTimeNs,
    decision: AdapterPersistenceAuditDecision,
    operation_id: Option<String>,
    actor_user_id: Option<String>,
    idempotency_key: Option<String>,
    session_id: Option<String>,
    turn_id: Option<u64>,
    _reconciliation_decision: Option<ReconciliationDecision>,
    conflict_severity: Option<PersistenceConflictSeverity>,
    runtime_node_id: &str,
    note: Option<String>,
) -> u64 {
    let sequence = state.next_audit_sequence;
    state.next_audit_sequence = state.next_audit_sequence.saturating_add(1);
    state.audit_trail.push(AdapterPersistenceAuditEntry {
        sequence,
        at_ns: now.0,
        operation_id,
        actor_user_id,
        idempotency_key,
        session_id,
        turn_id,
        decision,
        recovery_mode: state.recovery_mode,
        conflict_severity,
        runtime_node_id: runtime_node_id.to_string(),
        note,
    });
    sequence
}

fn set_persistence_recovery_mode_locked(
    state: &mut AdapterPersistenceState,
    now: MonotonicTimeNs,
    recovery_mode: PersistenceRecoveryMode,
    runtime_node_id: &str,
    note: Option<String>,
) {
    if state.recovery_mode == recovery_mode {
        return;
    }
    state.recovery_mode = recovery_mode;
    append_persistence_audit_locked(
        state,
        now,
        AdapterPersistenceAuditDecision::RecoveryModeTransition,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        runtime_node_id,
        note,
    );
}

fn sorted_pending_outbox_operation_ids(state: &AdapterPersistenceState) -> Vec<String> {
    let mut pending: Vec<_> = state
        .outbox_records
        .values()
        .filter(|row| {
            row.acknowledgement_state
                == PersistenceAcknowledgementState::PendingCloudAcknowledgement
                && row.cleared_from_active_outbox_at_ns.is_none()
        })
        .collect();
    pending.sort_by_key(|row| {
        (
            row.submission_timestamp_ns,
            row.device_turn_sequence.unwrap_or(0),
        )
    });
    pending
        .into_iter()
        .map(|row| row.operation_id.clone())
        .collect()
}

fn recompute_persistence_recovery_mode_locked(
    state: &AdapterPersistenceState,
) -> PersistenceRecoveryMode {
    if state.audit_trail.iter().any(|entry| {
        entry.decision == AdapterPersistenceAuditDecision::QuarantineAction
            && entry.operation_id.is_none()
    }) {
        PersistenceRecoveryMode::QuarantinedLocalState
    } else if state.outbox_records.values().any(|row| {
        row.acknowledgement_state == PersistenceAcknowledgementState::QuarantinedLocalState
    }) {
        PersistenceRecoveryMode::QuarantinedLocalState
    } else if state.outbox_records.values().any(|row| {
        row.acknowledgement_state == PersistenceAcknowledgementState::PendingCloudAcknowledgement
            && row.retry_counter > 0
    }) {
        PersistenceRecoveryMode::DegradedRecovery
    } else if state.outbox_records.values().any(|row| {
        row.acknowledgement_state == PersistenceAcknowledgementState::PendingCloudAcknowledgement
            && row.cleared_from_active_outbox_at_ns.is_none()
    }) {
        PersistenceRecoveryMode::Recovering
    } else {
        PersistenceRecoveryMode::Normal
    }
}

fn is_stale_conflict_reason(reason_code: &str) -> bool {
    let normalized = reason_code.to_ascii_uppercase();
    normalized.contains("STALE") || normalized.contains("DUPLICATE_DEVICE_TURN_SEQUENCE")
}

fn terminal_persistence_failure_state(
    error: &VoiceTurnIngressError,
) -> PersistenceAcknowledgementState {
    if is_stale_conflict_reason(&error.reason_code) {
        PersistenceAcknowledgementState::StaleRejected
    } else {
        PersistenceAcknowledgementState::AuthoritativelyAcknowledged
    }
}

fn terminal_persistence_failure_severity(
    error: &VoiceTurnIngressError,
) -> PersistenceConflictSeverity {
    if is_stale_conflict_reason(&error.reason_code) {
        PersistenceConflictSeverity::StaleRejected
    } else {
        PersistenceConflictSeverity::Info
    }
}

fn stale_replay_error_for_record(
    ingress: &AppServerIngressRuntime,
    state: &AdapterPersistenceState,
    record: &AdapterOutboxRecord,
    persistence_state: Option<PersistenceExecutionState>,
) -> Option<VoiceTurnIngressError> {
    let record_session_id = record.session_id.as_ref()?;
    let record_device_turn_sequence = record.device_turn_sequence?;
    let record_turn_id = record.turn_id.unwrap_or(0);
    let stale = state.authoritative_outcomes.values().any(|outcome| {
        outcome.session_id.as_ref() == Some(record_session_id)
            && outcome.device_id == record.device_id
            && (outcome
                .device_turn_sequence
                .map(|sequence| sequence > record_device_turn_sequence)
                .unwrap_or(false)
                || outcome
                    .turn_id
                    .map(|turn_id| turn_id > record_turn_id)
                    .unwrap_or(false))
    });
    if stale {
        let envelope = fallback_runtime_execution_envelope_for_voice_turn_request_with_identities(
            FallbackRuntimeExecutionEnvelopeParams {
                correlation_id: CorrelationId(u128::from(record.request.correlation_id)),
                turn_id: TurnId(record.request.turn_id),
                app_platform: parse_app_platform(&record.request.app_platform).ok()?,
                actor_user_id: &UserId::new(record.actor_user_id.clone()).ok()?,
                device_id: &DeviceId::new(record.device_id.clone()).ok()?,
                device_turn_sequence: record.device_turn_sequence,
                session_attach_outcome: None,
                platform_context: None,
            },
        )
        .ok()?
        .with_session_and_admission_state(
            record
                .session_id
                .as_deref()
                .and_then(|value| value.parse::<u128>().ok())
                .map(SessionId),
            AdmissionState::IngressValidated,
        )
        .ok()?
        .with_persistence_state(Some(persistence_state.unwrap_or_else(|| {
            build_persistence_execution_state(
                PersistenceRecoveryMode::Recovering,
                PersistenceAcknowledgementState::StaleRejected,
                Some(ReconciliationDecision::RejectStaleOperation),
                Some(PersistenceConflictSeverity::StaleRejected),
                false,
                None,
            )
            .expect("static stale-replay persistence state must validate")
        })))
        .ok()?;
        let decision = ingress.govern_persistence_signal(
            Some(&envelope),
            GovernanceProtectedActionClass::PersistenceReplay,
            "stale_replay_rejected",
            Some("stale replay rejected during persistence reconciliation".to_string()),
        );
        Some(voice_turn_ingress_error_from_governance_decision(&decision))
    } else {
        None
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
        "TABLET" => Ok(AppPlatform::Tablet),
        "DESKTOP" => Ok(AppPlatform::Desktop),
        _ => Err(format!(
            "invalid app_platform '{}'; expected IOS|ANDROID|TABLET|DESKTOP",
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

fn normalize_ascii_platform_token(
    field: &'static str,
    value: Option<&str>,
    fallback: &str,
    max_len: usize,
) -> Result<String, String> {
    let candidate = value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback);
    if candidate.len() > max_len {
        return Err(format!("{field} exceeds max length {max_len}"));
    }
    if !candidate.is_ascii() {
        return Err(format!("{field} must be ASCII"));
    }
    Ok(candidate.to_ascii_uppercase())
}

fn parse_device_class_literal(value: &str) -> Result<DeviceClass, String> {
    match value.trim().to_ascii_uppercase().as_str() {
        "PHONE" => Ok(DeviceClass::Phone),
        "TABLET" => Ok(DeviceClass::Tablet),
        "DESKTOP" => Ok(DeviceClass::Desktop),
        _ => Err(format!(
            "invalid device_class '{}'; expected PHONE|TABLET|DESKTOP",
            value
        )),
    }
}

fn parse_network_profile_literal(value: &str) -> Result<NetworkProfile, String> {
    match value.trim().to_ascii_uppercase().as_str() {
        "UNKNOWN" => Ok(NetworkProfile::Unknown),
        "OFFLINE" => Ok(NetworkProfile::Offline),
        "DEGRADED" => Ok(NetworkProfile::Degraded),
        "STANDARD" => Ok(NetworkProfile::Standard),
        "HIGH_THROUGHPUT" => Ok(NetworkProfile::HighThroughput),
        _ => Err(format!(
            "invalid network_profile '{}'; expected UNKNOWN|OFFLINE|DEGRADED|STANDARD|HIGH_THROUGHPUT",
            value
        )),
    }
}

fn parse_device_capability_literal(value: &str) -> Result<DeviceCapability, String> {
    match value.trim().to_ascii_uppercase().as_str() {
        "MICROPHONE" => Ok(DeviceCapability::Microphone),
        "CAMERA" => Ok(DeviceCapability::Camera),
        "SPEAKER_OUTPUT" => Ok(DeviceCapability::SpeakerOutput),
        "FILE_SYSTEM_ACCESS" => Ok(DeviceCapability::FileSystemAccess),
        "SENSOR_AVAILABILITY" => Ok(DeviceCapability::SensorAvailability),
        "HARDWARE_ACCELERATION" => Ok(DeviceCapability::HardwareAcceleration),
        "WAKE_WORD" => Ok(DeviceCapability::WakeWord),
        _ => Err(format!(
            "invalid claimed capability '{}'; expected canonical capability token",
            value
        )),
    }
}

fn parse_integrity_status_literal(value: &str) -> Result<ClientIntegrityStatus, String> {
    match value.trim().to_ascii_uppercase().as_str() {
        "UNKNOWN" => Ok(ClientIntegrityStatus::Unknown),
        "INTEGRITY_UNAVAILABLE" => Ok(ClientIntegrityStatus::IntegrityUnavailable),
        "INTEGRITY_VERIFIED" => Ok(ClientIntegrityStatus::IntegrityVerified),
        "ATTESTED" => Ok(ClientIntegrityStatus::Attested),
        "INTEGRITY_FAILED" => Ok(ClientIntegrityStatus::IntegrityFailed),
        _ => Err(format!(
            "invalid integrity_status '{}'; expected UNKNOWN|INTEGRITY_UNAVAILABLE|INTEGRITY_VERIFIED|ATTESTED|INTEGRITY_FAILED",
            value
        )),
    }
}

fn runtime_entry_trigger_from_os_trigger(trigger: OsVoiceTrigger) -> RuntimeEntryTrigger {
    match trigger {
        OsVoiceTrigger::Explicit => RuntimeEntryTrigger::Explicit,
        OsVoiceTrigger::WakeWord => RuntimeEntryTrigger::WakeWord,
    }
}

fn minimum_supported_client_version_for_platform(app_platform: AppPlatform) -> String {
    let env_key = match app_platform {
        AppPlatform::Ios => "SELENE_MIN_CLIENT_VERSION_IOS",
        AppPlatform::Android => "SELENE_MIN_CLIENT_VERSION_ANDROID",
        AppPlatform::Tablet => "SELENE_MIN_CLIENT_VERSION_TABLET",
        AppPlatform::Desktop => "SELENE_MIN_CLIENT_VERSION_DESKTOP",
    };
    env::var(env_key).unwrap_or_else(|_| "1.0.0".to_string())
}

fn parse_version_segments(version: &str) -> Result<Vec<u32>, String> {
    let raw = version.trim();
    if raw.is_empty() {
        return Err("version must not be empty".to_string());
    }
    if raw == "UNKNOWN" {
        return Ok(Vec::new());
    }
    let mut segments = Vec::new();
    for part in raw.split('.') {
        if part.is_empty() || !part.chars().all(|ch| ch.is_ascii_digit()) {
            return Err(format!("invalid version '{}'", version));
        }
        let value = part
            .parse::<u32>()
            .map_err(|_| format!("invalid version '{}'", version))?;
        segments.push(value);
    }
    Ok(segments)
}

fn compare_version_segments(left: &[u32], right: &[u32]) -> std::cmp::Ordering {
    let width = left.len().max(right.len());
    for idx in 0..width {
        let l = *left.get(idx).unwrap_or(&0);
        let r = *right.get(idx).unwrap_or(&0);
        match l.cmp(&r) {
            std::cmp::Ordering::Equal => continue,
            ordering => return ordering,
        }
    }
    std::cmp::Ordering::Equal
}

fn derive_device_trust_class(
    integrity_status: ClientIntegrityStatus,
    compatibility_status: ClientCompatibilityStatus,
) -> DeviceTrustClass {
    match (integrity_status, compatibility_status) {
        (ClientIntegrityStatus::IntegrityFailed, _) => DeviceTrustClass::UntrustedDevice,
        (_, ClientCompatibilityStatus::UnsupportedClient) => DeviceTrustClass::UntrustedDevice,
        (_, ClientCompatibilityStatus::UpgradeRequired) => DeviceTrustClass::RestrictedDevice,
        (ClientIntegrityStatus::Attested, ClientCompatibilityStatus::Compatible) => {
            DeviceTrustClass::TrustedDevice
        }
        _ => DeviceTrustClass::StandardDevice,
    }
}

fn normalize_claimed_capabilities(
    request: &VoiceTurnAdapterRequest,
    app_platform: AppPlatform,
) -> Result<(Vec<DeviceCapability>, Vec<DeviceCapability>), String> {
    let supported = supported_capabilities_for_platform(app_platform);
    let supported_set: std::collections::BTreeSet<DeviceCapability> =
        supported.iter().copied().collect();
    let mut claimed = std::collections::BTreeSet::new();
    if let Some(raw_capabilities) = request.claimed_capabilities.as_ref() {
        for raw in raw_capabilities {
            let capability = parse_device_capability_literal(raw)?;
            if !supported_set.contains(&capability) {
                return Err(format!(
                    "unsupported capability '{}' for platform {}",
                    raw,
                    app_platform.as_str()
                ));
            }
            claimed.insert(capability);
        }
    }
    let claimed_capabilities: Vec<DeviceCapability> = claimed.iter().copied().collect();
    let negotiated_capabilities = if claimed_capabilities.is_empty() {
        supported
    } else {
        claimed_capabilities.clone()
    };
    Ok((claimed_capabilities, negotiated_capabilities))
}

const CAPTURE_ARTIFACT_RETENTION_WINDOW_NS: u64 = 5_000_000_000;

fn derive_capture_artifact_posture(
    request: &VoiceTurnAdapterRequest,
    integrity_status: ClientIntegrityStatus,
) -> Result<(bool, Option<u64>, Option<u64>), String> {
    if integrity_status != ClientIntegrityStatus::Attested || request.attestation_ref.is_none() {
        return Ok((false, None, None));
    }
    let capture = request
        .audio_capture_ref
        .as_ref()
        .ok_or_else(|| "attested capture requires audio_capture_ref".to_string())?;
    let selected_mic_present = capture
        .selected_mic
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty());
    let device_route_present = capture
        .device_route
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty());
    let capture_degraded = capture.capture_degraded.unwrap_or(false);
    let stream_gap_detected = capture.stream_gap_detected.unwrap_or(false);
    let device_changed = capture.device_changed.unwrap_or(false);
    let observed_at_ns = capture.t_end_ns.max(capture.t_start_ns).max(1);
    let retention_deadline_ns = observed_at_ns.saturating_add(CAPTURE_ARTIFACT_RETENTION_WINDOW_NS);
    let trust_verified = selected_mic_present
        && device_route_present
        && !capture_degraded
        && !stream_gap_detected
        && !device_changed;
    Ok((
        trust_verified,
        Some(observed_at_ns),
        Some(retention_deadline_ns),
    ))
}

fn normalize_platform_runtime_context(
    request: &VoiceTurnAdapterRequest,
    app_platform: AppPlatform,
    trigger: OsVoiceTrigger,
) -> Result<PlatformRuntimeContext, String> {
    let requested_trigger = runtime_entry_trigger_from_os_trigger(trigger);
    let device_class = match request.device_class.as_deref() {
        Some(raw) => parse_device_class_literal(raw)?,
        None => default_device_class_for_platform(app_platform),
    };
    let platform_version = normalize_ascii_platform_token(
        "voice_turn_adapter_request.platform_version",
        request.platform_version.as_deref(),
        "UNKNOWN",
        64,
    )?;
    let runtime_client_version = normalize_ascii_platform_token(
        "voice_turn_adapter_request.runtime_client_version",
        request.runtime_client_version.as_deref(),
        "UNKNOWN",
        64,
    )?;
    let hardware_capability_profile = normalize_ascii_platform_token(
        "voice_turn_adapter_request.hardware_capability_profile",
        request.hardware_capability_profile.as_deref(),
        default_hardware_capability_profile(app_platform),
        64,
    )?;
    let network_profile = match request.network_profile.as_deref() {
        Some(raw) => parse_network_profile_literal(raw)?,
        None => NetworkProfile::Unknown,
    };
    let (claimed_capabilities, negotiated_capabilities) =
        normalize_claimed_capabilities(request, app_platform)?;
    let integrity_status = match request.integrity_status.as_deref() {
        Some(raw) => parse_integrity_status_literal(raw)?,
        None => ClientIntegrityStatus::Unknown,
    };
    let minimum_supported_client_version =
        minimum_supported_client_version_for_platform(app_platform);
    let compatibility_status = if runtime_client_version == "UNKNOWN" {
        ClientCompatibilityStatus::Unknown
    } else {
        let client_segments = parse_version_segments(&runtime_client_version)?;
        let minimum_segments = parse_version_segments(&minimum_supported_client_version)?;
        if compare_version_segments(&client_segments, &minimum_segments).is_lt() {
            ClientCompatibilityStatus::UpgradeRequired
        } else {
            ClientCompatibilityStatus::Compatible
        }
    };
    let device_trust_class = derive_device_trust_class(integrity_status, compatibility_status);
    let trigger_policy = match app_platform {
        AppPlatform::Ios => PlatformTriggerPolicy::ExplicitOnly,
        AppPlatform::Android | AppPlatform::Tablet | AppPlatform::Desktop => {
            PlatformTriggerPolicy::WakeOrExplicit
        }
    };
    let wake_capability_present = negotiated_capabilities.contains(&DeviceCapability::WakeWord);
    let trigger_allowed = match requested_trigger {
        RuntimeEntryTrigger::Explicit => true,
        RuntimeEntryTrigger::WakeWord => {
            trigger_policy == PlatformTriggerPolicy::WakeOrExplicit && wake_capability_present
        }
    };
    let (
        capture_artifact_trust_verified,
        capture_artifact_observed_at_ns,
        capture_artifact_retention_deadline_ns,
    ) = derive_capture_artifact_posture(request, integrity_status)?;
    let mut context = PlatformRuntimeContext::v1(
        app_platform,
        platform_version,
        device_class,
        runtime_client_version,
        hardware_capability_profile,
        network_profile,
        claimed_capabilities,
        negotiated_capabilities,
        device_trust_class,
        integrity_status,
        compatibility_status,
        Some(minimum_supported_client_version),
        request.attestation_ref.clone(),
        requested_trigger,
        trigger_policy,
        trigger_allowed,
    )
    .map_err(|err| format!("invalid platform_runtime_context: {err:?}"))?;
    context.capture_artifact_trust_verified = capture_artifact_trust_verified;
    context.capture_artifact_observed_at_ns = capture_artifact_observed_at_ns;
    context.capture_artifact_retention_deadline_ns = capture_artifact_retention_deadline_ns;
    context
        .validate()
        .map_err(|err| format!("invalid platform_runtime_context: {err:?}"))?;
    Ok(context)
}

fn session_id_to_string(session_id: SessionId) -> String {
    session_id.0.to_string()
}

fn adapter_response_session_id(
    response: &VoiceTurnAdapterResponse,
) -> Result<Option<SessionId>, String> {
    response
        .session_id
        .as_deref()
        .map(|value| {
            value
                .parse::<u128>()
                .map(SessionId)
                .map_err(|err| format!("invalid reused response session_id '{value}': {err}"))
        })
        .transpose()
}

fn session_state_to_api_value(session_state: SessionState) -> String {
    match session_state {
        SessionState::Closed => "CLOSED",
        SessionState::Open => "OPEN",
        SessionState::Active => "ACTIVE",
        SessionState::SoftClosed => "SOFT_CLOSED",
        SessionState::Suspended => "SUSPENDED",
    }
    .to_string()
}

fn session_attach_outcome_to_api_value(session_attach_outcome: SessionAttachOutcome) -> String {
    session_attach_outcome.as_str().to_string()
}

fn canonical_reason_code_token(reason: &str, failure_class: FailureClass) -> String {
    if reason.starts_with("auth_identity_unknown") {
        return "AUTH_IDENTITY_UNKNOWN".to_string();
    }
    if reason.starts_with("auth_device_unknown") {
        return "AUTH_DEVICE_UNKNOWN".to_string();
    }
    if reason.starts_with("wake_rejected") {
        return "WAKE_REJECTED".to_string();
    }
    let mut token = String::with_capacity(reason.len());
    let mut prev_underscore = false;
    for ch in reason.chars() {
        let normalized = if ch.is_ascii_alphanumeric() {
            prev_underscore = false;
            ch.to_ascii_uppercase()
        } else {
            if prev_underscore {
                continue;
            }
            prev_underscore = true;
            '_'
        };
        token.push(normalized);
        if token.len() >= 64 {
            break;
        }
    }
    let token = token.trim_matches('_').to_string();
    if token.is_empty() {
        failure_class.as_str().to_string()
    } else {
        token
    }
}

fn voice_turn_ingress_error(
    failure_class: FailureClass,
    reason_code: String,
    reason: Option<String>,
    session_id: Option<String>,
    turn_id: Option<u64>,
    session_state: Option<SessionState>,
) -> VoiceTurnIngressError {
    VoiceTurnIngressError {
        failure_class,
        reason_code,
        reason,
        session_id,
        turn_id,
        session_state: session_state.map(session_state_to_api_value),
    }
}

fn voice_turn_ingress_error_from_governance_decision(
    decision: &RuntimeGovernanceDecision,
) -> VoiceTurnIngressError {
    voice_turn_ingress_error(
        governance_failure_class_for_response(decision.response_class),
        decision.reason_code.clone(),
        Some(governance_runtime_reason(decision)),
        decision.session_id.map(|session_id| session_id.to_string()),
        decision.turn_id,
        governance_reason_to_session_state(decision.response_class),
    )
}

fn classify_voice_turn_runtime_error(
    reason: &str,
    turn_id: Option<u64>,
    session_id: Option<String>,
    session_state: Option<SessionState>,
) -> VoiceTurnIngressError {
    let failure_class = if reason.starts_with("auth_identity_unknown") {
        FailureClass::AuthenticationFailure
    } else if reason.starts_with("auth_device_unknown") {
        FailureClass::AuthorizationFailure
    } else if reason.contains("governance_safe_mode") {
        FailureClass::RetryableRuntime
    } else if reason.contains("governance_policy_block") {
        FailureClass::PolicyViolation
    } else if reason.contains("governance_degrade") {
        FailureClass::RetryableRuntime
    } else if reason.starts_with("wake_rejected") || reason == "ios_wake_disabled" {
        FailureClass::PolicyViolation
    } else if reason.contains("replayed_request") {
        FailureClass::ReplayRejected
    } else if reason.contains("retry_result_missing") {
        FailureClass::RetryableRuntime
    } else if reason.contains("session_conflict") {
        FailureClass::SessionConflict
    } else if reason.contains("lock poisoned") {
        FailureClass::RetryableRuntime
    } else if reason.starts_with("invalid ")
        || reason.starts_with("missing_")
        || reason.starts_with("stale_request")
        || reason.starts_with("timestamp_in_future")
        || reason.contains("expected ")
    {
        FailureClass::InvalidPayload
    } else {
        FailureClass::ExecutionFailure
    };
    voice_turn_ingress_error(
        failure_class,
        canonical_reason_code_token(reason, failure_class),
        Some(reason.to_string()),
        session_id,
        turn_id,
        session_state,
    )
}

fn fallback_runtime_execution_envelope_for_voice_turn_request(
    request: &VoiceTurnAdapterRequest,
) -> Result<RuntimeExecutionEnvelope, ContractViolation> {
    let app_platform =
        parse_app_platform(&request.app_platform).map_err(|_| ContractViolation::InvalidValue {
            field: "voice_turn_adapter_request.app_platform",
            reason: "must be IOS|ANDROID|TABLET|DESKTOP",
        })?;
    let trigger = parse_trigger(&request.trigger).map_err(|_| ContractViolation::InvalidValue {
        field: "voice_turn_adapter_request.trigger",
        reason: "must be EXPLICIT|WAKE_WORD",
    })?;
    let actor_user_id = UserId::new(request.actor_user_id.clone())?;
    let device_id = match request.device_id.as_ref() {
        Some(device_id) => DeviceId::new(device_id.clone())?,
        None => DeviceId::new(format!(
            "adapter_auto_{}",
            stable_hash_hex_16(actor_user_id.as_str())
        ))?,
    };
    let platform_context = normalize_platform_runtime_context(request, app_platform, trigger)
        .map_err(|_| ContractViolation::InvalidValue {
            field: "voice_turn_adapter_request.platform_context",
            reason: "invalid normalized platform context",
        })?;
    fallback_runtime_execution_envelope_for_voice_turn_request_with_identities(
        FallbackRuntimeExecutionEnvelopeParams {
            correlation_id: CorrelationId(request.correlation_id.into()),
            turn_id: TurnId(request.turn_id),
            app_platform,
            actor_user_id: &actor_user_id,
            device_id: &device_id,
            device_turn_sequence: request
                .device_turn_sequence
                .or(Some(request.turn_id.max(1))),
            session_attach_outcome: None,
            platform_context: Some(platform_context),
        },
    )
}

pub fn build_runtime_execution_envelope_for_voice_turn_request(
    request: &VoiceTurnAdapterRequest,
    request_id: &str,
    idempotency_key: &str,
    device_id: &str,
) -> Result<RuntimeExecutionEnvelope, String> {
    let app_platform = parse_app_platform(&request.app_platform)?;
    let trigger = parse_trigger(&request.trigger)?;
    let actor_identity = UserId::new(request.actor_user_id.clone())
        .map_err(|err| format!("invalid actor_user_id: {err:?}"))?;
    let device_identity = DeviceId::new(device_id.to_string())
        .map_err(|err| format!("invalid device_id: {err:?}"))?;
    let turn_id = TurnId(request.turn_id);
    let platform_context = normalize_platform_runtime_context(request, app_platform, trigger)?;
    RuntimeExecutionEnvelope::v1_with_platform_context_device_turn_sequence_and_attach_outcome(
        request_id.to_string(),
        format!("trace:voice-turn:{}:{}", request_id, request.turn_id),
        idempotency_key.to_string(),
        actor_identity,
        device_identity,
        app_platform,
        platform_context,
        None,
        turn_id,
        request
            .device_turn_sequence
            .or(Some(request.turn_id.max(1))),
        AdmissionState::IngressValidated,
        None,
    )
    .map_err(|err| format!("invalid runtime_execution_envelope: {err:?}"))
}

fn fallback_runtime_execution_envelope_for_voice_turn_request_with_identities(
    params: FallbackRuntimeExecutionEnvelopeParams<'_>,
) -> Result<RuntimeExecutionEnvelope, ContractViolation> {
    let FallbackRuntimeExecutionEnvelopeParams {
        correlation_id,
        turn_id,
        app_platform,
        actor_user_id,
        device_id,
        device_turn_sequence,
        session_attach_outcome,
        platform_context,
    } = params;
    RuntimeExecutionEnvelope::v1_with_platform_context_device_turn_sequence_and_attach_outcome(
        format!("corr-{}", correlation_id.0),
        format!("trace:voice:{}:{}", correlation_id.0, turn_id.0),
        format!("turn:{}:{}", correlation_id.0, turn_id.0),
        actor_user_id.clone(),
        device_id.clone(),
        app_platform,
        platform_context.unwrap_or(PlatformRuntimeContext::default_for_platform(app_platform)?),
        None,
        turn_id,
        device_turn_sequence,
        AdmissionState::IngressValidated,
        session_attach_outcome,
    )
}

fn onboarding_next_step_to_api_value(next_step: OnboardingNextStep) -> String {
    match next_step {
        OnboardingNextStep::Install => "INSTALL",
        OnboardingNextStep::Terms => "TERMS",
        OnboardingNextStep::LoadPrefilled => "LOAD_PREFILLED",
        OnboardingNextStep::AskMissing => "ASK_MISSING",
    }
    .to_string()
}

fn parse_onboarding_continue_action(
    action: &str,
    input: OnboardingContinueActionInput,
) -> Result<AppOnboardingContinueAction, String> {
    let OnboardingContinueActionInput {
        field_value,
        receipt_kind,
        receipt_ref,
        signer,
        payload_hash,
        terms_version_id,
        accepted,
        device_id,
        proof_ok,
        sample_seed,
        photo_blob_ref,
        sender_decision,
    } = input;
    let normalized = action.trim().to_ascii_uppercase();
    match normalized.as_str() {
        "ASK_MISSING_SUBMIT" => Ok(AppOnboardingContinueAction::AskMissingSubmit { field_value }),
        "PLATFORM_SETUP_RECEIPT" => {
            let receipt_kind = receipt_kind
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| "receipt_kind is required for PLATFORM_SETUP_RECEIPT".to_string())?;
            let receipt_ref = receipt_ref
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| "receipt_ref is required for PLATFORM_SETUP_RECEIPT".to_string())?;
            let signer = signer
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| "signer is required for PLATFORM_SETUP_RECEIPT".to_string())?;
            let payload_hash = payload_hash
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| "payload_hash is required for PLATFORM_SETUP_RECEIPT".to_string())?;
            Ok(AppOnboardingContinueAction::PlatformSetupReceipt {
                receipt_kind,
                receipt_ref,
                signer,
                payload_hash,
            })
        }
        "TERMS_ACCEPT" => {
            let terms_version_id = terms_version_id
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| "terms_version_id is required for TERMS_ACCEPT".to_string())?;
            Ok(AppOnboardingContinueAction::TermsAccept {
                terms_version_id,
                accepted: accepted.unwrap_or(true),
            })
        }
        "PRIMARY_DEVICE_CONFIRM" => {
            let device_id = device_id
                .ok_or_else(|| "device_id is required for PRIMARY_DEVICE_CONFIRM".to_string())?;
            let device_id = DeviceId::new(device_id)
                .map_err(|err| format!("invalid device_id for PRIMARY_DEVICE_CONFIRM: {err:?}"))?;
            Ok(AppOnboardingContinueAction::PrimaryDeviceConfirm {
                device_id,
                proof_ok: proof_ok.unwrap_or(true),
            })
        }
        "VOICE_ENROLL_LOCK" => {
            let device_id =
                device_id.ok_or_else(|| "device_id is required for VOICE_ENROLL_LOCK".to_string())?;
            let device_id = DeviceId::new(device_id)
                .map_err(|err| format!("invalid device_id for VOICE_ENROLL_LOCK: {err:?}"))?;
            let sample_seed = sample_seed
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| "sample_seed is required for VOICE_ENROLL_LOCK".to_string())?;
            Ok(AppOnboardingContinueAction::VoiceEnrollLock {
                device_id,
                sample_seed,
            })
        }
        "WAKE_ENROLL_START_DRAFT" => {
            let device_id = device_id
                .ok_or_else(|| "device_id is required for WAKE_ENROLL_START_DRAFT".to_string())?;
            let device_id = DeviceId::new(device_id).map_err(|err| {
                format!("invalid device_id for WAKE_ENROLL_START_DRAFT: {err:?}")
            })?;
            Ok(AppOnboardingContinueAction::WakeEnrollStartDraft { device_id })
        }
        "WAKE_ENROLL_SAMPLE_COMMIT" => {
            let device_id = device_id
                .ok_or_else(|| "device_id is required for WAKE_ENROLL_SAMPLE_COMMIT".to_string())?;
            let device_id = DeviceId::new(device_id).map_err(|err| {
                format!("invalid device_id for WAKE_ENROLL_SAMPLE_COMMIT: {err:?}")
            })?;
            Ok(AppOnboardingContinueAction::WakeEnrollSampleCommit {
                device_id,
                sample_pass: proof_ok.unwrap_or(true),
            })
        }
        "WAKE_ENROLL_COMPLETE_COMMIT" => {
            let device_id = device_id.ok_or_else(|| {
                "device_id is required for WAKE_ENROLL_COMPLETE_COMMIT".to_string()
            })?;
            let device_id = DeviceId::new(device_id).map_err(|err| {
                format!("invalid device_id for WAKE_ENROLL_COMPLETE_COMMIT: {err:?}")
            })?;
            Ok(AppOnboardingContinueAction::WakeEnrollCompleteCommit { device_id })
        }
        "WAKE_ENROLL_DEFER_COMMIT" => {
            let device_id = device_id
                .ok_or_else(|| "device_id is required for WAKE_ENROLL_DEFER_COMMIT".to_string())?;
            let device_id = DeviceId::new(device_id)
                .map_err(|err| format!("invalid device_id for WAKE_ENROLL_DEFER_COMMIT: {err:?}"))?;
            Ok(AppOnboardingContinueAction::WakeEnrollDeferCommit { device_id })
        }
        "EMPLOYEE_PHOTO_CAPTURE_SEND" => {
            let photo_blob_ref = photo_blob_ref
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| {
                    "photo_blob_ref is required for EMPLOYEE_PHOTO_CAPTURE_SEND".to_string()
                })?;
            Ok(AppOnboardingContinueAction::EmployeePhotoCaptureSend { photo_blob_ref })
        }
        "EMPLOYEE_SENDER_VERIFY_COMMIT" => {
            let decision = sender_decision
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| {
                    "sender_decision is required for EMPLOYEE_SENDER_VERIFY_COMMIT".to_string()
                })?;
            let normalized_decision = decision.trim().to_ascii_uppercase();
            let decision = match normalized_decision.as_str() {
                "CONFIRM" => SenderVerifyDecision::Confirm,
                "REJECT" => SenderVerifyDecision::Reject,
                _ => {
                    return Err(
                        "sender_decision must be CONFIRM or REJECT for EMPLOYEE_SENDER_VERIFY_COMMIT"
                            .to_string(),
                    );
                }
            };
            Ok(AppOnboardingContinueAction::EmployeeSenderVerifyCommit { decision })
        }
        "EMO_PERSONA_LOCK" => Ok(AppOnboardingContinueAction::EmoPersonaLock),
        "ACCESS_PROVISION_COMMIT" => Ok(AppOnboardingContinueAction::AccessProvisionCommit),
        "COMPLETE_COMMIT" => Ok(AppOnboardingContinueAction::CompleteCommit),
        _ => Err(format!(
            "invalid action '{}'; expected ASK_MISSING_SUBMIT|PLATFORM_SETUP_RECEIPT|TERMS_ACCEPT|PRIMARY_DEVICE_CONFIRM|VOICE_ENROLL_LOCK|WAKE_ENROLL_START_DRAFT|WAKE_ENROLL_SAMPLE_COMMIT|WAKE_ENROLL_COMPLETE_COMMIT|WAKE_ENROLL_DEFER_COMMIT|EMPLOYEE_PHOTO_CAPTURE_SEND|EMPLOYEE_SENDER_VERIFY_COMMIT|EMO_PERSONA_LOCK|ACCESS_PROVISION_COMMIT|COMPLETE_COMMIT",
            action
        )),
    }
}

fn onboarding_continue_next_step_to_api_value(next_step: AppOnboardingContinueNextStep) -> String {
    match next_step {
        AppOnboardingContinueNextStep::AskMissing => "ASK_MISSING",
        AppOnboardingContinueNextStep::PlatformSetup => "PLATFORM_SETUP",
        AppOnboardingContinueNextStep::Terms => "TERMS",
        AppOnboardingContinueNextStep::PrimaryDeviceConfirm => "PRIMARY_DEVICE_CONFIRM",
        AppOnboardingContinueNextStep::VoiceEnroll => "VOICE_ENROLL",
        AppOnboardingContinueNextStep::WakeEnroll => "WAKE_ENROLL",
        AppOnboardingContinueNextStep::SenderVerification => "SENDER_VERIFICATION",
        AppOnboardingContinueNextStep::EmoPersonaLock => "EMO_PERSONA_LOCK",
        AppOnboardingContinueNextStep::AccessProvision => "ACCESS_PROVISION",
        AppOnboardingContinueNextStep::Complete => "COMPLETE",
        AppOnboardingContinueNextStep::Ready => "READY",
        AppOnboardingContinueNextStep::Blocked => "BLOCKED",
    }
    .to_string()
}

fn build_base_nlp_request_for_vision_handoff(
    request: &VoiceTurnAdapterRequest,
    base_transcript_text: Option<&str>,
    runtime_tenant_scope: Option<&str>,
) -> Result<Ph1nRequest, String> {
    let transcript_text = sanitize_transcript_text_option(
        base_transcript_text
            .map(str::to_string)
            .or_else(|| request.user_text_final.clone()),
    )
    .unwrap_or_else(|| "analyze the uploaded visual evidence".to_string());
    let locale_guess = request
        .audio_capture_ref
        .as_ref()
        .and_then(|capture| capture.locale_tag.as_ref())
        .map(|value| truncate_ascii(value.trim(), 16))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "en".to_string());
    let language_tag = LanguageTag::new(locale_guess)
        .or_else(|_| LanguageTag::new("en".to_string()))
        .map_err(|err| format!("invalid language tag for vision handoff: {err:?}"))?;
    let transcript_ok =
        Ph1cTranscriptOk::v1(transcript_text, language_tag, Ph1cConfidenceBucket::High)
            .map_err(|err| format!("failed to build transcript for vision handoff: {err:?}"))?;
    let runtime_tenant_id = runtime_tenant_scope
        .map(|tenant| truncate_ascii(tenant.trim(), 64))
        .filter(|tenant| !tenant.is_empty());
    Ph1nRequest::v1(
        transcript_ok,
        Ph1cSessionStateRef::v1(SessionState::Active, false),
    )
    .map_err(|err| format!("failed to build NLP request for vision handoff: {err:?}"))?
    .with_runtime_tenant_id(runtime_tenant_id)
    .map_err(|err| format!("failed to set runtime tenant context for NLP request: {err:?}"))
}

fn build_nlp_output_for_voice_turn(
    request: &VoiceTurnAdapterRequest,
    transcript_text: Option<&str>,
    runtime_tenant_scope: Option<&str>,
) -> Result<Ph1nResponse, String> {
    let nlp_request =
        build_base_nlp_request_for_vision_handoff(request, transcript_text, runtime_tenant_scope)?;
    AdapterNlpEngineRuntime::new()
        .run(&nlp_request)
        .map_err(|err| format!("ph1n runtime failed while building PH1.X input: {err:?}"))
}

fn build_vision_turn_input_from_adapter_request(
    request: &VoiceTurnAdapterRequest,
    correlation_id: CorrelationId,
    turn_id: TurnId,
) -> Result<Option<VisionTurnInput>, String> {
    let Some(visual) = request.visual_input_ref.as_ref() else {
        return Ok(None);
    };
    if !visual.turn_opt_in_enabled {
        return Ok(None);
    }
    let source_kind = parse_visual_source_kind(visual.source_kind.as_deref())?;
    let source_id = visual
        .source_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| truncate_ascii(value, 128))
        .unwrap_or_else(|| {
            let seed = format!(
                "{}:{}:{}:{}",
                request.correlation_id,
                request.turn_id,
                visual.image_ref.as_deref().unwrap_or(""),
                visual.blob_ref.as_deref().unwrap_or("")
            );
            format!("vision_src_{}", stable_hash_hex_16(&seed))
        });
    let source_ref = VisualSourceRef::v1(
        VisualSourceId::new(source_id)
            .map_err(|err| format!("invalid PH1.VISION source_id: {err:?}"))?,
        source_kind,
    )
    .map_err(|err| format!("invalid PH1.VISION source_ref: {err:?}"))?;
    let mut visible_tokens = Vec::with_capacity(visual.visible_tokens.len());
    for token_ref in &visual.visible_tokens {
        visible_tokens.push(parse_visual_token_ref(token_ref)?);
    }
    let turn_input = VisionTurnInput::v1(correlation_id, turn_id, source_ref, visible_tokens)
        .map_err(|err| format!("invalid PH1.VISION turn input: {err:?}"))?;
    Ok(Some(turn_input))
}

fn parse_visual_source_kind(value: Option<&str>) -> Result<VisualSourceKind, String> {
    let normalized = value
        .map(|raw| raw.trim().to_ascii_uppercase())
        .filter(|raw| !raw.is_empty())
        .unwrap_or_else(|| "IMAGE".to_string());
    match normalized.as_str() {
        "IMAGE" => Ok(VisualSourceKind::Image),
        "SCREENSHOT" => Ok(VisualSourceKind::Screenshot),
        "DIAGRAM" => Ok(VisualSourceKind::Diagram),
        _ => Err(format!(
            "invalid visual source_kind '{}'; expected IMAGE|SCREENSHOT|DIAGRAM",
            normalized
        )),
    }
}

fn parse_visual_token_ref(token_ref: &VoiceTurnVisualTokenRef) -> Result<VisualToken, String> {
    let token = truncate_utf8(token_ref.token.trim(), 256);
    let bbox = match (token_ref.x, token_ref.y, token_ref.w, token_ref.h) {
        (Some(x), Some(y), Some(w), Some(h)) => Some(
            BoundingBoxPx::new(x, y, w, h)
                .map_err(|err| format!("invalid PH1.VISION visual token bbox: {err:?}"))?,
        ),
        (None, None, None, None) => None,
        _ => {
            return Err(
                "invalid PH1.VISION visual token bbox: x,y,w,h must be provided together"
                    .to_string(),
            )
        }
    };
    VisualToken::v1(token, bbox)
        .map_err(|err| format!("invalid PH1.VISION visual token payload: {err:?}"))
}

fn ensure_actor_identity_and_device(
    store: &mut Ph1fStore,
    actor_user_id: &UserId,
    device_id: Option<&DeviceId>,
    app_platform: AppPlatform,
    now: MonotonicTimeNs,
    allow_auto_provision: bool,
) -> Result<(), String> {
    if store.get_identity(actor_user_id).is_none() {
        if !allow_auto_provision {
            return Err("auth_identity_unknown".to_string());
        }
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
            if !allow_auto_provision {
                return Err("auth_device_unknown".to_string());
            }
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
        AppPlatform::Tablet => "tablet",
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

fn voice_outcome_reason(outcome: &OsVoiceLiveTurnOutcome) -> Option<String> {
    match outcome {
        OsVoiceLiveTurnOutcome::NotInvokedDisabled => None,
        OsVoiceLiveTurnOutcome::Refused(refuse) => Some(format!(
            "os_refuse reason_code={} message={}",
            refuse.reason_code.0, refuse.message
        )),
        OsVoiceLiveTurnOutcome::Forwarded(bundle) => match &bundle.voice_identity_assertion {
            Ph1VoiceIdResponse::SpeakerAssertionOk(ok) => Some(format!(
                "voice_identity=OK score_bp={} user_id={}",
                ok.score_bp,
                ok.user_id.as_ref().map(UserId::as_str).unwrap_or("none")
            )),
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(u) => Some(format!(
                "voice_identity=UNKNOWN reason_code={} score_bp={}",
                u.reason_code.0, u.score_bp
            )),
        },
    }
}

fn next_move_label(execution: &AppVoiceTurnExecutionOutcome) -> &'static str {
    if execution.dispatch_outcome.is_some() {
        return "dispatch_sim";
    }
    if execution.tool_response.is_some() {
        return "dispatch_tool";
    }
    match execution.next_move {
        AppVoiceTurnNextMove::Confirm | AppVoiceTurnNextMove::Clarify => "clarify",
        AppVoiceTurnNextMove::Dispatch => "dispatch_sim",
        AppVoiceTurnNextMove::NotInvokedDisabled
        | AppVoiceTurnNextMove::Refused
        | AppVoiceTurnNextMove::Respond
        | AppVoiceTurnNextMove::Wait => "respond",
    }
}

fn outcome_label(execution: &AppVoiceTurnExecutionOutcome) -> &'static str {
    if execution.dispatch_outcome.is_some() {
        return "DISPATCH_SIM";
    }
    if execution.tool_response.is_some() {
        return "FINAL_TOOL";
    }
    "FINAL"
}

fn detect_read_only_turn_incidents(
    user_text_final: Option<&str>,
    execution: &AppVoiceTurnExecutionOutcome,
) -> Vec<ReadOnlyIncidentRecord> {
    if execution.dispatch_outcome.is_some() {
        return Vec::new();
    }

    let mut incidents = Vec::new();
    if let Some(tool_response) = execution.tool_response.as_ref() {
        if tool_response.tool_status == ToolStatus::Fail {
            let reason_code = execution
                .reason_code
                .or(tool_response.fail_reason_code)
                .unwrap_or(reason_codes::ADAPTER_READ_ONLY_TOOL_FAIL_INCIDENT);
            incidents.push(ReadOnlyIncidentRecord {
                kind: ReadOnlyIncidentKind::ToolFail,
                reason_code,
                evidence_ref: truncate_ascii(
                    &format!(
                        "tool_fail:query_hash:{}:cache:{}",
                        tool_response.query_hash.0,
                        cache_status_label(tool_response.cache_status)
                    ),
                    128,
                ),
                provenance_ref: truncate_ascii(
                    &format!("ph1e_tool_fail:reason_code:{}", reason_code.0),
                    128,
                ),
            });
        }
    }

    if let Some(ph1x_response) = execution.ph1x_response.as_ref() {
        if matches!(&ph1x_response.directive, Ph1xDirective::Clarify(_)) {
            if let Some(PendingState::Clarify {
                missing_field,
                attempts,
            }) = ph1x_response.thread_state.pending.as_ref()
            {
                if *attempts >= 2 {
                    let reason_code = execution
                        .reason_code
                        .unwrap_or(reason_codes::ADAPTER_READ_ONLY_CLARIFY_LOOP_INCIDENT);
                    incidents.push(ReadOnlyIncidentRecord {
                        kind: ReadOnlyIncidentKind::ClarifyLoop,
                        reason_code,
                        evidence_ref: truncate_ascii(
                            &format!(
                                "clarify_loop:attempts:{}:field:{:?}",
                                attempts, missing_field
                            ),
                            128,
                        ),
                        provenance_ref: truncate_ascii(
                            &format!("ph1x_clarify_loop:attempts:{attempts}"),
                            128,
                        ),
                    });
                }
            }
        }
    }

    if let Some(text) = user_text_final {
        if user_text_looks_like_correction(text) {
            incidents.push(ReadOnlyIncidentRecord {
                kind: ReadOnlyIncidentKind::UserCorrection,
                reason_code: reason_codes::ADAPTER_READ_ONLY_USER_CORRECTION_INCIDENT,
                evidence_ref: truncate_ascii(text.trim(), 128),
                provenance_ref: "user_text:correction_phrase".to_string(),
            });
        }
    }

    incidents
}

fn cache_status_label(cache_status: CacheStatus) -> &'static str {
    match cache_status {
        CacheStatus::Hit => "hit",
        CacheStatus::Miss => "miss",
        CacheStatus::Bypassed => "bypassed",
    }
}

fn provenance_from_tool_response(tool_response: &ToolResponse) -> VoiceTurnProvenance {
    let (sources, retrieved_at) = match tool_response.source_metadata.as_ref() {
        Some(meta) => (
            meta.sources
                .iter()
                .map(|src| VoiceTurnProvenanceSource {
                    title: src.title.clone(),
                    url: src.url.clone(),
                })
                .collect(),
            meta.retrieved_at_unix_ms,
        ),
        None => (Vec::new(), 0),
    };
    VoiceTurnProvenance {
        sources,
        retrieved_at,
        cache_status: cache_status_label(tool_response.cache_status).to_string(),
    }
}

fn execution_outcome_to_adapter_response(
    execution: AppVoiceTurnExecutionOutcome,
) -> VoiceTurnAdapterResponse {
    VoiceTurnAdapterResponse {
        status: "ok".to_string(),
        outcome: outcome_label(&execution).to_string(),
        session_id: execution
            .runtime_execution_envelope
            .session_id
            .map(session_id_to_string),
        turn_id: Some(execution.runtime_execution_envelope.turn_id.0),
        session_state: Some(session_state_to_api_value(execution.session_state)),
        session_attach_outcome: execution.runtime_execution_envelope.session_attach_outcome,
        failure_class: None,
        reason: voice_outcome_reason(&execution.voice_outcome),
        next_move: next_move_label(&execution).to_string(),
        response_text: execution.response_text.unwrap_or_default(),
        reason_code: execution
            .reason_code
            .map(|code| code.0.to_string())
            .unwrap_or_else(|| "0".to_string()),
        provenance: execution
            .tool_response
            .as_ref()
            .map(provenance_from_tool_response),
    }
}

fn sanitize_transcript_text_option(value: Option<String>) -> Option<String> {
    value
        .map(|v| truncate_ascii(v.trim(), 8192))
        .filter(|v| !v.trim().is_empty())
}

fn user_text_looks_like_correction(text: &str) -> bool {
    let normalized = text.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return false;
    }
    const PREFIXES: [&str; 8] = [
        "no ",
        "no,",
        "actually",
        "i meant",
        "sorry, i meant",
        "let me correct",
        "correction:",
        "that's not",
    ];
    if PREFIXES.iter().any(|prefix| normalized.starts_with(prefix)) {
        return true;
    }
    normalized.contains(" i meant ")
        || normalized.contains(" correction ")
        || normalized.contains(" not that")
}

fn infer_confirm_answer_from_user_text(
    thread_state: &ThreadState,
    user_text_final: Option<&str>,
) -> Option<ConfirmAnswer> {
    let awaiting_confirm = thread_state.return_check_pending
        || matches!(
            thread_state.pending.as_ref(),
            Some(PendingState::Confirm { .. } | PendingState::MemoryPermission { .. })
        );
    if !awaiting_confirm {
        return None;
    }

    let normalized = user_text_final?.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return None;
    }

    const YES_EXACT: [&str; 8] = [
        "yes", "y", "yeah", "yep", "confirm", "correct", "ok", "okay",
    ];
    const NO_EXACT: [&str; 7] = ["no", "n", "nope", "nah", "cancel", "stop", "don't"];
    const YES_PREFIXES: [&str; 4] = ["yes,", "yes.", "confirm,", "confirm."];
    const NO_PREFIXES: [&str; 3] = ["no,", "no.", "cancel,"];

    if YES_EXACT.contains(&normalized.as_str())
        || YES_PREFIXES
            .iter()
            .any(|prefix| normalized.starts_with(prefix))
    {
        return Some(ConfirmAnswer::Yes);
    }
    if NO_EXACT.contains(&normalized.as_str())
        || NO_PREFIXES
            .iter()
            .any(|prefix| normalized.starts_with(prefix))
    {
        return Some(ConfirmAnswer::No);
    }
    None
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
    session_id: Option<SessionId>,
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
        session_id,
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

fn resolve_adapter_thread_key(value: Option<&str>) -> String {
    let raw = value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or("default");
    let mut out = String::with_capacity(raw.len());
    for c in raw.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.') {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "default".to_string()
    } else {
        truncate_ascii(&out, 96)
    }
}

fn resolve_adapter_project_id(value: Option<&str>) -> Option<String> {
    let raw = value.map(str::trim).filter(|v| !v.is_empty())?;
    let mut out = String::with_capacity(raw.len());
    for c in raw.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | ':' | '/') {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(truncate_ascii(&out, 96))
    }
}

fn resolve_adapter_pinned_context_refs(values: Option<&[String]>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let Some(values) = values else {
        return out;
    };
    for value in values {
        let raw = value.trim();
        if raw.is_empty() {
            continue;
        }
        let mut normalized = String::with_capacity(raw.len());
        for c in raw.chars() {
            if c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | ':' | '/') {
                normalized.push(c);
            } else {
                normalized.push('_');
            }
        }
        let normalized = truncate_ascii(&normalized, 128);
        if normalized.is_empty() || out.iter().any(|existing| existing == &normalized) {
            continue;
        }
        out.push(normalized);
        if out.len() >= 16 {
            break;
        }
    }
    out
}

#[derive(Debug, Clone)]
struct AdapterSessionTurnState {
    session_snapshot: SessionSnapshot,
    session_id_for_commits: Option<SessionId>,
    wake_event: Option<WakeDecision>,
    session_attach_outcome: SessionAttachOutcome,
    device_turn_sequence: u64,
}

#[derive(Debug, Clone)]
enum AdapterSessionResolution {
    Proceed(AdapterSessionTurnState),
    Retry(VoiceTurnAdapterResponse),
}

#[derive(Debug, Clone)]
struct CanonicalActorSessionSelection {
    latest: Option<SessionRecord>,
    latest_recoverable: Option<SessionRecord>,
}

#[derive(Debug, Clone)]
struct WakeEvaluation {
    decision: WakeDecision,
    wake_profile_id: Option<String>,
    threshold_used_bp: u16,
    model_version: String,
    window_start_ns: MonotonicTimeNs,
    window_end_ns: MonotonicTimeNs,
}

fn confidence_to_basis_points(value: Option<Confidence>) -> Option<u16> {
    value.map(|score| ((score.0.clamp(0.0, 1.0) * 10_000.0).round() as u16).min(10_000))
}

fn build_ph1w_live_step_input(
    now: MonotonicTimeNs,
    ph1k: &Ph1kLiveSignalBundle,
    wake_trigger_alignment_hint: bool,
) -> Result<WakeStepInput, String> {
    let pre_roll_start = MonotonicTimeNs(now.0.saturating_sub(500_000_000));
    let pre_roll_end = MonotonicTimeNs(now.0.saturating_add(500_000_000));
    let pre_roll = PreRollBufferRef::v1(
        ph1k.pre_roll_buffer_ref.buffer_id,
        ph1k.processed_stream_ref.stream_id,
        pre_roll_start,
        pre_roll_end,
    );
    let vad_event = VadEvent::v1(
        ph1k.processed_stream_ref.stream_id,
        pre_roll_start,
        MonotonicTimeNs(pre_roll_start.0.saturating_add(200_000_000)),
        Confidence::new(ph1k.interrupt_input.vad_confidence.clamp(0.0, 1.0))
            .map_err(|err| format!("invalid PH1.W VAD confidence: {err:?}"))?,
        SpeechLikeness::new(ph1k.interrupt_input.speech_likeness.clamp(0.0, 1.0))
            .map_err(|err| format!("invalid PH1.W speech likeness: {err:?}"))?,
    );
    let light_score = match ph1k.interrupt_input.detection.as_ref() {
        Some(detection) => Some(
            Confidence::new(detection.confidence.clamp(0.0, 1.0))
                .map_err(|err| format!("invalid PH1.W light score: {err:?}"))?,
        ),
        None => Some(
            Confidence::new(
                ph1k.interrupt_input
                    .acoustic_confidence
                    .max(ph1k.interrupt_input.vad_confidence)
                    .clamp(0.0, 1.0),
            )
            .map_err(|err| format!("invalid PH1.W fallback light score: {err:?}"))?,
        ),
    };
    let strong_score = Some(
        Confidence::new(
            ph1k.interrupt_input
                .acoustic_confidence
                .max(ph1k.interrupt_input.vad_confidence)
                .max(ph1k.interrupt_input.prosody_confidence)
                .max(ph1k.interrupt_input.speech_likeness)
                .clamp(0.0, 1.0),
        )
        .map_err(|err| format!("invalid PH1.W strong score: {err:?}"))?,
    );
    let source_liveness_hint =
        if ph1k.interrupt_input.capture_degraded || ph1k.interrupt_input.stream_gap_detected {
            SourceLivenessHint::ReplaySuspected
        } else {
            SourceLivenessHint::Live
        };
    let near_field_speech_hint = ph1k
        .interrupt_input
        .nearfield_confidence
        .map(|confidence| confidence >= 0.5);
    let speaker_match_ok = !ph1k.interrupt_input.capture_degraded
        && (near_field_speech_hint.unwrap_or(false) || ph1k.interrupt_input.vad_confidence >= 0.10);
    Ok(WakeStepInput {
        now,
        policy: WakePolicyContext::v1_with_media_and_trigger(
            WakeSessionState::Active,
            false,
            false,
            ph1k.tts_playback.active,
            false,
            false,
        ),
        processed_stream: ph1k.processed_stream_ref,
        pre_roll,
        vad: Some(vad_event),
        preceding_silence_ms: Some(250),
        utterance_start_offset_ms: Some(320),
        timing: ph1k.timing_stats,
        device_state: ph1k.device_state.clone(),
        aec_stable: !ph1k.interrupt_input.aec_unstable,
        light_score,
        strong_score,
        strong_alignment_ok: ph1k.interrupt_input.detection.is_some()
            || wake_trigger_alignment_hint,
        speaker_match_ok,
        source_liveness_hint,
        near_field_speech_hint,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Ph1wLiveDecisionLoopConfig {
    window_ms: u32,
    hop_ms: u32,
    max_steps: u64,
}

fn parse_u32_env(key: &str, min: u32, max: u32) -> Option<u32> {
    env::var(key)
        .ok()
        .and_then(|raw| raw.trim().parse::<u32>().ok())
        .map(|v| v.clamp(min, max))
}

fn resolve_ph1w_live_loop_config(app_platform: AppPlatform) -> Ph1wLiveDecisionLoopConfig {
    let (default_window_ms, default_hop_ms) = match app_platform {
        AppPlatform::Desktop => (1_500_u32, 200_u32),
        AppPlatform::Android | AppPlatform::Tablet => (800_u32, 20_u32),
        AppPlatform::Ios => (800_u32, 20_u32),
    };
    let window_ms =
        parse_u32_env("SELENE_PH1W_LIVE_WINDOW_MS", 200, 10_000).unwrap_or(default_window_ms);
    let hop_ms = parse_u32_env("SELENE_PH1W_LIVE_HOP_MS", 20, 2_000)
        .unwrap_or(default_hop_ms)
        .min(window_ms.max(20));
    let derived_steps = ((window_ms as u64)
        .saturating_add(hop_ms as u64)
        .saturating_sub(1))
    .saturating_div(hop_ms as u64)
    .saturating_add(1)
    .max(2);
    let max_steps = parse_u32_env("SELENE_PH1W_LIVE_MAX_STEPS", 2, 512)
        .map(u64::from)
        .unwrap_or(derived_steps);
    Ph1wLiveDecisionLoopConfig {
        window_ms,
        hop_ms,
        max_steps,
    }
}

fn run_ph1w_live_decision(
    now: MonotonicTimeNs,
    ph1k: &Ph1kLiveSignalBundle,
    app_platform: AppPlatform,
) -> Result<(WakeDecision, u16, String), String> {
    let mut cfg = EngineWakeConfig::mvp_desktop_v1();
    if app_platform == AppPlatform::Desktop {
        cfg.min_vad_confidence = cfg.min_vad_confidence.min(0.10);
        cfg.min_speech_likeness = cfg.min_speech_likeness.min(0.0);
        cfg.light_score_threshold = cfg.light_score_threshold.min(0.10);
        cfg.strong_score_threshold = cfg.strong_score_threshold.min(0.10);
        cfg.strong_score_threshold_tts = cfg.strong_score_threshold_tts.min(0.12);
        cfg.max_jitter_ms = cfg.max_jitter_ms.max(300.0);
        cfg.max_drift_ppm = cfg.max_drift_ppm.max(5_000.0);
        cfg.max_underruns = cfg.max_underruns.max(10);
    }
    let loop_cfg = resolve_ph1w_live_loop_config(app_platform);
    cfg.candidate_validation_window_ms = cfg.candidate_validation_window_ms.max(loop_cfg.window_ms);
    let threshold_used_bp = if ph1k.tts_playback.active {
        ((cfg.strong_score_threshold_tts.clamp(0.0, 1.0) * 10_000.0).round() as u16).min(10_000)
    } else {
        ((cfg.strong_score_threshold.clamp(0.0, 1.0) * 10_000.0).round() as u16).min(10_000)
    };
    let mut runtime = Ph1wRuntime::new(cfg);
    let hop_ns = (loop_cfg.hop_ms as u64).saturating_mul(1_000_000);
    for idx in 0..loop_cfg.max_steps {
        let step_now = MonotonicTimeNs(now.0.saturating_add(idx.saturating_mul(hop_ns)));
        let step_input = build_ph1w_live_step_input(step_now, ph1k, true)?;
        let events = runtime
            .step_for_implementation(PH1W_IMPLEMENTATION_ID, step_input)
            .map_err(|err| format!("PH1.W runtime rejected step input: {err:?}"))?;
        if let Some(decision) = events.into_iter().find_map(|event| match event {
            Ph1wOutputEvent::Decision(d) => Some(d),
            Ph1wOutputEvent::StateChanged { .. } => None,
        }) {
            return Ok((
                decision,
                threshold_used_bp,
                PH1W_IMPLEMENTATION_ID.to_string(),
            ));
        }
    }
    let fallback = WakeDecision::reject_v1(
        ph1w_reason_codes::W_WAKE_REJECTED_TIMEOUT,
        WakeGateResults {
            g0_integrity_ok: false,
            g1_activity_ok: false,
            g1a_utterance_start_ok: false,
            g2_light_ok: false,
            g3_strong_ok: false,
            g3a_liveness_ok: false,
            g4_personalization_ok: false,
            g5_policy_ok: true,
        },
        now,
        None,
        None,
    )
    .map_err(|err| format!("PH1.W timeout fallback decision invalid: {err:?}"))?;
    Ok((
        fallback,
        threshold_used_bp,
        PH1W_IMPLEMENTATION_ID.to_string(),
    ))
}

fn evaluate_wake_for_turn(
    store: &Ph1fStore,
    now: MonotonicTimeNs,
    actor_user_id: &UserId,
    device_id: &DeviceId,
    app_platform: AppPlatform,
    trigger: OsVoiceTrigger,
    ph1k: &Ph1kLiveSignalBundle,
) -> Result<Option<WakeEvaluation>, String> {
    if trigger != OsVoiceTrigger::WakeWord {
        return Ok(None);
    }
    if app_platform == AppPlatform::Ios {
        return Err("ios_wake_disabled".to_string());
    }
    let wake_profile_id = match app_platform {
        AppPlatform::Android | AppPlatform::Tablet | AppPlatform::Desktop => Some(
            store
                .ph1w_get_active_wake_profile(actor_user_id, device_id)
                .ok_or_else(|| "wake_not_enrolled".to_string())?
                .to_string(),
        ),
        AppPlatform::Ios => None,
    };
    let (decision, threshold_used_bp, model_version) =
        run_ph1w_live_decision(now, ph1k, app_platform)?;
    let (window_start_ns, window_end_ns) = decision
        .capture
        .map(|capture| (capture.t_start, capture.t_end))
        .unwrap_or((
            ph1k.pre_roll_buffer_ref.t_start,
            ph1k.pre_roll_buffer_ref.t_end,
        ));
    Ok(Some(WakeEvaluation {
        decision,
        wake_profile_id,
        threshold_used_bp,
        model_version,
        window_start_ns,
        window_end_ns,
    }))
}

#[allow(clippy::too_many_arguments)]
fn commit_wake_runtime_event(
    store: &mut Ph1fStore,
    now: MonotonicTimeNs,
    correlation_id: CorrelationId,
    turn_id: TurnId,
    actor_user_id: &UserId,
    device_id: &DeviceId,
    session_id: Option<SessionId>,
    ph1k: &Ph1kLiveSignalBundle,
    wake_eval: &WakeEvaluation,
) -> Result<(), String> {
    let decision = &wake_eval.decision;
    let wake_event_id = format!(
        "wake_evt_{}_{}_{}",
        correlation_id.0,
        turn_id.0,
        if decision.accepted {
            "accept"
        } else {
            "reject"
        }
    );
    let idempotency_key = sanitize_idempotency_token(&format!(
        "adapter_wake_runtime:{}:{}:{}:{}",
        correlation_id.0,
        turn_id.0,
        if decision.accepted { 1 } else { 0 },
        decision.reason_code.0
    ));
    store
        .ph1w_runtime_event_commit(
            now,
            wake_event_id,
            session_id,
            Some(actor_user_id.clone()),
            device_id.clone(),
            decision.accepted,
            decision.reason_code,
            wake_eval.wake_profile_id.clone(),
            ph1k.tts_playback.active,
            false,
            None,
            confidence_to_basis_points(decision.light_score),
            confidence_to_basis_points(decision.strong_score),
            Some(wake_eval.threshold_used_bp),
            Some(truncate_ascii(&wake_eval.model_version, 64)),
            Some(wake_eval.window_start_ns),
            Some(wake_eval.window_end_ns),
            idempotency_key,
        )
        .map_err(storage_error_to_string)?;
    Ok(())
}

fn map_wake_decision_to_learn_signal_type(
    decision: &WakeDecision,
    threshold_used_bp: u16,
) -> LearnSignalType {
    if decision.accepted {
        return LearnSignalType::WakeAccepted;
    }
    if decision.reason_code == ph1w_reason_codes::W_FAIL_G1_NOISE {
        return LearnSignalType::NoisyEnvironment;
    }
    let low_confidence_reason = matches!(
        decision.reason_code,
        ph1w_reason_codes::W_FAIL_G3_SCORE_LOW | ph1w_reason_codes::W_FAIL_G3_UNSTABLE_SCORE
    );
    let near_threshold = confidence_to_basis_points(decision.strong_score)
        .map(|score_bp| score_bp.saturating_add(300) >= threshold_used_bp)
        .unwrap_or(false);
    if low_confidence_reason || near_threshold {
        return LearnSignalType::LowConfidenceWake;
    }
    LearnSignalType::WakeRejected
}

fn wake_learn_trigger(trigger: OsVoiceTrigger) -> WakeLearnTrigger {
    match trigger {
        OsVoiceTrigger::WakeWord => WakeLearnTrigger::WakeWord,
        OsVoiceTrigger::Explicit => WakeLearnTrigger::Explicit,
    }
}

fn wake_window_id_for_signal(device_id: &DeviceId, wake_eval: &WakeEvaluation) -> String {
    let suffix = stable_hash_hex_16(&format!(
        "{}:{}:{}:{}:{}",
        device_id.as_str(),
        wake_eval.window_start_ns.0,
        wake_eval.window_end_ns.0,
        wake_eval.decision.reason_code.0,
        if wake_eval.decision.accepted { 1 } else { 0 }
    ));
    format!("wake_window_{suffix}")
}

#[allow(clippy::too_many_arguments)]
fn commit_wake_learn_signal(
    store: &mut Ph1fStore,
    now: MonotonicTimeNs,
    correlation_id: CorrelationId,
    turn_id: TurnId,
    device_id: &DeviceId,
    session_id: Option<SessionId>,
    trigger: OsVoiceTrigger,
    ph1k: &Ph1kLiveSignalBundle,
    wake_eval: &WakeEvaluation,
) -> Result<(), String> {
    let decision = &wake_eval.decision;
    let event_type = map_wake_decision_to_learn_signal_type(decision, wake_eval.threshold_used_bp);
    let signal_id = format!(
        "wake_sig_{}_{}_{}",
        correlation_id.0,
        turn_id.0,
        if decision.accepted {
            "accept"
        } else {
            "reject"
        }
    );
    let idempotency_key = sanitize_idempotency_token(&format!(
        "adapter_wake_learn:{}:{}:{}:{}",
        correlation_id.0, turn_id.0, decision.reason_code.0, wake_eval.window_start_ns.0
    ));
    let wake_window_id = wake_window_id_for_signal(device_id, wake_eval);
    let score_bp = confidence_to_basis_points(decision.strong_score)
        .or(confidence_to_basis_points(decision.light_score));
    let snr_db_milli = Some(
        (ph1k
            .interrupt_input
            .adaptive_policy_input
            .quality_metrics
            .snr_db
            * 1000.0)
            .round() as i32,
    );
    let vad_coverage_bp = Some(
        ((ph1k.interrupt_input.vad_confidence.clamp(0.0, 1.0) * 10_000.0).round() as u16)
            .min(10_000),
    );
    let timestamp_ms = (now.0 / 1_000_000).max(1);
    let signal = WakeLearnSignalV1::v1(
        signal_id,
        idempotency_key,
        wake_window_id,
        event_type,
        device_id.clone(),
        session_id,
        wake_learn_trigger(trigger),
        Some(truncate_ascii(&wake_eval.model_version, 64)),
        score_bp,
        Some(wake_eval.threshold_used_bp),
        Some(decision.reason_code),
        snr_db_milli,
        vad_coverage_bp,
        timestamp_ms,
    )
    .map_err(|err| format!("wake learn signal contract invalid: {err:?}"))?;
    store
        .wake_learn_signal_commit_and_enqueue(now, signal)
        .map_err(storage_error_to_string)?;
    Ok(())
}

fn resolve_session_turn_state(
    store: &mut Ph1fStore,
    now: MonotonicTimeNs,
    correlation_id: CorrelationId,
    turn_id: TurnId,
    input: ResolveSessionTurnStateInput<'_>,
) -> Result<AdapterSessionResolution, String> {
    let ResolveSessionTurnStateInput {
        actor_user_id,
        device_id,
        trigger,
        ph1k,
        wake_event,
        idempotency_key,
        device_turn_sequence,
        runtime_node_id,
        session_lease_ttl_ms,
        session_retry_cache,
    } = input;
    let selection = canonical_actor_session_selection(store, actor_user_id)?;
    if let Some(existing) = selection.latest_recoverable.as_ref() {
        match existing.device_turn_sequence_for(device_id) {
            Some(previous_sequence) if device_turn_sequence < previous_sequence => {
                return Err("session_conflict stale_device_turn".to_string());
            }
            Some(previous_sequence) if device_turn_sequence == previous_sequence => {
                if existing.last_idempotency_key_for(device_id) == Some(idempotency_key) {
                    let response = cached_retry_response_for_actor(
                        session_retry_cache,
                        actor_user_id,
                        idempotency_key,
                    )?
                    .ok_or_else(|| "retry_result_missing".to_string())?;
                    return Ok(AdapterSessionResolution::Retry(response));
                }
                return Err("session_conflict duplicate_device_turn_sequence".to_string());
            }
            _ => {}
        }
        acquire_or_refresh_session_lease(
            store,
            now,
            correlation_id,
            turn_id,
            device_id,
            existing,
            runtime_node_id,
            session_lease_ttl_ms,
        )?;
    }
    let next_session_id_seed = store
        .session_rows()
        .keys()
        .map(|session_id| session_id.0)
        .max()
        .unwrap_or(0)
        .saturating_add(1)
        .max(1);
    let (state, session_id) = match selection.latest_recoverable.as_ref() {
        Some(rec) if rec.session_state != SessionState::Closed => {
            (rec.session_state, Some(rec.session_id))
        }
        _ => (SessionState::Closed, None),
    };
    let mut lifecycle = Ph1lRuntime::from_persisted_state(
        Ph1lConfig::mvp_desktop_v1(),
        state,
        session_id,
        next_session_id_seed,
    )
    .map_err(|err| format!("invalid PH1.L persisted state: {err:?}"))?;

    let policy_context_ref = PolicyContextRef::v1(false, false, SafetyTier::Standard);
    if let Some(rec) = selection.latest_recoverable.as_ref() {
        let silence_ms = now
            .0
            .saturating_sub(rec.last_activity_at.0)
            .saturating_div(1_000_000)
            .min(u32::MAX as u64) as u32;
        if silence_ms > 0 {
            let idle_prev_session_id = lifecycle.session_id();
            let idle_out = lifecycle.step(Ph1lInput::v1(
                now,
                None,
                None,
                tts_playback_state_from_bool(ph1k.tts_playback.active),
                UserActivitySignals {
                    speech_detected: false,
                    barge_in: false,
                    silence_ms,
                },
                policy_context_ref,
                false,
                false,
                false,
            ));
            persist_session_snapshot(
                store,
                now,
                correlation_id,
                turn_id,
                actor_user_id,
                device_id,
                idle_prev_session_id,
                &idle_out,
                "idle",
                runtime_node_id,
                session_lease_ttl_ms,
                false,
            )?;
        }
    }

    let ph1l_turn_trigger = ph1l_turn_trigger_from_os(trigger);
    let wake_event = if trigger_requires_session_open_step(ph1l_turn_trigger) {
        wake_event
    } else {
        None
    };
    let active_prev_session_id = lifecycle.session_id();
    let active_out = ph1l_step_voice_turn(
        &mut lifecycle,
        now,
        ph1l_turn_trigger,
        wake_event.clone(),
        tts_playback_state_from_bool(ph1k.tts_playback.active),
        policy_context_ref,
    );
    let session_attach_outcome = match selection.latest_recoverable.as_ref() {
        Some(rec) if rec.attached_devices.contains(device_id) => {
            SessionAttachOutcome::ExistingSessionReused
        }
        Some(_) => SessionAttachOutcome::ExistingSessionAttached,
        None => SessionAttachOutcome::NewSessionCreated,
    };
    persist_session_snapshot(
        store,
        now,
        correlation_id,
        turn_id,
        actor_user_id,
        device_id,
        active_prev_session_id,
        &active_out,
        "turn",
        runtime_node_id,
        session_lease_ttl_ms,
        true,
    )?;
    let session_id_for_commits = if active_out.snapshot.session_state == SessionState::Closed {
        active_prev_session_id
    } else {
        active_out.snapshot.session_id
    };
    Ok(AdapterSessionResolution::Proceed(AdapterSessionTurnState {
        session_snapshot: active_out.snapshot,
        session_id_for_commits,
        wake_event,
        session_attach_outcome,
        device_turn_sequence,
    }))
}

fn ph1l_turn_trigger_from_os(trigger: OsVoiceTrigger) -> Ph1lTurnTrigger {
    match trigger {
        OsVoiceTrigger::WakeWord => Ph1lTurnTrigger::WakeWord,
        OsVoiceTrigger::Explicit => Ph1lTurnTrigger::Explicit,
    }
}

fn canonical_actor_session_selection(
    store: &Ph1fStore,
    actor_user_id: &UserId,
) -> Result<CanonicalActorSessionSelection, String> {
    let mut latest: Option<SessionRecord> = None;
    let mut latest_recoverable: Option<SessionRecord> = None;
    let recoverable_count = store
        .session_rows()
        .values()
        .filter(|row| &row.user_id == actor_user_id && row.is_recoverable())
        .count();
    if recoverable_count > 1 {
        return Err("session_conflict canonical_session_owner_uncertain".to_string());
    }
    for row in store
        .session_rows()
        .values()
        .filter(|row| &row.user_id == actor_user_id)
    {
        let better_than_latest = latest
            .as_ref()
            .map(|existing| {
                (row.last_activity_at.0, row.session_id.0)
                    > (existing.last_activity_at.0, existing.session_id.0)
            })
            .unwrap_or(true);
        if better_than_latest {
            latest = Some(row.clone());
        }
        if row.is_recoverable() {
            let better_than_recoverable = latest_recoverable
                .as_ref()
                .map(|existing| {
                    (row.last_activity_at.0, row.session_id.0)
                        > (existing.last_activity_at.0, existing.session_id.0)
                })
                .unwrap_or(true);
            if better_than_recoverable {
                latest_recoverable = Some(row.clone());
            }
        }
    }
    Ok(CanonicalActorSessionSelection {
        latest,
        latest_recoverable,
    })
}

fn latest_canonical_session_for_actor(
    store: &Ph1fStore,
    actor_user_id: &UserId,
) -> Result<Option<SessionRecord>, String> {
    let selection = canonical_actor_session_selection(store, actor_user_id)?;
    Ok(selection.latest_recoverable.or(selection.latest))
}

fn session_lease_expires_at(now: MonotonicTimeNs, session_lease_ttl_ms: u64) -> MonotonicTimeNs {
    MonotonicTimeNs(
        now.0
            .saturating_add(session_lease_ttl_ms.max(1).saturating_mul(1_000_000)),
    )
}

fn cached_retry_response_for_actor(
    session_retry_cache: &Arc<Mutex<BTreeMap<AdapterRetryCacheKey, VoiceTurnAdapterResponse>>>,
    actor_user_id: &UserId,
    idempotency_key: &str,
) -> Result<Option<VoiceTurnAdapterResponse>, String> {
    let cache = session_retry_cache
        .lock()
        .map_err(|_| "adapter retry cache lock poisoned".to_string())?;
    Ok(cache
        .get(&AdapterRetryCacheKey {
            actor_user_id: actor_user_id.clone(),
            idempotency_key: idempotency_key.to_string(),
        })
        .cloned()
        .map(|mut response| {
            response.session_attach_outcome = Some(SessionAttachOutcome::RetryReusedResult);
            response
        }))
}

fn cache_authoritative_turn_response(
    session_retry_cache: &Arc<Mutex<BTreeMap<AdapterRetryCacheKey, VoiceTurnAdapterResponse>>>,
    actor_user_id: &UserId,
    idempotency_key: &str,
    response: &VoiceTurnAdapterResponse,
) -> Result<(), String> {
    let mut cache = session_retry_cache
        .lock()
        .map_err(|_| "adapter retry cache lock poisoned".to_string())?;
    cache.insert(
        AdapterRetryCacheKey {
            actor_user_id: actor_user_id.clone(),
            idempotency_key: idempotency_key.to_string(),
        },
        response.clone(),
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn acquire_or_refresh_session_lease(
    store: &mut Ph1fStore,
    now: MonotonicTimeNs,
    correlation_id: CorrelationId,
    turn_id: TurnId,
    device_id: &DeviceId,
    session_record: &SessionRecord,
    runtime_node_id: &str,
    session_lease_ttl_ms: u64,
) -> Result<(), String> {
    let mut leased_record = session_record.clone();
    if leased_record.lease_is_active_at(now) {
        if leased_record.lease_owner_id.as_deref() != Some(runtime_node_id) {
            return Err("session_conflict session_lease_conflict".to_string());
        }
        if let Some(active_turn_id) = leased_record.active_turn_id {
            if active_turn_id != turn_id {
                return Err("session_conflict session_single_writer_conflict".to_string());
            }
        }
    }
    leased_record.attached_devices.insert(device_id.clone());
    leased_record.last_attached_device_id = device_id.clone();
    leased_record.lease_owner_id = Some(truncate_ascii(runtime_node_id, 128));
    leased_record.lease_acquired_at = Some(now);
    leased_record.lease_expires_at = Some(session_lease_expires_at(now, session_lease_ttl_ms));
    leased_record.active_turn_id = Some(turn_id);
    store
        .upsert_session_lifecycle(
            leased_record,
            Some(sanitize_idempotency_token(&format!(
                "adapter_session_lease:{}:{}:{}",
                correlation_id.0, turn_id.0, session_record.session_id.0
            ))),
        )
        .map_err(storage_error_to_string)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn finalize_session_turn_record(
    store: &mut Ph1fStore,
    now: MonotonicTimeNs,
    correlation_id: CorrelationId,
    turn_id: TurnId,
    device_id: &DeviceId,
    session_id: Option<SessionId>,
    device_turn_sequence: u64,
    idempotency_key: &str,
    runtime_node_id: &str,
    session_lease_ttl_ms: u64,
) -> Result<(), String> {
    let Some(session_id) = session_id else {
        return Ok(());
    };
    let Some(mut record) = store.get_session(&session_id).cloned() else {
        return Ok(());
    };
    record.attached_devices.insert(device_id.clone());
    record.last_attached_device_id = device_id.clone();
    record.last_turn_id = Some(turn_id);
    record.last_activity_at = now;
    record
        .device_turn_sequences
        .insert(device_id.clone(), device_turn_sequence);
    record
        .device_last_idempotency_keys
        .insert(device_id.clone(), truncate_ascii(idempotency_key, 128));
    if record.session_state == SessionState::Closed {
        record.active_turn_id = None;
        record.lease_owner_id = None;
        record.lease_acquired_at = None;
        record.lease_expires_at = None;
        record.closed_at = Some(now);
    } else {
        record.active_turn_id = None;
        record.lease_owner_id = Some(truncate_ascii(runtime_node_id, 128));
        record.lease_acquired_at = Some(now);
        record.lease_expires_at = Some(session_lease_expires_at(now, session_lease_ttl_ms));
    }
    store
        .upsert_session_lifecycle(
            record,
            Some(sanitize_idempotency_token(&format!(
                "adapter_session_finalize:{}:{}:{}",
                correlation_id.0, turn_id.0, session_id.0
            ))),
        )
        .map_err(storage_error_to_string)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn persist_session_snapshot(
    store: &mut Ph1fStore,
    now: MonotonicTimeNs,
    correlation_id: CorrelationId,
    turn_id: TurnId,
    actor_user_id: &UserId,
    device_id: &DeviceId,
    previous_session_id: Option<SessionId>,
    out: &selene_kernel_contracts::ph1l::Ph1lOutput,
    stage: &str,
    runtime_node_id: &str,
    session_lease_ttl_ms: u64,
    hold_active_turn: bool,
) -> Result<(), String> {
    let session_id = if out.snapshot.session_state == SessionState::Closed {
        previous_session_id
    } else {
        out.snapshot.session_id
    };
    let Some(session_id) = session_id else {
        return Ok(());
    };
    let mut record = match store.get_session(&session_id).cloned() {
        Some(existing) => existing,
        None => SessionRecord::v1(
            session_id,
            actor_user_id.clone(),
            device_id.clone(),
            out.snapshot.session_state,
            now,
            now,
            None,
        )
        .map_err(|err| format!("invalid PH1.L session record: {err:?}"))?,
    };
    record.session_state = out.snapshot.session_state;
    record.last_activity_at = now;
    record.attached_devices.insert(device_id.clone());
    record.last_attached_device_id = device_id.clone();
    record.closed_at = if out.snapshot.session_state == SessionState::Closed {
        Some(now)
    } else {
        None
    };
    if out.snapshot.session_state == SessionState::Closed {
        record.active_turn_id = None;
        record.lease_owner_id = None;
        record.lease_acquired_at = None;
        record.lease_expires_at = None;
    } else {
        if record.lease_owner_id.is_none() || !record.lease_is_active_at(now) {
            record.lease_owner_id = Some(truncate_ascii(runtime_node_id, 128));
            record.lease_acquired_at = Some(now);
        }
        record.lease_expires_at = Some(session_lease_expires_at(now, session_lease_ttl_ms));
        if hold_active_turn {
            record.active_turn_id = Some(turn_id);
        }
    }
    store
        .upsert_session_lifecycle(
            record,
            Some(sanitize_idempotency_token(&format!(
                "adapter_session:{}:{}:{}:{}",
                correlation_id.0, turn_id.0, stage, session_id.0
            ))),
        )
        .map_err(storage_error_to_string)?;
    Ok(())
}

fn tts_playback_state_from_bool(active: bool) -> TtsPlaybackState {
    if active {
        TtsPlaybackState::Playing
    } else {
        TtsPlaybackState::Stopped
    }
}

fn load_ph1x_thread_state(
    store: &Ph1fStore,
    actor_user_id: &UserId,
    thread_key: &str,
) -> ThreadState {
    store
        .ph1x_thread_state_current_row(actor_user_id, thread_key)
        .map(|row| row.thread_state.clone())
        .unwrap_or_else(ThreadState::empty_v1)
}

fn persist_ph1x_thread_state(
    store: &mut Ph1fStore,
    now: MonotonicTimeNs,
    input: PersistPh1xThreadStateInput<'_>,
) -> Result<(), String> {
    let PersistPh1xThreadStateInput {
        actor_user_id,
        thread_key,
        thread_state,
        reason_code,
        correlation_id,
        turn_id,
    } = input;
    let idempotency_key = sanitize_idempotency_token(&format!(
        "adapter_ph1x_thread_state:{}:{}:{}",
        correlation_id.0, turn_id.0, thread_key
    ));
    let _ = store
        .ph1x_thread_state_upsert_commit(
            now,
            actor_user_id.clone(),
            thread_key.to_string(),
            thread_state,
            reason_code,
            idempotency_key,
        )
        .map_err(storage_error_to_string)?;
    Ok(())
}

fn truncate_ascii(value: &str, max_len: usize) -> String {
    value.chars().take(max_len).collect::<String>()
}

fn truncate_utf8(value: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    value.chars().take(max_chars).collect()
}

fn stable_hash_hex_16(value: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn parse_bool_env(key: &str, default: bool) -> bool {
    match env::var(key) {
        Ok(v) => !matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "0" | "false" | "off" | "no"
        ),
        Err(_) => default,
    }
}

fn parse_u64_env(key: &str, default: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

fn runtime_node_id_from_env() -> String {
    env::var("SELENE_RUNTIME_NODE_ID")
        .ok()
        .map(|value| truncate_ascii(value.trim(), 128))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "adapter_runtime_node_v1".to_string())
}

fn build_ph1d_live_adapter_from_env() -> Option<EnvPh1dLiveAdapter> {
    if !parse_bool_env("SELENE_PH1D_LIVE_ADAPTER_ENABLED", true) {
        return None;
    }
    if env::var("SELENE_PH1D_LIVE_PROVIDER_ID")
        .ok()
        .map(|value| value.trim().is_empty())
        .unwrap_or(true)
    {
        return None;
    }
    match EnvPh1dLiveAdapter::from_env() {
        Ok(adapter) => Some(adapter),
        Err(err) => {
            eprintln!("selene_adapter ph1d live adapter bootstrap failed: {err:?}");
            None
        }
    }
}

fn correlation_id_to_u64(correlation_id: CorrelationId) -> u64 {
    (correlation_id.0 as u64).max(1)
}

fn resolve_tenant_scope(
    explicit_tenant_id: Option<String>,
    actor_user_id: &UserId,
    device_id: Option<&DeviceId>,
) -> Option<String> {
    explicit_tenant_id
        .map(|v| truncate_ascii(v.trim(), 64))
        .filter(|v| !v.is_empty())
        .or_else(|| tenant_scope_from_user_id(actor_user_id).map(str::to_string))
        .or_else(|| {
            device_id
                .map(|d| truncate_ascii(&format!("tenant_{}", stable_hash_hex_16(d.as_str())), 64))
        })
}

fn parse_device_route_label(value: &str) -> Option<DeviceRoute> {
    match value.trim().to_ascii_uppercase().as_str() {
        "BUILT_IN" | "BUILTIN" => Some(DeviceRoute::BuiltIn),
        "BLUETOOTH" => Some(DeviceRoute::Bluetooth),
        "USB" => Some(DeviceRoute::Usb),
        "VIRTUAL" => Some(DeviceRoute::Virtual),
        "UNKNOWN" => Some(DeviceRoute::Unknown),
        _ => None,
    }
}

fn sanitize_audio_device_token(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.') {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    let out = truncate_ascii(&out, 56);
    if out.trim().is_empty() {
        "adapter_device".to_string()
    } else {
        out
    }
}

fn locale_key(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace('_', "-")
}

fn locale_matches(lhs: &str, rhs: &str) -> bool {
    let lhs_key = locale_key(lhs);
    let rhs_key = locale_key(rhs);
    lhs_key == rhs_key
        || lhs_key.split('-').next().unwrap_or(lhs_key.as_str())
            == rhs_key.split('-').next().unwrap_or(rhs_key.as_str())
}

fn resolve_interrupt_locale_tag_from_capture(
    capture: &VoiceTurnAudioCaptureRef,
) -> Result<InterruptLocaleTag, String> {
    let raw = capture
        .locale_tag
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| "ph1k live capture missing locale_tag".to_string())?;
    InterruptLocaleTag::new(truncate_ascii(raw, 32))
        .map_err(|err| format!("ph1k locale_tag invalid: {err:?}"))
}

fn build_interrupt_matcher_and_binding(
    _store: &Ph1fStore,
    _tenant_scope: Option<&str>,
    _device_id: Option<&DeviceId>,
    locale_tag: &InterruptLocaleTag,
) -> Result<(InterruptPhraseMatcher, InterruptLexiconPolicyBinding), String> {
    let matcher = InterruptPhraseMatcher::built_in();
    let default_binding = matcher.default_policy_binding();
    let binding = InterruptLexiconPolicyBinding::v1(
        default_binding.policy_profile_id,
        default_binding.tenant_profile_id,
        locale_tag.clone(),
    )
    .map_err(|err| format!("ph1k binding invalid: {err:?}"))?;
    Ok((matcher, binding))
}

#[allow(clippy::too_many_arguments)]
fn build_ph1k_live_signal_bundle(
    store: &Ph1fStore,
    request: &VoiceTurnAdapterRequest,
    now: MonotonicTimeNs,
    tenant_scope: Option<&str>,
    device_id: Option<&DeviceId>,
) -> Result<Ph1kLiveSignalBundle, String> {
    let capture = request
        .audio_capture_ref
        .as_ref()
        .ok_or_else(|| "ph1k live capture bundle is required for voice turns".to_string())?;

    let locale_tag = resolve_interrupt_locale_tag_from_capture(capture)?;
    let (matcher, binding) =
        build_interrupt_matcher_and_binding(store, tenant_scope, device_id, &locale_tag)?;
    let selected_mic_raw = capture
        .selected_mic
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| "ph1k live capture missing selected_mic".to_string())?;
    let selected_speaker_raw = capture
        .selected_speaker
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| "ph1k live capture missing selected_speaker".to_string())?;
    let selected_mic = AudioDeviceId::new(sanitize_audio_device_token(selected_mic_raw))
        .map_err(|err| format!("ph1k selected_mic invalid: {err:?}"))?;
    let selected_speaker = AudioDeviceId::new(sanitize_audio_device_token(selected_speaker_raw))
        .map_err(|err| format!("ph1k selected_speaker invalid: {err:?}"))?;
    let device_route_raw = capture
        .device_route
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| "ph1k live capture missing device_route".to_string())?;
    let device_route = parse_device_route_label(device_route_raw)
        .ok_or_else(|| format!("ph1k live capture device_route invalid: '{device_route_raw}'"))?;

    let t_start = MonotonicTimeNs(capture.t_start_ns.max(1));
    let t_end = MonotonicTimeNs(capture.t_end_ns.max(capture.t_start_ns.saturating_add(1)));
    let t_candidate = MonotonicTimeNs(capture.t_candidate_start_ns.max(capture.t_start_ns));
    let t_confirmed = MonotonicTimeNs(capture.t_confirmed_ns.max(capture.t_candidate_start_ns));
    let span_ms = ((t_end.0.saturating_sub(t_start.0)) / 1_000_000).max(1);
    let confirm_delta_ms = ((t_confirmed.0.saturating_sub(t_candidate.0)) / 1_000_000).max(1);
    let jitter_ms = capture
        .timing_jitter_ms_milli
        .map(|v| v as f32 / 1000.0)
        .ok_or_else(|| "ph1k live capture missing timing_jitter_ms_milli".to_string())?;
    let drift_ppm = capture
        .timing_drift_ppm_milli
        .map(|v| v as f32 / 1000.0)
        .ok_or_else(|| "ph1k live capture missing timing_drift_ppm_milli".to_string())?;
    let buffer_depth_ms = capture
        .timing_buffer_depth_ms_milli
        .map(|v| v as f32 / 1000.0)
        .ok_or_else(|| "ph1k live capture missing timing_buffer_depth_ms_milli".to_string())?;
    let timing_underruns = capture.timing_underruns.unwrap_or(0);
    let timing_overruns = capture.timing_overruns.unwrap_or(0);

    let timing_stats = Ph1kTimingStats::v1(
        jitter_ms,
        drift_ppm,
        buffer_depth_ms,
        timing_underruns,
        timing_overruns,
    );
    timing_stats
        .validate()
        .map_err(|err| format!("ph1k timing stats invalid: {err:?}"))?;

    let vad_conf = capture
        .vad_confidence_bp
        .map(|v| (v as f32) / 10_000.0)
        .ok_or_else(|| "ph1k live capture missing vad_confidence_bp".to_string())?;
    let speech_likeness = capture
        .speech_likeness_bp
        .map(|v| (v as f32) / 10_000.0)
        .ok_or_else(|| "ph1k live capture missing speech_likeness_bp".to_string())?;
    let vad_events = vec![VadEvent::v1(
        AudioStreamId(capture.stream_id),
        t_start,
        t_end,
        Confidence::new(vad_conf).map_err(|err| format!("ph1k vad confidence invalid: {err:?}"))?,
        SpeechLikeness::new(speech_likeness)
            .map_err(|err| format!("ph1k speech likeness invalid: {err:?}"))?,
    )];

    let processed_stream_ref = AudioStreamRef::v1(
        AudioStreamId(capture.stream_id),
        AudioStreamKind::MicProcessed,
        AudioFormat {
            sample_rate_hz: SampleRateHz(16_000),
            channels: ChannelCount(1),
            sample_format: SampleFormat::PcmS16LE,
        },
        FrameDurationMs::Ms20,
    );
    let pre_roll_buffer_ref = PreRollBufferRef::v1(
        PreRollBufferId(capture.pre_roll_buffer_id),
        AudioStreamId(capture.stream_id),
        t_start,
        t_end,
    );
    let tts_playback_active = capture
        .tts_playback_active
        .ok_or_else(|| "ph1k live capture missing tts_playback_active".to_string())?;
    let tts_playback = TtsPlaybackActiveEvent::v1(tts_playback_active, now);
    let capture_degraded = capture
        .capture_degraded
        .ok_or_else(|| "ph1k live capture missing capture_degraded".to_string())?;
    let stream_gap_detected = capture
        .stream_gap_detected
        .ok_or_else(|| "ph1k live capture missing stream_gap_detected".to_string())?;
    let aec_unstable = capture
        .aec_unstable
        .ok_or_else(|| "ph1k live capture missing aec_unstable".to_string())?;
    let device_changed = capture
        .device_changed
        .ok_or_else(|| "ph1k live capture missing device_changed".to_string())?;
    let snr_db = capture
        .snr_db_milli
        .map(|v| v as f32 / 1000.0)
        .ok_or_else(|| "ph1k live capture missing snr_db_milli".to_string())?;
    let clipping_ratio = capture
        .clipping_ratio_bp
        .map(|v| (v as f32) / 10_000.0)
        .ok_or_else(|| "ph1k live capture missing clipping_ratio_bp".to_string())?;
    let echo_delay_ms = capture
        .echo_delay_ms_milli
        .map(|v| v as f32 / 1000.0)
        .ok_or_else(|| "ph1k live capture missing echo_delay_ms_milli".to_string())?;
    let packet_loss_pct = capture
        .packet_loss_bp
        .map(|v| (v as f32) / 100.0)
        .ok_or_else(|| "ph1k live capture missing packet_loss_bp".to_string())?;
    let double_talk_score = capture
        .double_talk_bp
        .map(|v| (v as f32) / 10_000.0)
        .ok_or_else(|| "ph1k live capture missing double_talk_bp".to_string())?;
    let erle_db = capture
        .erle_db_milli
        .map(|v| v as f32 / 1000.0)
        .ok_or_else(|| "ph1k live capture missing erle_db_milli".to_string())?;

    let mut adaptive_policy_input = default_adaptive_policy_input(device_route);
    adaptive_policy_input.quality_metrics = AdvancedAudioQualityMetrics::v1(
        snr_db,
        clipping_ratio,
        echo_delay_ms,
        packet_loss_pct,
        double_talk_score,
        erle_db,
    )
    .map_err(|err| format!("ph1k quality metrics invalid: {err:?}"))?;
    adaptive_policy_input.device_reliability = DeviceReliabilityScoreInput::v1(
        capture.device_failures_24h.unwrap_or(0),
        capture.device_recoveries_24h.unwrap_or(0),
        capture.device_mean_recovery_ms.unwrap_or(0),
        Confidence::new(
            capture
                .device_reliability_bp
                .map(|v| (v as f32) / 10_000.0)
                .ok_or_else(|| "ph1k live capture missing device_reliability_bp".to_string())?,
        )
        .map_err(|err| format!("ph1k device reliability score invalid: {err:?}"))?,
    )
    .map_err(|err| format!("ph1k device reliability invalid: {err:?}"))?;
    adaptive_policy_input.timing_stats = timing_stats;
    adaptive_policy_input.capture_to_handoff_latency_ms = confirm_delta_ms.min(10_000) as u32;
    let timing_stats_for_bundle = adaptive_policy_input.timing_stats;
    let device_health = if capture_degraded || aec_unstable || stream_gap_detected {
        DeviceHealth::Degraded
    } else {
        DeviceHealth::Healthy
    };
    let device_state = DeviceState::v1_with_route(
        selected_mic,
        selected_speaker,
        device_route,
        device_health,
        Vec::new(),
    );

    let detection = match capture
        .detection_text
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        Some(text) => {
            let detection_confidence = capture
                .detection_confidence_bp
                .map(|v| (v as f32) / 10_000.0)
                .ok_or_else(|| "ph1k live capture missing detection_confidence_bp".to_string())?;
            Some(PhraseDetection {
                text: truncate_utf8(text, 128),
                confidence: detection_confidence,
            })
        }
        None => None,
    };
    let acoustic_confidence = capture
        .acoustic_confidence_bp
        .map(|v| (v as f32) / 10_000.0)
        .ok_or_else(|| "ph1k live capture missing acoustic_confidence_bp".to_string())?;
    let prosody_confidence = capture
        .prosody_confidence_bp
        .map(|v| (v as f32) / 10_000.0)
        .ok_or_else(|| "ph1k live capture missing prosody_confidence_bp".to_string())?;
    let echo_safe_confidence = capture
        .echo_safe_confidence_bp
        .map(|v| (v as f32) / 10_000.0)
        .ok_or_else(|| "ph1k live capture missing echo_safe_confidence_bp".to_string())?;
    let nearfield_confidence = capture
        .nearfield_confidence_bp
        .map(|v| (v as f32) / 10_000.0);
    let interrupt_input = InterruptInput {
        lexicon_policy_binding: binding,
        adaptive_policy_input,
        tts_playback_active,
        capture_degraded,
        stream_gap_detected,
        aec_unstable,
        device_changed,
        voiced_window_ms: span_ms.min(2_000) as u32,
        vad_confidence: vad_conf,
        acoustic_confidence,
        prosody_confidence,
        speech_likeness,
        echo_safe_confidence,
        nearfield_confidence,
        detection,
        t_event: now,
    };
    let interrupt_decision = evaluate_interrupt_candidate(&matcher, interrupt_input.clone())
        .map_err(|err| format!("ph1k interrupt decision failed: {err:?}"))?;
    let ph1c_handoff = build_ph1k_to_ph1c_handoff(&interrupt_input, &interrupt_decision)
        .map_err(|err| format!("ph1k->ph1c handoff invalid: {err:?}"))?;

    Ok(Ph1kLiveSignalBundle {
        locale_tag,
        processed_stream_ref,
        pre_roll_buffer_ref,
        vad_events,
        device_state,
        timing_stats: timing_stats_for_bundle,
        tts_playback,
        interrupt_input,
        interrupt_decision,
        ph1c_handoff,
    })
}

pub fn validate_voice_turn_capture_bundle_for_live_path(
    request: &VoiceTurnAdapterRequest,
) -> Result<(), String> {
    let store = Ph1fStore::new_in_memory();
    let now = MonotonicTimeNs(request.now_ns.unwrap_or_else(system_time_now_ns).max(1));
    let _ = build_ph1k_live_signal_bundle(&store, request, now, None, None)?;
    Ok(())
}

fn build_voice_id_request_from_ph1k_bundle(
    now: MonotonicTimeNs,
    ph1k: &Ph1kLiveSignalBundle,
    session_snapshot: SessionSnapshot,
    wake_event: Option<WakeDecision>,
) -> Result<Ph1VoiceIdRequest, selene_kernel_contracts::ContractViolation> {
    Ph1VoiceIdRequest::v1(
        now,
        ph1k.processed_stream_ref,
        ph1k.vad_events.clone(),
        ph1k.device_state.selected_mic.clone(),
        session_snapshot,
        wake_event,
        ph1k.tts_playback.active,
        DeviceTrustLevel::Trusted,
        None,
    )
}

fn build_ph1c_live_request(
    ph1k: &Ph1kLiveSignalBundle,
    session_state: SessionState,
) -> Result<Ph1cRequest, String> {
    let bounded_audio_segment_ref = BoundedAudioSegmentRef::v1(
        ph1k.processed_stream_ref.stream_id,
        ph1k.pre_roll_buffer_ref.buffer_id,
        ph1k.pre_roll_buffer_ref.t_start,
        ph1k.pre_roll_buffer_ref.t_end,
        ph1k.pre_roll_buffer_ref.t_start,
        ph1k.pre_roll_buffer_ref.t_end,
    )
    .map_err(|err| format!("ph1c bounded audio segment invalid: {err:?}"))?;

    let language_hint = Some(LanguageHint::v1(
        LanguageTag::new(ph1k.locale_tag.as_str().to_string())
            .map_err(|err| format!("ph1c language tag invalid: {err:?}"))?,
        LanguageHintConfidence::Med,
    ));
    let noise_level_hint = Some(
        NoiseLevelHint::new(
            (ph1k.ph1c_handoff.quality_metrics.packet_loss_pct / 100.0).clamp(0.0, 1.0),
        )
        .map_err(|err| format!("ph1c noise hint invalid: {err:?}"))?,
    );
    let vad_quality_hint = Some(
        VadQualityHint::new(ph1k.interrupt_input.vad_confidence.clamp(0.0, 1.0))
            .map_err(|err| format!("ph1c vad hint invalid: {err:?}"))?,
    );
    let speaker_overlap_hint = Some(
        SpeakerOverlapHint::v1(
            if ph1k.tts_playback.active {
                SpeakerOverlapClass::InterruptionOverlap
            } else {
                SpeakerOverlapClass::SingleSpeaker
            },
            Confidence::new((0.88 + (ph1k.interrupt_input.vad_confidence * 0.1)).clamp(0.0, 1.0))
                .map_err(|err| format!("ph1c overlap confidence invalid: {err:?}"))?,
        )
        .map_err(|err| format!("ph1c overlap hint invalid: {err:?}"))?,
    );
    let req = Ph1cRequest::v1(
        bounded_audio_segment_ref,
        Ph1cSessionStateRef::v1(
            match session_state {
                SessionState::Closed => WakeSessionState::Closed,
                SessionState::Open => WakeSessionState::Open,
                SessionState::Active => WakeSessionState::Active,
                SessionState::SoftClosed => WakeSessionState::SoftClosed,
                SessionState::Suspended => WakeSessionState::Suspended,
            },
            ph1k.tts_playback.active,
        ),
        ph1k.device_state.clone(),
        language_hint,
        noise_level_hint,
        vad_quality_hint,
        Some(ph1k.ph1c_handoff.clone()),
    )
    .map_err(|err| format!("ph1c request invalid: {err:?}"))?;
    req.with_speaker_overlap_hint(speaker_overlap_hint)
        .map_err(|err| format!("ph1c overlap patch invalid: {err:?}"))
}

fn storage_device_health_from_bundle(bundle: &Ph1kLiveSignalBundle) -> Ph1kDeviceHealth {
    if bundle.interrupt_input.capture_degraded
        || bundle.interrupt_input.aec_unstable
        || bundle.interrupt_input.stream_gap_detected
    {
        return Ph1kDeviceHealth::Degraded;
    }
    match bundle.device_state.health {
        DeviceHealth::Healthy => Ph1kDeviceHealth::Healthy,
        DeviceHealth::Degraded => Ph1kDeviceHealth::Degraded,
        DeviceHealth::Failed => Ph1kDeviceHealth::Failed,
    }
}

fn interrupt_noise_class_label(noise_class: Option<InterruptNoiseClass>) -> &'static str {
    match noise_class.unwrap_or(InterruptNoiseClass::Clean) {
        InterruptNoiseClass::Clean => "CLEAN",
        InterruptNoiseClass::Elevated => "ELEVATED",
        InterruptNoiseClass::Severe => "SEVERE",
    }
}

fn interrupt_feedback_kind_label(kind: InterruptFeedbackSignalKind) -> &'static str {
    match kind {
        InterruptFeedbackSignalKind::FalseLexicalTrigger => "false_lexical",
        InterruptFeedbackSignalKind::MissedLexicalTrigger => "missed_lexical",
        InterruptFeedbackSignalKind::WrongConfidenceBand => "wrong_confidence",
    }
}

fn normalize_eval_locale_tag(value: &str) -> &'static str {
    match value.to_ascii_lowercase().as_str() {
        "en" | "en-us" => "en-US",
        "es" | "es-es" => "es-ES",
        "zh" | "zh-cn" => "zh-CN",
        "tr" | "tr-tr" => "tr-TR",
        _ => "en-US",
    }
}

fn eval_device_route_label(route: DeviceRoute) -> &'static str {
    match route {
        DeviceRoute::BuiltIn => "BUILT_IN",
        DeviceRoute::Bluetooth => "BLUETOOTH",
        DeviceRoute::Usb => "USB",
        DeviceRoute::Virtual => "VIRTUAL",
        DeviceRoute::Unknown => "VIRTUAL",
    }
}

fn percentile_p95_u32(values: &[u32]) -> Option<u32> {
    if values.is_empty() {
        return None;
    }
    let mut ordered = values.to_vec();
    ordered.sort_unstable();
    let rank = ((ordered.len() as f64) * 0.95).ceil() as usize;
    let idx = rank.saturating_sub(1).min(ordered.len().saturating_sub(1));
    ordered.get(idx).copied()
}

fn ph1k_device_failover_recovery_samples_ms(runtime_rows: &[&Ph1kRuntimeEventRecord]) -> Vec<u32> {
    let mut rows = runtime_rows
        .iter()
        .copied()
        .filter(|row| row.event_kind == Ph1kRuntimeEventKind::DeviceState)
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| row.created_at.0);

    let mut outage_start_ns: Option<u64> = None;
    let mut out = Vec::new();
    for row in rows {
        let Some(health) = row.device_health else {
            continue;
        };
        match health {
            Ph1kDeviceHealth::Failed | Ph1kDeviceHealth::Degraded => {
                if outage_start_ns.is_none() {
                    outage_start_ns = Some(row.created_at.0);
                }
            }
            Ph1kDeviceHealth::Healthy => {
                if let Some(start_ns) = outage_start_ns.take() {
                    let ms = ((row.created_at.0.saturating_sub(start_ns)) / 1_000_000).max(1);
                    out.push(ms.min(u32::MAX as u64) as u32);
                }
            }
        }
    }
    out
}

fn eval_commit_hash() -> String {
    env::var("SELENE_COMMIT_HASH")
        .ok()
        .map(|v| truncate_ascii(v.trim(), 40))
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "live_runtime".to_string())
}

fn resolve_repo_root_from_cwd() -> Option<PathBuf> {
    let cwd = env::current_dir().ok()?;
    for ancestor in cwd.ancestors() {
        if ancestor.join(".git").exists() {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}

fn append_ph1k_live_eval_snapshot_csv(
    store: &Ph1fStore,
    now: MonotonicTimeNs,
    correlation_id: CorrelationId,
    turn_id: TurnId,
    tenant_id: &str,
    bundle: &Ph1kLiveSignalBundle,
) -> Result<(), String> {
    let default_csv_path = resolve_repo_root_from_cwd()
        .map(|root| root.join(".dev/ph1k_live_eval_snapshot.csv"))
        .unwrap_or_else(|| PathBuf::from(".dev/ph1k_live_eval_snapshot.csv"));
    let csv_path = env::var("SELENE_PH1K_LIVE_EVAL_PATH")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .unwrap_or(default_csv_path);
    if let Some(parent) = csv_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|err| {
                format!(
                    "failed to create ph1k eval csv directory '{}': {}",
                    parent.display(),
                    err
                )
            })?;
        }
    }
    let needs_header = !csv_path.exists()
        || fs::metadata(&csv_path)
            .map(|meta| meta.len() == 0)
            .unwrap_or(true);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&csv_path)
        .map_err(|err| {
            format!(
                "failed to open ph1k eval csv '{}' for append: {}",
                csv_path.display(),
                err
            )
        })?;
    if needs_header {
        file.write_all(b"captured_at_utc,commit_hash,window_min,locale_tag,device_route,noise_class,overlap_speech,active_session_hours,interrupt_events,false_interrupt_count,missed_interrupt_count,false_interrupt_rate_per_hour,missed_interrupt_rate_pct,end_of_speech_p95_ms,capture_to_ph1c_handoff_p95_ms,device_failover_recovery_p95_ms,noisy_recovery_success_pct,multilingual_interrupt_recall_pct,audit_completeness_pct,tenant_isolation_pct\n")
            .map_err(|err| {
                format!(
                    "failed to write ph1k eval csv header '{}': {}",
                    csv_path.display(),
                    err
                )
            })?;
    }

    let captured_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| now.0.to_string());
    let locale_tag = normalize_eval_locale_tag(bundle.locale_tag.as_str());
    let device_route =
        eval_device_route_label(bundle.interrupt_input.adaptive_policy_input.device_route);
    let noise_class = interrupt_noise_class_label(bundle.interrupt_decision.adaptive_noise_class);
    let overlap_speech = if bundle.tts_playback.active { 1 } else { 0 };
    let window_start_ns = now.0.saturating_sub(3_600_000_000_000);
    let tenant = truncate_ascii(tenant_id, 64);
    let runtime_rows = store
        .ph1k_runtime_event_rows()
        .iter()
        .filter(|row| row.tenant_id == tenant && row.created_at.0 >= window_start_ns)
        .collect::<Vec<_>>();
    let feedback_rows = store
        .ph1k_feedback_capture_rows()
        .iter()
        .filter(|row| row.tenant_id == tenant && row.created_at.0 >= window_start_ns)
        .collect::<Vec<_>>();

    let (min_ns, max_ns) = runtime_rows.iter().fold((u64::MAX, 0_u64), |acc, row| {
        (acc.0.min(row.created_at.0), acc.1.max(row.created_at.0))
    });
    let active_session_hours = if runtime_rows.is_empty() {
        1.0 / 60.0
    } else {
        (((max_ns.saturating_sub(min_ns)).max(60_000_000_000) as f64) / 3_600_000_000_000.0) as f32
    };
    let interrupt_events = runtime_rows
        .iter()
        .filter(|row| row.event_kind == Ph1kRuntimeEventKind::InterruptCandidate)
        .count()
        .max(1) as u32;
    let false_interrupt_count = feedback_rows
        .iter()
        .filter(|row| row.issue_kind == Ph1kFeedbackIssueKind::FalseInterrupt)
        .count() as u32;
    let missed_interrupt_count = feedback_rows
        .iter()
        .filter(|row| row.issue_kind == Ph1kFeedbackIssueKind::MissedInterrupt)
        .count() as u32;
    let false_interrupt_rate_per_hour =
        false_interrupt_count as f32 / active_session_hours.max(1.0 / 60.0);
    let missed_interrupt_rate_pct =
        (missed_interrupt_count as f32 * 100.0) / interrupt_events as f32;
    let eos_samples_ms = runtime_rows
        .iter()
        .filter_map(|row| {
            row.interrupt_extended.as_ref().map(|ext| {
                ((ext
                    .timing_markers
                    .window_end
                    .0
                    .saturating_sub(ext.timing_markers.window_start.0))
                    / 1_000_000)
                    .clamp(1, 2_000) as u32
            })
        })
        .collect::<Vec<_>>();
    let end_of_speech_p95_ms = percentile_p95_u32(&eos_samples_ms).unwrap_or_else(|| {
        bundle
            .vad_events
            .last()
            .map(|ev| {
                ((ev.t_end.0.saturating_sub(ev.t_start.0)) / 1_000_000).clamp(1, 2_000) as u32
            })
            .unwrap_or(180)
    });
    let handoff_samples_ms = runtime_rows
        .iter()
        .filter_map(|row| {
            row.interrupt_extended
                .as_ref()
                .map(|ext| ext.adaptive_capture_to_handoff_latency_ms.max(1))
        })
        .collect::<Vec<_>>();
    let capture_to_ph1c_handoff_p95_ms =
        percentile_p95_u32(&handoff_samples_ms).unwrap_or_else(|| {
            bundle
                .interrupt_input
                .adaptive_policy_input
                .capture_to_handoff_latency_ms
                .max(1)
        });
    let failover_samples_ms = ph1k_device_failover_recovery_samples_ms(&runtime_rows);
    let device_failover_recovery_p95_ms =
        percentile_p95_u32(&failover_samples_ms).unwrap_or_else(|| {
            bundle
                .interrupt_input
                .adaptive_policy_input
                .device_reliability
                .mean_recovery_ms
                .max(1)
        });
    let noisy_attempts = runtime_rows
        .iter()
        .filter(|row| {
            row.event_kind == Ph1kRuntimeEventKind::InterruptCandidate
                && row
                    .interrupt_extended
                    .as_ref()
                    .map(|ext| ext.adaptive_noise_class.as_str() != "CLEAN")
                    .unwrap_or(false)
        })
        .count()
        .max(1) as u32;
    let noisy_failures = feedback_rows
        .iter()
        .filter(|row| {
            row.issue_kind == Ph1kFeedbackIssueKind::WrongDegradationClassification
                && row
                    .adaptive_noise_class
                    .as_deref()
                    .map(|v| v != "CLEAN")
                    .unwrap_or(false)
        })
        .count() as u32;
    let noisy_recovery_success_pct =
        (((noisy_attempts.saturating_sub(noisy_failures)) as f32) * 100.0) / noisy_attempts as f32;

    let multilingual_candidates = runtime_rows
        .iter()
        .filter(|row| {
            row.event_kind == Ph1kRuntimeEventKind::InterruptCandidate
                && row
                    .interrupt_extended
                    .as_ref()
                    .map(|ext| {
                        !locale_matches(ext.interrupt_locale_tag.as_str(), "en-US")
                            && !locale_matches(ext.interrupt_locale_tag.as_str(), "en")
                    })
                    .unwrap_or(false)
        })
        .count() as u32;
    let multilingual_denominator = multilingual_candidates
        .saturating_add(missed_interrupt_count)
        .max(1);
    let multilingual_interrupt_recall_pct =
        (multilingual_candidates as f32 * 100.0) / multilingual_denominator as f32;

    let turn_prefix = format!("ph1k_runtime:{}:{}:", correlation_id.0, turn_id.0);
    let turn_rows = store
        .ph1k_runtime_event_rows()
        .iter()
        .filter(|row| row.idempotency_key.starts_with(&turn_prefix))
        .collect::<Vec<_>>();
    let mut required_kinds = vec![
        Ph1kRuntimeEventKind::StreamRefs,
        Ph1kRuntimeEventKind::VadEvent,
        Ph1kRuntimeEventKind::DeviceState,
        Ph1kRuntimeEventKind::TimingStats,
        Ph1kRuntimeEventKind::DegradationFlags,
        Ph1kRuntimeEventKind::TtsPlaybackActive,
    ];
    if bundle.interrupt_decision.candidate.is_some() {
        required_kinds.push(Ph1kRuntimeEventKind::InterruptCandidate);
    }
    required_kinds.sort();
    required_kinds.dedup();
    let present_required = required_kinds
        .iter()
        .filter(|kind| turn_rows.iter().any(|row| row.event_kind == **kind))
        .count();
    let audit_completeness_pct = if required_kinds.is_empty() {
        100.0
    } else {
        (present_required as f32 * 100.0) / required_kinds.len() as f32
    };
    let tenant_isolation_pct = if turn_rows.is_empty() {
        100.0
    } else {
        (turn_rows
            .iter()
            .filter(|row| row.tenant_id == tenant)
            .count() as f32
            * 100.0)
            / turn_rows.len() as f32
    };
    let line = format!(
        "{},{},{},{},{},{},{},{:.4},{},{},{},{:.4},{:.2},{},{},{},{:.2},{:.2},{:.2},{:.2}\n",
        captured_at,
        eval_commit_hash(),
        60,
        locale_tag,
        device_route,
        noise_class,
        overlap_speech,
        active_session_hours,
        interrupt_events,
        false_interrupt_count,
        missed_interrupt_count,
        false_interrupt_rate_per_hour,
        missed_interrupt_rate_pct,
        end_of_speech_p95_ms,
        capture_to_ph1c_handoff_p95_ms,
        device_failover_recovery_p95_ms,
        noisy_recovery_success_pct,
        multilingual_interrupt_recall_pct,
        audit_completeness_pct,
        tenant_isolation_pct,
    );
    file.write_all(line.as_bytes()).map_err(|err| {
        format!(
            "failed to append ph1k eval csv row '{}': {}",
            csv_path.display(),
            err
        )
    })?;
    file.flush().map_err(|err| {
        format!(
            "failed to flush ph1k eval csv '{}': {}",
            csv_path.display(),
            err
        )
    })?;

    let _ = (correlation_id, turn_id);
    Ok(())
}

fn ph1c_live_reject_summary(
    reason_code: ReasonCodeId,
    retry_advice: Ph1cRetryAdvice,
) -> Ph1cLiveTurnOutcomeSummary {
    Ph1cLiveTurnOutcomeSummary {
        response: Ph1cResponse::TranscriptReject(
            selene_kernel_contracts::ph1c::TranscriptReject::v1(reason_code, retry_advice),
        ),
        partial_text: None,
        final_text: None,
        finalized: false,
        low_latency_commit: false,
        provider_call_trace: Vec::new(),
    }
}

fn summarize_ph1c_stream_commit(
    stream_commit: Ph1cStreamCommit,
    provider_call_trace: Vec<Ph1dProviderCallResponse>,
) -> Ph1cLiveTurnOutcomeSummary {
    let final_text = match &stream_commit.response {
        Ph1cResponse::TranscriptOk(ok) => Some(ok.transcript_text.clone()),
        Ph1cResponse::TranscriptReject(_) => None,
    };
    let partial_text = stream_commit.partial_batch.as_ref().and_then(|batch| {
        batch
            .partials
            .last()
            .map(|partial| partial.text_chunk.clone())
    });
    Ph1cLiveTurnOutcomeSummary {
        response: stream_commit.response,
        partial_text,
        final_text,
        finalized: stream_commit.finalized,
        low_latency_commit: stream_commit.low_latency_commit,
        provider_call_trace,
    }
}

fn snapshot_provider_calls(
    records: &Arc<Mutex<Vec<Ph1dProviderCallResponse>>>,
) -> Vec<Ph1dProviderCallResponse> {
    records.lock().map(|rows| rows.clone()).unwrap_or_default()
}

#[allow(clippy::too_many_arguments)]
fn append_ph1c_live_telemetry_csv(
    now: MonotonicTimeNs,
    correlation_id: CorrelationId,
    turn_id: TurnId,
    tenant_id: &str,
    outcome_type: &str,
    reason_code: ReasonCodeId,
    latency_ms: u32,
    decision_delta: bool,
    finalized: bool,
    low_latency_commit: bool,
) -> Result<(), String> {
    let default_csv_path = resolve_repo_root_from_cwd()
        .map(|root| root.join(".dev/ph1c_live_telemetry.csv"))
        .unwrap_or_else(|| PathBuf::from(".dev/ph1c_live_telemetry.csv"));
    let csv_path = env::var("SELENE_PH1C_LIVE_TELEMETRY_PATH")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .unwrap_or(default_csv_path);
    if let Some(parent) = csv_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|err| {
                format!(
                    "failed to create telemetry csv directory '{}': {}",
                    parent.display(),
                    err
                )
            })?;
        }
    }
    let needs_header = !csv_path.exists()
        || fs::metadata(&csv_path)
            .map(|meta| meta.len() == 0)
            .unwrap_or(true);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&csv_path)
        .map_err(|err| {
            format!(
                "failed to open telemetry csv '{}' for append: {}",
                csv_path.display(),
                err
            )
        })?;
    if needs_header {
        file.write_all(
            b"captured_at_ns,correlation_id,turn_id,tenant_id,outcome_type,reason_code,latency_ms,decision_delta,finalized,low_latency_commit\n",
        )
        .map_err(|err| {
            format!(
                "failed to write telemetry csv header '{}': {}",
                csv_path.display(),
                err
            )
        })?;
    }
    let line = format!(
        "{},{},{},{},{},{},{},{},{},{}\n",
        now.0,
        correlation_id.0,
        turn_id.0,
        tenant_id,
        outcome_type,
        reason_code.0,
        latency_ms,
        if decision_delta { "1" } else { "0" },
        if finalized { "1" } else { "0" },
        if low_latency_commit { "1" } else { "0" }
    );
    file.write_all(line.as_bytes()).map_err(|err| {
        format!(
            "failed to append telemetry csv row '{}': {}",
            csv_path.display(),
            err
        )
    })?;
    Ok(())
}

fn ph1d_model_outcome_from_os_outcome(outcome: &OsVoiceLiveTurnOutcome) -> Ph1dModelCallOutcome {
    match outcome {
        OsVoiceLiveTurnOutcome::Forwarded(forwarded) => {
            let next_move = forwarded
                .top_level_bundle
                .os_bundle
                .decision_compute
                .next_move;
            if next_move == OsNextMove::Refuse {
                return Ph1dModelCallOutcome::SafetyBlock;
            }
            Ph1dModelCallOutcome::Ok {
                raw_json: ph1d_model_json_for_next_move(next_move),
            }
        }
        OsVoiceLiveTurnOutcome::Refused(_) => Ph1dModelCallOutcome::SafetyBlock,
        OsVoiceLiveTurnOutcome::NotInvokedDisabled => Ph1dModelCallOutcome::BudgetExceeded,
    }
}

fn ph1d_model_json_for_next_move(next_move: OsNextMove) -> String {
    match next_move {
        OsNextMove::Clarify => format!(
            r#"{{"mode":"clarify","question":"Could you clarify?","what_is_missing":["Task"],"accepted_answer_formats":["One short sentence","A few keywords"],"reason_code":{}}}"#,
            ph1d_reason_codes::D_CLARIFY_EVIDENCE_REQUIRED.0
        ),
        OsNextMove::DispatchTool | OsNextMove::DispatchSimulation | OsNextMove::Confirm => format!(
            r#"{{"mode":"intent","intent_type":"Continue","field_refinements":[],"missing_fields":[],"reason_code":{}}}"#,
            ph1d_reason_codes::D_PROVIDER_OK.0
        ),
        OsNextMove::Explain | OsNextMove::Wait => format!(
            r#"{{"mode":"analysis","short_analysis":"route:analysis_required","reason_code":{}}}"#,
            ph1d_reason_codes::D_PROVIDER_OK.0
        ),
        OsNextMove::Respond | OsNextMove::Refuse => format!(
            r#"{{"mode":"chat","response_text":"Acknowledged.","reason_code":{}}}"#,
            ph1d_reason_codes::D_PROVIDER_OK.0
        ),
    }
}

fn ph1d_fail_code(kind: Ph1dFailureKind) -> &'static str {
    match kind {
        Ph1dFailureKind::InvalidSchema => "D_FAIL_INVALID_SCHEMA",
        Ph1dFailureKind::ForbiddenOutput => "D_FAIL_FORBIDDEN_OUTPUT",
        Ph1dFailureKind::SafetyBlock => "D_FAIL_SAFETY_BLOCK",
        Ph1dFailureKind::Timeout => "D_FAIL_TIMEOUT",
        Ph1dFailureKind::BudgetExceeded => "D_FAIL_BUDGET_EXCEEDED",
    }
}

fn ph1c_language_locale(response: &Ph1cResponse) -> Option<String> {
    match response {
        Ph1cResponse::TranscriptOk(ok) => Some(ok.language_tag.as_str().to_string()),
        Ph1cResponse::TranscriptReject(_) => None,
    }
}

fn feedback_learn_pair_for_ph1c_capture(
    event_type: FeedbackEventType,
) -> Option<(&'static str, &'static str)> {
    match event_type {
        FeedbackEventType::SttReject => Some((
            feedback_event_type_str(FeedbackEventType::SttReject),
            learn_signal_type_str(LearnSignalType::SttReject),
        )),
        // Storage pair-lock currently learns STT retrys through the canonical STT reject signal lane.
        FeedbackEventType::SttRetry => Some((
            feedback_event_type_str(FeedbackEventType::SttReject),
            learn_signal_type_str(LearnSignalType::SttReject),
        )),
        _ => None,
    }
}

fn feedback_learn_pair_for_ph1d_capture(
    event_type: FeedbackEventType,
) -> Option<(&'static str, &'static str)> {
    match event_type {
        FeedbackEventType::SttReject => Some((
            feedback_event_type_str(FeedbackEventType::SttReject),
            learn_signal_type_str(LearnSignalType::SttReject),
        )),
        FeedbackEventType::ToolFail => Some((
            feedback_event_type_str(FeedbackEventType::ToolFail),
            learn_signal_type_str(LearnSignalType::ToolFail),
        )),
        _ => None,
    }
}

fn feedback_event_type_str(event_type: FeedbackEventType) -> &'static str {
    match event_type {
        FeedbackEventType::SttReject => "SttReject",
        FeedbackEventType::SttRetry => "SttRetry",
        FeedbackEventType::LanguageMismatch => "LanguageMismatch",
        FeedbackEventType::UserCorrection => "UserCorrection",
        FeedbackEventType::ClarifyLoop => "ClarifyLoop",
        FeedbackEventType::ConfirmAbort => "ConfirmAbort",
        FeedbackEventType::ToolFail => "ToolFail",
        FeedbackEventType::MemoryOverride => "MemoryOverride",
        FeedbackEventType::DeliverySwitch => "DeliverySwitch",
        FeedbackEventType::BargeIn => "BargeIn",
        FeedbackEventType::VoiceIdFalseReject => "VoiceIdFalseReject",
        FeedbackEventType::VoiceIdFalseAccept => "VoiceIdFalseAccept",
        FeedbackEventType::VoiceIdSpoofRisk => "VoiceIdSpoofRisk",
        FeedbackEventType::VoiceIdMultiSpeaker => "VoiceIdMultiSpeaker",
        FeedbackEventType::VoiceIdDriftAlert => "VoiceIdDriftAlert",
        FeedbackEventType::VoiceIdReauthFriction => "VoiceIdReauthFriction",
        FeedbackEventType::VoiceIdConfusionPair => "VoiceIdConfusionPair",
        FeedbackEventType::VoiceIdDrift => "VoiceIdDrift",
        FeedbackEventType::VoiceIdLowQuality => "VoiceIdLowQuality",
    }
}

fn learn_signal_type_str(signal_type: LearnSignalType) -> &'static str {
    match signal_type {
        LearnSignalType::SttReject => "SttReject",
        LearnSignalType::UserCorrection => "UserCorrection",
        LearnSignalType::ClarifyLoop => "ClarifyLoop",
        LearnSignalType::ToolFail => "ToolFail",
        LearnSignalType::VocabularyRepeat => "VocabularyRepeat",
        LearnSignalType::BargeIn => "BargeIn",
        LearnSignalType::DeliverySwitch => "DeliverySwitch",
        LearnSignalType::VoiceIdFalseReject => "VoiceIdFalseReject",
        LearnSignalType::VoiceIdFalseAccept => "VoiceIdFalseAccept",
        LearnSignalType::VoiceIdSpoofRisk => "VoiceIdSpoofRisk",
        LearnSignalType::VoiceIdMultiSpeaker => "VoiceIdMultiSpeaker",
        LearnSignalType::VoiceIdDriftAlert => "VoiceIdDriftAlert",
        LearnSignalType::VoiceIdReauthFriction => "VoiceIdReauthFriction",
        LearnSignalType::VoiceIdConfusionPair => "VoiceIdConfusionPair",
        LearnSignalType::VoiceIdDrift => "VoiceIdDrift",
        LearnSignalType::VoiceIdLowQuality => "VoiceIdLowQuality",
        LearnSignalType::WakeAccepted => "WakeAccepted",
        LearnSignalType::WakeRejected => "WakeRejected",
        LearnSignalType::FalseWake => "FalseWake",
        LearnSignalType::MissedWake => "MissedWake",
        LearnSignalType::LowConfidenceWake => "LowConfidenceWake",
        LearnSignalType::NoisyEnvironment => "NoisyEnvironment",
    }
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
    match raw
        .unwrap_or("UNRESOLVED_ESCALATED")
        .trim()
        .to_ascii_uppercase()
        .as_str()
    {
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

    struct HealthIssueEventSeed<'a> {
        tenant: &'a TenantId,
        now_ns: u64,
        issue_id: &'a str,
        engine_owner_id: &'a str,
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
    }

    fn add_event(out: &mut Vec<HealthIssueEvent>, seed: HealthIssueEventSeed<'_>) {
        let HealthIssueEventSeed {
            tenant,
            now_ns,
            issue_id,
            engine_owner_id,
            severity,
            status,
            reason_code,
            bcast_id,
            ack_state,
            impact_summary,
            attempted_fix_actions,
            current_monitoring_evidence,
            unresolved_reason_exact,
            issue_fingerprint,
            recurrence_observed,
        } = seed;
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
            Some(MonotonicTimeNs(
                now_ns.saturating_add(15 * 60 * 1_000_000_000),
            )),
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
                Some(MonotonicTimeNs(
                    now_ns.saturating_sub(5 * 60 * 1_000_000_000),
                )),
                Some(MonotonicTimeNs(now_ns)),
                recurrence_observed,
            )
            .unwrap_or(with_payload);
        out.push(full);
    }

    if health.sync.queue.dead_letter_count > 0 {
        add_event(
            &mut out,
            HealthIssueEventSeed {
                tenant,
                now_ns,
                issue_id: "sync_dead_letter",
                engine_owner_id: "PH1.OS",
                severity: HealthSeverity::Critical,
                status: HealthIssueStatus::Escalated,
                reason_code: reason_codes::ADAPTER_SYNC_DEADLETTER,
                bcast_id: Some("bcast_sync_dead_letter".to_string()),
                ack_state: Some(HealthAckState::Waiting),
                impact_summary: Some(
                    "Critical sync dead letters are blocking artifact continuity.".to_string(),
                ),
                attempted_fix_actions: vec!["retry queued artifacts".to_string()],
                current_monitoring_evidence: Some(format!(
                    "dead_letter_count={} replay_due_count={}",
                    health.sync.queue.dead_letter_count, health.sync.queue.replay_due_count
                )),
                unresolved_reason_exact: Some("dead letters remain after retry budget".to_string()),
                issue_fingerprint: Some("sync_dead_letter_fingerprint".to_string()),
                recurrence_observed: Some(true),
            },
        );
    }

    if health.sync.queue.retry_pending_count > 0 {
        add_event(
            &mut out,
            HealthIssueEventSeed {
                tenant,
                now_ns,
                issue_id: "sync_retry_backlog",
                engine_owner_id: "PH1.OS",
                severity: HealthSeverity::Warn,
                status: HealthIssueStatus::Open,
                reason_code: reason_codes::ADAPTER_SYNC_RETRY,
                bcast_id: None,
                ack_state: None,
                impact_summary: Some("Retry queue backlog is above zero.".to_string()),
                attempted_fix_actions: vec!["retry worker pass".to_string()],
                current_monitoring_evidence: Some(format!(
                    "retry_pending_count={}",
                    health.sync.queue.retry_pending_count
                )),
                unresolved_reason_exact: Some("retry queue has not drained yet".to_string()),
                issue_fingerprint: Some("sync_retry_fingerprint".to_string()),
                recurrence_observed: Some(true),
            },
        );
    }

    if health.sync.queue.replay_due_count > 0 {
        add_event(
            &mut out,
            HealthIssueEventSeed {
                tenant,
                now_ns,
                issue_id: "sync_replay_due",
                engine_owner_id: "PH1.OS",
                severity: HealthSeverity::Critical,
                status: HealthIssueStatus::Open,
                reason_code: reason_codes::ADAPTER_SYNC_REPLAY_DUE,
                bcast_id: None,
                ack_state: None,
                impact_summary: Some("Replay-due jobs exceeded threshold.".to_string()),
                attempted_fix_actions: vec!["replay scan".to_string()],
                current_monitoring_evidence: Some(format!(
                    "replay_due_count={}",
                    health.sync.queue.replay_due_count
                )),
                unresolved_reason_exact: Some("replay-due jobs remain unresolved".to_string()),
                issue_fingerprint: Some("sync_replay_due_fingerprint".to_string()),
                recurrence_observed: Some(true),
            },
        );
    }

    if out.is_empty() {
        add_event(
            &mut out,
            HealthIssueEventSeed {
                tenant,
                now_ns,
                issue_id: "health_nominal",
                engine_owner_id: "PH1.HEALTH",
                severity: HealthSeverity::Info,
                status: HealthIssueStatus::Resolved,
                reason_code: ReasonCodeId(health_reason_codes::PH1_HEALTH_OK_REPORT_QUERY_READ.0),
                bcast_id: None,
                ack_state: Some(HealthAckState::Acknowledged),
                impact_summary: Some("No active unresolved health issues.".to_string()),
                attempted_fix_actions: vec!["daily health scan".to_string()],
                current_monitoring_evidence: Some("no recurrence detected".to_string()),
                unresolved_reason_exact: Some("resolved by live verification".to_string()),
                issue_fingerprint: Some("health_nominal_fingerprint".to_string()),
                recurrence_observed: Some(false),
            },
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

fn filter_health_issues(
    issues: &[UiHealthIssueRow],
    filter: &UiHealthDetailFilter,
) -> Vec<UiHealthIssueRow> {
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
                if !issue
                    .engine_owner
                    .to_ascii_lowercase()
                    .contains(owner_filter)
                {
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
            blocker: Some("Retry queue backlog not drained.".to_string()),
            unresolved_deadline_at_ns: at_ns.map(|v| v.saturating_add(15 * 60 * 1_000_000_000)),
        });
        timeline.push(UiHealthTimelineEntry {
            issue_id: "sync_retry_backlog".to_string(),
            at_ns,
            action_id: "SYNC_WORKER_RETRY_PASS".to_string(),
            result: format!("retry_pending={}", health.sync.queue.retry_pending_count),
            reason_code: reason_codes::ADAPTER_SYNC_RETRY.0.to_string(),
            evidence_ref: Some("sync.queue.retry_pending_count".to_string()),
            blocker: Some("Retry queue backlog not drained.".to_string()),
            unresolved_deadline_at_ns: at_ns.map(|v| v.saturating_add(15 * 60 * 1_000_000_000)),
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
            blocker: Some("Dead-letter queue is non-zero.".to_string()),
            unresolved_deadline_at_ns: at_ns.map(|v| v.saturating_add(15 * 60 * 1_000_000_000)),
        });
        timeline.push(UiHealthTimelineEntry {
            issue_id: "sync_dead_letter".to_string(),
            at_ns,
            action_id: "SYNC_WORKER_DEADLETTER".to_string(),
            result: format!("dead_lettered={}", health.sync.queue.dead_letter_count),
            reason_code: reason_codes::ADAPTER_SYNC_DEADLETTER.0.to_string(),
            evidence_ref: Some("sync.queue.dead_letter_count".to_string()),
            blocker: Some("Dead-letter queue is non-zero.".to_string()),
            unresolved_deadline_at_ns: at_ns.map(|v| v.saturating_add(15 * 60 * 1_000_000_000)),
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
            blocker: Some("Replay-due queue exceeds threshold.".to_string()),
            unresolved_deadline_at_ns: at_ns.map(|v| v.saturating_add(15 * 60 * 1_000_000_000)),
        });
        timeline.push(UiHealthTimelineEntry {
            issue_id: "sync_replay_due".to_string(),
            at_ns,
            action_id: "SYNC_WORKER_REPLAY_DUE_SCAN".to_string(),
            result: format!("replay_due={}", health.sync.queue.replay_due_count),
            reason_code: reason_codes::ADAPTER_SYNC_REPLAY_DUE.0.to_string(),
            evidence_ref: Some("sync.queue.replay_due_count".to_string()),
            blocker: Some("Replay-due queue exceeds threshold.".to_string()),
            unresolved_deadline_at_ns: at_ns.map(|v| v.saturating_add(15 * 60 * 1_000_000_000)),
        });
    }

    if timeline.is_empty() {
        timeline.push(UiHealthTimelineEntry {
            issue_id: "sync_nominal".to_string(),
            at_ns,
            action_id: "SYNC_WORKER_PASS".to_string(),
            result: "NO_OPEN_ISSUES".to_string(),
            reason_code: "0".to_string(),
            evidence_ref: Some("sync.worker.last_pass_at_ns".to_string()),
            blocker: None,
            unresolved_deadline_at_ns: None,
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
            blocker: Some("Builder status is outside healthy range.".to_string()),
            unresolved_deadline_at_ns: at_ns.map(|v| v.saturating_add(15 * 60 * 1_000_000_000)),
        });
    }

    timeline.push(UiHealthTimelineEntry {
        issue_id: "builder_health".to_string(),
        at_ns,
        action_id: "BUILDER_STATUS_TRACK".to_string(),
        result: builder_status,
        reason_code: "0".to_string(),
        evidence_ref: Some("sync.improvement.last_builder_status".to_string()),
        blocker: if status == "HEALTHY" {
            None
        } else {
            Some("Builder status is outside healthy range.".to_string())
        },
        unresolved_deadline_at_ns: if status == "HEALTHY" {
            None
        } else {
            at_ns.map(|v| v.saturating_add(15 * 60 * 1_000_000_000))
        },
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
    use selene_engines::device_vault::DeviceVault;
    use selene_kernel_contracts::ph1_voice_id::{
        VOICE_ID_ENROLL_COMPLETE_COMMIT, VOICE_ID_ENROLL_SAMPLE_COMMIT, VOICE_ID_ENROLL_START_DRAFT,
    };
    use selene_kernel_contracts::ph1emocore::EMO_SIM_001;
    use selene_kernel_contracts::ph1link::{
        InviteeType, LINK_INVITE_DRAFT_UPDATE_COMMIT, LINK_INVITE_OPEN_ACTIVATE_COMMIT,
    };
    use selene_kernel_contracts::ph1m::{
        MemoryConfidence, MemoryConsent, MemoryKey, MemoryLayer, MemoryLedgerEvent,
        MemoryLedgerEventKind, MemoryProvenance, MemorySensitivityFlag, MemoryUsePolicy,
        MemoryValue,
    };
    use selene_kernel_contracts::ph1n::FieldKey;
    use selene_kernel_contracts::ph1onb::{
        ONB_ACCESS_INSTANCE_CREATE_COMMIT, ONB_COMPLETE_COMMIT,
        ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT, ONB_EMPLOYEE_SENDER_VERIFY_COMMIT,
        ONB_PRIMARY_DEVICE_CONFIRM_COMMIT, ONB_SESSION_START_DRAFT, ONB_TERMS_ACCEPT_COMMIT,
    };
    use selene_kernel_contracts::ph1position::TenantId;
    use selene_kernel_contracts::ph1rem::{
        Ph1RemRequest, Ph1RemResponse, ReminderChannel, ReminderLocalTimeMode,
        ReminderPriorityLevel, ReminderState, ReminderType, REMINDER_CANCEL_COMMIT,
        REMINDER_SCHEDULE_COMMIT,
    };
    use selene_kernel_contracts::ph1simcat::{
        SimulationCatalogEventInput, SimulationId, SimulationStatus, SimulationType,
        SimulationVersion,
    };
    use selene_kernel_contracts::ph1w::{
        WAKE_ENROLL_COMPLETE_COMMIT, WAKE_ENROLL_DEFER_COMMIT, WAKE_ENROLL_SAMPLE_COMMIT,
        WAKE_ENROLL_START_DRAFT,
    };
    use selene_kernel_contracts::ph1x::{
        PendingState, ThreadPolicyFlags, ThreadState as KernelThreadState,
    };
    use selene_storage::ph1f::{
        AccessDeviceTrustLevel, AccessLifecycleState, AccessMode, AccessVerificationLevel,
        DeviceRecord, IdentityRecord, IdentityStatus, MobileArtifactSyncKind, WakeSampleResult,
    };
    use std::ffi::OsString;
    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct ScopedEnvVar {
        key: &'static str,
        previous: Option<OsString>,
    }

    impl ScopedEnvVar {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var_os(key);
            std::env::set_var(key, value);
            Self { key, previous }
        }
    }

    impl Drop for ScopedEnvVar {
        fn drop(&mut self) {
            if let Some(value) = self.previous.as_ref() {
                std::env::set_var(self.key, value);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    fn with_isolated_device_vault<T>(
        label: &str,
        secrets: &[(&str, &str)],
        env_overrides: &[(&'static str, &'static str)],
        f: impl FnOnce() -> T,
    ) -> T {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let env_lock = ENV_LOCK.get_or_init(|| Mutex::new(()));
        let _guard = env_lock.lock().expect("env lock poisoned");
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time must be monotonic for tests")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("selene-vault-{label}-{nanos}.json"));
        let key_path = path.with_extension("master.key");
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&key_path);
        let vault = DeviceVault::for_paths(path.clone(), key_path);
        for (key, value) in secrets {
            vault
                .set_secret(key, value)
                .expect("test vault secret seed should succeed");
        }
        let path_text = path
            .to_str()
            .expect("temp path should be valid UTF-8 for test env var")
            .to_string();
        let mut scopes = Vec::new();
        scopes.push(ScopedEnvVar::set("SELENE_DEVICE_VAULT_PATH", &path_text));
        for (key, value) in env_overrides {
            scopes.push(ScopedEnvVar::set(key, value));
        }
        let out = f();
        drop(scopes);
        let _ = std::fs::remove_file(path);
        out
    }

    fn with_isolated_empty_device_vault<T>(label: &str, f: impl FnOnce() -> T) -> T {
        with_isolated_device_vault(label, &[], &[], f)
    }

    fn base_request() -> VoiceTurnAdapterRequest {
        VoiceTurnAdapterRequest {
            correlation_id: 10_001,
            turn_id: 20_001,
            device_turn_sequence: None,
            app_platform: "IOS".to_string(),
            platform_version: None,
            device_class: None,
            runtime_client_version: None,
            hardware_capability_profile: None,
            network_profile: None,
            claimed_capabilities: None,
            integrity_status: None,
            attestation_ref: None,
            trigger: "EXPLICIT".to_string(),
            actor_user_id: "tenant_a:user_adapter_test".to_string(),
            tenant_id: Some("tenant_a".to_string()),
            device_id: Some("adapter_device_1".to_string()),
            now_ns: Some(3),
            thread_key: None,
            project_id: None,
            pinned_context_refs: None,
            thread_policy_flags: None,
            user_text_partial: None,
            user_text_final: None,
            selene_text_partial: None,
            selene_text_final: None,
            audio_capture_ref: Some(VoiceTurnAudioCaptureRef {
                stream_id: 11,
                pre_roll_buffer_id: 1,
                t_start_ns: 1,
                t_end_ns: 3,
                t_candidate_start_ns: 2,
                t_confirmed_ns: 3,
                locale_tag: Some("en-US".to_string()),
                device_route: Some("BUILT_IN".to_string()),
                selected_mic: Some("ios_mic_default".to_string()),
                selected_speaker: Some("ios_speaker_default".to_string()),
                tts_playback_active: Some(true),
                detection_text: Some("stop".to_string()),
                detection_confidence_bp: Some(9_600),
                vad_confidence_bp: Some(9_400),
                acoustic_confidence_bp: Some(9_300),
                prosody_confidence_bp: Some(9_200),
                speech_likeness_bp: Some(9_500),
                echo_safe_confidence_bp: Some(9_100),
                nearfield_confidence_bp: Some(9_000),
                capture_degraded: Some(false),
                stream_gap_detected: Some(false),
                aec_unstable: Some(false),
                device_changed: Some(false),
                snr_db_milli: Some(22_000),
                clipping_ratio_bp: Some(80),
                echo_delay_ms_milli: Some(26_000),
                packet_loss_bp: Some(25),
                double_talk_bp: Some(400),
                erle_db_milli: Some(20_000),
                device_failures_24h: Some(0),
                device_recoveries_24h: Some(0),
                device_mean_recovery_ms: Some(100),
                device_reliability_bp: Some(9_900),
                timing_jitter_ms_milli: Some(7_000),
                timing_drift_ppm_milli: Some(3_000),
                timing_buffer_depth_ms_milli: Some(35_000),
                timing_underruns: Some(0),
                timing_overruns: Some(0),
            }),
            visual_input_ref: None,
        }
    }

    fn base_tablet_request() -> VoiceTurnAdapterRequest {
        let mut request = base_request();
        request.app_platform = "TABLET".to_string();
        request.device_class = Some("TABLET".to_string());
        request.platform_version = Some("15.2".to_string());
        request.runtime_client_version = Some("2.3.4".to_string());
        request.hardware_capability_profile = Some("TABLET_PRO".to_string());
        request.network_profile = Some("STANDARD".to_string());
        request.claimed_capabilities = Some(vec![
            "MICROPHONE".to_string(),
            "CAMERA".to_string(),
            "SPEAKER_OUTPUT".to_string(),
            "WAKE_WORD".to_string(),
            "SENSOR_AVAILABILITY".to_string(),
        ]);
        request.integrity_status = Some("ATTESTED".to_string());
        request.attestation_ref = Some("tablet_attest_ref_01".to_string());
        request
    }

    fn mark_request_as_attested_capture(request: &mut VoiceTurnAdapterRequest) {
        request.integrity_status = Some("ATTESTED".to_string());
        request.attestation_ref = Some(format!(
            "attest:{}:{}:{}",
            request.app_platform.to_ascii_lowercase(),
            request.correlation_id,
            request.turn_id
        ));
        if let Some(capture) = request.audio_capture_ref.as_mut() {
            let end_ns = request.now_ns.unwrap_or(capture.t_end_ns).max(1);
            capture.t_end_ns = end_ns;
            capture.t_start_ns = end_ns.saturating_sub(2).max(1);
            capture.t_candidate_start_ns = end_ns.saturating_sub(1).max(1);
            capture.t_confirmed_ns = end_ns;
            capture.selected_mic = Some(
                capture
                    .selected_mic
                    .clone()
                    .unwrap_or_else(|| "trusted_capture_mic".to_string()),
            );
            capture.device_route = Some(
                capture
                    .device_route
                    .clone()
                    .unwrap_or_else(|| "BUILT_IN".to_string()),
            );
            capture.capture_degraded = Some(false);
            capture.stream_gap_detected = Some(false);
            capture.device_changed = Some(false);
        }
    }

    fn temp_persistence_journal_path(label: &str) -> PathBuf {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be >= unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("selene_adapter_{label}_{seed}.jsonl"));
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(adapter_persistence_state_path(&path));
        path
    }

    fn read_persistence_state_for_test(journal_path: &Path) -> AdapterPersistenceState {
        let state_path = adapter_persistence_state_path(journal_path);
        let raw = std::fs::read_to_string(&state_path).expect("persistence state must be readable");
        if raw.trim().is_empty() {
            AdapterPersistenceState::default()
        } else {
            serde_json::from_str(&raw).expect("persistence state must decode")
        }
    }

    fn write_persistence_state_for_test(journal_path: &Path, state: &AdapterPersistenceState) {
        let state_path = adapter_persistence_state_path(journal_path);
        let json =
            serde_json::to_vec_pretty(state).expect("persistence state must encode for test");
        std::fs::write(&state_path, json).expect("persistence state must be writable");
    }

    fn cleanup_persistence_files_for_test(journal_path: &Path) {
        let _ = std::fs::remove_file(journal_path);
        let state_path = adapter_persistence_state_path(journal_path);
        let _ = std::fs::remove_file(&state_path);
        let _ = std::fs::remove_file(quarantined_persistence_path(&state_path, "state_corrupt"));
        let _ = std::fs::remove_file(quarantined_persistence_path(&state_path, "state_integrity"));
        let _ = std::fs::remove_file(quarantined_persistence_path(
            journal_path,
            "legacy_journal_corrupt",
        ));
    }

    fn with_scoped_runtime_node_id<T>(node_id: &str, f: impl FnOnce() -> T) -> T {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let env_lock = ENV_LOCK.get_or_init(|| Mutex::new(()));
        let _guard = env_lock.lock().expect("env lock poisoned");
        let scope = ScopedEnvVar::set("SELENE_RUNTIME_NODE_ID", node_id);
        let out = f();
        drop(scope);
        out
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

    fn seed_identity_and_device(store: &mut Ph1fStore, user_id: &UserId, device_id: &DeviceId) {
        store
            .insert_identity(IdentityRecord::v1(
                user_id.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        store
            .insert_device(
                DeviceRecord::v1(
                    device_id.clone(),
                    user_id.clone(),
                    "phone".to_string(),
                    MonotonicTimeNs(1),
                    None,
                )
                .unwrap(),
            )
            .unwrap();
    }

    fn seed_identity_and_device_for_request(
        runtime: &AdapterRuntime,
        request: &VoiceTurnAdapterRequest,
    ) {
        let actor_user_id =
            UserId::new(request.actor_user_id.clone()).expect("actor id must parse");
        let device_id = DeviceId::new(
            request
                .device_id
                .clone()
                .expect("device id must be present for seed"),
        )
        .expect("device id must parse");
        let mut store = runtime.store.lock().expect("store lock must not poison");
        if store.get_identity(&actor_user_id).is_none() {
            store
                .insert_identity(IdentityRecord::v1(
                    actor_user_id.clone(),
                    None,
                    None,
                    MonotonicTimeNs(1),
                    IdentityStatus::Active,
                ))
                .expect("identity seed must succeed");
        }
        if store.get_device(&device_id).is_none() {
            store
                .insert_device(
                    DeviceRecord::v1(
                        device_id,
                        actor_user_id,
                        "phone".to_string(),
                        MonotonicTimeNs(1),
                        None,
                    )
                    .expect("device seed must validate"),
                )
                .expect("device seed must succeed");
        }
    }

    fn seed_wake_enrollment_complete_for_request(
        runtime: &AdapterRuntime,
        request: &mut VoiceTurnAdapterRequest,
        label: &str,
    ) {
        mark_request_as_attested_capture(request);
        let actor_user_id =
            UserId::new(request.actor_user_id.clone()).expect("actor_user_id must parse");
        let app_platform =
            parse_app_platform(&request.app_platform).expect("app platform must parse");
        let device_id = DeviceId::new(
            request
                .device_id
                .clone()
                .expect("device_id must be present for wake enrollment seeding"),
        )
        .expect("device id must parse");
        let mut store = runtime.store.lock().expect("store lock must not poison");
        let now = MonotonicTimeNs(request.now_ns.unwrap_or(1).saturating_add(100));
        ensure_actor_identity_and_device(
            &mut store,
            &actor_user_id,
            Some(&device_id),
            app_platform,
            now,
            true,
        )
        .expect("identity/device seed must succeed");
        let started = store
            .ph1w_enroll_start_draft(
                now,
                actor_user_id.clone(),
                device_id.clone(),
                None,
                3,
                8,
                180_000,
                format!("{label}_wake_start"),
            )
            .expect("wake enrollment start should succeed");
        for seq in 1_u16..=3_u16 {
            store
                .ph1w_enroll_sample_commit(
                    MonotonicTimeNs(now.0.saturating_add(seq as u64)),
                    started.wake_enrollment_session_id.clone(),
                    1_200,
                    0.95,
                    18.0,
                    0.02,
                    -20.0,
                    -45.0,
                    -5.0,
                    0.0,
                    WakeSampleResult::Pass,
                    None,
                    format!("{label}_wake_sample_{seq}"),
                )
                .expect("wake enrollment sample should succeed");
        }
        store
            .ph1w_enroll_complete_commit(
                MonotonicTimeNs(now.0.saturating_add(10)),
                started.wake_enrollment_session_id,
                format!(
                    "wake_profile_{}_{}",
                    label,
                    stable_hash_hex_16(actor_user_id.as_str())
                ),
                format!("{label}_wake_complete"),
            )
            .expect("wake enrollment complete should succeed");
    }

    fn feedback_event_type_matches(
        row: &selene_kernel_contracts::ph1j::AuditEvent,
        expected: &str,
    ) -> bool {
        let key = selene_kernel_contracts::ph1j::PayloadKey::new("feedback_event_type")
            .expect("feedback_event_type key is valid");
        row.payload_min
            .entries
            .get(&key)
            .map(|value| value.as_str() == expected)
            .unwrap_or(false)
    }

    fn seed_simulation_catalog_status(
        store: &mut Ph1fStore,
        tenant: &str,
        simulation_id: &str,
        simulation_type: SimulationType,
        status: SimulationStatus,
    ) {
        let event = SimulationCatalogEventInput::v1(
            MonotonicTimeNs(1),
            TenantId::new(tenant.to_string()).unwrap(),
            SimulationId::new(simulation_id.to_string()).unwrap(),
            SimulationVersion(1),
            simulation_type,
            status,
            "PH1.TEST".to_string(),
            "reads_v1".to_string(),
            "writes_v1".to_string(),
            ReasonCodeId(1),
            None,
        )
        .unwrap();
        store.append_simulation_catalog_event(event).unwrap();
    }

    fn seed_calendar_access_instance(store: &mut Ph1fStore, actor: &UserId, tenant: &str) {
        store
            .ph2access_upsert_instance_commit(
                MonotonicTimeNs(1),
                tenant.to_string(),
                actor.clone(),
                "role.owner".to_string(),
                AccessMode::A,
                "{\"allow\":[\"CALENDAR_EVENT_CREATE\"]}".to_string(),
                true,
                AccessVerificationLevel::PasscodeTime,
                AccessDeviceTrustLevel::Dtl4,
                AccessLifecycleState::Active,
                "policy_snapshot_v1".to_string(),
                None,
            )
            .unwrap();
    }

    fn seed_reminder_access_instance(store: &mut Ph1fStore, actor: &UserId, tenant: &str) {
        store
            .ph2access_upsert_instance_commit(
                MonotonicTimeNs(1),
                tenant.to_string(),
                actor.clone(),
                "role.owner".to_string(),
                AccessMode::A,
                "{\"allow\":[\"REMINDER_SET\",\"REMINDER_UPDATE\",\"REMINDER_CANCEL\",\"CALENDAR_EVENT_CREATE\"]}".to_string(),
                true,
                AccessVerificationLevel::PasscodeTime,
                AccessDeviceTrustLevel::Dtl4,
                AccessLifecycleState::Active,
                "policy_snapshot_v1".to_string(),
                None,
            )
            .unwrap();
    }

    fn seed_invite_link_for_click(
        store: &mut Ph1fStore,
        inviter_user_id: &UserId,
    ) -> (String, String) {
        let now = MonotonicTimeNs(system_time_now_ns().max(1));
        let (link, _) = store
            .ph1link_invite_generate_draft(
                now,
                inviter_user_id.clone(),
                InviteeType::Employee,
                Some("tenant_1".to_string()),
                None,
                None,
                None,
            )
            .unwrap();
        (
            link.token_id.as_str().to_string(),
            link.token_signature.clone(),
        )
    }

    fn seed_invite_link_for_click_with_employee_prefilled_context(
        store: &mut Ph1fStore,
        inviter_user_id: &UserId,
    ) -> (String, String) {
        let now = MonotonicTimeNs(system_time_now_ns().max(1));
        let prefilled = selene_kernel_contracts::ph1link::PrefilledContext::v1(
            Some("tenant_1".to_string()),
            Some("company_1".to_string()),
            Some("position_1".to_string()),
            Some("loc_1".to_string()),
            Some("2026-03-01".to_string()),
            None,
            Some("band_l2".to_string()),
            vec!["US".to_string()],
        )
        .unwrap();
        let (link, _) = store
            .ph1link_invite_generate_draft(
                now,
                inviter_user_id.clone(),
                InviteeType::Employee,
                Some("tenant_1".to_string()),
                None,
                Some(prefilled),
                None,
            )
            .unwrap();
        (
            link.token_id.as_str().to_string(),
            link.token_signature.clone(),
        )
    }

    fn seed_employee_company_and_position(store: &mut Ph1fStore) {
        let tenant_id = TenantId::new("tenant_1".to_string()).unwrap();
        store
            .ph1tenant_company_upsert(selene_storage::ph1f::TenantCompanyRecord {
                schema_version: selene_kernel_contracts::SchemaVersion(1),
                tenant_id: tenant_id.clone(),
                company_id: "company_1".to_string(),
                legal_name: "Selene Co".to_string(),
                jurisdiction: "US".to_string(),
                lifecycle_state: selene_storage::ph1f::TenantCompanyLifecycleState::Active,
                created_at: MonotonicTimeNs(1),
                updated_at: MonotonicTimeNs(1),
            })
            .unwrap();
        let position = selene_kernel_contracts::ph1position::PositionRecord::v1(
            tenant_id.clone(),
            "company_1".to_string(),
            selene_kernel_contracts::ph1position::PositionId::new("position_1").unwrap(),
            "Operator".to_string(),
            "Operations".to_string(),
            "US".to_string(),
            selene_kernel_contracts::ph1position::PositionScheduleType::FullTime,
            "profile_ops".to_string(),
            "band_l2".to_string(),
            selene_kernel_contracts::ph1position::PositionLifecycleState::Active,
            MonotonicTimeNs(1),
            MonotonicTimeNs(1),
        )
        .unwrap();
        store.ph1position_upsert(position).unwrap();
    }

    fn seed_employee_position_schema_requiring_sender_verification(
        store: &mut Ph1fStore,
        actor_user_id: &UserId,
    ) {
        let tenant_id = TenantId::new("tenant_1".to_string()).unwrap();
        let selector = selene_kernel_contracts::ph1position::PositionSchemaSelectorSnapshot {
            company_size: Some("SMALL".to_string()),
            industry_code: Some("LOGISTICS".to_string()),
            jurisdiction: Some("US".to_string()),
            position_family: Some("OPS".to_string()),
        };
        let field = selene_kernel_contracts::ph1position::PositionRequirementFieldSpec {
            field_key: "working_hours".to_string(),
            field_type: selene_kernel_contracts::ph1position::PositionRequirementFieldType::String,
            required_rule:
                selene_kernel_contracts::ph1position::PositionRequirementRuleType::Always,
            required_predicate_ref: None,
            validation_ref: None,
            sensitivity:
                selene_kernel_contracts::ph1position::PositionRequirementSensitivity::Private,
            exposure_rule:
                selene_kernel_contracts::ph1position::PositionRequirementExposureRule::InternalOnly,
            evidence_mode:
                selene_kernel_contracts::ph1position::PositionRequirementEvidenceMode::DocRequired,
            prompt_short: "Provide working hours".to_string(),
            prompt_long: "Please provide working hours evidence.".to_string(),
        };
        store
            .ph1position_requirements_schema_create_draft(
                MonotonicTimeNs(2),
                actor_user_id.clone(),
                tenant_id.clone(),
                "company_1".to_string(),
                selene_kernel_contracts::ph1position::PositionId::new("position_1").unwrap(),
                "schema_v1".to_string(),
                selector,
                vec![field],
                "adapter-onb-schema-create".to_string(),
                "POSITION_REQUIREMENTS_SCHEMA_CREATE_DRAFT",
                ReasonCodeId(0x5900_0006),
            )
            .unwrap();
        store
            .ph1position_requirements_schema_activate_commit(
                MonotonicTimeNs(3),
                actor_user_id.clone(),
                tenant_id,
                "company_1".to_string(),
                selene_kernel_contracts::ph1position::PositionId::new("position_1").unwrap(),
                "schema_v1".to_string(),
                selene_kernel_contracts::ph1position::PositionSchemaApplyScope::NewHiresOnly,
                "adapter-onb-schema-activate".to_string(),
                "POSITION_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT",
                ReasonCodeId(0x5900_0008),
            )
            .unwrap();
    }

    #[test]
    fn at_adapter_vision_01_build_vision_turn_input_accepts_visual_source_and_tokens() {
        let mut request = base_request();
        request.visual_input_ref = Some(VoiceTurnVisualInputRef {
            turn_opt_in_enabled: true,
            source_id: Some("vision_source_adapter_1".to_string()),
            source_kind: Some("IMAGE".to_string()),
            image_ref: Some("image://invoice_capture_001".to_string()),
            blob_ref: Some("blob://capture/invoice_001".to_string()),
            visible_tokens: vec![],
        });
        let input = build_vision_turn_input_from_adapter_request(
            &request,
            CorrelationId(request.correlation_id as u128),
            TurnId(request.turn_id),
        )
        .unwrap()
        .expect("vision input should be present");
        assert_eq!(
            input.source_ref.source_id.as_str(),
            "vision_source_adapter_1"
        );
        assert!(input.visible_tokens.is_empty());
    }

    #[test]
    fn at_adapter_vision_02_skips_visual_turn_without_opt_in() {
        let mut request = base_request();
        request.visual_input_ref = Some(VoiceTurnVisualInputRef {
            turn_opt_in_enabled: false,
            source_id: Some("vision_source_adapter_2".to_string()),
            source_kind: Some("IMAGE".to_string()),
            image_ref: Some("image://invoice_capture_002".to_string()),
            blob_ref: None,
            visible_tokens: vec![VoiceTurnVisualTokenRef {
                token: "invoice".to_string(),
                x: None,
                y: None,
                w: None,
                h: None,
            }],
        });
        let input = build_vision_turn_input_from_adapter_request(
            &request,
            CorrelationId(request.correlation_id as u128),
            TurnId(request.turn_id),
        )
        .unwrap();
        assert!(input.is_none());
    }

    #[test]
    fn run2_desktop_request_builder_sets_runtime_tenant_for_nlp() {
        let mut request = base_request();
        request.app_platform = "DESKTOP".to_string();
        request.tenant_id = None;
        request.actor_user_id = "tenant_a:user_adapter_test".to_string();
        request.user_text_final =
            Some("Selene send a link to Tom for tenant tenant_999".to_string());

        let actor_user_id =
            UserId::new(request.actor_user_id.clone()).expect("actor user id must parse");
        let runtime_tenant_scope =
            resolve_tenant_scope(request.tenant_id.clone(), &actor_user_id, None);
        let nlp_req = build_base_nlp_request_for_vision_handoff(
            &request,
            request.user_text_final.as_deref(),
            runtime_tenant_scope.as_deref(),
        )
        .expect("nlp request builder should succeed");
        assert_eq!(nlp_req.runtime_tenant_id.as_deref(), Some("tenant_a"));

        let nlp_rt = EnginePh1nRuntime::new(EnginePh1nConfig::mvp_v1());
        let out = nlp_rt.run(&nlp_req).expect("nlp run should succeed");
        match out {
            Ph1nResponse::IntentDraft(d) => {
                assert_eq!(
                    d.intent_type,
                    selene_kernel_contracts::ph1n::IntentType::CreateInviteLink
                );
                let tenant = d
                    .fields
                    .iter()
                    .find(|f| f.key == selene_kernel_contracts::ph1n::FieldKey::TenantId)
                    .expect("invite intent should carry runtime tenant");
                assert_eq!(tenant.value.original_span, "tenant_a");
            }
            _ => panic!("expected invite intent draft"),
        }
    }

    #[test]
    fn run1_invite_click_adapter_starts_onboarding_without_turn_or_client_time_inputs() {
        let runtime = AdapterRuntime::default();
        let inviter_user_id = UserId::new("tenant_1:run1_adapter_inviter").unwrap();
        let inviter_device_id = DeviceId::new("run1_adapter_inviter_device").unwrap();

        let (token_id, token_signature) = {
            let mut store = runtime.store.lock().expect("adapter store lock");
            seed_identity_and_device(&mut store, &inviter_user_id, &inviter_device_id);
            seed_simulation_catalog_status(
                &mut store,
                "tenant_1",
                LINK_INVITE_OPEN_ACTIVATE_COMMIT,
                SimulationType::Commit,
                SimulationStatus::Active,
            );
            seed_simulation_catalog_status(
                &mut store,
                "tenant_1",
                ONB_SESSION_START_DRAFT,
                SimulationType::Draft,
                SimulationStatus::Active,
            );
            seed_invite_link_for_click(&mut store, &inviter_user_id)
        };

        let response = runtime
            .run_invite_link_open_and_start_onboarding(InviteLinkOpenAdapterRequest {
                correlation_id: 71_001,
                idempotency_key: "run1-invite-click-adapter-1".to_string(),
                token_id,
                token_signature,
                tenant_id: Some("tenant_1".to_string()),
                app_platform: "IOS".to_string(),
                device_fingerprint: "run1_adapter_fp".to_string(),
                app_instance_id: "ios_instance_run1_adapter".to_string(),
                deep_link_nonce: "nonce_run1_adapter".to_string(),
            })
            .expect("invite click should start onboarding");

        assert_eq!(response.status, "ok");
        assert_eq!(response.outcome, "ONBOARDING_STARTED");
        assert!(response.onboarding_session_id.is_some());
        assert_eq!(response.next_step.as_deref(), Some("ASK_MISSING"));
        assert!(!response.required_fields.is_empty());
    }

    #[test]
    fn run1_invite_click_adapter_fails_closed_for_bad_signature() {
        let runtime = AdapterRuntime::default();
        let inviter_user_id = UserId::new("tenant_1:run1_adapter_inviter_sig").unwrap();
        let inviter_device_id = DeviceId::new("run1_adapter_inviter_device_sig").unwrap();

        let token_id = {
            let mut store = runtime.store.lock().expect("adapter store lock");
            seed_identity_and_device(&mut store, &inviter_user_id, &inviter_device_id);
            seed_simulation_catalog_status(
                &mut store,
                "tenant_1",
                LINK_INVITE_OPEN_ACTIVATE_COMMIT,
                SimulationType::Commit,
                SimulationStatus::Active,
            );
            seed_simulation_catalog_status(
                &mut store,
                "tenant_1",
                ONB_SESSION_START_DRAFT,
                SimulationType::Draft,
                SimulationStatus::Active,
            );
            let (token_id, _signature) = seed_invite_link_for_click(&mut store, &inviter_user_id);
            token_id
        };

        let err = runtime
            .run_invite_link_open_and_start_onboarding(InviteLinkOpenAdapterRequest {
                correlation_id: 71_002,
                idempotency_key: "run1-invite-click-adapter-2".to_string(),
                token_id,
                token_signature: "v1.link_kid_v1.invalid".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                app_platform: "IOS".to_string(),
                device_fingerprint: "run1_adapter_fp_bad".to_string(),
                app_instance_id: "ios_instance_run1_adapter_bad".to_string(),
                deep_link_nonce: "nonce_run1_adapter_bad".to_string(),
            })
            .expect_err("bad signature must fail closed");
        assert!(err.contains("TOKEN_SIGNATURE_INVALID"));
    }

    #[test]
    fn runc_onboarding_continue_adapter_progresses_terms_device_voice() {
        let runtime = AdapterRuntime::default();
        let inviter_user_id = UserId::new("tenant_1:runc_adapter_inviter").unwrap();
        let inviter_device_id = DeviceId::new("runc_adapter_inviter_device").unwrap();

        let (token_id, token_signature) = {
            let mut store = runtime.store.lock().expect("adapter store lock");
            seed_identity_and_device(&mut store, &inviter_user_id, &inviter_device_id);
            seed_employee_company_and_position(&mut store);
            for (simulation_id, simulation_type) in [
                (LINK_INVITE_OPEN_ACTIVATE_COMMIT, SimulationType::Commit),
                (ONB_SESSION_START_DRAFT, SimulationType::Draft),
                (LINK_INVITE_DRAFT_UPDATE_COMMIT, SimulationType::Commit),
                (ONB_TERMS_ACCEPT_COMMIT, SimulationType::Commit),
                (ONB_PRIMARY_DEVICE_CONFIRM_COMMIT, SimulationType::Commit),
                (VOICE_ID_ENROLL_START_DRAFT, SimulationType::Draft),
                (VOICE_ID_ENROLL_SAMPLE_COMMIT, SimulationType::Commit),
                (VOICE_ID_ENROLL_COMPLETE_COMMIT, SimulationType::Commit),
                (EMO_SIM_001, SimulationType::Commit),
                (ONB_ACCESS_INSTANCE_CREATE_COMMIT, SimulationType::Commit),
                (ONB_COMPLETE_COMMIT, SimulationType::Commit),
            ] {
                seed_simulation_catalog_status(
                    &mut store,
                    "tenant_1",
                    simulation_id,
                    simulation_type,
                    SimulationStatus::Active,
                );
            }
            seed_invite_link_for_click(&mut store, &inviter_user_id)
        };

        let start = runtime
            .run_invite_link_open_and_start_onboarding(InviteLinkOpenAdapterRequest {
                correlation_id: 72_001,
                idempotency_key: "runc-adapter-start".to_string(),
                token_id,
                token_signature,
                tenant_id: Some("tenant_1".to_string()),
                app_platform: "IOS".to_string(),
                device_fingerprint: "runc_adapter_fp".to_string(),
                app_instance_id: "ios_instance_runc_adapter".to_string(),
                deep_link_nonce: "nonce_runc_adapter".to_string(),
            })
            .expect("invite click should start onboarding");
        assert_eq!(start.next_step.as_deref(), Some("ASK_MISSING"));
        let onboarding_session_id = start
            .onboarding_session_id
            .expect("onboarding session id must be present");

        let ask_prompt = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 72_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runc-adapter-ask-prompt-1".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "ASK_MISSING_SUBMIT".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: None,
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect("first ask-missing turn should prompt");
        assert_eq!(ask_prompt.next_step.as_deref(), Some("ASK_MISSING"));

        let mut ask_out = ask_prompt;
        for idx in 0..8 {
            if ask_out.next_step.as_deref() != Some("ASK_MISSING") {
                break;
            }
            let field_key = ask_out
                .blocking_field
                .clone()
                .expect("blocking field must be returned");
            let field_value = match field_key.as_str() {
                "tenant_id" => "tenant_1",
                "company_id" => "company_1",
                "position_id" => "position_1",
                "location_id" => "loc_1",
                "start_date" => "2026-03-01",
                "working_hours" => "09:00-17:00",
                "compensation_tier_ref" => "band_l2",
                "jurisdiction_tags" => "US,CA",
                _ => "value_1",
            };
            ask_out = runtime
                .run_onboarding_continue(OnboardingContinueAdapterRequest {
                    correlation_id: 72_001,
                    onboarding_session_id: onboarding_session_id.clone(),
                    idempotency_key: format!("runc-adapter-ask-value-{idx}"),
                    tenant_id: Some("tenant_1".to_string()),
                    action: "ASK_MISSING_SUBMIT".to_string(),
                    field_value: Some(field_value.to_string()),
                    receipt_kind: None,
                    receipt_ref: None,
                    signer: None,
                    payload_hash: None,
                    terms_version_id: None,
                    accepted: None,
                    device_id: None,
                    proof_ok: None,
                    sample_seed: None,
                    photo_blob_ref: None,
                    sender_decision: None,
                })
                .expect("ask-missing value submit should succeed");
        }
        assert_eq!(ask_out.next_step.as_deref(), Some("PLATFORM_SETUP"));
        assert!(!ask_out.remaining_platform_receipt_kinds.is_empty());

        let required_receipts = ask_out.remaining_platform_receipt_kinds.clone();
        let mut platform_out = ask_out;
        for (idx, receipt_kind) in required_receipts.iter().enumerate() {
            platform_out = runtime
                .run_onboarding_continue(OnboardingContinueAdapterRequest {
                    correlation_id: 72_001,
                    onboarding_session_id: onboarding_session_id.clone(),
                    idempotency_key: format!("runc-adapter-platform-{idx}"),
                    tenant_id: Some("tenant_1".to_string()),
                    action: "PLATFORM_SETUP_RECEIPT".to_string(),
                    field_value: None,
                    receipt_kind: Some(receipt_kind.clone()),
                    receipt_ref: Some(format!("receipt:runc-adapter:{receipt_kind}")),
                    signer: Some("selene_mobile_app".to_string()),
                    payload_hash: Some(format!("{:064x}", idx + 1)),
                    terms_version_id: None,
                    accepted: None,
                    device_id: None,
                    proof_ok: None,
                    sample_seed: None,
                    photo_blob_ref: None,
                    sender_decision: None,
                })
                .expect("platform setup receipt should succeed");
        }
        assert_eq!(platform_out.next_step.as_deref(), Some("TERMS"));
        assert!(platform_out.remaining_platform_receipt_kinds.is_empty());

        let terms = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 72_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runc-adapter-terms".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "TERMS_ACCEPT".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: Some("terms_v1".to_string()),
                accepted: Some(true),
                device_id: None,
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect("terms should succeed");
        assert_eq!(terms.next_step.as_deref(), Some("PRIMARY_DEVICE_CONFIRM"));

        let device = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 72_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runc-adapter-device".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "PRIMARY_DEVICE_CONFIRM".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: Some("runc_adapter_inviter_device".to_string()),
                proof_ok: Some(true),
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect("device confirm should succeed");
        assert_eq!(device.next_step.as_deref(), Some("VOICE_ENROLL"));

        let voice = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 72_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runc-adapter-voice".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "VOICE_ENROLL_LOCK".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: Some("runc_adapter_inviter_device".to_string()),
                proof_ok: None,
                sample_seed: Some("runc_adapter_seed".to_string()),
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect("voice enroll should succeed");
        assert_eq!(voice.next_step.as_deref(), Some("EMO_PERSONA_LOCK"));
        assert!(voice.voice_artifact_sync_receipt_ref.is_some());

        let emo = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 72_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runc-adapter-emo".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "EMO_PERSONA_LOCK".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: None,
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect("emo/persona lock should succeed");
        assert_eq!(emo.next_step.as_deref(), Some("ACCESS_PROVISION"));

        let access = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 72_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runc-adapter-access".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "ACCESS_PROVISION_COMMIT".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: None,
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect("access provision should succeed");
        assert_eq!(access.next_step.as_deref(), Some("COMPLETE"));
        assert_eq!(
            access.onboarding_status.as_deref(),
            Some("ACCESSINSTANCECREATED")
        );
        assert!(access.access_engine_instance_id.is_some());

        let complete = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 72_001,
                onboarding_session_id,
                idempotency_key: "runc-adapter-complete".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "COMPLETE_COMMIT".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: None,
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect("onboarding complete should succeed");
        assert_eq!(complete.next_step.as_deref(), Some("READY"));
        assert_eq!(complete.onboarding_status.as_deref(), Some("COMPLETE"));
        assert!(complete.access_engine_instance_id.is_some());
        assert!(complete.voice_artifact_sync_receipt_ref.is_some());
    }

    #[test]
    fn runh_onboarding_continue_adapter_sender_verification_progresses_to_ready() {
        let runtime = AdapterRuntime::default();
        let inviter_user_id = UserId::new("tenant_1:runh_adapter_inviter").unwrap();
        let inviter_device_id = DeviceId::new("runh_adapter_inviter_device").unwrap();

        let (token_id, token_signature) = {
            let mut store = runtime.store.lock().expect("adapter store lock");
            seed_identity_and_device(&mut store, &inviter_user_id, &inviter_device_id);
            seed_employee_company_and_position(&mut store);
            seed_employee_position_schema_requiring_sender_verification(
                &mut store,
                &inviter_user_id,
            );
            for (simulation_id, simulation_type) in [
                (LINK_INVITE_OPEN_ACTIVATE_COMMIT, SimulationType::Commit),
                (ONB_SESSION_START_DRAFT, SimulationType::Draft),
                (LINK_INVITE_DRAFT_UPDATE_COMMIT, SimulationType::Commit),
                (ONB_TERMS_ACCEPT_COMMIT, SimulationType::Commit),
                (
                    ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT,
                    SimulationType::Commit,
                ),
                (ONB_EMPLOYEE_SENDER_VERIFY_COMMIT, SimulationType::Commit),
                (ONB_PRIMARY_DEVICE_CONFIRM_COMMIT, SimulationType::Commit),
                (VOICE_ID_ENROLL_START_DRAFT, SimulationType::Draft),
                (VOICE_ID_ENROLL_SAMPLE_COMMIT, SimulationType::Commit),
                (VOICE_ID_ENROLL_COMPLETE_COMMIT, SimulationType::Commit),
                (EMO_SIM_001, SimulationType::Commit),
                (ONB_ACCESS_INSTANCE_CREATE_COMMIT, SimulationType::Commit),
                (ONB_COMPLETE_COMMIT, SimulationType::Commit),
            ] {
                seed_simulation_catalog_status(
                    &mut store,
                    "tenant_1",
                    simulation_id,
                    simulation_type,
                    SimulationStatus::Active,
                );
            }
            seed_invite_link_for_click_with_employee_prefilled_context(&mut store, &inviter_user_id)
        };

        let start = runtime
            .run_invite_link_open_and_start_onboarding(InviteLinkOpenAdapterRequest {
                correlation_id: 73_001,
                idempotency_key: "runh-adapter-start".to_string(),
                token_id,
                token_signature,
                tenant_id: Some("tenant_1".to_string()),
                app_platform: "IOS".to_string(),
                device_fingerprint: "runh_adapter_fp".to_string(),
                app_instance_id: "ios_instance_runh_adapter".to_string(),
                deep_link_nonce: "nonce_runh_adapter".to_string(),
            })
            .expect("invite click should start onboarding");
        assert_eq!(start.next_step.as_deref(), Some("ASK_MISSING"));
        assert!(start
            .required_verification_gates
            .contains(&"SENDER_CONFIRMATION".to_string()));
        let onboarding_session_id = start
            .onboarding_session_id
            .expect("onboarding session id must be present");

        let mut ask_out = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 73_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runh-adapter-ask-prompt".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "ASK_MISSING_SUBMIT".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: None,
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect("first ask-missing turn should prompt");
        while ask_out.next_step.as_deref() == Some("ASK_MISSING") {
            let field_key = ask_out
                .blocking_field
                .clone()
                .expect("blocking field must be returned");
            let field_value = match field_key.as_str() {
                "working_hours" => "09:00-17:00",
                _ => "value_1",
            };
            ask_out = runtime
                .run_onboarding_continue(OnboardingContinueAdapterRequest {
                    correlation_id: 73_001,
                    onboarding_session_id: onboarding_session_id.clone(),
                    idempotency_key: format!("runh-adapter-ask-{field_key}"),
                    tenant_id: Some("tenant_1".to_string()),
                    action: "ASK_MISSING_SUBMIT".to_string(),
                    field_value: Some(field_value.to_string()),
                    receipt_kind: None,
                    receipt_ref: None,
                    signer: None,
                    payload_hash: None,
                    terms_version_id: None,
                    accepted: None,
                    device_id: None,
                    proof_ok: None,
                    sample_seed: None,
                    photo_blob_ref: None,
                    sender_decision: None,
                })
                .expect("ask-missing value submit should succeed");
        }
        assert_eq!(ask_out.next_step.as_deref(), Some("PLATFORM_SETUP"));

        let required_receipts = ask_out.remaining_platform_receipt_kinds.clone();
        let mut platform_out = ask_out;
        for (idx, receipt_kind) in required_receipts.iter().enumerate() {
            platform_out = runtime
                .run_onboarding_continue(OnboardingContinueAdapterRequest {
                    correlation_id: 73_001,
                    onboarding_session_id: onboarding_session_id.clone(),
                    idempotency_key: format!("runh-adapter-platform-{idx}"),
                    tenant_id: Some("tenant_1".to_string()),
                    action: "PLATFORM_SETUP_RECEIPT".to_string(),
                    field_value: None,
                    receipt_kind: Some(receipt_kind.clone()),
                    receipt_ref: Some(format!("receipt:runh-adapter:{receipt_kind}")),
                    signer: Some("selene_mobile_app".to_string()),
                    payload_hash: Some(format!("{:064x}", idx + 1)),
                    terms_version_id: None,
                    accepted: None,
                    device_id: None,
                    proof_ok: None,
                    sample_seed: None,
                    photo_blob_ref: None,
                    sender_decision: None,
                })
                .expect("platform setup receipt should succeed");
        }
        assert_eq!(platform_out.next_step.as_deref(), Some("TERMS"));

        let terms = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 73_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runh-adapter-terms".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "TERMS_ACCEPT".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: Some("terms_v1".to_string()),
                accepted: Some(true),
                device_id: None,
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect("terms should succeed");
        assert_eq!(terms.next_step.as_deref(), Some("SENDER_VERIFICATION"));

        let access_err = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 73_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runh-adapter-access-blocked".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "ACCESS_PROVISION_COMMIT".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: None,
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect_err("access should fail before sender verification");
        assert!(access_err.contains("ONB_SENDER_VERIFICATION_REQUIRED_BEFORE_ACCESS_PROVISION"));

        let photo = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 73_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runh-adapter-photo".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "EMPLOYEE_PHOTO_CAPTURE_SEND".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: None,
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: Some("blob:photo:runh:1".to_string()),
                sender_decision: None,
            })
            .expect("photo capture should succeed");
        assert_eq!(photo.next_step.as_deref(), Some("SENDER_VERIFICATION"));

        let verify = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 73_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runh-adapter-verify".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "EMPLOYEE_SENDER_VERIFY_COMMIT".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: None,
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: Some("CONFIRM".to_string()),
            })
            .expect("sender verify should succeed");
        assert_eq!(verify.next_step.as_deref(), Some("PRIMARY_DEVICE_CONFIRM"));

        let device = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 73_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runh-adapter-device".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "PRIMARY_DEVICE_CONFIRM".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: Some("runh_adapter_inviter_device".to_string()),
                proof_ok: Some(true),
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect("device confirm should succeed");
        assert_eq!(device.next_step.as_deref(), Some("VOICE_ENROLL"));

        let access_before_voice_err = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 73_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runh-adapter-access-before-voice".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "ACCESS_PROVISION_COMMIT".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: None,
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect_err("access should fail before voice enrollment");
        assert!(
            access_before_voice_err.contains("ONB_VOICE_ENROLL_REQUIRED_BEFORE_ACCESS_PROVISION")
        );

        let voice = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 73_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runh-adapter-voice".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "VOICE_ENROLL_LOCK".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: Some("runh_adapter_inviter_device".to_string()),
                proof_ok: None,
                sample_seed: Some("runh_adapter_seed".to_string()),
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect("voice enroll should succeed");
        assert_eq!(voice.next_step.as_deref(), Some("EMO_PERSONA_LOCK"));

        let access_before_emo_err = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 73_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runh-adapter-access-before-emo".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "ACCESS_PROVISION_COMMIT".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: None,
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect_err("access should fail before emo/persona lock");
        assert!(
            access_before_emo_err.contains("ONB_EMO_PERSONA_LOCK_REQUIRED_BEFORE_ACCESS_PROVISION")
        );

        let complete_before_emo_err = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 73_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runh-adapter-complete-before-emo".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "COMPLETE_COMMIT".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: None,
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect_err("complete should fail before emo/persona lock");
        assert!(complete_before_emo_err.contains("ONB_EMO_PERSONA_LOCK_REQUIRED_BEFORE_COMPLETE"));

        let emo = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 73_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runh-adapter-emo".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "EMO_PERSONA_LOCK".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: None,
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect("emo/persona lock should succeed");
        assert_eq!(emo.next_step.as_deref(), Some("ACCESS_PROVISION"));

        let complete_before_access_err = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 73_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runh-adapter-complete-before-access".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "COMPLETE_COMMIT".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: None,
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect_err("complete should fail before access provisioning");
        assert!(
            complete_before_access_err.contains("ONB_ACCESS_PROVISION_REQUIRED_BEFORE_COMPLETE")
        );

        let access = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 73_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runh-adapter-access".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "ACCESS_PROVISION_COMMIT".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: None,
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect("access provision should succeed");
        assert_eq!(access.next_step.as_deref(), Some("COMPLETE"));
        assert!(access.access_engine_instance_id.is_some());

        let complete = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 73_001,
                onboarding_session_id,
                idempotency_key: "runh-adapter-complete".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "COMPLETE_COMMIT".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: None,
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect("onboarding complete should succeed");
        assert_eq!(complete.next_step.as_deref(), Some("READY"));
        assert_eq!(complete.onboarding_status.as_deref(), Some("COMPLETE"));
    }

    #[test]
    fn runi_onboarding_continue_adapter_android_requires_wake_enrollment_before_complete() {
        let runtime = AdapterRuntime::default();
        let inviter_user_id = UserId::new("tenant_1:runi_adapter_inviter").unwrap();
        let inviter_device_id = DeviceId::new("runi_adapter_inviter_device").unwrap();

        let (token_id, token_signature) = {
            let mut store = runtime.store.lock().expect("adapter store lock");
            seed_identity_and_device(&mut store, &inviter_user_id, &inviter_device_id);
            seed_employee_company_and_position(&mut store);
            for (simulation_id, simulation_type) in [
                (LINK_INVITE_OPEN_ACTIVATE_COMMIT, SimulationType::Commit),
                (ONB_SESSION_START_DRAFT, SimulationType::Draft),
                (LINK_INVITE_DRAFT_UPDATE_COMMIT, SimulationType::Commit),
                (ONB_TERMS_ACCEPT_COMMIT, SimulationType::Commit),
                (ONB_PRIMARY_DEVICE_CONFIRM_COMMIT, SimulationType::Commit),
                (VOICE_ID_ENROLL_START_DRAFT, SimulationType::Draft),
                (VOICE_ID_ENROLL_SAMPLE_COMMIT, SimulationType::Commit),
                (VOICE_ID_ENROLL_COMPLETE_COMMIT, SimulationType::Commit),
                (WAKE_ENROLL_START_DRAFT, SimulationType::Draft),
                (WAKE_ENROLL_SAMPLE_COMMIT, SimulationType::Commit),
                (WAKE_ENROLL_COMPLETE_COMMIT, SimulationType::Commit),
                (WAKE_ENROLL_DEFER_COMMIT, SimulationType::Commit),
                (EMO_SIM_001, SimulationType::Commit),
                (ONB_ACCESS_INSTANCE_CREATE_COMMIT, SimulationType::Commit),
                (ONB_COMPLETE_COMMIT, SimulationType::Commit),
            ] {
                seed_simulation_catalog_status(
                    &mut store,
                    "tenant_1",
                    simulation_id,
                    simulation_type,
                    SimulationStatus::Active,
                );
            }
            seed_invite_link_for_click_with_employee_prefilled_context(&mut store, &inviter_user_id)
        };

        let start = runtime
            .run_invite_link_open_and_start_onboarding(InviteLinkOpenAdapterRequest {
                correlation_id: 74_001,
                idempotency_key: "runi-adapter-start".to_string(),
                token_id,
                token_signature,
                tenant_id: Some("tenant_1".to_string()),
                app_platform: "ANDROID".to_string(),
                device_fingerprint: "runi_adapter_fp".to_string(),
                app_instance_id: "android_instance_runi_adapter".to_string(),
                deep_link_nonce: "nonce_runi_adapter".to_string(),
            })
            .expect("invite click should start onboarding");
        let onboarding_session_id = start
            .onboarding_session_id
            .expect("onboarding session id must be present");

        let mut ask_out = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 74_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runi-adapter-ask-prompt".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "ASK_MISSING_SUBMIT".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: None,
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect("first ask-missing turn should prompt");
        while ask_out.next_step.as_deref() == Some("ASK_MISSING") {
            let field_key = ask_out
                .blocking_field
                .clone()
                .expect("blocking field must be returned");
            let field_value = match field_key.as_str() {
                "working_hours" => "09:00-17:00",
                _ => "value_1",
            };
            ask_out = runtime
                .run_onboarding_continue(OnboardingContinueAdapterRequest {
                    correlation_id: 74_001,
                    onboarding_session_id: onboarding_session_id.clone(),
                    idempotency_key: format!("runi-adapter-ask-{field_key}"),
                    tenant_id: Some("tenant_1".to_string()),
                    action: "ASK_MISSING_SUBMIT".to_string(),
                    field_value: Some(field_value.to_string()),
                    receipt_kind: None,
                    receipt_ref: None,
                    signer: None,
                    payload_hash: None,
                    terms_version_id: None,
                    accepted: None,
                    device_id: None,
                    proof_ok: None,
                    sample_seed: None,
                    photo_blob_ref: None,
                    sender_decision: None,
                })
                .expect("ask-missing value submit should succeed");
        }
        assert_eq!(ask_out.next_step.as_deref(), Some("PLATFORM_SETUP"));

        let required_receipts = ask_out.remaining_platform_receipt_kinds.clone();
        let mut platform_out = ask_out;
        for (idx, receipt_kind) in required_receipts.iter().enumerate() {
            platform_out = runtime
                .run_onboarding_continue(OnboardingContinueAdapterRequest {
                    correlation_id: 74_001,
                    onboarding_session_id: onboarding_session_id.clone(),
                    idempotency_key: format!("runi-adapter-platform-{idx}"),
                    tenant_id: Some("tenant_1".to_string()),
                    action: "PLATFORM_SETUP_RECEIPT".to_string(),
                    field_value: None,
                    receipt_kind: Some(receipt_kind.clone()),
                    receipt_ref: Some(format!("receipt:runi-adapter:{receipt_kind}")),
                    signer: Some("selene_mobile_app".to_string()),
                    payload_hash: Some(format!("{:064x}", idx + 1)),
                    terms_version_id: None,
                    accepted: None,
                    device_id: None,
                    proof_ok: None,
                    sample_seed: None,
                    photo_blob_ref: None,
                    sender_decision: None,
                })
                .expect("platform setup receipt should succeed");
        }
        assert_eq!(platform_out.next_step.as_deref(), Some("TERMS"));

        runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 74_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runi-adapter-terms".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "TERMS_ACCEPT".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: Some("terms_v1".to_string()),
                accepted: Some(true),
                device_id: None,
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect("terms should succeed");

        let device = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 74_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runi-adapter-device".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "PRIMARY_DEVICE_CONFIRM".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: Some("runi_adapter_inviter_device".to_string()),
                proof_ok: Some(true),
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect("device confirm should succeed");
        assert_eq!(device.next_step.as_deref(), Some("VOICE_ENROLL"));

        let voice = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 74_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runi-adapter-voice".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "VOICE_ENROLL_LOCK".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: Some("runi_adapter_inviter_device".to_string()),
                proof_ok: None,
                sample_seed: Some("runi_adapter_seed".to_string()),
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect("voice enroll should succeed");
        assert_eq!(voice.next_step.as_deref(), Some("WAKE_ENROLL"));

        let complete_before_wake = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 74_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runi-adapter-complete-before-wake".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "COMPLETE_COMMIT".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: None,
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect_err("complete must fail before wake enrollment completes");
        assert!(complete_before_wake.contains("ONB_WAKE_ENROLL_REQUIRED_BEFORE_COMPLETE"));

        let wake_start = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 74_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runi-adapter-wake-start".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "WAKE_ENROLL_START_DRAFT".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: Some("runi_adapter_inviter_device".to_string()),
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect("wake enroll start should succeed");
        assert_eq!(wake_start.next_step.as_deref(), Some("WAKE_ENROLL"));

        for idx in 0..3 {
            runtime
                .run_onboarding_continue(OnboardingContinueAdapterRequest {
                    correlation_id: 74_001,
                    onboarding_session_id: onboarding_session_id.clone(),
                    idempotency_key: format!("runi-adapter-wake-sample-{idx}"),
                    tenant_id: Some("tenant_1".to_string()),
                    action: "WAKE_ENROLL_SAMPLE_COMMIT".to_string(),
                    field_value: None,
                    receipt_kind: None,
                    receipt_ref: None,
                    signer: None,
                    payload_hash: None,
                    terms_version_id: None,
                    accepted: None,
                    device_id: Some("runi_adapter_inviter_device".to_string()),
                    proof_ok: Some(true),
                    sample_seed: None,
                    photo_blob_ref: None,
                    sender_decision: None,
                })
                .expect("wake enroll sample should succeed");
        }

        let wake_complete = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 74_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runi-adapter-wake-complete".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "WAKE_ENROLL_COMPLETE_COMMIT".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: Some("runi_adapter_inviter_device".to_string()),
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect("wake complete should succeed");
        assert_eq!(wake_complete.next_step.as_deref(), Some("EMO_PERSONA_LOCK"));

        runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 74_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runi-adapter-emo".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "EMO_PERSONA_LOCK".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: None,
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect("emo/persona lock should succeed");

        runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 74_001,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: "runi-adapter-access".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "ACCESS_PROVISION_COMMIT".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: None,
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect("access provision should succeed");

        let complete = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 74_001,
                onboarding_session_id,
                idempotency_key: "runi-adapter-complete".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "COMPLETE_COMMIT".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: None,
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect("onboarding complete should succeed after wake enroll");
        assert_eq!(complete.next_step.as_deref(), Some("READY"));
    }

    #[test]
    fn runj_onboarding_continue_adapter_rejects_ios_wake_enroll_action() {
        let runtime = AdapterRuntime::default();
        let inviter_user_id = UserId::new("tenant_1:runj_adapter_inviter").unwrap();
        let inviter_device_id = DeviceId::new("runj_adapter_inviter_device").unwrap();

        let (token_id, token_signature) = {
            let mut store = runtime.store.lock().expect("adapter store lock");
            seed_identity_and_device(&mut store, &inviter_user_id, &inviter_device_id);
            seed_employee_company_and_position(&mut store);
            for (simulation_id, simulation_type) in [
                (LINK_INVITE_OPEN_ACTIVATE_COMMIT, SimulationType::Commit),
                (ONB_SESSION_START_DRAFT, SimulationType::Draft),
            ] {
                seed_simulation_catalog_status(
                    &mut store,
                    "tenant_1",
                    simulation_id,
                    simulation_type,
                    SimulationStatus::Active,
                );
            }
            seed_invite_link_for_click_with_employee_prefilled_context(&mut store, &inviter_user_id)
        };

        let start = runtime
            .run_invite_link_open_and_start_onboarding(InviteLinkOpenAdapterRequest {
                correlation_id: 75_001,
                idempotency_key: "runj-adapter-start".to_string(),
                token_id,
                token_signature,
                tenant_id: Some("tenant_1".to_string()),
                app_platform: "IOS".to_string(),
                device_fingerprint: "runj_adapter_fp".to_string(),
                app_instance_id: "ios_instance_runj_adapter".to_string(),
                deep_link_nonce: "nonce_runj_adapter".to_string(),
            })
            .expect("invite click should start onboarding");
        let onboarding_session_id = start
            .onboarding_session_id
            .expect("onboarding session id must be present");

        let err = runtime
            .run_onboarding_continue(OnboardingContinueAdapterRequest {
                correlation_id: 75_001,
                onboarding_session_id,
                idempotency_key: "runj-adapter-wake-start".to_string(),
                tenant_id: Some("tenant_1".to_string()),
                action: "WAKE_ENROLL_START_DRAFT".to_string(),
                field_value: None,
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: Some("runj_adapter_inviter_device".to_string()),
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            })
            .expect_err("ios wake enroll action must fail closed");
        assert!(err.contains("ios_wake_disabled"));
    }

    #[test]
    fn rung_onboarding_continue_adapter_parses_sender_verification_actions() {
        let photo = parse_onboarding_continue_action(
            "EMPLOYEE_PHOTO_CAPTURE_SEND",
            OnboardingContinueActionInput {
                photo_blob_ref: Some("blob:photo:test".to_string()),
                ..OnboardingContinueActionInput::default()
            },
        )
        .expect("photo capture action must parse");
        assert!(matches!(
            photo,
            AppOnboardingContinueAction::EmployeePhotoCaptureSend { .. }
        ));

        let verify = parse_onboarding_continue_action(
            "EMPLOYEE_SENDER_VERIFY_COMMIT",
            OnboardingContinueActionInput {
                sender_decision: Some("CONFIRM".to_string()),
                ..OnboardingContinueActionInput::default()
            },
        )
        .expect("sender verify action must parse");
        assert!(matches!(
            verify,
            AppOnboardingContinueAction::EmployeeSenderVerifyCommit {
                decision: SenderVerifyDecision::Confirm
            }
        ));

        let wake_start = parse_onboarding_continue_action(
            "WAKE_ENROLL_START_DRAFT",
            OnboardingContinueActionInput {
                device_id: Some("wake_device_1".to_string()),
                ..OnboardingContinueActionInput::default()
            },
        )
        .expect("wake start action must parse");
        assert!(matches!(
            wake_start,
            AppOnboardingContinueAction::WakeEnrollStartDraft { .. }
        ));

        let err = parse_onboarding_continue_action(
            "EMPLOYEE_SENDER_VERIFY_COMMIT",
            OnboardingContinueActionInput {
                sender_decision: Some("MAYBE".to_string()),
                ..OnboardingContinueActionInput::default()
            },
        )
        .expect_err("invalid sender decision must fail");
        assert!(err.contains("sender_decision must be CONFIRM or REJECT"));
    }

    #[test]
    fn rund_onboarding_continue_adapter_requires_platform_receipt_fields() {
        let err = parse_onboarding_continue_action(
            "PLATFORM_SETUP_RECEIPT",
            OnboardingContinueActionInput {
                receipt_kind: Some("install_launch_handshake".to_string()),
                receipt_ref: Some("receipt:rund-adapter:install".to_string()),
                payload_hash: Some(format!("{:064x}", 1)),
                ..OnboardingContinueActionInput::default()
            },
        )
        .expect_err("missing signer must fail action parsing");
        assert_eq!(err, "signer is required for PLATFORM_SETUP_RECEIPT");
    }

    fn synthetic_health_for_detail_tests() -> AdapterHealthResponse {
        AdapterHealthResponse {
            status: "ok".to_string(),
            outcome: "AT_RISK".to_string(),
            reason: None,
            sync: AdapterSyncHealth {
                worker: AdapterSyncWorkerCounters {
                    pass_count: 3,
                    dequeued_total: 7,
                    acked_total: 2,
                    retry_scheduled_total: 2,
                    dead_lettered_total: 1,
                    last_pass_at_ns: Some(500),
                    last_dequeued_count: 2,
                    last_acked_count: 1,
                    last_retry_scheduled_count: 1,
                    last_dead_lettered_count: 1,
                },
                queue: AdapterSyncQueueCounters {
                    queued_count: 4,
                    in_flight_count: 1,
                    acked_count: 2,
                    dead_letter_count: 1,
                    replay_due_count: 1,
                    retry_pending_count: 2,
                },
                improvement: AdapterImprovementCounters::default(),
            },
        }
    }

    #[test]
    fn at_adapter_01_valid_ios_request_forwards() {
        let runtime = AdapterRuntime::default();
        let out = runtime
            .run_voice_turn(base_request())
            .expect("valid request must succeed");
        assert_eq!(out.status, "ok");
        assert_eq!(out.outcome, "FINAL");
        assert!(out
            .reason
            .as_deref()
            .unwrap_or_default()
            .contains("voice_identity="));
    }

    #[test]
    fn at_wake_01_desktop_wake_without_enrollment_fails() {
        let runtime = AdapterRuntime::default();
        let mut req = base_request();
        req.app_platform = "DESKTOP".to_string();
        req.trigger = "WAKE_WORD".to_string();
        req.device_id = Some("adapter_wake_desktop_no_enroll".to_string());
        let err = runtime
            .run_voice_turn(req)
            .expect_err("desktop wake without enrollment must fail");
        assert_eq!(err, "wake_not_enrolled");
    }

    #[test]
    fn at_wake_02_android_wake_without_enrollment_fails() {
        let runtime = AdapterRuntime::default();
        let mut req = base_request();
        req.app_platform = "ANDROID".to_string();
        req.trigger = "WAKE_WORD".to_string();
        req.device_id = Some("adapter_wake_android_no_enroll".to_string());
        let err = runtime
            .run_voice_turn(req)
            .expect_err("android wake without enrollment must fail");
        assert_eq!(err, "wake_not_enrolled");
    }

    #[test]
    fn at_wake_03_ios_wakeword_is_always_disabled() {
        let runtime = AdapterRuntime::default();
        let mut req = base_request();
        req.trigger = "WAKE_WORD".to_string();
        req.device_id = Some("adapter_wake_ios_disabled".to_string());
        seed_wake_enrollment_complete_for_request(&runtime, &mut req, "at_wake_03");
        let err = runtime
            .run_voice_turn(req)
            .expect_err("ios wake must fail closed");
        assert_eq!(err, "ios_wake_disabled");
    }

    #[test]
    fn at_wake_04_ph1w_inference_controls_accept_reject() {
        let runtime = AdapterRuntime::default();
        let mut req = base_request();
        req.app_platform = "DESKTOP".to_string();
        req.trigger = "WAKE_WORD".to_string();
        req.device_id = Some("adapter_wake_inference_gate".to_string());
        seed_wake_enrollment_complete_for_request(&runtime, &mut req, "at_wake_04");
        if let Some(capture) = req.audio_capture_ref.as_mut() {
            capture.capture_degraded = Some(true);
        }
        let err = runtime
            .run_voice_turn(req)
            .expect_err("degraded capture should reject via PH1.W inference");
        assert!(
            err.contains("wake_rejected"),
            "expected wake_rejected from PH1.W inference path, got {err}"
        );
        let store = runtime.store.lock().expect("store lock must not poison");
        assert!(store
            .ph1w_get_runtime_events()
            .iter()
            .any(|row| !row.accepted));
    }

    #[test]
    fn at_wakelearn_01_accept_enqueues_wakeaccepted_signal() {
        let runtime = AdapterRuntime::default();
        let mut req = base_request();
        req.correlation_id = 33_001;
        req.turn_id = 43_001;
        req.now_ns = Some(9_000_000_000);
        req.app_platform = "DESKTOP".to_string();
        req.trigger = "WAKE_WORD".to_string();
        req.device_id = Some("adapter_wakelearn_accept".to_string());
        seed_wake_enrollment_complete_for_request(&runtime, &mut req, "at_wakelearn_01");
        runtime
            .run_voice_turn(req)
            .expect("accepted wake turn should succeed");

        let store = runtime.store.lock().expect("store lock must not poison");
        assert!(store
            .wake_learn_signal_rows()
            .iter()
            .any(|row| row.event_type == LearnSignalType::WakeAccepted));
        assert!(store
            .device_artifact_sync_queue_rows()
            .iter()
            .any(|row| { row.sync_kind == MobileArtifactSyncKind::WakeLearnSignal }));
    }

    #[test]
    fn at_wakelearn_02_reject_enqueues_wakerejected_family_signal() {
        let runtime = AdapterRuntime::default();
        let mut req = base_request();
        req.correlation_id = 33_002;
        req.turn_id = 43_002;
        req.now_ns = Some(9_000_000_100);
        req.app_platform = "DESKTOP".to_string();
        req.trigger = "WAKE_WORD".to_string();
        req.device_id = Some("adapter_wakelearn_reject".to_string());
        seed_wake_enrollment_complete_for_request(&runtime, &mut req, "at_wakelearn_02");
        if let Some(capture) = req.audio_capture_ref.as_mut() {
            capture.capture_degraded = Some(true);
        }
        let err = runtime
            .run_voice_turn(req)
            .expect_err("rejected wake turn should fail");
        assert!(err.contains("wake_rejected"));

        let store = runtime.store.lock().expect("store lock must not poison");
        assert!(store.wake_learn_signal_rows().iter().any(|row| {
            matches!(
                row.event_type,
                LearnSignalType::WakeRejected
                    | LearnSignalType::LowConfidenceWake
                    | LearnSignalType::NoisyEnvironment
            )
        }));
        assert!(store
            .device_artifact_sync_queue_rows()
            .iter()
            .any(|row| { row.sync_kind == MobileArtifactSyncKind::WakeLearnSignal }));
    }

    #[test]
    fn at_wake_05_desktop_live_decision_window_is_non_trivial() {
        let cfg = resolve_ph1w_live_loop_config(AppPlatform::Desktop);
        assert!(
            cfg.window_ms >= 1_000,
            "desktop wake loop window must be >= 1000ms, got {}",
            cfg.window_ms
        );
        assert!(
            cfg.max_steps > 8 || cfg.window_ms >= 1_000,
            "desktop wake loop must be non-trivial (max_steps={}, window_ms={})",
            cfg.max_steps,
            cfg.window_ms
        );
        assert!(cfg.hop_ms >= 20);
    }

    #[test]
    fn at_l_01_wake_opens_new_session_persists_session_id() {
        let runtime = AdapterRuntime::default();
        let mut req = base_request();
        req.correlation_id = 31_001;
        req.turn_id = 41_001;
        req.now_ns = Some(1_000_000_000);
        req.app_platform = "DESKTOP".to_string();
        req.trigger = "WAKE_WORD".to_string();
        seed_wake_enrollment_complete_for_request(&runtime, &mut req, "at_l_01");
        runtime
            .run_voice_turn(req.clone())
            .expect("wake turn must succeed");

        let actor_user_id = UserId::new(req.actor_user_id).expect("actor id must parse");
        let device_id = DeviceId::new(req.device_id.expect("device id must be present"))
            .expect("device id must parse");
        let store = runtime.store.lock().expect("store lock must not poison");
        let session = latest_canonical_session_for_actor(&store, &actor_user_id)
            .expect("canonical session lookup must succeed")
            .expect("wake turn must persist a session row");
        assert_eq!(session.session_state, SessionState::Active);
        assert!(session.attached_devices.contains(&device_id));
        assert_eq!(session.last_attached_device_id, device_id);
        assert!(store.get_session(&session.session_id).is_some());
    }

    #[test]
    fn at_l_02_next_turn_reuses_canonical_actor_session_when_active() {
        let runtime = AdapterRuntime::default();
        let mut first = base_request();
        first.correlation_id = 31_002;
        first.turn_id = 41_002;
        first.now_ns = Some(2_000_000_000);
        first.app_platform = "DESKTOP".to_string();
        first.trigger = "WAKE_WORD".to_string();
        seed_wake_enrollment_complete_for_request(&runtime, &mut first, "at_l_02");
        runtime
            .run_voice_turn(first.clone())
            .expect("first wake turn must succeed");

        let actor_user_id = UserId::new(first.actor_user_id.clone()).expect("actor id must parse");
        let device_id = DeviceId::new(first.device_id.clone().expect("device id must exist"))
            .expect("device id must parse");
        let first_session_id = {
            let store = runtime.store.lock().expect("store lock must not poison");
            latest_canonical_session_for_actor(&store, &actor_user_id)
                .expect("canonical session lookup must succeed")
                .expect("first turn must persist session")
                .session_id
        };

        let mut second = base_request();
        second.correlation_id = 31_003;
        second.turn_id = 41_003;
        second.now_ns = Some(7_000_000_000);
        second.app_platform = "DESKTOP".to_string();
        second.trigger = "WAKE_WORD".to_string();
        mark_request_as_attested_capture(&mut second);
        runtime
            .run_voice_turn(second)
            .expect("second wake turn must succeed");

        let store = runtime.store.lock().expect("store lock must not poison");
        let second_session = latest_canonical_session_for_actor(&store, &actor_user_id)
            .expect("canonical session lookup must succeed")
            .expect("second turn must persist session");
        assert_eq!(second_session.session_id, first_session_id);
        assert_eq!(
            store
                .session_rows()
                .values()
                .filter(|row| row.user_id == actor_user_id && row.is_recoverable())
                .count(),
            1
        );
        assert!(second_session.attached_devices.contains(&device_id));
    }

    #[test]
    fn at_l_03_timeout_closes_session_next_turn_opens_new() {
        let runtime = AdapterRuntime::default();
        let mut first = base_request();
        first.correlation_id = 31_004;
        first.turn_id = 41_004;
        first.now_ns = Some(3_000_000_000);
        first.app_platform = "DESKTOP".to_string();
        first.trigger = "WAKE_WORD".to_string();
        seed_wake_enrollment_complete_for_request(&runtime, &mut first, "at_l_03");
        runtime
            .run_voice_turn(first.clone())
            .expect("first wake turn must succeed");

        let actor_user_id = UserId::new(first.actor_user_id.clone()).expect("actor id must parse");
        let device_id = DeviceId::new(first.device_id.clone().expect("device id must exist"))
            .expect("device id must parse");
        let first_session = {
            let store = runtime.store.lock().expect("store lock must not poison");
            latest_canonical_session_for_actor(&store, &actor_user_id)
                .expect("canonical session lookup must succeed")
                .expect("first turn must persist session")
        };

        {
            let mut store = runtime.store.lock().expect("store lock must not poison");
            let forced_soft_closed = SessionRecord::v1(
                first_session.session_id,
                actor_user_id.clone(),
                device_id.clone(),
                SessionState::SoftClosed,
                first_session.opened_at,
                first_session.opened_at,
                None,
            )
            .expect("forced soft-closed record must validate");
            store
                .upsert_session_lifecycle(
                    forced_soft_closed,
                    Some("at_l_03_force_soft_closed".to_string()),
                )
                .expect("forced soft-close upsert must succeed");
        }

        let mut second = base_request();
        second.correlation_id = 31_005;
        second.turn_id = 41_005;
        second.now_ns = Some(250_000_000_000);
        second.app_platform = "DESKTOP".to_string();
        second.trigger = "WAKE_WORD".to_string();
        if let Some(capture) = second.audio_capture_ref.as_mut() {
            capture.tts_playback_active = Some(false);
        }
        mark_request_as_attested_capture(&mut second);
        runtime
            .run_voice_turn(second)
            .expect("turn after timeout must succeed");

        let store = runtime.store.lock().expect("store lock must not poison");
        let latest = latest_canonical_session_for_actor(&store, &actor_user_id)
            .expect("canonical session lookup must succeed")
            .expect("latest session must exist");
        assert_ne!(latest.session_id, first_session.session_id);
        assert_eq!(latest.session_state, SessionState::Active);
        assert!(latest.attached_devices.contains(&device_id));
        let previous = store
            .get_session(&first_session.session_id)
            .expect("prior session row must still exist");
        assert_eq!(previous.session_state, SessionState::Closed);
        assert_eq!(previous.closed_at, Some(MonotonicTimeNs(250_000_000_000)));
    }

    #[test]
    fn at_l_04_cross_device_attach_reuses_canonical_session() {
        let runtime = AdapterRuntime::default();

        let mut wake = base_request();
        wake.correlation_id = 31_006;
        wake.turn_id = 41_006;
        wake.now_ns = Some(4_000_000_000);
        wake.app_platform = "DESKTOP".to_string();
        wake.trigger = "WAKE_WORD".to_string();
        wake.device_id = Some("adapter_session_wake_device".to_string());
        seed_wake_enrollment_complete_for_request(&runtime, &mut wake, "at_l_04");
        let wake_response = runtime
            .run_voice_turn(wake.clone())
            .expect("wake turn must succeed");

        let mut explicit = base_request();
        explicit.correlation_id = 31_007;
        explicit.turn_id = 41_007;
        explicit.now_ns = Some(5_000_000_000);
        explicit.app_platform = "DESKTOP".to_string();
        explicit.trigger = "EXPLICIT".to_string();
        explicit.device_id = Some("adapter_session_explicit_device".to_string());
        let explicit_response = runtime
            .run_voice_turn(explicit.clone())
            .expect("explicit turn must succeed");

        let actor_user_id = UserId::new(wake.actor_user_id).expect("actor id must parse");
        let wake_device =
            DeviceId::new(wake.device_id.expect("wake device id must exist")).expect("valid id");
        let explicit_device =
            DeviceId::new(explicit.device_id.expect("explicit device id must exist"))
                .expect("valid id");
        let store = runtime.store.lock().expect("store lock must not poison");
        let canonical_session = latest_canonical_session_for_actor(&store, &actor_user_id)
            .expect("canonical session lookup must succeed")
            .expect("canonical actor session must exist");
        assert_eq!(wake_response.session_id, explicit_response.session_id);
        assert_eq!(
            wake_response.session_attach_outcome,
            Some(SessionAttachOutcome::NewSessionCreated)
        );
        assert_eq!(
            explicit_response.session_attach_outcome,
            Some(SessionAttachOutcome::ExistingSessionAttached)
        );
        assert_eq!(canonical_session.session_state, SessionState::Active);
        assert!(canonical_session.attached_devices.contains(&wake_device));
        assert!(canonical_session
            .attached_devices
            .contains(&explicit_device));
        assert_eq!(canonical_session.last_attached_device_id, explicit_device);
    }

    #[test]
    fn at_l_05_session_outputs_are_authoritative_and_reconcilable() {
        let runtime = AdapterRuntime::default();
        let mut req = base_request();
        req.correlation_id = 31_008;
        req.turn_id = 41_008;
        req.now_ns = Some(6_000_000_000);
        req.app_platform = "DESKTOP".to_string();
        req.trigger = "WAKE_WORD".to_string();
        seed_wake_enrollment_complete_for_request(&runtime, &mut req, "at_l_05");
        let response = runtime
            .run_voice_turn(req.clone())
            .expect("wake turn must succeed");

        let actor_user_id = UserId::new(req.actor_user_id).expect("actor id must parse");
        let device_id = DeviceId::new(req.device_id.expect("device id must be present"))
            .expect("device id must parse");
        let mut store = runtime.store.lock().expect("store lock must not poison");
        let session_id = latest_canonical_session_for_actor(&store, &actor_user_id)
            .expect("canonical session lookup must succeed")
            .expect("session row must exist")
            .session_id;
        assert_eq!(response.session_id, Some(session_id_to_string(session_id)));
        assert_eq!(response.turn_id, Some(41_008));
        assert_eq!(response.session_state.as_deref(), Some("ACTIVE"));
        assert_eq!(
            response.session_attach_outcome,
            Some(SessionAttachOutcome::NewSessionCreated)
        );
        assert!(store
            .get_session(&session_id)
            .expect("session row must exist")
            .attached_devices
            .contains(&device_id));

        let correlation_id = CorrelationId(req.correlation_id.into());
        assert!(
            store
                .audit_events()
                .iter()
                .filter(|event| event.correlation_id == correlation_id)
                .any(|event| event.session_id == Some(session_id)),
            "at least one audit event for the turn must include session_id"
        );

        let memory_key =
            MemoryKey::new("at_l_05.favorite_food".to_string()).expect("memory key must be valid");
        let memory_event = MemoryLedgerEvent::v1(
            MemoryLedgerEventKind::Stored,
            MonotonicTimeNs(6_000_000_100),
            memory_key.clone(),
            Some(
                MemoryValue::v1("pizza".to_string(), Some("pizza".to_string()))
                    .expect("memory value must validate"),
            ),
            Some("JD said remember pizza".to_string()),
            MemoryProvenance::v1(
                Some(session_id),
                Some("at_l_05_transcript_hash".to_string()),
            )
            .expect("memory provenance must validate"),
            MemoryLayer::Working,
            MemorySensitivityFlag::Low,
            MemoryConfidence::High,
            MemoryConsent::ExplicitRemember,
            ReasonCodeId(0x4C00_0005),
        )
        .expect("memory ledger event must validate");
        store
            .append_memory_ledger_event(
                &actor_user_id,
                memory_event,
                MemoryUsePolicy::AlwaysUsable,
                None,
                Some("at_l_05_memory_event".to_string()),
            )
            .expect("memory event append must succeed");

        let memory_record = store
            .memory_current()
            .get(&(actor_user_id.clone(), memory_key))
            .expect("memory current row must exist");
        assert_eq!(memory_record.provenance.session_id, Some(session_id));
    }

    #[test]
    fn at_l_06_stale_device_turn_is_rejected_deterministically() {
        let runtime = AdapterRuntime::default();
        let mut first = base_request();
        first.correlation_id = 31_009;
        first.turn_id = 41_009;
        first.device_turn_sequence = Some(1);
        seed_identity_and_device_for_request(&runtime, &first);
        let first_response = runtime
            .run_voice_turn_ingress(first)
            .expect("first explicit turn must succeed");
        assert_eq!(
            first_response.session_attach_outcome,
            Some(SessionAttachOutcome::NewSessionCreated)
        );

        let mut second = base_request();
        second.correlation_id = 31_010;
        second.turn_id = 41_010;
        second.device_turn_sequence = Some(2);
        runtime
            .run_voice_turn_ingress(second)
            .expect("second explicit turn must succeed");

        let mut stale = base_request();
        stale.correlation_id = 31_011;
        stale.turn_id = 41_011;
        stale.device_turn_sequence = Some(1);
        let error = runtime
            .run_voice_turn_ingress(stale)
            .expect_err("stale device turn must fail");
        assert_eq!(error.failure_class, FailureClass::SessionConflict);
        assert!(error.reason_code.contains("STALE_DEVICE_TURN"));
    }

    #[test]
    fn at_l_07_lawful_retry_reuses_prior_authoritative_result() {
        let runtime = AdapterRuntime::default();
        let mut request = base_request();
        request.correlation_id = 31_012;
        request.turn_id = 41_012;
        request.device_turn_sequence = Some(7);
        seed_identity_and_device_for_request(&runtime, &request);

        let first = runtime
            .run_voice_turn_ingress(request.clone())
            .expect("first turn must succeed");
        let second = runtime
            .run_voice_turn_ingress(request)
            .expect("retry turn must reuse prior result");

        assert_eq!(second.session_id, first.session_id);
        assert_eq!(second.turn_id, first.turn_id);
        assert_eq!(second.session_state, first.session_state);
        assert_eq!(second.response_text, first.response_text);
        assert_eq!(
            second.session_attach_outcome,
            Some(SessionAttachOutcome::RetryReusedResult)
        );
    }

    #[test]
    fn at_l_08_session_lease_owner_blocks_conflicting_node_until_expiry() {
        let runtime = AdapterRuntime::default();
        let mut first = base_request();
        first.correlation_id = 31_013;
        first.turn_id = 41_013;
        first.device_turn_sequence = Some(1);
        seed_identity_and_device_for_request(&runtime, &first);
        let first_response = runtime
            .run_voice_turn_ingress(first.clone())
            .expect("first turn must succeed");
        let actor_user_id = UserId::new(first.actor_user_id.clone()).expect("actor id must parse");
        let device_id = DeviceId::new(first.device_id.clone().expect("device id must exist"))
            .expect("device id");
        let session_id = SessionId(
            first_response
                .session_id
                .as_deref()
                .expect("session id must exist")
                .parse::<u128>()
                .expect("session id must parse"),
        );

        {
            let mut store = runtime.store.lock().expect("store lock must not poison");
            let mut leased = store
                .get_session(&session_id)
                .expect("session row must exist")
                .clone();
            leased.attached_devices.insert(device_id.clone());
            leased.last_attached_device_id = device_id.clone();
            leased.lease_owner_id = Some("other_runtime_node".to_string());
            leased.lease_acquired_at = Some(MonotonicTimeNs(2_000_000_000));
            leased.lease_expires_at = Some(MonotonicTimeNs(20_000_000_000));
            leased.active_turn_id = Some(TurnId(99_999));
            store
                .upsert_session_lifecycle(leased, Some("at_l_08_force_other_lease".to_string()))
                .expect("forced lease must upsert");
        }

        let mut blocked = base_request();
        blocked.correlation_id = 31_014;
        blocked.turn_id = 41_014;
        blocked.now_ns = Some(3_000_000_000);
        blocked.device_turn_sequence = Some(2);
        let blocked_error = runtime
            .run_voice_turn_ingress(blocked.clone())
            .expect_err("active foreign lease must block mutation");
        assert_eq!(blocked_error.failure_class, FailureClass::SessionConflict);
        assert!(blocked_error.reason_code.contains("SESSION_LEASE_CONFLICT"));

        {
            let mut store = runtime.store.lock().expect("store lock must not poison");
            let mut expired = store
                .get_session(&session_id)
                .expect("session row must exist")
                .clone();
            expired.lease_acquired_at = Some(MonotonicTimeNs(1));
            expired.lease_expires_at = Some(MonotonicTimeNs(1));
            store
                .upsert_session_lifecycle(expired, Some("at_l_08_expire_lease".to_string()))
                .expect("expired lease must upsert");
        }

        let recovered = runtime
            .run_voice_turn_ingress(blocked)
            .expect("expired lease must allow safe recovery");
        assert_eq!(recovered.session_id, Some(session_id_to_string(session_id)));
        assert_eq!(
            latest_canonical_session_for_actor(
                &runtime.store.lock().expect("store lock must not poison"),
                &actor_user_id
            )
            .expect("canonical session lookup must succeed")
            .expect("session must still exist")
            .lease_owner_id
            .as_deref(),
            Some(runtime.runtime_node_id.as_str())
        );
    }

    #[test]
    fn at_l_09_single_writer_conflict_is_explicit() {
        let runtime = AdapterRuntime::default();
        let mut first = base_request();
        first.correlation_id = 31_015;
        first.turn_id = 41_015;
        first.device_turn_sequence = Some(1);
        seed_identity_and_device_for_request(&runtime, &first);
        let first_response = runtime
            .run_voice_turn_ingress(first.clone())
            .expect("first turn must succeed");
        let device_id = DeviceId::new(first.device_id.clone().expect("device id must exist"))
            .expect("device id");
        let session_id = SessionId(
            first_response
                .session_id
                .as_deref()
                .expect("session id must exist")
                .parse::<u128>()
                .expect("session id must parse"),
        );

        {
            let mut store = runtime.store.lock().expect("store lock must not poison");
            let mut active = store
                .get_session(&session_id)
                .expect("session row must exist")
                .clone();
            active.attached_devices.insert(device_id.clone());
            active.last_attached_device_id = device_id;
            active.lease_owner_id = Some(runtime.runtime_node_id.clone());
            active.lease_acquired_at = Some(MonotonicTimeNs(2_000_000_000));
            active.lease_expires_at = Some(MonotonicTimeNs(20_000_000_000));
            active.active_turn_id = Some(TurnId(88_888));
            store
                .upsert_session_lifecycle(active, Some("at_l_09_force_active_turn".to_string()))
                .expect("forced active turn must upsert");
        }

        let mut conflicting = base_request();
        conflicting.correlation_id = 31_016;
        conflicting.turn_id = 41_016;
        conflicting.now_ns = Some(3_000_000_000);
        conflicting.device_turn_sequence = Some(2);
        let error = runtime
            .run_voice_turn_ingress(conflicting)
            .expect_err("parallel writer must fail closed");
        assert_eq!(error.failure_class, FailureClass::SessionConflict);
        assert!(error.reason_code.contains("SINGLE_WRITER_CONFLICT"));
    }

    #[test]
    fn at_l_10_deterministic_turn_ordering_tracks_multiple_attached_devices() {
        let runtime = AdapterRuntime::default();

        let mut device_a_first = base_request();
        device_a_first.correlation_id = 31_017;
        device_a_first.turn_id = 41_017;
        device_a_first.device_id = Some("session_order_device_a".to_string());
        device_a_first.device_turn_sequence = Some(1);
        seed_identity_and_device_for_request(&runtime, &device_a_first);
        let first = runtime
            .run_voice_turn_ingress(device_a_first.clone())
            .expect("device A first turn must succeed");

        let mut device_b_first = base_request();
        device_b_first.correlation_id = 31_018;
        device_b_first.turn_id = 41_018;
        device_b_first.device_id = Some("session_order_device_b".to_string());
        device_b_first.device_turn_sequence = Some(1);
        seed_identity_and_device_for_request(&runtime, &device_b_first);
        let second = runtime
            .run_voice_turn_ingress(device_b_first.clone())
            .expect("device B attach turn must succeed");

        let mut device_a_second = base_request();
        device_a_second.correlation_id = 31_019;
        device_a_second.turn_id = 41_019;
        device_a_second.device_id = Some("session_order_device_a".to_string());
        device_a_second.device_turn_sequence = Some(2);
        let third = runtime
            .run_voice_turn_ingress(device_a_second.clone())
            .expect("device A second turn must succeed");

        let actor_user_id =
            UserId::new(device_a_first.actor_user_id.clone()).expect("actor id must parse");
        let device_a = DeviceId::new(device_a_first.device_id.expect("device A must exist"))
            .expect("device A");
        let device_b = DeviceId::new(device_b_first.device_id.expect("device B must exist"))
            .expect("device B");
        let store = runtime.store.lock().expect("store lock must not poison");
        let session = latest_canonical_session_for_actor(&store, &actor_user_id)
            .expect("canonical session lookup must succeed")
            .expect("canonical session must exist");

        assert_eq!(first.session_id, second.session_id);
        assert_eq!(second.session_id, third.session_id);
        assert_eq!(session.device_turn_sequence_for(&device_a), Some(2));
        assert_eq!(session.device_turn_sequence_for(&device_b), Some(1));
        assert_eq!(session.last_turn_id, Some(TurnId(41_019)));
        assert!(session.attached_devices.contains(&device_a));
        assert!(session.attached_devices.contains(&device_b));
    }

    #[test]
    fn at_os_01_tablet_platform_is_accepted_end_to_end() {
        let runtime = AdapterRuntime::default();
        let mut request = base_tablet_request();
        request.correlation_id = 32_001;
        request.turn_id = 42_001;
        request.device_id = Some("tablet_end_to_end_device".to_string());
        request.user_text_final = Some("Show my session status".to_string());

        let response = runtime
            .run_voice_turn(request)
            .expect("tablet request must succeed");
        assert_eq!(response.status, "ok");
        assert!(response.session_id.is_some());
        assert_eq!(response.session_state.as_deref(), Some("ACTIVE"));
    }

    #[test]
    fn at_os_02_invalid_platform_is_rejected_deterministically() {
        let runtime = AdapterRuntime::default();
        let mut request = base_request();
        request.correlation_id = 32_002;
        request.turn_id = 42_002;
        request.app_platform = "BLACKBERRY".to_string();

        let error = runtime
            .run_voice_turn_ingress(request)
            .expect_err("invalid platform must fail closed");
        assert_eq!(error.failure_class, FailureClass::InvalidPayload);
        assert_eq!(error.reason_code, "INVALID_RUNTIME_EXECUTION_ENVELOPE");
    }

    #[test]
    fn at_os_03_normalized_platform_identity_is_carried_into_runtime() {
        let runtime = AdapterRuntime::default();
        let mut request = base_tablet_request();
        request.correlation_id = 32_003;
        request.turn_id = 42_003;
        request.device_id = Some("tablet_identity_device".to_string());
        request.user_text_final = Some("What can this device do?".to_string());

        runtime
            .run_voice_turn(request)
            .expect("tablet request must succeed");
        let packet = runtime
            .ingress
            .debug_last_agent_input_packet()
            .expect("agent packet should be captured");
        let runtime_execution_envelope = packet
            .runtime_execution_envelope
            .expect("runtime execution envelope must exist");
        let platform_context = runtime_execution_envelope.platform_context;

        assert_eq!(runtime_execution_envelope.platform, AppPlatform::Tablet);
        assert_eq!(platform_context.platform_type, AppPlatform::Tablet);
        assert_eq!(platform_context.platform_version, "15.2");
        assert_eq!(platform_context.device_class, DeviceClass::Tablet);
        assert_eq!(platform_context.runtime_client_version, "2.3.4");
        assert_eq!(platform_context.hardware_capability_profile, "TABLET_PRO");
        assert_eq!(platform_context.network_profile, NetworkProfile::Standard);
    }

    #[test]
    fn at_os_04_capability_negotiation_state_is_attached() {
        let runtime = AdapterRuntime::default();
        let mut request = base_tablet_request();
        request.correlation_id = 32_004;
        request.turn_id = 42_004;
        request.device_id = Some("tablet_caps_device".to_string());
        request.claimed_capabilities = Some(vec![
            "MICROPHONE".to_string(),
            "WAKE_WORD".to_string(),
            "SENSOR_AVAILABILITY".to_string(),
        ]);
        request.user_text_final = Some("Use the platform registry".to_string());

        runtime
            .run_voice_turn(request)
            .expect("tablet request must succeed");
        let packet = runtime
            .ingress
            .debug_last_agent_input_packet()
            .expect("agent packet should be captured");
        let platform_context = packet
            .runtime_execution_envelope
            .expect("runtime execution envelope must exist")
            .platform_context;

        assert_eq!(
            platform_context.claimed_capabilities,
            vec![
                DeviceCapability::Microphone,
                DeviceCapability::SensorAvailability,
                DeviceCapability::WakeWord,
            ]
        );
        assert_eq!(
            platform_context.negotiated_capabilities,
            vec![
                DeviceCapability::Microphone,
                DeviceCapability::SensorAvailability,
                DeviceCapability::WakeWord,
            ]
        );
        assert_eq!(
            platform_context.requested_trigger,
            RuntimeEntryTrigger::Explicit
        );
        assert_eq!(
            platform_context.trigger_policy,
            PlatformTriggerPolicy::WakeOrExplicit
        );
        assert!(platform_context.trigger_allowed);
    }

    #[test]
    fn at_os_05_trust_integrity_and_compatibility_survive_ingress() {
        let _min_version = ScopedEnvVar::set("SELENE_MIN_CLIENT_VERSION_TABLET", "5.0.0");
        let runtime = AdapterRuntime::default();
        let mut request = base_tablet_request();
        request.correlation_id = 32_005;
        request.turn_id = 42_005;
        request.device_id = Some("tablet_trust_device".to_string());
        request.runtime_client_version = Some("4.2.0".to_string());
        request.user_text_final = Some("Check client posture".to_string());

        runtime
            .run_voice_turn(request)
            .expect("tablet request must succeed");
        let packet = runtime
            .ingress
            .debug_last_agent_input_packet()
            .expect("agent packet should be captured");
        let platform_context = packet
            .runtime_execution_envelope
            .expect("runtime execution envelope must exist")
            .platform_context;

        assert_eq!(
            platform_context.integrity_status,
            ClientIntegrityStatus::Attested
        );
        assert_eq!(
            platform_context.compatibility_status,
            ClientCompatibilityStatus::UpgradeRequired
        );
        assert_eq!(
            platform_context.device_trust_class,
            DeviceTrustClass::RestrictedDevice
        );
        assert_eq!(
            platform_context.minimum_supported_client_version.as_deref(),
            Some("5.0.0")
        );
        assert_eq!(
            platform_context.attestation_ref.as_deref(),
            Some("tablet_attest_ref_01")
        );
        assert!(platform_context.capture_artifact_trust_verified);
        assert_eq!(platform_context.capture_artifact_observed_at_ns, Some(3));
        assert_eq!(
            platform_context.capture_artifact_retention_deadline_ns,
            Some(5_000_000_003)
        );
    }

    #[test]
    fn at_mem_01_probable_identity_returns_empty_memory_candidates() {
        let runtime = AdapterRuntime::default();
        let mut req = base_request();
        req.correlation_id = 33_001;
        req.turn_id = 43_001;
        req.now_ns = Some(9_000_000_000);
        req.trigger = "EXPLICIT".to_string();
        req.user_text_final = Some("Send the invite link to Tom".to_string());

        let prepared = prepare_trigger_agent_input_shape(&runtime, req);
        assert_eq!(prepared.shape.memory_candidates_len, 0);
    }

    #[test]
    fn at_mem_04_memory_provenance_contains_session_id() {
        at_l_05_session_outputs_are_authoritative_and_reconcilable();
    }

    #[test]
    fn at_agentpkt_01_packet_contains_all_required_fields() {
        let runtime = AdapterRuntime::default();
        let mut req = base_request();
        req.correlation_id = 34_001;
        req.turn_id = 44_001;
        req.now_ns = Some(10_000_000_000);
        req.thread_key = Some("adapter_agentpkt_thread".to_string());
        req.user_text_final = Some("show me weather".to_string());

        let out = runtime
            .run_voice_turn(req)
            .expect("voice turn should succeed for agent packet test");
        assert_eq!(out.status, "ok");
        let packet = runtime
            .ingress
            .debug_last_agent_input_packet()
            .expect("agent packet should be captured");
        assert_eq!(packet.correlation_id, 34_001);
        assert_eq!(packet.turn_id, 44_001);
        assert_eq!(
            packet.thread_key.as_deref(),
            Some("adapter_agentpkt_thread")
        );
        assert!(packet.session_id.is_some());
        assert!(!packet.trace_id.is_empty());
        assert!(!packet.packet_hash.is_empty());
        assert!(!packet.sim_catalog_snapshot_hash.is_empty());
    }

    #[test]
    fn at_agentpkt_02_packet_hash_stable_for_same_inputs() {
        let runtime = AdapterRuntime::default();
        let mut req = base_request();
        req.correlation_id = 34_002;
        req.turn_id = 44_002;
        req.now_ns = Some(10_000_000_500);
        req.thread_key = Some("adapter_agentpkt_hash_thread".to_string());
        req.user_text_final = Some("same deterministic request".to_string());

        runtime
            .run_voice_turn(req.clone())
            .expect("first voice turn should succeed");
        let first = runtime
            .ingress
            .debug_last_agent_input_packet()
            .expect("first packet should exist");
        runtime
            .run_voice_turn(req)
            .expect("second voice turn should succeed");
        let second = runtime
            .ingress
            .debug_last_agent_input_packet()
            .expect("second packet should exist");
        assert_eq!(first.packet_hash, second.packet_hash);
    }

    #[test]
    fn at_agentpkt_03_packet_built_once_per_turn() {
        let runtime = AdapterRuntime::default();
        let mut req = base_request();
        req.correlation_id = 34_003;
        req.turn_id = 44_003;
        req.now_ns = Some(10_000_001_000);
        req.thread_key = Some("adapter_agentpkt_once_thread".to_string());
        req.user_text_final = Some("packet build count".to_string());

        let before = runtime.ingress.debug_agent_input_packet_build_count();
        runtime
            .run_voice_turn(req)
            .expect("voice turn should succeed");
        let after = runtime.ingress.debug_agent_input_packet_build_count();
        assert_eq!(after, before + 1);
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct AgentInputPacketShape {
        session_state: SessionState,
        session_id_present: bool,
        wake_event_present: bool,
        voice_id_vad_events_len: usize,
        voice_id_owner_user_present: bool,
        ingress_tenant_present: bool,
        ingress_device_present: bool,
        identity_context_is_voice: bool,
        identity_prompt_scope_key_present: bool,
        nlp_output_present: bool,
        tool_response_present: bool,
        memory_candidates_len: usize,
        confirm_answer_present: bool,
        locale_present: bool,
        thread_pending_present: bool,
        thread_project_id_present: bool,
        thread_policy_flags_present: bool,
        pinned_context_refs_len: usize,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TriggerPrepResult {
        session_id: Option<SessionId>,
        shape: AgentInputPacketShape,
    }

    fn prepare_trigger_agent_input_shape(
        runtime: &AdapterRuntime,
        request: VoiceTurnAdapterRequest,
    ) -> TriggerPrepResult {
        let app_platform =
            parse_app_platform(&request.app_platform).expect("request app_platform must parse");
        let trigger = parse_trigger(&request.trigger).expect("request trigger must parse");
        let actor_user_id =
            UserId::new(request.actor_user_id.clone()).expect("request actor_user_id must parse");
        let request_device_id = request
            .device_id
            .as_ref()
            .map(|id| DeviceId::new(id.clone()).expect("request device_id must parse"));
        let now = MonotonicTimeNs(request.now_ns.unwrap_or(1));
        let runtime_device_id = request_device_id.unwrap_or_else(|| {
            DeviceId::new(format!(
                "adapter_auto_{}",
                stable_hash_hex_16(actor_user_id.as_str())
            ))
            .expect("generated runtime device id must be valid")
        });
        let (wake_event_present, voice_id_vad_events_len, voice_id_owner_user_present) = {
            let mut store = runtime.store.lock().expect("store lock must not poison");
            ensure_actor_identity_and_device(
                &mut store,
                &actor_user_id,
                Some(&runtime_device_id),
                app_platform,
                now,
                true,
            )
            .expect("identity/device seed must succeed");
            let tenant_id_for_ph1c = resolve_tenant_scope(
                request.tenant_id.clone(),
                &actor_user_id,
                Some(&runtime_device_id),
            );
            let ph1k_bundle = build_ph1k_live_signal_bundle(
                &store,
                &request,
                now,
                tenant_id_for_ph1c.as_deref(),
                Some(&runtime_device_id),
            )
            .expect("ph1k live signal bundle must build");
            let wake_evaluation = evaluate_wake_for_turn(
                &store,
                now,
                &actor_user_id,
                &runtime_device_id,
                app_platform,
                trigger,
                &ph1k_bundle,
            )
            .expect("wake gate + inference should resolve for trigger prep");
            if let Some(wake_eval) = wake_evaluation.as_ref() {
                assert!(
                    wake_eval.decision.accepted,
                    "trigger prep helper expects accepted wake decisions"
                );
            }
            let session_snapshot = SessionSnapshot {
                schema_version: selene_kernel_contracts::SchemaVersion(1),
                session_state: SessionState::Closed,
                session_id: None,
                next_allowed_actions: selene_kernel_contracts::ph1l::NextAllowedActions {
                    may_speak: true,
                    must_wait: false,
                    must_rewake: false,
                },
            };
            let voice_id_request = build_voice_id_request_from_ph1k_bundle(
                now,
                &ph1k_bundle,
                session_snapshot,
                wake_evaluation.as_ref().map(|wake| wake.decision.clone()),
            )
            .expect("voice id request must build");
            (
                voice_id_request.wake_event.is_some(),
                voice_id_request.vad_events.len(),
                voice_id_request.device_owner_user_id.is_some(),
            )
        };

        runtime
            .run_voice_turn(request.clone())
            .expect("trigger prep helper voice turn must succeed");
        let packet = runtime
            .ingress
            .debug_last_agent_input_packet()
            .expect("trigger prep helper must capture agent input packet");
        let session_id = packet.session_id;
        let shape = AgentInputPacketShape {
            session_state: packet.session_state,
            session_id_present: packet.session_id.is_some(),
            wake_event_present,
            voice_id_vad_events_len,
            voice_id_owner_user_present,
            ingress_tenant_present: request.tenant_id.is_some(),
            ingress_device_present: request.device_id.is_some(),
            identity_context_is_voice: true,
            identity_prompt_scope_key_present: packet.identity_prompt_scope_key.is_some(),
            nlp_output_present: packet.nlp_output.is_some(),
            tool_response_present: packet.tool_response.is_some(),
            memory_candidates_len: packet.memory_candidates.len(),
            confirm_answer_present: packet.confirm_answer.is_some(),
            locale_present: packet.language_hint.is_some(),
            thread_pending_present: packet.thread_state.pending.is_some(),
            thread_project_id_present: packet.thread_state.project_id.is_some(),
            thread_policy_flags_present: packet.thread_state.thread_policy_flags.is_some(),
            pinned_context_refs_len: packet.thread_state.pinned_context_refs.len(),
        };
        TriggerPrepResult { session_id, shape }
    }

    #[test]
    fn at_trigger_01_wakeword_and_explicit_share_same_session_open_path() {
        let runtime = AdapterRuntime::default();

        let mut wake = base_request();
        wake.correlation_id = 32_001;
        wake.turn_id = 42_001;
        wake.now_ns = Some(7_000_000_000);
        wake.app_platform = "DESKTOP".to_string();
        wake.trigger = "WAKE_WORD".to_string();
        wake.device_id = Some("adapter_trigger_wake_1".to_string());
        wake.user_text_final = Some("check trigger parity".to_string());
        seed_wake_enrollment_complete_for_request(&runtime, &mut wake, "at_trigger_01");
        let wake_prepared = prepare_trigger_agent_input_shape(&runtime, wake);

        let mut explicit = base_request();
        explicit.correlation_id = 32_002;
        explicit.turn_id = 42_002;
        explicit.now_ns = Some(7_000_000_100);
        explicit.app_platform = "DESKTOP".to_string();
        explicit.trigger = "EXPLICIT".to_string();
        explicit.device_id = Some("adapter_trigger_explicit_1".to_string());
        explicit.user_text_final = Some("check trigger parity".to_string());
        let explicit_prepared = prepare_trigger_agent_input_shape(&runtime, explicit);

        assert!(wake_prepared.session_id.is_some());
        assert!(explicit_prepared.session_id.is_some());
        assert_eq!(wake_prepared.shape.session_state, SessionState::Active);
        assert_eq!(explicit_prepared.shape.session_state, SessionState::Active);
        assert!(wake_prepared.shape.identity_context_is_voice);
        assert!(explicit_prepared.shape.identity_context_is_voice);
    }

    #[test]
    fn at_trigger_02_wakeword_and_explicit_produce_same_agent_input_packet_shape() {
        let runtime = AdapterRuntime::default();

        let mut wake = base_request();
        wake.correlation_id = 32_101;
        wake.turn_id = 42_101;
        wake.now_ns = Some(8_000_000_000);
        wake.app_platform = "DESKTOP".to_string();
        wake.trigger = "WAKE_WORD".to_string();
        wake.device_id = Some("adapter_trigger_shape_wake".to_string());
        wake.user_text_final = Some("show me weather".to_string());
        seed_wake_enrollment_complete_for_request(&runtime, &mut wake, "at_trigger_02");
        let wake_shape = prepare_trigger_agent_input_shape(&runtime, wake).shape;

        let mut explicit = base_request();
        explicit.correlation_id = 32_102;
        explicit.turn_id = 42_102;
        explicit.now_ns = Some(8_000_000_100);
        explicit.app_platform = "DESKTOP".to_string();
        explicit.trigger = "EXPLICIT".to_string();
        explicit.device_id = Some("adapter_trigger_shape_explicit".to_string());
        explicit.user_text_final = Some("show me weather".to_string());
        let explicit_shape = prepare_trigger_agent_input_shape(&runtime, explicit).shape;

        assert!(wake_shape.wake_event_present);
        assert!(!explicit_shape.wake_event_present);
        let mut wake_shape_without_wake_flag = wake_shape.clone();
        wake_shape_without_wake_flag.wake_event_present = explicit_shape.wake_event_present;
        assert_eq!(wake_shape_without_wake_flag, explicit_shape);
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
        assert_eq!(out.outcome, "FINAL");
    }

    #[test]
    fn at_adapter_03b_voice_turn_provider_unconfigured_fails_closed_with_vault_hint() {
        with_isolated_empty_device_vault("at_adapter_03b", || {
            let runtime = AdapterRuntime::default();
            let mut req = base_request();
            req.user_text_final = Some("Selene search the web for H100 pricing".to_string());
            let out = runtime
                .run_voice_turn(req)
                .expect("voice turn with explicit web query must succeed");
            assert_eq!(out.status, "ok");
            assert_eq!(out.outcome, "FINAL_TOOL");
            assert_eq!(out.next_move, "dispatch_tool");
            assert_eq!(out.reason_code, "1476395017");
            let response_text = out.response_text.as_str();
            assert!(response_text.contains("selene vault set brave_search_api_key"));
        });
    }

    #[test]
    fn at_adapter_03bb_voice_turn_surfaces_safe_brave_failure_detail() {
        with_isolated_device_vault(
            "at_adapter_03bb",
            &[("brave_search_api_key", "test_brave_key")],
            &[
                (
                    "BRAVE_SEARCH_WEB_URL",
                    "http://127.0.0.1:9/res/v1/web/search",
                ),
                (
                    "BRAVE_SEARCH_NEWS_URL",
                    "http://127.0.0.1:9/res/v1/news/search",
                ),
            ],
            || {
                let runtime = AdapterRuntime::default();
                let mut req = base_request();
                req.user_text_final = Some("Selene search the web for H100 pricing".to_string());
                let out = runtime
                    .run_voice_turn(req)
                    .expect("voice turn with forced brave failure must return adapter response");
                assert_eq!(out.status, "ok");
                assert_eq!(out.outcome, "FINAL_TOOL");
                let text = out.response_text.to_ascii_lowercase();
                assert!(text.contains("provider=brave"));
                assert!(text.contains("error="));
            },
        );
    }

    #[test]
    fn at_adapter_03bc_voice_turn_surfaces_safe_openai_fallback_failure_detail() {
        with_isolated_device_vault(
            "at_adapter_03bc",
            &[("openai_api_key", "test_openai_key")],
            &[("OPENAI_RESPONSES_URL", "http://127.0.0.1:9/v1/responses")],
            || {
                let runtime = AdapterRuntime::default();
                let mut req = base_request();
                req.user_text_final = Some("Selene search the web for H100 pricing".to_string());
                let out = runtime
                    .run_voice_turn(req)
                    .expect("voice turn with forced openai failure must return adapter response");
                assert_eq!(out.status, "ok");
                assert_eq!(out.outcome, "FINAL_TOOL");
                let text = out.response_text.to_ascii_lowercase();
                assert!(text.contains("provider=openai"));
                assert!(text.contains("error="));
            },
        );
    }

    #[test]
    fn at_adapter_03ba_calendar_event_confirm_yes_dispatches_sim_and_persists_meeting_reminder() {
        let runtime = AdapterRuntime::default();
        let actor_user_id = UserId::new("tenant_a:user_adapter_test").unwrap();
        {
            let mut store = runtime.store.lock().expect("store lock should succeed");
            ensure_actor_identity_and_device(
                &mut store,
                &actor_user_id,
                None,
                AppPlatform::Desktop,
                MonotonicTimeNs(1),
                true,
            )
            .expect("identity + device seed should succeed");
            seed_simulation_catalog_status(
                &mut store,
                "tenant_a",
                REMINDER_SCHEDULE_COMMIT,
                SimulationType::Commit,
                SimulationStatus::Active,
            );
            seed_calendar_access_instance(&mut store, &actor_user_id, "tenant_a");
        }

        let mut first = base_request();
        first.app_platform = "DESKTOP".to_string();
        first.trigger = "EXPLICIT".to_string();
        first.device_id = Some("adapter_desktop_calendar_1".to_string());
        first.thread_key = Some("calendar_draft_thread".to_string());
        first.correlation_id = 10_201;
        first.turn_id = 20_201;
        first.now_ns = Some(21);
        first.user_text_final =
            Some("Selene create a calendar event tomorrow 3pm called demo".to_string());

        let out_first = runtime
            .run_voice_turn(first)
            .expect("calendar event first turn should return confirm");
        assert_eq!(out_first.status, "ok");
        assert_eq!(out_first.next_move, "clarify");

        let mut second = base_request();
        second.app_platform = "DESKTOP".to_string();
        second.trigger = "EXPLICIT".to_string();
        second.device_id = Some("adapter_desktop_calendar_1".to_string());
        second.thread_key = Some("calendar_draft_thread".to_string());
        second.correlation_id = 10_202;
        second.turn_id = 20_202;
        second.now_ns = Some(22);
        second.user_text_final = Some("yes".to_string());

        let out_second = runtime
            .run_voice_turn(second)
            .expect("calendar event confirm turn should dispatch simulation");
        assert_eq!(out_second.status, "ok");
        assert_eq!(out_second.outcome, "DISPATCH_SIM");
        assert_eq!(out_second.next_move, "dispatch_sim");
        assert_eq!(
            out_second.response_text,
            "Draft created; not sent to external calendar yet."
        );
        assert!(out_second.provenance.is_none());

        let store = runtime.store.lock().expect("store lock should succeed");
        let reminders = store.reminders();
        assert_eq!(reminders.len(), 1);
        let reminder = reminders
            .values()
            .next()
            .expect("calendar confirm should create reminder row");
        assert_eq!(reminder.reminder_type, ReminderType::Meeting);
        assert_eq!(reminder.user_id, actor_user_id);
        assert!(store.work_order_ledger().is_empty());
    }

    #[test]
    fn at_adapter_03bb_cancel_reminder_confirm_yes_dispatches_sim_and_cancels_row() {
        let runtime = AdapterRuntime::default();
        let actor_user_id = UserId::new("tenant_a:user_adapter_test").unwrap();
        let reminder_id = {
            let mut store = runtime.store.lock().expect("store lock should succeed");
            ensure_actor_identity_and_device(
                &mut store,
                &actor_user_id,
                None,
                AppPlatform::Desktop,
                MonotonicTimeNs(1),
                true,
            )
            .expect("identity + device seed should succeed");
            seed_simulation_catalog_status(
                &mut store,
                "tenant_a",
                REMINDER_CANCEL_COMMIT,
                SimulationType::Commit,
                SimulationStatus::Active,
            );
            seed_reminder_access_instance(&mut store, &actor_user_id, "tenant_a");
            let seed_req = Ph1RemRequest::schedule_commit_v1(
                CorrelationId(90_001),
                TurnId(90_101),
                MonotonicTimeNs(10),
                TenantId::new("tenant_a".to_string()).unwrap(),
                actor_user_id.clone(),
                None,
                ReminderType::Task,
                "review payroll".to_string(),
                "in 5 minutes".to_string(),
                "UTC".to_string(),
                ReminderLocalTimeMode::LocalTime,
                ReminderPriorityLevel::Normal,
                None,
                vec![ReminderChannel::Text],
                "seed_adapter_cancel".to_string(),
            )
            .unwrap();
            match store
                .ph1rem_run(&seed_req)
                .expect("seed reminder should schedule")
            {
                Ph1RemResponse::Ok(ok) => ok.reminder_id,
                _ => panic!("expected seeded reminder"),
            }
        };

        let mut first = base_request();
        first.app_platform = "DESKTOP".to_string();
        first.trigger = "EXPLICIT".to_string();
        first.device_id = Some("adapter_desktop_cancel_1".to_string());
        first.thread_key = Some("cancel_reminder_thread".to_string());
        first.correlation_id = 10_301;
        first.turn_id = 20_301;
        first.now_ns = Some(31);
        first.user_text_final = Some(format!("Selene cancel reminder {}", reminder_id.as_str()));

        let out_first = runtime
            .run_voice_turn(first)
            .expect("cancel reminder first turn should return confirm");
        assert_eq!(out_first.status, "ok");
        assert_eq!(out_first.next_move, "clarify");

        let mut second = base_request();
        second.app_platform = "DESKTOP".to_string();
        second.trigger = "EXPLICIT".to_string();
        second.device_id = Some("adapter_desktop_cancel_1".to_string());
        second.thread_key = Some("cancel_reminder_thread".to_string());
        second.correlation_id = 10_302;
        second.turn_id = 20_302;
        second.now_ns = Some(32);
        second.user_text_final = Some("yes".to_string());

        let out_second = runtime
            .run_voice_turn(second)
            .expect("cancel reminder confirm turn should dispatch simulation");
        assert_eq!(out_second.status, "ok");
        assert_eq!(out_second.outcome, "DISPATCH_SIM");
        assert_eq!(out_second.next_move, "dispatch_sim");
        assert_eq!(out_second.response_text, "I canceled that reminder.");

        let store = runtime.store.lock().expect("store lock should succeed");
        let reminder = store
            .reminder_row(&reminder_id)
            .expect("reminder row should exist");
        assert_eq!(reminder.state, ReminderState::Canceled);
    }

    #[test]
    fn at_adapter_03bc_list_reminders_uses_read_only_tool_lane_with_provenance() {
        let runtime = AdapterRuntime::default();
        let mut req = base_request();
        req.app_platform = "DESKTOP".to_string();
        req.user_text_final = Some("Selene list my reminders".to_string());
        let out = runtime
            .run_voice_turn(req)
            .expect("list reminders query should succeed");
        assert_eq!(out.status, "ok");
        assert_eq!(out.outcome, "FINAL_TOOL");
        assert_eq!(out.next_move, "dispatch_tool");
        assert!(out.response_text.contains("Summary:"));
        assert!(out.provenance.is_some());
    }

    #[test]
    fn at_adapter_03c_thread_state_loader_round_trips_from_ph1f() {
        let runtime = AdapterRuntime::default();
        let actor_user_id = UserId::new("tenant_a:user_adapter_test").unwrap();
        let thread_key = resolve_adapter_thread_key(Some("trip_japan"));

        {
            let mut store = runtime.store.lock().expect("store lock should succeed");
            ensure_actor_identity_and_device(
                &mut store,
                &actor_user_id,
                None,
                AppPlatform::Desktop,
                MonotonicTimeNs(1),
                true,
            )
            .expect("identity + device seed should succeed");
            store
                .ph1x_thread_state_upsert_commit(
                    MonotonicTimeNs(2),
                    actor_user_id.clone(),
                    thread_key.clone(),
                    KernelThreadState::v1(
                        Some(PendingState::Clarify {
                            missing_field: FieldKey::Task,
                            attempts: 1,
                        }),
                        None,
                    ),
                    ReasonCodeId(0x5800_7001),
                    "adapter_thread_state_seed".to_string(),
                )
                .expect("thread state seed should commit");
            let loaded = load_ph1x_thread_state(&store, &actor_user_id, &thread_key);
            match loaded.pending {
                Some(PendingState::Clarify {
                    missing_field,
                    attempts,
                }) => {
                    assert_eq!(missing_field, FieldKey::Task);
                    assert_eq!(attempts, 1);
                }
                _ => panic!("expected clarify pending state"),
            }
        }
    }

    #[test]
    fn at_adapter_03d_cross_device_turns_share_same_thread_state_scope() {
        let runtime = AdapterRuntime::default();
        let mut ios = base_request();
        ios.thread_key = Some("trip_sync".to_string());
        ios.user_text_final = Some("Selene search the web for H100 pricing".to_string());
        ios.device_id = Some("adapter_ios_device_cross_1".to_string());
        ios.app_platform = "IOS".to_string();
        ios.correlation_id = 10_101;
        ios.turn_id = 20_101;
        ios.now_ns = Some(11);
        runtime
            .run_voice_turn(ios)
            .expect("ios voice turn should succeed");

        let mut desktop = base_request();
        desktop.thread_key = Some("trip_sync".to_string());
        desktop.user_text_final = Some("Selene what's the latest news about NVIDIA".to_string());
        desktop.device_id = Some("adapter_desktop_device_cross_1".to_string());
        desktop.app_platform = "DESKTOP".to_string();
        desktop.correlation_id = 10_102;
        desktop.turn_id = 20_102;
        desktop.now_ns = Some(12);
        runtime
            .run_voice_turn(desktop)
            .expect("desktop voice turn should succeed");

        let actor_user_id = UserId::new("tenant_a:user_adapter_test").unwrap();
        let store = runtime.store.lock().expect("store lock should succeed");
        let current = store
            .ph1x_thread_state_current_row(&actor_user_id, "trip_sync")
            .expect("shared thread state should exist");
        assert_eq!(current.updated_at, MonotonicTimeNs(12));
        let ledger_count = store
            .ph1x_thread_state_ledger_rows()
            .iter()
            .filter(|row| {
                row.user_id.as_str() == actor_user_id.as_str() && row.thread_key == "trip_sync"
            })
            .count();
        assert_eq!(ledger_count, 2);
    }

    #[test]
    fn at_adapter_03e_thread_state_project_context_and_policy_flags_round_trip() {
        let runtime = AdapterRuntime::default();
        let actor_user_id = UserId::new("tenant_a:user_adapter_test").unwrap();
        let thread_key = resolve_adapter_thread_key(Some("proj_scope"));

        {
            let mut store = runtime.store.lock().expect("store lock should succeed");
            ensure_actor_identity_and_device(
                &mut store,
                &actor_user_id,
                None,
                AppPlatform::Desktop,
                MonotonicTimeNs(1),
                true,
            )
            .expect("identity + device seed should succeed");
            let seeded_state = KernelThreadState::empty_v1()
                .with_project_context(
                    Some("proj_q3_planning".to_string()),
                    vec![
                        "ctx_budget_sheet".to_string(),
                        "ctx_roadmap_notes".to_string(),
                    ],
                )
                .unwrap()
                .with_thread_policy_flags(Some(ThreadPolicyFlags::v1(true, false, true).unwrap()))
                .unwrap();
            store
                .ph1x_thread_state_upsert_commit(
                    MonotonicTimeNs(2),
                    actor_user_id.clone(),
                    thread_key.clone(),
                    seeded_state,
                    ReasonCodeId(0x5800_7002),
                    "adapter_thread_state_project_seed".to_string(),
                )
                .expect("thread state seed should commit");
            let loaded = load_ph1x_thread_state(&store, &actor_user_id, &thread_key);
            assert_eq!(loaded.project_id.as_deref(), Some("proj_q3_planning"));
            assert_eq!(
                loaded.pinned_context_refs,
                vec![
                    "ctx_budget_sheet".to_string(),
                    "ctx_roadmap_notes".to_string()
                ]
            );
            let flags = loaded
                .thread_policy_flags
                .expect("thread policy flags should round-trip");
            assert!(flags.force_privacy_mode);
            assert!(!flags.force_do_not_disturb);
            assert!(flags.force_strict_safety);
        }
    }

    #[test]
    fn at_adapter_03f_voice_turn_persists_project_context_and_policy_flags() {
        let runtime = AdapterRuntime::default();
        let mut req = base_request();
        req.thread_key = Some("proj_live".to_string());
        req.user_text_final = Some("Selene search the web for H100 pricing".to_string());
        req.project_id = Some("proj q3 planning".to_string());
        req.pinned_context_refs = Some(vec![
            "ctx budget sheet".to_string(),
            "ctx-roadmap-notes".to_string(),
        ]);
        req.thread_policy_flags = Some(VoiceTurnThreadPolicyFlags {
            privacy_mode: true,
            do_not_disturb: false,
            strict_safety: true,
        });
        req.correlation_id = 10_103;
        req.turn_id = 20_103;
        req.now_ns = Some(13);

        runtime
            .run_voice_turn(req)
            .expect("voice turn with project context should succeed");

        let actor_user_id = UserId::new("tenant_a:user_adapter_test").unwrap();
        let store = runtime.store.lock().expect("store lock should succeed");
        let current = store
            .ph1x_thread_state_current_row(&actor_user_id, "proj_live")
            .expect("thread state should persist for project context");
        assert_eq!(
            current.thread_state.project_id.as_deref(),
            Some("proj_q3_planning")
        );
        assert_eq!(
            current.thread_state.pinned_context_refs,
            vec![
                "ctx_budget_sheet".to_string(),
                "ctx-roadmap-notes".to_string()
            ]
        );
        let flags = current
            .thread_state
            .thread_policy_flags
            .expect("thread policy flags should persist");
        assert!(flags.force_privacy_mode);
        assert!(!flags.force_do_not_disturb);
        assert!(flags.force_strict_safety);
    }

    #[test]
    fn at_adapter_03g_read_only_tool_fail_emits_feedback_learn_and_builder_signal() {
        let runtime = AdapterRuntime::default();
        let mut req = base_request();
        req.correlation_id = 10_104;
        req.turn_id = 20_104;
        req.now_ns = Some(14);
        req.user_text_final = Some("Selene search the web for timeout".to_string());

        let out = runtime
            .run_voice_turn(req)
            .expect("tool-fail turn should still return an adapter response");
        assert_eq!(out.status, "ok");
        assert_eq!(out.outcome, "FINAL_TOOL");

        let correlation_id = CorrelationId(10_104);
        let store = runtime.store.lock().expect("store lock should succeed");
        let feedback_rows = store.ph1feedback_audit_rows(correlation_id);
        assert!(feedback_rows
            .iter()
            .any(|row| { feedback_event_type_matches(row, "ToolFail") }));
        let learn_rows = store.ph1feedback_learn_signal_bundle_rows(correlation_id);
        assert!(learn_rows
            .iter()
            .any(|row| { row.learn_signal_type == LearnSignalType::ToolFail }));
        drop(store);

        let health = runtime
            .health_report(Some(14))
            .expect("health report should expose builder counters");
        assert!(health.sync.improvement.builder_runs_total >= 1);
    }

    #[test]
    fn at_adapter_03h_clarify_loop_emits_feedback_and_learn_signal_bundle() {
        let runtime = AdapterRuntime::default();

        let mut first = base_request();
        first.correlation_id = 10_105;
        first.turn_id = 20_105;
        first.now_ns = Some(15);
        first.thread_key = Some("clarify_loop_thread".to_string());
        first.user_text_final = Some("Set reminder".to_string());
        let out_first = runtime
            .run_voice_turn(first)
            .expect("first clarify turn should succeed");
        assert_eq!(out_first.next_move, "clarify");

        let mut second = base_request();
        second.correlation_id = 10_106;
        second.turn_id = 20_106;
        second.now_ns = Some(16);
        second.thread_key = Some("clarify_loop_thread".to_string());
        second.user_text_final = Some("Set reminder".to_string());
        let out_second = runtime
            .run_voice_turn(second)
            .expect("second clarify turn should succeed");
        assert_eq!(out_second.next_move, "clarify");

        let correlation_id = CorrelationId(10_106);
        let store = runtime.store.lock().expect("store lock should succeed");
        let feedback_rows = store.ph1feedback_audit_rows(correlation_id);
        assert!(feedback_rows
            .iter()
            .any(|row| { feedback_event_type_matches(row, "ClarifyLoop") }));
        let learn_rows = store.ph1feedback_learn_signal_bundle_rows(correlation_id);
        assert!(learn_rows
            .iter()
            .any(|row| { row.learn_signal_type == LearnSignalType::ClarifyLoop }));
    }

    #[test]
    fn at_adapter_03i_user_correction_phrase_emits_feedback_and_learn_signal_bundle() {
        let runtime = AdapterRuntime::default();
        let mut req = base_request();
        req.correlation_id = 10_107;
        req.turn_id = 20_107;
        req.now_ns = Some(17);
        req.user_text_final = Some("No, I meant weather in Singapore".to_string());

        runtime
            .run_voice_turn(req)
            .expect("user-correction turn should succeed");

        let correlation_id = CorrelationId(10_107);
        let store = runtime.store.lock().expect("store lock should succeed");
        let feedback_rows = store.ph1feedback_audit_rows(correlation_id);
        assert!(feedback_rows
            .iter()
            .any(|row| { feedback_event_type_matches(row, "UserCorrection") }));
        let learn_rows = store.ph1feedback_learn_signal_bundle_rows(correlation_id);
        assert!(learn_rows
            .iter()
            .any(|row| { row.learn_signal_type == LearnSignalType::UserCorrection }));
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
    fn at_adapter_07b_journal_replay_restores_thread_state_across_runtime_restart() {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be >= unix epoch")
            .as_nanos();
        let journal_path =
            std::env::temp_dir().join(format!("selene_adapter_thread_state_replay_{seed}.jsonl"));

        let runtime_one = AdapterRuntime::new_with_persistence(
            AppServerIngressRuntime::default(),
            Arc::new(Mutex::new(Ph1fStore::new_in_memory())),
            journal_path.clone(),
            true,
        )
        .expect("first runtime with persistence must construct");

        let mut first = base_request();
        first.correlation_id = 30_001;
        first.turn_id = 40_001;
        first.now_ns = Some(11);
        first.thread_key = Some("trip_restart".to_string());
        first.device_id = Some("adapter_ios_device_restart_1".to_string());
        first.app_platform = "IOS".to_string();
        first.user_text_final = Some("Selene search the web for H100 pricing".to_string());
        runtime_one
            .run_voice_turn(first)
            .expect("first runtime request must succeed");

        let actor_user_id = UserId::new("tenant_a:user_adapter_test").unwrap();
        {
            let store = runtime_one
                .store
                .lock()
                .expect("first runtime store lock must succeed");
            let current = store
                .ph1x_thread_state_current_row(&actor_user_id, "trip_restart")
                .expect("thread state must be persisted in first runtime");
            assert_eq!(current.updated_at, MonotonicTimeNs(11));
            let count = store
                .ph1x_thread_state_ledger_rows()
                .iter()
                .filter(|row| {
                    row.user_id.as_str() == actor_user_id.as_str()
                        && row.thread_key == "trip_restart"
                })
                .count();
            assert_eq!(count, 1);
        }

        let runtime_two = AdapterRuntime::new_with_persistence(
            AppServerIngressRuntime::default(),
            Arc::new(Mutex::new(Ph1fStore::new_in_memory())),
            journal_path.clone(),
            true,
        )
        .expect("second runtime should replay prior journal");

        {
            let store = runtime_two
                .store
                .lock()
                .expect("second runtime store lock must succeed");
            let current = store
                .ph1x_thread_state_current_row(&actor_user_id, "trip_restart")
                .expect("thread state must be restored by replay in second runtime");
            assert_eq!(current.updated_at, MonotonicTimeNs(11));
            let count = store
                .ph1x_thread_state_ledger_rows()
                .iter()
                .filter(|row| {
                    row.user_id.as_str() == actor_user_id.as_str()
                        && row.thread_key == "trip_restart"
                })
                .count();
            assert_eq!(count, 1);
        }

        let mut second = base_request();
        second.correlation_id = 30_002;
        second.turn_id = 40_002;
        second.now_ns = Some(12);
        second.thread_key = Some("trip_restart".to_string());
        second.device_id = Some("adapter_desktop_device_restart_1".to_string());
        second.app_platform = "DESKTOP".to_string();
        second.user_text_final = Some("Selene what's the latest news about NVIDIA".to_string());
        runtime_two
            .run_voice_turn(second)
            .expect("second runtime request must succeed");

        {
            let store = runtime_two
                .store
                .lock()
                .expect("second runtime store lock must succeed after second turn");
            let current = store
                .ph1x_thread_state_current_row(&actor_user_id, "trip_restart")
                .expect("thread state must persist after second runtime request");
            assert_eq!(current.updated_at, MonotonicTimeNs(12));
            let count = store
                .ph1x_thread_state_ledger_rows()
                .iter()
                .filter(|row| {
                    row.user_id.as_str() == actor_user_id.as_str()
                        && row.thread_key == "trip_restart"
                })
                .count();
            assert_eq!(count, 2);
        }

        let _ = std::fs::remove_file(journal_path);
    }

    #[test]
    fn at_persistence_01_pending_operation_survives_restart() {
        let journal_path = temp_persistence_journal_path("pending_survives_restart");
        let runtime_one = AdapterRuntime::new_with_persistence(
            AppServerIngressRuntime::default(),
            Arc::new(Mutex::new(Ph1fStore::new_in_memory())),
            journal_path.clone(),
            true,
        )
        .expect("runtime with persistence must construct");
        let request = base_request();
        let envelope = fallback_runtime_execution_envelope_for_voice_turn_request(&request)
            .expect("fallback runtime execution envelope must build");
        let operation_id = derived_operation_id(&request.actor_user_id, &envelope.idempotency_key);

        runtime_one
            .prepare_persistence_operation(
                &request,
                &envelope,
                MonotonicTimeNs(request.now_ns.expect("request now must be present")),
                PersistenceInvocationMode::Standard,
            )
            .expect("persistence prepare must succeed")
            .expect("prepared persistence operation must exist");

        let state_before_restart = read_persistence_state_for_test(&journal_path);
        let pending_record = state_before_restart
            .outbox_records
            .get(&operation_id)
            .expect("pending outbox record must exist before restart");
        assert_eq!(
            pending_record.acknowledgement_state,
            PersistenceAcknowledgementState::PendingCloudAcknowledgement
        );
        assert!(pending_record.cleared_from_active_outbox_at_ns.is_none());

        drop(runtime_one);

        let _runtime_two = AdapterRuntime::new_with_persistence(
            AppServerIngressRuntime::default(),
            Arc::new(Mutex::new(Ph1fStore::new_in_memory())),
            journal_path.clone(),
            true,
        )
        .expect("second runtime must reconcile pending outbox");

        let state_after_restart = read_persistence_state_for_test(&journal_path);
        let acknowledged_record = state_after_restart
            .outbox_records
            .get(&operation_id)
            .expect("outbox record must survive restart");
        assert_eq!(
            acknowledged_record.acknowledgement_state,
            PersistenceAcknowledgementState::AuthoritativelyAcknowledged
        );
        assert!(acknowledged_record
            .cleared_from_active_outbox_at_ns
            .is_some());
        assert!(state_after_restart.last_reconciled_at_ns.is_some());
        assert_eq!(
            state_after_restart.recovery_mode,
            PersistenceRecoveryMode::Normal
        );

        cleanup_persistence_files_for_test(&journal_path);
    }

    #[test]
    fn at_persistence_02_acknowledged_operation_clears_active_outbox() {
        let journal_path = temp_persistence_journal_path("acknowledgement_clears");
        let runtime = AdapterRuntime::new_with_persistence(
            AppServerIngressRuntime::default(),
            Arc::new(Mutex::new(Ph1fStore::new_in_memory())),
            journal_path.clone(),
            true,
        )
        .expect("runtime with persistence must construct");
        let request = base_request();
        let envelope = fallback_runtime_execution_envelope_for_voice_turn_request(&request)
            .expect("fallback runtime execution envelope must build");
        let operation_id = derived_operation_id(&request.actor_user_id, &envelope.idempotency_key);
        let lookup_key =
            actor_idempotency_lookup_key(&request.actor_user_id, &envelope.idempotency_key);

        let response = runtime
            .run_voice_turn(request)
            .expect("voice turn must succeed with persistence");

        let state = read_persistence_state_for_test(&journal_path);
        let record = state
            .outbox_records
            .get(&operation_id)
            .expect("acknowledged outbox record must exist");
        assert_eq!(
            record.acknowledgement_state,
            PersistenceAcknowledgementState::AuthoritativelyAcknowledged
        );
        assert!(record.cleared_from_active_outbox_at_ns.is_some());
        assert_eq!(record.session_id, response.session_id);
        assert_eq!(record.turn_id, response.turn_id);
        assert!(sorted_pending_outbox_operation_ids(&state).is_empty());

        let outcome = state
            .authoritative_outcomes
            .get(&lookup_key)
            .expect("authoritative outcome must exist");
        assert_eq!(outcome.session_id, response.session_id);
        assert_eq!(outcome.turn_id, response.turn_id);

        cleanup_persistence_files_for_test(&journal_path);
    }

    #[test]
    fn at_persistence_03_stale_replay_is_rejected() {
        let journal_path = temp_persistence_journal_path("stale_replay_rejected");
        let ingress_one = AppServerIngressRuntime::default();
        let runtime_one = AdapterRuntime::new_with_persistence(
            ingress_one,
            Arc::new(Mutex::new(Ph1fStore::new_in_memory())),
            journal_path.clone(),
            true,
        )
        .expect("runtime with persistence must construct");

        let mut first = base_request();
        first.correlation_id = 30_301;
        first.turn_id = 40_301;
        first.device_turn_sequence = Some(5);
        first.now_ns = Some(10);
        let first_response = runtime_one
            .run_voice_turn(first.clone())
            .expect("first request must succeed");
        let session_id = first_response
            .session_id
            .clone()
            .expect("first response session_id must be present");

        let mut state = read_persistence_state_for_test(&journal_path);
        let mut stale_request = base_request();
        stale_request.correlation_id = 30_300;
        stale_request.turn_id = 40_300;
        stale_request.device_turn_sequence = Some(4);
        stale_request.now_ns = Some(9);
        let stale_envelope =
            fallback_runtime_execution_envelope_for_voice_turn_request(&stale_request)
                .expect("stale fallback runtime execution envelope must build");
        let stale_operation_id = derived_operation_id(
            &stale_request.actor_user_id,
            &stale_envelope.idempotency_key,
        );
        state.outbox_records.insert(
            stale_operation_id.clone(),
            AdapterOutboxRecord {
                operation_id: stale_operation_id.clone(),
                request_id: stale_envelope.request_id.clone(),
                trace_id: stale_envelope.trace_id.clone(),
                actor_user_id: stale_request.actor_user_id.clone(),
                idempotency_key: stale_envelope.idempotency_key.clone(),
                session_id: Some(session_id.clone()),
                turn_id: Some(stale_request.turn_id),
                device_id: stale_request
                    .device_id
                    .clone()
                    .expect("stale request device_id must be present"),
                device_turn_sequence: stale_request.device_turn_sequence,
                submission_timestamp_ns: stale_request
                    .now_ns
                    .expect("stale request now must exist"),
                retry_counter: 1,
                acknowledgement_state: PersistenceAcknowledgementState::PendingCloudAcknowledgement,
                cleared_from_active_outbox_at_ns: None,
                last_reconciliation_decision: None,
                last_conflict_severity: None,
                request: stale_request.clone(),
            },
        );
        write_persistence_state_for_test(&journal_path, &state);
        drop(runtime_one);

        let ingress_two = AppServerIngressRuntime::default();
        let _runtime_two = AdapterRuntime::new_with_persistence(
            ingress_two.clone(),
            Arc::new(Mutex::new(Ph1fStore::new_in_memory())),
            journal_path.clone(),
            true,
        )
        .expect("restart runtime must reconcile stale outbox record");

        let after = read_persistence_state_for_test(&journal_path);
        let stale_record = after
            .outbox_records
            .get(&stale_operation_id)
            .expect("stale outbox record must remain for auditability");
        assert_eq!(
            stale_record.acknowledgement_state,
            PersistenceAcknowledgementState::StaleRejected
        );
        assert_eq!(
            stale_record.last_reconciliation_decision,
            Some(ReconciliationDecision::RejectStaleOperation)
        );
        assert_eq!(
            stale_record.last_conflict_severity,
            Some(PersistenceConflictSeverity::StaleRejected)
        );
        assert!(after.operation_journal.iter().any(|entry| {
            entry.operation_id == stale_operation_id
                && entry.conflict_severity == Some(PersistenceConflictSeverity::StaleRejected)
        }));
        assert!(after.audit_trail.iter().any(|entry| {
            entry.operation_id.as_deref() == Some(stale_operation_id.as_str())
                && entry.decision == AdapterPersistenceAuditDecision::ReconciliationDecision
                && entry.conflict_severity == Some(PersistenceConflictSeverity::StaleRejected)
                && entry
                    .note
                    .as_deref()
                    .map(|note| note == "stale replay policy decision recorded")
                    .unwrap_or(false)
        }));
        assert!(ingress_two
            .runtime_governance_decision_log_snapshot()
            .iter()
            .any(|entry| {
                entry.reason_code
                    == selene_os::runtime_governance::reason_codes::GOV_PERSISTENCE_STALE_REJECTED
            }));

        cleanup_persistence_files_for_test(&journal_path);
    }

    #[test]
    fn at_persistence_04_lawful_retry_reuses_prior_authoritative_outcome_across_nodes() {
        let journal_path = temp_persistence_journal_path("cross_node_retry_reuse");
        let ingress_a = AppServerIngressRuntime::default();
        let first_response = with_scoped_runtime_node_id("node_a", || {
            let runtime = AdapterRuntime::new_with_persistence(
                ingress_a.clone(),
                Arc::new(Mutex::new(Ph1fStore::new_in_memory())),
                journal_path.clone(),
                true,
            )
            .expect("node_a runtime must construct");
            runtime
                .run_voice_turn(base_request())
                .expect("node_a request must succeed")
        });

        let ingress_b = AppServerIngressRuntime::default();
        let retried_response = with_scoped_runtime_node_id("node_b", || {
            let runtime = AdapterRuntime::new_with_persistence(
                ingress_b.clone(),
                Arc::new(Mutex::new(Ph1fStore::new_in_memory())),
                journal_path.clone(),
                true,
            )
            .expect("node_b runtime must construct");
            runtime
                .run_voice_turn(base_request())
                .expect("node_b retry request must reuse prior authoritative result")
        });

        assert_eq!(retried_response.session_id, first_response.session_id);
        assert_eq!(retried_response.turn_id, first_response.turn_id);
        assert_eq!(
            retried_response.session_attach_outcome,
            Some(SessionAttachOutcome::RetryReusedResult)
        );

        let state = read_persistence_state_for_test(&journal_path);
        assert!(state.audit_trail.iter().any(|entry| {
            entry.decision == AdapterPersistenceAuditDecision::DedupeOutcome
                && entry
                    .note
                    .as_deref()
                    .map(|note| {
                        note.contains(
                            "cross-node dedupe reused authoritative outcome from node node_a",
                        )
                    })
                    .unwrap_or(false)
        }));
        assert!(state
            .authoritative_outcomes
            .values()
            .any(|outcome| outcome.governance_policy_version.is_some()));
        assert!(ingress_b
            .runtime_governance_decision_log_snapshot()
            .iter()
            .any(|entry| {
                entry.reason_code
                    == selene_os::runtime_governance::reason_codes::GOV_POLICY_VERSION_DRIFT
            }));

        cleanup_persistence_files_for_test(&journal_path);
    }

    #[test]
    fn at_persistence_05_inconsistent_persistence_is_quarantined() {
        let journal_path = temp_persistence_journal_path("quarantine_inconsistent");
        let ingress_one = AppServerIngressRuntime::default();
        let runtime_one = AdapterRuntime::new_with_persistence(
            ingress_one.clone(),
            Arc::new(Mutex::new(Ph1fStore::new_in_memory())),
            journal_path.clone(),
            true,
        )
        .expect("runtime with persistence must construct");
        let request = base_request();
        let envelope = fallback_runtime_execution_envelope_for_voice_turn_request(&request)
            .expect("fallback runtime execution envelope must build");
        let operation_id = derived_operation_id(&request.actor_user_id, &envelope.idempotency_key);

        runtime_one
            .prepare_persistence_operation(
                &request,
                &envelope,
                MonotonicTimeNs(request.now_ns.expect("request now must be present")),
                PersistenceInvocationMode::Standard,
            )
            .expect("persistence prepare must succeed")
            .expect("prepared persistence operation must exist");

        let mut state = read_persistence_state_for_test(&journal_path);
        let row = state
            .outbox_records
            .get_mut(&operation_id)
            .expect("pending outbox record must exist");
        row.device_id.clear();
        write_persistence_state_for_test(&journal_path, &state);
        drop(runtime_one);

        let ingress_two = AppServerIngressRuntime::default();
        let _runtime_two = AdapterRuntime::new_with_persistence(
            ingress_two.clone(),
            Arc::new(Mutex::new(Ph1fStore::new_in_memory())),
            journal_path.clone(),
            true,
        )
        .expect("restart runtime must quarantine inconsistent persistence");

        let after = read_persistence_state_for_test(&journal_path);
        let quarantined_record = after
            .outbox_records
            .get(&operation_id)
            .expect("quarantined outbox record must remain visible");
        assert_eq!(
            quarantined_record.acknowledgement_state,
            PersistenceAcknowledgementState::QuarantinedLocalState
        );
        assert_eq!(
            quarantined_record.last_conflict_severity,
            Some(PersistenceConflictSeverity::QuarantineRequired)
        );
        assert_eq!(
            after.recovery_mode,
            PersistenceRecoveryMode::QuarantinedLocalState
        );
        assert!(after.audit_trail.iter().any(|entry| {
            entry.decision == AdapterPersistenceAuditDecision::QuarantineAction
                && entry.conflict_severity == Some(PersistenceConflictSeverity::QuarantineRequired)
        }));
        assert!(ingress_two
            .runtime_governance_decision_log_snapshot()
            .iter()
            .any(|entry| {
                entry.reason_code
                    == selene_os::runtime_governance::reason_codes::GOV_PERSISTENCE_QUARANTINE_REQUIRED
            }));

        cleanup_persistence_files_for_test(&journal_path);
    }

    #[test]
    fn at_persistence_06_recovery_modes_transition_deterministically() {
        let journal_path = temp_persistence_journal_path("recovery_mode_transitions");
        let runtime = AdapterRuntime::new_with_persistence(
            AppServerIngressRuntime::default(),
            Arc::new(Mutex::new(Ph1fStore::new_in_memory())),
            journal_path.clone(),
            true,
        )
        .expect("runtime with persistence must construct");

        let mut request = base_request();
        request.correlation_id = 31_006;
        request.turn_id = 41_006;
        request.now_ns = Some(50);
        let envelope = fallback_runtime_execution_envelope_for_voice_turn_request(&request)
            .expect("fallback runtime execution envelope must build");
        let prepared = runtime
            .prepare_persistence_operation(
                &request,
                &envelope,
                MonotonicTimeNs(50),
                PersistenceInvocationMode::Standard,
            )
            .expect("persistence prepare must succeed")
            .expect("prepared persistence operation must exist");
        let initial_state = read_persistence_state_for_test(&journal_path);
        assert_eq!(initial_state.recovery_mode, PersistenceRecoveryMode::Normal);

        runtime
            .finalize_persistence_operation(
                &prepared,
                MonotonicTimeNs(50),
                &DeviceId::new(
                    request
                        .device_id
                        .clone()
                        .expect("device_id must exist for finalize test"),
                )
                .expect("device_id must parse"),
                &envelope.idempotency_key,
                &Err(voice_turn_ingress_error(
                    FailureClass::RetryableRuntime,
                    "RETRY_RESULT_MISSING".to_string(),
                    Some("retry_result_missing".to_string()),
                    None,
                    Some(request.turn_id),
                    None,
                )),
            )
            .expect("retryable persistence finalization must succeed");

        let degraded_state = read_persistence_state_for_test(&journal_path);
        assert_eq!(
            degraded_state.recovery_mode,
            PersistenceRecoveryMode::DegradedRecovery
        );
        let operation_id = derived_operation_id(&request.actor_user_id, &envelope.idempotency_key);
        let degraded_record = degraded_state
            .outbox_records
            .get(&operation_id)
            .expect("degraded outbox record must exist");
        assert_eq!(
            degraded_record.last_conflict_severity,
            Some(PersistenceConflictSeverity::Retryable)
        );

        let mut quarantined_state = degraded_state.clone();
        let row = quarantined_state
            .outbox_records
            .get_mut(&operation_id)
            .expect("degraded outbox row must exist");
        row.device_id.clear();
        write_persistence_state_for_test(&journal_path, &quarantined_state);
        drop(runtime);

        let _runtime_after_restart = AdapterRuntime::new_with_persistence(
            AppServerIngressRuntime::default(),
            Arc::new(Mutex::new(Ph1fStore::new_in_memory())),
            journal_path.clone(),
            true,
        )
        .expect("restart runtime must process quarantined persistence");

        let final_state = read_persistence_state_for_test(&journal_path);
        let transitions = final_state
            .audit_trail
            .iter()
            .filter(|entry| {
                entry.decision == AdapterPersistenceAuditDecision::RecoveryModeTransition
            })
            .map(|entry| entry.recovery_mode)
            .collect::<Vec<_>>();
        assert!(transitions.contains(&PersistenceRecoveryMode::DegradedRecovery));
        assert!(transitions.contains(&PersistenceRecoveryMode::QuarantinedLocalState));
        assert_eq!(
            final_state.recovery_mode,
            PersistenceRecoveryMode::QuarantinedLocalState
        );

        cleanup_persistence_files_for_test(&journal_path);
    }

    #[test]
    fn at_persistence_07_restart_reconciliation_requests_fresh_session_state_and_converges() {
        let journal_path = temp_persistence_journal_path("fresh_session_state_reconcile");
        let runtime_one = AdapterRuntime::new_with_persistence(
            AppServerIngressRuntime::default(),
            Arc::new(Mutex::new(Ph1fStore::new_in_memory())),
            journal_path.clone(),
            true,
        )
        .expect("runtime with persistence must construct");

        let mut request = base_request();
        request.correlation_id = 31_007;
        request.turn_id = 41_007;
        request.now_ns = Some(70);
        let envelope = fallback_runtime_execution_envelope_for_voice_turn_request(&request)
            .expect("fallback runtime execution envelope must build");
        let operation_id = derived_operation_id(&request.actor_user_id, &envelope.idempotency_key);

        runtime_one
            .prepare_persistence_operation(
                &request,
                &envelope,
                MonotonicTimeNs(70),
                PersistenceInvocationMode::Standard,
            )
            .expect("persistence prepare must succeed")
            .expect("prepared persistence operation must exist");

        let mut state = read_persistence_state_for_test(&journal_path);
        let row = state
            .outbox_records
            .get_mut(&operation_id)
            .expect("pending outbox record must exist");
        row.session_id = Some("1".to_string());
        row.turn_id = Some(request.turn_id);
        write_persistence_state_for_test(&journal_path, &state);
        drop(runtime_one);

        let _runtime_two = AdapterRuntime::new_with_persistence(
            AppServerIngressRuntime::default(),
            Arc::new(Mutex::new(Ph1fStore::new_in_memory())),
            journal_path.clone(),
            true,
        )
        .expect("restart runtime must reconcile with fresh-session-state request");

        let after = read_persistence_state_for_test(&journal_path);
        let record = after
            .outbox_records
            .get(&operation_id)
            .expect("reconciled outbox record must remain visible");
        assert_eq!(
            record.acknowledgement_state,
            PersistenceAcknowledgementState::AuthoritativelyAcknowledged
        );
        assert_eq!(
            record.last_reconciliation_decision,
            Some(ReconciliationDecision::RequestFreshSessionState)
        );
        assert_eq!(
            record.last_conflict_severity,
            Some(PersistenceConflictSeverity::Info)
        );
        assert!(after.operation_journal.iter().any(|entry| {
            entry.operation_id == operation_id
                && entry.event_kind == AdapterOperationJournalEventKind::FreshSessionStateRequested
                && entry.reconciliation_decision
                    == Some(ReconciliationDecision::RequestFreshSessionState)
        }));
        assert!(after.audit_trail.iter().any(|entry| {
            entry.operation_id.as_deref() == Some(operation_id.as_str())
                && entry.decision == AdapterPersistenceAuditDecision::ReconciliationDecision
                && entry
                    .note
                    .as_deref()
                    .map(|note| note == "reconciliation policy decision recorded")
                    .unwrap_or(false)
        }));
        assert_eq!(after.recovery_mode, PersistenceRecoveryMode::Normal);
        assert!(after.last_reconciled_at_ns.is_some());

        cleanup_persistence_files_for_test(&journal_path);
    }

    #[test]
    fn at_persistence_08_outbox_identity_integrity_break_is_quarantined_on_bootstrap() {
        let journal_path = temp_persistence_journal_path("outbox_identity_integrity");
        let runtime_one = AdapterRuntime::new_with_persistence(
            AppServerIngressRuntime::default(),
            Arc::new(Mutex::new(Ph1fStore::new_in_memory())),
            journal_path.clone(),
            true,
        )
        .expect("runtime with persistence must construct");
        let request = base_request();
        let envelope = fallback_runtime_execution_envelope_for_voice_turn_request(&request)
            .expect("fallback runtime execution envelope must build");
        let operation_id = derived_operation_id(&request.actor_user_id, &envelope.idempotency_key);

        runtime_one
            .prepare_persistence_operation(
                &request,
                &envelope,
                MonotonicTimeNs(request.now_ns.expect("request now must be present")),
                PersistenceInvocationMode::Standard,
            )
            .expect("persistence prepare must succeed")
            .expect("prepared persistence operation must exist");

        let mut state = read_persistence_state_for_test(&journal_path);
        let row = state
            .outbox_records
            .get_mut(&operation_id)
            .expect("pending outbox record must exist");
        row.operation_id = "op_integrity_mismatch".to_string();
        write_persistence_state_for_test(&journal_path, &state);
        drop(runtime_one);

        let ingress_two = AppServerIngressRuntime::default();
        let _runtime_two = AdapterRuntime::new_with_persistence(
            ingress_two.clone(),
            Arc::new(Mutex::new(Ph1fStore::new_in_memory())),
            journal_path.clone(),
            true,
        )
        .expect("restart runtime must quarantine structurally invalid outbox state");

        let after = read_persistence_state_for_test(&journal_path);
        assert_eq!(
            after.recovery_mode,
            PersistenceRecoveryMode::QuarantinedLocalState
        );
        assert!(after.outbox_records.is_empty());
        assert!(after.audit_trail.iter().any(|entry| {
            entry.decision == AdapterPersistenceAuditDecision::QuarantineAction
                && entry
                    .note
                    .as_deref()
                    .map(|note| note.contains("integrity validation quarantined"))
                    .unwrap_or(false)
                && entry
                    .note
                    .as_deref()
                    .map(|note| note.contains("outbox key"))
                    .unwrap_or(false)
        }));
        let quarantined_state_path = quarantined_persistence_path(
            &adapter_persistence_state_path(&journal_path),
            "state_integrity",
        );
        assert!(quarantined_state_path.exists());
        assert!(ingress_two
            .runtime_governance_decision_log_snapshot()
            .iter()
            .any(|entry| {
                entry.reason_code
                    == selene_os::runtime_governance::reason_codes::GOV_PERSISTENCE_QUARANTINE_REQUIRED
            }));

        cleanup_persistence_files_for_test(&journal_path);
    }

    #[test]
    fn at_persistence_09_journal_sequence_integrity_break_is_quarantined_on_bootstrap() {
        let journal_path = temp_persistence_journal_path("journal_sequence_integrity");
        let runtime_one = AdapterRuntime::new_with_persistence(
            AppServerIngressRuntime::default(),
            Arc::new(Mutex::new(Ph1fStore::new_in_memory())),
            journal_path.clone(),
            true,
        )
        .expect("runtime with persistence must construct");
        let request = base_request();
        let envelope = fallback_runtime_execution_envelope_for_voice_turn_request(&request)
            .expect("fallback runtime execution envelope must build");

        runtime_one
            .prepare_persistence_operation(
                &request,
                &envelope,
                MonotonicTimeNs(request.now_ns.expect("request now must be present")),
                PersistenceInvocationMode::Standard,
            )
            .expect("persistence prepare must succeed")
            .expect("prepared persistence operation must exist");

        let mut state = read_persistence_state_for_test(&journal_path);
        let first_entry = state
            .operation_journal
            .first_mut()
            .expect("operation journal must contain created entry");
        first_entry.sequence = 9;
        write_persistence_state_for_test(&journal_path, &state);
        drop(runtime_one);

        let ingress_two = AppServerIngressRuntime::default();
        let _runtime_two = AdapterRuntime::new_with_persistence(
            ingress_two.clone(),
            Arc::new(Mutex::new(Ph1fStore::new_in_memory())),
            journal_path.clone(),
            true,
        )
        .expect("restart runtime must quarantine structurally invalid journal state");

        let after = read_persistence_state_for_test(&journal_path);
        assert_eq!(
            after.recovery_mode,
            PersistenceRecoveryMode::QuarantinedLocalState
        );
        assert!(after.operation_journal.is_empty());
        assert!(after.audit_trail.iter().any(|entry| {
            entry.decision == AdapterPersistenceAuditDecision::QuarantineAction
                && entry
                    .note
                    .as_deref()
                    .map(|note| note.contains("integrity validation quarantined"))
                    .unwrap_or(false)
                && entry
                    .note
                    .as_deref()
                    .map(|note| note.contains("operation_journal sequence"))
                    .unwrap_or(false)
        }));
        let quarantined_state_path = quarantined_persistence_path(
            &adapter_persistence_state_path(&journal_path),
            "state_integrity",
        );
        assert!(quarantined_state_path.exists());
        assert!(ingress_two
            .runtime_governance_decision_log_snapshot()
            .iter()
            .any(|entry| {
                entry.reason_code
                    == selene_os::runtime_governance::reason_codes::GOV_PERSISTENCE_QUARANTINE_REQUIRED
            }));

        cleanup_persistence_files_for_test(&journal_path);
    }

    #[test]
    fn at_persistence_10_retryable_post_session_failure_recovers_pending_state_after_restart() {
        let journal_path = temp_persistence_journal_path("retryable_post_session_restart");
        let runtime_one = AdapterRuntime::new_with_persistence(
            AppServerIngressRuntime::default(),
            Arc::new(Mutex::new(Ph1fStore::new_in_memory())),
            journal_path.clone(),
            true,
        )
        .expect("runtime with persistence must construct");

        let mut request = base_request();
        request.correlation_id = 31_010;
        request.turn_id = 41_010;
        request.now_ns = Some(90);
        let envelope = fallback_runtime_execution_envelope_for_voice_turn_request(&request)
            .expect("fallback runtime execution envelope must build");
        let operation_id = derived_operation_id(&request.actor_user_id, &envelope.idempotency_key);
        let prepared = runtime_one
            .prepare_persistence_operation(
                &request,
                &envelope,
                MonotonicTimeNs(90),
                PersistenceInvocationMode::Standard,
            )
            .expect("persistence prepare must succeed")
            .expect("prepared persistence operation must exist");

        runtime_one
            .finalize_persistence_operation(
                &prepared,
                MonotonicTimeNs(90),
                &DeviceId::new(
                    request
                        .device_id
                        .clone()
                        .expect("device_id must exist for restart recovery test"),
                )
                .expect("device_id must parse"),
                &envelope.idempotency_key,
                &Err(voice_turn_ingress_error(
                    FailureClass::RetryableRuntime,
                    "RETRY_RESULT_MISSING".to_string(),
                    Some("retry_result_missing".to_string()),
                    Some("1".to_string()),
                    Some(request.turn_id),
                    Some(SessionState::Active),
                )),
            )
            .expect("retryable persistence finalization must succeed");

        let state_after_failure = read_persistence_state_for_test(&journal_path);
        let degraded_record = state_after_failure
            .outbox_records
            .get(&operation_id)
            .expect("degraded outbox record must exist");
        assert_eq!(degraded_record.session_id.as_deref(), Some("1"));
        assert_eq!(degraded_record.turn_id, Some(request.turn_id));
        assert_eq!(
            degraded_record.acknowledgement_state,
            PersistenceAcknowledgementState::PendingCloudAcknowledgement
        );
        assert_eq!(
            degraded_record.last_reconciliation_decision,
            Some(ReconciliationDecision::RetrySameOperation)
        );
        assert_eq!(
            degraded_record.last_conflict_severity,
            Some(PersistenceConflictSeverity::Retryable)
        );
        assert_eq!(
            state_after_failure.recovery_mode,
            PersistenceRecoveryMode::DegradedRecovery
        );
        assert_eq!(
            sorted_pending_outbox_operation_ids(&state_after_failure),
            vec![operation_id.clone()]
        );
        assert!(state_after_failure.operation_journal.iter().any(|entry| {
            entry.operation_id == operation_id
                && entry.event_kind == AdapterOperationJournalEventKind::RetryAttempted
                && entry.session_id.as_deref() == Some("1")
                && entry.turn_id == Some(request.turn_id)
                && entry.reconciliation_decision == Some(ReconciliationDecision::RetrySameOperation)
        }));

        drop(runtime_one);

        let _runtime_two = AdapterRuntime::new_with_persistence(
            AppServerIngressRuntime::default(),
            Arc::new(Mutex::new(Ph1fStore::new_in_memory())),
            journal_path.clone(),
            true,
        )
        .expect("restart runtime must reconcile retryable pending operation");

        let after_restart = read_persistence_state_for_test(&journal_path);
        let recovered_record = after_restart
            .outbox_records
            .get(&operation_id)
            .expect("recovered outbox record must remain visible");
        assert_eq!(
            recovered_record.acknowledgement_state,
            PersistenceAcknowledgementState::AuthoritativelyAcknowledged
        );
        assert_eq!(recovered_record.session_id.as_deref(), Some("1"));
        assert_eq!(recovered_record.turn_id, Some(request.turn_id));
        assert_eq!(recovered_record.retry_counter, 2);
        assert_eq!(
            recovered_record.last_reconciliation_decision,
            Some(ReconciliationDecision::RequestFreshSessionState)
        );
        assert_eq!(
            recovered_record.last_conflict_severity,
            Some(PersistenceConflictSeverity::Info)
        );
        assert!(recovered_record.cleared_from_active_outbox_at_ns.is_some());
        assert!(after_restart.operation_journal.iter().any(|entry| {
            entry.operation_id == operation_id
                && entry.event_kind == AdapterOperationJournalEventKind::FreshSessionStateRequested
                && entry.session_id.as_deref() == Some("1")
                && entry.reconciliation_decision
                    == Some(ReconciliationDecision::RequestFreshSessionState)
        }));
        let transitions = after_restart
            .audit_trail
            .iter()
            .filter(|entry| {
                entry.decision == AdapterPersistenceAuditDecision::RecoveryModeTransition
            })
            .map(|entry| entry.recovery_mode)
            .collect::<Vec<_>>();
        assert!(transitions.contains(&PersistenceRecoveryMode::DegradedRecovery));
        assert!(transitions.contains(&PersistenceRecoveryMode::Recovering));
        assert_eq!(after_restart.recovery_mode, PersistenceRecoveryMode::Normal);
        assert!(after_restart.last_reconciled_at_ns.is_some());

        cleanup_persistence_files_for_test(&journal_path);
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
        seed_wake_enrollment_complete_for_request(&runtime, &mut android, "at_adapter_08_android");
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
    fn at_adapter_36_sync_retry_improvement_and_builder_observation_remain_downstream_only() {
        let runtime = AdapterRuntime::default();
        let correlation_id_raw = 10_136_u64;
        let turn_id_raw = 20_136_u64;
        let correlation_id = CorrelationId(correlation_id_raw as u128);
        let turn_id = TurnId(turn_id_raw);
        let improvement_now = MonotonicTimeNs(36_500);
        let worker_id = "at_adapter_36_worker".to_string();

        let mut request = base_request();
        request.correlation_id = correlation_id_raw;
        request.turn_id = turn_id_raw;
        request.now_ns = Some(36_000);
        request.app_platform = "ANDROID".to_string();
        request.trigger = "WAKE_WORD".to_string();
        request.actor_user_id = "tenant_a:user_adapter_test_36".to_string();
        request.tenant_id = Some("tenant_a".to_string());
        request.device_id = Some("adapter_android_device_36".to_string());

        seed_wake_enrollment_complete_for_request(&runtime, &mut request, "at_adapter_36_retry");

        let actor_user_id =
            UserId::new(request.actor_user_id.clone()).expect("actor_user_id must parse");
        let device_id = DeviceId::new(
            request
                .device_id
                .clone()
                .expect("device_id must be present for test"),
        )
        .expect("device_id must parse");

        let (original_sync_job_id, original_retry_row_before, queue_before) = {
            let mut store = runtime.store.lock().expect("store lock should succeed");
            let enrollment_rows = store
                .ph1w_all_enrollment_session_rows()
                .into_iter()
                .filter(|row| row.user_id == actor_user_id && row.device_id == device_id)
                .collect::<Vec<_>>();
            assert_eq!(
                enrollment_rows.len(),
                1,
                "wake enrollment seed should create one session for this actor/device"
            );
            let receipt_ref = enrollment_rows[0]
                .wake_artifact_sync_receipt_ref
                .clone()
                .expect("wake enrollment seed should produce sync receipt");
            let original_row = store
                .mobile_artifact_sync_queue_row_for_receipt(&receipt_ref)
                .cloned()
                .expect("wake receipt should resolve queue row");
            assert_eq!(original_row.state, MobileArtifactSyncState::Queued);
            assert_eq!(original_row.user_id, Some(actor_user_id.clone()));
            assert_eq!(original_row.device_id, device_id);

            let dequeued = store
                .device_artifact_sync_dequeue_batch(improvement_now, 1, 1_000, worker_id.clone())
                .expect("retry dequeue should succeed");
            assert_eq!(dequeued.len(), 1, "fresh retry path should dequeue one row");
            assert_eq!(dequeued[0].sync_job_id, original_row.sync_job_id);
            store
                .device_artifact_sync_fail_commit(
                    improvement_now,
                    &original_row.sync_job_id,
                    Some(worker_id.as_str()),
                    "retry_pending_for_improvement".to_string(),
                    1_000,
                )
                .expect("retry fail commit should succeed");

            let retry_row_before = store
                .device_artifact_sync_queue_rows()
                .iter()
                .find(|row| row.sync_job_id == original_row.sync_job_id)
                .cloned()
                .expect("retry row should remain present");
            let queue_before = snapshot_sync_queue_counters(&store, improvement_now);
            (original_row.sync_job_id, retry_row_before, queue_before)
        };

        let health_before = runtime
            .health_report(Some(improvement_now.0))
            .expect("pre-improvement health report should succeed");
        let worker_before = health_before.sync.worker.clone();
        let improvement_before = health_before.sync.improvement.clone();
        assert_eq!(health_before.sync.queue, queue_before);

        let (
            emission,
            feedback_before,
            feedback_after,
            bundle_before,
            bundle_after,
            outcome_before,
            outcome_after,
            learn_artifact_before,
            learn_artifact_after,
            original_retry_row_after,
            queue_after,
            new_queue_rows,
        ) = {
            let mut store = runtime.store.lock().expect("store lock should succeed");
            let feedback_before = store.ph1feedback_audit_rows(correlation_id).len();
            let bundle_before = store
                .ph1feedback_learn_signal_bundle_rows(correlation_id)
                .len();
            let outcome_before = store
                .outcome_utilization_ledger_rows()
                .iter()
                .filter(|row| {
                    row.correlation_id == correlation_id
                        && row.engine_id == "PH1.FEEDBACK"
                        && row.consumed_by == "PH1.LEARN"
                        && row.outcome_type == "VOICE_SYNC_RETRY"
                })
                .count();
            let tenant_scope =
                tenant_scope_from_user_id(&actor_user_id).expect("tenant scope must resolve");
            let learn_artifact_before = store
                .ph1learn_artifact_rows(
                    ArtifactScopeType::Tenant,
                    tenant_scope,
                    artifact_type_for_sync_issue(SyncIssueKind::Retry),
                )
                .len();
            let queue_rows_before = store.device_artifact_sync_queue_rows().to_vec();
            let before_sync_job_ids = queue_rows_before
                .iter()
                .map(|row| row.sync_job_id.clone())
                .collect::<Vec<_>>();

            let emission = runtime
                .emit_sync_improvement_events(
                    &mut store,
                    improvement_now,
                    correlation_id,
                    turn_id,
                    &DeviceArtifactSyncWorkerPassMetrics::default(),
                    &queue_before,
                )
                .expect("retry improvement emission should succeed");
            runtime
                .maybe_run_builder_for_sync_improvements(
                    &mut store,
                    SyncImprovementBuilderContext {
                        now: improvement_now,
                        correlation_id,
                        turn_id,
                        metrics: &DeviceArtifactSyncWorkerPassMetrics::default(),
                        queue_after: &queue_before,
                        outcome_entries: &emission.builder_input_entries,
                    },
                )
                .expect("builder observation should succeed");
            runtime
                .record_sync_improvement_metrics(&emission)
                .expect("improvement metrics update should succeed");

            let feedback_after = store.ph1feedback_audit_rows(correlation_id).len();
            let bundle_after = store
                .ph1feedback_learn_signal_bundle_rows(correlation_id)
                .len();
            let outcome_after = store
                .outcome_utilization_ledger_rows()
                .iter()
                .filter(|row| {
                    row.correlation_id == correlation_id
                        && row.engine_id == "PH1.FEEDBACK"
                        && row.consumed_by == "PH1.LEARN"
                        && row.outcome_type == "VOICE_SYNC_RETRY"
                })
                .count();
            let learn_artifact_after = store
                .ph1learn_artifact_rows(
                    ArtifactScopeType::Tenant,
                    tenant_scope,
                    artifact_type_for_sync_issue(SyncIssueKind::Retry),
                )
                .len();
            let original_retry_row_after = store
                .device_artifact_sync_queue_rows()
                .iter()
                .find(|row| row.sync_job_id == original_sync_job_id)
                .cloned()
                .expect("original retry row should remain present after downstream emission");
            let queue_after = snapshot_sync_queue_counters(&store, improvement_now);
            let new_queue_rows = store
                .device_artifact_sync_queue_rows()
                .iter()
                .filter(|row| !before_sync_job_ids.iter().any(|id| id == &row.sync_job_id))
                .cloned()
                .collect::<Vec<_>>();

            (
                emission,
                feedback_before,
                feedback_after,
                bundle_before,
                bundle_after,
                outcome_before,
                outcome_after,
                learn_artifact_before,
                learn_artifact_after,
                original_retry_row_after,
                queue_after,
                new_queue_rows,
            )
        };

        let health_after = runtime
            .health_report(Some(improvement_now.0))
            .expect("post-improvement health report should succeed");

        assert!(queue_before.retry_pending_count > 0);
        assert_eq!(queue_before.dead_letter_count, 0);
        assert_eq!(queue_before.replay_due_count, 0);
        assert!(feedback_after > feedback_before);
        assert_eq!(bundle_after, bundle_before);
        assert!(outcome_after > outcome_before);
        assert!(learn_artifact_after > learn_artifact_before);
        assert!(emission.feedback_events_emitted > 0);
        assert!(emission.learn_artifacts_emitted > 0);
        assert!(!emission.builder_input_entries.is_empty());
        assert!(emission.builder_input_entries.iter().any(|entry| {
            entry.engine_id == "PH1.FEEDBACK"
                && entry.outcome_type == "VOICE_SYNC_RETRY"
                && entry.consumed_by == "PH1.LEARN"
        }));
        assert_eq!(
            health_after.sync.improvement.last_builder_status.as_deref(),
            Some("SKIPPED_NON_SEVERE")
        );
        assert_eq!(
            health_after.sync.improvement.builder_not_invoked_total,
            improvement_before.builder_not_invoked_total + 1
        );
        assert_eq!(
            health_after.sync.improvement.builder_runs_total,
            improvement_before.builder_runs_total
        );
        assert_eq!(
            health_after.sync.improvement.feedback_events_emitted_total,
            improvement_before.feedback_events_emitted_total + emission.feedback_events_emitted
        );
        assert_eq!(
            health_after.sync.improvement.learn_artifacts_emitted_total,
            improvement_before.learn_artifacts_emitted_total + emission.learn_artifacts_emitted
        );
        assert_eq!(health_after.sync.worker, worker_before);
        assert_eq!(health_after.sync.queue, queue_after);

        assert_eq!(
            original_retry_row_after.sync_job_id,
            original_retry_row_before.sync_job_id
        );
        assert_eq!(
            original_retry_row_after.state,
            original_retry_row_before.state
        );
        assert_eq!(
            original_retry_row_after.attempt_count,
            original_retry_row_before.attempt_count
        );
        assert_eq!(
            original_retry_row_after.worker_id,
            original_retry_row_before.worker_id
        );
        assert_eq!(
            original_retry_row_after.last_error,
            original_retry_row_before.last_error
        );
        assert_eq!(
            original_retry_row_after.lease_expires_at,
            original_retry_row_before.lease_expires_at
        );
        assert_eq!(
            original_retry_row_after.last_attempted_at,
            original_retry_row_before.last_attempted_at
        );
        assert_eq!(queue_after.in_flight_count, queue_before.in_flight_count);
        assert_eq!(
            queue_after.retry_pending_count,
            queue_before.retry_pending_count
        );
        assert_eq!(
            queue_after.dead_letter_count,
            queue_before.dead_letter_count
        );
        assert_eq!(queue_after.replay_due_count, queue_before.replay_due_count);
        assert_eq!(queue_after.queued_count, queue_before.queued_count + 1);
        assert_eq!(
            new_queue_rows.len(),
            1,
            "retry improvement fan-out should append exactly one queued manifest row"
        );
        assert_ne!(new_queue_rows[0].sync_job_id, original_sync_job_id);
        assert_eq!(new_queue_rows[0].state, MobileArtifactSyncState::Queued);
        assert_eq!(new_queue_rows[0].attempt_count, 0);
        assert_eq!(new_queue_rows[0].worker_id, None);
        assert_eq!(new_queue_rows[0].last_error, None);
        assert_eq!(new_queue_rows[0].last_attempted_at, None);
        assert_eq!(new_queue_rows[0].lease_expires_at, None);
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

        let set_target =
            runtime.ui_health_report_query(base_report_query_request(), Some(5_000_000_001));
        assert_eq!(set_target.status, "ok");
        assert_eq!(
            set_target.display_target_applied.as_deref(),
            Some("desktop")
        );
        assert!(set_target.requires_clarification.is_none());

        let mut remembered_req = base_report_query_request();
        remembered_req.display_target = None;
        let remembered = runtime.ui_health_report_query(remembered_req, Some(5_000_000_002));
        assert_eq!(remembered.status, "ok");
        assert_eq!(
            remembered.display_target_applied.as_deref(),
            Some("desktop")
        );
        assert!(remembered.requires_clarification.is_none());
    }

    #[test]
    fn at_adapter_16_report_query_context_supports_follow_up_patch() {
        let runtime = AdapterRuntime::default();
        let first =
            runtime.ui_health_report_query(base_report_query_request(), Some(5_000_000_100));
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
        assert_eq!(
            second.report_context_id.as_deref(),
            Some(context_id.as_str())
        );
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
                blocker: Some("Retry queue backlog not drained.".to_string()),
                unresolved_deadline_at_ns: Some(500),
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
                blocker: Some("Dead-letter queue is non-zero.".to_string()),
                unresolved_deadline_at_ns: Some(501),
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
                blocker: Some("Replay-due queue exceeds threshold.".to_string()),
                unresolved_deadline_at_ns: Some(502),
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
                evidence_ref: Some("sync.queue.dead_letter_count".to_string()),
                blocker: Some("Dead-letter queue is non-zero.".to_string()),
                unresolved_deadline_at_ns: Some(540),
            },
            UiHealthTimelineEntry {
                issue_id: "sync_dead_letter".to_string(),
                at_ns: Some(300),
                action_id: "A3".to_string(),
                result: "r3".to_string(),
                reason_code: "3".to_string(),
                evidence_ref: Some("sync.queue.dead_letter_count".to_string()),
                blocker: Some("Dead-letter queue is non-zero.".to_string()),
                unresolved_deadline_at_ns: Some(530),
            },
            UiHealthTimelineEntry {
                issue_id: "sync_dead_letter".to_string(),
                at_ns: Some(200),
                action_id: "A2".to_string(),
                result: "r2".to_string(),
                reason_code: "2".to_string(),
                evidence_ref: Some("sync.queue.dead_letter_count".to_string()),
                blocker: Some("Dead-letter queue is non-zero.".to_string()),
                unresolved_deadline_at_ns: Some(520),
            },
            UiHealthTimelineEntry {
                issue_id: "sync_dead_letter".to_string(),
                at_ns: Some(100),
                action_id: "A1".to_string(),
                result: "r1".to_string(),
                reason_code: "1".to_string(),
                evidence_ref: Some("sync.queue.dead_letter_count".to_string()),
                blocker: Some("Dead-letter queue is non-zero.".to_string()),
                unresolved_deadline_at_ns: Some(510),
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
            (1_u64, "IOS", "EXPLICIT", "ios_1"),
            (2_u64, "ANDROID", "WAKE_WORD", "android_1"),
            (3_u64, "DESKTOP", "EXPLICIT", "desktop_1"),
        ] {
            let mut req = base_request();
            req.turn_id = 20_100 + idx;
            req.now_ns = Some(10_000 + idx);
            req.app_platform = platform.to_string();
            req.trigger = trigger.to_string();
            req.device_id = Some(device_id.to_string());
            if trigger == "WAKE_WORD" {
                seed_wake_enrollment_complete_for_request(&runtime, &mut req, "at_adapter_21");
            }
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
        voice_turn.app_platform = "DESKTOP".to_string();
        voice_turn.trigger = "WAKE_WORD".to_string();
        voice_turn.device_id = Some("adapter_voice_text_wake_desktop".to_string());
        voice_turn.user_text_final = Some("show missed stt report for june".to_string());
        voice_turn.selene_text_final = Some("Opening the report on desktop.".to_string());
        seed_wake_enrollment_complete_for_request(&runtime, &mut voice_turn, "at_adapter_22");
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
    fn at_adapter_33_ph1c_live_bootstrap_gold_capture_and_telemetry_are_always_on() {
        let runtime = AdapterRuntime::default();
        let mut req = base_request();
        req.turn_id = 20_333;
        req.now_ns = Some(33_003);
        req.user_text_partial = None;
        req.user_text_final = None;
        req.selene_text_partial = None;
        req.selene_text_final = None;
        runtime
            .run_voice_turn(req)
            .expect("live voice turn should succeed");

        let store = runtime.store.lock().expect("store lock must not poison");
        let correlation_id = CorrelationId(10_001);
        let ph1c_audits = store
            .audit_events()
            .iter()
            .filter(|event| {
                event.correlation_id == correlation_id
                    && matches!(
                        event.engine,
                        selene_kernel_contracts::ph1j::AuditEngine::Ph1C
                    )
            })
            .count();
        assert!(ph1c_audits >= 1);

        let feedback_rows = store.ph1feedback_audit_rows(correlation_id);
        assert!(!feedback_rows.is_empty());
        let learn_rows = store.ph1feedback_learn_signal_bundle_rows(correlation_id);
        assert!(!learn_rows.is_empty());

        let telemetry_rows = store
            .outcome_utilization_ledger_rows()
            .iter()
            .filter(|row| {
                row.correlation_id == correlation_id
                    && row.engine_id == "PH1.C"
                    && row.consumed_by == "PH1.C.SUPERIORITY"
            })
            .count();
        assert!(telemetry_rows >= 1);
    }

    #[test]
    fn at_adapter_34_ph1d_runtime_commit_writes_full_payload_contract() {
        let runtime = AdapterRuntime::default();
        let mut req = base_request();
        req.turn_id = 20_334;
        req.now_ns = Some(33_004);
        req.user_text_final = Some("set reminder for payroll review friday".to_string());
        runtime
            .run_voice_turn(req)
            .expect("voice turn should succeed");

        let store = runtime.store.lock().expect("store lock must not poison");
        let row = store
            .audit_events()
            .iter()
            .find(|event| {
                event.correlation_id == CorrelationId(10_001)
                    && matches!(
                        event.engine,
                        selene_kernel_contracts::ph1j::AuditEngine::Ph1D
                    )
            })
            .expect("PH1.D audit row must exist");
        let entries = &row.payload_min.entries;
        for key in [
            "decision",
            "output_mode",
            "request_id",
            "prompt_template_version",
            "output_schema_hash",
            "tool_catalog_hash",
            "policy_context_hash",
            "transcript_hash",
            "model_id",
            "model_route_class",
            "temperature_bp",
            "max_tokens",
        ] {
            assert!(
                entries.contains_key(&selene_kernel_contracts::ph1j::PayloadKey::new(key).unwrap()),
                "missing PH1.D payload key: {key}"
            );
        }
    }

    #[test]
    fn at_adapter_35_ph1d_provider_outcome_capture_emits_feedback_and_learn_rows() {
        let runtime = AdapterRuntime::default();
        let mut req = base_request();
        req.turn_id = 20_335;
        req.now_ns = Some(33_005);
        req.user_text_final = Some("hello".to_string());
        runtime
            .run_voice_turn(req.clone())
            .expect("voice turn should succeed");

        let actor_user_id = UserId::new(req.actor_user_id).unwrap();
        let device_id = DeviceId::new(req.device_id.expect("device_id must exist")).unwrap();
        let correlation_id = CorrelationId(req.correlation_id.into());
        let turn_id = TurnId(req.turn_id);
        let provider_correlation_id = correlation_id_to_u64(correlation_id);
        let provider_response = Ph1dProviderCallResponse::v1(
            provider_correlation_id,
            turn_id.0,
            selene_kernel_contracts::ph1d::RequestId(9_501),
            "ph1d_provider_capture_test".to_string(),
            Some("provider_call_capture_01".to_string()),
            "openai_primary".to_string(),
            selene_kernel_contracts::ph1d::Ph1dProviderTask::SttTranscribe,
            "gpt_4o_mini_transcribe".to_string(),
            selene_kernel_contracts::ph1d::Ph1dProviderStatus::Error,
            120,
            0,
            Some(1_800),
            None,
            None,
            selene_kernel_contracts::ph1d::Ph1dProviderValidationStatus::SchemaFail,
            ph1d_reason_codes::D_PROVIDER_SCHEMA_DRIFT,
        )
        .unwrap();

        let mut store = runtime.store.lock().expect("store lock must not poison");
        let before_feedback = store.ph1feedback_audit_rows(correlation_id).len();
        let before_learn = store
            .ph1feedback_learn_signal_bundle_rows(correlation_id)
            .len();
        runtime
            .emit_ph1d_gold_capture_and_learning(
                &mut store,
                MonotonicTimeNs(33_006),
                correlation_id,
                turn_id,
                &actor_user_id,
                Some("tenant_a"),
                Some(&device_id),
                None,
                &[provider_response],
                Some("hello".to_string()),
                Some("en".to_string()),
            )
            .expect("ph1d provider capture emission should succeed");

        let after_feedback = store.ph1feedback_audit_rows(correlation_id).len();
        let after_learn = store
            .ph1feedback_learn_signal_bundle_rows(correlation_id)
            .len();
        assert!(after_feedback > before_feedback);
        assert!(after_learn > before_learn);
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
        assert_eq!(
            second.report_context_id.as_deref(),
            Some(first_context.as_str())
        );
        assert_ne!(second.report_revision, first_revision);
        assert_ne!(second.rows, first_rows);
    }

    #[test]
    fn at_health_12_voice_wave_degraded_marker_is_wired() {
        assert!(app_ui_assets::APP_HTML.contains("voice-wave-state"));
        assert!(app_ui_assets::APP_CSS.contains(".voice-wave.wave-degraded"));
        assert!(app_ui_assets::APP_JS.contains("degraded (transcript sync failed)"));
    }

    #[test]
    fn at_adapter_23_hui01_shell_nav_health_first_with_mobile_hooks() {
        let html = app_ui_assets::APP_HTML;
        let health_idx = html
            .find("data-section=\"health\"")
            .expect("health nav item must exist");
        let inbox_idx = html
            .find("data-section=\"inbox\"")
            .expect("inbox nav item must exist");
        assert!(health_idx < inbox_idx);
        assert!(html.contains("class=\"app-shell\""));
        assert!(html.contains("class=\"sidebar\""));
        assert!(app_ui_assets::APP_CSS.contains("@media (max-width: 900px)"));
        assert!(app_ui_assets::APP_CSS.contains(".app-shell"));
    }

    #[test]
    fn at_adapter_24_hui02_health_landing_uses_check_row_selector() {
        assert!(app_ui_assets::APP_HTML.contains("id=\"checks-list\""));
        assert!(app_ui_assets::APP_JS.contains("selectedSection: \"health\""));
        assert!(app_ui_assets::APP_JS.contains("renderChecks()"));
        let runtime = AdapterRuntime::default();
        let checks = runtime
            .ui_health_checks_report(Some(200))
            .expect("health checks should succeed");
        assert_eq!(checks.status, "ok");
        assert!(!checks.checks.is_empty());
    }

    #[test]
    fn at_adapter_25_hui03_health_cards_show_status_counts_and_last_event() {
        let runtime = AdapterRuntime::default();
        runtime
            .run_device_artifact_sync_worker_pass(Some(777))
            .expect("sync pass should succeed");
        let checks = runtime
            .ui_health_checks_report(Some(777))
            .expect("health checks should succeed");
        let sync = checks
            .checks
            .iter()
            .find(|row| row.check_id == "SYNC")
            .expect("SYNC row must exist");
        assert!(!sync.label.trim().is_empty());
        assert!(!sync.status.trim().is_empty());
        assert!(sync.last_event_at_ns.is_some());
    }

    #[test]
    fn at_adapter_26_hui04_summary_strip_maps_runtime_summary_fields() {
        let detail =
            build_ui_health_detail_response(&synthetic_health_for_detail_tests(), "SYNC", 900)
                .expect("detail build should succeed");
        assert_eq!(detail.summary.open_issues, 3);
        assert_eq!(detail.summary.critical_open_count, 2);
        assert_eq!(detail.summary.escalated_24h_count, 1);
        let html = app_ui_assets::APP_HTML;
        assert!(html.contains("id=\"summary-open\""));
        assert!(html.contains("id=\"summary-critical\""));
        assert!(html.contains("id=\"summary-auto-resolved\""));
        assert!(html.contains("id=\"summary-escalated\""));
        assert!(html.contains("id=\"summary-mttr\""));
    }

    #[test]
    fn at_adapter_27_hui05_primary_queue_table_columns_and_projection_locked() {
        let detail =
            build_ui_health_detail_response(&synthetic_health_for_detail_tests(), "SYNC", 901)
                .expect("detail build should succeed");
        assert!(detail.issues.iter().any(|issue| {
            !issue.severity.is_empty()
                && !issue.issue_type.is_empty()
                && !issue.engine_owner.is_empty()
                && !issue.status.is_empty()
                && !issue.resolution_state.is_empty()
        }));
        let html = app_ui_assets::APP_HTML;
        for header in [
            "Severity",
            "Type",
            "Engine",
            "First Seen",
            "Last Update",
            "Status",
            "Resolution",
        ] {
            assert!(html.contains(header), "missing queue header: {header}");
        }
    }

    #[test]
    fn at_adapter_28_hui06_detail_timeline_shows_reason_evidence_blocker_deadline() {
        let detail =
            build_ui_health_detail_response(&synthetic_health_for_detail_tests(), "SYNC", 902)
                .expect("detail build should succeed");
        assert!(detail.timeline.iter().any(|entry| {
            !entry.reason_code.is_empty()
                && entry.evidence_ref.is_some()
                && entry.blocker.is_some()
                && entry.unresolved_deadline_at_ns.is_some()
        }));
        assert!(detail
            .issues
            .iter()
            .any(|issue| issue.blocker.is_some() && issue.unresolved_deadline_at_ns.is_some()));
        assert!(app_ui_assets::APP_HTML.contains("id=\"detail-meta\""));
        assert!(app_ui_assets::APP_JS.contains("Evidence:"));
        assert!(app_ui_assets::APP_JS.contains("Blocker:"));
        assert!(app_ui_assets::APP_JS.contains("Deadline:"));
    }

    #[test]
    fn at_adapter_29_hui13_chat_shell_transcript_and_wave_layout_present() {
        let html = app_ui_assets::APP_HTML;
        let wave_idx = html
            .find("id=\"voice-wave\"")
            .expect("voice wave must exist");
        let input_idx = html
            .find("id=\"chat-input\"")
            .expect("chat input must exist");
        assert!(wave_idx < input_idx);
        assert!(html.contains("id=\"section-selene\""));
        assert!(html.contains("id=\"transcript-list\""));
        assert!(html.contains("id=\"chat-send-btn\""));
        assert!(app_ui_assets::APP_CSS.contains("#section-selene"));
        assert!(app_ui_assets::APP_CSS.contains("@media (max-width: 900px)"));
    }
}
