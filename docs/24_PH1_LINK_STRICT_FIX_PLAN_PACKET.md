# PH1.LINK Strict Fix Plan Packet (v3)

Last updated: 2026-02-15
Owner: Selene core design + runtime
Status: STEP8_COMPLETED_FROZEN

## 1) Purpose

This packet is the strict execution plan to re-lock PH1.LINK after PH1.POSITION schema-ownership closure.

Rules:
- No Simulation -> No Execution.
- Engines never call engines directly; Selene OS orchestrates.
- PH1.LINK owns invite lifecycle + selector-hint capture only.
- PH1.POSITION owns requirements-schema truth.
- PH1.ONB executes pinned schema context only.

## 2) Scope

1. Keep PH1.LINK as canonical owner of draft/token/open/activate/revoke/block lifecycle truth.
2. Keep selector-hint capture deterministic, bounded, and tenant-safe.
3. Prevent schema-ownership drift into PH1.LINK surfaces.
4. Keep LINK <-> ONB handoff deterministic (token + prefilled hints only).
5. Revalidate runtime/storage idempotency and fail-closed behavior for critical LINK transitions.

## 3) Baseline Gate (must run before Step 2)

```bash
cd "$(git rev-parse --show-toplevel)"
git rev-parse HEAD
git status --short

rg -n "PH1.LINK|selector hint|prefilled|schema truth|LINK_INVITE_DRAFT_UPDATE_COMMIT|LINK_INVITE_OPEN_ACTIVATE_COMMIT|LINK_INVITE_FORWARD_BLOCK_COMMIT|LINK_DELIVER_INVITE" \
  docs/04_KERNEL_CONTRACTS.md \
  docs/05_OS_CONSTITUTION.md \
  docs/06_ENGINE_MAP.md \
  docs/07_ENGINE_REGISTRY.md \
  docs/10_DB_OWNERSHIP_MATRIX.md \
  docs/DB_WIRING/PH1_LINK.md \
  docs/ECM/PH1_LINK.md \
  docs/BLUEPRINTS/LINK_INVITE.md \
  docs/BLUEPRINTS/LINK_DELIVER_INVITE.md \
  docs/BLUEPRINTS/LINK_OPEN_ACTIVATE.md
```

## 4) Patch Order

### Step 1: Docs Lock + Packet Bootstrap

Patch files (only):
1. `docs/02_BUILD_PLAN.md`
2. `docs/24_PH1_LINK_STRICT_FIX_PLAN_PACKET.md`

Patch intent:
- Freeze packet 23 closure in build plan.
- Set packet 24 as canonical next strict scope.

Post-step acceptance:

```bash
rg -n "35a25bc|24_PH1_LINK_STRICT_FIX_PLAN_PACKET|Next Strict Packet" docs/02_BUILD_PLAN.md
test -f docs/24_PH1_LINK_STRICT_FIX_PLAN_PACKET.md
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
- Keep LINK ownership and boundaries exact across canonical docs.

---

### Step 3: DB Wiring + ECM Lock

Patch files (only if drift is found):
1. `docs/DB_WIRING/PH1_LINK.md`
2. `docs/DB_WIRING/PH1_ONB.md` (only if handoff wording drift found)
3. `docs/DB_WIRING/PH1_POSITION.md` (only if ownership wording drift found)
4. `docs/ECM/PH1_LINK.md`
5. `docs/ECM/PH1_ONB.md` (only if handoff wording drift found)
6. `docs/ECM/PH1_POSITION.md` (only if ownership wording drift found)

Patch intent:
- Lock LINK lifecycle ownership and selector-hint-only boundaries.

---

### Step 4: Blueprint + Simulation Lock

Patch files (only if drift is found):
1. `docs/BLUEPRINTS/LINK_INVITE.md`
2. `docs/BLUEPRINTS/LINK_DELIVER_INVITE.md`
3. `docs/BLUEPRINTS/LINK_OPEN_ACTIVATE.md`
4. `docs/08_SIMULATION_CATALOG.md`
5. `docs/09_BLUEPRINT_REGISTRY.md` (only if required)

Patch intent:
- Ensure link blueprints/simulation requirements stay coherent and non-duplicative.

---

### Step 5: Kernel + Runtime Parity

Patch files (only if drift is found):
1. `crates/selene_kernel_contracts/src/ph1link.rs`
2. `crates/selene_os/src/ph1link.rs`
3. `crates/selene_os/src/simulation_executor.rs` (only if required)

Patch intent:
- Ensure request/response and runtime dispatch boundaries are exact.

---

### Step 6: Storage + SQL Parity

Patch files (only if drift is found):
1. `crates/selene_storage/src/repo.rs`
2. `crates/selene_storage/src/ph1f.rs`
3. `crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql` (only when SQL parity requires)

Patch intent:
- Keep deterministic lifecycle/idempotency + SQL enum/index parity.

---

### Step 7: Test Closure

Patch files (only if drift is found):
1. `crates/selene_storage/tests/ph1_link/db_wiring.rs`
2. `crates/selene_os/src/ph1link.rs` (test module)
3. `crates/selene_os/src/ph1onb.rs` (only if LINK->ONB handoff tests require extension)

Required coverage:
- Draft update idempotent replay.
- Open/activate idempotent replay.
- Revoke guard correctness.
- Forward-block single deterministic path.
- Selector-hint/prefilled handoff consistency to ONB.

---

### Step 8: Final Proof + Freeze Checkpoint

No patching in this step.

Run:

```bash
scripts/selene_design_readiness_audit.sh
cargo test -p selene_storage --test db_wiring_ph1link_tables -- --nocapture
cargo test -p selene_os ph1link -- --nocapture
cargo test --workspace
git status --short
git rev-parse HEAD
git log -1 --oneline
```

Checkpoint expectations:
- PH1.LINK docs/contracts/runtime/storage/tests are coherent.
- Audit passes with clean tree checkpoint.
- Pinned commit hash and closure evidence are recorded.

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
- Updated `docs/02_BUILD_PLAN.md` to freeze packet 23 closure checkpoint (`35a25bc`) and point next strict scope to this packet.
- Created this packet as the canonical strict execution plan for PH1.LINK closure refresh.

Step 2 note:
- Cross-doc ownership lock review completed for:
  - `docs/05_OS_CONSTITUTION.md`
  - `docs/06_ENGINE_MAP.md`
  - `docs/07_ENGINE_REGISTRY.md`
  - `docs/10_DB_OWNERSHIP_MATRIX.md`
  - `docs/04_KERNEL_CONTRACTS.md`
- No drift patch was required in Step 2.
- Verified canonical boundary wording is already aligned:
  - `PH1.LINK` owns invite lifecycle + selector-hint capture only.
  - `PH1.POSITION` owns requirements-schema truth.
  - `PH1.ONB` executes pinned schema only and does not own schema definitions.

Step 3 note:
- DB_WIRING + ECM lock review completed for:
  - `docs/DB_WIRING/PH1_LINK.md`
  - `docs/DB_WIRING/PH1_ONB.md`
  - `docs/DB_WIRING/PH1_POSITION.md`
  - `docs/ECM/PH1_LINK.md`
  - `docs/ECM/PH1_ONB.md`
  - `docs/ECM/PH1_POSITION.md`
- No drift patch was required in Step 3.
- Verified boundary wording remains coherent:
  - PH1.LINK DB/ECM: lifecycle + selector-hint capture only; no schema truth ownership.
  - PH1.ONB DB/ECM: execution-only from pinned schema context.
  - PH1.POSITION DB/ECM: requirements-schema ownership and lifecycle writes.

Step 4 note:
- Blueprint + simulation lock review completed for:
  - `docs/BLUEPRINTS/LINK_INVITE.md`
  - `docs/BLUEPRINTS/LINK_DELIVER_INVITE.md`
  - `docs/BLUEPRINTS/LINK_OPEN_ACTIVATE.md`
  - `docs/08_SIMULATION_CATALOG.md`
  - `docs/09_BLUEPRINT_REGISTRY.md`
- No drift patch was required in Step 4.
- Verified coherence:
  - Blueprint registry remains ACTIVE for all three LINK blueprints.
  - LINK blueprints reference canonical LINK simulations and keep delivery ownership in `LINK_DELIVER_INVITE`.
  - Legacy LINK delivery simulations remain explicitly `LEGACY_DO_NOT_WIRE` in simulation catalog (expected compliance evidence).

Step 5 note:
- Kernel + runtime parity review completed for:
  - `crates/selene_kernel_contracts/src/ph1link.rs`
  - `crates/selene_os/src/ph1link.rs`
  - `crates/selene_os/src/simulation_executor.rs`
- No drift patch was required in Step 5.
- Verified parity:
  - Canonical LINK simulation constants are present and stable in kernel contracts.
  - `LinkRequest::simulation_id()` mapping matches request constructors and canonical simulation IDs.
  - PH1.LINK runtime dispatch remains aligned with LINK request variants and reason-coded outcomes.
  - Simulation executor LINK path remains wired to PH1.LINK runtime.
- Proof runs:
  - `cargo test -p selene_os ph1link -- --nocapture` -> pass (`9 passed; 0 failed`)
  - `cargo test -p selene_kernel_contracts ph1link -- --nocapture` -> pass (`0 passed; 0 failed`; filtered target only)

Step 6 note:
- Storage + SQL parity review completed for:
  - `crates/selene_storage/src/repo.rs`
  - `crates/selene_storage/src/ph1f.rs`
  - `crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql`
- Drift found and fixed in storage adapter (`Ph1LinkRepo for Ph1fStore`):
  - `ph1link_invite_draft_update_commit_row` now calls `ph1link_invite_draft_update_commit` (removed stale not-implemented stub).
  - `ph1link_invite_open_activate_commit_row_with_idempotency` now passes through `idempotency_key` to `ph1link_invite_open_activate_commit_with_idempotency` (no longer ignored).
- SQL parity check:
  - `onboarding_link_tokens.status` CHECK list remains aligned with kernel `LinkStatus` enum.
  - Link dedupe scope CHECK list remains aligned with LINK operation-scoped idempotency domains.
- Proof runs:
  - `cargo test -p selene_storage --test db_wiring_ph1link_tables -- --nocapture` -> pass (`11 passed; 0 failed`)
  - `cargo test -p selene_storage ph1link -- --nocapture` -> pass (`0 failed`; filtered run across package targets)

Step 7 note:
- Test closure completed for:
  - `crates/selene_storage/tests/ph1_link/db_wiring.rs`
  - `crates/selene_os/src/ph1link.rs` (existing LINK runtime tests re-run)
  - `crates/selene_os/src/ph1onb.rs` (existing ONB handoff path re-run)
- Drift found and fixed in coverage:
  - Added storage row-adapter regression test for draft-update idempotent replay:
    - `at_link_db_12_draft_update_row_method_is_idempotent`
  - Added storage row-adapter regression test for open/activate idempotency-key replay:
    - `at_link_db_13_open_activate_row_with_idempotency_replays_by_key`
- Required coverage checklist is now explicitly proven:
  - Draft update idempotent replay.
  - Open/activate idempotent replay.
  - Revoke guard correctness.
  - Forward-block single deterministic path.
  - Selector-hint/prefilled handoff consistency to ONB (via ONB happy-path run).
- Proof runs:
  - `cargo test -p selene_storage --test db_wiring_ph1link_tables -- --nocapture` -> pass (`13 passed; 0 failed`)
  - `cargo test -p selene_os ph1link -- --nocapture` -> pass (`9 passed; 0 failed`)
  - `cargo test -p selene_os onb_happy_path_employee_minimal -- --nocapture` -> pass (`1 passed; 0 failed`)

Step 8 note:
- Final proof suite completed:
  - `scripts/selene_design_readiness_audit.sh` -> pass (AUDIT_TREE_STATE captured with pinned hash at run time).
  - `cargo test -p selene_storage --test db_wiring_ph1link_tables -- --nocapture` -> pass (`13 passed; 0 failed`).
  - `cargo test -p selene_os ph1link -- --nocapture` -> pass (`9 passed; 0 failed`).
  - `cargo test --workspace` -> pass (all workspace/unit/integration/doc tests green).
- Freeze checkpoint action:
  - Packet closed with Step 8 completion and committed as a pinned baseline.
  - Post-commit proof commands must report clean tree + pinned commit hash.

## 6) Done Criteria

This packet is done only when all are true:
1. PH1.LINK boundaries are explicit and unchanged across docs/contracts/runtime/storage.
2. Selector-hint capture is deterministic and never treated as schema truth ownership.
3. Critical LINK lifecycle transitions are deterministic/idempotent/fail-closed and test-proven.
4. Final readiness audit + tests pass at a clean pinned checkpoint.
