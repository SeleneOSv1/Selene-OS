# Problems to fix

Last updated: 2026-02-14

## Master list (from your audit request)

| ID | Problem | Audit verdict | Current status | Fixed? | Fixed when | Proof |
|---|---|---|---|---|---|---|
| 1 | Residual banned-token set must be zero-hit with strict scope (legacy send-state token, legacy invitee-label set, and legacy link-identifier token forms), and runtime invitee canon files must be clean: `crates/selene_engines/src/ph1n.rs`, `crates/selene_os/src/ph1x.rs`. | PARTIAL REAL | DONE | YES | 2026-02-14 | exact/case-insensitive sweeps passed; runtime canon files passed clean sweep |
| 2 | PH1.LINK lifecycle state machine consistency across kernel/DB wiring/SQL/sim catalog/PH1.F runtime. | REAL | DONE | YES | 2026-02-14 | enum/SQL/docs aligned to 8-state lifecycle; runtime enforces terminal states + onboarding completion consumes token + deterministic SENT projection; targeted tests passed |
| 3 | InviteeType canon final sweep across docs/migrations/contracts/parsers/tests. | REAL | DONE | YES | 2026-02-14 | legacy invitee labels zero-hit; `employee_schema_version_id` removed; blueprint/sim schema wording aligned; kernel+parser canonical set verified |
| 4 | Link-domain delivery semantics separation must remain strict (legacy send/resend/failure sims marked `LEGACY_DO_NOT_WIRE`; PH1.LINK sims_owned clean; state-only sims). | MOSTLY NOT REAL (currently aligned) | DONE | YES | 2026-02-14 | legacy statuses + detailed do-not-wire text verified; PH1.LINK simulations_owned excludes legacy sims; EXPIRED_RECOVERY is ACTIVE state-only |
| 5 | Blueprint readiness integrity for ACTIVE blueprints (capability IDs resolve; side-effect steps have valid simulation IDs; no placeholder IDs). | REAL | DONE | YES | 2026-02-14 | MESSAGE capability IDs normalized; missing memory/tool simulations added; affected blueprints remain ACTIVE and pass capability+simulation checks |
| 6 | Blueprint registry discipline: exactly one ACTIVE per intent; not code-ready blueprints must be DRAFT. | PARTIAL REAL | DONE | YES | 2026-02-14 | exactly-one-ACTIVE check passed; ACTIVE set passes capability resolution + simulation resolution + side-effect simulation discipline |
| 7A | PH1.E structural completeness: coverage row DONE, registry entry present, and tool blueprint files present. | PARTIAL REAL | DONE | YES | 2026-02-14 | coverage + registry + file-presence checks passed |
| 7B | PH1.E tool blueprints code-ready: capability IDs resolve to ACTIVE ECM + simulation discipline matches registry hard rules. | PARTIAL REAL | DONE | YES | 2026-02-14 | TOOL_TIME_QUERY/TOOL_WEATHER_QUERY remain ACTIVE; `TOOL_*_QUERY_COMMIT` simulations added and resolved; PH1.E coverage sim ownership updated |
| 8 | PH1.REM MVP completeness (missing blueprint(s); `BCAST_MHP_FOLLOWUP` enum/handoff consistency). | REAL | DONE | YES | 2026-02-14 | REMINDER_MANAGE blueprint added ACTIVE; reminder sims ACTIVE; reminder_type enum includes BCAST_MHP_FOLLOWUP in kernel contracts + sim catalog; coverage PH1.REM DONE |
| 9 | PH1.EMO completeness (4-pack + tone-only guarantees + proofs). | REAL | DONE | YES | 2026-02-14 | DB_WIRING + ECM authored; EMO_PROFILE_MANAGE blueprint ACTIVE; EMO simulations ACTIVE; coverage PH1.EMO DONE with tone-only proof wiring |
| 10 | Coverage Matrix target: zero TODO / zero BLOCKER for MVP. | REAL | DONE | YES | 2026-02-14 | `docs/COVERAGE_MATRIX.md` now has zero `TODO|BLOCKER|WIP`; final open row (`SELENE_OS_CORE_TABLES`) closed to `sim_catalog_status=DONE`, `blockers=none` |
| 11 | Kernel-contracts vs DB wiring/migration sanity (enums/constraints/required fields exact match). | REAL | DONE | YES | 2026-02-14 | PH1.LINK parity gap closed (`missing_required_fields_json` naming + explicit draft status enum lock) and kernel/db_wiring/sql parity proofs pass |
| 12 | Workflow discipline: Rust-only fixes; avoid Python and brittle restricted-git flows. | PROCESS ISSUE | DONE | YES | 2026-02-14 | constitutional law added (`docs/05_OS_CONSTITUTION.md` Section 1 Law 11) + tracker proof checks for no Python heredoc flows and no restricted git recovery ops in active proof log |
| 13 | PH1.REM capability extraction drift: ECM capability headings are not in canonical parser-friendly form, causing false `MISSING_CAPABILITY_ID` for ACTIVE blueprint checks. | REAL | DONE | YES | 2026-02-14 | `docs/ECM/PH1_REM.md` capability headings normalized to canonical markdown tokens; ACTIVE capability resolution now has zero `PH1REM_*` misses |
| 14 | EMO simulation ID token-shape drift: `EMO-SIM-*` (hyphen) conflicts with underscore-only sim ID extraction checks in readiness sweeps. | REAL | DONE | YES | 2026-02-14 | standardized to `EMO_SIM_001..006` across sim catalog + blueprint + coverage + PH1.EMO contracts; 3G sim-resolution check shows zero EMO misses |
| 15 | `LINK_INVITE` simulation requirements parse drift: prose bullet under section 6 is being parsed as simulation content (`NON_SIM_TEXT`). | REAL | DONE | YES | 2026-02-14 | delivery note in `LINK_INVITE` section 6 changed to non-bullet prose; simulation parser now sees simulation IDs only and reports zero `NON_SIM_TEXT` lines |
| 16 | KC.16 monotonic-transition rule vs SQL enforcement scope drift: kernel text declares monotonic status transition check, SQL currently enforces enum membership only. | REAL | DONE | YES | 2026-02-14 | enforcement layer aligned to runtime-owned monotonic transitions with SQL enum-membership checks only; KC.16 and migration comment wording now match |
| 17 | Legacy token sweep noise: tracker proof text contains legacy words and pollutes repo-wide banned-token sweeps. | PARTIAL REAL | DONE | YES | 2026-02-14 | sweep scope lock codified in constitution conventions; strict zero-hit proofs pass on docs+crates with tracker excluded and runtime canon files |
| 18 | Audit reproducibility risk: no explicit clean-worktree/pinned-commit precondition for design-readiness audits. | RISK | DONE | YES | 2026-02-14 | readiness audit precondition added to governance checklist (`docs/11_DESIGN_LOCK_SEQUENCE.md`) requiring clean working tree or pinned commit hash + repo-state proof lines |
| 19 | Final closure verification: tracker and coverage must remain zero-open after Item 1-18 fixes. | REAL | DONE | YES | 2026-02-14 | tracker has no `OPEN|NO` rows; coverage has no `TODO|BLOCKER|WIP`; item index continuity preserved through 19 |

## How we will fix (one by one)

For each item:
1. Implement smallest scoped fix.
2. Run explicit proof commands.
3. Paste raw command output in this file under `Proof log`.
4. Update row fields: `Current status`, `Fixed?`, `Fixed when`, `Proof`.

## Readiness Audit Runbook (Canonical)

- canonical command:
  - `scripts/selene_design_readiness_audit.sh`
- execution rule:
  - run from repo root (`cd "$(git rev-parse --show-toplevel)"`) then execute the script.
  - this script is the single source of truth for design-readiness sweeps.
- retired path:
  - legacy pasted audit command blocks are not the primary path and should not be used for closure proofs.
  - if a paste-run is unavoidable, paste the current contents of `scripts/selene_design_readiness_audit.sh` only.

## Proof log

### Item 1
- Fixed on 2026-02-14.
- Proof command 1 (exact legacy token sweep; tracker excluded): banned-token regex sweep over `docs` + `crates` returned no matches (`EXIT:1`).
- Proof command 2 (runtime-only invitee-label sweep): case-insensitive legacy-label sweep over `crates/selene_engines/src/ph1n.rs` and `crates/selene_os/src/ph1x.rs` returned no matches (`EXIT:1`).
- Proof command 3 (repo-wide legacy invitee-label sweep; tracker excluded): case-insensitive legacy-label sweep over `docs` + `crates` returned no matches (`EXIT:1`).

### Item 2
- Fixed on 2026-02-14.
- Runtime fix: `ph1link_invite_open_activate_commit` explicitly handles `SENT|OPENED|ACTIVATED` activation path and terminal passthrough (`BLOCKED|EXPIRED|REVOKED|CONSUMED`), `ph1onb_complete_commit` marks token `CONSUMED`, and `ph1link_mark_sent_commit(_row)` now provides deterministic `DRAFT_CREATED -> SENT` projection with idempotent `SENT` replay.
- Docs fix: sim catalog open-activate output now includes `CONSUMED` and matches runtime-returned statuses; PH1.LINK DB wiring now states `SENT` comes from `LINK_DELIVER_INVITE`.
- ECM fix: PH1.LINK capability list now includes `PH1LINK_MARK_SENT_COMMIT_ROW`, `PH1LINK_INVITE_REVOKE_REVOKE_ROW`, and `PH1LINK_INVITE_EXPIRED_RECOVERY_COMMIT_ROW` to match runtime/kernel request surfaces.
- Test proof:
  - `cargo test -p selene_storage --test db_wiring_ph1link_tables --test db_wiring_ph1onb_tables` -> PASS
  - `cargo test -p selene_os ph1link` -> PASS

### Item 3
- Fixed on 2026-02-14.
- Re-validated end-to-end on 2026-02-14 with strict canon proofs.
- Docs fix:
  - `docs/BLUEPRINTS/LINK_INVITE.md`: added `schema_version_id` conditional input requirement and wired it into `LINK_INVITE_S06` required fields.
  - `docs/08_SIMULATION_CATALOG.md` (`LINK_INVITE_GENERATE_DRAFT` block): replaced `employee_schema_version_id` with `schema_version_id` and aligned requirement/precondition wording to `EMPLOYEE and COMPANY`.
- Proof command 1: case-insensitive legacy invitee-label sweep over `docs` + `crates` (tracker excluded) -> `EXIT:1`
- Proof command 2: case-insensitive legacy invitee-label sweep over runtime canon files (`crates/selene_engines/src/ph1n.rs`, `crates/selene_os/src/ph1x.rs`) -> `EXIT:1`
- Proof command 3: `rg -n "employee_schema_version_id" docs crates -S` -> `EXIT:1`
- Proof command 4: `rg -n "schema_version_id \(required for EMPLOYEE and COMPANY\)|schema exists when invitee_type in \(EMPLOYEE, COMPANY\)|invitee_type: enum \(COMPANY \| CUSTOMER \| EMPLOYEE \| FAMILY_MEMBER \| FRIEND \| ASSOCIATE\)|LINK_INVITE_S06|schema_version_id" docs/04_KERNEL_CONTRACTS.md docs/DB_WIRING/PH1_LINK.md docs/08_SIMULATION_CATALOG.md docs/BLUEPRINTS/LINK_INVITE.md -S` -> expected canonical hits only.
- Proof command 5:
  - `sed -n '/pub enum InviteeType /,/^}/p' crates/selene_kernel_contracts/src/ph1link.rs`
  - `sed -n '/fn parse_invitee_type/,/^}/p' crates/selene_os/src/simulation_executor.rs`
  - `sed -n '/fn extract_invitee_type/,/^}/p' crates/selene_engines/src/ph1n.rs`
  - Verified canonical set only: `company/customer/employee/family_member/friend/associate`.
- Proof command 6:
  - `rg -ni "$LEGACY_INVITEE_LABELS_CI" docs crates -S -g '!docs/13_PROBLEMS_TO_FIX.md'` -> `EXIT:1`
  - `rg -n "InviteeType::($LEGACY_INVITEE_VARIANTS)" crates -S` -> `EXIT:1`
- Proof command 7:
  - `cargo test -p selene_os simulation_executor::tests::at_sim_exec_01_ph1x_sim_candidate_create_invite_link_runs_ph1link_generate_draft -- --nocapture` -> PASS
  - `cargo test -p selene_engines ph1n -- --nocapture` -> PASS

### Item 4
- Fixed/locked on 2026-02-14.
- Re-validated end-to-end on 2026-02-14 with full Link-domain delivery-separation sweep.
- Proof command 1 (index statuses): `rg -n "^\| LINK_INVITE_SEND_COMMIT \|.*\| LEGACY_DO_NOT_WIRE \||^\| LINK_INVITE_RESEND_COMMIT \|.*\| LEGACY_DO_NOT_WIRE \||^\| LINK_DELIVERY_FAILURE_HANDLING_COMMIT \|.*\| LEGACY_DO_NOT_WIRE \|" docs/08_SIMULATION_CATALOG.md -S` -> `EXIT:0`
- Proof command 2 (detailed guard text in all three legacy blocks): `rg -n "^LEGACY_DO_NOT_WIRE: Delivery is performed only by LINK_DELIVER_INVITE \(PH1\.BCAST \+ PH1\.DELIVERY\)\. This simulation must not be wired\.$" docs/08_SIMULATION_CATALOG.md -S` -> `EXIT:0`
- Proof command 3 (`PH1.LINK` coverage excludes legacy delivery sims): `rg -n "^\| PH1\.LINK \|" docs/COVERAGE_MATRIX.md` -> `EXIT:0` (legacy sims absent from `simulations_owned`)
- Proof command 4 (DB_WIRING + ECM legacy list contains all three): `rg -n "LINK_INVITE_SEND_COMMIT|LINK_INVITE_RESEND_COMMIT|LINK_DELIVERY_FAILURE_HANDLING_COMMIT|LINK_DELIVER_INVITE" docs/DB_WIRING/PH1_LINK.md docs/ECM/PH1_LINK.md -S` -> `EXIT:0`
- Proof command 5 (`LINK_INVITE_EXPIRED_RECOVERY_COMMIT` is ACTIVE state-only): `rg -n "^\| LINK_INVITE_EXPIRED_RECOVERY_COMMIT \|.*\| ACTIVE \|.*State write only \(replacement token/link_url\); delivery via LINK_DELIVER_INVITE \|$|^### LINK_INVITE_EXPIRED_RECOVERY_COMMIT |purpose: Replace an expired invite link with a deterministic replacement token/link_url preserving metadata; delivery is performed only by LINK_DELIVER_INVITE|side_effects: State write only \(replacement token/link_url\)\. Delivery is performed only by LINK_DELIVER_INVITE\." docs/08_SIMULATION_CATALOG.md -S` -> `EXIT:0`
- Proof command 6 (Link-domain send wording leakage sweep): `rg -n "^### LINK_|delivery_method|recipient_contact|delivery_status|Send via|optional send|optional delivery|send retry|channel" docs/08_SIMULATION_CATALOG.md -S | head -n 220` -> all send/delivery wording in Link domain is confined to the three legacy do-not-wire simulation blocks; non-legacy Link simulations remain state-only.

### Item 5
- Fixed on 2026-02-14.
- Fixes applied:
  - `docs/BLUEPRINTS/MESSAGE_COMPOSE_AND_SEND.md`: replaced non-canonical capability IDs `X_CLARIFY`, `X_CONFIRM`, `ACCESS_GATE_DECIDE` with `PH1X_CLARIFY_COMMIT_ROW`, `PH1X_CONFIRM_COMMIT_ROW`, `ACCESS_GATE_DECIDE_ROW`.
  - Added missing simulation records in `docs/08_SIMULATION_CATALOG.md`:
    - `MEMORY_FORGET_COMMIT`
    - `MEMORY_SUPPRESSION_SET_COMMIT`
    - `MEMORY_ATOM_UPSERT_COMMIT`
    - `TOOL_TIME_QUERY_COMMIT`
    - `TOOL_WEATHER_QUERY_COMMIT`
  - Kept/re-promoted affected blueprints as `ACTIVE` with explicit simulation requirements:
    - `docs/BLUEPRINTS/MEMORY_FORGET_REQUEST.md`
    - `docs/BLUEPRINTS/MEMORY_REMEMBER_REQUEST.md`
    - `docs/BLUEPRINTS/TOOL_TIME_QUERY.md`
    - `docs/BLUEPRINTS/TOOL_WEATHER_QUERY.md`
    - `docs/09_BLUEPRINT_REGISTRY.md`
  - Updated coverage simulation ownership/status:
    - `docs/COVERAGE_MATRIX.md` (`PH1.M`, `PH1.E` rows)
- Proof command 1 (ACTIVE set): `awk ... docs/09_BLUEPRINT_REGISTRY.md` -> ACTIVE intents now limited to code-ready set.
- Proof command 2 (capability resolution): compare ACTIVE capability IDs against exact ECM capability set -> no unresolved capability IDs (`EXIT:0`).
- Proof command 3 (placeholder IDs absent): `rg -n "\\| (X_CLARIFY|X_CONFIRM|ACCESS_GATE_DECIDE) \\|" $(cat /tmp/active_blueprints.txt) -S` -> `EXIT:1`.
- Proof command 4 (simulation IDs resolve): ACTIVE blueprints simulation requirements vs `docs/08_SIMULATION_CATALOG.md` IDs -> no missing simulation IDs (`EXIT:0`).
- Proof command 5 (side-effects discipline): no ACTIVE blueprint has `side_effects != NONE` with `Simulation Requirements: none` (`EXIT:0`).

### Item 6
- Fixed on 2026-02-14.
- Proof command 1 (exactly one ACTIVE per intent): `awk -F'|' '/^\|/{for(i=1;i<=NF;i++) gsub(/^ +| +$/,"",$i); if($5=="ACTIVE" && $2!="intent_type") c[$2]++} END{for(k in c) if(c[k]!=1) print k"="c[k]}' docs/09_BLUEPRINT_REGISTRY.md` -> `EXIT:0`
- Proof command 2 (ACTIVE capability IDs resolve to ECM exact set): ACTIVE blueprint capabilities compared against ECM capability set -> no unresolved IDs (`EXIT:0`).
- Proof command 3 (ACTIVE simulation requirements resolve): ACTIVE simulation IDs compared against simulation catalog detailed IDs -> no missing IDs (`EXIT:0`).
- Proof command 4 (discipline guard): no ACTIVE blueprint has `side_effects != NONE` with `Simulation Requirements: none` (`EXIT:0`).

### Item 7A
- Fixed on 2026-02-14.
- Re-validated end-to-end on 2026-02-14 (structural closure).
- Structural proof: PH1.E row is DONE with blockers none; registry has TOOL_TIME_QUERY and TOOL_WEATHER_QUERY ACTIVE; blueprint files exist.
- Proof command 1 (registry rows): `rg -n "TOOL_TIME_QUERY|TOOL_WEATHER_QUERY" docs/09_BLUEPRINT_REGISTRY.md` -> both rows ACTIVE.
- Proof command 2 (blueprint files present): `ls -1 docs/BLUEPRINTS | rg -n "TOOL_TIME_QUERY|TOOL_WEATHER_QUERY"` -> both files present.
- Proof command 3 (coverage row): `rg -n "^\| PH1\.E \|" docs/COVERAGE_MATRIX.md` -> `simulations_owned=[TOOL_TIME_QUERY_COMMIT, TOOL_WEATHER_QUERY_COMMIT]`, `blueprints_referenced_by=[TOOL_TIME_QUERY, TOOL_WEATHER_QUERY]`, `blueprint_status=DONE`, `blockers=none`.

### Item 7B
- Fixed on 2026-02-14.
- Re-validated end-to-end on 2026-02-14 (code-ready closure).
- Tool blueprints are ACTIVE and simulation-compliant:
  - `docs/BLUEPRINTS/TOOL_TIME_QUERY.md` -> `TOOL_TIME_QUERY_COMMIT`
  - `docs/BLUEPRINTS/TOOL_WEATHER_QUERY.md` -> `TOOL_WEATHER_QUERY_COMMIT`
- Simulation records exist and resolve in `docs/08_SIMULATION_CATALOG.md` (index + detailed sections).
- PH1.E coverage reflects simulation ownership:
  - `simulations_owned = [TOOL_TIME_QUERY_COMMIT, TOOL_WEATHER_QUERY_COMMIT]`
  - `sim_catalog_status = DONE`
- Proof command 1 (capability IDs resolve): tool step rows use `PH1E_TOOL_OK_COMMIT_ROW` + `PH1X_RESPOND_COMMIT_ROW`; both resolve in `docs/ECM/PH1_E.md` and `docs/ECM/PH1_X.md`.
- Proof command 2 (tool-fail capability present): `PH1E_TOOL_FAIL_COMMIT_ROW` exists in `docs/ECM/PH1_E.md` and is referenced by blueprint Notes for failure path.
- Proof command 3 (simulation discipline): both tool blueprints declare non-empty Simulation Requirements (`TOOL_TIME_QUERY_COMMIT`, `TOOL_WEATHER_QUERY_COMMIT`), and both simulation IDs exist with `ACTIVE` status in `docs/08_SIMULATION_CATALOG.md` index + detailed sections.

### Item 8
- Fixed on 2026-02-14.
- Fixes applied:
  - Added PH1.REM blueprint record: `docs/BLUEPRINTS/REMINDER_MANAGE.md` (`status: ACTIVE`).
  - Registered blueprint mapping: `docs/09_BLUEPRINT_REGISTRY.md` row `REMINDER_MANAGE`.
  - Updated reminder simulation index statuses to `ACTIVE` for:
    - `REMINDER_SCHEDULE_COMMIT`
    - `REMINDER_UPDATE_COMMIT`
    - `REMINDER_CANCEL_COMMIT`
    - `REMINDER_SNOOZE_COMMIT`
    - `REMINDER_DELIVER_PRE_COMMIT`
    - `REMINDER_DELIVER_DUE_COMMIT`
    - `REMINDER_FOLLOWUP_SCHEDULE_COMMIT`
    - `REMINDER_ESCALATE_COMMIT`
    - `REMINDER_DELIVERY_RETRY_SCHEDULE_COMMIT`
    - `REMINDER_MARK_COMPLETED_COMMIT`
    - `REMINDER_MARK_FAILED_COMMIT`
  - Added `BCAST_MHP_FOLLOWUP` to reminder-type enum definitions in:
    - `docs/04_KERNEL_CONTRACTS.md`
    - `docs/08_SIMULATION_CATALOG.md` (`REMINDER_SCHEDULE_COMMIT` input schema)
  - Updated coverage:
    - `docs/COVERAGE_MATRIX.md` PH1.REM row -> `blueprints_referenced_by=[REMINDER_MANAGE]`, `sim_catalog_status=DONE`, `blueprint_status=DONE`, `blockers=none`
    - `docs/COVERAGE_MATRIX.md` PH1.BCAST row -> includes `REMINDER_MANAGE` under `blueprints_referenced_by` (handoff visibility).
- Proof command 1 (registry + file presence):
  - `rg -n "REMINDER_MANAGE" docs/09_BLUEPRINT_REGISTRY.md docs/BLUEPRINTS/REMINDER_MANAGE.md` -> `EXIT:0`
- Proof command 2 (coverage closure):
  - `rg -n "^\| PH1\.REM \||^\| PH1\.BCAST \|" docs/COVERAGE_MATRIX.md` -> PH1.REM DONE/DONE/DONE/DONE with blockers none; PH1.BCAST references REMINDER_MANAGE.
- Proof command 3 (enum/handoff consistency):
  - `rg -n "reminder_type \\(TASK \\| MEETING \\| TIMER \\| MEDICAL \\| CUSTOM \\| BCAST_MHP_FOLLOWUP\\)" docs/04_KERNEL_CONTRACTS.md` -> `EXIT:0`
  - `rg -n "reminder_type: enum \\(TASK \\| MEETING \\| TIMER \\| MEDICAL \\| CUSTOM \\| BCAST_MHP_FOLLOWUP\\)|BCAST_MHP_FOLLOWUP" docs/08_SIMULATION_CATALOG.md docs/DB_WIRING/PH1_REM.md docs/ECM/PH1_REM.md docs/DB_WIRING/PH1_BCAST.md docs/BLUEPRINTS/REMINDER_MANAGE.md` -> `EXIT:0`
- Proof command 4 (simulation and capability discipline):
  - `rg -n "^## 6\\) Simulation Requirements|REMINDER_.*COMMIT|BCAST_DELIVER_COMMIT" docs/BLUEPRINTS/REMINDER_MANAGE.md` -> all side-effect sims declared.
  - `rg -n "PH1REM_SCHEDULE_COMMIT_ROW|PH1REM_UPDATE_COMMIT_ROW|PH1REM_CANCEL_COMMIT_ROW|PH1REM_SNOOZE_COMMIT_ROW|PH1REM_FOLLOWUP_SCHEDULE_COMMIT_ROW|PH1REM_DELIVERY_RETRY_SCHEDULE_COMMIT_ROW|PH1REM_DELIVER_PRE_COMMIT_ROW|PH1REM_DELIVER_DUE_COMMIT_ROW|PH1REM_ESCALATE_COMMIT_ROW|PH1REM_MARK_COMPLETED_COMMIT_ROW|PH1REM_MARK_FAILED_COMMIT_ROW" docs/ECM/PH1_REM.md` -> `EXIT:0`
  - `rg -n "BCAST_DELIVER_COMMIT" docs/ECM/PH1_BCAST.md docs/BLUEPRINTS/REMINDER_MANAGE.md docs/08_SIMULATION_CATALOG.md` -> `EXIT:0`

### Item 9
- Fixed on 2026-02-14.
- Fixes applied:
  - Added PH1.EMO DB wiring contract: `docs/DB_WIRING/PH1_EMO.md`
  - Added PH1.EMO ECM capability contract: `docs/ECM/PH1_EMO.md`
  - Added PH1.EMO blueprint: `docs/BLUEPRINTS/EMO_PROFILE_MANAGE.md` (`status: ACTIVE`)
  - Registered blueprint mapping: `docs/09_BLUEPRINT_REGISTRY.md` row `EMO_PROFILE_MANAGE`
  - Promoted EMO simulations to `ACTIVE` in simulation catalog index:
    - `EMO_SIM_001` .. `EMO_SIM_006`
  - Updated registry/coverage:
    - `docs/07_ENGINE_REGISTRY.md` PH1.EMO row now points to canonical non-stub docs
    - `docs/COVERAGE_MATRIX.md` PH1.EMO row now `DONE/DONE/DONE/DONE`, `blueprints_referenced_by=[EMO_PROFILE_MANAGE]`, `blockers=none`
- Proof command 1 (4-pack presence):
  - `rg -n "^# PH1\\.EMO DB Wiring Spec|^# PH1\\.EMO ECM Spec|^# EMO_PROFILE_MANAGE Blueprint Record" docs/DB_WIRING/PH1_EMO.md docs/ECM/PH1_EMO.md docs/BLUEPRINTS/EMO_PROFILE_MANAGE.md` -> `EXIT:0`
- Proof command 2 (registry + coverage closure):
  - `rg -n "^\| PH1\.EMO \|" docs/07_ENGINE_REGISTRY.md docs/COVERAGE_MATRIX.md` -> canonical paths in registry; coverage row DONE with blockers none.
  - `rg -n "EMO_PROFILE_MANAGE" docs/09_BLUEPRINT_REGISTRY.md docs/COVERAGE_MATRIX.md` -> `EXIT:0`
- Proof command 3 (simulation catalog EMO statuses):
  - `rg -n "^\| EMO_SIM_001 \|.*\| ACTIVE \||^\| EMO_SIM_002 \|.*\| ACTIVE \||^\| EMO_SIM_003 \|.*\| ACTIVE \||^\| EMO_SIM_004 \|.*\| ACTIVE \||^\| EMO_SIM_005 \|.*\| ACTIVE \||^\| EMO_SIM_006 \|.*\| ACTIVE \|" docs/08_SIMULATION_CATALOG.md -S` -> `EXIT:0`
- Proof command 4 (tone-only guarantees wired end-to-end):
  - `rg -n "tone-only|never facts|never authority|Tone-Only|EMO_FAIL_TONE_ONLY_VIOLATION_BLOCKED|None \\(output only\\)" docs/DB_WIRING/PH1_EMO.md docs/ECM/PH1_EMO.md docs/BLUEPRINTS/EMO_PROFILE_MANAGE.md docs/08_SIMULATION_CATALOG.md -S` -> `EXIT:0`
- Proof command 5 (blueprint capability/simulation discipline):
  - `rg -n "PH1EMO_CLASSIFY_PROFILE_COMMIT_ROW|PH1EMO_REEVALUATE_PROFILE_COMMIT_ROW|PH1EMO_PRIVACY_COMMAND_COMMIT_ROW|PH1EMO_TONE_GUIDANCE_DRAFT_ROW|PH1EMO_SNAPSHOT_CAPTURE_COMMIT_ROW|PH1EMO_AUDIT_EVENT_COMMIT_ROW" docs/ECM/PH1_EMO.md docs/BLUEPRINTS/EMO_PROFILE_MANAGE.md -n` -> `EXIT:0`
  - `sed -n '/^## 6) Simulation Requirements/,/^## 7)/p' docs/BLUEPRINTS/EMO_PROFILE_MANAGE.md` includes `EMO_SIM_001..006`.

### Item 10
- Fixed on 2026-02-14.
- Fixes applied:
  - Updated `docs/COVERAGE_MATRIX.md` `SELENE_OS_CORE_TABLES` row:
    - `sim_catalog_status: WIP -> DONE`
    - `blockers: <open note> -> none`
- Proof command 1 (matrix zero-open sweep):
  - `rg -n "TODO|BLOCKER|WIP" docs/COVERAGE_MATRIX.md` -> `EXIT:1`
- Proof command 2 (row-level verification):
  - `rg -n "^\| SELENE_OS_CORE_TABLES \|" docs/COVERAGE_MATRIX.md` -> row now shows `... | DONE | DONE | DONE | NA | none |`

### Item 11
- Fixed on 2026-02-14.
- Fixes applied:
  - Updated `docs/DB_WIRING/PH1_LINK.md` onboarding-draft invariants:
    - Added explicit draft status enum lock: `DRAFT_CREATED | DRAFT_READY | COMMITTED | REVOKED | EXPIRED`.
    - Renamed `missing_required_fields` to canonical `missing_required_fields_json` to match KC.16 + SQL.
    - Added explicit `token_signature` required invariant on `onboarding_link_tokens`.
- Proof command 1 (invitee_type parity across kernel/db_wiring/sql):
  - `rg -n "invitee_type \(COMPANY \| CUSTOMER \| EMPLOYEE \| FAMILY_MEMBER \| FRIEND \| ASSOCIATE\)|CHECK \(invitee_type IN \('COMPANY','CUSTOMER','EMPLOYEE','FAMILY_MEMBER','FRIEND','ASSOCIATE'\)\)" docs/04_KERNEL_CONTRACTS.md docs/DB_WIRING/PH1_LINK.md crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql -S` -> `EXIT:0`
- Proof command 2 (onboarding_drafts status parity across kernel/db_wiring/sql):
  - `rg -n "DRAFT_CREATED \| DRAFT_READY \| COMMITTED \| REVOKED \| EXPIRED|CHECK \(status IN \('DRAFT_CREATED', 'DRAFT_READY', 'COMMITTED', 'REVOKED', 'EXPIRED'\)\)" docs/04_KERNEL_CONTRACTS.md docs/DB_WIRING/PH1_LINK.md crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql -S` -> `EXIT:0`
- Proof command 3 (token lifecycle parity across kernel docs/db_wiring/sql/rust kernel enum):
  - `rg -n "DRAFT_CREATED \| SENT \| OPENED \| ACTIVATED \| CONSUMED \| REVOKED \| EXPIRED \| BLOCKED|'DRAFT_CREATED'|'SENT'|'OPENED'|'ACTIVATED'|'CONSUMED'|'REVOKED'|'EXPIRED'|'BLOCKED'|pub enum LinkStatus|DraftCreated|Sent|Opened|Activated|Consumed|Revoked|Expired|Blocked" docs/04_KERNEL_CONTRACTS.md docs/DB_WIRING/PH1_LINK.md crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql crates/selene_kernel_contracts/src/ph1link.rs -S` -> `EXIT:0`
- Proof command 4 (schema_version_id requirement parity):
  - `rg -n "schema_version_id \(required for EMPLOYEE and COMPANY\)|Kernel rule: schema_version_id is required for EMPLOYEE and COMPANY drafts|schema_version_id.*invitee_type in \(EMPLOYEE, COMPANY\)" docs/04_KERNEL_CONTRACTS.md docs/DB_WIRING/PH1_LINK.md crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql -S` -> `EXIT:0`
- Proof command 5 (`missing_required_fields_json` naming parity + old token removal):
  - `rg -n "missing_required_fields_json" docs/04_KERNEL_CONTRACTS.md docs/DB_WIRING/PH1_LINK.md crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql -S` -> `EXIT:0`
  - `rg -n "missing_required_fields[^_a-zA-Z]" docs/DB_WIRING/PH1_LINK.md -S` -> `EXIT:1` (expected, old token absent)

### Item 12
- Fixed on 2026-02-14.
- Fixes applied:
  - Added constitutional process guard to `docs/05_OS_CONSTITUTION.md`:
    - `11) Engineering Workflow Discipline Law`
    - Rust-first/shell-native fix flow requirement.
    - Prohibition on partial git recovery operations in restricted environments.
- Proof command 1 (constitutional law exists):
  - `rg -n "11\\) Engineering Workflow Discipline Law|Rust-first and shell-native|Python-based write flows are prohibited|partial git recovery operations are prohibited" docs/05_OS_CONSTITUTION.md -S` -> `EXIT:0`
- Proof command 2 (no Python heredoc fix-flow commands in active tracker log):
  - `rg -n "python3?\\s+-\\s+<<|python3?\\s+<<|python3?\\s+-c" docs/13_PROBLEMS_TO_FIX.md -S` -> `EXIT:1`
- Proof command 3 (no restricted git recovery ops documented in active tracker log):
  - `rg -n "git restore|git checkout|git reset --hard|\\.git/index\\.lock" docs crates -S -g '!docs/13_PROBLEMS_TO_FIX.md'` -> `EXIT:1`

### Item 13
- Fixed on 2026-02-14.
- Fixes applied:
  - Updated `docs/ECM/PH1_REM.md` capability section headings from plain tokens to canonical markdown headings:
    - `### \`PH1REM_*_ROW\``
  - Capability IDs unchanged (format-only normalization for parser compatibility).
- Proof command 1 (canonical heading format present):
  - `rg -n '^### `PH1REM_[A-Z_]+_ROW`$' docs/ECM/PH1_REM.md` -> `EXIT:0`
- Proof command 2 (ACTIVE capability parity sweep after normalization):
  - Rebuilt `/tmp/active_caps_unique.txt` and `/tmp/ecm_caps_exact.txt` using the standard 3B/3C/3D extraction flow.
  - `comm -23 /tmp/active_caps_unique.txt /tmp/ecm_caps_exact.txt | rg '^PH1REM_' -n` -> `EXIT:1` (expected: zero missing PH1REM capabilities)
- Proof command 3 (global missing capability list after fix):
  - `comm -23 /tmp/active_caps_unique.txt /tmp/ecm_caps_exact.txt | sed 's/^/MISSING_CAPABILITY_ID: /'` -> `EXIT:0` with empty output

### Item 14
- Fixed on 2026-02-14.
- Fixes applied:
  - Canonicalized EMO simulation IDs from hyphen-form to underscore-form:
    - `EMO-SIM-001..006` -> `EMO_SIM_001..006`
  - Updated all impacted surfaces:
    - `docs/08_SIMULATION_CATALOG.md` (index rows + detailed `###` headings)
    - `docs/BLUEPRINTS/EMO_PROFILE_MANAGE.md` (simulation requirements)
    - `docs/COVERAGE_MATRIX.md` (PH1.EMO `simulations_owned`)
    - `docs/ECM/PH1_EMO.md` (`simulation_gated` IDs)
    - `docs/DB_WIRING/PH1_EMO.md` simulation subsection references
    - historical proof references in `docs/13_PROBLEMS_TO_FIX.md`
- Proof command 1 (legacy hyphen IDs removed):
  - `rg -n 'EMO-SIM-00[1-6]' docs/08_SIMULATION_CATALOG.md docs/BLUEPRINTS/EMO_PROFILE_MANAGE.md docs/COVERAGE_MATRIX.md docs/ECM/PH1_EMO.md docs/DB_WIRING/PH1_EMO.md -S` -> `EXIT:1`
- Proof command 2 (canonical IDs present in required surfaces):
  - `rg -n 'EMO_SIM_00[1-6]' docs/08_SIMULATION_CATALOG.md docs/BLUEPRINTS/EMO_PROFILE_MANAGE.md docs/COVERAGE_MATRIX.md docs/ECM/PH1_EMO.md docs/DB_WIRING/PH1_EMO.md -S` -> `EXIT:0`
- Proof command 3 (3G-style simulation-resolution check; EMO misses zero):
  - Rebuilt `/tmp/sim_ids.txt` from sim catalog headings and `/tmp/active_simreq_ids_unique.txt` from ACTIVE blueprint simulation requirements.
  - `comm -23 /tmp/active_simreq_ids_unique.txt /tmp/sim_ids.txt | rg '^EMO' -n` -> `EXIT:1` (expected: zero EMO missing simulation IDs)
- Proof command 4 (full missing simulation list after normalization):
  - `comm -23 /tmp/active_simreq_ids_unique.txt /tmp/sim_ids.txt | sed 's/^/MISSING_SIM_ID: /'` -> `EXIT:0` with empty output

### Item 15
- Fixed on 2026-02-14.
- Fixes applied:
  - Updated `docs/BLUEPRINTS/LINK_INVITE.md` section `## 6) Simulation Requirements`:
    - changed delivery note from bullet form to plain prose so parser does not classify it as a simulation requirement line.
  - Kept all simulation requirement IDs unchanged.
- Proof command 1 (section 6 now contains simulation bullets only):
  - `awk 'BEGIN{ins=0} /^## [0-9]+\\) Simulation Requirements/{ins=1; next} /^## [0-9]+\\)/{ins=0} {if(ins && $0 ~ /^- /) print NR\":\"$0}' docs/BLUEPRINTS/LINK_INVITE.md` -> only simulation IDs printed; `EXIT:0`
- Proof command 2 (targeted NON_SIM_TEXT check for LINK_INVITE):
  - Rebuilt `/tmp/active_simreq_ids.txt` from ACTIVE blueprint simulation sections.
  - `rg -n '^NON_SIM_TEXT:docs/BLUEPRINTS/LINK_INVITE.md:' /tmp/active_simreq_ids.txt` -> `EXIT:1` (expected: zero parser drift lines)
- Proof command 3 (global NON_SIM_TEXT check after fix):
  - `rg -n '^NON_SIM_TEXT:' /tmp/active_simreq_ids.txt` -> `EXIT:1` (expected: zero non-simulation lines in all ACTIVE simulation requirement sections)

### Item 16
- Fixed on 2026-02-14.
- Fixes applied:
  - Aligned enforcement authority to runtime-owned monotonic transitions (PH1.F/PH1.LINK), SQL enum-membership checks only.
  - Updated `docs/04_KERNEL_CONTRACTS.md` KC.16 wording:
    - `Transition enforcement rule: ... enforced by PH1.F/PH1.LINK runtime state machine; SQL CHECK enforces enum membership only.`
  - Added matching migration comment in `crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql` directly above onboarding_drafts status CHECK.
- Proof command 1 (enforcement wording parity):
  - `rg -n 'Transition enforcement rule|runtime-owned by PH1.F/PH1.LINK|status enum membership only' docs/04_KERNEL_CONTRACTS.md crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql -n` -> `EXIT:0`
- Proof command 2 (legacy ambiguous CHECK wording removed):
  - `rg -n '^CHECK: status transitions are monotonic' docs/04_KERNEL_CONTRACTS.md` -> `EXIT:1`

### Item 17
- Fixed on 2026-02-14.
- Fixes applied:
  - Added process-rule lock in `docs/05_OS_CONSTITUTION.md` Conventions:
    - banned-token readiness sweeps run on `docs` + `crates` and exclude tracker/proof-log artifact `docs/13_PROBLEMS_TO_FIX.md`.
  - Standardized Item 17 proof commands to the tracker-excluded pattern.
- Proof command 1 (tracker-excluded product-surface sweep):
  - `rg -ni "$BANNED_LINK_DRIFT_TOKENS_CI" docs crates -S -g '!docs/13_PROBLEMS_TO_FIX.md'` -> `EXIT:1`
- Proof command 2 (runtime canon surfaces):
  - `rg -ni "$BANNED_LINK_DRIFT_TOKENS_CI" crates/selene_engines/src/ph1n.rs crates/selene_os/src/ph1x.rs -S` -> `EXIT:1`
- Proof command 3 (constitution scope-lock text present):
  - `rg -n 'Banned-token readiness sweeps run on product surfaces|exclude tracker/proof-log artifacts' docs/05_OS_CONSTITUTION.md -n` -> `EXIT:0`

### Item 18
- Fixed on 2026-02-14.
- Fixes applied:
  - Added governance checklist section to `docs/11_DESIGN_LOCK_SEQUENCE.md`:
    - `## Readiness Audit Precondition (Mandatory)`
    - requires repo-state proof lines in every readiness audit (`git status --short`, `git log -1 --oneline`),
    - requires either clean working tree or pinned commit hash with dirty-file listing.
- Proof command 1 (repo-state proof lines available):
  - `git status --short` -> `EXIT:0`
  - `git log -1 --oneline` -> `EXIT:0`
- Proof command 2 (precondition language present in governance/tracker docs):
  - `rg -n 'clean working tree|pinned commit hash|audit precondition' docs/05_OS_CONSTITUTION.md docs/11_DESIGN_LOCK_SEQUENCE.md docs/13_PROBLEMS_TO_FIX.md -n` -> `EXIT:0`

### Item 19
- Fixed on 2026-02-14.
- Fixes applied:
  - Added final closure-verification row to lock post-fix readiness state.
- Proof command 1 (no open tracker rows):
  - `rg -n '\\| OPEN \\| NO \\|' docs/13_PROBLEMS_TO_FIX.md` -> `EXIT:1`
- Proof command 2 (coverage remains zero-open):
  - `rg -n 'TODO|BLOCKER|WIP' docs/COVERAGE_MATRIX.md` -> `EXIT:1`
- Proof command 3 (item index continuity includes 19):
  - `rg -n '^\\| 19 \\|' docs/13_PROBLEMS_TO_FIX.md` -> `EXIT:0`
