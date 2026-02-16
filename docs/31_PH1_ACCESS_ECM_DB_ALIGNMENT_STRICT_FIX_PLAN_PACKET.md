# PH1.ACCESS ECM + DB Alignment Strict Fix Plan Packet (v1)

Last updated: 2026-02-16
Owner: Selene core design + runtime
Status: STEP4_COMPLETED_PENDING_STEP5

## 1) Purpose

Close authoritative documentation parity for PH1.ACCESS AP authoring review surfaces so ECM + DB_WIRING docs match implemented contract/storage behavior exactly, without changing runtime access behavior.

## 2) Frozen Design Truth (Step 1 lock)

1. AP authoring review remains simulation-gated and audit-gated.
2. Runtime gate outcome contract remains exactly `ALLOW | DENY | ESCALATE`.
3. AP authoring review row capabilities are authoritative execution surfaces and must be explicitly documented.
4. Access schema activation lineage fields are storage truth and must be explicitly documented.
5. This packet is parity/closure work; no policy expansion and no scope creep.

## 3) Engine Ownership for this Scope

1. `PH1.ACCESS.001_PH2.ACCESS.002` owns access schema lifecycle, AP authoring review commit surfaces, and gate decision truth.
2. `PH1.F` owns persisted storage truth for access ledgers/current projections.
3. `PH1.J` owns audit-trail truth for governed access changes.
4. Selene OS orchestrates only; engines never call engines directly.

## 4) Strict 6-Step Order

### Step 1: Docs lock

Create this packet and move `docs/02_BUILD_PLAN.md` next-packet pointer.

### Step 2: ECM lock

Align `docs/ECM/PH1_ACCESS_001_PH2_ACCESS_002.md` with AP authoring review row capabilities and failure semantics.

### Step 3: DB wiring lock

Align `docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md` with authoring-review ledgers/current projections and activation-lineage fields.

### Step 4: Coverage/ownership lock

Align `docs/COVERAGE_MATRIX.md` and `docs/10_DB_OWNERSHIP_MATRIX.md` only where parity updates are required.

### Step 5: Drift sweep + acceptance proof

Run strict doc/contract/runtime parity checks and readiness audit for PH1.ACCESS AP authoring review surfaces.

### Step 6: Final proof + freeze checkpoint

From clean tree, run final proof set and commit freeze checkpoint.

## 5) Acceptance Checklist (must all be true)

1. ECM row capabilities for AP authoring review are explicitly documented.
2. DB wiring documents AP authoring review storage objects and activation-lineage fields.
3. Coverage/ownership docs do not contradict ECM/DB wiring docs.
4. No drift between docs and implemented PH1.ACCESS contract/runtime surfaces.
5. Readiness audit remains clean for this scope.

## 6) Execution Record

- Step 1: COMPLETED (2026-02-16)
- Step 2: COMPLETED (2026-02-16)
- Step 3: COMPLETED (2026-02-16)
- Step 4: COMPLETED (2026-02-16)
- Step 5: PENDING
- Step 6: PENDING

Step 1 note:
- Added this packet as canonical PH1.ACCESS ECM/DB parity closure scope (`docs/31_PH1_ACCESS_ECM_DB_ALIGNMENT_STRICT_FIX_PLAN_PACKET.md`).
- Updated `docs/02_BUILD_PLAN.md` next strict packet pointer from Packet 30 to Packet 31.
- Logged Packet 30 freeze checkpoint commit (`f75ea97`) in build-plan strict checkpoint history.

Step 2 note:
- Locked PH1.ACCESS ECM capability parity in:
  - `docs/ECM/PH1_ACCESS_001_PH2_ACCESS_002.md`
- ECM lock additions:
  - added AP authoring review row capabilities:
    - `ACCESS_AP_AUTHORING_REVIEW_CHANNEL_COMMIT_ROW`
    - `ACCESS_AP_AUTHORING_RULE_ACTION_COMMIT_ROW`
    - `ACCESS_AP_AUTHORING_CONFIRM_COMMIT_ROW`
  - corrected AP schema lifecycle row outputs to `AccessApSchemaLedgerRecord` for create/update/activate/retire capabilities.
  - documented AP authoring fail-closed semantics (bounded channel/action sets, required review state, confirmation/action requirements).
  - documented activation-lineage persistence fields (`activation_review_event_id`, `activation_rule_action_count`, `activation_rule_action_set_ref`) for activation output behavior.
- Step-2 proof:
  - `rg -n "ACCESS_AP_AUTHORING_REVIEW_CHANNEL_COMMIT_ROW|ACCESS_AP_AUTHORING_RULE_ACTION_COMMIT_ROW|ACCESS_AP_AUTHORING_CONFIRM_COMMIT_ROW|activation_review_event_id|activation_rule_action_count|activation_rule_action_set_ref" docs/ECM/PH1_ACCESS_001_PH2_ACCESS_002.md -n` -> pass
  - `rg -n "ph1access_ap_authoring_review_channel_commit_row|ph1access_ap_authoring_rule_action_commit_row|ph1access_ap_authoring_confirm_commit_row|activation_review_event_id|activation_rule_action_count|activation_rule_action_set_ref" crates/selene_storage/src/repo.rs crates/selene_storage/src/ph1f.rs -n` -> pass

Step 3 note:
- Locked PH1.ACCESS DB wiring parity in:
  - `docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md`
- DB wiring lock additions:
  - added AP authoring review storage objects to authoritative data-owned set:
    - `access_ap_authoring_review_ledger`
    - `access_ap_authoring_review_current`
    - `access_ap_rule_review_actions_ledger`
  - added AP schema activation-lineage invariants and write-field requirements:
    - `activation_review_event_id`
    - `activation_rule_action_count`
    - `activation_rule_action_set_ref`
  - added AP authoring review read dependencies, index requirements, and cross-tenant scope rules.
  - added AP authoring row write contracts:
    - `ACCESS_AP_AUTHORING_REVIEW_CHANNEL_COMMIT`
    - `ACCESS_AP_AUTHORING_RULE_ACTION_COMMIT`
    - `ACCESS_AP_AUTHORING_CONFIRM_COMMIT`
  - aligned relation/constraint section with migration-backed indexes:
    - `ux_access_ap_authoring_review_channel_idem`
    - `ux_access_ap_authoring_confirm_idem`
    - `ux_access_ap_rule_review_action_idem`
    - `ix_access_ap_authoring_review_scope_profile_version`
    - `ix_access_ap_rule_review_action_scope_profile_version`
    - `ix_access_ap_schemas_activation_review_event`
  - aligned DB wiring acceptance coverage to AP authoring tests (`AT-ACCESS-17..21`) and migration references (`0015`, `0016`).
- Step-3 proof:
  - `rg -n "access_ap_authoring_review_ledger|access_ap_authoring_review_current|access_ap_rule_review_actions_ledger|activation_review_event_id|activation_rule_action_count|activation_rule_action_set_ref" docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md -n` -> pass
  - `rg -n "CREATE TABLE IF NOT EXISTS access_ap_authoring_review_ledger|CREATE TABLE IF NOT EXISTS access_ap_authoring_review_current|CREATE TABLE IF NOT EXISTS access_ap_rule_review_actions_ledger|activation_review_event_id|activation_rule_action_count|activation_rule_action_set_ref" crates/selene_storage/migrations/0016_access_ap_authoring_review_tables.sql -n` -> pass
  - `rg -n "ph1access_ap_authoring_review_channel_commit_row|ph1access_ap_authoring_rule_action_commit_row|ph1access_ap_authoring_confirm_commit_row|activation_review_event_id|activation_rule_action_count|activation_rule_action_set_ref" crates/selene_storage/src/repo.rs crates/selene_storage/src/ph1f.rs -n` -> pass

Step 4 note:
- Locked PH1.ACCESS coverage/ownership parity in:
  - `docs/COVERAGE_MATRIX.md`
  - `docs/10_DB_OWNERSHIP_MATRIX.md`
- Coverage/ownership lock updates:
  - kept PH1.ACCESS coverage row simulation and blueprint bindings unchanged (already matching ECM/DB wiring).
  - extended PH1.ACCESS coverage blocker note to explicitly include AP authoring review + activation-lineage parity lock.
  - corrected DB ownership summary from "instances/overrides only" to full PH1.ACCESS storage scope:
    - AP schema ledger/current
    - AP authoring review ledger/current
    - AP rule-review action ledger
    - overlay/board lifecycle ledgers/current
    - board vote ledger
    - compile-lineage refs on access instances
- Step-4 proof:
  - `rg -n "^\\| PH1\\.ACCESS\\.001_PH2\\.ACCESS\\.002 \\|" docs/COVERAGE_MATRIX.md docs/10_DB_OWNERSHIP_MATRIX.md -n` -> pass
  - `rg -n "ACCESS_AP_AUTHORING_REVIEW_CHANNEL_COMMIT|ACCESS_AP_AUTHORING_RULE_ACTION_COMMIT|ACCESS_AP_AUTHORING_CONFIRM_COMMIT|activation lineage|AP authoring review" docs/COVERAGE_MATRIX.md docs/10_DB_OWNERSHIP_MATRIX.md -n` -> pass
