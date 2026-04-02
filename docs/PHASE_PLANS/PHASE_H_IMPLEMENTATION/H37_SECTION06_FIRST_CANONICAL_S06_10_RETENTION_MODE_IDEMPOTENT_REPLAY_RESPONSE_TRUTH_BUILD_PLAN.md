# H37 Section 06: First Canonical S06-10 Retention-Mode Idempotent Replay Response-Truth Build Plan

## Objective
This H37 slice is now the first canonical PH1.M real-runtime retention-mode same-idempotency-key replay response-truth proof slice inside `S06-10`.

## Current Repo Truth
`S06-10` remains `PARTIAL`, `S06-12` remains `PARTIAL`, and `S06-19` remains `NOT_EXPLICIT`.

Current repo truth before this run already proved:
- core architecture already required the cloud runtime to return the same deterministic result / previously computed result for duplicate idempotency identity
- H35 remains published as the first canonical PH1.M real-runtime `RememberEverything` `run_turn_and_persist(...)` retention-mode proof slice
- H36 remains published as the first canonical PH1.M real-runtime `RememberEverything` retention-mode idempotent-retry and later-default-overwrite proof slice
- current engine `retention_mode_set(...)` still returned request-time response truth using the latest request mode and `req.now`
- current `run_turn_and_persist(...)` still returned the forwarded engine outcome after persistence
- the live `Ph1MRepo` carrier already exposed retention-preference row readback on the wrapper path

## Exact Seam
The exact canonical proof published by this run is `at_m_37_real_runtime_run_turn_and_persist_retention_mode_same_idempotency_key_replays_original_response_truth`.

The exact seam is now canonically proven:
- same-idempotency-key retry now replays original response truth on the wrapper path
- persisted row truth remains preserved on same-key retry
- H36 remains the adjacent canonical proof for later distinct-key overwrite truth

## Implementation Boundary
H37 stayed bounded to the live PH1.M wrapper path and existing repo carrier:
- `run_turn_and_persist(...)`
- `persist_memory_forwarded_outcome(...)`
- retention preference readback through the live `Ph1MRepo` carrier
- no carrier widening was required in this run

H37 did not reopen:
- broader lifecycle-worker closure
- broader retention-policy closure outside this response-truth seam
- broader automatic temperature-transition closure

## Proof Plan
H37 now canonically proves that:
- an initial `RememberEverything` retention-mode request persists and returns expected wrapper-path truth
- a same-idempotency-key retry with later request-time inputs replays the original response truth rather than emitting newly forwarded request-time response truth
- H35 remains adjacent canonical wrapper-path single-commit coverage
- H36 remains adjacent canonical persisted no-op and later-overwrite coverage

`S06-12` is not selected because `S06-10` still exposes the smaller live seam.

`S06-19` is not selected because it remains `NOT_EXPLICIT`.

## Out Of Scope
This H37 plan does not claim or implement:
- lifecycle-worker closure
- broader retention-policy closure beyond the replay response-truth seam
- broader automatic temperature-transition closure
- broader `S06-12` trust closure
- any `S06-19` decay-model closure

`S06-10` remains `PARTIAL`.
`S06-12` remains `PARTIAL`.
`S06-19` remains `NOT_EXPLICIT`.
