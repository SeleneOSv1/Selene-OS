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

**WORLD-CLASS UPGRADE — Canonical Android Runtime Topology Map**
- B3 later wires one Android-local runtime topology only:
- `WakeForegroundService`
  - owns lawful foreground-service lifecycle only
  - must never own trust, proof, governance, or runtime-law meaning
- `WakeEligibilityCoordinator`
  - owns Android wake legality evaluation from B2 contracts only
  - must never own Phase A authority or proof semantics
- `MicrophoneReadinessCoordinator`
  - owns permission, privacy-toggle, readiness, and receipt freshness evaluation only
  - must never mint trust or proof truth
- `CaptureSessionCoordinator`
  - owns capture start, stop, downgrade, and fallback transitions only
  - must never reinterpret legality into authority
- `AudioRouteMonitor`
  - owns route observation and instability posture only
  - must never own capture legality
- `AudioContentionMonitor`
  - owns contention and silent-audio observation only
  - must never own trust or enforcement meaning
- `AndroidWakeReceiptEmitter`
  - emits readiness, restoration, and legality-related operational receipts only
  - must never emit proof or governance artifacts
- `AndroidOperationalPostureEmitter`
  - emits non-authoritative Android operational posture only
  - must never emit canonical Phase A trust/proof/enforcement objects
- `AndroidIngressBridge`
  - packages approved Android refs and posture for adapter carriage only
  - must never infer trust/proof/enforcement truth
- `AndroidRestartRestorationCoordinator`
  - owns restart invalidation, recomputation, and restoration-safe fallback only
  - must never auto-resume hidden microphone capture

C) CANONICAL B3 WIRING DESIGN
**WORLD-CLASS UPGRADE — Canonical Android Receipt/Event Pipeline**
- B3 must wire one deterministic Android-local operational receipt and event pipeline:
1. compute `AndroidPlatformLawContext`
2. emit readiness evaluation receipt when readiness is computed or recomputed
3. emit wake/capture legality posture outcome
4. emit capture operational receipt when capture starts, blocks, degrades, stops, silences, or contends
5. emit route/contention events when audio route or contention changes affect behavior
6. emit invalidation event when freshness or legality assumptions are invalidated
7. emit restoration receipt when boot or process restoration re-evaluates the Android runtime posture
- Ordering is deterministic by local event sequence and context fingerprint.
- All emitted Android receipts and events remain operational evidence only.
- Repeated identical receipts within the same `context_fingerprint` and suppression window must collapse into a single upstream emission.

**WORLD-CLASS UPGRADE — Stable Receipt/Event Identity Law**
- Every Android-local operational receipt or event must bind:
- `receipt_id` or `event_id`
- `event_sequence`
- `context_fingerprint`
- `request_id` where available
- `session_id` where available
- `turn_id` where available
- Android receipts and events are operational evidence only.
- They must be stable and traceable.
- They must never become trust truth, proof truth, or enforcement truth.

**WORLD-CLASS UPGRADE — Explicit Per-Contract Ownership Hints**
- `AndroidPlatformLawContext`
  - Android-local producer/coordinator owns creation
  - adapter carries only
  - PH1.OS consumes posture only
  - Section 04 consumes only non-authoritative inputs
- `AndroidWakeEligibilityOutcome`
  - Android-local producer/coordinator owns creation
  - adapter carries only
  - PH1.OS consumes posture only
  - Section 04 consumes only non-authoritative inputs
- `AndroidCaptureEligibilityOutcome`
  - Android-local producer/coordinator owns creation
  - adapter carries only
  - PH1.OS consumes posture only
  - Section 04 consumes only non-authoritative inputs
- readiness receipt refs
  - Android-local producer/coordinator owns emission
  - adapter carries only
  - PH1.OS consumes freshness posture only
  - Section 04 consumes only non-authoritative refs
- capture operational receipt refs
  - Android-local producer/coordinator owns emission
  - adapter carries only
  - PH1.OS consumes operational posture only
  - Section 04 consumes only non-authoritative refs
- capture-session refs
  - Android-local producer/coordinator owns emission
  - adapter carries only
  - PH1.OS consumes posture only
  - Section 04 consumes only non-authoritative inputs

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
**WORLD-CLASS UPGRADE — Service-Start Legality Wiring Table**
| Trigger source | Service start attempted? | Explicit-only allowed? | Blocked? | Required receipt or posture |
| --- | --- | --- | --- | --- |
| `VISIBLE_UI_ACTION` | YES, when B2 legality is lawful | YES, if capture legality fails but explicit path remains lawful | YES, if B2 legality blocks all capture | readiness receipt plus service-start posture |
| `VISIBLE_NOTIFICATION_ACTION` | YES, only when Android-lawful visible path exists | YES | YES | notification-action legality receipt plus fallback posture |
| `QUICK_SETTINGS_TILE` | YES, only as visible user-triggered path | YES | YES | tile-trigger legality receipt plus capture posture |
| `WIDGET` | NO hidden start; only if it lawfully transitions to visible explicit path | YES | YES | widget legality receipt plus explicit-only or blocked posture |
| `WAKE_PHRASE` | YES only where B2 legality and capability tier allow it | YES, if hotword path is unlawful but explicit path remains lawful | YES | wake legality receipt plus capture-start or fallback posture |
| `BOOT_RESTORE` | NO microphone service auto-start | YES, for later explicit recovery only | YES for direct capture start | restoration receipt and recomputation posture |
| `BACKGROUND_DORMANT_PATH` | NO | YES only if B2 allows explicit-only surfaced path later | YES | blocked or explicit-only operational posture |

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
**WORLD-CLASS UPGRADE — Android-to-Phase-A Transport Seam Table**
| Android-local output | Adapter-carried field or ref | Ingress-carried field or ref | PH1.OS normalized field or ref | Section 04 input boundary |
| --- | --- | --- | --- | --- |
| readiness receipt ref | upstream readiness receipt ref | carried upstream receipt ref | receipt freshness posture | `ArtifactTrustPreVerificationInput.upstream_receipt_refs` only |
| attestation ref | upstream attestation ref | carried upstream attestation ref | attestation availability posture | `ArtifactTrustPreVerificationInput.upstream_attestation_ref` only |
| operational posture ref | upstream posture ref | carried upstream posture ref | operational legality posture | pre-verification posture only |
| capture-session ref | upstream capture-session ref | carried capture-session ref | capture availability posture | runtime envelope / pre-verification carrier only |
| context fingerprint | normalized context identity | carried context identity | normalized context linkage | pre-verification context only |
- None of these may map directly into:
- `artifact_trust_state`
- `ArtifactTrustDecisionRecord`
- `ArtifactTrustProofEntry`
- `proof_entry_ref`
- `proof_record_ref`
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
**WORLD-CLASS UPGRADE — No-Illegal-Fallback Law**
- When Android legality is `BLOCKED`, B3 may not secretly continue via hidden local fallback.
- When B2 says `EXPLICIT_ONLY`, B3 must not attempt hidden microphone startup.
- Illegal contexts may not be retried until they work.
- Illegal contexts must emit blocked or explicit-only posture honestly and stop there.

**WORLD-CLASS UPGRADE — Deterministic Fallback Matrix**
| Failure or gap class | Recoverability class | User action requirement | Allowed fallback outcome |
| --- | --- | --- | --- |
| `MICROPHONE_PERMISSION_DENIED` | `RECOVERABLE_BY_USER_ACTION` | `GRANT_RECORD_AUDIO` | `EXPLICIT_ONLY` |
| `MICROPHONE_PRIVACY_TOGGLE_OFF` | `RECOVERABLE_BY_USER_ACTION` | `TURN_MIC_TOGGLE_ON` | `EXPLICIT_ONLY` |
| `FGS_START_ILLEGAL` | `RECOVERABLE_BY_FOREGROUND_TRANSITION` | `OPEN_APP_IN_FOREGROUND` or `APPROVE_FGS_PATH` | `EXPLICIT_ONLY` |
| `BACKGROUND_START_ILLEGAL` | `RECOVERABLE_BY_FOREGROUND_TRANSITION` | `OPEN_APP_IN_FOREGROUND` | `EXPLICIT_ONLY` |
| `BOOT_MICROPHONE_START_ILLEGAL` | `HARD_BLOCKED_UNDER_CURRENT_PLATFORM_LAW` | `NO_USER_ACTION_REQUIRED` until later lawful explicit entry | `BLOCKED` |
| `RESTRICTED_APP_BLOCKED` | `RECOVERABLE_BY_POLICY_CHANGE` | `DISABLE_RESTRICTED_APP_POSTURE` | `RETRY_LATER_OPERATIONALLY` or `EXPLICIT_ONLY` |
| `DOZE_POLICY_BLOCKED` | `RECOVERABLE_BY_POLICY_CHANGE` | `NO_USER_ACTION_REQUIRED` | `RETRY_LATER_OPERATIONALLY` |
| `BATTERY_POLICY_BLOCKED` | `RECOVERABLE_BY_POLICY_CHANGE` | `NO_USER_ACTION_REQUIRED` | `RETRY_LATER_OPERATIONALLY` |
| `CAPTURE_CONTENTION_BLOCKED` | `RECOVERABLE_BY_POLICY_CHANGE` | `NO_USER_ACTION_REQUIRED` | `DEGRADE` or `RETRY_LATER_OPERATIONALLY` |
| `SILENT_AUDIO_DELIVERED` | `RECOVERABLE_BY_POLICY_CHANGE` | `NO_USER_ACTION_REQUIRED` | `DEGRADE` |
| `OEM_DIVERGENCE_UNCERTAIN` | `HARD_BLOCKED_UNDER_CURRENT_PLATFORM_LAW` | `ACKNOWLEDGE_EXPLICIT_ONLY_MODE` | `EXPLICIT_ONLY` or `BLOCKED` |
9. how restart/recompute/receipt freshness flows
- Android restart invalidates stale legality assumptions where B2 requires it.
- Android must recompute readiness and emit fresh readiness receipts when invalidation triggers fire.
- Adapter/ingress/PH1.OS carry receipt freshness posture and refs only.
- No stale local legality may be silently reused as authority truth.
**WORLD-CLASS UPGRADE — Process-Death and Restart Restoration Wiring Law**
- `AndroidRestartRestorationCoordinator` later emits:
- restoration receipt
- invalidation event for stale readiness assumptions
- recomputation request for readiness and legality posture
- conservative fallback posture defaulting to `EXPLICIT_ONLY`
- What is recomputed:
- readiness state
- legality state
- route/contention posture when required by current context
- What is invalidated:
- stale readiness receipts
- stale legality assumptions tied to the old context fingerprint
- What must never auto-resume:
- hidden microphone capture
- hidden passive wake when fresh readiness is absent
- Fresh readiness receipt is mandatory before any restored path may return from `EXPLICIT_ONLY` to a lawful wake or capture posture.

**WORLD-CLASS UPGRADE — Retry, Suppression, and Cooldown Wiring Law**
- B3 must wire retry according to B2 contract surfaces only.
- `RETRY_ALLOWED` may schedule a later operational attempt only within lawful contexts.
- `RETRY_SUPPRESSED` must stop repeated attempts and surface suppression posture.
- `RETRY_COOLDOWN_WINDOW_ACTIVE` must preserve cooldown and suppress noisy repeats.
- `RETRY_FORBIDDEN_UNDER_PLATFORM_LAW` must block any retry attempt.
- Hidden background retry is forbidden.
- Boot retry that violates Android legality is forbidden.

**WORLD-CLASS UPGRADE — Route, Contention, and Silence Handling Wiring Law**
- Android route changes, Bluetooth/headset transitions, contention, silent-audio delivery, and route-instability debounce stay Android-local operational behavior only.
- B3 later wires:
- route/contention monitor output -> operational event emission
- event emission -> adapter carriage as posture/receipt refs only
- capture-session downgrade or block -> local fallback state plus upstream operational receipt
- Route instability must be debounced or throttled before repeated upstream emission.
- Capture-session downgrade or block under churn must remain deterministic and visible.

**WORLD-CLASS UPGRADE — Duplicate Suppression and Idempotency Posture**
- B3 must suppress duplicate Android-local receipt spam under churn.
- Required deterministic collapse rules:
- duplicate readiness receipt emission collapse
- repeated route-change collapse
- repeated restoration-event collapse
- no duplicate upstream spam for equivalent context and event sequence
- This is operational idempotency only and not a second trust/proof path.

**WORLD-CLASS UPGRADE — Multi-Trigger Arbitration Rule**
- B3 must define one deterministic arbitration order when triggers arrive simultaneously or near-simultaneously.
- Interactive visible triggers outrank passive or restoration triggers:
1. `VISIBLE_UI_ACTION`
2. `NOTIFICATION_ACTION`
3. `QUICK_SETTINGS_TILE`
4. `WIDGET`
5. `WAKE_PHRASE`
6. `BOOT_RESTORE`
- The winning trigger owns the active operational attempt.
- Loser triggers must be downgraded or suppressed deterministically.
- Receipts must record both the winning trigger and suppressed competing triggers where relevant.

**WORLD-CLASS UPGRADE — Capability-Tier-to-Runtime-Behavior Map**
| Capability tier | Allowed runtime path | Expected fallback | Expected receipt behavior |
| --- | --- | --- | --- |
| `HARDWARE_HOTWORD_CAPABLE` | lawful passive wake path only when B2 legality permits | `EXPLICIT_ONLY` or `BLOCKED` when legality fails | readiness receipt plus wake/capture legality receipt |
| `SOFTWARE_HOTWORD_CAPABLE` | lawful software wake only when B2 legality and battery posture permit | `EXPLICIT_ONLY` first, `BLOCKED` when prohibited | readiness receipt plus battery/legality posture receipt |
| `EXPLICIT_ONLY_CAPABLE` | visible explicit capture path only | remain `EXPLICIT_ONLY` when lawful, otherwise `BLOCKED` | explicit-only operational receipt |
| `UNSUPPORTED` | no passive or explicit Android wake path beyond surfaced blocked posture | `BLOCKED` | blocked/unsupported operational receipt |

D) WIRING OWNERSHIP MODEL
**WORLD-CLASS UPGRADE — Consumer Visibility and Readability Boundary**
- Adapter may see and carry Android posture and refs only.
- Ingress may see and carry Android posture and refs only.
- PH1.OS may read and normalize Android posture only.
- Section 04 may consume only Android non-authoritative inputs through canonical Phase A carriers.
- PH1.J later may see only canonical Phase A trust/proof outputs and must not treat raw Android receipts as proof truth.
- PH1.GOV and PH1.LAW later may see only canonical Phase A enforcement inputs and must not consume raw Android legality as enforcement truth.
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
**WORLD-CLASS UPGRADE — Android Foreground and Notification Coupling Law**
- B3 must wire foreground-service legality with visible-state coupling exactly.
- When foreground notification must already exist:
- before continuing any lawful microphone capture path that requires visible foreground execution
- When capture cannot proceed without visible user state:
- whenever B2 legality says explicit visibility or lawful FGS presence is required
- What happens on notification dismissal:
- the Android-local runtime must transition deterministically to `EXPLICIT_ONLY` or `BLOCKED` per B2 contract, emit a receipt, and stop hidden continuation
- What happens on foreground loss:
- B3 must transition to `EXPLICIT_ONLY` or `BLOCKED`, emit a foreground-loss receipt or posture event, and require fresh readiness where B2 says so
- Consequence table:
| Event | Resulting posture | Required operational receipt |
| --- | --- | --- |
| notification dismissed | `EXPLICIT_ONLY` or `BLOCKED` depending current legality context | foreground-notification-loss receipt |
| app loses visible foreground | `EXPLICIT_ONLY` or `BLOCKED` depending current legality context | foreground-loss downgrade receipt |
| FGS no longer lawful | `BLOCKED` | fgs-illegal transition receipt |
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
**Implementation acceptance bullets**
- Receipt recomputation must remain within bounded churn classes and may not continuously reevaluate under stable blocked contexts.
- Route-instability handling must debounce Bluetooth/headset churn and suppress repeated capture start/stop loops.
- Explicit-only fallback must always surface a visible, user-legible path and emit an operational receipt on entry.
**WORLD-CLASS UPGRADE — Bounded-Resource and Battery-Safe Wiring Law**
- B3 must preserve:
- wake-lock boundedness
- receipt recomputation budget
- route-instability churn control
- restart churn suppression
- receipt freshness balanced against battery cost
- capture-session startup and teardown noise control
- Android-local wiring must fail conservative rather than spend battery aggressively under uncertain legality or unstable runtime posture.

**WORLD-CLASS UPGRADE — Managed-Device and Work-Profile Enforcement Consumption Note**
- Managed-device, work-profile, and enterprise-admin restrictions may affect Android operational posture only.
- Android may emit that posture only.
- Adapter, ingress, and PH1.OS may carry or normalize that posture only.
- Section 04 may consume it only as non-authoritative context.
- Enterprise policy posture never becomes trust, proof, or enforcement truth by itself.

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
**WORLD-CLASS UPGRADE — Forbidden Android-Local State Table**
| Object | May store? | May emit? | May carry as ref? |
| --- | --- | --- | --- |
| `artifact_trust_state` | NO | NO | NO |
| `ArtifactTrustDecisionRecord` | NO | NO | NO |
| `ArtifactTrustProofEntry` | NO | NO | NO |
| `proof_entry_ref` | NO | NO | NO |
| `proof_record_ref` | NO | NO | NO |
| governance outputs | NO | NO | NO |
| runtime-law outputs | NO | NO | NO |
| canonical Phase A trust object | NO | NO | NO |
| canonical Phase A proof object | NO | NO | NO |
| canonical Phase A enforcement object | NO | NO | NO |

**WORLD-CLASS UPGRADE — Observability and Correlation Contract**
- Minimum correlation fields for B3 operational events:
- `request_id`
- `session_id`
- `turn_id` where available
- `context_fingerprint`
- `readiness_receipt_ref`
- `capture_operational_receipt_ref`
- `trigger_source`
- `visibility_state`
- `fallback_outcome`
- This remains operational telemetry only and does not create proof or enforcement truth.
- Exemplar appendix:
- `readiness_recompute_event`: emitted when readiness is recomputed from a fresh `context_fingerprint` after invalidation.
- `route_instability_suppressed_event`: emitted when repeated Bluetooth/headset churn is collapsed under debounce or throttle posture.
- `explicit_only_entered_event`: emitted when Android falls back to `EXPLICIT_ONLY` and surfaces the user-visible recovery path.
- `foreground_loss_downgrade_event`: emitted when visible foreground conditions are lost and the runtime downgrades to `EXPLICIT_ONLY` or `BLOCKED`.

**WORLD-CLASS UPGRADE — Explicit No-Repair Law**
- Adapter may not repair missing receipts.
- PH1.OS may not repair missing legality state.
- Ingress may not infer trust, proof, or enforcement truth from Android posture.
- Missing receipt or stale receipt must be surfaced, not hidden.
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
**WORLD-CLASS UPGRADE — B3 Implementation Safety Gates**
- Gate 1: Android-local producer surfaces land.
- Gate 2: adapter carriage lands.
- Gate 3: ingress carriage lands.
- Gate 4: PH1.OS normalization lands.
- Gate 5: Section 04 boundary consumption lands.
- Gate 6: compile-through and read-only downstream compatibility lands.
- Gate 7: receipt-churn, retry-suppression, and route-instability acceptance verified before B3 completion.
- stop before B4.
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
- `WAKE_BUILD_PLAN.md` is secondary/reference-only for Android runtime wiring; B1/B2/B3 are canonical where wording differs.

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
