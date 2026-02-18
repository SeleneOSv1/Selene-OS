# PH1.LEARN / PH1.FEEDBACK / PH1.KNOW ECM Spec

## Engine Header
- `engine_id`: `PH1.LEARN_FEEDBACK_KNOW` (storage grouping only; not a runtime callable engine)
- `purpose`: Persist deterministic feedback signals and versioned learning/dictionary artifacts using append-only audit/artifact ledgers.
- `data_owned`: `audit_events` writes in PH1.FEEDBACK scope, `artifacts_ledger` writes in PH1.LEARN/PH1.KNOW scope
- `version`: `v1`
- `status`: `ACTIVE`
- runtime ownership lock:
  - runtime callable contracts live in `PH1.FEEDBACK`, `PH1.LEARN`, and `PH1.KNOW` docs.
  - this ECM page defines grouped persistence capabilities only.

## Capability List

### `PH1FEEDBACK_EVENT_COMMIT_ROW`
- `name`: Commit feedback signal event
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, feedback_event_type, signal_bucket, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1LEARN_ARTIFACT_COMMIT_ROW`
- `name`: Commit learning artifact package row
- `input_schema`: `(now, tenant_id, scope_type, scope_id, artifact_type, artifact_version, package_hash, payload_ref, provenance_ref, status, idempotency_key)`
- `output_schema`: `Result<artifact_id, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`
- artifact ownership rule:
  - allowed artifact types are `STT_ROUTING_POLICY_PACK | STT_ADAPTATION_PROFILE | TTS_ROUTING_POLICY_PACK` only.

### `PH1KNOW_DICTIONARY_PACK_COMMIT_ROW`
- `name`: Commit tenant dictionary/pronunciation pack artifact row
- `input_schema`: `(now, tenant_id, artifact_type, artifact_version, package_hash, payload_ref, provenance_ref, idempotency_key)`
- `output_schema`: `Result<artifact_id, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`
- artifact ownership rule:
  - allowed artifact types are `STT_VOCAB_PACK | TTS_PRONUNCIATION_PACK` only.

### `PH1FEEDBACK_READ_AUDIT_ROWS`
- `name`: Read feedback audit rows for one correlation thread
- `input_schema`: `correlation_id`
- `output_schema`: `AuditEvent[]`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1LEARN_READ_ARTIFACT_ROWS`
- `name`: Read learning artifact rows by scope/type
- `input_schema`: `(scope_type, scope_id, artifact_type)`
- `output_schema`: `ArtifactLedgerRow[]`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1KNOW_READ_ARTIFACT_ROWS`
- `name`: Read tenant dictionary artifact rows by tenant/type
- `input_schema`: `(tenant_id, artifact_type)`
- `output_schema`: `ArtifactLedgerRow[]`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- deterministic reason-coded failures include:
  - tenant/scope validation failure
  - artifact type/scope mismatch
  - idempotency replay/no-op
- all failures are fail-closed and auditable.

## Audit Emission Requirements Per Capability
- write capabilities emit PH1.J with bounded payload and deterministic reason code.
- read capabilities emit audit only in replay/diagnostic mode.

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1LearnFeedbackKnowRepo`)
- `docs/DB_WIRING/PH1_LEARN_FEEDBACK_KNOW.md`

## Related Engine Boundary (`PH1.FEEDBACK`)
- Runtime FEEDBACK capability contracts are defined in:
  - `docs/DB_WIRING/PH1_FEEDBACK.md`
  - `docs/ECM/PH1_FEEDBACK.md`
- This combined ECM remains authoritative for append-only persistence capabilities (`PH1FEEDBACK_EVENT_COMMIT_ROW`, `PH1LEARN_ARTIFACT_COMMIT_ROW`, `PH1KNOW_DICTIONARY_PACK_COMMIT_ROW`).

## Related Engine Boundary (`PH1.LEARN`)
- Runtime LEARN capability contracts are defined in:
  - `docs/DB_WIRING/PH1_LEARN.md`
  - `docs/ECM/PH1_LEARN.md`
- This combined ECM remains authoritative for persistence capabilities only (`PH1LEARN_ARTIFACT_COMMIT_ROW` append-only artifact writes).

## Related Engine Boundary (`PH1.KG`)
- PH1.KG may consume PH1.KNOW artifacts only through Selene OS tenant-scoped routing and only as bounded seed metadata.
- PH1.KG runtime contracts remain responsible for evidence-backed, tenant-scope-preserved, no-guessing relationship output.

## Related Engine Boundary (`PH1.KNOW`)

- Runtime PH1.KNOW capability contracts are defined in:
  - `docs/DB_WIRING/PH1_KNOW.md`
  - `docs/ECM/PH1_KNOW.md`
- This combined ECM remains authoritative for persistence capabilities only (`PH1KNOW_DICTIONARY_PACK_COMMIT_ROW` append-only artifact writes).
