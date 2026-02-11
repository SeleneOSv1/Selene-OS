#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1capreq::{
    CapabilityRequestAction, CapabilityRequestLedgerEventInput, CapabilityRequestStatus,
    CapreqCreateDraftRequest, CapreqId, CapreqLifecycleResult, CapreqRequest, Ph1CapreqOk,
    Ph1CapreqRequest, Ph1CapreqResponse,
};
use selene_kernel_contracts::{ContractViolation, ReasonCodeId, Validate};
use selene_storage::ph1f::{Ph1fStore, StorageError};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.CAPREQ reason-code namespace. Values are placeholders until global registry finalization.
    pub const CAPREQ_OK_CREATE_DRAFT: ReasonCodeId = ReasonCodeId(0xCA00_0001);
    pub const CAPREQ_OK_SUBMIT_FOR_APPROVAL: ReasonCodeId = ReasonCodeId(0xCA00_0002);
    pub const CAPREQ_OK_APPROVE: ReasonCodeId = ReasonCodeId(0xCA00_0003);
    pub const CAPREQ_OK_REJECT: ReasonCodeId = ReasonCodeId(0xCA00_0004);
    pub const CAPREQ_OK_FULFILL: ReasonCodeId = ReasonCodeId(0xCA00_0005);
    pub const CAPREQ_OK_CANCEL: ReasonCodeId = ReasonCodeId(0xCA00_0006);
}

#[derive(Debug, Default, Clone)]
pub struct Ph1CapreqRuntime;

impl Ph1CapreqRuntime {
    pub fn run(
        &self,
        store: &mut Ph1fStore,
        req: &Ph1CapreqRequest,
    ) -> Result<Ph1CapreqResponse, StorageError> {
        req.validate().map_err(StorageError::ContractViolation)?;

        match &req.request {
            CapreqRequest::CreateDraft(r) => self.run_create_draft(store, req, r),
            CapreqRequest::SubmitForApprovalCommit(r) => self.run_lifecycle_action(
                store,
                req,
                r.actor_user_id.clone(),
                r.tenant_id.clone(),
                r.capreq_id.clone(),
                CapabilityRequestAction::SubmitForApproval,
                None,
                Some(r.idempotency_key.clone()),
            ),
            CapreqRequest::ApproveCommit(r) => self.run_lifecycle_action(
                store,
                req,
                r.actor_user_id.clone(),
                r.tenant_id.clone(),
                r.capreq_id.clone(),
                CapabilityRequestAction::Approve,
                None,
                Some(r.idempotency_key.clone()),
            ),
            CapreqRequest::RejectCommit(r) => self.run_lifecycle_action(
                store,
                req,
                r.actor_user_id.clone(),
                r.tenant_id.clone(),
                r.capreq_id.clone(),
                CapabilityRequestAction::Reject,
                None,
                Some(r.idempotency_key.clone()),
            ),
            CapreqRequest::FulfillCommit(r) => self.run_lifecycle_action(
                store,
                req,
                r.actor_user_id.clone(),
                r.tenant_id.clone(),
                r.capreq_id.clone(),
                CapabilityRequestAction::Fulfill,
                None,
                Some(r.idempotency_key.clone()),
            ),
            CapreqRequest::CancelRevoke(r) => self.run_lifecycle_action(
                store,
                req,
                r.actor_user_id.clone(),
                r.tenant_id.clone(),
                r.capreq_id.clone(),
                CapabilityRequestAction::Cancel,
                None,
                Some(r.idempotency_key.clone()),
            ),
        }
    }

    fn run_create_draft(
        &self,
        store: &mut Ph1fStore,
        req: &Ph1CapreqRequest,
        r: &CapreqCreateDraftRequest,
    ) -> Result<Ph1CapreqResponse, StorageError> {
        let capreq_id = derive_capreq_id(
            &r.tenant_id,
            &r.requested_capability_id,
            &r.target_scope_ref,
            &r.justification,
        )
        .map_err(StorageError::ContractViolation)?;

        self.run_lifecycle_action(
            store,
            req,
            r.actor_user_id.clone(),
            r.tenant_id.clone(),
            capreq_id,
            CapabilityRequestAction::CreateDraft,
            Some(CapreqPayloadSnapshot {
                requested_capability_id: Some(r.requested_capability_id.as_str()),
                target_scope_ref: Some(r.target_scope_ref.as_str()),
                justification: Some(r.justification.as_str()),
            }),
            Some(r.idempotency_key.clone()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn run_lifecycle_action(
        &self,
        store: &mut Ph1fStore,
        req: &Ph1CapreqRequest,
        actor_user_id: selene_kernel_contracts::ph1_voice_id::UserId,
        tenant_id: selene_kernel_contracts::ph1position::TenantId,
        capreq_id: CapreqId,
        action: CapabilityRequestAction,
        payload_snapshot: Option<CapreqPayloadSnapshot<'_>>,
        idempotency_key: Option<String>,
    ) -> Result<Ph1CapreqResponse, StorageError> {
        let current_status = store
            .capreq_current_row(&tenant_id, &capreq_id)
            .map(|row| row.status);
        let next_status = validate_capreq_transition(action, current_status)?;

        let payload_hash = deterministic_capreq_payload_hash(
            action,
            tenant_id.as_str(),
            capreq_id.as_str(),
            payload_snapshot
                .as_ref()
                .and_then(|v| v.requested_capability_id),
            payload_snapshot.as_ref().and_then(|v| v.target_scope_ref),
            payload_snapshot.as_ref().and_then(|v| v.justification),
        );

        let input = CapabilityRequestLedgerEventInput::v1(
            req.now,
            tenant_id,
            capreq_id.clone(),
            actor_user_id,
            action,
            next_status,
            capreq_reason_code(action),
            payload_hash,
            idempotency_key,
        )?;
        let capreq_event_id = store.append_capreq_event(input)?;

        let lifecycle_result =
            CapreqLifecycleResult::v1(capreq_id, capreq_event_id, action, next_status)
                .map_err(StorageError::ContractViolation)?;

        let ok = Ph1CapreqOk::v1(
            req.simulation_id.clone(),
            capreq_reason_code(action),
            lifecycle_result,
        )
        .map_err(StorageError::ContractViolation)?;
        Ok(Ph1CapreqResponse::Ok(ok))
    }
}

#[derive(Debug, Clone, Copy)]
struct CapreqPayloadSnapshot<'a> {
    requested_capability_id: Option<&'a str>,
    target_scope_ref: Option<&'a str>,
    justification: Option<&'a str>,
}

fn capreq_reason_code(action: CapabilityRequestAction) -> ReasonCodeId {
    match action {
        CapabilityRequestAction::CreateDraft => reason_codes::CAPREQ_OK_CREATE_DRAFT,
        CapabilityRequestAction::SubmitForApproval => reason_codes::CAPREQ_OK_SUBMIT_FOR_APPROVAL,
        CapabilityRequestAction::Approve => reason_codes::CAPREQ_OK_APPROVE,
        CapabilityRequestAction::Reject => reason_codes::CAPREQ_OK_REJECT,
        CapabilityRequestAction::Fulfill => reason_codes::CAPREQ_OK_FULFILL,
        CapabilityRequestAction::Cancel => reason_codes::CAPREQ_OK_CANCEL,
    }
}

fn derive_capreq_id(
    tenant_id: &selene_kernel_contracts::ph1position::TenantId,
    requested_capability_id: &str,
    target_scope_ref: &str,
    justification: &str,
) -> Result<CapreqId, ContractViolation> {
    CapreqId::new(format!(
        "capreq_{}",
        short_hash_hex(&[
            tenant_id.as_str(),
            requested_capability_id,
            target_scope_ref,
            justification,
        ])
    ))
}

fn validate_capreq_transition(
    action: CapabilityRequestAction,
    current: Option<CapabilityRequestStatus>,
) -> Result<CapabilityRequestStatus, StorageError> {
    match action {
        CapabilityRequestAction::CreateDraft => {
            if current.is_some() {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "capreq_transition",
                        reason: "create_draft requires no existing CAPREQ row",
                    },
                ));
            }
            Ok(CapabilityRequestStatus::Draft)
        }
        CapabilityRequestAction::SubmitForApproval => {
            if current != Some(CapabilityRequestStatus::Draft) {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "capreq_transition",
                        reason: "submit_for_approval requires current status DRAFT",
                    },
                ));
            }
            Ok(CapabilityRequestStatus::PendingApproval)
        }
        CapabilityRequestAction::Approve => {
            if current != Some(CapabilityRequestStatus::PendingApproval) {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "capreq_transition",
                        reason: "approve requires current status PENDING_APPROVAL",
                    },
                ));
            }
            Ok(CapabilityRequestStatus::Approved)
        }
        CapabilityRequestAction::Reject => {
            if current != Some(CapabilityRequestStatus::PendingApproval) {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "capreq_transition",
                        reason: "reject requires current status PENDING_APPROVAL",
                    },
                ));
            }
            Ok(CapabilityRequestStatus::Rejected)
        }
        CapabilityRequestAction::Fulfill => {
            if current != Some(CapabilityRequestStatus::Approved) {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "capreq_transition",
                        reason: "fulfill requires current status APPROVED",
                    },
                ));
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
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "capreq_transition",
                        reason: "cancel requires an open request (DRAFT|PENDING_APPROVAL|APPROVED)",
                    },
                ));
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
        // Stable delimiter to avoid accidental concatenation ambiguity.
        h ^= b'|' as u64;
        h = h.wrapping_mul(PRIME);
    }
    format!("{h:016x}")
}
