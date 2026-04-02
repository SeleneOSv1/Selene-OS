# H43 Section 08: First Post-H42 Canonical S08-06 Runtime-Law Upgrade-Required Compatibility Warning Build Plan

This is the first canonical H43 Section 08 build plan after the live H42 restricted-device platform-trust proof slice.

## Objective

the next exact active winner is now `S08-06`.

the exact seam is the first post-H42 canonical Section 08 runtime-law `platform_trust_warning(...)` degrade path on `RuntimeLawRuntime::evaluate(...)`.

the smallest direct seam is the upgrade-required compatibility branch via `ClientCompatibilityStatus::UpgradeRequired` while device trust and integrity remain non-blocking.

no code is changed in this run; this run only publishes the next active target.

## Current Repo Truth

the live carrier path is `RuntimeLawRuntime::evaluate(...)` -> `platform_trust_warning(...)`.

current authoritative docs already map unresolved compatibility-governance closure to `runtime_law.rs#L1010` through the current `S08-06` ledger row.

current source already degrades protected execution with `RULE_PLATFORM_TRUST` and reason code `LAW_PLATFORM_TRUST_REQUIRED`.

the exact H40 canonical proof already live is `at_os_22i_voice_live_entrypoint_rejects_envelope_platform_mismatch`.

the exact H41 canonical proof already live is `at_os_22j_voice_live_entrypoint_rejects_requested_trigger_mismatch`.

the exact H42 canonical proof already live is `at_runtime_law_15_restricted_device_platform_trust_degrades_protected_execution`.

current repo truth already preserves the mixed platform-compatibility hard-block proof `at_runtime_law_01_conflicting_inputs_resolve_deterministically`.

current repo truth already suggests the upgrade-required compatibility-warning seam may be satisfiable by proof alone with zero production-logic edits.

no dedicated canonical proof has yet been published for the smaller upgrade-required compatibility-warning seam.

the exact candidate canonical proof for the next implementation is `at_runtime_law_16_upgrade_required_platform_trust_degrades_protected_execution`.

S08-08 remains partial but its restricted-device branch is already implemented and no longer the next active winner.

S08-05 remains proven complete and the adjacent `ClientIntegrityStatus::Unknown` warning branch is not selected in this run.

Section 06 remains parked with the next exact winner `NOT_EXPLICIT` and is not selected in this run.

## Exact Seam

the exact seam is the first post-H42 canonical Section 08 runtime-law `platform_trust_warning(...)` degrade path on `RuntimeLawRuntime::evaluate(...)`.

the smallest direct seam is the upgrade-required compatibility branch via `ClientCompatibilityStatus::UpgradeRequired` while device trust and integrity remain non-blocking.

## Implementation Boundary

The later implementation should stay on the existing runtime-law carrier without widening PH1.OS, engine, contract, or runtime-law scope beyond the selected upgrade-required compatibility-warning seam.

The later proof should keep device trust and integrity on non-blocking values so the selected seam stays isolated from the already-implemented restricted-device branch and the adjacent `ClientIntegrityStatus::Unknown` warning branch.

## Proof Plan

The later implementation should publish the first canonical proof that `RuntimeLawRuntime::evaluate(...)` degrades protected execution through `platform_trust_warning(...)` when compatibility is downgraded to `ClientCompatibilityStatus::UpgradeRequired` while device trust and integrity remain non-blocking.

The exact candidate canonical proof for the next implementation is `at_runtime_law_16_upgrade_required_platform_trust_degrades_protected_execution`.

That proof should remain adjacent to the already-live H40/H41 PH1.OS normalization proofs, the H42 restricted-device trust-warning proof, and the mixed runtime-law platform-compatibility hard-block proof without widening into broader Section 08 closure.

## Out Of Scope

Broader Section 08 closure remains out of scope for this run, including:

- the already-implemented H42 restricted-device trust-warning branch
- the adjacent `ClientIntegrityStatus::Unknown` warning branch
- mixed `S08-06` hard-block bundles already preserved by `at_runtime_law_01_conflicting_inputs_resolve_deterministically`
- `UnsupportedClient` and `IntegrityFailed` hard-block branches
- `S08-03` device capability registry closure
- `S08-04` capability negotiation closure
- `S08-07` platform event stream closure
- `S08-09` platform telemetry closure
- top-level PH1.OS closure
- any Section 06, Section 09, Section 10, or Section 11 implementation work
