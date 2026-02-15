# PH1.ACCESS + PH1.CAPREQ Governance Strict Fix Plan Packet (v1)

Last updated: 2026-02-15
Owner: Selene core design + runtime
Status: STEP8_COMPLETED_READY_FOR_FREEZE_CHECKPOINT

## 1) Purpose

This packet hardens governance paths where schema-changing flows require explicit Access decisions and Capability Request lifecycle control.

Governance laws to preserve:
- `PH1.ACCESS.001_PH2.ACCESS.002` is the authority gate truth (`ALLOW | DENY | ESCALATE`).
- `PH1.CAPREQ` records capability request lifecycle truth only (`Draft -> PendingApproval -> Approved/Rejected -> Fulfilled/Canceled`).
- `PH1.CAPREQ` does not grant authority by itself.
- Selene OS must orchestrate Access + CAPREQ sequencing; engines never call engines directly.

System laws:
- No Simulation -> No Execution.
- No silent authority changes.
- Deterministic idempotent transitions only.

## 2) Scope

1. Lock Access/CAPREQ governance wording across canonical docs.
2. Lock DB wiring + ECM boundaries for deny/escalate/approve lifecycle paths.
3. Lock blueprint + simulation sequencing for governed schema-change flows.
4. Verify runtime/storage/tests enforce fail-closed behavior when gates are missing.

## 3) Baseline Gate (must run before Step 2)

```bash
cd "$(git rev-parse --show-toplevel)"
git rev-parse HEAD
git status --short

rg -n "PH1.ACCESS|PH2.ACCESS|PH1.CAPREQ|ACCESS_GATE_DECIDE_ROW|ALLOW|DENY|ESCALATE|CAPREQ_SUBMIT_FOR_APPROVAL_COMMIT|CAPREQ_APPROVE_COMMIT|CAPREQ_REJECT_COMMIT|CAPREQ_FULFILL_COMMIT|CAPREQ_CANCEL_REVOKE|governed|schema" \
  docs/04_KERNEL_CONTRACTS.md \
  docs/05_OS_CONSTITUTION.md \
  docs/06_ENGINE_MAP.md \
  docs/07_ENGINE_REGISTRY.md \
  docs/08_SIMULATION_CATALOG.md \
  docs/09_BLUEPRINT_REGISTRY.md \
  docs/10_DB_OWNERSHIP_MATRIX.md \
  docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md \
  docs/DB_WIRING/PH1_CAPREQ.md \
  docs/DB_WIRING/PH1_ONB.md \
  docs/DB_WIRING/PH1_POSITION.md \
  docs/DB_WIRING/PH1_LINK.md \
  docs/ECM/PH1_ACCESS_001_PH2_ACCESS_002.md \
  docs/ECM/PH1_CAPREQ.md \
  docs/BLUEPRINTS/CAPREQ_MANAGE.md \
  docs/BLUEPRINTS/ONB_SCHEMA_MANAGE.md \
  docs/BLUEPRINTS/ONB_REQUIREMENT_BACKFILL.md \
  docs/BLUEPRINTS/POSITION_MANAGE.md \
  docs/BLUEPRINTS/LINK_INVITE.md \
  crates/selene_kernel_contracts/src/ph1capreq.rs \
  crates/selene_os/src/ph1capreq.rs \
  crates/selene_os/src/ph1onb.rs \
  crates/selene_os/src/ph1position.rs \
  crates/selene_os/src/ph1link.rs \
  crates/selene_os/src/simulation_executor.rs \
  crates/selene_storage/src/repo.rs \
  crates/selene_storage/src/ph1f.rs
```

## 4) Patch Order

### Step 1: Docs Lock + Packet Bootstrap

Patch files (only):
1. `docs/02_BUILD_PLAN.md`
2. `docs/27_PH1_ACCESS_CAPREQ_GOVERNANCE_STRICT_FIX_PLAN_PACKET.md`

Patch intent:
- Freeze packet 26 closure in build plan.
- Set packet 27 as canonical next strict scope.

Post-step acceptance:

```bash
rg -n "a7acbff|27_PH1_ACCESS_CAPREQ_GOVERNANCE_STRICT_FIX_PLAN_PACKET|Next Strict Packet" docs/02_BUILD_PLAN.md
test -f docs/27_PH1_ACCESS_CAPREQ_GOVERNANCE_STRICT_FIX_PLAN_PACKET.md
```

---

### Step 2: Cross-Doc Governance Lock

Patch files (only if drift is found):
1. `docs/04_KERNEL_CONTRACTS.md`
2. `docs/05_OS_CONSTITUTION.md`
3. `docs/06_ENGINE_MAP.md`
4. `docs/07_ENGINE_REGISTRY.md`
5. `docs/10_DB_OWNERSHIP_MATRIX.md`

Patch intent:
- Ensure Access is authority gate and CAPREQ is lifecycle-truth only across canonical docs.

---

### Step 3: DB Wiring + ECM Governance Lock

Patch files (only if drift is found):
1. `docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md`
2. `docs/DB_WIRING/PH1_CAPREQ.md`
3. `docs/DB_WIRING/PH1_ONB.md`
4. `docs/DB_WIRING/PH1_POSITION.md`
5. `docs/DB_WIRING/PH1_LINK.md`
6. `docs/ECM/PH1_ACCESS_001_PH2_ACCESS_002.md`
7. `docs/ECM/PH1_CAPREQ.md`

Patch intent:
- Remove governance ambiguity and lock deny/escalate/approve boundaries.

---

### Step 4: Blueprint + Simulation Governance Lock

Patch files (only if drift is found):
1. `docs/BLUEPRINTS/CAPREQ_MANAGE.md`
2. `docs/BLUEPRINTS/ONB_SCHEMA_MANAGE.md`
3. `docs/BLUEPRINTS/ONB_REQUIREMENT_BACKFILL.md`
4. `docs/BLUEPRINTS/POSITION_MANAGE.md`
5. `docs/BLUEPRINTS/LINK_INVITE.md`
6. `docs/08_SIMULATION_CATALOG.md`
7. `docs/09_BLUEPRINT_REGISTRY.md` (only if required)

Patch intent:
- Keep blueprint step order and simulation IDs coherent for governed side effects.

---

### Step 5: Kernel + Runtime Parity

Patch files (only if drift is found):
1. `crates/selene_kernel_contracts/src/ph1capreq.rs`
2. `crates/selene_os/src/ph1capreq.rs`
3. `crates/selene_os/src/ph1onb.rs`
4. `crates/selene_os/src/ph1position.rs`
5. `crates/selene_os/src/ph1link.rs`
6. `crates/selene_os/src/simulation_executor.rs`

Patch intent:
- Ensure runtime enforces Access decisions before governed CAPREQ transitions.

---

### Step 6: Storage + Migration Parity

Patch files (only if drift is found):
1. `crates/selene_storage/src/repo.rs`
2. `crates/selene_storage/src/ph1f.rs`
3. `crates/selene_storage/migrations/0009_access_instance_tables.sql`
4. `crates/selene_storage/migrations/0013_capreq_tables.sql`
5. `crates/selene_storage/migrations/*` (only when parity requires)

Patch intent:
- Keep Access/CAPREQ persistence deterministic, idempotent, and tenant-scoped.

---

### Step 7: Test Closure

Patch files (only if drift is found):
1. `crates/selene_storage/tests/ph1_access_ph2_access/db_wiring.rs`
2. `crates/selene_storage/tests/ph1_capreq/db_wiring.rs`
3. `crates/selene_os/src/ph1capreq.rs` (test module)
4. `crates/selene_os/src/ph1onb.rs` (test module)
5. `crates/selene_os/src/ph1position.rs` (test module)

Required coverage:
- Access deny path blocks governed commit.
- Access escalate path requires approval before governed commit.
- CAPREQ transition invalid paths fail closed with deterministic reason codes.
- Approved path executes commit exactly once under idempotent replay.
- Tenant-scope mismatch fails closed.

---

### Step 8: Final Proof + Freeze Checkpoint

No patching in this step.

Run:

```bash
scripts/selene_design_readiness_audit.sh
cargo test -p selene_storage --test db_wiring_access_tables -- --nocapture
cargo test -p selene_storage --test db_wiring_ph1capreq_tables -- --nocapture
cargo test -p selene_os capreq -- --nocapture
cargo test -p selene_os ph1onb -- --nocapture
cargo test -p selene_os ph1position -- --nocapture
cargo test --workspace
git status --short
git rev-parse HEAD
git log -1 --oneline
```

Checkpoint expectations:
- Access/CAPREQ governance boundaries are coherent across docs/contracts/runtime/storage/tests.
- Audit + tests pass at a clean pinned checkpoint.

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
- Updated `docs/02_BUILD_PLAN.md` to freeze packet 26 closure checkpoint (`a7acbff`) and set this packet as canonical next strict scope.
- Created this packet as the strict execution plan for PH1.ACCESS + PH1.CAPREQ governance hardening.

Step 2 note:
- Cross-doc governance lock review completed for:
  - `docs/04_KERNEL_CONTRACTS.md`
  - `docs/05_OS_CONSTITUTION.md`
  - `docs/06_ENGINE_MAP.md`
  - `docs/07_ENGINE_REGISTRY.md`
  - `docs/10_DB_OWNERSHIP_MATRIX.md`
- Drift patch applied:
  - Normalized Access decision token wording in `docs/04_KERNEL_CONTRACTS.md` from legacy `REQUIRE_APPROVAL` to canonical `ESCALATE` in KC.12 runtime decision and acceptance-test wording.
- Verified lock points:
  - Access remains explicit authority gate (`ALLOW | DENY | ESCALATE`) before governed commit side effects.
  - CAPREQ remains lifecycle-truth only; authority gating remains Access/AP-driven.
  - No cross-doc ownership inversion found for ONB/POSITION/LINK touched governance statements.

Step 3 note:
- DB wiring + ECM governance lock review completed for:
  - `docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md`
  - `docs/DB_WIRING/PH1_CAPREQ.md`
  - `docs/DB_WIRING/PH1_ONB.md`
  - `docs/DB_WIRING/PH1_POSITION.md`
  - `docs/DB_WIRING/PH1_LINK.md`
  - `docs/ECM/PH1_ACCESS_001_PH2_ACCESS_002.md`
  - `docs/ECM/PH1_CAPREQ.md`
- No drift patch was required in Step 3.
- Verified lock points:
  - Access DB/ECM surfaces consistently define `access_decision` as `ALLOW | DENY | ESCALATE`.
  - Access escalation rules remain fail-closed (`ESCALATE` when approval path exists; `DENY` only when no approval path exists).
  - CAPREQ DB/ECM surfaces consistently keep lifecycle-truth-only semantics and explicitly state CAPREQ does not grant authority by itself.
  - ONB/POSITION/LINK DB wiring surfaces remain ownership-safe relative to governance boundaries (no schema/authority role inversion).

Step 4 note:
- Blueprint + simulation governance lock review completed for:
  - `docs/BLUEPRINTS/CAPREQ_MANAGE.md`
  - `docs/BLUEPRINTS/ONB_SCHEMA_MANAGE.md`
  - `docs/BLUEPRINTS/ONB_REQUIREMENT_BACKFILL.md`
  - `docs/BLUEPRINTS/POSITION_MANAGE.md`
  - `docs/BLUEPRINTS/LINK_INVITE.md`
  - `docs/08_SIMULATION_CATALOG.md`
  - `docs/09_BLUEPRINT_REGISTRY.md`
- Drift patch applied:
  - Added explicit `ESCALATE` fail-closed refusal clauses (`ACCESS_AP_REQUIRED`) in blueprint refusal sections so governed side effects are blocked until approval/override is resolved.
  - Patched files:
    - `docs/BLUEPRINTS/CAPREQ_MANAGE.md`
    - `docs/BLUEPRINTS/ONB_SCHEMA_MANAGE.md`
    - `docs/BLUEPRINTS/ONB_REQUIREMENT_BACKFILL.md`
    - `docs/BLUEPRINTS/POSITION_MANAGE.md`
    - `docs/BLUEPRINTS/LINK_INVITE.md`
- Verified lock points:
  - Target blueprint registry entries remain ACTIVE and unchanged.
  - Referenced governance simulation IDs remain present in simulation catalog.
  - No simulation catalog or blueprint registry patch was required for Step 4.

Step 5 note:
- Kernel + runtime parity review completed for:
  - `crates/selene_kernel_contracts/src/ph1capreq.rs`
  - `crates/selene_os/src/ph1capreq.rs`
  - `crates/selene_os/src/ph1onb.rs`
  - `crates/selene_os/src/ph1position.rs`
  - `crates/selene_os/src/ph1link.rs`
  - `crates/selene_os/src/simulation_executor.rs`
- Drift patch applied:
  - Added explicit Access-gate enforcement to CAPREQ simulation-candidate dispatch path in `simulation_executor`:
    - resolve actor tenant access instance
    - execute gate decision for `CAPREQ_MANAGE`
    - fail closed on `DENY` (`ACCESS_SCOPE_VIOLATION`) and `ESCALATE` (`ACCESS_AP_REQUIRED`)
    - allow CAPREQ lifecycle execution only on `ALLOW`
  - Added deterministic CAPREQ test helper access-instance seeding in `simulation_executor` tests to preserve governed path setup.
- Patched runtime file:
  - `crates/selene_os/src/simulation_executor.rs`
- No contract patch was required in `ph1capreq.rs` for Step 5.
- Proof runs:
  - `cargo test -p selene_os capreq -- --nocapture` -> pass (`9 passed; 0 failed`)
  - `cargo test -p selene_os simulation_executor::tests::at_sim_exec_0 -- --nocapture` -> pass (`9 passed; 0 failed`)

Step 6 note:
- Storage + migration parity review completed for:
  - `crates/selene_storage/src/repo.rs`
  - `crates/selene_storage/src/ph1f.rs`
  - `crates/selene_storage/migrations/0009_access_instance_tables.sql`
  - `crates/selene_storage/migrations/0013_capreq_tables.sql`
- No storage/repository/migration patch was required in Step 6.
- Verified lock points:
  - Access storage surfaces remain tenant-scoped and idempotent (`access_instances`, `access_overrides` unique/idempotency indices).
  - CAPREQ storage surfaces remain append-only ledger + rebuildable current projection with tenant-scoped idempotency.
  - Access gate decision path in storage remains fail-closed (`DENY` for missing/scope mismatch, `ESCALATE` for restricted/step-up requirements).
- Proof runs:
  - `cargo test -p selene_storage --test db_wiring_access_tables -- --nocapture` -> pass (`4 passed; 0 failed`)
  - `cargo test -p selene_storage --test db_wiring_ph1capreq_tables -- --nocapture` -> pass (`4 passed; 0 failed`)

Step 7 note:
- Test-closure review completed against required coverage:
  - Access deny path blocks governed commit.
  - Access escalate path requires approval before governed commit.
  - CAPREQ invalid transition fails closed with deterministic failure reason.
  - Approved path executes once under idempotent replay.
  - Tenant-scope mismatch fails closed.
- Drift patch applied (tests only):
  - `crates/selene_os/src/simulation_executor.rs`
    - added CAPREQ governance gate tests for `DENY`, `ESCALATE`, and tenant-scope mismatch fail-closed outcomes
  - `crates/selene_storage/tests/ph1_capreq/db_wiring.rs`
    - added approved-path idempotent replay test (`at_capreq_db_05_approved_path_idempotent_replay_executes_once`)
  - `crates/selene_os/src/ph1capreq.rs` (test module)
    - tightened invalid-transition assertion to deterministic `capreq_transition` reason text
- Proof runs:
  - `cargo test -p selene_storage --test db_wiring_access_tables -- --nocapture` -> pass (`4 passed; 0 failed`)
  - `cargo test -p selene_storage --test db_wiring_ph1capreq_tables -- --nocapture` -> pass (`5 passed; 0 failed`)
  - `cargo test -p selene_os capreq -- --nocapture` -> pass (`12 passed; 0 failed`)
  - `cargo test -p selene_os ph1onb -- --nocapture` -> pass (`10 passed; 0 failed`)
  - `cargo test -p selene_os ph1position -- --nocapture` -> pass (`3 passed; 0 failed`)

Step 8 note:
- Final proof run executed for packet closure:
  - `scripts/selene_design_readiness_audit.sh` -> pass (`EXIT:0`; expected dirty-tree note before freeze commit)
  - `cargo test -p selene_storage --test db_wiring_access_tables -- --nocapture` -> pass (`4 passed; 0 failed`)
  - `cargo test -p selene_storage --test db_wiring_ph1capreq_tables -- --nocapture` -> pass (`5 passed; 0 failed`)
  - `cargo test -p selene_os capreq -- --nocapture` -> pass (`12 passed; 0 failed`)
  - `cargo test -p selene_os ph1onb -- --nocapture` -> pass (`10 passed; 0 failed`)
  - `cargo test -p selene_os ph1position -- --nocapture` -> pass (`3 passed; 0 failed`)
  - `cargo test --workspace` -> pass (workspace green)
- Freeze checkpoint commit/proof is executed immediately after this packet update so closure artifacts include clean-tree status + pinned HEAD.

## 6) Done Criteria

This packet is done only when all are true:
1. Access remains the sole authority gate for governed side effects.
2. CAPREQ remains lifecycle-truth only and never grants authority alone.
3. Blueprint/simulation sequencing remains deterministic for deny/escalate/approve flows.
4. Runtime/storage/tests enforce fail-closed behavior for missing approvals, invalid transitions, and tenant mismatches.
5. Final readiness audit + tests pass from a clean pinned checkpoint.
