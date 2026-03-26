# H25 Section 06: First Canonical PH1.M Conflict-Handling Trust-Signal Context-Bundle Surface and Bound Proof Build Plan

## Objective
Current published repo truth already exposes conflict trust-signal contract surfaces in `selene_kernel_contracts::ph1m` and engine-side `current_state_facts` conflict detection, conflict tag assignment, and conflict-count metric emission inside `selene_engines::ph1m`, but canonical PH1.M proof still does not show a real conflict-specific `ContextBundleBuild` slice. This H25 document freezes the first canonical PH1.M conflict-handling trust-signal context-bundle surface-and-proof slice only. Broader `S06-12` and broader Section 06 closure remain partial.

## Current Published Repo Truth
Section 06 remains live but partial by current repo truth at [MASTER_BUILD_COMPLETION_PLAN.md#L89](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L89), [COVERAGE_MATRIX.md#L10](/Users/selene/Documents/Selene-OS/docs/COVERAGE_MATRIX.md#L10), and [CORE_ARCHITECTURE.md#L173](/Users/selene/Documents/Selene-OS/docs/CORE_ARCHITECTURE.md#L173). The execution order still places PH1.M inside Section 06 after Sections 01-05 at [SELENE_BUILD_EXECUTION_ORDER.md#L178](/Users/selene/Documents/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md#L178), and the authoritative engine inventory still identifies PH1.M as the Section 06 storage/learning engine at [SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md#L86](/Users/selene/Documents/Selene-OS/docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md#L86).

Current repo truth already exposes the conflict-handling trust-signal contract surface at [selene_kernel_contracts/ph1m.rs#L70](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L70), [selene_kernel_contracts/ph1m.rs#L531](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L531), [selene_kernel_contracts/ph1m.rs#L1005](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L1005), [selene_kernel_contracts/ph1m.rs#L1788](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L1788), and [selene_kernel_contracts/ph1m.rs#L1921](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L1921). Current repo truth also exposes engine-side conflict handling on the `ContextBundleBuild` carrier at [selene_engines/ph1m.rs#L818](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L818), [selene_engines/ph1m.rs#L821](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L821), and [selene_engines/ph1m.rs#L929](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L929). The canonical PH1.M `ContextBundleBuild` forward branch already lives at [ph1m.rs#L743](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L743), but current PH1.M proof still only shows generic forwarding plus H24 confidence/ranking proof rather than conflict-tag proof.

## Exact Winner
The exact H25 winner is:
- add bounded engine tests in the `#[cfg(test)]` surface of `crates/selene_engines/src/ph1m.rs`
- add `context_bundle_current_state_conflict_emits_conflict_tag`
- add `context_bundle_conflict_metric_counts_only_conflicting_entries`
- add bounded PH1.M real-runtime tests in the `#[cfg(test)]` surface of `crates/selene_os/src/ph1m.rs`
- add `at_m_23_real_runtime_context_bundle_current_state_conflict_emits_conflict_tag`
- add `at_m_24_real_runtime_context_bundle_conflict_metric_counts_only_conflicting_entries`
- prove real conflict-tag trust-signal output through the canonical PH1.M turn surface by seeding with `MemoryOperation::Propose(...)`, populating `current_state_facts`, and asserting over `MemoryTurnOutput::ContextBundleBuild(...)`

This slice is the first canonical PH1.M conflict-handling trust-signal context-bundle surface-and-proof slice only. Broader `S06-12` and broader Section 06 closure remain partial after this slice because broader retrieval-ranking, eligibility, and broader conflict-handling trust closure are still unresolved.

## Primary Carrier And Boundary
The lawful H25 carrier stays bounded to the existing PH1.M contract and runtime surfaces:
- conflict tag / context-fact / conflict-count contracts at [selene_kernel_contracts/ph1m.rs#L70](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L70), [selene_kernel_contracts/ph1m.rs#L531](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L531), [selene_kernel_contracts/ph1m.rs#L1005](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L1005), [selene_kernel_contracts/ph1m.rs#L1788](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L1788), and [selene_kernel_contracts/ph1m.rs#L1921](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L1921)
- engine-side conflict detection / conflict tag / conflict-count emission at [selene_engines/ph1m.rs#L818](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L818), [selene_engines/ph1m.rs#L821](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L821), and [selene_engines/ph1m.rs#L929](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L929)
- canonical PH1.M `ContextBundleBuild` forwarding at [ph1m.rs#L743](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L743)

No storage edit, contract edit, engine-logic edit, or build-section wording edit is authorized in this run.

## Deterministic Invariants
H25 must mirror current repo truth instead of inventing new semantics:
- a mismatched `current_state_facts` value for the same key surfaces as `MemoryItemTag::Conflict`
- aligned high-confidence entries remain `MemoryItemTag::Confirmed`
- current context-bundle output surfaces `MemoryProvenanceTier::UserStated`
- `metric_payload.conflict_count` counts conflicting entries on the same bounded carrier
- `metric_payload.stale_count` must remain `0` in the non-stale H25 proof cases

H25 does not claim that full Section 06 trust architecture is complete. Broader `S06-12` trust-level closure remains partial.

## Proof Surface
Current repo truth already exposes the bounded engine proof surface:
- `ph1m::tests::context_bundle_is_bounded_and_tagged`
- `ph1m::tests::context_bundle_high_confidence_emits_confirmed_tag`
- `ph1m::tests::context_bundle_confidence_ranking_prefers_high_over_low`

Current PH1.M truth already exposes mock canonical forwarding proof and the existing real-runtime helper path:
- `ph1m::tests::at_m_14_architecture_operations_forwarded`
- `ph1m::tests::at_m_21_real_runtime_context_bundle_high_confidence_emits_confirmed_tag`
- `ph1m::tests::at_m_22_real_runtime_context_bundle_confidence_ranking_prefers_high_over_low`
- `real_runtime_wiring()` in `crates/selene_os/src/ph1m.rs`

The new H25 proof required by this seam is:
- `ph1m::tests::context_bundle_current_state_conflict_emits_conflict_tag`
- `ph1m::tests::context_bundle_conflict_metric_counts_only_conflicting_entries`
- `ph1m::tests::at_m_23_real_runtime_context_bundle_current_state_conflict_emits_conflict_tag`
- `ph1m::tests::at_m_24_real_runtime_context_bundle_conflict_metric_counts_only_conflicting_entries`

These new tests must exercise conflict-tag trust-signal output over the real engine/runtime path without any non-test production widening.

## Out-Of-Scope And Do-Not-Touch Areas
This H25 seam does not authorize:
- storage edits
- contract edits
- broader trust-level closure
- broader retrieval-ranking closure
- broader eligibility closure
- broader conflict-handling closure outside this exact slice
- build-section wording edits
- broader Section 06 completion claims

## Ordered Build Plan
1. Start from clean `main` with `HEAD == origin/main` and re-confirm that current plan truth still says the post-H24 next exact Section 06 winner is not yet published.
2. Re-confirm that conflict trust-signal contracts already exist in `selene_kernel_contracts::ph1m`.
3. Re-confirm that engine-side `ContextBundleBuild` already detects `current_state_facts` conflicts and emits conflict counts.
4. Keep the existing bounded engine tests and existing PH1.M mock-forward / H24 confidence coverage intact.
5. Add the two bounded engine conflict tests over `pull_items` only, using non-push memory keys.
6. Reuse the existing PH1.M real-runtime helper path and add the two bounded PH1.M real-runtime conflict tests over `MemoryTurnOutput::ContextBundleBuild(...)`.
7. Publish H25 in the master plan and master ledger without claiming full `S06-12` trust closure.
8. Run the bounded engine and PH1.M proof surface only.

## Verification And Publication Proof
The implementation run must prove:
- the H25 plan file exists
- the master plan now records that the first canonical H25 PH1.M conflict-handling trust-signal context-bundle slice is now published
- the master ledger now records that the first canonical H25 PH1.M conflict-handling trust-signal context-bundle slice is published
- real PH1.M wiring now proves conflict-tag output over the context-bundle engine/runtime path
- the two new engine tests exist and execute
- the two new PH1.M real-runtime tests exist and execute
- the pre-existing bounded engine and PH1.M proof surface still passes

Publication is lawful only if this proof stays bounded to the engine test surface, the PH1.M test surface, and master-doc publication truth. Broader `S06-12` and broader Section 06 closure remain partial after H25.

## Stop Conditions
Stop if repo truth no longer shows conflict trust-signal contract surfaces in kernel contracts, engine-side `ContextBundleBuild` no longer detects and counts conflicts through `current_state_facts`, or PH1.M no longer forwards `ContextBundleBuild` through the canonical turn surface.

Stop if the real PH1.M proof would require a production logic change, storage edit, contract edit, broader trust-level closure, broader retrieval-ranking integration, broader eligibility integration, broader conflict-handling integration outside this exact slice, or any wider Section 06 architecture change. Stop if the implementation attempt tries to claim more than this first canonical PH1.M conflict-handling trust-signal context-bundle surface-and-proof slice.
