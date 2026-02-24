# PH1_HEALTH DB Wiring (Design v1, Display-Only)

## A) Engine Header
- engine_id: PH1.HEALTH
- layer: Observability/Control
- authority: Non-Authoritative
- role: consolidated health incident reporting and timeline display
- placement: ENTERPRISE_SUPPORT (outside in-turn execution)

## B) Ownership
- Tables owned: NONE (v1 display-only)
- Reads:
  - audit/event rows from existing engine outputs (`PH1.VOICE.ID`, `PH1.C`, `PH1.TTS`, `PH1.DELIVERY`, `PH1.FEEDBACK`, `PH1.LEARN`, `PH1.BUILDER`, sync-worker outcomes)
  - `PH1.BCAST` escalation references for unresolved issue visibility
  - report-query inputs (time range, owner filter, tenant/country filters, paging cursor, report context id)
- Writes: NONE (v1)

## C) Hard Boundaries
- Display-only reporting engine.
- Must not execute remediation actions.
- Must not call simulations.
- Must not grant or modify authority.
- Must not mutate owner-engine state.

## D) Wiring
- Invoked_by: Selene OS + app adapter health routes.
- Inputs_from: existing engine outcomes and audits only.
- Outputs_to:
  - app health list/snapshot,
  - issue detail timeline,
  - unresolved/escalated status projections,
  - report-query rows with deterministic paging cursors.
- Invocation_condition: on demand UI read + periodic refresh.
- Not allowed:
  - engine-to-engine execution calls,
  - side-effect commits,
  - override/approval decisions.

## G) Report-Query Projection Rules
- Capability `HEALTH_REPORT_QUERY_READ` remains read-only (`writes: NONE`).
- Cursor paging must be deterministic under the same snapshot and query inputs.
- Cross-tenant requests are allowed only when caller scope explicitly permits cross-tenant reads.
- Output shape for multi-company requests is tenant-by-tenant rows (no collapsed aggregate row in place of rows).

## H) Resolution-Proof and Escalation Evidence Fields
- PH1.HEALTH read projections must carry issue-resolution proof fields:
  - `issue_fingerprint`
  - `verification_window_start`
  - `verification_window_end`
  - `recurrence_observed`
  - optional recurrence evidence refs
- For unresolved/escalated entries, minimum escalation evidence fields must be present:
  - `impact_summary`
  - `attempted_fix_actions[]`
  - `current_monitoring_evidence`
  - `unresolved_reason_exact`
  - `bcast_id`/`ack_state` when escalated

## E) Desktop UI Contract
- Must follow ChatGPT-style desktop shell layout:
  - left sidebar navigation,
  - center list/snapshot content,
  - right detail panel.
- Health content may vary; shell layout may not.

## F) Acceptance Tests
- AT-HEALTH-01: snapshot read is schema-valid + display-only (`crates/selene_engines/src/ph1health.rs`).
- AT-HEALTH-02: unresolved summary includes owner engine + latest reason code (`crates/selene_engines/src/ph1health.rs`).
- AT-HEALTH-03: issue timeline includes `bcast_id` proof reference when present (`crates/selene_engines/src/ph1health.rs`).
- AT-HEALTH-04: tenant scope mismatch fails closed (`crates/selene_engines/src/ph1health.rs`).
- AT-HEALTH-OS-01..04: OS read wiring enforces disabled gate + response-capability fail-closed behavior (`crates/selene_os/src/ph1health.rs`).
