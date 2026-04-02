# H37 Section 06: First Canonical S06-10 Retention-Mode Idempotent Replay Response-Truth Build Plan

## Objective
This is the first canonical H37 Section 06 build plan after the implemented H36 retention-mode idempotent-retry and later-default-overwrite proof slice.

The next exact target remains `S06-10`.

The exact seam is the canonical PH1.M real-runtime retention-mode same-idempotency-key replay response-truth path on `run_turn_and_persist(...)`.

## Current Repo Truth
`S06-10` remains `PARTIAL`, `S06-12` remains `PARTIAL`, and `S06-19` remains `NOT_EXPLICIT`.

Current repo truth already proves:
- core architecture already requires the cloud runtime to return the same deterministic result / previously computed result for duplicate idempotency identity
- H35 remains published as the first canonical PH1.M real-runtime `RememberEverything` `run_turn_and_persist(...)` retention-mode proof slice
- H36 already proves duplicate same-key retry is a persisted no-op and later distinct-key overwrite on the wrapper path
- current engine `retention_mode_set(...)` still returns request-time response truth using the latest request mode and `req.now`
- current `run_turn_and_persist(...)` still returns the forwarded engine outcome after persistence
- current generic `Ph1MRepo` carrier already exposes retention-preference row readback on the wrapper path
- no code is changed in this run; this run only publishes the next active target

## Exact Seam
H37 will publish the first canonical PH1.M real-runtime proof that same-idempotency-key retry replays original response truth rather than only preserving persisted row truth.

The exact candidate canonical proof for the next implementation is `at_m_37_real_runtime_run_turn_and_persist_retention_mode_same_idempotency_key_replays_original_response_truth`.

The remaining gap is now narrowly bounded:
- persisted row truth already remains stable on same-key retry
- later distinct-key overwrite truth is already canonically proven by H36
- the unresolved seam is whether the wrapper-path retry response replays original truth instead of returning newly forwarded request-time response truth

## Implementation Boundary
The next implementation should stay bounded to the live PH1.M wrapper path and existing repo carrier:
- `run_turn_and_persist(...)`
- `persist_memory_forwarded_outcome(...)`
- retention preference readback through the live `Ph1MRepo` carrier

H37 should not reopen:
- broader lifecycle-worker closure
- broader retention-policy closure outside this response-truth seam
- broader automatic temperature-transition closure

## Proof Plan
H37 should canonically prove that:
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
