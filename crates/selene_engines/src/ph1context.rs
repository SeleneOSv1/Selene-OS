#![forbid(unsafe_code)]

use std::cmp::min;
use std::collections::BTreeSet;

use selene_kernel_contracts::ph1context::{
    ContextBundleBuildOk, ContextBundleBuildRequest, ContextBundleItem, ContextBundleTrimOk,
    ContextBundleTrimRequest, ContextCapabilityId, ContextRefuse, ContextSourceItem,
    ContextSourceKind, ContextValidationStatus, Ph1ContextRequest, Ph1ContextResponse,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.CONTEXT reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_CONTEXT_OK_BUNDLE_BUILD: ReasonCodeId = ReasonCodeId(0x4358_0001);
    pub const PH1_CONTEXT_OK_BUNDLE_TRIM: ReasonCodeId = ReasonCodeId(0x4358_0002);

    pub const PH1_CONTEXT_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x4358_00F1);
    pub const PH1_CONTEXT_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x4358_00F2);
    pub const PH1_CONTEXT_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x4358_00F3);
    pub const PH1_CONTEXT_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4358_00F4);
    pub const PH1_CONTEXT_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4358_00F5);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1ContextConfig {
    pub max_items: u8,
    pub max_selected_items: u8,
    pub max_diagnostics: u8,
}

impl Ph1ContextConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_items: 12,
            max_selected_items: 3,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1ContextRuntime {
    config: Ph1ContextConfig,
}

impl Ph1ContextRuntime {
    pub fn new(config: Ph1ContextConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1ContextRequest) -> Ph1ContextResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_CONTEXT_INPUT_SCHEMA_INVALID,
                "context request failed contract validation",
            );
        }

        match req {
            Ph1ContextRequest::ContextBundleBuild(r) => self.run_bundle_build(r),
            Ph1ContextRequest::ContextBundleTrim(r) => self.run_bundle_trim(r),
        }
    }

    fn run_bundle_build(&self, req: &ContextBundleBuildRequest) -> Ph1ContextResponse {
        if req.intent_type.trim().is_empty() || req.source_items.is_empty() {
            return self.refuse(
                ContextCapabilityId::ContextBundleBuild,
                reason_codes::PH1_CONTEXT_UPSTREAM_INPUT_MISSING,
                "context intent/source input is missing",
            );
        }

        let budget = min(
            req.envelope.max_items as usize,
            self.config.max_items as usize,
        );
        if req.source_items.len() > budget {
            return self.refuse(
                ContextCapabilityId::ContextBundleBuild,
                reason_codes::PH1_CONTEXT_BUDGET_EXCEEDED,
                "source items exceed configured budget",
            );
        }

        if !req.multi_signal_align_ok
            && req
                .source_items
                .iter()
                .any(|item| item.source_kind == ContextSourceKind::MultiHint)
        {
            return self.refuse(
                ContextCapabilityId::ContextBundleBuild,
                reason_codes::PH1_CONTEXT_VALIDATION_FAILED,
                "multi hint input requires multi_signal_align_ok=true",
            );
        }

        if !req.cache_hint_refresh_ok
            && req
                .source_items
                .iter()
                .any(|item| item.source_kind == ContextSourceKind::CacheHint)
        {
            return self.refuse(
                ContextCapabilityId::ContextBundleBuild,
                reason_codes::PH1_CONTEXT_VALIDATION_FAILED,
                "cache hint input requires cache_hint_refresh_ok=true",
            );
        }

        let ordered_source = ordered_context_source_items(req.source_items.as_slice());
        let mut ordered_bundle_items = Vec::new();
        for (idx, source_item) in ordered_source.into_iter().take(budget).enumerate() {
            let item = match ContextBundleItem::v1(
                source_item.item_id,
                source_item.source_engine,
                source_item.source_kind,
                (idx + 1) as u8,
                source_item.content_ref,
                source_item.evidence_ref,
                source_item.sensitivity_private,
            ) {
                Ok(item) => item,
                Err(_) => {
                    return self.refuse(
                        ContextCapabilityId::ContextBundleBuild,
                        reason_codes::PH1_CONTEXT_INTERNAL_PIPELINE_ERROR,
                        "failed to construct context bundle item",
                    );
                }
            };
            ordered_bundle_items.push(item);
        }

        if ordered_bundle_items.is_empty() {
            return self.refuse(
                ContextCapabilityId::ContextBundleBuild,
                reason_codes::PH1_CONTEXT_UPSTREAM_INPUT_MISSING,
                "no context bundle items were produced",
            );
        }

        let selected_limit = min(
            self.config.max_selected_items as usize,
            ordered_bundle_items.len(),
        );
        let selected_item_ids = ordered_bundle_items
            .iter()
            .take(selected_limit)
            .map(|item| item.item_id.clone())
            .collect::<Vec<_>>();

        let preserved_evidence_refs = ordered_bundle_items
            .iter()
            .all(|item| !item.evidence_ref.trim().is_empty());

        match ContextBundleBuildOk::v1(
            reason_codes::PH1_CONTEXT_OK_BUNDLE_BUILD,
            selected_item_ids,
            ordered_bundle_items,
            preserved_evidence_refs,
            true,
            true,
        ) {
            Ok(ok) => Ph1ContextResponse::ContextBundleBuildOk(ok),
            Err(_) => self.refuse(
                ContextCapabilityId::ContextBundleBuild,
                reason_codes::PH1_CONTEXT_INTERNAL_PIPELINE_ERROR,
                "failed to construct context bundle build output",
            ),
        }
    }

    fn run_bundle_trim(&self, req: &ContextBundleTrimRequest) -> Ph1ContextResponse {
        if req.intent_type.trim().is_empty() || req.ordered_bundle_items.is_empty() {
            return self.refuse(
                ContextCapabilityId::ContextBundleTrim,
                reason_codes::PH1_CONTEXT_UPSTREAM_INPUT_MISSING,
                "context trim input is missing",
            );
        }

        let budget = min(
            req.envelope.max_items as usize,
            self.config.max_items as usize,
        );
        if req.ordered_bundle_items.len() > budget {
            return self.refuse(
                ContextCapabilityId::ContextBundleTrim,
                reason_codes::PH1_CONTEXT_BUDGET_EXCEEDED,
                "ordered context bundle exceeds configured budget",
            );
        }

        let mut diagnostics = Vec::new();
        let mut seen_ids = BTreeSet::new();
        let mut expected_rank = 1u8;

        for item in &req.ordered_bundle_items {
            if !seen_ids.insert(item.item_id.as_str()) {
                diagnostics.push("duplicate_bundle_item_id".to_string());
                break;
            }
            if item.bundle_rank != expected_rank {
                diagnostics.push("bundle_rank_sequence_gap_detected".to_string());
                break;
            }
            expected_rank = expected_rank.saturating_add(1);
        }

        if req.selected_item_ids[0] != req.ordered_bundle_items[0].item_id {
            diagnostics.push("selected_not_first_in_ordered_bundle".to_string());
        }

        for selected_item_id in &req.selected_item_ids {
            if !seen_ids.contains(selected_item_id.as_str()) {
                diagnostics.push("selected_item_missing_from_bundle".to_string());
                break;
            }
        }

        let preserved_evidence_refs = req
            .ordered_bundle_items
            .iter()
            .all(|item| !item.evidence_ref.trim().is_empty());
        if !preserved_evidence_refs {
            diagnostics.push("missing_evidence_ref_in_bundle_item".to_string());
        }

        if !req.multi_signal_align_ok
            && req
                .ordered_bundle_items
                .iter()
                .any(|item| item.source_kind == ContextSourceKind::MultiHint)
        {
            diagnostics.push("multi_signal_align_not_ok".to_string());
        }

        if !req.cache_hint_refresh_ok
            && req
                .ordered_bundle_items
                .iter()
                .any(|item| item.source_kind == ContextSourceKind::CacheHint)
        {
            diagnostics.push("cache_hint_refresh_not_ok".to_string());
        }

        diagnostics.truncate(self.config.max_diagnostics as usize);
        let preserved_ranked_source_order = diagnostics.is_empty();

        let (validation_status, reason_code) = if diagnostics.is_empty() {
            (
                ContextValidationStatus::Ok,
                reason_codes::PH1_CONTEXT_OK_BUNDLE_TRIM,
            )
        } else {
            (
                ContextValidationStatus::Fail,
                reason_codes::PH1_CONTEXT_VALIDATION_FAILED,
            )
        };

        match ContextBundleTrimOk::v1(
            reason_code,
            validation_status,
            diagnostics,
            preserved_ranked_source_order,
            preserved_evidence_refs,
            true,
            true,
        ) {
            Ok(ok) => Ph1ContextResponse::ContextBundleTrimOk(ok),
            Err(_) => self.refuse(
                ContextCapabilityId::ContextBundleTrim,
                reason_codes::PH1_CONTEXT_INTERNAL_PIPELINE_ERROR,
                "failed to construct context bundle trim output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: ContextCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1ContextResponse {
        let refuse = ContextRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("ContextRefuse::v1 must construct for static message");
        Ph1ContextResponse::Refuse(refuse)
    }
}

fn capability_from_request(req: &Ph1ContextRequest) -> ContextCapabilityId {
    match req {
        Ph1ContextRequest::ContextBundleBuild(_) => ContextCapabilityId::ContextBundleBuild,
        Ph1ContextRequest::ContextBundleTrim(_) => ContextCapabilityId::ContextBundleTrim,
    }
}

fn ordered_context_source_items(items: &[ContextSourceItem]) -> Vec<ContextSourceItem> {
    let mut ordered = items.to_vec();
    ordered.sort_by(|a, b| {
        b.rank_score_bp
            .cmp(&a.rank_score_bp)
            .then(a.item_id.cmp(&b.item_id))
    });
    ordered
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1context::{
        ContextBundleBuildRequest, ContextBundleTrimRequest, ContextRequestEnvelope,
    };
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};

    fn runtime() -> Ph1ContextRuntime {
        Ph1ContextRuntime::new(Ph1ContextConfig::mvp_v1())
    }

    fn envelope(max_items: u8, max_diagnostics: u8) -> ContextRequestEnvelope {
        ContextRequestEnvelope::v1(
            CorrelationId(9201),
            TurnId(511),
            max_items,
            max_diagnostics,
            true,
        )
        .unwrap()
    }

    fn source_item(
        id: &str,
        source_kind: ContextSourceKind,
        rank_score_bp: i16,
    ) -> ContextSourceItem {
        ContextSourceItem::v1(
            id.to_string(),
            "PH1.SUMMARY".to_string(),
            source_kind,
            rank_score_bp,
            format!("context:content:{}", id),
            format!("context:evidence:{}", id),
            false,
        )
        .unwrap()
    }

    fn build_request() -> ContextBundleBuildRequest {
        ContextBundleBuildRequest::v1(
            envelope(8, 6),
            "QUERY_WEATHER".to_string(),
            false,
            vec![
                source_item("ctx_1", ContextSourceKind::SummaryEvidence, 1200),
                source_item("ctx_2", ContextSourceKind::WebEvidence, 1100),
                source_item("ctx_3", ContextSourceKind::Memory, 900),
            ],
            true,
            true,
        )
        .unwrap()
    }

    #[test]
    fn at_context_01_bundle_build_output_is_schema_valid() {
        let req = Ph1ContextRequest::ContextBundleBuild(build_request());

        let out = runtime().run(&req);
        assert!(out.validate().is_ok());
        match out {
            Ph1ContextResponse::ContextBundleBuildOk(ok) => {
                assert!(!ok.selected_item_ids.is_empty());
                assert!(!ok.ordered_bundle_items.is_empty());
            }
            _ => panic!("expected ContextBundleBuildOk"),
        }
    }

    #[test]
    fn at_context_02_bundle_build_order_is_deterministic() {
        let req = Ph1ContextRequest::ContextBundleBuild(build_request());
        let runtime = runtime();

        let out1 = runtime.run(&req);
        let out2 = runtime.run(&req);

        match (out1, out2) {
            (
                Ph1ContextResponse::ContextBundleBuildOk(a),
                Ph1ContextResponse::ContextBundleBuildOk(b),
            ) => {
                assert_eq!(a.selected_item_ids, b.selected_item_ids);
                assert_eq!(a.ordered_bundle_items, b.ordered_bundle_items);
            }
            _ => panic!("expected ContextBundleBuildOk outputs"),
        }
    }

    #[test]
    fn at_context_03_budget_bound_is_enforced() {
        let runtime = Ph1ContextRuntime::new(Ph1ContextConfig {
            max_items: 2,
            max_selected_items: 2,
            max_diagnostics: 8,
        });

        let req = Ph1ContextRequest::ContextBundleBuild(build_request());
        let out = runtime.run(&req);

        match out {
            Ph1ContextResponse::Refuse(refuse) => {
                assert_eq!(
                    refuse.reason_code,
                    reason_codes::PH1_CONTEXT_BUDGET_EXCEEDED
                );
            }
            _ => panic!("expected Refuse"),
        }
    }

    #[test]
    fn at_context_04_bundle_trim_fails_on_multi_align_drift() {
        let build_out = runtime().run(&Ph1ContextRequest::ContextBundleBuild(
            ContextBundleBuildRequest::v1(
                envelope(8, 6),
                "QUERY_WEATHER".to_string(),
                false,
                vec![source_item("ctx_1", ContextSourceKind::MultiHint, 1200)],
                true,
                true,
            )
            .unwrap(),
        ));

        let build_ok = match build_out {
            Ph1ContextResponse::ContextBundleBuildOk(ok) => ok,
            _ => panic!("expected ContextBundleBuildOk"),
        };

        let trim_req = Ph1ContextRequest::ContextBundleTrim(
            ContextBundleTrimRequest::v1(
                envelope(8, 6),
                "QUERY_WEATHER".to_string(),
                false,
                build_ok.selected_item_ids,
                build_ok.ordered_bundle_items,
                false,
                true,
            )
            .unwrap(),
        );

        let out = runtime().run(&trim_req);
        match out {
            Ph1ContextResponse::ContextBundleTrimOk(ok) => {
                assert_eq!(ok.validation_status, ContextValidationStatus::Fail);
            }
            _ => panic!("expected ContextBundleTrimOk"),
        }
    }
}
