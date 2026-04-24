# H354 APP_MAC_DESKTOP Same-Thread Input-Modality Continuity And Composer Alignment Build Plan

## Purpose

Document the next lawful desktop implementation seam after H353.

H353 is complete for its own approved scope. H353 correctly surfaced already-live pre-ready-time-handoff typed / voice / runtime continuity carriers in the main conversation pane without widening runtime authority. But H353 did not solve the next user-visible gap that still remains inside the H350 shell:

- typing and explicit voice still behave like separate bounded foreground request modes instead of one natural same-thread conversation experience
- the current composer still behaves like a bounded vertically-growing `TextField` rather than a true chat-style multiline composer
- the current composer still tops out below the desired ten-visible-line behavior
- current repo truth still does not prove a separate long-form recording lane, so that feature must remain outside this run

The next lawful desktop seam is therefore not another shell redesign. It is one narrower same-thread conversation input seam:

- preserve H350 shell structure
- preserve H352 dominant-surface inline session-entry work
- preserve H353 pre-ready-time-handoff continuity
- preserve current runtime law
- align the composer and explicit voice behavior so typing and speaking remain one conversation thread for the lawful dominant surface

The canonical repo path for this build and every later implementation command is:

- `/Users/selene/Documents/Selene-OS`

## Authority

This build is governed by:

- H350 completion truth in [MASTER_BUILD_COMPLETION_PLAN.md#L341](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L341) through [MASTER_BUILD_COMPLETION_PLAN.md#L349](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L349)
- H352 completion truth in [MASTER_BUILD_COMPLETION_PLAN.md#L350](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L350) through [MASTER_BUILD_COMPLETION_PLAN.md#L359](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L359)
- H353 completion truth in [MASTER_BUILD_COMPLETION_PLAN.md#L360](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L360) through [MASTER_BUILD_COMPLETION_PLAN.md#L367](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L367)
- H353 completion truth in [MASTER_BUILD_COMPLETION_LEDGER.md#L704](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md#L704)
- text modality law in [02_BUILD_PLAN.md#L48](/Users/selene/Documents/Selene-OS/docs/02_BUILD_PLAN.md#L48) through [02_BUILD_PLAN.md#L51](/Users/selene/Documents/Selene-OS/docs/02_BUILD_PLAN.md#L51)
- PH1.LISTEN delivery-hint-only law in [PH1_LISTEN.md#L53](/Users/selene/Documents/Selene-OS/docs/ECM/PH1_LISTEN.md#L53) through [PH1_LISTEN.md#L58](/Users/selene/Documents/Selene-OS/docs/ECM/PH1_LISTEN.md#L58)
- PH1.L deterministic session lifecycle law in [PH1_L.md#L1](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_L.md#L1) through [PH1_L.md#L21](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_L.md#L21)
- PH1.VOICE.ID fail-closed identity discipline in [PH1_VOICE_ID.md#L1](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_VOICE_ID.md#L1) through [PH1_VOICE_ID.md#L17](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_VOICE_ID.md#L17)
- PH1.ACCESS fail-closed access law in [PH1_ACCESS_001_PH2_ACCESS_002.md#L1](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md#L1) through [PH1_ACCESS_001_PH2_ACCESS_002.md#L21](/Users/selene/Documents/Selene-OS/docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md#L21)
- simulation-gated execution law in [08_SIMULATION_CATALOG.md#L1](/Users/selene/Documents/Selene-OS/docs/08_SIMULATION_CATALOG.md#L1) through [08_SIMULATION_CATALOG.md#L16](/Users/selene/Documents/Selene-OS/docs/08_SIMULATION_CATALOG.md#L16)

## Repo Truth Proof

Current repo truth proves all of the following:

- the H350 shell remains live and keeps the visible sidebar, transcript-first pane, and bottom composer as the default shell baseline in [MASTER_BUILD_COMPLETION_PLAN.md#L341](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L341) through [MASTER_BUILD_COMPLETION_PLAN.md#L349](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L349)
- H352 remains complete and already preserves exact dominant-surface-only main-pane session-entry behavior without widening runtime authority in [MASTER_BUILD_COMPLETION_PLAN.md#L350](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L350) through [MASTER_BUILD_COMPLETION_PLAN.md#L359](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L359)
- H353 remains complete and already preserves exact pre-ready-time-handoff typed / voice / runtime continuity in the main pane without widening runtime authority in [MASTER_BUILD_COMPLETION_PLAN.md#L360](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L360) through [MASTER_BUILD_COMPLETION_PLAN.md#L367](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L367)
- the current desktop composer is still implemented by `desktopTypedTurnComposerCard` in [DesktopSessionShellView.swift#L6065](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6065) through [DesktopSessionShellView.swift#L6168](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6065)
- the current composer still uses `TextField(..., axis: .vertical)` with `.lineLimit(1...6)` rather than a true multiline chat editor in [DesktopSessionShellView.swift#L6089](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6089) through [DesktopSessionShellView.swift#L6099](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6089)
- explicit voice live preview, explicit voice pending preview, explicit voice failure preview, pending typed-turn preview, failed typed-turn preview, runtime dispatch failure preview, and authoritative reply text already appear in the same pre-ready primary pane state in [DesktopSessionShellView.swift#L6834](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6834) through [DesktopSessionShellView.swift#L7047](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L6834)
- explicit voice capture already performs bounded speech-to-text preview through `SFSpeechAudioBufferRecognitionRequest` with `.dictation`, and writes that live preview into `transcriptPreview` in [DesktopSessionShellView.swift#L1569](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L1569) through [DesktopSessionShellView.swift#L1627](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L1569)
- stopping explicit voice currently calls `stopCaptureAndPrepareVoiceTurn()`, which ends capture and converts the bounded transcript preview into one pending explicit voice-turn request in [DesktopSessionShellView.swift#L1488](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L1488) through [DesktopSessionShellView.swift#L1548](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L1488)
- typed-turn staging still fails closed while explicit voice capture or pending voice posture remains active in [DesktopSessionShellView.swift#L17200](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L17200) through [DesktopSessionShellView.swift#L17279](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L17200)
- typed input and voice already reuse the same higher-level text-modality law direction, where text must enter the same pipeline as voice without forking logic, in [02_BUILD_PLAN.md#L48](/Users/selene/Documents/Selene-OS/docs/02_BUILD_PLAN.md#L48) through [02_BUILD_PLAN.md#L51](/Users/selene/Documents/Selene-OS/docs/02_BUILD_PLAN.md#L48)
- current repo truth does not prove any separate long-form recording lane, any separate recording-conversation lane, or any lawful desktop feature that records indefinitely and analyzes later as a distinct conversation path
- current repo truth does not require any `RuntimeBridge` change, backend route change, session-law widening, or synthetic local authority to improve same-thread input-modality continuity plus composer UX alignment inside the one-file desktop shell

Therefore:

- H353 is complete for its own approved scope
- H353 improved pre-ready-time-handoff continuity, but did not solve shared same-thread modality switching
- the next lawful desktop seam is not another shell redesign
- the next lawful desktop seam is same-thread input-modality continuity plus composer UX alignment
- separate long-form recording is not part of this run and must not be smuggled into H354

## CURRENT

Current desktop truth after H353 is:

- H350 shell is live
- H353 pre-ready main-pane continuity is live
- the composer is still built around the current bounded desktop typed-turn composer in `DesktopSessionShellView.swift`
- the current composer still uses a vertically-growing `TextField` rather than a true chat-style multiline text editor
- the current composer still tops out below the desired ten-line visual behavior
- explicit voice currently behaves like an exclusive foreground request posture rather than a fluid same-thread input-mode switch
- typed-turn staging fails while explicit voice capture or pending voice request posture is active
- the user can already speak and type from the same shell, but the experience still behaves like separate bounded request modes instead of one natural chat conversation with input-mode switching
- no current lawful desktop seam proves a separate long-form recording lane yet

## TARGET

The exact H354 target is:

- one current dominant conversation thread remains the same whether the user types or uses explicit voice
- switching between typing and voice changes only the input mode, not the conversation identity
- the main message pane continues to show the same conversation thread while the user changes input mode
- the bottom composer becomes a chat-style multiline composer that grows up to ten visible lines and then scrolls
- the composer visually aligns with the attached reference direction without redesigning the H350 shell
- send remains bounded and canonical
- explicit voice remains bounded and foreground-only
- stopping voice capture ends capture only; it must not imply leaving the same conversation thread
- already-live typed-turn and explicit-voice continuity remain visible in the same main message pane
- no new transcript system is invented
- no new local message lane is invented
- no separate recording-conversation feature is added in this run

## GAP

This gap is not:

- sidebar redesign
- shell redesign
- RuntimeBridge redesign
- typed-turn route redesign
- explicit-voice route redesign

This gap is specifically:

- turning the current bounded typed / voice request surfaces into one same-thread conversation composer experience for the lawful dominant surface

## In Scope

The later H354 implementation run may only:

- keep typing and explicit voice inside the same current lawful conversation thread
- allow the user to switch between typing and explicit voice input without changing the conversation/thread identity
- preserve the current dominant-surface-only authority model
- keep already-live typed-turn and explicit-voice continuity visible in the same main message pane
- redesign the bottom composer to a ChatGPT-like compact composer shape while preserving H350 shell structure
- replace the current single-line-ish composer behavior with a multiline input that auto-expands up to ten visible lines and then scrolls
- keep send and voice controls inside the same composer
- preserve already-wired canonical submit paths exactly as they are
- preserve already-wired voice and TTS behavior in the same visible thread
- preserve fail-closed read-only behavior for suspended / quarantined / non-dominant selections where current repo truth requires it
- stay inside `DesktopSessionShellView.swift`

## Out Of Scope

- no separate long-form recording lane
- no separate "record conversation and analyze later" feature
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
- no sidebar data-model redesign
- no shell redesign away from the approved H350 shell
- no false wake-parity claim

## Files Allowed To Change

The later H354 implementation run may change only:

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

The later H354 implementation run passes only if all of the following are true:

- H353 remains complete for its own approved scope
- the default desktop shell still preserves the H350 layout and visual structure
- one current dominant conversation thread remains the same whether the user types or uses explicit voice
- switching between typing and voice changes only the input mode, not the conversation identity
- the main message pane continues to show the same conversation thread while the user changes input mode
- the bottom composer becomes a chat-style multiline composer that grows up to ten visible lines and then scrolls
- send remains bounded and canonical
- explicit voice remains bounded and foreground-only
- stopping voice capture ends capture only and does not imply leaving the same conversation thread
- already-wired canonical submit paths remain unchanged
- voice and TTS remain in the same visible thread
- suspended and quarantined selections remain fail-closed and explanation-only
- non-dominant selections remain read-only wherever current repo truth still requires that boundary
- no RuntimeBridge change, backend route change, wake-law change, session-law change, identity change, access change, or simulation change is introduced
- no new transcript system, no new local message lane, no separate recording-conversation feature, and no false wake-parity claim are introduced

## Exact Test Commands

- `git -C /Users/selene/Documents/Selene-OS status --short`
- `git -C /Users/selene/Documents/Selene-OS diff --name-only`
- `rg -n "desktopTypedTurnComposerCard|desktopPreReadyConversationPrimaryPaneState|explicitVoiceController|stopCaptureAndPrepareVoiceTurn|submitDesktopTypedTurn" /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`
- `rg -n "TextField\\(|TextEditor\\(|lineLimit\\(|axis: \\.vertical|transcriptPreview" /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`
- `rg -n "failed_desktop_typed_turn_other_voice_request_active|explicit_voice_live_preview|explicit_voice_pending_preview|typed_turn_failed_request_preview|authoritative_reply_text" /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`
- `rg -n "voiceTurnEndpoint|sessionAttachEndpoint|sessionResumeEndpoint|sessionRecoverEndpoint" /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift`
- `xcodebuild -project /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop.xcodeproj -scheme SeleneMacDesktop -configuration Debug build`
- `git -C /Users/selene/Documents/Selene-OS diff --stat`
- `git -C /Users/selene/Documents/Selene-OS status --short`

## Stop Conditions

Stop immediately in the later H354 implementation run if any of the following becomes true:

- same-thread modality switching would require `RuntimeBridge` changes
- same-thread modality switching would require backend route changes
- same-thread modality switching would require wake/session/identity/access/simulation law changes
- same-thread modality switching would require widening non-dominant writability
- same-thread modality switching would require inventing a new local transcript family instead of reusing already-live timeline carriers
- same-thread modality switching would require inventing a new local send lane instead of using already-live canonical submit paths
- any file outside `DesktopSessionShellView.swift` becomes necessary
- repo truth disproves the current dominant-surface-only approach
- preserving voice and TTS in the same visible thread would require new runtime authority rather than already-live shell carriers
- separate recording mode cannot be kept fully out of scope for H354

## Ledger Update Rule

No landed-truth update is allowed during this doc-only planning run.

If a later H354 implementation run succeeds, landed-truth updates must:

- preserve H353 as complete for its own approved scope
- state explicitly that H353 improved pre-ready-time-handoff continuity but did not solve shared same-thread modality switching
- record only the exact H354 same-thread input-modality continuity plus composer alignment landing
- state explicitly that canonical submit paths remained unchanged
- state explicitly that suspended / quarantined fail-closed behavior remained unchanged
- state explicitly that separate long-form recording remained out of scope and did not land here
- preserve H353 / H352 / H350 / H349 / H322 / H321 / H312 truth unchanged outside the bounded H354 seam
- state explicitly that no RuntimeBridge change, backend route change, wake-law change, session-law change, identity change, access change, or simulation change was introduced
- state explicitly that no new transcript system, no new local message lane, no separate recording-conversation feature, and no false wake-parity claim were introduced
