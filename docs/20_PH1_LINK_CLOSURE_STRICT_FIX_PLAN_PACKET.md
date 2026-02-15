# PH1.LINK Closure Strict Fix Plan Packet (v2)

Last updated: 2026-02-15
Owner: Selene core design + runtime
Status: EXECUTED_COMMITTED

## 1) Purpose

This packet is the single step-by-step plan to run a fresh PH1.LINK closure pass after ONB backfill closure.

Rules:
- Do not skip steps.
- Do not patch files outside each step scope.
- Keep `No Simulation -> No Execution`.
- Keep engine boundaries (Selene OS orchestrates; engines do not call engines directly).

## 2) Scope (what this packet validates/fixes)

1. Confirm PH1.LINK docs/contracts/runtime/storage are still coherent after recent ONB changes.
2. Re-validate draft update, revoke guard, open/activate idempotency, forward-block single-path behavior.
3. Re-validate schema-driven missing-required-field computation for link draft updates.
4. Refresh test closure and audit checkpoint from current pinned baseline.

## 3) Baseline Gate (must run before Step 1)

```bash
cd "$(git rev-parse --show-toplevel)"
git rev-parse HEAD
git status --short

rg -n "LINK_INVITE_DRAFT_UPDATE_COMMIT|LINK_INVITE_REVOKE_REVOKE|LINK_INVITE_OPEN_ACTIVATE_COMMIT|LINK_INVITE_FORWARD_BLOCK_COMMIT|LINK_DELIVER_INVITE" \
  docs/DB_WIRING/PH1_LINK.md \
  docs/ECM/PH1_LINK.md \
  docs/BLUEPRINTS/LINK_INVITE.md \
  docs/BLUEPRINTS/LINK_DELIVER_INVITE.md \
  docs/BLUEPRINTS/LINK_OPEN_ACTIVATE.md \
  docs/08_SIMULATION_CATALOG.md

rg -n "InviteDraftUpdateCommit|InviteRevokeRevoke|InviteOpenActivateCommit|ph1link_compute_missing_required_fields" \
  crates/selene_kernel_contracts/src/ph1link.rs \
  crates/selene_os/src/ph1link.rs \
  crates/selene_storage/src/repo.rs \
  crates/selene_storage/src/ph1f.rs
```

## 4) Patch Order

### Step 1: Docs Lock Recheck

Patch files (only if drift is found):
1. `docs/DB_WIRING/PH1_LINK.md`
2. `docs/ECM/PH1_LINK.md`
3. `docs/BLUEPRINTS/LINK_INVITE.md`
4. `docs/BLUEPRINTS/LINK_DELIVER_INVITE.md`
5. `docs/BLUEPRINTS/LINK_OPEN_ACTIVATE.md`
6. `docs/08_SIMULATION_CATALOG.md` (PH1.LINK rows only)

Patch intent:
- Ensure one coherent behavior for draft update, revoke guard, open/activate idempotency, forward-block, and delivery ownership.

Post-step acceptance:

```bash
rg -n "LINK_INVITE_DRAFT_UPDATE_COMMIT|LINK_INVITE_REVOKE_REVOKE|LINK_INVITE_OPEN_ACTIVATE_COMMIT|LINK_INVITE_FORWARD_BLOCK_COMMIT|LINK_DELIVER_INVITE" \
  docs/DB_WIRING/PH1_LINK.md docs/ECM/PH1_LINK.md docs/BLUEPRINTS/LINK_INVITE.md docs/BLUEPRINTS/LINK_DELIVER_INVITE.md docs/BLUEPRINTS/LINK_OPEN_ACTIVATE.md docs/08_SIMULATION_CATALOG.md
```

---

### Step 2: Kernel Contract Parity

Patch files (only if drift is found):
1. `crates/selene_kernel_contracts/src/ph1link.rs`

Patch intent:
- Keep PH1.LINK simulation constants and request/response validation aligned with Step 1 docs lock.

Post-step acceptance:

```bash
cargo test -p selene_kernel_contracts ph1link -- --nocapture
```

---

### Step 3: Repo Interface Parity

Patch files (only if drift is found):
1. `crates/selene_storage/src/repo.rs`

Patch intent:
- Ensure `Ph1LinkRepo` method signatures and row methods match kernel contracts exactly.

Post-step acceptance:

```bash
rg -n "trait Ph1LinkRepo|ph1link_invite_" crates/selene_storage/src/repo.rs
```

---

### Step 4: Storage Behavior Parity

Patch files (only if drift is found):
1. `crates/selene_storage/src/ph1f.rs`

Patch intent:
- Confirm deterministic behavior + idempotency for draft update/open-activate/revoke/forward-block.
- Confirm missing-required fields remain schema-driven.

Post-step acceptance:

```bash
cargo test -p selene_storage --test db_wiring_ph1link_tables -- --nocapture
```

---

### Step 5: Runtime Wiring Parity

Patch files (only if drift is found):
1. `crates/selene_os/src/ph1link.rs`
2. `crates/selene_os/src/simulation_executor.rs` (only if required)

Patch intent:
- Ensure runtime dispatch and reason-coded transitions remain aligned with docs/contracts.

Post-step acceptance:

```bash
cargo test -p selene_os ph1link -- --nocapture
```

---

### Step 6: SQL Parity Check

Patch files (conditional):
1. `crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql`

Patch intent:
- Only patch SQL if enum/index/idempotency contract drift is found.

Post-step acceptance:

```bash
rg -n "onboarding_drafts|onboarding_link_tokens|status IN|idempotency" crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql
```

---

### Step 7: Test Closure Refresh

Patch files (only if drift is found):
1. `crates/selene_storage/tests/ph1_link/db_wiring.rs`
2. `crates/selene_os/src/ph1link.rs` (test module)

Required coverage:
- draft update success + idempotent replay
- draft update invalid-state refusal
- revoke guard behavior
- open/activate idempotency replay
- forward-block single-path behavior
- schema-driven missing-required-fields recompute

Post-step acceptance:

```bash
cargo test -p selene_storage --test db_wiring_ph1link_tables -- --nocapture
cargo test -p selene_os ph1link -- --nocapture
```

---

### Step 8: Final Proof + Audit Checkpoint

No patching in this step.

Run:

```bash
scripts/selene_design_readiness_audit.sh
cargo test -p selene_storage --test db_wiring_ph1link_tables -- --nocapture
cargo test -p selene_os ph1link -- --nocapture
git status --short
git rev-parse HEAD
git log -1 --oneline
```

Checkpoint expectations:
- PH1.LINK drift checks are clean.
- Tests pass for PH1.LINK slices.
- Pinned commit hash + status are captured.

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
- Baseline/docs lock recheck found no PH1.LINK doc drift requiring patch in this pass.

Step 2 note:
- Kernel contract parity checks passed for PH1.LINK; no delta required.

Step 3 note:
- `Ph1LinkRepo` typed surface already includes draft-update/open-activate(replay)/revoke/forward-block parity.

Step 4 note:
- PH1.LINK storage DB wiring tests passed; deterministic/idempotent behavior validated.

Step 5 note:
- PH1.LINK runtime tests passed; dispatch and refusal behavior validated.

Step 6 note:
- SQL parity check for `0012_ph1link_onboarding_draft_tables.sql` showed no patch needed.

Step 7 note:
- Existing PH1.LINK storage/runtime tests already cover required critical paths in this packet.

Step 8 note:
- Final proof commands passed for PH1.LINK slices:
  - `scripts/selene_design_readiness_audit.sh`
  - `cargo test -p selene_storage --test db_wiring_ph1link_tables -- --nocapture`
  - `cargo test -p selene_os ph1link -- --nocapture`
- Audit run was pinned and valid; dirty tree during proof was only this packet file plus ledger freeze entry.
- Revalidated at clean tree checkpoint on commit `a7295208939e6dd09ae8f1201ff15dec920e4c3c`:
  - readiness audit `PASS` (`AUDIT_TREE_STATE: CLEAN`)
  - PH1.LINK storage/runtime slices `PASS`

## 6) Done Criteria

PH1.LINK closure pass is done only when all are true:
1. Docs/contracts/runtime/storage/SQL agree.
2. Draft update/revoke/open-activate/forward-block semantics are deterministic and tested.
3. Missing-required-fields remain schema-driven.
4. PH1.LINK tests and readiness audit are clean at the pinned commit.
