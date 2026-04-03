# H48 Section 08: First Post-H47 Canonical S08-06 Runtime-Law Integrity-Failed Compatibility Hard-Block Build Plan

## Objective

This H48 slice is now the first canonical post-H47 Section 08 runtime-law `platform_hard_block_required(...)` integrity-failed compatibility hard-block proof slice inside `S08-06`.

The exact canonical proof implemented by this run is `at_runtime_law_19_integrity_failed_platform_compatibility_blocks_protected_execution`.

The live carrier path is `RuntimeLawRuntime::evaluate(...)` -> `platform_hard_block_required(...)`.

H48 remains published as the first canonical post-H47 Section 08 next-target publication.

`S08-05` remains PROVEN_COMPLETE and is not reopened in this run.

`S08-06` remains PARTIAL.

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

Current authoritative docs already mapped unresolved compatibility-governance closure to `runtime_law.rs#L1010` before this run.

Current source already exposed `RULE_PLATFORM_COMPATIBILITY` and reason code `LAW_PLATFORM_COMPATIBILITY_REQUIRED` before this run.

Current `base_envelope()` default desktop platform context was not seam-isolated before this run because it came from `PlatformRuntimeContext::default_for_platform(AppPlatform::Desktop)` and therefore carried `DeviceTrustClass::StandardDevice`, `ClientIntegrityStatus::Unknown`, and `ClientCompatibilityStatus::Unknown`.

Current `blocked_platform_envelope()` helper was not seam-isolated before this run because it already carried `ClientCompatibilityStatus::UnsupportedClient`, `ClientIntegrityStatus::IntegrityFailed`, and `DeviceTrustClass::UntrustedDevice`.

Current contract validation already required `platform_context.platform_type == platform` before this run.

Current platform runtime context validation only imposed attestation-only integrity metadata constraints before this run.

## Exact Seam

The exact seam is the first canonical post-H47 Section 08 runtime-law `platform_hard_block_required(...)` integrity-failed compatibility hard-block path on `RuntimeLawRuntime::evaluate(...)`.

The H48 proof kept the adjacent `ClientCompatibilityStatus::UnsupportedClient` hard-block branch, adjacent `DeviceTrustClass::UntrustedDevice` hard-block branch, adjacent `DeviceTrustClass::RestrictedDevice` warning branch, adjacent `ClientCompatibilityStatus::UpgradeRequired` warning branch, and adjacent `ClientIntegrityStatus::Unknown` warning branch aligned and unselected by using `DeviceTrustClass::StandardDevice` with `ClientIntegrityStatus::IntegrityFailed` and `ClientCompatibilityStatus::Compatible`.

## Implementation Boundary

The implementation remained bounded to the live runtime-law carrier on `RuntimeLawRuntime::evaluate(...)` -> `platform_hard_block_required(...)`.

No engine, contract, PH1.OS, or runtime-governance file changed in this run.

No post-H48 next exact winner is published in this run.

## Proof Plan

The exact canonical proof implemented by this run is `at_runtime_law_19_integrity_failed_platform_compatibility_blocks_protected_execution`.

The proof now establishes that a contract-valid runtime execution envelope reaches the live runtime-law carrier and blocks protected execution through `platform_hard_block_required(...)` when integrity is set to `ClientIntegrityStatus::IntegrityFailed` while device trust remains `DeviceTrustClass::StandardDevice` and compatibility remains `ClientCompatibilityStatus::Compatible`.

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
