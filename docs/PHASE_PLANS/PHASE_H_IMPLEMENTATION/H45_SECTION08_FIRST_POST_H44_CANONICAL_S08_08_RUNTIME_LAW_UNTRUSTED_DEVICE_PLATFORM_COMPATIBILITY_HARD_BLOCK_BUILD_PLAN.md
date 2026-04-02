# H45 Section 08: First Post-H44 Canonical S08-08 Runtime-Law Untrusted-Device Platform-Compatibility Hard-Block Build Plan

This is the first canonical H45 Section 08 build plan after the live H44 unsupported-client compatibility hard-block proof slice.

## Objective

the next exact active winner is now `S08-08`.

the exact seam is the first post-H44 canonical Section 08 runtime-law `platform_hard_block_required(...)` block path on `RuntimeLawRuntime::evaluate(...)`.

the smallest direct seam is the untrusted-device platform-compatibility hard-block branch via `DeviceTrustClass::UntrustedDevice` while compatibility and integrity remain non-blocking.

the exact candidate canonical proof for the next implementation is `at_runtime_law_18_untrusted_device_platform_compatibility_blocks_protected_execution`.

no code is changed in this run; this run only publishes the next active target.

## Current Repo Truth

H40 remains live as the first canonical PH1.OS voice-live `runtime_execution_envelope.platform` normalization fail-closed proof slice.

H41 remains live as the first post-H40 canonical PH1.OS voice-live `platform_context.requested_trigger` normalization fail-closed proof slice.

H42 remains live as the first canonical Section 08 runtime-law `platform_trust_warning(...)` restricted-device degrade proof slice.

H43 remains live as the first canonical Section 08 runtime-law `platform_trust_warning(...)` upgrade-required compatibility-warning proof slice.

H44 remains live as the first canonical Section 08 runtime-law `platform_hard_block_required(...)` unsupported-client compatibility hard-block proof slice.

the exact H40 canonical proof already live is `at_os_22i_voice_live_entrypoint_rejects_envelope_platform_mismatch`.

the exact H41 canonical proof already live is `at_os_22j_voice_live_entrypoint_rejects_requested_trigger_mismatch`.

the exact H42 canonical proof already live is `at_runtime_law_15_restricted_device_platform_trust_degrades_protected_execution`.

the exact H43 canonical proof already live is `at_runtime_law_16_upgrade_required_platform_trust_degrades_protected_execution`.

the exact H44 canonical proof already live is `at_runtime_law_17_unsupported_client_platform_compatibility_blocks_protected_execution`.

current authoritative docs already map unresolved device trust-level closure to `runtime_law.rs#L1019` through the current `S08-08` ledger row.

current source already blocks protected execution with `RULE_PLATFORM_COMPATIBILITY` and reason code `LAW_PLATFORM_COMPATIBILITY_REQUIRED`.

current repo truth already preserves the mixed platform-compatibility hard-block proof `at_runtime_law_01_conflicting_inputs_resolve_deterministically`.

current repo truth already suggests the untrusted-device platform-compatibility hard-block seam may be satisfiable by proof alone with zero production-logic edits.

no dedicated canonical proof has yet been published for the smaller untrusted-device platform-compatibility hard-block seam.

## Exact Seam

the exact seam is the first post-H44 canonical Section 08 runtime-law `platform_hard_block_required(...)` block path on `RuntimeLawRuntime::evaluate(...)`.

the live carrier path is `RuntimeLawRuntime::evaluate(...)` -> `platform_hard_block_required(...)`.

the smallest direct seam is the untrusted-device platform-compatibility hard-block branch via `DeviceTrustClass::UntrustedDevice` while compatibility and integrity remain non-blocking.

## Implementation Boundary

Section 06 remains parked with the next exact winner `NOT_EXPLICIT` and is not selected in this run.

no code is changed in this run; this run only publishes the next active target.

## Proof Plan

the exact candidate canonical proof for the next implementation is `at_runtime_law_18_untrusted_device_platform_compatibility_blocks_protected_execution`.

The next implementation should prove that `RuntimeLawRuntime::evaluate(...)` blocks protected execution through `platform_hard_block_required(...)` when `DeviceTrustClass::UntrustedDevice` is selected while compatibility and integrity remain non-blocking.

## Out Of Scope

S08-05 remains proven complete and the adjacent `ClientIntegrityStatus::Unknown` warning branch plus adjacent `ClientIntegrityStatus::IntegrityFailed` hard-block branch are not selected in this run.

S08-06 remains partial and the already-implemented `ClientCompatibilityStatus::UpgradeRequired` warning branch plus already-implemented `ClientCompatibilityStatus::UnsupportedClient` hard-block branch are not selected in this run.

the already-implemented H42 restricted-device warning branch is not selected in this run.

any broader `platform_trust_warning(...)` warning bundle remains out of scope for this run.
