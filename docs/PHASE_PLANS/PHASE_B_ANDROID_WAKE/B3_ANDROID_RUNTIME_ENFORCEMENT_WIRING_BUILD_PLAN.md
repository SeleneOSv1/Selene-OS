PHASE B3 — ANDROID RUNTIME ENFORCEMENT WIRING BUILD PLAN

A) REPO TRUTH CHECK
- `git status --short`: empty
- current branch: `main`
- HEAD commit: `e6044f7d96750f3e74f16890ee0f57c8ef60a303`
- exact files reviewed:
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A1_ARTIFACT_TRUST_ARCHITECTURE_REVIEW.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A1_ARTIFACT_TRUST_ARCHITECTURE_REVIEW.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A2_ARTIFACT_IDENTITY_TRUST_CONTRACT_LAYER_BUILD_PLAN.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A2_ARTIFACT_IDENTITY_TRUST_CONTRACT_LAYER_BUILD_PLAN.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A3_RUNTIME_VERIFICATION_WIRING_BUILD_PLAN.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A3_RUNTIME_VERIFICATION_WIRING_BUILD_PLAN.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A4_PH1J_PROOF_CAPTURE_TRUST_VERIFICATION_OUTCOMES_BUILD_PLAN.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A4_PH1J_PROOF_CAPTURE_TRUST_VERIFICATION_OUTCOMES_BUILD_PLAN.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A5_GOV_LAW_ENFORCEMENT_TRUST_FAILURES_BUILD_PLAN.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A5_GOV_LAW_ENFORCEMENT_TRUST_FAILURES_BUILD_PLAN.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A6_TESTS_DOCS_VERIFICATION_BUILD_PLAN.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A6_TESTS_DOCS_VERIFICATION_BUILD_PLAN.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE/B1_ANDROID_WAKE_MICROPHONE_GAP_REVIEW.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE/B1_ANDROID_WAKE_MICROPHONE_GAP_REVIEW.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE/B2_ANDROID_TRIGGER_CAPTURE_PARITY_CONTRACT_BUILD_PLAN.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE/B2_ANDROID_TRIGGER_CAPTURE_PARITY_CONTRACT_BUILD_PLAN.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/WAKE_BUILD_PLAN.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/WAKE_BUILD_PLAN.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_08.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_08.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_11.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_11.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_OS.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_OS.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_GOV.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_GOV.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_LAW.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_LAW.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_J.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_J.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_adapter/src/lib.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_adapter/src/lib.rs)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1os.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1os.rs)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/device_artifact_sync.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/device_artifact_sync.rs)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1art.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1art.rs)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_governance.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_governance.rs)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_law.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_law.rs)
- whether Android-specific code/docs were found and where:
- Android-specific docs were found in [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE/B1_ANDROID_WAKE_MICROPHONE_GAP_REVIEW.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE/B1_ANDROID_WAKE_MICROPHONE_GAP_REVIEW.md), [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE/B2_ANDROID_TRIGGER_CAPTURE_PARITY_CONTRACT_BUILD_PLAN.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE/B2_ANDROID_TRIGGER_CAPTURE_PARITY_CONTRACT_BUILD_PLAN.md), and [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/WAKE_BUILD_PLAN.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/WAKE_BUILD_PLAN.md).
- Android-specific code was not found. There is no Android client/module, no `AndroidManifest.xml`, and no Kotlin/Java Android source tree in the repo.

B) CURRENT WIRING STATE
- Android client/module presence. CURRENT: absent. TARGET: a future Android module emits only B2-defined operational/platform outputs. GAP: full client/runtime producer is missing.
- adapter carriage readiness. CURRENT: adapter already carries non-authoritative platform posture into Phase A runtime surfaces; it has no Android-specific contract carriage yet. TARGET: carry Android readiness receipts, attestation refs, posture refs, capture-session refs, and context fingerprint only. GAP: Android-specific adapter mapping is missing.
- ingress carriage readiness. CURRENT: ingress already has canonical runtime envelope and pre-verification carriage patterns. TARGET: carry Android refs and posture inputs only, with no Android-local legality becoming truth. GAP: Android-specific ingress fields/mapping are missing.
- PH1.OS normalization readiness. CURRENT: PH1.OS already normalizes upstream posture and orchestration state only. TARGET: normalize Android operational/platform posture only. GAP: Android-specific normalization path is missing.
- Section 04 input-boundary readiness. CURRENT: Phase A already provides the authoritative boundary and pre-verification input surfaces. TARGET: Section 04 consumes only Android non-authoritative inputs through those existing surfaces. GAP: Android-specific feed into that boundary is missing.
- Android receipt/attestation carriage. CURRENT: no Android receipts or Android attestation carriers exist in code. TARGET: non-authoritative carriage of B2 receipt/attestation refs. GAP: missing.
- Android legality/posture carriage. CURRENT: no Android legality or posture contract is wired. TARGET: carry B2 legality/posture outputs as upstream operational inputs only. GAP: missing.
- Android capture-session carriage. CURRENT: no Android capture-session refs or lifecycle carriage exists. TARGET: capture-session refs flow through ingress/runtime transport only. GAP: missing.
- Phase A non-regression safety. CURRENT: safe because no Android path exists yet. TARGET: remain safe while adding Android-local producers and canonical carriage only. GAP: future wiring risk if B3 is sloppy.
- performance/battery risk exposure. CURRENT: high, because Android startup/retry/receipt churn behavior is undefined in code. TARGET: bounded lawful operational wiring under B2 contracts. GAP: missing implementation discipline.
- platform-law risk exposure. CURRENT: high, because Android legality is specified only in docs, not enforced in code. TARGET: B3 wiring must preserve B2’s Android-law model without turning it into authority truth. GAP: missing wiring.

C) CANONICAL B3 WIRING DESIGN
1. what Android client emits
- The future Android module emits only B2-defined non-authoritative outputs:
- `AndroidPlatformLawContext`
- `AndroidWakeEligibilityOutcome`
- `AndroidCaptureEligibilityOutcome`
- readiness receipt refs
- attestation refs
- operational posture refs
- capture-session refs
- route/contention/visibility/privacy/restriction state
- process-restoration state
- context fingerprint
2. what adapter may normalize only
- Adapter may normalize Android outputs into stable, non-authoritative upstream posture inputs and refs.
- Adapter may carry:
- readiness receipt refs
- attestation refs
- operational posture refs
- capture-session refs
- visibility/posture summaries
- context fingerprint
- Adapter may not:
- decide trust
- decide proof
- decide enforcement
- persist or mint canonical Phase A trust/proof objects
3. what ingress may carry only
- Ingress may carry Android non-authoritative refs and posture into the canonical runtime envelope and A3 pre-verification input surfaces.
- Ingress may not:
- reinterpret Android legality as trust truth
- create Android-specific proof records
- create Android-specific enforcement state
4. what PH1.OS may normalize only
- PH1.OS may normalize:
- Android receipt freshness posture
- Android attestation availability posture
- Android operational legality posture for orchestration purposes
- wake/capture availability posture
- PH1.OS may not:
- own Android legality semantics
- own trust/proof/enforcement truth
- decide artifact authenticity
5. what enters Section 04
- Only non-authoritative Android refs and posture enter Section 04.
- Concretely: Android readiness receipt refs, attestation refs, operational posture refs, capture-session refs, and pre-verification posture fields enter through existing Phase A runtime carriers such as `ArtifactTrustPreVerificationInput` and the canonical runtime envelope.
- Section 04 remains the first-time authoritative verifier and must treat Android inputs as upstream evidence only.
6. what must remain Android-local only
- permission workflow
- privacy-toggle handling
- foreground service lifecycle
- wake-lock lifecycle
- capture start/stop mechanics
- route transitions
- contention handling
- restart restoration mechanics
- user-visible notification behavior
- explicit-only fallback UX
7. what must explicitly NOT be wired yet
- no PH1.J proof behavior changes
- no new proof transport
- no PH1.GOV behavior changes
- no PH1.LAW behavior changes
- no Android-side trust verification
- no Android-side enforcement
- no B4/B5 work
8. how fallback/explicit-only/block states flow
- Android computes operational legality/fallback locally under B2.
- Android emits those as non-authoritative posture and receipts only.
- Adapter/ingress/PH1.OS carry them as operational inputs only.
- Section 04 may later consume them as context, but they do not become trust truth.
- Explicit-only/block states remain operational posture until canonical Phase A transport carries them upstream.
9. how restart/recompute/receipt freshness flows
- Android restart invalidates stale legality assumptions where B2 requires it.
- Android must recompute readiness and emit fresh readiness receipts when invalidation triggers fire.
- Adapter/ingress/PH1.OS carry receipt freshness posture and refs only.
- No stale local legality may be silently reused as authority truth.

D) WIRING OWNERSHIP MODEL
- Android client/module
  - owns Android-local platform-law evaluation, readiness evaluation, capture-session lifecycle, receipt emission, and user-visible foreground behavior
- adapter
  - owns carriage and normalization of Android non-authoritative posture and refs only
- ingress
  - owns carriage of Android refs/posture into the canonical runtime envelope only
- PH1.OS
  - owns normalization of Android device/platform/receipt/attestation posture for orchestration only
- Section 04 consumer boundary
  - owns first-time authoritative consumption of canonical runtime inputs only
- downstream Phase A readers
  - PH1.J, PH1.GOV, and PH1.LAW remain downstream canonical readers of Phase A trust/proof/enforcement outputs only

E) LEGALITY / PERFORMANCE / BATTERY MODEL
- wake legality under Android 12/13/14/15+
- B3 must wire B2’s legality matrix exactly. It must not weaken Android background-start, boot, target-SDK, visibility, or FGS restrictions.
- foreground-service legality and visible-state coupling
- B3 must require Android-local production of visible-state and FGS-start posture before carriage. If visibility or lawful FGS state is absent, Android must degrade locally to `EXPLICIT_ONLY` or `BLOCKED` per B2.
- microphone readiness transitions
- B3 must carry readiness transitions as posture and receipts only. Freshness and invalidation must remain explicit.
- privacy-toggle handling
- Privacy-toggle off and silent-audio posture must be carried as distinct operational conditions. They must not be collapsed into generic capture failure.
- silent-audio detection handling
- Silent-audio detection must remain Android-local detection with non-authoritative receipt/posture output.
- retry suppression in illegal contexts
- B3 must not introduce hidden retry loops. If B2 says retry is suppressed or forbidden, B3 must preserve that state and not reanimate it in adapter, ingress, or PH1.OS.
- Doze/restricted-app transitions
- B3 must carry Doze/restricted-app transitions as operational posture changes and freshness invalidation triggers only.
- wake-lock boundedness
- B3 must keep wake lock Android-local and bounded. Wake-lock state may inform operational posture only; it may never imply trust or legality truth upstream.
- route-change and contention handling
- Route changes and contention stay Android-local operational behavior with receipts/posture emitted upstream only.
- recomputation budget / receipt freshness discipline
- B3 must preserve B2 recomputation budget posture and avoid battery-noisy receipt churn. Freshness is required, but churn is forbidden.

F) REQUIRED FILE CHANGE MAP
- docs
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE/B2_ANDROID_TRIGGER_CAPTURE_PARITY_CONTRACT_BUILD_PLAN.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE/B2_ANDROID_TRIGGER_CAPTURE_PARITY_CONTRACT_BUILD_PLAN.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/WAKE_BUILD_PLAN.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/WAKE_BUILD_PLAN.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_OS.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_OS.md)
- kernel contracts
- [/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1art.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1art.rs)
- adapter
- [/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_adapter/src/lib.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_adapter/src/lib.rs)
- os/runtime
- [/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1os.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1os.rs)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/device_artifact_sync.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/device_artifact_sync.rs)
- android client/module if later created
- [/Users/xiamo/Documents/A-Selene/Selene-OS/android/selene_android_wake/AndroidManifest.xml](/Users/xiamo/Documents/A-Selene/Selene-OS/android/selene_android_wake/AndroidManifest.xml)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/android/selene_android_wake/src/main/java/com/selene/wake/WakeForegroundService.kt](/Users/xiamo/Documents/A-Selene/Selene-OS/android/selene_android_wake/src/main/java/com/selene/wake/WakeForegroundService.kt)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/android/selene_android_wake/src/main/java/com/selene/wake/WakeEligibilityCoordinator.kt](/Users/xiamo/Documents/A-Selene/Selene-OS/android/selene_android_wake/src/main/java/com/selene/wake/WakeEligibilityCoordinator.kt)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/android/selene_android_wake/src/main/java/com/selene/wake/MicrophoneReadinessCoordinator.kt](/Users/xiamo/Documents/A-Selene/Selene-OS/android/selene_android_wake/src/main/java/com/selene/wake/MicrophoneReadinessCoordinator.kt)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/android/selene_android_wake/src/main/java/com/selene/wake/CaptureSessionCoordinator.kt](/Users/xiamo/Documents/A-Selene/Selene-OS/android/selene_android_wake/src/main/java/com/selene/wake/ CaptureSessionCoordinator.kt)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/android/selene_android_wake/src/main/java/com/selene/wake/AndroidWakeReceiptEmitter.kt](/Users/xiamo/Documents/A-Selene/Selene-OS/android/selene_android_wake/src/main/java/com/selene/wake/AndroidWakeReceiptEmitter.kt)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/android/selene_android_wake/src/main/java/com/selene/wake/AndroidOperationalPostureEmitter.kt](/Users/xiamo/Documents/A-Selene/Selene-OS/android/selene_android_wake/src/main/java/com/selene/wake/AndroidOperationalPostureEmitter.kt)

G) NON-REGRESSION / FORBIDDEN PATH MAP
- no Android trust authority
- no Android proof authority
- no Android enforcement authority
- no raw adapter hints as trust truth
- no raw Android legality outcomes as trust truth
- no PH1.OS legality ownership beyond normalization
- no device persistence of canonical Phase A trust/proof objects
- no mapping of Android outputs directly into `artifact_trust_state`
- no mapping of Android outputs directly into `ArtifactTrustDecisionRecord`
- no mapping of Android outputs directly into `ArtifactTrustProofEntry`
- no device-side persistence or reuse of `proof_entry_ref`
- no device-side persistence or reuse of `proof_record_ref`
- no second trust transport
- no second proof transport
- no second enforcement path

H) STAGING PLAN
1. Add the Android-local module surfaces that produce B2 operational/platform outputs only.
2. Wire adapter carriage for Android refs and posture only.
3. Wire ingress carriage into existing canonical runtime envelope/pre-verification inputs only.
4. Wire PH1.OS normalization for Android posture only, with no legality ownership expansion.
5. Wire Section 04 input-boundary consumption of Android non-authoritative refs/posture only.
6. Add compile-through adjustments for downstream readers only if needed.
7. Stop before any B4 proof/governance/law expansion.
- This order is additive-first, drift-resistant, and keeps proof/enforcement untouched.

I) RISKS / DRIFT WARNINGS
- B3 could accidentally create a second authority path if Android legality outcomes are treated as trust truth.
- B3 could accidentally create a second proof path if Android receipts are mistaken for proof artifacts.
- B3 could let PH1.OS own Android legality if normalization is allowed to drift into decision ownership.
- B3 could break Phase A transport assumptions if Android invents a side channel instead of using canonical pre-verification and runtime envelope carriage.
- B3 could violate Android platform law if retry, boot, visibility, or FGS legality are softened in implementation.
- B3 could create battery/noise regressions if receipt recomputation and retry suppression are not respected.
- B3 could regress into stale legality reuse if restart/recompute invalidation is not preserved exactly from B2.

J) FINAL APPROVAL PACKAGE
- recommended B3 scope
- wire Android-local operational/platform outputs into canonical adapter, ingress, PH1.OS, and Section 04 input boundaries only, while preserving B2 fallback and legality semantics
- what must be approved before coding
- exact Android-local outputs to be emitted
- exact adapter carriage fields
- exact ingress carriage fields
- exact PH1.OS normalization fields
- exact Section 04 input-boundary mapping
- exact restart/recompute/freshness flow
- exact local-only vs upstream-carried ownership boundaries
- what must NOT be implemented yet
- no new trust verification logic
- no new proof transport
- no new GOV/LAW behavior
- no PH1.J behavior changes
- no B4/B5 work
- whether B3 is ready for implementation planning after approval: YES

Completed in this chat.
Test command.
`git status --short`
Expected output.
`empty`
Next step.
Review and approve this B3 wiring plan, then begin additive B3 implementation against the frozen Phase A baseline and approved B1/B2 contracts.
Next file to open.
`/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE/B2_ANDROID_TRIGGER_CAPTURE_PARITY_CONTRACT_BUILD_PLAN.md`
