# H356 Runtime Adapter Natural Conversation PH1D Provider And General Answer Lane Build Plan

## Purpose

Classify and document the next truthful post-H355 build.

H350 through H355 moved the native macOS shell from a visible chat surface into a same-thread typed and explicit-voice conversation shell. Live testing now proves the next blocker is not another desktop shell redraw. The desktop can capture text and voice, keep a visible thread, and call the existing submit seams, but Selene still cannot reliably hold a natural open-ended conversation because the runtime/adapter answer lane is not yet wired for broad public chat.

This build is therefore a runtime/adapter-scoped H implementation plan, not a desktop-shell H run.

The canonical repo path for this build and every later implementation command is:

- `/Users/selene/Documents/Selene-OS`

## Authority

This build is governed by:

- current preflight truth at HEAD `4499799201968102d35f946fa232cdd65d0cb8ac`
- existing implementation-plan convention under `/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/`
- H353 completion truth for pre-ready main-pane continuity in `/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H353_APP_MAC_DESKTOP_PRE_READY_TIME_HANDOFF_MAIN_PANE_CONTINUITY_BUILD_PLAN.md`
- H354 completion truth for same-thread input modality and composer alignment in `/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H354_APP_MAC_DESKTOP_SAME_THREAD_INPUT_MODALITY_CONTINUITY_AND_COMPOSER_ALIGNMENT_BUILD_PLAN.md`
- H355 completion truth for same-pane continuity and primary send-button behavior in `/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H355_APP_MAC_DESKTOP_CORRECTIVE_SAME_THREAD_CONTINUITY_AND_PRIMARY_SEND_BUTTON_ACCEPTANCE_FIX_BUILD_PLAN.md`
- PH1.N runtime intent classification in `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1n.rs`
- PH1.X answer shaping and tool dispatch in `/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1x.rs`
- app ingress voice/text/session handoff in `/Users/selene/Documents/Selene-OS/crates/selene_os/src/app_ingress.rs`
- PH1.D adapter availability in `/Users/selene/Documents/Selene-OS/crates/selene_adapter/src/lib.rs`
- PH1.L deterministic session lifecycle law in `/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_L.md`
- PH1.VOICE.ID speaker verification law in `/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_VOICE_ID.md`
- PH1.ACCESS fail-closed access law in `/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md`
- simulation-gated execution law in `/Users/selene/Documents/Selene-OS/docs/08_SIMULATION_CATALOG.md`

## Repo Truth Proof

Current repo truth proves all of the following:

- H350-H355 desktop shell work is already the correct visible UI baseline for typed and explicit voice continuity.
- The next free H implementation id is `H356`.
- Existing plan convention places runtime, adapter, and app implementation plans under `PHASE_H_IMPLEMENTATION`; therefore the correct next plan path remains in that family while the scope is runtime/adapter, not desktop UI.
- PH1.N still emits the no-intent fallback text `"Okay. What would you like to do?"` in `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1n.rs`.
- PH1.N currently routes broad general question shapes through `looks_like_general_web_question(...)` into `IntentType::WebSearchQuery` in `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1n.rs`.
- PH1.X currently handles `IntentType::WebSearchQuery` through tool/search dispatch and `search_tool_ok_text(...)` shaping in `/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1x.rs`.
- The current web-search answer lane can make some public questions visible, but it is not a natural general answer lane and can produce poor, source-snippet-shaped answers.
- The adapter's `EnvPh1dLiveAdapter::execute(...)` currently returns an unavailable terminal error in `/Users/selene/Documents/Selene-OS/crates/selene_adapter/src/lib.rs`.
- The adapter only attempts to build a PH1.D live adapter from environment state around `SELENE_PH1D_LIVE_PROVIDER_ID`, but the current live adapter body still does not call a real provider.
- Existing dirty edits in `/Users/selene/Documents/Selene-OS/crates/selene_os/src/app_ingress.rs` and `/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1x.rs` are runtime edits, not desktop UI edits, and must not be committed during this doc-only classification run.

Therefore:

- the next truthful build is not another H desktop-shell run
- the next truthful build is a runtime/adapter H implementation run
- PH1.D adapter wiring is the hard next blocker for provider-backed natural open-ended answers
- PH1.N broad public-question classification and PH1.X answer-lane shaping must be corrected in the same bounded runtime/adapter seam

## Build Family Classification

Correct build family:

- `PHASE_H_IMPLEMENTATION`, runtime/adapter-scoped H356

Not selected:

- APP_MAC_DESKTOP-only H run
- shell redesign run
- new runtime-law series outside current repo plan convention
- protected-action identity expansion run

Reason:

- the visible desktop shell is no longer the first blocker
- broad public/open-ended prompts are still classified or shaped through web-search/no-intent paths rather than a real natural answer lane
- the live PH1.D provider adapter is unavailable
- the future implementation must touch runtime/adapter files, which proves this is not desktop-only

## CURRENT

Current repo truth is:

- the macOS desktop shell can capture typed turns and explicit voice turns
- H353-H355 keep typed and voice continuity visible in the main pane much better than the prior shell
- broad public questions can still route into `IntentType::WebSearchQuery` through PH1.N general-question heuristics
- unmatched or failed classification can still surface the `"Okay. What would you like to do?"` no-intent posture
- PH1.X currently relies on tool/search answer shaping for public web-search responses
- the live PH1.D provider adapter is unavailable in the adapter layer
- existing dirty `app_ingress.rs` changes try to preserve low-risk public responses through identity fail-closed boundaries
- existing dirty `ph1x.rs` changes try to improve low-value web-search answer shaping
- typed-only protected identity fallback through Face ID, Touch ID, or passcode is not yet a bounded implemented protected-action seam

## TARGET

The exact H356 target is:

- route broad low-risk public/open-ended chat prompts into a real lawful general answer lane instead of forcing them into no-intent fallback or web-search-only behavior
- wire the existing PH1.D live provider bridge so provider-backed natural answers can be produced lawfully when configuration is present
- preserve PH1.N intent classification discipline while separating general public answer intent from web-search-specific intent
- preserve PH1.X answer shaping so provider answers, tool answers, failures, and provenance remain distinguishable
- preserve one lawful conversation lane across typed and explicit voice turns for low-risk public/general chat
- preserve PH1.VOICE.ID and access-law boundaries for speaker-personalized, memory-scoped, permissioned, or protected actions
- preserve simulation-law fail-closed behavior
- keep typed-only device identity fallback as a later protected-action build unless the future H356 code proof shows a minimal carrier split is unavoidable
- avoid desktop shell redesign

## GAP

This gap is not:

- sidebar redesign
- composer redesign
- message bubble redesign
- wake-law widening
- session-law widening
- identity/access/simulation-law weakening
- protected-action expansion

This gap is specifically:

- the runtime/adapter natural conversation answer lane is incomplete because broad public/open-ended prompts still rely on PH1.N web-search/no-intent routing and PH1.X search/fallback shaping while the PH1.D live provider adapter is unavailable.

## Blocker Diagnosis

Why Selene still cannot hold a natural open-ended conversation:

- PH1.N currently treats many broad public questions as web-search queries.
- PH1.N still has a no-intent fallback that produces `"Okay. What would you like to do?"` instead of a natural answer.
- PH1.X can shape tool/search results, but web-search snippets are not the same as a general answer engine.
- PH1.D exists as the lawful provider-facing answer seam, but the adapter live provider implementation currently returns unavailable.
- Desktop message continuity can show what the runtime returns, but it cannot invent the missing answer lane without violating scope.

Whether broad public questions are still being routed into `WebSearchQuery` or no-intent fallback:

- Yes. Current `ph1n.rs` routes broad general question patterns through `looks_like_general_web_question(...)` into `IntentType::WebSearchQuery`.
- The no-intent fallback remains present and still returns `"Okay. What would you like to do?"` when no classified answer path wins.

Whether missing live PH1.D provider bridge is a hard blocker:

- Yes. It is the hard blocker for real provider-backed natural answers.
- Web-search answer shaping can improve visibility and citation quality, but it cannot replace the missing PH1.D live answer bridge.

Whether typed and voice can share one lawful conversation lane without widening protected-action law:

- Yes, for low-risk public/general chat continuity.
- No protected memory, personalization, permissioned action, or identity-scoped behavior may unlock from that shared lane without the existing PH1.VOICE.ID or device-auth law.

Whether typed-only Face ID, Touch ID, or passcode fallback belongs in this same build:

- No. It should be kept as a later separate protected-action build.
- Typed-only device identity fallback affects desktop and iPhone capability surfaces plus protected memory/permission boundaries, so it must not be smuggled into H356.

## Uncommitted Runtime Edit Classification

Current dirty edits in `/Users/selene/Documents/Selene-OS/crates/selene_os/src/app_ingress.rs`:

- classification: keep the substance, rework in H356
- reason: the changes belong to runtime conversation continuity and low-risk public response visibility, not desktop UI
- constraint: the future code run must prove the changes do not widen protected-action identity, access, memory, or simulation law
- doc-only rule: do not commit these edits in this run

Current dirty edits in `/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1x.rs`:

- classification: keep the substance, rework in H356
- reason: the changes improve search answer shaping but are not the complete natural-answer fix
- constraint: the future code run must pair PH1.X shaping with a lawful provider-backed PH1.D answer lane and clear PH1.N classification
- doc-only rule: do not commit these edits in this run

The dirty runtime pass should not be discarded wholesale, but it should not be accepted as final without H356 review, tests, and PH1.D adapter wiring.

## In Scope

The later H356 implementation run may:

- implement or complete the existing PH1.D live provider bridge in `/Users/selene/Documents/Selene-OS/crates/selene_adapter/src/lib.rs`
- correct PH1.N broad public/open-ended classification so general answer prompts are not forced into web-search-only or no-intent fallback behavior
- correct PH1.X answer dispatch and shaping so provider-backed general answers, web-search answers, tool answers, and failures remain distinct
- preserve low-risk public/general chat continuity through app ingress across typed and explicit voice turns
- keep the useful portions of the current dirty `app_ingress.rs` low-risk public response visibility work if tests prove it remains lawful
- keep the useful portions of the current dirty `ph1x.rs` answer-shaping work if tests prove it remains lawful
- add or adjust runtime/adapter tests that prove natural general answers route through the lawful lane
- update landed-truth docs only after successful code-bearing implementation if repo law requires it

## Out Of Scope

- no desktop shell redesign
- no sidebar redesign
- no composer redesign
- no hidden/background wake work
- no wake-law changes
- no session-law changes
- no identity/access/simulation-law weakening
- no protected-action widening
- no fake local answer lane outside the existing runtime/adapter authority model
- no fake PH1.D provider success when the provider is unavailable
- no generic memory or personalization unlock
- no typed-only Face ID, Touch ID, or passcode protected-action implementation in H356
- no iPhone UI implementation
- no long-form recording lane
- no broad tool/search expansion beyond what is required to separate web search from general answer routing

## Files Allowed To Change

The later H356 implementation run may change only the files that repo truth proves are necessary from this bounded set:

- `/Users/selene/Documents/Selene-OS/crates/selene_adapter/src/lib.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1n.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1x.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/app_ingress.rs`

If current contracts lack the exact carrier needed for a lawful general answer lane, the later code run may stop and report, or may change only the minimum proven contract files:

- `/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1n.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1x.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1d.rs`

If and only if later implementation succeeds and repo law requires landed-truth updates:

- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md`
- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md`

## Files Not Allowed To Change

- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`
- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift`
- any iPhone source file
- any DB wiring spec
- any simulation-law spec
- any wake-law spec
- any unrelated runtime, adapter, storage, or UI file unless repo truth proves it is unavoidable, in which case the later implementation run must stop and report before drifting

## Acceptance Standard

The later H356 implementation run passes only if all of the following are true:

- broad public/open-ended prompts no longer fall to `"Okay. What would you like to do?"` when a lawful general answer lane is available
- broad public/open-ended prompts are no longer forced into web-search-only behavior when no explicit web-search intent is present
- explicit web-search prompts still route to web search with provenance where appropriate
- PH1.D live provider bridge produces a lawful provider-backed answer when configured
- PH1.D live provider unavailability remains explicit, fail-closed, and non-fake
- typed and explicit voice turns can remain in one low-risk public/general conversation lane without unlocking protected memory, personalization, permissions, or actions
- existing PH1.VOICE.ID, PH1.ACCESS, PH1.L, and simulation-law boundaries remain intact
- current dirty `app_ingress.rs` and `ph1x.rs` work is either lawfully integrated with tests or consciously reduced/reworked
- no desktop shell redesign is introduced

## Exact Test Commands

The later H356 implementation run must run the relevant subset and add narrower tests where needed:

- `git -C /Users/selene/Documents/Selene-OS status --short`
- `git -C /Users/selene/Documents/Selene-OS diff --name-only`
- `rg -n "Okay\\. What would you like to do\\?|looks_like_general_web_question|WebSearchQuery|search_tool_ok_text" /Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1n.rs /Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1x.rs`
- `rg -n "EnvPh1dLiveAdapter|SELENE_PH1D_LIVE_PROVIDER_ID|build_ph1d_live_adapter_from_env" /Users/selene/Documents/Selene-OS/crates/selene_adapter/src/lib.rs`
- `cargo test -p selene_engines ph1n --quiet`
- `cargo test -p selene_os ph1x --quiet`
- `cargo test -p selene_os run_a --quiet`
- `cargo test -p selene_adapter --lib ph1d --quiet`
- `cargo build -p selene_adapter --bin selene_adapter_http`
- `xcodebuild -project /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop.xcodeproj -scheme SeleneMacDesktop -configuration Debug build`
- manual desktop proof: type a public general question such as `What is 2+2?` and confirm Selene produces a real answer instead of `"Okay. What would you like to do?"`
- manual desktop proof: ask a broad public open-ended prompt and confirm it uses the lawful answer lane rather than web-search-only fallback unless web search is explicitly selected
- manual desktop proof: ask an explicit web-search prompt and confirm web-search provenance remains visible
- manual desktop proof: use explicit voice for the same public general question and confirm the same conversation lane receives the answer without protected-action widening
- `git -C /Users/selene/Documents/Selene-OS diff --stat`
- `git -C /Users/selene/Documents/Selene-OS status --short`

## Stop Conditions

Stop immediately in the later H356 implementation run if any of the following becomes true:

- a real natural general answer lane would require weakening PH1.VOICE.ID, PH1.ACCESS, PH1.L, or simulation law
- PH1.D provider success cannot be truthfully implemented without a fake provider or fake answer
- broad public answer routing would require a desktop shell redesign
- typed and voice same-lane continuity would require protected-action widening
- typed-only Face ID, Touch ID, or passcode fallback becomes necessary to complete H356
- contract changes become broader than a minimal general-answer carrier split
- any file outside the allowed scope becomes necessary without a new stop-and-report decision
- the current dirty runtime pass cannot be reduced to lawful bounded runtime/adapter changes

## Ledger Update Rule

No landed-truth update is allowed during this doc-only planning run.

If a later H356 implementation run succeeds, landed-truth updates must:

- state that H356 is runtime/adapter-scoped, not a desktop-shell redesign
- preserve H350-H355 desktop shell truth unchanged
- state exactly how broad public/open-ended PH1.N classification changed
- state exactly how PH1.X answer shaping distinguishes provider answers, web-search answers, tool answers, and failures
- state exactly how PH1.D live provider availability is wired and how unavailable provider state remains fail-closed
- state that typed and voice can share one low-risk public/general conversation lane without protected-action widening
- state that typed-only Face ID, Touch ID, or passcode fallback remains a later protected-action build
- state that no simulation-law weakening, protected-action widening, or fake local answer lane was introduced
