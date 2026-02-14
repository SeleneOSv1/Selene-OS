#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use selene_kernel_contracts::ph1j::{
    AuditEngine, AuditEventInput, AuditEventType, AuditPayloadMin, AuditSeverity, PayloadKey,
    PayloadValue,
};
use selene_kernel_contracts::ph1link::{
    DualRoleConflictEscalationResult, EscalationStatus, LinkActivationResult,
    LinkExpiredRecoveryResult, LinkGenerateResult, LinkRevokeResult, LinkStatus, Ph1LinkOk,
    Ph1LinkRequest, Ph1LinkResponse, PrefilledContextRef, RoleProposalResult, RoleProposalStatus,
};
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId, Validate};
use selene_storage::ph1f::{Ph1fStore, StorageError};
use selene_storage::ph1j::Ph1jRuntime;

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.LINK reason-code namespace. Values are placeholders until the global registry is formalized.
    pub const LINK_OK_GENERATE_DRAFT: ReasonCodeId = ReasonCodeId(0x4E00_0001);
    pub const LINK_OK_OPEN_ACTIVATE: ReasonCodeId = ReasonCodeId(0x4E00_0003);
    pub const LINK_OK_REVOKE: ReasonCodeId = ReasonCodeId(0x4E00_0005);
    pub const LINK_OK_EXPIRED_RECOVERY_COMMIT: ReasonCodeId = ReasonCodeId(0x4E00_0006);
    pub const LINK_OK_FORWARD_BLOCK_COMMIT: ReasonCodeId = ReasonCodeId(0x4E00_0007);
    pub const LINK_OK_ROLE_PROPOSE_DRAFT: ReasonCodeId = ReasonCodeId(0x4E00_0008);
    pub const LINK_OK_DUAL_ROLE_CONFLICT_ESCALATE_DRAFT: ReasonCodeId = ReasonCodeId(0x4E00_0009);
    pub const LINK_REFUSE_INVALID: ReasonCodeId = ReasonCodeId(0x4E00_00F1);
    pub const LINK_REFUSE_NOT_FOUND: ReasonCodeId = ReasonCodeId(0x4E00_00F2);
    pub const LINK_REFUSE_NOT_IMPLEMENTED: ReasonCodeId = ReasonCodeId(0x4E00_00F3);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1LinkConfig {
    pub default_ttl_days: u16,
    pub base_url: &'static str,
}

impl Ph1LinkConfig {
    pub fn mvp_v1() -> Self {
        Self {
            default_ttl_days: 7,
            base_url: "https://selene.app/invite",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1LinkRuntime {
    config: Ph1LinkConfig,
}

impl Ph1LinkRuntime {
    pub fn new(config: Ph1LinkConfig) -> Self {
        Self { config }
    }

    pub fn run(
        &self,
        store: &mut Ph1fStore,
        req: &Ph1LinkRequest,
    ) -> Result<Ph1LinkResponse, StorageError> {
        // Fail closed on any contract mismatch.
        req.validate().map_err(StorageError::ContractViolation)?;

        match &req.request {
            selene_kernel_contracts::ph1link::LinkRequest::InviteGenerateDraft(r) => {
                let (link_rec, parts) = store.ph1link_invite_generate_draft(
                    req.now,
                    r.inviter_user_id.clone(),
                    r.invitee_type,
                    r.tenant_id.clone(),
                    r.schema_version_id.clone(),
                    r.prefilled_context.clone(),
                    r.expiration_policy_id.clone(),
                )?;

                if parts.was_new {
                    self.audit_transition(
                        store,
                        req.now,
                        req.correlation_id,
                        req.turn_id,
                        Some(link_rec.inviter_user_id.clone()),
                        "NONE",
                        "DRAFT_CREATED",
                        reason_codes::LINK_OK_GENERATE_DRAFT,
                        Some(format!("link_create:{}", link_rec.token_id.as_str())),
                    )?;
                }

                let ctx_ref = link_rec
                    .prefilled_context
                    .as_ref()
                    .map(|_| {
                        PrefilledContextRef::new(format!(
                            "prefilled:{}",
                            link_rec.token_id.as_str()
                        ))
                    })
                    .transpose()
                    .map_err(StorageError::ContractViolation)?;

                let link_url = format!("{}/{}", self.config.base_url, link_rec.token_id.as_str());
                let out = LinkGenerateResult::v1(
                    link_rec.draft_id.clone(),
                    link_rec.token_id.clone(),
                    link_url,
                    link_rec.missing_required_fields.clone(),
                    parts.payload_hash,
                    link_rec.expires_at,
                    link_rec.status,
                    ctx_ref,
                )
                .map_err(StorageError::ContractViolation)?;

                Ok(Ph1LinkResponse::Ok(
                    Ph1LinkOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::LINK_OK_GENERATE_DRAFT,
                        Some(out),
                        None,
                        None,
                        None,
                        None,
                        None,
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }

            selene_kernel_contracts::ph1link::LinkRequest::InviteOpenActivateCommit(r) => {
                let (
                    status,
                    draft_id,
                    missing_required_fields,
                    bound_hash,
                    conflict_reason,
                    ctx_ref,
                ) = store.ph1link_invite_open_activate_commit(
                    req.now,
                    r.token_id.clone(),
                    r.device_fingerprint.clone(),
                )?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    None,
                    "DRAFT_CREATED",
                    match status {
                        LinkStatus::DraftCreated => "DRAFT_CREATED",
                        LinkStatus::Sent => "SENT",
                        LinkStatus::Opened => "OPENED",
                        LinkStatus::Activated => "ACTIVATED",
                        LinkStatus::Consumed => "CONSUMED",
                        LinkStatus::Blocked => "BLOCKED",
                        LinkStatus::Expired => "EXPIRED",
                        LinkStatus::Revoked => "REVOKED",
                    },
                    reason_codes::LINK_OK_OPEN_ACTIVATE,
                    Some(format!("link_open:{}", r.token_id.as_str())),
                )?;

                let out = LinkActivationResult::v1(
                    r.token_id.clone(),
                    draft_id,
                    status,
                    missing_required_fields,
                    conflict_reason,
                    bound_hash,
                    ctx_ref,
                )
                .map_err(StorageError::ContractViolation)?;

                Ok(Ph1LinkResponse::Ok(
                    Ph1LinkOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::LINK_OK_OPEN_ACTIVATE,
                        None,
                        Some(out),
                        None,
                        None,
                        None,
                        None,
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }

            selene_kernel_contracts::ph1link::LinkRequest::InviteRevokeRevoke(r) => {
                store.ph1link_invite_revoke_revoke(r.token_id.clone(), r.reason.clone())?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    None,
                    "ANY",
                    "REVOKED",
                    reason_codes::LINK_OK_REVOKE,
                    Some(format!("link_revoke:{}", r.token_id.as_str())),
                )?;

                let out = LinkRevokeResult::v1(LinkStatus::Revoked)
                    .map_err(StorageError::ContractViolation)?;

                Ok(Ph1LinkResponse::Ok(
                    Ph1LinkOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::LINK_OK_REVOKE,
                        None,
                        None,
                        Some(out),
                        None,
                        None,
                        None,
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }

            selene_kernel_contracts::ph1link::LinkRequest::InviteExpiredRecoveryCommit(r) => {
                let idempotency_key = r
                    .idempotency_key
                    .clone()
                    .unwrap_or_else(|| "default".to_string());

                let new_link = store.ph1link_invite_expired_recovery_commit(
                    req.now,
                    r.token_id.clone(),
                    idempotency_key.clone(),
                )?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    Some(new_link.inviter_user_id.clone()),
                    "EXPIRED",
                    "RECOVERED",
                    reason_codes::LINK_OK_EXPIRED_RECOVERY_COMMIT,
                    Some(format!(
                        "link_recovery:{}:{}",
                        r.token_id.as_str(),
                        idempotency_key
                    )),
                )?;

                let new_link_url =
                    format!("{}/{}", self.config.base_url, new_link.token_id.as_str());
                let out = LinkExpiredRecoveryResult::v1(
                    new_link.token_id.clone(),
                    new_link.draft_id.clone(),
                    new_link_url,
                    new_link.missing_required_fields.clone(),
                )
                .map_err(StorageError::ContractViolation)?;

                Ok(Ph1LinkResponse::Ok(
                    Ph1LinkOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::LINK_OK_EXPIRED_RECOVERY_COMMIT,
                        None,
                        None,
                        None,
                        Some(out),
                        None,
                        None,
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }

            selene_kernel_contracts::ph1link::LinkRequest::InviteForwardBlockCommit(r) => {
                let (status, bound, draft_id, missing_required_fields, conflict_reason) = store
                    .ph1link_invite_forward_block_commit(
                        r.token_id.clone(),
                        r.device_fingerprint.clone(),
                    )?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    None,
                    "ANY",
                    "BLOCKED",
                    reason_codes::LINK_OK_FORWARD_BLOCK_COMMIT,
                    Some(format!("link_forward_block:{}", r.token_id.as_str())),
                )?;

                let out = LinkActivationResult::v1(
                    r.token_id.clone(),
                    draft_id,
                    status,
                    missing_required_fields,
                    conflict_reason,
                    bound,
                    None,
                )
                .map_err(StorageError::ContractViolation)?;

                Ok(Ph1LinkResponse::Ok(
                    Ph1LinkOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::LINK_OK_FORWARD_BLOCK_COMMIT,
                        None,
                        Some(out),
                        None,
                        None,
                        None,
                        None,
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }

            selene_kernel_contracts::ph1link::LinkRequest::RoleProposeDraft(r) => {
                let proposal_id = store.ph1link_role_propose_draft(
                    req.now,
                    r.tenant_id.clone(),
                    r.proposal_text.clone(),
                )?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    None,
                    "NONE",
                    "ROLE_PROPOSAL_DRAFTED",
                    reason_codes::LINK_OK_ROLE_PROPOSE_DRAFT,
                    Some(format!("link_role_propose:{proposal_id}")),
                )?;

                let out =
                    RoleProposalResult::v1(proposal_id, RoleProposalStatus::PendingApApproval)
                        .map_err(StorageError::ContractViolation)?;

                Ok(Ph1LinkResponse::Ok(
                    Ph1LinkOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::LINK_OK_ROLE_PROPOSE_DRAFT,
                        None,
                        None,
                        None,
                        None,
                        Some(out),
                        None,
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }

            selene_kernel_contracts::ph1link::LinkRequest::DualRoleConflictEscalateDraft(r) => {
                let case_id = store.ph1link_dual_role_conflict_escalate_draft(
                    req.now,
                    r.tenant_id.clone(),
                    r.token_id.clone(),
                    r.note.clone(),
                )?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    None,
                    "NONE",
                    "ESCALATED",
                    reason_codes::LINK_OK_DUAL_ROLE_CONFLICT_ESCALATE_DRAFT,
                    Some(format!("link_dual_role_conflict:{case_id}")),
                )?;

                let out =
                    DualRoleConflictEscalationResult::v1(case_id, EscalationStatus::Escalated)
                        .map_err(StorageError::ContractViolation)?;

                Ok(Ph1LinkResponse::Ok(
                    Ph1LinkOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::LINK_OK_DUAL_ROLE_CONFLICT_ESCALATE_DRAFT,
                        None,
                        None,
                        None,
                        None,
                        None,
                        Some(out),
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }
        }
    }

    fn audit_transition(
        &self,
        store: &mut Ph1fStore,
        now: MonotonicTimeNs,
        correlation_id: selene_kernel_contracts::ph1j::CorrelationId,
        turn_id: selene_kernel_contracts::ph1j::TurnId,
        user_id: Option<selene_kernel_contracts::ph1_voice_id::UserId>,
        state_from: &'static str,
        state_to: &'static str,
        reason_code: ReasonCodeId,
        idempotency_key: Option<String>,
    ) -> Result<(), StorageError> {
        let mut entries: BTreeMap<PayloadKey, PayloadValue> = BTreeMap::new();
        entries.insert(
            PayloadKey::new("state_from").map_err(StorageError::ContractViolation)?,
            PayloadValue::new(state_from).map_err(StorageError::ContractViolation)?,
        );
        entries.insert(
            PayloadKey::new("state_to").map_err(StorageError::ContractViolation)?,
            PayloadValue::new(state_to).map_err(StorageError::ContractViolation)?,
        );
        let payload_min = AuditPayloadMin::v1(entries).map_err(StorageError::ContractViolation)?;

        // Use Other("ph1_link") until the global audit engine enum is updated.
        let engine = AuditEngine::Other("ph1_link".to_string());

        let ev = AuditEventInput::v1(
            now,
            None,
            None,
            None,
            user_id,
            None,
            engine,
            AuditEventType::StateTransition,
            reason_code,
            AuditSeverity::Info,
            correlation_id,
            turn_id,
            payload_min,
            None,
            idempotency_key,
        )
        .map_err(StorageError::ContractViolation)?;

        Ph1jRuntime::emit(store, ev)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1_voice_id::{SpeakerId, UserId};
    use selene_kernel_contracts::ph1c::LanguageTag;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1link::{
        DualRoleConflictEscalateDraftRequest, InviteExpiredRecoveryCommitRequest,
        InviteForwardBlockCommitRequest, InviteeType, LinkRequest, Ph1LinkRequest,
        RoleProposeDraftRequest, SimulationType, LINK_INVITE_DUAL_ROLE_CONFLICT_ESCALATE_DRAFT,
        LINK_INVITE_EXPIRED_RECOVERY_COMMIT, LINK_INVITE_FORWARD_BLOCK_COMMIT,
        LINK_ROLE_PROPOSE_DRAFT,
    };
    use selene_storage::ph1f::{IdentityRecord, IdentityStatus};

    fn now(n: u64) -> MonotonicTimeNs {
        MonotonicTimeNs(n * 1_000_000_000)
    }

    fn user() -> UserId {
        UserId::new("user_inviter").unwrap()
    }

    fn store_with_inviter() -> Ph1fStore {
        let mut s = Ph1fStore::new_in_memory();
        s.insert_identity(IdentityRecord::v1(
            user(),
            Some(SpeakerId::new("spk_1").unwrap()),
            Some(LanguageTag::new("en-US").unwrap()),
            MonotonicTimeNs(1),
            IdentityStatus::Active,
        ))
        .unwrap();
        s
    }

    #[test]
    fn at_link_01_generate_draft_is_idempotent_and_hash_deterministic() {
        let mut store = store_with_inviter();
        let rt = Ph1LinkRuntime::new(Ph1LinkConfig::mvp_v1());

        let req1 = Ph1LinkRequest::invite_generate_draft_v1(
            CorrelationId(1),
            TurnId(1),
            now(10),
            user(),
            InviteeType::Employee,
            Some("tenant_1".to_string()),
            None,
            None,
            None,
        )
        .unwrap();

        let out1 = rt.run(&mut store, &req1).unwrap();
        let (link_id_1, payload_hash_1) = match out1 {
            Ph1LinkResponse::Ok(o) => {
                let g = o.link_generate_result.expect("generate result");
                (g.token_id.as_str().to_string(), g.payload_hash)
            }
            _ => panic!("expected OK"),
        };

        // Same inputs, new 'now' -> still returns the same link (idempotent on payload hash).
        let req2 = Ph1LinkRequest::invite_generate_draft_v1(
            CorrelationId(1),
            TurnId(2),
            now(11),
            user(),
            InviteeType::Employee,
            Some("tenant_1".to_string()),
            None,
            None,
            None,
        )
        .unwrap();

        let out2 = rt.run(&mut store, &req2).unwrap();
        let (link_id_2, payload_hash_2) = match out2 {
            Ph1LinkResponse::Ok(o) => {
                let g = o.link_generate_result.expect("generate result");
                (g.token_id.as_str().to_string(), g.payload_hash)
            }
            _ => panic!("expected OK"),
        };

        assert_eq!(payload_hash_1, payload_hash_2);
        assert_eq!(link_id_1, link_id_2);
    }

    #[test]
    fn at_link_03_open_binds_device_and_blocks_mismatch() {
        let mut store = store_with_inviter();
        let rt = Ph1LinkRuntime::new(Ph1LinkConfig::mvp_v1());

        let gen = Ph1LinkRequest::invite_generate_draft_v1(
            CorrelationId(2),
            TurnId(1),
            now(10),
            user(),
            InviteeType::FamilyMember,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.run(&mut store, &gen).unwrap();
        let token_id = match out {
            Ph1LinkResponse::Ok(o) => o.link_generate_result.unwrap().token_id,
            _ => panic!("expected OK"),
        };

        let open1 = Ph1LinkRequest::invite_open_activate_commit_v1(
            CorrelationId(2),
            TurnId(2),
            now(20),
            token_id.clone(),
            "device_fp_a".to_string(),
        )
        .unwrap();

        let out1 = rt.run(&mut store, &open1).unwrap();
        match out1 {
            Ph1LinkResponse::Ok(o) => {
                let a = o.link_activation_result.unwrap();
                assert_eq!(a.activation_status, LinkStatus::Activated);
                assert!(a.bound_device_fingerprint_hash.is_some());
            }
            _ => panic!("expected OK"),
        }

        let open2 = Ph1LinkRequest::invite_open_activate_commit_v1(
            CorrelationId(2),
            TurnId(3),
            now(21),
            token_id,
            "device_fp_b".to_string(),
        )
        .unwrap();

        let out2 = rt.run(&mut store, &open2).unwrap();
        match out2 {
            Ph1LinkResponse::Ok(o) => {
                let a = o.link_activation_result.unwrap();
                assert_eq!(a.activation_status, LinkStatus::Blocked);
            }
            _ => panic!("expected OK"),
        }
    }

    #[test]
    fn at_link_06_expired_recovery_creates_replacement_and_is_idempotent() {
        let mut store = store_with_inviter();
        let rt = Ph1LinkRuntime::new(Ph1LinkConfig::mvp_v1());

        let gen = Ph1LinkRequest::invite_generate_draft_v1(
            CorrelationId(6),
            TurnId(1),
            now(10),
            user(),
            InviteeType::Employee,
            Some("tenant_1".to_string()),
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.run(&mut store, &gen).unwrap();
        let expired_link_id = match out {
            Ph1LinkResponse::Ok(o) => o.link_generate_result.unwrap().token_id,
            _ => panic!("expected OK"),
        };

        // Force-expire by jumping time beyond TTL.
        let recover_req = Ph1LinkRequest {
            schema_version: selene_kernel_contracts::ph1link::PH1LINK_CONTRACT_VERSION,
            correlation_id: CorrelationId(6),
            turn_id: TurnId(2),
            now: now(604_900),
            simulation_id: LINK_INVITE_EXPIRED_RECOVERY_COMMIT.to_string(),
            simulation_type: SimulationType::Commit,
            request: LinkRequest::InviteExpiredRecoveryCommit(InviteExpiredRecoveryCommitRequest {
                token_id: expired_link_id.clone(),
                idempotency_key: Some("idem_recover_1".to_string()),
            }),
        };

        let out1 = rt.run(&mut store, &recover_req).unwrap();
        let new_id_1 = match out1 {
            Ph1LinkResponse::Ok(o) => {
                let r = o.link_expired_recovery_result.unwrap();
                r.new_token_id
            }
            _ => panic!("expected OK"),
        };
        assert_ne!(new_id_1.as_str(), expired_link_id.as_str());

        // Retry: must return the same new token id (idempotent).
        let out2 = rt.run(&mut store, &recover_req).unwrap();
        let new_id_2 = match out2 {
            Ph1LinkResponse::Ok(o) => {
                let r = o.link_expired_recovery_result.unwrap();
                r.new_token_id
            }
            _ => panic!("expected OK"),
        };
        assert_eq!(new_id_1.as_str(), new_id_2.as_str());
    }

    #[test]
    fn at_link_07_forward_block_commit_records_block_attempt() {
        let mut store = store_with_inviter();
        let rt = Ph1LinkRuntime::new(Ph1LinkConfig::mvp_v1());

        let gen = Ph1LinkRequest::invite_generate_draft_v1(
            CorrelationId(7),
            TurnId(1),
            now(10),
            user(),
            InviteeType::FamilyMember,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.run(&mut store, &gen).unwrap();
        let token_id = match out {
            Ph1LinkResponse::Ok(o) => o.link_generate_result.unwrap().token_id,
            _ => panic!("expected OK"),
        };

        // Bind on first open.
        let open = Ph1LinkRequest::invite_open_activate_commit_v1(
            CorrelationId(7),
            TurnId(2),
            now(20),
            token_id.clone(),
            "device_fp_a".to_string(),
        )
        .unwrap();
        let _ = rt.run(&mut store, &open).unwrap();

        // Now attempt forward-block on mismatch device.
        let fwd = Ph1LinkRequest {
            schema_version: selene_kernel_contracts::ph1link::PH1LINK_CONTRACT_VERSION,
            correlation_id: CorrelationId(7),
            turn_id: TurnId(3),
            now: now(21),
            simulation_id: LINK_INVITE_FORWARD_BLOCK_COMMIT.to_string(),
            simulation_type: SimulationType::Commit,
            request: LinkRequest::InviteForwardBlockCommit(InviteForwardBlockCommitRequest {
                token_id,
                device_fingerprint: "device_fp_b".to_string(),
            }),
        };

        let out = rt.run(&mut store, &fwd).unwrap();
        match out {
            Ph1LinkResponse::Ok(o) => {
                let a = o.link_activation_result.unwrap();
                assert_eq!(a.activation_status, LinkStatus::Blocked);
            }
            _ => panic!("expected OK"),
        }
    }

    #[test]
    fn at_link_08_role_propose_is_idempotent() {
        let mut store = store_with_inviter();
        let rt = Ph1LinkRuntime::new(Ph1LinkConfig::mvp_v1());

        let req = Ph1LinkRequest {
            schema_version: selene_kernel_contracts::ph1link::PH1LINK_CONTRACT_VERSION,
            correlation_id: CorrelationId(8),
            turn_id: TurnId(1),
            now: now(10),
            simulation_id: LINK_ROLE_PROPOSE_DRAFT.to_string(),
            simulation_type: SimulationType::Draft,
            request: LinkRequest::RoleProposeDraft(RoleProposeDraftRequest {
                tenant_id: Some("tenant_1".to_string()),
                proposal_text: "add a new role: store_supervisor".to_string(),
            }),
        };

        let out1 = rt.run(&mut store, &req).unwrap();
        let id1 = match out1 {
            Ph1LinkResponse::Ok(o) => o.role_proposal_result.unwrap().role_proposal_id,
            _ => panic!("expected OK"),
        };

        let out2 = rt.run(&mut store, &req).unwrap();
        let id2 = match out2 {
            Ph1LinkResponse::Ok(o) => o.role_proposal_result.unwrap().role_proposal_id,
            _ => panic!("expected OK"),
        };

        assert_eq!(id1, id2);
    }

    #[test]
    fn at_link_09_dual_role_conflict_escalate_is_idempotent() {
        let mut store = store_with_inviter();
        let rt = Ph1LinkRuntime::new(Ph1LinkConfig::mvp_v1());

        let req = Ph1LinkRequest {
            schema_version: selene_kernel_contracts::ph1link::PH1LINK_CONTRACT_VERSION,
            correlation_id: CorrelationId(9),
            turn_id: TurnId(1),
            now: now(10),
            simulation_id: LINK_INVITE_DUAL_ROLE_CONFLICT_ESCALATE_DRAFT.to_string(),
            simulation_type: SimulationType::Draft,
            request: LinkRequest::DualRoleConflictEscalateDraft(
                DualRoleConflictEscalateDraftRequest {
                    tenant_id: Some("tenant_1".to_string()),
                    token_id: None,
                    note: "conflict: already an employee, cannot invite as agent".to_string(),
                },
            ),
        };

        let out1 = rt.run(&mut store, &req).unwrap();
        let id1 = match out1 {
            Ph1LinkResponse::Ok(o) => {
                o.dual_role_conflict_escalation_result
                    .unwrap()
                    .escalation_case_id
            }
            _ => panic!("expected OK"),
        };

        let out2 = rt.run(&mut store, &req).unwrap();
        let id2 = match out2 {
            Ph1LinkResponse::Ok(o) => {
                o.dual_role_conflict_escalation_result
                    .unwrap()
                    .escalation_case_id
            }
            _ => panic!("expected OK"),
        };

        assert_eq!(id1, id2);
    }
}
