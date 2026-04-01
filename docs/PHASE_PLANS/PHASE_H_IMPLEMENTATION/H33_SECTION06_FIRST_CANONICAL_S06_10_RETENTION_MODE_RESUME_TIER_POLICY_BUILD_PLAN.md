# H33 Section 06: First Canonical S06-10 Retention-Mode Resume-Tier Policy Build Plan

## Objective
This H33 slice is now the first canonical `RememberEverything` stronger auto-resume ranking priority and canonical PH1.M proof slice inside `S06-10`. Broader topic-match recall via digest summaries was already live before this run and remains preserved. Canonical `HOT/WARM/COLD` windows remain unchanged. `S06-10` remains `PARTIAL`, `S06-12` remains `PARTIAL`, and `S06-19` remains `NOT_EXPLICIT`.

## Current Repo Truth
Section 06 remains live and partial in current repo truth at [MASTER_BUILD_COMPLETION_PLAN.md#L158](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L158) and [MASTER_BUILD_COMPLETION_LEDGER.md#L220](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md#L220).

Current source already proved three adjacent retention-mode facts before this run:
- broader topic-match recall is already live through digest-summary matching on the canonical thread resume path at [ph1m.rs#L583](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L583)
- `RememberEverything` text-fallback behavior is already live and preserved at [ph1m.rs#L2921](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L2921)
- canonical `HOT/WARM/COLD` windows remain unchanged because [ph1m.rs#L1177](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L1177) still leaves `resume_tier_for(... retention_mode ...)` threshold-neutral for this slice

## Exact Seam
The exact implemented carrier is the canonical thread candidate ranking path inside `resume_select`, not threshold widening inside `resume_tier_for(... retention_mode ...)`:
- [ph1m.rs#L570](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L570)
- [ph1m.rs#L614](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L614)
- [ph1m.rs#L620](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L620)
- [ph1m.rs#L927](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L927)

This run proves that `RememberEverything` gives stronger auto-resume ranking priority on the canonical thread resume path while keeping both candidate threads inside the same `Warm` tier.

## Implementation Boundary
This run is tightly bounded to the already-live thread resume carrier and its canonical PH1.M forwarding path:
- the exact implemented carrier is the canonical thread candidate ranking path inside `resume_select`
- this run does not widen age thresholds
- this run does not reinterpret unresolved decay
- this run does not touch pending-work ranking
- broader topic-match recall remains preserved through the already-live digest-summary carrier

## Proof Plan
The implemented H33 proof surface is now published by these exact tests:
- engine proof: `remember_everything_prefers_higher_use_warm_thread_for_resume`
- engine proof: `default_prefers_more_recent_warm_thread_over_use_count_for_resume`
- PH1.M real-runtime proof: `at_m_32_real_runtime_remember_everything_prefers_higher_use_warm_thread`
- PH1.M real-runtime proof: `at_m_33_real_runtime_default_prefers_more_recent_warm_thread_over_use_count`

Those proofs establish that:
- under `RememberEverything`, an older higher-use `Warm` thread outranks a newer lower-use `Warm` thread
- under `Default`, the more recent `Warm` thread still outranks higher use count
- broader topic-match recall via digest summaries remained already live and preserved
- canonical `HOT/WARM/COLD` windows remain unchanged

## Out Of Scope
This run does not claim or implement:
- broader lifecycle-worker closure
- broader retention-policy closure
- broader automatic temperature-transition closure
- broader `S06-12` trust closure
- any `S06-19` decay-model closure
- any pending-work ranking change
- any threshold widening inside `resume_tier_for(... retention_mode ...)`
