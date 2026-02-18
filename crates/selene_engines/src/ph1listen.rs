#![forbid(unsafe_code)]

use std::cmp::min;
use std::collections::BTreeSet;

use selene_kernel_contracts::ph1listen::{
    ListenAdjustmentHint, ListenCapabilityId, ListenCaptureProfile, ListenDeliveryPolicyHint,
    ListenEndpointProfile, ListenEnvironmentMode, ListenRefuse, ListenSignalCollectOk,
    ListenSignalCollectRequest, ListenSignalFilterOk, ListenSignalFilterRequest,
    ListenValidationStatus, Ph1ListenRequest, Ph1ListenResponse,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.LISTEN reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_LISTEN_OK_SIGNAL_COLLECT: ReasonCodeId = ReasonCodeId(0x4C49_0001);
    pub const PH1_LISTEN_OK_SIGNAL_FILTER: ReasonCodeId = ReasonCodeId(0x4C49_0002);

    pub const PH1_LISTEN_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x4C49_00F1);
    pub const PH1_LISTEN_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x4C49_00F2);
    pub const PH1_LISTEN_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x4C49_00F3);
    pub const PH1_LISTEN_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4C49_00F4);
    pub const PH1_LISTEN_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4C49_00F5);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1ListenConfig {
    pub max_signal_windows: u8,
    pub max_adjustments: u8,
    pub max_diagnostics: u8,
}

impl Ph1ListenConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_signal_windows: 24,
            max_adjustments: 8,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1ListenRuntime {
    config: Ph1ListenConfig,
}

impl Ph1ListenRuntime {
    pub fn new(config: Ph1ListenConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1ListenRequest) -> Ph1ListenResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_LISTEN_INPUT_SCHEMA_INVALID,
                "listen request failed contract validation",
            );
        }

        match req {
            Ph1ListenRequest::ListenSignalCollect(r) => self.run_signal_collect(r),
            Ph1ListenRequest::ListenSignalFilter(r) => self.run_signal_filter(r),
        }
    }

    fn run_signal_collect(&self, req: &ListenSignalCollectRequest) -> Ph1ListenResponse {
        if req.signal_windows.is_empty() {
            return self.refuse(
                ListenCapabilityId::ListenSignalCollect,
                reason_codes::PH1_LISTEN_UPSTREAM_INPUT_MISSING,
                "signal_windows is empty",
            );
        }

        let signal_budget = min(
            req.envelope.max_signal_windows as usize,
            self.config.max_signal_windows as usize,
        );
        if req.signal_windows.len() > signal_budget {
            return self.refuse(
                ListenCapabilityId::ListenSignalCollect,
                reason_codes::PH1_LISTEN_BUDGET_EXCEEDED,
                "signal_windows exceeds configured budget",
            );
        }

        if req
            .signal_windows
            .iter()
            .all(|window| window.source_engine != "PH1.K")
        {
            return self.refuse(
                ListenCapabilityId::ListenSignalCollect,
                reason_codes::PH1_LISTEN_VALIDATION_FAILED,
                "at least one PH1.K signal window is required",
            );
        }

        let adjustment_budget = min(
            req.envelope.max_adjustments as usize,
            self.config.max_adjustments as usize,
        );
        if adjustment_budget == 0 {
            return self.refuse(
                ListenCapabilityId::ListenSignalCollect,
                reason_codes::PH1_LISTEN_BUDGET_EXCEEDED,
                "adjustment budget exceeded",
            );
        }

        let mut adjustments = Vec::new();
        for window in &req.signal_windows {
            let environment_mode = classify_environment(window, req);
            let capture_profile = capture_profile_for(environment_mode);
            let endpoint_profile = endpoint_profile_for(environment_mode);
            let delivery_policy_hint = delivery_policy_for(environment_mode, req);
            let priority_bp = priority_score(window, environment_mode, req);

            let adjustment = match ListenAdjustmentHint::v1(
                format!("adj_{}", window.window_id),
                environment_mode,
                capture_profile,
                endpoint_profile,
                delivery_policy_hint,
                priority_bp,
                window.window_id.clone(),
                window.evidence_ref.clone(),
            ) {
                Ok(hint) => hint,
                Err(_) => {
                    return self.refuse(
                        ListenCapabilityId::ListenSignalCollect,
                        reason_codes::PH1_LISTEN_INTERNAL_PIPELINE_ERROR,
                        "failed to build listen adjustment hint",
                    );
                }
            };
            adjustments.push(adjustment);
        }

        adjustments.sort_by(|a, b| {
            b.priority_bp
                .cmp(&a.priority_bp)
                .then(a.adjustment_id.cmp(&b.adjustment_id))
        });
        adjustments.truncate(adjustment_budget);

        if adjustments.is_empty() {
            return self.refuse(
                ListenCapabilityId::ListenSignalCollect,
                reason_codes::PH1_LISTEN_UPSTREAM_INPUT_MISSING,
                "no listen adjustments were produced",
            );
        }

        let selected_adjustment_id = adjustments[0].adjustment_id.clone();
        let environment_profile_ref = environment_profile_ref(&adjustments[0]);

        match ListenSignalCollectOk::v1(
            reason_codes::PH1_LISTEN_OK_SIGNAL_COLLECT,
            environment_profile_ref,
            selected_adjustment_id,
            adjustments,
            true,
            true,
            true,
            true,
            true,
        ) {
            Ok(ok) => Ph1ListenResponse::ListenSignalCollectOk(ok),
            Err(_) => self.refuse(
                ListenCapabilityId::ListenSignalCollect,
                reason_codes::PH1_LISTEN_INTERNAL_PIPELINE_ERROR,
                "failed to construct listen collect output",
            ),
        }
    }

    fn run_signal_filter(&self, req: &ListenSignalFilterRequest) -> Ph1ListenResponse {
        if req.ordered_adjustments.is_empty() {
            return self.refuse(
                ListenCapabilityId::ListenSignalFilter,
                reason_codes::PH1_LISTEN_UPSTREAM_INPUT_MISSING,
                "ordered_adjustments is empty",
            );
        }

        let adjustment_budget = min(
            req.envelope.max_adjustments as usize,
            self.config.max_adjustments as usize,
        );
        if req.ordered_adjustments.len() > adjustment_budget {
            return self.refuse(
                ListenCapabilityId::ListenSignalFilter,
                reason_codes::PH1_LISTEN_BUDGET_EXCEEDED,
                "ordered_adjustments exceeds configured budget",
            );
        }

        let mut diagnostics = Vec::new();
        if req.ordered_adjustments[0].adjustment_id != req.selected_adjustment_id {
            diagnostics.push("selected_not_first_in_ordered_adjustments".to_string());
        }

        let selected = req
            .ordered_adjustments
            .iter()
            .find(|adjustment| adjustment.adjustment_id == req.selected_adjustment_id);
        if selected.is_none() {
            diagnostics.push("selected_adjustment_missing_from_ordered_adjustments".to_string());
        }

        if req.ordered_adjustments.windows(2).any(|pair| {
            pair[0].priority_bp < pair[1].priority_bp
                || (pair[0].priority_bp == pair[1].priority_bp
                    && pair[0].adjustment_id > pair[1].adjustment_id)
        }) {
            diagnostics.push("priority_not_sorted_desc".to_string());
        }

        let mut seen = BTreeSet::new();
        if req
            .ordered_adjustments
            .iter()
            .any(|adjustment| !seen.insert(adjustment.adjustment_id.as_str()))
        {
            diagnostics.push("duplicate_adjustment_id".to_string());
        }

        if !req.no_meaning_mutation_required {
            diagnostics.push("meaning_mutation_guard_not_set".to_string());
        }

        let mut applies_capture_profile = true;
        let mut applies_endpoint_profile = true;
        let mut applies_delivery_policy_hint = true;
        if let Some(selected) = selected {
            let expected_env_token = environment_mode_token(selected.environment_mode);
            if !req.environment_profile_ref.contains(expected_env_token) {
                diagnostics.push("environment_profile_ref_mode_mismatch".to_string());
            }

            applies_endpoint_profile =
                !matches!(selected.endpoint_profile, ListenEndpointProfile::Balanced);
            applies_delivery_policy_hint = !matches!(
                selected.delivery_policy_hint,
                ListenDeliveryPolicyHint::VoicePreferred
            );
            applies_capture_profile =
                !matches!(selected.capture_profile, ListenCaptureProfile::Standard);

            if !applies_capture_profile
                && !applies_endpoint_profile
                && !applies_delivery_policy_hint
            {
                applies_capture_profile = true;
            }
        }

        diagnostics.truncate(self.config.max_diagnostics as usize);
        let (validation_status, reason_code) = if diagnostics.is_empty() {
            (
                ListenValidationStatus::Ok,
                reason_codes::PH1_LISTEN_OK_SIGNAL_FILTER,
            )
        } else {
            (
                ListenValidationStatus::Fail,
                reason_codes::PH1_LISTEN_VALIDATION_FAILED,
            )
        };

        match ListenSignalFilterOk::v1(
            reason_code,
            validation_status,
            diagnostics,
            applies_capture_profile,
            applies_endpoint_profile,
            applies_delivery_policy_hint,
            true,
            true,
            true,
        ) {
            Ok(ok) => Ph1ListenResponse::ListenSignalFilterOk(ok),
            Err(_) => self.refuse(
                ListenCapabilityId::ListenSignalFilter,
                reason_codes::PH1_LISTEN_INTERNAL_PIPELINE_ERROR,
                "failed to construct listen filter output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: ListenCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1ListenResponse {
        let refuse = ListenRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("ListenRefuse::v1 must construct for static message");
        Ph1ListenResponse::Refuse(refuse)
    }
}

fn capability_from_request(req: &Ph1ListenRequest) -> ListenCapabilityId {
    match req {
        Ph1ListenRequest::ListenSignalCollect(_) => ListenCapabilityId::ListenSignalCollect,
        Ph1ListenRequest::ListenSignalFilter(_) => ListenCapabilityId::ListenSignalFilter,
    }
}

fn classify_environment(
    window: &selene_kernel_contracts::ph1listen::ListenSignalWindow,
    req: &ListenSignalCollectRequest,
) -> ListenEnvironmentMode {
    if req.session_context.session_mode_meeting {
        return ListenEnvironmentMode::Meeting;
    }
    if req.session_context.session_mode_car {
        return ListenEnvironmentMode::Car;
    }
    if window.noise_level_dbfs >= -30
        || window.speech_likeness_bp < 5000
        || req.correction_snapshot.correction_rate_bp >= 2000
        || req.correction_snapshot.delivery_switch_count >= 2
    {
        return ListenEnvironmentMode::Noisy;
    }
    if window.noise_level_dbfs <= -55
        && window.vad_confidence_bp >= 8500
        && req.correction_snapshot.user_correction_count == 0
    {
        return ListenEnvironmentMode::Quiet;
    }
    ListenEnvironmentMode::Office
}

fn capture_profile_for(environment: ListenEnvironmentMode) -> ListenCaptureProfile {
    match environment {
        ListenEnvironmentMode::Meeting => ListenCaptureProfile::NearFieldBoost,
        ListenEnvironmentMode::Car => ListenCaptureProfile::AggressiveAec,
        ListenEnvironmentMode::Noisy => ListenCaptureProfile::NoiseSuppressed,
        ListenEnvironmentMode::Quiet | ListenEnvironmentMode::Office => {
            ListenCaptureProfile::Standard
        }
    }
}

fn endpoint_profile_for(environment: ListenEnvironmentMode) -> ListenEndpointProfile {
    match environment {
        ListenEnvironmentMode::Meeting | ListenEnvironmentMode::Car => {
            ListenEndpointProfile::ExtendTail
        }
        ListenEnvironmentMode::Noisy | ListenEnvironmentMode::Office => {
            ListenEndpointProfile::Balanced
        }
        ListenEnvironmentMode::Quiet => ListenEndpointProfile::EarlyClose,
    }
}

fn delivery_policy_for(
    environment: ListenEnvironmentMode,
    req: &ListenSignalCollectRequest,
) -> ListenDeliveryPolicyHint {
    if req.session_context.session_mode_meeting {
        return ListenDeliveryPolicyHint::TextOnlyMeeting;
    }
    if req.session_context.privacy_mode || req.session_context.text_preferred {
        return ListenDeliveryPolicyHint::TextPreferred;
    }
    if environment == ListenEnvironmentMode::Noisy
        && req.correction_snapshot.delivery_switch_count > 0
    {
        return ListenDeliveryPolicyHint::TextPreferred;
    }
    ListenDeliveryPolicyHint::VoicePreferred
}

fn priority_score(
    window: &selene_kernel_contracts::ph1listen::ListenSignalWindow,
    environment: ListenEnvironmentMode,
    req: &ListenSignalCollectRequest,
) -> i16 {
    let base = match environment {
        ListenEnvironmentMode::Meeting => 1400,
        ListenEnvironmentMode::Car => 1320,
        ListenEnvironmentMode::Noisy => 1240,
        ListenEnvironmentMode::Office => 940,
        ListenEnvironmentMode::Quiet => 860,
    };

    let correction_component = (req.correction_snapshot.user_correction_count.min(20) as i32) * 25;
    let delivery_component = (req.correction_snapshot.delivery_switch_count.min(20) as i32) * 30;
    let noise_component = ((window.noise_level_dbfs + 100) as i32).max(0) * 6;
    let overlap_component = if window.overlap_tts_ms > 0 { 120 } else { 0 };
    let uncertainty_component = ((10_000u16.saturating_sub(window.speech_likeness_bp)) as i32) / 40;

    (base
        + correction_component
        + delivery_component
        + noise_component
        + overlap_component
        + uncertainty_component)
        .clamp(-20_000, 20_000) as i16
}

fn environment_profile_ref(selected: &ListenAdjustmentHint) -> String {
    format!(
        "env:{}:{}:{}",
        environment_mode_token(selected.environment_mode),
        capture_profile_token(selected.capture_profile),
        delivery_policy_token(selected.delivery_policy_hint)
    )
}

fn environment_mode_token(mode: ListenEnvironmentMode) -> &'static str {
    match mode {
        ListenEnvironmentMode::Quiet => "quiet",
        ListenEnvironmentMode::Noisy => "noisy",
        ListenEnvironmentMode::Meeting => "meeting",
        ListenEnvironmentMode::Car => "car",
        ListenEnvironmentMode::Office => "office",
    }
}

fn capture_profile_token(profile: ListenCaptureProfile) -> &'static str {
    match profile {
        ListenCaptureProfile::Standard => "standard",
        ListenCaptureProfile::NoiseSuppressed => "noise_suppressed",
        ListenCaptureProfile::NearFieldBoost => "near_field_boost",
        ListenCaptureProfile::AggressiveAec => "aggressive_aec",
    }
}

fn delivery_policy_token(hint: ListenDeliveryPolicyHint) -> &'static str {
    match hint {
        ListenDeliveryPolicyHint::VoicePreferred => "voice_preferred",
        ListenDeliveryPolicyHint::TextPreferred => "text_preferred",
        ListenDeliveryPolicyHint::TextOnlyMeeting => "text_only_meeting",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1listen::{
        ListenCorrectionSnapshot, ListenRequestEnvelope, ListenSessionContext,
        ListenSignalCollectRequest, ListenSignalFilterRequest,
    };

    fn runtime() -> Ph1ListenRuntime {
        Ph1ListenRuntime::new(Ph1ListenConfig::mvp_v1())
    }

    fn envelope(max_signal_windows: u8, max_adjustments: u8) -> ListenRequestEnvelope {
        ListenRequestEnvelope::v1(
            CorrelationId(3401),
            TurnId(311),
            max_signal_windows,
            max_adjustments,
            8,
        )
        .unwrap()
    }

    fn window(
        id: &str,
        noise_level_dbfs: i16,
    ) -> selene_kernel_contracts::ph1listen::ListenSignalWindow {
        selene_kernel_contracts::ph1listen::ListenSignalWindow::v1(
            id.to_string(),
            "PH1.K".to_string(),
            8300,
            7900,
            noise_level_dbfs,
            0,
            420,
            format!("listen:evidence:{}", id),
        )
        .unwrap()
    }

    fn correction(delivery_switch_count: u16) -> ListenCorrectionSnapshot {
        ListenCorrectionSnapshot::v1(2, delivery_switch_count, 1, 1500).unwrap()
    }

    fn context(session_mode_meeting: bool) -> ListenSessionContext {
        ListenSessionContext::v1(session_mode_meeting, false, false, false).unwrap()
    }

    fn collect_request() -> ListenSignalCollectRequest {
        ListenSignalCollectRequest::v1(
            envelope(8, 4),
            vec![window("w_1", -28), window("w_2", -46)],
            correction(1),
            context(false),
        )
        .unwrap()
    }

    #[test]
    fn at_listen_01_collect_output_is_schema_valid() {
        let req = Ph1ListenRequest::ListenSignalCollect(collect_request());

        let out = runtime().run(&req);
        assert!(out.validate().is_ok());
        match out {
            Ph1ListenResponse::ListenSignalCollectOk(ok) => {
                assert!(!ok.ordered_adjustments.is_empty());
                assert!(!ok.environment_profile_ref.is_empty());
            }
            _ => panic!("expected ListenSignalCollectOk"),
        }
    }

    #[test]
    fn at_listen_02_collect_order_is_deterministic() {
        let req = Ph1ListenRequest::ListenSignalCollect(collect_request());
        let runtime = runtime();

        let out1 = runtime.run(&req);
        let out2 = runtime.run(&req);

        match (out1, out2) {
            (
                Ph1ListenResponse::ListenSignalCollectOk(a),
                Ph1ListenResponse::ListenSignalCollectOk(b),
            ) => {
                assert_eq!(a.environment_profile_ref, b.environment_profile_ref);
                assert_eq!(a.selected_adjustment_id, b.selected_adjustment_id);
                assert_eq!(a.ordered_adjustments, b.ordered_adjustments);
            }
            _ => panic!("expected ListenSignalCollectOk outputs"),
        }
    }

    #[test]
    fn at_listen_03_budget_bound_is_enforced() {
        let runtime = Ph1ListenRuntime::new(Ph1ListenConfig {
            max_signal_windows: 1,
            max_adjustments: 1,
            max_diagnostics: 8,
        });

        let req = Ph1ListenRequest::ListenSignalCollect(collect_request());
        let out = runtime.run(&req);

        match out {
            Ph1ListenResponse::Refuse(refuse) => {
                assert_eq!(refuse.reason_code, reason_codes::PH1_LISTEN_BUDGET_EXCEEDED)
            }
            _ => panic!("expected Refuse"),
        }
    }

    #[test]
    fn at_listen_04_signal_filter_fails_on_selection_drift() {
        let collect_out = runtime().run(&Ph1ListenRequest::ListenSignalCollect(collect_request()));
        let collect_ok = match collect_out {
            Ph1ListenResponse::ListenSignalCollectOk(ok) => ok,
            _ => panic!("expected ListenSignalCollectOk"),
        };
        assert!(collect_ok.ordered_adjustments.len() >= 2);

        let drift_req = Ph1ListenRequest::ListenSignalFilter(
            ListenSignalFilterRequest::v1(
                envelope(8, 4),
                collect_ok.environment_profile_ref,
                collect_ok.ordered_adjustments[1].adjustment_id.clone(),
                collect_ok.ordered_adjustments,
                true,
            )
            .unwrap(),
        );

        let out = runtime().run(&drift_req);
        match out {
            Ph1ListenResponse::ListenSignalFilterOk(ok) => {
                assert_eq!(ok.validation_status, ListenValidationStatus::Fail)
            }
            _ => panic!("expected ListenSignalFilterOk"),
        }
    }
}
