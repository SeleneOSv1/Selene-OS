#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1CACHE_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheCapabilityId {
    CacheHintSnapshotRead,
    CacheHintSnapshotRefresh,
}

impl CacheCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            CacheCapabilityId::CacheHintSnapshotRead => "CACHE_HINT_SNAPSHOT_READ",
            CacheCapabilityId::CacheHintSnapshotRefresh => "CACHE_HINT_SNAPSHOT_REFRESH",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheMoveKind {
    Respond,
    ClarifyOneQuestion,
    Confirm,
    DispatchReadOnlyTool,
    Explain,
    Wait,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheRouteHint {
    Standard,
    FastTrack,
    CostSaver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheDeliveryHint {
    VoiceAllowed,
    TextRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_skeletons: u8,
    pub max_diagnostics: u8,
}

impl CacheRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_skeletons: u8,
        max_diagnostics: u8,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1CACHE_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_skeletons,
            max_diagnostics,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for CacheRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1CACHE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "cache_request_envelope.schema_version",
                reason: "must match PH1CACHE_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_skeletons == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "cache_request_envelope.max_skeletons",
                reason: "must be > 0",
            });
        }
        if self.max_skeletons > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "cache_request_envelope.max_skeletons",
                reason: "must be <= 8",
            });
        }
        if self.max_diagnostics == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "cache_request_envelope.max_diagnostics",
                reason: "must be > 0",
            });
        }
        if self.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "cache_request_envelope.max_diagnostics",
                reason: "must be <= 16",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachePlanSkeleton {
    pub schema_version: SchemaVersion,
    pub skeleton_id: String,
    pub intent_type: String,
    pub environment_profile_ref: String,
    pub persona_profile_ref: Option<String>,
    pub suggested_move: CacheMoveKind,
    pub route_hint: CacheRouteHint,
    pub delivery_hint: CacheDeliveryHint,
    pub prefetch_hint_enabled: bool,
    pub requires_access_gate: bool,
    pub requires_simulation_gate: bool,
    pub ttl_seconds: u16,
    pub cache_policy_pack_id: Option<String>,
    pub evidence_ref: String,
}

impl CachePlanSkeleton {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        skeleton_id: String,
        intent_type: String,
        environment_profile_ref: String,
        persona_profile_ref: Option<String>,
        suggested_move: CacheMoveKind,
        route_hint: CacheRouteHint,
        delivery_hint: CacheDeliveryHint,
        prefetch_hint_enabled: bool,
        requires_access_gate: bool,
        requires_simulation_gate: bool,
        ttl_seconds: u16,
        cache_policy_pack_id: Option<String>,
        evidence_ref: String,
    ) -> Result<Self, ContractViolation> {
        let skeleton = Self {
            schema_version: PH1CACHE_CONTRACT_VERSION,
            skeleton_id,
            intent_type,
            environment_profile_ref,
            persona_profile_ref,
            suggested_move,
            route_hint,
            delivery_hint,
            prefetch_hint_enabled,
            requires_access_gate,
            requires_simulation_gate,
            ttl_seconds,
            cache_policy_pack_id,
            evidence_ref,
        };
        skeleton.validate()?;
        Ok(skeleton)
    }
}

impl Validate for CachePlanSkeleton {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1CACHE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "cache_plan_skeleton.schema_version",
                reason: "must match PH1CACHE_CONTRACT_VERSION",
            });
        }
        validate_token("cache_plan_skeleton.skeleton_id", &self.skeleton_id, 128)?;
        validate_text("cache_plan_skeleton.intent_type", &self.intent_type, 96)?;
        validate_text(
            "cache_plan_skeleton.environment_profile_ref",
            &self.environment_profile_ref,
            96,
        )?;
        if let Some(persona_profile_ref) = &self.persona_profile_ref {
            validate_text(
                "cache_plan_skeleton.persona_profile_ref",
                persona_profile_ref,
                96,
            )?;
        }
        if !self.requires_access_gate {
            return Err(ContractViolation::InvalidValue {
                field: "cache_plan_skeleton.requires_access_gate",
                reason: "must be true (cache cannot bypass access gate)",
            });
        }
        if !self.requires_simulation_gate {
            return Err(ContractViolation::InvalidValue {
                field: "cache_plan_skeleton.requires_simulation_gate",
                reason: "must be true (cache cannot bypass simulation gate)",
            });
        }
        if self.ttl_seconds < 30 {
            return Err(ContractViolation::InvalidValue {
                field: "cache_plan_skeleton.ttl_seconds",
                reason: "must be >= 30",
            });
        }
        if self.ttl_seconds > 3_600 {
            return Err(ContractViolation::InvalidValue {
                field: "cache_plan_skeleton.ttl_seconds",
                reason: "must be <= 3600",
            });
        }
        if let Some(cache_policy_pack_id) = &self.cache_policy_pack_id {
            validate_token(
                "cache_plan_skeleton.cache_policy_pack_id",
                cache_policy_pack_id,
                128,
            )?;
        }
        validate_token("cache_plan_skeleton.evidence_ref", &self.evidence_ref, 128)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheHintSnapshotReadRequest {
    pub schema_version: SchemaVersion,
    pub envelope: CacheRequestEnvelope,
    pub intent_type: String,
    pub environment_profile_ref: String,
    pub persona_profile_ref: Option<String>,
    pub route_budget_hint: Option<CacheRouteHint>,
    pub cache_policy_pack_id: Option<String>,
    pub policy_cache_enabled: bool,
    pub privacy_mode: bool,
}

impl CacheHintSnapshotReadRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: CacheRequestEnvelope,
        intent_type: String,
        environment_profile_ref: String,
        persona_profile_ref: Option<String>,
        route_budget_hint: Option<CacheRouteHint>,
        cache_policy_pack_id: Option<String>,
        policy_cache_enabled: bool,
        privacy_mode: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1CACHE_CONTRACT_VERSION,
            envelope,
            intent_type,
            environment_profile_ref,
            persona_profile_ref,
            route_budget_hint,
            cache_policy_pack_id,
            policy_cache_enabled,
            privacy_mode,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for CacheHintSnapshotReadRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1CACHE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "cache_hint_snapshot_read_request.schema_version",
                reason: "must match PH1CACHE_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_text(
            "cache_hint_snapshot_read_request.intent_type",
            &self.intent_type,
            96,
        )?;
        validate_text(
            "cache_hint_snapshot_read_request.environment_profile_ref",
            &self.environment_profile_ref,
            96,
        )?;
        if let Some(persona_profile_ref) = &self.persona_profile_ref {
            validate_text(
                "cache_hint_snapshot_read_request.persona_profile_ref",
                persona_profile_ref,
                96,
            )?;
        }
        if let Some(cache_policy_pack_id) = &self.cache_policy_pack_id {
            validate_token(
                "cache_hint_snapshot_read_request.cache_policy_pack_id",
                cache_policy_pack_id,
                128,
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheHintSnapshotRefreshRequest {
    pub schema_version: SchemaVersion,
    pub envelope: CacheRequestEnvelope,
    pub intent_type: String,
    pub environment_profile_ref: String,
    pub persona_profile_ref: Option<String>,
    pub route_budget_hint: Option<CacheRouteHint>,
    pub cache_policy_pack_id: Option<String>,
    pub policy_cache_enabled: bool,
    pub privacy_mode: bool,
    pub selected_skeleton_id: String,
    pub ordered_skeletons: Vec<CachePlanSkeleton>,
    pub contains_ungoverned_artifacts: bool,
}

impl CacheHintSnapshotRefreshRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: CacheRequestEnvelope,
        intent_type: String,
        environment_profile_ref: String,
        persona_profile_ref: Option<String>,
        route_budget_hint: Option<CacheRouteHint>,
        cache_policy_pack_id: Option<String>,
        policy_cache_enabled: bool,
        privacy_mode: bool,
        selected_skeleton_id: String,
        ordered_skeletons: Vec<CachePlanSkeleton>,
        contains_ungoverned_artifacts: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1CACHE_CONTRACT_VERSION,
            envelope,
            intent_type,
            environment_profile_ref,
            persona_profile_ref,
            route_budget_hint,
            cache_policy_pack_id,
            policy_cache_enabled,
            privacy_mode,
            selected_skeleton_id,
            ordered_skeletons,
            contains_ungoverned_artifacts,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for CacheHintSnapshotRefreshRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1CACHE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "cache_hint_snapshot_refresh_request.schema_version",
                reason: "must match PH1CACHE_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_text(
            "cache_hint_snapshot_refresh_request.intent_type",
            &self.intent_type,
            96,
        )?;
        validate_text(
            "cache_hint_snapshot_refresh_request.environment_profile_ref",
            &self.environment_profile_ref,
            96,
        )?;
        if let Some(persona_profile_ref) = &self.persona_profile_ref {
            validate_text(
                "cache_hint_snapshot_refresh_request.persona_profile_ref",
                persona_profile_ref,
                96,
            )?;
        }
        if let Some(cache_policy_pack_id) = &self.cache_policy_pack_id {
            validate_token(
                "cache_hint_snapshot_refresh_request.cache_policy_pack_id",
                cache_policy_pack_id,
                128,
            )?;
        }
        validate_token(
            "cache_hint_snapshot_refresh_request.selected_skeleton_id",
            &self.selected_skeleton_id,
            128,
        )?;
        if self.ordered_skeletons.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "cache_hint_snapshot_refresh_request.ordered_skeletons",
                reason: "must be non-empty",
            });
        }
        if self.ordered_skeletons.len() > self.envelope.max_skeletons as usize {
            return Err(ContractViolation::InvalidValue {
                field: "cache_hint_snapshot_refresh_request.ordered_skeletons",
                reason: "must be <= envelope.max_skeletons",
            });
        }

        let mut seen = BTreeSet::new();
        for skeleton in &self.ordered_skeletons {
            skeleton.validate()?;
            if !seen.insert(skeleton.skeleton_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "cache_hint_snapshot_refresh_request.ordered_skeletons",
                    reason: "skeleton_id values must be unique",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1CacheRequest {
    CacheHintSnapshotRead(CacheHintSnapshotReadRequest),
    CacheHintSnapshotRefresh(CacheHintSnapshotRefreshRequest),
}

impl Validate for Ph1CacheRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1CacheRequest::CacheHintSnapshotRead(r) => r.validate(),
            Ph1CacheRequest::CacheHintSnapshotRefresh(r) => r.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheHintSnapshotReadOk {
    pub schema_version: SchemaVersion,
    pub capability_id: CacheCapabilityId,
    pub reason_code: ReasonCodeId,
    pub selected_skeleton_id: String,
    pub ordered_skeletons: Vec<CachePlanSkeleton>,
    pub advisory_only: bool,
    pub no_execution_authority: bool,
}

impl CacheHintSnapshotReadOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        selected_skeleton_id: String,
        ordered_skeletons: Vec<CachePlanSkeleton>,
        advisory_only: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let ok = Self {
            schema_version: PH1CACHE_CONTRACT_VERSION,
            capability_id: CacheCapabilityId::CacheHintSnapshotRead,
            reason_code,
            selected_skeleton_id,
            ordered_skeletons,
            advisory_only,
            no_execution_authority,
        };
        ok.validate()?;
        Ok(ok)
    }
}

impl Validate for CacheHintSnapshotReadOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1CACHE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "cache_hint_snapshot_read_ok.schema_version",
                reason: "must match PH1CACHE_CONTRACT_VERSION",
            });
        }
        if self.capability_id != CacheCapabilityId::CacheHintSnapshotRead {
            return Err(ContractViolation::InvalidValue {
                field: "cache_hint_snapshot_read_ok.capability_id",
                reason: "must be CACHE_HINT_SNAPSHOT_READ",
            });
        }
        validate_token(
            "cache_hint_snapshot_read_ok.selected_skeleton_id",
            &self.selected_skeleton_id,
            128,
        )?;
        if self.ordered_skeletons.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "cache_hint_snapshot_read_ok.ordered_skeletons",
                reason: "must be non-empty",
            });
        }
        if self.ordered_skeletons.len() > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "cache_hint_snapshot_read_ok.ordered_skeletons",
                reason: "must be <= 8",
            });
        }
        let mut seen = BTreeSet::new();
        for skeleton in &self.ordered_skeletons {
            skeleton.validate()?;
            if !seen.insert(skeleton.skeleton_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "cache_hint_snapshot_read_ok.ordered_skeletons",
                    reason: "skeleton_id values must be unique",
                });
            }
        }
        if !self
            .ordered_skeletons
            .iter()
            .any(|s| s.skeleton_id == self.selected_skeleton_id)
        {
            return Err(ContractViolation::InvalidValue {
                field: "cache_hint_snapshot_read_ok.selected_skeleton_id",
                reason: "must reference an ordered skeleton",
            });
        }
        if !self.advisory_only {
            return Err(ContractViolation::InvalidValue {
                field: "cache_hint_snapshot_read_ok.advisory_only",
                reason: "must be true",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "cache_hint_snapshot_read_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheHintSnapshotRefreshOk {
    pub schema_version: SchemaVersion,
    pub capability_id: CacheCapabilityId,
    pub reason_code: ReasonCodeId,
    pub validation_status: CacheValidationStatus,
    pub diagnostics: Vec<String>,
    pub all_artifacts_governed_active: bool,
    pub advisory_only: bool,
    pub no_execution_authority: bool,
}

impl CacheHintSnapshotRefreshOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        validation_status: CacheValidationStatus,
        diagnostics: Vec<String>,
        all_artifacts_governed_active: bool,
        advisory_only: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let ok = Self {
            schema_version: PH1CACHE_CONTRACT_VERSION,
            capability_id: CacheCapabilityId::CacheHintSnapshotRefresh,
            reason_code,
            validation_status,
            diagnostics,
            all_artifacts_governed_active,
            advisory_only,
            no_execution_authority,
        };
        ok.validate()?;
        Ok(ok)
    }
}

impl Validate for CacheHintSnapshotRefreshOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1CACHE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "cache_hint_snapshot_refresh_ok.schema_version",
                reason: "must match PH1CACHE_CONTRACT_VERSION",
            });
        }
        if self.capability_id != CacheCapabilityId::CacheHintSnapshotRefresh {
            return Err(ContractViolation::InvalidValue {
                field: "cache_hint_snapshot_refresh_ok.capability_id",
                reason: "must be CACHE_HINT_SNAPSHOT_REFRESH",
            });
        }
        if self.diagnostics.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "cache_hint_snapshot_refresh_ok.diagnostics",
                reason: "must be <= 16",
            });
        }
        for diagnostic in &self.diagnostics {
            validate_token("cache_hint_snapshot_refresh_ok.diagnostics", diagnostic, 96)?;
        }
        if self.validation_status == CacheValidationStatus::Ok
            && !self.all_artifacts_governed_active
        {
            return Err(ContractViolation::InvalidValue {
                field: "cache_hint_snapshot_refresh_ok.all_artifacts_governed_active",
                reason: "must be true when validation_status=OK",
            });
        }
        if !self.advisory_only {
            return Err(ContractViolation::InvalidValue {
                field: "cache_hint_snapshot_refresh_ok.advisory_only",
                reason: "must be true",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "cache_hint_snapshot_refresh_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: CacheCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl CacheRefuse {
    pub fn v1(
        capability_id: CacheCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let refuse = Self {
            schema_version: PH1CACHE_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        refuse.validate()?;
        Ok(refuse)
    }
}

impl Validate for CacheRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1CACHE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "cache_refuse.schema_version",
                reason: "must match PH1CACHE_CONTRACT_VERSION",
            });
        }
        validate_text("cache_refuse.message", &self.message, 192)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1CacheResponse {
    CacheHintSnapshotReadOk(CacheHintSnapshotReadOk),
    CacheHintSnapshotRefreshOk(CacheHintSnapshotRefreshOk),
    Refuse(CacheRefuse),
}

impl Validate for Ph1CacheResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1CacheResponse::CacheHintSnapshotReadOk(ok) => ok.validate(),
            Ph1CacheResponse::CacheHintSnapshotRefreshOk(ok) => ok.validate(),
            Ph1CacheResponse::Refuse(r) => r.validate(),
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

    fn envelope() -> CacheRequestEnvelope {
        CacheRequestEnvelope::v1(CorrelationId(7201), TurnId(211), 4, 6).unwrap()
    }

    fn skeleton(id: &str) -> CachePlanSkeleton {
        CachePlanSkeleton::v1(
            id.to_string(),
            "QUERY_WEATHER".to_string(),
            "office_quiet".to_string(),
            Some("persona_brief".to_string()),
            CacheMoveKind::DispatchReadOnlyTool,
            CacheRouteHint::Standard,
            CacheDeliveryHint::TextRequired,
            true,
            true,
            true,
            300,
            Some("artifact_cache_pack_v1".to_string()),
            format!("cache:evidence:{}", id),
        )
        .unwrap()
    }

    #[test]
    fn at_cache_contract_01_read_request_is_schema_valid() {
        let req = CacheHintSnapshotReadRequest::v1(
            envelope(),
            "QUERY_WEATHER".to_string(),
            "office_quiet".to_string(),
            Some("persona_brief".to_string()),
            Some(CacheRouteHint::Standard),
            Some("artifact_cache_pack_v1".to_string()),
            true,
            true,
        )
        .unwrap();
        assert!(req.validate().is_ok());
    }

    #[test]
    fn at_cache_contract_02_skeleton_cannot_bypass_gates() {
        let out = CachePlanSkeleton::v1(
            "cache:bad".to_string(),
            "QUERY_WEATHER".to_string(),
            "office_quiet".to_string(),
            None,
            CacheMoveKind::Respond,
            CacheRouteHint::Standard,
            CacheDeliveryHint::VoiceAllowed,
            false,
            true,
            false,
            300,
            None,
            "cache:evidence:bad".to_string(),
        );
        assert!(out.is_err());
    }

    #[test]
    fn at_cache_contract_03_refresh_ok_requires_governed_artifacts() {
        let out = CacheHintSnapshotRefreshOk::v1(
            ReasonCodeId(1),
            CacheValidationStatus::Ok,
            vec![],
            false,
            true,
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn at_cache_contract_04_refresh_request_enforces_budget() {
        let req = CacheHintSnapshotRefreshRequest::v1(
            envelope(),
            "QUERY_WEATHER".to_string(),
            "office_quiet".to_string(),
            None,
            None,
            None,
            true,
            false,
            "cache:s1".to_string(),
            vec![
                skeleton("cache:s1"),
                skeleton("cache:s2"),
                skeleton("cache:s3"),
                skeleton("cache:s4"),
                skeleton("cache:s5"),
            ],
            false,
        );
        assert!(req.is_err());
    }
}
