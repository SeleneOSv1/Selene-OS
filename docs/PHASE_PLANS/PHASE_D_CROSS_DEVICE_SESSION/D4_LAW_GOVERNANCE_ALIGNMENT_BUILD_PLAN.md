PHASE D4 — LAW / GOVERNANCE ALIGNMENT BUILD PLAN

A) REPO TRUTH CHECK
- `git status --short`: empty
- current branch: `main`
- repo-truth snapshot HEAD at review start: `dbd464c2e56027ae7bf1b481d1aa40e470e1c597`
- snapshot semantics: this value records the repo state reviewed at the start of this pass; it is not required to equal the later commit hash that records this document update.
- exact files reviewed:
  - `docs/CORE_ARCHITECTURE.md`
  - `docs/SELENE_BUILD_EXECUTION_ORDER.md`
  - `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`
  - `docs/PHASE_PLANS/PHASE_D_CROSS_DEVICE_SESSION/D1_CROSS_DEVICE_SESSION_CONSISTENCY_REVIEW.md`
  - `docs/PHASE_PLANS/PHASE_D_CROSS_DEVICE_SESSION/D2_ATTACH_RECOVER_DETACH_CONTRACT_FIXES_BUILD_PLAN.md`
  - `docs/PHASE_PLANS/PHASE_D_CROSS_DEVICE_SESSION/D3_RUNTIME_AND_PERSISTENCE_WIRING_BUILD_PLAN.md`
  - `docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C4_STORAGE_PROOF_LAW_INTEGRATION_BUILD_PLAN.md`
  - `docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C5_TESTS_DOCS_VERIFICATION_BUILD_PLAN.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_01.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_02.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_05.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_07.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_08.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_11.md`
  - `docs/DB_WIRING/PH1_L.md`
  - `docs/DB_WIRING/PH1_LEASE.md`
  - `docs/DB_WIRING/PH1_MULTI.md`
  - `docs/DB_WIRING/PH1_OS.md`
  - `docs/DB_WIRING/PH1_GOV.md`
  - `docs/DB_WIRING/PH1_LAW.md`
  - `docs/DB_WIRING/PH1_J.md`
  - `crates/selene_kernel_contracts/src/ph1l.rs`
  - `crates/selene_kernel_contracts/src/ph1lease.rs`
  - `crates/selene_kernel_contracts/src/ph1multi.rs`
  - `crates/selene_kernel_contracts/src/ph1os.rs`
  - `crates/selene_kernel_contracts/src/ph1_voice_id.rs`
  - `crates/selene_kernel_contracts/src/runtime_execution.rs`
  - `crates/selene_kernel_contracts/src/runtime_governance.rs`
  - `crates/selene_kernel_contracts/src/runtime_law.rs`
  - `crates/selene_storage/src/repo.rs`
  - `crates/selene_storage/src/ph1f.rs`
  - `crates/selene_storage/src/ph1j.rs`
  - `crates/selene_os/src/device_artifact_sync.rs`

B) PURPOSE
- D4 aligns governance, runtime law, and proof visibility with the frozen D1 / D2 / D3 cross-device session baseline.
- D4 does not redesign session authority, contract semantics, ingress handling, or persistence wiring. It only defines how already-frozen cross-device actions are classified, observed, gated, escalated, and protected by PH1.GOV, PH1.LAW, and PH1.J visibility.
- D4 must leave final verification closure to D5.

C) DEPENDENCY RULE
- D4 consumes D1 as frozen law for:
  - canonical session authority
  - canonical cross-device identity tuple
  - attach / resume / recover / detach meaning
  - device ordering, stale, retry, duplicate, ownership, failover, and transfer law
- D4 consumes D2 as frozen contract law for:
  - canonical field names
  - canonical outcome families
  - canonical blocked / fail-closed outcome families
  - canonical role / consistency / coordination vocabulary
- D4 consumes D3 as frozen wiring law for:
  - request / envelope / PH1.L / persistence write order
  - authoritative versus derived surface split
  - blocked outcome materialization
  - replay / re-entry / dedupe reuse
- D4 also consumes:
  - Section 04 verification-before-authority law
  - Section 07 identity/device posture law
  - Section 08 platform trust / posture law
  - Section 09 governance response classes and invariant enforcement
  - Section 11 final runtime law response classes and proof-critical enforcement
  - Phase C C4 storage / proof / law completion law as the reusable completion model
- No D4 rule may weaken D1 / D2 / D3 semantics to match current repo gaps. Gaps must remain explicit bounded assumptions.

D) ARCHITECTURAL POSITION
- D4 sits after frozen D3 materialization and before D5 verification closure.
- The canonical order remains:
  - D3 authoritative session decision and materialization
  - D4 governance visibility and runtime-law judgment for protected or law-sensitive actions
  - D5 verification that the D4 rules were implemented and observed correctly
- PH1.L / Session Engine remains the authoritative writer for session truth.
- PH1.GOV remains a deterministic governance visibility and decision layer. It must not mutate session truth directly.
- PH1.LAW remains the final runtime-law posture layer. It must not become a parallel session writer.
- PH1.J remains proof / evidence visibility. It must not become alternate session authority.
- PH1.OS remains orchestration posture and move legality visibility. It must not redefine session or law truth.

E) D1 / D2 / D3 ASSUMPTIONS CONSUMED
- D4 consumes the D1 canonical tuple:
  - `(session_id, turn_id, device_id, actor_identity_scope, platform_context, device_turn_sequence, owning_node_or_lease_ref)`
- D4 consumes the D2 canonical session-action families:
  - `attach`
  - `resume`
  - `recover`
  - `detach`
- D4 consumes the D2 / D3 outcome families:
  - success or reuse:
    - `NEW_SESSION_CREATED`
    - `EXISTING_SESSION_ATTACHED`
    - `EXISTING_SESSION_REUSED`
    - `RETRY_REUSED_RESULT`
    - `RECOVERY_ATTACH_ACCEPTED`
    - `DETACH_ACCEPTED`
  - blocked or fail-closed:
    - `STALE_REJECTED`
    - `DUPLICATE_RETRY_REUSED`
    - `IDENTITY_SCOPE_BLOCKED`
    - `PLATFORM_CONTEXT_BLOCKED`
    - `OWNERSHIP_UNCERTAIN_BLOCKED`
    - `TRANSFER_PENDING_BLOCKED`
    - `RECOVERY_WINDOW_CLOSED_BLOCKED`
    - `DETACH_NOT_LAWFUL_BLOCKED`
- D4 consumes the D2 / D3 canonical session posture vocabulary:
  - roles: `PRIMARY_INTERACTOR`, `SECONDARY_VIEWER`, `LIMITED_ATTACH`, `RECOVERY_ATTACH`
  - consistency: `STRICT`, `LEASED_DISTRIBUTED`, `DEGRADED_RECOVERY`
  - coordination: `PRIMARY_OWNED`, `TRANSFER_PENDING`, `FAILOVER_RECOVERING`, `OWNERSHIP_UNCERTAIN`
- D4 consumes the D3 rule that authoritative session truth is already committed before governance / law / proof visibility is meaningful.

F) CURRENT GOVERNANCE / LAW / PROOF / SESSION SURFACES IN SCOPE
Current Repo Surface → D4 Governance / Law Scope Mapping
| surface | current role | authoritative / evidence / decision / visibility | D4 relevance | notes / constraints |
| --- | --- | --- | --- | --- |
| `os_core.sessions` | authoritative session row family for active session truth | authoritative | canonical session authority that D4 must never override | D4 may read and gate on this truth but may not mutate it directly |
| `SessionRecord` in `ph1f.rs` / `repo.rs` | concrete storage contract for attached devices, active turn, lease fields, and per-device ordering | authoritative | canonical authoritative input to governance/law posture | richer than current PH1.L DB wiring doc; D4 must consume both without collapsing them |
| `RuntimeExecutionEnvelope.session_id` / `turn_id` / `device_identity` / `platform_context` / `device_turn_sequence` / `session_attach_outcome` | runtime-carried session tuple and action result visibility | visibility | canonical runtime-carried session context for D4 | envelope fields are read-only to downstream readers after D3 materialization |
| `RuntimeExecutionEnvelope.persistence_state` | replay, recovery, acknowledgement, reconciliation, and quarantine visibility | visibility | canonical persistence and replay posture input | persistence visibility is not session authority |
| `RuntimeExecutionEnvelope.governance_state` | governance execution state | decision / visibility | canonical PH1.GOV output carriage | D4 must define when it is required and how it is interpreted |
| `RuntimeExecutionEnvelope.proof_state` | proof result visibility | evidence / visibility | canonical PH1.J result carriage | D4 must define when protected session actions require it |
| `RuntimeExecutionEnvelope.law_state` | final runtime law posture | decision / visibility | canonical PH1.LAW output carriage | law state never replaces session authority |
| `gov_policy_bundle` / `gov_decision_bundle` | governance request evaluation and deterministic governance decision | decision | D4 governance visibility seam | no table writes in current runtime slice |
| `GovernanceExecutionState.decision_log_ref` | reference to governance decision log | visibility | deterministic governance traceability and replay anchor | log reference is evidence of decision, not session truth |
| `RuntimeLawExecutionState.decision_log_ref` | reference to runtime law decision log | visibility | deterministic law traceability and replay anchor | log reference is evidence of law posture, not session truth |
| `audit.audit_events` | append-only audit ledger | evidence | audit visibility for cross-device protected actions | PH1.J events may prove transitions but may not author them |
| canonical PH1.J proof ledger via `Ph1jRuntime::emit_proof` / replay / verify | protected proof record append and replay | evidence | proof visibility seam reused from Phase C | D4 must not invent a second proof path |
| `os_policy_bundle` / `os_decision_bundle` | orchestration legality and fail-closed next-move outputs | decision / visibility | PH1.OS posture input and observer surface | orchestration posture must never redefine governance or law truth |
| `PH1.LEASE` request / decision bundles | lease and takeover decision visibility | decision | ownership, failover, transfer, and uncertainty input | current repo slice is decision-oriented, not a second authoritative owner store |
| `multi_hint_bundle` | bounded multi-device hint visibility | visibility | secondary observer for cross-device awareness | may not clear ownership uncertainty or fabricate coordination state |

G) CANONICAL CROSS-DEVICE PROTECTED-ACTION MODEL
- D4 uses the following protected-action rule:
  - actions that create or reuse a session without changing ownership under ordinary posture are law-sensitive, but not proof-critical by default
  - actions that change primary ownership, resolve transfer or failover, admit degraded recovery, or proceed under uncertainty are protected
  - blocked outcomes caused by identity mismatch, platform mismatch, stale turn sequence, transfer pending, or ownership uncertainty are protected decision points because they must preserve authority and may affect runtime posture
- D4 uses the following safest canonical reading of current repo truth:
  - `attach` is protected when it changes canonical ownership or occurs under `STRICT` posture with sensitive role elevation
  - `resume` is protected when it reuses a session under stale, conflicting, or ownership-sensitive conditions
  - `recover` is protected by default because it depends on degraded or recovery posture
  - `detach` is protected when it changes authoritative participation or can expose unresolved ownership / transfer / failover state
  - role change, ownership uncertainty, transfer pending, failover recovering, identity mismatch, platform mismatch, and recovery-window expiration are protected or law-sensitive decision points even when they do not create a new session

Cross-Device Action → Protected / Governance / Law Matrix
| action or case | protected or not | governance visibility required | law posture required | proof visibility required | notes |
| --- | --- | --- | --- | --- | --- |
| attach | protected when it changes authoritative participation or runs under `STRICT` / elevated role posture; otherwise law-sensitive only | conditional | yes | conditional | ordinary viewer attach may remain bounded local success if policy allows |
| resume | protected when reuse collides with ownership, stale, or degraded posture; otherwise law-sensitive only | conditional | yes | conditional | `resume` never creates a new `session_id` |
| recover | protected | yes | yes | conditional and often required | safest reading is that degraded recovery is always law-sensitive |
| detach | protected when it changes authoritative participation or could expose unresolved ownership state | conditional | yes | conditional | detach is not session close unless Session Engine separately closes it |
| role change | protected | yes | yes | conditional | primary-role changes must not be treated as trivial visibility changes |
| stale message rejection | protected decision point | conditional | yes | no | must preserve authority and reason-coded refusal |
| duplicate retry reuse | not protected by itself if authoritative outcome already exists | no new governance decision required | law posture may be reused from authoritative prior decision | no new proof required | D4 must require reuse, not re-decision |
| ownership uncertainty | protected | yes | yes | conditional | default fail-closed class |
| transfer pending | protected | yes | yes | conditional | transfer in progress must block conflicting admission |
| failover recovering | protected | yes | yes | conditional and often required | degraded recovery posture may require explicit law response |
| identity mismatch | protected refusal | conditional | yes | no | must fail closed |
| platform mismatch | protected refusal | conditional | yes | no | platform trust downgrade is a law input |
| recovery-window expired | protected refusal | conditional | yes | no | expired recovery window blocks reuse/recover |

H) GOVERNANCE ALIGNMENT MODEL
- PH1.GOV sees authoritative session truth, canonical D3 materialization state, and any required proof linkage that already exists; it does not author session state.
- Governance alignment in D4 means:
  - mapping cross-device action class to governance visibility requirement
  - recording deterministic response class and severity posture when cross-device actions are protected
  - carrying quarantine or safe-mode recommendations when cluster consistency, ownership certainty, certification, or drift posture requires them
  - preserving D3 authoritative session truth even when governance visibility is delayed or incomplete
- Governance must not:
  - invent alternate attach outcomes
  - clear ownership uncertainty locally
  - change `session_id`, `turn_id`, `device_turn_sequence`, or PH1.L ownership state
  - reinterpret proof absence as successful protected completion

Governance Alignment Matrix
| concern | authoritative source | governance visibility surface | governance output or decision | must never be treated as | notes |
| --- | --- | --- | --- | --- | --- |
| protected attach or resume | PH1.L / `SessionRecord` plus D3 envelope materialization | `gov_policy_bundle`, `gov_decision_bundle`, `RuntimeExecutionEnvelope.governance_state` | deterministic governance response class and decision-log linkage | alternate session writer | governance observes and classifies; it does not create the session |
| recover under degraded posture | authoritative session truth plus `persistence_state` recovery posture | governance execution state with severity / response class / decision log | explicit governance posture for degraded recovery admission | proof substitute or persistence authority | recovery posture must remain deterministic across nodes |
| ownership uncertainty or transfer | authoritative session truth plus PH1.LEASE posture | governance execution state and optional quarantine posture | `BLOCK`, `QUARANTINE`, or governed degrade posture where required | ownership authority | governance may recommend quarantine but may not claim ownership |
| identity-sensitive role elevation | authoritative session truth plus identity verification posture | governance visibility state and certification posture | allow / warn / block according to policy | identity verifier or session authority | role elevation is not lawful if identity posture is insufficient |
| platform mismatch or device trust downgrade | authoritative session truth plus platform posture | governance visibility state and drift / certification signals | deterministic severity and response-class suggestion | platform truth or session truth | PH1.OS and platform posture remain inputs only |
| proof-critical protected action | authoritative session truth plus PH1.J proof refs/state | governance execution state and decision-log ref | decision may require proof-complete posture before protected completion | proof writer | D4 reuses C4 protected-completion law instead of redefining proof |

I) RUNTIME LAW ALIGNMENT MODEL
- PH1.LAW receives the final protected-action posture for cross-device session behavior and returns one final runtime response class.
- D4 reuses Section 11 response classes exactly:
  - `ALLOW`
  - `ALLOW_WITH_WARNING`
  - `DEGRADE`
  - `BLOCK`
  - `QUARANTINE`
  - `SAFE_MODE`
- Runtime-law inputs for cross-device session behavior include at minimum:
  - session ownership uncertainty
  - stale turn detection
  - identity mismatch
  - platform mismatch or trust downgrade
  - transfer pending
  - failover recovering
  - replay inconsistency or persistence quarantine posture
  - proof-write failure or proof-chain failure where proof is required
  - governance uncertainty or active safe-mode posture
- Law alignment must preserve the rule that authoritative session truth already committed by D3 is not rewritten by PH1.LAW. PH1.LAW only governs resulting runtime posture and protected completion.

Runtime Law Response Matrix
| concern | authoritative source | law visibility surface | response class | fail-closed rule | notes |
| --- | --- | --- | --- | --- | --- |
| ordinary lawful attach/resume with no protected posture | authoritative session truth plus clean envelope/persistence posture | `RuntimeExecutionEnvelope.law_state` | `ALLOW` | if required law inputs are missing, action may not silently self-approve | non-protected action may still be bounded-retry if visibility lags |
| lawful attach/resume with minor non-blocking posture drift | authoritative session truth plus bounded warning posture | `RuntimeExecutionEnvelope.law_state` | `ALLOW_WITH_WARNING` | warning may not suppress required blocked outcome | warning does not change authority |
| degraded recovery or replay-repair posture | authoritative session truth plus degraded persistence/recovery input | `RuntimeExecutionEnvelope.law_state` | `DEGRADE` | degraded mode must remain explicit and replayable | recovery remains protected |
| identity mismatch / platform mismatch / stale rejection / unlawful detach | authoritative session truth preserved with blocked outcome | `RuntimeExecutionEnvelope.law_state` | `BLOCK` | fail closed by default | blocked outcome remains authoritative and reason-coded |
| ownership uncertainty / repeated cross-device conflict / required isolation posture | authoritative session truth preserved, no new owner | `RuntimeExecutionEnvelope.law_state` plus quarantine inputs | `QUARANTINE` | local retry may not clear uncertainty | quarantine isolates path/subsystem, not truth |
| critical protected inconsistency, cluster divergence, or proof-critical failure | authoritative session truth preserved, protected completion withheld | `RuntimeExecutionEnvelope.law_state` plus safe-mode posture | `SAFE_MODE` | must fail closed until governed recovery path exists | D4 reuses Section 11 safe-mode posture rather than inventing a new class |

J) PROOF / VISIBILITY / COMPLETION MODEL
- D4 reuses Phase C C4 law:
  - authoritative storage/session commit must already exist before proof, governance, or law visibility matters
  - proof/governance/law visibility must never outrank authoritative truth
  - protected completion cannot be considered complete until required visibility succeeds
- The safest session-specific completion reading is:
  - ordinary attach/resume/detach under non-protected posture: authoritative D3 commit plus bounded visibility retry is enough for completion
  - protected attach/recover/role-change/ownership-sensitive action: authoritative D3 commit exists, but protected completion is withheld until required proof visibility, governance state, and final law posture are present
  - if proof/governance/law visibility fails after D3 authoritative commit, the authoritative session truth stays intact, but the runtime posture must remain degraded, blocked, quarantined, or safe-mode according to the law matrix until repaired
- Proof visibility in D4 is required where the protected action class or law policy requires it. Current repo truth does not yet expose a session-specific proof taxonomy, so D4 freezes the rule that proof-critical session actions must consume the existing PH1.J proof path and `proof_state`, not invent a session-only proof mechanism.

Proof / Completion Matrix
| cross-device action class | authoritative storage prerequisite | proof visibility required or not | governance visibility required or not | law posture required or not | completion rule | retry / refusal rule |
| --- | --- | --- | --- | --- | --- | --- |
| ordinary attach / resume / detach under non-protected posture | yes | no by default | no by default | yes | complete when authoritative session decision exists and final required law posture exists; visibility lag may be repaired | bounded retry only; no duplicate authoritative mutation |
| protected attach that changes primary participation | yes | conditional and policy-driven | yes | yes | not protected-complete until required proof/governance/law visibility all exist | if visibility fails, preserve truth but withhold protected completion |
| recover under degraded or failover posture | yes | conditional and often required | yes | yes | complete only when authoritative recovery decision plus required protected visibility exists | if required visibility is absent, remain degraded/blocked |
| blocked identity/platform/stale/transfer/ownership outcome | preserved prior authoritative truth | no new proof by default | conditional | yes | complete when blocked outcome is materialized and required law posture exists | retries must reuse blocked result until authoritative conditions change |
| ownership uncertainty / quarantine / safe-mode escalation | authoritative session truth preserved | conditional | yes | yes | complete only when escalation posture is recorded deterministically | replay may repair visibility only; it may not clear escalation locally |

K) REPLAY, RE-ENTRY, OWNERSHIP-UNCERTAINTY, AND CONFLICT POSTURE
- Replay and re-entry must start from authoritative D3 session truth, not from governance logs, law logs, proof records, or device-local hints.
- Governance and law posture for replay must follow these rules:
  - if authoritative session truth exists and required visibility is complete, replay reuses the existing decision and must not trigger a second authoritative mutation
  - if authoritative session truth exists but governance/law/proof visibility is incomplete, replay may repair missing visibility only
  - if ownership uncertainty or transfer/failover posture is already authoritative, replay must preserve that posture until authoritative recovery clears it
  - stale or duplicate device submissions may not reopen authoritative ownership or attach decisions already recorded

Replay / Ownership-Uncertainty Matrix
| case | authoritative truth source | governance effect | law effect | retry / replay behavior | notes |
| --- | --- | --- | --- | --- | --- |
| duplicate retry after authoritative success | PH1.L / `SessionRecord` plus D3 outcome | no new governance decision | reuse prior law posture | return canonical prior outcome; repair missing visibility only | duplicate retries are never a second attach |
| stale message after newer authoritative sequence exists | PH1.L sequence truth plus envelope sequence | governance may observe refusal posture only | `BLOCK` or reused blocked posture | stale message rejected deterministically | stale device evidence cannot outrank session truth |
| authoritative success with missing governance visibility | authoritative session truth | governance write repair required | law may stay provisional/degraded until repaired | replay repairs governance state only | no second session mutation |
| authoritative success with missing law visibility | authoritative session truth | governance may already exist | law write repair required; protected completion withheld | replay repairs law state only | no silent completion |
| ownership uncertainty already materialized | authoritative session truth plus uncertainty posture | governance may escalate quarantine/safe-mode | law must preserve blocked or quarantine posture | retry may not clear uncertainty | only authoritative recovery path can clear it |
| transfer pending or failover recovering | authoritative session truth plus coordination state | governance observes protected path | law may `DEGRADE`, `BLOCK`, or `QUARANTINE` as policy requires | retries reuse posture until authoritative transition completes | local retries may not finish transfer/failover by themselves |

L) FAILURE, REFUSAL, ESCALATION, QUARANTINE, AND SAFE-MODE MODEL
- D4 must classify failures into bounded versus escalated postures:
  - bounded failures:
    - missing optional governance visibility for non-protected actions
    - missing optional proof visibility for non-protected actions
    - duplicate retries when authoritative truth already exists
  - escalated failures:
    - protected action missing required proof visibility
    - ownership uncertainty unresolved
    - transfer or failover posture unresolved under protected execution
    - identity/platform mismatch on protected action
    - replay inconsistency or persistence quarantine posture affecting protected action
    - governance uncertainty or cluster inconsistency requiring quarantine or safe-mode
- Quarantine and safe-mode are law/governance outputs, not alternate session authorities.
- Protected session actions must fail closed when required governance/law/proof inputs are missing.

Failure / Escalation Matrix
| case | authoritative truth preserved | governance visibility effect | law posture effect | quarantine / safe-mode note | downstream D5 verification note |
| --- | --- | --- | --- | --- | --- |
| authoritative D3 write succeeds, proof visibility fails for protected action | yes | governance may mark protected completion incomplete | `BLOCK`, `QUARANTINE`, or `SAFE_MODE` per policy | quarantine or safe-mode when proof-critical posture requires it | D5 must verify protected completion was withheld |
| authoritative D3 write succeeds, governance visibility fails for protected action | yes | governance state incomplete and must be repaired | provisional `DEGRADE` or `BLOCK` until repaired | no local bypass allowed | D5 verifies repair path and no false completion |
| authoritative D3 write succeeds, law visibility/state fails | yes | governance may already exist | final law posture incomplete; protected completion withheld | may escalate if protected action cannot safely continue | D5 verifies final-law gating |
| proof visibility exists, authoritative D3 write failed | no authoritative session change | governance may observe refusal only | law may `BLOCK` or `SAFE_MODE` depending severity | proof may not fabricate success | D5 verifies no authority leak |
| ownership uncertainty persists after retry window | yes, prior authoritative truth only | governance may escalate quarantine | `QUARANTINE` or `SAFE_MODE` | unresolved uncertainty never self-clears | D5 verifies uncertainty escalation |
| transfer pending blocks attach/resume | yes, existing owner preserved | governance sees protected conflict | `BLOCK` or `DEGRADE` until transfer completes | no dual ownership allowed | D5 verifies no split-brain |
| failover recovering under degraded persistence posture | yes | governance sees degraded recovery posture | `DEGRADE`, `BLOCK`, or `QUARANTINE` as policy requires | may quarantine if persistence inconsistency becomes critical | D5 verifies fail-closed degraded posture |
| identity or platform mismatch on protected action | yes, no new session authority | governance visibility optional if policy does not require it | `BLOCK` minimum | quarantine optional only if repeated or critical | D5 verifies mismatch refusal remained deterministic |

M) WORKER / ENGINE / DECISION BOUNDARIES
- Session Engine / PH1.L:
  - remains the only canonical writer for session truth
  - decides session outcome first through the D3 path
- RuntimeExecutionEnvelope:
  - carries session, persistence, governance, proof, and law visibility
  - must not become an alternate authority store
- PH1.GOV:
  - observes canonical session and protected-action posture
  - emits deterministic governance decision state only
- PH1.LAW:
  - emits the final runtime response class only
  - must not author session mutations
- PH1.J:
  - emits proof / audit visibility only
  - must not author session truth or governance outcome
- PH1.OS:
  - remains orchestration legality and next-move visibility
  - must not reinterpret governance or law into alternate session authority
- PH1.MULTI, Identity, Platform, Persistence, and Lease surfaces:
  - remain input/observer surfaces only except where D3 already made them authoritative for their own domain
  - must not clear blocked cross-device postures on their own

N) D4 → D5 FREEZE BOUNDARY
D4 → D5 Boundary Matrix
| concern | frozen in D4 | deferred to D5 | rationale |
| --- | --- | --- | --- |
| protected cross-device action classes | yes | no | D5 verifies; it does not redefine protected classes |
| governance visibility requirements per action/case | yes | no | D5 checks coverage only |
| final law response classes for cross-device actions | yes | no | D5 validates implemented posture only |
| proof-critical completion rule for protected actions | yes | no | D5 verifies protected completion behavior only |
| replay / ownership-uncertainty posture | yes | no | D5 verifies determinism and no split-brain |
| quarantine / safe-mode escalation thresholds as frozen D4 reading | yes | no | D5 may only test and document them |
| command/test/doc/evidence closure for Phase D | no | yes | D5 is the explicit verification and closure phase |

O) COMPLETION CRITERIA
- D4 is complete when:
  - cross-device protected-action classes are frozen
  - governance visibility requirements are frozen
  - runtime-law response classes for cross-device session behavior are frozen
  - proof / completion rules are frozen for protected session actions
  - replay / ownership-uncertainty posture is frozen
  - failure / escalation / quarantine / safe-mode rules are frozen
  - worker / engine / decision boundaries are frozen
  - D4 → D5 boundaries are frozen
- D4 must leave no ambiguity about:
  - what governance sees
  - what law decides
  - when proof visibility matters
  - how authoritative truth is preserved when visibility is incomplete
  - when protected session actions must fail closed
