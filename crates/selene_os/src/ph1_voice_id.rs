#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::time::Instant;

use selene_engines::ph1_voice_id::{
    reason_codes as engine_voice_reason_codes, EnrolledSpeaker as EngineEnrolledSpeaker,
    Ph1VoiceIdConfig as EngineVoiceIdConfig, Ph1VoiceIdRuntime as EngineVoiceIdRuntime,
    VoiceIdObservation as EngineVoiceIdObservation,
};
use selene_kernel_contracts::ph1_voice_id::{
    IdentityTierV2, Ph1VoiceIdRequest, Ph1VoiceIdResponse, Ph1VoiceIdSimOk, Ph1VoiceIdSimRequest,
    Ph1VoiceIdSimResponse, UserId, VoiceEnrollStatus as ContractVoiceEnrollStatus,
    VoiceEnrollmentSessionId, VoiceIdDecision, VoiceIdEnrollCompleteResult,
    VoiceIdEnrollDeferResult, VoiceIdEnrollSampleResult, VoiceIdEnrollStartResult,
    VoiceIdSimulationRequest, VoiceIdentityV2, VoiceSampleResult as ContractVoiceSampleResult,
    DEFAULT_CONF_HIGH_BP, DEFAULT_CONF_MID_BP, PH1VOICEID_IMPLEMENTATION_ID,
};
use selene_kernel_contracts::ph1art::{
    ArtifactScopeType, ArtifactStatus, ArtifactType, ArtifactVersion,
};
use selene_kernel_contracts::ph1feedback::{
    classify_feedback_path, FeedbackEventType, FeedbackPathType,
};
use selene_kernel_contracts::ph1j::{
    AuditEngine, AuditEventInput, AuditEventType, AuditPayloadMin, AuditSeverity, CorrelationId,
    DeviceId, PayloadKey, PayloadValue, TurnId,
};
use selene_kernel_contracts::ph1learn::LearnSignalType;
use selene_kernel_contracts::ph1link::AppPlatform;
use selene_kernel_contracts::ph1onb::OnboardingSessionId;
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs, ReasonCodeId, Validate};
use selene_storage::ph1f::{
    Ph1fStore, StorageError, VoiceEnrollStatus as StoreVoiceEnrollStatus,
    VoiceSampleResult as StoreVoiceSampleResult,
};
use selene_storage::ph1j::Ph1jRuntime;

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.VOICE.ID enrollment reason-code namespace. Values are placeholders until global registry is formalized.
    pub const VID_OK_ENROLL_START_DRAFT: ReasonCodeId = ReasonCodeId(0x5649_1001);
    pub const VID_OK_ENROLL_SAMPLE_COMMIT: ReasonCodeId = ReasonCodeId(0x5649_1002);
    pub const VID_OK_ENROLL_COMPLETE_COMMIT: ReasonCodeId = ReasonCodeId(0x5649_1003);
    pub const VID_OK_ENROLL_DEFER_COMMIT: ReasonCodeId = ReasonCodeId(0x5649_1004);
}

pub const PH1_VOICE_ID_ENGINE_ID: &str = "PH1.VOICE.ID";
pub const PH1_VOICE_ID_ACTIVE_IMPLEMENTATION_IDS: &[&str] = &[PH1VOICEID_IMPLEMENTATION_ID];
pub const VOICE_ID_EMBEDDING_GATE_PAYLOAD_REF_V1_PREFIX: &str =
    "voice_id_embedding_gate_profiles:v1:";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VoiceIdentityPlatform {
    Unknown,
    Ios,
    Android,
    Desktop,
}

impl VoiceIdentityPlatform {
    pub fn from_app_platform(app_platform: Option<AppPlatform>) -> Self {
        match app_platform {
            Some(AppPlatform::Ios) => Self::Ios,
            Some(AppPlatform::Android) => Self::Android,
            Some(AppPlatform::Tablet) => Self::Android,
            Some(AppPlatform::Desktop) => Self::Desktop,
            None => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VoiceIdentityChannel {
    Explicit,
    WakeWord,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoiceIdentityEmbeddingGateProfile {
    pub require_primary_embedding: bool,
}

impl VoiceIdentityEmbeddingGateProfile {
    pub const fn required() -> Self {
        Self {
            require_primary_embedding: true,
        }
    }

    pub const fn optional() -> Self {
        Self {
            require_primary_embedding: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoiceIdentityEmbeddingGateProfiles {
    pub global_default: VoiceIdentityEmbeddingGateProfile,
    pub ios_explicit: VoiceIdentityEmbeddingGateProfile,
    pub ios_wake: VoiceIdentityEmbeddingGateProfile,
    pub android_explicit: VoiceIdentityEmbeddingGateProfile,
    pub android_wake: VoiceIdentityEmbeddingGateProfile,
    pub desktop_explicit: VoiceIdentityEmbeddingGateProfile,
    pub desktop_wake: VoiceIdentityEmbeddingGateProfile,
}

impl VoiceIdentityEmbeddingGateProfiles {
    pub fn mvp_v1_phone_first() -> Self {
        Self {
            // Unknown platform/channel is strict by default (fail closed).
            global_default: VoiceIdentityEmbeddingGateProfile::required(),
            ios_explicit: VoiceIdentityEmbeddingGateProfile::required(),
            ios_wake: VoiceIdentityEmbeddingGateProfile::required(),
            android_explicit: VoiceIdentityEmbeddingGateProfile::required(),
            android_wake: VoiceIdentityEmbeddingGateProfile::required(),
            // Desktop defaults to optional while desktop capture stack matures.
            desktop_explicit: VoiceIdentityEmbeddingGateProfile::optional(),
            desktop_wake: VoiceIdentityEmbeddingGateProfile::optional(),
        }
    }

    pub fn profile_for(
        &self,
        platform: VoiceIdentityPlatform,
        channel: VoiceIdentityChannel,
    ) -> VoiceIdentityEmbeddingGateProfile {
        match (platform, channel) {
            (VoiceIdentityPlatform::Unknown, _) => self.global_default,
            (VoiceIdentityPlatform::Ios, VoiceIdentityChannel::Explicit) => self.ios_explicit,
            (VoiceIdentityPlatform::Ios, VoiceIdentityChannel::WakeWord) => self.ios_wake,
            (VoiceIdentityPlatform::Android, VoiceIdentityChannel::Explicit) => {
                self.android_explicit
            }
            (VoiceIdentityPlatform::Android, VoiceIdentityChannel::WakeWord) => self.android_wake,
            (VoiceIdentityPlatform::Desktop, VoiceIdentityChannel::Explicit) => {
                self.desktop_explicit
            }
            (VoiceIdentityPlatform::Desktop, VoiceIdentityChannel::WakeWord) => self.desktop_wake,
        }
    }

    pub fn to_payload_ref_v1(self) -> String {
        format!(
            "{prefix}global_default={global_default},ios_explicit={ios_explicit},ios_wake={ios_wake},android_explicit={android_explicit},android_wake={android_wake},desktop_explicit={desktop_explicit},desktop_wake={desktop_wake}",
            prefix = VOICE_ID_EMBEDDING_GATE_PAYLOAD_REF_V1_PREFIX,
            global_default = gate_profile_label(self.global_default),
            ios_explicit = gate_profile_label(self.ios_explicit),
            ios_wake = gate_profile_label(self.ios_wake),
            android_explicit = gate_profile_label(self.android_explicit),
            android_wake = gate_profile_label(self.android_wake),
            desktop_explicit = gate_profile_label(self.desktop_explicit),
            desktop_wake = gate_profile_label(self.desktop_wake),
        )
    }

    pub fn from_payload_ref_v1(payload_ref: &str) -> Result<Self, ContractViolation> {
        let encoded = payload_ref
            .strip_prefix(VOICE_ID_EMBEDDING_GATE_PAYLOAD_REF_V1_PREFIX)
            .ok_or(ContractViolation::InvalidValue {
                field: "voice_id_embedding_gate_profiles.payload_ref",
                reason: "must start with voice_id_embedding_gate_profiles:v1:",
            })?;
        let mut map: BTreeMap<&str, &str> = BTreeMap::new();
        for entry in encoded.split(',') {
            let (k, v) = entry
                .split_once('=')
                .ok_or(ContractViolation::InvalidValue {
                    field: "voice_id_embedding_gate_profiles.payload_ref",
                    reason: "must encode key=value pairs separated by commas",
                })?;
            map.insert(k, v);
        }
        let required_keys = [
            "global_default",
            "ios_explicit",
            "ios_wake",
            "android_explicit",
            "android_wake",
            "desktop_explicit",
            "desktop_wake",
        ];
        for key in required_keys {
            if !map.contains_key(key) {
                return Err(ContractViolation::InvalidValue {
                    field: "voice_id_embedding_gate_profiles.payload_ref",
                    reason: "missing required gate profile key",
                });
            }
        }
        if map.len() != required_keys.len() {
            return Err(ContractViolation::InvalidValue {
                field: "voice_id_embedding_gate_profiles.payload_ref",
                reason: "contains unexpected gate profile key",
            });
        }
        Ok(Self {
            global_default: parse_gate_profile_field(
                map["global_default"],
                "voice_id_embedding_gate_profiles.global_default",
            )?,
            ios_explicit: parse_gate_profile_field(
                map["ios_explicit"],
                "voice_id_embedding_gate_profiles.ios_explicit",
            )?,
            ios_wake: parse_gate_profile_field(
                map["ios_wake"],
                "voice_id_embedding_gate_profiles.ios_wake",
            )?,
            android_explicit: parse_gate_profile_field(
                map["android_explicit"],
                "voice_id_embedding_gate_profiles.android_explicit",
            )?,
            android_wake: parse_gate_profile_field(
                map["android_wake"],
                "voice_id_embedding_gate_profiles.android_wake",
            )?,
            desktop_explicit: parse_gate_profile_field(
                map["desktop_explicit"],
                "voice_id_embedding_gate_profiles.desktop_explicit",
            )?,
            desktop_wake: parse_gate_profile_field(
                map["desktop_wake"],
                "voice_id_embedding_gate_profiles.desktop_wake",
            )?,
        })
    }
}

fn gate_profile_label(profile: VoiceIdentityEmbeddingGateProfile) -> &'static str {
    if profile.require_primary_embedding {
        "required"
    } else {
        "optional"
    }
}

fn parse_gate_profile_field(
    value: &str,
    field: &'static str,
) -> Result<VoiceIdentityEmbeddingGateProfile, ContractViolation> {
    match value {
        "required" => Ok(VoiceIdentityEmbeddingGateProfile::required()),
        "optional" => Ok(VoiceIdentityEmbeddingGateProfile::optional()),
        _ => Err(ContractViolation::InvalidValue {
            field,
            reason: "must be required|optional",
        }),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceIdentityEmbeddingGateGovernedConfig {
    pub global_profiles: VoiceIdentityEmbeddingGateProfiles,
    pub tenant_overrides: BTreeMap<String, VoiceIdentityEmbeddingGateProfiles>,
}

impl VoiceIdentityEmbeddingGateGovernedConfig {
    pub fn mvp_v1_phone_first() -> Self {
        Self {
            global_profiles: VoiceIdentityEmbeddingGateProfiles::mvp_v1_phone_first(),
            tenant_overrides: BTreeMap::new(),
        }
    }

    pub fn profile_for(
        &self,
        tenant_id: Option<&str>,
        platform: VoiceIdentityPlatform,
        channel: VoiceIdentityChannel,
    ) -> VoiceIdentityEmbeddingGateProfile {
        if let Some(tid) = normalize_tenant_id(tenant_id) {
            if let Some(override_profiles) = self.tenant_overrides.get(tid) {
                return override_profiles.profile_for(platform, channel);
            }
        }
        self.global_profiles.profile_for(platform, channel)
    }

    pub fn with_tenant_override(
        mut self,
        tenant_id: impl Into<String>,
        profiles: VoiceIdentityEmbeddingGateProfiles,
    ) -> Result<Self, ContractViolation> {
        let tid = tenant_id.into();
        validate_tenant_id(
            "voice_identity_embedding_gate_governed_config.tenant_overrides.tenant_id",
            &tid,
        )?;
        self.tenant_overrides.insert(tid, profiles);
        Ok(self)
    }
}

fn normalize_tenant_id(tenant_id: Option<&str>) -> Option<&str> {
    let value = tenant_id?.trim();
    if value.is_empty() {
        return None;
    }
    Some(value)
}

fn validate_tenant_id(field: &'static str, tenant_id: &str) -> Result<(), ContractViolation> {
    if tenant_id.trim().is_empty() || tenant_id.len() > 64 || !tenant_id.is_ascii() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be non-empty ASCII and <= 64 chars",
        });
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceIdentityRuntimeContext {
    pub tenant_id: Option<String>,
    pub platform: VoiceIdentityPlatform,
    pub channel: VoiceIdentityChannel,
}

impl VoiceIdentityRuntimeContext {
    pub fn v1(platform: VoiceIdentityPlatform, channel: VoiceIdentityChannel) -> Self {
        Self {
            tenant_id: None,
            platform,
            channel,
        }
    }

    pub fn for_tenant(
        tenant_id: Option<String>,
        platform: VoiceIdentityPlatform,
        channel: VoiceIdentityChannel,
    ) -> Self {
        Self {
            tenant_id,
            platform,
            channel,
        }
    }

    pub fn from_app_platform(
        app_platform: Option<AppPlatform>,
        channel: VoiceIdentityChannel,
    ) -> Self {
        Self::from_tenant_app_platform(None, app_platform, channel)
    }

    pub fn from_tenant_app_platform(
        tenant_id: Option<String>,
        app_platform: Option<AppPlatform>,
        channel: VoiceIdentityChannel,
    ) -> Self {
        Self {
            tenant_id,
            platform: VoiceIdentityPlatform::from_app_platform(app_platform),
            channel,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1VoiceIdLiveConfig {
    pub embedding_gate_profiles: VoiceIdentityEmbeddingGateGovernedConfig,
    pub contract_migration: VoiceIdContractMigrationConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceIdContractMigrationStage {
    M0,
    M1,
    M2,
    M3,
}

impl VoiceIdContractMigrationStage {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::M0 => "M0",
            Self::M1 => "M1",
            Self::M2 => "M2",
            Self::M3 => "M3",
        }
    }

    pub const fn read_contract(self) -> &'static str {
        match self {
            Self::M0 | Self::M1 => "V1",
            Self::M2 | Self::M3 => "V2",
        }
    }

    pub const fn force_provisional_v2(self) -> bool {
        matches!(self, Self::M0 | Self::M1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoiceIdContractMigrationConfig {
    pub stage: VoiceIdContractMigrationStage,
}

impl VoiceIdContractMigrationConfig {
    pub const fn mvp_default() -> Self {
        // Runtime currently reads identity_v2 by default; M2 keeps that behavior while
        // still allowing deterministic M0/M1 fallback in config.
        Self {
            stage: VoiceIdContractMigrationStage::M2,
        }
    }
}

impl Ph1VoiceIdLiveConfig {
    pub fn mvp_v1_phone_first() -> Self {
        Self {
            embedding_gate_profiles: VoiceIdentityEmbeddingGateGovernedConfig::mvp_v1_phone_first(),
            contract_migration: VoiceIdContractMigrationConfig::mvp_default(),
        }
    }

    pub fn with_tenant_embedding_gate_override(
        mut self,
        tenant_id: impl Into<String>,
        profiles: VoiceIdentityEmbeddingGateProfiles,
    ) -> Result<Self, ContractViolation> {
        self.embedding_gate_profiles = self
            .embedding_gate_profiles
            .with_tenant_override(tenant_id, profiles)?;
        Ok(self)
    }

    pub fn with_contract_migration_stage(mut self, stage: VoiceIdContractMigrationStage) -> Self {
        self.contract_migration.stage = stage;
        self
    }
}

#[derive(Debug, Clone)]
pub struct Ph1VoiceIdLiveRuntime {
    config: Ph1VoiceIdLiveConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceArtifactPointer {
    pub artifact_id: u64,
    pub artifact_type: ArtifactType,
    pub artifact_version: ArtifactVersion,
    pub payload_ref: String,
    pub status: ArtifactStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VoiceArtifactPointerSet {
    pub active: Option<VoiceArtifactPointer>,
    pub rollback: Option<VoiceArtifactPointer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VoiceTenantArtifactPointers {
    pub threshold_pack: VoiceArtifactPointerSet,
    pub confusion_pair_pack: VoiceArtifactPointerSet,
    pub spoof_policy_pack: VoiceArtifactPointerSet,
    pub profile_delta_pack: VoiceArtifactPointerSet,
}

impl Default for Ph1VoiceIdLiveRuntime {
    fn default() -> Self {
        Self::new(Ph1VoiceIdLiveConfig::mvp_v1_phone_first())
    }
}

impl Ph1VoiceIdLiveRuntime {
    pub fn new(config: Ph1VoiceIdLiveConfig) -> Self {
        Self { config }
    }

    pub fn embedding_gate_profile_for(
        &self,
        context: VoiceIdentityRuntimeContext,
    ) -> VoiceIdentityEmbeddingGateProfile {
        self.config.embedding_gate_profiles.profile_for(
            context.tenant_id.as_deref(),
            context.platform,
            context.channel,
        )
    }

    pub fn with_governed_threshold_pack_overrides(&self, store: &Ph1fStore) -> Self {
        let mut config = self.config.clone();
        let mut tenant_ids: BTreeMap<String, ()> = BTreeMap::new();
        for row in store.artifacts_ledger_rows() {
            if row.scope_type == ArtifactScopeType::Tenant
                && (row.created_by == "PH1.LEARN" || row.created_by == "PH1.BUILDER")
                && row.artifact_type == ArtifactType::VoiceIdThresholdPack
            {
                tenant_ids.insert(row.scope_id.clone(), ());
            }
        }
        for tenant_id in tenant_ids.keys() {
            let pointers = self.tenant_artifact_pointers(store, tenant_id);
            let Some(active_threshold) = pointers.threshold_pack.active else {
                continue;
            };
            let Ok(profiles) = VoiceIdentityEmbeddingGateProfiles::from_payload_ref_v1(
                &active_threshold.payload_ref,
            ) else {
                continue;
            };
            config
                .embedding_gate_profiles
                .tenant_overrides
                .insert(tenant_id.clone(), profiles);
        }
        Self::new(config)
    }

    pub fn tenant_artifact_pointers(
        &self,
        store: &Ph1fStore,
        tenant_id: &str,
    ) -> VoiceTenantArtifactPointers {
        VoiceTenantArtifactPointers {
            threshold_pack: select_artifact_pointer_set(
                store,
                tenant_id,
                ArtifactType::VoiceIdThresholdPack,
            ),
            confusion_pair_pack: select_artifact_pointer_set(
                store,
                tenant_id,
                ArtifactType::VoiceIdConfusionPairPack,
            ),
            spoof_policy_pack: select_artifact_pointer_set(
                store,
                tenant_id,
                ArtifactType::VoiceIdSpoofPolicyPack,
            ),
            profile_delta_pack: select_artifact_pointer_set(
                store,
                tenant_id,
                ArtifactType::VoiceIdProfileDeltaPack,
            ),
        }
    }

    pub fn run_identity_assertion(
        &self,
        req: &Ph1VoiceIdRequest,
        context: VoiceIdentityRuntimeContext,
        enrolled: Vec<EngineEnrolledSpeaker>,
        obs: EngineVoiceIdObservation,
    ) -> Result<Ph1VoiceIdResponse, StorageError> {
        req.validate().map_err(StorageError::ContractViolation)?;
        let mut config = EngineVoiceIdConfig::mvp_v1();
        config.require_primary_embedding = self
            .embedding_gate_profile_for(context)
            .require_primary_embedding;
        let mut runtime =
            EngineVoiceIdRuntime::new(config, enrolled).map_err(StorageError::ContractViolation)?;
        Ok(runtime.run(req, obs))
    }

    pub fn run_identity_assertion_with_signal_emission(
        &self,
        store: &mut Ph1fStore,
        req: &Ph1VoiceIdRequest,
        context: VoiceIdentityRuntimeContext,
        enrolled: Vec<EngineEnrolledSpeaker>,
        obs: EngineVoiceIdObservation,
        signal_scope: VoiceIdentitySignalScope,
    ) -> Result<Ph1VoiceIdResponse, StorageError> {
        let started = Instant::now();
        let mut response = self.run_identity_assertion(req, context.clone(), enrolled, obs)?;
        let migration =
            apply_contract_migration_stage(&mut response, self.config.contract_migration.stage);
        emit_voice_id_contract_migration_audit(store, &signal_scope, migration)?;
        emit_voice_id_cohort_kpi_audit(
            store,
            req,
            context.clone(),
            &signal_scope,
            &response,
            started.elapsed().as_millis().min(u128::from(u32::MAX)) as u32,
        )?;
        let feedback_learn_audit_context = voice_feedback_learn_audit_context(&context, &response);
        emit_voice_id_decision_audit(store, context, &signal_scope, &response)?;
        if let Some(signal) = map_voice_response_to_feedback_learn_signal(&response) {
            emit_voice_id_feedback_and_learn_signal(
                store,
                &signal_scope,
                signal,
                feedback_learn_audit_context,
            )?;
        }
        Ok(response)
    }
}

fn select_artifact_pointer_set(
    store: &Ph1fStore,
    tenant_id: &str,
    artifact_type: ArtifactType,
) -> VoiceArtifactPointerSet {
    let mut rows = store
        .artifacts_ledger_rows()
        .iter()
        .filter(|row| {
            row.scope_type == ArtifactScopeType::Tenant
                && row.scope_id == tenant_id
                && (row.created_by == "PH1.LEARN" || row.created_by == "PH1.BUILDER")
                && row.artifact_type == artifact_type
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b.artifact_version
            .cmp(&a.artifact_version)
            .then_with(|| b.artifact_id.cmp(&a.artifact_id))
    });
    if rows.is_empty() {
        return VoiceArtifactPointerSet::default();
    }

    let selected_idx = rows
        .iter()
        .position(|row| row.status == ArtifactStatus::Active)
        .unwrap_or(0);
    let is_revoked = |row: &&selene_kernel_contracts::ph1art::ArtifactLedgerRow| {
        store.voice_artifact_is_revoked(tenant_id, artifact_type, row.artifact_version)
    };
    let active_idx = if is_revoked(&rows[selected_idx]) {
        match rows
            .iter()
            .enumerate()
            .find_map(|(idx, row)| (idx > selected_idx && !is_revoked(&row)).then_some(idx))
        {
            Some(idx) => idx,
            None => return VoiceArtifactPointerSet::default(),
        }
    } else {
        selected_idx
    };
    let active_row = rows[active_idx];
    let rollback_row = rows
        .iter()
        .enumerate()
        .find_map(|(idx, row)| (idx > active_idx && !is_revoked(&row)).then_some(*row));

    VoiceArtifactPointerSet {
        active: Some(VoiceArtifactPointer {
            artifact_id: active_row.artifact_id,
            artifact_type: active_row.artifact_type,
            artifact_version: active_row.artifact_version,
            payload_ref: active_row.payload_ref.clone(),
            status: active_row.status,
        }),
        rollback: rollback_row.map(|row| VoiceArtifactPointer {
            artifact_id: row.artifact_id,
            artifact_type: row.artifact_type,
            artifact_version: row.artifact_version,
            payload_ref: row.payload_ref.clone(),
            status: row.status,
        }),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceIdentitySignalScope {
    pub now: MonotonicTimeNs,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub actor_user_id: UserId,
    pub tenant_id: Option<String>,
    pub device_id: Option<DeviceId>,
}

impl VoiceIdentitySignalScope {
    pub fn v1(
        now: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        actor_user_id: UserId,
        tenant_id: Option<String>,
        device_id: Option<DeviceId>,
    ) -> Self {
        Self {
            now,
            correlation_id,
            turn_id,
            actor_user_id,
            tenant_id,
            device_id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VoiceFeedbackLearnSignal {
    reason_code: ReasonCodeId,
    feedback_event_type: FeedbackEventType,
    learn_signal_type: LearnSignalType,
    decision: &'static str,
    score_bp: Option<u16>,
    margin_to_next_bp: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VoiceIdContractMigrationSnapshot {
    stage: VoiceIdContractMigrationStage,
    read_contract: &'static str,
    decision_v1: VoiceIdDecision,
    observed_identity_v2: VoiceIdentityV2,
    provisional_identity_v2: VoiceIdentityV2,
    final_identity_v2: VoiceIdentityV2,
    shadow_drift: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VoiceIdCohortKpiSnapshot {
    tar: u8,
    frr: u8,
    far: u8,
    score_bp: u16,
    margin_to_next_bp: Option<u16>,
    decision_v1: VoiceIdDecision,
    identity_tier_v2: IdentityTierV2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VoiceIdDecisionAuditSnapshot {
    decision_v1: VoiceIdDecision,
    identity_tier_v2: IdentityTierV2,
    score_bp: u16,
    margin_to_next_bp: Option<u16>,
    reason_code: ReasonCodeId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VoiceIdFeedbackLearnAuditContext {
    final_identity_decision: VoiceIdDecision,
    final_reason_code: ReasonCodeId,
    identity_tier_v2: IdentityTierV2,
    platform: VoiceIdentityPlatform,
    channel: VoiceIdentityChannel,
}

fn feedback_event_type_label(event_type: FeedbackEventType) -> &'static str {
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

fn learn_signal_type_label(signal_type: LearnSignalType) -> &'static str {
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

fn feedback_path_type_label(path_type: FeedbackPathType) -> &'static str {
    match path_type {
        FeedbackPathType::Defect => "PathA_Defect",
        FeedbackPathType::Improvement => "PathB_Improvement",
    }
}

fn apply_contract_migration_stage(
    response: &mut Ph1VoiceIdResponse,
    stage: VoiceIdContractMigrationStage,
) -> VoiceIdContractMigrationSnapshot {
    let decision_v1 = voice_decision_v1(response);
    let score_bp = voice_score_bp(response);
    let observed_identity_v2 = response.identity_v2();
    let provisional_identity_v2 = VoiceIdentityV2::from_v1(
        decision_v1,
        score_bp,
        DEFAULT_CONF_HIGH_BP,
        DEFAULT_CONF_MID_BP,
    );
    let shadow_drift = observed_identity_v2 != provisional_identity_v2;
    if stage.force_provisional_v2() {
        set_response_identity_v2(response, provisional_identity_v2);
    }
    let final_identity_v2 = response.identity_v2();
    VoiceIdContractMigrationSnapshot {
        stage,
        read_contract: stage.read_contract(),
        decision_v1,
        observed_identity_v2,
        provisional_identity_v2,
        final_identity_v2,
        shadow_drift,
    }
}

fn emit_voice_id_contract_migration_audit(
    store: &mut Ph1fStore,
    signal_scope: &VoiceIdentitySignalScope,
    snapshot: VoiceIdContractMigrationSnapshot,
) -> Result<(), StorageError> {
    let mut payload_entries = BTreeMap::new();
    payload_entries.insert(
        PayloadKey::new("migration_stage").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(snapshot.stage.as_str()).map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("read_contract").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(snapshot.read_contract).map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("decision_v1").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(voice_decision_label(snapshot.decision_v1))
            .map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("identity_tier_v2_observed").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(identity_tier_label(
            snapshot.observed_identity_v2.identity_tier_v2,
        ))
        .map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("identity_tier_v2_provisional").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(identity_tier_label(
            snapshot.provisional_identity_v2.identity_tier_v2,
        ))
        .map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("identity_tier_v2_final").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(identity_tier_label(
            snapshot.final_identity_v2.identity_tier_v2,
        ))
        .map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("shadow_drift").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(if snapshot.shadow_drift {
            "true"
        } else {
            "false"
        })
        .map_err(StorageError::ContractViolation)?,
    );
    let payload = AuditPayloadMin::v1(payload_entries).map_err(StorageError::ContractViolation)?;
    let event = AuditEventInput::v1(
        signal_scope.now,
        signal_scope.tenant_id.clone(),
        None,
        None,
        Some(signal_scope.actor_user_id.clone()),
        signal_scope.device_id.clone(),
        AuditEngine::Other(PH1_VOICE_ID_ENGINE_ID.to_string()),
        AuditEventType::Other,
        engine_voice_reason_codes::VID_OK_MATCHED,
        AuditSeverity::Info,
        signal_scope.correlation_id,
        signal_scope.turn_id,
        payload,
        None,
        Some(format!(
            "voice_migration:{}:{}:{}",
            signal_scope.correlation_id.0,
            signal_scope.turn_id.0,
            snapshot.stage.as_str()
        )),
    )
    .map_err(StorageError::ContractViolation)?;
    Ph1jRuntime::emit(store, event)?;
    Ok(())
}

fn emit_voice_id_cohort_kpi_audit(
    store: &mut Ph1fStore,
    req: &Ph1VoiceIdRequest,
    context: VoiceIdentityRuntimeContext,
    signal_scope: &VoiceIdentitySignalScope,
    response: &Ph1VoiceIdResponse,
    latency_ms: u32,
) -> Result<(), StorageError> {
    let snapshot = voice_kpi_snapshot(response);
    let device_cohort = format!(
        "{}_{}",
        platform_label(context.platform),
        channel_label(context.channel)
    );
    let noise_cohort = classify_noise_cohort(req);
    let mut payload_entries = BTreeMap::new();
    payload_entries.insert(
        PayloadKey::new("metric_family").map_err(StorageError::ContractViolation)?,
        PayloadValue::new("voice_id_cohort_kpi").map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("cohort_language").map_err(StorageError::ContractViolation)?,
        PayloadValue::new("unknown").map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("cohort_accent").map_err(StorageError::ContractViolation)?,
        PayloadValue::new("unknown").map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("cohort_device").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(device_cohort).map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("cohort_noise").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(noise_cohort).map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("tar").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(snapshot.tar.to_string()).map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("frr").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(snapshot.frr.to_string()).map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("far").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(snapshot.far.to_string()).map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("latency_ms").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(latency_ms.to_string()).map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("decision_v1").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(voice_decision_label(snapshot.decision_v1))
            .map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("identity_tier_v2").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(identity_tier_label(snapshot.identity_tier_v2))
            .map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("score_bp").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(snapshot.score_bp.to_string())
            .map_err(StorageError::ContractViolation)?,
    );
    if let Some(margin) = snapshot.margin_to_next_bp {
        payload_entries.insert(
            PayloadKey::new("margin_to_next_bp").map_err(StorageError::ContractViolation)?,
            PayloadValue::new(margin.to_string()).map_err(StorageError::ContractViolation)?,
        );
    }

    let payload = AuditPayloadMin::v1(payload_entries).map_err(StorageError::ContractViolation)?;
    let event = AuditEventInput::v1(
        signal_scope.now,
        signal_scope.tenant_id.clone(),
        None,
        None,
        Some(signal_scope.actor_user_id.clone()),
        signal_scope.device_id.clone(),
        AuditEngine::Other(PH1_VOICE_ID_ENGINE_ID.to_string()),
        AuditEventType::Other,
        response_reason_code(response),
        AuditSeverity::Info,
        signal_scope.correlation_id,
        signal_scope.turn_id,
        payload,
        None,
        Some(format!(
            "voice_kpi:{}:{}",
            signal_scope.correlation_id.0, signal_scope.turn_id.0
        )),
    )
    .map_err(StorageError::ContractViolation)?;
    Ph1jRuntime::emit(store, event)?;
    Ok(())
}

fn emit_voice_id_decision_audit(
    store: &mut Ph1fStore,
    context: VoiceIdentityRuntimeContext,
    signal_scope: &VoiceIdentitySignalScope,
    response: &Ph1VoiceIdResponse,
) -> Result<(), StorageError> {
    let snapshot = voice_decision_audit_snapshot(response);
    let mut payload_entries = BTreeMap::new();
    payload_entries.insert(
        PayloadKey::new("decision_log_family").map_err(StorageError::ContractViolation)?,
        PayloadValue::new("voice_id_live_assertion").map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("decision_v1").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(voice_decision_label(snapshot.decision_v1))
            .map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("identity_tier_v2").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(identity_tier_label(snapshot.identity_tier_v2))
            .map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("score_bp").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(snapshot.score_bp.to_string())
            .map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("platform").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(platform_label(context.platform))
            .map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("channel").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(channel_label(context.channel))
            .map_err(StorageError::ContractViolation)?,
    );
    if let Some(margin) = snapshot.margin_to_next_bp {
        payload_entries.insert(
            PayloadKey::new("margin_to_next_bp").map_err(StorageError::ContractViolation)?,
            PayloadValue::new(margin.to_string()).map_err(StorageError::ContractViolation)?,
        );
    }

    let payload = AuditPayloadMin::v1(payload_entries).map_err(StorageError::ContractViolation)?;
    let event = AuditEventInput::v1(
        signal_scope.now,
        signal_scope.tenant_id.clone(),
        None,
        None,
        Some(signal_scope.actor_user_id.clone()),
        signal_scope.device_id.clone(),
        AuditEngine::Other(PH1_VOICE_ID_ENGINE_ID.to_string()),
        AuditEventType::Other,
        snapshot.reason_code,
        AuditSeverity::Info,
        signal_scope.correlation_id,
        signal_scope.turn_id,
        payload,
        None,
        Some(format!(
            "voice_decision:{}:{}",
            signal_scope.correlation_id.0, signal_scope.turn_id.0
        )),
    )
    .map_err(StorageError::ContractViolation)?;
    Ph1jRuntime::emit(store, event)?;
    Ok(())
}

fn voice_kpi_snapshot(response: &Ph1VoiceIdResponse) -> VoiceIdCohortKpiSnapshot {
    match response {
        Ph1VoiceIdResponse::SpeakerAssertionOk(ok) => VoiceIdCohortKpiSnapshot {
            tar: 1,
            frr: 0,
            far: u8::from(ok.margin_to_next_bp.is_some_and(|margin| margin < 300)),
            score_bp: ok.score_bp,
            margin_to_next_bp: ok.margin_to_next_bp,
            decision_v1: VoiceIdDecision::Ok,
            identity_tier_v2: ok.identity_v2.identity_tier_v2,
        },
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(u) => VoiceIdCohortKpiSnapshot {
            tar: 0,
            frr: 1,
            far: 0,
            score_bp: u.score_bp,
            margin_to_next_bp: u.margin_to_next_bp,
            decision_v1: VoiceIdDecision::Unknown,
            identity_tier_v2: u.identity_v2.identity_tier_v2,
        },
    }
}

fn voice_decision_audit_snapshot(response: &Ph1VoiceIdResponse) -> VoiceIdDecisionAuditSnapshot {
    VoiceIdDecisionAuditSnapshot {
        decision_v1: voice_decision_v1(response),
        identity_tier_v2: response.identity_v2().identity_tier_v2,
        score_bp: voice_score_bp(response),
        margin_to_next_bp: match response {
            Ph1VoiceIdResponse::SpeakerAssertionOk(ok) => ok.margin_to_next_bp,
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(u) => u.margin_to_next_bp,
        },
        reason_code: response_reason_code(response),
    }
}

fn voice_feedback_learn_audit_context(
    context: &VoiceIdentityRuntimeContext,
    response: &Ph1VoiceIdResponse,
) -> VoiceIdFeedbackLearnAuditContext {
    VoiceIdFeedbackLearnAuditContext {
        final_identity_decision: voice_decision_v1(response),
        final_reason_code: response_reason_code(response),
        identity_tier_v2: response.identity_v2().identity_tier_v2,
        platform: context.platform,
        channel: context.channel,
    }
}

fn voice_decision_v1(response: &Ph1VoiceIdResponse) -> VoiceIdDecision {
    match response {
        Ph1VoiceIdResponse::SpeakerAssertionOk(_) => VoiceIdDecision::Ok,
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(_) => VoiceIdDecision::Unknown,
    }
}

fn voice_score_bp(response: &Ph1VoiceIdResponse) -> u16 {
    match response {
        Ph1VoiceIdResponse::SpeakerAssertionOk(ok) => ok.score_bp,
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(u) => u.score_bp,
    }
}

fn set_response_identity_v2(response: &mut Ph1VoiceIdResponse, identity_v2: VoiceIdentityV2) {
    match response {
        Ph1VoiceIdResponse::SpeakerAssertionOk(ok) => ok.identity_v2 = identity_v2,
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(u) => u.identity_v2 = identity_v2,
    }
}

fn response_reason_code(response: &Ph1VoiceIdResponse) -> ReasonCodeId {
    match response {
        Ph1VoiceIdResponse::SpeakerAssertionOk(ok) => ok
            .reason_code
            .unwrap_or(engine_voice_reason_codes::VID_OK_MATCHED),
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(u) => u.reason_code,
    }
}

fn voice_decision_label(decision: VoiceIdDecision) -> &'static str {
    match decision {
        VoiceIdDecision::Ok => "OK",
        VoiceIdDecision::Unknown => "UNKNOWN",
    }
}

fn identity_tier_label(tier: IdentityTierV2) -> &'static str {
    match tier {
        IdentityTierV2::Confirmed => "CONFIRMED",
        IdentityTierV2::Probable => "PROBABLE",
        IdentityTierV2::Unknown => "UNKNOWN",
    }
}

fn platform_label(platform: VoiceIdentityPlatform) -> &'static str {
    match platform {
        VoiceIdentityPlatform::Unknown => "unknown",
        VoiceIdentityPlatform::Ios => "ios",
        VoiceIdentityPlatform::Android => "android",
        VoiceIdentityPlatform::Desktop => "desktop",
    }
}

fn channel_label(channel: VoiceIdentityChannel) -> &'static str {
    match channel {
        VoiceIdentityChannel::Explicit => "explicit",
        VoiceIdentityChannel::WakeWord => "wake",
    }
}

fn classify_noise_cohort(req: &Ph1VoiceIdRequest) -> &'static str {
    if req.vad_events.is_empty() {
        return "unknown";
    }
    let avg = req
        .vad_events
        .iter()
        .map(|event| event.speech_likeness.0)
        .sum::<f32>()
        / (req.vad_events.len() as f32);
    if avg >= 0.90 {
        "quiet"
    } else if avg >= 0.75 {
        "normal"
    } else {
        "noisy"
    }
}

fn map_voice_response_to_feedback_learn_signal(
    response: &Ph1VoiceIdResponse,
) -> Option<VoiceFeedbackLearnSignal> {
    match response {
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(u) => {
            let (feedback_event_type, learn_signal_type) = match u.reason_code {
                engine_voice_reason_codes::VID_SPOOF_RISK => (
                    FeedbackEventType::VoiceIdSpoofRisk,
                    LearnSignalType::VoiceIdSpoofRisk,
                ),
                engine_voice_reason_codes::VID_FAIL_MULTI_SPEAKER_PRESENT => (
                    FeedbackEventType::VoiceIdMultiSpeaker,
                    LearnSignalType::VoiceIdMultiSpeaker,
                ),
                engine_voice_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN => (
                    FeedbackEventType::VoiceIdFalseAccept,
                    LearnSignalType::VoiceIdFalseAccept,
                ),
                engine_voice_reason_codes::VID_FAIL_PROFILE_NOT_ENROLLED
                | engine_voice_reason_codes::VID_ENROLLMENT_REQUIRED => (
                    FeedbackEventType::VoiceIdDriftAlert,
                    LearnSignalType::VoiceIdDriftAlert,
                ),
                engine_voice_reason_codes::VID_REAUTH_REQUIRED
                | engine_voice_reason_codes::VID_DEVICE_CLAIM_REQUIRED => (
                    FeedbackEventType::VoiceIdReauthFriction,
                    LearnSignalType::VoiceIdReauthFriction,
                ),
                engine_voice_reason_codes::VID_FAIL_NO_SPEECH
                | engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE
                | engine_voice_reason_codes::VID_FAIL_ECHO_UNSAFE => (
                    FeedbackEventType::VoiceIdFalseReject,
                    LearnSignalType::VoiceIdFalseReject,
                ),
                _ => (
                    FeedbackEventType::VoiceIdFalseReject,
                    LearnSignalType::VoiceIdFalseReject,
                ),
            };
            Some(VoiceFeedbackLearnSignal {
                reason_code: u.reason_code,
                feedback_event_type,
                learn_signal_type,
                decision: "UNKNOWN",
                score_bp: Some(u.score_bp),
                margin_to_next_bp: u.margin_to_next_bp,
            })
        }
        Ph1VoiceIdResponse::SpeakerAssertionOk(ok) => {
            if ok.margin_to_next_bp.is_some_and(|margin| margin < 300) {
                Some(VoiceFeedbackLearnSignal {
                    reason_code: engine_voice_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN,
                    feedback_event_type: FeedbackEventType::VoiceIdFalseAccept,
                    learn_signal_type: LearnSignalType::VoiceIdFalseAccept,
                    decision: "OK_LOW_MARGIN",
                    score_bp: Some(ok.score_bp),
                    margin_to_next_bp: ok.margin_to_next_bp,
                })
            } else if ok.score_bp < 9_500 {
                Some(VoiceFeedbackLearnSignal {
                    reason_code: engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE,
                    feedback_event_type: FeedbackEventType::VoiceIdReauthFriction,
                    learn_signal_type: LearnSignalType::VoiceIdReauthFriction,
                    decision: "OK_LOW_SCORE",
                    score_bp: Some(ok.score_bp),
                    margin_to_next_bp: ok.margin_to_next_bp,
                })
            } else {
                None
            }
        }
    }
}

fn voice_audit_context_payload_metadata(
    signal: VoiceFeedbackLearnSignal,
    live_context: VoiceIdFeedbackLearnAuditContext,
) -> BTreeMap<String, String> {
    let mut payload_metadata = BTreeMap::new();
    payload_metadata.insert("voice_decision".to_string(), signal.decision.to_string());
    payload_metadata.insert(
        "voice_reason_code_hex".to_string(),
        format!("0x{:X}", signal.reason_code.0),
    );
    payload_metadata.insert(
        "identity_tier_v2".to_string(),
        identity_tier_label(live_context.identity_tier_v2).to_string(),
    );
    payload_metadata.insert(
        "platform".to_string(),
        platform_label(live_context.platform).to_string(),
    );
    payload_metadata.insert(
        "channel".to_string(),
        channel_label(live_context.channel).to_string(),
    );
    if let Some(score) = signal.score_bp {
        payload_metadata.insert("voice_score_bp".to_string(), score.to_string());
    }
    if let Some(margin) = signal.margin_to_next_bp {
        payload_metadata.insert("voice_margin_to_next_bp".to_string(), margin.to_string());
    }
    payload_metadata
}

fn voice_feedback_audit_payload_metadata(
    signal: VoiceFeedbackLearnSignal,
    live_context: VoiceIdFeedbackLearnAuditContext,
) -> BTreeMap<String, String> {
    let mut payload_metadata = voice_audit_context_payload_metadata(signal, live_context);
    payload_metadata.insert(
        "final_identity_decision".to_string(),
        voice_decision_label(live_context.final_identity_decision).to_string(),
    );
    payload_metadata.insert(
        "final_reason_code_hex".to_string(),
        format!("0x{:X}", live_context.final_reason_code.0),
    );
    payload_metadata
}

fn voice_learn_audit_payload_metadata(
    signal: VoiceFeedbackLearnSignal,
    live_context: VoiceIdFeedbackLearnAuditContext,
) -> BTreeMap<String, String> {
    let can_surface_final_decision_context = signal.margin_to_next_bp.is_none();
    let mut payload_metadata = voice_audit_context_payload_metadata(signal, live_context);
    payload_metadata.insert(
        "signal_bucket".to_string(),
        learn_signal_type_label(signal.learn_signal_type).to_string(),
    );
    if can_surface_final_decision_context {
        payload_metadata.insert(
            "final_identity_decision".to_string(),
            voice_decision_label(live_context.final_identity_decision).to_string(),
        );
        payload_metadata.insert(
            "final_reason_code_hex".to_string(),
            format!("0x{:X}", live_context.final_reason_code.0),
        );
    }
    payload_metadata
}

fn unscoped_voice_feedback_learn_payload_base(
    signal: VoiceFeedbackLearnSignal,
    live_context: VoiceIdFeedbackLearnAuditContext,
    feedback_event_type: &str,
    learn_signal_type: &str,
    evidence_ref: &str,
    provenance_ref: &str,
    include_signal_bucket: bool,
    include_path_type: bool,
) -> Result<AuditPayloadMin, StorageError> {
    let mut payload_entries = BTreeMap::new();
    payload_entries.insert(
        PayloadKey::new("feedback_event_type").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(feedback_event_type).map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("learn_signal_type").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(learn_signal_type).map_err(StorageError::ContractViolation)?,
    );
    if include_signal_bucket {
        payload_entries.insert(
            PayloadKey::new("signal_bucket").map_err(StorageError::ContractViolation)?,
            PayloadValue::new(learn_signal_type).map_err(StorageError::ContractViolation)?,
        );
    }
    if include_path_type {
        payload_entries.insert(
            PayloadKey::new("path_type").map_err(StorageError::ContractViolation)?,
            PayloadValue::new(feedback_path_type_label(classify_feedback_path(
                signal.feedback_event_type,
            )))
            .map_err(StorageError::ContractViolation)?,
        );
    }
    payload_entries.insert(
        PayloadKey::new("voice_decision").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(signal.decision).map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("voice_reason_code_hex").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(format!("0x{:X}", signal.reason_code.0))
            .map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("final_identity_decision").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(voice_decision_label(live_context.final_identity_decision))
            .map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("final_reason_code_hex").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(format!("0x{:X}", live_context.final_reason_code.0))
            .map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("identity_tier_v2").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(identity_tier_label(live_context.identity_tier_v2))
            .map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("platform").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(platform_label(live_context.platform))
            .map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("channel").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(channel_label(live_context.channel))
            .map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("evidence_ref").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(evidence_ref).map_err(StorageError::ContractViolation)?,
    );
    payload_entries.insert(
        PayloadKey::new("provenance_ref").map_err(StorageError::ContractViolation)?,
        PayloadValue::new(provenance_ref).map_err(StorageError::ContractViolation)?,
    );
    if let Some(score) = signal.score_bp {
        payload_entries.insert(
            PayloadKey::new("voice_score_bp").map_err(StorageError::ContractViolation)?,
            PayloadValue::new(score.to_string()).map_err(StorageError::ContractViolation)?,
        );
    }
    if let Some(margin) = signal.margin_to_next_bp {
        payload_entries.insert(
            PayloadKey::new("voice_margin_to_next_bp").map_err(StorageError::ContractViolation)?,
            PayloadValue::new(margin.to_string()).map_err(StorageError::ContractViolation)?,
        );
    }
    AuditPayloadMin::v1(payload_entries).map_err(StorageError::ContractViolation)
}

fn unscoped_voice_feedback_payload(
    signal: VoiceFeedbackLearnSignal,
    live_context: VoiceIdFeedbackLearnAuditContext,
    feedback_event_type: &str,
    learn_signal_type: &str,
    evidence_ref: &str,
    provenance_ref: &str,
) -> Result<AuditPayloadMin, StorageError> {
    unscoped_voice_feedback_learn_payload_base(
        signal,
        live_context,
        feedback_event_type,
        learn_signal_type,
        evidence_ref,
        provenance_ref,
        true,
        true,
    )
}

fn unscoped_voice_learn_payload(
    signal: VoiceFeedbackLearnSignal,
    live_context: VoiceIdFeedbackLearnAuditContext,
    feedback_event_type: &str,
    learn_signal_type: &str,
    evidence_ref: &str,
    provenance_ref: &str,
) -> Result<AuditPayloadMin, StorageError> {
    unscoped_voice_feedback_learn_payload_base(
        signal,
        live_context,
        feedback_event_type,
        learn_signal_type,
        evidence_ref,
        provenance_ref,
        true,
        true,
    )
}

fn emit_voice_id_feedback_and_learn_signal(
    store: &mut Ph1fStore,
    signal_scope: &VoiceIdentitySignalScope,
    signal: VoiceFeedbackLearnSignal,
    live_context: VoiceIdFeedbackLearnAuditContext,
) -> Result<(), StorageError> {
    let ingest_started = Instant::now();
    let feedback_event_type = feedback_event_type_label(signal.feedback_event_type);
    let learn_signal_type = learn_signal_type_label(signal.learn_signal_type);
    let evidence_ref = format!(
        "voice_feedback_evidence:{}:{}:{}:{}",
        signal_scope.actor_user_id.as_str(),
        signal_scope.correlation_id.0,
        signal_scope.turn_id.0,
        feedback_event_type
    );
    let provenance_ref = format!("ph1.voice.id:feedback:{feedback_event_type}:v1");

    if let (Some(tenant_id), Some(device_id)) = (
        signal_scope.tenant_id.clone(),
        signal_scope.device_id.clone(),
    ) {
        let learn_audit_payload_metadata = voice_learn_audit_payload_metadata(signal, live_context);
        let mut feedback_payload_metadata =
            voice_feedback_audit_payload_metadata(signal, live_context);
        feedback_payload_metadata.insert("evidence_ref".to_string(), evidence_ref.clone());
        feedback_payload_metadata.insert("provenance_ref".to_string(), provenance_ref.clone());

        store.ph1feedback_event_commit_with_payload_metadata(
            signal_scope.now,
            tenant_id.clone(),
            signal_scope.correlation_id,
            signal_scope.turn_id,
            None,
            signal_scope.actor_user_id.clone(),
            device_id.clone(),
            feedback_event_type.to_string(),
            learn_signal_type.to_string(),
            signal.reason_code,
            format!(
                "voice_feedback:{}:{}:{}:{}:{}",
                signal_scope.actor_user_id.as_str(),
                signal_scope.correlation_id.0,
                signal_scope.turn_id.0,
                feedback_event_type,
                signal.reason_code.0
            ),
            feedback_payload_metadata,
        )?;
        let ingest_latency_ms = ingest_started
            .elapsed()
            .as_millis()
            .min(u128::from(u32::MAX)) as u32;
        store.ph1feedback_learn_signal_bundle_commit_with_audit_payload_metadata(
            signal_scope.now,
            tenant_id,
            signal_scope.correlation_id,
            signal_scope.turn_id,
            None,
            signal_scope.actor_user_id.clone(),
            device_id,
            feedback_event_type.to_string(),
            learn_signal_type.to_string(),
            signal.reason_code,
            evidence_ref,
            provenance_ref,
            ingest_latency_ms,
            format!(
                "voice_learn_bundle:{}:{}:{}:{}:{}",
                signal_scope.actor_user_id.as_str(),
                signal_scope.correlation_id.0,
                signal_scope.turn_id.0,
                learn_signal_type,
                signal.reason_code.0
            ),
            learn_audit_payload_metadata,
        )?;
        return Ok(());
    }

    let feedback_payload = unscoped_voice_feedback_payload(
        signal,
        live_context,
        feedback_event_type,
        learn_signal_type,
        evidence_ref.as_str(),
        provenance_ref.as_str(),
    )?;
    let fallback_feedback_event = AuditEventInput::v1(
        signal_scope.now,
        None,
        None,
        None,
        Some(signal_scope.actor_user_id.clone()),
        None,
        AuditEngine::Other("PH1.FEEDBACK".to_string()),
        AuditEventType::Other,
        signal.reason_code,
        AuditSeverity::Info,
        signal_scope.correlation_id,
        signal_scope.turn_id,
        feedback_payload,
        None,
        Some(format!(
            "voice_feedback_unscoped:{}:{}:{}:{}",
            signal_scope.actor_user_id.as_str(),
            signal_scope.correlation_id.0,
            signal_scope.turn_id.0,
            signal.reason_code.0
        )),
    )
    .map_err(StorageError::ContractViolation)?;
    Ph1jRuntime::emit(store, fallback_feedback_event)?;

    let learn_payload = unscoped_voice_learn_payload(
        signal,
        live_context,
        feedback_event_type,
        learn_signal_type,
        evidence_ref.as_str(),
        provenance_ref.as_str(),
    )?;
    let fallback_learn_event = AuditEventInput::v1(
        signal_scope.now,
        None,
        None,
        None,
        Some(signal_scope.actor_user_id.clone()),
        None,
        AuditEngine::Other("PH1.LEARN".to_string()),
        AuditEventType::Other,
        signal.reason_code,
        AuditSeverity::Info,
        signal_scope.correlation_id,
        signal_scope.turn_id,
        learn_payload,
        None,
        Some(format!(
            "voice_learn_unscoped:{}:{}:{}:{}:{}",
            signal_scope.actor_user_id.as_str(),
            signal_scope.correlation_id.0,
            signal_scope.turn_id.0,
            learn_signal_type,
            signal.reason_code.0
        )),
    )
    .map_err(StorageError::ContractViolation)?;
    Ph1jRuntime::emit(store, fallback_learn_event)?;
    Ok(())
}

#[derive(Debug, Default, Clone)]
pub struct Ph1VoiceIdRuntime;

impl Ph1VoiceIdRuntime {
    pub fn run(
        &self,
        store: &mut Ph1fStore,
        req: &Ph1VoiceIdSimRequest,
    ) -> Result<Ph1VoiceIdSimResponse, StorageError> {
        self.run_for_implementation(store, PH1VOICEID_IMPLEMENTATION_ID, req)
    }

    pub fn run_for_implementation(
        &self,
        store: &mut Ph1fStore,
        implementation_id: &str,
        req: &Ph1VoiceIdSimRequest,
    ) -> Result<Ph1VoiceIdSimResponse, StorageError> {
        if implementation_id != PH1VOICEID_IMPLEMENTATION_ID {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1_voice_id.implementation_id",
                    reason: "unknown implementation_id",
                },
            ));
        }
        req.validate().map_err(StorageError::ContractViolation)?;

        match &req.request {
            VoiceIdSimulationRequest::EnrollStartDraft(r) => {
                let onboarding_session_id =
                    OnboardingSessionId::new(r.onboarding_session_id.clone())
                        .map_err(StorageError::ContractViolation)?;

                let rec = store.ph1vid_enroll_start_draft(
                    req.now,
                    onboarding_session_id,
                    r.device_id.clone(),
                    r.consent_asserted,
                    r.max_total_attempts,
                    r.max_session_enroll_time_ms,
                    r.lock_after_consecutive_passes,
                )?;

                let out = VoiceIdEnrollStartResult::v1(
                    VoiceEnrollmentSessionId::new(rec.voice_enrollment_session_id.clone())
                        .map_err(StorageError::ContractViolation)?,
                    map_status(rec.voice_enroll_status),
                    rec.max_total_attempts,
                    rec.max_session_enroll_time_ms,
                    rec.lock_after_consecutive_passes,
                )
                .map_err(StorageError::ContractViolation)?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "NONE",
                    status_label(rec.voice_enroll_status),
                    reason_codes::VID_OK_ENROLL_START_DRAFT,
                    Some(format!(
                        "vid_enroll_start:{}",
                        rec.voice_enrollment_session_id
                    )),
                )?;

                Ok(Ph1VoiceIdSimResponse::Ok(
                    Ph1VoiceIdSimOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::VID_OK_ENROLL_START_DRAFT,
                        Some(out),
                        None,
                        None,
                        None,
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }
            VoiceIdSimulationRequest::EnrollSampleCommit(r) => {
                let rec = store.ph1vid_enroll_sample_commit(
                    req.now,
                    r.voice_enrollment_session_id.as_str().to_string(),
                    r.audio_sample_ref.clone(),
                    r.attempt_index,
                    r.sample_duration_ms,
                    r.vad_coverage,
                    r.snr_db,
                    r.clipping_pct,
                    r.overlap_ratio,
                    r.app_embedding_capture_ref.clone(),
                    r.idempotency_key.clone(),
                )?;
                let sample = store
                    .ph1vid_get_sample_for_attempt_and_idempotency(
                        r.voice_enrollment_session_id.as_str(),
                        r.attempt_index,
                        &r.idempotency_key,
                    )
                    .ok_or(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "ph1vid_runtime.sample_lookup",
                            reason: "sample commit row must exist after commit",
                        },
                    ))?;

                let out = VoiceIdEnrollSampleResult::v1(
                    VoiceEnrollmentSessionId::new(rec.voice_enrollment_session_id.clone())
                        .map_err(StorageError::ContractViolation)?,
                    map_sample_result_from_store(sample.result),
                    sample.reason_code.or(rec.reason_code),
                    rec.consecutive_passes,
                    map_status(rec.voice_enroll_status),
                )
                .map_err(StorageError::ContractViolation)?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "VOICE_ENROLL_SAMPLE",
                    status_label(rec.voice_enroll_status),
                    reason_codes::VID_OK_ENROLL_SAMPLE_COMMIT,
                    Some(r.idempotency_key.clone()),
                )?;

                Ok(Ph1VoiceIdSimResponse::Ok(
                    Ph1VoiceIdSimOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::VID_OK_ENROLL_SAMPLE_COMMIT,
                        None,
                        Some(out),
                        None,
                        None,
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }
            VoiceIdSimulationRequest::EnrollCompleteCommit(r) => {
                let rec = store.ph1vid_enroll_complete_commit(
                    req.now,
                    r.voice_enrollment_session_id.as_str().to_string(),
                    r.idempotency_key.clone(),
                )?;

                let voice_profile_id =
                    rec.voice_profile_id
                        .clone()
                        .ok_or(StorageError::ContractViolation(
                            ContractViolation::InvalidValue {
                                field: "ph1vid_runtime.voice_profile_id",
                                reason: "must be present after complete commit",
                            },
                        ))?;

                let out = VoiceIdEnrollCompleteResult::v1_with_sync_receipt(
                    VoiceEnrollmentSessionId::new(rec.voice_enrollment_session_id.clone())
                        .map_err(StorageError::ContractViolation)?,
                    voice_profile_id,
                    map_status(rec.voice_enroll_status),
                    rec.voice_artifact_sync_receipt_ref.clone(),
                )
                .map_err(StorageError::ContractViolation)?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "VOICE_ENROLL_LOCKED",
                    status_label(rec.voice_enroll_status),
                    reason_codes::VID_OK_ENROLL_COMPLETE_COMMIT,
                    Some(r.idempotency_key.clone()),
                )?;

                Ok(Ph1VoiceIdSimResponse::Ok(
                    Ph1VoiceIdSimOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::VID_OK_ENROLL_COMPLETE_COMMIT,
                        None,
                        None,
                        Some(out),
                        None,
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }
            VoiceIdSimulationRequest::EnrollDeferCommit(r) => {
                let rec = store.ph1vid_enroll_defer_commit(
                    req.now,
                    r.voice_enrollment_session_id.as_str().to_string(),
                    r.reason_code,
                    r.idempotency_key.clone(),
                )?;

                let out = VoiceIdEnrollDeferResult::v1(
                    VoiceEnrollmentSessionId::new(rec.voice_enrollment_session_id.clone())
                        .map_err(StorageError::ContractViolation)?,
                    map_status(rec.voice_enroll_status),
                    rec.reason_code.unwrap_or(r.reason_code),
                )
                .map_err(StorageError::ContractViolation)?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "VOICE_ENROLL_IN_PROGRESS",
                    status_label(rec.voice_enroll_status),
                    reason_codes::VID_OK_ENROLL_DEFER_COMMIT,
                    Some(r.idempotency_key.clone()),
                )?;

                Ok(Ph1VoiceIdSimResponse::Ok(
                    Ph1VoiceIdSimOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::VID_OK_ENROLL_DEFER_COMMIT,
                        None,
                        None,
                        None,
                        Some(out),
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }
        }
    }

    fn audit_transition(
        &self,
        store: &mut Ph1fStore,
        now: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        state_from: &'static str,
        state_to: &'static str,
        reason_code: ReasonCodeId,
        idempotency_key: Option<String>,
    ) -> Result<(), StorageError> {
        let mut entries: BTreeMap<PayloadKey, PayloadValue> = BTreeMap::new();
        entries.insert(
            PayloadKey::new("state_from").map_err(StorageError::ContractViolation)?,
            PayloadValue::new(state_from).map_err(StorageError::ContractViolation)?,
        );
        entries.insert(
            PayloadKey::new("state_to").map_err(StorageError::ContractViolation)?,
            PayloadValue::new(state_to).map_err(StorageError::ContractViolation)?,
        );
        let payload_min = AuditPayloadMin::v1(entries).map_err(StorageError::ContractViolation)?;

        let ev = AuditEventInput::v1(
            now,
            None,
            None,
            None,
            None,
            None,
            AuditEngine::Other("ph1_voice_id".to_string()),
            AuditEventType::StateTransition,
            reason_code,
            AuditSeverity::Info,
            correlation_id,
            turn_id,
            payload_min,
            None,
            idempotency_key,
        )
        .map_err(StorageError::ContractViolation)?;

        Ph1jRuntime::emit(store, ev)?;
        Ok(())
    }
}

fn map_status(v: StoreVoiceEnrollStatus) -> ContractVoiceEnrollStatus {
    match v {
        StoreVoiceEnrollStatus::InProgress => ContractVoiceEnrollStatus::InProgress,
        StoreVoiceEnrollStatus::Locked => ContractVoiceEnrollStatus::Locked,
        StoreVoiceEnrollStatus::Pending => ContractVoiceEnrollStatus::Pending,
        StoreVoiceEnrollStatus::Declined => ContractVoiceEnrollStatus::Declined,
    }
}

fn status_label(v: StoreVoiceEnrollStatus) -> &'static str {
    match v {
        StoreVoiceEnrollStatus::InProgress => "VOICE_ENROLL_IN_PROGRESS",
        StoreVoiceEnrollStatus::Locked => "VOICE_ENROLL_LOCKED",
        StoreVoiceEnrollStatus::Pending => "VOICE_ENROLL_PENDING",
        StoreVoiceEnrollStatus::Declined => "VOICE_ENROLL_DECLINED",
    }
}

fn map_sample_result_from_store(v: StoreVoiceSampleResult) -> ContractVoiceSampleResult {
    match v {
        StoreVoiceSampleResult::Pass => ContractVoiceSampleResult::Pass,
        StoreVoiceSampleResult::Fail => ContractVoiceSampleResult::Fail,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_engines::ph1_voice_id::{
        reason_codes as engine_voice_reason_codes, simulation_profile_embedding_from_seed,
    };
    use selene_kernel_contracts::ph1_voice_id::{
        DeviceTrustLevel, DiarizationSegment, Ph1VoiceIdRequest, Ph1VoiceIdResponse,
        Ph1VoiceIdSimRequest, SpeakerAssertionOk, SpeakerLabel, VoiceIdEnrollStartDraftRequest,
        VoiceIdSimulationRequest, VoiceIdSimulationType, PH1VOICEID_SIM_CONTRACT_VERSION,
    };
    use selene_kernel_contracts::ph1art::{
        ArtifactScopeType, ArtifactStatus, ArtifactType, ArtifactVersion,
    };
    use selene_kernel_contracts::ph1j::{AuditEngine, CorrelationId, DeviceId, PayloadKey, TurnId};
    use selene_kernel_contracts::ph1k::{
        AudioDeviceId, AudioFormat, AudioStreamId, AudioStreamKind, AudioStreamRef, ChannelCount,
        Confidence, FrameDurationMs, SampleFormat, SampleRateHz, SpeechLikeness, VadEvent,
    };
    use selene_kernel_contracts::ph1l::{
        NextAllowedActions, SessionId, SessionSnapshot, PH1L_CONTRACT_VERSION,
    };
    use selene_kernel_contracts::ph1link::AppPlatform;
    use selene_kernel_contracts::{MonotonicTimeNs, SessionState};
    use selene_storage::ph1f::{DeviceRecord, IdentityRecord, IdentityStatus};

    fn sample_start_request() -> Ph1VoiceIdSimRequest {
        Ph1VoiceIdSimRequest {
            schema_version: PH1VOICEID_SIM_CONTRACT_VERSION,
            correlation_id: CorrelationId(1),
            turn_id: TurnId(1),
            now: MonotonicTimeNs(1),
            simulation_id: "VOICE_ID_ENROLL_START_DRAFT".to_string(),
            simulation_type: VoiceIdSimulationType::Draft,
            request: VoiceIdSimulationRequest::EnrollStartDraft(VoiceIdEnrollStartDraftRequest {
                onboarding_session_id: "onb_1".to_string(),
                device_id: DeviceId::new("device_1").unwrap(),
                consent_asserted: true,
                max_total_attempts: 8,
                max_session_enroll_time_ms: 120_000,
                lock_after_consecutive_passes: 3,
            }),
        }
    }

    fn sample_live_request() -> Ph1VoiceIdRequest {
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
            MonotonicTimeNs(1),
            MonotonicTimeNs(2),
            Confidence::new(0.95).unwrap(),
            SpeechLikeness::new(0.95).unwrap(),
        )];
        let session_state_ref = SessionSnapshot {
            schema_version: PH1L_CONTRACT_VERSION,
            session_state: SessionState::Active,
            session_id: Some(SessionId(1)),
            next_allowed_actions: NextAllowedActions {
                may_speak: true,
                must_wait: false,
                must_rewake: false,
            },
        };
        Ph1VoiceIdRequest::v1(
            MonotonicTimeNs(3),
            processed_stream_ref,
            vad_events,
            AudioDeviceId::new("live_dev_1").unwrap(),
            session_state_ref,
            None,
            false,
            DeviceTrustLevel::Trusted,
            None,
        )
        .unwrap()
    }

    fn commit_voice_artifact(
        store: &mut Ph1fStore,
        tenant_id: &str,
        artifact_type: ArtifactType,
        artifact_version: ArtifactVersion,
        payload_ref: String,
        status: ArtifactStatus,
        now: u64,
        idempotency_key: &str,
    ) {
        if status == ArtifactStatus::Active {
            store
                .ph1builder_active_artifact_commit(
                    MonotonicTimeNs(now),
                    tenant_id.to_string(),
                    ArtifactScopeType::Tenant,
                    tenant_id.to_string(),
                    artifact_type,
                    artifact_version,
                    format!("pkg_hash_{}_{}", tenant_id, artifact_version.0),
                    payload_ref,
                    format!("prov_{}_{}", tenant_id, artifact_version.0),
                    idempotency_key.to_string(),
                )
                .expect("voice artifact commit must succeed");
            return;
        }

        store
            .ph1learn_artifact_commit(
                MonotonicTimeNs(now),
                tenant_id.to_string(),
                ArtifactScopeType::Tenant,
                tenant_id.to_string(),
                artifact_type,
                artifact_version,
                format!("pkg_hash_{}_{}", tenant_id, artifact_version.0),
                payload_ref,
                format!("prov_{}_{}", tenant_id, artifact_version.0),
                status,
                idempotency_key.to_string(),
            )
            .expect("voice artifact commit must succeed");
    }

    fn sample_signal_scope(correlation_id: u128, turn_id: u64) -> VoiceIdentitySignalScope {
        VoiceIdentitySignalScope::v1(
            MonotonicTimeNs(3),
            CorrelationId(correlation_id),
            TurnId(turn_id),
            UserId::new(&format!("user_audit_{correlation_id}")).unwrap(),
            None,
            None,
        )
    }

    fn sample_scoped_signal_scope(correlation_id: u128, turn_id: u64) -> VoiceIdentitySignalScope {
        VoiceIdentitySignalScope::v1(
            MonotonicTimeNs(3),
            CorrelationId(correlation_id),
            TurnId(turn_id),
            UserId::new(&format!("user_scoped_audit_{correlation_id}")).unwrap(),
            Some("tenant_audit".to_string()),
            Some(DeviceId::new(&format!("device_scoped_{correlation_id}")).unwrap()),
        )
    }

    fn seed_scoped_feedback_identity_device(
        store: &mut Ph1fStore,
        signal_scope: &VoiceIdentitySignalScope,
    ) {
        let device_id = signal_scope
            .device_id
            .clone()
            .expect("scoped signal scope must carry device_id");
        store
            .insert_identity(IdentityRecord::v1(
                signal_scope.actor_user_id.clone(),
                None,
                None,
                signal_scope.now,
                IdentityStatus::Active,
            ))
            .expect("scoped feedback identity seed must succeed");
        store
            .insert_device(
                DeviceRecord::v1(
                    device_id,
                    signal_scope.actor_user_id.clone(),
                    "phone".to_string(),
                    signal_scope.now,
                    None,
                )
                .expect("scoped feedback device seed must be valid"),
            )
            .expect("scoped feedback device seed must succeed");
    }

    fn sample_decision_segments() -> Vec<DiarizationSegment> {
        vec![DiarizationSegment::v1(
            MonotonicTimeNs(1),
            MonotonicTimeNs(2),
            Some(SpeakerLabel::speaker_a()),
        )
        .unwrap()]
    }

    #[test]
    fn at_vid_impl_01_unknown_implementation_fails_closed() {
        let runtime = Ph1VoiceIdRuntime;
        let mut store = Ph1fStore::new_in_memory();
        let out =
            runtime.run_for_implementation(&mut store, "PH1.VOICE.ID.999", &sample_start_request());
        assert!(matches!(
            out,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1_voice_id.implementation_id",
                    reason: "unknown implementation_id",
                }
            ))
        ));
    }

    #[test]
    fn at_vid_impl_02_active_implementation_list_is_locked() {
        assert_eq!(PH1_VOICE_ID_ENGINE_ID, "PH1.VOICE.ID");
        assert_eq!(
            PH1_VOICE_ID_ACTIVE_IMPLEMENTATION_IDS,
            &["PH1.VOICE.ID.001"]
        );
    }

    #[test]
    fn at_vid_live_gate_01_profiles_are_platform_channel_scoped() {
        let runtime = Ph1VoiceIdLiveRuntime::default();
        let ios_explicit = VoiceIdentityRuntimeContext::from_app_platform(
            Some(AppPlatform::Ios),
            VoiceIdentityChannel::Explicit,
        );
        let android_wake = VoiceIdentityRuntimeContext::from_app_platform(
            Some(AppPlatform::Android),
            VoiceIdentityChannel::WakeWord,
        );
        let unknown_explicit =
            VoiceIdentityRuntimeContext::from_app_platform(None, VoiceIdentityChannel::Explicit);
        let desktop_explicit = VoiceIdentityRuntimeContext::v1(
            VoiceIdentityPlatform::Desktop,
            VoiceIdentityChannel::Explicit,
        );

        assert!(
            runtime
                .embedding_gate_profile_for(ios_explicit)
                .require_primary_embedding
        );
        assert!(
            runtime
                .embedding_gate_profile_for(android_wake)
                .require_primary_embedding
        );
        assert!(
            runtime
                .embedding_gate_profile_for(unknown_explicit)
                .require_primary_embedding
        );
        assert!(
            !runtime
                .embedding_gate_profile_for(desktop_explicit)
                .require_primary_embedding
        );
    }

    #[test]
    fn at_vid_live_gate_02_ios_explicit_fails_closed_without_embedding() {
        let runtime = Ph1VoiceIdLiveRuntime::default();
        let req = sample_live_request();
        let context = VoiceIdentityRuntimeContext::from_app_platform(
            Some(AppPlatform::Ios),
            VoiceIdentityChannel::Explicit,
        );
        let enrolled = vec![EngineEnrolledSpeaker {
            speaker_id: selene_kernel_contracts::ph1_voice_id::SpeakerId::new("speaker_live")
                .unwrap(),
            user_id: Some(selene_kernel_contracts::ph1_voice_id::UserId::new("user_live").unwrap()),
            fingerprint: 7,
            profile_embedding: Some(simulation_profile_embedding_from_seed(7)),
        }];
        let obs = EngineVoiceIdObservation {
            primary_fingerprint: Some(7),
            secondary_fingerprint: None,
            primary_embedding: None,
            secondary_embedding: None,
            spoof_risk: false,
        };

        let out = runtime
            .run_identity_assertion(&req, context, enrolled, obs)
            .expect("live identity assertion should return contract-safe output");
        match out {
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(u) => {
                assert_eq!(
                    u.reason_code,
                    engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE
                );
            }
            _ => panic!("expected fail-closed unknown"),
        }
    }

    #[test]
    fn at_vid_live_gate_03_desktop_explicit_allows_fingerprint_fallback_profile() {
        let runtime = Ph1VoiceIdLiveRuntime::default();
        let req = sample_live_request();
        let context = VoiceIdentityRuntimeContext::v1(
            VoiceIdentityPlatform::Desktop,
            VoiceIdentityChannel::Explicit,
        );
        let enrolled = vec![EngineEnrolledSpeaker {
            speaker_id: selene_kernel_contracts::ph1_voice_id::SpeakerId::new("speaker_live")
                .unwrap(),
            user_id: Some(selene_kernel_contracts::ph1_voice_id::UserId::new("user_live").unwrap()),
            fingerprint: 7,
            profile_embedding: Some(simulation_profile_embedding_from_seed(7)),
        }];
        let obs = EngineVoiceIdObservation {
            primary_fingerprint: Some(7),
            secondary_fingerprint: None,
            primary_embedding: None,
            secondary_embedding: None,
            spoof_risk: false,
        };

        let out = runtime
            .run_identity_assertion(&req, context, enrolled, obs)
            .expect("live identity assertion should return contract-safe output");
        assert!(matches!(out, Ph1VoiceIdResponse::SpeakerAssertionOk(_)));
    }

    #[test]
    fn at_vid_live_gate_04_unknown_platform_fails_closed_without_embedding() {
        let runtime = Ph1VoiceIdLiveRuntime::default();
        let req = sample_live_request();
        let context =
            VoiceIdentityRuntimeContext::from_app_platform(None, VoiceIdentityChannel::Explicit);
        let enrolled = vec![EngineEnrolledSpeaker {
            speaker_id: selene_kernel_contracts::ph1_voice_id::SpeakerId::new("speaker_live")
                .unwrap(),
            user_id: Some(selene_kernel_contracts::ph1_voice_id::UserId::new("user_live").unwrap()),
            fingerprint: 7,
            profile_embedding: Some(simulation_profile_embedding_from_seed(7)),
        }];
        let obs = EngineVoiceIdObservation {
            primary_fingerprint: Some(7),
            secondary_fingerprint: None,
            primary_embedding: None,
            secondary_embedding: None,
            spoof_risk: false,
        };

        let out = runtime
            .run_identity_assertion(&req, context, enrolled, obs)
            .expect("live identity assertion should return contract-safe output");
        match out {
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(u) => {
                assert_eq!(
                    u.reason_code,
                    engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE
                );
            }
            _ => panic!("expected fail-closed unknown"),
        }
    }

    #[test]
    fn at_vid_live_gate_05_tenant_override_profile_applies_before_global_default() {
        let mut relaxed_profiles = VoiceIdentityEmbeddingGateProfiles::mvp_v1_phone_first();
        relaxed_profiles.android_wake = VoiceIdentityEmbeddingGateProfile::optional();
        relaxed_profiles.android_explicit = VoiceIdentityEmbeddingGateProfile::optional();

        let config = Ph1VoiceIdLiveConfig::mvp_v1_phone_first()
            .with_tenant_embedding_gate_override("tenant_relaxed", relaxed_profiles)
            .expect("tenant override must accept deterministic tenant id");
        let runtime = Ph1VoiceIdLiveRuntime::new(config);

        let relaxed_context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_relaxed".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::WakeWord,
        );
        let strict_context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_strict".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::WakeWord,
        );

        assert!(
            !runtime
                .embedding_gate_profile_for(relaxed_context)
                .require_primary_embedding
        );
        assert!(
            runtime
                .embedding_gate_profile_for(strict_context)
                .require_primary_embedding
        );
    }

    #[test]
    fn at_vid_live_gate_06_invalid_tenant_override_id_fails_contract() {
        let config = Ph1VoiceIdLiveConfig::mvp_v1_phone_first()
            .with_tenant_embedding_gate_override("tenant-with-unicode-\u{2603}", {
                let mut p = VoiceIdentityEmbeddingGateProfiles::mvp_v1_phone_first();
                p.android_explicit = VoiceIdentityEmbeddingGateProfile::optional();
                p
            });

        assert!(matches!(
            config,
            Err(ContractViolation::InvalidValue {
                field: "voice_identity_embedding_gate_governed_config.tenant_overrides.tenant_id",
                reason: "must be non-empty ASCII and <= 64 chars",
            })
        ));
    }

    #[test]
    fn at_vid_live_gate_07_governed_threshold_pack_override_is_applied_by_tenant() {
        let mut store = Ph1fStore::new_in_memory();
        let runtime = Ph1VoiceIdLiveRuntime::default();
        let mut override_profiles = VoiceIdentityEmbeddingGateProfiles::mvp_v1_phone_first();
        override_profiles.android_explicit = VoiceIdentityEmbeddingGateProfile::optional();

        store
            .ph1builder_active_artifact_commit(
                MonotonicTimeNs(11),
                "tenant_relaxed".to_string(),
                ArtifactScopeType::Tenant,
                "tenant_relaxed".to_string(),
                ArtifactType::VoiceIdThresholdPack,
                ArtifactVersion(1),
                "pkg_hash_vid_gate_1".to_string(),
                override_profiles.to_payload_ref_v1(),
                "prov_vid_gate_1".to_string(),
                "idem_vid_gate_1".to_string(),
            )
            .expect("voice-id threshold pack commit must succeed");

        let governed_runtime = runtime.with_governed_threshold_pack_overrides(&store);
        let relaxed_context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_relaxed".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let strict_context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_strict".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );

        assert!(
            !governed_runtime
                .embedding_gate_profile_for(relaxed_context)
                .require_primary_embedding
        );
        assert!(
            governed_runtime
                .embedding_gate_profile_for(strict_context)
                .require_primary_embedding
        );
    }

    #[test]
    fn at_vid_live_gate_08_governed_override_prefers_highest_active_version() {
        let mut store = Ph1fStore::new_in_memory();
        let runtime = Ph1VoiceIdLiveRuntime::default();
        let mut v1_profiles = VoiceIdentityEmbeddingGateProfiles::mvp_v1_phone_first();
        v1_profiles.android_explicit = VoiceIdentityEmbeddingGateProfile::optional();
        let mut v2_profiles = VoiceIdentityEmbeddingGateProfiles::mvp_v1_phone_first();
        v2_profiles.android_explicit = VoiceIdentityEmbeddingGateProfile::required();

        store
            .ph1builder_active_artifact_commit(
                MonotonicTimeNs(21),
                "tenant_rollout".to_string(),
                ArtifactScopeType::Tenant,
                "tenant_rollout".to_string(),
                ArtifactType::VoiceIdThresholdPack,
                ArtifactVersion(1),
                "pkg_hash_vid_gate_v1".to_string(),
                v1_profiles.to_payload_ref_v1(),
                "prov_vid_gate_v1".to_string(),
                "idem_vid_gate_v1".to_string(),
            )
            .expect("voice-id threshold pack v1 commit must succeed");
        store
            .ph1builder_active_artifact_commit(
                MonotonicTimeNs(22),
                "tenant_rollout".to_string(),
                ArtifactScopeType::Tenant,
                "tenant_rollout".to_string(),
                ArtifactType::VoiceIdThresholdPack,
                ArtifactVersion(2),
                "pkg_hash_vid_gate_v2".to_string(),
                v2_profiles.to_payload_ref_v1(),
                "prov_vid_gate_v2".to_string(),
                "idem_vid_gate_v2".to_string(),
            )
            .expect("voice-id threshold pack v2 commit must succeed");

        let governed_runtime = runtime.with_governed_threshold_pack_overrides(&store);
        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_rollout".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );

        assert!(
            governed_runtime
                .embedding_gate_profile_for(context)
                .require_primary_embedding
        );
    }

    #[test]
    fn at_vid_live_gate_09_from_app_platform_maps_desktop() {
        assert_eq!(
            VoiceIdentityPlatform::from_app_platform(Some(AppPlatform::Desktop)),
            VoiceIdentityPlatform::Desktop
        );
    }

    #[test]
    fn at_vid_live_gate_10_tenant_artifact_pointers_select_active_and_rollback_for_all_voice_packs()
    {
        let mut store = Ph1fStore::new_in_memory();
        let runtime = Ph1VoiceIdLiveRuntime::default();
        let tenant_id = "tenant_vid_pointers";

        let pack_types = [
            ArtifactType::VoiceIdThresholdPack,
            ArtifactType::VoiceIdConfusionPairPack,
            ArtifactType::VoiceIdSpoofPolicyPack,
            ArtifactType::VoiceIdProfileDeltaPack,
        ];
        for (idx, artifact_type) in pack_types.into_iter().enumerate() {
            let base = 100 + (idx as u64) * 10;
            let payload_v1 = format!("voice_pack_{idx}_v1");
            let payload_v2 = format!("voice_pack_{idx}_v2");
            let payload_v3 = format!("voice_pack_{idx}_v3");
            commit_voice_artifact(
                &mut store,
                tenant_id,
                artifact_type,
                ArtifactVersion(1),
                payload_v1,
                ArtifactStatus::Active,
                base,
                &format!("idem_{idx}_v1"),
            );
            commit_voice_artifact(
                &mut store,
                tenant_id,
                artifact_type,
                ArtifactVersion(2),
                payload_v2,
                ArtifactStatus::RolledBack,
                base + 1,
                &format!("idem_{idx}_v2"),
            );
            commit_voice_artifact(
                &mut store,
                tenant_id,
                artifact_type,
                ArtifactVersion(3),
                payload_v3,
                ArtifactStatus::Active,
                base + 2,
                &format!("idem_{idx}_v3"),
            );
        }

        let pointers = runtime.tenant_artifact_pointers(&store, tenant_id);
        let sets = [
            pointers.threshold_pack,
            pointers.confusion_pair_pack,
            pointers.spoof_policy_pack,
            pointers.profile_delta_pack,
        ];
        for set in sets {
            let active = set.active.expect("active pointer must exist");
            let rollback = set.rollback.expect("rollback pointer must exist");
            assert_eq!(active.artifact_version, ArtifactVersion(3));
            assert_eq!(active.status, ArtifactStatus::Active);
            assert_eq!(rollback.artifact_version, ArtifactVersion(2));
            assert_eq!(rollback.status, ArtifactStatus::RolledBack);
        }
    }

    #[test]
    fn at_vid_live_gate_11_tenant_artifact_pointer_fallback_without_active_uses_latest_and_n1() {
        let mut store = Ph1fStore::new_in_memory();
        let runtime = Ph1VoiceIdLiveRuntime::default();
        let tenant_id = "tenant_vid_pointer_fallback";

        commit_voice_artifact(
            &mut store,
            tenant_id,
            ArtifactType::VoiceIdThresholdPack,
            ArtifactVersion(1),
            "voice_pack_threshold_v1".to_string(),
            ArtifactStatus::Deprecated,
            201,
            "idem_threshold_v1",
        );
        commit_voice_artifact(
            &mut store,
            tenant_id,
            ArtifactType::VoiceIdThresholdPack,
            ArtifactVersion(2),
            "voice_pack_threshold_v2".to_string(),
            ArtifactStatus::RolledBack,
            202,
            "idem_threshold_v2",
        );

        let pointers = runtime.tenant_artifact_pointers(&store, tenant_id);
        let active = pointers
            .threshold_pack
            .active
            .expect("latest pointer must be selected as active fallback");
        let rollback = pointers
            .threshold_pack
            .rollback
            .expect("n-1 pointer must be available");
        assert_eq!(active.artifact_version, ArtifactVersion(2));
        assert_eq!(active.status, ArtifactStatus::RolledBack);
        assert_eq!(rollback.artifact_version, ArtifactVersion(1));
        assert_eq!(rollback.status, ArtifactStatus::Deprecated);
    }

    #[test]
    fn at_vid_live_gate_12_cohort_kpi_audit_emits_tar_frr_far_and_latency() {
        let mut store = Ph1fStore::new_in_memory();
        let actor = UserId::new("tenant_kpi:user_1").unwrap();
        let device = DeviceId::new("voice-kpi-device-1").unwrap();
        let runtime = Ph1VoiceIdLiveRuntime::default();
        let req = sample_live_request();
        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_kpi".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let enrolled = vec![EngineEnrolledSpeaker {
            speaker_id: selene_kernel_contracts::ph1_voice_id::SpeakerId::new("speaker_kpi")
                .unwrap(),
            user_id: Some(selene_kernel_contracts::ph1_voice_id::UserId::new("user_kpi").unwrap()),
            fingerprint: 7,
            profile_embedding: Some(simulation_profile_embedding_from_seed(7)),
        }];
        let obs = EngineVoiceIdObservation {
            primary_fingerprint: Some(7),
            secondary_fingerprint: None,
            primary_embedding: Some(simulation_profile_embedding_from_seed(7)),
            secondary_embedding: None,
            spoof_risk: false,
        };
        runtime
            .run_identity_assertion_with_signal_emission(
                &mut store,
                &req,
                context,
                enrolled,
                obs,
                VoiceIdentitySignalScope::v1(
                    MonotonicTimeNs(3),
                    CorrelationId(5512),
                    TurnId(1),
                    actor,
                    Some("tenant_kpi".to_string()),
                    Some(device),
                ),
            )
            .expect("identity run with kpi emission must succeed");

        let kpi_row = store
            .audit_events_by_correlation(CorrelationId(5512))
            .into_iter()
            .find(|row| {
                matches!(&row.engine, AuditEngine::Other(engine) if engine == PH1_VOICE_ID_ENGINE_ID)
                    && row.payload_min.entries.contains_key(
                        &PayloadKey::new("metric_family")
                            .expect("metric_family payload key is valid"),
                    )
            })
            .expect("voice-id cohort kpi audit row must exist");
        let payload = &kpi_row.payload_min.entries;
        assert_eq!(
            payload
                .get(&PayloadKey::new("metric_family").unwrap())
                .expect("metric_family must exist")
                .as_str(),
            "voice_id_cohort_kpi"
        );
        assert!(payload.contains_key(&PayloadKey::new("tar").unwrap()));
        assert!(payload.contains_key(&PayloadKey::new("frr").unwrap()));
        assert!(payload.contains_key(&PayloadKey::new("far").unwrap()));
        assert!(payload.contains_key(&PayloadKey::new("latency_ms").unwrap()));
        assert!(payload.contains_key(&PayloadKey::new("cohort_device").unwrap()));
        assert!(payload.contains_key(&PayloadKey::new("cohort_noise").unwrap()));
    }

    #[test]
    fn at_vid_live_gate_13_revoked_active_voice_artifact_falls_back_to_last_good_pointer() {
        let mut store = Ph1fStore::new_in_memory();
        let runtime = Ph1VoiceIdLiveRuntime::default();
        let tenant_id = "tenant_vid_revocation_fallback";

        commit_voice_artifact(
            &mut store,
            tenant_id,
            ArtifactType::VoiceIdThresholdPack,
            ArtifactVersion(1),
            "threshold_v1".to_string(),
            ArtifactStatus::Active,
            301,
            "idem_threshold_v1",
        );
        commit_voice_artifact(
            &mut store,
            tenant_id,
            ArtifactType::VoiceIdThresholdPack,
            ArtifactVersion(2),
            "threshold_v2".to_string(),
            ArtifactStatus::RolledBack,
            302,
            "idem_threshold_v2",
        );
        commit_voice_artifact(
            &mut store,
            tenant_id,
            ArtifactType::VoiceIdThresholdPack,
            ArtifactVersion(3),
            "threshold_v3".to_string(),
            ArtifactStatus::Active,
            303,
            "idem_threshold_v3",
        );
        commit_voice_artifact(
            &mut store,
            tenant_id,
            ArtifactType::VoiceIdConfusionPairPack,
            ArtifactVersion(1),
            "confusion_v1".to_string(),
            ArtifactStatus::Active,
            304,
            "idem_confusion_v1",
        );

        store
            .ph1builder_voice_artifact_revocation_commit(
                MonotonicTimeNs(305),
                tenant_id.to_string(),
                ArtifactType::VoiceIdThresholdPack,
                ArtifactVersion(3),
                "decision_threshold_revoke".to_string(),
                "idem_threshold_revoke".to_string(),
            )
            .expect("threshold revocation must succeed");
        store
            .ph1builder_voice_artifact_revocation_commit(
                MonotonicTimeNs(306),
                tenant_id.to_string(),
                ArtifactType::VoiceIdConfusionPairPack,
                ArtifactVersion(1),
                "decision_confusion_revoke".to_string(),
                "idem_confusion_revoke".to_string(),
            )
            .expect("confusion revocation must succeed");

        let pointers = runtime.tenant_artifact_pointers(&store, tenant_id);
        let threshold_active = pointers
            .threshold_pack
            .active
            .expect("threshold must fall back to the highest lower non-revoked version");
        let threshold_rollback = pointers
            .threshold_pack
            .rollback
            .expect("threshold must still expose rollback visibility");
        assert_eq!(threshold_active.artifact_version, ArtifactVersion(2));
        assert_eq!(threshold_active.status, ArtifactStatus::RolledBack);
        assert_eq!(threshold_rollback.artifact_version, ArtifactVersion(1));
        assert_eq!(threshold_rollback.status, ArtifactStatus::Active);
        assert!(pointers.confusion_pair_pack.active.is_none());
        assert!(pointers.confusion_pair_pack.rollback.is_none());
    }

    #[test]
    fn at_vid_live_gate_14_contract_migration_stage_m1_emits_shadow_audit_with_v1_read_mode() {
        let mut store = Ph1fStore::new_in_memory();
        let actor = UserId::new("tenant_mig:user_1").unwrap();
        let device = DeviceId::new("voice-mig-device-1").unwrap();
        let runtime = Ph1VoiceIdLiveRuntime::new(
            Ph1VoiceIdLiveConfig::mvp_v1_phone_first()
                .with_contract_migration_stage(VoiceIdContractMigrationStage::M1),
        );
        let req = sample_live_request();
        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_mig".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let enrolled = vec![EngineEnrolledSpeaker {
            speaker_id: selene_kernel_contracts::ph1_voice_id::SpeakerId::new("speaker_mig")
                .unwrap(),
            user_id: Some(selene_kernel_contracts::ph1_voice_id::UserId::new("user_mig").unwrap()),
            fingerprint: 7,
            profile_embedding: Some(simulation_profile_embedding_from_seed(7)),
        }];
        let out = runtime
            .run_identity_assertion_with_signal_emission(
                &mut store,
                &req,
                context,
                enrolled,
                EngineVoiceIdObservation {
                    primary_fingerprint: Some(7),
                    secondary_fingerprint: None,
                    primary_embedding: Some(simulation_profile_embedding_from_seed(7)),
                    secondary_embedding: None,
                    spoof_risk: false,
                },
                VoiceIdentitySignalScope::v1(
                    MonotonicTimeNs(3),
                    CorrelationId(5513),
                    TurnId(1),
                    actor,
                    Some("tenant_mig".to_string()),
                    Some(device),
                ),
            )
            .expect("identity run with migration stage m1 must succeed");
        assert_eq!(
            out.identity_v2().identity_tier_v2,
            IdentityTierV2::Confirmed
        );

        let migration_row = store
            .audit_events_by_correlation(CorrelationId(5513))
            .into_iter()
            .find(|row| {
                matches!(&row.engine, AuditEngine::Other(engine) if engine == PH1_VOICE_ID_ENGINE_ID)
                    && row.payload_min.entries.contains_key(
                        &PayloadKey::new("migration_stage")
                            .expect("migration_stage payload key is valid"),
                    )
            })
            .expect("migration audit row must exist");
        let payload = &migration_row.payload_min.entries;
        assert_eq!(
            payload
                .get(&PayloadKey::new("migration_stage").unwrap())
                .expect("migration_stage must exist")
                .as_str(),
            "M1"
        );
        assert_eq!(
            payload
                .get(&PayloadKey::new("read_contract").unwrap())
                .expect("read_contract must exist")
                .as_str(),
            "V1"
        );
    }

    #[test]
    fn at_vid_live_gate_14_contract_migration_stage_m2_emits_v2_read_mode() {
        let mut store = Ph1fStore::new_in_memory();
        let actor = UserId::new("tenant_mig:user_2").unwrap();
        let device = DeviceId::new("voice-mig-device-2").unwrap();
        let runtime = Ph1VoiceIdLiveRuntime::new(
            Ph1VoiceIdLiveConfig::mvp_v1_phone_first()
                .with_contract_migration_stage(VoiceIdContractMigrationStage::M2),
        );
        let req = sample_live_request();
        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_mig".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let enrolled = vec![EngineEnrolledSpeaker {
            speaker_id: selene_kernel_contracts::ph1_voice_id::SpeakerId::new("speaker_mig_2")
                .unwrap(),
            user_id: Some(
                selene_kernel_contracts::ph1_voice_id::UserId::new("user_mig_2").unwrap(),
            ),
            fingerprint: 7,
            profile_embedding: Some(simulation_profile_embedding_from_seed(7)),
        }];
        runtime
            .run_identity_assertion_with_signal_emission(
                &mut store,
                &req,
                context,
                enrolled,
                EngineVoiceIdObservation {
                    primary_fingerprint: Some(7),
                    secondary_fingerprint: None,
                    primary_embedding: Some(simulation_profile_embedding_from_seed(7)),
                    secondary_embedding: None,
                    spoof_risk: false,
                },
                VoiceIdentitySignalScope::v1(
                    MonotonicTimeNs(3),
                    CorrelationId(5514),
                    TurnId(1),
                    actor,
                    Some("tenant_mig".to_string()),
                    Some(device),
                ),
            )
            .expect("identity run with migration stage m2 must succeed");

        let migration_row = store
            .audit_events_by_correlation(CorrelationId(5514))
            .into_iter()
            .find(|row| {
                matches!(&row.engine, AuditEngine::Other(engine) if engine == PH1_VOICE_ID_ENGINE_ID)
                    && row.payload_min.entries.contains_key(
                        &PayloadKey::new("migration_stage")
                            .expect("migration_stage payload key is valid"),
                    )
            })
            .expect("migration audit row must exist");
        let payload = &migration_row.payload_min.entries;
        assert_eq!(
            payload
                .get(&PayloadKey::new("migration_stage").unwrap())
                .expect("migration_stage must exist")
                .as_str(),
            "M2"
        );
        assert_eq!(
            payload
                .get(&PayloadKey::new("read_contract").unwrap())
                .expect("read_contract must exist")
                .as_str(),
            "V2"
        );
    }

    #[test]
    fn at_vid_live_gate_15_matched_assertion_emits_explicit_decision_audit() {
        let mut store = Ph1fStore::new_in_memory();
        let runtime = Ph1VoiceIdLiveRuntime::default();
        let req = sample_live_request();
        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let enrolled = vec![EngineEnrolledSpeaker {
            speaker_id: selene_kernel_contracts::ph1_voice_id::SpeakerId::new("speaker_audit_ok")
                .unwrap(),
            user_id: Some(
                selene_kernel_contracts::ph1_voice_id::UserId::new("user_audit_ok").unwrap(),
            ),
            fingerprint: 7,
            profile_embedding: Some(simulation_profile_embedding_from_seed(7)),
        }];
        let out = runtime
            .run_identity_assertion_with_signal_emission(
                &mut store,
                &req,
                context,
                enrolled,
                EngineVoiceIdObservation {
                    primary_fingerprint: Some(7),
                    secondary_fingerprint: None,
                    primary_embedding: Some(simulation_profile_embedding_from_seed(7)),
                    secondary_embedding: None,
                    spoof_risk: false,
                },
                sample_signal_scope(5515, 1),
            )
            .expect("matched identity run with signal emission must succeed");

        assert!(matches!(out, Ph1VoiceIdResponse::SpeakerAssertionOk(_)));

        let rows = store.audit_events_by_correlation(CorrelationId(5515));
        let decision_row = rows
            .iter()
            .copied()
            .find(|row| {
                matches!(&row.engine, AuditEngine::Other(engine) if engine == PH1_VOICE_ID_ENGINE_ID)
                    && row.payload_min.entries.contains_key(
                        &PayloadKey::new("decision_log_family")
                            .expect("decision_log_family payload key is valid"),
                    )
            })
            .expect("decision audit row must exist");
        let decision_payload = &decision_row.payload_min.entries;
        let expected_score = voice_score_bp(&out).to_string();

        assert_eq!(
            decision_payload
                .get(&PayloadKey::new("decision_log_family").unwrap())
                .expect("decision_log_family must exist")
                .as_str(),
            "voice_id_live_assertion"
        );
        assert_eq!(
            decision_payload
                .get(&PayloadKey::new("decision_v1").unwrap())
                .expect("decision_v1 must exist")
                .as_str(),
            "OK"
        );
        assert_eq!(
            decision_payload
                .get(&PayloadKey::new("identity_tier_v2").unwrap())
                .expect("identity_tier_v2 must exist")
                .as_str(),
            identity_tier_label(out.identity_v2().identity_tier_v2)
        );
        assert_eq!(
            decision_payload
                .get(&PayloadKey::new("score_bp").unwrap())
                .expect("score_bp must exist")
                .as_str(),
            expected_score.as_str()
        );
        assert_eq!(
            decision_payload
                .get(&PayloadKey::new("platform").unwrap())
                .expect("platform must exist")
                .as_str(),
            "android"
        );
        assert_eq!(
            decision_payload
                .get(&PayloadKey::new("channel").unwrap())
                .expect("channel must exist")
                .as_str(),
            "explicit"
        );
        assert_eq!(
            decision_row.reason_code,
            engine_voice_reason_codes::VID_OK_MATCHED
        );
        assert_eq!(
            decision_row.idempotency_key.as_deref(),
            Some("voice_decision:5515:1")
        );
        assert!(
            rows.iter().any(|row| row.payload_min.entries.contains_key(
                &PayloadKey::new("migration_stage").expect("migration_stage payload key is valid"),
            )),
            "migration audit row must still exist"
        );
        assert!(
            rows.iter().any(|row| row.payload_min.entries.contains_key(
                &PayloadKey::new("metric_family").expect("metric_family payload key is valid"),
            )),
            "kpi audit row must still exist"
        );
        assert!(
            !decision_payload.contains_key(&PayloadKey::new("migration_stage").unwrap()),
            "decision audit row must be distinct from migration audit"
        );
        assert!(
            !decision_payload.contains_key(&PayloadKey::new("metric_family").unwrap()),
            "decision audit row must be distinct from kpi audit"
        );
    }

    #[test]
    fn at_vid_live_gate_16_unknown_assertion_emits_explicit_decision_audit_with_preserved_reason() {
        let mut store = Ph1fStore::new_in_memory();
        let runtime = Ph1VoiceIdLiveRuntime::default();
        let req = sample_live_request();
        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let enrolled = vec![EngineEnrolledSpeaker {
            speaker_id: selene_kernel_contracts::ph1_voice_id::SpeakerId::new(
                "speaker_audit_unknown",
            )
            .unwrap(),
            user_id: Some(
                selene_kernel_contracts::ph1_voice_id::UserId::new("user_audit_unknown").unwrap(),
            ),
            fingerprint: 7,
            profile_embedding: Some(simulation_profile_embedding_from_seed(7)),
        }];
        let out = runtime
            .run_identity_assertion_with_signal_emission(
                &mut store,
                &req,
                context,
                enrolled,
                EngineVoiceIdObservation {
                    primary_fingerprint: Some(7),
                    secondary_fingerprint: None,
                    primary_embedding: None,
                    secondary_embedding: None,
                    spoof_risk: false,
                },
                sample_signal_scope(5516, 1),
            )
            .expect("unknown identity run with signal emission must succeed");

        assert!(matches!(
            out,
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(_)
        ));

        let rows = store.audit_events_by_correlation(CorrelationId(5516));
        let decision_row = rows
            .iter()
            .copied()
            .find(|row| {
                matches!(&row.engine, AuditEngine::Other(engine) if engine == PH1_VOICE_ID_ENGINE_ID)
                    && row.payload_min.entries.contains_key(
                        &PayloadKey::new("decision_log_family")
                            .expect("decision_log_family payload key is valid"),
                    )
            })
            .expect("decision audit row must exist");
        let decision_payload = &decision_row.payload_min.entries;
        let expected_score = voice_score_bp(&out).to_string();

        assert_eq!(
            decision_payload
                .get(&PayloadKey::new("decision_log_family").unwrap())
                .expect("decision_log_family must exist")
                .as_str(),
            "voice_id_live_assertion"
        );
        assert_eq!(
            decision_payload
                .get(&PayloadKey::new("decision_v1").unwrap())
                .expect("decision_v1 must exist")
                .as_str(),
            "UNKNOWN"
        );
        assert_eq!(
            decision_payload
                .get(&PayloadKey::new("identity_tier_v2").unwrap())
                .expect("identity_tier_v2 must exist")
                .as_str(),
            identity_tier_label(out.identity_v2().identity_tier_v2)
        );
        assert_eq!(
            decision_payload
                .get(&PayloadKey::new("score_bp").unwrap())
                .expect("score_bp must exist")
                .as_str(),
            expected_score.as_str()
        );
        assert_eq!(
            decision_row.reason_code,
            engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE
        );
        assert!(
            !decision_payload.contains_key(&PayloadKey::new("margin_to_next_bp").unwrap()),
            "unknown low-confidence path should not invent a margin payload"
        );
        assert!(
            rows.iter().any(|row| row.payload_min.entries.contains_key(
                &PayloadKey::new("migration_stage").expect("migration_stage payload key is valid"),
            )),
            "migration audit row must still exist"
        );
        assert!(
            rows.iter().any(|row| row.payload_min.entries.contains_key(
                &PayloadKey::new("metric_family").expect("metric_family payload key is valid"),
            )),
            "kpi audit row must still exist"
        );
        assert!(
            !decision_payload.contains_key(&PayloadKey::new("migration_stage").unwrap()),
            "decision audit row must be distinct from migration audit"
        );
        assert!(
            !decision_payload.contains_key(&PayloadKey::new("metric_family").unwrap()),
            "decision audit row must be distinct from kpi audit"
        );
    }

    #[test]
    fn at_vid_live_gate_17_low_margin_ok_decision_audit_preserves_match_reason_and_margin() {
        let mut store = Ph1fStore::new_in_memory();
        let response = Ph1VoiceIdResponse::SpeakerAssertionOk(
            SpeakerAssertionOk::v1_with_metrics(
                selene_kernel_contracts::ph1_voice_id::SpeakerId::new("speaker_low_margin")
                    .unwrap(),
                Some(
                    selene_kernel_contracts::ph1_voice_id::UserId::new("user_low_margin").unwrap(),
                ),
                sample_decision_segments(),
                SpeakerLabel::speaker_a(),
                9_700,
                Some(250),
                Some(engine_voice_reason_codes::VID_OK_MATCHED),
                selene_kernel_contracts::ph1_voice_id::SpoofLivenessStatus::Live,
                vec![],
            )
            .expect("low-margin matched response fixture must be valid"),
        );
        let signal = map_voice_response_to_feedback_learn_signal(&response)
            .expect("low-margin matched response should still map to feedback side emission");
        assert_eq!(
            signal.reason_code,
            engine_voice_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN
        );
        assert_eq!(signal.decision, "OK_LOW_MARGIN");

        emit_voice_id_decision_audit(
            &mut store,
            VoiceIdentityRuntimeContext::from_tenant_app_platform(
                Some("tenant_audit".to_string()),
                Some(AppPlatform::Android),
                VoiceIdentityChannel::Explicit,
            ),
            &sample_signal_scope(5517, 1),
            &response,
        )
        .expect("decision audit emission must succeed for low-margin matched response");

        let decision_row = store
            .audit_events_by_correlation(CorrelationId(5517))
            .into_iter()
            .find(|row| {
                matches!(&row.engine, AuditEngine::Other(engine) if engine == PH1_VOICE_ID_ENGINE_ID)
                    && row.payload_min.entries.contains_key(
                        &PayloadKey::new("decision_log_family")
                            .expect("decision_log_family payload key is valid"),
                    )
            })
            .expect("decision audit row must exist");
        let decision_payload = &decision_row.payload_min.entries;

        assert_eq!(
            decision_payload
                .get(&PayloadKey::new("decision_v1").unwrap())
                .expect("decision_v1 must exist")
                .as_str(),
            "OK"
        );
        assert_eq!(
            decision_row.reason_code,
            engine_voice_reason_codes::VID_OK_MATCHED
        );
        assert_eq!(
            decision_payload
                .get(&PayloadKey::new("identity_tier_v2").unwrap())
                .expect("identity_tier_v2 must exist")
                .as_str(),
            identity_tier_label(response.identity_v2().identity_tier_v2)
        );
        assert_eq!(
            decision_payload
                .get(&PayloadKey::new("margin_to_next_bp").unwrap())
                .expect("margin_to_next_bp must exist")
                .as_str(),
            "250"
        );
    }

    #[test]
    fn at_vid_live_gate_18_scoped_unknown_feedback_audit_preserves_voice_context() {
        let mut store = Ph1fStore::new_in_memory();
        let runtime = Ph1VoiceIdLiveRuntime::default();
        let req = sample_live_request();
        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let signal_scope = sample_scoped_signal_scope(5518, 1);
        seed_scoped_feedback_identity_device(&mut store, &signal_scope);
        let enrolled = vec![EngineEnrolledSpeaker {
            speaker_id: selene_kernel_contracts::ph1_voice_id::SpeakerId::new(
                "speaker_audit_unknown_scoped",
            )
            .unwrap(),
            user_id: Some(
                selene_kernel_contracts::ph1_voice_id::UserId::new("user_audit_unknown_scoped")
                    .unwrap(),
            ),
            fingerprint: 7,
            profile_embedding: Some(simulation_profile_embedding_from_seed(7)),
        }];
        let out = runtime
            .run_identity_assertion_with_signal_emission(
                &mut store,
                &req,
                context,
                enrolled,
                EngineVoiceIdObservation {
                    primary_fingerprint: Some(7),
                    secondary_fingerprint: None,
                    primary_embedding: None,
                    secondary_embedding: None,
                    spoof_risk: false,
                },
                signal_scope.clone(),
            )
            .expect("scoped unknown identity run with signal emission must succeed");

        assert!(matches!(
            out,
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(_)
        ));

        let feedback_rows = store.ph1feedback_audit_rows(CorrelationId(5518));
        assert_eq!(feedback_rows.len(), 1);
        let feedback_row = feedback_rows[0];
        let feedback_payload = &feedback_row.payload_min.entries;
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("feedback_event_type").unwrap())
                .expect("feedback_event_type must exist")
                .as_str(),
            "VoiceIdFalseReject"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("signal_bucket must exist")
                .as_str(),
            "VoiceIdFalseReject"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("voice_decision").unwrap())
                .expect("voice_decision must exist")
                .as_str(),
            "UNKNOWN"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("voice_reason_code_hex").unwrap())
                .expect("voice_reason_code_hex must exist")
                .as_str(),
            format!(
                "0x{:X}",
                engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE.0
            )
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("evidence_ref").unwrap())
                .expect("evidence_ref must exist")
                .as_str(),
            format!(
                "voice_feedback_evidence:{}:{}:{}:VoiceIdFalseReject",
                signal_scope.actor_user_id.as_str(),
                signal_scope.correlation_id.0,
                signal_scope.turn_id.0
            )
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("provenance_ref").unwrap())
                .expect("provenance_ref must exist")
                .as_str(),
            "ph1.voice.id:feedback:VoiceIdFalseReject:v1"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("voice_score_bp").unwrap())
                .expect("voice_score_bp must exist")
                .as_str(),
            voice_score_bp(&out).to_string()
        );
        assert!(
            !feedback_payload.contains_key(&PayloadKey::new("voice_margin_to_next_bp").unwrap()),
            "unknown low-confidence path should not invent margin context"
        );
        assert_eq!(
            feedback_row.reason_code,
            engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE
        );

        let all_rows = store.audit_events_by_correlation(CorrelationId(5518));
        assert!(
            all_rows.iter().any(|row| {
                matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.LEARN")
                    && row
                        .payload_min
                        .entries
                        .contains_key(&PayloadKey::new("provenance_ref").unwrap())
            }),
            "scoped PH1.LEARN bundle audit row must still exist"
        );
        assert!(
            all_rows.iter().any(|row| {
                matches!(&row.engine, AuditEngine::Other(engine) if engine == PH1_VOICE_ID_ENGINE_ID)
                    && row.payload_min.entries.contains_key(
                        &PayloadKey::new("decision_log_family").unwrap()
                    )
            }),
            "H166 decision audit row must still exist"
        );
        assert!(
            !feedback_payload.contains_key(&PayloadKey::new("decision_log_family").unwrap()),
            "feedback audit row must remain distinct from the H166 decision row"
        );
    }

    #[test]
    fn at_vid_live_gate_19_scoped_low_margin_feedback_audit_preserves_margin_context() {
        let mut store = Ph1fStore::new_in_memory();
        let signal_scope = sample_scoped_signal_scope(5519, 1);
        seed_scoped_feedback_identity_device(&mut store, &signal_scope);
        let response = Ph1VoiceIdResponse::SpeakerAssertionOk(
            SpeakerAssertionOk::v1_with_metrics(
                selene_kernel_contracts::ph1_voice_id::SpeakerId::new("speaker_low_margin_scoped")
                    .unwrap(),
                Some(
                    selene_kernel_contracts::ph1_voice_id::UserId::new("user_low_margin_scoped")
                        .unwrap(),
                ),
                sample_decision_segments(),
                SpeakerLabel::speaker_a(),
                9_700,
                Some(250),
                Some(engine_voice_reason_codes::VID_OK_MATCHED),
                selene_kernel_contracts::ph1_voice_id::SpoofLivenessStatus::Live,
                vec![],
            )
            .expect("low-margin matched response fixture must be valid"),
        );
        let signal = map_voice_response_to_feedback_learn_signal(&response)
            .expect("low-margin matched response should map to scoped feedback emission");
        assert_eq!(
            signal.reason_code,
            engine_voice_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN
        );
        assert_eq!(signal.decision, "OK_LOW_MARGIN");

        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let live_context = voice_feedback_learn_audit_context(&context, &response);
        emit_voice_id_feedback_and_learn_signal(&mut store, &signal_scope, signal, live_context)
            .expect("scoped feedback emission must succeed for low-margin matched response");
        emit_voice_id_decision_audit(&mut store, context, &signal_scope, &response)
            .expect("decision audit emission must succeed for low-margin matched response");

        let feedback_rows = store.ph1feedback_audit_rows(CorrelationId(5519));
        assert_eq!(feedback_rows.len(), 1);
        let feedback_row = feedback_rows[0];
        let feedback_payload = &feedback_row.payload_min.entries;
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("feedback_event_type").unwrap())
                .expect("feedback_event_type must exist")
                .as_str(),
            "VoiceIdFalseAccept"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("signal_bucket must exist")
                .as_str(),
            "VoiceIdFalseAccept"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("voice_decision").unwrap())
                .expect("voice_decision must exist")
                .as_str(),
            "OK_LOW_MARGIN"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("voice_reason_code_hex").unwrap())
                .expect("voice_reason_code_hex must exist")
                .as_str(),
            format!(
                "0x{:X}",
                engine_voice_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN.0
            )
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("final_identity_decision").unwrap())
                .expect("final_identity_decision must exist")
                .as_str(),
            "OK"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("final_reason_code_hex").unwrap())
                .expect("final_reason_code_hex must exist")
                .as_str(),
            format!("0x{:X}", engine_voice_reason_codes::VID_OK_MATCHED.0)
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("voice_score_bp").unwrap())
                .expect("voice_score_bp must exist")
                .as_str(),
            "9700"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("voice_margin_to_next_bp").unwrap())
                .expect("voice_margin_to_next_bp must exist")
                .as_str(),
            "250"
        );
        assert_eq!(
            feedback_row.reason_code,
            engine_voice_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN
        );

        let all_rows = store.audit_events_by_correlation(CorrelationId(5519));
        let decision_row = all_rows
            .iter()
            .find(|row| {
                matches!(&row.engine, AuditEngine::Other(engine) if engine == PH1_VOICE_ID_ENGINE_ID)
                    && row.payload_min.entries.contains_key(
                        &PayloadKey::new("decision_log_family").unwrap()
                    )
            })
            .expect("decision audit row must exist");
        assert_eq!(
            decision_row.reason_code,
            engine_voice_reason_codes::VID_OK_MATCHED
        );
        assert_eq!(
            decision_row
                .payload_min
                .entries
                .get(&PayloadKey::new("decision_v1").unwrap())
                .expect("decision_v1 must exist")
                .as_str(),
            "OK"
        );
        assert!(
            all_rows.iter().any(|row| {
                matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.LEARN")
                    && row
                        .payload_min
                        .entries
                        .contains_key(&PayloadKey::new("provenance_ref").unwrap())
            }),
            "scoped PH1.LEARN bundle audit row must still exist"
        );
    }

    #[test]
    fn at_vid_live_gate_20_scoped_unknown_learn_audit_preserves_voice_context() {
        let mut store = Ph1fStore::new_in_memory();
        let runtime = Ph1VoiceIdLiveRuntime::default();
        let req = sample_live_request();
        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let signal_scope = sample_scoped_signal_scope(5520, 1);
        seed_scoped_feedback_identity_device(&mut store, &signal_scope);
        let enrolled = vec![EngineEnrolledSpeaker {
            speaker_id: selene_kernel_contracts::ph1_voice_id::SpeakerId::new(
                "speaker_audit_unknown_learn_scoped",
            )
            .unwrap(),
            user_id: Some(
                selene_kernel_contracts::ph1_voice_id::UserId::new(
                    "user_audit_unknown_learn_scoped",
                )
                .unwrap(),
            ),
            fingerprint: 7,
            profile_embedding: Some(simulation_profile_embedding_from_seed(7)),
        }];
        let out = runtime
            .run_identity_assertion_with_signal_emission(
                &mut store,
                &req,
                context,
                enrolled,
                EngineVoiceIdObservation {
                    primary_fingerprint: Some(7),
                    secondary_fingerprint: None,
                    primary_embedding: None,
                    secondary_embedding: None,
                    spoof_risk: false,
                },
                signal_scope.clone(),
            )
            .expect("scoped unknown identity run with signal emission must succeed");

        assert!(matches!(
            out,
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(_)
        ));

        let all_rows = store.audit_events_by_correlation(CorrelationId(5520));
        let learn_row = all_rows
            .iter()
            .find(|row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.LEARN"))
            .expect("scoped PH1.LEARN bundle audit row must exist");
        let learn_payload = &learn_row.payload_min.entries;
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("feedback_event_type").unwrap())
                .expect("feedback_event_type must exist")
                .as_str(),
            "VoiceIdFalseReject"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("learn_signal_type").unwrap())
                .expect("learn_signal_type must exist")
                .as_str(),
            "VoiceIdFalseReject"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("path_type").unwrap())
                .expect("path_type must exist")
                .as_str(),
            "PathA_Defect"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("signal_bucket must exist")
                .as_str(),
            "VoiceIdFalseReject"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_decision").unwrap())
                .expect("voice_decision must exist")
                .as_str(),
            "UNKNOWN"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_reason_code_hex").unwrap())
                .expect("voice_reason_code_hex must exist")
                .as_str(),
            format!(
                "0x{:X}",
                engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE.0
            )
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("final_identity_decision").unwrap())
                .expect("final_identity_decision must exist when margin is absent")
                .as_str(),
            "UNKNOWN"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("final_reason_code_hex").unwrap())
                .expect("final_reason_code_hex must exist when margin is absent")
                .as_str(),
            format!("0x{:X}", response_reason_code(&out).0)
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_score_bp").unwrap())
                .expect("voice_score_bp must exist")
                .as_str(),
            voice_score_bp(&out).to_string()
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("evidence_ref").unwrap())
                .expect("evidence_ref must exist")
                .as_str(),
            format!(
                "voice_feedback_evidence:{}:{}:{}:VoiceIdFalseReject",
                signal_scope.actor_user_id.as_str(),
                signal_scope.correlation_id.0,
                signal_scope.turn_id.0
            )
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("provenance_ref").unwrap())
                .expect("provenance_ref must exist")
                .as_str(),
            "ph1.voice.id:feedback:VoiceIdFalseReject:v1"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("voice_margin_to_next_bp").unwrap()),
            "unknown low-confidence path should not invent margin context"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("decision_log_family").unwrap()),
            "PH1.LEARN row must remain distinct from the H166 decision row"
        );
        assert_eq!(
            learn_row.reason_code,
            engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE
        );

        assert!(
            all_rows.iter().any(|row| {
                matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.FEEDBACK")
                    && row
                        .payload_min
                        .entries
                        .contains_key(&PayloadKey::new("signal_bucket").unwrap())
            }),
            "scoped PH1.FEEDBACK audit row must still exist"
        );
        assert!(
            all_rows.iter().any(|row| {
                matches!(&row.engine, AuditEngine::Other(engine) if engine == PH1_VOICE_ID_ENGINE_ID)
                    && row.payload_min.entries.contains_key(
                        &PayloadKey::new("decision_log_family").unwrap()
                    )
            }),
            "H166 decision audit row must still exist"
        );
    }

    #[test]
    fn at_vid_live_gate_21_scoped_low_margin_learn_audit_preserves_margin_context() {
        let mut store = Ph1fStore::new_in_memory();
        let signal_scope = sample_scoped_signal_scope(5521, 1);
        seed_scoped_feedback_identity_device(&mut store, &signal_scope);
        let response = Ph1VoiceIdResponse::SpeakerAssertionOk(
            SpeakerAssertionOk::v1_with_metrics(
                selene_kernel_contracts::ph1_voice_id::SpeakerId::new("speaker_low_margin_learn")
                    .unwrap(),
                Some(
                    selene_kernel_contracts::ph1_voice_id::UserId::new("user_low_margin_learn")
                        .unwrap(),
                ),
                sample_decision_segments(),
                SpeakerLabel::speaker_a(),
                9_700,
                Some(250),
                Some(engine_voice_reason_codes::VID_OK_MATCHED),
                selene_kernel_contracts::ph1_voice_id::SpoofLivenessStatus::Live,
                vec![],
            )
            .expect("low-margin matched response fixture must be valid"),
        );
        let signal = map_voice_response_to_feedback_learn_signal(&response)
            .expect("low-margin matched response should map to scoped feedback emission");
        assert_eq!(
            signal.reason_code,
            engine_voice_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN
        );
        assert_eq!(signal.decision, "OK_LOW_MARGIN");

        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let live_context = voice_feedback_learn_audit_context(&context, &response);
        emit_voice_id_feedback_and_learn_signal(&mut store, &signal_scope, signal, live_context)
            .expect("scoped feedback emission must succeed for low-margin matched response");
        emit_voice_id_decision_audit(&mut store, context, &signal_scope, &response)
            .expect("decision audit emission must succeed for low-margin matched response");

        let all_rows = store.audit_events_by_correlation(CorrelationId(5521));
        let learn_row = all_rows
            .iter()
            .find(|row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.LEARN"))
            .expect("scoped PH1.LEARN bundle audit row must exist");
        let learn_payload = &learn_row.payload_min.entries;
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("feedback_event_type").unwrap())
                .expect("feedback_event_type must exist")
                .as_str(),
            "VoiceIdFalseAccept"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("learn_signal_type").unwrap())
                .expect("learn_signal_type must exist")
                .as_str(),
            "VoiceIdFalseAccept"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("path_type").unwrap())
                .expect("path_type must exist")
                .as_str(),
            "PathA_Defect"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("signal_bucket must exist")
                .as_str(),
            "VoiceIdFalseAccept"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_decision").unwrap())
                .expect("voice_decision must exist")
                .as_str(),
            "OK_LOW_MARGIN"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_reason_code_hex").unwrap())
                .expect("voice_reason_code_hex must exist")
                .as_str(),
            format!(
                "0x{:X}",
                engine_voice_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN.0
            )
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_score_bp").unwrap())
                .expect("voice_score_bp must exist")
                .as_str(),
            "9700"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_margin_to_next_bp").unwrap())
                .expect("voice_margin_to_next_bp must exist")
                .as_str(),
            "250"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("final_identity_decision").unwrap()),
            "low-margin scoped learn row must remain blocked from final_identity_decision"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("final_reason_code_hex").unwrap()),
            "low-margin scoped learn row must remain blocked from final_reason_code_hex"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("decision_log_family").unwrap()),
            "PH1.LEARN row must remain distinct from the H166 decision row"
        );
        assert_eq!(
            learn_row.reason_code,
            engine_voice_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN
        );

        let decision_row = all_rows
            .iter()
            .find(|row| {
                matches!(&row.engine, AuditEngine::Other(engine) if engine == PH1_VOICE_ID_ENGINE_ID)
                    && row.payload_min.entries.contains_key(
                        &PayloadKey::new("decision_log_family").unwrap()
                    )
            })
            .expect("decision audit row must exist");
        assert_eq!(
            decision_row.reason_code,
            engine_voice_reason_codes::VID_OK_MATCHED
        );
        assert_eq!(
            decision_row
                .payload_min
                .entries
                .get(&PayloadKey::new("decision_v1").unwrap())
                .expect("decision_v1 must exist")
                .as_str(),
            "OK"
        );
        assert!(
            all_rows.iter().any(|row| {
                matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.FEEDBACK")
                    && row
                        .payload_min
                        .entries
                        .contains_key(&PayloadKey::new("signal_bucket").unwrap())
            }),
            "scoped PH1.FEEDBACK audit row must still exist"
        );
    }

    #[test]
    fn at_vid_live_gate_22_unscoped_unknown_learn_audit_preserves_voice_context() {
        let mut store = Ph1fStore::new_in_memory();
        let runtime = Ph1VoiceIdLiveRuntime::default();
        let req = sample_live_request();
        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let signal_scope = sample_signal_scope(5522, 1);
        let enrolled = vec![EngineEnrolledSpeaker {
            speaker_id: selene_kernel_contracts::ph1_voice_id::SpeakerId::new(
                "speaker_audit_unknown_learn_unscoped",
            )
            .unwrap(),
            user_id: Some(
                selene_kernel_contracts::ph1_voice_id::UserId::new(
                    "user_audit_unknown_learn_unscoped",
                )
                .unwrap(),
            ),
            fingerprint: 7,
            profile_embedding: Some(simulation_profile_embedding_from_seed(7)),
        }];
        let out = runtime
            .run_identity_assertion_with_signal_emission(
                &mut store,
                &req,
                context,
                enrolled,
                EngineVoiceIdObservation {
                    primary_fingerprint: Some(7),
                    secondary_fingerprint: None,
                    primary_embedding: None,
                    secondary_embedding: None,
                    spoof_risk: false,
                },
                signal_scope.clone(),
            )
            .expect("unscoped unknown identity run with signal emission must succeed");

        assert!(matches!(
            out,
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(_)
        ));

        let all_rows = store.audit_events_by_correlation(CorrelationId(5522));
        let learn_row = all_rows
            .iter()
            .find(|row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.LEARN"))
            .expect("fallback PH1.LEARN audit row must exist");
        let learn_payload = &learn_row.payload_min.entries;
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("feedback_event_type").unwrap())
                .expect("feedback_event_type must exist")
                .as_str(),
            "VoiceIdFalseReject"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("learn_signal_type").unwrap())
                .expect("learn_signal_type must exist")
                .as_str(),
            "VoiceIdFalseReject"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("path_type").unwrap())
                .expect("path_type must exist")
                .as_str(),
            "PathA_Defect"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("signal_bucket must exist")
                .as_str(),
            "VoiceIdFalseReject"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_decision").unwrap())
                .expect("voice_decision must exist")
                .as_str(),
            "UNKNOWN"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_reason_code_hex").unwrap())
                .expect("voice_reason_code_hex must exist")
                .as_str(),
            format!(
                "0x{:X}",
                engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE.0
            )
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_score_bp").unwrap())
                .expect("voice_score_bp must exist")
                .as_str(),
            voice_score_bp(&out).to_string()
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("evidence_ref").unwrap())
                .expect("evidence_ref must exist")
                .as_str(),
            format!(
                "voice_feedback_evidence:{}:{}:{}:VoiceIdFalseReject",
                signal_scope.actor_user_id.as_str(),
                signal_scope.correlation_id.0,
                signal_scope.turn_id.0
            )
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("provenance_ref").unwrap())
                .expect("provenance_ref must exist")
                .as_str(),
            "ph1.voice.id:feedback:VoiceIdFalseReject:v1"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("voice_margin_to_next_bp").unwrap()),
            "unknown fallback path should not invent margin context"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("bundle_id").unwrap()),
            "fallback PH1.LEARN row must not invent bundle_id"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("ingest_latency_ms").unwrap()),
            "fallback PH1.LEARN row must not invent ingest latency"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("decision_log_family").unwrap()),
            "fallback PH1.LEARN row must remain distinct from the H166 decision row"
        );
        assert_eq!(
            learn_row.reason_code,
            engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE
        );
        let expected_learn_idempotency = format!(
            "voice_learn_unscoped:{}:{}:{}:{}:{}",
            signal_scope.actor_user_id.as_str(),
            signal_scope.correlation_id.0,
            signal_scope.turn_id.0,
            "VoiceIdFalseReject",
            engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE.0
        );
        assert_eq!(
            learn_row.idempotency_key.as_deref(),
            Some(expected_learn_idempotency.as_str())
        );

        let feedback_row = all_rows
            .iter()
            .find(
                |row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.FEEDBACK"),
            )
            .expect("fallback PH1.FEEDBACK audit row must still exist");
        let expected_feedback_idempotency = format!(
            "voice_feedback_unscoped:{}:{}:{}:{}",
            signal_scope.actor_user_id.as_str(),
            signal_scope.correlation_id.0,
            signal_scope.turn_id.0,
            engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE.0
        );
        assert_eq!(
            feedback_row.idempotency_key.as_deref(),
            Some(expected_feedback_idempotency.as_str())
        );
        assert_ne!(
            learn_row.idempotency_key, feedback_row.idempotency_key,
            "fallback PH1.LEARN idempotency must remain distinct from fallback PH1.FEEDBACK"
        );

        let decision_row = all_rows
            .iter()
            .find(|row| {
                matches!(&row.engine, AuditEngine::Other(engine) if engine == PH1_VOICE_ID_ENGINE_ID)
                    && row.payload_min.entries.contains_key(
                        &PayloadKey::new("decision_log_family").unwrap()
                    )
            })
            .expect("H166 decision audit row must still exist");
        assert_eq!(
            decision_row
                .payload_min
                .entries
                .get(&PayloadKey::new("decision_v1").unwrap())
                .expect("decision_v1 must exist")
                .as_str(),
            "UNKNOWN"
        );
    }

    #[test]
    fn at_vid_live_gate_23_unscoped_low_margin_learn_audit_preserves_margin_context() {
        let mut store = Ph1fStore::new_in_memory();
        let signal_scope = sample_signal_scope(5523, 1);
        let response = Ph1VoiceIdResponse::SpeakerAssertionOk(
            SpeakerAssertionOk::v1_with_metrics(
                selene_kernel_contracts::ph1_voice_id::SpeakerId::new(
                    "speaker_low_margin_learn_unscoped",
                )
                .unwrap(),
                Some(
                    selene_kernel_contracts::ph1_voice_id::UserId::new(
                        "user_low_margin_learn_unscoped",
                    )
                    .unwrap(),
                ),
                sample_decision_segments(),
                SpeakerLabel::speaker_a(),
                9_700,
                Some(250),
                Some(engine_voice_reason_codes::VID_OK_MATCHED),
                selene_kernel_contracts::ph1_voice_id::SpoofLivenessStatus::Live,
                vec![],
            )
            .expect("low-margin matched response fixture must be valid"),
        );
        let signal = map_voice_response_to_feedback_learn_signal(&response)
            .expect("low-margin matched response should map to fallback learn emission");
        assert_eq!(
            signal.reason_code,
            engine_voice_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN
        );
        assert_eq!(signal.decision, "OK_LOW_MARGIN");

        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let live_context = voice_feedback_learn_audit_context(&context, &response);
        emit_voice_id_feedback_and_learn_signal(&mut store, &signal_scope, signal, live_context)
            .expect("fallback learn emission must succeed for low-margin matched response");
        emit_voice_id_decision_audit(&mut store, context, &signal_scope, &response)
            .expect("decision audit emission must succeed for low-margin matched response");

        let all_rows = store.audit_events_by_correlation(CorrelationId(5523));
        let learn_row = all_rows
            .iter()
            .find(|row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.LEARN"))
            .expect("fallback PH1.LEARN audit row must exist");
        let learn_payload = &learn_row.payload_min.entries;
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("feedback_event_type").unwrap())
                .expect("feedback_event_type must exist")
                .as_str(),
            "VoiceIdFalseAccept"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("learn_signal_type").unwrap())
                .expect("learn_signal_type must exist")
                .as_str(),
            "VoiceIdFalseAccept"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("path_type").unwrap())
                .expect("path_type must exist")
                .as_str(),
            "PathA_Defect"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("signal_bucket must exist")
                .as_str(),
            "VoiceIdFalseAccept"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_decision").unwrap())
                .expect("voice_decision must exist")
                .as_str(),
            "OK_LOW_MARGIN"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_reason_code_hex").unwrap())
                .expect("voice_reason_code_hex must exist")
                .as_str(),
            format!(
                "0x{:X}",
                engine_voice_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN.0
            )
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_score_bp").unwrap())
                .expect("voice_score_bp must exist")
                .as_str(),
            "9700"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_margin_to_next_bp").unwrap())
                .expect("voice_margin_to_next_bp must exist")
                .as_str(),
            "250"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("bundle_id").unwrap()),
            "fallback PH1.LEARN row must not invent bundle_id"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("ingest_latency_ms").unwrap()),
            "fallback PH1.LEARN row must not invent ingest latency"
        );
        assert_eq!(
            learn_row.reason_code,
            engine_voice_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN
        );
        let expected_learn_idempotency = format!(
            "voice_learn_unscoped:{}:{}:{}:{}:{}",
            signal_scope.actor_user_id.as_str(),
            signal_scope.correlation_id.0,
            signal_scope.turn_id.0,
            "VoiceIdFalseAccept",
            engine_voice_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN.0
        );
        assert_eq!(
            learn_row.idempotency_key.as_deref(),
            Some(expected_learn_idempotency.as_str())
        );

        let feedback_row = all_rows
            .iter()
            .find(
                |row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.FEEDBACK"),
            )
            .expect("fallback PH1.FEEDBACK audit row must still exist");
        let expected_feedback_idempotency = format!(
            "voice_feedback_unscoped:{}:{}:{}:{}",
            signal_scope.actor_user_id.as_str(),
            signal_scope.correlation_id.0,
            signal_scope.turn_id.0,
            engine_voice_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN.0
        );
        assert_eq!(
            feedback_row.idempotency_key.as_deref(),
            Some(expected_feedback_idempotency.as_str())
        );
        assert_ne!(
            learn_row.idempotency_key, feedback_row.idempotency_key,
            "fallback PH1.LEARN idempotency must remain distinct from fallback PH1.FEEDBACK"
        );

        let decision_row = all_rows
            .iter()
            .find(|row| {
                matches!(&row.engine, AuditEngine::Other(engine) if engine == PH1_VOICE_ID_ENGINE_ID)
                    && row.payload_min.entries.contains_key(
                        &PayloadKey::new("decision_log_family").unwrap()
                    )
            })
            .expect("H166 decision audit row must still exist");
        assert_eq!(
            decision_row.reason_code,
            engine_voice_reason_codes::VID_OK_MATCHED
        );
        assert_eq!(
            decision_row
                .payload_min
                .entries
                .get(&PayloadKey::new("decision_v1").unwrap())
                .expect("decision_v1 must exist")
                .as_str(),
            "OK"
        );
    }

    #[test]
    fn at_vid_live_gate_24_unscoped_low_score_learn_audit_preserves_reauth_friction_context() {
        let mut store = Ph1fStore::new_in_memory();
        let signal_scope = sample_signal_scope(5524, 1);
        let response = Ph1VoiceIdResponse::SpeakerAssertionOk(
            SpeakerAssertionOk::v1_with_metrics(
                selene_kernel_contracts::ph1_voice_id::SpeakerId::new(
                    "speaker_low_score_learn_unscoped",
                )
                .unwrap(),
                Some(
                    selene_kernel_contracts::ph1_voice_id::UserId::new(
                        "user_low_score_learn_unscoped",
                    )
                    .unwrap(),
                ),
                sample_decision_segments(),
                SpeakerLabel::speaker_a(),
                9_400,
                Some(350),
                Some(engine_voice_reason_codes::VID_OK_MATCHED),
                selene_kernel_contracts::ph1_voice_id::SpoofLivenessStatus::Live,
                vec![],
            )
            .expect("low-score matched response fixture must be valid"),
        );
        let signal = map_voice_response_to_feedback_learn_signal(&response)
            .expect("low-score matched response should map to fallback learn emission");
        assert_eq!(
            signal.reason_code,
            engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE
        );
        assert_eq!(signal.decision, "OK_LOW_SCORE");

        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let live_context = voice_feedback_learn_audit_context(&context, &response);
        emit_voice_id_feedback_and_learn_signal(&mut store, &signal_scope, signal, live_context)
            .expect("fallback learn emission must succeed for low-score matched response");
        emit_voice_id_decision_audit(&mut store, context, &signal_scope, &response)
            .expect("decision audit emission must succeed for low-score matched response");

        let all_rows = store.audit_events_by_correlation(CorrelationId(5524));
        let learn_row = all_rows
            .iter()
            .find(|row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.LEARN"))
            .expect("fallback PH1.LEARN audit row must exist");
        let learn_payload = &learn_row.payload_min.entries;
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("feedback_event_type").unwrap())
                .expect("feedback_event_type must exist")
                .as_str(),
            "VoiceIdReauthFriction"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("learn_signal_type").unwrap())
                .expect("learn_signal_type must exist")
                .as_str(),
            "VoiceIdReauthFriction"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("path_type").unwrap())
                .expect("path_type must exist")
                .as_str(),
            "PathB_Improvement"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("signal_bucket must exist")
                .as_str(),
            "VoiceIdReauthFriction"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_decision").unwrap())
                .expect("voice_decision must exist")
                .as_str(),
            "OK_LOW_SCORE"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_reason_code_hex").unwrap())
                .expect("voice_reason_code_hex must exist")
                .as_str(),
            format!(
                "0x{:X}",
                engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE.0
            )
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_score_bp").unwrap())
                .expect("voice_score_bp must exist")
                .as_str(),
            "9400"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_margin_to_next_bp").unwrap())
                .expect("voice_margin_to_next_bp must exist")
                .as_str(),
            "350"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("bundle_id").unwrap()),
            "fallback PH1.LEARN row must not invent bundle_id"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("ingest_latency_ms").unwrap()),
            "fallback PH1.LEARN row must not invent ingest latency"
        );
        assert_eq!(
            learn_row.reason_code,
            engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE
        );
        let expected_learn_idempotency = format!(
            "voice_learn_unscoped:{}:{}:{}:{}:{}",
            signal_scope.actor_user_id.as_str(),
            signal_scope.correlation_id.0,
            signal_scope.turn_id.0,
            "VoiceIdReauthFriction",
            engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE.0
        );
        assert_eq!(
            learn_row.idempotency_key.as_deref(),
            Some(expected_learn_idempotency.as_str())
        );

        let feedback_row = all_rows
            .iter()
            .find(
                |row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.FEEDBACK"),
            )
            .expect("fallback PH1.FEEDBACK audit row must still exist");
        let expected_feedback_idempotency = format!(
            "voice_feedback_unscoped:{}:{}:{}:{}",
            signal_scope.actor_user_id.as_str(),
            signal_scope.correlation_id.0,
            signal_scope.turn_id.0,
            engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE.0
        );
        assert_eq!(
            feedback_row.idempotency_key.as_deref(),
            Some(expected_feedback_idempotency.as_str())
        );
        assert_ne!(
            learn_row.idempotency_key, feedback_row.idempotency_key,
            "fallback PH1.LEARN idempotency must remain distinct from fallback PH1.FEEDBACK"
        );

        let decision_row = all_rows
            .iter()
            .find(|row| {
                matches!(&row.engine, AuditEngine::Other(engine) if engine == PH1_VOICE_ID_ENGINE_ID)
                    && row.payload_min.entries.contains_key(
                        &PayloadKey::new("decision_log_family").unwrap()
                    )
            })
            .expect("H166 decision audit row must still exist");
        assert_eq!(
            decision_row.reason_code,
            engine_voice_reason_codes::VID_OK_MATCHED
        );
        assert_eq!(
            decision_row
                .payload_min
                .entries
                .get(&PayloadKey::new("decision_v1").unwrap())
                .expect("decision_v1 must exist")
                .as_str(),
            "OK"
        );
    }

    #[test]
    fn at_vid_live_gate_25_unscoped_unknown_feedback_audit_surfaces_bucket_and_path_type() {
        let mut store = Ph1fStore::new_in_memory();
        let runtime = Ph1VoiceIdLiveRuntime::default();
        let req = sample_live_request();
        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let signal_scope = sample_signal_scope(5525, 1);
        let enrolled = vec![EngineEnrolledSpeaker {
            speaker_id: selene_kernel_contracts::ph1_voice_id::SpeakerId::new(
                "speaker_audit_unknown_feedback_unscoped",
            )
            .unwrap(),
            user_id: Some(
                selene_kernel_contracts::ph1_voice_id::UserId::new(
                    "user_audit_unknown_feedback_unscoped",
                )
                .unwrap(),
            ),
            fingerprint: 7,
            profile_embedding: Some(simulation_profile_embedding_from_seed(7)),
        }];
        let out = runtime
            .run_identity_assertion_with_signal_emission(
                &mut store,
                &req,
                context,
                enrolled,
                EngineVoiceIdObservation {
                    primary_fingerprint: Some(7),
                    secondary_fingerprint: None,
                    primary_embedding: None,
                    secondary_embedding: None,
                    spoof_risk: false,
                },
                signal_scope.clone(),
            )
            .expect("unscoped unknown identity run with signal emission must succeed");

        assert!(matches!(
            out,
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(_)
        ));

        let all_rows = store.audit_events_by_correlation(CorrelationId(5525));
        let feedback_row = all_rows
            .iter()
            .find(
                |row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.FEEDBACK"),
            )
            .expect("fallback PH1.FEEDBACK audit row must exist");
        let feedback_payload = &feedback_row.payload_min.entries;
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("feedback_event_type").unwrap())
                .expect("feedback_event_type must exist")
                .as_str(),
            "VoiceIdFalseReject"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("learn_signal_type").unwrap())
                .expect("learn_signal_type must exist")
                .as_str(),
            "VoiceIdFalseReject"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("signal_bucket must exist")
                .as_str(),
            "VoiceIdFalseReject"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("path_type").unwrap())
                .expect("path_type must exist")
                .as_str(),
            "PathA_Defect"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("voice_decision").unwrap())
                .expect("voice_decision must exist")
                .as_str(),
            "UNKNOWN"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("voice_reason_code_hex").unwrap())
                .expect("voice_reason_code_hex must exist")
                .as_str(),
            format!(
                "0x{:X}",
                engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE.0
            )
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("voice_score_bp").unwrap())
                .expect("voice_score_bp must exist")
                .as_str(),
            voice_score_bp(&out).to_string()
        );
        assert!(
            !feedback_payload.contains_key(&PayloadKey::new("voice_margin_to_next_bp").unwrap()),
            "unknown fallback feedback path should not invent margin context"
        );
        assert!(
            !feedback_payload.contains_key(&PayloadKey::new("decision_log_family").unwrap()),
            "fallback PH1.FEEDBACK row must remain distinct from the H166 decision row"
        );
        assert_eq!(
            feedback_row.reason_code,
            engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE
        );
        let expected_feedback_idempotency = format!(
            "voice_feedback_unscoped:{}:{}:{}:{}",
            signal_scope.actor_user_id.as_str(),
            signal_scope.correlation_id.0,
            signal_scope.turn_id.0,
            engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE.0
        );
        assert_eq!(
            feedback_row.idempotency_key.as_deref(),
            Some(expected_feedback_idempotency.as_str())
        );

        let learn_row = all_rows
            .iter()
            .find(|row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.LEARN"))
            .expect("fallback PH1.LEARN audit row must still exist");
        let learn_payload = &learn_row.payload_min.entries;
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("learn_signal_type").unwrap())
                .expect("learn_signal_type must exist")
                .as_str(),
            "VoiceIdFalseReject"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("signal_bucket must exist")
                .as_str(),
            "VoiceIdFalseReject"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("path_type").unwrap())
                .expect("path_type must exist")
                .as_str(),
            "PathA_Defect"
        );

        let decision_row = all_rows
            .iter()
            .find(|row| {
                matches!(&row.engine, AuditEngine::Other(engine) if engine == PH1_VOICE_ID_ENGINE_ID)
                    && row.payload_min.entries.contains_key(
                        &PayloadKey::new("decision_log_family").unwrap()
                    )
            })
            .expect("H166 decision audit row must still exist");
        assert_eq!(
            decision_row
                .payload_min
                .entries
                .get(&PayloadKey::new("decision_v1").unwrap())
                .expect("decision_v1 must exist")
                .as_str(),
            "UNKNOWN"
        );
    }

    #[test]
    fn at_vid_live_gate_26_unscoped_low_margin_feedback_audit_surfaces_bucket_and_path_type() {
        let mut store = Ph1fStore::new_in_memory();
        let signal_scope = sample_signal_scope(5526, 1);
        let response = Ph1VoiceIdResponse::SpeakerAssertionOk(
            SpeakerAssertionOk::v1_with_metrics(
                selene_kernel_contracts::ph1_voice_id::SpeakerId::new(
                    "speaker_low_margin_feedback_unscoped",
                )
                .unwrap(),
                Some(
                    selene_kernel_contracts::ph1_voice_id::UserId::new(
                        "user_low_margin_feedback_unscoped",
                    )
                    .unwrap(),
                ),
                sample_decision_segments(),
                SpeakerLabel::speaker_a(),
                9_700,
                Some(250),
                Some(engine_voice_reason_codes::VID_OK_MATCHED),
                selene_kernel_contracts::ph1_voice_id::SpoofLivenessStatus::Live,
                vec![],
            )
            .expect("low-margin matched response fixture must be valid"),
        );
        let signal = map_voice_response_to_feedback_learn_signal(&response)
            .expect("low-margin matched response should map to fallback feedback emission");
        assert_eq!(
            signal.reason_code,
            engine_voice_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN
        );
        assert_eq!(signal.decision, "OK_LOW_MARGIN");

        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let live_context = voice_feedback_learn_audit_context(&context, &response);
        emit_voice_id_feedback_and_learn_signal(&mut store, &signal_scope, signal, live_context)
            .expect("fallback feedback emission must succeed for low-margin matched response");
        emit_voice_id_decision_audit(&mut store, context, &signal_scope, &response)
            .expect("decision audit emission must succeed for low-margin matched response");

        let all_rows = store.audit_events_by_correlation(CorrelationId(5526));
        let feedback_row = all_rows
            .iter()
            .find(
                |row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.FEEDBACK"),
            )
            .expect("fallback PH1.FEEDBACK audit row must exist");
        let feedback_payload = &feedback_row.payload_min.entries;
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("feedback_event_type").unwrap())
                .expect("feedback_event_type must exist")
                .as_str(),
            "VoiceIdFalseAccept"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("learn_signal_type").unwrap())
                .expect("learn_signal_type must exist")
                .as_str(),
            "VoiceIdFalseAccept"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("signal_bucket must exist")
                .as_str(),
            "VoiceIdFalseAccept"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("path_type").unwrap())
                .expect("path_type must exist")
                .as_str(),
            "PathA_Defect"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("voice_decision").unwrap())
                .expect("voice_decision must exist")
                .as_str(),
            "OK_LOW_MARGIN"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("voice_reason_code_hex").unwrap())
                .expect("voice_reason_code_hex must exist")
                .as_str(),
            format!(
                "0x{:X}",
                engine_voice_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN.0
            )
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("voice_score_bp").unwrap())
                .expect("voice_score_bp must exist")
                .as_str(),
            "9700"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("voice_margin_to_next_bp").unwrap())
                .expect("voice_margin_to_next_bp must exist")
                .as_str(),
            "250"
        );
        assert_eq!(
            feedback_row.reason_code,
            engine_voice_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN
        );
        let expected_feedback_idempotency = format!(
            "voice_feedback_unscoped:{}:{}:{}:{}",
            signal_scope.actor_user_id.as_str(),
            signal_scope.correlation_id.0,
            signal_scope.turn_id.0,
            engine_voice_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN.0
        );
        assert_eq!(
            feedback_row.idempotency_key.as_deref(),
            Some(expected_feedback_idempotency.as_str())
        );

        let learn_row = all_rows
            .iter()
            .find(|row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.LEARN"))
            .expect("fallback PH1.LEARN audit row must still exist");
        let learn_payload = &learn_row.payload_min.entries;
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("learn_signal_type").unwrap())
                .expect("learn_signal_type must exist")
                .as_str(),
            "VoiceIdFalseAccept"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("signal_bucket must exist")
                .as_str(),
            "VoiceIdFalseAccept"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("path_type").unwrap())
                .expect("path_type must exist")
                .as_str(),
            "PathA_Defect"
        );

        let decision_row = all_rows
            .iter()
            .find(|row| {
                matches!(&row.engine, AuditEngine::Other(engine) if engine == PH1_VOICE_ID_ENGINE_ID)
                    && row.payload_min.entries.contains_key(
                        &PayloadKey::new("decision_log_family").unwrap()
                    )
            })
            .expect("H166 decision audit row must still exist");
        assert_eq!(
            decision_row.reason_code,
            engine_voice_reason_codes::VID_OK_MATCHED
        );
        assert_eq!(
            decision_row
                .payload_min
                .entries
                .get(&PayloadKey::new("decision_v1").unwrap())
                .expect("decision_v1 must exist")
                .as_str(),
            "OK"
        );
    }

    #[test]
    fn at_vid_live_gate_27_unscoped_low_score_feedback_audit_surfaces_reauth_bucket_and_path_type()
    {
        let mut store = Ph1fStore::new_in_memory();
        let signal_scope = sample_signal_scope(5527, 1);
        let response = Ph1VoiceIdResponse::SpeakerAssertionOk(
            SpeakerAssertionOk::v1_with_metrics(
                selene_kernel_contracts::ph1_voice_id::SpeakerId::new(
                    "speaker_low_score_feedback_unscoped",
                )
                .unwrap(),
                Some(
                    selene_kernel_contracts::ph1_voice_id::UserId::new(
                        "user_low_score_feedback_unscoped",
                    )
                    .unwrap(),
                ),
                sample_decision_segments(),
                SpeakerLabel::speaker_a(),
                9_400,
                Some(350),
                Some(engine_voice_reason_codes::VID_OK_MATCHED),
                selene_kernel_contracts::ph1_voice_id::SpoofLivenessStatus::Live,
                vec![],
            )
            .expect("low-score matched response fixture must be valid"),
        );
        let signal = map_voice_response_to_feedback_learn_signal(&response)
            .expect("low-score matched response should map to fallback feedback emission");
        assert_eq!(
            signal.reason_code,
            engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE
        );
        assert_eq!(signal.decision, "OK_LOW_SCORE");

        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let live_context = voice_feedback_learn_audit_context(&context, &response);
        emit_voice_id_feedback_and_learn_signal(&mut store, &signal_scope, signal, live_context)
            .expect("fallback feedback emission must succeed for low-score matched response");
        emit_voice_id_decision_audit(&mut store, context, &signal_scope, &response)
            .expect("decision audit emission must succeed for low-score matched response");

        let all_rows = store.audit_events_by_correlation(CorrelationId(5527));
        let feedback_row = all_rows
            .iter()
            .find(
                |row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.FEEDBACK"),
            )
            .expect("fallback PH1.FEEDBACK audit row must exist");
        let feedback_payload = &feedback_row.payload_min.entries;
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("feedback_event_type").unwrap())
                .expect("feedback_event_type must exist")
                .as_str(),
            "VoiceIdReauthFriction"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("learn_signal_type").unwrap())
                .expect("learn_signal_type must exist")
                .as_str(),
            "VoiceIdReauthFriction"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("signal_bucket must exist")
                .as_str(),
            "VoiceIdReauthFriction"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("path_type").unwrap())
                .expect("path_type must exist")
                .as_str(),
            "PathB_Improvement"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("voice_decision").unwrap())
                .expect("voice_decision must exist")
                .as_str(),
            "OK_LOW_SCORE"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("voice_reason_code_hex").unwrap())
                .expect("voice_reason_code_hex must exist")
                .as_str(),
            format!(
                "0x{:X}",
                engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE.0
            )
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("voice_score_bp").unwrap())
                .expect("voice_score_bp must exist")
                .as_str(),
            "9400"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("voice_margin_to_next_bp").unwrap())
                .expect("voice_margin_to_next_bp must exist")
                .as_str(),
            "350"
        );
        assert_eq!(
            feedback_row.reason_code,
            engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE
        );
        let expected_feedback_idempotency = format!(
            "voice_feedback_unscoped:{}:{}:{}:{}",
            signal_scope.actor_user_id.as_str(),
            signal_scope.correlation_id.0,
            signal_scope.turn_id.0,
            engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE.0
        );
        assert_eq!(
            feedback_row.idempotency_key.as_deref(),
            Some(expected_feedback_idempotency.as_str())
        );

        let learn_row = all_rows
            .iter()
            .find(|row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.LEARN"))
            .expect("fallback PH1.LEARN audit row must still exist");
        let learn_payload = &learn_row.payload_min.entries;
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("learn_signal_type").unwrap())
                .expect("learn_signal_type must exist")
                .as_str(),
            "VoiceIdReauthFriction"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("signal_bucket must exist")
                .as_str(),
            "VoiceIdReauthFriction"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("path_type").unwrap())
                .expect("path_type must exist")
                .as_str(),
            "PathB_Improvement"
        );

        let decision_row = all_rows
            .iter()
            .find(|row| {
                matches!(&row.engine, AuditEngine::Other(engine) if engine == PH1_VOICE_ID_ENGINE_ID)
                    && row.payload_min.entries.contains_key(
                        &PayloadKey::new("decision_log_family").unwrap()
                    )
            })
            .expect("H166 decision audit row must still exist");
        assert_eq!(
            decision_row.reason_code,
            engine_voice_reason_codes::VID_OK_MATCHED
        );
        assert_eq!(
            decision_row
                .payload_min
                .entries
                .get(&PayloadKey::new("decision_v1").unwrap())
                .expect("decision_v1 must exist")
                .as_str(),
            "OK"
        );
    }

    #[test]
    fn at_vid_live_gate_28_unscoped_unknown_learn_audit_surfaces_defect_path_type() {
        let mut store = Ph1fStore::new_in_memory();
        let runtime = Ph1VoiceIdLiveRuntime::default();
        let req = sample_live_request();
        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let signal_scope = sample_signal_scope(5528, 1);
        let enrolled = vec![EngineEnrolledSpeaker {
            speaker_id: selene_kernel_contracts::ph1_voice_id::SpeakerId::new(
                "speaker_audit_unknown_learn_path_unscoped",
            )
            .unwrap(),
            user_id: Some(
                selene_kernel_contracts::ph1_voice_id::UserId::new(
                    "user_audit_unknown_learn_path_unscoped",
                )
                .unwrap(),
            ),
            fingerprint: 7,
            profile_embedding: Some(simulation_profile_embedding_from_seed(7)),
        }];
        let out = runtime
            .run_identity_assertion_with_signal_emission(
                &mut store,
                &req,
                context,
                enrolled,
                EngineVoiceIdObservation {
                    primary_fingerprint: Some(7),
                    secondary_fingerprint: None,
                    primary_embedding: None,
                    secondary_embedding: None,
                    spoof_risk: false,
                },
                signal_scope.clone(),
            )
            .expect("unscoped unknown identity run with signal emission must succeed");

        assert!(matches!(
            out,
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(_)
        ));

        let all_rows = store.audit_events_by_correlation(CorrelationId(5528));
        let learn_row = all_rows
            .iter()
            .find(|row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.LEARN"))
            .expect("fallback PH1.LEARN audit row must exist");
        let learn_payload = &learn_row.payload_min.entries;
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("feedback_event_type").unwrap())
                .expect("feedback_event_type must exist")
                .as_str(),
            "VoiceIdFalseReject"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("learn_signal_type").unwrap())
                .expect("learn_signal_type must exist")
                .as_str(),
            "VoiceIdFalseReject"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("path_type").unwrap())
                .expect("path_type must exist")
                .as_str(),
            "PathA_Defect"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("signal_bucket must exist")
                .as_str(),
            "VoiceIdFalseReject"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_decision").unwrap())
                .expect("voice_decision must exist")
                .as_str(),
            "UNKNOWN"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_reason_code_hex").unwrap())
                .expect("voice_reason_code_hex must exist")
                .as_str(),
            format!(
                "0x{:X}",
                engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE.0
            )
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_score_bp").unwrap())
                .expect("voice_score_bp must exist")
                .as_str(),
            voice_score_bp(&out).to_string()
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("voice_margin_to_next_bp").unwrap()),
            "unknown fallback learn path should not invent margin context"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("bundle_id").unwrap()),
            "fallback PH1.LEARN row must not invent bundle_id"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("ingest_latency_ms").unwrap()),
            "fallback PH1.LEARN row must not invent ingest latency"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("decision_log_family").unwrap()),
            "fallback PH1.LEARN row must remain distinct from the H166 decision row"
        );
        assert_eq!(
            learn_row.reason_code,
            engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE
        );
        let expected_learn_idempotency = format!(
            "voice_learn_unscoped:{}:{}:{}:{}:{}",
            signal_scope.actor_user_id.as_str(),
            signal_scope.correlation_id.0,
            signal_scope.turn_id.0,
            "VoiceIdFalseReject",
            engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE.0
        );
        assert_eq!(
            learn_row.idempotency_key.as_deref(),
            Some(expected_learn_idempotency.as_str())
        );

        let feedback_row = all_rows
            .iter()
            .find(
                |row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.FEEDBACK"),
            )
            .expect("fallback PH1.FEEDBACK audit row must still exist");
        let feedback_payload = &feedback_row.payload_min.entries;
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("signal_bucket must exist")
                .as_str(),
            "VoiceIdFalseReject"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("path_type").unwrap())
                .expect("path_type must exist")
                .as_str(),
            "PathA_Defect"
        );
        let expected_feedback_idempotency = format!(
            "voice_feedback_unscoped:{}:{}:{}:{}",
            signal_scope.actor_user_id.as_str(),
            signal_scope.correlation_id.0,
            signal_scope.turn_id.0,
            engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE.0
        );
        assert_eq!(
            feedback_row.idempotency_key.as_deref(),
            Some(expected_feedback_idempotency.as_str())
        );
        assert_ne!(
            learn_row.idempotency_key, feedback_row.idempotency_key,
            "fallback PH1.LEARN row must remain distinct from fallback PH1.FEEDBACK"
        );

        let decision_row = all_rows
            .iter()
            .find(|row| {
                matches!(&row.engine, AuditEngine::Other(engine) if engine == PH1_VOICE_ID_ENGINE_ID)
                    && row.payload_min.entries.contains_key(
                        &PayloadKey::new("decision_log_family").unwrap()
                    )
            })
            .expect("H166 decision audit row must still exist");
        assert_eq!(
            decision_row
                .payload_min
                .entries
                .get(&PayloadKey::new("decision_v1").unwrap())
                .expect("decision_v1 must exist")
                .as_str(),
            "UNKNOWN"
        );
    }

    #[test]
    fn at_vid_live_gate_29_unscoped_low_margin_learn_audit_surfaces_defect_path_type() {
        let mut store = Ph1fStore::new_in_memory();
        let signal_scope = sample_signal_scope(5529, 1);
        let response = Ph1VoiceIdResponse::SpeakerAssertionOk(
            SpeakerAssertionOk::v1_with_metrics(
                selene_kernel_contracts::ph1_voice_id::SpeakerId::new(
                    "speaker_low_margin_learn_path_unscoped",
                )
                .unwrap(),
                Some(
                    selene_kernel_contracts::ph1_voice_id::UserId::new(
                        "user_low_margin_learn_path_unscoped",
                    )
                    .unwrap(),
                ),
                sample_decision_segments(),
                SpeakerLabel::speaker_a(),
                9_700,
                Some(250),
                Some(engine_voice_reason_codes::VID_OK_MATCHED),
                selene_kernel_contracts::ph1_voice_id::SpoofLivenessStatus::Live,
                vec![],
            )
            .expect("low-margin matched response fixture must be valid"),
        );
        let signal = map_voice_response_to_feedback_learn_signal(&response)
            .expect("low-margin matched response should map to fallback learn emission");
        assert_eq!(
            signal.reason_code,
            engine_voice_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN
        );
        assert_eq!(signal.decision, "OK_LOW_MARGIN");

        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let live_context = voice_feedback_learn_audit_context(&context, &response);
        emit_voice_id_feedback_and_learn_signal(&mut store, &signal_scope, signal, live_context)
            .expect("fallback learn emission must succeed for low-margin matched response");
        emit_voice_id_decision_audit(&mut store, context, &signal_scope, &response)
            .expect("decision audit emission must succeed for low-margin matched response");

        let all_rows = store.audit_events_by_correlation(CorrelationId(5529));
        let learn_row = all_rows
            .iter()
            .find(|row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.LEARN"))
            .expect("fallback PH1.LEARN audit row must exist");
        let learn_payload = &learn_row.payload_min.entries;
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("feedback_event_type").unwrap())
                .expect("feedback_event_type must exist")
                .as_str(),
            "VoiceIdFalseAccept"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("learn_signal_type").unwrap())
                .expect("learn_signal_type must exist")
                .as_str(),
            "VoiceIdFalseAccept"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("path_type").unwrap())
                .expect("path_type must exist")
                .as_str(),
            "PathA_Defect"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("signal_bucket must exist")
                .as_str(),
            "VoiceIdFalseAccept"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_decision").unwrap())
                .expect("voice_decision must exist")
                .as_str(),
            "OK_LOW_MARGIN"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_reason_code_hex").unwrap())
                .expect("voice_reason_code_hex must exist")
                .as_str(),
            format!(
                "0x{:X}",
                engine_voice_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN.0
            )
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_score_bp").unwrap())
                .expect("voice_score_bp must exist")
                .as_str(),
            "9700"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_margin_to_next_bp").unwrap())
                .expect("voice_margin_to_next_bp must exist")
                .as_str(),
            "250"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("bundle_id").unwrap()),
            "fallback PH1.LEARN row must not invent bundle_id"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("ingest_latency_ms").unwrap()),
            "fallback PH1.LEARN row must not invent ingest latency"
        );
        assert_eq!(
            learn_row.reason_code,
            engine_voice_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN
        );
        let expected_learn_idempotency = format!(
            "voice_learn_unscoped:{}:{}:{}:{}:{}",
            signal_scope.actor_user_id.as_str(),
            signal_scope.correlation_id.0,
            signal_scope.turn_id.0,
            "VoiceIdFalseAccept",
            engine_voice_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN.0
        );
        assert_eq!(
            learn_row.idempotency_key.as_deref(),
            Some(expected_learn_idempotency.as_str())
        );

        let feedback_row = all_rows
            .iter()
            .find(
                |row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.FEEDBACK"),
            )
            .expect("fallback PH1.FEEDBACK audit row must still exist");
        let feedback_payload = &feedback_row.payload_min.entries;
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("signal_bucket must exist")
                .as_str(),
            "VoiceIdFalseAccept"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("path_type").unwrap())
                .expect("path_type must exist")
                .as_str(),
            "PathA_Defect"
        );
        let expected_feedback_idempotency = format!(
            "voice_feedback_unscoped:{}:{}:{}:{}",
            signal_scope.actor_user_id.as_str(),
            signal_scope.correlation_id.0,
            signal_scope.turn_id.0,
            engine_voice_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN.0
        );
        assert_eq!(
            feedback_row.idempotency_key.as_deref(),
            Some(expected_feedback_idempotency.as_str())
        );
        assert_ne!(
            learn_row.idempotency_key, feedback_row.idempotency_key,
            "fallback PH1.LEARN row must remain distinct from fallback PH1.FEEDBACK"
        );

        let decision_row = all_rows
            .iter()
            .find(|row| {
                matches!(&row.engine, AuditEngine::Other(engine) if engine == PH1_VOICE_ID_ENGINE_ID)
                    && row.payload_min.entries.contains_key(
                        &PayloadKey::new("decision_log_family").unwrap()
                    )
            })
            .expect("H166 decision audit row must still exist");
        assert_eq!(
            decision_row.reason_code,
            engine_voice_reason_codes::VID_OK_MATCHED
        );
        assert_eq!(
            decision_row
                .payload_min
                .entries
                .get(&PayloadKey::new("decision_v1").unwrap())
                .expect("decision_v1 must exist")
                .as_str(),
            "OK"
        );
    }

    #[test]
    fn at_vid_live_gate_30_unscoped_low_score_learn_audit_surfaces_improvement_path_type() {
        let mut store = Ph1fStore::new_in_memory();
        let signal_scope = sample_signal_scope(5530, 1);
        let response = Ph1VoiceIdResponse::SpeakerAssertionOk(
            SpeakerAssertionOk::v1_with_metrics(
                selene_kernel_contracts::ph1_voice_id::SpeakerId::new(
                    "speaker_low_score_learn_path_unscoped",
                )
                .unwrap(),
                Some(
                    selene_kernel_contracts::ph1_voice_id::UserId::new(
                        "user_low_score_learn_path_unscoped",
                    )
                    .unwrap(),
                ),
                sample_decision_segments(),
                SpeakerLabel::speaker_a(),
                9_400,
                Some(350),
                Some(engine_voice_reason_codes::VID_OK_MATCHED),
                selene_kernel_contracts::ph1_voice_id::SpoofLivenessStatus::Live,
                vec![],
            )
            .expect("low-score matched response fixture must be valid"),
        );
        let signal = map_voice_response_to_feedback_learn_signal(&response)
            .expect("low-score matched response should map to fallback learn emission");
        assert_eq!(
            signal.reason_code,
            engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE
        );
        assert_eq!(signal.decision, "OK_LOW_SCORE");

        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let live_context = voice_feedback_learn_audit_context(&context, &response);
        emit_voice_id_feedback_and_learn_signal(&mut store, &signal_scope, signal, live_context)
            .expect("fallback learn emission must succeed for low-score matched response");
        emit_voice_id_decision_audit(&mut store, context, &signal_scope, &response)
            .expect("decision audit emission must succeed for low-score matched response");

        let all_rows = store.audit_events_by_correlation(CorrelationId(5530));
        let learn_row = all_rows
            .iter()
            .find(|row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.LEARN"))
            .expect("fallback PH1.LEARN audit row must exist");
        let learn_payload = &learn_row.payload_min.entries;
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("feedback_event_type").unwrap())
                .expect("feedback_event_type must exist")
                .as_str(),
            "VoiceIdReauthFriction"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("learn_signal_type").unwrap())
                .expect("learn_signal_type must exist")
                .as_str(),
            "VoiceIdReauthFriction"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("path_type").unwrap())
                .expect("path_type must exist")
                .as_str(),
            "PathB_Improvement"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("signal_bucket must exist")
                .as_str(),
            "VoiceIdReauthFriction"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_decision").unwrap())
                .expect("voice_decision must exist")
                .as_str(),
            "OK_LOW_SCORE"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_reason_code_hex").unwrap())
                .expect("voice_reason_code_hex must exist")
                .as_str(),
            format!(
                "0x{:X}",
                engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE.0
            )
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_score_bp").unwrap())
                .expect("voice_score_bp must exist")
                .as_str(),
            "9400"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_margin_to_next_bp").unwrap())
                .expect("voice_margin_to_next_bp must exist")
                .as_str(),
            "350"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("bundle_id").unwrap()),
            "fallback PH1.LEARN row must not invent bundle_id"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("ingest_latency_ms").unwrap()),
            "fallback PH1.LEARN row must not invent ingest latency"
        );
        assert_eq!(
            learn_row.reason_code,
            engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE
        );
        let expected_learn_idempotency = format!(
            "voice_learn_unscoped:{}:{}:{}:{}:{}",
            signal_scope.actor_user_id.as_str(),
            signal_scope.correlation_id.0,
            signal_scope.turn_id.0,
            "VoiceIdReauthFriction",
            engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE.0
        );
        assert_eq!(
            learn_row.idempotency_key.as_deref(),
            Some(expected_learn_idempotency.as_str())
        );

        let feedback_row = all_rows
            .iter()
            .find(
                |row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.FEEDBACK"),
            )
            .expect("fallback PH1.FEEDBACK audit row must still exist");
        let feedback_payload = &feedback_row.payload_min.entries;
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("signal_bucket must exist")
                .as_str(),
            "VoiceIdReauthFriction"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("path_type").unwrap())
                .expect("path_type must exist")
                .as_str(),
            "PathB_Improvement"
        );
        let expected_feedback_idempotency = format!(
            "voice_feedback_unscoped:{}:{}:{}:{}",
            signal_scope.actor_user_id.as_str(),
            signal_scope.correlation_id.0,
            signal_scope.turn_id.0,
            engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE.0
        );
        assert_eq!(
            feedback_row.idempotency_key.as_deref(),
            Some(expected_feedback_idempotency.as_str())
        );
        assert_ne!(
            learn_row.idempotency_key, feedback_row.idempotency_key,
            "fallback PH1.LEARN row must remain distinct from fallback PH1.FEEDBACK"
        );

        let decision_row = all_rows
            .iter()
            .find(|row| {
                matches!(&row.engine, AuditEngine::Other(engine) if engine == PH1_VOICE_ID_ENGINE_ID)
                    && row.payload_min.entries.contains_key(
                        &PayloadKey::new("decision_log_family").unwrap()
                    )
            })
            .expect("H166 decision audit row must still exist");
        assert_eq!(
            decision_row.reason_code,
            engine_voice_reason_codes::VID_OK_MATCHED
        );
        assert_eq!(
            decision_row
                .payload_min
                .entries
                .get(&PayloadKey::new("decision_v1").unwrap())
                .expect("decision_v1 must exist")
                .as_str(),
            "OK"
        );
    }

    #[test]
    fn at_vid_live_gate_31_scoped_unknown_learn_audit_surfaces_signal_bucket() {
        let mut store = Ph1fStore::new_in_memory();
        let runtime = Ph1VoiceIdLiveRuntime::default();
        let req = sample_live_request();
        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let signal_scope = sample_scoped_signal_scope(5531, 1);
        seed_scoped_feedback_identity_device(&mut store, &signal_scope);
        let enrolled = vec![EngineEnrolledSpeaker {
            speaker_id: selene_kernel_contracts::ph1_voice_id::SpeakerId::new(
                "speaker_scoped_unknown_learn_bucket",
            )
            .unwrap(),
            user_id: Some(
                selene_kernel_contracts::ph1_voice_id::UserId::new(
                    "user_scoped_unknown_learn_bucket",
                )
                .unwrap(),
            ),
            fingerprint: 7,
            profile_embedding: Some(simulation_profile_embedding_from_seed(7)),
        }];

        let out = runtime
            .run_identity_assertion_with_signal_emission(
                &mut store,
                &req,
                context,
                enrolled,
                EngineVoiceIdObservation {
                    primary_fingerprint: Some(7),
                    secondary_fingerprint: None,
                    primary_embedding: None,
                    secondary_embedding: None,
                    spoof_risk: false,
                },
                signal_scope.clone(),
            )
            .expect("scoped unknown identity run with signal emission must succeed");

        assert!(matches!(
            out,
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(_)
        ));

        let all_rows = store.audit_events_by_correlation(CorrelationId(5531));
        let learn_row = all_rows
            .iter()
            .find(|row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.LEARN"))
            .expect("scoped PH1.LEARN bundle audit row must exist");
        let learn_payload = &learn_row.payload_min.entries;
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("learn_signal_type").unwrap())
                .expect("learn_signal_type must exist")
                .as_str(),
            "VoiceIdFalseReject"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("signal_bucket must exist")
                .as_str(),
            "VoiceIdFalseReject"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("path_type").unwrap())
                .expect("path_type must exist")
                .as_str(),
            "PathA_Defect"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("final_identity_decision").unwrap())
                .expect("final_identity_decision must exist when scoped learn margin is absent")
                .as_str(),
            "UNKNOWN"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("final_reason_code_hex").unwrap())
                .expect("final_reason_code_hex must exist when scoped learn margin is absent")
                .as_str(),
            format!("0x{:X}", response_reason_code(&out).0)
        );
        assert!(
            learn_payload.contains_key(&PayloadKey::new("bundle_id").unwrap()),
            "scoped PH1.LEARN row must still carry bundle_id"
        );
        assert!(
            learn_payload.contains_key(&PayloadKey::new("ingest_latency_ms").unwrap()),
            "scoped PH1.LEARN row must still carry ingest_latency_ms"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("decision_log_family").unwrap()),
            "scoped PH1.LEARN row must remain distinct from the H166 decision row"
        );

        let feedback_row = all_rows
            .iter()
            .find(
                |row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.FEEDBACK"),
            )
            .expect("scoped PH1.FEEDBACK audit row must still exist");
        assert_eq!(
            feedback_row
                .payload_min
                .entries
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("feedback signal_bucket must exist")
                .as_str(),
            "VoiceIdFalseReject"
        );
    }

    #[test]
    fn at_vid_live_gate_32_scoped_low_margin_learn_audit_surfaces_signal_bucket() {
        let mut store = Ph1fStore::new_in_memory();
        let signal_scope = sample_scoped_signal_scope(5532, 1);
        seed_scoped_feedback_identity_device(&mut store, &signal_scope);
        let response = Ph1VoiceIdResponse::SpeakerAssertionOk(
            SpeakerAssertionOk::v1_with_metrics(
                selene_kernel_contracts::ph1_voice_id::SpeakerId::new(
                    "speaker_scoped_low_margin_learn_bucket",
                )
                .unwrap(),
                Some(
                    selene_kernel_contracts::ph1_voice_id::UserId::new(
                        "user_scoped_low_margin_learn_bucket",
                    )
                    .unwrap(),
                ),
                sample_decision_segments(),
                SpeakerLabel::speaker_a(),
                9_700,
                Some(250),
                Some(engine_voice_reason_codes::VID_OK_MATCHED),
                selene_kernel_contracts::ph1_voice_id::SpoofLivenessStatus::Live,
                vec![],
            )
            .expect("low-margin matched response fixture must be valid"),
        );
        let signal = map_voice_response_to_feedback_learn_signal(&response)
            .expect("low-margin matched response should map to scoped learn emission");

        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let live_context = voice_feedback_learn_audit_context(&context, &response);
        emit_voice_id_feedback_and_learn_signal(&mut store, &signal_scope, signal, live_context)
            .expect("scoped low-margin learn emission must succeed");
        emit_voice_id_decision_audit(&mut store, context, &signal_scope, &response)
            .expect("decision audit emission must succeed for scoped low-margin learn test");

        let all_rows = store.audit_events_by_correlation(CorrelationId(5532));
        let learn_row = all_rows
            .iter()
            .find(|row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.LEARN"))
            .expect("scoped PH1.LEARN bundle audit row must exist");
        let learn_payload = &learn_row.payload_min.entries;
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("learn_signal_type").unwrap())
                .expect("learn_signal_type must exist")
                .as_str(),
            "VoiceIdFalseAccept"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("signal_bucket must exist")
                .as_str(),
            "VoiceIdFalseAccept"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("path_type").unwrap())
                .expect("path_type must exist")
                .as_str(),
            "PathA_Defect"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_margin_to_next_bp").unwrap())
                .expect("voice_margin_to_next_bp must exist")
                .as_str(),
            "250"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("final_identity_decision").unwrap()),
            "low-margin scoped learn row must remain blocked from final_identity_decision"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("final_reason_code_hex").unwrap()),
            "low-margin scoped learn row must remain blocked from final_reason_code_hex"
        );
        assert!(
            learn_payload.contains_key(&PayloadKey::new("bundle_id").unwrap()),
            "scoped PH1.LEARN row must still carry bundle_id"
        );
        assert!(
            learn_payload.contains_key(&PayloadKey::new("ingest_latency_ms").unwrap()),
            "scoped PH1.LEARN row must still carry ingest_latency_ms"
        );

        let decision_row = all_rows
            .iter()
            .find(|row| {
                matches!(&row.engine, AuditEngine::Other(engine) if engine == PH1_VOICE_ID_ENGINE_ID)
                    && row
                        .payload_min
                        .entries
                        .contains_key(&PayloadKey::new("decision_log_family").unwrap())
            })
            .expect("H166 decision audit row must still exist");
        assert_eq!(
            decision_row.reason_code,
            engine_voice_reason_codes::VID_OK_MATCHED
        );
    }

    #[test]
    fn at_vid_live_gate_33_unscoped_low_score_learn_audit_surfaces_signal_bucket() {
        let mut store = Ph1fStore::new_in_memory();
        let signal_scope = sample_signal_scope(5533, 1);
        let response = Ph1VoiceIdResponse::SpeakerAssertionOk(
            SpeakerAssertionOk::v1_with_metrics(
                selene_kernel_contracts::ph1_voice_id::SpeakerId::new(
                    "speaker_unscoped_low_score_learn_bucket",
                )
                .unwrap(),
                Some(
                    selene_kernel_contracts::ph1_voice_id::UserId::new(
                        "user_unscoped_low_score_learn_bucket",
                    )
                    .unwrap(),
                ),
                sample_decision_segments(),
                SpeakerLabel::speaker_a(),
                9_400,
                Some(350),
                Some(engine_voice_reason_codes::VID_OK_MATCHED),
                selene_kernel_contracts::ph1_voice_id::SpoofLivenessStatus::Live,
                vec![],
            )
            .expect("low-score matched response fixture must be valid"),
        );
        let signal = map_voice_response_to_feedback_learn_signal(&response)
            .expect("low-score matched response should map to fallback learn emission");

        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let live_context = voice_feedback_learn_audit_context(&context, &response);
        emit_voice_id_feedback_and_learn_signal(&mut store, &signal_scope, signal, live_context)
            .expect("fallback learn emission must succeed for low-score matched response");
        emit_voice_id_decision_audit(&mut store, context, &signal_scope, &response)
            .expect("decision audit emission must succeed for low-score matched response");

        let all_rows = store.audit_events_by_correlation(CorrelationId(5533));
        let learn_row = all_rows
            .iter()
            .find(|row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.LEARN"))
            .expect("fallback PH1.LEARN audit row must exist");
        let learn_payload = &learn_row.payload_min.entries;
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("learn_signal_type").unwrap())
                .expect("learn_signal_type must exist")
                .as_str(),
            "VoiceIdReauthFriction"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("signal_bucket must exist")
                .as_str(),
            "VoiceIdReauthFriction"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("path_type").unwrap())
                .expect("path_type must exist")
                .as_str(),
            "PathB_Improvement"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("bundle_id").unwrap()),
            "fallback PH1.LEARN row must not invent bundle_id"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("ingest_latency_ms").unwrap()),
            "fallback PH1.LEARN row must not invent ingest_latency_ms"
        );
        let expected_learn_idempotency = format!(
            "voice_learn_unscoped:{}:{}:{}:{}:{}",
            signal_scope.actor_user_id.as_str(),
            signal_scope.correlation_id.0,
            signal_scope.turn_id.0,
            "VoiceIdReauthFriction",
            engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE.0
        );
        assert_eq!(
            learn_row.idempotency_key.as_deref(),
            Some(expected_learn_idempotency.as_str())
        );

        let feedback_row = all_rows
            .iter()
            .find(
                |row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.FEEDBACK"),
            )
            .expect("fallback PH1.FEEDBACK audit row must still exist");
        assert_eq!(
            feedback_row
                .payload_min
                .entries
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("feedback signal_bucket must exist")
                .as_str(),
            "VoiceIdReauthFriction"
        );
        assert_ne!(
            learn_row.idempotency_key, feedback_row.idempotency_key,
            "fallback PH1.LEARN row must remain distinct from fallback PH1.FEEDBACK"
        );
    }

    #[test]
    fn at_vid_live_gate_34_scoped_unknown_feedback_audit_surfaces_identity_context() {
        let mut store = Ph1fStore::new_in_memory();
        let runtime = Ph1VoiceIdLiveRuntime::default();
        let req = sample_live_request();
        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let expected_platform = platform_label(context.platform);
        let expected_channel = channel_label(context.channel);
        let signal_scope = sample_scoped_signal_scope(5534, 1);
        seed_scoped_feedback_identity_device(&mut store, &signal_scope);
        let enrolled = vec![EngineEnrolledSpeaker {
            speaker_id: selene_kernel_contracts::ph1_voice_id::SpeakerId::new(
                "speaker_scoped_unknown_feedback_identity_context",
            )
            .unwrap(),
            user_id: Some(
                selene_kernel_contracts::ph1_voice_id::UserId::new(
                    "user_scoped_unknown_feedback_identity_context",
                )
                .unwrap(),
            ),
            fingerprint: 7,
            profile_embedding: Some(simulation_profile_embedding_from_seed(7)),
        }];

        let out = runtime
            .run_identity_assertion_with_signal_emission(
                &mut store,
                &req,
                context,
                enrolled,
                EngineVoiceIdObservation {
                    primary_fingerprint: Some(7),
                    secondary_fingerprint: None,
                    primary_embedding: None,
                    secondary_embedding: None,
                    spoof_risk: false,
                },
                signal_scope.clone(),
            )
            .expect("scoped unknown identity run with signal emission must succeed");

        let feedback_row = store
            .ph1feedback_audit_rows(CorrelationId(5534))
            .into_iter()
            .next()
            .expect("scoped PH1.FEEDBACK audit row must exist");
        let feedback_payload = &feedback_row.payload_min.entries;
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("identity_tier_v2").unwrap())
                .expect("identity_tier_v2 must exist")
                .as_str(),
            identity_tier_label(out.identity_v2().identity_tier_v2)
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("platform").unwrap())
                .expect("platform must exist")
                .as_str(),
            expected_platform
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("channel").unwrap())
                .expect("channel must exist")
                .as_str(),
            expected_channel
        );
        assert!(
            !feedback_payload.contains_key(&PayloadKey::new("decision_log_family").unwrap()),
            "feedback row must remain distinct from the H166 decision row"
        );
        assert!(
            !feedback_payload.contains_key(&PayloadKey::new("decision_v1").unwrap()),
            "feedback row must not gain decision_v1"
        );

        let decision_row = store
            .audit_events_by_correlation(CorrelationId(5534))
            .into_iter()
            .find(|row| {
                matches!(&row.engine, AuditEngine::Other(engine) if engine == PH1_VOICE_ID_ENGINE_ID)
                    && row
                        .payload_min
                        .entries
                        .contains_key(&PayloadKey::new("decision_log_family").unwrap())
            })
            .expect("H166 decision audit row must exist");
        assert_eq!(
            decision_row
                .payload_min
                .entries
                .get(&PayloadKey::new("decision_v1").unwrap())
                .expect("decision_v1 must exist")
                .as_str(),
            "UNKNOWN"
        );
    }

    #[test]
    fn at_vid_live_gate_35_scoped_low_margin_learn_audit_surfaces_identity_context() {
        let mut store = Ph1fStore::new_in_memory();
        let signal_scope = sample_scoped_signal_scope(5535, 1);
        seed_scoped_feedback_identity_device(&mut store, &signal_scope);
        let response = Ph1VoiceIdResponse::SpeakerAssertionOk(
            SpeakerAssertionOk::v1_with_metrics(
                selene_kernel_contracts::ph1_voice_id::SpeakerId::new(
                    "speaker_scoped_low_margin_learn_identity_context",
                )
                .unwrap(),
                Some(
                    selene_kernel_contracts::ph1_voice_id::UserId::new(
                        "user_scoped_low_margin_learn_identity_context",
                    )
                    .unwrap(),
                ),
                sample_decision_segments(),
                SpeakerLabel::speaker_a(),
                9_700,
                Some(250),
                Some(engine_voice_reason_codes::VID_OK_MATCHED),
                selene_kernel_contracts::ph1_voice_id::SpoofLivenessStatus::Live,
                vec![],
            )
            .expect("low-margin matched response fixture must be valid"),
        );
        let signal = map_voice_response_to_feedback_learn_signal(&response)
            .expect("low-margin matched response should map to scoped learn emission");
        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let expected_platform = platform_label(context.platform);
        let expected_channel = channel_label(context.channel);
        let live_context = voice_feedback_learn_audit_context(&context, &response);

        emit_voice_id_feedback_and_learn_signal(&mut store, &signal_scope, signal, live_context)
            .expect("scoped low-margin learn emission must succeed");
        emit_voice_id_decision_audit(&mut store, context, &signal_scope, &response)
            .expect("decision audit emission must succeed for scoped low-margin learn test");

        let all_rows = store.audit_events_by_correlation(CorrelationId(5535));
        let learn_row = all_rows
            .iter()
            .find(|row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.LEARN"))
            .expect("scoped PH1.LEARN bundle audit row must exist");
        let learn_payload = &learn_row.payload_min.entries;
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("identity_tier_v2").unwrap())
                .expect("identity_tier_v2 must exist")
                .as_str(),
            identity_tier_label(response.identity_v2().identity_tier_v2)
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("platform").unwrap())
                .expect("platform must exist")
                .as_str(),
            expected_platform
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("channel").unwrap())
                .expect("channel must exist")
                .as_str(),
            expected_channel
        );
        assert!(
            learn_payload.contains_key(&PayloadKey::new("bundle_id").unwrap()),
            "scoped learn row must preserve bundle_id"
        );
        assert!(
            learn_payload.contains_key(&PayloadKey::new("ingest_latency_ms").unwrap()),
            "scoped learn row must preserve ingest latency"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("decision_log_family").unwrap()),
            "learn row must remain distinct from the H166 decision row"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("decision_v1").unwrap()),
            "learn row must not gain decision_v1"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("final_identity_decision").unwrap()),
            "low-margin scoped learn row must remain blocked from final_identity_decision in H176"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("final_reason_code_hex").unwrap()),
            "low-margin scoped learn row must remain blocked from final_reason_code_hex in H176"
        );
    }

    #[test]
    fn at_vid_live_gate_36_unscoped_unknown_feedback_audit_surfaces_identity_context() {
        let mut store = Ph1fStore::new_in_memory();
        let runtime = Ph1VoiceIdLiveRuntime::default();
        let req = sample_live_request();
        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let expected_platform = platform_label(context.platform);
        let expected_channel = channel_label(context.channel);
        let signal_scope = sample_signal_scope(5536, 1);
        let enrolled = vec![EngineEnrolledSpeaker {
            speaker_id: selene_kernel_contracts::ph1_voice_id::SpeakerId::new(
                "speaker_unscoped_unknown_feedback_identity_context",
            )
            .unwrap(),
            user_id: Some(
                selene_kernel_contracts::ph1_voice_id::UserId::new(
                    "user_unscoped_unknown_feedback_identity_context",
                )
                .unwrap(),
            ),
            fingerprint: 7,
            profile_embedding: Some(simulation_profile_embedding_from_seed(7)),
        }];

        let out = runtime
            .run_identity_assertion_with_signal_emission(
                &mut store,
                &req,
                context,
                enrolled,
                EngineVoiceIdObservation {
                    primary_fingerprint: Some(7),
                    secondary_fingerprint: None,
                    primary_embedding: None,
                    secondary_embedding: None,
                    spoof_risk: false,
                },
                signal_scope.clone(),
            )
            .expect("unscoped unknown identity run with signal emission must succeed");

        let all_rows = store.audit_events_by_correlation(CorrelationId(5536));
        let feedback_row = all_rows
            .iter()
            .find(
                |row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.FEEDBACK"),
            )
            .expect("fallback PH1.FEEDBACK audit row must exist");
        let feedback_payload = &feedback_row.payload_min.entries;
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("identity_tier_v2").unwrap())
                .expect("identity_tier_v2 must exist")
                .as_str(),
            identity_tier_label(out.identity_v2().identity_tier_v2)
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("platform").unwrap())
                .expect("platform must exist")
                .as_str(),
            expected_platform
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("channel").unwrap())
                .expect("channel must exist")
                .as_str(),
            expected_channel
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("final_identity_decision").unwrap())
                .expect("final_identity_decision must exist")
                .as_str(),
            voice_decision_label(voice_decision_v1(&out))
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("final_reason_code_hex").unwrap())
                .expect("final_reason_code_hex must exist")
                .as_str(),
            format!("0x{:X}", response_reason_code(&out).0)
        );
        assert!(
            !feedback_payload.contains_key(&PayloadKey::new("decision_log_family").unwrap()),
            "fallback feedback row must remain distinct from the H166 decision row"
        );
        assert!(
            !feedback_payload.contains_key(&PayloadKey::new("decision_v1").unwrap()),
            "fallback feedback row must not gain decision_v1"
        );
    }

    #[test]
    fn at_vid_live_gate_37_unscoped_low_score_learn_audit_surfaces_identity_context() {
        let mut store = Ph1fStore::new_in_memory();
        let signal_scope = sample_signal_scope(5537, 1);
        let response = Ph1VoiceIdResponse::SpeakerAssertionOk(
            SpeakerAssertionOk::v1_with_metrics(
                selene_kernel_contracts::ph1_voice_id::SpeakerId::new(
                    "speaker_unscoped_low_score_learn_identity_context",
                )
                .unwrap(),
                Some(
                    selene_kernel_contracts::ph1_voice_id::UserId::new(
                        "user_unscoped_low_score_learn_identity_context",
                    )
                    .unwrap(),
                ),
                sample_decision_segments(),
                SpeakerLabel::speaker_a(),
                9_400,
                Some(350),
                Some(engine_voice_reason_codes::VID_OK_MATCHED),
                selene_kernel_contracts::ph1_voice_id::SpoofLivenessStatus::Live,
                vec![],
            )
            .expect("low-score matched response fixture must be valid"),
        );
        let signal = map_voice_response_to_feedback_learn_signal(&response)
            .expect("low-score matched response should map to fallback learn emission");
        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let expected_platform = platform_label(context.platform);
        let expected_channel = channel_label(context.channel);
        let live_context = voice_feedback_learn_audit_context(&context, &response);

        emit_voice_id_feedback_and_learn_signal(&mut store, &signal_scope, signal, live_context)
            .expect("fallback learn emission must succeed for low-score matched response");
        emit_voice_id_decision_audit(&mut store, context, &signal_scope, &response)
            .expect("decision audit emission must succeed for low-score matched response");

        let all_rows = store.audit_events_by_correlation(CorrelationId(5537));
        let learn_row = all_rows
            .iter()
            .find(|row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.LEARN"))
            .expect("fallback PH1.LEARN audit row must exist");
        let learn_payload = &learn_row.payload_min.entries;
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("identity_tier_v2").unwrap())
                .expect("identity_tier_v2 must exist")
                .as_str(),
            identity_tier_label(response.identity_v2().identity_tier_v2)
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("platform").unwrap())
                .expect("platform must exist")
                .as_str(),
            expected_platform
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("channel").unwrap())
                .expect("channel must exist")
                .as_str(),
            expected_channel
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("final_identity_decision").unwrap())
                .expect("final_identity_decision must exist")
                .as_str(),
            "OK"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("final_reason_code_hex").unwrap())
                .expect("final_reason_code_hex must exist")
                .as_str(),
            format!("0x{:X}", engine_voice_reason_codes::VID_OK_MATCHED.0)
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_decision").unwrap())
                .expect("voice_decision must exist")
                .as_str(),
            "OK_LOW_SCORE"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_reason_code_hex").unwrap())
                .expect("voice_reason_code_hex must exist")
                .as_str(),
            format!(
                "0x{:X}",
                engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE.0
            )
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("signal_bucket must exist")
                .as_str(),
            "VoiceIdReauthFriction"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("bundle_id").unwrap()),
            "fallback learn row must not invent bundle_id"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("ingest_latency_ms").unwrap()),
            "fallback learn row must not invent ingest latency"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("decision_log_family").unwrap()),
            "fallback learn row must remain distinct from the H166 decision row"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("decision_v1").unwrap()),
            "fallback learn row must not gain decision_v1"
        );
    }

    #[test]
    fn at_vid_live_gate_38_scoped_low_margin_feedback_audit_surfaces_final_decision_context() {
        let mut store = Ph1fStore::new_in_memory();
        let signal_scope = sample_scoped_signal_scope(5538, 1);
        seed_scoped_feedback_identity_device(&mut store, &signal_scope);
        let response = Ph1VoiceIdResponse::SpeakerAssertionOk(
            SpeakerAssertionOk::v1_with_metrics(
                selene_kernel_contracts::ph1_voice_id::SpeakerId::new(
                    "speaker_scoped_low_margin_feedback_final_context",
                )
                .unwrap(),
                Some(
                    selene_kernel_contracts::ph1_voice_id::UserId::new(
                        "user_scoped_low_margin_feedback_final_context",
                    )
                    .unwrap(),
                ),
                sample_decision_segments(),
                SpeakerLabel::speaker_a(),
                9_700,
                Some(250),
                Some(engine_voice_reason_codes::VID_OK_MATCHED),
                selene_kernel_contracts::ph1_voice_id::SpoofLivenessStatus::Live,
                vec![],
            )
            .expect("low-margin matched response fixture must be valid"),
        );
        let signal = map_voice_response_to_feedback_learn_signal(&response)
            .expect("low-margin matched response should map to scoped feedback emission");
        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let live_context = voice_feedback_learn_audit_context(&context, &response);

        emit_voice_id_feedback_and_learn_signal(&mut store, &signal_scope, signal, live_context)
            .expect("scoped feedback emission must succeed for final decision context test");
        emit_voice_id_decision_audit(&mut store, context, &signal_scope, &response)
            .expect("decision audit emission must succeed for scoped feedback final context test");

        let feedback_row = store
            .ph1feedback_audit_rows(CorrelationId(5538))
            .into_iter()
            .next()
            .expect("scoped PH1.FEEDBACK audit row must exist");
        let feedback_payload = &feedback_row.payload_min.entries;
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("final_identity_decision").unwrap())
                .expect("final_identity_decision must exist")
                .as_str(),
            "OK"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("final_reason_code_hex").unwrap())
                .expect("final_reason_code_hex must exist")
                .as_str(),
            format!("0x{:X}", engine_voice_reason_codes::VID_OK_MATCHED.0)
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("voice_decision").unwrap())
                .expect("voice_decision must exist")
                .as_str(),
            "OK_LOW_MARGIN"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("voice_reason_code_hex").unwrap())
                .expect("voice_reason_code_hex must exist")
                .as_str(),
            format!(
                "0x{:X}",
                engine_voice_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN.0
            )
        );
        assert!(
            !feedback_payload.contains_key(&PayloadKey::new("decision_log_family").unwrap()),
            "scoped feedback row must remain distinct from the H166 decision row"
        );
        assert!(
            !feedback_payload.contains_key(&PayloadKey::new("decision_v1").unwrap()),
            "scoped feedback row must not gain decision_v1"
        );

        let all_rows = store.audit_events_by_correlation(CorrelationId(5538));
        let learn_row = all_rows
            .iter()
            .find(|row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.LEARN"))
            .expect("scoped PH1.LEARN bundle audit row must still exist");
        assert!(
            !learn_row
                .payload_min
                .entries
                .contains_key(&PayloadKey::new("final_identity_decision").unwrap()),
            "scoped learn row must remain unchanged in H175"
        );
        assert!(
            !learn_row
                .payload_min
                .entries
                .contains_key(&PayloadKey::new("final_reason_code_hex").unwrap()),
            "scoped learn row must remain unchanged in H175"
        );
    }

    #[test]
    fn at_vid_live_gate_39_unscoped_unknown_feedback_audit_surfaces_final_decision_context() {
        let mut store = Ph1fStore::new_in_memory();
        let runtime = Ph1VoiceIdLiveRuntime::default();
        let req = sample_live_request();
        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let signal_scope = sample_signal_scope(5539, 1);
        let enrolled = vec![EngineEnrolledSpeaker {
            speaker_id: selene_kernel_contracts::ph1_voice_id::SpeakerId::new(
                "speaker_unscoped_unknown_feedback_final_context",
            )
            .unwrap(),
            user_id: Some(
                selene_kernel_contracts::ph1_voice_id::UserId::new(
                    "user_unscoped_unknown_feedback_final_context",
                )
                .unwrap(),
            ),
            fingerprint: 7,
            profile_embedding: Some(simulation_profile_embedding_from_seed(7)),
        }];

        let out = runtime
            .run_identity_assertion_with_signal_emission(
                &mut store,
                &req,
                context,
                enrolled,
                EngineVoiceIdObservation {
                    primary_fingerprint: Some(7),
                    secondary_fingerprint: None,
                    primary_embedding: None,
                    secondary_embedding: None,
                    spoof_risk: false,
                },
                signal_scope.clone(),
            )
            .expect("unscoped unknown feedback final context run must succeed");

        let all_rows = store.audit_events_by_correlation(CorrelationId(5539));
        let feedback_row = all_rows
            .iter()
            .find(
                |row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.FEEDBACK"),
            )
            .expect("fallback PH1.FEEDBACK audit row must exist");
        let feedback_payload = &feedback_row.payload_min.entries;
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("final_identity_decision").unwrap())
                .expect("final_identity_decision must exist")
                .as_str(),
            "UNKNOWN"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("final_reason_code_hex").unwrap())
                .expect("final_reason_code_hex must exist")
                .as_str(),
            format!("0x{:X}", response_reason_code(&out).0)
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("voice_decision").unwrap())
                .expect("voice_decision must exist")
                .as_str(),
            "UNKNOWN"
        );
        assert_eq!(
            feedback_payload
                .get(&PayloadKey::new("voice_reason_code_hex").unwrap())
                .expect("voice_reason_code_hex must exist")
                .as_str(),
            format!("0x{:X}", response_reason_code(&out).0)
        );
        assert!(
            !feedback_payload.contains_key(&PayloadKey::new("decision_log_family").unwrap()),
            "fallback feedback row must remain distinct from the H166 decision row"
        );
        assert!(
            !feedback_payload.contains_key(&PayloadKey::new("decision_v1").unwrap()),
            "fallback feedback row must not gain decision_v1"
        );
    }

    #[test]
    fn at_vid_live_gate_40_unscoped_unknown_learn_audit_surfaces_final_decision_context() {
        let mut store = Ph1fStore::new_in_memory();
        let runtime = Ph1VoiceIdLiveRuntime::default();
        let req = sample_live_request();
        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let signal_scope = sample_signal_scope(5540, 1);
        let enrolled = vec![EngineEnrolledSpeaker {
            speaker_id: selene_kernel_contracts::ph1_voice_id::SpeakerId::new(
                "speaker_unscoped_unknown_learn_final_context",
            )
            .unwrap(),
            user_id: Some(
                selene_kernel_contracts::ph1_voice_id::UserId::new(
                    "user_unscoped_unknown_learn_final_context",
                )
                .unwrap(),
            ),
            fingerprint: 7,
            profile_embedding: Some(simulation_profile_embedding_from_seed(7)),
        }];

        let out = runtime
            .run_identity_assertion_with_signal_emission(
                &mut store,
                &req,
                context,
                enrolled,
                EngineVoiceIdObservation {
                    primary_fingerprint: Some(7),
                    secondary_fingerprint: None,
                    primary_embedding: None,
                    secondary_embedding: None,
                    spoof_risk: false,
                },
                signal_scope.clone(),
            )
            .expect("unscoped unknown learn final context run must succeed");

        let all_rows = store.audit_events_by_correlation(CorrelationId(5540));
        let learn_row = all_rows
            .iter()
            .find(|row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.LEARN"))
            .expect("fallback PH1.LEARN audit row must exist");
        let learn_payload = &learn_row.payload_min.entries;
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("final_identity_decision").unwrap())
                .expect("final_identity_decision must exist")
                .as_str(),
            "UNKNOWN"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("final_reason_code_hex").unwrap())
                .expect("final_reason_code_hex must exist")
                .as_str(),
            format!("0x{:X}", response_reason_code(&out).0)
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_decision").unwrap())
                .expect("voice_decision must exist")
                .as_str(),
            "UNKNOWN"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_reason_code_hex").unwrap())
                .expect("voice_reason_code_hex must exist")
                .as_str(),
            format!("0x{:X}", response_reason_code(&out).0)
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("bundle_id").unwrap()),
            "fallback learn row must not invent bundle_id"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("ingest_latency_ms").unwrap()),
            "fallback learn row must not invent ingest latency"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("decision_log_family").unwrap()),
            "fallback learn row must remain distinct from the H166 decision row"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("decision_v1").unwrap()),
            "fallback learn row must not gain decision_v1"
        );
    }

    #[test]
    fn at_vid_live_gate_41_unscoped_low_score_learn_audit_surfaces_final_decision_context() {
        let mut store = Ph1fStore::new_in_memory();
        let signal_scope = sample_signal_scope(5541, 1);
        let response = Ph1VoiceIdResponse::SpeakerAssertionOk(
            SpeakerAssertionOk::v1_with_metrics(
                selene_kernel_contracts::ph1_voice_id::SpeakerId::new(
                    "speaker_unscoped_low_score_learn_final_context",
                )
                .unwrap(),
                Some(
                    selene_kernel_contracts::ph1_voice_id::UserId::new(
                        "user_unscoped_low_score_learn_final_context",
                    )
                    .unwrap(),
                ),
                sample_decision_segments(),
                SpeakerLabel::speaker_a(),
                9_400,
                Some(350),
                Some(engine_voice_reason_codes::VID_OK_MATCHED),
                selene_kernel_contracts::ph1_voice_id::SpoofLivenessStatus::Live,
                vec![],
            )
            .expect("low-score matched response fixture must be valid"),
        );
        let signal = map_voice_response_to_feedback_learn_signal(&response)
            .expect("low-score matched response should map to fallback learn emission");
        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let live_context = voice_feedback_learn_audit_context(&context, &response);

        emit_voice_id_feedback_and_learn_signal(&mut store, &signal_scope, signal, live_context)
            .expect("fallback learn emission must succeed for low-score final context test");
        emit_voice_id_decision_audit(&mut store, context, &signal_scope, &response)
            .expect("decision audit emission must succeed for low-score final context test");

        let all_rows = store.audit_events_by_correlation(CorrelationId(5541));
        let learn_row = all_rows
            .iter()
            .find(|row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.LEARN"))
            .expect("fallback PH1.LEARN audit row must exist");
        let learn_payload = &learn_row.payload_min.entries;
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("final_identity_decision").unwrap())
                .expect("final_identity_decision must exist")
                .as_str(),
            "OK"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("final_reason_code_hex").unwrap())
                .expect("final_reason_code_hex must exist")
                .as_str(),
            format!("0x{:X}", engine_voice_reason_codes::VID_OK_MATCHED.0)
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_decision").unwrap())
                .expect("voice_decision must exist")
                .as_str(),
            "OK_LOW_SCORE"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_reason_code_hex").unwrap())
                .expect("voice_reason_code_hex must exist")
                .as_str(),
            format!(
                "0x{:X}",
                engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE.0
            )
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("signal_bucket must exist")
                .as_str(),
            "VoiceIdReauthFriction"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("bundle_id").unwrap()),
            "fallback learn row must not invent bundle_id"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("ingest_latency_ms").unwrap()),
            "fallback learn row must not invent ingest latency"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("decision_log_family").unwrap()),
            "fallback learn row must remain distinct from the H166 decision row"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("decision_v1").unwrap()),
            "fallback learn row must not gain decision_v1"
        );
    }

    #[test]
    fn at_vid_live_gate_42_scoped_unknown_learn_audit_surfaces_final_decision_context_when_margin_absent()
    {
        let mut store = Ph1fStore::new_in_memory();
        let runtime = Ph1VoiceIdLiveRuntime::default();
        let req = sample_live_request();
        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let signal_scope = sample_scoped_signal_scope(5542, 1);
        seed_scoped_feedback_identity_device(&mut store, &signal_scope);
        let enrolled = vec![EngineEnrolledSpeaker {
            speaker_id: selene_kernel_contracts::ph1_voice_id::SpeakerId::new(
                "speaker_scoped_unknown_learn_final_context_marginless",
            )
            .unwrap(),
            user_id: Some(
                selene_kernel_contracts::ph1_voice_id::UserId::new(
                    "user_scoped_unknown_learn_final_context_marginless",
                )
                .unwrap(),
            ),
            fingerprint: 7,
            profile_embedding: Some(simulation_profile_embedding_from_seed(7)),
        }];

        let out = runtime
            .run_identity_assertion_with_signal_emission(
                &mut store,
                &req,
                context,
                enrolled,
                EngineVoiceIdObservation {
                    primary_fingerprint: Some(7),
                    secondary_fingerprint: None,
                    primary_embedding: None,
                    secondary_embedding: None,
                    spoof_risk: false,
                },
                signal_scope.clone(),
            )
            .expect("scoped unknown learn final-context run must succeed");

        let all_rows = store.audit_events_by_correlation(CorrelationId(5542));
        let learn_row = all_rows
            .iter()
            .find(|row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.LEARN"))
            .expect("scoped PH1.LEARN bundle audit row must exist");
        let learn_payload = &learn_row.payload_min.entries;
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("final_identity_decision").unwrap())
                .expect("final_identity_decision must exist when margin is absent")
                .as_str(),
            "UNKNOWN"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("final_reason_code_hex").unwrap())
                .expect("final_reason_code_hex must exist when margin is absent")
                .as_str(),
            format!("0x{:X}", response_reason_code(&out).0)
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_decision").unwrap())
                .expect("voice_decision must exist")
                .as_str(),
            "UNKNOWN"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_reason_code_hex").unwrap())
                .expect("voice_reason_code_hex must exist")
                .as_str(),
            format!("0x{:X}", response_reason_code(&out).0)
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("voice_margin_to_next_bp").unwrap()),
            "marginless scoped learn row must not invent voice_margin_to_next_bp"
        );
        assert!(
            learn_payload.contains_key(&PayloadKey::new("bundle_id").unwrap()),
            "scoped learn row must preserve bundle_id"
        );
        assert!(
            learn_payload.contains_key(&PayloadKey::new("ingest_latency_ms").unwrap()),
            "scoped learn row must preserve ingest latency"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("decision_log_family").unwrap()),
            "scoped learn row must remain distinct from the H166 decision row"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("decision_v1").unwrap()),
            "scoped learn row must not gain decision_v1"
        );
    }

    #[test]
    fn at_vid_live_gate_43_scoped_low_score_learn_audit_surfaces_final_decision_context_when_margin_absent()
    {
        let mut store = Ph1fStore::new_in_memory();
        let signal_scope = sample_scoped_signal_scope(5543, 1);
        seed_scoped_feedback_identity_device(&mut store, &signal_scope);
        let response = Ph1VoiceIdResponse::SpeakerAssertionOk(
            SpeakerAssertionOk::v1_with_metrics(
                selene_kernel_contracts::ph1_voice_id::SpeakerId::new(
                    "speaker_scoped_low_score_learn_final_context_marginless",
                )
                .unwrap(),
                Some(
                    selene_kernel_contracts::ph1_voice_id::UserId::new(
                        "user_scoped_low_score_learn_final_context_marginless",
                    )
                    .unwrap(),
                ),
                sample_decision_segments(),
                SpeakerLabel::speaker_a(),
                9_400,
                None,
                Some(engine_voice_reason_codes::VID_OK_MATCHED),
                selene_kernel_contracts::ph1_voice_id::SpoofLivenessStatus::Live,
                vec![],
            )
            .expect("marginless low-score matched response fixture must be valid"),
        );
        let signal = map_voice_response_to_feedback_learn_signal(&response)
            .expect("marginless low-score matched response should map to scoped learn emission");
        assert_eq!(
            signal.reason_code,
            engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE
        );
        assert_eq!(signal.decision, "OK_LOW_SCORE");
        assert!(
            signal.margin_to_next_bp.is_none(),
            "H176 scoped low-score learn fixture must stay marginless"
        );

        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let live_context = voice_feedback_learn_audit_context(&context, &response);

        emit_voice_id_feedback_and_learn_signal(&mut store, &signal_scope, signal, live_context)
            .expect("marginless scoped low-score learn emission must succeed");
        emit_voice_id_decision_audit(&mut store, context, &signal_scope, &response)
            .expect("decision audit emission must succeed for marginless scoped low-score learn");

        let all_rows = store.audit_events_by_correlation(CorrelationId(5543));
        let learn_row = all_rows
            .iter()
            .find(|row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.LEARN"))
            .expect("scoped PH1.LEARN bundle audit row must exist");
        let learn_payload = &learn_row.payload_min.entries;
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("final_identity_decision").unwrap())
                .expect("final_identity_decision must exist when margin is absent")
                .as_str(),
            "OK"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("final_reason_code_hex").unwrap())
                .expect("final_reason_code_hex must exist when margin is absent")
                .as_str(),
            format!("0x{:X}", engine_voice_reason_codes::VID_OK_MATCHED.0)
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_decision").unwrap())
                .expect("voice_decision must exist")
                .as_str(),
            "OK_LOW_SCORE"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_reason_code_hex").unwrap())
                .expect("voice_reason_code_hex must exist")
                .as_str(),
            format!(
                "0x{:X}",
                engine_voice_reason_codes::VID_FAIL_LOW_CONFIDENCE.0
            )
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("signal_bucket must exist")
                .as_str(),
            "VoiceIdReauthFriction"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("voice_margin_to_next_bp").unwrap()),
            "marginless scoped low-score learn row must not invent voice_margin_to_next_bp"
        );
        assert!(
            learn_payload.contains_key(&PayloadKey::new("bundle_id").unwrap()),
            "scoped learn row must preserve bundle_id"
        );
        assert!(
            learn_payload.contains_key(&PayloadKey::new("ingest_latency_ms").unwrap()),
            "scoped learn row must preserve ingest latency"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("decision_log_family").unwrap()),
            "scoped learn row must remain distinct from the H166 decision row"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("decision_v1").unwrap()),
            "scoped learn row must not gain decision_v1"
        );
    }

    #[test]
    fn at_vid_live_gate_44_scoped_low_margin_learn_audit_does_not_surface_final_decision_context_when_margin_present()
    {
        let mut store = Ph1fStore::new_in_memory();
        let signal_scope = sample_scoped_signal_scope(5544, 1);
        seed_scoped_feedback_identity_device(&mut store, &signal_scope);
        let response = Ph1VoiceIdResponse::SpeakerAssertionOk(
            SpeakerAssertionOk::v1_with_metrics(
                selene_kernel_contracts::ph1_voice_id::SpeakerId::new(
                    "speaker_scoped_low_margin_learn_final_context_blocked",
                )
                .unwrap(),
                Some(
                    selene_kernel_contracts::ph1_voice_id::UserId::new(
                        "user_scoped_low_margin_learn_final_context_blocked",
                    )
                    .unwrap(),
                ),
                sample_decision_segments(),
                SpeakerLabel::speaker_a(),
                9_700,
                Some(250),
                Some(engine_voice_reason_codes::VID_OK_MATCHED),
                selene_kernel_contracts::ph1_voice_id::SpoofLivenessStatus::Live,
                vec![],
            )
            .expect("low-margin matched response fixture must be valid"),
        );
        let signal = map_voice_response_to_feedback_learn_signal(&response)
            .expect("low-margin matched response should map to scoped learn emission");
        assert_eq!(signal.decision, "OK_LOW_MARGIN");
        assert_eq!(
            signal.margin_to_next_bp,
            Some(250),
            "H176 blocked low-margin fixture must preserve margin"
        );

        let context = VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_audit".to_string()),
            Some(AppPlatform::Android),
            VoiceIdentityChannel::Explicit,
        );
        let live_context = voice_feedback_learn_audit_context(&context, &response);

        emit_voice_id_feedback_and_learn_signal(&mut store, &signal_scope, signal, live_context)
            .expect("scoped low-margin learn emission must still succeed");
        emit_voice_id_decision_audit(&mut store, context, &signal_scope, &response)
            .expect("decision audit emission must succeed for blocked low-margin scoped learn");

        let all_rows = store.audit_events_by_correlation(CorrelationId(5544));
        let learn_row = all_rows
            .iter()
            .find(|row| matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.LEARN"))
            .expect("scoped PH1.LEARN bundle audit row must exist");
        let learn_payload = &learn_row.payload_min.entries;
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_decision").unwrap())
                .expect("voice_decision must exist")
                .as_str(),
            "OK_LOW_MARGIN"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_reason_code_hex").unwrap())
                .expect("voice_reason_code_hex must exist")
                .as_str(),
            format!(
                "0x{:X}",
                engine_voice_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN.0
            )
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("voice_margin_to_next_bp").unwrap())
                .expect("voice_margin_to_next_bp must exist")
                .as_str(),
            "250"
        );
        assert_eq!(
            learn_payload
                .get(&PayloadKey::new("signal_bucket").unwrap())
                .expect("signal_bucket must exist")
                .as_str(),
            "VoiceIdFalseAccept"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("final_identity_decision").unwrap()),
            "low-margin scoped learn row must remain blocked from final_identity_decision"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("final_reason_code_hex").unwrap()),
            "low-margin scoped learn row must remain blocked from final_reason_code_hex"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("decision_log_family").unwrap()),
            "scoped learn row must remain distinct from the H166 decision row"
        );
        assert!(
            !learn_payload.contains_key(&PayloadKey::new("decision_v1").unwrap()),
            "scoped learn row must not gain decision_v1"
        );
    }
}
