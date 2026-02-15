# PH1.LINK Strict Fix Plan Packet

Last updated: 2026-02-15
Owner: Selene core design + runtime
Status: READY_FOR_EXECUTION

## 1) Purpose

This packet is the single step-by-step plan to close PH1.LINK drift.

It defines:
- Exact patch order by file.
- What to check before each step.
- What must pass before moving to the next step.

Rules:
- Do not skip steps.
- Do not patch files outside each step scope.
- Keep `No Simulation -> No Execution` intact.
- Keep engine boundaries intact (Selene OS orchestrates; engines do not call engines directly).

## 2) Scope (what this packet fixes)

1. Draft update is documented but not implemented (`LINK_INVITE_DRAFT_UPDATE_COMMIT`).
2. Revoke behavior does not enforce the documented AP override rule for already activated links.
3. Open/activate idempotency semantics drift between docs and code.
4. Forward-block behavior is split across two paths and can drift.
5. Missing required fields are still hardcoded by invitee type instead of schema-driven.
6. PH1.LINK tests do not cover the above critical paths.

## 3) Baseline Gate (must run before Step 1)

Run:

```bash
cd "$(git rev-parse --show-toplevel)"
git rev-parse HEAD
git status --short

rg -n "PH1LINK_INVITE_DRAFT_UPDATE_COMMIT_ROW|LINK_INVITE_DRAFT_UPDATE_COMMIT" \
  docs/BLUEPRINTS/LINK_INVITE.md docs/ECM/PH1_LINK.md docs/08_SIMULATION_CATALOG.md

rg -n "InviteDraftUpdate|draft_update|LINK_DRAFT_UPDATE_RETRYABLE" \
  crates/selene_kernel_contracts/src/ph1link.rs \
  crates/selene_os/src/ph1link.rs \
  crates/selene_storage/src/repo.rs \
  crates/selene_storage/src/ph1f.rs

rg -n "LINK_INVITE_REVOKE_REVOKE|already activated|AP override" docs/08_SIMULATION_CATALOG.md
rg -n "pub fn ph1link_invite_revoke_revoke" crates/selene_storage/src/ph1f.rs

rg -n "input_schema.*idempotency_key|OPEN_ACTIVATE" docs/ECM/PH1_LINK.md docs/BLUEPRINTS/LINK_OPEN_ACTIVATE.md
rg -n "InviteOpenActivateCommitRequest|ph1link_invite_open_activate_commit\(" \
  crates/selene_kernel_contracts/src/ph1link.rs \
  crates/selene_storage/src/ph1f.rs

rg -n "ph1link_compute_missing_required_fields" crates/selene_storage/src/ph1f.rs
```

Expected baseline evidence:
- Docs contain draft-update simulation/capability.
- Code does not yet contain draft-update request/runtime/storage path.
- Revoke simulation text mentions AP override for already activated links.
- Storage revoke path currently does not enforce AP override.
- Docs mention open/activate idempotency key.
- Code open/activate path does not carry idempotency key.
- Missing required fields function exists and is hardcoded.

## 4) Patch Order

### Step 1: Docs Lock (single truth before code)

Patch files (only):
1. `docs/DB_WIRING/PH1_LINK.md`
2. `docs/ECM/PH1_LINK.md`
3. `docs/BLUEPRINTS/LINK_INVITE.md`
4. `docs/BLUEPRINTS/LINK_OPEN_ACTIVATE.md`
5. `docs/08_SIMULATION_CATALOG.md` (only PH1.LINK records)

Patch intent:
- Freeze one clear behavior for:
  - draft update lifecycle,
  - revoke AP override rule,
  - open/activate idempotency,
  - forward-block execution path,
  - schema-driven missing-required-field computation.
- Remove any wording conflict between these docs.

Post-step acceptance:

```bash
rg -n "LINK_INVITE_DRAFT_UPDATE_COMMIT|creator_update_fields|draft_id" \
  docs/DB_WIRING/PH1_LINK.md docs/ECM/PH1_LINK.md docs/BLUEPRINTS/LINK_INVITE.md docs/08_SIMULATION_CATALOG.md

rg -n "already activated|AP override|refuse" docs/08_SIMULATION_CATALOG.md docs/DB_WIRING/PH1_LINK.md docs/ECM/PH1_LINK.md

rg -n "OPEN_ACTIVATE.*idempotency_key|idempotency_key.*OPEN_ACTIVATE" \
  docs/ECM/PH1_LINK.md docs/BLUEPRINTS/LINK_OPEN_ACTIVATE.md docs/08_SIMULATION_CATALOG.md

rg -n "schema-driven|tenant schema|missing_required_fields" \
  docs/DB_WIRING/PH1_LINK.md docs/08_SIMULATION_CATALOG.md
```

Step 1 exit criteria:
- All PH1.LINK design docs describe the same behavior with no contradictions.

---

### Step 2: Kernel Contract Delta

Patch files (only):
1. `crates/selene_kernel_contracts/src/ph1link.rs`

Patch intent:
- Add missing `LINK_INVITE_DRAFT_UPDATE_COMMIT` simulation constant and request/response contract.
- Add open/activate idempotency key field if docs lock says it is required.
- Keep validation bounded and fail-closed.

Post-step acceptance:

```bash
rg -n "LINK_INVITE_DRAFT_UPDATE_COMMIT|InviteDraftUpdateCommit|creator_update_fields" crates/selene_kernel_contracts/src/ph1link.rs
rg -n "InviteOpenActivateCommitRequest|idempotency_key" crates/selene_kernel_contracts/src/ph1link.rs
cargo test -p selene_kernel_contracts ph1link -- --nocapture
```

Step 2 exit criteria:
- Contracts compile and enforce the locked doc semantics.

---

### Step 3: Typed Repo Surface Parity

Patch files (only):
1. `crates/selene_storage/src/repo.rs`

Patch intent:
- Add missing PH1.LINK draft-update repo method.
- Align method signatures with updated kernel contracts (including idempotency where required).

Post-step acceptance:

```bash
rg -n "trait Ph1LinkRepo|draft_update|open_activate|idempotency" crates/selene_storage/src/repo.rs
```

Step 3 exit criteria:
- Typed PH1.LINK repo interface covers all documented capabilities.

---

### Step 4: Storage Behavior Fixes

Patch files (only):
1. `crates/selene_storage/src/ph1f.rs`

Patch intent:
- Implement draft-update commit path with deterministic dedupe and recompute of missing required fields.
- Enforce revoke guard:
  - if token is already activated, require AP override evidence or refuse.
- Align open/activate idempotency behavior with docs lock.
- Make forward-block behavior single-path and deterministic.
- Replace hardcoded missing-required-field computation with schema-driven resolver.

Post-step acceptance:

```bash
rg -n "ph1link_invite_draft_update_commit|creator_update_fields|idempotency" crates/selene_storage/src/ph1f.rs
rg -n "ph1link_invite_revoke_revoke|AP override|activated" crates/selene_storage/src/ph1f.rs
rg -n "ph1link_invite_open_activate_commit|idempotency_key" crates/selene_storage/src/ph1f.rs
rg -n "ph1link_compute_missing_required_fields|schema" crates/selene_storage/src/ph1f.rs
cargo test -p selene_storage --test ph1_link_db_wiring -- --nocapture
```

Step 4 exit criteria:
- Storage behavior is deterministic and matches docs + contracts.

---

### Step 5: Runtime Wiring (PH1.LINK)

Patch files (only):
1. `crates/selene_os/src/ph1link.rs`
2. `crates/selene_os/src/simulation_executor.rs` (only if mapping changes are required)

Patch intent:
- Wire new draft-update request path in runtime.
- Enforce revoke refusal behavior when AP override is missing.
- Keep simulation IDs and reason codes consistent.

Post-step acceptance:

```bash
rg -n "InviteDraftUpdateCommit|LINK_INVITE_DRAFT_UPDATE_COMMIT" crates/selene_os/src/ph1link.rs crates/selene_os/src/simulation_executor.rs
rg -n "InviteRevokeRevoke|override|refuse" crates/selene_os/src/ph1link.rs
cargo test -p selene_os ph1link -- --nocapture
```

Step 5 exit criteria:
- Runtime paths execute only allowed transitions and emit correct simulation reasoning.

---

### Step 6: SQL/Migration Contract Parity

Patch files (only):
1. `crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql` (only if parity changes are needed)

Patch intent:
- Ensure SQL constraints and indexes match final contract decisions.
- Keep enum/status bounds and dedupe shape aligned with storage behavior.

Post-step acceptance:

```bash
rg -n "onboarding_drafts|onboarding_link_tokens|status IN|idempotency" crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql
```

Step 6 exit criteria:
- SQL contract is aligned with docs/contracts/storage.

---

### Step 7: Test Closure (must prove drift is closed)

Patch files (only):
1. `crates/selene_storage/tests/ph1_link/db_wiring.rs`
2. `crates/selene_os/src/ph1link.rs` (test module if present)

Required new tests:
- Draft update success + idempotent replay.
- Draft update refused for invalid state.
- Revoke refused for activated without AP override.
- Revoke allowed for activated with AP override.
- Open/activate idempotency replay behavior.
- Forward-block deterministic single-path behavior.
- Missing-required-fields recompute is schema-driven (not hardcoded type map).

Post-step acceptance:

```bash
cargo test -p selene_storage --test ph1_link_db_wiring -- --nocapture
cargo test -p selene_os ph1link -- --nocapture
```

Step 7 exit criteria:
- All critical PH1.LINK behavior has direct tests.

---

### Step 8: Final Drift Proof + Audit Checkpoint

No patching in this step.

Run:

```bash
scripts/selene_design_readiness_audit.sh

rg -n "PH1\.LINK|LINK_INVITE_DRAFT_UPDATE_COMMIT|OPEN_ACTIVATE|REVOKE" \
  docs/COVERAGE_MATRIX.md docs/DB_WIRING/PH1_LINK.md docs/ECM/PH1_LINK.md docs/BLUEPRINTS/LINK_INVITE.md docs/BLUEPRINTS/LINK_OPEN_ACTIVATE.md docs/08_SIMULATION_CATALOG.md

git status --short
git rev-parse HEAD
git log -1 --oneline
```

Checkpoint expectations:
- Audit shows no PH1.LINK drift findings.
- Only expected changed files are present.
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

Step 7 note:
- Runtime/storage now directly test draft update idempotency, invalid-state refusal, activated revoke refusal path, open/activate idempotency, forward-block single-path behavior, and schema-driven missing-fields recompute.
- The historical case “revoke allowed for activated with AP override” remains contract-blocked because `InviteRevokeRevokeRequest` does not carry `ap_override_ref`.

Step 8 note:
- Audit command completed with `EXIT:0`; PH1.LINK drift checks are clean and section 6 output is expected `LEGACY_DO_NOT_WIRE` compliance evidence.
- Pinned hash + dirty-file listing were captured as required before final commit closure.

## 6) Done Criteria

PH1.LINK is done for this packet only when all are true:
1. Docs, contracts, runtime, storage, and SQL all describe the same rules.
2. Draft update exists and is wired end-to-end.
3. Revoke AP override rule is enforced for activated links.
4. Open/activate idempotency is explicit and tested.
5. Missing-required-fields are schema-driven.
6. Tests cover all critical paths.
7. Audit checkpoint is clean for PH1.LINK drift.
