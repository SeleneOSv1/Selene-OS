#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use selene_kernel_contracts::ph1j::{
    AuditEngine, AuditEventInput, AuditEventType, AuditPayloadMin, AuditSeverity, CorrelationId,
    PayloadKey, PayloadValue, TurnId,
};
use selene_kernel_contracts::ph1position::{
    Ph1PositionOk, Ph1PositionRequest, Ph1PositionResponse, PositionBandPolicyCheckResult,
    PositionCreateDraftResult, PositionLifecycleResult, PositionLifecycleState, PositionRequest,
    PositionValidateAuthCompanyResult, PositionValidationStatus, POSITION_SIM_001_CREATE_DRAFT,
    POSITION_SIM_004_ACTIVATE_COMMIT, POSITION_SIM_005_RETIRE_OR_SUSPEND_COMMIT,
};
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId, Validate};
use selene_storage::ph1f::{Ph1fStore, StorageError};
use selene_storage::ph1j::Ph1jRuntime;

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.POSITION reason-code namespace. Values are placeholders until the global registry is formalized.
    pub const POSITION_OK_CREATE_DRAFT: ReasonCodeId = ReasonCodeId(0x5900_0001);
    pub const POSITION_OK_VALIDATE_AUTH_COMPANY: ReasonCodeId = ReasonCodeId(0x5900_0002);
    pub const POSITION_OK_BAND_POLICY_CHECK: ReasonCodeId = ReasonCodeId(0x5900_0003);
    pub const POSITION_OK_ACTIVATE_COMMIT: ReasonCodeId = ReasonCodeId(0x5900_0004);
    pub const POSITION_OK_RETIRE_OR_SUSPEND_COMMIT: ReasonCodeId = ReasonCodeId(0x5900_0005);
}

#[derive(Debug, Default, Clone)]
pub struct Ph1PositionRuntime;

impl Ph1PositionRuntime {
    pub fn run(
        &self,
        store: &mut Ph1fStore,
        req: &Ph1PositionRequest,
    ) -> Result<Ph1PositionResponse, StorageError> {
        req.validate().map_err(StorageError::ContractViolation)?;

        match &req.request {
            PositionRequest::CreateDraft(r) => {
                let rec = store.ph1position_create_draft(
                    req.now,
                    r.actor_user_id.clone(),
                    r.tenant_id.clone(),
                    r.company_id.clone(),
                    r.position_title.clone(),
                    r.department.clone(),
                    r.jurisdiction.clone(),
                    r.schedule_type,
                    r.permission_profile_ref.clone(),
                    r.compensation_band_ref.clone(),
                    r.idempotency_key.clone(),
                    POSITION_SIM_001_CREATE_DRAFT,
                    reason_codes::POSITION_OK_CREATE_DRAFT,
                )?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "NONE",
                    "DRAFT_CREATED",
                    reason_codes::POSITION_OK_CREATE_DRAFT,
                    Some(r.idempotency_key.clone()),
                )?;

                let out = PositionCreateDraftResult::v1(rec.position_id, rec.lifecycle_state)
                    .map_err(StorageError::ContractViolation)?;

                Ok(Ph1PositionResponse::Ok(
                    Ph1PositionOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::POSITION_OK_CREATE_DRAFT,
                        Some(out),
                        None,
                        None,
                        None,
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }
            PositionRequest::ValidateAuthCompany(r) => {
                let (status, reason_code) = store.ph1position_validate_auth_company_draft(
                    &r.tenant_id,
                    &r.company_id,
                    &r.position_id,
                    r.requested_action,
                )?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "VALIDATION_START",
                    if status == PositionValidationStatus::Ok {
                        "VALIDATION_OK"
                    } else {
                        "VALIDATION_FAIL"
                    },
                    reason_codes::POSITION_OK_VALIDATE_AUTH_COMPANY,
                    Some(r.idempotency_key.clone()),
                )?;

                let out = PositionValidateAuthCompanyResult::v1(status, reason_code)
                    .map_err(StorageError::ContractViolation)?;

                Ok(Ph1PositionResponse::Ok(
                    Ph1PositionOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::POSITION_OK_VALIDATE_AUTH_COMPANY,
                        None,
                        Some(out),
                        None,
                        None,
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }
            PositionRequest::BandPolicyCheck(r) => {
                let (policy_result, reason_code) = store.ph1position_band_policy_check_draft(
                    &r.tenant_id,
                    &r.position_id,
                    &r.compensation_band_ref,
                )?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "POLICY_CHECK_START",
                    match policy_result {
                        selene_kernel_contracts::ph1position::PositionPolicyResult::Allow => {
                            "POLICY_ALLOW"
                        }
                        selene_kernel_contracts::ph1position::PositionPolicyResult::Escalate => {
                            "POLICY_ESCALATE"
                        }
                        selene_kernel_contracts::ph1position::PositionPolicyResult::Deny => {
                            "POLICY_DENY"
                        }
                    },
                    reason_codes::POSITION_OK_BAND_POLICY_CHECK,
                    Some(r.idempotency_key.clone()),
                )?;

                let out = PositionBandPolicyCheckResult::v1(policy_result, reason_code)
                    .map_err(StorageError::ContractViolation)?;

                Ok(Ph1PositionResponse::Ok(
                    Ph1PositionOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::POSITION_OK_BAND_POLICY_CHECK,
                        None,
                        None,
                        Some(out),
                        None,
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }
            PositionRequest::ActivateCommit(r) => {
                let rec = store.ph1position_activate_commit(
                    req.now,
                    r.actor_user_id.clone(),
                    r.tenant_id.clone(),
                    r.position_id.clone(),
                    r.idempotency_key.clone(),
                    POSITION_SIM_004_ACTIVATE_COMMIT,
                    reason_codes::POSITION_OK_ACTIVATE_COMMIT,
                )?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "DRAFT_OR_SUSPENDED",
                    "ACTIVE",
                    reason_codes::POSITION_OK_ACTIVATE_COMMIT,
                    Some(r.idempotency_key.clone()),
                )?;

                let out = PositionLifecycleResult::v1(rec.position_id, rec.lifecycle_state)
                    .map_err(StorageError::ContractViolation)?;

                Ok(Ph1PositionResponse::Ok(
                    Ph1PositionOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::POSITION_OK_ACTIVATE_COMMIT,
                        None,
                        None,
                        None,
                        Some(out),
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }
            PositionRequest::RetireOrSuspendCommit(r) => {
                let rec = store.ph1position_retire_or_suspend_commit(
                    req.now,
                    r.actor_user_id.clone(),
                    r.tenant_id.clone(),
                    r.position_id.clone(),
                    r.requested_state,
                    r.idempotency_key.clone(),
                    POSITION_SIM_005_RETIRE_OR_SUSPEND_COMMIT,
                    reason_codes::POSITION_OK_RETIRE_OR_SUSPEND_COMMIT,
                )?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "ACTIVE_OR_SUSPENDED",
                    match rec.lifecycle_state {
                        PositionLifecycleState::Suspended => "SUSPENDED",
                        PositionLifecycleState::Retired => "RETIRED",
                        PositionLifecycleState::Draft => "DRAFT",
                        PositionLifecycleState::Active => "ACTIVE",
                    },
                    reason_codes::POSITION_OK_RETIRE_OR_SUSPEND_COMMIT,
                    Some(r.idempotency_key.clone()),
                )?;

                let out = PositionLifecycleResult::v1(rec.position_id, rec.lifecycle_state)
                    .map_err(StorageError::ContractViolation)?;

                Ok(Ph1PositionResponse::Ok(
                    Ph1PositionOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::POSITION_OK_RETIRE_OR_SUSPEND_COMMIT,
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
        correlation_id: CorrelationId,
        turn_id: TurnId,
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

        let engine = AuditEngine::Other("ph1_position".to_string());
        let ev = AuditEventInput::v1(
            now,
            None,
            None,
            None,
            None,
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
    use selene_kernel_contracts::ph1_voice_id::UserId;
    use selene_kernel_contracts::ph1position::{
        Ph1PositionRequest, Ph1PositionResponse, PositionActivateCommitRequest,
        PositionBandPolicyCheckRequest, PositionCreateDraftRequest,
        PositionLifecycleState as ContractPositionLifecycleState, PositionRequest,
        PositionRequestedAction, PositionRetireOrSuspendCommitRequest, PositionScheduleType,
        PositionSimulationType, PositionValidateAuthCompanyRequest, PH1POSITION_CONTRACT_VERSION,
        POSITION_SIM_002_VALIDATE_AUTH_COMPANY, POSITION_SIM_003_BAND_POLICY_CHECK,
    };
    use selene_storage::ph1f::{
        IdentityRecord, IdentityStatus, TenantCompanyLifecycleState, TenantCompanyRecord,
    };

    fn now() -> MonotonicTimeNs {
        MonotonicTimeNs(1_000_000_000)
    }

    fn user(id: &str) -> UserId {
        UserId::new(id).unwrap()
    }

    fn setup_store() -> (Ph1fStore, UserId) {
        let mut store = Ph1fStore::new_in_memory();
        let actor = user("position_actor_1");
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                now(),
                IdentityStatus::Active,
            ))
            .unwrap();
        store
            .ph1tenant_company_upsert(TenantCompanyRecord {
                schema_version: selene_kernel_contracts::SchemaVersion(1),
                tenant_id: selene_kernel_contracts::ph1position::TenantId::new("tenant_1").unwrap(),
                company_id: "company_1".to_string(),
                legal_name: "Selene Inc".to_string(),
                jurisdiction: "CN".to_string(),
                lifecycle_state: TenantCompanyLifecycleState::Active,
                created_at: now(),
                updated_at: now(),
            })
            .unwrap();
        (store, actor)
    }

    #[test]
    fn ph1position_happy_path_create_validate_activate_suspend() {
        let rt = Ph1PositionRuntime;
        let (mut store, actor) = setup_store();

        let create_req = Ph1PositionRequest {
            schema_version: PH1POSITION_CONTRACT_VERSION,
            correlation_id: CorrelationId(1),
            turn_id: TurnId(1),
            now: MonotonicTimeNs(now().0 + 1),
            simulation_id: POSITION_SIM_001_CREATE_DRAFT.to_string(),
            simulation_type: PositionSimulationType::Draft,
            request: PositionRequest::CreateDraft(PositionCreateDraftRequest {
                actor_user_id: actor.clone(),
                tenant_id: selene_kernel_contracts::ph1position::TenantId::new("tenant_1").unwrap(),
                company_id: "company_1".to_string(),
                position_title: "Store Manager".to_string(),
                department: "Retail".to_string(),
                jurisdiction: "CN".to_string(),
                schedule_type: PositionScheduleType::FullTime,
                permission_profile_ref: "profile_store_mgr".to_string(),
                compensation_band_ref: "band_l3".to_string(),
                idempotency_key: "p-create-1".to_string(),
            }),
        };
        let create_out = rt.run(&mut store, &create_req).unwrap();
        let position_id = match create_out {
            Ph1PositionResponse::Ok(ok) => ok.create_draft_result.unwrap().position_id,
            Ph1PositionResponse::Refuse(_) => panic!("expected ok"),
        };

        let validate_req = Ph1PositionRequest {
            schema_version: PH1POSITION_CONTRACT_VERSION,
            correlation_id: CorrelationId(1),
            turn_id: TurnId(2),
            now: MonotonicTimeNs(now().0 + 2),
            simulation_id: POSITION_SIM_002_VALIDATE_AUTH_COMPANY.to_string(),
            simulation_type: PositionSimulationType::Draft,
            request: PositionRequest::ValidateAuthCompany(PositionValidateAuthCompanyRequest {
                actor_user_id: actor.clone(),
                tenant_id: selene_kernel_contracts::ph1position::TenantId::new("tenant_1").unwrap(),
                company_id: "company_1".to_string(),
                position_id: position_id.clone(),
                requested_action: PositionRequestedAction::Activate,
                idempotency_key: "p-validate-1".to_string(),
            }),
        };
        let validate_out = rt.run(&mut store, &validate_req).unwrap();
        match validate_out {
            Ph1PositionResponse::Ok(ok) => assert_eq!(
                ok.validate_auth_company_result.unwrap().validation_status,
                PositionValidationStatus::Ok
            ),
            Ph1PositionResponse::Refuse(_) => panic!("expected ok"),
        }

        let policy_req = Ph1PositionRequest {
            schema_version: PH1POSITION_CONTRACT_VERSION,
            correlation_id: CorrelationId(1),
            turn_id: TurnId(3),
            now: MonotonicTimeNs(now().0 + 3),
            simulation_id: POSITION_SIM_003_BAND_POLICY_CHECK.to_string(),
            simulation_type: PositionSimulationType::Draft,
            request: PositionRequest::BandPolicyCheck(PositionBandPolicyCheckRequest {
                actor_user_id: actor.clone(),
                tenant_id: selene_kernel_contracts::ph1position::TenantId::new("tenant_1").unwrap(),
                position_id: position_id.clone(),
                compensation_band_ref: "band_l3".to_string(),
                idempotency_key: "p-policy-1".to_string(),
            }),
        };
        let policy_out = rt.run(&mut store, &policy_req).unwrap();
        match policy_out {
            Ph1PositionResponse::Ok(ok) => assert_eq!(
                ok.band_policy_check_result.unwrap().policy_result,
                selene_kernel_contracts::ph1position::PositionPolicyResult::Allow
            ),
            Ph1PositionResponse::Refuse(_) => panic!("expected ok"),
        }

        let activate_req = Ph1PositionRequest {
            schema_version: PH1POSITION_CONTRACT_VERSION,
            correlation_id: CorrelationId(1),
            turn_id: TurnId(4),
            now: MonotonicTimeNs(now().0 + 4),
            simulation_id: POSITION_SIM_004_ACTIVATE_COMMIT.to_string(),
            simulation_type: PositionSimulationType::Commit,
            request: PositionRequest::ActivateCommit(PositionActivateCommitRequest {
                actor_user_id: actor.clone(),
                tenant_id: selene_kernel_contracts::ph1position::TenantId::new("tenant_1").unwrap(),
                position_id: position_id.clone(),
                idempotency_key: "p-activate-1".to_string(),
            }),
        };
        let activate_out = rt.run(&mut store, &activate_req).unwrap();
        match activate_out {
            Ph1PositionResponse::Ok(ok) => assert_eq!(
                ok.lifecycle_result.unwrap().lifecycle_state,
                ContractPositionLifecycleState::Active
            ),
            Ph1PositionResponse::Refuse(_) => panic!("expected ok"),
        }

        let suspend_req = Ph1PositionRequest {
            schema_version: PH1POSITION_CONTRACT_VERSION,
            correlation_id: CorrelationId(1),
            turn_id: TurnId(5),
            now: MonotonicTimeNs(now().0 + 5),
            simulation_id: POSITION_SIM_005_RETIRE_OR_SUSPEND_COMMIT.to_string(),
            simulation_type: PositionSimulationType::Commit,
            request: PositionRequest::RetireOrSuspendCommit(PositionRetireOrSuspendCommitRequest {
                actor_user_id: actor,
                tenant_id: selene_kernel_contracts::ph1position::TenantId::new("tenant_1").unwrap(),
                position_id,
                requested_state: ContractPositionLifecycleState::Suspended,
                idempotency_key: "p-suspend-1".to_string(),
            }),
        };
        let suspend_out = rt.run(&mut store, &suspend_req).unwrap();
        match suspend_out {
            Ph1PositionResponse::Ok(ok) => assert_eq!(
                ok.lifecycle_result.unwrap().lifecycle_state,
                ContractPositionLifecycleState::Suspended
            ),
            Ph1PositionResponse::Refuse(_) => panic!("expected ok"),
        }
    }
}
