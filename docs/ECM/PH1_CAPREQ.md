# PH1.CAPREQ / PH1.CAPREQ.001 ECM Spec

## Engine Header
- `engine_id`: `PH1.CAPREQ`
- `implementation_id`: `PH1.CAPREQ.001`
- `purpose`: Execute deterministic capability-request lifecycle actions and persist append-only CAPREQ ledger/current state.
- `data_owned`: `capreq_ledger`, `capreq_current`
- `version`: `v1`
- `status`: `ACTIVE`

## Family Namespace Lock
- `PH1.CAPREQ` is a family namespace.
- Active implementation ids (locked):
  - `PH1.CAPREQ.001`
- Family runtime dispatch must fail closed on unknown implementation ids.

## Capability List

### Family dispatch capability (non-persistent)

#### `PH1CAPREQ_FAMILY_RESOLVE_IMPLEMENTATION`
- `name`: Resolve CAPREQ family dispatch to an active implementation id
- `input_schema`: `implementation_id` + `Ph1CapreqRequest`
- `output_schema`: active implementation dispatch decision or fail-closed contract violation
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### Runtime lifecycle capabilities (simulation-gated)

#### `PH1CAPREQ001_EVALUATE`
- `name`: Evaluate one CAPREQ lifecycle transition deterministically (non-persistent decision step)
- `input_schema`: `Ph1CapreqRequest` + optional `current_status`
- `output_schema`: `Capreq001Decision` (`capreq_id`, `action`, `next_status`, `reason_code`, `payload_hash`)
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

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
- unknown CAPREQ family implementation ids fail closed (`ph1capreq.implementation_id` contract violation).
- deterministic success reason codes:
  - `CAPREQ_CREATED`
  - `CAPREQ_SUBMITTED`
  - `CAPREQ_APPROVED`
  - `CAPREQ_REJECTED`
  - `CAPREQ_FULFILLED`
  - `CAPREQ_CANCELED`
- scope/idempotency failures are fail-closed and auditable.

## Hard Rules
- PH1.CAPREQ.001 records governed request lifecycle truth only; it does not grant authority by itself.
- Access/approval authority remains in PH1.ACCESS.001 -> PH2.ACCESS.002.
- Simulation-gated execution is mandatory (`No Simulation -> No Execution`).
- Engines never call engines directly; cross-engine CAPREQ/ACCESS sequencing is Selene OS orchestration only.

## Related Engine Boundaries
- PH1.ACCESS consumes CAPREQ lifecycle state as governance evidence only; CAPREQ state does not equal permission grant.
- Selene OS + SimulationExecutor enforce simulation id/type match before PH1.CAPREQ execution.
- Selene OS CAPREQ family dispatcher may route only to active implementation ids.
- PH1.CAPREQ may be used by governed schema/position/onboarding rollouts, but never bypasses those engines’ own access and simulation gates.

## Audit Emission Requirements Per Capability
- lifecycle execution capabilities must emit PH1.J state-transition events with deterministic reason codes and bounded payload.
- storage projection write capabilities emit audit in replay/diagnostic mode when invoked outside normal runtime flow.

## Sources
- kernel contracts: `crates/selene_kernel_contracts/src/ph1capreq.rs`
- implementation runtime: `crates/selene_engines/src/ph1capreq.rs`
- os runtime + persistence: `crates/selene_os/src/ph1capreq.rs`
- `docs/DB_WIRING/PH1_CAPREQ.md`
