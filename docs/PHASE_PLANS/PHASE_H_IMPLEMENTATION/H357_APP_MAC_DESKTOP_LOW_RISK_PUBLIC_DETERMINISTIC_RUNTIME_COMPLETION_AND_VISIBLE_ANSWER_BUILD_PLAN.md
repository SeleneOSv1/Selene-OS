# H357 - App Mac Desktop Low-Risk Public Deterministic Runtime Completion And Visible Answer Build Plan

## Purpose

H357 is the next narrow implementation plan after H356. It targets live desktop runtime completion for low-risk public and deterministic answers.

H356 improved PH1.N / PH1.X / PH1.D routing and shaping, but failed strict live product acceptance because it did not prove live desktop end-to-end completion for normal user conversation. The live blocker is desktop-to-runtime completion and visible answer return, not another shell redesign.

Low-risk public and deterministic answers must complete without widening protected-action law. Protected memory, personalization, permissions, and actions still require current identity/access law.

## Authority

- `/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H356_RUNTIME_ADAPTER_NATURAL_CONVERSATION_PH1D_PROVIDER_AND_GENERAL_ANSWER_LANE_BUILD_PLAN.md`
- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`
- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift`
- `/Users/selene/Documents/Selene-OS/crates/selene_adapter/src/lib.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1n.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1x.rs`
- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md`
- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md`
- `/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_L.md`
- `/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_VOICE_ID.md`
- `/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md`
- `/Users/selene/Documents/Selene-OS/docs/08_SIMULATION_CATALOG.md`

## Repo Truth Proof

- The highest existing H-series implementation plan before this document is `H356_RUNTIME_ADAPTER_NATURAL_CONVERSATION_PH1D_PROVIDER_AND_GENERAL_ANSWER_LANE_BUILD_PLAN.md`; therefore the next free implementation id is H357.
- H356 routing/shaping work can pass repo-level proof for PH1.N / PH1.X / PH1.D lanes, but strict live desktop product acceptance remains unproven.
- The live desktop can still show the runtime-bridge completion failure posture instead of a completed answer, including the shell-facing message equivalent to "I couldn't answer just now because the desktop runtime bridge did not complete the request."
- PH1.N contains deterministic time-query classification for questions such as "what is the time in New York".
- PH1.X contains dispatch from deterministic `TimeQuery` to `ToolName::Time` and time-tool result rendering.
- Desktop runtime bridge code carries typed-turn request completion, failure, and `responseText` decoding paths.
- Desktop shell code carries submitted user continuity, authoritative reply persistence, visible main-pane rendering, and existing TTS playback carriers.
- Existing proof does not justify protected-action law widening. Low-risk public/deterministic answers must not require Voice ID personalization authority.

## CURRENT

- H356 repo/tests may pass for routing and shaping.
- Strict live product acceptance still failed.
- Deterministic time query still does not reliably complete end-to-end in live desktop use.
- Live desktop can still show bridge/runtime completion failure instead of the answer.
- No current proof justifies protected-action law widening.
- Low-risk public/deterministic answers must not require Voice ID personalization authority.
- Current web-evidence proof must be kept separate from deterministic primitive answers and must not leak raw snippets, HTML entities, raw payloads, or `Retrieved at (unix_ms)` as final user answers.

## TARGET

- Low-risk public deterministic queries complete end-to-end in live desktop use.
- "what is the time in New York" returns a clean final answer in the main thread.
- Completed answers are not dropped at the desktop runtime bridge layer.
- Completed answers can use existing lawful TTS playback.
- Same-thread continuity and the current H350-H355 shell stay intact.
- No new protected-action authority is added.

## GAP

This is not shell redesign, PH1.D redesign alone, generic backend expansion, or protected-action identity/access widening.

The exact gap is live desktop request completion from runtime result to visible answer and lawful TTS. H357 must prove that completed low-risk public/deterministic runtime answers return through the desktop bridge and remain visible in the current main conversation pane.

## In Scope

- Completing low-risk public/deterministic desktop requests end-to-end from the visible desktop shell.
- Returning completed `TimeQuery` and public-answer results into the same main conversation pane.
- Making "what is the time in New York" show a clean final answer in the main thread.
- Ensuring the desktop runtime bridge does not drop, timeout, misclassify, or withhold completed low-risk answers.
- Preserving same-thread continuity and the current H350-H355 shell.
- Preserving existing TTS playback for completed answers when lawful.
- Preserving protected-action fail-closed and simulation-first law.
- Reusing existing PH1.N classification, PH1.X dispatch/shaping, PH1.D provider lane, adapter response payloads, desktop bridge completion carriers, and shell authoritative-reply carriers.
- Correcting low-risk public/deterministic identity handling only if current repo truth proves those answers are incorrectly blocked behind Voice ID personalization authority.
- Tightening final-answer output guards only where required to prevent raw search snippets, HTML entities, raw payloads, or `Retrieved at (unix_ms)` from being shown as the final answer.

## Out of Scope

- No shell redesign.
- No sidebar redesign.
- No composer redesign.
- No generic backend expansion.
- No new local answer lane.
- No fake local provider.
- No protected-action law widening.
- No identity/access/simulation law weakening.
- No wake-law changes.
- No session-law changes.
- No hidden/background wake work.
- No false "works" claim without live desktop proof.
- No separate recording lane.
- No iPhone implementation work.
- No DB wiring spec rewrite.
- No PH1.D redesign beyond unavoidable provider-contract mismatch proof.

## Files Allowed To Change

- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`
- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift`
- `/Users/selene/Documents/Selene-OS/crates/selene_adapter/src/lib.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1x.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1n.rs`

If and only if implementation succeeds and repo law requires landed-truth updates:

- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md`
- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md`

## Files Not Allowed To Change

- PH1.D contract files, unless repo truth proves a provider-contract mismatch is unavoidable; if so, stop and report first.
- Rust runtime law files outside the listed allowed scope.
- Backend route expansion files.
- DB wiring specs.
- iPhone source files.
- Unrelated desktop source files.
- Any other file unless repo truth proves it is unavoidable; if so, stop and report before editing.

## Acceptance Standard

H357 is accepted only when live desktop proof and non-live routing proof both pass.

Live desktop acceptance must prove:

- The actual macOS desktop app or exact repo-supported desktop runtime path launches successfully.
- The visible composer accepts the typed prompt `what is the time in New York`.
- Enter or the primary send control submits the prompt.
- The main pane shows a clean final answer equivalent to `It's [current local time] in New York.`
- The final answer is not a web-search dump.
- The final answer does not expose raw search snippets, HTML entities, raw source payloads, or `Retrieved at (unix_ms)`.
- Desktop bridge logs or visible runtime state prove request completion, not `desktop_runtime_bridge_failure` and not "runtime bridge did not complete the request."
- The same completed answer can be spoken through existing TTS when TTS is lawful and enabled.
- Screenshot proof captures the visible main pane after completion.
- Exact commands, logs, and screenshot proof are included.

If the actual desktop app cannot be launched/tested, the implementation run must stop and report. Unit tests alone must not be substituted for live desktop acceptance.

Non-live proof must prove:

- PH1.N classifies `what is the time in New York` as `TimeQuery`.
- PH1.X dispatches `TimeQuery` to `ToolName::Time`.
- PH1.X renders `ToolResult::Time` as a concise final answer.
- Explicit web-search requests still use web search only as evidence.
- Raw snippets, HTML entities, raw payloads, and `Retrieved at (unix_ms)` do not appear as the final answer.
- Protected-action, identity/access, and simulation fail-closed behavior are not widened.

## Exact Test Commands

Preflight:

```sh
git -C /Users/selene/Documents/Selene-OS status --short
git -C /Users/selene/Documents/Selene-OS branch --show-current
git -C /Users/selene/Documents/Selene-OS rev-parse HEAD
```

Repo truth inspection:

```sh
rg -n "desktopFriendlyRuntimeFailureMessage|desktopPersistAuthoritativeReplyIfNeeded|authoritative_reply_text|desktopConversationShouldSuppressDedicatedAuthoritativeReplyTextEntry" /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift
rg -n "desktopTypedTurnIngressRequestBuilder|submitPreparedTypedTurnRequest|completedTyped|failedTyped|VoiceTurnAdapterResponsePayload|responseText|desktop_runtime_bridge_failure" /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift
rg -n "TimeQuery|ToolName::Time|ToolResult::Time|Retrieved at \\(unix_ms\\)|I found supporting web evidence|Snippet" /Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1x.rs
```

Non-live proof tests:

```sh
cargo test -p selene_engines ph1n::tests::at_n_38_time_in_new_york_is_deterministic_time_query
cargo test -p selene_os ph1x::tests::at_x_dispatches_read_only_time_query_to_tool_router_and_sets_pending_tool
cargo test -p selene_os ph1x::tests::at_x_tool_ok_completes_pending_dispatch_into_respond
```

Mac desktop build:

```sh
xcodebuild -project /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop.xcodeproj -scheme SeleneMacDesktop -configuration Debug build
```

The implementation run must add any exact live launch/log commands required by current repo truth and must include their output.

## Required Live Desktop Proof

The later implementation run must perform this live test:

1. Launch the actual macOS desktop app or exact repo-supported desktop runtime path.
2. Confirm the desktop runtime/adapter path is healthy enough to accept a low-risk public deterministic request.
3. Type `what is the time in New York` into the visible composer.
4. Submit with Enter or the primary send control.
5. Prove the main pane shows a clean final answer equivalent to `It's [current local time] in New York.`
6. Prove the answer is not a web-search dump and does not contain raw snippets, HTML entities, raw source payloads, or `Retrieved at (unix_ms)`.
7. Prove bridge logs or runtime state show completion, not timeout/drop/failure.
8. Prove the same answer can be spoken through existing TTS when TTS is lawful and enabled.
9. Capture screenshot proof of the visible main pane after completion.
10. Stop and report if the actual desktop app cannot be launched/tested.

## Stop Conditions

- Stop if live desktop completion requires protected-action law widening.
- Stop if low-risk public answers require new identity/access authority instead of existing public-answer law.
- Stop if the desktop app cannot be launched or tested.
- Stop if the bridge failure cannot be reproduced or disproven from logs.
- Stop if fixing completion requires files outside the allowed scope.
- Stop if PH1.D contract changes appear necessary.
- Stop if backend route expansion appears necessary.
- Stop if TTS playback would require new runtime authority instead of existing carriers.
- Stop if the implementation would require a shell/sidebar/composer redesign.
- Stop if deterministic public answers can only be made to pass by adding a fake local answer lane or fake provider.
- Stop if protected-action, identity/access, simulation, wake, or session law would need to be weakened.

## Ledger Update Rule

Do not update landed-truth docs during the plan-only run.

During the later implementation run, update `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md` and `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md` only after successful code-bearing implementation and live desktop acceptance proof, if repo law requires landed-truth updates.
