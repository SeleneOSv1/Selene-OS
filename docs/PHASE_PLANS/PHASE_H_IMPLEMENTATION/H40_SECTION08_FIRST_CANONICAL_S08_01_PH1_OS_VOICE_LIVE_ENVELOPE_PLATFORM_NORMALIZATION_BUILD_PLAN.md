# H40 Section 08: First Canonical S08-01 PH1.OS Voice-Live Envelope Platform Normalization Build Plan

This is the first canonical H40 Section 08 build plan after the live H39 post-H38 Section 06 frontier correction.

## Objective

Publish the next exact active winner as `S08-01`.

The exact seam is the first canonical PH1.OS voice-live `runtime_execution_envelope.platform` normalization fail-closed path on `OsVoiceLiveTurnInput::v1_with_runtime_execution_envelope(...)`.

No code is changed in this run; this run only publishes the next active target.

## Current Repo Truth

The live carrier path is `OsVoiceLiveTurnInput::v1_with_runtime_execution_envelope(...)` -> `validate_voice_turn_platform_governance(...)` -> `app_platform_from_os_voice_platform(...)`.

Current repo truth already rejects `runtime_execution_envelope.platform` mismatches against `voice_context.platform`.

The current rejection reason already exists as `must match os_voice_live_turn_input.top_level_turn_input.voice_context.platform`.

Current repo truth already preserves adjacent PH1.OS voice-entrypoint proofs for trigger governance, attestation integrity, negotiated microphone capability, trusted capture artifact, and retention-deadline enforcement through `at_os_22c` through `at_os_22h`.

No dedicated canonical proof has yet been published for the smaller envelope-platform mismatch seam.

Current repo truth suggests the later implementation may be proof-first and may require zero production-logic edits.

Section 06 remains parked with the next exact winner `NOT_EXPLICIT` and is not selected in this run.

## Exact Seam

The next exact active winner is now `S08-01`.

The H40 slice remains limited to the `runtime_execution_envelope.platform` mismatch branch only.

The adjacent `platform_context.platform_type` normalization branch remains preserved but is not selected in this H40 slice.

## Implementation Boundary

The later implementation should stay on the existing PH1.OS live voice-entrypoint carrier.

The exact candidate canonical proof for the next implementation is `at_os_22i_voice_live_entrypoint_rejects_envelope_platform_mismatch`.

## Proof Plan

The later implementation should publish the first canonical proof that the voice-live entrypoint fails closed when `runtime_execution_envelope.platform` does not match the canonical platform derived from `voice_context.platform`.

That proof should remain adjacent to the already-live `at_os_22c` through `at_os_22h` voice-entrypoint coverage without widening into broader Section 08 closure.

## Out Of Scope

Broader Section 08 closure remains out of scope for this run, including:

- generalized `S08-01` architecture closure
- `S08-03` device capability registry closure
- `S08-04` capability negotiation closure
- `S08-06` compatibility governance closure
- `S08-08` device trust-level closure
- the adjacent `platform_context.platform_type` normalization branch
- any Section 06, Section 09, Section 10, or Section 11 implementation work
