# H350 APP_MAC_DESKTOP Chat-First Visible Sidebar And Main Composer Alignment Correction Build Plan

## Purpose

Document one narrow corrective visual-shell alignment build for APP_MAC_DESKTOP after the successful completion of H349.

H349 is complete for its own approved scope, but H349 is not the final approved visual target. The new user-approved reference image attached in the correction request now overrides the older H349 assumption that history should move behind a compact button or drawer.

H350 is now the execution authority for the corrective UI alignment run.

This build exists only to correct the visual-shell target so a later implementation run can align the desktop shell with the approved reference without changing backend routes, wake law, session law, identity law, access law, simulation law, or other runtime authority boundaries.

The canonical repo path for this correction build and every command in the later implementation run is:

- `/Users/selene/Documents/Selene-OS`

## Authority

This correction build is governed by:

- H349 completion truth in [/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L332](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L332)
- H349 completion truth in [/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md#L674](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md#L674)
- the still-live H349 implementation boundary in [/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H349_APP_MAC_DESKTOP_CHAT_FIRST_VISUAL_SHELL_SIMPLIFICATION_PHASE0_BASELINE_RESTORE_AND_PHASE1_UI_ONLY_IMPLEMENTATION_BUILD_PLAN.md#L155](/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H349_APP_MAC_DESKTOP_CHAT_FIRST_VISUAL_SHELL_SIMPLIFICATION_PHASE0_BASELINE_RESTORE_AND_PHASE1_UI_ONLY_IMPLEMENTATION_BUILD_PLAN.md#L155)
- the same route-specific attach / resume / recover law already proven in H349 at [/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H349_APP_MAC_DESKTOP_CHAT_FIRST_VISUAL_SHELL_SIMPLIFICATION_PHASE0_BASELINE_RESTORE_AND_PHASE1_UI_ONLY_IMPLEMENTATION_BUILD_PLAN.md#L43](/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H349_APP_MAC_DESKTOP_CHAT_FIRST_VISUAL_SHELL_SIMPLIFICATION_PHASE0_BASELINE_RESTORE_AND_PHASE1_UI_ONLY_IMPLEMENTATION_BUILD_PLAN.md#L43)
- the same partial desktop wake truth already proven in H349 at [/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H349_APP_MAC_DESKTOP_CHAT_FIRST_VISUAL_SHELL_SIMPLIFICATION_PHASE0_BASELINE_RESTORE_AND_PHASE1_UI_ONLY_IMPLEMENTATION_BUILD_PLAN.md#L50](/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H349_APP_MAC_DESKTOP_CHAT_FIRST_VISUAL_SHELL_SIMPLIFICATION_PHASE0_BASELINE_RESTORE_AND_PHASE1_UI_ONLY_IMPLEMENTATION_BUILD_PLAN.md#L50)

Visual authority for this correction build is:

- the newly attached user-approved reference image in the correction request
- the current live implemented H349 shell screenshot as the failing result

Override note for this correction build:

- any older brief language that hides history behind a compact button or drawer is now overridden by the new visual authority
- the sidebar must be visible by default

## Repo Truth Proof

Current H349 shell structure now lives in:

- evidence-first default shell at [/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L4848](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L4848)
- top chrome with `History`, `Setup` / `Controls`, and `More` entry points at [/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L4890](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L4890)
- secondary history / workspace / developer presentation at [/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L4969](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L4969)
- operational conversation shell at [/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L10585](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L10585)
- current transcript-first primary pane at [/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L10603](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L10603)
- current composer at [/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L5890](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L5890)
- current conversation bubble rendering at [/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L11077](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L11077)

No runtime bridge change is required or authorized for this correction build.

## CURRENT

H349 truth is complete and remains preserved:

- the shell is cleaner than before
- engineering clutter is no longer dumped directly into one default support rail
- history, controls, and developer detail were moved into secondary presentation
- typed and spoken turns remain in the same visible thread
- spoken TTS reply remains preserved as the spoken form of the same assistant turn

But the current implemented shell is still visually wrong against the approved target:

- no visible left sidebar is shown by default
- conversation history is not visible in a persistent sidebar
- the main pane can open with large placeholder explanation cards instead of a ready chat surface
- the default shell uses top-right `History`, `Setup` / `Controls`, and `More` as the primary structure
- the default empty-state shell does not show the main composer ready at the bottom

## TARGET

The corrected visual target is:

- a visible left sidebar by default, serving the same structural role as the ChatGPT desktop sidebar
- conversation/session history visible in that left sidebar by default
- a main transcript-first conversation pane
- a visible bottom composer/input area in the main pane by default
- no large placeholder explanation cards in the normal default shell
- no top-right `History` / `Setup` / `More` buttons as the primary shell structure
- same ChatGPT-like light visual structure, spacing rhythm, and overall compositional hierarchy as the approved reference image
- no engineering wording in the normal user view
- user messages on the right and Selene messages on the left
- if no lawful session exists, show a clean empty conversation view with the composer ready
- if a lawful session exists, reopen that lawful thread in the main pane
- typed and spoken turns remain in the same visible thread
- spoken TTS reply remains preserved as the spoken form of the same assistant turn

## GAP

The gap is now a narrower corrective visual alignment gap inside the desktop shell:

- H349 removed clutter, but it hid history instead of making the sidebar visible by default
- H349 preserved the transcript/composer path, but the default empty-state visual still behaves like a placeholder shell instead of a ready chat surface
- H349 replaced the old clutter with top-right structural buttons, which the approved target now rejects
- H349 kept the conversation chrome too sparse and too explanation-card-driven for the approved reference

## Exact Visual Mismatches

The exact mismatches between the current implementation and the new approved reference are:

- current implementation has no default visible left sidebar
- current implementation does not render history/session list as a persistent left sidebar
- current implementation uses top-right `History`, `Setup` / `Controls`, and `More` buttons as primary navigation
- current implementation shows large placeholder explanation cards in the default shell
- current implementation does not show the bottom composer ready in the clean empty default view
- current implementation still treats setup/controls secondary surfaces as first-class shell structure instead of keeping the default experience centered on sidebar + transcript + composer

## In Scope

The later corrective implementation run may only:

- reshape the desktop shell into a visible-left-sidebar plus main-transcript plus bottom-composer layout
- render history/session list in the default visible left sidebar
- remove the top-right `History`, `Setup` / `Controls`, and `More` shell structure from the default view
- remove the large placeholder explanation cards from the normal default shell
- align the default shell to the same ChatGPT-like light visual structure and rhythm shown in the approved reference image
- keep the main composer visible and ready in the empty default view
- preserve lawful thread reopen behavior when a lawful active / resumable / recoverable session exists
- preserve typed-turn and spoken-turn rendering in the same thread
- preserve TTS as the spoken form of the same assistant turn
- keep engineering surfaces out of the normal user view

## Out Of Scope

- backend route changes
- runtime bridge changes
- wake-law changes
- session-law changes
- attach / resume / recover semantic changes
- identity logic changes
- access logic changes
- simulation logic changes
- search-routing logic changes
- tool-routing logic changes
- hidden/background wake claims
- wake parity claims
- any false wake-parity claim
- speaker-identification redesign
- access-flow redesign
- broader history/search/tool/session authority expansion
- any work outside visual-shell correction

## Files Allowed To Change

The later corrective implementation run may change only:

- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`

After successful completion only, landed-truth docs may be updated if repo law requires:

- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md`
- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md`

## Files Not Allowed To Change

- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift`
- any Rust runtime crate
- any backend adapter/server file
- any iPhone source file
- any identity/access/simulation/search/tool contract file
- any other desktop source file

## Acceptance Standard

The later corrective implementation run passes only if all of the following are true:

- H349 remains preserved as complete for its own scope
- the default shell now shows a visible left sidebar by default
- the visible left sidebar now carries conversation/session history in the primary shell structure
- the main pane is transcript-first and opens as a clean conversation surface
- the main composer is visible at the bottom of the default main pane
- no large placeholder explanation cards remain in the normal default shell
- no top-right `History`, `Setup` / `Controls`, or `More` buttons remain as the primary shell structure
- the default shell now matches the same ChatGPT-like light structural rhythm as the approved reference image
- no engineering wording remains in the normal user view
- user messages render on the right and Selene messages on the left
- lawful session reopen behavior remains unchanged
- typed and spoken turns remain in the same visible thread
- spoken TTS reply remains preserved as the same assistant turn
- no backend route, wake-law, session-law, identity, access, or simulation logic change is introduced
- no false wake-parity claim is introduced

## Exact Test Commands

- `git -C /Users/selene/Documents/Selene-OS status --short`
- `git -C /Users/selene/Documents/Selene-OS diff --name-only`
- `rg -n "desktopShellHeader|desktopSecondaryPanelButton|desktopOperationalConversationShell|desktopConversationPrimaryPane|desktopTypedTurnComposerCard|historyCard" /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`
- `rg -n "sessionAttachEndpoint|sessionResumeEndpoint|sessionRecoverEndpoint|voiceTurnEndpoint" /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift`
- `xcodebuild -project /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop.xcodeproj -scheme SeleneMacDesktop -configuration Debug build`
- `git -C /Users/selene/Documents/Selene-OS diff --stat`
- `git -C /Users/selene/Documents/Selene-OS status --short`

## Stop Conditions

Stop immediately in the later implementation run if any of the following becomes true:

- any code change is needed outside `DesktopSessionShellView.swift`
- any runtime bridge change appears necessary
- any backend route change appears necessary
- any wake-law change appears necessary
- any session-law change appears necessary
- any identity/access/simulation logic change appears necessary
- preserving lawful attach / resume / recover behavior cannot be achieved through shell presentation work alone
- the implementation starts to widen authority beyond corrective visual-shell alignment

## Ledger Update Rule

No landed-truth update is allowed during this correction-planning run.

If a later corrective implementation run succeeds, landed-truth updates must:

- preserve H349 as complete for its own scope
- state explicitly that H349 was not the final approved visual target
- record only the exact corrective visual-shell alignment landing
- preserve all existing H259/H349/H348/H324 runtime-authority truth unchanged
- explicitly state that no backend route, wake-law, session-law, identity, access, or simulation logic changed
- explicitly state that no false wake-parity claim was introduced
