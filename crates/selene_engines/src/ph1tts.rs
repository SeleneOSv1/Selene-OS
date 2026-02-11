#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1k::TtsPlaybackActiveEvent;
use selene_kernel_contracts::ph1tts::{
    AnswerId, Ph1ttsEvent, Ph1ttsRequest, SpokenCursor, TtsControl, TtsFailed, TtsProgress,
    TtsStarted, TtsStopReason, TtsStopped, VoiceId,
};
use selene_kernel_contracts::{MonotonicTimeNs, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.TTS reason-code namespace. Values are placeholders until the global registry is formalized.
    pub const TTS_FAIL_INVALID_REQUEST: ReasonCodeId = ReasonCodeId(0x5454_0001);
    pub const TTS_FAIL_PROVIDER: ReasonCodeId = ReasonCodeId(0x5454_0002);
    pub const TTS_FAIL_POLICY_RESTRICTED: ReasonCodeId = ReasonCodeId(0x5454_0003);
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
pub struct Ph1ttsConfig {
    pub max_ms_played: u32,
}

impl Ph1ttsConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_ms_played: 60_000,
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
        if req.validate().is_err() {
            return vec![Ph1ttsOutput::Event(Ph1ttsEvent::Failed(TtsFailed {
                schema_version: selene_kernel_contracts::SchemaVersion(1),
                answer_id: req.answer_id,
                reason_code: reason_codes::TTS_FAIL_INVALID_REQUEST,
            }))];
        }

        // Defense-in-depth: never speak when policy forbids audible output.
        if matches!(req.tts_control, TtsControl::Play)
            && (req.policy_context_ref.do_not_disturb || req.policy_context_ref.privacy_mode)
        {
            return vec![Ph1ttsOutput::Event(Ph1ttsEvent::Failed(TtsFailed {
                schema_version: selene_kernel_contracts::SchemaVersion(1),
                answer_id: req.answer_id,
                reason_code: reason_codes::TTS_FAIL_POLICY_RESTRICTED,
            }))];
        }

        match req.tts_control {
            TtsControl::Play => self.on_play(now, req.answer_id, req.response_text),
            TtsControl::Cancel => self.on_cancel(now, req.answer_id),
            TtsControl::Pause => self.on_pause(req.answer_id),
            TtsControl::Resume => self.on_resume(req.answer_id),
        }
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
    use selene_kernel_contracts::ph1c::{LanguageTag, SessionStateRef};
    use selene_kernel_contracts::ph1d::{PolicyContextRef, SafetyTier};
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
        let mut rt = Ph1ttsRuntime::new(Ph1ttsConfig { max_ms_played: 10 });
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
}
