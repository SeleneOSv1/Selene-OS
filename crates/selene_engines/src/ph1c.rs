#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1c::{
    ConfidenceBucket, LanguageHintConfidence, LanguageTag, Ph1cAuditMeta, Ph1cRequest,
    Ph1cResponse, Ph1cSttStrategy, QualityBucket, RetryAdvice, RouteClassUsed, RoutingModeUsed,
    SelectedSlot, TranscriptOk, TranscriptReject,
};
use selene_kernel_contracts::ph1k::{
    CaptureQualityClass, DeviceHealth, InterruptCandidateConfidenceBand, NetworkStabilityClass,
    RecoverabilityClass, VadDecisionConfidenceBand,
};
use selene_kernel_contracts::ReasonCodeId;

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.C reason-code namespace. Values are placeholders until the global registry is formalized.
    pub const STT_FAIL_EMPTY: ReasonCodeId = ReasonCodeId(0x4300_0001);
    pub const STT_FAIL_LOW_CONFIDENCE: ReasonCodeId = ReasonCodeId(0x4300_0002);
    pub const STT_FAIL_LOW_COVERAGE: ReasonCodeId = ReasonCodeId(0x4300_0003);
    pub const STT_FAIL_GARBLED: ReasonCodeId = ReasonCodeId(0x4300_0004);
    pub const STT_FAIL_LANGUAGE_MISMATCH: ReasonCodeId = ReasonCodeId(0x4300_0005);
    pub const STT_FAIL_AUDIO_DEGRADED: ReasonCodeId = ReasonCodeId(0x4300_0006);
    pub const STT_FAIL_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x4300_0007);
    pub const STT_FAIL_POLICY_RESTRICTED: ReasonCodeId = ReasonCodeId(0x4300_0008);
    pub const STT_FAIL_PROVIDER_TIMEOUT: ReasonCodeId = ReasonCodeId(0x4300_0009);
    pub const STT_FAIL_NETWORK_UNAVAILABLE: ReasonCodeId = ReasonCodeId(0x4300_000A);
    pub const STT_FAIL_BACKGROUND_SPEECH: ReasonCodeId = ReasonCodeId(0x4300_000B);
    pub const STT_FAIL_ECHO_SUSPECTED: ReasonCodeId = ReasonCodeId(0x4300_000C);
    pub const STT_FAIL_QUOTA_THROTTLED: ReasonCodeId = ReasonCodeId(0x4300_000D);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderSlot {
    Primary,
    Secondary,
    Tertiary,
}

#[derive(Debug, Clone)]
pub struct SttAttempt {
    pub provider: ProviderSlot,
    pub latency_ms: u32,
    pub transcript_text: String,
    pub language_tag: LanguageTag,
    pub avg_word_confidence: f32,
    pub low_confidence_ratio: f32,
    pub stable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ph1cConfig {
    pub max_attempts_per_turn: u8,
    pub max_total_latency_budget_ms: u32,

    pub min_avg_word_confidence: f32,
    pub max_low_confidence_ratio: f32,
    pub require_stable: bool,

    pub min_confidence_bucket_to_pass: ConfidenceBucket,

    pub min_chars_per_second: f32,
    pub min_chars_absolute: usize,
    pub routing_mode_used: RoutingModeUsed,
}

impl Ph1cConfig {
    pub fn mvp_desktop_v1() -> Self {
        Self {
            max_attempts_per_turn: 3,
            max_total_latency_budget_ms: 2_000,
            min_avg_word_confidence: 0.85,
            max_low_confidence_ratio: 0.15,
            require_stable: true,
            // Spec: "MED must not pass in MVP".
            min_confidence_bucket_to_pass: ConfidenceBucket::High,
            // Coverage heuristics are deliberately conservative in the skeleton.
            min_chars_per_second: 1.5,
            min_chars_absolute: 2,
            routing_mode_used: RoutingModeUsed::Lead,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1cRuntime {
    config: Ph1cConfig,
}

impl Ph1cRuntime {
    pub fn new(config: Ph1cConfig) -> Self {
        Self { config }
    }

    /// Deterministic evaluation over already-produced attempt outputs.
    ///
    /// In production, attempts would be produced by calling STT providers; this skeleton focuses on:
    /// budgets, quality gating, and non-leaky output contracts.
    pub fn run(&self, req: &Ph1cRequest, attempts: &[SttAttempt]) -> Ph1cResponse {
        if req.device_state_ref.health != DeviceHealth::Healthy {
            return Ph1cResponse::TranscriptReject(TranscriptReject::v1(
                reason_codes::STT_FAIL_AUDIO_DEGRADED,
                RetryAdvice::MoveCloser,
            ));
        }

        let strategy = select_stt_strategy(req);
        if matches!(strategy, Ph1cSttStrategy::ClarifyOnly) {
            let reject_meta = build_audit_meta(
                &self.config,
                0,
                0,
                SelectedSlot::None,
                0,
                QualityBucket::Low,
                QualityBucket::Low,
                QualityBucket::Low,
                None,
                None,
                Some(strategy_policy_profile_id(strategy).to_string()),
                Some("ph1k_handoff_clarify_only".to_string()),
            );
            return Ph1cResponse::TranscriptReject(TranscriptReject::v1_with_metadata(
                reason_codes::STT_FAIL_AUDIO_DEGRADED,
                RetryAdvice::SwitchToText,
                Some(reject_meta),
            ));
        }
        let ladder = select_provider_ladder(strategy);

        let mut attempts_used: u8 = 0;
        let mut total_latency_ms: u32 = 0;

        let mut best_ok: Option<(TranscriptOk, u32 /*score*/, ProviderSlot)> = None;
        let mut best_fail: Option<ReasonCodeId> = None;

        for slot in ladder {
            if attempts_used >= self.config.max_attempts_per_turn {
                best_fail.get_or_insert(reason_codes::STT_FAIL_BUDGET_EXCEEDED);
                break;
            }

            let Some(att) = attempts.iter().find(|a| a.provider == slot) else {
                continue;
            };

            if total_latency_ms.saturating_add(att.latency_ms)
                > self.config.max_total_latency_budget_ms
            {
                best_fail.get_or_insert(reason_codes::STT_FAIL_BUDGET_EXCEEDED);
                break;
            }

            attempts_used = attempts_used.saturating_add(1);
            total_latency_ms = total_latency_ms.saturating_add(att.latency_ms);

            match self.eval_attempt(req, att) {
                AttemptEval::Ok { out, score } => match &best_ok {
                    Some((_, best_score, _)) if *best_score >= score => {}
                    _ => best_ok = Some((out, score, slot)),
                },
                AttemptEval::Reject { reason } => {
                    best_fail = Some(select_more_specific_failure(best_fail, reason));
                }
            }
        }

        if let Some((ok, _, selected_provider)) = best_ok {
            let audit_meta = build_audit_meta(
                &self.config,
                attempts_used,
                attempts_used,
                selected_slot_for_provider(selected_provider),
                total_latency_ms,
                quality_bucket_from_confidence(ok.confidence_bucket),
                quality_bucket_from_confidence(ok.confidence_bucket),
                QualityBucket::High,
                None,
                None,
                Some(strategy_policy_profile_id(strategy).to_string()),
                Some("ph1k_handoff_strategy".to_string()),
            );
            let ok = TranscriptOk::v1_with_metadata(
                ok.transcript_text,
                ok.language_tag,
                ok.confidence_bucket,
                ok.uncertain_spans,
                Some(audit_meta),
            )
            .expect("transcript_ok with audit metadata must be constructible");
            return Ph1cResponse::TranscriptOk(ok);
        }

        let reason = best_fail.unwrap_or(reason_codes::STT_FAIL_BUDGET_EXCEEDED);
        let reject_meta = build_audit_meta(
            &self.config,
            attempts_used,
            attempts_used,
            SelectedSlot::None,
            total_latency_ms,
            QualityBucket::Low,
            QualityBucket::Low,
            QualityBucket::Low,
            None,
            None,
            Some(strategy_policy_profile_id(strategy).to_string()),
            Some("ph1k_handoff_strategy".to_string()),
        );
        Ph1cResponse::TranscriptReject(TranscriptReject::v1_with_metadata(
            reason,
            retry_advice_for(reason),
            Some(reject_meta),
        ))
    }

    fn eval_attempt(&self, req: &Ph1cRequest, att: &SttAttempt) -> AttemptEval {
        let t = att.transcript_text.trim();
        if t.is_empty() {
            return AttemptEval::Reject {
                reason: reason_codes::STT_FAIL_EMPTY,
            };
        }

        if is_garbled_or_stutter(t) {
            return AttemptEval::Reject {
                reason: reason_codes::STT_FAIL_GARBLED,
            };
        }

        if is_language_mismatch(req, &att.language_tag) {
            return AttemptEval::Reject {
                reason: reason_codes::STT_FAIL_LANGUAGE_MISMATCH,
            };
        }

        if !coverage_ok(&self.config, req, t) {
            return AttemptEval::Reject {
                reason: reason_codes::STT_FAIL_LOW_COVERAGE,
            };
        }

        let conf_bucket = confidence_bucket(att.avg_word_confidence, att.low_confidence_ratio);
        if bucket_rank(conf_bucket) < bucket_rank(self.config.min_confidence_bucket_to_pass) {
            return AttemptEval::Reject {
                reason: reason_codes::STT_FAIL_LOW_CONFIDENCE,
            };
        }

        if !confidence_ok(&self.config, att) {
            return AttemptEval::Reject {
                reason: reason_codes::STT_FAIL_LOW_CONFIDENCE,
            };
        }

        // Score is deterministic and used only to pick among multiple passing attempts.
        let score = score_attempt(att);
        let ok = TranscriptOk::v1(t.to_string(), att.language_tag.clone(), conf_bucket)
            .expect("TranscriptOk must be constructible for non-empty transcript");
        AttemptEval::Ok { out: ok, score }
    }
}

#[derive(Debug, Clone)]
enum AttemptEval {
    Ok { out: TranscriptOk, score: u32 },
    Reject { reason: ReasonCodeId },
}

fn strategy_policy_profile_id(strategy: Ph1cSttStrategy) -> &'static str {
    match strategy {
        Ph1cSttStrategy::Standard => "ph1k_handoff_standard",
        Ph1cSttStrategy::NoiseRobust => "ph1k_handoff_noise_robust",
        Ph1cSttStrategy::CloudAssist => "ph1k_handoff_cloud_assist",
        Ph1cSttStrategy::ClarifyOnly => "ph1k_handoff_clarify_only",
    }
}

fn select_stt_strategy(req: &Ph1cRequest) -> Ph1cSttStrategy {
    let Some(h) = &req.ph1k_handoff else {
        return Ph1cSttStrategy::Standard;
    };

    if matches!(
        h.degradation_class_bundle.capture_quality_class,
        CaptureQualityClass::Critical
    ) || matches!(
        h.degradation_class_bundle.recoverability_class,
        RecoverabilityClass::FailoverRequired
    ) {
        return Ph1cSttStrategy::ClarifyOnly;
    }

    if h.quality_metrics.packet_loss_pct >= 4.0
        || h.quality_metrics.snr_db < 14.0
        || matches!(
            h.degradation_class_bundle.network_stability_class,
            NetworkStabilityClass::Flaky | NetworkStabilityClass::Unstable
        )
    {
        return Ph1cSttStrategy::NoiseRobust;
    }

    if matches!(h.interrupt_confidence_band, InterruptCandidateConfidenceBand::Low)
        || matches!(h.vad_confidence_band, VadDecisionConfidenceBand::Low)
        || matches!(
            h.degradation_class_bundle.capture_quality_class,
            CaptureQualityClass::Degraded
        )
    {
        return Ph1cSttStrategy::CloudAssist;
    }

    Ph1cSttStrategy::Standard
}

fn select_provider_ladder(strategy: Ph1cSttStrategy) -> [ProviderSlot; 3] {
    match strategy {
        Ph1cSttStrategy::Standard => [
            ProviderSlot::Primary,
            ProviderSlot::Secondary,
            ProviderSlot::Tertiary,
        ],
        Ph1cSttStrategy::NoiseRobust => [
            ProviderSlot::Secondary,
            ProviderSlot::Primary,
            ProviderSlot::Tertiary,
        ],
        Ph1cSttStrategy::CloudAssist => [
            ProviderSlot::Tertiary,
            ProviderSlot::Primary,
            ProviderSlot::Secondary,
        ],
        Ph1cSttStrategy::ClarifyOnly => [
            ProviderSlot::Secondary,
            ProviderSlot::Tertiary,
            ProviderSlot::Primary,
        ],
    }
}

fn is_language_mismatch(req: &Ph1cRequest, actual: &LanguageTag) -> bool {
    let Some(hint) = &req.language_hint else {
        return false;
    };

    // Only enforce mismatch when the hint is strong.
    if hint.confidence != LanguageHintConfidence::High {
        return false;
    }

    hint.language_tag.as_str() != actual.as_str()
}

fn confidence_bucket(avg_word_conf: f32, low_ratio: f32) -> ConfidenceBucket {
    if avg_word_conf >= 0.90 && low_ratio <= 0.10 {
        ConfidenceBucket::High
    } else if avg_word_conf >= 0.80 {
        ConfidenceBucket::Med
    } else {
        ConfidenceBucket::Low
    }
}

fn bucket_rank(b: ConfidenceBucket) -> u8 {
    use ConfidenceBucket::*;
    match b {
        Low => 0,
        Med => 1,
        High => 2,
    }
}

fn confidence_ok(cfg: &Ph1cConfig, att: &SttAttempt) -> bool {
    if !(att.avg_word_confidence.is_finite() && att.low_confidence_ratio.is_finite()) {
        return false;
    }
    if !(0.0..=1.0).contains(&att.avg_word_confidence) {
        return false;
    }
    if !(0.0..=1.0).contains(&att.low_confidence_ratio) {
        return false;
    }
    if cfg.require_stable && !att.stable {
        return false;
    }
    att.avg_word_confidence >= cfg.min_avg_word_confidence
        && att.low_confidence_ratio <= cfg.max_low_confidence_ratio
}

fn coverage_ok(cfg: &Ph1cConfig, req: &Ph1cRequest, transcript: &str) -> bool {
    if transcript.chars().count() < cfg.min_chars_absolute {
        return false;
    }

    // Use the bounded segment duration as a conservative proxy for expected content length.
    let dur_ns = req
        .bounded_audio_segment_ref
        .t_end
        .0
        .saturating_sub(req.bounded_audio_segment_ref.t_start.0);
    let dur_s = (dur_ns as f32) / 1_000_000_000.0;
    if dur_s <= 0.0 {
        return false;
    }

    let min_chars = (cfg.min_chars_per_second * dur_s).ceil() as usize;
    transcript.chars().count() >= min_chars
}

fn is_garbled_or_stutter(transcript: &str) -> bool {
    // Detect extreme repetition ("I I I I"), which can appear as stutter or duplicate garbage.
    let tokens: Vec<&str> = transcript.split_whitespace().collect();
    if tokens.len() >= 4 {
        let mut run = 1usize;
        for i in 1..tokens.len() {
            if tokens[i].eq_ignore_ascii_case(tokens[i - 1]) {
                run += 1;
                if run >= 4 {
                    return true;
                }
            } else {
                run = 1;
            }
        }
    }

    // Provider "unknown" token patterns (keep conservative).
    let lower = transcript.to_ascii_lowercase();
    lower.contains("<unk>") || lower.contains("[unk]") || lower.contains("???")
}

fn score_attempt(att: &SttAttempt) -> u32 {
    // Higher is better.
    let conf = (att.avg_word_confidence * 1000.0).round() as i32;
    let penalty = (att.low_confidence_ratio * 500.0).round() as i32;
    let stable_bonus = if att.stable { 10 } else { 0 };
    (conf - penalty + stable_bonus).max(0) as u32
}

fn select_more_specific_failure(prev: Option<ReasonCodeId>, next: ReasonCodeId) -> ReasonCodeId {
    // Deterministic priority order: pick the "strongest" known failure to explain upstream.
    fn rank(rc: ReasonCodeId) -> u8 {
        match rc {
            reason_codes::STT_FAIL_AUDIO_DEGRADED => 0,
            reason_codes::STT_FAIL_BUDGET_EXCEEDED => 1,
            reason_codes::STT_FAIL_LANGUAGE_MISMATCH => 2,
            reason_codes::STT_FAIL_LOW_COVERAGE => 3,
            reason_codes::STT_FAIL_LOW_CONFIDENCE => 4,
            reason_codes::STT_FAIL_GARBLED => 5,
            reason_codes::STT_FAIL_EMPTY => 6,
            _ => 100,
        }
    }

    match prev {
        None => next,
        Some(p) => {
            if rank(next) < rank(p) {
                next
            } else {
                p
            }
        }
    }
}

fn retry_advice_for(reason: ReasonCodeId) -> RetryAdvice {
    match reason {
        reason_codes::STT_FAIL_EMPTY => RetryAdvice::Repeat,
        reason_codes::STT_FAIL_LOW_COVERAGE => RetryAdvice::Repeat,
        reason_codes::STT_FAIL_LOW_CONFIDENCE => RetryAdvice::SpeakSlower,
        reason_codes::STT_FAIL_GARBLED => RetryAdvice::QuietEnv,
        reason_codes::STT_FAIL_LANGUAGE_MISMATCH => RetryAdvice::Repeat,
        reason_codes::STT_FAIL_AUDIO_DEGRADED => RetryAdvice::MoveCloser,
        reason_codes::STT_FAIL_BUDGET_EXCEEDED => RetryAdvice::Repeat,
        reason_codes::STT_FAIL_QUOTA_THROTTLED => RetryAdvice::SwitchToText,
        reason_codes::STT_FAIL_POLICY_RESTRICTED => RetryAdvice::SwitchToText,
        reason_codes::STT_FAIL_NETWORK_UNAVAILABLE => RetryAdvice::SwitchToText,
        _ => RetryAdvice::Repeat,
    }
}

fn selected_slot_for_provider(provider: ProviderSlot) -> SelectedSlot {
    match provider {
        ProviderSlot::Primary => SelectedSlot::Primary,
        ProviderSlot::Secondary => SelectedSlot::Secondary,
        ProviderSlot::Tertiary => SelectedSlot::Tertiary,
    }
}

fn quality_bucket_from_confidence(c: ConfidenceBucket) -> QualityBucket {
    match c {
        ConfidenceBucket::High => QualityBucket::High,
        ConfidenceBucket::Med => QualityBucket::Med,
        ConfidenceBucket::Low => QualityBucket::Low,
    }
}

#[allow(clippy::too_many_arguments)]
fn build_audit_meta(
    cfg: &Ph1cConfig,
    attempt_count: u8,
    candidate_count: u8,
    selected_slot: SelectedSlot,
    total_latency_ms: u32,
    quality_coverage_bucket: QualityBucket,
    quality_confidence_bucket: QualityBucket,
    quality_plausibility_bucket: QualityBucket,
    tenant_vocabulary_pack_id: Option<String>,
    user_vocabulary_pack_id: Option<String>,
    policy_profile_id: Option<String>,
    stt_routing_policy_pack_id: Option<String>,
) -> Ph1cAuditMeta {
    Ph1cAuditMeta::v1(
        RouteClassUsed::OnDevice,
        attempt_count,
        candidate_count,
        selected_slot,
        cfg.routing_mode_used,
        attempt_count > 1,
        total_latency_ms,
        quality_coverage_bucket,
        quality_confidence_bucket,
        quality_plausibility_bucket,
        tenant_vocabulary_pack_id,
        user_vocabulary_pack_id,
        policy_profile_id,
        stt_routing_policy_pack_id,
    )
    .expect("ph1c_audit_meta must be constructible")
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1c::{LanguageHint, Ph1kToPh1cHandoff, SessionStateRef};
    use selene_kernel_contracts::ph1k::{
        AdvancedAudioQualityMetrics, AudioDeviceId, AudioStreamId, CaptureQualityClass,
        DegradationClassBundle, DeviceHealth, DeviceState, EchoRiskClass,
        InterruptCandidateConfidenceBand, NetworkStabilityClass, PreRollBufferId,
        RecoverabilityClass, VadDecisionConfidenceBand,
    };
    use selene_kernel_contracts::ph1w::BoundedAudioSegmentRef;
    use selene_kernel_contracts::ph1w::SessionState;
    use selene_kernel_contracts::MonotonicTimeNs;

    fn dev(id: &str) -> AudioDeviceId {
        AudioDeviceId::new(id).unwrap()
    }

    fn seg(duration_ms: u64) -> BoundedAudioSegmentRef {
        BoundedAudioSegmentRef::v1(
            AudioStreamId(1),
            PreRollBufferId(1),
            MonotonicTimeNs(0),
            MonotonicTimeNs(duration_ms * 1_000_000),
            MonotonicTimeNs(0),
            MonotonicTimeNs(0),
        )
        .unwrap()
    }

    fn req_with_duration(duration_ms: u64) -> Ph1cRequest {
        Ph1cRequest::v1(
            seg(duration_ms),
            SessionStateRef::v1(SessionState::Active, false),
            DeviceState::v1(dev("mic"), dev("spk"), DeviceHealth::Healthy, vec![]),
            None,
            None,
            None,
            None,
        )
        .unwrap()
    }

    fn handoff(
        interrupt_band: InterruptCandidateConfidenceBand,
        vad_band: VadDecisionConfidenceBand,
        snr_db: f32,
        packet_loss_pct: f32,
        degradation_class_bundle: DegradationClassBundle,
    ) -> Ph1kToPh1cHandoff {
        Ph1kToPh1cHandoff::v1(
            interrupt_band,
            vad_band,
            AdvancedAudioQualityMetrics::v1(snr_db, 0.03, 40.0, packet_loss_pct, 0.15, 16.0)
                .unwrap(),
            degradation_class_bundle,
        )
        .unwrap()
    }

    #[test]
    fn rejects_when_audio_degraded() {
        let mut req = req_with_duration(500);
        req.device_state_ref.health = DeviceHealth::Degraded;
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());
        let out = rt.run(&req, &[]);
        assert!(
            matches!(out, Ph1cResponse::TranscriptReject(r) if r.reason_code == reason_codes::STT_FAIL_AUDIO_DEGRADED)
        );
    }

    #[test]
    fn retries_and_returns_best_passing_without_leaking_provider() {
        let req = req_with_duration(800);
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());

        let attempts = vec![
            SttAttempt {
                provider: ProviderSlot::Primary,
                latency_ms: 200,
                transcript_text: "".to_string(),
                language_tag: LanguageTag::new("en").unwrap(),
                avg_word_confidence: 0.0,
                low_confidence_ratio: 1.0,
                stable: false,
            },
            SttAttempt {
                provider: ProviderSlot::Secondary,
                latency_ms: 300,
                transcript_text: "set meeting tomorrow".to_string(),
                language_tag: LanguageTag::new("en").unwrap(),
                avg_word_confidence: 0.92,
                low_confidence_ratio: 0.05,
                stable: true,
            },
        ];

        let out = rt.run(&req, &attempts);
        match out {
            Ph1cResponse::TranscriptOk(ok) => {
                assert_eq!(ok.transcript_text, "set meeting tomorrow");
            }
            _ => panic!("expected transcript_ok"),
        }
    }

    #[test]
    fn detects_language_mismatch_when_hint_is_high() {
        let mut req = req_with_duration(800);
        req.language_hint = Some(LanguageHint::v1(
            LanguageTag::new("en").unwrap(),
            LanguageHintConfidence::High,
        ));

        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());
        let attempts = vec![SttAttempt {
            provider: ProviderSlot::Primary,
            latency_ms: 200,
            transcript_text: "hola".to_string(),
            language_tag: LanguageTag::new("es").unwrap(),
            avg_word_confidence: 0.95,
            low_confidence_ratio: 0.02,
            stable: true,
        }];

        let out = rt.run(&req, &attempts);
        assert!(
            matches!(out, Ph1cResponse::TranscriptReject(r) if r.reason_code == reason_codes::STT_FAIL_LANGUAGE_MISMATCH)
        );
    }

    #[test]
    fn low_coverage_fails_for_long_audio_short_text() {
        let req = req_with_duration(5_000);
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());

        let attempts = vec![SttAttempt {
            provider: ProviderSlot::Primary,
            latency_ms: 200,
            transcript_text: "ok".to_string(),
            language_tag: LanguageTag::new("en").unwrap(),
            avg_word_confidence: 0.95,
            low_confidence_ratio: 0.02,
            stable: true,
        }];

        let out = rt.run(&req, &attempts);
        assert!(
            matches!(out, Ph1cResponse::TranscriptReject(r) if r.reason_code == reason_codes::STT_FAIL_LOW_COVERAGE)
        );
    }

    #[test]
    fn stutter_is_rejected_as_garbled() {
        let req = req_with_duration(800);
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());

        let attempts = vec![SttAttempt {
            provider: ProviderSlot::Primary,
            latency_ms: 200,
            transcript_text: "I I I I want that".to_string(),
            language_tag: LanguageTag::new("en").unwrap(),
            avg_word_confidence: 0.95,
            low_confidence_ratio: 0.02,
            stable: true,
        }];

        let out = rt.run(&req, &attempts);
        assert!(
            matches!(out, Ph1cResponse::TranscriptReject(r) if r.reason_code == reason_codes::STT_FAIL_GARBLED)
        );
    }

    #[test]
    fn budget_exceeded_fails_closed() {
        let req = req_with_duration(800);
        let rt = Ph1cRuntime::new(Ph1cConfig {
            max_attempts_per_turn: 1,
            max_total_latency_budget_ms: 100,
            ..Ph1cConfig::mvp_desktop_v1()
        });

        let attempts = vec![SttAttempt {
            provider: ProviderSlot::Primary,
            latency_ms: 200,
            transcript_text: "set meeting".to_string(),
            language_tag: LanguageTag::new("en").unwrap(),
            avg_word_confidence: 0.95,
            low_confidence_ratio: 0.02,
            stable: true,
        }];

        let out = rt.run(&req, &attempts);
        assert!(
            matches!(out, Ph1cResponse::TranscriptReject(r) if r.reason_code == reason_codes::STT_FAIL_BUDGET_EXCEEDED)
        );
    }

    #[test]
    fn ph1k_handoff_noise_robust_strategy_prefers_secondary_under_budget() {
        let mut req = req_with_duration(800);
        req.ph1k_handoff = Some(handoff(
            InterruptCandidateConfidenceBand::Medium,
            VadDecisionConfidenceBand::Medium,
            12.0,
            6.0,
            DegradationClassBundle {
                capture_quality_class: CaptureQualityClass::Guarded,
                echo_risk_class: EchoRiskClass::Elevated,
                network_stability_class: NetworkStabilityClass::Flaky,
                recoverability_class: RecoverabilityClass::Guarded,
            },
        ));

        let mut cfg = Ph1cConfig::mvp_desktop_v1();
        cfg.max_total_latency_budget_ms = 1_000;
        let rt = Ph1cRuntime::new(cfg);
        let attempts = vec![
            SttAttempt {
                provider: ProviderSlot::Primary,
                latency_ms: 900,
                transcript_text: "set meeting tomorrow".to_string(),
                language_tag: LanguageTag::new("en").unwrap(),
                avg_word_confidence: 0.96,
                low_confidence_ratio: 0.02,
                stable: true,
            },
            SttAttempt {
                provider: ProviderSlot::Secondary,
                latency_ms: 180,
                transcript_text: "set meeting tomorrow".to_string(),
                language_tag: LanguageTag::new("en").unwrap(),
                avg_word_confidence: 0.95,
                low_confidence_ratio: 0.02,
                stable: true,
            },
        ];
        let out = rt.run(&req, &attempts);
        assert!(matches!(out, Ph1cResponse::TranscriptOk(_)));
    }

    #[test]
    fn ph1k_handoff_critical_degradation_forces_clarify_only() {
        let mut req = req_with_duration(800);
        req.ph1k_handoff = Some(handoff(
            InterruptCandidateConfidenceBand::Low,
            VadDecisionConfidenceBand::Low,
            8.0,
            20.0,
            DegradationClassBundle {
                capture_quality_class: CaptureQualityClass::Critical,
                echo_risk_class: EchoRiskClass::High,
                network_stability_class: NetworkStabilityClass::Unstable,
                recoverability_class: RecoverabilityClass::FailoverRequired,
            },
        ));
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());
        let out = rt.run(&req, &[]);
        assert!(
            matches!(out, Ph1cResponse::TranscriptReject(r) if r.retry_advice == RetryAdvice::SwitchToText)
        );
    }

    #[test]
    fn ph1k_handoff_cloud_assist_strategy_prefers_tertiary_when_confidence_is_low() {
        let mut req = req_with_duration(800);
        req.ph1k_handoff = Some(handoff(
            InterruptCandidateConfidenceBand::Low,
            VadDecisionConfidenceBand::Low,
            26.0,
            0.5,
            DegradationClassBundle {
                capture_quality_class: CaptureQualityClass::Guarded,
                echo_risk_class: EchoRiskClass::Low,
                network_stability_class: NetworkStabilityClass::Stable,
                recoverability_class: RecoverabilityClass::Guarded,
            },
        ));

        let mut cfg = Ph1cConfig::mvp_desktop_v1();
        cfg.max_attempts_per_turn = 1;
        let rt = Ph1cRuntime::new(cfg);
        let attempts = vec![
            SttAttempt {
                provider: ProviderSlot::Primary,
                latency_ms: 90,
                transcript_text: "".to_string(),
                language_tag: LanguageTag::new("en").unwrap(),
                avg_word_confidence: 0.10,
                low_confidence_ratio: 1.0,
                stable: false,
            },
            SttAttempt {
                provider: ProviderSlot::Tertiary,
                latency_ms: 95,
                transcript_text: "use cloud assist path".to_string(),
                language_tag: LanguageTag::new("en").unwrap(),
                avg_word_confidence: 0.96,
                low_confidence_ratio: 0.02,
                stable: true,
            },
        ];

        match rt.run(&req, &attempts) {
            Ph1cResponse::TranscriptOk(ok) => {
                assert_eq!(ok.transcript_text, "use cloud assist path")
            }
            other => panic!("expected transcript_ok from tertiary-first cloud assist, got: {other:?}"),
        }
    }

    #[test]
    fn ph1k_handoff_standard_strategy_prefers_primary_when_quality_is_clean() {
        let mut req = req_with_duration(800);
        req.ph1k_handoff = Some(handoff(
            InterruptCandidateConfidenceBand::High,
            VadDecisionConfidenceBand::High,
            30.0,
            0.2,
            DegradationClassBundle {
                capture_quality_class: CaptureQualityClass::Clear,
                echo_risk_class: EchoRiskClass::Low,
                network_stability_class: NetworkStabilityClass::Stable,
                recoverability_class: RecoverabilityClass::Fast,
            },
        ));

        let mut cfg = Ph1cConfig::mvp_desktop_v1();
        cfg.max_attempts_per_turn = 1;
        let rt = Ph1cRuntime::new(cfg);
        let attempts = vec![
            SttAttempt {
                provider: ProviderSlot::Primary,
                latency_ms: 80,
                transcript_text: "primary strategy path".to_string(),
                language_tag: LanguageTag::new("en").unwrap(),
                avg_word_confidence: 0.97,
                low_confidence_ratio: 0.02,
                stable: true,
            },
            SttAttempt {
                provider: ProviderSlot::Secondary,
                latency_ms: 70,
                transcript_text: "secondary strategy path".to_string(),
                language_tag: LanguageTag::new("en").unwrap(),
                avg_word_confidence: 0.98,
                low_confidence_ratio: 0.01,
                stable: true,
            },
        ];

        match rt.run(&req, &attempts) {
            Ph1cResponse::TranscriptOk(ok) => {
                assert_eq!(ok.transcript_text, "primary strategy path")
            }
            other => panic!("expected transcript_ok from primary-first standard path, got: {other:?}"),
        }
    }
}
