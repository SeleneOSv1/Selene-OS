#![forbid(unsafe_code)]

use std::cmp::min;
use std::collections::BTreeSet;

use selene_kernel_contracts::ph1kg::{
    KgCapabilityId, KgEntityCandidate, KgEntityLinkOk, KgEntityLinkRequest, KgEntityType,
    KgFactBundleSelectOk, KgFactBundleSelectRequest, KgFactCandidate, KgRefuse, KgRelationType,
    KgValidationStatus, Ph1KgRequest, Ph1KgResponse,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.KG reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_KG_OK_ENTITY_LINK: ReasonCodeId = ReasonCodeId(0x4B47_0001);
    pub const PH1_KG_OK_FACT_BUNDLE_SELECT: ReasonCodeId = ReasonCodeId(0x4B47_0002);

    pub const PH1_KG_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x4B47_00F1);
    pub const PH1_KG_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x4B47_00F2);
    pub const PH1_KG_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x4B47_00F3);
    pub const PH1_KG_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4B47_00F4);
    pub const PH1_KG_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4B47_00F5);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1KgConfig {
    pub max_entity_candidates: u8,
    pub max_fact_candidates: u8,
    pub max_diagnostics: u8,
}

impl Ph1KgConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_entity_candidates: 24,
            max_fact_candidates: 12,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1KgRuntime {
    config: Ph1KgConfig,
}

impl Ph1KgRuntime {
    pub fn new(config: Ph1KgConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1KgRequest) -> Ph1KgResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_KG_INPUT_SCHEMA_INVALID,
                "kg request failed contract validation",
            );
        }

        match req {
            Ph1KgRequest::KgEntityLink(r) => self.run_entity_link(r),
            Ph1KgRequest::KgFactBundleSelect(r) => self.run_fact_bundle_select(r),
        }
    }

    fn run_entity_link(&self, req: &KgEntityLinkRequest) -> Ph1KgResponse {
        if req.entity_candidates.is_empty() || req.relation_type_hints.is_empty() {
            return self.refuse(
                KgCapabilityId::KgEntityLink,
                reason_codes::PH1_KG_UPSTREAM_INPUT_MISSING,
                "entity_candidates or relation_type_hints is empty",
            );
        }

        let entity_budget = min(
            req.envelope.max_entity_candidates as usize,
            self.config.max_entity_candidates as usize,
        );
        if req.entity_candidates.len() > entity_budget {
            return self.refuse(
                KgCapabilityId::KgEntityLink,
                reason_codes::PH1_KG_BUDGET_EXCEEDED,
                "entity_candidates exceeds configured budget",
            );
        }

        let fact_budget = min(
            req.envelope.max_fact_candidates as usize,
            self.config.max_fact_candidates as usize,
        );
        if fact_budget == 0 {
            return self.refuse(
                KgCapabilityId::KgEntityLink,
                reason_codes::PH1_KG_BUDGET_EXCEEDED,
                "fact candidate budget exceeded",
            );
        }

        if req
            .entity_candidates
            .iter()
            .any(|entity| entity.tenant_id != req.tenant_id)
        {
            return self.refuse(
                KgCapabilityId::KgEntityLink,
                reason_codes::PH1_KG_VALIDATION_FAILED,
                "entity tenant_id mismatch detected",
            );
        }

        let mut facts = Vec::new();
        for relation in &req.relation_type_hints {
            for subject in &req.entity_candidates {
                for object in &req.entity_candidates {
                    if subject.candidate_id == object.candidate_id {
                        continue;
                    }
                    if !relation_supported(*relation, subject.entity_type, object.entity_type) {
                        continue;
                    }
                    let fact = KgFactCandidate::v1(
                        format!(
                            "fact:{}:{}:{}",
                            subject.candidate_id,
                            object.candidate_id,
                            relation_type_token(*relation)
                        ),
                        req.tenant_id.clone(),
                        *relation,
                        subject.candidate_id.clone(),
                        object.candidate_id.clone(),
                        fact_priority(subject, object, *relation),
                        subject.evidence_ref.clone(),
                    );
                    match fact {
                        Ok(f) => facts.push(f),
                        Err(_) => {
                            return self.refuse(
                                KgCapabilityId::KgEntityLink,
                                reason_codes::PH1_KG_INTERNAL_PIPELINE_ERROR,
                                "failed to construct fact candidate",
                            );
                        }
                    }
                }
            }
        }

        facts.sort_by(|a, b| {
            b.priority_bp
                .cmp(&a.priority_bp)
                .then(a.fact_id.cmp(&b.fact_id))
        });

        let mut deduped = Vec::new();
        let mut seen = BTreeSet::new();
        for fact in facts {
            if seen.insert(fact.fact_id.clone()) {
                deduped.push(fact);
            }
            if deduped.len() >= fact_budget {
                break;
            }
        }

        if deduped.is_empty() {
            return self.refuse(
                KgCapabilityId::KgEntityLink,
                reason_codes::PH1_KG_UPSTREAM_INPUT_MISSING,
                "no relation-compatible entity pairs found",
            );
        }

        let selected_fact_id = deduped[0].fact_id.clone();
        match KgEntityLinkOk::v1(
            reason_codes::PH1_KG_OK_ENTITY_LINK,
            selected_fact_id,
            deduped,
            true,
            true,
            true,
            true,
            true,
        ) {
            Ok(out) => Ph1KgResponse::KgEntityLinkOk(out),
            Err(_) => self.refuse(
                KgCapabilityId::KgEntityLink,
                reason_codes::PH1_KG_INTERNAL_PIPELINE_ERROR,
                "failed to construct entity-link output",
            ),
        }
    }

    fn run_fact_bundle_select(&self, req: &KgFactBundleSelectRequest) -> Ph1KgResponse {
        if req.ordered_fact_candidates.is_empty() {
            return self.refuse(
                KgCapabilityId::KgFactBundleSelect,
                reason_codes::PH1_KG_UPSTREAM_INPUT_MISSING,
                "ordered_fact_candidates is empty",
            );
        }

        let fact_budget = min(
            req.envelope.max_fact_candidates as usize,
            self.config.max_fact_candidates as usize,
        );
        if req.ordered_fact_candidates.len() > fact_budget {
            return self.refuse(
                KgCapabilityId::KgFactBundleSelect,
                reason_codes::PH1_KG_BUDGET_EXCEEDED,
                "ordered_fact_candidates exceeds configured budget",
            );
        }

        let mut diagnostics = Vec::new();
        if req.ordered_fact_candidates[0].fact_id != req.selected_fact_id {
            diagnostics.push("selected_not_first_in_ordered_fact_candidates".to_string());
        }

        if !req
            .ordered_fact_candidates
            .iter()
            .any(|fact| fact.fact_id == req.selected_fact_id)
        {
            diagnostics.push("selected_fact_missing_from_ordered_fact_candidates".to_string());
        }

        if req.ordered_fact_candidates.windows(2).any(|pair| {
            pair[0].priority_bp < pair[1].priority_bp
                || (pair[0].priority_bp == pair[1].priority_bp && pair[0].fact_id > pair[1].fact_id)
        }) {
            diagnostics.push("fact_priority_not_sorted_desc".to_string());
        }

        let mut seen = BTreeSet::new();
        if req
            .ordered_fact_candidates
            .iter()
            .any(|fact| !seen.insert(fact.fact_id.as_str()))
        {
            diagnostics.push("duplicate_fact_id".to_string());
        }

        let preserved_tenant_scope = req
            .ordered_fact_candidates
            .iter()
            .all(|fact| fact.tenant_id == req.tenant_id);
        if !preserved_tenant_scope {
            diagnostics.push("tenant_scope_mismatch_detected".to_string());
        }

        let preserved_evidence_refs = req
            .ordered_fact_candidates
            .iter()
            .all(|fact| !fact.evidence_ref.trim().is_empty());
        if !preserved_evidence_refs {
            diagnostics.push("missing_evidence_ref_in_fact_candidate".to_string());
        }

        if !req.no_guessing_required {
            diagnostics.push("no_guessing_required_not_set".to_string());
        }

        diagnostics.truncate(self.config.max_diagnostics as usize);
        let (validation_status, reason_code) = if diagnostics.is_empty() {
            (
                KgValidationStatus::Ok,
                reason_codes::PH1_KG_OK_FACT_BUNDLE_SELECT,
            )
        } else {
            (
                KgValidationStatus::Fail,
                reason_codes::PH1_KG_VALIDATION_FAILED,
            )
        };

        match KgFactBundleSelectOk::v1(
            reason_code,
            validation_status,
            diagnostics,
            preserved_tenant_scope,
            preserved_evidence_refs,
            req.no_guessing_required,
            true,
            true,
        ) {
            Ok(out) => Ph1KgResponse::KgFactBundleSelectOk(out),
            Err(_) => self.refuse(
                KgCapabilityId::KgFactBundleSelect,
                reason_codes::PH1_KG_INTERNAL_PIPELINE_ERROR,
                "failed to construct fact-bundle-select output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: KgCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1KgResponse {
        let out = KgRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("KgRefuse::v1 must construct for static message");
        Ph1KgResponse::Refuse(out)
    }
}

fn capability_from_request(req: &Ph1KgRequest) -> KgCapabilityId {
    match req {
        Ph1KgRequest::KgEntityLink(_) => KgCapabilityId::KgEntityLink,
        Ph1KgRequest::KgFactBundleSelect(_) => KgCapabilityId::KgFactBundleSelect,
    }
}

fn relation_supported(
    relation: KgRelationType,
    subject_type: KgEntityType,
    object_type: KgEntityType,
) -> bool {
    match relation {
        KgRelationType::PersonHasRole => {
            subject_type == KgEntityType::Person && object_type == KgEntityType::Role
        }
        KgRelationType::PersonInTeam => {
            subject_type == KgEntityType::Person && object_type == KgEntityType::Team
        }
        KgRelationType::PersonOnProject => {
            subject_type == KgEntityType::Person && object_type == KgEntityType::Project
        }
        KgRelationType::ProjectInDepartment => {
            subject_type == KgEntityType::Project && object_type == KgEntityType::Department
        }
    }
}

fn fact_priority(
    subject: &KgEntityCandidate,
    object: &KgEntityCandidate,
    relation: KgRelationType,
) -> i16 {
    let relation_weight = match relation {
        KgRelationType::PersonHasRole => 1100,
        KgRelationType::PersonInTeam => 1000,
        KgRelationType::PersonOnProject => 980,
        KgRelationType::ProjectInDepartment => 920,
    };
    let confidence_component =
        ((subject.confidence_bp as i32 + object.confidence_bp as i32) / 2) / 5;
    (relation_weight + confidence_component).clamp(-20_000, 20_000) as i16
}

fn relation_type_token(relation: KgRelationType) -> &'static str {
    match relation {
        KgRelationType::PersonHasRole => "person_has_role",
        KgRelationType::PersonInTeam => "person_in_team",
        KgRelationType::PersonOnProject => "person_on_project",
        KgRelationType::ProjectInDepartment => "project_in_department",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1kg::{KgRequestEnvelope, KgValidationStatus};

    fn runtime() -> Ph1KgRuntime {
        Ph1KgRuntime::new(Ph1KgConfig::mvp_v1())
    }

    fn envelope(max_entity_candidates: u8, max_fact_candidates: u8) -> KgRequestEnvelope {
        KgRequestEnvelope::v1(
            CorrelationId(3601),
            TurnId(331),
            max_entity_candidates,
            max_fact_candidates,
            8,
        )
        .unwrap()
    }

    fn entity(
        candidate_id: &str,
        entity_type: KgEntityType,
        confidence_bp: u16,
    ) -> KgEntityCandidate {
        KgEntityCandidate::v1(
            candidate_id.to_string(),
            "tenant_1".to_string(),
            entity_type,
            format!("key:{}", candidate_id),
            format!("Label {}", candidate_id),
            confidence_bp,
            format!("kg:evidence:{}", candidate_id),
        )
        .unwrap()
    }

    fn link_request() -> KgEntityLinkRequest {
        KgEntityLinkRequest::v1(
            envelope(8, 4),
            "tenant_1".to_string(),
            vec![
                entity("person_1", KgEntityType::Person, 8800),
                entity("role_1", KgEntityType::Role, 8600),
                entity("team_1", KgEntityType::Team, 8100),
            ],
            vec![KgRelationType::PersonHasRole, KgRelationType::PersonInTeam],
        )
        .unwrap()
    }

    #[test]
    fn at_kg_01_entity_link_output_is_schema_valid() {
        let req = Ph1KgRequest::KgEntityLink(link_request());

        let out = runtime().run(&req);
        assert!(out.validate().is_ok());
        match out {
            Ph1KgResponse::KgEntityLinkOk(ok) => {
                assert!(!ok.selected_fact_id.is_empty());
                assert!(!ok.ordered_fact_candidates.is_empty());
            }
            _ => panic!("expected KgEntityLinkOk"),
        }
    }

    #[test]
    fn at_kg_02_entity_link_order_is_deterministic() {
        let req = Ph1KgRequest::KgEntityLink(link_request());
        let runtime = runtime();

        let out_1 = runtime.run(&req);
        let out_2 = runtime.run(&req);

        match (out_1, out_2) {
            (Ph1KgResponse::KgEntityLinkOk(a), Ph1KgResponse::KgEntityLinkOk(b)) => {
                assert_eq!(a.selected_fact_id, b.selected_fact_id);
                assert_eq!(a.ordered_fact_candidates, b.ordered_fact_candidates);
            }
            _ => panic!("expected KgEntityLinkOk outputs"),
        }
    }

    #[test]
    fn at_kg_03_budget_bound_is_enforced() {
        let runtime = Ph1KgRuntime::new(Ph1KgConfig {
            max_entity_candidates: 2,
            max_fact_candidates: 2,
            max_diagnostics: 8,
        });
        let req = Ph1KgRequest::KgEntityLink(link_request());

        let out = runtime.run(&req);
        match out {
            Ph1KgResponse::Refuse(refuse) => {
                assert_eq!(refuse.reason_code, reason_codes::PH1_KG_BUDGET_EXCEEDED)
            }
            _ => panic!("expected Refuse"),
        }
    }

    #[test]
    fn at_kg_04_fact_bundle_select_fails_on_selection_drift() {
        let link_out = runtime().run(&Ph1KgRequest::KgEntityLink(link_request()));
        let link_ok = match link_out {
            Ph1KgResponse::KgEntityLinkOk(ok) => ok,
            _ => panic!("expected KgEntityLinkOk"),
        };
        assert!(link_ok.ordered_fact_candidates.len() >= 2);

        let drift_req = Ph1KgRequest::KgFactBundleSelect(
            KgFactBundleSelectRequest::v1(
                envelope(8, 4),
                "tenant_1".to_string(),
                link_ok.ordered_fact_candidates[1].fact_id.clone(),
                link_ok.ordered_fact_candidates,
                true,
                true,
                true,
            )
            .unwrap(),
        );
        let out = runtime().run(&drift_req);

        match out {
            Ph1KgResponse::KgFactBundleSelectOk(ok) => {
                assert_eq!(ok.validation_status, KgValidationStatus::Fail)
            }
            _ => panic!("expected KgFactBundleSelectOk"),
        }
    }
}
