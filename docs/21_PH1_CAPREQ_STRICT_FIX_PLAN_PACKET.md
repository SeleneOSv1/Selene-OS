# PH1.CAPREQ Strict Fix Plan Packet

Last updated: 2026-02-15
Owner: Selene core design + runtime
Status: STEP1_COMPLETED_PENDING_STEP2

## 1) Purpose

This packet is the single step-by-step plan to close PH1.CAPREQ naming and lifecycle parity drift.

Rules:
- Do not skip steps.
- Do not patch files outside each step scope.
- Keep `No Simulation -> No Execution`.
- Keep engine boundaries (Selene OS orchestrates; engines do not call engines directly).

## 2) Scope (what this packet fixes)

1. CAPREQ lifecycle simulation naming must be exact across DB wiring, ECM, blueprint, and simulation catalog.
2. DB wiring lifecycle write section must include all active CAPREQ transitions:
   - `CREATE_DRAFT`
   - `SUBMIT_FOR_APPROVAL_COMMIT`
   - `APPROVE_COMMIT`
   - `REJECT_COMMIT`
   - `FULFILL_COMMIT`
   - `CANCEL_REVOKE`
3. Step-level execution proofs must be recorded from current pinned commit.

## 3) Baseline Gate (must run before Step 1)

```bash
cd "$(git rev-parse --show-toplevel)"
git rev-parse HEAD
git status --short

rg -n "CAPREQ_CREATE_DRAFT|CAPREQ_SUBMIT_FOR_APPROVAL_COMMIT|CAPREQ_APPROVE_COMMIT|CAPREQ_REJECT_COMMIT|CAPREQ_FULFILL_COMMIT|CAPREQ_CANCEL_REVOKE|CAPREQ_SUBMIT_COMMIT" \
  docs/DB_WIRING/PH1_CAPREQ.md \
  docs/ECM/PH1_CAPREQ.md \
  docs/BLUEPRINTS/CAPREQ_MANAGE.md \
  docs/08_SIMULATION_CATALOG.md \
  crates/selene_kernel_contracts/src/ph1capreq.rs
```

## 4) Patch Order

### Step 1: Docs Lock (single source naming/lifecycle parity)

Patch files (only):
1. `docs/DB_WIRING/PH1_CAPREQ.md`
2. `docs/ECM/PH1_CAPREQ.md` (only if drift found)
3. `docs/BLUEPRINTS/CAPREQ_MANAGE.md` (only if drift found)
4. `docs/08_SIMULATION_CATALOG.md` (only if drift found)

Patch intent:
- Normalize CAPREQ simulation names to kernel contract constants.
- Ensure DB wiring write section covers full CAPREQ lifecycle.

Post-step acceptance:

```bash
rg -n "CAPREQ_CREATE_DRAFT|CAPREQ_SUBMIT_FOR_APPROVAL_COMMIT|CAPREQ_APPROVE_COMMIT|CAPREQ_REJECT_COMMIT|CAPREQ_FULFILL_COMMIT|CAPREQ_CANCEL_REVOKE" \
  docs/DB_WIRING/PH1_CAPREQ.md \
  docs/ECM/PH1_CAPREQ.md \
  docs/BLUEPRINTS/CAPREQ_MANAGE.md \
  docs/08_SIMULATION_CATALOG.md \
  crates/selene_kernel_contracts/src/ph1capreq.rs

rg -n "CAPREQ_SUBMIT_COMMIT" docs/DB_WIRING/PH1_CAPREQ.md docs/ECM/PH1_CAPREQ.md docs/BLUEPRINTS/CAPREQ_MANAGE.md docs/08_SIMULATION_CATALOG.md -S
```

Step 1 exit criteria:
- No legacy CAPREQ simulation token remains in canonical docs.
- DB wiring lifecycle section fully matches active simulation catalog + kernel constants.

---

### Step 2: Kernel/Runtime Recheck

Patch files (only if drift found):
1. `crates/selene_kernel_contracts/src/ph1capreq.rs`
2. `crates/selene_os/src/ph1capreq.rs`
3. `crates/selene_os/src/simulation_executor.rs`

---

### Step 3: Storage/Repo Recheck

Patch files (only if drift found):
1. `crates/selene_storage/src/repo.rs`
2. `crates/selene_storage/src/ph1f.rs`

---

### Step 4: Test Closure + Final Checkpoint

Run:

```bash
cargo test -p selene_storage --test db_wiring_ph1capreq_tables -- --nocapture
cargo test -p selene_os capreq -- --nocapture
scripts/selene_design_readiness_audit.sh
```

## 5) Execution Record

- Step 1: COMPLETED (2026-02-15)
- Step 2: NOT_STARTED
- Step 3: NOT_STARTED
- Step 4: NOT_STARTED

Step 1 note:
- CAPREQ docs-lock pass started and completed.
- `docs/DB_WIRING/PH1_CAPREQ.md` naming drift closed (`CAPREQ_SUBMIT_COMMIT` -> `CAPREQ_SUBMIT_FOR_APPROVAL_COMMIT`) and lifecycle write coverage expanded to include `CAPREQ_FULFILL_COMMIT` + `CAPREQ_CANCEL_REVOKE`.

## 6) Done Criteria

PH1.CAPREQ is done for this packet only when all are true:
1. CAPREQ lifecycle names are exact and coherent across docs + contracts.
2. CAPREQ lifecycle write coverage is complete in DB wiring.
3. Runtime/storage/tests recheck passes with clean audit checkpoint.
