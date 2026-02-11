#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use selene_kernel_contracts::ph1_voice_id::{SpeakerId, UserId};
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
    CapabilityId, CapabilityMapVersion, EngineCapabilityMapCurrentRecord, EngineCapabilityMapEvent,
    EngineCapabilityMapEventInput, EngineId,
};
use selene_kernel_contracts::ph1f::{
    ConversationRole, ConversationSource, ConversationTurnId, ConversationTurnInput,
    ConversationTurnRecord, PrivacyScope,
};
use selene_kernel_contracts::ph1j::{
    AuditEngine, AuditEvent, AuditEventId, AuditEventInput, AuditEventType, AuditPayloadMin,
    AuditSeverity, CorrelationId, DeviceId, PayloadKey, PayloadValue, TurnId,
};
use selene_kernel_contracts::ph1l::SessionId;
use selene_kernel_contracts::ph1link::{
    deterministic_contact_hash_hex, deterministic_device_fingerprint_hash_hex,
    deterministic_payload_hash_hex, DeliveryMethod, DeliveryProofRef, DeliveryStatus, InviteeType,
    LinkId, LinkRecord, LinkStatus, PrefilledContext, PrefilledContextRef,
};
use selene_kernel_contracts::ph1m::{
    MemoryConfidence, MemoryKey, MemoryLayer, MemoryLedgerEvent, MemoryLedgerEventKind,
    MemoryProvenance, MemorySensitivityFlag, MemoryUsePolicy, MemoryValue,
};
use selene_kernel_contracts::ph1onb::{
    OnbAccessInstanceCreateResult, OnbCompleteResult, OnbEmployeePhotoCaptureSendResult,
    OnbEmployeeSenderVerifyResult, OnbPrimaryDeviceConfirmResult, OnbSessionStartResult,
    OnbTermsAcceptResult, OnboardingNextStep, OnboardingSessionId, OnboardingStatus, ProofType,
    SenderVerifyDecision, TermsStatus, VerificationStatus,
};
use selene_kernel_contracts::ph1pbs::{
    BlueprintRegistryRecord, BlueprintStatus, BlueprintVersion, IntentType, ProcessBlueprintEvent,
    ProcessBlueprintEventInput, ProcessId,
};
use selene_kernel_contracts::ph1position::{
    PositionId, PositionLifecycleAction, PositionLifecycleState, PositionPolicyResult,
    PositionRecord, PositionRequestedAction, PositionScheduleType, PositionValidationStatus,
    TenantId,
};
use selene_kernel_contracts::ph1simcat::{
    SimulationCatalogCurrentRecord, SimulationCatalogEvent, SimulationCatalogEventInput,
    SimulationId, SimulationVersion,
};
use selene_kernel_contracts::ph1work::{
    WorkOrderCurrentRecord, WorkOrderId, WorkOrderLedgerEvent, WorkOrderLedgerEventInput,
};
use selene_kernel_contracts::{
    ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, SessionState, Validate,
};

#[derive(Debug, Clone, PartialEq)]
pub enum StorageError {
    ForeignKeyViolation { table: &'static str, key: String },
    DuplicateKey { table: &'static str, key: String },
    AppendOnlyViolation { table: &'static str },
    ContractViolation(ContractViolation),
}

impl From<ContractViolation> for StorageError {
    fn from(v: ContractViolation) -> Self {
        StorageError::ContractViolation(v)
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    // FNV-1a 64-bit (stable across platforms, deterministic).
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut h = OFFSET;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(PRIME);
    }
    h
}

fn hash_hex_64(s: &str) -> String {
    let mut h = fnv1a64(s.as_bytes());
    if h == 0 {
        h = 1;
    }
    format!("{:016x}", h)
}

fn ms_to_ns(ms: u32) -> u64 {
    (ms as u64).saturating_mul(1_000_000)
}

fn is_allowed_session_transition(from: SessionState, to: SessionState) -> bool {
    if from == to {
        return true;
    }
    matches!(
        (from, to),
        (SessionState::Closed, SessionState::Open)
            | (SessionState::Open, SessionState::Active)
            | (SessionState::Open, SessionState::SoftClosed)
            | (SessionState::Open, SessionState::Closed)
            | (SessionState::Open, SessionState::Suspended)
            | (SessionState::Active, SessionState::SoftClosed)
            | (SessionState::Active, SessionState::Closed)
            | (SessionState::Active, SessionState::Suspended)
            | (SessionState::SoftClosed, SessionState::Active)
            | (SessionState::SoftClosed, SessionState::Closed)
            | (SessionState::SoftClosed, SessionState::Suspended)
            | (SessionState::Suspended, SessionState::Active)
            | (SessionState::Suspended, SessionState::Closed)
    )
}

fn access_mode_rank(mode: AccessMode) -> u8 {
    match mode {
        AccessMode::R => 1,
        AccessMode::W => 2,
        AccessMode::A => 3,
        AccessMode::X => 4,
    }
}

fn role_to_default_access_mode(role_template_id: &str) -> AccessMode {
    let role = role_template_id.to_ascii_lowercase();
    if role.contains("admin") {
        AccessMode::X
    } else if role.contains("approve") {
        AccessMode::A
    } else if role.contains("write") || role.contains("editor") {
        AccessMode::W
    } else {
        AccessMode::R
    }
}

fn windows_overlap(
    start_a: MonotonicTimeNs,
    end_a: Option<MonotonicTimeNs>,
    start_b: MonotonicTimeNs,
    end_b: Option<MonotonicTimeNs>,
) -> bool {
    let a_ends_after_b_starts = end_a.map(|e| e.0 > start_b.0).unwrap_or(true);
    let b_ends_after_a_starts = end_b.map(|e| e.0 > start_a.0).unwrap_or(true);
    a_ends_after_b_starts && b_ends_after_a_starts
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentityStatus {
    Active,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityRecord {
    pub schema_version: SchemaVersion,
    pub user_id: UserId,
    pub speaker_id: Option<SpeakerId>,
    pub primary_language: Option<LanguageTag>,
    pub created_at: MonotonicTimeNs,
    pub status: IdentityStatus,
}

impl IdentityRecord {
    pub fn v1(
        user_id: UserId,
        speaker_id: Option<SpeakerId>,
        primary_language: Option<LanguageTag>,
        created_at: MonotonicTimeNs,
        status: IdentityStatus,
    ) -> Self {
        Self {
            schema_version: SchemaVersion(1),
            user_id,
            speaker_id,
            primary_language,
            created_at,
            status,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceRecord {
    pub schema_version: SchemaVersion,
    pub device_id: DeviceId,
    pub user_id: UserId,
    pub device_type: String,
    pub last_seen_at: MonotonicTimeNs,
    pub audio_profile_ref: Option<String>,
}

impl DeviceRecord {
    pub fn v1(
        device_id: DeviceId,
        user_id: UserId,
        device_type: String,
        last_seen_at: MonotonicTimeNs,
        audio_profile_ref: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let d = Self {
            schema_version: SchemaVersion(1),
            device_id,
            user_id,
            device_type,
            last_seen_at,
            audio_profile_ref,
        };
        d.validate()?;
        Ok(d)
    }
}

impl Validate for DeviceRecord {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.device_type.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "device_record.device_type",
                reason: "must not be empty",
            });
        }
        if self.device_type.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "device_record.device_type",
                reason: "must be <= 32 chars",
            });
        }
        if let Some(p) = &self.audio_profile_ref {
            if p.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "device_record.audio_profile_ref",
                    reason: "must not be empty when provided",
                });
            }
            if p.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "device_record.audio_profile_ref",
                    reason: "must be <= 128 chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRecord {
    pub schema_version: SchemaVersion,
    pub session_id: SessionId,
    pub user_id: UserId,
    pub device_id: DeviceId,
    pub session_state: SessionState,
    pub opened_at: MonotonicTimeNs,
    pub last_activity_at: MonotonicTimeNs,
    pub closed_at: Option<MonotonicTimeNs>,
}

impl SessionRecord {
    pub fn v1(
        session_id: SessionId,
        user_id: UserId,
        device_id: DeviceId,
        session_state: SessionState,
        opened_at: MonotonicTimeNs,
        last_activity_at: MonotonicTimeNs,
        closed_at: Option<MonotonicTimeNs>,
    ) -> Result<Self, ContractViolation> {
        let s = Self {
            schema_version: SchemaVersion(1),
            session_id,
            user_id,
            device_id,
            session_state,
            opened_at,
            last_activity_at,
            closed_at,
        };
        s.validate()?;
        Ok(s)
    }
}

impl Validate for SessionRecord {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.last_activity_at.0 < self.opened_at.0 {
            return Err(ContractViolation::InvalidValue {
                field: "session_record.last_activity_at",
                reason: "must be >= opened_at",
            });
        }
        if let Some(c) = self.closed_at {
            if c.0 < self.opened_at.0 {
                return Err(ContractViolation::InvalidValue {
                    field: "session_record.closed_at",
                    reason: "must be >= opened_at",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLedgerRow {
    pub ledger_id: u64,
    pub user_id: UserId,
    pub event: MemoryLedgerEvent,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCurrentRecord {
    pub schema_version: SchemaVersion,
    pub user_id: UserId,
    pub memory_key: MemoryKey,
    pub memory_value: Option<MemoryValue>,
    pub confidence: MemoryConfidence,
    pub sensitivity_flag: MemorySensitivityFlag,
    pub last_seen_at: MonotonicTimeNs,
    pub active: bool,
    pub use_policy: MemoryUsePolicy,
    pub expires_at: Option<MonotonicTimeNs>,
    pub provenance: MemoryProvenance,
}

impl MemoryCurrentRecord {
    #[allow(clippy::too_many_arguments)]
    fn v1(
        user_id: UserId,
        memory_key: MemoryKey,
        memory_value: Option<MemoryValue>,
        confidence: MemoryConfidence,
        sensitivity_flag: MemorySensitivityFlag,
        last_seen_at: MonotonicTimeNs,
        active: bool,
        use_policy: MemoryUsePolicy,
        expires_at: Option<MonotonicTimeNs>,
        provenance: MemoryProvenance,
    ) -> Self {
        Self {
            schema_version: SchemaVersion(1),
            user_id,
            memory_key,
            memory_value,
            confidence,
            sensitivity_flag,
            last_seen_at,
            active,
            use_policy,
            expires_at,
            provenance,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1fStore {
    identities: BTreeMap<UserId, IdentityRecord>,
    devices: BTreeMap<DeviceId, DeviceRecord>,
    sessions: BTreeMap<SessionId, SessionRecord>,
    // PH1.L session lifecycle write idempotency:
    // (session_id, idempotency_key) -> deterministic no-op on retry.
    session_lifecycle_idempotency_index: BTreeSet<(SessionId, String)>,

    memory_ledger: Vec<MemoryLedgerRow>,
    memory_current: BTreeMap<(UserId, MemoryKey), MemoryCurrentRecord>,

    conversation_ledger: Vec<ConversationTurnRecord>,

    // PH1.LINK current-state store (authoritative via simulations; audit remains append-only proof).
    links: BTreeMap<LinkId, LinkRecord>,
    next_link_seq: u64,
    // Idempotency detection for link draft generation:
    // (inviter_user_id, payload_hash, expiration_policy_id) -> link_id
    link_draft_idempotency_index: BTreeMap<(UserId, String, Option<String>), LinkId>,

    // PH1.LINK delivery proof ledger (append-only).
    link_delivery_proofs: BTreeMap<DeliveryProofRef, LinkDeliveryProofRecord>,
    next_link_delivery_proof_seq: u64,
    // Idempotency detection for delivery attempts:
    // (link_id, delivery_method, recipient_contact_hash, idempotency_key) -> delivery_proof_ref
    link_delivery_idempotency_index:
        BTreeMap<(LinkId, DeliveryMethod, String, String), DeliveryProofRef>,

    // Additional PH1.LINK simulations (v1): recovery, forward-block attempts, role proposals,
    // dual-role conflict escalation, delivery failure handling.
    // Idempotency: (expired_link_id, idempotency_key) -> new_link_id
    link_recovery_idempotency_index: BTreeMap<(LinkId, String), LinkId>,
    // Idempotency: record (link_id, presented_device_fingerprint_hash) once.
    link_forward_block_attempts: BTreeSet<(LinkId, String)>,
    // Idempotency: (link_id, idempotency_key) -> delivery_proof_ref
    link_delivery_failure_idempotency_index: BTreeMap<(LinkId, String), DeliveryProofRef>,

    // Role proposal drafts (sandbox).
    link_role_proposals: BTreeMap<String, LinkRoleProposalRecord>,
    // Idempotency: (tenant_id, proposal_text_hash) -> role_proposal_id
    link_role_proposal_idempotency_index: BTreeMap<(Option<String>, String), String>,

    // Dual-role conflict escalation drafts (sandbox).
    link_dual_role_conflict_cases: BTreeMap<String, LinkDualRoleConflictCaseRecord>,
    // Idempotency: (tenant_id, link_id, note_hash) -> escalation_case_id
    link_dual_role_conflict_idempotency_index:
        BTreeMap<(Option<String>, Option<LinkId>, String), String>,

    // ------------------------
    // PH1.ONB (Onboarding) - current-state store + idempotency indexes.
    // ------------------------
    onboarding_sessions: BTreeMap<OnboardingSessionId, OnboardingSessionRecord>,
    // Idempotency: ONB_SESSION_START_DRAFT is idempotent on link_id.
    onboarding_session_by_link: BTreeMap<LinkId, OnboardingSessionId>,
    // Idempotency: (session_id + idempotency_key) per commit simulation.
    onb_terms_idempotency_index: BTreeMap<(OnboardingSessionId, String), TermsStatus>,
    onb_photo_idempotency_index: BTreeMap<(OnboardingSessionId, String), String>,
    onb_sender_verify_idempotency_index:
        BTreeMap<(OnboardingSessionId, String), VerificationStatus>,
    onb_primary_device_idempotency_index: BTreeMap<(OnboardingSessionId, String), bool>,
    // Idempotency: (user_id + role_id + idempotency_key) for access instance create.
    onb_access_instance_idempotency_index: BTreeMap<(UserId, String, String), String>,
    onb_complete_idempotency_index: BTreeMap<(OnboardingSessionId, String), OnboardingStatus>,

    // ------------------------
    // PH1.ACCESS.001 + PH2.ACCESS.002 (Access/Authority).
    // ------------------------
    access_instances: BTreeMap<(String, UserId), AccessInstanceRecord>,
    access_instances_by_id: BTreeMap<String, (String, UserId)>,
    // Idempotency: (tenant_id, user_id, idempotency_key) -> access_instance_id
    access_instance_idempotency_index: BTreeMap<(String, UserId, String), String>,
    // Append-only override lifecycle rows.
    access_overrides: Vec<AccessOverrideRecord>,
    // Idempotency: (tenant_id, access_instance_id, idempotency_key) -> override_id
    access_override_idempotency_index: BTreeMap<(String, String, String), String>,

    // ------------------------
    // PH1.K (Voice Runtime I/O).
    // ------------------------
    ph1k_runtime_events: Vec<Ph1kRuntimeEventRecord>,
    ph1k_runtime_current: BTreeMap<(String, DeviceId), Ph1kRuntimeCurrentRecord>,
    // Idempotency: (tenant_id, device_id, event_kind, idempotency_key) -> event_id
    ph1k_runtime_event_idempotency_index:
        BTreeMap<(String, DeviceId, Ph1kRuntimeEventKind, String), u64>,
    // Deterministic isolation guard: one tenant binding per device in PH1.K runtime rows.
    ph1k_device_tenant_bindings: BTreeMap<DeviceId, String>,
    // Deterministic isolation guard: one tenant binding per device in PH1.C transcript rows.
    ph1c_device_tenant_bindings: BTreeMap<DeviceId, String>,
    // Deterministic isolation guard: one tenant binding per device in PH1.NLP rows.
    ph1nlp_device_tenant_bindings: BTreeMap<DeviceId, String>,
    // Deterministic isolation guard: one tenant binding per device in PH1.D rows.
    ph1d_device_tenant_bindings: BTreeMap<DeviceId, String>,
    // Deterministic isolation guard: one tenant binding per device in PH1.X rows.
    ph1x_device_tenant_bindings: BTreeMap<DeviceId, String>,
    // Deterministic isolation guard: one tenant binding per device in PH1.WRITE rows.
    ph1write_device_tenant_bindings: BTreeMap<DeviceId, String>,
    // Deterministic isolation guard: one tenant binding per device in PH1.TTS rows.
    ph1tts_device_tenant_bindings: BTreeMap<DeviceId, String>,
    // Deterministic isolation guard: one tenant binding per device in PH1.E rows.
    ph1e_device_tenant_bindings: BTreeMap<DeviceId, String>,
    // Deterministic isolation guard: one tenant binding per device in PH1.PERSONA rows.
    ph1persona_device_tenant_bindings: BTreeMap<DeviceId, String>,
    // Deterministic isolation guard: one tenant binding per device in PH1.FEEDBACK rows.
    ph1feedback_device_tenant_bindings: BTreeMap<DeviceId, String>,
    // Deterministic isolation guard: one tenant binding per user-scoped PH1.LEARN artifacts.
    ph1learn_user_tenant_bindings: BTreeMap<UserId, String>,

    // ------------------------
    // PH1.ONB.BIZ + PH1.POSITION (tenant/company + position truth).
    // ------------------------
    tenant_companies: BTreeMap<(TenantId, String), TenantCompanyRecord>,
    positions: BTreeMap<(TenantId, PositionId), PositionRecord>,
    position_lifecycle_events: Vec<PositionLifecycleEventRecord>,
    // Idempotency indexes for position simulations.
    // (tenant_id, company_id, position_title, department, jurisdiction, idempotency_key) -> position_id
    position_create_idempotency_index:
        BTreeMap<(TenantId, String, String, String, String, String), PositionId>,
    // (tenant_id, position_id, idempotency_key) -> lifecycle_state
    position_activate_idempotency_index:
        BTreeMap<(TenantId, PositionId, String), PositionLifecycleState>,
    // (tenant_id, position_id, requested_state, idempotency_key) -> lifecycle_state
    position_retire_suspend_idempotency_index:
        BTreeMap<(TenantId, PositionId, PositionLifecycleState, String), PositionLifecycleState>,

    // ------------------------
    // PH1.W (Wake) - enrollment/session persistence + runtime event ledger.
    // ------------------------
    wake_enrollment_sessions: BTreeMap<String, WakeEnrollmentSessionRecord>,
    wake_enrollment_samples: Vec<WakeEnrollmentSampleRecord>,
    wake_runtime_events: Vec<WakeRuntimeEventRecord>,
    wake_profile_bindings: BTreeMap<(UserId, DeviceId), String>,

    // Idempotency indexes for wake simulations.
    // (user_id, device_id, idempotency_key) -> wake_enrollment_session_id
    wake_start_idempotency_index: BTreeMap<(UserId, DeviceId, String), String>,
    // (wake_enrollment_session_id, idempotency_key) -> sample_seq
    wake_sample_idempotency_index: BTreeMap<(String, String), u16>,
    // (wake_enrollment_session_id, idempotency_key) -> wake_profile_id
    wake_complete_idempotency_index: BTreeMap<(String, String), String>,
    // (wake_enrollment_session_id, idempotency_key) -> wake_enroll_status
    wake_defer_idempotency_index: BTreeMap<(String, String), WakeEnrollStatus>,
    // (device_id, idempotency_key) -> wake_event_id
    wake_runtime_event_idempotency_index: BTreeMap<(DeviceId, String), String>,

    // ------------------------
    // PH1.VOICE.ID (Voice enrollment) - enrollment/session persistence + profile artifacts.
    // ------------------------
    voice_enrollment_sessions: BTreeMap<String, VoiceEnrollmentSessionRecord>,
    voice_enrollment_samples: Vec<VoiceEnrollmentSampleRecord>,
    voice_profiles: BTreeMap<String, VoiceProfileRecord>,
    voice_profile_bindings: BTreeMap<(OnboardingSessionId, DeviceId), String>,

    // Idempotency indexes for voice-id enrollment simulations.
    // (onboarding_session_id, device_id) -> voice_enrollment_session_id
    voice_start_idempotency_index: BTreeMap<(OnboardingSessionId, DeviceId), String>,
    // (voice_enrollment_session_id, attempt_index, idempotency_key) -> sample_seq
    voice_sample_idempotency_index: BTreeMap<(String, u16, String), u16>,
    // (voice_enrollment_session_id, idempotency_key) -> voice_profile_id
    voice_complete_idempotency_index: BTreeMap<(String, String), String>,
    // (voice_enrollment_session_id, idempotency_key) -> voice_enroll_status
    voice_defer_idempotency_index: BTreeMap<(String, String), VoiceEnrollStatus>,

    // ------------------------
    // PBS tables (blueprint_registry + process_blueprints).
    // ------------------------
    process_blueprint_events: Vec<ProcessBlueprintEvent>,
    blueprint_registry: BTreeMap<(TenantId, IntentType), BlueprintRegistryRecord>,
    // Idempotency dedupe: (tenant_id, process_id, blueprint_version, idempotency_key) -> event_id.
    process_blueprint_idempotency_index:
        BTreeMap<(TenantId, ProcessId, BlueprintVersion, String), u64>,

    // ------------------------
    // Simulation Catalog tables (`simulation_catalog` ledger + current projection).
    // ------------------------
    simulation_catalog_events: Vec<SimulationCatalogEvent>,
    simulation_catalog_current: BTreeMap<(TenantId, SimulationId), SimulationCatalogCurrentRecord>,
    // Idempotency dedupe: (tenant_id, simulation_id, simulation_version, idempotency_key) -> event_id.
    simulation_catalog_idempotency_index:
        BTreeMap<(TenantId, SimulationId, SimulationVersion, String), u64>,

    // ------------------------
    // Engine Capability Maps tables (`engine_capability_maps` ledger + current projection).
    // ------------------------
    engine_capability_map_events: Vec<EngineCapabilityMapEvent>,
    engine_capability_maps_current:
        BTreeMap<(TenantId, EngineId, CapabilityId), EngineCapabilityMapCurrentRecord>,
    // Idempotency dedupe: (tenant_id, engine_id, capability_id, capability_map_version, idempotency_key) -> event_id.
    engine_capability_map_idempotency_index: BTreeMap<
        (
            TenantId,
            EngineId,
            CapabilityId,
            CapabilityMapVersion,
            String,
        ),
        u64,
    >,

    // ------------------------
    // Artifacts ledger + tool cache tables.
    // ------------------------
    artifacts_ledger_rows: Vec<ArtifactLedgerRow>,
    // Unique scope binding: (scope_type, scope_id, artifact_type, artifact_version) -> artifact_id.
    artifacts_scope_version_index:
        BTreeMap<(ArtifactScopeType, String, ArtifactType, ArtifactVersion), u64>,
    // Idempotency dedupe: (scope_type, scope_id, artifact_type, artifact_version, idempotency_key) -> artifact_id.
    artifacts_idempotency_index: BTreeMap<
        (
            ArtifactScopeType,
            String,
            ArtifactType,
            ArtifactVersion,
            String,
        ),
        u64,
    >,
    tool_cache_rows: BTreeMap<u64, ToolCacheRow>,
    // Upsert index: (tool_name, query_hash, locale) -> cache_id.
    tool_cache_lookup_index: BTreeMap<(String, String, String), u64>,

    // ------------------------
    // Selene OS core WorkOrder persistence tables.
    // ------------------------
    work_order_ledger: Vec<WorkOrderLedgerEvent>,
    work_orders_current: BTreeMap<(TenantId, WorkOrderId), WorkOrderCurrentRecord>,
    // Idempotency dedupe for ledger writes: (tenant_id, work_order_id, idempotency_key).
    work_order_ledger_idempotency_index: BTreeMap<(TenantId, WorkOrderId, String), u64>,

    // ------------------------
    // PH1.CAPREQ tables (`capreq_ledger` ledger + `capreq_current` projection).
    // ------------------------
    capreq_ledger_events: Vec<CapabilityRequestLedgerEvent>,
    capreq_current: BTreeMap<(TenantId, CapreqId), CapabilityRequestCurrentRecord>,
    // Idempotency dedupe: (tenant_id, capreq_id, idempotency_key) -> event_id.
    capreq_idempotency_index: BTreeMap<(TenantId, CapreqId, String), u64>,

    audit_events: Vec<AuditEvent>,
    next_memory_ledger_id: u64,
    next_conversation_turn_id: u64,
    next_audit_event_id: u64,
    next_position_lifecycle_event_id: u64,
    next_voice_enrollment_sample_seq: u16,
    next_process_blueprint_event_id: u64,
    next_simulation_catalog_event_id: u64,
    next_engine_capability_map_event_id: u64,
    next_artifact_id: u64,
    next_tool_cache_id: u64,
    next_work_order_event_id: u64,
    next_capreq_event_id: u64,
    next_ph1k_runtime_event_id: u64,

    // Idempotency detection for memory ledger writes: (user_id, key) -> ledger_id.
    memory_idempotency_index: BTreeMap<(UserId, String), u64>,

    // Idempotency detection for conversation writes: (correlation_id, key) -> conversation_turn_id.
    conversation_idempotency_index: BTreeMap<(CorrelationId, String), ConversationTurnId>,

    // Idempotency detection for audit emissions (canonical scope):
    // (tenant_id, work_order_id, idempotency_key) -> event_id.
    audit_idempotency_index_scoped: BTreeMap<(String, String, String), AuditEventId>,
    // Backward-compatible fallback for events that do not carry tenant/work-order scope:
    // (correlation_id, idempotency_key) -> event_id.
    audit_idempotency_index_legacy: BTreeMap<(CorrelationId, String), AuditEventId>,

    // Prevent "silent deletes": track which memory keys have been forgotten (tombstones).
    forgotten_memory: BTreeSet<(UserId, MemoryKey)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkDeliveryProofRecord {
    pub schema_version: SchemaVersion,
    pub delivery_proof_ref: DeliveryProofRef,
    pub created_at: MonotonicTimeNs,
    pub link_id: LinkId,
    pub delivery_method: DeliveryMethod,
    pub recipient_contact_hash: String,
    pub delivery_status: DeliveryStatus,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkRoleProposalRecord {
    pub schema_version: SchemaVersion,
    pub role_proposal_id: String,
    pub created_at: MonotonicTimeNs,
    pub tenant_id: Option<String>,
    pub proposal_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkDualRoleConflictCaseRecord {
    pub schema_version: SchemaVersion,
    pub escalation_case_id: String,
    pub created_at: MonotonicTimeNs,
    pub tenant_id: Option<String>,
    pub link_id: Option<LinkId>,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnboardingSessionRecord {
    pub schema_version: SchemaVersion,
    pub onboarding_session_id: OnboardingSessionId,
    pub link_id: LinkId,
    pub invitee_type: InviteeType,
    pub tenant_id: Option<String>,
    pub prefilled_context_ref: Option<PrefilledContextRef>,
    pub device_fingerprint_hash: String,
    pub status: OnboardingStatus,
    pub created_at: MonotonicTimeNs,
    pub updated_at: MonotonicTimeNs,
    pub terms_version_id: Option<String>,
    pub terms_status: Option<TermsStatus>,
    pub photo_blob_ref: Option<String>,
    pub photo_proof_ref: Option<String>,
    pub sender_user_id: Option<UserId>,
    pub verification_status: Option<VerificationStatus>,
    pub primary_device_device_id: Option<DeviceId>,
    pub primary_device_proof_type: Option<ProofType>,
    pub primary_device_confirmed: bool,
    pub access_engine_instance_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    R,
    W,
    A,
    X,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessVerificationLevel {
    None,
    PasscodeTime,
    Biometric,
    StepUp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AccessDeviceTrustLevel {
    Dtl1,
    Dtl2,
    Dtl3,
    Dtl4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessLifecycleState {
    Restricted,
    Active,
    Suspended,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessInstanceRecord {
    pub schema_version: SchemaVersion,
    pub access_instance_id: String,
    pub tenant_id: String,
    pub user_id: UserId,
    pub role_template_id: String,
    pub effective_access_mode: AccessMode,
    pub baseline_permissions_json: String,
    pub identity_verified: bool,
    pub verification_level: AccessVerificationLevel,
    pub device_trust_level: AccessDeviceTrustLevel,
    pub lifecycle_state: AccessLifecycleState,
    pub policy_snapshot_ref: String,
    pub created_at: MonotonicTimeNs,
    pub updated_at: MonotonicTimeNs,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessOverrideType {
    OneShot,
    Temporary,
    Permanent,
    Revoke,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessOverrideStatus {
    Active,
    Expired,
    Revoked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessOverrideRecord {
    pub schema_version: SchemaVersion,
    pub override_id: String,
    pub access_instance_id: String,
    pub tenant_id: String,
    pub override_type: AccessOverrideType,
    pub scope_json: String,
    pub status: AccessOverrideStatus,
    pub approved_by_user_id: UserId,
    pub approved_via_simulation_id: String,
    pub reason_code: ReasonCodeId,
    pub starts_at: MonotonicTimeNs,
    pub expires_at: Option<MonotonicTimeNs>,
    pub created_at: MonotonicTimeNs,
    pub updated_at: MonotonicTimeNs,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessDecision {
    Allow,
    Deny,
    Escalate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessEscalationTrigger {
    StepUpProofRequired,
    ApApprovalRequired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessGateDecisionRecord {
    pub schema_version: SchemaVersion,
    pub access_decision: AccessDecision,
    pub effective_access_mode: AccessMode,
    pub restriction_flags: Vec<String>,
    pub escalation_trigger: Option<AccessEscalationTrigger>,
    pub reason_code: ReasonCodeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1cTranscriptOkCommitResult {
    pub conversation_turn_id: ConversationTurnId,
    pub transcript_audit_event_id: AuditEventId,
    pub candidate_eval_audit_event_id: AuditEventId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1cTranscriptRejectCommitResult {
    pub transcript_reject_audit_event_id: AuditEventId,
    pub candidate_eval_audit_event_id: AuditEventId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ph1kRuntimeEventKind {
    StreamRefs,
    VadEvent,
    DeviceState,
    TimingStats,
    InterruptCandidate,
    DegradationFlags,
    TtsPlaybackActive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ph1kDeviceHealth {
    Healthy,
    Degraded,
    Failed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1kRuntimeEventRecord {
    pub schema_version: SchemaVersion,
    pub event_id: u64,
    pub tenant_id: String,
    pub device_id: DeviceId,
    pub session_id: Option<SessionId>,
    pub event_kind: Ph1kRuntimeEventKind,
    pub processed_stream_id: Option<u128>,
    pub raw_stream_id: Option<u128>,
    pub pre_roll_buffer_id: Option<u64>,
    pub selected_mic: Option<String>,
    pub selected_speaker: Option<String>,
    pub device_health: Option<Ph1kDeviceHealth>,
    pub jitter_ms: Option<f32>,
    pub drift_ppm: Option<f32>,
    pub buffer_depth_ms: Option<f32>,
    pub underruns: Option<u64>,
    pub overruns: Option<u64>,
    pub phrase_id: Option<u32>,
    pub phrase_text: Option<String>,
    pub reason_code: Option<ReasonCodeId>,
    pub tts_playback_active: Option<bool>,
    pub capture_degraded: Option<bool>,
    pub aec_unstable: Option<bool>,
    pub device_changed: Option<bool>,
    pub stream_gap_detected: Option<bool>,
    pub idempotency_key: String,
    pub created_at: MonotonicTimeNs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1kRuntimeCurrentRecord {
    pub schema_version: SchemaVersion,
    pub tenant_id: String,
    pub device_id: DeviceId,
    pub session_id: Option<SessionId>,
    pub processed_stream_id: Option<u128>,
    pub raw_stream_id: Option<u128>,
    pub pre_roll_buffer_id: Option<u64>,
    pub selected_mic: Option<String>,
    pub selected_speaker: Option<String>,
    pub device_health: Option<Ph1kDeviceHealth>,
    pub jitter_ms: Option<i64>,
    pub drift_ppm: Option<i64>,
    pub buffer_depth_ms: Option<i64>,
    pub underruns: Option<u64>,
    pub overruns: Option<u64>,
    pub tts_playback_active: bool,
    pub capture_degraded: bool,
    pub aec_unstable: bool,
    pub device_changed: bool,
    pub stream_gap_detected: bool,
    pub last_interrupt_phrase: Option<String>,
    pub last_interrupt_reason_code: Option<ReasonCodeId>,
    pub last_event_id: u64,
    pub updated_at: MonotonicTimeNs,
}

// PH1.ACCESS.001 + PH2.ACCESS.002 deterministic reason-code constants.
const ACCESS_REASON_ALLOWED: ReasonCodeId = ReasonCodeId(0x4143_0001);
const ACCESS_REASON_DENIED: ReasonCodeId = ReasonCodeId(0x4143_0002);
const ACCESS_REASON_ESCALATE_REQUIRED: ReasonCodeId = ReasonCodeId(0x4143_0003);
const ACCESS_REASON_INSTANCE_MISSING: ReasonCodeId = ReasonCodeId(0x4143_0004);
const ACCESS_REASON_SCOPE_MISMATCH: ReasonCodeId = ReasonCodeId(0x4143_0005);
const ACCESS_REASON_AP_REQUIRED: ReasonCodeId = ReasonCodeId(0x4143_0006);
const ACCESS_REASON_SENSITIVE_DENY: ReasonCodeId = ReasonCodeId(0x4143_0008);
const ACCESS_REASON_DEVICE_UNTRUSTED: ReasonCodeId = ReasonCodeId(0x4143_0009);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TenantCompanyLifecycleState {
    Draft,
    Active,
    Suspended,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantCompanyRecord {
    pub schema_version: SchemaVersion,
    pub tenant_id: TenantId,
    pub company_id: String,
    pub legal_name: String,
    pub jurisdiction: String,
    pub lifecycle_state: TenantCompanyLifecycleState,
    pub created_at: MonotonicTimeNs,
    pub updated_at: MonotonicTimeNs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionLifecycleEventRecord {
    pub schema_version: SchemaVersion,
    pub event_id: u64,
    pub tenant_id: TenantId,
    pub position_id: PositionId,
    pub action: PositionLifecycleAction,
    pub from_state: PositionLifecycleState,
    pub to_state: PositionLifecycleState,
    pub reason_code: ReasonCodeId,
    pub simulation_id: String,
    pub actor_user_id: UserId,
    pub created_at: MonotonicTimeNs,
    pub idempotency_key: Option<String>,
}

// PH1.W (Wake) deterministic persistence records.
const W_ENROLL_REASON_MAX_ATTEMPTS: ReasonCodeId = ReasonCodeId(0x5700_0201);
const W_ENROLL_REASON_TIMEOUT: ReasonCodeId = ReasonCodeId(0x5700_0202);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WakeEnrollStatus {
    InProgress,
    Pending,
    Complete,
    Declined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WakeSampleResult {
    Pass,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeEnrollmentSessionRecord {
    pub schema_version: SchemaVersion,
    pub wake_enrollment_session_id: String,
    pub user_id: UserId,
    pub device_id: DeviceId,
    pub onboarding_session_id: Option<OnboardingSessionId>,
    pub wake_profile_id: Option<String>,
    pub wake_enroll_status: WakeEnrollStatus,
    pub pass_target: u8,
    pub pass_count: u8,
    pub attempt_count: u8,
    pub max_attempts: u8,
    pub enrollment_timeout_ms: u32,
    pub reason_code: Option<ReasonCodeId>,
    pub created_at: MonotonicTimeNs,
    pub updated_at: MonotonicTimeNs,
    pub completed_at: Option<MonotonicTimeNs>,
    pub deferred_until: Option<MonotonicTimeNs>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WakeEnrollmentSampleRecord {
    pub schema_version: SchemaVersion,
    pub wake_enrollment_session_id: String,
    pub sample_seq: u16,
    pub captured_at: MonotonicTimeNs,
    pub sample_duration_ms: u16,
    pub vad_coverage: f32,
    pub snr_db: f32,
    pub clipping_pct: f32,
    pub rms_dbfs: f32,
    pub noise_floor_dbfs: f32,
    pub peak_dbfs: f32,
    pub overlap_ratio: f32,
    pub result: WakeSampleResult,
    pub reason_code: Option<ReasonCodeId>,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeRuntimeEventRecord {
    pub schema_version: SchemaVersion,
    pub wake_event_id: String,
    pub created_at: MonotonicTimeNs,
    pub session_id: Option<SessionId>,
    pub user_id: Option<UserId>,
    pub device_id: DeviceId,
    pub accepted: bool,
    pub reason_code: ReasonCodeId,
    pub wake_profile_id: Option<String>,
    pub tts_active_at_trigger: bool,
    pub media_playback_active_at_trigger: bool,
    pub suppression_reason_code: Option<ReasonCodeId>,
    pub idempotency_key: String,
}

// PH1.VOICE.ID (voice enrollment) deterministic persistence records.
const VID_ENROLL_REASON_MAX_ATTEMPTS: ReasonCodeId = ReasonCodeId(0x5649_0201);
const VID_ENROLL_REASON_TIMEOUT: ReasonCodeId = ReasonCodeId(0x5649_0202);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceEnrollStatus {
    InProgress,
    Locked,
    Pending,
    Declined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceSampleResult {
    Pass,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceEnrollmentSessionRecord {
    pub schema_version: SchemaVersion,
    pub voice_enrollment_session_id: String,
    pub onboarding_session_id: OnboardingSessionId,
    pub device_id: DeviceId,
    pub voice_profile_id: Option<String>,
    pub voice_enroll_status: VoiceEnrollStatus,
    pub lock_after_consecutive_passes: u8,
    pub consecutive_passes: u8,
    pub attempt_count: u8,
    pub max_total_attempts: u8,
    pub max_session_enroll_time_ms: u32,
    pub created_at: MonotonicTimeNs,
    pub updated_at: MonotonicTimeNs,
    pub deferred_until: Option<MonotonicTimeNs>,
    pub reason_code: Option<ReasonCodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceEnrollmentSampleRecord {
    pub schema_version: SchemaVersion,
    pub sample_seq: u16,
    pub voice_enrollment_session_id: String,
    pub created_at: MonotonicTimeNs,
    pub attempt_index: u16,
    pub audio_sample_ref: String,
    pub result: VoiceSampleResult,
    pub reason_code: Option<ReasonCodeId>,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceProfileRecord {
    pub schema_version: SchemaVersion,
    pub voice_profile_id: String,
    pub onboarding_session_id: OnboardingSessionId,
    pub device_id: DeviceId,
    pub created_at: MonotonicTimeNs,
}

impl Ph1fStore {
    pub fn new_in_memory() -> Self {
        Self {
            identities: BTreeMap::new(),
            devices: BTreeMap::new(),
            sessions: BTreeMap::new(),
            session_lifecycle_idempotency_index: BTreeSet::new(),
            memory_ledger: Vec::new(),
            memory_current: BTreeMap::new(),
            conversation_ledger: Vec::new(),
            links: BTreeMap::new(),
            next_link_seq: 1,
            link_draft_idempotency_index: BTreeMap::new(),
            link_delivery_proofs: BTreeMap::new(),
            next_link_delivery_proof_seq: 1,
            link_delivery_idempotency_index: BTreeMap::new(),
            link_recovery_idempotency_index: BTreeMap::new(),
            link_forward_block_attempts: BTreeSet::new(),
            link_delivery_failure_idempotency_index: BTreeMap::new(),
            link_role_proposals: BTreeMap::new(),
            link_role_proposal_idempotency_index: BTreeMap::new(),
            link_dual_role_conflict_cases: BTreeMap::new(),
            link_dual_role_conflict_idempotency_index: BTreeMap::new(),
            onboarding_sessions: BTreeMap::new(),
            onboarding_session_by_link: BTreeMap::new(),
            onb_terms_idempotency_index: BTreeMap::new(),
            onb_photo_idempotency_index: BTreeMap::new(),
            onb_sender_verify_idempotency_index: BTreeMap::new(),
            onb_primary_device_idempotency_index: BTreeMap::new(),
            onb_access_instance_idempotency_index: BTreeMap::new(),
            onb_complete_idempotency_index: BTreeMap::new(),
            access_instances: BTreeMap::new(),
            access_instances_by_id: BTreeMap::new(),
            access_instance_idempotency_index: BTreeMap::new(),
            access_overrides: Vec::new(),
            access_override_idempotency_index: BTreeMap::new(),
            ph1k_runtime_events: Vec::new(),
            ph1k_runtime_current: BTreeMap::new(),
            ph1k_runtime_event_idempotency_index: BTreeMap::new(),
            ph1k_device_tenant_bindings: BTreeMap::new(),
            ph1c_device_tenant_bindings: BTreeMap::new(),
            ph1nlp_device_tenant_bindings: BTreeMap::new(),
            ph1d_device_tenant_bindings: BTreeMap::new(),
            ph1x_device_tenant_bindings: BTreeMap::new(),
            ph1write_device_tenant_bindings: BTreeMap::new(),
            ph1tts_device_tenant_bindings: BTreeMap::new(),
            ph1e_device_tenant_bindings: BTreeMap::new(),
            ph1persona_device_tenant_bindings: BTreeMap::new(),
            ph1feedback_device_tenant_bindings: BTreeMap::new(),
            ph1learn_user_tenant_bindings: BTreeMap::new(),
            tenant_companies: BTreeMap::new(),
            positions: BTreeMap::new(),
            position_lifecycle_events: Vec::new(),
            position_create_idempotency_index: BTreeMap::new(),
            position_activate_idempotency_index: BTreeMap::new(),
            position_retire_suspend_idempotency_index: BTreeMap::new(),
            wake_enrollment_sessions: BTreeMap::new(),
            wake_enrollment_samples: Vec::new(),
            wake_runtime_events: Vec::new(),
            wake_profile_bindings: BTreeMap::new(),
            wake_start_idempotency_index: BTreeMap::new(),
            wake_sample_idempotency_index: BTreeMap::new(),
            wake_complete_idempotency_index: BTreeMap::new(),
            wake_defer_idempotency_index: BTreeMap::new(),
            wake_runtime_event_idempotency_index: BTreeMap::new(),
            voice_enrollment_sessions: BTreeMap::new(),
            voice_enrollment_samples: Vec::new(),
            voice_profiles: BTreeMap::new(),
            voice_profile_bindings: BTreeMap::new(),
            voice_start_idempotency_index: BTreeMap::new(),
            voice_sample_idempotency_index: BTreeMap::new(),
            voice_complete_idempotency_index: BTreeMap::new(),
            voice_defer_idempotency_index: BTreeMap::new(),
            process_blueprint_events: Vec::new(),
            blueprint_registry: BTreeMap::new(),
            process_blueprint_idempotency_index: BTreeMap::new(),
            simulation_catalog_events: Vec::new(),
            simulation_catalog_current: BTreeMap::new(),
            simulation_catalog_idempotency_index: BTreeMap::new(),
            engine_capability_map_events: Vec::new(),
            engine_capability_maps_current: BTreeMap::new(),
            engine_capability_map_idempotency_index: BTreeMap::new(),
            artifacts_ledger_rows: Vec::new(),
            artifacts_scope_version_index: BTreeMap::new(),
            artifacts_idempotency_index: BTreeMap::new(),
            tool_cache_rows: BTreeMap::new(),
            tool_cache_lookup_index: BTreeMap::new(),
            work_order_ledger: Vec::new(),
            work_orders_current: BTreeMap::new(),
            work_order_ledger_idempotency_index: BTreeMap::new(),
            capreq_ledger_events: Vec::new(),
            capreq_current: BTreeMap::new(),
            capreq_idempotency_index: BTreeMap::new(),
            audit_events: Vec::new(),
            next_memory_ledger_id: 1,
            next_conversation_turn_id: 1,
            next_audit_event_id: 1,
            next_position_lifecycle_event_id: 1,
            next_voice_enrollment_sample_seq: 1,
            next_process_blueprint_event_id: 1,
            next_simulation_catalog_event_id: 1,
            next_engine_capability_map_event_id: 1,
            next_artifact_id: 1,
            next_tool_cache_id: 1,
            next_work_order_event_id: 1,
            next_capreq_event_id: 1,
            next_ph1k_runtime_event_id: 1,
            memory_idempotency_index: BTreeMap::new(),
            conversation_idempotency_index: BTreeMap::new(),
            audit_idempotency_index_scoped: BTreeMap::new(),
            audit_idempotency_index_legacy: BTreeMap::new(),
            forgotten_memory: BTreeSet::new(),
        }
    }

    pub fn insert_identity(&mut self, record: IdentityRecord) -> Result<(), StorageError> {
        if self.identities.contains_key(&record.user_id) {
            return Err(StorageError::DuplicateKey {
                table: "identities",
                key: record.user_id.as_str().to_string(),
            });
        }
        self.identities.insert(record.user_id.clone(), record);
        Ok(())
    }

    pub fn insert_device(&mut self, record: DeviceRecord) -> Result<(), StorageError> {
        record.validate()?;
        if !self.identities.contains_key(&record.user_id) {
            return Err(StorageError::ForeignKeyViolation {
                table: "devices.user_id",
                key: record.user_id.as_str().to_string(),
            });
        }
        if self.devices.contains_key(&record.device_id) {
            return Err(StorageError::DuplicateKey {
                table: "devices",
                key: record.device_id.as_str().to_string(),
            });
        }
        self.devices.insert(record.device_id.clone(), record);
        Ok(())
    }

    pub fn insert_session(&mut self, record: SessionRecord) -> Result<(), StorageError> {
        record.validate()?;
        if !self.identities.contains_key(&record.user_id) {
            return Err(StorageError::ForeignKeyViolation {
                table: "sessions.user_id",
                key: record.user_id.as_str().to_string(),
            });
        }
        if !self.devices.contains_key(&record.device_id) {
            return Err(StorageError::ForeignKeyViolation {
                table: "sessions.device_id",
                key: record.device_id.as_str().to_string(),
            });
        }
        if self.sessions.contains_key(&record.session_id) {
            return Err(StorageError::DuplicateKey {
                table: "sessions",
                key: record.session_id.0.to_string(),
            });
        }
        self.sessions.insert(record.session_id, record);
        Ok(())
    }

    pub fn upsert_session_lifecycle(
        &mut self,
        record: SessionRecord,
        idempotency_key: Option<String>,
    ) -> Result<SessionId, StorageError> {
        record.validate()?;
        match (record.session_state, record.closed_at.is_some()) {
            (SessionState::Closed, true) => {}
            (SessionState::Closed, false) => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "sessions.closed_at",
                        reason: "must be set when session_state=Closed",
                    },
                ));
            }
            (_, true) => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "sessions.closed_at",
                        reason: "must be None unless session_state=Closed",
                    },
                ));
            }
            (_, false) => {}
        }
        if !self.identities.contains_key(&record.user_id) {
            return Err(StorageError::ForeignKeyViolation {
                table: "sessions.user_id",
                key: record.user_id.as_str().to_string(),
            });
        }
        if !self.devices.contains_key(&record.device_id) {
            return Err(StorageError::ForeignKeyViolation {
                table: "sessions.device_id",
                key: record.device_id.as_str().to_string(),
            });
        }

        if let Some(k) = idempotency_key {
            if k.trim().is_empty() {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "sessions.idempotency_key",
                        reason: "must not be empty when provided",
                    },
                ));
            }
            if k.len() > 128 {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "sessions.idempotency_key",
                        reason: "must be <= 128 chars",
                    },
                ));
            }
            let idem = (record.session_id, k);
            if self.session_lifecycle_idempotency_index.contains(&idem) {
                return Ok(record.session_id);
            }
            self.session_lifecycle_idempotency_index.insert(idem);
        }

        if let Some(existing) = self.sessions.get(&record.session_id) {
            if existing.user_id != record.user_id {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "sessions.user_id",
                        reason: "session user_id is immutable",
                    },
                ));
            }
            if existing.device_id != record.device_id {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "sessions.device_id",
                        reason: "session device_id is immutable",
                    },
                ));
            }
            if existing.opened_at != record.opened_at {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "sessions.opened_at",
                        reason: "opened_at is immutable",
                    },
                ));
            }
            if record.last_activity_at.0 < existing.last_activity_at.0 {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "sessions.last_activity_at",
                        reason: "must be monotonic per session",
                    },
                ));
            }
            if !is_allowed_session_transition(existing.session_state, record.session_state) {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "sessions.session_state",
                        reason: "invalid PH1.L transition",
                    },
                ));
            }
        }

        self.sessions.insert(record.session_id, record.clone());
        Ok(record.session_id)
    }

    pub fn get_identity(&self, user_id: &UserId) -> Option<&IdentityRecord> {
        self.identities.get(user_id)
    }

    pub fn get_device(&self, device_id: &DeviceId) -> Option<&DeviceRecord> {
        self.devices.get(device_id)
    }

    pub fn get_session(&self, session_id: &SessionId) -> Option<&SessionRecord> {
        self.sessions.get(session_id)
    }

    pub fn session_rows(&self) -> &BTreeMap<SessionId, SessionRecord> {
        &self.sessions
    }

    pub fn append_memory_ledger_event(
        &mut self,
        user_id: &UserId,
        event: MemoryLedgerEvent,
        use_policy: MemoryUsePolicy,
        expires_at: Option<MonotonicTimeNs>,
        idempotency_key: Option<String>,
    ) -> Result<u64, StorageError> {
        if !self.identities.contains_key(user_id) {
            return Err(StorageError::ForeignKeyViolation {
                table: "memory_ledger.user_id",
                key: user_id.as_str().to_string(),
            });
        }
        event.validate()?;

        if let Some(k) = &idempotency_key {
            if k.trim().is_empty() {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "memory_ledger.idempotency_key",
                        reason: "must not be empty when provided",
                    },
                ));
            }
            if k.len() > 128 {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "memory_ledger.idempotency_key",
                        reason: "must be <= 128 chars",
                    },
                ));
            }
            if let Some(existing) = self
                .memory_idempotency_index
                .get(&(user_id.clone(), k.clone()))
            {
                // Deterministic no-op on retry: return the original ledger_id.
                return Ok(*existing);
            }
        }

        let ledger_id = self.next_memory_ledger_id;
        self.next_memory_ledger_id = self.next_memory_ledger_id.saturating_add(1);

        self.memory_ledger.push(MemoryLedgerRow {
            ledger_id,
            user_id: user_id.clone(),
            event: event.clone(),
            idempotency_key: idempotency_key.clone(),
        });
        self.apply_memory_event_to_current(user_id, &event, use_policy, expires_at);

        if let Some(k) = &idempotency_key {
            self.memory_idempotency_index
                .insert((user_id.clone(), k.clone()), ledger_id);
        }

        Ok(ledger_id)
    }

    fn apply_memory_event_to_current(
        &mut self,
        user_id: &UserId,
        event: &MemoryLedgerEvent,
        use_policy: MemoryUsePolicy,
        expires_at: Option<MonotonicTimeNs>,
    ) {
        let key = (user_id.clone(), event.memory_key.clone());
        match event.kind {
            MemoryLedgerEventKind::Stored | MemoryLedgerEventKind::Updated => {
                let rec = MemoryCurrentRecord::v1(
                    user_id.clone(),
                    event.memory_key.clone(),
                    event.memory_value.clone(),
                    event.confidence,
                    event.sensitivity_flag,
                    event.t_event,
                    true,
                    use_policy,
                    expires_at,
                    event.provenance.clone(),
                );
                self.forgotten_memory.remove(&key);
                self.memory_current.insert(key, rec);
            }
            MemoryLedgerEventKind::Forgotten => {
                // Tombstone; remove value and mark inactive.
                self.forgotten_memory.insert(key.clone());
                let rec = MemoryCurrentRecord::v1(
                    user_id.clone(),
                    event.memory_key.clone(),
                    None,
                    event.confidence,
                    event.sensitivity_flag,
                    event.t_event,
                    false,
                    use_policy,
                    None,
                    event.provenance.clone(),
                );
                self.memory_current.insert(key, rec);
            }
        }
    }

    pub fn memory_ledger_rows(&self) -> &[MemoryLedgerRow] {
        &self.memory_ledger
    }

    pub fn memory_current(&self) -> &BTreeMap<(UserId, MemoryKey), MemoryCurrentRecord> {
        &self.memory_current
    }

    pub fn rebuild_memory_current_from_ledger(&mut self) {
        self.memory_current.clear();
        self.forgotten_memory.clear();
        self.memory_idempotency_index.clear();

        for row in self.memory_ledger.clone() {
            // Use conservative default policies when rebuilding.
            let use_policy = match row.event.layer {
                MemoryLayer::LongTerm => MemoryUsePolicy::AlwaysUsable,
                MemoryLayer::Working => MemoryUsePolicy::ContextRelevantOnly,
                MemoryLayer::Micro => MemoryUsePolicy::RepeatedOrConfirmed,
            };
            self.apply_memory_event_to_current(&row.user_id, &row.event, use_policy, None);
            if let Some(k) = &row.idempotency_key {
                self.memory_idempotency_index
                    .insert((row.user_id.clone(), k.clone()), row.ledger_id);
            }
        }
    }

    pub fn append_conversation_turn(
        &mut self,
        input: ConversationTurnInput,
    ) -> Result<ConversationTurnId, StorageError> {
        input.validate()?;

        if !self.identities.contains_key(&input.user_id) {
            return Err(StorageError::ForeignKeyViolation {
                table: "conversation_ledger.user_id",
                key: input.user_id.as_str().to_string(),
            });
        }
        if let Some(d) = &input.device_id {
            if !self.devices.contains_key(d) {
                return Err(StorageError::ForeignKeyViolation {
                    table: "conversation_ledger.device_id",
                    key: d.as_str().to_string(),
                });
            }
        }
        if let Some(s) = input.session_id {
            if !self.sessions.contains_key(&s) {
                return Err(StorageError::ForeignKeyViolation {
                    table: "conversation_ledger.session_id",
                    key: s.0.to_string(),
                });
            }
        }

        if let Some(k) = &input.idempotency_key {
            if let Some(existing) = self
                .conversation_idempotency_index
                .get(&(input.correlation_id, k.clone()))
            {
                // Deterministic no-op on retry: return the original conversation_turn_id.
                return Ok(*existing);
            }
        }

        let conversation_turn_id = ConversationTurnId(self.next_conversation_turn_id);
        self.next_conversation_turn_id = self.next_conversation_turn_id.saturating_add(1);

        let rec = ConversationTurnRecord::from_input_v1(conversation_turn_id, input)?;

        if let Some(k) = &rec.idempotency_key {
            self.conversation_idempotency_index
                .insert((rec.correlation_id, k.clone()), rec.conversation_turn_id);
        }
        self.conversation_ledger.push(rec);
        Ok(conversation_turn_id)
    }

    pub fn conversation_ledger(&self) -> &[ConversationTurnRecord] {
        &self.conversation_ledger
    }

    pub fn attempt_overwrite_conversation_turn(
        &mut self,
        _conversation_turn_id: ConversationTurnId,
    ) -> Result<(), StorageError> {
        Err(StorageError::AppendOnlyViolation {
            table: "conversation_ledger",
        })
    }

    pub fn attempt_overwrite_memory_ledger_row(
        &mut self,
        _ledger_id: u64,
    ) -> Result<(), StorageError> {
        Err(StorageError::AppendOnlyViolation {
            table: "memory_ledger",
        })
    }

    pub(crate) fn append_audit_event(
        &mut self,
        input: AuditEventInput,
    ) -> Result<AuditEventId, StorageError> {
        input.validate()?;

        if let Some(k) = &input.idempotency_key {
            if let (Some(tenant_id), Some(work_order_id)) = (&input.tenant_id, &input.work_order_id)
            {
                let idx = (tenant_id.clone(), work_order_id.clone(), k.clone());
                if let Some(existing) = self.audit_idempotency_index_scoped.get(&idx) {
                    // Deterministic no-op on retry: return the original event_id.
                    return Ok(*existing);
                }
            } else if let Some(existing) = self
                .audit_idempotency_index_legacy
                .get(&(input.correlation_id, k.clone()))
            {
                // Backward-compatible deterministic no-op for legacy scoped writes.
                return Ok(*existing);
            }
        }

        let event_id = AuditEventId(self.next_audit_event_id);
        self.next_audit_event_id = self.next_audit_event_id.saturating_add(1);

        let ev = AuditEvent::from_input_v1(event_id, input)?;

        if let Some(k) = &ev.idempotency_key {
            if let (Some(tenant_id), Some(work_order_id)) = (&ev.tenant_id, &ev.work_order_id) {
                self.audit_idempotency_index_scoped.insert(
                    (tenant_id.clone(), work_order_id.clone(), k.clone()),
                    ev.event_id,
                );
            } else {
                self.audit_idempotency_index_legacy
                    .insert((ev.correlation_id, k.clone()), ev.event_id);
            }
        }

        self.audit_events.push(ev);
        Ok(event_id)
    }

    pub fn audit_events(&self) -> &[AuditEvent] {
        &self.audit_events
    }

    pub fn attempt_overwrite_audit_event(
        &mut self,
        _event_id: AuditEventId,
    ) -> Result<(), StorageError> {
        Err(StorageError::AppendOnlyViolation {
            table: "audit_events",
        })
    }

    pub fn audit_events_by_correlation(&self, correlation_id: CorrelationId) -> Vec<&AuditEvent> {
        self.audit_events
            .iter()
            .filter(|e| e.correlation_id == correlation_id)
            .collect()
    }

    pub fn audit_events_by_tenant(&self, tenant_id: &str) -> Vec<&AuditEvent> {
        self.audit_events
            .iter()
            .filter(|e| e.tenant_id.as_deref() == Some(tenant_id))
            .collect()
    }

    pub fn last_turn_ids_for_correlation(&self, correlation_id: CorrelationId) -> BTreeSet<TurnId> {
        self.audit_events_by_correlation(correlation_id)
            .into_iter()
            .map(|e| e.turn_id)
            .collect()
    }

    // ------------------------
    // PBS tables (process_blueprints ledger + blueprint_registry current projection).
    // ------------------------

    fn apply_process_blueprint_event_to_registry(&mut self, ev: &ProcessBlueprintEvent) {
        let key = (ev.tenant_id.clone(), ev.intent_type.clone());
        let current = self.blueprint_registry.get(&key).cloned();
        let should_apply = match (&ev.status, current.as_ref()) {
            (BlueprintStatus::Active, _) => true,
            (BlueprintStatus::Draft, None) => true,
            (BlueprintStatus::Draft, Some(existing)) => {
                existing.process_id == ev.process_id
                    && existing.blueprint_version == ev.blueprint_version
                    && existing.status != BlueprintStatus::Active
            }
            (BlueprintStatus::Retired, Some(existing)) => {
                existing.process_id == ev.process_id
                    && existing.blueprint_version == ev.blueprint_version
            }
            (BlueprintStatus::Retired, None) => false,
        };

        if should_apply {
            let row = BlueprintRegistryRecord::v1(
                ev.tenant_id.clone(),
                ev.intent_type.clone(),
                ev.process_id.clone(),
                ev.blueprint_version,
                ev.status,
                ev.process_blueprint_event_id,
                ev.created_at,
            )
            .expect("process blueprint event already validated; projection must be valid");
            self.blueprint_registry.insert(key, row);
        }
    }

    pub fn append_process_blueprint_event(
        &mut self,
        input: ProcessBlueprintEventInput,
    ) -> Result<u64, StorageError> {
        input.validate()?;

        if let Some(k) = &input.idempotency_key {
            let idx = (
                input.tenant_id.clone(),
                input.process_id.clone(),
                input.blueprint_version,
                k.clone(),
            );
            if let Some(existing_id) = self.process_blueprint_idempotency_index.get(&idx) {
                // Deterministic no-op on retry.
                return Ok(*existing_id);
            }
        }

        let event_id = self.next_process_blueprint_event_id;
        self.next_process_blueprint_event_id =
            self.next_process_blueprint_event_id.saturating_add(1);

        let row = ProcessBlueprintEvent::from_input_v1(event_id, input)?;
        if let Some(k) = &row.idempotency_key {
            self.process_blueprint_idempotency_index.insert(
                (
                    row.tenant_id.clone(),
                    row.process_id.clone(),
                    row.blueprint_version,
                    k.clone(),
                ),
                row.process_blueprint_event_id,
            );
        }

        self.apply_process_blueprint_event_to_registry(&row);
        self.process_blueprint_events.push(row);
        Ok(event_id)
    }

    pub fn process_blueprint_events(&self) -> &[ProcessBlueprintEvent] {
        &self.process_blueprint_events
    }

    pub fn blueprint_registry(&self) -> &BTreeMap<(TenantId, IntentType), BlueprintRegistryRecord> {
        &self.blueprint_registry
    }

    pub fn blueprint_registry_row(
        &self,
        tenant_id: &TenantId,
        intent_type: &IntentType,
    ) -> Option<&BlueprintRegistryRecord> {
        self.blueprint_registry
            .get(&(tenant_id.clone(), intent_type.clone()))
    }

    pub fn rebuild_blueprint_registry_from_process_blueprint_events(&mut self) {
        self.blueprint_registry.clear();
        self.process_blueprint_idempotency_index.clear();
        let mut ordered = self.process_blueprint_events.clone();
        ordered.sort_by_key(|r| r.process_blueprint_event_id);
        for row in ordered {
            if let Some(k) = &row.idempotency_key {
                self.process_blueprint_idempotency_index.insert(
                    (
                        row.tenant_id.clone(),
                        row.process_id.clone(),
                        row.blueprint_version,
                        k.clone(),
                    ),
                    row.process_blueprint_event_id,
                );
            }
            self.apply_process_blueprint_event_to_registry(&row);
        }
    }

    pub fn attempt_overwrite_process_blueprint_event(
        &mut self,
        _process_blueprint_event_id: u64,
    ) -> Result<(), StorageError> {
        Err(StorageError::AppendOnlyViolation {
            table: "process_blueprints",
        })
    }

    // ------------------------
    // Simulation Catalog tables (`simulation_catalog` ledger + current projection).
    // ------------------------

    fn apply_simulation_catalog_event_to_current(&mut self, ev: &SimulationCatalogEvent) {
        let key = (ev.tenant_id.clone(), ev.simulation_id.clone());
        let should_apply = match self.simulation_catalog_current.get(&key) {
            Some(existing) => ev.simulation_version >= existing.simulation_version,
            None => true,
        };
        if !should_apply {
            return;
        }

        let row = SimulationCatalogCurrentRecord::v1(
            ev.tenant_id.clone(),
            ev.simulation_id.clone(),
            ev.simulation_version,
            ev.simulation_type,
            ev.status,
            ev.owning_domain.clone(),
            ev.simulation_catalog_event_id,
            ev.created_at,
        )
        .expect("simulation catalog event already validated; projection must be valid");

        self.simulation_catalog_current.insert(key, row);
    }

    pub fn append_simulation_catalog_event(
        &mut self,
        input: SimulationCatalogEventInput,
    ) -> Result<u64, StorageError> {
        input.validate()?;

        if let Some(k) = &input.idempotency_key {
            let idx = (
                input.tenant_id.clone(),
                input.simulation_id.clone(),
                input.simulation_version,
                k.clone(),
            );
            if let Some(existing_id) = self.simulation_catalog_idempotency_index.get(&idx) {
                // Deterministic no-op on retry.
                return Ok(*existing_id);
            }
        }

        let event_id = self.next_simulation_catalog_event_id;
        self.next_simulation_catalog_event_id =
            self.next_simulation_catalog_event_id.saturating_add(1);

        let row = SimulationCatalogEvent::from_input_v1(event_id, input)?;
        if let Some(k) = &row.idempotency_key {
            self.simulation_catalog_idempotency_index.insert(
                (
                    row.tenant_id.clone(),
                    row.simulation_id.clone(),
                    row.simulation_version,
                    k.clone(),
                ),
                row.simulation_catalog_event_id,
            );
        }

        self.apply_simulation_catalog_event_to_current(&row);
        self.simulation_catalog_events.push(row);
        Ok(event_id)
    }

    pub fn simulation_catalog_events(&self) -> &[SimulationCatalogEvent] {
        &self.simulation_catalog_events
    }

    pub fn simulation_catalog_current(
        &self,
    ) -> &BTreeMap<(TenantId, SimulationId), SimulationCatalogCurrentRecord> {
        &self.simulation_catalog_current
    }

    pub fn simulation_catalog_current_row(
        &self,
        tenant_id: &TenantId,
        simulation_id: &SimulationId,
    ) -> Option<&SimulationCatalogCurrentRecord> {
        self.simulation_catalog_current
            .get(&(tenant_id.clone(), simulation_id.clone()))
    }

    pub fn rebuild_simulation_catalog_current_from_ledger(&mut self) {
        self.simulation_catalog_current.clear();
        self.simulation_catalog_idempotency_index.clear();
        let mut ordered = self.simulation_catalog_events.clone();
        ordered.sort_by_key(|r| r.simulation_catalog_event_id);
        for row in ordered {
            if let Some(k) = &row.idempotency_key {
                self.simulation_catalog_idempotency_index.insert(
                    (
                        row.tenant_id.clone(),
                        row.simulation_id.clone(),
                        row.simulation_version,
                        k.clone(),
                    ),
                    row.simulation_catalog_event_id,
                );
            }
            self.apply_simulation_catalog_event_to_current(&row);
        }
    }

    pub fn attempt_overwrite_simulation_catalog_event(
        &mut self,
        _simulation_catalog_event_id: u64,
    ) -> Result<(), StorageError> {
        Err(StorageError::AppendOnlyViolation {
            table: "simulation_catalog",
        })
    }

    // ------------------------
    // Engine Capability Maps tables (`engine_capability_maps` ledger + current projection).
    // ------------------------

    fn apply_engine_capability_map_event_to_current(&mut self, ev: &EngineCapabilityMapEvent) {
        let key = (
            ev.tenant_id.clone(),
            ev.engine_id.clone(),
            ev.capability_id.clone(),
        );
        let should_apply = match self.engine_capability_maps_current.get(&key) {
            Some(existing) => ev.capability_map_version >= existing.capability_map_version,
            None => true,
        };
        if !should_apply {
            return;
        }

        let row = EngineCapabilityMapCurrentRecord::v1(
            ev.tenant_id.clone(),
            ev.engine_id.clone(),
            ev.capability_id.clone(),
            ev.capability_map_version,
            ev.map_status,
            ev.owning_domain.clone(),
            ev.capability_name.clone(),
            ev.allowed_callers,
            ev.side_effects_mode,
            ev.engine_capability_map_event_id,
            ev.created_at,
        )
        .expect("engine capability map event already validated; projection must be valid");

        self.engine_capability_maps_current.insert(key, row);
    }

    pub fn append_engine_capability_map_event(
        &mut self,
        input: EngineCapabilityMapEventInput,
    ) -> Result<u64, StorageError> {
        input.validate()?;

        if let Some(k) = &input.idempotency_key {
            let idx = (
                input.tenant_id.clone(),
                input.engine_id.clone(),
                input.capability_id.clone(),
                input.capability_map_version,
                k.clone(),
            );
            if let Some(existing_id) = self.engine_capability_map_idempotency_index.get(&idx) {
                // Deterministic no-op on retry.
                return Ok(*existing_id);
            }
        }

        let event_id = self.next_engine_capability_map_event_id;
        self.next_engine_capability_map_event_id =
            self.next_engine_capability_map_event_id.saturating_add(1);

        let row = EngineCapabilityMapEvent::from_input_v1(event_id, input)?;
        if let Some(k) = &row.idempotency_key {
            self.engine_capability_map_idempotency_index.insert(
                (
                    row.tenant_id.clone(),
                    row.engine_id.clone(),
                    row.capability_id.clone(),
                    row.capability_map_version,
                    k.clone(),
                ),
                row.engine_capability_map_event_id,
            );
        }

        self.apply_engine_capability_map_event_to_current(&row);
        self.engine_capability_map_events.push(row);
        Ok(event_id)
    }

    pub fn engine_capability_map_events(&self) -> &[EngineCapabilityMapEvent] {
        &self.engine_capability_map_events
    }

    pub fn engine_capability_maps_current(
        &self,
    ) -> &BTreeMap<(TenantId, EngineId, CapabilityId), EngineCapabilityMapCurrentRecord> {
        &self.engine_capability_maps_current
    }

    pub fn engine_capability_maps_current_row(
        &self,
        tenant_id: &TenantId,
        engine_id: &EngineId,
        capability_id: &CapabilityId,
    ) -> Option<&EngineCapabilityMapCurrentRecord> {
        self.engine_capability_maps_current.get(&(
            tenant_id.clone(),
            engine_id.clone(),
            capability_id.clone(),
        ))
    }

    pub fn rebuild_engine_capability_maps_current_from_ledger(&mut self) {
        self.engine_capability_maps_current.clear();
        self.engine_capability_map_idempotency_index.clear();
        let mut ordered = self.engine_capability_map_events.clone();
        ordered.sort_by_key(|r| r.engine_capability_map_event_id);
        for row in ordered {
            if let Some(k) = &row.idempotency_key {
                self.engine_capability_map_idempotency_index.insert(
                    (
                        row.tenant_id.clone(),
                        row.engine_id.clone(),
                        row.capability_id.clone(),
                        row.capability_map_version,
                        k.clone(),
                    ),
                    row.engine_capability_map_event_id,
                );
            }
            self.apply_engine_capability_map_event_to_current(&row);
        }
    }

    pub fn attempt_overwrite_engine_capability_map_event(
        &mut self,
        _engine_capability_map_event_id: u64,
    ) -> Result<(), StorageError> {
        Err(StorageError::AppendOnlyViolation {
            table: "engine_capability_maps",
        })
    }

    // ------------------------
    // Artifacts ledger + tool cache tables.
    // ------------------------

    pub fn append_artifact_ledger_row(
        &mut self,
        input: ArtifactLedgerRowInput,
    ) -> Result<u64, StorageError> {
        input.validate()?;

        if let Some(k) = &input.idempotency_key {
            let idx = (
                input.scope_type,
                input.scope_id.clone(),
                input.artifact_type,
                input.artifact_version,
                k.clone(),
            );
            if let Some(existing_id) = self.artifacts_idempotency_index.get(&idx) {
                // Deterministic no-op on retry.
                return Ok(*existing_id);
            }
        }

        let unique_key = (
            input.scope_type,
            input.scope_id.clone(),
            input.artifact_type,
            input.artifact_version,
        );
        if let Some(existing_id) = self.artifacts_scope_version_index.get(&unique_key) {
            return Err(StorageError::DuplicateKey {
                table: "artifacts_ledger.scope_type_scope_id_artifact_type_artifact_version",
                key: existing_id.to_string(),
            });
        }

        let artifact_id = self.next_artifact_id;
        self.next_artifact_id = self.next_artifact_id.saturating_add(1);

        let row = ArtifactLedgerRow::from_input_v1(artifact_id, input)?;
        self.artifacts_scope_version_index.insert(
            (
                row.scope_type,
                row.scope_id.clone(),
                row.artifact_type,
                row.artifact_version,
            ),
            row.artifact_id,
        );
        if let Some(k) = &row.idempotency_key {
            self.artifacts_idempotency_index.insert(
                (
                    row.scope_type,
                    row.scope_id.clone(),
                    row.artifact_type,
                    row.artifact_version,
                    k.clone(),
                ),
                row.artifact_id,
            );
        }
        self.artifacts_ledger_rows.push(row);
        Ok(artifact_id)
    }

    pub fn artifacts_ledger_rows(&self) -> &[ArtifactLedgerRow] {
        &self.artifacts_ledger_rows
    }

    pub fn artifact_ledger_row(
        &self,
        scope_type: ArtifactScopeType,
        scope_id: &str,
        artifact_type: ArtifactType,
        artifact_version: ArtifactVersion,
    ) -> Option<&ArtifactLedgerRow> {
        let artifact_id = self.artifacts_scope_version_index.get(&(
            scope_type,
            scope_id.to_string(),
            artifact_type,
            artifact_version,
        ))?;
        self.artifacts_ledger_rows
            .iter()
            .find(|r| r.artifact_id == *artifact_id)
    }

    pub fn attempt_overwrite_artifact_ledger_row(
        &mut self,
        _artifact_id: u64,
    ) -> Result<(), StorageError> {
        Err(StorageError::AppendOnlyViolation {
            table: "artifacts_ledger",
        })
    }

    pub fn upsert_tool_cache_row(&mut self, input: ToolCacheRowInput) -> Result<u64, StorageError> {
        input.validate()?;
        let lookup = (
            input.tool_name.clone(),
            input.query_hash.clone(),
            input.locale.clone(),
        );

        if let Some(existing_id) = self.tool_cache_lookup_index.get(&lookup).copied() {
            let updated = ToolCacheRow::from_input_v1(existing_id, input)?;
            self.tool_cache_rows.insert(existing_id, updated);
            return Ok(existing_id);
        }

        let cache_id = self.next_tool_cache_id;
        self.next_tool_cache_id = self.next_tool_cache_id.saturating_add(1);
        let row = ToolCacheRow::from_input_v1(cache_id, input)?;
        self.tool_cache_lookup_index.insert(lookup, cache_id);
        self.tool_cache_rows.insert(cache_id, row);
        Ok(cache_id)
    }

    pub fn tool_cache_rows(&self) -> &BTreeMap<u64, ToolCacheRow> {
        &self.tool_cache_rows
    }

    pub fn tool_cache_row(
        &self,
        tool_name: &str,
        query_hash: &str,
        locale: &str,
    ) -> Option<&ToolCacheRow> {
        let id = self.tool_cache_lookup_index.get(&(
            tool_name.to_string(),
            query_hash.to_string(),
            locale.to_string(),
        ))?;
        self.tool_cache_rows.get(id)
    }

    pub fn tool_cache_hit(
        &self,
        tool_name: &str,
        query_hash: &str,
        locale: &str,
        now: MonotonicTimeNs,
    ) -> Option<&ToolCacheRow> {
        let row = self.tool_cache_row(tool_name, query_hash, locale)?;
        if row.expires_at <= now {
            return None;
        }
        Some(row)
    }

    // ------------------------
    // Selene OS core WorkOrder persistence (ledger + current projection).
    // ------------------------

    fn apply_work_order_event_to_current(&mut self, ev: &WorkOrderLedgerEvent) {
        let key = (ev.tenant_id.clone(), ev.work_order_id.clone());
        let next = WorkOrderCurrentRecord::v1(
            ev.tenant_id.clone(),
            ev.work_order_id.clone(),
            ev.correlation_id,
            ev.turn_id,
            ev.work_order_status,
            ev.work_order_event_id,
            ev.reason_code,
            ev.created_at,
            ev.step_id.clone(),
            ev.step_input_hash.clone(),
            ev.lease_owner_id.clone(),
            ev.lease_token_hash.clone(),
            ev.lease_expires_at,
        )
        .expect("ledger event already validated; projection must be valid");

        self.work_orders_current.insert(key, next);
    }

    pub fn append_work_order_ledger_event(
        &mut self,
        input: WorkOrderLedgerEventInput,
    ) -> Result<u64, StorageError> {
        input.validate()?;

        if let Some(k) = &input.idempotency_key {
            let idx = (
                input.tenant_id.clone(),
                input.work_order_id.clone(),
                k.clone(),
            );
            if let Some(existing_id) = self.work_order_ledger_idempotency_index.get(&idx) {
                // Deterministic no-op on retry.
                return Ok(*existing_id);
            }
        }

        let work_order_event_id = self.next_work_order_event_id;
        self.next_work_order_event_id = self.next_work_order_event_id.saturating_add(1);

        let row = WorkOrderLedgerEvent::from_input_v1(work_order_event_id, input)?;
        if let Some(k) = &row.idempotency_key {
            self.work_order_ledger_idempotency_index.insert(
                (row.tenant_id.clone(), row.work_order_id.clone(), k.clone()),
                row.work_order_event_id,
            );
        }

        self.apply_work_order_event_to_current(&row);
        self.work_order_ledger.push(row);
        Ok(work_order_event_id)
    }

    pub fn work_order_ledger(&self) -> &[WorkOrderLedgerEvent] {
        &self.work_order_ledger
    }

    pub fn work_orders_current(
        &self,
    ) -> &BTreeMap<(TenantId, WorkOrderId), WorkOrderCurrentRecord> {
        &self.work_orders_current
    }

    pub fn work_order_current(
        &self,
        tenant_id: &TenantId,
        work_order_id: &WorkOrderId,
    ) -> Option<&WorkOrderCurrentRecord> {
        self.work_orders_current
            .get(&(tenant_id.clone(), work_order_id.clone()))
    }

    pub fn rebuild_work_orders_current_from_ledger(&mut self) {
        self.work_orders_current.clear();
        self.work_order_ledger_idempotency_index.clear();
        let mut ordered = self.work_order_ledger.clone();
        ordered.sort_by_key(|r| r.work_order_event_id);
        for row in ordered {
            if let Some(k) = &row.idempotency_key {
                self.work_order_ledger_idempotency_index.insert(
                    (row.tenant_id.clone(), row.work_order_id.clone(), k.clone()),
                    row.work_order_event_id,
                );
            }
            self.apply_work_order_event_to_current(&row);
        }
    }

    pub fn attempt_overwrite_work_order_ledger_event(
        &mut self,
        _work_order_event_id: u64,
    ) -> Result<(), StorageError> {
        Err(StorageError::AppendOnlyViolation {
            table: "work_order_ledger",
        })
    }

    // ------------------------
    // PH1.CAPREQ persistence (append-only ledger + rebuildable current projection).
    // ------------------------

    fn apply_capreq_event_to_current(&mut self, ev: &CapabilityRequestLedgerEvent) {
        let key = (ev.tenant_id.clone(), ev.capreq_id.clone());
        let should_apply = match self.capreq_current.get(&key) {
            Some(existing) => ev.capreq_event_id >= existing.source_event_id,
            None => true,
        };
        if !should_apply {
            return;
        }

        let row = CapabilityRequestCurrentRecord::v1(
            ev.tenant_id.clone(),
            ev.capreq_id.clone(),
            ev.requester_user_id.clone(),
            ev.status,
            ev.action,
            ev.payload_hash.clone(),
            ev.capreq_event_id,
            ev.created_at,
            ev.reason_code,
        )
        .expect("capreq event already validated; projection must be valid");

        self.capreq_current.insert(key, row);
    }

    pub fn append_capreq_event(
        &mut self,
        input: CapabilityRequestLedgerEventInput,
    ) -> Result<u64, StorageError> {
        input.validate()?;

        if !self.identities.contains_key(&input.requester_user_id) {
            return Err(StorageError::ForeignKeyViolation {
                table: "capreq_ledger.requester_user_id",
                key: input.requester_user_id.as_str().to_string(),
            });
        }

        if let Some(k) = &input.idempotency_key {
            let idx = (input.tenant_id.clone(), input.capreq_id.clone(), k.clone());
            if let Some(existing_id) = self.capreq_idempotency_index.get(&idx) {
                return Ok(*existing_id);
            }
        }

        let capreq_event_id = self.next_capreq_event_id;
        self.next_capreq_event_id = self.next_capreq_event_id.saturating_add(1);

        let row = CapabilityRequestLedgerEvent::from_input_v1(capreq_event_id, input)?;
        if let Some(k) = &row.idempotency_key {
            self.capreq_idempotency_index.insert(
                (row.tenant_id.clone(), row.capreq_id.clone(), k.clone()),
                row.capreq_event_id,
            );
        }

        self.apply_capreq_event_to_current(&row);
        self.capreq_ledger_events.push(row);
        Ok(capreq_event_id)
    }

    pub fn capreq_events(&self) -> &[CapabilityRequestLedgerEvent] {
        &self.capreq_ledger_events
    }

    pub fn capreq_current(
        &self,
    ) -> &BTreeMap<(TenantId, CapreqId), CapabilityRequestCurrentRecord> {
        &self.capreq_current
    }

    pub fn capreq_current_row(
        &self,
        tenant_id: &TenantId,
        capreq_id: &CapreqId,
    ) -> Option<&CapabilityRequestCurrentRecord> {
        self.capreq_current
            .get(&(tenant_id.clone(), capreq_id.clone()))
    }

    pub fn rebuild_capreq_current_from_ledger(&mut self) {
        self.capreq_current.clear();
        self.capreq_idempotency_index.clear();
        let mut ordered = self.capreq_ledger_events.clone();
        ordered.sort_by_key(|r| r.capreq_event_id);
        for row in ordered {
            if let Some(k) = &row.idempotency_key {
                self.capreq_idempotency_index.insert(
                    (row.tenant_id.clone(), row.capreq_id.clone(), k.clone()),
                    row.capreq_event_id,
                );
            }
            self.apply_capreq_event_to_current(&row);
        }
    }

    pub fn attempt_overwrite_capreq_event(
        &mut self,
        _capreq_event_id: u64,
    ) -> Result<(), StorageError> {
        Err(StorageError::AppendOnlyViolation {
            table: "capreq_ledger",
        })
    }

    // ------------------------
    // PH1.LINK (Link Engine) - minimal storage API for simulations.
    // ------------------------

    fn validate_ph1link_optional_tenant_id(tenant_id: &Option<String>) -> Result<(), StorageError> {
        if let Some(tid) = tenant_id {
            if tid.trim().is_empty() || tid.len() > 64 {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1link.tenant_id",
                        reason: "must be non-empty and <= 64 chars when provided",
                    },
                ));
            }
        }
        Ok(())
    }

    fn validate_ph1link_inviter_tenant_scope(
        inviter_user_id: &UserId,
        tenant_id: &Option<String>,
        prefilled_context: &Option<PrefilledContext>,
    ) -> Result<(), StorageError> {
        Self::validate_ph1link_optional_tenant_id(tenant_id)?;

        let effective_tenant_id = tenant_id.clone().or_else(|| {
            prefilled_context
                .as_ref()
                .and_then(|ctx| ctx.tenant_id.clone())
        });

        if let Some(tid) = effective_tenant_id {
            // Compatibility rule:
            // - If user_id is tenant-scoped in "<tenant_id>:<local_user>" form, enforce match.
            // - If user_id has no tenant prefix, allow (legacy fixtures / non-prefixed ids).
            if let Some((user_tenant_scope, _)) = inviter_user_id.as_str().split_once(':') {
                if !user_tenant_scope.is_empty() && user_tenant_scope != tid {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "ph1link.inviter_user_id",
                            reason: "must match tenant scope",
                        },
                    ));
                }
            }

            if let Some(ctx_tenant_id) = prefilled_context
                .as_ref()
                .and_then(|ctx| ctx.tenant_id.as_ref())
            {
                if ctx_tenant_id != &tid {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "ph1link.prefilled_context.tenant_id",
                            reason: "must match tenant_id when both are provided",
                        },
                    ));
                }
            }
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1link_invite_generate_draft(
        &mut self,
        now: MonotonicTimeNs,
        inviter_user_id: UserId,
        invitee_type: InviteeType,
        recipient_contact: String,
        delivery_method: DeliveryMethod,
        tenant_id: Option<String>,
        prefilled_context: Option<PrefilledContext>,
        expiration_policy_id: Option<String>,
    ) -> Result<(LinkRecord, LinkGenerateResultParts), StorageError> {
        if !self.identities.contains_key(&inviter_user_id) {
            return Err(StorageError::ForeignKeyViolation {
                table: "links.inviter_user_id",
                key: inviter_user_id.as_str().to_string(),
            });
        }
        Self::validate_ph1link_inviter_tenant_scope(
            &inviter_user_id,
            &tenant_id,
            &prefilled_context,
        )?;

        let payload_hash = deterministic_payload_hash_hex(
            &inviter_user_id,
            invitee_type,
            &recipient_contact,
            delivery_method,
            &tenant_id,
            &expiration_policy_id,
            &prefilled_context,
        );

        let idx_key = (
            inviter_user_id.clone(),
            payload_hash.clone(),
            expiration_policy_id.clone(),
        );
        if let Some(existing_id) = self.link_draft_idempotency_index.get(&idx_key) {
            let existing =
                self.links
                    .get(existing_id)
                    .cloned()
                    .ok_or(StorageError::ForeignKeyViolation {
                        table: "links.link_id",
                        key: existing_id.as_str().to_string(),
                    })?;

            return Ok((
                existing,
                LinkGenerateResultParts {
                    payload_hash,
                    was_new: false,
                },
            ));
        }

        // Deterministic TTL: 7 days (in ns) unless a policy-specific TTL exists (not implemented yet).
        const TTL_NS: u64 = 7 * 24 * 60 * 60 * 1_000_000_000;
        let expires_at = MonotonicTimeNs(now.0.saturating_add(TTL_NS));

        let link_seq = self.next_link_seq;
        self.next_link_seq = self.next_link_seq.saturating_add(1);
        let link_id = LinkId::new(format!("link_{link_seq}_{payload_hash}"))?;

        let recipient_contact_hash = deterministic_contact_hash_hex(&recipient_contact);

        let rec = LinkRecord::v1(
            link_id.clone(),
            payload_hash.clone(),
            LinkStatus::DraftCreated,
            now,
            expires_at,
            inviter_user_id,
            invitee_type,
            delivery_method,
            recipient_contact_hash,
            expiration_policy_id,
            prefilled_context,
            None,
            None,
        )?;

        self.links.insert(link_id.clone(), rec.clone());
        self.link_draft_idempotency_index.insert(idx_key, link_id);

        Ok((
            rec,
            LinkGenerateResultParts {
                payload_hash,
                was_new: true,
            },
        ))
    }

    pub fn ph1link_get_link(&self, link_id: &LinkId) -> Option<&LinkRecord> {
        self.links.get(link_id)
    }

    pub fn ph1link_delivery_proofs_for_link(
        &self,
        link_id: &LinkId,
    ) -> Vec<&LinkDeliveryProofRecord> {
        self.link_delivery_proofs
            .values()
            .filter(|p| &p.link_id == link_id)
            .collect()
    }

    pub fn ph1link_invite_send_commit(
        &mut self,
        now: MonotonicTimeNs,
        link_id: LinkId,
        delivery_method: DeliveryMethod,
        recipient_contact: String,
        idempotency_key: String,
    ) -> Result<LinkDeliveryProofRecord, StorageError> {
        let rec = self
            .links
            .get_mut(&link_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "links.link_id",
                key: link_id.as_str().to_string(),
            })?;

        if matches!(rec.status, LinkStatus::Expired | LinkStatus::Revoked) {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1link_invite_send_commit.link_status",
                    reason: "cannot send expired or revoked links",
                },
            ));
        }

        let recipient_contact_hash = deterministic_contact_hash_hex(&recipient_contact);
        let idx_key = (
            link_id.clone(),
            delivery_method,
            recipient_contact_hash.clone(),
            idempotency_key.clone(),
        );
        if let Some(existing_proof_ref) = self.link_delivery_idempotency_index.get(&idx_key) {
            return self
                .link_delivery_proofs
                .get(existing_proof_ref)
                .cloned()
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "link_delivery_proofs.delivery_proof_ref",
                    key: existing_proof_ref.as_str().to_string(),
                });
        }

        let proof_seq = self.next_link_delivery_proof_seq;
        self.next_link_delivery_proof_seq = self.next_link_delivery_proof_seq.saturating_add(1);
        let proof_ref = DeliveryProofRef::new(format!("proof_{proof_seq}_{}", rec.payload_hash))?;

        let proof = LinkDeliveryProofRecord {
            schema_version: SchemaVersion(1),
            delivery_proof_ref: proof_ref.clone(),
            created_at: now,
            link_id: link_id.clone(),
            delivery_method,
            recipient_contact_hash,
            delivery_status: DeliveryStatus::Sent,
            idempotency_key: idempotency_key.clone(),
        };

        self.link_delivery_proofs
            .insert(proof_ref.clone(), proof.clone());
        self.link_delivery_idempotency_index
            .insert(idx_key, proof_ref);

        // Update current-state link status deterministically.
        rec.status = LinkStatus::Sent;
        rec.delivery_method = delivery_method;

        Ok(proof)
    }

    pub fn ph1link_invite_open_activate_commit(
        &mut self,
        now: MonotonicTimeNs,
        link_id: LinkId,
        device_fingerprint: String,
    ) -> Result<(LinkStatus, Option<String>, Option<PrefilledContextRef>), StorageError> {
        let rec = self
            .links
            .get_mut(&link_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "links.link_id",
                key: link_id.as_str().to_string(),
            })?;

        // Expiry gate.
        if now.0 > rec.expires_at.0 {
            rec.status = LinkStatus::Expired;
            return Ok((LinkStatus::Expired, None, None));
        }

        // Revocation gate.
        if rec.status == LinkStatus::Revoked {
            return Ok((LinkStatus::Revoked, None, None));
        }

        let df_hash = deterministic_device_fingerprint_hash_hex(&device_fingerprint);
        match &rec.bound_device_fingerprint_hash {
            None => {
                rec.bound_device_fingerprint_hash = Some(df_hash.clone());
            }
            Some(existing) if existing != &df_hash => {
                // Forwarded-link protection: fail closed.
                rec.status = LinkStatus::Blocked;
                return Ok((LinkStatus::Blocked, Some(existing.clone()), None));
            }
            _ => {}
        }

        // Mark OPENED/ACTIVATED in current record. In v1 skeleton, we mark ACTIVATED directly.
        rec.status = LinkStatus::Activated;
        let ctx_ref = rec
            .prefilled_context
            .as_ref()
            .map(|_| PrefilledContextRef::new(format!("prefilled:{}", rec.link_id.as_str())))
            .transpose()?;

        Ok((
            LinkStatus::Activated,
            rec.bound_device_fingerprint_hash.clone(),
            ctx_ref,
        ))
    }

    pub fn ph1link_invite_revoke_revoke(
        &mut self,
        link_id: LinkId,
        reason: String,
    ) -> Result<(), StorageError> {
        let rec = self
            .links
            .get_mut(&link_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "links.link_id",
                key: link_id.as_str().to_string(),
            })?;
        rec.status = LinkStatus::Revoked;
        rec.revoked_reason = Some(reason);
        Ok(())
    }

    pub fn ph1link_invite_expired_recovery_commit(
        &mut self,
        now: MonotonicTimeNs,
        expired_link_id: LinkId,
        delivery_method: Option<DeliveryMethod>,
        recipient_contact: Option<String>,
        idempotency_key: String,
    ) -> Result<LinkRecord, StorageError> {
        let old =
            self.links
                .get_mut(&expired_link_id)
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "links.link_id",
                    key: expired_link_id.as_str().to_string(),
                })?;

        if old.status == LinkStatus::Revoked {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1link_invite_expired_recovery_commit.link_status",
                    reason: "cannot recover revoked links",
                },
            ));
        }

        // Mark expired deterministically if time passed.
        if now.0 > old.expires_at.0 {
            old.status = LinkStatus::Expired;
        }

        if old.status != LinkStatus::Expired {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1link_invite_expired_recovery_commit.link_status",
                    reason: "link must be EXPIRED",
                },
            ));
        }

        let idx_key = (expired_link_id.clone(), idempotency_key.clone());
        if let Some(existing_new_id) = self.link_recovery_idempotency_index.get(&idx_key) {
            return self.links.get(existing_new_id).cloned().ok_or(
                StorageError::ForeignKeyViolation {
                    table: "links.link_id",
                    key: existing_new_id.as_str().to_string(),
                },
            );
        }

        // Deterministic TTL: 7 days (in ns) unless a policy-specific TTL exists (not implemented yet).
        const TTL_NS: u64 = 7 * 24 * 60 * 60 * 1_000_000_000;
        let expires_at = MonotonicTimeNs(now.0.saturating_add(TTL_NS));

        let link_seq = self.next_link_seq;
        self.next_link_seq = self.next_link_seq.saturating_add(1);
        let link_id = LinkId::new(format!("link_{link_seq}_{}", old.payload_hash))?;

        let new_delivery_method = delivery_method.unwrap_or(old.delivery_method);
        let new_recipient_contact_hash = match recipient_contact {
            Some(c) => deterministic_contact_hash_hex(&c),
            None => old.recipient_contact_hash.clone(),
        };

        let rec = LinkRecord::v1(
            link_id.clone(),
            old.payload_hash.clone(),
            LinkStatus::DraftCreated,
            now,
            expires_at,
            old.inviter_user_id.clone(),
            old.invitee_type,
            new_delivery_method,
            new_recipient_contact_hash,
            old.expiration_policy_id.clone(),
            old.prefilled_context.clone(),
            None,
            None,
        )?;

        self.links.insert(link_id.clone(), rec.clone());
        self.link_recovery_idempotency_index
            .insert(idx_key, link_id);

        Ok(rec)
    }

    pub fn ph1link_invite_forward_block_commit(
        &mut self,
        link_id: LinkId,
        presented_device_fingerprint: String,
    ) -> Result<(LinkStatus, Option<String>), StorageError> {
        let rec = self
            .links
            .get_mut(&link_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "links.link_id",
                key: link_id.as_str().to_string(),
            })?;

        let presented_hash =
            deterministic_device_fingerprint_hash_hex(&presented_device_fingerprint);
        let bound =
            rec.bound_device_fingerprint_hash
                .clone()
                .ok_or(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1link_invite_forward_block_commit.bound_device_fingerprint_hash",
                        reason: "link must already be bound to a different device",
                    },
                ))?;

        if bound == presented_hash {
            return Err(StorageError::ContractViolation(ContractViolation::InvalidValue {
                field: "ph1link_invite_forward_block_commit.presented_device_fingerprint",
                reason: "presented fingerprint matches bound fingerprint (not a forwarded-link case)",
            }));
        }

        let key = (link_id.clone(), presented_hash);
        if self.link_forward_block_attempts.contains(&key) {
            rec.status = LinkStatus::Blocked;
            return Ok((LinkStatus::Blocked, Some(bound)));
        }

        self.link_forward_block_attempts.insert(key);
        rec.status = LinkStatus::Blocked;
        Ok((LinkStatus::Blocked, Some(bound)))
    }

    pub fn ph1link_role_propose_draft(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: Option<String>,
        proposal_text: String,
    ) -> Result<String, StorageError> {
        let proposal_hash = hash_hex_64(&proposal_text);
        let idx_key = (tenant_id.clone(), proposal_hash.clone());
        if let Some(existing) = self.link_role_proposal_idempotency_index.get(&idx_key) {
            return Ok(existing.clone());
        }

        let role_proposal_id = format!("role_prop_{proposal_hash}");
        let rec = LinkRoleProposalRecord {
            schema_version: SchemaVersion(1),
            role_proposal_id: role_proposal_id.clone(),
            created_at: now,
            tenant_id,
            proposal_text,
        };

        self.link_role_proposals
            .insert(role_proposal_id.clone(), rec);
        self.link_role_proposal_idempotency_index
            .insert(idx_key, role_proposal_id.clone());

        Ok(role_proposal_id)
    }

    pub fn ph1link_dual_role_conflict_escalate_draft(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: Option<String>,
        link_id: Option<LinkId>,
        note: String,
    ) -> Result<String, StorageError> {
        let note_hash = hash_hex_64(&note);
        let idx_key = (tenant_id.clone(), link_id.clone(), note_hash.clone());
        if let Some(existing) = self.link_dual_role_conflict_idempotency_index.get(&idx_key) {
            return Ok(existing.clone());
        }

        let escalation_case_id = format!("esc_{note_hash}");
        let rec = LinkDualRoleConflictCaseRecord {
            schema_version: SchemaVersion(1),
            escalation_case_id: escalation_case_id.clone(),
            created_at: now,
            tenant_id,
            link_id,
            note,
        };

        self.link_dual_role_conflict_cases
            .insert(escalation_case_id.clone(), rec);
        self.link_dual_role_conflict_idempotency_index
            .insert(idx_key, escalation_case_id.clone());

        Ok(escalation_case_id)
    }

    pub fn ph1link_delivery_failure_handling_commit(
        &mut self,
        now: MonotonicTimeNs,
        link_id: LinkId,
        attempt: u8,
        idempotency_key: String,
    ) -> Result<LinkDeliveryProofRecord, StorageError> {
        let rec = self
            .links
            .get_mut(&link_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "links.link_id",
                key: link_id.as_str().to_string(),
            })?;

        if matches!(rec.status, LinkStatus::Expired | LinkStatus::Revoked) {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1link_delivery_failure_handling_commit.link_status",
                    reason: "cannot deliver expired or revoked links",
                },
            ));
        }

        let idx_key = (link_id.clone(), idempotency_key.clone());
        if let Some(existing_proof_ref) = self.link_delivery_failure_idempotency_index.get(&idx_key)
        {
            return self
                .link_delivery_proofs
                .get(existing_proof_ref)
                .cloned()
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "link_delivery_proofs.delivery_proof_ref",
                    key: existing_proof_ref.as_str().to_string(),
                });
        }

        let delivery_status = if attempt <= 3 {
            DeliveryStatus::Sent
        } else {
            DeliveryStatus::Fail
        };

        let proof_seq = self.next_link_delivery_proof_seq;
        self.next_link_delivery_proof_seq = self.next_link_delivery_proof_seq.saturating_add(1);
        let proof_ref =
            DeliveryProofRef::new(format!("proof_fail_{proof_seq}_{}", rec.payload_hash))?;

        let proof = LinkDeliveryProofRecord {
            schema_version: SchemaVersion(1),
            delivery_proof_ref: proof_ref.clone(),
            created_at: now,
            link_id: link_id.clone(),
            delivery_method: rec.delivery_method,
            recipient_contact_hash: rec.recipient_contact_hash.clone(),
            delivery_status,
            idempotency_key: idempotency_key.clone(),
        };

        self.link_delivery_proofs
            .insert(proof_ref.clone(), proof.clone());
        self.link_delivery_failure_idempotency_index
            .insert(idx_key, proof_ref);

        if delivery_status == DeliveryStatus::Sent {
            rec.status = LinkStatus::Sent;
        }

        Ok(proof)
    }

    pub fn attempt_overwrite_link_delivery_proof(
        &mut self,
        _delivery_proof_ref: &DeliveryProofRef,
    ) -> Result<(), StorageError> {
        Err(StorageError::AppendOnlyViolation {
            table: "link_delivery_proofs",
        })
    }

    // ------------------------
    // PH1.ONB (Onboarding) - minimal storage API for simulations.
    // ------------------------

    pub fn ph1onb_session_start_draft(
        &mut self,
        now: MonotonicTimeNs,
        link_id: LinkId,
        prefilled_context_ref: Option<PrefilledContextRef>,
        tenant_id: Option<String>,
        device_fingerprint: String,
    ) -> Result<OnbSessionStartResult, StorageError> {
        // Preconditions: link exists and is ACTIVATED.
        let link = self
            .links
            .get(&link_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "onboarding_sessions.link_id",
                key: link_id.as_str().to_string(),
            })?;
        if link.status != LinkStatus::Activated {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1onb_session_start_draft.link_status",
                    reason: "link must be ACTIVATED",
                },
            ));
        }

        if let Some(tid) = tenant_id.as_deref() {
            if tid.trim().is_empty() || tid.len() > 64 || !tid.is_ascii() {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1onb_session_start_draft.tenant_id",
                        reason: "must be non-empty ASCII and <= 64 chars when provided",
                    },
                ));
            }
        }
        if let (Some(request_tenant), Some(link_tenant)) = (
            tenant_id.as_deref(),
            link.prefilled_context
                .as_ref()
                .and_then(|ctx| ctx.tenant_id.as_deref()),
        ) {
            if request_tenant != link_tenant {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1onb_session_start_draft.tenant_id",
                        reason: "must match link prefilled tenant scope when both are present",
                    },
                ));
            }
        }

        if let Some(existing) = self.onboarding_session_by_link.get(&link_id) {
            let rec = self.onboarding_sessions.get(existing).ok_or(
                StorageError::ForeignKeyViolation {
                    table: "onboarding_sessions.onboarding_session_id",
                    key: existing.as_str().to_string(),
                },
            )?;
            if let (Some(request_tenant), Some(existing_tenant)) =
                (tenant_id.as_deref(), rec.tenant_id.as_deref())
            {
                if request_tenant != existing_tenant {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "ph1onb_session_start_draft.tenant_id",
                            reason: "must match existing onboarding session tenant scope",
                        },
                    ));
                }
            }
            let next_step = match rec.status {
                OnboardingStatus::DraftCreated => {
                    if rec.invitee_type == InviteeType::Employee && rec.tenant_id.is_none() {
                        OnboardingNextStep::AskMissing
                    } else if rec.prefilled_context_ref.is_some() {
                        OnboardingNextStep::LoadPrefilled
                    } else {
                        OnboardingNextStep::Terms
                    }
                }
                _ => OnboardingNextStep::Terms,
            };
            return Ok(OnbSessionStartResult::v1(
                rec.onboarding_session_id.clone(),
                OnboardingStatus::DraftCreated,
                next_step,
            )?);
        }

        let effective_tenant_id = tenant_id.or_else(|| {
            link.prefilled_context
                .as_ref()
                .and_then(|ctx| ctx.tenant_id.clone())
        });
        let effective_prefilled_context_ref = if prefilled_context_ref.is_some() {
            prefilled_context_ref
        } else if link.prefilled_context.is_some() {
            Some(
                PrefilledContextRef::new(format!("prefilled:{}", link.link_id.as_str()))
                    .map_err(StorageError::ContractViolation)?,
            )
        } else {
            None
        };

        let session_hash = hash_hex_64(link_id.as_str());
        let onboarding_session_id = OnboardingSessionId::new(format!("onb_{session_hash}"))?;

        let df_hash = deterministic_device_fingerprint_hash_hex(&device_fingerprint);

        let rec = OnboardingSessionRecord {
            schema_version: SchemaVersion(1),
            onboarding_session_id: onboarding_session_id.clone(),
            link_id: link_id.clone(),
            invitee_type: link.invitee_type,
            tenant_id: effective_tenant_id.clone(),
            prefilled_context_ref: effective_prefilled_context_ref.clone(),
            device_fingerprint_hash: df_hash,
            status: OnboardingStatus::DraftCreated,
            created_at: now,
            updated_at: now,
            terms_version_id: None,
            terms_status: None,
            photo_blob_ref: None,
            photo_proof_ref: None,
            sender_user_id: None,
            verification_status: None,
            primary_device_device_id: None,
            primary_device_proof_type: None,
            primary_device_confirmed: false,
            access_engine_instance_id: None,
        };

        self.onboarding_sessions
            .insert(onboarding_session_id.clone(), rec);
        self.onboarding_session_by_link
            .insert(link_id.clone(), onboarding_session_id.clone());

        let next_step =
            if link.invitee_type == InviteeType::Employee && effective_tenant_id.is_none() {
                OnboardingNextStep::AskMissing
            } else if effective_prefilled_context_ref.is_some() {
                OnboardingNextStep::LoadPrefilled
            } else {
                OnboardingNextStep::Terms
            };

        Ok(OnbSessionStartResult::v1(
            onboarding_session_id,
            OnboardingStatus::DraftCreated,
            next_step,
        )?)
    }

    fn ph1onb_validate_employee_position_prereq(
        &self,
        link_id: &LinkId,
        effective_tenant_id: Option<&str>,
    ) -> Result<(), StorageError> {
        let link = self
            .links
            .get(link_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "onboarding_sessions.link_id",
                key: link_id.as_str().to_string(),
            })?;

        let Some(prefilled) = link.prefilled_context.as_ref() else {
            return Ok(());
        };
        let Some(prefilled_position_id) = prefilled.position_id.as_ref() else {
            return Ok(());
        };

        let tenant_raw = effective_tenant_id
            .or(prefilled.tenant_id.as_deref())
            .ok_or(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1onb_employee_position_prereq.tenant_id",
                    reason: "must exist when prefilled position_id is present",
                },
            ))?;
        let company_id = prefilled
            .company_id
            .as_ref()
            .ok_or(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1onb_employee_position_prereq.company_id",
                    reason: "must exist when prefilled position_id is present",
                },
            ))?;

        let tenant_id =
            TenantId::new(tenant_raw.to_string()).map_err(StorageError::ContractViolation)?;
        let position_id = PositionId::new(prefilled_position_id.clone())
            .map_err(StorageError::ContractViolation)?;

        let company = self
            .tenant_companies
            .get(&(tenant_id.clone(), company_id.clone()))
            .ok_or(StorageError::ForeignKeyViolation {
                table: "tenant_companies.company_id",
                key: company_id.clone(),
            })?;
        if company.lifecycle_state != TenantCompanyLifecycleState::Active {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1onb_employee_position_prereq.company_state",
                    reason: "company must be ACTIVE",
                },
            ));
        }

        let position = self
            .positions
            .get(&(tenant_id.clone(), position_id.clone()))
            .ok_or(StorageError::ForeignKeyViolation {
                table: "positions.position_id",
                key: position_id.as_str().to_string(),
            })?;
        if position.company_id != *company_id {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1onb_employee_position_prereq.position_company_id",
                    reason: "position.company_id must match prefilled company_id",
                },
            ));
        }
        if position.lifecycle_state != PositionLifecycleState::Active {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1onb_employee_position_prereq.position_lifecycle_state",
                    reason: "position must be ACTIVE",
                },
            ));
        }
        if let Some(comp_tier) = prefilled.compensation_tier_ref.as_ref() {
            if position.compensation_band_ref != *comp_tier {
                return Err(StorageError::ContractViolation(ContractViolation::InvalidValue {
                    field: "ph1onb_employee_position_prereq.compensation_tier_ref",
                    reason: "prefilled compensation_tier_ref must match position compensation_band_ref",
                }));
            }
        }

        Ok(())
    }

    pub fn ph1onb_terms_accept_commit(
        &mut self,
        now: MonotonicTimeNs,
        onboarding_session_id: OnboardingSessionId,
        terms_version_id: String,
        accepted: bool,
        idempotency_key: String,
    ) -> Result<OnbTermsAcceptResult, StorageError> {
        let idx = (onboarding_session_id.clone(), idempotency_key.clone());
        if let Some(existing) = self.onb_terms_idempotency_index.get(&idx) {
            return Ok(OnbTermsAcceptResult::v1(onboarding_session_id, *existing)?);
        }

        let rec = self
            .onboarding_sessions
            .get_mut(&onboarding_session_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "onboarding_sessions.onboarding_session_id",
                key: onboarding_session_id.as_str().to_string(),
            })?;

        if rec.status != OnboardingStatus::DraftCreated {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1onb_terms_accept_commit.status",
                    reason: "session must be DRAFT_CREATED",
                },
            ));
        }

        rec.terms_version_id = Some(terms_version_id);
        let terms_status = if accepted {
            rec.terms_status = Some(TermsStatus::Accepted);
            rec.status = OnboardingStatus::TermsAccepted;
            TermsStatus::Accepted
        } else {
            rec.terms_status = Some(TermsStatus::Declined);
            rec.status = OnboardingStatus::TermsDeclined;
            TermsStatus::Declined
        };
        rec.updated_at = now;

        self.onb_terms_idempotency_index.insert(idx, terms_status);

        Ok(OnbTermsAcceptResult::v1(
            onboarding_session_id,
            terms_status,
        )?)
    }

    pub fn ph1onb_employee_photo_capture_send_commit(
        &mut self,
        now: MonotonicTimeNs,
        onboarding_session_id: OnboardingSessionId,
        photo_blob_ref: String,
        sender_user_id: UserId,
        idempotency_key: String,
    ) -> Result<OnbEmployeePhotoCaptureSendResult, StorageError> {
        let idx = (onboarding_session_id.clone(), idempotency_key.clone());
        if let Some(existing_photo_proof_ref) = self.onb_photo_idempotency_index.get(&idx) {
            return Ok(OnbEmployeePhotoCaptureSendResult::v1(
                onboarding_session_id,
                existing_photo_proof_ref.clone(),
                VerificationStatus::Pending,
            )?);
        }

        let rec = self
            .onboarding_sessions
            .get_mut(&onboarding_session_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "onboarding_sessions.onboarding_session_id",
                key: onboarding_session_id.as_str().to_string(),
            })?;

        if rec.invitee_type != InviteeType::Employee {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1onb_employee_photo_capture_send_commit.invitee_type",
                    reason: "must be Employee",
                },
            ));
        }
        if rec.terms_status != Some(TermsStatus::Accepted) {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1onb_employee_photo_capture_send_commit.terms_status",
                    reason: "must be ACCEPTED",
                },
            ));
        }

        // Store references only (no raw blobs).
        rec.photo_blob_ref = Some(photo_blob_ref);
        rec.sender_user_id = Some(sender_user_id);
        let proof_ref = format!(
            "photo_proof_{}",
            hash_hex_64(&format!("{}:{}", idx.0.as_str(), idx.1))
        );
        rec.photo_proof_ref = Some(proof_ref.clone());
        rec.verification_status = Some(VerificationStatus::Pending);
        rec.status = OnboardingStatus::VerificationPending;
        rec.updated_at = now;

        self.onb_photo_idempotency_index
            .insert(idx, proof_ref.clone());

        Ok(OnbEmployeePhotoCaptureSendResult::v1(
            onboarding_session_id,
            proof_ref,
            VerificationStatus::Pending,
        )?)
    }

    pub fn ph1onb_employee_sender_verify_commit(
        &mut self,
        now: MonotonicTimeNs,
        onboarding_session_id: OnboardingSessionId,
        sender_user_id: UserId,
        decision: SenderVerifyDecision,
        idempotency_key: String,
    ) -> Result<OnbEmployeeSenderVerifyResult, StorageError> {
        let idx = (onboarding_session_id.clone(), idempotency_key.clone());
        if let Some(existing_status) = self.onb_sender_verify_idempotency_index.get(&idx) {
            return Ok(OnbEmployeeSenderVerifyResult::v1(
                onboarding_session_id,
                *existing_status,
            )?);
        }

        let rec = self
            .onboarding_sessions
            .get_mut(&onboarding_session_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "onboarding_sessions.onboarding_session_id",
                key: onboarding_session_id.as_str().to_string(),
            })?;

        if rec.invitee_type != InviteeType::Employee {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1onb_employee_sender_verify_commit.invitee_type",
                    reason: "must be Employee",
                },
            ));
        }
        if rec.verification_status != Some(VerificationStatus::Pending) {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1onb_employee_sender_verify_commit.verification_status",
                    reason: "must be PENDING",
                },
            ));
        }
        if rec.sender_user_id.as_ref() != Some(&sender_user_id) {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1onb_employee_sender_verify_commit.sender_user_id",
                    reason: "must match the sender_user_id recorded at photo capture",
                },
            ));
        }

        let next = match decision {
            SenderVerifyDecision::Confirm => {
                rec.verification_status = Some(VerificationStatus::Confirmed);
                rec.status = OnboardingStatus::VerificationConfirmed;
                VerificationStatus::Confirmed
            }
            SenderVerifyDecision::Reject => {
                rec.verification_status = Some(VerificationStatus::Rejected);
                rec.status = OnboardingStatus::VerificationRejected;
                VerificationStatus::Rejected
            }
        };
        rec.updated_at = now;

        self.onb_sender_verify_idempotency_index.insert(idx, next);

        Ok(OnbEmployeeSenderVerifyResult::v1(
            onboarding_session_id,
            next,
        )?)
    }

    pub fn ph1onb_primary_device_confirm_commit(
        &mut self,
        now: MonotonicTimeNs,
        onboarding_session_id: OnboardingSessionId,
        device_id: DeviceId,
        proof_type: ProofType,
        proof_ok: bool,
        idempotency_key: String,
    ) -> Result<OnbPrimaryDeviceConfirmResult, StorageError> {
        let idx = (onboarding_session_id.clone(), idempotency_key.clone());
        if let Some(existing_ok) = self.onb_primary_device_idempotency_index.get(&idx) {
            return Ok(OnbPrimaryDeviceConfirmResult::v1(
                onboarding_session_id,
                *existing_ok,
            )?);
        }

        let rec = self
            .onboarding_sessions
            .get_mut(&onboarding_session_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "onboarding_sessions.onboarding_session_id",
                key: onboarding_session_id.as_str().to_string(),
            })?;

        if rec.terms_status != Some(TermsStatus::Accepted) {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1onb_primary_device_confirm_commit.terms_status",
                    reason: "must be ACCEPTED",
                },
            ));
        }

        rec.primary_device_device_id = Some(device_id);
        rec.primary_device_proof_type = Some(proof_type);
        rec.primary_device_confirmed = proof_ok;
        if proof_ok {
            rec.status = OnboardingStatus::PrimaryDeviceConfirmed;
        }
        rec.updated_at = now;

        self.onb_primary_device_idempotency_index
            .insert(idx, proof_ok);

        Ok(OnbPrimaryDeviceConfirmResult::v1(
            onboarding_session_id,
            proof_ok,
        )?)
    }

    pub fn ph1onb_access_instance_create_commit(
        &mut self,
        now: MonotonicTimeNs,
        onboarding_session_id: OnboardingSessionId,
        user_id: UserId,
        tenant_id: Option<String>,
        role_id: String,
        idempotency_key: String,
    ) -> Result<OnbAccessInstanceCreateResult, StorageError> {
        let idx = (user_id.clone(), role_id.clone(), idempotency_key.clone());
        if let Some(existing_instance_id) = self.onb_access_instance_idempotency_index.get(&idx) {
            return Ok(OnbAccessInstanceCreateResult::v1(
                onboarding_session_id,
                existing_instance_id.clone(),
            )?);
        }

        let (
            invitee_type,
            verification_status,
            primary_device_confirmed,
            terms_status,
            link_id,
            existing_tenant_id,
        ) = {
            let rec = self.onboarding_sessions.get(&onboarding_session_id).ok_or(
                StorageError::ForeignKeyViolation {
                    table: "onboarding_sessions.onboarding_session_id",
                    key: onboarding_session_id.as_str().to_string(),
                },
            )?;
            (
                rec.invitee_type,
                rec.verification_status,
                rec.primary_device_confirmed,
                rec.terms_status,
                rec.link_id.clone(),
                rec.tenant_id.clone(),
            )
        };

        if terms_status != Some(TermsStatus::Accepted) {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1onb_access_instance_create_commit.terms_status",
                    reason: "must be ACCEPTED",
                },
            ));
        }
        if !primary_device_confirmed {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1onb_access_instance_create_commit.primary_device_confirmed",
                    reason: "must be true before creating access instance",
                },
            ));
        }
        if invitee_type == InviteeType::Employee
            && verification_status != Some(VerificationStatus::Confirmed)
        {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1onb_access_instance_create_commit.verification_status",
                    reason: "employee must be VERIFIED before creating access instance",
                },
            ));
        }

        let effective_tenant_id = tenant_id.clone().or(existing_tenant_id);
        if invitee_type == InviteeType::Employee {
            self.ph1onb_validate_employee_position_prereq(
                &link_id,
                effective_tenant_id.as_deref(),
            )?;
        }

        let resolved_tenant_id = effective_tenant_id
            .unwrap_or_else(|| format!("personal_{}", hash_hex_64(user_id.as_str())));

        // Onboarding may reach access-instance creation before identity rows are materialized
        // elsewhere. Ensure the per-user identity row exists for PH2 FK discipline.
        if !self.identities.contains_key(&user_id) {
            self.insert_identity(IdentityRecord::v1(
                user_id.clone(),
                None,
                None,
                now,
                IdentityStatus::Active,
            ))?;
        }

        // Persist per-user access truth through PH2.ACCESS.002 storage wiring.
        let access_row = self.ph2access_upsert_instance_commit(
            now,
            resolved_tenant_id.clone(),
            user_id.clone(),
            role_id.clone(),
            role_to_default_access_mode(&role_id),
            "{\"financial_auth\":false}".to_string(),
            true,
            AccessVerificationLevel::PasscodeTime,
            AccessDeviceTrustLevel::Dtl3,
            AccessLifecycleState::Active,
            format!("role_template:{role_id}"),
            Some(idempotency_key.clone()),
        )?;
        let inst_id = access_row.access_instance_id.clone();

        let rec = self
            .onboarding_sessions
            .get_mut(&onboarding_session_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "onboarding_sessions.onboarding_session_id",
                key: onboarding_session_id.as_str().to_string(),
            })?;

        rec.tenant_id = Some(resolved_tenant_id);
        rec.access_engine_instance_id = Some(inst_id.clone());
        rec.status = OnboardingStatus::AccessInstanceCreated;
        rec.updated_at = now;

        self.onb_access_instance_idempotency_index
            .insert(idx, inst_id.clone());

        Ok(OnbAccessInstanceCreateResult::v1(
            onboarding_session_id,
            inst_id,
        )?)
    }

    pub fn ph1onb_complete_commit(
        &mut self,
        now: MonotonicTimeNs,
        onboarding_session_id: OnboardingSessionId,
        idempotency_key: String,
    ) -> Result<OnbCompleteResult, StorageError> {
        let idx = (onboarding_session_id.clone(), idempotency_key.clone());
        if let Some(existing_status) = self.onb_complete_idempotency_index.get(&idx) {
            return Ok(OnbCompleteResult::v1(
                onboarding_session_id,
                *existing_status,
            )?);
        }

        let (
            invitee_type,
            verification_status,
            terms_status,
            access_engine_instance_id,
            link_id,
            tenant_id,
        ) = {
            let rec = self.onboarding_sessions.get(&onboarding_session_id).ok_or(
                StorageError::ForeignKeyViolation {
                    table: "onboarding_sessions.onboarding_session_id",
                    key: onboarding_session_id.as_str().to_string(),
                },
            )?;
            (
                rec.invitee_type,
                rec.verification_status,
                rec.terms_status,
                rec.access_engine_instance_id.clone(),
                rec.link_id.clone(),
                rec.tenant_id.clone(),
            )
        };

        if terms_status != Some(TermsStatus::Accepted) {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1onb_complete_commit.terms_status",
                    reason: "must be ACCEPTED",
                },
            ));
        }
        if invitee_type == InviteeType::Employee
            && verification_status != Some(VerificationStatus::Confirmed)
        {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1onb_complete_commit.verification_status",
                    reason: "employee must be VERIFIED before completing onboarding",
                },
            ));
        }
        if access_engine_instance_id.is_none() {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1onb_complete_commit.access_engine_instance_id",
                    reason: "must exist before completing onboarding",
                },
            ));
        }
        if invitee_type == InviteeType::Employee {
            self.ph1onb_validate_employee_position_prereq(&link_id, tenant_id.as_deref())?;
        }

        let rec = self
            .onboarding_sessions
            .get_mut(&onboarding_session_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "onboarding_sessions.onboarding_session_id",
                key: onboarding_session_id.as_str().to_string(),
            })?;

        rec.status = OnboardingStatus::Complete;
        rec.updated_at = now;

        self.onb_complete_idempotency_index
            .insert(idx, OnboardingStatus::Complete);

        Ok(OnbCompleteResult::v1(
            onboarding_session_id,
            OnboardingStatus::Complete,
        )?)
    }

    pub fn ph1onb_session_row(
        &self,
        onboarding_session_id: &OnboardingSessionId,
    ) -> Option<&OnboardingSessionRecord> {
        self.onboarding_sessions.get(onboarding_session_id)
    }

    pub fn ph1onb_session_rows(&self) -> &BTreeMap<OnboardingSessionId, OnboardingSessionRecord> {
        &self.onboarding_sessions
    }

    // ------------------------
    // PH1.W (Wake) - minimal storage API for wake simulations and runtime records.
    // ------------------------

    #[allow(clippy::too_many_arguments)]
    pub fn ph1w_enroll_start_draft(
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
        if !self.identities.contains_key(&user_id) {
            return Err(StorageError::ForeignKeyViolation {
                table: "wake_enrollment_sessions.user_id",
                key: user_id.as_str().to_string(),
            });
        }

        let dev = self
            .devices
            .get(&device_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "wake_enrollment_sessions.device_id",
                key: device_id.as_str().to_string(),
            })?;
        if dev.user_id != user_id {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1w_enroll_start_draft.device_id",
                    reason: "device must belong to user_id",
                },
            ));
        }

        if !(3..=8).contains(&pass_target) {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1w_enroll_start_draft.pass_target",
                    reason: "must be in [3, 8]",
                },
            ));
        }
        if !(8..=20).contains(&max_attempts) {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1w_enroll_start_draft.max_attempts",
                    reason: "must be in [8, 20]",
                },
            ));
        }
        if !(180_000..=600_000).contains(&enrollment_timeout_ms) {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1w_enroll_start_draft.enrollment_timeout_ms",
                    reason: "must be in [180000, 600000]",
                },
            ));
        }
        if idempotency_key.trim().is_empty() || idempotency_key.len() > 128 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1w_enroll_start_draft.idempotency_key",
                    reason: "must be non-empty and <= 128 chars",
                },
            ));
        }
        if let Some(onboarding_sid) = onboarding_session_id.as_ref() {
            if !self.onboarding_sessions.contains_key(onboarding_sid) {
                return Err(StorageError::ForeignKeyViolation {
                    table: "wake_enrollment_sessions.onboarding_session_id",
                    key: onboarding_sid.as_str().to_string(),
                });
            }
        }

        let idem = (user_id.clone(), device_id.clone(), idempotency_key.clone());
        if let Some(existing_id) = self.wake_start_idempotency_index.get(&idem) {
            return self
                .wake_enrollment_sessions
                .get(existing_id)
                .cloned()
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "wake_enrollment_sessions.wake_enrollment_session_id",
                    key: existing_id.clone(),
                });
        }
        if self.wake_enrollment_sessions.values().any(|row| {
            row.user_id == user_id
                && row.device_id == device_id
                && row.wake_enroll_status == WakeEnrollStatus::InProgress
        }) {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "wake_enrollment_sessions.wake_enroll_status",
                    reason: "only one IN_PROGRESS session is allowed per user/device",
                },
            ));
        }

        let sid_hash = hash_hex_64(&format!(
            "{}:{}:{}:{}",
            user_id.as_str(),
            device_id.as_str(),
            now.0,
            idempotency_key
        ));
        let wake_enrollment_session_id = format!("wake_enr_{sid_hash}");

        let rec = WakeEnrollmentSessionRecord {
            schema_version: SchemaVersion(1),
            wake_enrollment_session_id: wake_enrollment_session_id.clone(),
            user_id,
            device_id,
            onboarding_session_id,
            wake_profile_id: None,
            wake_enroll_status: WakeEnrollStatus::InProgress,
            pass_target,
            pass_count: 0,
            attempt_count: 0,
            max_attempts,
            enrollment_timeout_ms,
            reason_code: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
            deferred_until: None,
        };

        self.wake_enrollment_sessions
            .insert(wake_enrollment_session_id.clone(), rec.clone());
        self.wake_start_idempotency_index
            .insert(idem, wake_enrollment_session_id);

        Ok(rec)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1w_enroll_sample_commit(
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
        if wake_enrollment_session_id.trim().is_empty() || wake_enrollment_session_id.len() > 64 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1w_enroll_sample_commit.wake_enrollment_session_id",
                    reason: "must be non-empty and <= 64 chars",
                },
            ));
        }
        if idempotency_key.trim().is_empty() || idempotency_key.len() > 128 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1w_enroll_sample_commit.idempotency_key",
                    reason: "must be non-empty and <= 128 chars",
                },
            ));
        }
        if !(500..=2200).contains(&sample_duration_ms) {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1w_enroll_sample_commit.sample_duration_ms",
                    reason: "must be in [500, 2200]",
                },
            ));
        }

        let idx = (wake_enrollment_session_id.clone(), idempotency_key.clone());
        if self.wake_sample_idempotency_index.contains_key(&idx) {
            return self
                .wake_enrollment_sessions
                .get(&wake_enrollment_session_id)
                .cloned()
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "wake_enrollment_sessions.wake_enrollment_session_id",
                    key: wake_enrollment_session_id,
                });
        }

        let rec = self
            .wake_enrollment_sessions
            .get_mut(&wake_enrollment_session_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "wake_enrollment_sessions.wake_enrollment_session_id",
                key: wake_enrollment_session_id.clone(),
            })?;

        if matches!(
            rec.wake_enroll_status,
            WakeEnrollStatus::Complete | WakeEnrollStatus::Declined
        ) {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1w_enroll_sample_commit.wake_enroll_status",
                    reason: "cannot append samples to COMPLETE/DECLINED sessions",
                },
            ));
        }

        let elapsed_ns = now.0.saturating_sub(rec.created_at.0);
        let timed_out = elapsed_ns >= ms_to_ns(rec.enrollment_timeout_ms);

        rec.attempt_count = rec.attempt_count.saturating_add(1);
        if result == WakeSampleResult::Pass {
            rec.pass_count = rec.pass_count.saturating_add(1);
        }

        if rec.pass_count >= rec.pass_target {
            rec.wake_enroll_status = WakeEnrollStatus::Complete;
            rec.reason_code = None;
            rec.completed_at = Some(now);
            rec.deferred_until = None;
        } else if timed_out {
            rec.wake_enroll_status = WakeEnrollStatus::Pending;
            rec.reason_code = Some(reason_code.unwrap_or(W_ENROLL_REASON_TIMEOUT));
            rec.completed_at = None;
        } else if rec.attempt_count >= rec.max_attempts {
            rec.wake_enroll_status = WakeEnrollStatus::Pending;
            rec.reason_code = Some(reason_code.unwrap_or(W_ENROLL_REASON_MAX_ATTEMPTS));
            rec.completed_at = None;
        } else {
            rec.wake_enroll_status = WakeEnrollStatus::InProgress;
        }
        rec.updated_at = now;

        let sample_seq = rec.attempt_count as u16;
        self.wake_enrollment_samples
            .push(WakeEnrollmentSampleRecord {
                schema_version: SchemaVersion(1),
                wake_enrollment_session_id: wake_enrollment_session_id.clone(),
                sample_seq,
                captured_at: now,
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
                idempotency_key: idempotency_key.clone(),
            });
        self.wake_sample_idempotency_index.insert(idx, sample_seq);

        Ok(rec.clone())
    }

    pub fn ph1w_enroll_complete_commit(
        &mut self,
        now: MonotonicTimeNs,
        wake_enrollment_session_id: String,
        wake_profile_id: String,
        idempotency_key: String,
    ) -> Result<WakeEnrollmentSessionRecord, StorageError> {
        if wake_profile_id.trim().is_empty() || wake_profile_id.len() > 128 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1w_enroll_complete_commit.wake_profile_id",
                    reason: "must be non-empty and <= 128 chars",
                },
            ));
        }
        if idempotency_key.trim().is_empty() || idempotency_key.len() > 128 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1w_enroll_complete_commit.idempotency_key",
                    reason: "must be non-empty and <= 128 chars",
                },
            ));
        }

        let idx = (wake_enrollment_session_id.clone(), idempotency_key.clone());
        if self.wake_complete_idempotency_index.contains_key(&idx) {
            return self
                .wake_enrollment_sessions
                .get(&wake_enrollment_session_id)
                .cloned()
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "wake_enrollment_sessions.wake_enrollment_session_id",
                    key: wake_enrollment_session_id,
                });
        }

        let rec = self
            .wake_enrollment_sessions
            .get_mut(&wake_enrollment_session_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "wake_enrollment_sessions.wake_enrollment_session_id",
                key: wake_enrollment_session_id.clone(),
            })?;

        if rec.pass_count < rec.pass_target {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1w_enroll_complete_commit.pass_count",
                    reason: "pass_count must be >= pass_target",
                },
            ));
        }

        rec.wake_profile_id = Some(wake_profile_id.clone());
        rec.wake_enroll_status = WakeEnrollStatus::Complete;
        rec.reason_code = None;
        rec.completed_at = Some(now);
        rec.updated_at = now;
        rec.deferred_until = None;

        self.wake_profile_bindings.insert(
            (rec.user_id.clone(), rec.device_id.clone()),
            wake_profile_id.clone(),
        );
        self.wake_complete_idempotency_index
            .insert(idx, wake_profile_id);

        Ok(rec.clone())
    }

    pub fn ph1w_enroll_defer_reminder_commit(
        &mut self,
        now: MonotonicTimeNs,
        wake_enrollment_session_id: String,
        deferred_until: Option<MonotonicTimeNs>,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<WakeEnrollmentSessionRecord, StorageError> {
        if idempotency_key.trim().is_empty() || idempotency_key.len() > 128 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1w_enroll_defer_reminder_commit.idempotency_key",
                    reason: "must be non-empty and <= 128 chars",
                },
            ));
        }

        let idx = (wake_enrollment_session_id.clone(), idempotency_key.clone());
        if self.wake_defer_idempotency_index.contains_key(&idx) {
            return self
                .wake_enrollment_sessions
                .get(&wake_enrollment_session_id)
                .cloned()
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "wake_enrollment_sessions.wake_enrollment_session_id",
                    key: wake_enrollment_session_id,
                });
        }

        let rec = self
            .wake_enrollment_sessions
            .get_mut(&wake_enrollment_session_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "wake_enrollment_sessions.wake_enrollment_session_id",
                key: wake_enrollment_session_id.clone(),
            })?;

        rec.wake_enroll_status = WakeEnrollStatus::Pending;
        rec.reason_code = Some(reason_code);
        rec.deferred_until = deferred_until;
        rec.updated_at = now;

        self.wake_defer_idempotency_index
            .insert(idx, WakeEnrollStatus::Pending);
        Ok(rec.clone())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1w_runtime_event_commit(
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
        if wake_event_id.trim().is_empty() || wake_event_id.len() > 128 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1w_runtime_event_commit.wake_event_id",
                    reason: "must be non-empty and <= 128 chars",
                },
            ));
        }
        if idempotency_key.trim().is_empty() || idempotency_key.len() > 128 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1w_runtime_event_commit.idempotency_key",
                    reason: "must be non-empty and <= 128 chars",
                },
            ));
        }
        if !self.devices.contains_key(&device_id) {
            return Err(StorageError::ForeignKeyViolation {
                table: "wake_runtime_events.device_id",
                key: device_id.as_str().to_string(),
            });
        }
        if let Some(uid) = &user_id {
            if !self.identities.contains_key(uid) {
                return Err(StorageError::ForeignKeyViolation {
                    table: "wake_runtime_events.user_id",
                    key: uid.as_str().to_string(),
                });
            }
        }
        if let Some(sid) = session_id {
            if !self.sessions.contains_key(&sid) {
                return Err(StorageError::ForeignKeyViolation {
                    table: "wake_runtime_events.session_id",
                    key: sid.0.to_string(),
                });
            }
        }

        let idx = (device_id.clone(), idempotency_key.clone());
        if let Some(existing_event_id) = self.wake_runtime_event_idempotency_index.get(&idx) {
            if let Some(existing) = self
                .wake_runtime_events
                .iter()
                .find(|e| &e.wake_event_id == existing_event_id)
            {
                return Ok(existing.clone());
            }
        }

        let rec = WakeRuntimeEventRecord {
            schema_version: SchemaVersion(1),
            wake_event_id: wake_event_id.clone(),
            created_at: now,
            session_id,
            user_id,
            device_id: device_id.clone(),
            accepted,
            reason_code,
            wake_profile_id,
            tts_active_at_trigger,
            media_playback_active_at_trigger,
            suppression_reason_code,
            idempotency_key: idempotency_key.clone(),
        };

        self.wake_runtime_events.push(rec.clone());
        self.wake_runtime_event_idempotency_index
            .insert(idx, wake_event_id);
        Ok(rec)
    }

    pub fn ph1w_get_enrollment_session(
        &self,
        wake_enrollment_session_id: &str,
    ) -> Option<&WakeEnrollmentSessionRecord> {
        self.wake_enrollment_sessions
            .get(wake_enrollment_session_id)
    }

    pub fn ph1w_get_samples_for_session(
        &self,
        wake_enrollment_session_id: &str,
    ) -> Vec<&WakeEnrollmentSampleRecord> {
        self.wake_enrollment_samples
            .iter()
            .filter(|row| row.wake_enrollment_session_id == wake_enrollment_session_id)
            .collect()
    }

    pub fn ph1w_get_runtime_events(&self) -> &[WakeRuntimeEventRecord] {
        &self.wake_runtime_events
    }

    pub fn attempt_overwrite_wake_enrollment_sample(
        &mut self,
        _wake_enrollment_session_id: &str,
        _sample_seq: u16,
    ) -> Result<(), StorageError> {
        Err(StorageError::AppendOnlyViolation {
            table: "wake_enrollment_samples",
        })
    }

    pub fn attempt_overwrite_wake_runtime_event(
        &mut self,
        _wake_event_id: &str,
    ) -> Result<(), StorageError> {
        Err(StorageError::AppendOnlyViolation {
            table: "wake_runtime_events",
        })
    }

    pub fn ph1w_get_active_wake_profile(
        &self,
        user_id: &UserId,
        device_id: &DeviceId,
    ) -> Option<&str> {
        self.wake_profile_bindings
            .get(&(user_id.clone(), device_id.clone()))
            .map(|v| v.as_str())
    }

    // ------------------------
    // PH1.VOICE.ID (voice enrollment) - storage API for simulation-backed enrollment.
    // ------------------------

    pub fn ph1vid_enroll_start_draft(
        &mut self,
        now: MonotonicTimeNs,
        onboarding_session_id: OnboardingSessionId,
        device_id: DeviceId,
        consent_asserted: bool,
        max_total_attempts: u8,
        max_session_enroll_time_ms: u32,
        lock_after_consecutive_passes: u8,
    ) -> Result<VoiceEnrollmentSessionRecord, StorageError> {
        if !self
            .onboarding_sessions
            .contains_key(&onboarding_session_id)
        {
            return Err(StorageError::ForeignKeyViolation {
                table: "voice_enrollment_sessions.onboarding_session_id",
                key: onboarding_session_id.as_str().to_string(),
            });
        }
        if !self.devices.contains_key(&device_id) {
            return Err(StorageError::ForeignKeyViolation {
                table: "voice_enrollment_sessions.device_id",
                key: device_id.as_str().to_string(),
            });
        }
        if !consent_asserted {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1vid_enroll_start_draft.consent_asserted",
                    reason: "must be true",
                },
            ));
        }
        if !(5..=20).contains(&max_total_attempts) {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1vid_enroll_start_draft.max_total_attempts",
                    reason: "must be in [5, 20]",
                },
            ));
        }
        if !(60_000..=300_000).contains(&max_session_enroll_time_ms) {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1vid_enroll_start_draft.max_session_enroll_time_ms",
                    reason: "must be in [60000, 300000]",
                },
            ));
        }
        if !(2..=5).contains(&lock_after_consecutive_passes) {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1vid_enroll_start_draft.lock_after_consecutive_passes",
                    reason: "must be in [2, 5]",
                },
            ));
        }

        let idx = (onboarding_session_id.clone(), device_id.clone());
        if let Some(existing_id) = self.voice_start_idempotency_index.get(&idx) {
            return self
                .voice_enrollment_sessions
                .get(existing_id)
                .cloned()
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "voice_enrollment_sessions.voice_enrollment_session_id",
                    key: existing_id.clone(),
                });
        }

        let sid_hash = hash_hex_64(&format!(
            "{}:{}:{}",
            onboarding_session_id.as_str(),
            device_id.as_str(),
            now.0
        ));
        let voice_enrollment_session_id = format!("voice_enr_{sid_hash}");

        let rec = VoiceEnrollmentSessionRecord {
            schema_version: SchemaVersion(1),
            voice_enrollment_session_id: voice_enrollment_session_id.clone(),
            onboarding_session_id: onboarding_session_id.clone(),
            device_id: device_id.clone(),
            voice_profile_id: None,
            voice_enroll_status: VoiceEnrollStatus::InProgress,
            lock_after_consecutive_passes,
            consecutive_passes: 0,
            attempt_count: 0,
            max_total_attempts,
            max_session_enroll_time_ms,
            created_at: now,
            updated_at: now,
            deferred_until: None,
            reason_code: None,
        };

        self.voice_enrollment_sessions
            .insert(voice_enrollment_session_id.clone(), rec.clone());
        self.voice_start_idempotency_index
            .insert(idx, voice_enrollment_session_id);
        Ok(rec)
    }

    pub fn ph1vid_enroll_sample_commit(
        &mut self,
        now: MonotonicTimeNs,
        voice_enrollment_session_id: String,
        audio_sample_ref: String,
        attempt_index: u16,
        result: VoiceSampleResult,
        reason_code: Option<ReasonCodeId>,
        idempotency_key: String,
    ) -> Result<VoiceEnrollmentSessionRecord, StorageError> {
        if voice_enrollment_session_id.trim().is_empty() || voice_enrollment_session_id.len() > 64 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1vid_enroll_sample_commit.voice_enrollment_session_id",
                    reason: "must be non-empty and <= 64 chars",
                },
            ));
        }
        if audio_sample_ref.trim().is_empty() || audio_sample_ref.len() > 256 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1vid_enroll_sample_commit.audio_sample_ref",
                    reason: "must be non-empty and <= 256 chars",
                },
            ));
        }
        if attempt_index == 0 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1vid_enroll_sample_commit.attempt_index",
                    reason: "must be > 0",
                },
            ));
        }
        if idempotency_key.trim().is_empty() || idempotency_key.len() > 128 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1vid_enroll_sample_commit.idempotency_key",
                    reason: "must be non-empty and <= 128 chars",
                },
            ));
        }
        if result == VoiceSampleResult::Fail && reason_code.is_none() {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1vid_enroll_sample_commit.reason_code",
                    reason: "required on FAIL",
                },
            ));
        }

        let idx = (
            voice_enrollment_session_id.clone(),
            attempt_index,
            idempotency_key.clone(),
        );
        if self.voice_sample_idempotency_index.contains_key(&idx) {
            return self
                .voice_enrollment_sessions
                .get(&voice_enrollment_session_id)
                .cloned()
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "voice_enrollment_sessions.voice_enrollment_session_id",
                    key: voice_enrollment_session_id,
                });
        }

        let rec_clone = {
            let rec = self
                .voice_enrollment_sessions
                .get_mut(&voice_enrollment_session_id)
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "voice_enrollment_sessions.voice_enrollment_session_id",
                    key: voice_enrollment_session_id.clone(),
                })?;

            if matches!(
                rec.voice_enroll_status,
                VoiceEnrollStatus::Pending | VoiceEnrollStatus::Declined
            ) {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1vid_enroll_sample_commit.voice_enroll_status",
                        reason: "cannot append samples to PENDING/DECLINED sessions",
                    },
                ));
            }
            if rec.voice_enroll_status == VoiceEnrollStatus::Locked {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1vid_enroll_sample_commit.voice_enroll_status",
                        reason: "session already LOCKED",
                    },
                ));
            }

            let elapsed_ns = now.0.saturating_sub(rec.created_at.0);
            let timed_out = elapsed_ns >= ms_to_ns(rec.max_session_enroll_time_ms);

            rec.attempt_count = rec.attempt_count.saturating_add(1);
            if result == VoiceSampleResult::Pass {
                rec.consecutive_passes = rec.consecutive_passes.saturating_add(1);
            } else {
                rec.consecutive_passes = 0;
            }

            if rec.consecutive_passes >= rec.lock_after_consecutive_passes {
                rec.voice_enroll_status = VoiceEnrollStatus::Locked;
                rec.reason_code = None;
                rec.deferred_until = None;
            } else if timed_out {
                rec.voice_enroll_status = VoiceEnrollStatus::Pending;
                rec.reason_code = Some(reason_code.unwrap_or(VID_ENROLL_REASON_TIMEOUT));
            } else if rec.attempt_count >= rec.max_total_attempts {
                rec.voice_enroll_status = VoiceEnrollStatus::Pending;
                rec.reason_code = Some(reason_code.unwrap_or(VID_ENROLL_REASON_MAX_ATTEMPTS));
            } else {
                rec.voice_enroll_status = VoiceEnrollStatus::InProgress;
                rec.reason_code = reason_code;
            }
            rec.updated_at = now;
            rec.clone()
        };

        let sample_seq = self.next_voice_enrollment_sample_seq;
        self.next_voice_enrollment_sample_seq =
            self.next_voice_enrollment_sample_seq.saturating_add(1);

        self.voice_enrollment_samples
            .push(VoiceEnrollmentSampleRecord {
                schema_version: SchemaVersion(1),
                sample_seq,
                voice_enrollment_session_id: voice_enrollment_session_id.clone(),
                created_at: now,
                attempt_index,
                audio_sample_ref,
                result,
                reason_code,
                idempotency_key: idempotency_key.clone(),
            });
        self.voice_sample_idempotency_index.insert(idx, sample_seq);

        Ok(rec_clone)
    }

    pub fn ph1vid_enroll_complete_commit(
        &mut self,
        now: MonotonicTimeNs,
        voice_enrollment_session_id: String,
        idempotency_key: String,
    ) -> Result<VoiceEnrollmentSessionRecord, StorageError> {
        if idempotency_key.trim().is_empty() || idempotency_key.len() > 128 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1vid_enroll_complete_commit.idempotency_key",
                    reason: "must be non-empty and <= 128 chars",
                },
            ));
        }
        let idx = (voice_enrollment_session_id.clone(), idempotency_key.clone());
        if self.voice_complete_idempotency_index.contains_key(&idx) {
            return self
                .voice_enrollment_sessions
                .get(&voice_enrollment_session_id)
                .cloned()
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "voice_enrollment_sessions.voice_enrollment_session_id",
                    key: voice_enrollment_session_id,
                });
        }

        let rec_clone = {
            let rec = self
                .voice_enrollment_sessions
                .get_mut(&voice_enrollment_session_id)
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "voice_enrollment_sessions.voice_enrollment_session_id",
                    key: voice_enrollment_session_id.clone(),
                })?;

            if rec.voice_enroll_status != VoiceEnrollStatus::Locked {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1vid_enroll_complete_commit.voice_enroll_status",
                        reason: "must be LOCKED before complete",
                    },
                ));
            }

            if rec.voice_profile_id.is_none() {
                let profile_hash = hash_hex_64(&format!(
                    "{}:{}:{}",
                    rec.voice_enrollment_session_id,
                    rec.onboarding_session_id.as_str(),
                    rec.device_id.as_str()
                ));
                rec.voice_profile_id = Some(format!("voice_prof_{profile_hash}"));
            }
            rec.updated_at = now;
            rec.reason_code = None;
            rec.clone()
        };

        let voice_profile_id =
            rec_clone
                .voice_profile_id
                .clone()
                .ok_or(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1vid_enroll_complete_commit.voice_profile_id",
                        reason: "must be present after completion",
                    },
                ))?;

        self.voice_profiles
            .entry(voice_profile_id.clone())
            .or_insert(VoiceProfileRecord {
                schema_version: SchemaVersion(1),
                voice_profile_id: voice_profile_id.clone(),
                onboarding_session_id: rec_clone.onboarding_session_id.clone(),
                device_id: rec_clone.device_id.clone(),
                created_at: now,
            });
        self.voice_profile_bindings.insert(
            (
                rec_clone.onboarding_session_id.clone(),
                rec_clone.device_id.clone(),
            ),
            voice_profile_id.clone(),
        );
        self.voice_complete_idempotency_index
            .insert(idx, voice_profile_id);
        Ok(rec_clone)
    }

    pub fn ph1vid_enroll_defer_reminder_commit(
        &mut self,
        now: MonotonicTimeNs,
        voice_enrollment_session_id: String,
        reason_code: ReasonCodeId,
        idempotency_key: String,
    ) -> Result<VoiceEnrollmentSessionRecord, StorageError> {
        if idempotency_key.trim().is_empty() || idempotency_key.len() > 128 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1vid_enroll_defer_reminder_commit.idempotency_key",
                    reason: "must be non-empty and <= 128 chars",
                },
            ));
        }

        let idx = (voice_enrollment_session_id.clone(), idempotency_key);
        if self.voice_defer_idempotency_index.contains_key(&idx) {
            return self
                .voice_enrollment_sessions
                .get(&voice_enrollment_session_id)
                .cloned()
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "voice_enrollment_sessions.voice_enrollment_session_id",
                    key: voice_enrollment_session_id,
                });
        }

        let rec = self
            .voice_enrollment_sessions
            .get_mut(&voice_enrollment_session_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "voice_enrollment_sessions.voice_enrollment_session_id",
                key: voice_enrollment_session_id.clone(),
            })?;

        rec.voice_enroll_status = VoiceEnrollStatus::Pending;
        rec.reason_code = Some(reason_code);
        rec.updated_at = now;
        rec.deferred_until = None;

        self.voice_defer_idempotency_index
            .insert(idx, VoiceEnrollStatus::Pending);
        Ok(rec.clone())
    }

    pub fn ph1vid_get_enrollment_session(
        &self,
        voice_enrollment_session_id: &str,
    ) -> Option<&VoiceEnrollmentSessionRecord> {
        self.voice_enrollment_sessions
            .get(voice_enrollment_session_id)
    }

    pub fn ph1vid_get_samples_for_session(
        &self,
        voice_enrollment_session_id: &str,
    ) -> Vec<&VoiceEnrollmentSampleRecord> {
        self.voice_enrollment_samples
            .iter()
            .filter(|row| row.voice_enrollment_session_id == voice_enrollment_session_id)
            .collect()
    }

    pub fn attempt_overwrite_voice_enrollment_sample(
        &mut self,
        _voice_enrollment_session_id: &str,
        _sample_seq: u16,
    ) -> Result<(), StorageError> {
        Err(StorageError::AppendOnlyViolation {
            table: "voice_enrollment_samples",
        })
    }

    pub fn ph1vid_get_voice_profile(&self, voice_profile_id: &str) -> Option<&VoiceProfileRecord> {
        self.voice_profiles.get(voice_profile_id)
    }

    // ------------------------
    // PH1.ACCESS.001 + PH2.ACCESS.002 (Access/Authority)
    // ------------------------

    fn has_financial_authorization(baseline_permissions_json: &str) -> bool {
        let compact: String = baseline_permissions_json
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect();
        compact.contains("\"financial_auth\":true")
    }

    fn parse_grant_mode(scope_json: &str) -> Option<AccessMode> {
        let scope = scope_json.to_ascii_lowercase();
        if scope.contains("\"grant_mode\":\"x\"") {
            Some(AccessMode::X)
        } else if scope.contains("\"grant_mode\":\"a\"") {
            Some(AccessMode::A)
        } else if scope.contains("\"grant_mode\":\"w\"") {
            Some(AccessMode::W)
        } else if scope.contains("\"grant_mode\":\"r\"") {
            Some(AccessMode::R)
        } else {
            None
        }
    }

    fn validate_access_tenant_id(tenant_id: &str) -> Result<(), StorageError> {
        if tenant_id.trim().is_empty() || tenant_id.len() > 64 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "access_instances.tenant_id",
                    reason: "must be non-empty and <= 64 chars",
                },
            ));
        }
        Ok(())
    }

    fn validate_access_idempotency(
        field: &'static str,
        idempotency_key: &str,
    ) -> Result<(), StorageError> {
        if idempotency_key.trim().is_empty() || idempotency_key.len() > 128 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field,
                    reason: "must be non-empty and <= 128 chars",
                },
            ));
        }
        Ok(())
    }

    fn ph2access_effective_mode(
        &self,
        access_instance_id: &str,
        base_mode: AccessMode,
        now: MonotonicTimeNs,
    ) -> AccessMode {
        let mut effective = base_mode;
        for row in self
            .access_overrides
            .iter()
            .filter(|r| r.access_instance_id == access_instance_id)
        {
            if row.status != AccessOverrideStatus::Active {
                continue;
            }
            if row.starts_at.0 > now.0 {
                continue;
            }
            if row.expires_at.map(|ts| ts.0 <= now.0).unwrap_or(false) {
                continue;
            }
            if let Some(grant_mode) = Self::parse_grant_mode(&row.scope_json) {
                if access_mode_rank(grant_mode) > access_mode_rank(effective) {
                    effective = grant_mode;
                }
            }
        }
        effective
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph2access_upsert_instance_commit(
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
        Self::validate_access_tenant_id(&tenant_id)?;
        if role_template_id.trim().is_empty() || role_template_id.len() > 96 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "access_instances.role_template_id",
                    reason: "must be non-empty and <= 96 chars",
                },
            ));
        }
        if baseline_permissions_json.trim().is_empty() || baseline_permissions_json.len() > 4096 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "access_instances.baseline_permissions_json",
                    reason: "must be non-empty and <= 4096 chars",
                },
            ));
        }
        if policy_snapshot_ref.trim().is_empty() || policy_snapshot_ref.len() > 128 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "access_instances.policy_snapshot_ref",
                    reason: "must be non-empty and <= 128 chars",
                },
            ));
        }
        if !self.identities.contains_key(&user_id) {
            return Err(StorageError::ForeignKeyViolation {
                table: "access_instances.user_id",
                key: user_id.as_str().to_string(),
            });
        }

        let mut idempotency_key_norm = None;
        if let Some(k) = idempotency_key {
            Self::validate_access_idempotency("access_instances.idempotency_key", &k)?;
            let idx = (tenant_id.clone(), user_id.clone(), k.clone());
            if let Some(existing_instance_id) = self.access_instance_idempotency_index.get(&idx) {
                if let Some(existing_key) = self.access_instances_by_id.get(existing_instance_id) {
                    if let Some(existing_row) = self.access_instances.get(existing_key) {
                        return Ok(existing_row.clone());
                    }
                }
                return Err(StorageError::ForeignKeyViolation {
                    table: "access_instances.access_instance_id",
                    key: existing_instance_id.clone(),
                });
            }
            idempotency_key_norm = Some(k);
        }

        let access_instance_id = format!(
            "accinst_{}",
            hash_hex_64(&format!("{}:{}", tenant_id, user_id.as_str()))
        );
        let key = (tenant_id.clone(), user_id.clone());
        let created_at = self
            .access_instances
            .get(&key)
            .map(|existing| existing.created_at)
            .unwrap_or(now);

        let row = AccessInstanceRecord {
            schema_version: SchemaVersion(1),
            access_instance_id: access_instance_id.clone(),
            tenant_id: tenant_id.clone(),
            user_id: user_id.clone(),
            role_template_id,
            effective_access_mode,
            baseline_permissions_json,
            identity_verified,
            verification_level,
            device_trust_level,
            lifecycle_state,
            policy_snapshot_ref,
            created_at,
            updated_at: now,
            idempotency_key: idempotency_key_norm.clone(),
        };

        self.access_instances.insert(key.clone(), row.clone());
        self.access_instances_by_id
            .insert(access_instance_id.clone(), key);
        if let Some(k) = idempotency_key_norm {
            self.access_instance_idempotency_index
                .insert((tenant_id, user_id, k), access_instance_id);
        }
        Ok(row)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph2access_apply_override_commit(
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
        Self::validate_access_tenant_id(&tenant_id)?;
        Self::validate_access_idempotency("access_overrides.idempotency_key", &idempotency_key)?;
        if scope_json.trim().is_empty() || scope_json.len() > 4096 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "access_overrides.scope_json",
                    reason: "must be non-empty and <= 4096 chars",
                },
            ));
        }
        if approved_via_simulation_id.trim().is_empty() || approved_via_simulation_id.len() > 96 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "access_overrides.approved_via_simulation_id",
                    reason: "must be non-empty and <= 96 chars",
                },
            ));
        }
        if let Some(ts) = expires_at {
            if ts.0 <= starts_at.0 {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "access_overrides.expires_at",
                        reason: "must be > starts_at when provided",
                    },
                ));
            }
        }
        if !self.identities.contains_key(&approved_by_user_id) {
            return Err(StorageError::ForeignKeyViolation {
                table: "access_overrides.approved_by_user_id",
                key: approved_by_user_id.as_str().to_string(),
            });
        }

        let instance_key = self
            .access_instances_by_id
            .get(&access_instance_id)
            .cloned()
            .ok_or(StorageError::ForeignKeyViolation {
                table: "access_overrides.access_instance_id",
                key: access_instance_id.clone(),
            })?;
        let instance =
            self.access_instances
                .get(&instance_key)
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "access_instances.access_instance_id",
                    key: access_instance_id.clone(),
                })?;
        if instance.tenant_id != tenant_id {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "access_overrides.tenant_id",
                    reason: "must match access instance tenant",
                },
            ));
        }

        let idx = (
            tenant_id.clone(),
            access_instance_id.clone(),
            idempotency_key.clone(),
        );
        if let Some(existing_override_id) = self.access_override_idempotency_index.get(&idx) {
            if let Some(existing) = self
                .access_overrides
                .iter()
                .find(|row| row.override_id == *existing_override_id)
            {
                return Ok(existing.clone());
            }
            return Err(StorageError::ForeignKeyViolation {
                table: "access_overrides.override_id",
                key: existing_override_id.clone(),
            });
        }

        if override_type != AccessOverrideType::Revoke {
            let overlap_exists = self.access_overrides.iter().any(|row| {
                row.access_instance_id == access_instance_id
                    && row.scope_json == scope_json
                    && row.status == AccessOverrideStatus::Active
                    && windows_overlap(row.starts_at, row.expires_at, starts_at, expires_at)
            });
            if overlap_exists {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "access_overrides.scope_json",
                        reason: "overlapping ACTIVE override scope is not allowed",
                    },
                ));
            }
        }

        let status = match override_type {
            AccessOverrideType::Revoke => AccessOverrideStatus::Revoked,
            _ => {
                if expires_at.map(|ts| ts.0 <= now.0).unwrap_or(false) {
                    AccessOverrideStatus::Expired
                } else {
                    AccessOverrideStatus::Active
                }
            }
        };

        let override_id = format!(
            "accessovr_{}",
            hash_hex_64(&format!(
                "{}:{}:{}",
                tenant_id, access_instance_id, idempotency_key
            ))
        );
        let row = AccessOverrideRecord {
            schema_version: SchemaVersion(1),
            override_id: override_id.clone(),
            access_instance_id: access_instance_id.clone(),
            tenant_id: tenant_id.clone(),
            override_type,
            scope_json,
            status,
            approved_by_user_id,
            approved_via_simulation_id,
            reason_code,
            starts_at,
            expires_at,
            created_at: now,
            updated_at: now,
            idempotency_key: idempotency_key.clone(),
        };
        self.access_overrides.push(row.clone());
        self.access_override_idempotency_index
            .insert(idx, override_id);
        Ok(row)
    }

    pub fn ph2access_get_instance_by_tenant_user(
        &self,
        tenant_id: &str,
        user_id: &UserId,
    ) -> Option<&AccessInstanceRecord> {
        self.access_instances
            .get(&(tenant_id.to_string(), user_id.clone()))
    }

    pub fn ph2access_get_instance_by_id(
        &self,
        access_instance_id: &str,
    ) -> Option<&AccessInstanceRecord> {
        let key = self.access_instances_by_id.get(access_instance_id)?;
        self.access_instances.get(key)
    }

    pub fn ph2access_get_overrides_for_instance(
        &self,
        access_instance_id: &str,
    ) -> Vec<&AccessOverrideRecord> {
        self.access_overrides
            .iter()
            .filter(|row| row.access_instance_id == access_instance_id)
            .collect()
    }

    pub fn ph2access_instance_rows(&self) -> &BTreeMap<(String, UserId), AccessInstanceRecord> {
        &self.access_instances
    }

    pub fn ph2access_override_rows(&self) -> &[AccessOverrideRecord] {
        &self.access_overrides
    }

    pub fn attempt_overwrite_access_override(
        &mut self,
        _override_id: &str,
    ) -> Result<(), StorageError> {
        Err(StorageError::AppendOnlyViolation {
            table: "access_overrides",
        })
    }

    pub fn ph1access_gate_decide(
        &self,
        user_id: UserId,
        access_engine_instance_id: String,
        requested_action: String,
        access_request_context: AccessMode,
        device_trust_level: AccessDeviceTrustLevel,
        sensitive_data_request: bool,
        now: MonotonicTimeNs,
    ) -> Result<AccessGateDecisionRecord, StorageError> {
        if requested_action.trim().is_empty() || requested_action.len() > 128 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1access_gate_decide.requested_action",
                    reason: "must be non-empty and <= 128 chars",
                },
            ));
        }

        let Some(instance) = self.ph2access_get_instance_by_id(&access_engine_instance_id) else {
            return Ok(AccessGateDecisionRecord {
                schema_version: SchemaVersion(1),
                access_decision: AccessDecision::Deny,
                effective_access_mode: AccessMode::R,
                restriction_flags: vec!["INSTANCE_MISSING".to_string()],
                escalation_trigger: None,
                reason_code: ACCESS_REASON_INSTANCE_MISSING,
            });
        };

        if instance.user_id != user_id {
            return Ok(AccessGateDecisionRecord {
                schema_version: SchemaVersion(1),
                access_decision: AccessDecision::Deny,
                effective_access_mode: instance.effective_access_mode,
                restriction_flags: vec!["USER_SCOPE_MISMATCH".to_string()],
                escalation_trigger: None,
                reason_code: ACCESS_REASON_SCOPE_MISMATCH,
            });
        }

        let effective_mode = self.ph2access_effective_mode(
            &access_engine_instance_id,
            instance.effective_access_mode,
            now,
        );

        if instance.lifecycle_state == AccessLifecycleState::Suspended {
            return Ok(AccessGateDecisionRecord {
                schema_version: SchemaVersion(1),
                access_decision: AccessDecision::Deny,
                effective_access_mode: effective_mode,
                restriction_flags: vec!["INSTANCE_SUSPENDED".to_string()],
                escalation_trigger: None,
                reason_code: ACCESS_REASON_DENIED,
            });
        }

        if instance.lifecycle_state == AccessLifecycleState::Restricted
            || !instance.identity_verified
        {
            return Ok(AccessGateDecisionRecord {
                schema_version: SchemaVersion(1),
                access_decision: AccessDecision::Escalate,
                effective_access_mode: effective_mode,
                restriction_flags: vec!["IDENTITY_OR_INSTANCE_RESTRICTED".to_string()],
                escalation_trigger: Some(AccessEscalationTrigger::StepUpProofRequired),
                reason_code: ACCESS_REASON_ESCALATE_REQUIRED,
            });
        }

        if sensitive_data_request
            && !Self::has_financial_authorization(&instance.baseline_permissions_json)
        {
            return Ok(AccessGateDecisionRecord {
                schema_version: SchemaVersion(1),
                access_decision: AccessDecision::Deny,
                effective_access_mode: effective_mode,
                restriction_flags: vec!["SENSITIVE_FIELDS_BLOCKED".to_string()],
                escalation_trigger: None,
                reason_code: ACCESS_REASON_SENSITIVE_DENY,
            });
        }

        if device_trust_level <= AccessDeviceTrustLevel::Dtl2
            && access_mode_rank(access_request_context) >= access_mode_rank(AccessMode::A)
        {
            return Ok(AccessGateDecisionRecord {
                schema_version: SchemaVersion(1),
                access_decision: AccessDecision::Escalate,
                effective_access_mode: effective_mode,
                restriction_flags: vec!["DEVICE_UNTRUSTED".to_string()],
                escalation_trigger: Some(AccessEscalationTrigger::StepUpProofRequired),
                reason_code: ACCESS_REASON_DEVICE_UNTRUSTED,
            });
        }

        if access_mode_rank(access_request_context) <= access_mode_rank(effective_mode) {
            return Ok(AccessGateDecisionRecord {
                schema_version: SchemaVersion(1),
                access_decision: AccessDecision::Allow,
                effective_access_mode: effective_mode,
                restriction_flags: Vec::new(),
                escalation_trigger: None,
                reason_code: ACCESS_REASON_ALLOWED,
            });
        }

        Ok(AccessGateDecisionRecord {
            schema_version: SchemaVersion(1),
            access_decision: AccessDecision::Escalate,
            effective_access_mode: effective_mode,
            restriction_flags: vec!["MODE_UPGRADE_REQUIRED".to_string()],
            escalation_trigger: Some(AccessEscalationTrigger::ApApprovalRequired),
            reason_code: ACCESS_REASON_AP_REQUIRED,
        })
    }

    // ------------------------
    // PH1.C (STT router + transcript gate)
    // ------------------------

    fn validate_ph1c_tenant_id(tenant_id: &str) -> Result<(), StorageError> {
        if tenant_id.trim().is_empty() || tenant_id.len() > 64 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1c.tenant_id",
                    reason: "must be non-empty and <= 64 chars",
                },
            ));
        }
        Ok(())
    }

    fn validate_ph1c_idempotency(
        field: &'static str,
        idempotency_key: &str,
    ) -> Result<(), StorageError> {
        if idempotency_key.trim().is_empty() || idempotency_key.len() > 128 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field,
                    reason: "must be non-empty and <= 128 chars",
                },
            ));
        }
        Ok(())
    }

    fn ph1c_confidence_bucket_str(confidence_bucket: Ph1cConfidenceBucket) -> &'static str {
        match confidence_bucket {
            Ph1cConfidenceBucket::High => "HIGH",
            Ph1cConfidenceBucket::Med => "MED",
            Ph1cConfidenceBucket::Low => "LOW",
        }
    }

    fn ph1c_retry_advice_str(retry_advice: Ph1cRetryAdvice) -> &'static str {
        match retry_advice {
            Ph1cRetryAdvice::Repeat => "REPEAT",
            Ph1cRetryAdvice::SpeakSlower => "SPEAK_SLOWER",
            Ph1cRetryAdvice::MoveCloser => "MOVE_CLOSER",
            Ph1cRetryAdvice::QuietEnv => "QUIET_ENV",
        }
    }

    fn ph1c_audit_idempotency(idempotency_key: &str, suffix: &str) -> Result<String, StorageError> {
        let scoped = format!("{idempotency_key}:{suffix}");
        Self::validate_ph1c_idempotency("ph1c.audit.idempotency_key", &scoped)?;
        Ok(scoped)
    }

    fn ph1c_validate_scope_and_bindings(
        &mut self,
        tenant_id: &str,
        user_id: &UserId,
        device_id: &DeviceId,
        session_id: Option<SessionId>,
    ) -> Result<(), StorageError> {
        if !self.identities.contains_key(user_id) {
            return Err(StorageError::ForeignKeyViolation {
                table: "conversation_ledger.user_id",
                key: user_id.as_str().to_string(),
            });
        }

        let dev = self
            .devices
            .get(device_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "conversation_ledger.device_id",
                key: device_id.as_str().to_string(),
            })?;
        if dev.user_id != *user_id {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1c.device_id",
                    reason: "device must belong to user_id",
                },
            ));
        }

        if let Some(sid) = session_id {
            let session = self
                .sessions
                .get(&sid)
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "conversation_ledger.session_id",
                    key: sid.0.to_string(),
                })?;
            if session.user_id != *user_id || session.device_id != *device_id {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1c.session_scope",
                        reason: "session must match user_id and device_id",
                    },
                ));
            }
        }

        if let Some(bound_tenant_id) = self.ph1c_device_tenant_bindings.get(device_id) {
            if *bound_tenant_id != tenant_id {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1c.tenant_id",
                        reason: "must match device tenant binding",
                    },
                ));
            }
        } else {
            self.ph1c_device_tenant_bindings
                .insert(device_id.clone(), tenant_id.to_string());
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1c_transcript_ok_commit(
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
        const C_REASON_TRANSCRIPT_OK: ReasonCodeId = ReasonCodeId(0x4300_1001);
        const C_REASON_CANDIDATE_EVAL_OK: ReasonCodeId = ReasonCodeId(0x4300_1002);

        Self::validate_ph1c_tenant_id(&tenant_id)?;
        Self::validate_ph1c_idempotency("ph1c.idempotency_key", &idempotency_key)?;
        self.ph1c_validate_scope_and_bindings(&tenant_id, &user_id, &device_id, session_id)?;

        let conversation_input = ConversationTurnInput::v1(
            now,
            correlation_id,
            turn_id,
            session_id,
            user_id.clone(),
            Some(device_id.clone()),
            ConversationRole::User,
            ConversationSource::VoiceTranscript,
            transcript_text,
            transcript_hash.clone(),
            PrivacyScope::PublicChat,
            Some(idempotency_key.clone()),
            None,
            None,
        )?;

        let transcript_payload = AuditPayloadMin::v1(BTreeMap::from([(
            PayloadKey::new("transcript_hash")?,
            PayloadValue::new(transcript_hash)?,
        )]))?;

        let candidate_eval_payload = AuditPayloadMin::v1(BTreeMap::from([
            (
                PayloadKey::new("decision")?,
                PayloadValue::new("TRANSCRIPT_OK")?,
            ),
            (
                PayloadKey::new("language_tag")?,
                PayloadValue::new(language_tag.as_str())?,
            ),
            (
                PayloadKey::new("confidence_bucket")?,
                PayloadValue::new(Self::ph1c_confidence_bucket_str(confidence_bucket))?,
            ),
        ]))?;

        let transcript_audit_input = AuditEventInput::v1(
            now,
            Some(tenant_id.clone()),
            None,
            session_id,
            Some(user_id.clone()),
            Some(device_id.clone()),
            AuditEngine::Ph1C,
            AuditEventType::TranscriptOk,
            C_REASON_TRANSCRIPT_OK,
            AuditSeverity::Info,
            correlation_id,
            turn_id,
            transcript_payload,
            None,
            Some(Self::ph1c_audit_idempotency(
                &idempotency_key,
                "transcript_ok",
            )?),
        )?;

        let candidate_audit_input = AuditEventInput::v1(
            now,
            Some(tenant_id),
            None,
            session_id,
            Some(user_id),
            Some(device_id),
            AuditEngine::Ph1C,
            AuditEventType::SttCandidateEval,
            C_REASON_CANDIDATE_EVAL_OK,
            AuditSeverity::Info,
            correlation_id,
            turn_id,
            candidate_eval_payload,
            None,
            Some(Self::ph1c_audit_idempotency(
                &idempotency_key,
                "candidate_eval_ok",
            )?),
        )?;

        let conversation_turn_id = self.append_conversation_turn(conversation_input)?;
        let transcript_audit_event_id = self.append_audit_event(transcript_audit_input)?;
        let candidate_eval_audit_event_id = self.append_audit_event(candidate_audit_input)?;

        Ok(Ph1cTranscriptOkCommitResult {
            conversation_turn_id,
            transcript_audit_event_id,
            candidate_eval_audit_event_id,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1c_transcript_reject_commit(
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
        const C_REASON_CANDIDATE_EVAL_REJECT: ReasonCodeId = ReasonCodeId(0x4300_2001);

        Self::validate_ph1c_tenant_id(&tenant_id)?;
        Self::validate_ph1c_idempotency("ph1c.idempotency_key", &idempotency_key)?;
        self.ph1c_validate_scope_and_bindings(&tenant_id, &user_id, &device_id, session_id)?;

        let mut reject_payload_entries = BTreeMap::new();
        if let Some(hash) = transcript_hash {
            reject_payload_entries.insert(
                PayloadKey::new("transcript_hash")?,
                PayloadValue::new(hash)?,
            );
        }
        let transcript_reject_payload = AuditPayloadMin::v1(reject_payload_entries)?;

        let candidate_eval_payload = AuditPayloadMin::v1(BTreeMap::from([
            (
                PayloadKey::new("decision")?,
                PayloadValue::new("TRANSCRIPT_REJECT")?,
            ),
            (
                PayloadKey::new("retry_advice")?,
                PayloadValue::new(Self::ph1c_retry_advice_str(retry_advice))?,
            ),
        ]))?;

        let reject_audit_input = AuditEventInput::v1(
            now,
            Some(tenant_id.clone()),
            None,
            session_id,
            Some(user_id.clone()),
            Some(device_id.clone()),
            AuditEngine::Ph1C,
            AuditEventType::TranscriptReject,
            reject_reason_code,
            AuditSeverity::Warn,
            correlation_id,
            turn_id,
            transcript_reject_payload,
            None,
            Some(Self::ph1c_audit_idempotency(
                &idempotency_key,
                "transcript_reject",
            )?),
        )?;

        let candidate_audit_input = AuditEventInput::v1(
            now,
            Some(tenant_id),
            None,
            session_id,
            Some(user_id),
            Some(device_id),
            AuditEngine::Ph1C,
            AuditEventType::SttCandidateEval,
            C_REASON_CANDIDATE_EVAL_REJECT,
            AuditSeverity::Warn,
            correlation_id,
            turn_id,
            candidate_eval_payload,
            None,
            Some(Self::ph1c_audit_idempotency(
                &idempotency_key,
                "candidate_eval_reject",
            )?),
        )?;

        let transcript_reject_audit_event_id = self.append_audit_event(reject_audit_input)?;
        let candidate_eval_audit_event_id = self.append_audit_event(candidate_audit_input)?;

        Ok(Ph1cTranscriptRejectCommitResult {
            transcript_reject_audit_event_id,
            candidate_eval_audit_event_id,
        })
    }

    // ------------------------
    // PH1.NLP (Deterministic NLP Normalizer)
    // ------------------------

    fn validate_ph1nlp_tenant_id(tenant_id: &str) -> Result<(), StorageError> {
        if tenant_id.trim().is_empty() || tenant_id.len() > 64 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1nlp.tenant_id",
                    reason: "must be non-empty and <= 64 chars",
                },
            ));
        }
        Ok(())
    }

    fn validate_ph1nlp_idempotency(
        field: &'static str,
        idempotency_key: &str,
    ) -> Result<(), StorageError> {
        if idempotency_key.trim().is_empty() || idempotency_key.len() > 128 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field,
                    reason: "must be non-empty and <= 128 chars",
                },
            ));
        }
        Ok(())
    }

    fn validate_ph1nlp_bounded_text(
        field: &'static str,
        value: &str,
        max_len: usize,
    ) -> Result<(), StorageError> {
        if value.trim().is_empty() || value.len() > max_len {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field,
                    reason: "must be non-empty and within max length",
                },
            ));
        }
        Ok(())
    }

    fn validate_ph1nlp_scope_and_bindings(
        &mut self,
        tenant_id: &str,
        user_id: &UserId,
        device_id: &DeviceId,
        session_id: Option<SessionId>,
    ) -> Result<(), StorageError> {
        if !self.identities.contains_key(user_id) {
            return Err(StorageError::ForeignKeyViolation {
                table: "audit_events.user_id",
                key: user_id.as_str().to_string(),
            });
        }

        let dev = self
            .devices
            .get(device_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "audit_events.device_id",
                key: device_id.as_str().to_string(),
            })?;
        if dev.user_id != *user_id {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1nlp.device_id",
                    reason: "device must belong to user_id",
                },
            ));
        }

        if let Some(sid) = session_id {
            let session = self
                .sessions
                .get(&sid)
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "audit_events.session_id",
                    key: sid.0.to_string(),
                })?;
            if session.user_id != *user_id || session.device_id != *device_id {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1nlp.session_scope",
                        reason: "session must match user_id and device_id",
                    },
                ));
            }
        }

        if let Some(bound_tenant_id) = self.ph1nlp_device_tenant_bindings.get(device_id) {
            if *bound_tenant_id != tenant_id {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1nlp.tenant_id",
                        reason: "must match device tenant binding",
                    },
                ));
            }
        } else {
            self.ph1nlp_device_tenant_bindings
                .insert(device_id.clone(), tenant_id.to_string());
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1nlp_intent_draft_commit(
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
        Self::validate_ph1nlp_tenant_id(&tenant_id)?;
        Self::validate_ph1nlp_idempotency("ph1nlp.idempotency_key", &idempotency_key)?;
        Self::validate_ph1nlp_bounded_text("ph1nlp.intent_type", &intent_type, 64)?;
        Self::validate_ph1nlp_bounded_text("ph1nlp.overall_confidence", &overall_confidence, 8)?;
        self.validate_ph1nlp_scope_and_bindings(&tenant_id, &user_id, &device_id, session_id)?;

        let payload = AuditPayloadMin::v1(BTreeMap::from([
            (
                PayloadKey::new("decision")?,
                PayloadValue::new("INTENT_DRAFT")?,
            ),
            (
                PayloadKey::new("intent_type")?,
                PayloadValue::new(intent_type)?,
            ),
            (
                PayloadKey::new("overall_confidence")?,
                PayloadValue::new(overall_confidence)?,
            ),
        ]))?;

        let input = AuditEventInput::v1(
            now,
            Some(tenant_id),
            None,
            session_id,
            Some(user_id),
            Some(device_id),
            AuditEngine::Ph1Nlp,
            AuditEventType::NlpIntentDraft,
            reason_code,
            AuditSeverity::Info,
            correlation_id,
            turn_id,
            payload,
            None,
            Some(idempotency_key),
        )?;

        self.append_audit_event(input)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1nlp_clarify_commit(
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
        Self::validate_ph1nlp_tenant_id(&tenant_id)?;
        Self::validate_ph1nlp_idempotency("ph1nlp.idempotency_key", &idempotency_key)?;
        Self::validate_ph1nlp_bounded_text("ph1nlp.what_is_missing", &what_is_missing, 64)?;
        self.validate_ph1nlp_scope_and_bindings(&tenant_id, &user_id, &device_id, session_id)?;

        let payload = AuditPayloadMin::v1(BTreeMap::from([
            (PayloadKey::new("decision")?, PayloadValue::new("CLARIFY")?),
            (
                PayloadKey::new("what_is_missing")?,
                PayloadValue::new(what_is_missing)?,
            ),
        ]))?;

        let input = AuditEventInput::v1(
            now,
            Some(tenant_id),
            None,
            session_id,
            Some(user_id),
            Some(device_id),
            AuditEngine::Ph1Nlp,
            AuditEventType::NlpClarify,
            reason_code,
            AuditSeverity::Warn,
            correlation_id,
            turn_id,
            payload,
            None,
            Some(idempotency_key),
        )?;

        self.append_audit_event(input)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1nlp_chat_commit(
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
        Self::validate_ph1nlp_tenant_id(&tenant_id)?;
        Self::validate_ph1nlp_idempotency("ph1nlp.idempotency_key", &idempotency_key)?;
        self.validate_ph1nlp_scope_and_bindings(&tenant_id, &user_id, &device_id, session_id)?;

        let payload = AuditPayloadMin::v1(BTreeMap::from([(
            PayloadKey::new("decision")?,
            PayloadValue::new("CHAT")?,
        )]))?;

        let input = AuditEventInput::v1(
            now,
            Some(tenant_id),
            None,
            session_id,
            Some(user_id),
            Some(device_id),
            AuditEngine::Ph1Nlp,
            AuditEventType::NlpIntentDraft,
            reason_code,
            AuditSeverity::Info,
            correlation_id,
            turn_id,
            payload,
            None,
            Some(idempotency_key),
        )?;

        self.append_audit_event(input)
    }

    pub fn ph1nlp_audit_rows(&self, correlation_id: CorrelationId) -> Vec<&AuditEvent> {
        self.audit_events
            .iter()
            .filter(|e| e.correlation_id == correlation_id && e.engine == AuditEngine::Ph1Nlp)
            .collect()
    }

    // ------------------------
    // PH1.D (LLM Router Contract)
    // ------------------------

    fn validate_ph1d_tenant_id(tenant_id: &str) -> Result<(), StorageError> {
        if tenant_id.trim().is_empty() || tenant_id.len() > 64 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1d.tenant_id",
                    reason: "must be non-empty and <= 64 chars",
                },
            ));
        }
        Ok(())
    }

    fn validate_ph1d_idempotency(
        field: &'static str,
        idempotency_key: &str,
    ) -> Result<(), StorageError> {
        if idempotency_key.trim().is_empty() || idempotency_key.len() > 128 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field,
                    reason: "must be non-empty and <= 128 chars",
                },
            ));
        }
        Ok(())
    }

    fn validate_ph1d_bounded_text(
        field: &'static str,
        value: &str,
        max_len: usize,
    ) -> Result<(), StorageError> {
        if value.trim().is_empty() || value.len() > max_len {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field,
                    reason: "must be non-empty and within max length",
                },
            ));
        }
        Ok(())
    }

    fn validate_ph1d_scope_and_bindings(
        &mut self,
        tenant_id: &str,
        user_id: &UserId,
        device_id: &DeviceId,
        session_id: Option<SessionId>,
    ) -> Result<(), StorageError> {
        if !self.identities.contains_key(user_id) {
            return Err(StorageError::ForeignKeyViolation {
                table: "audit_events.user_id",
                key: user_id.as_str().to_string(),
            });
        }

        let dev = self
            .devices
            .get(device_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "audit_events.device_id",
                key: device_id.as_str().to_string(),
            })?;
        if dev.user_id != *user_id {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1d.device_id",
                    reason: "device must belong to user_id",
                },
            ));
        }

        if let Some(sid) = session_id {
            let session = self
                .sessions
                .get(&sid)
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "audit_events.session_id",
                    key: sid.0.to_string(),
                })?;
            if session.user_id != *user_id || session.device_id != *device_id {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1d.session_scope",
                        reason: "session must match user_id and device_id",
                    },
                ));
            }
        }

        if let Some(bound_tenant_id) = self.ph1d_device_tenant_bindings.get(device_id) {
            if *bound_tenant_id != tenant_id {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1d.tenant_id",
                        reason: "must match device tenant binding",
                    },
                ));
            }
        } else {
            self.ph1d_device_tenant_bindings
                .insert(device_id.clone(), tenant_id.to_string());
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1d_chat_commit(
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
        Self::validate_ph1d_tenant_id(&tenant_id)?;
        Self::validate_ph1d_idempotency("ph1d.idempotency_key", &idempotency_key)?;
        self.validate_ph1d_scope_and_bindings(&tenant_id, &user_id, &device_id, session_id)?;

        let payload = AuditPayloadMin::v1(BTreeMap::from([
            (PayloadKey::new("decision")?, PayloadValue::new("CHAT")?),
            (PayloadKey::new("output_mode")?, PayloadValue::new("chat")?),
        ]))?;

        let input = AuditEventInput::v1(
            now,
            Some(tenant_id),
            None,
            session_id,
            Some(user_id),
            Some(device_id),
            AuditEngine::Ph1D,
            AuditEventType::Other,
            reason_code,
            AuditSeverity::Info,
            correlation_id,
            turn_id,
            payload,
            None,
            Some(idempotency_key),
        )?;

        self.append_audit_event(input)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1d_intent_commit(
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
        Self::validate_ph1d_tenant_id(&tenant_id)?;
        Self::validate_ph1d_idempotency("ph1d.idempotency_key", &idempotency_key)?;
        Self::validate_ph1d_bounded_text("ph1d.refined_intent_type", &refined_intent_type, 64)?;
        self.validate_ph1d_scope_and_bindings(&tenant_id, &user_id, &device_id, session_id)?;

        let payload = AuditPayloadMin::v1(BTreeMap::from([
            (PayloadKey::new("decision")?, PayloadValue::new("INTENT")?),
            (
                PayloadKey::new("refined_intent_type")?,
                PayloadValue::new(refined_intent_type)?,
            ),
            (
                PayloadKey::new("output_mode")?,
                PayloadValue::new("intent")?,
            ),
        ]))?;

        let input = AuditEventInput::v1(
            now,
            Some(tenant_id),
            None,
            session_id,
            Some(user_id),
            Some(device_id),
            AuditEngine::Ph1D,
            AuditEventType::Other,
            reason_code,
            AuditSeverity::Info,
            correlation_id,
            turn_id,
            payload,
            None,
            Some(idempotency_key),
        )?;

        self.append_audit_event(input)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1d_clarify_commit(
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
        Self::validate_ph1d_tenant_id(&tenant_id)?;
        Self::validate_ph1d_idempotency("ph1d.idempotency_key", &idempotency_key)?;
        Self::validate_ph1d_bounded_text("ph1d.what_is_missing", &what_is_missing, 64)?;
        self.validate_ph1d_scope_and_bindings(&tenant_id, &user_id, &device_id, session_id)?;

        let payload = AuditPayloadMin::v1(BTreeMap::from([
            (PayloadKey::new("decision")?, PayloadValue::new("CLARIFY")?),
            (
                PayloadKey::new("what_is_missing")?,
                PayloadValue::new(what_is_missing)?,
            ),
            (
                PayloadKey::new("output_mode")?,
                PayloadValue::new("clarify")?,
            ),
        ]))?;

        let input = AuditEventInput::v1(
            now,
            Some(tenant_id),
            None,
            session_id,
            Some(user_id),
            Some(device_id),
            AuditEngine::Ph1D,
            AuditEventType::Other,
            reason_code,
            AuditSeverity::Warn,
            correlation_id,
            turn_id,
            payload,
            None,
            Some(idempotency_key),
        )?;

        self.append_audit_event(input)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1d_analysis_commit(
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
        Self::validate_ph1d_tenant_id(&tenant_id)?;
        Self::validate_ph1d_idempotency("ph1d.idempotency_key", &idempotency_key)?;
        Self::validate_ph1d_bounded_text("ph1d.analysis_kind", &analysis_kind, 64)?;
        self.validate_ph1d_scope_and_bindings(&tenant_id, &user_id, &device_id, session_id)?;

        let payload = AuditPayloadMin::v1(BTreeMap::from([
            (PayloadKey::new("decision")?, PayloadValue::new("ANALYSIS")?),
            (
                PayloadKey::new("analysis_kind")?,
                PayloadValue::new(analysis_kind)?,
            ),
            (
                PayloadKey::new("output_mode")?,
                PayloadValue::new("analysis")?,
            ),
        ]))?;

        let input = AuditEventInput::v1(
            now,
            Some(tenant_id),
            None,
            session_id,
            Some(user_id),
            Some(device_id),
            AuditEngine::Ph1D,
            AuditEventType::Other,
            reason_code,
            AuditSeverity::Info,
            correlation_id,
            turn_id,
            payload,
            None,
            Some(idempotency_key),
        )?;

        self.append_audit_event(input)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1d_fail_closed_commit(
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
        Self::validate_ph1d_tenant_id(&tenant_id)?;
        Self::validate_ph1d_idempotency("ph1d.idempotency_key", &idempotency_key)?;
        Self::validate_ph1d_bounded_text("ph1d.fail_code", &fail_code, 64)?;
        self.validate_ph1d_scope_and_bindings(&tenant_id, &user_id, &device_id, session_id)?;

        let payload = AuditPayloadMin::v1(BTreeMap::from([
            (
                PayloadKey::new("decision")?,
                PayloadValue::new("FAIL_CLOSED")?,
            ),
            (PayloadKey::new("fail_code")?, PayloadValue::new(fail_code)?),
            (PayloadKey::new("output_mode")?, PayloadValue::new("fail")?),
        ]))?;

        let input = AuditEventInput::v1(
            now,
            Some(tenant_id),
            None,
            session_id,
            Some(user_id),
            Some(device_id),
            AuditEngine::Ph1D,
            AuditEventType::Other,
            reason_code,
            AuditSeverity::Error,
            correlation_id,
            turn_id,
            payload,
            None,
            Some(idempotency_key),
        )?;

        self.append_audit_event(input)
    }

    pub fn ph1d_audit_rows(&self, correlation_id: CorrelationId) -> Vec<&AuditEvent> {
        self.audit_events
            .iter()
            .filter(|e| e.correlation_id == correlation_id && e.engine == AuditEngine::Ph1D)
            .collect()
    }

    // ------------------------
    // PH1.X (Conversation Orchestrator)
    // ------------------------

    fn validate_ph1x_tenant_id(tenant_id: &str) -> Result<(), StorageError> {
        if tenant_id.trim().is_empty() || tenant_id.len() > 64 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1x.tenant_id",
                    reason: "must be non-empty and <= 64 chars",
                },
            ));
        }
        Ok(())
    }

    fn validate_ph1x_idempotency(
        field: &'static str,
        idempotency_key: &str,
    ) -> Result<(), StorageError> {
        if idempotency_key.trim().is_empty() || idempotency_key.len() > 128 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field,
                    reason: "must be non-empty and <= 128 chars",
                },
            ));
        }
        Ok(())
    }

    fn validate_ph1x_bounded_text(
        field: &'static str,
        value: &str,
        max_len: usize,
    ) -> Result<(), StorageError> {
        if value.trim().is_empty() || value.len() > max_len {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field,
                    reason: "must be non-empty and within max length",
                },
            ));
        }
        Ok(())
    }

    fn validate_ph1x_scope_and_bindings(
        &mut self,
        tenant_id: &str,
        user_id: &UserId,
        device_id: &DeviceId,
        session_id: Option<SessionId>,
    ) -> Result<(), StorageError> {
        if !self.identities.contains_key(user_id) {
            return Err(StorageError::ForeignKeyViolation {
                table: "audit_events.user_id",
                key: user_id.as_str().to_string(),
            });
        }

        let dev = self
            .devices
            .get(device_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "audit_events.device_id",
                key: device_id.as_str().to_string(),
            })?;
        if dev.user_id != *user_id {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1x.device_id",
                    reason: "device must belong to user_id",
                },
            ));
        }

        if let Some(sid) = session_id {
            let session = self
                .sessions
                .get(&sid)
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "audit_events.session_id",
                    key: sid.0.to_string(),
                })?;
            if session.user_id != *user_id || session.device_id != *device_id {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1x.session_scope",
                        reason: "session must match user_id and device_id",
                    },
                ));
            }
        }

        if let Some(bound_tenant_id) = self.ph1x_device_tenant_bindings.get(device_id) {
            if *bound_tenant_id != tenant_id {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1x.tenant_id",
                        reason: "must match device tenant binding",
                    },
                ));
            }
        } else {
            self.ph1x_device_tenant_bindings
                .insert(device_id.clone(), tenant_id.to_string());
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1x_confirm_commit(
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
        Self::validate_ph1x_tenant_id(&tenant_id)?;
        Self::validate_ph1x_idempotency("ph1x.idempotency_key", &idempotency_key)?;
        Self::validate_ph1x_bounded_text("ph1x.confirm_kind", &confirm_kind, 64)?;
        self.validate_ph1x_scope_and_bindings(&tenant_id, &user_id, &device_id, session_id)?;

        let payload = AuditPayloadMin::v1(BTreeMap::from([
            (PayloadKey::new("directive")?, PayloadValue::new("confirm")?),
            (
                PayloadKey::new("confirm_kind")?,
                PayloadValue::new(confirm_kind)?,
            ),
        ]))?;

        let input = AuditEventInput::v1(
            now,
            Some(tenant_id),
            None,
            session_id,
            Some(user_id),
            Some(device_id),
            AuditEngine::Ph1X,
            AuditEventType::XConfirm,
            reason_code,
            AuditSeverity::Info,
            correlation_id,
            turn_id,
            payload,
            None,
            Some(idempotency_key),
        )?;

        self.append_audit_event(input)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1x_clarify_commit(
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
        Self::validate_ph1x_tenant_id(&tenant_id)?;
        Self::validate_ph1x_idempotency("ph1x.idempotency_key", &idempotency_key)?;
        Self::validate_ph1x_bounded_text("ph1x.what_is_missing", &what_is_missing, 64)?;
        self.validate_ph1x_scope_and_bindings(&tenant_id, &user_id, &device_id, session_id)?;

        let payload = AuditPayloadMin::v1(BTreeMap::from([
            (PayloadKey::new("directive")?, PayloadValue::new("clarify")?),
            (
                PayloadKey::new("what_is_missing")?,
                PayloadValue::new(what_is_missing)?,
            ),
        ]))?;

        let input = AuditEventInput::v1(
            now,
            Some(tenant_id),
            None,
            session_id,
            Some(user_id),
            Some(device_id),
            AuditEngine::Ph1X,
            AuditEventType::Other,
            reason_code,
            AuditSeverity::Warn,
            correlation_id,
            turn_id,
            payload,
            None,
            Some(idempotency_key),
        )?;

        self.append_audit_event(input)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1x_respond_commit(
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
        Self::validate_ph1x_tenant_id(&tenant_id)?;
        Self::validate_ph1x_idempotency("ph1x.idempotency_key", &idempotency_key)?;
        Self::validate_ph1x_bounded_text("ph1x.response_kind", &response_kind, 64)?;
        self.validate_ph1x_scope_and_bindings(&tenant_id, &user_id, &device_id, session_id)?;

        let payload = AuditPayloadMin::v1(BTreeMap::from([
            (PayloadKey::new("directive")?, PayloadValue::new("respond")?),
            (
                PayloadKey::new("response_kind")?,
                PayloadValue::new(response_kind)?,
            ),
        ]))?;

        let input = AuditEventInput::v1(
            now,
            Some(tenant_id),
            None,
            session_id,
            Some(user_id),
            Some(device_id),
            AuditEngine::Ph1X,
            AuditEventType::Other,
            reason_code,
            AuditSeverity::Info,
            correlation_id,
            turn_id,
            payload,
            None,
            Some(idempotency_key),
        )?;

        self.append_audit_event(input)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1x_dispatch_commit(
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
        Self::validate_ph1x_tenant_id(&tenant_id)?;
        Self::validate_ph1x_idempotency("ph1x.idempotency_key", &idempotency_key)?;
        Self::validate_ph1x_bounded_text("ph1x.dispatch_target", &dispatch_target, 64)?;
        self.validate_ph1x_scope_and_bindings(&tenant_id, &user_id, &device_id, session_id)?;

        let payload = AuditPayloadMin::v1(BTreeMap::from([
            (
                PayloadKey::new("directive")?,
                PayloadValue::new("dispatch")?,
            ),
            (
                PayloadKey::new("dispatch_target")?,
                PayloadValue::new(dispatch_target)?,
            ),
        ]))?;

        let input = AuditEventInput::v1(
            now,
            Some(tenant_id),
            None,
            session_id,
            Some(user_id),
            Some(device_id),
            AuditEngine::Ph1X,
            AuditEventType::XDispatch,
            reason_code,
            AuditSeverity::Info,
            correlation_id,
            turn_id,
            payload,
            None,
            Some(idempotency_key),
        )?;

        self.append_audit_event(input)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1x_wait_commit(
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
        Self::validate_ph1x_tenant_id(&tenant_id)?;
        Self::validate_ph1x_idempotency("ph1x.idempotency_key", &idempotency_key)?;
        Self::validate_ph1x_bounded_text("ph1x.wait_kind", &wait_kind, 64)?;
        self.validate_ph1x_scope_and_bindings(&tenant_id, &user_id, &device_id, session_id)?;

        let payload = AuditPayloadMin::v1(BTreeMap::from([
            (PayloadKey::new("directive")?, PayloadValue::new("wait")?),
            (PayloadKey::new("wait_kind")?, PayloadValue::new(wait_kind)?),
        ]))?;

        let input = AuditEventInput::v1(
            now,
            Some(tenant_id),
            None,
            session_id,
            Some(user_id),
            Some(device_id),
            AuditEngine::Ph1X,
            AuditEventType::Other,
            reason_code,
            AuditSeverity::Info,
            correlation_id,
            turn_id,
            payload,
            None,
            Some(idempotency_key),
        )?;

        self.append_audit_event(input)
    }

    pub fn ph1x_audit_rows(&self, correlation_id: CorrelationId) -> Vec<&AuditEvent> {
        self.audit_events
            .iter()
            .filter(|e| e.correlation_id == correlation_id && e.engine == AuditEngine::Ph1X)
            .collect()
    }

    // ------------------------
    // PH1.WRITE (Professional Writing & Formatting)
    // ------------------------

    fn validate_ph1write_tenant_id(tenant_id: &str) -> Result<(), StorageError> {
        if tenant_id.trim().is_empty() || tenant_id.len() > 64 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1write.tenant_id",
                    reason: "must be non-empty and <= 64 chars",
                },
            ));
        }
        Ok(())
    }

    fn validate_ph1write_idempotency(
        field: &'static str,
        idempotency_key: &str,
    ) -> Result<(), StorageError> {
        if idempotency_key.trim().is_empty() || idempotency_key.len() > 128 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field,
                    reason: "must be non-empty and <= 128 chars",
                },
            ));
        }
        Ok(())
    }

    fn validate_ph1write_bounded_text(
        field: &'static str,
        value: &str,
        max_len: usize,
    ) -> Result<(), StorageError> {
        if value.trim().is_empty() || value.len() > max_len {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field,
                    reason: "must be non-empty and within max length",
                },
            ));
        }
        Ok(())
    }

    fn validate_ph1write_scope_and_bindings(
        &mut self,
        tenant_id: &str,
        user_id: &UserId,
        device_id: &DeviceId,
        session_id: Option<SessionId>,
    ) -> Result<(), StorageError> {
        if !self.identities.contains_key(user_id) {
            return Err(StorageError::ForeignKeyViolation {
                table: "audit_events.user_id",
                key: user_id.as_str().to_string(),
            });
        }

        let dev = self
            .devices
            .get(device_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "audit_events.device_id",
                key: device_id.as_str().to_string(),
            })?;
        if dev.user_id != *user_id {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1write.device_id",
                    reason: "device must belong to user_id",
                },
            ));
        }

        if let Some(sid) = session_id {
            let session = self
                .sessions
                .get(&sid)
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "audit_events.session_id",
                    key: sid.0.to_string(),
                })?;
            if session.user_id != *user_id || session.device_id != *device_id {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1write.session_scope",
                        reason: "session must match user_id and device_id",
                    },
                ));
            }
        }

        if let Some(bound_tenant_id) = self.ph1write_device_tenant_bindings.get(device_id) {
            if *bound_tenant_id != tenant_id {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1write.tenant_id",
                        reason: "must match device tenant binding",
                    },
                ));
            }
        } else {
            self.ph1write_device_tenant_bindings
                .insert(device_id.clone(), tenant_id.to_string());
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1write_format_commit(
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
        Self::validate_ph1write_tenant_id(&tenant_id)?;
        Self::validate_ph1write_idempotency("ph1write.idempotency_key", &idempotency_key)?;
        Self::validate_ph1write_bounded_text("ph1write.format_mode", &format_mode, 64)?;
        self.validate_ph1write_scope_and_bindings(&tenant_id, &user_id, &device_id, session_id)?;

        let payload = AuditPayloadMin::v1(BTreeMap::from([
            (PayloadKey::new("directive")?, PayloadValue::new("format")?),
            (
                PayloadKey::new("format_mode")?,
                PayloadValue::new(format_mode)?,
            ),
        ]))?;

        let input = AuditEventInput::v1(
            now,
            Some(tenant_id),
            None,
            session_id,
            Some(user_id),
            Some(device_id),
            AuditEngine::Other("PH1.WRITE".to_string()),
            AuditEventType::Other,
            reason_code,
            AuditSeverity::Info,
            correlation_id,
            turn_id,
            payload,
            None,
            Some(idempotency_key),
        )?;

        self.append_audit_event(input)
    }

    pub fn ph1write_audit_rows(&self, correlation_id: CorrelationId) -> Vec<&AuditEvent> {
        self.audit_events
            .iter()
            .filter(|e| {
                e.correlation_id == correlation_id
                    && matches!(&e.engine, AuditEngine::Other(name) if name == "PH1.WRITE")
            })
            .collect()
    }

    // ------------------------
    // PH1.TTS (Speech Output Engine)
    // ------------------------

    fn validate_ph1tts_tenant_id(tenant_id: &str) -> Result<(), StorageError> {
        if tenant_id.trim().is_empty() || tenant_id.len() > 64 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1tts.tenant_id",
                    reason: "must be non-empty and <= 64 chars",
                },
            ));
        }
        Ok(())
    }

    fn validate_ph1tts_idempotency(
        field: &'static str,
        idempotency_key: &str,
    ) -> Result<(), StorageError> {
        if idempotency_key.trim().is_empty() || idempotency_key.len() > 128 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field,
                    reason: "must be non-empty and <= 128 chars",
                },
            ));
        }
        Ok(())
    }

    fn validate_ph1tts_bounded_text(
        field: &'static str,
        value: &str,
        max_len: usize,
    ) -> Result<(), StorageError> {
        if value.trim().is_empty() || value.len() > max_len {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field,
                    reason: "must be non-empty and within max length",
                },
            ));
        }
        Ok(())
    }

    fn validate_ph1tts_scope_and_bindings(
        &mut self,
        tenant_id: &str,
        user_id: &UserId,
        device_id: &DeviceId,
        session_id: Option<SessionId>,
    ) -> Result<(), StorageError> {
        if !self.identities.contains_key(user_id) {
            return Err(StorageError::ForeignKeyViolation {
                table: "audit_events.user_id",
                key: user_id.as_str().to_string(),
            });
        }

        let dev = self
            .devices
            .get(device_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "audit_events.device_id",
                key: device_id.as_str().to_string(),
            })?;
        if dev.user_id != *user_id {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1tts.device_id",
                    reason: "device must belong to user_id",
                },
            ));
        }

        if let Some(sid) = session_id {
            let session = self
                .sessions
                .get(&sid)
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "audit_events.session_id",
                    key: sid.0.to_string(),
                })?;
            if session.user_id != *user_id || session.device_id != *device_id {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1tts.session_scope",
                        reason: "session must match user_id and device_id",
                    },
                ));
            }
        }

        if let Some(bound_tenant_id) = self.ph1tts_device_tenant_bindings.get(device_id) {
            if *bound_tenant_id != tenant_id {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1tts.tenant_id",
                        reason: "must match device tenant binding",
                    },
                ));
            }
        } else {
            self.ph1tts_device_tenant_bindings
                .insert(device_id.clone(), tenant_id.to_string());
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1tts_render_summary_commit(
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
        Self::validate_ph1tts_tenant_id(&tenant_id)?;
        Self::validate_ph1tts_idempotency("ph1tts.idempotency_key", &idempotency_key)?;
        Self::validate_ph1tts_bounded_text("ph1tts.route_class_used", &route_class_used, 32)?;
        Self::validate_ph1tts_bounded_text("ph1tts.mode_used", &mode_used, 32)?;
        Self::validate_ph1tts_bounded_text("ph1tts.voice_id", &voice_id, 96)?;
        self.validate_ph1tts_scope_and_bindings(&tenant_id, &user_id, &device_id, session_id)?;

        let payload = AuditPayloadMin::v1(BTreeMap::from([
            (
                PayloadKey::new("route_class_used")?,
                PayloadValue::new(route_class_used)?,
            ),
            (PayloadKey::new("mode_used")?, PayloadValue::new(mode_used)?),
            (PayloadKey::new("voice_id")?, PayloadValue::new(voice_id)?),
        ]))?;

        let input = AuditEventInput::v1(
            now,
            Some(tenant_id),
            None,
            session_id,
            Some(user_id),
            Some(device_id),
            AuditEngine::Ph1Tts,
            AuditEventType::TtsRenderSummary,
            reason_code,
            AuditSeverity::Info,
            correlation_id,
            turn_id,
            payload,
            None,
            Some(idempotency_key),
        )?;

        self.append_audit_event(input)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1tts_started_commit(
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
        Self::validate_ph1tts_tenant_id(&tenant_id)?;
        Self::validate_ph1tts_idempotency("ph1tts.idempotency_key", &idempotency_key)?;
        Self::validate_ph1tts_bounded_text("ph1tts.answer_id", &answer_id, 96)?;
        self.validate_ph1tts_scope_and_bindings(&tenant_id, &user_id, &device_id, session_id)?;

        let payload = AuditPayloadMin::v1(BTreeMap::from([(
            PayloadKey::new("answer_id")?,
            PayloadValue::new(answer_id)?,
        )]))?;

        let input = AuditEventInput::v1(
            now,
            Some(tenant_id),
            None,
            session_id,
            Some(user_id),
            Some(device_id),
            AuditEngine::Ph1Tts,
            AuditEventType::TtsStarted,
            reason_code,
            AuditSeverity::Info,
            correlation_id,
            turn_id,
            payload,
            None,
            Some(idempotency_key),
        )?;

        self.append_audit_event(input)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1tts_canceled_commit(
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
        Self::validate_ph1tts_tenant_id(&tenant_id)?;
        Self::validate_ph1tts_idempotency("ph1tts.idempotency_key", &idempotency_key)?;
        Self::validate_ph1tts_bounded_text("ph1tts.answer_id", &answer_id, 96)?;
        Self::validate_ph1tts_bounded_text("ph1tts.stop_reason", &stop_reason, 64)?;
        self.validate_ph1tts_scope_and_bindings(&tenant_id, &user_id, &device_id, session_id)?;

        let payload = AuditPayloadMin::v1(BTreeMap::from([
            (PayloadKey::new("answer_id")?, PayloadValue::new(answer_id)?),
            (
                PayloadKey::new("stop_reason")?,
                PayloadValue::new(stop_reason)?,
            ),
        ]))?;

        let input = AuditEventInput::v1(
            now,
            Some(tenant_id),
            None,
            session_id,
            Some(user_id),
            Some(device_id),
            AuditEngine::Ph1Tts,
            AuditEventType::TtsCanceled,
            reason_code,
            AuditSeverity::Info,
            correlation_id,
            turn_id,
            payload,
            None,
            Some(idempotency_key),
        )?;

        self.append_audit_event(input)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1tts_failed_commit(
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
        Self::validate_ph1tts_tenant_id(&tenant_id)?;
        Self::validate_ph1tts_idempotency("ph1tts.idempotency_key", &idempotency_key)?;
        Self::validate_ph1tts_bounded_text("ph1tts.answer_id", &answer_id, 96)?;
        Self::validate_ph1tts_bounded_text("ph1tts.fail_code", &fail_code, 64)?;
        self.validate_ph1tts_scope_and_bindings(&tenant_id, &user_id, &device_id, session_id)?;

        let payload = AuditPayloadMin::v1(BTreeMap::from([
            (PayloadKey::new("answer_id")?, PayloadValue::new(answer_id)?),
            (PayloadKey::new("fail_code")?, PayloadValue::new(fail_code)?),
        ]))?;

        let input = AuditEventInput::v1(
            now,
            Some(tenant_id),
            None,
            session_id,
            Some(user_id),
            Some(device_id),
            AuditEngine::Ph1Tts,
            AuditEventType::TtsFailed,
            reason_code,
            AuditSeverity::Error,
            correlation_id,
            turn_id,
            payload,
            None,
            Some(idempotency_key),
        )?;

        self.append_audit_event(input)
    }

    pub fn ph1tts_audit_rows(&self, correlation_id: CorrelationId) -> Vec<&AuditEvent> {
        self.audit_events
            .iter()
            .filter(|e| e.correlation_id == correlation_id && e.engine == AuditEngine::Ph1Tts)
            .collect()
    }

    // ------------------------
    // PH1.E (Tool Router)
    // ------------------------

    fn validate_ph1e_tenant_id(tenant_id: &str) -> Result<(), StorageError> {
        if tenant_id.trim().is_empty() || tenant_id.len() > 64 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1e.tenant_id",
                    reason: "must be non-empty and <= 64 chars",
                },
            ));
        }
        Ok(())
    }

    fn validate_ph1e_idempotency(
        field: &'static str,
        idempotency_key: &str,
    ) -> Result<(), StorageError> {
        if idempotency_key.trim().is_empty() || idempotency_key.len() > 128 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field,
                    reason: "must be non-empty and <= 128 chars",
                },
            ));
        }
        Ok(())
    }

    fn validate_ph1e_bounded_text(
        field: &'static str,
        value: &str,
        max_len: usize,
    ) -> Result<(), StorageError> {
        if value.trim().is_empty() || value.len() > max_len {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field,
                    reason: "must be non-empty and within max length",
                },
            ));
        }
        Ok(())
    }

    fn validate_ph1e_scope_and_bindings(
        &mut self,
        tenant_id: &str,
        user_id: &UserId,
        device_id: &DeviceId,
        session_id: Option<SessionId>,
    ) -> Result<(), StorageError> {
        if !self.identities.contains_key(user_id) {
            return Err(StorageError::ForeignKeyViolation {
                table: "audit_events.user_id",
                key: user_id.as_str().to_string(),
            });
        }

        let dev = self
            .devices
            .get(device_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "audit_events.device_id",
                key: device_id.as_str().to_string(),
            })?;
        if dev.user_id != *user_id {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1e.device_id",
                    reason: "device must belong to user_id",
                },
            ));
        }

        if let Some(sid) = session_id {
            let session = self
                .sessions
                .get(&sid)
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "audit_events.session_id",
                    key: sid.0.to_string(),
                })?;
            if session.user_id != *user_id || session.device_id != *device_id {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1e.session_scope",
                        reason: "session must match user_id and device_id",
                    },
                ));
            }
        }

        if let Some(bound_tenant_id) = self.ph1e_device_tenant_bindings.get(device_id) {
            if *bound_tenant_id != tenant_id {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1e.tenant_id",
                        reason: "must match device tenant binding",
                    },
                ));
            }
        } else {
            self.ph1e_device_tenant_bindings
                .insert(device_id.clone(), tenant_id.to_string());
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1e_tool_ok_commit(
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
        Self::validate_ph1e_tenant_id(&tenant_id)?;
        Self::validate_ph1e_idempotency("ph1e.idempotency_key", &idempotency_key)?;
        Self::validate_ph1e_bounded_text("ph1e.tool_name", &tool_name, 32)?;
        Self::validate_ph1e_bounded_text("ph1e.query_hash", &query_hash, 128)?;
        Self::validate_ph1e_bounded_text("ph1e.cache_status", &cache_status, 16)?;
        self.validate_ph1e_scope_and_bindings(&tenant_id, &user_id, &device_id, session_id)?;

        let payload = AuditPayloadMin::v1(BTreeMap::from([
            (PayloadKey::new("tool_name")?, PayloadValue::new(tool_name)?),
            (
                PayloadKey::new("query_hash")?,
                PayloadValue::new(query_hash)?,
            ),
            (
                PayloadKey::new("cache_status")?,
                PayloadValue::new(cache_status)?,
            ),
        ]))?;

        let input = AuditEventInput::v1(
            now,
            Some(tenant_id),
            None,
            session_id,
            Some(user_id),
            Some(device_id),
            AuditEngine::Ph1E,
            AuditEventType::ToolOk,
            reason_code,
            AuditSeverity::Info,
            correlation_id,
            turn_id,
            payload,
            None,
            Some(idempotency_key),
        )?;

        self.append_audit_event(input)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1e_tool_fail_commit(
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
        Self::validate_ph1e_tenant_id(&tenant_id)?;
        Self::validate_ph1e_idempotency("ph1e.idempotency_key", &idempotency_key)?;
        Self::validate_ph1e_bounded_text("ph1e.tool_name", &tool_name, 32)?;
        Self::validate_ph1e_bounded_text("ph1e.fail_code", &fail_code, 64)?;
        Self::validate_ph1e_bounded_text("ph1e.cache_status", &cache_status, 16)?;
        self.validate_ph1e_scope_and_bindings(&tenant_id, &user_id, &device_id, session_id)?;

        let payload = AuditPayloadMin::v1(BTreeMap::from([
            (PayloadKey::new("tool_name")?, PayloadValue::new(tool_name)?),
            (PayloadKey::new("fail_code")?, PayloadValue::new(fail_code)?),
            (
                PayloadKey::new("cache_status")?,
                PayloadValue::new(cache_status)?,
            ),
        ]))?;

        let input = AuditEventInput::v1(
            now,
            Some(tenant_id),
            None,
            session_id,
            Some(user_id),
            Some(device_id),
            AuditEngine::Ph1E,
            AuditEventType::ToolFail,
            reason_code,
            AuditSeverity::Warn,
            correlation_id,
            turn_id,
            payload,
            None,
            Some(idempotency_key),
        )?;

        self.append_audit_event(input)
    }

    pub fn ph1e_audit_rows(&self, correlation_id: CorrelationId) -> Vec<&AuditEvent> {
        self.audit_events
            .iter()
            .filter(|e| e.correlation_id == correlation_id && e.engine == AuditEngine::Ph1E)
            .collect()
    }

    // ------------------------
    // PH1.PERSONA (Per-User Personalization Profile)
    // ------------------------

    fn validate_ph1persona_tenant_id(tenant_id: &str) -> Result<(), StorageError> {
        if tenant_id.trim().is_empty() || tenant_id.len() > 64 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1persona.tenant_id",
                    reason: "must be non-empty and <= 64 chars",
                },
            ));
        }
        Ok(())
    }

    fn validate_ph1persona_idempotency(
        field: &'static str,
        idempotency_key: &str,
    ) -> Result<(), StorageError> {
        if idempotency_key.trim().is_empty() || idempotency_key.len() > 128 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field,
                    reason: "must be non-empty and <= 128 chars",
                },
            ));
        }
        Ok(())
    }

    fn validate_ph1persona_bounded_text(
        field: &'static str,
        value: &str,
        max_len: usize,
    ) -> Result<(), StorageError> {
        if value.trim().is_empty() || value.len() > max_len {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field,
                    reason: "must be non-empty and within max length",
                },
            ));
        }
        Ok(())
    }

    fn validate_ph1persona_scope_and_bindings(
        &mut self,
        tenant_id: &str,
        user_id: &UserId,
        device_id: &DeviceId,
        session_id: Option<SessionId>,
    ) -> Result<(), StorageError> {
        if !self.identities.contains_key(user_id) {
            return Err(StorageError::ForeignKeyViolation {
                table: "audit_events.user_id",
                key: user_id.as_str().to_string(),
            });
        }

        let dev = self
            .devices
            .get(device_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "audit_events.device_id",
                key: device_id.as_str().to_string(),
            })?;
        if dev.user_id != *user_id {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1persona.device_id",
                    reason: "device must belong to user_id",
                },
            ));
        }

        if let Some(sid) = session_id {
            let session = self
                .sessions
                .get(&sid)
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "audit_events.session_id",
                    key: sid.0.to_string(),
                })?;
            if session.user_id != *user_id || session.device_id != *device_id {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1persona.session_scope",
                        reason: "session must match user_id and device_id",
                    },
                ));
            }
        }

        if let Some(bound_tenant_id) = self.ph1persona_device_tenant_bindings.get(device_id) {
            if *bound_tenant_id != tenant_id {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1persona.tenant_id",
                        reason: "must match device tenant binding",
                    },
                ));
            }
        } else {
            self.ph1persona_device_tenant_bindings
                .insert(device_id.clone(), tenant_id.to_string());
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1persona_profile_commit(
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
        Self::validate_ph1persona_tenant_id(&tenant_id)?;
        Self::validate_ph1persona_idempotency("ph1persona.idempotency_key", &idempotency_key)?;
        Self::validate_ph1persona_bounded_text(
            "ph1persona.style_profile_ref",
            &style_profile_ref,
            64,
        )?;
        Self::validate_ph1persona_bounded_text(
            "ph1persona.delivery_policy_ref",
            &delivery_policy_ref,
            32,
        )?;
        Self::validate_ph1persona_bounded_text(
            "ph1persona.preferences_snapshot_ref",
            &preferences_snapshot_ref,
            96,
        )?;
        self.validate_ph1persona_scope_and_bindings(&tenant_id, &user_id, &device_id, session_id)?;

        let payload = AuditPayloadMin::v1(BTreeMap::from([
            (
                PayloadKey::new("style_profile_ref")?,
                PayloadValue::new(style_profile_ref)?,
            ),
            (
                PayloadKey::new("delivery_policy_ref")?,
                PayloadValue::new(delivery_policy_ref)?,
            ),
            (
                PayloadKey::new("preferences_snapshot_ref")?,
                PayloadValue::new(preferences_snapshot_ref)?,
            ),
        ]))?;

        let input = AuditEventInput::v1(
            now,
            Some(tenant_id),
            None,
            session_id,
            Some(user_id),
            Some(device_id),
            AuditEngine::Other("PH1.PERSONA".to_string()),
            AuditEventType::Other,
            reason_code,
            AuditSeverity::Info,
            correlation_id,
            turn_id,
            payload,
            None,
            Some(idempotency_key),
        )?;

        self.append_audit_event(input)
    }

    pub fn ph1persona_audit_rows(&self, correlation_id: CorrelationId) -> Vec<&AuditEvent> {
        self.audit_events
            .iter()
            .filter(|e| {
                e.correlation_id == correlation_id
                    && matches!(&e.engine, AuditEngine::Other(name) if name == "PH1.PERSONA")
            })
            .collect()
    }

    // ------------------------
    // PH1.FEEDBACK / PH1.LEARN / PH1.KNOW (Learning Layer)
    // ------------------------

    fn validate_ph1learn_tenant_id(tenant_id: &str) -> Result<(), StorageError> {
        if tenant_id.trim().is_empty() || tenant_id.len() > 64 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1learn.tenant_id",
                    reason: "must be non-empty and <= 64 chars",
                },
            ));
        }
        Ok(())
    }

    fn validate_ph1learn_idempotency(
        field: &'static str,
        idempotency_key: &str,
    ) -> Result<(), StorageError> {
        if idempotency_key.trim().is_empty() || idempotency_key.len() > 128 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field,
                    reason: "must be non-empty and <= 128 chars",
                },
            ));
        }
        Ok(())
    }

    fn validate_ph1learn_bounded_text(
        field: &'static str,
        value: &str,
        max_len: usize,
    ) -> Result<(), StorageError> {
        if value.trim().is_empty() || value.len() > max_len {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field,
                    reason: "must be non-empty and within max length",
                },
            ));
        }
        Ok(())
    }

    fn validate_ph1feedback_scope_and_bindings(
        &mut self,
        tenant_id: &str,
        user_id: &UserId,
        device_id: &DeviceId,
        session_id: Option<SessionId>,
    ) -> Result<(), StorageError> {
        if !self.identities.contains_key(user_id) {
            return Err(StorageError::ForeignKeyViolation {
                table: "audit_events.user_id",
                key: user_id.as_str().to_string(),
            });
        }

        let dev = self
            .devices
            .get(device_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "audit_events.device_id",
                key: device_id.as_str().to_string(),
            })?;
        if dev.user_id != *user_id {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1feedback.device_id",
                    reason: "device must belong to user_id",
                },
            ));
        }

        if let Some(sid) = session_id {
            let session = self
                .sessions
                .get(&sid)
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "audit_events.session_id",
                    key: sid.0.to_string(),
                })?;
            if session.user_id != *user_id || session.device_id != *device_id {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1feedback.session_scope",
                        reason: "session must match user_id and device_id",
                    },
                ));
            }
        }

        if let Some(bound_tenant_id) = self.ph1feedback_device_tenant_bindings.get(device_id) {
            if *bound_tenant_id != tenant_id {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1feedback.tenant_id",
                        reason: "must match device tenant binding",
                    },
                ));
            }
        } else {
            self.ph1feedback_device_tenant_bindings
                .insert(device_id.clone(), tenant_id.to_string());
        }

        Ok(())
    }

    fn validate_ph1learn_scope_and_bindings(
        &mut self,
        tenant_id: &str,
        scope_type: ArtifactScopeType,
        scope_id: &str,
    ) -> Result<(), StorageError> {
        match scope_type {
            ArtifactScopeType::Tenant => {
                if scope_id != tenant_id {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "ph1learn.scope_id",
                            reason: "tenant scope_id must equal tenant_id",
                        },
                    ));
                }
            }
            ArtifactScopeType::User => {
                let user_id = UserId::new(scope_id.to_string()).map_err(|_| {
                    StorageError::ContractViolation(ContractViolation::InvalidValue {
                        field: "ph1learn.scope_id",
                        reason: "user scope_id must be a valid user_id",
                    })
                })?;
                if !self.identities.contains_key(&user_id) {
                    return Err(StorageError::ForeignKeyViolation {
                        table: "artifacts_ledger.scope_id(user_id)",
                        key: scope_id.to_string(),
                    });
                }

                if let Some(bound_tenant_id) = self.ph1learn_user_tenant_bindings.get(&user_id) {
                    if *bound_tenant_id != tenant_id {
                        return Err(StorageError::ContractViolation(
                            ContractViolation::InvalidValue {
                                field: "ph1learn.tenant_id",
                                reason: "must match user tenant binding",
                            },
                        ));
                    }
                } else {
                    self.ph1learn_user_tenant_bindings
                        .insert(user_id, tenant_id.to_string());
                }
            }
            ArtifactScopeType::Device => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1learn.scope_type",
                        reason: "device scope is out of row-25 lock scope",
                    },
                ));
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1feedback_event_commit(
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
        Self::validate_ph1learn_tenant_id(&tenant_id)?;
        Self::validate_ph1learn_idempotency("ph1feedback.idempotency_key", &idempotency_key)?;
        Self::validate_ph1learn_bounded_text(
            "ph1feedback.feedback_event_type",
            &feedback_event_type,
            64,
        )?;
        Self::validate_ph1learn_bounded_text("ph1feedback.signal_bucket", &signal_bucket, 32)?;
        self.validate_ph1feedback_scope_and_bindings(&tenant_id, &user_id, &device_id, session_id)?;

        let payload = AuditPayloadMin::v1(BTreeMap::from([
            (
                PayloadKey::new("feedback_event_type")?,
                PayloadValue::new(feedback_event_type)?,
            ),
            (
                PayloadKey::new("signal_bucket")?,
                PayloadValue::new(signal_bucket)?,
            ),
        ]))?;

        let input = AuditEventInput::v1(
            now,
            Some(tenant_id),
            None,
            session_id,
            Some(user_id),
            Some(device_id),
            AuditEngine::Other("PH1.FEEDBACK".to_string()),
            AuditEventType::Other,
            reason_code,
            AuditSeverity::Info,
            correlation_id,
            turn_id,
            payload,
            None,
            Some(idempotency_key),
        )?;

        self.append_audit_event(input)
    }

    pub fn ph1feedback_audit_rows(&self, correlation_id: CorrelationId) -> Vec<&AuditEvent> {
        self.audit_events
            .iter()
            .filter(|e| {
                e.correlation_id == correlation_id
                    && matches!(&e.engine, AuditEngine::Other(name) if name == "PH1.FEEDBACK")
            })
            .collect()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1learn_artifact_commit(
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
        Self::validate_ph1learn_tenant_id(&tenant_id)?;
        Self::validate_ph1learn_idempotency("ph1learn.idempotency_key", &idempotency_key)?;
        self.validate_ph1learn_scope_and_bindings(&tenant_id, scope_type, &scope_id)?;

        let input = ArtifactLedgerRowInput::v1(
            now,
            scope_type,
            scope_id,
            artifact_type,
            artifact_version,
            package_hash,
            payload_ref,
            "PH1.LEARN".to_string(),
            provenance_ref,
            status,
            Some(idempotency_key),
        )?;

        self.append_artifact_ledger_row(input)
    }

    pub fn ph1learn_artifact_rows(
        &self,
        scope_type: ArtifactScopeType,
        scope_id: &str,
        artifact_type: ArtifactType,
    ) -> Vec<&ArtifactLedgerRow> {
        self.artifacts_ledger_rows
            .iter()
            .filter(|row| {
                row.scope_type == scope_type
                    && row.scope_id == scope_id
                    && row.artifact_type == artifact_type
                    && row.created_by == "PH1.LEARN"
            })
            .collect()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1know_dictionary_pack_commit(
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
        Self::validate_ph1learn_tenant_id(&tenant_id)?;
        Self::validate_ph1learn_idempotency("ph1know.idempotency_key", &idempotency_key)?;
        if !matches!(
            artifact_type,
            ArtifactType::SttVocabPack | ArtifactType::TtsPronunciationPack
        ) {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1know.artifact_type",
                    reason: "must be STT_VOCAB_PACK or TTS_PRONUNCIATION_PACK",
                },
            ));
        }
        self.validate_ph1learn_scope_and_bindings(
            &tenant_id,
            ArtifactScopeType::Tenant,
            &tenant_id,
        )?;

        let input = ArtifactLedgerRowInput::v1(
            now,
            ArtifactScopeType::Tenant,
            tenant_id.clone(),
            artifact_type,
            artifact_version,
            package_hash,
            payload_ref,
            "PH1.KNOW".to_string(),
            provenance_ref,
            ArtifactStatus::Active,
            Some(idempotency_key),
        )?;

        self.append_artifact_ledger_row(input)
    }

    pub fn ph1know_artifact_rows(
        &self,
        tenant_id: &str,
        artifact_type: ArtifactType,
    ) -> Vec<&ArtifactLedgerRow> {
        self.artifacts_ledger_rows
            .iter()
            .filter(|row| {
                row.scope_type == ArtifactScopeType::Tenant
                    && row.scope_id == tenant_id
                    && row.artifact_type == artifact_type
                    && row.created_by == "PH1.KNOW"
            })
            .collect()
    }

    // ------------------------
    // PH1.K (Voice Runtime I/O)
    // ------------------------

    fn validate_ph1k_tenant_id(tenant_id: &str) -> Result<(), StorageError> {
        if tenant_id.trim().is_empty() || tenant_id.len() > 64 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "audio_runtime_events.tenant_id",
                    reason: "must be non-empty and <= 64 chars",
                },
            ));
        }
        Ok(())
    }

    fn validate_ph1k_idempotency(idempotency_key: &str) -> Result<(), StorageError> {
        if idempotency_key.trim().is_empty() || idempotency_key.len() > 128 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "audio_runtime_events.idempotency_key",
                    reason: "must be non-empty and <= 128 chars",
                },
            ));
        }
        Ok(())
    }

    fn validate_ph1k_optional_text(
        field: &'static str,
        value: &Option<String>,
        max_len: usize,
    ) -> Result<(), StorageError> {
        if let Some(v) = value {
            if v.trim().is_empty() || v.len() > max_len {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field,
                        reason: "must be non-empty and within max length",
                    },
                ));
            }
        }
        Ok(())
    }

    fn validate_nonnegative_f32(field: &'static str, value: f32) -> Result<(), StorageError> {
        if !value.is_finite() {
            return Err(StorageError::ContractViolation(
                ContractViolation::NotFinite { field },
            ));
        }
        if value < 0.0 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field,
                    reason: "must be >= 0",
                },
            ));
        }
        Ok(())
    }

    fn quantize_milli(v: f32) -> i64 {
        (v * 1000.0).round() as i64
    }

    fn apply_ph1k_event_to_current(&mut self, event: &Ph1kRuntimeEventRecord) {
        let key = (event.tenant_id.clone(), event.device_id.clone());
        let row =
            self.ph1k_runtime_current
                .entry(key)
                .or_insert_with(|| Ph1kRuntimeCurrentRecord {
                    schema_version: SchemaVersion(1),
                    tenant_id: event.tenant_id.clone(),
                    device_id: event.device_id.clone(),
                    session_id: event.session_id.clone(),
                    processed_stream_id: None,
                    raw_stream_id: None,
                    pre_roll_buffer_id: None,
                    selected_mic: None,
                    selected_speaker: None,
                    device_health: None,
                    jitter_ms: None,
                    drift_ppm: None,
                    buffer_depth_ms: None,
                    underruns: None,
                    overruns: None,
                    tts_playback_active: false,
                    capture_degraded: false,
                    aec_unstable: false,
                    device_changed: false,
                    stream_gap_detected: false,
                    last_interrupt_phrase: None,
                    last_interrupt_reason_code: None,
                    last_event_id: event.event_id,
                    updated_at: event.created_at,
                });

        if event.session_id.is_some() {
            row.session_id = event.session_id.clone();
        }
        row.last_event_id = event.event_id;
        row.updated_at = event.created_at;

        match event.event_kind {
            Ph1kRuntimeEventKind::StreamRefs => {
                row.processed_stream_id = event.processed_stream_id;
                row.raw_stream_id = event.raw_stream_id;
                row.pre_roll_buffer_id = event.pre_roll_buffer_id;
            }
            Ph1kRuntimeEventKind::VadEvent => {
                row.processed_stream_id = event.processed_stream_id;
            }
            Ph1kRuntimeEventKind::DeviceState => {
                row.selected_mic = event.selected_mic.clone();
                row.selected_speaker = event.selected_speaker.clone();
                row.device_health = event.device_health;
            }
            Ph1kRuntimeEventKind::TimingStats => {
                row.jitter_ms = event.jitter_ms.map(Self::quantize_milli);
                row.drift_ppm = event.drift_ppm.map(Self::quantize_milli);
                row.buffer_depth_ms = event.buffer_depth_ms.map(Self::quantize_milli);
                row.underruns = event.underruns;
                row.overruns = event.overruns;
            }
            Ph1kRuntimeEventKind::InterruptCandidate => {
                row.last_interrupt_phrase = event.phrase_text.clone();
                row.last_interrupt_reason_code = event.reason_code;
            }
            Ph1kRuntimeEventKind::DegradationFlags => {
                row.capture_degraded = event.capture_degraded.unwrap_or(false);
                row.aec_unstable = event.aec_unstable.unwrap_or(false);
                row.device_changed = event.device_changed.unwrap_or(false);
                row.stream_gap_detected = event.stream_gap_detected.unwrap_or(false);
            }
            Ph1kRuntimeEventKind::TtsPlaybackActive => {
                row.tts_playback_active = event.tts_playback_active.unwrap_or(false);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1k_runtime_event_commit(
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
        Self::validate_ph1k_tenant_id(&tenant_id)?;
        Self::validate_ph1k_idempotency(&idempotency_key)?;
        Self::validate_ph1k_optional_text("audio_runtime_events.selected_mic", &selected_mic, 128)?;
        Self::validate_ph1k_optional_text(
            "audio_runtime_events.selected_speaker",
            &selected_speaker,
            128,
        )?;
        Self::validate_ph1k_optional_text("audio_runtime_events.phrase_text", &phrase_text, 128)?;

        if !self.devices.contains_key(&device_id) {
            return Err(StorageError::ForeignKeyViolation {
                table: "audio_runtime_events.device_id",
                key: device_id.as_str().to_string(),
            });
        }
        if let Some(sid) = session_id.as_ref() {
            if !self.sessions.contains_key(sid) {
                return Err(StorageError::ForeignKeyViolation {
                    table: "audio_runtime_events.session_id",
                    key: sid.0.to_string(),
                });
            }
        }

        if let Some(bound_tenant_id) = self.ph1k_device_tenant_bindings.get(&device_id) {
            if *bound_tenant_id != tenant_id {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "audio_runtime_events.tenant_id",
                        reason: "must match device tenant binding",
                    },
                ));
            }
        } else {
            self.ph1k_device_tenant_bindings
                .insert(device_id.clone(), tenant_id.clone());
        }

        match event_kind {
            Ph1kRuntimeEventKind::StreamRefs => {
                if processed_stream_id.is_none() {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "audio_runtime_events.processed_stream_id",
                            reason: "required for STREAM_REFS",
                        },
                    ));
                }
                if pre_roll_buffer_id.is_none() {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "audio_runtime_events.pre_roll_buffer_id",
                            reason: "required for STREAM_REFS",
                        },
                    ));
                }
            }
            Ph1kRuntimeEventKind::VadEvent => {
                if processed_stream_id.is_none() {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "audio_runtime_events.processed_stream_id",
                            reason: "required for VAD_EVENT",
                        },
                    ));
                }
            }
            Ph1kRuntimeEventKind::DeviceState => {
                if selected_mic.is_none() || selected_speaker.is_none() || device_health.is_none() {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "audio_runtime_events.device_state",
                            reason:
                                "selected_mic, selected_speaker, and device_health are required",
                        },
                    ));
                }
            }
            Ph1kRuntimeEventKind::TimingStats => {
                let (Some(j), Some(d), Some(b), Some(_u), Some(_o)) =
                    (jitter_ms, drift_ppm, buffer_depth_ms, underruns, overruns)
                else {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "audio_runtime_events.timing_stats",
                            reason: "timing fields are required",
                        },
                    ));
                };
                Self::validate_nonnegative_f32("audio_runtime_events.jitter_ms", j)?;
                Self::validate_nonnegative_f32("audio_runtime_events.drift_ppm", d)?;
                Self::validate_nonnegative_f32("audio_runtime_events.buffer_depth_ms", b)?;
            }
            Ph1kRuntimeEventKind::InterruptCandidate => {
                if phrase_id.is_none() || phrase_text.is_none() || reason_code.is_none() {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "audio_runtime_events.interrupt_candidate",
                            reason: "phrase_id, phrase_text, and reason_code are required",
                        },
                    ));
                }
            }
            Ph1kRuntimeEventKind::DegradationFlags => {
                if capture_degraded.is_none()
                    || aec_unstable.is_none()
                    || device_changed.is_none()
                    || stream_gap_detected.is_none()
                {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "audio_runtime_events.degradation_flags",
                            reason:
                                "capture_degraded, aec_unstable, device_changed, and stream_gap_detected are required",
                        },
                    ));
                }
            }
            Ph1kRuntimeEventKind::TtsPlaybackActive => {
                if tts_playback_active.is_none() {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "audio_runtime_events.tts_playback_active",
                            reason: "required for TTS_PLAYBACK_ACTIVE",
                        },
                    ));
                }
            }
        }

        let idx = (
            tenant_id.clone(),
            device_id.clone(),
            event_kind,
            idempotency_key.clone(),
        );
        if let Some(existing_event_id) = self.ph1k_runtime_event_idempotency_index.get(&idx) {
            if let Some(existing) = self
                .ph1k_runtime_events
                .iter()
                .find(|row| row.event_id == *existing_event_id)
            {
                return Ok(existing.clone());
            }
            return Err(StorageError::ForeignKeyViolation {
                table: "audio_runtime_events.event_id",
                key: existing_event_id.to_string(),
            });
        }

        let event_id = self.next_ph1k_runtime_event_id;
        self.next_ph1k_runtime_event_id = self.next_ph1k_runtime_event_id.saturating_add(1);

        let row = Ph1kRuntimeEventRecord {
            schema_version: SchemaVersion(1),
            event_id,
            tenant_id: tenant_id.clone(),
            device_id: device_id.clone(),
            session_id: session_id.clone(),
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
            created_at: now,
        };

        self.ph1k_runtime_events.push(row.clone());
        self.ph1k_runtime_event_idempotency_index
            .insert(idx, event_id);
        self.apply_ph1k_event_to_current(&row);
        Ok(row)
    }

    pub fn ph1k_runtime_event_rows(&self) -> &[Ph1kRuntimeEventRecord] {
        &self.ph1k_runtime_events
    }

    pub fn ph1k_runtime_current_rows(
        &self,
    ) -> &BTreeMap<(String, DeviceId), Ph1kRuntimeCurrentRecord> {
        &self.ph1k_runtime_current
    }

    pub fn ph1k_runtime_current_row(
        &self,
        tenant_id: &str,
        device_id: &DeviceId,
    ) -> Option<&Ph1kRuntimeCurrentRecord> {
        self.ph1k_runtime_current
            .get(&(tenant_id.to_string(), device_id.clone()))
    }

    pub fn rebuild_ph1k_runtime_current_from_ledger(&mut self) {
        self.ph1k_runtime_current.clear();
        self.ph1k_device_tenant_bindings.clear();
        let rows = self.ph1k_runtime_events.clone();
        for row in &rows {
            self.ph1k_device_tenant_bindings
                .insert(row.device_id.clone(), row.tenant_id.clone());
            self.apply_ph1k_event_to_current(row);
        }
    }

    pub fn attempt_overwrite_ph1k_runtime_event(
        &mut self,
        _event_id: u64,
    ) -> Result<(), StorageError> {
        Err(StorageError::AppendOnlyViolation {
            table: "audio_runtime_events",
        })
    }

    // ------------------------
    // PH1.ONB.BIZ + PH1.POSITION (tenant/company + position truth)
    // ------------------------

    pub fn ph1tenant_company_upsert(
        &mut self,
        record: TenantCompanyRecord,
    ) -> Result<(), StorageError> {
        record.tenant_id.validate()?;
        if record.company_id.trim().is_empty() || record.company_id.len() > 64 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "tenant_company_record.company_id",
                    reason: "must be non-empty and <= 64 chars",
                },
            ));
        }
        if record.legal_name.trim().is_empty() || record.legal_name.len() > 128 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "tenant_company_record.legal_name",
                    reason: "must be non-empty and <= 128 chars",
                },
            ));
        }
        if record.jurisdiction.trim().is_empty() || record.jurisdiction.len() > 64 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "tenant_company_record.jurisdiction",
                    reason: "must be non-empty and <= 64 chars",
                },
            ));
        }
        self.tenant_companies.insert(
            (record.tenant_id.clone(), record.company_id.clone()),
            record,
        );
        Ok(())
    }

    pub fn ph1tenant_company_get(
        &self,
        tenant_id: &TenantId,
        company_id: &str,
    ) -> Option<&TenantCompanyRecord> {
        self.tenant_companies
            .get(&(tenant_id.clone(), company_id.to_string()))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ph1position_create_draft(
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
        if !self.identities.contains_key(&actor_user_id) {
            return Err(StorageError::ForeignKeyViolation {
                table: "positions.actor_user_id",
                key: actor_user_id.as_str().to_string(),
            });
        }
        if idempotency_key.trim().is_empty() || idempotency_key.len() > 128 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1position_create_draft.idempotency_key",
                    reason: "must be non-empty and <= 128 chars",
                },
            ));
        }
        if simulation_id.trim().is_empty() || simulation_id.len() > 96 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1position_create_draft.simulation_id",
                    reason: "must be non-empty and <= 96 chars",
                },
            ));
        }

        let company = self
            .tenant_companies
            .get(&(tenant_id.clone(), company_id.clone()))
            .ok_or(StorageError::ForeignKeyViolation {
                table: "positions.company_id",
                key: company_id.clone(),
            })?;
        if company.lifecycle_state != TenantCompanyLifecycleState::Active {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1position_create_draft.company_state",
                    reason: "company must be ACTIVE",
                },
            ));
        }

        let create_idx = (
            tenant_id.clone(),
            company_id.clone(),
            position_title.clone(),
            department.clone(),
            jurisdiction.clone(),
            idempotency_key.clone(),
        );
        if let Some(existing_id) = self.position_create_idempotency_index.get(&create_idx) {
            let existing = self
                .positions
                .get(&(tenant_id.clone(), existing_id.clone()))
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "positions.position_id",
                    key: existing_id.as_str().to_string(),
                })?;
            return Ok(existing.clone());
        }

        if self.positions.values().any(|p| {
            p.tenant_id == tenant_id
                && p.company_id == company_id
                && p.position_title == position_title
                && p.department == department
                && p.jurisdiction == jurisdiction
        }) {
            return Err(StorageError::DuplicateKey {
                table: "positions(tenant_id,company_id,position_title,department,jurisdiction)",
                key: format!(
                    "{}:{}:{}:{}:{}",
                    tenant_id.as_str(),
                    company_id,
                    position_title,
                    department,
                    jurisdiction
                ),
            });
        }

        let position_id = PositionId::new(format!(
            "pos_{}",
            hash_hex_64(&format!(
                "{}:{}:{}:{}:{}",
                tenant_id.as_str(),
                company_id,
                position_title,
                department,
                jurisdiction
            ))
        ))
        .map_err(StorageError::ContractViolation)?;

        let rec = PositionRecord::v1(
            tenant_id.clone(),
            company_id,
            position_id.clone(),
            position_title,
            department,
            jurisdiction,
            schedule_type,
            permission_profile_ref,
            compensation_band_ref,
            PositionLifecycleState::Draft,
            now,
            now,
        )?;

        self.positions
            .insert((tenant_id.clone(), position_id.clone()), rec.clone());
        self.position_create_idempotency_index
            .insert(create_idx, position_id.clone());

        self.append_position_lifecycle_event(
            now,
            tenant_id,
            position_id,
            PositionLifecycleAction::CreateDraft,
            PositionLifecycleState::Draft,
            PositionLifecycleState::Draft,
            reason_code,
            simulation_id.to_string(),
            actor_user_id,
            Some(idempotency_key),
        );

        Ok(rec)
    }

    pub fn ph1position_validate_auth_company_draft(
        &self,
        tenant_id: &TenantId,
        company_id: &str,
        position_id: &PositionId,
        requested_action: PositionRequestedAction,
    ) -> Result<(PositionValidationStatus, ReasonCodeId), StorageError> {
        let company = self
            .tenant_companies
            .get(&(tenant_id.clone(), company_id.to_string()))
            .ok_or(StorageError::ForeignKeyViolation {
                table: "positions.company_id",
                key: company_id.to_string(),
            })?;

        if company.lifecycle_state != TenantCompanyLifecycleState::Active {
            return Ok((PositionValidationStatus::Fail, ReasonCodeId(0x5900_0102)));
        }

        let rec = self
            .positions
            .get(&(tenant_id.clone(), position_id.clone()))
            .ok_or(StorageError::ForeignKeyViolation {
                table: "positions.position_id",
                key: position_id.as_str().to_string(),
            })?;

        if rec.company_id != company_id {
            return Ok((PositionValidationStatus::Fail, ReasonCodeId(0x5900_0103)));
        }

        let valid = match requested_action {
            PositionRequestedAction::Activate => matches!(
                rec.lifecycle_state,
                PositionLifecycleState::Draft | PositionLifecycleState::Suspended
            ),
            PositionRequestedAction::Suspend => {
                rec.lifecycle_state == PositionLifecycleState::Active
            }
            PositionRequestedAction::Retire => matches!(
                rec.lifecycle_state,
                PositionLifecycleState::Active | PositionLifecycleState::Suspended
            ),
        };
        if valid {
            Ok((PositionValidationStatus::Ok, ReasonCodeId(0x5900_0001)))
        } else {
            Ok((PositionValidationStatus::Fail, ReasonCodeId(0x5900_0104)))
        }
    }

    pub fn ph1position_band_policy_check_draft(
        &self,
        tenant_id: &TenantId,
        position_id: &PositionId,
        compensation_band_ref: &str,
    ) -> Result<(PositionPolicyResult, ReasonCodeId), StorageError> {
        let rec = self
            .positions
            .get(&(tenant_id.clone(), position_id.clone()))
            .ok_or(StorageError::ForeignKeyViolation {
                table: "positions.position_id",
                key: position_id.as_str().to_string(),
            })?;

        if rec.compensation_band_ref == compensation_band_ref {
            Ok((PositionPolicyResult::Allow, ReasonCodeId(0x5900_0002)))
        } else {
            Ok((PositionPolicyResult::Escalate, ReasonCodeId(0x5900_0105)))
        }
    }

    pub fn ph1position_activate_commit(
        &mut self,
        now: MonotonicTimeNs,
        actor_user_id: UserId,
        tenant_id: TenantId,
        position_id: PositionId,
        idempotency_key: String,
        simulation_id: &str,
        reason_code: ReasonCodeId,
    ) -> Result<PositionRecord, StorageError> {
        if !self.identities.contains_key(&actor_user_id) {
            return Err(StorageError::ForeignKeyViolation {
                table: "positions.actor_user_id",
                key: actor_user_id.as_str().to_string(),
            });
        }
        if idempotency_key.trim().is_empty() || idempotency_key.len() > 128 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1position_activate_commit.idempotency_key",
                    reason: "must be non-empty and <= 128 chars",
                },
            ));
        }

        let idx = (
            tenant_id.clone(),
            position_id.clone(),
            idempotency_key.clone(),
        );
        if self.position_activate_idempotency_index.contains_key(&idx) {
            return self
                .positions
                .get(&(tenant_id, position_id))
                .cloned()
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "positions.position_id",
                    key: "missing_after_idempotency".to_string(),
                });
        }

        let (from_state, out) = {
            let rec = self
                .positions
                .get_mut(&(tenant_id.clone(), position_id.clone()))
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "positions.position_id",
                    key: position_id.as_str().to_string(),
                })?;

            if !matches!(
                rec.lifecycle_state,
                PositionLifecycleState::Draft | PositionLifecycleState::Suspended
            ) {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1position_activate_commit.lifecycle_state",
                        reason: "must be Draft or Suspended",
                    },
                ));
            }

            let from_state = rec.lifecycle_state;
            rec.lifecycle_state = PositionLifecycleState::Active;
            rec.updated_at = now;
            (from_state, rec.clone())
        };

        self.position_activate_idempotency_index
            .insert(idx, PositionLifecycleState::Active);
        self.append_position_lifecycle_event(
            now,
            tenant_id,
            position_id,
            PositionLifecycleAction::Activate,
            from_state,
            PositionLifecycleState::Active,
            reason_code,
            simulation_id.to_string(),
            actor_user_id,
            Some(idempotency_key),
        );
        Ok(out)
    }

    pub fn ph1position_retire_or_suspend_commit(
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
        if !self.identities.contains_key(&actor_user_id) {
            return Err(StorageError::ForeignKeyViolation {
                table: "positions.actor_user_id",
                key: actor_user_id.as_str().to_string(),
            });
        }
        if !matches!(
            requested_state,
            PositionLifecycleState::Suspended | PositionLifecycleState::Retired
        ) {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1position_retire_or_suspend_commit.requested_state",
                    reason: "must be Suspended or Retired",
                },
            ));
        }
        if idempotency_key.trim().is_empty() || idempotency_key.len() > 128 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1position_retire_or_suspend_commit.idempotency_key",
                    reason: "must be non-empty and <= 128 chars",
                },
            ));
        }

        let idx = (
            tenant_id.clone(),
            position_id.clone(),
            requested_state,
            idempotency_key.clone(),
        );
        if self
            .position_retire_suspend_idempotency_index
            .contains_key(&idx)
        {
            return self
                .positions
                .get(&(tenant_id, position_id))
                .cloned()
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "positions.position_id",
                    key: "missing_after_idempotency".to_string(),
                });
        }

        let (from_state, out) = {
            let rec = self
                .positions
                .get_mut(&(tenant_id.clone(), position_id.clone()))
                .ok_or(StorageError::ForeignKeyViolation {
                    table: "positions.position_id",
                    key: position_id.as_str().to_string(),
                })?;

            if !matches!(
                rec.lifecycle_state,
                PositionLifecycleState::Active | PositionLifecycleState::Suspended
            ) {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1position_retire_or_suspend_commit.lifecycle_state",
                        reason: "must be Active or Suspended",
                    },
                ));
            }

            let from_state = rec.lifecycle_state;
            rec.lifecycle_state = requested_state;
            rec.updated_at = now;
            (from_state, rec.clone())
        };

        self.position_retire_suspend_idempotency_index
            .insert(idx, requested_state);

        self.append_position_lifecycle_event(
            now,
            tenant_id,
            position_id,
            match requested_state {
                PositionLifecycleState::Suspended => PositionLifecycleAction::Suspend,
                PositionLifecycleState::Retired => PositionLifecycleAction::Retire,
                _ => PositionLifecycleAction::PolicyOverride,
            },
            from_state,
            requested_state,
            reason_code,
            simulation_id.to_string(),
            actor_user_id,
            Some(idempotency_key),
        );

        Ok(out)
    }

    fn append_position_lifecycle_event(
        &mut self,
        now: MonotonicTimeNs,
        tenant_id: TenantId,
        position_id: PositionId,
        action: PositionLifecycleAction,
        from_state: PositionLifecycleState,
        to_state: PositionLifecycleState,
        reason_code: ReasonCodeId,
        simulation_id: String,
        actor_user_id: UserId,
        idempotency_key: Option<String>,
    ) {
        let event_id = self.next_position_lifecycle_event_id;
        self.next_position_lifecycle_event_id =
            self.next_position_lifecycle_event_id.saturating_add(1);
        self.position_lifecycle_events
            .push(PositionLifecycleEventRecord {
                schema_version: SchemaVersion(1),
                event_id,
                tenant_id,
                position_id,
                action,
                from_state,
                to_state,
                reason_code,
                simulation_id,
                actor_user_id,
                created_at: now,
                idempotency_key,
            });
    }

    pub fn ph1position_upsert(&mut self, record: PositionRecord) -> Result<(), StorageError> {
        record.validate()?;
        self.positions.insert(
            (record.tenant_id.clone(), record.position_id.clone()),
            record,
        );
        Ok(())
    }

    pub fn ph1position_get(
        &self,
        tenant_id: &TenantId,
        position_id: &PositionId,
    ) -> Option<&PositionRecord> {
        self.positions
            .get(&(tenant_id.clone(), position_id.clone()))
    }

    pub fn ph1position_get_lifecycle_events_for_position(
        &self,
        tenant_id: &TenantId,
        position_id: &PositionId,
    ) -> Vec<&PositionLifecycleEventRecord> {
        self.position_lifecycle_events
            .iter()
            .filter(|e| &e.tenant_id == tenant_id && &e.position_id == position_id)
            .collect()
    }

    pub fn attempt_overwrite_position_lifecycle_event(
        &mut self,
        _event_id: u64,
    ) -> Result<(), StorageError> {
        Err(StorageError::AppendOnlyViolation {
            table: "position_lifecycle_events",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkGenerateResultParts {
    pub payload_hash: String,
    pub was_new: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1_voice_id::{
        DiarizationSegment, SpeakerAssertionOk, SpeakerLabel,
    };
    use selene_kernel_contracts::ph1d::{PolicyContextRef, SafetyTier};
    use selene_kernel_contracts::ph1f::{ConversationRole, ConversationSource, PrivacyScope};
    use selene_kernel_contracts::ph1j::{
        AuditEngine, AuditEventType, AuditPayloadMin, AuditSeverity, DeviceId, PayloadKey,
        PayloadValue,
    };
    use selene_kernel_contracts::ph1m::{MemoryConsent, MemoryLedgerEvent, MemoryLedgerEventKind};
    use selene_kernel_contracts::ReasonCodeId;

    fn user() -> UserId {
        UserId::new("user_1").unwrap()
    }

    fn device() -> DeviceId {
        DeviceId::new("device_1").unwrap()
    }

    fn store_with_user_and_device() -> Ph1fStore {
        let mut s = Ph1fStore::new_in_memory();
        s.insert_identity(IdentityRecord::v1(
            user(),
            Some(SpeakerId::new("spk_1").unwrap()),
            Some(LanguageTag::new("en-US").unwrap()),
            MonotonicTimeNs(1),
            IdentityStatus::Active,
        ))
        .unwrap();

        s.insert_device(
            DeviceRecord::v1(
                device(),
                user(),
                "desktop".to_string(),
                MonotonicTimeNs(2),
                None,
            )
            .unwrap(),
        )
        .unwrap();
        s
    }

    fn store_with_user_device_and_session() -> Ph1fStore {
        let mut s = store_with_user_and_device();
        s.insert_session(
            SessionRecord::v1(
                SessionId(1),
                user(),
                device(),
                SessionState::Open,
                MonotonicTimeNs(3),
                MonotonicTimeNs(3),
                None,
            )
            .unwrap(),
        )
        .unwrap();
        s
    }

    fn insert_onboarding_session(
        s: &mut Ph1fStore,
        onboarding_session_id: &str,
    ) -> OnboardingSessionId {
        let onb_id = OnboardingSessionId::new(onboarding_session_id).unwrap();
        s.onboarding_sessions.insert(
            onb_id.clone(),
            OnboardingSessionRecord {
                schema_version: SchemaVersion(1),
                onboarding_session_id: onb_id.clone(),
                link_id: LinkId::new("link_1").unwrap(),
                invitee_type: InviteeType::Employee,
                tenant_id: Some("tenant_1".to_string()),
                prefilled_context_ref: None,
                device_fingerprint_hash: "fp_hash_1".to_string(),
                status: OnboardingStatus::DraftCreated,
                created_at: MonotonicTimeNs(5),
                updated_at: MonotonicTimeNs(5),
                terms_version_id: None,
                terms_status: None,
                photo_blob_ref: None,
                photo_proof_ref: None,
                sender_user_id: Some(user()),
                verification_status: None,
                primary_device_device_id: None,
                primary_device_proof_type: None,
                primary_device_confirmed: false,
                access_engine_instance_id: None,
            },
        );
        onb_id
    }

    fn mem_event(kind: MemoryLedgerEventKind, key: &str, value: Option<&str>) -> MemoryLedgerEvent {
        MemoryLedgerEvent::v1(
            kind,
            MonotonicTimeNs(10),
            MemoryKey::new(key).unwrap(),
            value.map(|v| MemoryValue::v1(v.to_string(), None).unwrap()),
            Some("evidence".to_string()),
            MemoryProvenance::v1(Some(SessionId(1)), None).unwrap(),
            MemoryLayer::LongTerm,
            MemorySensitivityFlag::Low,
            MemoryConfidence::High,
            MemoryConsent::NotRequested,
            ReasonCodeId(1),
        )
        .unwrap()
    }

    #[test]
    fn at_f_01_ledger_append_only() {
        let mut s = store_with_user_and_device();
        let _ = s
            .append_memory_ledger_event(
                &user(),
                mem_event(MemoryLedgerEventKind::Stored, "k", Some("v")),
                MemoryUsePolicy::AlwaysUsable,
                None,
                None,
            )
            .unwrap();
        assert!(matches!(
            s.attempt_overwrite_memory_ledger_row(1),
            Err(StorageError::AppendOnlyViolation { .. })
        ));
    }

    #[test]
    fn at_f_02_current_state_rebuild_matches() {
        let mut s = store_with_user_and_device();
        s.append_memory_ledger_event(
            &user(),
            mem_event(
                MemoryLedgerEventKind::Stored,
                "preferred_name",
                Some("John"),
            ),
            MemoryUsePolicy::AlwaysUsable,
            None,
            None,
        )
        .unwrap();
        s.append_memory_ledger_event(
            &user(),
            mem_event(
                MemoryLedgerEventKind::Updated,
                "preferred_name",
                Some("John P."),
            ),
            MemoryUsePolicy::AlwaysUsable,
            None,
            None,
        )
        .unwrap();

        let before = s.memory_current().clone();
        s.rebuild_memory_current_from_ledger();
        let after = s.memory_current().clone();
        assert_eq!(before, after);
    }

    #[test]
    fn at_f_03_forget_writes_ledger_and_deactivates_current() {
        let mut s = store_with_user_and_device();
        s.append_memory_ledger_event(
            &user(),
            mem_event(
                MemoryLedgerEventKind::Stored,
                "micro:name:benji",
                Some("Benji"),
            ),
            MemoryUsePolicy::RepeatedOrConfirmed,
            Some(MonotonicTimeNs(999)),
            None,
        )
        .unwrap();
        s.append_memory_ledger_event(
            &user(),
            mem_event(MemoryLedgerEventKind::Forgotten, "micro:name:benji", None),
            MemoryUsePolicy::RepeatedOrConfirmed,
            None,
            None,
        )
        .unwrap();

        let key = (user(), MemoryKey::new("micro:name:benji").unwrap());
        let rec = s.memory_current().get(&key).unwrap();
        assert!(!rec.active);
        assert!(rec.memory_value.is_none());

        assert!(s
            .memory_ledger_rows()
            .iter()
            .any(|r| r.event.kind == MemoryLedgerEventKind::Forgotten));
    }

    #[test]
    fn at_f_04_session_integrity_fk_constraints() {
        let mut s = Ph1fStore::new_in_memory();
        // Missing identity/device => rejected.
        let res = s.insert_session(
            SessionRecord::v1(
                SessionId(1),
                UserId::new("missing_user").unwrap(),
                DeviceId::new("missing_device").unwrap(),
                SessionState::Open,
                MonotonicTimeNs(0),
                MonotonicTimeNs(0),
                None,
            )
            .unwrap(),
        );
        assert!(matches!(res, Err(StorageError::ForeignKeyViolation { .. })));
    }

    #[test]
    fn at_f_05_multilingual_text_preserved() {
        let mut s = store_with_user_and_device();
        let mixed = "remind me to call 妈妈";
        let ev = MemoryLedgerEvent::v1(
            MemoryLedgerEventKind::Stored,
            MonotonicTimeNs(10),
            MemoryKey::new("micro:call_target").unwrap(),
            Some(MemoryValue::v1("妈妈".to_string(), None).unwrap()),
            Some(mixed.to_string()),
            MemoryProvenance::v1(None, None).unwrap(),
            MemoryLayer::Micro,
            MemorySensitivityFlag::Low,
            MemoryConfidence::High,
            MemoryConsent::NotRequested,
            ReasonCodeId(1),
        )
        .unwrap();
        s.append_memory_ledger_event(
            &user(),
            ev,
            MemoryUsePolicy::RepeatedOrConfirmed,
            None,
            None,
        )
        .unwrap();

        let key = (user(), MemoryKey::new("micro:call_target").unwrap());
        let rec = s.memory_current().get(&key).unwrap();
        assert_eq!(rec.memory_value.as_ref().unwrap().verbatim, "妈妈");
    }

    #[test]
    fn at_f_06_conversation_history_append_only() {
        let mut s = store_with_user_device_and_session();
        let corr = CorrelationId(500);
        let turn = TurnId(1);

        let id = s
            .append_conversation_turn(
                ConversationTurnInput::v1(
                    MonotonicTimeNs(10),
                    corr,
                    turn,
                    Some(SessionId(1)),
                    user(),
                    Some(device()),
                    ConversationRole::User,
                    ConversationSource::VoiceTranscript,
                    "hello".to_string(),
                    "hash_hello".to_string(),
                    PrivacyScope::PublicChat,
                    Some("conv_hello".to_string()),
                    None,
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        assert!(matches!(
            s.attempt_overwrite_conversation_turn(id),
            Err(StorageError::AppendOnlyViolation { .. })
        ));
    }

    #[test]
    fn at_f_08_idempotency_key_dedupe() {
        let mut s = store_with_user_device_and_session();

        let id1 = s
            .append_memory_ledger_event(
                &user(),
                mem_event(MemoryLedgerEventKind::Stored, "k", Some("v")),
                MemoryUsePolicy::AlwaysUsable,
                None,
                Some("mem_dup".to_string()),
            )
            .unwrap();
        let id2 = s
            .append_memory_ledger_event(
                &user(),
                mem_event(MemoryLedgerEventKind::Stored, "k", Some("v")),
                MemoryUsePolicy::AlwaysUsable,
                None,
                Some("mem_dup".to_string()),
            )
            .unwrap();
        assert_eq!(id1, id2);
        assert_eq!(s.memory_ledger_rows().len(), 1);

        let corr = CorrelationId(600);
        let turn = TurnId(1);

        let c1 = s
            .append_conversation_turn(
                ConversationTurnInput::v1(
                    MonotonicTimeNs(11),
                    corr,
                    turn,
                    Some(SessionId(1)),
                    user(),
                    Some(device()),
                    ConversationRole::User,
                    ConversationSource::TypedText,
                    "typed".to_string(),
                    "hash_typed".to_string(),
                    PrivacyScope::PublicChat,
                    Some("conv_dup".to_string()),
                    None,
                    None,
                )
                .unwrap(),
            )
            .unwrap();
        let c2 = s
            .append_conversation_turn(
                ConversationTurnInput::v1(
                    MonotonicTimeNs(12),
                    corr,
                    turn,
                    Some(SessionId(1)),
                    user(),
                    Some(device()),
                    ConversationRole::User,
                    ConversationSource::TypedText,
                    "typed".to_string(),
                    "hash_typed".to_string(),
                    PrivacyScope::PublicChat,
                    Some("conv_dup".to_string()),
                    None,
                    None,
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(c1, c2);
        assert_eq!(s.conversation_ledger().len(), 1);

        let e1 = s
            .append_audit_event(
                AuditEventInput::v1(
                    MonotonicTimeNs(1),
                    None,
                    None,
                    Some(SessionId(1)),
                    Some(user()),
                    Some(device()),
                    AuditEngine::Ph1J,
                    AuditEventType::Other,
                    ReasonCodeId(1),
                    AuditSeverity::Info,
                    corr,
                    turn,
                    AuditPayloadMin::empty_v1(),
                    None,
                    Some("audit_dup".to_string()),
                )
                .unwrap(),
            )
            .unwrap();
        let e2 = s
            .append_audit_event(
                AuditEventInput::v1(
                    MonotonicTimeNs(2),
                    None,
                    None,
                    Some(SessionId(1)),
                    Some(user()),
                    Some(device()),
                    AuditEngine::Ph1J,
                    AuditEventType::Other,
                    ReasonCodeId(1),
                    AuditSeverity::Info,
                    corr,
                    turn,
                    AuditPayloadMin::empty_v1(),
                    None,
                    Some("audit_dup".to_string()),
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(e1, e2);
        assert_eq!(s.audit_events().len(), 1);
    }

    #[test]
    fn at_f_09_canonical_ordering_by_ids_not_timestamps() {
        let mut s = store_with_user_and_device();
        let corr = CorrelationId(700);
        let turn = TurnId(1);

        let _ = s
            .append_audit_event(
                AuditEventInput::v1(
                    MonotonicTimeNs(10),
                    None,
                    None,
                    None,
                    Some(user()),
                    Some(device()),
                    AuditEngine::Ph1X,
                    AuditEventType::XDispatch,
                    ReasonCodeId(1),
                    AuditSeverity::Info,
                    corr,
                    turn,
                    AuditPayloadMin::empty_v1(),
                    None,
                    Some("order_1".to_string()),
                )
                .unwrap(),
            )
            .unwrap();
        let _ = s
            .append_audit_event(
                AuditEventInput::v1(
                    MonotonicTimeNs(5), // skewed clock or out-of-order timestamp
                    None,
                    None,
                    None,
                    Some(user()),
                    Some(device()),
                    AuditEngine::Ph1X,
                    AuditEventType::XDispatch,
                    ReasonCodeId(1),
                    AuditSeverity::Info,
                    corr,
                    turn,
                    AuditPayloadMin::empty_v1(),
                    None,
                    Some("order_2".to_string()),
                )
                .unwrap(),
            )
            .unwrap();

        let ids_by_event_id: Vec<u64> = s.audit_events().iter().map(|e| e.event_id.0).collect();
        assert_eq!(ids_by_event_id, vec![1, 2]);

        let mut by_time = s.audit_events().to_vec();
        by_time.sort_by_key(|e| e.created_at.0);
        let ids_by_time: Vec<u64> = by_time.into_iter().map(|e| e.event_id.0).collect();
        assert_eq!(ids_by_time, vec![2, 1]);
    }

    #[test]
    fn at_f_10_redaction_tombstone_conversation() {
        use selene_kernel_contracts::ph1j::AuditEvidenceRef;

        let mut s = store_with_user_device_and_session();
        let corr = CorrelationId(800);

        let original = s
            .append_conversation_turn(
                ConversationTurnInput::v1(
                    MonotonicTimeNs(10),
                    corr,
                    TurnId(1),
                    Some(SessionId(1)),
                    user(),
                    Some(device()),
                    ConversationRole::User,
                    ConversationSource::VoiceTranscript,
                    "secret".to_string(),
                    "hash_secret".to_string(),
                    PrivacyScope::PublicChat,
                    Some("conv_secret".to_string()),
                    None,
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        let _tombstone = s
            .append_conversation_turn(
                ConversationTurnInput::v1(
                    MonotonicTimeNs(11),
                    corr,
                    TurnId(2),
                    Some(SessionId(1)),
                    user(),
                    Some(device()),
                    ConversationRole::Selene,
                    ConversationSource::Tombstone,
                    "[REDACTED]".to_string(),
                    "hash_redacted".to_string(),
                    PrivacyScope::PublicChat,
                    Some("conv_secret_redact".to_string()),
                    Some(original),
                    Some(ReasonCodeId(1)),
                )
                .unwrap(),
            )
            .unwrap();

        assert_eq!(s.conversation_ledger().len(), 2);
        assert_eq!(s.conversation_ledger()[0].text, "secret");
        assert_eq!(
            s.conversation_ledger()[1].tombstone_of_conversation_turn_id,
            Some(original)
        );

        s.append_audit_event(
            AuditEventInput::v1(
                MonotonicTimeNs(12),
                None,
                None,
                Some(SessionId(1)),
                Some(user()),
                Some(device()),
                AuditEngine::Ph1J,
                AuditEventType::JRedactApplied,
                ReasonCodeId(1),
                AuditSeverity::Info,
                corr,
                TurnId(2),
                AuditPayloadMin::empty_v1(),
                Some(AuditEvidenceRef::v1(None, None, Some(original.0)).unwrap()),
                Some("redact_event".to_string()),
            )
            .unwrap(),
        )
        .unwrap();

        assert!(s
            .audit_events_by_correlation(corr)
            .iter()
            .any(|e| e.event_type == AuditEventType::JRedactApplied));
    }

    #[test]
    fn at_j_01_every_gate_emits_an_audit_event() {
        let mut s = store_with_user_and_device();
        let corr = CorrelationId(100);
        let turn = TurnId(1);

        let payload = AuditPayloadMin::v1(BTreeMap::from([(
            PayloadKey::new("gate").unwrap(),
            PayloadValue::new("wake").unwrap(),
        )]))
        .unwrap();

        // Wake rejected.
        s.append_audit_event(
            AuditEventInput::v1(
                MonotonicTimeNs(1),
                None,
                None,
                None,
                Some(user()),
                Some(device()),
                AuditEngine::Ph1W,
                AuditEventType::GateFail,
                ReasonCodeId(0x5700_0050),
                AuditSeverity::Warn,
                corr,
                turn,
                payload.clone(),
                None,
                Some("wake_fail".to_string()),
            )
            .unwrap(),
        )
        .unwrap();

        // STT rejected.
        s.append_audit_event(
            AuditEventInput::v1(
                MonotonicTimeNs(2),
                None,
                None,
                None,
                Some(user()),
                Some(device()),
                AuditEngine::Ph1C,
                AuditEventType::TranscriptReject,
                ReasonCodeId(0x4300_0004),
                AuditSeverity::Warn,
                corr,
                turn,
                AuditPayloadMin::empty_v1(),
                None,
                Some("stt_reject".to_string()),
            )
            .unwrap(),
        )
        .unwrap();

        // Tool fail.
        s.append_audit_event(
            AuditEventInput::v1(
                MonotonicTimeNs(3),
                None,
                None,
                None,
                Some(user()),
                Some(device()),
                AuditEngine::Ph1E,
                AuditEventType::ToolFail,
                ReasonCodeId(0x4500_0003),
                AuditSeverity::Warn,
                corr,
                turn,
                AuditPayloadMin::empty_v1(),
                None,
                Some("tool_fail".to_string()),
            )
            .unwrap(),
        )
        .unwrap();

        let evs = s.audit_events_by_correlation(corr);
        assert!(evs.iter().any(|e| e.event_type == AuditEventType::GateFail));
        assert!(evs
            .iter()
            .any(|e| e.event_type == AuditEventType::TranscriptReject));
        assert!(evs.iter().any(|e| e.event_type == AuditEventType::ToolFail));
    }

    #[test]
    fn at_j_02_append_only_enforcement() {
        let mut s = store_with_user_and_device();
        let corr = CorrelationId(101);
        let turn = TurnId(1);
        let id = s
            .append_audit_event(
                AuditEventInput::v1(
                    MonotonicTimeNs(1),
                    None,
                    None,
                    None,
                    Some(user()),
                    Some(device()),
                    AuditEngine::Ph1J,
                    AuditEventType::Other,
                    ReasonCodeId(1),
                    AuditSeverity::Info,
                    corr,
                    turn,
                    AuditPayloadMin::empty_v1(),
                    None,
                    Some("x".to_string()),
                )
                .unwrap(),
            )
            .unwrap();
        assert!(matches!(
            s.attempt_overwrite_audit_event(id),
            Err(StorageError::AppendOnlyViolation { .. })
        ));
    }

    #[test]
    fn at_j_03_correlation_integrity_one_turn_one_id() {
        let mut s = store_with_user_and_device();
        let corr = CorrelationId(200);
        let turn = TurnId(9);

        for i in 0..3 {
            s.append_audit_event(
                AuditEventInput::v1(
                    MonotonicTimeNs(1 + i),
                    None,
                    None,
                    None,
                    Some(user()),
                    Some(device()),
                    AuditEngine::Ph1X,
                    AuditEventType::XDispatch,
                    ReasonCodeId(1),
                    AuditSeverity::Info,
                    corr,
                    turn,
                    AuditPayloadMin::empty_v1(),
                    None,
                    Some(format!("dispatch_{i}")),
                )
                .unwrap(),
            )
            .unwrap();
        }

        let turns = s.last_turn_ids_for_correlation(corr);
        assert_eq!(turns.len(), 1);
        assert!(turns.contains(&turn));
    }

    #[test]
    fn at_j_04_redaction_is_logged() {
        let mut s = store_with_user_and_device();
        let corr = CorrelationId(300);
        let turn = TurnId(1);

        let payload = AuditPayloadMin::v1(BTreeMap::from([(
            PayloadKey::new("target").unwrap(),
            PayloadValue::new("memory_evidence_quote").unwrap(),
        )]))
        .unwrap();

        s.append_audit_event(
            AuditEventInput::v1(
                MonotonicTimeNs(1),
                None,
                None,
                None,
                Some(user()),
                Some(device()),
                AuditEngine::Ph1J,
                AuditEventType::JRedactApplied,
                ReasonCodeId(1),
                AuditSeverity::Info,
                corr,
                turn,
                payload,
                Some(
                    selene_kernel_contracts::ph1j::AuditEvidenceRef::v1(None, Some(1), None)
                        .unwrap(),
                ),
                Some("redact_1".to_string()),
            )
            .unwrap(),
        )
        .unwrap();

        let evs = s.audit_events_by_correlation(corr);
        assert!(evs
            .iter()
            .any(|e| e.event_type == AuditEventType::JRedactApplied));
    }

    #[test]
    fn at_w_01_wake_enrollment_persists_and_completes() {
        let mut s = store_with_user_and_device();

        let started = s
            .ph1w_enroll_start_draft(
                MonotonicTimeNs(10),
                user(),
                device(),
                None,
                3,
                12,
                300_000,
                "wake-start-1".to_string(),
            )
            .unwrap();
        assert_eq!(started.wake_enroll_status, WakeEnrollStatus::InProgress);
        assert_eq!(started.pass_target, 3);

        let mut st = started.clone();
        for i in 0..3 {
            st = s
                .ph1w_enroll_sample_commit(
                    MonotonicTimeNs(20 + i),
                    started.wake_enrollment_session_id.clone(),
                    900,
                    0.70,
                    14.0,
                    1.0,
                    -24.0,
                    -46.0,
                    -10.0,
                    0.04,
                    WakeSampleResult::Pass,
                    None,
                    format!("wake-sample-{i}"),
                )
                .unwrap();
        }
        assert_eq!(st.wake_enroll_status, WakeEnrollStatus::Complete);
        assert_eq!(st.pass_count, 3);

        let completed = s
            .ph1w_enroll_complete_commit(
                MonotonicTimeNs(30),
                started.wake_enrollment_session_id.clone(),
                "wake_profile_user1_dev1_v1".to_string(),
                "wake-complete-1".to_string(),
            )
            .unwrap();
        assert_eq!(completed.wake_enroll_status, WakeEnrollStatus::Complete);
        assert_eq!(
            s.ph1w_get_active_wake_profile(&user(), &device()),
            Some("wake_profile_user1_dev1_v1")
        );
        assert_eq!(
            s.ph1w_get_samples_for_session(&started.wake_enrollment_session_id)
                .len(),
            3
        );
    }

    #[test]
    fn at_w_02_wake_enrollment_idempotency_and_defer() {
        let mut s = store_with_user_and_device();
        let started = s
            .ph1w_enroll_start_draft(
                MonotonicTimeNs(10),
                user(),
                device(),
                None,
                5,
                8,
                180_000,
                "wake-start-idem".to_string(),
            )
            .unwrap();

        let same_start = s
            .ph1w_enroll_start_draft(
                MonotonicTimeNs(11),
                user(),
                device(),
                None,
                5,
                8,
                180_000,
                "wake-start-idem".to_string(),
            )
            .unwrap();
        assert_eq!(
            started.wake_enrollment_session_id,
            same_start.wake_enrollment_session_id
        );

        let sample_1 = s
            .ph1w_enroll_sample_commit(
                MonotonicTimeNs(20),
                started.wake_enrollment_session_id.clone(),
                900,
                0.65,
                12.0,
                1.2,
                -26.0,
                -47.0,
                -12.0,
                0.05,
                WakeSampleResult::Fail,
                Some(ReasonCodeId(0x5700_3001)),
                "wake-sample-idem".to_string(),
            )
            .unwrap();
        let sample_2 = s
            .ph1w_enroll_sample_commit(
                MonotonicTimeNs(21),
                started.wake_enrollment_session_id.clone(),
                900,
                0.65,
                12.0,
                1.2,
                -26.0,
                -47.0,
                -12.0,
                0.05,
                WakeSampleResult::Fail,
                Some(ReasonCodeId(0x5700_3001)),
                "wake-sample-idem".to_string(),
            )
            .unwrap();
        assert_eq!(sample_1.attempt_count, sample_2.attempt_count);
        assert_eq!(
            s.ph1w_get_samples_for_session(&started.wake_enrollment_session_id)
                .len(),
            1
        );

        let deferred = s
            .ph1w_enroll_defer_reminder_commit(
                MonotonicTimeNs(40),
                started.wake_enrollment_session_id.clone(),
                Some(MonotonicTimeNs(1000)),
                ReasonCodeId(0x5700_3002),
                "wake-defer-1".to_string(),
            )
            .unwrap();
        assert_eq!(deferred.wake_enroll_status, WakeEnrollStatus::Pending);
    }

    #[test]
    fn at_w_03_wake_runtime_event_idempotent() {
        let mut s = store_with_user_device_and_session();
        let ev1 = s
            .ph1w_runtime_event_commit(
                MonotonicTimeNs(100),
                "wake_event_1".to_string(),
                Some(SessionId(1)),
                Some(user()),
                device(),
                true,
                ReasonCodeId(0x5700_0100),
                Some("wake_profile_user1_dev1_v1".to_string()),
                false,
                false,
                None,
                "wake-rt-idem".to_string(),
            )
            .unwrap();
        let ev2 = s
            .ph1w_runtime_event_commit(
                MonotonicTimeNs(101),
                "wake_event_1_retry".to_string(),
                Some(SessionId(1)),
                Some(user()),
                device(),
                true,
                ReasonCodeId(0x5700_0100),
                Some("wake_profile_user1_dev1_v1".to_string()),
                false,
                false,
                None,
                "wake-rt-idem".to_string(),
            )
            .unwrap();
        assert_eq!(ev1.wake_event_id, ev2.wake_event_id);
        assert_eq!(s.ph1w_get_runtime_events().len(), 1);
    }

    #[test]
    fn at_vid_01_voice_enrollment_persists_and_completes() {
        let mut s = store_with_user_and_device();
        let onb_id = insert_onboarding_session(&mut s, "onb_voice_1");

        let started = s
            .ph1vid_enroll_start_draft(
                MonotonicTimeNs(10),
                onb_id.clone(),
                device(),
                true,
                8,
                120_000,
                3,
            )
            .unwrap();
        assert_eq!(started.voice_enroll_status, VoiceEnrollStatus::InProgress);

        let mut st = s
            .ph1vid_enroll_sample_commit(
                MonotonicTimeNs(20),
                started.voice_enrollment_session_id.clone(),
                "sample_ref_1".to_string(),
                1,
                VoiceSampleResult::Pass,
                None,
                "vid-sample-1".to_string(),
            )
            .unwrap();
        assert_eq!(st.voice_enroll_status, VoiceEnrollStatus::InProgress);

        st = s
            .ph1vid_enroll_sample_commit(
                MonotonicTimeNs(21),
                started.voice_enrollment_session_id.clone(),
                "sample_ref_2".to_string(),
                2,
                VoiceSampleResult::Pass,
                None,
                "vid-sample-2".to_string(),
            )
            .unwrap();
        assert_eq!(st.voice_enroll_status, VoiceEnrollStatus::InProgress);

        st = s
            .ph1vid_enroll_sample_commit(
                MonotonicTimeNs(22),
                started.voice_enrollment_session_id.clone(),
                "sample_ref_3".to_string(),
                3,
                VoiceSampleResult::Pass,
                None,
                "vid-sample-3".to_string(),
            )
            .unwrap();
        assert_eq!(st.voice_enroll_status, VoiceEnrollStatus::Locked);
        assert_eq!(st.consecutive_passes, 3);

        let completed = s
            .ph1vid_enroll_complete_commit(
                MonotonicTimeNs(30),
                started.voice_enrollment_session_id.clone(),
                "vid-complete-1".to_string(),
            )
            .unwrap();
        assert_eq!(completed.voice_enroll_status, VoiceEnrollStatus::Locked);
        let profile_id = completed.voice_profile_id.clone().unwrap();
        assert!(s.ph1vid_get_voice_profile(&profile_id).is_some());
        assert_eq!(
            s.ph1vid_get_samples_for_session(&started.voice_enrollment_session_id)
                .len(),
            3
        );
    }

    #[test]
    fn at_vid_02_voice_enrollment_idempotency_and_defer() {
        let mut s = store_with_user_and_device();
        let onb_id = insert_onboarding_session(&mut s, "onb_voice_2");

        let started = s
            .ph1vid_enroll_start_draft(
                MonotonicTimeNs(10),
                onb_id.clone(),
                device(),
                true,
                5,
                60_000,
                2,
            )
            .unwrap();

        let same_start = s
            .ph1vid_enroll_start_draft(MonotonicTimeNs(12), onb_id, device(), true, 5, 60_000, 2)
            .unwrap();
        assert_eq!(
            started.voice_enrollment_session_id,
            same_start.voice_enrollment_session_id
        );

        let sample_1 = s
            .ph1vid_enroll_sample_commit(
                MonotonicTimeNs(20),
                started.voice_enrollment_session_id.clone(),
                "sample_ref_a".to_string(),
                1,
                VoiceSampleResult::Fail,
                Some(ReasonCodeId(0x5649_3001)),
                "vid-sample-idem".to_string(),
            )
            .unwrap();
        let sample_2 = s
            .ph1vid_enroll_sample_commit(
                MonotonicTimeNs(21),
                started.voice_enrollment_session_id.clone(),
                "sample_ref_a".to_string(),
                1,
                VoiceSampleResult::Fail,
                Some(ReasonCodeId(0x5649_3001)),
                "vid-sample-idem".to_string(),
            )
            .unwrap();
        assert_eq!(sample_1.attempt_count, sample_2.attempt_count);
        assert_eq!(
            s.ph1vid_get_samples_for_session(&started.voice_enrollment_session_id)
                .len(),
            1
        );

        let deferred = s
            .ph1vid_enroll_defer_reminder_commit(
                MonotonicTimeNs(40),
                started.voice_enrollment_session_id.clone(),
                ReasonCodeId(0x5649_3002),
                "vid-defer-1".to_string(),
            )
            .unwrap();
        assert_eq!(deferred.voice_enroll_status, VoiceEnrollStatus::Pending);
    }

    #[test]
    fn at_position_01_create_activate_suspend_with_lifecycle_events() {
        let mut s = store_with_user_and_device();
        let tenant_id = TenantId::new("tenant_1").unwrap();
        s.ph1tenant_company_upsert(TenantCompanyRecord {
            schema_version: SchemaVersion(1),
            tenant_id: tenant_id.clone(),
            company_id: "company_1".to_string(),
            legal_name: "Selene Inc".to_string(),
            jurisdiction: "CN".to_string(),
            lifecycle_state: TenantCompanyLifecycleState::Active,
            created_at: MonotonicTimeNs(1),
            updated_at: MonotonicTimeNs(1),
        })
        .unwrap();

        let draft = s
            .ph1position_create_draft(
                MonotonicTimeNs(10),
                user(),
                tenant_id.clone(),
                "company_1".to_string(),
                "Store Manager".to_string(),
                "Retail".to_string(),
                "CN".to_string(),
                PositionScheduleType::FullTime,
                "profile_store_mgr".to_string(),
                "band_l3".to_string(),
                "pos-create-1".to_string(),
                "POSITION_SIM_001_CREATE_DRAFT",
                ReasonCodeId(0x5900_0001),
            )
            .unwrap();
        assert_eq!(draft.lifecycle_state, PositionLifecycleState::Draft);

        let active = s
            .ph1position_activate_commit(
                MonotonicTimeNs(20),
                user(),
                tenant_id.clone(),
                draft.position_id.clone(),
                "pos-activate-1".to_string(),
                "POSITION_SIM_004_ACTIVATE_COMMIT",
                ReasonCodeId(0x5900_0004),
            )
            .unwrap();
        assert_eq!(active.lifecycle_state, PositionLifecycleState::Active);

        let suspended = s
            .ph1position_retire_or_suspend_commit(
                MonotonicTimeNs(30),
                user(),
                tenant_id.clone(),
                active.position_id.clone(),
                PositionLifecycleState::Suspended,
                "pos-suspend-1".to_string(),
                "POSITION_SIM_005_RETIRE_OR_SUSPEND_COMMIT",
                ReasonCodeId(0x5900_0005),
            )
            .unwrap();
        assert_eq!(suspended.lifecycle_state, PositionLifecycleState::Suspended);

        let events =
            s.ph1position_get_lifecycle_events_for_position(&tenant_id, &suspended.position_id);
        assert_eq!(events.len(), 3);
    }

    #[test]
    fn at_position_02_create_requires_active_company() {
        let mut s = store_with_user_and_device();
        let tenant_id = TenantId::new("tenant_2").unwrap();
        s.ph1tenant_company_upsert(TenantCompanyRecord {
            schema_version: SchemaVersion(1),
            tenant_id: tenant_id.clone(),
            company_id: "company_2".to_string(),
            legal_name: "Selene Dormant".to_string(),
            jurisdiction: "US".to_string(),
            lifecycle_state: TenantCompanyLifecycleState::Draft,
            created_at: MonotonicTimeNs(1),
            updated_at: MonotonicTimeNs(1),
        })
        .unwrap();

        let res = s.ph1position_create_draft(
            MonotonicTimeNs(10),
            user(),
            tenant_id,
            "company_2".to_string(),
            "Assistant".to_string(),
            "Ops".to_string(),
            "US".to_string(),
            PositionScheduleType::PartTime,
            "profile_ops".to_string(),
            "band_l1".to_string(),
            "pos-create-draft-deny".to_string(),
            "POSITION_SIM_001_CREATE_DRAFT",
            ReasonCodeId(0x5900_0001),
        );
        assert!(matches!(res, Err(StorageError::ContractViolation(_))));
    }

    // Ensures we still compile against other crate contracts used elsewhere.
    #[test]
    fn _compile_checks() {
        let _ = PolicyContextRef::v1(false, false, SafetyTier::Standard);
        let _ = SpeakerAssertionOk::v1(
            SpeakerId::new("spk").unwrap(),
            Some(UserId::new("user").unwrap()),
            vec![DiarizationSegment::v1(
                MonotonicTimeNs(0),
                MonotonicTimeNs(1),
                Some(SpeakerLabel::speaker_a()),
            )
            .unwrap()],
            SpeakerLabel::speaker_a(),
        )
        .unwrap();
    }
}
