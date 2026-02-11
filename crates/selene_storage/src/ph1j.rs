#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1j::{AuditEventId, AuditEventInput};

use crate::ph1f::{Ph1fStore, StorageError};

/// PH1.J (Audit Engine) runtime wrapper.
///
/// In MVP skeleton form, PH1.J is a disciplined append-only writer into PH1.F's `audit_events` ledger.
#[derive(Debug, Default)]
pub struct Ph1jRuntime;

impl Ph1jRuntime {
    pub fn emit(
        store: &mut Ph1fStore,
        input: AuditEventInput,
    ) -> Result<AuditEventId, StorageError> {
        store.append_audit_event(input)
    }
}
