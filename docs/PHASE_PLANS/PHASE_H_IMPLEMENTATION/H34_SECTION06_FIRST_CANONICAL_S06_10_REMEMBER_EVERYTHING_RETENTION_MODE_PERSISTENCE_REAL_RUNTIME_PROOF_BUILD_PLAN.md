# H34 Section 06: First Canonical S06-10 RememberEverything Retention-Mode Persistence Real-Runtime Proof Build Plan

## Objective
This is the first canonical H34 Section 06 build plan after the implemented H33 RememberEverything ranking-priority slice. the next exact target remains `S06-10`. No code is changed in this run; this run only publishes the next active target.

## Current Repo Truth
H33 remains the implemented ranking-priority slice inside `S06-10`, while `S06-10` remains `PARTIAL`, `S06-12` remains `PARTIAL`, and `S06-19` remains `NOT_EXPLICIT`.

Current repo truth already exposes the smaller remaining retention-policy seam:
- `AT-MEM-PREF-01` already states that memory_retention_mode changes retention/resume policy deterministically
- `repo.ph1m_retention_mode_set_commit_row(...)` already exists
- the current persistence proof remains stub-forwarded and does not yet prove canonical real-runtime RememberEverything persistence truth

## Exact Seam
the next exact target remains `S06-10`.

The exact seam is the canonical PH1.M real-runtime RememberEverything retention-mode persistence proof path:
- `RetentionModeSet -> forwarded output -> persist_memory_forwarded_outcome -> retention-preference-row`
- live commit-row carrier in `/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs`
- current proof gap remains the stub-forwarded persistence test at `at_m_16_persist_forwarded_thread_and_retention_to_repo`

## Implementation Boundary
H34 will add bounded real-runtime PH1.M proof that RememberEverything retention preference persists correctly over the live RetentionModeSet -> forwarded output -> persist_memory_forwarded_outcome -> retention-preference-row path.

H34 will stay inside retention-policy closure and will not reopen lifecycle-worker or automatic temperature-transition closure.

## Proof Plan
H34 will publish canonical PH1.M real-runtime proof that:
- a real runtime `RetentionModeSet` with `MemoryRetentionMode::RememberEverything` produces the canonical forwarded output
- `persist_memory_forwarded_outcome(...)` commits the resulting retention preference row correctly
- canonical retention-preference row truth reflects `RememberEverything`, not the current stub-forwarded `Default` placeholder

## Out Of Scope
This H34 slice does not claim or implement:
- lifecycle-worker closure
- broader retention-policy closure beyond this exact persistence proof seam
- broader automatic temperature-transition closure
- broader `S06-12` trust closure
- any `S06-19` decay-model closure

S06-12 is not selected because S06-10 still exposes the smaller live seam.
S06-19 is not selected because it remains `NOT_EXPLICIT`.
