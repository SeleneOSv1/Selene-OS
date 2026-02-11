#![forbid(unsafe_code)]

use crate::ph1_voice_id::UserId;
use crate::ph1j::{CorrelationId, DeviceId, TurnId};
use crate::ph1k::{AudioStreamId, Confidence, PreRollBufferId};
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1W_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

pub use crate::SessionState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WakePolicyContext {
    pub schema_version: SchemaVersion,
    pub session_state: SessionState,
    pub do_not_disturb: bool,
    pub privacy_mode: bool,
    pub tts_playback_active: bool,
}

impl WakePolicyContext {
    pub fn v1(
        session_state: SessionState,
        do_not_disturb: bool,
        privacy_mode: bool,
        tts_playback_active: bool,
    ) -> Self {
        Self {
            schema_version: PH1W_CONTRACT_VERSION,
            session_state,
            do_not_disturb,
            privacy_mode,
            tts_playback_active,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserWakeProfileId(String);

impl UserWakeProfileId {
    pub fn new(id: impl Into<String>) -> Result<Self, ContractViolation> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "user_wake_profile_id",
                reason: "must not be empty",
            });
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WakeThresholds {
    pub light_min: Confidence,
    pub strong_min: Confidence,
    pub strong_stability_frames: u8,
    pub speaker_similarity_min: Confidence,
}

impl Validate for WakeThresholds {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.strong_stability_frames == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "wake_thresholds.strong_stability_frames",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WakeArtifactPackage {
    pub schema_version: SchemaVersion,
    pub artifact_version: SchemaVersion,
    pub user_wake_profile_id: UserWakeProfileId,
    pub wake_phrase_variants: Vec<String>,
    pub thresholds: WakeThresholds,
    pub device_calibration_hint: Option<String>,
}

impl WakeArtifactPackage {
    pub fn v1(
        artifact_version: SchemaVersion,
        user_wake_profile_id: UserWakeProfileId,
        wake_phrase_variants: Vec<String>,
        thresholds: WakeThresholds,
        device_calibration_hint: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let pkg = Self {
            schema_version: PH1W_CONTRACT_VERSION,
            artifact_version,
            user_wake_profile_id,
            wake_phrase_variants,
            thresholds,
            device_calibration_hint,
        };
        pkg.validate()?;
        Ok(pkg)
    }
}

impl Validate for WakeArtifactPackage {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.thresholds.validate()?;
        if self.wake_phrase_variants.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "wake_artifact_package.wake_phrase_variants",
                reason: "must not be empty",
            });
        }
        for (i, v) in self.wake_phrase_variants.iter().enumerate() {
            if v.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "wake_artifact_package.wake_phrase_variants[]",
                    reason: "must not contain empty strings",
                });
            }
            if v.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "wake_artifact_package.wake_phrase_variants[]",
                    reason: "must be <= 128 chars",
                });
            }
            // Keep the validation deterministic; don't do locale-aware transforms here.
            let _ = i;
        }
        if let Some(h) = &self.device_calibration_hint {
            if h.len() > 256 {
                return Err(ContractViolation::InvalidValue {
                    field: "wake_artifact_package.device_calibration_hint",
                    reason: "must be <= 256 chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WakeGateResults {
    pub g0_integrity_ok: bool,
    pub g1_activity_ok: bool,
    pub g2_light_ok: bool,
    pub g3_strong_ok: bool,
    pub g4_personalization_ok: bool,
    pub g5_policy_ok: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoundedAudioSegmentRef {
    pub schema_version: SchemaVersion,
    pub stream_id: AudioStreamId,
    pub source_pre_roll_buffer_id: PreRollBufferId,
    pub t_start: MonotonicTimeNs,
    pub t_end: MonotonicTimeNs,
    pub t_candidate_start: MonotonicTimeNs,
    pub t_confirmed: MonotonicTimeNs,
}

impl BoundedAudioSegmentRef {
    pub fn v1(
        stream_id: AudioStreamId,
        source_pre_roll_buffer_id: PreRollBufferId,
        t_start: MonotonicTimeNs,
        t_end: MonotonicTimeNs,
        t_candidate_start: MonotonicTimeNs,
        t_confirmed: MonotonicTimeNs,
    ) -> Result<Self, ContractViolation> {
        let seg = Self {
            schema_version: PH1W_CONTRACT_VERSION,
            stream_id,
            source_pre_roll_buffer_id,
            t_start,
            t_end,
            t_candidate_start,
            t_confirmed,
        };
        seg.validate()?;
        Ok(seg)
    }
}

impl Validate for BoundedAudioSegmentRef {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.t_end.0 < self.t_start.0 {
            return Err(ContractViolation::InvalidValue {
                field: "bounded_audio_segment_ref.t_end",
                reason: "must be >= t_start",
            });
        }
        if !(self.t_start.0..=self.t_end.0).contains(&self.t_candidate_start.0) {
            return Err(ContractViolation::InvalidValue {
                field: "bounded_audio_segment_ref.t_candidate_start",
                reason: "must be within [t_start, t_end]",
            });
        }
        if !(self.t_candidate_start.0..=self.t_end.0).contains(&self.t_confirmed.0) {
            return Err(ContractViolation::InvalidValue {
                field: "bounded_audio_segment_ref.t_confirmed",
                reason: "must be within [t_candidate_start, t_end]",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WakeDecision {
    pub schema_version: SchemaVersion,
    pub accepted: bool,
    pub reason_code: ReasonCodeId,
    pub gates: WakeGateResults,
    pub t_decision: MonotonicTimeNs,
    pub light_score: Option<Confidence>,
    pub strong_score: Option<Confidence>,
    pub capture: Option<BoundedAudioSegmentRef>,
}

impl WakeDecision {
    pub fn accept_v1(
        reason_code: ReasonCodeId,
        gates: WakeGateResults,
        t_decision: MonotonicTimeNs,
        light_score: Option<Confidence>,
        strong_score: Option<Confidence>,
        capture: BoundedAudioSegmentRef,
    ) -> Result<Self, ContractViolation> {
        let d = Self {
            schema_version: PH1W_CONTRACT_VERSION,
            accepted: true,
            reason_code,
            gates,
            t_decision,
            light_score,
            strong_score,
            capture: Some(capture),
        };
        d.validate()?;
        Ok(d)
    }

    pub fn reject_v1(
        reason_code: ReasonCodeId,
        gates: WakeGateResults,
        t_decision: MonotonicTimeNs,
        light_score: Option<Confidence>,
        strong_score: Option<Confidence>,
    ) -> Result<Self, ContractViolation> {
        let d = Self {
            schema_version: PH1W_CONTRACT_VERSION,
            accepted: false,
            reason_code,
            gates,
            t_decision,
            light_score,
            strong_score,
            capture: None,
        };
        d.validate()?;
        Ok(d)
    }
}

impl Validate for WakeDecision {
    fn validate(&self) -> Result<(), ContractViolation> {
        match (self.accepted, self.capture.is_some()) {
            (true, true) => {}
            (false, false) => {}
            (true, false) => {
                return Err(ContractViolation::InvalidValue {
                    field: "wake_decision.capture",
                    reason: "must be Some(...) when accepted=true",
                });
            }
            (false, true) => {
                return Err(ContractViolation::InvalidValue {
                    field: "wake_decision.capture",
                    reason: "must be None when accepted=false",
                });
            }
        }

        if let Some(c) = &self.capture {
            c.validate()?;
        }

        Ok(())
    }
}

// Simulation IDs (authoritative strings; must match docs/08_SIMULATION_CATALOG.md).
pub const WAKE_ENROLL_START_DRAFT: &str = "WAKE_ENROLL_START_DRAFT";
pub const WAKE_ENROLL_SAMPLE_COMMIT: &str = "WAKE_ENROLL_SAMPLE_COMMIT";
pub const WAKE_ENROLL_COMPLETE_COMMIT: &str = "WAKE_ENROLL_COMPLETE_COMMIT";
pub const WAKE_ENROLL_DEFER_REMINDER_COMMIT: &str = "WAKE_ENROLL_DEFER_REMINDER_COMMIT";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WakeSimulationType {
    Draft,
    Commit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WakeEnrollStatus {
    InProgress,
    Pending,
    Complete,
    Declined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WakeSampleResult {
    Pass,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WakeEnrollmentSessionId(String);

impl WakeEnrollmentSessionId {
    pub fn new(id: impl Into<String>) -> Result<Self, ContractViolation> {
        let id = id.into();
        let v = Self(id);
        v.validate()?;
        Ok(v)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for WakeEnrollmentSessionId {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "wake_enrollment_session_id",
                reason: "must not be empty",
            });
        }
        if self.0.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "wake_enrollment_session_id",
                reason: "must be <= 64 chars",
            });
        }
        if !self.0.is_ascii() {
            return Err(ContractViolation::InvalidValue {
                field: "wake_enrollment_session_id",
                reason: "must be ASCII",
            });
        }
        Ok(())
    }
}

fn validate_id(field: &'static str, s: &str, max_len: usize) -> Result<(), ContractViolation> {
    if s.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if s.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "too long",
        });
    }
    if !s.is_ascii() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be ASCII",
        });
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeEnrollStartDraftRequest {
    pub user_id: UserId,
    pub device_id: DeviceId,
    pub onboarding_session_id: Option<String>,
    pub pass_target: u8,
    pub max_attempts: u8,
    pub enrollment_timeout_ms: u32,
    pub idempotency_key: String,
}

impl Validate for WakeEnrollStartDraftRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id(
            "wake_enroll_start_draft_request.user_id",
            self.user_id.as_str(),
            128,
        )?;
        self.device_id.validate()?;
        if let Some(v) = &self.onboarding_session_id {
            validate_id(
                "wake_enroll_start_draft_request.onboarding_session_id",
                v,
                64,
            )?;
        }
        if !(3..=8).contains(&self.pass_target) {
            return Err(ContractViolation::InvalidValue {
                field: "wake_enroll_start_draft_request.pass_target",
                reason: "must be in [3, 8]",
            });
        }
        if !(8..=20).contains(&self.max_attempts) {
            return Err(ContractViolation::InvalidValue {
                field: "wake_enroll_start_draft_request.max_attempts",
                reason: "must be in [8, 20]",
            });
        }
        if !(180_000..=600_000).contains(&self.enrollment_timeout_ms) {
            return Err(ContractViolation::InvalidValue {
                field: "wake_enroll_start_draft_request.enrollment_timeout_ms",
                reason: "must be in [180000, 600000]",
            });
        }
        validate_id(
            "wake_enroll_start_draft_request.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WakeEnrollSampleCommitRequest {
    pub wake_enrollment_session_id: WakeEnrollmentSessionId,
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

impl Validate for WakeEnrollSampleCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.wake_enrollment_session_id.validate()?;
        if !(500..=2200).contains(&self.sample_duration_ms) {
            return Err(ContractViolation::InvalidValue {
                field: "wake_enroll_sample_commit_request.sample_duration_ms",
                reason: "must be in [500, 2200]",
            });
        }
        if !(0.0..=1.0).contains(&self.vad_coverage) {
            return Err(ContractViolation::InvalidValue {
                field: "wake_enroll_sample_commit_request.vad_coverage",
                reason: "must be in [0, 1]",
            });
        }
        if !(0.0..=100.0).contains(&self.clipping_pct) {
            return Err(ContractViolation::InvalidValue {
                field: "wake_enroll_sample_commit_request.clipping_pct",
                reason: "must be in [0, 100]",
            });
        }
        if !(0.0..=1.0).contains(&self.overlap_ratio) {
            return Err(ContractViolation::InvalidValue {
                field: "wake_enroll_sample_commit_request.overlap_ratio",
                reason: "must be in [0, 1]",
            });
        }
        validate_id(
            "wake_enroll_sample_commit_request.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeEnrollCompleteCommitRequest {
    pub wake_enrollment_session_id: WakeEnrollmentSessionId,
    pub wake_profile_id: String,
    pub idempotency_key: String,
}

impl Validate for WakeEnrollCompleteCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.wake_enrollment_session_id.validate()?;
        validate_id(
            "wake_enroll_complete_commit_request.wake_profile_id",
            &self.wake_profile_id,
            128,
        )?;
        validate_id(
            "wake_enroll_complete_commit_request.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeEnrollDeferReminderCommitRequest {
    pub wake_enrollment_session_id: WakeEnrollmentSessionId,
    pub deferred_until: Option<MonotonicTimeNs>,
    pub reason_code: ReasonCodeId,
    pub idempotency_key: String,
}

impl Validate for WakeEnrollDeferReminderCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.wake_enrollment_session_id.validate()?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "wake_enroll_defer_reminder_commit_request.reason_code",
                reason: "must be > 0",
            });
        }
        validate_id(
            "wake_enroll_defer_reminder_commit_request.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WakeRequest {
    EnrollStartDraft(WakeEnrollStartDraftRequest),
    EnrollSampleCommit(WakeEnrollSampleCommitRequest),
    EnrollCompleteCommit(WakeEnrollCompleteCommitRequest),
    EnrollDeferReminderCommit(WakeEnrollDeferReminderCommitRequest),
}

impl WakeRequest {
    pub fn simulation_id(&self) -> &'static str {
        match self {
            WakeRequest::EnrollStartDraft(_) => WAKE_ENROLL_START_DRAFT,
            WakeRequest::EnrollSampleCommit(_) => WAKE_ENROLL_SAMPLE_COMMIT,
            WakeRequest::EnrollCompleteCommit(_) => WAKE_ENROLL_COMPLETE_COMMIT,
            WakeRequest::EnrollDeferReminderCommit(_) => WAKE_ENROLL_DEFER_REMINDER_COMMIT,
        }
    }

    pub fn simulation_type(&self) -> WakeSimulationType {
        match self {
            WakeRequest::EnrollStartDraft(_) => WakeSimulationType::Draft,
            WakeRequest::EnrollSampleCommit(_)
            | WakeRequest::EnrollCompleteCommit(_)
            | WakeRequest::EnrollDeferReminderCommit(_) => WakeSimulationType::Commit,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1wRequest {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub now: MonotonicTimeNs,
    pub simulation_id: String,
    pub simulation_type: WakeSimulationType,
    pub request: WakeRequest,
}

impl Validate for Ph1wRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1W_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1w_request.schema_version",
                reason: "must match PH1W_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.now.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1w_request.now",
                reason: "must be > 0",
            });
        }
        if self.simulation_id != self.request.simulation_id() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1w_request.simulation_id",
                reason: "must match the request variant's simulation_id",
            });
        }
        if self.simulation_type != self.request.simulation_type() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1w_request.simulation_type",
                reason: "must match the request variant's simulation_type",
            });
        }
        match &self.request {
            WakeRequest::EnrollStartDraft(r) => r.validate(),
            WakeRequest::EnrollSampleCommit(r) => r.validate(),
            WakeRequest::EnrollCompleteCommit(r) => r.validate(),
            WakeRequest::EnrollDeferReminderCommit(r) => r.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeEnrollStartResult {
    pub schema_version: SchemaVersion,
    pub wake_enrollment_session_id: WakeEnrollmentSessionId,
    pub wake_enroll_status: WakeEnrollStatus,
    pub pass_target: u8,
    pub max_attempts: u8,
    pub enrollment_timeout_ms: u32,
}

impl WakeEnrollStartResult {
    pub fn v1(
        wake_enrollment_session_id: WakeEnrollmentSessionId,
        wake_enroll_status: WakeEnrollStatus,
        pass_target: u8,
        max_attempts: u8,
        enrollment_timeout_ms: u32,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1W_CONTRACT_VERSION,
            wake_enrollment_session_id,
            wake_enroll_status,
            pass_target,
            max_attempts,
            enrollment_timeout_ms,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for WakeEnrollStartResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1W_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "wake_enroll_start_result.schema_version",
                reason: "must match PH1W_CONTRACT_VERSION",
            });
        }
        self.wake_enrollment_session_id.validate()?;
        if !(3..=8).contains(&self.pass_target) {
            return Err(ContractViolation::InvalidValue {
                field: "wake_enroll_start_result.pass_target",
                reason: "must be in [3, 8]",
            });
        }
        if !(8..=20).contains(&self.max_attempts) {
            return Err(ContractViolation::InvalidValue {
                field: "wake_enroll_start_result.max_attempts",
                reason: "must be in [8, 20]",
            });
        }
        if !(180_000..=600_000).contains(&self.enrollment_timeout_ms) {
            return Err(ContractViolation::InvalidValue {
                field: "wake_enroll_start_result.enrollment_timeout_ms",
                reason: "must be in [180000, 600000]",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeEnrollSampleResult {
    pub schema_version: SchemaVersion,
    pub wake_enrollment_session_id: WakeEnrollmentSessionId,
    pub wake_enroll_status: WakeEnrollStatus,
    pub pass_count: u8,
    pub attempt_count: u8,
    pub reason_code: Option<ReasonCodeId>,
}

impl WakeEnrollSampleResult {
    pub fn v1(
        wake_enrollment_session_id: WakeEnrollmentSessionId,
        wake_enroll_status: WakeEnrollStatus,
        pass_count: u8,
        attempt_count: u8,
        reason_code: Option<ReasonCodeId>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1W_CONTRACT_VERSION,
            wake_enrollment_session_id,
            wake_enroll_status,
            pass_count,
            attempt_count,
            reason_code,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for WakeEnrollSampleResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1W_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "wake_enroll_sample_result.schema_version",
                reason: "must match PH1W_CONTRACT_VERSION",
            });
        }
        self.wake_enrollment_session_id.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeEnrollCompleteResult {
    pub schema_version: SchemaVersion,
    pub wake_enrollment_session_id: WakeEnrollmentSessionId,
    pub wake_enroll_status: WakeEnrollStatus,
    pub wake_profile_id: String,
}

impl WakeEnrollCompleteResult {
    pub fn v1(
        wake_enrollment_session_id: WakeEnrollmentSessionId,
        wake_enroll_status: WakeEnrollStatus,
        wake_profile_id: String,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1W_CONTRACT_VERSION,
            wake_enrollment_session_id,
            wake_enroll_status,
            wake_profile_id,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for WakeEnrollCompleteResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1W_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "wake_enroll_complete_result.schema_version",
                reason: "must match PH1W_CONTRACT_VERSION",
            });
        }
        self.wake_enrollment_session_id.validate()?;
        validate_id(
            "wake_enroll_complete_result.wake_profile_id",
            &self.wake_profile_id,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeEnrollDeferResult {
    pub schema_version: SchemaVersion,
    pub wake_enrollment_session_id: WakeEnrollmentSessionId,
    pub wake_enroll_status: WakeEnrollStatus,
    pub deferred_until: Option<MonotonicTimeNs>,
    pub reason_code: ReasonCodeId,
}

impl WakeEnrollDeferResult {
    pub fn v1(
        wake_enrollment_session_id: WakeEnrollmentSessionId,
        wake_enroll_status: WakeEnrollStatus,
        deferred_until: Option<MonotonicTimeNs>,
        reason_code: ReasonCodeId,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1W_CONTRACT_VERSION,
            wake_enrollment_session_id,
            wake_enroll_status,
            deferred_until,
            reason_code,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for WakeEnrollDeferResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1W_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "wake_enroll_defer_result.schema_version",
                reason: "must match PH1W_CONTRACT_VERSION",
            });
        }
        self.wake_enrollment_session_id.validate()?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "wake_enroll_defer_result.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1wOk {
    pub schema_version: SchemaVersion,
    pub simulation_id: String,
    pub reason_code: ReasonCodeId,
    pub enroll_start_result: Option<WakeEnrollStartResult>,
    pub enroll_sample_result: Option<WakeEnrollSampleResult>,
    pub enroll_complete_result: Option<WakeEnrollCompleteResult>,
    pub enroll_defer_result: Option<WakeEnrollDeferResult>,
}

impl Ph1wOk {
    pub fn v1(
        simulation_id: String,
        reason_code: ReasonCodeId,
        enroll_start_result: Option<WakeEnrollStartResult>,
        enroll_sample_result: Option<WakeEnrollSampleResult>,
        enroll_complete_result: Option<WakeEnrollCompleteResult>,
        enroll_defer_result: Option<WakeEnrollDeferResult>,
    ) -> Result<Self, ContractViolation> {
        let o = Self {
            schema_version: PH1W_CONTRACT_VERSION,
            simulation_id,
            reason_code,
            enroll_start_result,
            enroll_sample_result,
            enroll_complete_result,
            enroll_defer_result,
        };
        o.validate()?;
        Ok(o)
    }
}

impl Validate for Ph1wOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1W_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1w_ok.schema_version",
                reason: "must match PH1W_CONTRACT_VERSION",
            });
        }
        validate_id("ph1w_ok.simulation_id", &self.simulation_id, 96)?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1w_ok.reason_code",
                reason: "must be > 0",
            });
        }
        let mut count = 0u8;
        if let Some(r) = &self.enroll_start_result {
            r.validate()?;
            count += 1;
        }
        if let Some(r) = &self.enroll_sample_result {
            r.validate()?;
            count += 1;
        }
        if let Some(r) = &self.enroll_complete_result {
            r.validate()?;
            count += 1;
        }
        if let Some(r) = &self.enroll_defer_result {
            r.validate()?;
            count += 1;
        }
        if count != 1 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1w_ok",
                reason: "must contain exactly one result kind",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1wRefuse {
    pub schema_version: SchemaVersion,
    pub simulation_id: String,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl Ph1wRefuse {
    pub fn v1(
        simulation_id: String,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1W_CONTRACT_VERSION,
            simulation_id,
            reason_code,
            message,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1wRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1W_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1w_refuse.schema_version",
                reason: "must match PH1W_CONTRACT_VERSION",
            });
        }
        validate_id("ph1w_refuse.simulation_id", &self.simulation_id, 96)?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1w_refuse.reason_code",
                reason: "must be > 0",
            });
        }
        if self.message.trim().is_empty() || self.message.len() > 512 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1w_refuse.message",
                reason: "must be non-empty and <= 512 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1wResponse {
    Ok(Ph1wOk),
    Refuse(Ph1wRefuse),
}

impl Validate for Ph1wResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1wResponse::Ok(o) => o.validate(),
            Ph1wResponse::Refuse(r) => r.validate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ph1k::AudioStreamId;

    #[test]
    fn accepted_requires_capture() {
        let gates = WakeGateResults {
            g0_integrity_ok: true,
            g1_activity_ok: true,
            g2_light_ok: true,
            g3_strong_ok: true,
            g4_personalization_ok: true,
            g5_policy_ok: true,
        };

        let seg = BoundedAudioSegmentRef::v1(
            AudioStreamId(1),
            PreRollBufferId(9),
            MonotonicTimeNs(100),
            MonotonicTimeNs(200),
            MonotonicTimeNs(120),
            MonotonicTimeNs(130),
        )
        .unwrap();

        let d = WakeDecision::accept_v1(
            ReasonCodeId(1),
            gates,
            MonotonicTimeNs(130),
            None,
            None,
            seg,
        )
        .unwrap();

        assert!(d.validate().is_ok());
    }

    #[test]
    fn segment_requires_non_negative_range_and_ordering() {
        let bad = BoundedAudioSegmentRef::v1(
            AudioStreamId(1),
            PreRollBufferId(9),
            MonotonicTimeNs(200),
            MonotonicTimeNs(100),
            MonotonicTimeNs(120),
            MonotonicTimeNs(130),
        );
        assert!(bad.is_err());
    }
}
