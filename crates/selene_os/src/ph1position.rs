#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use selene_kernel_contracts::ph1j::{
    AuditEngine, AuditEventInput, AuditEventType, AuditPayloadMin, AuditSeverity, CorrelationId,
    PayloadKey, PayloadValue, TurnId,
};
use selene_kernel_contracts::ph1position::{
    Ph1PositionOk, Ph1PositionRequest, Ph1PositionResponse, PositionBandPolicyCheckResult,
    PositionCreateDraftResult, PositionLifecycleResult, PositionLifecycleState, PositionRequest,
    PositionSchemaApplyScope,
    PositionValidateAuthCompanyResult, PositionValidationStatus, POSITION_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT,
    POSITION_REQUIREMENTS_SCHEMA_CREATE_DRAFT, POSITION_REQUIREMENTS_SCHEMA_UPDATE_COMMIT,
    POSITION_SIM_001_CREATE_DRAFT, POSITION_SIM_004_ACTIVATE_COMMIT,
    POSITION_SIM_005_RETIRE_OR_SUSPEND_COMMIT,
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
    pub const POSITION_OK_REQUIREMENTS_SCHEMA_CREATE_DRAFT: ReasonCodeId = ReasonCodeId(0x5900_0006);
    pub const POSITION_OK_REQUIREMENTS_SCHEMA_UPDATE_COMMIT: ReasonCodeId = ReasonCodeId(0x5900_0007);
    pub const POSITION_OK_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT: ReasonCodeId =
        ReasonCodeId(0x5900_0008);
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
            PositionRequest::RequirementsSchemaCreateDraft(r) => {
                let out = store.ph1position_requirements_schema_create_draft(
                    req.now,
                    r.actor_user_id.clone(),
                    r.tenant_id.clone(),
                    r.company_id.clone(),
                    r.position_id.clone(),
                    r.schema_version_id.clone(),
                    r.selectors.clone(),
                    r.field_specs.clone(),
                    r.idempotency_key.clone(),
                    POSITION_REQUIREMENTS_SCHEMA_CREATE_DRAFT,
                    reason_codes::POSITION_OK_REQUIREMENTS_SCHEMA_CREATE_DRAFT,
                )?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "REQUIREMENTS_SCHEMA_NONE",
                    "REQUIREMENTS_SCHEMA_DRAFT_CREATED",
                    reason_codes::POSITION_OK_REQUIREMENTS_SCHEMA_CREATE_DRAFT,
                    Some(r.idempotency_key.clone()),
                )?;

                let ok = Ph1PositionOk {
                    schema_version: selene_kernel_contracts::ph1position::PH1POSITION_CONTRACT_VERSION,
                    simulation_id: req.simulation_id.clone(),
                    reason_code: reason_codes::POSITION_OK_REQUIREMENTS_SCHEMA_CREATE_DRAFT,
                    create_draft_result: None,
                    validate_auth_company_result: None,
                    band_policy_check_result: None,
                    lifecycle_result: None,
                    requirements_schema_draft_result: Some(out),
                    requirements_schema_lifecycle_result: None,
                };
                ok.validate().map_err(StorageError::ContractViolation)?;
                Ok(Ph1PositionResponse::Ok(ok))
            }
            PositionRequest::RequirementsSchemaUpdateCommit(r) => {
                let out = store.ph1position_requirements_schema_update_commit(
                    req.now,
                    r.actor_user_id.clone(),
                    r.tenant_id.clone(),
                    r.company_id.clone(),
                    r.position_id.clone(),
                    r.schema_version_id.clone(),
                    r.selectors.clone(),
                    r.field_specs.clone(),
                    r.change_reason.clone(),
                    r.idempotency_key.clone(),
                    POSITION_REQUIREMENTS_SCHEMA_UPDATE_COMMIT,
                    reason_codes::POSITION_OK_REQUIREMENTS_SCHEMA_UPDATE_COMMIT,
                )?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "REQUIREMENTS_SCHEMA_DRAFT_OR_ACTIVE",
                    "REQUIREMENTS_SCHEMA_UPDATED",
                    reason_codes::POSITION_OK_REQUIREMENTS_SCHEMA_UPDATE_COMMIT,
                    Some(r.idempotency_key.clone()),
                )?;

                let ok = Ph1PositionOk {
                    schema_version: selene_kernel_contracts::ph1position::PH1POSITION_CONTRACT_VERSION,
                    simulation_id: req.simulation_id.clone(),
                    reason_code: reason_codes::POSITION_OK_REQUIREMENTS_SCHEMA_UPDATE_COMMIT,
                    create_draft_result: None,
                    validate_auth_company_result: None,
                    band_policy_check_result: None,
                    lifecycle_result: None,
                    requirements_schema_draft_result: Some(out),
                    requirements_schema_lifecycle_result: None,
                };
                ok.validate().map_err(StorageError::ContractViolation)?;
                Ok(Ph1PositionResponse::Ok(ok))
            }
            PositionRequest::RequirementsSchemaActivateCommit(r) => {
                let out = store.ph1position_requirements_schema_activate_commit(
                    req.now,
                    r.actor_user_id.clone(),
                    r.tenant_id.clone(),
                    r.company_id.clone(),
                    r.position_id.clone(),
                    r.schema_version_id.clone(),
                    r.apply_scope,
                    r.idempotency_key.clone(),
                    POSITION_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT,
                    reason_codes::POSITION_OK_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT,
                )?;

                let state_to = match out.apply_scope_result {
                    PositionSchemaApplyScope::NewHiresOnly => "REQUIREMENTS_SCHEMA_ACTIVE_NEW_HIRES_ONLY",
                    PositionSchemaApplyScope::CurrentAndNew => {
                        "REQUIREMENTS_SCHEMA_ACTIVE_CURRENT_AND_NEW"
                    }
                };
                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "REQUIREMENTS_SCHEMA_DRAFT",
                    state_to,
                    reason_codes::POSITION_OK_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT,
                    Some(r.idempotency_key.clone()),
                )?;

                let ok = Ph1PositionOk {
                    schema_version: selene_kernel_contracts::ph1position::PH1POSITION_CONTRACT_VERSION,
                    simulation_id: req.simulation_id.clone(),
                    reason_code: reason_codes::POSITION_OK_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT,
                    create_draft_result: None,
                    validate_auth_company_result: None,
                    band_policy_check_result: None,
                    lifecycle_result: None,
                    requirements_schema_draft_result: None,
                    requirements_schema_lifecycle_result: Some(out),
                };
                ok.validate().map_err(StorageError::ContractViolation)?;
                Ok(Ph1PositionResponse::Ok(ok))
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
        self.audit_transition_with_details(
            store,
            now,
            correlation_id,
            turn_id,
            state_from,
            state_to,
            &[],
            reason_code,
            idempotency_key,
        )
    }

    fn audit_transition_with_details(
        &self,
        store: &mut Ph1fStore,
        now: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        state_from: &'static str,
        state_to: &'static str,
        detail_entries: &[(&'static str, String)],
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
        for (k, v) in detail_entries {
            entries.insert(
                PayloadKey::new(*k).map_err(StorageError::ContractViolation)?,
                PayloadValue::new(v.as_str()).map_err(StorageError::ContractViolation)?,
            );
        }
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
        PositionRequestedAction, PositionRequirementEvidenceMode, PositionRequirementExposureRule,
        PositionRequirementFieldSpec, PositionRequirementFieldType, PositionRequirementRuleType,
        PositionRequirementSensitivity, PositionRequirementsSchemaActivateCommitRequest,
        PositionRequirementsSchemaCreateDraftRequest, PositionRequirementsSchemaUpdateCommitRequest,
        PositionRetireOrSuspendCommitRequest, PositionScheduleType, PositionSchemaApplyScope,
        PositionSchemaSelectorSnapshot, PositionSimulationType, PositionValidateAuthCompanyRequest,
        PH1POSITION_CONTRACT_VERSION, POSITION_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT,
        POSITION_REQUIREMENTS_SCHEMA_CREATE_DRAFT, POSITION_REQUIREMENTS_SCHEMA_UPDATE_COMMIT,
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

    fn selectors() -> PositionSchemaSelectorSnapshot {
        PositionSchemaSelectorSnapshot {
            company_size: Some("SMALL".to_string()),
            industry_code: Some("RETAIL".to_string()),
            jurisdiction: Some("CN".to_string()),
            position_family: Some("STORE".to_string()),
        }
    }

    fn required_field(field_key: &str) -> PositionRequirementFieldSpec {
        PositionRequirementFieldSpec {
            field_key: field_key.to_string(),
            field_type: PositionRequirementFieldType::String,
            required_rule: PositionRequirementRuleType::Always,
            required_predicate_ref: None,
            validation_ref: None,
            sensitivity: PositionRequirementSensitivity::Private,
            exposure_rule: PositionRequirementExposureRule::InternalOnly,
            evidence_mode: PositionRequirementEvidenceMode::Attestation,
            prompt_short: format!("Provide {field_key}"),
            prompt_long: format!("Please provide {field_key}"),
        }
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

    #[test]
    fn ph1position_requirements_schema_create_update_activate_scope_outputs() {
        let rt = Ph1PositionRuntime;
        let (mut store, actor) = setup_store();

        let create_position_req = Ph1PositionRequest {
            schema_version: PH1POSITION_CONTRACT_VERSION,
            correlation_id: CorrelationId(11),
            turn_id: TurnId(1),
            now: MonotonicTimeNs(now().0 + 11),
            simulation_id: POSITION_SIM_001_CREATE_DRAFT.to_string(),
            simulation_type: PositionSimulationType::Draft,
            request: PositionRequest::CreateDraft(PositionCreateDraftRequest {
                actor_user_id: actor.clone(),
                tenant_id: selene_kernel_contracts::ph1position::TenantId::new("tenant_1").unwrap(),
                company_id: "company_1".to_string(),
                position_title: "Warehouse Manager".to_string(),
                department: "Operations".to_string(),
                jurisdiction: "CN".to_string(),
                schedule_type: PositionScheduleType::FullTime,
                permission_profile_ref: "profile_wh_mgr".to_string(),
                compensation_band_ref: "band_l4".to_string(),
                idempotency_key: "rs-create-position".to_string(),
            }),
        };
        let position_id = match rt.run(&mut store, &create_position_req).unwrap() {
            Ph1PositionResponse::Ok(ok) => ok.create_draft_result.unwrap().position_id,
            Ph1PositionResponse::Refuse(_) => panic!("expected ok"),
        };

        let activate_position_req = Ph1PositionRequest {
            schema_version: PH1POSITION_CONTRACT_VERSION,
            correlation_id: CorrelationId(11),
            turn_id: TurnId(2),
            now: MonotonicTimeNs(now().0 + 12),
            simulation_id: POSITION_SIM_004_ACTIVATE_COMMIT.to_string(),
            simulation_type: PositionSimulationType::Commit,
            request: PositionRequest::ActivateCommit(PositionActivateCommitRequest {
                actor_user_id: actor.clone(),
                tenant_id: selene_kernel_contracts::ph1position::TenantId::new("tenant_1").unwrap(),
                position_id: position_id.clone(),
                idempotency_key: "rs-activate-position".to_string(),
            }),
        };
        assert!(matches!(
            rt.run(&mut store, &activate_position_req).unwrap(),
            Ph1PositionResponse::Ok(_)
        ));

        let create_schema_req = Ph1PositionRequest {
            schema_version: PH1POSITION_CONTRACT_VERSION,
            correlation_id: CorrelationId(11),
            turn_id: TurnId(3),
            now: MonotonicTimeNs(now().0 + 13),
            simulation_id: POSITION_REQUIREMENTS_SCHEMA_CREATE_DRAFT.to_string(),
            simulation_type: PositionSimulationType::Draft,
            request: PositionRequest::RequirementsSchemaCreateDraft(
                PositionRequirementsSchemaCreateDraftRequest {
                    actor_user_id: actor.clone(),
                    tenant_id: selene_kernel_contracts::ph1position::TenantId::new("tenant_1")
                        .unwrap(),
                    company_id: "company_1".to_string(),
                    position_id: position_id.clone(),
                    schema_version_id: "schema_v1".to_string(),
                    selectors: selectors(),
                    field_specs: vec![required_field("sender_verification")],
                    idempotency_key: "rs-schema-create".to_string(),
                },
            ),
        };
        let create_schema_out = rt.run(&mut store, &create_schema_req).unwrap();
        match create_schema_out {
            Ph1PositionResponse::Ok(ok) => {
                let result = ok.requirements_schema_draft_result.unwrap();
                assert_eq!(result.schema_version_id, "schema_v1");
                assert_eq!(result.field_count, 1);
            }
            Ph1PositionResponse::Refuse(_) => panic!("expected ok"),
        }

        let update_schema_req = Ph1PositionRequest {
            schema_version: PH1POSITION_CONTRACT_VERSION,
            correlation_id: CorrelationId(11),
            turn_id: TurnId(4),
            now: MonotonicTimeNs(now().0 + 14),
            simulation_id: POSITION_REQUIREMENTS_SCHEMA_UPDATE_COMMIT.to_string(),
            simulation_type: PositionSimulationType::Commit,
            request: PositionRequest::RequirementsSchemaUpdateCommit(
                PositionRequirementsSchemaUpdateCommitRequest {
                    actor_user_id: actor.clone(),
                    tenant_id: selene_kernel_contracts::ph1position::TenantId::new("tenant_1")
                        .unwrap(),
                    company_id: "company_1".to_string(),
                    position_id: position_id.clone(),
                    schema_version_id: "schema_v1".to_string(),
                    selectors: selectors(),
                    field_specs: vec![
                        required_field("sender_verification"),
                        required_field("employee_photo"),
                    ],
                    change_reason: "Expand evidence requirements".to_string(),
                    idempotency_key: "rs-schema-update".to_string(),
                },
            ),
        };
        let update_schema_out = rt.run(&mut store, &update_schema_req).unwrap();
        match update_schema_out {
            Ph1PositionResponse::Ok(ok) => {
                let result = ok.requirements_schema_draft_result.unwrap();
                assert_eq!(result.schema_version_id, "schema_v1");
                assert_eq!(result.field_count, 2);
            }
            Ph1PositionResponse::Refuse(_) => panic!("expected ok"),
        }

        let activate_schema_req = Ph1PositionRequest {
            schema_version: PH1POSITION_CONTRACT_VERSION,
            correlation_id: CorrelationId(11),
            turn_id: TurnId(5),
            now: MonotonicTimeNs(now().0 + 15),
            simulation_id: POSITION_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT.to_string(),
            simulation_type: PositionSimulationType::Commit,
            request: PositionRequest::RequirementsSchemaActivateCommit(
                PositionRequirementsSchemaActivateCommitRequest {
                    actor_user_id: actor.clone(),
                    tenant_id: selene_kernel_contracts::ph1position::TenantId::new("tenant_1")
                        .unwrap(),
                    company_id: "company_1".to_string(),
                    position_id: position_id.clone(),
                    schema_version_id: "schema_v1".to_string(),
                    apply_scope: PositionSchemaApplyScope::CurrentAndNew,
                    idempotency_key: "rs-schema-activate".to_string(),
                },
            ),
        };
        let activate_schema_out = rt.run(&mut store, &activate_schema_req).unwrap();
        match activate_schema_out {
            Ph1PositionResponse::Ok(ok) => {
                let result = ok.requirements_schema_lifecycle_result.unwrap();
                assert_eq!(
                    result.apply_scope_result,
                    PositionSchemaApplyScope::CurrentAndNew
                );
                assert!(result.backfill_handoff_required);
            }
            Ph1PositionResponse::Refuse(_) => panic!("expected ok"),
        }

        let replay_activate_req = Ph1PositionRequest {
            schema_version: PH1POSITION_CONTRACT_VERSION,
            correlation_id: CorrelationId(11),
            turn_id: TurnId(6),
            now: MonotonicTimeNs(now().0 + 16),
            simulation_id: POSITION_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT.to_string(),
            simulation_type: PositionSimulationType::Commit,
            request: PositionRequest::RequirementsSchemaActivateCommit(
                PositionRequirementsSchemaActivateCommitRequest {
                    actor_user_id: actor,
                    tenant_id: selene_kernel_contracts::ph1position::TenantId::new("tenant_1")
                        .unwrap(),
                    company_id: "company_1".to_string(),
                    position_id,
                    schema_version_id: "schema_v1".to_string(),
                    apply_scope: PositionSchemaApplyScope::NewHiresOnly,
                    idempotency_key: "rs-schema-activate".to_string(),
                },
            ),
        };
        let replay_activate_out = rt.run(&mut store, &replay_activate_req).unwrap();
        match replay_activate_out {
            Ph1PositionResponse::Ok(ok) => {
                let result = ok.requirements_schema_lifecycle_result.unwrap();
                assert_eq!(
                    result.apply_scope_result,
                    PositionSchemaApplyScope::CurrentAndNew
                );
                assert!(result.backfill_handoff_required);
            }
            Ph1PositionResponse::Refuse(_) => panic!("expected ok"),
        }
    }
}
