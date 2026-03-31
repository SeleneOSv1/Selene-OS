# H32 Section 06: First Post-H31 Canonical PH1.M Propose Native Closure Publication Correction Build Plan

## Objective
Current published repo truth still carries broader `S06-09` partial wording after H31 even though the accepted post-H31 audits now settle the narrower PH1.M `Propose` native closure question. This H32 run is a docs-only publication/correction build that updates repo truth to match the already-proven H28/H30 native closure state only.

## Accepted Starting Authority
The settled starting authority for this docs-only correction is:
- the accepted post-H31 scope-separation result `EXACT_PROPOSE_FRONTIER_NARROWS_TO_IDENTITY_POLICY_SENSITIVITY`
- the accepted post-H31 native-frontier result `NOT_EXPLICIT_NATIVE_SIBLING_DEADLOCK`
- the accepted post-H31 native-closure-status result `NOT_EXPLICIT_NATIVE_CLOSURE_STATUS`
- the accepted post-H31 native residual-exposure result `EXACT_NO_BROADER_NATIVE_CONTINUATION_EXPOSED`

## Current Published Repo Truth
Section 06 remains live in current repo truth at [MASTER_BUILD_COMPLETION_PLAN.md#L158](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L158) and [MASTER_BUILD_COMPLETION_LEDGER.md#L214](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md#L214). The exact `S06-09` row still anchors to `memory eligibility checks function` at [SELENE_BUILD_SECTION_06.md#L499](/Users/selene/Documents/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_06.md#L499).

Current repo truth already publishes:
- H28 as the first canonical PH1.M propose eligibility decision-envelope slice
- H29 as the accepted residual hard-rejection trio boundary winner
- H30 as the first canonical PH1.M propose residual hard-rejection trio proof slice
- H31 as the first canonical PH1.M propose updated persistence continuation slice

Current source and the accepted post-H31 native residual-exposure audit now establish that no broader unpublished PH1.M `Propose` native continuation remains exposed inside identity-scope, policy-compliance, or sensitivity-validation, and that the narrowed PH1.M `Propose` native frontier is already satisfied by the published H28/H30 chain.

## Exact Correction
The exact H32 correction is docs-only:
- publish this H32 docs-only publication/correction build block
- update the master plan so it publishes this H32 correction and states that `S06-09` is now `PROVEN_COMPLETE`
- update the master ledger so the exact `S06-09` row is corrected to `status=PROVEN_COMPLETE` with `blocker=NONE`
- keep H31 carried forward as adjacent persistence truth

No new code proof is authored in H32. No source edits, no build-section wording edits, and no broader Section 06 completion claim are authorized in this run.

## Deterministic Invariants
H32 mirrors settled repo truth instead of inventing new semantics:
- `EXACT_NO_BROADER_NATIVE_CONTINUATION_EXPOSED` remains the accepted post-H31 native residual-exposure result
- no broader unpublished PH1.M `Propose` native continuation remains exposed inside identity-scope, policy-compliance, or sensitivity-validation
- the narrowed PH1.M `Propose` native frontier is already satisfied by the published H28/H30 chain
- H31 remains carried-forward adjacent persistence truth
- broader non-`S06-09` rivals remain unopened
- broader `S06-10`, broader `S06-12`, and `S06-19` disclosures remain unchanged

## Proof Surface
This is a docs-only publication/correction build block. No new code proof is authored in H32.

The publication basis remains bounded to the already-published H28/H30 chain, the carried-forward H31 persistence truth, and the accepted post-H31 native residual-exposure result that resolved the narrowed PH1.M `Propose` native frontier without authorizing any new code seam.

## Out-Of-Scope And Do-Not-Touch Areas
This H32 seam does not authorize:
- source edits
- contract edits
- engine-logic edits
- storage edits
- build-section wording edits
- broader non-`S06-09` frontier re-audit
- broader `S06-10` closure
- broader `S06-12` closure
- any `S06-19` decay-model closure
- broader Section 06 completion claims

## Ordered Build Plan
1. Start from clean `main` with `HEAD == origin/main`.
2. Re-confirm that the exact `S06-09` row still anchors to `Design [SELENE_BUILD_SECTION_06.md#L499]`.
3. Re-confirm that current repo truth still publishes H28, H29, H30, and H31 while broader `S06-09` wording remains uncorrected before this run.
4. Re-confirm that the current native `Propose` carrier and runtime proof surface still match the already-accepted H28/H30 native closure basis.
5. Author this H32 docs-only publication/correction build plan.
6. Update the master plan to publish this H32 correction, remove the broader `S06-09` partial wording, and state that `S06-09` is now `PROVEN_COMPLETE` while preserving broader `S06-10`, `S06-12`, and `S06-19` disclosures.
7. Update the master ledger so the exact `S06-09` row records `status=PROVEN_COMPLETE`, preserves `current_state=memory eligibility checks function`, records `blocker=NONE`, preserves `Design [SELENE_BUILD_SECTION_06.md#L499]`, preserves the H28/H29/H30/H31 publication update lines, and adds the H32 publication update line.
8. Verify that the write set remains docs-only and bounded to the two tracked master docs plus this one new H32 file, with no staged files.
9. Re-run the bounded exact engine and PH1.M tests that already support the published H28/H30/H31 chain.

## Verification And Publication Proof
The publication run must prove:
- this H32 plan file exists
- this H32 plan file is explicitly docs-only
- this H32 plan file records `EXACT_NO_BROADER_NATIVE_CONTINUATION_EXPOSED`
- this H32 plan file states that no broader unpublished PH1.M `Propose` native continuation remains exposed
- this H32 plan file states that the narrowed native frontier is already satisfied by the published H28/H30 chain
- the master plan now publishes this H32 correction
- the master plan now states that `S06-09` is `PROVEN_COMPLETE`
- the master plan no longer states that broader `S06-09` eligibility closure remains partial
- the master ledger now records `status=PROVEN_COMPLETE` and `blocker=NONE` on the exact `S06-09` row while preserving `current_state=memory eligibility checks function`
- the master ledger preserves the exact `Design [SELENE_BUILD_SECTION_06.md#L499]` anchor on the exact `S06-09` row
- the master ledger preserves the H28/H29/H30/H31 publication update lines and adds the H32 publication update line
- the build-section row text remains unchanged
- the write set remains docs-only and no file is staged

## Stop Conditions
Stop if repo truth no longer shows the canonical PH1.M `Propose` carrier, the accepted H28/H30 native proof basis, the carried-forward H31 persistence truth, or the exact `S06-09` design anchor.

Stop if the correction attempt requires any source edit, any build-section wording edit, any broader non-`S06-09` frontier re-audit, or any claim broader than this docs-only correction of current repo truth.
