PHASE B1 — ANDROID WAKE / MICROPHONE GAP REVIEW

A) REPO TRUTH CHECK
- `git status --short`: empty
- current branch: `main`
- HEAD commit: `89d64e69b9909497404400313d6567f0acea9e5d`
- exact files reviewed: [A1](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A1_ARTIFACT_TRUST_ARCHITECTURE_REVIEW.md), [A2](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A2_ARTIFACT_IDENTITY_TRUST_CONTRACT_LAYER_BUILD_PLAN.md), [A3](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A3_RUNTIME_VERIFICATION_WIRING_BUILD_PLAN.md), [A4](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A4_PH1J_PROOF_CAPTURE_TRUST_VERIFICATION_OUTCOMES_BUILD_PLAN.md), [A5](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A5_GOV_LAW_ENFORCEMENT_TRUST_FAILURES_BUILD_PLAN.md), [A6](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A6_TESTS_DOCS_VERIFICATION_BUILD_PLAN.md), [WAKE_BUILD_PLAN](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/WAKE_BUILD_PLAN.md), [CORE_ARCHITECTURE](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md), [SELENE_BUILD_EXECUTION_ORDER](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md), [SECTION_03](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md), [SECTION_04](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md), [SECTION_08](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_08.md), [SECTION_09](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md), [SECTION_11](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_11.md), [PH1_OS.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_OS.md), [PH1_GOV.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_GOV.md), [PH1_LAW.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_LAW.md), [PH1_J.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_J.md), [selene_adapter/lib.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_adapter/src/lib.rs), [app_ingress.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs), [ph1os.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1os.rs), [device_artifact_sync.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/device_artifact_sync.rs), [runtime_execution.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs), [ph1art.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1art.rs), [runtime_governance.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_governance.rs), [runtime_law.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_law.rs)
- whether Android-specific code/docs were found and where:
- Android-specific docs were found in [WAKE_BUILD_PLAN](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/WAKE_BUILD_PLAN.md) and in the gap sections of [CORE_ARCHITECTURE](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md#L4105).
- No Android app/module code was found. A targeted search found no `AndroidManifest.xml`, Gradle Android module, Kotlin, or Java source files in the repo outside build artifacts.
- Current repo truth therefore consists of generic cross-platform cloud/runtime code plus Android design intent, not a live Android client implementation.

B) CURRENT ANDROID WAKE / MICROPHONE STATE
- Android wake trigger entry path. CURRENT: design intent says Android uses wake phrase and low-power always-listening mode, but repo truth has no Android client path; [WAKE_BUILD_PLAN#L29](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/WAKE_BUILD_PLAN.md#L29), [WAKE_BUILD_PLAN#L966](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/WAKE_BUILD_PLAN.md#L966). TARGET: lawful Android wake entry that converges into canonical `/v1/voice/turn` and session-first cloud runtime. GAP: no Android producer or app runtime exists.
- Microphone permission handling. CURRENT: no Android permission workflow or manifest exists; only generic device capability posture exists. TARGET: explicit `RECORD_AUDIO` readiness and denial handling. GAP: missing.
- Microphone privacy-toggle handling. CURRENT: no Android-specific handling for device-wide mic-off or silent-audio posture. TARGET: explicit silent-capture detection and readiness state. GAP: missing.
- Foreground service type / permission compliance. CURRENT: no Android service/module exists, so there is no evidence of `foregroundServiceType="microphone"` or `FOREGROUND_SERVICE_MICROPHONE`. TARGET: compliant microphone FGS contract. GAP: missing.
- Background-start restrictions relevance. CURRENT: highly relevant; repo has no lawful Android background-start model. TARGET: explicit Android wake legality state machine. GAP: missing.
- `BOOT_COMPLETED` limitations relevance. CURRENT: highly relevant; repo has no Android boot path and no explicit boot restriction contract. TARGET: boot behavior that does not violate Android mic FGS restrictions. GAP: missing.
- Wake-lock usage / exposure. CURRENT: no Android wake-lock code or contract found. TARGET: bounded, justified, user-visible wake-lock use only if unavoidable. GAP: undefined.
- Audio-input sharing / contention handling. CURRENT: no Android capture-session contract for silence, preemption, or contention. TARGET: explicit contention posture and fallback behavior. GAP: missing.
- Adapter normalization posture. CURRENT: adapter derives non-authoritative capture posture and platform trust class only; [lib.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_adapter/src/lib.rs). TARGET: Android-specific posture normalization only, not authority. GAP: Android-specific inputs are absent.
- PH1.OS posture handling. CURRENT: PH1.OS uses attestation and non-authoritative capture posture for wake entry gating; [ph1os.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1os.rs), [PH1_OS.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_OS.md). TARGET: keep PH1.OS upstream only. GAP: no Android-specific posture source exists yet.
- Ingress/runtime transport posture. CURRENT: canonical envelope and Phase A trust/proof surfaces exist; [runtime_execution.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs). TARGET: Android client feeds only canonical refs/inputs. GAP: no Android client exists to do so.
- Section 04 boundary compliance. CURRENT: compliant. The repo keeps cloud authority and does not show Android-side authority logic. TARGET: remain compliant. GAP: none in live code; future implementation risk only.
- Proof/governance/law boundary compliance. CURRENT: compliant. Android-specific code is absent, so no second proof or enforcement path exists. TARGET: remain compliant. GAP: future implementation risk only.
- Battery / doze / restricted-app handling. CURRENT: design budgets exist, but no Android operational implementation exists; [WAKE_BUILD_PLAN#L96](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/WAKE_BUILD_PLAN.md#L96), [WAKE_BUILD_PLAN#L1004](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/WAKE_BUILD_PLAN.md#L1004). TARGET: explicit bounded operational posture under Doze, restricted app, and battery optimization. GAP: missing.
- Parity against frozen Phase A model. CURRENT: repo truth already says parity is incomplete; [CORE_ARCHITECTURE#L4105](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md#L4105), [CORE_ARCHITECTURE#L4137](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md#L4137), [WAKE_BUILD_PLAN#L966](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/WAKE_BUILD_PLAN.md#L966). TARGET: Android consumes the frozen Phase A trust/proof/enforcement stack. GAP: still open.

C) GAP TAXONOMY
- `PLATFORM_RESTRICTION_GAP`: Android background-start, `BOOT_COMPLETED`, Doze, restricted-app, and FGS limits are not formalized in repo truth.
- `PERMISSION_GAP`: no Android `RECORD_AUDIO` readiness workflow or manifest evidence exists.
- `PRIVACY_TOGGLE_GAP`: no handling for Android 12+ device-wide microphone-off silent-audio posture.
- `FGS_COMPLIANCE_GAP`: no Android microphone foreground service declaration or runtime path exists.
- `WAKE_LOCK_GAP`: no Android wake-lock contract or bounded-use discipline is implemented.
- `ATTESTATION_GAP`: Android capture-bundle attestation / receipt path is still open relative to parity goals.
- `CAPTURE_PARITY_GAP`: desktop capture parity exists; Android microphone producer does not.
- `BATTERY_POLICY_GAP`: budgets exist in docs, but no Android operational posture contract exists for battery optimization, restricted-app state, or Doze.
- `AUDIO_CONTENTION_GAP`: no Android audio-input sharing/contention handling contract exists.
- `A1_A6_ALIGNMENT_GAP`: future Android work could drift if B2 does not explicitly force Phase A transport/proof/enforcement reuse.

D) PHASE A NON-REGRESSION CHECK
- Section 04 sole first-time verifier ownership: current live violation `NO`. Risk exists only if Phase B tries to make Android wake or mic readiness an authority decision.
- A3 canonical trust transport: current live violation `NO`. Android-specific transport does not exist yet.
- A4 canonical proof transport: current live violation `NO`. Android-specific proof path does not exist yet.
- A5 canonical GOV/LAW consumption: current live violation `NO`. No Android-specific enforcement path exists.
- no raw adapter/PH1.OS hint as trust truth: current live violation `NO`, but there is implementation sensitivity because PH1.OS already gates wake entry using non-authoritative capture posture. That must remain upstream posture only.
- no second trust/proof/enforcement path: current live violation `NO`. Future risk is high if B2/B3 are not strict.

E) PLATFORM-LAW COMPLIANCE REVIEW
- Microphone foreground service requirements: repo truth is unproven/incomplete for Android. Official Android docs require a microphone FGS type, `FOREGROUND_SERVICE_MICROPHONE`, `FOREGROUND_SERVICE_TYPE_MICROPHONE`, and `RECORD_AUDIO` for background mic capture; [FGS types required](https://developer.android.com/about/versions/14/changes/fgs-types-required). No Android module or manifest exists in the repo to satisfy this.
- Background-start restrictions: current repo truth does not define a lawful Android background microphone start path. Android 12+ restricts background FGS starts, and Android 14+ blocks creating microphone FGS from background without the narrow allowed situations; [FGS background-start restrictions](https://developer.android.com/develop/background-work/services/fgs/restrictions-bg-start).
- `BOOT_COMPLETED` microphone restrictions: current repo truth does not define a compliant boot-start model. Android docs state apps targeting Android 14+ are not allowed to launch a microphone FGS from `BOOT_COMPLETED`; [Android foreground service type changes](https://developer.android.com/about/versions/15/changes/foreground-service-types).
- Wake-lock best-practice expectations: current repo truth has no Android wake-lock implementation and therefore no compliance evidence. Android guidance is explicit that wake locks should be minimized, short-lived, and normally coupled with a visible foreground service if used at all; [wake lock best practices](https://developer.android.com/develop/background-work/background-tasks/awake/wakelock/best-practices).
- Audio-input contention realities: current repo truth is incompatible with platform reality because it has no Android contention contract. Android can mute one recorder and deliver silence depending on privacy-sensitive sources, UI visibility, and call state; [Sharing audio input](https://developer.android.com/media/platform/sharing-audio-input).
- Privacy/permission workflow expectations: current repo truth is incomplete. Android 12+ device-wide microphone-off can yield silent audio, and users must be shown/handled through proper permission/privacy workflow; [sensitive access / microphone toggle](https://developer.android.com/training/permissions/explaining-access).
- Battery / Doze / restricted-app expectations: current repo truth is incomplete. Android restricted-app state can block background operation and even suppress `BOOT_COMPLETED` delivery for newer targets; [background optimization](https://developer.android.com/topic/performance/background-optimization), [Doze and App Standby](https://developer.android.com/training/monitoring-device-state/doze-standby).

F) B1 FINDINGS
- P0 blockers
- NONE
- P1 serious gaps
- No Android app/module exists in the repo, so Android wake/microphone behavior cannot currently be claimed compliant or even live.
- Android microphone FGS compliance is entirely undefined in implementation terms: no manifest, no service type, no permission path, no background-start model.
- The repo explicitly documents Android microphone/runtime parity as open; [WAKE_BUILD_PLAN#L966](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/WAKE_BUILD_PLAN.md#L966), [CORE_ARCHITECTURE#L4137](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md#L4137).
- There is no Android privacy-toggle, silent-audio, or audio-contention contract, even though Android platform behavior makes those cases normal rather than edge cases.
- There is no Android battery/Doze/restricted-app operating contract, which matters directly for always-listening wake.
- There is no Android attestation/receipt/capture parity implementation path that cleanly feeds the frozen Phase A trust/proof stack.
- P2 normal gaps
- Wake-lock usage is not formalized for Android.
- Android onboarding/platform receipts are described only at a high level and not yet tied to a concrete Android runtime readiness model.
- The repo has generic cross-platform runtime code, but no Android-specific operational posture source to feed it.
- optional polish
- A dedicated Phase B doc tree does not yet exist; Android wake work still lives mainly in the older [WAKE_BUILD_PLAN](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/WAKE_BUILD_PLAN.md).

G) WHAT B2 MUST DEFINE
- Android wake eligibility contract
- Exact lawful conditions under which Android may listen, stay listening, stop listening, resume, and fall back to explicit-only mode.
- Android microphone readiness contract
- `RECORD_AUDIO`, privacy-toggle state, foreground/while-in-use status, FGS legality, and restricted-app/battery state as one readiness model.
- Android operational posture contract
- Doze, battery optimization, restricted-app state, boot behavior, wake-lock exposure, and notification/foreground visibility posture.
- Android capture session contract
- Capture start/stop, silence/contention signaling, attestation/receipt inputs, failure states, and canonical ingress packaging.
- parity/fallback contract
- How Android achieves parity with desktop where lawful, and how it degrades to explicit-only or blocked behavior where Android forbids always-listening mic behavior.
- Android onboarding/platform-receipt contract
- Exact Android readiness receipts required before wake can be enabled.
- Android Phase A consumption contract
- Explicit rule that Android may only feed Phase A canonical transport/proof/enforcement surfaces and may not add a second trust/proof/enforcement path.

H) BUILD ORDER RECOMMENDATION
- whether B2 should proceed next: `YES`
- whether any prerequisite repo cleanup is required first: `NO`
- whether any live implementation should start before B2: `NO`

I) FINAL VERDICT
- PROCEED TO B2

Completed in this chat.
Test command.
`git status --short`
Expected output.
`empty`
Next step.
Write B2 as the formal Android wake eligibility, microphone readiness, operational posture, capture session, and parity/fallback contract phase against the frozen Phase A baseline.
Next file to open.
`/Users/xiamo/Documents/A-Selene/Selene-OS/docs/WAKE_BUILD_PLAN.md`

**WORLD-CLASS UPGRADE — Implementation-Ready Android Module Baseline**
- The current B1 review remains the repo-truth baseline. The following upgrade is additive execution guidance for B2 and later implementation, not a claim that these Android surfaces already exist in the repo.
- Recommended future Android module skeleton:
- `/Users/xiamo/Documents/A-Selene/Selene-OS/android/selene_android_wake/`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/android/selene_android_wake/app/src/main/AndroidManifest.xml`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/android/selene_android_wake/app/src/main/java/.../WakeForegroundService.kt`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/android/selene_android_wake/app/src/main/java/.../WakeEligibilityCoordinator.kt`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/android/selene_android_wake/app/src/main/java/.../MicrophoneReadinessCoordinator.kt`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/android/selene_android_wake/app/src/main/java/.../CaptureSessionCoordinator.kt`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/android/selene_android_wake/app/src/main/java/.../AndroidWakeReceiptEmitter.kt`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/android/selene_android_wake/app/src/main/java/.../AndroidOperationalPostureEmitter.kt`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/android/selene_android_wake/app/build.gradle.kts`
- Recommended module responsibility split:
- Android module owns device-local wake eligibility, microphone readiness, operational posture, and capture-session orchestration only.
- The Android module must never become a trust authority, proof authority, governance authority, or runtime-law authority.
- Adapter and PH1.OS remain upstream-only normalization layers consistent with [A3](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A3_RUNTIME_VERIFICATION_WIRING_BUILD_PLAN.md), [A4](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A4_PH1J_PROOF_CAPTURE_TRUST_VERIFICATION_OUTCOMES_BUILD_PLAN.md), and [A5](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A5_GOV_LAW_ENFORCEMENT_TRUST_FAILURES_BUILD_PLAN.md).

**WORLD-CLASS UPGRADE — Android Manifest, Permission, and Foreground-Service Contract**
- Future Android implementation must declare only the minimum lawful manifest surface required for microphone capture and wake orchestration.
- Minimum expected manifest/permission surface:
- `android.permission.RECORD_AUDIO`
- `android.permission.FOREGROUND_SERVICE`
- `android.permission.FOREGROUND_SERVICE_MICROPHONE`
- `android.permission.POST_NOTIFICATIONS` where required for visible ongoing FGS disclosure on newer Android versions
- `android.permission.RECEIVE_BOOT_COMPLETED` only if boot-time receipt restoration or readiness re-evaluation is needed; it must not be used as authority to start microphone capture
- Microphone capture must run only inside a foreground service declared with `android:foregroundServiceType="microphone"` and started with the corresponding microphone FGS runtime type where the platform requires it.
- Android 14+ microphone foreground-service requirements and Android 12+ background-start restrictions are binding platform law for B2/B3 implementation, not optional guidance: [Foreground service types are required](https://developer.android.com/about/versions/14/changes/fgs-types-required), [Restrictions on starting a foreground service from the background](https://developer.android.com/develop/background-work/services/fgs/restrictions-bg-start).
- The Android implementation must default to `explicit-only` posture whenever lawful microphone FGS start conditions are not satisfied.

**WORLD-CLASS UPGRADE — Android Wake Eligibility and Background-Start Legality State Machine**
- B2 must formalize one explicit Android wake legality state machine. Minimum recommended states:
- `NOT_CONFIGURED`
- `CONFIGURED_BLOCKED_PERMISSION`
- `CONFIGURED_BLOCKED_PRIVACY_TOGGLE`
- `CONFIGURED_BLOCKED_FGS_ILLEGAL`
- `CONFIGURED_BLOCKED_RESTRICTED_APP`
- `CONFIGURED_BLOCKED_BACKGROUND_START`
- `CONFIGURED_BLOCKED_BOOT_RESTRICTION`
- `CONFIGURED_EXPLICIT_ONLY`
- `CONFIGURED_WAKE_ELIGIBLE`
- Legal microphone wake start paths must be narrowly defined and limited to platform-lawful cases such as visible app state or other explicitly allowed Android exemptions. Background dormant auto-start must not be assumed lawful.
- `BOOT_COMPLETED` must be treated as receipt restoration / readiness recomputation only. For target SDK 34+, it must not be used to launch a microphone foreground service; [Android foreground service type changes](https://developer.android.com/about/versions/15/changes/foreground-service-types).
- Background-start legality must be evaluated before any microphone service creation, not after failure.
- If wake eligibility cannot be proven under current Android restrictions, the device must fall back to `CONFIGURED_EXPLICIT_ONLY` instead of attempting hidden capture.

**WORLD-CLASS UPGRADE — Microphone Readiness, Privacy Toggle, and Silent-Audio Contract**
- B2 must formalize one Android microphone readiness contract with at least these posture classes:
- `MIC_READY`
- `MIC_PERMISSION_DENIED`
- `MIC_PERMISSION_REVOKED`
- `MIC_PRIVACY_TOGGLE_OFF`
- `MIC_SILENT_AUDIO_DETECTED`
- `MIC_FGS_START_NOT_ALLOWED`
- `MIC_CONTENTION_PREEMPTED`
- `MIC_DEVICE_RESTRICTED`
- `MIC_UNKNOWN_UNSAFE`
- Privacy-toggle off and device-level microphone disablement must not be modeled as generic capture failure. They are distinct platform posture states and must feed the canonical upstream posture path only as non-authoritative hints.
- Silence from Android audio APIs must be treated as an explicit operational posture subject, not as implicit success. Silent-audio detection must be available to B2/B3 as a first-class downgrade trigger.
- The Android implementation must degrade gracefully when permission is denied or revoked, consistent with Android privacy guidance: [Explain access to sensitive information](https://developer.android.com/training/permissions/explaining-access), [Privacy checklist](https://developer.android.com/privacy-and-security/about).

**WORLD-CLASS UPGRADE — Wake-Lock Exposure and Foreground Visibility Rules**
- Wake locks must not be used as a substitute for lawful foreground microphone service execution.
- Any wake-lock use must be:
- short-lived
- justified by a specific transition or capture lifecycle step
- coupled to visible/ongoing foreground service state where microphone capture is active
- explicitly released on every stop, failure, downgrade, explicit-only transition, or restricted-app transition
- B2 should treat wake-lock exposure as an operational receipt subject and not as a trust, proof, or enforcement signal.
- Android wake-lock behavior must follow bounded-use expectations, not indefinite hidden retention: [Wake lock best practices](https://developer.android.com/develop/background-work/background-tasks/awake/wakelock/best-practices).

**WORLD-CLASS UPGRADE — Battery, Doze, Restricted-App, and Operational Posture Contract**
- B2 must define one Android operational posture contract covering:
- battery-optimization state
- Doze/App Standby state
- restricted-app state
- foreground visibility state
- notification visibility state
- background-start legality state
- boot-restoration legality state
- These posture facts remain upstream only and may feed adapter / PH1.OS normalization, but may never become Section 04 authority or A5 enforcement truth.
- The Android operational posture contract must explicitly model fallback to `explicit-only` mode when battery policy, restricted-app state, or Doze prevents lawful always-listening behavior.
- Platform constraints here are real and cannot be hand-waved away: [Background optimization](https://developer.android.com/topic/performance/background-optimization), [Doze and App Standby](https://developer.android.com/training/monitoring-device-state/doze-standby).

**WORLD-CLASS UPGRADE — Capture Session Lifecycle and Contention Contract**
- B2 must formalize a capture-session lifecycle with at least:
- `SESSION_NOT_READY`
- `SESSION_READY`
- `SESSION_START_REQUESTED`
- `SESSION_START_CONFIRMED`
- `SESSION_ACTIVE`
- `SESSION_SILENT`
- `SESSION_CONTENTION_DETECTED`
- `SESSION_STOP_REQUESTED`
- `SESSION_STOPPED`
- `SESSION_FALLBACK_EXPLICIT_ONLY`
- `SESSION_BLOCKED_PLATFORM`
- Each session transition must produce canonical Android operational receipts and posture hints for upstream carriage only.
- Audio-input sharing/contention must be modeled explicitly because Android may mute or deprioritize recorders under privacy-sensitive contention conditions; [Sharing audio input](https://developer.android.com/media/platform/sharing-audio-input).
- Android capture session output must feed the same canonical cloud ingress path already required by [SECTION_03](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md) and Phase A A3 transport, not a mobile-specific parallel path.

**WORLD-CLASS UPGRADE — Phase A Integration and Non-Authority Rules**
- All Android outputs must terminate in Phase A canonical surfaces only:
- trust/transport: [A3](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A3_RUNTIME_VERIFICATION_WIRING_BUILD_PLAN.md)
- proof: [A4](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A4_PH1J_PROOF_CAPTURE_TRUST_VERIFICATION_OUTCOMES_BUILD_PLAN.md)
- GOV/LAW enforcement: [A5](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A5_GOV_LAW_ENFORCEMENT_TRUST_FAILURES_BUILD_PLAN.md)
- Android module may emit:
- artifact references
- attestation refs
- readiness receipts
- operational posture refs
- capture-session refs
- non-authoritative hints
- Android module may not emit:
- `artifact_trust_state`
- `ArtifactTrustDecisionRecord`
- `ArtifactTrustProofEntry`
- final GOV or LAW enforcement posture
- Adapter normalization remains non-authoritative.
- PH1.OS posture handling remains upstream-only, consistent with [SECTION_08](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_08.md) and [PH1_OS.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_OS.md).
- Section 04 remains the sole first-time authoritative verifier and must never move to Android.

**WORLD-CLASS UPGRADE — Desktop/Android Parity and Fallback Contract**
- B2 must define parity as “same canonical ingress, trust, proof, and enforcement surfaces,” not “same platform mechanics.”
- Android parity is lawful parity, not forced behavioral parity.
- If Android platform restrictions prohibit always-listening microphone behavior under current posture, the lawful fallback is `explicit-only` mode, not hidden best-effort background capture.
- Desktop parity must not pressure Android implementation into violating Android background-start, FGS, or privacy rules.
- B2 must explicitly distinguish:
- `desktop-full-parity`
- `android-lawful-parity`
- `android-explicit-only-fallback`
- `android-blocked-platform-posture`

**WORLD-CLASS UPGRADE — Android Onboarding, Receipt, and Attestation Alignment**
- B2 must define Android onboarding/platform receipts that prove readiness without acting as authority:
- permission-granted receipt
- privacy-toggle-on receipt
- FGS-capable receipt
- battery-policy acknowledgment receipt
- restricted-app-state receipt
- boot-restoration receipt
- capture-session start/stop receipt
- These receipts may feed adapter and PH1.OS normalization only.
- They must not bypass Phase A trust or proof transport.
- Android attestation/receipt design must remain compatible with the frozen Phase A baseline and the current repo’s canonical `artifact_trust_inputs` / `artifact_trust_state` split in [/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs).

**WORLD-CLASS UPGRADE — Developer Implementation Checklist**
- Future Android implementation must include:
- one Android manifest with explicit microphone FGS declarations
- one lawful background-start eligibility evaluator
- one microphone readiness evaluator
- one operational posture emitter
- one capture-session coordinator
- one adapter-facing normalization input mapper
- zero authority logic on device
- zero proof emission on device
- zero GOV/LAW enforcement logic on device
- zero direct use of raw Android posture as final trust truth

**WORLD-CLASS UPGRADE — Risk-to-Phase Mapping**
- `FGS_COMPLIANCE_GAP` -> Phase A dependency: A3 ingress carriage and A5 no-raw-field enforcement. Severity: high. Mitigation: explicit-only fallback and lawful FGS contract.
- `PRIVACY_TOGGLE_GAP` -> Phase A dependency: A3 non-authoritative posture transport. Severity: high. Mitigation: explicit microphone readiness state and silent-audio detection.
- `BOOT_COMPLETED` restriction gap -> Phase A dependency: A3 operational posture, A5 release/enforcement truth. Severity: high. Mitigation: boot restores readiness only, never microphone FGS start.
- `AUDIO_CONTENTION_GAP` -> Phase A dependency: A3 capture-session posture carriage and A5 downgrade/block handling. Severity: medium-high. Mitigation: explicit contention posture and session fallback.
- `ATTESTATION_GAP` -> Phase A dependency: A3 canonical trust inputs and A4 proof readiness later. Severity: high. Mitigation: Android receipt/attestation contract without authority drift.
- `BATTERY_POLICY_GAP` -> Phase A dependency: A3 operational posture only. Severity: high. Mitigation: explicit restricted-app/Doze/battery state machine and fallback.
- `A1_A6_ALIGNMENT_GAP` -> Phase A dependency: all frozen phases. Severity: high. Mitigation: B2 must explicitly restate that Android is a consumer of Phase A, not a redefinition of it.

**WORLD-CLASS UPGRADE — Optional Enterprise-Grade Polish**
- Assign stable traceability IDs to Android wake eligibility, microphone readiness, FGS activation, privacy-toggle transitions, Doze transitions, restricted-app transitions, capture-session start/stop, and fallback transitions.
- Emit explicit operational logging for:
- FGS start accepted / denied
- privacy-toggle off / on
- silent-audio detection
- audio contention / preemption
- Doze / restricted-app posture changes
- fallback to explicit-only mode
- Add freeze / rollback detection hooks for device-side readiness contracts so B2/B3 can prove that stale readiness or stale configuration is not silently reused.

**WORLD-CLASS UPGRADE — Baseline Snapshot Clarification**
- The baseline line at [B1 gap review line 74](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE/B1_ANDROID_WAKE_MICROPHONE_GAP_REVIEW.md#L74) stating that a dedicated Phase B doc tree did not yet exist is a historical baseline snapshot taken at review time, not a current repo-fact assertion.
- The existence of `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE/` does not invalidate the original baseline review; it means the gap review has now been promoted into the formal Phase B planning tree.

**WORLD-CLASS UPGRADE — Android API-Level Legality Matrix**
- B2 and later implementation must treat Android wake legality as a product of both platform version and current execution posture, not as one timeless rule.

| Android API band | App visibility | Trigger source | Microphone capture intent | FGS start context | Allowed posture |
| --- | --- | --- | --- | --- | --- |
| Android 12 | Visible foreground UI | Visible UI action / notification action / quick settings tile | immediate microphone capture | foreground-visible start | `WAKE_ELIGIBLE` |
| Android 12 | Background / not user-visible | wake phrase / widget / external trigger / boot | always-listening or immediate microphone capture | background FGS start required | `EXPLICIT_ONLY` unless platform-lawful exemption is proven |
| Android 13 | Visible foreground UI | Visible UI action / notification action / quick settings tile | immediate microphone capture | foreground-visible start | `WAKE_ELIGIBLE` |
| Android 13 | Background / not user-visible | wake phrase / widget / external trigger / boot | always-listening or immediate microphone capture | background FGS start required | `EXPLICIT_ONLY` unless platform-lawful exemption is proven |
| Android 14 | Visible foreground UI | Visible UI action / notification action / quick settings tile | immediate microphone capture | microphone FGS start from allowed visible context | `WAKE_ELIGIBLE` |
| Android 14 | Background / not user-visible | wake phrase / widget / external trigger | always-listening or immediate microphone capture | microphone FGS start from background | `BLOCKED` unless a documented Android exemption clearly applies |
| Android 14 | Boot/restart path | boot/restart | microphone capture start | microphone FGS auto-start from boot | `BLOCKED` |
| Android 15+ | Visible foreground UI | Visible UI action / notification action / quick settings tile | immediate microphone capture | microphone FGS start from allowed visible context | `WAKE_ELIGIBLE` |
| Android 15+ | Background / not user-visible | wake phrase / widget / external trigger | always-listening or immediate microphone capture | background microphone FGS start | `BLOCKED` unless a documented Android exemption clearly applies |
| Android 15+ | Boot/restart path | boot/restart | microphone capture start | microphone FGS auto-start from boot | `BLOCKED` |

- This matrix is minimum legality guidance, not permission to invent loopholes. Where Android platform law is ambiguous, B2/B3 must choose the more conservative posture.

**WORLD-CLASS UPGRADE — Recoverable vs Non-Recoverable Posture Classes**
- Android wake and microphone posture must be classified into exactly one recoverability family:
- `recoverable-by-user-action`
- `recoverable-by-foreground-transition`
- `recoverable-by-policy-change`
- `hard-blocked-under-current-platform-law`
- Minimum classification guidance:
- `MIC_PERMISSION_DENIED`, `MIC_PERMISSION_REVOKED`, `MIC_PRIVACY_TOGGLE_OFF` -> `recoverable-by-user-action`
- `MIC_FGS_START_NOT_ALLOWED`, `CONFIGURED_BLOCKED_BACKGROUND_START` -> `recoverable-by-foreground-transition`
- `CONFIGURED_BLOCKED_RESTRICTED_APP`, battery-optimization denial, enterprise/admin microphone disablement -> `recoverable-by-policy-change`
- `CONFIGURED_BLOCKED_BOOT_RESTRICTION`, targetSdk/API-level illegal microphone FGS boot start, platform-forbidden background microphone capture -> `hard-blocked-under-current-platform-law`
- This recoverability class is operational and advisory only. It may influence Android fallback posture, but it may never become Section 04 authority, A4 proof truth, or A5 enforcement truth.

**WORLD-CLASS UPGRADE — Android-to-Phase-A Mapping Appendix**
- Android outputs must map into frozen Phase A surfaces exactly as follows:

| Android-origin output | Canonical Phase A destination | Notes |
| --- | --- | --- |
| wake/capture artifact references | A3 artifact refs / `artifact_trust_inputs` carrier | non-authoritative carriage only |
| capture attestation refs | A3 upstream attestation refs | used only as normalized trust input |
| readiness receipts | A3 operational / readiness receipt refs | no device-side authority semantics |
| operational posture refs | A3 pre-verification posture refs | adapter and PH1.OS normalize only |
| capture-session refs | A3 capture-session refs | used to correlate runtime state, not to create authority |

- Device-side persistence or reuse of the following is forbidden:
- `artifact_trust_state`
- `ArtifactTrustDecisionRecord`
- `ArtifactTrustProofEntry`
- `proof_entry_ref`
- `proof_record_ref`
- Android may never persist or replay canonical cloud trust/proof outputs as if they were device authority.

**WORLD-CLASS UPGRADE — Android Audio-Contention Matrix**

| Contention case | Emitted posture | Receipt / event expectation | Fallback state |
| --- | --- | --- | --- |
| phone call active | `MIC_CONTENTION_PREEMPTED` | contention receipt + route/call-state event | `EXPLICIT_ONLY` |
| other recorder active | `MIC_CONTENTION_PREEMPTED` or `MIC_SILENT_AUDIO_DETECTED` | contention receipt | `retry-later-operationally` or `EXPLICIT_ONLY` |
| privacy-sensitive competing source | `MIC_SILENT_AUDIO_DETECTED` | silent-audio receipt + competing-source event if visible | `EXPLICIT_ONLY` |
| Bluetooth route change | `MIC_UNKNOWN_UNSAFE` until route stabilizes | route-change event + readiness recompute receipt | `retry-later-operationally` |
| headset transition | `MIC_UNKNOWN_UNSAFE` until route stabilizes | device-route transition receipt | `retry-later-operationally` |
| silent-audio delivery | `MIC_SILENT_AUDIO_DETECTED` | silent-audio detection receipt | `EXPLICIT_ONLY` |

- Android contention handling must remain operational-only and must feed Phase A as non-authoritative capture/posture input.

**WORLD-CLASS UPGRADE — Recovery Rules for Doze, Restricted-App, and Battery Transitions**
- Entering Doze:
- if current path was lawful only because the app was visibly foregrounded and remains so, posture may remain `WAKE_ELIGIBLE`
- if always-listening legality becomes uncertain, posture must become `EXPLICIT_ONLY`
- if the app can no longer lawfully or reliably sustain wake readiness, posture becomes `BLOCKED` until a fresh readiness receipt exists
- Leaving Doze:
- posture must not silently return to `WAKE_ELIGIBLE`; readiness must be recomputed and a fresh readiness receipt must exist first
- Restricted-app transition:
- posture becomes `BLOCKED` or `EXPLICIT_ONLY` depending on whether explicit visible action remains lawful; it must not silently resume wake eligibility
- Battery optimization whitelist change:
- moving off whitelist may force `EXPLICIT_ONLY` or `BLOCKED`
- moving onto whitelist does not by itself restore `WAKE_ELIGIBLE`; a fresh readiness receipt is still required
- App standby downgrade:
- posture must degrade conservatively to `EXPLICIT_ONLY` or `BLOCKED` until a fresh readiness receipt exists and lawful start conditions are re-established

**WORLD-CLASS UPGRADE — Boot-Restoration Action Table**

| Boot / restart action | May do | Must never do |
| --- | --- | --- |
| restore persisted config | restore user preference, feature flag, and prior non-sensitive wake configuration | restore active microphone capture session as if uninterrupted |
| recompute readiness | recompute permission, privacy-toggle, restricted-app, battery, and legality posture | assume prior readiness remains valid without recomputation |
| emit boot-restoration receipt | emit one explicit boot-restoration receipt for upstream normalization | treat boot receipt as trust, proof, or enforcement authority |
| schedule user-visible recovery prompt | schedule or present visible recovery prompt where lawful and needed | silently background-start microphone capture |
| microphone FGS auto-start | never | always forbidden on this plan baseline |

**WORLD-CLASS UPGRADE — Parity Acceptance Criteria**
- `desktop-full-parity`: desktop delivers full lawful wake behavior and canonical Phase A transport/proof/enforcement integration.
- `android-lawful-parity`: Android matches desktop on canonical ingress/trust/proof/enforcement surfaces while remaining inside Android platform law.
- `android-explicit-only-fallback`: acceptable release target when Android cannot lawfully sustain always-listening wake but still supports explicit user-initiated capture through the canonical Phase A path.
- `android-blocked-platform-posture`: acceptable temporary release target only when Android platform or managed-device posture forbids lawful microphone wake entirely.
- Acceptable release targets for later B3/B4 are `android-lawful-parity` and `android-explicit-only-fallback`. `android-blocked-platform-posture` is acceptable only when the block is truthful, explicit, and user-visible.

**WORLD-CLASS UPGRADE — Layer-Boundary Implementation Map**
- `WakeForegroundService`
- owns only lawful foreground microphone service lifecycle, notification visibility, and wake/session coupling
- `WakeEligibilityCoordinator`
- owns only Android legality evaluation for wake and start contexts
- `MicrophoneReadinessCoordinator`
- owns only permission/privacy/FGS/readiness posture
- `CaptureSessionCoordinator`
- owns only capture-session lifecycle, route transitions, silence detection, and contention handling
- receipt / posture emitters / bridges
- own only normalized receipts, posture refs, traceability IDs, and upstream bridge packaging
- The following must never leak into adapter, PH1.OS, or Section 04 except as normalized non-authoritative inputs:
- platform-law decisions as authority
- trust/proof/enforcement decisions
- cached canonical trust/proof refs
- Adapter remains non-authoritative normalization only.
- PH1.OS remains upstream posture handling only.
- Section 04 remains sole first-time authoritative verifier and consumes only normalized upstream inputs.

**WORLD-CLASS UPGRADE — Minimum Event Schema**
- Minimum Android operational event schema:
- `event_id`
- `traceability_id`
- `event_type`
- `observed_at`
- `session_ref` where applicable
- `capture_session_ref` where applicable
- `device_posture_class`
- `wake_posture_class`
- `microphone_readiness_class`
- `trigger_source`
- `api_level`
- `target_sdk`
- `user_visible_state`
- `fgs_start_context`
- `resulting_allowed_posture`
- Required event families:
- wake eligibility evaluation
- FGS start accepted / denied
- privacy-toggle transitions
- silent-audio detection
- contention events
- Doze / restricted-app transitions
- fallback transitions
- boot-restoration receipts

**WORLD-CLASS UPGRADE — Readiness Invalidation Triggers**
- Cached readiness must be invalidated on:
- permission revocation
- privacy-toggle change
- target-SDK or platform update
- restricted-app transition
- boot or restart
- FGS capability change
- audio-route or microphone-source capability change where legality/readiness may differ
- After invalidation, Android may not silently reuse prior readiness. A fresh readiness receipt is required before any return to `WAKE_ELIGIBLE`.

**WORLD-CLASS UPGRADE — Target-SDK and Platform-Law Matrix**

| targetSdk / platform posture | Legal sensitivity | Required interpretation |
| --- | --- | --- |
| targetSdk 31–33 on Android 12–13 | background FGS starts restricted | default to conservative `EXPLICIT_ONLY` when visibility/legal exemption is unclear |
| targetSdk 34 on Android 14 | microphone FGS type enforcement and background-start restrictions tightened | treat background microphone FGS as `BLOCKED` unless explicitly documented lawful |
| targetSdk 35+ on Android 15+ | microphone FGS and boot/start limits remain strict or stricter | do not assume legacy behavior; re-evaluate legality against current platform law |

- B2/B3 must key legality to both OS level and `targetSdkVersion`. Platform version alone is not enough.

**WORLD-CLASS UPGRADE — Trigger-Source Legality Matrix**

| Trigger source | Default legality treatment |
| --- | --- |
| wake phrase | `EXPLICIT_ONLY` unless Android capability tier and current visible/allowed posture make continuous listening lawful |
| visible UI action | `WAKE_ELIGIBLE` when permission/privacy/FGS requirements are satisfied |
| notification action | `WAKE_ELIGIBLE` only when it yields a lawful visible/foreground transition |
| quick settings tile | `WAKE_ELIGIBLE` only when it creates a lawful visible/foreground transition |
| widget | `EXPLICIT_ONLY` unless it clearly transitions into a lawful visible start context |
| external trigger | `EXPLICIT_ONLY` by default; may not auto-create hidden microphone capture |
| boot/restart path | `BLOCKED` for microphone auto-start; may only restore config and readiness receipts |

**WORLD-CLASS UPGRADE — Capability-Tier Model**
- Android implementation must classify device capability into exactly one tier:
- `hardware_hotword_path`
- `software_hotword_path`
- `explicit-only_path`
- `unsupported_path`
- Capability tier affects parity and fallback only at the operational level.
- Capability tier must never become authority, proof, or enforcement truth.
- `hardware_hotword_path` may support stronger lawful parity if platform and device policy allow it.
- `software_hotword_path` must remain conservative under battery, Doze, and background-start restrictions.
- `explicit-only_path` is the default lawful fallback when always-listening legality is not provable.
- `unsupported_path` means wake remains blocked and only non-wake explicit capture may remain lawful.

**WORLD-CLASS UPGRADE — Enterprise Managed-Device and Work-Profile Posture**
- Android B2/B3 must explicitly model:
- managed-device restrictions
- work-profile restrictions
- enterprise/admin microphone disablement posture
- These states feed only normalized non-authoritative posture.
- Managed-device or admin restrictions may push Android into `EXPLICIT_ONLY` or `BLOCKED`, but may never create device-side authority shortcuts or alternate enforcement paths.

**WORLD-CLASS UPGRADE — Process-Death and Restart Restoration Model**
- After process death or restart:
- state that may be restored: persisted user configuration, non-sensitive readiness metadata, last known non-authoritative operational posture class
- state that must be recomputed: permission state, privacy-toggle state, restricted-app state, battery/Doze state, legality posture, capture-session legality, readiness receipt freshness
- state that must never auto-resume: active microphone capture, wake-eligible always-listening state, any cloud trust/proof/enforcement artifact
- If lawful wake readiness cannot be freshly re-proved after restart, Android must fall back to `EXPLICIT_ONLY` or `BLOCKED`.

**WORLD-CLASS UPGRADE — Client-Side Data Minimization and Retention Hygiene**
- Android implementation must keep temporary audio buffers bounded and short-lived.
- Sensitive capture data must not be silently persisted outside the lawful capture path.
- Readiness receipts and attestation artifacts must have explicit retention boundaries and must be wiped on block, failure, or invalidation where continued retention is not required.
- Wipe-on-block and wipe-on-failure expectations are mandatory for device-local transient capture artifacts.

**WORLD-CLASS UPGRADE — OEM Divergence Containment Rule**
- Uncertain OEM or device-specific behavior must fail conservatively.
- OEM-specific behavior may not create authority shortcuts, hidden microphone starts, or proof/enforcement shortcuts.
- If OEM behavior is not stable enough to prove lawful wake eligibility, fallback must remain explicit and visible.

**WORLD-CLASS UPGRADE — User-Visible State and Notification Contract**
- Microphone capture requiring a foreground service must also require user-visible state consistent with Android platform law.
- If lawful user-visible state cannot be maintained, wake must degrade to `EXPLICIT_ONLY` or `BLOCKED`.
- A visible notification or equivalent lawful foreground condition is required whenever Android microphone FGS capture is active.

**WORLD-CLASS UPGRADE — Deterministic Fallback Matrix**

| Android failure / gap class | Allowed runtime outcome |
| --- | --- |
| permission denied / revoked | `explicit-only` |
| privacy-toggle off / enterprise mic disablement | `explicit-only` or `hard block` depending user/admin reversibility |
| background-start illegal | `explicit-only` |
| boot/restart microphone auto-start illegal | `hard block` for auto-start, with operational recovery prompt allowed |
| Doze / restricted-app uncertainty | `retry-later-operationally` or `explicit-only` |
| audio contention / silent-audio detection | `retry-later-operationally` or `explicit-only` |
| unsupported capability tier | `hard block` |
| uncertain OEM/platform behavior | `hard block` or `explicit-only`, never hidden allow |

- The fallback matrix is operational-only and must not replace canonical Phase A governance or law semantics.

**WORLD-CLASS UPGRADE — Android Receipt Freshness and Invalidation Model**
- Android readiness receipts require a bounded freshness window.
- Freshness must be invalidated by:
- Doze posture changes
- restart or reboot
- restriction changes
- privacy-toggle changes
- permission changes
- battery-policy changes
- managed-device policy changes
- FGS legality changes
- On invalidation, readiness must be recomputed before `WAKE_ELIGIBLE` can be reasserted.
- Stale readiness receipts may not be silently reused.
