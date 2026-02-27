#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use selene_kernel_contracts::ph1j::{
    AuditEngine, AuditEventInput, AuditEventType, AuditPayloadMin, AuditSeverity, PayloadKey,
    PayloadValue,
};
use selene_kernel_contracts::ph1link::{
    DualRoleConflictEscalationResult, EscalationStatus, LinkActivationResult,
    LinkDraftUpdateResult, LinkExpiredRecoveryResult, LinkGenerateResult, LinkRevokeResult,
    LinkStatus, Ph1LinkOk, Ph1LinkRefuse, Ph1LinkRequest, Ph1LinkResponse, PrefilledContextRef,
    RoleProposalResult, RoleProposalStatus, PH1LINK_CONTRACT_VERSION,
};
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs, ReasonCodeId, Validate};
use selene_storage::ph1f::{Ph1fStore, StorageError};
use selene_storage::ph1j::Ph1jRuntime;

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.LINK reason-code namespace. Values are placeholders until the global registry is formalized.
    pub const LINK_OK_GENERATE_DRAFT: ReasonCodeId = ReasonCodeId(0x4E00_0001);
    pub const LINK_OK_DRAFT_UPDATE_COMMIT: ReasonCodeId = ReasonCodeId(0x4E00_0002);
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

                let link_url = format!(
                    "{}/{}?sig={}",
                    self.config.base_url,
                    link_rec.token_id.as_str(),
                    link_rec.token_signature
                );
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

            selene_kernel_contracts::ph1link::LinkRequest::InviteDraftUpdateCommit(r) => {
                let (draft_id, draft_status, missing_required_fields) = store
                    .ph1link_invite_draft_update_commit(
                        req.now,
                        r.draft_id.clone(),
                        r.creator_update_fields.clone(),
                        r.idempotency_key.clone(),
                    )?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    None,
                    "DRAFT_CREATED_OR_DRAFT_READY",
                    match draft_status {
                        selene_kernel_contracts::ph1link::DraftStatus::DraftCreated => {
                            "DRAFT_CREATED"
                        }
                        selene_kernel_contracts::ph1link::DraftStatus::DraftReady => "DRAFT_READY",
                    },
                    reason_codes::LINK_OK_DRAFT_UPDATE_COMMIT,
                    Some(format!("link_draft_update:{}", draft_id.as_str())),
                )?;

                let out =
                    LinkDraftUpdateResult::v1(draft_id, draft_status, missing_required_fields)
                        .map_err(StorageError::ContractViolation)?;

                let ok = Ph1LinkOk {
                    schema_version: PH1LINK_CONTRACT_VERSION,
                    simulation_id: req.simulation_id.clone(),
                    reason_code: reason_codes::LINK_OK_DRAFT_UPDATE_COMMIT,
                    link_generate_result: None,
                    link_draft_update_result: Some(out),
                    link_activation_result: None,
                    link_revoke_result: None,
                    link_expired_recovery_result: None,
                    role_proposal_result: None,
                    dual_role_conflict_escalation_result: None,
                };
                ok.validate().map_err(StorageError::ContractViolation)?;
                Ok(Ph1LinkResponse::Ok(ok))
            }

            selene_kernel_contracts::ph1link::LinkRequest::InviteOpenActivateCommit(r) => {
                let (
                    status,
                    draft_id,
                    missing_required_fields,
                    bound_hash,
                    conflict_reason,
                    app_platform,
                    app_instance_id,
                    deep_link_nonce,
                    link_opened_at,
                    ctx_ref,
                ) = store.ph1link_invite_open_activate_commit_with_idempotency(
                    req.now,
                    r.token_id.clone(),
                    r.token_signature.clone(),
                    r.device_fingerprint.clone(),
                    r.app_platform,
                    r.app_instance_id.clone(),
                    r.deep_link_nonce.clone(),
                    r.link_opened_at,
                    r.idempotency_key.clone(),
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
                    app_platform,
                    app_instance_id,
                    deep_link_nonce,
                    link_opened_at,
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
                if let Err(err) =
                    store.ph1link_invite_revoke_revoke(r.token_id.clone(), r.reason.clone())
                {
                    if let StorageError::ContractViolation(ContractViolation::InvalidValue {
                        field,
                        reason,
                    }) = &err
                    {
                        if *field == "ph1link_invite_revoke_revoke.ap_override_ref" {
                            let refuse = Ph1LinkRefuse::v1(
                                req.simulation_id.clone(),
                                reason_codes::LINK_REFUSE_INVALID,
                                format!("refuse: {reason}"),
                            )
                            .map_err(StorageError::ContractViolation)?;
                            return Ok(Ph1LinkResponse::Refuse(refuse));
                        }
                    }
                    return Err(err);
                }

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

                let new_link_url = format!(
                    "{}/{}?sig={}",
                    self.config.base_url,
                    new_link.token_id.as_str(),
                    new_link.token_signature
                );
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
                    None,
                    None,
                    None,
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
    use std::collections::BTreeMap;

    use super::*;
    use selene_kernel_contracts::ph1_voice_id::{SpeakerId, UserId};
    use selene_kernel_contracts::ph1c::LanguageTag;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1link::{
        DraftStatus, DualRoleConflictEscalateDraftRequest, InviteExpiredRecoveryCommitRequest,
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
        let token_signature = store
            .ph1link_get_link(&token_id)
            .expect("link must exist after generate")
            .token_signature
            .clone();

        let open1 = Ph1LinkRequest::invite_open_activate_commit_v1(
            CorrelationId(2),
            TurnId(2),
            now(20),
            token_id.clone(),
            token_signature.clone(),
            "device_fp_a".to_string(),
            selene_kernel_contracts::ph1link::AppPlatform::Ios,
            "ios_instance_link".to_string(),
            "nonce_link_1".to_string(),
            now(20),
            "idem_link_open_1".to_string(),
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
            token_signature,
            "device_fp_b".to_string(),
            selene_kernel_contracts::ph1link::AppPlatform::Ios,
            "ios_instance_link".to_string(),
            "nonce_link_2".to_string(),
            now(21),
            "idem_link_open_2".to_string(),
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
    fn at_link_04_draft_update_commit_runtime_is_idempotent_and_advances_status() {
        let mut store = store_with_inviter();
        let rt = Ph1LinkRuntime::new(Ph1LinkConfig::mvp_v1());

        let gen = Ph1LinkRequest::invite_generate_draft_v1(
            CorrelationId(4),
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
        let draft_id = match out {
            Ph1LinkResponse::Ok(o) => o.link_generate_result.unwrap().draft_id,
            _ => panic!("expected OK"),
        };

        let mut updates = BTreeMap::new();
        updates.insert("tenant_id".to_string(), "tenant_1".to_string());

        let update_1 = Ph1LinkRequest::invite_draft_update_commit_v1(
            CorrelationId(4),
            TurnId(2),
            now(11),
            draft_id.clone(),
            updates.clone(),
            "idem_link_draft_update_1".to_string(),
        )
        .unwrap();

        let out_1 = rt.run(&mut store, &update_1).unwrap();
        match out_1 {
            Ph1LinkResponse::Ok(o) => {
                let r = o.link_draft_update_result.unwrap();
                assert_eq!(r.draft_status, DraftStatus::DraftReady);
                assert!(r.missing_required_fields.is_empty());
            }
            _ => panic!("expected OK"),
        }

        // Same draft + same idempotency key must replay the exact update result.
        let update_2 = Ph1LinkRequest::invite_draft_update_commit_v1(
            CorrelationId(4),
            TurnId(3),
            now(12),
            draft_id,
            updates,
            "idem_link_draft_update_1".to_string(),
        )
        .unwrap();

        let out_2 = rt.run(&mut store, &update_2).unwrap();
        match out_2 {
            Ph1LinkResponse::Ok(o) => {
                let r = o.link_draft_update_result.unwrap();
                assert_eq!(r.draft_status, DraftStatus::DraftReady);
                assert!(r.missing_required_fields.is_empty());
            }
            _ => panic!("expected OK"),
        }
    }

    #[test]
    fn at_link_05_revoke_returns_refuse_for_activated_without_ap_override() {
        let mut store = store_with_inviter();
        let rt = Ph1LinkRuntime::new(Ph1LinkConfig::mvp_v1());

        let gen = Ph1LinkRequest::invite_generate_draft_v1(
            CorrelationId(5),
            TurnId(1),
            now(10),
            user(),
            InviteeType::FamilyMember,
            Some("tenant_1".to_string()),
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
        let token_signature = store
            .ph1link_get_link(&token_id)
            .expect("link must exist after generate")
            .token_signature
            .clone();

        let open = Ph1LinkRequest::invite_open_activate_commit_v1(
            CorrelationId(5),
            TurnId(2),
            now(11),
            token_id.clone(),
            token_signature,
            "device_fp_a".to_string(),
            selene_kernel_contracts::ph1link::AppPlatform::Ios,
            "ios_instance_link".to_string(),
            "nonce_link_5".to_string(),
            now(11),
            "idem_link_open_bind_5".to_string(),
        )
        .unwrap();
        let out_open = rt.run(&mut store, &open).unwrap();
        match out_open {
            Ph1LinkResponse::Ok(o) => {
                let a = o.link_activation_result.unwrap();
                assert_eq!(a.activation_status, LinkStatus::Activated);
            }
            _ => panic!("expected OK"),
        }

        let revoke = Ph1LinkRequest::invite_revoke_revoke_v1(
            CorrelationId(5),
            TurnId(3),
            now(12),
            token_id.clone(),
            "admin_revoke".to_string(),
        )
        .unwrap();
        let out_revoke = rt.run(&mut store, &revoke).unwrap();
        match out_revoke {
            Ph1LinkResponse::Refuse(r) => {
                assert_eq!(r.reason_code, reason_codes::LINK_REFUSE_INVALID);
            }
            _ => panic!("expected REFUSE"),
        }

        assert_eq!(
            store.ph1link_get_link(&token_id).unwrap().status,
            LinkStatus::Activated
        );
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
        let token_signature = store
            .ph1link_get_link(&token_id)
            .expect("link must exist after generate")
            .token_signature
            .clone();

        // Bind on first open.
        let open = Ph1LinkRequest::invite_open_activate_commit_v1(
            CorrelationId(7),
            TurnId(2),
            now(20),
            token_id.clone(),
            token_signature,
            "device_fp_a".to_string(),
            selene_kernel_contracts::ph1link::AppPlatform::Ios,
            "ios_instance_link".to_string(),
            "nonce_link_7".to_string(),
            now(20),
            "idem_link_open_bind".to_string(),
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
