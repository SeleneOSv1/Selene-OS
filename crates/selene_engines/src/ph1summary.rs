#![forbid(unsafe_code)]

use std::cmp::min;
use std::collections::{BTreeMap, BTreeSet};

use selene_kernel_contracts::ph1summary::{
    Ph1SummaryRequest, Ph1SummaryResponse, SummaryBuildOk, SummaryBuildRequest, SummaryBullet,
    SummaryCapabilityId, SummaryCitationValidateOk, SummaryCitationValidateRequest,
    SummaryEvidenceId, SummaryRefuse, SummaryValidationStatus,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.SUMMARY reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_SUMMARY_OK_BUILD: ReasonCodeId = ReasonCodeId(0x5355_0001);
    pub const PH1_SUMMARY_OK_CITATION_VALIDATE: ReasonCodeId = ReasonCodeId(0x5355_0002);

    pub const PH1_SUMMARY_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x5355_00F1);
    pub const PH1_SUMMARY_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x5355_00F2);
    pub const PH1_SUMMARY_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x5355_00F3);
    pub const PH1_SUMMARY_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5355_00F4);
    pub const PH1_SUMMARY_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x5355_00F5);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1SummaryConfig {
    pub max_input_evidence_items: usize,
    pub max_output_bullets: u8,
    pub max_diagnostics: u8,
}

impl Ph1SummaryConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_input_evidence_items: 64,
            max_output_bullets: 8,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1SummaryRuntime {
    config: Ph1SummaryConfig,
}

impl Ph1SummaryRuntime {
    pub fn new(config: Ph1SummaryConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1SummaryRequest) -> Ph1SummaryResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_SUMMARY_INPUT_SCHEMA_INVALID,
                "summary request failed contract validation",
            );
        }

        match req {
            Ph1SummaryRequest::SummaryBuild(r) => self.run_build(r),
            Ph1SummaryRequest::SummaryCitationValidate(r) => self.run_validate(r),
        }
    }

    fn run_build(&self, req: &SummaryBuildRequest) -> Ph1SummaryResponse {
        if req.evidence_items.is_empty() {
            return self.refuse(
                SummaryCapabilityId::SummaryBuild,
                reason_codes::PH1_SUMMARY_UPSTREAM_INPUT_MISSING,
                "no evidence items were provided",
            );
        }

        if req.evidence_items.len() > self.config.max_input_evidence_items {
            return self.refuse(
                SummaryCapabilityId::SummaryBuild,
                reason_codes::PH1_SUMMARY_BUDGET_EXCEEDED,
                "evidence item budget exceeded",
            );
        }

        let bullet_budget = min(
            req.envelope.max_summary_bullets,
            self.config.max_output_bullets,
        ) as usize;

        let mut seen: BTreeSet<String> = BTreeSet::new();
        let mut bullets: Vec<SummaryBullet> = Vec::new();

        for item in &req.evidence_items {
            if bullets.len() >= bullet_budget {
                break;
            }
            let canonical = canonical_text(&item.text);
            if canonical.is_empty() {
                continue;
            }
            if !seen.insert(canonical) {
                continue;
            }

            let bullet_text = truncate_to_char_boundary(item.text.trim(), 240);
            match SummaryBullet::v1(bullet_text, vec![item.evidence_id.clone()]) {
                Ok(b) => bullets.push(b),
                Err(_) => {
                    return self.refuse(
                        SummaryCapabilityId::SummaryBuild,
                        reason_codes::PH1_SUMMARY_INTERNAL_PIPELINE_ERROR,
                        "failed to build summary bullet",
                    )
                }
            }
        }

        if bullets.is_empty() {
            return self.refuse(
                SummaryCapabilityId::SummaryBuild,
                reason_codes::PH1_SUMMARY_UPSTREAM_INPUT_MISSING,
                "no summary bullet could be produced from evidence",
            );
        }

        match SummaryBuildOk::v1(reason_codes::PH1_SUMMARY_OK_BUILD, bullets, true) {
            Ok(ok) => Ph1SummaryResponse::SummaryBuildOk(ok),
            Err(_) => self.refuse(
                SummaryCapabilityId::SummaryBuild,
                reason_codes::PH1_SUMMARY_INTERNAL_PIPELINE_ERROR,
                "failed to construct summary build output",
            ),
        }
    }

    fn run_validate(&self, req: &SummaryCitationValidateRequest) -> Ph1SummaryResponse {
        if req.evidence_items.is_empty() {
            return self.refuse(
                SummaryCapabilityId::SummaryCitationValidate,
                reason_codes::PH1_SUMMARY_UPSTREAM_INPUT_MISSING,
                "no evidence items were provided",
            );
        }

        if req.evidence_items.len() > self.config.max_input_evidence_items {
            return self.refuse(
                SummaryCapabilityId::SummaryCitationValidate,
                reason_codes::PH1_SUMMARY_BUDGET_EXCEEDED,
                "evidence item budget exceeded",
            );
        }

        if req.summary_bullets.len() > self.config.max_output_bullets as usize {
            return self.refuse(
                SummaryCapabilityId::SummaryCitationValidate,
                reason_codes::PH1_SUMMARY_BUDGET_EXCEEDED,
                "summary bullet budget exceeded",
            );
        }

        let evidence_by_id: BTreeMap<SummaryEvidenceId, String> = req
            .evidence_items
            .iter()
            .map(|item| (item.evidence_id.clone(), canonical_text(&item.text)))
            .collect();

        let mut diagnostics: Vec<String> = Vec::new();

        for (idx, bullet) in req.summary_bullets.iter().enumerate() {
            let mut matched = false;
            let canonical_bullet = canonical_text(&bullet.text);

            for cited in &bullet.cited_evidence_ids {
                let Some(canonical_evidence) = evidence_by_id.get(cited) else {
                    diagnostics.push(format!("bullet_{idx}_missing_cited_evidence_id"));
                    continue;
                };
                if !canonical_bullet.is_empty() && canonical_evidence.contains(&canonical_bullet) {
                    matched = true;
                }
            }

            if !matched {
                diagnostics.push(format!("bullet_{idx}_not_evidence_backed"));
            }

            if diagnostics.len() >= self.config.max_diagnostics as usize {
                break;
            }
        }

        let (status, reason_code) = if diagnostics.is_empty() {
            (
                SummaryValidationStatus::Ok,
                reason_codes::PH1_SUMMARY_OK_CITATION_VALIDATE,
            )
        } else {
            (
                SummaryValidationStatus::Fail,
                reason_codes::PH1_SUMMARY_VALIDATION_FAILED,
            )
        };

        match SummaryCitationValidateOk::v1(reason_code, status, diagnostics, true) {
            Ok(ok) => Ph1SummaryResponse::SummaryCitationValidateOk(ok),
            Err(_) => self.refuse(
                SummaryCapabilityId::SummaryCitationValidate,
                reason_codes::PH1_SUMMARY_INTERNAL_PIPELINE_ERROR,
                "failed to construct summary citation validation output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: SummaryCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1SummaryResponse {
        let r = SummaryRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("SummaryRefuse::v1 must construct for static message");
        Ph1SummaryResponse::Refuse(r)
    }
}

fn capability_from_request(req: &Ph1SummaryRequest) -> SummaryCapabilityId {
    match req {
        Ph1SummaryRequest::SummaryBuild(_) => SummaryCapabilityId::SummaryBuild,
        Ph1SummaryRequest::SummaryCitationValidate(_) => {
            SummaryCapabilityId::SummaryCitationValidate
        }
    }
}

fn canonical_text(input: &str) -> String {
    input
        .trim()
        .to_ascii_lowercase()
        .split_whitespace()
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn truncate_to_char_boundary(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }
    input.chars().take(max_chars).collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1summary::{
        SummaryBuildRequest, SummaryCitationValidateRequest, SummaryEvidenceId,
        SummaryEvidenceItem, SummaryRequestEnvelope,
    };

    fn runtime() -> Ph1SummaryRuntime {
        Ph1SummaryRuntime::new(Ph1SummaryConfig::mvp_v1())
    }

    fn envelope(max_bullets: u8) -> SummaryRequestEnvelope {
        SummaryRequestEnvelope::v1(CorrelationId(501), TurnId(9), max_bullets).unwrap()
    }

    fn evidence(id: &str, text: &str) -> SummaryEvidenceItem {
        SummaryEvidenceItem::v1(SummaryEvidenceId::new(id).unwrap(), text.to_string()).unwrap()
    }

    #[test]
    fn at_summary_01_build_output_is_schema_valid() {
        let req = Ph1SummaryRequest::SummaryBuild(
            SummaryBuildRequest::v1(
                envelope(4),
                vec![
                    evidence("e1", "Budget variance remained below threshold."),
                    evidence("e2", "No policy breaches were detected."),
                ],
            )
            .unwrap(),
        );

        let resp = runtime().run(&req);
        assert!(resp.validate().is_ok());
        match resp {
            Ph1SummaryResponse::SummaryBuildOk(ok) => {
                assert_eq!(ok.capability_id, SummaryCapabilityId::SummaryBuild);
                assert!(ok.evidence_backed_only);
                assert_eq!(ok.summary_bullets.len(), 2);
            }
            _ => panic!("expected SummaryBuildOk"),
        }
    }

    #[test]
    fn at_summary_02_build_order_is_deterministic() {
        let req = Ph1SummaryRequest::SummaryBuild(
            SummaryBuildRequest::v1(
                envelope(8),
                vec![
                    evidence("e1", "First item"),
                    evidence("e2", "Second item"),
                    evidence("e3", "First item"),
                    evidence("e4", "Third item"),
                ],
            )
            .unwrap(),
        );

        let runtime = runtime();
        let out1 = runtime.run(&req);
        let out2 = runtime.run(&req);

        let list1 = match out1 {
            Ph1SummaryResponse::SummaryBuildOk(ok) => ok
                .summary_bullets
                .into_iter()
                .map(|b| b.text)
                .collect::<Vec<_>>(),
            _ => panic!("expected SummaryBuildOk"),
        };
        let list2 = match out2 {
            Ph1SummaryResponse::SummaryBuildOk(ok) => ok
                .summary_bullets
                .into_iter()
                .map(|b| b.text)
                .collect::<Vec<_>>(),
            _ => panic!("expected SummaryBuildOk"),
        };

        assert_eq!(list1, vec!["First item", "Second item", "Third item"]);
        assert_eq!(list1, list2);
    }

    #[test]
    fn at_summary_03_output_is_bounded_by_budget() {
        let req = Ph1SummaryRequest::SummaryBuild(
            SummaryBuildRequest::v1(
                envelope(2),
                vec![
                    evidence("e1", "One"),
                    evidence("e2", "Two"),
                    evidence("e3", "Three"),
                ],
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        match out {
            Ph1SummaryResponse::SummaryBuildOk(ok) => {
                assert_eq!(ok.summary_bullets.len(), 2);
            }
            _ => panic!("expected SummaryBuildOk"),
        }
    }

    #[test]
    fn at_summary_04_validate_fails_for_non_evidence_backed_bullet() {
        let build = runtime().run(&Ph1SummaryRequest::SummaryBuild(
            SummaryBuildRequest::v1(
                envelope(4),
                vec![
                    evidence("e1", "Revenue remains stable."),
                    evidence("e2", "No payout changes."),
                ],
            )
            .unwrap(),
        ));

        let mut bullets = match build {
            Ph1SummaryResponse::SummaryBuildOk(ok) => ok.summary_bullets,
            _ => panic!("expected SummaryBuildOk"),
        };

        bullets.push(
            SummaryBullet::v1(
                "Inferred hidden risk spike".to_string(),
                vec![SummaryEvidenceId::new("e1").unwrap()],
            )
            .unwrap(),
        );

        let validate_req = Ph1SummaryRequest::SummaryCitationValidate(
            SummaryCitationValidateRequest::v1(
                envelope(8),
                vec![
                    evidence("e1", "Revenue remains stable."),
                    evidence("e2", "No payout changes."),
                ],
                bullets,
            )
            .unwrap(),
        );

        let out = runtime().run(&validate_req);
        match out {
            Ph1SummaryResponse::SummaryCitationValidateOk(ok) => {
                assert_eq!(ok.validation_status, SummaryValidationStatus::Fail);
                assert_eq!(ok.reason_code, reason_codes::PH1_SUMMARY_VALIDATION_FAILED);
                assert!(ok
                    .diagnostics
                    .iter()
                    .any(|d| d == "bullet_2_not_evidence_backed"));
            }
            _ => panic!("expected SummaryCitationValidateOk"),
        }
    }
}
