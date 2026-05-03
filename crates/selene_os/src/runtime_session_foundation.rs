#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use selene_kernel_contracts::ph1j::{DeviceId, TurnId};
use selene_kernel_contracts::ph1l::SessionId;
use selene_kernel_contracts::runtime_execution::{
    FailureClass, RuntimeExecutionEnvelope, SessionAttachOutcome,
};
use selene_kernel_contracts::{ContractViolation, SessionState, Validate};

use crate::runtime_bootstrap::{
    RuntimeBootstrapError, RuntimeClock, RuntimeSecretsProvider, RuntimeServiceContainer,
};

mod reason_codes {
    pub const SESSION_NOT_FOUND: &str = "runtime_session_not_found";
    pub const INVALID_TRANSITION: &str = "runtime_session_invalid_transition";
    pub const SINGLE_WRITER_CONFLICT: &str = "runtime_session_single_writer_conflict";
    pub const INVALID_DEVICE_SEQUENCE: &str = "runtime_session_invalid_device_sequence";
    pub const STALE_DEVICE_TURN: &str = "runtime_session_stale_device_turn";
    pub const STALE_ATTACH: &str = "runtime_session_stale_attach";
    pub const DEVICE_NOT_ATTACHED: &str = "runtime_session_device_not_attached";
    pub const ATTACH_NOT_ALLOWED: &str = "runtime_session_attach_not_allowed";
    pub const RESUME_NOT_RECOVERABLE: &str = "runtime_session_resume_not_recoverable";
    pub const CONFLICTING_RESUME: &str = "runtime_session_conflicting_resume";
    pub const RECOVER_NOT_SUSPENDED: &str = "runtime_session_recover_not_suspended";
    pub const DETACH_NOT_ATTACHED: &str = "runtime_session_detach_not_attached";
    pub const CLOSE_REQUIRES_SOFT_CLOSED: &str = "runtime_session_close_requires_soft_closed";
    pub const CLOSE_REQUIRES_NO_ATTACHMENTS: &str = "runtime_session_close_requires_no_attachments";
    pub const TOO_MANY_DEVICES: &str = "runtime_session_too_many_attached_devices";
    pub const DUPLICATE_DEVICE_CLAIM: &str = "runtime_session_duplicate_device_claim";
    pub const INVALID_ACCESS_CLASS: &str = "runtime_session_invalid_access_class";
    pub const LEASE_EXPIRED: &str = "runtime_session_lease_expired";
    pub const NOT_SESSION_OWNER: &str = "runtime_session_not_owner";
    pub const OWNERSHIP_UNCERTAIN: &str = "runtime_session_ownership_uncertain";
    pub const OWNERSHIP_TRANSFER_PENDING: &str = "runtime_session_transfer_pending";
    pub const OWNERSHIP_TRANSFER_NOT_PENDING: &str = "runtime_session_transfer_not_pending";
    pub const OWNERSHIP_TRANSFER_DRAIN_REQUIRED: &str = "runtime_session_transfer_drain_required";
    pub const OWNERSHIP_TRANSFER_TARGET_MISMATCH: &str = "runtime_session_transfer_target_mismatch";
    pub const OWNERSHIP_TRANSFER_INVALID_TARGET: &str = "runtime_session_transfer_invalid_target";
    pub const BACKPRESSURE_EXCEEDED: &str = "runtime_session_backpressure_exceeded";
    pub const INTEGRITY_VIOLATION: &str = "runtime_session_integrity_violation";
    pub const STAGE5_CURRENT_COMMITTED_TURN: &str = "stage5_current_committed_turn";
    pub const STAGE5_RETRY_REUSED_RESULT: &str = "stage5_retry_reused_result";
    pub const STAGE5_DEFERRED_TURN: &str = "stage5_deferred_turn";
    pub const STAGE5_STALE_TURN_QUARANTINED: &str = "stage5_stale_turn_quarantined";
    pub const STAGE5_SUPERSEDED_TURN_QUARANTINED: &str = "stage5_superseded_turn_quarantined";
    pub const STAGE5_CANCELLED_TURN_QUARANTINED: &str = "stage5_cancelled_turn_quarantined";
    pub const STAGE5_ABANDONED_TURN_QUARANTINED: &str = "stage5_abandoned_turn_quarantined";
    pub const STAGE5_CLOSED_SESSION_REJECTED: &str = "stage5_closed_session_rejected";
    pub const STAGE5_RECORD_ARTIFACT_ONLY: &str = "stage5_record_artifact_only";
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionFoundationErrorKind {
    SessionNotFound,
    InvalidTransition,
    SingleWriterConflict,
    InvalidDeviceSequence,
    StaleDeviceTurn,
    StaleAttach,
    DeviceNotAttached,
    AttachNotAllowed,
    ResumeNotRecoverable,
    ConflictingResume,
    RecoverNotSuspended,
    DetachNotAttached,
    CloseRequiresSoftClosed,
    CloseRequiresNoAttachments,
    TooManyDevices,
    DuplicateDeviceClaim,
    InvalidAccessClass,
    LeaseExpired,
    NotSessionOwner,
    OwnershipUncertain,
    OwnershipTransferPending,
    OwnershipTransferNotPending,
    OwnershipTransferDrainRequired,
    OwnershipTransferTargetMismatch,
    OwnershipTransferInvalidTarget,
    BackpressureExceeded,
    IntegrityViolation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionFoundationError {
    pub kind: SessionFoundationErrorKind,
    pub failure_class: FailureClass,
    pub reason_code: &'static str,
    pub message: String,
}

impl SessionFoundationError {
    fn session_not_found(session_id: SessionId) -> Self {
        Self {
            kind: SessionFoundationErrorKind::SessionNotFound,
            failure_class: FailureClass::ExecutionFailure,
            reason_code: reason_codes::SESSION_NOT_FOUND,
            message: format!("session {} was not found", session_id.0),
        }
    }

    fn invalid_transition(from: SessionState, to: SessionState) -> Self {
        Self {
            kind: SessionFoundationErrorKind::InvalidTransition,
            failure_class: FailureClass::SessionConflict,
            reason_code: reason_codes::INVALID_TRANSITION,
            message: format!("session transition {from:?} -> {to:?} is not allowed"),
        }
    }

    fn single_writer_conflict(session_id: SessionId) -> Self {
        Self {
            kind: SessionFoundationErrorKind::SingleWriterConflict,
            failure_class: FailureClass::SessionConflict,
            reason_code: reason_codes::SINGLE_WRITER_CONFLICT,
            message: format!(
                "session {} already has an in-flight writer and cannot accept another mutation",
                session_id.0
            ),
        }
    }

    fn invalid_device_sequence(device_id: &str, sequence: u64) -> Self {
        Self {
            kind: SessionFoundationErrorKind::InvalidDeviceSequence,
            failure_class: FailureClass::InvalidPayload,
            reason_code: reason_codes::INVALID_DEVICE_SEQUENCE,
            message: format!(
                "device {} supplied invalid device_turn_sequence {}",
                device_id, sequence
            ),
        }
    }

    fn stale_device_turn(device_id: &str, sequence: u64, highest_seen_sequence: u64) -> Self {
        Self {
            kind: SessionFoundationErrorKind::StaleDeviceTurn,
            failure_class: FailureClass::SessionConflict,
            reason_code: reason_codes::STALE_DEVICE_TURN,
            message: format!(
                "device {} supplied stale sequence {} below highest seen {}",
                device_id, sequence, highest_seen_sequence
            ),
        }
    }

    fn stale_attach(
        session_id: SessionId,
        device_id: &str,
        sequence: u64,
        highest_seen: u64,
    ) -> Self {
        Self {
            kind: SessionFoundationErrorKind::StaleAttach,
            failure_class: FailureClass::SessionConflict,
            reason_code: reason_codes::STALE_ATTACH,
            message: format!(
                "device {} cannot attach to session {} with stale claim {} below highest seen {}",
                device_id, session_id.0, sequence, highest_seen
            ),
        }
    }

    fn device_not_attached(session_id: SessionId, device_id: &str) -> Self {
        Self {
            kind: SessionFoundationErrorKind::DeviceNotAttached,
            failure_class: FailureClass::SessionConflict,
            reason_code: reason_codes::DEVICE_NOT_ATTACHED,
            message: format!(
                "device {} is not attached to session {}",
                device_id, session_id.0
            ),
        }
    }

    fn attach_not_allowed(session_id: SessionId, state: SessionState) -> Self {
        Self {
            kind: SessionFoundationErrorKind::AttachNotAllowed,
            failure_class: FailureClass::SessionConflict,
            reason_code: reason_codes::ATTACH_NOT_ALLOWED,
            message: format!(
                "session {} cannot accept attachment while in state {:?}",
                session_id.0, state
            ),
        }
    }

    fn resume_not_recoverable(session_id: SessionId, state: SessionState) -> Self {
        Self {
            kind: SessionFoundationErrorKind::ResumeNotRecoverable,
            failure_class: FailureClass::SessionConflict,
            reason_code: reason_codes::RESUME_NOT_RECOVERABLE,
            message: format!(
                "session {} in state {:?} is not resumable; recover or create a new session",
                session_id.0, state
            ),
        }
    }

    fn conflicting_resume(
        session_id: SessionId,
        requesting_device_id: &str,
        active_device_id: &str,
    ) -> Self {
        Self {
            kind: SessionFoundationErrorKind::ConflictingResume,
            failure_class: FailureClass::SessionConflict,
            reason_code: reason_codes::CONFLICTING_RESUME,
            message: format!(
                "device {} cannot resume session {} because {} already holds PRIMARY_INTERACTOR",
                requesting_device_id, session_id.0, active_device_id
            ),
        }
    }

    fn recover_not_suspended(session_id: SessionId, state: SessionState) -> Self {
        Self {
            kind: SessionFoundationErrorKind::RecoverNotSuspended,
            failure_class: FailureClass::SessionConflict,
            reason_code: reason_codes::RECOVER_NOT_SUSPENDED,
            message: format!(
                "session {} in state {:?} cannot be recovered because it is not suspended",
                session_id.0, state
            ),
        }
    }

    fn detach_not_attached(session_id: SessionId, device_id: &str) -> Self {
        Self {
            kind: SessionFoundationErrorKind::DetachNotAttached,
            failure_class: FailureClass::SessionConflict,
            reason_code: reason_codes::DETACH_NOT_ATTACHED,
            message: format!(
                "device {} cannot detach from session {} because it is not attached",
                device_id, session_id.0
            ),
        }
    }

    fn close_requires_soft_closed(session_id: SessionId, state: SessionState) -> Self {
        Self {
            kind: SessionFoundationErrorKind::CloseRequiresSoftClosed,
            failure_class: FailureClass::SessionConflict,
            reason_code: reason_codes::CLOSE_REQUIRES_SOFT_CLOSED,
            message: format!(
                "session {} cannot close from state {:?}; it must be SoftClosed first",
                session_id.0, state
            ),
        }
    }

    fn close_requires_no_attachments(session_id: SessionId) -> Self {
        Self {
            kind: SessionFoundationErrorKind::CloseRequiresNoAttachments,
            failure_class: FailureClass::SessionConflict,
            reason_code: reason_codes::CLOSE_REQUIRES_NO_ATTACHMENTS,
            message: format!(
                "session {} cannot close while devices remain attached",
                session_id.0
            ),
        }
    }

    fn too_many_devices(session_id: SessionId, max_attached_devices: usize) -> Self {
        Self {
            kind: SessionFoundationErrorKind::TooManyDevices,
            failure_class: FailureClass::PolicyViolation,
            reason_code: reason_codes::TOO_MANY_DEVICES,
            message: format!(
                "session {} exceeded the Slice 1C attached-device limit of {}",
                session_id.0, max_attached_devices
            ),
        }
    }

    fn duplicate_device_claim(
        session_id: SessionId,
        requesting_device_id: &str,
        active_device_id: &str,
    ) -> Self {
        Self {
            kind: SessionFoundationErrorKind::DuplicateDeviceClaim,
            failure_class: FailureClass::SessionConflict,
            reason_code: reason_codes::DUPLICATE_DEVICE_CLAIM,
            message: format!(
                "device {} cannot claim PRIMARY_INTERACTOR for session {} while {} already owns it",
                requesting_device_id, session_id.0, active_device_id
            ),
        }
    }

    fn invalid_access_class(
        session_id: SessionId,
        access_class: SessionAccessClass,
        coordination_state: SessionCoordinationState,
        state: SessionState,
    ) -> Self {
        Self {
            kind: SessionFoundationErrorKind::InvalidAccessClass,
            failure_class: FailureClass::PolicyViolation,
            reason_code: reason_codes::INVALID_ACCESS_CLASS,
            message: format!(
                "access class {:?} is not allowed for session {} while session_state={:?} coordination_state={:?}",
                access_class, session_id.0, state, coordination_state
            ),
        }
    }

    fn lease_expired(session_id: SessionId, runtime_id: &str) -> Self {
        Self {
            kind: SessionFoundationErrorKind::LeaseExpired,
            failure_class: FailureClass::RetryableRuntime,
            reason_code: reason_codes::LEASE_EXPIRED,
            message: format!(
                "session {} lease expired before runtime {} could mutate it",
                session_id.0, runtime_id
            ),
        }
    }

    fn not_session_owner(session_id: SessionId, runtime_id: &str, owner_runtime_id: &str) -> Self {
        Self {
            kind: SessionFoundationErrorKind::NotSessionOwner,
            failure_class: FailureClass::PolicyViolation,
            reason_code: reason_codes::NOT_SESSION_OWNER,
            message: format!(
                "runtime {} cannot mutate session {} because owner is {}",
                runtime_id, session_id.0, owner_runtime_id
            ),
        }
    }

    fn ownership_uncertain(session_id: SessionId, detail: &str) -> Self {
        Self {
            kind: SessionFoundationErrorKind::OwnershipUncertain,
            failure_class: FailureClass::RetryableRuntime,
            reason_code: reason_codes::OWNERSHIP_UNCERTAIN,
            message: format!("session {} ownership is uncertain: {detail}", session_id.0),
        }
    }

    fn ownership_transfer_pending(session_id: SessionId, target_runtime_id: &str) -> Self {
        Self {
            kind: SessionFoundationErrorKind::OwnershipTransferPending,
            failure_class: FailureClass::SessionConflict,
            reason_code: reason_codes::OWNERSHIP_TRANSFER_PENDING,
            message: format!(
                "session {} already has an ownership transfer pending for {}",
                session_id.0, target_runtime_id
            ),
        }
    }

    fn ownership_transfer_not_pending(session_id: SessionId) -> Self {
        Self {
            kind: SessionFoundationErrorKind::OwnershipTransferNotPending,
            failure_class: FailureClass::SessionConflict,
            reason_code: reason_codes::OWNERSHIP_TRANSFER_NOT_PENDING,
            message: format!(
                "session {} does not have an ownership transfer pending",
                session_id.0
            ),
        }
    }

    fn ownership_transfer_drain_required(session_id: SessionId) -> Self {
        Self {
            kind: SessionFoundationErrorKind::OwnershipTransferDrainRequired,
            failure_class: FailureClass::SessionConflict,
            reason_code: reason_codes::OWNERSHIP_TRANSFER_DRAIN_REQUIRED,
            message: format!(
                "session {} cannot transfer ownership until active and deferred mutations drain",
                session_id.0
            ),
        }
    }

    fn ownership_transfer_target_mismatch(
        session_id: SessionId,
        expected_target_runtime_id: &str,
        actual_target_runtime_id: &str,
    ) -> Self {
        Self {
            kind: SessionFoundationErrorKind::OwnershipTransferTargetMismatch,
            failure_class: FailureClass::SessionConflict,
            reason_code: reason_codes::OWNERSHIP_TRANSFER_TARGET_MISMATCH,
            message: format!(
                "session {} transfer target mismatch: expected {}, got {}",
                session_id.0, expected_target_runtime_id, actual_target_runtime_id
            ),
        }
    }

    fn ownership_transfer_invalid_target(session_id: SessionId, target_runtime_id: &str) -> Self {
        Self {
            kind: SessionFoundationErrorKind::OwnershipTransferInvalidTarget,
            failure_class: FailureClass::PolicyViolation,
            reason_code: reason_codes::OWNERSHIP_TRANSFER_INVALID_TARGET,
            message: format!(
                "session {} transfer target {} is not valid for this ownership change",
                session_id.0, target_runtime_id
            ),
        }
    }

    fn backpressure_exceeded(
        session_id: SessionId,
        max_pending_turns: usize,
        device_id: &str,
        device_turn_sequence: u64,
    ) -> Self {
        Self {
            kind: SessionFoundationErrorKind::BackpressureExceeded,
            failure_class: FailureClass::RetryableRuntime,
            reason_code: reason_codes::BACKPRESSURE_EXCEEDED,
            message: format!(
                "session {} exceeded pending-turn threshold {} while deferring device {} sequence {}",
                session_id.0, max_pending_turns, device_id, device_turn_sequence
            ),
        }
    }

    fn integrity_violation(session_id: SessionId, detail: &str) -> Self {
        Self {
            kind: SessionFoundationErrorKind::IntegrityViolation,
            failure_class: FailureClass::ExecutionFailure,
            reason_code: reason_codes::INTEGRITY_VIOLATION,
            message: format!("session {} integrity violation: {detail}", session_id.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSessionFoundationConfig {
    pub max_attached_devices: usize,
    pub max_events: usize,
    pub max_pending_turns: usize,
    pub lease_duration_ms: i64,
    pub runtime_identity: String,
}

impl RuntimeSessionFoundationConfig {
    pub fn slice_1c_defaults() -> Self {
        Self {
            max_attached_devices: 8,
            max_events: 256,
            max_pending_turns: 2,
            lease_duration_ms: 30_000,
            runtime_identity: "runtime.slice1.local".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceTimelineClassification {
    New,
    Retry,
    Stale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionDetachDisposition {
    Detached,
    DetachedAndSoftClosed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SessionCoordinationState {
    PrimaryOwned,
    TransferPending,
    FailoverRecovering,
    OwnershipUncertain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SessionConsistencyLevel {
    Strict,
    LeasedDistributed,
    DegradedRecovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SessionAccessClass {
    PrimaryInteractor,
    SecondaryViewer,
    LimitedAttach,
    RecoveryAttach,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionFoundationEventKind {
    SessionCreated,
    SessionResumed,
    SessionRecovered,
    SessionSuspended,
    SessionClosed,
    DeviceAttached,
    DeviceDetached,
    TurnStarted,
    TurnCompleted,
    TurnDeferred,
    BackpressureRejected,
    RetryReused,
    StaleRejected,
    OwnershipTransferRequested,
    OwnershipTransferAcknowledged,
    OwnershipTransferRejected,
    FailoverRecoveryStarted,
    FailoverRecoveryCompleted,
    LeaseRenewed,
    IntegrityViolation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionFoundationEvent {
    pub kind: SessionFoundationEventKind,
    pub session_id: SessionId,
    pub session_state: SessionState,
    pub turn_id: Option<TurnId>,
    pub device_id: Option<String>,
    pub device_turn_sequence: Option<u64>,
    pub detail: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionFoundationCounters {
    pub sessions_created: u64,
    pub devices_attached: u64,
    pub resumes: u64,
    pub recovers: u64,
    pub suspends: u64,
    pub detaches: u64,
    pub turns_started: u64,
    pub turns_completed: u64,
    pub retries_reused: u64,
    pub stale_rejections: u64,
    pub single_writer_rejections: u64,
    pub turn_deferrals: u64,
    pub backpressure_rejections: u64,
    pub ownership_transfer_requests: u64,
    pub ownership_transfer_acknowledgements: u64,
    pub ownership_transfer_rejections: u64,
    pub failover_recoveries_started: u64,
    pub failover_recoveries_completed: u64,
    pub lease_renewals: u64,
    pub integrity_violations: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionCoordinationView {
    pub session_id: SessionId,
    pub coordination_state: SessionCoordinationState,
    pub consistency_level: SessionConsistencyLevel,
    pub owner_runtime_id: String,
    pub pending_transfer_target: Option<String>,
    pub lease_token: String,
    pub lease_expires_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionAccessSnapshot {
    pub device_id: String,
    pub access_class: SessionAccessClass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeferredTurnSnapshot {
    pub device_id: String,
    pub device_turn_sequence: u64,
    pub runtime_id: String,
    pub access_class: SessionAccessClass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRuntimeProjection {
    pub session_id: SessionId,
    pub turn_id: Option<TurnId>,
    pub session_state: SessionState,
    pub device_turn_sequence: Option<u64>,
    pub attach_outcome: Option<SessionAttachOutcome>,
}

impl SessionRuntimeProjection {
    pub fn bind_to_runtime_envelope(
        &self,
        envelope: &RuntimeExecutionEnvelope,
    ) -> Result<RuntimeExecutionEnvelope, ContractViolation> {
        let mut bound = envelope.clone();
        bound.session_id = Some(self.session_id);
        if let Some(turn_id) = self.turn_id {
            bound.turn_id = turn_id;
        }
        bound.device_turn_sequence = self.device_turn_sequence;
        bound.session_attach_outcome = self.attach_outcome;
        bound.validate()?;
        Ok(bound)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionAttachResult {
    pub projection: SessionRuntimeProjection,
    pub attached_devices: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionResumeResult {
    pub projection: SessionRuntimeProjection,
    pub attached_devices: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRecoverResult {
    pub projection: SessionRuntimeProjection,
    pub attached_devices: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionDetachResult {
    pub projection: SessionRuntimeProjection,
    pub remaining_devices: Vec<String>,
    pub disposition: SessionDetachDisposition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionTurnPermit {
    pub session_id: SessionId,
    pub turn_id: TurnId,
    pub device_id: String,
    pub device_turn_sequence: u64,
    pub previous_state: SessionState,
    pub attach_outcome: Option<SessionAttachOutcome>,
    pub runtime_id: String,
    pub access_class: SessionAccessClass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionTurnDeferred {
    pub session_id: SessionId,
    pub device_id: String,
    pub device_turn_sequence: u64,
    pub pending_turn_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionTurnResolution {
    Started(SessionTurnPermit),
    Retry(SessionRuntimeProjection),
    Deferred(SessionTurnDeferred),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionTurnCommit {
    pub projection: SessionRuntimeProjection,
    pub classification: DeviceTimelineClassification,
    pub previous_state: SessionState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage5TurnAuthorityDisposition {
    CurrentCommittedTurn,
    RetryReusedResult,
    DeferredTurn,
    StaleTurnQuarantined,
    SupersededTurnQuarantined,
    CancelledTurnQuarantined,
    AbandonedTurnQuarantined,
    ClosedSessionRejected,
    RecordArtifactOnly,
}

impl Stage5TurnAuthorityDisposition {
    pub const fn default_reason_code(self) -> &'static str {
        match self {
            Stage5TurnAuthorityDisposition::CurrentCommittedTurn => {
                reason_codes::STAGE5_CURRENT_COMMITTED_TURN
            }
            Stage5TurnAuthorityDisposition::RetryReusedResult => {
                reason_codes::STAGE5_RETRY_REUSED_RESULT
            }
            Stage5TurnAuthorityDisposition::DeferredTurn => reason_codes::STAGE5_DEFERRED_TURN,
            Stage5TurnAuthorityDisposition::StaleTurnQuarantined => {
                reason_codes::STAGE5_STALE_TURN_QUARANTINED
            }
            Stage5TurnAuthorityDisposition::SupersededTurnQuarantined => {
                reason_codes::STAGE5_SUPERSEDED_TURN_QUARANTINED
            }
            Stage5TurnAuthorityDisposition::CancelledTurnQuarantined => {
                reason_codes::STAGE5_CANCELLED_TURN_QUARANTINED
            }
            Stage5TurnAuthorityDisposition::AbandonedTurnQuarantined => {
                reason_codes::STAGE5_ABANDONED_TURN_QUARANTINED
            }
            Stage5TurnAuthorityDisposition::ClosedSessionRejected => {
                reason_codes::STAGE5_CLOSED_SESSION_REJECTED
            }
            Stage5TurnAuthorityDisposition::RecordArtifactOnly => {
                reason_codes::STAGE5_RECORD_ARTIFACT_ONLY
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stage5TurnWorkAuthority {
    pub can_enter_understanding: bool,
    pub can_render_current_result: bool,
    pub can_route_tools: bool,
    pub can_route_search: bool,
    pub can_route_providers: bool,
    pub can_route_tts: bool,
    pub can_route_protected_execution: bool,
}

impl Stage5TurnWorkAuthority {
    pub const fn current_committed_turn() -> Self {
        Self {
            can_enter_understanding: true,
            can_render_current_result: true,
            can_route_tools: false,
            can_route_search: false,
            can_route_providers: false,
            can_route_tts: false,
            can_route_protected_execution: false,
        }
    }

    pub const fn quarantined() -> Self {
        Self {
            can_enter_understanding: false,
            can_render_current_result: false,
            can_route_tools: false,
            can_route_search: false,
            can_route_providers: false,
            can_route_tts: false,
            can_route_protected_execution: false,
        }
    }

    pub const fn can_route_any_work(self) -> bool {
        self.can_route_tools
            || self.can_route_search
            || self.can_route_providers
            || self.can_route_tts
            || self.can_route_protected_execution
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage5TurnAuthorityPacket {
    pub session_id: SessionId,
    pub turn_id: Option<TurnId>,
    pub device_id: Option<String>,
    pub device_turn_sequence: Option<u64>,
    pub session_state: SessionState,
    pub disposition: Stage5TurnAuthorityDisposition,
    pub reason_code: &'static str,
    pub authority: Stage5TurnWorkAuthority,
}

impl Stage5TurnAuthorityPacket {
    pub fn current_committed(
        session_id: SessionId,
        turn_id: TurnId,
        device_id: impl Into<String>,
        device_turn_sequence: u64,
        session_state: SessionState,
    ) -> Result<Self, ContractViolation> {
        let packet = Self {
            session_id,
            turn_id: Some(turn_id),
            device_id: Some(device_id.into()),
            device_turn_sequence: Some(device_turn_sequence),
            session_state,
            disposition: Stage5TurnAuthorityDisposition::CurrentCommittedTurn,
            reason_code: Stage5TurnAuthorityDisposition::CurrentCommittedTurn.default_reason_code(),
            authority: Stage5TurnWorkAuthority::current_committed_turn(),
        };
        packet.validate()?;
        Ok(packet)
    }

    pub fn quarantined(
        session_id: SessionId,
        turn_id: Option<TurnId>,
        device_id: Option<String>,
        device_turn_sequence: Option<u64>,
        session_state: SessionState,
        disposition: Stage5TurnAuthorityDisposition,
    ) -> Result<Self, ContractViolation> {
        let packet = Self {
            session_id,
            turn_id,
            device_id,
            device_turn_sequence,
            session_state,
            disposition,
            reason_code: disposition.default_reason_code(),
            authority: Stage5TurnWorkAuthority::quarantined(),
        };
        packet.validate()?;
        Ok(packet)
    }

    pub const fn can_enter_understanding(&self) -> bool {
        self.authority.can_enter_understanding
    }

    pub const fn can_render_current_result(&self) -> bool {
        self.authority.can_render_current_result
    }

    pub const fn can_route_any_work(&self) -> bool {
        self.authority.can_route_any_work()
    }
}

impl Validate for Stage5TurnAuthorityPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.reason_code.trim().is_empty() || self.reason_code.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "stage5_turn_authority_packet.reason_code",
                reason: "must be a bounded non-empty reason code",
            });
        }
        if let Some(device_id) = self.device_id.as_ref() {
            validate_stage5_ascii_token("stage5_turn_authority_packet.device_id", device_id, 128)?;
        }
        if matches!(self.device_turn_sequence, Some(0)) {
            return Err(ContractViolation::InvalidValue {
                field: "stage5_turn_authority_packet.device_turn_sequence",
                reason: "must be greater than zero when present",
            });
        }
        if self.authority.can_route_any_work() {
            return Err(ContractViolation::InvalidValue {
                field: "stage5_turn_authority_packet.authority",
                reason: "session/turn authority cannot route tools, search, providers, TTS, or protected execution",
            });
        }

        match self.disposition {
            Stage5TurnAuthorityDisposition::CurrentCommittedTurn => {
                if self.turn_id.is_none()
                    || self.device_id.is_none()
                    || self.device_turn_sequence.is_none()
                    || self.session_state == SessionState::Closed
                    || !self.authority.can_enter_understanding
                    || !self.authority.can_render_current_result
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage5_turn_authority_packet",
                        reason: "current committed turns require turn/device/sequence, open session state, understanding admission, and current-render authority",
                    });
                }
            }
            Stage5TurnAuthorityDisposition::ClosedSessionRejected => {
                if self.session_state != SessionState::Closed {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage5_turn_authority_packet.session_state",
                        reason: "closed-session rejection must carry session_state=Closed",
                    });
                }
                if self.authority.can_enter_understanding
                    || self.authority.can_render_current_result
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage5_turn_authority_packet.authority",
                        reason:
                            "closed-session turns cannot enter understanding or render as current",
                    });
                }
            }
            _ => {
                if self.authority.can_enter_understanding
                    || self.authority.can_render_current_result
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage5_turn_authority_packet.authority",
                        reason: "quarantined turn dispositions cannot enter understanding or render as current",
                    });
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceTimelineSnapshot {
    pub device_id: String,
    pub highest_seen_sequence: u64,
    pub last_turn_id: TurnId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveWriterSnapshot {
    pub device_id: String,
    pub turn_id: TurnId,
    pub device_turn_sequence: u64,
    pub runtime_id: String,
    pub access_class: SessionAccessClass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionFoundationSnapshot {
    pub session_id: SessionId,
    pub session_state: SessionState,
    pub coordination_state: SessionCoordinationState,
    pub consistency_level: SessionConsistencyLevel,
    pub owner_runtime_id: String,
    pub pending_transfer_target: Option<String>,
    pub lease_token: String,
    pub lease_expires_at_ms: i64,
    pub next_turn_id: u64,
    pub attached_devices: Vec<String>,
    pub access_classes: Vec<SessionAccessSnapshot>,
    pub device_timelines: Vec<DeviceTimelineSnapshot>,
    pub deferred_turns: Vec<DeferredTurnSnapshot>,
    pub active_writer: Option<ActiveWriterSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedSoftClosedSessionImport {
    pub session_id: SessionId,
    pub persisted_session_state: SessionState,
    pub attached_devices: Vec<String>,
    pub last_attached_device_id: String,
    pub last_turn_id: Option<TurnId>,
    pub device_turn_sequences: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedSuspendedSessionImport {
    pub session_id: SessionId,
    pub persisted_session_state: SessionState,
    pub attached_devices: Vec<String>,
    pub last_attached_device_id: String,
    pub last_turn_id: Option<TurnId>,
    pub device_turn_sequences: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedAttachableSessionImport {
    pub session_id: SessionId,
    pub persisted_session_state: SessionState,
    pub attached_devices: Vec<String>,
    pub last_attached_device_id: String,
    pub last_turn_id: Option<TurnId>,
    pub device_turn_sequences: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SessionOwnershipRecord {
    owner_runtime_id: String,
    lease_generation: u64,
    lease_token: String,
    lease_expires_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeviceTimelineRecord {
    highest_seen_sequence: u64,
    last_turn_id: TurnId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ActiveTurnMutation {
    turn_id: TurnId,
    device_id: String,
    device_turn_sequence: u64,
    runtime_id: String,
    access_class: SessionAccessClass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeferredTurnRecord {
    runtime_id: String,
    access_class: SessionAccessClass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SessionRecord {
    state: SessionState,
    coordination_state: SessionCoordinationState,
    consistency_level: SessionConsistencyLevel,
    ownership: SessionOwnershipRecord,
    pending_transfer_target: Option<String>,
    attached_devices: BTreeSet<String>,
    device_access_classes: BTreeMap<String, SessionAccessClass>,
    device_timeline_map: BTreeMap<String, DeviceTimelineRecord>,
    deferred_turns: BTreeMap<(String, u64), DeferredTurnRecord>,
    next_turn_id: u64,
    active_writer: Option<ActiveTurnMutation>,
}

impl SessionRecord {
    fn new(
        session_id: SessionId,
        runtime_identity: &str,
        logical_now_ms: i64,
        lease_duration_ms: i64,
    ) -> Self {
        Self {
            state: SessionState::Closed,
            coordination_state: SessionCoordinationState::PrimaryOwned,
            consistency_level: SessionConsistencyLevel::Strict,
            ownership: SessionOwnershipRecord {
                owner_runtime_id: runtime_identity.to_string(),
                lease_generation: 1,
                lease_token: lease_token(session_id, runtime_identity, 1),
                lease_expires_at_ms: logical_now_ms.saturating_add(lease_duration_ms),
            },
            pending_transfer_target: None,
            attached_devices: BTreeSet::new(),
            device_access_classes: BTreeMap::new(),
            device_timeline_map: BTreeMap::new(),
            deferred_turns: BTreeMap::new(),
            next_turn_id: 1,
            active_writer: None,
        }
    }

    fn projection(
        &self,
        session_id: SessionId,
        turn_id: Option<TurnId>,
        device_turn_sequence: Option<u64>,
        attach_outcome: Option<SessionAttachOutcome>,
    ) -> SessionRuntimeProjection {
        SessionRuntimeProjection {
            session_id,
            turn_id,
            session_state: self.state,
            device_turn_sequence,
            attach_outcome,
        }
    }

    fn attached_devices(&self) -> Vec<String> {
        self.attached_devices.iter().cloned().collect()
    }

    fn snapshot(&self, session_id: SessionId) -> SessionFoundationSnapshot {
        SessionFoundationSnapshot {
            session_id,
            session_state: self.state,
            coordination_state: self.coordination_state,
            consistency_level: self.consistency_level,
            owner_runtime_id: self.ownership.owner_runtime_id.clone(),
            pending_transfer_target: self.pending_transfer_target.clone(),
            lease_token: self.ownership.lease_token.clone(),
            lease_expires_at_ms: self.ownership.lease_expires_at_ms,
            next_turn_id: self.next_turn_id,
            attached_devices: self.attached_devices(),
            access_classes: self
                .device_access_classes
                .iter()
                .map(|(device_id, access_class)| SessionAccessSnapshot {
                    device_id: device_id.clone(),
                    access_class: *access_class,
                })
                .collect(),
            device_timelines: self
                .device_timeline_map
                .iter()
                .map(|(device_id, record)| DeviceTimelineSnapshot {
                    device_id: device_id.clone(),
                    highest_seen_sequence: record.highest_seen_sequence,
                    last_turn_id: record.last_turn_id,
                })
                .collect(),
            deferred_turns: self
                .deferred_turns
                .iter()
                .map(
                    |((device_id, device_turn_sequence), deferred)| DeferredTurnSnapshot {
                        device_id: device_id.clone(),
                        device_turn_sequence: *device_turn_sequence,
                        runtime_id: deferred.runtime_id.clone(),
                        access_class: deferred.access_class,
                    },
                )
                .collect(),
            active_writer: self
                .active_writer
                .as_ref()
                .map(|writer| ActiveWriterSnapshot {
                    device_id: writer.device_id.clone(),
                    turn_id: writer.turn_id,
                    device_turn_sequence: writer.device_turn_sequence,
                    runtime_id: writer.runtime_id.clone(),
                    access_class: writer.access_class,
                }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeSessionFoundation {
    config: RuntimeSessionFoundationConfig,
    logical_now_ms: i64,
    next_session_id: u128,
    sessions: BTreeMap<SessionId, SessionRecord>,
    events: Vec<SessionFoundationEvent>,
    counters: SessionFoundationCounters,
}

impl Default for RuntimeSessionFoundation {
    fn default() -> Self {
        Self::new(RuntimeSessionFoundationConfig::slice_1c_defaults())
    }
}

impl RuntimeSessionFoundation {
    pub fn new(config: RuntimeSessionFoundationConfig) -> Self {
        Self {
            config,
            logical_now_ms: 0,
            next_session_id: 1,
            sessions: BTreeMap::new(),
            events: Vec::new(),
            counters: SessionFoundationCounters::default(),
        }
    }

    pub fn register_slice_1c_session_foundation_services<C, S>(
        container: &mut RuntimeServiceContainer<C, S>,
    ) -> Result<(), RuntimeBootstrapError>
    where
        C: RuntimeClock,
        S: RuntimeSecretsProvider,
    {
        container.register_service("runtime_session_store", &["runtime_clock"])?;
        container.register_service(
            "runtime_session_identifier_generator",
            &["runtime_session_store"],
        )?;
        container.register_service(
            "runtime_session_turn_gate",
            &[
                "runtime_session_store",
                "runtime_session_identifier_generator",
            ],
        )?;
        container.register_service(
            "runtime_session_event_stream",
            &["runtime_session_store", "runtime_session_turn_gate"],
        )?;
        container.register_service(
            "runtime_session_projection",
            &["runtime_session_store", "runtime_session_turn_gate"],
        )?;
        container.register_service(
            "runtime_session_coordination",
            &["runtime_session_store", "runtime_session_projection"],
        )?;
        container.register_service(
            "runtime_session_lease",
            &["runtime_session_store", "runtime_session_coordination"],
        )?;
        container.register_service(
            "runtime_session_access_gate",
            &["runtime_session_store", "runtime_session_coordination"],
        )?;
        container.register_service(
            "runtime_session_conflict_resolution",
            &["runtime_session_turn_gate", "runtime_session_access_gate"],
        )?;
        container.register_service(
            "runtime_session_backpressure",
            &[
                "runtime_session_turn_gate",
                "runtime_session_conflict_resolution",
            ],
        )?;
        container.register_service(
            "runtime_session_transfer",
            &["runtime_session_lease", "runtime_session_coordination"],
        )?;
        Ok(())
    }

    pub fn counters(&self) -> &SessionFoundationCounters {
        &self.counters
    }

    pub fn events(&self) -> &[SessionFoundationEvent] {
        &self.events
    }

    pub fn current_logical_time_ms(&self) -> i64 {
        self.logical_now_ms
    }

    pub fn advance_logical_time_ms(&mut self, delta_ms: i64) {
        if delta_ms > 0 {
            self.logical_now_ms = self.logical_now_ms.saturating_add(delta_ms);
        }
    }

    pub fn session_snapshot(
        &self,
        session_id: SessionId,
    ) -> Result<SessionFoundationSnapshot, SessionFoundationError> {
        let record = self
            .sessions
            .get(&session_id)
            .ok_or_else(|| SessionFoundationError::session_not_found(session_id))?;
        Ok(snapshot_with_effective_posture(
            record,
            session_id,
            self.logical_now_ms,
        ))
    }

    pub fn coordination_view(
        &self,
        session_id: SessionId,
    ) -> Result<SessionCoordinationView, SessionFoundationError> {
        let record = self
            .sessions
            .get(&session_id)
            .ok_or_else(|| SessionFoundationError::session_not_found(session_id))?;
        Ok(coordination_view_from_record(
            record,
            session_id,
            self.logical_now_ms,
        ))
    }

    pub fn import_persisted_attachable_session(
        &mut self,
        import: PersistedAttachableSessionImport,
    ) -> Result<SessionFoundationSnapshot, SessionFoundationError> {
        let PersistedAttachableSessionImport {
            session_id,
            persisted_session_state,
            attached_devices,
            last_attached_device_id,
            last_turn_id,
            device_turn_sequences,
        } = import;

        if !matches!(
            persisted_session_state,
            SessionState::Open | SessionState::Active
        ) {
            return Err(SessionFoundationError::attach_not_allowed(
                session_id,
                persisted_session_state,
            ));
        }
        if self.sessions.contains_key(&session_id) {
            return Err(SessionFoundationError::integrity_violation(
                session_id,
                "persisted attachable session import cannot overwrite an existing runtime record",
            ));
        }
        if attached_devices.is_empty() {
            return Err(SessionFoundationError::integrity_violation(
                session_id,
                "persisted attachable session import requires at least one attached device",
            ));
        }
        if let Some(last_turn_id) = last_turn_id {
            last_turn_id.validate().map_err(|_| {
                SessionFoundationError::integrity_violation(
                    session_id,
                    "persisted attachable session import carried an invalid last_turn_id",
                )
            })?;
        }
        if !device_turn_sequences.is_empty() && last_turn_id.is_none() {
            return Err(SessionFoundationError::integrity_violation(
                session_id,
                "persisted attachable session import requires last_turn_id when device sequence history is present",
            ));
        }

        let mut record = SessionRecord::new(
            session_id,
            self.local_runtime_id(),
            self.logical_now_ms,
            self.config.lease_duration_ms,
        );
        record.state = persisted_session_state;

        for raw_device_id in attached_devices {
            let device_id = DeviceId::new(raw_device_id.clone()).map_err(|_| {
                SessionFoundationError::integrity_violation(
                    session_id,
                    "persisted attachable session import carried an invalid attached device identifier",
                )
            })?;
            attach_device_to_record(
                session_id,
                &mut record,
                device_id.as_str(),
                self.config.max_attached_devices,
            )?;
        }

        let last_attached_device_id =
            DeviceId::new(last_attached_device_id.clone()).map_err(|_| {
                SessionFoundationError::integrity_violation(
                    session_id,
                    "persisted attachable session import carried an invalid last_attached_device_id",
                )
            })?;
        if !record
            .attached_devices
            .contains(last_attached_device_id.as_str())
        {
            return Err(SessionFoundationError::integrity_violation(
                session_id,
                "persisted attachable session import last_attached_device_id must remain attached",
            ));
        }

        let attached_device_keys: Vec<String> = record.attached_devices.iter().cloned().collect();
        for device_id in attached_device_keys {
            let access_class = if device_id == last_attached_device_id.as_str() {
                SessionAccessClass::PrimaryInteractor
            } else {
                SessionAccessClass::LimitedAttach
            };
            assign_access_class_to_record(
                session_id,
                &mut record,
                &device_id,
                access_class,
                SessionCoordinationState::PrimaryOwned,
                false,
            )?;
        }

        if let Some(last_turn_id) = last_turn_id {
            record.next_turn_id = record.next_turn_id.max(last_turn_id.0.saturating_add(1));

            for (raw_device_id, highest_seen_sequence) in device_turn_sequences {
                let device_id = DeviceId::new(raw_device_id.clone()).map_err(|_| {
                    SessionFoundationError::integrity_violation(
                        session_id,
                        "persisted attachable session import carried an invalid device timeline identifier",
                    )
                })?;
                if !record.attached_devices.contains(device_id.as_str()) {
                    return Err(SessionFoundationError::integrity_violation(
                        session_id,
                        "persisted attachable session import device timeline must belong to an attached device",
                    ));
                }
                validate_device_turn_sequence(device_id.as_str(), highest_seen_sequence)?;
                record.device_timeline_map.insert(
                    device_id.as_str().to_string(),
                    DeviceTimelineRecord {
                        highest_seen_sequence,
                        last_turn_id,
                    },
                );
            }
        }

        self.next_session_id = self.next_session_id.max(session_id.0.saturating_add(1));
        let snapshot = snapshot_with_effective_posture(&record, session_id, self.logical_now_ms);
        self.sessions.insert(session_id, record);
        Ok(snapshot)
    }

    pub fn import_persisted_soft_closed_session(
        &mut self,
        import: PersistedSoftClosedSessionImport,
    ) -> Result<SessionFoundationSnapshot, SessionFoundationError> {
        let PersistedSoftClosedSessionImport {
            session_id,
            persisted_session_state,
            attached_devices,
            last_attached_device_id,
            last_turn_id,
            device_turn_sequences,
        } = import;

        if persisted_session_state != SessionState::SoftClosed {
            return Err(SessionFoundationError::resume_not_recoverable(
                session_id,
                persisted_session_state,
            ));
        }
        if self.sessions.contains_key(&session_id) {
            return Err(SessionFoundationError::integrity_violation(
                session_id,
                "persisted soft-closed session import cannot overwrite an existing runtime record",
            ));
        }
        if attached_devices.is_empty() {
            return Err(SessionFoundationError::integrity_violation(
                session_id,
                "persisted soft-closed session import requires at least one attached device",
            ));
        }
        if let Some(last_turn_id) = last_turn_id {
            last_turn_id.validate().map_err(|_| {
                SessionFoundationError::integrity_violation(
                    session_id,
                    "persisted soft-closed session import carried an invalid last_turn_id",
                )
            })?;
        }
        if !device_turn_sequences.is_empty() && last_turn_id.is_none() {
            return Err(SessionFoundationError::integrity_violation(
                session_id,
                "persisted soft-closed session import requires last_turn_id when device sequence history is present",
            ));
        }

        let mut record = SessionRecord::new(
            session_id,
            self.local_runtime_id(),
            self.logical_now_ms,
            self.config.lease_duration_ms,
        );
        record.state = SessionState::SoftClosed;

        for raw_device_id in attached_devices {
            let device_id = DeviceId::new(raw_device_id.clone()).map_err(|_| {
                SessionFoundationError::integrity_violation(
                    session_id,
                    "persisted soft-closed session import carried an invalid attached device identifier",
                )
            })?;
            attach_device_to_record(
                session_id,
                &mut record,
                device_id.as_str(),
                self.config.max_attached_devices,
            )?;
        }

        let last_attached_device_id =
            DeviceId::new(last_attached_device_id.clone()).map_err(|_| {
                SessionFoundationError::integrity_violation(
                session_id,
                "persisted soft-closed session import carried an invalid last_attached_device_id",
            )
            })?;
        if !record
            .attached_devices
            .contains(last_attached_device_id.as_str())
        {
            return Err(SessionFoundationError::integrity_violation(
                session_id,
                "persisted soft-closed session import last_attached_device_id must remain attached",
            ));
        }

        let attached_device_keys: Vec<String> = record.attached_devices.iter().cloned().collect();
        for device_id in attached_device_keys {
            let access_class = if device_id == last_attached_device_id.as_str() {
                SessionAccessClass::PrimaryInteractor
            } else {
                SessionAccessClass::LimitedAttach
            };
            assign_access_class_to_record(
                session_id,
                &mut record,
                &device_id,
                access_class,
                SessionCoordinationState::PrimaryOwned,
                false,
            )?;
        }

        if let Some(last_turn_id) = last_turn_id {
            record.next_turn_id = record.next_turn_id.max(last_turn_id.0.saturating_add(1));

            for (raw_device_id, highest_seen_sequence) in device_turn_sequences {
                let device_id = DeviceId::new(raw_device_id.clone()).map_err(|_| {
                    SessionFoundationError::integrity_violation(
                        session_id,
                        "persisted soft-closed session import carried an invalid device timeline identifier",
                    )
                })?;
                if !record.attached_devices.contains(device_id.as_str()) {
                    return Err(SessionFoundationError::integrity_violation(
                        session_id,
                        "persisted soft-closed session import device timeline must belong to an attached device",
                    ));
                }
                validate_device_turn_sequence(device_id.as_str(), highest_seen_sequence)?;
                record.device_timeline_map.insert(
                    device_id.as_str().to_string(),
                    DeviceTimelineRecord {
                        highest_seen_sequence,
                        last_turn_id,
                    },
                );
            }
        }

        self.next_session_id = self.next_session_id.max(session_id.0.saturating_add(1));
        let snapshot = snapshot_with_effective_posture(&record, session_id, self.logical_now_ms);
        self.sessions.insert(session_id, record);
        Ok(snapshot)
    }

    pub fn import_persisted_suspended_session(
        &mut self,
        import: PersistedSuspendedSessionImport,
    ) -> Result<SessionFoundationSnapshot, SessionFoundationError> {
        let PersistedSuspendedSessionImport {
            session_id,
            persisted_session_state,
            attached_devices,
            last_attached_device_id,
            last_turn_id,
            device_turn_sequences,
        } = import;

        if persisted_session_state != SessionState::Suspended {
            return Err(SessionFoundationError::recover_not_suspended(
                session_id,
                persisted_session_state,
            ));
        }
        if self.sessions.contains_key(&session_id) {
            return Err(SessionFoundationError::integrity_violation(
                session_id,
                "persisted suspended session import cannot overwrite an existing runtime record",
            ));
        }
        if attached_devices.is_empty() {
            return Err(SessionFoundationError::integrity_violation(
                session_id,
                "persisted suspended session import requires at least one attached device",
            ));
        }
        if let Some(last_turn_id) = last_turn_id {
            last_turn_id.validate().map_err(|_| {
                SessionFoundationError::integrity_violation(
                    session_id,
                    "persisted suspended session import carried an invalid last_turn_id",
                )
            })?;
        }
        if !device_turn_sequences.is_empty() && last_turn_id.is_none() {
            return Err(SessionFoundationError::integrity_violation(
                session_id,
                "persisted suspended session import requires last_turn_id when device sequence history is present",
            ));
        }

        let mut record = SessionRecord::new(
            session_id,
            self.local_runtime_id(),
            self.logical_now_ms,
            self.config.lease_duration_ms,
        );
        record.state = SessionState::Suspended;

        for raw_device_id in attached_devices {
            let device_id = DeviceId::new(raw_device_id.clone()).map_err(|_| {
                SessionFoundationError::integrity_violation(
                    session_id,
                    "persisted suspended session import carried an invalid attached device identifier",
                )
            })?;
            attach_device_to_record(
                session_id,
                &mut record,
                device_id.as_str(),
                self.config.max_attached_devices,
            )?;
        }

        let last_attached_device_id =
            DeviceId::new(last_attached_device_id.clone()).map_err(|_| {
                SessionFoundationError::integrity_violation(
                    session_id,
                    "persisted suspended session import carried an invalid last_attached_device_id",
                )
            })?;
        if !record
            .attached_devices
            .contains(last_attached_device_id.as_str())
        {
            return Err(SessionFoundationError::integrity_violation(
                session_id,
                "persisted suspended session import last_attached_device_id must remain attached",
            ));
        }

        let attached_device_keys: Vec<String> = record.attached_devices.iter().cloned().collect();
        for device_id in attached_device_keys {
            let access_class = if device_id == last_attached_device_id.as_str() {
                SessionAccessClass::PrimaryInteractor
            } else {
                SessionAccessClass::LimitedAttach
            };
            assign_access_class_to_record(
                session_id,
                &mut record,
                &device_id,
                access_class,
                SessionCoordinationState::PrimaryOwned,
                false,
            )?;
        }

        if let Some(last_turn_id) = last_turn_id {
            record.next_turn_id = record.next_turn_id.max(last_turn_id.0.saturating_add(1));

            for (raw_device_id, highest_seen_sequence) in device_turn_sequences {
                let device_id = DeviceId::new(raw_device_id.clone()).map_err(|_| {
                    SessionFoundationError::integrity_violation(
                        session_id,
                        "persisted suspended session import carried an invalid device timeline identifier",
                    )
                })?;
                if !record.attached_devices.contains(device_id.as_str()) {
                    return Err(SessionFoundationError::integrity_violation(
                        session_id,
                        "persisted suspended session import device timeline must belong to an attached device",
                    ));
                }
                validate_device_turn_sequence(device_id.as_str(), highest_seen_sequence)?;
                record.device_timeline_map.insert(
                    device_id.as_str().to_string(),
                    DeviceTimelineRecord {
                        highest_seen_sequence,
                        last_turn_id,
                    },
                );
            }
        }

        self.next_session_id = self.next_session_id.max(session_id.0.saturating_add(1));
        let snapshot = snapshot_with_effective_posture(&record, session_id, self.logical_now_ms);
        self.sessions.insert(session_id, record);
        Ok(snapshot)
    }

    pub fn create_session(
        &mut self,
        device_id: DeviceId,
    ) -> Result<SessionAttachResult, SessionFoundationError> {
        let session_id = self.allocate_session_id();
        let device_key = device_id.as_str().to_string();
        let logical_now_ms = self.logical_now_ms;

        let mut record = SessionRecord::new(
            session_id,
            self.local_runtime_id(),
            logical_now_ms,
            self.config.lease_duration_ms,
        );
        attach_device_to_record(
            session_id,
            &mut record,
            &device_key,
            self.config.max_attached_devices,
        )?;
        assign_access_class_to_record(
            session_id,
            &mut record,
            &device_key,
            SessionAccessClass::PrimaryInteractor,
            SessionCoordinationState::PrimaryOwned,
            true,
        )?;
        transition_record_state(&mut record, SessionState::Open, false)?;

        let projection = record.projection(
            session_id,
            None,
            None,
            Some(SessionAttachOutcome::NewSessionCreated),
        );
        let attached_devices = record.attached_devices();
        self.sessions.insert(session_id, record);
        self.counters.sessions_created += 1;
        self.counters.devices_attached += 1;

        self.emit_event(SessionFoundationEvent {
            kind: SessionFoundationEventKind::SessionCreated,
            session_id,
            session_state: SessionState::Open,
            turn_id: None,
            device_id: Some(device_key),
            device_turn_sequence: None,
            detail: "session created in Open state".to_string(),
        });

        Ok(SessionAttachResult {
            projection,
            attached_devices,
        })
    }

    pub fn start_new_session_turn(
        &mut self,
        device_id: DeviceId,
        device_turn_sequence: u64,
    ) -> Result<SessionTurnResolution, SessionFoundationError> {
        validate_device_turn_sequence(device_id.as_str(), device_turn_sequence)?;

        let session_id = self.allocate_session_id();
        let device_key = device_id.as_str().to_string();
        let logical_now_ms = self.logical_now_ms;

        let mut record = SessionRecord::new(
            session_id,
            self.local_runtime_id(),
            logical_now_ms,
            self.config.lease_duration_ms,
        );
        attach_device_to_record(
            session_id,
            &mut record,
            &device_key,
            self.config.max_attached_devices,
        )?;
        assign_access_class_to_record(
            session_id,
            &mut record,
            &device_key,
            SessionAccessClass::PrimaryInteractor,
            SessionCoordinationState::PrimaryOwned,
            true,
        )?;
        transition_record_state(&mut record, SessionState::Active, true)?;

        let turn_id = allocate_turn_id(&mut record);
        record.active_writer = Some(ActiveTurnMutation {
            turn_id,
            device_id: device_key.clone(),
            device_turn_sequence,
            runtime_id: self.local_runtime_id().to_string(),
            access_class: SessionAccessClass::PrimaryInteractor,
        });
        self.counters.sessions_created += 1;
        self.counters.devices_attached += 1;
        self.counters.turns_started += 1;

        self.sessions.insert(session_id, record);
        self.emit_event(SessionFoundationEvent {
            kind: SessionFoundationEventKind::SessionCreated,
            session_id,
            session_state: SessionState::Active,
            turn_id: Some(turn_id),
            device_id: Some(device_key.clone()),
            device_turn_sequence: Some(device_turn_sequence),
            detail: "session created through lawful Closed -> Active open bypass".to_string(),
        });
        self.emit_event(SessionFoundationEvent {
            kind: SessionFoundationEventKind::TurnStarted,
            session_id,
            session_state: SessionState::Active,
            turn_id: Some(turn_id),
            device_id: Some(device_key.clone()),
            device_turn_sequence: Some(device_turn_sequence),
            detail: "first session turn started through open bypass".to_string(),
        });

        Ok(SessionTurnResolution::Started(SessionTurnPermit {
            session_id,
            turn_id,
            device_id: device_key,
            device_turn_sequence,
            previous_state: SessionState::Closed,
            attach_outcome: Some(SessionAttachOutcome::NewSessionCreated),
            runtime_id: self.local_runtime_id().to_string(),
            access_class: SessionAccessClass::PrimaryInteractor,
        }))
    }

    pub fn attach_session(
        &mut self,
        session_id: SessionId,
        device_id: DeviceId,
    ) -> Result<SessionAttachResult, SessionFoundationError> {
        self.attach_session_with_access_claim(
            session_id,
            device_id,
            SessionAccessClass::LimitedAttach,
            None,
        )
    }

    pub fn attach_session_with_access_claim(
        &mut self,
        session_id: SessionId,
        device_id: DeviceId,
        access_class: SessionAccessClass,
        claimed_device_turn_sequence: Option<u64>,
    ) -> Result<SessionAttachResult, SessionFoundationError> {
        let local_runtime_id = self.local_runtime_id().to_string();
        self.attach_session_internal(
            session_id,
            device_id,
            access_class,
            claimed_device_turn_sequence,
            &local_runtime_id,
        )
    }

    pub fn resume_session(
        &mut self,
        session_id: SessionId,
        device_id: DeviceId,
    ) -> Result<SessionResumeResult, SessionFoundationError> {
        let local_runtime_id = self.local_runtime_id().to_string();
        self.resume_session_internal(session_id, device_id, &local_runtime_id)
    }

    pub fn suspend_session(
        &mut self,
        session_id: SessionId,
        detail: &str,
    ) -> Result<SessionRuntimeProjection, SessionFoundationError> {
        let local_runtime_id = self.local_runtime_id().to_string();
        let logical_now_ms = self.logical_now_ms;
        let (projection, session_state) = {
            let record = self
                .sessions
                .get_mut(&session_id)
                .ok_or_else(|| SessionFoundationError::session_not_found(session_id))?;

            ensure_primary_owned_runtime(session_id, record, &local_runtime_id, logical_now_ms)?;
            ensure_no_active_writer(session_id, record)?;
            transition_record_state(record, SessionState::Suspended, false)?;
            let session_state = record.state;
            let projection = record.projection(session_id, None, None, None);
            (projection, session_state)
        };
        self.counters.suspends += 1;

        self.emit_event(SessionFoundationEvent {
            kind: SessionFoundationEventKind::SessionSuspended,
            session_id,
            session_state,
            turn_id: None,
            device_id: None,
            device_turn_sequence: None,
            detail: detail.to_string(),
        });

        Ok(projection)
    }

    pub fn recover_session(
        &mut self,
        session_id: SessionId,
        device_id: DeviceId,
    ) -> Result<SessionRecoverResult, SessionFoundationError> {
        let local_runtime_id = self.local_runtime_id().to_string();
        self.recover_session_internal(session_id, device_id, &local_runtime_id)
    }

    pub fn detach_session(
        &mut self,
        session_id: SessionId,
        device_id: &DeviceId,
    ) -> Result<SessionDetachResult, SessionFoundationError> {
        let device_key = device_id.as_str().to_string();
        let local_runtime_id = self.local_runtime_id().to_string();
        let logical_now_ms = self.logical_now_ms;
        let (projection, remaining_devices, disposition, session_state) = {
            let record = self
                .sessions
                .get_mut(&session_id)
                .ok_or_else(|| SessionFoundationError::session_not_found(session_id))?;

            ensure_primary_owned_runtime(session_id, record, &local_runtime_id, logical_now_ms)?;
            ensure_no_active_writer(session_id, record)?;
            if !record.attached_devices.remove(&device_key) {
                return Err(SessionFoundationError::detach_not_attached(
                    session_id,
                    &device_key,
                ));
            }
            record.device_access_classes.remove(&device_key);
            record
                .deferred_turns
                .retain(|(pending_device_id, _), _| pending_device_id != &device_key);

            let disposition =
                if record.attached_devices.is_empty() && record.state == SessionState::Active {
                    transition_record_state(record, SessionState::SoftClosed, false)?;
                    SessionDetachDisposition::DetachedAndSoftClosed
                } else {
                    SessionDetachDisposition::Detached
                };

            let session_state = record.state;
            let projection = record.projection(session_id, None, None, None);
            let remaining_devices = record.attached_devices();
            (projection, remaining_devices, disposition, session_state)
        };
        self.counters.detaches += 1;

        self.emit_event(SessionFoundationEvent {
            kind: SessionFoundationEventKind::DeviceDetached,
            session_id,
            session_state,
            turn_id: None,
            device_id: Some(device_key.clone()),
            device_turn_sequence: None,
            detail: "device detached from session".to_string(),
        });

        Ok(SessionDetachResult {
            projection,
            remaining_devices,
            disposition,
        })
    }

    pub fn close_soft_closed_session(
        &mut self,
        session_id: SessionId,
    ) -> Result<SessionRuntimeProjection, SessionFoundationError> {
        let local_runtime_id = self.local_runtime_id().to_string();
        let logical_now_ms = self.logical_now_ms;
        let (projection, session_state) = {
            let record = self
                .sessions
                .get_mut(&session_id)
                .ok_or_else(|| SessionFoundationError::session_not_found(session_id))?;

            ensure_primary_owned_runtime(session_id, record, &local_runtime_id, logical_now_ms)?;
            ensure_no_active_writer(session_id, record)?;
            if record.state != SessionState::SoftClosed {
                return Err(SessionFoundationError::close_requires_soft_closed(
                    session_id,
                    record.state,
                ));
            }
            if !record.attached_devices.is_empty() {
                return Err(SessionFoundationError::close_requires_no_attachments(
                    session_id,
                ));
            }
            transition_record_state(record, SessionState::Closed, false)?;
            let session_state = record.state;
            let projection = record.projection(session_id, None, None, None);
            (projection, session_state)
        };

        self.emit_event(SessionFoundationEvent {
            kind: SessionFoundationEventKind::SessionClosed,
            session_id,
            session_state,
            turn_id: None,
            device_id: None,
            device_turn_sequence: None,
            detail: "soft-closed session closed deterministically".to_string(),
        });

        Ok(projection)
    }

    pub fn begin_turn(
        &mut self,
        session_id: SessionId,
        device_id: &DeviceId,
        device_turn_sequence: u64,
    ) -> Result<SessionTurnResolution, SessionFoundationError> {
        let local_runtime_id = self.local_runtime_id().to_string();
        self.begin_turn_internal(
            session_id,
            device_id,
            device_turn_sequence,
            &local_runtime_id,
        )
    }

    pub fn finish_turn(
        &mut self,
        permit: SessionTurnPermit,
        target_state: SessionState,
    ) -> Result<SessionTurnCommit, SessionFoundationError> {
        let (projection, previous_state, session_state) = {
            let record = self
                .sessions
                .get_mut(&permit.session_id)
                .ok_or_else(|| SessionFoundationError::session_not_found(permit.session_id))?;

            let Some(active_writer) = record.active_writer.as_ref() else {
                return Err(SessionFoundationError::single_writer_conflict(
                    permit.session_id,
                ));
            };

            if active_writer.turn_id != permit.turn_id
                || active_writer.device_id != permit.device_id
                || active_writer.device_turn_sequence != permit.device_turn_sequence
                || active_writer.runtime_id != permit.runtime_id
                || active_writer.access_class != permit.access_class
            {
                return Err(SessionFoundationError::integrity_violation(
                    permit.session_id,
                    "finish_turn permit did not match the active writer",
                ));
            }

            if !matches!(
                target_state,
                SessionState::Active | SessionState::SoftClosed | SessionState::Suspended
            ) {
                return Err(SessionFoundationError::invalid_transition(
                    record.state,
                    target_state,
                ));
            }

            record.device_timeline_map.insert(
                permit.device_id.clone(),
                DeviceTimelineRecord {
                    highest_seen_sequence: permit.device_turn_sequence,
                    last_turn_id: permit.turn_id,
                },
            );
            record.active_writer = None;

            let previous_state = record.state;
            transition_record_state(record, target_state, false)?;
            let session_state = record.state;
            let projection = record.projection(
                permit.session_id,
                Some(permit.turn_id),
                Some(permit.device_turn_sequence),
                permit.attach_outcome,
            );
            (projection, previous_state, session_state)
        };
        self.counters.turns_completed += 1;
        self.emit_event(SessionFoundationEvent {
            kind: SessionFoundationEventKind::TurnCompleted,
            session_id: permit.session_id,
            session_state,
            turn_id: Some(permit.turn_id),
            device_id: Some(permit.device_id.clone()),
            device_turn_sequence: Some(permit.device_turn_sequence),
            detail: "session turn committed and timeline advanced deterministically".to_string(),
        });

        Ok(SessionTurnCommit {
            projection,
            classification: DeviceTimelineClassification::New,
            previous_state,
        })
    }

    pub fn authorize_stage5_current_committed_turn(
        &self,
        commit: &SessionTurnCommit,
        permit: &SessionTurnPermit,
    ) -> Result<Stage5TurnAuthorityPacket, SessionFoundationError> {
        if commit.projection.session_id != permit.session_id
            || commit.projection.turn_id != Some(permit.turn_id)
            || commit.projection.device_turn_sequence != Some(permit.device_turn_sequence)
            || commit.classification != DeviceTimelineClassification::New
        {
            return Err(SessionFoundationError::integrity_violation(
                permit.session_id,
                "committed turn projection must match the session turn permit before Stage 5 authority",
            ));
        }

        let snapshot = self.session_snapshot(permit.session_id)?;
        if snapshot.session_state == SessionState::Closed {
            return Stage5TurnAuthorityPacket::quarantined(
                permit.session_id,
                Some(permit.turn_id),
                Some(permit.device_id.clone()),
                Some(permit.device_turn_sequence),
                snapshot.session_state,
                Stage5TurnAuthorityDisposition::ClosedSessionRejected,
            )
            .map_err(|_| {
                SessionFoundationError::integrity_violation(
                    permit.session_id,
                    "Stage 5 closed-session quarantine packet failed validation",
                )
            });
        }

        if snapshot.active_writer.is_some() || !snapshot.deferred_turns.is_empty() {
            return Stage5TurnAuthorityPacket::quarantined(
                permit.session_id,
                Some(permit.turn_id),
                Some(permit.device_id.clone()),
                Some(permit.device_turn_sequence),
                snapshot.session_state,
                Stage5TurnAuthorityDisposition::SupersededTurnQuarantined,
            )
            .map_err(|_| {
                SessionFoundationError::integrity_violation(
                    permit.session_id,
                    "Stage 5 superseded-turn quarantine packet failed validation",
                )
            });
        }

        let Some(timeline) = snapshot
            .device_timelines
            .iter()
            .find(|timeline| timeline.device_id == permit.device_id)
        else {
            return Err(SessionFoundationError::integrity_violation(
                permit.session_id,
                "committed turn device timeline was missing before Stage 5 authority",
            ));
        };

        if timeline.highest_seen_sequence != permit.device_turn_sequence
            || timeline.last_turn_id != permit.turn_id
        {
            return Stage5TurnAuthorityPacket::quarantined(
                permit.session_id,
                Some(permit.turn_id),
                Some(permit.device_id.clone()),
                Some(permit.device_turn_sequence),
                snapshot.session_state,
                Stage5TurnAuthorityDisposition::SupersededTurnQuarantined,
            )
            .map_err(|_| {
                SessionFoundationError::integrity_violation(
                    permit.session_id,
                    "Stage 5 stale-turn quarantine packet failed validation",
                )
            });
        }

        Stage5TurnAuthorityPacket::current_committed(
            permit.session_id,
            permit.turn_id,
            permit.device_id.clone(),
            permit.device_turn_sequence,
            snapshot.session_state,
        )
        .map_err(|_| {
            SessionFoundationError::integrity_violation(
                permit.session_id,
                "Stage 5 current committed turn authority packet failed validation",
            )
        })
    }

    pub fn renew_session_lease(
        &mut self,
        session_id: SessionId,
    ) -> Result<SessionCoordinationView, SessionFoundationError> {
        let local_runtime_id = self.local_runtime_id().to_string();
        self.renew_session_lease_internal(session_id, &local_runtime_id)
    }

    pub fn begin_ownership_transfer(
        &mut self,
        session_id: SessionId,
        target_runtime_id: &str,
    ) -> Result<SessionCoordinationView, SessionFoundationError> {
        let local_runtime_id = self.local_runtime_id().to_string();
        let logical_now_ms = self.logical_now_ms;
        let view = {
            let record = self
                .sessions
                .get_mut(&session_id)
                .ok_or_else(|| SessionFoundationError::session_not_found(session_id))?;

            ensure_primary_owned_runtime(session_id, record, &local_runtime_id, logical_now_ms)?;
            if target_runtime_id.is_empty()
                || target_runtime_id == record.ownership.owner_runtime_id
            {
                return Err(SessionFoundationError::ownership_transfer_invalid_target(
                    session_id,
                    target_runtime_id,
                ));
            }
            if record.pending_transfer_target.is_some() {
                return Err(SessionFoundationError::ownership_transfer_pending(
                    session_id,
                    record
                        .pending_transfer_target
                        .as_deref()
                        .unwrap_or(target_runtime_id),
                ));
            }
            if record.active_writer.is_some() || !record.deferred_turns.is_empty() {
                return Err(SessionFoundationError::ownership_transfer_drain_required(
                    session_id,
                ));
            }

            record.pending_transfer_target = Some(target_runtime_id.to_string());
            record.coordination_state = SessionCoordinationState::TransferPending;
            record.consistency_level = SessionConsistencyLevel::LeasedDistributed;
            coordination_view_from_record(record, session_id, logical_now_ms)
        };
        self.counters.ownership_transfer_requests += 1;
        self.emit_event(SessionFoundationEvent {
            kind: SessionFoundationEventKind::OwnershipTransferRequested,
            session_id,
            session_state: self.sessions[&session_id].state,
            turn_id: None,
            device_id: None,
            device_turn_sequence: None,
            detail: format!("ownership transfer requested for {}", target_runtime_id),
        });
        Ok(view)
    }

    pub fn acknowledge_ownership_transfer(
        &mut self,
        session_id: SessionId,
        target_runtime_id: &str,
    ) -> Result<SessionCoordinationView, SessionFoundationError> {
        let logical_now_ms = self.logical_now_ms;
        let lease_duration_ms = self.config.lease_duration_ms;
        let view = {
            let record = self
                .sessions
                .get_mut(&session_id)
                .ok_or_else(|| SessionFoundationError::session_not_found(session_id))?;

            let Some(expected_target_runtime_id) = record.pending_transfer_target.as_deref() else {
                return Err(SessionFoundationError::ownership_transfer_not_pending(
                    session_id,
                ));
            };
            if expected_target_runtime_id != target_runtime_id {
                return Err(SessionFoundationError::ownership_transfer_target_mismatch(
                    session_id,
                    expected_target_runtime_id,
                    target_runtime_id,
                ));
            }
            if record.active_writer.is_some() || !record.deferred_turns.is_empty() {
                return Err(SessionFoundationError::ownership_transfer_drain_required(
                    session_id,
                ));
            }

            record.pending_transfer_target = None;
            set_owner_runtime(
                record,
                session_id,
                target_runtime_id,
                logical_now_ms,
                lease_duration_ms,
            );
            record.coordination_state = SessionCoordinationState::PrimaryOwned;
            record.consistency_level = SessionConsistencyLevel::Strict;
            coordination_view_from_record(record, session_id, logical_now_ms)
        };
        self.counters.ownership_transfer_acknowledgements += 1;
        self.emit_event(SessionFoundationEvent {
            kind: SessionFoundationEventKind::OwnershipTransferAcknowledged,
            session_id,
            session_state: self.sessions[&session_id].state,
            turn_id: None,
            device_id: None,
            device_turn_sequence: None,
            detail: format!("ownership transfer acknowledged by {}", target_runtime_id),
        });
        Ok(view)
    }

    pub fn reject_ownership_transfer(
        &mut self,
        session_id: SessionId,
        target_runtime_id: &str,
        detail: &str,
    ) -> Result<SessionCoordinationView, SessionFoundationError> {
        let logical_now_ms = self.logical_now_ms;
        let lease_duration_ms = self.config.lease_duration_ms;
        let view = {
            let record = self
                .sessions
                .get_mut(&session_id)
                .ok_or_else(|| SessionFoundationError::session_not_found(session_id))?;

            let Some(expected_target_runtime_id) = record.pending_transfer_target.as_deref() else {
                return Err(SessionFoundationError::ownership_transfer_not_pending(
                    session_id,
                ));
            };
            if expected_target_runtime_id != target_runtime_id {
                return Err(SessionFoundationError::ownership_transfer_target_mismatch(
                    session_id,
                    expected_target_runtime_id,
                    target_runtime_id,
                ));
            }
            record.pending_transfer_target = None;
            record.coordination_state = SessionCoordinationState::PrimaryOwned;
            record.consistency_level = SessionConsistencyLevel::Strict;
            let owner_runtime_id = record.ownership.owner_runtime_id.clone();
            bump_lease(
                record,
                session_id,
                &owner_runtime_id,
                logical_now_ms,
                lease_duration_ms,
            );
            coordination_view_from_record(record, session_id, logical_now_ms)
        };
        self.counters.ownership_transfer_rejections += 1;
        self.emit_event(SessionFoundationEvent {
            kind: SessionFoundationEventKind::OwnershipTransferRejected,
            session_id,
            session_state: self.sessions[&session_id].state,
            turn_id: None,
            device_id: None,
            device_turn_sequence: None,
            detail: format!(
                "ownership transfer rejected by {}: {}",
                target_runtime_id, detail
            ),
        });
        Ok(view)
    }

    pub fn begin_failover_recovery(
        &mut self,
        session_id: SessionId,
        recovering_runtime_id: &str,
    ) -> Result<SessionCoordinationView, SessionFoundationError> {
        let logical_now_ms = self.logical_now_ms;
        let lease_duration_ms = self.config.lease_duration_ms;
        let view = {
            let record = self
                .sessions
                .get_mut(&session_id)
                .ok_or_else(|| SessionFoundationError::session_not_found(session_id))?;

            if recovering_runtime_id.is_empty() {
                return Err(SessionFoundationError::ownership_transfer_invalid_target(
                    session_id,
                    recovering_runtime_id,
                ));
            }
            if effective_coordination_state(record, logical_now_ms)
                != SessionCoordinationState::OwnershipUncertain
            {
                return Err(SessionFoundationError::ownership_uncertain(
                    session_id,
                    "failover recovery requires ownership uncertainty first",
                ));
            }
            if record.active_writer.is_some() || !record.deferred_turns.is_empty() {
                return Err(SessionFoundationError::ownership_transfer_drain_required(
                    session_id,
                ));
            }
            record.pending_transfer_target = None;
            set_owner_runtime(
                record,
                session_id,
                recovering_runtime_id,
                logical_now_ms,
                lease_duration_ms,
            );
            record.coordination_state = SessionCoordinationState::FailoverRecovering;
            record.consistency_level = SessionConsistencyLevel::DegradedRecovery;
            coordination_view_from_record(record, session_id, logical_now_ms)
        };
        self.counters.failover_recoveries_started += 1;
        self.emit_event(SessionFoundationEvent {
            kind: SessionFoundationEventKind::FailoverRecoveryStarted,
            session_id,
            session_state: self.sessions[&session_id].state,
            turn_id: None,
            device_id: None,
            device_turn_sequence: None,
            detail: format!("failover recovery started by {}", recovering_runtime_id),
        });
        Ok(view)
    }

    pub fn complete_failover_recovery(
        &mut self,
        session_id: SessionId,
        recovering_runtime_id: &str,
    ) -> Result<SessionCoordinationView, SessionFoundationError> {
        let logical_now_ms = self.logical_now_ms;
        let lease_duration_ms = self.config.lease_duration_ms;
        let view = {
            let record = self
                .sessions
                .get_mut(&session_id)
                .ok_or_else(|| SessionFoundationError::session_not_found(session_id))?;

            if record.coordination_state != SessionCoordinationState::FailoverRecovering {
                return Err(SessionFoundationError::ownership_uncertain(
                    session_id,
                    "failover recovery has not started",
                ));
            }
            ensure_owner_runtime(session_id, record, recovering_runtime_id)?;
            if record.active_writer.is_some() || !record.deferred_turns.is_empty() {
                return Err(SessionFoundationError::ownership_transfer_drain_required(
                    session_id,
                ));
            }
            record.coordination_state = SessionCoordinationState::PrimaryOwned;
            record.consistency_level = SessionConsistencyLevel::Strict;
            bump_lease(
                record,
                session_id,
                recovering_runtime_id,
                logical_now_ms,
                lease_duration_ms,
            );
            coordination_view_from_record(record, session_id, logical_now_ms)
        };
        self.counters.failover_recoveries_completed += 1;
        self.emit_event(SessionFoundationEvent {
            kind: SessionFoundationEventKind::FailoverRecoveryCompleted,
            session_id,
            session_state: self.sessions[&session_id].state,
            turn_id: None,
            device_id: None,
            device_turn_sequence: None,
            detail: format!("failover recovery completed by {}", recovering_runtime_id),
        });
        Ok(view)
    }

    pub fn integrity_check(&mut self, session_id: SessionId) -> Result<(), SessionFoundationError> {
        let record = self
            .sessions
            .get(&session_id)
            .cloned()
            .ok_or_else(|| SessionFoundationError::session_not_found(session_id))?;

        if record.next_turn_id == 0 {
            return self.record_integrity_violation(
                session_id,
                "next_turn_id must remain greater than zero",
            );
        }

        if record.ownership.owner_runtime_id.is_empty() {
            return self.record_integrity_violation(
                session_id,
                "session owner runtime identity must not be empty",
            );
        }
        if record.ownership.lease_generation == 0 || record.ownership.lease_token.is_empty() {
            return self.record_integrity_violation(
                session_id,
                "lease generation and token must remain initialized",
            );
        }

        if record.state == SessionState::Closed && !record.attached_devices.is_empty() {
            return self.record_integrity_violation(
                session_id,
                "closed sessions must not retain attached devices",
            );
        }

        if matches!(record.state, SessionState::Closed | SessionState::Suspended)
            && record.active_writer.is_some()
        {
            return self.record_integrity_violation(
                session_id,
                "closed or suspended sessions must not retain an active writer",
            );
        }

        if let Some(writer) = &record.active_writer {
            if writer.turn_id.0 == 0 || writer.turn_id.0 >= record.next_turn_id {
                return self.record_integrity_violation(
                    session_id,
                    "active writer turn_id must be > 0 and below next_turn_id",
                );
            }
            if writer.device_turn_sequence == 0 {
                return self.record_integrity_violation(
                    session_id,
                    "active writer device_turn_sequence must be > 0",
                );
            }
            if !record.attached_devices.contains(&writer.device_id) {
                return self.record_integrity_violation(
                    session_id,
                    "active writer device must remain attached to the session",
                );
            }
        }

        if record.attached_devices.len() != record.device_access_classes.len() {
            return self.record_integrity_violation(
                session_id,
                "attached device set and access-class map must stay aligned",
            );
        }

        if record.pending_transfer_target.is_some()
            != (record.coordination_state == SessionCoordinationState::TransferPending)
        {
            return self.record_integrity_violation(
                session_id,
                "pending transfer target must exist only during TRANSFER_PENDING",
            );
        }

        match record.coordination_state {
            SessionCoordinationState::PrimaryOwned => {
                if record.consistency_level != SessionConsistencyLevel::Strict {
                    return self.record_integrity_violation(
                        session_id,
                        "PRIMARY_OWNED sessions must expose STRICT consistency",
                    );
                }
            }
            SessionCoordinationState::TransferPending => {
                if record.consistency_level != SessionConsistencyLevel::LeasedDistributed {
                    return self.record_integrity_violation(
                        session_id,
                        "TRANSFER_PENDING sessions must expose LEASED_DISTRIBUTED consistency",
                    );
                }
                if record.active_writer.is_some() || !record.deferred_turns.is_empty() {
                    return self.record_integrity_violation(
                        session_id,
                        "TRANSFER_PENDING sessions must be drained before handoff",
                    );
                }
            }
            SessionCoordinationState::FailoverRecovering
            | SessionCoordinationState::OwnershipUncertain => {
                if record.consistency_level != SessionConsistencyLevel::DegradedRecovery {
                    return self.record_integrity_violation(
                        session_id,
                        "degraded coordination postures must expose DEGRADED_RECOVERY",
                    );
                }
            }
        }

        let mut primary_interactor_count = 0usize;
        for (device_id, access_class) in &record.device_access_classes {
            if !record.attached_devices.contains(device_id) {
                return self.record_integrity_violation(
                    session_id,
                    "device access-class entries must remain attached",
                );
            }
            if *access_class == SessionAccessClass::PrimaryInteractor {
                primary_interactor_count += 1;
            }
            if !access_class_allows_attach(
                *access_class,
                record.coordination_state,
                record.state,
                false,
            ) {
                return self.record_integrity_violation(
                    session_id,
                    "access-class and coordination posture combination is impossible",
                );
            }
        }
        if primary_interactor_count > 1 {
            return self.record_integrity_violation(
                session_id,
                "only one PRIMARY_INTERACTOR may exist at a time",
            );
        }

        for (device_id, timeline) in &record.device_timeline_map {
            if timeline.highest_seen_sequence == 0 {
                return self.record_integrity_violation(
                    session_id,
                    &format!("device {device_id} recorded impossible sequence 0"),
                );
            }
            if timeline.last_turn_id.0 == 0 || timeline.last_turn_id.0 >= record.next_turn_id {
                return self.record_integrity_violation(
                    session_id,
                    &format!("device {device_id} has impossible last_turn_id history"),
                );
            }
        }

        for ((device_id, device_turn_sequence), deferred) in &record.deferred_turns {
            if *device_turn_sequence == 0 {
                return self.record_integrity_violation(
                    session_id,
                    "deferred turn device_turn_sequence must be greater than zero",
                );
            }
            if !record.attached_devices.contains(device_id) {
                return self.record_integrity_violation(
                    session_id,
                    "deferred turns must belong to attached devices",
                );
            }
            if !access_class_allows_turn_submission(
                deferred.access_class,
                record.coordination_state,
            ) {
                return self.record_integrity_violation(
                    session_id,
                    "deferred turns must preserve a lawful submitting access class",
                );
            }
        }

        Ok(())
    }

    fn attach_session_internal(
        &mut self,
        session_id: SessionId,
        device_id: DeviceId,
        access_class: SessionAccessClass,
        claimed_device_turn_sequence: Option<u64>,
        runtime_id: &str,
    ) -> Result<SessionAttachResult, SessionFoundationError> {
        let device_key = device_id.as_str().to_string();
        let logical_now_ms = self.logical_now_ms;
        let max_attached_devices = self.config.max_attached_devices;
        let (projection, attached_devices, session_state, attach_outcome) = {
            let record = self
                .sessions
                .get_mut(&session_id)
                .ok_or_else(|| SessionFoundationError::session_not_found(session_id))?;

            let effective_coordination =
                ensure_primary_owned_runtime(session_id, record, runtime_id, logical_now_ms)?;
            ensure_no_active_writer(session_id, record)?;
            if record.state == SessionState::Closed {
                return Err(SessionFoundationError::attach_not_allowed(
                    session_id,
                    record.state,
                ));
            }
            if let Some(sequence) = claimed_device_turn_sequence {
                validate_device_turn_sequence(&device_key, sequence)?;
                if let Some(timeline) = record.device_timeline_map.get(&device_key) {
                    if sequence < timeline.highest_seen_sequence {
                        return Err(SessionFoundationError::stale_attach(
                            session_id,
                            &device_key,
                            sequence,
                            timeline.highest_seen_sequence,
                        ));
                    }
                }
            }

            let attached =
                attach_device_to_record(session_id, record, &device_key, max_attached_devices)?;
            assign_access_class_to_record(
                session_id,
                record,
                &device_key,
                access_class,
                effective_coordination,
                false,
            )?;
            let attach_outcome = if attached {
                SessionAttachOutcome::ExistingSessionAttached
            } else {
                SessionAttachOutcome::ExistingSessionReused
            };
            let session_state = record.state;
            let attached_devices = record.attached_devices();
            let projection = record.projection(session_id, None, None, Some(attach_outcome));
            (projection, attached_devices, session_state, attach_outcome)
        };

        if attach_outcome == SessionAttachOutcome::ExistingSessionAttached {
            self.counters.devices_attached += 1;
        }
        self.emit_event(SessionFoundationEvent {
            kind: SessionFoundationEventKind::DeviceAttached,
            session_id,
            session_state,
            turn_id: None,
            device_id: Some(device_key),
            device_turn_sequence: claimed_device_turn_sequence,
            detail: format!("device attached with {:?}", access_class),
        });

        Ok(SessionAttachResult {
            projection,
            attached_devices,
        })
    }

    fn resume_session_internal(
        &mut self,
        session_id: SessionId,
        device_id: DeviceId,
        runtime_id: &str,
    ) -> Result<SessionResumeResult, SessionFoundationError> {
        let device_key = device_id.as_str().to_string();
        let logical_now_ms = self.logical_now_ms;
        let max_attached_devices = self.config.max_attached_devices;
        let (projection, attached_devices, session_state, attach_outcome) = {
            let record = self
                .sessions
                .get_mut(&session_id)
                .ok_or_else(|| SessionFoundationError::session_not_found(session_id))?;

            let effective_coordination =
                ensure_primary_owned_runtime(session_id, record, runtime_id, logical_now_ms)?;
            ensure_no_active_writer(session_id, record)?;
            if matches!(record.state, SessionState::Closed | SessionState::Suspended) {
                return Err(SessionFoundationError::resume_not_recoverable(
                    session_id,
                    record.state,
                ));
            }
            if let Some(active_primary_device) = primary_interactor_device(record) {
                if active_primary_device != device_key {
                    return Err(SessionFoundationError::conflicting_resume(
                        session_id,
                        &device_key,
                        active_primary_device,
                    ));
                }
            }

            let attached =
                attach_device_to_record(session_id, record, &device_key, max_attached_devices)?;
            assign_access_class_to_record(
                session_id,
                record,
                &device_key,
                SessionAccessClass::PrimaryInteractor,
                effective_coordination,
                false,
            )?;
            let attach_outcome = if attached {
                SessionAttachOutcome::ExistingSessionAttached
            } else {
                SessionAttachOutcome::ExistingSessionReused
            };
            transition_record_state(record, SessionState::Active, false)?;
            let session_state = record.state;
            let projection = record.projection(session_id, None, None, Some(attach_outcome));
            let attached_devices = record.attached_devices();
            (projection, attached_devices, session_state, attach_outcome)
        };
        if attach_outcome == SessionAttachOutcome::ExistingSessionAttached {
            self.counters.devices_attached += 1;
        }
        self.counters.resumes += 1;
        self.emit_event(SessionFoundationEvent {
            kind: SessionFoundationEventKind::SessionResumed,
            session_id,
            session_state,
            turn_id: None,
            device_id: Some(device_key),
            device_turn_sequence: None,
            detail: "session resumed into Active state with PRIMARY_INTERACTOR".to_string(),
        });

        Ok(SessionResumeResult {
            projection,
            attached_devices,
        })
    }

    fn recover_session_internal(
        &mut self,
        session_id: SessionId,
        device_id: DeviceId,
        runtime_id: &str,
    ) -> Result<SessionRecoverResult, SessionFoundationError> {
        let device_key = device_id.as_str().to_string();
        let logical_now_ms = self.logical_now_ms;
        let max_attached_devices = self.config.max_attached_devices;
        let (projection, attached_devices, session_state, attach_outcome, access_class) = {
            let record = self
                .sessions
                .get_mut(&session_id)
                .ok_or_else(|| SessionFoundationError::session_not_found(session_id))?;

            ensure_owner_runtime(session_id, record, runtime_id)?;
            ensure_no_active_writer(session_id, record)?;
            if record.state != SessionState::Suspended {
                return Err(SessionFoundationError::recover_not_suspended(
                    session_id,
                    record.state,
                ));
            }
            let effective_coordination = effective_coordination_state(record, logical_now_ms);
            if matches!(
                effective_coordination,
                SessionCoordinationState::OwnershipUncertain
                    | SessionCoordinationState::TransferPending
            ) {
                return Err(SessionFoundationError::ownership_uncertain(
                    session_id,
                    "recover requires an explicit primary or failover-recovering owner",
                ));
            }
            let access_class =
                if effective_coordination == SessionCoordinationState::FailoverRecovering {
                    SessionAccessClass::RecoveryAttach
                } else if let Some(active_primary_device) = primary_interactor_device(record) {
                    if active_primary_device == device_key {
                        SessionAccessClass::PrimaryInteractor
                    } else {
                        SessionAccessClass::LimitedAttach
                    }
                } else {
                    SessionAccessClass::PrimaryInteractor
                };
            let attached =
                attach_device_to_record(session_id, record, &device_key, max_attached_devices)?;
            assign_access_class_to_record(
                session_id,
                record,
                &device_key,
                access_class,
                effective_coordination,
                false,
            )?;
            let attach_outcome = if attached {
                SessionAttachOutcome::ExistingSessionAttached
            } else {
                SessionAttachOutcome::ExistingSessionReused
            };
            transition_record_state(record, SessionState::Open, false)?;
            let session_state = record.state;
            let projection = record.projection(session_id, None, None, Some(attach_outcome));
            let attached_devices = record.attached_devices();
            (
                projection,
                attached_devices,
                session_state,
                attach_outcome,
                access_class,
            )
        };
        if attach_outcome == SessionAttachOutcome::ExistingSessionAttached {
            self.counters.devices_attached += 1;
        }
        self.counters.recovers += 1;
        self.emit_event(SessionFoundationEvent {
            kind: SessionFoundationEventKind::SessionRecovered,
            session_id,
            session_state,
            turn_id: None,
            device_id: Some(device_key),
            device_turn_sequence: None,
            detail: format!("session recovered into Open state with {:?}", access_class),
        });

        Ok(SessionRecoverResult {
            projection,
            attached_devices,
        })
    }

    fn begin_turn_internal(
        &mut self,
        session_id: SessionId,
        device_id: &DeviceId,
        device_turn_sequence: u64,
        runtime_id: &str,
    ) -> Result<SessionTurnResolution, SessionFoundationError> {
        enum BeginTurnOutcome {
            Stale {
                session_state: SessionState,
                last_turn_id: TurnId,
                highest_seen_sequence: u64,
            },
            Retry {
                projection: SessionRuntimeProjection,
                session_state: SessionState,
                last_turn_id: TurnId,
            },
            Deferred {
                session_state: SessionState,
                pending_turn_count: usize,
            },
            Started {
                permit: SessionTurnPermit,
                session_state: SessionState,
            },
        }

        validate_device_turn_sequence(device_id.as_str(), device_turn_sequence)?;
        let device_key = device_id.as_str().to_string();
        let runtime_id_owned = runtime_id.to_string();
        let logical_now_ms = self.logical_now_ms;
        let max_pending_turns = self.config.max_pending_turns;

        let outcome = {
            let record = self
                .sessions
                .get_mut(&session_id)
                .ok_or_else(|| SessionFoundationError::session_not_found(session_id))?;

            let effective_coordination =
                ensure_primary_owned_runtime(session_id, record, runtime_id, logical_now_ms)?;
            ensure_device_attached(session_id, record, &device_key)?;
            match record.state {
                SessionState::Open | SessionState::Active | SessionState::SoftClosed => {}
                _ => {
                    return Err(SessionFoundationError::invalid_transition(
                        record.state,
                        SessionState::Active,
                    ));
                }
            }

            let access_class = access_class_for_device(session_id, record, &device_key)?;
            if !access_class_allows_turn_submission(access_class, effective_coordination) {
                return Err(SessionFoundationError::invalid_access_class(
                    session_id,
                    access_class,
                    effective_coordination,
                    record.state,
                ));
            }

            if let Some(active_writer) = record.active_writer.as_ref() {
                if active_writer.device_id == device_key
                    && active_writer.device_turn_sequence == device_turn_sequence
                {
                    BeginTurnOutcome::Deferred {
                        session_state: record.state,
                        pending_turn_count: record.deferred_turns.len(),
                    }
                } else {
                    let pending_key = (device_key.clone(), device_turn_sequence);
                    if !record.deferred_turns.contains_key(&pending_key) {
                        if record.deferred_turns.len() >= max_pending_turns {
                            return Err(SessionFoundationError::backpressure_exceeded(
                                session_id,
                                max_pending_turns,
                                &device_key,
                                device_turn_sequence,
                            ));
                        }
                        record.deferred_turns.insert(
                            pending_key,
                            DeferredTurnRecord {
                                runtime_id: runtime_id_owned.clone(),
                                access_class,
                            },
                        );
                    }
                    BeginTurnOutcome::Deferred {
                        session_state: record.state,
                        pending_turn_count: record.deferred_turns.len(),
                    }
                }
            } else {
                record
                    .deferred_turns
                    .remove(&(device_key.clone(), device_turn_sequence));
                if let Some(timeline) = record.device_timeline_map.get(&device_key) {
                    if device_turn_sequence < timeline.highest_seen_sequence {
                        BeginTurnOutcome::Stale {
                            session_state: record.state,
                            last_turn_id: timeline.last_turn_id,
                            highest_seen_sequence: timeline.highest_seen_sequence,
                        }
                    } else if device_turn_sequence == timeline.highest_seen_sequence {
                        let projection = record.projection(
                            session_id,
                            Some(timeline.last_turn_id),
                            Some(device_turn_sequence),
                            Some(SessionAttachOutcome::RetryReusedResult),
                        );
                        BeginTurnOutcome::Retry {
                            projection,
                            session_state: record.state,
                            last_turn_id: timeline.last_turn_id,
                        }
                    } else {
                        let previous_state = record.state;
                        transition_record_state(record, SessionState::Active, false)?;
                        let turn_id = allocate_turn_id(record);
                        record.active_writer = Some(ActiveTurnMutation {
                            turn_id,
                            device_id: device_key.clone(),
                            device_turn_sequence,
                            runtime_id: runtime_id_owned.clone(),
                            access_class,
                        });
                        BeginTurnOutcome::Started {
                            permit: SessionTurnPermit {
                                session_id,
                                turn_id,
                                device_id: device_key.clone(),
                                device_turn_sequence,
                                previous_state,
                                attach_outcome: None,
                                runtime_id: runtime_id_owned.clone(),
                                access_class,
                            },
                            session_state: record.state,
                        }
                    }
                } else {
                    let previous_state = record.state;
                    transition_record_state(record, SessionState::Active, false)?;
                    let turn_id = allocate_turn_id(record);
                    record.active_writer = Some(ActiveTurnMutation {
                        turn_id,
                        device_id: device_key.clone(),
                        device_turn_sequence,
                        runtime_id: runtime_id_owned.clone(),
                        access_class,
                    });
                    BeginTurnOutcome::Started {
                        permit: SessionTurnPermit {
                            session_id,
                            turn_id,
                            device_id: device_key.clone(),
                            device_turn_sequence,
                            previous_state,
                            attach_outcome: None,
                            runtime_id: runtime_id_owned,
                            access_class,
                        },
                        session_state: record.state,
                    }
                }
            }
        };

        match outcome {
            BeginTurnOutcome::Stale {
                session_state,
                last_turn_id,
                highest_seen_sequence,
            } => {
                self.counters.stale_rejections += 1;
                self.emit_event(SessionFoundationEvent {
                    kind: SessionFoundationEventKind::StaleRejected,
                    session_id,
                    session_state,
                    turn_id: Some(last_turn_id),
                    device_id: Some(device_key),
                    device_turn_sequence: Some(device_turn_sequence),
                    detail: "stale device turn rejected without mutating session state".to_string(),
                });
                Err(SessionFoundationError::stale_device_turn(
                    device_id.as_str(),
                    device_turn_sequence,
                    highest_seen_sequence,
                ))
            }
            BeginTurnOutcome::Retry {
                projection,
                session_state,
                last_turn_id,
            } => {
                self.counters.retries_reused += 1;
                self.emit_event(SessionFoundationEvent {
                    kind: SessionFoundationEventKind::RetryReused,
                    session_id,
                    session_state,
                    turn_id: Some(last_turn_id),
                    device_id: Some(device_key),
                    device_turn_sequence: Some(device_turn_sequence),
                    detail: "retry reused prior turn result without mutating session state"
                        .to_string(),
                });
                Ok(SessionTurnResolution::Retry(projection))
            }
            BeginTurnOutcome::Deferred {
                session_state,
                pending_turn_count,
            } => {
                self.counters.turn_deferrals += 1;
                self.emit_event(SessionFoundationEvent {
                    kind: SessionFoundationEventKind::TurnDeferred,
                    session_id,
                    session_state,
                    turn_id: None,
                    device_id: Some(device_key.clone()),
                    device_turn_sequence: Some(device_turn_sequence),
                    detail:
                        "turn deferred while active writer drained or single-writer gate cleared"
                            .to_string(),
                });
                Ok(SessionTurnResolution::Deferred(SessionTurnDeferred {
                    session_id,
                    device_id: device_key,
                    device_turn_sequence,
                    pending_turn_count,
                }))
            }
            BeginTurnOutcome::Started {
                permit,
                session_state,
            } => {
                self.counters.turns_started += 1;
                self.emit_event(SessionFoundationEvent {
                    kind: SessionFoundationEventKind::TurnStarted,
                    session_id,
                    session_state,
                    turn_id: Some(permit.turn_id),
                    device_id: Some(device_key),
                    device_turn_sequence: Some(device_turn_sequence),
                    detail: "new session turn admitted into the single-writer gate".to_string(),
                });
                Ok(SessionTurnResolution::Started(permit))
            }
        }
    }

    fn renew_session_lease_internal(
        &mut self,
        session_id: SessionId,
        runtime_id: &str,
    ) -> Result<SessionCoordinationView, SessionFoundationError> {
        let logical_now_ms = self.logical_now_ms;
        let lease_duration_ms = self.config.lease_duration_ms;
        let view = {
            let record = self
                .sessions
                .get_mut(&session_id)
                .ok_or_else(|| SessionFoundationError::session_not_found(session_id))?;

            ensure_primary_owned_runtime(session_id, record, runtime_id, logical_now_ms)?;
            bump_lease(
                record,
                session_id,
                runtime_id,
                logical_now_ms,
                lease_duration_ms,
            );
            coordination_view_from_record(record, session_id, logical_now_ms)
        };
        self.counters.lease_renewals += 1;
        self.emit_event(SessionFoundationEvent {
            kind: SessionFoundationEventKind::LeaseRenewed,
            session_id,
            session_state: self.sessions[&session_id].state,
            turn_id: None,
            device_id: None,
            device_turn_sequence: None,
            detail: format!("lease renewed by {}", runtime_id),
        });
        Ok(view)
    }

    fn local_runtime_id(&self) -> &str {
        &self.config.runtime_identity
    }

    fn allocate_session_id(&mut self) -> SessionId {
        let session_id = SessionId(self.next_session_id);
        self.next_session_id = self.next_session_id.saturating_add(1);
        session_id
    }

    fn emit_event(&mut self, event: SessionFoundationEvent) {
        if self.events.len() == self.config.max_events {
            self.events.remove(0);
        }
        self.events.push(event);
    }

    fn record_integrity_violation<T>(
        &mut self,
        session_id: SessionId,
        detail: &str,
    ) -> Result<T, SessionFoundationError> {
        self.counters.integrity_violations += 1;
        self.emit_event(SessionFoundationEvent {
            kind: SessionFoundationEventKind::IntegrityViolation,
            session_id,
            session_state: self
                .sessions
                .get(&session_id)
                .map(|record| record.state)
                .unwrap_or(SessionState::Closed),
            turn_id: None,
            device_id: None,
            device_turn_sequence: None,
            detail: detail.to_string(),
        });
        Err(SessionFoundationError::integrity_violation(
            session_id, detail,
        ))
    }
}

fn validate_device_turn_sequence(
    device_id: &str,
    device_turn_sequence: u64,
) -> Result<(), SessionFoundationError> {
    if device_turn_sequence == 0 {
        return Err(SessionFoundationError::invalid_device_sequence(
            device_id,
            device_turn_sequence,
        ));
    }
    Ok(())
}

fn validate_stage5_ascii_token(
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

fn allocate_turn_id(record: &mut SessionRecord) -> TurnId {
    let turn_id = TurnId(record.next_turn_id);
    record.next_turn_id = record.next_turn_id.saturating_add(1);
    turn_id
}

fn attach_device_to_record(
    session_id: SessionId,
    record: &mut SessionRecord,
    device_id: &str,
    max_attached_devices: usize,
) -> Result<bool, SessionFoundationError> {
    if record.attached_devices.contains(device_id) {
        return Ok(false);
    }
    if record.attached_devices.len() >= max_attached_devices {
        return Err(SessionFoundationError::too_many_devices(
            session_id,
            max_attached_devices,
        ));
    }
    record.attached_devices.insert(device_id.to_string());
    Ok(true)
}

fn assign_access_class_to_record(
    session_id: SessionId,
    record: &mut SessionRecord,
    device_id: &str,
    access_class: SessionAccessClass,
    coordination_state: SessionCoordinationState,
    allow_closed_attach: bool,
) -> Result<(), SessionFoundationError> {
    if !access_class_allows_attach(
        access_class,
        coordination_state,
        record.state,
        allow_closed_attach,
    ) {
        return Err(SessionFoundationError::invalid_access_class(
            session_id,
            access_class,
            coordination_state,
            record.state,
        ));
    }
    if access_class == SessionAccessClass::PrimaryInteractor {
        if let Some(active_primary_device) = primary_interactor_device(record) {
            if active_primary_device != device_id {
                return Err(SessionFoundationError::duplicate_device_claim(
                    session_id,
                    device_id,
                    active_primary_device,
                ));
            }
        }
    }
    record
        .device_access_classes
        .insert(device_id.to_string(), access_class);
    Ok(())
}

fn access_class_for_device(
    session_id: SessionId,
    record: &SessionRecord,
    device_id: &str,
) -> Result<SessionAccessClass, SessionFoundationError> {
    record
        .device_access_classes
        .get(device_id)
        .copied()
        .ok_or_else(|| {
            SessionFoundationError::integrity_violation(
                session_id,
                &format!("device {} is attached without an access class", device_id),
            )
        })
}

fn ensure_device_attached(
    session_id: SessionId,
    record: &SessionRecord,
    device_id: &str,
) -> Result<(), SessionFoundationError> {
    if record.attached_devices.contains(device_id) {
        Ok(())
    } else {
        Err(SessionFoundationError::device_not_attached(
            session_id, device_id,
        ))
    }
}

fn ensure_no_active_writer(
    session_id: SessionId,
    record: &SessionRecord,
) -> Result<(), SessionFoundationError> {
    if record.active_writer.is_some() {
        Err(SessionFoundationError::single_writer_conflict(session_id))
    } else {
        Ok(())
    }
}

fn ensure_owner_runtime(
    session_id: SessionId,
    record: &SessionRecord,
    runtime_id: &str,
) -> Result<(), SessionFoundationError> {
    if record.ownership.owner_runtime_id == runtime_id {
        Ok(())
    } else {
        Err(SessionFoundationError::not_session_owner(
            session_id,
            runtime_id,
            &record.ownership.owner_runtime_id,
        ))
    }
}

fn ensure_primary_owned_runtime(
    session_id: SessionId,
    record: &SessionRecord,
    runtime_id: &str,
    logical_now_ms: i64,
) -> Result<SessionCoordinationState, SessionFoundationError> {
    ensure_owner_runtime(session_id, record, runtime_id)?;
    let effective_coordination = effective_coordination_state(record, logical_now_ms);
    match effective_coordination {
        SessionCoordinationState::PrimaryOwned => Ok(effective_coordination),
        SessionCoordinationState::TransferPending => {
            Err(SessionFoundationError::ownership_transfer_pending(
                session_id,
                record
                    .pending_transfer_target
                    .as_deref()
                    .unwrap_or("unknown_target"),
            ))
        }
        SessionCoordinationState::OwnershipUncertain => {
            if logical_now_ms >= record.ownership.lease_expires_at_ms {
                Err(SessionFoundationError::lease_expired(
                    session_id, runtime_id,
                ))
            } else {
                Err(SessionFoundationError::ownership_uncertain(
                    session_id,
                    "ownership posture is degraded and cannot admit primary mutation",
                ))
            }
        }
        SessionCoordinationState::FailoverRecovering => {
            Err(SessionFoundationError::ownership_uncertain(
                session_id,
                "failover recovery must complete before primary mutation resumes",
            ))
        }
    }
}

fn transition_record_state(
    record: &mut SessionRecord,
    target_state: SessionState,
    allow_open_bypass: bool,
) -> Result<(), SessionFoundationError> {
    if record.state == target_state {
        return Ok(());
    }
    if !is_transition_allowed(record.state, target_state, allow_open_bypass) {
        return Err(SessionFoundationError::invalid_transition(
            record.state,
            target_state,
        ));
    }
    record.state = target_state;
    Ok(())
}

fn is_transition_allowed(from: SessionState, to: SessionState, allow_open_bypass: bool) -> bool {
    match (from, to) {
        (SessionState::Closed, SessionState::Open) => true,
        (SessionState::Closed, SessionState::Active) => allow_open_bypass,
        (SessionState::Open, SessionState::Active) => true,
        (SessionState::Active, SessionState::Active) => true,
        (SessionState::Active, SessionState::SoftClosed) => true,
        (SessionState::SoftClosed, SessionState::Active) => true,
        (SessionState::SoftClosed, SessionState::Closed) => true,
        (SessionState::Suspended, SessionState::Open) => true,
        (_, SessionState::Suspended) => true,
        _ => false,
    }
}

fn primary_interactor_device(record: &SessionRecord) -> Option<&str> {
    record
        .device_access_classes
        .iter()
        .find_map(|(device_id, access_class)| {
            (*access_class == SessionAccessClass::PrimaryInteractor).then_some(device_id.as_str())
        })
}

fn access_class_allows_attach(
    access_class: SessionAccessClass,
    coordination_state: SessionCoordinationState,
    session_state: SessionState,
    allow_closed_attach: bool,
) -> bool {
    if !allow_closed_attach && session_state == SessionState::Closed {
        return false;
    }
    match access_class {
        SessionAccessClass::PrimaryInteractor => {
            coordination_state == SessionCoordinationState::PrimaryOwned
        }
        SessionAccessClass::SecondaryViewer | SessionAccessClass::LimitedAttach => {
            coordination_state != SessionCoordinationState::OwnershipUncertain
        }
        SessionAccessClass::RecoveryAttach => {
            coordination_state == SessionCoordinationState::FailoverRecovering
                || session_state == SessionState::Suspended
        }
    }
}

fn access_class_allows_turn_submission(
    access_class: SessionAccessClass,
    coordination_state: SessionCoordinationState,
) -> bool {
    match access_class {
        SessionAccessClass::PrimaryInteractor => {
            coordination_state == SessionCoordinationState::PrimaryOwned
        }
        SessionAccessClass::RecoveryAttach => {
            coordination_state == SessionCoordinationState::FailoverRecovering
        }
        SessionAccessClass::SecondaryViewer | SessionAccessClass::LimitedAttach => false,
    }
}

fn effective_coordination_state(
    record: &SessionRecord,
    logical_now_ms: i64,
) -> SessionCoordinationState {
    if record.coordination_state == SessionCoordinationState::PrimaryOwned
        && logical_now_ms >= record.ownership.lease_expires_at_ms
    {
        SessionCoordinationState::OwnershipUncertain
    } else {
        record.coordination_state
    }
}

fn effective_consistency_level(
    record: &SessionRecord,
    logical_now_ms: i64,
) -> SessionConsistencyLevel {
    if effective_coordination_state(record, logical_now_ms)
        == SessionCoordinationState::OwnershipUncertain
        && record.coordination_state == SessionCoordinationState::PrimaryOwned
    {
        SessionConsistencyLevel::DegradedRecovery
    } else {
        record.consistency_level
    }
}

fn snapshot_with_effective_posture(
    record: &SessionRecord,
    session_id: SessionId,
    logical_now_ms: i64,
) -> SessionFoundationSnapshot {
    let mut snapshot = record.snapshot(session_id);
    snapshot.coordination_state = effective_coordination_state(record, logical_now_ms);
    snapshot.consistency_level = effective_consistency_level(record, logical_now_ms);
    snapshot
}

fn coordination_view_from_record(
    record: &SessionRecord,
    session_id: SessionId,
    logical_now_ms: i64,
) -> SessionCoordinationView {
    SessionCoordinationView {
        session_id,
        coordination_state: effective_coordination_state(record, logical_now_ms),
        consistency_level: effective_consistency_level(record, logical_now_ms),
        owner_runtime_id: record.ownership.owner_runtime_id.clone(),
        pending_transfer_target: record.pending_transfer_target.clone(),
        lease_token: record.ownership.lease_token.clone(),
        lease_expires_at_ms: record.ownership.lease_expires_at_ms,
    }
}

fn lease_token(session_id: SessionId, runtime_id: &str, lease_generation: u64) -> String {
    format!(
        "session-lease-{}-{}-{}",
        session_id.0, runtime_id, lease_generation
    )
}

fn bump_lease(
    record: &mut SessionRecord,
    session_id: SessionId,
    runtime_id: &str,
    logical_now_ms: i64,
    lease_duration_ms: i64,
) {
    record.ownership.lease_generation = record.ownership.lease_generation.saturating_add(1);
    record.ownership.owner_runtime_id = runtime_id.to_string();
    record.ownership.lease_token =
        lease_token(session_id, runtime_id, record.ownership.lease_generation);
    record.ownership.lease_expires_at_ms = logical_now_ms.saturating_add(lease_duration_ms);
}

fn set_owner_runtime(
    record: &mut SessionRecord,
    session_id: SessionId,
    runtime_id: &str,
    logical_now_ms: i64,
    lease_duration_ms: i64,
) {
    record.ownership.owner_runtime_id = runtime_id.to_string();
    bump_lease(
        record,
        session_id,
        runtime_id,
        logical_now_ms,
        lease_duration_ms,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::runtime_bootstrap::{
        RuntimeBootstrapConfig, RuntimeBuildMetadata, RuntimeSecretValue,
    };
    use selene_kernel_contracts::ph1_voice_id::UserId;
    use selene_kernel_contracts::ph1j::DeviceId;
    use selene_kernel_contracts::ph1link::AppPlatform;
    use selene_kernel_contracts::provider_secrets::ProviderSecretId;
    use selene_kernel_contracts::runtime_execution::AdmissionState;

    #[derive(Debug, Clone, Copy)]
    struct FixedClock;

    impl RuntimeClock for FixedClock {
        fn now_unix_ms(&self) -> i64 {
            1_700_000_000_000
        }
    }

    #[derive(Debug, Clone)]
    struct StaticSecretsProvider;

    impl RuntimeSecretsProvider for StaticSecretsProvider {
        fn get_secret(&self, _secret_id: ProviderSecretId) -> Option<RuntimeSecretValue> {
            Some(RuntimeSecretValue::new("slice_1c_secret".to_string()).expect("secret"))
        }
    }

    fn device(id: &str) -> DeviceId {
        DeviceId::new(id).expect("device_id")
    }

    fn user(id: &str) -> UserId {
        UserId::new(id).expect("user_id")
    }

    fn base_runtime_envelope() -> RuntimeExecutionEnvelope {
        RuntimeExecutionEnvelope::v1(
            "req-1".to_string(),
            "trace-1".to_string(),
            "idem-1".to_string(),
            user("user-1"),
            device("device-a"),
            AppPlatform::Desktop,
            None,
            TurnId(1),
            AdmissionState::IngressValidated,
        )
        .expect("envelope")
    }

    fn session_runtime() -> RuntimeSessionFoundation {
        RuntimeSessionFoundation::default()
    }

    fn session_runtime_with_config(
        config: RuntimeSessionFoundationConfig,
    ) -> RuntimeSessionFoundation {
        RuntimeSessionFoundation::new(config)
    }

    fn startup_container() -> RuntimeServiceContainer<FixedClock, StaticSecretsProvider> {
        RuntimeServiceContainer::with_startup_foundation(FixedClock, StaticSecretsProvider)
            .expect("container")
    }

    #[test]
    fn slice_1c_session_id_generation_is_bounded_to_foundation_service() {
        let mut runtime = session_runtime();

        let first = runtime
            .create_session(device("device-a"))
            .expect("first session");
        let second = runtime
            .create_session(device("device-b"))
            .expect("second session");

        assert_eq!(first.projection.session_id, SessionId(1));
        assert_eq!(second.projection.session_id, SessionId(2));
        assert_eq!(first.projection.session_state, SessionState::Open);
        assert_eq!(second.projection.session_state, SessionState::Open);
    }

    #[test]
    fn slice_1c_turn_id_generation_is_deterministic_and_retry_safe() {
        let mut runtime = session_runtime();
        let created = runtime.create_session(device("device-a")).expect("session");

        let started = runtime
            .begin_turn(created.projection.session_id, &device("device-a"), 1)
            .expect("turn start");
        let permit = match started {
            SessionTurnResolution::Started(permit) => permit,
            _ => panic!("expected started turn"),
        };
        assert_eq!(permit.turn_id, TurnId(1));

        runtime
            .finish_turn(permit.clone(), SessionState::Active)
            .expect("turn finish");

        let retry = runtime
            .begin_turn(created.projection.session_id, &device("device-a"), 1)
            .expect("retry");
        match retry {
            SessionTurnResolution::Retry(projection) => {
                assert_eq!(projection.turn_id, Some(TurnId(1)));
                assert_eq!(
                    projection.attach_outcome,
                    Some(SessionAttachOutcome::RetryReusedResult)
                );
            }
            _ => panic!("expected retry reuse"),
        }

        let next_started = runtime
            .begin_turn(created.projection.session_id, &device("device-a"), 2)
            .expect("second turn");
        let next_permit = match next_started {
            SessionTurnResolution::Started(permit) => permit,
            _ => panic!("expected second started turn"),
        };
        assert_eq!(next_permit.turn_id, TurnId(2));
    }

    #[test]
    fn slice_1c_valid_state_transitions_succeed() {
        let mut runtime = session_runtime();
        let created = runtime.create_session(device("device-a")).expect("session");
        assert_eq!(created.projection.session_state, SessionState::Open);

        let started = runtime
            .begin_turn(created.projection.session_id, &device("device-a"), 1)
            .expect("turn start");
        let permit = match started {
            SessionTurnResolution::Started(permit) => permit,
            _ => panic!("expected started turn"),
        };

        let committed = runtime
            .finish_turn(permit, SessionState::SoftClosed)
            .expect("turn finish");
        assert_eq!(committed.projection.session_state, SessionState::SoftClosed);

        let detached = runtime
            .detach_session(created.projection.session_id, &device("device-a"))
            .expect("detach");
        assert!(detached.remaining_devices.is_empty());

        let closed = runtime
            .close_soft_closed_session(created.projection.session_id)
            .expect("close");
        assert_eq!(closed.session_state, SessionState::Closed);
    }

    #[test]
    fn slice_1c_invalid_state_transitions_fail_closed() {
        let mut runtime = session_runtime();
        let created = runtime.create_session(device("device-a")).expect("session");

        let close_err = runtime
            .close_soft_closed_session(created.projection.session_id)
            .expect_err("open session cannot close directly");
        assert_eq!(
            close_err.kind,
            SessionFoundationErrorKind::CloseRequiresSoftClosed
        );

        let suspend = runtime
            .suspend_session(created.projection.session_id, "runtime guard")
            .expect("suspend");
        assert_eq!(suspend.session_state, SessionState::Suspended);

        let resume_err = runtime
            .resume_session(created.projection.session_id, device("device-a"))
            .expect_err("suspended session must recover first");
        assert_eq!(
            resume_err.kind,
            SessionFoundationErrorKind::ResumeNotRecoverable
        );

        let seq_err = runtime
            .begin_turn(created.projection.session_id, &device("device-a"), 0)
            .expect_err("zero sequence must fail");
        assert_eq!(
            seq_err.kind,
            SessionFoundationErrorKind::InvalidDeviceSequence
        );
    }

    #[test]
    fn slice_1c_open_bypass_rule_is_lawful_for_first_turn() {
        let mut runtime = session_runtime();

        let started = runtime
            .start_new_session_turn(device("device-a"), 1)
            .expect("open bypass");
        let permit = match started {
            SessionTurnResolution::Started(permit) => permit,
            _ => panic!("expected started turn"),
        };

        assert_eq!(permit.previous_state, SessionState::Closed);
        assert_eq!(permit.turn_id, TurnId(1));
        assert_eq!(
            permit.attach_outcome,
            Some(SessionAttachOutcome::NewSessionCreated)
        );

        let snapshot = runtime
            .session_snapshot(permit.session_id)
            .expect("snapshot");
        assert_eq!(snapshot.session_state, SessionState::Active);
        assert_eq!(
            snapshot.active_writer.as_ref().map(|writer| writer.turn_id),
            Some(TurnId(1))
        );
    }

    #[test]
    fn slice_1c_attach_resume_recover_and_detach_are_coherent() {
        let mut runtime = session_runtime();
        let created = runtime.create_session(device("device-a")).expect("session");

        let attached = runtime
            .attach_session(created.projection.session_id, device("device-b"))
            .expect("attach second device");
        assert_eq!(
            attached.projection.attach_outcome,
            Some(SessionAttachOutcome::ExistingSessionAttached)
        );
        assert_eq!(
            attached.attached_devices,
            vec!["device-a".to_string(), "device-b".to_string()]
        );

        let resumed = runtime
            .resume_session(created.projection.session_id, device("device-a"))
            .expect("resume");
        assert_eq!(resumed.projection.session_state, SessionState::Active);

        runtime
            .suspend_session(created.projection.session_id, "capture degraded")
            .expect("suspend");

        let recovered = runtime
            .recover_session(created.projection.session_id, device("device-b"))
            .expect("recover");
        assert_eq!(recovered.projection.session_state, SessionState::Open);

        let detached = runtime
            .detach_session(created.projection.session_id, &device("device-b"))
            .expect("detach");
        assert_eq!(detached.projection.session_state, SessionState::Open);
        assert_eq!(detached.remaining_devices, vec!["device-a".to_string()]);
    }

    #[test]
    fn import_persisted_soft_closed_session_round_trips_into_resume() {
        let mut runtime = session_runtime();
        let imported = runtime
            .import_persisted_soft_closed_session(PersistedSoftClosedSessionImport {
                session_id: SessionId(41),
                persisted_session_state: SessionState::SoftClosed,
                attached_devices: vec!["device-a".to_string()],
                last_attached_device_id: "device-a".to_string(),
                last_turn_id: Some(TurnId(7)),
                device_turn_sequences: BTreeMap::from([("device-a".to_string(), 3)]),
            })
            .expect("import");

        assert_eq!(imported.session_state, SessionState::SoftClosed);
        assert_eq!(imported.next_turn_id, 8);
        assert_eq!(imported.attached_devices, vec!["device-a".to_string()]);
        assert_eq!(imported.access_classes.len(), 1);
        assert_eq!(
            imported.access_classes[0],
            SessionAccessSnapshot {
                device_id: "device-a".to_string(),
                access_class: SessionAccessClass::PrimaryInteractor,
            }
        );

        let resumed = runtime
            .resume_session(SessionId(41), device("device-a"))
            .expect("resume");
        assert_eq!(resumed.projection.session_state, SessionState::Active);
        assert_eq!(
            resumed.projection.attach_outcome,
            Some(SessionAttachOutcome::ExistingSessionReused)
        );
        assert_eq!(resumed.attached_devices, vec!["device-a".to_string()]);

        let snapshot = runtime.session_snapshot(SessionId(41)).expect("snapshot");
        assert_eq!(snapshot.session_state, SessionState::Active);
        assert_eq!(snapshot.next_turn_id, 8);
        assert_eq!(
            snapshot
                .device_timelines
                .iter()
                .find(|timeline| timeline.device_id == "device-a")
                .map(|timeline| timeline.highest_seen_sequence),
            Some(3)
        );
    }

    #[test]
    fn import_persisted_soft_closed_session_rejects_non_soft_closed() {
        let mut runtime = session_runtime();
        let err = runtime
            .import_persisted_soft_closed_session(PersistedSoftClosedSessionImport {
                session_id: SessionId(42),
                persisted_session_state: SessionState::Open,
                attached_devices: vec!["device-a".to_string()],
                last_attached_device_id: "device-a".to_string(),
                last_turn_id: Some(TurnId(2)),
                device_turn_sequences: BTreeMap::from([("device-a".to_string(), 1)]),
            })
            .expect_err("non-soft-closed import must fail");

        assert_eq!(err.kind, SessionFoundationErrorKind::ResumeNotRecoverable);
    }

    #[test]
    fn import_persisted_suspended_session_round_trips_into_recover() {
        let mut runtime = session_runtime();
        let imported = runtime
            .import_persisted_suspended_session(PersistedSuspendedSessionImport {
                session_id: SessionId(43),
                persisted_session_state: SessionState::Suspended,
                attached_devices: vec!["device-a".to_string()],
                last_attached_device_id: "device-a".to_string(),
                last_turn_id: Some(TurnId(9)),
                device_turn_sequences: BTreeMap::from([("device-a".to_string(), 4)]),
            })
            .expect("import");

        assert_eq!(imported.session_state, SessionState::Suspended);
        assert_eq!(imported.next_turn_id, 10);
        assert_eq!(imported.attached_devices, vec!["device-a".to_string()]);
        assert_eq!(imported.access_classes.len(), 1);
        assert_eq!(
            imported.access_classes[0],
            SessionAccessSnapshot {
                device_id: "device-a".to_string(),
                access_class: SessionAccessClass::PrimaryInteractor,
            }
        );

        let recovered = runtime
            .recover_session(SessionId(43), device("device-a"))
            .expect("recover");
        assert_eq!(recovered.projection.session_state, SessionState::Open);
        assert_eq!(
            recovered.projection.attach_outcome,
            Some(SessionAttachOutcome::ExistingSessionReused)
        );
        assert_eq!(recovered.attached_devices, vec!["device-a".to_string()]);

        let snapshot = runtime.session_snapshot(SessionId(43)).expect("snapshot");
        assert_eq!(snapshot.session_state, SessionState::Open);
        assert_eq!(snapshot.next_turn_id, 10);
        assert_eq!(
            snapshot
                .device_timelines
                .iter()
                .find(|timeline| timeline.device_id == "device-a")
                .map(|timeline| timeline.highest_seen_sequence),
            Some(4)
        );
    }

    #[test]
    fn import_persisted_suspended_session_rejects_non_suspended() {
        let mut runtime = session_runtime();
        let err = runtime
            .import_persisted_suspended_session(PersistedSuspendedSessionImport {
                session_id: SessionId(44),
                persisted_session_state: SessionState::SoftClosed,
                attached_devices: vec!["device-a".to_string()],
                last_attached_device_id: "device-a".to_string(),
                last_turn_id: Some(TurnId(3)),
                device_turn_sequences: BTreeMap::from([("device-a".to_string(), 2)]),
            })
            .expect_err("non-suspended import must fail");

        assert_eq!(err.kind, SessionFoundationErrorKind::RecoverNotSuspended);
    }

    #[test]
    fn slice_1c_single_writer_gate_rejects_concurrent_mutation_attempts() {
        let mut runtime = session_runtime();
        let created = runtime.create_session(device("device-a")).expect("session");

        let started = runtime
            .begin_turn(created.projection.session_id, &device("device-a"), 1)
            .expect("turn start");
        let permit = match started {
            SessionTurnResolution::Started(permit) => permit,
            _ => panic!("expected started turn"),
        };

        let err = runtime
            .attach_session(created.projection.session_id, device("device-b"))
            .expect_err("attach must fail while writer is active");
        assert_eq!(err.kind, SessionFoundationErrorKind::SingleWriterConflict);

        runtime
            .finish_turn(permit, SessionState::Active)
            .expect("finish");
    }

    #[test]
    fn import_persisted_open_session_round_trips_into_attach() {
        let mut runtime = session_runtime();
        let imported = runtime
            .import_persisted_attachable_session(PersistedAttachableSessionImport {
                session_id: SessionId(45),
                persisted_session_state: SessionState::Open,
                attached_devices: vec!["device-a".to_string()],
                last_attached_device_id: "device-a".to_string(),
                last_turn_id: Some(TurnId(5)),
                device_turn_sequences: BTreeMap::from([("device-a".to_string(), 2)]),
            })
            .expect("import");

        assert_eq!(imported.session_state, SessionState::Open);
        assert_eq!(imported.next_turn_id, 6);
        assert_eq!(imported.attached_devices, vec!["device-a".to_string()]);
        assert_eq!(imported.access_classes.len(), 1);
        assert_eq!(
            imported.access_classes[0],
            SessionAccessSnapshot {
                device_id: "device-a".to_string(),
                access_class: SessionAccessClass::PrimaryInteractor,
            }
        );

        let attached = runtime
            .attach_session(SessionId(45), device("device-b"))
            .expect("attach");
        assert_eq!(attached.projection.session_state, SessionState::Open);
        assert_eq!(
            attached.projection.attach_outcome,
            Some(SessionAttachOutcome::ExistingSessionAttached)
        );
        assert_eq!(
            attached.attached_devices,
            vec!["device-a".to_string(), "device-b".to_string()]
        );

        let snapshot = runtime.session_snapshot(SessionId(45)).expect("snapshot");
        assert_eq!(snapshot.session_state, SessionState::Open);
        assert_eq!(snapshot.next_turn_id, 6);
        assert_eq!(
            snapshot
                .device_timelines
                .iter()
                .find(|timeline| timeline.device_id == "device-a")
                .map(|timeline| timeline.highest_seen_sequence),
            Some(2)
        );
    }

    #[test]
    fn import_persisted_active_session_round_trips_into_attach() {
        let mut runtime = session_runtime();
        let imported = runtime
            .import_persisted_attachable_session(PersistedAttachableSessionImport {
                session_id: SessionId(46),
                persisted_session_state: SessionState::Active,
                attached_devices: vec!["device-a".to_string()],
                last_attached_device_id: "device-a".to_string(),
                last_turn_id: Some(TurnId(11)),
                device_turn_sequences: BTreeMap::from([("device-a".to_string(), 6)]),
            })
            .expect("import");

        assert_eq!(imported.session_state, SessionState::Active);
        assert_eq!(imported.next_turn_id, 12);
        assert_eq!(imported.attached_devices, vec!["device-a".to_string()]);
        assert_eq!(imported.access_classes.len(), 1);
        assert_eq!(
            imported.access_classes[0],
            SessionAccessSnapshot {
                device_id: "device-a".to_string(),
                access_class: SessionAccessClass::PrimaryInteractor,
            }
        );

        let attached = runtime
            .attach_session(SessionId(46), device("device-b"))
            .expect("attach");
        assert_eq!(attached.projection.session_state, SessionState::Active);
        assert_eq!(
            attached.projection.attach_outcome,
            Some(SessionAttachOutcome::ExistingSessionAttached)
        );
        assert_eq!(
            attached.attached_devices,
            vec!["device-a".to_string(), "device-b".to_string()]
        );

        let snapshot = runtime.session_snapshot(SessionId(46)).expect("snapshot");
        assert_eq!(snapshot.session_state, SessionState::Active);
        assert_eq!(snapshot.next_turn_id, 12);
        assert_eq!(
            snapshot
                .device_timelines
                .iter()
                .find(|timeline| timeline.device_id == "device-a")
                .map(|timeline| timeline.highest_seen_sequence),
            Some(6)
        );
    }

    #[test]
    fn import_persisted_attachable_session_rejects_non_attachable() {
        let mut runtime = session_runtime();
        let err = runtime
            .import_persisted_attachable_session(PersistedAttachableSessionImport {
                session_id: SessionId(47),
                persisted_session_state: SessionState::SoftClosed,
                attached_devices: vec!["device-a".to_string()],
                last_attached_device_id: "device-a".to_string(),
                last_turn_id: Some(TurnId(3)),
                device_turn_sequences: BTreeMap::from([("device-a".to_string(), 1)]),
            })
            .expect_err("non-attachable import must fail");

        assert_eq!(err.kind, SessionFoundationErrorKind::AttachNotAllowed);
    }

    #[test]
    fn slice_1c_device_timeline_classifies_new_retry_and_stale() {
        let mut runtime = session_runtime();
        let created = runtime.create_session(device("device-a")).expect("session");

        let started = runtime
            .begin_turn(created.projection.session_id, &device("device-a"), 1)
            .expect("turn start");
        let permit = match started {
            SessionTurnResolution::Started(permit) => permit,
            _ => panic!("expected started turn"),
        };
        let committed = runtime
            .finish_turn(permit, SessionState::Active)
            .expect("finish");
        assert_eq!(committed.classification, DeviceTimelineClassification::New);

        let retry = runtime
            .begin_turn(created.projection.session_id, &device("device-a"), 1)
            .expect("retry");
        match retry {
            SessionTurnResolution::Retry(projection) => {
                assert_eq!(projection.turn_id, Some(TurnId(1)));
            }
            _ => panic!("expected retry"),
        }

        let stale_err = runtime
            .begin_turn(created.projection.session_id, &device("device-a"), 0)
            .expect_err("zero sequence is invalid");
        assert_eq!(
            stale_err.kind,
            SessionFoundationErrorKind::InvalidDeviceSequence
        );
    }

    #[test]
    fn slice_1c_stale_and_retry_device_turns_do_not_mutate_session_state() {
        let mut runtime = session_runtime();
        let created = runtime.create_session(device("device-a")).expect("session");

        let started = runtime
            .begin_turn(created.projection.session_id, &device("device-a"), 1)
            .expect("turn start");
        let first_permit = match started {
            SessionTurnResolution::Started(permit) => permit,
            _ => panic!("expected started turn"),
        };
        runtime
            .finish_turn(first_permit, SessionState::Active)
            .expect("finish");

        let next_started = runtime
            .begin_turn(created.projection.session_id, &device("device-a"), 2)
            .expect("second turn");
        let second_permit = match next_started {
            SessionTurnResolution::Started(permit) => permit,
            _ => panic!("expected second started turn"),
        };
        runtime
            .finish_turn(second_permit, SessionState::SoftClosed)
            .expect("finish second");

        let before = runtime
            .session_snapshot(created.projection.session_id)
            .expect("snapshot before");

        let retry = runtime
            .begin_turn(created.projection.session_id, &device("device-a"), 2)
            .expect("retry");
        match retry {
            SessionTurnResolution::Retry(projection) => {
                assert_eq!(projection.turn_id, Some(TurnId(2)));
                assert_eq!(projection.session_state, SessionState::SoftClosed);
            }
            _ => panic!("expected retry"),
        }

        let stale_err = runtime
            .begin_turn(created.projection.session_id, &device("device-a"), 1)
            .expect_err("stale turn must fail");
        assert_eq!(stale_err.kind, SessionFoundationErrorKind::StaleDeviceTurn);

        let after = runtime
            .session_snapshot(created.projection.session_id)
            .expect("snapshot after");
        assert_eq!(before, after);
    }

    #[test]
    fn slice_1c_integrity_checks_detect_impossible_state_or_timeline() {
        let mut runtime = session_runtime();
        let created = runtime.create_session(device("device-a")).expect("session");
        let session_id = created.projection.session_id;

        {
            let record = runtime.sessions.get_mut(&session_id).expect("record");
            record.state = SessionState::Closed;
            record.attached_devices.insert("device-b".to_string());
        }

        let err = runtime
            .integrity_check(session_id)
            .expect_err("closed session with attached devices must fail");
        assert_eq!(err.kind, SessionFoundationErrorKind::IntegrityViolation);

        {
            let record = runtime.sessions.get_mut(&session_id).expect("record");
            record.state = SessionState::Open;
            record.attached_devices.clear();
            record.attached_devices.insert("device-a".to_string());
            record.device_timeline_map.insert(
                "device-a".to_string(),
                DeviceTimelineRecord {
                    highest_seen_sequence: 2,
                    last_turn_id: TurnId(record.next_turn_id),
                },
            );
        }

        let timeline_err = runtime
            .integrity_check(session_id)
            .expect_err("impossible timeline must fail");
        assert_eq!(
            timeline_err.kind,
            SessionFoundationErrorKind::IntegrityViolation
        );
    }

    #[test]
    fn slice_1c_projection_binds_session_fields_into_runtime_envelope() {
        let mut runtime = session_runtime();
        let started = runtime
            .start_new_session_turn(device("device-a"), 1)
            .expect("open bypass");
        let permit = match started {
            SessionTurnResolution::Started(permit) => permit,
            _ => panic!("expected started turn"),
        };
        let committed = runtime
            .finish_turn(permit, SessionState::Active)
            .expect("finish");

        let bound = committed
            .projection
            .bind_to_runtime_envelope(&base_runtime_envelope())
            .expect("bind");

        assert_eq!(bound.session_id, Some(committed.projection.session_id));
        assert_eq!(bound.turn_id, TurnId(1));
        assert_eq!(bound.device_turn_sequence, Some(1));
        assert_eq!(
            bound.session_attach_outcome,
            Some(SessionAttachOutcome::NewSessionCreated)
        );
    }

    #[test]
    fn stage_5a_current_committed_turn_can_enter_understanding_without_route_authority() {
        let mut runtime = session_runtime();
        let created = runtime.create_session(device("device-a")).expect("session");
        let started = runtime
            .begin_turn(created.projection.session_id, &device("device-a"), 1)
            .expect("turn start");
        let permit = match started {
            SessionTurnResolution::Started(permit) => permit,
            _ => panic!("expected started turn"),
        };
        let commit = runtime
            .finish_turn(permit.clone(), SessionState::Active)
            .expect("turn commit");

        let authority = runtime
            .authorize_stage5_current_committed_turn(&commit, &permit)
            .expect("stage 5 authority");

        assert_eq!(
            authority.disposition,
            Stage5TurnAuthorityDisposition::CurrentCommittedTurn
        );
        assert!(authority.can_enter_understanding());
        assert!(authority.can_render_current_result());
        assert!(!authority.can_route_any_work());
        assert!(!authority.authority.can_route_tools);
        assert!(!authority.authority.can_route_search);
        assert!(!authority.authority.can_route_providers);
        assert!(!authority.authority.can_route_tts);
        assert!(!authority.authority.can_route_protected_execution);
    }

    #[test]
    fn stage_5a_deferred_newer_turn_quarantines_old_result_render() {
        let mut runtime = session_runtime();
        let created = runtime.create_session(device("device-a")).expect("session");
        let session_id = created.projection.session_id;
        let started = runtime
            .begin_turn(session_id, &device("device-a"), 1)
            .expect("turn start");
        let permit = match started {
            SessionTurnResolution::Started(permit) => permit,
            _ => panic!("expected started turn"),
        };

        let deferred = runtime
            .begin_turn(session_id, &device("device-a"), 2)
            .expect("newer turn defers while first drains");
        assert!(matches!(deferred, SessionTurnResolution::Deferred(_)));

        let commit = runtime
            .finish_turn(permit.clone(), SessionState::Active)
            .expect("turn commit");
        let authority = runtime
            .authorize_stage5_current_committed_turn(&commit, &permit)
            .expect("stage 5 quarantine");

        assert_eq!(
            authority.disposition,
            Stage5TurnAuthorityDisposition::SupersededTurnQuarantined
        );
        assert!(!authority.can_enter_understanding());
        assert!(!authority.can_render_current_result());
        assert!(!authority.can_route_any_work());
    }

    #[test]
    fn stage_5a_older_completed_turn_cannot_become_current_after_newer_commit() {
        let mut runtime = session_runtime();
        let created = runtime.create_session(device("device-a")).expect("session");
        let session_id = created.projection.session_id;
        let first_started = runtime
            .begin_turn(session_id, &device("device-a"), 1)
            .expect("first turn start");
        let first_permit = match first_started {
            SessionTurnResolution::Started(permit) => permit,
            _ => panic!("expected first turn"),
        };
        let first_commit = runtime
            .finish_turn(first_permit.clone(), SessionState::Active)
            .expect("first turn commit");

        let second_started = runtime
            .begin_turn(session_id, &device("device-a"), 2)
            .expect("second turn start");
        let second_permit = match second_started {
            SessionTurnResolution::Started(permit) => permit,
            _ => panic!("expected second turn"),
        };
        runtime
            .finish_turn(second_permit, SessionState::Active)
            .expect("second turn commit");

        let old_authority = runtime
            .authorize_stage5_current_committed_turn(&first_commit, &first_permit)
            .expect("old turn quarantine");

        assert_eq!(
            old_authority.disposition,
            Stage5TurnAuthorityDisposition::SupersededTurnQuarantined
        );
        assert!(!old_authority.can_enter_understanding());
        assert!(!old_authority.can_render_current_result());
        assert!(!old_authority.can_route_any_work());
    }

    #[test]
    fn stage_5a_quarantine_dispositions_cannot_enter_understanding_or_route_work() {
        let dispositions = [
            Stage5TurnAuthorityDisposition::RetryReusedResult,
            Stage5TurnAuthorityDisposition::DeferredTurn,
            Stage5TurnAuthorityDisposition::StaleTurnQuarantined,
            Stage5TurnAuthorityDisposition::SupersededTurnQuarantined,
            Stage5TurnAuthorityDisposition::CancelledTurnQuarantined,
            Stage5TurnAuthorityDisposition::AbandonedTurnQuarantined,
            Stage5TurnAuthorityDisposition::RecordArtifactOnly,
        ];

        for disposition in dispositions {
            let packet = Stage5TurnAuthorityPacket::quarantined(
                SessionId(77),
                Some(TurnId(7)),
                Some("device-a".to_string()),
                Some(3),
                SessionState::Active,
                disposition,
            )
            .expect("quarantine packet");
            assert!(!packet.can_enter_understanding());
            assert!(!packet.can_render_current_result());
            assert!(!packet.can_route_any_work());
        }

        let closed = Stage5TurnAuthorityPacket::quarantined(
            SessionId(77),
            Some(TurnId(7)),
            Some("device-a".to_string()),
            Some(3),
            SessionState::Closed,
            Stage5TurnAuthorityDisposition::ClosedSessionRejected,
        )
        .expect("closed session packet");
        assert!(!closed.can_enter_understanding());
        assert!(!closed.can_render_current_result());
        assert!(!closed.can_route_any_work());
    }

    #[test]
    fn slice_1c_registers_internal_foundation_services_without_section03_or_authority_drift() {
        let mut container = startup_container();
        RuntimeSessionFoundation::register_slice_1c_session_foundation_services(&mut container)
            .expect("register services");

        let service_ids = container.service_ids();
        assert!(service_ids.contains(&"runtime_session_store"));
        assert!(service_ids.contains(&"runtime_session_identifier_generator"));
        assert!(service_ids.contains(&"runtime_session_turn_gate"));
        assert!(service_ids.contains(&"runtime_session_event_stream"));
        assert!(service_ids.contains(&"runtime_session_projection"));
        assert!(service_ids
            .iter()
            .all(|service_id| !service_id.starts_with("/v1/")));
        assert!(service_ids
            .iter()
            .all(|service_id| !service_id.contains("authority")));
    }

    #[test]
    fn slice_1c_service_registration_reuses_slice_1a_container_and_leaves_bootstrap_contracts_valid(
    ) {
        let mut container = startup_container();
        RuntimeSessionFoundation::register_slice_1c_session_foundation_services(&mut container)
            .expect("register services");

        let config = RuntimeBootstrapConfig {
            service_name: "slice_1c_runtime".to_string(),
            shutdown_grace_period_ms: 1_000,
            required_secrets: vec![ProviderSecretId::OpenAIApiKey],
            build_metadata: RuntimeBuildMetadata {
                node_id: "node-a".to_string(),
                runtime_instance_identity: "runtime-a".to_string(),
                environment_identity: "test".to_string(),
                build_version: "1.0.0".to_string(),
                git_commit: "deadbeef".to_string(),
            },
        };
        let runtime = crate::runtime_bootstrap::RuntimeProcess::new(config, container);
        assert!(runtime.service_ids().contains(&"runtime_session_store"));
        assert!(runtime.service_ids().contains(&"runtime_session_turn_gate"));
    }

    #[test]
    fn slice_1d_coordination_and_consistency_posture_are_exposed_lawfully() {
        let mut runtime = session_runtime();
        let created = runtime.create_session(device("device-a")).expect("session");
        let session_id = created.projection.session_id;

        let primary_owned = runtime.coordination_view(session_id).expect("coordination");
        assert_eq!(
            primary_owned.coordination_state,
            SessionCoordinationState::PrimaryOwned
        );
        assert_eq!(
            primary_owned.consistency_level,
            SessionConsistencyLevel::Strict
        );
        assert_eq!(primary_owned.owner_runtime_id, "runtime.slice1.local");

        let transfer_pending = runtime
            .begin_ownership_transfer(session_id, "runtime-b")
            .expect("transfer request");
        assert_eq!(
            transfer_pending.coordination_state,
            SessionCoordinationState::TransferPending
        );
        assert_eq!(
            transfer_pending.consistency_level,
            SessionConsistencyLevel::LeasedDistributed
        );

        let transferred = runtime
            .acknowledge_ownership_transfer(session_id, "runtime-b")
            .expect("transfer ack");
        assert_eq!(
            transferred.coordination_state,
            SessionCoordinationState::PrimaryOwned
        );
        assert_eq!(transferred.owner_runtime_id, "runtime-b");

        runtime.advance_logical_time_ms(30_000);
        let uncertain = runtime
            .coordination_view(session_id)
            .expect("expired lease");
        assert_eq!(
            uncertain.coordination_state,
            SessionCoordinationState::OwnershipUncertain
        );
        assert_eq!(
            uncertain.consistency_level,
            SessionConsistencyLevel::DegradedRecovery
        );

        let recovering = runtime
            .begin_failover_recovery(session_id, "runtime-c")
            .expect("failover recovery");
        assert_eq!(
            recovering.coordination_state,
            SessionCoordinationState::FailoverRecovering
        );
        assert_eq!(recovering.owner_runtime_id, "runtime-c");

        let recovered = runtime
            .complete_failover_recovery(session_id, "runtime-c")
            .expect("recovery completion");
        assert_eq!(
            recovered.coordination_state,
            SessionCoordinationState::PrimaryOwned
        );
        assert_eq!(recovered.consistency_level, SessionConsistencyLevel::Strict);
    }

    #[test]
    fn slice_1d_access_class_handling_is_deterministic_and_bounded() {
        let mut runtime = session_runtime();
        let created = runtime.create_session(device("device-a")).expect("session");
        let session_id = created.projection.session_id;

        runtime
            .attach_session_with_access_claim(
                session_id,
                device("device-b"),
                SessionAccessClass::SecondaryViewer,
                None,
            )
            .expect("viewer attach");

        let snapshot = runtime.session_snapshot(session_id).expect("snapshot");
        assert!(snapshot.access_classes.contains(&SessionAccessSnapshot {
            device_id: "device-a".to_string(),
            access_class: SessionAccessClass::PrimaryInteractor,
        }));
        assert!(snapshot.access_classes.contains(&SessionAccessSnapshot {
            device_id: "device-b".to_string(),
            access_class: SessionAccessClass::SecondaryViewer,
        }));

        let recovery_attach_err = runtime
            .attach_session_with_access_claim(
                session_id,
                device("device-c"),
                SessionAccessClass::RecoveryAttach,
                None,
            )
            .expect_err("recovery attach must stay bounded");
        assert_eq!(
            recovery_attach_err.kind,
            SessionFoundationErrorKind::InvalidAccessClass
        );

        let duplicate_claim_err = runtime
            .attach_session_with_access_claim(
                session_id,
                device("device-b"),
                SessionAccessClass::PrimaryInteractor,
                None,
            )
            .expect_err("second primary interactor must fail");
        assert_eq!(
            duplicate_claim_err.kind,
            SessionFoundationErrorKind::DuplicateDeviceClaim
        );
    }

    #[test]
    fn slice_1d_late_created_sessions_start_with_a_fresh_primary_owned_strict_lease_window() {
        let mut runtime = session_runtime();
        runtime.advance_logical_time_ms(120_000);
        let creation_time_ms = runtime.current_logical_time_ms();

        let created = runtime.create_session(device("device-a")).expect("session");
        let session_id = created.projection.session_id;
        let view = runtime.coordination_view(session_id).expect("coordination");

        assert_eq!(
            view.coordination_state,
            SessionCoordinationState::PrimaryOwned
        );
        assert_eq!(view.consistency_level, SessionConsistencyLevel::Strict);
        assert_eq!(view.owner_runtime_id, "runtime.slice1.local");
        assert_eq!(
            view.lease_expires_at_ms,
            creation_time_ms.saturating_add(runtime.config.lease_duration_ms)
        );
        assert!(view.lease_expires_at_ms > creation_time_ms);
    }

    #[test]
    fn slice_1d_late_created_sessions_expire_only_after_their_own_lease_window_elapses() {
        let mut runtime = session_runtime();
        runtime.advance_logical_time_ms(120_000);

        let created = runtime.create_session(device("device-a")).expect("session");
        let session_id = created.projection.session_id;

        runtime.advance_logical_time_ms(runtime.config.lease_duration_ms - 1);
        let before_expiry = runtime
            .coordination_view(session_id)
            .expect("before expiry");
        assert_eq!(
            before_expiry.coordination_state,
            SessionCoordinationState::PrimaryOwned
        );
        assert_eq!(
            before_expiry.consistency_level,
            SessionConsistencyLevel::Strict
        );

        runtime.advance_logical_time_ms(1);
        let after_expiry = runtime.coordination_view(session_id).expect("after expiry");
        assert_eq!(
            after_expiry.coordination_state,
            SessionCoordinationState::OwnershipUncertain
        );
        assert_eq!(
            after_expiry.consistency_level,
            SessionConsistencyLevel::DegradedRecovery
        );
    }

    #[test]
    fn slice_1d_ownership_and_lease_foundation_is_coherent_in_process() {
        let mut runtime = session_runtime();
        let created = runtime.create_session(device("device-a")).expect("session");
        let session_id = created.projection.session_id;

        let before = runtime.coordination_view(session_id).expect("before");
        let renewed = runtime
            .renew_session_lease(session_id)
            .expect("lease renewal");
        assert_eq!(renewed.owner_runtime_id, before.owner_runtime_id);
        assert_ne!(renewed.lease_token, before.lease_token);
        assert!(renewed.lease_expires_at_ms >= before.lease_expires_at_ms);
    }

    #[test]
    fn slice_1d_lease_renewal_extends_from_current_logical_time_for_late_created_sessions() {
        let mut runtime = session_runtime();
        runtime.advance_logical_time_ms(90_000);

        let created = runtime.create_session(device("device-a")).expect("session");
        let session_id = created.projection.session_id;
        let before = runtime
            .coordination_view(session_id)
            .expect("before renewal");

        runtime.advance_logical_time_ms(5_000);
        let renewal_base_ms = runtime.current_logical_time_ms();
        let renewed = runtime
            .renew_session_lease(session_id)
            .expect("lease renewal");

        assert_eq!(
            renewed.lease_expires_at_ms,
            renewal_base_ms.saturating_add(runtime.config.lease_duration_ms)
        );
        assert!(renewed.lease_expires_at_ms > before.lease_expires_at_ms);
        assert_eq!(
            renewed.coordination_state,
            SessionCoordinationState::PrimaryOwned
        );
        assert_eq!(renewed.consistency_level, SessionConsistencyLevel::Strict);
    }

    #[test]
    fn slice_1d_ownership_transfer_happy_path_moves_primary_owner_without_split_brain() {
        let mut runtime = session_runtime();
        let created = runtime.create_session(device("device-a")).expect("session");
        let session_id = created.projection.session_id;

        runtime
            .begin_ownership_transfer(session_id, "runtime-b")
            .expect("transfer request");

        let pending_err = runtime
            .begin_turn(session_id, &device("device-a"), 1)
            .expect_err("old owner must block turn while transfer pending");
        assert_eq!(
            pending_err.kind,
            SessionFoundationErrorKind::OwnershipTransferPending
        );

        runtime
            .acknowledge_ownership_transfer(session_id, "runtime-b")
            .expect("transfer ack");

        let old_owner_err = runtime
            .begin_turn(session_id, &device("device-a"), 1)
            .expect_err("old owner must refuse after handoff");
        assert_eq!(
            old_owner_err.kind,
            SessionFoundationErrorKind::NotSessionOwner
        );

        let started = runtime
            .begin_turn_internal(session_id, &device("device-a"), 1, "runtime-b")
            .expect("new owner may start turn");
        let permit = match started {
            SessionTurnResolution::Started(permit) => permit,
            _ => panic!("expected started turn"),
        };
        assert_eq!(permit.runtime_id, "runtime-b");

        runtime
            .finish_turn(permit, SessionState::Active)
            .expect("finish transferred turn");
    }

    #[test]
    fn slice_1d_transfer_and_failover_remain_coherent_after_late_session_creation() {
        let mut runtime = session_runtime();
        runtime.advance_logical_time_ms(75_000);

        let created = runtime.create_session(device("device-a")).expect("session");
        let session_id = created.projection.session_id;

        runtime
            .begin_ownership_transfer(session_id, "runtime-b")
            .expect("transfer request");
        let transferred = runtime
            .acknowledge_ownership_transfer(session_id, "runtime-b")
            .expect("transfer ack");
        assert_eq!(
            transferred.coordination_state,
            SessionCoordinationState::PrimaryOwned
        );
        assert_eq!(
            transferred.consistency_level,
            SessionConsistencyLevel::Strict
        );
        assert_eq!(transferred.owner_runtime_id, "runtime-b");
        assert!(transferred.lease_expires_at_ms > runtime.current_logical_time_ms());

        runtime.advance_logical_time_ms(runtime.config.lease_duration_ms);
        let uncertain = runtime
            .coordination_view(session_id)
            .expect("expired lease view");
        assert_eq!(
            uncertain.coordination_state,
            SessionCoordinationState::OwnershipUncertain
        );
        assert_eq!(
            uncertain.consistency_level,
            SessionConsistencyLevel::DegradedRecovery
        );

        let recovering = runtime
            .begin_failover_recovery(session_id, "runtime-c")
            .expect("failover recovery");
        assert_eq!(
            recovering.coordination_state,
            SessionCoordinationState::FailoverRecovering
        );
        assert_eq!(recovering.owner_runtime_id, "runtime-c");
        assert!(recovering.lease_expires_at_ms > runtime.current_logical_time_ms());

        let recovered = runtime
            .complete_failover_recovery(session_id, "runtime-c")
            .expect("recovery complete");
        assert_eq!(
            recovered.coordination_state,
            SessionCoordinationState::PrimaryOwned
        );
        assert_eq!(recovered.consistency_level, SessionConsistencyLevel::Strict);
        assert_eq!(recovered.owner_runtime_id, "runtime-c");
        assert!(recovered.lease_expires_at_ms > runtime.current_logical_time_ms());
    }

    #[test]
    fn slice_1d_invalid_or_partial_transfer_is_rejected_fail_closed() {
        let mut runtime = session_runtime();
        let created = runtime.create_session(device("device-a")).expect("session");
        let session_id = created.projection.session_id;

        let started = runtime
            .begin_turn(session_id, &device("device-a"), 1)
            .expect("turn start");
        let permit = match started {
            SessionTurnResolution::Started(permit) => permit,
            _ => panic!("expected started turn"),
        };

        let drain_err = runtime
            .begin_ownership_transfer(session_id, "runtime-b")
            .expect_err("transfer must drain active work first");
        assert_eq!(
            drain_err.kind,
            SessionFoundationErrorKind::OwnershipTransferDrainRequired
        );

        runtime
            .finish_turn(permit, SessionState::Active)
            .expect("finish");
        runtime
            .begin_ownership_transfer(session_id, "runtime-b")
            .expect("transfer request");

        let mismatch_err = runtime
            .acknowledge_ownership_transfer(session_id, "runtime-c")
            .expect_err("mismatched target must fail");
        assert_eq!(
            mismatch_err.kind,
            SessionFoundationErrorKind::OwnershipTransferTargetMismatch
        );

        let rejected = runtime
            .reject_ownership_transfer(session_id, "runtime-b", "target refused")
            .expect("reject");
        assert_eq!(
            rejected.coordination_state,
            SessionCoordinationState::PrimaryOwned
        );
        assert_eq!(rejected.owner_runtime_id, "runtime.slice1.local");
    }

    #[test]
    fn slice_1d_conflict_resolution_is_deterministic() {
        let mut runtime = session_runtime();
        let created = runtime.create_session(device("device-a")).expect("session");
        let session_id = created.projection.session_id;

        runtime
            .attach_session_with_access_claim(
                session_id,
                device("device-b"),
                SessionAccessClass::LimitedAttach,
                None,
            )
            .expect("limited attach");

        let started = runtime
            .begin_turn(session_id, &device("device-a"), 1)
            .expect("turn start");
        let permit = match started {
            SessionTurnResolution::Started(permit) => permit,
            _ => panic!("expected started turn"),
        };

        let deferred = runtime
            .begin_turn(session_id, &device("device-a"), 2)
            .expect("simultaneous turn should defer");
        match deferred {
            SessionTurnResolution::Deferred(deferred) => {
                assert_eq!(deferred.pending_turn_count, 1);
            }
            _ => panic!("expected deferred turn"),
        }

        let duplicate_claim_err = runtime
            .attach_session_with_access_claim(
                session_id,
                device("device-b"),
                SessionAccessClass::PrimaryInteractor,
                None,
            )
            .expect_err("duplicate primary claim must fail");
        assert_eq!(
            duplicate_claim_err.kind,
            SessionFoundationErrorKind::SingleWriterConflict
        );

        runtime
            .finish_turn(permit, SessionState::Active)
            .expect("finish");
        let second_started = runtime
            .begin_turn(session_id, &device("device-a"), 2)
            .expect("deferred turn starts once drain clears");
        let second_permit = match second_started {
            SessionTurnResolution::Started(permit) => permit,
            _ => panic!("expected started deferred turn"),
        };
        runtime
            .finish_turn(second_permit, SessionState::Active)
            .expect("finish second");

        let stale_attach_err = runtime
            .attach_session_with_access_claim(
                session_id,
                device("device-a"),
                SessionAccessClass::LimitedAttach,
                Some(1),
            )
            .expect_err("stale attach must fail");
        assert_eq!(
            stale_attach_err.kind,
            SessionFoundationErrorKind::StaleAttach
        );

        let conflicting_resume_err = runtime
            .resume_session(session_id, device("device-b"))
            .expect_err("conflicting resume must fail");
        assert_eq!(
            conflicting_resume_err.kind,
            SessionFoundationErrorKind::ConflictingResume
        );
    }

    #[test]
    fn slice_1d_session_backpressure_defers_and_rejects_without_corrupting_state() {
        let mut runtime = session_runtime_with_config(RuntimeSessionFoundationConfig {
            max_attached_devices: 8,
            max_events: 256,
            max_pending_turns: 1,
            lease_duration_ms: 30_000,
            runtime_identity: "runtime.slice1.local".to_string(),
        });
        let created = runtime.create_session(device("device-a")).expect("session");
        let session_id = created.projection.session_id;

        let started = runtime
            .begin_turn(session_id, &device("device-a"), 1)
            .expect("turn start");
        let permit = match started {
            SessionTurnResolution::Started(permit) => permit,
            _ => panic!("expected started turn"),
        };

        let deferred = runtime
            .begin_turn(session_id, &device("device-a"), 2)
            .expect("first pending turn");
        assert!(matches!(deferred, SessionTurnResolution::Deferred(_)));

        let before = runtime.session_snapshot(session_id).expect("before");
        let overflow_err = runtime
            .begin_turn(session_id, &device("device-a"), 3)
            .expect_err("second pending turn must overflow");
        assert_eq!(
            overflow_err.kind,
            SessionFoundationErrorKind::BackpressureExceeded
        );
        let after = runtime.session_snapshot(session_id).expect("after");
        assert_eq!(before.active_writer, after.active_writer);
        assert_eq!(before.session_state, after.session_state);
        assert_eq!(after.deferred_turns.len(), 1);

        runtime
            .finish_turn(permit, SessionState::Active)
            .expect("finish");
        let deferred_started = runtime
            .begin_turn(session_id, &device("device-a"), 2)
            .expect("deferred turn starts");
        assert!(matches!(
            deferred_started,
            SessionTurnResolution::Started(_)
        ));
    }

    #[test]
    fn slice_1d_integrity_checks_detect_impossible_coordination_or_access_posture() {
        let mut runtime = session_runtime();
        let created = runtime.create_session(device("device-a")).expect("session");
        let session_id = created.projection.session_id;

        {
            let record = runtime.sessions.get_mut(&session_id).expect("record");
            record.coordination_state = SessionCoordinationState::TransferPending;
            record.pending_transfer_target = None;
        }
        let err = runtime
            .integrity_check(session_id)
            .expect_err("transfer posture without target must fail");
        assert_eq!(err.kind, SessionFoundationErrorKind::IntegrityViolation);

        {
            let record = runtime.sessions.get_mut(&session_id).expect("record");
            record.coordination_state = SessionCoordinationState::PrimaryOwned;
            record.consistency_level = SessionConsistencyLevel::Strict;
            record.pending_transfer_target = None;
            record
                .device_access_classes
                .insert("device-a".to_string(), SessionAccessClass::RecoveryAttach);
        }
        let access_err = runtime
            .integrity_check(session_id)
            .expect_err("invalid access-class combination must fail");
        assert_eq!(
            access_err.kind,
            SessionFoundationErrorKind::IntegrityViolation
        );
    }

    #[test]
    fn slice_1d_service_registration_stays_inside_section02_foundation() {
        let mut container = startup_container();
        RuntimeSessionFoundation::register_slice_1c_session_foundation_services(&mut container)
            .expect("register services");

        let service_ids = container.service_ids();
        assert!(service_ids.contains(&"runtime_session_coordination"));
        assert!(service_ids.contains(&"runtime_session_lease"));
        assert!(service_ids.contains(&"runtime_session_access_gate"));
        assert!(service_ids.contains(&"runtime_session_conflict_resolution"));
        assert!(service_ids.contains(&"runtime_session_backpressure"));
        assert!(service_ids.contains(&"runtime_session_transfer"));
        assert!(service_ids
            .iter()
            .all(|service_id| !service_id.starts_with("/v1/")));
        assert!(service_ids
            .iter()
            .all(|service_id| !service_id.contains("authority")));
        assert!(service_ids
            .iter()
            .all(|service_id| !service_id.contains("persistence")));
        assert!(service_ids
            .iter()
            .all(|service_id| !service_id.contains("apple")));
    }
}
