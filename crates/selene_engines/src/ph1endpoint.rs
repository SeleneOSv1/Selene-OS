#![forbid(unsafe_code)]

use std::cmp::min;
use std::collections::BTreeSet;

use selene_kernel_contracts::ph1endpoint::{
    EndpointBoundaryScoreOk, EndpointBoundaryScoreRequest, EndpointCapabilityId,
    EndpointConfidenceBucket, EndpointHintsBuildOk, EndpointHintsBuildRequest, EndpointRefuse,
    EndpointSegmentHint, EndpointValidationStatus, Ph1EndpointRequest, Ph1EndpointResponse,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.ENDPOINT reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_ENDPOINT_OK_HINTS_BUILD: ReasonCodeId = ReasonCodeId(0x454E_0001);
    pub const PH1_ENDPOINT_OK_BOUNDARY_SCORE: ReasonCodeId = ReasonCodeId(0x454E_0002);

    pub const PH1_ENDPOINT_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x454E_00F1);
    pub const PH1_ENDPOINT_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x454E_00F2);
    pub const PH1_ENDPOINT_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x454E_00F3);
    pub const PH1_ENDPOINT_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x454E_00F4);
    pub const PH1_ENDPOINT_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x454E_00F5);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1EndpointConfig {
    pub max_vad_windows: u8,
    pub max_segment_hints: u8,
    pub max_diagnostics: u8,
    pub min_voiced_duration_ms: u16,
    pub min_finalize_silence_ms: u16,
}

impl Ph1EndpointConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_vad_windows: 16,
            max_segment_hints: 16,
            max_diagnostics: 8,
            min_voiced_duration_ms: 120,
            min_finalize_silence_ms: 220,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1EndpointRuntime {
    config: Ph1EndpointConfig,
}

impl Ph1EndpointRuntime {
    pub fn new(config: Ph1EndpointConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1EndpointRequest) -> Ph1EndpointResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_ENDPOINT_INPUT_SCHEMA_INVALID,
                "endpoint request failed contract validation",
            );
        }

        match req {
            Ph1EndpointRequest::EndpointHintsBuild(r) => self.run_hints_build(r),
            Ph1EndpointRequest::EndpointBoundaryScore(r) => self.run_boundary_score(r),
        }
    }

    fn run_hints_build(&self, req: &EndpointHintsBuildRequest) -> Ph1EndpointResponse {
        if req.vad_windows.is_empty() {
            return self.refuse(
                EndpointCapabilityId::EndpointHintsBuild,
                reason_codes::PH1_ENDPOINT_UPSTREAM_INPUT_MISSING,
                "vad_windows is empty",
            );
        }

        let window_budget = min(req.envelope.max_vad_windows, self.config.max_vad_windows) as usize;
        if req.vad_windows.len() > window_budget {
            return self.refuse(
                EndpointCapabilityId::EndpointHintsBuild,
                reason_codes::PH1_ENDPOINT_BUDGET_EXCEEDED,
                "vad_windows exceeds configured budget",
            );
        }

        let mut hints = Vec::new();
        for (index, window) in req.vad_windows.iter().enumerate() {
            let voiced_duration = window.t_end_ms.saturating_sub(window.t_start_ms) as u16;
            if voiced_duration < self.config.min_voiced_duration_ms {
                continue;
            }

            let silence_score = if window.trailing_silence_ms >= self.config.min_finalize_silence_ms
            {
                100u16
            } else {
                ((window.trailing_silence_ms as u32 * 100)
                    / self.config.min_finalize_silence_ms as u32) as u16
            };
            let score = weighted_score(
                window.vad_confidence_pct as u16,
                window.speech_likeness_pct as u16,
                silence_score,
            );

            let should_finalize_turn = !req.tts_playback_active
                && window.trailing_silence_ms >= self.config.min_finalize_silence_ms;

            let endpoint_reason = if should_finalize_turn {
                "silence_window"
            } else if req.tts_playback_active {
                "barge_in_window"
            } else {
                "voiced_window"
            };

            let confidence_bucket = if score >= 85 {
                EndpointConfidenceBucket::High
            } else if score >= 60 {
                EndpointConfidenceBucket::Medium
            } else {
                EndpointConfidenceBucket::Low
            };

            let segment_id = format!("endpoint_segment_{:02}", index + 1);
            let hint = match EndpointSegmentHint::v1(
                segment_id,
                window.window_id.clone(),
                window.t_start_ms,
                window.t_end_ms,
                endpoint_reason.to_string(),
                score,
                confidence_bucket,
                should_finalize_turn,
            ) {
                Ok(hint) => hint,
                Err(_) => {
                    return self.refuse(
                        EndpointCapabilityId::EndpointHintsBuild,
                        reason_codes::PH1_ENDPOINT_INTERNAL_PIPELINE_ERROR,
                        "failed to build endpoint segment hint",
                    )
                }
            };
            hints.push(hint);
        }

        if hints.is_empty() {
            return self.refuse(
                EndpointCapabilityId::EndpointHintsBuild,
                reason_codes::PH1_ENDPOINT_UPSTREAM_INPUT_MISSING,
                "no valid endpoint hints after filtering",
            );
        }

        let max_hints = min(self.config.max_segment_hints as usize, hints.len());
        hints.sort_by(|a, b| {
            b.endpoint_score_pct
                .cmp(&a.endpoint_score_pct)
                .then(a.t_start_ms.cmp(&b.t_start_ms))
                .then(a.segment_id.cmp(&b.segment_id))
        });
        hints.truncate(max_hints);

        let selected_segment_id = hints[0].segment_id.clone();
        match EndpointHintsBuildOk::v1(
            reason_codes::PH1_ENDPOINT_OK_HINTS_BUILD,
            selected_segment_id,
            hints,
            true,
            true,
        ) {
            Ok(ok) => Ph1EndpointResponse::EndpointHintsBuildOk(ok),
            Err(_) => self.refuse(
                EndpointCapabilityId::EndpointHintsBuild,
                reason_codes::PH1_ENDPOINT_INTERNAL_PIPELINE_ERROR,
                "failed to construct endpoint hints build output",
            ),
        }
    }

    fn run_boundary_score(&self, req: &EndpointBoundaryScoreRequest) -> Ph1EndpointResponse {
        if req.ordered_segment_hints.is_empty() {
            return self.refuse(
                EndpointCapabilityId::EndpointBoundaryScore,
                reason_codes::PH1_ENDPOINT_UPSTREAM_INPUT_MISSING,
                "ordered_segment_hints is empty",
            );
        }

        let hint_budget = min(req.envelope.max_vad_windows, self.config.max_segment_hints) as usize;
        if req.ordered_segment_hints.len() > hint_budget {
            return self.refuse(
                EndpointCapabilityId::EndpointBoundaryScore,
                reason_codes::PH1_ENDPOINT_BUDGET_EXCEEDED,
                "ordered_segment_hints exceeds configured budget",
            );
        }

        let mut diagnostics: Vec<String> = Vec::new();
        if req.ordered_segment_hints[0].segment_id != req.selected_segment_id {
            diagnostics.push("selected_not_first_in_ordered_hints".to_string());
        }
        if !req
            .ordered_segment_hints
            .iter()
            .any(|hint| hint.segment_id == req.selected_segment_id)
        {
            diagnostics.push("selected_not_found_in_ordered_hints".to_string());
        }
        if req
            .ordered_segment_hints
            .windows(2)
            .any(|pair| pair[0].endpoint_score_pct < pair[1].endpoint_score_pct)
        {
            diagnostics.push("ordered_hints_not_sorted_by_score_desc".to_string());
        }

        let mut seen_ids: BTreeSet<&str> = BTreeSet::new();
        if req
            .ordered_segment_hints
            .iter()
            .any(|hint| !seen_ids.insert(hint.segment_id.as_str()))
        {
            diagnostics.push("duplicate_segment_id".to_string());
        }

        if let Some(previous) = &req.previous_selected_segment_id {
            if previous == &req.selected_segment_id && req.ordered_segment_hints.len() > 1 {
                diagnostics
                    .push("selected_repeats_previous_with_alternative_available".to_string());
            }
        }

        diagnostics.truncate(self.config.max_diagnostics as usize);
        let (status, reason_code) = if diagnostics.is_empty() {
            (
                EndpointValidationStatus::Ok,
                reason_codes::PH1_ENDPOINT_OK_BOUNDARY_SCORE,
            )
        } else {
            (
                EndpointValidationStatus::Fail,
                reason_codes::PH1_ENDPOINT_VALIDATION_FAILED,
            )
        };

        match EndpointBoundaryScoreOk::v1(reason_code, status, diagnostics, true, true) {
            Ok(ok) => Ph1EndpointResponse::EndpointBoundaryScoreOk(ok),
            Err(_) => self.refuse(
                EndpointCapabilityId::EndpointBoundaryScore,
                reason_codes::PH1_ENDPOINT_INTERNAL_PIPELINE_ERROR,
                "failed to construct endpoint boundary-score output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: EndpointCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1EndpointResponse {
        let out = EndpointRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("EndpointRefuse::v1 must construct for static messages");
        Ph1EndpointResponse::Refuse(out)
    }
}

fn capability_from_request(req: &Ph1EndpointRequest) -> EndpointCapabilityId {
    match req {
        Ph1EndpointRequest::EndpointHintsBuild(_) => EndpointCapabilityId::EndpointHintsBuild,
        Ph1EndpointRequest::EndpointBoundaryScore(_) => EndpointCapabilityId::EndpointBoundaryScore,
    }
}

fn weighted_score(vad_confidence_pct: u16, speech_likeness_pct: u16, silence_score_pct: u16) -> u8 {
    let score = (vad_confidence_pct * 45 + speech_likeness_pct * 35 + silence_score_pct * 20) / 100;
    min(score, 100) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1endpoint::{EndpointRequestEnvelope, EndpointVadWindow};

    fn runtime() -> Ph1EndpointRuntime {
        Ph1EndpointRuntime::new(Ph1EndpointConfig::mvp_v1())
    }

    fn envelope(max_vad_windows: u8) -> EndpointRequestEnvelope {
        EndpointRequestEnvelope::v1(
            selene_kernel_contracts::ph1j::CorrelationId(1601),
            selene_kernel_contracts::ph1j::TurnId(121),
            max_vad_windows,
        )
        .unwrap()
    }

    fn windows() -> Vec<EndpointVadWindow> {
        vec![
            EndpointVadWindow::v1("window_1".to_string(), 0, 480, 90, 88, 300).unwrap(),
            EndpointVadWindow::v1("window_2".to_string(), 520, 900, 84, 79, 120).unwrap(),
        ]
    }

    #[test]
    fn at_endpoint_01_hints_build_output_is_schema_valid() {
        let req = Ph1EndpointRequest::EndpointHintsBuild(
            EndpointHintsBuildRequest::v1(envelope(8), windows(), 24, false).unwrap(),
        );

        let out = runtime().run(&req);
        assert!(out.validate().is_ok());
        match out {
            Ph1EndpointResponse::EndpointHintsBuildOk(ok) => {
                assert!(!ok.ordered_segment_hints.is_empty());
                assert_eq!(
                    ok.selected_segment_id,
                    ok.ordered_segment_hints[0].segment_id
                );
            }
            _ => panic!("expected EndpointHintsBuildOk"),
        }
    }

    #[test]
    fn at_endpoint_02_hints_order_is_deterministic() {
        let req = Ph1EndpointRequest::EndpointHintsBuild(
            EndpointHintsBuildRequest::v1(envelope(8), windows(), 24, false).unwrap(),
        );

        let out1 = runtime().run(&req);
        let out2 = runtime().run(&req);
        let hints1 = match out1 {
            Ph1EndpointResponse::EndpointHintsBuildOk(ok) => ok.ordered_segment_hints,
            _ => panic!("expected EndpointHintsBuildOk"),
        };
        let hints2 = match out2 {
            Ph1EndpointResponse::EndpointHintsBuildOk(ok) => ok.ordered_segment_hints,
            _ => panic!("expected EndpointHintsBuildOk"),
        };
        assert_eq!(hints1, hints2);
    }

    #[test]
    fn at_endpoint_03_budget_bound_is_enforced() {
        let req = Ph1EndpointRequest::EndpointHintsBuild(
            EndpointHintsBuildRequest::v1(envelope(1), windows(), 24, false).unwrap(),
        );

        let out = runtime().run(&req);
        match out {
            Ph1EndpointResponse::Refuse(refuse) => {
                assert_eq!(
                    refuse.reason_code,
                    reason_codes::PH1_ENDPOINT_BUDGET_EXCEEDED
                );
            }
            _ => panic!("expected Refuse"),
        }
    }

    #[test]
    fn at_endpoint_04_boundary_score_fails_on_selection_drift() {
        let build = runtime().run(&Ph1EndpointRequest::EndpointHintsBuild(
            EndpointHintsBuildRequest::v1(envelope(8), windows(), 24, false).unwrap(),
        ));
        let build_ok = match build {
            Ph1EndpointResponse::EndpointHintsBuildOk(ok) => ok,
            _ => panic!("expected EndpointHintsBuildOk"),
        };

        let req = Ph1EndpointRequest::EndpointBoundaryScore(
            EndpointBoundaryScoreRequest::v1(
                envelope(8),
                build_ok.ordered_segment_hints[1].segment_id.clone(),
                build_ok.ordered_segment_hints.clone(),
                None,
            )
            .unwrap(),
        );
        let out = runtime().run(&req);
        match out {
            Ph1EndpointResponse::EndpointBoundaryScoreOk(ok) => {
                assert_eq!(ok.validation_status, EndpointValidationStatus::Fail);
            }
            _ => panic!("expected EndpointBoundaryScoreOk"),
        }
    }
}
