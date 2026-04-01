# H33 Section 06: First Canonical S06-10 Retention-Mode Resume-Tier Policy Build Plan

## Objective
This is the first canonical H33 Section 06 build plan after the AGENTS deadlock-prevention override ended repeat same-state audit looping. the next exact target is `S06-10`, and this run publishes that target only. No code is changed in this run; this run only publishes the next active target.

## Current Repo Truth
Section 06 remains live and partial in current repo truth at [MASTER_BUILD_COMPLETION_PLAN.md#L158](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L158) and [MASTER_BUILD_COMPLETION_LEDGER.md#L220](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md#L220). `S06-10` remains `PARTIAL`, `S06-12` remains `PARTIAL`, and `S06-19` remains `NOT_EXPLICIT`.

Current source already exposes the live seam needed for the next exact target:
- the live seam is in `resume_tier_for(... retention_mode ...)`
- retention_mode already exists in contracts/engine/runtime
- the current tier function still ignores `retention_mode`
- adjacent retention-mode truth is already live through text-fallback, summary-bullet topic-hint matching, and preserved 30d warm-boundary behavior

## Exact Seam
the next exact target is `S06-10`.

H33 is bounded to the direct retention-mode resume-tier policy seam already visible in current source:
- [ph1m.rs#L1177](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L1177)
- [ph1m.rs#L1180](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L1180)
- [ph1m.rs#L1183](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L1183)

This next build will make retention mode affect resume-tier behavior.

## Implementation Boundary
The H33 implementation seam is expected to stay bounded to already-live PH1.M carriers and their canonical proof surfaces:
- kernel contract retention-mode and resume-tier carriers in [ph1m.rs#L132](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L132) and [ph1m.rs#L138](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L138)
- engine-side retention-mode set and resume-select behavior in [ph1m.rs#L587](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L587), [ph1m.rs#L1138](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L1138), and [ph1m.rs#L1177](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L1177)
- canonical PH1.M runtime path in [ph1m.rs#L498](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L498), [ph1m.rs#L577](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L577), and [ph1m.rs#L882](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L882)

H33 will add bounded engine proofs and PH1.M runtime proofs for retention-mode resume-tier policy behavior.

## Proof Plan
The H33 implementation run should prove the exact live seam over the bounded proof family already adjacent to current repo truth:
- engine proof that retention mode changes resume-tier policy instead of being ignored in `resume_tier_for(... retention_mode ...)`
- engine proof that adjacent retention-mode behavior remains consistent with:
  - text-fallback delivery truth
  - summary-bullet topic-hint matching truth
  - preserved 30d warm-boundary truth
- PH1.M real-runtime proof that the canonical turn surface reflects the new retention-mode resume-tier policy behavior without widening the implementation boundary

## Out Of Scope
H33 does not select or claim:
- broader `S06-10` lifecycle-worker closure
- broader `S06-10` retention-policy closure beyond this exact seam
- broader automatic temperature-transition closure
- broader `S06-12` trust closure
- any `S06-19` decay-model closure
- broader Section 06 completion

S06-12 is not selected because it remains broader and less direct.
S06-19 is not selected because it remains `NOT_EXPLICIT`.
