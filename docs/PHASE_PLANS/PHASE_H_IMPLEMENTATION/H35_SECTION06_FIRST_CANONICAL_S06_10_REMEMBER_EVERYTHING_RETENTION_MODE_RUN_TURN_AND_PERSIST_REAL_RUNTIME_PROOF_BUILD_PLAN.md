# H35 Section 06: First Canonical S06-10 RememberEverything Retention-Mode Run-Turn-And-Persist Real-Runtime Proof Build Plan

## Objective
This H35 slice is now the first canonical PH1.M real-runtime `RememberEverything` `run_turn_and_persist(...)` retention-mode proof slice inside `S06-10`.

## Current Repo Truth
H34 remains the implemented PH1.M real-runtime `RememberEverything` retention-mode persistence proof slice inside `S06-10`, while `S06-10` remains `PARTIAL`, `S06-12` remains `PARTIAL`, and `S06-19` remains `NOT_EXPLICIT`.

The live wrapper and persistence carriers already existed before this run:
- `run_turn_and_persist(...)` already exists as a live wrapper over run_turn plus persist_memory_forwarded_outcome
- `persist_memory_forwarded_outcome(...)` already commits `RetentionModeSet` rows through `repo.ph1m_retention_mode_set_commit_row(...)`
- H34 direct-path proof remains published and adjacent
- `at_m_18_run_turn_and_persist_commits_write_outcome` remains preserved as adjacent non-canonical wrapper write-outcome coverage

## Exact Seam
The exact seam is the canonical PH1.M real-runtime `RememberEverything` `run_turn_and_persist(...)` retention-mode proof path:
- `run_turn_and_persist(...)`
- `MemoryOperation::RetentionModeSet(...)`
- retention preference truth committed through the wrapper path rather than the direct persist helper path
- the exact canonical proof published by this run is `at_m_35_real_runtime_run_turn_and_persist_retention_mode_set_commits_remember_everything_to_repo`

## Implementation Boundary
This run adds bounded PH1.M real-runtime proof that `run_turn_and_persist(...)` commits `RememberEverything` retention preference truth directly.
This run stays inside retention-policy closure and does not reopen lifecycle-worker or automatic temperature-transition closure.

## Proof Plan
Canonical PH1.M real-runtime proof published by this run:
- `run_turn_and_persist(...)` forwards a real `RetentionModeSet` output for `MemoryRetentionMode::RememberEverything`
- the wrapper path commits `RememberEverything` retention preference truth directly into the repo
- `at_m_35_real_runtime_run_turn_and_persist_retention_mode_set_commits_remember_everything_to_repo` is the exact canonical proof
- the H34 direct-path proof remains published and adjacent

## Out Of Scope
This H35 slice does not claim or implement:
- lifecycle-worker closure
- broader retention-policy closure beyond this exact wrapper-path proof seam
- broader automatic temperature-transition closure
- broader `S06-12` trust closure
- any `S06-19` decay-model closure

`S06-10` remains `PARTIAL`.
`S06-12` remains `PARTIAL`.
`S06-19` remains `NOT_EXPLICIT`.
S06-12 is not selected because S06-10 still exposes the smaller live seam.
S06-19 is not selected because it remains `NOT_EXPLICIT`.
