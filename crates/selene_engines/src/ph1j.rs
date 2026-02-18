#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use selene_kernel_contracts::ph1j::{AuditEvent, AuditEventId, AuditEventInput, CorrelationId};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.J reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_J_OK_APPEND: ReasonCodeId = ReasonCodeId(0x4A00_0001);
    pub const J_CONTRACT_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4A00_00F1);
    pub const J_APPEND_ONLY_VIOLATION: ReasonCodeId = ReasonCodeId(0x4A00_00F2);
    pub const J_IDEMPOTENCY_REPLAY: ReasonCodeId = ReasonCodeId(0x4A00_00F3);
    pub const J_TENANT_SCOPE_VIOLATION: ReasonCodeId = ReasonCodeId(0x4A00_00F4);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1jConfig {
    pub max_events: usize,
}

impl Ph1jConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_events: 200_000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1jRuntime {
    config: Ph1jConfig,
    events: Vec<AuditEvent>,
    next_event_id: u64,
    idempotency_index_scoped: BTreeMap<(String, String, String), AuditEventId>,
    idempotency_index_legacy: BTreeMap<(CorrelationId, String), AuditEventId>,
}

impl Ph1jRuntime {
    pub fn new(config: Ph1jConfig) -> Self {
        Self {
            config,
            events: Vec::new(),
            next_event_id: 1,
            idempotency_index_scoped: BTreeMap::new(),
            idempotency_index_legacy: BTreeMap::new(),
        }
    }

    pub fn append_audit_row(
        &mut self,
        input: AuditEventInput,
    ) -> Result<AuditEventId, ContractViolation> {
        input.validate()?;

        if self.events.len() >= self.config.max_events {
            return Err(ContractViolation::InvalidValue {
                field: "ph1j_runtime.events",
                reason: "max_events exceeded",
            });
        }

        if let Some(existing_event_id) = self.idempotency_replay_match(&input) {
            return Ok(existing_event_id);
        }

        let event_id = AuditEventId(self.next_event_id);
        self.next_event_id = self.next_event_id.saturating_add(1);
        let event = AuditEvent::from_input_v1(event_id, input.clone())?;

        self.index_idempotency(&input, event_id);
        self.events.push(event);
        Ok(event_id)
    }

    pub fn audit_rows(&self) -> &[AuditEvent] {
        &self.events
    }

    pub fn audit_rows_by_correlation(&self, correlation_id: CorrelationId) -> Vec<AuditEvent> {
        self.events
            .iter()
            .filter(|e| e.correlation_id == correlation_id)
            .cloned()
            .collect()
    }

    pub fn audit_rows_by_tenant(
        &self,
        tenant_id: &str,
    ) -> Result<Vec<AuditEvent>, ContractViolation> {
        validate_tenant_id("ph1j_runtime.tenant_id", tenant_id)?;
        Ok(self
            .events
            .iter()
            .filter(|e| e.tenant_id.as_deref() == Some(tenant_id))
            .cloned()
            .collect())
    }

    fn idempotency_replay_match(&self, input: &AuditEventInput) -> Option<AuditEventId> {
        let key = input.idempotency_key.as_ref()?;
        if let (Some(tenant_id), Some(work_order_id)) = (&input.tenant_id, &input.work_order_id) {
            return self
                .idempotency_index_scoped
                .get(&(tenant_id.clone(), work_order_id.clone(), key.clone()))
                .copied();
        }
        self.idempotency_index_legacy
            .get(&(input.correlation_id, key.clone()))
            .copied()
    }

    fn index_idempotency(&mut self, input: &AuditEventInput, event_id: AuditEventId) {
        let Some(key) = &input.idempotency_key else {
            return;
        };
        if let (Some(tenant_id), Some(work_order_id)) = (&input.tenant_id, &input.work_order_id) {
            self.idempotency_index_scoped.insert(
                (tenant_id.clone(), work_order_id.clone(), key.clone()),
                event_id,
            );
            return;
        }
        self.idempotency_index_legacy
            .insert((input.correlation_id, key.clone()), event_id);
    }
}

fn validate_tenant_id(field: &'static str, tenant_id: &str) -> Result<(), ContractViolation> {
    if tenant_id.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be non-empty",
        });
    }
    if tenant_id.len() > 64 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be <= 64 chars",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1j::{
        AuditEngine, AuditEventType, AuditPayloadMin, AuditSeverity, PayloadKey, PayloadValue,
        TurnId,
    };
    use selene_kernel_contracts::MonotonicTimeNs;
    use selene_kernel_contracts::ReasonCodeId;

    fn payload_gate(gate: &str) -> AuditPayloadMin {
        let mut entries = BTreeMap::new();
        entries.insert(
            PayloadKey::new("gate").unwrap(),
            PayloadValue::new(gate).unwrap(),
        );
        AuditPayloadMin::v1(entries).unwrap()
    }

    fn event_input(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        tenant_id: Option<&str>,
        work_order_id: Option<&str>,
        idempotency_key: Option<&str>,
    ) -> AuditEventInput {
        AuditEventInput::v1(
            MonotonicTimeNs(1_000),
            tenant_id.map(ToString::to_string),
            work_order_id.map(ToString::to_string),
            None,
            None,
            None,
            AuditEngine::Ph1J,
            AuditEventType::GatePass,
            ReasonCodeId(0x1001),
            AuditSeverity::Info,
            correlation_id,
            turn_id,
            payload_gate("policy"),
            None,
            idempotency_key.map(ToString::to_string),
        )
        .unwrap()
    }

    #[test]
    fn at_j_01_every_gate_emits_an_audit_event() {
        let mut rt = Ph1jRuntime::new(Ph1jConfig::mvp_v1());
        let corr = CorrelationId(9101);
        let _ = rt
            .append_audit_row(event_input(corr, TurnId(1), None, None, None))
            .unwrap();
        let _ = rt
            .append_audit_row(event_input(corr, TurnId(2), None, None, None))
            .unwrap();
        let _ = rt
            .append_audit_row(event_input(corr, TurnId(3), None, None, None))
            .unwrap();

        let rows = rt.audit_rows_by_correlation(corr);
        assert_eq!(rows.len(), 3);
        assert!(rows
            .iter()
            .all(|r| r.event_type == AuditEventType::GatePass));
    }

    #[test]
    fn at_j_02_append_only_audit_events() {
        let mut rt = Ph1jRuntime::new(Ph1jConfig::mvp_v1());
        let corr = CorrelationId(9102);
        let first = rt
            .append_audit_row(event_input(corr, TurnId(1), None, None, None))
            .unwrap();
        let second = rt
            .append_audit_row(event_input(corr, TurnId(2), None, None, None))
            .unwrap();
        let third = rt
            .append_audit_row(event_input(corr, TurnId(3), None, None, None))
            .unwrap();

        assert!(first.0 < second.0 && second.0 < third.0);
        assert_eq!(rt.audit_rows().len(), 3);
    }

    #[test]
    fn at_j_03_idempotency_dedupe_works_scoped() {
        let mut rt = Ph1jRuntime::new(Ph1jConfig::mvp_v1());
        let corr = CorrelationId(9103);
        let a = rt
            .append_audit_row(event_input(
                corr,
                TurnId(1),
                Some("tenant_a"),
                Some("wo_1"),
                Some("dup"),
            ))
            .unwrap();
        let b = rt
            .append_audit_row(event_input(
                corr,
                TurnId(1),
                Some("tenant_a"),
                Some("wo_1"),
                Some("dup"),
            ))
            .unwrap();

        assert_eq!(a, b);
        assert_eq!(rt.audit_rows().len(), 1);
    }

    #[test]
    fn at_j_04_idempotency_dedupe_works_legacy_fallback() {
        let mut rt = Ph1jRuntime::new(Ph1jConfig::mvp_v1());
        let corr = CorrelationId(9104);
        let a = rt
            .append_audit_row(event_input(corr, TurnId(1), None, None, Some("legacy_dup")))
            .unwrap();
        let b = rt
            .append_audit_row(event_input(corr, TurnId(2), None, None, Some("legacy_dup")))
            .unwrap();

        assert_eq!(a, b);
        assert_eq!(rt.audit_rows().len(), 1);
    }

    #[test]
    fn at_j_05_tenant_query_filters_scope() {
        let mut rt = Ph1jRuntime::new(Ph1jConfig::mvp_v1());
        let _ = rt
            .append_audit_row(event_input(
                CorrelationId(9201),
                TurnId(1),
                Some("tenant_a"),
                Some("wo_1"),
                None,
            ))
            .unwrap();
        let _ = rt
            .append_audit_row(event_input(
                CorrelationId(9202),
                TurnId(2),
                Some("tenant_b"),
                Some("wo_2"),
                None,
            ))
            .unwrap();

        let tenant_a_rows = rt.audit_rows_by_tenant("tenant_a").unwrap();
        assert_eq!(tenant_a_rows.len(), 1);
        assert_eq!(tenant_a_rows[0].tenant_id.as_deref(), Some("tenant_a"));
    }
}
