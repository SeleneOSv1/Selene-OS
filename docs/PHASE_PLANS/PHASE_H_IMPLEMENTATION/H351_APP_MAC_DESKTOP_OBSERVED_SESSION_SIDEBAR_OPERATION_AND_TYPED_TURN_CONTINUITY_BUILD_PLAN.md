# H351 APP_MAC_DESKTOP Observed Session Sidebar Operation And Typed-Turn Continuity Build Plan

## Purpose

Document the first lawful post-H350 real desktop operation build for APP_MAC_DESKTOP.

H350 is complete as the approved visible-shell baseline, but H350 itself was a bounded visual alignment run only. H351 is the next narrow build authority for operationalizing already-live desktop session selection and already-live typed-turn continuity inside the H350 shell without widening runtime law.

This build does not create a new desktop runtime contract. It only selects and tightens one already-live desktop operation seam:

- lawful visible-sidebar population from already-observed session surfaces
- lawful sidebar foreground selection of an observed session surface
- lawful main-pane hydration from already-live selected-surface carriers
- lawful typed-turn continuity through the already-live canonical `/v1/voice/turn` path

The canonical repo path for this build and every later implementation command is:

- `/Users/selene/Documents/Selene-OS`

## Authority

This build is governed by:

- H350 completion truth in [MASTER_BUILD_COMPLETION_PLAN.md#L341](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L341)
- H350 completion truth in [MASTER_BUILD_COMPLETION_PLAN.md#L345](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L345)
- H350 completion truth in [MASTER_BUILD_COMPLETION_LEDGER.md#L675](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md#L675)
- H337 evidence-only recent-session boundary in [MASTER_BUILD_COMPLETION_PLAN.md#L243](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L243) through [MASTER_BUILD_COMPLETION_PLAN.md#L250](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L250)
- PH1.L deterministic session lifecycle law in [PH1_L.md#L5](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_L.md#L5) through [PH1_L.md#L21](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_L.md#L21)
- PH1.VOICE.ID phone-first identity lock in [PH1_VOICE_ID.md#L5](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_VOICE_ID.md#L5) through [PH1_VOICE_ID.md#L17](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_VOICE_ID.md#L17)
- PH1.ACCESS fail-closed access law in [PH1_ACCESS_001_PH2_ACCESS_002.md#L5](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md#L5) through [PH1_ACCESS_001_PH2_ACCESS_002.md#L21](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md#L21)
- simulation-gated execution law in [08_SIMULATION_CATALOG.md#L5](/Users/selene/Documents/Selene-OS/docs/08_SIMULATION_CATALOG.md#L5) through [08_SIMULATION_CATALOG.md#L24](/Users/selene/Documents/Selene-OS/docs/08_SIMULATION_CATALOG.md#L24)

## Repo Truth Proof

Current live desktop shell truth proves that H351 may stay narrow and shell-local:

- visible sidebar history currently derives only from `currentDominantObservedSessionSurface` plus `observedSessionSurfaces` in [DesktopSessionShellView.swift#L4854](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L4854) through [DesktopSessionShellView.swift#L4865](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L4865)
- the visible sidebar itself and its current row rendering are already live in [DesktopSessionShellView.swift#L4889](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L4889) through [DesktopSessionShellView.swift#L5031](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L5031)
- lawful sidebar selection already flows through `selectObservedSessionSurface` and never through recent-row promotion in [DesktopSessionShellView.swift#L5369](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L5369) through [DesktopSessionShellView.swift#L5473](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L5473)
- selected-session project and pinned-context carriage is already synchronized only from completed lawful session outcomes in [DesktopSessionShellView.swift#L5476](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L5476) through [DesktopSessionShellView.swift#L5515](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L5515)
- main-pane operational state already hydrates only from lawful active / soft-closed / suspended / runtime carriers in [DesktopSessionShellView.swift#L6826](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6826) through [DesktopSessionShellView.swift#L7095](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L7095)
- the H350 main pane already keeps the composer writable only when the foreground selection is the lawful current dominant writable surface in [DesktopSessionShellView.swift#L10789](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L10789) through [DesktopSessionShellView.swift#L10957](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L10957)
- typed turns already stage and dispatch through canonical `/v1/voice/turn` with `threadKey`, `authorityStatePolicyContextRef`, `projectID`, and `pinnedContextRefs` in [DesktopSessionShellView.swift#L16646](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L16646) through [DesktopSessionShellView.swift#L16823](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L16823)
- the runtime bridge already preserves exact `/v1/voice/turn`, exact `/v1/session/attach`, exact `/v1/session/resume`, exact `/v1/session/recover`, exact `/v1/session/recent`, and exact `/v1/session/posture` endpoints in [SeleneMacDesktopRuntimeBridge.swift#L8299](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift#L8299) through [SeleneMacDesktopRuntimeBridge.swift#L8328](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift#L8328)
- recent-session rows remain evidence-only and must stay separate from openable session surfaces under current canon in [MASTER_BUILD_COMPLETION_PLAN.md#L245](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L245) through [MASTER_BUILD_COMPLETION_PLAN.md#L250](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L250)

Current repo truth therefore does not require runtime-bridge edits, backend-route changes, or generic reopen authority for this narrowed build plan.

## CURRENT

Current desktop truth after H350 is:

- H350 visual shell is live and remains the approved shell baseline
- visible left sidebar and bottom composer are live
- sidebar history currently renders from `currentDominantObservedSessionSurface` plus `observedSessionSurfaces`
- sidebar row selection currently foregrounds one observed session surface only
- primary pane already hydrates from lawful active-session text, soft-closed archived recent-slice text, suspended / quarantined explanation-only posture, pending / failed typed or voice previews, and completed canonical runtime reply / provenance / TTS attachments
- typed turns already stage and dispatch through canonical `/v1/voice/turn`
- voice and TTS already remain preserved in the same visible thread
- non-dominant, suspended, and quarantined selections already fail closed to read-only composer behavior
- current-device recent-session rows remain evidence-only and do not become openable transcript surfaces

But the first real desktop operation seam after H350 is not yet isolated as its own approved execution authority. The currently live shell mixes visual H350 landing truth with already-live operational selection and hydration truth without a dedicated narrow build boundary for that operational continuation.

## TARGET

The exact H351 target is:

- visible sidebar population from real lawful observed-session entries only
- lawful sidebar selection / open behavior for one observed session surface only
- main-pane hydration from already-live lawful selected-surface carriers only
- a real bottom composer that sends typed turns into the lawful current dominant selected thread only when current repo truth permits write posture
- read-only fail-closed composer behavior for previously observed non-dominant, soft-closed, suspended, or quarantined selections where current repo truth still requires it
- preserved H350 visual shell, preserved typed/spoken thread visibility, and preserved TTS as the spoken form of the same assistant turn

If no lawful observed session surface exists, the H350 empty conversation shell with ready composer remains the correct fail-closed posture.

## GAP

This gap is not visual-shell work and not a new runtime family.

The gap is:

- operationalizing already-live observed-session sidebar truth inside the H350 shell as a dedicated approved build
- tightening lawful selected-surface continuity between sidebar, main pane, and writable composer gating
- preserving already-live typed-turn continuity through the canonical runtime path while making that continuity the explicit focus of the build

This gap is specifically not:

- wake work
- identity work
- access work
- simulation work
- recent-session evidence promotion
- generic reopen work
- transcript-history backend expansion

## In Scope

The later H351 implementation run may only:

- keep sidebar population limited to `currentDominantObservedSessionSurface` plus `observedSessionSurfaces`
- keep sidebar rows limited to lawful observed-session entries already available in the shell
- preserve and tighten lawful `selectObservedSessionSurface` foreground behavior inside the H350 shell
- preserve and tighten main-pane hydration from already-live active-session, soft-closed archived-slice, suspended/quarantined explanation-only, pending/failed request preview, and completed canonical runtime reply/provenance/TTS carriers
- preserve the H350 visible shell layout while making the sidebar/main-pane/composer relationship behave as one explicit real desktop operation seam
- keep the composer writable only when the foregrounded surface remains the lawful current dominant writable surface
- keep previously observed non-dominant surfaces read-only
- preserve voice and TTS in the same visible thread exactly as already wired
- preserve typed-turn sending through the already-live canonical `/v1/voice/turn` path only

## Out Of Scope

- no runtime bridge changes
- no backend route changes
- no new wake parity claims
- no hidden/background wake work
- no wake-law changes
- no session-law changes
- no attach / resume / recover semantic widening
- no identity logic changes
- no access logic changes
- no simulation logic changes
- no tool/search expansion
- no generic local reopen shortcuts
- no recent-row-driven attach / resume / recover authority
- no promotion of evidence-only recent-session rows into openable transcript surfaces
- no generic transcript-history backend fetch invention
- no non-canonical local thread retargeting
- no writable composer for previously observed non-dominant surfaces when current repo truth still requires read-only fail-closed behavior
- no UI redesign away from the approved H350 shell

## Files Allowed To Change

The later H351 implementation run may change only:

- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`

If and only if the later implementation succeeds and repo law requires landed-truth updates:

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

The later H351 implementation run passes only if all of the following are true:

- H350 remains complete and preserved as the approved visible-shell baseline
- H351 is implemented as the first real desktop operation run after H350
- the visible sidebar remains populated only by lawful observed-session entries already present in shell truth
- current-device recent-session rows remain evidence-only and do not become openable conversation surfaces
- selecting a lawful observed-session entry foregrounds the correct surface in the main pane
- the main pane shows only already-live lawful selected-surface content and does not imply a new generic transcript-history fetch route
- the main composer sends real typed turns only through the already-live canonical `/v1/voice/turn` path
- typed-turn sending remains writable only for the lawful current dominant active/ready surface
- previously observed non-dominant, soft-closed, suspended, and quarantined selections remain read-only where current repo truth requires fail-closed behavior
- typed and spoken turns remain in the same visible thread
- TTS remains preserved as the spoken form of the same assistant turn
- no backend route, wake-law, session-law, identity, access, or simulation logic change is introduced
- no generic reopen shortcut, no generic sidebar-open mutation, and no non-canonical local thread retargeting are introduced
- no false wake-parity claim is introduced

## Exact Test Commands

- `git -C /Users/selene/Documents/Selene-OS status --short`
- `git -C /Users/selene/Documents/Selene-OS diff --name-only`
- `rg -n "desktopSidebarHistorySurfaces|desktopVisibleConversationSidebar|desktopSidebarHistoryRow|selectObservedSessionSurface|foregroundObservedSessionSurface" /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`
- `rg -n "desktopConversationPrimaryPaneState|desktopOperationalConversationMainPane|desktopConversationComposerRegion|desktopConversationReadOnlyComposerMessage" /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`
- `rg -n "dispatchPreparedTypedTurnRequestIfNeeded|submitDesktopTypedTurn|desktopForegroundVoiceTurnMatchingSelectedThreadKey|desktopForegroundVoiceTurnActiveAuthorityPolicyContextRef|desktopForegroundVoiceTurnSelectedProjectID|desktopForegroundVoiceTurnSelectedPinnedContextRefs" /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`
- `rg -n "voiceTurnEndpoint|sessionAttachEndpoint|sessionResumeEndpoint|sessionRecoverEndpoint|sessionRecentEndpoint|sessionPostureEndpoint" /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift`
- `xcodebuild -project /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop.xcodeproj -scheme SeleneMacDesktop -configuration Debug build`
- `git -C /Users/selene/Documents/Selene-OS diff --stat`
- `git -C /Users/selene/Documents/Selene-OS status --short`

## Stop Conditions

Stop immediately in the later H351 implementation run if any of the following becomes true:

- real sidebar population would require promoting recent-session evidence rows into session surfaces
- main-pane hydration would require a new backend transcript/history route
- making the selected thread writable would require generic reopen, session retargeting, or mutation authority beyond the current dominant lawful surface
- sidebar population would require a broader persisted session-history family beyond the already-live current-dominant plus observed-session rail
- any change outside `DesktopSessionShellView.swift` becomes necessary
- any `RuntimeBridge` change appears necessary
- any backend route change appears necessary
- any wake/session/identity/access/simulation law change appears necessary
- preserving voice and TTS in the same visible thread would require new runtime authority rather than existing shell carriers

## Ledger Update Rule

No landed-truth update is allowed during this doc-only planning run.

If a later H351 implementation run succeeds, landed-truth updates must:

- preserve H350 as complete and authoritative for the approved visual shell baseline
- record only the exact H351 observed-session sidebar operation and typed-turn continuity landing
- state explicitly that recent-session rows remain evidence-only in the H351 landing
- state explicitly that transcript hydration remained limited to already-live lawful primary-pane carriers
- state explicitly that typed-turn sending remained limited to the already-live canonical `/v1/voice/turn` path
- preserve all H350 / H349 / H348 / H337 / H324 runtime-authority truth unchanged
- state explicitly that no backend route, wake-law, session-law, identity, access, or simulation logic changed
- state explicitly that no false wake-parity claim was introduced
