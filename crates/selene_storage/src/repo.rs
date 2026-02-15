#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1art::{
    ArtifactLedgerRow, ArtifactLedgerRowInput, ArtifactScopeType, ArtifactStatus, ArtifactType,
    ArtifactVersion, ToolCacheRow, ToolCacheRowInput,
};
use selene_kernel_contracts::ph1c::{
    ConfidenceBucket as Ph1cConfidenceBucket, LanguageTag, RetryAdvice as Ph1cRetryAdvice,
};
use selene_kernel_contracts::ph1capreq::{
    CapabilityRequestCurrentRecord, CapabilityRequestLedgerEvent,
    CapabilityRequestLedgerEventInput, CapreqId,
};
use selene_kernel_contracts::ph1ecm::{
    CapabilityId, EngineCapabilityMapCurrentRecord, EngineCapabilityMapEvent,
    EngineCapabilityMapEventInput, EngineId,
};
use selene_kernel_contracts::ph1f::{
    ConversationSource, ConversationTurnId, ConversationTurnInput, ConversationTurnRecord,
};
use selene_kernel_contracts::ph1j::{
    AuditEvent, AuditEventId, AuditEventInput, CorrelationId, DeviceId, TurnId,
};
use selene_kernel_contracts::ph1l::SessionId;
use selene_kernel_contracts::ph1link::{
    DraftId, DraftStatus, LinkStatus, PrefilledContext, PrefilledContextRef, TokenId,
};
use selene_kernel_contracts::ph1m::{MemoryKey, MemoryLedgerEvent, MemoryUsePolicy};
use selene_kernel_contracts::ph1onb::{
    BackfillCampaignId, BackfillRolloutScope, OnbAccessInstanceCreateResult, OnbCompleteResult,
    OnbEmployeePhotoCaptureSendResult, OnbEmployeeSenderVerifyResult,
    OnbPrimaryDeviceConfirmResult, OnbRequirementBackfillCompleteCommitResult,
    OnbRequirementBackfillNotifyCommitResult, OnbRequirementBackfillStartDraftResult,
    OnbSessionStartResult, OnbTermsAcceptResult, OnboardingSessionId, ProofType,
    SenderVerifyDecision,
};
use selene_kernel_contracts::ph1pbs::{
    BlueprintRegistryRecord, IntentType, ProcessBlueprintEvent, ProcessBlueprintEventInput,
};
use selene_kernel_contracts::ph1position::{
    PositionId, PositionLifecycleState, PositionPolicyResult, PositionRecord,
    PositionRequestedAction, PositionRequirementFieldSpec, PositionRequirementsSchemaDraftResult,
    PositionRequirementsSchemaLifecycleResult, PositionSchemaApplyScope,
    PositionSchemaSelectorSnapshot, PositionScheduleType, PositionValidationStatus, TenantId,
};
use selene_kernel_contracts::ph1simcat::{
    SimulationCatalogCurrentRecord, SimulationCatalogEvent, SimulationCatalogEventInput,
    SimulationId,
};
use selene_kernel_contracts::ph1work::{
    WorkOrderCurrentRecord, WorkOrderId, WorkOrderLedgerEvent, WorkOrderLedgerEventInput,
};
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};

use crate::ph1f::{
    AccessDeviceTrustLevel, AccessGateDecisionRecord, AccessInstanceRecord, AccessLifecycleState,
    AccessMode, AccessOverrideRecord, AccessOverrideType, AccessVerificationLevel, DeviceRecord,
    IdentityRecord, LinkGenerateResultParts, MemoryCurrentRecord, MemoryLedgerRow,
    OnboardingSessionRecord, Ph1cTranscriptOkCommitResult, Ph1cTranscriptRejectCommitResult,
    Ph1fStore, Ph1kDeviceHealth, Ph1kRuntimeCurrentRecord, Ph1kRuntimeEventKind,
    Ph1kRuntimeEventRecord, PositionLifecycleEventRecord, SessionRecord, StorageError,
    TenantCompanyRecord, VoiceEnrollmentSampleRecord, VoiceEnrollmentSessionRecord,
    VoiceProfileRecord, VoiceSampleResult, WakeEnrollmentSampleRecord, WakeEnrollmentSessionRecord,
    WakeRuntimeEventRecord, WakeSampleResult,
};

/// Typed repository interface for PH1.F foundational storage wiring.
pub trait Ph1fFoundationRepo {
    fn insert_identity_row(&mut self, record: IdentityRecord) -> Result<(), StorageError>;
    fn insert_device_row(&mut self, record: DeviceRecord) -> Result<(), StorageError>;
    fn insert_session_row(&mut self, record: SessionRecord) -> Result<(), StorageError>;

    fn append_memory_row(
        &mut self,
        user_id: &UserId,
        event: MemoryLedgerEvent,
        use_policy: MemoryUsePolicy,
        expires_at: Option<MonotonicTimeNs>,
        idempotency_key: Option<String>,
    ) -> Result<u64, StorageError>;

    fn append_conversation_row(
        &mut self,
        input: ConversationTurnInput,
    ) -> Result<ConversationTurnId, StorageError>;

    fn get_identity_row(&self, user_id: &UserId) -> Option<&IdentityRecord>;
    fn memory_ledger_rows(&self) -> &[MemoryLedgerRow];
    fn memory_current_rows(&self) -> &BTreeMap<(UserId, MemoryKey), MemoryCurrentRecord>;
    fn conversation_rows(&self) -> &[ConversationTurnRecord];
    fn rebuild_memory_current_rows(&mut self);
}

/// Typed repository interface for PH1.J append-only audit persistence.
pub trait Ph1jAuditRepo {
    fn append_audit_row(&mut self, input: AuditEventInput) -> Result<AuditEventId, StorageError>;
    fn audit_rows(&self) -> &[AuditEvent];
    fn audit_rows_by_correlation(&self, correlation_id: CorrelationId) -> Vec<&AuditEvent>;
    fn audit_rows_by_tenant(&self, tenant_id: &str) -> Vec<&AuditEvent>;
}

/// Typed repository interface for Selene OS core WorkOrder persistence wiring.
pub trait SeleneOsWorkOrderRepo {
    fn append_work_order_ledger_row(
        &mut self,
        input: WorkOrderLedgerEventInput,
    ) -> Result<u64, StorageError>;
    fn work_order_ledger_rows(&self) -> &[WorkOrderLedgerEvent];
    fn work_orders_current_rows(
        &self,
    ) -> &BTreeMap<(TenantId, WorkOrderId), WorkOrderCurrentRecord>;
    fn work_order_current_row(
        &self,
        tenant_id: &TenantId,
        work_order_id: &WorkOrderId,
    ) -> Option<&WorkOrderCurrentRecord>;
    fn rebuild_work_orders_current_rows(&mut self);
}

/// Typed repository interface for PH1.CAPREQ persistence wiring.
pub trait Ph1CapreqRepo {
    fn append_capreq_row(
        &mut self,
        input: CapabilityRequestLedgerEventInput,
    ) -> Result<u64, StorageError>;
    fn capreq_rows(&self) -> &[CapabilityRequestLedgerEvent];
    fn capreq_current_rows(
        &self,
    ) -> &BTreeMap<(TenantId, CapreqId), CapabilityRequestCurrentRecord>;
    fn capreq_current_row(
        &self,
        tenant_id: &TenantId,
        capreq_id: &CapreqId,
    ) -> Option<&CapabilityRequestCurrentRecord>;
    fn rebuild_capreq_current_rows(&mut self);
}

/// Typed repository interface for PBS tables (`process_blueprints` + `blueprint_registry`).
pub trait PbsTablesRepo {
    fn append_process_blueprint_row(
        &mut self,
        input: ProcessBlueprintEventInput,
    ) -> Result<u64, StorageError>;
    fn process_blueprint_rows(&self) -> &[ProcessBlueprintEvent];
    fn blueprint_registry_rows(&self)
        -> &BTreeMap<(TenantId, IntentType), BlueprintRegistryRecord>;
    fn blueprint_registry_row(
        &self,
        tenant_id: &TenantId,
        intent_type: &IntentType,
    ) -> Option<&BlueprintRegistryRecord>;
    fn rebuild_blueprint_registry_rows(&mut self);
}

/// Typed repository interface for simulation catalog tables (`simulation_catalog` + current projection).
pub trait SimulationCatalogTablesRepo {
    fn append_simulation_catalog_row(
        &mut self,
        input: SimulationCatalogEventInput,
    ) -> Result<u64, StorageError>;
    fn simulation_catalog_rows(&self) -> &[SimulationCatalogEvent];
    fn simulation_catalog_current_rows(
        &self,
    ) -> &BTreeMap<(TenantId, SimulationId), SimulationCatalogCurrentRecord>;
    fn simulation_catalog_current_row(
        &self,
        tenant_id: &TenantId,
        simulation_id: &SimulationId,
    ) -> Option<&SimulationCatalogCurrentRecord>;
    fn rebuild_simulation_catalog_current_rows(&mut self);
}

/// Typed repository interface for engine capability map tables (`engine_capability_maps` + current projection).
pub trait EngineCapabilityMapsTablesRepo {
    fn append_engine_capability_map_row(
        &mut self,
        input: EngineCapabilityMapEventInput,
    ) -> Result<u64, StorageError>;
    fn engine_capability_map_rows(&self) -> &[EngineCapabilityMapEvent];
    fn engine_capability_maps_current_rows(
        &self,
    ) -> &BTreeMap<(TenantId, EngineId, CapabilityId), EngineCapabilityMapCurrentRecord>;
    fn engine_capability_maps_current_row(
        &self,
        tenant_id: &TenantId,
        engine_id: &EngineId,
        capability_id: &CapabilityId,
    ) -> Option<&EngineCapabilityMapCurrentRecord>;
    fn rebuild_engine_capability_maps_current_rows(&mut self);
}

/// Typed repository interface for artifacts ledger + tool cache tables.
pub trait ArtifactsLedgerTablesRepo {
    fn append_artifact_ledger_row(
        &mut self,
        input: ArtifactLedgerRowInput,
    ) -> Result<u64, StorageError>;
    fn artifacts_ledger_rows(&self) -> &[ArtifactLedgerRow];
    fn artifact_ledger_row(
        &self,
        scope_type: ArtifactScopeType,
        scope_id: &str,
        artifact_type: ArtifactType,
        artifact_version: ArtifactVersion,
    ) -> Option<&ArtifactLedgerRow>;
    fn upsert_tool_cache_row(&mut self, input: ToolCacheRowInput) -> Result<u64, StorageError>;
    fn tool_cache_rows(&self) -> &BTreeMap<u64, ToolCacheRow>;
    fn tool_cache_row(
        &self,
        tool_name: &str,
        query_hash: &str,
        locale: &str,
    ) -> Option<&ToolCacheRow>;
}

/// Typed repository interface for PH1.L session lifecycle persistence.
pub trait Ph1lSessionLifecycleRepo {
    fn upsert_session_lifecycle_row(
        &mut self,
        record: SessionRecord,
        idempotency_key: Option<String>,
    ) -> Result<SessionId, StorageError>;
    fn session_row(&self, session_id: &SessionId) -> Option<&SessionRecord>;
    fn session_rows(&self) -> &BTreeMap<SessionId, SessionRecord>;
}

/// Typed repository interface for PH1.VOICE.ID enrollment/profile persistence.
pub trait Ph1VidEnrollmentRepo {
    #[allow(clippy::too_many_arguments)]
    fn ph1vid_enroll_start_draft_row(
        &mut self,
        now: MonotonicTimeNs,
        onboarding_session_id: OnboardingSessionId,
        device_id: DeviceId,
        consent_asserted: bool,
        max_total_attempts: u8,
        max_session_enroll_time_ms: u32,
        lock_after_consecutive_passes: u8,
    ) -> Result<VoiceEnrollmentSessionRecord, StorageError>;

    fn ph1vid_enroll_sample_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        voice_enrollment_session_id: String,
        audio_sample_ref: String,
        attempt_index: u16,
        result: VoiceSampleResult,
        reason_code: Option<ReasonCodeId>,
        idempotency_key: String,
    ) -> Result<VoiceEnrollmentSessionRecord, StorageError>;

    fn ph1vid_enroll_complete_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        voice_enrollment_session_id: String,
        idempotency_key: String,
    ) -> Result<VoiceEnrollmentSessionRecord, StorageError>;

    fn ph1vid_enroll_defer_reminder_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        voice_enrollment_session_id: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<VoiceEnrollmentSessionRecord, StorageError>;

    fn ph1vid_enrollment_session_row(
        &self,
        voice_enrollment_session_id: &str,
    ) -> Option<&VoiceEnrollmentSessionRecord>;

    fn ph1vid_enrollment_sample_rows(
        &self,
        voice_enrollment_session_id: &str,
    ) -> Vec<&VoiceEnrollmentSampleRecord>;

    fn ph1vid_voice_profile_row(&self, voice_profile_id: &str) -> Option<&VoiceProfileRecord>;

    fn attempt_overwrite_voice_enrollment_sample_row(
        &mut self,
        voice_enrollment_session_id: &str,
        sample_seq: u16,
    ) -> Result<(), StorageError>;
}

/// Typed repository interface for PH1.ACCESS.001 + PH2.ACCESS.002 DB wiring.
pub trait Ph1AccessPh2AccessRepo {
    #[allow(clippy::too_many_arguments)]
    fn ph2access_upsert_instance_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        user_id: UserId,
        role_template_id: String,
        effective_access_mode: AccessMode,
        baseline_permissions_json: String,
        identity_verified: bool,
        verification_level: AccessVerificationLevel,
        device_trust_level: AccessDeviceTrustLevel,
        lifecycle_state: AccessLifecycleState,
        policy_snapshot_ref: String,
        idempotency_key: Option<String>,
    ) -> Result<AccessInstanceRecord, StorageError>;

    #[allow(clippy::too_many_arguments)]
    fn ph2access_apply_override_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        access_instance_id: String,
        override_type: AccessOverrideType,
        scope_json: String,
        approved_by_user_id: UserId,
        approved_via_simulation_id: String,
        reason_code: ReasonCodeId,
        starts_at: MonotonicTimeNs,
        expires_at: Option<MonotonicTimeNs>,
        idempotency_key: String,
    ) -> Result<AccessOverrideRecord, StorageError>;

    fn ph2access_instance_row_by_tenant_user(
        &self,
        tenant_id: &str,
        user_id: &UserId,
    ) -> Option<&AccessInstanceRecord>;

    fn ph2access_instance_row_by_id(
        &self,
        access_instance_id: &str,
    ) -> Option<&AccessInstanceRecord>;

    fn ph2access_override_rows_for_instance(
        &self,
        access_instance_id: &str,
    ) -> Vec<&AccessOverrideRecord>;

    fn ph2access_instance_rows(&self) -> &BTreeMap<(String, UserId), AccessInstanceRecord>;
    fn ph2access_override_rows(&self) -> &[AccessOverrideRecord];

    fn ph1access_gate_decide_row(
        &self,
        user_id: UserId,
        access_engine_instance_id: String,
        requested_action: String,
        access_request_context: AccessMode,
        device_trust_level: AccessDeviceTrustLevel,
        sensitive_data_request: bool,
        now: MonotonicTimeNs,
    ) -> Result<AccessGateDecisionRecord, StorageError>;

    fn attempt_overwrite_access_override_row(
        &mut self,
        override_id: &str,
    ) -> Result<(), StorageError>;
}

/// Typed repository interface for PH1.K voice runtime I/O persistence.
pub trait Ph1kVoiceRuntimeRepo {
    #[allow(clippy::too_many_arguments)]
    fn ph1k_runtime_event_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        device_id: DeviceId,
        session_id: Option<SessionId>,
        event_kind: Ph1kRuntimeEventKind,
        processed_stream_id: Option<u128>,
        raw_stream_id: Option<u128>,
        pre_roll_buffer_id: Option<u64>,
        selected_mic: Option<String>,
        selected_speaker: Option<String>,
        device_health: Option<Ph1kDeviceHealth>,
        jitter_ms: Option<f32>,
        drift_ppm: Option<f32>,
        buffer_depth_ms: Option<f32>,
        underruns: Option<u64>,
        overruns: Option<u64>,
        phrase_id: Option<u32>,
        phrase_text: Option<String>,
        reason_code: Option<ReasonCodeId>,
        tts_playback_active: Option<bool>,
        capture_degraded: Option<bool>,
        aec_unstable: Option<bool>,
        device_changed: Option<bool>,
        stream_gap_detected: Option<bool>,
        idempotency_key: String,
    ) -> Result<Ph1kRuntimeEventRecord, StorageError>;

    fn ph1k_runtime_event_rows(&self) -> &[Ph1kRuntimeEventRecord];

    fn ph1k_runtime_current_rows(&self) -> &BTreeMap<(String, DeviceId), Ph1kRuntimeCurrentRecord>;

    fn ph1k_runtime_current_row(
        &self,
        tenant_id: &str,
        device_id: &DeviceId,
    ) -> Option<&Ph1kRuntimeCurrentRecord>;

    fn rebuild_ph1k_runtime_current_rows(&mut self);

    fn attempt_overwrite_ph1k_runtime_event_row(
        &mut self,
        event_id: u64,
    ) -> Result<(), StorageError>;
}

/// Typed repository interface for PH1.W wake enrollment/runtime persistence.
pub trait Ph1wWakeRepo {
    #[allow(clippy::too_many_arguments)]
    fn ph1w_enroll_start_draft_row(
        &mut self,
        now: MonotonicTimeNs,
        user_id: UserId,
        device_id: DeviceId,
        onboarding_session_id: Option<OnboardingSessionId>,
        pass_target: u8,
        max_attempts: u8,
        enrollment_timeout_ms: u32,
        idempotency_key: String,
    ) -> Result<WakeEnrollmentSessionRecord, StorageError>;

    #[allow(clippy::too_many_arguments)]
    fn ph1w_enroll_sample_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        wake_enrollment_session_id: String,
        sample_duration_ms: u16,
        vad_coverage: f32,
        snr_db: f32,
        clipping_pct: f32,
        rms_dbfs: f32,
        noise_floor_dbfs: f32,
        peak_dbfs: f32,
        overlap_ratio: f32,
        result: WakeSampleResult,
        reason_code: Option<ReasonCodeId>,
        idempotency_key: String,
    ) -> Result<WakeEnrollmentSessionRecord, StorageError>;

    fn ph1w_enroll_complete_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        wake_enrollment_session_id: String,
        wake_profile_id: String,
        idempotency_key: String,
    ) -> Result<WakeEnrollmentSessionRecord, StorageError>;

    fn ph1w_enroll_defer_reminder_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        wake_enrollment_session_id: String,
        deferred_until: Option<MonotonicTimeNs>,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<WakeEnrollmentSessionRecord, StorageError>;

    #[allow(clippy::too_many_arguments)]
    fn ph1w_runtime_event_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        wake_event_id: String,
        session_id: Option<SessionId>,
        user_id: Option<UserId>,
        device_id: DeviceId,
        accepted: bool,
        reason_code: ReasonCodeId,
        wake_profile_id: Option<String>,
        tts_active_at_trigger: bool,
        media_playback_active_at_trigger: bool,
        suppression_reason_code: Option<ReasonCodeId>,
        idempotency_key: String,
    ) -> Result<WakeRuntimeEventRecord, StorageError>;

    fn ph1w_enrollment_session_row(
        &self,
        wake_enrollment_session_id: &str,
    ) -> Option<&WakeEnrollmentSessionRecord>;

    fn ph1w_enrollment_sample_rows(
        &self,
        wake_enrollment_session_id: &str,
    ) -> Vec<&WakeEnrollmentSampleRecord>;

    fn ph1w_runtime_event_rows(&self) -> &[WakeRuntimeEventRecord];

    fn ph1w_active_wake_profile(&self, user_id: &UserId, device_id: &DeviceId) -> Option<&str>;

    fn attempt_overwrite_wake_enrollment_sample_row(
        &mut self,
        wake_enrollment_session_id: &str,
        sample_seq: u16,
    ) -> Result<(), StorageError>;

    fn attempt_overwrite_wake_runtime_event_row(
        &mut self,
        wake_event_id: &str,
    ) -> Result<(), StorageError>;
}

/// Typed repository interface for PH1.C transcript gate persistence.
pub trait Ph1cSttRepo {
    #[allow(clippy::too_many_arguments)]
    fn ph1c_transcript_ok_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        transcript_text: String,
        transcript_hash: String,
        language_tag: LanguageTag,
        confidence_bucket: Ph1cConfidenceBucket,
        idempotency_key: String,
    ) -> Result<Ph1cTranscriptOkCommitResult, StorageError>;

    #[allow(clippy::too_many_arguments)]
    fn ph1c_transcript_reject_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        reject_reason_code: ReasonCodeId,
        retry_advice: Ph1cRetryAdvice,
        transcript_hash: Option<String>,
        idempotency_key: String,
    ) -> Result<Ph1cTranscriptRejectCommitResult, StorageError>;

    fn ph1c_voice_transcript_rows(
        &self,
        correlation_id: CorrelationId,
    ) -> Vec<&ConversationTurnRecord>;
}

/// Typed repository interface for PH1.NLP normalization decision persistence.
pub trait Ph1NlpRepo {
    #[allow(clippy::too_many_arguments)]
    fn ph1nlp_intent_draft_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        intent_type: String,
        overall_confidence: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError>;

    #[allow(clippy::too_many_arguments)]
    fn ph1nlp_clarify_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        what_is_missing: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError>;

    #[allow(clippy::too_many_arguments)]
    fn ph1nlp_chat_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError>;

    fn ph1nlp_audit_rows(&self, correlation_id: CorrelationId) -> Vec<&AuditEvent>;
}

/// Typed repository interface for PH1.D LLM router decision persistence.
pub trait Ph1dRouterRepo {
    #[allow(clippy::too_many_arguments)]
    fn ph1d_chat_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError>;

    #[allow(clippy::too_many_arguments)]
    fn ph1d_intent_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        refined_intent_type: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError>;

    #[allow(clippy::too_many_arguments)]
    fn ph1d_clarify_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        what_is_missing: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError>;

    #[allow(clippy::too_many_arguments)]
    fn ph1d_analysis_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        analysis_kind: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError>;

    #[allow(clippy::too_many_arguments)]
    fn ph1d_fail_closed_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        fail_code: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError>;

    fn ph1d_audit_rows(&self, correlation_id: CorrelationId) -> Vec<&AuditEvent>;
}

/// Typed repository interface for PH1.X conversation directive persistence.
pub trait Ph1xConversationRepo {
    #[allow(clippy::too_many_arguments)]
    fn ph1x_confirm_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        confirm_kind: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError>;

    #[allow(clippy::too_many_arguments)]
    fn ph1x_clarify_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        what_is_missing: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError>;

    #[allow(clippy::too_many_arguments)]
    fn ph1x_respond_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        response_kind: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError>;

    #[allow(clippy::too_many_arguments)]
    fn ph1x_dispatch_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        dispatch_target: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError>;

    #[allow(clippy::too_many_arguments)]
    fn ph1x_wait_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        wait_kind: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError>;

    fn ph1x_audit_rows(&self, correlation_id: CorrelationId) -> Vec<&AuditEvent>;
}

/// Typed repository interface for PH1.WRITE formatting decision persistence.
pub trait Ph1WriteRepo {
    #[allow(clippy::too_many_arguments)]
    fn ph1write_format_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        format_mode: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError>;

    fn ph1write_audit_rows(&self, correlation_id: CorrelationId) -> Vec<&AuditEvent>;
}

/// Typed repository interface for PH1.TTS runtime decision persistence.
pub trait Ph1TtsRepo {
    #[allow(clippy::too_many_arguments)]
    fn ph1tts_render_summary_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        route_class_used: String,
        mode_used: String,
        voice_id: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError>;

    #[allow(clippy::too_many_arguments)]
    fn ph1tts_started_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        answer_id: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError>;

    #[allow(clippy::too_many_arguments)]
    fn ph1tts_canceled_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        answer_id: String,
        stop_reason: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError>;

    #[allow(clippy::too_many_arguments)]
    fn ph1tts_failed_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        answer_id: String,
        fail_code: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError>;

    fn ph1tts_audit_rows(&self, correlation_id: CorrelationId) -> Vec<&AuditEvent>;
}

/// Typed repository interface for PH1.E tool router persistence.
pub trait Ph1ERepo {
    #[allow(clippy::too_many_arguments)]
    fn ph1e_tool_ok_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        tool_name: String,
        query_hash: String,
        cache_status: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError>;

    #[allow(clippy::too_many_arguments)]
    fn ph1e_tool_fail_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        tool_name: String,
        fail_code: String,
        cache_status: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError>;

    fn ph1e_audit_rows(&self, correlation_id: CorrelationId) -> Vec<&AuditEvent>;
}

/// Typed repository interface for PH1.PERSONA personalization-profile audit persistence.
pub trait Ph1PersonaRepo {
    #[allow(clippy::too_many_arguments)]
    fn ph1persona_profile_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        style_profile_ref: String,
        delivery_policy_ref: String,
        preferences_snapshot_ref: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError>;

    fn ph1persona_audit_rows(&self, correlation_id: CorrelationId) -> Vec<&AuditEvent>;
}

/// Typed repository interface for PH1.FEEDBACK + PH1.LEARN + PH1.KNOW persistence.
pub trait Ph1LearnFeedbackKnowRepo {
    #[allow(clippy::too_many_arguments)]
    fn ph1feedback_event_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        feedback_event_type: String,
        signal_bucket: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError>;

    #[allow(clippy::too_many_arguments)]
    fn ph1learn_artifact_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        scope_type: ArtifactScopeType,
        scope_id: String,
        artifact_type: ArtifactType,
        artifact_version: ArtifactVersion,
        package_hash: String,
        payload_ref: String,
        provenance_ref: String,
        status: ArtifactStatus,
        idempotency_key: String,
    ) -> Result<u64, StorageError>;

    #[allow(clippy::too_many_arguments)]
    fn ph1know_dictionary_pack_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        artifact_type: ArtifactType,
        artifact_version: ArtifactVersion,
        package_hash: String,
        payload_ref: String,
        provenance_ref: String,
        idempotency_key: String,
    ) -> Result<u64, StorageError>;

    fn ph1feedback_audit_rows(&self, correlation_id: CorrelationId) -> Vec<&AuditEvent>;

    fn ph1learn_artifact_rows(
        &self,
        scope_type: ArtifactScopeType,
        scope_id: &str,
        artifact_type: ArtifactType,
    ) -> Vec<&ArtifactLedgerRow>;

    fn ph1know_artifact_rows(
        &self,
        tenant_id: &str,
        artifact_type: ArtifactType,
    ) -> Vec<&ArtifactLedgerRow>;
}

/// Typed repository interface for PH1.LINK link lifecycle persistence.
pub trait Ph1LinkRepo {
    #[allow(clippy::too_many_arguments)]
    fn ph1link_invite_generate_draft_row(
        &mut self,
        now: MonotonicTimeNs,
        inviter_user_id: UserId,
        invitee_type: selene_kernel_contracts::ph1link::InviteeType,
        tenant_id: Option<String>,
        schema_version_id: Option<String>,
        prefilled_context: Option<PrefilledContext>,
        expiration_policy_id: Option<String>,
    ) -> Result<
        (
            selene_kernel_contracts::ph1link::LinkRecord,
            LinkGenerateResultParts,
        ),
        StorageError,
    >;

    fn ph1link_get_link_row(
        &self,
        token_id: &TokenId,
    ) -> Option<&selene_kernel_contracts::ph1link::LinkRecord>;

    fn ph1link_mark_sent_commit_row(&mut self, token_id: TokenId)
        -> Result<LinkStatus, StorageError>;

    fn ph1link_invite_draft_update_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        draft_id: DraftId,
        creator_update_fields: BTreeMap<String, String>,
        idempotency_key: String,
    ) -> Result<(DraftId, DraftStatus, Vec<String>), StorageError>;

    fn ph1link_invite_open_activate_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        token_id: TokenId,
        device_fingerprint: String,
    ) -> Result<
        (
            LinkStatus,
            DraftId,
            Vec<String>,
            Option<String>,
            Option<String>,
            Option<PrefilledContextRef>,
        ),
        StorageError,
    >;

    fn ph1link_invite_open_activate_commit_row_with_idempotency(
        &mut self,
        now: MonotonicTimeNs,
        token_id: TokenId,
        device_fingerprint: String,
        idempotency_key: String,
    ) -> Result<
        (
            LinkStatus,
            DraftId,
            Vec<String>,
            Option<String>,
            Option<String>,
            Option<PrefilledContextRef>,
        ),
        StorageError,
    >;

    fn ph1link_invite_revoke_revoke_row(
        &mut self,
        token_id: TokenId,
        reason: String,
    ) -> Result<(), StorageError>;

    fn ph1link_invite_expired_recovery_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        expired_token_id: TokenId,
        idempotency_key: String,
    ) -> Result<selene_kernel_contracts::ph1link::LinkRecord, StorageError>;

    fn ph1link_invite_forward_block_commit_row(
        &mut self,
        token_id: TokenId,
        presented_device_fingerprint: String,
    ) -> Result<
        (
            LinkStatus,
            Option<String>,
            DraftId,
            Vec<String>,
            Option<String>,
        ),
        StorageError,
    >;
}

/// Typed repository interface for PH1.ONB onboarding persistence.
pub trait Ph1OnbRepo {
    fn ph1onb_session_start_draft_row(
        &mut self,
        now: MonotonicTimeNs,
        token_id: TokenId,
        prefilled_context_ref: Option<PrefilledContextRef>,
        tenant_id: Option<String>,
        device_fingerprint: String,
    ) -> Result<OnbSessionStartResult, StorageError>;

    fn ph1onb_session_row(
        &self,
        onboarding_session_id: &OnboardingSessionId,
    ) -> Option<&OnboardingSessionRecord>;

    fn ph1onb_session_rows(&self) -> &BTreeMap<OnboardingSessionId, OnboardingSessionRecord>;

    fn ph1onb_terms_accept_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        onboarding_session_id: OnboardingSessionId,
        terms_version_id: String,
        accepted: bool,
        idempotency_key: String,
    ) -> Result<OnbTermsAcceptResult, StorageError>;

    fn ph1onb_employee_photo_capture_send_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        onboarding_session_id: OnboardingSessionId,
        photo_blob_ref: String,
        sender_user_id: UserId,
        idempotency_key: String,
    ) -> Result<OnbEmployeePhotoCaptureSendResult, StorageError>;

    fn ph1onb_employee_sender_verify_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        onboarding_session_id: OnboardingSessionId,
        sender_user_id: UserId,
        decision: SenderVerifyDecision,
        idempotency_key: String,
    ) -> Result<OnbEmployeeSenderVerifyResult, StorageError>;

    fn ph1onb_primary_device_confirm_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        onboarding_session_id: OnboardingSessionId,
        device_id: DeviceId,
        proof_type: ProofType,
        proof_ok: bool,
        idempotency_key: String,
    ) -> Result<OnbPrimaryDeviceConfirmResult, StorageError>;

    fn ph1onb_access_instance_create_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        onboarding_session_id: OnboardingSessionId,
        user_id: UserId,
        tenant_id: Option<String>,
        role_id: String,
        idempotency_key: String,
    ) -> Result<OnbAccessInstanceCreateResult, StorageError>;

    fn ph1onb_complete_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        onboarding_session_id: OnboardingSessionId,
        idempotency_key: String,
    ) -> Result<OnbCompleteResult, StorageError>;

    #[allow(clippy::too_many_arguments)]
    fn ph1onb_requirement_backfill_start_draft_row(
        &mut self,
        now: MonotonicTimeNs,
        actor_user_id: UserId,
        tenant_id: String,
        company_id: String,
        position_id: String,
        schema_version_id: String,
        rollout_scope: BackfillRolloutScope,
        idempotency_key: String,
        simulation_id: &str,
        reason_code: ReasonCodeId,
    ) -> Result<OnbRequirementBackfillStartDraftResult, StorageError>;

    #[allow(clippy::too_many_arguments)]
    fn ph1onb_requirement_backfill_notify_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        campaign_id: BackfillCampaignId,
        tenant_id: String,
        recipient_user_id: UserId,
        idempotency_key: String,
        simulation_id: &str,
        reason_code: ReasonCodeId,
    ) -> Result<OnbRequirementBackfillNotifyCommitResult, StorageError>;

    #[allow(clippy::too_many_arguments)]
    fn ph1onb_requirement_backfill_complete_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        campaign_id: BackfillCampaignId,
        tenant_id: String,
        idempotency_key: String,
        simulation_id: &str,
        reason_code: ReasonCodeId,
    ) -> Result<OnbRequirementBackfillCompleteCommitResult, StorageError>;
}

/// Typed repository interface for PH1.POSITION persistence.
pub trait Ph1PositionRepo {
    fn ph1tenant_company_upsert_row(
        &mut self,
        record: TenantCompanyRecord,
    ) -> Result<(), StorageError>;

    fn ph1tenant_company_row(
        &self,
        tenant_id: &TenantId,
        company_id: &str,
    ) -> Option<&TenantCompanyRecord>;

    #[allow(clippy::too_many_arguments)]
    fn ph1position_create_draft_row(
        &mut self,
        now: MonotonicTimeNs,
        actor_user_id: UserId,
        tenant_id: TenantId,
        company_id: String,
        position_title: String,
        department: String,
        jurisdiction: String,
        schedule_type: PositionScheduleType,
        permission_profile_ref: String,
        compensation_band_ref: String,
        idempotency_key: String,
        simulation_id: &str,
        reason_code: ReasonCodeId,
    ) -> Result<PositionRecord, StorageError>;

    fn ph1position_validate_auth_company_draft_row(
        &self,
        tenant_id: &TenantId,
        company_id: &str,
        position_id: &PositionId,
        requested_action: PositionRequestedAction,
    ) -> Result<(PositionValidationStatus, ReasonCodeId), StorageError>;

    fn ph1position_band_policy_check_draft_row(
        &self,
        tenant_id: &TenantId,
        position_id: &PositionId,
        compensation_band_ref: &str,
    ) -> Result<(PositionPolicyResult, ReasonCodeId), StorageError>;

    fn ph1position_activate_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        actor_user_id: UserId,
        tenant_id: TenantId,
        position_id: PositionId,
        idempotency_key: String,
        simulation_id: &str,
        reason_code: ReasonCodeId,
    ) -> Result<PositionRecord, StorageError>;

    fn ph1position_retire_or_suspend_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        actor_user_id: UserId,
        tenant_id: TenantId,
        position_id: PositionId,
        requested_state: PositionLifecycleState,
        idempotency_key: String,
        simulation_id: &str,
        reason_code: ReasonCodeId,
    ) -> Result<PositionRecord, StorageError>;

    #[allow(clippy::too_many_arguments)]
    fn ph1position_requirements_schema_create_draft_row(
        &mut self,
        now: MonotonicTimeNs,
        actor_user_id: UserId,
        tenant_id: TenantId,
        company_id: String,
        position_id: PositionId,
        schema_version_id: String,
        selectors: PositionSchemaSelectorSnapshot,
        field_specs: Vec<PositionRequirementFieldSpec>,
        idempotency_key: String,
        simulation_id: &str,
        reason_code: ReasonCodeId,
    ) -> Result<PositionRequirementsSchemaDraftResult, StorageError>;

    #[allow(clippy::too_many_arguments)]
    fn ph1position_requirements_schema_update_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        actor_user_id: UserId,
        tenant_id: TenantId,
        company_id: String,
        position_id: PositionId,
        schema_version_id: String,
        selectors: PositionSchemaSelectorSnapshot,
        field_specs: Vec<PositionRequirementFieldSpec>,
        change_reason: String,
        idempotency_key: String,
        simulation_id: &str,
        reason_code: ReasonCodeId,
    ) -> Result<PositionRequirementsSchemaDraftResult, StorageError>;

    #[allow(clippy::too_many_arguments)]
    fn ph1position_requirements_schema_activate_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        actor_user_id: UserId,
        tenant_id: TenantId,
        company_id: String,
        position_id: PositionId,
        schema_version_id: String,
        apply_scope: PositionSchemaApplyScope,
        idempotency_key: String,
        simulation_id: &str,
        reason_code: ReasonCodeId,
    ) -> Result<PositionRequirementsSchemaLifecycleResult, StorageError>;

    fn ph1position_row(
        &self,
        tenant_id: &TenantId,
        position_id: &PositionId,
    ) -> Option<&PositionRecord>;

    fn ph1position_lifecycle_rows_for_position(
        &self,
        tenant_id: &TenantId,
        position_id: &PositionId,
    ) -> Vec<&PositionLifecycleEventRecord>;

    fn attempt_overwrite_position_lifecycle_event_row(
        &mut self,
        event_id: u64,
    ) -> Result<(), StorageError>;
}

/// Typed repository interface for PH1.M memory persistence (`memory_ledger` + `memory_current`).
pub trait Ph1MRepo {
    fn ph1m_append_ledger_row(
        &mut self,
        user_id: &UserId,
        event: MemoryLedgerEvent,
        use_policy: MemoryUsePolicy,
        expires_at: Option<MonotonicTimeNs>,
        idempotency_key: Option<String>,
    ) -> Result<u64, StorageError>;

    fn ph1m_memory_ledger_rows(&self) -> &[MemoryLedgerRow];

    fn ph1m_memory_current_rows(&self) -> &BTreeMap<(UserId, MemoryKey), MemoryCurrentRecord>;

    fn ph1m_memory_current_row(
        &self,
        user_id: &UserId,
        memory_key: &MemoryKey,
    ) -> Option<&MemoryCurrentRecord>;

    fn ph1m_rebuild_current_from_ledger(&mut self);

    fn ph1m_attempt_overwrite_memory_ledger_row(
        &mut self,
        ledger_id: u64,
    ) -> Result<(), StorageError>;
}

impl Ph1fFoundationRepo for Ph1fStore {
    fn insert_identity_row(&mut self, record: IdentityRecord) -> Result<(), StorageError> {
        self.insert_identity(record)
    }

    fn insert_device_row(&mut self, record: DeviceRecord) -> Result<(), StorageError> {
        self.insert_device(record)
    }

    fn insert_session_row(&mut self, record: SessionRecord) -> Result<(), StorageError> {
        self.insert_session(record)
    }

    fn append_memory_row(
        &mut self,
        user_id: &UserId,
        event: MemoryLedgerEvent,
        use_policy: MemoryUsePolicy,
        expires_at: Option<MonotonicTimeNs>,
        idempotency_key: Option<String>,
    ) -> Result<u64, StorageError> {
        self.append_memory_ledger_event(user_id, event, use_policy, expires_at, idempotency_key)
    }

    fn append_conversation_row(
        &mut self,
        input: ConversationTurnInput,
    ) -> Result<ConversationTurnId, StorageError> {
        self.append_conversation_turn(input)
    }

    fn get_identity_row(&self, user_id: &UserId) -> Option<&IdentityRecord> {
        self.get_identity(user_id)
    }

    fn memory_ledger_rows(&self) -> &[MemoryLedgerRow] {
        self.memory_ledger_rows()
    }

    fn memory_current_rows(&self) -> &BTreeMap<(UserId, MemoryKey), MemoryCurrentRecord> {
        self.memory_current()
    }

    fn conversation_rows(&self) -> &[ConversationTurnRecord] {
        self.conversation_ledger()
    }

    fn rebuild_memory_current_rows(&mut self) {
        self.rebuild_memory_current_from_ledger();
    }
}

impl Ph1jAuditRepo for Ph1fStore {
    fn append_audit_row(&mut self, input: AuditEventInput) -> Result<AuditEventId, StorageError> {
        self.append_audit_event(input)
    }

    fn audit_rows(&self) -> &[AuditEvent] {
        self.audit_events()
    }

    fn audit_rows_by_correlation(&self, correlation_id: CorrelationId) -> Vec<&AuditEvent> {
        self.audit_events_by_correlation(correlation_id)
    }

    fn audit_rows_by_tenant(&self, tenant_id: &str) -> Vec<&AuditEvent> {
        self.audit_events_by_tenant(tenant_id)
    }
}

impl SeleneOsWorkOrderRepo for Ph1fStore {
    fn append_work_order_ledger_row(
        &mut self,
        input: WorkOrderLedgerEventInput,
    ) -> Result<u64, StorageError> {
        self.append_work_order_ledger_event(input)
    }

    fn work_order_ledger_rows(&self) -> &[WorkOrderLedgerEvent] {
        self.work_order_ledger()
    }

    fn work_orders_current_rows(
        &self,
    ) -> &BTreeMap<(TenantId, WorkOrderId), WorkOrderCurrentRecord> {
        self.work_orders_current()
    }

    fn work_order_current_row(
        &self,
        tenant_id: &TenantId,
        work_order_id: &WorkOrderId,
    ) -> Option<&WorkOrderCurrentRecord> {
        self.work_order_current(tenant_id, work_order_id)
    }

    fn rebuild_work_orders_current_rows(&mut self) {
        self.rebuild_work_orders_current_from_ledger();
    }
}

impl Ph1CapreqRepo for Ph1fStore {
    fn append_capreq_row(
        &mut self,
        input: CapabilityRequestLedgerEventInput,
    ) -> Result<u64, StorageError> {
        self.append_capreq_event(input)
    }

    fn capreq_rows(&self) -> &[CapabilityRequestLedgerEvent] {
        self.capreq_events()
    }

    fn capreq_current_rows(
        &self,
    ) -> &BTreeMap<(TenantId, CapreqId), CapabilityRequestCurrentRecord> {
        self.capreq_current()
    }

    fn capreq_current_row(
        &self,
        tenant_id: &TenantId,
        capreq_id: &CapreqId,
    ) -> Option<&CapabilityRequestCurrentRecord> {
        self.capreq_current_row(tenant_id, capreq_id)
    }

    fn rebuild_capreq_current_rows(&mut self) {
        self.rebuild_capreq_current_from_ledger();
    }
}

impl PbsTablesRepo for Ph1fStore {
    fn append_process_blueprint_row(
        &mut self,
        input: ProcessBlueprintEventInput,
    ) -> Result<u64, StorageError> {
        self.append_process_blueprint_event(input)
    }

    fn process_blueprint_rows(&self) -> &[ProcessBlueprintEvent] {
        self.process_blueprint_events()
    }

    fn blueprint_registry_rows(
        &self,
    ) -> &BTreeMap<(TenantId, IntentType), BlueprintRegistryRecord> {
        self.blueprint_registry()
    }

    fn blueprint_registry_row(
        &self,
        tenant_id: &TenantId,
        intent_type: &IntentType,
    ) -> Option<&BlueprintRegistryRecord> {
        self.blueprint_registry_row(tenant_id, intent_type)
    }

    fn rebuild_blueprint_registry_rows(&mut self) {
        self.rebuild_blueprint_registry_from_process_blueprint_events();
    }
}

impl SimulationCatalogTablesRepo for Ph1fStore {
    fn append_simulation_catalog_row(
        &mut self,
        input: SimulationCatalogEventInput,
    ) -> Result<u64, StorageError> {
        self.append_simulation_catalog_event(input)
    }

    fn simulation_catalog_rows(&self) -> &[SimulationCatalogEvent] {
        self.simulation_catalog_events()
    }

    fn simulation_catalog_current_rows(
        &self,
    ) -> &BTreeMap<(TenantId, SimulationId), SimulationCatalogCurrentRecord> {
        self.simulation_catalog_current()
    }

    fn simulation_catalog_current_row(
        &self,
        tenant_id: &TenantId,
        simulation_id: &SimulationId,
    ) -> Option<&SimulationCatalogCurrentRecord> {
        self.simulation_catalog_current_row(tenant_id, simulation_id)
    }

    fn rebuild_simulation_catalog_current_rows(&mut self) {
        self.rebuild_simulation_catalog_current_from_ledger();
    }
}

impl EngineCapabilityMapsTablesRepo for Ph1fStore {
    fn append_engine_capability_map_row(
        &mut self,
        input: EngineCapabilityMapEventInput,
    ) -> Result<u64, StorageError> {
        self.append_engine_capability_map_event(input)
    }

    fn engine_capability_map_rows(&self) -> &[EngineCapabilityMapEvent] {
        self.engine_capability_map_events()
    }

    fn engine_capability_maps_current_rows(
        &self,
    ) -> &BTreeMap<(TenantId, EngineId, CapabilityId), EngineCapabilityMapCurrentRecord> {
        self.engine_capability_maps_current()
    }

    fn engine_capability_maps_current_row(
        &self,
        tenant_id: &TenantId,
        engine_id: &EngineId,
        capability_id: &CapabilityId,
    ) -> Option<&EngineCapabilityMapCurrentRecord> {
        self.engine_capability_maps_current_row(tenant_id, engine_id, capability_id)
    }

    fn rebuild_engine_capability_maps_current_rows(&mut self) {
        self.rebuild_engine_capability_maps_current_from_ledger();
    }
}

impl ArtifactsLedgerTablesRepo for Ph1fStore {
    fn append_artifact_ledger_row(
        &mut self,
        input: ArtifactLedgerRowInput,
    ) -> Result<u64, StorageError> {
        self.append_artifact_ledger_row(input)
    }

    fn artifacts_ledger_rows(&self) -> &[ArtifactLedgerRow] {
        self.artifacts_ledger_rows()
    }

    fn artifact_ledger_row(
        &self,
        scope_type: ArtifactScopeType,
        scope_id: &str,
        artifact_type: ArtifactType,
        artifact_version: ArtifactVersion,
    ) -> Option<&ArtifactLedgerRow> {
        self.artifact_ledger_row(scope_type, scope_id, artifact_type, artifact_version)
    }

    fn upsert_tool_cache_row(&mut self, input: ToolCacheRowInput) -> Result<u64, StorageError> {
        self.upsert_tool_cache_row(input)
    }

    fn tool_cache_rows(&self) -> &BTreeMap<u64, ToolCacheRow> {
        self.tool_cache_rows()
    }

    fn tool_cache_row(
        &self,
        tool_name: &str,
        query_hash: &str,
        locale: &str,
    ) -> Option<&ToolCacheRow> {
        self.tool_cache_row(tool_name, query_hash, locale)
    }
}

impl Ph1lSessionLifecycleRepo for Ph1fStore {
    fn upsert_session_lifecycle_row(
        &mut self,
        record: SessionRecord,
        idempotency_key: Option<String>,
    ) -> Result<SessionId, StorageError> {
        self.upsert_session_lifecycle(record, idempotency_key)
    }

    fn session_row(&self, session_id: &SessionId) -> Option<&SessionRecord> {
        self.get_session(session_id)
    }

    fn session_rows(&self) -> &BTreeMap<SessionId, SessionRecord> {
        Ph1fStore::session_rows(self)
    }
}

impl Ph1VidEnrollmentRepo for Ph1fStore {
    fn ph1vid_enroll_start_draft_row(
        &mut self,
        now: MonotonicTimeNs,
        onboarding_session_id: OnboardingSessionId,
        device_id: DeviceId,
        consent_asserted: bool,
        max_total_attempts: u8,
        max_session_enroll_time_ms: u32,
        lock_after_consecutive_passes: u8,
    ) -> Result<VoiceEnrollmentSessionRecord, StorageError> {
        self.ph1vid_enroll_start_draft(
            now,
            onboarding_session_id,
            device_id,
            consent_asserted,
            max_total_attempts,
            max_session_enroll_time_ms,
            lock_after_consecutive_passes,
        )
    }

    fn ph1vid_enroll_sample_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        voice_enrollment_session_id: String,
        audio_sample_ref: String,
        attempt_index: u16,
        result: VoiceSampleResult,
        reason_code: Option<ReasonCodeId>,
        idempotency_key: String,
    ) -> Result<VoiceEnrollmentSessionRecord, StorageError> {
        self.ph1vid_enroll_sample_commit(
            now,
            voice_enrollment_session_id,
            audio_sample_ref,
            attempt_index,
            result,
            reason_code,
            idempotency_key,
        )
    }

    fn ph1vid_enroll_complete_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        voice_enrollment_session_id: String,
        idempotency_key: String,
    ) -> Result<VoiceEnrollmentSessionRecord, StorageError> {
        self.ph1vid_enroll_complete_commit(now, voice_enrollment_session_id, idempotency_key)
    }

    fn ph1vid_enroll_defer_reminder_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        voice_enrollment_session_id: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<VoiceEnrollmentSessionRecord, StorageError> {
        self.ph1vid_enroll_defer_reminder_commit(
            now,
            voice_enrollment_session_id,
            reason_code,
            idempotency_key,
        )
    }

    fn ph1vid_enrollment_session_row(
        &self,
        voice_enrollment_session_id: &str,
    ) -> Option<&VoiceEnrollmentSessionRecord> {
        self.ph1vid_get_enrollment_session(voice_enrollment_session_id)
    }

    fn ph1vid_enrollment_sample_rows(
        &self,
        voice_enrollment_session_id: &str,
    ) -> Vec<&VoiceEnrollmentSampleRecord> {
        self.ph1vid_get_samples_for_session(voice_enrollment_session_id)
    }

    fn ph1vid_voice_profile_row(&self, voice_profile_id: &str) -> Option<&VoiceProfileRecord> {
        self.ph1vid_get_voice_profile(voice_profile_id)
    }

    fn attempt_overwrite_voice_enrollment_sample_row(
        &mut self,
        voice_enrollment_session_id: &str,
        sample_seq: u16,
    ) -> Result<(), StorageError> {
        self.attempt_overwrite_voice_enrollment_sample(voice_enrollment_session_id, sample_seq)
    }
}

impl Ph1AccessPh2AccessRepo for Ph1fStore {
    fn ph2access_upsert_instance_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        user_id: UserId,
        role_template_id: String,
        effective_access_mode: AccessMode,
        baseline_permissions_json: String,
        identity_verified: bool,
        verification_level: AccessVerificationLevel,
        device_trust_level: AccessDeviceTrustLevel,
        lifecycle_state: AccessLifecycleState,
        policy_snapshot_ref: String,
        idempotency_key: Option<String>,
    ) -> Result<AccessInstanceRecord, StorageError> {
        self.ph2access_upsert_instance_commit(
            now,
            tenant_id,
            user_id,
            role_template_id,
            effective_access_mode,
            baseline_permissions_json,
            identity_verified,
            verification_level,
            device_trust_level,
            lifecycle_state,
            policy_snapshot_ref,
            idempotency_key,
        )
    }

    fn ph2access_apply_override_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        access_instance_id: String,
        override_type: AccessOverrideType,
        scope_json: String,
        approved_by_user_id: UserId,
        approved_via_simulation_id: String,
        reason_code: ReasonCodeId,
        starts_at: MonotonicTimeNs,
        expires_at: Option<MonotonicTimeNs>,
        idempotency_key: String,
    ) -> Result<AccessOverrideRecord, StorageError> {
        self.ph2access_apply_override_commit(
            now,
            tenant_id,
            access_instance_id,
            override_type,
            scope_json,
            approved_by_user_id,
            approved_via_simulation_id,
            reason_code,
            starts_at,
            expires_at,
            idempotency_key,
        )
    }

    fn ph2access_instance_row_by_tenant_user(
        &self,
        tenant_id: &str,
        user_id: &UserId,
    ) -> Option<&AccessInstanceRecord> {
        self.ph2access_get_instance_by_tenant_user(tenant_id, user_id)
    }

    fn ph2access_instance_row_by_id(
        &self,
        access_instance_id: &str,
    ) -> Option<&AccessInstanceRecord> {
        self.ph2access_get_instance_by_id(access_instance_id)
    }

    fn ph2access_override_rows_for_instance(
        &self,
        access_instance_id: &str,
    ) -> Vec<&AccessOverrideRecord> {
        self.ph2access_get_overrides_for_instance(access_instance_id)
    }

    fn ph2access_instance_rows(&self) -> &BTreeMap<(String, UserId), AccessInstanceRecord> {
        Ph1fStore::ph2access_instance_rows(self)
    }

    fn ph2access_override_rows(&self) -> &[AccessOverrideRecord] {
        Ph1fStore::ph2access_override_rows(self)
    }

    fn ph1access_gate_decide_row(
        &self,
        user_id: UserId,
        access_engine_instance_id: String,
        requested_action: String,
        access_request_context: AccessMode,
        device_trust_level: AccessDeviceTrustLevel,
        sensitive_data_request: bool,
        now: MonotonicTimeNs,
    ) -> Result<AccessGateDecisionRecord, StorageError> {
        self.ph1access_gate_decide(
            user_id,
            access_engine_instance_id,
            requested_action,
            access_request_context,
            device_trust_level,
            sensitive_data_request,
            now,
        )
    }

    fn attempt_overwrite_access_override_row(
        &mut self,
        override_id: &str,
    ) -> Result<(), StorageError> {
        self.attempt_overwrite_access_override(override_id)
    }
}

impl Ph1kVoiceRuntimeRepo for Ph1fStore {
    fn ph1k_runtime_event_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        device_id: DeviceId,
        session_id: Option<SessionId>,
        event_kind: Ph1kRuntimeEventKind,
        processed_stream_id: Option<u128>,
        raw_stream_id: Option<u128>,
        pre_roll_buffer_id: Option<u64>,
        selected_mic: Option<String>,
        selected_speaker: Option<String>,
        device_health: Option<Ph1kDeviceHealth>,
        jitter_ms: Option<f32>,
        drift_ppm: Option<f32>,
        buffer_depth_ms: Option<f32>,
        underruns: Option<u64>,
        overruns: Option<u64>,
        phrase_id: Option<u32>,
        phrase_text: Option<String>,
        reason_code: Option<ReasonCodeId>,
        tts_playback_active: Option<bool>,
        capture_degraded: Option<bool>,
        aec_unstable: Option<bool>,
        device_changed: Option<bool>,
        stream_gap_detected: Option<bool>,
        idempotency_key: String,
    ) -> Result<Ph1kRuntimeEventRecord, StorageError> {
        self.ph1k_runtime_event_commit(
            now,
            tenant_id,
            device_id,
            session_id,
            event_kind,
            processed_stream_id,
            raw_stream_id,
            pre_roll_buffer_id,
            selected_mic,
            selected_speaker,
            device_health,
            jitter_ms,
            drift_ppm,
            buffer_depth_ms,
            underruns,
            overruns,
            phrase_id,
            phrase_text,
            reason_code,
            tts_playback_active,
            capture_degraded,
            aec_unstable,
            device_changed,
            stream_gap_detected,
            idempotency_key,
        )
    }

    fn ph1k_runtime_event_rows(&self) -> &[Ph1kRuntimeEventRecord] {
        Ph1fStore::ph1k_runtime_event_rows(self)
    }

    fn ph1k_runtime_current_rows(&self) -> &BTreeMap<(String, DeviceId), Ph1kRuntimeCurrentRecord> {
        Ph1fStore::ph1k_runtime_current_rows(self)
    }

    fn ph1k_runtime_current_row(
        &self,
        tenant_id: &str,
        device_id: &DeviceId,
    ) -> Option<&Ph1kRuntimeCurrentRecord> {
        Ph1fStore::ph1k_runtime_current_row(self, tenant_id, device_id)
    }

    fn rebuild_ph1k_runtime_current_rows(&mut self) {
        self.rebuild_ph1k_runtime_current_from_ledger();
    }

    fn attempt_overwrite_ph1k_runtime_event_row(
        &mut self,
        event_id: u64,
    ) -> Result<(), StorageError> {
        self.attempt_overwrite_ph1k_runtime_event(event_id)
    }
}

impl Ph1wWakeRepo for Ph1fStore {
    fn ph1w_enroll_start_draft_row(
        &mut self,
        now: MonotonicTimeNs,
        user_id: UserId,
        device_id: DeviceId,
        onboarding_session_id: Option<OnboardingSessionId>,
        pass_target: u8,
        max_attempts: u8,
        enrollment_timeout_ms: u32,
        idempotency_key: String,
    ) -> Result<WakeEnrollmentSessionRecord, StorageError> {
        self.ph1w_enroll_start_draft(
            now,
            user_id,
            device_id,
            onboarding_session_id,
            pass_target,
            max_attempts,
            enrollment_timeout_ms,
            idempotency_key,
        )
    }

    fn ph1w_enroll_sample_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        wake_enrollment_session_id: String,
        sample_duration_ms: u16,
        vad_coverage: f32,
        snr_db: f32,
        clipping_pct: f32,
        rms_dbfs: f32,
        noise_floor_dbfs: f32,
        peak_dbfs: f32,
        overlap_ratio: f32,
        result: WakeSampleResult,
        reason_code: Option<ReasonCodeId>,
        idempotency_key: String,
    ) -> Result<WakeEnrollmentSessionRecord, StorageError> {
        self.ph1w_enroll_sample_commit(
            now,
            wake_enrollment_session_id,
            sample_duration_ms,
            vad_coverage,
            snr_db,
            clipping_pct,
            rms_dbfs,
            noise_floor_dbfs,
            peak_dbfs,
            overlap_ratio,
            result,
            reason_code,
            idempotency_key,
        )
    }

    fn ph1w_enroll_complete_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        wake_enrollment_session_id: String,
        wake_profile_id: String,
        idempotency_key: String,
    ) -> Result<WakeEnrollmentSessionRecord, StorageError> {
        self.ph1w_enroll_complete_commit(
            now,
            wake_enrollment_session_id,
            wake_profile_id,
            idempotency_key,
        )
    }

    fn ph1w_enroll_defer_reminder_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        wake_enrollment_session_id: String,
        deferred_until: Option<MonotonicTimeNs>,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<WakeEnrollmentSessionRecord, StorageError> {
        self.ph1w_enroll_defer_reminder_commit(
            now,
            wake_enrollment_session_id,
            deferred_until,
            reason_code,
            idempotency_key,
        )
    }

    fn ph1w_runtime_event_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        wake_event_id: String,
        session_id: Option<SessionId>,
        user_id: Option<UserId>,
        device_id: DeviceId,
        accepted: bool,
        reason_code: ReasonCodeId,
        wake_profile_id: Option<String>,
        tts_active_at_trigger: bool,
        media_playback_active_at_trigger: bool,
        suppression_reason_code: Option<ReasonCodeId>,
        idempotency_key: String,
    ) -> Result<WakeRuntimeEventRecord, StorageError> {
        self.ph1w_runtime_event_commit(
            now,
            wake_event_id,
            session_id,
            user_id,
            device_id,
            accepted,
            reason_code,
            wake_profile_id,
            tts_active_at_trigger,
            media_playback_active_at_trigger,
            suppression_reason_code,
            idempotency_key,
        )
    }

    fn ph1w_enrollment_session_row(
        &self,
        wake_enrollment_session_id: &str,
    ) -> Option<&WakeEnrollmentSessionRecord> {
        self.ph1w_get_enrollment_session(wake_enrollment_session_id)
    }

    fn ph1w_enrollment_sample_rows(
        &self,
        wake_enrollment_session_id: &str,
    ) -> Vec<&WakeEnrollmentSampleRecord> {
        self.ph1w_get_samples_for_session(wake_enrollment_session_id)
    }

    fn ph1w_runtime_event_rows(&self) -> &[WakeRuntimeEventRecord] {
        self.ph1w_get_runtime_events()
    }

    fn ph1w_active_wake_profile(&self, user_id: &UserId, device_id: &DeviceId) -> Option<&str> {
        self.ph1w_get_active_wake_profile(user_id, device_id)
    }

    fn attempt_overwrite_wake_enrollment_sample_row(
        &mut self,
        wake_enrollment_session_id: &str,
        sample_seq: u16,
    ) -> Result<(), StorageError> {
        self.attempt_overwrite_wake_enrollment_sample(wake_enrollment_session_id, sample_seq)
    }

    fn attempt_overwrite_wake_runtime_event_row(
        &mut self,
        wake_event_id: &str,
    ) -> Result<(), StorageError> {
        self.attempt_overwrite_wake_runtime_event(wake_event_id)
    }
}

impl Ph1cSttRepo for Ph1fStore {
    fn ph1c_transcript_ok_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        transcript_text: String,
        transcript_hash: String,
        language_tag: LanguageTag,
        confidence_bucket: Ph1cConfidenceBucket,
        idempotency_key: String,
    ) -> Result<Ph1cTranscriptOkCommitResult, StorageError> {
        self.ph1c_transcript_ok_commit(
            now,
            tenant_id,
            correlation_id,
            turn_id,
            session_id,
            user_id,
            device_id,
            transcript_text,
            transcript_hash,
            language_tag,
            confidence_bucket,
            idempotency_key,
        )
    }

    fn ph1c_transcript_reject_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        reject_reason_code: ReasonCodeId,
        retry_advice: Ph1cRetryAdvice,
        transcript_hash: Option<String>,
        idempotency_key: String,
    ) -> Result<Ph1cTranscriptRejectCommitResult, StorageError> {
        self.ph1c_transcript_reject_commit(
            now,
            tenant_id,
            correlation_id,
            turn_id,
            session_id,
            user_id,
            device_id,
            reject_reason_code,
            retry_advice,
            transcript_hash,
            idempotency_key,
        )
    }

    fn ph1c_voice_transcript_rows(
        &self,
        correlation_id: CorrelationId,
    ) -> Vec<&ConversationTurnRecord> {
        self.conversation_ledger()
            .iter()
            .filter(|r| {
                r.correlation_id == correlation_id
                    && r.source == ConversationSource::VoiceTranscript
            })
            .collect()
    }
}

impl Ph1NlpRepo for Ph1fStore {
    fn ph1nlp_intent_draft_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        intent_type: String,
        overall_confidence: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError> {
        self.ph1nlp_intent_draft_commit(
            now,
            tenant_id,
            correlation_id,
            turn_id,
            session_id,
            user_id,
            device_id,
            intent_type,
            overall_confidence,
            reason_code,
            idempotency_key,
        )
    }

    fn ph1nlp_clarify_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        what_is_missing: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError> {
        self.ph1nlp_clarify_commit(
            now,
            tenant_id,
            correlation_id,
            turn_id,
            session_id,
            user_id,
            device_id,
            what_is_missing,
            reason_code,
            idempotency_key,
        )
    }

    fn ph1nlp_chat_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError> {
        self.ph1nlp_chat_commit(
            now,
            tenant_id,
            correlation_id,
            turn_id,
            session_id,
            user_id,
            device_id,
            reason_code,
            idempotency_key,
        )
    }

    fn ph1nlp_audit_rows(&self, correlation_id: CorrelationId) -> Vec<&AuditEvent> {
        Ph1fStore::ph1nlp_audit_rows(self, correlation_id)
    }
}

impl Ph1dRouterRepo for Ph1fStore {
    fn ph1d_chat_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError> {
        self.ph1d_chat_commit(
            now,
            tenant_id,
            correlation_id,
            turn_id,
            session_id,
            user_id,
            device_id,
            reason_code,
            idempotency_key,
        )
    }

    fn ph1d_intent_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        refined_intent_type: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError> {
        self.ph1d_intent_commit(
            now,
            tenant_id,
            correlation_id,
            turn_id,
            session_id,
            user_id,
            device_id,
            refined_intent_type,
            reason_code,
            idempotency_key,
        )
    }

    fn ph1d_clarify_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        what_is_missing: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError> {
        self.ph1d_clarify_commit(
            now,
            tenant_id,
            correlation_id,
            turn_id,
            session_id,
            user_id,
            device_id,
            what_is_missing,
            reason_code,
            idempotency_key,
        )
    }

    fn ph1d_analysis_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        analysis_kind: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError> {
        self.ph1d_analysis_commit(
            now,
            tenant_id,
            correlation_id,
            turn_id,
            session_id,
            user_id,
            device_id,
            analysis_kind,
            reason_code,
            idempotency_key,
        )
    }

    fn ph1d_fail_closed_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        fail_code: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError> {
        self.ph1d_fail_closed_commit(
            now,
            tenant_id,
            correlation_id,
            turn_id,
            session_id,
            user_id,
            device_id,
            fail_code,
            reason_code,
            idempotency_key,
        )
    }

    fn ph1d_audit_rows(&self, correlation_id: CorrelationId) -> Vec<&AuditEvent> {
        Ph1fStore::ph1d_audit_rows(self, correlation_id)
    }
}

impl Ph1xConversationRepo for Ph1fStore {
    fn ph1x_confirm_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        confirm_kind: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError> {
        self.ph1x_confirm_commit(
            now,
            tenant_id,
            correlation_id,
            turn_id,
            session_id,
            user_id,
            device_id,
            confirm_kind,
            reason_code,
            idempotency_key,
        )
    }

    fn ph1x_clarify_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        what_is_missing: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError> {
        self.ph1x_clarify_commit(
            now,
            tenant_id,
            correlation_id,
            turn_id,
            session_id,
            user_id,
            device_id,
            what_is_missing,
            reason_code,
            idempotency_key,
        )
    }

    fn ph1x_respond_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        response_kind: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError> {
        self.ph1x_respond_commit(
            now,
            tenant_id,
            correlation_id,
            turn_id,
            session_id,
            user_id,
            device_id,
            response_kind,
            reason_code,
            idempotency_key,
        )
    }

    fn ph1x_dispatch_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        dispatch_target: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError> {
        self.ph1x_dispatch_commit(
            now,
            tenant_id,
            correlation_id,
            turn_id,
            session_id,
            user_id,
            device_id,
            dispatch_target,
            reason_code,
            idempotency_key,
        )
    }

    fn ph1x_wait_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        wait_kind: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError> {
        self.ph1x_wait_commit(
            now,
            tenant_id,
            correlation_id,
            turn_id,
            session_id,
            user_id,
            device_id,
            wait_kind,
            reason_code,
            idempotency_key,
        )
    }

    fn ph1x_audit_rows(&self, correlation_id: CorrelationId) -> Vec<&AuditEvent> {
        Ph1fStore::ph1x_audit_rows(self, correlation_id)
    }
}

impl Ph1WriteRepo for Ph1fStore {
    fn ph1write_format_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        format_mode: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError> {
        self.ph1write_format_commit(
            now,
            tenant_id,
            correlation_id,
            turn_id,
            session_id,
            user_id,
            device_id,
            format_mode,
            reason_code,
            idempotency_key,
        )
    }

    fn ph1write_audit_rows(&self, correlation_id: CorrelationId) -> Vec<&AuditEvent> {
        Ph1fStore::ph1write_audit_rows(self, correlation_id)
    }
}

impl Ph1TtsRepo for Ph1fStore {
    fn ph1tts_render_summary_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        route_class_used: String,
        mode_used: String,
        voice_id: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError> {
        self.ph1tts_render_summary_commit(
            now,
            tenant_id,
            correlation_id,
            turn_id,
            session_id,
            user_id,
            device_id,
            route_class_used,
            mode_used,
            voice_id,
            reason_code,
            idempotency_key,
        )
    }

    fn ph1tts_started_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        answer_id: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError> {
        self.ph1tts_started_commit(
            now,
            tenant_id,
            correlation_id,
            turn_id,
            session_id,
            user_id,
            device_id,
            answer_id,
            reason_code,
            idempotency_key,
        )
    }

    fn ph1tts_canceled_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        answer_id: String,
        stop_reason: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError> {
        self.ph1tts_canceled_commit(
            now,
            tenant_id,
            correlation_id,
            turn_id,
            session_id,
            user_id,
            device_id,
            answer_id,
            stop_reason,
            reason_code,
            idempotency_key,
        )
    }

    fn ph1tts_failed_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        answer_id: String,
        fail_code: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError> {
        self.ph1tts_failed_commit(
            now,
            tenant_id,
            correlation_id,
            turn_id,
            session_id,
            user_id,
            device_id,
            answer_id,
            fail_code,
            reason_code,
            idempotency_key,
        )
    }

    fn ph1tts_audit_rows(&self, correlation_id: CorrelationId) -> Vec<&AuditEvent> {
        Ph1fStore::ph1tts_audit_rows(self, correlation_id)
    }
}

impl Ph1ERepo for Ph1fStore {
    fn ph1e_tool_ok_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        tool_name: String,
        query_hash: String,
        cache_status: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError> {
        self.ph1e_tool_ok_commit(
            now,
            tenant_id,
            correlation_id,
            turn_id,
            session_id,
            user_id,
            device_id,
            tool_name,
            query_hash,
            cache_status,
            reason_code,
            idempotency_key,
        )
    }

    fn ph1e_tool_fail_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        tool_name: String,
        fail_code: String,
        cache_status: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError> {
        self.ph1e_tool_fail_commit(
            now,
            tenant_id,
            correlation_id,
            turn_id,
            session_id,
            user_id,
            device_id,
            tool_name,
            fail_code,
            cache_status,
            reason_code,
            idempotency_key,
        )
    }

    fn ph1e_audit_rows(&self, correlation_id: CorrelationId) -> Vec<&AuditEvent> {
        Ph1fStore::ph1e_audit_rows(self, correlation_id)
    }
}

impl Ph1PersonaRepo for Ph1fStore {
    fn ph1persona_profile_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        style_profile_ref: String,
        delivery_policy_ref: String,
        preferences_snapshot_ref: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError> {
        self.ph1persona_profile_commit(
            now,
            tenant_id,
            correlation_id,
            turn_id,
            session_id,
            user_id,
            device_id,
            style_profile_ref,
            delivery_policy_ref,
            preferences_snapshot_ref,
            reason_code,
            idempotency_key,
        )
    }

    fn ph1persona_audit_rows(&self, correlation_id: CorrelationId) -> Vec<&AuditEvent> {
        Ph1fStore::ph1persona_audit_rows(self, correlation_id)
    }
}

impl Ph1LearnFeedbackKnowRepo for Ph1fStore {
    fn ph1feedback_event_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        feedback_event_type: String,
        signal_bucket: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<AuditEventId, StorageError> {
        self.ph1feedback_event_commit(
            now,
            tenant_id,
            correlation_id,
            turn_id,
            session_id,
            user_id,
            device_id,
            feedback_event_type,
            signal_bucket,
            reason_code,
            idempotency_key,
        )
    }

    fn ph1learn_artifact_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        scope_type: ArtifactScopeType,
        scope_id: String,
        artifact_type: ArtifactType,
        artifact_version: ArtifactVersion,
        package_hash: String,
        payload_ref: String,
        provenance_ref: String,
        status: ArtifactStatus,
        idempotency_key: String,
    ) -> Result<u64, StorageError> {
        self.ph1learn_artifact_commit(
            now,
            tenant_id,
            scope_type,
            scope_id,
            artifact_type,
            artifact_version,
            package_hash,
            payload_ref,
            provenance_ref,
            status,
            idempotency_key,
        )
    }

    fn ph1know_dictionary_pack_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: String,
        artifact_type: ArtifactType,
        artifact_version: ArtifactVersion,
        package_hash: String,
        payload_ref: String,
        provenance_ref: String,
        idempotency_key: String,
    ) -> Result<u64, StorageError> {
        self.ph1know_dictionary_pack_commit(
            now,
            tenant_id,
            artifact_type,
            artifact_version,
            package_hash,
            payload_ref,
            provenance_ref,
            idempotency_key,
        )
    }

    fn ph1feedback_audit_rows(&self, correlation_id: CorrelationId) -> Vec<&AuditEvent> {
        Ph1fStore::ph1feedback_audit_rows(self, correlation_id)
    }

    fn ph1learn_artifact_rows(
        &self,
        scope_type: ArtifactScopeType,
        scope_id: &str,
        artifact_type: ArtifactType,
    ) -> Vec<&ArtifactLedgerRow> {
        Ph1fStore::ph1learn_artifact_rows(self, scope_type, scope_id, artifact_type)
    }

    fn ph1know_artifact_rows(
        &self,
        tenant_id: &str,
        artifact_type: ArtifactType,
    ) -> Vec<&ArtifactLedgerRow> {
        Ph1fStore::ph1know_artifact_rows(self, tenant_id, artifact_type)
    }
}

impl Ph1LinkRepo for Ph1fStore {
    fn ph1link_invite_generate_draft_row(
        &mut self,
        now: MonotonicTimeNs,
        inviter_user_id: UserId,
        invitee_type: selene_kernel_contracts::ph1link::InviteeType,
        tenant_id: Option<String>,
        schema_version_id: Option<String>,
        prefilled_context: Option<PrefilledContext>,
        expiration_policy_id: Option<String>,
    ) -> Result<
        (
            selene_kernel_contracts::ph1link::LinkRecord,
            LinkGenerateResultParts,
        ),
        StorageError,
    > {
        self.ph1link_invite_generate_draft(
            now,
            inviter_user_id,
            invitee_type,
            tenant_id,
            schema_version_id,
            prefilled_context,
            expiration_policy_id,
        )
    }

    fn ph1link_get_link_row(
        &self,
        token_id: &TokenId,
    ) -> Option<&selene_kernel_contracts::ph1link::LinkRecord> {
        self.ph1link_get_link(token_id)
    }

    fn ph1link_mark_sent_commit_row(
        &mut self,
        token_id: TokenId,
    ) -> Result<LinkStatus, StorageError> {
        self.ph1link_mark_sent_commit(token_id)
    }

    fn ph1link_invite_draft_update_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        draft_id: DraftId,
        creator_update_fields: BTreeMap<String, String>,
        idempotency_key: String,
    ) -> Result<(DraftId, DraftStatus, Vec<String>), StorageError> {
        self.ph1link_invite_draft_update_commit(
            now,
            draft_id,
            creator_update_fields,
            idempotency_key,
        )
    }

    fn ph1link_invite_open_activate_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        token_id: TokenId,
        device_fingerprint: String,
    ) -> Result<
        (
            LinkStatus,
            DraftId,
            Vec<String>,
            Option<String>,
            Option<String>,
            Option<PrefilledContextRef>,
        ),
        StorageError,
    > {
        self.ph1link_invite_open_activate_commit_row_with_idempotency(
            now,
            token_id,
            device_fingerprint,
            "default".to_string(),
        )
    }

    fn ph1link_invite_open_activate_commit_row_with_idempotency(
        &mut self,
        now: MonotonicTimeNs,
        token_id: TokenId,
        device_fingerprint: String,
        idempotency_key: String,
    ) -> Result<
        (
            LinkStatus,
            DraftId,
            Vec<String>,
            Option<String>,
            Option<String>,
            Option<PrefilledContextRef>,
        ),
        StorageError,
    > {
        self.ph1link_invite_open_activate_commit_with_idempotency(
            now,
            token_id,
            device_fingerprint,
            idempotency_key,
        )
    }

    fn ph1link_invite_revoke_revoke_row(
        &mut self,
        token_id: TokenId,
        reason: String,
    ) -> Result<(), StorageError> {
        self.ph1link_invite_revoke_revoke(token_id, reason)
    }

    fn ph1link_invite_expired_recovery_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        expired_token_id: TokenId,
        idempotency_key: String,
    ) -> Result<selene_kernel_contracts::ph1link::LinkRecord, StorageError> {
        self.ph1link_invite_expired_recovery_commit(now, expired_token_id, idempotency_key)
    }

    fn ph1link_invite_forward_block_commit_row(
        &mut self,
        token_id: TokenId,
        presented_device_fingerprint: String,
    ) -> Result<
        (
            LinkStatus,
            Option<String>,
            DraftId,
            Vec<String>,
            Option<String>,
        ),
        StorageError,
    > {
        self.ph1link_invite_forward_block_commit(token_id, presented_device_fingerprint)
    }
}

impl Ph1OnbRepo for Ph1fStore {
    fn ph1onb_session_start_draft_row(
        &mut self,
        now: MonotonicTimeNs,
        token_id: TokenId,
        prefilled_context_ref: Option<PrefilledContextRef>,
        tenant_id: Option<String>,
        device_fingerprint: String,
    ) -> Result<OnbSessionStartResult, StorageError> {
        self.ph1onb_session_start_draft(
            now,
            token_id,
            prefilled_context_ref,
            tenant_id,
            device_fingerprint,
        )
    }

    fn ph1onb_session_row(
        &self,
        onboarding_session_id: &OnboardingSessionId,
    ) -> Option<&OnboardingSessionRecord> {
        Ph1fStore::ph1onb_session_row(self, onboarding_session_id)
    }

    fn ph1onb_session_rows(&self) -> &BTreeMap<OnboardingSessionId, OnboardingSessionRecord> {
        Ph1fStore::ph1onb_session_rows(self)
    }

    fn ph1onb_terms_accept_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        onboarding_session_id: OnboardingSessionId,
        terms_version_id: String,
        accepted: bool,
        idempotency_key: String,
    ) -> Result<OnbTermsAcceptResult, StorageError> {
        self.ph1onb_terms_accept_commit(
            now,
            onboarding_session_id,
            terms_version_id,
            accepted,
            idempotency_key,
        )
    }

    fn ph1onb_employee_photo_capture_send_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        onboarding_session_id: OnboardingSessionId,
        photo_blob_ref: String,
        sender_user_id: UserId,
        idempotency_key: String,
    ) -> Result<OnbEmployeePhotoCaptureSendResult, StorageError> {
        self.ph1onb_employee_photo_capture_send_commit(
            now,
            onboarding_session_id,
            photo_blob_ref,
            sender_user_id,
            idempotency_key,
        )
    }

    fn ph1onb_employee_sender_verify_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        onboarding_session_id: OnboardingSessionId,
        sender_user_id: UserId,
        decision: SenderVerifyDecision,
        idempotency_key: String,
    ) -> Result<OnbEmployeeSenderVerifyResult, StorageError> {
        self.ph1onb_employee_sender_verify_commit(
            now,
            onboarding_session_id,
            sender_user_id,
            decision,
            idempotency_key,
        )
    }

    fn ph1onb_primary_device_confirm_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        onboarding_session_id: OnboardingSessionId,
        device_id: DeviceId,
        proof_type: ProofType,
        proof_ok: bool,
        idempotency_key: String,
    ) -> Result<OnbPrimaryDeviceConfirmResult, StorageError> {
        self.ph1onb_primary_device_confirm_commit(
            now,
            onboarding_session_id,
            device_id,
            proof_type,
            proof_ok,
            idempotency_key,
        )
    }

    fn ph1onb_access_instance_create_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        onboarding_session_id: OnboardingSessionId,
        user_id: UserId,
        tenant_id: Option<String>,
        role_id: String,
        idempotency_key: String,
    ) -> Result<OnbAccessInstanceCreateResult, StorageError> {
        self.ph1onb_access_instance_create_commit(
            now,
            onboarding_session_id,
            user_id,
            tenant_id,
            role_id,
            idempotency_key,
        )
    }

    fn ph1onb_complete_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        onboarding_session_id: OnboardingSessionId,
        idempotency_key: String,
    ) -> Result<OnbCompleteResult, StorageError> {
        self.ph1onb_complete_commit(now, onboarding_session_id, idempotency_key)
    }

    fn ph1onb_requirement_backfill_start_draft_row(
        &mut self,
        now: MonotonicTimeNs,
        actor_user_id: UserId,
        tenant_id: String,
        company_id: String,
        position_id: String,
        schema_version_id: String,
        rollout_scope: BackfillRolloutScope,
        idempotency_key: String,
        simulation_id: &str,
        reason_code: ReasonCodeId,
    ) -> Result<OnbRequirementBackfillStartDraftResult, StorageError> {
        self.ph1onb_requirement_backfill_start_draft(
            now,
            actor_user_id,
            tenant_id,
            company_id,
            position_id,
            schema_version_id,
            rollout_scope,
            idempotency_key,
            simulation_id,
            reason_code,
        )
    }

    fn ph1onb_requirement_backfill_notify_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        campaign_id: BackfillCampaignId,
        tenant_id: String,
        recipient_user_id: UserId,
        idempotency_key: String,
        simulation_id: &str,
        reason_code: ReasonCodeId,
    ) -> Result<OnbRequirementBackfillNotifyCommitResult, StorageError> {
        self.ph1onb_requirement_backfill_notify_commit(
            now,
            campaign_id,
            tenant_id,
            recipient_user_id,
            idempotency_key,
            simulation_id,
            reason_code,
        )
    }

    fn ph1onb_requirement_backfill_complete_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        campaign_id: BackfillCampaignId,
        tenant_id: String,
        idempotency_key: String,
        simulation_id: &str,
        reason_code: ReasonCodeId,
    ) -> Result<OnbRequirementBackfillCompleteCommitResult, StorageError> {
        self.ph1onb_requirement_backfill_complete_commit(
            now,
            campaign_id,
            tenant_id,
            idempotency_key,
            simulation_id,
            reason_code,
        )
    }
}

impl Ph1PositionRepo for Ph1fStore {
    fn ph1tenant_company_upsert_row(
        &mut self,
        record: TenantCompanyRecord,
    ) -> Result<(), StorageError> {
        self.ph1tenant_company_upsert(record)
    }

    fn ph1tenant_company_row(
        &self,
        tenant_id: &TenantId,
        company_id: &str,
    ) -> Option<&TenantCompanyRecord> {
        self.ph1tenant_company_get(tenant_id, company_id)
    }

    #[allow(clippy::too_many_arguments)]
    fn ph1position_create_draft_row(
        &mut self,
        now: MonotonicTimeNs,
        actor_user_id: UserId,
        tenant_id: TenantId,
        company_id: String,
        position_title: String,
        department: String,
        jurisdiction: String,
        schedule_type: PositionScheduleType,
        permission_profile_ref: String,
        compensation_band_ref: String,
        idempotency_key: String,
        simulation_id: &str,
        reason_code: ReasonCodeId,
    ) -> Result<PositionRecord, StorageError> {
        self.ph1position_create_draft(
            now,
            actor_user_id,
            tenant_id,
            company_id,
            position_title,
            department,
            jurisdiction,
            schedule_type,
            permission_profile_ref,
            compensation_band_ref,
            idempotency_key,
            simulation_id,
            reason_code,
        )
    }

    fn ph1position_validate_auth_company_draft_row(
        &self,
        tenant_id: &TenantId,
        company_id: &str,
        position_id: &PositionId,
        requested_action: PositionRequestedAction,
    ) -> Result<(PositionValidationStatus, ReasonCodeId), StorageError> {
        self.ph1position_validate_auth_company_draft(
            tenant_id,
            company_id,
            position_id,
            requested_action,
        )
    }

    fn ph1position_band_policy_check_draft_row(
        &self,
        tenant_id: &TenantId,
        position_id: &PositionId,
        compensation_band_ref: &str,
    ) -> Result<(PositionPolicyResult, ReasonCodeId), StorageError> {
        self.ph1position_band_policy_check_draft(tenant_id, position_id, compensation_band_ref)
    }

    fn ph1position_activate_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        actor_user_id: UserId,
        tenant_id: TenantId,
        position_id: PositionId,
        idempotency_key: String,
        simulation_id: &str,
        reason_code: ReasonCodeId,
    ) -> Result<PositionRecord, StorageError> {
        self.ph1position_activate_commit(
            now,
            actor_user_id,
            tenant_id,
            position_id,
            idempotency_key,
            simulation_id,
            reason_code,
        )
    }

    fn ph1position_retire_or_suspend_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        actor_user_id: UserId,
        tenant_id: TenantId,
        position_id: PositionId,
        requested_state: PositionLifecycleState,
        idempotency_key: String,
        simulation_id: &str,
        reason_code: ReasonCodeId,
    ) -> Result<PositionRecord, StorageError> {
        self.ph1position_retire_or_suspend_commit(
            now,
            actor_user_id,
            tenant_id,
            position_id,
            requested_state,
            idempotency_key,
            simulation_id,
            reason_code,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn ph1position_requirements_schema_create_draft_row(
        &mut self,
        now: MonotonicTimeNs,
        actor_user_id: UserId,
        tenant_id: TenantId,
        company_id: String,
        position_id: PositionId,
        schema_version_id: String,
        selectors: PositionSchemaSelectorSnapshot,
        field_specs: Vec<PositionRequirementFieldSpec>,
        idempotency_key: String,
        simulation_id: &str,
        reason_code: ReasonCodeId,
    ) -> Result<PositionRequirementsSchemaDraftResult, StorageError> {
        self.ph1position_requirements_schema_create_draft(
            now,
            actor_user_id,
            tenant_id,
            company_id,
            position_id,
            schema_version_id,
            selectors,
            field_specs,
            idempotency_key,
            simulation_id,
            reason_code,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn ph1position_requirements_schema_update_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        actor_user_id: UserId,
        tenant_id: TenantId,
        company_id: String,
        position_id: PositionId,
        schema_version_id: String,
        selectors: PositionSchemaSelectorSnapshot,
        field_specs: Vec<PositionRequirementFieldSpec>,
        change_reason: String,
        idempotency_key: String,
        simulation_id: &str,
        reason_code: ReasonCodeId,
    ) -> Result<PositionRequirementsSchemaDraftResult, StorageError> {
        self.ph1position_requirements_schema_update_commit(
            now,
            actor_user_id,
            tenant_id,
            company_id,
            position_id,
            schema_version_id,
            selectors,
            field_specs,
            change_reason,
            idempotency_key,
            simulation_id,
            reason_code,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn ph1position_requirements_schema_activate_commit_row(
        &mut self,
        now: MonotonicTimeNs,
        actor_user_id: UserId,
        tenant_id: TenantId,
        company_id: String,
        position_id: PositionId,
        schema_version_id: String,
        apply_scope: PositionSchemaApplyScope,
        idempotency_key: String,
        simulation_id: &str,
        reason_code: ReasonCodeId,
    ) -> Result<PositionRequirementsSchemaLifecycleResult, StorageError> {
        self.ph1position_requirements_schema_activate_commit(
            now,
            actor_user_id,
            tenant_id,
            company_id,
            position_id,
            schema_version_id,
            apply_scope,
            idempotency_key,
            simulation_id,
            reason_code,
        )
    }

    fn ph1position_row(
        &self,
        tenant_id: &TenantId,
        position_id: &PositionId,
    ) -> Option<&PositionRecord> {
        self.ph1position_get(tenant_id, position_id)
    }

    fn ph1position_lifecycle_rows_for_position(
        &self,
        tenant_id: &TenantId,
        position_id: &PositionId,
    ) -> Vec<&PositionLifecycleEventRecord> {
        self.ph1position_get_lifecycle_events_for_position(tenant_id, position_id)
    }

    fn attempt_overwrite_position_lifecycle_event_row(
        &mut self,
        event_id: u64,
    ) -> Result<(), StorageError> {
        self.attempt_overwrite_position_lifecycle_event(event_id)
    }
}

impl Ph1MRepo for Ph1fStore {
    fn ph1m_append_ledger_row(
        &mut self,
        user_id: &UserId,
        event: MemoryLedgerEvent,
        use_policy: MemoryUsePolicy,
        expires_at: Option<MonotonicTimeNs>,
        idempotency_key: Option<String>,
    ) -> Result<u64, StorageError> {
        self.append_memory_ledger_event(user_id, event, use_policy, expires_at, idempotency_key)
    }

    fn ph1m_memory_ledger_rows(&self) -> &[MemoryLedgerRow] {
        self.memory_ledger_rows()
    }

    fn ph1m_memory_current_rows(&self) -> &BTreeMap<(UserId, MemoryKey), MemoryCurrentRecord> {
        self.memory_current()
    }

    fn ph1m_memory_current_row(
        &self,
        user_id: &UserId,
        memory_key: &MemoryKey,
    ) -> Option<&MemoryCurrentRecord> {
        self.memory_current()
            .get(&(user_id.clone(), memory_key.clone()))
    }

    fn ph1m_rebuild_current_from_ledger(&mut self) {
        self.rebuild_memory_current_from_ledger();
    }

    fn ph1m_attempt_overwrite_memory_ledger_row(
        &mut self,
        ledger_id: u64,
    ) -> Result<(), StorageError> {
        self.attempt_overwrite_memory_ledger_row(ledger_id)
    }
}
