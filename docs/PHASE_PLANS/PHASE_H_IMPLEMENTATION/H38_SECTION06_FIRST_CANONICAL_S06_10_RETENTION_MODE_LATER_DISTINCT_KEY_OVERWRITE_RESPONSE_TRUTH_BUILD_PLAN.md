# H38 Section 06: First Canonical S06-10 Retention-Mode Later-Distinct-Key Overwrite Response-Truth Build Plan

## Objective
This H38 slice is now the first canonical PH1.M real-runtime retention-mode later-distinct-key overwrite latest-response-truth proof slice inside `S06-10`.

## Current Repo Truth
`S06-10` remains `PARTIAL`, `S06-12` remains `PARTIAL`, and `S06-19` remains `NOT_EXPLICIT`.

Current repo truth before this run already proved:
- core architecture already required the cloud runtime to return the same deterministic result / previously computed result for duplicate idempotency identity
- H35 remains published as the first canonical PH1.M real-runtime `RememberEverything` `run_turn_and_persist(...)` retention-mode proof slice
- H36 remains published as the first canonical PH1.M real-runtime `RememberEverything` retention-mode idempotent-retry and later-default-overwrite proof slice
- H37 remains published as the first canonical PH1.M real-runtime retention-mode same-idempotency-key replay response-truth proof slice
- the current wrapper path already rebuilt retention-mode response truth from the persisted repo row before this run
- the live `Ph1MRepo` carrier already exposed retention-preference row readback before this run

## Exact Seam
The exact canonical proof published by this run is `at_m_38_real_runtime_run_turn_and_persist_retention_mode_later_distinct_key_overwrite_returns_latest_response_truth`.

The exact seam is now canonically proven:
- later distinct-key overwrite now canonically returns the latest overwrite response truth on the wrapper path
- H36 remains the adjacent canonical persisted-truth proof for later overwrite row truth
- H37 remains the adjacent canonical replay-response-truth proof for same-key retries

## Implementation Boundary
H38 stayed bounded to the live PH1.M wrapper path and existing repo carrier:
- `run_turn_and_persist(...)`
- `persist_memory_forwarded_outcome(...)`
- `replay_retention_mode_response_truth_from_repo(...)`
- retention preference readback through the live `Ph1MRepo` carrier
- no carrier widening was required in this run

## Proof Plan
H38 now canonically proves that:
- an initial retention-mode request returns expected wrapper-path response truth
- a later distinct idempotency key overwrite returns the latest overwrite response truth rather than stale prior response truth
- H35 remains adjacent canonical wrapper-path single-commit coverage
- H36 remains adjacent canonical persisted no-op and later-overwrite coverage
- H37 remains adjacent canonical same-key replay response-truth coverage

`S06-12` is not selected because `S06-10` still exposes the smaller live seam.

`S06-19` is not selected because it remains `NOT_EXPLICIT`.

## Out Of Scope
This H38 plan does not claim or implement:
- lifecycle-worker closure
- broader retention-policy closure beyond the later-distinct-key overwrite latest-response-truth seam
- broader automatic temperature-transition closure
- broader `S06-12` trust closure
- any `S06-19` decay-model closure

`S06-10` remains `PARTIAL`.
`S06-12` remains `PARTIAL`.
`S06-19` remains `NOT_EXPLICIT`.
