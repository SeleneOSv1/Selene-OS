# H358 - PH1.E Deterministic Time Execution And Desktop Visible Answer Completion Build Plan

## Purpose

H358 is the narrow corrective build after H357. H357 stopped before code because repo truth proved the deterministic time engine was outside H357 allowed scope.

H357 must not be forced by adding a fake adapter-local or desktop-local answer lane. The next lawful seam is PH1.E deterministic time execution plus end-to-end desktop visible-answer completion.

This is not shell redesign, PH1.D redesign, backend expansion, or protected-action law widening.

## Authority

- `/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H357_APP_MAC_DESKTOP_LOW_RISK_PUBLIC_DETERMINISTIC_RUNTIME_COMPLETION_AND_VISIBLE_ANSWER_BUILD_PLAN.md`
- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1e.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1x.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/app_ingress.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_adapter/src/lib.rs`
- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift`
- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`
- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md`
- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md`

## Repo Truth Proof

- The highest committed H-series plan before this document is H357, so the next free implementation id is H358.
- H357 execution stopped cleanly before editing because the actual PH1.E time result lives in `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1e.rs`, outside H357 allowed scope.
- PH1.N already classifies `what is the time in New York` as `TimeQuery`.
- PH1.X already dispatches `TimeQuery` to `ToolName::Time`.
- PH1.E currently returns the fixed placeholder `2026-01-01T00:00:00Z` for `ToolName::Time`.
- Desktop bridge and shell already carry completed `responseText` into the visible thread when the runtime returns one.
- No repo truth justifies protected-action law widening for low-risk public deterministic time answers.

## CURRENT

- H357 authority doc is committed.
- H357 stopped before code because PH1.E was outside scope.
- Deterministic time classification and dispatch are already present.
- PH1.E currently returns a fixed placeholder time value.
- Live desktop acceptance cannot truthfully pass while PH1.E returns placeholder time.
- No current proof justifies protected-action law widening.

## TARGET

- PH1.E returns a truthful current deterministic time result for supported New York time queries.
- `what is the time in New York` completes end-to-end in the live macOS desktop app.
- The main pane shows a clean final answer, not an ISO placeholder and not a web-search dump.
- Desktop bridge logs prove request completion, not runtime bridge failure.
- Existing lawful TTS can speak the completed answer when enabled.
- No new protected-action authority is added.

## GAP

This is not shell redesign, sidebar redesign, composer redesign, PH1.D redesign alone, or generic backend expansion.

The gap is PH1.E deterministic time execution plus end-to-end desktop visible-answer proof.

## In Scope

- Replace the fixed PH1.E time placeholder with a deterministic current-time result for supported time queries.
- Preserve PH1.N `TimeQuery` classification.
- Preserve PH1.X dispatch of `TimeQuery` to `ToolName::Time`.
- Shape the final user-visible time answer as a clean concise response equivalent to `It's [current local time] in New York.`
- Return the completed result through app ingress, adapter response, desktop runtime bridge, and visible main pane.
- Preserve existing lawful TTS playback for the completed answer.
- Preserve protected-action fail-closed and simulation-first law.

## Out of Scope

- No fake local time answer in the adapter or desktop shell.
- No web search fallback for deterministic time.
- No PH1.D provider call for deterministic time.
- No protected-action law widening.
- No identity/access/simulation law weakening.
- No wake-law changes.
- No session-law changes.
- No shell/sidebar/composer redesign.
- No backend route expansion.
- No iPhone work.

## Files Allowed To Change

- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1e.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1x.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/app_ingress.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_adapter/src/lib.rs`
- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift`
- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`

If and only if implementation succeeds and repo law requires landed-truth updates:

- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md`
- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md`

## Files Not Allowed To Change

- PH1.E contract files, unless repo truth proves unavoidable; stop and report first.
- PH1.D contract files.
- Backend route expansion files.
- DB wiring specs.
- iPhone source files.
- Unrelated desktop source files.
- Unrelated runtime law files.

## Acceptance Standard

- `what is the time in New York` routes to `TimeQuery` and `ToolName::Time`.
- PH1.E returns current New York time, not the placeholder `2026-01-01T00:00:00Z`.
- PH1.X renders the time result as a concise final answer.
- The live desktop app shows the completed answer in the same main thread.
- The answer is not a web-search dump.
- The answer does not expose raw snippets, HTML entities, raw payloads, or `Retrieved at (unix_ms)`.
- Desktop bridge proof shows completion, not runtime bridge failure.
- Existing TTS can speak the answer when lawful and enabled.

## Exact Test Commands

```sh
git -C /Users/selene/Documents/Selene-OS status --short
cargo test -p selene_engines ph1n::tests::at_n_38_time_in_new_york_is_deterministic_time_query
cargo test -p selene_engines ph1e::tests::at_e_time_query_returns_current_new_york_time
cargo test -p selene_os ph1x::tests::at_x_dispatches_read_only_time_query_to_tool_router_and_sets_pending_tool
cargo test -p selene_os ph1x::tests::at_x_tool_ok_completes_pending_dispatch_into_respond
cargo test -p selene_adapter at_adapter_40_time_in_new_york_returns_clean_current_time_answer
xcodebuild -project /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop.xcodeproj -scheme SeleneMacDesktop -configuration Debug build
```

## Required Live Desktop Proof

The implementation run must:

1. Launch the actual macOS desktop app or exact repo-supported desktop runtime path.
2. Type `what is the time in New York`.
3. Submit with Enter or the primary send control.
4. Prove the main pane shows a clean final answer equivalent to `It's [time] in New York.`
5. Prove it is not a web-search dump.
6. Prove it is not the placeholder `2026-01-01T00:00:00Z`.
7. Prove desktop bridge logs show request completion, not runtime bridge failure.
8. Prove the same answer can be spoken through existing TTS when lawful and enabled.
9. Include screenshot proof of the visible main pane after completion.

## Stop Conditions

- Stop if truthful current time requires PH1.E contract changes.
- Stop if timezone/city support requires a new backend route.
- Stop if desktop completion requires protected-action authority.
- Stop if fixing this requires files outside the allowed scope.
- Stop if live desktop app cannot be launched/tested.
- Stop if the result can only pass by fabricating a desktop-local or adapter-local answer.
- Stop if TTS playback requires new runtime authority instead of existing carriers.

## Ledger Update Rule

Update `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md` and `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md` only after successful implementation and live desktop acceptance proof, if repo law requires landed-truth updates.
