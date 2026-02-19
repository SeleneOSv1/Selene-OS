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
