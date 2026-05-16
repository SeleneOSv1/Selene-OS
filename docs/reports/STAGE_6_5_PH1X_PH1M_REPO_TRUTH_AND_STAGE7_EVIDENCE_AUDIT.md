# Stage 6.5 PH1.X / PH1.M Repo Truth And Stage 7 Evidence Audit

Date: 2026-05-16

Build class: no-code architecture / repo-truth audit

Latest Stage 6 baseline requested: `f4787d15dd70efe4e8d870ee42144de3873f42fb`

Audited repo baseline:

- Branch: `main`
- Start HEAD: `122db443ce1e6357133ef63bad4532322d661754`
- Origin main: `122db443ce1e6357133ef63bad4532322d661754`
- Stage 6 ancestor proof: `STAGE6_ANCESTOR_OK`
- Start tree: clean

## 1. Executive Conclusion

Stage 7 should not start as a broad storage/archive implementation yet.

The repo already has strong foundations:

- PH1.X has a real owner in `crates/selene_engines/src/ph1x.rs` with contract state in `crates/selene_kernel_contracts/src/ph1x.rs`.
- PH1.M has a real owner in `crates/selene_engines/src/ph1m.rs` with extensive contracts in `crates/selene_kernel_contracts/src/ph1m.rs`.
- PH1.F storage already contains session, conversation, memory, thread digest, graph, archive index, audit, and proof surfaces in `crates/selene_storage/src/ph1f.rs`.
- The adapter already preserves conversation text and thread digest data, and it bridges voice/typed turns into the runtime.

The repo is not ready for Stage 7 evidence storage as-is because the canonical future evidence packets are not yet explicit:

- `ActiveContextPacket` does not exist.
- `HumanConversationDirective` does not exist.
- `MemoryEvidencePacket` does not exist as the target design describes it.
- `FreshMemoryHandoff` does not exist.
- `MemoryContinuationDecision` does not exist as a single PH1.M contract.

There is also a duplicate-owner risk: the adapter currently contains substantial live-context and discourse logic (`H380`, `H411`, weather/time follow-up helpers, recent archive shortcut routing) that overlaps with the PH1.X and PH1.M target designs.

Readiness statement:

`NOT_READY_FOR_STAGE_7`

Recommended next build:

`Option 2: Before Stage 7, a small contracts build is required.`

Required contracts:

- `ActiveContextPacket`
- `HumanConversationDirective`
- `MemoryEvidencePacket`
- `MemoryRecallRequest` or a clearly renamed equivalent layered over existing `Ph1mRecallRequest`
- `FreshMemoryHandoff`
- `MemoryContinuationDecision`

Reason: Stage 7 is about filing immutable evidence. It should file the right evidence refs, not freeze temporary adapter context shortcuts into archive truth.

## 2. Current Repo Baseline

Mandatory law and plan review completed:

- `AGENTS.md` reviewed through line 900.
- Master plan reviewed through all available requested ranges.
- PH1.X Human Conversation Core design reviewed.
- PH1.M Human Memory Core design reviewed.
- Core architecture and engine inventory documents reviewed as required by `AGENTS.md`.

Start proof:

- Branch: `main`
- HEAD: `122db443ce1e6357133ef63bad4532322d661754`
- Origin main: `122db443ce1e6357133ef63bad4532322d661754`
- Stage 6 baseline ancestor: yes
- Start tree clean: yes

No code files were edited for this audit.

## 3. PH1.X Repo Truth

### 3.1 Owner Files

Current PH1.X owner paths:

- `crates/selene_kernel_contracts/src/ph1x.rs`
- `crates/selene_engines/src/ph1x.rs`
- `crates/selene_os/src/ph1x.rs`
- `crates/selene_os/src/app_ingress.rs`, as the runtime ingress builder/executor that constructs PH1.X requests and executes PH1.X responses

Adapter paths that currently overlap with PH1.X target behavior:

- `crates/selene_adapter/src/lib.rs`

### 3.2 Existing Active Context Structures

Current central PH1.X state is `ThreadState` in `crates/selene_kernel_contracts/src/ph1x.rs`.

Important fields:

- `pending`
- `resume_buffer`
- `last_turn_context`
- `identity_prompt_state`
- `active_subject_ref`
- `interrupted_subject_ref`
- `return_check_pending`
- `return_check_expires_at`
- `active_speaker_user_id`
- `project_id`
- `pinned_context_refs`
- `thread_policy_flags`

Current follow-up context carrier:

- `LastTurnContext`
- `LastTurnRouteClass`

Current PH1.X directive contract:

- `Ph1xDirective`
- `ConfirmDirective`
- `ClarifyDirective`
- `DispatchDirective`
- `WaitDirective`

Gap against target: `ThreadState` is a useful active-state carrier, but it is not the target `ActiveContextPacket`. `Ph1xDirective` is useful, but it is not the target `HumanConversationDirective`.

### 3.3 Existing Follow-Up Logic

PH1.X has follow-up and continuation behavior through:

- `ResumeBuffer`
- interrupt continuity state
- return-check state
- pending clarify/confirm/tool state
- `LastTurnContext`
- tool follow-up request handling in `crates/selene_os/src/app_ingress.rs`

PH1.X tests confirm this existing behavior:

- resume continue
- resume more detail
- same-subject merge
- switch topic then return check
- subject/speaker mismatch protection
- memory permission handling
- tool dispatch and tool response follow-up

Adapter-level follow-up behavior also exists in `crates/selene_adapter/src/lib.rs`, including:

- `deterministic_active_context_followup_query`
- `deterministic_weather_context_followup_query`
- `H380TurnUnderstandingPacket`
- H379/H380/H381/H409/H411/H412 helper logic
- adapter-local tests for time/weather follow-up and topic-switch protection

### 3.4 Existing Time/Weather Specific Patches

Time/weather continuity currently has specific adapter support:

- adapter weather context state
- deterministic weather context follow-up query
- deterministic active context follow-up query
- tests such as `active_session_context_time_what_about_location_inherits_time_intent`
- tests such as `active_session_context_weather_australia_sydney_fills_pending_slot_and_clears`
- tests proving identity/public chat is not poisoned by time slot context

This is useful current behavior, but it is not the target architecture. The target says PH1.X must become universal live context, not a pile of time/weather follow-up patches.

### 3.5 Existing Universal Continuation Logic

Partial.

Universal pieces exist:

- `ThreadState`
- `PendingState`
- `ResumeBuffer`
- `LastTurnContext`
- interruption continuity
- PH1.X clarify/confirm/respond/dispatch/wait directives

But the repo does not yet have one universal continuation gate that covers:

- writing artifacts
- topic stack
- references like `it`, `that`, `same`, `again`
- human posture
- response rhythm
- correction controller
- social response control

### 3.6 Existing Topic Switch Protection

Partial.

PH1.X protects against subject mismatch when pending state exists and can ask whether to continue or switch. It also has interruption switch-topic return-check behavior.

Adapter tests also cover topic switch protection:

- `active_session_context_topic_switch_does_not_reuse_time_intent`
- `active_session_context_time_slot_does_not_poison_identity_or_public_chat`
- `active_session_context_voice_time_slot_does_not_poison_identity_or_public_chat`

Gap: this protection is split between PH1.X and adapter helpers.

### 3.7 Existing Reference Resolution

Partial.

Reference handling exists through:

- `ResumeBuffer`
- `LastTurnContext`
- pending clarification state
- adapter H380/H411 direct noun phrase and ambiguous context handling

Gap: there is no central PH1.X reference resolver that covers `it`, `that`, `same`, `again`, `the first one`, `the other one`, `back to that`, or writing-artifact references.

### 3.8 Existing Correction Handling

Partial.

PH1.X supports confirm/clarify/step-up and deterministic protected boundaries. Adapter H380 includes correction and rephrase detection.

Gap: correction is not yet a PH1.X `Correction Controller` with a standard output packet.

### 3.9 Existing Writing Artifact Handling

No central PH1.X writing-artifact state was found.

PH1.WRITE exists and formats answer text, but PH1.X does not yet track a live writing artifact such as:

- draft email
- story
- report
- instruction
- proposal
- prior generated artifact to modify

### 3.10 Existing PH1.M Handoff

Partial.

PH1.X accepts `memory_candidates` and applies limited personalization and sensitive-memory permission rules.

Adapter directly invokes recent archive recall in some flows:

- `Ph1mRecentArchiveRecallRequest`
- `recent_archive_recall_from_repo`
- adapter tests that recent archive recall does not pollute active context

Gap: there is no central PH1.X to PH1.M handoff port that asks for fresh memory continuation after wake using a stable memory evidence packet.

### 3.11 Duplicate / Stale PH1.X Context Paths

Duplicate or overlapping paths:

- PH1.X `ThreadState` and `LastTurnContext`
- adapter H379/H380/H381/H409/H411/H412 live context and discourse helpers
- adapter weather context state and deterministic weather follow-up helper
- adapter direct active-context follow-up rewrite

These are not all dead today. Many are actively tested and needed for current Stage 1-6 behavior. But they are duplicate ownership risks for the PH1.X rewrite.

## 4. PH1.M Repo Truth

### 4.1 Owner Files

Current PH1.M owner paths:

- `crates/selene_kernel_contracts/src/ph1m.rs`
- `crates/selene_engines/src/ph1m.rs`
- `crates/selene_os/src/ph1m.rs`
- `crates/selene_storage/src/ph1f.rs`
- `crates/selene_storage/src/repo.rs`
- `crates/selene_storage/migrations/0021_ph1m_vnext_memory_tables.sql`

Adapter paths that currently touch memory/archive:

- `crates/selene_adapter/src/lib.rs`

### 4.2 Existing Recall Request/Response Contracts

Existing PH1.M contracts include:

- `Ph1mProposeRequest`
- `Ph1mProposeResponse`
- `Ph1mRecallRequest`
- `Ph1mRecallResponse`
- `Ph1mForgetRequest`
- `Ph1mForgetResponse`
- `Ph1mRecentArchiveRecallRequest`
- `Ph1mRecentArchiveRecallResponse`
- `Ph1mThreadDigestUpsertRequest`
- `Ph1mThreadDigestUpsertResponse`
- `Ph1mResumeSelectRequest`
- `Ph1mResumeSelectResponse`
- `Ph1mHintBundleBuildRequest`
- `Ph1mHintBundleBuildResponse`
- `Ph1mContextBundleBuildRequest`
- `Ph1mContextBundleBuildResponse`
- `Ph1mSafeSummaryRequest`
- `Ph1mSafeSummaryResponse`
- `Ph1mGraphUpdateRequest`
- `Ph1mGraphUpdateResponse`
- `Ph1mRetentionModeSetRequest`
- `Ph1mRetentionModeSetResponse`
- suppression/emotional/metrics contracts

The existing `Ph1mRecallRequest` is not the same as the target design's broad `MemoryRecallRequest`, but it is the current closest owner contract.

### 4.3 Existing Recent Recall Logic

PH1.M has recent archive recall:

- query term extraction
- 72-hour style windowing
- topic matching over thread digests
- wake-address normalization for `Selene` / `Celine` in wake context
- unknown-speaker blocking

Tests include:

- `recent_archive_recall_by_topic_uses_72h_window`
- `recent_archive_yesterday_uses_fixed_clock_range`
- `recent_archive_recall_distinguishes_ph1m_ph1x_and_desktop_thin_queries`
- `recent_archive_recall_normalizes_celine_only_in_wake_context`
- `recent_archive_recall_blocks_unknown_speaker`

### 4.4 Existing Archive/Digest Logic

PH1.M has thread digest support:

- `MemoryThreadDigest`
- `memory_threads_ledger`
- `memory_threads_current`
- `memory_thread_refs`
- `memory_archive_index`

Adapter also updates recent archive digest from conversation ledger after transcript writes.

### 4.5 Existing Memory Evidence / Provenance Refs

Existing evidence/provenance surfaces:

- `MemoryProvenance`
- `MemoryCandidate`
- `MemoryProposedItem`
- `MemoryLedgerEvent`
- `MemoryArchiveExcerpt`
- `MemoryRecentArchiveMatch`
- `MemoryBundleItem`
- audit events with `evidence_ref`
- canonical proof ledger

Gap: no target-style `MemoryEvidencePacket` was found.

### 4.6 Existing No-Match Behavior

Partial.

PH1.M recent archive recall can return no matches with a reason code. The target design's human-facing `No-Record Handler` is not yet a central memory contract.

### 4.7 Existing Active-Context Pollution Protection

Partial.

Current protections:

- PH1.X owns live `ThreadState`.
- PH1.L close/new-session clears active context.
- Adapter has tests that recent archive recall does not inject PH1.M candidates into active context.
- Adapter tests prove new sessions do not inherit weather slots.

Gap: PH1.M does not yet have a central `MemoryContinuationDecision` packet that PH1.X asks after wake.

### 4.8 Existing Permanent Governed Memory

Partial.

Current memory layers and policies include:

- `MemoryLayer`
- `MemoryUsePolicy`
- `MemorySensitivityFlag`
- `MemoryConsent`
- `MemoryConfidence`
- memory ledger/current projections
- retention preferences
- suppression rules

Gap: this is not yet a full `Permanent Governed Memory` module with explicit trust/privacy/conflict/staleness/no-record packet semantics.

### 4.9 Existing Topic Memory

Partial.

Topic-like memory exists through:

- memory thread digest rows
- thread refs into conversation ledger
- graph nodes and edges
- recent archive recall by topic

Gap: this is not yet a full human topic-memory brain with current status, rejected options, next likely step, stale/superseded handling, and natural user-facing recall style.

### 4.10 Existing Deep Recall

Partial.

Recent archive recall and memory archive index exist. A deep recall orchestrator over older history was not found.

### 4.11 Duplicate Recall / Memory Paths

Duplicate or overlapping memory paths:

- PH1.M engine recall and recent archive recall
- OS PH1.M wiring
- adapter direct recent archive recall route
- adapter digest update from conversation ledger
- storage memory ledger/current and PH1.M vNext tables

These are not necessarily dead today. The adapter routes are active, but they should later be reduced once PH1.M recall orchestration and PH1.X handoff are canonical.

## 5. Adapter Transport Truth

### 5.1 Voice Turn Transport

Voice turns enter through the adapter's canonical turn request shape:

- `VoiceTurnAdapterRequest`

Important fields:

- `correlation_id`
- `turn_id`
- `device_turn_sequence`
- `app_platform`
- `trigger`
- `actor_user_id`
- `device_id`
- `now_ns`
- `thread_key`
- `project_id`
- `pinned_context_refs`
- `thread_policy_flags`
- `user_text_partial`
- `user_text_final`
- `selene_text_partial`
- `selene_text_final`
- `audio_capture_ref`
- `visual_input_ref`

The adapter calls runtime ingress through `crates/selene_os/src/app_ingress.rs`.

### 5.2 Typed Turn Transport

Typed turns also use the adapter's turn bridge. They are represented as final user text with no audio capture ref.

Gap: storage currently distinguishes user/Selene role and source, but Stage 7 should require explicit `modality: voice/typed` evidence rather than inferring typed input from missing audio.

### 5.3 Session / Thread / Turn IDs

The adapter and storage carry:

- `session_id`
- `thread_key`
- `turn_id`
- `correlation_id`
- `device_turn_sequence`
- `actor_user_id`
- `device_id`
- project and pinned context refs

PH1.F storage owns durable session rows and conversation rows.

### 5.4 PH1.X Context Data Transport

PH1.X thread state is loaded and persisted through adapter/runtime/store surfaces.

Current PH1.X data is not transported as a target `ActiveContextPacket`; it is transported as `ThreadState`, runtime request/response, and adapter-local trace/metadata.

### 5.5 PH1.M Recall Data Transport

PH1.M recall is transported through existing PH1.M requests/responses and adapter direct recent archive routes.

Gap: there is no single target `MemoryEvidencePacket` transport from PH1.M to PH1.X/PH1.WRITE/Desktop.

### 5.6 Response Text And TTS Text Transport

Adapter response fields include:

- `response_text`
- `tts_text`
- `source_chips`
- `source_cards`
- `image_cards`
- `answer_class`
- `metadata_safe_for_user`
- `screen_lifecycle_action`
- `session_lifecycle_action`

Stage 7 should store both visible answer text and approved TTS text/status refs. The conversation ledger currently stores text turns, but not a dedicated TTS delivery record per turn.

### 5.7 Tool / Provider Metadata Transport

Tool/provider metadata exists in PH1.E responses, source cards/chips, public brain trace, tool cache, audit/proof ledgers, and adapter metadata.

Gap: Stage 7 needs a stable evidence ref that points to tool family, provider, route, source refs, and result provenance without exposing private reasoning or raw debug traces.

### 5.8 Adapter Shortcut Logic That Belongs Later In PH1.X / PH1.M

The adapter currently contains shortcut live-context and memory-adjacent logic:

- H380 understanding packet
- H411/H412 public discourse context
- deterministic active-context follow-up query
- deterministic weather context follow-up query
- weather context state
- direct recent archive recall trigger
- recent archive digest update from conversation ledger

These are active and tested. They should not be deleted in this audit. But the future PH1.X/PH1.M build should absorb or retire them under the clean replacement law.

## 6. Storage / Archive / Ledger Truth

### 6.1 Current Storage Owners

Primary storage owner:

- `crates/selene_storage/src/ph1f.rs`

Repository trait owner:

- `crates/selene_storage/src/repo.rs`

Schema migrations:

- `crates/selene_storage/migrations/0001_ph1f_foundation.sql`
- `crates/selene_storage/migrations/0007_ph1l_sessions_indexes.sql`
- `crates/selene_storage/migrations/0021_ph1m_vnext_memory_tables.sql`

### 6.2 What Is Stored Per Turn Today

Conversation ledger stores:

- `correlation_id`
- `turn_id`
- `session_id`
- `user_id`
- `device_id`
- `role`
- `source`
- `text`
- `text_hash`
- `privacy_scope`
- `created_at`
- `idempotency_key`
- tombstone refs

The adapter writes final transcript turns through `append_transcript_final_conversation_turn`.

### 6.3 What Is Stored Per Session

Session storage includes:

- `session_id`
- `user_id`
- attached devices
- last attached device
- `device_id`
- `session_state`
- `opened_at`
- `last_activity_at`
- `closed_at`
- `last_turn_id`
- `active_turn_id`
- lease owner/acquired/expires
- `project_id`
- `pinned_context_refs`
- device turn sequences
- device idempotency keys

### 6.4 User Transcript Storage

Yes. User final text is stored in `conversation_ledger`.

Gap: Stage 7 should store PH1.C transcript status/confidence/rejection refs separately from the final committed text.

### 6.5 Selene Response Text Storage

Yes. Selene final output text is stored as a conversation turn.

Gap: Stage 7 should store PH1.WRITE formatted/presentation refs and answer hashes explicitly.

### 6.6 TTS Text Storage

Partial.

The adapter response carries `tts_text`. The conversation ledger stores Selene text, but a dedicated per-turn TTS text/status/playback evidence record was not found in the audited storage truth.

Stage 7 should store:

- approved `tts_text`
- TTS provider/status
- audio ready ref
- playback start/end refs
- fail-closed reason when TTS is unavailable

### 6.7 Modality Storage

Partial.

The conversation ledger has `source`, but Stage 7 should store explicit modality:

- `voice`
- `typed`
- possibly `system/lifecycle`

This matters for PH1.M and PH1.X because voice and typed are equal conversation turns but have different safety evidence.

### 6.8 Route / Tool / Provider Metadata Storage

Partial.

Tool/provider metadata exists in PH1.E outputs, adapter response metadata, audit/proof/tool-cache surfaces, and source cards. It is not yet a single immutable per-turn evidence packet attached to the conversation ledger.

### 6.9 Correction Events

Partial.

Adapter and PH1.X have correction/rephrase handling, and audit/proof can store events. A dedicated correction event row/ref per turn was not found.

### 6.10 Protected Fail-Closed Events

Yes, through audit/proof/governance surfaces.

Stage 7 should preserve fail-closed refs tied to the turn and never reduce them to a normal assistant answer.

### 6.11 Memory Proposal / Write / Rejection Events

Yes, memory ledger/current and PH1.M propose/forget/persist flows exist.

Gap: Stage 7 needs memory proposal/write/reject refs linked to turn/segment/session evidence so PH1.M can later explain memory use naturally.

### 6.12 Topic Tags / Digest Rows

Partial.

PH1.M thread digest and graph storage exist:

- `memory_threads_ledger`
- `memory_threads_current`
- `memory_thread_refs`
- `memory_graph_nodes`
- `memory_graph_edges`
- `memory_archive_index`

Gap: topic candidates and salience candidates are not yet stored per turn as Stage 7 evidence.

## 7. Stage 7 Evidence Requirements

Stage 7 must store enough immutable evidence for future PH1.X and PH1.M to build human-like conversation and memory without scraping raw old sessions.

Minimum required evidence:

1. Identity and boundary evidence
   - `internal_segment_id`
   - `session_id`
   - `thread_key`
   - `turn_id`
   - `correlation_id`
   - timestamps
   - user/device IDs
   - wake boundary refs
   - sleep/close/new-session refs

2. Input evidence
   - modality: `voice` / `typed`
   - committed user transcript
   - unsent typed draft exclusion status when relevant
   - PH1.C status
   - transcript confidence / rejection reason
   - noise/cough/self-echo rejection refs
   - audio capture ref where allowed

3. Output evidence
   - Selene response text
   - approved `tts_text`
   - TTS status
   - playback delivery refs
   - visible final answer hash
   - PH1.WRITE format/presentation refs

4. Live context evidence
   - PH1.X active context packet ref
   - PH1.X human conversation directive ref
   - continuation / topic switch / clarification / correction decision refs
   - confidence and ambiguity refs
   - reference target refs

5. Tool/action evidence
   - PH1.E route refs
   - tool family
   - provider metadata
   - source refs
   - tool result status
   - protected fail-closed refs
   - no protected execution proof refs

6. Memory evidence
   - PH1.M memory evidence packet refs
   - memory proposal/write/reject refs
   - topic candidates
   - salience candidates
   - privacy/sensitivity tags
   - trust/source type
   - conflict/staleness refs
   - no-record / no-match refs

7. Audit and proof evidence
   - audit event refs
   - canonical proof refs
   - idempotency keys
   - provenance hashes
   - reason codes

## 8. Stage 8 Fresh Memory Requirements

Target proof:

`New York -> sleep -> wake -> what about Sydney`

Required Stage 7 evidence for that proof:

### PH1.L Must Emit

PH1.L must emit a sleep/close boundary evidence ref containing:

- session ID
- thread key
- last valid user turn
- last Selene answer turn
- close reason
- wake-ready state
- timestamp

### PH1.M Must Preserve

PH1.M must preserve fresh memory evidence:

- last live topic
- last tool family: time
- last user entity: New York
- last answer type: time answer
- confidence
- freshness label
- allowed continuation status
- evidence refs to original turns

### PH1.X Must Ask PH1.M After Wake

After wake, PH1.X must ask PH1.M whether the new utterance is a fresh-memory continuation.

For `what about Sydney`, PH1.M should return high-confidence fresh continuation evidence.

### Adapter Must Transport

Adapter must transport packet refs only:

- fresh memory handoff ref
- PH1.X context packet ref
- PH1.M memory evidence packet ref
- not adapter-owned meaning

### Storage Must Preserve

Storage must preserve:

- New York user turn
- New York answer
- sleep/close boundary
- wake boundary
- Sydney user turn
- Sydney answer
- evidence refs linking the fresh continuation

### PH1.E Must Receive

PH1.E needs:

- tool family: time
- new entity/location: Sydney
- previous route family carried by PH1.X/PH1.M evidence, not stale adapter string rewriting

### PH1.WRITE Must Receive

PH1.WRITE needs:

- final answer text/result
- recall style if memory is surfaced
- no private reasoning
- no mechanical "session search" wording

### Desktop Must Display

Desktop displays:

- user message
- Thinking/timing
- final answer
- optional human memory language if runtime provides it

Desktop must not resolve `what about Sydney`.

## 9. PH1.X Target Gap Table

| Target module | Exists now | Owner path | Current evidence | Gap | Recommended build timing | Risk if skipped |
|---|---:|---|---|---|---|---|
| Active Conversation Frame | Partial | `crates/selene_kernel_contracts/src/ph1x.rs`, `crates/selene_engines/src/ph1x.rs` | `ThreadState`, `LastTurnContext`, `active_subject_ref` | Not a full active frame with topic/task/posture/artifact slots | Option 2 contracts, then PH1.X rewrite | Stage 7 stores incomplete context refs |
| Topic Stack | Partial | PH1.X plus adapter | interrupt return-check, resume buffer | No general stack for temporary/returnable topics | PH1.X Build 2/3 | Selene loses returnable topics |
| Interaction Posture Engine | Partial | Adapter H380/H411, PH1.N/PH1.X | H380 posture-like fields, PH1.N intent | Not centralized in PH1.X | PH1.X Build 2 | Adapter remains semantic owner |
| Conversation Rhythm Engine | Partial | PH1.X | clarify/respond/wait directives | No full rhythm model for tiny ack/direct/structured/wait | PH1.X Build 3/5 | Robotic response shape |
| Reference Resolver | Partial | PH1.X plus adapter H411 | resume buffer, LastTurnContext, noun phrase tests | No central resolver for `it/that/same/back to` | PH1.X Build 3 | Follow-ups stay patchy |
| Continuation Gate | Partial | PH1.X plus adapter | resume/return-check and active context helpers | Split across owners | Option 2 then PH1.X Build 2 | Duplicate context brains |
| Slot + Entity Frame | Partial | PH1.N, PH1.X, adapter | intent fields and adapter slot repairs | Not one PH1.X entity frame | PH1.X Build 4 | Tool follow-ups remain special cases |
| Writing Artifact State | No | None found | PH1.WRITE formats text only | No live artifact state | PH1.X Build 4 | "Make it shorter" cannot generalize |
| Tool Continuity State | Partial | PH1.X, PH1.E, adapter | `LastTurnRouteClass`, PH1.E route, weather context helper | Not a universal tool continuity state | PH1.X Build 4 | Stale tool context bugs |
| Correction Controller | Partial | Adapter H380, PH1.X clarify/confirm | correction/rephrase tests | Not PH1.X-owned controller | PH1.X Build 5 | Corrections stay brittle |
| Topic Switch Detector | Partial | PH1.X plus adapter | subject mismatch, topic switch tests | Split protection | PH1.X Build 5 | Old context hijacks normal questions |
| Ambiguity / Clarification Engine | Partial | PH1.X | `ClarifyDirective`, pending clarify | Not full ambiguity model | PH1.X Build 2/5 | Over/under clarifies |
| Social Response Control | Partial | PH1.X/PH1.N | response text and delivery hint | No response-shape packet | PH1.X Build 5 | Responses too mechanical |
| Risk / Protected Boundary Gate | Partial | PH1.X/governance | confirm/step-up/protected fail closed | Good base, not integrated into human directive | PH1.X Build 5 | Risk if context engine guesses |
| PH1.M Handoff Port | Partial | PH1.X/adapter/PH1.M | memory candidates, recent archive shortcut | No fresh memory handoff contract | Option 2 | Stage 8 fresh memory cannot be clean |
| ActiveContextPacket | No | None | No exact type found | Missing | Option 2 | Stage 7 cannot store canonical refs |
| HumanConversationDirective | No | None | Closest is `Ph1xDirective` | Missing | Option 2 | Other engines invent context logic |

## 10. PH1.M Target Gap Table

| Target module | Exists now | Owner path | Current evidence | Gap | Recommended build timing | Risk if skipped |
|---|---:|---|---|---|---|---|
| Recall Orchestrator | Partial | `crates/selene_engines/src/ph1m.rs` | propose/recall/recent archive/resume/context bundle functions | Not one gateway deciding fresh/day/topic/deep/permanent | PH1.M Build 2 | Scattered recall behavior |
| Encoding Engine | Partial | PH1.M propose | `MemoryProposedItem`, commit decisions | No full notice/encode pipeline | PH1.M Build 3 | Memory quality stays low |
| Salience Engine | Partial | PH1.M contracts/runtime | layer, confidence, use policy | No explicit salience type | PH1.M Build 3 | Everything remembered too equally |
| Consolidation Engine | Partial | PH1.M thread digest | thread digest upsert | No full post-session consolidation | PH1.M Build 3 | Memory remains transcript-ish |
| Fresh Memory | Partial | PH1.M resume/recent archive | hot/warm/cold resume, recent recall | No fresh handoff packet | Option 2 then PH1.M Build 4 | Sleep/wake continuation will be ad hoc |
| Day Memory | Partial | recent archive windows | yesterday/fixed clock tests | No day-memory module | PH1.M Build 5/7 | Today/yesterday recall weak |
| Topic Memory | Partial | thread digest/current/refs | thread digest and refs | Not human topic state | PH1.M Build 5 | Topic recall remains search-like |
| Topic Graph | Partial | PH1.M graph/storage | graph nodes/edges | No living topic graph behavior yet | PH1.M Build 5 | Related topics not connected |
| Deep Recall | Partial | archive index/recent recall | archive index exists | No deep recall orchestrator | PH1.M Build 7 | Older memory remains shallow |
| Permanent Governed Memory | Partial | PH1.M memory ledger/current | layers/use policy/privacy/consent | Needs governed stable memory brain | PH1.M Build 8 | Stable facts not governed enough |
| Continuation Gate | Partial | resume_select/recent archive | resume hot/warm/cold | Not memory continuation decision packet | Option 2 | PH1.X cannot ask memory cleanly |
| Memory Posture Engine | No | None found | None | Missing | PH1.M Build 4/5 | Over-memory and intrusive recall |
| Freshness Gradient | Partial | PH1.M resume tiers | hot/warm/cold constants | Needs human freshness labels | PH1.M Build 4/5 | Recall language feels mechanical |
| Conflict + Staleness Checker | Partial | context bundle | conflict/stale tags and metrics | Not global conflict/staleness lifecycle | PH1.M Build 8 | Old decisions may outrank new |
| Memory Trust Engine | Partial | contracts | confidence/provenance tier | Target trust levels not complete | PH1.M Build 8 | Unverified memory too strong |
| Memory Privacy Gate | Partial | PH1.M policies | sensitivity, consent, exposure, unknown speaker block | Good base, needs full gate | PH1.M Build 8 | Sensitive recall risk |
| No-Record Handler | Partial | recent archive no matches | empty result/reason code | No human no-record response contract | PH1.M Build 6/10 | Robotic "no result" behavior |
| Memory Evidence Packet | No | None exact | MemoryCandidate/ArchiveMatch are closest | Missing | Option 2 | Stage 7 cannot preserve future proof cleanly |
| Memory Use Policy | Partial | PH1.M contract | `MemoryUsePolicy` | Needs human memory use policy orchestration | PH1.M Build 8 | Memory may over-apply |
| Human Memory Eval Matrix | Partial | PH1.M/adapter tests | recent archive/resume/context tests | Not full human matrix | PH1.M Build 10 | Regressions hard to catch |

## 11. Duplicate / Stale Path Report

### Duplicate PH1.X / Context Paths

Active duplicate-risk paths:

- `ThreadState` in PH1.X contracts
- H380/H411/H412 adapter context/discourse packets
- adapter deterministic active-context follow-up rewrite
- adapter deterministic weather context follow-up rewrite
- adapter weather context state

Status: active, tested, not removed in this audit.

Future action: migrate toward one PH1.X `ActiveContextPacket` and `HumanConversationDirective`, then remove adapter shortcut meaning under the clean replacement law.

### Duplicate PH1.M / Recall Paths

Active duplicate-risk paths:

- PH1.M runtime recall/recent archive/resume select
- OS PH1.M wiring
- adapter direct recent archive recall call
- adapter digest update from conversation ledger

Status: active, tested, not removed in this audit.

Future action: route all memory relevance decisions through a PH1.M recall orchestrator and PH1.X handoff port.

### Adapter Shortcut Memory/Context Paths

The adapter currently carries more than pure transport:

- context classification
- discourse routing
- weather/time context gate
- recent archive trigger
- digest update

These are the biggest risk for Stage 7 evidence filing because Stage 7 could accidentally preserve adapter-local decisions as canonical architecture.

### Desktop Memory/Search Logic

No Desktop-owned memory brain was found in the audited Desktop surfaces. Desktop appears to render timeline/search-like UI and bridge transport, but not own memory relevance.

### PH1.E Stale Context Ownership

PH1.E owns tool action. It should not own memory. Current risk is not PH1.E itself, but context-rich query text and adapter rewrites feeding PH1.E. Stage 7 should preserve route evidence, not make PH1.E a memory holder.

### User-Facing Session Search Wording

The repo has archive/recent recall language internally. Future PH1.M work should keep this internal and surface human memory language through PH1.WRITE.

## 12. Test / Validation Summary

Validation run during this no-code audit:

- `cargo check`: passed.
- `cargo test -p selene_engines ph1x -- --test-threads=1`: passed, 56 tests.
- `cargo test -p selene_engines ph1m -- --test-threads=1`: passed, 38 tests.
- `cargo test -p selene_storage ph1m -- --test-threads=1`: passed as a filter run; storage unit/integration targets completed with zero matching tests in many targets.
- `cargo test -p selene_adapter active_session_context -- --test-threads=1`: passed, 11 tests.
- `cargo test -p selene_adapter recent_archive_recall_does_not_pollute_active_context_after_answer -- --test-threads=1`: passed, 1 test.

Broad `cargo test -p selene_adapter`, `cargo test -p selene_os`, and `cargo test -p selene_engines` were not run because this task is no-code architecture/report-only and the targeted tests directly cover the audited PH1.X, PH1.M, adapter active-context, recent archive, and storage memory surfaces. `cargo check` was run to verify the workspace still compiles.

No behavior was modified.

## 13. Recommended Next Build

Recommendation:

`Option 2: Before Stage 7, a small contracts build is required.`

Contracts to add or formalize:

- `ActiveContextPacket`
- `HumanConversationDirective`
- `MemoryEvidencePacket`
- `MemoryRecallRequest` or a compatible renamed wrapper around current `Ph1mRecallRequest`
- `FreshMemoryHandoff`
- `MemoryContinuationDecision`

Why Option 2:

- PH1.X exists but is not yet the universal live-attention engine.
- PH1.M exists and is far richer than simple search, but it lacks the target evidence packet and fresh memory handoff contract.
- Storage exists, but Stage 7 needs to know exactly which PH1.X/PH1.M refs to preserve before immutable archive rows are expanded.
- Adapter currently contains context shortcuts that are valuable but should not become canonical Stage 7 evidence.

Not recommended:

- Option 1 is too early because the packet contracts are missing.
- Option 3 is not required first because duplicate paths are active and tests are green; cleanup should follow contract replacement, not precede it blindly.
- Option 4 is not the primary blocker because Stage 6 active close/wake was already implemented, and the current gap is evidence contract readiness for Stage 7/8.

## 14. Clear Readiness Statement

`NOT_READY_FOR_STAGE_7`

Exact reason:

Stage 7 needs to preserve immutable evidence for future PH1.X and PH1.M human conversation/memory work, but the repo does not yet define the canonical packets that Stage 7 should store. Starting Stage 7 now risks baking adapter-local H380/H411/weather-context shortcuts and incomplete PH1.X/PH1.M evidence into archive truth.

Required next fix:

Run a small contracts build to define the PH1.X and PH1.M packet refs listed above, then begin Stage 7 evidence storage against those contracts.
