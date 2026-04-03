# H52 Section 08: First Post-H51 Canonical S08-03 PH1.OS Voice-Live Claimed WakeWord Capability Registry Fail-Closed Build Plan

## Objective

this is the first canonical H52 post-H51 Section 08 next-target publication build plan.

the first canonical H52 Section 08 PH1.OS voice-live claimed WakeWord capability-registry fail-closed proof slice is now live.

H51 remains published as the first canonical post-H50 Section 08 next-target publication.

the exact H51 canonical proof already live is `at_os_22l_voice_live_entrypoint_rejects_ios_explicit_unsupported_wake_word_capability`.

S08-05 remains PROVEN_COMPLETE and is not reopened in this run.

the post-H51 Section 08 next exact active winner is now `S08-03`.

the exact canonical proof implemented by this run is `at_os_22m_voice_live_entrypoint_rejects_ios_explicit_claimed_unsupported_wake_word_capability`.

the exact seam is the first post-H51 canonical PH1.OS voice-live claimed WakeWord capability unsupported-platform fail-closed path on `OsVoiceLiveTurnInput::v1_with_runtime_execution_envelope(...)` through `runtime_execution_envelope.validate()`.

the live carrier path is `OsVoiceLiveTurnInput::v1_with_runtime_execution_envelope(...)` -> `runtime_execution_envelope.validate()`.

no production logic change was required in this run.

## Current Repo Truth

H40 remains live as the first canonical PH1.OS voice-live `runtime_execution_envelope.platform` normalization fail-closed proof slice.

H41 remains live as the first post-H40 canonical PH1.OS voice-live `platform_context.requested_trigger` normalization fail-closed proof slice.

H42 remains live as the first canonical Section 08 runtime-law `platform_trust_warning(...)` restricted-device degrade proof slice.

H43 remains live as the first canonical Section 08 runtime-law `platform_trust_warning(...)` upgrade-required compatibility-warning proof slice.

H44 remains live as the first canonical Section 08 runtime-law `platform_hard_block_required(...)` unsupported-client compatibility hard-block proof slice.

H45 remains live as the first canonical Section 08 runtime-law `platform_hard_block_required(...)` untrusted-device platform-compatibility hard-block proof slice.

H48 remains live as the first canonical post-H47 Section 08 next-target publication and the exact H48 proof remains `at_runtime_law_19_integrity_failed_platform_compatibility_blocks_protected_execution`.

H49 remains live as the first canonical post-H48 Section 08 next-target publication and the exact H49 proof remains `at_runtime_law_20_integrity_unknown_platform_trust_degrades_protected_execution`.

H50 remains live as the first canonical post-H49 Section 08 next-target publication and the exact H50 proof already live remains `at_os_22e_voice_live_entrypoint_rejects_android_voice_without_microphone`.

H51 remains live as the first canonical post-H50 Section 08 next-target publication and the exact H51 proof already live remains `at_os_22l_voice_live_entrypoint_rejects_ios_explicit_unsupported_wake_word_capability`.

current authoritative docs already map unresolved device-capability-registry closure to `ph1os.rs#L608` through the current `S08-03` ledger row.

current source already exposes `PlatformRuntimeContext::default_for_platform_and_trigger(...)`, `supported_capabilities_for_platform(...)`, `DeviceCapability::WakeWord`, field `platform_runtime_context.claimed_capabilities`, and reason `contains capability unsupported by platform`.

current source already proves `PlatformRuntimeContext::default_for_platform_and_trigger(...)` clones the supported capability set into both `claimed_capabilities` and `negotiated_capabilities`.

current source already proves `supported_capabilities_for_platform(AppPlatform::Ios)` excludes `DeviceCapability::WakeWord` while Android and Desktop include it.

current source already proves unsupported claimed-capability membership is checked before the later `claimed capabilities must be present in negotiated capabilities` discipline branch.

current source already proves the seam is live on the PH1.OS voice-live entrypoint because `OsVoiceLiveTurnInput::v1_with_runtime_execution_envelope(...)` validates `runtime_execution_envelope` before later PH1.OS governance checks.

current `runtime_envelope_for_voice_context(...)` helper was not seam-selected by itself before this run because the default iOS explicit platform context excluded `DeviceCapability::WakeWord` from both `claimed_capabilities` and `negotiated_capabilities` and would pass the supported-capability registry gate.

current `android_attested_wake_runtime_envelope(...)` helper was not seam-selected by itself before this run because it already carried Android wake / attested / capture-artifact posture and therefore selected adjacent seams.

current repo truth now preserves the dedicated canonical PH1.OS proof `at_os_22m_voice_live_entrypoint_rejects_ios_explicit_claimed_unsupported_wake_word_capability` for the unsupported claimed-WakeWord-on-iOS explicit registry seam.

## Exact Seam

the exact seam is the first post-H51 canonical PH1.OS voice-live claimed WakeWord capability unsupported-platform fail-closed path on `OsVoiceLiveTurnInput::v1_with_runtime_execution_envelope(...)` through `runtime_execution_envelope.validate()`.

the smallest direct seam is the `runtime_execution_envelope.platform_context.claimed_capabilities` contains `DeviceCapability::WakeWord` branch while `runtime_execution_envelope.platform == AppPlatform::Ios`, `runtime_execution_envelope.platform_context.platform_type == AppPlatform::Ios`, `runtime_execution_envelope.platform_context.requested_trigger == RuntimeEntryTrigger::Explicit`, `runtime_execution_envelope.platform_context.trigger_allowed == true`, `runtime_execution_envelope.platform_context.negotiated_capabilities` remains unchanged and does not contain `DeviceCapability::WakeWord`, and both capability lists still contain `DeviceCapability::Microphone`, which keeps adjacent negotiated-unsupported-capability, claimed-subset discipline, microphone-negotiation, trigger-governance, wake-stage-only capability, integrity-attestation, capture-artifact, and platform-normalization branches aligned and unselected.

the exact canonical proof implemented by this run is `at_os_22m_voice_live_entrypoint_rejects_ios_explicit_claimed_unsupported_wake_word_capability`.

## Implementation Boundary

This run is implementation-bounded to the live PH1.OS claimed-capability registry seam on `OsVoiceLiveTurnInput::v1_with_runtime_execution_envelope(...)` plus the H52 documentation updates that record the implemented slice.

H51 already canonically implemented the negotiated unsupported WakeWord fail-closed proof slice, so that H51 seam is not selected in this run.

the residual later PH1.OS `wake trigger requires negotiated WAKE_WORD capability` branch on `validate_voice_turn_platform_governance(...)` is not the clearest lawful next winner because current contract validation already preempts missing-WakeWord wake-trigger envelopes by enforcing `trigger_allowed` alignment and already exposing the contract-side WakeWord reason on `platform_runtime_context.trigger_allowed`.

the upstream claimed-subset discipline branch is not the clearest lawful next winner because current contract validation rejects unsupported claimed capabilities before it evaluates `claimed capabilities must be present in negotiated capabilities`.

the H52 proof neutralized adjacent branches by keeping `AppPlatform::Ios`, `RuntimeEntryTrigger::Explicit`, `trigger_allowed == true`, `negotiated_capabilities` unchanged without `DeviceCapability::WakeWord` while still containing `DeviceCapability::Microphone`, and `claimed_capabilities` still containing `DeviceCapability::Microphone` while adding unsupported `DeviceCapability::WakeWord` only to `claimed_capabilities`.

S08-04 remains partial and is not selected in this run.

S08-06 and S08-08 remain partial and are not selected in this run because their direct runtime-law branch sets are already exhausted.

S08-01 remains partial with the post-H46 next exact winner `NOT_EXPLICIT` after H47 correction and is not selected in this run.

Section 06 remains parked with the next exact winner `NOT_EXPLICIT`.

H47 correction truth remains unchanged in substance and frozen `at_os_22k...` is not reopened.

## Proof Plan

The canonical implementation basis for this H52 run is the live PH1.OS entrypoint carrier plus the current contract-side capability-registry truth in `PlatformRuntimeContext::validate()`.

The implemented proof begins from the default iOS explicit runtime envelope and adds unsupported `DeviceCapability::WakeWord` only to `runtime_execution_envelope.platform_context.claimed_capabilities` while leaving `runtime_execution_envelope.platform_context.negotiated_capabilities` unchanged, so the fail-closed result selects field `platform_runtime_context.claimed_capabilities` with reason `contains capability unsupported by platform`.

That implementation boundary keeps the adjacent negotiated-unsupported-capability branch already implemented by H51, the later claimed-subset discipline branch, the H50 microphone-negotiation seam, the residual PH1.OS WakeWord branch, and the adjacent trigger-governance, wake-stage-only capability, integrity-attestation, capture-artifact, and platform-normalization branches aligned and unselected.

## Out Of Scope

This H52 implementation does not authorize:

- source edits outside `ph1os.rs`
- contract edits
- engine edits
- runtime-law edits
- build-section wording edits
- any post-H52 target or implementation publication
- re-authoring the historical H51 build-plan file
- broader `S08-01` closure claims
- broader `S08-03` device-capability-registry closure beyond the unsupported claimed WakeWord seam
- broader `S08-04` capability-negotiation closure
- broader `S08-06` compatibility-governance closure
- broader `S08-08` device-trust closure
- any Section 06, Section 09, Section 10, or Section 11 implementation work

no post-H52 next exact winner is published in this run.
