#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use selene_kernel_contracts::ph1_voice_id::{
    DeviceTrustLevel, DiarizationSegment, IdentityConfidence, Ph1VoiceIdRequest,
    Ph1VoiceIdResponse, SpeakerAssertionOk, SpeakerAssertionUnknown, SpeakerId, SpeakerLabel,
    UserId,
};
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs, ReasonCodeId};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.VOICE.ID reason-code namespace. Values are placeholders until the global registry is formalized.
    pub const VID_FAIL_NO_SPEECH: ReasonCodeId = ReasonCodeId(0x5649_0001);
    pub const VID_FAIL_LOW_CONFIDENCE: ReasonCodeId = ReasonCodeId(0x5649_0002);
    pub const VID_FAIL_MULTI_SPEAKER_PRESENT: ReasonCodeId = ReasonCodeId(0x5649_0003);
    pub const VID_FAIL_ECHO_UNSAFE: ReasonCodeId = ReasonCodeId(0x5649_0004);
    pub const VID_FAIL_PROFILE_NOT_ENROLLED: ReasonCodeId = ReasonCodeId(0x5649_0005);
    pub const VID_ENROLLMENT_REQUIRED: ReasonCodeId = ReasonCodeId(0x5649_0006);
    pub const VID_REAUTH_REQUIRED: ReasonCodeId = ReasonCodeId(0x5649_0007);
    pub const VID_SPOOF_RISK: ReasonCodeId = ReasonCodeId(0x5649_0008);
    pub const VID_DEVICE_CLAIM_REQUIRED: ReasonCodeId = ReasonCodeId(0x5649_0009);
}

#[derive(Debug, Clone)]
pub struct EnrolledSpeaker {
    pub speaker_id: SpeakerId,
    pub user_id: Option<UserId>,
    /// Stand-in for real embeddings. Real implementations derive this from audio.
    pub fingerprint: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoiceIdObservation {
    pub primary_fingerprint: Option<u64>,
    pub secondary_fingerprint: Option<u64>,
    pub spoof_risk: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1VoiceIdConfig {
    pub forbid_binding_during_tts: bool,
    pub enable_spoof_guard: bool,
    pub require_reauth_on_untrusted_device: bool,
    /// Recommended default: 10 minutes.
    pub reauth_interval_ns: u64,
}

impl Ph1VoiceIdConfig {
    pub fn mvp_v1() -> Self {
        Self {
            forbid_binding_during_tts: true,
            enable_spoof_guard: true,
            require_reauth_on_untrusted_device: true,
            reauth_interval_ns: 600_000_000_000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1VoiceIdRuntime {
    config: Ph1VoiceIdConfig,
    enrolled_by_fingerprint: BTreeMap<u64, EnrolledSpeaker>,
    locked_fingerprint: Option<u64>,
    locked_user_id: Option<UserId>,
    locked_session_id: Option<u128>,
    last_verified_at: Option<MonotonicTimeNs>,
}

impl Ph1VoiceIdRuntime {
    pub fn new(
        config: Ph1VoiceIdConfig,
        enrolled: Vec<EnrolledSpeaker>,
    ) -> Result<Self, ContractViolation> {
        let mut enrolled_by_fingerprint = BTreeMap::new();
        for e in enrolled {
            if enrolled_by_fingerprint.insert(e.fingerprint, e).is_some() {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1_voice_id.enrolled_by_fingerprint",
                    reason: "fingerprints must be unique",
                });
            }
        }
        Ok(Self {
            config,
            enrolled_by_fingerprint,
            locked_fingerprint: None,
            locked_user_id: None,
            locked_session_id: None,
            last_verified_at: None,
        })
    }

    pub fn run(&mut self, req: &Ph1VoiceIdRequest, obs: VoiceIdObservation) -> Ph1VoiceIdResponse {
        // Reset locks when the session_id changes (or the session is CLOSED).
        let sid = req.session_state_ref.session_id.map(|s| s.0);
        if sid != self.locked_session_id {
            self.locked_fingerprint = None;
            self.locked_user_id = None;
            self.last_verified_at = None;
            self.locked_session_id = sid;
        }

        // Echo-safe identity: fail closed while TTS is active, unless policy changes.
        if self.config.forbid_binding_during_tts && req.tts_playback_active {
            return unknown(
                IdentityConfidence::Medium,
                reason_codes::VID_FAIL_ECHO_UNSAFE,
                diarization_segments(req, 1, None),
                None,
                req.device_owner_user_id.clone(),
            );
        }

        // Anti-spoofing / liveness: fail closed on replay suspicion.
        if self.config.enable_spoof_guard && obs.spoof_risk {
            return unknown(
                IdentityConfidence::Low,
                reason_codes::VID_SPOOF_RISK,
                diarization_segments(req, 1, None),
                None,
                req.device_owner_user_id.clone(),
            );
        }

        // Continuous verification: reauth required after a deterministic interval.
        if let (Some(_locked), Some(t_last)) = (self.locked_fingerprint, self.last_verified_at) {
            if req.now.0.saturating_sub(t_last.0) >= self.config.reauth_interval_ns {
                let candidate_user_id = self.locked_user_id.clone();
                self.locked_fingerprint = None;
                self.locked_user_id = None;
                self.last_verified_at = None;
                return unknown(
                    IdentityConfidence::Medium,
                    reason_codes::VID_REAUTH_REQUIRED,
                    diarization_segments(req, 1, None),
                    candidate_user_id,
                    req.device_owner_user_id.clone(),
                );
            }
        }

        // No speech => fail closed.
        if req.vad_events.is_empty() {
            return unknown(
                IdentityConfidence::Low,
                reason_codes::VID_FAIL_NO_SPEECH,
                vec![],
                None,
                req.device_owner_user_id.clone(),
            );
        }

        // Multi-speaker presence => fail closed. (Downstream can opt into special handling later.)
        if obs.secondary_fingerprint.is_some() {
            // Expose "speaker change points" through multiple segments even when labels are omitted.
            return unknown(
                IdentityConfidence::Medium,
                reason_codes::VID_FAIL_MULTI_SPEAKER_PRESENT,
                diarization_segments(req, 2, None),
                None,
                req.device_owner_user_id.clone(),
            );
        }

        let fp = match obs.primary_fingerprint {
            Some(v) => v,
            None => {
                return unknown(
                    IdentityConfidence::Medium,
                    reason_codes::VID_FAIL_LOW_CONFIDENCE,
                    diarization_segments(req, 1, None),
                    None,
                    req.device_owner_user_id.clone(),
                )
            }
        };

        // If the fingerprint changes while we had a locked identity, treat as multi-speaker until re-resolved.
        if let Some(locked) = self.locked_fingerprint {
            if locked != fp {
                self.locked_fingerprint = None;
                self.locked_user_id = None;
                self.last_verified_at = None;
                return unknown(
                    IdentityConfidence::Medium,
                    reason_codes::VID_FAIL_MULTI_SPEAKER_PRESENT,
                    diarization_segments(req, 2, None),
                    None,
                    req.device_owner_user_id.clone(),
                );
            }
        }

        // No profiles => deterministic enrollment required (policy can choose to ignore this for "guest" mode later).
        if self.enrolled_by_fingerprint.is_empty() {
            return unknown(
                IdentityConfidence::Medium,
                reason_codes::VID_ENROLLMENT_REQUIRED,
                diarization_segments(req, 1, None),
                None,
                req.device_owner_user_id.clone(),
            );
        }

        match self.enrolled_by_fingerprint.get(&fp) {
            Some(e) => {
                // Untrusted device => require step-up before binding identity for memory/personalization.
                if self.config.require_reauth_on_untrusted_device
                    && req.device_trust_level == DeviceTrustLevel::Untrusted
                {
                    return unknown(
                        IdentityConfidence::Medium,
                        reason_codes::VID_REAUTH_REQUIRED,
                        diarization_segments(req, 1, None),
                        e.user_id.clone(),
                        req.device_owner_user_id.clone(),
                    );
                }

                // Foreign device claim: block personalization until the user explicitly claims one-time vs persistent use.
                if let (Some(owner), Some(user)) = (&req.device_owner_user_id, &e.user_id) {
                    if user != owner {
                        return unknown(
                            IdentityConfidence::Medium,
                            reason_codes::VID_DEVICE_CLAIM_REQUIRED,
                            diarization_segments(req, 1, None),
                            Some(user.clone()),
                            Some(owner.clone()),
                        );
                    }
                }

                self.locked_fingerprint = Some(fp);
                self.locked_user_id = e.user_id.clone();
                self.last_verified_at = Some(req.now);
                let segs = diarization_segments(req, 1, Some(SpeakerLabel::speaker_a()));
                let ok = SpeakerAssertionOk::v1(
                    e.speaker_id.clone(),
                    e.user_id.clone(),
                    segs,
                    SpeakerLabel::speaker_a(),
                )
                .expect("SpeakerAssertionOk::v1 must construct");
                Ph1VoiceIdResponse::SpeakerAssertionOk(ok)
            }
            None => unknown(
                IdentityConfidence::Medium,
                reason_codes::VID_FAIL_PROFILE_NOT_ENROLLED,
                diarization_segments(req, 1, None),
                None,
                req.device_owner_user_id.clone(),
            ),
        }
    }
}

fn unknown(
    confidence: IdentityConfidence,
    reason_code: ReasonCodeId,
    segs: Vec<DiarizationSegment>,
    candidate_user_id: Option<UserId>,
    device_owner_user_id: Option<UserId>,
) -> Ph1VoiceIdResponse {
    let u = SpeakerAssertionUnknown::v1_with_candidate(
        confidence,
        reason_code,
        segs,
        candidate_user_id,
        device_owner_user_id,
    )
    .expect("SpeakerAssertionUnknown::v1 must construct");
    Ph1VoiceIdResponse::SpeakerAssertionUnknown(u)
}

fn diarization_segments(
    req: &Ph1VoiceIdRequest,
    segments: usize,
    label: Option<SpeakerLabel>,
) -> Vec<DiarizationSegment> {
    let Some((t0, t1)) = vad_bounds(&req.vad_events) else {
        return vec![];
    };

    let segments = segments.max(1).min(3);
    match segments {
        1 => vec![DiarizationSegment::v1(t0, t1, label).expect("segment must construct")],
        2 => {
            let mid = MonotonicTimeNs(t0.0.saturating_add(t1.0.saturating_sub(t0.0) / 2));
            vec![
                DiarizationSegment::v1(t0, mid, None).expect("segment must construct"),
                DiarizationSegment::v1(mid, t1, None).expect("segment must construct"),
            ]
        }
        _ => {
            let d = t1.0.saturating_sub(t0.0);
            let t_a = MonotonicTimeNs(t0.0.saturating_add(d / 3));
            let t_b = MonotonicTimeNs(t0.0.saturating_add((d * 2) / 3));
            vec![
                DiarizationSegment::v1(t0, t_a, None).expect("segment must construct"),
                DiarizationSegment::v1(t_a, t_b, None).expect("segment must construct"),
                DiarizationSegment::v1(t_b, t1, None).expect("segment must construct"),
            ]
        }
    }
}

fn vad_bounds(
    vad_events: &[selene_kernel_contracts::ph1k::VadEvent],
) -> Option<(MonotonicTimeNs, MonotonicTimeNs)> {
    let mut min_start: Option<MonotonicTimeNs> = None;
    let mut max_end: Option<MonotonicTimeNs> = None;
    for ev in vad_events {
        min_start = Some(match min_start {
            Some(x) => MonotonicTimeNs(x.0.min(ev.t_start.0)),
            None => ev.t_start,
        });
        max_end = Some(match max_end {
            Some(x) => MonotonicTimeNs(x.0.max(ev.t_end.0)),
            None => ev.t_end,
        });
    }
    match (min_start, max_end) {
        (Some(a), Some(b)) => Some((a, b)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1k::{
        AudioDeviceId, AudioFormat, AudioStreamId, AudioStreamKind, AudioStreamRef, ChannelCount,
        Confidence, SampleFormat, SampleRateHz, SpeechLikeness, VadEvent,
    };
    use selene_kernel_contracts::ph1l::{NextAllowedActions, SessionId, SessionSnapshot};
    use selene_kernel_contracts::SchemaVersion;
    use selene_kernel_contracts::SessionState;

    fn req(
        now: u64,
        vad: Vec<VadEvent>,
        tts_playback_active: bool,
        device_trust_level: DeviceTrustLevel,
        device_owner_user_id: Option<UserId>,
    ) -> Ph1VoiceIdRequest {
        let stream = AudioStreamRef::v1(
            AudioStreamId(1),
            AudioStreamKind::MicProcessed,
            AudioFormat {
                sample_rate_hz: SampleRateHz(16_000),
                channels: ChannelCount(1),
                sample_format: SampleFormat::PcmS16LE,
            },
        );

        let snap = SessionSnapshot {
            schema_version: SchemaVersion(1),
            session_state: SessionState::Active,
            session_id: Some(SessionId(1)),
            next_allowed_actions: NextAllowedActions {
                may_speak: true,
                must_wait: false,
                must_rewake: false,
            },
        };

        Ph1VoiceIdRequest::v1(
            MonotonicTimeNs(now),
            stream,
            vad,
            AudioDeviceId::new("mic").unwrap(),
            snap,
            None,
            tts_playback_active,
            device_trust_level,
            device_owner_user_id,
        )
        .unwrap()
    }

    fn vad(t0: u64, t1: u64) -> VadEvent {
        VadEvent::v1(
            AudioStreamId(1),
            MonotonicTimeNs(t0),
            MonotonicTimeNs(t1),
            Confidence::new(0.9).unwrap(),
            SpeechLikeness::new(0.9).unwrap(),
        )
    }

    #[test]
    fn at_vid_01_correct_user_identity_when_enrolled() {
        let mut rt = Ph1VoiceIdRuntime::new(
            Ph1VoiceIdConfig::mvp_v1(),
            vec![EnrolledSpeaker {
                speaker_id: SpeakerId::new("speaker_1").unwrap(),
                user_id: Some(UserId::new("user_1").unwrap()),
                fingerprint: 42,
            }],
        )
        .unwrap();

        let out = rt.run(
            &req(0, vec![vad(0, 10)], false, DeviceTrustLevel::Trusted, None),
            VoiceIdObservation {
                primary_fingerprint: Some(42),
                secondary_fingerprint: None,
                spoof_risk: false,
            },
        );

        match out {
            Ph1VoiceIdResponse::SpeakerAssertionOk(ok) => {
                assert_eq!(ok.speaker_id.as_str(), "speaker_1");
                assert_eq!(ok.user_id.unwrap().as_str(), "user_1");
                assert_eq!(ok.confidence, IdentityConfidence::High);
            }
            _ => panic!("expected ok"),
        }
    }

    #[test]
    fn at_vid_02_no_guessing_on_unknown_speaker() {
        let mut rt = Ph1VoiceIdRuntime::new(
            Ph1VoiceIdConfig::mvp_v1(),
            vec![EnrolledSpeaker {
                speaker_id: SpeakerId::new("speaker_1").unwrap(),
                user_id: Some(UserId::new("user_1").unwrap()),
                fingerprint: 42,
            }],
        )
        .unwrap();

        let out = rt.run(
            &req(0, vec![vad(0, 10)], false, DeviceTrustLevel::Trusted, None),
            VoiceIdObservation {
                primary_fingerprint: Some(999),
                secondary_fingerprint: None,
                spoof_risk: false,
            },
        );

        assert!(matches!(
            out,
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(u)
                if u.reason_code == reason_codes::VID_FAIL_PROFILE_NOT_ENROLLED
        ));
    }

    #[test]
    fn at_vid_03_speaker_change_detected_mid_session() {
        let mut rt = Ph1VoiceIdRuntime::new(
            Ph1VoiceIdConfig::mvp_v1(),
            vec![EnrolledSpeaker {
                speaker_id: SpeakerId::new("speaker_1").unwrap(),
                user_id: None,
                fingerprint: 1,
            }],
        )
        .unwrap();

        // First speaker OK.
        let out = rt.run(
            &req(0, vec![vad(0, 10)], false, DeviceTrustLevel::Trusted, None),
            VoiceIdObservation {
                primary_fingerprint: Some(1),
                secondary_fingerprint: None,
                spoof_risk: false,
            },
        );
        assert!(matches!(out, Ph1VoiceIdResponse::SpeakerAssertionOk(_)));

        // Different fingerprint implies a different human speaker -> fail closed with multi-speaker reason.
        let out = rt.run(
            &req(
                10,
                vec![vad(10, 20)],
                false,
                DeviceTrustLevel::Trusted,
                None,
            ),
            VoiceIdObservation {
                primary_fingerprint: Some(2),
                secondary_fingerprint: None,
                spoof_risk: false,
            },
        );
        match out {
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(u) => {
                assert_eq!(u.reason_code, reason_codes::VID_FAIL_MULTI_SPEAKER_PRESENT);
                assert!(u.diarization_segments.len() >= 2);
            }
            _ => panic!("expected unknown"),
        }
    }

    #[test]
    fn at_vid_04_multi_speaker_privacy_safety_fails_closed() {
        let mut rt = Ph1VoiceIdRuntime::new(Ph1VoiceIdConfig::mvp_v1(), vec![]).unwrap();
        let out = rt.run(
            &req(0, vec![vad(0, 30)], false, DeviceTrustLevel::Trusted, None),
            VoiceIdObservation {
                primary_fingerprint: Some(1),
                secondary_fingerprint: Some(2),
                spoof_risk: false,
            },
        );
        assert!(matches!(
            out,
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(u)
                if u.reason_code == reason_codes::VID_FAIL_MULTI_SPEAKER_PRESENT
        ));
    }

    #[test]
    fn at_vid_05_echo_safe_identity_fails_closed_during_tts() {
        let mut rt = Ph1VoiceIdRuntime::new(Ph1VoiceIdConfig::mvp_v1(), vec![]).unwrap();
        let out = rt.run(
            &req(0, vec![vad(0, 10)], true, DeviceTrustLevel::Trusted, None),
            VoiceIdObservation {
                primary_fingerprint: Some(1),
                secondary_fingerprint: None,
                spoof_risk: false,
            },
        );
        assert!(matches!(
            out,
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(u) if u.reason_code == reason_codes::VID_FAIL_ECHO_UNSAFE
        ));
    }

    #[test]
    fn at_vid_06_continuous_verification_requires_reauth() {
        let mut cfg = Ph1VoiceIdConfig::mvp_v1();
        cfg.reauth_interval_ns = 5;
        let mut rt = Ph1VoiceIdRuntime::new(
            cfg,
            vec![EnrolledSpeaker {
                speaker_id: SpeakerId::new("speaker_1").unwrap(),
                user_id: Some(UserId::new("user_1").unwrap()),
                fingerprint: 1,
            }],
        )
        .unwrap();

        let out = rt.run(
            &req(0, vec![vad(0, 10)], false, DeviceTrustLevel::Trusted, None),
            VoiceIdObservation {
                primary_fingerprint: Some(1),
                secondary_fingerprint: None,
                spoof_risk: false,
            },
        );
        assert!(matches!(out, Ph1VoiceIdResponse::SpeakerAssertionOk(_)));

        let out = rt.run(
            &req(
                10,
                vec![vad(10, 20)],
                false,
                DeviceTrustLevel::Trusted,
                None,
            ),
            VoiceIdObservation {
                primary_fingerprint: Some(1),
                secondary_fingerprint: None,
                spoof_risk: false,
            },
        );
        match out {
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(u)
                if u.reason_code == reason_codes::VID_REAUTH_REQUIRED =>
            {
                assert_eq!(u.candidate_user_id.unwrap().as_str(), "user_1");
            }
            _ => panic!("expected reauth-required unknown"),
        }
    }

    #[test]
    fn at_vid_07_spoof_risk_fails_closed() {
        let mut rt = Ph1VoiceIdRuntime::new(
            Ph1VoiceIdConfig::mvp_v1(),
            vec![EnrolledSpeaker {
                speaker_id: SpeakerId::new("speaker_1").unwrap(),
                user_id: Some(UserId::new("user_1").unwrap()),
                fingerprint: 1,
            }],
        )
        .unwrap();

        let out = rt.run(
            &req(0, vec![vad(0, 10)], false, DeviceTrustLevel::Trusted, None),
            VoiceIdObservation {
                primary_fingerprint: Some(1),
                secondary_fingerprint: None,
                spoof_risk: true,
            },
        );
        assert!(matches!(
            out,
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(u)
                if u.reason_code == reason_codes::VID_SPOOF_RISK
        ));
    }

    #[test]
    fn at_vid_08_foreign_device_claim_required() {
        let mut rt = Ph1VoiceIdRuntime::new(
            Ph1VoiceIdConfig::mvp_v1(),
            vec![EnrolledSpeaker {
                speaker_id: SpeakerId::new("speaker_1").unwrap(),
                user_id: Some(UserId::new("user_a").unwrap()),
                fingerprint: 1,
            }],
        )
        .unwrap();

        let out = rt.run(
            &req(
                0,
                vec![vad(0, 10)],
                false,
                DeviceTrustLevel::Trusted,
                Some(UserId::new("owner_b").unwrap()),
            ),
            VoiceIdObservation {
                primary_fingerprint: Some(1),
                secondary_fingerprint: None,
                spoof_risk: false,
            },
        );
        match out {
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(u)
                if u.reason_code == reason_codes::VID_DEVICE_CLAIM_REQUIRED =>
            {
                assert_eq!(u.candidate_user_id.unwrap().as_str(), "user_a");
                assert_eq!(u.device_owner_user_id.unwrap().as_str(), "owner_b");
            }
            _ => panic!("expected device-claim-required unknown"),
        }
    }

    #[test]
    fn at_vid_09_enrollment_required_when_no_profiles() {
        let mut rt = Ph1VoiceIdRuntime::new(Ph1VoiceIdConfig::mvp_v1(), vec![]).unwrap();
        let out = rt.run(
            &req(0, vec![vad(0, 10)], false, DeviceTrustLevel::Trusted, None),
            VoiceIdObservation {
                primary_fingerprint: Some(1),
                secondary_fingerprint: None,
                spoof_risk: false,
            },
        );
        assert!(matches!(
            out,
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(u)
                if u.reason_code == reason_codes::VID_ENROLLMENT_REQUIRED
        ));
    }

    #[test]
    fn at_vid_10_untrusted_device_requires_reauth() {
        let mut rt = Ph1VoiceIdRuntime::new(
            Ph1VoiceIdConfig::mvp_v1(),
            vec![EnrolledSpeaker {
                speaker_id: SpeakerId::new("speaker_1").unwrap(),
                user_id: Some(UserId::new("user_1").unwrap()),
                fingerprint: 1,
            }],
        )
        .unwrap();

        let out = rt.run(
            &req(
                0,
                vec![vad(0, 10)],
                false,
                DeviceTrustLevel::Untrusted,
                None,
            ),
            VoiceIdObservation {
                primary_fingerprint: Some(1),
                secondary_fingerprint: None,
                spoof_risk: false,
            },
        );
        match out {
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(u)
                if u.reason_code == reason_codes::VID_REAUTH_REQUIRED =>
            {
                assert_eq!(u.candidate_user_id.unwrap().as_str(), "user_1");
            }
            _ => panic!("expected reauth-required unknown"),
        }
    }
}
