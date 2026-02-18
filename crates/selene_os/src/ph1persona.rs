#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1persona::{
    PersonaCapabilityId, PersonaPreferenceSignal, PersonaProfileBuildOk,
    PersonaProfileBuildRequest, PersonaProfileValidateOk, PersonaProfileValidateRequest,
    PersonaRefuse, PersonaRequestEnvelope, PersonaValidationStatus, Ph1PersonaRequest,
    Ph1PersonaResponse,
};
use selene_kernel_contracts::ph1tts::StyleProfileRef;
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.PERSONA OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_PERSONA_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x5052_0101);
    pub const PH1_PERSONA_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5052_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1PersonaWiringConfig {
    pub persona_enabled: bool,
    pub max_signals: u8,
    pub max_diagnostics: u8,
}

impl Ph1PersonaWiringConfig {
    pub fn mvp_v1(persona_enabled: bool) -> Self {
        Self {
            persona_enabled,
            max_signals: 16,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonaTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub verified_user_id: Option<String>,
    pub verified_speaker_id: Option<String>,
    pub preference_signals: Vec<PersonaPreferenceSignal>,
    pub correction_event_count: u16,
    pub emo_guide_style_profile_ref: Option<StyleProfileRef>,
    pub previous_snapshot_ref: Option<String>,
}

impl PersonaTurnInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        verified_user_id: Option<String>,
        verified_speaker_id: Option<String>,
        preference_signals: Vec<PersonaPreferenceSignal>,
        correction_event_count: u16,
        emo_guide_style_profile_ref: Option<StyleProfileRef>,
        previous_snapshot_ref: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            verified_user_id,
            verified_speaker_id,
            preference_signals,
            correction_event_count,
            emo_guide_style_profile_ref,
            previous_snapshot_ref,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for PersonaTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;

        if let Some(verified_user_id) = &self.verified_user_id {
            validate_token("persona_turn_input.verified_user_id", verified_user_id, 128)?;
        }
        if let Some(verified_speaker_id) = &self.verified_speaker_id {
            validate_token(
                "persona_turn_input.verified_speaker_id",
                verified_speaker_id,
                128,
            )?;
        }
        if self.preference_signals.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "persona_turn_input.preference_signals",
                reason: "must be <= 128",
            });
        }
        for signal in &self.preference_signals {
            signal.validate()?;
        }
        if self.correction_event_count > 1000 {
            return Err(ContractViolation::InvalidValue {
                field: "persona_turn_input.correction_event_count",
                reason: "must be <= 1000",
            });
        }
        if let Some(previous_snapshot_ref) = &self.previous_snapshot_ref {
            validate_token(
                "persona_turn_input.previous_snapshot_ref",
                previous_snapshot_ref,
                128,
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonaForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub profile_build: PersonaProfileBuildOk,
    pub profile_validate: PersonaProfileValidateOk,
}

impl PersonaForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        profile_build: PersonaProfileBuildOk,
        profile_validate: PersonaProfileValidateOk,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            profile_build,
            profile_validate,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Validate for PersonaForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.profile_build.validate()?;
        self.profile_validate.validate()?;

        if self.profile_validate.validation_status != PersonaValidationStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "persona_forward_bundle.profile_validate.validation_status",
                reason: "must be OK",
            });
        }
        if !self.profile_build.tone_only
            || !self.profile_build.no_meaning_drift
            || !self.profile_build.no_execution_authority
            || !self.profile_validate.tone_only
            || !self.profile_validate.no_meaning_drift
            || !self.profile_validate.no_execution_authority
        {
            return Err(ContractViolation::InvalidValue {
                field: "persona_forward_bundle",
                reason: "all tone-only/no-drift/no-execution flags must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersonaWiringOutcome {
    NotInvokedDisabled,
    NotInvokedIdentityUnknown,
    NotInvokedNoPersonaInput,
    Refused(PersonaRefuse),
    Forwarded(PersonaForwardBundle),
}

pub trait Ph1PersonaEngine {
    fn run(&self, req: &Ph1PersonaRequest) -> Ph1PersonaResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1PersonaWiring<E>
where
    E: Ph1PersonaEngine,
{
    config: Ph1PersonaWiringConfig,
    engine: E,
}

impl<E> Ph1PersonaWiring<E>
where
    E: Ph1PersonaEngine,
{
    pub fn new(config: Ph1PersonaWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_signals == 0 || config.max_signals > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1persona_wiring_config.max_signals",
                reason: "must be within 1..=32",
            });
        }
        if config.max_diagnostics == 0 || config.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1persona_wiring_config.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(
        &self,
        input: &PersonaTurnInput,
    ) -> Result<PersonaWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.persona_enabled {
            return Ok(PersonaWiringOutcome::NotInvokedDisabled);
        }

        let (verified_user_id, verified_speaker_id) =
            match (&input.verified_user_id, &input.verified_speaker_id) {
                (Some(user), Some(speaker))
                    if !user.trim().is_empty() && !speaker.trim().is_empty() =>
                {
                    (user.clone(), speaker.clone())
                }
                _ => return Ok(PersonaWiringOutcome::NotInvokedIdentityUnknown),
            };

        if input.preference_signals.is_empty() && input.previous_snapshot_ref.is_none() {
            return Ok(PersonaWiringOutcome::NotInvokedNoPersonaInput);
        }

        let envelope = PersonaRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_signals, 32),
            min(self.config.max_diagnostics, 16),
        )?;

        let build_req = Ph1PersonaRequest::PersonaProfileBuild(PersonaProfileBuildRequest::v1(
            envelope.clone(),
            verified_user_id.clone(),
            verified_speaker_id.clone(),
            input.preference_signals.clone(),
            input.correction_event_count,
            input.emo_guide_style_profile_ref,
            input.previous_snapshot_ref.clone(),
        )?);
        let build_resp = self.engine.run(&build_req);
        build_resp.validate()?;

        let build_ok = match build_resp {
            Ph1PersonaResponse::Refuse(refuse) => return Ok(PersonaWiringOutcome::Refused(refuse)),
            Ph1PersonaResponse::PersonaProfileBuildOk(ok) => ok,
            Ph1PersonaResponse::PersonaProfileValidateOk(_) => {
                return Ok(PersonaWiringOutcome::Refused(PersonaRefuse::v1(
                    PersonaCapabilityId::PersonaProfileBuild,
                    reason_codes::PH1_PERSONA_INTERNAL_PIPELINE_ERROR,
                    "unexpected profile-validate response for profile-build request".to_string(),
                )?))
            }
        };

        let validate_req =
            Ph1PersonaRequest::PersonaProfileValidate(PersonaProfileValidateRequest::v1(
                envelope,
                verified_user_id,
                verified_speaker_id,
                input.preference_signals.clone(),
                input.correction_event_count,
                input.emo_guide_style_profile_ref,
                input.previous_snapshot_ref.clone(),
                build_ok.profile_snapshot.clone(),
            )?);
        let validate_resp = self.engine.run(&validate_req);
        validate_resp.validate()?;

        let validate_ok = match validate_resp {
            Ph1PersonaResponse::Refuse(refuse) => return Ok(PersonaWiringOutcome::Refused(refuse)),
            Ph1PersonaResponse::PersonaProfileValidateOk(ok) => ok,
            Ph1PersonaResponse::PersonaProfileBuildOk(_) => {
                return Ok(PersonaWiringOutcome::Refused(PersonaRefuse::v1(
                    PersonaCapabilityId::PersonaProfileValidate,
                    reason_codes::PH1_PERSONA_INTERNAL_PIPELINE_ERROR,
                    "unexpected profile-build response for profile-validate request".to_string(),
                )?))
            }
        };

        if validate_ok.validation_status != PersonaValidationStatus::Ok {
            return Ok(PersonaWiringOutcome::Refused(PersonaRefuse::v1(
                PersonaCapabilityId::PersonaProfileValidate,
                reason_codes::PH1_PERSONA_VALIDATION_FAILED,
                "persona profile validation failed".to_string(),
            )?));
        }

        let bundle =
            PersonaForwardBundle::v1(input.correlation_id, input.turn_id, build_ok, validate_ok)?;
        Ok(PersonaWiringOutcome::Forwarded(bundle))
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1persona::{
        PersonaBrevityRef, PersonaDeliveryPolicyRef, PersonaPreferenceKey, PersonaProfileSnapshot,
    };
    use selene_kernel_contracts::ph1tts::StyleProfileRef;
    use selene_kernel_contracts::ReasonCodeId;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct DeterministicPersonaEngine {
        fail_validate: bool,
    }

    impl Ph1PersonaEngine for DeterministicPersonaEngine {
        fn run(&self, req: &Ph1PersonaRequest) -> Ph1PersonaResponse {
            match req {
                Ph1PersonaRequest::PersonaProfileBuild(r) => {
                    let style = r
                        .emo_guide_style_profile_ref
                        .unwrap_or(StyleProfileRef::Gentle);
                    let has_text_only = r.preference_signals.iter().any(|signal| {
                        signal.key == PersonaPreferenceKey::PrivacyPreference
                            && signal.value.to_ascii_lowercase().contains("text")
                    });
                    let delivery = if has_text_only {
                        PersonaDeliveryPolicyRef::TextOnly
                    } else {
                        PersonaDeliveryPolicyRef::VoiceAllowed
                    };
                    let snapshot = PersonaProfileSnapshot::v1(
                        style,
                        delivery,
                        PersonaBrevityRef::Balanced,
                        "pers:v1|u:test|s:gentle|d:text|b:balanced|c:0|src:signal|p:0".to_string(),
                    )
                    .unwrap();
                    Ph1PersonaResponse::PersonaProfileBuildOk(
                        PersonaProfileBuildOk::v1(
                            ReasonCodeId(1),
                            snapshot,
                            true,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1PersonaRequest::PersonaProfileValidate(_) => {
                    let (status, reason_code, diagnostics) = if self.fail_validate {
                        (
                            PersonaValidationStatus::Fail,
                            ReasonCodeId(2),
                            vec!["forced_validation_drift".to_string()],
                        )
                    } else {
                        (PersonaValidationStatus::Ok, ReasonCodeId(3), vec![])
                    };
                    Ph1PersonaResponse::PersonaProfileValidateOk(
                        PersonaProfileValidateOk::v1(
                            reason_code,
                            status,
                            diagnostics,
                            true,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    fn wiring(fail_validate: bool) -> Ph1PersonaWiring<DeterministicPersonaEngine> {
        Ph1PersonaWiring::new(
            Ph1PersonaWiringConfig::mvp_v1(true),
            DeterministicPersonaEngine { fail_validate },
        )
        .unwrap()
    }

    fn signal(
        key: PersonaPreferenceKey,
        value: &str,
        evidence_ref: &str,
    ) -> PersonaPreferenceSignal {
        PersonaPreferenceSignal::v1(key, value.to_string(), evidence_ref.to_string()).unwrap()
    }

    fn full_input() -> PersonaTurnInput {
        PersonaTurnInput::v1(
            CorrelationId(3901),
            TurnId(91),
            Some("user_10".to_string()),
            Some("speaker_10".to_string()),
            vec![
                signal(
                    PersonaPreferenceKey::BrevityPreference,
                    "brief",
                    "ev:brevity",
                ),
                signal(
                    PersonaPreferenceKey::PrivacyPreference,
                    "text_only",
                    "ev:privacy",
                ),
            ],
            4,
            Some(StyleProfileRef::Gentle),
            None,
        )
        .unwrap()
    }

    #[test]
    fn at_pers_01_unknown_speaker_no_persona_applied() {
        let wiring = wiring(false);
        let input = PersonaTurnInput::v1(
            CorrelationId(3902),
            TurnId(92),
            Some("user_11".to_string()),
            None,
            vec![signal(
                PersonaPreferenceKey::BrevityPreference,
                "brief",
                "ev:1",
            )],
            0,
            None,
            None,
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        assert_eq!(out, PersonaWiringOutcome::NotInvokedIdentityUnknown);
    }

    #[test]
    fn at_pers_02_preference_updates_require_evidence_and_auditable_bundle() {
        let wiring = wiring(false);
        let out = wiring.run_turn(&full_input()).unwrap();
        match out {
            PersonaWiringOutcome::Forwarded(bundle) => {
                assert!(bundle.profile_build.auditable);
                assert!(bundle.profile_validate.auditable);
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_pers_03_persona_hints_are_tone_only_and_no_meaning_drift() {
        let wiring = wiring(false);
        let out = wiring.run_turn(&full_input()).unwrap();
        match out {
            PersonaWiringOutcome::Forwarded(bundle) => {
                assert!(bundle.profile_build.tone_only);
                assert!(bundle.profile_build.no_meaning_drift);
                assert!(bundle.profile_build.no_execution_authority);
                assert!(bundle.profile_validate.tone_only);
                assert!(bundle.profile_validate.no_meaning_drift);
                assert!(bundle.profile_validate.no_execution_authority);
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_pers_04_validate_drift_fails_closed() {
        let wiring = wiring(true);
        let out = wiring.run_turn(&full_input()).unwrap();
        match out {
            PersonaWiringOutcome::Refused(refuse) => {
                assert_eq!(
                    refuse.reason_code,
                    reason_codes::PH1_PERSONA_VALIDATION_FAILED
                );
            }
            _ => panic!("expected Refused"),
        }
    }
}
