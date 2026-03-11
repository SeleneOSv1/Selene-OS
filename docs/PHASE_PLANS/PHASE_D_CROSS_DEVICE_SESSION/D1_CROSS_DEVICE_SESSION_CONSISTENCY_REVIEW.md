PHASE D1 — CROSS-DEVICE SESSION CONSISTENCY REVIEW

A) REPO TRUTH CHECK
- `git status --short`: empty
- current branch: `main`
- repo-truth snapshot HEAD at review start: `09c112355f67dae6fc5169badbcc1f197cc3a221`
- snapshot semantics: this value records the repo state reviewed at the start of this pass; it is not required to equal the later commit hash that records this document update.
- exact files reviewed:
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C1_LIFECYCLE_MODEL_REVIEW.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C2_WAKE_ARTIFACT_LIFECYCLE_WORKERS_BUILD_PLAN.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C3_MEMORY_RETENTION_PURGE_DELETE_ENFORCEMENT_BUILD_PLAN.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C4_STORAGE_PROOF_LAW_INTEGRATION_BUILD_PLAN.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C5_TESTS_DOCS_VERIFICATION_BUILD_PLAN.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_01.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_02.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_05.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_06.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_07.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_08.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_11.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_L.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_LEASE.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_MULTI.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_OS.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_M.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_GOV.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_LAW.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_J.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1l.rs`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1lease.rs`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1multi.rs`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1os.rs`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1_voice_id.rs`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_governance.rs`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_law.rs`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/repo.rs`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1f.rs`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/device_artifact_sync.rs`

B) PURPOSE
- D1 freezes the canonical reading of cross-device session consistency for the current Selene architecture.
- D1 is a review baseline only. It does not redesign Session Engine, persistence, runtime, identity, governance, or law.
- D1 must make future Phase D work consume one authority model for session truth, device ordering, multi-device attach, retry, stale rejection, failover, and downstream visibility.

C) DEPENDENCY RULE
- D1 consumes the approved Section 02 Session Engine model, Section 05 persistence/replay law, Section 07 identity posture, Section 08 platform runtime normalization, Section 09 governance enforcement, and Section 11 runtime-law posture.
- D1 also consumes frozen C1-C5 authority/evidence law:
  - C1 freezes authority vs receipt vs projection vs evidence.
  - C2 freezes worker re-entry, idempotency, and reverse-authority refusal for wake lifecycle.
  - C3 freezes derived-view refusal and cloud-authoritative memory posture.
  - C4 freezes storage-before-visibility and protected completion rules.
  - C5 freezes verification/closure expectations.
- No downstream D phase may reinterpret D1 authority, tuple, stale/retry, or ownership law without an explicit new review.

D) ARCHITECTURAL POSITION
- Cross-device session consistency sits at the boundary where Session Engine authority, persistence reconciliation, device/platform normalization, identity verification posture, memory continuity, governance visibility, proof visibility, and runtime law must agree on one canonical session event.
- Session Engine remains the only subsystem allowed to mutate canonical session state.
- Persistence + Sync may reconcile, dedupe, and recover, but it does not become alternate session authority.
- Platform Runtime may normalize device-originated input and reject invalid device ordering, but it may not mutate session state directly.
- Identity + Voice may verify actor/device posture, but it does not create or transfer session authority.
- Memory may preserve continuity across sessions and devices, but it does not authoritatively resume, recover, or detach a session.
- PH1.J, PH1.GOV, and PH1.LAW remain evidence, decision, and law-visibility participants only.

E) CURRENT REPO SURFACES IN SCOPE
- Current repo truth for cross-device session consistency is distributed across architectural law, PH1.L current state persistence, session runtime envelopes, Section 05 replay/reconciliation posture, and bounded lease/failover coordination surfaces.
- Two current-repo slices must be read together:
  - PH1.L DB wiring documents `os_core.sessions` as the current authoritative persistence slice.
  - `SessionRecord` in `crates/selene_storage/src/ph1f.rs` already carries multi-device and lease fields (`attached_devices`, `last_attached_device_id`, `device_turn_sequences`, `device_last_idempotency_keys`, `lease_owner_id`, `lease_expires_at`).
- D1 therefore freezes the safest canonical reading:
  - session authority remains PH1.L-owned
  - current session truth is stored as current-row state, not as a PH1.L-owned append-only ledger in the current slice
  - multi-device and lease posture already exist in typed storage truth even though PH1.L DB wiring has not yet been normalized to show the full multi-device field set

Current Repo Surface → D1 Session Consistency Scope Mapping
| repo surface | current role | authoritative / derived / evidence / decision | cross-device relevance | notes / constraints |
| --- | --- | --- | --- | --- |
| `os_core.sessions` / `SessionRecord` | current session state persistence | authoritative | canonical session state, attached device set, active turn, device ordering, lease posture | PH1.L owns session lifecycle truth; current slice is current-row truth, not append-only session ledger |
| `SessionRecord.attached_devices` | membership of devices bound to the session | authoritative | defines which devices may lawfully act within the session scope | current repo truth already exceeds single-device wording in `PH1_L.md` |
| `SessionRecord.device_turn_sequences` | highest accepted per-device turn sequence | authoritative | new/retry/stale classification per attached device | monotonic per device; stale values must not mutate session state |
| `SessionRecord.device_last_idempotency_keys` | last accepted idempotency key per attached device | authoritative | duplicate detection and retry reuse | must remain subordinate to session authority, not a second state machine |
| `SessionRecord.lease_owner_id` / `lease_acquired_at` / `lease_expires_at` | current ownership posture for active execution | authoritative | single-writer and failover posture | exact cross-node session lease event stream is not separately surfaced in current repo truth |
| `RuntimeExecutionEnvelope.session_id` / `turn_id` | canonical execution tuple carried through runtime | evidence of current execution slice, not storage authority | binds device actions to session/turn truth | must be read from cloud-authoritative admission/session resolution, never minted locally |
| `RuntimeExecutionEnvelope.device_identity` / `device_turn_sequence` | device-originated ordering and identity context | evidence carried into authoritative resolution path | per-device ordering, stale detection, retry reuse | platform/runtime may reject invalid values but do not become session writers |
| `RuntimeExecutionEnvelope.session_attach_outcome` | attach/resume outcome visibility | evidence | exposes whether a request created, reused, or attached to session truth | visibility only; does not itself mutate session state |
| `RuntimeExecutionEnvelope.persistence_state` | persistence reconciliation and recovery posture | evidence / decision input | replay, retry, stale rejection, quarantine, fresh-state requests | authoritative for persistence posture only, never for session truth |
| `audit.audit_events` | audit/event visibility | evidence | attach/resume/recover/stale/failover observability | PH1.J evidence only |
| `os_policy_bundle` / `os_decision_bundle` | orchestration legality and fail-closed next move | decision | attach/recover legality and runtime refusal posture | PH1.OS does not write session state |
| `PH1.LEASE` request/decision bundles | deterministic persisted-lease law pattern | decision | informs canonical ownership discipline and takeover law | PH1.LEASE current runtime slice is work-order specific; D1 treats it as lease-discipline law, not proof of a session-specific table |
| `memory.memory_threads_ledger` / `memory.memory_thread_refs` / `memory.memory_archive_index` | continuity and restore/archive context across devices | authoritative in memory domain only | cross-session continuity and resumability hints | memory may inform continuity but must never authoritatively resume a session |
| `PH1.GOV` decision surfaces | governance certification and decision visibility | decision | attach/recover restrictions when governed posture applies | not a direct session writer |
| `RuntimeExecutionEnvelope.law_state` | final runtime-law posture | decision | fail-closed runtime posture for protected session actions | not a direct session writer |

F) CANONICAL CROSS-DEVICE CONSISTENCY MODEL
- Canonical cross-device consistency means one session container, one authoritative turn stream, and one cloud-resolved ownership posture even when many devices observe or submit against that session.
- The governing rules are:
  - session state is cloud-authoritative and PH1.L-owned
  - one active execution path may mutate session state for a turn
  - each attached device advances its own monotonic `device_turn_sequence`
  - retries reuse prior authoritative outcome; stale submissions are rejected
  - device-originated evidence, visibility, or local recovery posture may never reverse-authorize session truth
  - attach/resume/recover decisions occur in cloud authority, not on clients
  - failover/transfer/recovery posture must be explicit; uncertainty must fail closed
- D1 canonicalizes four layers:
  - session authority layer
  - persistence/replay layer
  - device/platform ingress layer
  - visibility/decision/law layer
- The layers must agree on one event ordering, but only the authority layer may decide canonical session state.

G) DEVICE / SESSION / TURN / ACTOR IDENTITY TUPLE
Device / Session / Turn / Actor Identity Tuple

Canonical tuple:
- `(session_id, turn_id, device_id, actor_identity_scope, platform_context, device_turn_sequence, owning_node_or_lease_ref)`

| tuple field | what it identifies | current repo truth anchor | D1 law |
| --- | --- | --- | --- |
| `session_id` | authoritative conversation container | `SessionId`, `SessionSnapshot`, `RuntimeExecutionEnvelope.session_id`, `SessionRecord.session_id` | must always resolve from Session Engine authority, never client guesswork |
| `turn_id` | one execution unit within the session | `RuntimeExecutionEnvelope.turn_id`, PH1.J correlation, PH1.LEASE envelope, Section 02 causality chain | turn authority is per authoritative execution slice; retries must not mint a new canonical turn |
| `device_id` | submitting or attached device identity | `RuntimeExecutionEnvelope.device_identity`, `SessionRecord.attached_devices`, PH1.K/PH1.VOICE.ID scope checks | a device may only act if it is lawfully attached and identity/platform gates pass |
| `actor_identity_scope` | the user/identity scope bound to the session event | `RuntimeExecutionEnvelope.actor_identity`, identity verification posture, PH1.VOICE.ID and Section 07 consistency levels | identity posture constrains legality, but it does not replace session authority |
| `platform_context` | platform-normalized device/runtime context | `RuntimeExecutionEnvelope.platform` and `platform_context` | platform context informs legality and stale/retry handling, not ownership |
| `device_turn_sequence` | per-device monotonic ordering counter | `RuntimeExecutionEnvelope.device_turn_sequence`, `SessionRecord.device_turn_sequences` | required to classify new vs retry vs stale device submissions |
| `owning_node_or_lease_ref` | the runtime ownership posture that authorizes mutation | `SessionRecord.lease_owner_id`, `lease_acquired_at`, `lease_expires_at`, Section 02 coordination states | concrete session ownership is authoritative only when cloud-visible and unambiguous; if uncertain, fail closed |

H) ATTACH / RESUME / RECOVER / DETACH CONSISTENCY LAW
- `attach` means a lawful additional device joins an already-governed session container without creating a new session identity.
- `resume` means the cloud reuses the same recoverable session for continued interaction.
- `recover` means the system re-establishes lawful session progression after retry, reconnect, failover, restart, or degraded recovery posture.
- `detach` means a device stops participating in session visibility or interaction. In current repo truth, detach is not yet a first-class persisted session event surface and must therefore be treated as a bounded gap, not a hidden extra state family.
- Canonical D1 law:
  - attach and resume are session-authoritative outcomes
  - recover is lawful only when session ownership, persistence posture, and law/governance posture permit it
  - detach may affect visibility or allowed interaction, but it may not silently mutate canonical session authority without an explicit authoritative Session Engine write

Attach / Resume / Recover / Detach Applicability Matrix
| action | prerequisite state | authoritative writer | visibility surfaces | lawful outcome | blocked outcome | notes |
| --- | --- | --- | --- | --- | --- | --- |
| `ATTACH` | recoverable session exists, identity/device scope is lawful, ownership is not uncertain | Session Engine via PH1.L authoritative session write | `RuntimeExecutionEnvelope.session_attach_outcome`, PH1.J event, PH1.OS decision posture | existing session remains canonical; device is added to lawful attached scope | unauthorized attach, stale attach, attach during ownership uncertainty | attach may widen lawful `attached_devices`; it does not mint new session authority |
| `RESUME` | recoverable session window remains valid and policy/law allow reuse | Session Engine via PH1.L | attach outcome, PH1.J, persistence state, optional GOV/LAW visibility | `ExistingSessionReused` or equivalent canonical reuse | new session required, unsafe recovery, law/governance block | resume is cloud-only decision, never client inference |
| `RECOVER` | retry/reconnect/failover/restart path with authoritative session state still lawful | Session Engine for session truth; persistence layer for reconciliation posture only | `persistence_state`, PH1.J, optional GOV/LAW posture | same authoritative session and turn posture is reused or safely resumed | ownership uncertain, stale replay, protected recovery lacking required posture | recover may request fresh session state; it may not invent a new authority path |
| `DETACH` | device was previously attached or observing the session | bounded gap: no standalone detach writer is surfaced in current slice; safest canonical reading is PH1.L-owned session scope update when implemented | client disconnect visibility, platform telemetry, PH1.J if lifecycle-significant | device stops being treated as an active interactor or observer | client-local detach claim that attempts to rewrite session truth | detach is a canonical concept but current repo truth lacks a first-class detached event or row |

I) DEVICE TIMELINE, ORDERING, STALE, AND RETRY LAW
- Cross-device ordering is not one global client clock. It is the combination of:
  - authoritative `turn_id`
  - per-device monotonic `device_turn_sequence`
  - single-writer session mutation
  - persistence reconciliation and dedupe posture
- New-turn, retry, and stale behavior are frozen from Section 02:
  - larger device sequence = new device turn candidate
  - equal device sequence = retry; cached authoritative result must be reused
  - smaller device sequence = stale; reject
- Cross-device collisions must resolve through authoritative session/ownership truth, not by trusting whichever client arrived last or fastest.

Device Timeline / Ordering Matrix
| signal or case | authoritative source | ordering basis | retry behavior | stale behavior | duplicate behavior | notes |
| --- | --- | --- | --- | --- | --- | --- |
| new turn from attached device | PH1.L session truth plus active ownership posture | `turn_id` allocation plus `device_turn_sequence > highest_seen` for that device | not a retry; may proceed if ownership and policy are lawful | N/A | duplicate mutation blocked by idempotency and active-turn authority | single-writer rule still applies across devices |
| retry of same device sequence | `SessionRecord.device_turn_sequences` and `device_last_idempotency_keys` | same `device_id`, same `device_turn_sequence`, same or equivalent idempotency family | return prior authoritative outcome or bounded reconciliation result | N/A | duplicate authoritative mutation forbidden | if prior downstream visibility is missing, only visibility repair may retry |
| stale device message | `SessionRecord.device_turn_sequences` | incoming sequence lower than stored sequence | none; do not rerun | reject stale request | stale duplicates stay rejected | stale client evidence may never reopen prior session state |
| simultaneous multi-device submission | Session Engine ownership and active-turn authority | authoritative admission ordering under one owner/lease | losing submission may receive retry/reuse or fresh-state request depending conflict posture | stale if replayed after winner commits | duplicate turn mutation blocked | clients never arbitrate winner |
| reconnect after partial success | authoritative session row and persistence reconciliation state | authoritative commit first, then replay/reconciliation decision | repair missing downstream visibility only | stale local state must request fresh session state | duplicate authoritative write refused | canonical from C4 storage-before-visibility law |
| cross-device resume conflict | PH1.L session truth plus ownership/lease posture | recoverable session check, current owner certainty, policy/law visibility | only one canonical resume outcome may survive | stale resume claims rejected or fresh-state requested | duplicate resume result may be reused, not recomputed as new authority | when ownership is uncertain, recover fail closed |

J) OWNERSHIP, LEASE, FAILOVER, AND TRANSFER MODEL
- Current repo truth already supports a bounded session ownership model:
  - Section 02 defines coordination states, consistency levels, and access classes
  - `SessionRecord` carries concrete lease fields in typed storage
  - PH1.LEASE documents the deterministic persisted-ledger-only takeover law pattern
  - Section 05 and `RuntimeExecutionEnvelope.persistence_state` freeze recovery, quarantine, and replay posture
- D1 safest canonical reading:
  - session mutation authority belongs to exactly one cloud-visible owner at a time
  - attached devices do not own the session; they submit into the owner-governed path
  - transfer and failover are lawful only when ownership posture is explicit and deterministically coordinated
  - any ownership ambiguity must produce refusal, fresh-state request, or degraded recovery posture rather than silent continuation

Ownership / Lease / Failover Matrix
| concern | current repo truth | authoritative owner or source | bounded assumption if exact surface is incomplete | forbidden inference | downstream design impact |
| --- | --- | --- | --- | --- | --- |
| active session mutation owner | Section 02 single-writer rule plus `SessionRecord.lease_owner_id` | Session Engine cloud owner | exact persisted session-owner event stream is not separately exposed today | a client device or visibility surface may not self-elect as owner | D2/D3 may refine storage/API shape, not the one-owner rule |
| attached-device participation | `SessionRecord.attached_devices`, `last_attached_device_id`, access classes from Section 02 | PH1.L-owned session truth | current repo lacks a standalone attach ledger/event table | attached device membership may not be inferred from local cache alone | D2 may design authoritative attach surface without changing the rule |
| lease expiry and renewal | `lease_acquired_at`, `lease_expires_at`, PH1.LEASE persisted-ledger-only takeover law | authoritative session owner under persisted cloud state | current repo exposes work-order lease law explicitly and session lease fields concretely, but not one normalized session-lease DB wiring document | RAM-only takeover or hidden local lease recovery | D3 may design transfer/failover implementation details only |
| transfer | Section 02 `TRANSFER_PENDING` plus ownership handoff law | Session Engine coordination posture | exact transfer record surface is not exposed in current repo slice | observing both devices does not mean dual ownership | D3 may design handoff mechanics; D1 freezes no split-brain |
| failover recovery | Section 02 `FAILOVER_RECOVERING` and `OWNERSHIP_UNCERTAIN`; persistence recovery modes | Session Engine plus persistence reconciliation posture | exact failover outcome record surface is not exposed today | stale node or stale device evidence cannot prove recovery completion | D3/D4 may design recovery visibility and escalation, not alternate authority |
| access role vs ownership | Section 02 `PRIMARY_INTERACTOR`, `SECONDARY_VIEWER`, `LIMITED_ATTACH`, `RECOVERY_ATTACH` | Session Engine session scope and policy/law gates | exact access-role storage surface is architectural rather than normalized as a current table | a device with visibility is not necessarily allowed to mutate | D2 may design attach-role materialization without changing the meaning |

K) CROSS-SUBSYSTEM VISIBILITY AND AUTHORITY SPLIT
- Cross-device consistency fails if any visibility layer is allowed to rewrite session truth.
- D1 freezes the split:
  - Session Engine writes canonical session truth
  - Persistence + Sync writes reconciliation posture and durable supporting state
  - Platform Runtime and Identity + Voice validate device/actor context
  - Memory supplies continuity hints only
  - PH1.J records evidence
  - PH1.GOV and PH1.LAW decide visibility and runtime restrictions

Cross-Subsystem Visibility Matrix
| subsystem | authoritative writer or not | visibility surface(s) | cross-device effect | must never be treated as | notes |
| --- | --- | --- | --- | --- | --- |
| Session Engine / PH1.L | yes | session state exposure, attach outcome, PH1.J event refs | canonical session continuity, ordering, ownership, attached-device scope | a mere visibility layer | sole writer of canonical session lifecycle truth |
| Persistence + Sync | authoritative for persistence/reconciliation posture only | `RuntimeExecutionEnvelope.persistence_state`, recovery/quarantine posture | replay, dedupe, stale rejection, fresh-state requests | alternate session lifecycle owner | may repair missing downstream visibility, not rewrite prior session truth |
| Platform Runtime | no session writes | `platform_context`, device trust posture, platform events | rejects invalid device ordering and illegal device actions | session authority | ensures device-originated actions match session law |
| PH1.OS | decision only | `os_policy_bundle`, `os_decision_bundle` | fail-closed orchestration and attach/recover legality posture | direct session writer | consumes session/lease/idempotency posture only |
| Identity + Voice | no session writes | `identity_state`, voice/device verification posture | constrains whether a device/actor may attach, resume, or continue | session ownership or session recovery truth | identity trust may restrict action but not mint a session |
| Memory | authoritative in memory domain only | memory thread/archive/ref surfaces, `memory_state` | continuity hints and cross-device recall context | session resume authority | memory cannot authoritatively attach, detach, or recover sessions |
| PH1.MULTI | no | `multi_hint_bundle` | may enrich context for a turn across device-originated inputs | session authority or session continuity truth | advisory only and fail-closed on evidence/privacy drift |
| PH1.J | no | `audit.audit_events`, `proof_entry_ref`, `proof_record_ref` | makes attach/recover/stale/failover visible and auditable | authoritative session state | evidence only |
| PH1.GOV | no | `gov_decision_bundle`, `governance_state` | restricts or certifies governed cross-device actions | direct session writer | decision/certification layer only |
| PH1.LAW | no | `RuntimeExecutionEnvelope.law_state` | final runtime posture for protected cross-device actions | direct session writer | final runtime-law visibility, not session storage |

L) CURRENT CONFLICTS / GAPS
- `PH1_L.md` still documents `sessions(session_id, user_id, device_id, session_state, opened_at, last_activity_at, closed_at)` as the current persistence slice, while `SessionRecord` already contains `attached_devices`, `last_attached_device_id`, `lease_owner_id`, `lease_expires_at`, `device_turn_sequences`, and `device_last_idempotency_keys`.
- Section 02 and `SessionRecord` support multi-device attach, but the PH1.L DB wiring document has not yet been normalized to reflect the full multi-device session current row.
- The current repo slice does not expose a standalone attach/detach ledger or session event stream table for session-governed device participation.
- The current repo slice does not expose one normalized persisted session transfer/failover event surface even though Section 02 requires explicit ownership posture and `SessionRecord` already carries ownership fields.
- `CORE_ARCHITECTURE.md` already records a gap: canonical session identifiers are not yet exposed consistently in every client-visible response path.
- `detach` is canonically necessary but not yet represented as an explicit first-class persisted session action in the current repo slice.
- PH1.LEASE gives deterministic persisted-takeover law for work orders, not a separately documented session-lease table; D1 therefore treats session lease discipline as concrete in `SessionRecord` plus architectural law, while explicitly refusing to invent a separate current repo surface that is not present.

Current Repo Surface to Canonical D1 Mapping
| current surface | canonical D1 role | authority class | downstream phase most affected | notes |
| --- | --- | --- | --- | --- |
| `os_core.sessions` / `SessionRecord` | canonical cross-device session truth | authoritative | D2 | D2 may normalize storage/API surfaces, not alter authority |
| `SessionRecord.attached_devices` | lawful device membership set | authoritative | D2 | canonical attach scope already exists in typed storage truth |
| `SessionRecord.device_turn_sequences` | per-device monotonic ordering memory | authoritative | D3 | D3 may design sync/recovery mechanics, not stale/new/retry meaning |
| `SessionRecord.device_last_idempotency_keys` | retry/dedup anchor per device | authoritative supporting state | D3 | subordinate to session authority |
| `SessionRecord.lease_*` fields | current ownership posture | authoritative supporting state | D3 | D3 may formalize transfer/failover surfaces |
| `RuntimeExecutionEnvelope.session_attach_outcome` | attach/resume/retry visibility | evidence | D2 | visibility only, never authority |
| `RuntimeExecutionEnvelope.persistence_state` | replay/recovery/quarantine posture | decision/evidence input | D3/D4 | not a session writer |
| `os_policy_bundle` / `os_decision_bundle` | orchestration legality and fail-closed posture | decision | D4 | no direct session mutation |
| `audit.audit_events` and proof refs | audit/proof visibility | evidence | D4 | no alternate session truth |
| `gov_decision_bundle` / `law_state` | governed and runtime-law posture | decision | D4 | constrain action completion only |
| memory continuity surfaces | cross-device continuity context | authoritative in memory domain only | D2/D4 | memory continuity is not session authority |

M) D1 → D2 / D3 / D4 / D5 FREEZE BOUNDARY
- D1 freezes the following for all downstream D phases:
  - Session Engine is canonical session authority.
  - The cross-device event tuple is fixed.
  - Attach/resume/recover/detach meanings are fixed.
  - New/retry/stale/duplicate law is fixed.
  - Ownership uncertainty must fail closed.
  - Visibility, proof, governance, law, identity, platform, and memory may not become alternate session authority.
- Downstream phases may add implementation detail, storage normalization, proofs, tests, and operational closure only within those frozen meanings.

D1 → D2 / D3 / D4 / D5 Boundary Matrix
| concern | frozen in D1 | deferred to D2 | deferred to D3 | deferred to D4 | deferred to D5 | rationale |
| --- | --- | --- | --- | --- | --- | --- |
| canonical session authority / visibility split | yes | no reinterpretation; only materialization details | no | no | no | D1 must lock the authority model first |
| session storage/API normalization for attach and identifier exposure | yes, as law | yes | no | no | no | D2 may normalize authoritative/session-visible surfaces without changing semantics |
| device ordering, dedupe, stale rejection, reconnect, failover mechanics | yes, as law | bounded consumption only | yes | bounded visibility/escalation only | no | D3 may implement mechanics, not redefine rules |
| proof/governance/law handling for cross-device anomalies or protected transitions | yes, as boundary | no | bounded handoff only | yes | no | D4 may wire visibility and fail-closed completion against frozen authority |
| verification, evidence pack, traceability, residual-risk closure | yes, as closure requirement | no | no | no | yes | D5 verifies D1-D4 rather than redesigning them |
| memory continuity interaction with cross-device session | yes, as non-authority boundary | bounded consumption only | bounded consumption only | bounded visibility only | verification only | prevents memory from becoming alternate session authority |

N) COMPLETION CRITERIA
- D1 is complete when:
  - current repo surfaces participating in cross-device session consistency are explicitly mapped
  - authority vs derived vs evidence vs decision posture is explicit
  - the canonical identity tuple is frozen
  - attach/resume/recover/detach semantics are frozen
  - device ordering, retry, stale, and duplicate law are frozen
  - ownership, lease, failover, and transfer are frozen at the current canonical reading without inventing missing tables
  - subsystem authority/visibility split is frozen
  - repo-truth conflicts and missing surfaces are named explicitly
  - D2/D3/D4/D5 are bounded tightly enough that they cannot reopen D1 semantics
