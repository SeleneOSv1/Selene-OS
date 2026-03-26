# H26 Section 06: First Canonical PH1.M Stale-Expiry Retention Context-Bundle Surface and Bound Proof Build Plan

## Objective
Current published repo truth already exposes replay-stable expiry authority through H22, PH1.M Hot/Warm/Cold retention-tier publication through H23, and engine-side stale classification on the canonical `ContextBundleBuild` carrier, but canonical PH1.M proof still does not show a real stale-specific `ContextBundleBuild` slice. This H26 document freezes the first canonical PH1.M stale-expiry retention context-bundle surface-and-proof slice only. This slice belongs to the remaining `S06-10` retention / expiry surface, does not claim `S06-19` decay-model quality-change closure, and broader `S06-10` and broader Section 06 closure remain partial.

## Current Published Repo Truth
Section 06 remains live but partial by current repo truth at [MASTER_BUILD_COMPLETION_PLAN.md#L89](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L89), [COVERAGE_MATRIX.md#L10](/Users/selene/Documents/Selene-OS/docs/COVERAGE_MATRIX.md#L10), and [CORE_ARCHITECTURE.md#L4437](/Users/selene/Documents/Selene-OS/docs/CORE_ARCHITECTURE.md#L4437). The execution order still places PH1.M inside Section 06 after Sections 01-05 at [SELENE_BUILD_EXECUTION_ORDER.md#L201](/Users/selene/Documents/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md#L201), and the authoritative engine inventory still identifies PH1.M as the Section 06 storage/learning engine at [SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md#L100](/Users/selene/Documents/Selene-OS/docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md#L100).

Current repo truth already exposes replay-stable expiry publication through H22 at [H22_SECTION06_FIRST_CANONICAL_MEMORY_LEDGER_EXPIRY_CARRIER_EXTENSION_AND_REBUILD_REPLAY_BOUNDARY_AND_BUILD_PLAN.md#L4](/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H22_SECTION06_FIRST_CANONICAL_MEMORY_LEDGER_EXPIRY_CARRIER_EXTENSION_AND_REBUILD_REPLAY_BOUNDARY_AND_BUILD_PLAN.md#L4) and [H22_SECTION06_FIRST_CANONICAL_MEMORY_LEDGER_EXPIRY_CARRIER_EXTENSION_AND_REBUILD_REPLAY_BOUNDARY_AND_BUILD_PLAN.md#L9](/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H22_SECTION06_FIRST_CANONICAL_MEMORY_LEDGER_EXPIRY_CARRIER_EXTENSION_AND_REBUILD_REPLAY_BOUNDARY_AND_BUILD_PLAN.md#L9). Current repo truth already exposes PH1.M Hot/Warm/Cold retention-tier publication through H23 at [H23_SECTION06_FIRST_CANONICAL_PH1M_HOT_WARM_COLD_RESUME_TIER_SURFACE_AND_BOUND_PROOF_BUILD_PLAN.md#L4](/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H23_SECTION06_FIRST_CANONICAL_PH1M_HOT_WARM_COLD_RESUME_TIER_SURFACE_AND_BOUND_PROOF_BUILD_PLAN.md#L4) and [H23_SECTION06_FIRST_CANONICAL_PH1M_HOT_WARM_COLD_RESUME_TIER_SURFACE_AND_BOUND_PROOF_BUILD_PLAN.md#L19](/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H23_SECTION06_FIRST_CANONICAL_PH1M_HOT_WARM_COLD_RESUME_TIER_SURFACE_AND_BOUND_PROOF_BUILD_PLAN.md#L19).

Current repo truth also exposes the stale-tag retention / expiry presentation truth on the live `ContextBundleBuild` carrier:
- `MemoryItemTag::Stale` at [selene_kernel_contracts/ph1m.rs#L70](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L70)
- `MemoryCandidate.expires_at` at [selene_kernel_contracts/ph1m.rs#L477](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L477)
- `MemoryMetricPayload.stale_count` at [selene_kernel_contracts/ph1m.rs#L1004](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L1004)
- `Ph1mContextBundleBuildResponse.metric_payload` at [selene_kernel_contracts/ph1m.rs#L1916](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L1916)
- engine-side stale detection / tag assignment / stale-count emission at [selene_engines/ph1m.rs#L814](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L814), [selene_engines/ph1m.rs#L824](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L824), [selene_engines/ph1m.rs#L924](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L924), and [selene_engines/ph1m.rs#L1324](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L1324)
- canonical PH1.M `ContextBundleBuild` forwarding at [ph1m.rs#L743](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L743)

Canonical PH1.M proof still did not show real stale-specific `ContextBundleBuild` proof before this H26 run. H26 is therefore the first canonical PH1.M stale-expiry retention context-bundle surface-and-proof slice.

## Exact Winner
The exact H26 winner is:
- add one bounded engine helper inside the `#[cfg(test)]` surface of `crates/selene_engines/src/ph1m.rs`
- add `context_bundle_expired_micro_entry_emits_stale_tag`
- add `context_bundle_stale_metric_counts_only_expired_entries`
- add one bounded PH1.M helper inside the `#[cfg(test)]` surface of `crates/selene_os/src/ph1m.rs`
- add `at_m_25_real_runtime_context_bundle_expired_micro_entry_emits_stale_tag`
- add `at_m_26_real_runtime_context_bundle_stale_metric_counts_only_expired_entries`
- prove real stale-tag output through the canonical PH1.M turn surface by seeding with `MemoryOperation::Propose(...)` and asserting over `MemoryTurnOutput::ContextBundleBuild(...)`

This slice is the first canonical PH1.M stale-expiry retention context-bundle surface-and-proof slice only. Broader `S06-10` retention-class closure remains partial after this slice because lifecycle workers, retention policies, and broader automatic temperature-transition closure remain unresolved. This slice does not claim `S06-19` memory-decay closure.

## Primary Carrier And Boundary
The lawful H26 carrier stays bounded to the existing PH1.M contract and runtime surfaces:
- H22-published expiry authority and replay-stable carrier publication at [H22_SECTION06_FIRST_CANONICAL_MEMORY_LEDGER_EXPIRY_CARRIER_EXTENSION_AND_REBUILD_REPLAY_BOUNDARY_AND_BUILD_PLAN.md#L9](/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H22_SECTION06_FIRST_CANONICAL_MEMORY_LEDGER_EXPIRY_CARRIER_EXTENSION_AND_REBUILD_REPLAY_BOUNDARY_AND_BUILD_PLAN.md#L9)
- H23-published Hot/Warm/Cold retention-tier publication at [H23_SECTION06_FIRST_CANONICAL_PH1M_HOT_WARM_COLD_RESUME_TIER_SURFACE_AND_BOUND_PROOF_BUILD_PLAN.md#L9](/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H23_SECTION06_FIRST_CANONICAL_PH1M_HOT_WARM_COLD_RESUME_TIER_SURFACE_AND_BOUND_PROOF_BUILD_PLAN.md#L9)
- stale tag / `expires_at` / `stale_count` contracts at [selene_kernel_contracts/ph1m.rs#L70](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L70), [selene_kernel_contracts/ph1m.rs#L477](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L477), [selene_kernel_contracts/ph1m.rs#L1004](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L1004), and [selene_kernel_contracts/ph1m.rs#L1916](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L1916)
- engine-side stale detection / tag assignment / stale-count emission at [selene_engines/ph1m.rs#L814](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L814), [selene_engines/ph1m.rs#L824](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L824), [selene_engines/ph1m.rs#L924](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L924), and [selene_engines/ph1m.rs#L1324](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L1324)
- canonical PH1.M `ContextBundleBuild` forwarding at [ph1m.rs#L743](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L743)

No storage edit, contract edit, engine-logic edit, or build-section wording edit is authorized in this run.

## Deterministic Invariants
H26 must mirror current repo truth instead of inventing new semantics:
- an expired micro-memory entry surfaces as `MemoryItemTag::Stale`
- a non-expired high-confidence comparison entry remains `MemoryItemTag::Confirmed`
- current context-bundle output surfaces `MemoryProvenanceTier::UserStated`
- `metric_payload.stale_count` counts expired entries on the same bounded carrier
- `metric_payload.conflict_count` must remain `0` in the non-conflict H26 proof cases

H26 proves retention / expiry presentation truth on the live `ContextBundleBuild` carrier. It does not prove confidence reduction over time, background re-evaluation, or automatic demotion, so it must not be used to claim `S06-19` decay-model quality-change closure.

## Proof Surface
Current repo truth already exposes the bounded engine proof surface:
- `ph1m::tests::context_bundle_is_bounded_and_tagged`
- `ph1m::tests::context_bundle_high_confidence_emits_confirmed_tag`
- `ph1m::tests::context_bundle_confidence_ranking_prefers_high_over_low`
- `ph1m::tests::context_bundle_current_state_conflict_emits_conflict_tag`
- `ph1m::tests::context_bundle_conflict_metric_counts_only_conflicting_entries`

Current PH1.M truth already exposes mock canonical forwarding proof and real-runtime helper paths:
- `ph1m::tests::at_m_14_architecture_operations_forwarded`
- `ph1m::tests::at_m_21_real_runtime_context_bundle_high_confidence_emits_confirmed_tag`
- `ph1m::tests::at_m_22_real_runtime_context_bundle_confidence_ranking_prefers_high_over_low`
- `ph1m::tests::at_m_23_real_runtime_context_bundle_current_state_conflict_emits_conflict_tag`
- `ph1m::tests::at_m_24_real_runtime_context_bundle_conflict_metric_counts_only_conflicting_entries`
- `real_runtime_wiring()` at [ph1m.rs#L1296](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L1296)

The new H26 proof required by this seam is:
- `ph1m::tests::context_bundle_expired_micro_entry_emits_stale_tag` at [selene_engines/ph1m.rs#L2493](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L2493)
- `ph1m::tests::context_bundle_stale_metric_counts_only_expired_entries` at [selene_engines/ph1m.rs#L2551](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L2551)
- `ph1m::tests::at_m_25_real_runtime_context_bundle_expired_micro_entry_emits_stale_tag` at [ph1m.rs#L2149](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L2149)
- `ph1m::tests::at_m_26_real_runtime_context_bundle_stale_metric_counts_only_expired_entries` at [ph1m.rs#L2196](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L2196)

These new tests must exercise stale-tag retention output over the real engine/runtime path without any non-test production widening.

## Out-Of-Scope And Do-Not-Touch Areas
This H26 seam does not authorize:
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
1. Start from clean `main` with `HEAD == origin/main` and re-confirm that current plan truth still says the post-H25 next exact Section 06 winner is not yet published.
2. Re-confirm that replay-stable expiry authority already exists through H22 and that PH1.M retention-tier publication already exists through H23.
3. Re-confirm that stale tag / `expires_at` / `stale_count` contracts already exist in `selene_kernel_contracts::ph1m`.
4. Re-confirm that engine-side `ContextBundleBuild` already derives stale tags from `expires_at` and emits stale counts.
5. Keep the existing bounded engine tests and existing PH1.M mock-forward / H24 / H25 coverage intact.
6. Add the two bounded engine stale tests over `pull_items` only, using non-push keys and an expired micro-memory carrier.
7. Reuse the existing PH1.M real-runtime helper path, add one bounded micro-proposal helper, and add the two bounded PH1.M real-runtime stale tests over `MemoryTurnOutput::ContextBundleBuild(...)`.
8. Publish H26 in the master plan and master ledger without claiming full `S06-10` retention closure and without claiming `S06-19` decay closure.
9. Run the bounded engine and PH1.M proof surface only.

## Verification And Publication Proof
The implementation run must prove:
- the H26 plan file exists
- the master plan now records that the first canonical H26 PH1.M stale-expiry retention context-bundle slice is now published
- the master ledger now records that the first canonical H26 PH1.M stale-expiry retention context-bundle slice is published
- real PH1.M wiring now proves stale-tag output over the context-bundle engine/runtime path
- the two new engine tests exist and execute
- the two new PH1.M real-runtime tests exist and execute
- the pre-existing bounded engine and PH1.M proof surface still passes

Publication is lawful only if this proof stays bounded to the PH1.M and engine test surfaces plus master-doc publication truth. Broader `S06-10` and broader Section 06 closure remain partial after H26.

## Stop Conditions
Stop if repo truth no longer shows replay-stable expiry authority through H22, PH1.M retention-tier publication through H23, stale-tag / `expires_at` / `stale_count` contracts in `selene_kernel_contracts::ph1m`, engine-side stale classification in `selene_engines::ph1m`, or canonical `ContextBundleBuild` forwarding in `crates/selene_os/src/ph1m.rs`.

Stop if the real PH1.M proof would require a production logic change, storage edit, contract edit, lifecycle-worker implementation, retention-policy implementation, broader automatic temperature-transition closure, or any memory-decay implementation. Stop if the implementation attempt tries to claim more than this first canonical PH1.M stale-expiry retention context-bundle surface-and-proof slice.
