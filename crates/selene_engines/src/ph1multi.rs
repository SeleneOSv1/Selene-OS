#![forbid(unsafe_code)]

use std::cmp::min;
use std::collections::BTreeSet;

use selene_kernel_contracts::ph1multi::{
    MultiBundleComposeOk, MultiBundleComposeRequest, MultiBundleItem, MultiCapabilityId,
    MultiModality, MultiRefuse, MultiSignalAlignOk, MultiSignalAlignRequest, MultiValidationStatus,
    Ph1MultiRequest, Ph1MultiResponse,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.MULTI reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_MULTI_OK_BUNDLE_COMPOSE: ReasonCodeId = ReasonCodeId(0x4D55_0001);
    pub const PH1_MULTI_OK_SIGNAL_ALIGN: ReasonCodeId = ReasonCodeId(0x4D55_0002);

    pub const PH1_MULTI_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x4D55_00F1);
    pub const PH1_MULTI_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x4D55_00F2);
    pub const PH1_MULTI_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x4D55_00F3);
    pub const PH1_MULTI_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4D55_00F4);
    pub const PH1_MULTI_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4D55_00F5);
    pub const PH1_MULTI_PRIVACY_SCOPE_REQUIRED: ReasonCodeId = ReasonCodeId(0x4D55_00F6);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1MultiConfig {
    pub max_signals: u8,
    pub max_bundle_items: u8,
    pub max_diagnostics: u8,
}

impl Ph1MultiConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_signals: 32,
            max_bundle_items: 16,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1MultiRuntime {
    config: Ph1MultiConfig,
}

impl Ph1MultiRuntime {
    pub fn new(config: Ph1MultiConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1MultiRequest) -> Ph1MultiResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_MULTI_INPUT_SCHEMA_INVALID,
                "multi request failed contract validation",
            );
        }

        match req {
            Ph1MultiRequest::MultiBundleCompose(r) => self.run_bundle_compose(r),
            Ph1MultiRequest::MultiSignalAlign(r) => self.run_signal_align(r),
        }
    }

    fn run_bundle_compose(&self, req: &MultiBundleComposeRequest) -> Ph1MultiResponse {
        if req.signals.is_empty() {
            return self.refuse(
                MultiCapabilityId::MultiBundleCompose,
                reason_codes::PH1_MULTI_UPSTREAM_INPUT_MISSING,
                "signals is empty",
            );
        }

        let signal_budget = min(req.envelope.max_signals, self.config.max_signals) as usize;
        if req.signals.len() > signal_budget {
            return self.refuse(
                MultiCapabilityId::MultiBundleCompose,
                reason_codes::PH1_MULTI_BUDGET_EXCEEDED,
                "signals exceeds configured budget",
            );
        }

        let item_budget = min(req.envelope.max_bundle_items, self.config.max_bundle_items) as usize;
        if item_budget == 0 {
            return self.refuse(
                MultiCapabilityId::MultiBundleCompose,
                reason_codes::PH1_MULTI_BUDGET_EXCEEDED,
                "bundle item budget exceeded",
            );
        }

        if req.signals.iter().any(|signal| !signal.privacy_scoped) {
            return self.refuse(
                MultiCapabilityId::MultiBundleCompose,
                reason_codes::PH1_MULTI_PRIVACY_SCOPE_REQUIRED,
                "all signals must be privacy-scoped",
            );
        }

        let mut scored = req
            .signals
            .iter()
            .map(|signal| {
                (
                    signal.signal_id.clone(),
                    signal.source_engine.clone(),
                    signal.modality,
                    signal.confidence_pct,
                    signal.evidence_ref.clone(),
                    score(
                        signal.confidence_pct,
                        signal.modality,
                        &signal.source_engine,
                    ),
                )
            })
            .collect::<Vec<_>>();

        scored.sort_by(|a, b| b.5.cmp(&a.5).then(a.0.cmp(&b.0)));

        let mut dedupe = BTreeSet::new();
        let mut ordered_bundle_items = Vec::new();
        for (signal_id, source_engine, modality, confidence_pct, evidence_ref, _) in scored {
            if !dedupe.insert(signal_id.clone()) {
                continue;
            }
            if ordered_bundle_items.len() >= item_budget {
                break;
            }

            let item = match MultiBundleItem::v1(
                signal_id,
                source_engine,
                modality,
                (ordered_bundle_items.len() + 1) as u8,
                confidence_pct,
                evidence_ref,
            ) {
                Ok(item) => item,
                Err(_) => {
                    return self.refuse(
                        MultiCapabilityId::MultiBundleCompose,
                        reason_codes::PH1_MULTI_INTERNAL_PIPELINE_ERROR,
                        "failed to construct bundle item",
                    )
                }
            };

            ordered_bundle_items.push(item);
        }

        if ordered_bundle_items.is_empty() {
            return self.refuse(
                MultiCapabilityId::MultiBundleCompose,
                reason_codes::PH1_MULTI_UPSTREAM_INPUT_MISSING,
                "no bundle items could be composed",
            );
        }

        let evidence_backed = ordered_bundle_items.iter().all(|item| {
            if matches!(
                item.modality,
                MultiModality::Vision | MultiModality::Document
            ) {
                item.evidence_ref.is_some()
            } else {
                true
            }
        });

        if !evidence_backed {
            return self.refuse(
                MultiCapabilityId::MultiBundleCompose,
                reason_codes::PH1_MULTI_VALIDATION_FAILED,
                "vision/document items require evidence_ref",
            );
        }

        let selected_signal_id = ordered_bundle_items[0].signal_id.clone();
        match MultiBundleComposeOk::v1(
            reason_codes::PH1_MULTI_OK_BUNDLE_COMPOSE,
            selected_signal_id,
            ordered_bundle_items,
            true,
            true,
            true,
        ) {
            Ok(ok) => Ph1MultiResponse::MultiBundleComposeOk(ok),
            Err(_) => self.refuse(
                MultiCapabilityId::MultiBundleCompose,
                reason_codes::PH1_MULTI_INTERNAL_PIPELINE_ERROR,
                "failed to construct bundle compose output",
            ),
        }
    }

    fn run_signal_align(&self, req: &MultiSignalAlignRequest) -> Ph1MultiResponse {
        if req.ordered_bundle_items.is_empty() {
            return self.refuse(
                MultiCapabilityId::MultiSignalAlign,
                reason_codes::PH1_MULTI_UPSTREAM_INPUT_MISSING,
                "ordered_bundle_items is empty",
            );
        }

        let budget = min(req.envelope.max_bundle_items, self.config.max_bundle_items) as usize;
        if req.ordered_bundle_items.len() > budget {
            return self.refuse(
                MultiCapabilityId::MultiSignalAlign,
                reason_codes::PH1_MULTI_BUDGET_EXCEEDED,
                "ordered_bundle_items exceeds configured budget",
            );
        }

        let mut diagnostics = Vec::new();

        if req.ordered_bundle_items[0].signal_id != req.selected_signal_id {
            diagnostics.push("selected_not_first_in_ordered_bundle".to_string());
        }
        if !req
            .ordered_bundle_items
            .iter()
            .any(|item| item.signal_id == req.selected_signal_id)
        {
            diagnostics.push("selected_signal_not_present_in_bundle".to_string());
        }

        let mut expected_rank = 1u8;
        for item in &req.ordered_bundle_items {
            if item.fused_rank != expected_rank {
                diagnostics.push("fused_rank_sequence_gap_detected".to_string());
                break;
            }
            expected_rank = expected_rank.saturating_add(1);
        }

        if req
            .ordered_bundle_items
            .windows(2)
            .any(|pair| pair[0].confidence_pct < pair[1].confidence_pct)
        {
            diagnostics.push("confidence_not_sorted_desc".to_string());
        }

        let mut signal_ids = BTreeSet::new();
        if req
            .ordered_bundle_items
            .iter()
            .any(|item| !signal_ids.insert(item.signal_id.as_str()))
        {
            diagnostics.push("duplicate_signal_id".to_string());
        }

        for item in &req.ordered_bundle_items {
            if matches!(
                item.modality,
                MultiModality::Vision | MultiModality::Document
            ) && item.evidence_ref.is_none()
            {
                diagnostics.push("vision_or_document_item_missing_evidence_ref".to_string());
                break;
            }
        }

        diagnostics.truncate(self.config.max_diagnostics as usize);

        let (validation_status, reason_code) = if diagnostics.is_empty() {
            (
                MultiValidationStatus::Ok,
                reason_codes::PH1_MULTI_OK_SIGNAL_ALIGN,
            )
        } else {
            (
                MultiValidationStatus::Fail,
                reason_codes::PH1_MULTI_VALIDATION_FAILED,
            )
        };

        match MultiSignalAlignOk::v1(
            reason_code,
            validation_status,
            diagnostics,
            true,
            true,
            true,
        ) {
            Ok(ok) => Ph1MultiResponse::MultiSignalAlignOk(ok),
            Err(_) => self.refuse(
                MultiCapabilityId::MultiSignalAlign,
                reason_codes::PH1_MULTI_INTERNAL_PIPELINE_ERROR,
                "failed to construct signal align output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: MultiCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1MultiResponse {
        let out = MultiRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("MultiRefuse::v1 must construct for static messages");
        Ph1MultiResponse::Refuse(out)
    }
}

fn capability_from_request(req: &Ph1MultiRequest) -> MultiCapabilityId {
    match req {
        Ph1MultiRequest::MultiBundleCompose(_) => MultiCapabilityId::MultiBundleCompose,
        Ph1MultiRequest::MultiSignalAlign(_) => MultiCapabilityId::MultiSignalAlign,
    }
}

fn score(confidence_pct: u8, modality: MultiModality, source_engine: &str) -> u8 {
    let mut score = confidence_pct as i16;
    score += modality_bonus(modality);
    score += source_bonus(source_engine);
    score.clamp(0, 100) as u8
}

fn modality_bonus(modality: MultiModality) -> i16 {
    match modality {
        MultiModality::Text => 12,
        MultiModality::Voice => 10,
        MultiModality::Vision => 8,
        MultiModality::Document => 7,
    }
}

fn source_bonus(source_engine: &str) -> i16 {
    match source_engine {
        "PH1.CACHE" => 10,
        "PH1.PAE" => 9,
        "PH1.LISTEN" => 8,
        "PH1.VISION" => 7,
        "PH1.DOC" => 7,
        _ => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1multi::{
        MultiRequestEnvelope, MultiSignalAlignRequest, MultiSourceSignal,
    };

    fn runtime() -> Ph1MultiRuntime {
        Ph1MultiRuntime::new(Ph1MultiConfig::mvp_v1())
    }

    fn envelope(max_signals: u8, max_bundle_items: u8) -> MultiRequestEnvelope {
        MultiRequestEnvelope::v1(
            CorrelationId(2201),
            TurnId(191),
            max_signals,
            max_bundle_items,
            true,
        )
        .unwrap()
    }

    fn signal(
        signal_id: &str,
        source_engine: &str,
        modality: MultiModality,
        confidence_pct: u8,
        evidence_ref: Option<&str>,
    ) -> MultiSourceSignal {
        MultiSourceSignal::v1(
            signal_id.to_string(),
            source_engine.to_string(),
            modality,
            "context_hint".to_string(),
            "value".to_string(),
            evidence_ref.map(|v| v.to_string()),
            confidence_pct,
            true,
        )
        .unwrap()
    }

    fn base_signals() -> Vec<MultiSourceSignal> {
        vec![
            signal("s_voice", "PH1.LISTEN", MultiModality::Voice, 82, None),
            signal("s_text", "PH1.CACHE", MultiModality::Text, 79, None),
            signal(
                "s_vision",
                "PH1.VISION",
                MultiModality::Vision,
                75,
                Some("vision:evidence:1"),
            ),
        ]
    }

    #[test]
    fn at_multi_01_bundle_compose_output_is_schema_valid() {
        let req = Ph1MultiRequest::MultiBundleCompose(
            MultiBundleComposeRequest::v1(envelope(8, 4), base_signals(), true).unwrap(),
        );
        let out = runtime().run(&req);
        assert!(out.validate().is_ok());
        match out {
            Ph1MultiResponse::MultiBundleComposeOk(ok) => {
                assert_eq!(ok.selected_signal_id, ok.ordered_bundle_items[0].signal_id);
            }
            _ => panic!("expected MultiBundleComposeOk"),
        }
    }

    #[test]
    fn at_multi_02_bundle_order_is_deterministic() {
        let req = Ph1MultiRequest::MultiBundleCompose(
            MultiBundleComposeRequest::v1(envelope(8, 4), base_signals(), true).unwrap(),
        );

        let out1 = runtime().run(&req);
        let out2 = runtime().run(&req);
        let ordered1 = match out1 {
            Ph1MultiResponse::MultiBundleComposeOk(ok) => ok.ordered_bundle_items,
            _ => panic!("expected MultiBundleComposeOk"),
        };
        let ordered2 = match out2 {
            Ph1MultiResponse::MultiBundleComposeOk(ok) => ok.ordered_bundle_items,
            _ => panic!("expected MultiBundleComposeOk"),
        };
        assert_eq!(ordered1, ordered2);
    }

    #[test]
    fn at_multi_03_vision_evidence_bundle_is_preserved_in_output() {
        let req = Ph1MultiRequest::MultiBundleCompose(
            MultiBundleComposeRequest::v1(envelope(8, 4), base_signals(), true).unwrap(),
        );

        let out = runtime().run(&req);
        match out {
            Ph1MultiResponse::MultiBundleComposeOk(ok) => {
                let vision = ok
                    .ordered_bundle_items
                    .iter()
                    .find(|item| item.modality == MultiModality::Vision)
                    .expect("vision item must exist");
                assert_eq!(vision.evidence_ref.as_deref(), Some("vision:evidence:1"));
            }
            _ => panic!("expected MultiBundleComposeOk"),
        }
    }

    #[test]
    fn at_multi_04_signal_align_detects_selection_drift() {
        let compose = runtime().run(&Ph1MultiRequest::MultiBundleCompose(
            MultiBundleComposeRequest::v1(envelope(8, 4), base_signals(), true).unwrap(),
        ));
        let compose_ok = match compose {
            Ph1MultiResponse::MultiBundleComposeOk(ok) => ok,
            _ => panic!("expected MultiBundleComposeOk"),
        };

        let req = Ph1MultiRequest::MultiSignalAlign(
            MultiSignalAlignRequest::v1(
                envelope(8, 4),
                compose_ok.ordered_bundle_items[1].signal_id.clone(),
                compose_ok.ordered_bundle_items,
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        match out {
            Ph1MultiResponse::MultiSignalAlignOk(ok) => {
                assert_eq!(ok.validation_status, MultiValidationStatus::Fail);
            }
            _ => panic!("expected MultiSignalAlignOk"),
        }
    }
}
