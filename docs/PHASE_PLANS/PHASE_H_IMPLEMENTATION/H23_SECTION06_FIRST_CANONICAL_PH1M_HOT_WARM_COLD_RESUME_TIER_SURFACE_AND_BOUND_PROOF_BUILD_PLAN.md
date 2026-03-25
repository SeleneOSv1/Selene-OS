# H23 Section 06: First Canonical PH1.M Hot/Warm/Cold Resume-Tier Surface and Bound Proof Build Plan

## Objective
Current published repo truth already exposes real engine-side `Hot/Warm/Cold` resume-tier behavior inside `selene_engines::ph1m`, but the canonical PH1.M turn-surface proof in `crates/selene_os/src/ph1m.rs` still only shows mock forwarding. This H23 document freezes the first canonical PH1.M Hot/Warm/Cold resume-tier surface-and-proof slice only. Broader Section 06 lifecycle-based retention-class closure remains partial.

## Current Published Repo Truth
Section 06 remains live but partial by current repo truth at [MASTER_BUILD_COMPLETION_PLAN.md#L89](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L89), [COVERAGE_MATRIX.md#L10](/Users/selene/Documents/Selene-OS/docs/COVERAGE_MATRIX.md#L10), and [CORE_ARCHITECTURE.md#L173](/Users/selene/Documents/Selene-OS/docs/CORE_ARCHITECTURE.md#L173). The execution order still places PH1.M inside Section 06 after Sections 01-05 at [SELENE_BUILD_EXECUTION_ORDER.md#L178](/Users/selene/Documents/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md#L178), and the authoritative engine inventory still identifies PH1.M as the Section 06 storage/learning engine at [SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md#L86](/Users/selene/Documents/Selene-OS/docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md#L86). Broader Section 06 closure remains partial outside this seam.

Current repo truth already exposes the kernel contract tier enum at [selene_kernel_contracts/ph1m.rs#L138](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L138), the real engine-side classifier and action mapping at [selene_engines/ph1m.rs#L1177](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L1177), and bounded engine proof at [selene_engines/ph1m.rs#L1749](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L1749). Current repo truth also exposes the canonical PH1.M `ResumeSelect` forward branch at [ph1m.rs#L927](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L927), but the current PH1.M proof still only shows mock `at_m_13_resume_select_forwarded` coverage inside `crates/selene_os/src/ph1m.rs`. Warm is the current middle tier exposed by repo truth in code.

## Exact Winner
The exact H23 winner is:
- add one bounded real-engine helper inside the `#[cfg(test)]` surface of `crates/selene_os/src/ph1m.rs`
- add `at_m_17_real_runtime_resume_hot_surface`
- add `at_m_18_real_runtime_resume_warm_surface`
- add `at_m_19_real_runtime_resume_cold_without_topic_surface`
- prove real `Hot/Warm/Cold` behavior through the canonical PH1.M turn surface by seeding with `MemoryOperation::ThreadDigestUpsert(...)` and asserting over `MemoryOperation::ResumeSelect(...)`

This slice is the first canonical PH1.M Hot/Warm/Cold resume-tier surface-and-proof slice only. Broader Section 06 closure remains partial after this slice because lifecycle workers, retention policies, and broader automatic temperature-transition closure are still unresolved.

## Primary Carrier And Boundary
The lawful implementation boundary for H23 is wholly inside the `#[cfg(test)]` surface of [ph1m.rs](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs). Current repo truth proves that the carrier is already live and bounded:
- PH1.M runtime trait bridge at [ph1m.rs#L512](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L512)
- canonical `ResumeSelect` forward branch at [ph1m.rs#L927](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L927)
- real engine-side classifier at [selene_engines/ph1m.rs#L1177](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L1177)

No storage edit, contract edit, engine-logic edit, or build-section wording edit is authorized in this run.

## Deterministic Invariants
H23 must mirror current repo truth instead of inventing new semantics:
- `Hot` must surface as `AutoLoad`
- `Warm` must surface as `Suggest`
- `Cold` without topic must surface as no selected tier and `None`

H23 does not claim that full lifecycle-based retention classes are complete. Broader Section 06 lifecycle-based retention-class closure remains partial.

## Proof Surface
Current repo truth already exposes the bounded engine proof surface:
- `ph1m::tests::resume_hot_window_auto_loads_with_72h_policy`
- `ph1m::tests::resume_warm_window_suggests_with_30d_policy`
- `ph1m::tests::resume_cold_window_returns_none`
- `ph1m::tests::resume_select_prefers_actionable_warm_over_cold_without_topic`

Current PH1.M truth already exposes mock canonical forwarding proof:
- `ph1m::tests::at_m_13_resume_select_forwarded`

The new H23 proof required by this seam is:
- `ph1m::tests::at_m_17_real_runtime_resume_hot_surface`
- `ph1m::tests::at_m_18_real_runtime_resume_warm_surface`
- `ph1m::tests::at_m_19_real_runtime_resume_cold_without_topic_surface`

These new tests must exercise the canonical PH1.M turn surface in `crates/selene_os/src/ph1m.rs` over the real engine/runtime path.

## Out-Of-Scope And Do-Not-Touch Areas
This H23 seam does not authorize:
- storage edits
- lifecycle-worker closure
- retention-policy closure
- broader automatic temperature-transition closure beyond current repo truth
- build-section wording edits
- broader Section 06 completion claims

## Ordered Build Plan
1. Start from clean `main` with `HEAD == origin/main` and re-confirm that current plan truth still says the post-H22 next exact Section 06 winner is not yet published.
2. Re-confirm that `MemoryResumeTier::{Hot, Warm, Cold}` already exists in kernel contracts and that the real classifier already exists in `selene_engines::ph1m`.
3. Keep the existing mock `wiring(true)` coverage in `crates/selene_os/src/ph1m.rs` intact.
4. Add one bounded real-engine helper inside the PH1.M test module using `selene_engines::ph1m::Ph1mRuntime`, `selene_engines::ph1m::Ph1mConfig::mvp_v1()`, and `Ph1mWiringConfig::mvp_v1(true)`.
5. Add the three bounded PH1.M real-runtime tests and assert the real canonical output over `MemoryTurnOutput::ResumeSelect(...)`.
6. Publish H23 in the master plan and master ledger without claiming full Section 06 retention-class closure.
7. Run the bounded engine and PH1.M proof surface only.

## Verification And Publication Proof
The implementation run must prove:
- the H23 plan file exists
- the master plan now records that the first canonical H23 PH1.M Hot/Warm/Cold resume-tier slice is now published
- the master ledger now records that the first canonical H23 PH1.M Hot/Warm/Cold resume-tier slice is published
- real PH1.M wiring now proves Hot/Warm/Cold resume-tier output over the engine/runtime path
- the three new PH1.M real-runtime tests exist and execute
- the bounded engine proof family still passes

Publication is lawful only if this proof stays bounded to the PH1.M test surface plus master-doc publication truth. Broader Section 06 closure remains partial after H23.

## Stop Conditions
Stop if repo truth no longer shows `MemoryResumeTier::{Hot, Warm, Cold}` in kernel contracts, the real classifier no longer exists in `selene_engines::ph1m`, or PH1.M no longer forwards `ResumeSelect` through the canonical turn surface.

Stop if the real PH1.M proof would require a production logic change, storage edit, contract edit, lifecycle-worker implementation, retention-policy implementation, or broader automatic temperature-transition closure. Stop if the implementation attempt tries to claim more than this first canonical PH1.M Hot/Warm/Cold resume-tier surface-and-proof slice.
