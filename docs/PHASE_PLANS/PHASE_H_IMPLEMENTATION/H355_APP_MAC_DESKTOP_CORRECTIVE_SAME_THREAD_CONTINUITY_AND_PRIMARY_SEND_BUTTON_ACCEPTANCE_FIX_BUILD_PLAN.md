# H355 APP_MAC_DESKTOP Corrective Same-Thread Continuity And Primary Send-Button Acceptance Fix Build Plan

## Purpose

Document the next lawful desktop correction after H354.

H354 landed in repo truth, but H354 did not pass live product acceptance. Current live user testing still shows two bounded product failures in the default desktop shell:

- typed continuity is still not reliably retained in the main message pane during normal dominant-surface use
- explicit voice continuity is still not reliably retained in that same main message pane during normal dominant-surface use
- clicking the arrow/send button does not yet behave as the real primary submit action

This correction is not a shell redesign and not a runtime-law change. It is one narrower acceptance-fix seam inside the already-approved H350 shell:

- preserve H350 shell structure
- preserve H353 pre-ready continuity law boundary
- preserve H354 composer shape and same-thread intent
- correct broken same-thread continuity visibility
- correct broken primary send-button behavior

The canonical repo path for this build and every later implementation command is:

- `/Users/selene/Documents/Selene-OS`

## Authority

This build is governed by:

- H353 completion truth in [MASTER_BUILD_COMPLETION_PLAN.md#L360](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L360) through [MASTER_BUILD_COMPLETION_PLAN.md#L367](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L367)
- H354 completion truth in [MASTER_BUILD_COMPLETION_PLAN.md#L368](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L368) through [MASTER_BUILD_COMPLETION_PLAN.md#L374](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L374)
- H353 implementation boundary in [H353_APP_MAC_DESKTOP_PRE_READY_TIME_HANDOFF_MAIN_PANE_CONTINUITY_BUILD_PLAN.md#L1](/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H353_APP_MAC_DESKTOP_PRE_READY_TIME_HANDOFF_MAIN_PANE_CONTINUITY_BUILD_PLAN.md#L1)
- H354 implementation boundary in [H354_APP_MAC_DESKTOP_SAME_THREAD_INPUT_MODALITY_CONTINUITY_AND_COMPOSER_ALIGNMENT_BUILD_PLAN.md#L1](/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H354_APP_MAC_DESKTOP_SAME_THREAD_INPUT_MODALITY_CONTINUITY_AND_COMPOSER_ALIGNMENT_BUILD_PLAN.md#L1)
- PH1.L deterministic session lifecycle law in [PH1_L.md#L1](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_L.md#L1) through [PH1_L.md#L21](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_L.md#L21)
- PH1.VOICE.ID fail-closed identity discipline in [PH1_VOICE_ID.md#L1](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_VOICE_ID.md#L1) through [PH1_VOICE_ID.md#L17](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_VOICE_ID.md#L17)
- PH1.ACCESS fail-closed access law in [PH1_ACCESS_001_PH2_ACCESS_002.md#L1](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md#L1) through [PH1_ACCESS_001_PH2_ACCESS_002.md#L21](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md#L21)
- simulation-gated execution law in [08_SIMULATION_CATALOG.md#L1](/Users/selene/Documents/Selene-OS/docs/08_SIMULATION_CATALOG.md#L1) through [08_SIMULATION_CATALOG.md#L16](/Users/selene/Documents/Selene-OS/docs/08_SIMULATION_CATALOG.md#L16)

## Repo Truth Proof

Current repo truth proves all of the following:

- H350 shell is live and remains the approved desktop shell baseline in [MASTER_BUILD_COMPLETION_PLAN.md#L341](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L341) through [MASTER_BUILD_COMPLETION_PLAN.md#L349](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L349)
- H353 is live and already exposes pre-ready main-pane continuity carriers in repo truth in [MASTER_BUILD_COMPLETION_PLAN.md#L360](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L360) through [MASTER_BUILD_COMPLETION_PLAN.md#L367](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L367)
- H354 is live and already reshaped the composer and same-thread modality surface in repo truth in [MASTER_BUILD_COMPLETION_PLAN.md#L368](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L368) through [MASTER_BUILD_COMPLETION_PLAN.md#L374](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L374)
- the current composer now uses `TextEditor` and one arrow/send button inside `desktopTypedTurnComposerCard` in [DesktopSessionShellView.swift#L6129](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6129) through [DesktopSessionShellView.swift#L6235](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6129)
- the arrow/send button already calls `submitDesktopTypedTurn()` in [DesktopSessionShellView.swift#L6217](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6217) through [DesktopSessionShellView.swift#L6235](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6217)
- the already-live canonical typed submit action is still `submitDesktopTypedTurn()` in [DesktopSessionShellView.swift#L17304](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L17304) through [DesktopSessionShellView.swift#L17343](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L17304)
- the already-live canonical typed staging path is still `stageDesktopTypedTurnRequest(...)` in [DesktopSessionShellView.swift#L17239](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L17239) through [DesktopSessionShellView.swift#L17281](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L17239)
- the pre-ready main pane already attempts to surface explicit voice live preview, explicit voice pending preview, explicit voice failed preview, typed pending preview, typed failed preview, runtime failure, and authoritative reply in [DesktopSessionShellView.swift#L6889](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6889) through [DesktopSessionShellView.swift#L7065](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6889)
- the current pre-ready continuity implementation still renders explicit-voice failure and typed-turn failure bubbles from `failedRequest.summary` instead of preserving the actual user-authored utterance text in [DesktopSessionShellView.swift#L7006](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L7006) through [DesktopSessionShellView.swift#L7016](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L7006) and [DesktopSessionShellView.swift#L7032](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L7032) through [DesktopSessionShellView.swift#L7041](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L7032)
- the current typed-turn staging path still fails closed while explicit voice is listening or pending in [DesktopSessionShellView.swift#L17255](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L17255) through [DesktopSessionShellView.swift#L17261](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L17255)
- explicit voice capture and explicit voice submit already remain on the same canonical voice path without `RuntimeBridge` widening in [DesktopSessionShellView.swift#L1488](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L1488) through [DesktopSessionShellView.swift#L1548](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L1488) and [DesktopSessionShellView.swift#L16999](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L16999) through [DesktopSessionShellView.swift#L17058](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L16999)
- live user testing in the current desktop product still shows that typed continuity and explicit-voice continuity do not remain correctly visible in the main message pane during normal use, and the arrow/send button does not yet behave as the real primary submit action
- no current repo truth justifies widening runtime authority, widening session law, or adding a new transcript family to correct this acceptance failure

Therefore:

- H354 landed, but failed live product acceptance
- typed continuity is still not reliably retained in the main message pane during normal use
- explicit voice continuity is still not reliably retained in the same main message pane during normal use
- clicking the arrow/send button does not yet behave as the real primary submit action
- this correction is not a shell redesign and not a runtime-law change

## CURRENT

Current desktop truth after H354 is:

- H350 shell is live
- H353 continuity work is live
- H354 composer/modality work is live in repo truth
- live user testing still shows that typed and voice continuity do not stay correctly visible in the main message pane during normal use
- live user testing still shows the arrow/send button is not functioning as the real primary submit action
- the canonical typed submit path already exists
- the canonical voice path already exists
- no current proof justifies widening runtime authority

## TARGET

The exact H355 target is:

- typing a message and clicking the arrow/send button shows typed continuity in the same main thread using the existing lawful dominant thread
- explicit voice continuity, previews, and later replies also remain visible in that same main thread
- the send arrow invokes the same already-live canonical typed submit action
- conversation identity stays the same while switching input mode
- no new transcript system is invented
- no new local message lane is invented
- no separate recording mode is added
- no shell redesign away from H350 is introduced

## GAP

This gap is not:

- sidebar redesign
- shell redesign
- RuntimeBridge redesign
- typed-turn route redesign
- voice route redesign

This gap is specifically:

- correcting broken same-thread continuity visibility and broken primary send-button submit behavior in the current desktop shell

## In Scope

The later H355 implementation run may only:

- make typed input visibly remain in the same main conversation thread during normal dominant-surface use
- make explicit voice continuity, previews, and replies visibly remain in that same main conversation thread during normal dominant-surface use
- wire the arrow/send button so it invokes the same already-live canonical typed submit action as the current keyboard submit path
- preserve the same lawful dominant conversation identity while switching between typing and explicit voice
- preserve the existing H350 shell structure and H354 composer shape unless a tiny same-file correction is needed
- preserve already-wired voice and TTS behavior in the same visible thread
- preserve fail-closed read-only behavior for suspended, quarantined, and non-dominant selections where current repo truth requires it
- stay inside `DesktopSessionShellView.swift`

## Out Of Scope

- no RuntimeBridge changes
- no backend route changes
- no wake-law changes
- no session-law changes
- no identity/access/simulation logic changes
- no generic reopen shortcuts
- no recent-row-driven authority
- no hidden/background recording work
- no new transcript-history fetch route
- no new local message lane
- no sidebar redesign
- no shell redesign away from the approved H350 shell
- no false wake-parity claim
- no separate recording lane

## Files Allowed To Change

The later H355 implementation run may change only:

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

The later H355 implementation run passes only if all of the following are true:

- H354 remains complete for its own approved scope
- the default desktop shell still preserves the H350 layout and visual structure
- typing a message and clicking the arrow/send button keeps the user-authored continuity visible in the same main thread
- explicit voice continuity, previews, and later replies also remain visible in that same main thread
- the arrow/send button invokes the same already-live canonical typed submit action
- conversation identity stays the same while switching input mode
- suspended and quarantined selections remain fail-closed and explanation-only
- non-dominant selections remain read-only wherever current repo truth still requires that boundary
- no RuntimeBridge change, backend route change, wake-law change, session-law change, identity change, access change, or simulation change is introduced
- no new transcript system, no new local message lane, no separate recording lane, and no false wake-parity claim are introduced

## Exact Test Commands

- `git -C /Users/selene/Documents/Selene-OS status --short`
- `git -C /Users/selene/Documents/Selene-OS diff --name-only`
- `rg -n "desktopTypedTurnComposerCard|submitDesktopTypedTurn|stageDesktopTypedTurnRequest|dispatchPreparedTypedTurnRequestIfNeeded" /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`
- `rg -n "explicit_voice_live_preview|explicit_voice_pending_preview|explicit_voice_failed_request_preview|typed_turn_pending_preview|typed_turn_failed_request_preview|runtime_dispatch_failure_preview|authoritative_reply_text" /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`
- `rg -n "TextEditor|arrow.up|desktopVisibleTypedTurnDraft|desktopTypedTurnComposerDraftBinding" /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`
- `rg -n "voiceTurnEndpoint|sessionAttachEndpoint|sessionResumeEndpoint|sessionRecoverEndpoint" /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift`
- `xcodebuild -project /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop.xcodeproj -scheme SeleneMacDesktop -configuration Debug build`
- manual desktop proof:
  - type a message into the H354 composer
  - click the arrow/send button without pressing Return first
  - confirm the typed user-authored continuity remains visible in the same main thread
  - start explicit voice capture from the same dominant conversation
  - stop explicit voice capture
  - confirm explicit voice continuity remains visible in the same main thread
  - confirm suspended or quarantined posture still remains fail-closed if that posture is available
- `git -C /Users/selene/Documents/Selene-OS diff --stat`
- `git -C /Users/selene/Documents/Selene-OS status --short`

## Stop Conditions

Stop immediately in the later H355 implementation run if any of the following becomes true:

- fixing visible same-thread continuity would require RuntimeBridge changes
- fixing arrow/send behavior would require backend route changes
- either fix would require wake/session/identity/access/simulation law changes
- either fix would require widening non-dominant writability
- either fix would require inventing a new local transcript family instead of reusing already-live carriers
- either fix would require inventing a new local send lane instead of using the already-live canonical submit path
- any file outside `DesktopSessionShellView.swift` becomes necessary
- repo truth disproves fixing this inside the current dominant-surface-only shell model

## Ledger Update Rule

No landed-truth update is allowed during this doc-only planning run.

If a later H355 implementation run succeeds, landed-truth updates must:

- preserve H354 as complete for its own approved scope
- state explicitly that H354 landed but failed live product acceptance
- record only the exact H355 corrective same-thread continuity and primary send-button acceptance fix
- state explicitly that the arrow/send button now invokes the already-live canonical typed submit action
- state explicitly that suspended, quarantined, and non-dominant fail-closed behavior remained unchanged
- preserve H354, H353, H352, H350, and H349 truth unchanged outside the bounded H355 seam
- state explicitly that no RuntimeBridge change, backend route change, wake-law change, session-law change, identity change, access change, or simulation change was introduced
- state explicitly that no new transcript system, no new local message lane, no separate recording lane, and no false wake-parity claim were introduced
