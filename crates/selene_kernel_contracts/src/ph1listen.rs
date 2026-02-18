#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1LISTEN_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ListenCapabilityId {
    ListenSignalCollect,
    ListenSignalFilter,
}

impl ListenCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            ListenCapabilityId::ListenSignalCollect => "LISTEN_SIGNAL_COLLECT",
            ListenCapabilityId::ListenSignalFilter => "LISTEN_SIGNAL_FILTER",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ListenEnvironmentMode {
    Quiet,
    Noisy,
    Meeting,
    Car,
    Office,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ListenCaptureProfile {
    Standard,
    NoiseSuppressed,
    NearFieldBoost,
    AggressiveAec,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ListenEndpointProfile {
    EarlyClose,
    Balanced,
    ExtendTail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ListenDeliveryPolicyHint {
    VoicePreferred,
    TextPreferred,
    TextOnlyMeeting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ListenValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListenRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_signal_windows: u8,
    pub max_adjustments: u8,
    pub max_diagnostics: u8,
}

impl ListenRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_signal_windows: u8,
        max_adjustments: u8,
        max_diagnostics: u8,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1LISTEN_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_signal_windows,
            max_adjustments,
            max_diagnostics,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for ListenRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LISTEN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "listen_request_envelope.schema_version",
                reason: "must match PH1LISTEN_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_signal_windows == 0 || self.max_signal_windows > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "listen_request_envelope.max_signal_windows",
                reason: "must be within 1..=64",
            });
        }
        if self.max_adjustments == 0 || self.max_adjustments > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "listen_request_envelope.max_adjustments",
                reason: "must be within 1..=32",
            });
        }
        if self.max_diagnostics == 0 || self.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "listen_request_envelope.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListenSignalWindow {
    pub schema_version: SchemaVersion,
    pub window_id: String,
    pub source_engine: String,
    pub vad_confidence_bp: u16,
    pub speech_likeness_bp: u16,
    pub noise_level_dbfs: i16,
    pub overlap_tts_ms: u16,
    pub trailing_silence_ms: u16,
    pub evidence_ref: String,
}

impl ListenSignalWindow {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        window_id: String,
        source_engine: String,
        vad_confidence_bp: u16,
        speech_likeness_bp: u16,
        noise_level_dbfs: i16,
        overlap_tts_ms: u16,
        trailing_silence_ms: u16,
        evidence_ref: String,
    ) -> Result<Self, ContractViolation> {
        let window = Self {
            schema_version: PH1LISTEN_CONTRACT_VERSION,
            window_id,
            source_engine,
            vad_confidence_bp,
            speech_likeness_bp,
            noise_level_dbfs,
            overlap_tts_ms,
            trailing_silence_ms,
            evidence_ref,
        };
        window.validate()?;
        Ok(window)
    }
}

impl Validate for ListenSignalWindow {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LISTEN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_window.schema_version",
                reason: "must match PH1LISTEN_CONTRACT_VERSION",
            });
        }
        validate_token("listen_signal_window.window_id", &self.window_id, 64)?;
        validate_engine_id("listen_signal_window.source_engine", &self.source_engine)?;
        if self.vad_confidence_bp > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_window.vad_confidence_bp",
                reason: "must be <= 10000",
            });
        }
        if self.speech_likeness_bp > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_window.speech_likeness_bp",
                reason: "must be <= 10000",
            });
        }
        if !(-100..=0).contains(&self.noise_level_dbfs) {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_window.noise_level_dbfs",
                reason: "must be within -100..=0",
            });
        }
        if self.overlap_tts_ms > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_window.overlap_tts_ms",
                reason: "must be <= 10000",
            });
        }
        if self.trailing_silence_ms > 30_000 {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_window.trailing_silence_ms",
                reason: "must be <= 30000",
            });
        }
        validate_token("listen_signal_window.evidence_ref", &self.evidence_ref, 128)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListenCorrectionSnapshot {
    pub schema_version: SchemaVersion,
    pub user_correction_count: u16,
    pub delivery_switch_count: u16,
    pub barge_in_count: u16,
    pub correction_rate_bp: u16,
}

impl ListenCorrectionSnapshot {
    pub fn v1(
        user_correction_count: u16,
        delivery_switch_count: u16,
        barge_in_count: u16,
        correction_rate_bp: u16,
    ) -> Result<Self, ContractViolation> {
        let snapshot = Self {
            schema_version: PH1LISTEN_CONTRACT_VERSION,
            user_correction_count,
            delivery_switch_count,
            barge_in_count,
            correction_rate_bp,
        };
        snapshot.validate()?;
        Ok(snapshot)
    }
}

impl Validate for ListenCorrectionSnapshot {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LISTEN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "listen_correction_snapshot.schema_version",
                reason: "must match PH1LISTEN_CONTRACT_VERSION",
            });
        }
        if self.user_correction_count > 4_000 {
            return Err(ContractViolation::InvalidValue {
                field: "listen_correction_snapshot.user_correction_count",
                reason: "must be <= 4000",
            });
        }
        if self.delivery_switch_count > 4_000 {
            return Err(ContractViolation::InvalidValue {
                field: "listen_correction_snapshot.delivery_switch_count",
                reason: "must be <= 4000",
            });
        }
        if self.barge_in_count > 4_000 {
            return Err(ContractViolation::InvalidValue {
                field: "listen_correction_snapshot.barge_in_count",
                reason: "must be <= 4000",
            });
        }
        if self.correction_rate_bp > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "listen_correction_snapshot.correction_rate_bp",
                reason: "must be <= 10000",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListenSessionContext {
    pub schema_version: SchemaVersion,
    pub session_mode_meeting: bool,
    pub session_mode_car: bool,
    pub privacy_mode: bool,
    pub text_preferred: bool,
}

impl ListenSessionContext {
    pub fn v1(
        session_mode_meeting: bool,
        session_mode_car: bool,
        privacy_mode: bool,
        text_preferred: bool,
    ) -> Result<Self, ContractViolation> {
        let context = Self {
            schema_version: PH1LISTEN_CONTRACT_VERSION,
            session_mode_meeting,
            session_mode_car,
            privacy_mode,
            text_preferred,
        };
        context.validate()?;
        Ok(context)
    }
}

impl Validate for ListenSessionContext {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LISTEN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "listen_session_context.schema_version",
                reason: "must match PH1LISTEN_CONTRACT_VERSION",
            });
        }
        if self.session_mode_meeting && self.session_mode_car {
            return Err(ContractViolation::InvalidValue {
                field: "listen_session_context",
                reason: "meeting and car session modes cannot both be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListenAdjustmentHint {
    pub schema_version: SchemaVersion,
    pub adjustment_id: String,
    pub environment_mode: ListenEnvironmentMode,
    pub capture_profile: ListenCaptureProfile,
    pub endpoint_profile: ListenEndpointProfile,
    pub delivery_policy_hint: ListenDeliveryPolicyHint,
    pub priority_bp: i16,
    pub source_window_id: String,
    pub evidence_ref: String,
}

impl ListenAdjustmentHint {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        adjustment_id: String,
        environment_mode: ListenEnvironmentMode,
        capture_profile: ListenCaptureProfile,
        endpoint_profile: ListenEndpointProfile,
        delivery_policy_hint: ListenDeliveryPolicyHint,
        priority_bp: i16,
        source_window_id: String,
        evidence_ref: String,
    ) -> Result<Self, ContractViolation> {
        let hint = Self {
            schema_version: PH1LISTEN_CONTRACT_VERSION,
            adjustment_id,
            environment_mode,
            capture_profile,
            endpoint_profile,
            delivery_policy_hint,
            priority_bp,
            source_window_id,
            evidence_ref,
        };
        hint.validate()?;
        Ok(hint)
    }
}

impl Validate for ListenAdjustmentHint {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LISTEN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "listen_adjustment_hint.schema_version",
                reason: "must match PH1LISTEN_CONTRACT_VERSION",
            });
        }
        validate_token(
            "listen_adjustment_hint.adjustment_id",
            &self.adjustment_id,
            64,
        )?;
        if !(-20_000..=20_000).contains(&self.priority_bp) {
            return Err(ContractViolation::InvalidValue {
                field: "listen_adjustment_hint.priority_bp",
                reason: "must be within -20000..=20000",
            });
        }
        validate_token(
            "listen_adjustment_hint.source_window_id",
            &self.source_window_id,
            64,
        )?;
        validate_token(
            "listen_adjustment_hint.evidence_ref",
            &self.evidence_ref,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListenSignalCollectRequest {
    pub schema_version: SchemaVersion,
    pub envelope: ListenRequestEnvelope,
    pub signal_windows: Vec<ListenSignalWindow>,
    pub correction_snapshot: ListenCorrectionSnapshot,
    pub session_context: ListenSessionContext,
}

impl ListenSignalCollectRequest {
    pub fn v1(
        envelope: ListenRequestEnvelope,
        signal_windows: Vec<ListenSignalWindow>,
        correction_snapshot: ListenCorrectionSnapshot,
        session_context: ListenSessionContext,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1LISTEN_CONTRACT_VERSION,
            envelope,
            signal_windows,
            correction_snapshot,
            session_context,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for ListenSignalCollectRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LISTEN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_collect_request.schema_version",
                reason: "must match PH1LISTEN_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        self.correction_snapshot.validate()?;
        self.session_context.validate()?;
        if self.signal_windows.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_collect_request.signal_windows",
                reason: "must be non-empty",
            });
        }
        if self.signal_windows.len() > self.envelope.max_signal_windows as usize {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_collect_request.signal_windows",
                reason: "must be <= envelope.max_signal_windows",
            });
        }
        let mut seen_ids = BTreeSet::new();
        for window in &self.signal_windows {
            window.validate()?;
            if !seen_ids.insert(window.window_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "listen_signal_collect_request.signal_windows",
                    reason: "window_id must be unique",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListenSignalFilterRequest {
    pub schema_version: SchemaVersion,
    pub envelope: ListenRequestEnvelope,
    pub environment_profile_ref: String,
    pub selected_adjustment_id: String,
    pub ordered_adjustments: Vec<ListenAdjustmentHint>,
    pub no_meaning_mutation_required: bool,
}

impl ListenSignalFilterRequest {
    pub fn v1(
        envelope: ListenRequestEnvelope,
        environment_profile_ref: String,
        selected_adjustment_id: String,
        ordered_adjustments: Vec<ListenAdjustmentHint>,
        no_meaning_mutation_required: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1LISTEN_CONTRACT_VERSION,
            envelope,
            environment_profile_ref,
            selected_adjustment_id,
            ordered_adjustments,
            no_meaning_mutation_required,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for ListenSignalFilterRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LISTEN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_filter_request.schema_version",
                reason: "must match PH1LISTEN_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_token(
            "listen_signal_filter_request.environment_profile_ref",
            &self.environment_profile_ref,
            128,
        )?;
        validate_token(
            "listen_signal_filter_request.selected_adjustment_id",
            &self.selected_adjustment_id,
            64,
        )?;
        if self.ordered_adjustments.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_filter_request.ordered_adjustments",
                reason: "must be non-empty",
            });
        }
        if self.ordered_adjustments.len() > self.envelope.max_adjustments as usize {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_filter_request.ordered_adjustments",
                reason: "must be <= envelope.max_adjustments",
            });
        }
        let mut seen = BTreeSet::new();
        for adjustment in &self.ordered_adjustments {
            adjustment.validate()?;
            if !seen.insert(adjustment.adjustment_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "listen_signal_filter_request.ordered_adjustments",
                    reason: "adjustment_id must be unique",
                });
            }
        }
        if !self
            .ordered_adjustments
            .iter()
            .any(|adjustment| adjustment.adjustment_id == self.selected_adjustment_id)
        {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_filter_request.selected_adjustment_id",
                reason: "must exist in ordered_adjustments",
            });
        }
        if !self.no_meaning_mutation_required {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_filter_request.no_meaning_mutation_required",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListenSignalCollectOk {
    pub schema_version: SchemaVersion,
    pub capability_id: ListenCapabilityId,
    pub reason_code: ReasonCodeId,
    pub environment_profile_ref: String,
    pub selected_adjustment_id: String,
    pub ordered_adjustments: Vec<ListenAdjustmentHint>,
    pub affects_capture_only: bool,
    pub affects_delivery_mode_only: bool,
    pub no_meaning_mutation: bool,
    pub advisory_only: bool,
    pub no_execution_authority: bool,
}

impl ListenSignalCollectOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        environment_profile_ref: String,
        selected_adjustment_id: String,
        ordered_adjustments: Vec<ListenAdjustmentHint>,
        affects_capture_only: bool,
        affects_delivery_mode_only: bool,
        no_meaning_mutation: bool,
        advisory_only: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1LISTEN_CONTRACT_VERSION,
            capability_id: ListenCapabilityId::ListenSignalCollect,
            reason_code,
            environment_profile_ref,
            selected_adjustment_id,
            ordered_adjustments,
            affects_capture_only,
            affects_delivery_mode_only,
            no_meaning_mutation,
            advisory_only,
            no_execution_authority,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for ListenSignalCollectOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LISTEN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_collect_ok.schema_version",
                reason: "must match PH1LISTEN_CONTRACT_VERSION",
            });
        }
        if self.capability_id != ListenCapabilityId::ListenSignalCollect {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_collect_ok.capability_id",
                reason: "must be LISTEN_SIGNAL_COLLECT",
            });
        }
        validate_token(
            "listen_signal_collect_ok.environment_profile_ref",
            &self.environment_profile_ref,
            128,
        )?;
        validate_token(
            "listen_signal_collect_ok.selected_adjustment_id",
            &self.selected_adjustment_id,
            64,
        )?;
        if self.ordered_adjustments.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_collect_ok.ordered_adjustments",
                reason: "must be non-empty",
            });
        }
        if self.ordered_adjustments.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_collect_ok.ordered_adjustments",
                reason: "must be <= 32",
            });
        }
        let mut seen = BTreeSet::new();
        for adjustment in &self.ordered_adjustments {
            adjustment.validate()?;
            if !seen.insert(adjustment.adjustment_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "listen_signal_collect_ok.ordered_adjustments",
                    reason: "adjustment_id must be unique",
                });
            }
        }
        if !seen.contains(self.selected_adjustment_id.as_str()) {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_collect_ok.selected_adjustment_id",
                reason: "must exist in ordered_adjustments",
            });
        }
        if !self.affects_capture_only {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_collect_ok.affects_capture_only",
                reason: "must be true",
            });
        }
        if !self.affects_delivery_mode_only {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_collect_ok.affects_delivery_mode_only",
                reason: "must be true",
            });
        }
        if !self.no_meaning_mutation {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_collect_ok.no_meaning_mutation",
                reason: "must be true",
            });
        }
        if !self.advisory_only {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_collect_ok.advisory_only",
                reason: "must be true",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_collect_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListenSignalFilterOk {
    pub schema_version: SchemaVersion,
    pub capability_id: ListenCapabilityId,
    pub reason_code: ReasonCodeId,
    pub validation_status: ListenValidationStatus,
    pub diagnostics: Vec<String>,
    pub applies_capture_profile: bool,
    pub applies_endpoint_profile: bool,
    pub applies_delivery_policy_hint: bool,
    pub no_meaning_mutation: bool,
    pub advisory_only: bool,
    pub no_execution_authority: bool,
}

impl ListenSignalFilterOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        validation_status: ListenValidationStatus,
        diagnostics: Vec<String>,
        applies_capture_profile: bool,
        applies_endpoint_profile: bool,
        applies_delivery_policy_hint: bool,
        no_meaning_mutation: bool,
        advisory_only: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1LISTEN_CONTRACT_VERSION,
            capability_id: ListenCapabilityId::ListenSignalFilter,
            reason_code,
            validation_status,
            diagnostics,
            applies_capture_profile,
            applies_endpoint_profile,
            applies_delivery_policy_hint,
            no_meaning_mutation,
            advisory_only,
            no_execution_authority,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for ListenSignalFilterOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LISTEN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_filter_ok.schema_version",
                reason: "must match PH1LISTEN_CONTRACT_VERSION",
            });
        }
        if self.capability_id != ListenCapabilityId::ListenSignalFilter {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_filter_ok.capability_id",
                reason: "must be LISTEN_SIGNAL_FILTER",
            });
        }
        if self.diagnostics.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_filter_ok.diagnostics",
                reason: "must be <= 16",
            });
        }
        for diagnostic in &self.diagnostics {
            validate_token("listen_signal_filter_ok.diagnostics", diagnostic, 96)?;
        }
        if self.validation_status == ListenValidationStatus::Fail && self.diagnostics.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_filter_ok.diagnostics",
                reason: "must be non-empty when validation_status=FAIL",
            });
        }
        if !self.applies_capture_profile
            && !self.applies_endpoint_profile
            && !self.applies_delivery_policy_hint
        {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_filter_ok",
                reason: "must apply one or more listen adjustments",
            });
        }
        if !self.no_meaning_mutation {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_filter_ok.no_meaning_mutation",
                reason: "must be true",
            });
        }
        if !self.advisory_only {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_filter_ok.advisory_only",
                reason: "must be true",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "listen_signal_filter_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListenRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: ListenCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl ListenRefuse {
    pub fn v1(
        capability_id: ListenCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1LISTEN_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for ListenRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LISTEN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "listen_refuse.schema_version",
                reason: "must match PH1LISTEN_CONTRACT_VERSION",
            });
        }
        validate_text("listen_refuse.message", &self.message, 192)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1ListenRequest {
    ListenSignalCollect(ListenSignalCollectRequest),
    ListenSignalFilter(ListenSignalFilterRequest),
}

impl Validate for Ph1ListenRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1ListenRequest::ListenSignalCollect(r) => r.validate(),
            Ph1ListenRequest::ListenSignalFilter(r) => r.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1ListenResponse {
    ListenSignalCollectOk(ListenSignalCollectOk),
    ListenSignalFilterOk(ListenSignalFilterOk),
    Refuse(ListenRefuse),
}

impl Validate for Ph1ListenResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1ListenResponse::ListenSignalCollectOk(out) => out.validate(),
            Ph1ListenResponse::ListenSignalFilterOk(out) => out.validate(),
            Ph1ListenResponse::Refuse(out) => out.validate(),
        }
    }
}

fn validate_token(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if value.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be non-empty",
        });
    }
    if value.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    if value.chars().any(|c| {
        !(c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ':' || c == '.' || c == '/')
    }) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must contain token-safe ASCII only",
        });
    }
    Ok(())
}

fn validate_text(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if value.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be non-empty",
        });
    }
    if value.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    if value.chars().any(|c| c.is_control()) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not contain control characters",
        });
    }
    Ok(())
}

fn validate_engine_id(field: &'static str, value: &str) -> Result<(), ContractViolation> {
    if value.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be non-empty",
        });
    }
    if value.len() > 64 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be <= 64 chars",
        });
    }
    if value
        .chars()
        .any(|c| !(c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_' || c == '.'))
    {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must contain uppercase engine id characters only",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope() -> ListenRequestEnvelope {
        ListenRequestEnvelope::v1(CorrelationId(9401), TurnId(531), 8, 4, 6).unwrap()
    }

    fn window(id: &str, noise_level_dbfs: i16) -> ListenSignalWindow {
        ListenSignalWindow::v1(
            id.to_string(),
            "PH1.K".to_string(),
            8300,
            7900,
            noise_level_dbfs,
            0,
            400,
            format!("listen:evidence:{}", id),
        )
        .unwrap()
    }

    fn correction() -> ListenCorrectionSnapshot {
        ListenCorrectionSnapshot::v1(1, 1, 0, 1200).unwrap()
    }

    fn context() -> ListenSessionContext {
        ListenSessionContext::v1(false, false, false, false).unwrap()
    }

    fn hint(id: &str) -> ListenAdjustmentHint {
        ListenAdjustmentHint::v1(
            id.to_string(),
            ListenEnvironmentMode::Noisy,
            ListenCaptureProfile::NoiseSuppressed,
            ListenEndpointProfile::Balanced,
            ListenDeliveryPolicyHint::TextPreferred,
            1200,
            "w_1".to_string(),
            "listen:evidence:w_1".to_string(),
        )
        .unwrap()
    }

    #[test]
    fn listen_contract_01_collect_request_is_schema_valid() {
        let req = ListenSignalCollectRequest::v1(
            envelope(),
            vec![window("w_1", -32)],
            correction(),
            context(),
        )
        .unwrap();
        assert!(req.validate().is_ok());
    }

    #[test]
    fn listen_contract_02_session_mode_conflict_is_rejected() {
        let bad = ListenSessionContext::v1(true, true, false, false);
        assert!(bad.is_err());
    }

    #[test]
    fn listen_contract_03_filter_request_requires_selected_in_ordered() {
        let req = ListenSignalFilterRequest::v1(
            envelope(),
            "env:noisy:noise_suppressed:text_preferred".to_string(),
            "adj_missing".to_string(),
            vec![hint("adj_1")],
            true,
        );
        assert!(req.is_err());
    }

    #[test]
    fn listen_contract_04_filter_ok_fail_requires_diagnostics() {
        let out = ListenSignalFilterOk::v1(
            ReasonCodeId(1),
            ListenValidationStatus::Fail,
            vec![],
            true,
            false,
            false,
            true,
            true,
            true,
        );
        assert!(out.is_err());
    }
}
