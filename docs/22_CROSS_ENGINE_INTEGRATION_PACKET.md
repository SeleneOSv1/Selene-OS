# Cross-Engine Integration Strict Fix Plan Packet

Last updated: 2026-02-15
Owner: Selene core design + runtime
Status: STEP8_COMPLETED_READY_FOR_NEXT_PACKET

## 1) Purpose

This packet is the single step-by-step plan to close cross-engine integration seams across:
- `PH1.LINK` (draft + selector hints + token lifecycle)
- `PH1.ONB` (pinned schema execution + backfill orchestration)
- `PH1.POSITION` (requirements-schema ownership + rollout scope)
- `PH1.CAPREQ` + `PH1.ACCESS` (approval/permission gate path)
- `PH1.BCAST` + `PH1.REM` (current-staff backfill delivery/reminder loop)

Rules:
- Do not skip steps.
- Do not patch files outside each step scope.
- Keep `No Simulation -> No Execution`.
- Keep engine boundaries (Selene OS orchestrates; engines do not call engines directly).

## 2) Scope (what this packet validates/fixes)

1. LINK-to-ONB handoff is deterministic and schema-driven (selector hints -> schema pin -> one-question execution).
2. POSITION schema activation/rollout scope hands off cleanly to ONB backfill behavior.
3. CAPREQ + ACCESS gate path is explicit for governed schema changes.
4. BCAST/REM integration for current-staff backfill remains ownership-correct and fail-closed.
5. Docs/contracts/runtime/storage/tests remain coherent for end-to-end integration paths.

## 3) Baseline Gate (must run before Step 1)

```bash
cd "$(git rev-parse --show-toplevel)"
git rev-parse HEAD
git status --short

rg -n "LINK|ONB|POSITION|CAPREQ|ACCESS|BCAST|REM|schema|backfill|CurrentAndNew|pinned_schema" \
  docs/05_OS_CONSTITUTION.md \
  docs/06_ENGINE_MAP.md \
  docs/07_ENGINE_REGISTRY.md \
  docs/10_DB_OWNERSHIP_MATRIX.md \
  docs/DB_WIRING/PH1_LINK.md \
  docs/DB_WIRING/PH1_ONB.md \
  docs/DB_WIRING/PH1_POSITION.md \
  docs/DB_WIRING/PH1_CAPREQ.md \
  docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md \
  docs/DB_WIRING/PH1_BCAST.md \
  docs/DB_WIRING/PH1_REM.md
```

## 4) Patch Order

### Step 1: Docs Lock + Packet Bootstrap

Patch files (only):
1. `docs/02_BUILD_PLAN.md`
2. `docs/22_CROSS_ENGINE_INTEGRATION_PACKET.md`

Patch intent:
- Freeze current packet-wave checkpoint in the build plan.
- Create this canonical strict packet for cross-engine integration closure.

Post-step acceptance:

```bash
rg -n "Strict Packet Checkpoint|Next Strict Packet|22_CROSS_ENGINE_INTEGRATION_PACKET" docs/02_BUILD_PLAN.md
test -f docs/22_CROSS_ENGINE_INTEGRATION_PACKET.md
```

Step 1 exit criteria:
- Build plan contains current checkpoint + next-packet pointer.
- Packet file exists and is set as canonical scope for cross-engine integration work.

---

### Step 2: Cross-Doc Integration Lock

Patch files (only if drift is found):
1. `docs/05_OS_CONSTITUTION.md`
2. `docs/06_ENGINE_MAP.md`
3. `docs/07_ENGINE_REGISTRY.md`
4. `docs/10_DB_OWNERSHIP_MATRIX.md`
5. `docs/04_KERNEL_CONTRACTS.md` (only if required for cross-slice contract wording parity)

Patch intent:
- Lock ownership boundaries and integration handoff wording across LINK/ONB/POSITION/CAPREQ/ACCESS/BCAST/REM.

---

### Step 3: DB Wiring + ECM Integration Lock

Patch files (only if drift is found):
1. `docs/DB_WIRING/PH1_LINK.md`
2. `docs/DB_WIRING/PH1_ONB.md`
3. `docs/DB_WIRING/PH1_POSITION.md`
4. `docs/DB_WIRING/PH1_CAPREQ.md`
5. `docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md`
6. `docs/DB_WIRING/PH1_BCAST.md`
7. `docs/DB_WIRING/PH1_REM.md`
8. `docs/ECM/PH1_LINK.md`
9. `docs/ECM/PH1_ONB.md`
10. `docs/ECM/PH1_POSITION.md`
11. `docs/ECM/PH1_CAPREQ.md`
12. `docs/ECM/PH1_BCAST.md`
13. `docs/ECM/PH1_REM.md`

Patch intent:
- Ensure capability ownership, row methods, and lifecycle responsibilities are coherent at integration boundaries.

---

### Step 4: Blueprint + Simulation Integration Lock

Patch files (only if drift is found):
1. `docs/BLUEPRINTS/LINK_INVITE.md`
2. `docs/BLUEPRINTS/LINK_OPEN_ACTIVATE.md`
3. `docs/BLUEPRINTS/ONB_INVITED.md`
4. `docs/BLUEPRINTS/ONB_SCHEMA_MANAGE.md`
5. `docs/BLUEPRINTS/ONB_REQUIREMENT_BACKFILL.md`
6. `docs/BLUEPRINTS/POSITION_MANAGE.md`
7. `docs/BLUEPRINTS/CAPREQ_MANAGE.md`
8. `docs/08_SIMULATION_CATALOG.md`
9. `docs/09_BLUEPRINT_REGISTRY.md` (only if required)

Patch intent:
- Lock simulation ownership and step sequencing for end-to-end integration flows.

---

### Step 5: Kernel + Runtime Parity

Patch files (only if drift is found):
1. `crates/selene_kernel_contracts/src/ph1link.rs`
2. `crates/selene_kernel_contracts/src/ph1onb.rs`
3. `crates/selene_kernel_contracts/src/ph1position.rs`
4. `crates/selene_kernel_contracts/src/ph1capreq.rs`
5. `crates/selene_os/src/ph1link.rs`
6. `crates/selene_os/src/ph1onb.rs`
7. `crates/selene_os/src/ph1position.rs`
8. `crates/selene_os/src/ph1capreq.rs`
9. `crates/selene_os/src/simulation_executor.rs`

Patch intent:
- Enforce deterministic integration behavior and fail-closed gating across runtime orchestration surfaces.

---

### Step 6: Storage + Migration Parity

Patch files (only if drift is found):
1. `crates/selene_storage/src/repo.rs`
2. `crates/selene_storage/src/ph1f.rs`
3. `crates/selene_storage/migrations/*` (only when schema/index parity requires a migration delta)

Patch intent:
- Ensure storage interfaces and SQL constraints support integration semantics and idempotency guarantees.

---

### Step 7: Integration Test Closure

Patch files (only if drift is found):
1. `crates/selene_storage/tests/ph1_link/db_wiring.rs`
2. `crates/selene_storage/tests/ph1_onb/db_wiring.rs`
3. `crates/selene_storage/tests/ph1_position/db_wiring.rs`
4. `crates/selene_storage/tests/ph1_capreq/db_wiring.rs`
5. `crates/selene_os/src/ph1link.rs` (test module)
6. `crates/selene_os/src/ph1onb.rs` (test module)
7. `crates/selene_os/src/ph1position.rs` (test module)
8. `crates/selene_os/src/ph1capreq.rs` (test module)

Required coverage:
- LINK selector-hint handoff -> ONB schema pin path.
- POSITION requirements change with `CurrentAndNew` -> ONB backfill campaign path.
- CAPREQ approval/denial path for governed schema changes.
- BCAST + REM reminder loop and deterministic ONB completion behavior.
- Fail-closed behavior when required approval/access/simulation gates are missing.

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
- Cross-engine integration drifts are closed or explicitly tracked.
- Workspace tests pass.
- Readiness audit is clean for the pinned checkpoint.

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
- Added strict checkpoint + next-packet pointer to `docs/02_BUILD_PLAN.md`.
- Created this packet as the canonical cross-engine integration closure plan.

Step 2 note:
- Cross-doc integration lock completed for LINK/ONB/POSITION/CAPREQ/ACCESS/BCAST/REM boundaries.
- `PH1.REM` was promoted from stale planned/stub summary text to active authoritative ownership/registry wording.

Step 3 note:
- DB wiring + ECM integration lock completed for cross-engine seams:
  - normalized POSITION `apply_scope` tokens to `NewHiresOnly | CurrentAndNew` for parity with ONB/kernel naming,
  - added explicit CAPREQ governance boundary lock (CAPREQ lifecycle truth vs ACCESS/AP authority),
  - added explicit ONB/LINK/REM ECM boundary notes for pinned-schema ownership, selector-hint handoff, and backfill reminder timing ownership.

Step 4 note:
- Blueprint + simulation integration lock completed for scope-token parity and handoff sequencing:
  - normalized `ONB_SCHEMA_MANAGE` `apply_scope`/`apply_scope_result` token usage to `NewHiresOnly | CurrentAndNew`,
  - aligned `POSITION_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT` simulation input/output contract to `apply_scope` with `apply_scope_result` + `backfill_handoff_required`,
  - preserved deterministic launch condition: `apply_scope=CurrentAndNew` triggers ONB backfill start flow.

Step 5 note:
- Kernel/runtime parity sweep completed for Step 5 scope with no code deltas required:
  - `PH1.POSITION` contracts/runtime already enforce `apply_scope` + `apply_scope_result` + `backfill_handoff_required` invariants,
  - `PH1.ONB` backfill start remains fail-closed on `rollout_scope=CurrentAndNew`,
  - `PH1.CAPREQ` submit-for-approval simulation naming and runtime dispatch are coherent.

Step 6 note:
- Storage + migration parity lock completed with targeted SQL constraint alignment:
  - updated `0014_position_requirements_schema_and_backfill_tables.sql` `apply_scope` check tokens to `NewHiresOnly | CurrentAndNew`,
  - tightened backfill campaign `rollout_scope` check to `CurrentAndNew` only, matching ONB contract fail-closed semantics.

Step 7 note:
- Integration test closure completed with explicit CAPREQ runtime path coverage added in `crates/selene_os/src/ph1capreq.rs`:
  - create -> submit_for_approval -> approve happy path,
  - create -> submit_for_approval -> reject happy path,
  - fail-closed approve without pending approval.
- Existing Step 7 required coverage was re-verified as present:
  - LINK selector-hint handoff -> ONB pinned schema path,
  - POSITION `CurrentAndNew` handoff -> ONB backfill start/notify/complete path,
  - ONB reminder-loop and fail-closed tenant/target gate behavior.
- Proof command executed successfully:
  - `cargo test -p selene_os -p selene_storage`

Step 8 note:
- Final proof run executed for this packet scope:
  - `scripts/selene_design_readiness_audit.sh`
  - `cargo test --workspace`
  - `git status --short`
  - `git rev-parse HEAD`
  - `git log -1 --oneline`
- Audit and tests passed; checkpoint remained dirty until commit due to intentional packet-scope deltas.

## 6) Done Criteria

Cross-engine integration closure is done only when all are true:
1. LINK -> ONB -> POSITION -> CAPREQ handoffs are coherent across docs/contracts/runtime/storage.
2. Backfill and approval paths are deterministic, simulation-gated, and fail-closed.
3. Integration test coverage proves required end-to-end paths.
4. Final readiness audit + workspace tests pass at a pinned clean checkpoint.
