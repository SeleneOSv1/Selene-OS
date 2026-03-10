# PH1_GOV DB Wiring (Design vNext)

## Phase A Artifact Trust Closure Alignment (2026-03-10)
- For the Phase A artifact-trust stack, PH1.GOV consumes canonical A3/A4 trust surfaces only:
  - `RuntimeExecutionEnvelope.artifact_trust_state`
  - ordered `ArtifactTrustDecisionRecord`
  - per-artifact `proof_entry_ref`
  - turn-level `proof_record_ref`
  - `trust_policy_snapshot_ref`
  - `trust_set_snapshot_ref`
  - `verification_basis_fingerprint`
  - `negative_verification_result_ref` where present
- `artifact_hash_sha256`, `signature_ref`, raw proof fragments, adapter hints, and PH1.OS hints are legacy compatibility metadata only and are not lawful trust-enforcement truth for the Phase A stack.
- Runtime closure proof for this surface lives in:
  - `crates/selene_os/src/runtime_governance.rs`
  - tests `at_runtime_gov_07` through `at_runtime_gov_11`

## A) Engine Header
- engine_id: PH1.GOV
- layer: Enterprise Support
- authority: Authoritative (definition governance decision only)
- role: Deterministic governance gate for activation/deprecation/rollback of blueprint, simulation, and capability-map artifacts (`GOV_POLICY_EVALUATE -> GOV_DECISION_COMPUTE`)
- placement: ENTERPRISE_SUPPORT (OS-internal governance path)

## B) Ownership
- Tables owned: NONE in current runtime slice (decision-only runtime)
- Reads:
  - canonical runtime envelope governance context
  - canonical artifact trust decision ids, proof refs, trust snapshot refs, and basis fingerprints
  - ACTIVE reference ids (required blueprint/simulation/capability bindings)
  - optional rollback target version
- Writes:
  - no direct table writes in this runtime slice
  - emits deterministic governance decisions plus canonical artifact-trust linkage state only

## C) Hard Boundaries
- Must never execute workflows, tools, or simulations.
- Must never allow activation when required references are missing or inactive.
- Must enforce one ACTIVE blueprint per `intent_type` when configured.
- Must require enterprise signature proof in signature-required mode.
- Must keep decisions deterministic, reason-coded, and audit-required.

## D) Wiring
- Invoked_by: Selene OS governance path before any artifact activation/deprecation/rollback commit.
- Inputs_from:
  - governance request metadata (`artifact_kind`, `artifact_id`, `artifact_version`, `requested_action`)
  - canonical artifact trust state (`artifact_trust_state.decision_records`)
  - canonical proof linkage (`proof_entry_ref`, `proof_record_ref`)
  - trust snapshot linkage (`trust_policy_snapshot_ref`, `trust_set_snapshot_ref`)
  - verification basis linkage (`verification_basis_fingerprint`, `negative_verification_result_ref`)
  - reference activity snapshot (`required_reference_ids`, `active_reference_ids`)
  - current/rollback state (`existing_active_versions`, `current_active_version`, `rollback_target_version`)
- Outputs_to:
  - `gov_policy_bundle` (`requester_authorized`, `signature_valid`, `references_active`, `single_active_blueprint_ok`)
  - `gov_decision_bundle` (`decision`, `active_version`, `reason_code`, deterministic/audit flags)
  - canonical governance execution state (`artifact_trust_decision_ids`, `artifact_trust_proof_entry_refs`, `artifact_trust_proof_record_ref`, `artifact_trust_policy_snapshot_refs`, `artifact_trust_set_snapshot_refs`)
- Invocation_condition: ENTERPRISE_SUPPORT (governance enabled)
- Deterministic sequence:
  - `GOV_POLICY_EVALUATE`:
    - validates authorization, signature, reference completeness, and single-active rules.
    - emits deterministic policy booleans and reason code.
  - `GOV_DECISION_COMPUTE`:
    - emits exactly one governance result (`ALLOWED` or `BLOCKED`).
    - computes deterministic resulting active version for `ACTIVATE | DEPRECATE | ROLLBACK`.
    - enforces non-execution boundary (`no_execution_authority=true`).
- Not allowed:
  - engine-to-engine direct calls
  - hidden procedure activation
  - bypassing audit/trace requirements

## E) Related Engine Boundaries
- `PH1.TENANT`: tenant scope and policy context are upstream prerequisites; PH1.GOV consumes resolved tenant context and must not guess tenant scope.
- `PBS_TABLES`: blueprint registry persistence remains append-only/current-projection truth; PH1.GOV governs activation/deprecation/rollback decisions before those state transitions are committed.
- `SIMULATION_CATALOG_TABLES`: simulation catalog activation lineage remains table-owned; PH1.GOV decides whether a requested transition is allowed.
- `ENGINE_CAPABILITY_MAPS_TABLES`: callable capability bindings remain table-owned; PH1.GOV blocks transitions when reference integrity is broken.
- `PH1.J`: every governance transition decision is reason-coded and audit-required.

## F) Acceptance Tests
- AT-GOV-01: cannot activate when references are missing.
- AT-GOV-02: single ACTIVE blueprint rule is enforced.
- AT-GOV-03: activation is blocked when enterprise signature is invalid/missing.
- AT-GOV-04: rollback decision is deterministic and auditable.
- AT-GOV-A5-01: missing canonical artifact trust state blocks artifact activation.
- AT-GOV-A5-02: cluster trust divergence quarantines artifact activation.
- AT-GOV-A5-03: proof-required activation blocks when canonical proof linkage is incomplete.
- AT-GOV-A5-04: turn-level proof linkage does not substitute for per-artifact `proof_entry_ref`.
- AT-GOV-A5-05: verified artifact activation records canonical trust/proof linkage deterministically.
