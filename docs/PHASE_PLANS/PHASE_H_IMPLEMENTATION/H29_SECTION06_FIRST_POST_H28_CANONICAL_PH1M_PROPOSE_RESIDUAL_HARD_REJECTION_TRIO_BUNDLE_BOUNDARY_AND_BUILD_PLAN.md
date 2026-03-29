# H29 Section 06: First Post-H28 Canonical PH1.M Propose Residual Hard-Rejection Trio Bundle Boundary and Build Plan

## Objective
Current published repo truth already exposes the first canonical PH1.M propose eligibility decision-envelope slice through H28, while the accepted post-H28 continuation frontier result now establishes one exact next post-H28 Section 06 build unit. H29 publishes that next exact bounded build unit only.

## Current Published Repo Truth
Section 06 remains live but partial by current repo truth at [MASTER_BUILD_COMPLETION_PLAN.md#L158](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L158) and [MASTER_BUILD_COMPLETION_LEDGER.md#L214](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md#L214). The exact `S06-09` row still anchors to `memory eligibility checks function` at [SELENE_BUILD_SECTION_06.md#L499](/Users/selene/Documents/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_06.md#L499).

Current repo truth already exposes the exact residual hard-rejection trio carrier:
- `MemoryCommitDecision` at [selene_kernel_contracts/ph1m.rs#L338](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L338)
- `Ph1mProposeResponse.decisions` at [selene_kernel_contracts/ph1m.rs#L2575](/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs#L2575)
- pre-loop unknown-speaker rejection at [selene_engines/ph1m.rs#L168](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L168)
- pre-loop privacy-mode rejection at [selene_engines/ph1m.rs#L183](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L183)
- loop-level do-not-store rejection at [selene_engines/ph1m.rs#L202](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L202)
- stored-vs-updated commit outcomes remain distinct at [selene_engines/ph1m.rs#L299](/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs#L299)
- canonical PH1.M `MemoryTurnOutput::Propose(...)` forwarding at [ph1m.rs#L639](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L639)
- propose persistence remains bounded to forwarded `ledger_events` at [ph1m.rs#L334](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L334)
- the published H28 real-runtime prove surface at [ph1m.rs#L2016](/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1m.rs#L2016)

## Exact Winner
The exact H29 winner is `S06-09_PROPOSE_RESIDUAL_UNPUBLISHED_HARD_REJECTION_TRIO_BUNDLE`.

This bundle covers pre-loop unknown-speaker rejection, pre-loop privacy-mode rejection, and loop-level do-not-store rejection only.

Denied-consent and sensitive-needs-consent remain already proven within H28 and are out of scope for H29.

The smaller pre-loop pair fails because do-not-store remains an unpublished same-carrier sibling outside that pair.

The broader loop policy and consent family fails because it is already partly published and broader than one exact build unit.

Broader S06-09 eligibility closure remains partial after H29 authoring.

## Primary Carrier And Boundary
The lawful H29 carrier remains the existing `MemoryCommitDecision` / `Ph1mProposeResponse.decisions` / canonical `MemoryTurnOutput::Propose(...)` boundary.

The current-source carrier files cited by this H29 block are `crates/selene_kernel_contracts/src/ph1m.rs`, `crates/selene_engines/src/ph1m.rs`, and `crates/selene_os/src/ph1m.rs`.

H29 is a bounded post-H28 build block for the residual hard-rejection trio only.

## Settled Background
Broader non-`S06-09` rivals remain carried-forward settled background and are not reopened in H29.

The accepted post-H28 corrective frontier result and accepted post-H28 bundle-lawfulness continuation result remain the settled frontier basis for this publication run.

## Deterministic Invariants
H29 mirrors current repo truth instead of inventing new semantics:
- unknown-speaker rejection remains a pre-loop rejected decision
- privacy-mode rejection remains a pre-loop rejected decision
- do-not-store rejection remains a loop-level rejected decision
- denied-consent remains already proven within H28
- sensitive-needs-consent remains already proven within H28
- Stored-vs-updated commit outcomes remain out of scope for H29.

H29 does not claim broader `S06-09` closure, broader `S06-10` closure, broader `S06-12` closure, broader `S06-19` closure, or broader Section 06 completion.

## Proof Surface
This is a docs-only publication build block. No new code proof is authored in H29.

The publication basis remains bounded to the already-published H28 proof and the accepted post-H28 continuation frontier result that exposed the residual hard-rejection trio bundle as the exact next build unit.

## Out-Of-Scope And Do-Not-Touch Areas
This H29 seam does not authorize:
- code edits
- contract edits
- engine-logic edits
- storage edits
- build-section wording edits
- broader `S06-09` eligibility closure
- broader `S06-10` closure
- broader `S06-12` closure
- any `S06-19` decay-model closure
- any broader non-`S06-09` frontier re-audit
- broader Section 06 completion claims

## Ordered Build Plan
1. Start from clean `main` with `HEAD == origin/main`.
2. Re-confirm that the exact `S06-09` row still anchors to `Design [SELENE_BUILD_SECTION_06.md#L499]`.
3. Re-confirm that current repo truth still exposes the residual trio carrier in contracts, engine logic, and PH1.M runtime forwarding.
4. Author this H29 build block as the first post-H28 canonical PH1.M propose residual hard-rejection trio bundle boundary and build plan.
5. Update the master plan to publish the exact post-H28 Section 06 winner as `S06-09_PROPOSE_RESIDUAL_UNPUBLISHED_HARD_REJECTION_TRIO_BUNDLE` while keeping H28 published and broader closure disclosures intact.
6. Update the master ledger to keep `S06-09` as `PARTIAL`, preserve the exact `Design [SELENE_BUILD_SECTION_06.md#L499]` anchor on the exact `S06-09` row, preserve the H28 publication update line, and add the H29 publication update line.
7. Verify that the write set remains bounded to the two tracked master docs plus this one new H29 file, with no staged files.

## Verification And Publication Proof
The publication run must prove:
- this H29 plan file exists
- the master plan still states that H28 remains published
- the master plan now states that the next exact post-H28 Section 06 winner is published by this H29 file
- the master plan still states that broader `S06-09` eligibility closure remains partial
- the master plan still states that broader `S06-10` and `S06-12` closure remain partial and that `S06-19` remains unclaimed
- the master ledger still keeps `S06-09` as `PARTIAL`
- the master ledger still keeps the exact `Design [SELENE_BUILD_SECTION_06.md#L499]` anchor on the exact `S06-09` row
- the master ledger still preserves the H28 publication update line
- the master ledger now adds the H29 publication update line
- the write set remains docs-only and no file is staged

## Stop Conditions
Stop if repo truth no longer shows the canonical `MemoryCommitDecision` / `Ph1mProposeResponse.decisions` / `MemoryTurnOutput::Propose(...)` carrier, the exact pre-loop unknown-speaker branch, the exact pre-loop privacy-mode branch, or the exact loop-level do-not-store branch.

Stop if the publication attempt requires any code edit, any build-section wording edit, any broader non-`S06-09` frontier re-audit, or any claim broader than this exact residual hard-rejection trio bundle.
