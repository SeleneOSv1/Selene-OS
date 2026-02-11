#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use selene_kernel_contracts::ph1_voice_id::{
    Ph1VoiceIdSimOk, Ph1VoiceIdSimRequest, Ph1VoiceIdSimResponse,
    VoiceEnrollStatus as ContractVoiceEnrollStatus, VoiceEnrollmentSessionId,
    VoiceIdEnrollCompleteResult, VoiceIdEnrollDeferResult, VoiceIdEnrollSampleResult,
    VoiceIdEnrollStartResult, VoiceIdSimulationRequest,
    VoiceSampleResult as ContractVoiceSampleResult,
};
use selene_kernel_contracts::ph1j::{
    AuditEngine, AuditEventInput, AuditEventType, AuditPayloadMin, AuditSeverity, CorrelationId,
    PayloadKey, PayloadValue, TurnId,
};
use selene_kernel_contracts::ph1onb::OnboardingSessionId;
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs, ReasonCodeId, Validate};
use selene_storage::ph1f::{
    Ph1fStore, StorageError, VoiceEnrollStatus as StoreVoiceEnrollStatus,
    VoiceSampleResult as StoreVoiceSampleResult,
};
use selene_storage::ph1j::Ph1jRuntime;

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.VOICE.ID enrollment reason-code namespace. Values are placeholders until global registry is formalized.
    pub const VID_OK_ENROLL_START_DRAFT: ReasonCodeId = ReasonCodeId(0x5649_1001);
    pub const VID_OK_ENROLL_SAMPLE_COMMIT: ReasonCodeId = ReasonCodeId(0x5649_1002);
    pub const VID_OK_ENROLL_COMPLETE_COMMIT: ReasonCodeId = ReasonCodeId(0x5649_1003);
    pub const VID_OK_ENROLL_DEFER_REMINDER_COMMIT: ReasonCodeId = ReasonCodeId(0x5649_1004);
}

#[derive(Debug, Default, Clone)]
pub struct Ph1VoiceIdRuntime;

impl Ph1VoiceIdRuntime {
    pub fn run(
        &self,
        store: &mut Ph1fStore,
        req: &Ph1VoiceIdSimRequest,
    ) -> Result<Ph1VoiceIdSimResponse, StorageError> {
        req.validate().map_err(StorageError::ContractViolation)?;

        match &req.request {
            VoiceIdSimulationRequest::EnrollStartDraft(r) => {
                let onboarding_session_id =
                    OnboardingSessionId::new(r.onboarding_session_id.clone())
                        .map_err(StorageError::ContractViolation)?;

                let rec = store.ph1vid_enroll_start_draft(
                    req.now,
                    onboarding_session_id,
                    r.device_id.clone(),
                    r.consent_asserted,
                    r.max_total_attempts,
                    r.max_session_enroll_time_ms,
                    r.lock_after_consecutive_passes,
                )?;

                let out = VoiceIdEnrollStartResult::v1(
                    VoiceEnrollmentSessionId::new(rec.voice_enrollment_session_id.clone())
                        .map_err(StorageError::ContractViolation)?,
                    map_status(rec.voice_enroll_status),
                    rec.max_total_attempts,
                    rec.max_session_enroll_time_ms,
                    rec.lock_after_consecutive_passes,
                )
                .map_err(StorageError::ContractViolation)?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "NONE",
                    status_label(rec.voice_enroll_status),
                    reason_codes::VID_OK_ENROLL_START_DRAFT,
                    Some(format!(
                        "vid_enroll_start:{}",
                        rec.voice_enrollment_session_id
                    )),
                )?;

                Ok(Ph1VoiceIdSimResponse::Ok(
                    Ph1VoiceIdSimOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::VID_OK_ENROLL_START_DRAFT,
                        Some(out),
                        None,
                        None,
                        None,
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }
            VoiceIdSimulationRequest::EnrollSampleCommit(r) => {
                let rec = store.ph1vid_enroll_sample_commit(
                    req.now,
                    r.voice_enrollment_session_id.as_str().to_string(),
                    r.audio_sample_ref.clone(),
                    r.attempt_index,
                    map_sample_result(r.sample_result),
                    r.reason_code,
                    r.idempotency_key.clone(),
                )?;

                let out = VoiceIdEnrollSampleResult::v1(
                    VoiceEnrollmentSessionId::new(rec.voice_enrollment_session_id.clone())
                        .map_err(StorageError::ContractViolation)?,
                    r.sample_result,
                    rec.reason_code.or(r.reason_code),
                    rec.consecutive_passes,
                    map_status(rec.voice_enroll_status),
                )
                .map_err(StorageError::ContractViolation)?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "VOICE_ENROLL_SAMPLE",
                    status_label(rec.voice_enroll_status),
                    reason_codes::VID_OK_ENROLL_SAMPLE_COMMIT,
                    Some(r.idempotency_key.clone()),
                )?;

                Ok(Ph1VoiceIdSimResponse::Ok(
                    Ph1VoiceIdSimOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::VID_OK_ENROLL_SAMPLE_COMMIT,
                        None,
                        Some(out),
                        None,
                        None,
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }
            VoiceIdSimulationRequest::EnrollCompleteCommit(r) => {
                let rec = store.ph1vid_enroll_complete_commit(
                    req.now,
                    r.voice_enrollment_session_id.as_str().to_string(),
                    r.idempotency_key.clone(),
                )?;

                let voice_profile_id =
                    rec.voice_profile_id
                        .clone()
                        .ok_or(StorageError::ContractViolation(
                            ContractViolation::InvalidValue {
                                field: "ph1vid_runtime.voice_profile_id",
                                reason: "must be present after complete commit",
                            },
                        ))?;

                let out = VoiceIdEnrollCompleteResult::v1(
                    VoiceEnrollmentSessionId::new(rec.voice_enrollment_session_id.clone())
                        .map_err(StorageError::ContractViolation)?,
                    voice_profile_id,
                    map_status(rec.voice_enroll_status),
                )
                .map_err(StorageError::ContractViolation)?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "VOICE_ENROLL_LOCKED",
                    status_label(rec.voice_enroll_status),
                    reason_codes::VID_OK_ENROLL_COMPLETE_COMMIT,
                    Some(r.idempotency_key.clone()),
                )?;

                Ok(Ph1VoiceIdSimResponse::Ok(
                    Ph1VoiceIdSimOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::VID_OK_ENROLL_COMPLETE_COMMIT,
                        None,
                        None,
                        Some(out),
                        None,
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }
            VoiceIdSimulationRequest::EnrollDeferReminderCommit(r) => {
                let rec = store.ph1vid_enroll_defer_reminder_commit(
                    req.now,
                    r.voice_enrollment_session_id.as_str().to_string(),
                    r.reason_code,
                    r.idempotency_key.clone(),
                )?;

                let out = VoiceIdEnrollDeferResult::v1(
                    VoiceEnrollmentSessionId::new(rec.voice_enrollment_session_id.clone())
                        .map_err(StorageError::ContractViolation)?,
                    map_status(rec.voice_enroll_status),
                    rec.reason_code.unwrap_or(r.reason_code),
                )
                .map_err(StorageError::ContractViolation)?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "VOICE_ENROLL_IN_PROGRESS",
                    status_label(rec.voice_enroll_status),
                    reason_codes::VID_OK_ENROLL_DEFER_REMINDER_COMMIT,
                    Some(r.idempotency_key.clone()),
                )?;

                Ok(Ph1VoiceIdSimResponse::Ok(
                    Ph1VoiceIdSimOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::VID_OK_ENROLL_DEFER_REMINDER_COMMIT,
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

        let ev = AuditEventInput::v1(
            now,
            None,
            None,
            None,
            None,
            None,
            AuditEngine::Other("ph1_voice_id".to_string()),
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

fn map_status(v: StoreVoiceEnrollStatus) -> ContractVoiceEnrollStatus {
    match v {
        StoreVoiceEnrollStatus::InProgress => ContractVoiceEnrollStatus::InProgress,
        StoreVoiceEnrollStatus::Locked => ContractVoiceEnrollStatus::Locked,
        StoreVoiceEnrollStatus::Pending => ContractVoiceEnrollStatus::Pending,
        StoreVoiceEnrollStatus::Declined => ContractVoiceEnrollStatus::Declined,
    }
}

fn status_label(v: StoreVoiceEnrollStatus) -> &'static str {
    match v {
        StoreVoiceEnrollStatus::InProgress => "VOICE_ENROLL_IN_PROGRESS",
        StoreVoiceEnrollStatus::Locked => "VOICE_ENROLL_LOCKED",
        StoreVoiceEnrollStatus::Pending => "VOICE_ENROLL_PENDING",
        StoreVoiceEnrollStatus::Declined => "VOICE_ENROLL_DECLINED",
    }
}

fn map_sample_result(v: ContractVoiceSampleResult) -> StoreVoiceSampleResult {
    match v {
        ContractVoiceSampleResult::Pass => StoreVoiceSampleResult::Pass,
        ContractVoiceSampleResult::Fail => StoreVoiceSampleResult::Fail,
    }
}
