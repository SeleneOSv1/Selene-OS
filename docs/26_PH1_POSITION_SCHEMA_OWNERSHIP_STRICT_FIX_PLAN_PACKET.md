# PH1.POSITION Schema Ownership Strict Fix Plan Packet (v2)

Last updated: 2026-02-15
Owner: Selene core design + runtime
Status: STEP8_COMPLETED_FROZEN

## 1) Purpose

This packet re-locks PH1.POSITION schema ownership after PH1.ONB schema-driven closure.

Ownership laws to preserve:
- `PH1.POSITION` is the only writer/owner for position requirements schema truth.
- `PH1.ONB` executes pinned schema requirements only (executor-only; no schema writes).
- `PH1.LINK` carries selector/prefill hints only (never schema truth).
- `PH1.CAPREQ` + `PH1.ACCESS` govern controlled schema-change permissions/approvals.

System laws:
- No Simulation -> No Execution.
- Engines never call engines directly; Selene OS orchestrates.
- No silent behavior changes; schema lifecycle transitions are versioned + audited.

## 2) Scope

1. Re-validate POSITION ownership boundaries across canonical docs and runtime/storage surfaces.
2. Ensure ONB session pinning and gate execution do not drift into schema ownership.
3. Ensure LINK selector-hint handoff remains read-only and deterministic.
4. Ensure backfill/rollout rules stay under controlled deterministic flows.

## 3) Baseline Gate (must run before Step 2)

```bash
cd "$(git rev-parse --show-toplevel)"
git rev-parse HEAD
git status --short

rg -n "PH1.POSITION|requirements schema|schema truth|pinned schema|executor-only|CurrentAndNew|NewHiresOnly|backfill|selector hints|PH1.ONB|PH1.LINK|PH1.CAPREQ" \
  docs/04_KERNEL_CONTRACTS.md \
  docs/05_OS_CONSTITUTION.md \
  docs/06_ENGINE_MAP.md \
  docs/07_ENGINE_REGISTRY.md \
  docs/10_DB_OWNERSHIP_MATRIX.md \
  docs/DB_WIRING/PH1_POSITION.md \
  docs/DB_WIRING/PH1_ONB.md \
  docs/DB_WIRING/PH1_LINK.md \
  docs/DB_WIRING/PH1_CAPREQ.md \
  docs/ECM/PH1_POSITION.md \
  docs/ECM/PH1_ONB.md \
  docs/ECM/PH1_LINK.md \
  docs/ECM/PH1_CAPREQ.md \
  docs/BLUEPRINTS/POSITION_MANAGE.md \
  docs/BLUEPRINTS/ONB_INVITED.md \
  docs/BLUEPRINTS/ONB_REQUIREMENT_BACKFILL.md \
  crates/selene_kernel_contracts/src/ph1position.rs \
  crates/selene_kernel_contracts/src/ph1onb.rs \
  crates/selene_os/src/ph1position.rs \
  crates/selene_os/src/ph1onb.rs \
  crates/selene_storage/src/ph1f.rs
```

## 4) Patch Order

### Step 1: Docs Lock + Packet Bootstrap

Patch files (only):
1. `docs/02_BUILD_PLAN.md`
2. `docs/26_PH1_POSITION_SCHEMA_OWNERSHIP_STRICT_FIX_PLAN_PACKET.md`

Patch intent:
- Freeze packet 25 closure in build plan.
- Set packet 26 as canonical next strict scope.

Post-step acceptance:

```bash
rg -n "a2f7aa8|26_PH1_POSITION_SCHEMA_OWNERSHIP_STRICT_FIX_PLAN_PACKET|Next Strict Packet" docs/02_BUILD_PLAN.md
test -f docs/26_PH1_POSITION_SCHEMA_OWNERSHIP_STRICT_FIX_PLAN_PACKET.md
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
- Keep POSITION as schema truth owner and ONB as executor-only across canonical docs.

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
- Remove ownership ambiguity and lock deterministic handoff contracts.

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
- Keep simulation and blueprint flow consistent with POSITION ownership boundaries.

---

### Step 5: Kernel + Runtime Parity

Patch files (only if drift is found):
1. `crates/selene_kernel_contracts/src/ph1position.rs`
2. `crates/selene_kernel_contracts/src/ph1onb.rs`
3. `crates/selene_os/src/ph1position.rs`
4. `crates/selene_os/src/ph1onb.rs`
5. `crates/selene_os/src/simulation_executor.rs`

Patch intent:
- Ensure runtime contract flow enforces schema ownership boundaries and fail-closed behavior.

---

### Step 6: Storage + Migration Parity

Patch files (only if drift is found):
1. `crates/selene_storage/src/repo.rs`
2. `crates/selene_storage/src/ph1f.rs`
3. `crates/selene_storage/migrations/*` (only when SQL parity requires)

Patch intent:
- Keep persistence and idempotency semantics deterministic across position schema lifecycle flows.

---

### Step 7: Test Closure

Patch files (only if drift is found):
1. `crates/selene_storage/tests/ph1_position/db_wiring.rs`
2. `crates/selene_storage/tests/ph1_onb/db_wiring.rs`
3. `crates/selene_storage/tests/ph1_link/db_wiring.rs` (only if handoff coverage drift found)
4. `crates/selene_os/src/ph1position.rs` (test module)
5. `crates/selene_os/src/ph1onb.rs` (test module)

Required coverage:
- Position schema activation/versioning is monotonic and deterministic.
- `NewHiresOnly` does not start backfill campaign.
- `CurrentAndNew` starts controlled backfill deterministically.
- ONB fails closed when required schema/gates are absent.
- LINK handoff remains selector-hints-only and deterministic.

---

### Step 8: Final Proof + Freeze Checkpoint

No patching in this step.

Run:

```bash
scripts/selene_design_readiness_audit.sh
cargo test -p selene_storage --test db_wiring_ph1position_tables -- --nocapture
cargo test -p selene_storage --test db_wiring_ph1onb_tables -- --nocapture
cargo test -p selene_os ph1position -- --nocapture
cargo test -p selene_os ph1onb -- --nocapture
cargo test --workspace
git status --short
git rev-parse HEAD
git log -1 --oneline
```

Checkpoint expectations:
- POSITION schema ownership remains coherent across docs/contracts/runtime/storage/tests.
- Audit and tests pass at a clean pinned checkpoint.

## 5) Execution Record (fill during work)

- Step 1: COMPLETED (2026-02-15)
- Step 2: COMPLETED (2026-02-15)
- Step 3: COMPLETED (2026-02-15)
- Step 4: COMPLETED (2026-02-15)
- Step 5: COMPLETED (2026-02-15)
- Step 6: COMPLETED (2026-02-15)
- Step 7: COMPLETED (2026-02-15)
- Step 8: COMPLETED (2026-02-15)

Step 1 note:
- Updated `docs/02_BUILD_PLAN.md` to freeze packet 25 closure checkpoint (`a2f7aa8`) and set this packet as canonical next strict scope.
- Created this packet as the strict execution plan for PH1.POSITION schema ownership closure refresh.

Step 2 note:
- Cross-doc ownership lock review completed for:
  - `docs/05_OS_CONSTITUTION.md`
  - `docs/06_ENGINE_MAP.md`
  - `docs/07_ENGINE_REGISTRY.md`
  - `docs/10_DB_OWNERSHIP_MATRIX.md`
  - `docs/04_KERNEL_CONTRACTS.md`
- No drift patch was required in Step 2.
- Verified alignment:
  - `PH1.POSITION` remains schema truth owner for requirements schema lifecycle writes.
  - `PH1.ONB` remains executor-only (pinned schema execution; no schema mutation).
  - `PH1.LINK` remains selector-hints-only (no schema ownership).
  - governed change paths remain gated through access/approval flows (`PH1.ACCESS` / `PH1.CAPREQ` where applicable).

Step 3 note:
- DB wiring + ECM ownership lock review completed for:
  - `docs/DB_WIRING/PH1_POSITION.md`
  - `docs/DB_WIRING/PH1_ONB.md`
  - `docs/DB_WIRING/PH1_LINK.md`
  - `docs/DB_WIRING/PH1_CAPREQ.md`
  - `docs/DB_WIRING/PH1_BCAST.md`
  - `docs/DB_WIRING/PH1_REM.md`
  - `docs/ECM/PH1_POSITION.md`
  - `docs/ECM/PH1_ONB.md`
  - `docs/ECM/PH1_LINK.md`
  - `docs/ECM/PH1_CAPREQ.md`
  - `docs/ECM/PH1_BCAST.md`
  - `docs/ECM/PH1_REM.md`
- No drift patch was required in Step 3.
- Verified lock points:
  - POSITION DB/ECM surfaces explicitly retain requirements-schema ownership.
  - ONB DB/ECM surfaces explicitly retain executor-only, pinned-schema behavior.
  - LINK DB/ECM surfaces explicitly retain selector-hint-only handoff and no schema mutation.
  - BCAST/REM DB/ECM surfaces preserve deterministic ONB backfill handoff boundaries (BCAST lifecycle + REM timing only).

Step 4 note:
- Blueprint + simulation lock review completed for:
  - `docs/BLUEPRINTS/POSITION_MANAGE.md`
  - `docs/BLUEPRINTS/ONB_INVITED.md`
  - `docs/BLUEPRINTS/ONB_SCHEMA_MANAGE.md`
  - `docs/BLUEPRINTS/ONB_REQUIREMENT_BACKFILL.md`
  - `docs/08_SIMULATION_CATALOG.md`
  - `docs/09_BLUEPRINT_REGISTRY.md`
- No drift patch was required in Step 4.
- Verified lock points:
  - Blueprint step flows preserve schema ownership boundaries (`PH1.POSITION` schema lifecycle, `PH1.ONB` executor/backfill progress only, `PH1.LINK` hints handoff only).
  - `ONB_SCHEMA_MANAGE` and `ONB_REQUIREMENT_BACKFILL` remain deterministic around `NewHiresOnly` vs `CurrentAndNew`.
  - Required simulation ids for position schema lifecycle and ONB backfill are present and ACTIVE in simulation catalog.
  - Blueprint registry entries for `POSITION_MANAGE`, `ONB_INVITED`, `ONB_SCHEMA_MANAGE`, and `ONB_REQUIREMENT_BACKFILL` are ACTIVE and aligned.

Step 5 note:
- Kernel + runtime parity review completed for:
  - `crates/selene_kernel_contracts/src/ph1position.rs`
  - `crates/selene_kernel_contracts/src/ph1onb.rs`
  - `crates/selene_os/src/ph1position.rs`
  - `crates/selene_os/src/ph1onb.rs`
  - `crates/selene_os/src/simulation_executor.rs`
- No runtime/contract drift patch was required in Step 5.
- Verified lock points:
  - Position requirements schema lifecycle simulations remain mapped to PH1.POSITION contract/runtime surfaces.
  - ONB backfill and verification-gate flows remain mapped to PH1.ONB contract/runtime surfaces with fail-closed rollout guard semantics (`CurrentAndNew` enforced where required).
  - In `ph1onb.rs`, position requirements schema create/activate calls are test-only (`#[cfg(test)]`) and are not present in production runtime flow.
  - Simulation executor continues to route through explicit runtime entrypoints (`execute_position`, `execute_onb`) without ownership-role remapping.
- Proof runs:
  - `cargo test -p selene_os ph1position -- --nocapture` -> pass (`3 passed; 0 failed`)
  - `cargo test -p selene_os ph1onb -- --nocapture` -> pass (`10 passed; 0 failed`)

Step 6 note:
- Storage + migration parity review completed for:
  - `crates/selene_storage/src/repo.rs`
  - `crates/selene_storage/src/ph1f.rs`
  - `crates/selene_storage/migrations/0014_position_requirements_schema_and_backfill_tables.sql`
- No storage/migration patch was required in Step 6.
- Verified lock points:
  - Repo trait method signatures and `Ph1fStore` row adapters remain pass-through and preserve deterministic idempotency boundaries for position schema lifecycle and ONB backfill flows.
  - Storage enforcement keeps rollout guard fail-closed (`ONB_REQUIREMENT_BACKFILL` requires `CurrentAndNew`) and preserves position schema lifecycle ownership under PH1.POSITION writes.
  - SQL migration constraints/indexes remain aligned with current runtime invariants (`apply_scope` bounds, backfill campaign `rollout_scope='CurrentAndNew'`, and idempotency uniqueness indexes).
- Proof runs:
  - `cargo test -p selene_storage --test db_wiring_ph1position_tables -- --nocapture` -> pass (`6 passed; 0 failed`)
  - `cargo test -p selene_storage --test db_wiring_ph1onb_tables -- --nocapture` -> pass (`12 passed; 0 failed`)

Step 7 note:
- Test-closure verification completed for required coverage surfaces:
  - `crates/selene_storage/tests/ph1_position/db_wiring.rs`
  - `crates/selene_storage/tests/ph1_onb/db_wiring.rs`
  - `crates/selene_storage/tests/ph1_link/db_wiring.rs`
  - `crates/selene_os/src/ph1position.rs` (test module)
  - `crates/selene_os/src/ph1onb.rs` (test module)
- No test patch was required in Step 7.
- Required coverage checks were confirmed:
  - Position schema activation/versioning monotonic and deterministic.
  - `NewHiresOnly` does not start backfill campaign.
  - `CurrentAndNew` starts controlled deterministic backfill campaign.
  - ONB fails closed when required verification gates are absent.
  - LINK handoff remains selector-hints-only and deterministic (schema-driven recompute path covered).
- Proof runs:
  - `cargo test -p selene_storage --test db_wiring_ph1position_tables -- --nocapture` -> pass (`6 passed; 0 failed`)
  - `cargo test -p selene_storage --test db_wiring_ph1onb_tables -- --nocapture` -> pass (`12 passed; 0 failed`)
  - `cargo test -p selene_storage --test db_wiring_ph1link_tables -- --nocapture` -> pass (`13 passed; 0 failed`)
  - `cargo test -p selene_os ph1position -- --nocapture` -> pass (`3 passed; 0 failed`)
  - `cargo test -p selene_os ph1onb -- --nocapture` -> pass (`10 passed; 0 failed`)

Step 8 note:
- Final proof command set executed successfully:
  - `scripts/selene_design_readiness_audit.sh` -> pass (audit exit 0)
  - `cargo test -p selene_storage --test db_wiring_ph1position_tables -- --nocapture` -> pass (`6 passed; 0 failed`)
  - `cargo test -p selene_storage --test db_wiring_ph1onb_tables -- --nocapture` -> pass (`12 passed; 0 failed`)
  - `cargo test -p selene_os ph1position -- --nocapture` -> pass (`3 passed; 0 failed`)
  - `cargo test -p selene_os ph1onb -- --nocapture` -> pass (`10 passed; 0 failed`)
  - `cargo test --workspace` -> pass (all workspace tests green)
- Freeze target for this packet is now ready for clean-tree commit and pinned-hash closure proof.

## 6) Done Criteria

This packet is done only when all are true:
1. POSITION remains the sole requirements-schema truth owner.
2. ONB remains executor-only and pinned-schema driven.
3. LINK remains selector-hints-only for onboarding handoff.
4. Rollout/backfill behavior remains deterministic, simulation-gated, and fail-closed.
5. Final readiness audit + tests pass from a clean pinned checkpoint.
