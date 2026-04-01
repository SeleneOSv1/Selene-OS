# H36 Section 06: First Canonical S06-10 RememberEverything Retention-Mode Idempotent-Retry And Later-Default-Overwrite Real-Runtime Proof Build Plan

## Objective
This is the first canonical H36 Section 06 build plan after the implemented H35 RememberEverything run_turn_and_persist proof slice. the next exact target remains `S06-10`. No code is changed in this run; this run only publishes the next active target.

## Current Repo Truth
`S06-10` remains `PARTIAL`, `S06-12` remains `PARTIAL`, and `S06-19` remains `NOT_EXPLICIT`.

Current repo truth already exposes the next smaller retention-policy seam:
- `run_turn_and_persist(...)` already exists as a live wrapper over run_turn plus persist_memory_forwarded_outcome
- H35 already proves a single RememberEverything wrapper-path persistence commit
- current lifecycle truth already states duplicate request returns original `updated_at` and no-op, while later config row wins
- `at_m_db_10_retention_mode_commit_idempotent` already proves storage-level idempotent retry and later overwrite semantics

## Exact Seam
the next exact target remains `S06-10`.

The exact seam is the canonical PH1.M real-runtime RememberEverything retention-mode idempotent-retry and later-default-overwrite proof path:
- duplicate retention-mode request idempotency remains a no-op on the live wrapper path
- a later distinct idempotency key can overwrite the persisted preference on that same wrapper path
- the proof remains bounded to PH1.M real-runtime retention preference truth rather than broader lifecycle-worker closure

## Implementation Boundary
H36 will add bounded PH1.M real-runtime proof that duplicate retention-mode request idempotency is a no-op and that a later distinct idempotency key can overwrite the persisted preference.

H36 will stay inside retention-policy closure and will not reopen lifecycle-worker or automatic temperature-transition closure.

## Proof Plan
H36 will publish canonical PH1.M real-runtime proof that:
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

S06-12 is not selected because S06-10 still exposes the smaller live seam.
S06-19 is not selected because it remains `NOT_EXPLICIT`.
