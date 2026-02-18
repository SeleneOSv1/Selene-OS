#![forbid(unsafe_code)]

use crate::ph1_voice_id::UserId;
use crate::ph1j::{CorrelationId, TurnId};
use crate::ph1onb::OnboardingSessionId;
use crate::ph1position::TenantId;
use crate::ph1tts::{StyleModifier, StyleProfileRef};
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1EMOCORE_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);
pub const PH1EMOCORE_ENGINE_ID: &str = "PH1.EMO.CORE";
pub const PH1EMOCORE_IMPLEMENTATION_ID: &str = "PH1.EMO.CORE.001";
pub const PH1EMOCORE_ACTIVE_IMPLEMENTATION_IDS: &[&str] = &[PH1EMOCORE_IMPLEMENTATION_ID];

pub const EMO_SIM_001: &str = "EMO_SIM_001";
pub const EMO_SIM_002: &str = "EMO_SIM_002";
pub const EMO_SIM_003: &str = "EMO_SIM_003";
pub const EMO_SIM_004: &str = "EMO_SIM_004";
pub const EMO_SIM_005: &str = "EMO_SIM_005";
pub const EMO_SIM_006: &str = "EMO_SIM_006";

fn validate_token(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if value.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if value.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    if !value.is_ascii() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be ASCII",
        });
    }
    Ok(())
}

fn validate_opt_token(
    field: &'static str,
    value: &Option<String>,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if let Some(v) = value {
        validate_token(field, v, max_len)?;
    }
    Ok(())
}

fn style_modifier_rank(v: StyleModifier) -> u8 {
    match v {
        StyleModifier::Brief => 0,
        StyleModifier::Warm => 1,
        StyleModifier::Formal => 2,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmoCoreSimulationType {
    Draft,
    Commit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmoCoreCapabilityId {
    ClassifyProfileCommit,
    ReevaluateProfileCommit,
    PrivacyCommandCommit,
    ToneGuidanceDraft,
    SnapshotCaptureCommit,
    AuditEventCommit,
}

impl EmoCoreCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            EmoCoreCapabilityId::ClassifyProfileCommit => "PH1EMO_CLASSIFY_PROFILE_COMMIT_ROW",
            EmoCoreCapabilityId::ReevaluateProfileCommit => "PH1EMO_REEVALUATE_PROFILE_COMMIT_ROW",
            EmoCoreCapabilityId::PrivacyCommandCommit => "PH1EMO_PRIVACY_COMMAND_COMMIT_ROW",
            EmoCoreCapabilityId::ToneGuidanceDraft => "PH1EMO_TONE_GUIDANCE_DRAFT_ROW",
            EmoCoreCapabilityId::SnapshotCaptureCommit => "PH1EMO_SNAPSHOT_CAPTURE_COMMIT_ROW",
            EmoCoreCapabilityId::AuditEventCommit => "PH1EMO_AUDIT_EVENT_COMMIT_ROW",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmoPersonalityType {
    Passive,
    Domineering,
    Undetermined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmoPersonalityLockStatus {
    Locked,
    ReevalDue,
    ReevalChanged,
    ReevalConfirmed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmoPrivacyCommand {
    ForgetThisKey,
    ForgetAll,
    DoNotRemember,
    RecallOnly,
    KeepActive,
    Archive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmoPrivacyState {
    KeepActive,
    DoNotRemember,
    RecallOnly,
    Archive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmoStyleBucket {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmoTonePacing {
    Slow,
    Balanced,
    Fast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmoSnapshotStatus {
    Complete,
    Defer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmoAuditEventStatus {
    Recorded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmoSignalBundle {
    pub schema_version: SchemaVersion,
    pub assertive_score: u8,
    pub distress_score: u8,
    pub anger_score: u8,
    pub warmth_signal: u8,
}

impl EmoSignalBundle {
    pub fn v1(
        assertive_score: u8,
        distress_score: u8,
        anger_score: u8,
        warmth_signal: u8,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            schema_version: PH1EMOCORE_CONTRACT_VERSION,
            assertive_score,
            distress_score,
            anger_score,
            warmth_signal,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for EmoSignalBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1EMOCORE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "emo_signal_bundle.schema_version",
                reason: "must match PH1EMOCORE_CONTRACT_VERSION",
            });
        }
        if self.assertive_score > 100 {
            return Err(ContractViolation::InvalidValue {
                field: "emo_signal_bundle.assertive_score",
                reason: "must be <= 100",
            });
        }
        if self.distress_score > 100 {
            return Err(ContractViolation::InvalidValue {
                field: "emo_signal_bundle.distress_score",
                reason: "must be <= 100",
            });
        }
        if self.anger_score > 100 {
            return Err(ContractViolation::InvalidValue {
                field: "emo_signal_bundle.anger_score",
                reason: "must be <= 100",
            });
        }
        if self.warmth_signal > 100 {
            return Err(ContractViolation::InvalidValue {
                field: "emo_signal_bundle.warmth_signal",
                reason: "must be <= 100",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmoVoiceStyleProfile {
    pub schema_version: SchemaVersion,
    pub pace_bucket: EmoStyleBucket,
    pub energy_bucket: EmoStyleBucket,
    pub warmth_bucket: EmoStyleBucket,
}

impl EmoVoiceStyleProfile {
    pub fn v1(
        pace_bucket: EmoStyleBucket,
        energy_bucket: EmoStyleBucket,
        warmth_bucket: EmoStyleBucket,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            schema_version: PH1EMOCORE_CONTRACT_VERSION,
            pace_bucket,
            energy_bucket,
            warmth_bucket,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for EmoVoiceStyleProfile {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1EMOCORE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "emo_voice_style_profile.schema_version",
                reason: "must match PH1EMOCORE_CONTRACT_VERSION",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmoToneGuidance {
    pub schema_version: SchemaVersion,
    pub style_profile_ref: StyleProfileRef,
    pub modifiers: Vec<StyleModifier>,
    pub pacing_guidance: EmoTonePacing,
    pub directness_level: u8,
    pub empathy_level: u8,
}

impl EmoToneGuidance {
    pub fn v1(
        style_profile_ref: StyleProfileRef,
        modifiers: Vec<StyleModifier>,
        pacing_guidance: EmoTonePacing,
        directness_level: u8,
        empathy_level: u8,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            schema_version: PH1EMOCORE_CONTRACT_VERSION,
            style_profile_ref,
            modifiers,
            pacing_guidance,
            directness_level,
            empathy_level,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for EmoToneGuidance {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1EMOCORE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "emo_tone_guidance.schema_version",
                reason: "must match PH1EMOCORE_CONTRACT_VERSION",
            });
        }
        if self.modifiers.len() > 3 {
            return Err(ContractViolation::InvalidValue {
                field: "emo_tone_guidance.modifiers",
                reason: "must include <= 3 modifiers",
            });
        }
        let mut prev_rank: Option<u8> = None;
        for (idx, v) in self.modifiers.iter().enumerate() {
            if self.modifiers[..idx].contains(v) {
                return Err(ContractViolation::InvalidValue {
                    field: "emo_tone_guidance.modifiers",
                    reason: "must not contain duplicates",
                });
            }
            let rank = style_modifier_rank(*v);
            if let Some(prev) = prev_rank {
                if rank < prev {
                    return Err(ContractViolation::InvalidValue {
                        field: "emo_tone_guidance.modifiers",
                        reason: "must be sorted in canonical order",
                    });
                }
            }
            prev_rank = Some(rank);
        }
        if self.directness_level > 100 {
            return Err(ContractViolation::InvalidValue {
                field: "emo_tone_guidance.directness_level",
                reason: "must be <= 100",
            });
        }
        if self.empathy_level > 100 {
            return Err(ContractViolation::InvalidValue {
                field: "emo_tone_guidance.empathy_level",
                reason: "must be <= 100",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmoClassifyProfileCommitRequest {
    pub tenant_id: TenantId,
    pub requester_user_id: UserId,
    pub session_id: String,
    pub consent_asserted: bool,
    pub identity_verified: bool,
    pub signals: EmoSignalBundle,
    pub idempotency_key: String,
}

impl Validate for EmoClassifyProfileCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.tenant_id.validate()?;
        validate_token(
            "emo_classify.requester_user_id",
            self.requester_user_id.as_str(),
            128,
        )?;
        validate_token("emo_classify.session_id", &self.session_id, 128)?;
        self.signals.validate()?;
        validate_token("emo_classify.idempotency_key", &self.idempotency_key, 128)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmoReevaluateProfileCommitRequest {
    pub tenant_id: TenantId,
    pub requester_user_id: UserId,
    pub session_id: String,
    pub consent_asserted: bool,
    pub identity_verified: bool,
    pub signals_window_ref: String,
    pub idempotency_key: String,
}

impl Validate for EmoReevaluateProfileCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.tenant_id.validate()?;
        validate_token(
            "emo_reevaluate.requester_user_id",
            self.requester_user_id.as_str(),
            128,
        )?;
        validate_token("emo_reevaluate.session_id", &self.session_id, 128)?;
        validate_token(
            "emo_reevaluate.signals_window_ref",
            &self.signals_window_ref,
            128,
        )?;
        validate_token("emo_reevaluate.idempotency_key", &self.idempotency_key, 128)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmoPrivacyCommandCommitRequest {
    pub tenant_id: TenantId,
    pub requester_user_id: UserId,
    pub session_id: String,
    pub identity_verified: bool,
    pub privacy_command: EmoPrivacyCommand,
    pub target_key: Option<String>,
    pub confirmation_asserted: bool,
    pub idempotency_key: String,
}

impl Validate for EmoPrivacyCommandCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.tenant_id.validate()?;
        validate_token(
            "emo_privacy.requester_user_id",
            self.requester_user_id.as_str(),
            128,
        )?;
        validate_token("emo_privacy.session_id", &self.session_id, 128)?;
        validate_opt_token("emo_privacy.target_key", &self.target_key, 128)?;
        validate_token("emo_privacy.idempotency_key", &self.idempotency_key, 128)?;
        if self.privacy_command == EmoPrivacyCommand::ForgetThisKey && self.target_key.is_none() {
            return Err(ContractViolation::InvalidValue {
                field: "emo_privacy.target_key",
                reason: "target_key is required for FORGET_THIS_KEY",
            });
        }
        if self.privacy_command != EmoPrivacyCommand::ForgetThisKey && self.target_key.is_some() {
            return Err(ContractViolation::InvalidValue {
                field: "emo_privacy.target_key",
                reason: "target_key is only valid for FORGET_THIS_KEY",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmoToneGuidanceDraftRequest {
    pub tenant_id: TenantId,
    pub requester_user_id: Option<UserId>,
    pub profile_snapshot_ref: Option<String>,
    pub signals: EmoSignalBundle,
}

impl Validate for EmoToneGuidanceDraftRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.tenant_id.validate()?;
        if let Some(v) = &self.requester_user_id {
            validate_token("emo_tone_guidance.requester_user_id", v.as_str(), 128)?;
        }
        validate_opt_token(
            "emo_tone_guidance.profile_snapshot_ref",
            &self.profile_snapshot_ref,
            128,
        )?;
        self.signals.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmoSnapshotCaptureCommitRequest {
    pub tenant_id: TenantId,
    pub requester_user_id: UserId,
    pub onboarding_session_id: OnboardingSessionId,
    pub consent_asserted: bool,
    pub identity_verified: bool,
    pub signals: EmoSignalBundle,
    pub idempotency_key: String,
}

impl Validate for EmoSnapshotCaptureCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.tenant_id.validate()?;
        validate_token(
            "emo_snapshot.requester_user_id",
            self.requester_user_id.as_str(),
            128,
        )?;
        self.onboarding_session_id.validate()?;
        self.signals.validate()?;
        validate_token("emo_snapshot.idempotency_key", &self.idempotency_key, 128)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmoAuditEventCommitRequest {
    pub tenant_id: TenantId,
    pub requester_user_id: UserId,
    pub session_id: Option<String>,
    pub event_type: String,
    pub reason_codes: Vec<ReasonCodeId>,
    pub idempotency_key: String,
}

impl Validate for EmoAuditEventCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.tenant_id.validate()?;
        validate_token(
            "emo_audit.requester_user_id",
            self.requester_user_id.as_str(),
            128,
        )?;
        validate_opt_token("emo_audit.session_id", &self.session_id, 128)?;
        validate_token("emo_audit.event_type", &self.event_type, 64)?;
        if self.reason_codes.is_empty() || self.reason_codes.len() > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "emo_audit.reason_codes",
                reason: "must include 1..=8 reason codes",
            });
        }
        for code in &self.reason_codes {
            if code.0 == 0 {
                return Err(ContractViolation::InvalidValue {
                    field: "emo_audit.reason_codes",
                    reason: "reason codes must be > 0",
                });
            }
        }
        validate_token("emo_audit.idempotency_key", &self.idempotency_key, 128)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmoCoreRequest {
    ClassifyProfileCommit(EmoClassifyProfileCommitRequest),
    ReevaluateProfileCommit(EmoReevaluateProfileCommitRequest),
    PrivacyCommandCommit(EmoPrivacyCommandCommitRequest),
    ToneGuidanceDraft(EmoToneGuidanceDraftRequest),
    SnapshotCaptureCommit(EmoSnapshotCaptureCommitRequest),
    AuditEventCommit(EmoAuditEventCommitRequest),
}

impl EmoCoreRequest {
    pub fn simulation_id(&self) -> &'static str {
        match self {
            EmoCoreRequest::ClassifyProfileCommit(_) => EMO_SIM_001,
            EmoCoreRequest::ReevaluateProfileCommit(_) => EMO_SIM_002,
            EmoCoreRequest::PrivacyCommandCommit(_) => EMO_SIM_003,
            EmoCoreRequest::ToneGuidanceDraft(_) => EMO_SIM_004,
            EmoCoreRequest::SnapshotCaptureCommit(_) => EMO_SIM_005,
            EmoCoreRequest::AuditEventCommit(_) => EMO_SIM_006,
        }
    }

    pub fn simulation_type(&self) -> EmoCoreSimulationType {
        match self {
            EmoCoreRequest::ToneGuidanceDraft(_) => EmoCoreSimulationType::Draft,
            _ => EmoCoreSimulationType::Commit,
        }
    }

    pub fn capability_id(&self) -> EmoCoreCapabilityId {
        match self {
            EmoCoreRequest::ClassifyProfileCommit(_) => EmoCoreCapabilityId::ClassifyProfileCommit,
            EmoCoreRequest::ReevaluateProfileCommit(_) => {
                EmoCoreCapabilityId::ReevaluateProfileCommit
            }
            EmoCoreRequest::PrivacyCommandCommit(_) => EmoCoreCapabilityId::PrivacyCommandCommit,
            EmoCoreRequest::ToneGuidanceDraft(_) => EmoCoreCapabilityId::ToneGuidanceDraft,
            EmoCoreRequest::SnapshotCaptureCommit(_) => EmoCoreCapabilityId::SnapshotCaptureCommit,
            EmoCoreRequest::AuditEventCommit(_) => EmoCoreCapabilityId::AuditEventCommit,
        }
    }
}

impl Validate for EmoCoreRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            EmoCoreRequest::ClassifyProfileCommit(v) => v.validate(),
            EmoCoreRequest::ReevaluateProfileCommit(v) => v.validate(),
            EmoCoreRequest::PrivacyCommandCommit(v) => v.validate(),
            EmoCoreRequest::ToneGuidanceDraft(v) => v.validate(),
            EmoCoreRequest::SnapshotCaptureCommit(v) => v.validate(),
            EmoCoreRequest::AuditEventCommit(v) => v.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1EmoCoreRequest {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub now: MonotonicTimeNs,
    pub simulation_id: String,
    pub simulation_type: EmoCoreSimulationType,
    pub request: EmoCoreRequest,
}

impl Validate for Ph1EmoCoreRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1EMOCORE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1emo_core_request.schema_version",
                reason: "must match PH1EMOCORE_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.now.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1emo_core_request.now",
                reason: "must be > 0",
            });
        }
        self.request.validate()?;
        if self.simulation_id != self.request.simulation_id() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1emo_core_request.simulation_id",
                reason: "must match request variant simulation_id",
            });
        }
        if self.simulation_type != self.request.simulation_type() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1emo_core_request.simulation_type",
                reason: "must match request variant simulation_type",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmoClassifyProfileResult {
    pub requester_user_id: UserId,
    pub personality_type: EmoPersonalityType,
    pub personality_lock_status: EmoPersonalityLockStatus,
    pub voice_style_profile: EmoVoiceStyleProfile,
    pub reason_code: ReasonCodeId,
}

impl Validate for EmoClassifyProfileResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_token(
            "emo_classify_profile_result.requester_user_id",
            self.requester_user_id.as_str(),
            128,
        )?;
        self.voice_style_profile.validate()?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "emo_classify_profile_result.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmoReevaluateProfileResult {
    pub requester_user_id: UserId,
    pub personality_type: EmoPersonalityType,
    pub personality_lock_status: EmoPersonalityLockStatus,
    pub reason_code: ReasonCodeId,
}

impl Validate for EmoReevaluateProfileResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_token(
            "emo_reevaluate_profile_result.requester_user_id",
            self.requester_user_id.as_str(),
            128,
        )?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "emo_reevaluate_profile_result.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmoPrivacyCommandResult {
    pub requester_user_id: UserId,
    pub privacy_state: EmoPrivacyState,
    pub reason_code: ReasonCodeId,
}

impl Validate for EmoPrivacyCommandResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_token(
            "emo_privacy_command_result.requester_user_id",
            self.requester_user_id.as_str(),
            128,
        )?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "emo_privacy_command_result.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmoToneGuidanceResult {
    pub requester_user_id: Option<UserId>,
    pub tone_guidance: EmoToneGuidance,
    pub reason_code: ReasonCodeId,
}

impl Validate for EmoToneGuidanceResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if let Some(v) = &self.requester_user_id {
            validate_token(
                "emo_tone_guidance_result.requester_user_id",
                v.as_str(),
                128,
            )?;
        }
        self.tone_guidance.validate()?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "emo_tone_guidance_result.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmoSnapshotCaptureResult {
    pub requester_user_id: UserId,
    pub snapshot_ref: Option<String>,
    pub snapshot_status: EmoSnapshotStatus,
    pub reason_code: ReasonCodeId,
}

impl Validate for EmoSnapshotCaptureResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_token(
            "emo_snapshot_capture_result.requester_user_id",
            self.requester_user_id.as_str(),
            128,
        )?;
        validate_opt_token(
            "emo_snapshot_capture_result.snapshot_ref",
            &self.snapshot_ref,
            128,
        )?;
        if self.snapshot_status == EmoSnapshotStatus::Complete && self.snapshot_ref.is_none() {
            return Err(ContractViolation::InvalidValue {
                field: "emo_snapshot_capture_result.snapshot_ref",
                reason: "required when snapshot_status=Complete",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "emo_snapshot_capture_result.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmoAuditEventResult {
    pub event_id: String,
    pub status: EmoAuditEventStatus,
}

impl Validate for EmoAuditEventResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_token("emo_audit_event_result.event_id", &self.event_id, 128)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmoCoreOutcome {
    ClassifyProfile(EmoClassifyProfileResult),
    ReevaluateProfile(EmoReevaluateProfileResult),
    PrivacyCommand(EmoPrivacyCommandResult),
    ToneGuidance(EmoToneGuidanceResult),
    SnapshotCapture(EmoSnapshotCaptureResult),
    AuditEvent(EmoAuditEventResult),
}

impl EmoCoreOutcome {
    pub fn capability_id(&self) -> EmoCoreCapabilityId {
        match self {
            EmoCoreOutcome::ClassifyProfile(_) => EmoCoreCapabilityId::ClassifyProfileCommit,
            EmoCoreOutcome::ReevaluateProfile(_) => EmoCoreCapabilityId::ReevaluateProfileCommit,
            EmoCoreOutcome::PrivacyCommand(_) => EmoCoreCapabilityId::PrivacyCommandCommit,
            EmoCoreOutcome::ToneGuidance(_) => EmoCoreCapabilityId::ToneGuidanceDraft,
            EmoCoreOutcome::SnapshotCapture(_) => EmoCoreCapabilityId::SnapshotCaptureCommit,
            EmoCoreOutcome::AuditEvent(_) => EmoCoreCapabilityId::AuditEventCommit,
        }
    }
}

impl Validate for EmoCoreOutcome {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            EmoCoreOutcome::ClassifyProfile(v) => v.validate(),
            EmoCoreOutcome::ReevaluateProfile(v) => v.validate(),
            EmoCoreOutcome::PrivacyCommand(v) => v.validate(),
            EmoCoreOutcome::ToneGuidance(v) => v.validate(),
            EmoCoreOutcome::SnapshotCapture(v) => v.validate(),
            EmoCoreOutcome::AuditEvent(v) => v.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1EmoCoreOk {
    pub schema_version: SchemaVersion,
    pub engine_id: String,
    pub capability_id: EmoCoreCapabilityId,
    pub simulation_id: String,
    pub reason_code: ReasonCodeId,
    pub outcome: EmoCoreOutcome,
    pub tone_only: bool,
    pub no_meaning_drift: bool,
    pub no_execution_authority: bool,
}

impl Ph1EmoCoreOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        capability_id: EmoCoreCapabilityId,
        simulation_id: String,
        reason_code: ReasonCodeId,
        outcome: EmoCoreOutcome,
        tone_only: bool,
        no_meaning_drift: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            schema_version: PH1EMOCORE_CONTRACT_VERSION,
            engine_id: PH1EMOCORE_ENGINE_ID.to_string(),
            capability_id,
            simulation_id,
            reason_code,
            outcome,
            tone_only,
            no_meaning_drift,
            no_execution_authority,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for Ph1EmoCoreOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1EMOCORE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1emo_core_ok.schema_version",
                reason: "must match PH1EMOCORE_CONTRACT_VERSION",
            });
        }
        if self.engine_id != PH1EMOCORE_ENGINE_ID {
            return Err(ContractViolation::InvalidValue {
                field: "ph1emo_core_ok.engine_id",
                reason: "must match PH1EMOCORE_ENGINE_ID",
            });
        }
        validate_token("ph1emo_core_ok.simulation_id", &self.simulation_id, 64)?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1emo_core_ok.reason_code",
                reason: "must be > 0",
            });
        }
        self.outcome.validate()?;
        if self.capability_id != self.outcome.capability_id() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1emo_core_ok.capability_id",
                reason: "must match outcome capability",
            });
        }
        if !self.tone_only || !self.no_meaning_drift || !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "ph1emo_core_ok",
                reason: "tone/no-drift/no-execution flags must all be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1EmoCoreRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: EmoCoreCapabilityId,
    pub simulation_id: String,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl Ph1EmoCoreRefuse {
    pub fn v1(
        capability_id: EmoCoreCapabilityId,
        simulation_id: String,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            schema_version: PH1EMOCORE_CONTRACT_VERSION,
            capability_id,
            simulation_id,
            reason_code,
            message,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for Ph1EmoCoreRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1EMOCORE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1emo_core_refuse.schema_version",
                reason: "must match PH1EMOCORE_CONTRACT_VERSION",
            });
        }
        validate_token("ph1emo_core_refuse.simulation_id", &self.simulation_id, 64)?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1emo_core_refuse.reason_code",
                reason: "must be > 0",
            });
        }
        validate_token("ph1emo_core_refuse.message", &self.message, 256)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1EmoCoreResponse {
    Ok(Ph1EmoCoreOk),
    Refuse(Ph1EmoCoreRefuse),
}

impl Validate for Ph1EmoCoreResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1EmoCoreResponse::Ok(v) => v.validate(),
            Ph1EmoCoreResponse::Refuse(v) => v.validate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ph1j::{CorrelationId, TurnId};

    fn tenant() -> TenantId {
        TenantId::new("tenant_emo").unwrap()
    }

    fn user() -> UserId {
        UserId::new("user_emo").unwrap()
    }

    fn signals() -> EmoSignalBundle {
        EmoSignalBundle::v1(40, 20, 15, 70).unwrap()
    }

    #[test]
    fn at_emo_core_contract_01_simulation_id_must_match_variant() {
        let req = Ph1EmoCoreRequest {
            schema_version: PH1EMOCORE_CONTRACT_VERSION,
            correlation_id: CorrelationId(1001),
            turn_id: TurnId(21),
            now: MonotonicTimeNs(99),
            simulation_id: EMO_SIM_002.to_string(),
            simulation_type: EmoCoreSimulationType::Commit,
            request: EmoCoreRequest::ClassifyProfileCommit(EmoClassifyProfileCommitRequest {
                tenant_id: tenant(),
                requester_user_id: user(),
                session_id: "session_emo".to_string(),
                consent_asserted: true,
                identity_verified: true,
                signals: signals(),
                idempotency_key: "idem_emo_001".to_string(),
            }),
        };
        assert!(matches!(
            req.validate(),
            Err(ContractViolation::InvalidValue {
                field: "ph1emo_core_request.simulation_id",
                ..
            })
        ));
    }

    #[test]
    fn at_emo_core_contract_02_complete_snapshot_requires_ref() {
        let out = EmoSnapshotCaptureResult {
            requester_user_id: user(),
            snapshot_ref: None,
            snapshot_status: EmoSnapshotStatus::Complete,
            reason_code: ReasonCodeId(1),
        };
        assert!(matches!(
            out.validate(),
            Err(ContractViolation::InvalidValue {
                field: "emo_snapshot_capture_result.snapshot_ref",
                ..
            })
        ));
    }

    #[test]
    fn at_emo_core_contract_03_ok_requires_tone_flags_true() {
        let out = Ph1EmoCoreOk::v1(
            EmoCoreCapabilityId::ToneGuidanceDraft,
            EMO_SIM_004.to_string(),
            ReasonCodeId(1),
            EmoCoreOutcome::ToneGuidance(EmoToneGuidanceResult {
                requester_user_id: Some(user()),
                tone_guidance: EmoToneGuidance::v1(
                    StyleProfileRef::Gentle,
                    vec![StyleModifier::Warm],
                    EmoTonePacing::Balanced,
                    30,
                    80,
                )
                .unwrap(),
                reason_code: ReasonCodeId(2),
            }),
            true,
            true,
            false,
        );
        assert!(matches!(
            out,
            Err(ContractViolation::InvalidValue {
                field: "ph1emo_core_ok",
                ..
            })
        ));
    }

    #[test]
    fn at_emo_core_contract_04_forget_this_key_requires_target_key() {
        let req = EmoPrivacyCommandCommitRequest {
            tenant_id: tenant(),
            requester_user_id: user(),
            session_id: "session_emo".to_string(),
            identity_verified: true,
            privacy_command: EmoPrivacyCommand::ForgetThisKey,
            target_key: None,
            confirmation_asserted: true,
            idempotency_key: "idem_emo_004".to_string(),
        };
        assert!(matches!(
            req.validate(),
            Err(ContractViolation::InvalidValue {
                field: "emo_privacy.target_key",
                ..
            })
        ));
    }
}
