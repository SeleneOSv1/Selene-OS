Selene Phase A
Artifact Trust-Root and Verification Architecture

Purpose of Phase A

Phase A establishes the Artifact Trust-Root System for Selene.

This system guarantees that:

• all executable artifacts are cryptographically verifiable
• no untrusted artifact can enter runtime execution
• artifact identity is deterministic and auditable
• verification outcomes are provable and replayable
• trust failures propagate through Authority, Governance, and Law layers

Artifacts include at minimum:

• simulation definitions
• simulation workflows
• runtime policy bundles
• builder outputs
• deployment packages
• wake artifacts
• model artifacts
• learning promotion artifacts

Phase A therefore establishes the cryptographic trust foundation for the entire Selene runtime.

A1 — Artifact Trust-Root Design Review

Objective

Define the canonical Artifact Trust-Root model that all runtime artifacts must satisfy.

Without a trust root, artifacts could be injected or mutated silently.

The trust root guarantees:

• artifact authenticity
• artifact integrity
• artifact lineage
• artifact certification status
• artifact revocation safety

Artifact Trust Model

Every artifact must have a deterministic identity.

Canonical artifact identity fields:

artifact_id
artifact_type
artifact_version
artifact_hash
artifact_signature
artifact_signer_identity
artifact_certification_state
artifact_created_at
artifact_lineage_parent
artifact_lineage_root
artifact_schema_version

Artifact identity must be deterministic across runtime nodes.

A1 Architecture Boundary

A1 defines architecture law only.

A1 is allowed to define:

• authority ownership boundaries
• artifact trust-root hierarchy
• artifact lifecycle stages
• artifact trust model
• artifact verification order
• artifact scope law
• verification cache law
• proof linkage requirements
• deterministic failure classes
• failure escalation posture through Authority, Governance, and Law
• canonical no-bypass trust-path rules

A1 must not define:

• exact structs
• exact enums
• exact DB schema
• exact wire packets
• exact storage tables
• exact Rust type definitions
• exact runtime/storage contract signatures
• implementation-specific algorithm details

A2 will define:

• exact structs
• exact enums
• exact DB schema
• exact wire contracts
• typed runtime contracts
• typed storage contracts

A1 therefore establishes the canonical law and system boundaries.

A2 and later phases implement those boundaries in typed and executable form.

Canonical Verifier Ownership Law

Section 04 Authority Layer is the one and only canonical owner of first-time authoritative artifact trust verification.

Canonical ownership sequence:

Section 03 ingress
→ PH1.OS posture normalization
→ Section 04 Authority verification
→ PH1.J proof capture
→ PH1.GOV governance evaluation
→ PH1.LAW final runtime posture
→ protected executor or consumer path

Ownership rules:

• Section 03 ingress may only normalize and carry artifact references, claims, receipts, and scope inputs into the Runtime Execution Envelope
• PH1.OS may only contribute device, platform, receipt, attestation, compatibility, and environment posture
• Section 04 Authority Layer must perform first-time authoritative artifact trust verification
• PH1.J must prove the verification basis and decision context after authoritative verification occurs
• PH1.GOV must consume verified artifact trust state for governed activation, promotion, deployment, rollback, and release decisions
• PH1.LAW must consume governed runtime inputs and produce the final runtime posture
• executors, simulation paths, builder, learning, self-heal, and device sync paths may only consume verified trust decisions

Hard rules:

• no other subsystem may perform first-time authoritative artifact trust verification
• no adapter, client, device worker, executor, or consumer path may upgrade an unverified artifact into trusted state locally
• no subsystem may act as a co-owner of authoritative artifact verification beside Section 04
• no parallel verifier may be introduced for wake, builder, simulation, voice, device sync, or any other artifact class

Artifact Classes

Artifacts must be classified by trust scope.

Example artifact classes:

SIMULATION_DEFINITION
SIMULATION_WORKFLOW
WAKE_MODEL
VOICE_IDENTITY_MODEL
MEMORY_SCHEMA
POLICY_BUNDLE
BUILDER_OUTPUT
DEPLOYMENT_PACKAGE
LEARNING_PROMOTION_ARTIFACT

Each artifact class may define additional verification rules.

Artifact Certification States

Artifacts must expose certification state.

Example states:

DRAFT
TEST_CERTIFIED
RUNTIME_CERTIFIED
REVOKED
EXPIRED

Only RUNTIME_CERTIFIED artifacts may enter protected runtime execution.

Artifact Trust Chain

Artifacts must maintain lineage.

Each artifact must record:

artifact_lineage_parent
artifact_lineage_root

This allows:

• rollback verification
• deployment ancestry tracking
• artifact replacement safety

Artifact Trust Anchors

Selene must define root signing identities.

Example trust anchors:

SELENE_ROOT_CA
SELENE_RUNTIME_SIGNER
SELENE_BUILDER_SIGNER
SELENE_LEARNING_SIGNER

These anchors must exist in a Trust Anchor Registry.

Trust-Root Hierarchy

Selene must define a strict trust hierarchy.

Required architecture levels:

• Root trust authority
• Intermediate or domain trust authorities
• Artifact-class signers
• Runtime verifier trust set
• Cluster-visible trust-root state

The hierarchy must operate as follows:

• artifact-class signers chain upward to an approved intermediate or domain trust authority
• intermediate or domain trust authorities chain upward to the root trust authority
• runtime verification must validate the full chain, not only the artifact signer
• runtime verifier trust sets must be derived from cloud-authoritative trust-root state
• trust-root state must be visible across the runtime cluster

No flat signer model is allowed.

No ad hoc signer sprawl is allowed.

Artifacts must not be accepted merely because a signer is locally known.

Artifact verification must chain upward through the approved hierarchy before the artifact may be considered trusted.

Trust Policy Snapshot and Trust-Set Version Law

Every authoritative verification decision must bind to one governed trust snapshot.

Required architecture-level snapshot references:

• trust_policy_version
• trust_set_version

Architecture law:

• trust_policy_version defines the governed verification policy basis in effect at decision time
• trust_set_version defines the monotonic trust-root and signer-set snapshot used at decision time
• authoritative verification must not mix inputs from multiple trust snapshots in one decision
• every authoritative verification decision must record the exact trust_policy_version and trust_set_version used
• the same trust_policy_version and trust_set_version must flow into:
  • Runtime Execution Envelope
  • PH1.J proof basis
  • verification cache basis
  • offline replay and independent verification

Cluster uncertainty around the active trust_set_version must not be treated as a warning-only condition.

If trust-set convergence is uncertain:

• protected execution must fail closed by default
• degraded posture is allowed only where explicit governance and law permit it

Cryptographic Suite Version Law

Artifact trust verification must bind to a governed cryptographic suite basis.

Required architecture-level concept:

• crypto_suite_version

Architecture law:

• authoritative verification must record the cryptographic suite basis used for the decision
• proof and replay must preserve the crypto_suite_version used at decision time
• future cryptographic upgrades must not silently reinterpret historical verification records
• future cryptographic upgrades must preserve historical verification capability through governed compatibility and archived verification context
• inability to verify historical proof solely because the cryptographic basis was not preserved is an architecture failure

Key Rotation, Overlap, and Revocation Law

Signer rotation must be explicitly governed.

Architecture requirements:

• rotation windows must be defined
• overlap acceptance rules must be defined
• cutover rules must be deterministic
• revocation effects on future verification must be immediate once cluster convergence is reached
• cluster nodes must converge on the same active signer and trust-root set

During authorized rotation windows:

• both retiring and successor signers may be accepted only if overlap policy explicitly allows it
• overlap acceptance must be time-bounded
• overlap acceptance must be replayable and auditable

At cutover:

• the successor signer set becomes authoritative
• retired signers must no longer authorize new artifacts unless policy explicitly allows a governed overlap interval

For revocation:

• revoked signers and revoked trust roots must not authorize future verification success
• revocation must invalidate future trust decisions for affected artifacts
• historical proof created before revocation remains historical evidence but does not authorize future execution by itself
• proof validation must preserve the signer state observed at decision time

Cluster convergence is mandatory during rotation and revocation.

If signer-set consensus is uncertain across the cluster:

• protected execution must not silently continue
• runtime must enter deterministic degraded or block posture according to governance and law

Historical Proof Verification Law

Historical trust verification and forward trust authorization are distinct.

Architecture law:

• retired signers may remain valid for historical proof verification only through archived trust snapshots that match the decision context
• retired signers must not automatically remain valid for future execution
• historical proof verification must preserve the trust snapshot, signer state, and cryptographic suite basis observed at decision time
• forward execution authority must be evaluated against the current governed trust snapshot, not the historical one

Clock and Time Authority Law

Expiry, freshness, rotation, overlap, and revocation timing must rely on governed time authority.

Architecture law:

• authoritative verification must use a governed time source for expiry, freshness, and rotation checks
• allowed clock skew posture must be explicitly bounded by system law
• device-local time must not become authoritative for protected trust decisions
• if trusted time is unavailable and freshness or expiry cannot be established, protected execution must block by default
• degraded behavior when trusted time is unavailable is lawful only where explicit governance and law permit it for non-protected use
• proof and replay must preserve the time authority basis used at decision time
• proof and replay must preserve any governed degraded posture caused by time uncertainty

Artifact Lifecycle Stages

Artifact trust controls apply across distinct lifecycle stages.

Canonical stages:

• ingest
• identity parse
• verification
• certification
• publication
• distribution
• delivery
• installation
• apply request
• apply receipt
• activation
• runtime use
• replacement
• rollback
• revocation
• retirement

Architecture law:

• ingest and storage do not imply trust
• identity parse does not imply verification success
• verification does not imply certification
• certification does not imply activation
• publication does not imply distribution
• distribution and delivery do not imply installation
• installation does not imply authoritative verification success
• apply request does not imply apply approval
• apply receipt does not imply activation
• activation does not replace runtime-use checks where runtime validation is still required
• replacement and rollback must remain lineage-aware
• revocation and retirement are not the same state

Distribution, installation, apply request, and apply receipt must not be conflated with authoritative verification or activation.

Activation and runtime use must never be conflated with ingest or storage.

Legacy Artifact Compatibility Posture

Legacy artifact handling must be explicit and policy-governed.

Compatibility policy classes:

• STRICT_NO_LEGACY
• GOVERNED_COMPATIBILITY_WINDOW
• LEGACY_BLOCKED

Architecture law:

• protected execution defaults to STRICT_NO_LEGACY
• legacy artifacts must never be silently trusted
• legacy compatibility requires explicit governed policy
• any compatibility window must be policy-driven
• any compatibility window must be time-bounded
• any compatibility decision must be replayable
• any compatibility decision must be auditable
• protected execution must not accept legacy artifacts outside approved policy

Legacy acceptance is therefore exceptional, governed, and temporary.

Artifact Scope Law

Artifact validity is scope-bound.

Canonical scope dimensions include:

• tenant scope
• environment scope
• runtime scope
• platform scope
• rollout scope
• feature-flag scope
• identity scope where required

Architecture law:

• an artifact is valid only inside the scopes for which it was certified
• scope validation is mandatory during verification
• scope validity must not bleed across incompatible environments, runtimes, or tenants
• rollout scope must constrain artifact use during staged activation
• feature-flag scope must not be used as an authority bypass
• identity-scoped artifacts must not be reused outside their valid identity boundary

Strict Verification-Cache Law

Verification caching is allowed only under explicit law.

Verification outcomes that may be cached:

• successful verification outcomes
• governed degraded verification outcomes where policy explicitly allows cache reuse

Verification outcomes that must not be silently reused:

• revoked outcomes
• unknown-trust-root outcomes
• lineage-invalid outcomes
• scope-invalid outcomes

Architecture requirements:

• cached verification must have a defined freshness window
• cache freshness requirements must be enforced before reuse
• revocation must invalidate affected cached verification outcomes
• trust-root rotation must invalidate affected cached verification outcomes when policy requires fresh verification against the new set
• cluster invalidation must converge across nodes
• proof-required actions must record whether verification was fresh or cache-derived
• cache basis must bind the applicable trust_policy_version, trust_set_version, crypto_suite_version, and validated scope basis
• cluster uncertainty around trust_set_version must fail closed or degrade only where explicit governance and law allow it
• cached verification must never become an authority shortcut

Cached verification is evidence reuse only.

Cached verification is not a substitute for the canonical trust path.

Artifact Storage Boundary

Artifacts must be stored in cloud-authoritative artifact storage.

Rules:

• clients may download artifacts but cannot sign or certify them
• artifacts cannot become authoritative outside cloud runtime
• artifact signatures must be verified before runtime use
• devices may cache artifacts but device-local cache state is never trust authority
• PH1.OS may carry trust inputs but PH1.OS is not an artifact trust authority
• proof records do not replace artifact verification
• publication, distribution, installation, apply request, and apply receipt are operational stages only and do not establish authoritative trust or activation

No Parallel Trust-Path Law

Selene must operate one canonical artifact trust-root path only.

The following are forbidden as independent trust paths:

• wake-only trust path
• builder-only trust path
• simulation-only trust path
• voice-only trust path
• device-local trust path
• PH1.OS trust authority path
• proof-only substitute for verification

All protected artifact trust decisions must resolve through the same canonical trust-root model.

A2 — Artifact Identity + Trust Contract Layer

Objective

Define the canonical Artifact Identity Contract and Trust Verification Contract used by the runtime.

This contract ensures artifacts are validated before runtime execution.

A2 Preview / Non-Approved Typed Surface

The preview labels below are informational scaffolding for A2 only.

They are not approved A1 typed contracts.

They must not be treated as final struct names, enum names, packet names, database schema, or storage/runtime interfaces.

Artifact Identity Object

Canonical artifact identity structure:

ArtifactIdentity

Fields:

artifact_id
artifact_type
artifact_version
artifact_hash
artifact_signature
artifact_signer_identity
artifact_schema_version
artifact_certification_state
artifact_created_at
artifact_lineage_parent
artifact_lineage_root

Artifact Verification Result

Verification outcomes must be deterministic.

Example verification result structure:

ArtifactVerificationResult

Fields:

artifact_id
verification_result
verification_reason_code
signature_valid
hash_valid
trust_anchor_valid
certification_state_valid
lineage_valid
verification_timestamp
verification_node_id

Verification Result Classes

Artifact verification must produce deterministic outcomes.

Example results:

VERIFIED
CERTIFICATION_INVALID
SIGNATURE_INVALID
HASH_MISMATCH
TRUST_ANCHOR_UNKNOWN
ARTIFACT_REVOKED
ARTIFACT_EXPIRED
SCHEMA_INCOMPATIBLE

Artifact Revocation Model

Revocation must be explicit.

Revocation mechanisms:

artifact_revocation_event
artifact_revocation_reason
artifact_revocation_timestamp
artifact_revoked_by

Revoked artifacts must be blocked by runtime.

A1 to A2 Boundary Reminder

The items above define required architecture-level concepts for later contract work.

The labels `ArtifactIdentity` and `ArtifactVerificationResult` above are preview markers only.

They do not define final structs, enums, DB tables, packet schemas, or typed storage/runtime interfaces.

Those typed definitions belong to A2.

A3 — Runtime Verification Wiring

Objective

Wire artifact verification into runtime entry paths.

Canonical runtime ownership path:

Section 03 ingress carries artifact references, claims, receipts, and scope inputs.

PH1.OS normalizes device, platform, receipt, attestation, compatibility, and environment posture.

Section 04 Authority Layer performs authoritative artifact trust verification.

PH1.J captures proof of the verification basis and outcome.

PH1.GOV evaluates governed activation, promotion, deployment, replacement, rollback, and release posture using verified trust state.

PH1.LAW produces the final runtime posture.

Protected executors and consumers may proceed only by consuming already verified trust decisions.

Artifact Verification Points

Artifact verification must occur at:

simulation load
artifact activation
builder deployment
learning promotion
artifact apply request
runtime model load

Verification Flow

Artifact Verification Flow

artifact_load_request
→ artifact_identity_parse
→ artifact_hash_validation
→ artifact_signature_validation
→ trust_anchor_validation
→ trust_policy_snapshot_bind
→ crypto_suite_basis_bind
→ certification_state_validation
→ artifact_lineage_validation
→ artifact_scope_validation
→ time_authority_validation
→ verification_outcome

Runtime Execution Envelope Integration

Verification outcomes must be attached to the Runtime Execution Envelope.

Example envelope fields:

artifact_id
artifact_verification_result
artifact_certification_state
artifact_signer_identity
artifact_verification_timestamp
artifact_trust_policy_version
artifact_trust_set_version
artifact_crypto_suite_version
artifact_time_authority_basis

Verification Cache Posture

If cached verification is used:

• the verification basis must still be lawful
• freshness window compliance must be proven
• revocation and trust-root state must be re-checked according to policy
• proof-required actions must disclose cache-derived verification posture
• cache-derived reuse must remain bound to trust_policy_version, trust_set_version, crypto_suite_version, and validated scope basis

Cached verification must not create a silent bypass around fresh trust evaluation requirements.

Runtime Blocking Rules

If artifact verification fails:

execution must stop
runtime must emit deterministic failure class
authority layer must record rejection

A4 — PH1.J Proof Capture for Artifact Verification

Objective

Ensure artifact verification outcomes are cryptographically recorded.

PH1.J must capture verification results when artifacts are used in protected execution.

Proof Record Extensions

Proof records must include:

artifact_id
artifact_hash
artifact_signature
artifact_signer_identity
artifact_certification_state
artifact_verification_result
artifact_verification_timestamp
artifact_verification_node_id

PH1.J Trust-Linkage Minimums

Architecture law requires proof to capture at minimum:

• artifact verification basis
• signer identity used
• trust-root id and version used
• trust_policy_version and trust_set_version used
• crypto_suite_version used
• certification state observed
• revocation state observed at decision time
• fresh versus cache-derived verification outcome
• lineage validation posture
• scope validation posture
• time authority basis observed
• archived trust snapshot reference where historical proof verification depends on retired signer state

This is architecture law only.

Exact proof schema is defined later.

Proof Event Types

Example events:

ArtifactVerificationSucceeded
ArtifactVerificationFailed
ArtifactRevoked
ArtifactCertificationChanged
ArtifactActivationBlocked

Proof Chain Integration

Verification events must enter the append-only proof ledger.

Proof payload must include:

proof_payload_hash
previous_event_hash
current_event_hash

A5 — Governance and Law Enforcement

Objective

Ensure trust failures propagate through runtime governance and law layers.

Authority Layer Integration

Authority must block protected actions when artifact verification fails.

Example rule:

if artifact_verification_result != VERIFIED
→ block execution

Runtime Governance Integration

Governance must monitor trust failures.

Example governance signals:

artifact_verification_failures_total
artifact_revocation_events
artifact_signature_failures

Runtime Law Enforcement

PH1.LAW must treat artifact failures as law inputs.

Example law inputs:

artifact_signature_invalid
artifact_certification_invalid
artifact_revoked

Possible law responses:

BLOCK
QUARANTINE
SAFE_MODE
DEGRADE

Artifact Trust Failure Escalation Matrix

The following failure classes require deterministic posture:

HASH_MISMATCH
→ BLOCK

SIGNATURE_INVALID
→ BLOCK

TRUST_ROOT_UNKNOWN
→ BLOCK

TRUST_ROOT_REVOKED
→ QUARANTINE

ARTIFACT_REVOKED
→ BLOCK

ARTIFACT_EXPIRED
→ BLOCK

LINEAGE_INVALID
→ BLOCK

SCOPE_INVALID
→ BLOCK

VERIFICATION_UNAVAILABLE
→ DEGRADE only where explicitly lawful for non-protected usage
→ BLOCK for protected execution by default

CLUSTER_TRUST_DIVERGENCE
→ SAFE_MODE or QUARANTINE

Degrade posture is lawful only where explicit policy allows it.

Protected execution must fail closed when no such lawful degrade path exists.

A6 — Tests, Documentation, Verification

Objective

Ensure artifact trust architecture is verifiable.

Test Coverage

Required test classes:

artifact_identity_parsing_tests
artifact_hash_validation_tests
artifact_signature_validation_tests
artifact_certification_state_tests
artifact_revocation_tests
artifact_lineage_tests
artifact_runtime_block_tests

Integration Tests

Runtime tests must verify:

simulation artifact verification
builder artifact verification
learning artifact verification
artifact failure propagation

Proof Verification Tests

Verify PH1.J records verification events.

Tests include:

proof_chain_integrity
verification_event_logging
artifact_revocation_proof_record

Governance and Law Tests

Test that:

artifact failures propagate to PH1.GOV
artifact failures propagate to PH1.LAW
runtime blocks unsafe artifact execution

Documentation Updates

Required docs:

artifact trust root design doc
artifact verification contract doc
artifact revocation policy doc
artifact certification workflow doc

Phase A Completion Criteria

Phase A is complete when:

artifact trust root exists
artifact identity contract exists
runtime verification wiring exists
PH1.J captures verification proof
governance reacts to artifact failures
law engine blocks unsafe artifacts
tests verify trust behavior

Phase A Result

After Phase A:

Selene runtime gains a cryptographic artifact trust foundation.

This ensures:

• only trusted artifacts execute
• artifact lineage is traceable
• runtime execution is provable
• artifact failures trigger runtime law enforcement

If you want, next I can also produce the Codex prompt for Step 2 (repo truth sweep for Phase A) so Codex checks exactly the correct files before writing the implementation plan.
