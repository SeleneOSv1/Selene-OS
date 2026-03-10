# PH1.LAW DB Wiring Spec

## Phase A Artifact Trust Closure Alignment (2026-03-10)
- For the Phase A artifact-trust stack, PH1.LAW consumes canonical A3/A4 outputs only:
  - `RuntimeExecutionEnvelope.artifact_trust_state`
  - ordered `ArtifactTrustDecisionRecord`
  - per-artifact `proof_entry_ref`
  - turn-level `proof_record_ref`
  - governance-produced canonical artifact-trust linkage
- PH1.LAW does not consume raw `artifact_hash_sha256`, `signature_ref`, raw proof fragments, adapter hints, or PH1.OS hints as final trust inputs.
- Runtime closure proof for this surface lives in:
  - `crates/selene_os/src/runtime_law.rs`
  - tests `at_runtime_law_10` through `at_runtime_law_14`

## 1) Engine Header

- `engine_id`: `PH1.LAW`
- `purpose`: Final runtime law engine for deterministic protected-execution completion judgment.
- `version`: `v1`
- `status`: `ACTIVE`

## 2) Ownership

### PH1.LAW owns
- final runtime law rule registry
- final runtime law decision model
- final runtime law execution state carried by the runtime execution envelope
- replayable runtime-law decision log maintained by the runtime law subsystem

### PH1.LAW does not own standalone persistent tables in the current slice
- no dedicated PH1.LAW database tables are defined in the current repository slice
- no direct authoritative state mutation outside governed completion decisions

### PH1.F owns
- any persistence substrate used by callers that store envelopes or decision references

## 3) Reads (dependencies)

### runtime law inputs
- reads `RuntimeExecutionEnvelope` state produced by upstream runtime layers, including:
  - session / persistence / governance / proof / computation / identity / memory / authority state
- reads canonical artifact trust linkage carried by the envelope:
  - `artifact_trust_state`
  - `artifact_trust_decision_ids`
  - `artifact_trust_proof_entry_refs`
  - `artifact_trust_proof_record_ref`
  - `trust_policy_snapshot_ref`
  - `trust_set_snapshot_ref`
- reads builder / learning / self-heal / override inputs supplied by the live protected path

## 4) Writes (outputs)

### runtime output
- writes no standalone database rows directly
- emits:
  - `RuntimeLawExecutionState`
  - `RuntimeLawDecision`
  - final response-class decision bound to the current envelope

### runtime envelope integration
- `RuntimeExecutionEnvelope.law_state`
- law reason codes
- law policy version
- override / rollback / blast-radius / dry-run state

## 5) Invariants

- identical law inputs must produce identical final law outputs
- protected completion must not be considered lawful without final PH1.LAW judgment
- proof-required actions must fail closed when PH1.J proof posture is insufficient
- proof-required artifact-trust decisions must treat `proof_entry_ref` as canonical per-artifact linkage and must not treat `proof_record_ref` as an interchangeable substitute
- runtime law must not become a parallel business-action engine

## 6) Acceptance Tests (DB Wiring / Runtime Proof)

Required proof coverage:
- conflicting subsystem inputs resolve deterministically
- proof failure can force `BLOCK`, `QUARANTINE`, or `SAFE_MODE`
- builder deployment without rollback readiness is blocked
- learning promotion without law approval is blocked
- self-heal remediation without safe posture is blocked
- dry-run predicts law outcome without executing
- override path requires controlled state
- blast-radius containment works deterministically
- final law decision is recorded deterministically
- artifact-trust authority without canonical trust state is blocked
- cluster trust divergence can force `SAFE_MODE`
- proof-required artifact authority without canonical per-artifact proof linkage is blocked
- verified artifact authority records canonical trust/proof linkage deterministically

Implemented references:
- contracts: `crates/selene_kernel_contracts/src/runtime_law.rs`
- runtime: `crates/selene_os/src/runtime_law.rs`
- envelope integration: `crates/selene_kernel_contracts/src/runtime_execution.rs`
- live protected paths:
  - `crates/selene_os/src/app_ingress.rs`
  - `crates/selene_os/src/ph1builder.rs`
  - `crates/selene_os/src/ph1learn.rs`
  - `crates/selene_os/src/ph1os.rs`
