PHASE D3 — RUNTIME AND PERSISTENCE WIRING BUILD PLAN

A) REPO TRUTH CHECK
- `git status --short`: empty
- current branch: `main`
- repo-truth snapshot HEAD at review start: `8377f860b774ba961b8011cf68cefbaea244f3e0`
- snapshot semantics: this value records the repo state reviewed at the start of this pass; it is not required to equal the later commit hash that records this document update.
- exact files reviewed:
  - `docs/CORE_ARCHITECTURE.md`
  - `docs/SELENE_BUILD_EXECUTION_ORDER.md`
  - `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`
  - `docs/PHASE_PLANS/PHASE_D_CROSS_DEVICE_SESSION/D1_CROSS_DEVICE_SESSION_CONSISTENCY_REVIEW.md`
  - `docs/PHASE_PLANS/PHASE_D_CROSS_DEVICE_SESSION/D2_ATTACH_RECOVER_DETACH_CONTRACT_FIXES_BUILD_PLAN.md`
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
- D3 materializes the frozen D2 attach / resume / recover / detach contract into runtime and persistence surfaces.
- D3 does not redesign D1 consistency law or D2 contract semantics. It only defines how the already-frozen contract becomes concrete in ingress, runtime envelopes, PH1.L/session storage, reconciliation state, and sync/re-entry behavior.
- D3 must leave proof, governance, runtime-law completion, and final verification closure to later phases.

C) DEPENDENCY RULE
- D3 consumes D1 and D2 as frozen law:
  - D1 freezes cross-device session authority, the canonical identity tuple, ordering law, and downstream boundaries.
  - D2 freezes canonical field names, attach outcome families, blocked outcome families, binding rules, and contract-level ownership vocabulary.
- D3 also consumes:
  - Section 02 Session Engine single-writer, device timeline, attach, transfer, and failover law
  - Section 05 persistence, reconciliation, replay, dedupe, and quarantine law
  - Section 07 identity/device binding law
  - Section 08 platform-context and device-ordering law
  - Section 09 and Section 11 only as downstream visibility/deferred integration boundaries
  - C4 storage-before-visibility and protected-completion law only as a later dependency, not as D3 implementation scope
- No D3 decision may weaken D2 contract semantics to match current gaps. Current gaps must be materialized additively or carried forward as explicit bounded gaps.

D) ARCHITECTURAL POSITION
- D3 sits between frozen contract law and later protected-completion integration.
- The canonical flow remains:
  - request / ingress normalization
  - runtime-envelope materialization
  - authoritative PH1.L/session-storage decision and write
  - persistence/reconciliation visibility update
  - later proof/governance/law visibility only where D4 requires it
- Session Engine / PH1.L remains the only canonical session writer.
- RuntimeExecutionEnvelope remains the canonical runtime-carried execution context.
- Persistence + Sync remains authoritative only for reconciliation posture and retry/re-entry handling, never for session truth itself.
- Platform Runtime and Identity + Voice continue to validate and normalize inputs without becoming session authority.

E) D1 / D2 ASSUMPTIONS CONSUMED
- D3 consumes the D1 canonical tuple:
  - `(session_id, turn_id, device_id, actor_identity_scope, platform_context, device_turn_sequence, owning_node_or_lease_ref)`
- D3 consumes D2 canonical contract fields:
  - `session_id`
  - `turn_id`
  - `device_id`
  - `device_turn_sequence`
  - `actor_identity_scope`
  - `platform_context`
  - `session_attach_outcome`
  - `attach_role`
  - `coordination_state`
  - `consistency_level`
  - `owning_node_or_lease_ref`
- D3 consumes D2 outcome families:
  - success/reuse: `NEW_SESSION_CREATED`, `EXISTING_SESSION_REUSED`, `EXISTING_SESSION_ATTACHED`, `RETRY_REUSED_RESULT`, `RECOVERY_ATTACH_ACCEPTED`, `DETACH_ACCEPTED`
  - blocked/fail-closed: `STALE_REJECTED`, `DUPLICATE_RETRY_REUSED`, `IDENTITY_SCOPE_BLOCKED`, `PLATFORM_CONTEXT_BLOCKED`, `OWNERSHIP_UNCERTAIN_BLOCKED`, `TRANSFER_PENDING_BLOCKED`, `RECOVERY_WINDOW_CLOSED_BLOCKED`, `DETACH_NOT_LAWFUL_BLOCKED`
- D3 consumes D2 contract law that request/envelope/storage normalization gaps must be made explicit and additive rather than silently redefined.

F) CURRENT RUNTIME AND PERSISTENCE SURFACES IN SCOPE
Current Repo Surface → D3 Wiring Scope Mapping
| repo surface | current role | authoritative / derived / evidence / decision | D3 relevance | notes / constraints |
| --- | --- | --- | --- | --- |
| `RuntimeExecutionEnvelope` | runtime-carried execution context | derived runtime contract, not storage authority | primary D3 runtime materialization surface | already carries `session_id`, `turn_id`, `device_identity`, `platform_context`, `device_turn_sequence`, `session_attach_outcome`, `persistence_state`, `governance_state`, `law_state` |
| `RuntimeExecutionEnvelope.session_attach_outcome` | attach/reuse visibility | derived runtime visibility | D3 must extend materialization to all D2 outcome families | current enum only covers success/retry reuse classes |
| `RuntimeExecutionEnvelope.persistence_state` | replay/reconciliation/recovery posture | authoritative only for persistence posture | D3 must wire blocked/replay/refusal outcomes into persistence visibility | must not become alternate session authority |
| `RuntimeExecutionEnvelope.governance_state` | downstream governance visibility | decision visibility only | D3 must preserve as downstream-only carriage | D4 integrates protected visibility |
| `RuntimeExecutionEnvelope.law_state` | downstream runtime-law visibility | decision visibility only | D3 must preserve as downstream-only carriage | D4 integrates protected visibility |
| `RuntimeExecutionEnvelope.actor_identity` / `device_identity` / `platform_context` | identity/device/platform binding | runtime-carried evidence of validated context | D3 must propagate these bindings consistently | platform and identity remain subordinate to session truth |
| `os_core.sessions` / `SessionRecord` | canonical current-row session truth | authoritative | primary D3 storage materialization surface | D3 must use PH1.L-owned current-row truth, not invent a second session writer |
| `SessionRecord.attached_devices` / `last_attached_device_id` | current attached-device scope | authoritative | D3 must materialize attach, recover, and detach participation changes here | attached-device set may widen, but prior devices may not be silently dropped |
| `SessionRecord.last_turn_id` / `active_turn_id` | turn continuity and in-flight ownership | authoritative | D3 must wire turn continuity into attach/resume/recover paths | `active_turn_id` requires active lease owner |
| `SessionRecord.device_turn_sequences` / `device_last_idempotency_keys` | per-device stale/retry/duplicate anchor | authoritative | D3 must reuse these for replay/stale/duplicate enforcement | monotonic and attached-device-scoped |
| `SessionRecord.lease_owner_id` / `lease_acquired_at` / `lease_expires_at` | current session ownership posture | authoritative | D3 must materialize D2 ownership-ref and coordination-state visibility from these fields | D3 may not invent a second session-lease subsystem |
| `Ph1fStore::upsert_session_lifecycle` | authoritative PH1.L current-row write path | authoritative | D3 write order must center on this class of session write | current path already enforces monotonicity, idempotency, transition validity, and attached-device discipline |
| `Ph1fStore::get_session` / `session_rows` | authoritative replay/re-entry readback | authoritative read | D3 must use these reads for re-entry and stale classification | runtime retries must reread canonical session truth |
| `PersistenceRecoveryMode` / `PersistenceAcknowledgementState` / `ReconciliationDecision` | persistence/reconciliation vocabulary | authoritative for persistence posture only | D3 must wire attach-related decisions into these families where applicable | Section 05 meanings are frozen |
| `PH1.LEASE` decision bundles | ownership/takeover decision posture | decision-only | D3 must surface D2 ownership / transfer / recovery contract from these outputs where needed | work-order lease law informs session ownership discipline |
| `PH1.OS` `os_policy_bundle` / `os_decision_bundle` | platform/orchestration visibility | decision-only | D3 may carry blocked/fail-closed outcome visibility here only when runtime already requires it | PH1.OS does not become session writer |
| `PH1.J` `audit.audit_events` | audit/proof visibility | evidence-only | D3 may identify where lifecycle-significant runtime events should emit visibility later | D4 owns protected proof completion |
| `PH1.GOV` and `PH1.LAW` state surfaces | governance and runtime-law visibility | decision-only | D3 must keep them as downstream-only integration surfaces | no D3 direct mutation of session truth through GOV/LAW |

G) CANONICAL WIRING TARGET
- D3 freezes one canonical materialization path for every D2 action:
  1. request / ingress carries the D2 canonical input tuple and intent
  2. runtime envelope is created with ingress-carried identity/device/platform/session context
  3. Session Engine resolves attach/resume/recover/detach against authoritative PH1.L truth
  4. PH1.L/session-storage commit or authoritative no-mutation refusal occurs
  5. runtime envelope is updated with canonical attach outcome and persistence posture derived from the authoritative result
  6. reconciliation/sync surfaces reuse the authoritative result for replay, retry, stale rejection, and recovery
- D3 freezes that authoritative session write/refusal happens before any downstream visibility is treated as meaningful.
- D3 also freezes the current repo-truth gaps that must be materialized additively:
  - request-facing explicit D2 field set is still implicit today
  - `RuntimeExecutionEnvelope` lacks first-class `attach_role`, `coordination_state`, `consistency_level`, and `owning_node_or_lease_ref`
  - PH1.L DB wiring is still narrower than `SessionRecord`
  - detach is not yet a first-class persisted session action surface

Canonical Field → Runtime / Persistence Materialization Matrix
| canonical field | request / ingress surface | runtime envelope surface | PH1.L / session storage surface | persistence / reconciliation surface | current gap if any | D3 materialization rule |
| --- | --- | --- | --- | --- | --- | --- |
| `session_id` | explicit attach/resume/recover request session ref | `RuntimeExecutionEnvelope.session_id` | `SessionRecord.session_id` / `os_core.sessions.session_id` | referenced in reconciliation decision context | request side still implicit | D3 must make request carriage explicit and keep storage truth cloud-owned |
| `turn_id` | ingress may carry prior turn context only as visibility | `RuntimeExecutionEnvelope.turn_id` | `SessionRecord.last_turn_id` / `active_turn_id` | carried into retry/re-entry reconciliation | no exact request packet today | D3 must preserve cloud turn authority and wire readback into replay/refusal |
| `device_id` | authenticated ingress device identity | `RuntimeExecutionEnvelope.device_identity` | `SessionRecord.device_id`, `attached_devices`, `last_attached_device_id` | used in reconciliation, stale, and duplicate classification | PH1.L DB wiring still looks single-device | D3 must materialize multi-device storage/API truth without narrowing back to one device |
| `device_turn_sequence` | per-device ingress counter | `RuntimeExecutionEnvelope.device_turn_sequence` | `SessionRecord.device_turn_sequences` | used by `ReconciliationDecision` / stale or retry posture | missing from PH1.L DB wiring | D3 must wire this end-to-end as required field for attach-related actions |
| `actor_identity_scope` | verified identity scope at ingress | `RuntimeExecutionEnvelope.actor_identity` | current PH1.L row stores `user_id`; explicit scope ref is still a gap | may influence reconciliation refusal context only | exact persisted scope field is absent | D3 must propagate verified scope through runtime and preserve PH1.L authority without inventing alternate identity storage |
| `platform_context` | normalized ingress platform posture | `RuntimeExecutionEnvelope.platform_context` | no PH1.L authoritative storage column | may influence persistence refusal / fresh-state request | storage field absent by design | D3 must keep platform context runtime-visible and non-authoritative |
| `session_attach_outcome` | response/result-facing outcome family | `RuntimeExecutionEnvelope.session_attach_outcome` | no authoritative storage field; derived from session decision result | may also be mirrored into reconciliation visibility | current enum is incomplete | D3 must extend materialization to the full D2 semantic family |
| `attach_role` | ingress intent / granted access class | additive envelope field required | additive PH1.L/session scope field or derived current-row visibility required | may influence reconciliation refusal visibility | missing today | D3 must materialize explicit runtime/storage carriage without redefining Section 02 values |
| `coordination_state` | requester never authors; cloud returns it | additive envelope visibility field required | derived from `lease_owner_id` plus recovery/transfer posture and later explicit storage materialization | may influence `RequestFreshSessionState` / quarantine / retry decisions | missing today | D3 must materialize explicit runtime visibility and persisted derivation source |
| `consistency_level` | optional requester hint, cloud-owned final value | additive envelope visibility field required | derived current-row or persisted visibility field required | used in re-entry and degraded recovery visibility | missing today | D3 must materialize cloud-decided level and keep it subordinate to authoritative session truth |
| `owning_node_or_lease_ref` | request never authors it | additive envelope visibility field required | `SessionRecord.lease_owner_id` and lease timing fields as authoritative source | may influence recovery / retry / transfer visibility only | no exact normalized alias today | D3 must surface an additive alias over authoritative session ownership truth |

H) REQUEST / INGRESS WIRING
- Current repo truth does not yet expose a single D2-normalized request packet. D3 therefore must materialize the frozen canonical field set at ingress before runtime envelope construction.
- D3 request / ingress wiring rules:
  - `session_id`, `device_id`, `device_turn_sequence`, actor identity, and platform context are normalized before Session Engine decisioning
  - attach/resume/recover/detach intent is carried explicitly rather than inferred from absent fields
  - missing required attach-related fields fail closed before any PH1.L mutation attempt
  - client-originated values for `coordination_state`, `consistency_level`, or `owning_node_or_lease_ref` must be ignored or rejected because they are cloud-authored visibility fields
- D3 must not let ingress materialization invent a second authority path. Ingress is only the normalized proposal surface that feeds the authoritative Session Engine decision.

I) RUNTIME EXECUTION ENVELOPE WIRING
- `RuntimeExecutionEnvelope` is the canonical D3 runtime-carried surface.
- Current repo truth already supports staged envelope materialization through:
  - `v1_with_platform_context_device_turn_sequence_and_attach_outcome`
  - `v1_with_platform_context_device_turn_sequence_attach_outcome_and_persistence_state`
  - `v1_with_platform_context_device_turn_sequence_attach_outcome_persistence_and_governance_state`
  - `with_session_device_turn_and_attach_outcome`
  - `with_persistence_state`
  - `with_governance_state`
- D3 must extend envelope wiring additively so the frozen D2 contract is visible without reinterpretation:
  - add explicit carriage for `attach_role`
  - add explicit carriage for `coordination_state`
  - add explicit carriage for `consistency_level`
  - add explicit carriage for `owning_node_or_lease_ref`
  - expand `session_attach_outcome` materialization to D2 blocked/recovery/detach semantic families
- Envelope write order in D3:
  - ingress fields are copied in as non-authoritative request context
  - PH1.L authoritative session result is applied next
  - persistence/reconciliation state is applied only after the authoritative session result is known
  - governance/law/proof fields remain downstream-only until D4

J) PH1.L / SESSION STORAGE WIRING
- PH1.L/session storage remains authoritative for session truth.
- D3 must materialize D2 contract semantics in PH1.L/session storage through `SessionRecord` and the `upsert_session_lifecycle` write family.
- Required D3 storage-wiring rules:
  - attach/resume/recover decisions update or reuse `SessionRecord` rather than creating parallel state
  - `attached_devices` and `last_attached_device_id` must reflect authoritative participation outcomes
  - `last_turn_id` and `active_turn_id` remain the canonical turn continuity anchors
  - `device_turn_sequences` and `device_last_idempotency_keys` remain the authoritative stale/retry/duplicate anchors
  - `lease_owner_id`, `lease_acquired_at`, and `lease_expires_at` remain the authoritative ownership posture anchors
  - D3 must normalize PH1.L DB wiring/documented shape to the real `SessionRecord` field set rather than narrowing the real storage truth
- D3 may add explicit storage fields or projections for D2 additive visibility concepts, but only if they remain subordinate to PH1.L canonical session truth.

K) PERSISTENCE / RECONCILIATION / SYNC WIRING
- Section 05 already freezes cloud-wins reconciliation, fresh-session-state requests, stale rejection, and cross-device dedupe.
- D3 must connect D2 attach-related actions to existing persistence vocabulary:
  - `PersistenceRecoveryMode`
  - `PersistenceAcknowledgementState`
  - `ReconciliationDecision`
  - `PersistenceConflictSeverity`
- D3 persistence rules:
  - reuse prior authoritative outcome on same-sequence retry
  - reject stale operations when per-device sequence is lower than authoritative truth
  - request fresh session state when reconnect/replay cannot prove safe reuse
  - quarantine local state only when persistence inconsistency is the dominant posture
  - never let reconciliation overwrite canonical session truth
- D3 sync/re-entry behavior must always re-read authoritative PH1.L/session truth first, then repair downstream visibility or local state.

Attach / Resume / Recover / Detach Wiring Matrix
| action | authoritative writer | runtime surfaces written | persistence surfaces written | blocked / fail-closed outcome materialized where | notes |
| --- | --- | --- | --- | --- | --- |
| attach | Session Engine via PH1.L current-row update | envelope `session_id`, `device_turn_sequence`, `session_attach_outcome`, additive role/coordination/consistency/ownership visibility | `SessionRecord.attached_devices`, `last_attached_device_id`, `device_turn_sequences`, `device_last_idempotency_keys`, turn/lease posture as needed | envelope outcome plus persistence reconciliation/refusal posture | may create new session or attach to existing one, but always through authoritative PH1.L result |
| resume | Session Engine via PH1.L current-row reuse/update | envelope `session_id`, `session_attach_outcome`, persistence visibility, additive consistency/ownership fields | `SessionRecord` reused and advanced only if lawful | envelope and persistence visibility carry blocked reuse/closed-window cases | resume never mints a new canonical session when reuse is lawful |
| recover | Session Engine via PH1.L plus authoritative lease/recovery posture | envelope `session_attach_outcome`, `persistence_state`, additive coordination/consistency/ownership fields | `SessionRecord` and authoritative recovery/lease posture only where lawful | blocked outcomes reflected in envelope and persistence state | recover is stricter than resume and must materialize degraded or blocked posture explicitly |
| detach | Session Engine via PH1.L participation-scope update when implemented | envelope detach outcome family and additive role/coordination visibility | authoritative participation scope change in `SessionRecord` or additive detach record materialized from PH1.L truth | envelope and persistence visibility surface `DETACH_NOT_LAWFUL_BLOCKED` or accepted outcome | current repo truth lacks first-class detach action surface; D3 must materialize it additively without redefining semantics |

L) ATTACH OUTCOME / BLOCKED OUTCOME MATERIALIZATION
- D3 must materialize both success and blocked D2 semantic families in runtime and persistence visibility.
- Outcome materialization rules:
  - authoritative success or reuse comes from PH1.L/session truth
  - blocked/fail-closed outcomes preserve existing authoritative session truth and must still be visible in runtime/persistence surfaces
  - D3 may not collapse blocked outcomes into generic failure classes
  - D3 may not treat `session_attach_outcome` as authoritative session truth; it is visibility of an authoritative decision

Attach Outcome / Blocked Outcome Matrix
| outcome family | authoritative source | runtime visibility surface | persistence visibility surface | current gap if any | downstream integration note |
| --- | --- | --- | --- | --- | --- |
| `NEW_SESSION_CREATED` | PH1.L `SessionRecord` create/commit | `session_attach_outcome` plus `session_id` in envelope | authoritative acknowledgement / replay-safe persistence posture | no semantic gap; only exact materialization path | D4 later adds protected visibility only where required |
| `EXISTING_SESSION_REUSED` | PH1.L current-row reuse | `session_attach_outcome` | reconciliation visibility may indicate authoritative reuse | current enum already supports this | D3 must preserve same `session_id` |
| `EXISTING_SESSION_ATTACHED` | PH1.L update of attached-device scope | `session_attach_outcome` plus additive role/consistency/coordination fields | acknowledgement and replay visibility | current enum already supports attach success | D3 must materialize multi-device storage truth |
| `RETRY_REUSED_RESULT` / `DUPLICATE_RETRY_REUSED` | PH1.L plus device sequence/idempotency anchors | `session_attach_outcome` or bounded retry outcome family | `ReconciliationDecision::ReusePriorAuthoritativeOutcome` | blocked duplicate class not yet explicit | D3 must extend visibility without double mutation |
| `RECOVERY_ATTACH_ACCEPTED` | PH1.L plus authoritative recovery posture | additive outcome visibility in envelope | recovery mode / reconciliation visibility | missing explicit outcome family today | D3 must add explicit recovery outcome materialization |
| `DETACH_ACCEPTED` | PH1.L participation-scope change | additive detach outcome visibility | acknowledgement/refusal visibility | no first-class detach action surface today | D3 must materialize additively |
| blocked families (`STALE_REJECTED`, `IDENTITY_SCOPE_BLOCKED`, `PLATFORM_CONTEXT_BLOCKED`, `OWNERSHIP_UNCERTAIN_BLOCKED`, `TRANSFER_PENDING_BLOCKED`, `RECOVERY_WINDOW_CLOSED_BLOCKED`, `DETACH_NOT_LAWFUL_BLOCKED`) | preserved prior authoritative session truth | explicit blocked outcome in envelope/result surface | persistence acknowledgement / reconciliation / quarantine visibility as appropriate | blocked outcome family not fully materialized today | D4 later decides protected escalation only |

M) IDEMPOTENCY, STALE, RETRY, AND RE-ENTRY WIRING
- D3 must reuse existing authoritative session fields for replay-safe behavior.
- Frozen D3 idempotency/re-entry law:
  - same `(session_id or session ref, device_id, device_turn_sequence)` family reuses authoritative result
  - lower device sequence rejects stale operation
  - reconnect after partial success rereads PH1.L first and only repairs missing visibility or local sync state
  - simultaneous cross-device submission resolves through authoritative session write order and ownership posture
- D3 must not invent alternate local dedupe or device-only reconciliation truth.

Idempotency / Stale / Retry / Duplicate Wiring Matrix
| case | authoritative source | ordering basis | runtime handling | persistence handling | re-entry behavior | notes |
| --- | --- | --- | --- | --- | --- | --- |
| attach | `SessionRecord.device_turn_sequences` and `device_last_idempotency_keys` | `(session ref, device_id, device_turn_sequence)` | runtime envelope carries new/retry/stale outcome explicitly | persistence marks acknowledged, stale, or retry reuse posture | reread authoritative session row before any retry mutation | attach may create or join, but never twice for one device sequence |
| resume | authoritative session row plus per-device sequence | same tuple, anchored to existing `session_id` | runtime emits reuse or blocked resume outcome | persistence reuses prior authoritative outcome or requests fresh session state | re-entry never mints a second canonical session | same-sequence resume must reuse |
| recover | authoritative session plus ownership/recovery posture | same tuple plus current recovery posture | runtime emits accepted recovery, blocked recovery, or degraded recovery posture | persistence uses recovery mode and reconciliation decision families | reread session row and active lease posture before any recovery retry | no client-side recovery inference |
| detach | authoritative participation scope plus device sequence or detach idempotency family | `(session_id, device_id, sequence or detach idempotency key)` | runtime emits accepted or blocked detach outcome | persistence reflects acknowledgement/refusal only | reread authoritative participation scope before repeated detach | exact detach idempotency placement is D3 work, not D2 redesign |
| retry of same device sequence | `SessionRecord.device_turn_sequences` and last idempotency key map | equal sequence for same device | reuse prior runtime outcome | `ReusePriorAuthoritativeOutcome` | repair visibility only | no second authoritative mutation |
| stale device message | `SessionRecord.device_turn_sequences` | lower sequence for same device | explicit stale refusal outcome | `StaleRejected` / `RejectStaleOperation` | caller must request fresh session state | stale messages never reopen session state |
| simultaneous multi-device submission | authoritative session write order plus lease posture | cloud-authoritative commit order and per-device sequence | losing runtime request gets blocked/reuse/fresh-state outcome | persistence follows authoritative winner | reread canonical owner and active turn | no device-local arbitration |
| reconnect after partial success | authoritative PH1.L row first, then persistence state | existing authoritative commit plus visibility lag | runtime reuses committed outcome | persistence repairs missing acknowledgement/decision posture | only downstream visibility repair may replay | aligns with C4 storage-before-visibility law |
| cross-device resume conflict | authoritative session plus lease/coordination state | session truth plus `coordination_state` / `consistency_level` | runtime emits blocked or reused canonical outcome | persistence may request fresh session state or quarantine local state | no duplicate authoritative resume decision | `TRANSFER_PENDING` and `OWNERSHIP_UNCERTAIN` remain explicit |

N) OWNERSHIP, LEASE, FAILOVER, AND TRANSFER WIRING
- D3 freezes wiring, not new ownership algorithms.
- D3 ownership rules:
  - `SessionRecord.lease_owner_id`, `lease_acquired_at`, and `lease_expires_at` remain the authoritative session ownership anchor
  - PH1.LEASE decision bundles remain decision-only but inform runtime/persistence materialization of ownership visibility
  - `coordination_state`, `consistency_level`, and `owning_node_or_lease_ref` must be derived from authoritative ownership/recovery truth and then surfaced through runtime envelope and persistence visibility
  - `TRANSFER_PENDING`, `FAILOVER_RECOVERING`, and `OWNERSHIP_UNCERTAIN` must be explicitly materialized rather than implied

Ownership / Lease / Failover / Transfer Wiring Matrix
| concern | authoritative source | runtime surface(s) | persistence surface(s) | bounded gap if any | forbidden inference | downstream impact |
| --- | --- | --- | --- | --- | --- | --- |
| active owner visibility | `SessionRecord.lease_owner_id` and lease times | additive `owning_node_or_lease_ref`, `coordination_state`, `consistency_level` | `SessionRecord` authoritative fields plus additive derived visibility if needed | no single normalized alias today | local device or platform posture may not claim ownership | D4 only adds protected visibility, not ownership meaning |
| transfer in progress | authoritative session truth plus PH1.LEASE decision posture | additive `coordination_state=TRANSFER_PENDING` | persisted/derived transfer visibility sourced from session/lease truth | no first-class transfer row today | `session_attach_outcome` may not fabricate transfer truth | D5 later verifies transfer/refusal paths |
| failover recovering | authoritative session truth plus recovery/lease posture | additive `coordination_state=FAILOVER_RECOVERING`, degraded `consistency_level` when required | persistence recovery mode plus authoritative session truth | current repo lacks one dedicated failover row | persistence visibility alone may not claim recovered ownership | D4 may later add protected escalation |
| ownership uncertainty | authoritative session truth preserved; no new owner until explicit | additive `coordination_state=OWNERSHIP_UNCERTAIN` and blocked outcome | persistence may request fresh state or quarantine local state | explicit materialization missing today | stale replay or client retry may not clear uncertainty | D3 must make the blocked path explicit |
| resume-from-ledger-required posture | PH1.LEASE decision output | runtime visibility beside recovery outcome | persistence re-entry / fresh-state request posture | session-specific normalization still a gap | requester may not override | D4 later decides protected-completion implications |

O) FAILURE, REFUSAL, AND SAFE-FAIL WIRING
- D3 must wire the D2 blocked/refused model into concrete runtime and persistence surfaces.
- Safe-fail rules:
  - preserve current authoritative session truth
  - materialize explicit blocked/refused outcome
  - update runtime/persistence visibility deterministically
  - never continue by guessing, silently degrading, or inventing local ownership/session truth
- D3 must wire these failure/refusal surfaces now even when D4 later adds protected proof/governance/law completion.

Failure / Refusal / Safe-Fail Wiring Matrix
| case | authoritative truth preserved | runtime surface updated | persistence surface updated | blocked / fail-closed rule | downstream defer note |
| --- | --- | --- | --- | --- | --- |
| missing required ingress field for attach/resume/recover/detach | yes | explicit blocked outcome in runtime envelope/result | optional persistence refusal visibility only; no session mutation | fail closed before PH1.L mutation attempt | D4 only if protected visibility later required |
| actor identity mismatch | yes | `IDENTITY_SCOPE_BLOCKED` materialized in runtime outcome | persistence refusal / stale/fresh-state posture as appropriate | no identity rebinding | D4 may add protected governance/law visibility |
| platform-context mismatch | yes | `PLATFORM_CONTEXT_BLOCKED` materialized in runtime outcome | persistence refusal visibility as appropriate | no platform-based continuation | D4 may add protected visibility only |
| stale device sequence | yes | explicit stale outcome | `STALE_REJECTED` / `RejectStaleOperation` | no session mutation | D5 later verifies stale-path evidence |
| duplicate retry after prior authoritative success | prior authoritative truth reused | runtime reuse outcome | persistence reuse / acknowledged posture | no duplicate mutation | D4 only if protected completion visibility later needed |
| ownership uncertain or transfer unresolved | yes | explicit blocked outcome plus coordination/consistency visibility | fresh-state request, quarantine, or bounded retry posture | fail closed until ownership certainty returns | D4 later integrates protected escalation |
| recovery window closed | yes | explicit blocked recovery outcome | persistence refusal / fresh-state request posture | no forced resume | D4 later adds protected visibility only |
| detach not lawful for current role/state | yes | explicit detach-blocked outcome | refusal visibility only | no hidden detach or role downgrade | D5 later verifies detach-block evidence |
| runtime wrote non-authoritative visibility before authoritative session decision | authoritative truth must still prevail | runtime must be corrected to authoritative result | persistence must repair visibility only | storage-before-visibility law applies | D4 later governs protected-completion cases |

P) D3 → D4 / D5 FREEZE BOUNDARY
- D3 freezes runtime/persistence materialization and write order.
- D4 may add protected proof/governance/law integration only.
- D5 may verify and close only.
- Neither D4 nor D5 may reinterpret D2 contract semantics or D3 wiring order.

D3 → D4 / D5 Boundary Matrix
| concern | frozen in D3 | deferred to D4 | deferred to D5 | rationale |
| --- | --- | --- | --- | --- |
| request -> envelope -> PH1.L/session-storage -> persistence write order | yes | no | no | D3 is the materialization phase and must freeze wiring order |
| exact runtime and storage placement of D2 canonical fields | yes | no | no | D4/D5 may not guess field placement later |
| blocked/refused outcome materialization into runtime/persistence surfaces | yes | no semantic redesign; only protected integration | verification only | D3 must make blocked paths concrete before D4 can govern them |
| session authority vs derived visibility split | yes | no | no | D1/D2 law must remain intact through implementation |
| replay/re-entry/stale/duplicate reuse of authoritative session truth | yes | no semantic redesign; only protected visibility | verification only | D3 owns runtime/persistence reuse wiring |
| ownership / lease / failover / transfer field materialization | yes | protected escalation / law visibility only | verification only | D4 may observe and govern, not redefine the materialized ownership model |
| proof / governance / runtime-law completion requirements | no | yes | verification only | D4 owns protected integration under frozen D3 wiring |
| tests, docs, evidence pack, residual-risk closure | no | no | yes | D5 remains verification and closure only |

Q) COMPLETION CRITERIA
- D3 is complete when this plan baseline makes all of the following explicit:
  - current runtime and persistence surfaces in scope are enumerated and authority-classed
  - every D2 canonical field has a runtime/persistence materialization rule
  - request -> envelope -> PH1.L/session-storage -> persistence ordering is explicit
  - attach, resume, recover, and detach all have explicit authoritative writers and runtime/persistence write targets
  - success outcomes and blocked outcomes are both materially wired
  - stale/retry/duplicate/re-entry behavior explicitly reuses authoritative session truth
  - ownership / lease / failover / transfer visibility is explicitly derived from authoritative sources rather than invented
  - failure/refusal/safe-fail behavior is explicit and deterministic
  - D4 and D5 boundaries are explicit enough that neither phase can reinterpret D3 semantics later
- D3 does not require code, tests, CI, or docs updates beyond this plan file.
