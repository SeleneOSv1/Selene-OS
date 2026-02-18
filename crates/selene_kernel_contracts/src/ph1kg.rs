#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1KG_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KgCapabilityId {
    KgEntityLink,
    KgFactBundleSelect,
}

impl KgCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            KgCapabilityId::KgEntityLink => "KG_ENTITY_LINK",
            KgCapabilityId::KgFactBundleSelect => "KG_FACT_BUNDLE_SELECT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KgEntityType {
    Person,
    Role,
    Team,
    Project,
    Department,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KgRelationType {
    PersonHasRole,
    PersonInTeam,
    PersonOnProject,
    ProjectInDepartment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KgValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KgRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_entity_candidates: u8,
    pub max_fact_candidates: u8,
    pub max_diagnostics: u8,
}

impl KgRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_entity_candidates: u8,
        max_fact_candidates: u8,
        max_diagnostics: u8,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1KG_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_entity_candidates,
            max_fact_candidates,
            max_diagnostics,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for KgRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1KG_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "kg_request_envelope.schema_version",
                reason: "must match PH1KG_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_entity_candidates == 0 || self.max_entity_candidates > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "kg_request_envelope.max_entity_candidates",
                reason: "must be within 1..=64",
            });
        }
        if self.max_fact_candidates == 0 || self.max_fact_candidates > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "kg_request_envelope.max_fact_candidates",
                reason: "must be within 1..=32",
            });
        }
        if self.max_diagnostics == 0 || self.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "kg_request_envelope.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KgEntityCandidate {
    pub schema_version: SchemaVersion,
    pub candidate_id: String,
    pub tenant_id: String,
    pub entity_type: KgEntityType,
    pub entity_key: String,
    pub canonical_label: String,
    pub confidence_bp: u16,
    pub evidence_ref: String,
}

impl KgEntityCandidate {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        candidate_id: String,
        tenant_id: String,
        entity_type: KgEntityType,
        entity_key: String,
        canonical_label: String,
        confidence_bp: u16,
        evidence_ref: String,
    ) -> Result<Self, ContractViolation> {
        let candidate = Self {
            schema_version: PH1KG_CONTRACT_VERSION,
            candidate_id,
            tenant_id,
            entity_type,
            entity_key,
            canonical_label,
            confidence_bp,
            evidence_ref,
        };
        candidate.validate()?;
        Ok(candidate)
    }
}

impl Validate for KgEntityCandidate {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1KG_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "kg_entity_candidate.schema_version",
                reason: "must match PH1KG_CONTRACT_VERSION",
            });
        }
        validate_token("kg_entity_candidate.candidate_id", &self.candidate_id, 64)?;
        validate_token("kg_entity_candidate.tenant_id", &self.tenant_id, 64)?;
        validate_token("kg_entity_candidate.entity_key", &self.entity_key, 96)?;
        validate_text(
            "kg_entity_candidate.canonical_label",
            &self.canonical_label,
            96,
        )?;
        if self.confidence_bp > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "kg_entity_candidate.confidence_bp",
                reason: "must be <= 10000",
            });
        }
        validate_token("kg_entity_candidate.evidence_ref", &self.evidence_ref, 128)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KgFactCandidate {
    pub schema_version: SchemaVersion,
    pub fact_id: String,
    pub tenant_id: String,
    pub relation_type: KgRelationType,
    pub subject_candidate_id: String,
    pub object_candidate_id: String,
    pub priority_bp: i16,
    pub evidence_ref: String,
}

impl KgFactCandidate {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        fact_id: String,
        tenant_id: String,
        relation_type: KgRelationType,
        subject_candidate_id: String,
        object_candidate_id: String,
        priority_bp: i16,
        evidence_ref: String,
    ) -> Result<Self, ContractViolation> {
        let candidate = Self {
            schema_version: PH1KG_CONTRACT_VERSION,
            fact_id,
            tenant_id,
            relation_type,
            subject_candidate_id,
            object_candidate_id,
            priority_bp,
            evidence_ref,
        };
        candidate.validate()?;
        Ok(candidate)
    }
}

impl Validate for KgFactCandidate {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1KG_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "kg_fact_candidate.schema_version",
                reason: "must match PH1KG_CONTRACT_VERSION",
            });
        }
        validate_token("kg_fact_candidate.fact_id", &self.fact_id, 96)?;
        validate_token("kg_fact_candidate.tenant_id", &self.tenant_id, 64)?;
        validate_token(
            "kg_fact_candidate.subject_candidate_id",
            &self.subject_candidate_id,
            64,
        )?;
        validate_token(
            "kg_fact_candidate.object_candidate_id",
            &self.object_candidate_id,
            64,
        )?;
        if !(-20_000..=20_000).contains(&self.priority_bp) {
            return Err(ContractViolation::InvalidValue {
                field: "kg_fact_candidate.priority_bp",
                reason: "must be within -20000..=20000",
            });
        }
        validate_token("kg_fact_candidate.evidence_ref", &self.evidence_ref, 128)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KgEntityLinkRequest {
    pub schema_version: SchemaVersion,
    pub envelope: KgRequestEnvelope,
    pub tenant_id: String,
    pub entity_candidates: Vec<KgEntityCandidate>,
    pub relation_type_hints: Vec<KgRelationType>,
}

impl KgEntityLinkRequest {
    pub fn v1(
        envelope: KgRequestEnvelope,
        tenant_id: String,
        entity_candidates: Vec<KgEntityCandidate>,
        relation_type_hints: Vec<KgRelationType>,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1KG_CONTRACT_VERSION,
            envelope,
            tenant_id,
            entity_candidates,
            relation_type_hints,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for KgEntityLinkRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1KG_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "kg_entity_link_request.schema_version",
                reason: "must match PH1KG_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_token("kg_entity_link_request.tenant_id", &self.tenant_id, 64)?;
        if self.entity_candidates.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "kg_entity_link_request.entity_candidates",
                reason: "must be non-empty",
            });
        }
        if self.entity_candidates.len() > self.envelope.max_entity_candidates as usize {
            return Err(ContractViolation::InvalidValue {
                field: "kg_entity_link_request.entity_candidates",
                reason: "must be <= envelope.max_entity_candidates",
            });
        }
        if self.relation_type_hints.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "kg_entity_link_request.relation_type_hints",
                reason: "must be non-empty",
            });
        }
        if self.relation_type_hints.len() > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "kg_entity_link_request.relation_type_hints",
                reason: "must be <= 8",
            });
        }
        let mut seen_ids = BTreeSet::new();
        for entity in &self.entity_candidates {
            entity.validate()?;
            if entity.tenant_id != self.tenant_id {
                return Err(ContractViolation::InvalidValue {
                    field: "kg_entity_link_request.entity_candidates",
                    reason: "candidate tenant_id must match request tenant_id",
                });
            }
            if !seen_ids.insert(entity.candidate_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "kg_entity_link_request.entity_candidates",
                    reason: "candidate_id must be unique",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KgFactBundleSelectRequest {
    pub schema_version: SchemaVersion,
    pub envelope: KgRequestEnvelope,
    pub tenant_id: String,
    pub selected_fact_id: String,
    pub ordered_fact_candidates: Vec<KgFactCandidate>,
    pub tenant_scope_required: bool,
    pub evidence_required: bool,
    pub no_guessing_required: bool,
}

impl KgFactBundleSelectRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: KgRequestEnvelope,
        tenant_id: String,
        selected_fact_id: String,
        ordered_fact_candidates: Vec<KgFactCandidate>,
        tenant_scope_required: bool,
        evidence_required: bool,
        no_guessing_required: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1KG_CONTRACT_VERSION,
            envelope,
            tenant_id,
            selected_fact_id,
            ordered_fact_candidates,
            tenant_scope_required,
            evidence_required,
            no_guessing_required,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for KgFactBundleSelectRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1KG_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "kg_fact_bundle_select_request.schema_version",
                reason: "must match PH1KG_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_token(
            "kg_fact_bundle_select_request.tenant_id",
            &self.tenant_id,
            64,
        )?;
        validate_token(
            "kg_fact_bundle_select_request.selected_fact_id",
            &self.selected_fact_id,
            96,
        )?;
        if self.ordered_fact_candidates.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "kg_fact_bundle_select_request.ordered_fact_candidates",
                reason: "must be non-empty",
            });
        }
        if self.ordered_fact_candidates.len() > self.envelope.max_fact_candidates as usize {
            return Err(ContractViolation::InvalidValue {
                field: "kg_fact_bundle_select_request.ordered_fact_candidates",
                reason: "must be <= envelope.max_fact_candidates",
            });
        }

        if !self.tenant_scope_required {
            return Err(ContractViolation::InvalidValue {
                field: "kg_fact_bundle_select_request.tenant_scope_required",
                reason: "must be true",
            });
        }
        if !self.evidence_required {
            return Err(ContractViolation::InvalidValue {
                field: "kg_fact_bundle_select_request.evidence_required",
                reason: "must be true",
            });
        }
        if !self.no_guessing_required {
            return Err(ContractViolation::InvalidValue {
                field: "kg_fact_bundle_select_request.no_guessing_required",
                reason: "must be true",
            });
        }

        let mut seen = BTreeSet::new();
        let mut selected_present = false;
        for fact in &self.ordered_fact_candidates {
            fact.validate()?;
            if fact.tenant_id != self.tenant_id {
                return Err(ContractViolation::InvalidValue {
                    field: "kg_fact_bundle_select_request.ordered_fact_candidates",
                    reason: "fact tenant_id must match request tenant_id",
                });
            }
            if !seen.insert(fact.fact_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "kg_fact_bundle_select_request.ordered_fact_candidates",
                    reason: "fact_id must be unique",
                });
            }
            if fact.fact_id == self.selected_fact_id {
                selected_present = true;
            }
        }
        if !selected_present {
            return Err(ContractViolation::InvalidValue {
                field: "kg_fact_bundle_select_request.selected_fact_id",
                reason: "must exist in ordered_fact_candidates",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1KgRequest {
    KgEntityLink(KgEntityLinkRequest),
    KgFactBundleSelect(KgFactBundleSelectRequest),
}

impl Validate for Ph1KgRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1KgRequest::KgEntityLink(req) => req.validate(),
            Ph1KgRequest::KgFactBundleSelect(req) => req.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KgEntityLinkOk {
    pub schema_version: SchemaVersion,
    pub capability_id: KgCapabilityId,
    pub reason_code: ReasonCodeId,
    pub selected_fact_id: String,
    pub ordered_fact_candidates: Vec<KgFactCandidate>,
    pub tenant_scoped: bool,
    pub evidence_backed: bool,
    pub no_guessing: bool,
    pub advisory_only: bool,
    pub no_execution_authority: bool,
}

impl KgEntityLinkOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        selected_fact_id: String,
        ordered_fact_candidates: Vec<KgFactCandidate>,
        tenant_scoped: bool,
        evidence_backed: bool,
        no_guessing: bool,
        advisory_only: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1KG_CONTRACT_VERSION,
            capability_id: KgCapabilityId::KgEntityLink,
            reason_code,
            selected_fact_id,
            ordered_fact_candidates,
            tenant_scoped,
            evidence_backed,
            no_guessing,
            advisory_only,
            no_execution_authority,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for KgEntityLinkOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1KG_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "kg_entity_link_ok.schema_version",
                reason: "must match PH1KG_CONTRACT_VERSION",
            });
        }
        if self.capability_id != KgCapabilityId::KgEntityLink {
            return Err(ContractViolation::InvalidValue {
                field: "kg_entity_link_ok.capability_id",
                reason: "must be KG_ENTITY_LINK",
            });
        }
        validate_token(
            "kg_entity_link_ok.selected_fact_id",
            &self.selected_fact_id,
            96,
        )?;
        if self.ordered_fact_candidates.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "kg_entity_link_ok.ordered_fact_candidates",
                reason: "must be non-empty",
            });
        }
        if self.ordered_fact_candidates.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "kg_entity_link_ok.ordered_fact_candidates",
                reason: "must be <= 32",
            });
        }
        let mut seen = BTreeSet::new();
        let mut selected_present = false;
        for fact in &self.ordered_fact_candidates {
            fact.validate()?;
            if !seen.insert(fact.fact_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "kg_entity_link_ok.ordered_fact_candidates",
                    reason: "fact_id must be unique",
                });
            }
            if fact.fact_id == self.selected_fact_id {
                selected_present = true;
            }
        }
        if !selected_present {
            return Err(ContractViolation::InvalidValue {
                field: "kg_entity_link_ok.selected_fact_id",
                reason: "must exist in ordered_fact_candidates",
            });
        }
        if !self.tenant_scoped {
            return Err(ContractViolation::InvalidValue {
                field: "kg_entity_link_ok.tenant_scoped",
                reason: "must be true",
            });
        }
        if !self.evidence_backed {
            return Err(ContractViolation::InvalidValue {
                field: "kg_entity_link_ok.evidence_backed",
                reason: "must be true",
            });
        }
        if !self.no_guessing {
            return Err(ContractViolation::InvalidValue {
                field: "kg_entity_link_ok.no_guessing",
                reason: "must be true",
            });
        }
        if !self.advisory_only {
            return Err(ContractViolation::InvalidValue {
                field: "kg_entity_link_ok.advisory_only",
                reason: "must be true",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "kg_entity_link_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KgFactBundleSelectOk {
    pub schema_version: SchemaVersion,
    pub capability_id: KgCapabilityId,
    pub reason_code: ReasonCodeId,
    pub validation_status: KgValidationStatus,
    pub diagnostics: Vec<String>,
    pub preserved_tenant_scope: bool,
    pub preserved_evidence_refs: bool,
    pub no_guessing_confirmed: bool,
    pub advisory_only: bool,
    pub no_execution_authority: bool,
}

impl KgFactBundleSelectOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        validation_status: KgValidationStatus,
        diagnostics: Vec<String>,
        preserved_tenant_scope: bool,
        preserved_evidence_refs: bool,
        no_guessing_confirmed: bool,
        advisory_only: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1KG_CONTRACT_VERSION,
            capability_id: KgCapabilityId::KgFactBundleSelect,
            reason_code,
            validation_status,
            diagnostics,
            preserved_tenant_scope,
            preserved_evidence_refs,
            no_guessing_confirmed,
            advisory_only,
            no_execution_authority,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for KgFactBundleSelectOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1KG_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "kg_fact_bundle_select_ok.schema_version",
                reason: "must match PH1KG_CONTRACT_VERSION",
            });
        }
        if self.capability_id != KgCapabilityId::KgFactBundleSelect {
            return Err(ContractViolation::InvalidValue {
                field: "kg_fact_bundle_select_ok.capability_id",
                reason: "must be KG_FACT_BUNDLE_SELECT",
            });
        }
        if self.diagnostics.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "kg_fact_bundle_select_ok.diagnostics",
                reason: "must be <= 16",
            });
        }
        for diagnostic in &self.diagnostics {
            validate_token("kg_fact_bundle_select_ok.diagnostics", diagnostic, 96)?;
        }
        if self.validation_status == KgValidationStatus::Fail && self.diagnostics.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "kg_fact_bundle_select_ok.diagnostics",
                reason: "must be non-empty when validation_status=FAIL",
            });
        }
        if self.validation_status == KgValidationStatus::Ok
            && (!self.preserved_tenant_scope
                || !self.preserved_evidence_refs
                || !self.no_guessing_confirmed)
        {
            return Err(ContractViolation::InvalidValue {
                field: "kg_fact_bundle_select_ok",
                reason: "OK status requires tenant scope, evidence refs, and no-guessing preserved",
            });
        }
        if !self.advisory_only {
            return Err(ContractViolation::InvalidValue {
                field: "kg_fact_bundle_select_ok.advisory_only",
                reason: "must be true",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "kg_fact_bundle_select_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KgRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: KgCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl KgRefuse {
    pub fn v1(
        capability_id: KgCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1KG_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for KgRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1KG_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "kg_refuse.schema_version",
                reason: "must match PH1KG_CONTRACT_VERSION",
            });
        }
        validate_text("kg_refuse.message", &self.message, 192)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1KgResponse {
    KgEntityLinkOk(KgEntityLinkOk),
    KgFactBundleSelectOk(KgFactBundleSelectOk),
    Refuse(KgRefuse),
}

impl Validate for Ph1KgResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1KgResponse::KgEntityLinkOk(out) => out.validate(),
            Ph1KgResponse::KgFactBundleSelectOk(out) => out.validate(),
            Ph1KgResponse::Refuse(out) => out.validate(),
        }
    }
}

fn validate_token(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if value.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be non-empty",
        });
    }
    if value.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    if value.chars().any(|c| {
        !(c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ':' || c == '.' || c == '/')
    }) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must contain token-safe ASCII only",
        });
    }
    Ok(())
}

fn validate_text(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if value.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be non-empty",
        });
    }
    if value.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    if value.chars().any(|c| c.is_control()) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not contain control characters",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope() -> KgRequestEnvelope {
        KgRequestEnvelope::v1(CorrelationId(9501), TurnId(541), 8, 4, 6).unwrap()
    }

    fn entity(
        candidate_id: &str,
        tenant_id: &str,
        entity_type: KgEntityType,
        confidence_bp: u16,
    ) -> KgEntityCandidate {
        KgEntityCandidate::v1(
            candidate_id.to_string(),
            tenant_id.to_string(),
            entity_type,
            format!("key:{}", candidate_id),
            format!("Label {}", candidate_id),
            confidence_bp,
            format!("kg:evidence:{}", candidate_id),
        )
        .unwrap()
    }

    fn fact(fact_id: &str) -> KgFactCandidate {
        KgFactCandidate::v1(
            fact_id.to_string(),
            "tenant_1".to_string(),
            KgRelationType::PersonHasRole,
            "person_1".to_string(),
            "role_1".to_string(),
            1200,
            format!("kg:evidence:{}", fact_id),
        )
        .unwrap()
    }

    #[test]
    fn kg_contract_01_entity_link_request_is_schema_valid() {
        let req = KgEntityLinkRequest::v1(
            envelope(),
            "tenant_1".to_string(),
            vec![
                entity("person_1", "tenant_1", KgEntityType::Person, 8500),
                entity("role_1", "tenant_1", KgEntityType::Role, 8300),
            ],
            vec![KgRelationType::PersonHasRole],
        )
        .unwrap();
        assert!(req.validate().is_ok());
    }

    #[test]
    fn kg_contract_02_entity_link_request_rejects_tenant_mismatch() {
        let req = KgEntityLinkRequest::v1(
            envelope(),
            "tenant_1".to_string(),
            vec![entity("person_1", "tenant_2", KgEntityType::Person, 8500)],
            vec![KgRelationType::PersonHasRole],
        );
        assert!(req.is_err());
    }

    #[test]
    fn kg_contract_03_fact_bundle_select_requires_selected_fact_present() {
        let req = KgFactBundleSelectRequest::v1(
            envelope(),
            "tenant_1".to_string(),
            "fact_missing".to_string(),
            vec![fact("fact_1")],
            true,
            true,
            true,
        );
        assert!(req.is_err());
    }

    #[test]
    fn kg_contract_04_select_ok_fail_requires_diagnostics() {
        let out = KgFactBundleSelectOk::v1(
            ReasonCodeId(1),
            KgValidationStatus::Fail,
            vec![],
            false,
            false,
            false,
            true,
            true,
        );
        assert!(out.is_err());
    }
}
