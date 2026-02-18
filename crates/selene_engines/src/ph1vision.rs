#![forbid(unsafe_code)]

use std::cmp::min;
use std::collections::BTreeSet;

use selene_kernel_contracts::ph1vision::{
    BoundingBoxPx, Ph1VisionRequest, Ph1VisionResponse, VisionCapabilityId,
    VisionEvidenceExtractOk, VisionEvidenceExtractRequest, VisionEvidenceItem, VisionRefuse,
    VisionValidationStatus, VisionVisibleContentValidateOk, VisionVisibleContentValidateRequest,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.VISION reason-code namespace. Values are placeholders until the global registry is formalized.
    pub const PH1_VISION_OK_EVIDENCE_EXTRACT: ReasonCodeId = ReasonCodeId(0x5649_0101);
    pub const PH1_VISION_OK_VISIBLE_CONTENT_VALIDATE: ReasonCodeId = ReasonCodeId(0x5649_0102);

    pub const PH1_VISION_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x5649_01F1);
    pub const PH1_VISION_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x5649_01F2);
    pub const PH1_VISION_OPT_IN_DISABLED: ReasonCodeId = ReasonCodeId(0x5649_01F3);
    pub const PH1_VISION_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x5649_01F4);
    pub const PH1_VISION_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5649_01F5);
    pub const PH1_VISION_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x5649_01F6);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1VisionConfig {
    pub max_extract_items: u8,
    pub max_input_tokens: usize,
    pub max_diagnostics: u8,
}

impl Ph1VisionConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_extract_items: 32,
            max_input_tokens: 256,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1VisionRuntime {
    config: Ph1VisionConfig,
}

impl Ph1VisionRuntime {
    pub fn new(config: Ph1VisionConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1VisionRequest) -> Ph1VisionResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_VISION_INPUT_SCHEMA_INVALID,
                "vision request failed contract validation",
            );
        }

        match req {
            Ph1VisionRequest::EvidenceExtract(r) => self.run_extract(r),
            Ph1VisionRequest::VisibleContentValidate(r) => self.run_visible_content_validate(r),
        }
    }

    fn run_extract(&self, req: &VisionEvidenceExtractRequest) -> Ph1VisionResponse {
        if !req.envelope.opt_in_enabled {
            return self.refuse(
                VisionCapabilityId::EvidenceExtract,
                reason_codes::PH1_VISION_OPT_IN_DISABLED,
                "vision opt-in is disabled",
            );
        }

        if req.visible_tokens.is_empty() {
            return self.refuse(
                VisionCapabilityId::EvidenceExtract,
                reason_codes::PH1_VISION_UPSTREAM_INPUT_MISSING,
                "no upstream visible tokens were provided",
            );
        }

        if req.visible_tokens.len() > self.config.max_input_tokens {
            return self.refuse(
                VisionCapabilityId::EvidenceExtract,
                reason_codes::PH1_VISION_BUDGET_EXCEEDED,
                "visible token budget exceeded",
            );
        }

        let budget = min(
            req.envelope.max_evidence_items,
            self.config.max_extract_items,
        ) as usize;
        if budget == 0 {
            return self.refuse(
                VisionCapabilityId::EvidenceExtract,
                reason_codes::PH1_VISION_BUDGET_EXCEEDED,
                "evidence budget exceeded",
            );
        }

        let mut dedupe: BTreeSet<(String, Option<BoundingBoxPx>)> = BTreeSet::new();
        let mut out: Vec<VisionEvidenceItem> = Vec::new();

        for token in &req.visible_tokens {
            if out.len() >= budget {
                break;
            }

            let canonical = canonical_text(&token.token);
            if canonical.is_empty() {
                continue;
            }
            let key = (canonical, token.bbox);
            if dedupe.insert(key) {
                match VisionEvidenceItem::v1(token.token.clone(), token.bbox) {
                    Ok(item) => out.push(item),
                    Err(_) => {
                        return self.refuse(
                            VisionCapabilityId::EvidenceExtract,
                            reason_codes::PH1_VISION_INTERNAL_PIPELINE_ERROR,
                            "failed to build vision evidence item",
                        )
                    }
                }
            }
        }

        if out.is_empty() {
            return self.refuse(
                VisionCapabilityId::EvidenceExtract,
                reason_codes::PH1_VISION_UPSTREAM_INPUT_MISSING,
                "no visible-content evidence could be extracted",
            );
        }

        match VisionEvidenceExtractOk::v1(
            reason_codes::PH1_VISION_OK_EVIDENCE_EXTRACT,
            req.source_ref.clone(),
            out,
            true,
        ) {
            Ok(ok) => Ph1VisionResponse::EvidenceExtractOk(ok),
            Err(_) => self.refuse(
                VisionCapabilityId::EvidenceExtract,
                reason_codes::PH1_VISION_INTERNAL_PIPELINE_ERROR,
                "failed to construct extract output",
            ),
        }
    }

    fn run_visible_content_validate(
        &self,
        req: &VisionVisibleContentValidateRequest,
    ) -> Ph1VisionResponse {
        if !req.envelope.opt_in_enabled {
            return self.refuse(
                VisionCapabilityId::VisibleContentValidate,
                reason_codes::PH1_VISION_OPT_IN_DISABLED,
                "vision opt-in is disabled",
            );
        }

        if req.visible_tokens.is_empty() {
            return self.refuse(
                VisionCapabilityId::VisibleContentValidate,
                reason_codes::PH1_VISION_UPSTREAM_INPUT_MISSING,
                "no upstream visible tokens were provided",
            );
        }

        if req.evidence_items.len() > req.envelope.max_evidence_items as usize {
            return self.refuse(
                VisionCapabilityId::VisibleContentValidate,
                reason_codes::PH1_VISION_BUDGET_EXCEEDED,
                "evidence item budget exceeded",
            );
        }

        let mut visible_pairs: BTreeSet<(String, Option<BoundingBoxPx>)> = BTreeSet::new();
        let mut visible_texts: BTreeSet<String> = BTreeSet::new();

        for token in &req.visible_tokens {
            let canonical = canonical_text(&token.token);
            if canonical.is_empty() {
                continue;
            }
            visible_texts.insert(canonical.clone());
            visible_pairs.insert((canonical, token.bbox));
        }

        let mut diagnostics: Vec<String> = Vec::new();

        for (idx, item) in req.evidence_items.iter().enumerate() {
            let canonical = canonical_text(&item.text);
            let matched = if let Some(bbox) = item.bbox {
                visible_pairs.contains(&(canonical, Some(bbox)))
            } else {
                visible_texts.contains(&canonical)
            };

            if !matched {
                diagnostics.push(format!("evidence_index_{idx}_not_visible_content"));
                if diagnostics.len() >= self.config.max_diagnostics as usize {
                    break;
                }
            }
        }

        let (status, reason_code) = if diagnostics.is_empty() {
            (
                VisionValidationStatus::Ok,
                reason_codes::PH1_VISION_OK_VISIBLE_CONTENT_VALIDATE,
            )
        } else {
            (
                VisionValidationStatus::Fail,
                reason_codes::PH1_VISION_VALIDATION_FAILED,
            )
        };

        match VisionVisibleContentValidateOk::v1(
            reason_code,
            req.source_ref.clone(),
            status,
            diagnostics,
            true,
        ) {
            Ok(ok) => Ph1VisionResponse::VisibleContentValidateOk(ok),
            Err(_) => self.refuse(
                VisionCapabilityId::VisibleContentValidate,
                reason_codes::PH1_VISION_INTERNAL_PIPELINE_ERROR,
                "failed to construct visible-content validation output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: VisionCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1VisionResponse {
        let r = VisionRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("VisionRefuse::v1 must construct for static message");
        Ph1VisionResponse::Refuse(r)
    }
}

fn capability_from_request(req: &Ph1VisionRequest) -> VisionCapabilityId {
    match req {
        Ph1VisionRequest::EvidenceExtract(_) => VisionCapabilityId::EvidenceExtract,
        Ph1VisionRequest::VisibleContentValidate(_) => VisionCapabilityId::VisibleContentValidate,
    }
}

fn canonical_text(input: &str) -> String {
    let lowered = input.trim().to_ascii_lowercase();
    lowered
        .split_whitespace()
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1vision::{
        VisionRequestEnvelope, VisualSourceId, VisualSourceKind, VisualSourceRef, VisualToken,
    };

    fn runtime() -> Ph1VisionRuntime {
        Ph1VisionRuntime::new(Ph1VisionConfig::mvp_v1())
    }

    fn env(opt_in: bool, max: u8) -> VisionRequestEnvelope {
        VisionRequestEnvelope::v1(CorrelationId(123), TurnId(7), opt_in, max).unwrap()
    }

    fn source() -> VisualSourceRef {
        VisualSourceRef::v1(
            VisualSourceId::new("src_001").unwrap(),
            VisualSourceKind::Screenshot,
        )
        .unwrap()
    }

    fn token(s: &str) -> VisualToken {
        VisualToken::v1(s.to_string(), None).unwrap()
    }

    #[test]
    fn at_vision_01_extract_output_is_schema_valid() {
        let req = Ph1VisionRequest::EvidenceExtract(
            VisionEvidenceExtractRequest::v1(
                env(true, 4),
                source(),
                vec![token("Revenue"), token("Cost")],
            )
            .unwrap(),
        );

        let resp = runtime().run(&req);
        assert!(resp.validate().is_ok());
        match resp {
            Ph1VisionResponse::EvidenceExtractOk(ok) => {
                assert_eq!(ok.capability_id, VisionCapabilityId::EvidenceExtract);
                assert!(ok.visible_content_only);
                assert_eq!(ok.evidence_items.len(), 2);
            }
            _ => panic!("expected EvidenceExtractOk"),
        }
    }

    #[test]
    fn at_vision_02_order_is_deterministic_and_preserved() {
        let req = Ph1VisionRequest::EvidenceExtract(
            VisionEvidenceExtractRequest::v1(
                env(true, 8),
                source(),
                vec![token("B"), token("A"), token("B"), token("C"), token("A")],
            )
            .unwrap(),
        );

        let runtime = runtime();
        let resp1 = runtime.run(&req);
        let resp2 = runtime.run(&req);

        let list1 = match resp1 {
            Ph1VisionResponse::EvidenceExtractOk(ok) => ok
                .evidence_items
                .into_iter()
                .map(|it| it.text)
                .collect::<Vec<_>>(),
            _ => panic!("expected EvidenceExtractOk"),
        };
        let list2 = match resp2 {
            Ph1VisionResponse::EvidenceExtractOk(ok) => ok
                .evidence_items
                .into_iter()
                .map(|it| it.text)
                .collect::<Vec<_>>(),
            _ => panic!("expected EvidenceExtractOk"),
        };

        assert_eq!(list1, vec!["B", "A", "C"]);
        assert_eq!(list1, list2);
    }

    #[test]
    fn at_vision_03_opt_in_disabled_skips_with_refuse() {
        let req = Ph1VisionRequest::EvidenceExtract(
            VisionEvidenceExtractRequest::v1(
                env(false, 4),
                source(),
                vec![token("should_not_run")],
            )
            .unwrap(),
        );

        let resp = runtime().run(&req);
        match resp {
            Ph1VisionResponse::Refuse(r) => {
                assert_eq!(r.reason_code, reason_codes::PH1_VISION_OPT_IN_DISABLED);
                assert_eq!(r.capability_id, VisionCapabilityId::EvidenceExtract);
            }
            _ => panic!("expected Refuse"),
        }
    }

    #[test]
    fn at_vision_04_validate_fails_for_non_visible_inferred_item() {
        let req = Ph1VisionRequest::VisibleContentValidate(
            VisionVisibleContentValidateRequest::v1(
                env(true, 4),
                source(),
                vec![token("visible_total"), token("visible_tax")],
                vec![
                    VisionEvidenceItem::v1("visible_total".to_string(), None).unwrap(),
                    VisionEvidenceItem::v1("inferred_profit".to_string(), None).unwrap(),
                ],
            )
            .unwrap(),
        );

        let resp = runtime().run(&req);
        match resp {
            Ph1VisionResponse::VisibleContentValidateOk(ok) => {
                assert_eq!(ok.validation_status, VisionValidationStatus::Fail);
                assert_eq!(ok.reason_code, reason_codes::PH1_VISION_VALIDATION_FAILED);
                assert!(ok
                    .diagnostics
                    .iter()
                    .any(|d| d == "evidence_index_1_not_visible_content"));
                assert!(ok.visible_content_only);
            }
            _ => panic!("expected VisibleContentValidateOk"),
        }
    }
}
