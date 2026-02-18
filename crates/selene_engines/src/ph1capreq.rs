#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1capreq::{
    CapabilityRequestAction, CapabilityRequestStatus, CapreqId, CapreqRequest, Ph1CapreqRequest,
};
use selene_kernel_contracts::{ContractViolation, ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.CAPREQ.001 reason-code namespace. Values are placeholders until registry finalization.
    pub const CAPREQ_CREATED: ReasonCodeId = ReasonCodeId(0xCA00_0001);
    pub const CAPREQ_SUBMITTED: ReasonCodeId = ReasonCodeId(0xCA00_0002);
    pub const CAPREQ_APPROVED: ReasonCodeId = ReasonCodeId(0xCA00_0003);
    pub const CAPREQ_REJECTED: ReasonCodeId = ReasonCodeId(0xCA00_0004);
    pub const CAPREQ_FULFILLED: ReasonCodeId = ReasonCodeId(0xCA00_0005);
    pub const CAPREQ_CANCELED: ReasonCodeId = ReasonCodeId(0xCA00_0006);
}

pub const PH1_CAPREQ_ENGINE_ID: &str = "PH1.CAPREQ";
pub const PH1_CAPREQ_IMPLEMENTATION_ID: &str = "PH1.CAPREQ.001";
pub const PH1_CAPREQ_ACTIVE_IMPLEMENTATION_IDS: &[&str] = &[PH1_CAPREQ_IMPLEMENTATION_ID];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capreq001Decision {
    pub capreq_id: CapreqId,
    pub action: CapabilityRequestAction,
    pub next_status: CapabilityRequestStatus,
    pub reason_code: ReasonCodeId,
    pub payload_hash: String,
}

#[derive(Debug, Default, Clone)]
pub struct Ph1Capreq001Runtime;

impl Ph1Capreq001Runtime {
    pub fn evaluate(
        &self,
        req: &Ph1CapreqRequest,
        current_status: Option<CapabilityRequestStatus>,
    ) -> Result<Capreq001Decision, ContractViolation> {
        req.validate()?;

        match &req.request {
            CapreqRequest::CreateDraft(r) => {
                if current_status.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "capreq_transition",
                        reason: "create_draft requires no existing CAPREQ row",
                    });
                }
                let capreq_id = CapreqId::new(format!(
                    "capreq_{}",
                    short_hash_hex(&[
                        r.tenant_id.as_str(),
                        r.requested_capability_id.as_str(),
                        r.target_scope_ref.as_str(),
                        r.justification.as_str(),
                    ])
                ))?;
                Ok(Capreq001Decision {
                    capreq_id: capreq_id.clone(),
                    action: CapabilityRequestAction::CreateDraft,
                    next_status: CapabilityRequestStatus::Draft,
                    reason_code: reason_codes::CAPREQ_CREATED,
                    payload_hash: deterministic_capreq_payload_hash(
                        CapabilityRequestAction::CreateDraft,
                        r.tenant_id.as_str(),
                        capreq_id.as_str(),
                        Some(r.requested_capability_id.as_str()),
                        Some(r.target_scope_ref.as_str()),
                        Some(r.justification.as_str()),
                    ),
                })
            }
            CapreqRequest::SubmitForApprovalCommit(r) => {
                let next_status = validate_capreq_transition(
                    CapabilityRequestAction::SubmitForApproval,
                    current_status,
                )?;
                Ok(Capreq001Decision {
                    capreq_id: r.capreq_id.clone(),
                    action: CapabilityRequestAction::SubmitForApproval,
                    next_status,
                    reason_code: reason_codes::CAPREQ_SUBMITTED,
                    payload_hash: deterministic_capreq_payload_hash(
                        CapabilityRequestAction::SubmitForApproval,
                        r.tenant_id.as_str(),
                        r.capreq_id.as_str(),
                        None,
                        None,
                        None,
                    ),
                })
            }
            CapreqRequest::ApproveCommit(r) => {
                let next_status =
                    validate_capreq_transition(CapabilityRequestAction::Approve, current_status)?;
                Ok(Capreq001Decision {
                    capreq_id: r.capreq_id.clone(),
                    action: CapabilityRequestAction::Approve,
                    next_status,
                    reason_code: reason_codes::CAPREQ_APPROVED,
                    payload_hash: deterministic_capreq_payload_hash(
                        CapabilityRequestAction::Approve,
                        r.tenant_id.as_str(),
                        r.capreq_id.as_str(),
                        None,
                        None,
                        None,
                    ),
                })
            }
            CapreqRequest::RejectCommit(r) => {
                let next_status =
                    validate_capreq_transition(CapabilityRequestAction::Reject, current_status)?;
                Ok(Capreq001Decision {
                    capreq_id: r.capreq_id.clone(),
                    action: CapabilityRequestAction::Reject,
                    next_status,
                    reason_code: reason_codes::CAPREQ_REJECTED,
                    payload_hash: deterministic_capreq_payload_hash(
                        CapabilityRequestAction::Reject,
                        r.tenant_id.as_str(),
                        r.capreq_id.as_str(),
                        None,
                        None,
                        None,
                    ),
                })
            }
            CapreqRequest::FulfillCommit(r) => {
                let next_status =
                    validate_capreq_transition(CapabilityRequestAction::Fulfill, current_status)?;
                Ok(Capreq001Decision {
                    capreq_id: r.capreq_id.clone(),
                    action: CapabilityRequestAction::Fulfill,
                    next_status,
                    reason_code: reason_codes::CAPREQ_FULFILLED,
                    payload_hash: deterministic_capreq_payload_hash(
                        CapabilityRequestAction::Fulfill,
                        r.tenant_id.as_str(),
                        r.capreq_id.as_str(),
                        None,
                        None,
                        None,
                    ),
                })
            }
            CapreqRequest::CancelRevoke(r) => {
                let next_status =
                    validate_capreq_transition(CapabilityRequestAction::Cancel, current_status)?;
                Ok(Capreq001Decision {
                    capreq_id: r.capreq_id.clone(),
                    action: CapabilityRequestAction::Cancel,
                    next_status,
                    reason_code: reason_codes::CAPREQ_CANCELED,
                    payload_hash: deterministic_capreq_payload_hash(
                        CapabilityRequestAction::Cancel,
                        r.tenant_id.as_str(),
                        r.capreq_id.as_str(),
                        None,
                        None,
                        None,
                    ),
                })
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Ph1CapreqFamilyRuntime {
    v001: Ph1Capreq001Runtime,
}

impl Ph1CapreqFamilyRuntime {
    pub fn active_implementation_ids() -> &'static [&'static str] {
        PH1_CAPREQ_ACTIVE_IMPLEMENTATION_IDS
    }

    pub fn evaluate_for_implementation(
        &self,
        implementation_id: &str,
        req: &Ph1CapreqRequest,
        current_status: Option<CapabilityRequestStatus>,
    ) -> Result<Capreq001Decision, ContractViolation> {
        match implementation_id {
            PH1_CAPREQ_IMPLEMENTATION_ID => self.v001.evaluate(req, current_status),
            _ => Err(ContractViolation::InvalidValue {
                field: "ph1capreq.implementation_id",
                reason: "unknown implementation_id",
            }),
        }
    }

    pub fn evaluate(
        &self,
        req: &Ph1CapreqRequest,
        current_status: Option<CapabilityRequestStatus>,
    ) -> Result<Capreq001Decision, ContractViolation> {
        self.evaluate_for_implementation(PH1_CAPREQ_IMPLEMENTATION_ID, req, current_status)
    }
}

fn validate_capreq_transition(
    action: CapabilityRequestAction,
    current: Option<CapabilityRequestStatus>,
) -> Result<CapabilityRequestStatus, ContractViolation> {
    match action {
        CapabilityRequestAction::CreateDraft => {
            if current.is_some() {
                return Err(ContractViolation::InvalidValue {
                    field: "capreq_transition",
                    reason: "create_draft requires no existing CAPREQ row",
                });
            }
            Ok(CapabilityRequestStatus::Draft)
        }
        CapabilityRequestAction::SubmitForApproval => {
            if current != Some(CapabilityRequestStatus::Draft) {
                return Err(ContractViolation::InvalidValue {
                    field: "capreq_transition",
                    reason: "submit_for_approval requires current status DRAFT",
                });
            }
            Ok(CapabilityRequestStatus::PendingApproval)
        }
        CapabilityRequestAction::Approve => {
            if current != Some(CapabilityRequestStatus::PendingApproval) {
                return Err(ContractViolation::InvalidValue {
                    field: "capreq_transition",
                    reason: "approve requires current status PENDING_APPROVAL",
                });
            }
            Ok(CapabilityRequestStatus::Approved)
        }
        CapabilityRequestAction::Reject => {
            if current != Some(CapabilityRequestStatus::PendingApproval) {
                return Err(ContractViolation::InvalidValue {
                    field: "capreq_transition",
                    reason: "reject requires current status PENDING_APPROVAL",
                });
            }
            Ok(CapabilityRequestStatus::Rejected)
        }
        CapabilityRequestAction::Fulfill => {
            if current != Some(CapabilityRequestStatus::Approved) {
                return Err(ContractViolation::InvalidValue {
                    field: "capreq_transition",
                    reason: "fulfill requires current status APPROVED",
                });
            }
            Ok(CapabilityRequestStatus::Fulfilled)
        }
        CapabilityRequestAction::Cancel => {
            if !matches!(
                current,
                Some(CapabilityRequestStatus::Draft)
                    | Some(CapabilityRequestStatus::PendingApproval)
                    | Some(CapabilityRequestStatus::Approved)
            ) {
                return Err(ContractViolation::InvalidValue {
                    field: "capreq_transition",
                    reason: "cancel requires an open request (DRAFT|PENDING_APPROVAL|APPROVED)",
                });
            }
            Ok(CapabilityRequestStatus::Canceled)
        }
    }
}

fn deterministic_capreq_payload_hash(
    action: CapabilityRequestAction,
    tenant_id: &str,
    capreq_id: &str,
    requested_capability_id: Option<&str>,
    target_scope_ref: Option<&str>,
    justification: Option<&str>,
) -> String {
    let action_token = capreq_action_token(action);
    let requested_capability_id = requested_capability_id.unwrap_or("");
    let target_scope_ref = target_scope_ref.unwrap_or("");
    let justification = justification.unwrap_or("");
    let hex = short_hash_hex(&[
        action_token,
        tenant_id,
        capreq_id,
        requested_capability_id,
        target_scope_ref,
        justification,
    ]);
    format!("capreq_payload_{hex}")
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
        h ^= b'|' as u64;
        h = h.wrapping_mul(PRIME);
    }
    format!("{h:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1_voice_id::UserId;
    use selene_kernel_contracts::ph1capreq::Ph1CapreqRequest;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1position::TenantId;
    use selene_kernel_contracts::MonotonicTimeNs;

    fn tenant(v: &str) -> TenantId {
        TenantId::new(v).unwrap()
    }

    fn user(v: &str) -> UserId {
        UserId::new(v).unwrap()
    }

    #[test]
    fn at_capreq_01_tenant_isolation_keeps_ids_distinct_for_same_input_payload() {
        let runtime = Ph1Capreq001Runtime;
        let req_a = Ph1CapreqRequest::create_draft_v1(
            CorrelationId(1),
            TurnId(1),
            MonotonicTimeNs(10),
            user("user_1"),
            tenant("tenant_a"),
            "PH1.ACCESS.OVERRIDE".to_string(),
            "scope:finance".to_string(),
            "need temporary scope".to_string(),
            "idem_1".to_string(),
        )
        .unwrap();
        let req_b = Ph1CapreqRequest::create_draft_v1(
            CorrelationId(2),
            TurnId(1),
            MonotonicTimeNs(10),
            user("user_1"),
            tenant("tenant_b"),
            "PH1.ACCESS.OVERRIDE".to_string(),
            "scope:finance".to_string(),
            "need temporary scope".to_string(),
            "idem_1".to_string(),
        )
        .unwrap();

        let dec_a = runtime.evaluate(&req_a, None).unwrap();
        let dec_b = runtime.evaluate(&req_b, None).unwrap();

        assert_ne!(dec_a.capreq_id, dec_b.capreq_id);
        assert_ne!(dec_a.payload_hash, dec_b.payload_hash);
    }

    #[test]
    fn at_capreq_02_append_only_transition_rejects_approve_without_pending() {
        let runtime = Ph1Capreq001Runtime;
        let req = Ph1CapreqRequest::approve_commit_v1(
            CorrelationId(10),
            TurnId(2),
            MonotonicTimeNs(20),
            user("user_2"),
            tenant("tenant_a"),
            CapreqId::new("capreq_abc").unwrap(),
            "idem_approve".to_string(),
        )
        .unwrap();

        let out = runtime.evaluate(&req, Some(CapabilityRequestStatus::Draft));
        assert!(matches!(
            out,
            Err(ContractViolation::InvalidValue {
                field: "capreq_transition",
                reason: "approve requires current status PENDING_APPROVAL",
            })
        ));
    }

    #[test]
    fn at_capreq_03_idempotent_inputs_produce_deterministic_decision_payload() {
        let runtime = Ph1Capreq001Runtime;
        let req = Ph1CapreqRequest::create_draft_v1(
            CorrelationId(3),
            TurnId(1),
            MonotonicTimeNs(30),
            user("user_3"),
            tenant("tenant_c"),
            "PH1.POSITION.EDIT".to_string(),
            "scope:position:driver".to_string(),
            "need job-level override".to_string(),
            "idem_create".to_string(),
        )
        .unwrap();

        let d1 = runtime.evaluate(&req, None).unwrap();
        let d2 = runtime.evaluate(&req, None).unwrap();
        assert_eq!(d1.capreq_id, d2.capreq_id);
        assert_eq!(d1.payload_hash, d2.payload_hash);
        assert_eq!(d1.reason_code, reason_codes::CAPREQ_CREATED);
    }

    #[test]
    fn at_capreq_04_status_progression_is_deterministic() {
        let runtime = Ph1Capreq001Runtime;
        let capreq_id = CapreqId::new("capreq_progression").unwrap();

        let submit = Ph1CapreqRequest::submit_for_approval_commit_v1(
            CorrelationId(4),
            TurnId(2),
            MonotonicTimeNs(40),
            user("user_4"),
            tenant("tenant_d"),
            capreq_id.clone(),
            "idem_submit".to_string(),
        )
        .unwrap();
        let submit_dec = runtime
            .evaluate(&submit, Some(CapabilityRequestStatus::Draft))
            .unwrap();
        assert_eq!(
            submit_dec.next_status,
            CapabilityRequestStatus::PendingApproval
        );
        assert_eq!(submit_dec.reason_code, reason_codes::CAPREQ_SUBMITTED);

        let approve = Ph1CapreqRequest::approve_commit_v1(
            CorrelationId(4),
            TurnId(3),
            MonotonicTimeNs(41),
            user("user_4"),
            tenant("tenant_d"),
            capreq_id.clone(),
            "idem_approve".to_string(),
        )
        .unwrap();
        let approve_dec = runtime
            .evaluate(&approve, Some(CapabilityRequestStatus::PendingApproval))
            .unwrap();
        assert_eq!(approve_dec.next_status, CapabilityRequestStatus::Approved);
        assert_eq!(approve_dec.reason_code, reason_codes::CAPREQ_APPROVED);

        let fulfill = Ph1CapreqRequest::fulfill_commit_v1(
            CorrelationId(4),
            TurnId(4),
            MonotonicTimeNs(42),
            user("user_4"),
            tenant("tenant_d"),
            capreq_id,
            "idem_fulfill".to_string(),
        )
        .unwrap();
        let fulfill_dec = runtime
            .evaluate(&fulfill, Some(CapabilityRequestStatus::Approved))
            .unwrap();
        assert_eq!(fulfill_dec.next_status, CapabilityRequestStatus::Fulfilled);
        assert_eq!(fulfill_dec.reason_code, reason_codes::CAPREQ_FULFILLED);
    }

    #[test]
    fn at_capreq_family_01_dispatch_rejects_unknown_implementation() {
        let runtime = Ph1CapreqFamilyRuntime::default();
        let req = Ph1CapreqRequest::create_draft_v1(
            CorrelationId(5),
            TurnId(1),
            MonotonicTimeNs(50),
            user("user_5"),
            tenant("tenant_e"),
            "PH1.CAPREQ.MANAGE".to_string(),
            "scope:tenant".to_string(),
            "family dispatch".to_string(),
            "idem_family".to_string(),
        )
        .unwrap();

        let out = runtime.evaluate_for_implementation("PH1.CAPREQ.999", &req, None);
        assert!(matches!(
            out,
            Err(ContractViolation::InvalidValue {
                field: "ph1capreq.implementation_id",
                reason: "unknown implementation_id",
            })
        ));
    }

    #[test]
    fn at_capreq_family_02_dispatches_active_implementation_deterministically() {
        let runtime = Ph1CapreqFamilyRuntime::default();
        let req = Ph1CapreqRequest::create_draft_v1(
            CorrelationId(6),
            TurnId(1),
            MonotonicTimeNs(60),
            user("user_6"),
            tenant("tenant_f"),
            "PH1.CAPREQ.MANAGE".to_string(),
            "scope:tenant".to_string(),
            "family dispatch".to_string(),
            "idem_family".to_string(),
        )
        .unwrap();

        let decision = runtime
            .evaluate_for_implementation(PH1_CAPREQ_IMPLEMENTATION_ID, &req, None)
            .unwrap();
        assert_eq!(decision.next_status, CapabilityRequestStatus::Draft);
        assert_eq!(decision.reason_code, reason_codes::CAPREQ_CREATED);
        assert_eq!(
            Ph1CapreqFamilyRuntime::active_implementation_ids(),
            &["PH1.CAPREQ.001"]
        );
    }
}
