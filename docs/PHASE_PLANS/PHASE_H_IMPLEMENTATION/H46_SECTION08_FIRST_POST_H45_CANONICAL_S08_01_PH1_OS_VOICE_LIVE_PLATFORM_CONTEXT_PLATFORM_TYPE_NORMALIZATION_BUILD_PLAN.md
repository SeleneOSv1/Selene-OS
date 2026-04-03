# H46 Section 08: First Post-H45 Canonical S08-01 PH1.OS Voice-Live Platform-Context Platform-Type Normalization Build Plan

This is the first canonical H46 Section 08 build plan after the live H45 untrusted-device platform-compatibility hard-block proof slice.

## Objective

the next exact active winner is now `S08-01`.

the exact seam is the first post-H45 canonical PH1.OS voice-live `platform_context.platform_type` normalization fail-closed path on `OsVoiceLiveTurnInput::v1_with_runtime_execution_envelope(...)`.

the smallest direct seam is the `runtime_execution_envelope.platform_context.platform_type` mismatch branch against the canonical platform derived from `voice_context.platform` while `runtime_execution_envelope.platform` and `platform_context.requested_trigger` remain canonical and unselected.

no code is changed in this run; this run only publishes the next active target.

## Current Repo Truth

the live carrier path is `OsVoiceLiveTurnInput::v1_with_runtime_execution_envelope(...)` -> `validate_voice_turn_platform_governance(...)` -> `app_platform_from_os_voice_platform(...)`.

current authoritative docs already map unresolved platform-identity closure to `ph1os.rs#L581` through the current `S08-01` ledger row.

current source already rejects `platform_context.platform_type` mismatches with the reason `must match canonical platform for voice_context`.

the exact H40 canonical proof already live is `at_os_22i_voice_live_entrypoint_rejects_envelope_platform_mismatch`.

the exact H41 canonical proof already live is `at_os_22j_voice_live_entrypoint_rejects_requested_trigger_mismatch`.

the exact H45 canonical proof already live is `at_runtime_law_18_untrusted_device_platform_compatibility_blocks_protected_execution`.

current repo truth already preserves adjacent `at_os_22c` through `at_os_22h` voice-entrypoint proofs.

H40 remains live as the first canonical PH1.OS voice-live `runtime_execution_envelope.platform` normalization fail-closed proof slice.

H41 remains live as the first post-H40 canonical PH1.OS voice-live `platform_context.requested_trigger` normalization fail-closed proof slice.

H42 remains live as the first canonical Section 08 runtime-law `platform_trust_warning(...)` restricted-device degrade proof slice.

H43 remains live as the first canonical Section 08 runtime-law `platform_trust_warning(...)` upgrade-required compatibility-warning proof slice.

H44 remains live as the first canonical Section 08 runtime-law `platform_hard_block_required(...)` unsupported-client compatibility hard-block proof slice.

H45 remains live as the first canonical Section 08 runtime-law `platform_hard_block_required(...)` untrusted-device platform-compatibility hard-block proof slice.

Section 06 remains parked with the next exact winner `NOT_EXPLICIT` and is not selected in this run.

## Exact Seam

the exact seam is the first post-H45 canonical PH1.OS voice-live `platform_context.platform_type` normalization fail-closed path on `OsVoiceLiveTurnInput::v1_with_runtime_execution_envelope(...)`.

no dedicated canonical proof has yet been published for the smaller `platform_context.platform_type` normalization seam.

the exact candidate canonical proof for the next implementation is `at_os_22k_voice_live_entrypoint_rejects_platform_context_platform_type_mismatch`.

## Implementation Boundary

the planned implementation remains bounded to the existing PH1.OS live voice-entrypoint carrier without widening engine, contract, or runtime-law surfaces.

`S08-03` and `S08-04` remain partial and are not selected in this run.

`S08-06` and `S08-08` remain partial and are not selected in this run.

## Proof Plan

the next implementation should publish the first canonical proof that the voice-live entrypoint fails closed when `runtime_execution_envelope.platform_context.platform_type` does not match the canonical platform derived from `voice_context.platform`.

that proof should keep the adjacent H40 `runtime_execution_envelope.platform` branch and adjacent H41 `platform_context.requested_trigger` branch canonical and unselected.

## Out Of Scope

Broader Section 08 closure remains out of scope for this run, including:

- the already-implemented H40 `runtime_execution_envelope.platform` branch
- the already-implemented H41 `platform_context.requested_trigger` branch
- adjacent `at_os_22c` through `at_os_22h` voice-entrypoint proofs
- `S08-03` device capability registry closure
- `S08-04` capability negotiation closure
- `S08-06` compatibility governance closure
- `S08-08` device trust-level closure
- `S08-07` platform event stream closure
- `S08-09` platform telemetry closure
- top-level PH1.OS closure
- any Section 06, Section 09, Section 10, or Section 11 implementation work
