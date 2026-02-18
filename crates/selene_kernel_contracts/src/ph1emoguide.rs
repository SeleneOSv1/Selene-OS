#![forbid(unsafe_code)]

use crate::ph1j::{CorrelationId, TurnId};
use crate::ph1tts::{StyleModifier, StyleProfileRef};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1EMOGUIDE_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmoGuideCapabilityId {
    EmoGuideProfileBuild,
    EmoGuideProfileValidate,
}

impl EmoGuideCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            EmoGuideCapabilityId::EmoGuideProfileBuild => "EMO_GUIDE_PROFILE_BUILD",
            EmoGuideCapabilityId::EmoGuideProfileValidate => "EMO_GUIDE_PROFILE_VALIDATE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmoGuideValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmoGuideRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_interactions: u16,
    pub max_modifiers: u8,
    pub max_diagnostics: u8,
}

impl EmoGuideRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_interactions: u16,
        max_modifiers: u8,
        max_diagnostics: u8,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1EMOGUIDE_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_interactions,
            max_modifiers,
            max_diagnostics,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for EmoGuideRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1EMOGUIDE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "emo_guide_request_envelope.schema_version",
                reason: "must match PH1EMOGUIDE_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;

        if self.max_interactions == 0 || self.max_interactions > 4096 {
            return Err(ContractViolation::InvalidValue {
                field: "emo_guide_request_envelope.max_interactions",
                reason: "must be within 1..=4096",
            });
        }
        if self.max_modifiers == 0 || self.max_modifiers > 3 {
            return Err(ContractViolation::InvalidValue {
                field: "emo_guide_request_envelope.max_modifiers",
                reason: "must be within 1..=3",
            });
        }
        if self.max_diagnostics == 0 || self.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "emo_guide_request_envelope.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmoGuideInteractionSignals {
    pub schema_version: SchemaVersion,
    pub interaction_count: u16,
    pub correction_events: u16,
    pub interruption_events: u16,
    pub assertive_events: u16,
    pub cooperative_events: u16,
}

impl EmoGuideInteractionSignals {
    pub fn v1(
        interaction_count: u16,
        correction_events: u16,
        interruption_events: u16,
        assertive_events: u16,
        cooperative_events: u16,
    ) -> Result<Self, ContractViolation> {
        let signals = Self {
            schema_version: PH1EMOGUIDE_CONTRACT_VERSION,
            interaction_count,
            correction_events,
            interruption_events,
            assertive_events,
            cooperative_events,
        };
        signals.validate()?;
        Ok(signals)
    }
}

impl Validate for EmoGuideInteractionSignals {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1EMOGUIDE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "emo_guide_interaction_signals.schema_version",
                reason: "must match PH1EMOGUIDE_CONTRACT_VERSION",
            });
        }
        if self.interaction_count == 0 || self.interaction_count > 4096 {
            return Err(ContractViolation::InvalidValue {
                field: "emo_guide_interaction_signals.interaction_count",
                reason: "must be within 1..=4096",
            });
        }
        validate_event_counter(
            "emo_guide_interaction_signals.correction_events",
            self.correction_events,
            self.interaction_count,
        )?;
        validate_event_counter(
            "emo_guide_interaction_signals.interruption_events",
            self.interruption_events,
            self.interaction_count,
        )?;
        validate_event_counter(
            "emo_guide_interaction_signals.assertive_events",
            self.assertive_events,
            self.interaction_count,
        )?;
        validate_event_counter(
            "emo_guide_interaction_signals.cooperative_events",
            self.cooperative_events,
            self.interaction_count,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmoGuideProfile {
    pub schema_version: SchemaVersion,
    pub style_profile_ref: StyleProfileRef,
    pub modifiers: Vec<StyleModifier>,
    pub stability_window_turns: u16,
}

impl EmoGuideProfile {
    pub fn v1(
        style_profile_ref: StyleProfileRef,
        modifiers: Vec<StyleModifier>,
        stability_window_turns: u16,
    ) -> Result<Self, ContractViolation> {
        let profile = Self {
            schema_version: PH1EMOGUIDE_CONTRACT_VERSION,
            style_profile_ref,
            modifiers,
            stability_window_turns,
        };
        profile.validate()?;
        Ok(profile)
    }
}

impl Validate for EmoGuideProfile {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1EMOGUIDE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "emo_guide_profile.schema_version",
                reason: "must match PH1EMOGUIDE_CONTRACT_VERSION",
            });
        }
        if self.modifiers.len() > 3 {
            return Err(ContractViolation::InvalidValue {
                field: "emo_guide_profile.modifiers",
                reason: "must include <= 3 modifiers",
            });
        }
        let mut prev_rank: Option<u8> = None;
        for (idx, m) in self.modifiers.iter().enumerate() {
            if self.modifiers[..idx].contains(m) {
                return Err(ContractViolation::InvalidValue {
                    field: "emo_guide_profile.modifiers",
                    reason: "must not contain duplicates",
                });
            }
            let rank = style_modifier_rank(*m);
            if let Some(prev) = prev_rank {
                if rank < prev {
                    return Err(ContractViolation::InvalidValue {
                        field: "emo_guide_profile.modifiers",
                        reason: "must be sorted in canonical order",
                    });
                }
            }
            prev_rank = Some(rank);
        }
        if self.stability_window_turns < 3 || self.stability_window_turns > 4096 {
            return Err(ContractViolation::InvalidValue {
                field: "emo_guide_profile.stability_window_turns",
                reason: "must be within 3..=4096",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmoGuideProfileBuildRequest {
    pub schema_version: SchemaVersion,
    pub envelope: EmoGuideRequestEnvelope,
    pub verified_speaker_id: String,
    pub interaction_signals: EmoGuideInteractionSignals,
    pub emo_core_snapshot_ref: Option<String>,
}

impl EmoGuideProfileBuildRequest {
    pub fn v1(
        envelope: EmoGuideRequestEnvelope,
        verified_speaker_id: String,
        interaction_signals: EmoGuideInteractionSignals,
        emo_core_snapshot_ref: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1EMOGUIDE_CONTRACT_VERSION,
            envelope,
            verified_speaker_id,
            interaction_signals,
            emo_core_snapshot_ref,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for EmoGuideProfileBuildRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1EMOGUIDE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "emo_guide_profile_build_request.schema_version",
                reason: "must match PH1EMOGUIDE_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_token(
            "emo_guide_profile_build_request.verified_speaker_id",
            &self.verified_speaker_id,
            128,
        )?;
        self.interaction_signals.validate()?;
        if self.interaction_signals.interaction_count > self.envelope.max_interactions {
            return Err(ContractViolation::InvalidValue {
                field: "emo_guide_profile_build_request.interaction_signals.interaction_count",
                reason: "must be <= envelope.max_interactions",
            });
        }
        if let Some(snapshot_ref) = &self.emo_core_snapshot_ref {
            validate_token(
                "emo_guide_profile_build_request.emo_core_snapshot_ref",
                snapshot_ref,
                128,
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmoGuideProfileBuildOk {
    pub schema_version: SchemaVersion,
    pub capability_id: EmoGuideCapabilityId,
    pub reason_code: ReasonCodeId,
    pub verified_speaker_id: String,
    pub profile: EmoGuideProfile,
    pub tone_only: bool,
    pub no_meaning_drift: bool,
    pub auditable: bool,
    pub reversible: bool,
    pub no_execution_authority: bool,
}

impl EmoGuideProfileBuildOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        verified_speaker_id: String,
        profile: EmoGuideProfile,
        tone_only: bool,
        no_meaning_drift: bool,
        auditable: bool,
        reversible: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let ok = Self {
            schema_version: PH1EMOGUIDE_CONTRACT_VERSION,
            capability_id: EmoGuideCapabilityId::EmoGuideProfileBuild,
            reason_code,
            verified_speaker_id,
            profile,
            tone_only,
            no_meaning_drift,
            auditable,
            reversible,
            no_execution_authority,
        };
        ok.validate()?;
        Ok(ok)
    }
}

impl Validate for EmoGuideProfileBuildOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1EMOGUIDE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "emo_guide_profile_build_ok.schema_version",
                reason: "must match PH1EMOGUIDE_CONTRACT_VERSION",
            });
        }
        if self.capability_id != EmoGuideCapabilityId::EmoGuideProfileBuild {
            return Err(ContractViolation::InvalidValue {
                field: "emo_guide_profile_build_ok.capability_id",
                reason: "must be EMO_GUIDE_PROFILE_BUILD",
            });
        }
        validate_token(
            "emo_guide_profile_build_ok.verified_speaker_id",
            &self.verified_speaker_id,
            128,
        )?;
        self.profile.validate()?;
        validate_guard_flags(
            self.tone_only,
            self.no_meaning_drift,
            self.no_execution_authority,
            "emo_guide_profile_build_ok",
        )?;
        if !self.auditable {
            return Err(ContractViolation::InvalidValue {
                field: "emo_guide_profile_build_ok.auditable",
                reason: "must be true",
            });
        }
        if !self.reversible {
            return Err(ContractViolation::InvalidValue {
                field: "emo_guide_profile_build_ok.reversible",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmoGuideProfileValidateRequest {
    pub schema_version: SchemaVersion,
    pub envelope: EmoGuideRequestEnvelope,
    pub verified_speaker_id: String,
    pub interaction_signals: EmoGuideInteractionSignals,
    pub emo_core_snapshot_ref: Option<String>,
    pub proposed_profile: EmoGuideProfile,
}

impl EmoGuideProfileValidateRequest {
    pub fn v1(
        envelope: EmoGuideRequestEnvelope,
        verified_speaker_id: String,
        interaction_signals: EmoGuideInteractionSignals,
        emo_core_snapshot_ref: Option<String>,
        proposed_profile: EmoGuideProfile,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1EMOGUIDE_CONTRACT_VERSION,
            envelope,
            verified_speaker_id,
            interaction_signals,
            emo_core_snapshot_ref,
            proposed_profile,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for EmoGuideProfileValidateRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1EMOGUIDE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "emo_guide_profile_validate_request.schema_version",
                reason: "must match PH1EMOGUIDE_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_token(
            "emo_guide_profile_validate_request.verified_speaker_id",
            &self.verified_speaker_id,
            128,
        )?;
        self.interaction_signals.validate()?;
        if self.interaction_signals.interaction_count > self.envelope.max_interactions {
            return Err(ContractViolation::InvalidValue {
                field: "emo_guide_profile_validate_request.interaction_signals.interaction_count",
                reason: "must be <= envelope.max_interactions",
            });
        }
        if let Some(snapshot_ref) = &self.emo_core_snapshot_ref {
            validate_token(
                "emo_guide_profile_validate_request.emo_core_snapshot_ref",
                snapshot_ref,
                128,
            )?;
        }
        if self.proposed_profile.modifiers.len() > self.envelope.max_modifiers as usize {
            return Err(ContractViolation::InvalidValue {
                field: "emo_guide_profile_validate_request.proposed_profile.modifiers",
                reason: "must be <= envelope.max_modifiers",
            });
        }
        self.proposed_profile.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmoGuideProfileValidateOk {
    pub schema_version: SchemaVersion,
    pub capability_id: EmoGuideCapabilityId,
    pub reason_code: ReasonCodeId,
    pub validation_status: EmoGuideValidationStatus,
    pub diagnostics: Vec<String>,
    pub tone_only: bool,
    pub no_meaning_drift: bool,
    pub no_execution_authority: bool,
}

impl EmoGuideProfileValidateOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        validation_status: EmoGuideValidationStatus,
        diagnostics: Vec<String>,
        tone_only: bool,
        no_meaning_drift: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let ok = Self {
            schema_version: PH1EMOGUIDE_CONTRACT_VERSION,
            capability_id: EmoGuideCapabilityId::EmoGuideProfileValidate,
            reason_code,
            validation_status,
            diagnostics,
            tone_only,
            no_meaning_drift,
            no_execution_authority,
        };
        ok.validate()?;
        Ok(ok)
    }
}

impl Validate for EmoGuideProfileValidateOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1EMOGUIDE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "emo_guide_profile_validate_ok.schema_version",
                reason: "must match PH1EMOGUIDE_CONTRACT_VERSION",
            });
        }
        if self.capability_id != EmoGuideCapabilityId::EmoGuideProfileValidate {
            return Err(ContractViolation::InvalidValue {
                field: "emo_guide_profile_validate_ok.capability_id",
                reason: "must be EMO_GUIDE_PROFILE_VALIDATE",
            });
        }
        if self.diagnostics.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "emo_guide_profile_validate_ok.diagnostics",
                reason: "must include <= 16 diagnostics",
            });
        }
        for diagnostic in &self.diagnostics {
            validate_token("emo_guide_profile_validate_ok.diagnostics", diagnostic, 96)?;
        }
        validate_guard_flags(
            self.tone_only,
            self.no_meaning_drift,
            self.no_execution_authority,
            "emo_guide_profile_validate_ok",
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmoGuideRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: EmoGuideCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl EmoGuideRefuse {
    pub fn v1(
        capability_id: EmoGuideCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let refuse = Self {
            schema_version: PH1EMOGUIDE_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        refuse.validate()?;
        Ok(refuse)
    }
}

impl Validate for EmoGuideRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1EMOGUIDE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "emo_guide_refuse.schema_version",
                reason: "must match PH1EMOGUIDE_CONTRACT_VERSION",
            });
        }
        validate_message("emo_guide_refuse.message", &self.message, 256)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1EmoGuideRequest {
    EmoGuideProfileBuild(EmoGuideProfileBuildRequest),
    EmoGuideProfileValidate(EmoGuideProfileValidateRequest),
}

impl Validate for Ph1EmoGuideRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1EmoGuideRequest::EmoGuideProfileBuild(r) => r.validate(),
            Ph1EmoGuideRequest::EmoGuideProfileValidate(r) => r.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1EmoGuideResponse {
    EmoGuideProfileBuildOk(EmoGuideProfileBuildOk),
    EmoGuideProfileValidateOk(EmoGuideProfileValidateOk),
    Refuse(EmoGuideRefuse),
}

impl Validate for Ph1EmoGuideResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1EmoGuideResponse::EmoGuideProfileBuildOk(r) => r.validate(),
            Ph1EmoGuideResponse::EmoGuideProfileValidateOk(r) => r.validate(),
            Ph1EmoGuideResponse::Refuse(r) => r.validate(),
        }
    }
}

fn validate_event_counter(
    field: &'static str,
    value: u16,
    interaction_count: u16,
) -> Result<(), ContractViolation> {
    if value > interaction_count {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be <= interaction_count",
        });
    }
    Ok(())
}

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
    if value.chars().any(|c| c.is_control()) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not contain control characters",
        });
    }
    Ok(())
}

fn validate_message(
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
    Ok(())
}

fn validate_guard_flags(
    tone_only: bool,
    no_meaning_drift: bool,
    no_execution_authority: bool,
    field_prefix: &'static str,
) -> Result<(), ContractViolation> {
    if !tone_only {
        return Err(ContractViolation::InvalidValue {
            field: field_prefix,
            reason: "tone_only must be true",
        });
    }
    if !no_meaning_drift {
        return Err(ContractViolation::InvalidValue {
            field: field_prefix,
            reason: "no_meaning_drift must be true",
        });
    }
    if !no_execution_authority {
        return Err(ContractViolation::InvalidValue {
            field: field_prefix,
            reason: "no_execution_authority must be true",
        });
    }
    Ok(())
}

fn style_modifier_rank(m: StyleModifier) -> u8 {
    match m {
        StyleModifier::Brief => 0,
        StyleModifier::Warm => 1,
        StyleModifier::Formal => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env() -> EmoGuideRequestEnvelope {
        EmoGuideRequestEnvelope::v1(CorrelationId(901), TurnId(41), 120, 3, 8).unwrap()
    }

    fn signals() -> EmoGuideInteractionSignals {
        EmoGuideInteractionSignals::v1(20, 4, 3, 8, 5).unwrap()
    }

    #[test]
    fn at_emo_guide_01_profile_build_contract_is_schema_valid() {
        let profile = EmoGuideProfile::v1(
            StyleProfileRef::Dominant,
            vec![StyleModifier::Brief, StyleModifier::Formal],
            20,
        )
        .unwrap();

        let out = Ph1EmoGuideResponse::EmoGuideProfileBuildOk(
            EmoGuideProfileBuildOk::v1(
                ReasonCodeId(1),
                "speaker_1".to_string(),
                profile,
                true,
                true,
                true,
                true,
                true,
            )
            .unwrap(),
        );

        assert!(out.validate().is_ok());
    }

    #[test]
    fn at_emo_guide_02_modifiers_must_be_unique_and_sorted() {
        let duplicated = EmoGuideProfile::v1(
            StyleProfileRef::Gentle,
            vec![StyleModifier::Warm, StyleModifier::Warm],
            12,
        );
        assert!(duplicated.is_err());

        let unsorted = EmoGuideProfile::v1(
            StyleProfileRef::Gentle,
            vec![StyleModifier::Formal, StyleModifier::Brief],
            12,
        );
        assert!(unsorted.is_err());
    }

    #[test]
    fn at_emo_guide_03_request_rejects_unverified_speaker_id() {
        let req = EmoGuideProfileBuildRequest::v1(env(), " ".to_string(), signals(), None);
        assert!(req.is_err());
    }

    #[test]
    fn at_emo_guide_04_validate_response_requires_guard_flags() {
        let out = EmoGuideProfileValidateOk::v1(
            ReasonCodeId(2),
            EmoGuideValidationStatus::Ok,
            vec![],
            true,
            false,
            true,
        );
        assert!(out.is_err());
    }
}
