#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1KNOW_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KnowCapabilityId {
    KnowDictionaryPackBuild,
    KnowHintBundleSelect,
}

impl KnowCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            KnowCapabilityId::KnowDictionaryPackBuild => "KNOW_DICTIONARY_PACK_BUILD",
            KnowCapabilityId::KnowHintBundleSelect => "KNOW_HINT_BUNDLE_SELECT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KnowEntryKind {
    EmployeeNamePreferred,
    ApprovedAbbreviation,
    InternalProductName,
    ProjectCode,
    PronunciationHint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KnowSourceKind {
    HrOrgAuthorized,
    UserProvidedConsent,
    LearnArtifact,
    Unverified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KnowTargetEngine {
    C,
    Srl,
    Nlp,
    Tts,
}

impl KnowTargetEngine {
    pub fn as_str(self) -> &'static str {
        match self {
            KnowTargetEngine::C => "PH1.C",
            KnowTargetEngine::Srl => "PH1.SRL",
            KnowTargetEngine::Nlp => "PH1.NLP",
            KnowTargetEngine::Tts => "PH1.TTS",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KnowValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_entries: u8,
    pub max_diagnostics: u8,
}

impl KnowRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_entries: u8,
        max_diagnostics: u8,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1KNOW_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_entries,
            max_diagnostics,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for KnowRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1KNOW_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "know_request_envelope.schema_version",
                reason: "must match PH1KNOW_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_entries == 0 || self.max_entries > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "know_request_envelope.max_entries",
                reason: "must be within 1..=128",
            });
        }
        if self.max_diagnostics == 0 || self.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "know_request_envelope.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowDictionaryEntry {
    pub schema_version: SchemaVersion,
    pub entry_id: String,
    pub tenant_id: String,
    pub entry_kind: KnowEntryKind,
    pub source_kind: KnowSourceKind,
    pub canonical_term: String,
    pub normalized_term: String,
    pub locale_tag: String,
    pub pronunciation_hint: Option<String>,
    pub evidence_ref: String,
}

impl KnowDictionaryEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        entry_id: String,
        tenant_id: String,
        entry_kind: KnowEntryKind,
        source_kind: KnowSourceKind,
        canonical_term: String,
        normalized_term: String,
        locale_tag: String,
        pronunciation_hint: Option<String>,
        evidence_ref: String,
    ) -> Result<Self, ContractViolation> {
        let entry = Self {
            schema_version: PH1KNOW_CONTRACT_VERSION,
            entry_id,
            tenant_id,
            entry_kind,
            source_kind,
            canonical_term,
            normalized_term,
            locale_tag,
            pronunciation_hint,
            evidence_ref,
        };
        entry.validate()?;
        Ok(entry)
    }
}

impl Validate for KnowDictionaryEntry {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1KNOW_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "know_dictionary_entry.schema_version",
                reason: "must match PH1KNOW_CONTRACT_VERSION",
            });
        }
        validate_token("know_dictionary_entry.entry_id", &self.entry_id, 96)?;
        validate_token("know_dictionary_entry.tenant_id", &self.tenant_id, 64)?;
        validate_text(
            "know_dictionary_entry.canonical_term",
            &self.canonical_term,
            120,
        )?;
        validate_text(
            "know_dictionary_entry.normalized_term",
            &self.normalized_term,
            120,
        )?;
        validate_locale_tag("know_dictionary_entry.locale_tag", &self.locale_tag)?;
        if let Some(pronunciation_hint) = &self.pronunciation_hint {
            validate_text(
                "know_dictionary_entry.pronunciation_hint",
                pronunciation_hint,
                120,
            )?;
        }
        if self.entry_kind == KnowEntryKind::PronunciationHint && self.pronunciation_hint.is_none()
        {
            return Err(ContractViolation::InvalidValue {
                field: "know_dictionary_entry.pronunciation_hint",
                reason: "must be present when entry_kind=PRONUNCIATION_HINT",
            });
        }
        validate_token(
            "know_dictionary_entry.evidence_ref",
            &self.evidence_ref,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowDictionaryPackBuildRequest {
    pub schema_version: SchemaVersion,
    pub envelope: KnowRequestEnvelope,
    pub tenant_id: String,
    pub entries: Vec<KnowDictionaryEntry>,
    pub user_terms_present: bool,
    pub user_consent_asserted: bool,
    pub hr_org_authorized: bool,
    pub learn_artifact_ref: Option<String>,
    pub tenant_scope_required: bool,
    pub authorized_only_required: bool,
    pub no_cross_tenant_required: bool,
}

impl KnowDictionaryPackBuildRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: KnowRequestEnvelope,
        tenant_id: String,
        entries: Vec<KnowDictionaryEntry>,
        user_terms_present: bool,
        user_consent_asserted: bool,
        hr_org_authorized: bool,
        learn_artifact_ref: Option<String>,
        tenant_scope_required: bool,
        authorized_only_required: bool,
        no_cross_tenant_required: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1KNOW_CONTRACT_VERSION,
            envelope,
            tenant_id,
            entries,
            user_terms_present,
            user_consent_asserted,
            hr_org_authorized,
            learn_artifact_ref,
            tenant_scope_required,
            authorized_only_required,
            no_cross_tenant_required,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for KnowDictionaryPackBuildRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1KNOW_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "know_dictionary_pack_build_request.schema_version",
                reason: "must match PH1KNOW_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_token(
            "know_dictionary_pack_build_request.tenant_id",
            &self.tenant_id,
            64,
        )?;
        if self.entries.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "know_dictionary_pack_build_request.entries",
                reason: "must be non-empty",
            });
        }
        if self.entries.len() > self.envelope.max_entries as usize {
            return Err(ContractViolation::InvalidValue {
                field: "know_dictionary_pack_build_request.entries",
                reason: "must be <= envelope.max_entries",
            });
        }
        if !self.tenant_scope_required {
            return Err(ContractViolation::InvalidValue {
                field: "know_dictionary_pack_build_request.tenant_scope_required",
                reason: "must be true",
            });
        }
        if !self.authorized_only_required {
            return Err(ContractViolation::InvalidValue {
                field: "know_dictionary_pack_build_request.authorized_only_required",
                reason: "must be true",
            });
        }
        if !self.no_cross_tenant_required {
            return Err(ContractViolation::InvalidValue {
                field: "know_dictionary_pack_build_request.no_cross_tenant_required",
                reason: "must be true",
            });
        }
        if let Some(learn_artifact_ref) = &self.learn_artifact_ref {
            validate_token(
                "know_dictionary_pack_build_request.learn_artifact_ref",
                learn_artifact_ref,
                128,
            )?;
        }

        let mut has_user_source = false;
        let mut has_hr_source = false;
        let mut seen_ids = BTreeSet::new();
        for entry in &self.entries {
            entry.validate()?;
            if entry.tenant_id != self.tenant_id {
                return Err(ContractViolation::InvalidValue {
                    field: "know_dictionary_pack_build_request.entries",
                    reason: "entry tenant_id must match request tenant_id",
                });
            }
            if !seen_ids.insert(entry.entry_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "know_dictionary_pack_build_request.entries",
                    reason: "entry_id must be unique",
                });
            }
            match entry.source_kind {
                KnowSourceKind::UserProvidedConsent => has_user_source = true,
                KnowSourceKind::HrOrgAuthorized => has_hr_source = true,
                _ => {}
            }
        }

        if self.user_terms_present && !has_user_source {
            return Err(ContractViolation::InvalidValue {
                field: "know_dictionary_pack_build_request.user_terms_present",
                reason: "true requires at least one USER_PROVIDED_CONSENT source",
            });
        }
        if has_user_source && !self.user_terms_present {
            return Err(ContractViolation::InvalidValue {
                field: "know_dictionary_pack_build_request.user_terms_present",
                reason: "must be true when USER_PROVIDED_CONSENT entries are present",
            });
        }
        if has_user_source && !self.user_consent_asserted {
            return Err(ContractViolation::InvalidValue {
                field: "know_dictionary_pack_build_request.user_consent_asserted",
                reason: "must be true when USER_PROVIDED_CONSENT entries are present",
            });
        }
        if has_hr_source && !self.hr_org_authorized {
            return Err(ContractViolation::InvalidValue {
                field: "know_dictionary_pack_build_request.hr_org_authorized",
                reason: "must be true when HR_ORG_AUTHORIZED entries are present",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowHintBundleSelectRequest {
    pub schema_version: SchemaVersion,
    pub envelope: KnowRequestEnvelope,
    pub tenant_id: String,
    pub pack_id: String,
    pub ordered_entries: Vec<KnowDictionaryEntry>,
    pub target_engines: Vec<KnowTargetEngine>,
    pub tenant_scope_required: bool,
    pub authorized_only_required: bool,
    pub no_cross_tenant_required: bool,
}

impl KnowHintBundleSelectRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: KnowRequestEnvelope,
        tenant_id: String,
        pack_id: String,
        ordered_entries: Vec<KnowDictionaryEntry>,
        target_engines: Vec<KnowTargetEngine>,
        tenant_scope_required: bool,
        authorized_only_required: bool,
        no_cross_tenant_required: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1KNOW_CONTRACT_VERSION,
            envelope,
            tenant_id,
            pack_id,
            ordered_entries,
            target_engines,
            tenant_scope_required,
            authorized_only_required,
            no_cross_tenant_required,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for KnowHintBundleSelectRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1KNOW_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "know_hint_bundle_select_request.schema_version",
                reason: "must match PH1KNOW_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_token(
            "know_hint_bundle_select_request.tenant_id",
            &self.tenant_id,
            64,
        )?;
        validate_token(
            "know_hint_bundle_select_request.pack_id",
            &self.pack_id,
            128,
        )?;
        if self.ordered_entries.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "know_hint_bundle_select_request.ordered_entries",
                reason: "must be non-empty",
            });
        }
        if self.ordered_entries.len() > self.envelope.max_entries as usize {
            return Err(ContractViolation::InvalidValue {
                field: "know_hint_bundle_select_request.ordered_entries",
                reason: "must be <= envelope.max_entries",
            });
        }
        if self.target_engines.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "know_hint_bundle_select_request.target_engines",
                reason: "must be non-empty",
            });
        }
        if self.target_engines.len() > 4 {
            return Err(ContractViolation::InvalidValue {
                field: "know_hint_bundle_select_request.target_engines",
                reason: "must be <= 4",
            });
        }
        if !self.tenant_scope_required {
            return Err(ContractViolation::InvalidValue {
                field: "know_hint_bundle_select_request.tenant_scope_required",
                reason: "must be true",
            });
        }
        if !self.authorized_only_required {
            return Err(ContractViolation::InvalidValue {
                field: "know_hint_bundle_select_request.authorized_only_required",
                reason: "must be true",
            });
        }
        if !self.no_cross_tenant_required {
            return Err(ContractViolation::InvalidValue {
                field: "know_hint_bundle_select_request.no_cross_tenant_required",
                reason: "must be true",
            });
        }

        let mut seen_entry_ids = BTreeSet::new();
        for entry in &self.ordered_entries {
            entry.validate()?;
            if entry.tenant_id != self.tenant_id {
                return Err(ContractViolation::InvalidValue {
                    field: "know_hint_bundle_select_request.ordered_entries",
                    reason: "entry tenant_id must match request tenant_id",
                });
            }
            if !seen_entry_ids.insert(entry.entry_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "know_hint_bundle_select_request.ordered_entries",
                    reason: "entry_id must be unique",
                });
            }
        }

        let mut seen_targets = BTreeSet::new();
        for target in &self.target_engines {
            if !seen_targets.insert(target.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "know_hint_bundle_select_request.target_engines",
                    reason: "must be unique",
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1KnowRequest {
    KnowDictionaryPackBuild(KnowDictionaryPackBuildRequest),
    KnowHintBundleSelect(KnowHintBundleSelectRequest),
}

impl Validate for Ph1KnowRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1KnowRequest::KnowDictionaryPackBuild(req) => req.validate(),
            Ph1KnowRequest::KnowHintBundleSelect(req) => req.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowDictionaryPackBuildOk {
    pub schema_version: SchemaVersion,
    pub capability_id: KnowCapabilityId,
    pub reason_code: ReasonCodeId,
    pub pack_id: String,
    pub target_engines: Vec<KnowTargetEngine>,
    pub ordered_entries: Vec<KnowDictionaryEntry>,
    pub tenant_scoped: bool,
    pub authorized_only: bool,
    pub no_cross_tenant: bool,
    pub advisory_only: bool,
    pub no_execution_authority: bool,
}

impl KnowDictionaryPackBuildOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        pack_id: String,
        target_engines: Vec<KnowTargetEngine>,
        ordered_entries: Vec<KnowDictionaryEntry>,
        tenant_scoped: bool,
        authorized_only: bool,
        no_cross_tenant: bool,
        advisory_only: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1KNOW_CONTRACT_VERSION,
            capability_id: KnowCapabilityId::KnowDictionaryPackBuild,
            reason_code,
            pack_id,
            target_engines,
            ordered_entries,
            tenant_scoped,
            authorized_only,
            no_cross_tenant,
            advisory_only,
            no_execution_authority,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for KnowDictionaryPackBuildOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1KNOW_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "know_dictionary_pack_build_ok.schema_version",
                reason: "must match PH1KNOW_CONTRACT_VERSION",
            });
        }
        if self.capability_id != KnowCapabilityId::KnowDictionaryPackBuild {
            return Err(ContractViolation::InvalidValue {
                field: "know_dictionary_pack_build_ok.capability_id",
                reason: "must be KNOW_DICTIONARY_PACK_BUILD",
            });
        }
        validate_token("know_dictionary_pack_build_ok.pack_id", &self.pack_id, 128)?;
        if self.target_engines.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "know_dictionary_pack_build_ok.target_engines",
                reason: "must be non-empty",
            });
        }
        if self.target_engines.len() > 4 {
            return Err(ContractViolation::InvalidValue {
                field: "know_dictionary_pack_build_ok.target_engines",
                reason: "must be <= 4",
            });
        }
        let mut seen_targets = BTreeSet::new();
        for target in &self.target_engines {
            if !seen_targets.insert(target.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "know_dictionary_pack_build_ok.target_engines",
                    reason: "must be unique",
                });
            }
        }

        if self.ordered_entries.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "know_dictionary_pack_build_ok.ordered_entries",
                reason: "must be non-empty",
            });
        }
        if self.ordered_entries.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "know_dictionary_pack_build_ok.ordered_entries",
                reason: "must be <= 128",
            });
        }
        let mut seen_entries = BTreeSet::new();
        for entry in &self.ordered_entries {
            entry.validate()?;
            if !seen_entries.insert(entry.entry_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "know_dictionary_pack_build_ok.ordered_entries",
                    reason: "entry_id must be unique",
                });
            }
        }

        if !self.tenant_scoped {
            return Err(ContractViolation::InvalidValue {
                field: "know_dictionary_pack_build_ok.tenant_scoped",
                reason: "must be true",
            });
        }
        if !self.authorized_only {
            return Err(ContractViolation::InvalidValue {
                field: "know_dictionary_pack_build_ok.authorized_only",
                reason: "must be true",
            });
        }
        if !self.no_cross_tenant {
            return Err(ContractViolation::InvalidValue {
                field: "know_dictionary_pack_build_ok.no_cross_tenant",
                reason: "must be true",
            });
        }
        if !self.advisory_only {
            return Err(ContractViolation::InvalidValue {
                field: "know_dictionary_pack_build_ok.advisory_only",
                reason: "must be true",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "know_dictionary_pack_build_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowHintBundleSelectOk {
    pub schema_version: SchemaVersion,
    pub capability_id: KnowCapabilityId,
    pub reason_code: ReasonCodeId,
    pub validation_status: KnowValidationStatus,
    pub diagnostics: Vec<String>,
    pub selected_targets: Vec<KnowTargetEngine>,
    pub preserved_tenant_scope: bool,
    pub preserved_authorized_only: bool,
    pub preserved_no_cross_tenant: bool,
    pub advisory_only: bool,
    pub no_execution_authority: bool,
}

impl KnowHintBundleSelectOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        validation_status: KnowValidationStatus,
        diagnostics: Vec<String>,
        selected_targets: Vec<KnowTargetEngine>,
        preserved_tenant_scope: bool,
        preserved_authorized_only: bool,
        preserved_no_cross_tenant: bool,
        advisory_only: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1KNOW_CONTRACT_VERSION,
            capability_id: KnowCapabilityId::KnowHintBundleSelect,
            reason_code,
            validation_status,
            diagnostics,
            selected_targets,
            preserved_tenant_scope,
            preserved_authorized_only,
            preserved_no_cross_tenant,
            advisory_only,
            no_execution_authority,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for KnowHintBundleSelectOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1KNOW_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "know_hint_bundle_select_ok.schema_version",
                reason: "must match PH1KNOW_CONTRACT_VERSION",
            });
        }
        if self.capability_id != KnowCapabilityId::KnowHintBundleSelect {
            return Err(ContractViolation::InvalidValue {
                field: "know_hint_bundle_select_ok.capability_id",
                reason: "must be KNOW_HINT_BUNDLE_SELECT",
            });
        }
        if self.diagnostics.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "know_hint_bundle_select_ok.diagnostics",
                reason: "must be <= 16",
            });
        }
        for diagnostic in &self.diagnostics {
            validate_token("know_hint_bundle_select_ok.diagnostics", diagnostic, 96)?;
        }
        if self.validation_status == KnowValidationStatus::Fail && self.diagnostics.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "know_hint_bundle_select_ok.diagnostics",
                reason: "must be non-empty when validation_status=FAIL",
            });
        }

        if self.selected_targets.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "know_hint_bundle_select_ok.selected_targets",
                reason: "must be non-empty",
            });
        }
        if self.selected_targets.len() > 4 {
            return Err(ContractViolation::InvalidValue {
                field: "know_hint_bundle_select_ok.selected_targets",
                reason: "must be <= 4",
            });
        }
        let mut seen_targets = BTreeSet::new();
        for target in &self.selected_targets {
            if !seen_targets.insert(target.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "know_hint_bundle_select_ok.selected_targets",
                    reason: "must be unique",
                });
            }
        }

        if self.validation_status == KnowValidationStatus::Ok
            && (!self.preserved_tenant_scope
                || !self.preserved_authorized_only
                || !self.preserved_no_cross_tenant)
        {
            return Err(ContractViolation::InvalidValue {
                field: "know_hint_bundle_select_ok",
                reason: "OK status requires tenant scope, authorized-only, and no-cross-tenant preserved",
            });
        }
        if !self.advisory_only {
            return Err(ContractViolation::InvalidValue {
                field: "know_hint_bundle_select_ok.advisory_only",
                reason: "must be true",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "know_hint_bundle_select_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: KnowCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl KnowRefuse {
    pub fn v1(
        capability_id: KnowCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1KNOW_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for KnowRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1KNOW_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "know_refuse.schema_version",
                reason: "must match PH1KNOW_CONTRACT_VERSION",
            });
        }
        validate_text("know_refuse.message", &self.message, 192)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1KnowResponse {
    KnowDictionaryPackBuildOk(KnowDictionaryPackBuildOk),
    KnowHintBundleSelectOk(KnowHintBundleSelectOk),
    Refuse(KnowRefuse),
}

impl Validate for Ph1KnowResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1KnowResponse::KnowDictionaryPackBuildOk(out) => out.validate(),
            Ph1KnowResponse::KnowHintBundleSelectOk(out) => out.validate(),
            Ph1KnowResponse::Refuse(out) => out.validate(),
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
            reason: "must contain only ASCII token characters",
        });
    }
    Ok(())
}

fn validate_text(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if value.trim().is_empty() {
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
    if value.chars().any(char::is_control) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not contain control chars",
        });
    }
    Ok(())
}

fn validate_locale_tag(field: &'static str, value: &str) -> Result<(), ContractViolation> {
    if value.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be non-empty",
        });
    }
    if value.len() > 16 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be <= 16",
        });
    }
    if value
        .chars()
        .any(|c| !(c.is_ascii_alphanumeric() || c == '-'))
    {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must contain only ASCII alphanumeric and hyphen",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope() -> KnowRequestEnvelope {
        KnowRequestEnvelope::v1(CorrelationId(3901), TurnId(371), 8, 8).unwrap()
    }

    fn entry(
        entry_id: &str,
        tenant_id: &str,
        entry_kind: KnowEntryKind,
        source_kind: KnowSourceKind,
        canonical_term: &str,
        normalized_term: &str,
        pronunciation_hint: Option<&str>,
    ) -> KnowDictionaryEntry {
        KnowDictionaryEntry::v1(
            entry_id.to_string(),
            tenant_id.to_string(),
            entry_kind,
            source_kind,
            canonical_term.to_string(),
            normalized_term.to_string(),
            "en".to_string(),
            pronunciation_hint.map(|v| v.to_string()),
            format!("know:evidence:{}", entry_id),
        )
        .unwrap()
    }

    #[test]
    fn at_know_01_dictionary_pack_build_contract_is_schema_valid() {
        let req = KnowDictionaryPackBuildRequest::v1(
            envelope(),
            "tenant_1".to_string(),
            vec![
                entry(
                    "entry_1",
                    "tenant_1",
                    KnowEntryKind::EmployeeNamePreferred,
                    KnowSourceKind::HrOrgAuthorized,
                    "Jia Li",
                    "jia li",
                    None,
                ),
                entry(
                    "entry_2",
                    "tenant_1",
                    KnowEntryKind::PronunciationHint,
                    KnowSourceKind::UserProvidedConsent,
                    "Selene",
                    "selene",
                    Some("seh-leen"),
                ),
            ],
            true,
            true,
            true,
            Some("artifact.learn.v3".to_string()),
            true,
            true,
            true,
        )
        .unwrap();

        assert!(req.validate().is_ok());

        let out = KnowDictionaryPackBuildOk::v1(
            ReasonCodeId(401),
            "know.pack.tenant_1.abc".to_string(),
            vec![
                KnowTargetEngine::C,
                KnowTargetEngine::Srl,
                KnowTargetEngine::Nlp,
            ],
            req.entries.clone(),
            true,
            true,
            true,
            true,
            true,
        )
        .unwrap();
        assert!(out.validate().is_ok());
    }

    #[test]
    fn at_know_02_cross_tenant_entries_are_rejected() {
        let req = KnowDictionaryPackBuildRequest::v1(
            envelope(),
            "tenant_1".to_string(),
            vec![entry(
                "entry_1",
                "tenant_2",
                KnowEntryKind::ProjectCode,
                KnowSourceKind::HrOrgAuthorized,
                "Atlas",
                "atlas",
                None,
            )],
            false,
            false,
            true,
            None,
            true,
            true,
            true,
        );

        assert!(req.is_err());
    }

    #[test]
    fn at_know_03_hint_bundle_select_requires_true_boundary_flags() {
        let req = KnowHintBundleSelectRequest::v1(
            envelope(),
            "tenant_1".to_string(),
            "know.pack.tenant_1.def".to_string(),
            vec![entry(
                "entry_1",
                "tenant_1",
                KnowEntryKind::ApprovedAbbreviation,
                KnowSourceKind::LearnArtifact,
                "SRE",
                "sre",
                None,
            )],
            vec![KnowTargetEngine::C],
            false,
            true,
            true,
        );

        assert!(req.is_err());
    }

    #[test]
    fn at_know_04_select_ok_fail_status_requires_diagnostics() {
        let out = KnowHintBundleSelectOk::v1(
            ReasonCodeId(402),
            KnowValidationStatus::Fail,
            vec![],
            vec![KnowTargetEngine::Nlp],
            false,
            false,
            false,
            true,
            true,
        );

        assert!(out.is_err());
    }
}
