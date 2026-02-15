# PH1.ACCESS Execution Strict Fix Plan Packet (v1)

Last updated: 2026-02-15
Owner: Selene core design + runtime
Status: STEP8_COMPLETED

## 1) Purpose

This packet hardens PH1.ACCESS execution enforcement across governed runtime side effects so every impacted path fails closed on `DENY` and `ESCALATE`, not only CAPREQ.

Governance laws to preserve:
- `PH1.ACCESS.001_PH2.ACCESS.002` is the sole authority gate truth (`ALLOW | DENY | ESCALATE`).
- Runtime side effects must not execute when Access returns non-allow.
- `PH1.CAPREQ` remains lifecycle truth and never authority source.
- Selene OS orchestrates gating before engine commit side effects.

System laws:
- No Simulation -> No Execution.
- No silent authority changes.
- Fail closed on missing gate context.

## 2) Scope

1. Lock Access execution policy language in canonical docs.
2. Verify runtime dispatch surfaces that can produce governed side effects are Access-gated or explicitly non-governed.
3. Add/close tests for deny, escalate, scope mismatch, and missing-instance fail-closed outcomes.
4. Finish with clean-tree proof checkpoint (audit + targeted tests + workspace tests).

## 3) Baseline Gate (must run before Step 2)

```bash
cd "$(git rev-parse --show-toplevel)"
git rev-parse HEAD
git status --short

rg -n "PH1.ACCESS|PH2.ACCESS|ALLOW|DENY|ESCALATE|ACCESS_AP_REQUIRED|ACCESS_SCOPE_VIOLATION|simulation_candidate_dispatch|governed|fail closed|No Simulation -> No Execution" \
  docs/04_KERNEL_CONTRACTS.md \
  docs/05_OS_CONSTITUTION.md \
  docs/06_ENGINE_MAP.md \
  docs/07_ENGINE_REGISTRY.md \
  docs/08_SIMULATION_CATALOG.md \
  docs/09_BLUEPRINT_REGISTRY.md \
  docs/10_DB_OWNERSHIP_MATRIX.md \
  docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md \
  docs/ECM/PH1_ACCESS_001_PH2_ACCESS_002.md \
  docs/BLUEPRINTS/CAPREQ_MANAGE.md \
  docs/BLUEPRINTS/ONB_SCHEMA_MANAGE.md \
  docs/BLUEPRINTS/ONB_REQUIREMENT_BACKFILL.md \
  docs/BLUEPRINTS/POSITION_MANAGE.md \
  docs/BLUEPRINTS/LINK_INVITE.md \
  crates/selene_os/src/simulation_executor.rs \
  crates/selene_os/src/ph1onb.rs \
  crates/selene_os/src/ph1position.rs \
  crates/selene_os/src/ph1link.rs \
  crates/selene_storage/src/ph1f.rs \
  crates/selene_storage/src/repo.rs
```

## 4) Patch Order

### Step 1: Docs Lock + Packet Bootstrap

Patch files (only):
1. `docs/02_BUILD_PLAN.md`
2. `docs/28_PH1_ACCESS_EXECUTION_STRICT_FIX_PLAN_PACKET.md`

Patch intent:
- Freeze packet 27 closure in build plan.
- Set packet 28 as canonical next strict scope.

Post-step acceptance:

```bash
rg -n "40c25b8|28_PH1_ACCESS_EXECUTION_STRICT_FIX_PLAN_PACKET|Next Strict Packet" docs/02_BUILD_PLAN.md
test -f docs/28_PH1_ACCESS_EXECUTION_STRICT_FIX_PLAN_PACKET.md
```

---

### Step 2: Cross-Doc Access Execution Lock

Patch files (only if drift is found):
1. `docs/04_KERNEL_CONTRACTS.md`
2. `docs/05_OS_CONSTITUTION.md`
3. `docs/06_ENGINE_MAP.md`
4. `docs/07_ENGINE_REGISTRY.md`
5. `docs/10_DB_OWNERSHIP_MATRIX.md`

Patch intent:
- Remove ambiguity on where Access decisions must be enforced before governed side effects.

---

### Step 3: DB Wiring + ECM Access Lock

Patch files (only if drift is found):
1. `docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md`
2. `docs/DB_WIRING/PH1_ONB.md`
3. `docs/DB_WIRING/PH1_POSITION.md`
4. `docs/DB_WIRING/PH1_LINK.md`
5. `docs/ECM/PH1_ACCESS_001_PH2_ACCESS_002.md`

Patch intent:
- Lock explicit pre-commit Access gate expectations and fail-closed semantics in wiring surfaces.

---

### Step 4: Blueprint + Simulation Access Lock

Patch files (only if drift is found):
1. `docs/BLUEPRINTS/CAPREQ_MANAGE.md`
2. `docs/BLUEPRINTS/ONB_SCHEMA_MANAGE.md`
3. `docs/BLUEPRINTS/ONB_REQUIREMENT_BACKFILL.md`
4. `docs/BLUEPRINTS/POSITION_MANAGE.md`
5. `docs/BLUEPRINTS/LINK_INVITE.md`
6. `docs/08_SIMULATION_CATALOG.md`
7. `docs/09_BLUEPRINT_REGISTRY.md` (only if required)

Patch intent:
- Ensure governed blueprint steps clearly map to Access-gated commit behavior.

---

### Step 5: Runtime Dispatch Parity

Patch files (only if drift is found):
1. `crates/selene_os/src/simulation_executor.rs`
2. `crates/selene_os/src/ph1onb.rs`
3. `crates/selene_os/src/ph1position.rs`
4. `crates/selene_os/src/ph1link.rs`

Patch intent:
- Enforce fail-closed Access gating in governed dispatch paths with deterministic denial/escalation reasons.

---

### Step 6: Storage + Repo Parity

Patch files (only if drift is found):
1. `crates/selene_storage/src/repo.rs`
2. `crates/selene_storage/src/ph1f.rs`
3. `crates/selene_storage/migrations/0009_access_instance_tables.sql`
4. `crates/selene_storage/migrations/*` (only when parity requires)

Patch intent:
- Keep Access decision inputs/outputs deterministic, tenant-scoped, and idempotent.

---

### Step 7: Test Closure

Patch files (only if drift is found):
1. `crates/selene_os/src/simulation_executor.rs` (test module)
2. `crates/selene_os/src/ph1onb.rs` (test module)
3. `crates/selene_os/src/ph1position.rs` (test module)
4. `crates/selene_os/src/ph1link.rs` (test module)
5. `crates/selene_storage/tests/ph1_access_ph2_access/db_wiring.rs`

Required coverage:
- Missing access instance fails closed.
- `DENY` blocks governed commit.
- `ESCALATE` blocks commit pending approval path.
- Tenant scope mismatch fails closed.
- `ALLOW` path remains idempotent across retries.

---

### Step 8: Final Proof + Freeze Checkpoint

No patching in this step.

Run:

```bash
scripts/selene_design_readiness_audit.sh
cargo test -p selene_storage --test db_wiring_access_tables -- --nocapture
cargo test -p selene_os capreq -- --nocapture
cargo test -p selene_os ph1onb -- --nocapture
cargo test -p selene_os ph1position -- --nocapture
cargo test -p selene_os ph1link -- --nocapture
cargo test --workspace
git status --short
git rev-parse HEAD
git log -1 --oneline
```

Checkpoint expectations:
- Access execution boundaries are coherent across docs/contracts/runtime/storage/tests.
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
- Updated `docs/02_BUILD_PLAN.md` to freeze packet 27 closure checkpoint (`40c25b8`) and set this packet as canonical next strict scope.
- Created this packet as the strict execution plan for PH1.ACCESS runtime dispatch closure.

Step 2 note:
- Cross-doc Access execution lock review completed for:
  - `docs/04_KERNEL_CONTRACTS.md`
  - `docs/05_OS_CONSTITUTION.md`
  - `docs/06_ENGINE_MAP.md`
  - `docs/07_ENGINE_REGISTRY.md`
  - `docs/10_DB_OWNERSHIP_MATRIX.md`
- No drift patch was required in Step 2.
- Verified lock points:
  - Access decision model remains canonical (`ALLOW | DENY | ESCALATE`) in contract/map surfaces.
  - Cross-doc wording keeps governed side effects simulation-gated and Access-gated before commit execution.
  - No ownership inversion introduced between Access and CAPREQ in these docs.

Step 3 note:
- DB wiring + ECM Access lock review completed for:
  - `docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md`
  - `docs/DB_WIRING/PH1_ONB.md`
  - `docs/DB_WIRING/PH1_POSITION.md`
  - `docs/DB_WIRING/PH1_LINK.md`
  - `docs/ECM/PH1_ACCESS_001_PH2_ACCESS_002.md`
- Drift patch applied (docs-only):
  - added explicit pre-commit Access enforcement wording for governed paths (`ALLOW` required; `DENY|ESCALATE` fail-closed) across ONB/POSITION/LINK DB wiring surfaces
  - added explicit consumer/caller fail-closed obligations in Access DB wiring + ECM surfaces
- Verified lock points:
  - governed commit paths now have explicit Access precondition language in all Step-3 target docs
  - no ownership inversion introduced between Access and CAPREQ

Step 4 note:
- Blueprint + simulation Access lock review completed for:
  - `docs/BLUEPRINTS/CAPREQ_MANAGE.md`
  - `docs/BLUEPRINTS/ONB_SCHEMA_MANAGE.md`
  - `docs/BLUEPRINTS/ONB_REQUIREMENT_BACKFILL.md`
  - `docs/BLUEPRINTS/POSITION_MANAGE.md`
  - `docs/BLUEPRINTS/LINK_INVITE.md`
  - `docs/08_SIMULATION_CATALOG.md`
  - `docs/09_BLUEPRINT_REGISTRY.md`
- No drift patch was required in Step 4.
- Verified lock points:
  - all Step-4 blueprints include explicit `ACCESS_GATE_DECIDE_ROW` pre-commit gate and fail-closed refusal conditions for `DENY` and unresolved `ESCALATE`
  - all Step-4 blueprint simulation requirements resolve to existing simulation catalog rows
  - all Step-4 intents remain `ACTIVE` and uniquely registered in blueprint registry

Step 5 note:
- Runtime dispatch parity review completed for:
  - `crates/selene_os/src/simulation_executor.rs`
  - `crates/selene_os/src/ph1onb.rs`
  - `crates/selene_os/src/ph1position.rs`
  - `crates/selene_os/src/ph1link.rs`
- Drift patch applied in `simulation_executor`:
  - added fail-closed Access gate enforcement for `CreateInviteLink` simulation-candidate dispatch using `requested_action=LINK_INVITE`
  - `DENY` now returns deterministic `ACCESS_SCOPE_VIOLATION`
  - unresolved `ESCALATE` now returns deterministic `ACCESS_AP_REQUIRED`
  - missing actor+tenant access instance now fails closed before link generation commit path
- Runtime surface assessment:
  - no additional Step-5 patch was required in `ph1onb.rs`, `ph1position.rs`, `ph1link.rs` for simulation-candidate dispatch closure in this packet scope
- Verification:
  - `cargo test -p selene_os at_sim_exec_ -- --nocapture` passed with new link access deny/escalate/scope tests included

Step 6 note:
- Storage + repo parity review completed for:
  - `crates/selene_storage/src/repo.rs`
  - `crates/selene_storage/src/ph1f.rs`
  - `crates/selene_storage/migrations/0009_access_instance_tables.sql`
- Drift patch applied in `ph1f.rs`:
  - access gate now enforces `requested_action` against explicit baseline `allow` list when present (`allow=["*"]` supported; legacy rows without allow-list remain backward-compatible)
  - device trust decision now uses strict lower-bound trust (`min(request_context_trust, instance.device_trust_level)`) before high-impact mode checks
- Repo/migration assessment:
  - no Step-6 patch required in `repo.rs` (trait/impl parity already consistent with store methods)
  - no Step-6 patch required in `0009_access_instance_tables.sql` (tenant/idempotency constraints already present for access instances and overrides)
- Verification:
  - `cargo test -p selene_storage --test db_wiring_access_tables -- --nocapture` passed
  - `cargo test -p selene_os at_sim_exec_ -- --nocapture` passed after storage parity patch

Step 7 note:
- Test-closure review completed for:
  - `crates/selene_os/src/simulation_executor.rs` (patched)
  - `crates/selene_storage/tests/ph1_access_ph2_access/db_wiring.rs` (patched)
  - `crates/selene_os/src/ph1onb.rs` (no Step-7 drift patch required)
  - `crates/selene_os/src/ph1position.rs` (no Step-7 drift patch required)
  - `crates/selene_os/src/ph1link.rs` (no Step-7 drift patch required)
- Added/closed required coverage:
  - missing access instance fails closed
  - `DENY` blocks governed commit
  - `ESCALATE` blocks governed commit pending approval
  - tenant/user scope mismatch fails closed
  - allow path is deterministic/idempotent across retries
- Verification:
  - `cargo test -p selene_storage --test db_wiring_access_tables -- --nocapture` passed (9 tests)
  - `cargo test -p selene_os at_sim_exec_ -- --nocapture` passed (17 tests)

Step 8 note:
- Final proof sequence executed end-to-end at clean checkpoint `e35c0f5cd4cab1b6acb6e5582f4a3dc8b770e73e`:
  - `scripts/selene_design_readiness_audit.sh` passed (`AUDIT_TREE_STATE: CLEAN`)
  - `cargo test -p selene_storage --test db_wiring_access_tables -- --nocapture` passed
  - `cargo test -p selene_os capreq -- --nocapture` passed
  - `cargo test -p selene_os ph1onb -- --nocapture` passed
  - `cargo test -p selene_os ph1position -- --nocapture` passed
  - `cargo test -p selene_os ph1link -- --nocapture` passed
  - `cargo test --workspace` passed
- Freeze checkpoint criteria satisfied: docs/contracts/runtime/storage/tests coherent and clean-tree proof complete.

## 6) Done Criteria

This packet is done only when all are true:
1. Governed runtime side effects are blocked on `DENY` and `ESCALATE`.
2. Missing access context fails closed deterministically.
3. Tenant-scope mismatches fail closed across touched paths.
4. Idempotent allow-path behavior is preserved.
5. Final readiness audit + tests pass from a clean pinned checkpoint.
