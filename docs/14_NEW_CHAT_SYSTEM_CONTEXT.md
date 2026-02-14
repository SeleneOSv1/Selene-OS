# Selene OS New Chat System Context (Current Stage)

Last updated: 2026-02-14
Purpose: complete handoff context for a new chat so work can continue from current design stage without re-discovery.
Scope: design contracts + governance + readiness + runtime verification status.

## 0) Repository State Snapshot
- Repository root: `/Users/xiamo/Documents/A-Selene/Selene-OS`
- Branch: `main`
- Pinned commit: `1aeda83`
- Last commit message: `Audit: clarify section 6 as legacy compliance evidence`
- Working tree status at snapshot creation: clean (`git status --short` produced no lines)

## 1) Canonical Truth Model (Do Not Drift)
Canonical source control for design truth is defined in:
- `docs/00_DESIGN_TRUTH_OPTION_B.md`

Single-source ownership by topic:
- Engine inventory: `docs/07_ENGINE_REGISTRY.md`
- Simulation inventory: `docs/08_SIMULATION_CATALOG.md`
- Blueprint mapping: `docs/09_BLUEPRINT_REGISTRY.md`
- Blueprint records: `docs/BLUEPRINTS/*.md`
- DB ownership summary: `docs/10_DB_OWNERSHIP_MATRIX.md`
- Design lock status: `docs/11_DESIGN_LOCK_SEQUENCE.md`
- Coverage completion state: `docs/COVERAGE_MATRIX.md`
- Memory architecture narrative: `docs/12_MEMORY_ARCHITECTURE.md`

High-level law and policy pointers:
- `docs/05_OS_CONSTITUTION.md`

## 2) Current Completion Status
Source of truth:
- `docs/COVERAGE_MATRIX.md`

Current key rows (all DONE):
- `PH1.E`: db_wiring DONE, ecm DONE, sim_catalog DONE, blueprint DONE, blockers none
- `PH1.LINK`: db_wiring DONE, ecm DONE, sim_catalog DONE, blueprint DONE, blockers none
- `PH1.REM`: db_wiring DONE, ecm DONE, sim_catalog DONE, blueprint DONE, blockers none
- `PH1.EMO`: db_wiring DONE, ecm DONE, sim_catalog DONE, blueprint DONE, blockers none

Current global matrix state:
- `TODO|BLOCKER|WIP` sweep in `docs/COVERAGE_MATRIX.md` returns no matches.

## 3) Resolved Problem Tracker (Authoritative Historical Closure)
Source of truth:
- `docs/13_PROBLEMS_TO_FIX.md`

Status:
- Items 1 through 19 are marked DONE/YES with proof notes.
- Includes closure of:
  - PH1.LINK enum/state drift
  - invitee_type canon drift
  - LINK legacy-delivery simulation separation
  - ACTIVE blueprint discipline and simulation/capability resolution
  - PH1.E tool blueprint readiness
  - PH1.REM + PH1.EMO 4-pack closure
  - kernel/db_wiring/sql parity sweeps
  - portable readiness audit runbook and process hardening

## 4) PH1.LINK Canonical State (Locked)
Primary contracts:
- `docs/DB_WIRING/PH1_LINK.md`
- `docs/ECM/PH1_LINK.md`
- `crates/selene_kernel_contracts/src/ph1link.rs`
- `crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql`

Canonical Link token lifecycle states:
- `DRAFT_CREATED | SENT | OPENED | ACTIVATED | CONSUMED | REVOKED | EXPIRED | BLOCKED`

Canonical invitee_type set:
- `COMPANY | CUSTOMER | EMPLOYEE | FAMILY_MEMBER | FRIEND | ASSOCIATE`

Legacy delivery simulations are intentionally retained as do-not-wire placeholders:
- `LINK_INVITE_SEND_COMMIT`
- `LINK_INVITE_RESEND_COMMIT`
- `LINK_DELIVERY_FAILURE_HANDLING_COMMIT`
Status must remain `LEGACY_DO_NOT_WIRE` and delivery ownership remains only:
- `LINK_DELIVER_INVITE` via `PH1.BCAST + PH1.DELIVERY`

`LINK_INVITE_EXPIRED_RECOVERY_COMMIT` is ACTIVE and state-only:
- replacement token/link_url generation only
- no direct delivery semantics in that simulation

## 5) Blueprint Layer State (Locked)
Registry source:
- `docs/09_BLUEPRINT_REGISTRY.md`

Important ACTIVE blueprints now present and aligned:
- `LINK_DELIVER_INVITE`
- `ONB_INVITED`
- `ONB_BIZ_SETUP`
- `POSITION_MANAGE`
- `MESSAGE_COMPOSE_AND_SEND`
- `MEMORY_QUERY`
- `MEMORY_FORGET_REQUEST`
- `MEMORY_REMEMBER_REQUEST`
- `TOOL_TIME_QUERY`
- `TOOL_WEATHER_QUERY`
- `REMINDER_MANAGE`
- `EMO_PROFILE_MANAGE`

Discipline rules currently satisfied:
- exactly one ACTIVE blueprint per intent_type
- ACTIVE capability_id values resolve against ECM capability definitions
- ACTIVE side-effect steps include valid simulation requirements

## 6) PH1.E Tool Layer (Locked)
Contracts:
- `docs/DB_WIRING/PH1_E.md`
- `docs/ECM/PH1_E.md`

Blueprints:
- `docs/BLUEPRINTS/TOOL_TIME_QUERY.md`
- `docs/BLUEPRINTS/TOOL_WEATHER_QUERY.md`

Coverage expectations now realized:
- PH1.E `simulations_owned = [TOOL_TIME_QUERY_COMMIT, TOOL_WEATHER_QUERY_COMMIT]`
- PH1.E `blueprints_referenced_by = [TOOL_TIME_QUERY, TOOL_WEATHER_QUERY]`
- PH1.E blockers none

## 7) PH1.REM and PH1.EMO (Locked)
PH1.REM canonical docs:
- `docs/DB_WIRING/PH1_REM.md`
- `docs/ECM/PH1_REM.md`
- `docs/BLUEPRINTS/REMINDER_MANAGE.md`

PH1.EMO canonical docs:
- `docs/DB_WIRING/PH1_EMO.md`
- `docs/ECM/PH1_EMO.md`
- `docs/BLUEPRINTS/EMO_PROFILE_MANAGE.md`

Engine registry planned rows point to canonical docs:
- `PH1.REM` -> `docs/DB_WIRING/PH1_REM.md` + `docs/ECM/PH1_REM.md`
- `PH1.EMO` -> `docs/DB_WIRING/PH1_EMO.md` + `docs/ECM/PH1_EMO.md`

## 8) Readiness Audit: Canonical Execution Path
Canonical script:
- `scripts/selene_design_readiness_audit.sh`

Why this script is canonical:
- portable awk logic (no non-portable regex features)
- explicit preflight checks
- emits pinned hash + clean/dirty tree validity note
- avoids brittle copy/paste audit block drift

Governance/runbook references:
- `docs/11_DESIGN_LOCK_SEQUENCE.md`
- `docs/13_PROBLEMS_TO_FIX.md`

## 9) Runtime Verification Status (End-to-End)
Runtime correctness was verified separately from design audit.

Executed commands:
- `cargo test --workspace`
- `cargo test --workspace --release`

Result:
- all workspace unit tests passed
- all workspace integration/db_wiring tests passed
- all doc-tests passed
- failures: `0`

Post-run state:
- working tree remained clean

## 10) Required Operating Rules for Next Chat
1. Use canonical documents only for truth decisions (Section 1 above).
2. Do not reintroduce legacy LINK send/resend semantics into active wiring.
3. Keep PH1.LINK invitee_type and lifecycle enums aligned across docs + kernel + SQL.
4. Keep ACTIVE blueprint discipline strict:
   - capability IDs must resolve to ECM
   - side-effects must have valid simulation IDs
5. Use `scripts/selene_design_readiness_audit.sh` for design-readiness checks.
6. Use Rust/shell-native workflows; avoid Python in fix flows per constitutional workflow law.

## 11) Repro Command Pack (New Chat Quick Start)
From repo root:

```bash
git branch --show-current
git rev-parse HEAD
git status --short

scripts/selene_design_readiness_audit.sh

cargo test --workspace
cargo test --workspace --release
```

Optional focused sweeps:

```bash
# NOTE: Run scripts/selene_design_readiness_audit.sh for drift/banned-token sweeps; do not paste the regex here.
rg -n "TODO|BLOCKER|WIP" docs/COVERAGE_MATRIX.md
```

## 12) Fast Orientation Paths for a Fresh Chat
Read in this order:
1. `docs/00_DESIGN_TRUTH_OPTION_B.md`
2. `docs/07_ENGINE_REGISTRY.md`
3. `docs/COVERAGE_MATRIX.md`
4. `docs/08_SIMULATION_CATALOG.md`
5. `docs/09_BLUEPRINT_REGISTRY.md`
6. `docs/13_PROBLEMS_TO_FIX.md`
7. `scripts/selene_design_readiness_audit.sh`

Then open subsystem contracts as needed:
- Link: `docs/DB_WIRING/PH1_LINK.md`, `docs/ECM/PH1_LINK.md`
- Tools: `docs/DB_WIRING/PH1_E.md`, `docs/ECM/PH1_E.md`
- Reminders: `docs/DB_WIRING/PH1_REM.md`, `docs/ECM/PH1_REM.md`
- Emotion: `docs/DB_WIRING/PH1_EMO.md`, `docs/ECM/PH1_EMO.md`
- Memory: `docs/12_MEMORY_ARCHITECTURE.md`, `docs/DB_WIRING/PH1_M.md`, `docs/ECM/PH1_M.md`
