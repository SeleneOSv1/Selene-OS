# H38 Section 06: First Canonical S06-10 Retention-Mode Later-Distinct-Key Overwrite Response-Truth Build Plan

## Objective
This is the first canonical H38 Section 06 build plan after the implemented H37 same-idempotency-key replay response-truth proof slice.

The next exact target remains `S06-10`.

## Current Repo Truth
`S06-10` remains `PARTIAL`, `S06-12` remains `PARTIAL`, and `S06-19` remains `NOT_EXPLICIT`.

Current repo truth already proves:
- core architecture already requires the cloud runtime to return the same deterministic result / previously computed result for duplicate idempotency identity
- H35 remains published as the first canonical PH1.M real-runtime `RememberEverything` `run_turn_and_persist(...)` retention-mode proof slice
- H36 already proves later distinct-key overwrite wins in persisted truth on the wrapper path
- H37 already proves same-idempotency-key retry replays original response truth on the wrapper path
- current engine `retention_mode_set(...)` still returns request-time response truth using the latest request mode and `req.now`
- current wrapper path now rebuilds retention-mode response truth from the persisted repo row after persistence through `replay_retention_mode_response_truth_from_repo(...)`
- current generic `Ph1MRepo` carrier already exposes retention-preference row readback on the wrapper path

## Exact Seam
The exact seam is the canonical PH1.M real-runtime retention-mode later-distinct-idempotency-key overwrite latest-response-truth path on `run_turn_and_persist(...)`.

H38 will publish the first canonical PH1.M real-runtime proof that a later distinct idempotency key overwrite returns the latest overwrite response truth rather than stale prior response truth.

The exact candidate canonical proof for the next implementation is `at_m_38_real_runtime_run_turn_and_persist_retention_mode_later_distinct_key_overwrite_returns_latest_response_truth`.

## Implementation Boundary
The next implementation should stay bounded to the live PH1.M wrapper path and existing repo carrier:
- `run_turn_and_persist(...)`
- `persist_memory_forwarded_outcome(...)`
- `replay_retention_mode_response_truth_from_repo(...)`
- retention preference readback through the live `Ph1MRepo` carrier

Current repo truth suggests the later implementation may be proof-first and may require zero production-logic edits.

## Proof Plan
H38 should canonically prove that:
- an initial retention-mode request returns expected wrapper-path response truth
- a later distinct idempotency key overwrite returns the latest overwrite response truth rather than stale prior response truth
- H35 remains adjacent canonical wrapper-path single-commit coverage
- H36 remains adjacent canonical persisted no-op and later-overwrite coverage
- H37 remains adjacent canonical same-key replay response-truth coverage

`S06-12` is not selected because `S06-10` still exposes the smaller live seam.

`S06-19` is not selected because it remains `NOT_EXPLICIT`.

## Out Of Scope
No code is changed in this run; this run only publishes the next active target.

This H38 plan does not claim or implement:
- lifecycle-worker closure
- broader retention-policy closure beyond the later-distinct-key overwrite latest-response-truth seam
- broader automatic temperature-transition closure
- broader `S06-12` trust closure
- any `S06-19` decay-model closure

`S06-10` remains `PARTIAL`.
`S06-12` remains `PARTIAL`.
`S06-19` remains `NOT_EXPLICIT`.
