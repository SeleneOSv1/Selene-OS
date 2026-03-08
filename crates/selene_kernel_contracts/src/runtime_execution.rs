#![forbid(unsafe_code)]

use crate::ph1_voice_id::UserId;
use crate::ph1j::{DeviceId, TurnId};
use crate::ph1l::SessionId;
use crate::ph1link::AppPlatform;
use crate::{ContractViolation, Validate};

fn validate_ascii_token(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if value.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if value.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    if !value.is_ascii() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be ASCII",
        });
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AdmissionState {
    IngressValidated,
    SessionResolved,
    ExecutionAdmitted,
    Rejected,
}

impl AdmissionState {
    pub const fn as_str(self) -> &'static str {
        match self {
            AdmissionState::IngressValidated => "INGRESS_VALIDATED",
            AdmissionState::SessionResolved => "SESSION_RESOLVED",
            AdmissionState::ExecutionAdmitted => "EXECUTION_ADMITTED",
            AdmissionState::Rejected => "REJECTED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FailureClass {
    AuthenticationFailure,
    AuthorizationFailure,
    InvalidPayload,
    ReplayRejected,
    SessionConflict,
    PolicyViolation,
    ExecutionFailure,
    RetryableRuntime,
}

impl FailureClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            FailureClass::AuthenticationFailure => "AUTHENTICATION_FAILURE",
            FailureClass::AuthorizationFailure => "AUTHORIZATION_FAILURE",
            FailureClass::InvalidPayload => "INVALID_PAYLOAD",
            FailureClass::ReplayRejected => "REPLAY_REJECTED",
            FailureClass::SessionConflict => "SESSION_CONFLICT",
            FailureClass::PolicyViolation => "POLICY_VIOLATION",
            FailureClass::ExecutionFailure => "EXECUTION_FAILURE",
            FailureClass::RetryableRuntime => "RETRYABLE_RUNTIME",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SessionAttachOutcome {
    NewSessionCreated,
    ExistingSessionReused,
    ExistingSessionAttached,
    RetryReusedResult,
}

impl SessionAttachOutcome {
    pub const fn as_str(self) -> &'static str {
        match self {
            SessionAttachOutcome::NewSessionCreated => "NEW_SESSION_CREATED",
            SessionAttachOutcome::ExistingSessionReused => "EXISTING_SESSION_REUSED",
            SessionAttachOutcome::ExistingSessionAttached => "EXISTING_SESSION_ATTACHED",
            SessionAttachOutcome::RetryReusedResult => "RETRY_REUSED_RESULT",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeExecutionEnvelope {
    pub request_id: String,
    pub trace_id: String,
    pub idempotency_key: String,
    pub actor_identity: UserId,
    pub device_identity: DeviceId,
    pub platform: AppPlatform,
    pub session_id: Option<SessionId>,
    pub turn_id: TurnId,
    pub device_turn_sequence: Option<u64>,
    pub admission_state: AdmissionState,
    pub session_attach_outcome: Option<SessionAttachOutcome>,
}

impl RuntimeExecutionEnvelope {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        request_id: String,
        trace_id: String,
        idempotency_key: String,
        actor_identity: UserId,
        device_identity: DeviceId,
        platform: AppPlatform,
        session_id: Option<SessionId>,
        turn_id: TurnId,
        admission_state: AdmissionState,
    ) -> Result<Self, ContractViolation> {
        Self::v1_with_device_turn_sequence_and_attach_outcome(
            request_id,
            trace_id,
            idempotency_key,
            actor_identity,
            device_identity,
            platform,
            session_id,
            turn_id,
            None,
            admission_state,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn v1_with_device_turn_sequence(
        request_id: String,
        trace_id: String,
        idempotency_key: String,
        actor_identity: UserId,
        device_identity: DeviceId,
        platform: AppPlatform,
        session_id: Option<SessionId>,
        turn_id: TurnId,
        device_turn_sequence: Option<u64>,
        admission_state: AdmissionState,
    ) -> Result<Self, ContractViolation> {
        Self::v1_with_device_turn_sequence_and_attach_outcome(
            request_id,
            trace_id,
            idempotency_key,
            actor_identity,
            device_identity,
            platform,
            session_id,
            turn_id,
            device_turn_sequence,
            admission_state,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn v1_with_device_turn_sequence_and_attach_outcome(
        request_id: String,
        trace_id: String,
        idempotency_key: String,
        actor_identity: UserId,
        device_identity: DeviceId,
        platform: AppPlatform,
        session_id: Option<SessionId>,
        turn_id: TurnId,
        device_turn_sequence: Option<u64>,
        admission_state: AdmissionState,
        session_attach_outcome: Option<SessionAttachOutcome>,
    ) -> Result<Self, ContractViolation> {
        let envelope = Self {
            request_id,
            trace_id,
            idempotency_key,
            actor_identity,
            device_identity,
            platform,
            session_id,
            turn_id,
            device_turn_sequence,
            admission_state,
            session_attach_outcome,
        };
        envelope.validate()?;
        Ok(envelope)
    }

    pub fn with_admission_state(
        &self,
        admission_state: AdmissionState,
    ) -> Result<Self, ContractViolation> {
        Self::v1_with_device_turn_sequence_and_attach_outcome(
            self.request_id.clone(),
            self.trace_id.clone(),
            self.idempotency_key.clone(),
            self.actor_identity.clone(),
            self.device_identity.clone(),
            self.platform,
            self.session_id,
            self.turn_id,
            self.device_turn_sequence,
            admission_state,
            self.session_attach_outcome,
        )
    }

    pub fn with_session_and_admission_state(
        &self,
        session_id: Option<SessionId>,
        admission_state: AdmissionState,
    ) -> Result<Self, ContractViolation> {
        Self::v1_with_device_turn_sequence_and_attach_outcome(
            self.request_id.clone(),
            self.trace_id.clone(),
            self.idempotency_key.clone(),
            self.actor_identity.clone(),
            self.device_identity.clone(),
            self.platform,
            session_id,
            self.turn_id,
            self.device_turn_sequence,
            admission_state,
            self.session_attach_outcome,
        )
    }

    pub fn with_session_device_turn_and_attach_outcome(
        &self,
        session_id: Option<SessionId>,
        admission_state: AdmissionState,
        device_turn_sequence: Option<u64>,
        session_attach_outcome: Option<SessionAttachOutcome>,
    ) -> Result<Self, ContractViolation> {
        Self::v1_with_device_turn_sequence_and_attach_outcome(
            self.request_id.clone(),
            self.trace_id.clone(),
            self.idempotency_key.clone(),
            self.actor_identity.clone(),
            self.device_identity.clone(),
            self.platform,
            session_id,
            self.turn_id,
            device_turn_sequence,
            admission_state,
            session_attach_outcome,
        )
    }

    pub fn with_attach_outcome(
        &self,
        session_attach_outcome: Option<SessionAttachOutcome>,
    ) -> Result<Self, ContractViolation> {
        Self::v1_with_device_turn_sequence_and_attach_outcome(
            self.request_id.clone(),
            self.trace_id.clone(),
            self.idempotency_key.clone(),
            self.actor_identity.clone(),
            self.device_identity.clone(),
            self.platform,
            self.session_id,
            self.turn_id,
            self.device_turn_sequence,
            self.admission_state,
            session_attach_outcome,
        )
    }
}

impl Validate for RuntimeExecutionEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_ascii_token(
            "runtime_execution_envelope.request_id",
            &self.request_id,
            256,
        )?;
        validate_ascii_token("runtime_execution_envelope.trace_id", &self.trace_id, 256)?;
        validate_ascii_token(
            "runtime_execution_envelope.idempotency_key",
            &self.idempotency_key,
            256,
        )?;
        if self.actor_identity.as_str().trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "runtime_execution_envelope.actor_identity",
                reason: "must not be empty",
            });
        }
        if self.actor_identity.as_str().len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "runtime_execution_envelope.actor_identity",
                reason: "must be <= 128 chars",
            });
        }
        self.device_identity.validate()?;
        self.platform.validate()?;
        self.turn_id.validate()?;
        if matches!(self.session_id, Some(SessionId(0))) {
            return Err(ContractViolation::InvalidValue {
                field: "runtime_execution_envelope.session_id",
                reason: "must be > 0 when provided",
            });
        }
        if matches!(self.device_turn_sequence, Some(0)) {
            return Err(ContractViolation::InvalidValue {
                field: "runtime_execution_envelope.device_turn_sequence",
                reason: "must be > 0 when provided",
            });
        }
        Ok(())
    }
}
