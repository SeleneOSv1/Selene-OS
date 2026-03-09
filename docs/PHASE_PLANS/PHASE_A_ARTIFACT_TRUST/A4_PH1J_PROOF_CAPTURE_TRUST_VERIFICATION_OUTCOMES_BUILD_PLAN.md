PHASE A4 — PH1.J PROOF CAPTURE FOR TRUST VERIFICATION OUTCOMES BUILD PLAN

**A) REPO TRUTH CHECK**
- `git status --short` result: empty
- current branch: `main`
- HEAD commit: `babc8ddb49838f989b31fb5366af556500723df2`
- exact files reviewed:
- [A1_ARTIFACT_TRUST_ARCHITECTURE_REVIEW.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A1_ARTIFACT_TRUST_ARCHITECTURE_REVIEW.md)
- [A2_ARTIFACT_IDENTITY_TRUST_CONTRACT_LAYER_BUILD_PLAN.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A2_ARTIFACT_IDENTITY_TRUST_CONTRACT_LAYER_BUILD_PLAN.md)
- [A3_RUNTIME_VERIFICATION_WIRING_BUILD_PLAN.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A3_RUNTIME_VERIFICATION_WIRING_BUILD_PLAN.md)
- [CORE_ARCHITECTURE.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md)
- [SELENE_BUILD_EXECUTION_ORDER.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md)
- [SELENE_BUILD_SECTION_01.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_01.md)
- [SELENE_BUILD_SECTION_03.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md)
- [SELENE_BUILD_SECTION_04.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md)
- [SELENE_BUILD_SECTION_09.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md)
- [SELENE_BUILD_SECTION_11.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_11.md)
- [PH1_J.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_J.md)
- [PH1_GOV.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_GOV.md)
- [PH1_LAW.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_LAW.md)
- [ARTIFACTS_LEDGER_TABLES.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/ARTIFACTS_LEDGER_TABLES.md)
- [ph1j.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1j.rs)
- [runtime_execution.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs)
- [ph1gov.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1gov.rs)
- [runtime_law.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_law.rs)
- [ph1j.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1j.rs)
- [runtime_governance.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_governance.rs)
- [runtime_law.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_law.rs)
- [ph1j.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1j.rs)
- [ph1f.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1f.rs)
- [ph1gov.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_engines/src/ph1gov.rs)
- [app_ingress.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs)
- [ph1os.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1os.rs)
- [lib.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_adapter/src/lib.rs)
- [device_artifact_sync.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/device_artifact_sync.rs)

**B) CURRENT PROOF STATE**
- PH1.J proof schema surface. CURRENT: canonical proof records already exist with `proof_payload_hash`, `previous_event_hash`, `current_event_hash`, signer metadata, verifier metadata, and reason codes in [ph1j.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1j.rs). TARGET: PH1.J must capture artifact-trust outcomes through structured trust-proof content. GAP: no artifact-trust-specific proof surface exists in live contracts.
- Artifact-trust proof support. CURRENT: none. TARGET: PH1.J consumes A3 `ArtifactTrustDecisionRecord` and serializes trust-proof entries. GAP: missing entirely.
- Proof-chain support. CURRENT: strong append-only chain exists in [ph1j.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1j.rs) and [ph1f.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1f.rs). TARGET: reuse that chain without inventing a second ledger. GAP: trust outcomes are not mapped into it yet.
- Proof signer metadata. CURRENT: PH1.J already carries signer identity/key/algorithm metadata. TARGET: preserve that and add artifact-trust basis from A3/A2. GAP: no trust-decision signer/root/snapshot linkage yet.
- Proof payload hashing. CURRENT: canonical payload hashing exists. TARGET: hash structured trust-proof payloads deterministically. GAP: no trust-proof payload definition.
- Proof linkage to runtime envelope. CURRENT: runtime envelope carries `proof_state`, and OS/ingress already emit protected proof from the envelope. TARGET: proof capture must consume `artifact_trust_state` from the envelope and link proof output back into it. GAP: no `artifact_trust_state` exists live.
- Proof linkage to trust decision records. CURRENT: none. TARGET: one structured proof entry per A3 decision record. GAP: missing.
- Proof linkage to trust snapshots. CURRENT: none. TARGET: carry `trust_policy_snapshot_ref` and `trust_set_snapshot_ref`. GAP: missing.
- Proof linkage to negative verification outcomes. CURRENT: generic proof failures exist, but artifact negative-result capture does not. TARGET: failed verification outcomes must still be proved. GAP: missing.
- Proof linkage to multi-artifact turns. CURRENT: proof indexes exist by request and session/turn, but not per artifact. TARGET: deterministic ordered per-artifact proof capture inside the canonical proof path. GAP: undefined.
- GOV/LAW future consumer readiness. CURRENT: later systems can consume `proof_record_ref`, but not artifact-trust-specific proof content. TARGET: future GOV/LAW read canonical trust-proof outputs only. GAP: no trust-proof consumer surface yet.
- Repo drift risks. CURRENT: proof is emitted from OS/ingress using generic proof requests, while GOV still uses raw `artifact_hash_sha256` and `signature_ref`, and engine GOV still has placeholder signature logic. TARGET: PH1.J must consume only A3 decision records. GAP: high drift risk if A4 is not explicit.

**C) CANONICAL A4 PROOF CAPTURE DESIGN**
1. What PH1.J consumes from A3:
- PH1.J consumes only envelope-carried `artifact_trust_state` and its ordered `ArtifactTrustDecisionRecord` set.
- PH1.J does not consume raw adapter hints, PH1.OS posture flags, `artifact_hash_sha256`, `signature_ref`, or client claims as proof truth.
- Section 04 remains the only first-time verifier; PH1.J is a serializer and recorder only.

2. What proof entry/unit is created for each artifact trust decision:
- A4 creates one structured `ArtifactTrustProofEntry` per `ArtifactTrustDecisionRecord`.
- Each entry is keyed by stable decision linkage, not by array position alone.
- The proof entry is the canonical per-artifact unit inside PH1.J proof capture.

3. How multi-artifact proof capture works:
- For a single protected action/turn, PH1.J captures one canonical proof event payload containing an ordered collection of `ArtifactTrustProofEntry` records.
- Ordering must be deterministic and must follow A3 decision-record ordering.
- No flat parallel arrays. No ad hoc correlation by index across unrelated fields.
- Multi-artifact turns remain one proof-capture transaction, with multiple structured entries.

4. What proof basis must be captured:
- `authority_decision_id`
- `artifact_identity_ref`
- `artifact_trust_binding_ref`
- `trust_policy_snapshot_ref`
- `trust_set_snapshot_ref`
- `verification_basis_fingerprint`
- `artifact_verification_result`
- `failure_class` if any
- `negative_verification_result_ref` if any
- `artifact_scope_fingerprint`
- provenance fields from the decision record
- request/session/turn identity
- replay-reconstructable trust basis required by A1/A2

5. What refs/ids/hashes/signer metadata must be carried:
- `proof_record_ref`
- `proof_payload_hash`
- `previous_event_hash`
- `current_event_hash`
- PH1.J signer metadata
- PH1.J verifier metadata ref
- decision-record refs/ids
- trust snapshot refs
- historical trust snapshot refs when historical verification continuity applies
- deterministic reason codes for proof capture context

6. How negative verification outcomes are captured:
- Failed verification outcomes must still produce `ArtifactTrustProofEntry`.
- Negative outcomes are not optional and are not dropped.
- Failure class, negative-result refs, trust snapshot refs, and decision provenance must be captured exactly as received from A3.
- PH1.J must not reinterpret failure as success, degrade, or retry authority.

7. How proof output links back to envelope state:
- After proof capture, envelope `proof_state` is updated through existing PH1.J linkage.
- A4 should add proof linkage back to the trust decision transport, so each `ArtifactTrustDecisionRecord` can reference its proof-entry/proof-record outcome.
- The envelope remains the single runtime transport for trust state plus proof linkage.

8. What downstream stages may read only later:
- Future PH1.GOV may read canonical trust-proof linkage only.
- Future PH1.LAW may read canonical trust-proof linkage only.
- Executors may read proof linkage only as already-carried runtime state; they do not recalculate proof.
- No downstream stage may reconstruct trust meaning from raw fragments if canonical proof linkage exists.

9. What must explicitly NOT be implemented yet:
- No GOV behavior changes.
- No LAW behavior changes.
- No escalation policy.
- No proof-enforcement semantics beyond capture and carriage.
- No migrations execution.
- No tests execution.
- No A5 logic.

**D) PROOF OWNERSHIP MODEL**
- Section 04 trust decision output: owns authoritative `ArtifactTrustDecisionRecord` production before proof capture.
- PH1.J proof input mapping: owns mapping ordered decision records into ordered `ArtifactTrustProofEntry` payload content.
- Storage append-only proof persistence: owns canonical append-only persistence of the resulting proof record and chain hashes.
- Envelope proof linkage: owns attaching proof refs/results back onto envelope-carried state after proof write completes.
- Future GOV/LAW consumers: read canonical proof linkage only; they do not redefine proof semantics.
- Executors/readers: read only; no proof regeneration, no trust reinterpretation.

**E) REQUIRED FILE CHANGE MAP**
- docs:
- [PH1_J.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_J.md)
- [PH1_GOV.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_GOV.md)
- [PH1_LAW.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_LAW.md)
- [SELENE_BUILD_SECTION_09.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md)
- [SELENE_BUILD_SECTION_11.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_11.md)
- [ARTIFACTS_LEDGER_TABLES.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/ARTIFACTS_LEDGER_TABLES.md)
- kernel contracts:
- [ph1j.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1j.rs)
- [runtime_execution.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs)
- [ph1gov.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1gov.rs)
- [runtime_law.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_law.rs)
- os/runtime:
- [ph1j.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1j.rs)
- [app_ingress.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs)
- [ph1os.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1os.rs)
- [runtime_governance.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_governance.rs)
- [runtime_law.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_law.rs)
- storage:
- [ph1j.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1j.rs)
- [ph1f.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1f.rs)
- engines if needed:
- [ph1gov.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_engines/src/ph1gov.rs)

**F) PROOF ENTRY / TRANSPORT DEPRECATION MAP**
- Generic protected-proof payloads that omit artifact-trust decision content must be retired for artifact-trust-governed protected actions.
- Flat parallel proof arrays are explicitly non-canonical and must not be introduced.
- Raw `artifact_hash_sha256` and `signature_ref` as proof truth must be downgraded to legacy compatibility inputs only.
- `capture_artifact_trust_verified` must remain non-authoritative upstream posture only and must not be used as PH1.J proof truth.
- Hash-only device apply signals remain operational-only and must not be elevated into trust-proof evidence.
- Legacy audit-only paths are non-canonical for protected artifact trust; the canonical PH1.J proof ledger remains the target.

**G) STAGING PLAN**
1. Freeze the A4 input boundary: PH1.J consumes only A3 `artifact_trust_state` / `ArtifactTrustDecisionRecord`.
2. Add additive kernel-contract proof-entry surfaces and proof linkage fields without removing existing proof types.
3. Add additive storage support for structured trust-proof content within the canonical proof ledger path.
4. Add PH1.J runtime mapping from ordered decision records to ordered `ArtifactTrustProofEntry` payloads.
5. Add proof linkage back into envelope-carried trust state and existing `proof_state`.
6. Make future GOV/LAW consumers compile-through compatible with canonical trust-proof linkage only, without changing behavior.
7. Stop before A5.

**H) RISKS / DRIFT WARNINGS**
- If PH1.J consumes raw adapter, PH1.OS, or GOV fields instead of A3 decision records, A4 invents a second trust transport.
- If multi-artifact proof is flattened into parallel arrays, proof correlation becomes unsafe.
- If `trust_policy_snapshot_ref` or `trust_set_snapshot_ref` are omitted, replay becomes unverifiable.
- If failed verification decisions are not captured, proof becomes biased and incomplete.
- If PH1.J re-verifies or “corrects” trust decisions, Section 04 ownership is broken.
- If GOV/LAW later consume raw proof fragments instead of canonical proof entries/refs, drift resumes immediately.

**I) FINAL APPROVAL PACKAGE**
- recommended A4 scope: PH1.J consumes A3-carried trust decision records, serializes structured per-artifact trust-proof entries into the existing canonical proof chain, links proof output back to envelope-carried trust state, and stops before GOV/LAW behavior
- what must be approved before coding: exact proof-entry content, exact multi-artifact proof packaging model, exact proof linkage back into envelope/decision records, exact negative-result capture rule, exact trust snapshot refs required, exact signer/verifier metadata carried, and exact deprecation posture for raw legacy trust/proof fields
- what must NOT be implemented yet: GOV behavior, LAW behavior, escalation mapping, proof-enforcement policy, migrations execution, tests execution, or any A5 logic
- whether A4 is ready for implementation planning after approval: YES
