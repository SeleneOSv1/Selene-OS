# H27 Section 06: First Canonical PH1.M Unresolved-Resume Priority Window Surface and Bound Proof Build Plan

## Objective
Current published repo truth already exposes PH1.M Hot/Warm/Cold retention-tier publication through H23, stale-expiry retention context-bundle publication through H26, and engine-side unresolved-aware `ResumeSelect` prioritization / tie-break logic inside `selene_engines::ph1m`, but the canonical PH1.M proof still does not show a real unresolved-specific `ResumeSelect` priority slice. This H27 document freezes the first canonical PH1.M unresolved-resume priority window surface-and-proof slice only. This slice belongs to the remaining `S06-10` retention / actionability truth surface, does not claim `S06-19` decay-model quality-change closure, and broader `S06-10` and broader Section 06 closure remain partial.

## Current Published Repo Truth
Section 06 remains live but partial by current repo truth at [MASTER_BUILD_COMPLETION_PLAN.md#L89](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L89), [COVERAGE_MATRIX.md#L10](/Users/selene/Documents/Selene-OS/docs/COVERAGE_MATRIX.md#L10), and [CORE_ARCHITECTURE.md#L4437](/Users/selene/Documents/Selene-OS/docs/CORE_ARCHITECTURE.md#L4437). The execution order still places PH1.M inside Section 06 after Sections 01-05 at [SELENE_BUILD_EXECUTION_ORDER.md#L201](/Users/selene/Documents/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md#L201), and the authoritative engine inventory still identifies PH1.M as the Section 06 storage/learning engine at [SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md#L100](/Users/selene/Documents/Selene-OS/docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md#L100).

Current repo truth already exposes PH1.M Hot/Warm/Cold retention-tier publication through H23 at [H23_SECTION06_FIRST_CANONICAL_PH1M_HOT_WARM_COLD_RESUME_TIER_SURFACE_AND_BOUND_PROOF_BUILD_PLAN.md#L1](/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H23_SECTION06_FIRST_CANONICAL_PH1M_HOT_WARM_COLD_RESUME_TIER_SURFACE_AND_BOUND_PROOF_BUILD_PLAN.md#L1) and stale-expiry retention context-bundle publication through H26 at [H26_SECTION06_FIRST_CANONICAL_PH1M_STALE_EXPIRY_RETENTION_CONTEXT_BUNDLE_SURFACE_AND_BOUND_PROOF_BUILD_PLAN.md#L1](/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H26_SECTION06_FIRST_CANONICAL_PH1M_STALE_EXPIRY_RETENTION_CONTEXT_BUNDLE_SURFACE_AND_BOUND_PROOF_BUILD_PLAN.md#L1).

Current repo truth also exposes the unresolved-resume retention / actionability truth on the live `ResumeSelect` carrier:
- `MemoryThreadDigest.unresolved` at [selene_kernel_contracts/ph1m.rs#L1203](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L1203)
- `Ph1mResumeSelectResponse` on the canonical PH1.M output carrier at [selene_kernel_contracts/ph1m.rs#L1565](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L1565)
- `Ph1mConfig.unresolved_decay_window_ms` at [selene_engines/ph1m.rs#L62](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L62)
- unresolved-aware candidate ordering and tie-break logic at [selene_engines/ph1m.rs#L604](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L604), [selene_engines/ph1m.rs#L608](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L608), and [selene_engines/ph1m.rs#L630](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L630)
- unresolved-aware tier classification remains bounded inside `resume_tier_for(...)` at [selene_engines/ph1m.rs#L1177](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L1177)
- canonical PH1.M `ResumeSelect` forwarding at [ph1m.rs#L927](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L927)

The earlier unresolved cold-demotion-inside-warm hypothesis is invalidated by current repo truth because [selene_kernel_contracts/ph1m.rs#L10](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L10) is narrower than [selene_kernel_contracts/ph1m.rs#L11](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L11), while unresolved cold demotion still starts inside [selene_engines/ph1m.rs#L1177](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L1177). Canonical PH1.M proof still did not show real unresolved-specific `ResumeSelect` priority proof before this H27 run. H27 is therefore the first canonical PH1.M unresolved-resume priority window surface-and-proof slice.

## Exact Winner
The exact H27 winner is:
- add one bounded engine test inside the `#[cfg(test)]` surface of `crates/selene_engines/src/ph1m.rs`
- add `resume_select_unresolved_within_decay_window_breaks_warm_tie`
- add one bounded PH1.M test-only helper inside the `#[cfg(test)]` surface of `crates/selene_os/src/ph1m.rs` so unresolved-specific seeding uses truthful `pinned` then `unresolved` contract order
- add `at_m_27_real_runtime_resume_unresolved_within_decay_window_breaks_warm_tie`
- prove real unresolved-aware `ResumeSelect` priority output through the canonical PH1.M turn surface by seeding with `MemoryOperation::ThreadDigestUpsert(...)` and asserting over `MemoryTurnOutput::ResumeSelect(...)`

This slice is the first canonical PH1.M unresolved-resume priority window surface-and-proof slice only. Broader `S06-10` retention-class closure remains partial after this slice because lifecycle workers, retention policies, and broader automatic temperature-transition closure remain unresolved. This slice does not claim `S06-19` memory-decay closure.

## Primary Carrier And Boundary
The lawful H27 carrier stays bounded to the existing PH1.M contract and runtime surfaces:
- H23-published Hot/Warm/Cold retention-tier publication at [H23_SECTION06_FIRST_CANONICAL_PH1M_HOT_WARM_COLD_RESUME_TIER_SURFACE_AND_BOUND_PROOF_BUILD_PLAN.md#L1](/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H23_SECTION06_FIRST_CANONICAL_PH1M_HOT_WARM_COLD_RESUME_TIER_SURFACE_AND_BOUND_PROOF_BUILD_PLAN.md#L1)
- H26-published stale-expiry retention context-bundle publication at [H26_SECTION06_FIRST_CANONICAL_PH1M_STALE_EXPIRY_RETENTION_CONTEXT_BUNDLE_SURFACE_AND_BOUND_PROOF_BUILD_PLAN.md#L1](/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H26_SECTION06_FIRST_CANONICAL_PH1M_STALE_EXPIRY_RETENTION_CONTEXT_BUNDLE_SURFACE_AND_BOUND_PROOF_BUILD_PLAN.md#L1)
- unresolved thread-digest and `ResumeSelect` response contracts at [selene_kernel_contracts/ph1m.rs#L1203](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L1203) and [selene_kernel_contracts/ph1m.rs#L1565](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L1565)
- unresolved-aware engine-side candidate sort and tie-break logic at [selene_engines/ph1m.rs#L604](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L604), [selene_engines/ph1m.rs#L608](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L608), and [selene_engines/ph1m.rs#L630](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L630)
- canonical PH1.M `ResumeSelect` forwarding at [ph1m.rs#L927](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L927)

No storage edit, contract edit, engine-logic edit, or build-section wording edit is authorized in this run.

## Deterministic Invariants
H27 must mirror current repo truth instead of inventing new semantics:
- `cfg.resume_hot_window_ms < cfg.resume_warm_window_ms < cfg.unresolved_decay_window_ms`
- the proof age is strictly greater than the hot window and still inside the warm and unresolved-decay windows
- the resolved candidate must have a lexicographically earlier `thread_id` than the unresolved candidate
- every non-unresolved sort-relevant field must remain equal across the two candidates
- the unresolved warm candidate must surface as `MemoryResumeTier::Warm`
- the unresolved warm candidate must surface as `MemoryResumeAction::Suggest`

H27 proves retention / actionability truth on the live `ResumeSelect` carrier. It does not prove confidence reduction over time, background re-evaluation, or automatic demotion of stored knowledge quality, so it must not be used to claim `S06-19` decay-model quality-change closure.

## Proof Surface
Current repo truth already exposes the bounded engine proof surface:
- `ph1m::tests::resume_hot_window_auto_loads_with_72h_policy`
- `ph1m::tests::resume_warm_window_suggests_with_30d_policy`
- `ph1m::tests::resume_cold_window_returns_none`
- `ph1m::tests::resume_select_prefers_actionable_warm_over_cold_without_topic`

Current PH1.M truth already exposes mock canonical forwarding proof and real-runtime helper paths:
- `ph1m::tests::at_m_13_resume_select_forwarded`
- `ph1m::tests::at_m_17_real_runtime_resume_hot_surface`
- `ph1m::tests::at_m_18_real_runtime_resume_warm_surface`
- `ph1m::tests::at_m_19_real_runtime_resume_cold_without_topic_surface`
- `real_runtime_wiring()` at [ph1m.rs#L1324](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L1324)

The new H27 proof required by this seam is:
- `ph1m::tests::resume_select_unresolved_within_decay_window_breaks_warm_tie` at [selene_engines/ph1m.rs#L2128](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L2128)
- `thread_digest_upsert_request_at_with_contract_flags` at [ph1m.rs#L1280](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L1280)
- `ph1m::tests::at_m_27_real_runtime_resume_unresolved_within_decay_window_breaks_warm_tie` at [ph1m.rs#L1948](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L1948)

These new tests must exercise unresolved-aware resume priority output over the real engine/runtime path without any non-test production widening.

## Out-Of-Scope And Do-Not-Touch Areas
This H27 seam does not authorize:
- storage edits
- contract edits
- broader retention-class closure
- lifecycle-worker closure
- retention-policy closure
- broader automatic temperature-transition closure
- memory-decay closure
- build-section wording edits
- broader Section 06 completion claims

## Ordered Build Plan
1. Start from clean `main` with `HEAD == origin/main` and re-confirm that current plan truth still says the post-H26 next exact Section 06 winner is not yet published.
2. Re-confirm that PH1.M Hot/Warm/Cold publication already exists through H23 and stale-expiry context-bundle publication already exists through H26.
3. Re-confirm that unresolved thread-digest contract fields, unresolved decay configuration, engine-side unresolved-aware `ResumeSelect` prioritization / tie-break logic, and canonical PH1.M `ResumeSelect` forwarding already exist.
4. Re-confirm that the earlier unresolved cold-demotion-inside-warm hypothesis is invalidated by current repo truth because unresolved decay exceeds the warm window.
5. Keep the existing bounded engine tests and existing PH1.M mock-forward / H23 runtime coverage intact.
6. Add one bounded engine unresolved tie-break test that keeps every non-unresolved sort-relevant field equal while giving the resolved candidate the lexicographically earlier `thread_id`.
7. Add one bounded PH1.M test-only helper with truthful `pinned` then `unresolved` contract order and add one bounded PH1.M real-runtime unresolved tie-break test over `MemoryTurnOutput::ResumeSelect(...)`.
8. Publish H27 in the master plan and master ledger without claiming broader `S06-10` retention closure and without claiming `S06-19` decay closure.
9. Run the bounded engine and PH1.M proof surface only.

## Verification And Publication Proof
The implementation run must prove:
- the H27 plan file exists
- the master plan now records that the first canonical H27 PH1.M unresolved-resume priority window slice is now published
- the master ledger now records that the first canonical H27 PH1.M unresolved-resume priority window slice is published
- real PH1.M wiring now proves unresolved-resume priority output over the resume-select engine/runtime path
- the new engine unresolved tie-break test exists and executes
- the new PH1.M real-runtime unresolved tie-break test exists and executes
- the pre-existing bounded engine and PH1.M proof surface still passes

Publication is lawful only if this proof stays bounded to the PH1.M and engine test surfaces plus master-doc publication truth. Broader `S06-10` and broader Section 06 closure remain partial after H27.

## Stop Conditions
Stop if repo truth no longer shows PH1.M Hot/Warm/Cold publication through H23, stale-expiry publication through H26, unresolved thread-digest or `ResumeSelect` output contracts in `selene_kernel_contracts::ph1m`, unresolved-aware `ResumeSelect` prioritization / tie-break logic in `selene_engines::ph1m`, or canonical `ResumeSelect` forwarding in `crates/selene_os/src/ph1m.rs`.

Stop if the real PH1.M proof would require a production logic change, storage edit, contract edit, lifecycle-worker implementation, retention-policy implementation, broader automatic temperature-transition closure, or any memory-decay implementation. Stop if the implementation attempt tries to claim more than this first canonical PH1.M unresolved-resume priority window surface-and-proof slice.
