#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1persona::{
    PersonaBrevityRef, PersonaCapabilityId, PersonaDeliveryPolicyRef, PersonaPreferenceKey,
    PersonaPreferenceSignal, PersonaProfileBuildOk, PersonaProfileBuildRequest,
    PersonaProfileSnapshot, PersonaProfileValidateOk, PersonaProfileValidateRequest, PersonaRefuse,
    PersonaValidationStatus, Ph1PersonaRequest, Ph1PersonaResponse,
};
use selene_kernel_contracts::ph1tts::StyleProfileRef;
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.PERSONA reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_PERSONA_OK_PROFILE_BUILD: ReasonCodeId = ReasonCodeId(0x5052_0001);
    pub const PH1_PERSONA_OK_PROFILE_VALIDATE: ReasonCodeId = ReasonCodeId(0x5052_0002);

    pub const PH1_PERSONA_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x5052_00F1);
    pub const PH1_PERSONA_IDENTITY_REQUIRED: ReasonCodeId = ReasonCodeId(0x5052_00F2);
    pub const PH1_PERSONA_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x5052_00F3);
    pub const PH1_PERSONA_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x5052_00F4);
    pub const PH1_PERSONA_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5052_00F5);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1PersonaConfig {
    pub max_signals: u8,
    pub max_diagnostics: u8,
}

impl Ph1PersonaConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_signals: 16,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1PersonaRuntime {
    config: Ph1PersonaConfig,
}

impl Ph1PersonaRuntime {
    pub fn new(config: Ph1PersonaConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1PersonaRequest) -> Ph1PersonaResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_PERSONA_INPUT_SCHEMA_INVALID,
                "persona request failed contract validation",
            );
        }

        match req {
            Ph1PersonaRequest::PersonaProfileBuild(r) => self.run_profile_build(r),
            Ph1PersonaRequest::PersonaProfileValidate(r) => self.run_profile_validate(r),
        }
    }

    fn run_profile_build(&self, req: &PersonaProfileBuildRequest) -> Ph1PersonaResponse {
        if req.verified_user_id.trim().is_empty() || req.verified_speaker_id.trim().is_empty() {
            return self.refuse(
                PersonaCapabilityId::PersonaProfileBuild,
                reason_codes::PH1_PERSONA_IDENTITY_REQUIRED,
                "verified user and speaker ids are required",
            );
        }

        if req.preference_signals.len() > self.config.max_signals as usize {
            return self.refuse(
                PersonaCapabilityId::PersonaProfileBuild,
                reason_codes::PH1_PERSONA_BUDGET_EXCEEDED,
                "preference signal budget exceeded",
            );
        }

        let snapshot = match build_profile_snapshot(
            &req.verified_user_id,
            &req.verified_speaker_id,
            &req.preference_signals,
            req.correction_event_count,
            req.emo_guide_style_profile_ref,
            req.previous_snapshot_ref.as_deref(),
        ) {
            Ok(snapshot) => snapshot,
            Err(_) => {
                return self.refuse(
                    PersonaCapabilityId::PersonaProfileBuild,
                    reason_codes::PH1_PERSONA_INTERNAL_PIPELINE_ERROR,
                    "failed to build persona profile snapshot",
                )
            }
        };

        match PersonaProfileBuildOk::v1(
            reason_codes::PH1_PERSONA_OK_PROFILE_BUILD,
            snapshot,
            true,
            true,
            true,
            true,
        ) {
            Ok(ok) => Ph1PersonaResponse::PersonaProfileBuildOk(ok),
            Err(_) => self.refuse(
                PersonaCapabilityId::PersonaProfileBuild,
                reason_codes::PH1_PERSONA_INTERNAL_PIPELINE_ERROR,
                "failed to construct persona build output",
            ),
        }
    }

    fn run_profile_validate(&self, req: &PersonaProfileValidateRequest) -> Ph1PersonaResponse {
        if req.verified_user_id.trim().is_empty() || req.verified_speaker_id.trim().is_empty() {
            return self.refuse(
                PersonaCapabilityId::PersonaProfileValidate,
                reason_codes::PH1_PERSONA_IDENTITY_REQUIRED,
                "verified user and speaker ids are required",
            );
        }

        if req.preference_signals.len() > self.config.max_signals as usize {
            return self.refuse(
                PersonaCapabilityId::PersonaProfileValidate,
                reason_codes::PH1_PERSONA_BUDGET_EXCEEDED,
                "preference signal budget exceeded",
            );
        }

        let expected = match build_profile_snapshot(
            &req.verified_user_id,
            &req.verified_speaker_id,
            &req.preference_signals,
            req.correction_event_count,
            req.emo_guide_style_profile_ref,
            req.previous_snapshot_ref.as_deref(),
        ) {
            Ok(snapshot) => snapshot,
            Err(_) => {
                return self.refuse(
                    PersonaCapabilityId::PersonaProfileValidate,
                    reason_codes::PH1_PERSONA_INTERNAL_PIPELINE_ERROR,
                    "failed to rebuild expected persona profile snapshot",
                )
            }
        };

        let mut diagnostics = Vec::new();
        if req.proposed_profile_snapshot.style_profile_ref != expected.style_profile_ref {
            diagnostics.push("style_profile_ref_mismatch".to_string());
        }
        if req.proposed_profile_snapshot.delivery_policy_ref != expected.delivery_policy_ref {
            diagnostics.push("delivery_policy_ref_mismatch".to_string());
        }
        if req.proposed_profile_snapshot.brevity_ref != expected.brevity_ref {
            diagnostics.push("brevity_ref_mismatch".to_string());
        }
        if req.proposed_profile_snapshot.preferences_snapshot_ref
            != expected.preferences_snapshot_ref
        {
            diagnostics.push("preferences_snapshot_ref_mismatch".to_string());
        }

        diagnostics.truncate(min(
            self.config.max_diagnostics as usize,
            req.envelope.max_diagnostics as usize,
        ));

        let (validation_status, reason_code) = if diagnostics.is_empty() {
            (
                PersonaValidationStatus::Ok,
                reason_codes::PH1_PERSONA_OK_PROFILE_VALIDATE,
            )
        } else {
            (
                PersonaValidationStatus::Fail,
                reason_codes::PH1_PERSONA_VALIDATION_FAILED,
            )
        };

        match PersonaProfileValidateOk::v1(
            reason_code,
            validation_status,
            diagnostics,
            true,
            true,
            true,
            true,
        ) {
            Ok(ok) => Ph1PersonaResponse::PersonaProfileValidateOk(ok),
            Err(_) => self.refuse(
                PersonaCapabilityId::PersonaProfileValidate,
                reason_codes::PH1_PERSONA_INTERNAL_PIPELINE_ERROR,
                "failed to construct persona validate output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: PersonaCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1PersonaResponse {
        let refuse = PersonaRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("PersonaRefuse::v1 must construct for static messages");
        Ph1PersonaResponse::Refuse(refuse)
    }
}

fn capability_from_request(req: &Ph1PersonaRequest) -> PersonaCapabilityId {
    match req {
        Ph1PersonaRequest::PersonaProfileBuild(_) => PersonaCapabilityId::PersonaProfileBuild,
        Ph1PersonaRequest::PersonaProfileValidate(_) => PersonaCapabilityId::PersonaProfileValidate,
    }
}

fn build_profile_snapshot(
    verified_user_id: &str,
    verified_speaker_id: &str,
    preference_signals: &[PersonaPreferenceSignal],
    correction_event_count: u16,
    emo_guide_style_profile_ref: Option<StyleProfileRef>,
    previous_snapshot_ref: Option<&str>,
) -> Result<PersonaProfileSnapshot, selene_kernel_contracts::ContractViolation> {
    let style_profile_ref = select_style_profile(
        preference_signals,
        emo_guide_style_profile_ref,
        previous_snapshot_ref,
    );
    let delivery_policy_ref = select_delivery_policy(preference_signals, previous_snapshot_ref);
    let brevity_ref = select_brevity(preference_signals, previous_snapshot_ref);

    let preferences_snapshot_ref = build_preferences_snapshot_ref(
        verified_user_id,
        verified_speaker_id,
        style_profile_ref,
        delivery_policy_ref,
        brevity_ref,
        correction_event_count,
        preference_signals.is_empty(),
        previous_snapshot_ref.is_some(),
    );

    PersonaProfileSnapshot::v1(
        style_profile_ref,
        delivery_policy_ref,
        brevity_ref,
        preferences_snapshot_ref,
    )
}

fn select_style_profile(
    preference_signals: &[PersonaPreferenceSignal],
    emo_guide_style_profile_ref: Option<StyleProfileRef>,
    previous_snapshot_ref: Option<&str>,
) -> StyleProfileRef {
    let explicit = signal_value(preference_signals, PersonaPreferenceKey::ResponseToneTarget)
        .and_then(parse_style_profile);
    let previous = previous_snapshot_ref
        .and_then(|snapshot| parse_snapshot_component(snapshot, "s"))
        .and_then(parse_style_profile);

    explicit
        .or(emo_guide_style_profile_ref)
        .or(previous)
        .unwrap_or(StyleProfileRef::Gentle)
}

fn select_delivery_policy(
    preference_signals: &[PersonaPreferenceSignal],
    previous_snapshot_ref: Option<&str>,
) -> PersonaDeliveryPolicyRef {
    let explicit = signal_value(preference_signals, PersonaPreferenceKey::PrivacyPreference)
        .and_then(parse_delivery_policy)
        .or_else(|| {
            signal_value(
                preference_signals,
                PersonaPreferenceKey::ConfirmationPreference,
            )
            .and_then(parse_delivery_policy)
        });

    let previous = previous_snapshot_ref
        .and_then(|snapshot| parse_snapshot_component(snapshot, "d"))
        .and_then(parse_delivery_policy);

    explicit
        .or(previous)
        .unwrap_or(PersonaDeliveryPolicyRef::VoiceAllowed)
}

fn select_brevity(
    preference_signals: &[PersonaPreferenceSignal],
    previous_snapshot_ref: Option<&str>,
) -> PersonaBrevityRef {
    let explicit = signal_value(preference_signals, PersonaPreferenceKey::BrevityPreference)
        .and_then(parse_brevity_ref);
    let previous = previous_snapshot_ref
        .and_then(|snapshot| parse_snapshot_component(snapshot, "b"))
        .and_then(parse_brevity_ref);

    explicit.or(previous).unwrap_or(PersonaBrevityRef::Balanced)
}

fn signal_value<'a>(
    preference_signals: &'a [PersonaPreferenceSignal],
    key: PersonaPreferenceKey,
) -> Option<&'a str> {
    preference_signals
        .iter()
        .rev()
        .find(|signal| signal.key == key)
        .map(|signal| signal.value.as_str())
}

fn parse_snapshot_component<'a>(snapshot: &'a str, component: &str) -> Option<&'a str> {
    snapshot.split('|').find_map(|part| {
        let (key, value) = part.split_once(':')?;
        if key == component {
            Some(value)
        } else {
            None
        }
    })
}

fn parse_style_profile(value: &str) -> Option<StyleProfileRef> {
    let token = normalize_token(value);
    if token.contains("dominant") || token.contains("assertive") || token.contains("direct") {
        return Some(StyleProfileRef::Dominant);
    }
    if token.contains("gentle") || token.contains("calm") || token.contains("warm") {
        return Some(StyleProfileRef::Gentle);
    }
    None
}

fn parse_delivery_policy(value: &str) -> Option<PersonaDeliveryPolicyRef> {
    let token = normalize_token(value);
    if token.contains("silent") || token.contains("mute") {
        return Some(PersonaDeliveryPolicyRef::Silent);
    }
    if token.contains("text_only")
        || token.contains("private_text")
        || token == "text"
        || token == "quiet"
    {
        return Some(PersonaDeliveryPolicyRef::TextOnly);
    }
    if token.contains("voice") || token.contains("speak") {
        return Some(PersonaDeliveryPolicyRef::VoiceAllowed);
    }
    None
}

fn parse_brevity_ref(value: &str) -> Option<PersonaBrevityRef> {
    let token = normalize_token(value);
    if token.contains("brief") || token.contains("short") {
        return Some(PersonaBrevityRef::Brief);
    }
    if token.contains("detailed") || token.contains("verbose") || token.contains("long") {
        return Some(PersonaBrevityRef::Detailed);
    }
    if token.contains("balanced") || token.contains("normal") {
        return Some(PersonaBrevityRef::Balanced);
    }
    None
}

fn normalize_token(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut last_was_sep = false;
    for ch in value.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_was_sep = false;
            continue;
        }
        if !last_was_sep {
            out.push('_');
            last_was_sep = true;
        }
    }
    while out.ends_with('_') {
        out.pop();
    }
    out.trim_start_matches('_').to_string()
}

fn build_preferences_snapshot_ref(
    verified_user_id: &str,
    verified_speaker_id: &str,
    style_profile_ref: StyleProfileRef,
    delivery_policy_ref: PersonaDeliveryPolicyRef,
    brevity_ref: PersonaBrevityRef,
    correction_event_count: u16,
    no_signals_supplied: bool,
    previous_snapshot_present: bool,
) -> String {
    let identity_seed = format!("{verified_user_id}|{verified_speaker_id}");
    let checksum = checksum_hex(&identity_seed);
    let source_token = if no_signals_supplied {
        "carry"
    } else {
        "signal"
    };
    let prev_token = if previous_snapshot_present { "1" } else { "0" };

    format!(
        "pers:v1|u:{checksum}|s:{}|d:{}|b:{}|c:{}|src:{source_token}|p:{prev_token}",
        style_token(style_profile_ref),
        delivery_token(delivery_policy_ref),
        brevity_token(brevity_ref),
        correction_event_count
    )
}

fn checksum_hex(input: &str) -> String {
    let mut hash: u32 = 0x811C9DC5;
    for byte in input.bytes() {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(16_777_619);
    }
    format!("{hash:08x}")
}

fn style_token(value: StyleProfileRef) -> &'static str {
    match value {
        StyleProfileRef::Dominant => "dominant",
        StyleProfileRef::Gentle => "gentle",
    }
}

fn delivery_token(value: PersonaDeliveryPolicyRef) -> &'static str {
    match value {
        PersonaDeliveryPolicyRef::VoiceAllowed => "voice",
        PersonaDeliveryPolicyRef::TextOnly => "text",
        PersonaDeliveryPolicyRef::Silent => "silent",
    }
}

fn brevity_token(value: PersonaBrevityRef) -> &'static str {
    match value {
        PersonaBrevityRef::Brief => "brief",
        PersonaBrevityRef::Balanced => "balanced",
        PersonaBrevityRef::Detailed => "detailed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1persona::{
        PersonaRequestEnvelope, PH1PERSONA_CONTRACT_VERSION,
    };

    fn runtime() -> Ph1PersonaRuntime {
        Ph1PersonaRuntime::new(Ph1PersonaConfig::mvp_v1())
    }

    fn envelope() -> PersonaRequestEnvelope {
        PersonaRequestEnvelope::v1(CorrelationId(3801), TurnId(81), 8, 8).unwrap()
    }

    fn signal(
        key: PersonaPreferenceKey,
        value: &str,
        evidence_ref: &str,
    ) -> PersonaPreferenceSignal {
        PersonaPreferenceSignal::v1(key, value.to_string(), evidence_ref.to_string()).unwrap()
    }

    #[test]
    fn at_pers_runtime_01_profile_build_outputs_tone_only_guards() {
        let req = Ph1PersonaRequest::PersonaProfileBuild(
            PersonaProfileBuildRequest::v1(
                envelope(),
                "user_1".to_string(),
                "speaker_1".to_string(),
                vec![
                    signal(PersonaPreferenceKey::ResponseToneTarget, "gentle", "ev:1"),
                    signal(PersonaPreferenceKey::PrivacyPreference, "text_only", "ev:2"),
                ],
                2,
                Some(StyleProfileRef::Dominant),
                None,
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        match out {
            Ph1PersonaResponse::PersonaProfileBuildOk(ok) => {
                assert!(ok.auditable);
                assert!(ok.tone_only);
                assert!(ok.no_meaning_drift);
                assert!(ok.no_execution_authority);
                assert_eq!(
                    ok.profile_snapshot.style_profile_ref,
                    StyleProfileRef::Gentle
                );
                assert_eq!(
                    ok.profile_snapshot.delivery_policy_ref,
                    PersonaDeliveryPolicyRef::TextOnly
                );
            }
            _ => panic!("expected PersonaProfileBuildOk"),
        }
    }

    #[test]
    fn at_pers_runtime_02_profile_validate_detects_drift() {
        let build_req = Ph1PersonaRequest::PersonaProfileBuild(
            PersonaProfileBuildRequest::v1(
                envelope(),
                "user_2".to_string(),
                "speaker_2".to_string(),
                vec![signal(
                    PersonaPreferenceKey::BrevityPreference,
                    "balanced",
                    "ev:brevity",
                )],
                1,
                None,
                None,
            )
            .unwrap(),
        );

        let build_out = runtime().run(&build_req);
        let mut proposed = match build_out {
            Ph1PersonaResponse::PersonaProfileBuildOk(ok) => ok.profile_snapshot,
            _ => panic!("expected PersonaProfileBuildOk"),
        };
        proposed.brevity_ref = PersonaBrevityRef::Detailed;

        let validate_req = Ph1PersonaRequest::PersonaProfileValidate(
            PersonaProfileValidateRequest::v1(
                envelope(),
                "user_2".to_string(),
                "speaker_2".to_string(),
                vec![signal(
                    PersonaPreferenceKey::BrevityPreference,
                    "balanced",
                    "ev:brevity",
                )],
                1,
                None,
                None,
                proposed,
            )
            .unwrap(),
        );

        let out = runtime().run(&validate_req);
        match out {
            Ph1PersonaResponse::PersonaProfileValidateOk(ok) => {
                assert_eq!(ok.validation_status, PersonaValidationStatus::Fail);
                assert_eq!(ok.reason_code, reason_codes::PH1_PERSONA_VALIDATION_FAILED);
                assert!(ok
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic == "brevity_ref_mismatch"));
            }
            _ => panic!("expected PersonaProfileValidateOk"),
        }
    }

    #[test]
    fn at_pers_runtime_03_schema_invalid_request_fails_closed() {
        let mut invalid_signal =
            signal(PersonaPreferenceKey::PreferredLanguage, "en", "ev:language");
        invalid_signal.evidence_ref = " ".to_string();

        let req = Ph1PersonaRequest::PersonaProfileBuild(PersonaProfileBuildRequest {
            schema_version: PH1PERSONA_CONTRACT_VERSION,
            envelope: envelope(),
            verified_user_id: "user_3".to_string(),
            verified_speaker_id: "speaker_3".to_string(),
            preference_signals: vec![invalid_signal],
            correction_event_count: 0,
            emo_guide_style_profile_ref: None,
            previous_snapshot_ref: None,
        });

        let out = runtime().run(&req);
        match out {
            Ph1PersonaResponse::Refuse(refuse) => {
                assert_eq!(
                    refuse.reason_code,
                    reason_codes::PH1_PERSONA_INPUT_SCHEMA_INVALID
                );
            }
            _ => panic!("expected Refuse"),
        }
    }

    #[test]
    fn at_pers_runtime_04_explicit_style_preference_overrides_emo_guide_hint() {
        let req = Ph1PersonaRequest::PersonaProfileBuild(
            PersonaProfileBuildRequest::v1(
                envelope(),
                "user_4".to_string(),
                "speaker_4".to_string(),
                vec![signal(
                    PersonaPreferenceKey::ResponseToneTarget,
                    "gentle",
                    "ev:style",
                )],
                0,
                Some(StyleProfileRef::Dominant),
                None,
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        match out {
            Ph1PersonaResponse::PersonaProfileBuildOk(ok) => {
                assert_eq!(
                    ok.profile_snapshot.style_profile_ref,
                    StyleProfileRef::Gentle
                );
            }
            _ => panic!("expected PersonaProfileBuildOk"),
        }
    }
}
