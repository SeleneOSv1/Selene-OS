# PH1.COMP ECM Spec

## Engine Header
- `engine_id`: `PH1.COMP`
- `purpose`: Deterministic quantitative computation for canonical ranking, consensus, normalization, and budget math.
- `data_owned`: runtime-visible computation packet and execution state only
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `COMP_RANK_CANDIDATES`
- `name`: Rank candidate outcomes deterministically
- `input_schema`: ranked candidate input set
- `output_schema`: `ComputationPacket`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `COMP_EVALUATE_CONSENSUS`
- `name`: Evaluate deterministic weighted consensus and outlier handling
- `input_schema`: consensus signal input set
- `output_schema`: `ComputationPacket`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `COMP_COMPUTE_BUDGET_POSTURE`
- `name`: Compute deterministic budget / quota posture
- `input_schema`: budget or quota input set
- `output_schema`: `ComputationPacket`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `COMP_ATTACH_EXECUTION_STATE`
- `name`: Attach canonical computation state to the runtime execution envelope
- `input_schema`: `ComputationPacket`
- `output_schema`: `ComputationExecutionState`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- `NORMALIZATION_FAILURE`
- `COMPUTATION_OVERFLOW`
- `INVALID_INPUT_SET`
- `CONSENSUS_UNRESOLVED`
- `CONFIDENCE_BELOW_THRESHOLD`
- `BUDGET_COMPUTATION_FAILURE`
- `OUTLIER_HANDLING_FAILURE`

## Hard Boundary

PH1.COMP computes only. It must never:
- authorize protected execution
- mutate authoritative state directly
- bypass the Authority Layer, Runtime Governance Layer, or Runtime Law Engine
- become a parallel quantitative authority outside the canonical computation contract

## Sources
- `crates/selene_kernel_contracts/src/ph1comp.rs`
- `crates/selene_os/src/ph1comp.rs`
- `docs/DB_WIRING/PH1_COMP.md`
