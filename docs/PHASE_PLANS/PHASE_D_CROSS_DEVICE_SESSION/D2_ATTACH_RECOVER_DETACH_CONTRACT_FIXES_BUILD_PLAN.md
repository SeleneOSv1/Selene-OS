PHASE D2 — ATTACH / RECOVER / DETACH CONTRACT FIXES BUILD PLAN

A) REPO TRUTH CHECK
- `git status --short`: empty
- current branch: `main`
- repo-truth snapshot HEAD at review start: `dd6cd0ed76862054e9c023078bf0912c052bf4a9`
- snapshot semantics: this value records the repo state reviewed at the start of this pass; it is not required to equal the later commit hash that records this document update.
- exact files reviewed:
  - `docs/CORE_ARCHITECTURE.md`
  - `docs/SELENE_BUILD_EXECUTION_ORDER.md`
  - `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`
  - `docs/PHASE_PLANS/PHASE_D_CROSS_DEVICE_SESSION/D1_CROSS_DEVICE_SESSION_CONSISTENCY_REVIEW.md`
  - `docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C1_LIFECYCLE_MODEL_REVIEW.md`
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
  - `crates/selene_os/src/device_artifact_sync.rs`

B) PURPOSE
- Freeze the contract-level normalization required so attach, resume, recover, and detach mean one thing across request inputs, runtime envelopes, PH1.L storage truth, and downstream visibility surfaces.
- Close the current mismatch between D1 canonical session-consistency law and the narrower PH1.L DB wiring slice without redesigning Section 02, Section 05, Section 08, PH1.LEASE, PH1.OS, PH1.GOV, or PH1.LAW.
- Define the exact contract classes, fields, refusal cases, and authority boundaries that D3, D4, and D5 must consume without reinterpretation.

C) DEPENDENCY RULE
- D2 consumes D1 as frozen authority for:
  - cross-device session authority
  - canonical tuple law
  - attach / resume / recover / detach semantics
  - device timeline law
  - ownership / lease / failover law
  - downstream freeze boundaries
- D2 may normalize contract surfaces only.
- D2 may not redesign runtime behavior, storage mutation order, lease decision law, governance law, runtime law, or proof law.
- D3 may materialize D2 contracts in runtime/storage code.
- D4 may wire protected completion, proof, governance, and law visibility for the D2 contracts.
- D5 may verify the D2 contracts and collect closure evidence.

D) ARCHITECTURAL POSITION
- D2 sits between D1 review law and later implementation work.
- Section 02 already freezes the behavioral model:
  - cloud-authoritative session ownership
  - one execution path mutating per turn
  - per-device monotonic sequencing
  - multi-device attach and recovery
  - ownership / transfer / failover vocabulary
- D2 therefore fixes contract shape, not behavior.
- D2 covers three contract layers only:
  - request-facing attach / resume / recover / detach inputs
  - `RuntimeExecutionEnvelope` and related outcome / replay / persistence fields
  - PH1.L / `SessionRecord` canonical session storage fields

E) D1 ASSUMPTIONS CONSUMED
- Canonical D1 tuple remains:
  - `(session_id, turn_id, device_id, actor_identity_scope, platform_context, device_turn_sequence, owning_node_or_lease_ref)`
- Canonical D1 session authority remains cloud-authoritative and centralized in Session Engine / PH1.L session truth.
- Attach, resume, recover, and detach semantics remain exactly as frozen in D1:
  - attach joins a lawful already-known or newly-created session
  - resume reuses an already-recoverable authoritative session
  - recover re-enters during degraded or failover recovery posture
  - detach removes active participation or interactive ownership without destroying session truth
- Device-side, platform-side, proof-side, governance-side, law-side, and memory-side surfaces remain visibility or decision participants only unless D1 already marked them authoritative.
- D2 must preserve the D1 statement that the PH1.L DB wiring slice is narrower than the real `SessionRecord` contract and must not let that mismatch silently redefine session law.

F) CURRENT CONTRACT SURFACES IN SCOPE
Current Repo Surface → D2 Contract Scope Mapping
| repo surface | current role | authoritative / derived / evidence / decision | D2 relevance | notes / constraints |
| --- | --- | --- | --- | --- |
| `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_02.md` Session Engine law | canonical behavioral source for attach, sequencing, cross-device ownership, and consistency vocabulary | authoritative architecture law | defines contract semantics that D2 must preserve | D2 may normalize contract fields but may not weaken Section 02 semantics |
| `crates/selene_kernel_contracts/src/runtime_execution.rs` `RuntimeExecutionEnvelope` | runtime contract carrying session, device, persistence, governance, and law context | contract surface, not storage authority | primary D2 envelope normalization target | currently carries `session_id`, `turn_id`, `device_identity`, `platform_context`, `device_turn_sequence`, `session_attach_outcome`, `persistence_state`, `governance_state`, `law_state` but not explicit attach role / consistency / coordination fields |
| `RuntimeExecutionEnvelope.session_attach_outcome` | current attach result visibility | derived visibility contract | D2 must expand and normalize outcome coverage | current values cover success/retry reuse only |
| `PersistenceRecoveryMode` / `PersistenceAcknowledgementState` / `ReconciliationDecision` | replay, recovery, and stale / duplicate handling visibility | derived visibility contract | D2 must align attach-related contract outcomes to these families | D2 must not change Section 05 replay law |
| `docs/DB_WIRING/PH1_L.md` `os_core.sessions` | PH1.L documented authoritative session storage slice | authoritative current-row session truth in current wiring spec | D2 must normalize its narrower field set against D1 and `SessionRecord` | current slice still shows single-device invariants and no PH1.L-owned ledger in this slice |
| `crates/selene_storage/src/ph1f.rs` `SessionRecord` | typed storage truth for session identity, attachment set, lease state, and per-device ordering | authoritative storage contract | primary PH1.L/session-record normalization target | real current storage truth is wider than PH1.L DB wiring doc |
| `docs/DB_WIRING/PH1_LEASE.md` and `crates/selene_kernel_contracts/src/ph1lease.rs` | lease decision law and decision bundle outputs | decision-only, not lifecycle/session writer | D2 must align ownership-ref and failover contract fields to this law | no session-only lease subsystem may be invented |
| `docs/DB_WIRING/PH1_OS.md` and `crates/selene_kernel_contracts/src/ph1os.rs` | platform runtime normalization and decision bundles | decision / visibility only | D2 must define platform-context binding without turning PH1.OS into session authority | `os_policy_bundle` / `os_decision_bundle` remain subordinate |
| `docs/DB_WIRING/PH1_GOV.md` and `docs/DB_WIRING/PH1_LAW.md` | governance and runtime-law visibility/decision surfaces | decision / visibility only | D2 must state protected blocked/refused visibility without giving these surfaces session authority | D4 later integrates protected completion |
| `docs/DB_WIRING/PH1_J.md` | audit / proof visibility | evidence only | D2 must keep attach-related audit visibility non-authoritative | D4 later integrates protected proof gating |
| `docs/DB_WIRING/PH1_MULTI.md` `multi_hint_bundle` | multi-device advisory hints | visibility only | D2 must keep advisory-only | may not become session authority |
| `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_07.md` and `crates/selene_kernel_contracts/src/ph1_voice_id.rs` | actor identity and device-context binding law | authoritative for identity verification within its domain, not for session authority | D2 must bind actor/device/platform inputs fail closed | actor mismatch cannot be corrected by platform or memory surfaces |
| `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_08.md` | platform turn-sequencing and device runtime obligations | authoritative platform behavior law, not session writer | D2 must normalize device/platform fields and ordering semantics | platform must not mutate session truth directly |

G) CANONICAL ATTACH / RESUME / RECOVER / DETACH CONTRACT MODEL
- D2 freezes four distinct contract actions:
  - `ATTACH`: device joins a lawful authoritative session container, either by creating a new session or attaching to an active one.
  - `RESUME`: device reuses the same authoritative session because the recoverable window remains lawful.
  - `RECOVER`: device re-enters while ownership, recovery, or degraded persistence posture is active and explicit recovery semantics apply.
  - `DETACH`: device or role exits active participation while leaving authoritative session truth cloud-owned and intact.
- D2 freezes required contract outcome classes:
  - success/reuse classes:
    - `NEW_SESSION_CREATED`
    - `EXISTING_SESSION_REUSED`
    - `EXISTING_SESSION_ATTACHED`
    - `RETRY_REUSED_RESULT`
    - `RECOVERY_ATTACH_ACCEPTED`
    - `DETACH_ACCEPTED`
  - blocked/fail-closed classes:
    - `STALE_REJECTED`
    - `DUPLICATE_RETRY_REUSED`
    - `IDENTITY_SCOPE_BLOCKED`
    - `PLATFORM_CONTEXT_BLOCKED`
    - `OWNERSHIP_UNCERTAIN_BLOCKED`
    - `TRANSFER_PENDING_BLOCKED`
    - `RECOVERY_WINDOW_CLOSED_BLOCKED`
    - `DETACH_NOT_LAWFUL_BLOCKED`
- D2 does not require these exact Rust symbol names to be implemented in D2 itself, but it freezes these semantic classes so D3 cannot collapse or reinterpret them.

Attach / Resume / Recover / Detach Outcome Matrix
| action | prerequisite state | authoritative writer | success outcome | blocked outcome | visibility surfaces | notes |
| --- | --- | --- | --- | --- | --- | --- |
| attach | no active authoritative session for the tuple, or lawful attach to an existing active session | Session Engine via PH1.L / `SessionRecord` | `NEW_SESSION_CREATED` or `EXISTING_SESSION_ATTACHED` | `IDENTITY_SCOPE_BLOCKED`, `PLATFORM_CONTEXT_BLOCKED`, `TRANSFER_PENDING_BLOCKED`, `OWNERSHIP_UNCERTAIN_BLOCKED` | `session_attach_outcome`, `persistence_state`, optional PH1.J / GOV / LAW visibility when protected | devices may request; only cloud session authority decides |
| resume | authoritative session exists and recoverable window is still lawful | Session Engine via PH1.L / `SessionRecord` | `EXISTING_SESSION_REUSED` | `RECOVERY_WINDOW_CLOSED_BLOCKED`, `STALE_REJECTED`, `OWNERSHIP_UNCERTAIN_BLOCKED` | `session_attach_outcome`, `persistence_state` | resume is not a new session and must preserve same canonical `session_id` |
| recover | authoritative session exists but recovery posture or degraded persistence posture is active | Session Engine via PH1.L / `SessionRecord` plus PH1.LEASE decision law where required | `RECOVERY_ATTACH_ACCEPTED` | `RECOVERY_WINDOW_CLOSED_BLOCKED`, `OWNERSHIP_UNCERTAIN_BLOCKED`, `TRANSFER_PENDING_BLOCKED` | `session_attach_outcome`, `persistence_state`, lease decision visibility, optional GOV / LAW visibility | recover is stricter than resume and must remain explicit |
| detach | authoritative session exists and detach is lawful for the current attach role / coordination state | Session Engine via PH1.L / `SessionRecord`; detach may also clear or downgrade role/participation state | `DETACH_ACCEPTED` | `DETACH_NOT_LAWFUL_BLOCKED`, `OWNERSHIP_UNCERTAIN_BLOCKED` | session current-row visibility, `session_attach_outcome` or detach result family, optional PH1.J visibility | detach is not session close unless Session Engine separately closes the session |

H) REQUEST / ENVELOPE / SESSION RECORD FIELD NORMALIZATION
- Current repo truth does not yet expose one PH1.L-owned typed request packet for attach / resume / recover / detach.
- D2 therefore freezes a canonical field map across three surfaces:
  - request-facing inputs from clients/runtime callers
  - runtime envelope fields used during execution
  - PH1.L / `SessionRecord` authoritative session storage fields
- Where the request-facing surface is currently implicit, D2 records that as a normalization gap instead of inventing a new protocol family.

Request / Envelope / Session Record Field Mapping Matrix
| canonical field | request surface | envelope surface | PH1.L / session storage surface | current gap or mismatch | normalization rule | downstream phase impact |
| --- | --- | --- | --- | --- | --- | --- |
| `session_id` | request may carry prior canonical session ref when resuming or attaching | `RuntimeExecutionEnvelope.session_id` | `os_core.sessions.session_id`; `SessionRecord.session_id` | PH1.L DB wiring and `SessionRecord` align; request-side field is implicit rather than normalized | request contract must explicitly carry canonical `session_id` whenever client believes it is resuming/attaching | D3 materializes explicit request packet fields |
| `turn_id` | request may carry prior turn context for replay/resume visibility, but cloud assigns authoritative next turn | `RuntimeExecutionEnvelope.turn_id` | `SessionRecord.last_turn_id`, `SessionRecord.active_turn_id` | no request-side normalized field today | request contract may carry prior known `turn_id` only as visibility context; authoritative turn assignment remains cloud-only | D3 enforces exact input/response placement |
| `device_id` | device-authenticated request identity | `RuntimeExecutionEnvelope.device_identity` | `os_core.sessions.device_id`; `SessionRecord.device_id`; `SessionRecord.attached_devices`; `last_attached_device_id` | PH1.L DB wiring documents one `device_id` field while `SessionRecord` already carries multi-device truth | request and envelope must carry the calling device identity; PH1.L/session storage must keep origin device plus attached-device set without narrowing back to one-device semantics | D3 normalizes storage/API field exposure |
| `device_turn_sequence` | per-device monotonic request counter | `RuntimeExecutionEnvelope.device_turn_sequence` | `SessionRecord.device_turn_sequences` | PH1.L DB wiring omits this field completely | D2 freezes it as required for attach/resume/recover idempotency and stale law | D3 adds storage/API wiring |
| `actor_identity_scope` | actor-authenticated identity scope from request/session context | `RuntimeExecutionEnvelope.actor_identity` | no exact PH1.L storage column today; derived from session/user identity scope | exact storage field absent; D1 already bounded this as tuple law | D2 freezes it as canonical contract field even where PH1.L stores only `user_id` today; mismatches must fail closed | D3 may expose or normalize persisted scope references |
| `platform_context` | platform/client request context | `RuntimeExecutionEnvelope.platform_context` | no PH1.L authoritative storage field; runtime-normalization only | currently visibility-only and not normalized into PH1.L | D2 freezes platform context as required request/envelope field and explicitly non-authoritative for session truth | D3 preserves visibility-only posture |
| `session_attach_outcome` | attach result returned to requester | `RuntimeExecutionEnvelope.session_attach_outcome` | no authoritative PH1.L storage field; outcome may be derivable from session mutation result | current outcome family lacks blocked / recover / detach classes | D2 freezes complete semantic outcome family and requires additive envelope/result normalization | D3 adds exact enum/response shape |
| `owning_node_or_lease_ref` | requester never authors it; it is returned as authoritative visibility only | no exact current envelope field; closest current visibility is lease-related state and persistence/reconciliation posture | `SessionRecord.lease_owner_id`, `lease_acquired_at`, `lease_expires_at`; PH1.LEASE active lease outputs | no single normalized field today | D2 freezes this as a contract alias for authoritative ownership/lease reference; clients may observe, never write | D3 materializes additive field exposure |
| `attach_role` | request intent or granted attach access class | no exact envelope field today | no exact PH1.L storage field today | Section 02 freezes role values but current envelope/storage do not carry them explicitly | D2 freezes explicit request/envelope/session-role normalization for `PRIMARY_INTERACTOR`, `SECONDARY_VIEWER`, `LIMITED_ATTACH`, `RECOVERY_ATTACH` | D3 chooses exact placement |
| `coordination_state` | requester never authors it; returned as authoritative visibility | no exact envelope field today | no exact single storage field; derives from session/lease posture | Section 02 vocabulary exists but contract fields do not | D2 freezes explicit visibility field for `PRIMARY_OWNED`, `TRANSFER_PENDING`, `FAILOVER_RECOVERING`, `OWNERSHIP_UNCERTAIN` | D3 materializes exact field and derivation |
| `consistency_level` | requester may declare desired posture only within lawful policy; authoritative answer is cloud-owned | no exact envelope field today | no exact PH1.L storage field today; derives from coordination/recovery posture | Section 02 vocabulary exists but no normalized field | D2 freezes explicit visibility field for `STRICT`, `LEASED_DISTRIBUTED`, `DEGRADED_RECOVERY`; cloud runtime decides actual level | D3 materializes field and lawful transitions |

I) DEVICE / ACTOR / PLATFORM BINDING RULES
- `device_id` must come from authenticated device identity, not from local client assertion alone.
- `actor_identity_scope` must come from the active verified actor/session scope and remain bound to the same authoritative session container.
- `platform_context` must describe the active runtime platform and must match `platform` validation in `RuntimeExecutionEnvelope`.
- Binding mismatches must fail closed. D2 freezes the contract obligation that none of these mismatches may be silently corrected.
- PH1.MULTI, PH1.OS, PH1.J, PH1.GOV, PH1.LAW, and memory visibility may observe these bindings but may not author or repair them.

Device / Actor / Platform Binding Matrix
| binding concern | canonical source | visibility-only sources | blocked mismatch case | downstream design impact | notes |
| --- | --- | --- | --- | --- | --- |
| device identity binding | authenticated device context and `RuntimeExecutionEnvelope.device_identity` | PH1.OS posture, PH1.J audit, PH1.MULTI hints | request device identity disagrees with authoritative attached-device or lease posture | D3 must enforce exact request/env/storage binding; D4 may add protected visibility only | device-side guesswork is forbidden |
| actor identity scope binding | `RuntimeExecutionEnvelope.actor_identity` validated against Section 07 identity law and active session scope | PH1.J, GOV, LAW visibility | actor scope does not match active session/user scope | D3 may add exact storage/API exposure; D4 may add protected refusal visibility | actor mismatch must fail closed, not degrade silently |
| platform context binding | `RuntimeExecutionEnvelope.platform_context` with platform validation | PH1.OS decision bundles, PH1.J audit | platform type mismatch, runtime context mismatch, or unauthorized device/platform combination | D3 enforces exact contract field carriage; D4 may escalate protected mismatch if required | platform context never becomes session authority |
| device-to-actor binding | authoritative session scope plus verified actor identity plus authenticated device | PH1.MULTI hints, memory thread refs | device attempts to attach under a different actor scope than the authoritative session | D3 materializes blocked outcome; D4 handles protected escalations | D2 freezes the mismatch class now |
| device-to-platform binding | authenticated device plus platform context | PH1.OS visibility | device/platform tuple violates runtime validation or policy | D3 handles exact error surface; D4 later wires governance/law visibility if protected | no fallback rebinding |

J) ATTACH ROLE / CONSISTENCY / COORDINATION CONTRACT
- D2 freezes that attach roles, consistency levels, and coordination states are contract-level values, not documentation-only prose.
- Current repo truth for these values lives mainly in Section 02 and D1, not yet in normalized envelope/session fields.
- D2 therefore requires additive contract fields so D3 can materialize them without semantic guesswork.

Attach Role / Consistency / Coordination Matrix
| contract concept | canonical values | authoritative source | visibility surfaces | current gap if any | normalization note |
| --- | --- | --- | --- | --- | --- |
| attach role | `PRIMARY_INTERACTOR`, `SECONDARY_VIEWER`, `LIMITED_ATTACH`, `RECOVERY_ATTACH` | Section 02 law plus authoritative session decision | response/result surface, envelope visibility, PH1.J / GOV / LAW only when relevant | no normalized envelope/session field today | D2 freezes these as explicit contract values |
| consistency level | `STRICT`, `LEASED_DISTRIBUTED`, `DEGRADED_RECOVERY` | Section 02 law plus Session Engine decision using storage/lease posture | envelope visibility, response visibility, PH1.OS / GOV / LAW when relevant | no normalized envelope/session field today | client preference may be observed, but authoritative level is cloud-decided |
| coordination state | `PRIMARY_OWNED`, `TRANSFER_PENDING`, `FAILOVER_RECOVERING`, `OWNERSHIP_UNCERTAIN` | Session Engine plus PH1.LEASE decision law and authoritative session state | envelope visibility, response visibility, PH1.OS / PH1.J / GOV / LAW only as observers | current value family exists only in Section 02/D1 prose | D2 freezes exact vocabulary and forbids downstream reinvention |

K) IDEMPOTENCY, STALE, RETRY, AND DUPLICATE CONTRACT LAW
- `device_turn_sequence` remains the canonical per-device ordering basis.
- D2 freezes that attach-related actions use the same deterministic ordering families as other session actions:
  - higher sequence = new operation
  - same sequence = retry or duplicate reuse
  - lower sequence = stale reject
- `SessionRecord.device_turn_sequences` and `device_last_idempotency_keys` remain the authoritative stale/duplicate reference in current repo truth.
- `ReconciliationDecision`, `PersistenceAcknowledgementState`, and `PersistenceRecoveryMode` remain visibility of the authoritative outcome, not alternate authority.

Idempotency / Stale / Retry / Duplicate Matrix
| action or case | authoritative source | ordering basis | retry behavior | stale behavior | duplicate behavior | notes |
| --- | --- | --- | --- | --- | --- | --- |
| attach | `SessionRecord.device_turn_sequences` plus session authority state | `(session_id or attempted session ref, device_id, device_turn_sequence)` | same sequence returns prior authoritative attach outcome or reuse result | lower sequence rejects as stale | same sequence with same authoritative result must reuse, not mutate again | attach may create or attach, but never twice for same sequence |
| resume | authoritative session plus per-device sequence map | `(session_id, device_id, device_turn_sequence)` | same sequence reuses prior resume outcome | lower sequence rejects | duplicate resume must not mint a new turn or new session | resume preserves same `session_id` |
| recover | authoritative session plus PH1.LEASE recovery posture plus per-device sequence map | `(session_id, device_id, device_turn_sequence, recovery posture)` | same sequence reuses prior recover decision | lower sequence rejects | duplicate recover must reuse or block deterministically | recover may require `resume_from_ledger_required` visibility |
| detach | authoritative session plus device participation state | `(session_id, device_id, device_turn_sequence or detach idempotency family)` | same detach request reuses prior detach result | stale detach rejects if later authoritative state supersedes it | duplicate detach may not drop additional devices or mutate ownership twice | D3 decides exact idempotency key placement; D2 freezes semantics |
| retry of same device sequence | authoritative session record | `device_turn_sequence` equality | return cached authoritative result | not applicable | duplicate-retry reuse only | must align with Section 02 retry law |
| stale device message | authoritative session record | `device_turn_sequence` lower than highest seen for that device | no retry as same operation; requester must fetch fresh state | reject stale | stale duplicate remains rejected | stale never mutates session state |
| simultaneous multi-device submission | authoritative session storage plus single-writer rule | cloud ordering by authoritative commit and per-device sequence discipline | later lawful request may retry after authoritative response | losing stale/conflicting request must not mutate | same-device duplicates reuse; different-device collisions resolve by authoritative ordering and coordination state | D2 freezes contract outcomes, D3 materializes queueing/wiring |
| reconnect after partial success | authoritative session record plus persistence/reconciliation visibility | authoritative stored result first, then missing downstream visibility repair | retry must repair visibility, not re-mutate session authority | stale replay still rejected | duplicate returns existing outcome | D4 later integrates protected completion |
| cross-device resume conflict | authoritative session and lease posture | authoritative session plus coordination/consistency state | retry returns canonical blocked or success outcome already decided | stale/losing side rejected | duplicate conflicting submissions reuse canonical decision | must surface `TRANSFER_PENDING` / `OWNERSHIP_UNCERTAIN` / recovery outcomes explicitly |

L) OWNERSHIP, LEASE, FAILOVER, AND TRANSFER CONTRACT BOUNDARIES
- D2 freezes ownership / lease / failover / transfer as contract vocabulary and authoritative-source law only.
- D2 does not design new takeover algorithms or storage queues.
- PH1.LEASE remains the decision engine for lease/takeover posture.
- Session storage remains the authoritative persisted session truth.
- D2 freezes `owning_node_or_lease_ref` as an additive contract field alias, not as a new subsystem.

Ownership / Lease / Failover / Transfer Contract Matrix
| concern | current repo truth | authoritative source | bounded assumption if exact surface is incomplete | forbidden inference | downstream design impact |
| --- | --- | --- | --- | --- | --- |
| current owner visibility | `SessionRecord.lease_owner_id`, lease times; Section 02 `PRIMARY_OWNED` vocabulary | authoritative session storage plus PH1.LEASE decision law | expose `owning_node_or_lease_ref` as alias over current owner/lease ref | platform or device-local observation may not claim ownership | D3 materializes exact field carriage |
| transfer in progress | Section 02 `TRANSFER_PENDING`; PH1.LEASE decision-only bundle | Session Engine plus PH1.LEASE | if no exact persisted transfer row exists, authoritative transfer posture may still be surfaced from lease/session decision state | `session_attach_outcome` alone may not invent transfer truth | D3 implements exact storage/runtime exposure |
| failover recovering | Section 02 `FAILOVER_RECOVERING`; `PersistenceRecoveryMode::Recovering` / `DegradedRecovery` | Session Engine plus persistence/lease state | current repo truth uses recovery/persistence posture instead of one dedicated failover row | memory, PH1.OS, GOV, or LAW may not author failover truth | D3 materializes exact contract fields; D4 integrates protected visibility |
| ownership uncertainty | Section 02 `OWNERSHIP_UNCERTAIN`; `QuarantinedLocalState` visibility family | authoritative session plus PH1.LEASE / persistence reconciliation decision | if current repo lacks one persisted uncertainty marker, D2 still freezes the blocked contract outcome | client retry or PH1.MULTI hint may not clear uncertainty by itself | D3 must surface explicit blocked outcome; D4 may escalate protected posture |
| resume-from-ledger requirement | PH1.LEASE `resume_from_ledger_required` | PH1.LEASE decision bundle | current repo truth already exposes this as decision output, not session storage | requester may not override it | D3 carries the field; D4 decides protected completion implications |

M) FAILURE, REFUSAL, AND SAFE-FAIL CONTRACT MODEL
- D2 freezes the contract-level refusal families that D3 must materialize.
- Fail-closed means:
  - preserve authoritative session truth
  - emit explicit blocked/refused outcome
  - never synthesize client-local continuation
- Visibility surfaces may report the refusal but may not repair or override authoritative truth.

Failure / Refusal / Escalation Matrix
| case | authoritative truth preserved | blocked or fail-closed outcome | visibility surfaces updated | downstream defer note | notes |
| --- | --- | --- | --- | --- | --- |
| actor identity mismatch on attach/resume | authoritative session and lease state unchanged | `IDENTITY_SCOPE_BLOCKED` | response outcome, optional PH1.J, optional GOV / LAW if protected | D3 materializes exact contract fields; D4 handles protected escalation | no silent identity rebinding |
| platform context mismatch | authoritative session unchanged | `PLATFORM_CONTEXT_BLOCKED` | response outcome, PH1.OS visibility, optional audit | D3 materializes exact contract fields | platform mismatch may not degrade into best-effort attach |
| stale device sequence | authoritative session unchanged | `STALE_REJECTED` | `persistence_state`, reconciliation visibility, response outcome | D3 wires exact stale response | stale requests never reopen or transfer ownership |
| duplicate retry of same operation | authoritative prior result reused | `DUPLICATE_RETRY_REUSED` or existing success outcome reused | response outcome, reconciliation visibility | D3 exact idempotency implementation | duplicate retries must not double-mutate |
| recovery window closed | authoritative session unchanged or new-session decision remains cloud-controlled outside blocked recover path | `RECOVERY_WINDOW_CLOSED_BLOCKED` | response outcome, persistence visibility | D3 exact branching; D4 if protected path requires visibility | recover blocked does not let client force attach |
| ownership uncertain / failover unresolved | authoritative session unchanged until resolved | `OWNERSHIP_UNCERTAIN_BLOCKED` | response outcome, persistence visibility, optional GOV / LAW | D4 may later integrate protected escalation | no device-side ownership claim allowed |
| transfer pending blocks attach/resume | authoritative transfer posture unchanged | `TRANSFER_PENDING_BLOCKED` | response outcome, persistence visibility, optional PH1.LEASE decision visibility | D3 exact output surface | transfer posture must remain explicit |
| detach not lawful for current role/state | authoritative session unchanged | `DETACH_NOT_LAWFUL_BLOCKED` | response outcome, optional audit | D3 exact detach output materialization | detach may be blocked while primary active turn or recovery posture persists |
| protected attach/recover result stored but downstream protected visibility missing | authoritative session commit preserved | bounded incomplete/protected-not-complete posture; no false protected completion | response visibility, persistence visibility, later proof/gov/law visibility | D4 owns protected completion integration | D2 freezes contract requirement only |

N) D2 → D3 / D4 / D5 FREEZE BOUNDARY
- D2 freezes semantics and contract normalization.
- D3 may implement field placement, storage shape normalization, runtime wiring, and response materialization only.
- D4 may integrate proof/governance/law visibility and protected completion only.
- D5 may verify the results and collect closure evidence only.

D2 → D3 / D4 / D5 Boundary Matrix
| concern | frozen in D2 | deferred to D3 | deferred to D4 | deferred to D5 | rationale |
| --- | --- | --- | --- | --- | --- |
| attach / resume / recover / detach semantic meaning | yes | no | no | no | D1 and D2 must lock semantics before implementation |
| canonical field set across request / envelope / session storage | yes | exact struct/field placement and additive code wiring | no | verification only | D3 may materialize, not reinterpret |
| outcome class families, including blocked/refused outcomes | yes | exact enum/result placement | protected visibility completion rules for protected cases | verification only | downstream phases may not collapse or rename away semantics |
| attach role / consistency / coordination vocabulary | yes | exact storage/API carriage | protected visibility where required | verification only | Section 02 values must not drift |
| idempotency / stale / duplicate semantic law | yes | exact runtime/storage dedupe implementation | protected replay visibility integration | verification only | semantics must be fixed before materialization |
| ownership / lease / failover / transfer contract boundaries | yes | exact field exposure and runtime/storage plumbing | proof/gov/law integration for protected ownership transitions | verification only | D2 freezes authority boundaries without inventing new lease architecture |
| PH1.J / GOV / LAW protected session completion | no semantic redesign; only D2 acknowledgment that protected visibility may be required | no | yes | verification only | D4 owns integration, not D2 |
| tests, docs, traceability, closure evidence | no | no | no | yes | D5 remains closure phase |

O) COMPLETION CRITERIA
- D2 is complete when all of the following are true in the plan baseline:
  - attach / resume / recover / detach semantic classes are explicitly frozen
  - request / envelope / PH1.L/session-record normalization is explicit for every canonical D1 tuple field relevant to attach flows
  - success, blocked, and fail-closed outcome classes are explicit and bounded
  - device / actor / platform mismatch rules are explicit and fail closed
  - attach role / consistency / coordination vocabulary is explicitly normalized from Section 02 into contract law
  - idempotency / stale / retry / duplicate rules are explicit and aligned to Section 02 and Section 05
  - ownership / lease / failover / transfer boundaries are explicit and tied to PH1.LEASE plus authoritative session storage truth
  - D3 / D4 / D5 boundaries are explicit enough that none of those phases can reinterpret D2 semantics later
- D2 does not require code, tests, CI, or docs updates beyond this plan file.
