#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1cache::{
    CacheCapabilityId, CacheHintSnapshotReadOk, CacheHintSnapshotReadRequest,
    CacheHintSnapshotRefreshOk, CacheHintSnapshotRefreshRequest, CacheRefuse, CacheRouteHint,
    CacheValidationStatus, Ph1CacheRequest, Ph1CacheResponse,
};
use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.CACHE OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_CACHE_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4348_0101);
    pub const PH1_CACHE_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4348_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1CacheWiringConfig {
    pub cache_enabled: bool,
    pub max_skeletons: u8,
    pub max_diagnostics: u8,
}

impl Ph1CacheWiringConfig {
    pub fn mvp_v1(cache_enabled: bool) -> Self {
        Self {
            cache_enabled,
            max_skeletons: 4,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub intent_type: String,
    pub environment_profile_ref: String,
    pub persona_profile_ref: Option<String>,
    pub route_budget_hint: Option<CacheRouteHint>,
    pub cache_policy_pack_id: Option<String>,
    pub policy_cache_enabled: bool,
    pub privacy_mode: bool,
    pub contains_ungoverned_artifacts: bool,
}

impl CacheTurnInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        intent_type: String,
        environment_profile_ref: String,
        persona_profile_ref: Option<String>,
        route_budget_hint: Option<CacheRouteHint>,
        cache_policy_pack_id: Option<String>,
        policy_cache_enabled: bool,
        privacy_mode: bool,
        contains_ungoverned_artifacts: bool,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            intent_type,
            environment_profile_ref,
            persona_profile_ref,
            route_budget_hint,
            cache_policy_pack_id,
            policy_cache_enabled,
            privacy_mode,
            contains_ungoverned_artifacts,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for CacheTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.intent_type.len() > 96 || self.intent_type.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "cache_turn_input.intent_type",
                reason: "must be <= 96 chars and contain no control chars",
            });
        }
        if self.environment_profile_ref.len() > 96
            || self.environment_profile_ref.chars().any(|c| c.is_control())
        {
            return Err(ContractViolation::InvalidValue {
                field: "cache_turn_input.environment_profile_ref",
                reason: "must be <= 96 chars and contain no control chars",
            });
        }
        if let Some(persona_profile_ref) = &self.persona_profile_ref {
            if persona_profile_ref.len() > 96 || persona_profile_ref.chars().any(|c| c.is_control())
            {
                return Err(ContractViolation::InvalidValue {
                    field: "cache_turn_input.persona_profile_ref",
                    reason: "must be <= 96 chars and contain no control chars",
                });
            }
        }
        if let Some(cache_policy_pack_id) = &self.cache_policy_pack_id {
            if cache_policy_pack_id.len() > 128
                || cache_policy_pack_id.chars().any(|c| c.is_control())
            {
                return Err(ContractViolation::InvalidValue {
                    field: "cache_turn_input.cache_policy_pack_id",
                    reason: "must be <= 128 chars and contain no control chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub snapshot_read: CacheHintSnapshotReadOk,
    pub snapshot_refresh: CacheHintSnapshotRefreshOk,
}

impl CacheForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        snapshot_read: CacheHintSnapshotReadOk,
        snapshot_refresh: CacheHintSnapshotRefreshOk,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            snapshot_read,
            snapshot_refresh,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Validate for CacheForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.snapshot_read.validate()?;
        self.snapshot_refresh.validate()?;
        if self.snapshot_refresh.validation_status != CacheValidationStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "cache_forward_bundle.snapshot_refresh.validation_status",
                reason: "must be OK",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheWiringOutcome {
    NotInvokedDisabled,
    NotInvokedNoCacheInput,
    NotInvokedPolicyDisabled,
    Refused(CacheRefuse),
    Forwarded(CacheForwardBundle),
}

pub trait Ph1CacheEngine {
    fn run(&self, req: &Ph1CacheRequest) -> Ph1CacheResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1CacheWiring<E>
where
    E: Ph1CacheEngine,
{
    config: Ph1CacheWiringConfig,
    engine: E,
}

impl<E> Ph1CacheWiring<E>
where
    E: Ph1CacheEngine,
{
    pub fn new(config: Ph1CacheWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_skeletons == 0 || config.max_skeletons > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1cache_wiring_config.max_skeletons",
                reason: "must be within 1..=8",
            });
        }
        if config.max_diagnostics == 0 || config.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1cache_wiring_config.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(
        &self,
        input: &CacheTurnInput,
    ) -> Result<CacheWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.cache_enabled {
            return Ok(CacheWiringOutcome::NotInvokedDisabled);
        }
        if !input.policy_cache_enabled {
            return Ok(CacheWiringOutcome::NotInvokedPolicyDisabled);
        }
        if input.intent_type.trim().is_empty() || input.environment_profile_ref.trim().is_empty() {
            return Ok(CacheWiringOutcome::NotInvokedNoCacheInput);
        }

        let envelope = selene_kernel_contracts::ph1cache::CacheRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_skeletons, 8),
            min(self.config.max_diagnostics, 16),
        )?;

        let read_req = Ph1CacheRequest::CacheHintSnapshotRead(CacheHintSnapshotReadRequest::v1(
            envelope.clone(),
            input.intent_type.clone(),
            input.environment_profile_ref.clone(),
            input.persona_profile_ref.clone(),
            input.route_budget_hint,
            input.cache_policy_pack_id.clone(),
            true,
            input.privacy_mode,
        )?);
        let read_resp = self.engine.run(&read_req);
        read_resp.validate()?;

        let read_ok = match read_resp {
            Ph1CacheResponse::Refuse(refuse) => return Ok(CacheWiringOutcome::Refused(refuse)),
            Ph1CacheResponse::CacheHintSnapshotReadOk(ok) => ok,
            Ph1CacheResponse::CacheHintSnapshotRefreshOk(_) => {
                return Ok(CacheWiringOutcome::Refused(CacheRefuse::v1(
                    CacheCapabilityId::CacheHintSnapshotRead,
                    reason_codes::PH1_CACHE_INTERNAL_PIPELINE_ERROR,
                    "unexpected snapshot-refresh response for snapshot-read request".to_string(),
                )?));
            }
        };

        let refresh_req =
            Ph1CacheRequest::CacheHintSnapshotRefresh(CacheHintSnapshotRefreshRequest::v1(
                envelope,
                input.intent_type.clone(),
                input.environment_profile_ref.clone(),
                input.persona_profile_ref.clone(),
                input.route_budget_hint,
                input.cache_policy_pack_id.clone(),
                true,
                input.privacy_mode,
                read_ok.selected_skeleton_id.clone(),
                read_ok.ordered_skeletons.clone(),
                input.contains_ungoverned_artifacts,
            )?);
        let refresh_resp = self.engine.run(&refresh_req);
        refresh_resp.validate()?;

        let refresh_ok = match refresh_resp {
            Ph1CacheResponse::Refuse(refuse) => return Ok(CacheWiringOutcome::Refused(refuse)),
            Ph1CacheResponse::CacheHintSnapshotRefreshOk(ok) => ok,
            Ph1CacheResponse::CacheHintSnapshotReadOk(_) => {
                return Ok(CacheWiringOutcome::Refused(CacheRefuse::v1(
                    CacheCapabilityId::CacheHintSnapshotRefresh,
                    reason_codes::PH1_CACHE_INTERNAL_PIPELINE_ERROR,
                    "unexpected snapshot-read response for snapshot-refresh request".to_string(),
                )?));
            }
        };

        if refresh_ok.validation_status != CacheValidationStatus::Ok {
            return Ok(CacheWiringOutcome::Refused(CacheRefuse::v1(
                CacheCapabilityId::CacheHintSnapshotRefresh,
                reason_codes::PH1_CACHE_VALIDATION_FAILED,
                "cache snapshot refresh validation failed".to_string(),
            )?));
        }

        let bundle =
            CacheForwardBundle::v1(input.correlation_id, input.turn_id, read_ok, refresh_ok)?;
        Ok(CacheWiringOutcome::Forwarded(bundle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1cache::{
        CacheDeliveryHint, CacheHintSnapshotReadOk, CacheHintSnapshotRefreshOk, CacheMoveKind,
        CachePlanSkeleton,
    };
    use selene_kernel_contracts::ReasonCodeId;

    struct DeterministicCacheEngine;

    impl Ph1CacheEngine for DeterministicCacheEngine {
        fn run(&self, req: &Ph1CacheRequest) -> Ph1CacheResponse {
            match req {
                Ph1CacheRequest::CacheHintSnapshotRead(r) => {
                    let skeleton = CachePlanSkeleton::v1(
                        "cache:primary:test".to_string(),
                        r.intent_type.clone(),
                        r.environment_profile_ref.clone(),
                        r.persona_profile_ref.clone(),
                        CacheMoveKind::Respond,
                        CacheRouteHint::Standard,
                        CacheDeliveryHint::VoiceAllowed,
                        false,
                        true,
                        true,
                        300,
                        r.cache_policy_pack_id.clone(),
                        "cache:evidence:test".to_string(),
                    )
                    .unwrap();

                    Ph1CacheResponse::CacheHintSnapshotReadOk(
                        CacheHintSnapshotReadOk::v1(
                            ReasonCodeId(1),
                            skeleton.skeleton_id.clone(),
                            vec![skeleton],
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1CacheRequest::CacheHintSnapshotRefresh(_r) => {
                    Ph1CacheResponse::CacheHintSnapshotRefreshOk(
                        CacheHintSnapshotRefreshOk::v1(
                            ReasonCodeId(2),
                            CacheValidationStatus::Ok,
                            vec![],
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    struct DriftCacheEngine;

    impl Ph1CacheEngine for DriftCacheEngine {
        fn run(&self, req: &Ph1CacheRequest) -> Ph1CacheResponse {
            match req {
                Ph1CacheRequest::CacheHintSnapshotRead(r) => {
                    let skeleton = CachePlanSkeleton::v1(
                        "cache:primary:test".to_string(),
                        r.intent_type.clone(),
                        r.environment_profile_ref.clone(),
                        r.persona_profile_ref.clone(),
                        CacheMoveKind::Respond,
                        CacheRouteHint::Standard,
                        CacheDeliveryHint::VoiceAllowed,
                        false,
                        true,
                        true,
                        300,
                        r.cache_policy_pack_id.clone(),
                        "cache:evidence:test".to_string(),
                    )
                    .unwrap();

                    Ph1CacheResponse::CacheHintSnapshotReadOk(
                        CacheHintSnapshotReadOk::v1(
                            ReasonCodeId(11),
                            skeleton.skeleton_id.clone(),
                            vec![skeleton],
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1CacheRequest::CacheHintSnapshotRefresh(_r) => {
                    Ph1CacheResponse::CacheHintSnapshotRefreshOk(
                        CacheHintSnapshotRefreshOk::v1(
                            ReasonCodeId(12),
                            CacheValidationStatus::Fail,
                            vec!["payload_mismatch".to_string()],
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    fn input() -> CacheTurnInput {
        CacheTurnInput::v1(
            CorrelationId(8301),
            TurnId(351),
            "QUERY_TIME".to_string(),
            "desktop".to_string(),
            Some("persona_short".to_string()),
            Some(CacheRouteHint::Standard),
            Some("artifact_cache_pack_v1".to_string()),
            true,
            false,
            false,
        )
        .unwrap()
    }

    #[test]
    fn at_cache_01_os_invokes_and_returns_schema_valid_bundle() {
        let wiring =
            Ph1CacheWiring::new(Ph1CacheWiringConfig::mvp_v1(true), DeterministicCacheEngine)
                .unwrap();

        let out = wiring.run_turn(&input()).unwrap();
        match out {
            CacheWiringOutcome::Forwarded(bundle) => {
                assert!(bundle.validate().is_ok());
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_cache_02_os_forwarded_bundle_is_deterministic() {
        let wiring =
            Ph1CacheWiring::new(Ph1CacheWiringConfig::mvp_v1(true), DeterministicCacheEngine)
                .unwrap();

        let out1 = wiring.run_turn(&input()).unwrap();
        let out2 = wiring.run_turn(&input()).unwrap();

        match (out1, out2) {
            (CacheWiringOutcome::Forwarded(a), CacheWiringOutcome::Forwarded(b)) => {
                assert_eq!(a.snapshot_read, b.snapshot_read);
                assert_eq!(a.snapshot_refresh, b.snapshot_refresh);
            }
            _ => panic!("expected Forwarded outcomes"),
        }
    }

    #[test]
    fn at_cache_03_os_policy_disable_short_circuits() {
        let wiring = Ph1CacheWiring::new(
            Ph1CacheWiringConfig::mvp_v1(false),
            DeterministicCacheEngine,
        )
        .unwrap();

        let out = wiring.run_turn(&input()).unwrap();
        assert_eq!(out, CacheWiringOutcome::NotInvokedDisabled);
    }

    #[test]
    fn at_cache_04_os_validation_fail_is_refused() {
        let wiring =
            Ph1CacheWiring::new(Ph1CacheWiringConfig::mvp_v1(true), DriftCacheEngine).unwrap();

        let out = wiring.run_turn(&input()).unwrap();
        match out {
            CacheWiringOutcome::Refused(refuse) => {
                assert_eq!(
                    refuse.capability_id,
                    CacheCapabilityId::CacheHintSnapshotRefresh
                );
            }
            _ => panic!("expected Refused"),
        }
    }
}
