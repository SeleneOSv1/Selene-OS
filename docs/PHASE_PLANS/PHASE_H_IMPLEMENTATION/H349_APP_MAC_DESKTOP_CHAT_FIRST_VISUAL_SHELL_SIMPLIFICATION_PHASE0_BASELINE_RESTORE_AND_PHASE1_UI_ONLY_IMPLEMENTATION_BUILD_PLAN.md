# H349 APP_MAC_DESKTOP Chat-First Visual Shell Simplification Phase 0 Baseline Restore And Phase 1 UI-Only Implementation Build Plan

## Purpose

Implement one narrow APP_MAC_DESKTOP visual-shell-only build that converts the default native macOS desktop shell into a clean chat-first Selene conversation surface without changing backend routes, wake law, session law, identity law, access law, simulation law, or other runtime authority boundaries.

This build is intentionally split into two approved execution slices only:

- Phase 0: discard any unapproved local scratch shell diff and confirm a clean approved baseline
- Phase 1: visual-shell UI simplification only

All later phases remain blocked until separate approval.

## Authority

This build is governed by the current architecture and desktop residual truth, including:

- global runtime gate order in [/Users/selene/Documents/Selene-OS/docs/01_ARCHITECTURE.md#L9](/Users/selene/Documents/Selene-OS/docs/01_ARCHITECTURE.md#L9)
- full Selene MVP voice/runtime pipeline in [/Users/selene/Documents/Selene-OS/docs/01_ARCHITECTURE.md#L59](/Users/selene/Documents/Selene-OS/docs/01_ARCHITECTURE.md#L59)
- authoritative engine ownership in [/Users/selene/Documents/Selene-OS/docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md#L81](/Users/selene/Documents/Selene-OS/docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md#L81)
- session lifecycle truth in [/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_L.md#L6](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_L.md#L6)
- wake runtime truth in [/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_W.md#L15](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_W.md#L15)
- voice identity fail-closed truth in [/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_VOICE_ID.md#L111](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_VOICE_ID.md#L111)
- simulation-first protected execution law in [/Users/selene/Documents/Selene-OS/docs/01_ARCHITECTURE.md#L18](/Users/selene/Documents/Selene-OS/docs/01_ARCHITECTURE.md#L18) and [/Users/selene/Documents/Selene-OS/docs/08_SIMULATION_CATALOG.md#L5](/Users/selene/Documents/Selene-OS/docs/08_SIMULATION_CATALOG.md#L5)
- H259 desktop closure-program discipline in [/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H259_APP_MAC_DESKTOP_FIRST_POST_H347_IMPLEMENTATION_FIRST_CLOSURE_PROGRAM_FOR_REMAINING_REAL_OPERATION_FAMILIES_BUILD_PLAN.md#L62](/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H259_APP_MAC_DESKTOP_FIRST_POST_H347_IMPLEMENTATION_FIRST_CLOSURE_PROGRAM_FOR_REMAINING_REAL_OPERATION_FAMILIES_BUILD_PLAN.md#L62)

## Repo Truth Proof

Primary desktop shell entry and runtime bridge truth currently live at:

- desktop shell root view state and entry body in [/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L4616](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L4616)
- desktop shell task/open-url orchestration in [/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L4677](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L4677)
- current desktop explicit voice surface in [/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L5674](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L5674)
- current desktop keyboard composer surface in [/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L5986](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L5986)
- current authoritative reply and provenance rendering in [/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L15110](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L15110)
- current explicit voice request dispatch in [/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L16371](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L16371)
- current wake-triggered voice request dispatch in [/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L16449](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L16449)

Lawful session reopen behavior is already route-specific and must remain route-specific:

- exact current-visible attach path in [/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift#L1650](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift#L1650)
- exact soft-closed explicit resume path in [/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift#L1779](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift#L1779)
- exact suspended recover path in [/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift#L1929](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift#L1929)
- shell-side route selection remains exact and bounded in [/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L16115](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L16115) and [/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L16178](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L16178)

Current desktop wake residual truth remains partial and must not be overclaimed:

- residual family selection preserves broader hidden/background wake auto-start and broader wake parity in [/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H259_APP_MAC_DESKTOP_FIRST_POST_H347_IMPLEMENTATION_FIRST_CLOSURE_PROGRAM_FOR_REMAINING_REAL_OPERATION_FAMILIES_BUILD_PLAN.md#L22](/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H259_APP_MAC_DESKTOP_FIRST_POST_H347_IMPLEMENTATION_FIRST_CLOSURE_PROGRAM_FOR_REMAINING_REAL_OPERATION_FAMILIES_BUILD_PLAN.md#L22)
- Program D remains broader hidden/background wake auto-start plus wake parity in [/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H259_APP_MAC_DESKTOP_FIRST_POST_H347_IMPLEMENTATION_FIRST_CLOSURE_PROGRAM_FOR_REMAINING_REAL_OPERATION_FAMILIES_BUILD_PLAN.md#L56](/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H259_APP_MAC_DESKTOP_FIRST_POST_H347_IMPLEMENTATION_FIRST_CLOSURE_PROGRAM_FOR_REMAINING_REAL_OPERATION_FAMILIES_BUILD_PLAN.md#L56)
- current native macOS wake auto-start is visible active-shell only in [/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L335](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L335) through [/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L346](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L346)

Planning preflight also proved one local unapproved scratch shell diff had existed in `DesktopSessionShellView.swift` before this plan was authored. Phase 0 therefore exists to guarantee that no approved implementation begins from any local scratch presentation experiment or dirty shell baseline.

## CURRENT

Current repo truth exposes a working but engineering-heavy native desktop shell:

- explicit voice capture exists, but it is surfaced as a technical operational card rather than a normal conversation interface
- typed-turn, search, tool, provenance, posture, history, wake, and support surfaces are visible by default
- lawful session attach / resume / recover already exist, but the default visual presentation does not feel like a calm user-first conversation product
- desktop wake behavior is still bounded to visible active-shell form only and does not lawfully claim full hidden/background hands-free parity

## TARGET

The target for this build is one clean chat-first Selene desktop shell:

- same conversation-first light visual rhythm as a clean ChatGPT-style desktop chat surface
- one main transcript, user on the right and Selene on the left
- one bottom composer for text
- voice and text visible inside the same conversation thread
- blank start only when there is no lawful session to reopen
- if there is a lawful active / resumable / recoverable session, reopen that lawful thread instead of always starting blank
- no default engineering clutter, no default diagnostic side rails, no default provenance/debug panels
- subtle state only: idle, listening, thinking, speaking

## GAP

The gap is a shell-presentation gap, not a runtime-authority gap:

- the current desktop shell exposes too much engineering and operational detail by default
- conversation history, settings, and debug support are not sufficiently separated from the normal user conversation surface
- the default layout does not yet present one clean transcript-first shell even though the underlying lawful conversation paths already exist

This build closes only that shell-presentation gap.

## In Scope

- discard any unapproved local scratch UI diff and restore the approved baseline
- simplify the default desktop shell into one transcript-first chat surface
- move history behind secondary navigation
- move settings behind app settings or equivalent secondary navigation
- move debug/engineering surfaces behind explicit developer/debug mode only
- preserve one lawful session-aware launch behavior:
  - blank only when no lawful session exists
  - reopen lawful attach / resume / recover thread when lawful session context exists
- preserve one unified visible conversation thread for typed turns and spoken turns
- keep subtle non-technical user state affordances only

## Out of Scope

- backend route changes
- runtime bridge expansion
- wake-law changes
- session-law changes
- attach / resume / recover semantic changes
- identity logic changes
- access logic changes
- simulation logic changes
- tool-routing logic changes
- search-routing logic changes
- new hidden/background wake claim
- new wake parity claim
- speaker-identification redesign
- new memory/persona behavior
- new session-selection authority
- new generic reopen authority

## Files Allowed To Change

Phase 0 and Phase 1 future code execution may change only:

- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`

After successful completion only, landed-truth docs may be updated if repo law requires:

- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md`
- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md`

## Files Not Allowed To Change

- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift`
- any Rust runtime crate
- any backend adapter/server file
- any iPhone source file
- any search/tool/identity/access/simulation doc or contract file
- any phase-plan file other than this one during the future implementation run

## Phase 0

Phase 0 is mandatory baseline cleanup:

- discard the prior unapproved scratch UI diff
- confirm the repo is on a clean approved baseline
- confirm no accidental presentation experiment remains in `DesktopSessionShellView.swift`
- stop immediately if baseline restoration would require changes outside the one desktop shell file

Phase 0 is not feature work.
Phase 0 is baseline restoration only.

## Phase 1

Phase 1 is visual-shell UI only:

- convert the default shell into one clean transcript-first desktop conversation surface
- hide engineering rails/cards from the default user view
- place history into secondary navigation
- place settings into secondary navigation
- place debug/runtime surfaces into developer/debug mode only
- keep existing lawful attach / resume / recover behavior intact
- keep existing voice, typed-turn, search, tool, provenance, and playback runtime behaviors intact
- allow only shell composition, view hierarchy, user-facing labeling, spacing, and presentation simplification changes

Phase 1 must not widen authority.

## Blocked Later Phases

All later phases are blocked until separate approval:

- full hands-free desktop wake claims
- hidden/background wake behavior
- wake parity claims
- speaker-identification changes in the live wake flow
- per-user access-flow redesign
- identity-scoped personalization changes
- broader session reopen behavior
- any runtime bridge or backend route work

## Acceptance Standard

The future implementation run passes only if all of the following are true:

- the default user-facing desktop shell opens as a clean chat-first Selene conversation surface
- no default right-side or left-side engineering clutter remains visible
- voice and text appear in the same visible conversation thread
- history/settings/debug are no longer dumped into the default shell and are reachable only through secondary navigation or developer/debug mode
- lawful session reopen behavior is preserved exactly
- no backend route change is introduced
- no wake-law change is introduced
- no session-law change is introduced
- no identity/access/simulation logic change is introduced
- no hidden/background wake parity claim is introduced

## Exact Test Commands

- `git -C /Users/selene/Documents/Selene-OS status --short`
- `git -C /Users/selene/Documents/Selene-OS diff --name-only`
- `rg -n "struct DesktopSessionShellView|var body|desktopOperationalConversationShell|desktopTypedTurnComposerCard|historyCard|explicitVoiceEntryAffordanceCard" /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`
- `rg -n "sessionAttachEndpoint|sessionResumeEndpoint|sessionRecoverEndpoint|voiceTurnEndpoint" /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift`
- `xcodebuild -project /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop.xcodeproj -scheme SeleneMacDesktop -configuration Debug build`
- `git -C /Users/selene/Documents/Selene-OS diff --stat`
- `git -C /Users/selene/Documents/Selene-OS status --short`

## Stop Conditions

Stop immediately if any of the following becomes true during the future implementation run:

- any code change is needed outside `DesktopSessionShellView.swift`
- any runtime bridge change appears necessary
- any backend route change appears necessary
- any wake-law change appears necessary
- any session-law change appears necessary
- any identity/access/simulation logic change appears necessary
- preserving lawful attach / resume / recover behavior cannot be achieved through shell presentation work alone
- the implementation begins to imply or claim full hidden/background wake parity
- the implementation begins to imply generic reopen authority

## Ledger Update Rule

No ledger update is allowed during Phase 0 baseline cleanup alone.

If a later approved Phase 1 implementation run succeeds, landed-truth updates must:

- record only the exact H349 visual-shell-only desktop shell simplification landing
- preserve all existing H259/H348/H324 truth unchanged
- explicitly state that no backend route, wake-law, session-law, identity, access, or simulation logic changed
- explicitly state that later phases remain blocked pending separate approval
