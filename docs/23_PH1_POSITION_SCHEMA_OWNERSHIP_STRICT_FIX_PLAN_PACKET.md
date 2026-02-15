# PH1.POSITION Schema Ownership Strict Fix Plan Packet

Last updated: 2026-02-15
Owner: Selene core design + runtime
Status: STEP7_COMPLETED_PENDING_STEP8

## 1) Purpose

This packet locks the ownership model where:
- `PH1.POSITION` owns requirements-schema truth (versioned, auditable).
- `PH1.ONB` executes pinned requirements schema only (no hardcoded requirement logic).
- `PH1.LINK` provides selector hints only (never schema truth).
- `PH1.CAPREQ` and `PH1.ACCESS` govern approvals for controlled schema changes.

Rules:
- No Simulation -> No Execution.
- Engines never call engines directly; Selene OS orchestrates.
- No silent behavior change; schema/overlay activation must be versioned + audited.

## 2) Scope

1. Position requirements schema ownership is explicit and non-overlapping.
2. ONB one-question execution is schema-driven and replay-stable.
3. Rollout scope (`NewHiresOnly | CurrentAndNew`) handoff stays deterministic.
4. Current staff updates use controlled backfill path (BCAST + REM), not ad-hoc logic.

## 3) Baseline Gate (must run before Step 2)

```bash
cd "$(git rev-parse --show-toplevel)"
git rev-parse HEAD
git status --short

rg -n "PH1.POSITION|requirements schema|pinned schema|CurrentAndNew|NewHiresOnly|backfill|PH1.ONB" \
  docs/05_OS_CONSTITUTION.md \
  docs/06_ENGINE_MAP.md \
  docs/07_ENGINE_REGISTRY.md \
  docs/10_DB_OWNERSHIP_MATRIX.md \
  docs/DB_WIRING/PH1_POSITION.md \
  docs/DB_WIRING/PH1_ONB.md \
  docs/ECM/PH1_POSITION.md \
  docs/ECM/PH1_ONB.md
```

## 4) Patch Order

### Step 1: Docs Lock + Packet Bootstrap

Patch files (only):
1. `docs/02_BUILD_PLAN.md`
2. `docs/23_PH1_POSITION_SCHEMA_OWNERSHIP_STRICT_FIX_PLAN_PACKET.md`

Patch intent:
- Freeze packet 22 completion in build plan.
- Set this packet as canonical next scope.

Post-step acceptance:

```bash
rg -n "a22c5fe|23_PH1_POSITION_SCHEMA_OWNERSHIP_STRICT_FIX_PLAN_PACKET|Next Strict Packet" docs/02_BUILD_PLAN.md
test -f docs/23_PH1_POSITION_SCHEMA_OWNERSHIP_STRICT_FIX_PLAN_PACKET.md
```

---

### Step 2: Cross-Doc Ownership Lock

Patch files (only if drift is found):
1. `docs/05_OS_CONSTITUTION.md`
2. `docs/06_ENGINE_MAP.md`
3. `docs/07_ENGINE_REGISTRY.md`
4. `docs/10_DB_OWNERSHIP_MATRIX.md`
5. `docs/04_KERNEL_CONTRACTS.md` (only if wording parity requires)

Patch intent:
- Lock POSITION owns schema truth, ONB executes only, LINK hints only.

---

### Step 3: DB Wiring + ECM Ownership Lock

Patch files (only if drift is found):
1. `docs/DB_WIRING/PH1_POSITION.md`
2. `docs/DB_WIRING/PH1_ONB.md`
3. `docs/DB_WIRING/PH1_LINK.md`
4. `docs/DB_WIRING/PH1_CAPREQ.md`
5. `docs/DB_WIRING/PH1_BCAST.md`
6. `docs/DB_WIRING/PH1_REM.md`
7. `docs/ECM/PH1_POSITION.md`
8. `docs/ECM/PH1_ONB.md`
9. `docs/ECM/PH1_LINK.md`
10. `docs/ECM/PH1_CAPREQ.md`
11. `docs/ECM/PH1_BCAST.md`
12. `docs/ECM/PH1_REM.md`

Patch intent:
- Remove ownership ambiguity and lock handoff contracts.

---

### Step 4: Blueprint + Simulation Lock

Patch files (only if drift is found):
1. `docs/BLUEPRINTS/POSITION_MANAGE.md`
2. `docs/BLUEPRINTS/ONB_INVITED.md`
3. `docs/BLUEPRINTS/ONB_SCHEMA_MANAGE.md`
4. `docs/BLUEPRINTS/ONB_REQUIREMENT_BACKFILL.md`
5. `docs/08_SIMULATION_CATALOG.md`
6. `docs/09_BLUEPRINT_REGISTRY.md` (only if required)

Patch intent:
- Ensure simulations/steps match ownership and rollout rules.

---

### Step 5: Kernel + Runtime Parity

Patch files (only if drift is found):
1. `crates/selene_kernel_contracts/src/ph1position.rs`
2. `crates/selene_kernel_contracts/src/ph1onb.rs`
3. `crates/selene_os/src/ph1position.rs`
4. `crates/selene_os/src/ph1onb.rs`
5. `crates/selene_os/src/simulation_executor.rs`

Patch intent:
- Enforce schema-driven required fields and fail-closed gate behavior.

---

### Step 6: Storage + Migration Parity

Patch files (only if drift is found):
1. `crates/selene_storage/src/repo.rs`
2. `crates/selene_storage/src/ph1f.rs`
3. `crates/selene_storage/migrations/*` (only when SQL parity requires)

Patch intent:
- Ensure persistence constraints match deterministic rollout and replay semantics.

---

### Step 7: Test Closure

Patch files (only if drift is found):
1. `crates/selene_storage/tests/ph1_position/db_wiring.rs`
2. `crates/selene_storage/tests/ph1_onb/db_wiring.rs`
3. `crates/selene_os/src/ph1position.rs` (test module)
4. `crates/selene_os/src/ph1onb.rs` (test module)

Required coverage:
- Position schema version activation monotonic + deterministic.
- `NewHiresOnly` does not launch backfill.
- `CurrentAndNew` launches backfill handoff deterministically.
- ONB fail-closed when schema/gates are missing.

---

### Step 8: Final Proof + Freeze Checkpoint

No patching in this step.

Run:

```bash
scripts/selene_design_readiness_audit.sh
cargo test --workspace
git status --short
git rev-parse HEAD
git log -1 --oneline
```

Checkpoint expectations:
- Ownership model is coherent across docs/contracts/runtime/storage/tests.
- Workspace tests pass.
- Packet checkpoint is clean and commit-pinned.

## 5) Execution Record (fill during work)

- Step 1: COMPLETED (2026-02-15)
- Step 2: COMPLETED (2026-02-15)
- Step 3: COMPLETED (2026-02-15)
- Step 4: COMPLETED (2026-02-15)
- Step 5: COMPLETED (2026-02-15)
- Step 6: COMPLETED (2026-02-15)
- Step 7: COMPLETED (2026-02-15)
- Step 8: PENDING

Step 1 note:
- Updated `docs/02_BUILD_PLAN.md` checkpoint/pointer to close packet 22 and set this packet as next canonical scope.
- Created this packet as the strict execution plan for POSITION schema ownership lock.

Step 2 note:
- Cross-doc ownership wording lock completed across constitution/map/registry/ownership/contracts:
  - reinforced schema-defined requirement prompting (no hardcoded ONB requirement branches),
  - tightened PH1.LINK wording to selector-hint capture only (no schema ownership),
  - aligned KC.25 rollout scope tokens to `NewHiresOnly | CurrentAndNew` and added LINK non-ownership hard-rule line.

Step 3 note:
- DB_WIRING + ECM ownership lock completed with minimal boundary clarifications:
  - PH1.LINK DB/ECM now explicitly states selector-hint/prefill capture only and no schema definition ownership,
  - PH1.ONB DB/ECM now explicitly states required-question prompting is schema-derived from pinned field specs/gates only.
- Other Step-3 scope files were reviewed and required no additional wording changes.

Step 4 note:
- Blueprint + simulation lock completed for ownership wording and strict simulation token hygiene:
  - reinforced schema-ownership boundaries in `POSITION_MANAGE`, `ONB_SCHEMA_MANAGE`, and `ONB_REQUIREMENT_BACKFILL`,
  - normalized `ONB_INVITED` simulation requirement lines to plain simulation IDs and moved conditional semantics into explicit note text.
- Simulation catalog and blueprint registry were reviewed; no additional lock edits required in this step.

Step 5 note:
- Kernel/runtime parity review completed for Step-5 scope files:
  - `crates/selene_kernel_contracts/src/ph1position.rs`
  - `crates/selene_kernel_contracts/src/ph1onb.rs`
  - `crates/selene_os/src/ph1position.rs`
  - `crates/selene_os/src/ph1onb.rs`
  - `crates/selene_os/src/simulation_executor.rs`
- No additional code delta was required; current runtime already enforces schema-gated/fail-closed behavior through contracts + storage/runtime checks.
- Targeted proof tests executed:
  - `cargo test -p selene_os onb_fail_closed_when_required_verification_gates_missing`
  - `cargo test -p selene_os ph1position_requirements_schema_create_update_activate_scope_outputs`

Step 6 note:
- Storage + migration parity review completed for Step-6 scope files:
  - `crates/selene_storage/src/repo.rs`
  - `crates/selene_storage/src/ph1f.rs`
  - `crates/selene_storage/migrations/0014_position_requirements_schema_and_backfill_tables.sql`
- No additional storage/migration delta was required.
- Verified deterministic scope constraints remain fail-closed and aligned:
  - `apply_scope` locked to `NewHiresOnly | CurrentAndNew` for `ACTIVATE_COMMIT` rows.
  - ONB backfill rollout remains `CurrentAndNew`-only in kernel validation, storage validation, and SQL `CHECK`.
  - ONB verification actions remain schema-gated via pinned required verification gates.

Step 7 note:
- Test-closure review completed for all Step-7 scope files:
  - `crates/selene_storage/tests/ph1_position/db_wiring.rs`
  - `crates/selene_storage/tests/ph1_onb/db_wiring.rs`
  - `crates/selene_os/src/ph1position.rs` (test module)
  - `crates/selene_os/src/ph1onb.rs` (test module)
- No additional test-file patch was required; required coverage was already present and passing.
- Proof tests executed:
  - `cargo test -p selene_storage at_position_db_05_requirements_schema_activation_monotonic`
  - `cargo test -p selene_storage at_onb_db_08_backfill_new_hires_only_is_refused_no_campaign_started`
  - `cargo test -p selene_storage at_onb_db_09_backfill_current_and_new_creates_campaign_with_deterministic_snapshot`
  - `cargo test -p selene_storage at_onb_db_10_backfill_notify_loop_and_complete_are_idempotent`
  - `cargo test -p selene_storage at_onb_db_11_backfill_fail_closed_on_tenant_scope_and_missing_target`
  - `cargo test -p selene_os ph1position_requirements_schema_create_update_activate_scope_outputs`
  - `cargo test -p selene_os onb_backfill_start_refuses_new_hires_only_scope`
  - `cargo test -p selene_os onb_backfill_current_and_new_notify_loop_then_complete`
  - `cargo test -p selene_os onb_fail_closed_when_required_verification_gates_missing`

## 6) Done Criteria

This packet is done only when all are true:
1. `PH1.POSITION` is explicit schema owner everywhere.
2. `PH1.ONB` is explicit schema executor everywhere.
3. Rollout scope + backfill behavior is deterministic and fail-closed.
4. Final readiness audit + workspace tests pass at a pinned clean checkpoint.
