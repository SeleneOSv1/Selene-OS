# H22 Section 06: First Canonical Memory Ledger Expiry Carrier Extension and Rebuild Replay Boundary and Build Plan

## Objective
Current published repo truth now exposes the first exact lawful Section 06 implementation winner after the H21 truth-correction pass. This H22 document freezes that winner only: extend the PH1.F memory-ledger carrier so replay can preserve the already-derived PH1.M micro-memory expiry semantics during `memory_current` rebuild. This is the first canonical Section 06 implementation seam only, not a claim that broader Section 06 lifecycle closure is complete.

## Current Published Repo Truth
Section 06 remains live but partial by current repo truth at [MASTER_BUILD_COMPLETION_PLAN.md#L89](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L89), [COVERAGE_MATRIX.md#L10](/Users/selene/Documents/Selene-OS/docs/COVERAGE_MATRIX.md#L10), and [CORE_ARCHITECTURE.md#L4437](/Users/selene/Documents/Selene-OS/docs/CORE_ARCHITECTURE.md#L4437). The execution order still places PH1.M inside Section 06 after Sections 01-05 at [SELENE_BUILD_EXECUTION_ORDER.md#L178](/Users/selene/Documents/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md#L178), and the authoritative engine inventory still identifies PH1.M as the Section 06 storage/learning engine at [SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md#L86](/Users/selene/Documents/Selene-OS/docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md#L86). Broader retention and purge lifecycle enforcement therefore remains partial outside this seam.

Current repo truth now exposes one exact replay-stable expiry carrier seam. PH1.M derives deterministic `expires_at` authority in `derive_expires_at_from_ledger_event` at [ph1m.rs#L294](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L294), and `persist_memory_forwarded_outcome` threads that value into `ph1m_append_ledger_row` at [ph1m.rs#L338](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L338). The PH1.M repo contract already carries `expires_at` at [repo.rs#L1738](/Users/selene/Documents/Selene-OS/crates/selene_storage/src/repo.rs#L1738), and `append_memory_ledger_event()` already accepts `expires_at` at [ph1f.rs#L3858](/Users/selene/Documents/Selene-OS/crates/selene_storage/src/ph1f.rs#L3858).

Current repo truth also exposes the exact drop points. `MemoryLedgerRow` still does not expose `expires_at` at [ph1f.rs#L1022](/Users/selene/Documents/Selene-OS/crates/selene_storage/src/ph1f.rs#L1022), the only visible constructor path is `self.memory_ledger.push(MemoryLedgerRow {` at [ph1f.rs#L3903](/Users/selene/Documents/Selene-OS/crates/selene_storage/src/ph1f.rs#L3903), and `rebuild_memory_current_from_ledger()` still replays with `apply_memory_event_to_current(..., None)` at [ph1f.rs#L3984](/Users/selene/Documents/Selene-OS/crates/selene_storage/src/ph1f.rs#L3984). That makes the PH1.F ledger-row carrier extension the first exact lawful implementation slice.

## Exact Winner
The exact H22 winner is:
- add `expires_at` to `MemoryLedgerRow`
- persist append-time `expires_at` into the `MemoryLedgerRow` constructor
- replay `row.expires_at` instead of `None` during `rebuild_memory_current_from_ledger()`
- add one same-file proof test named `at_f_02b_current_state_rebuild_preserves_micro_expiry`

This slice is the first canonical Section 06 implementation seam only. Broader Section 06 closure remains partial after this slice because generalized retention, purge, and lifecycle-governance completion are still unresolved architectural gaps.

## Primary Carrier And Boundary
The producer authority remains upstream in PH1.M and is already published. The storage/replay carrier boundary for this slice is wholly inside PH1.F at [ph1f.rs](/Users/selene/Documents/Selene-OS/crates/selene_storage/src/ph1f.rs). Current repo truth proves that the lawful implementation boundary is limited to the ledger-row carrier and rebuild replay path inside that file:
- `MemoryLedgerRow` definition at [ph1f.rs#L1022](/Users/selene/Documents/Selene-OS/crates/selene_storage/src/ph1f.rs#L1022)
- append-time constructor at [ph1f.rs#L3903](/Users/selene/Documents/Selene-OS/crates/selene_storage/src/ph1f.rs#L3903)
- replay call site at [ph1f.rs#L3984](/Users/selene/Documents/Selene-OS/crates/selene_storage/src/ph1f.rs#L3984)

No repo trait change, schema change, or edit outside `ph1f.rs` is authorized for the later implementation run. The repo contract already carries `expires_at`, so this slice is a same-file PH1.F carrier-extension seam.

## Deterministic Invariants
Micro-memory `Stored` and `Updated` events must preserve the same deterministic `expires_at` across append and rebuild. Working-memory and long-term-memory rows must continue to rebuild with no expiry. `Forgotten` rows must remain inactive tombstones with no expiry. Replay must stay deterministic from ledger state alone and must not recompute expiry from a different authority source during the H22 implementation run.

This slice must preserve the already-published meaning of PH1.M: `expires_at` remains derived upstream from ledger event semantics, not re-invented in PH1.F. Idempotency behavior, append-only ledger guarantees, and existing current-view mutation semantics must remain unchanged except for replay-stable expiry preservation.

## Proof Surface
Current repo truth already exposes the bounded proof surface:
- `at_f_01_ledger_append_only` at [ph1f.rs#L24125](/Users/selene/Documents/Selene-OS/crates/selene_storage/src/ph1f.rs#L24125)
- `at_f_02_current_state_rebuild_matches` at [ph1f.rs#L24143](/Users/selene/Documents/Selene-OS/crates/selene_storage/src/ph1f.rs#L24143)
- `at_f_03_forget_writes_ledger_and_deactivates_current` at [ph1f.rs#L24177](/Users/selene/Documents/Selene-OS/crates/selene_storage/src/ph1f.rs#L24177)
- `at_m_db_04_rebuild_current_from_ledger` at [db_wiring.rs#L228](/Users/selene/Documents/Selene-OS/crates/selene_storage/tests/ph1_m/db_wiring.rs#L228)

The new proof required by this seam is one same-file PH1.F test: `at_f_02b_current_state_rebuild_preserves_micro_expiry`. That test must prove that micro-memory expiry survives rebuild when the ledger row now carries append-time `expires_at`.

## Out-Of-Scope And Do-Not-Touch Areas
This H22 seam does not authorize repo trait changes, schema changes, migrations, generalized retention workers, purge orchestration, Section 05 generalized closure, Section 07 and Section 11 identity work, Section 09 governance closure, or app-layer work.

This H22 seam also does not authorize broader Section 06 lifecycle completion claims. Even after the later implementation run publishes this slice, broader Section 06 closure remains partial unless later repo truth separately proves governed retention and purge lifecycle completion.

## Ordered Build Plan
1. Start from clean `main` with `HEAD == origin/main` and re-confirm the current PH1.M expiry authority, PH1.M repo contract, PH1.F append-time acceptance, PH1.F ledger-row drop point, and PH1.F replay-with-`None` call site.
2. Extend `MemoryLedgerRow` inside `crates/selene_storage/src/ph1f.rs` to store `expires_at` without changing the repo trait surface or widening outside PH1.F.
3. Persist append-time `expires_at` into the only visible `MemoryLedgerRow` constructor at [ph1f.rs#L3903](/Users/selene/Documents/Selene-OS/crates/selene_storage/src/ph1f.rs#L3903).
4. Replay `row.expires_at` instead of `None` at [ph1f.rs#L3984](/Users/selene/Documents/Selene-OS/crates/selene_storage/src/ph1f.rs#L3984).
5. Add `at_f_02b_current_state_rebuild_preserves_micro_expiry` alongside the existing PH1.F rebuild proof family in `crates/selene_storage/src/ph1f.rs`.
6. Re-run the bounded PH1.F proof family plus the existing PH1.M db-wiring rebuild proof to prove the seam without widening into broader Section 06 lifecycle work.

## Verification And Publication Proof
The later implementation run must prove:
- `MemoryLedgerRow.expires_at` exists in PH1.F
- append-time persistence writes the already-derived `expires_at` into the ledger row
- rebuild replays from `row.expires_at`
- `at_f_02b_current_state_rebuild_preserves_micro_expiry` exists and executes
- the existing `at_f_01` / `at_f_02` / `at_f_03` family still passes
- the existing `at_m_db_04_rebuild_current_from_ledger` db-wiring proof still passes

Publication for the later implementation run is lawful only if the proof stays bounded to `ph1f.rs` plus the already-existing PH1.M db-wiring anchor. This H22 plan-freeze publication proof is narrower: repo truth now exposes one explicit Section 06 implementation winner, H22 freezes that winner, and broader Section 06 closure remains partial.

## Stop Conditions
Stop if repo truth no longer shows PH1.M deriving deterministic `expires_at` before persistence, the repo contract no longer carries `expires_at`, or PH1.F no longer accepts `expires_at` on append. Stop if the actual PH1.F constructor path or replay-with-`None` call site changes such that this seam is no longer exact.

Stop if the later implementation attempt requires a repo trait change, schema change, migration, or any edit outside `ph1f.rs`. Stop if the later proof widens into generalized retention workers, purge orchestration, Section 05 generalized closure, Section 07 or Section 11 identity work, Section 09 governance closure, or app-layer work. Stop if the later run tries to claim broader Section 06 completion instead of publishing only this first canonical seam.
