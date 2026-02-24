# HEALTH Engine Display Plan (App Left Menu First)

## Scope Lock (V1)

V1 scope is display-only reporting.

Hard boundaries:
- PH1.HEALTH reports what happened.
- PH1.HEALTH does not execute remediation.
- PH1.HEALTH does not grant authority.
- PH1.HEALTH does not run simulations.

Action execution remains in owner engines (for example `PH1.VOICE.ID`, `PH1.DELIVERY`, `PH1.BUILDER`, `PH1.OS`), and PH1.HEALTH only displays those outcomes.

## 1) Purpose

Define the Health Engine experience in the Selene app so you can always see:
- all bads/sads/problems/errors,
- what Selene did about each issue,
- what is still unresolved,
- and when escalation to boss is sent.

Hard rule:
- no silent failures.
- every issue must end in `RESOLVED` or `ESCALATED`.

---

## 2) App Left Menu (Required Order)

The app left menu must place Health first:

1. `Health`
2. `Inbox`
3. `Work Orders`
4. `Learning`
5. `Governance`
6. `Settings`

`Health` opens by default when user taps it.

Health open behavior (locked):
- first view is `Health Checks` list.
- user selects one health check from the list.
- selected check opens its event feed and issue details.
- do not use top-of-screen category chips for health-check selection.

---

## 3) Health Screen Layout (Desktop + Mobile)

Desktop UI contract:
- must use ChatGPT-style shell layout.
- same left sidebar + main content pattern.
- no alternate custom desktop layout.
- must use ChatGPT-style colors/theme behavior (no custom palette).
- app logo in sidebar header is `A Digital Girl that talks`.

Desktop structure:
- Left sidebar: navigation (`Health` first).
- Main center: health-check list or selected-check issue/event list.
- Right detail panel: selected issue timeline.

Mobile structure:
- same content and order, stacked.
- issue detail opens as full-screen panel.

### 3A) Shared Chat Surface Contract (Voice + Text, ChatGPT-Style)

This app must also support a ChatGPT-style conversation timeline for normal Selene chat, in the same visual language as desktop/mobile shell.

Hard behavior:
- when user speaks, Selene writes what user said into the transcript.
- when Selene speaks, Selene also writes the same reply text into the transcript.
- voice and text must stay in one timeline (no separate hidden voice-only stream).

Turn display contract:
- user spoken turn appears as `USER` message with timestamp and final transcript text.
- Selene spoken turn appears as `SELENE` message with timestamp and final reply text.
- partial transcripts may be shown during capture/playback, but final text must replace partial text deterministically.

Source-of-truth lock:
- displayed Selene text must come from canonical `PH1.WRITE` output.
- spoken Selene audio (`PH1.TTS`) must match the same `PH1.WRITE` text to prevent drift.
- user speech transcript (`PH1.C`) must be the displayed user text after finalization.

Fail-closed UX:
- if user speech is low confidence, show uncertain state and ask for clarification.
- do not silently invent words.
- if TTS plays but text cannot render, mark turn degraded and retry text render path.

### A) Health Checks List (first view after click on Health)
- list items are selectable rows, not top chips.
- minimum list entries:
  - `Voice`
  - `Wake`
  - `Sync`
  - `STT`
  - `TTS`
  - `Delivery`
  - `Builder`
  - `Memory`
- each row shows:
  - current status badge (`HEALTHY|AT_RISK|CRITICAL`)
  - open issue count
  - last event timestamp

### B) Top Summary Strip (for selected health check)
- `Open Issues`
- `Critical`
- `Auto-Resolved (24h)`
- `Escalated (24h)`
- `MTTR` (mean time to resolve)

### C) Primary Queue (center)
Table columns:
- `severity` (`CRITICAL|HIGH|MEDIUM|LOW`)
- `issue_type`
- `engine_owner`
- `first_seen`
- `last_update`
- `status`
- `resolution_state`

### D) Issue Detail Panel (right or full-screen on mobile)
Must show:
- Problem summary (plain English)
- Deterministic reason code(s)
- Evidence refs (`correlation_id`, `turn_id`, `job_id`, `artifact_ref`)
- Action timeline (every attempt + result)
- Current blocker (if unresolved)
- Next action deadline

### E) Recent Events Feed
- shows most recent events first.
- every row must include date+time.
- infinite/virtual scroll for long history.

### F) Filters
- `Open only`
- `Critical only`
- `Voice only`
- `Delivery only`
- `Builder only`
- `Escalated only`
- `Date period` (`from` + `to`)
- quick presets: `24h`, `7d`, `30d`, `Custom`

---

## 4) Issue Lifecycle (Deterministic)

Canonical state machine:

`NEW -> TRIAGED -> ACTION_RUNNING -> VERIFYING -> RESOLVED`

Unresolved branch:

`NEW/TRIAGED/ACTION_RUNNING/VERIFYING -> ESCALATION_REQUIRED -> BCAST_SENT -> WAITING_ACK -> ESCALATED_OPEN`

Closure branch from escalation:

`ESCALATED_OPEN -> ACTION_RUNNING -> VERIFYING -> RESOLVED`

Hard rules:
- issue must always have one owner engine.
- no issue may remain without `next_action_at`.
- if `next_action_at` is breached, escalate.

---

## 5) What PH1.HEALTH Does With Reports (Display-Only)

For every incoming report:

1. Normalize into `health_issue`.
2. Deduplicate by deterministic key.
3. Assign owner + severity.
4. Link owner-engine actions that already happened.
5. Compute current state (`RESOLVED|UNRESOLVED|ESCALATED`).
6. Expose dashboard + timeline for user.

Input sources (minimum):
- `PH1.VOICE.ID` identity failures/drift/spoof/low quality.
- `PH1.C` transcript rejects/retries.
- `PH1.TTS` render failures/timeouts.
- `PH1.DELIVERY` send failures/dead letters/provider outages.
- `PH1.FEEDBACK` correction and quality signals.
- `PH1.LEARN` artifact build/validation failures.
- `PH1.BUILDER` gate/refusal/rollback failures.
- device artifact sync worker retry/dead-letter/replay-due failures.

---

## 6) Action Trace Mapping (No Execution in PH1.HEALTH)

### Voice-ID quality drift
- owner: `PH1.VOICE.ID`
- display: latest identity failures, feedback/learn emissions, threshold pack version applied (if any).

### Delivery provider outage
- owner: `PH1.DELIVERY`
- display: provider status changes, retry/failover attempts, current send health.

### Sync queue dead letters
- owner: `PH1.OS` (sync worker path)
- display: replay queue counts, dead-letter counts, builder trigger status.

### Builder rollout regression
- owner: `PH1.BUILDER`
- display: rollout gate results, rollback/refusal events, current build status.

Every traced action must show:
- `attempt_no`
- `action_id`
- `started_at`
- `result`
- `reason_code`

---

## 7) Boss Escalation (BCAST)

Escalation trigger source:
- owned by `PH1.OS` + `PH1.BCAST` policy.
- PH1.HEALTH displays escalation status and proof ids.

Escalate via `PH1.BCAST` when:
- severity is `CRITICAL`, or
- retry budget exhausted, or
- unresolved timeout breached.

Escalation message template:

`Hey Boss, Houston problem detected. Issue {issue_id} is unresolved in {engine_owner}. Severity {severity}. Last action {last_action}. Need Superman/Codex assist.`

Escalation policy:
- first alert immediately on trigger.
- reminder every 15 minutes while unresolved.
- stop reminders only after `RESOLVED` or explicit acknowledge + defer.

---

## 8) Health Report Contract (Display Model)

Each displayed issue must include:
- `issue_id`
- `status`
- `severity`
- `engine_owner`
- `problem_type`
- `reason_codes[]`
- `first_seen_at`
- `last_seen_at`
- `attempt_count`
- `last_action`
- `last_action_result`
- `resolved_at` (nullable)
- `escalated` (bool)
- `bcast_id` (nullable)
- `ack_state`

---

## 9) Non-Negotiable UX Rules

- Never hide unresolved issues.
- Never show "green" if critical unresolved exists.
- Never allow issue row with blank owner.
- Every issue row must show "what Selene did".
- Every escalated issue must show BCAST proof id.

---

## 10) Build Phases

### Phase H1: Display + Tracking
- app `Health` screen
- issue model + list/detail
- action timeline panel

### Phase H2: Source Feed Mapping
- map owner-engine action events into one timeline
- unresolved timers + SLA badges
- state projection (`RESOLVED|UNRESOLVED|ESCALATED`)

### Phase H3: Boss Escalation Visibility
- consume `PH1.BCAST` escalation outcomes
- show alert/reminder cadence state
- show acknowledgement state

### Phase H4: Quality Gates
- no-unowned issues
- no-stale unresolved past SLA
- deterministic closure proof

## 11) Keep It Simple (V1)

Only include:
- summary counters,
- open issue list,
- issue detail timeline,
- unresolved/escalated filters,
- BCAST escalation proof link.

Do not include in V1:
- custom charts,
- predictive scoring,
- remediation controls,
- manual run buttons.

## 11A) Simple Extras (Optional, Still Minimal)

- Issue search by `issue_id` or `engine_owner`.
- One-click copy for `correlation_id` / `turn_id` / `bcast_id`.
- `Last refreshed at` timestamp with manual refresh button.
- SLA breach badge (`ON_TIME|AT_RISK|BREACHED`).
- Desktop keyboard shortcuts (`j/k` list navigation, `enter` open detail).

## 12) Acceptance Tests

- `AT-HEALTH-01`: new issue appears in Health within one cycle.
- `AT-HEALTH-02`: issue detail shows reason codes + evidence refs.
- `AT-HEALTH-03`: action timeline records all attempted remediations.
- `AT-HEALTH-04`: unresolved timeout triggers BCAST escalation.
- `AT-HEALTH-05`: escalated issue shows `bcast_id` and reminder cadence.
- `AT-HEALTH-06`: resolved issue is removed from open queue and kept in history.
- `AT-HEALTH-07`: critical unresolved blocks "all green" status.

## 13) UI Execution Tracker (Build Control)

Status legend:
- `TODO`: not started
- `IN_PROGRESS`: active build
- `DONE`: completed + verified
- `BLOCKED`: waiting on dependency

Update rule:
- Every `DONE` item must include commit id and test/proof command.
- Every `DONE` item must also be logged in `docs/03_BUILD_LEDGER.md`.

| UI ID | Build Item | Runtime/Contract Dependency | Status | Evidence |
| --- | --- | --- | --- | --- |
| HUI-01 | App shell with ChatGPT-style layout, left menu, `Health` first, desktop+mobile parity | app shell + nav | DONE | `cargo test -p selene_adapter at_adapter_23_hui01_shell_nav_health_first_with_mobile_hooks -- --nocapture ; cargo test -p selene_adapter -- --nocapture` |
| HUI-02 | `Health` first-click opens `Health Checks` list (row selector, not top chips) | `PH1.HEALTH` display model | DONE | `cargo test -p selene_adapter at_adapter_24_hui02_health_landing_uses_check_row_selector -- --nocapture ; cargo test -p selene_adapter -- --nocapture` |
| HUI-03 | Health checks row cards (`Voice`, `Wake`, `Sync`, `STT`, `TTS`, `Delivery`, `Builder`, `Memory`) with status + counts + last event | `HEALTH_SNAPSHOT_READ` | DONE | `cargo test -p selene_adapter at_adapter_25_hui03_health_cards_show_status_counts_and_last_event -- --nocapture ; cargo test -p selene_adapter -- --nocapture` |
| HUI-04 | Selected health check summary strip (`Open`, `Critical`, `Auto-Resolved 24h`, `Escalated 24h`, `MTTR`) | `HEALTH_SNAPSHOT_READ` | DONE | `cargo test -p selene_adapter at_adapter_26_hui04_summary_strip_maps_runtime_summary_fields -- --nocapture ; cargo test -p selene_adapter -- --nocapture` |
| HUI-05 | Primary queue table with severity/type/owner/timestamps/status/resolution state | `HEALTH_SNAPSHOT_READ` + table projection | DONE | `cargo test -p selene_adapter at_adapter_27_hui05_primary_queue_table_columns_and_projection_locked -- --nocapture ; cargo test -p selene_adapter -- --nocapture` |
| HUI-06 | Issue detail panel timeline with reason codes + evidence refs + attempt history + blocker + deadline | `HEALTH_ISSUE_TIMELINE_READ` | DONE | `cargo test -p selene_adapter at_adapter_28_hui06_detail_timeline_shows_reason_evidence_blocker_deadline -- --nocapture ; cargo test -p selene_adapter -- --nocapture` |
| HUI-07 | Unresolved/escalated visibility with BCAST proof (`bcast_id`, ack/reminder state) | `HEALTH_UNRESOLVED_SUMMARY_READ` + `HEALTH_REPORT_QUERY_READ` + `PH1.BCAST` refs | DONE | `cargo test -p selene_engines ph1health::tests::at_health_03_issue_timeline_exposes_bcast_reference_when_escalated -- --nocapture ; cargo test -p selene_adapter -- --nocapture` |
| HUI-08 | Filters and search (`open`, `critical`, `engine`, `escalated`, date range, presets 24h/7d/30d/custom) | query params + snapshot/timeline reads | DONE | `cargo test -p selene_adapter at_adapter_17_health_detail_filters_open_critical_escalated -- --nocapture ; cargo test -p selene_adapter at_adapter_19_health_detail_filter_rejects_invalid_date_range -- --nocapture ; cargo test -p selene_adapter -- --nocapture` |
| HUI-09 | Infinite scroll / pagination for recent events feed | timeline paging | DONE | `cargo test -p selene_adapter at_adapter_18_health_detail_timeline_cursor_paging_is_deterministic -- --nocapture ; cargo test -p selene_adapter -- --nocapture` |
| HUI-10 | Unknown/error/empty/loading states with fail-closed messaging (no fake green) | UI state management | DONE | `cargo test -p selene_adapter at_adapter_20_fail_closed_ui_state_markers_are_present -- --nocapture ; cargo test -p selene_adapter -- --nocapture` |
| HUI-11 | iOS/Android/Desktop view parity checks for same fields/order/contracts | QA parity checks | DONE | `cargo test -p selene_adapter at_adapter_21_ios_android_desktop_contract_parity_is_locked -- --nocapture ; cargo test -p selene_adapter -- --nocapture` |
| HUI-12 | Final acceptance sweep (`AT-HEALTH-01..07`) | runtime + UI integration | DONE | `cargo test -p selene_kernel_contracts ph1health -- --nocapture ; cargo test -p selene_engines ph1health -- --nocapture ; cargo test -p selene_os ph1health -- --nocapture ; cargo test -p selene_adapter -- --nocapture` |
| HUI-13 | ChatGPT-style conversation timeline shell for normal Selene chat (desktop + mobile parity) | app shell + transcript renderer | DONE | `cargo test -p selene_adapter at_adapter_29_hui13_chat_shell_transcript_and_wave_layout_present -- --nocapture ; cargo test -p selene_adapter -- --nocapture` |
| HUI-14 | User voice-in -> transcript row mapping (`PH1.C` finalized text) | STT final transcript contract | DONE | `cargo test -p selene_adapter at_adapter_12_ui_chat_transcript_maps_user_and_selene_final_rows -- --nocapture` |
| HUI-15 | Selene voice-out -> transcript row mapping (`PH1.WRITE` text + `PH1.TTS` playback parity) | WRITE/TTS parity lock | DONE | `cargo test -p selene_adapter at_adapter_12_ui_chat_transcript_maps_user_and_selene_final_rows -- --nocapture` |
| HUI-16 | Partial->final transcript replacement logic (no duplicate ghost lines) | streaming/finalization state | DONE | `cargo test -p selene_adapter at_adapter_13_partial_replaced_by_final_without_ghost_line -- --nocapture` |
| HUI-17 | Voice/text parity acceptance checks for both directions | end-to-end chat turn tests | DONE | `cargo test -p selene_adapter at_adapter_22_voice_text_bidirectional_transcript_parity_is_locked -- --nocapture ; cargo test -p selene_adapter -- --nocapture` |

## 14) Strict Build Order (Do Not Reorder)

1. `HUI-01`: shell + left nav.
2. `HUI-02`: health checks first-click list behavior.
3. `HUI-03` + `HUI-04`: snapshot cards + summary strip.
4. `HUI-05`: primary queue table.
5. `HUI-06`: issue detail panel timeline.
6. `HUI-07`: unresolved/escalated + BCAST proof.
7. `HUI-08` + `HUI-09`: filters/search/date range + scroll/paging.
8. `HUI-10` + `HUI-11`: robustness states + platform parity.
9. `HUI-12`: acceptance sweep and ledger evidence.
10. `HUI-13`: ChatGPT-style chat timeline shell.
11. `HUI-14` + `HUI-15`: voice-in/out transcript wiring.
12. `HUI-16`: partial-to-final replacement behavior.
13. `HUI-17`: voice/text parity acceptance sweep.

## 15) Reporting Format Per Completed UI Item

For each completed item (`HUI-xx`), record:
- `status`: `DONE`
- `commit`: short sha
- `files`: touched UI/runtime paths
- `proof`: command(s) run
- `result`: `PASS|FAIL`
- `notes`: any follow-up work

## 16) Health Report Query Expansion (Locked 2026-02-23)

Purpose:
- Add deterministic report-query behavior so voice/text commands can request configurable Health reports and display them on desktop/phone with paging continuity.

Locked user decisions:
- Multi-company output format: `tenant-by-tenant rows` (not one collapsed aggregate row).
- Display target memory: once the user explicitly chooses `desktop` or `phone`, remember it as per-user default until changed.
- Page size policy: no fixed global page size; use auto-fit based on active screen/form factor.

Execution insertion rule:
- This expansion is mandatory before closing `HUI-07`, `HUI-08`, and `HUI-09`.
- Do not mark those UI items `DONE` until this section is complete.

### 16A) Contract Change Set (Must Land Before UI Closure)

1. Kernel contracts (`crates/selene_kernel_contracts/src/ph1health.rs`)
- Add deterministic report-query request/response contracts:
  - `HealthReportQueryRequest`
  - `HealthReportQueryResponse`
- Add request fields:
  - `report_kind`
  - `time_range` (`from_utc`, `to_utc`)
  - `engine_owner_filter`
  - `company_scope` (`TENANT_ONLY | CROSS_TENANT_TENANT_ROWS`)
  - `company_ids` (optional)
  - `country_codes` (optional)
  - `escalated_only`
  - `unresolved_only`
  - `display_target` (`DESKTOP | PHONE`, optional to allow one clarify)
  - `page_action` (`FIRST | NEXT | PREV | REFRESH`)
  - `page_cursor` (optional)
  - `report_context_id` (for "same report but ...")
- Add response fields:
  - `report_context_id`
  - `report_revision`
  - `normalized_query`
  - `rows` (tenant-row scoped)
  - `paging` (`has_next`, `has_prev`, `next_cursor`, `prev_cursor`)
  - `display_target_applied`
  - `requires_clarification` (nullable)
- Add fail-closed reason codes:
  - `PH1_HEALTH_DISPLAY_TARGET_REQUIRED`
  - `PH1_HEALTH_DATE_RANGE_INVALID`
  - `PH1_HEALTH_COUNTRY_FILTER_INVALID`
  - `PH1_HEALTH_CROSS_TENANT_UNAUTHORIZED`
  - `PH1_HEALTH_REPORT_CONTEXT_NOT_FOUND`
  - `PH1_HEALTH_PAGE_CURSOR_INVALID`

2. Engine runtime (`crates/selene_engines/src/ph1health.rs`)
- Add deterministic query projection and filtering for:
  - time range
  - engine owner
  - escalated/unresolved
  - country code
  - tenant-by-tenant row shaping
- Add cursor-based paging that remains deterministic under same input snapshot.

3. OS orchestration clarification behavior (`crates/selene_os/src/ph1x.rs` + ingress wiring)
- If `display_target` is missing, ask exactly one clarify question:
  - "Where do you want this report displayed: desktop or phone?"
- Persist chosen display target in per-user profile/defaults and reuse on later report requests.

4. Adapter/API wiring (`crates/selene_adapter/src/lib.rs`, `crates/selene_adapter/src/bin/http_adapter.rs`, `crates/selene_adapter/src/bin/grpc_adapter.rs`)
- Add request/response transport fields for report query, pagination, display target, and report-context continuation.
- Support follow-up query patch behavior (for "same report for ...").

5. ECM + DB wiring docs
- `docs/ECM/PH1_HEALTH.md`:
  - add capability `HEALTH_REPORT_QUERY_READ`
  - encode cross-tenant permission gate and tenant-row output shape.
- `docs/DB_WIRING/PH1_HEALTH.md`:
  - keep `display-only` + `writes: NONE`
  - add report-query read projection inputs and cursor semantics.

6. Health UI plan alignment
- `docs/36_HEALTH_ENGINE_DISPLAY_PLAN.md`:
  - keep HUI order unchanged
  - ensure `HUI-07/08/09` closure evidence references new report-query contract tests.

### 16B) Strict Implementation Checklist (Do Not Reorder)

1. Lock policy decision in docs: cross-tenant output is `tenant-by-tenant rows`.
2. Add kernel contract types + validators + reason codes for report query.
3. Add kernel contract tests for date-range, cursor, country filter, unauthorized cross-tenant cases.
4. Add engine runtime report-query path (filtering + tenant-row output + deterministic paging).
5. Add engine runtime tests for:
- missed STT report in date range,
- unresolved/escalated with `bcast_id`,
- "same report but country=CN",
- next/prev paging continuity.
6. Add OS clarify flow for missing display target (one question only).
7. Add per-user display-target memory behavior.
8. Add adapter/http/grpc request-response wiring for report query + paging + report context.
9. Add UI bindings for desktop/phone target presentation and scroll/page controls.
10. Run acceptance suite for `HUI-07`, `HUI-08`, `HUI-09` + report query e2e.
11. Update `docs/03_BUILD_LEDGER.md` with proof entries for each closed step.
12. Mark `HUI-07`, `HUI-08`, `HUI-09` as `DONE` only after all above pass.

### 16C) Execution Tracker (Report Query Expansion)

Status legend:
- `TODO`
- `IN_PROGRESS`
- `DONE`
- `BLOCKED`

| RPT ID | Work Item | Owner | Status | Evidence |
| --- | --- | --- | --- | --- |
| RPT-01 | Contract add: request/response/query enums + reason codes | Contracts | DONE | `cargo test -p selene_kernel_contracts ph1health -- --nocapture` |
| RPT-02 | Runtime add: deterministic filters + tenant-row output + cursor paging | PH1.HEALTH Runtime | DONE | `cargo test -p selene_engines ph1health -- --nocapture` |
| RPT-03 | Clarify + remembered display target (`desktop|phone`) | PH1.X + Adapter | DONE | `cargo test -p selene_os ph1x::tests::at_x_report_display_target_uses_explicit_then_memory_then_clarify -- --nocapture ; cargo test -p selene_adapter at_adapter_15_report_query_clarify_then_remember_display_target -- --nocapture` |
| RPT-04 | API transport wiring (HTTP/gRPC + adapter models) | Adapter | DONE | `cargo test -p selene_adapter -- --nocapture` |
| RPT-05 | UI report rendering + paging controls (desktop/phone auto-fit) | App UI | DONE | `cargo test -p selene_adapter -- --nocapture` |
| RPT-06 | Acceptance + ledger + HUI status closure (`HUI-07/08/09`) | QA + Runtime | DONE | `cargo test -p selene_kernel_contracts ph1health -- --nocapture ; cargo test -p selene_engines ph1health -- --nocapture ; cargo test -p selene_os ph1health -- --nocapture ; cargo test -p selene_os ph1x::tests::at_x_report_display_target_uses_explicit_then_memory_then_clarify -- --nocapture ; cargo test -p selene_adapter at_health_10_display_target_clarify_then_memory_reuse -- --nocapture ; cargo test -p selene_adapter at_health_11_follow_up_report_patch_reuses_context -- --nocapture ; cargo test -p selene_adapter at_health_12_voice_wave_degraded_marker_is_wired -- --nocapture ; cargo test -p selene_adapter -- --nocapture` |

## 17) Live Resolution Proof + Voice-First UX Lock (Locked 2026-02-24)

This section locks the new product requirements in one place before code execution.

### 17A) "100% Resolved" Acceptance Rule

Core rule:
- An issue is `100% RESOLVED` only when live production evidence proves the same issue is no longer recurring.

Continuous verification:
- After any fix is deployed, Selene must keep monitoring the same issue fingerprint used to detect the issue.
- Fingerprint examples: event signature, error code pattern, anomaly marker set, deterministic reason-code cluster.

Acceptance gate:
- If follow-up monitoring still matches the same fingerprint, the issue remains open.
- A fix may not be accepted while recurrence evidence exists.

Closure gate:
- Selene may close only when verification evidence shows the fix held and recurrence stopped for the defined verification window.

### 17B) Failure-to-Fix Escalation Rule

Escalation trigger:
- If Selene cannot fix the issue, or cannot prove fix success through live verification, escalate immediately.

Minimum escalation payload (required fields):
- `issue_id`
- `impact_summary`
- `attempted_fix_actions[]`
- `current_monitoring_evidence`
- `unresolved_reason_exact`
- `bcast_id` when escalation is dispatched

### 17C) Primary Screen Experience (Voice-First)

Landing behavior:
- On open, Selene enters listen-ready mode and prompts for user intent.
- Prompt wording is not fixed text. Selene must generate natural wording (NLP+LLM), not robotic static phrasing.

Rule for docs and build specs:
- This natural-language behavior is global OS policy; it does not need to be repeated in every feature section.
- Only compliance-critical wording (for example legal/policy refusal text) may be fixed.

Voice-driven report retrieval:
- User can request any supported report by voice or text.
- Selene must resolve query deterministically and present report without manual navigation.

### 17D) Report Presentation + Rapid Switching

Professional output standard:
- Every report must render with professional structure:
  - clear title/header
  - clearly labeled filters
  - deterministic column/field order
  - clean spacing and bullet grouping for summary blocks

Rapid switching:
- "give me another report" must replace the current report view with the new report in the same display target.
- No manual close/back clicks required.
- Follow-up commands like "same report for all customers in China" must reuse `report_context_id` and patch only changed filters.

### 17E) Voice-Wave UI Requirement

- While Selene is speaking, the UI must show real-time animated voice waves.
- Wave amplitude and cadence must track voice playback state in real time.
- If audio is active but wave sync fails, UI must show degraded state (never fake inactive).

### 17F) ChatGPT-Parity Shell Requirement

- App layout must stay ChatGPT-style:
  - side menu with traditional navigation items,
  - center conversation/report surface,
  - input box in center area,
  - Selene voice waves displayed above input area.

- Report view behavior:
  - user can scroll report,
  - user can request paging by command (example: "next page"),
  - page size follows active screen auto-fit rules.

### 17G) Execution Order Impact

- Section 17 is mandatory input to `RPT-01..RPT-05` implementation.
- Do not close `RPT-06` until Section 17 acceptance checks are added and passing.
- `RPT-06` closure gate is satisfied (2026-02-24) with `AT-HEALTH-08..12` added and passing.

### 17H) Added Acceptance Checks (Section 17)

- `AT-HEALTH-08`: issue is not closed when recurrence fingerprint still appears post-fix.
- `AT-HEALTH-09`: unresolved/unverified issue escalation payload includes all minimum required fields.
- `AT-HEALTH-10`: missing display target triggers one clarify question, then remembered default is reused.
- `AT-HEALTH-11`: report follow-up patch (`same report but ...`) reuses context and replaces current report view.
- `AT-HEALTH-12`: voice wave animates during Selene speech and enters degraded marker on sync failure.

Proof mapping:
- `AT-HEALTH-08`: `cargo test -p selene_engines ph1health::tests::at_health_09_recurrence_true_post_fix_keeps_issue_unresolved -- --nocapture`
- `AT-HEALTH-09`: `cargo test -p selene_engines ph1health::tests::at_health_10_escalated_issue_requires_minimum_payload -- --nocapture ; cargo test -p selene_kernel_contracts ph1health::tests::at_health_contract_09_escalated_event_requires_minimum_payload_fields -- --nocapture`
- `AT-HEALTH-10`: `cargo test -p selene_os ph1x::tests::at_x_report_display_target_uses_explicit_then_memory_then_clarify -- --nocapture ; cargo test -p selene_adapter at_health_10_display_target_clarify_then_memory_reuse -- --nocapture`
- `AT-HEALTH-11`: `cargo test -p selene_adapter at_health_11_follow_up_report_patch_reuses_context -- --nocapture`
- `AT-HEALTH-12`: `cargo test -p selene_adapter at_health_12_voice_wave_degraded_marker_is_wired -- --nocapture`
