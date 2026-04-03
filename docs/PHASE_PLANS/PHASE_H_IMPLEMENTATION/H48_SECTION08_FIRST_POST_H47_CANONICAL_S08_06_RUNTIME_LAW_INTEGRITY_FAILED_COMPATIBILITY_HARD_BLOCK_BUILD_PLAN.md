# H48 Section 08: First Post-H47 Canonical S08-06 Runtime-Law Integrity-Failed Compatibility Hard-Block Build Plan

## Objective

This is the first canonical H48 post-H47 Section 08 next-target publication build plan.

H46 remains published as the first canonical post-H45 Section 08 next-target publication.

H47 remains published as the first canonical post-H46 Section 08 frontier correction.

`S08-05` remains PROVEN_COMPLETE and is not reopened in this run.

The next exact active winner is now `S08-06`.

The exact seam is the first post-H47 canonical Section 08 runtime-law `platform_hard_block_required(...)` integrity-failed compatibility hard-block path on `RuntimeLawRuntime::evaluate(...)`.

No code is changed in this run; this run only publishes the next active target.

## Current Repo Truth

H40 remains live as the first canonical PH1.OS voice-live `runtime_execution_envelope.platform` normalization fail-closed proof slice.

H41 remains live as the first post-H40 canonical PH1.OS voice-live `platform_context.requested_trigger` normalization fail-closed proof slice.

H42 remains live as the first canonical Section 08 runtime-law `platform_trust_warning(...)` restricted-device degrade proof slice.

H43 remains live as the first canonical Section 08 runtime-law `platform_trust_warning(...)` upgrade-required compatibility-warning proof slice.

H44 remains live as the first canonical Section 08 runtime-law `platform_hard_block_required(...)` unsupported-client compatibility hard-block proof slice.

H45 remains live as the first canonical Section 08 runtime-law `platform_hard_block_required(...)` untrusted-device platform-compatibility hard-block proof slice.

The exact H42 canonical proof already live is `at_runtime_law_15_restricted_device_platform_trust_degrades_protected_execution`.

The exact H43 canonical proof already live is `at_runtime_law_16_upgrade_required_platform_trust_degrades_protected_execution`.

The exact H44 canonical proof already live is `at_runtime_law_17_unsupported_client_platform_compatibility_blocks_protected_execution`.

The exact H45 canonical proof already live is `at_runtime_law_18_untrusted_device_platform_compatibility_blocks_protected_execution`.

Current authoritative docs already map unresolved compatibility-governance closure to `runtime_law.rs#L1010` through the current `S08-06` ledger row.

Current source already exposes `RULE_PLATFORM_COMPATIBILITY` and reason code `LAW_PLATFORM_COMPATIBILITY_REQUIRED`.

Current repo truth already preserves adjacent `at_os_22c` through `at_os_22h` voice-entrypoint proofs and `at_runtime_law_01_conflicting_inputs_resolve_deterministically`.

Section 06 remains parked with the next exact winner `NOT_EXPLICIT` and is not selected in this run.

`S08-03` and `S08-04` remain partial and are not selected in this run.

`S08-08` remains partial and is not selected in this run.

## Exact Seam

The exact seam is the first post-H47 canonical Section 08 runtime-law `platform_hard_block_required(...)` integrity-failed compatibility hard-block path on `RuntimeLawRuntime::evaluate(...)`.

The smallest direct seam is the `runtime_execution_envelope.platform_context.integrity_status == ClientIntegrityStatus::IntegrityFailed` branch while `runtime_execution_envelope.platform_context.device_trust_class == DeviceTrustClass::StandardDevice` and `runtime_execution_envelope.platform_context.compatibility_status == ClientCompatibilityStatus::Compatible` keep adjacent warning and hard-block branches aligned and unselected.

The live carrier path is `RuntimeLawRuntime::evaluate(...)` -> `platform_hard_block_required(...)`.

Current source already keeps the residual `ClientIntegrityStatus::IntegrityFailed` hard-block seam contract-reachable because current platform runtime context validation only imposes attestation-only integrity metadata constraints and does not preempt `IntegrityFailed` by itself.

## Implementation Boundary

This publication stays on the existing runtime-law carrier without widening PH1.OS, engine, contract, or runtime-governance scope beyond the residual `ClientIntegrityStatus::IntegrityFailed` hard-block seam.

No dedicated canonical proof has yet been published for the residual `ClientIntegrityStatus::IntegrityFailed` hard-block seam.

The exact candidate canonical proof for the next implementation is `at_runtime_law_19_integrity_failed_platform_compatibility_blocks_protected_execution`.

The adjacent `ClientIntegrityStatus::Unknown` warning branch is not selected in this run.

## Proof Plan

This run publishes the first explicit post-H47 next target on the already-live Section 08 compatibility-governance carrier.

The next implementation should prove that a contract-valid runtime execution envelope reaches `RuntimeLawRuntime::evaluate(...)` and blocks protected execution through `platform_hard_block_required(...)` when integrity is set to `ClientIntegrityStatus::IntegrityFailed` while device trust remains `DeviceTrustClass::StandardDevice` and compatibility remains `ClientCompatibilityStatus::Compatible`.

That next implementation should preserve the already-implemented H44 unsupported-client hard-block proof slice, the already-implemented H45 untrusted-device hard-block proof slice, the already-implemented H42 restricted-device warning proof slice, the already-implemented H43 upgrade-required warning proof slice, and the adjacent `ClientIntegrityStatus::Unknown` warning branch as aligned and unselected.

## Out Of Scope

Broader Section 08 closure remains out of scope for this run, including:

- the already-implemented H44 unsupported-client hard-block branch
- the already-implemented H45 untrusted-device hard-block branch
- the already-implemented H42 restricted-device warning branch
- the already-implemented H43 upgrade-required warning branch
- the adjacent `ClientIntegrityStatus::Unknown` warning branch
- broader `S08-03` device capability registry closure
- broader `S08-04` capability negotiation closure
- broader `S08-08` device-trust closure
- broader `S08-11` top-level PH1.OS closure
- broader `S08-12` cross-platform closure
- any Section 06, Section 09, Section 10, or Section 11 implementation work
