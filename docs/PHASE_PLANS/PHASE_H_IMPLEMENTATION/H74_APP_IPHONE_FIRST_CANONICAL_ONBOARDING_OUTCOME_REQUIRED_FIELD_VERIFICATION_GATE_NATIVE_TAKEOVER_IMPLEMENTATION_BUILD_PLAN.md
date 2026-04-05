# H74 APP_IPHONE: First Canonical Onboarding-Outcome Required-Field / Verification-Gate Native Takeover Implementation Build

## Objective

This is the first canonical APP_IPHONE onboarding-outcome required-field / verification-gate native takeover implementation build.

H73 implementation remains live and the native iPhone shell already exposes `onOpenURL`, `EXPLICIT_ENTRY_READY`, `ONBOARDING_ENTRY_ACTIVE`, and exact setup-receipt display including `ios_side_button_configured`.

H72 publication remains live and the post-H71 exact APP_IPHONE side-button producer winner remains `NOT_EXPLICIT`.

H71 publication remains live and the post-H70 exact APP_IPHONE wake-parity winner remains `NOT_EXPLICIT`.

H69 implementation remains live and `APP_IPHONE-05` remains `PROVEN_COMPLETE`.

H70 implementation remains live and `APP_MAC_DESKTOP-05` remains `PROVEN_COMPLETE`.

H67 publication remains live and `APP_IPHONE-06` remains `PROVEN_COMPLETE`.

H68 publication remains live and `APP_MAC_DESKTOP-06` remains `PROVEN_COMPLETE`.

H66 publication remains live and the post-H65 Section 11 next exact winner remains `NOT_EXPLICIT`.

Phase F freeze truth remains live.

Phase G freeze truth remains live.

Current strict dependency order still places `APP_IPHONE` before `APP_MAC_DESKTOP`.

APP_IPHONE remains first-class but non-authority, `EXPLICIT_ONLY`, and cloud-authoritative parity only.

No proven live side-button producer claim is lawful in this run.

No wake parity claim is lawful in this run.

No autonomous unlock claim is lawful in this run.

This run changes only the bounded native iPhone shell plus the authorized master docs.

## Current Repo Truth

Current shared source still preserves `AppVoiceIngressRequest`.

Current shared source still preserves `AppInviteLinkOpenRequest`.

Current shared source still preserves `AppInviteLinkOpenOutcome` with `onboarding_session_id`, `next_step`, `required_fields`, and `required_verification_gates`.

Current shared source still preserves `AppOnboardingContinueAction::PlatformSetupReceipt`.

Current shared source still preserves `RuntimeExecutionEnvelope` inside `AppVoiceIngressRequest`.

Current shared source still preserves the exact iOS receipt family including `install_launch_handshake`, `push_permission_granted`, `notification_token_bound`, and `ios_side_button_configured`.

Current shared source still preserves `VoiceIdentityEmbeddingGateProfiles::mvp_v1_phone_first()` with `ios_explicit` and `ios_wake` `required()`.

Current shared source still preserves `voice_context_ios_explicit()` and `voice_context_ios_wake()`.

Native iPhone source tree and local Xcode project remain exposed in-tree at:

- [project.pbxproj](/Users/selene/Documents/Selene-OS/apple/iphone/SeleneIPhone.xcodeproj/project.pbxproj)
- [Info.plist](/Users/selene/Documents/Selene-OS/apple/iphone/SeleneIPhone/Info.plist)
- [SeleneIPhoneApp.swift](/Users/selene/Documents/Selene-OS/apple/iphone/SeleneIPhone/SeleneIPhoneApp.swift)
- [SessionShellView.swift](/Users/selene/Documents/Selene-OS/apple/iphone/SeleneIPhone/SessionShellView.swift)

Native macOS source tree and local Xcode project remain exposed in-tree at:

- [project.pbxproj](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop.xcodeproj/project.pbxproj)
- [Info.plist](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/Info.plist)
- [SeleneMacDesktopApp.swift](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopApp.swift)
- [DesktopSessionShellView.swift](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift)

The pre-H74 native iPhone shell already preserved `onOpenURL`, `EXPLICIT_ENTRY_READY`, `ONBOARDING_ENTRY_ACTIVE`, and exact setup-receipt display, but still lacked `onboarding_session_id`, `next_step`, `required_fields`, and `required_verification_gates` surfaces.

`APP_IPHONE-04` remains `PARTIAL`.

`APP_IPHONE-05` remains `PROVEN_COMPLETE`.

`APP_IPHONE-06` remains `PROVEN_COMPLETE`.

`APP_MAC_DESKTOP-05` remains `PROVEN_COMPLETE`.

`APP_MAC_DESKTOP-06` remains `PROVEN_COMPLETE`.

APP_MAC_DESKTOP remains later in dependency order and is not selected in this run.

## Implemented Result

This run adds a bounded native onboarding-outcome takeover surface that exposes `onboarding_session_id`, `next_step`, `required_fields`, and `required_verification_gates` in read-only `ONBOARDING_ENTRY_ACTIVE` posture.

Lawful app-open / invite-open URL query items now seed a bounded preview using exact query keys `onboarding_session_id`, `next_step`, repeated `required_field`, and repeated `verification_gate`.

The native shell preserves exact setup-receipt visibility including `install_launch_handshake`, `push_permission_granted`, `notification_token_bound`, and `ios_side_button_configured`.

The shell preserves first-class but non-authority, `EXPLICIT_ONLY`, and cloud-authoritative posture in visible UI state.

This run does not add networking, persistence, runtime request production, local authority, invite activation, onboarding mutation, push-token handling, wake-word behavior, side-button producer wiring, wake parity, or autonomous unlock.

Current repo truth still does not expose a proven live side-button producer, a lawful wake parity claim, or autonomous unlock.

No post-H74 next exact winner is published in this run.

## Out Of Scope

This H74 implementation does not authorize:

- edits to `project.pbxproj` or `Info.plist`
- edits to any Rust source
- runtime-law, engine, contract, or Section 04-11 source work
- APP_IPHONE side-button producer wiring
- APP_IPHONE wake-parity implementation
- autonomous unlock claims
- APP_MAC_DESKTOP work
- post-H74 target publication
