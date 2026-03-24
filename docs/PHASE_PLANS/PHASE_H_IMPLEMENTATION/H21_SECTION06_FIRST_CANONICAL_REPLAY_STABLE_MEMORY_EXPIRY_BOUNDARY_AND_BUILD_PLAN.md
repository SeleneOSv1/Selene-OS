# H21 Section 06: First Canonical Replay-Stable Memory Expiry Boundary and Build Plan

## Objective
Current published repo truth is that Section 06 remains the next partial memory-closure area after the published Section 05 H20 slice, but the latest read-only audit has now exposed one exact bounded Section 06 winner. This document freezes the first canonical Section 06 replay-stable memory expiry boundary from current repo evidence, defines the smallest lawful later implementation slice, and states the exact verification proof that a later implementation run must satisfy before publication.

## Current Published Repo Truth
Section 06 remains live but partial by current repo truth at [MASTER_BUILD_COMPLETION_PLAN.md#L89](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L89), [COVERAGE_MATRIX.md#L10](/Users/selene/Documents/Selene-OS/docs/COVERAGE_MATRIX.md#L10), and [MASTER_BUILD_COMPLETION_LEDGER.md#L225](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md#L225). The architecture still describes broader PH1.M retention and lifecycle closure as incomplete at [CORE_ARCHITECTURE.md#L4437](/Users/selene/Documents/Selene-OS/docs/CORE_ARCHITECTURE.md#L4437), so this H21 boundary is not a claim that all of Section 06 is finished.

Current published repo truth already exposes the exact replay-stable expiry seam. PH1.M derives expiry before persistence in `derive_expires_at_from_ledger_event` at [ph1m.rs#L294](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L294) and threads that derived `expires_at` into `ph1m_append_ledger_row` inside `persist_memory_forwarded_outcome` at [ph1m.rs#L338](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L338). PH1.F already preserves that expiry when applying current-view state through `apply_memory_event_to_current` at [ph1f.rs#L3919](/Users/selene/Documents/Selene-OS/crates/selene_storage/src/ph1f.rs#L3919). But `rebuild_memory_current_from_ledger()` currently replays with `apply_memory_event_to_current(..., None)` at [ph1f.rs#L3972](/Users/selene/Documents/Selene-OS/crates/selene_storage/src/ph1f.rs#L3972), which drops replay-time expiry semantics even though the append path already preserved them.

Current published repo truth also already exposes the bounded proof surface for this seam in the same file. The existing rebuild proof family compares `memory_current()` before and after rebuild at [ph1f.rs#L24170](/Users/selene/Documents/Selene-OS/crates/selene_storage/src/ph1f.rs#L24170). That means the later implementation run can prove this boundary without widening into unrelated files, carriers, or sections.

## Exact Winner
The first canonical Section 06 winner is replay-stable PH1.M memory expiry preservation during `memory_current` rebuild in `crates/selene_storage/src/ph1f.rs`. This is the first exact lawful Section 06 seam because the producer already derives expiry upstream, the append path already preserves it, the rebuild path demonstrably drops it, and the proof surface is already localized to the same-file rebuild invariance family.

This H21 winner is intentionally narrower than generalized PH1.M lifecycle closure. Publishing this slice later would mean the first replay-stable expiry carrier is complete; it would not mean broader retention workers, purge orchestration, or all Section 06 architecture closure are complete.

## Primary Carrier And Boundary
The primary producer boundary begins in PH1.M when `persist_memory_forwarded_outcome` derives one deterministic `expires_at` value from one ledger event at [ph1m.rs#L314](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L314). The primary consumer boundary ends in PH1.F when `rebuild_memory_current_from_ledger()` reconstructs current state from ledger rows at [ph1f.rs#L3972](/Users/selene/Documents/Selene-OS/crates/selene_storage/src/ph1f.rs#L3972). The lawful implementation carrier for this first canonical seam is therefore only `crates/selene_storage/src/ph1f.rs`.

`apply_memory_event_to_current` is the canonical current-view mutation edge for this slice at [ph1f.rs#L3919](/Users/selene/Documents/Selene-OS/crates/selene_storage/src/ph1f.rs#L3919). The later implementation run may tighten only how replay routes expiry into that existing edge. It may not invent a second current-view mutation path, move responsibility into a different crate, or reopen the PH1.M repo contract surface.

## Deterministic Invariants
Micro-memory `Stored` and `Updated` events must preserve the same deterministic `expires_at` semantics across append and rebuild. Working-memory and long-term-memory rows must continue to rebuild with no expiry. Forgotten rows must remain inactive tombstones with no expiry. Rebuild must remain deterministic from ledger state alone and must not depend on app-layer state, generalized retention workers, or parallel authority paths.

The later implementation run must preserve existing PH1.M meaning: expiry remains derived from the already-published ledger event semantics, not recomputed from a different authority source. No schema change, repo trait change, governance widening, or cross-section contract rename is lawful inside this first canonical seam.

## Out-Of-Scope And Do-Not-Touch Areas
Generalized retention workers are out of scope. Purge orchestration is out of scope. Repo trait changes, schema changes, migration work, and storage-wide refactors are out of scope. Section 05 generalized persistence closure, Section 07 and Section 11 identity work, Section 09 governance closure, Apple client work, and app-layer memory consumers are all out of scope.

This boundary also does not authorize broader PH1.M lifecycle completion claims. If later repo truth still leaves generalized Section 06 closure partial after this slice is implemented, the lawful publication claim is that the first canonical replay-stable expiry seam is published, not that the entire section is complete.

## Ordered Build Plan
1. Start from clean `main` with `HEAD == origin/main` and baseline the current replay gap in `crates/selene_storage/src/ph1f.rs`.
2. Tighten only the `rebuild_memory_current_from_ledger()` path so replay preserves the same expiry semantics already derived upstream and already honored by the append path.
3. Keep the implementation confined to `crates/selene_storage/src/ph1f.rs`; do not widen into `ph1m.rs`, `repo.rs`, schema work, or lifecycle worker orchestration.
4. Extend the existing same-file rebuild proof family to prove that micro-memory expiry survives rebuild while working, long-term, and forgotten behavior remain unchanged.
5. Run only the targeted verification needed for the touched PH1.F carrier and publish only if the tree ends clean and broader Section 06 is still truthfully described as partial.

## Verification And Publication Proof
A later implementation run must prove that the old replay path that passes `None` into `apply_memory_event_to_current` is replaced by replay-stable expiry preservation in `crates/selene_storage/src/ph1f.rs`, and that the proof remains bounded to the same-file rebuild test family rooted at [ph1f.rs#L24170](/Users/selene/Documents/Selene-OS/crates/selene_storage/src/ph1f.rs#L24170). The implementation proof must explicitly show that a micro-memory row that carried expiry before rebuild still carries the same `expires_at` after rebuild.

The same later implementation run must also prove that no out-of-scope files changed, no repo trait or schema widening occurred, and no broader Section 06 lifecycle claims were introduced. If the replay-stable expiry seam publishes cleanly while broader Section 06 closure remains partial, the lawful publication claim is that the first canonical Section 06 replay-stable expiry slice is published.

## Stop Conditions
Stop if the later implementation requires touching any file outside `crates/selene_storage/src/ph1f.rs`. Stop if the later implementation requires repo trait changes, schema changes, migrations, generalized retention workers, purge orchestration, or broader PH1.M lifecycle closure. Stop if the later implementation would reopen Section 05 generalized closure, Section 07 or Section 11 identity work, Section 09 governance closure, or app-layer work.

Stop if baseline repo truth no longer shows the exact replay-stable expiry gap at the declared anchors. Stop if the later implementation cannot stay bounded to the same-file rebuild proof family. Stop if the later implementation would change contract meaning instead of preserving already-published expiry semantics through replay.
