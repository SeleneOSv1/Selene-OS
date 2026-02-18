#![forbid(unsafe_code)]

use std::cmp::min;
use std::collections::BTreeSet;

use selene_kernel_contracts::ph1learn::{
    LearnArtifactCandidate, LearnArtifactPackageBuildOk, LearnArtifactPackageBuildRequest,
    LearnArtifactTarget, LearnCapabilityId, LearnRefuse, LearnScope, LearnSignal,
    LearnSignalAggregateOk, LearnSignalAggregateRequest, LearnSignalType, LearnTargetEngine,
    LearnValidationStatus, Ph1LearnRequest, Ph1LearnResponse,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.LEARN reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_LEARN_OK_SIGNAL_AGGREGATE: ReasonCodeId = ReasonCodeId(0x4C45_0001);
    pub const PH1_LEARN_OK_ARTIFACT_PACKAGE_BUILD: ReasonCodeId = ReasonCodeId(0x4C45_0002);

    pub const PH1_LEARN_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x4C45_00F1);
    pub const PH1_LEARN_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x4C45_00F2);
    pub const PH1_LEARN_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x4C45_00F3);
    pub const PH1_LEARN_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4C45_00F4);
    pub const PH1_LEARN_CONSENT_REQUIRED: ReasonCodeId = ReasonCodeId(0x4C45_00F5);
    pub const PH1_LEARN_DERIVED_ONLY_GLOBAL_REQUIRED: ReasonCodeId = ReasonCodeId(0x4C45_00F6);
    pub const PH1_LEARN_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4C45_00F7);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1LearnConfig {
    pub max_signals: u8,
    pub max_artifacts: u8,
    pub max_diagnostics: u8,
}

impl Ph1LearnConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_signals: 24,
            max_artifacts: 16,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1LearnRuntime {
    config: Ph1LearnConfig,
}

impl Ph1LearnRuntime {
    pub fn new(config: Ph1LearnConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1LearnRequest) -> Ph1LearnResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_LEARN_INPUT_SCHEMA_INVALID,
                "learn request failed contract validation",
            );
        }

        match req {
            Ph1LearnRequest::LearnSignalAggregate(r) => self.run_signal_aggregate(r),
            Ph1LearnRequest::LearnArtifactPackageBuild(r) => self.run_artifact_package_build(r),
        }
    }

    fn run_signal_aggregate(&self, req: &LearnSignalAggregateRequest) -> Ph1LearnResponse {
        if req.signals.is_empty() {
            return self.refuse(
                LearnCapabilityId::LearnSignalAggregate,
                reason_codes::PH1_LEARN_UPSTREAM_INPUT_MISSING,
                "signals is empty",
            );
        }

        let signal_budget = min(req.envelope.max_signals, self.config.max_signals) as usize;
        if req.signals.len() > signal_budget {
            return self.refuse(
                LearnCapabilityId::LearnSignalAggregate,
                reason_codes::PH1_LEARN_BUDGET_EXCEEDED,
                "signals exceeds configured budget",
            );
        }

        let artifact_budget = min(req.envelope.max_artifacts, self.config.max_artifacts) as usize;
        if artifact_budget == 0 {
            return self.refuse(
                LearnCapabilityId::LearnSignalAggregate,
                reason_codes::PH1_LEARN_BUDGET_EXCEEDED,
                "artifact budget exceeded",
            );
        }

        if req
            .signals
            .iter()
            .any(|signal| signal.consent_required && !signal.consent_asserted)
        {
            return self.refuse(
                LearnCapabilityId::LearnSignalAggregate,
                reason_codes::PH1_LEARN_CONSENT_REQUIRED,
                "consent-required signal without asserted consent",
            );
        }

        if req.signals.iter().any(|signal| {
            signal.scope_hint == LearnScope::GlobalDerived
                && (signal.contains_sensitive_data || !signal.consent_asserted)
        }) {
            return self.refuse(
                LearnCapabilityId::LearnSignalAggregate,
                reason_codes::PH1_LEARN_DERIVED_ONLY_GLOBAL_REQUIRED,
                "global-derived signals must remain consent-safe and derived-only",
            );
        }

        let mut candidates: Vec<LearnArtifactCandidate> = Vec::new();
        let mut seen_artifact_ids: BTreeSet<String> = BTreeSet::new();

        for signal in &req.signals {
            let candidate = match build_candidate(signal) {
                Ok(candidate) => candidate,
                Err(_) => {
                    return self.refuse(
                        LearnCapabilityId::LearnSignalAggregate,
                        reason_codes::PH1_LEARN_INTERNAL_PIPELINE_ERROR,
                        "failed to derive learn artifact candidate",
                    );
                }
            };

            if seen_artifact_ids.insert(candidate.artifact_id.clone()) {
                candidates.push(candidate);
            }
        }

        if candidates.is_empty() {
            return self.refuse(
                LearnCapabilityId::LearnSignalAggregate,
                reason_codes::PH1_LEARN_UPSTREAM_INPUT_MISSING,
                "no candidates could be derived",
            );
        }

        candidates.sort_by(|a, b| {
            b.expected_effect_bp
                .cmp(&a.expected_effect_bp)
                .then(b.artifact_version.cmp(&a.artifact_version))
                .then(a.artifact_id.cmp(&b.artifact_id))
        });
        candidates.truncate(artifact_budget);

        let selected_artifact_id = candidates[0].artifact_id.clone();
        match LearnSignalAggregateOk::v1(
            reason_codes::PH1_LEARN_OK_SIGNAL_AGGREGATE,
            selected_artifact_id,
            candidates,
            true,
            true,
            true,
            true,
        ) {
            Ok(ok) => Ph1LearnResponse::LearnSignalAggregateOk(ok),
            Err(_) => self.refuse(
                LearnCapabilityId::LearnSignalAggregate,
                reason_codes::PH1_LEARN_INTERNAL_PIPELINE_ERROR,
                "failed to construct signal-aggregate output",
            ),
        }
    }

    fn run_artifact_package_build(
        &self,
        req: &LearnArtifactPackageBuildRequest,
    ) -> Ph1LearnResponse {
        if req.ordered_artifacts.is_empty() {
            return self.refuse(
                LearnCapabilityId::LearnArtifactPackageBuild,
                reason_codes::PH1_LEARN_UPSTREAM_INPUT_MISSING,
                "ordered_artifacts is empty",
            );
        }

        let artifact_budget = min(req.envelope.max_artifacts, self.config.max_artifacts) as usize;
        if req.ordered_artifacts.len() > artifact_budget {
            return self.refuse(
                LearnCapabilityId::LearnArtifactPackageBuild,
                reason_codes::PH1_LEARN_BUDGET_EXCEEDED,
                "ordered_artifacts exceeds configured budget",
            );
        }

        let mut diagnostics: Vec<String> = Vec::new();

        if req.ordered_artifacts[0].artifact_id != req.selected_artifact_id {
            diagnostics.push("selected_not_first_in_ordered_artifacts".to_string());
        }
        if !req
            .ordered_artifacts
            .iter()
            .any(|artifact| artifact.artifact_id == req.selected_artifact_id)
        {
            diagnostics.push("selected_artifact_not_present_in_ordered_artifacts".to_string());
        }

        let mut expected = req.ordered_artifacts.clone();
        expected.sort_by(|a, b| {
            b.artifact_version
                .cmp(&a.artifact_version)
                .then(b.expected_effect_bp.cmp(&a.expected_effect_bp))
                .then(a.artifact_id.cmp(&b.artifact_id))
        });

        let expected_order = expected
            .iter()
            .map(|artifact| artifact.artifact_id.as_str())
            .collect::<Vec<_>>();
        let actual_order = req
            .ordered_artifacts
            .iter()
            .map(|artifact| artifact.artifact_id.as_str())
            .collect::<Vec<_>>();

        if actual_order != expected_order {
            diagnostics.push("artifact_order_not_canonical".to_string());
        }

        if req.require_rollback_ptr
            && req
                .ordered_artifacts
                .iter()
                .any(|artifact| artifact.rollback_to.is_none())
        {
            diagnostics.push("rollback_pointer_missing".to_string());
        }

        let mut required_targets = BTreeSet::new();
        for artifact in &req.ordered_artifacts {
            for target in expected_target_engines(artifact.target) {
                required_targets.insert(target.as_str());
            }
        }

        let provided_targets = req
            .target_engines
            .iter()
            .map(|target| target.as_str())
            .collect::<BTreeSet<_>>();

        for target in required_targets {
            if !provided_targets.contains(target) {
                diagnostics.push(format!("target_engine_missing:{}", target));
            }
        }

        if req
            .ordered_artifacts
            .iter()
            .any(|artifact| artifact.scope == LearnScope::GlobalDerived && !artifact.consent_safe)
        {
            diagnostics.push("global_derived_artifact_not_consent_safe".to_string());
        }

        diagnostics.truncate(min(
            req.envelope.max_diagnostics as usize,
            self.config.max_diagnostics as usize,
        ));

        let artifacts_versioned = req
            .ordered_artifacts
            .iter()
            .all(|artifact| artifact.artifact_version > 0);
        let rollbackable = req
            .ordered_artifacts
            .iter()
            .all(|artifact| artifact.rollback_to.is_some());
        let no_runtime_drift = diagnostics.is_empty();

        let (validation_status, reason_code) = if diagnostics.is_empty() {
            (
                LearnValidationStatus::Ok,
                reason_codes::PH1_LEARN_OK_ARTIFACT_PACKAGE_BUILD,
            )
        } else {
            (
                LearnValidationStatus::Fail,
                reason_codes::PH1_LEARN_VALIDATION_FAILED,
            )
        };

        match LearnArtifactPackageBuildOk::v1(
            reason_code,
            validation_status,
            diagnostics,
            req.target_engines.clone(),
            artifacts_versioned,
            rollbackable,
            no_runtime_drift,
            true,
            true,
        ) {
            Ok(ok) => Ph1LearnResponse::LearnArtifactPackageBuildOk(ok),
            Err(_) => self.refuse(
                LearnCapabilityId::LearnArtifactPackageBuild,
                reason_codes::PH1_LEARN_INTERNAL_PIPELINE_ERROR,
                "failed to construct artifact-package-build output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: LearnCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1LearnResponse {
        let out = LearnRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("LearnRefuse::v1 must construct for static messages");
        Ph1LearnResponse::Refuse(out)
    }
}

fn capability_from_request(req: &Ph1LearnRequest) -> LearnCapabilityId {
    match req {
        Ph1LearnRequest::LearnSignalAggregate(_) => LearnCapabilityId::LearnSignalAggregate,
        Ph1LearnRequest::LearnArtifactPackageBuild(_) => {
            LearnCapabilityId::LearnArtifactPackageBuild
        }
    }
}

fn build_candidate(
    signal: &LearnSignal,
) -> Result<LearnArtifactCandidate, selene_kernel_contracts::ContractViolation> {
    let target = artifact_target_from_signal(signal.signal_type);
    let artifact_id = format!("learn.artifact.{}.{}", signal.tenant_id, signal.signal_id);
    let artifact_version = signal.occurrence_count as u32;
    let expected_effect_bp = score_signal(signal);
    let rollback_to = Some(format!("{}.prev", artifact_id));
    let consent_safe = !signal.contains_sensitive_data || signal.consent_asserted;

    LearnArtifactCandidate::v1(
        artifact_id,
        target,
        signal.scope_hint,
        signal.scope_ref.clone(),
        artifact_version,
        expected_effect_bp,
        signal.evidence_ref.clone(),
        rollback_to,
        consent_safe,
    )
}

fn artifact_target_from_signal(signal_type: LearnSignalType) -> LearnArtifactTarget {
    match signal_type {
        LearnSignalType::SttReject => LearnArtifactTarget::PaeRoutingWeights,
        LearnSignalType::UserCorrection => LearnArtifactTarget::KnowTenantGlossaryPack,
        LearnSignalType::ClarifyLoop => LearnArtifactTarget::PruneClarificationOrdering,
        LearnSignalType::ToolFail => LearnArtifactTarget::SearchWebExtractionHints,
        LearnSignalType::VocabularyRepeat => LearnArtifactTarget::KnowTenantGlossaryPack,
        LearnSignalType::BargeIn => LearnArtifactTarget::ListenEnvironmentProfile,
        LearnSignalType::DeliverySwitch => LearnArtifactTarget::CacheDecisionSkeleton,
    }
}

pub(crate) fn expected_target_engines(target: LearnArtifactTarget) -> Vec<LearnTargetEngine> {
    match target {
        LearnArtifactTarget::KnowTenantGlossaryPack => vec![LearnTargetEngine::Know],
        LearnArtifactTarget::PronLexiconPack => vec![LearnTargetEngine::Pron],
        LearnArtifactTarget::CacheDecisionSkeleton => vec![LearnTargetEngine::Cache],
        LearnArtifactTarget::PruneClarificationOrdering => vec![LearnTargetEngine::Prune],
        LearnArtifactTarget::PaeRoutingWeights => vec![LearnTargetEngine::Pae],
        LearnArtifactTarget::SearchWebExtractionHints => vec![LearnTargetEngine::Search],
        LearnArtifactTarget::ListenEnvironmentProfile => vec![LearnTargetEngine::Listen],
    }
}

fn score_signal(signal: &LearnSignal) -> i16 {
    let base = match signal.signal_type {
        LearnSignalType::SttReject => 950,
        LearnSignalType::UserCorrection => 840,
        LearnSignalType::ClarifyLoop => 780,
        LearnSignalType::ToolFail => 860,
        LearnSignalType::VocabularyRepeat => 700,
        LearnSignalType::BargeIn => 620,
        LearnSignalType::DeliverySwitch => 560,
    };

    let frequency_component = (signal.occurrence_count as i32 * 25).min(1200);
    let metric_component = signal.metric_value_bp as i32;
    let total = base + frequency_component + metric_component;
    total.clamp(-20_000, 20_000) as i16
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1learn::{
        LearnArtifactPackageBuildRequest, LearnRequestEnvelope, LearnScope,
    };

    fn runtime() -> Ph1LearnRuntime {
        Ph1LearnRuntime::new(Ph1LearnConfig::mvp_v1())
    }

    fn envelope(max_signals: u8, max_artifacts: u8) -> LearnRequestEnvelope {
        LearnRequestEnvelope::v1(
            CorrelationId(4301),
            TurnId(411),
            max_signals,
            max_artifacts,
            8,
        )
        .unwrap()
    }

    fn signal(
        signal_id: &str,
        signal_type: LearnSignalType,
        scope: LearnScope,
        scope_ref: Option<&str>,
        metric_value_bp: i16,
    ) -> LearnSignal {
        LearnSignal::v1(
            signal_id.to_string(),
            "tenant_1".to_string(),
            signal_type,
            scope,
            scope_ref.map(|v| v.to_string()),
            "signal_metric".to_string(),
            metric_value_bp,
            6,
            false,
            false,
            false,
            format!("learn:evidence:{}", signal_id),
        )
        .unwrap()
    }

    fn base_artifacts() -> Vec<LearnArtifactCandidate> {
        vec![
            LearnArtifactCandidate::v1(
                "artifact_b".to_string(),
                LearnArtifactTarget::PruneClarificationOrdering,
                LearnScope::Tenant,
                Some("tenant_1".to_string()),
                8,
                900,
                "learn:evidence:artifact_b".to_string(),
                Some("artifact_b.prev".to_string()),
                true,
            )
            .unwrap(),
            LearnArtifactCandidate::v1(
                "artifact_a".to_string(),
                LearnArtifactTarget::KnowTenantGlossaryPack,
                LearnScope::Tenant,
                Some("tenant_1".to_string()),
                7,
                850,
                "learn:evidence:artifact_a".to_string(),
                Some("artifact_a.prev".to_string()),
                true,
            )
            .unwrap(),
        ]
    }

    #[test]
    fn at_learn_01_signal_aggregate_output_is_schema_valid() {
        let req = Ph1LearnRequest::LearnSignalAggregate(
            LearnSignalAggregateRequest::v1(
                envelope(8, 6),
                "tenant_1".to_string(),
                vec![
                    signal(
                        "sig_1",
                        LearnSignalType::SttReject,
                        LearnScope::Tenant,
                        Some("tenant_1"),
                        300,
                    ),
                    signal(
                        "sig_2",
                        LearnSignalType::UserCorrection,
                        LearnScope::User,
                        Some("user_1"),
                        220,
                    ),
                ],
                true,
                true,
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        assert!(out.validate().is_ok());

        match out {
            Ph1LearnResponse::LearnSignalAggregateOk(ok) => {
                assert_eq!(ok.selected_artifact_id, ok.ordered_artifacts[0].artifact_id);
            }
            _ => panic!("expected LearnSignalAggregateOk"),
        }
    }

    #[test]
    fn at_learn_02_sensitive_signal_without_consent_fails_closed() {
        let sensitive = LearnSignal::v1(
            "sig_sensitive".to_string(),
            "tenant_1".to_string(),
            LearnSignalType::UserCorrection,
            LearnScope::User,
            Some("user_1".to_string()),
            "signal_metric".to_string(),
            100,
            4,
            true,
            true,
            false,
            "learn:evidence:sig_sensitive".to_string(),
        )
        .unwrap();

        let req = Ph1LearnRequest::LearnSignalAggregate(
            LearnSignalAggregateRequest::v1(
                envelope(8, 6),
                "tenant_1".to_string(),
                vec![sensitive],
                true,
                true,
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        match out {
            Ph1LearnResponse::Refuse(refuse) => {
                assert_eq!(refuse.reason_code, reason_codes::PH1_LEARN_CONSENT_REQUIRED);
            }
            _ => panic!("expected Refuse"),
        }
    }

    #[test]
    fn at_learn_03_package_build_detects_validation_drift() {
        let mut artifacts = base_artifacts();
        artifacts.swap(0, 1);

        let req = Ph1LearnRequest::LearnArtifactPackageBuild(
            LearnArtifactPackageBuildRequest::v1(
                envelope(8, 6),
                "tenant_1".to_string(),
                artifacts[0].artifact_id.clone(),
                artifacts,
                vec![LearnTargetEngine::Know],
                true,
                true,
                true,
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        match out {
            Ph1LearnResponse::LearnArtifactPackageBuildOk(ok) => {
                assert_eq!(ok.validation_status, LearnValidationStatus::Fail);
                assert!(!ok.diagnostics.is_empty());
            }
            _ => panic!("expected LearnArtifactPackageBuildOk"),
        }
    }

    #[test]
    fn at_learn_04_package_build_ok_when_constraints_hold() {
        let req = Ph1LearnRequest::LearnArtifactPackageBuild(
            LearnArtifactPackageBuildRequest::v1(
                envelope(8, 6),
                "tenant_1".to_string(),
                "artifact_b".to_string(),
                base_artifacts(),
                vec![LearnTargetEngine::Prune, LearnTargetEngine::Know],
                true,
                true,
                true,
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        match out {
            Ph1LearnResponse::LearnArtifactPackageBuildOk(ok) => {
                assert_eq!(ok.validation_status, LearnValidationStatus::Ok);
            }
            _ => panic!("expected LearnArtifactPackageBuildOk"),
        }
    }
}
