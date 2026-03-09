# PH1.LAW ECM Spec

## Engine Header
- `engine_id`: `PH1.LAW`
- `purpose`: Final deterministic runtime-law decision engine for protected completion.
- `data_owned`: runtime-law execution state and decision log
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `LAW_EVALUATE_COMPLETION`
- `name`: Evaluate final runtime-law completion posture
- `input_schema`: `RuntimeExecutionEnvelope + RuntimeLawEvaluationContext`
- `output_schema`: `Result<RuntimeExecutionEnvelope, RuntimeLawDecision>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `LAW_RECORD_DECISION`
- `name`: Record runtime-law decision state for replayable completion judgment
- `input_schema`: evaluated law inputs
- `output_schema`: `RuntimeLawDecision`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- `LAW_PROOF_REQUIRED`
- `LAW_PROOF_CHAIN_BROKEN`
- `LAW_GOVERNANCE_SAFE_MODE`
- `LAW_BUILDER_ROLLBACK_REQUIRED`
- `LAW_LEARNING_APPROVAL_REQUIRED`
- `LAW_SELF_HEAL_UNSAFE`
- `LAW_OVERRIDE_INVALID`

## Hard Boundary

PH1.LAW must never:
- replace PH1.J proof capture
- replace the Authority Layer
- replace Runtime Governance
- mutate runtime state outside the final completion decision boundary
- become a client-visible authority source

## Sources
- `crates/selene_kernel_contracts/src/runtime_law.rs`
- `crates/selene_os/src/runtime_law.rs`
- `docs/DB_WIRING/PH1_LAW.md`
