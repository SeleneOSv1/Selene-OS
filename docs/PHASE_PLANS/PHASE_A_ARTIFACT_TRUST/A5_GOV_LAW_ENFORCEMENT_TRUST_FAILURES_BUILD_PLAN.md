PHASE A5 — GOV + LAW ENFORCEMENT FOR TRUST FAILURES BUILD PLAN

A) REPO TRUTH CHECK
- `git status --short` result: empty
- current branch: `main`
- HEAD commit: `a4a5911cacd12533ea40a9f551d9d94ecd7a1143`
- exact files reviewed: [A1](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A1_ARTIFACT_TRUST_ARCHITECTURE_REVIEW.md), [A2](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A2_ARTIFACT_IDENTITY_TRUST_CONTRACT_LAYER_BUILD_PLAN.md), [A3](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A3_RUNTIME_VERIFICATION_WIRING_BUILD_PLAN.md), [A4](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A4_PH1J_PROOF_CAPTURE_TRUST_VERIFICATION_OUTCOMES_BUILD_PLAN.md), [CORE_ARCHITECTURE](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md), [SELENE_BUILD_EXECUTION_ORDER](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md), [SECTION_04](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md), [SECTION_09](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md), [SECTION_11](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_11.md), [PH1_GOV](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_GOV.md), [PH1_LAW](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_LAW.md), [PH1_J](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_J.md), [PH1_OS](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_OS.md), [ARTIFACTS_LEDGER_TABLES](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/ARTIFACTS_LEDGER_TABLES.md), [ph1gov.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1gov.rs), [runtime_law.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_law.rs), [runtime_execution.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs), [ph1j.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1j.rs), [runtime_governance.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_governance.rs), [runtime_law.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_law.rs), [ph1j.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1j.rs), [ph1gov.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_engines/src/ph1gov.rs), [ph1j.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1j.rs), [ph1f.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1f.rs)

B) CURRENT GOV / LAW ENFORCEMENT STATE
- GOV trust input surface. CURRENT: raw `artifact_hash_sha256`, `signature_ref`, tenant id, and policy request shape in [ph1gov.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1gov.rs). TARGET: canonical A3/A4 decision-record and proof-linkage consumption only. GAP: GOV has no live `ArtifactTrustDecisionRecord` input.
- LAW trust input surface. CURRENT: strong final posture engine with proof/governance/override/blast-radius inputs in [runtime_law.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_law.rs). TARGET: LAW consumes canonical trust decision records plus proof linkage. GAP: LAW is not yet artifact-trust-decision-centric.
- Current raw hash/signature legacy paths. CURRENT: active in GOV contracts and engine placeholder `sig_` logic in [ph1gov.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_engines/src/ph1gov.rs). TARGET: retired or downgraded. GAP: these paths are still live.
- Current proof-consumer surface. CURRENT: `proof_record_ref` and proof-state oriented, not per-artifact trust-proof-entry oriented. TARGET: GOV/LAW consume `proof_entry_ref` plus `proof_record_ref`. GAP: trust-proof linkage not wired.
- Current quarantine / safe-mode / block posture. CURRENT: already strong in runtime governance and runtime law. TARGET: reuse that machinery with canonical trust-failure inputs. GAP: posture engine exists, trust-failure feed does not.
- Current trust-failure reason handling. CURRENT: generic proof/governance/platform/persistence reasons. TARGET: canonical artifact-trust failure-class handling. GAP: trust vocabulary is not yet authoritative.
- Current policy-version and drift handling. CURRENT: governance and law already track policy version drift. TARGET: bind to `trust_policy_version`, `trust_policy_snapshot_ref`, `trust_set_snapshot_ref`, and proof linkage. GAP: trust-policy pinning not yet enforced.
- Current tenant / blast-radius scoping posture. CURRENT: tenant-aware GOV inputs and LAW blast-radius scope exist. TARGET: trust failures scope cleanly from artifact-local to global. GAP: trust-specific blast-radius rules are not defined.
- Current exception / override posture if any. CURRENT: LAW already supports controlled human override, dual approval, expiry, and decision logging. TARGET: bounded trust-failure exception path with proof/governance logging only. GAP: no trust-specific exception model.
- Current release / exit posture if any. CURRENT: safe-mode exit and runtime release ideas exist. TARGET: quarantine release and safe-mode exit become trust-evidence-driven. GAP: no trust-specific release criteria.
- Current drift risks between GOV and LAW. CURRENT: GOV uses raw fields; LAW uses broader runtime state. TARGET: both consume one canonical meaning. GAP: semantic split risk is real.
- Repo readiness for canonical decision-record consumption. CURRENT: A2/A3/A4 define the right surfaces on paper. TARGET: GOV/LAW consume them directly. GAP: implementation path is ready, live code is not.

C) CANONICAL A5 ENFORCEMENT DESIGN
1. GOV consumes from A3/A4:
- `artifact_trust_state.decision_records`
- per-artifact `proof_entry_ref`
- turn-level `proof_record_ref`
- `trust_policy_snapshot_ref`
- `trust_set_snapshot_ref`
- `verification_basis_fingerprint`
- `negative_verification_result_ref` where present
2. LAW consumes from A3/A4:
- the same canonical decision records
- the same proof linkage
- GOV output posture and blast-radius scoping
- no raw adapter, PH1.OS, hash, signature, or audit fragments
3. Trust failures for governance consumption:
- governance evaluates admission, activation, promotion, rollout, rollback readiness, quarantine readiness, safe-mode eligibility, and blast radius from canonical failure classes only
4. Trust failures for runtime-law consumption:
- law evaluates final runtime posture from canonical failure classes plus proof-required state plus governance output only
5. Proof-linked trust failures flow into GOV/LAW:
- Section 04 decides
- A3 transports
- A4 proves
- A5 consumes
- no alternate path
6. Negative verification outcomes flow into GOV/LAW:
- same as positive outcomes
- same refs
- same proof expectations
- same replay semantics
7. Canonical refs/ids that must be preserved:
- `authority_decision_id`
- `artifact_identity_ref`
- `artifact_trust_binding_ref`
- `trust_policy_snapshot_ref`
- `trust_set_snapshot_ref`
- `verification_basis_fingerprint`
- `negative_verification_result_ref`
- `proof_entry_ref`
- `proof_record_ref`
8. Allowed posture classes:
- `ALLOW_WITH_WARNING`
- `DEGRADE`
- `BLOCK`
- `QUARANTINE`
- `SAFE_MODE`
9. Release / unquarantine / safe-mode-exit conditions:
- represented as governed and law-visible state transitions with proof linkage, evidence refs, tenant/blast-radius scope, and explicit release eligibility
10. What must explicitly NOT be implemented yet:
- new trust verification logic
- new proof transport
- A6 tests execution
- migrations execution
- post-A5 closure logic

D) ENTERPRISE / GLOBAL-STANDARD UPGRADES
- must-have upgrades:
- tenant-aware enforcement on every trust-failure decision path
- blast-radius scoping as first-class GOV and LAW input
- policy-version pinning to trust snapshots and proof linkage
- cross-node drift detection for trust-policy and trust-set mismatches
- no-raw-input enforcement for GOV and LAW
- quarantine release gating with explicit evidence and proof linkage
- safe-mode exit gating with explicit evidence and proof linkage
- exception ledger / reason handling with bounded scope and expiry
- enforcement evidence requirements for any override, release, or downgrade
- one shared GOV/LAW failure vocabulary sourced from A2
- strong optional upgrades:
- dual-control approval for high-risk release from trust-root or cluster-wide quarantine
- cooldown window before safe-mode exit after cluster-trust divergence
- operator acknowledgment refs on exception/release paths
- per-tenant release controls where multi-tenant blast radius is isolated

E) ENFORCEMENT CLASS / RESPONSE MAP
- `HASH_MISMATCH`: GOV `BLOCK`, LAW `BLOCK`; blast radius `artifact-local`; tenant posture localized unless repeated; release eligible only after fresh verified artifact; proof-required completion must succeed before any release.
- `SIGNATURE_INVALID`: GOV `BLOCK`, LAW `QUARANTINE`; blast radius `artifact-local` to `tenant`; release eligible only after new valid decision and proof; no warning-only path.
- `TRUST_ROOT_UNKNOWN`: GOV `BLOCK`, LAW `QUARANTINE`; blast radius `tenant` or `environment`; release only after governed trust-set update and fresh proof.
- `TRUST_ROOT_REVOKED`: GOV `QUARANTINE`, LAW `QUARANTINE` or `SAFE_MODE` if scope exceeds tenant; blast radius `tenant` to `cluster`; release only after governed replacement root and fresh proof.
- `ARTIFACT_REVOKED`: GOV `BLOCK`, LAW `BLOCK`; blast radius `artifact-local`; unquarantine not applicable unless replacement artifact exists.
- `ARTIFACT_EXPIRED`: GOV `BLOCK`, LAW `BLOCK`; blast radius `artifact-local`; release eligible after refreshed certified artifact; no silent degrade for protected execution.
- `CERTIFICATION_INVALID`: GOV `BLOCK`, LAW `BLOCK`; blast radius `artifact-local`; release only after valid certification and proof linkage.
- `LINEAGE_INVALID`: GOV `BLOCK`, LAW `QUARANTINE`; blast radius `tenant`; release only after governed lineage-correct replacement or rollback target.
- `SCOPE_INVALID`: GOV `BLOCK`, LAW `BLOCK`; blast radius `artifact-local` or `tenant`; release only after scope-correct artifact or scope correction with fresh proof.
- `CRYPTO_SUITE_UNSUPPORTED`: GOV `BLOCK`, LAW `QUARANTINE`; blast radius `environment` to `cluster`; release only after governed suite support or artifact replacement.
- `TIME_AUTHORITY_UNAVAILABLE`: GOV `DEGRADE` or `BLOCK` depending protected-action class; LAW `BLOCK` for trust-required protected action; blast radius `environment`; release after authoritative time restored and proof refreshed.
- `VERIFICATION_UNAVAILABLE`: GOV `BLOCK`, LAW `BLOCK`; blast radius `environment`; no executor fallback; release only after verification path restored and proof captured.
- `CACHE_BASIS_INVALID`: GOV `BLOCK`, LAW `BLOCK`; blast radius `artifact-local`; release only after fresh non-cache verification and proof.
- `LEGACY_BLOCKED`: GOV `BLOCK`, LAW `BLOCK`; blast radius `artifact-local`; release only through explicit governed compatibility window if lawful.
- `CLUSTER_TRUST_DIVERGENCE`: GOV `QUARANTINE`, LAW `SAFE_MODE`; blast radius `cluster` or `global`; release only after trust-set convergence and proof-confirmed recovery.
- `HISTORICAL_SNAPSHOT_MISSING`: GOV `BLOCK`, LAW `BLOCK`; blast radius `environment`; release only after archived snapshot continuity restored.
- `proof capture failure for trust-required action`: GOV `BLOCK` or `QUARANTINE` depending failure breadth, LAW `BLOCK`, `QUARANTINE`, or `SAFE_MODE` if proof-chain integrity is systemic; blast radius `artifact-local` to `cluster`; release only after canonical proof capture succeeds.

F) GOV / LAW OWNERSHIP MODEL
- Section 04 trust decision origin: owns first-time trust decision only.
- PH1.J proof linkage origin: owns trust-proof linkage only.
- PH1.GOV governed trust-failure consumption: owns governed response selection, blast-radius containment, release gating, and exception gating from canonical inputs only.
- PH1.LAW final runtime posture consumption: owns final runtime response class only from canonical inputs plus governance outputs.
- Runtime envelope transport: carries decision records, trust state, and proof linkage; it does not reinterpret them.
- Executors/readers: read-only consumers of final posture and carried state.
- Override / exception ownership if relevant: bounded exceptions and overrides remain governed and law-visible; no bypass of Section 04 or PH1.J.

G) TENANT / BLAST-RADIUS / SCOPE MODEL
- `artifact-local` scope: default for single-artifact trust failures.
- `session` scope: only when the failure is confined to one protected turn/session.
- `tenant` scope: use when artifact or trust binding is tenant-bound.
- `environment` scope: use when the issue affects a deployment environment or shared verifier/time/crypto basis.
- `cluster` scope: use when trust-set, proof-chain, or cross-node consistency is compromised.
- `global` scope: use only for root-level or globally shared systemic trust compromise.
- GOV and LAW must escalate to the smallest lawful scope first and widen only when canonical inputs prove shared blast radius.

H) EXCEPTION / OVERRIDE / RELEASE MODEL
- Trust-failure exceptions may exist only as bounded, reason-coded, proof-linked, policy-pinned exceptions.
- Exceptions must never bypass Section 04 verification or PH1.J proof capture.
- Overrides must be logged, scoped, expiring, and consistent with Section 11 controlled human override.
- Quarantine release must be represented as explicit release eligibility plus evidence refs plus proof linkage plus scope.
- Safe-mode exit must be represented as explicit governed and law-visible exit readiness, not an operator-side shortcut.
- What must never be bypassed:
- canonical decision transport
- canonical proof linkage
- tenant/blast-radius scoping
- decision logging

I) POLICY VERSIONING / DRIFT CONTROL MODEL
- GOV and LAW must bind to `trust_policy_version`, `trust_policy_snapshot_ref`, `trust_set_snapshot_ref`, `proof_entry_ref`, `proof_record_ref`, and `verification_basis_fingerprint`.
- If GOV and LAW see different policy/view/proof linkages for the same decision, that is canonical drift and must not be silently tolerated.
- Cross-node or cross-component drift must surface as deterministic governance/law input, not ad hoc logging only.
- Shared A2 failure classes and posture vocabulary prevent divergent interpretation between GOV and LAW.

J) PROOF CONSUMPTION MODEL
- GOV and LAW consume per-artifact `proof_entry_ref` plus turn-level `proof_record_ref`.
- Proof-required actions must require canonical proof linkage, not just canonical decision transport.
- Non-proof-required postures may still read proof linkage when present, but may not replace canonical decision meaning with raw proof fragments.
- If canonical proof linkage exists, raw proof fragments must not be consumed as parallel truth.
- Negative verification outcomes are proof subjects and must be consumed the same way as positive ones.

K) REQUIRED FILE CHANGE MAP
- docs:
- [PH1_GOV.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_GOV.md)
- [PH1_LAW.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_LAW.md)
- [PH1_J.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_J.md)
- [SELENE_BUILD_SECTION_09.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md)
- [SELENE_BUILD_SECTION_11.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_11.md)
- kernel contracts:
- [ph1gov.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1gov.rs)
- [runtime_law.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_law.rs)
- [runtime_execution.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs)
- [ph1j.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1j.rs)
- os/runtime:
- [runtime_governance.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_governance.rs)
- [runtime_law.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_law.rs)
- [ph1j.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1j.rs)
- engines:
- [ph1gov.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_engines/src/ph1gov.rs)
- storage if needed:
- [ph1j.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1j.rs)
- [ph1f.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1f.rs)

L) LEGACY RETIREMENT / REPLACEMENT MAP
- `artifact_hash_sha256`: retire as GOV/LAW trust truth; replace with `ArtifactTrustDecisionRecord` plus `verification_basis_fingerprint` and trust-binding refs.
- `signature_ref`: retire as GOV/LAW trust truth; replace with canonical decision record and proof linkage.
- Placeholder `sig_` logic: retire completely; replace with canonical A3/A4 outputs only.
- Raw proof fragments: downgrade; replace with `proof_entry_ref` and `proof_record_ref`.
- Raw adapter / PH1.OS hints reaching GOV/LAW: block as non-canonical; replace with A3-carried trust state only.
- Any non-canonical exception path: retire or downgrade; replace with bounded, logged, proof-linked exception/override model.

M) STAGING PLAN
1. Additive-first GOV/LAW contract-readiness for canonical decision-record and proof-linkage consumption.
2. Additive GOV consumption path from A3/A4 outputs, without deleting raw legacy fields yet.
3. Additive LAW consumption path from A3/A4 outputs plus GOV outputs, without deleting existing proof/governance posture machinery.
4. Align shared failure vocabulary, posture classes, blast-radius scope, tenant scope, and release eligibility semantics.
5. Introduce bounded exception/override/release representations aligned with Section 11 and proof logging.
6. Deprecate raw hash/signature/raw-hint enforcement paths.
7. Stop before A6.

N) RISKS / DRIFT WARNINGS
- If GOV consumes raw non-canonical trust inputs, A5 fails.
- If LAW consumes raw non-canonical trust inputs, A5 fails.
- If GOV and LAW define different meanings for the same trust failure class, drift starts immediately.
- If either layer bypasses A3/A4 transport, a second trust path is created.
- If separate enforcement vocabularies appear, the stack stops being replay-safe.
- If shadow escalation logic appears outside GOV/LAW canonical inputs, policy control is broken.
- If tenant-local failures are escalated to cluster/global posture without canonical evidence, enterprise scoping is wrong.

O) FINAL APPROVAL PACKAGE
- recommended A5 scope: canonical GOV and LAW consumption and enforcement over A3 decision transport and A4 proof linkage only, including deterministic posture mapping, blast-radius scoping, exception/release gating, and legacy-path retirement
- what must be approved before coding: exact canonical GOV/LAW input set, exact failure-to-posture response map, exact tenant/blast-radius scope model, exact exception/override/release model, exact policy/drift binding, exact proof-consumption model, and exact legacy retirement path
- what must NOT be implemented yet: A6 tests/docs closure, new trust verification logic, new proof transport, Section 04 redesign, PH1.J redesign, or migrations execution
- whether A5 is ready for implementation planning after approval: YES
