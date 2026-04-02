# H44 Section 08: First Post-H43 Canonical S08-06 Runtime-Law Unsupported-Client Compatibility Hard-Block Build Plan

This H44 slice is now the first canonical Section 08 runtime-law `platform_hard_block_required(...)` unsupported-client compatibility hard-block proof slice inside `S08-06`.

## Objective

the exact canonical proof published by this run is `at_runtime_law_17_unsupported_client_platform_compatibility_blocks_protected_execution`.

the live carrier path is `RuntimeLawRuntime::evaluate(...)` -> `platform_hard_block_required(...)`.

no production logic change was required in this run.

## Current Repo Truth

H40 remains live as the first canonical PH1.OS voice-live `runtime_execution_envelope.platform` normalization fail-closed proof slice.

H41 remains live as the first post-H40 canonical PH1.OS voice-live `platform_context.requested_trigger` normalization fail-closed proof slice.

H42 remains live as the first canonical Section 08 runtime-law `platform_trust_warning(...)` restricted-device degrade proof slice.

H43 remains live as the first canonical Section 08 runtime-law `platform_trust_warning(...)` upgrade-required compatibility-warning proof slice.

H39 remains live and Section 06 remains parked with next exact winner `NOT_EXPLICIT`.

current repo truth already preserved H40 `at_os_22i`, H41 `at_os_22j`, H42 `at_runtime_law_15_restricted_device_platform_trust_degrades_protected_execution`, H43 `at_runtime_law_16_upgrade_required_platform_trust_degrades_protected_execution`, and `at_runtime_law_01_conflicting_inputs_resolve_deterministically` before this run.

current authoritative docs already mapped unresolved compatibility-governance closure to `runtime_law.rs#L1010` before this run.

current source already exposed `RULE_PLATFORM_COMPATIBILITY` and reason code `LAW_PLATFORM_COMPATIBILITY_REQUIRED` before this run.

current `base_envelope()` default desktop platform context was not seam-isolated before this run because it came from `PlatformRuntimeContext::default_for_platform(AppPlatform::Desktop)` and therefore carried `DeviceTrustClass::StandardDevice`, `ClientIntegrityStatus::Unknown`, and `ClientCompatibilityStatus::Unknown`.

current `blocked_platform_envelope()` helper was not seam-isolated before this run because it already carried `ClientCompatibilityStatus::UnsupportedClient`, `ClientIntegrityStatus::IntegrityFailed`, and `DeviceTrustClass::UntrustedDevice`.

current contract validation already required `platform_context.platform_type == platform` before this run.

## Exact Seam

the exact seam is the first canonical Section 08 runtime-law `platform_hard_block_required(...)` unsupported-client compatibility hard-block proof slice inside `S08-06`.

the H44 proof kept the adjacent `ClientIntegrityStatus::IntegrityFailed` hard-block branch, adjacent `DeviceTrustClass::UntrustedDevice` hard-block branch, and adjacent `ClientIntegrityStatus::Unknown` warning branch aligned and unselected by using `DeviceTrustClass::StandardDevice` with `ClientIntegrityStatus::IntegrityVerified` and `ClientCompatibilityStatus::UnsupportedClient`.

## Implementation Boundary

the implementation remained bounded to the live runtime-law carrier on `RuntimeLawRuntime::evaluate(...)` -> `platform_hard_block_required(...)`.

no engine, contract, PH1.OS, or runtime-governance file changed in this run.

## Proof Plan

the exact canonical proof published by this run is `at_runtime_law_17_unsupported_client_platform_compatibility_blocks_protected_execution`.

the proof now establishes that a contract-valid runtime execution envelope reaches the live runtime-law carrier and blocks protected execution through `platform_hard_block_required(...)` when compatibility is set to `ClientCompatibilityStatus::UnsupportedClient` while integrity and device trust remain non-blocking.

`S08-06` remains `PARTIAL`.

no post-H44 next exact winner is published in this run.

## Out Of Scope

Broader Section 08 closure remains out of scope for this run, including:

- the adjacent `ClientIntegrityStatus::Unknown` warning branch
- the adjacent `ClientIntegrityStatus::IntegrityFailed` hard-block branch
- the adjacent `DeviceTrustClass::UntrustedDevice` hard-block branch
- mixed `S08-06` hard-block bundles already preserved by `at_runtime_law_01_conflicting_inputs_resolve_deterministically`
- broader `S08-06` compatibility governance closure beyond this proof seam
- the already-implemented H42 restricted-device warning branch
- the already-implemented H43 upgrade-required warning branch
- `S08-03` device capability registry closure
- `S08-04` capability negotiation closure
- `S08-07` platform event stream closure
- `S08-09` platform telemetry closure
- top-level PH1.OS closure
- any Section 06, Section 09, Section 10, or Section 11 implementation work
