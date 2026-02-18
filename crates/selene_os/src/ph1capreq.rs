#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1capreq::{
    CapabilityRequestAction, CapabilityRequestLedgerEventInput, CapabilityRequestStatus,
    CapreqCreateDraftRequest, CapreqId, CapreqLifecycleResult, CapreqRequest, Ph1CapreqOk,
    Ph1CapreqRequest, Ph1CapreqResponse, PH1CAPREQ_IMPLEMENTATION_ID,
};
use selene_kernel_contracts::{ContractViolation, ReasonCodeId, Validate};
use selene_storage::ph1f::{Ph1fStore, StorageError};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.CAPREQ reason-code namespace. Values are placeholders until global registry finalization.
    pub const CAPREQ_CREATED: ReasonCodeId = ReasonCodeId(0xCA00_0001);
    pub const CAPREQ_SUBMITTED: ReasonCodeId = ReasonCodeId(0xCA00_0002);
    pub const CAPREQ_APPROVED: ReasonCodeId = ReasonCodeId(0xCA00_0003);
    pub const CAPREQ_REJECTED: ReasonCodeId = ReasonCodeId(0xCA00_0004);
    pub const CAPREQ_FULFILLED: ReasonCodeId = ReasonCodeId(0xCA00_0005);
    pub const CAPREQ_CANCELED: ReasonCodeId = ReasonCodeId(0xCA00_0006);
}

pub const PH1_CAPREQ_ENGINE_ID: &str = "PH1.CAPREQ";
pub const PH1_CAPREQ_ACTIVE_IMPLEMENTATION_IDS: &[&str] = &[PH1CAPREQ_IMPLEMENTATION_ID];

#[derive(Debug, Default, Clone)]
pub struct Ph1CapreqRuntime;

impl Ph1CapreqRuntime {
    pub fn run(
        &self,
        store: &mut Ph1fStore,
        req: &Ph1CapreqRequest,
    ) -> Result<Ph1CapreqResponse, StorageError> {
        self.run_for_implementation(store, PH1CAPREQ_IMPLEMENTATION_ID, req)
    }

    pub fn run_for_implementation(
        &self,
        store: &mut Ph1fStore,
        implementation_id: &str,
        req: &Ph1CapreqRequest,
    ) -> Result<Ph1CapreqResponse, StorageError> {
        validate_capreq_implementation_id(implementation_id)?;
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

fn validate_capreq_implementation_id(implementation_id: &str) -> Result<(), StorageError> {
    match implementation_id {
        PH1CAPREQ_IMPLEMENTATION_ID => Ok(()),
        _ => Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "ph1capreq.implementation_id",
                reason: "unknown implementation_id",
            },
        )),
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
        CapabilityRequestAction::CreateDraft => reason_codes::CAPREQ_CREATED,
        CapabilityRequestAction::SubmitForApproval => reason_codes::CAPREQ_SUBMITTED,
        CapabilityRequestAction::Approve => reason_codes::CAPREQ_APPROVED,
        CapabilityRequestAction::Reject => reason_codes::CAPREQ_REJECTED,
        CapabilityRequestAction::Fulfill => reason_codes::CAPREQ_FULFILLED,
        CapabilityRequestAction::Cancel => reason_codes::CAPREQ_CANCELED,
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

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1_voice_id::UserId;
    use selene_kernel_contracts::ph1capreq::{CapabilityRequestStatus, Ph1CapreqResponse};
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1position::TenantId;
    use selene_kernel_contracts::MonotonicTimeNs;
    use selene_storage::ph1f::{IdentityRecord, IdentityStatus};

    fn seed_identity(store: &mut Ph1fStore, user_id: &str) -> UserId {
        let user_id = UserId::new(user_id).unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                user_id.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        user_id
    }

    fn tenant(id: &str) -> TenantId {
        TenantId::new(id).unwrap()
    }

    fn lifecycle_out(
        resp: Ph1CapreqResponse,
    ) -> selene_kernel_contracts::ph1capreq::CapreqLifecycleResult {
        match resp {
            Ph1CapreqResponse::Ok(ok) => ok.lifecycle_result,
            Ph1CapreqResponse::Refuse(_) => panic!("expected ok"),
        }
    }

    #[test]
    fn capreq_create_submit_approve_happy_path() {
        let rt = Ph1CapreqRuntime;
        let mut store = Ph1fStore::new_in_memory();
        let actor = seed_identity(&mut store, "tenant_a:user_alice");
        let tenant_id = tenant("tenant_a");

        let create = Ph1CapreqRequest::create_draft_v1(
            CorrelationId(1),
            TurnId(1),
            MonotonicTimeNs(10),
            actor.clone(),
            tenant_id.clone(),
            "PH1.POSITION.REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT".to_string(),
            "company:tenant_a:company_1:position:driver".to_string(),
            "require AP approval for current+new rollout".to_string(),
            "capreq-create-1".to_string(),
        )
        .unwrap();
        let created = lifecycle_out(rt.run(&mut store, &create).unwrap());
        assert_eq!(created.status, CapabilityRequestStatus::Draft);

        let submit = Ph1CapreqRequest::submit_for_approval_commit_v1(
            CorrelationId(1),
            TurnId(2),
            MonotonicTimeNs(11),
            actor.clone(),
            tenant_id.clone(),
            created.capreq_id.clone(),
            "capreq-submit-1".to_string(),
        )
        .unwrap();
        let submitted = lifecycle_out(rt.run(&mut store, &submit).unwrap());
        assert_eq!(submitted.status, CapabilityRequestStatus::PendingApproval);

        let approve = Ph1CapreqRequest::approve_commit_v1(
            CorrelationId(1),
            TurnId(3),
            MonotonicTimeNs(12),
            actor,
            tenant_id.clone(),
            submitted.capreq_id.clone(),
            "capreq-approve-1".to_string(),
        )
        .unwrap();
        let approved = lifecycle_out(rt.run(&mut store, &approve).unwrap());
        assert_eq!(approved.status, CapabilityRequestStatus::Approved);

        let current = store
            .capreq_current_row(&tenant_id, &approved.capreq_id)
            .expect("current row");
        assert_eq!(current.status, CapabilityRequestStatus::Approved);
    }

    #[test]
    fn capreq_create_submit_reject_happy_path() {
        let rt = Ph1CapreqRuntime;
        let mut store = Ph1fStore::new_in_memory();
        let actor = seed_identity(&mut store, "tenant_b:user_bob");
        let tenant_id = tenant("tenant_b");

        let create = Ph1CapreqRequest::create_draft_v1(
            CorrelationId(2),
            TurnId(1),
            MonotonicTimeNs(20),
            actor.clone(),
            tenant_id.clone(),
            "PH1.ONB.REQUIREMENT_BACKFILL_START_DRAFT".to_string(),
            "company:tenant_b:company_9:position:warehouse_manager".to_string(),
            "request temporary exception denied".to_string(),
            "capreq-create-2".to_string(),
        )
        .unwrap();
        let created = lifecycle_out(rt.run(&mut store, &create).unwrap());
        assert_eq!(created.status, CapabilityRequestStatus::Draft);

        let submit = Ph1CapreqRequest::submit_for_approval_commit_v1(
            CorrelationId(2),
            TurnId(2),
            MonotonicTimeNs(21),
            actor.clone(),
            tenant_id.clone(),
            created.capreq_id.clone(),
            "capreq-submit-2".to_string(),
        )
        .unwrap();
        let submitted = lifecycle_out(rt.run(&mut store, &submit).unwrap());
        assert_eq!(submitted.status, CapabilityRequestStatus::PendingApproval);

        let reject = Ph1CapreqRequest::reject_commit_v1(
            CorrelationId(2),
            TurnId(3),
            MonotonicTimeNs(22),
            actor,
            tenant_id.clone(),
            submitted.capreq_id.clone(),
            "capreq-reject-2".to_string(),
        )
        .unwrap();
        let rejected = lifecycle_out(rt.run(&mut store, &reject).unwrap());
        assert_eq!(rejected.status, CapabilityRequestStatus::Rejected);

        let current = store
            .capreq_current_row(&tenant_id, &rejected.capreq_id)
            .expect("current row");
        assert_eq!(current.status, CapabilityRequestStatus::Rejected);
    }

    #[test]
    fn capreq_fail_closed_when_approve_without_pending_approval() {
        let rt = Ph1CapreqRuntime;
        let mut store = Ph1fStore::new_in_memory();
        let actor = seed_identity(&mut store, "tenant_c:user_carla");
        let tenant_id = tenant("tenant_c");

        let create = Ph1CapreqRequest::create_draft_v1(
            CorrelationId(3),
            TurnId(1),
            MonotonicTimeNs(30),
            actor.clone(),
            tenant_id.clone(),
            "PH1.POSITION.REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT".to_string(),
            "company:tenant_c:company_2:position:driver".to_string(),
            "draft only".to_string(),
            "capreq-create-3".to_string(),
        )
        .unwrap();
        let created = lifecycle_out(rt.run(&mut store, &create).unwrap());
        assert_eq!(created.status, CapabilityRequestStatus::Draft);

        let approve_without_submit = Ph1CapreqRequest::approve_commit_v1(
            CorrelationId(3),
            TurnId(2),
            MonotonicTimeNs(31),
            actor,
            tenant_id,
            created.capreq_id,
            "capreq-approve-without-submit".to_string(),
        )
        .unwrap();

        let out = rt.run(&mut store, &approve_without_submit);
        assert!(matches!(
            out,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "capreq_transition",
                    reason: "approve requires current status PENDING_APPROVAL",
                }
            ))
        ));
    }

    #[test]
    fn at_capreq_family_01_unknown_implementation_fails_closed() {
        let rt = Ph1CapreqRuntime;
        let mut store = Ph1fStore::new_in_memory();
        let actor = seed_identity(&mut store, "tenant_d:user_dan");
        let tenant_id = tenant("tenant_d");

        let create = Ph1CapreqRequest::create_draft_v1(
            CorrelationId(4),
            TurnId(1),
            MonotonicTimeNs(40),
            actor,
            tenant_id,
            "PH1.CAPREQ.MANAGE".to_string(),
            "scope:tenant_d".to_string(),
            "family isolation".to_string(),
            "capreq-create-4".to_string(),
        )
        .unwrap();

        let out = rt.run_for_implementation(&mut store, "PH1.CAPREQ.999", &create);
        assert!(matches!(
            out,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1capreq.implementation_id",
                    reason: "unknown implementation_id",
                }
            ))
        ));
    }

    #[test]
    fn at_capreq_family_02_active_implementation_list_is_locked() {
        assert_eq!(PH1_CAPREQ_ENGINE_ID, "PH1.CAPREQ");
        assert_eq!(PH1_CAPREQ_ACTIVE_IMPLEMENTATION_IDS, &["PH1.CAPREQ.001"]);
    }
}
