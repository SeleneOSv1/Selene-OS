# H359 - PH1.E Global Deterministic Time And Weather Location Normalization Build Plan

## Purpose

H359 is the narrow build-plan run after H358.

H358 proved one deterministic time happy path for New York in the live macOS desktop app. H358 did not generalize deterministic time location normalization or clean final-answer formatting for all supported places. Queries such as Japan and Sydney can still fall through to raw ISO-style output instead of a clean local-time answer because PH1.X currently has a New York-specific display helper.

Weather needs the same final-answer discipline, but weather is not the same class of deterministic primitive as time. Time can be computed from timezone rules; weather requires an existing lawful weather tool/provider lane. H359 must not invent fake weather, static weather, or a desktop-local weather answer.

This next seam is not shell redesign, PH1.D provider redesign, generic backend expansion, or protected-action law widening.

## Authority

- `/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H356_RUNTIME_ADAPTER_NATURAL_CONVERSATION_PH1D_PROVIDER_AND_GENERAL_ANSWER_LANE_BUILD_PLAN.md`
- `/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H357_APP_MAC_DESKTOP_LOW_RISK_PUBLIC_DETERMINISTIC_RUNTIME_COMPLETION_AND_VISIBLE_ANSWER_BUILD_PLAN.md`
- `/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H358_PH1E_DETERMINISTIC_TIME_EXECUTION_AND_DESKTOP_VISIBLE_ANSWER_COMPLETION_BUILD_PLAN.md`
- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1e.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1x.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1n.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_adapter/src/lib.rs`
- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift`
- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`
- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md`
- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md`

## Repo Truth Proof

- The highest committed H-series plan before this document is H358, so the next free implementation id is H359.
- H358 completed a live desktop happy path for `what is the time in New York`.
- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1e.rs` now routes `ToolName::Time` through `current_time_result_for_query(&req.query)`.
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1x.rs` currently formats `ToolResult::Time` through `time_tool_answer_text`, but the clean human display helper is New York-specific.
- A non-New-York supported time result can still render as raw ISO text through the fallback `It's {local_time_iso}.`
- PH1.N already contains deterministic `TimeQuery` and `WeatherQuery` classification paths.
- PH1.X already dispatches deterministic `TimeQuery` to `ToolName::Time` and `WeatherQuery` to `ToolName::Weather`.
- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1e.rs` currently returns `Weather snapshot for {query}` for `ToolName::Weather`, so the later implementation must prove whether a lawful weather provider/tool result is already available before claiming real weather acceptance.
- No repo truth justifies protected-action law widening for low-risk public time or weather answers.

## CURRENT

- New York time can now complete cleanly in the live desktop app.
- Some other supported places can still fall through to raw ISO-style output.
- Time location normalization and final-answer formatting are not yet generalized.
- Weather classification and dispatch exist, but weather is not yet generalized to the same clean deterministic final-answer standard.
- Current PH1.E weather behavior is not enough by itself to prove truthful live weather.
- No current proof justifies protected-action law widening.

## TARGET

- Time queries for supported places return clean local-time answers.
- Supported examples must include:
- New York -> clean local time.
- Japan -> clean local time.
- Sydney -> clean local time.
- Weather queries for supported places return clean final answers only if repo truth proves a lawful weather provider/tool lane is available.
- Supported weather examples must include at least two different places, selected from locations actually supported by repo truth.
- No raw ISO, raw payload, raw source dump, HTML entity leak, or `Retrieved at (unix_ms)` appears in final user responses.
- Ambiguous place names clarify cleanly instead of guessing or leaking raw data.
- Unsupported place names fail closed with a clean explanation instead of raw data.
- No fake local answer lane is introduced.

## GAP

This is not shell redesign, sidebar redesign, composer redesign, PH1.D redesign, generic backend expansion, identity/access/simulation weakening, or protected-action law widening.

The exact gap is generalizing deterministic place normalization and final-answer rendering for supported time and weather lanes while preserving the H358 desktop visible-answer completion path.

## In Scope

- Add or refine supported-location normalization for deterministic time queries.
- Render supported time answers as clean local human answers instead of raw ISO.
- Preserve PH1.N deterministic classification for time and weather.
- Preserve PH1.X deterministic dispatch to `ToolName::Time` and `ToolName::Weather`.
- Add clean handling for ambiguous and unsupported locations.
- Reuse the current H358 desktop visible-answer completion path.
- Add weather final-answer shaping only where repo truth proves a lawful weather tool/provider result exists.
- Add tests proving New York, Japan, and Sydney time answers are clean.
- Add tests proving weather final answers are clean for at least two supported places, if weather provider/tool support exists.
- Preserve protected-action fail-closed and simulation-first law.

## Out of Scope

- No shell redesign.
- No sidebar redesign.
- No composer redesign.
- No PH1.D provider redesign.
- No generic backend expansion.
- No protected-action law widening.
- No identity/access/simulation weakening.
- No fake location answers.
- No fake weather answers.
- No static demo weather.
- No adapter-local or desktop-local answer lane.
- No web-search fallback for deterministic time/weather unless the user explicitly asks to search the web.
- No iPhone implementation work.
- No DB wiring spec rewrite.

## Files Allowed To Change

- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1e.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1x.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1n.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_adapter/src/lib.rs`
- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift`
- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`

If and only if later implementation succeeds and repo law requires landed-truth updates:

- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md`
- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md`

## Files Not Allowed To Change

- PH1.D provider or contract files, unless repo truth proves unavoidable; if so, stop and report first.
- Backend route expansion files.
- DB wiring specs.
- iPhone source files.
- Unrelated desktop source files.
- Unrelated runtime law files.
- Any file outside the allowed scope unless repo truth proves unavoidable; if so, stop and report first.

## Acceptance Standard

- `what is the time in New York` returns a clean final answer in the live desktop app.
- `what is the time in Japan` returns a clean final answer in the live desktop app.
- `what is the time in Sydney` returns a clean final answer in the live desktop app.
- Each supported time answer includes a human local time and place label, not raw ISO.
- At least two supported weather queries return clean final answers in the live desktop app if lawful weather provider/tool support exists.
- Weather acceptance is blocked if the only available behavior is placeholder, fake, static, or raw payload output.
- Ambiguous locations produce a clean clarification or fail-closed explanation.
- Unsupported locations produce a clean fail-closed explanation.
- Final user answers do not expose raw ISO, raw payloads, raw source dumps, HTML entities, or `Retrieved at (unix_ms)`.
- Explicit web-search requests may still use web search as evidence, but deterministic time/weather must not use web search unless the user explicitly asks to search.
- Protected-action, identity/access, and simulation fail-closed behavior are not widened.

## Exact Test Commands

```sh
git -C /Users/selene/Documents/Selene-OS status --short
git -C /Users/selene/Documents/Selene-OS branch --show-current
git -C /Users/selene/Documents/Selene-OS rev-parse HEAD
cargo test -p selene_engines ph1n::tests::at_n_38_time_in_new_york_is_deterministic_time_query
cargo test -p selene_engines ph1e::tests::at_e_time_query_returns_current_new_york_time
cargo test -p selene_os ph1x::tests::at_x_dispatches_read_only_time_query_to_tool_router_and_sets_pending_tool
cargo test -p selene_adapter at_adapter_40_time_in_new_york_returns_clean_current_time_answer
xcodebuild -project /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop.xcodeproj -scheme SeleneMacDesktop -configuration Debug build
```

The implementation run must add exact new tests for:

- New York clean time formatting.
- Japan clean time formatting.
- Sydney clean time formatting.
- Ambiguous location handling.
- Unsupported location handling.
- Weather provider/tool availability proof.
- At least two clean weather answers, if weather support is lawfully available.
- Explicit web-search override preserving web search as evidence only.

## Required Live Desktop Proof

The implementation run must launch the actual macOS desktop app or the exact repo-supported desktop runtime path and prove:

1. `what is the time in New York` returns a clean final answer in the main pane.
2. `what is the time in Japan` returns a clean final answer in the main pane.
3. `what is the time in Sydney` returns a clean final answer in the main pane.
4. At least two weather queries for different supported places return clean final answers in the main pane, if weather provider/tool support is lawfully available.
5. None of the tested answers show raw ISO, raw payload, raw source dump, HTML entities, or `Retrieved at (unix_ms)`.
6. Desktop bridge logs show completion for each tested request.
7. The current H358 visible-answer completion path remains intact.
8. Screenshot proof captures the visible main pane after completion.

If the actual desktop app cannot be launched/tested, stop and report instead of substituting unit tests.

## Stop Conditions

- Stop if the next free id is not H359.
- Stop if the tree is dirty before implementation.
- Stop if supported time location normalization requires backend route expansion.
- Stop if weather requires PH1.D provider redesign.
- Stop if weather requires a new backend route.
- Stop if weather can only pass through fake, static, placeholder, or desktop-local answers.
- Stop if ambiguous-location handling would require guessing.
- Stop if unsupported-location handling would leak raw data.
- Stop if fixing this requires files outside the allowed scope.
- Stop if deterministic time/weather completion requires protected-action law widening.
- Stop if identity/access/simulation law changes appear necessary.
- Stop if live desktop app cannot be launched/tested.
- Stop if existing TTS or desktop visible-answer completion would require new runtime authority.

## Ledger Update Rule

Update `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md` and `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md` only after successful implementation and live desktop acceptance proof, if repo law requires landed-truth updates.
