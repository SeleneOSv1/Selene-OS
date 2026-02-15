# ONB Backfill Strict Fix Plan Packet

Last updated: 2026-02-15
Owner: Selene core design + runtime
Status: EXECUTED_PENDING_COMMIT

## 1) Purpose

This packet is the single step-by-step plan to close ONB backfill orchestration drift for:
- `ONB_REQUIREMENT_BACKFILL` blueprint
- `PH1.ONB` backfill contracts/runtime
- `PH1.BCAST` + `PH1.REM` handoff semantics

Rules:
- Do not skip steps.
- Do not patch files outside each step scope.
- Keep `No Simulation -> No Execution`.
- Keep engine boundaries (Selene OS orchestrates; engines do not call engines directly).

## 2) Scope (what this packet fixes)

1. Blueprint flow/table must match ONB backfill capability set exactly.
2. Per-recipient notify/progress commit (`ONB_REQUIREMENT_BACKFILL_NOTIFY_COMMIT`) must be explicit in the blueprint path.
3. `rollout_scope` and campaign/target state naming must be consistent across blueprint + ECM + simulation catalog + kernel contracts.
4. BCAST/REM handoff semantics must be explicitly tied to ONB backfill loop without changing ownership boundaries.
5. Backfill tests must prove deterministic progress + completion behavior.

## 3) Baseline Gate (must run before Step 1)

```bash
cd "$(git rev-parse --show-toplevel)"
git rev-parse HEAD
git status --short

rg -n "ONB_REQUIREMENT_BACKFILL|BACKFILL|rollout_scope|PH1ONB_BACKFILL_NOTIFY_COMMIT_ROW|REMINDER_SCHEDULE_COMMIT|BCAST_DRAFT_CREATE|BCAST_DELIVER_COMMIT" \
  docs/BLUEPRINTS/ONB_REQUIREMENT_BACKFILL.md \
  docs/DB_WIRING/PH1_ONB.md \
  docs/ECM/PH1_ONB.md \
  docs/DB_WIRING/PH1_BCAST.md \
  docs/DB_WIRING/PH1_REM.md \
  docs/08_SIMULATION_CATALOG.md \
  crates/selene_kernel_contracts/src/ph1onb.rs
```

Expected baseline evidence:
- Blueprint references `ONB_REQUIREMENT_BACKFILL_NOTIFY_COMMIT` in simulation requirements.
- Step table may not explicitly include the ONB notify commit stage.
- Simulation catalog schema fields may drift from kernel contract enum/state names.

## 4) Patch Order

### Step 1: Docs Lock (blueprint + contracts docs + sim catalog)

Patch files (only):
1. `docs/BLUEPRINTS/ONB_REQUIREMENT_BACKFILL.md`
2. `docs/DB_WIRING/PH1_ONB.md`
3. `docs/ECM/PH1_ONB.md`
4. `docs/DB_WIRING/PH1_BCAST.md`
5. `docs/DB_WIRING/PH1_REM.md`
6. `docs/08_SIMULATION_CATALOG.md`

Patch intent:
- Add explicit ONB notify/progress step to the blueprint step table.
- Lock deterministic per-recipient loop ordering:
  - backfill start
  - bcast create
  - bcast deliver
  - rem schedule
  - onb notify commit
  - onb complete
- Align naming with kernel contracts:
  - `rollout_scope: CurrentAndNew` (process entry requirement)
  - campaign state enum and target status enum naming
- Keep ownership split explicit:
  - ONB owns campaign/target progress state
  - BCAST owns message lifecycle/delivery
  - REM owns timing mechanics only

Post-step acceptance:

```bash
rg -n "PH1ONB_BACKFILL_NOTIFY_COMMIT_ROW|ONB_BACKFILL_S09|CurrentAndNew|BackfillCampaignState|BackfillTargetStatus|BCAST.MHP.REM" \
  docs/BLUEPRINTS/ONB_REQUIREMENT_BACKFILL.md \
  docs/DB_WIRING/PH1_ONB.md \
  docs/ECM/PH1_ONB.md \
  docs/DB_WIRING/PH1_BCAST.md \
  docs/DB_WIRING/PH1_REM.md \
  docs/08_SIMULATION_CATALOG.md
```

Step 1 exit criteria:
- Backfill flow + ownership + enum naming are locked consistently in docs.

---

### Step 2: Kernel Contract Delta (PH1.ONB backfill payload parity)

Patch files (only):
1. `crates/selene_kernel_contracts/src/ph1onb.rs`

Patch intent:
- Align request/result payload shapes with Step 1 lock.
- Keep backward-compatible validation where possible.

---

### Step 3: Typed Repo Surface Parity

Patch files (only):
1. `crates/selene_storage/src/repo.rs`

Patch intent:
- Add/align explicit backfill repo methods for start/notify/complete.

---

### Step 4: Storage Behavior Fixes (PH1.ONB backfill)

Patch files (only):
1. `crates/selene_storage/src/ph1f.rs`

Patch intent:
- Ensure deterministic backfill campaign/target transitions and idempotency guarantees.
- Ensure notify/complete transitions are fail-closed on tenant/campaign scope mismatch.

---

### Step 5: Runtime Wiring (PH1.ONB orchestration)

Patch files (only):
1. `crates/selene_os/src/ph1onb.rs`
2. `crates/selene_os/src/simulation_executor.rs` (only if required)

Patch intent:
- Ensure runtime dispatch and reason-coded transitions for start/notify/complete match locked docs/contracts.

---

### Step 6: SQL/Migration Parity (conditional)

Patch files (only, conditional):
1. `crates/selene_storage/migrations/*onb*` (only if table changes are needed)

Patch intent:
- Confirm storage schema parity for campaign/target state and idempotency indexes.
- If no migration changes are needed, record explicit N/A.

---

### Step 7: Test Closure

Patch files (only):
1. `crates/selene_storage/tests/ph1_onb/db_wiring.rs`
2. `crates/selene_os/src/ph1onb.rs` (test module)

Required tests:
- start draft idempotency + deterministic target snapshot count
- notify commit idempotency + target status transitions
- complete commit idempotency + deterministic completed/total counts
- tenant/campaign scope fail-closed checks

---

### Step 8: Final Drift Proof + Audit Checkpoint

No patching in this step.

Run:

```bash
cargo test -p selene_storage --test db_wiring_ph1onb_tables -- --nocapture
cargo test -p selene_os ph1onb -- --nocapture
scripts/selene_design_readiness_audit.sh
git status --short
git rev-parse HEAD
git log -1 --oneline
```

## 5) Execution Record (fill during work)

- Step 1: COMPLETED (2026-02-15)
- Step 2: COMPLETED (2026-02-15)
- Step 3: COMPLETED (2026-02-15)
- Step 4: COMPLETED (2026-02-15)
- Step 5: COMPLETED (2026-02-15)
- Step 6: N/A_CONFIRMED (2026-02-15, no new migration required)
- Step 7: COMPLETED (2026-02-15)
- Step 8: COMPLETED (2026-02-15)

Step 1 note:
- Locked backfill entry scope to `CurrentAndNew` in ONB docs.
- Added explicit per-recipient ONB notify commit stage to the blueprint step table.
- Locked ownership split text across ONB/BCAST/REM docs for backfill reminder handoff.

Step 2 note:
- Kernel contract now fail-closes `ONB_REQUIREMENT_BACKFILL_START_DRAFT` unless `rollout_scope=CurrentAndNew`.

Step 3 note:
- Typed `Ph1OnbRepo` surface includes explicit backfill row methods for start/notify/complete parity.

Step 4 note:
- PH1.F backfill start path fail-closes on `NewHiresOnly` and enforces deterministic campaign snapshot + idempotent replay.

Step 5 note:
- Runtime flow for backfill start/notify/complete already aligned; no additional production runtime delta required in this pass.

Step 6 note:
- Existing migration `0014_position_requirements_schema_and_backfill_tables.sql` already contains campaign/target tables and idempotency indexes; no new SQL patch required.

Step 7 note:
- Added ONB backfill test coverage for:
  - `NewHiresOnly` reject (no campaign),
  - `CurrentAndNew` campaign creation,
  - notify/reminder loop idempotency + complete commit,
  - fail-closed tenant/gate checks.

Step 8 note:
- Proof commands completed successfully:
  - `cargo test -p selene_storage --test db_wiring_ph1onb_tables -- --nocapture`
  - `cargo test -p selene_os ph1onb -- --nocapture`
  - `scripts/selene_design_readiness_audit.sh`

## 6) Done Criteria

ONB backfill closure is done only when all are true:
1. Blueprint step table explicitly includes ONB notify/progress commit.
2. Backfill naming is coherent across blueprint + DB wiring + ECM + sim catalog + contracts.
3. Repo/storage/runtime surfaces for start/notify/complete are typed and coherent.
4. Backfill tests prove idempotency + deterministic completion behavior.
5. Final audit checkpoint runs clean from a pinned commit.
