# PH1.COMP DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.COMP`
- `purpose`: Canonical deterministic quantitative computation engine for ranking, consensus, normalization, and budget math.
- `version`: `v1`
- `status`: `ACTIVE`

## 2) Ownership

### PH1.COMP owns
- canonical `ComputationPacket` contract shape
- deterministic scoring and ranking math
- deterministic consensus and outlier math
- canonical normalization rules for currency, unit, time, percentage, and scale alignment
- canonical computation execution state attached to the runtime execution envelope

### PH1.COMP does not own standalone persistent tables in the current slice
- no authoritative PH1.COMP-specific database tables
- no direct storage writes as part of normal computation
- no alternate state-mutation path

### PH1.F owns
- any storage substrate used by callers that persist PH1.COMP outputs or references

## 3) Reads (dependencies)

### runtime and engine inputs
- reads caller-supplied computation inputs only
- reads runtime execution envelope context where computation state is attached
- reads deterministic candidate, consensus, and budget inputs from calling engines

Required caller discipline:
- callers must supply normalized input values or explicit normalization metadata
- callers must not treat nondeterministic local math as final quantitative authority

## 4) Writes (outputs)

### runtime output
- writes no database rows directly
- emits:
  - `ComputationPacket`
  - `ComputationExecutionState`
  - deterministic failure classifications when computation fails

### runtime envelope integration
- `RuntimeExecutionEnvelope.computation_state`
- packet reference / normalization trace / confidence posture / failure class

## 5) Invariants

- identical inputs must produce identical outputs
- final PH1.COMP outputs must remain deterministic and replayable
- safe rounding and threshold comparisons must be canonical
- computation failure must fail closed where quantitative correctness is required

## 6) Acceptance Tests (DB Wiring / Runtime Proof)

Required proof coverage:
- identical inputs produce identical `ComputationPacket` outputs
- deterministic tie-breaking works
- weighted consensus works
- outlier handling is deterministic
- heterogeneous normalization produces canonical values
- budget/quota computation is deterministic
- computation failure classes surface correctly
- computation state attaches to the runtime execution envelope

Implemented references:
- contracts: `crates/selene_kernel_contracts/src/ph1comp.rs`
- runtime: `crates/selene_os/src/ph1comp.rs`
- envelope integration: `crates/selene_kernel_contracts/src/runtime_execution.rs`
- tests: `crates/selene_os/src/ph1comp.rs`
