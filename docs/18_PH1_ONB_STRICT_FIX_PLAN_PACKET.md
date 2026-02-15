# PH1.ONB Strict Fix Plan Packet

Last updated: 2026-02-15
Owner: Selene core design + runtime
Status: READY_FOR_EXECUTION

## 1) Purpose

This packet is the single step-by-step plan to close PH1.ONB schema-execution drift.

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

1. ONB docs state pinned schema context, but ONB storage/session contract does not persist or return pinned schema context fields.
2. Sender-verification requirement check still uses hardcoded field aliases in storage (`photo_blob_ref|photo_proof_ref|employee_photo`, etc.) instead of schema-derived gating.
3. ONB completion/access checks depend on alias-based gate detection and can drift from active requirements schemas.
4. ONB session-start result does not expose enough deterministic schema context for replay/debug proof.
5. ONB test surface does not directly prove alias-free schema-gated verification behavior.
6. ONB docs/contracts/runtime naming around photo/sender verification is still legacy-heavy and needs explicit "legacy capability id, schema-driven semantics" lock.

## 3) Baseline Gate (must run before Step 1)

Run:

```bash
cd "$(git rev-parse --show-toplevel)"
git rev-parse HEAD
git status --short

rg -n "pinned schema|schema_id|schema_version|effective_overlay_set_id|selector snapshot|sender verification" \
  docs/DB_WIRING/PH1_ONB.md docs/ECM/PH1_ONB.md docs/BLUEPRINTS/ONB_INVITED.md

rg -n "ph1onb_sender_verification_required|PHOTO_FIELD_KEYS|VERIFY_FIELD_KEYS" \
  crates/selene_storage/src/ph1f.rs

rg -n "OnboardingSessionRecord|OnbSessionStartResult|EmployeePhoto|EmployeeSenderVerify" \
  crates/selene_storage/src/ph1f.rs crates/selene_kernel_contracts/src/ph1onb.rs crates/selene_os/src/ph1onb.rs

rg -n "at_onb_db_|onb_employee_can_complete_without_sender_verify_when_not_schema_required|schema-required" \
  crates/selene_storage/tests/ph1_onb/db_wiring.rs crates/selene_os/src/ph1onb.rs docs/DB_WIRING/PH1_ONB.md
```

Expected baseline evidence:
- Docs describe pinned schema context and schema-required verification behavior.
- Storage still contains alias arrays in `ph1onb_sender_verification_required`.
- `OnboardingSessionRecord` lacks explicit pinned schema context fields.
- Existing ONB tests do not directly prove alias-free schema-derived verification gating.

## 4) Patch Order

### Step 1: Docs Lock (single truth before code)

Patch files (only):
1. `docs/DB_WIRING/PH1_ONB.md`
2. `docs/ECM/PH1_ONB.md`
3. `docs/BLUEPRINTS/ONB_INVITED.md`
4. `docs/04_KERNEL_CONTRACTS.md` (only if wording parity is required)

Patch intent:
- Freeze one explicit ONB schema-execution model:
  - session start pins schema context and persists it,
  - verification gates are schema-derived (not field-alias derived),
  - legacy capability IDs remain for compatibility, but semantics are generic schema-driven gates.
- Define deterministic fields ONB must carry for replay proof:
  - `pinned_schema_id`
  - `pinned_schema_version`
  - `pinned_overlay_set_id`
  - `pinned_selector_snapshot_ref` (or equivalent bounded snapshot field)
  - `required_verification_gates[]`

Post-step acceptance:

```bash
rg -n "pinned_schema_id|pinned_schema_version|pinned_overlay_set_id|required_verification_gates|schema-derived" \
  docs/DB_WIRING/PH1_ONB.md docs/ECM/PH1_ONB.md docs/BLUEPRINTS/ONB_INVITED.md

rg -n "legacy capability id retained|schema-required verification gate" \
  docs/DB_WIRING/PH1_ONB.md docs/ECM/PH1_ONB.md docs/BLUEPRINTS/ONB_INVITED.md
```

Step 1 exit criteria:
- DB wiring, ECM, and blueprint describe the same ONB schema-gating semantics.

---

### Step 2: Kernel Contract Delta (PH1.ONB)

Patch files (only):
1. `crates/selene_kernel_contracts/src/ph1onb.rs`

Patch intent:
- Extend ONB session contracts to carry pinned schema context fields required by docs lock.
- Add/lock a deterministic gate representation (`required_verification_gates` or equivalent bounded enum/list).
- Keep legacy simulation IDs but document/validate schema-driven semantics in request/response validation.

Post-step acceptance:

```bash
rg -n "OnbSessionStartResult|pinned_schema|required_verification_gates|legacy simulation ids retained" crates/selene_kernel_contracts/src/ph1onb.rs
cargo test -p selene_kernel_contracts ph1onb -- --nocapture
```

Step 2 exit criteria:
- ONB contracts compile and include explicit schema pin + gate semantics.

---

### Step 3: Typed Repo Surface Parity

Patch files (only):
1. `crates/selene_storage/src/repo.rs`

Patch intent:
- Keep `Ph1OnbRepo` signatures aligned with updated kernel contracts for session-start output and schema-derived gate checks.
- Ensure no repo-layer ambiguity around legacy photo/sender names versus schema-gate semantics.

Post-step acceptance:

```bash
rg -n "trait Ph1OnbRepo|session_start|required_verification_gates|pinned_schema" crates/selene_storage/src/repo.rs
```

Step 3 exit criteria:
- Typed ONB repo methods match contract semantics.

---

### Step 4: Storage Behavior Fixes (core ONB drift closure)

Patch files (only):
1. `crates/selene_storage/src/ph1f.rs`

Patch intent:
- Persist pinned schema context fields on ONB session start.
- Replace alias-based sender-verification requirement detection with schema-derived gate resolution.
- Keep fail-closed behavior:
  - if required verification gate is unresolved, block `ONB_ACCESS_INSTANCE_CREATE_COMMIT` and `ONB_COMPLETE_COMMIT`.
- Preserve deterministic idempotency behavior for all ONB commits.

Post-step acceptance:

```bash
rg -n "OnboardingSessionRecord|pinned_schema|required_verification_gates" crates/selene_storage/src/ph1f.rs
rg -n "ph1onb_sender_verification_required|PHOTO_FIELD_KEYS|VERIFY_FIELD_KEYS" crates/selene_storage/src/ph1f.rs
cargo test -p selene_storage --test db_wiring_ph1onb_tables -- --nocapture
```

Expected:
- No hardcoded ONB alias arrays remain in sender-verification gate resolution.
- Session-start records carry deterministic schema context.

Step 4 exit criteria:
- Storage behavior is schema-driven and deterministic.

---

### Step 5: Runtime Wiring (PH1.ONB)

Patch files (only):
1. `crates/selene_os/src/ph1onb.rs`
2. `crates/selene_os/src/simulation_executor.rs` (only if mapping changes are required)

Patch intent:
- Return and audit pinned schema context consistently from session start.
- Keep legacy request names (`EmployeePhotoCaptureSend`, `EmployeeSenderVerify`) but enforce schema-derived gating semantics.
- Ensure refuse paths are reason-coded when required schema gates are unmet.

Post-step acceptance:

```bash
rg -n "SessionStart|pinned_schema|required_verification_gates|EmployeePhoto|EmployeeSenderVerify|Refuse" \
  crates/selene_os/src/ph1onb.rs
cargo test -p selene_os ph1onb -- --nocapture
```

Step 5 exit criteria:
- Runtime output and refusal semantics match docs/contracts.

---

### Step 6: SQL/Migration Parity (if ONB table lock exists in this branch)

Patch files (only, conditional):
1. `crates/selene_storage/migrations/*onb*` (only if onboarding session SQL tables are present)

Patch intent:
- If ONB SQL tables exist, align columns/constraints with pinned schema context and gate semantics.
- If ONB SQL tables do not exist in this branch, record explicit `N/A` evidence for this step.

Post-step acceptance:

```bash
rg -n "onboarding_sessions|pinned_schema|required_verification_gates|verification_status" crates/selene_storage/migrations -S
```

Step 6 exit criteria:
- SQL parity confirmed or explicit N/A recorded without drift ambiguity.

---

### Step 7: Test Closure (must prove ONB drift is closed)

Patch files (only):
1. `crates/selene_storage/tests/ph1_onb/db_wiring.rs`
2. `crates/selene_os/src/ph1onb.rs` (test module)

Required new tests:
- Session start persists and replays pinned schema context deterministically.
- Sender verification requirement is schema-derived (no alias fallback behavior).
- Access-instance create refuses when required verification gate unresolved.
- Complete refuses when required verification gate unresolved.
- Complete succeeds without sender verification when schema does not require it.
- Resume/device integrity fail-closed behavior is preserved.

Post-step acceptance:

```bash
cargo test -p selene_storage --test db_wiring_ph1onb_tables -- --nocapture
cargo test -p selene_os ph1onb -- --nocapture
```

Step 7 exit criteria:
- Critical ONB schema-gating paths are directly tested and green.

---

### Step 8: Final Drift Proof + Audit Checkpoint

No patching in this step.

Run:

```bash
scripts/selene_design_readiness_audit.sh

rg -n "PH1\.ONB|ONB_SESSION_START_DRAFT|ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT|ONB_EMPLOYEE_SENDER_VERIFY_COMMIT|required_verification_gates|pinned_schema" \
  docs/COVERAGE_MATRIX.md docs/DB_WIRING/PH1_ONB.md docs/ECM/PH1_ONB.md docs/BLUEPRINTS/ONB_INVITED.md crates/selene_kernel_contracts/src/ph1onb.rs crates/selene_storage/src/ph1f.rs crates/selene_os/src/ph1onb.rs

git status --short
git rev-parse HEAD
git log -1 --oneline
```

Checkpoint expectations:
- Audit shows no PH1.ONB drift findings.
- ONB docs/contracts/storage/runtime/test references are coherent.
- Pinned commit hash + status are captured.

## 5) Execution Record (fill during work)

- Step 1: COMPLETED (2026-02-15)
- Step 2: COMPLETED (2026-02-15)
- Step 3: COMPLETED (2026-02-15)
- Step 4: COMPLETED (2026-02-15)
- Step 5: COMPLETED (2026-02-15)
- Step 6: COMPLETED (2026-02-15, N/A migration delta in this branch)
- Step 7: COMPLETED (2026-02-15)
- Step 8: COMPLETED (2026-02-15)

Step 1 note:
- ONB docs now lock explicit pinned schema context field names and explicit `required_verification_gates[]`.
- Legacy photo/sender capability IDs are explicitly marked compatibility-only, with schema-derived gate semantics.

Step 2 note:
- `OnbSessionStartResult` now carries pinned schema context fields and `required_verification_gates[]`.
- Validation bounds were added for new fields while keeping backward-compatible constructor defaults.

Step 3 note:
- `Ph1OnbRepo` method signatures already aligned with updated contract objects; no trait shape change required.

Step 4 note:
- ONB session start now persists and replays pinned schema context and required verification gates.
- Sender verification gating is now schema-derived and fail-closed; hardcoded alias-array logic was removed.
- Photo/sender commit paths now refuse calls when required verification gates are not required by pinned schema.

Step 5 note:
- Runtime tests now cover both required-gate and optional-gate ONB paths.
- Session-start runtime output is asserted for schema-pinned gate semantics on required flows.

Step 6 note:
- No onboarding session SQL migration surface exists in this branch for pinned schema/gate columns.
- Migration parity recorded as explicit N/A.

Step 7 note:
- ONB storage test pack expanded to verify:
  - pinned schema context persistence/replay,
  - required-gate refusal for access/complete before confirmation,
  - refusal of photo/sender commits when gates are not required.

Step 8 note:
- Canonical audit script completed with `EXIT:0`.
- PH1.ONB grep checkpoint confirms docs/contracts/storage/runtime reference parity for pinned schema + gate semantics.

## 6) Done Criteria

PH1.ONB is done for this packet only when all are true:
1. ONB session start persists/returns deterministic pinned schema context.
2. Verification gate requirements are schema-derived, not hardcoded alias-derived.
3. Access-create and complete fail-closed on unmet required verification gates.
4. Legacy capability IDs remain compatible without changing schema-driven behavior.
5. Docs, contracts, runtime, storage, and tests all match one ONB behavior model.
6. Audit checkpoint is clean for PH1.ONB drift.
