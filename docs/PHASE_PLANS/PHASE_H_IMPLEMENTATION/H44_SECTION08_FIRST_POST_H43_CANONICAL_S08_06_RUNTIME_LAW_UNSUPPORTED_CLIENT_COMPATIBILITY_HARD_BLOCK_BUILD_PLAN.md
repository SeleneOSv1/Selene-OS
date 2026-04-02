# H44 Section 08: First Post-H43 Canonical S08-06 Runtime-Law Unsupported-Client Compatibility Hard-Block Build Plan

This is the first canonical H44 Section 08 build plan after the live H43 upgrade-required compatibility-warning proof slice.

## Objective

the next exact active winner remains `S08-06`.

the exact seam is the first post-H43 canonical Section 08 runtime-law `platform_hard_block_required(...)` block path on `RuntimeLawRuntime::evaluate(...)`.

the smallest direct seam is the unsupported-client compatibility hard-block branch via `ClientCompatibilityStatus::UnsupportedClient` while integrity and device trust remain non-blocking.

no code is changed in this run; this run only publishes the next active target.

## Current Repo Truth

the live carrier path is `RuntimeLawRuntime::evaluate(...)` -> `platform_hard_block_required(...)`.

current authoritative docs already map unresolved compatibility-governance closure to `runtime_law.rs#L1010` through the current `S08-06` ledger row.

current source already blocks protected execution with `RULE_PLATFORM_COMPATIBILITY` and reason code `LAW_PLATFORM_COMPATIBILITY_REQUIRED`.

the exact H40 canonical proof already live is `at_os_22i_voice_live_entrypoint_rejects_envelope_platform_mismatch`.

the exact H41 canonical proof already live is `at_os_22j_voice_live_entrypoint_rejects_requested_trigger_mismatch`.

the exact H42 canonical proof already live is `at_runtime_law_15_restricted_device_platform_trust_degrades_protected_execution`.

the exact H43 canonical proof already live is `at_runtime_law_16_upgrade_required_platform_trust_degrades_protected_execution`.

current repo truth already preserves the mixed platform-compatibility hard-block proof `at_runtime_law_01_conflicting_inputs_resolve_deterministically`.

current repo truth already suggests the unsupported-client compatibility hard-block seam may be satisfiable by proof alone with zero production-logic edits.

no dedicated canonical proof has yet been published for the smaller unsupported-client compatibility hard-block seam.

the exact candidate canonical proof for the next implementation is `at_runtime_law_17_unsupported_client_platform_compatibility_blocks_protected_execution`.

S08-05 remains proven complete and the adjacent `ClientIntegrityStatus::Unknown` warning branch plus adjacent `ClientIntegrityStatus::IntegrityFailed` hard-block branch are not selected in this run.

S08-08 remains partial and the adjacent `DeviceTrustClass::UntrustedDevice` hard-block branch is not selected in this run.

Section 06 remains parked with the next exact winner `NOT_EXPLICIT` and is not selected in this run.

## Exact Seam

the exact seam is the first post-H43 canonical Section 08 runtime-law `platform_hard_block_required(...)` block path on `RuntimeLawRuntime::evaluate(...)`.

the smallest direct seam is the unsupported-client compatibility hard-block branch via `ClientCompatibilityStatus::UnsupportedClient` while integrity and device trust remain non-blocking.

## Implementation Boundary

The later implementation should stay on the existing runtime-law carrier without widening PH1.OS, engine, contract, or runtime-law scope beyond the selected unsupported-client compatibility hard-block seam.

The later proof should keep integrity and device trust on non-blocking values so the selected seam stays isolated from the adjacent `ClientIntegrityStatus::IntegrityFailed` hard-block branch and the adjacent `DeviceTrustClass::UntrustedDevice` hard-block branch.

## Proof Plan

The later implementation should publish the first canonical proof that `RuntimeLawRuntime::evaluate(...)` blocks protected execution through `platform_hard_block_required(...)` when compatibility is downgraded to `ClientCompatibilityStatus::UnsupportedClient` while integrity and device trust remain non-blocking.

The exact candidate canonical proof for the next implementation is `at_runtime_law_17_unsupported_client_platform_compatibility_blocks_protected_execution`.

That proof should remain adjacent to the already-live H40/H41 PH1.OS normalization proofs, the H42 and H43 runtime-law warning proofs, and the mixed runtime-law platform-compatibility hard-block proof without widening into broader Section 08 closure.

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
