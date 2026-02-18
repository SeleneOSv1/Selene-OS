#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1emocore::{
    EmoAuditEventResult, EmoAuditEventStatus, EmoClassifyProfileCommitRequest,
    EmoClassifyProfileResult, EmoCoreCapabilityId, EmoCoreOutcome, EmoCoreRequest,
    EmoPersonalityLockStatus, EmoPersonalityType, EmoPrivacyCommand,
    EmoPrivacyCommandCommitRequest, EmoPrivacyCommandResult, EmoPrivacyState,
    EmoReevaluateProfileCommitRequest, EmoReevaluateProfileResult, EmoSignalBundle,
    EmoSnapshotCaptureCommitRequest, EmoSnapshotCaptureResult, EmoSnapshotStatus, EmoStyleBucket,
    EmoToneGuidance, EmoToneGuidanceDraftRequest, EmoToneGuidanceResult, EmoTonePacing,
    EmoVoiceStyleProfile, Ph1EmoCoreOk, Ph1EmoCoreRefuse, Ph1EmoCoreRequest, Ph1EmoCoreResponse,
};
use selene_kernel_contracts::ph1tts::{StyleModifier, StyleProfileRef};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.EMO.CORE reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_EMO_CORE_OK_PROFILE_CLASSIFIED: ReasonCodeId = ReasonCodeId(0x4543_0001);
    pub const PH1_EMO_CORE_OK_PROFILE_REEVALUATED: ReasonCodeId = ReasonCodeId(0x4543_0002);
    pub const PH1_EMO_CORE_OK_PRIVACY_APPLIED: ReasonCodeId = ReasonCodeId(0x4543_0003);
    pub const PH1_EMO_CORE_OK_TONE_GUIDANCE_EMITTED: ReasonCodeId = ReasonCodeId(0x4543_0004);
    pub const PH1_EMO_CORE_OK_SNAPSHOT_CAPTURED: ReasonCodeId = ReasonCodeId(0x4543_0005);
    pub const PH1_EMO_CORE_OK_SNAPSHOT_DEFERRED: ReasonCodeId = ReasonCodeId(0x4543_0006);
    pub const PH1_EMO_CORE_OK_AUDIT_RECORDED: ReasonCodeId = ReasonCodeId(0x4543_0007);

    pub const PH1_EMO_CORE_FAIL_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x4543_00F1);
    pub const PH1_EMO_CORE_FAIL_CONSENT_REQUIRED: ReasonCodeId = ReasonCodeId(0x4543_00F2);
    pub const PH1_EMO_CORE_FAIL_IDENTITY_REQUIRED: ReasonCodeId = ReasonCodeId(0x4543_00F3);
    pub const PH1_EMO_CORE_FAIL_PRIVACY_CONFIRMATION_REQUIRED: ReasonCodeId =
        ReasonCodeId(0x4543_00F4);
    pub const PH1_EMO_CORE_FAIL_SCOPE_VIOLATION: ReasonCodeId = ReasonCodeId(0x4543_00F5);
    pub const PH1_EMO_CORE_FAIL_INTERNAL: ReasonCodeId = ReasonCodeId(0x4543_00F6);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1EmoCoreConfig {
    pub classify_margin: i16,
}

impl Ph1EmoCoreConfig {
    pub fn mvp_v1() -> Self {
        Self {
            classify_margin: 15,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1EmoCoreRuntime {
    config: Ph1EmoCoreConfig,
}

impl Ph1EmoCoreRuntime {
    pub fn new(config: Ph1EmoCoreConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1EmoCoreRequest) -> Ph1EmoCoreResponse {
        if req.validate().is_err() {
            return refuse(
                req.request.capability_id(),
                req.simulation_id.clone(),
                reason_codes::PH1_EMO_CORE_FAIL_SCHEMA_INVALID,
                "ph1emo core request failed contract validation",
            );
        }

        match &req.request {
            EmoCoreRequest::ClassifyProfileCommit(r) => self.run_classify(req, r),
            EmoCoreRequest::ReevaluateProfileCommit(r) => self.run_reevaluate(req, r),
            EmoCoreRequest::PrivacyCommandCommit(r) => self.run_privacy(req, r),
            EmoCoreRequest::ToneGuidanceDraft(r) => self.run_tone_guidance(req, r),
            EmoCoreRequest::SnapshotCaptureCommit(r) => self.run_snapshot(req, r),
            EmoCoreRequest::AuditEventCommit(r) => self.run_audit_event(req, r),
        }
    }

    fn run_classify(
        &self,
        req: &Ph1EmoCoreRequest,
        r: &EmoClassifyProfileCommitRequest,
    ) -> Ph1EmoCoreResponse {
        if !r.consent_asserted {
            return refuse(
                req.request.capability_id(),
                req.simulation_id.clone(),
                reason_codes::PH1_EMO_CORE_FAIL_CONSENT_REQUIRED,
                "consent is required for profile classification",
            );
        }
        if !r.identity_verified {
            return refuse(
                req.request.capability_id(),
                req.simulation_id.clone(),
                reason_codes::PH1_EMO_CORE_FAIL_IDENTITY_REQUIRED,
                "identity must be verified for profile classification",
            );
        }

        let personality_type = classify_personality(&r.signals, self.config.classify_margin);
        let personality_lock_status = match personality_type {
            EmoPersonalityType::Undetermined => EmoPersonalityLockStatus::ReevalDue,
            _ => EmoPersonalityLockStatus::Locked,
        };
        let voice_style_profile = style_profile_for_personality(personality_type);

        ok(
            req.request.capability_id(),
            req.simulation_id.clone(),
            reason_codes::PH1_EMO_CORE_OK_PROFILE_CLASSIFIED,
            EmoCoreOutcome::ClassifyProfile(EmoClassifyProfileResult {
                requester_user_id: r.requester_user_id.clone(),
                personality_type,
                personality_lock_status,
                voice_style_profile,
                reason_code: reason_codes::PH1_EMO_CORE_OK_PROFILE_CLASSIFIED,
            }),
        )
    }

    fn run_reevaluate(
        &self,
        req: &Ph1EmoCoreRequest,
        r: &EmoReevaluateProfileCommitRequest,
    ) -> Ph1EmoCoreResponse {
        if !r.consent_asserted {
            return refuse(
                req.request.capability_id(),
                req.simulation_id.clone(),
                reason_codes::PH1_EMO_CORE_FAIL_CONSENT_REQUIRED,
                "consent is required for profile re-evaluation",
            );
        }
        if !r.identity_verified {
            return refuse(
                req.request.capability_id(),
                req.simulation_id.clone(),
                reason_codes::PH1_EMO_CORE_FAIL_IDENTITY_REQUIRED,
                "identity must be verified for profile re-evaluation",
            );
        }

        let hash = deterministic_hash(&r.signals_window_ref);
        let personality_type = match hash % 3 {
            0 => EmoPersonalityType::Passive,
            1 => EmoPersonalityType::Domineering,
            _ => EmoPersonalityType::Undetermined,
        };
        let personality_lock_status = match hash % 4 {
            0 => EmoPersonalityLockStatus::ReevalChanged,
            1 => EmoPersonalityLockStatus::ReevalConfirmed,
            2 => EmoPersonalityLockStatus::ReevalDue,
            _ => EmoPersonalityLockStatus::Locked,
        };

        ok(
            req.request.capability_id(),
            req.simulation_id.clone(),
            reason_codes::PH1_EMO_CORE_OK_PROFILE_REEVALUATED,
            EmoCoreOutcome::ReevaluateProfile(EmoReevaluateProfileResult {
                requester_user_id: r.requester_user_id.clone(),
                personality_type,
                personality_lock_status,
                reason_code: reason_codes::PH1_EMO_CORE_OK_PROFILE_REEVALUATED,
            }),
        )
    }

    fn run_privacy(
        &self,
        req: &Ph1EmoCoreRequest,
        r: &EmoPrivacyCommandCommitRequest,
    ) -> Ph1EmoCoreResponse {
        if !r.identity_verified {
            return refuse(
                req.request.capability_id(),
                req.simulation_id.clone(),
                reason_codes::PH1_EMO_CORE_FAIL_IDENTITY_REQUIRED,
                "identity must be verified for privacy command updates",
            );
        }

        if is_destructive_privacy_command(r.privacy_command) && !r.confirmation_asserted {
            return refuse(
                req.request.capability_id(),
                req.simulation_id.clone(),
                reason_codes::PH1_EMO_CORE_FAIL_PRIVACY_CONFIRMATION_REQUIRED,
                "destructive privacy command requires explicit confirmation",
            );
        }

        let privacy_state = match r.privacy_command {
            EmoPrivacyCommand::ForgetThisKey => EmoPrivacyState::KeepActive,
            EmoPrivacyCommand::ForgetAll => EmoPrivacyState::DoNotRemember,
            EmoPrivacyCommand::DoNotRemember => EmoPrivacyState::DoNotRemember,
            EmoPrivacyCommand::RecallOnly => EmoPrivacyState::RecallOnly,
            EmoPrivacyCommand::KeepActive => EmoPrivacyState::KeepActive,
            EmoPrivacyCommand::Archive => EmoPrivacyState::Archive,
        };

        ok(
            req.request.capability_id(),
            req.simulation_id.clone(),
            reason_codes::PH1_EMO_CORE_OK_PRIVACY_APPLIED,
            EmoCoreOutcome::PrivacyCommand(EmoPrivacyCommandResult {
                requester_user_id: r.requester_user_id.clone(),
                privacy_state,
                reason_code: reason_codes::PH1_EMO_CORE_OK_PRIVACY_APPLIED,
            }),
        )
    }

    fn run_tone_guidance(
        &self,
        req: &Ph1EmoCoreRequest,
        r: &EmoToneGuidanceDraftRequest,
    ) -> Ph1EmoCoreResponse {
        let style_profile_ref = if let Some(snapshot_ref) = &r.profile_snapshot_ref {
            if snapshot_ref.to_ascii_lowercase().contains("dom") {
                StyleProfileRef::Dominant
            } else if snapshot_ref.to_ascii_lowercase().contains("gentle")
                || snapshot_ref.to_ascii_lowercase().contains("passive")
            {
                StyleProfileRef::Gentle
            } else {
                style_profile_from_signal(&r.signals)
            }
        } else {
            style_profile_from_signal(&r.signals)
        };

        let mut modifiers: Vec<StyleModifier> = Vec::new();
        if r.signals.assertive_score >= 70 || r.signals.anger_score >= 65 {
            modifiers.push(StyleModifier::Brief);
            modifiers.push(StyleModifier::Formal);
        }
        if r.signals.warmth_signal >= 65 && r.signals.anger_score <= 45 {
            modifiers.push(StyleModifier::Warm);
        }
        modifiers.sort_by_key(|m| style_modifier_rank(*m));
        modifiers.dedup();
        modifiers.truncate(3);

        let pacing_guidance = if r.signals.assertive_score >= 75 {
            EmoTonePacing::Fast
        } else if r.signals.warmth_signal >= 80 && r.signals.assertive_score <= 40 {
            EmoTonePacing::Slow
        } else {
            EmoTonePacing::Balanced
        };

        let directness_level =
            ((r.signals.assertive_score as u16 + r.signals.anger_score as u16) / 2) as u8;
        let empathy_level = ((r.signals.warmth_signal as u16
            + (100u16.saturating_sub(r.signals.distress_score as u16)))
            / 2) as u8;

        let tone_guidance = match EmoToneGuidance::v1(
            style_profile_ref,
            modifiers,
            pacing_guidance,
            directness_level,
            empathy_level,
        ) {
            Ok(v) => v,
            Err(_) => {
                return refuse(
                    req.request.capability_id(),
                    req.simulation_id.clone(),
                    reason_codes::PH1_EMO_CORE_FAIL_INTERNAL,
                    "failed to construct tone guidance output",
                );
            }
        };

        ok(
            req.request.capability_id(),
            req.simulation_id.clone(),
            reason_codes::PH1_EMO_CORE_OK_TONE_GUIDANCE_EMITTED,
            EmoCoreOutcome::ToneGuidance(EmoToneGuidanceResult {
                requester_user_id: r.requester_user_id.clone(),
                tone_guidance,
                reason_code: reason_codes::PH1_EMO_CORE_OK_TONE_GUIDANCE_EMITTED,
            }),
        )
    }

    fn run_snapshot(
        &self,
        req: &Ph1EmoCoreRequest,
        r: &EmoSnapshotCaptureCommitRequest,
    ) -> Ph1EmoCoreResponse {
        if !r.consent_asserted || !r.identity_verified {
            return ok(
                req.request.capability_id(),
                req.simulation_id.clone(),
                reason_codes::PH1_EMO_CORE_OK_SNAPSHOT_DEFERRED,
                EmoCoreOutcome::SnapshotCapture(EmoSnapshotCaptureResult {
                    requester_user_id: r.requester_user_id.clone(),
                    snapshot_ref: None,
                    snapshot_status: EmoSnapshotStatus::Defer,
                    reason_code: reason_codes::PH1_EMO_CORE_OK_SNAPSHOT_DEFERRED,
                }),
            );
        }

        let fingerprint = deterministic_hash(&format!(
            "{}:{}:{}:{}:{}",
            r.onboarding_session_id.as_str(),
            r.requester_user_id.as_str(),
            r.signals.assertive_score,
            r.signals.warmth_signal,
            r.idempotency_key
        ));
        let snapshot_ref = format!(
            "emo_snap_{}_{}",
            r.onboarding_session_id.as_str(),
            fingerprint % 10_000
        );

        ok(
            req.request.capability_id(),
            req.simulation_id.clone(),
            reason_codes::PH1_EMO_CORE_OK_SNAPSHOT_CAPTURED,
            EmoCoreOutcome::SnapshotCapture(EmoSnapshotCaptureResult {
                requester_user_id: r.requester_user_id.clone(),
                snapshot_ref: Some(snapshot_ref),
                snapshot_status: EmoSnapshotStatus::Complete,
                reason_code: reason_codes::PH1_EMO_CORE_OK_SNAPSHOT_CAPTURED,
            }),
        )
    }

    fn run_audit_event(
        &self,
        req: &Ph1EmoCoreRequest,
        r: &selene_kernel_contracts::ph1emocore::EmoAuditEventCommitRequest,
    ) -> Ph1EmoCoreResponse {
        let event_seed = format!(
            "{}:{}:{}:{}:{}",
            req.correlation_id.0,
            req.turn_id.0,
            r.requester_user_id.as_str(),
            r.event_type,
            r.idempotency_key
        );
        let event_id = format!("emo_evt_{:08x}", deterministic_hash(&event_seed));

        ok(
            req.request.capability_id(),
            req.simulation_id.clone(),
            reason_codes::PH1_EMO_CORE_OK_AUDIT_RECORDED,
            EmoCoreOutcome::AuditEvent(EmoAuditEventResult {
                event_id,
                status: EmoAuditEventStatus::Recorded,
            }),
        )
    }
}

fn style_profile_from_signal(signals: &EmoSignalBundle) -> StyleProfileRef {
    if signals.assertive_score >= signals.warmth_signal {
        StyleProfileRef::Dominant
    } else {
        StyleProfileRef::Gentle
    }
}

fn classify_personality(signals: &EmoSignalBundle, margin: i16) -> EmoPersonalityType {
    let dominant_score = signals.assertive_score as i16
        + signals.anger_score as i16
        + (signals.distress_score as i16 / 2);
    let passive_score = signals.warmth_signal as i16 + ((100 - signals.anger_score) as i16 / 2);
    if dominant_score >= passive_score + margin {
        EmoPersonalityType::Domineering
    } else if passive_score >= dominant_score + margin {
        EmoPersonalityType::Passive
    } else {
        EmoPersonalityType::Undetermined
    }
}

fn style_profile_for_personality(v: EmoPersonalityType) -> EmoVoiceStyleProfile {
    match v {
        EmoPersonalityType::Passive => EmoVoiceStyleProfile::v1(
            EmoStyleBucket::Low,
            EmoStyleBucket::Low,
            EmoStyleBucket::High,
        )
        .unwrap(),
        EmoPersonalityType::Domineering => EmoVoiceStyleProfile::v1(
            EmoStyleBucket::High,
            EmoStyleBucket::High,
            EmoStyleBucket::Low,
        )
        .unwrap(),
        EmoPersonalityType::Undetermined => EmoVoiceStyleProfile::v1(
            EmoStyleBucket::Medium,
            EmoStyleBucket::Medium,
            EmoStyleBucket::Medium,
        )
        .unwrap(),
    }
}

fn is_destructive_privacy_command(v: EmoPrivacyCommand) -> bool {
    matches!(
        v,
        EmoPrivacyCommand::ForgetThisKey
            | EmoPrivacyCommand::ForgetAll
            | EmoPrivacyCommand::Archive
    )
}

fn deterministic_hash(v: &str) -> u32 {
    let mut h: u32 = 0x811C_9DC5;
    for b in v.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(16_777_619);
    }
    h
}

fn style_modifier_rank(v: StyleModifier) -> u8 {
    match v {
        StyleModifier::Brief => 0,
        StyleModifier::Warm => 1,
        StyleModifier::Formal => 2,
    }
}

fn ok(
    capability_id: EmoCoreCapabilityId,
    simulation_id: String,
    reason_code: ReasonCodeId,
    outcome: EmoCoreOutcome,
) -> Ph1EmoCoreResponse {
    match Ph1EmoCoreOk::v1(
        capability_id,
        simulation_id,
        reason_code,
        outcome,
        true,
        true,
        true,
    ) {
        Ok(v) => Ph1EmoCoreResponse::Ok(v),
        Err(_) => refuse(
            capability_id,
            "INTERNAL_SIMULATION".to_string(),
            reason_codes::PH1_EMO_CORE_FAIL_INTERNAL,
            "failed to construct ph1emo core ok response",
        ),
    }
}

fn refuse(
    capability_id: EmoCoreCapabilityId,
    simulation_id: String,
    reason_code: ReasonCodeId,
    message: &'static str,
) -> Ph1EmoCoreResponse {
    let v = Ph1EmoCoreRefuse::v1(
        capability_id,
        simulation_id,
        reason_code,
        message.to_string(),
    )
    .expect("Ph1EmoCoreRefuse::v1 must construct for static messages");
    Ph1EmoCoreResponse::Refuse(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1emocore::{
        EmoAuditEventCommitRequest, EmoCoreSimulationType, EMO_SIM_001, EMO_SIM_003, EMO_SIM_004,
        EMO_SIM_005, EMO_SIM_006, PH1EMOCORE_CONTRACT_VERSION,
    };
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1onb::OnboardingSessionId;
    use selene_kernel_contracts::ph1position::TenantId;
    use selene_kernel_contracts::MonotonicTimeNs;
    use selene_kernel_contracts::Validate;

    fn runtime() -> Ph1EmoCoreRuntime {
        Ph1EmoCoreRuntime::new(Ph1EmoCoreConfig::mvp_v1())
    }

    fn tenant() -> TenantId {
        TenantId::new("tenant_emo").unwrap()
    }

    fn user() -> selene_kernel_contracts::ph1_voice_id::UserId {
        selene_kernel_contracts::ph1_voice_id::UserId::new("user_emo").unwrap()
    }

    fn signals(assertive: u8, warmth: u8) -> EmoSignalBundle {
        EmoSignalBundle::v1(assertive, 20, 20, warmth).unwrap()
    }

    #[test]
    fn at_emo_core_01_classify_profile_commit_emits_tone_only_profile() {
        let req = Ph1EmoCoreRequest {
            schema_version: PH1EMOCORE_CONTRACT_VERSION,
            correlation_id: CorrelationId(401),
            turn_id: TurnId(21),
            now: MonotonicTimeNs(100),
            simulation_id: EMO_SIM_001.to_string(),
            simulation_type: EmoCoreSimulationType::Commit,
            request: EmoCoreRequest::ClassifyProfileCommit(EmoClassifyProfileCommitRequest {
                tenant_id: tenant(),
                requester_user_id: user(),
                session_id: "session_emo".to_string(),
                consent_asserted: true,
                identity_verified: true,
                signals: signals(90, 20),
                idempotency_key: "idem_emo_core_001".to_string(),
            }),
        };
        let out = runtime().run(&req);
        assert!(out.validate().is_ok());
        match out {
            Ph1EmoCoreResponse::Ok(ok) => {
                assert!(ok.tone_only);
                assert!(ok.no_meaning_drift);
                assert!(ok.no_execution_authority);
                match ok.outcome {
                    EmoCoreOutcome::ClassifyProfile(v) => {
                        assert_eq!(v.personality_type, EmoPersonalityType::Domineering);
                    }
                    _ => panic!("expected classify profile outcome"),
                }
            }
            _ => panic!("expected ok response"),
        }
    }

    #[test]
    fn at_emo_core_02_snapshot_capture_commit_is_non_blocking_when_identity_missing() {
        let req = Ph1EmoCoreRequest {
            schema_version: PH1EMOCORE_CONTRACT_VERSION,
            correlation_id: CorrelationId(402),
            turn_id: TurnId(22),
            now: MonotonicTimeNs(101),
            simulation_id: EMO_SIM_005.to_string(),
            simulation_type: EmoCoreSimulationType::Commit,
            request: EmoCoreRequest::SnapshotCaptureCommit(EmoSnapshotCaptureCommitRequest {
                tenant_id: tenant(),
                requester_user_id: user(),
                onboarding_session_id: OnboardingSessionId::new("onb_emo_001").unwrap(),
                consent_asserted: true,
                identity_verified: false,
                signals: signals(40, 60),
                idempotency_key: "idem_emo_core_002".to_string(),
            }),
        };
        let out = runtime().run(&req);
        assert!(out.validate().is_ok());
        match out {
            Ph1EmoCoreResponse::Ok(ok) => match ok.outcome {
                EmoCoreOutcome::SnapshotCapture(v) => {
                    assert_eq!(v.snapshot_status, EmoSnapshotStatus::Defer);
                    assert!(v.snapshot_ref.is_none());
                }
                _ => panic!("expected snapshot outcome"),
            },
            _ => panic!("expected ok response"),
        }
    }

    #[test]
    fn at_emo_core_03_privacy_command_commit_requires_confirmation_for_destructive_commands() {
        let req = Ph1EmoCoreRequest {
            schema_version: PH1EMOCORE_CONTRACT_VERSION,
            correlation_id: CorrelationId(403),
            turn_id: TurnId(23),
            now: MonotonicTimeNs(102),
            simulation_id: EMO_SIM_003.to_string(),
            simulation_type: EmoCoreSimulationType::Commit,
            request: EmoCoreRequest::PrivacyCommandCommit(EmoPrivacyCommandCommitRequest {
                tenant_id: tenant(),
                requester_user_id: user(),
                session_id: "session_emo".to_string(),
                identity_verified: true,
                privacy_command: EmoPrivacyCommand::ForgetAll,
                target_key: None,
                confirmation_asserted: false,
                idempotency_key: "idem_emo_core_003".to_string(),
            }),
        };
        let out = runtime().run(&req);
        match out {
            Ph1EmoCoreResponse::Refuse(r) => {
                assert_eq!(
                    r.reason_code,
                    reason_codes::PH1_EMO_CORE_FAIL_PRIVACY_CONFIRMATION_REQUIRED
                );
            }
            _ => panic!("expected refuse response"),
        }
    }

    #[test]
    fn at_emo_core_04_tone_guidance_draft_is_output_only() {
        let req = Ph1EmoCoreRequest {
            schema_version: PH1EMOCORE_CONTRACT_VERSION,
            correlation_id: CorrelationId(404),
            turn_id: TurnId(24),
            now: MonotonicTimeNs(103),
            simulation_id: EMO_SIM_004.to_string(),
            simulation_type: EmoCoreSimulationType::Draft,
            request: EmoCoreRequest::ToneGuidanceDraft(EmoToneGuidanceDraftRequest {
                tenant_id: tenant(),
                requester_user_id: Some(user()),
                profile_snapshot_ref: Some("gentle_profile_ref".to_string()),
                signals: signals(20, 90),
            }),
        };
        let out = runtime().run(&req);
        assert!(out.validate().is_ok());
        match out {
            Ph1EmoCoreResponse::Ok(ok) => match ok.outcome {
                EmoCoreOutcome::ToneGuidance(v) => {
                    assert_eq!(v.tone_guidance.style_profile_ref, StyleProfileRef::Gentle);
                }
                _ => panic!("expected tone guidance outcome"),
            },
            _ => panic!("expected ok response"),
        }
    }

    #[test]
    fn at_emo_core_05_audit_event_commit_emits_deterministic_event_id() {
        let req = Ph1EmoCoreRequest {
            schema_version: PH1EMOCORE_CONTRACT_VERSION,
            correlation_id: CorrelationId(405),
            turn_id: TurnId(25),
            now: MonotonicTimeNs(104),
            simulation_id: EMO_SIM_006.to_string(),
            simulation_type: EmoCoreSimulationType::Commit,
            request: EmoCoreRequest::AuditEventCommit(EmoAuditEventCommitRequest {
                tenant_id: tenant(),
                requester_user_id: user(),
                session_id: Some("session_emo".to_string()),
                event_type: "EMO_EVENT".to_string(),
                reason_codes: vec![ReasonCodeId(1), ReasonCodeId(2)],
                idempotency_key: "idem_emo_core_005".to_string(),
            }),
        };
        let out1 = runtime().run(&req);
        let out2 = runtime().run(&req);
        match (out1, out2) {
            (Ph1EmoCoreResponse::Ok(a), Ph1EmoCoreResponse::Ok(b)) => {
                let aid = match a.outcome {
                    EmoCoreOutcome::AuditEvent(v) => v.event_id,
                    _ => panic!("expected audit outcome"),
                };
                let bid = match b.outcome {
                    EmoCoreOutcome::AuditEvent(v) => v.event_id,
                    _ => panic!("expected audit outcome"),
                };
                assert_eq!(aid, bid);
            }
            _ => panic!("expected ok responses"),
        }
    }
}
