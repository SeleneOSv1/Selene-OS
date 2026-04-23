# H352 APP_MAC_DESKTOP Inline Dominant-Surface Session-Entry Actions In Main Pane Build Plan

## Purpose

Document the next lawful post-H350 desktop build boundary after review of H351 on unchanged repo truth.

H351 was reviewed against the live desktop shell and current published APP_MAC_DESKTOP truth. That review found no truthful code-bearing delta remaining for H351 on unchanged source. H351 therefore must not be forced into implementation.

The next lawful desktop seam is different:

- surface the already-live exact lawful attach / resume / recover / submit actions inline in the H350 main pane
- only for the lawful current dominant surface
- preserve the H350 shell, existing fail-closed read-only behavior, already-wired voice / TTS visibility, and already-wired canonical submit paths exactly as they are

This build does not create a new runtime family. It only selects one narrower shell-local presentation seam that is still real on current repo truth.

The canonical repo path for this build and every later implementation command is:

- `/Users/selene/Documents/Selene-OS`

## Authority

This build is governed by:

- H350 completion truth in [MASTER_BUILD_COMPLETION_PLAN.md#L341](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L341) through [MASTER_BUILD_COMPLETION_PLAN.md#L349](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L349)
- H350 completion truth in [MASTER_BUILD_COMPLETION_LEDGER.md#L675](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md#L675)
- H351 planning truth in [H351_APP_MAC_DESKTOP_OBSERVED_SESSION_SIDEBAR_OPERATION_AND_TYPED_TURN_CONTINUITY_BUILD_PLAN.md#L1](/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H351_APP_MAC_DESKTOP_OBSERVED_SESSION_SIDEBAR_OPERATION_AND_TYPED_TURN_CONTINUITY_BUILD_PLAN.md#L1)
- H321 typed-turn continuity truth in [MASTER_BUILD_COMPLETION_PLAN.md#L410](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L410) through [MASTER_BUILD_COMPLETION_PLAN.md#L432](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L432)
- H322 observed-session selection truth in [MASTER_BUILD_COMPLETION_PLAN.md#L390](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L390) through [MASTER_BUILD_COMPLETION_PLAN.md#L409](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L409)
- already-landed exact attach / resume / recover / entry submit truth in [MASTER_BUILD_COMPLETION_PLAN.md#L165](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L165) through [MASTER_BUILD_COMPLETION_PLAN.md#L180](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L180)
- already-landed exact attach / resume / recover / entry submit truth in [MASTER_BUILD_COMPLETION_LEDGER.md#L701](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md#L701) through [MASTER_BUILD_COMPLETION_LEDGER.md#L704](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md#L704)
- PH1.L deterministic session lifecycle law in [PH1_L.md#L1](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_L.md#L1) through [PH1_L.md#L21](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_L.md#L21)
- PH1.VOICE.ID fail-closed identity discipline in [PH1_VOICE_ID.md#L1](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_VOICE_ID.md#L1) through [PH1_VOICE_ID.md#L17](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_VOICE_ID.md#L17)
- PH1.ACCESS fail-closed access law in [PH1_ACCESS_001_PH2_ACCESS_002.md#L1](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md#L1) through [PH1_ACCESS_001_PH2_ACCESS_002.md#L21](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md#L21)
- simulation-gated execution law in [08_SIMULATION_CATALOG.md#L1](/Users/selene/Documents/Selene-OS/docs/08_SIMULATION_CATALOG.md#L1) through [08_SIMULATION_CATALOG.md#L24](/Users/selene/Documents/Selene-OS/docs/08_SIMULATION_CATALOG.md#L24)

## Repo Truth Proof

Current repo truth proves all of the following:

- H350 shell is already live, including the visible sidebar, transcript-first main pane, and bottom composer, in [DesktopSessionShellView.swift#L4848](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L4848) through [DesktopSessionShellView.swift#L5031](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L5031) and [DesktopSessionShellView.swift#L10740](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L10740) through [DesktopSessionShellView.swift#L10957](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L10957)
- the default main pane still falls back to ready/status canvas for empty or read-only postures in [DesktopSessionShellView.swift#L10802](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L10802) through [DesktopSessionShellView.swift#L10937](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L10937)
- exact lawful attach / resume / recover prompt states already exist in [DesktopSessionShellView.swift#L7878](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L7878) through [DesktopSessionShellView.swift#L8098](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L8098)
- exact lawful submit functions already exist in [DesktopSessionShellView.swift#L16181](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L16181) through [DesktopSessionShellView.swift#L16356](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L16356)
- those controls are still rendered as separate support surfaces, not inline default main-pane actions, in [DesktopSessionShellView.swift#L12330](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L12330) through [DesktopSessionShellView.swift#L12820](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L12820)
- H321 already covers the canonical typed-turn path, so H351 must not be forced merely to “re-land” typed-turn continuity, in [MASTER_BUILD_COMPLETION_PLAN.md#L410](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L410) through [MASTER_BUILD_COMPLETION_PLAN.md#L432](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L432)
- H322 already covers lawful observed-session selection, so H351 must not be forced merely to “re-land” sidebar selection continuity, in [MASTER_BUILD_COMPLETION_PLAN.md#L390](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L390) through [MASTER_BUILD_COMPLETION_PLAN.md#L409](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L409)
- already-landed exact attach / resume / recover / entry submit law remains route-specific and unchanged in [MASTER_BUILD_COMPLETION_PLAN.md#L165](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L165) through [MASTER_BUILD_COMPLETION_PLAN.md#L180](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L180)
- the canonical submit endpoints remain already wired and need no widening in [SeleneMacDesktopRuntimeBridge.swift#L8299](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift#L8299) through [SeleneMacDesktopRuntimeBridge.swift#L8328](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift#L8328)

Therefore:

- H351 was reviewed and found to have no truthful code-bearing delta on unchanged repo truth
- H351 must not be forced into implementation
- the next lawful desktop seam is different and narrower: inline dominant-surface main-pane presentation of already-live exact session-entry actions

## CURRENT

Current desktop truth after H350 and post-H351 review is:

- H350 shell is live
- default main pane still falls back to ready/status canvas for empty or read-only postures
- exact lawful attach / resume / recover prompt states already exist
- exact submit paths already exist
- those controls are still rendered as separate support surfaces, not inline default main-pane actions
- H321 and H322 already cover the typed-turn and observed-session selection behavior that H351 tried to re-select
- voice and TTS already remain in the same visible thread
- non-dominant / suspended / quarantined selections already fail closed to read-only behavior in the main pane

## TARGET

The exact H352 target is:

- inline the already-live lawful attach / resume / recover / submit actions into the H350 main pane
- only for the lawful current dominant surface
- keep the shell visually clean
- keep non-dominant / suspended / quarantined states read-only and fail-closed
- preserve already-wired voice and TTS behavior in the same visible thread
- preserve already-wired canonical submit paths exactly as they are
- do not widen runtime authority

## GAP

This gap is not:

- shell redesign
- sidebar redesign
- typed-turn path redesign
- session selection redesign

This gap is specifically:

- moving already-live lawful session-entry actions from support surfaces into the main pane for the lawful dominant surface

## In Scope

The later H352 implementation run may only:

- surface the already-live exact lawful attach / resume / recover / submit actions inline in the H350 main pane
- gate those inline actions to the lawful current dominant surface only
- preserve ready/status canvas behavior where no lawful inline dominant-surface action is available
- preserve fail-closed read-only behavior for non-dominant, suspended, and quarantined selections
- preserve typed and spoken turns in the same visible thread
- preserve TTS as the spoken form of the same assistant turn
- preserve the existing exact canonical submit functions and exact canonical route usage without altering their behavior
- preserve the approved H350 shell layout and visual structure outside this narrow inline-action move

## Out Of Scope

- no RuntimeBridge changes
- no backend route changes
- no wake-law changes
- no session-law changes
- no identity/access/simulation logic changes
- no generic reopen shortcuts
- no recent-row-driven authority
- no new transcript-history fetch route
- no new local message lane
- no sidebar data-model redesign
- no session selection redesign
- no typed-turn path redesign
- no UI redesign away from the approved H350 shell

## Files Allowed To Change

The later H352 implementation run may change only:

- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`

If and only if later implementation succeeds and repo law requires landed-truth updates:

- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md`
- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md`

## Files Not Allowed To Change

- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift`
- any Rust runtime crate
- any backend adapter/server file
- any DB wiring spec
- any iPhone source file
- any other desktop source file unless current repo truth newly proves that it is unavoidable, in which case the later implementation run must stop and report before drifting

## Acceptance Standard

The later H352 implementation run passes only if all of the following are true:

- H350 remains preserved as the approved visible-shell baseline
- H351 remains unforced because no truthful code-bearing delta existed for it on unchanged repo truth
- the default main pane no longer relies only on support-surface placement for lawful current-dominant attach / resume / recover actions
- already-live exact lawful attach / resume / recover / submit actions appear inline in the main pane only for the lawful current dominant surface
- non-dominant, suspended, and quarantined states remain read-only and fail-closed
- ready/status canvas behavior remains correct when no lawful inline dominant-surface action exists
- typed and spoken turns remain in the same visible thread
- TTS remains preserved as the spoken form of the same assistant turn
- the already-live canonical submit paths remain unchanged
- no RuntimeBridge change, backend route change, wake-law change, session-law change, identity change, access change, or simulation change is introduced
- no generic reopen shortcut, no recent-row-driven authority, and no false wake-parity claim is introduced

## Exact Test Commands

- `git -C /Users/selene/Documents/Selene-OS status --short`
- `git -C /Users/selene/Documents/Selene-OS diff --name-only`
- `rg -n "desktopOperationalConversationMainPane|desktopConversationPrimaryPane|desktopConversationReadyCanvas|desktopConversationStatusCanvas|desktopConversationComposerRegion" /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`
- `rg -n "desktopSessionAttachPromptState|desktopSessionMultiPostureResumePromptState|desktopSessionMultiPostureEntryPromptState" /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`
- `rg -n "submitDesktopSessionAttach|submitDesktopSessionMultiPostureResume|submitDesktopSessionMultiPostureEntry" /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`
- `rg -n "desktopSessionAttachCard|desktopSessionMultiPostureResumeCard|desktopSessionMultiPostureEntryCard" /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`
- `rg -n "sessionAttachEndpoint|sessionResumeEndpoint|sessionRecoverEndpoint|voiceTurnEndpoint" /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift`
- `xcodebuild -project /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop.xcodeproj -scheme SeleneMacDesktop -configuration Debug build`
- `git -C /Users/selene/Documents/Selene-OS diff --stat`
- `git -C /Users/selene/Documents/Selene-OS status --short`

## Stop Conditions

Stop immediately in the later H352 implementation run if any of the following becomes true:

- inline main-pane session-entry actions would require RuntimeBridge changes
- inline main-pane session-entry actions would require backend route changes
- inline main-pane session-entry actions would require wake/session/identity/access/simulation law changes
- inline main-pane session-entry actions would require widening non-dominant writability
- any file outside `DesktopSessionShellView.swift` becomes necessary
- repo truth disproves the current dominant-surface-only approach
- preserving already-wired voice and TTS in the same visible thread would require new runtime authority rather than existing shell carriers

## Ledger Update Rule

No landed-truth update is allowed during this doc-only planning run.

If a later H352 implementation run succeeds, landed-truth updates must:

- preserve H350 as complete and authoritative for the approved shell baseline
- state explicitly that H351 was reviewed and found to have no truthful code-bearing delta on unchanged repo truth
- state explicitly that H351 was not forced into implementation
- record only the exact H352 inline dominant-surface main-pane session-entry-action landing
- state explicitly that canonical submit paths remained unchanged
- state explicitly that non-dominant / suspended / quarantined states remained fail-closed
- preserve H350 / H322 / H321 / H296 / H295 / H294 / H293 / H286 truth unchanged
- state explicitly that no RuntimeBridge change, backend route change, wake-law change, session-law change, identity change, access change, or simulation change was introduced
- state explicitly that no generic reopen shortcut, no recent-row-driven authority, and no false wake-parity claim were introduced
