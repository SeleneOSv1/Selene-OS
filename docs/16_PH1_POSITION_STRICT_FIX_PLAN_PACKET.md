# PH1.POSITION Strict Fix Plan Packet

Last updated: 2026-02-15
Owner: Selene core design + runtime
Status: STEP7_COMPLETED_PENDING_STEP8

## 1) Purpose

This packet is the single execution plan for closing PH1.POSITION schema-gating drift.

It defines:
- Exact file-by-file patch order.
- Step gates that must be checked before each code step.
- Acceptance checks that must pass before moving to the next step.

Do not skip steps.
Do not patch files outside the listed step scope.

## 2) Scope (what this packet fixes)

1. `change_reason` is accepted in contract but ignored in storage update commit path.
2. `apply_scope` is accepted in contract but ignored in storage activate commit path.
3. Conditional required rules are not evaluated as predicates.
4. PH1.POSITION typed repo trait does not include requirements-schema methods.
5. DB wiring promises AT-05/AT-06 tests that do not exist yet.
6. Idempotency semantics drift between docs and implementation.
7. Position schema precondition language drifts from implementation.
8. ONB still has hardcoded sender-verification field aliases (must be schema-driven).

## 3) Execution Rules

1. Execute one step at a time.
2. Do not start the next step until current step acceptance checks pass.
3. If a check fails, stop and fix within the same step scope.
4. Keep simulation-gating invariant intact (`No Simulation -> No Execution`).
5. Keep engine orchestration invariant intact (engines do not call engines directly).

## 4) Baseline Gate (must run before Step 1)

Run:

```bash
cd "$(git rev-parse --show-toplevel)"
git rev-parse HEAD
git status --short
rg -n "_change_reason|_apply_scope|PositionRequirementRuleType::Always \\| PositionRequirementRuleType::Conditional" crates/selene_storage/src/ph1f.rs
rg -n "at_position_db_05|at_position_db_06" crates/selene_storage/tests/ph1_position/db_wiring.rs docs/DB_WIRING/PH1_POSITION.md
rg -n "trait Ph1PositionRepo|ph1position_requirements_schema" crates/selene_storage/src/repo.rs
```

Expected baseline evidence:
- `_change_reason` and `_apply_scope` are present in storage.
- `docs/DB_WIRING/PH1_POSITION.md` references AT-05/AT-06.
- test file does not yet contain AT-05/AT-06 bodies.
- `Ph1PositionRepo` trait exists but lacks requirements-schema methods.

## 5) Patch Order

### Step 1: Docs Lock (design semantics first)

Patch files (only):
1. `docs/DB_WIRING/PH1_POSITION.md`
2. `docs/ECM/PH1_POSITION.md`
3. `docs/BLUEPRINTS/ONB_SCHEMA_MANAGE.md`
4. `docs/BLUEPRINTS/ONB_REQUIREMENT_BACKFILL.md`
5. `docs/04_KERNEL_CONTRACTS.md` (only if wording parity is needed)

Pre-change gate:

```bash
rg -n "schema_change_set_hash|existing active position scope|active_for_new_hires|change_reason|apply_scope" docs/DB_WIRING/PH1_POSITION.md docs/ECM/PH1_POSITION.md docs/BLUEPRINTS/ONB_SCHEMA_MANAGE.md
```

Patch intent:
- Make docs match target runtime behavior:
  - `change_reason` is persisted and auditable.
  - `apply_scope` is enforced at activation (`NEW_HIRES_ONLY | CURRENT_AND_NEW`).
  - conditional required rules are predicate-evaluated, not treated as always-required.
  - idempotency rule is explicit and consistent with selected implementation.
  - schema activation for `CURRENT_AND_NEW` explicitly triggers backfill orchestration path.
- Keep ownership split clear:
  - PH1.POSITION owns requirements schema truth.
  - PH1.ONB executes schema and owns backfill campaign flow.

Post-step acceptance:

```bash
rg -n "change_reason.*auditable|apply_scope.*NEW_HIRES_ONLY|apply_scope.*CURRENT_AND_NEW|predicate|deterministic" docs/DB_WIRING/PH1_POSITION.md docs/ECM/PH1_POSITION.md
rg -n "CURRENT_AND_NEW|ONB_REQUIREMENT_BACKFILL" docs/BLUEPRINTS/ONB_SCHEMA_MANAGE.md docs/BLUEPRINTS/ONB_REQUIREMENT_BACKFILL.md
```

Step 1 exit criteria:
- All docs align on one definition of apply-scope, idempotency, and predicate semantics.

---

### Step 2: Kernel Contract Delta

Patch files (only):
1. `crates/selene_kernel_contracts/src/ph1position.rs`

Pre-change gate:

```bash
rg -n "PositionRequirementsSchemaLifecycleResult|active_for_new_hires|PositionSchemaApplyScope|change_reason" crates/selene_kernel_contracts/src/ph1position.rs
```

Patch intent:
- Make lifecycle result contract represent rollout scope explicitly.
- Ensure request/response contract fields required by docs are fully represented.
- Keep contract validation fail-closed and bounded.

Post-step acceptance:

```bash
rg -n "PositionRequirementsSchemaLifecycleResult|apply_scope|active_for_new_hires" crates/selene_kernel_contracts/src/ph1position.rs
cargo test -p selene_kernel_contracts ph1position -- --nocapture
```

Step 2 exit criteria:
- Contract compiles and tests pass.
- Lifecycle response semantics are explicit and non-ambiguous.

---

### Step 3: Typed Repo Surface Parity

Patch files (only):
1. `crates/selene_storage/src/repo.rs`

Pre-change gate:

```bash
rg -n "trait Ph1PositionRepo|ph1position_requirements_schema" crates/selene_storage/src/repo.rs
```

Patch intent:
- Add requirements-schema methods to `Ph1PositionRepo`.
- Keep method signatures aligned with kernel contracts and storage implementation.

Post-step acceptance:

```bash
rg -n "trait Ph1PositionRepo|ph1position_requirements_schema_create_draft|ph1position_requirements_schema_update_commit|ph1position_requirements_schema_activate_commit" crates/selene_storage/src/repo.rs
```

Step 3 exit criteria:
- Typed repo boundary fully includes requirements-schema lifecycle.

---

### Step 4: Storage Behavior Fixes (core drift closure)

Patch files (only):
1. `crates/selene_storage/src/ph1f.rs`

Pre-change gate:

```bash
rg -n "_change_reason|_apply_scope|PositionRequirementRuleType::Always \\| PositionRequirementRuleType::Conditional|ph1position_requirements_schema_activate_commit" crates/selene_storage/src/ph1f.rs
```

Patch intent:
- Stop ignoring request fields:
  - persist `change_reason` in schema ledger records.
  - enforce/store `apply_scope` during activation.
- Remove hardcoded activation result behavior:
  - lifecycle result reflects actual scope decision.
- Replace pseudo-conditional logic:
  - evaluate conditional rule deterministically using selector snapshot + predicate reference.
- Align preconditions:
  - enforce documented position lifecycle scope checks for schema update/activate.
- Align idempotency keys with decided contract semantics (consistent across docs + store + migration).

Post-step acceptance:

```bash
rg -n "_change_reason|_apply_scope" crates/selene_storage/src/ph1f.rs
rg -n "change_reason|apply_scope|predicate|selector_snapshot" crates/selene_storage/src/ph1f.rs
cargo test -p selene_storage --test db_wiring_ph1position_tables -- --nocapture
```

Expected:
- No `_change_reason` or `_apply_scope` placeholders remain.
- Schema update/activate logic uses concrete values.

Step 4 exit criteria:
- Storage methods enforce contract semantics deterministically.

---

### Step 5: Runtime Wiring (PH1.POSITION + ONB boundary)

Patch files (only):
1. `crates/selene_os/src/ph1position.rs`
2. `crates/selene_os/src/ph1onb.rs` (only if required for scope handoff fields)

Pre-change gate:

```bash
rg -n "RequirementsSchema|apply_scope|requirements_schema_lifecycle_result|ONB_REQUIREMENT_BACKFILL" crates/selene_os/src/ph1position.rs crates/selene_os/src/ph1onb.rs
```

Patch intent:
- Ensure PH1.POSITION runtime returns schema activation output with explicit rollout scope semantics.
- Keep orchestration boundary clear:
  - PH1.POSITION runtime does not directly run ONB engine logic.
  - Selene OS orchestrates follow-up backfill flow when scope is `CURRENT_AND_NEW`.
- Ensure audit payload contains enough deterministic reasoned evidence.

Post-step acceptance:

```bash
rg -n "apply_scope|CURRENT_AND_NEW|NEW_HIRES_ONLY|requirements_schema_lifecycle_result" crates/selene_os/src/ph1position.rs
cargo test -p selene_os ph1position -- --nocapture
```

Step 5 exit criteria:
- Runtime output and orchestration semantics match docs/contracts.

---

### Step 6: Migration Contract Parity

Patch files (only):
1. `crates/selene_storage/migrations/0014_position_requirements_schema_and_backfill_tables.sql`

Pre-change gate:

```bash
nl -ba crates/selene_storage/migrations/0014_position_requirements_schema_and_backfill_tables.sql | sed -n '1,220p'
```

Patch intent:
- Ensure SQL columns/indexes reflect the final contract decisions:
  - `change_reason` persistence (if selected contract requires).
  - `apply_scope` persistence on activation/backfill linkage (if selected contract requires).
  - idempotency uniqueness shape exactly matches docs + store logic.

Post-step acceptance:

```bash
rg -n "change_reason|apply_scope|idempotency|position_requirements_schema_ledger|onboarding_requirement_backfill_campaigns" crates/selene_storage/migrations/0014_position_requirements_schema_and_backfill_tables.sql
```

Step 6 exit criteria:
- Migration contract text is consistent with storage/runtime/docs.

---

### Step 7: Test Closure (prove AT-05 and AT-06)

Patch files (only):
1. `crates/selene_storage/tests/ph1_position/db_wiring.rs`
2. `crates/selene_os/src/ph1position.rs` (test module)
3. `docs/DB_WIRING/PH1_POSITION.md` (if test names need exact alignment)

Pre-change gate:

```bash
rg -n "at_position_db_05|at_position_db_06|RequirementsSchema" crates/selene_storage/tests/ph1_position/db_wiring.rs crates/selene_os/src/ph1position.rs docs/DB_WIRING/PH1_POSITION.md
```

Patch intent:
- Add explicit tests for:
  - schema activation monotonicity + replay safety (AT-05).
  - ONB reads active schema read-only and cannot mutate schema truth (AT-06).
- Add runtime tests for requirements schema create/update/activate with scope outputs.

Post-step acceptance:

```bash
cargo test -p selene_storage --test db_wiring_ph1position_tables -- --nocapture
cargo test -p selene_os ph1position -- --nocapture
rg -n "at_position_db_05|at_position_db_06" crates/selene_storage/tests/ph1_position/db_wiring.rs docs/DB_WIRING/PH1_POSITION.md
```

Step 7 exit criteria:
- Missing acceptance tests now exist and pass.

---

### Step 8: Final Closure Gates (design + runtime checkpoints)

Patch files (only):
1. `docs/13_PROBLEMS_TO_FIX.md` (update proof/closure status)

Pre-change gate:

```bash
git status --short
git rev-parse HEAD
```

Run final checks:

```bash
scripts/selene_design_readiness_audit.sh
cargo test --workspace
```

Post-step acceptance:
- Audit shows no PH1.POSITION schema-gating drift findings.
- Workspace tests pass.
- Tracker updated with proof commands and closure notes.

## 6) Non-Negotiable Done Criteria

1. No ignored schema lifecycle fields in storage (`change_reason`, `apply_scope`).
2. Conditional required rules are evaluated deterministically.
3. Repo trait, storage, contracts, docs, and migration all describe the same behavior.
4. AT-05 and AT-06 are implemented and passing.
5. ONB requirement checks are schema-driven (no hardcoded requirement aliases for position-specific onboarding).

## 7) Operator Command Set (copy/paste block)

```bash
cd "$(git rev-parse --show-toplevel)"
git rev-parse HEAD
git status --short
rg -n "_change_reason|_apply_scope|PositionRequirementRuleType::Always \\| PositionRequirementRuleType::Conditional" crates/selene_storage/src/ph1f.rs
rg -n "at_position_db_05|at_position_db_06" crates/selene_storage/tests/ph1_position/db_wiring.rs docs/DB_WIRING/PH1_POSITION.md
rg -n "trait Ph1PositionRepo|ph1position_requirements_schema" crates/selene_storage/src/repo.rs
scripts/selene_design_readiness_audit.sh
```

## 8) Change Control Notes

- If a step requires touching unlisted files, pause and amend this packet first.
- If idempotency shape is changed, update docs + migration + store in the same step.
- If rollout semantics are changed, update blueprint, contract, runtime, and tests in lock-step.

## 9) Execution Record

- Step 1: COMPLETED (2026-02-15)
- Step 2: COMPLETED (2026-02-15)
- Step 3: COMPLETED (2026-02-15)
- Step 4: COMPLETED (2026-02-15)
- Step 5: COMPLETED (2026-02-15)
- Step 6: COMPLETED (2026-02-15, N/A no migration delta required)
- Step 7: COMPLETED (2026-02-15)
- Step 8: NOT_STARTED

Step 1 note:
- Locked rollout scope naming for ONB backfill handoff to `CurrentAndNew` in blueprint flow docs so PH1.POSITION activation handoff language matches PH1.ONB kernel contract enum semantics (`BackfillRolloutScope::CurrentAndNew`).

Step 2 note:
- Kernel contract parity gate is already satisfied: `PositionRequirementsSchemaLifecycleResult` returns explicit `apply_scope_result` and validated `backfill_handoff_required` semantics; `ph1position` contract tests passed with no additional code delta required in this step.

Step 3 note:
- Typed repo parity gate is already satisfied: `Ph1PositionRepo` already includes requirements-schema row methods (`create_draft_row`, `update_commit_row`, `activate_commit_row`) and concrete wiring in `repo.rs`; no additional code delta required in this step.

Step 4 note:
- Storage behavior parity gate is already satisfied in `ph1f.rs`: no ignored placeholder fields remain (`_change_reason`/`_apply_scope` absent), conditional required rules are predicate-evaluated, schema update persists `change_reason`, schema activation persists `apply_scope`, and PH1.POSITION DB wiring tests pass.

Step 5 note:
- Runtime wiring parity gate is already satisfied in `ph1position.rs`: requirements-schema activate path returns explicit scope semantics (`NewHiresOnly` vs `CurrentAndNew`) through `requirements_schema_lifecycle_result`, and PH1.POSITION runtime tests pass without adding direct PH1.ONB execution coupling.

Step 6 note:
- Migration contract parity gate is already satisfied in `0014_position_requirements_schema_and_backfill_tables.sql`: SQL already persists/guards `change_reason` and `apply_scope`, and idempotency uniqueness/index shapes match KC.25 + DB wiring + storage behavior; no SQL patch required in this step.

Step 7 note:
- Test closure gate is satisfied with no code delta required: AT-05 (`at_position_db_05_requirements_schema_activation_monotonic`) and AT-06 (`at_position_db_06_onb_read_only_schema_boundary`) already exist in `crates/selene_storage/tests/ph1_position/db_wiring.rs`, runtime requirements-schema lifecycle tests already exist in `crates/selene_os/src/ph1position.rs`, and acceptance test suites pass.

END OF PACKET
