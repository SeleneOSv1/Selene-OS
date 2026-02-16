#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1_voice_id::{Ph1VoiceIdSimRequest, Ph1VoiceIdSimResponse};
use selene_kernel_contracts::ph1capreq::{
    CapabilityRequestAction, CapabilityRequestStatus, CapreqId, Ph1CapreqRequest, Ph1CapreqResponse,
};
use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1link::{Ph1LinkRequest, Ph1LinkResponse};
use selene_kernel_contracts::ph1n::{FieldKey, FieldValue, IntentDraft, IntentType};
use selene_kernel_contracts::ph1onb::{Ph1OnbRequest, Ph1OnbResponse};
use selene_kernel_contracts::ph1position::{Ph1PositionRequest, Ph1PositionResponse, TenantId};
use selene_kernel_contracts::ph1w::{Ph1wRequest, Ph1wResponse};
use selene_kernel_contracts::ph1x::{DispatchRequest, Ph1xDirective, Ph1xResponse};
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs};
use selene_storage::ph1f::{AccessDecision, AccessMode, Ph1fStore, StorageError};

use crate::ph1_voice_id::Ph1VoiceIdRuntime;
use crate::ph1capreq::Ph1CapreqRuntime;
use crate::ph1link::{Ph1LinkConfig, Ph1LinkRuntime};
use crate::ph1onb::{
    OnbPositionLiveRequest, OnbPositionLiveResult, OnbVoiceEnrollLiveRequest,
    OnbVoiceEnrollLiveResult, Ph1OnbOrchRuntime,
};
use crate::ph1position::Ph1PositionRuntime;
use crate::ph1w::Ph1wRuntime;

/// Minimal Simulation Executor (v1).
///
/// Hard rule (constitution): No Simulation -> No Execution.
///
/// In this repo's current slice, the executor supports PH1.LINK, PH1.ONB, PH1.POSITION,
/// PH1.W, and PH1.VOICE.ID enrollment simulations.
/// Other simulations are added incrementally and must be registered in docs/08_SIMULATION_CATALOG.md.
#[derive(Debug, Clone)]
pub struct SimulationExecutor {
    link: Ph1LinkRuntime,
    onb: Ph1OnbOrchRuntime,
    position: Ph1PositionRuntime,
    capreq: Ph1CapreqRuntime,
    voice_id: Ph1VoiceIdRuntime,
    wake: Ph1wRuntime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimulationDispatchOutcome {
    Link(Ph1LinkResponse),
    Onboarding(Ph1OnbResponse),
    Position(Ph1PositionResponse),
    VoiceId(Ph1VoiceIdSimResponse),
    Wake(Ph1wResponse),
    AccessGatePassed {
        requested_action: String,
    },
    CapreqLifecycle {
        capreq_id: CapreqId,
        capreq_event_id: u64,
        action: CapabilityRequestAction,
        status: CapabilityRequestStatus,
    },
}

impl Default for SimulationExecutor {
    fn default() -> Self {
        Self {
            link: Ph1LinkRuntime::new(Ph1LinkConfig::mvp_v1()),
            onb: Ph1OnbOrchRuntime::default(),
            position: Ph1PositionRuntime,
            capreq: Ph1CapreqRuntime,
            voice_id: Ph1VoiceIdRuntime,
            wake: Ph1wRuntime,
        }
    }
}

impl SimulationExecutor {
    pub fn new(link: Ph1LinkRuntime, onb: Ph1OnbOrchRuntime) -> Self {
        Self {
            link,
            onb,
            position: Ph1PositionRuntime,
            capreq: Ph1CapreqRuntime,
            voice_id: Ph1VoiceIdRuntime,
            wake: Ph1wRuntime,
        }
    }

    pub fn new_with_wake(link: Ph1LinkRuntime, onb: Ph1OnbOrchRuntime, wake: Ph1wRuntime) -> Self {
        Self {
            link,
            onb,
            position: Ph1PositionRuntime,
            capreq: Ph1CapreqRuntime,
            voice_id: Ph1VoiceIdRuntime,
            wake,
        }
    }

    pub fn new_with_position_and_wake(
        link: Ph1LinkRuntime,
        onb: Ph1OnbOrchRuntime,
        position: Ph1PositionRuntime,
        wake: Ph1wRuntime,
    ) -> Self {
        Self {
            link,
            onb,
            position,
            capreq: Ph1CapreqRuntime,
            voice_id: Ph1VoiceIdRuntime,
            wake,
        }
    }

    pub fn execute_link(
        &self,
        store: &mut Ph1fStore,
        req: &Ph1LinkRequest,
    ) -> Result<Ph1LinkResponse, StorageError> {
        self.link.run(store, req)
    }

    pub fn execute_onb(
        &self,
        store: &mut Ph1fStore,
        req: &Ph1OnbRequest,
    ) -> Result<Ph1OnbResponse, StorageError> {
        self.onb.run(store, req)
    }

    pub fn execute_onb_voice_enrollment_live_sequence(
        &self,
        store: &mut Ph1fStore,
        req: &OnbVoiceEnrollLiveRequest,
    ) -> Result<OnbVoiceEnrollLiveResult, StorageError> {
        self.onb.run_voice_enrollment_live_sequence(store, req)
    }

    pub fn execute_onb_position_live_sequence(
        &self,
        store: &mut Ph1fStore,
        req: &OnbPositionLiveRequest,
    ) -> Result<OnbPositionLiveResult, StorageError> {
        self.onb.run_position_live_sequence(store, req)
    }

    pub fn execute_position(
        &self,
        store: &mut Ph1fStore,
        req: &Ph1PositionRequest,
    ) -> Result<Ph1PositionResponse, StorageError> {
        self.position.run(store, req)
    }

    pub fn execute_capreq(
        &self,
        store: &mut Ph1fStore,
        req: &Ph1CapreqRequest,
    ) -> Result<Ph1CapreqResponse, StorageError> {
        self.capreq.run(store, req)
    }

    pub fn execute_voice_id(
        &self,
        store: &mut Ph1fStore,
        req: &Ph1VoiceIdSimRequest,
    ) -> Result<Ph1VoiceIdSimResponse, StorageError> {
        self.voice_id.run(store, req)
    }

    pub fn execute_wake(
        &self,
        store: &mut Ph1fStore,
        req: &Ph1wRequest,
    ) -> Result<Ph1wResponse, StorageError> {
        self.wake.run(store, req)
    }

    /// Execute a PH1.X `Dispatch(SimulationCandidate)` by mapping it to a concrete simulation call.
    ///
    /// Hard rule: No Simulation -> No Execution. This method only accepts a SimulationCandidate dispatch.
    pub fn execute_ph1x_dispatch_simulation_candidate(
        &self,
        store: &mut Ph1fStore,
        actor_user_id: UserId,
        now: MonotonicTimeNs,
        x: &Ph1xResponse,
    ) -> Result<SimulationDispatchOutcome, StorageError> {
        let dispatch = match &x.directive {
            Ph1xDirective::Dispatch(d) => d,
            _ => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1x_response.directive",
                        reason: "must be Dispatch(SimulationCandidate)",
                    },
                ))
            }
        };

        match &dispatch.dispatch_request {
            DispatchRequest::SimulationCandidate(c) => self.execute_simulation_candidate_v1(
                store,
                actor_user_id,
                now,
                CorrelationId(x.correlation_id),
                TurnId(x.turn_id),
                x.idempotency_key.as_deref(),
                &c.intent_draft,
            ),
            DispatchRequest::Tool(_) => Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1x_response.directive.dispatch_request",
                    reason: "tool dispatch must be handled by PH1.E (not SimulationExecutor)",
                },
            )),
        }
    }

    fn execute_simulation_candidate_v1(
        &self,
        store: &mut Ph1fStore,
        actor_user_id: UserId,
        now: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        x_idempotency_key: Option<&str>,
        d: &IntentDraft,
    ) -> Result<SimulationDispatchOutcome, StorageError> {
        match d.intent_type {
            IntentType::CreateInviteLink => {
                let invitee_type =
                    parse_invitee_type(required_field_value(d, FieldKey::InviteeType)?)?;
                let tenant_id = parse_tenant_id(required_field_value(d, FieldKey::TenantId)?)?;
                self.enforce_link_access_gate(store, &actor_user_id, &tenant_id, now)?;
                let tenant_id = Some(tenant_id.as_str().to_string());

                let req = Ph1LinkRequest::invite_generate_draft_v1(
                    correlation_id,
                    turn_id,
                    now,
                    actor_user_id,
                    invitee_type,
                    tenant_id,
                    None,
                    None,
                    None,
                )?;
                let resp = self.execute_link(store, &req)?;
                Ok(SimulationDispatchOutcome::Link(resp))
            }
            IntentType::CapreqManage => {
                let tenant_id = parse_tenant_id(required_field_value(d, FieldKey::TenantId)?)?;
                self.enforce_capreq_access_gate(store, &actor_user_id, &tenant_id, now)?;
                let requested_capability_id =
                    optional_field_value(d, FieldKey::RequestedCapabilityId)
                        .map(|v| field_str(v).to_string());
                let target_scope_ref = optional_field_value(d, FieldKey::TargetScopeRef)
                    .map(|v| field_str(v).to_string());
                let justification = optional_field_value(d, FieldKey::Justification)
                    .map(|v| field_str(v).to_string());

                let action = parse_capreq_action(optional_field_value(d, FieldKey::CapreqAction))?;

                let action_token = capreq_action_token(action);
                let idempotency_key = x_idempotency_key
                    .map(|k| format!("capreq_{action_token}:{k}"))
                    .unwrap_or_else(|| {
                        format!("capreq_{action_token}:{}:{}", correlation_id.0, turn_id.0)
                    });

                let req = match action {
                    CapabilityRequestAction::CreateDraft => Ph1CapreqRequest::create_draft_v1(
                        correlation_id,
                        turn_id,
                        now,
                        actor_user_id,
                        tenant_id,
                        required_capreq_snapshot_field(
                            requested_capability_id,
                            "simulation_candidate_dispatch.intent_draft.fields.requested_capability_id",
                        )?,
                        required_capreq_snapshot_field(
                            target_scope_ref,
                            "simulation_candidate_dispatch.intent_draft.fields.target_scope_ref",
                        )?,
                        required_capreq_snapshot_field(
                            justification,
                            "simulation_candidate_dispatch.intent_draft.fields.justification",
                        )?,
                        idempotency_key,
                    )?,
                    CapabilityRequestAction::SubmitForApproval => {
                        let capreq_id = resolve_capreq_id(
                            d,
                            action,
                            &tenant_id,
                            requested_capability_id.as_deref(),
                            target_scope_ref.as_deref(),
                            justification.as_deref(),
                        )?;
                        Ph1CapreqRequest::submit_for_approval_commit_v1(
                            correlation_id,
                            turn_id,
                            now,
                            actor_user_id,
                            tenant_id,
                            capreq_id,
                            idempotency_key,
                        )?
                    }
                    CapabilityRequestAction::Approve => {
                        let capreq_id = resolve_capreq_id(
                            d,
                            action,
                            &tenant_id,
                            requested_capability_id.as_deref(),
                            target_scope_ref.as_deref(),
                            justification.as_deref(),
                        )?;
                        Ph1CapreqRequest::approve_commit_v1(
                            correlation_id,
                            turn_id,
                            now,
                            actor_user_id,
                            tenant_id,
                            capreq_id,
                            idempotency_key,
                        )?
                    }
                    CapabilityRequestAction::Reject => {
                        let capreq_id = resolve_capreq_id(
                            d,
                            action,
                            &tenant_id,
                            requested_capability_id.as_deref(),
                            target_scope_ref.as_deref(),
                            justification.as_deref(),
                        )?;
                        Ph1CapreqRequest::reject_commit_v1(
                            correlation_id,
                            turn_id,
                            now,
                            actor_user_id,
                            tenant_id,
                            capreq_id,
                            idempotency_key,
                        )?
                    }
                    CapabilityRequestAction::Fulfill => {
                        let capreq_id = resolve_capreq_id(
                            d,
                            action,
                            &tenant_id,
                            requested_capability_id.as_deref(),
                            target_scope_ref.as_deref(),
                            justification.as_deref(),
                        )?;
                        Ph1CapreqRequest::fulfill_commit_v1(
                            correlation_id,
                            turn_id,
                            now,
                            actor_user_id,
                            tenant_id,
                            capreq_id,
                            idempotency_key,
                        )?
                    }
                    CapabilityRequestAction::Cancel => {
                        let capreq_id = resolve_capreq_id(
                            d,
                            action,
                            &tenant_id,
                            requested_capability_id.as_deref(),
                            target_scope_ref.as_deref(),
                            justification.as_deref(),
                        )?;
                        Ph1CapreqRequest::cancel_revoke_v1(
                            correlation_id,
                            turn_id,
                            now,
                            actor_user_id,
                            tenant_id,
                            capreq_id,
                            idempotency_key,
                        )?
                    }
                };

                let resp = self.execute_capreq(store, &req)?;
                match resp {
                    Ph1CapreqResponse::Ok(ok) => {
                        let lifecycle = ok.lifecycle_result;
                        Ok(SimulationDispatchOutcome::CapreqLifecycle {
                            capreq_id: lifecycle.capreq_id,
                            capreq_event_id: lifecycle.capreq_event_id,
                            action: lifecycle.action,
                            status: lifecycle.status,
                        })
                    }
                    Ph1CapreqResponse::Refuse(_) => Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "ph1capreq_response",
                            reason: "refuse is unexpected for simulation-candidate dispatch",
                        },
                    )),
                }
            }
            IntentType::AccessSchemaManage => {
                let tenant_id = parse_tenant_id(required_field_value(d, FieldKey::TenantId)?)?;
                let action = parse_access_ap_action(required_field_value(d, FieldKey::ApAction)?)?;
                let _review_channel = parse_access_review_channel(required_field_value(
                    d,
                    FieldKey::AccessReviewChannel,
                )?)?;
                let _access_profile_id = required_field_value(d, FieldKey::AccessProfileId)?;
                let _schema_version_id = required_field_value(d, FieldKey::SchemaVersionId)?;
                if matches!(action, AccessApAction::CreateDraft | AccessApAction::Update) {
                    let _profile_payload = required_field_value(d, FieldKey::ProfilePayloadJson)?;
                }
                if action == AccessApAction::Activate {
                    let _rule_action = parse_access_rule_action(required_field_value(
                        d,
                        FieldKey::AccessRuleAction,
                    )?)?;
                }
                self.enforce_access_schema_gate(store, &actor_user_id, &tenant_id, now)?;
                Ok(SimulationDispatchOutcome::AccessGatePassed {
                    requested_action: "ACCESS_SCHEMA_MANAGE".to_string(),
                })
            }
            IntentType::AccessEscalationVote => {
                let tenant_id = parse_tenant_id(required_field_value(d, FieldKey::TenantId)?)?;
                self.enforce_access_escalation_vote_gate(store, &actor_user_id, &tenant_id, now)?;
                Ok(SimulationDispatchOutcome::AccessGatePassed {
                    requested_action: "ACCESS_ESCALATION_VOTE".to_string(),
                })
            }
            IntentType::AccessInstanceCompileRefresh => {
                let tenant_id = parse_tenant_id(required_field_value(d, FieldKey::TenantId)?)?;
                self.enforce_access_instance_compile_gate(store, &actor_user_id, &tenant_id, now)?;
                Ok(SimulationDispatchOutcome::AccessGatePassed {
                    requested_action: "ACCESS_INSTANCE_COMPILE_REFRESH".to_string(),
                })
            }
            _ => Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.intent_draft.intent_type",
                    reason: "unsupported in this slice",
                },
            )),
        }
    }

    fn enforce_capreq_access_gate(
        &self,
        store: &Ph1fStore,
        actor_user_id: &UserId,
        tenant_id: &TenantId,
        now: MonotonicTimeNs,
    ) -> Result<(), StorageError> {
        self.enforce_access_gate(
            store,
            actor_user_id,
            tenant_id,
            "CAPREQ_MANAGE",
            "simulation_candidate_dispatch.capreq.access_instance_id",
            "simulation_candidate_dispatch.capreq.access_decision",
            now,
        )
    }

    fn enforce_link_access_gate(
        &self,
        store: &Ph1fStore,
        actor_user_id: &UserId,
        tenant_id: &TenantId,
        now: MonotonicTimeNs,
    ) -> Result<(), StorageError> {
        self.enforce_access_gate(
            store,
            actor_user_id,
            tenant_id,
            "LINK_INVITE",
            "simulation_candidate_dispatch.link.access_instance_id",
            "simulation_candidate_dispatch.link.access_decision",
            now,
        )
    }

    fn enforce_access_schema_gate(
        &self,
        store: &Ph1fStore,
        actor_user_id: &UserId,
        tenant_id: &TenantId,
        now: MonotonicTimeNs,
    ) -> Result<(), StorageError> {
        self.enforce_access_gate(
            store,
            actor_user_id,
            tenant_id,
            "ACCESS_SCHEMA_MANAGE",
            "simulation_candidate_dispatch.access_schema.access_instance_id",
            "simulation_candidate_dispatch.access_schema.access_decision",
            now,
        )
    }

    fn enforce_access_escalation_vote_gate(
        &self,
        store: &Ph1fStore,
        actor_user_id: &UserId,
        tenant_id: &TenantId,
        now: MonotonicTimeNs,
    ) -> Result<(), StorageError> {
        self.enforce_access_gate(
            store,
            actor_user_id,
            tenant_id,
            "ACCESS_ESCALATION_VOTE",
            "simulation_candidate_dispatch.access_escalation_vote.access_instance_id",
            "simulation_candidate_dispatch.access_escalation_vote.access_decision",
            now,
        )
    }

    fn enforce_access_instance_compile_gate(
        &self,
        store: &Ph1fStore,
        actor_user_id: &UserId,
        tenant_id: &TenantId,
        now: MonotonicTimeNs,
    ) -> Result<(), StorageError> {
        self.enforce_access_gate(
            store,
            actor_user_id,
            tenant_id,
            "ACCESS_INSTANCE_COMPILE_REFRESH",
            "simulation_candidate_dispatch.access_instance_compile.access_instance_id",
            "simulation_candidate_dispatch.access_instance_compile.access_decision",
            now,
        )
    }

    fn enforce_access_gate(
        &self,
        store: &Ph1fStore,
        actor_user_id: &UserId,
        tenant_id: &TenantId,
        requested_action: &str,
        field_access_instance: &'static str,
        field_access_decision: &'static str,
        now: MonotonicTimeNs,
    ) -> Result<(), StorageError> {
        let Some(access_instance) =
            store.ph2access_get_instance_by_tenant_user(tenant_id.as_str(), actor_user_id)
        else {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: field_access_instance,
                    reason: "missing access instance for actor_user_id + tenant_id",
                },
            ));
        };

        let gate = store.ph1access_gate_decide(
            actor_user_id.clone(),
            access_instance.access_instance_id.clone(),
            requested_action.to_string(),
            AccessMode::A,
            access_instance.device_trust_level,
            false,
            now,
        )?;

        match gate.access_decision {
            AccessDecision::Allow => Ok(()),
            AccessDecision::Deny => Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: field_access_decision,
                    reason: "ACCESS_SCOPE_VIOLATION",
                },
            )),
            AccessDecision::Escalate => Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: field_access_decision,
                    reason: "ACCESS_AP_REQUIRED",
                },
            )),
        }
    }
}

fn field_value<'a>(d: &'a IntentDraft, k: FieldKey) -> Option<&'a FieldValue> {
    d.fields.iter().find(|f| f.key == k).map(|f| &f.value)
}

fn optional_field_value<'a>(d: &'a IntentDraft, k: FieldKey) -> Option<&'a FieldValue> {
    field_value(d, k)
}

fn required_field_value<'a>(
    d: &'a IntentDraft,
    k: FieldKey,
) -> Result<&'a FieldValue, StorageError> {
    field_value(d, k).ok_or_else(|| {
        StorageError::ContractViolation(ContractViolation::InvalidValue {
            field: "simulation_candidate_dispatch.intent_draft.fields",
            reason: "missing required field",
        })
    })
}

fn field_str(v: &FieldValue) -> &str {
    v.normalized_value
        .as_deref()
        .unwrap_or(v.original_span.as_str())
        .trim()
}

fn parse_invitee_type(
    v: &FieldValue,
) -> Result<selene_kernel_contracts::ph1link::InviteeType, StorageError> {
    use selene_kernel_contracts::ph1link::InviteeType::*;
    let s = field_str(v).to_ascii_lowercase();
    match s.as_str() {
        "company" => Ok(Company),
        "customer" => Ok(Customer),
        "employee" => Ok(Employee),
        "family_member" | "familymember" => Ok(FamilyMember),
        "friend" => Ok(Friend),
        "associate" => Ok(Associate),
        _ => Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "simulation_candidate_dispatch.intent_draft.fields.invitee_type",
                reason:
                    "must be one of: company, customer, employee, family_member, friend, associate",
            },
        )),
    }
}

fn parse_tenant_id(v: &FieldValue) -> Result<TenantId, StorageError> {
    TenantId::new(field_str(v).to_string()).map_err(StorageError::ContractViolation)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccessApAction {
    CreateDraft,
    Update,
    Activate,
    Retire,
}

fn parse_access_ap_action(v: &FieldValue) -> Result<AccessApAction, StorageError> {
    let s = field_str(v).to_ascii_uppercase();
    match s.as_str() {
        "CREATE_DRAFT" => Ok(AccessApAction::CreateDraft),
        "UPDATE" => Ok(AccessApAction::Update),
        "ACTIVATE" => Ok(AccessApAction::Activate),
        "RETIRE" => Ok(AccessApAction::Retire),
        _ => Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "simulation_candidate_dispatch.intent_draft.fields.ap_action",
                reason: "must be one of: CREATE_DRAFT, UPDATE, ACTIVATE, RETIRE",
            },
        )),
    }
}

fn parse_access_review_channel(v: &FieldValue) -> Result<(), StorageError> {
    let s = field_str(v).to_ascii_uppercase();
    match s.as_str() {
        "PHONE_DESKTOP" | "READ_OUT_LOUD" => Ok(()),
        _ => Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "simulation_candidate_dispatch.intent_draft.fields.access_review_channel",
                reason: "must be PHONE_DESKTOP or READ_OUT_LOUD",
            },
        )),
    }
}

fn parse_access_rule_action(v: &FieldValue) -> Result<(), StorageError> {
    let s = field_str(v).to_ascii_uppercase();
    match s.as_str() {
        "AGREE" | "DISAGREE" | "EDIT" | "DELETE" | "DISABLE" | "ADD_CUSTOM_RULE" => Ok(()),
        _ => Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "simulation_candidate_dispatch.intent_draft.fields.access_rule_action",
                reason: "must be one of: AGREE, DISAGREE, EDIT, DELETE, DISABLE, ADD_CUSTOM_RULE",
            },
        )),
    }
}

fn parse_capreq_action(v: Option<&FieldValue>) -> Result<CapabilityRequestAction, StorageError> {
    let Some(v) = v else {
        return Ok(CapabilityRequestAction::CreateDraft);
    };
    let s = field_str(v).to_ascii_lowercase();
    match s.as_str() {
        "create_draft" | "create" | "draft" => Ok(CapabilityRequestAction::CreateDraft),
        "submit_for_approval" | "submit" => Ok(CapabilityRequestAction::SubmitForApproval),
        "approve" => Ok(CapabilityRequestAction::Approve),
        "reject" => Ok(CapabilityRequestAction::Reject),
        "fulfill" | "fulfilled" => Ok(CapabilityRequestAction::Fulfill),
        "cancel" | "revoke" => Ok(CapabilityRequestAction::Cancel),
        _ => Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "simulation_candidate_dispatch.intent_draft.fields.capreq_action",
                reason: "must be one of: create_draft, submit_for_approval, approve, reject, fulfill, cancel",
            },
        )),
    }
}

fn capreq_action_token(action: CapabilityRequestAction) -> &'static str {
    match action {
        CapabilityRequestAction::CreateDraft => "create_draft",
        CapabilityRequestAction::SubmitForApproval => "submit_for_approval",
        CapabilityRequestAction::Approve => "approve",
        CapabilityRequestAction::Reject => "reject",
        CapabilityRequestAction::Fulfill => "fulfill",
        CapabilityRequestAction::Cancel => "cancel",
    }
}

fn resolve_capreq_id(
    d: &IntentDraft,
    action: CapabilityRequestAction,
    tenant_id: &TenantId,
    requested_capability_id: Option<&str>,
    target_scope_ref: Option<&str>,
    justification: Option<&str>,
) -> Result<CapreqId, StorageError> {
    if let Some(v) = optional_field_value(d, FieldKey::CapreqId) {
        return CapreqId::new(field_str(v).to_string()).map_err(StorageError::ContractViolation);
    }

    match action {
        CapabilityRequestAction::CreateDraft => derive_capreq_id(
            tenant_id,
            requested_capability_id,
            target_scope_ref,
            justification,
        ),
        CapabilityRequestAction::SubmitForApproval => {
            if requested_capability_id.is_some()
                && target_scope_ref.is_some()
                && justification.is_some()
            {
                derive_capreq_id(
                    tenant_id,
                    requested_capability_id,
                    target_scope_ref,
                    justification,
                )
            } else {
                Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "simulation_candidate_dispatch.intent_draft.fields.capreq_id",
                        reason: "required for submit when request snapshot fields are missing",
                    },
                ))
            }
        }
        CapabilityRequestAction::Approve
        | CapabilityRequestAction::Reject
        | CapabilityRequestAction::Fulfill
        | CapabilityRequestAction::Cancel => Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "simulation_candidate_dispatch.intent_draft.fields.capreq_id",
                reason: "required for lifecycle actions after draft creation",
            },
        )),
    }
}

fn derive_capreq_id(
    tenant_id: &TenantId,
    requested_capability_id: Option<&str>,
    target_scope_ref: Option<&str>,
    justification: Option<&str>,
) -> Result<CapreqId, StorageError> {
    let requested_capability_id = requested_capability_id.ok_or_else(|| {
        StorageError::ContractViolation(ContractViolation::InvalidValue {
            field: "simulation_candidate_dispatch.intent_draft.fields.requested_capability_id",
            reason: "required",
        })
    })?;
    let target_scope_ref = target_scope_ref.ok_or_else(|| {
        StorageError::ContractViolation(ContractViolation::InvalidValue {
            field: "simulation_candidate_dispatch.intent_draft.fields.target_scope_ref",
            reason: "required",
        })
    })?;
    let justification = justification.ok_or_else(|| {
        StorageError::ContractViolation(ContractViolation::InvalidValue {
            field: "simulation_candidate_dispatch.intent_draft.fields.justification",
            reason: "required",
        })
    })?;

    CapreqId::new(format!(
        "capreq_{}",
        short_hash_hex(&[
            tenant_id.as_str(),
            requested_capability_id,
            target_scope_ref,
            justification,
        ])
    ))
    .map_err(StorageError::ContractViolation)
}

fn required_capreq_snapshot_field(
    value: Option<String>,
    field: &'static str,
) -> Result<String, StorageError> {
    let value = value.ok_or_else(|| {
        StorageError::ContractViolation(ContractViolation::InvalidValue {
            field,
            reason: "required",
        })
    })?;
    if value.trim().is_empty() {
        return Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field,
                reason: "required",
            },
        ));
    }
    Ok(value)
}

fn short_hash_hex(parts: &[&str]) -> String {
    // FNV-1a 64-bit; deterministic and bounded for id/payload derivation.
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut h = OFFSET;
    for part in parts {
        for &b in part.as_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(PRIME);
        }
        // Stable delimiter to avoid accidental concatenation ambiguity.
        h ^= b'|' as u64;
        h = h.wrapping_mul(PRIME);
    }
    format!("{h:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1_voice_id::{
        Ph1VoiceIdSimRequest, Ph1VoiceIdSimResponse, VoiceIdEnrollStartDraftRequest,
        VoiceIdSimulationRequest, VoiceIdSimulationType, PH1VOICEID_SIM_CONTRACT_VERSION,
        VOICE_ID_ENROLL_START_DRAFT,
    };
    use selene_kernel_contracts::ph1capreq::{
        CapabilityRequestAction, CapabilityRequestStatus, CapreqId,
    };
    use selene_kernel_contracts::ph1j::DeviceId;
    use selene_kernel_contracts::ph1link::InviteeType;
    use selene_kernel_contracts::ph1n::{IntentField, OverallConfidence, SensitivityLevel};
    use selene_kernel_contracts::ph1position::{
        Ph1PositionRequest, Ph1PositionResponse, PositionCreateDraftRequest, PositionRequest,
        PositionScheduleType, PositionSimulationType, PH1POSITION_CONTRACT_VERSION,
        POSITION_SIM_001_CREATE_DRAFT,
    };
    use selene_kernel_contracts::ph1w::{
        Ph1wRequest, Ph1wResponse, WakeEnrollStartDraftRequest, WakeRequest, WakeSimulationType,
        PH1W_CONTRACT_VERSION, WAKE_ENROLL_START_DRAFT,
    };
    use selene_kernel_contracts::ph1x::{
        DeliveryHint, DispatchDirective, Ph1xDirective, Ph1xResponse, ThreadState,
    };
    use selene_kernel_contracts::{ReasonCodeId, SchemaVersion};
    use selene_storage::ph1f::{
        AccessDeviceTrustLevel, AccessLifecycleState, AccessMode, AccessVerificationLevel,
        DeviceRecord, IdentityRecord, IdentityStatus, TenantCompanyLifecycleState,
        TenantCompanyRecord,
    };

    fn capreq_field(key: FieldKey, value: &str) -> IntentField {
        IntentField {
            key,
            value: FieldValue::verbatim(value.to_string()).unwrap(),
            confidence: OverallConfidence::High,
        }
    }

    fn capreq_draft(fields: Vec<IntentField>) -> IntentDraft {
        IntentDraft::v1(
            IntentType::CapreqManage,
            SchemaVersion(1),
            fields,
            vec![],
            OverallConfidence::High,
            vec![],
            ReasonCodeId(1),
            SensitivityLevel::Private,
            true,
            vec![],
            vec![],
        )
        .unwrap()
    }

    fn capreq_x(turn_id: u64, draft: IntentDraft, idempotency_key: &str) -> Ph1xResponse {
        Ph1xResponse::v1(
            10,
            turn_id,
            Ph1xDirective::Dispatch(DispatchDirective::simulation_candidate_v1(draft).unwrap()),
            ThreadState::empty_v1(),
            None,
            DeliveryHint::AudibleAndText,
            ReasonCodeId(1),
            Some(idempotency_key.to_string()),
        )
        .unwrap()
    }

    fn run_capreq(
        exec: &SimulationExecutor,
        store: &mut Ph1fStore,
        actor: &UserId,
        turn_id: u64,
        now: u64,
        idempotency_key: &str,
        fields: Vec<IntentField>,
    ) -> SimulationDispatchOutcome {
        let tenant = fields
            .iter()
            .find(|f| f.key == FieldKey::TenantId)
            .map(|f| field_str(&f.value).to_string())
            .expect("capreq tests require FieldKey::TenantId");
        seed_capreq_access_instance(store, actor, &tenant);

        let x = capreq_x(turn_id, capreq_draft(fields), idempotency_key);
        exec.execute_ph1x_dispatch_simulation_candidate(
            store,
            actor.clone(),
            MonotonicTimeNs(now),
            &x,
        )
        .unwrap()
    }

    fn seed_capreq_access_instance(store: &mut Ph1fStore, actor: &UserId, tenant: &str) {
        seed_capreq_access_instance_with(
            store,
            actor,
            tenant,
            AccessMode::A,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
        );
    }

    fn seed_capreq_access_instance_with(
        store: &mut Ph1fStore,
        actor: &UserId,
        tenant: &str,
        effective_access_mode: AccessMode,
        identity_verified: bool,
        device_trust_level: AccessDeviceTrustLevel,
        lifecycle_state: AccessLifecycleState,
    ) {
        seed_access_instance_with_permissions(
            store,
            actor,
            tenant,
            "role.capreq_manager",
            "{\"allow\":[\"CAPREQ_MANAGE\"]}",
            effective_access_mode,
            identity_verified,
            device_trust_level,
            lifecycle_state,
        );
    }

    fn seed_access_instance_with_permissions(
        store: &mut Ph1fStore,
        actor: &UserId,
        tenant: &str,
        role_template_id: &str,
        baseline_permissions_json: &str,
        effective_access_mode: AccessMode,
        identity_verified: bool,
        device_trust_level: AccessDeviceTrustLevel,
        lifecycle_state: AccessLifecycleState,
    ) {
        store
            .ph2access_upsert_instance_commit(
                MonotonicTimeNs(1),
                tenant.to_string(),
                actor.clone(),
                role_template_id.to_string(),
                effective_access_mode,
                baseline_permissions_json.to_string(),
                identity_verified,
                AccessVerificationLevel::PasscodeTime,
                device_trust_level,
                lifecycle_state,
                "policy_snapshot_v1".to_string(),
                None,
            )
            .unwrap();
    }

    fn seed_link_access_instance(store: &mut Ph1fStore, actor: &UserId, tenant: &str) {
        seed_link_access_instance_with(
            store,
            actor,
            tenant,
            AccessMode::A,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
        );
    }

    fn seed_link_access_instance_with(
        store: &mut Ph1fStore,
        actor: &UserId,
        tenant: &str,
        effective_access_mode: AccessMode,
        identity_verified: bool,
        device_trust_level: AccessDeviceTrustLevel,
        lifecycle_state: AccessLifecycleState,
    ) {
        seed_access_instance_with_permissions(
            store,
            actor,
            tenant,
            "role.link_inviter",
            "{\"allow\":[\"LINK_INVITE\"]}",
            effective_access_mode,
            identity_verified,
            device_trust_level,
            lifecycle_state,
        );
    }

    fn access_field(key: FieldKey, value: &str) -> IntentField {
        IntentField {
            key,
            value: FieldValue::verbatim(value.to_string()).unwrap(),
            confidence: OverallConfidence::High,
        }
    }

    fn access_draft(intent_type: IntentType, fields: Vec<IntentField>) -> IntentDraft {
        IntentDraft::v1(
            intent_type,
            SchemaVersion(1),
            fields,
            vec![],
            OverallConfidence::High,
            vec![],
            ReasonCodeId(1),
            SensitivityLevel::Private,
            true,
            vec![],
            vec![],
        )
        .unwrap()
    }

    fn access_x(turn_id: u64, draft: IntentDraft, idempotency_key: &str) -> Ph1xResponse {
        Ph1xResponse::v1(
            10,
            turn_id,
            Ph1xDirective::Dispatch(DispatchDirective::simulation_candidate_v1(draft).unwrap()),
            ThreadState::empty_v1(),
            None,
            DeliveryHint::AudibleAndText,
            ReasonCodeId(1),
            Some(idempotency_key.to_string()),
        )
        .unwrap()
    }

    #[test]
    fn at_sim_exec_01_ph1x_sim_candidate_create_invite_link_runs_ph1link_generate_draft() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("inviter-1").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_link_access_instance(&mut store, &actor, "tenant_1");

        let draft = IntentDraft::v1(
            IntentType::CreateInviteLink,
            SchemaVersion(1),
            vec![
                IntentField {
                    key: FieldKey::InviteeType,
                    value: FieldValue::normalized("employee".to_string(), "employee".to_string())
                        .unwrap(),
                    confidence: OverallConfidence::High,
                },
                IntentField {
                    key: FieldKey::TenantId,
                    value: FieldValue::verbatim("tenant_1".to_string()).unwrap(),
                    confidence: OverallConfidence::High,
                },
            ],
            vec![],
            OverallConfidence::High,
            vec![],
            ReasonCodeId(1),
            SensitivityLevel::Private,
            true,
            vec![],
            vec![],
        )
        .unwrap();

        let x = Ph1xResponse::v1(
            10,
            22,
            Ph1xDirective::Dispatch(DispatchDirective::simulation_candidate_v1(draft).unwrap()),
            ThreadState::empty_v1(),
            None,
            DeliveryHint::AudibleAndText,
            ReasonCodeId(1),
            Some("idem-1".to_string()),
        )
        .unwrap();

        let out = exec
            .execute_ph1x_dispatch_simulation_candidate(&mut store, actor, MonotonicTimeNs(123), &x)
            .unwrap();

        match out {
            SimulationDispatchOutcome::Link(r) => match r {
                Ph1LinkResponse::Ok(ok) => {
                    assert_eq!(ok.simulation_id, "LINK_INVITE_GENERATE_DRAFT");
                    assert!(ok.link_generate_result.is_some());
                }
                Ph1LinkResponse::Refuse(_) => panic!("expected ok"),
            },
            _ => panic!("expected link outcome"),
        }
    }

    #[test]
    fn at_sim_exec_14_link_access_deny_blocks_governed_commit() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("inviter-link-deny-1").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_link_access_instance_with(
            &mut store,
            &actor,
            "tenant_1",
            AccessMode::A,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Suspended,
        );

        let draft = IntentDraft::v1(
            IntentType::CreateInviteLink,
            SchemaVersion(1),
            vec![
                IntentField {
                    key: FieldKey::InviteeType,
                    value: FieldValue::verbatim("employee".to_string()).unwrap(),
                    confidence: OverallConfidence::High,
                },
                IntentField {
                    key: FieldKey::TenantId,
                    value: FieldValue::verbatim("tenant_1".to_string()).unwrap(),
                    confidence: OverallConfidence::High,
                },
            ],
            vec![],
            OverallConfidence::High,
            vec![],
            ReasonCodeId(1),
            SensitivityLevel::Private,
            true,
            vec![],
            vec![],
        )
        .unwrap();

        let x = Ph1xResponse::v1(
            10,
            31,
            Ph1xDirective::Dispatch(DispatchDirective::simulation_candidate_v1(draft).unwrap()),
            ThreadState::empty_v1(),
            None,
            DeliveryHint::AudibleAndText,
            ReasonCodeId(1),
            Some("idem-link-deny-1".to_string()),
        )
        .unwrap();

        let out = exec.execute_ph1x_dispatch_simulation_candidate(
            &mut store,
            actor,
            MonotonicTimeNs(190),
            &x,
        );
        assert!(matches!(
            out,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.link.access_decision",
                    reason: "ACCESS_SCOPE_VIOLATION",
                }
            ))
        ));
    }

    #[test]
    fn at_sim_exec_15_link_access_escalate_requires_approval_before_commit() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("inviter-link-escalate-1").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_link_access_instance_with(
            &mut store,
            &actor,
            "tenant_1",
            AccessMode::R,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
        );

        let draft = IntentDraft::v1(
            IntentType::CreateInviteLink,
            SchemaVersion(1),
            vec![
                IntentField {
                    key: FieldKey::InviteeType,
                    value: FieldValue::verbatim("employee".to_string()).unwrap(),
                    confidence: OverallConfidence::High,
                },
                IntentField {
                    key: FieldKey::TenantId,
                    value: FieldValue::verbatim("tenant_1".to_string()).unwrap(),
                    confidence: OverallConfidence::High,
                },
            ],
            vec![],
            OverallConfidence::High,
            vec![],
            ReasonCodeId(1),
            SensitivityLevel::Private,
            true,
            vec![],
            vec![],
        )
        .unwrap();

        let x = Ph1xResponse::v1(
            10,
            32,
            Ph1xDirective::Dispatch(DispatchDirective::simulation_candidate_v1(draft).unwrap()),
            ThreadState::empty_v1(),
            None,
            DeliveryHint::AudibleAndText,
            ReasonCodeId(1),
            Some("idem-link-escalate-1".to_string()),
        )
        .unwrap();

        let out = exec.execute_ph1x_dispatch_simulation_candidate(
            &mut store,
            actor,
            MonotonicTimeNs(191),
            &x,
        );
        assert!(matches!(
            out,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.link.access_decision",
                    reason: "ACCESS_AP_REQUIRED",
                }
            ))
        ));
    }

    #[test]
    fn at_sim_exec_16_link_tenant_scope_mismatch_fails_closed() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("inviter-link-scope-1").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_link_access_instance(&mut store, &actor, "tenant_1");

        let draft = IntentDraft::v1(
            IntentType::CreateInviteLink,
            SchemaVersion(1),
            vec![
                IntentField {
                    key: FieldKey::InviteeType,
                    value: FieldValue::verbatim("employee".to_string()).unwrap(),
                    confidence: OverallConfidence::High,
                },
                IntentField {
                    key: FieldKey::TenantId,
                    value: FieldValue::verbatim("tenant_2".to_string()).unwrap(),
                    confidence: OverallConfidence::High,
                },
            ],
            vec![],
            OverallConfidence::High,
            vec![],
            ReasonCodeId(1),
            SensitivityLevel::Private,
            true,
            vec![],
            vec![],
        )
        .unwrap();

        let x = Ph1xResponse::v1(
            10,
            33,
            Ph1xDirective::Dispatch(DispatchDirective::simulation_candidate_v1(draft).unwrap()),
            ThreadState::empty_v1(),
            None,
            DeliveryHint::AudibleAndText,
            ReasonCodeId(1),
            Some("idem-link-scope-1".to_string()),
        )
        .unwrap();

        let out = exec.execute_ph1x_dispatch_simulation_candidate(
            &mut store,
            actor,
            MonotonicTimeNs(192),
            &x,
        );
        assert!(matches!(
            out,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.link.access_instance_id",
                    reason: "missing access instance for actor_user_id + tenant_id",
                }
            ))
        ));
    }

    #[test]
    fn at_sim_exec_17_link_allow_path_idempotent_across_retries() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("inviter-link-allow-retry-1").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_link_access_instance(&mut store, &actor, "tenant_1");

        let mk_x = |turn_id: u64| {
            let draft = IntentDraft::v1(
                IntentType::CreateInviteLink,
                SchemaVersion(1),
                vec![
                    IntentField {
                        key: FieldKey::InviteeType,
                        value: FieldValue::verbatim("employee".to_string()).unwrap(),
                        confidence: OverallConfidence::High,
                    },
                    IntentField {
                        key: FieldKey::TenantId,
                        value: FieldValue::verbatim("tenant_1".to_string()).unwrap(),
                        confidence: OverallConfidence::High,
                    },
                ],
                vec![],
                OverallConfidence::High,
                vec![],
                ReasonCodeId(1),
                SensitivityLevel::Private,
                true,
                vec![],
                vec![],
            )
            .unwrap();
            Ph1xResponse::v1(
                10,
                turn_id,
                Ph1xDirective::Dispatch(DispatchDirective::simulation_candidate_v1(draft).unwrap()),
                ThreadState::empty_v1(),
                None,
                DeliveryHint::AudibleAndText,
                ReasonCodeId(1),
                Some("idem-link-allow-retry-1".to_string()),
            )
            .unwrap()
        };

        let out1 = exec
            .execute_ph1x_dispatch_simulation_candidate(
                &mut store,
                actor.clone(),
                MonotonicTimeNs(193),
                &mk_x(34),
            )
            .unwrap();
        let token_1 = match out1 {
            SimulationDispatchOutcome::Link(Ph1LinkResponse::Ok(ok)) => ok
                .link_generate_result
                .expect("link_generate_result")
                .token_id
                .as_str()
                .to_string(),
            _ => panic!("expected link ok"),
        };

        let out2 = exec
            .execute_ph1x_dispatch_simulation_candidate(
                &mut store,
                actor,
                MonotonicTimeNs(194),
                &mk_x(35),
            )
            .unwrap();
        let token_2 = match out2 {
            SimulationDispatchOutcome::Link(Ph1LinkResponse::Ok(ok)) => ok
                .link_generate_result
                .expect("link_generate_result")
                .token_id
                .as_str()
                .to_string(),
            _ => panic!("expected link ok"),
        };

        assert_eq!(token_1, token_2);
    }

    #[test]
    fn at_sim_exec_02_execute_wake_start_draft_runs_ph1w_runtime() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("wake-user-1").unwrap();
        let device_id = DeviceId::new("wake-device-1").unwrap();

        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();

        store
            .insert_device(
                DeviceRecord::v1(
                    device_id.clone(),
                    actor.clone(),
                    "phone".to_string(),
                    MonotonicTimeNs(1),
                    Some("audio_prof_1".to_string()),
                )
                .unwrap(),
            )
            .unwrap();

        let req = Ph1wRequest {
            schema_version: PH1W_CONTRACT_VERSION,
            correlation_id: CorrelationId(777),
            turn_id: TurnId(1),
            now: MonotonicTimeNs(123),
            simulation_id: WAKE_ENROLL_START_DRAFT.to_string(),
            simulation_type: WakeSimulationType::Draft,
            request: WakeRequest::EnrollStartDraft(WakeEnrollStartDraftRequest {
                user_id: actor,
                device_id,
                onboarding_session_id: None,
                pass_target: 5,
                max_attempts: 12,
                enrollment_timeout_ms: 300_000,
                idempotency_key: "wake-start-1".to_string(),
            }),
        };

        let out = exec.execute_wake(&mut store, &req).unwrap();
        match out {
            Ph1wResponse::Ok(ok) => {
                assert_eq!(ok.simulation_id, WAKE_ENROLL_START_DRAFT);
                assert!(ok.enroll_start_result.is_some());
            }
            Ph1wResponse::Refuse(_) => panic!("expected ok"),
        }
    }

    #[test]
    fn at_sim_exec_05_ph1x_sim_candidate_capreq_manage_runs_capreq_create_draft() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("capreq-actor-1").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_capreq_access_instance(&mut store, &actor, "tenant_1");

        let draft = IntentDraft::v1(
            IntentType::CapreqManage,
            SchemaVersion(1),
            vec![
                IntentField {
                    key: FieldKey::TenantId,
                    value: FieldValue::verbatim("tenant_1".to_string()).unwrap(),
                    confidence: OverallConfidence::High,
                },
                IntentField {
                    key: FieldKey::RequestedCapabilityId,
                    value: FieldValue::verbatim("payroll.approve".to_string()).unwrap(),
                    confidence: OverallConfidence::High,
                },
                IntentField {
                    key: FieldKey::TargetScopeRef,
                    value: FieldValue::verbatim("store_17".to_string()).unwrap(),
                    confidence: OverallConfidence::High,
                },
                IntentField {
                    key: FieldKey::Justification,
                    value: FieldValue::verbatim("monthly payroll processing".to_string()).unwrap(),
                    confidence: OverallConfidence::High,
                },
            ],
            vec![],
            OverallConfidence::High,
            vec![],
            ReasonCodeId(1),
            SensitivityLevel::Private,
            true,
            vec![],
            vec![],
        )
        .unwrap();

        let x = Ph1xResponse::v1(
            10,
            23,
            Ph1xDirective::Dispatch(DispatchDirective::simulation_candidate_v1(draft).unwrap()),
            ThreadState::empty_v1(),
            None,
            DeliveryHint::AudibleAndText,
            ReasonCodeId(1),
            Some("idem-capreq-1".to_string()),
        )
        .unwrap();

        let out = exec
            .execute_ph1x_dispatch_simulation_candidate(
                &mut store,
                actor.clone(),
                MonotonicTimeNs(200),
                &x,
            )
            .unwrap();

        let (capreq_id, capreq_event_id, action, status) = match out {
            SimulationDispatchOutcome::CapreqLifecycle {
                capreq_id,
                capreq_event_id,
                action,
                status,
            } => (capreq_id, capreq_event_id, action, status),
            _ => panic!("expected capreq draft outcome"),
        };
        assert!(capreq_event_id > 0);
        assert_eq!(action, CapabilityRequestAction::CreateDraft);
        assert_eq!(status, CapabilityRequestStatus::Draft);

        let tenant_id = TenantId::new("tenant_1").unwrap();
        let current = store
            .capreq_current_row(&tenant_id, &capreq_id)
            .expect("capreq current row exists");
        assert_eq!(current.status, CapabilityRequestStatus::Draft);
        assert_eq!(current.last_action, CapabilityRequestAction::CreateDraft);
        assert_eq!(current.requester_user_id, actor);
        assert_eq!(store.capreq_events().len(), 1);
        assert_eq!(
            store.capreq_events()[0].capreq_id,
            CapreqId::new(capreq_id.as_str().to_string()).unwrap()
        );
    }

    #[test]
    fn at_sim_exec_06_capreq_submit_for_approval_transition() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("capreq-actor-2").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();

        let created = run_capreq(
            &exec,
            &mut store,
            &actor,
            1,
            100,
            "idem-capreq-2-create",
            vec![
                capreq_field(FieldKey::TenantId, "tenant_1"),
                capreq_field(FieldKey::RequestedCapabilityId, "payroll.approve"),
                capreq_field(FieldKey::TargetScopeRef, "store_17"),
                capreq_field(FieldKey::Justification, "monthly payroll processing"),
            ],
        );
        let capreq_id = match created {
            SimulationDispatchOutcome::CapreqLifecycle { capreq_id, .. } => capreq_id,
            _ => panic!("expected capreq lifecycle outcome"),
        };

        let submitted = run_capreq(
            &exec,
            &mut store,
            &actor,
            2,
            101,
            "idem-capreq-2-submit",
            vec![
                capreq_field(FieldKey::TenantId, "tenant_1"),
                capreq_field(FieldKey::CapreqAction, "submit_for_approval"),
                capreq_field(FieldKey::CapreqId, capreq_id.as_str()),
            ],
        );

        match submitted {
            SimulationDispatchOutcome::CapreqLifecycle { action, status, .. } => {
                assert_eq!(action, CapabilityRequestAction::SubmitForApproval);
                assert_eq!(status, CapabilityRequestStatus::PendingApproval);
            }
            _ => panic!("expected capreq lifecycle outcome"),
        }

        let tenant_id = TenantId::new("tenant_1").unwrap();
        let current = store
            .capreq_current_row(&tenant_id, &capreq_id)
            .expect("capreq current row exists");
        assert_eq!(current.status, CapabilityRequestStatus::PendingApproval);
        assert_eq!(
            current.last_action,
            CapabilityRequestAction::SubmitForApproval
        );
        assert_eq!(store.capreq_events().len(), 2);
    }

    #[test]
    fn at_sim_exec_07_capreq_approve_and_fulfill_transitions() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("capreq-actor-3").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();

        let created = run_capreq(
            &exec,
            &mut store,
            &actor,
            1,
            110,
            "idem-capreq-3-create",
            vec![
                capreq_field(FieldKey::TenantId, "tenant_1"),
                capreq_field(FieldKey::RequestedCapabilityId, "payroll.approve"),
                capreq_field(FieldKey::TargetScopeRef, "store_17"),
                capreq_field(FieldKey::Justification, "monthly payroll processing"),
            ],
        );
        let capreq_id = match created {
            SimulationDispatchOutcome::CapreqLifecycle { capreq_id, .. } => capreq_id,
            _ => panic!("expected capreq lifecycle outcome"),
        };

        let _ = run_capreq(
            &exec,
            &mut store,
            &actor,
            2,
            111,
            "idem-capreq-3-submit",
            vec![
                capreq_field(FieldKey::TenantId, "tenant_1"),
                capreq_field(FieldKey::CapreqAction, "submit_for_approval"),
                capreq_field(FieldKey::CapreqId, capreq_id.as_str()),
            ],
        );

        let approved = run_capreq(
            &exec,
            &mut store,
            &actor,
            3,
            112,
            "idem-capreq-3-approve",
            vec![
                capreq_field(FieldKey::TenantId, "tenant_1"),
                capreq_field(FieldKey::CapreqAction, "approve"),
                capreq_field(FieldKey::CapreqId, capreq_id.as_str()),
            ],
        );
        match approved {
            SimulationDispatchOutcome::CapreqLifecycle { action, status, .. } => {
                assert_eq!(action, CapabilityRequestAction::Approve);
                assert_eq!(status, CapabilityRequestStatus::Approved);
            }
            _ => panic!("expected capreq lifecycle outcome"),
        }

        let fulfilled = run_capreq(
            &exec,
            &mut store,
            &actor,
            4,
            113,
            "idem-capreq-3-fulfill",
            vec![
                capreq_field(FieldKey::TenantId, "tenant_1"),
                capreq_field(FieldKey::CapreqAction, "fulfill"),
                capreq_field(FieldKey::CapreqId, capreq_id.as_str()),
            ],
        );
        match fulfilled {
            SimulationDispatchOutcome::CapreqLifecycle { action, status, .. } => {
                assert_eq!(action, CapabilityRequestAction::Fulfill);
                assert_eq!(status, CapabilityRequestStatus::Fulfilled);
            }
            _ => panic!("expected capreq lifecycle outcome"),
        }

        let tenant_id = TenantId::new("tenant_1").unwrap();
        let current = store
            .capreq_current_row(&tenant_id, &capreq_id)
            .expect("capreq current row exists");
        assert_eq!(current.status, CapabilityRequestStatus::Fulfilled);
        assert_eq!(current.last_action, CapabilityRequestAction::Fulfill);
        assert_eq!(store.capreq_events().len(), 4);
    }

    #[test]
    fn at_sim_exec_08_capreq_reject_transition() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("capreq-actor-4").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();

        let created = run_capreq(
            &exec,
            &mut store,
            &actor,
            1,
            120,
            "idem-capreq-4-create",
            vec![
                capreq_field(FieldKey::TenantId, "tenant_1"),
                capreq_field(FieldKey::RequestedCapabilityId, "payroll.approve"),
                capreq_field(FieldKey::TargetScopeRef, "store_17"),
                capreq_field(FieldKey::Justification, "monthly payroll processing"),
            ],
        );
        let capreq_id = match created {
            SimulationDispatchOutcome::CapreqLifecycle { capreq_id, .. } => capreq_id,
            _ => panic!("expected capreq lifecycle outcome"),
        };

        let _ = run_capreq(
            &exec,
            &mut store,
            &actor,
            2,
            121,
            "idem-capreq-4-submit",
            vec![
                capreq_field(FieldKey::TenantId, "tenant_1"),
                capreq_field(FieldKey::CapreqAction, "submit_for_approval"),
                capreq_field(FieldKey::CapreqId, capreq_id.as_str()),
            ],
        );

        let rejected = run_capreq(
            &exec,
            &mut store,
            &actor,
            3,
            122,
            "idem-capreq-4-reject",
            vec![
                capreq_field(FieldKey::TenantId, "tenant_1"),
                capreq_field(FieldKey::CapreqAction, "reject"),
                capreq_field(FieldKey::CapreqId, capreq_id.as_str()),
            ],
        );
        match rejected {
            SimulationDispatchOutcome::CapreqLifecycle { action, status, .. } => {
                assert_eq!(action, CapabilityRequestAction::Reject);
                assert_eq!(status, CapabilityRequestStatus::Rejected);
            }
            _ => panic!("expected capreq lifecycle outcome"),
        }

        let tenant_id = TenantId::new("tenant_1").unwrap();
        let current = store
            .capreq_current_row(&tenant_id, &capreq_id)
            .expect("capreq current row exists");
        assert_eq!(current.status, CapabilityRequestStatus::Rejected);
        assert_eq!(current.last_action, CapabilityRequestAction::Reject);
        assert_eq!(store.capreq_events().len(), 3);
    }

    #[test]
    fn at_sim_exec_09_capreq_cancel_transition() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("capreq-actor-5").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();

        let created = run_capreq(
            &exec,
            &mut store,
            &actor,
            1,
            130,
            "idem-capreq-5-create",
            vec![
                capreq_field(FieldKey::TenantId, "tenant_1"),
                capreq_field(FieldKey::RequestedCapabilityId, "payroll.approve"),
                capreq_field(FieldKey::TargetScopeRef, "store_17"),
                capreq_field(FieldKey::Justification, "monthly payroll processing"),
            ],
        );
        let capreq_id = match created {
            SimulationDispatchOutcome::CapreqLifecycle { capreq_id, .. } => capreq_id,
            _ => panic!("expected capreq lifecycle outcome"),
        };

        let canceled = run_capreq(
            &exec,
            &mut store,
            &actor,
            2,
            131,
            "idem-capreq-5-cancel",
            vec![
                capreq_field(FieldKey::TenantId, "tenant_1"),
                capreq_field(FieldKey::CapreqAction, "cancel"),
                capreq_field(FieldKey::CapreqId, capreq_id.as_str()),
            ],
        );
        match canceled {
            SimulationDispatchOutcome::CapreqLifecycle { action, status, .. } => {
                assert_eq!(action, CapabilityRequestAction::Cancel);
                assert_eq!(status, CapabilityRequestStatus::Canceled);
            }
            _ => panic!("expected capreq lifecycle outcome"),
        }

        let tenant_id = TenantId::new("tenant_1").unwrap();
        let current = store
            .capreq_current_row(&tenant_id, &capreq_id)
            .expect("capreq current row exists");
        assert_eq!(current.status, CapabilityRequestStatus::Canceled);
        assert_eq!(current.last_action, CapabilityRequestAction::Cancel);
        assert_eq!(store.capreq_events().len(), 2);
    }

    #[test]
    fn at_sim_exec_10_capreq_invalid_transition_fails_closed() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("capreq-actor-6").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();

        let created = run_capreq(
            &exec,
            &mut store,
            &actor,
            1,
            140,
            "idem-capreq-6-create",
            vec![
                capreq_field(FieldKey::TenantId, "tenant_1"),
                capreq_field(FieldKey::RequestedCapabilityId, "payroll.approve"),
                capreq_field(FieldKey::TargetScopeRef, "store_17"),
                capreq_field(FieldKey::Justification, "monthly payroll processing"),
            ],
        );
        let capreq_id = match created {
            SimulationDispatchOutcome::CapreqLifecycle { capreq_id, .. } => capreq_id,
            _ => panic!("expected capreq lifecycle outcome"),
        };

        let out = exec.execute_ph1x_dispatch_simulation_candidate(
            &mut store,
            actor,
            MonotonicTimeNs(141),
            &capreq_x(
                2,
                capreq_draft(vec![
                    capreq_field(FieldKey::TenantId, "tenant_1"),
                    capreq_field(FieldKey::CapreqAction, "approve"),
                    capreq_field(FieldKey::CapreqId, capreq_id.as_str()),
                ]),
                "idem-capreq-6-approve-invalid",
            ),
        );
        assert!(matches!(
            out,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "capreq_transition",
                    ..
                }
            ))
        ));
    }

    #[test]
    fn at_sim_exec_11_capreq_access_deny_blocks_governed_commit() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("capreq-actor-7").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_capreq_access_instance_with(
            &mut store,
            &actor,
            "tenant_1",
            AccessMode::A,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Suspended,
        );

        let out = exec.execute_ph1x_dispatch_simulation_candidate(
            &mut store,
            actor,
            MonotonicTimeNs(150),
            &capreq_x(
                1,
                capreq_draft(vec![
                    capreq_field(FieldKey::TenantId, "tenant_1"),
                    capreq_field(FieldKey::RequestedCapabilityId, "payroll.approve"),
                    capreq_field(FieldKey::TargetScopeRef, "store_17"),
                    capreq_field(FieldKey::Justification, "monthly payroll processing"),
                ]),
                "idem-capreq-7-create",
            ),
        );

        assert!(matches!(
            out,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.capreq.access_decision",
                    reason: "ACCESS_SCOPE_VIOLATION",
                }
            ))
        ));
        assert_eq!(store.capreq_events().len(), 0);
        assert_eq!(store.capreq_current().len(), 0);
    }

    #[test]
    fn at_sim_exec_12_capreq_access_escalate_requires_approval_before_commit() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("capreq-actor-8").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_capreq_access_instance_with(
            &mut store,
            &actor,
            "tenant_1",
            AccessMode::R,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
        );

        let out = exec.execute_ph1x_dispatch_simulation_candidate(
            &mut store,
            actor,
            MonotonicTimeNs(151),
            &capreq_x(
                1,
                capreq_draft(vec![
                    capreq_field(FieldKey::TenantId, "tenant_1"),
                    capreq_field(FieldKey::RequestedCapabilityId, "payroll.approve"),
                    capreq_field(FieldKey::TargetScopeRef, "store_17"),
                    capreq_field(FieldKey::Justification, "monthly payroll processing"),
                ]),
                "idem-capreq-8-create",
            ),
        );

        assert!(matches!(
            out,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.capreq.access_decision",
                    reason: "ACCESS_AP_REQUIRED",
                }
            ))
        ));
        assert_eq!(store.capreq_events().len(), 0);
        assert_eq!(store.capreq_current().len(), 0);
    }

    #[test]
    fn at_sim_exec_13_capreq_tenant_scope_mismatch_fails_closed() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("capreq-actor-9").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_capreq_access_instance(&mut store, &actor, "tenant_1");

        let out = exec.execute_ph1x_dispatch_simulation_candidate(
            &mut store,
            actor,
            MonotonicTimeNs(152),
            &capreq_x(
                1,
                capreq_draft(vec![
                    capreq_field(FieldKey::TenantId, "tenant_2"),
                    capreq_field(FieldKey::RequestedCapabilityId, "payroll.approve"),
                    capreq_field(FieldKey::TargetScopeRef, "store_17"),
                    capreq_field(FieldKey::Justification, "monthly payroll processing"),
                ]),
                "idem-capreq-9-create",
            ),
        );

        assert!(matches!(
            out,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.capreq.access_instance_id",
                    reason: "missing access instance for actor_user_id + tenant_id",
                }
            ))
        ));
        assert_eq!(store.capreq_events().len(), 0);
        assert_eq!(store.capreq_current().len(), 0);
    }

    #[test]
    fn at_sim_exec_19_access_schema_manage_gate_allow_returns_gate_passed() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("access-schema-actor-1").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_access_instance_with_permissions(
            &mut store,
            &actor,
            "tenant_1",
            "role.access_schema_admin",
            "{\"allow\":[\"ACCESS_SCHEMA_MANAGE\"]}",
            AccessMode::A,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
        );

        let out = exec
            .execute_ph1x_dispatch_simulation_candidate(
                &mut store,
                actor,
                MonotonicTimeNs(200),
                &access_x(
                    1,
                    access_draft(
                        IntentType::AccessSchemaManage,
                        vec![
                            access_field(FieldKey::TenantId, "tenant_1"),
                            access_field(FieldKey::ApAction, "CREATE_DRAFT"),
                            access_field(FieldKey::AccessProfileId, "AP_CLERK"),
                            access_field(FieldKey::SchemaVersionId, "v1"),
                            access_field(FieldKey::AccessReviewChannel, "PHONE_DESKTOP"),
                            access_field(
                                FieldKey::ProfilePayloadJson,
                                "{\"allow\":[\"ACCESS_SCHEMA_MANAGE\"]}",
                            ),
                        ],
                    ),
                    "idem-access-schema-allow-1",
                ),
            )
            .unwrap();
        assert!(matches!(
            out,
            SimulationDispatchOutcome::AccessGatePassed {
                requested_action
            } if requested_action == "ACCESS_SCHEMA_MANAGE"
        ));
    }

    #[test]
    fn at_sim_exec_20_access_escalation_vote_access_deny_blocks_governed_commit() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("access-vote-actor-1").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_access_instance_with_permissions(
            &mut store,
            &actor,
            "tenant_1",
            "role.access_vote_admin",
            "{\"allow\":[\"ACCESS_ESCALATION_VOTE\"]}",
            AccessMode::A,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Suspended,
        );

        let out = exec.execute_ph1x_dispatch_simulation_candidate(
            &mut store,
            actor,
            MonotonicTimeNs(201),
            &access_x(
                1,
                access_draft(
                    IntentType::AccessEscalationVote,
                    vec![access_field(FieldKey::TenantId, "tenant_1")],
                ),
                "idem-access-vote-deny-1",
            ),
        );
        assert!(matches!(
            out,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.access_escalation_vote.access_decision",
                    reason: "ACCESS_SCOPE_VIOLATION",
                }
            ))
        ));
    }

    #[test]
    fn at_sim_exec_21_access_instance_compile_access_escalate_requires_approval_before_commit() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("access-compile-actor-1").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_access_instance_with_permissions(
            &mut store,
            &actor,
            "tenant_1",
            "role.access_compile_admin",
            "{\"allow\":[\"ACCESS_INSTANCE_COMPILE_REFRESH\"]}",
            AccessMode::R,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
        );

        let out = exec.execute_ph1x_dispatch_simulation_candidate(
            &mut store,
            actor,
            MonotonicTimeNs(202),
            &access_x(
                1,
                access_draft(
                    IntentType::AccessInstanceCompileRefresh,
                    vec![access_field(FieldKey::TenantId, "tenant_1")],
                ),
                "idem-access-compile-escalate-1",
            ),
        );
        assert!(matches!(
            out,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.access_instance_compile.access_decision",
                    reason: "ACCESS_AP_REQUIRED",
                }
            ))
        ));
    }

    #[test]
    fn at_sim_exec_22_access_schema_manage_missing_review_channel_fails_closed() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("access-schema-actor-2").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_access_instance_with_permissions(
            &mut store,
            &actor,
            "tenant_1",
            "role.access_schema_admin",
            "{\"allow\":[\"ACCESS_SCHEMA_MANAGE\"]}",
            AccessMode::A,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
        );

        let out = exec.execute_ph1x_dispatch_simulation_candidate(
            &mut store,
            actor,
            MonotonicTimeNs(203),
            &access_x(
                1,
                access_draft(
                    IntentType::AccessSchemaManage,
                    vec![
                        access_field(FieldKey::TenantId, "tenant_1"),
                        access_field(FieldKey::ApAction, "CREATE_DRAFT"),
                        access_field(FieldKey::AccessProfileId, "AP_CLERK"),
                        access_field(FieldKey::SchemaVersionId, "v1"),
                        access_field(
                            FieldKey::ProfilePayloadJson,
                            "{\"allow\":[\"ACCESS_SCHEMA_MANAGE\"]}",
                        ),
                    ],
                ),
                "idem-access-schema-missing-channel-1",
            ),
        );

        assert!(matches!(
            out,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.intent_draft.fields",
                    reason: "missing required field",
                }
            ))
        ));
    }

    #[test]
    fn at_sim_exec_23_access_schema_manage_read_out_loud_gate_allow_returns_gate_passed() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("access-schema-actor-3").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_access_instance_with_permissions(
            &mut store,
            &actor,
            "tenant_1",
            "role.access_schema_admin",
            "{\"allow\":[\"ACCESS_SCHEMA_MANAGE\"]}",
            AccessMode::A,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
        );

        let out = exec
            .execute_ph1x_dispatch_simulation_candidate(
                &mut store,
                actor,
                MonotonicTimeNs(204),
                &access_x(
                    1,
                    access_draft(
                        IntentType::AccessSchemaManage,
                        vec![
                            access_field(FieldKey::TenantId, "tenant_1"),
                            access_field(FieldKey::ApAction, "CREATE_DRAFT"),
                            access_field(FieldKey::AccessProfileId, "AP_CEO"),
                            access_field(FieldKey::SchemaVersionId, "v1"),
                            access_field(FieldKey::AccessReviewChannel, "READ_OUT_LOUD"),
                            access_field(
                                FieldKey::ProfilePayloadJson,
                                "{\"allow\":[\"ACCESS_SCHEMA_MANAGE\"]}",
                            ),
                        ],
                    ),
                    "idem-access-schema-read-out-loud-1",
                ),
            )
            .unwrap();

        assert!(matches!(
            out,
            SimulationDispatchOutcome::AccessGatePassed {
                requested_action
            } if requested_action == "ACCESS_SCHEMA_MANAGE"
        ));
    }

    #[test]
    fn at_sim_exec_24_access_schema_manage_activate_missing_rule_action_fails_closed() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("access-schema-actor-4").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_access_instance_with_permissions(
            &mut store,
            &actor,
            "tenant_1",
            "role.access_schema_admin",
            "{\"allow\":[\"ACCESS_SCHEMA_MANAGE\"]}",
            AccessMode::A,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
        );

        let out = exec.execute_ph1x_dispatch_simulation_candidate(
            &mut store,
            actor,
            MonotonicTimeNs(205),
            &access_x(
                1,
                access_draft(
                    IntentType::AccessSchemaManage,
                    vec![
                        access_field(FieldKey::TenantId, "tenant_1"),
                        access_field(FieldKey::ApAction, "ACTIVATE"),
                        access_field(FieldKey::AccessProfileId, "AP_CEO"),
                        access_field(FieldKey::SchemaVersionId, "v2"),
                        access_field(FieldKey::AccessReviewChannel, "PHONE_DESKTOP"),
                    ],
                ),
                "idem-access-schema-activate-missing-rule-action-1",
            ),
        );

        assert!(matches!(
            out,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.intent_draft.fields",
                    reason: "missing required field",
                }
            ))
        ));
    }

    #[test]
    fn at_sim_exec_25_access_schema_manage_activate_rule_actions_bounded_and_validated() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("access-schema-actor-5").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_access_instance_with_permissions(
            &mut store,
            &actor,
            "tenant_1",
            "role.access_schema_admin",
            "{\"allow\":[\"ACCESS_SCHEMA_MANAGE\"]}",
            AccessMode::A,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
        );

        let allowed_actions = [
            "AGREE",
            "DISAGREE",
            "EDIT",
            "DELETE",
            "DISABLE",
            "ADD_CUSTOM_RULE",
        ];

        for (idx, action) in allowed_actions.iter().enumerate() {
            let review_channel = if idx % 2 == 0 {
                "PHONE_DESKTOP"
            } else {
                "READ_OUT_LOUD"
            };
            let out = exec
                .execute_ph1x_dispatch_simulation_candidate(
                    &mut store,
                    actor.clone(),
                    MonotonicTimeNs(206 + idx as u64),
                    &access_x(
                        idx as u64 + 1,
                        access_draft(
                            IntentType::AccessSchemaManage,
                            vec![
                                access_field(FieldKey::TenantId, "tenant_1"),
                                access_field(FieldKey::ApAction, "ACTIVATE"),
                                access_field(FieldKey::AccessProfileId, "AP_CEO"),
                                access_field(FieldKey::SchemaVersionId, &format!("v{}", idx + 1)),
                                access_field(FieldKey::AccessReviewChannel, review_channel),
                                access_field(FieldKey::AccessRuleAction, action),
                            ],
                        ),
                        &format!("idem-access-schema-activate-action-{}", idx + 1),
                    ),
                )
                .unwrap();
            assert!(matches!(
                out,
                SimulationDispatchOutcome::AccessGatePassed {
                    requested_action
                } if requested_action == "ACCESS_SCHEMA_MANAGE"
            ));
        }

        let invalid = exec.execute_ph1x_dispatch_simulation_candidate(
            &mut store,
            actor,
            MonotonicTimeNs(213),
            &access_x(
                999,
                access_draft(
                    IntentType::AccessSchemaManage,
                    vec![
                        access_field(FieldKey::TenantId, "tenant_1"),
                        access_field(FieldKey::ApAction, "ACTIVATE"),
                        access_field(FieldKey::AccessProfileId, "AP_CEO"),
                        access_field(FieldKey::SchemaVersionId, "v999"),
                        access_field(FieldKey::AccessReviewChannel, "PHONE_DESKTOP"),
                        access_field(FieldKey::AccessRuleAction, "UNBOUNDED_ACTION"),
                    ],
                ),
                "idem-access-schema-activate-action-invalid",
            ),
        );
        assert!(matches!(
            invalid,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.intent_draft.fields.access_rule_action",
                    reason:
                        "must be one of: AGREE, DISAGREE, EDIT, DELETE, DISABLE, ADD_CUSTOM_RULE",
                }
            ))
        ));
    }

    #[test]
    fn at_sim_exec_03_execute_position_create_draft_runs_ph1position_runtime() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("position-actor-1").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        store
            .ph1tenant_company_upsert(TenantCompanyRecord {
                schema_version: SchemaVersion(1),
                tenant_id: selene_kernel_contracts::ph1position::TenantId::new("tenant_1").unwrap(),
                company_id: "company_1".to_string(),
                legal_name: "Selene Inc".to_string(),
                jurisdiction: "CN".to_string(),
                lifecycle_state: TenantCompanyLifecycleState::Active,
                created_at: MonotonicTimeNs(1),
                updated_at: MonotonicTimeNs(1),
            })
            .unwrap();

        let req = Ph1PositionRequest {
            schema_version: PH1POSITION_CONTRACT_VERSION,
            correlation_id: CorrelationId(900),
            turn_id: TurnId(1),
            now: MonotonicTimeNs(123),
            simulation_id: POSITION_SIM_001_CREATE_DRAFT.to_string(),
            simulation_type: PositionSimulationType::Draft,
            request: PositionRequest::CreateDraft(PositionCreateDraftRequest {
                actor_user_id: actor,
                tenant_id: selene_kernel_contracts::ph1position::TenantId::new("tenant_1").unwrap(),
                company_id: "company_1".to_string(),
                position_title: "Store Manager".to_string(),
                department: "Retail".to_string(),
                jurisdiction: "CN".to_string(),
                schedule_type: PositionScheduleType::FullTime,
                permission_profile_ref: "profile_store_mgr".to_string(),
                compensation_band_ref: "band_l3".to_string(),
                idempotency_key: "position-create-1".to_string(),
            }),
        };

        let out = exec.execute_position(&mut store, &req).unwrap();
        match out {
            Ph1PositionResponse::Ok(ok) => {
                assert_eq!(ok.simulation_id, POSITION_SIM_001_CREATE_DRAFT);
                assert!(ok.create_draft_result.is_some());
            }
            Ph1PositionResponse::Refuse(_) => panic!("expected ok"),
        }
    }

    #[test]
    fn at_sim_exec_04_execute_voice_id_enroll_start_runs_ph1voiceid_runtime() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("voice-actor-1").unwrap();
        let device_id = DeviceId::new("voice-device-1").unwrap();

        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        store
            .insert_device(
                DeviceRecord::v1(
                    device_id.clone(),
                    actor.clone(),
                    "phone".to_string(),
                    MonotonicTimeNs(1),
                    Some("audio_prof_vid_1".to_string()),
                )
                .unwrap(),
            )
            .unwrap();

        let (link_rec, _) = store
            .ph1link_invite_generate_draft(
                MonotonicTimeNs(2),
                actor.clone(),
                InviteeType::Employee,
                Some("tenant_1".to_string()),
                None,
                None,
                None,
            )
            .unwrap();
        let _ = store
            .ph1link_invite_open_activate_commit(
                MonotonicTimeNs(3),
                link_rec.token_id.clone(),
                "voice-device-fp-1".to_string(),
            )
            .unwrap();
        let onb = store
            .ph1onb_session_start_draft(
                MonotonicTimeNs(4),
                link_rec.token_id,
                None,
                Some("tenant_1".to_string()),
                "voice-device-fp-1".to_string(),
            )
            .unwrap();

        let req = Ph1VoiceIdSimRequest {
            schema_version: PH1VOICEID_SIM_CONTRACT_VERSION,
            correlation_id: CorrelationId(1001),
            turn_id: TurnId(1),
            now: MonotonicTimeNs(10),
            simulation_id: VOICE_ID_ENROLL_START_DRAFT.to_string(),
            simulation_type: VoiceIdSimulationType::Draft,
            request: VoiceIdSimulationRequest::EnrollStartDraft(VoiceIdEnrollStartDraftRequest {
                onboarding_session_id: onb.onboarding_session_id.as_str().to_string(),
                device_id,
                consent_asserted: true,
                max_total_attempts: 8,
                max_session_enroll_time_ms: 120_000,
                lock_after_consecutive_passes: 3,
            }),
        };

        let out = exec.execute_voice_id(&mut store, &req).unwrap();
        match out {
            Ph1VoiceIdSimResponse::Ok(ok) => {
                assert_eq!(ok.simulation_id, VOICE_ID_ENROLL_START_DRAFT);
                assert!(ok.enroll_start_result.is_some());
            }
            Ph1VoiceIdSimResponse::Refuse(_) => panic!("expected ok"),
        }
    }
}
