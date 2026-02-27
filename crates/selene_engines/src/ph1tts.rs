#![forbid(unsafe_code)]

use crate::ph1d::decode_normalized_output_json;
use selene_kernel_contracts::ph1d::{
    Ph1dProviderCallResponse, Ph1dProviderStatus, Ph1dProviderTask, Ph1dProviderValidationStatus,
    PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_HASH_V1,
};
use selene_kernel_contracts::ph1k::TtsPlaybackActiveEvent;
use selene_kernel_contracts::ph1tts::{
    AnswerId, Ph1ttsEvent, Ph1ttsRequest, SpokenCursor, TtsControl, TtsFailed, TtsProgress,
    TtsStarted, TtsStopReason, TtsStopped, VoiceId,
};
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.TTS reason-code namespace. Values are placeholders until the global registry is formalized.
    pub const TTS_FAIL_INVALID_REQUEST: ReasonCodeId = ReasonCodeId(0x5454_0001);
    pub const TTS_FAIL_PROVIDER: ReasonCodeId = ReasonCodeId(0x5454_0002);
    pub const TTS_FAIL_POLICY_RESTRICTED: ReasonCodeId = ReasonCodeId(0x5454_0003);
    pub const TTS_FAIL_PROVIDER_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x5454_0004);
    pub const TTS_FAIL_TEXT_ONLY_FAILSAFE: ReasonCodeId = ReasonCodeId(0x5454_0005);
    pub const TTS_FAIL_PROVIDER_TIMEOUT: ReasonCodeId = ReasonCodeId(0x5454_0006);
    pub const TTS_FAIL_MEANING_DRIFT: ReasonCodeId = ReasonCodeId(0x5454_0007);
    pub const TTS_FAIL_SHADOW_INPUT_INVALID: ReasonCodeId = ReasonCodeId(0x5454_0008);
    pub const TTS_FAIL_SHADOW_PROVIDER_TRUTH_INVALID: ReasonCodeId = ReasonCodeId(0x5454_0009);
    pub const TTS_FAIL_SHADOW_PROMOTION_BLOCKED: ReasonCodeId = ReasonCodeId(0x5454_000A);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtsState {
    Idle,
    Playing,
    Paused,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TtsPlayback {
    pub answer_id: AnswerId,
    pub voice_id: VoiceId,
    pub response_text: String,
    pub ms_played: u32,
    pub estimated_total_ms: u32,
    pub segment_ends: Vec<u32>,
    pub spoken_cursor_byte: u32,
    pub started_at: MonotonicTimeNs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1ttsOutput {
    Event(Ph1ttsEvent),
    PlaybackMarker(TtsPlaybackActiveEvent),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtsProviderSlot {
    Primary,
    Secondary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TtsProviderAttempt {
    pub provider: TtsProviderSlot,
    pub response: Ph1dProviderCallResponse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1ttsShadowSliceKey {
    pub locale: String,
    pub device_route: String,
    pub tenant_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TtsInhouseShadowAttempt {
    pub rendered_text: String,
    pub language_tag: String,
    pub estimated_duration_ms: u32,
    pub latency_ms: u32,
    pub audio_output_ready: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ph1ttsShadowRouteDecision {
    HoldShadow,
    EligibleForPromotion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1ttsShadowRouteOutcome {
    pub slice_key: Ph1ttsShadowSliceKey,
    pub duration_delta_ms: u32,
    pub latency_delta_ms: i32,
    pub governed_gate_passed: bool,
    pub decision: Ph1ttsShadowRouteDecision,
    pub block_reason_code: Option<ReasonCodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1ttsConfig {
    pub max_ms_played: u32,
    pub max_attempts_per_turn: u8,
    pub max_total_latency_budget_ms: u32,
    pub max_retries_per_provider: u8,
}

impl Ph1ttsConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_ms_played: 60_000,
            max_attempts_per_turn: 3,
            max_total_latency_budget_ms: 2_000,
            max_retries_per_provider: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1ttsRuntime {
    config: Ph1ttsConfig,
    state: TtsState,
    playback: Option<TtsPlayback>,
}

impl Ph1ttsRuntime {
    pub fn new(config: Ph1ttsConfig) -> Self {
        Self {
            config,
            state: TtsState::Idle,
            playback: None,
        }
    }

    pub fn state(&self) -> TtsState {
        self.state
    }

    /// Exposes the current response text for testing "no meaning drift" (PH1.TTS must not rewrite).
    pub fn current_text(&self) -> Option<&str> {
        self.playback.as_ref().map(|p| p.response_text.as_str())
    }

    pub fn handle(&mut self, now: MonotonicTimeNs, req: Ph1ttsRequest) -> Vec<Ph1ttsOutput> {
        let req_result = validate_request(&req);
        if let Err(reason_code) = req_result {
            return fail_output(req.answer_id, reason_code);
        }

        match req.tts_control {
            TtsControl::Play => self.on_play(now, req.answer_id, req.response_text),
            TtsControl::Cancel => self.on_cancel(now, req.answer_id),
            TtsControl::Pause => self.on_pause(req.answer_id),
            TtsControl::Resume => self.on_resume(req.answer_id),
        }
    }

    /// Coupled PH1.TTS->PH1.D provider ladder:
    /// PRIMARY(OpenAI) -> SECONDARY(Google) -> text-only fail-safe.
    pub fn handle_with_provider_ladder(
        &mut self,
        now: MonotonicTimeNs,
        req: Ph1ttsRequest,
        attempts: &[TtsProviderAttempt],
    ) -> Vec<Ph1ttsOutput> {
        let req_result = validate_request(&req);
        if let Err(reason_code) = req_result {
            return fail_output(req.answer_id, reason_code);
        }
        match req.tts_control {
            TtsControl::Play => self.on_play_with_provider_ladder(now, req, attempts),
            TtsControl::Cancel => self.on_cancel(now, req.answer_id),
            TtsControl::Pause => self.on_pause(req.answer_id),
            TtsControl::Resume => self.on_resume(req.answer_id),
        }
    }

    pub fn evaluate_inhouse_shadow_route(
        &self,
        req: &Ph1ttsRequest,
        slice_key: Ph1ttsShadowSliceKey,
        provider_truth: &Ph1dProviderCallResponse,
        inhouse_shadow: &TtsInhouseShadowAttempt,
        governed_gate_passed: bool,
    ) -> Result<Ph1ttsShadowRouteOutcome, ReasonCodeId> {
        validate_request(req)?;
        if !valid_tts_shadow_slice_key(&slice_key)
            || !valid_tts_inhouse_shadow_attempt(inhouse_shadow)
        {
            return Err(reason_codes::TTS_FAIL_SHADOW_INPUT_INVALID);
        }
        if !provider_attempt_ok(req, provider_truth) {
            return Err(reason_codes::TTS_FAIL_SHADOW_PROVIDER_TRUTH_INVALID);
        }
        let provider_truth = parse_tts_provider_truth(provider_truth)
            .ok_or(reason_codes::TTS_FAIL_SHADOW_PROVIDER_TRUTH_INVALID)?;
        if !locale_family_matches_tts(&slice_key.locale, &provider_truth.language_tag)
            || !locale_family_matches_tts(&slice_key.locale, &inhouse_shadow.language_tag)
        {
            return Err(reason_codes::TTS_FAIL_SHADOW_INPUT_INVALID);
        }

        let duration_delta_ms = provider_truth
            .estimated_duration_ms
            .abs_diff(inhouse_shadow.estimated_duration_ms);
        let latency_delta_ms = inhouse_shadow.latency_ms as i32 - provider_truth.latency_ms as i32;

        let decision = if governed_gate_passed
            && inhouse_shadow.audio_output_ready
            && inhouse_shadow.rendered_text == req.response_text
            && inhouse_shadow.rendered_text == provider_truth.text_output
            && duration_delta_ms <= 250
            && inhouse_shadow.latency_ms <= provider_truth.latency_ms.saturating_add(250)
        {
            Ph1ttsShadowRouteDecision::EligibleForPromotion
        } else {
            Ph1ttsShadowRouteDecision::HoldShadow
        };

        let block_reason_code =
            if matches!(decision, Ph1ttsShadowRouteDecision::EligibleForPromotion) {
                None
            } else if !governed_gate_passed {
                Some(reason_codes::TTS_FAIL_SHADOW_PROMOTION_BLOCKED)
            } else if inhouse_shadow.rendered_text != req.response_text
                || inhouse_shadow.rendered_text != provider_truth.text_output
            {
                Some(reason_codes::TTS_FAIL_MEANING_DRIFT)
            } else if !inhouse_shadow.audio_output_ready {
                Some(reason_codes::TTS_FAIL_PROVIDER)
            } else if duration_delta_ms > 250
                || inhouse_shadow.latency_ms > provider_truth.latency_ms.saturating_add(250)
            {
                Some(reason_codes::TTS_FAIL_PROVIDER_BUDGET_EXCEEDED)
            } else {
                Some(reason_codes::TTS_FAIL_SHADOW_PROMOTION_BLOCKED)
            };

        Ok(Ph1ttsShadowRouteOutcome {
            slice_key,
            duration_delta_ms,
            latency_delta_ms,
            governed_gate_passed,
            decision,
            block_reason_code,
        })
    }

    pub fn tick_progress(&mut self, now: MonotonicTimeNs, delta_ms: u32) -> Vec<Ph1ttsOutput> {
        let Some(p) = self.playback.as_mut() else {
            return vec![];
        };
        if self.state != TtsState::Playing {
            return vec![];
        }

        p.ms_played = p.ms_played.saturating_add(delta_ms);
        let (byte_offset, segments_spoken, segments_total) = cursor_for(
            p.ms_played,
            p.estimated_total_ms,
            p.response_text.as_bytes().len() as u32,
            &p.segment_ends,
        );
        p.spoken_cursor_byte = byte_offset;

        if p.ms_played >= p.estimated_total_ms {
            // Deterministic completion at the estimated end (bounded by config.max_ms_played).
            let answer_id = p.answer_id;
            let final_cursor = SpokenCursor::v1(
                p.response_text.as_bytes().len() as u32,
                segments_total,
                segments_total,
            )
            .expect("final spoken cursor must be constructible");
            self.playback = None;
            self.state = TtsState::Idle;
            return vec![
                Ph1ttsOutput::Event(Ph1ttsEvent::Stopped(TtsStopped {
                    schema_version: selene_kernel_contracts::SchemaVersion(1),
                    answer_id,
                    reason: TtsStopReason::Completed,
                    t_stopped: now,
                    spoken_cursor: final_cursor,
                })),
                Ph1ttsOutput::PlaybackMarker(TtsPlaybackActiveEvent::v1(false, now)),
            ];
        }

        vec![Ph1ttsOutput::Event(Ph1ttsEvent::Progress(TtsProgress {
            schema_version: selene_kernel_contracts::SchemaVersion(1),
            answer_id: p.answer_id,
            ms_played: p.ms_played,
            spoken_cursor: SpokenCursor::v1(byte_offset, segments_spoken, segments_total)
                .expect("spoken cursor must be constructible"),
        }))]
    }

    fn on_play_with_provider_ladder(
        &mut self,
        now: MonotonicTimeNs,
        req: Ph1ttsRequest,
        attempts: &[TtsProviderAttempt],
    ) -> Vec<Ph1ttsOutput> {
        let mut attempts_used: u8 = 0;
        let mut total_latency_ms: u32 = 0;
        let mut budget_exceeded = false;
        let provider_attempt_cap = 1u8.saturating_add(self.config.max_retries_per_provider);

        for slot in [TtsProviderSlot::Primary, TtsProviderSlot::Secondary] {
            let mut slot_attempts: u8 = 0;
            for att in attempts.iter().filter(|a| a.provider == slot) {
                if slot_attempts >= provider_attempt_cap {
                    break;
                }
                if attempts_used >= self.config.max_attempts_per_turn {
                    budget_exceeded = true;
                    break;
                }
                if total_latency_ms.saturating_add(att.response.provider_latency_ms)
                    > self.config.max_total_latency_budget_ms
                {
                    budget_exceeded = true;
                    break;
                }

                slot_attempts = slot_attempts.saturating_add(1);
                attempts_used = attempts_used.saturating_add(1);
                total_latency_ms =
                    total_latency_ms.saturating_add(att.response.provider_latency_ms);

                if provider_attempt_ok(&req, &att.response) {
                    return self.on_play(now, req.answer_id, req.response_text);
                }
            }
            if budget_exceeded {
                break;
            }
        }

        if budget_exceeded {
            return fail_output(
                req.answer_id,
                reason_codes::TTS_FAIL_PROVIDER_BUDGET_EXCEEDED,
            );
        }
        fail_output(req.answer_id, reason_codes::TTS_FAIL_TEXT_ONLY_FAILSAFE)
    }

    fn on_play(
        &mut self,
        now: MonotonicTimeNs,
        answer_id: AnswerId,
        response_text: String,
    ) -> Vec<Ph1ttsOutput> {
        // Deterministic voice selection placeholder.
        let voice_id = VoiceId::new("VOICE_DEFAULT").expect("voice id must be constructible");

        let segment_ends = segment_plan(&response_text);
        let estimated_total_ms = estimate_total_ms(&response_text, self.config.max_ms_played);

        self.playback = Some(TtsPlayback {
            answer_id,
            voice_id: voice_id.clone(),
            response_text,
            ms_played: 0,
            estimated_total_ms,
            segment_ends,
            spoken_cursor_byte: 0,
            started_at: now,
        });
        self.state = TtsState::Playing;

        vec![
            Ph1ttsOutput::Event(Ph1ttsEvent::Started(TtsStarted {
                schema_version: selene_kernel_contracts::SchemaVersion(1),
                answer_id,
                voice_id,
                t_started: now,
            })),
            Ph1ttsOutput::PlaybackMarker(TtsPlaybackActiveEvent::v1(true, now)),
        ]
    }

    fn on_cancel(&mut self, now: MonotonicTimeNs, answer_id: AnswerId) -> Vec<Ph1ttsOutput> {
        let Some(p) = self.playback.as_ref() else {
            return vec![];
        };
        if p.answer_id != answer_id {
            return vec![];
        }
        let (byte_offset, segments_spoken, segments_total) = cursor_for(
            p.ms_played,
            p.estimated_total_ms,
            p.response_text.as_bytes().len() as u32,
            &p.segment_ends,
        );

        let cursor = SpokenCursor::v1(byte_offset, segments_spoken, segments_total)
            .expect("spoken cursor must be constructible");

        self.playback = None;
        self.state = TtsState::Idle;
        vec![
            Ph1ttsOutput::Event(Ph1ttsEvent::Stopped(TtsStopped {
                schema_version: selene_kernel_contracts::SchemaVersion(1),
                answer_id,
                reason: TtsStopReason::Cancelled,
                t_stopped: now,
                spoken_cursor: cursor,
            })),
            Ph1ttsOutput::PlaybackMarker(TtsPlaybackActiveEvent::v1(false, now)),
        ]
    }

    fn on_pause(&mut self, answer_id: AnswerId) -> Vec<Ph1ttsOutput> {
        if self
            .playback
            .as_ref()
            .is_none_or(|p| p.answer_id != answer_id)
        {
            return vec![];
        }
        if self.state == TtsState::Playing {
            self.state = TtsState::Paused;
        }
        vec![]
    }

    fn on_resume(&mut self, answer_id: AnswerId) -> Vec<Ph1ttsOutput> {
        if self
            .playback
            .as_ref()
            .is_none_or(|p| p.answer_id != answer_id)
        {
            return vec![];
        }
        if self.state == TtsState::Paused {
            self.state = TtsState::Playing;
        }
        vec![]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TtsProviderTruth {
    text_output: String,
    language_tag: String,
    estimated_duration_ms: u32,
    latency_ms: u32,
}

fn parse_tts_provider_truth(response: &Ph1dProviderCallResponse) -> Option<TtsProviderTruth> {
    let normalized_output_json = response.normalized_output_json.as_deref()?;
    let normalized = decode_normalized_output_json(normalized_output_json).ok()?;
    if normalized.provider_task != Ph1dProviderTask::TtsSynthesize {
        return None;
    }
    let text_output = normalized.text_output?.trim().to_string();
    if text_output.is_empty() {
        return None;
    }
    let language_tag = normalize_locale_tag(normalized.language_tag.as_deref().unwrap_or("und"));
    if language_tag.is_empty() {
        return None;
    }
    let estimated_duration_ms = normalized.estimated_duration_ms?;
    if estimated_duration_ms == 0 || estimated_duration_ms > 600_000 {
        return None;
    }
    Some(TtsProviderTruth {
        text_output,
        language_tag,
        estimated_duration_ms,
        latency_ms: response.provider_latency_ms,
    })
}

fn valid_tts_shadow_slice_key(slice_key: &Ph1ttsShadowSliceKey) -> bool {
    !normalize_locale_tag(&slice_key.locale).is_empty()
        && is_provider_token(&slice_key.device_route, 64)
        && is_provider_token(&slice_key.tenant_id, 128)
}

fn valid_tts_inhouse_shadow_attempt(attempt: &TtsInhouseShadowAttempt) -> bool {
    !attempt.rendered_text.trim().is_empty()
        && attempt.rendered_text.len() <= 65_536
        && !normalize_locale_tag(&attempt.language_tag).is_empty()
        && (1..=600_000).contains(&attempt.estimated_duration_ms)
        && attempt.latency_ms <= 120_000
}

fn locale_family_matches_tts(expected: &str, actual: &str) -> bool {
    let expected = normalize_locale_tag(expected);
    let actual = normalize_locale_tag(actual);
    if expected.is_empty() || actual.is_empty() {
        return false;
    }
    if expected == actual {
        return true;
    }
    let expected_family = expected.split('-').next().unwrap_or_default();
    let actual_family = actual.split('-').next().unwrap_or_default();
    !expected_family.is_empty() && expected_family == actual_family
}

fn normalize_locale_tag(tag: &str) -> String {
    let mut out = String::with_capacity(tag.len());
    for c in tag.trim().chars() {
        if c == '_' {
            out.push('-');
        } else if c.is_ascii_alphanumeric() || c == '-' {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

fn is_provider_token(value: &str, max_len: usize) -> bool {
    if value.trim().is_empty() || value.len() > max_len {
        return false;
    }
    value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | ':' | '/'))
}

fn validate_request(req: &Ph1ttsRequest) -> Result<(), selene_kernel_contracts::ReasonCodeId> {
    if req.validate().is_err() {
        return Err(reason_codes::TTS_FAIL_INVALID_REQUEST);
    }
    if matches!(req.tts_control, TtsControl::Play)
        && (req.policy_context_ref.do_not_disturb || req.policy_context_ref.privacy_mode)
    {
        return Err(reason_codes::TTS_FAIL_POLICY_RESTRICTED);
    }
    Ok(())
}

fn fail_output(
    answer_id: AnswerId,
    reason_code: selene_kernel_contracts::ReasonCodeId,
) -> Vec<Ph1ttsOutput> {
    vec![Ph1ttsOutput::Event(Ph1ttsEvent::Failed(TtsFailed {
        schema_version: selene_kernel_contracts::SchemaVersion(1),
        answer_id,
        reason_code,
    }))]
}

fn provider_attempt_ok(req: &Ph1ttsRequest, response: &Ph1dProviderCallResponse) -> bool {
    if response.validate().is_err() {
        return false;
    }
    if response.provider_task != Ph1dProviderTask::TtsSynthesize {
        return false;
    }
    if response.provider_status == Ph1dProviderStatus::Error {
        return false;
    }
    if response.validation_status != Ph1dProviderValidationStatus::SchemaOk {
        return false;
    }
    if response.normalized_output_schema_hash
        != Some(PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_HASH_V1)
    {
        return false;
    }
    let Some(normalized_output_json) = response.normalized_output_json.as_deref() else {
        return false;
    };
    let Ok(normalized) = decode_normalized_output_json(normalized_output_json) else {
        return false;
    };
    if normalized.provider_task != Ph1dProviderTask::TtsSynthesize {
        return false;
    }
    if normalized.text_output.as_deref() != Some(req.response_text.as_str()) {
        return false;
    }
    normalized.audio_output_ref.is_some()
        && normalized.audio_content_type.is_some()
        && normalized.estimated_duration_ms.is_some()
}

fn estimate_total_ms(text: &str, max_ms: u32) -> u32 {
    // Deterministic estimate: base + per-byte. Bounded by max_ms.
    let len_bytes = text.as_bytes().len() as u32;
    let ms = 200u32.saturating_add(len_bytes.saturating_mul(12));
    ms.min(max_ms).max(1)
}

fn segment_plan(text: &str) -> Vec<u32> {
    // Deterministic segmentation for streaming + safe cursor boundaries:
    // split at ASCII sentence terminators and newlines, and always include final end.
    let bytes = text.as_bytes();
    let mut ends: Vec<u32> = Vec::new();
    for (i, &b) in bytes.iter().enumerate() {
        if matches!(b, b'.' | b'?' | b'!' | b'\n') {
            ends.push((i + 1) as u32);
        }
    }
    let len = bytes.len() as u32;
    if *ends.last().unwrap_or(&0) != len {
        ends.push(len);
    }
    ends.sort_unstable();
    ends.dedup();

    // Bound segment count (fail-safe: one segment).
    if ends.is_empty() {
        return vec![len.max(1)];
    }
    if ends.len() > 512 {
        return vec![len];
    }
    ends
}

fn cursor_for(
    ms_played: u32,
    total_ms: u32,
    text_len_bytes: u32,
    segment_ends: &[u32],
) -> (u32, u16, u16) {
    let total_ms = total_ms.max(1);
    let raw = ((ms_played as u64).saturating_mul(text_len_bytes as u64) / total_ms as u64) as u32;
    let raw = raw.min(text_len_bytes);

    // Cursor advances only to the end of the last fully spoken segment.
    let mut byte_offset: u32 = 0;
    let mut segments_spoken: u16 = 0;
    let segments_total: u16 = segment_ends.len().min(u16::MAX as usize) as u16;
    for (idx, &end) in segment_ends.iter().enumerate() {
        if end <= raw {
            byte_offset = end;
            segments_spoken = (idx + 1).min(u16::MAX as usize) as u16;
        } else {
            break;
        }
    }
    (byte_offset, segments_spoken, segments_total.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ph1d::reason_codes as ph1d_reason_codes;
    use selene_kernel_contracts::ph1c::{LanguageTag, SessionStateRef};
    use selene_kernel_contracts::ph1d::{
        Ph1dProviderCallResponse, Ph1dProviderStatus, Ph1dProviderTask,
        Ph1dProviderValidationStatus, PolicyContextRef, RequestId, SafetyTier,
        PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_HASH_V1,
    };
    use selene_kernel_contracts::ph1tts::{
        BargeInPolicyRef, StyleModifier, StyleProfileRef, VoiceRenderPlan,
    };
    use selene_kernel_contracts::ph1w::SessionState;

    fn plan() -> VoiceRenderPlan {
        VoiceRenderPlan::v1(
            StyleProfileRef::Gentle,
            vec![StyleModifier::Warm],
            BargeInPolicyRef::Standard,
            LanguageTag::new("en").unwrap(),
            None,
        )
    }

    fn play_req(answer_id: AnswerId, text: &str) -> Ph1ttsRequest {
        Ph1ttsRequest::v1(
            answer_id,
            text.to_string(),
            TtsControl::Play,
            SessionStateRef::v1(SessionState::Active, true),
            plan(),
            PolicyContextRef::v1(false, false, SafetyTier::Standard),
        )
        .unwrap()
    }

    fn control_req(answer_id: AnswerId, control: TtsControl) -> Ph1ttsRequest {
        Ph1ttsRequest::v1(
            answer_id,
            "ignored".to_string(),
            control,
            SessionStateRef::v1(SessionState::Active, true),
            plan(),
            PolicyContextRef::v1(false, false, SafetyTier::Standard),
        )
        .unwrap()
    }

    fn provider_ok_response(
        request_id: u64,
        provider_id: &str,
        model_id: &str,
        text: &str,
        latency_ms: u32,
    ) -> Ph1dProviderCallResponse {
        let normalized_output_json = serde_json::json!({
            "schema_version": 1,
            "provider_task": "TTS_SYNTHESIZE",
            "text_output": text,
            "language_tag": "en",
            "confidence_bp": serde_json::Value::Null,
            "stable": serde_json::Value::Null,
            "audio_output_ref": "audio://tts/chunk_1",
            "audio_content_type": "audio/wav",
            "estimated_duration_ms": 900
        })
        .to_string();

        Ph1dProviderCallResponse::v1(
            1,
            1,
            RequestId(request_id),
            format!("tts_idem_{request_id}"),
            Some(format!("provider_call_{request_id}")),
            provider_id.to_string(),
            Ph1dProviderTask::TtsSynthesize,
            model_id.to_string(),
            Ph1dProviderStatus::Ok,
            latency_ms,
            20,
            None,
            Some(PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_HASH_V1),
            Some(normalized_output_json),
            Ph1dProviderValidationStatus::SchemaOk,
            ph1d_reason_codes::D_PROVIDER_OK,
        )
        .unwrap()
    }

    fn provider_timeout_response(
        request_id: u64,
        provider_id: &str,
        model_id: &str,
        latency_ms: u32,
    ) -> Ph1dProviderCallResponse {
        Ph1dProviderCallResponse::v1(
            1,
            1,
            RequestId(request_id),
            format!("tts_idem_{request_id}"),
            Some(format!("provider_call_{request_id}")),
            provider_id.to_string(),
            Ph1dProviderTask::TtsSynthesize,
            model_id.to_string(),
            Ph1dProviderStatus::Error,
            latency_ms,
            20,
            None,
            None,
            None,
            Ph1dProviderValidationStatus::SchemaFail,
            ph1d_reason_codes::D_PROVIDER_TIMEOUT,
        )
        .unwrap()
    }

    fn attempt(
        provider: TtsProviderSlot,
        response: Ph1dProviderCallResponse,
    ) -> TtsProviderAttempt {
        TtsProviderAttempt { provider, response }
    }

    fn inhouse_shadow_attempt(
        text: &str,
        language_tag: &str,
        estimated_duration_ms: u32,
        latency_ms: u32,
    ) -> TtsInhouseShadowAttempt {
        TtsInhouseShadowAttempt {
            rendered_text: text.to_string(),
            language_tag: language_tag.to_string(),
            estimated_duration_ms,
            latency_ms,
            audio_output_ready: true,
        }
    }

    #[test]
    fn at_tts_01_instant_cancel() {
        let mut rt = Ph1ttsRuntime::new(Ph1ttsConfig::mvp_v1());
        let now = MonotonicTimeNs(1);
        let id = AnswerId(1);

        let out = rt.handle(now, play_req(id, "hello"));
        assert!(out
            .iter()
            .any(|o| matches!(o, Ph1ttsOutput::Event(Ph1ttsEvent::Started(_)))));
        assert_eq!(rt.state(), TtsState::Playing);

        let cancel = Ph1ttsRequest::v1(
            id,
            "ignored".to_string(),
            TtsControl::Cancel,
            SessionStateRef::v1(SessionState::Active, true),
            plan(),
            PolicyContextRef::v1(false, false, SafetyTier::Standard),
        )
        .unwrap();

        let out = rt.handle(MonotonicTimeNs(2), cancel);
        assert!(out.iter().any(|o| matches!(o, Ph1ttsOutput::Event(Ph1ttsEvent::Stopped(s)) if s.reason == TtsStopReason::Cancelled)));
        assert_eq!(rt.state(), TtsState::Idle);
    }

    #[test]
    fn at_tts_03_no_meaning_drift_text_is_preserved() {
        let mut rt = Ph1ttsRuntime::new(Ph1ttsConfig::mvp_v1());
        let id = AnswerId(1);
        let text = "Meeting with John at 3pm";
        rt.handle(MonotonicTimeNs(1), play_req(id, text));
        assert_eq!(rt.current_text(), Some(text));
    }

    #[test]
    fn completes_deterministically_after_max_ms() {
        let mut rt = Ph1ttsRuntime::new(Ph1ttsConfig {
            max_ms_played: 10,
            ..Ph1ttsConfig::mvp_v1()
        });
        let id = AnswerId(1);
        rt.handle(MonotonicTimeNs(1), play_req(id, "hello"));
        let out = rt.tick_progress(MonotonicTimeNs(2), 10);
        assert!(out.iter().any(|o| matches!(o, Ph1ttsOutput::Event(Ph1ttsEvent::Stopped(s)) if s.reason == TtsStopReason::Completed)));
    }

    #[test]
    fn at_tts_07_policy_blocks_audible_output() {
        let mut rt = Ph1ttsRuntime::new(Ph1ttsConfig::mvp_v1());
        let id = AnswerId(1);
        let req = Ph1ttsRequest::v1(
            id,
            "hello".to_string(),
            TtsControl::Play,
            SessionStateRef::v1(SessionState::Active, true),
            plan(),
            PolicyContextRef::v1(false, true, SafetyTier::Standard),
        )
        .unwrap();
        let out = rt.handle(MonotonicTimeNs(1), req);
        assert!(
            out.iter().any(|o| matches!(o, Ph1ttsOutput::Event(Ph1ttsEvent::Failed(f)) if f.reason_code == reason_codes::TTS_FAIL_POLICY_RESTRICTED))
        );
        assert_eq!(rt.state(), TtsState::Idle);
    }

    #[test]
    fn at_tts_11_spoken_cursor_is_emitted_and_bounded() {
        let mut rt = Ph1ttsRuntime::new(Ph1ttsConfig::mvp_v1());
        let id = AnswerId(1);
        let text = "Hello. World!";
        rt.handle(MonotonicTimeNs(1), play_req(id, text));

        let out = rt.tick_progress(MonotonicTimeNs(2), 1);
        let cur = out
            .iter()
            .find_map(|o| match o {
                Ph1ttsOutput::Event(Ph1ttsEvent::Progress(p)) => Some(p.spoken_cursor),
                _ => None,
            })
            .expect("progress must include spoken_cursor");
        assert_eq!(cur.segments_total, 2);
        assert!(cur.byte_offset as usize <= text.as_bytes().len());

        let out = rt.tick_progress(MonotonicTimeNs(3), 10_000);
        let stopped = out.iter().find_map(|o| match o {
            Ph1ttsOutput::Event(Ph1ttsEvent::Stopped(s)) => Some(s),
            _ => None,
        });
        let stopped = stopped.expect("should stop");
        assert_eq!(stopped.reason, TtsStopReason::Completed);
        assert_eq!(
            stopped.spoken_cursor.byte_offset as usize,
            text.as_bytes().len()
        );
        assert_eq!(
            stopped.spoken_cursor.segments_spoken,
            stopped.spoken_cursor.segments_total
        );
    }

    #[test]
    fn at_tts_12_segment_plan_is_deterministic_and_cursor_lands_on_boundaries() {
        let text = "Hello.\nWorld! Another sentence?";
        let ends = segment_plan(text);
        assert!(ends.len() >= 1);

        let mut rt1 = Ph1ttsRuntime::new(Ph1ttsConfig::mvp_v1());
        let mut rt2 = Ph1ttsRuntime::new(Ph1ttsConfig::mvp_v1());
        rt1.handle(MonotonicTimeNs(1), play_req(AnswerId(1), text));
        rt2.handle(MonotonicTimeNs(1), play_req(AnswerId(2), text));

        let total_ms = estimate_total_ms(text, Ph1ttsConfig::mvp_v1().max_ms_played);
        let delta_ms = if total_ms > 2 { total_ms / 2 } else { 1 };

        // Same text => same deterministic segment count across replays.
        let cur1 = rt1
            .tick_progress(MonotonicTimeNs(2), delta_ms)
            .iter()
            .find_map(|o| match o {
                Ph1ttsOutput::Event(Ph1ttsEvent::Progress(p)) => Some(p.spoken_cursor),
                _ => None,
            })
            .expect("progress must include spoken_cursor");
        let cur2 = rt2
            .tick_progress(MonotonicTimeNs(2), delta_ms)
            .iter()
            .find_map(|o| match o {
                Ph1ttsOutput::Event(Ph1ttsEvent::Progress(p)) => Some(p.spoken_cursor),
                _ => None,
            })
            .expect("progress must include spoken_cursor");
        assert_eq!(cur1.segments_total, cur2.segments_total);
        assert_eq!(cur1.segments_total as usize, ends.len());

        // Cursor boundaries must land only on segment ends (safe resume boundaries).
        for cur in [cur1, cur2] {
            if cur.byte_offset != 0 {
                assert!(ends.contains(&cur.byte_offset));
            }
        }
    }

    #[test]
    fn at_tts_13_provider_ladder_primary_success() {
        let mut rt = Ph1ttsRuntime::new(Ph1ttsConfig::mvp_v1());
        let out = rt.handle_with_provider_ladder(
            MonotonicTimeNs(1),
            play_req(AnswerId(41), "hello ladder"),
            &[
                attempt(
                    TtsProviderSlot::Primary,
                    provider_ok_response(1, "openai.tts", "gpt-4o-mini-tts", "hello ladder", 80),
                ),
                attempt(
                    TtsProviderSlot::Secondary,
                    provider_ok_response(2, "google.tts", "chirp-3-hd", "hello ladder", 70),
                ),
            ],
        );
        assert!(out
            .iter()
            .any(|o| matches!(o, Ph1ttsOutput::Event(Ph1ttsEvent::Started(_)))));
        assert_eq!(rt.state(), TtsState::Playing);
        assert_eq!(rt.current_text(), Some("hello ladder"));
    }

    #[test]
    fn at_tts_14_provider_ladder_falls_back_to_secondary() {
        let mut rt = Ph1ttsRuntime::new(Ph1ttsConfig::mvp_v1());
        let out = rt.handle_with_provider_ladder(
            MonotonicTimeNs(1),
            play_req(AnswerId(42), "fallback text"),
            &[
                attempt(
                    TtsProviderSlot::Primary,
                    provider_timeout_response(11, "openai.tts", "gpt-4o-mini-tts", 90),
                ),
                attempt(
                    TtsProviderSlot::Secondary,
                    provider_ok_response(12, "google.tts", "chirp-3-hd", "fallback text", 95),
                ),
            ],
        );
        assert!(out
            .iter()
            .any(|o| matches!(o, Ph1ttsOutput::Event(Ph1ttsEvent::Started(_)))));
        assert_eq!(rt.state(), TtsState::Playing);
        assert_eq!(rt.current_text(), Some("fallback text"));
    }

    #[test]
    fn at_tts_15_provider_ladder_terminal_text_only_failsafe_when_both_fail() {
        let mut rt = Ph1ttsRuntime::new(Ph1ttsConfig::mvp_v1());
        let out = rt.handle_with_provider_ladder(
            MonotonicTimeNs(1),
            play_req(AnswerId(43), "no audio"),
            &[
                attempt(
                    TtsProviderSlot::Primary,
                    provider_timeout_response(21, "openai.tts", "gpt-4o-mini-tts", 110),
                ),
                attempt(
                    TtsProviderSlot::Secondary,
                    provider_timeout_response(22, "google.tts", "chirp-3-hd", 120),
                ),
            ],
        );
        assert!(out.iter().any(|o| matches!(
            o,
            Ph1ttsOutput::Event(Ph1ttsEvent::Failed(f))
                if f.reason_code == reason_codes::TTS_FAIL_TEXT_ONLY_FAILSAFE
        )));
        assert_eq!(rt.state(), TtsState::Idle);
    }

    #[test]
    fn at_tts_16_provider_ladder_budget_breach_fails_closed() {
        let mut rt = Ph1ttsRuntime::new(Ph1ttsConfig {
            max_total_latency_budget_ms: 50,
            ..Ph1ttsConfig::mvp_v1()
        });
        let out = rt.handle_with_provider_ladder(
            MonotonicTimeNs(1),
            play_req(AnswerId(44), "budget"),
            &[attempt(
                TtsProviderSlot::Primary,
                provider_timeout_response(31, "openai.tts", "gpt-4o-mini-tts", 90),
            )],
        );
        assert!(out.iter().any(|o| matches!(
            o,
            Ph1ttsOutput::Event(Ph1ttsEvent::Failed(f))
                if f.reason_code == reason_codes::TTS_FAIL_PROVIDER_BUDGET_EXCEEDED
        )));
        assert_eq!(rt.state(), TtsState::Idle);
    }

    #[test]
    fn at_tts_17_provider_ladder_preserves_pause_resume_cancel_semantics() {
        let mut rt = Ph1ttsRuntime::new(Ph1ttsConfig::mvp_v1());
        let answer_id = AnswerId(45);
        let start = rt.handle_with_provider_ladder(
            MonotonicTimeNs(1),
            play_req(answer_id, "duplex flow"),
            &[attempt(
                TtsProviderSlot::Primary,
                provider_ok_response(41, "openai.tts", "gpt-4o-mini-tts", "duplex flow", 70),
            )],
        );
        assert!(start
            .iter()
            .any(|o| matches!(o, Ph1ttsOutput::Event(Ph1ttsEvent::Started(_)))));
        assert_eq!(rt.state(), TtsState::Playing);

        let pause = rt.handle_with_provider_ladder(
            MonotonicTimeNs(2),
            control_req(answer_id, TtsControl::Pause),
            &[],
        );
        assert!(pause.is_empty());
        assert_eq!(rt.state(), TtsState::Paused);

        let resume = rt.handle_with_provider_ladder(
            MonotonicTimeNs(3),
            control_req(answer_id, TtsControl::Resume),
            &[],
        );
        assert!(resume.is_empty());
        assert_eq!(rt.state(), TtsState::Playing);

        let cancel = rt.handle_with_provider_ladder(
            MonotonicTimeNs(4),
            control_req(answer_id, TtsControl::Cancel),
            &[],
        );
        assert!(cancel.iter().any(|o| matches!(
            o,
            Ph1ttsOutput::Event(Ph1ttsEvent::Stopped(s)) if s.reason == TtsStopReason::Cancelled
        )));
        assert_eq!(rt.state(), TtsState::Idle);
    }

    #[test]
    fn at_tts_5h_step11_inhouse_shadow_route_eligible_only_with_governed_gate() {
        let rt = Ph1ttsRuntime::new(Ph1ttsConfig::mvp_v1());
        let req = play_req(AnswerId(46), "play shadow baseline");
        let provider_truth = provider_ok_response(
            51,
            "openai.tts",
            "gpt-4o-mini-tts",
            "play shadow baseline",
            90,
        );
        let inhouse = inhouse_shadow_attempt("play shadow baseline", "en-US", 930, 120);
        let slice_key = Ph1ttsShadowSliceKey {
            locale: "en".to_string(),
            device_route: "desktop_builtin".to_string(),
            tenant_id: "tenant_a".to_string(),
        };

        let hold = rt
            .evaluate_inhouse_shadow_route(
                &req,
                slice_key.clone(),
                &provider_truth,
                &inhouse,
                false,
            )
            .expect("shadow evaluation should succeed");
        assert_eq!(hold.decision, Ph1ttsShadowRouteDecision::HoldShadow);
        assert_eq!(
            hold.block_reason_code,
            Some(reason_codes::TTS_FAIL_SHADOW_PROMOTION_BLOCKED)
        );

        let promoted = rt
            .evaluate_inhouse_shadow_route(&req, slice_key, &provider_truth, &inhouse, true)
            .expect("shadow evaluation should succeed");
        assert_eq!(
            promoted.decision,
            Ph1ttsShadowRouteDecision::EligibleForPromotion
        );
        assert_eq!(promoted.block_reason_code, None);
        assert_eq!(promoted.duration_delta_ms, 30);
        assert_eq!(promoted.latency_delta_ms, 30);
    }

    #[test]
    fn at_tts_5h_step11_inhouse_shadow_route_blocks_meaning_drift() {
        let rt = Ph1ttsRuntime::new(Ph1ttsConfig::mvp_v1());
        let req = play_req(AnswerId(47), "play provider truth");
        let provider_truth = provider_ok_response(
            52,
            "openai.tts",
            "gpt-4o-mini-tts",
            "play provider truth",
            95,
        );
        let inhouse = inhouse_shadow_attempt("different words", "en", 900, 120);
        let slice_key = Ph1ttsShadowSliceKey {
            locale: "en-US".to_string(),
            device_route: "desktop_builtin".to_string(),
            tenant_id: "tenant_a".to_string(),
        };

        let out = rt
            .evaluate_inhouse_shadow_route(&req, slice_key, &provider_truth, &inhouse, true)
            .expect("shadow evaluation should succeed");
        assert_eq!(out.decision, Ph1ttsShadowRouteDecision::HoldShadow);
        assert_eq!(
            out.block_reason_code,
            Some(reason_codes::TTS_FAIL_MEANING_DRIFT)
        );
    }
}
