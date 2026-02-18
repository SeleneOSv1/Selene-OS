#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1j::{AuditEvent, AuditEventId, CorrelationId};
use selene_kernel_contracts::ph1write::{
    Ph1WriteOk, Ph1WriteRefuse, Ph1WriteRequest, Ph1WriteResponse, WriteFormatMode,
};
use selene_kernel_contracts::{ContractViolation, Validate};
use selene_storage::ph1f::StorageError;
use selene_storage::repo::Ph1WriteRepo;

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.WRITE OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_WRITE_DISABLED_PASSTHROUGH: ReasonCodeId = ReasonCodeId(0x5752_8101);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1WriteWiringConfig {
    pub write_enabled: bool,
    pub audit_commit_enabled: bool,
}

impl Ph1WriteWiringConfig {
    pub fn mvp_v1(write_enabled: bool) -> Self {
        Self {
            write_enabled,
            audit_commit_enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1WriteForwarded {
    pub write_ok: Ph1WriteOk,
    pub audit_event_id: Option<AuditEventId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1WriteWiringOutcome {
    NotInvokedDisabled(Ph1WriteOk),
    Refused(Ph1WriteRefuse),
    Forwarded(Ph1WriteForwarded),
}

pub trait Ph1WriteEngine {
    fn run(&self, req: &Ph1WriteRequest) -> Ph1WriteResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1WriteWiring<E>
where
    E: Ph1WriteEngine,
{
    config: Ph1WriteWiringConfig,
    engine: E,
}

impl<E> Ph1WriteWiring<E>
where
    E: Ph1WriteEngine,
{
    pub fn new(config: Ph1WriteWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        Ok(Self { config, engine })
    }

    pub fn run_turn<R: Ph1WriteRepo>(
        &self,
        repo: &mut R,
        req: &Ph1WriteRequest,
    ) -> Result<Ph1WriteWiringOutcome, StorageError> {
        req.validate().map_err(StorageError::ContractViolation)?;

        if !self.config.write_enabled {
            return Ok(Ph1WriteWiringOutcome::NotInvokedDisabled(
                disabled_passthrough_output(req)?,
            ));
        }

        let response = self.engine.run(req);
        response
            .validate()
            .map_err(StorageError::ContractViolation)?;

        match response {
            Ph1WriteResponse::Refuse(refuse) => Ok(Ph1WriteWiringOutcome::Refused(refuse)),
            Ph1WriteResponse::Ok(write_ok) => {
                let audit_event_id = if self.config.audit_commit_enabled {
                    Some(repo.ph1write_format_commit_row(
                        req.now,
                        req.tenant_id.as_str().to_string(),
                        req.correlation_id,
                        req.turn_id,
                        req.session_id,
                        req.user_id.clone(),
                        req.device_id.clone(),
                        write_ok.format_mode.as_str().to_string(),
                        write_ok.reason_code,
                        req.idempotency_key.clone(),
                    )?)
                } else {
                    None
                };

                Ok(Ph1WriteWiringOutcome::Forwarded(Ph1WriteForwarded {
                    write_ok,
                    audit_event_id,
                }))
            }
        }
    }

    pub fn read_audit_rows<'a, R: Ph1WriteRepo>(
        &self,
        repo: &'a R,
        correlation_id: CorrelationId,
    ) -> Vec<&'a AuditEvent> {
        repo.ph1write_audit_rows(correlation_id)
    }
}

fn disabled_passthrough_output(req: &Ph1WriteRequest) -> Result<Ph1WriteOk, ContractViolation> {
    Ph1WriteOk::v1(
        req.correlation_id,
        req.turn_id,
        req.response_text.clone(),
        WriteFormatMode::FallbackOriginal,
        reason_codes::PH1_WRITE_DISABLED_PASSTHROUGH,
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1_voice_id::UserId;
    use selene_kernel_contracts::ph1j::{CorrelationId, DeviceId, TurnId};
    use selene_kernel_contracts::ph1position::TenantId;
    use selene_kernel_contracts::ph1write::{Ph1WriteOk, WriteRenderStyle};
    use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};
    use selene_storage::ph1f::{DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore};
    use selene_storage::repo::Ph1fFoundationRepo;

    #[derive(Debug, Clone)]
    struct DeterministicEngine {
        response: Ph1WriteResponse,
    }

    impl Ph1WriteEngine for DeterministicEngine {
        fn run(&self, _req: &Ph1WriteRequest) -> Ph1WriteResponse {
            self.response.clone()
        }
    }

    #[derive(Debug, Clone)]
    struct PanicEngine;

    impl Ph1WriteEngine for PanicEngine {
        fn run(&self, _req: &Ph1WriteRequest) -> Ph1WriteResponse {
            panic!("engine must not be invoked")
        }
    }

    fn user(id: &str) -> UserId {
        UserId::new(id).unwrap()
    }

    fn device(id: &str) -> DeviceId {
        DeviceId::new(id).unwrap()
    }

    fn seed_identity_device(store: &mut Ph1fStore, user_id: UserId, device_id: DeviceId) {
        store
            .insert_identity_row(IdentityRecord::v1(
                user_id.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        store
            .insert_device_row(
                DeviceRecord::v1(
                    device_id,
                    user_id,
                    "desktop".to_string(),
                    MonotonicTimeNs(1),
                    None,
                )
                .unwrap(),
            )
            .unwrap();
    }

    fn request(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        idempotency_key: &str,
    ) -> Ph1WriteRequest {
        Ph1WriteRequest::v1(
            MonotonicTimeNs(200),
            TenantId::new("tenant_a").unwrap(),
            correlation_id,
            turn_id,
            None,
            user("tenant_a:user_1"),
            device("tenant_a_device_1"),
            "  John   owes $1200 on 2026-03-01 at 3:00pm.  ".to_string(),
            WriteRenderStyle::Professional,
            vec![
                selene_kernel_contracts::ph1write::CriticalToken::new("John").unwrap(),
                selene_kernel_contracts::ph1write::CriticalToken::new("$1200").unwrap(),
            ],
            false,
            idempotency_key.to_string(),
        )
        .unwrap()
    }

    fn payload_value(event: &AuditEvent, key: &str) -> Option<String> {
        event.payload_min.entries.iter().find_map(|(k, v)| {
            if k.as_str() == key {
                Some(v.as_str().to_string())
            } else {
                None
            }
        })
    }

    #[test]
    fn at_write_wiring_01_forwards_and_commits_formatted_output() {
        let mut store = Ph1fStore::new_in_memory();
        seed_identity_device(
            &mut store,
            user("tenant_a:user_1"),
            device("tenant_a_device_1"),
        );

        let req = request(CorrelationId(8101), TurnId(1), "write-wiring-1");
        let ok = Ph1WriteOk::v1(
            req.correlation_id,
            req.turn_id,
            "John owes $1200 on 2026-03-01 at 3:00pm.".to_string(),
            WriteFormatMode::FormattedText,
            ReasonCodeId(0x5752_0001),
            true,
        )
        .unwrap();

        let wiring = Ph1WriteWiring::new(
            Ph1WriteWiringConfig::mvp_v1(true),
            DeterministicEngine {
                response: Ph1WriteResponse::Ok(ok.clone()),
            },
        )
        .unwrap();

        let out = wiring.run_turn(&mut store, &req).unwrap();
        match out {
            Ph1WriteWiringOutcome::Forwarded(fwd) => {
                assert_eq!(fwd.write_ok, ok);
                assert!(fwd.audit_event_id.is_some());
            }
            _ => panic!("expected forwarded"),
        }

        let rows = wiring.read_audit_rows(&store, req.correlation_id);
        assert_eq!(rows.len(), 1);
        assert_eq!(
            payload_value(rows[0], "directive").as_deref(),
            Some("format")
        );
        assert_eq!(
            payload_value(rows[0], "format_mode").as_deref(),
            Some("FORMATTED_TEXT")
        );
    }

    #[test]
    fn at_write_wiring_02_fallback_output_is_committed_with_fallback_mode() {
        let mut store = Ph1fStore::new_in_memory();
        seed_identity_device(
            &mut store,
            user("tenant_a:user_1"),
            device("tenant_a_device_1"),
        );

        let req = request(CorrelationId(8102), TurnId(1), "write-wiring-2");
        let ok = Ph1WriteOk::v1(
            req.correlation_id,
            req.turn_id,
            req.response_text.clone(),
            WriteFormatMode::FallbackOriginal,
            ReasonCodeId(0x5752_0002),
            true,
        )
        .unwrap();

        let wiring = Ph1WriteWiring::new(
            Ph1WriteWiringConfig::mvp_v1(true),
            DeterministicEngine {
                response: Ph1WriteResponse::Ok(ok),
            },
        )
        .unwrap();

        let out = wiring.run_turn(&mut store, &req).unwrap();
        assert!(matches!(out, Ph1WriteWiringOutcome::Forwarded(_)));

        let rows = wiring.read_audit_rows(&store, req.correlation_id);
        assert_eq!(rows.len(), 1);
        assert_eq!(
            payload_value(rows[0], "format_mode").as_deref(),
            Some("FALLBACK_ORIGINAL")
        );
    }

    #[test]
    fn at_write_wiring_03_disabled_passthrough_does_not_commit() {
        let mut store = Ph1fStore::new_in_memory();
        seed_identity_device(
            &mut store,
            user("tenant_a:user_1"),
            device("tenant_a_device_1"),
        );

        let req = request(CorrelationId(8103), TurnId(1), "write-wiring-3");
        let wiring = Ph1WriteWiring::new(
            Ph1WriteWiringConfig {
                write_enabled: false,
                audit_commit_enabled: true,
            },
            PanicEngine,
        )
        .unwrap();

        let out = wiring.run_turn(&mut store, &req).unwrap();
        match out {
            Ph1WriteWiringOutcome::NotInvokedDisabled(ok) => {
                assert_eq!(ok.format_mode, WriteFormatMode::FallbackOriginal);
                assert_eq!(ok.formatted_text, req.response_text);
                assert_eq!(ok.reason_code, reason_codes::PH1_WRITE_DISABLED_PASSTHROUGH);
            }
            _ => panic!("expected disabled passthrough"),
        }
        assert!(wiring
            .read_audit_rows(&store, req.correlation_id)
            .is_empty());
    }

    #[test]
    fn at_write_wiring_04_idempotent_retries_reuse_audit_event() {
        let mut store = Ph1fStore::new_in_memory();
        seed_identity_device(
            &mut store,
            user("tenant_a:user_1"),
            device("tenant_a_device_1"),
        );

        let req = request(CorrelationId(8104), TurnId(1), "write-wiring-4");
        let ok = Ph1WriteOk::v1(
            req.correlation_id,
            req.turn_id,
            "John owes $1200 on 2026-03-01 at 3:00pm.".to_string(),
            WriteFormatMode::FormattedText,
            ReasonCodeId(0x5752_0001),
            true,
        )
        .unwrap();

        let wiring = Ph1WriteWiring::new(
            Ph1WriteWiringConfig::mvp_v1(true),
            DeterministicEngine {
                response: Ph1WriteResponse::Ok(ok),
            },
        )
        .unwrap();

        let first = wiring.run_turn(&mut store, &req).unwrap();
        let second = wiring.run_turn(&mut store, &req).unwrap();

        let first_id = match first {
            Ph1WriteWiringOutcome::Forwarded(fwd) => fwd.audit_event_id.unwrap(),
            _ => panic!("expected forwarded"),
        };
        let second_id = match second {
            Ph1WriteWiringOutcome::Forwarded(fwd) => fwd.audit_event_id.unwrap(),
            _ => panic!("expected forwarded"),
        };

        assert_eq!(first_id, second_id);
        assert_eq!(store.ph1write_audit_rows(req.correlation_id).len(), 1);
    }
}
