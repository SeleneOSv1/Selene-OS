#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1j::{
    AuditEventId, AuditEventInput, CanonicalProofRecord, CanonicalProofRecordInput, ProofVerificationResult,
    ProofWriteReceipt,
};
use selene_kernel_contracts::ph1l::SessionId;
use selene_kernel_contracts::ph1j::TurnId;

use crate::ph1f::{Ph1fStore, StorageError};

/// PH1.J storage wrapper.
///
/// Legacy audit rows are still supported for non-proof telemetry use cases,
/// but canonical protected execution now writes into the dedicated proof ledger.
#[derive(Debug, Default)]
pub struct Ph1jRuntime;

impl Ph1jRuntime {
    pub fn emit_audit(
        store: &mut Ph1fStore,
        input: AuditEventInput,
    ) -> Result<AuditEventId, StorageError> {
        store.append_audit_event(input)
    }

    pub fn emit(
        store: &mut Ph1fStore,
        input: AuditEventInput,
    ) -> Result<AuditEventId, StorageError> {
        Self::emit_audit(store, input)
    }

    pub fn emit_proof(
        store: &mut Ph1fStore,
        input: CanonicalProofRecordInput,
        idempotency_key: Option<String>,
    ) -> Result<ProofWriteReceipt, StorageError> {
        store.append_proof_record(input, idempotency_key)
    }

    pub fn replay_by_request_id(
        store: &Ph1fStore,
        request_id: &str,
        limit: usize,
    ) -> Result<Vec<CanonicalProofRecord>, StorageError> {
        store
            .proof_records_by_request_id_bounded(request_id, limit)
            .map(|rows| rows.into_iter().cloned().collect())
    }

    pub fn replay_by_session_turn(
        store: &Ph1fStore,
        session_id: SessionId,
        turn_id: TurnId,
        limit: usize,
    ) -> Result<Vec<CanonicalProofRecord>, StorageError> {
        store
            .proof_records_by_session_turn_bounded(session_id, turn_id, limit)
            .map(|rows| rows.into_iter().cloned().collect())
    }

    pub fn verify_by_request_id(
        store: &Ph1fStore,
        request_id: &str,
        limit: usize,
    ) -> Result<Vec<ProofVerificationResult>, StorageError> {
        store.verify_proof_records_by_request_id_bounded(request_id, limit)
    }
}
