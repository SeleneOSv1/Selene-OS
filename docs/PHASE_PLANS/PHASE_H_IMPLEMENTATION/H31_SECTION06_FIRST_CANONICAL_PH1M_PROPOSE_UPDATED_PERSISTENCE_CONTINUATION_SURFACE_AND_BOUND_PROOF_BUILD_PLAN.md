# H31 Section 06 First Canonical PH1.M Propose Updated Persistence Continuation Surface And Bound Proof Build Plan

## Exact Winner
The exact H31 winner is the first canonical PH1.M propose updated persistence continuation surface-and-proof slice.

The settled post-H30 persistence-continuation frontier audit returned `EXACT_UPDATED_PERSISTENCE_WINNER`.

H29 remains published as the residual hard-rejection trio boundary winner, and H30 remains published as the first canonical proof slice for that bundle.

Stored propose persistence remains already proven within `at_m_19_persist_forwarded_propose_commits_memory_rows` and is carried-forward settled background for H31.

This H31 slice proves updated forwarded propose persistence through `persist_memory_forwarded_outcome(...)` only.

Canonical PH1.M propose carrier proof remains already published background and is not widened by H31.

Denied-consent and sensitive-needs-consent remain already proven within H28 and are out of scope for H31.

Unknown-speaker rejection, privacy-mode rejection, and do-not-store rejection remain already proven within H30 and are out of scope for H31.

Broader S06-09 eligibility closure remains partial after H31.

## Primary Continuation
The current-source continuation files cited by this H31 block are `crates/selene_kernel_contracts/src/ph1m.rs`, `crates/selene_engines/src/ph1m.rs`, and `crates/selene_os/src/ph1m.rs`.

Broader non-`S06-09` rivals remain carried-forward settled background and are not reopened in H31.

## Bounded Proof Surface
The bounded PH1.M persistence test authored by this H31 slice is:
- `at_m_21_persist_forwarded_updated_propose_commits_memory_rows`

This test stays bounded to canonical `MemoryOperation::Propose(...)`, canonical forwarded `MemoryTurnOutput::Propose(propose_resp)`, downstream `persist_memory_forwarded_outcome(...)`, and the already-live `selene_os` persistence path.

## Deterministic Invariants
H31 mirrors current repo truth instead of inventing new semantics:
- updated forwarded propose persistence becomes proven on the downstream `selene_os` path
- stored propose persistence remains already proven within `at_m_19_persist_forwarded_propose_commits_memory_rows`
- canonical PH1.M propose carrier proof remains already published background
- denied-consent remains already proven within H28
- sensitive-needs-consent remains already proven within H28
- unknown-speaker rejection remains already proven within H30
- privacy-mode rejection remains already proven within H30
- do-not-store rejection remains already proven within H30

## Out Of Scope
This H31 slice does not authorize:
- contract edits
- engine-logic edits
- storage production edits
- build-section wording edits
- broader `S06-09` closure
- broader `S06-10` closure
- broader `S06-12` closure
- broader `S06-19` closure
- broader Section 06 completion claims
