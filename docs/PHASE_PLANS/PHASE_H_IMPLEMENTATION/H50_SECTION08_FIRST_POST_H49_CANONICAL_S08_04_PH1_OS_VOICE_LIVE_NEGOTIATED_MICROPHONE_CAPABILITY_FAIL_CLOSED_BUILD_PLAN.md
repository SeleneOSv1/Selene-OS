# H50 Section 08: First Post-H49 Canonical S08-04 PH1.OS Voice-Live Negotiated Microphone Capability Fail-Closed Build Plan

## Objective

this is the first canonical H50 post-H49 Section 08 next-target publication build plan.

H49 remains published as the first canonical post-H48 Section 08 next-target publication.

the exact H49 canonical proof now live is `at_runtime_law_20_integrity_unknown_platform_trust_degrades_protected_execution`.

S08-05 remains PROVEN_COMPLETE and is not reopened in this run.

the post-H49 Section 08 next exact active winner is `S08-04`.

the exact seam is the first post-H49 canonical PH1.OS voice-live negotiated microphone capability fail-closed path on `validate_voice_turn_platform_governance(...)`.

the live carrier path is `OsVoiceLiveTurnInput::v1_with_runtime_execution_envelope(...)` -> `validate_voice_turn_platform_governance(...)`.

no code is changed in this run; this run only publishes the next active target around an already-live proof slice.

## Current Repo Truth

H40 remains live as the first canonical PH1.OS voice-live `runtime_execution_envelope.platform` normalization fail-closed proof slice.

H41 remains live as the first post-H40 canonical PH1.OS voice-live `platform_context.requested_trigger` normalization fail-closed proof slice.

H42 remains live as the first canonical Section 08 runtime-law `platform_trust_warning(...)` restricted-device degrade proof slice.

H43 remains live as the first canonical Section 08 runtime-law `platform_trust_warning(...)` upgrade-required compatibility-warning proof slice.

H44 remains live as the first canonical Section 08 runtime-law `platform_hard_block_required(...)` unsupported-client compatibility hard-block proof slice.

H45 remains live as the first canonical Section 08 runtime-law `platform_hard_block_required(...)` untrusted-device platform-compatibility hard-block proof slice.

H48 remains live as the first canonical post-H47 Section 08 next-target publication and as the first canonical integrity-failed compatibility hard-block proof slice.

H49 remains live as the first canonical post-H48 Section 08 next-target publication.

current authoritative docs already map unresolved capability-negotiation closure to `ph1os.rs#L608` through the current `S08-04` ledger row.

current source already exposes `DeviceCapability::Microphone` and reason `voice turns require negotiated MICROPHONE capability`.

current repo truth already preserves the exact proof `at_os_22e_voice_live_entrypoint_rejects_android_voice_without_microphone`.

current source already proves the seam is contract-reachable because the exact proof validates the microphone-removed envelope before invoking `OsVoiceLiveTurnInput::v1_with_runtime_execution_envelope(...)`.

current contract validation only enforces supported-capability membership plus claimed-capability-subset discipline and does not require negotiated `DeviceCapability::Microphone` by itself on the selected Android wake envelope.

current repo truth already preserves adjacent `at_os_22c`, `at_os_22d`, `at_os_22f`, `at_os_22g`, `at_os_22h`, `at_os_22i`, and `at_os_22j` voice-entrypoint proofs.

## Exact Seam

the exact seam is the first post-H49 canonical PH1.OS voice-live negotiated microphone capability fail-closed path on `validate_voice_turn_platform_governance(...)`.

the smallest direct seam is the `!runtime_execution_envelope.platform_context.negotiated_capabilities.contains(&DeviceCapability::Microphone)` branch while `runtime_execution_envelope.platform == AppPlatform::Android`, `runtime_execution_envelope.platform_context.platform_type == AppPlatform::Android`, `runtime_execution_envelope.platform_context.requested_trigger == RuntimeEntryTrigger::WakeWord`, `runtime_execution_envelope.platform_context.trigger_allowed == true`, `runtime_execution_envelope.platform_context.negotiated_capabilities` still contains `DeviceCapability::WakeWord`, and `runtime_execution_envelope.platform_context.integrity_status == ClientIntegrityStatus::Attested` keep adjacent platform-normalization, trigger-normalization, trigger-governance, wake-word-capability, integrity-attestation, and capture-artifact branches aligned and unselected.

the live carrier path is `OsVoiceLiveTurnInput::v1_with_runtime_execution_envelope(...)` -> `validate_voice_turn_platform_governance(...)`.

## Implementation Boundary

This run is docs-only and remains bounded to canonical publication truth for the already-live PH1.OS negotiated microphone capability proof slice.

S08-03 remains partial and is not selected in this run because the negotiated microphone gate is the smaller direct live seam.

S08-06 and S08-08 remain partial and are not selected in this run because their direct runtime-law branch sets are already exhausted.

S08-01 remains partial with the post-H46 next exact winner `NOT_EXPLICIT` after H47 correction and is not selected in this run.

Section 06 remains parked with the next exact winner `NOT_EXPLICIT`.

## Proof Plan

The canonical publication basis for this H50 run is the already-live PH1.OS proof `at_os_22e_voice_live_entrypoint_rejects_android_voice_without_microphone`.

That proof already isolates the selected seam on `validate_voice_turn_platform_governance(...)` by removing `DeviceCapability::Microphone` while keeping Android platform alignment, WakeWord trigger alignment, negotiated `DeviceCapability::WakeWord`, and attested integrity posture intact.

The adjacent `at_os_22c`, `at_os_22d`, `at_os_22f`, `at_os_22g`, `at_os_22h`, `at_os_22i`, and `at_os_22j` proofs remain preserved and unselected in this run.

## Out Of Scope

This H50 publication does not authorize:

- source edits
- contract edits
- engine edits
- runtime-law edits
- build-section wording edits
- any H51 target or implementation publication
- re-authoring the historical H49 build-plan file
- broader `S08-03` device-capability-registry closure
- broader `S08-04` capability-negotiation closure beyond the already-live microphone seam
- broader `S08-06` compatibility-governance closure
- broader `S08-08` device-trust closure
- any Section 06, Section 09, Section 10, or Section 11 implementation work
