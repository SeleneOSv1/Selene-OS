#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use selene_kernel_contracts::ph1_voice_id::{
    DeviceTrustLevel, DiarizationSegment, IdentityConfidence, Ph1VoiceIdRequest,
    Ph1VoiceIdResponse, SpeakerAssertionOk, SpeakerAssertionUnknown, SpeakerId, SpeakerLabel,
    SpoofLivenessStatus, UserId, VoiceIdCandidate, VoiceIdRiskSignal, PH1VOICEID_IMPLEMENTATION_ID,
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
    pub const VID_FAIL_GRAY_ZONE_MARGIN: ReasonCodeId = ReasonCodeId(0x5649_000A);
    pub const VID_OK_MATCHED: ReasonCodeId = ReasonCodeId(0x5649_0010);
}

pub const PH1_VOICE_ID_ENGINE_ID: &str = "PH1.VOICE.ID";
pub const PH1_VOICE_ID_ACTIVE_IMPLEMENTATION_IDS: &[&str] = &[PH1VOICEID_IMPLEMENTATION_ID];
pub const VOICE_EMBEDDING_DIM: usize = 16;
pub type VoiceEmbedding = [i16; VOICE_EMBEDDING_DIM];

#[derive(Debug, Clone)]
pub struct EnrolledSpeaker {
    pub speaker_id: SpeakerId,
    pub user_id: Option<UserId>,
    /// Stable profile seed derived from enrollment artifacts and profile lineage.
    pub fingerprint: u64,
    /// Optional external profile embedding; when absent we use deterministic simulation fallback.
    pub profile_embedding: Option<VoiceEmbedding>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VoiceIdObservation {
    pub primary_fingerprint: Option<u64>,
    pub secondary_fingerprint: Option<u64>,
    pub primary_embedding: Option<VoiceEmbedding>,
    pub secondary_embedding: Option<VoiceEmbedding>,
    pub spoof_risk: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1VoiceIdConfig {
    pub forbid_binding_during_tts: bool,
    pub enable_spoof_guard: bool,
    pub require_reauth_on_untrusted_device: bool,
    pub enforce_wake_window_when_present: bool,
    /// Recommended default: 3 seconds.
    pub wake_binding_window_ns: u64,
    pub fail_closed_on_high_echo_risk: bool,
    /// Recommended default: 10 minutes.
    pub reauth_interval_ns: u64,
    /// Stage-1 fast prune candidate cap. Must remain bounded for deterministic latency.
    pub stage1_max_candidates: u8,
    /// Stage-1 distance threshold for candidate retention.
    pub stage1_distance_threshold: u64,
    /// Stage-2 score scaling distance.
    pub stage2_distance_scale: u64,
    /// Stage-2 acceptance score floor [0, 10000].
    pub stage2_accept_score_bp: u16,
    /// Stage-2 minimum best-vs-next margin floor [0, 10000].
    pub stage2_min_margin_bp: u16,
    /// Candidate hint floor for unknown decisions [0, 10000].
    pub stage2_candidate_hint_score_bp: u16,
    /// Require runtime-provided primary embedding (no fingerprint-only fallback).
    pub require_primary_embedding: bool,
}

impl Ph1VoiceIdConfig {
    pub fn mvp_v1() -> Self {
        Self {
            forbid_binding_during_tts: true,
            enable_spoof_guard: true,
            require_reauth_on_untrusted_device: true,
            enforce_wake_window_when_present: true,
            wake_binding_window_ns: 3_000_000_000,
            fail_closed_on_high_echo_risk: true,
            reauth_interval_ns: 600_000_000_000,
            stage1_max_candidates: 3,
            stage1_distance_threshold: 1_024,
            stage2_distance_scale: 2_048,
            stage2_accept_score_bp: 9_300,
            stage2_min_margin_bp: 300,
            stage2_candidate_hint_score_bp: 5_500,
            require_primary_embedding: false,
        }
    }
}

#[derive(Debug, Clone)]
struct MatchCandidate {
    fingerprint: u64,
    speaker_id: SpeakerId,
    user_id: Option<UserId>,
    stage1_distance: u64,
    stage1_score_bp: u16,
    stage2_score_bp: u16,
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
        self.run_for_implementation(PH1VOICEID_IMPLEMENTATION_ID, req, obs)
            .expect("PH1.VOICE.ID.001 must be valid")
    }

    pub fn run_for_implementation(
        &mut self,
        implementation_id: &str,
        req: &Ph1VoiceIdRequest,
        obs: VoiceIdObservation,
    ) -> Result<Ph1VoiceIdResponse, ContractViolation> {
        if implementation_id != PH1VOICEID_IMPLEMENTATION_ID {
            return Err(ContractViolation::InvalidValue {
                field: "ph1_voice_id.implementation_id",
                reason: "unknown implementation_id",
            });
        }
        Ok(self.evaluate(req, obs))
    }

    fn evaluate(&mut self, req: &Ph1VoiceIdRequest, obs: VoiceIdObservation) -> Ph1VoiceIdResponse {
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

        // Identity window: when wake context is present, only bind within a bounded post-wake window.
        if self.config.enforce_wake_window_when_present {
            if let Some(wake) = &req.wake_event {
                if !wake.accepted {
                    return unknown(
                        IdentityConfidence::Medium,
                        reason_codes::VID_FAIL_LOW_CONFIDENCE,
                        diarization_segments(req, 1, None),
                        None,
                        req.device_owner_user_id.clone(),
                    );
                }

                let age_ns = req.now.0.saturating_sub(wake.t_decision.0);
                if age_ns > self.config.wake_binding_window_ns {
                    return unknown(
                        IdentityConfidence::Medium,
                        reason_codes::VID_FAIL_LOW_CONFIDENCE,
                        diarization_segments(req, 1, None),
                        None,
                        req.device_owner_user_id.clone(),
                    );
                }
            }
        }

        // Optional bounded risk hint: high-echo environments must fail closed when policy enables it.
        if self.config.fail_closed_on_high_echo_risk
            && req.risk_signals.contains(&VoiceIdRiskSignal::HighEchoRisk)
        {
            return unknown(
                IdentityConfidence::Medium,
                reason_codes::VID_FAIL_ECHO_UNSAFE,
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
        if obs.secondary_fingerprint.is_some() || obs.secondary_embedding.is_some() {
            // Expose "speaker change points" through multiple segments even when labels are omitted.
            return unknown(
                IdentityConfidence::Medium,
                reason_codes::VID_FAIL_MULTI_SPEAKER_PRESENT,
                diarization_segments(req, 2, None),
                None,
                req.device_owner_user_id.clone(),
            );
        }

        if self.config.require_primary_embedding && obs.primary_embedding.is_none() {
            return unknown(
                IdentityConfidence::Medium,
                reason_codes::VID_FAIL_LOW_CONFIDENCE,
                diarization_segments(req, 1, None),
                None,
                req.device_owner_user_id.clone(),
            );
        }

        if obs.primary_fingerprint.is_none() && obs.primary_embedding.is_none() {
            return unknown(
                IdentityConfidence::Medium,
                reason_codes::VID_FAIL_LOW_CONFIDENCE,
                diarization_segments(req, 1, None),
                None,
                req.device_owner_user_id.clone(),
            );
        }

        let fp_hint = obs.primary_fingerprint;

        // If the fingerprint changes while we had a locked identity, treat as multi-speaker until re-resolved.
        if let (Some(locked), Some(fp)) = (self.locked_fingerprint, fp_hint) {
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

        let observed_embedding = observed_embedding_from_observation(req, obs, fp_hint);
        let fast_candidates = fast_prune_candidates(
            &self.config,
            &self.enrolled_by_fingerprint,
            &observed_embedding,
        );
        if fast_candidates.is_empty() {
            return unknown(
                IdentityConfidence::Medium,
                reason_codes::VID_FAIL_PROFILE_NOT_ENROLLED,
                diarization_segments(req, 1, None),
                None,
                req.device_owner_user_id.clone(),
            );
        }

        let ranked = verify_ranked_candidates(
            &self.config,
            &self.enrolled_by_fingerprint,
            &observed_embedding,
            fast_candidates,
        );
        let best = &ranked[0];
        let margin_bp = score_margin_bp(&ranked);
        let candidate_set = build_candidate_set(&ranked);

        let margin_ok = match margin_bp {
            Some(m) => m >= self.config.stage2_min_margin_bp,
            None => true,
        };
        if best.stage2_score_bp < self.config.stage2_accept_score_bp || !margin_ok {
            let reason_code = if !margin_ok {
                reason_codes::VID_FAIL_GRAY_ZONE_MARGIN
            } else if best.stage2_score_bp < self.config.stage2_candidate_hint_score_bp {
                reason_codes::VID_FAIL_PROFILE_NOT_ENROLLED
            } else {
                reason_codes::VID_FAIL_LOW_CONFIDENCE
            };
            let candidate_user_id =
                if best.stage2_score_bp >= self.config.stage2_candidate_hint_score_bp {
                    best.user_id.clone()
                } else {
                    None
                };
            return unknown_with_metrics(
                IdentityConfidence::Medium,
                reason_code,
                diarization_segments(req, 1, None),
                candidate_user_id,
                req.device_owner_user_id.clone(),
                best.stage2_score_bp,
                margin_bp,
                SpoofLivenessStatus::Unknown,
                candidate_set,
            );
        }

        // Untrusted device => require step-up before binding identity for memory/personalization.
        if self.config.require_reauth_on_untrusted_device
            && req.device_trust_level == DeviceTrustLevel::Untrusted
        {
            return unknown_with_metrics(
                IdentityConfidence::Medium,
                reason_codes::VID_REAUTH_REQUIRED,
                diarization_segments(req, 1, None),
                best.user_id.clone(),
                req.device_owner_user_id.clone(),
                best.stage2_score_bp,
                margin_bp,
                SpoofLivenessStatus::Unknown,
                candidate_set,
            );
        }

        // Foreign device claim: block personalization until explicit claim.
        if let (Some(owner), Some(user)) = (&req.device_owner_user_id, &best.user_id) {
            if user != owner {
                return unknown_with_metrics(
                    IdentityConfidence::Medium,
                    reason_codes::VID_DEVICE_CLAIM_REQUIRED,
                    diarization_segments(req, 1, None),
                    Some(user.clone()),
                    Some(owner.clone()),
                    best.stage2_score_bp,
                    margin_bp,
                    SpoofLivenessStatus::Unknown,
                    candidate_set,
                );
            }
        }

        let segs = diarization_segments(req, 1, Some(SpeakerLabel::speaker_a()));
        if segs.is_empty() {
            return unknown(
                IdentityConfidence::Low,
                reason_codes::VID_FAIL_NO_SPEECH,
                vec![],
                best.user_id.clone(),
                req.device_owner_user_id.clone(),
            );
        }

        let ok = match SpeakerAssertionOk::v1(
            best.speaker_id.clone(),
            best.user_id.clone(),
            segs.clone(),
            SpeakerLabel::speaker_a(),
        )
        .and_then(|_| {
            SpeakerAssertionOk::v1_with_metrics(
                best.speaker_id.clone(),
                best.user_id.clone(),
                segs,
                SpeakerLabel::speaker_a(),
                best.stage2_score_bp,
                margin_bp,
                Some(reason_codes::VID_OK_MATCHED),
                SpoofLivenessStatus::Live,
                candidate_set,
            )
        }) {
            Ok(v) => v,
            Err(_) => {
                return unknown(
                    IdentityConfidence::Low,
                    reason_codes::VID_FAIL_LOW_CONFIDENCE,
                    vec![],
                    best.user_id.clone(),
                    req.device_owner_user_id.clone(),
                )
            }
        };
        self.locked_fingerprint = Some(best.fingerprint);
        self.locked_user_id = best.user_id.clone();
        self.last_verified_at = Some(req.now);
        Ph1VoiceIdResponse::SpeakerAssertionOk(ok)
    }
}

fn fast_prune_candidates(
    config: &Ph1VoiceIdConfig,
    enrolled_by_fingerprint: &BTreeMap<u64, EnrolledSpeaker>,
    observed_embedding: &VoiceEmbedding,
) -> Vec<MatchCandidate> {
    let mut candidates: Vec<MatchCandidate> = enrolled_by_fingerprint
        .iter()
        .filter_map(|(fp, enrolled)| {
            let candidate_embedding = profile_embedding_for_enrolled(enrolled, *fp);
            let distance = embedding_l1_distance(observed_embedding, &candidate_embedding);
            if distance > config.stage1_distance_threshold {
                return None;
            }
            let stage1_score_bp =
                score_from_distance_bp(distance, config.stage1_distance_threshold);
            Some(MatchCandidate {
                fingerprint: *fp,
                speaker_id: enrolled.speaker_id.clone(),
                user_id: enrolled.user_id.clone(),
                stage1_distance: distance,
                stage1_score_bp,
                stage2_score_bp: 0,
            })
        })
        .collect();

    candidates.sort_by(|a, b| {
        a.stage1_distance
            .cmp(&b.stage1_distance)
            .then_with(|| b.stage1_score_bp.cmp(&a.stage1_score_bp))
            .then_with(|| a.fingerprint.cmp(&b.fingerprint))
    });
    candidates.truncate(config.stage1_max_candidates.max(1) as usize);
    candidates
}

fn verify_ranked_candidates(
    config: &Ph1VoiceIdConfig,
    enrolled_by_fingerprint: &BTreeMap<u64, EnrolledSpeaker>,
    observed_embedding: &VoiceEmbedding,
    mut candidates: Vec<MatchCandidate>,
) -> Vec<MatchCandidate> {
    if candidates.len() == 1 && candidates[0].stage1_distance == 0 {
        candidates[0].stage2_score_bp = 10_000;
        return candidates;
    }
    for c in &mut candidates {
        let candidate_embedding = enrolled_by_fingerprint
            .get(&c.fingerprint)
            .map(|enrolled| profile_embedding_for_enrolled(enrolled, c.fingerprint))
            .unwrap_or_else(|| simulation_profile_embedding_from_seed(c.fingerprint));
        let distance = embedding_l1_distance(observed_embedding, &candidate_embedding);
        let distance_score = score_from_distance_bp(distance, config.stage2_distance_scale) as u32;
        let cosine_score =
            embedding_cosine_similarity_bp(observed_embedding, &candidate_embedding) as u32;
        c.stage2_score_bp = ((distance_score * 6 + cosine_score * 4) / 10) as u16;
    }
    candidates.sort_by(|a, b| {
        b.stage2_score_bp
            .cmp(&a.stage2_score_bp)
            .then_with(|| a.stage1_distance.cmp(&b.stage1_distance))
            .then_with(|| a.fingerprint.cmp(&b.fingerprint))
    });
    candidates
}

fn score_from_distance_bp(distance: u64, scale: u64) -> u16 {
    if scale == 0 {
        return if distance == 0 { 10_000 } else { 0 };
    }
    let capped = distance.min(scale);
    let remaining = scale.saturating_sub(capped);
    ((remaining as u128 * 10_000u128) / scale as u128) as u16
}

fn score_margin_bp(ranked: &[MatchCandidate]) -> Option<u16> {
    let best = ranked.first()?;
    let next = ranked.get(1)?;
    Some(best.stage2_score_bp.saturating_sub(next.stage2_score_bp))
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

pub fn simulation_profile_embedding_from_seed(seed: u64) -> VoiceEmbedding {
    let mut out = [0_i16; VOICE_EMBEDDING_DIM];
    let mut state = seed ^ 0x51E1_3E5D_9AC1_0F27;
    for slot in &mut out {
        state = splitmix64(state);
        // Keep values in a compact symmetric range to preserve deterministic distance scaling.
        let v = ((state >> 50) & 0x3FF) as i32 - 512;
        *slot = v as i16;
    }
    out
}

fn profile_embedding_for_enrolled(enrolled: &EnrolledSpeaker, seed: u64) -> VoiceEmbedding {
    enrolled
        .profile_embedding
        .unwrap_or_else(|| simulation_profile_embedding_from_seed(seed))
}

fn observed_embedding_from_observation(
    req: &Ph1VoiceIdRequest,
    obs: VoiceIdObservation,
    seed_hint: Option<u64>,
) -> VoiceEmbedding {
    if let Some(embed) = obs.primary_embedding {
        return embed;
    }
    let seed = seed_hint.unwrap_or(0);
    let mut out = simulation_profile_embedding_from_seed(seed);
    let quality_bp = observation_quality_bp(req);
    let jitter_max = ((10_000u32.saturating_sub(quality_bp as u32)) / 1_000).min(16) as i16;
    if jitter_max == 0 {
        return out;
    }
    let salt = observation_salt(req, seed);
    for (idx, slot) in out.iter_mut().enumerate() {
        let h = splitmix64(salt ^ (idx as u64));
        let signed = ((h & 0x1F) as i16) - 16;
        let delta = signed.saturating_mul(jitter_max) / 8;
        *slot = slot.saturating_add(delta);
    }
    out
}

fn observation_quality_bp(req: &Ph1VoiceIdRequest) -> u16 {
    if req.vad_events.is_empty() {
        return 0;
    }
    let mut conf_sum = 0f32;
    let mut speech_sum = 0f32;
    for vad in &req.vad_events {
        conf_sum += vad.confidence.0;
        speech_sum += vad.speech_likeness.0;
    }
    let n = req.vad_events.len() as f32;
    let avg_conf = (conf_sum / n).clamp(0.0, 1.0);
    let avg_speech = (speech_sum / n).clamp(0.0, 1.0);
    let mut quality = ((avg_conf * 0.6 + avg_speech * 0.4) * 10_000.0).round() as i32;
    if req.risk_signals.contains(&VoiceIdRiskSignal::HighEchoRisk) {
        quality = quality.saturating_sub(2_000);
    }
    quality.clamp(0, 10_000) as u16
}

fn observation_salt(req: &Ph1VoiceIdRequest, seed: u64) -> u64 {
    let mut salt = seed ^ req.now.0 ^ ((req.vad_events.len() as u64) << 32);
    if let Some(wake) = &req.wake_event {
        salt ^= wake.t_decision.0.rotate_left(11);
        salt ^= u64::from(wake.reason_code.0.rotate_right(7));
    }
    for (idx, vad) in req.vad_events.iter().enumerate() {
        salt ^= vad.t_start.0.rotate_left((idx % 17) as u32);
        salt ^= vad.t_end.0.rotate_right((idx % 13) as u32);
    }
    splitmix64(salt)
}

fn embedding_l1_distance(a: &VoiceEmbedding, b: &VoiceEmbedding) -> u64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| i32::from(*x).abs_diff(i32::from(*y)) as u64)
        .sum()
}

fn embedding_cosine_similarity_bp(a: &VoiceEmbedding, b: &VoiceEmbedding) -> u16 {
    let mut dot = 0_f64;
    let mut norm_a = 0_f64;
    let mut norm_b = 0_f64;
    for (x, y) in a.iter().zip(b.iter()) {
        let xf = f64::from(*x);
        let yf = f64::from(*y);
        dot += xf * yf;
        norm_a += xf * xf;
        norm_b += yf * yf;
    }
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0;
    }
    let cosine = (dot / (norm_a.sqrt() * norm_b.sqrt())).clamp(-1.0, 1.0);
    (((cosine + 1.0) * 5_000.0).round() as i32).clamp(0, 10_000) as u16
}

fn build_candidate_set(ranked: &[MatchCandidate]) -> Vec<VoiceIdCandidate> {
    let mut out = Vec::with_capacity(ranked.len());
    for c in ranked {
        if let Ok(candidate) = VoiceIdCandidate::v1(
            c.user_id.clone(),
            Some(c.speaker_id.clone()),
            c.stage2_score_bp,
        ) {
            out.push(candidate);
        }
    }
    out
}

fn unknown_with_metrics(
    confidence: IdentityConfidence,
    reason_code: ReasonCodeId,
    segs: Vec<DiarizationSegment>,
    candidate_user_id: Option<UserId>,
    device_owner_user_id: Option<UserId>,
    score_bp: u16,
    margin_to_next_bp: Option<u16>,
    spoof_liveness_status: SpoofLivenessStatus,
    candidate_set: Vec<VoiceIdCandidate>,
) -> Ph1VoiceIdResponse {
    let u = SpeakerAssertionUnknown::v1_with_metrics_and_candidate(
        confidence,
        reason_code,
        segs,
        score_bp,
        margin_to_next_bp,
        spoof_liveness_status,
        candidate_set,
        candidate_user_id,
        device_owner_user_id,
    )
    .expect("SpeakerAssertionUnknown::v1 must construct");
    Ph1VoiceIdResponse::SpeakerAssertionUnknown(u)
}

fn unknown(
    confidence: IdentityConfidence,
    reason_code: ReasonCodeId,
    segs: Vec<DiarizationSegment>,
    candidate_user_id: Option<UserId>,
    device_owner_user_id: Option<UserId>,
) -> Ph1VoiceIdResponse {
    let score_bp = unknown_score_bp(confidence, reason_code);
    let candidate_set = candidate_set_for_unknown(&candidate_user_id, score_bp);
    unknown_with_metrics(
        confidence,
        reason_code,
        segs,
        candidate_user_id,
        device_owner_user_id,
        score_bp,
        None,
        spoof_status_for_reason(reason_code),
        candidate_set,
    )
}

fn unknown_score_bp(confidence: IdentityConfidence, reason_code: ReasonCodeId) -> u16 {
    match reason_code {
        reason_codes::VID_SPOOF_RISK => 400,
        reason_codes::VID_FAIL_NO_SPEECH => 0,
        reason_codes::VID_FAIL_MULTI_SPEAKER_PRESENT => 2_000,
        reason_codes::VID_FAIL_PROFILE_NOT_ENROLLED => 3_500,
        reason_codes::VID_ENROLLMENT_REQUIRED => 3_500,
        reason_codes::VID_FAIL_GRAY_ZONE_MARGIN => 5_200,
        reason_codes::VID_REAUTH_REQUIRED => 6_000,
        reason_codes::VID_DEVICE_CLAIM_REQUIRED => 6_200,
        reason_codes::VID_FAIL_ECHO_UNSAFE => 3_000,
        _ => match confidence {
            IdentityConfidence::High => 0,
            IdentityConfidence::Medium => 4_500,
            IdentityConfidence::Low => 2_000,
        },
    }
}

fn spoof_status_for_reason(reason_code: ReasonCodeId) -> SpoofLivenessStatus {
    if reason_code == reason_codes::VID_SPOOF_RISK {
        SpoofLivenessStatus::SuspectedSpoof
    } else {
        SpoofLivenessStatus::Unknown
    }
}

fn candidate_set_for_unknown(
    candidate_user_id: &Option<UserId>,
    score_bp: u16,
) -> Vec<VoiceIdCandidate> {
    match candidate_user_id.clone() {
        Some(user_id) => match VoiceIdCandidate::v1(Some(user_id), None, score_bp) {
            Ok(candidate) => vec![candidate],
            Err(_) => vec![],
        },
        None => vec![],
    }
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
        1 => DiarizationSegment::v1(t0, t1, label)
            .map(|segment| vec![segment])
            .unwrap_or_default(),
        2 => {
            let mid = MonotonicTimeNs(t0.0.saturating_add(t1.0.saturating_sub(t0.0) / 2));
            match (
                DiarizationSegment::v1(t0, mid, None),
                DiarizationSegment::v1(mid, t1, None),
            ) {
                (Ok(a), Ok(b)) => vec![a, b],
                _ => vec![],
            }
        }
        _ => {
            let d = t1.0.saturating_sub(t0.0);
            let t_a = MonotonicTimeNs(t0.0.saturating_add(d / 3));
            let t_b = MonotonicTimeNs(t0.0.saturating_add((d * 2) / 3));
            match (
                DiarizationSegment::v1(t0, t_a, None),
                DiarizationSegment::v1(t_a, t_b, None),
                DiarizationSegment::v1(t_b, t1, None),
            ) {
                (Ok(a), Ok(b), Ok(c)) => vec![a, b, c],
                _ => vec![],
            }
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
        Confidence, FrameDurationMs, PreRollBufferId, SampleFormat, SampleRateHz, SpeechLikeness,
        VadEvent,
    };
    use selene_kernel_contracts::ph1l::{NextAllowedActions, SessionId, SessionSnapshot};
    use selene_kernel_contracts::ph1w::{BoundedAudioSegmentRef, WakeDecision, WakeGateResults};
    use selene_kernel_contracts::SchemaVersion;
    use selene_kernel_contracts::SessionState;

    fn req(
        now: u64,
        vad: Vec<VadEvent>,
        tts_playback_active: bool,
        device_trust_level: DeviceTrustLevel,
        device_owner_user_id: Option<UserId>,
    ) -> Ph1VoiceIdRequest {
        req_with(
            now,
            vad,
            tts_playback_active,
            device_trust_level,
            device_owner_user_id,
            None,
            vec![],
        )
    }

    fn req_with(
        now: u64,
        vad: Vec<VadEvent>,
        tts_playback_active: bool,
        device_trust_level: DeviceTrustLevel,
        device_owner_user_id: Option<UserId>,
        wake_event: Option<WakeDecision>,
        risk_signals: Vec<VoiceIdRiskSignal>,
    ) -> Ph1VoiceIdRequest {
        let stream = AudioStreamRef::v1(
            AudioStreamId(1),
            AudioStreamKind::MicProcessed,
            AudioFormat {
                sample_rate_hz: SampleRateHz(16_000),
                channels: ChannelCount(1),
                sample_format: SampleFormat::PcmS16LE,
            },
            FrameDurationMs::Ms20,
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

        Ph1VoiceIdRequest::v1_with_risk_signals(
            MonotonicTimeNs(now),
            stream,
            vad,
            AudioDeviceId::new("mic").unwrap(),
            snap,
            wake_event,
            tts_playback_active,
            device_trust_level,
            device_owner_user_id,
            risk_signals,
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

    fn wake_gates_all_pass() -> WakeGateResults {
        WakeGateResults {
            g0_integrity_ok: true,
            g1_activity_ok: true,
            g1a_utterance_start_ok: true,
            g2_light_ok: true,
            g3_strong_ok: true,
            g3a_liveness_ok: true,
            g4_personalization_ok: true,
            g5_policy_ok: true,
        }
    }

    fn wake_accept(t_decision: u64) -> WakeDecision {
        WakeDecision::accept_v1(
            ReasonCodeId(1),
            wake_gates_all_pass(),
            MonotonicTimeNs(t_decision),
            None,
            None,
            BoundedAudioSegmentRef::v1(
                AudioStreamId(1),
                PreRollBufferId(1),
                MonotonicTimeNs(t_decision),
                MonotonicTimeNs(t_decision + 10),
                MonotonicTimeNs(t_decision + 1),
                MonotonicTimeNs(t_decision + 2),
            )
            .unwrap(),
        )
        .unwrap()
    }

    fn wake_reject(t_decision: u64) -> WakeDecision {
        WakeDecision::reject_v1(
            ReasonCodeId(1),
            wake_gates_all_pass(),
            MonotonicTimeNs(t_decision),
            None,
            None,
        )
        .unwrap()
    }

    #[test]
    fn at_vid_01_correct_user_identity_when_enrolled() {
        let mut rt = Ph1VoiceIdRuntime::new(
            Ph1VoiceIdConfig::mvp_v1(),
            vec![EnrolledSpeaker {
                speaker_id: SpeakerId::new("speaker_1").unwrap(),
                user_id: Some(UserId::new("user_1").unwrap()),
                fingerprint: 42,
                profile_embedding: None,
            }],
        )
        .unwrap();

        let out = rt.run(
            &req(0, vec![vad(0, 10)], false, DeviceTrustLevel::Trusted, None),
            VoiceIdObservation {
                primary_fingerprint: Some(42),
                secondary_fingerprint: None,
                primary_embedding: None,
                secondary_embedding: None,
                spoof_risk: false,
            },
        );

        match out {
            Ph1VoiceIdResponse::SpeakerAssertionOk(ok) => {
                assert_eq!(ok.speaker_id.as_str(), "speaker_1");
                assert_eq!(ok.user_id.unwrap().as_str(), "user_1");
                assert_eq!(
                    ok.decision,
                    selene_kernel_contracts::ph1_voice_id::VoiceIdDecision::Ok
                );
                assert_eq!(ok.confidence, IdentityConfidence::High);
                assert!(ok.score_bp >= 9_300);
                assert_eq!(ok.margin_to_next_bp, None);
                assert_eq!(ok.spoof_liveness_status, SpoofLivenessStatus::Live);
                assert_eq!(ok.candidate_set.len(), 1);
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
                profile_embedding: None,
            }],
        )
        .unwrap();

        let out = rt.run(
            &req(0, vec![vad(0, 10)], false, DeviceTrustLevel::Trusted, None),
            VoiceIdObservation {
                primary_fingerprint: Some(999),
                secondary_fingerprint: None,
                primary_embedding: None,
                secondary_embedding: None,
                spoof_risk: false,
            },
        );

        assert!(matches!(
            out,
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(_)
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
                profile_embedding: None,
            }],
        )
        .unwrap();

        // First speaker OK.
        let out = rt.run(
            &req(0, vec![vad(0, 10)], false, DeviceTrustLevel::Trusted, None),
            VoiceIdObservation {
                primary_fingerprint: Some(1),
                secondary_fingerprint: None,
                primary_embedding: None,
                secondary_embedding: None,
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
                primary_embedding: None,
                secondary_embedding: None,
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
                primary_embedding: None,
                secondary_embedding: None,
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
                primary_embedding: None,
                secondary_embedding: None,
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
                profile_embedding: None,
            }],
        )
        .unwrap();

        let out = rt.run(
            &req(0, vec![vad(0, 10)], false, DeviceTrustLevel::Trusted, None),
            VoiceIdObservation {
                primary_fingerprint: Some(1),
                secondary_fingerprint: None,
                primary_embedding: None,
                secondary_embedding: None,
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
                primary_embedding: None,
                secondary_embedding: None,
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
                profile_embedding: None,
            }],
        )
        .unwrap();

        let out = rt.run(
            &req(0, vec![vad(0, 10)], false, DeviceTrustLevel::Trusted, None),
            VoiceIdObservation {
                primary_fingerprint: Some(1),
                secondary_fingerprint: None,
                primary_embedding: None,
                secondary_embedding: None,
                spoof_risk: true,
            },
        );
        assert!(matches!(
            out,
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(u)
                if u.reason_code == reason_codes::VID_SPOOF_RISK
                    && u.spoof_liveness_status == SpoofLivenessStatus::SuspectedSpoof
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
                profile_embedding: None,
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
                primary_embedding: None,
                secondary_embedding: None,
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
                primary_embedding: None,
                secondary_embedding: None,
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
                profile_embedding: None,
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
                primary_embedding: None,
                secondary_embedding: None,
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
    fn at_vid_11_wake_window_rejects_stale_wake() {
        let mut cfg = Ph1VoiceIdConfig::mvp_v1();
        cfg.wake_binding_window_ns = 5;
        let mut rt = Ph1VoiceIdRuntime::new(
            cfg,
            vec![EnrolledSpeaker {
                speaker_id: SpeakerId::new("speaker_1").unwrap(),
                user_id: Some(UserId::new("user_1").unwrap()),
                fingerprint: 1,
                profile_embedding: None,
            }],
        )
        .unwrap();

        let out = rt.run(
            &req_with(
                10,
                vec![vad(10, 20)],
                false,
                DeviceTrustLevel::Trusted,
                None,
                Some(wake_accept(0)),
                vec![],
            ),
            VoiceIdObservation {
                primary_fingerprint: Some(1),
                secondary_fingerprint: None,
                primary_embedding: None,
                secondary_embedding: None,
                spoof_risk: false,
            },
        );

        assert!(matches!(
            out,
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(u)
                if u.reason_code == reason_codes::VID_FAIL_LOW_CONFIDENCE
        ));
    }

    #[test]
    fn at_vid_12_wake_reject_fails_closed() {
        let mut rt = Ph1VoiceIdRuntime::new(
            Ph1VoiceIdConfig::mvp_v1(),
            vec![EnrolledSpeaker {
                speaker_id: SpeakerId::new("speaker_1").unwrap(),
                user_id: Some(UserId::new("user_1").unwrap()),
                fingerprint: 1,
                profile_embedding: None,
            }],
        )
        .unwrap();

        let out = rt.run(
            &req_with(
                0,
                vec![vad(0, 10)],
                false,
                DeviceTrustLevel::Trusted,
                None,
                Some(wake_reject(0)),
                vec![],
            ),
            VoiceIdObservation {
                primary_fingerprint: Some(1),
                secondary_fingerprint: None,
                primary_embedding: None,
                secondary_embedding: None,
                spoof_risk: false,
            },
        );

        assert!(matches!(
            out,
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(u)
                if u.reason_code == reason_codes::VID_FAIL_LOW_CONFIDENCE
        ));
    }

    #[test]
    fn at_vid_13_high_echo_risk_signal_fails_closed() {
        let mut rt = Ph1VoiceIdRuntime::new(
            Ph1VoiceIdConfig::mvp_v1(),
            vec![EnrolledSpeaker {
                speaker_id: SpeakerId::new("speaker_1").unwrap(),
                user_id: Some(UserId::new("user_1").unwrap()),
                fingerprint: 1,
                profile_embedding: None,
            }],
        )
        .unwrap();

        let out = rt.run(
            &req_with(
                0,
                vec![vad(0, 10)],
                false,
                DeviceTrustLevel::Trusted,
                None,
                Some(wake_accept(0)),
                vec![VoiceIdRiskSignal::HighEchoRisk],
            ),
            VoiceIdObservation {
                primary_fingerprint: Some(1),
                secondary_fingerprint: None,
                primary_embedding: None,
                secondary_embedding: None,
                spoof_risk: false,
            },
        );

        assert!(matches!(
            out,
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(u)
                if u.reason_code == reason_codes::VID_FAIL_ECHO_UNSAFE
        ));
    }

    #[test]
    fn at_vid_14_empty_vad_fails_closed_without_panic() {
        let mut rt = Ph1VoiceIdRuntime::new(
            Ph1VoiceIdConfig::mvp_v1(),
            vec![EnrolledSpeaker {
                speaker_id: SpeakerId::new("speaker_1").unwrap(),
                user_id: Some(UserId::new("user_1").unwrap()),
                fingerprint: 1,
                profile_embedding: None,
            }],
        )
        .unwrap();

        let out = rt.run(
            &req(0, vec![], false, DeviceTrustLevel::Trusted, None),
            VoiceIdObservation {
                primary_fingerprint: Some(1),
                secondary_fingerprint: None,
                primary_embedding: None,
                secondary_embedding: None,
                spoof_risk: false,
            },
        );

        assert!(matches!(
            out,
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(u)
                if u.reason_code == reason_codes::VID_FAIL_NO_SPEECH
        ));
    }

    #[test]
    fn at_vid_15_two_stage_low_margin_returns_unknown() {
        let mut cfg = Ph1VoiceIdConfig::mvp_v1();
        cfg.stage1_distance_threshold = 64;
        cfg.stage2_distance_scale = 200;
        cfg.stage2_accept_score_bp = 9_000;
        cfg.stage2_min_margin_bp = 500;
        let emb_obs = [101_i16; VOICE_EMBEDDING_DIM];
        let emb_a = [100_i16; VOICE_EMBEDDING_DIM];
        let mut emb_b = [100_i16; VOICE_EMBEDDING_DIM];
        for v in emb_b.iter_mut().skip(VOICE_EMBEDDING_DIM / 2) {
            *v = 102;
        }

        let mut rt = Ph1VoiceIdRuntime::new(
            cfg,
            vec![
                EnrolledSpeaker {
                    speaker_id: SpeakerId::new("speaker_a").unwrap(),
                    user_id: Some(UserId::new("user_a").unwrap()),
                    fingerprint: 100,
                    profile_embedding: Some(emb_a),
                },
                EnrolledSpeaker {
                    speaker_id: SpeakerId::new("speaker_b").unwrap(),
                    user_id: Some(UserId::new("user_b").unwrap()),
                    fingerprint: 101,
                    profile_embedding: Some(emb_b),
                },
            ],
        )
        .unwrap();

        let out = rt.run(
            &req(0, vec![vad(0, 10)], false, DeviceTrustLevel::Trusted, None),
            VoiceIdObservation {
                primary_fingerprint: Some(100),
                secondary_fingerprint: None,
                primary_embedding: Some(emb_obs),
                secondary_embedding: None,
                spoof_risk: false,
            },
        );

        match out {
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(u) => {
                assert_eq!(u.reason_code, reason_codes::VID_FAIL_GRAY_ZONE_MARGIN);
                assert_eq!(u.candidate_set.len(), 2);
                assert_eq!(
                    u.candidate_user_id.as_ref().map(UserId::as_str),
                    Some("user_a")
                );
            }
            _ => panic!("expected low-margin unknown"),
        }
    }

    #[test]
    fn at_vid_16_fast_prune_caps_candidate_set_size() {
        let mut cfg = Ph1VoiceIdConfig::mvp_v1();
        cfg.stage1_max_candidates = 2;
        cfg.stage1_distance_threshold = 64;
        cfg.stage2_distance_scale = 200;
        cfg.stage2_accept_score_bp = 9_000;
        cfg.stage2_min_margin_bp = 500;
        let emb_obs = [101_i16; VOICE_EMBEDDING_DIM];
        let emb_a = [100_i16; VOICE_EMBEDDING_DIM];
        let mut emb_b = [100_i16; VOICE_EMBEDDING_DIM];
        for v in emb_b.iter_mut().skip(VOICE_EMBEDDING_DIM / 2) {
            *v = 102;
        }
        let mut emb_c = [100_i16; VOICE_EMBEDDING_DIM];
        for v in emb_c.iter_mut().skip((VOICE_EMBEDDING_DIM * 3) / 4) {
            *v = 104;
        }

        let mut rt = Ph1VoiceIdRuntime::new(
            cfg,
            vec![
                EnrolledSpeaker {
                    speaker_id: SpeakerId::new("speaker_a").unwrap(),
                    user_id: Some(UserId::new("user_a").unwrap()),
                    fingerprint: 100,
                    profile_embedding: Some(emb_a),
                },
                EnrolledSpeaker {
                    speaker_id: SpeakerId::new("speaker_b").unwrap(),
                    user_id: Some(UserId::new("user_b").unwrap()),
                    fingerprint: 101,
                    profile_embedding: Some(emb_b),
                },
                EnrolledSpeaker {
                    speaker_id: SpeakerId::new("speaker_c").unwrap(),
                    user_id: Some(UserId::new("user_c").unwrap()),
                    fingerprint: 102,
                    profile_embedding: Some(emb_c),
                },
            ],
        )
        .unwrap();

        let out = rt.run(
            &req(0, vec![vad(0, 10)], false, DeviceTrustLevel::Trusted, None),
            VoiceIdObservation {
                primary_fingerprint: Some(100),
                secondary_fingerprint: None,
                primary_embedding: Some(emb_obs),
                secondary_embedding: None,
                spoof_risk: false,
            },
        );

        match out {
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(u) => {
                assert_eq!(u.reason_code, reason_codes::VID_FAIL_GRAY_ZONE_MARGIN);
                assert_eq!(u.candidate_set.len(), 2);
            }
            _ => panic!("expected unknown with capped candidate_set"),
        }
    }

    #[test]
    fn at_vid_17_require_primary_embedding_fails_closed_without_embedding() {
        let mut cfg = Ph1VoiceIdConfig::mvp_v1();
        cfg.require_primary_embedding = true;
        let mut rt = Ph1VoiceIdRuntime::new(
            cfg,
            vec![EnrolledSpeaker {
                speaker_id: SpeakerId::new("speaker_1").unwrap(),
                user_id: Some(UserId::new("user_1").unwrap()),
                fingerprint: 7,
                profile_embedding: Some(simulation_profile_embedding_from_seed(7)),
            }],
        )
        .unwrap();

        let out = rt.run(
            &req(0, vec![vad(0, 10)], false, DeviceTrustLevel::Trusted, None),
            VoiceIdObservation {
                primary_fingerprint: Some(7),
                secondary_fingerprint: None,
                primary_embedding: None,
                secondary_embedding: None,
                spoof_risk: false,
            },
        );

        match out {
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(u) => {
                assert_eq!(u.reason_code, reason_codes::VID_FAIL_LOW_CONFIDENCE);
            }
            _ => panic!("expected unknown"),
        }
    }

    #[test]
    fn at_vid_18_require_primary_embedding_accepts_embedding_without_fingerprint() {
        let mut cfg = Ph1VoiceIdConfig::mvp_v1();
        cfg.require_primary_embedding = true;
        let mut rt = Ph1VoiceIdRuntime::new(
            cfg,
            vec![EnrolledSpeaker {
                speaker_id: SpeakerId::new("speaker_1").unwrap(),
                user_id: Some(UserId::new("user_1").unwrap()),
                fingerprint: 7,
                profile_embedding: Some(simulation_profile_embedding_from_seed(7)),
            }],
        )
        .unwrap();

        let out = rt.run(
            &req(0, vec![vad(0, 10)], false, DeviceTrustLevel::Trusted, None),
            VoiceIdObservation {
                primary_fingerprint: None,
                secondary_fingerprint: None,
                primary_embedding: Some(simulation_profile_embedding_from_seed(7)),
                secondary_embedding: None,
                spoof_risk: false,
            },
        );

        assert!(matches!(out, Ph1VoiceIdResponse::SpeakerAssertionOk(_)));
    }

    #[test]
    fn at_vid_impl_01_unknown_implementation_fails_closed() {
        let mut rt = Ph1VoiceIdRuntime::new(Ph1VoiceIdConfig::mvp_v1(), vec![]).unwrap();
        let out = rt.run_for_implementation(
            "PH1.VOICE.ID.999",
            &req(0, vec![vad(0, 10)], false, DeviceTrustLevel::Trusted, None),
            VoiceIdObservation {
                primary_fingerprint: Some(1),
                secondary_fingerprint: None,
                primary_embedding: None,
                secondary_embedding: None,
                spoof_risk: false,
            },
        );
        assert!(matches!(
            out,
            Err(ContractViolation::InvalidValue {
                field: "ph1_voice_id.implementation_id",
                reason: "unknown implementation_id",
            })
        ));
    }

    #[test]
    fn at_vid_impl_02_active_implementation_list_is_locked() {
        assert_eq!(PH1_VOICE_ID_ENGINE_ID, "PH1.VOICE.ID");
        assert_eq!(
            PH1_VOICE_ID_ACTIVE_IMPLEMENTATION_IDS,
            &["PH1.VOICE.ID.001"]
        );
    }
}
