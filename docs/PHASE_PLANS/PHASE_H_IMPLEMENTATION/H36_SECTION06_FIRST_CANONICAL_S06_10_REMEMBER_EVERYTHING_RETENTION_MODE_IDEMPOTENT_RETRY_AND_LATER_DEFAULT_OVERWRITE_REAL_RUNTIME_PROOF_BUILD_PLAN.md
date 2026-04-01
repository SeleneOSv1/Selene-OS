# H36 Section 06: First Canonical S06-10 RememberEverything Retention-Mode Idempotent-Retry And Later-Default-Overwrite Real-Runtime Proof Build Plan

## Objective
This H36 slice is now the first canonical PH1.M real-runtime `RememberEverything` retention-mode idempotent-retry and later-default-overwrite proof slice inside `S06-10`.

## Current Repo Truth
`S06-10` remains `PARTIAL`, `S06-12` remains `PARTIAL`, and `S06-19` remains `NOT_EXPLICIT`.

The live wrapper and persistence carriers already existed before this run:
- `run_turn_and_persist(...)` already exists as a live wrapper over run_turn plus persist_memory_forwarded_outcome
- H35 remains published as the first canonical PH1.M real-runtime `RememberEverything` `run_turn_and_persist(...)` retention-mode proof slice
- current lifecycle truth already states duplicate request returns original `updated_at` and no-op, while later config row wins
- storage truth remains published and adjacent through `at_m_db_10_retention_mode_commit_idempotent`

## Exact Seam
The exact canonical proof published by this run is `at_m_36_real_runtime_run_turn_and_persist_retention_mode_idempotent_retry_and_later_default_overwrite`.

The exact seam remains the canonical PH1.M real-runtime `RememberEverything` retention-mode idempotent-retry and later-default-overwrite proof path:
- duplicate same-key retry remains a persisted no-op on the live wrapper path
- later distinct-key overwrite to `Default` is now canonically proven on the wrapper path
- the proof remains bounded to PH1.M real-runtime retention preference truth rather than broader lifecycle-worker closure

## Implementation Boundary
H36 added bounded PH1.M real-runtime proof that duplicate retention-mode request idempotency is a no-op and that a later distinct idempotency key can overwrite the persisted preference.

H36 stays inside retention-policy closure and does not reopen lifecycle-worker or automatic temperature-transition closure.

## Proof Plan
H36 now canonically proves that:
- a first `RememberEverything` retention-mode request persists as expected through `run_turn_and_persist(...)`
- a duplicate request with the same idempotency key returns the original effective truth without overwriting the persisted preference
- a later distinct idempotency key can overwrite the persisted preference to `Default`
- the wrapper-path proof matches the already-published lifecycle and storage truth

## Out Of Scope
This H36 slice does not claim or implement:
- lifecycle-worker closure
- broader retention-policy closure beyond this exact idempotent-retry and later-overwrite wrapper-path proof seam
- broader automatic temperature-transition closure
- broader `S06-12` trust closure
- any `S06-19` decay-model closure

`S06-10` remains `PARTIAL`.
`S06-12` remains `PARTIAL`.
`S06-19` remains `NOT_EXPLICIT`.
