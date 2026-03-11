PHASE C3 — MEMORY RETENTION / PURGE / DELETE ENFORCEMENT BUILD PLAN

A) REPO TRUTH CHECK
- `git status --short`: empty
- current branch: `main`
- repo-truth snapshot HEAD at review start: `bd27a1bbc8ac9624fa41b4bb597948cd65741bb0`
- snapshot semantics: this value records the repo state reviewed at the start of this pass; it is not required to equal the later commit hash that records this document update.
- target file at review start: missing; C3 plan must be created as a new single-file docs artifact
- exact files reviewed:
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C1_LIFECYCLE_MODEL_REVIEW.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C2_WAKE_ARTIFACT_LIFECYCLE_WORKERS_BUILD_PLAN.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/12_MEMORY_ARCHITECTURE.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_01.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_05.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_06.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_11.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_M.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_KG.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_KNOW.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_PRUNE.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_J.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_GOV.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_LAW.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1kg.rs`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1know.rs`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1learn.rs`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/repo.rs`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/lib.rs`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1f.rs`

B) PURPOSE
- C3 defines the design build plan for memory retention, archive, tombstone, delete, purge, and restore enforcement only.
- C3 consumes the frozen C1 `MEMORY_LIFECYCLE` model and Build Section 06 memory law without redesigning either.
- C3 freezes:
- which memory lifecycle subjects exist
- which PH1.M-owned surfaces are authoritative
- which surfaces are derived, index, eligibility, packaging, or evidence only
- which worker families own which lifecycle actions
- how hold, expiry, tombstone, purge, and restore interact
- how re-entry, replay, and conflict refusal behave
- which proof, governance, and law visibility is required inside C3 scope
- C3 does not implement code, redesign PH1.M architecture, reopen C2 wake worker design, or absorb C4/C5 closure work.

C) DEPENDENCY RULE
- C3 precedence is:
- frozen C1 lifecycle model for lifecycle vocabulary, scope, terminality, and reverse-authority law
- Build Section 06 and Core Architecture for memory authority, ledger-first law, identity isolation, and retention governance
- PH1.M DB wiring and current storage code for exact repo-truth surfaces and current gaps
- frozen C2 plan for explicit non-overlap with wake artifact lifecycle workers
- If repo truth is weaker than the frozen lifecycle law, C3 must name the gap explicitly and bound the safest implementation assumption.
- If repo truth is stronger than older narrative text, C3 must anchor to the stronger current truth without changing C1 semantics.
- No C3 worker may create a second memory authority path, a device-side delete authority path, or a reverse-authority path from graph, bundle, or audit surfaces back into PH1.M lifecycle truth.

D) ARCHITECTURAL POSITION
- PH1.M is the authoritative owner of identity-scoped memory truth.
- PH1.F remains the storage spine enforcing append-only ledgers, rebuildable current views, and deterministic keys.
- PH1.J remains the authoritative audit and proof ledger surface for reason-coded lifecycle visibility.
- PH1.GOV and PH1.LAW remain decision/visibility engines only; neither becomes the writer of memory lifecycle state in C3 scope.
- PH1.KG, PH1.KNOW, and PH1.PRUNE remain non-authoritative assist engines; their outputs are bounded hints, not lifecycle truth.
- Engine B remains the outbox/mirror delivery path for cross-device consistency; C3 may emit deltas to that path but does not redesign outbox mechanics.
- C3 worker ownership is planning-level ownership over memory lifecycle enforcement. It does not require five new binaries, queues, or storage subsystems; a worker family may execute inline with an authoritative commit, as a background sweeper, or both, so long as the ownership boundary is explicit and replay-safe.
- Client devices may request deletion, forget, or restore flows, but client devices must never authoritatively delete or restore cloud-owned memory state.

E) C1 / SECTION-06 MEMORY ASSUMPTIONS CONSUMED
- C3 consumes the frozen C1 rule that `MEMORY_LIFECYCLE` is distinct from `ARTIFACT_IDENTITY_LIFECYCLE`, `TARGET_DEPLOYMENT_LIFECYCLE`, and `RUNTIME_USE_LIFECYCLE`.
- C3 consumes the frozen canonical lifecycle-subject identity tuple:
- `(lifecycle_plane, subject_class, subject_id, subject_scope, lineage_root_or_parent_ref)`
- For C3, the lifecycle plane is always `MEMORY_LIFECYCLE`.
- C3 consumes Section 06 law that the memory ledger is the single authority for memory state and that materialized views are read-optimized derivatives only.
- C3 consumes Section 06 and Core Architecture law that memory access is identity-scoped, session-bound for execution, cloud-authoritative, policy-validated, and replay-safe.
- C3 consumes Section 06 trust and provenance law:
- `VERIFIED`, `HIGH_CONFIDENCE`, `LOW_CONFIDENCE`, and `UNVERIFIED` memory trust levels remain authoritative metadata that can influence retention, archive, restore, and eligibility decisions
- provenance metadata (`identity_scope`, `source_session_id`, `source_turn_id`, confidence, sensitivity, retention class) must stay attached to authoritative lifecycle decisions
- low-confidence or stale memory may be demoted, expired, archived, or clarified, but derived trust hints may not override authoritative lifecycle state
- C3 consumes C1 terminality law:
- tombstone/delete request is not the same as purge
- purge is terminal
- restore may not reopen a purged same-subject record
- hold blocks purge/delete terminalization
- graph, archive pointers, bundle outputs, and proof surfaces must not reverse-authorize memory truth
- C3 consumes Section 06 graph law:
- graph relationships are bounded retrieval aids
- graph writes follow ledger-first authority
- graph retrieval respects eligibility and policy
- C3 consumes Section 06 deletion law:
- deletion must be explicit, governed, and auditable
- no silent removal
- Repo-truth ambiguity that C3 must preserve explicitly:
- current PH1.M storage surfaces encode explicit `Forgotten` / `ThreadForgotten` tombstone-style behavior, retention mode, and archive pointers
- current PH1.M storage surfaces do not yet expose standalone legal-hold / preservation-hold rows, explicit `Purged` ledger events, or explicit `Restored` ledger events
- safest C3 planning assumption:
- hold, purge, and restore must be implemented as additive PH1.M-owned lifecycle markers or ledger events inside the existing memory domain
- until those markers exist, no worker may infer hold/purge/restore authority from current views, graph rows, audit rows, or device receipts

F) CURRENT MEMORY SURFACES IN SCOPE
- Current PH1.M repo truth already separates authoritative ledger streams from derived current/index surfaces:
- authoritative event streams exist for atoms, thread digests, and emotional threads
- rebuildable current views exist for atoms, threads, and emotional continuity
- bounded config/current surfaces exist for suppression rules and retention preference
- graph nodes, graph edges, thread refs, and archive pointers exist as retrieval/index aids
- PH1.M bundle outputs, PH1.KG outputs, and PH1.KNOW outputs remain advisory/derived only
- C3 must enforce lifecycle on top of this split rather than collapsing it.
- Exact current PH1.M lifecycle markers/events already represented in repo truth are:
- atom lifecycle events: `Stored`, `Updated`, `Forgotten`
- thread lifecycle events: `THREAD_DIGEST_UPSERT`, `THREAD_RESOLVED`, `THREAD_FORGOTTEN`
- emotional lifecycle updates through `emotional_threads_ledger`
- authoritative config markers: `memory_suppression_rules`, `memory_retention_preferences`
- archive pointer markers: `memory_archive_index`
- Exact current gaps that C3 must close without inventing parallel architecture are:
- no standalone PH1.M legal-hold / preservation-hold surface yet
- no standalone PH1.M purge-terminal marker yet
- no standalone PH1.M restore marker yet

Current Repo Surface → C3 Memory Scope Mapping
| repo surface | memory subject class | lifecycle role | authoritative / derived / evidence-only | C3 worker relevance | notes / constraints |
| --- | --- | --- | --- | --- | --- |
| `memory.memory_atoms_ledger` | `MEMORY_ATOM_SUBJECT` | append-only atom lifecycle truth for `Stored | Updated | Forgotten` | authoritative | retention evaluation, delete/tombstone, purge, restore eligibility | current repo truth already treats `Forgotten` as tombstone-style removal from active use; no explicit purge event exists yet |
| `memory.memory_atoms_current` | `MEMORY_ATOM_CURRENT_VIEW` | active atom retrieval view with `active` and `expires_at` posture | derived | propagation target, rebuild target, eligibility input | must be recomputable from ledger; must never be treated as delete or restore authority |
| `memory.memory_threads_ledger` | `MEMORY_THREAD_SUBJECT` | append-only thread lifecycle truth for `THREAD_DIGEST_UPSERT | THREAD_RESOLVED | THREAD_FORGOTTEN` | authoritative | retention evaluation, archive/expiry handling, delete/tombstone, purge, restore eligibility | unresolved and pinned retention behavior is carried here through digest + event kind |
| `memory.memory_threads_current` | `MEMORY_THREAD_CURRENT_VIEW` | current continuity digest plus unresolved deadline / last-used posture | derived | propagation target, sweeper scan surface, resume eligibility input | current view is operationally useful but must never reverse-authorize thread lifecycle truth |
| `memory.emotional_threads_ledger` | `EMOTIONAL_THREAD_SUBJECT` | append-only emotional continuity lifecycle truth | authoritative | retention evaluation, delete/tombstone, purge eligibility | tone continuity only; must never become factual authority |
| `memory.emotional_threads_current` | `EMOTIONAL_THREAD_CURRENT_VIEW` | current emotional continuity projection | derived | propagation target, cleanup target | exposure gating remains required before composition |
| `memory.memory_suppression_rules` | `MEMORY_SUPPRESSION_RULE_CONFIG` | authoritative suppression and do-not-store config | authoritative config | read/write relevance for explicit user control commands and delete follow-up policy | suppression may block surfacing or future storage; suppression is not itself delete/purge authority |
| `memory.memory_retention_preferences` | `MEMORY_RETENTION_POLICY_SUBJECT` | authoritative user-scoped retention-mode preference | authoritative config | retention evaluation input and preference-change trigger | current repo truth covers `DEFAULT | REMEMBER_EVERYTHING`; this is policy/config truth, not subject lifecycle history |
| `memory.memory_thread_refs` | `MEMORY_THREAD_REF_POINTER` | bounded evidence pointer set for thread continuity | derived pointer set | cleanup target after tombstone/purge; restore support read-only | references only; no raw transcript dump; must never recreate deleted thread truth |
| `memory.memory_graph_nodes` + `memory.memory_graph_edges` | `MEMORY_GRAPH_RELATIONSHIP_RECORD` | retrieval/index graph for memory continuity | derived index | cleanup target, eligibility read input, rebuild target | graph is an index, not truth; graph rows must follow parent-authoritative lifecycle state |
| `memory.memory_archive_index` | `MEMORY_ARCHIVE_POINTER_SUBJECT` | pointer/index metadata for archive page-in candidates | derived pointer index with memory-owned archival meaning | archive/restore relevance and cleanup target | current repo truth stores pointers only; there is no standalone archive payload ledger in this slice |
| `Ph1mHintBundleBuildResponse`, `Ph1mContextBundleBuildResponse`, `Ph1mResumeSelectResponse` | `MEMORY_ELIGIBILITY_DERIVATIVE` | derived hint/resume/context eligibility outputs | derived | read-only downstream surface that must reflect authoritative lifecycle state | bundle outputs must not reverse-authorize retention/delete/restore truth |
| `KgFactBundleSelectOk` and `KnowHintBundleSelectOk` | `PACKAGING_OR_ASSIST_DERIVATIVE` | advisory grounding / vocabulary packaging side effects | derived | read-only invalidation boundary after tombstone/purge | PH1.KG and PH1.KNOW remain advisory only and must not mutate memory lifecycle |
| `audit.audit_events` plus PH1.J reason-coded memory emissions | `MEMORY_LIFECYCLE_EVIDENCE` | lifecycle visibility and proof/audit trace overlay | evidence-only | required visibility for lifecycle-significant actions | authoritative for audit only; must never become deletion or restore authority |
| `os_core.work_orders_current` | `EXTERNAL_CONTINUITY_DEPENDENCY` | pending-work authority read used by resume/suppression logic | external authoritative read, not PH1.M memory truth | read-only boundary for continuity cleanup after thread tombstone/delete | WorkOrder state remains authoritative for pending work; memory may assist recall or suppression only |

G) CANONICAL MEMORY SUBJECTS AND LIFECYCLE ACTIONS IN C3 SCOPE
- C3 memory lifecycle subject classes are:
- `MEMORY_ATOM_SUBJECT`
- `MEMORY_THREAD_SUBJECT`
- `EMOTIONAL_THREAD_SUBJECT`
- `MEMORY_RETENTION_POLICY_SUBJECT`
- `MEMORY_HOLD_MARKER_SUBJECT`
- `MEMORY_ARCHIVE_POINTER_SUBJECT`
- `MEMORY_GRAPH_RELATIONSHIP_RECORD`
- `MEMORY_THREAD_REF_POINTER`
- `MEMORY_ELIGIBILITY_DERIVATIVE`
- `PACKAGING_OR_ASSIST_DERIVATIVE`
- C3 lifecycle actions in scope are:
- retain / retain-evaluate
- archive
- hold attach / hold release
- tombstone
- delete request
- purge
- restore
- C3 must keep a hard distinction between:
- canonical lifecycle action on an authoritative memory subject
- derived projection cleanup after that action
- read-only eligibility or packaging side effects after that action
- Safest repo-truth interpretation for missing explicit hold/purge/restore rows:
- `MEMORY_HOLD_MARKER_SUBJECT` is required by frozen C1 law but not yet first-class in current PH1.M storage
- C3 must plan it as a PH1.M-owned authoritative subject, not a graph tag, audit-only note, UI flag, or runtime-law inference

Memory Subject / Lifecycle Action Applicability Matrix
| memory subject class | retain | archive | hold | tombstone | delete request | purge | restore | notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `MEMORY_ATOM_SUBJECT` | YES | YES when policy moves atom out of active retrieval but keeps lawful restore lineage | YES | YES | YES | YES | YES when not purged and policy allows | current repo truth already supports `Stored | Updated | Forgotten`; archive/purge/restore are the additive C3 lifecycle completion path |
| `MEMORY_THREAD_SUBJECT` | YES | YES | YES | YES | YES | YES | YES when thread lineage and policy allow | pinned/unresolved rules materially affect retention and restore eligibility |
| `EMOTIONAL_THREAD_SUBJECT` | YES | SOMETIMES | YES | YES | YES | YES | SOMETIMES | emotional continuity remains memory-owned but must never mutate factual authority |
| `MEMORY_ATOM_CURRENT_VIEW` | NO authoritative action | NO authoritative action | NO authoritative action | NO authoritative action | NO authoritative action | NO authoritative action | NO authoritative action | derived surface only; follows authoritative parent lifecycle |
| `MEMORY_THREAD_CURRENT_VIEW` | NO authoritative action | NO authoritative action | NO authoritative action | NO authoritative action | NO authoritative action | NO authoritative action | NO authoritative action | derived surface only; current resume posture must be recomputed after authoritative writes |
| `MEMORY_GRAPH_RELATIONSHIP_RECORD` | NO direct authority | NO direct authority | NO direct authority | derived cleanup only | NO direct authority | derived cleanup only | derived rebuild only | graph rows may exist only while parent authoritative subjects remain eligible |
| `MEMORY_ARCHIVE_POINTER_SUBJECT` | NO | YES | YES when protected retention blocks purge of the pointer target | NO direct authority | NO direct authority | YES as derived terminal cleanup after canonical purge | YES as pointer restoration after lawful restore | pointer/index metadata only; cannot itself authorize archive or restore of the parent subject |
| `MEMORY_HOLD_MARKER_SUBJECT` | NO | NO | YES | NO | YES as blocking input | YES as blocking input | YES as blocking input | frozen C1 requires legal/preservation hold semantics; current repo lacks a first-class PH1.M hold row, so C3 must add one inside PH1.M ownership |
| `MEMORY_RETENTION_POLICY_SUBJECT` | YES | YES as policy input | YES as policy input | NO direct authority | YES as policy input | YES as policy input | YES as policy input | current repo truth is `memory_retention_mode`; additional lifecycle policy markers must stay PH1.M-owned |
| `MEMORY_THREAD_REF_POINTER` | NO | derived pointer maintenance only | NO | derived cleanup only | NO | derived cleanup only | derived rebuild only | evidence pointers never authorize retention or deletion |
| `MEMORY_ELIGIBILITY_DERIVATIVE` | NO | NO | NO | NO | NO | NO | NO | derived retrieval/injection outputs only |
| `PACKAGING_OR_ASSIST_DERIVATIVE` | NO | NO | NO | NO | NO | NO | NO | advisory KG/KNOW outputs only |

H) AUTHORITATIVE WRITES VS DERIVED / INDEX / ELIGIBILITY SURFACES
- C3 authoritative memory lifecycle writes are confined to PH1.M-owned ledger/config surfaces.
- C3 derived surfaces may be updated, rebuilt, or removed only as consequences of authoritative lifecycle decisions.
- C3 must refuse any attempt to infer delete, purge, restore, or retention truth from current views, graph rows, bundle outputs, or audit records.

Authoritative Write vs Derived / Index / Eligibility Matrix
| surface | authoritative or derived | lifecycle action allowed | writer allowed | consumers allowed | must never be treated as | re-entry note |
| --- | --- | --- | --- | --- | --- | --- |
| `memory.memory_atoms_ledger` | authoritative | store, update, tombstone/delete, policy-expire, archive, purge, restore eligibility markers | PH1.M-owned C3 authoritative workers only | PH1.M current rebuild, eligibility builders, PH1.J, governed visibility readers | a cache, graph hint, or client-owned delete surface | re-entry always begins from latest lawful ledger event for `memory_key` |
| `memory.memory_threads_ledger` | authoritative | digest upsert, resolve, forget/tombstone, archive, purge, restore eligibility markers | PH1.M-owned C3 authoritative workers only | thread current rebuild, graph/index maintenance, resume builders, PH1.J | resume projection or work-order truth | re-entry begins from latest lawful `memory_thread_event_id` for `thread_id` |
| `memory.emotional_threads_ledger` | authoritative | emotional thread lifecycle updates, delete/tombstone, purge, restore eligibility markers | PH1.M-owned C3 authoritative workers only | emotional current rebuild, PH1.J, policy readers | factual memory authority or runtime truth | re-entry begins from latest lawful emotional thread ledger row |
| `memory.memory_suppression_rules` | authoritative config | do-not-mention, do-not-repeat, do-not-store policy updates | PH1.M-owned config writers only | bundle builders, resume selectors, delete/tombstone follow-up logic | a deletion proof that the memory subject no longer exists | re-entry re-reads current winning rule by `(target_type, target_id, rule_kind)` |
| `memory.memory_retention_preferences` | authoritative config | retention preference updates | PH1.M-owned config writers only | retention evaluation workers, resume selectors | proof that a subject was archived/purged/deleted | re-entry re-reads latest user preference row before evaluating lifecycle change |
| `memory.memory_atoms_current` | derived | projection/update/removal only after authoritative parent change | propagation/rebuild worker family only | runtime retrieval, eligibility checks, bundle builders | delete, purge, hold, or restore authority | stale current rows must be repaired from ledger, not trusted |
| `memory.memory_threads_current` | derived | projection/update/removal only after authoritative parent change | propagation/rebuild worker family only | resume selection, thread continuity, eligibility checks | archive/purge/restore authority | stale rows are rebuilt from thread ledger and retention rules |
| `memory.emotional_threads_current` | derived | projection/update/removal only after authoritative parent change | propagation/rebuild worker family only | tone continuity composition | deletion authority or factual truth | stale rows are rebuilt from emotional thread ledger |
| `memory.memory_graph_nodes` / `memory.memory_graph_edges` | derived index | index update, cleanup, rebuild | propagation/rebuild worker family only | PH1.M retrieval, KG-aware bounded reads | source of truth for whether a memory subject still exists or is restorable | if authoritative parent and graph diverge, parent wins and graph is rebuilt or removed |
| `memory.memory_archive_index` | derived pointer index with memory-owned archive meaning | pointer upsert, pointer cleanup, pointer restore | archive/propagation worker family only | bounded archive page-in, restore assist | proof that a subject is archived, purged, or lawful to restore | re-entry must re-check authoritative parent state before trusting pointer existence |
| `memory.memory_thread_refs` | derived pointer set | pointer upsert, pointer cleanup | propagation/rebuild worker family only | thread summary generation, bounded evidence lookup | thread existence authority or deletion receipt | stale refs are removed after authoritative thread tombstone/purge |
| `Ph1mHintBundleBuildResponse`, `Ph1mContextBundleBuildResponse`, `Ph1mResumeSelectResponse` | derived eligibility | no lifecycle mutation | PH1.M bundle builders only | runtime envelope, response assembly | memory retention, delete, purge, or restore authority | on re-entry these outputs are discarded and rebuilt from authoritative surfaces |
| `KgFactBundleSelectOk` / `KnowHintBundleSelectOk` | derived packaging/assist | no lifecycle mutation | PH1.KG / PH1.KNOW only | CONTEXT/NLP/TTS assist readers where allowed | memory lifecycle truth, deletion evidence, or restore authority | any drift is resolved by ignoring stale advisory output and rebuilding from authoritative memory state |
| `audit.audit_events` | evidence-only | append reason-coded visibility only | PH1.J and callers emitting bounded lifecycle evidence | audit, proof, compliance review | deletion or restore authority | re-entry may append missing audit rows, but never reconstruct memory truth from audit alone |

I) RETENTION, HOLD, ARCHIVE, TOMBSTONE, DELETE, PURGE, AND RESTORE LAW
- `retain` means the subject remains lawfully eligible for continued storage under current policy.
- `archive` means the subject is removed from normal active retrieval posture while preserving lawful lineage and possible later restore.
- `tombstone` means the authoritative subject is no longer active for normal retrieval or surfacing, but historical lineage still exists and purge may still be pending or blocked.
- `delete request` is a request or accepted intent to remove a subject. It is not terminal by itself.
- `purge` is terminal removal of the same lifecycle subject. Purged subjects may not be restored as the same subject.
- `restore` is lawful only when the parent subject was archived or tombstoned, has not been purged, passes identity/policy/hold checks, and restores through PH1.M authority rather than through current/index/evidence surfaces.
- `hold` means `LEGAL_HOLD` or `PRESERVATION_HOLD` as frozen by C1. Holds block delete terminalization and purge even when retention or delete requests would otherwise permit them.
- `policy expiry` is not silent removal. It must materialize as an authoritative lifecycle decision that either archives, tombstones, or explicitly refuses continued active eligibility.
- `memory_suppression_rules` may accompany delete/forget flows, but suppression is never a substitute for a tombstone, purge, or restore event.
- `graph`, `archive_index`, `thread_refs`, resume bundles, and assist packages are always follow-on surfaces. They may lag, but they may not win.

Retention / Hold / Archive / Delete / Purge Boundary Matrix
| case | winning canonical outcome | losing or blocked outcome | what remains authoritative | compensation / follow-up action | downstream visibility expectation |
| --- | --- | --- | --- | --- | --- |
| normal retention progression | subject remains retained or moves to archive/tombstone under lawful policy | silent expiration or silent removal | authoritative PH1.M ledger/config surfaces | propagate current/index cleanup and eligibility refresh | PH1.J visibility when policy transition is lifecycle-significant |
| policy expiry | `EXPIRED`-equivalent memory posture materializes as archive or tombstone per subject class and policy | continued active eligibility without policy basis | authoritative PH1.M lifecycle row/marker | remove active eligibility, refresh current/index surfaces, schedule purge only if policy allows | audit visibility required; governance/law only when governed posture exists |
| user-requested deletion | authoritative delete decision lands first, then tombstone becomes active unless blocked by hold or policy | client-local delete, UI-only removal, or suppression-only substitute | authoritative PH1.M ledger/config surfaces | propagate to current views, thread refs, graph, archive pointer policy, and outbox mirror | audit visibility required; proof/governance only when governed policy path requires it |
| hold attached before delete | hold wins; delete request is blocked or deferred | immediate tombstone/purge | authoritative hold marker plus pre-delete subject state | queue safe refusal or deferred review; preserve subject and record reason | audit visibility required; governance visibility if hold policy is governed |
| hold attached before purge | hold wins; purge is blocked | terminal purge | authoritative hold marker plus held subject posture | refuse purge, preserve subject, refresh derived surfaces to reflect held posture | audit visibility required; governance/law visibility when compliance posture matters |
| tombstoned but not yet purged | tombstone remains authoritative and restorable only if policy allows | derived view or graph entry reactivating the subject | authoritative ledger tombstone | continue cleanup, block active retrieval, evaluate later purge or restore | audit visibility required; proof/governance conditional |
| archived but still restorable | archived posture remains authoritative | auto-reactivation from archive pointer or bundle hit | authoritative archive marker plus parent ledger subject | restore only through lawful PH1.M restore path; keep pointer/index bounded | audit visibility when archive/restore is lifecycle-significant |
| purge terminality | `PURGED` wins and is terminal for the same subject | restore, current-view resurrection, graph recreation, or stale device replay | authoritative purge marker in PH1.M-owned lifecycle truth | remove derived residues, reject stale replays, emit bounded receipt/evidence | audit visibility required; governance/law only when governed deletion class requires it |
| restore lawful | restored non-terminal posture wins after archive/tombstone source passes identity, policy, hold, and lineage checks | stale archived/tombstoned posture continuing to suppress lawful use | authoritative PH1.M restore commit | rebuild current/index/eligibility surfaces from restored state | audit visibility required; governance/law only where existing governed paths apply |
| restore forbidden | refusal wins; subject stays archived, held, tombstoned, expired, or purged as applicable | bundle/output-driven or client-driven reactivation | authoritative pre-restore PH1.M posture | emit refusal reason, preserve subject, clean derived stale outputs | audit visibility required for refusal when lifecycle-significant |

J) WORKER FAMILIES
- C3 worker-family names are planning-level names grounded in PH1.M-owned commit groups and current storage surfaces. They do not imply that current code already contains identically named binaries.
- `Ph1mRetentionEvaluationWorker`
  - owns deterministic evaluation of retention windows, unresolved/pinned rules, policy expiry, and hold-aware eligibility
  - may author authoritative lifecycle decisions only for PH1.M-owned subjects
- `Ph1mArchiveAndExpiryWorker`
  - owns archive posture and archive-pointer maintenance for subjects moving out of active retrieval
  - may write authoritative archive/expiry markers and update `memory_archive_index` as a derived consequence
- `Ph1mDeleteTombstoneWorker`
  - owns explicit user delete/forget flows and authoritative tombstone decisions for atoms, threads, and emotional threads
  - may set suppression follow-up only as a companion control, never as substitute truth
- `Ph1mPurgeExecutionWorker`
  - owns terminal purge execution once hold, policy, and lineage checks allow it
  - removes derived residues only after authoritative purge wins
- `Ph1mDerivedViewPropagationWorker`
  - owns deterministic current-view, thread-ref, graph, archive-pointer, and eligibility refresh/rebuild after authoritative lifecycle changes
  - never authors lifecycle truth itself

Worker Family Matrix
| worker family | subject classes touched | authoritative write target | derived surfaces updated | trigger source | idempotency / ordering basis | forbidden writes |
| --- | --- | --- | --- | --- | --- | --- |
| `Ph1mRetentionEvaluationWorker` | `MEMORY_ATOM_SUBJECT`, `MEMORY_THREAD_SUBJECT`, `EMOTIONAL_THREAD_SUBJECT`, `MEMORY_RETENTION_POLICY_SUBJECT`, `MEMORY_HOLD_MARKER_SUBJECT` | PH1.M ledger/config surfaces for retention decision, expiry marker, hold-aware state | none directly; propagation may follow | policy tick, session close, retention preference change, unresolved-deadline scan, explicit lifecycle request | current repo truth uses identity-scoped idempotency, primarily `(user_id, idempotency_key)` plus ledger ordering by `ledger_id` / `memory_thread_event_id`; future tenant-aware implementation must preserve the same subject-scoped domain | direct writes to current/index surfaces as authority, PH1.J as state writer, PH1.GOV/PH1.LAW as state writers |
| `Ph1mArchiveAndExpiryWorker` | `MEMORY_ATOM_SUBJECT`, `MEMORY_THREAD_SUBJECT`, `MEMORY_ARCHIVE_POINTER_SUBJECT` | PH1.M authoritative archive/expiry markers plus lawful archive pointer updates | `memory_archive_index`, affected current views, bundle eligibility refresh | retention evaluation result, archive request, restore aftermath, rebuild scan | parent subject ordering first; archive pointer uses `(user_id, archive_ref_id)` primary-key stability and must never outrun parent-authoritative state | direct graph authority, direct bundle authority, client-side archive truth |
| `Ph1mDeleteTombstoneWorker` | `MEMORY_ATOM_SUBJECT`, `MEMORY_THREAD_SUBJECT`, `EMOTIONAL_THREAD_SUBJECT`, `MEMORY_SUPPRESSION_RULE_CONFIG` | PH1.M ledger tombstone/delete-request markers and companion suppression rule updates when explicitly required | current views become inactive or removed; thread refs and graph cleanup scheduled | explicit user delete/forget command, governed policy delete decision, admin/legal request within allowed scope | subject-scoped identity key plus current repo idempotency domains: atom/emotional actions by `(user_id, idempotency_key)`, thread actions by `(user_id, idempotency_key)`, suppression by `(user_id, target_type, target_id, rule_kind, idempotency_key)` | direct purge without prior lawful decision, graph/index writes as authority, audit-only delete |
| `Ph1mPurgeExecutionWorker` | `MEMORY_ATOM_SUBJECT`, `MEMORY_THREAD_SUBJECT`, `EMOTIONAL_THREAD_SUBJECT`, `MEMORY_ARCHIVE_POINTER_SUBJECT`, `MEMORY_GRAPH_RELATIONSHIP_RECORD`, `MEMORY_THREAD_REF_POINTER` | PH1.M-owned terminal purge marker or equivalent terminal lifecycle record | remove stale current/index/pointer residues after purge | purge deadline reached after tombstone/archive, compliance purge command, held-purge release reevaluation | purge must be subject-scoped and deterministic; if no explicit authoritative purge marker exists yet, worker must fail closed instead of inferring purge from residue deletion | direct restore, client ACK as purge authority, PH1.J/GOV/LAW as purge writers |
| `Ph1mDerivedViewPropagationWorker` | `MEMORY_ATOM_CURRENT_VIEW`, `MEMORY_THREAD_CURRENT_VIEW`, `EMOTIONAL_THREAD_CURRENT_VIEW`, `MEMORY_GRAPH_RELATIONSHIP_RECORD`, `MEMORY_THREAD_REF_POINTER`, `MEMORY_ARCHIVE_POINTER_SUBJECT`, `MEMORY_ELIGIBILITY_DERIVATIVE`, `PACKAGING_OR_ASSIST_DERIVATIVE` | none; authoritative inputs are read-only | all PH1.M current/index/pointer/eligibility outputs | authoritative lifecycle commit, rebuild request, divergence detection, restart recovery | deterministic rebuild from authoritative surfaces; duplicate updates are no-op if target already matches authoritative winner | any authoritative lifecycle write, any audit/proof/governance/law state mutation |

K) TRIGGERS, SWEEPERS, CHECKPOINTS, AND OWNERSHIP
- C3 must not invent a memory-only lease subsystem.
- Current repo truth provides strong single-authority law, append-only ordering, rebuildability, and idempotent commit groups, but no dedicated PH1.M worker lease table.
- Safest planning assumption:
- one lawful cloud owner per `(memory subject class, identity scope, subject_id)` at mutation time
- duplicate starts are refused by idempotency keys or by deterministic no-op after re-reading the authoritative checkpoint
- backstop sweepers scan authoritative PH1.M surfaces and never derived-only surfaces as their source of truth
- Authoritative checkpoints are:
- latest lawful ledger row for atom/thread/emotional subject
- latest current config row for suppression / retention preference / hold marker
- derived surfaces are never checkpoints

Trigger / Sweeper / Ownership Matrix
| worker family | primary trigger | backstop sweeper | ownership / lease or single-writer basis | duplicate-start protection | missed-trigger recovery rule | forbidden execution context |
| --- | --- | --- | --- | --- | --- | --- |
| `Ph1mRetentionEvaluationWorker` | retention preference update, session-close thread digest write, explicit policy-evaluation request, unresolved-deadline crossing | periodic scan of authoritative PH1.M ledgers/current config keyed by identity scope | single cloud owner per subject partition under ledger-first and cloud-authoritative law; no dedicated PH1.M lease row currently exists | re-read authoritative checkpoint; idempotent no-op if lifecycle decision already materialized | sweeper re-evaluates subjects whose retention posture is due or ambiguous | client device, current/index-only scan, graph-first execution |
| `Ph1mArchiveAndExpiryWorker` | retention evaluation marks archive/expiry transition or explicit archive/restore request | periodic scan of authoritative archive-eligible or expired subjects plus archive pointer divergence | same single-writer basis as parent authoritative subject; archive pointer update never owns the parent | parent-state re-check before archive pointer write; pointer overwrite is deterministic by PK | sweeper repairs missing or stale archive pointers and missing archive-derived cleanup | client device, archive-index-only mutation, KG/KNOW-driven execution |
| `Ph1mDeleteTombstoneWorker` | explicit user forget/delete command, admin/compliance delete decision, policy-expiry path resolving to tombstone | periodic scan for accepted delete intents lacking authoritative tombstone outcome | same single-writer basis as parent authoritative subject; no device-side delete authority | ledger/config idempotency plus subject re-read before tombstone commit | sweeper turns accepted-but-unfinished delete intent into a lawful tombstone or refusal | client-side cache eviction, audit-only delete, bundle-driven delete |
| `Ph1mPurgeExecutionWorker` | purge-eligible subject after lawful tombstone/archive plus non-blocking hold/policy posture | periodic scan for purge-eligible subjects and stale derived residues after terminal decision | same single-writer basis as parent authoritative subject; no PH1.M-specific lease row assumed | subject-scoped purge idempotency key and terminal-state re-read; purged same subject becomes immutable | sweeper completes missed purge cleanup or refuses if hold/policy changed | client device, graph/index residue deletion without authoritative purge win |
| `Ph1mDerivedViewPropagationWorker` | authoritative lifecycle commit, explicit rebuild request, divergence detection after restart | deterministic full or scoped rebuild from authoritative ledgers/config surfaces | single-writer basis at derived-surface partition level; authoritative parent always wins | compare target derived row to authoritative winner before mutating; identical result is no-op | sweeper rebuilds stale current/index/pointer/bundle surfaces from authoritative sources | any execution that treats derived surfaces as the checkpoint or authority |

L) IDEMPOTENCY, ORDERING, REPLAY, AND CONFLICT LAW
- Authoritative winner precedence for C3 is:
- PH1.M ledger/config truth
- then PH1.M current/index/pointer rebuild targets
- then derived eligibility/package outputs
- then audit/proof visibility
- Current repo-truth idempotency domains already exist for:
- atom events: `(user_id, idempotency_key)` in storage code, with PH1.M DB wiring reserving tenant/user scope at the contract layer
- thread lifecycle events: `(user_id, idempotency_key)`
- emotional thread updates: `(user_id, thread_key, idempotency_key)`
- suppression rules: `(user_id, target_type, target_id, rule_kind, idempotency_key)`
- graph updates: `(user_id, idempotency_key)`
- retention preference updates: `(user_id, idempotency_key)`
- Current repo-truth ambiguity:
- PH1.M DB wiring is tenant-scoped by contract
- current in-repo storage structs and indexes shown in `ph1f.rs` are user-scoped in this slice
- safest C3 interpretation:
- idempotency is identity-scoped today and must remain replay-safe
- implementation must not broaden dedupe outside the verified identity scope that current storage actually enforces
- Invalid transitions must refuse deterministically:
- restore after purge
- purge while hold is active
- archive/purge inferred only from graph or current view
- current-view resurrection after authoritative tombstone
- runtime bundle or KG/KNOW hint re-opening a deleted subject

Idempotency / Replay / Conflict Matrix
| lifecycle action | worker family | idempotency key tuple | ordering basis | replay-safe behavior | stale / conflicting behavior | authoritative winner |
| --- | --- | --- | --- | --- | --- | --- |
| atom retain/store/update/tombstone | `Ph1mRetentionEvaluationWorker` or `Ph1mDeleteTombstoneWorker` | current repo truth: `(user_id, idempotency_key)` for `memory_atoms_ledger`; business key must include `memory_key` and lifecycle action in the request scope | `ledger_id` append order, then `event.t_event` | duplicate append returns original ledger row and no-op current rebuild | stale request loses if a later lawful ledger event already changed the same `memory_key` | latest lawful `memory_atoms_ledger` row |
| thread digest update / resolve / forget | `Ph1mRetentionEvaluationWorker` or `Ph1mDeleteTombstoneWorker` | current repo truth: `(user_id, idempotency_key)` for `memory_threads_ledger`; business key must include `thread_id` and `event_kind` | `memory_thread_event_id` append order | duplicate request returns existing `(event_id, stored)` and no-op | stale thread action loses if later lawful `memory_thread_event_id` already changed that `thread_id` | latest lawful `memory_threads_ledger` row |
| emotional thread lifecycle update / tombstone | `Ph1mRetentionEvaluationWorker` or `Ph1mDeleteTombstoneWorker` | current repo truth: `(user_id, thread_key, idempotency_key)` | `emotional_thread_event_id` append order | duplicate request returns existing row and no-op | stale action loses if later lawful emotional thread ledger row exists for the same `thread_key` | latest lawful `emotional_threads_ledger` row |
| suppression rule companion update | `Ph1mDeleteTombstoneWorker` | current repo truth: `(user_id, target_type, target_id, rule_kind, idempotency_key)` | latest `created_at` wins within the exact key | duplicate request returns stored applied/no-op result | conflicting suppression never substitutes for parent lifecycle truth; parent subject state still wins | current winning `memory_suppression_rules` row plus parent authoritative subject |
| retention preference update | `Ph1mRetentionEvaluationWorker` | current repo truth: `(user_id, idempotency_key)` for `memory_retention_preferences` | latest `updated_at` | duplicate request returns original `updated_at` and no-op | stale preference update loses to later config row | latest `memory_retention_preferences` row |
| graph/index propagation | `Ph1mDerivedViewPropagationWorker` | current repo truth: `(user_id, idempotency_key)` for graph update commit | authoritative parent change first, then derived `updated_at` | duplicate propagation is a no-op if graph already matches authoritative state | conflicting graph row loses immediately to authoritative parent | authoritative parent ledger/config surface |
| archive pointer upsert / cleanup | `Ph1mArchiveAndExpiryWorker` or `Ph1mDerivedViewPropagationWorker` | current repo truth is PK-stable `(user_id, archive_ref_id)` with no explicit idempotency key on the index row; re-entry must bind to the parent authoritative lifecycle decision | parent authoritative decision first, then pointer `updated_at` | replay re-checks parent state and overwrites or removes pointer deterministically | stale pointer loses if parent subject is tombstoned, purged, restored, or otherwise no longer archive-eligible | authoritative parent subject, not `memory_archive_index` |
| purge execution | `Ph1mPurgeExecutionWorker` | required C3 implementation key: `(identity scope, subject_class, subject_id, purge idempotency key)` inside a PH1.M-owned purge marker; until that exists, worker must fail closed | terminal purge marker order at the authoritative subject | duplicate purge request must no-op once same-subject terminal state exists | conflicting restore/delete/current-view replay loses to terminal purge | authoritative PH1.M purge marker |
| restore execution | `Ph1mArchiveAndExpiryWorker` or `Ph1mDeleteTombstoneWorker` | required C3 implementation key: `(identity scope, subject_class, subject_id, restore idempotency key)` inside PH1.M-owned restore flow | latest lawful pre-restore state plus restore commit order | replay re-checks parent state and returns existing restored outcome | stale restore loses if hold blocks, purge already won, or later lawful state superseded the source | authoritative PH1.M restore commit |

M) FAILURE, COMPENSATION, QUARANTINE, AND SAFE REFUSAL MODEL
- C3 authoritative write order is fixed:
- authoritative PH1.M lifecycle commit first
- derived/index/eligibility propagation second
- audit/proof visibility third
- device acknowledgement or mirror delivery last
- If an authoritative lifecycle commit succeeds and a derived cleanup fails, the authoritative result remains canonical and the worker must re-enter from that checkpoint.
- C3 compensation is bounded:
- derived surface cleanup
- stale bundle invalidation
- pointer/index rebuild
- explicit refusal or quarantine when hold/policy/governance blocks further progress
- C3 quarantine is local to worker scope:
- hold/purge conflict
- missing identity scope or policy context for restore/purge
- stale current/index surfaces that cannot be deterministically reconciled on the same pass
- C3 must not silently absorb broader PH1.J / PH1.GOV / PH1.LAW redesign. If broader cross-subsystem reinterpretation is required, C3 stops at a safe refusal boundary and defers that redesign to C4.

Partial-Success / Re-entry Matrix
| case | authoritative state after failure | derived / projection posture | worker re-entry rule | retry rule | compensation rule | proof / governance / law visibility expectation |
| --- | --- | --- | --- | --- | --- | --- |
| authoritative ledger write succeeds, derived-view update fails | authoritative new ledger/config state stands | current/index/eligibility surfaces may be stale | re-read latest authoritative subject and rebuild only missing derived surfaces | YES | rebuild current/index surfaces; never roll back authoritative state because a derived update failed | audit required if lifecycle-significant; governance/law only if existing governed path is implicated |
| tombstone write succeeds, purge scheduling fails | subject remains authoritatively tombstoned | purge queue/schedule posture missing or delayed | re-enter from tombstone state; do not re-issue delete decision | YES | create or retry purge-eligible marker/schedule after re-checking hold/policy posture | audit required for tombstone; purge visibility emitted once purge actually runs or is refused |
| hold attach succeeds, delete request already queued | hold wins; delete remains blocked or deferred | queued delete intent may still exist operationally | re-enter from hold marker and refuse/defer delete terminalization | YES for refusal/defer update | convert queued delete into blocked/deferred posture; preserve subject | audit required; governance visibility when hold is governed |
| archive reference written, restore index update lags | archive/restore authoritative parent posture stands | `memory_archive_index` or bundle eligibility is stale | re-enter from parent authoritative state and repair pointer/index surfaces | YES | rebuild archive pointer, current, and bundle derivatives from authoritative winner | audit required when archive/restore is lifecycle-significant |
| purge succeeds, stale derived view remains | subject is terminally purged | current view, graph, thread refs, or bundle output may still show stale residue | re-enter from terminal purge marker and remove residue only | YES | remove stale derived residues; reject any read path that tries to use them | audit required; governance/law conditional by policy class |
| graph edge cleanup lags canonical record tombstone | parent subject remains tombstoned or purged | graph still references the stale parent | re-enter from parent subject and rebuild or remove graph rows | YES | graph cleanup only; never re-open parent subject from graph evidence | audit not always required for graph cleanup alone unless lifecycle-significant drift is detected |
| client acknowledgement lost after authoritative deletion decision | authoritative delete/tombstone/purge result stands | device may not know the latest result yet | re-enter from authoritative subject, then replay bounded mirror/receipt delivery | YES | resend bounded receipt or mirror delta through existing delivery path; no second lifecycle commit | audit already attached to authoritative decision; no new law/governance semantics created by ACK replay |

N) GRAPH / PACKAGING / DERIVED-VIEW BOUNDARIES
- `memory_graph_nodes` and `memory_graph_edges` are retrieval/index aids only.
- `memory_graph_nodes` and `memory_graph_edges` may never:
- recreate a purged or tombstoned parent subject
- prove that a subject is still retained
- bypass identity or eligibility gates
- `memory_archive_index` is pointer/index metadata only.
- `memory_archive_index` may never:
- prove that archive storage succeeded if the parent authoritative lifecycle state disagrees
- authorize restore without PH1.M-owned parent-state approval
- `memory_thread_refs` are bounded evidence pointers only.
- `memory_thread_refs` may never re-establish thread existence after authoritative thread forget/purge.
- `Ph1mHintBundleBuildResponse`, `Ph1mContextBundleBuildResponse`, and `Ph1mResumeSelectResponse` are derived eligibility/package outputs only.
- These outputs may never:
- serve as lifecycle checkpoints
- justify restore
- justify retention override
- justify delete/purge refusal without re-checking authoritative PH1.M state
- `KgFactBundleSelectOk` and `KnowHintBundleSelectOk` remain advisory outputs only.
- KG and KNOW may help bounded retrieval or vocabulary behavior, but they may never:
- keep a deleted or purged memory subject alive
- invent archive/restore eligibility
- override suppression, retention, or hold posture
- Learning/packaging boundary:
- C3 may invalidate or refresh derived package inputs when lifecycle changes invalidate memory sources
- C3 does not redesign PH1.LEARN or PH1.KNOW artifact packaging

O) PROOF / GOVERNANCE / LAW VISIBILITY AT C3 SCOPE
- PH1.J visibility is required at C3 scope for lifecycle-significant actions such as:
- tombstone/delete accepted
- delete refused because of hold or policy
- policy-expiry lifecycle transition
- purge executed or purge refused
- restore executed or restore refused
- hold attached or released when the hold materially changes lifecycle outcome
- PH1.J payload discipline remains bounded:
- reason-coded fields only
- evidence references only
- no raw transcript dumps
- PH1.GOV visibility at C3 scope is conditional, not universal:
- use governance visibility only when an existing governed memory policy path requires override, hold, or compliance review
- C3 does not create a new global governance stream for all memory lifecycle actions
- PH1.LAW visibility at C3 scope is conditional, not universal:
- use law visibility only when a memory lifecycle outcome changes live protected-execution admissibility in an already governed path
- C3 does not invent a new memory-only runtime-law subsystem
- C3 proof/governance/law boundary:
- reason-coded visibility yes
- worker-local safe refusal yes
- cross-subsystem proof/governance/law normalization no; that remains deferred to C4

P) EXPLICIT NON-GOALS / DEFERRED TO C4 OR C5
- C3 does not redesign PH1.M retrieval ranking, memory UX, or learning behavior beyond the lifecycle enforcement boundaries required here.
- C3 does not redesign the archive payload storage backend beyond the existing pointer/index model already present in repo truth.
- C3 does not implement tests, docs closure, or final verification packs.
- C3 does not redesign PH1.J / PH1.GOV / PH1.LAW schemas; it only states the visibility needed at worker scope.

C3 → Downstream Boundary Matrix
| concern | handled in C3 | deferred to C4 | deferred to C5 | rationale |
| --- | --- | --- | --- | --- |
| PH1.M authoritative retention / archive / tombstone / purge / restore worker ownership | YES | NO | NO | this is the core purpose of C3 |
| full PH1.J memory lifecycle proof schema normalization across all lifecycle actions | NO | YES | NO | C3 names required visibility but does not redesign proof architecture |
| full PH1.GOV / PH1.LAW end-to-end lifecycle normalization for every memory transition | NO | YES | NO | C3 stays within bounded worker-scope visibility and refusal rules |
| archive payload substrate redesign beyond `memory_archive_index` pointer model | NO | YES | NO | repo truth currently exposes pointer/index surfaces only; backend redesign is outside C3 |
| tests, docs verification closure, acceptance pack, and freeze proof | NO | NO | YES | C5 owns closure evidence and verification discipline |
| bundle/ranking/product UX tuning for memory retrieval | NO | YES | NO | lifecycle enforcement must not widen into retrieval-product redesign |

Q) COMPLETION CRITERIA
- C3 is complete when:
- PH1.M authoritative lifecycle subjects and derived surfaces are explicitly separated
- retention, hold, archive, tombstone, delete request, purge, and restore law is explicit and deterministic
- purge terminality and restore-forbidden cases are explicit
- graph/index/pointer/bundle/package reverse-authority is explicitly refused
- worker families, triggers, sweepers, checkpoints, and ownership are explicit
- idempotency, replay, stale-write refusal, and authoritative-winner rules are explicit
- partial-success re-entry rules are explicit
- proof/governance/law visibility at C3 scope is explicit without absorbing C4
- C3 vs C4 vs C5 boundaries are explicit and stable
- the plan is strong enough that implementation can proceed without guessing:
- which PH1.M surface is authoritative
- which surface is derived only
- which lifecycle action each worker owns
- how hold and purge conflicts resolve
- how stale graph, archive, and eligibility surfaces are repaired without reopening authoritative truth
