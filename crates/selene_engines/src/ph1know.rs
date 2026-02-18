#![forbid(unsafe_code)]

use std::cmp::min;
use std::collections::BTreeSet;

use selene_kernel_contracts::ph1know::{
    KnowCapabilityId, KnowDictionaryEntry, KnowDictionaryPackBuildOk,
    KnowDictionaryPackBuildRequest, KnowHintBundleSelectOk, KnowHintBundleSelectRequest,
    KnowRefuse, KnowSourceKind, KnowTargetEngine, KnowValidationStatus, Ph1KnowRequest,
    Ph1KnowResponse,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.KNOW reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_KNOW_OK_DICTIONARY_PACK_BUILD: ReasonCodeId = ReasonCodeId(0x4B4E_0001);
    pub const PH1_KNOW_OK_HINT_BUNDLE_SELECT: ReasonCodeId = ReasonCodeId(0x4B4E_0002);

    pub const PH1_KNOW_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x4B4E_00F1);
    pub const PH1_KNOW_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x4B4E_00F2);
    pub const PH1_KNOW_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x4B4E_00F3);
    pub const PH1_KNOW_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4B4E_00F4);
    pub const PH1_KNOW_CONSENT_REQUIRED: ReasonCodeId = ReasonCodeId(0x4B4E_00F5);
    pub const PH1_KNOW_UNAUTHORIZED_SOURCE: ReasonCodeId = ReasonCodeId(0x4B4E_00F6);
    pub const PH1_KNOW_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4B4E_00F7);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1KnowConfig {
    pub max_entries: u8,
    pub max_diagnostics: u8,
}

impl Ph1KnowConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_entries: 48,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1KnowRuntime {
    config: Ph1KnowConfig,
}

impl Ph1KnowRuntime {
    pub fn new(config: Ph1KnowConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1KnowRequest) -> Ph1KnowResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_KNOW_INPUT_SCHEMA_INVALID,
                "know request failed contract validation",
            );
        }

        match req {
            Ph1KnowRequest::KnowDictionaryPackBuild(r) => self.run_dictionary_pack_build(r),
            Ph1KnowRequest::KnowHintBundleSelect(r) => self.run_hint_bundle_select(r),
        }
    }

    fn run_dictionary_pack_build(&self, req: &KnowDictionaryPackBuildRequest) -> Ph1KnowResponse {
        if req.entries.is_empty() {
            return self.refuse(
                KnowCapabilityId::KnowDictionaryPackBuild,
                reason_codes::PH1_KNOW_UPSTREAM_INPUT_MISSING,
                "knowledge entries are empty",
            );
        }

        let budget = min(
            req.envelope.max_entries as usize,
            self.config.max_entries as usize,
        );
        if req.entries.len() > budget {
            return self.refuse(
                KnowCapabilityId::KnowDictionaryPackBuild,
                reason_codes::PH1_KNOW_BUDGET_EXCEEDED,
                "knowledge entry budget exceeded",
            );
        }

        if req
            .entries
            .iter()
            .any(|entry| entry.tenant_id != req.tenant_id)
        {
            return self.refuse(
                KnowCapabilityId::KnowDictionaryPackBuild,
                reason_codes::PH1_KNOW_VALIDATION_FAILED,
                "cross-tenant knowledge entry detected",
            );
        }

        if req.entries.iter().any(|entry| {
            matches!(entry.source_kind, KnowSourceKind::UserProvidedConsent)
                && !req.user_consent_asserted
        }) {
            return self.refuse(
                KnowCapabilityId::KnowDictionaryPackBuild,
                reason_codes::PH1_KNOW_CONSENT_REQUIRED,
                "user-provided knowledge terms require consent",
            );
        }

        if req.entries.iter().any(|entry| {
            matches!(entry.source_kind, KnowSourceKind::Unverified) && req.authorized_only_required
        }) {
            return self.refuse(
                KnowCapabilityId::KnowDictionaryPackBuild,
                reason_codes::PH1_KNOW_UNAUTHORIZED_SOURCE,
                "unverified knowledge source is not allowed",
            );
        }

        if req.entries.iter().any(|entry| {
            matches!(entry.source_kind, KnowSourceKind::HrOrgAuthorized) && !req.hr_org_authorized
        }) {
            return self.refuse(
                KnowCapabilityId::KnowDictionaryPackBuild,
                reason_codes::PH1_KNOW_VALIDATION_FAILED,
                "hr/org authorization missing for knowledge terms",
            );
        }

        let mut deduped: Vec<KnowDictionaryEntry> = Vec::new();
        let mut seen_keys: BTreeSet<String> = BTreeSet::new();
        for entry in &req.entries {
            let key = format!(
                "{}::{}::{:?}",
                entry.normalized_term.trim().to_lowercase(),
                entry.locale_tag.to_ascii_lowercase(),
                entry.entry_kind
            );
            if seen_keys.insert(key) {
                deduped.push(entry.clone());
            }
            if deduped.len() >= budget {
                break;
            }
        }

        if deduped.is_empty() {
            return self.refuse(
                KnowCapabilityId::KnowDictionaryPackBuild,
                reason_codes::PH1_KNOW_UPSTREAM_INPUT_MISSING,
                "no valid entries after normalization",
            );
        }

        deduped.sort_by(|a, b| {
            a.normalized_term
                .cmp(&b.normalized_term)
                .then(a.locale_tag.cmp(&b.locale_tag))
                .then(a.entry_id.cmp(&b.entry_id))
        });

        let mut target_engines = vec![
            KnowTargetEngine::C,
            KnowTargetEngine::Srl,
            KnowTargetEngine::Nlp,
        ];
        if deduped
            .iter()
            .any(|entry| entry.pronunciation_hint.is_some())
        {
            target_engines.push(KnowTargetEngine::Tts);
        }

        let pack_id = deterministic_pack_id(&req.tenant_id, &deduped);
        match KnowDictionaryPackBuildOk::v1(
            reason_codes::PH1_KNOW_OK_DICTIONARY_PACK_BUILD,
            pack_id,
            target_engines,
            deduped,
            true,
            true,
            true,
            true,
            true,
        ) {
            Ok(out) => Ph1KnowResponse::KnowDictionaryPackBuildOk(out),
            Err(_) => self.refuse(
                KnowCapabilityId::KnowDictionaryPackBuild,
                reason_codes::PH1_KNOW_INTERNAL_PIPELINE_ERROR,
                "failed to construct dictionary-pack build output",
            ),
        }
    }

    fn run_hint_bundle_select(&self, req: &KnowHintBundleSelectRequest) -> Ph1KnowResponse {
        if req.ordered_entries.is_empty() {
            return self.refuse(
                KnowCapabilityId::KnowHintBundleSelect,
                reason_codes::PH1_KNOW_UPSTREAM_INPUT_MISSING,
                "knowledge entries are empty",
            );
        }

        let budget = min(
            req.envelope.max_entries as usize,
            self.config.max_entries as usize,
        );
        if req.ordered_entries.len() > budget {
            return self.refuse(
                KnowCapabilityId::KnowHintBundleSelect,
                reason_codes::PH1_KNOW_BUDGET_EXCEEDED,
                "knowledge entry budget exceeded",
            );
        }

        let mut diagnostics: Vec<String> = Vec::new();

        let has_cross_tenant = req
            .ordered_entries
            .iter()
            .any(|entry| entry.tenant_id != req.tenant_id);
        if has_cross_tenant {
            diagnostics.push("tenant_scope_mismatch_detected".to_string());
        }

        let has_unauthorized = req
            .ordered_entries
            .iter()
            .any(|entry| matches!(entry.source_kind, KnowSourceKind::Unverified));
        if has_unauthorized {
            diagnostics.push("unauthorized_source_detected".to_string());
        }

        if req.target_engines.contains(&KnowTargetEngine::Tts)
            && !req
                .ordered_entries
                .iter()
                .any(|entry| entry.pronunciation_hint.is_some())
        {
            diagnostics.push("tts_target_missing_pronunciation_hint".to_string());
        }

        let mut expected = req.ordered_entries.clone();
        expected.sort_by(|a, b| {
            a.normalized_term
                .cmp(&b.normalized_term)
                .then(a.locale_tag.cmp(&b.locale_tag))
                .then(a.entry_id.cmp(&b.entry_id))
        });

        let expected_order_ids = expected
            .iter()
            .map(|entry| entry.entry_id.as_str())
            .collect::<Vec<_>>();
        let actual_order_ids = req
            .ordered_entries
            .iter()
            .map(|entry| entry.entry_id.as_str())
            .collect::<Vec<_>>();
        if actual_order_ids != expected_order_ids {
            diagnostics.push("entry_order_not_canonical".to_string());
        }

        diagnostics.truncate(min(
            self.config.max_diagnostics as usize,
            req.envelope.max_diagnostics as usize,
        ));

        let preserved_tenant_scope = !has_cross_tenant;
        let preserved_authorized_only = !has_unauthorized;
        let preserved_no_cross_tenant = !has_cross_tenant;

        let (validation_status, reason_code) = if diagnostics.is_empty() {
            (
                KnowValidationStatus::Ok,
                reason_codes::PH1_KNOW_OK_HINT_BUNDLE_SELECT,
            )
        } else {
            (
                KnowValidationStatus::Fail,
                reason_codes::PH1_KNOW_VALIDATION_FAILED,
            )
        };

        match KnowHintBundleSelectOk::v1(
            reason_code,
            validation_status,
            diagnostics,
            req.target_engines.clone(),
            preserved_tenant_scope,
            preserved_authorized_only,
            preserved_no_cross_tenant,
            true,
            true,
        ) {
            Ok(out) => Ph1KnowResponse::KnowHintBundleSelectOk(out),
            Err(_) => self.refuse(
                KnowCapabilityId::KnowHintBundleSelect,
                reason_codes::PH1_KNOW_INTERNAL_PIPELINE_ERROR,
                "failed to construct hint-bundle-select output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: KnowCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1KnowResponse {
        let out = KnowRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("KnowRefuse::v1 must construct for static message");
        Ph1KnowResponse::Refuse(out)
    }
}

fn capability_from_request(req: &Ph1KnowRequest) -> KnowCapabilityId {
    match req {
        Ph1KnowRequest::KnowDictionaryPackBuild(_) => KnowCapabilityId::KnowDictionaryPackBuild,
        Ph1KnowRequest::KnowHintBundleSelect(_) => KnowCapabilityId::KnowHintBundleSelect,
    }
}

fn deterministic_pack_id(tenant_id: &str, entries: &[KnowDictionaryEntry]) -> String {
    let mut hash: u64 = 1469598103934665603;
    for entry in entries {
        for byte in entry.entry_id.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(1099511628211);
        }
        for byte in entry.normalized_term.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(1099511628211);
        }
        if let Some(pronunciation_hint) = &entry.pronunciation_hint {
            for byte in pronunciation_hint.bytes() {
                hash ^= byte as u64;
                hash = hash.wrapping_mul(1099511628211);
            }
        }
    }

    let tenant_norm = tenant_id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>();

    format!("know.pack.{}.{:016x}", tenant_norm, hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1know::{
        KnowEntryKind, KnowRequestEnvelope, KnowSourceKind, KnowTargetEngine,
    };

    fn runtime() -> Ph1KnowRuntime {
        Ph1KnowRuntime::new(Ph1KnowConfig::mvp_v1())
    }

    fn envelope(max_entries: u8) -> KnowRequestEnvelope {
        KnowRequestEnvelope::v1(CorrelationId(4001), TurnId(381), max_entries, 8).unwrap()
    }

    fn entry(
        entry_id: &str,
        entry_kind: KnowEntryKind,
        source_kind: KnowSourceKind,
        canonical_term: &str,
        normalized_term: &str,
        pronunciation_hint: Option<&str>,
    ) -> KnowDictionaryEntry {
        KnowDictionaryEntry::v1(
            entry_id.to_string(),
            "tenant_1".to_string(),
            entry_kind,
            source_kind,
            canonical_term.to_string(),
            normalized_term.to_string(),
            "en".to_string(),
            pronunciation_hint.map(|value| value.to_string()),
            format!("know:evidence:{}", entry_id),
        )
        .unwrap()
    }

    fn build_request() -> KnowDictionaryPackBuildRequest {
        KnowDictionaryPackBuildRequest::v1(
            envelope(12),
            "tenant_1".to_string(),
            vec![
                entry(
                    "entry_1",
                    KnowEntryKind::EmployeeNamePreferred,
                    KnowSourceKind::HrOrgAuthorized,
                    "Jia Li",
                    "jia li",
                    None,
                ),
                entry(
                    "entry_2",
                    KnowEntryKind::ProjectCode,
                    KnowSourceKind::LearnArtifact,
                    "Atlas",
                    "atlas",
                    None,
                ),
                entry(
                    "entry_3",
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
            Some("artifact.learn.v4".to_string()),
            true,
            true,
            true,
        )
        .unwrap()
    }

    #[test]
    fn at_know_01_dictionary_pack_build_output_is_schema_valid() {
        let out = runtime().run(&Ph1KnowRequest::KnowDictionaryPackBuild(build_request()));

        assert!(out.validate().is_ok());
        match out {
            Ph1KnowResponse::KnowDictionaryPackBuildOk(ok) => {
                assert!(!ok.pack_id.is_empty());
                assert!(!ok.ordered_entries.is_empty());
            }
            _ => panic!("expected KnowDictionaryPackBuildOk"),
        }
    }

    #[test]
    fn at_know_02_build_order_and_pack_id_are_deterministic() {
        let req = Ph1KnowRequest::KnowDictionaryPackBuild(build_request());

        let out_1 = runtime().run(&req);
        let out_2 = runtime().run(&req);

        match (out_1, out_2) {
            (
                Ph1KnowResponse::KnowDictionaryPackBuildOk(a),
                Ph1KnowResponse::KnowDictionaryPackBuildOk(b),
            ) => {
                assert_eq!(a.pack_id, b.pack_id);
                assert_eq!(a.ordered_entries, b.ordered_entries);
            }
            _ => panic!("expected KnowDictionaryPackBuildOk outputs"),
        }
    }

    #[test]
    fn at_know_03_unverified_sources_fail_closed() {
        let req = KnowDictionaryPackBuildRequest::v1(
            envelope(8),
            "tenant_1".to_string(),
            vec![entry(
                "entry_1",
                KnowEntryKind::InternalProductName,
                KnowSourceKind::Unverified,
                "Phoenix",
                "phoenix",
                None,
            )],
            false,
            false,
            true,
            None,
            true,
            true,
            true,
        )
        .unwrap();

        let out = runtime().run(&Ph1KnowRequest::KnowDictionaryPackBuild(req));
        match out {
            Ph1KnowResponse::Refuse(refuse) => {
                assert_eq!(
                    refuse.reason_code,
                    reason_codes::PH1_KNOW_UNAUTHORIZED_SOURCE
                )
            }
            _ => panic!("expected Refuse"),
        }
    }

    #[test]
    fn at_know_04_hint_bundle_select_fails_when_tts_target_has_no_pronunciation_hints() {
        let build_ok = match runtime().run(&Ph1KnowRequest::KnowDictionaryPackBuild(
            KnowDictionaryPackBuildRequest::v1(
                envelope(8),
                "tenant_1".to_string(),
                vec![entry(
                    "entry_1",
                    KnowEntryKind::ApprovedAbbreviation,
                    KnowSourceKind::HrOrgAuthorized,
                    "SRE",
                    "sre",
                    None,
                )],
                false,
                false,
                true,
                None,
                true,
                true,
                true,
            )
            .unwrap(),
        )) {
            Ph1KnowResponse::KnowDictionaryPackBuildOk(ok) => ok,
            _ => panic!("expected KnowDictionaryPackBuildOk"),
        };

        let select_req = Ph1KnowRequest::KnowHintBundleSelect(
            KnowHintBundleSelectRequest::v1(
                envelope(8),
                "tenant_1".to_string(),
                build_ok.pack_id,
                build_ok.ordered_entries,
                vec![
                    KnowTargetEngine::C,
                    KnowTargetEngine::Nlp,
                    KnowTargetEngine::Tts,
                ],
                true,
                true,
                true,
            )
            .unwrap(),
        );

        let out = runtime().run(&select_req);
        match out {
            Ph1KnowResponse::KnowHintBundleSelectOk(ok) => {
                assert_eq!(ok.validation_status, KnowValidationStatus::Fail)
            }
            _ => panic!("expected KnowHintBundleSelectOk"),
        }
    }
}
