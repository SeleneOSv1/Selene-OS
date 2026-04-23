# H353 APP_MAC_DESKTOP Pre-Ready-Time-Handoff Main-Pane Continuity Build Plan

## Purpose

Document the next lawful desktop implementation seam after H352.

H352 is complete for its own approved scope. H352 correctly inlined dominant-surface session-entry actions in the H350 main pane without widening runtime authority. But H352 did not solve the user-visible continuity gap that still exists before ready-time handoff:

- the default shell can accept typed input
- the default shell can accept explicit voice capture
- the shell already stages and dispatches those bounded requests through the already-live canonical paths
- but the main conversation pane still does not truthfully surface those already-live continuity carriers before `desktopReadyTimeHandoffIsActive` becomes true

The next lawful desktop seam is therefore not another shell redesign. It is one narrower transcript-continuity seam:

- preserve the H350 shell
- preserve H352
- preserve current runtime law
- surface already-live typed / voice / runtime continuity carriers in the main conversation pane before ready-time handoff, without inventing new authority

The canonical repo path for this build and every later implementation command is:

- `/Users/selene/Documents/Selene-OS`

## Authority

This build is governed by:

- H350 completion truth in [MASTER_BUILD_COMPLETION_PLAN.md#L341](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L341) through [MASTER_BUILD_COMPLETION_PLAN.md#L349](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L349)
- H352 completion truth in [MASTER_BUILD_COMPLETION_PLAN.md#L350](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L350) through [MASTER_BUILD_COMPLETION_PLAN.md#L358](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L358)
- H352 completion truth in [MASTER_BUILD_COMPLETION_LEDGER.md#L676](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md#L676)
- H352 implementation boundary in [H352_APP_MAC_DESKTOP_INLINE_DOMINANT_SURFACE_SESSION_ENTRY_ACTIONS_IN_MAIN_PANE_BUILD_PLAN.md#L1](/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H352_APP_MAC_DESKTOP_INLINE_DOMINANT_SURFACE_SESSION_ENTRY_ACTIONS_IN_MAIN_PANE_BUILD_PLAN.md#L1)
- PH1.L deterministic session lifecycle law in [PH1_L.md#L1](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_L.md#L1) through [PH1_L.md#L21](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_L.md#L21)
- PH1.VOICE.ID fail-closed identity discipline in [PH1_VOICE_ID.md#L1](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_VOICE_ID.md#L1) through [PH1_VOICE_ID.md#L17](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_VOICE_ID.md#L17)
- PH1.ACCESS fail-closed access law in [PH1_ACCESS_001_PH2_ACCESS_002.md#L1](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md#L1) through [PH1_ACCESS_001_PH2_ACCESS_002.md#L21](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md#L21)
- simulation-gated execution law in [08_SIMULATION_CATALOG.md#L1](/Users/selene/Documents/Selene-OS/docs/08_SIMULATION_CATALOG.md#L1) through [08_SIMULATION_CATALOG.md#L24](/Users/selene/Documents/Selene-OS/docs/08_SIMULATION_CATALOG.md#L24)

## Repo Truth Proof

Current repo truth proves all of the following:

- the app body still falls back to `desktopEvidenceFirstOperationalShell` when `desktopOperationalConversationShellState` is nil in [DesktopSessionShellView.swift#L4672](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L4672) through [DesktopSessionShellView.swift#L4678](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L4678)
- typed-turn dispatch already exists through the canonical typed-turn path in [DesktopSessionShellView.swift#L4698](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L4698) through [DesktopSessionShellView.swift#L4700](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L4700) and [DesktopSessionShellView.swift#L16870](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L16870) through [DesktopSessionShellView.swift#L17020](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L16870)
- `desktopOperationalConversationShellState` still requires both the primary pane and the support rail in [DesktopSessionShellView.swift#L6814](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6814) through [DesktopSessionShellView.swift#L6823](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6823)
- `desktopConversationPrimaryPaneState` is still hard-gated by `desktopReadyTimeHandoffIsActive` in [DesktopSessionShellView.swift#L6826](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6826) through [DesktopSessionShellView.swift#L6829](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6826)
- the already-live continuity carriers already exist inside `desktopConversationPrimaryPaneState`, including:
  - explicit voice live preview in [DesktopSessionShellView.swift#L6914](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6914) through [DesktopSessionShellView.swift#L6929](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6929)
  - wake live preview in [DesktopSessionShellView.swift#L6931](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6931) through [DesktopSessionShellView.swift#L6947](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6947)
  - explicit voice pending preview in [DesktopSessionShellView.swift#L6949](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6949) through [DesktopSessionShellView.swift#L6959](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6959)
  - wake pending preview in [DesktopSessionShellView.swift#L6961](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6961) through [DesktopSessionShellView.swift#L6972](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6972)
  - explicit voice failure preview in [DesktopSessionShellView.swift#L6974](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6974) through [DesktopSessionShellView.swift#L6986](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6986)
  - wake failure preview in [DesktopSessionShellView.swift#L6988](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6988) through [DesktopSessionShellView.swift#L7002](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L7002)
  - pending typed-turn preview in [DesktopSessionShellView.swift#L7004](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L7004) through [DesktopSessionShellView.swift#L7014](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L7004)
  - failed typed-turn preview in [DesktopSessionShellView.swift#L7016](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L7016) through [DesktopSessionShellView.swift#L7027](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L7016)
  - runtime failure and authoritative reply carriers in [DesktopSessionShellView.swift#L7056](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L7056) through [DesktopSessionShellView.swift#L7310](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L7056)
- `desktopConversationSupportRailState` is also hard-gated by `desktopReadyTimeHandoffIsActive` in [DesktopSessionShellView.swift#L7453](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L7453) through [DesktopSessionShellView.swift#L7485](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L7453)
- H352 is already complete and must not be reopened as a visual-shell or dominant-surface action move in [MASTER_BUILD_COMPLETION_PLAN.md#L350](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L350) through [MASTER_BUILD_COMPLETION_PLAN.md#L358](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L358)
- the already-wired canonical submit paths remain unchanged and require no `RuntimeBridge` widening in [SeleneMacDesktopRuntimeBridge.swift#L8299](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift#L8299) through [SeleneMacDesktopRuntimeBridge.swift#L8328](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift#L8328)

Therefore:

- H352 is complete for its own approved scope
- H352 did not solve pre-ready-time-handoff transcript continuity in the default shell
- the next lawful desktop seam is not a shell redesign
- the next lawful desktop seam is bounded main-pane continuity before ready-time handoff
- current repo truth does not require `RuntimeBridge` change, backend-route change, session-law widening, or synthetic local authority for this seam

## CURRENT

Current desktop truth after H352 is:

- the H350 shell is live
- the default app body still falls back to `desktopEvidenceFirstOperationalShell` when `desktopOperationalConversationShellState` is nil
- `desktopOperationalConversationShellState`, `desktopConversationPrimaryPaneState`, and `desktopConversationSupportRailState` are currently gated by `desktopReadyTimeHandoffIsActive`
- typed-turn staging and dispatch already exist through the canonical typed-turn path
- explicit voice transcript preview, pending voice preview, wake preview, failed typed-turn preview, pending typed-turn preview, runtime failure, and authoritative reply carriers already exist in `desktopConversationPrimaryPaneState`
- the user can type and speak in the default shell, but the message pane does not yet truthfully surface those already-live carriers before ready-time handoff
- suspended and quarantined postures already remain fail-closed and explanation-only
- H352 already inlined dominant-surface session-entry actions and must not be reopened

## TARGET

The exact H353 target is:

- keep the H350 shell visible as the default desktop experience
- make the main pane show already-live typed / voice / runtime conversation continuity carriers before ready-time handoff when those carriers already exist
- keep the composer visible and consistent with the same thread
- preserve current lawful read-only behavior for suspended / quarantined / non-dominant selections
- preserve already-wired canonical submit paths exactly as they are
- preserve already-wired voice and TTS behavior in the same visible thread
- preserve current support-rail / secondary-surface gating unless current repo truth proves broader exposure is already lawful
- do not widen runtime authority
- do not invent a new transcript system
- do not invent a new local message lane

## GAP

This gap is not:

- sidebar redesign
- shell redesign
- typed-turn path redesign
- voice-path redesign
- session selection redesign

This gap is specifically:

- decoupling already-live main-pane conversation continuity rendering from the current ready-time-handoff-only shell gate, while keeping all current runtime-law boundaries intact

## In Scope

The later H353 implementation run may only:

- surface already-live typed-turn, explicit-voice, wake-preview, runtime-failure, and authoritative-reply timeline carriers in the main conversation pane even when `desktopReadyTimeHandoffIsActive` is not yet true
- preserve the H350 shell layout and visual structure
- preserve the existing bottom composer
- preserve fail-closed read-only behavior for suspended / quarantined / non-dominant selections where current repo truth requires it
- preserve already-wired canonical submit paths exactly as they are
- preserve already-wired voice and TTS behavior in the same visible thread
- preserve current support-rail / secondary-surface gating unless repo truth proves broader exposure is already lawful
- stay inside `DesktopSessionShellView.swift`

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
- no shell redesign away from the approved H350 shell
- no support-rail redesign unless already-live lawful shell truth requires a tiny same-file alignment
- no false wake-parity claim
- no hidden/background wake work

## Files Allowed To Change

The later H353 implementation run may change only:

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

The later H353 implementation run passes only if all of the following are true:

- H352 remains complete for its own approved scope
- the default desktop shell still preserves the H350 layout and visual structure
- already-live typed / voice / runtime continuity carriers can appear in the main conversation pane before ready-time handoff when those carriers already exist
- the composer remains visible and part of the same conversation thread
- suspended and quarantined selections remain fail-closed and explanation-only
- non-dominant selections remain read-only wherever current repo truth still requires that boundary
- canonical submit paths remain unchanged
- voice and TTS remain in the same visible thread
- no RuntimeBridge change, backend route change, wake-law change, session-law change, identity change, access change, or simulation change is introduced
- no new local message lane, no synthetic local authority, and no false wake-parity claim are introduced

## Exact Test Commands

- `git -C /Users/selene/Documents/Selene-OS status --short`
- `git -C /Users/selene/Documents/Selene-OS diff --name-only`
- `rg -n "desktopOperationalConversationShellState|desktopConversationPrimaryPaneState|desktopConversationSupportRailState|desktopEvidenceFirstOperationalShell" /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`
- `rg -n "explicit_voice_live_preview|wake_voice_live_preview|explicit_voice_pending_preview|wake_voice_pending_preview|typed_turn_failed_request_preview|runtime_dispatch_failure_preview|authoritative_reply_text" /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`
- `rg -n "dispatchPreparedTypedTurnRequestIfNeeded|submitDesktopTypedTurn|desktopTypedTurnPendingRequest" /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`
- `rg -n "voiceTurnEndpoint|sessionAttachEndpoint|sessionResumeEndpoint|sessionRecoverEndpoint" /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift`
- `xcodebuild -project /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop.xcodeproj -scheme SeleneMacDesktop -configuration Debug build`
- `git -C /Users/selene/Documents/Selene-OS diff --stat`
- `git -C /Users/selene/Documents/Selene-OS status --short`

## Stop Conditions

Stop immediately in the later H353 implementation run if any of the following becomes true:

- pre-ready-time-handoff main-pane continuity would require RuntimeBridge changes
- pre-ready-time-handoff main-pane continuity would require backend route changes
- pre-ready-time-handoff main-pane continuity would require wake/session/identity/access/simulation law changes
- pre-ready-time-handoff main-pane continuity would require widening non-dominant writability
- pre-ready-time-handoff main-pane continuity would require inventing a new local transcript family instead of reusing already-live timeline carriers
- any file outside `DesktopSessionShellView.swift` becomes necessary
- repo truth disproves exposing those carriers before ready-time handoff
- preserving voice and TTS in the same visible thread would require new runtime authority rather than already-live shell carriers

## Ledger Update Rule

No landed-truth update is allowed during this doc-only planning run.

If a later H353 implementation run succeeds, landed-truth updates must:

- preserve H352 as complete for its own approved scope
- state explicitly that H352 did not solve pre-ready-time-handoff transcript continuity in the default shell
- record only the exact H353 pre-ready-time-handoff main-pane continuity landing
- state explicitly that canonical submit paths remained unchanged
- state explicitly that suspended / quarantined fail-closed behavior remained unchanged
- preserve H350 / H352 / H322 / H321 / H312 truth unchanged outside the bounded H353 seam
- state explicitly that no RuntimeBridge change, backend route change, wake-law change, session-law change, identity change, access change, or simulation change was introduced
- state explicitly that no new local message lane, no synthetic local authority, and no false wake-parity claim were introduced
