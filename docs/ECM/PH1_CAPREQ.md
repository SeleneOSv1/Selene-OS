# PH1.CAPREQ ECM Spec

## Engine Header
- `engine_id`: `PH1.CAPREQ`
- `purpose`: Execute deterministic capability-request lifecycle actions and persist append-only CAPREQ ledger/current state.
- `data_owned`: `capreq_ledger`, `capreq_current`
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### Runtime lifecycle capabilities (simulation-gated)

#### `PH1CAPREQ_CREATE_DRAFT_EXECUTE`
- `name`: Execute create-draft lifecycle action
- `input_schema`: `Ph1CapreqRequest::create_draft_v1`
- `output_schema`: `Result<Ph1CapreqResponse, StorageError>`
- `allowed_callers`: `SELENE_OS_SIMULATION_EXECUTOR_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

#### `PH1CAPREQ_SUBMIT_FOR_APPROVAL_EXECUTE`
- `name`: Execute submit-for-approval lifecycle action
- `input_schema`: `Ph1CapreqRequest::submit_for_approval_commit_v1`
- `output_schema`: `Result<Ph1CapreqResponse, StorageError>`
- `allowed_callers`: `SELENE_OS_SIMULATION_EXECUTOR_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

#### `PH1CAPREQ_APPROVE_EXECUTE`
- `name`: Execute approve lifecycle action
- `input_schema`: `Ph1CapreqRequest::approve_commit_v1`
- `output_schema`: `Result<Ph1CapreqResponse, StorageError>`
- `allowed_callers`: `SELENE_OS_SIMULATION_EXECUTOR_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

#### `PH1CAPREQ_REJECT_EXECUTE`
- `name`: Execute reject lifecycle action
- `input_schema`: `Ph1CapreqRequest::reject_commit_v1`
- `output_schema`: `Result<Ph1CapreqResponse, StorageError>`
- `allowed_callers`: `SELENE_OS_SIMULATION_EXECUTOR_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

#### `PH1CAPREQ_FULFILL_EXECUTE`
- `name`: Execute fulfill lifecycle action
- `input_schema`: `Ph1CapreqRequest::fulfill_commit_v1`
- `output_schema`: `Result<Ph1CapreqResponse, StorageError>`
- `allowed_callers`: `SELENE_OS_SIMULATION_EXECUTOR_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

#### `PH1CAPREQ_CANCEL_REVOKE_EXECUTE`
- `name`: Execute cancel/revoke lifecycle action
- `input_schema`: `Ph1CapreqRequest::cancel_revoke_v1`
- `output_schema`: `Result<Ph1CapreqResponse, StorageError>`
- `allowed_callers`: `SELENE_OS_SIMULATION_EXECUTOR_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### Storage projection capabilities

#### `PH1CAPREQ_APPEND_LEDGER_ROW`
- `name`: Append CAPREQ ledger event
- `input_schema`: `CapabilityRequestLedgerEventInput`
- `output_schema`: `Result<capreq_event_id, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (through runtime)
- `side_effects`: `DECLARED (DB_WRITE)`

#### `PH1CAPREQ_READ_LEDGER_ROWS`
- `name`: Read CAPREQ ledger rows
- `input_schema`: `none`
- `output_schema`: `CapabilityRequestLedgerEvent[]`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

#### `PH1CAPREQ_READ_CURRENT_ROWS`
- `name`: Read CAPREQ current projection map
- `input_schema`: `none`
- `output_schema`: `Map<(tenant_id, capreq_id), CapabilityRequestCurrentRecord>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

#### `PH1CAPREQ_READ_CURRENT_ROW`
- `name`: Read one CAPREQ current projection row
- `input_schema`: `(tenant_id, capreq_id)`
- `output_schema`: `Option<CapabilityRequestCurrentRecord>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

#### `PH1CAPREQ_REBUILD_CURRENT_ROWS`
- `name`: Rebuild CAPREQ current projection from append-only ledger
- `input_schema`: `none`
- `output_schema`: `unit`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE_CURRENT_PROJECTION)`

## Failure Modes + Reason Codes
- deterministic lifecycle transition failures are fail-closed (`capreq_transition` contract violations).
- deterministic action success reason codes:
  - `CAPREQ_OK_CREATE_DRAFT`
  - `CAPREQ_OK_SUBMIT_FOR_APPROVAL`
  - `CAPREQ_OK_APPROVE`
  - `CAPREQ_OK_REJECT`
  - `CAPREQ_OK_FULFILL`
  - `CAPREQ_OK_CANCEL`
- scope/idempotency failures are fail-closed and auditable.

## Hard Rules
- PH1.CAPREQ records governed request lifecycle truth only; it does not grant authority by itself.
- Access/approval authority remains in PH1.ACCESS + AP policy flow; Selene OS must gate governed execution on Access outcomes.
- Engines never call engines directly; cross-engine CAPREQ/ACCESS sequencing is Selene OS orchestration only.

## Audit Emission Requirements Per Capability
- lifecycle execution capabilities must emit PH1.J state-transition events with deterministic reason codes and bounded payload.
- storage projection write capabilities emit audit in replay/diagnostic mode when invoked outside normal runtime flow.

## Sources
- `crates/selene_os/src/ph1capreq.rs` (`Ph1CapreqRuntime`)
- `crates/selene_storage/src/repo.rs` (`Ph1CapreqRepo`)
- `docs/DB_WIRING/PH1_CAPREQ.md`
