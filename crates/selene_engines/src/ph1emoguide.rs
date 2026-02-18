#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1emoguide::{
    EmoGuideCapabilityId, EmoGuideInteractionSignals, EmoGuideProfile, EmoGuideProfileBuildOk,
    EmoGuideProfileBuildRequest, EmoGuideProfileValidateOk, EmoGuideProfileValidateRequest,
    EmoGuideRefuse, EmoGuideValidationStatus, Ph1EmoGuideRequest, Ph1EmoGuideResponse,
};
use selene_kernel_contracts::ph1tts::{StyleModifier, StyleProfileRef};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.EMO.GUIDE reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_EMO_GUIDE_OK_PROFILE_BUILD: ReasonCodeId = ReasonCodeId(0x4547_0001);
    pub const PH1_EMO_GUIDE_OK_PROFILE_VALIDATE: ReasonCodeId = ReasonCodeId(0x4547_0002);

    pub const PH1_EMO_GUIDE_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x4547_00F1);
    pub const PH1_EMO_GUIDE_IDENTITY_REQUIRED: ReasonCodeId = ReasonCodeId(0x4547_00F2);
    pub const PH1_EMO_GUIDE_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x4547_00F3);
    pub const PH1_EMO_GUIDE_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4547_00F4);
    pub const PH1_EMO_GUIDE_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4547_00F5);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1EmoGuideConfig {
    pub max_interactions: u16,
    pub max_modifiers: u8,
    pub min_stability_window_turns: u16,
}

impl Ph1EmoGuideConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_interactions: 120,
            max_modifiers: 3,
            min_stability_window_turns: 6,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1EmoGuideRuntime {
    config: Ph1EmoGuideConfig,
}

impl Ph1EmoGuideRuntime {
    pub fn new(config: Ph1EmoGuideConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1EmoGuideRequest) -> Ph1EmoGuideResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_EMO_GUIDE_INPUT_SCHEMA_INVALID,
                "emo-guide request failed contract validation",
            );
        }

        match req {
            Ph1EmoGuideRequest::EmoGuideProfileBuild(r) => self.run_profile_build(r),
            Ph1EmoGuideRequest::EmoGuideProfileValidate(r) => self.run_profile_validate(r),
        }
    }

    fn run_profile_build(&self, req: &EmoGuideProfileBuildRequest) -> Ph1EmoGuideResponse {
        if req.verified_speaker_id.trim().is_empty() {
            return self.refuse(
                EmoGuideCapabilityId::EmoGuideProfileBuild,
                reason_codes::PH1_EMO_GUIDE_IDENTITY_REQUIRED,
                "verified speaker id is required",
            );
        }

        if req.interaction_signals.interaction_count > self.config.max_interactions {
            return self.refuse(
                EmoGuideCapabilityId::EmoGuideProfileBuild,
                reason_codes::PH1_EMO_GUIDE_BUDGET_EXCEEDED,
                "interaction_count exceeds runtime budget",
            );
        }

        let profile = match classify_profile(
            &req.interaction_signals,
            self.config.max_modifiers,
            self.config.min_stability_window_turns,
        ) {
            Ok(profile) => profile,
            Err(_) => {
                return self.refuse(
                    EmoGuideCapabilityId::EmoGuideProfileBuild,
                    reason_codes::PH1_EMO_GUIDE_INTERNAL_PIPELINE_ERROR,
                    "failed to classify emotional guidance profile",
                )
            }
        };

        match EmoGuideProfileBuildOk::v1(
            reason_codes::PH1_EMO_GUIDE_OK_PROFILE_BUILD,
            req.verified_speaker_id.clone(),
            profile,
            true,
            true,
            true,
            true,
            true,
        ) {
            Ok(ok) => Ph1EmoGuideResponse::EmoGuideProfileBuildOk(ok),
            Err(_) => self.refuse(
                EmoGuideCapabilityId::EmoGuideProfileBuild,
                reason_codes::PH1_EMO_GUIDE_INTERNAL_PIPELINE_ERROR,
                "failed to construct emo-guide build output",
            ),
        }
    }

    fn run_profile_validate(&self, req: &EmoGuideProfileValidateRequest) -> Ph1EmoGuideResponse {
        if req.verified_speaker_id.trim().is_empty() {
            return self.refuse(
                EmoGuideCapabilityId::EmoGuideProfileValidate,
                reason_codes::PH1_EMO_GUIDE_IDENTITY_REQUIRED,
                "verified speaker id is required",
            );
        }

        if req.interaction_signals.interaction_count > self.config.max_interactions {
            return self.refuse(
                EmoGuideCapabilityId::EmoGuideProfileValidate,
                reason_codes::PH1_EMO_GUIDE_BUDGET_EXCEEDED,
                "interaction_count exceeds runtime budget",
            );
        }

        let expected = match classify_profile(
            &req.interaction_signals,
            self.config.max_modifiers,
            self.config.min_stability_window_turns,
        ) {
            Ok(profile) => profile,
            Err(_) => {
                return self.refuse(
                    EmoGuideCapabilityId::EmoGuideProfileValidate,
                    reason_codes::PH1_EMO_GUIDE_INTERNAL_PIPELINE_ERROR,
                    "failed to reconstruct emo-guide profile",
                )
            }
        };

        let mut diagnostics: Vec<String> = Vec::new();
        if req.proposed_profile.style_profile_ref != expected.style_profile_ref {
            diagnostics.push("style_profile_ref_mismatch".to_string());
        }
        if req.proposed_profile.modifiers != expected.modifiers {
            diagnostics.push("style_modifiers_mismatch".to_string());
        }
        if req.proposed_profile.stability_window_turns != expected.stability_window_turns {
            diagnostics.push("stability_window_turns_mismatch".to_string());
        }

        let (validation_status, reason_code) = if diagnostics.is_empty() {
            (
                EmoGuideValidationStatus::Ok,
                reason_codes::PH1_EMO_GUIDE_OK_PROFILE_VALIDATE,
            )
        } else {
            (
                EmoGuideValidationStatus::Fail,
                reason_codes::PH1_EMO_GUIDE_VALIDATION_FAILED,
            )
        };

        match EmoGuideProfileValidateOk::v1(
            reason_code,
            validation_status,
            diagnostics,
            true,
            true,
            true,
        ) {
            Ok(ok) => Ph1EmoGuideResponse::EmoGuideProfileValidateOk(ok),
            Err(_) => self.refuse(
                EmoGuideCapabilityId::EmoGuideProfileValidate,
                reason_codes::PH1_EMO_GUIDE_INTERNAL_PIPELINE_ERROR,
                "failed to construct emo-guide validate output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: EmoGuideCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1EmoGuideResponse {
        let refuse = EmoGuideRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("EmoGuideRefuse::v1 must construct for static messages");
        Ph1EmoGuideResponse::Refuse(refuse)
    }
}

fn capability_from_request(req: &Ph1EmoGuideRequest) -> EmoGuideCapabilityId {
    match req {
        Ph1EmoGuideRequest::EmoGuideProfileBuild(_) => EmoGuideCapabilityId::EmoGuideProfileBuild,
        Ph1EmoGuideRequest::EmoGuideProfileValidate(_) => {
            EmoGuideCapabilityId::EmoGuideProfileValidate
        }
    }
}

fn classify_profile(
    signals: &EmoGuideInteractionSignals,
    max_modifiers: u8,
    min_stability_window_turns: u16,
) -> Result<EmoGuideProfile, selene_kernel_contracts::ContractViolation> {
    let dominant_score = (signals.assertive_events as u32 * 2)
        + signals.interruption_events as u32
        + signals.correction_events as u32;
    let gentle_score =
        (signals.cooperative_events as u32 * 2) + (signals.interaction_count as u32 / 2);

    let style_profile_ref = if dominant_score > gentle_score {
        StyleProfileRef::Dominant
    } else {
        StyleProfileRef::Gentle
    };

    let mut modifiers: Vec<StyleModifier> = Vec::new();
    if percent(signals.interruption_events, signals.interaction_count) >= 35
        || percent(signals.correction_events, signals.interaction_count) >= 30
    {
        modifiers.push(StyleModifier::Brief);
    }
    if percent(signals.cooperative_events, signals.interaction_count) >= 45 {
        modifiers.push(StyleModifier::Warm);
    }
    if percent(signals.correction_events, signals.interaction_count) >= 20
        || percent(signals.assertive_events, signals.interaction_count) >= 55
    {
        modifiers.push(StyleModifier::Formal);
    }

    modifiers.sort_by_key(|m| modifier_rank(*m));
    modifiers.dedup();
    modifiers.truncate(max_modifiers as usize);

    let stability_window_turns = signals
        .interaction_count
        .max(min_stability_window_turns)
        .min(120);

    EmoGuideProfile::v1(style_profile_ref, modifiers, stability_window_turns)
}

fn percent(value: u16, total: u16) -> u16 {
    if total == 0 {
        return 0;
    }
    ((value as u32 * 100) / total as u32) as u16
}

fn modifier_rank(m: StyleModifier) -> u8 {
    match m {
        StyleModifier::Brief => 0,
        StyleModifier::Warm => 1,
        StyleModifier::Formal => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1emoguide::{
        EmoGuideInteractionSignals, EmoGuideProfileBuildRequest, EmoGuideProfileValidateRequest,
        EmoGuideRequestEnvelope,
    };
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};

    fn runtime() -> Ph1EmoGuideRuntime {
        Ph1EmoGuideRuntime::new(Ph1EmoGuideConfig::mvp_v1())
    }

    fn envelope() -> EmoGuideRequestEnvelope {
        EmoGuideRequestEnvelope::v1(CorrelationId(1401), TurnId(101), 120, 3, 8).unwrap()
    }

    fn dominant_signals() -> EmoGuideInteractionSignals {
        EmoGuideInteractionSignals::v1(20, 5, 7, 12, 3).unwrap()
    }

    fn gentle_signals() -> EmoGuideInteractionSignals {
        EmoGuideInteractionSignals::v1(20, 1, 1, 4, 14).unwrap()
    }

    #[test]
    fn at_emo_guide_01_dominant_profile_emitted_for_assertive_pattern() {
        let req = Ph1EmoGuideRequest::EmoGuideProfileBuild(
            EmoGuideProfileBuildRequest::v1(
                envelope(),
                "speaker_dom".to_string(),
                dominant_signals(),
                None,
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        assert!(out.validate().is_ok());
        match out {
            Ph1EmoGuideResponse::EmoGuideProfileBuildOk(ok) => {
                assert_eq!(ok.profile.style_profile_ref, StyleProfileRef::Dominant);
                assert!(ok.profile.modifiers.contains(&StyleModifier::Brief));
            }
            _ => panic!("expected EmoGuideProfileBuildOk"),
        }
    }

    #[test]
    fn at_emo_guide_02_gentle_profile_emitted_for_cooperative_pattern() {
        let req = Ph1EmoGuideRequest::EmoGuideProfileBuild(
            EmoGuideProfileBuildRequest::v1(
                envelope(),
                "speaker_gentle".to_string(),
                gentle_signals(),
                Some("emo_core_snap_01".to_string()),
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        assert!(out.validate().is_ok());
        match out {
            Ph1EmoGuideResponse::EmoGuideProfileBuildOk(ok) => {
                assert_eq!(ok.profile.style_profile_ref, StyleProfileRef::Gentle);
                assert!(ok.profile.modifiers.contains(&StyleModifier::Warm));
            }
            _ => panic!("expected EmoGuideProfileBuildOk"),
        }
    }

    #[test]
    fn at_emo_guide_03_validation_fails_on_profile_drift() {
        let build = match runtime().run(&Ph1EmoGuideRequest::EmoGuideProfileBuild(
            EmoGuideProfileBuildRequest::v1(
                envelope(),
                "speaker_1".to_string(),
                dominant_signals(),
                None,
            )
            .unwrap(),
        )) {
            Ph1EmoGuideResponse::EmoGuideProfileBuildOk(ok) => ok,
            _ => panic!("expected build output"),
        };

        let mut drifted_profile = build.profile.clone();
        drifted_profile.style_profile_ref = StyleProfileRef::Gentle;

        let validate_req = Ph1EmoGuideRequest::EmoGuideProfileValidate(
            EmoGuideProfileValidateRequest::v1(
                envelope(),
                "speaker_1".to_string(),
                dominant_signals(),
                None,
                drifted_profile,
            )
            .unwrap(),
        );

        let out = runtime().run(&validate_req);
        assert!(out.validate().is_ok());
        match out {
            Ph1EmoGuideResponse::EmoGuideProfileValidateOk(ok) => {
                assert_eq!(ok.validation_status, EmoGuideValidationStatus::Fail);
                assert!(!ok.diagnostics.is_empty());
            }
            _ => panic!("expected EmoGuideProfileValidateOk"),
        }
    }

    #[test]
    fn at_emo_guide_04_tone_only_guard_flags_always_true() {
        let build = runtime().run(&Ph1EmoGuideRequest::EmoGuideProfileBuild(
            EmoGuideProfileBuildRequest::v1(
                envelope(),
                "speaker_guard".to_string(),
                gentle_signals(),
                None,
            )
            .unwrap(),
        ));

        match build {
            Ph1EmoGuideResponse::EmoGuideProfileBuildOk(ok) => {
                assert!(ok.tone_only);
                assert!(ok.no_meaning_drift);
                assert!(ok.no_execution_authority);
                assert!(ok.auditable);
                assert!(ok.reversible);
            }
            _ => panic!("expected EmoGuideProfileBuildOk"),
        }
    }
}
