# H35 Section 06: First Canonical S06-10 RememberEverything Retention-Mode Run-Turn-And-Persist Real-Runtime Proof Build Plan

## Objective
This is the first canonical H35 Section 06 build plan after the implemented H34 RememberEverything persistence-proof slice. the next exact target remains `S06-10`. No code is changed in this run; this run only publishes the next active target.

## Current Repo Truth
H34 remains the implemented PH1.M real-runtime `RememberEverything` retention-mode persistence proof slice inside `S06-10`, while `S06-10` remains `PARTIAL`, `S06-12` remains `PARTIAL`, and `S06-19` remains `NOT_EXPLICIT`.

Current repo truth already exposes the next smaller wrapper-path seam:
- `run_turn_and_persist(...)` already exists as a live wrapper over run_turn plus persist_memory_forwarded_outcome
- `at_m_18_run_turn_and_persist_commits_write_outcome` already exists but only proves a thread-digest write outcome
- `at_m_34_real_runtime_retention_mode_set_persists_remember_everything_to_repo` already exists but proves the direct persistence path rather than the wrapper path

## Exact Seam
the next exact target remains `S06-10`.

The exact seam is the canonical PH1.M real-runtime RememberEverything run_turn_and_persist retention-mode proof path:
- `run_turn_and_persist(...)`
- `MemoryOperation::RetentionModeSet(...)`
- retention preference truth committed through the wrapper path rather than the direct persist helper path

## Implementation Boundary
H35 will add bounded PH1.M real-runtime proof that run_turn_and_persist(...) commits RememberEverything retention preference truth directly.

H35 will stay inside retention-policy closure and will not reopen lifecycle-worker or automatic temperature-transition closure.

## Proof Plan
H35 will publish canonical PH1.M real-runtime proof that:
- `run_turn_and_persist(...)` forwards a real `RetentionModeSet` output for `MemoryRetentionMode::RememberEverything`
- the wrapper path commits `RememberEverything` retention preference truth directly into the repo
- the existing direct-path H34 proof remains preserved and adjacent

## Out Of Scope
This H35 slice does not claim or implement:
- lifecycle-worker closure
- broader retention-policy closure beyond this exact wrapper-path proof seam
- broader automatic temperature-transition closure
- broader `S06-12` trust closure
- any `S06-19` decay-model closure

S06-12 is not selected because S06-10 still exposes the smaller live seam.
S06-19 is not selected because it remains `NOT_EXPLICIT`.
