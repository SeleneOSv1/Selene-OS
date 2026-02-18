#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1emoguide::{
    EmoGuideCapabilityId, EmoGuideInteractionSignals, EmoGuideProfileBuildOk,
    EmoGuideProfileBuildRequest, EmoGuideProfileValidateOk, EmoGuideProfileValidateRequest,
    EmoGuideRefuse, EmoGuideRequestEnvelope, EmoGuideValidationStatus, Ph1EmoGuideRequest,
    Ph1EmoGuideResponse,
};
use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.EMO.GUIDE OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_EMO_GUIDE_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4547_0101);
    pub const PH1_EMO_GUIDE_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4547_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1EmoGuideWiringConfig {
    pub emo_guide_enabled: bool,
    pub max_interactions: u16,
    pub max_modifiers: u8,
    pub max_diagnostics: u8,
}

impl Ph1EmoGuideWiringConfig {
    pub fn mvp_v1(emo_guide_enabled: bool) -> Self {
        Self {
            emo_guide_enabled,
            max_interactions: 120,
            max_modifiers: 3,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmoGuideTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub verified_speaker_id: String,
    pub interaction_signals: EmoGuideInteractionSignals,
    pub emo_core_snapshot_ref: Option<String>,
}

impl EmoGuideTurnInput {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        verified_speaker_id: String,
        interaction_signals: EmoGuideInteractionSignals,
        emo_core_snapshot_ref: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            verified_speaker_id,
            interaction_signals,
            emo_core_snapshot_ref,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for EmoGuideTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.interaction_signals.validate()?;

        if self.verified_speaker_id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "emo_guide_turn_input.verified_speaker_id",
                reason: "must not be empty",
            });
        }
        if self.verified_speaker_id.len() > 128
            || self.verified_speaker_id.chars().any(|c| c.is_control())
        {
            return Err(ContractViolation::InvalidValue {
                field: "emo_guide_turn_input.verified_speaker_id",
                reason: "must be <= 128 chars and not contain control chars",
            });
        }

        if let Some(snapshot_ref) = &self.emo_core_snapshot_ref {
            if snapshot_ref.trim().is_empty()
                || snapshot_ref.len() > 128
                || snapshot_ref.chars().any(|c| c.is_control())
            {
                return Err(ContractViolation::InvalidValue {
                    field: "emo_guide_turn_input.emo_core_snapshot_ref",
                    reason: "must be non-empty, <= 128 chars, and not contain control chars",
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmoGuideForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub profile_build: EmoGuideProfileBuildOk,
    pub profile_validate: EmoGuideProfileValidateOk,
}

impl EmoGuideForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        profile_build: EmoGuideProfileBuildOk,
        profile_validate: EmoGuideProfileValidateOk,
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

impl Validate for EmoGuideForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.profile_build.validate()?;
        self.profile_validate.validate()?;

        if self.profile_validate.validation_status != EmoGuideValidationStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "emo_guide_forward_bundle.profile_validate.validation_status",
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
                field: "emo_guide_forward_bundle",
                reason: "all tone-only/no-drift/no-execution flags must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmoGuideWiringOutcome {
    NotInvokedDisabled,
    Refused(EmoGuideRefuse),
    Forwarded(EmoGuideForwardBundle),
}

pub trait Ph1EmoGuideEngine {
    fn run(&self, req: &Ph1EmoGuideRequest) -> Ph1EmoGuideResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1EmoGuideWiring<E>
where
    E: Ph1EmoGuideEngine,
{
    config: Ph1EmoGuideWiringConfig,
    engine: E,
}

impl<E> Ph1EmoGuideWiring<E>
where
    E: Ph1EmoGuideEngine,
{
    pub fn new(config: Ph1EmoGuideWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_interactions == 0 || config.max_interactions > 4096 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1_emoguide_wiring_config.max_interactions",
                reason: "must be within 1..=4096",
            });
        }
        if config.max_modifiers == 0 || config.max_modifiers > 3 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1_emoguide_wiring_config.max_modifiers",
                reason: "must be within 1..=3",
            });
        }
        if config.max_diagnostics == 0 || config.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1_emoguide_wiring_config.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(
        &self,
        input: &EmoGuideTurnInput,
    ) -> Result<EmoGuideWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.emo_guide_enabled {
            return Ok(EmoGuideWiringOutcome::NotInvokedDisabled);
        }

        let envelope = EmoGuideRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_interactions, 4096),
            min(self.config.max_modifiers, 3),
            min(self.config.max_diagnostics, 16),
        )?;

        let build_req = Ph1EmoGuideRequest::EmoGuideProfileBuild(EmoGuideProfileBuildRequest::v1(
            envelope.clone(),
            input.verified_speaker_id.clone(),
            input.interaction_signals.clone(),
            input.emo_core_snapshot_ref.clone(),
        )?);
        let build_resp = self.engine.run(&build_req);
        build_resp.validate()?;

        let build_ok = match build_resp {
            Ph1EmoGuideResponse::Refuse(r) => return Ok(EmoGuideWiringOutcome::Refused(r)),
            Ph1EmoGuideResponse::EmoGuideProfileBuildOk(ok) => ok,
            Ph1EmoGuideResponse::EmoGuideProfileValidateOk(_) => {
                return Ok(EmoGuideWiringOutcome::Refused(EmoGuideRefuse::v1(
                    EmoGuideCapabilityId::EmoGuideProfileBuild,
                    reason_codes::PH1_EMO_GUIDE_INTERNAL_PIPELINE_ERROR,
                    "unexpected validate response for build request".to_string(),
                )?))
            }
        };

        let validate_req =
            Ph1EmoGuideRequest::EmoGuideProfileValidate(EmoGuideProfileValidateRequest::v1(
                envelope,
                input.verified_speaker_id.clone(),
                input.interaction_signals.clone(),
                input.emo_core_snapshot_ref.clone(),
                build_ok.profile.clone(),
            )?);
        let validate_resp = self.engine.run(&validate_req);
        validate_resp.validate()?;

        let validate_ok = match validate_resp {
            Ph1EmoGuideResponse::Refuse(r) => return Ok(EmoGuideWiringOutcome::Refused(r)),
            Ph1EmoGuideResponse::EmoGuideProfileValidateOk(ok) => ok,
            Ph1EmoGuideResponse::EmoGuideProfileBuildOk(_) => {
                return Ok(EmoGuideWiringOutcome::Refused(EmoGuideRefuse::v1(
                    EmoGuideCapabilityId::EmoGuideProfileValidate,
                    reason_codes::PH1_EMO_GUIDE_INTERNAL_PIPELINE_ERROR,
                    "unexpected build response for validate request".to_string(),
                )?))
            }
        };

        if validate_ok.validation_status != EmoGuideValidationStatus::Ok {
            return Ok(EmoGuideWiringOutcome::Refused(EmoGuideRefuse::v1(
                EmoGuideCapabilityId::EmoGuideProfileValidate,
                reason_codes::PH1_EMO_GUIDE_VALIDATION_FAILED,
                "emo-guide profile validation failed".to_string(),
            )?));
        }

        let bundle =
            EmoGuideForwardBundle::v1(input.correlation_id, input.turn_id, build_ok, validate_ok)?;
        Ok(EmoGuideWiringOutcome::Forwarded(bundle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1emoguide::{
        EmoGuideProfile, EmoGuideProfileBuildOk, EmoGuideProfileValidateOk,
        EmoGuideValidationStatus,
    };
    use selene_kernel_contracts::ph1tts::{StyleModifier, StyleProfileRef};
    use selene_kernel_contracts::ReasonCodeId;

    struct DeterministicEmoGuideEngine;

    impl Ph1EmoGuideEngine for DeterministicEmoGuideEngine {
        fn run(&self, req: &Ph1EmoGuideRequest) -> Ph1EmoGuideResponse {
            match req {
                Ph1EmoGuideRequest::EmoGuideProfileBuild(r) => {
                    let style = if r.interaction_signals.assertive_events
                        >= r.interaction_signals.cooperative_events
                    {
                        StyleProfileRef::Dominant
                    } else {
                        StyleProfileRef::Gentle
                    };
                    let modifiers = if style == StyleProfileRef::Dominant {
                        vec![StyleModifier::Brief, StyleModifier::Formal]
                    } else {
                        vec![StyleModifier::Warm]
                    };
                    let profile = EmoGuideProfile::v1(
                        style,
                        modifiers,
                        r.interaction_signals.interaction_count,
                    )
                    .unwrap();
                    Ph1EmoGuideResponse::EmoGuideProfileBuildOk(
                        EmoGuideProfileBuildOk::v1(
                            ReasonCodeId(1),
                            r.verified_speaker_id.clone(),
                            profile,
                            true,
                            true,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1EmoGuideRequest::EmoGuideProfileValidate(_r) => {
                    Ph1EmoGuideResponse::EmoGuideProfileValidateOk(
                        EmoGuideProfileValidateOk::v1(
                            ReasonCodeId(2),
                            EmoGuideValidationStatus::Ok,
                            vec![],
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

    struct DriftEmoGuideEngine;

    impl Ph1EmoGuideEngine for DriftEmoGuideEngine {
        fn run(&self, req: &Ph1EmoGuideRequest) -> Ph1EmoGuideResponse {
            match req {
                Ph1EmoGuideRequest::EmoGuideProfileBuild(r) => {
                    let profile = EmoGuideProfile::v1(
                        StyleProfileRef::Dominant,
                        vec![StyleModifier::Brief],
                        r.interaction_signals.interaction_count,
                    )
                    .unwrap();
                    Ph1EmoGuideResponse::EmoGuideProfileBuildOk(
                        EmoGuideProfileBuildOk::v1(
                            ReasonCodeId(10),
                            r.verified_speaker_id.clone(),
                            profile,
                            true,
                            true,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1EmoGuideRequest::EmoGuideProfileValidate(_r) => {
                    Ph1EmoGuideResponse::EmoGuideProfileValidateOk(
                        EmoGuideProfileValidateOk::v1(
                            ReasonCodeId(11),
                            EmoGuideValidationStatus::Fail,
                            vec!["style_profile_ref_mismatch".to_string()],
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

    fn turn_input(assertive: u16, cooperative: u16) -> EmoGuideTurnInput {
        EmoGuideTurnInput::v1(
            CorrelationId(1501),
            TurnId(111),
            "speaker_turn".to_string(),
            EmoGuideInteractionSignals::v1(20, 3, 2, assertive, cooperative).unwrap(),
            None,
        )
        .unwrap()
    }

    #[test]
    fn at_emo_guide_01_os_invokes_and_returns_schema_valid_forward_bundle() {
        let wiring = Ph1EmoGuideWiring::new(
            Ph1EmoGuideWiringConfig::mvp_v1(true),
            DeterministicEmoGuideEngine,
        )
        .unwrap();

        let out = wiring.run_turn(&turn_input(12, 4)).unwrap();
        match out {
            EmoGuideWiringOutcome::Forwarded(bundle) => {
                assert!(bundle.validate().is_ok());
                assert_eq!(
                    bundle.profile_build.profile.style_profile_ref,
                    StyleProfileRef::Dominant
                );
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_emo_guide_02_disabled_wiring_does_not_invoke_engine() {
        let wiring =
            Ph1EmoGuideWiring::new(Ph1EmoGuideWiringConfig::mvp_v1(false), DriftEmoGuideEngine)
                .unwrap();

        let out = wiring.run_turn(&turn_input(10, 2)).unwrap();
        assert_eq!(out, EmoGuideWiringOutcome::NotInvokedDisabled);
    }

    #[test]
    fn at_emo_guide_03_profile_validation_fail_is_fail_closed() {
        let wiring =
            Ph1EmoGuideWiring::new(Ph1EmoGuideWiringConfig::mvp_v1(true), DriftEmoGuideEngine)
                .unwrap();

        let out = wiring.run_turn(&turn_input(10, 2)).unwrap();
        match out {
            EmoGuideWiringOutcome::Refused(r) => {
                assert_eq!(r.reason_code, reason_codes::PH1_EMO_GUIDE_VALIDATION_FAILED);
            }
            _ => panic!("expected Refused"),
        }
    }

    #[test]
    fn at_emo_guide_04_forward_bundle_keeps_tone_only_invariants() {
        let wiring = Ph1EmoGuideWiring::new(
            Ph1EmoGuideWiringConfig::mvp_v1(true),
            DeterministicEmoGuideEngine,
        )
        .unwrap();

        let out = wiring.run_turn(&turn_input(2, 12)).unwrap();
        match out {
            EmoGuideWiringOutcome::Forwarded(bundle) => {
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
}
