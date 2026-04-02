# H43 Section 08: First Post-H42 Canonical S08-06 Runtime-Law Upgrade-Required Compatibility Warning Build Plan

This H43 slice is now the first canonical Section 08 runtime-law `platform_trust_warning(...)` upgrade-required compatibility-warning proof slice inside `S08-06`.

## Objective

H40 remains live as the first canonical PH1.OS voice-live `runtime_execution_envelope.platform` normalization fail-closed proof slice.

H41 remains live as the first post-H40 canonical PH1.OS voice-live `platform_context.requested_trigger` normalization fail-closed proof slice.

H42 remains live as the first canonical Section 08 runtime-law `platform_trust_warning(...)` restricted-device degrade proof slice.

H39 remains live and Section 06 remains parked with next exact winner `NOT_EXPLICIT`.

## Current Repo Truth

current repo truth already preserved H40 `at_os_22i`, H41 `at_os_22j`, H42 `at_runtime_law_15_restricted_device_platform_trust_degrades_protected_execution`, and `at_runtime_law_01_conflicting_inputs_resolve_deterministically` before this run.

current authoritative docs already mapped unresolved compatibility-governance closure to `runtime_law.rs#L1010` before this run.

current source already exposed `RULE_PLATFORM_TRUST` and reason code `LAW_PLATFORM_TRUST_REQUIRED` before this run.

the live carrier path remains `RuntimeLawRuntime::evaluate(...)` -> `platform_trust_warning(...)`.

current `base_envelope()` default desktop platform context was not seam-isolated before this run because it came from `PlatformRuntimeContext::default_for_platform(AppPlatform::Desktop)` and therefore carried `DeviceTrustClass::StandardDevice`, `ClientIntegrityStatus::Unknown`, and `ClientCompatibilityStatus::Unknown`.

current contract validation already required `platform_context.platform_type == platform` before this run.

the exact canonical proof published by this run is `at_runtime_law_16_upgrade_required_platform_trust_degrades_protected_execution`.

no production logic change was required in this run.

## Exact Seam

the H43 proof kept the adjacent H42 restricted-device branch and adjacent `ClientIntegrityStatus::Unknown` warning branch aligned and unselected by using `DeviceTrustClass::StandardDevice` with `ClientIntegrityStatus::IntegrityVerified` and `ClientCompatibilityStatus::UpgradeRequired`.

## Implementation Boundary

the exact H40 canonical proof already live is `at_os_22i_voice_live_entrypoint_rejects_envelope_platform_mismatch`.

the exact H41 canonical proof already live is `at_os_22j_voice_live_entrypoint_rejects_requested_trigger_mismatch`.

the exact H42 canonical proof already live is `at_runtime_law_15_restricted_device_platform_trust_degrades_protected_execution`.

`S08-06` remains `PARTIAL`.

## Proof Plan

the proof published by this run stays on the existing runtime-law carrier without widening PH1.OS, engine, contract, or runtime-governance scope beyond the selected upgrade-required compatibility-warning seam.

the proof publishes `at_runtime_law_16_upgrade_required_platform_trust_degrades_protected_execution` only through `RuntimeLawRuntime::evaluate(...)`, with `DeviceTrustClass::StandardDevice`, `ClientIntegrityStatus::IntegrityVerified`, and `ClientCompatibilityStatus::UpgradeRequired`.

no post-H43 next exact winner is published in this run.

## Out Of Scope

Broader Section 08 closure remains out of scope for this run, including:

- the already-implemented H42 restricted-device trust-warning branch
- the adjacent `ClientIntegrityStatus::Unknown` warning branch
- broader `S08-06` compatibility governance closure beyond this proof slice
- mixed `S08-06` hard-block bundles already preserved by `at_runtime_law_01_conflicting_inputs_resolve_deterministically`
- `UnsupportedClient` and `IntegrityFailed` hard-block branches
- `S08-03` device capability registry closure
- `S08-04` capability negotiation closure
- `S08-07` platform event stream closure
- `S08-09` platform telemetry closure
- top-level PH1.OS closure
- any Section 06, Section 09, Section 10, or Section 11 implementation work
