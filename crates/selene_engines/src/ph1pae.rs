#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1pae::{
    PaeAdaptationHint, PaeAdaptationHintEmitOk, PaeAdaptationHintEmitRequest, PaeCapabilityId,
    PaeMode, PaePolicyScoreBuildOk, PaePolicyScoreBuildRequest, PaeProviderSlot, PaeRefuse,
    PaeScoreEntry, PaeSignalSource, PaeValidationStatus, Ph1PaeRequest, Ph1PaeResponse,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

use crate::ph1comp::{
    compare_pae_rank, compute_pae_candidate_score, compute_pae_signal_bias, pae_priority_from_total,
    pae_route_index,
};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.PAE reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_PAE_OK_POLICY_SCORE_BUILD: ReasonCodeId = ReasonCodeId(0x5041_0001);
    pub const PH1_PAE_OK_ADAPTATION_HINT_EMIT: ReasonCodeId = ReasonCodeId(0x5041_0002);

    pub const PH1_PAE_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x5041_00F1);
    pub const PH1_PAE_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x5041_00F2);
    pub const PH1_PAE_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x5041_00F3);
    pub const PH1_PAE_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x5041_00F4);
    pub const PH1_PAE_GOVERNED_ARTIFACT_REQUIRED: ReasonCodeId = ReasonCodeId(0x5041_00F5);
    pub const PH1_PAE_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5041_00F6);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1PaeConfig {
    pub max_signals: u8,
    pub max_candidates: u8,
    pub max_scores: u8,
    pub max_hints: u8,
    pub max_diagnostics: u8,
    pub default_min_sample_size: u16,
    pub default_promotion_threshold_bp: i16,
    pub lead_regression_guard_bp: u16,
}

impl Ph1PaeConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_signals: 24,
            max_candidates: 8,
            max_scores: 8,
            max_hints: 8,
            max_diagnostics: 8,
            default_min_sample_size: 120,
            default_promotion_threshold_bp: 1200,
            lead_regression_guard_bp: 1800,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1PaeRuntime {
    config: Ph1PaeConfig,
}

impl Ph1PaeRuntime {
    pub fn new(config: Ph1PaeConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1PaeRequest) -> Ph1PaeResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_PAE_INPUT_SCHEMA_INVALID,
                "pae request failed contract validation",
            );
        }

        match req {
            Ph1PaeRequest::PaePolicyScoreBuild(r) => self.run_policy_score_build(r),
            Ph1PaeRequest::PaeAdaptationHintEmit(r) => self.run_adaptation_hint_emit(r),
        }
    }

    fn run_policy_score_build(&self, req: &PaePolicyScoreBuildRequest) -> Ph1PaeResponse {
        if req.signals.is_empty() || req.candidates.is_empty() {
            return self.refuse(
                PaeCapabilityId::PaePolicyScoreBuild,
                reason_codes::PH1_PAE_UPSTREAM_INPUT_MISSING,
                "signals or candidates is empty",
            );
        }

        let signal_budget = min(req.envelope.max_signals, self.config.max_signals) as usize;
        if req.signals.len() > signal_budget {
            return self.refuse(
                PaeCapabilityId::PaePolicyScoreBuild,
                reason_codes::PH1_PAE_BUDGET_EXCEEDED,
                "signals exceeds configured budget",
            );
        }

        let candidate_budget =
            min(req.envelope.max_candidates, self.config.max_candidates) as usize;
        if req.candidates.len() > candidate_budget {
            return self.refuse(
                PaeCapabilityId::PaePolicyScoreBuild,
                reason_codes::PH1_PAE_BUDGET_EXCEEDED,
                "candidates exceeds configured budget",
            );
        }

        if req.signals.iter().any(|signal| {
            signal.source == PaeSignalSource::RllGoverned && !signal.governed_artifact_active
        }) {
            return self.refuse(
                PaeCapabilityId::PaePolicyScoreBuild,
                reason_codes::PH1_PAE_GOVERNED_ARTIFACT_REQUIRED,
                "rll-governed signal must be active",
            );
        }

        let mut route_signal_bp = [0i32; 4];
        for signal in &req.signals {
            let weighted =
                compute_pae_signal_bias(signal.signal_value_bp, signal.confidence_bp, signal.source);
            let idx = pae_route_index(signal.route_domain);
            route_signal_bp[idx] = (route_signal_bp[idx] + weighted).clamp(-4_000, 4_000);
        }

        let effective_min_sample = req
            .minimum_sample_size
            .max(self.config.default_min_sample_size);
        let effective_promotion_threshold = req
            .promotion_threshold_bp
            .max(self.config.default_promotion_threshold_bp);

        let mut scores = Vec::new();
        for candidate in &req.candidates {
            if req.require_governed_artifacts
                && candidate.proposed_mode != PaeMode::Shadow
                && candidate.governed_artifact_ref.is_none()
            {
                return self.refuse(
                    PaeCapabilityId::PaePolicyScoreBuild,
                    reason_codes::PH1_PAE_GOVERNED_ARTIFACT_REQUIRED,
                    "non-shadow candidate requires governed artifact",
                );
            }

            let signal_bias = route_signal_bp[pae_route_index(candidate.route_domain)];
            let quantitative = compute_pae_candidate_score(
                candidate.expected_quality_bp,
                signal_bias,
                candidate.expected_latency_ms,
                candidate.expected_cost_bp,
                candidate.regression_risk_bp,
                candidate.sample_size,
                effective_min_sample,
            );

            let score = match PaeScoreEntry::v1(
                candidate.candidate_id.clone(),
                candidate.route_domain,
                candidate.provider_slot,
                candidate.proposed_mode,
                quantitative.total_score_bp,
                quantitative.quality_score_bp,
                quantitative.latency_penalty_bp,
                quantitative.cost_penalty_bp,
                quantitative.regression_penalty_bp,
                candidate.sample_size,
            ) {
                Ok(score) => score,
                Err(_) => {
                    return self.refuse(
                        PaeCapabilityId::PaePolicyScoreBuild,
                        reason_codes::PH1_PAE_INTERNAL_PIPELINE_ERROR,
                        "failed to build pae score entry",
                    );
                }
            };
            scores.push(score);
        }

        if scores.is_empty() {
            return self.refuse(
                PaeCapabilityId::PaePolicyScoreBuild,
                reason_codes::PH1_PAE_UPSTREAM_INPUT_MISSING,
                "no score entries produced",
            );
        }

        scores.sort_by(|a, b| {
            compare_pae_rank(
                a.total_score_bp,
                a.sample_size,
                &a.candidate_id,
                b.total_score_bp,
                b.sample_size,
                &b.candidate_id,
            )
        });
        scores.truncate(min(scores.len(), req.envelope.max_scores as usize));

        let selected = &scores[0];
        let selected_candidate = req
            .candidates
            .iter()
            .find(|candidate| candidate.candidate_id == selected.candidate_id);
        let Some(selected_candidate) = selected_candidate else {
            return self.refuse(
                PaeCapabilityId::PaePolicyScoreBuild,
                reason_codes::PH1_PAE_INTERNAL_PIPELINE_ERROR,
                "selected candidate missing from input set",
            );
        };

        let promotion_eligible = selected.sample_size >= effective_min_sample
            && selected.total_score_bp >= effective_promotion_threshold as i32
            && selected_candidate.regression_risk_bp <= self.config.lead_regression_guard_bp;

        let mut selected_mode = req.current_mode;
        if promotion_eligible {
            selected_mode = bounded_promotion_mode(req.current_mode, selected.mode_applied);
        }

        if req.consecutive_threshold_failures >= req.demotion_failure_threshold {
            selected_mode = demoted_mode(selected_mode);
        }

        let rollback_ready = selected_candidate.rollback_to.is_some();
        if selected_mode == PaeMode::Lead && !rollback_ready {
            selected_mode = PaeMode::Assist;
        }

        match PaePolicyScoreBuildOk::v1(
            reason_codes::PH1_PAE_OK_POLICY_SCORE_BUILD,
            selected.candidate_id.clone(),
            scores,
            selected_mode,
            promotion_eligible,
            rollback_ready,
            true,
            true,
        ) {
            Ok(ok) => Ph1PaeResponse::PaePolicyScoreBuildOk(ok),
            Err(_) => self.refuse(
                PaeCapabilityId::PaePolicyScoreBuild,
                reason_codes::PH1_PAE_INTERNAL_PIPELINE_ERROR,
                "failed to construct policy-score-build output",
            ),
        }
    }

    fn run_adaptation_hint_emit(&self, req: &PaeAdaptationHintEmitRequest) -> Ph1PaeResponse {
        if req.ordered_scores.is_empty() {
            return self.refuse(
                PaeCapabilityId::PaeAdaptationHintEmit,
                reason_codes::PH1_PAE_UPSTREAM_INPUT_MISSING,
                "ordered_scores is empty",
            );
        }

        let score_budget = min(req.envelope.max_scores, self.config.max_scores) as usize;
        if req.ordered_scores.len() > score_budget {
            return self.refuse(
                PaeCapabilityId::PaeAdaptationHintEmit,
                reason_codes::PH1_PAE_BUDGET_EXCEEDED,
                "ordered_scores exceeds configured budget",
            );
        }

        let selected_score = req
            .ordered_scores
            .iter()
            .find(|score| score.candidate_id == req.selected_candidate_id);
        let Some(selected_score) = selected_score else {
            return self.refuse(
                PaeCapabilityId::PaeAdaptationHintEmit,
                reason_codes::PH1_PAE_VALIDATION_FAILED,
                "selected candidate missing from ordered_scores",
            );
        };

        let mut diagnostics = Vec::new();
        if req.ordered_scores[0].candidate_id != req.selected_candidate_id {
            diagnostics.push("selected_not_first".to_string());
        }

        if !is_sorted_scores(&req.ordered_scores) {
            diagnostics.push("score_order_not_canonical".to_string());
        }

        if req.selected_mode == PaeMode::Lead && selected_score.mode_applied == PaeMode::Shadow {
            diagnostics.push("lead_mode_not_supported_by_selected_candidate".to_string());
        }

        let hint_budget = min(req.envelope.max_hints, self.config.max_hints) as usize;
        let mut adaptation_hints = Vec::new();
        for (idx, target) in req.allowed_targets.iter().take(hint_budget).enumerate() {
            let hint_key = hint_key_for_target(*target);
            let hint_value = format!(
                "mode={};slot={};score_bp={}",
                mode_token(req.selected_mode),
                provider_slot_token(selected_score.provider_slot),
                selected_score.total_score_bp
            );
            let priority = pae_priority_from_total(selected_score.total_score_bp);

            let hint = match PaeAdaptationHint::v1(
                format!("pae_hint_{:02}", idx + 1),
                *target,
                selected_score.route_domain,
                hint_key.to_string(),
                hint_value,
                priority,
                format!("pae:selected:{}", req.selected_candidate_id),
            ) {
                Ok(hint) => hint,
                Err(_) => {
                    return self.refuse(
                        PaeCapabilityId::PaeAdaptationHintEmit,
                        reason_codes::PH1_PAE_INTERNAL_PIPELINE_ERROR,
                        "failed to construct adaptation hint",
                    );
                }
            };
            adaptation_hints.push(hint);
        }

        if adaptation_hints.is_empty() {
            return self.refuse(
                PaeCapabilityId::PaeAdaptationHintEmit,
                reason_codes::PH1_PAE_UPSTREAM_INPUT_MISSING,
                "no adaptation hints generated",
            );
        }

        diagnostics.truncate(min(
            req.envelope.max_diagnostics as usize,
            self.config.max_diagnostics as usize,
        ));

        let no_runtime_authority_drift = diagnostics.is_empty();
        let (validation_status, reason_code) = if diagnostics.is_empty() {
            (
                PaeValidationStatus::Ok,
                reason_codes::PH1_PAE_OK_ADAPTATION_HINT_EMIT,
            )
        } else {
            (
                PaeValidationStatus::Fail,
                reason_codes::PH1_PAE_VALIDATION_FAILED,
            )
        };

        if req.require_no_runtime_authority_drift && !no_runtime_authority_drift {
            return match PaeAdaptationHintEmitOk::v1(
                reason_code,
                validation_status,
                diagnostics,
                req.allowed_targets.clone(),
                adaptation_hints,
                false,
                true,
                true,
            ) {
                Ok(ok) => Ph1PaeResponse::PaeAdaptationHintEmitOk(ok),
                Err(_) => self.refuse(
                    PaeCapabilityId::PaeAdaptationHintEmit,
                    reason_codes::PH1_PAE_INTERNAL_PIPELINE_ERROR,
                    "failed to construct adaptation-hint output",
                ),
            };
        }

        match PaeAdaptationHintEmitOk::v1(
            reason_code,
            validation_status,
            diagnostics,
            req.allowed_targets.clone(),
            adaptation_hints,
            no_runtime_authority_drift,
            true,
            true,
        ) {
            Ok(ok) => Ph1PaeResponse::PaeAdaptationHintEmitOk(ok),
            Err(_) => self.refuse(
                PaeCapabilityId::PaeAdaptationHintEmit,
                reason_codes::PH1_PAE_INTERNAL_PIPELINE_ERROR,
                "failed to construct adaptation-hint output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: PaeCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1PaeResponse {
        let refuse = PaeRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("PaeRefuse::v1 must construct for static message");
        Ph1PaeResponse::Refuse(refuse)
    }
}

fn capability_from_request(req: &Ph1PaeRequest) -> PaeCapabilityId {
    match req {
        Ph1PaeRequest::PaePolicyScoreBuild(_) => PaeCapabilityId::PaePolicyScoreBuild,
        Ph1PaeRequest::PaeAdaptationHintEmit(_) => PaeCapabilityId::PaeAdaptationHintEmit,
    }
}

fn bounded_promotion_mode(current: PaeMode, requested: PaeMode) -> PaeMode {
    if mode_rank(requested) <= mode_rank(current) {
        return current;
    }

    match current {
        PaeMode::Shadow => {
            if requested == PaeMode::Lead {
                PaeMode::Assist
            } else {
                requested
            }
        }
        PaeMode::Assist => PaeMode::Lead,
        PaeMode::Lead => PaeMode::Lead,
    }
}

fn demoted_mode(mode: PaeMode) -> PaeMode {
    match mode {
        PaeMode::Lead => PaeMode::Assist,
        PaeMode::Assist => PaeMode::Shadow,
        PaeMode::Shadow => PaeMode::Shadow,
    }
}

fn mode_rank(mode: PaeMode) -> u8 {
    match mode {
        PaeMode::Shadow => 0,
        PaeMode::Assist => 1,
        PaeMode::Lead => 2,
    }
}

fn is_sorted_scores(scores: &[PaeScoreEntry]) -> bool {
    scores.windows(2).all(|pair| {
        let a = &pair[0];
        let b = &pair[1];
        compare_pae_rank(
            a.total_score_bp,
            a.sample_size,
            &a.candidate_id,
            b.total_score_bp,
            b.sample_size,
            &b.candidate_id,
        ) != std::cmp::Ordering::Greater
    })
}

fn hint_key_for_target(target: selene_kernel_contracts::ph1pae::PaeTargetEngine) -> &'static str {
    match target {
        selene_kernel_contracts::ph1pae::PaeTargetEngine::Ph1C => "stt_route_plan",
        selene_kernel_contracts::ph1pae::PaeTargetEngine::Ph1Tts => "tts_route_plan",
        selene_kernel_contracts::ph1pae::PaeTargetEngine::Ph1Cache => "cache_route_bias",
        selene_kernel_contracts::ph1pae::PaeTargetEngine::Ph1Multi => "multi_route_bias",
    }
}

fn mode_token(mode: PaeMode) -> &'static str {
    match mode {
        PaeMode::Shadow => "SHADOW",
        PaeMode::Assist => "ASSIST",
        PaeMode::Lead => "LEAD",
    }
}

fn provider_slot_token(slot: PaeProviderSlot) -> &'static str {
    match slot {
        PaeProviderSlot::Primary => "PRIMARY",
        PaeProviderSlot::Secondary => "SECONDARY",
        PaeProviderSlot::Tertiary => "TERTIARY",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1pae::{
        PaePolicyCandidate, PaePolicyScoreBuildRequest, PaeRequestEnvelope, PaeRouteDomain,
        PaeSignalVector, PaeTargetEngine,
    };

    fn envelope() -> PaeRequestEnvelope {
        PaeRequestEnvelope::v1(CorrelationId(9001), TurnId(7001), 8, 4, 4, 4, 8).unwrap()
    }

    fn signal(source: PaeSignalSource, value: i16, confidence_bp: u16) -> PaeSignalVector {
        PaeSignalVector::v1(
            format!("sig:{:?}:{value}", source),
            source,
            PaeRouteDomain::Stt,
            "quality_trend".to_string(),
            value,
            confidence_bp,
            true,
            "evidence:signal".to_string(),
        )
        .unwrap()
    }

    fn candidate(
        id: &str,
        mode: PaeMode,
        quality: i16,
        regression: u16,
        sample: u16,
        rollback: bool,
    ) -> PaePolicyCandidate {
        PaePolicyCandidate::v1(
            id.to_string(),
            PaeRouteDomain::Stt,
            PaeProviderSlot::Primary,
            mode,
            quality,
            180,
            260,
            regression,
            sample,
            if mode == PaeMode::Shadow {
                None
            } else {
                Some(format!("artifact:{id}"))
            },
            if rollback {
                Some(format!("artifact:{id}:rollback"))
            } else {
                None
            },
        )
        .unwrap()
    }

    fn build_req(
        current_mode: PaeMode,
        candidates: Vec<PaePolicyCandidate>,
        minimum_sample_size: u16,
        threshold_bp: i16,
        consecutive_failures: u8,
    ) -> PaePolicyScoreBuildRequest {
        PaePolicyScoreBuildRequest::v1(
            envelope(),
            "tenant_demo".to_string(),
            "desktop_profile_v1".to_string(),
            current_mode,
            vec![signal(PaeSignalSource::Feedback, 240, 9000)],
            candidates,
            true,
            minimum_sample_size,
            threshold_bp,
            3,
            consecutive_failures,
        )
        .unwrap()
    }

    #[test]
    fn at_pae_01_deterministic_candidate_order_for_same_input() {
        let runtime = Ph1PaeRuntime::new(Ph1PaeConfig::mvp_v1());
        let req = Ph1PaeRequest::PaePolicyScoreBuild(build_req(
            PaeMode::Assist,
            vec![
                candidate("c1", PaeMode::Assist, 2600, 240, 180, false),
                candidate("c2", PaeMode::Assist, 2500, 220, 180, false),
            ],
            120,
            1200,
            0,
        ));

        let first = runtime.run(&req);
        let second = runtime.run(&req);

        let Ph1PaeResponse::PaePolicyScoreBuildOk(first_ok) = first else {
            panic!("expected score build ok");
        };
        let Ph1PaeResponse::PaePolicyScoreBuildOk(second_ok) = second else {
            panic!("expected score build ok");
        };

        assert_eq!(
            first_ok.selected_candidate_id,
            second_ok.selected_candidate_id
        );
        assert_eq!(first_ok.ordered_scores, second_ok.ordered_scores);
    }

    #[test]
    fn at_pae_02_promotion_requires_sample_size_and_threshold() {
        let runtime = Ph1PaeRuntime::new(Ph1PaeConfig::mvp_v1());
        let req = Ph1PaeRequest::PaePolicyScoreBuild(build_req(
            PaeMode::Assist,
            vec![candidate("c1", PaeMode::Lead, 3000, 200, 40, true)],
            120,
            1500,
            0,
        ));

        let response = runtime.run(&req);
        let Ph1PaeResponse::PaePolicyScoreBuildOk(ok) = response else {
            panic!("expected score build ok");
        };

        assert_eq!(ok.selected_mode, PaeMode::Assist);
        assert!(!ok.promotion_eligible);
    }

    #[test]
    fn at_pae_03_lead_demotion_on_regression_failures() {
        let runtime = Ph1PaeRuntime::new(Ph1PaeConfig::mvp_v1());
        let req = Ph1PaeRequest::PaePolicyScoreBuild(build_req(
            PaeMode::Lead,
            vec![candidate("c1", PaeMode::Lead, 2900, 300, 220, true)],
            120,
            1400,
            3,
        ));

        let response = runtime.run(&req);
        let Ph1PaeResponse::PaePolicyScoreBuildOk(ok) = response else {
            panic!("expected score build ok");
        };

        assert_eq!(ok.selected_mode, PaeMode::Assist);
    }

    #[test]
    fn at_pae_04_hint_emit_fails_when_order_drift_detected() {
        let runtime = Ph1PaeRuntime::new(Ph1PaeConfig::mvp_v1());

        let score_a = PaeScoreEntry::v1(
            "c1".to_string(),
            PaeRouteDomain::Stt,
            PaeProviderSlot::Primary,
            PaeMode::Assist,
            1700,
            2400,
            180,
            220,
            120,
            180,
        )
        .unwrap();
        let score_b = PaeScoreEntry::v1(
            "c2".to_string(),
            PaeRouteDomain::Stt,
            PaeProviderSlot::Secondary,
            PaeMode::Assist,
            1900,
            2500,
            160,
            200,
            110,
            180,
        )
        .unwrap();

        let req = Ph1PaeRequest::PaeAdaptationHintEmit(
            PaeAdaptationHintEmitRequest::v1(
                envelope(),
                "tenant_demo".to_string(),
                "desktop_profile_v1".to_string(),
                "c2".to_string(),
                PaeMode::Assist,
                vec![score_a, score_b],
                vec![PaeTargetEngine::Ph1C, PaeTargetEngine::Ph1Cache],
                true,
            )
            .unwrap(),
        );

        let response = runtime.run(&req);
        let Ph1PaeResponse::PaeAdaptationHintEmitOk(ok) = response else {
            panic!("expected adaptation-hint emit output");
        };
        assert_eq!(ok.validation_status, PaeValidationStatus::Fail);
    }
}
