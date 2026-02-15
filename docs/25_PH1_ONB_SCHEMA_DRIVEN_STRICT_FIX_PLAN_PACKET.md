# PH1.ONB Schema-Driven Strict Fix Plan Packet (v1)

Last updated: 2026-02-15
Owner: Selene core design + runtime
Status: STEP8_COMPLETED

## 1) Purpose

This packet is the strict execution plan to lock PH1.ONB as schema-driven only.

Rules:
- No Simulation -> No Execution.
- Engines never call engines directly; Selene OS orchestrates.
- PH1.POSITION owns requirements schema truth.
- PH1.ONB executes pinned schema requirements only.
- No hardcoded onboarding requirements in ONB runtime flow.

## 2) Scope

1. Remove/close any drift where ONB asks or enforces requirements not coming from pinned schema gates.
2. Keep ONB deterministic: one-question discipline, fail-closed gating, idempotent commits.
3. Keep LINK -> ONB handoff strictly to token + prefilled context refs (no schema truth ownership drift).
4. Prove parity across docs, contracts, runtime, storage, and tests.

## 3) Baseline Gate (must run before Step 2)

```bash
cd "$(git rev-parse --show-toplevel)"
git rev-parse HEAD
git status --short

rg -n "PH1.ONB|schema-required verification gates|PHOTO_EVIDENCE_CAPTURE|SENDER_CONFIRMATION|hardcoded|pinned schema|missing_required_fields" \
  docs/04_KERNEL_CONTRACTS.md \
  docs/05_OS_CONSTITUTION.md \
  docs/06_ENGINE_MAP.md \
  docs/07_ENGINE_REGISTRY.md \
  docs/10_DB_OWNERSHIP_MATRIX.md \
  docs/DB_WIRING/PH1_ONB.md \
  docs/ECM/PH1_ONB.md \
  docs/BLUEPRINTS/ONB_INVITED.md \
  crates/selene_kernel_contracts/src/ph1onb.rs \
  crates/selene_os/src/ph1onb.rs \
  crates/selene_storage/src/ph1f.rs \
  crates/selene_storage/tests/ph1_onb/db_wiring.rs
```

## 4) Patch Order

### Step 1: Docs Lock + Packet Bootstrap

Patch files (only):
1. `docs/02_BUILD_PLAN.md`
2. `docs/25_PH1_ONB_SCHEMA_DRIVEN_STRICT_FIX_PLAN_PACKET.md`

Patch intent:
- Freeze packet 24 closure in build plan.
- Set packet 25 as canonical next strict scope.

Post-step acceptance:

```bash
rg -n "0c6d9ec|25_PH1_ONB_SCHEMA_DRIVEN_STRICT_FIX_PLAN_PACKET|Next Strict Packet" docs/02_BUILD_PLAN.md
test -f docs/25_PH1_ONB_SCHEMA_DRIVEN_STRICT_FIX_PLAN_PACKET.md
```

---

### Step 2: Cross-Doc Ownership + Law Lock

Patch files (only if drift is found):
1. `docs/05_OS_CONSTITUTION.md`
2. `docs/06_ENGINE_MAP.md`
3. `docs/07_ENGINE_REGISTRY.md`
4. `docs/10_DB_OWNERSHIP_MATRIX.md`
5. `docs/04_KERNEL_CONTRACTS.md` (only if wording parity requires)

Patch intent:
- Keep ONB as executor-only and POSITION as schema truth owner across canonical docs.
- Make schema-driven requirement law explicit.

---

### Step 3: DB Wiring + ECM + Blueprint Lock

Patch files (only if drift is found):
1. `docs/DB_WIRING/PH1_ONB.md`
2. `docs/DB_WIRING/PH1_POSITION.md` (only if ownership wording drift found)
3. `docs/DB_WIRING/PH1_LINK.md` (only if handoff wording drift found)
4. `docs/ECM/PH1_ONB.md`
5. `docs/ECM/PH1_POSITION.md` (only if ownership wording drift found)
6. `docs/ECM/PH1_LINK.md` (only if handoff wording drift found)
7. `docs/BLUEPRINTS/ONB_INVITED.md`
8. `docs/08_SIMULATION_CATALOG.md` (only if simulation wording drift found)
9. `docs/09_BLUEPRINT_REGISTRY.md` (only if required)

Patch intent:
- Lock ONB blueprint/ECM/DB wiring to schema-driven requirement gates.
- Remove ambiguous wording that could imply hardcoded gates.

---

### Step 4: Kernel Contract + Runtime ONB Parity

Patch files (only if drift is found):
1. `crates/selene_kernel_contracts/src/ph1onb.rs`
2. `crates/selene_os/src/ph1onb.rs`
3. `crates/selene_os/src/simulation_executor.rs` (only if dispatch drift found)

Patch intent:
- Ensure ONB request/response contract and runtime flow enforce only schema-required gates.
- Ensure required verification gates are read from pinned schema context, not hardcoded by flow path.

---

### Step 5: Storage + Repository + SQL Parity

Patch files (only if drift is found):
1. `crates/selene_storage/src/repo.rs`
2. `crates/selene_storage/src/ph1f.rs`
3. `crates/selene_storage/migrations/*` (only if schema parity explicitly requires SQL updates)

Patch intent:
- Keep ONB persistence deterministic and idempotent for schema-required verification paths.
- Keep fail-closed behavior for missing required gates.

---

### Step 6: Test Closure

Patch files (only if drift is found):
1. `crates/selene_storage/tests/ph1_onb/db_wiring.rs`
2. `crates/selene_os/src/ph1onb.rs` (test module)
3. `crates/selene_os/src/ph1link.rs` (only if LINK->ONB handoff test extension is required)
4. `crates/selene_storage/tests/ph1_link/db_wiring.rs` (only if handoff storage test extension is required)

Required coverage:
- Schema requires photo/sender verify -> ONB fail-closed until required commits are complete.
- Schema does not require photo/sender verify -> ONB can complete without those commits.
- Missing required gates in session start path fail closed deterministically.
- LINK prefilled handoff stays read-only and deterministic into ONB.
- Commit idempotency keys replay deterministically for required verification commits.

---

### Step 7: Strict Drift Sweep + Acceptance Proof

No broad patching in this step unless drift is proven.

Run:

```bash
rg -n "PHOTO_EVIDENCE_CAPTURE|SENDER_CONFIRMATION|hardcoded|required_verification_gates|schema-required" \
  docs/DB_WIRING/PH1_ONB.md docs/ECM/PH1_ONB.md docs/BLUEPRINTS/ONB_INVITED.md \
  crates/selene_os/src/ph1onb.rs crates/selene_storage/src/ph1f.rs -S

cargo test -p selene_storage --test db_wiring_ph1onb_tables -- --nocapture
cargo test -p selene_os ph1onb -- --nocapture
```

Acceptance bar:
- Any hardcoded-only gate behavior is eliminated or explicitly schema-driven.
- ONB targeted tests pass with deterministic outcomes.

---

### Step 8: Final Proof + Freeze Checkpoint

No patching in this step.

Run:

```bash
scripts/selene_design_readiness_audit.sh
cargo test -p selene_storage --test db_wiring_ph1onb_tables -- --nocapture
cargo test -p selene_os ph1onb -- --nocapture
cargo test --workspace
git status --short
git rev-parse HEAD
git log -1 --oneline
```

Checkpoint expectations:
- PH1.ONB schema-driven requirement behavior is coherent across docs/contracts/runtime/storage/tests.
- Audit and test proofs pass from a clean pinned checkpoint.

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
- Updated `docs/02_BUILD_PLAN.md` to freeze packet 24 closure checkpoint (`0c6d9ec`) and point next strict scope to this packet.
- Created this packet as the canonical strict execution plan for PH1.ONB schema-driven closure.

Step 2 note:
- Cross-doc ownership + law lock review completed for:
  - `docs/05_OS_CONSTITUTION.md`
  - `docs/06_ENGINE_MAP.md`
  - `docs/07_ENGINE_REGISTRY.md`
  - `docs/10_DB_OWNERSHIP_MATRIX.md`
  - `docs/04_KERNEL_CONTRACTS.md`
- No drift patch was required in Step 2.
- Verified boundary wording is already aligned:
  - `PH1.POSITION` owns requirements schema truth and lifecycle writes.
  - `PH1.ONB` executes pinned schema requirements only and does not mutate schema definitions.
  - Requirement prompts/gates are documented as schema-driven, not hardcoded ONB-only logic.

Step 3 note:
- DB wiring + ECM + blueprint lock review completed for:
  - `docs/DB_WIRING/PH1_ONB.md`
  - `docs/DB_WIRING/PH1_POSITION.md`
  - `docs/DB_WIRING/PH1_LINK.md`
  - `docs/ECM/PH1_ONB.md`
  - `docs/ECM/PH1_POSITION.md`
  - `docs/ECM/PH1_LINK.md`
  - `docs/BLUEPRINTS/ONB_INVITED.md`
  - `docs/08_SIMULATION_CATALOG.md`
  - `docs/09_BLUEPRINT_REGISTRY.md`
- No drift patch was required in Step 3.
- Verified lock points:
  - PH1.ONB DB/ECM surfaces explicitly state executor-only behavior from pinned schema context.
  - PH1.POSITION DB/ECM surfaces keep schema truth ownership and lifecycle writes.
  - ONB blueprint and simulation text state schema-required verification gates and reject hardcoded ONB-only branches.

Step 4 note:
- Kernel contract + runtime ONB parity review completed for:
  - `crates/selene_kernel_contracts/src/ph1onb.rs`
  - `crates/selene_os/src/ph1onb.rs`
  - `crates/selene_os/src/simulation_executor.rs`
- No drift patch was required in Step 4.
- Verified parity:
  - ONB contract response includes pinned schema context + `required_verification_gates[]` fields.
  - ONB runtime executes request variants through storage-gated outcomes and does not contain hardcoded gate strings in runtime paths (photo/sender gate literals only appear in tests).
  - Simulation executor preserves orchestration boundary (`execute_onb`), with no direct engine-to-engine bypass.
- Proof runs:
  - `cargo test -p selene_os ph1onb -- --nocapture` -> pass (`10 passed; 0 failed`)
  - `cargo test -p selene_kernel_contracts ph1onb -- --nocapture` -> pass (`0 passed; 0 failed`; filtered target only)

Step 5 note:
- Storage + repository + SQL parity review completed for:
  - `crates/selene_storage/src/repo.rs`
  - `crates/selene_storage/src/ph1f.rs`
  - `crates/selene_storage/migrations/0014_position_requirements_schema_and_backfill_tables.sql`
- Drift patch applied in `crates/selene_storage/src/ph1f.rs`:
  - `ph1onb_verification_gate_required` now treats pinned session gates as authoritative for replay stability.
  - If pinned schema context exists and a gate is not in `required_verification_gates`, storage now fails closed (`false`) instead of re-deriving from current schema state.
  - Legacy fallback re-derivation remains only for pre-pin sessions without pinned schema context.
- No SQL migration patch was required in Step 5.
- Proof runs:
  - `cargo test -p selene_storage --test db_wiring_ph1onb_tables -- --nocapture` -> pass (`11 passed; 0 failed`)
  - `cargo test -p selene_os ph1onb -- --nocapture` -> pass (`10 passed; 0 failed`)

Step 6 note:
- Test closure completed for required coverage in:
  - `crates/selene_storage/tests/ph1_onb/db_wiring.rs`
  - `crates/selene_os/src/ph1onb.rs` (test module)
  - `crates/selene_storage/tests/ph1_link/db_wiring.rs`
- Drift patch applied in `crates/selene_storage/tests/ph1_onb/db_wiring.rs`:
  - Added `at_onb_db_12_required_verification_commit_idempotency_replays_deterministically`.
  - Proves required `photo` and `sender verify` commits replay deterministically on idempotency-key reuse and do not mutate persisted state on replay.
- Existing tests already covered:
  - Schema-required verification fail-closed path.
  - Non-required verification path completes without photo/sender commits.
  - Missing required verification gates fail closed.
  - LINK prefilled handoff remains deterministic into ONB session pinning.
- Proof runs:
  - `cargo test -p selene_storage --test db_wiring_ph1onb_tables -- --nocapture` -> pass (`12 passed; 0 failed`)
  - `cargo test -p selene_os ph1onb -- --nocapture` -> pass (`10 passed; 0 failed`)
  - `cargo test -p selene_storage --test db_wiring_ph1link_tables -- --nocapture` -> pass (`13 passed; 0 failed`)

Step 7 note:
- Strict drift sweep + acceptance proof completed with no Step 7 code patch required.
- Sweep command run:
  - `rg -n "PHOTO_EVIDENCE_CAPTURE|SENDER_CONFIRMATION|hardcoded|required_verification_gates|schema-required" docs/DB_WIRING/PH1_ONB.md docs/ECM/PH1_ONB.md docs/BLUEPRINTS/ONB_INVITED.md crates/selene_os/src/ph1onb.rs crates/selene_storage/src/ph1f.rs -S`
- Result interpretation:
  - Docs/ECM/Blueprint hits remain schema-driven lock text.
  - Runtime hit in `crates/selene_os/src/ph1onb.rs` is test-only assertion coverage, not production hardcoded gating behavior.
  - Storage hits in `crates/selene_storage/src/ph1f.rs` enforce `required_verification_gates` checks and schema-derived gate computation.
- Acceptance proofs:
  - `cargo test -p selene_storage --test db_wiring_ph1onb_tables -- --nocapture` -> pass (`12 passed; 0 failed`)
  - `cargo test -p selene_os ph1onb -- --nocapture` -> pass (`10 passed; 0 failed`)

Step 8 note:
- Final proof + freeze checkpoint run completed:
  - `scripts/selene_design_readiness_audit.sh` -> exit `0` (expected LINK `LEGACY_DO_NOT_WIRE` evidence only)
  - `cargo test -p selene_storage --test db_wiring_ph1onb_tables -- --nocapture` -> pass (`12 passed; 0 failed`)
  - `cargo test -p selene_os ph1onb -- --nocapture` -> pass (`10 passed; 0 failed`)
  - `cargo test --workspace` -> pass (workspace-wide test suite green)
- Freeze metadata captured:
  - pre-freeze checkpoint base: `0c6d9ec81f079c5f64d3baf7301dcbf4930ee655`
  - pre-freeze tree state: dirty due to packet execution changes staged for freeze commit.

## 6) Done Criteria

This packet is done only when all are true:
1. ONB requirements/gates are executed from pinned schema context only.
2. No hardcoded onboarding requirement path bypasses schema truth ownership boundaries.
3. ONB verification and completion gates are deterministic, idempotent, and fail-closed.
4. Final readiness audit + tests pass at a clean pinned checkpoint.
