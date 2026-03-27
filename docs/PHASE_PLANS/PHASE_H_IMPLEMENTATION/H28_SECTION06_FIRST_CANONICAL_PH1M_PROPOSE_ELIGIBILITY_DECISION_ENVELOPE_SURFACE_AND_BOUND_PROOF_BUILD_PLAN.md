# H28 Section 06: First Canonical PH1.M Propose Eligibility Decision-Envelope Surface and Bound Proof Build Plan

## Objective
Current published repo truth already exposes PH1.M Hot/Warm/Cold retention-tier publication through H23, confidence/provenance and conflict-handling trust-signal publication through H24 and H25, stale-expiry retention publication through H26, unresolved-resume priority publication through H27, and live propose eligibility decision-envelope carriers in `selene_kernel_contracts::ph1m`, `selene_engines::ph1m`, and `crates/selene_os/src/ph1m`. Canonical PH1.M proof still did not show a real propose-specific eligibility decision-envelope slice before this H28 run. This H28 document freezes the first canonical PH1.M propose eligibility decision-envelope surface-and-proof slice only.

## Current Published Repo Truth
Section 06 remains live but partial by current repo truth at [MASTER_BUILD_COMPLETION_PLAN.md#L158](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L158), [COVERAGE_MATRIX.md#L10](/Users/selene/Documents/Selene-OS/docs/COVERAGE_MATRIX.md#L10), and [CORE_ARCHITECTURE.md#L953](/Users/selene/Documents/Selene-OS/docs/CORE_ARCHITECTURE.md#L953). The execution order still places PH1.M inside Section 06 after Sections 01-05 at [SELENE_BUILD_EXECUTION_ORDER.md#L145](/Users/selene/Documents/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md#L145), and the authoritative engine inventory still identifies PH1.M as the Section 06 storage/learning engine at [SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md#L86](/Users/selene/Documents/Selene-OS/docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md#L86).

Current repo truth already exposes the exact propose eligibility decision-envelope carrier:
- `MemoryCommitStatus::{NeedsConsent, Rejected}` and `MemoryCommitDecision.reason_code` / `MemoryCommitDecision.consent_prompt` at [selene_kernel_contracts/ph1m.rs#L329](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L329)
- `Ph1mProposeResponse.decisions` and `Ph1mProposeResponse.ledger_events` at [selene_kernel_contracts/ph1m.rs#L2574](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L2574)
- engine-side propose rejection / needs-consent logic at [selene_engines/ph1m.rs#L166](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L166)
- stored-vs-updated commit outcomes remain distinct at [selene_engines/ph1m.rs#L299](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L299)
- canonical PH1.M `MemoryTurnOutput::Propose(...)` forwarding at [ph1m.rs#L639](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L639)
- propose persistence remains bounded to forwarded `ledger_events` at [ph1m.rs#L334](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L334)

Current repo truth already exposed bounded engine proof through `ph1m::tests::at_m_05_sensitive_requires_confirmation` and `ph1m::tests::suppression_do_not_store_blocks_memory_propose`, and bounded PH1.M proof through `ph1m::tests::at_m_08_propose_forwarded` and `ph1m::tests::at_m_19_persist_forwarded_propose_commits_memory_rows`. Current PH1.M proof still did not show a real propose-specific eligibility decision-envelope slice before this H28 run because no real-runtime propose-specific PH1.M test existed.

## Exact Winner
The exact H28 winner is:
- add one bounded PH1.M real-runtime test inside the `#[cfg(test)]` surface of `crates/selene_os/src/ph1m.rs`
- add `at_m_28_real_runtime_propose_eligibility_decisions_recorded_in_envelope`
- prove canonical `MemoryTurnOutput::Propose(resp)` output records explicit eligibility-style decisions through the real engine/runtime path without any production logic widening

This slice is the first canonical PH1.M propose eligibility decision-envelope surface-and-proof slice only.

- the first canonical H28 PH1.M propose eligibility decision-envelope slice is published
- broader S06-09 eligibility closure remains partial after H28
- broader S06-10 closure remains partial after H28
- broader S06-12 closure remains partial after H28
- S06-19 decay-model closure remains unclaimed after H28

## Primary Carrier And Boundary
The lawful H28 carrier stays bounded to the existing PH1.M contract and runtime surfaces:
- decision-envelope contracts in `crates/selene_kernel_contracts/src/ph1m.rs`
- engine-side propose gating for privacy / do-not-store / denied-consent / sensitive-needs-consent in `crates/selene_engines/src/ph1m.rs`
- canonical PH1.M `Propose` forwarding and persisted-propose boundary in `crates/selene_os/src/ph1m.rs`

No storage edit, contract edit, engine-logic edit, or build-section wording edit is authorized in this run.

## Deterministic Invariants
H28 must mirror current repo truth instead of inventing new semantics:
- one decision surfaces as `MemoryCommitStatus::Rejected`
- the rejected decision uses `selene_engines::ph1m::reason_codes::M_POLICY_BLOCKED`
- the rejected decision surfaces `consent_prompt == None`
- one decision surfaces as `MemoryCommitStatus::NeedsConsent`
- the needs-consent decision uses `selene_engines::ph1m::reason_codes::M_NEEDS_CONSENT`
- the needs-consent decision surfaces `consent_prompt.is_some()`
- `resp.ledger_events` remains empty because no stored or updated outcome is emitted in this bounded proof

H28 proves eligibility-decision envelope truth on the live `Propose` carrier. It does not prove broader identity-scope validation closure, policy-compliance closure, confidence-threshold closure, sensitivity-rule closure, relevance-scoring closure, or any broader Section 06 architecture closure.

## Proof Surface
Current repo truth already exposes the bounded engine proof surface:
- `ph1m::tests::at_m_05_sensitive_requires_confirmation`
- `ph1m::tests::suppression_do_not_store_blocks_memory_propose`

Current PH1.M truth already exposes bounded canonical proof:
- `ph1m::tests::at_m_08_propose_forwarded`
- `ph1m::tests::at_m_19_persist_forwarded_propose_commits_memory_rows`

The new H28 proof required by this seam is:
- `ph1m::tests::at_m_28_real_runtime_propose_eligibility_decisions_recorded_in_envelope` at [ph1m.rs#L2016](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L2016)

These tests stay bounded to the real engine/runtime path plus canonical PH1.M forwarding and the persisted-propose no-write boundary.

## Out-Of-Scope And Do-Not-Touch Areas
This H28 seam does not authorize:
- storage edits
- contract edits
- engine-logic edits
- broader `S06-09` eligibility closure
- broader `S06-10` closure
- broader `S06-12` closure
- broader `S06-11` consistency closure
- broader `S06-16` snapshot-integrity closure
- broader `S06-20` certification-target closure
- any `S06-19` decay-model closure
- build-section wording edits
- broader Section 06 completion claims

## Ordered Build Plan
1. Start from clean `main` with `HEAD == origin/main` and re-confirm that current plan truth still says the post-H27 next exact Section 06 winner is not yet published.
2. Re-confirm that the propose eligibility decision-envelope carrier already exists in contracts, engine logic, and PH1.M forwarding.
3. Re-confirm that the existing engine and PH1.M propose proof surface is green before edits.
4. Add one bounded PH1.M real-runtime propose test that proves a `Rejected` eligibility decision and a `NeedsConsent` eligibility decision in the canonical response envelope.
5. Publish H28 in the master plan and master ledger without claiming broader `S06-09`, `S06-10`, `S06-12`, `S06-11`, `S06-16`, `S06-20`, or `S06-19` closure.
6. Run the bounded engine and PH1.M proof surface only.
7. Run the post-H28 read-only frontier audit and confirm that no exact next Section 06 winner is yet published.

## Verification And Publication Proof
The implementation run must prove:
- the H28 plan file exists
- the master plan now records that the first canonical H28 PH1.M propose eligibility decision-envelope slice is now published
- the master ledger now records that the first canonical H28 PH1.M propose eligibility decision-envelope slice is published
- real PH1.M wiring now proves propose eligibility decision output over the propose engine/runtime path
- the new PH1.M real-runtime test exists and executes
- the pre-existing bounded engine and PH1.M prove surface still passes
- the post-H28 read-only frontier audit still reports that no exact next Section 06 winner is yet published

Publication is lawful only if this proof stays bounded to the PH1.M test surface, the existing engine proof surface, and master-doc publication truth. Broader Section 06 closure remains partial after H28.

## Stop Conditions
Stop if repo truth no longer shows the propose decision-envelope contract surface in `selene_kernel_contracts::ph1m`, engine-side propose rejection / needs-consent logic in `selene_engines::ph1m`, or canonical `Propose` forwarding in `crates/selene_os/src/ph1m.rs`.

Stop if the real PH1.M proof would require a production logic change, storage edit, contract edit, broader eligibility integration, broader retention closure, broader trust closure, broader consistency closure, broader snapshot-integrity closure, broader certification-target closure, or any decay-model implementation. Stop if the implementation attempt tries to claim more than this first canonical PH1.M propose eligibility decision-envelope surface-and-proof slice.
