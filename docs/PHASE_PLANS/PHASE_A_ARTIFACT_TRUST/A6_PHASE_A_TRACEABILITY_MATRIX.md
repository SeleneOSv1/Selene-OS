# Phase A Traceability Matrix

| traceability_id | phase | canonical requirement | implementation evidence | linked_test_ids | linked_evidence_ids | status |
| --- | --- | --- | --- | --- | --- | --- |
| A1-REQ-SECTION04-ONLY | A1 | Section 04 remains the only first-time authoritative verifier; no client, adapter, PH1.OS, GOV, or LAW authority path may replace it | `crates/selene_kernel_contracts/src/ph1art.rs`, `crates/selene_kernel_contracts/src/runtime_execution.rs`, `crates/selene_os/src/runtime_governance.rs`, `crates/selene_os/src/runtime_law.rs` | `TEST-A1-ROOT-01`, `TEST-A3-TRANSPORT-01`, `TEST-A5-GOV-01`, `TEST-A5-LAW-01` | `EVID-PHASEA-CONTRACT`, `EVID-PHASEA-TRANSPORT`, `EVID-PHASEA-ENFORCEMENT` | PASS |
| A1-REQ-TRUSTROOT-FOUNDATION | A1 | Canonical trust-root registry foundation exists as append-only trust-root truth | `crates/selene_kernel_contracts/src/ph1art.rs`, `crates/selene_storage/src/ph1f.rs` | `TEST-A1-ROOT-01`, `TEST-A1-ROOT-02` | `EVID-PHASEA-CONTRACT` | PASS |
| A2-REQ-CANONICAL-CONTRACTS | A2 | Artifact identity, trust binding, snapshots, basis fingerprint, decision record, and proof entry are canonical typed surfaces | `crates/selene_kernel_contracts/src/ph1art.rs` | `TEST-A2-CONTRACT-01` | `EVID-PHASEA-CONTRACT` | PASS |
| A3-REQ-CANONICAL-TRANSPORT | A3 | `artifact_trust_state` is the only canonical runtime trust transport and is read-only downstream | `crates/selene_kernel_contracts/src/runtime_execution.rs` | `TEST-A3-TRANSPORT-01` | `EVID-PHASEA-TRANSPORT` | PASS |
| A4-REQ-CANONICAL-PROOF | A4 | PH1.J consumes A3 decision transport only and emits ordered per-artifact proof entries plus turn-level proof linkage | `crates/selene_kernel_contracts/src/ph1j.rs`, `crates/selene_os/src/ph1j.rs`, `crates/selene_storage/src/ph1f.rs` | `TEST-A4-PROOF-01`, `TEST-A4-PROOF-02` | `EVID-PHASEA-PROOF` | PASS |
| A5-REQ-GOV-CANONICAL | A5 | GOV consumes canonical trust decision/proof linkage only and blocks missing or incomplete canonical evidence | `crates/selene_os/src/runtime_governance.rs` | `TEST-A5-GOV-01`, `TEST-A5-GOV-02`, `TEST-A5-GOV-03` | `EVID-PHASEA-ENFORCEMENT` | PASS |
| A5-REQ-LAW-CANONICAL | A5 | LAW consumes canonical trust decision/proof linkage only and enforces deterministic artifact-trust posture mapping | `crates/selene_os/src/runtime_law.rs` | `TEST-A5-LAW-01`, `TEST-A5-LAW-02`, `TEST-A5-LAW-03` | `EVID-PHASEA-ENFORCEMENT` | PASS |
| A5-REQ-PROOF-ENTRY-PRECEDENCE | A5 | `proof_entry_ref` is canonical per-artifact proof linkage; `proof_record_ref` is not an interchangeable substitute | `crates/selene_os/src/runtime_governance.rs`, `crates/selene_os/src/runtime_law.rs` | `TEST-A5-GOV-03`, `TEST-A5-LAW-03` | `EVID-PHASEA-PROOF`, `EVID-PHASEA-ENFORCEMENT` | PASS |
| A6-REQ-DOCS-ALIGNMENT | A6 | Phase plans, build sections, DB wiring docs, coverage/tracker docs, and closure evidence must not contradict one another | `docs/DB_WIRING/*`, `docs/COVERAGE_MATRIX.md`, `docs/33_ENGINE_REVIEW_TRACKER.md`, `docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md` | `SWEEP-DOCS-ALIGNMENT` | `EVID-PHASEA-DOCS` | PASS |
| A6-REQ-CLOSURE-EVIDENCE | A6 | Closure evidence must be reviewable from a canonical manifest, evidence pack, and residual risk register | `A6_PHASE_A_CLOSURE_EVIDENCE_MANIFEST.md`, `A6_PHASE_A_CLOSURE_EVIDENCE_PACK.md`, `A6_PHASE_A_RESIDUAL_RISK_REGISTER.md` | `SWEEP-ACCEPTANCE-CLOSURE` | `EVID-PHASEA-MANIFEST` | PASS |

## Linked Test IDs

- `TEST-A1-ROOT-01` -> `cargo test -p selene_kernel_contracts artifact_trust_root_registry --lib`
- `TEST-A1-ROOT-02` -> `cargo test -p selene_storage --test db_wiring_artifacts_ledger_tables`
- `TEST-A2-CONTRACT-01` -> `cargo test -p selene_kernel_contracts --lib ph1art::tests::`
- `TEST-A3-TRANSPORT-01` -> `cargo test -p selene_kernel_contracts --lib at_runtime_execution_`
- `TEST-A4-PROOF-01` -> `cargo test -p selene_kernel_contracts --lib ph1j::`
- `TEST-A4-PROOF-02` -> `cargo test -p selene_os --lib at_j_runtime_`
- `TEST-A5-GOV-01` -> `cargo test -p selene_os --lib at_runtime_gov_`
- `TEST-A5-GOV-02` -> `at_runtime_gov_09_artifact_activation_requires_proof_linkage_when_hint_demands_it`
- `TEST-A5-GOV-03` -> `at_runtime_gov_10_turn_level_proof_without_per_artifact_entry_still_blocks`
- `TEST-A5-LAW-01` -> `cargo test -p selene_os --lib at_runtime_law_`
- `TEST-A5-LAW-02` -> `at_runtime_law_12_artifact_authority_requires_canonical_proof_linkage`
- `TEST-A5-LAW-03` -> `at_runtime_law_13_turn_level_proof_without_per_artifact_entry_still_blocks`

## Linked Evidence IDs

- `EVID-PHASEA-CONTRACT` -> canonical contract and storage foundation evidence
- `EVID-PHASEA-TRANSPORT` -> A3 runtime transport evidence
- `EVID-PHASEA-PROOF` -> A4 proof-entry and proof-linkage evidence
- `EVID-PHASEA-ENFORCEMENT` -> A5 GOV/LAW canonical consumption evidence
- `EVID-PHASEA-DOCS` -> docs alignment evidence
- `EVID-PHASEA-MANIFEST` -> closure manifest evidence
