# PH1.ACCESS AP Authoring Review Strict Fix Plan Packet (v1)

Last updated: 2026-02-15
Owner: Selene core design + runtime
Status: STEP6_COMPLETED_PENDING_STEP7

## 1) Purpose

This packet locks the AP authoring flow so JD can create APs (for example `AP_CLERK`, `AP_CEO`) with guided baseline suggestions and safe review controls, without breaking existing deterministic access behavior.

## 2) Frozen Design Truth (Step 1 lock)

1. APs are schema records, never hard-coded permission bundles.
2. Selene may suggest baseline rules using NLP + LLM + read-only market/tool evidence, but suggestions are non-authoritative until admin confirmation.
3. Selene must ask one explicit review-channel question:
   - "Should I send this to your phone/desktop for review, or do you want me to read it out loud?"
4. Admin can review each suggested rule with bounded actions:
   - agree
   - disagree
   - edit
   - delete
   - disable
   - add custom rule
5. Activation is simulation-gated and reason-coded; no silent policy changes.
6. Screen-facing output must use professional writing quality.
7. Runtime access outcome contract stays exactly: `ALLOW | DENY | ESCALATE`.

## 3) Engine Ownership for this Scope

1. `PH1.ACCESS.001_PH2.ACCESS.002` owns AP schema lifecycle and activation truth.
2. `PH1.NLP` and `PH1.D` assist with AP draft extraction/suggestions only (non-authoritative).
3. `PH1.E` provides read-only evidence/lookups for baseline guidance only.
4. `PH1.X` owns deterministic next-step conversation moves and review-channel branching.
5. `PH1.WRITE` formats professional screen text only; no authority decisions.
6. `PH1.J` and `PH1.F` keep append-only audit and schema storage truth.

## 4) Strict 8-Step Order

### Step 1: Docs lock

Lock this packet and build-plan pointer. No runtime edits in this step.

### Step 2: Contract lock

Add/lock AP authoring review objects (review channel choice, suggested-rule action payloads, authoring confirmation state) in kernel contracts.

### Step 3: Blueprint + simulation lock

Add/lock AP authoring review blueprint and simulation rows with explicit gates for review choice and rule action commits.

### Step 4: Runtime orchestration lock

Wire deterministic flow through `PH1.NLP/PH1.D/PH1.E/PH1.X/PH1.WRITE` into `PH1.ACCESS` without changing access gate semantics.

### Step 5: Storage/repo lock

Persist AP authoring draft, per-rule review decisions, and activation lineage with deterministic idempotency and tenant isolation.

### Step 6: Test closure

Add tests for:

1. phone/desktop review path
2. read-out-loud review path
3. rule-by-rule actions
4. fail-closed activation when review/gates are missing
5. professional-writing output presence in screen path

### Step 7: Drift sweep + acceptance proof

Run strict doc/contract/runtime drift checks for AP authoring surfaces.

### Step 8: Final proof + freeze checkpoint

From clean tree, run targeted suites + workspace tests + readiness audit, then commit freeze checkpoint.

## 5) Acceptance Checklist (must all be true)

1. AP authoring baseline suggestions are assist-only (non-authoritative).
2. Review channel prompt is explicit and deterministic (phone/desktop or read-out-loud).
3. Rule-by-rule actions are bounded and auditable.
4. Activation requires simulation + confirmation + approvals.
5. Screen output uses professional writing.
6. Access gate output contract remains `ALLOW|DENY|ESCALATE`.

## 6) Execution Record

- Step 1: COMPLETED (2026-02-15)
- Step 2: COMPLETED (2026-02-15)
- Step 3: COMPLETED (2026-02-15)
- Step 4: COMPLETED (2026-02-15)
- Step 5: COMPLETED (2026-02-15)
- Step 6: COMPLETED (2026-02-16)
- Step 7: PENDING
- Step 8: PENDING

Step 1 note:
- Added this packet as canonical AP authoring review scope (`docs/30_ACCESS_AP_AUTHORING_REVIEW_STRICT_FIX_PLAN_PACKET.md`).
- Updated `docs/02_BUILD_PLAN.md` "Next Strict Packet" pointer from Packet 29 to Packet 30.
- Logged Packet 29 Step 8 closure commit (`e9a0725`) in the strict checkpoint section.

Step 2 note:
- Locked AP authoring review contract objects in kernel docs + contract module:
  - `docs/04_KERNEL_CONTRACTS.md` (`KC.26.5` through `KC.26.8`)
  - `crates/selene_kernel_contracts/src/ph1access.rs`
- Added typed contract objects for Step-2 scope:
  - review channel choice (`AccessApReviewChannel`)
  - suggested-rule action payload (`AccessApRuleReviewActionPayload`)
  - authoring confirmation state (`AccessApAuthoringConfirmationState`)
  - authoring review state object (`AccessApAuthoringReviewState`)
- Added fail-closed validation tests for rule actions + review state.
- Step-2 proof:
  - `cargo test -p selene_kernel_contracts -- --nocapture` -> pass (46 tests)
  - `rg` anchors for KC.26.5..KC.26.8 + new `ph1access` objects -> pass

Step 3 note:
- Locked AP authoring review blueprint + simulation surfaces in docs:
  - `docs/BLUEPRINTS/ACCESS_SCHEMA_MANAGE.md`
  - `docs/08_SIMULATION_CATALOG.md`
  - `docs/COVERAGE_MATRIX.md`
- Blueprint lock additions:
  - explicit review-channel gate (`PHONE_DESKTOP | READ_OUT_LOUD`) before AP lifecycle writes
  - explicit rule-review confirmation gate for bounded actions (`AGREE | DISAGREE | EDIT | DELETE | DISABLE | ADD_CUSTOM_RULE`)
  - simulation requirements expanded with AP authoring review simulation IDs
- Simulation catalog lock additions:
  - `ACCESS_AP_AUTHORING_REVIEW_CHANNEL_COMMIT`
  - `ACCESS_AP_AUTHORING_RULE_ACTION_COMMIT`
  - `ACCESS_AP_AUTHORING_CONFIRM_COMMIT`
  - index rows + full simulation blocks added with bounded schemas and idempotency rules
- Coverage lock update:
  - PH1.ACCESS owned simulation list expanded with the three AP authoring review simulation IDs.
- Step-3 proof:
  - `rg` checks for new AP authoring simulation IDs in blueprint + simulation catalog + coverage matrix -> pass
  - readiness audit (`scripts/selene_design_readiness_audit.sh`) -> pass (no capability-id gaps, no missing simulation IDs, `BAD_ACTIVE_SIMREQ_NONE_FOUND:0`)

Step 4 note:
- Locked deterministic runtime orchestration for AP authoring review/channel surfaces across:
  - `crates/selene_kernel_contracts/src/ph1n.rs`
  - `crates/selene_engines/src/ph1n.rs`
  - `crates/selene_os/src/ph1x.rs`
  - `crates/selene_os/src/simulation_executor.rs`
  - `crates/selene_os/src/ph1explain.rs`
- Runtime lock additions:
  - `FieldKey` surfaces for explicit AP review flow:
    - `AccessReviewChannel`
    - `AccessRuleAction`
  - Deterministic extraction/clarify ordering in `PH1.NLP` for AP authoring:
    - explicit review-channel capture (`PHONE_DESKTOP | READ_OUT_LOUD`)
    - bounded rule-action capture (`AGREE | DISAGREE | EDIT | DELETE | DISABLE | ADD_CUSTOM_RULE`)
  - `PH1.X` clarification and confirmation text updated to include:
    - professional review wording
    - explicit review-channel + rule-action context
  - simulation executor fail-closed gates tightened for `AccessSchemaManage`:
    - review channel required for AP schema manage commits
    - rule-action required for activate path
    - payload required for create/update paths
- Step-4 proof:
  - `cargo test -p selene_kernel_contracts -- --nocapture` -> pass (46 tests)
  - `cargo test -p selene_engines -- --nocapture` -> pass (61 tests)
  - `cargo test -p selene_os at_sim_exec_ -- --nocapture` -> pass (21 tests)
  - `cargo test -p selene_os at_x_ -- --nocapture` -> pass (19 tests)
  - `cargo test -p selene_os -- --nocapture` -> pass (81 tests)

Step 5 note:
- Locked AP authoring review storage + repository surfaces in:
  - `crates/selene_storage/src/ph1f.rs`
  - `crates/selene_storage/src/repo.rs`
  - `crates/selene_storage/migrations/0016_access_ap_authoring_review_tables.sql`
  - `crates/selene_storage/tests/ph1_access_ph2_access/db_wiring.rs`
- Storage lock additions:
  - persisted authoring review ledger/current rows (review channel + confirmation state)
  - persisted per-rule review action ledger rows with bounded action payload validation
  - deterministic idempotency indexes for review-channel, confirmation, and rule-action commits
  - activation lineage capture on AP schema activation:
    - `activation_review_event_id`
    - `activation_rule_action_count`
    - `activation_rule_action_set_ref`
  - fail-closed activation when review state exists but is not `CONFIRMED_FOR_ACTIVATION`
- SQL parity additions:
  - new AP authoring review and rule-action tables
  - added activation-lineage columns on `access_ap_schemas_ledger`
  - added activation review lineage index
- Test closure for Step-5 surfaces in access DB wiring suite:
  - `at_access_db_17_ap_authoring_review_channel_persists_and_dedupes`
  - `at_access_db_18_ap_authoring_rule_action_requires_review_state_and_dedupes`
  - `at_access_db_19_ap_authoring_confirm_requires_rule_actions_and_blocks_after_decline`
  - `at_access_db_20_ap_activation_captures_authoring_lineage_when_confirmed`
  - `at_access_db_21_ap_activation_fails_closed_without_confirmed_review_state`
- Step-5 proof:
  - `cargo test -p selene_storage --test db_wiring_access_tables -- --nocapture` -> pass (21 tests)
  - `cargo test -p selene_os at_sim_exec_ -- --nocapture` -> pass (21 tests)

Step 6 note:
- Closed AP authoring review runtime test coverage in:
  - `crates/selene_os/src/simulation_executor.rs`
  - `crates/selene_os/src/ph1x.rs`
- Added/locked test coverage for Step-6 requirements:
  - phone/desktop review path:
    - existing `at_sim_exec_19_access_schema_manage_gate_allow_returns_gate_passed`
  - read-out-loud review path:
    - `at_sim_exec_23_access_schema_manage_read_out_loud_gate_allow_returns_gate_passed`
  - rule-by-rule actions:
    - `at_sim_exec_25_access_schema_manage_activate_rule_actions_bounded_and_validated`
  - fail-closed activation when required gates are missing:
    - existing `at_sim_exec_22_access_schema_manage_missing_review_channel_fails_closed`
    - `at_sim_exec_24_access_schema_manage_activate_missing_rule_action_fails_closed`
    - invalid rule-action fail-closed branch in `at_sim_exec_25_access_schema_manage_activate_rule_actions_bounded_and_validated`
  - professional-writing presence in screen path:
    - `at_x_access_schema_manage_confirm_uses_professional_screen_writing`
    - `at_x_access_schema_manage_missing_review_channel_asks_explicit_channel_question`
- Runtime validation hardening:
  - access schema activation path now validates `access_rule_action` against the bounded enum set in simulation dispatch.
- Step-6 proof:
  - `cargo test -p selene_os at_sim_exec_ -- --nocapture` -> pass (24 tests)
  - `cargo test -p selene_os at_x_ -- --nocapture` -> pass (21 tests)
  - `cargo test -p selene_storage --test db_wiring_access_tables -- --nocapture` -> pass (21 tests)
