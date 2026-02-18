#![forbid(unsafe_code)]

use std::cmp::min;
use std::collections::BTreeSet;

use selene_kernel_contracts::ph1pron::{
    Ph1PronRequest, Ph1PronResponse, PronApplyValidateOk, PronApplyValidateRequest,
    PronCapabilityId, PronLexiconEntry, PronLexiconPackBuildOk, PronLexiconPackBuildRequest,
    PronRefuse, PronScope, PronTargetEngine, PronValidationStatus,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.PRON reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_PRON_OK_LEXICON_PACK_BUILD: ReasonCodeId = ReasonCodeId(0x5052_4F01);
    pub const PH1_PRON_OK_APPLY_VALIDATE: ReasonCodeId = ReasonCodeId(0x5052_4F02);

    pub const PH1_PRON_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x5052_4FF1);
    pub const PH1_PRON_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x5052_4FF2);
    pub const PH1_PRON_CONSENT_REQUIRED: ReasonCodeId = ReasonCodeId(0x5052_4FF3);
    pub const PH1_PRON_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x5052_4FF4);
    pub const PH1_PRON_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5052_4FF5);
    pub const PH1_PRON_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x5052_4FF6);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1PronConfig {
    pub max_entries: u8,
    pub max_diagnostics: u8,
}

impl Ph1PronConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_entries: 32,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1PronRuntime {
    config: Ph1PronConfig,
}

impl Ph1PronRuntime {
    pub fn new(config: Ph1PronConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1PronRequest) -> Ph1PronResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_PRON_INPUT_SCHEMA_INVALID,
                "pron request failed contract validation",
            );
        }

        match req {
            Ph1PronRequest::PronLexiconPackBuild(r) => self.run_pack_build(r),
            Ph1PronRequest::PronApplyValidate(r) => self.run_apply_validate(r),
        }
    }

    fn run_pack_build(&self, req: &PronLexiconPackBuildRequest) -> Ph1PronResponse {
        if req.entries.is_empty() {
            return self.refuse(
                PronCapabilityId::PronLexiconPackBuild,
                reason_codes::PH1_PRON_UPSTREAM_INPUT_MISSING,
                "pron entries are empty",
            );
        }

        if req.entries.len() > self.config.max_entries as usize {
            return self.refuse(
                PronCapabilityId::PronLexiconPackBuild,
                reason_codes::PH1_PRON_BUDGET_EXCEEDED,
                "pron entry budget exceeded",
            );
        }

        if req.scope == PronScope::User && !req.consent_asserted {
            return self.refuse(
                PronCapabilityId::PronLexiconPackBuild,
                reason_codes::PH1_PRON_CONSENT_REQUIRED,
                "user-scoped pronunciation pack requires consent",
            );
        }

        let budget = min(req.envelope.max_entries, self.config.max_entries) as usize;
        let mut seen: BTreeSet<String> = BTreeSet::new();
        let mut entries: Vec<PronLexiconEntry> = Vec::new();

        for entry in &req.entries {
            if entries.len() >= budget {
                break;
            }
            let key = format!(
                "{}::{}",
                entry.grapheme.trim().to_ascii_lowercase(),
                entry.locale_tag.to_ascii_lowercase(),
            );
            if seen.insert(key) {
                entries.push(entry.clone());
            }
        }

        if entries.is_empty() {
            return self.refuse(
                PronCapabilityId::PronLexiconPackBuild,
                reason_codes::PH1_PRON_UPSTREAM_INPUT_MISSING,
                "no valid pronunciation entries after normalization",
            );
        }

        let pack_id =
            deterministic_pack_id(&req.tenant_id, req.user_id.as_deref(), req.scope, &entries);

        let out = PronLexiconPackBuildOk::v1(
            reason_codes::PH1_PRON_OK_LEXICON_PACK_BUILD,
            pack_id,
            req.scope,
            vec![
                PronTargetEngine::Tts,
                PronTargetEngine::VoiceId,
                PronTargetEngine::Wake,
            ],
            entries,
            true,
            req.scope == PronScope::User,
            true,
        );

        match out {
            Ok(ok) => Ph1PronResponse::PronLexiconPackBuildOk(ok),
            Err(_) => self.refuse(
                PronCapabilityId::PronLexiconPackBuild,
                reason_codes::PH1_PRON_INTERNAL_PIPELINE_ERROR,
                "failed to construct pronunciation pack build output",
            ),
        }
    }

    fn run_apply_validate(&self, req: &PronApplyValidateRequest) -> Ph1PronResponse {
        if req.entries.is_empty() {
            return self.refuse(
                PronCapabilityId::PronApplyValidate,
                reason_codes::PH1_PRON_UPSTREAM_INPUT_MISSING,
                "pron entries are empty",
            );
        }
        if req.entries.len() > self.config.max_entries as usize {
            return self.refuse(
                PronCapabilityId::PronApplyValidate,
                reason_codes::PH1_PRON_BUDGET_EXCEEDED,
                "pron entry budget exceeded",
            );
        }

        let mut diagnostics: Vec<String> = Vec::new();
        let mut dedupe: BTreeSet<(String, String)> = BTreeSet::new();

        for (idx, entry) in req.entries.iter().enumerate() {
            if diagnostics.len() >= self.config.max_diagnostics as usize {
                break;
            }

            let pair = (
                entry.grapheme.trim().to_ascii_lowercase(),
                entry.locale_tag.to_ascii_lowercase(),
            );
            if !dedupe.insert(pair) {
                diagnostics.push(format!("entry_{idx}_duplicate_grapheme_locale"));
            }

            if !entry.locale_tag.eq_ignore_ascii_case(&req.locale_tag) {
                diagnostics.push(format!("entry_{idx}_locale_mismatch"));
                if diagnostics.len() >= self.config.max_diagnostics as usize {
                    break;
                }
            }

            if req.target_engine == PronTargetEngine::Wake && entry.grapheme.chars().count() > 24 {
                diagnostics.push(format!("entry_{idx}_wake_grapheme_too_long"));
                if diagnostics.len() >= self.config.max_diagnostics as usize {
                    break;
                }
            }
        }

        let (validation_status, reason_code) = if diagnostics.is_empty() {
            (
                PronValidationStatus::Ok,
                reason_codes::PH1_PRON_OK_APPLY_VALIDATE,
            )
        } else {
            (
                PronValidationStatus::Fail,
                reason_codes::PH1_PRON_VALIDATION_FAILED,
            )
        };

        let out = PronApplyValidateOk::v1(
            reason_code,
            validation_status,
            req.pack_id.clone(),
            req.target_engine,
            req.locale_tag.clone(),
            diagnostics,
            true,
        );

        match out {
            Ok(ok) => Ph1PronResponse::PronApplyValidateOk(ok),
            Err(_) => self.refuse(
                PronCapabilityId::PronApplyValidate,
                reason_codes::PH1_PRON_INTERNAL_PIPELINE_ERROR,
                "failed to construct pronunciation apply-validate output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: PronCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1PronResponse {
        let r = PronRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("PronRefuse::v1 must construct for static message");
        Ph1PronResponse::Refuse(r)
    }
}

fn capability_from_request(req: &Ph1PronRequest) -> PronCapabilityId {
    match req {
        Ph1PronRequest::PronLexiconPackBuild(_) => PronCapabilityId::PronLexiconPackBuild,
        Ph1PronRequest::PronApplyValidate(_) => PronCapabilityId::PronApplyValidate,
    }
}

fn deterministic_pack_id(
    tenant_id: &str,
    user_id: Option<&str>,
    scope: PronScope,
    entries: &[PronLexiconEntry],
) -> String {
    let scope_id = match scope {
        PronScope::Tenant => "tenant",
        PronScope::User => "user",
    };
    let user_token = user_id.unwrap_or("none");

    let mut hash: u64 = 1469598103934665603;
    for entry in entries {
        for byte in entry.grapheme.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(1099511628211);
        }
        for byte in entry.phoneme.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(1099511628211);
        }
        for byte in entry.locale_tag.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(1099511628211);
        }
    }

    let tenant_norm = tenant_id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>();
    let user_norm = user_token
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>();

    format!(
        "pron.pack.{}.{}.{}.{:016x}",
        tenant_norm, scope_id, user_norm, hash
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1pron::{PronRequestEnvelope, PronScope, PronTargetEngine};

    fn envelope(max_entries: u8) -> PronRequestEnvelope {
        PronRequestEnvelope::v1(CorrelationId(1301), TurnId(91), max_entries).unwrap()
    }

    fn entry(id: &str, grapheme: &str, phoneme: &str, locale: &str) -> PronLexiconEntry {
        PronLexiconEntry::v1(
            id.to_string(),
            grapheme.to_string(),
            phoneme.to_string(),
            locale.to_string(),
        )
        .unwrap()
    }

    #[test]
    fn at_pron_01_pack_build_output_is_schema_valid() {
        let runtime = Ph1PronRuntime::new(Ph1PronConfig::mvp_v1());

        let req = Ph1PronRequest::PronLexiconPackBuild(
            PronLexiconPackBuildRequest::v1(
                envelope(8),
                "tenant_a".to_string(),
                None,
                PronScope::Tenant,
                false,
                vec![entry("e1", "selene", "suh-leen", "en")],
            )
            .unwrap(),
        );

        let out = runtime.run(&req);
        match out {
            Ph1PronResponse::PronLexiconPackBuildOk(ok) => {
                assert!(ok.validate().is_ok());
                assert!(ok
                    .target_engines
                    .iter()
                    .any(|t| *t == PronTargetEngine::Tts));
                assert!(ok
                    .target_engines
                    .iter()
                    .any(|t| *t == PronTargetEngine::VoiceId));
                assert!(ok
                    .target_engines
                    .iter()
                    .any(|t| *t == PronTargetEngine::Wake));
            }
            _ => panic!("expected PronLexiconPackBuildOk"),
        }
    }

    #[test]
    fn at_pron_02_user_scope_requires_consent() {
        let runtime = Ph1PronRuntime::new(Ph1PronConfig::mvp_v1());

        let req = Ph1PronRequest::PronLexiconPackBuild(
            PronLexiconPackBuildRequest::v1(
                envelope(8),
                "tenant_a".to_string(),
                Some("user_1".to_string()),
                PronScope::User,
                false,
                vec![entry("e1", "selene", "suh-leen", "en")],
            )
            .unwrap(),
        );

        let out = runtime.run(&req);
        match out {
            Ph1PronResponse::Refuse(r) => {
                assert_eq!(r.reason_code, reason_codes::PH1_PRON_CONSENT_REQUIRED);
            }
            _ => panic!("expected Refuse"),
        }
    }

    #[test]
    fn at_pron_03_apply_validate_fails_on_locale_mismatch() {
        let runtime = Ph1PronRuntime::new(Ph1PronConfig::mvp_v1());

        let req = Ph1PronRequest::PronApplyValidate(
            PronApplyValidateRequest::v1(
                envelope(8),
                "pron.pack.tenant_a.tenant.none.1234abcd".to_string(),
                PronTargetEngine::Tts,
                "en".to_string(),
                vec![entry("e1", "selene", "suh-leen", "es")],
            )
            .unwrap(),
        );

        let out = runtime.run(&req);
        match out {
            Ph1PronResponse::PronApplyValidateOk(ok) => {
                assert!(ok.validate().is_ok());
                assert_eq!(ok.validation_status, PronValidationStatus::Fail);
                assert!(!ok.diagnostics.is_empty());
            }
            _ => panic!("expected PronApplyValidateOk"),
        }
    }

    #[test]
    fn at_pron_04_budget_overflow_fails_closed() {
        let runtime = Ph1PronRuntime::new(Ph1PronConfig {
            max_entries: 2,
            max_diagnostics: 8,
        });

        let req = Ph1PronRequest::PronLexiconPackBuild(
            PronLexiconPackBuildRequest::v1(
                envelope(8),
                "tenant_a".to_string(),
                None,
                PronScope::Tenant,
                false,
                vec![
                    entry("e1", "selene", "suh-leen", "en"),
                    entry("e2", "acme", "ak-mee", "en"),
                    entry("e3", "qa", "kew-ey", "en"),
                ],
            )
            .unwrap(),
        );

        let out = runtime.run(&req);
        match out {
            Ph1PronResponse::Refuse(r) => {
                assert_eq!(r.reason_code, reason_codes::PH1_PRON_BUDGET_EXCEEDED);
            }
            _ => panic!("expected Refuse"),
        }
    }
}
