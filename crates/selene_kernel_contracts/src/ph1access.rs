#![forbid(unsafe_code)]

use crate::{ContractViolation, SchemaVersion, Validate};

pub const PH1ACCESS_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

pub const ACCESS_AP_SCHEMA_CREATE_DRAFT: &str = "ACCESS_AP_SCHEMA_CREATE_DRAFT";
pub const ACCESS_AP_SCHEMA_UPDATE_COMMIT: &str = "ACCESS_AP_SCHEMA_UPDATE_COMMIT";
pub const ACCESS_AP_SCHEMA_ACTIVATE_COMMIT: &str = "ACCESS_AP_SCHEMA_ACTIVATE_COMMIT";
pub const ACCESS_AP_SCHEMA_RETIRE_COMMIT: &str = "ACCESS_AP_SCHEMA_RETIRE_COMMIT";
pub const ACCESS_AP_OVERLAY_UPDATE_COMMIT: &str = "ACCESS_AP_OVERLAY_UPDATE_COMMIT";
pub const ACCESS_BOARD_POLICY_UPDATE_COMMIT: &str = "ACCESS_BOARD_POLICY_UPDATE_COMMIT";
pub const ACCESS_INSTANCE_COMPILE_COMMIT: &str = "ACCESS_INSTANCE_COMPILE_COMMIT";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessProfileScope {
    Global,
    Tenant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AccessProfileLifecycleState {
    Draft,
    Active,
    Retired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessOverlayOperation {
    AddPermission,
    RemovePermission,
    TightenConstraint,
    SetEscalationPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessApprovalPrimitive {
    SingleApprover,
    NOfM,
    BoardQuorumPercent,
    UnanimousBoard,
    Mixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessApReviewChannel {
    PhoneDesktop,
    ReadOutLoud,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessApRuleReviewAction {
    Agree,
    Disagree,
    Edit,
    Delete,
    Disable,
    AddCustomRule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessApAuthoringConfirmationState {
    NeedsChannelChoice,
    ReviewInProgress,
    PendingActivationConfirmation,
    ConfirmedForActivation,
    Declined,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AccessProfileId(String);

impl AccessProfileId {
    pub fn new(id: impl Into<String>) -> Result<Self, ContractViolation> {
        let id = id.into();
        let v = Self(id);
        v.validate()?;
        Ok(v)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for AccessProfileId {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id("access_profile_id", &self.0, 64)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AccessOverlayId(String);

impl AccessOverlayId {
    pub fn new(id: impl Into<String>) -> Result<Self, ContractViolation> {
        let id = id.into();
        let v = Self(id);
        v.validate()?;
        Ok(v)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for AccessOverlayId {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id("access_overlay_id", &self.0, 64)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoardId(String);

impl BoardId {
    pub fn new(id: impl Into<String>) -> Result<Self, ContractViolation> {
        let id = id.into();
        let v = Self(id);
        v.validate()?;
        Ok(v)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for BoardId {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id("board_id", &self.0, 64)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessProfileVersionRef {
    pub access_profile_id: AccessProfileId,
    pub schema_version_id: String,
}

impl Validate for AccessProfileVersionRef {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.access_profile_id.validate()?;
        validate_id(
            "access_profile_version_ref.schema_version_id",
            &self.schema_version_id,
            64,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessProfileSchemaRecord {
    pub schema_version: SchemaVersion,
    pub access_profile_id: AccessProfileId,
    pub profile_name: String,
    pub scope: AccessProfileScope,
    pub lifecycle_state: AccessProfileLifecycleState,
    pub owner_tenant_id: Option<String>,
    pub derived_from_global_profile_id: Option<AccessProfileId>,
    pub capability_allow_list: Vec<String>,
}

impl Validate for AccessProfileSchemaRecord {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "access_profile_schema_record.schema_version",
                reason: "must be > 0",
            });
        }
        self.access_profile_id.validate()?;
        validate_text(
            "access_profile_schema_record.profile_name",
            &self.profile_name,
            128,
        )?;
        match self.scope {
            AccessProfileScope::Global => {
                if self.owner_tenant_id.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "access_profile_schema_record.owner_tenant_id",
                        reason: "must be absent when scope=Global",
                    });
                }
            }
            AccessProfileScope::Tenant => {
                let Some(owner_tenant_id) = &self.owner_tenant_id else {
                    return Err(ContractViolation::InvalidValue {
                        field: "access_profile_schema_record.owner_tenant_id",
                        reason: "must be present when scope=Tenant",
                    });
                };
                validate_id(
                    "access_profile_schema_record.owner_tenant_id",
                    owner_tenant_id,
                    64,
                )?;
            }
        }
        if let Some(global_ref) = &self.derived_from_global_profile_id {
            global_ref.validate()?;
        }
        if self.capability_allow_list.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "access_profile_schema_record.capability_allow_list",
                reason: "must not be empty",
            });
        }
        for capability_id in &self.capability_allow_list {
            validate_id(
                "access_profile_schema_record.capability_allow_list[]",
                capability_id,
                96,
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessOverlayOpSpec {
    pub op: AccessOverlayOperation,
    pub capability_id: Option<String>,
    pub constraint_ref: Option<String>,
    pub escalation_policy_ref: Option<String>,
}

impl Validate for AccessOverlayOpSpec {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self.op {
            AccessOverlayOperation::AddPermission | AccessOverlayOperation::RemovePermission => {
                let Some(capability_id) = &self.capability_id else {
                    return Err(ContractViolation::InvalidValue {
                        field: "access_overlay_op_spec.capability_id",
                        reason: "must be present for add/remove permission ops",
                    });
                };
                validate_id("access_overlay_op_spec.capability_id", capability_id, 96)?;
            }
            AccessOverlayOperation::TightenConstraint => {
                let Some(constraint_ref) = &self.constraint_ref else {
                    return Err(ContractViolation::InvalidValue {
                        field: "access_overlay_op_spec.constraint_ref",
                        reason: "must be present for tighten constraint op",
                    });
                };
                validate_id("access_overlay_op_spec.constraint_ref", constraint_ref, 128)?;
            }
            AccessOverlayOperation::SetEscalationPolicy => {
                let Some(escalation_policy_ref) = &self.escalation_policy_ref else {
                    return Err(ContractViolation::InvalidValue {
                        field: "access_overlay_op_spec.escalation_policy_ref",
                        reason: "must be present for set escalation policy op",
                    });
                };
                validate_id(
                    "access_overlay_op_spec.escalation_policy_ref",
                    escalation_policy_ref,
                    128,
                )?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessApprovalPolicySpec {
    pub primitive: AccessApprovalPrimitive,
    pub required_approvals: Option<u16>,
    pub approver_pool_size: Option<u16>,
    pub board_quorum_percent: Option<u8>,
    pub require_cfo_approval: bool,
}

impl Validate for AccessApprovalPolicySpec {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self.primitive {
            AccessApprovalPrimitive::SingleApprover => {
                if let Some(required) = self.required_approvals {
                    if required != 1 {
                        return Err(ContractViolation::InvalidValue {
                            field: "access_approval_policy_spec.required_approvals",
                            reason: "must be 1 when primitive=SingleApprover",
                        });
                    }
                }
                if self.approver_pool_size.is_some() || self.board_quorum_percent.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "access_approval_policy_spec",
                        reason: "pool/quorum must be absent when primitive=SingleApprover",
                    });
                }
            }
            AccessApprovalPrimitive::NOfM => {
                let Some(required) = self.required_approvals else {
                    return Err(ContractViolation::InvalidValue {
                        field: "access_approval_policy_spec.required_approvals",
                        reason: "must be present when primitive=NOfM",
                    });
                };
                let Some(pool) = self.approver_pool_size else {
                    return Err(ContractViolation::InvalidValue {
                        field: "access_approval_policy_spec.approver_pool_size",
                        reason: "must be present when primitive=NOfM",
                    });
                };
                if required == 0 || pool == 0 || required > pool {
                    return Err(ContractViolation::InvalidValue {
                        field: "access_approval_policy_spec.required_approvals",
                        reason: "must satisfy 1 <= required_approvals <= approver_pool_size",
                    });
                }
                if self.board_quorum_percent.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "access_approval_policy_spec.board_quorum_percent",
                        reason: "must be absent when primitive=NOfM",
                    });
                }
            }
            AccessApprovalPrimitive::BoardQuorumPercent => {
                let Some(percent) = self.board_quorum_percent else {
                    return Err(ContractViolation::InvalidValue {
                        field: "access_approval_policy_spec.board_quorum_percent",
                        reason: "must be present when primitive=BoardQuorumPercent",
                    });
                };
                if percent == 0 || percent > 100 {
                    return Err(ContractViolation::InvalidRange {
                        field: "access_approval_policy_spec.board_quorum_percent",
                        min: 1.0,
                        max: 100.0,
                        got: percent as f64,
                    });
                }
                if self.required_approvals.is_some() || self.approver_pool_size.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "access_approval_policy_spec",
                        reason: "required/pool must be absent when primitive=BoardQuorumPercent",
                    });
                }
            }
            AccessApprovalPrimitive::UnanimousBoard => {
                if self.required_approvals.is_some()
                    || self.approver_pool_size.is_some()
                    || self.board_quorum_percent.is_some()
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "access_approval_policy_spec",
                        reason: "required/pool/quorum must be absent when primitive=UnanimousBoard",
                    });
                }
            }
            AccessApprovalPrimitive::Mixed => {
                let has_policy_signal = self.required_approvals.is_some()
                    || self.board_quorum_percent.is_some()
                    || self.require_cfo_approval;
                if !has_policy_signal {
                    return Err(ContractViolation::InvalidValue {
                        field: "access_approval_policy_spec",
                        reason: "mixed policy must define at least one approval signal",
                    });
                }
                if let Some(percent) = self.board_quorum_percent {
                    if percent == 0 || percent > 100 {
                        return Err(ContractViolation::InvalidRange {
                            field: "access_approval_policy_spec.board_quorum_percent",
                            min: 1.0,
                            max: 100.0,
                            got: percent as f64,
                        });
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessCompiledLineageRef {
    pub global_profile_version: AccessProfileVersionRef,
    pub tenant_profile_version: Option<AccessProfileVersionRef>,
    pub overlay_version_ids: Vec<AccessOverlayId>,
    pub position_id: Option<String>,
}

impl Validate for AccessCompiledLineageRef {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.global_profile_version.validate()?;
        if let Some(v) = &self.tenant_profile_version {
            v.validate()?;
        }
        if self.overlay_version_ids.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "access_compiled_lineage_ref.overlay_version_ids",
                reason: "must contain <= 32 overlay version references",
            });
        }
        for overlay_id in &self.overlay_version_ids {
            overlay_id.validate()?;
        }
        if let Some(position_id) = &self.position_id {
            validate_id("access_compiled_lineage_ref.position_id", position_id, 64)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessApRuleReviewActionPayload {
    pub action: AccessApRuleReviewAction,
    pub suggested_rule_ref: Option<String>,
    pub capability_id: Option<String>,
    pub constraint_ref: Option<String>,
    pub escalation_policy_ref: Option<String>,
}

impl Validate for AccessApRuleReviewActionPayload {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self.action {
            AccessApRuleReviewAction::AddCustomRule => {
                if self.suggested_rule_ref.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "access_ap_rule_review_action_payload.suggested_rule_ref",
                        reason: "must be absent when action=AddCustomRule",
                    });
                }
                let Some(capability_id) = &self.capability_id else {
                    return Err(ContractViolation::InvalidValue {
                        field: "access_ap_rule_review_action_payload.capability_id",
                        reason: "must be present when action=AddCustomRule",
                    });
                };
                validate_id(
                    "access_ap_rule_review_action_payload.capability_id",
                    capability_id,
                    96,
                )?;
            }
            AccessApRuleReviewAction::Edit => {
                let Some(suggested_rule_ref) = &self.suggested_rule_ref else {
                    return Err(ContractViolation::InvalidValue {
                        field: "access_ap_rule_review_action_payload.suggested_rule_ref",
                        reason: "must be present when action=Edit",
                    });
                };
                validate_id(
                    "access_ap_rule_review_action_payload.suggested_rule_ref",
                    suggested_rule_ref,
                    96,
                )?;
                let Some(capability_id) = &self.capability_id else {
                    return Err(ContractViolation::InvalidValue {
                        field: "access_ap_rule_review_action_payload.capability_id",
                        reason: "must be present when action=Edit",
                    });
                };
                validate_id(
                    "access_ap_rule_review_action_payload.capability_id",
                    capability_id,
                    96,
                )?;
            }
            AccessApRuleReviewAction::Agree
            | AccessApRuleReviewAction::Disagree
            | AccessApRuleReviewAction::Delete
            | AccessApRuleReviewAction::Disable => {
                let Some(suggested_rule_ref) = &self.suggested_rule_ref else {
                    return Err(ContractViolation::InvalidValue {
                        field: "access_ap_rule_review_action_payload.suggested_rule_ref",
                        reason: "must be present for agree/disagree/delete/disable actions",
                    });
                };
                validate_id(
                    "access_ap_rule_review_action_payload.suggested_rule_ref",
                    suggested_rule_ref,
                    96,
                )?;
            }
        }

        if let Some(constraint_ref) = &self.constraint_ref {
            validate_id(
                "access_ap_rule_review_action_payload.constraint_ref",
                constraint_ref,
                128,
            )?;
        }
        if let Some(escalation_policy_ref) = &self.escalation_policy_ref {
            validate_id(
                "access_ap_rule_review_action_payload.escalation_policy_ref",
                escalation_policy_ref,
                128,
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessApAuthoringReviewState {
    pub access_profile_id: AccessProfileId,
    pub schema_version_id: String,
    pub review_channel: AccessApReviewChannel,
    pub confirmation_state: AccessApAuthoringConfirmationState,
}

impl Validate for AccessApAuthoringReviewState {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.access_profile_id.validate()?;
        validate_id(
            "access_ap_authoring_review_state.schema_version_id",
            &self.schema_version_id,
            64,
        )?;
        Ok(())
    }
}

fn validate_id(field: &'static str, s: &str, max_len: usize) -> Result<(), ContractViolation> {
    if s.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if s.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "too long",
        });
    }
    if !s.is_ascii() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be ASCII",
        });
    }
    Ok(())
}

fn validate_text(field: &'static str, s: &str, max_len: usize) -> Result<(), ContractViolation> {
    if s.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if s.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "too long",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tenant_profile_requires_owner_tenant() {
        let profile = AccessProfileSchemaRecord {
            schema_version: PH1ACCESS_CONTRACT_VERSION,
            access_profile_id: AccessProfileId::new("ap_clerk").unwrap(),
            profile_name: "AP Clerk".to_string(),
            scope: AccessProfileScope::Tenant,
            lifecycle_state: AccessProfileLifecycleState::Draft,
            owner_tenant_id: None,
            derived_from_global_profile_id: None,
            capability_allow_list: vec!["CAN_VIEW_SHIFT".to_string()],
        };
        assert!(profile.validate().is_err());
    }

    #[test]
    fn nofm_policy_must_have_valid_bounds() {
        let policy = AccessApprovalPolicySpec {
            primitive: AccessApprovalPrimitive::NOfM,
            required_approvals: Some(3),
            approver_pool_size: Some(2),
            board_quorum_percent: None,
            require_cfo_approval: false,
        };
        assert!(policy.validate().is_err());
    }

    #[test]
    fn board_quorum_percent_must_be_in_range() {
        let policy = AccessApprovalPolicySpec {
            primitive: AccessApprovalPrimitive::BoardQuorumPercent,
            required_approvals: None,
            approver_pool_size: None,
            board_quorum_percent: Some(0),
            require_cfo_approval: false,
        };
        assert!(policy.validate().is_err());
    }

    #[test]
    fn tighten_constraint_overlay_requires_constraint_ref() {
        let overlay_op = AccessOverlayOpSpec {
            op: AccessOverlayOperation::TightenConstraint,
            capability_id: None,
            constraint_ref: None,
            escalation_policy_ref: None,
        };
        assert!(overlay_op.validate().is_err());
    }

    #[test]
    fn lineage_accepts_optional_tenant_profile_and_position() {
        let lineage = AccessCompiledLineageRef {
            global_profile_version: AccessProfileVersionRef {
                access_profile_id: AccessProfileId::new("ap_driver").unwrap(),
                schema_version_id: "v1".to_string(),
            },
            tenant_profile_version: Some(AccessProfileVersionRef {
                access_profile_id: AccessProfileId::new("ap_driver_tenant").unwrap(),
                schema_version_id: "v3".to_string(),
            }),
            overlay_version_ids: vec![AccessOverlayId::new("overlay_1").unwrap()],
            position_id: Some("position_driver".to_string()),
        };
        assert!(lineage.validate().is_ok());
    }

    #[test]
    fn rule_review_action_requires_suggested_rule_for_agree() {
        let action = AccessApRuleReviewActionPayload {
            action: AccessApRuleReviewAction::Agree,
            suggested_rule_ref: None,
            capability_id: None,
            constraint_ref: None,
            escalation_policy_ref: None,
        };
        assert!(action.validate().is_err());
    }

    #[test]
    fn rule_review_action_edit_requires_capability() {
        let action = AccessApRuleReviewActionPayload {
            action: AccessApRuleReviewAction::Edit,
            suggested_rule_ref: Some("rule_001".to_string()),
            capability_id: None,
            constraint_ref: None,
            escalation_policy_ref: None,
        };
        assert!(action.validate().is_err());
    }

    #[test]
    fn rule_review_action_add_custom_requires_capability_and_no_ref() {
        let action = AccessApRuleReviewActionPayload {
            action: AccessApRuleReviewAction::AddCustomRule,
            suggested_rule_ref: Some("rule_001".to_string()),
            capability_id: Some("CAN_VIEW_DASHBOARD".to_string()),
            constraint_ref: None,
            escalation_policy_ref: None,
        };
        assert!(action.validate().is_err());
    }

    #[test]
    fn authoring_review_state_validates_with_channel_and_confirmation_state() {
        let state = AccessApAuthoringReviewState {
            access_profile_id: AccessProfileId::new("ap_clerk").unwrap(),
            schema_version_id: "v1".to_string(),
            review_channel: AccessApReviewChannel::PhoneDesktop,
            confirmation_state: AccessApAuthoringConfirmationState::ReviewInProgress,
        };
        assert!(state.validate().is_ok());
    }
}
