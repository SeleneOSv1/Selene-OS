SELENE ALWAYS-AVAILABLE VOICE + CONTINUOUS SESSION MEMORY MASTER BUILD PLAN

TASK FAMILY:
SELENE_ALWAYS_AVAILABLE_VOICE_CONTINUOUS_CHAT_SESSION_MEMORY_MASTER_PLAN

STATUS:
Corrected rewrite after repo-truth audit and missing-stage review on 2026-05-12.

This is the active master roadmap for Selene's always-available voice product, optional visible chat screen, continuous chat timeline, internal sealed sessions, active session context, recent recall, full topic/session-bundle archive recall, searchable history, medium-term project memory, permanent governed memory, semantic repair, correction learning, natural barge-in, and protected simulation/business execution.

This is a roadmap, not one Codex build. Codex must implement it in ordered, verified slices. A foundation is not a completed product stage unless the full product behavior is built, wired, and verified by repo truth and real-app proof where the stage requires it.

STAGE COUNT CORRECTION

Old plan:
Stage 0 through Stage 21, which is 22 stages total.

Rejected rewrite:
13 active stages. That reduction was too aggressive because it moved or weakened product functions that are not fully built and verified.

Corrected active remaining roadmap:
18 active stages.

Reason for 18:
Only the Stage 0 law/provenance gate is treated as fully completed. Verified runtime foundations are preserved below, but missing, partial, unverified, and product-incomplete functions remain active. Barge-in/natural voice and protected simulation/business workflows are split into separate active stages because they have different owners, risks, and acceptance gates.

No product function is intentionally removed unless it is fully BUILT_AND_VERIFIED as a complete product capability. If a function is missing, partial, test-only, foundation-only, or lacks real current Desktop app proof, it remains active roadmap work.

CORE PRODUCT OUTCOME

Selene is always available by voice.
The screen is optional.
The user sees one continuous chat.
Internally, Selene uses sealed sessions and segments.
Current session context is separate from memory recall.
Recent recall is the fast recent memory tier.
The 72-hour window is a recent-recall fast tier, not the full memory system.
Full topic/session-bundle archive recall is the deeper memory target for older saved sessions.
Memory is searchable.
Desktop is never the brain.
Runtime owns meaning.
Protected execution remains simulation-gated.

HARD OWNER LAW

Before any implementation, Codex must search current repo truth for existing owners.
Codex must reuse or repair existing owners before creating new code.
Codex must not create duplicate Desktop app paths.
Codex must not create duplicate adapter/runtime owners.
Codex must not create duplicate wake/listening loops.
Codex must not create duplicate PH1.X active-context paths.
Codex must not create duplicate PH1.M memory/recall paths.
Codex must not create duplicate PH1.E tool/provider paths.
Codex must not create duplicate PH1.L session lifecycle paths.
Codex must not create duplicate PH1.TTS playback policy paths.

Desktop must remain capture, playback, transport, render, app lifecycle, and visible control only.
Runtime owns meaning.
PH1.X owns active current-session context.
PH1.M owns recall, memory, archive retrieval, and governed memory.
PH1.L owns session lifecycle and session boundaries.
PH1.E owns tools, providers, place, time, and weather routing.
PH1.C, PH1.LANG, PH1.SRL, and PH1.N own transcript quality, language, semantic repair, and intent normalization support.
PH1.TTS owns speech rendering authority and playback safety policy.
Protected execution requires simulation plus authority and must fail closed.

Desktop must not own:

- intent
- memory
- slot filling
- tool routing
- place resolution
- semantic repair
- PH1.X active context
- PH1.M recall
- PH1.E tools/providers
- protected execution
- authority decisions

New session means a logical runtime/session boundary, not a new app.
Close screen means hide/minimize the screen, not quit Selene.
Hidden screen conversation must still be captured and written to the timeline.
Recall must not become active context.
Full archive search must eventually search all eligible sealed sessions and return a truthful no-record result when nothing exists.
Semantic repair is essential for ChatGPT-level understanding, but should be built after the minimum session/archive structure is reliable unless JD explicitly reprioritizes.

CORE ARCHITECTURE SPLIT

PH1.L:
Session lifecycle: open, resume, close, timeout, session boundary, new-session control, no-leak.

PH1.X:
Active live conversation: pending slot, last intent, current topic, short follow-up, correction, topic switch, active trace, no-leak. PH1.X is current-session only.

PH1.M:
Memory and recall: recent recall, 72-hour fast tier, archive recall, topic bundles, medium-term memory, permanent governed memory. PH1.M recall must not become active pending intent.

Storage / Archive / Audit:
Immutable records: sealed sessions, turn records, transcript refs, archive refs, digests, topic tags, audit/proof history, black-box ledger events.

PH1.E:
Tool/runtime: time, weather, place resolution, provider/tool routing.

PH1.C / PH1.LANG / PH1.SRL / PH1.N:
Understanding stack: transcript quality, language normalization, semantic repair, intent normalization.

Adapter:
Runtime bridge: transcript transport, committed turn, routing bridge, session/thread/correlation IDs, provenance, API paths.

Desktop:
Client shell: app lifecycle, icon/status, capture, playback, TTS playback surface, show/hide window, render, transport. Desktop renders and transports; it does not decide meaning.

COMPLETED / VERIFIED FOUNDATIONS

These are verified foundations or law gates. They are not removed from preservation. Future work must reuse them and must not rebuild them as duplicate owners. A foundation listed here may still have active product work later in this plan.

1. Existing-owner and current-app provenance law

Status:
BUILT_AND_VERIFIED as the completed Stage 0 law/provenance gate.

Owner:
AGENTS.md.

Proof source:
AGENTS.md contains existing owner reuse law, duplicate-path hard stops, Desktop current-app provenance law, stale-app hard stops, and honest real-app failure reporting requirements.

Preservation gate:
Every future implementation must include existing owner discovery and must stop before duplicate owner creation.

2. PH1.X active follow-up core

Status:
BUILT_AND_VERIFIED as a core runtime foundation only. It is not full product completion.

Owner:
PH1.X plus adapter active-context bridge.

Proof source:
Repo tests and landed commits prove active follow-up behavior for weather/time clarification and short follow-ups such as Australia to Sydney, Spain to Canary Islands, Melbourne to London, New York to London, topic switch prevention, and no stale weather/time leakage after recall.

Preservation gate:
Future work must preserve PH1.X as the only active-context owner. Desktop and PH1.M must not fill active slots.

3. PH1.M recent archive core

Status:
BUILT_AND_VERIFIED as a core runtime foundation only. It is not full recent-recall product completion.

Owner:
PH1.M engine, PH1.M OS wiring, storage digest rows, adapter recent-recall route.

Proof source:
Recent archive request/response contracts, thread digest upsert, recent archive recall, live-turn digest indexing, matching/scoring, no-match behavior, and no active-context pollution tests exist in PH1.M and adapter paths.

Preservation gate:
Future memory work must reuse the existing PH1.M recent archive path and must not create a second memory system.

4. Adapter runtime bridge baseline

Status:
BUILT_AND_VERIFIED as a committed runtime bridge foundation.

Owner:
crates/selene_adapter/src/lib.rs and crates/selene_adapter/src/bin/http_adapter.rs.

Proof source:
Committed adapter full-suite baseline and targeted preservation suites for recent archive, active context, Desktop continuous session, Desktop voice, wake life, false transcript, post-TTS, Voice ID, and HTTP adapter surfaces.

Preservation gate:
Future Desktop, voice, session, and memory stages must use existing adapter endpoints and must not add parallel adapter owners.

5. OpenAI-first STT/TTS provider path and fallback policy foundation

Status:
BUILT_AND_VERIFIED as voice provider foundation only. It is not full voice product completion.

Owner:
PH1.C, PH1.TTS, adapter HTTP routes, Desktop playback bridge.

Proof source:
OpenAI realtime transcription and OpenAI TTS routes exist. Runtime returns fallback_allowed for bounded native macOS playback only when policy allows it. Desktop plays final runtime answer text and must not invent local response text.

Preservation gate:
OpenAI remains first for Selene voice. Apple/native speech may only be runtime-allowed fallback playback for already-rendered final Selene text.

6. Global place, time, and weather core

Status:
BUILT_AND_VERIFIED as runtime/tool foundation.

Owner:
PH1.E, PH1.X, adapter.

Proof source:
Global place resolution, time, weather, and same-thread follow-up tests exist for Spain, Canary Islands, Melbourne, London, New York, Sydney, and ambiguity cases.

Preservation gate:
PH1.E owns tool/provider/place behavior. Desktop must not route tools or choose providers.

7. False transcript, noise, and post-TTS gates

Status:
BUILT_AND_VERIFIED as runtime safety foundation.

Owner:
PH1.C, PH1.K/listening boundaries, adapter transcript gate, Desktop playback evidence.

Proof source:
False transcript and post-TTS tests reject cough/noise fragments, CJK hallucination-style noise, wake-like single tokens, and self-echo tail captures.

Preservation gate:
Future voice stages must preserve no false committed prompt from noise and no self-echo user turn.

8. Desktop icon/listening resource foundation

Status:
BUILT_AND_VERIFIED as resource foundation only.

Owner:
Desktop resources and Desktop shell icon controller.

Proof source:
Current committed work includes idle/listening icon resources, Xcode resource bundling, and dock icon pulse/listening resource surfaces.

Preservation gate:
Future UI work must preserve single app identity and must not create duplicate app bundles or duplicate process paths.

Important:
The full background availability status model is not complete. It remains active Stage 2.

9. Protected fail-closed foundation

Status:
BUILT_AND_VERIFIED as safety foundation only. It is not protected business workflow product completion.

Owner:
PH1.LAW, PH1.X, PH1.OS, access/policy/governance, SimulationExecutor.

Proof source:
Existing protected-action tests and runtime law require simulation, authority, audit, and fail-closed behavior before protected execution.

Preservation gate:
Public chat, search, and recall may remain probabilistic/read-only. Protected/business execution remains deterministic, simulation-gated, and authority-gated.

PARTIAL / PRESERVATION GATES

These have code, tests, docs, or foundations, but still require product proof, missing behavior, UX completion, or final acceptance. They also appear in the active roadmap below.

Desktop current-app provenance and single runtime owner:
PARTIAL_RUNTIME. Needs repeatable real-app proof against current bundle, one app, one adapter, no stale DerivedData or old /Applications app.

Wake acknowledgement and listening re-arm:
PARTIAL_RUNTIME. Code paths exist; reliable real current-app wake proof remains active Stage 1.

Hidden-screen active conversation:
PARTIAL_RUNTIME. Conversation shell and capture paths exist; hidden-screen always-available product proof remains active Stage 1.

Background icon/status full state model:
PARTIAL_RUNTIME. Resource files exist; full idle, wake listening, active listening, thinking, speaking, muted, error, hidden-but-available state proof remains active Stage 2.

Local wake privacy boundary:
PARTIAL_RUNTIME / OWNER_BOUNDARY_GAP. PH1.W and wake foundations exist, but current Desktop wake can involve OpenAI realtime wake transcription. Local pre-wake detection remains active Stage 3 unless JD changes the privacy requirement.

Voice-controlled screen show/hide:
MISSING / PRODUCT_INCOMPLETE. Must remain active Stage 4.

Voice and typed input parity:
PARTIAL_RUNTIME. Shared runtime ingress exists; real app parity proof remains active Stage 5.

Safe logical session boundary and new-session control:
PARTIAL_RUNTIME. PH1.L foundation and Desktop control exist; real no-leak proof remains active Stage 6.

Active session context preservation:
PARTIAL_RUNTIME. Core PH1.X tests pass; real-app preservation, trace visibility, correction expansion, and topic-switch robustness remain active through Stages 1, 5, 6, and 14.

Immutable session archive:
PARTIAL_RUNTIME. Conversation ledger and archive foundations exist; product-complete sealed session archive remains active Stage 7.

Black-box ledger:
PARTIAL_RUNTIME. Audit foundations exist; full append-only replay/audit/debug proof and protected fail-closed/memory/provider event ledger certification remains active Stage 7.

Recent recall fast tier:
PARTIAL_RUNTIME. Core PH1.M path exists; real voice proof and final product behavior remain active Stage 8.

Recent recall trace/eval and deeper-search offer:
PARTIAL_RUNTIME. Matching/scoring exists; trace/evidence/no-match reason/deeper-search offer remains active Stage 9.

Topic/session-bundle archive recall:
MISSING / SPECIFIED_NOT_IMPLEMENTED. Remains active Stage 10.

User-selected recall window:
MISSING. Remains active Stage 11.

Session search UI/API:
PARTIAL_RUNTIME / MISSING_UI_API. Remains active Stage 12.

Medium-term project memory:
PARTIAL_RUNTIME. Memory foundations exist; project continuity product remains active Stage 13.

Semantic repair and intent reranking:
PARTIAL_RUNTIME. Understanding foundations exist; full runtime repair/rerank remains active Stage 14.

Correction-pair learning and vocabulary packs:
PARTIAL_RUNTIME. Owners exist; governed learning product remains active Stage 15.

Permanent governed memory UX:
PARTIAL_RUNTIME / UX_MISSING. Remains active Stage 16.

Barge-in and natural voice loop:
PARTIAL_RUNTIME. Internal foundations exist; real Desktop natural interruption remains active Stage 17.

Protected business workflows:
PARTIAL_RUNTIME. Fail-closed foundation exists; concrete business workflows remain active Stage 18.

ACTIVE REMAINING BUILD STAGES

The active future roadmap has 18 remaining stages. Every stage below must reuse existing owners and must not duplicate completed foundations.

STAGE 1 - Desktop current-app, wake, listening, re-arm, and hidden-screen product proof

BUILD:
SELENE_DESKTOP_CURRENT_APP_WAKE_LISTENING_REARM_HIDDEN_SCREEN_PRODUCT_PROOF

Owner:
Desktop lifecycle/capture/playback/render shell, managed adapter bridge, adapter healthz/provenance endpoint, PH1.C transcript gate, PH1.W wake owner, PH1.X runtime response owner, PH1.L session lifecycle.

Purpose:
Prove Selene is one current app, one managed adapter, wake-capable, listening-capable, re-armed after response, and able to keep conversation alive when the visible screen is hidden.

What already exists:
AGENTS current-app law, Desktop singleton/provenance commits, healthz repo/bundle/PID surfaces, managed adapter lifecycle paths, wake lifecycle handling, OpenAI TTS bridge, conversation shell, and archive/recent slice surfaces.

What must be built:
Repeatable real-app proof and minimal owner-local repairs if proof fails: current bundle launch proof, stale app refusal/closure, duplicate app prevention, duplicate adapter prevention, port owner proof, wake acknowledgement, TTS playback, self-echo suppression, listening re-arm, hidden wake/capture/reply flow, background timeline writes, reopen-to-current-conversation rendering, and no quit/no stale app behavior.

What must not be duplicated:
No duplicate app process path. No duplicate adapter launcher. No second healthz owner. No voice-button substitute for wake proof. No Desktop semantic prompt handling. No hidden local transcript authority. No manual smoke against stale DerivedData or /Applications bundle.

Acceptance criteria:
Current HEAD is proven.
Exact app bundle path is proven.
Only one Desktop app instance is active.
Only one managed adapter owns the port.
/healthz proves repo head, app PID, adapter PID, and bundle path where available.
User says "Selene" and the wake word is not sent as a semantic user prompt.
Selene acknowledges readiness and listening re-arms.
Window hidden does not quit Selene.
User can wake and speak while hidden.
Selene answers by voice.
Committed turns are written to timeline/archive.
Opening the screen shows the background conversation.

Preservation gates:
Desktop remains lifecycle/capture/playback/render/transport only. Runtime owns meaning. PH1.W owns wake. PH1.C owns transcript quality. PH1.L owns session lifecycle.

Deferred items:
The full local pre-wake privacy boundary is Stage 3. Voice screen show/hide commands are Stage 4.

STAGE 2 - Background availability icon/status full state model

BUILD:
SELENE_BACKGROUND_AVAILABILITY_ICON_AND_STATUS_FULL_STATE_MODEL

Owner:
Desktop render/status shell only; runtime remains authority for state meaning.

Purpose:
Make Selene visibly available even when the screen is hidden and accurately reflect product state without creating duplicate app identities.

What already exists:
AppIcon and AppIconListening resources, Xcode resource bundling, dock pulse/listening resource foundation.

What must be built:
Full status mapping and proof for idle, wake listening, active listening, thinking, speaking, muted, error, hidden-but-available, and unavailable/fail-closed states.

What must not be duplicated:
No second icon bundle identity. No duplicate app path. No local runtime state authority in Desktop.

Acceptance criteria:
Status reflects runtime/capture/playback truth.
Hidden app still shows correct availability posture.
Speaking and listening states do not overlap incoherently.
Muted and error states are visible and non-authoritative.
No duplicate app identity appears.

Preservation gates:
Desktop renders status only.

Deferred items:
Voice show/hide commands are Stage 4.

STAGE 3 - Local wake listening and privacy boundary

BUILD:
SELENE_LOCAL_WAKE_LISTENER_PRIVACY_BOUNDARY_COMPLETION

Owner:
PH1.W for wake detection/capture boundaries, PH1.K for audio substrate, Desktop for local capture transport only, PH1.C for transcript gate after wake.

Purpose:
Ensure pre-wake listening is local and privacy-bounded, then allow OpenAI realtime STT only after lawful wake for the active user turn.

What already exists:
PH1.W and wake training foundations, Desktop foreground wake listener, wake-to-turn bridge, OpenAI realtime wake transcription path.

What must be built:
Reconcile current OpenAI wake transcription with the product privacy rule. Build or wire local pre-wake detection through existing owners, or explicitly preserve current limitation until JD approves a changed requirement.

What must not be duplicated:
No second wake engine. No Desktop semantic wake decision. No provider shortcut. No background audio streaming before lawful wake if the local privacy requirement remains.

Acceptance criteria:
"Selene" wakes the app.
Pre-wake audio is not streamed to OpenAI under the local-wake rule.
Wake word is not committed as a semantic prompt.
Wake detection emits bounded evidence only.
Protected execution remains impossible from wake alone.

Preservation gates:
PH1.W owns wake; PH1.C owns transcript gate; Desktop captures/transports only.

Deferred items:
Wake acknowledgement/re-arm product proof is covered by Stage 1.

STAGE 4 - Voice-controlled screen show/hide

BUILD:
SELENE_VOICE_CONTROLLED_SCREEN_SHOW_HIDE

Owner:
Runtime meaning/command classification through PH1.X or approved lifecycle command route; Desktop show/hide/focus/render only.

Purpose:
Let the user show or hide the Selene screen by voice without quitting Selene or giving Desktop semantic authority.

What already exists:
Desktop can show/render/focus/hide as a platform shell capability. Runtime command classification is not product-complete.

What must be built:
Runtime-owned recognition for commands such as "show me the screen", "open the chat", "show Selene", "bring Selene up", "hide the screen", "close the screen", "close desktop", and "minimize Selene"; Desktop executes only approved lifecycle action packets.

What must not be duplicated:
No Desktop intent parsing. No local command router. No hidden session mutation. No close-screen equals quit behavior.

Acceptance criteria:
Voice command can show the screen.
Voice command can hide/minimize the screen.
Hiding does not quit Selene.
Showing displays the current continuous chat.
Normal chat phrases are not mistaken for lifecycle commands.

Preservation gates:
Runtime owns meaning; Desktop executes lifecycle only.

Deferred items:
Advanced proactive UI behavior.

STAGE 5 - Voice and typed input parity

BUILD:
SELENE_VOICE_AND_TYPED_INPUT_PARITY_PRODUCT_PROOF

Owner:
Adapter canonical turn ingress, Desktop input transport, PH1.C transcript gate for voice, PH1.X current turn sequencing, PH1.TTS playback/cancel policy.

Purpose:
Voice and typed input must share one runtime path and one committed user turn at a time.

What already exists:
Voice and typed paths use existing runtime bridge surfaces. Tests cover pieces of Desktop continuous session and active context.

What must be built:
Real app parity proof for voice-then-type, type-then-voice, typed while TTS is speaking, cancellation behavior, no merge, no race, and same timeline/archive path.

What must not be duplicated:
No separate typed brain. No separate voice brain. No Desktop merge/split semantics.

Acceptance criteria:
Voice turn commits once.
Typed turn commits once.
Voice and typed do not merge unrelated input.
Typed send during TTS cancels or queues according to runtime policy.
The visible transcript and spoken answer remain aligned to the same final runtime answer.
TTS/self-echo boundaries are preserved.

Preservation gates:
Adapter owns turn ingress; Desktop transports only.

Deferred items:
Barge-in is Stage 17.

STAGE 6 - Safe logical session boundary and new-session control proof

BUILD:
SELENE_SAFE_SESSION_BOUNDARY_AND_NEW_SESSION_CONTROL_PROOF

Owner:
PH1.L session lifecycle, PH1.X active context reset/no-leak, adapter IDs/runtime bridge, Desktop lifecycle UI/transport only.

Purpose:
Keep one visible continuous chat while creating safe internal session boundaries that clear active context without deleting archive history.

What already exists:
PH1.L session foundation, session/thread/turn/correlation IDs, Desktop new conversation control, voice thread key rotation, PH1.X no-leak tests.

What must be built:
Real app proof and minimal owner-local repair if needed for new-session boundary, session sealing, active context reset, archive preservation, and no duplicate app/adapter.

What must not be duplicated:
No second session lifecycle owner. No Desktop-authored session truth. No local-only memory reset bypassing PH1.L/PH1.X.

Acceptance criteria:
New conversation creates a logical runtime/session boundary, not a new app.
Old PH1.X pending context clears.
Archive remains searchable.
London/Sydney after new session do not inherit old time/weather context.
No duplicate app or adapter appears.

Preservation gates:
PH1.L owns lifecycle. PH1.X owns active context. PH1.M recall remains searchable but not active.

Deferred items:
User-selected recall window is Stage 11.

STAGE 7 - Immutable session archive and black-box ledger certification

BUILD:
SELENE_IMMUTABLE_SESSION_ARCHIVE_AND_BLACK_BOX_LEDGER_CERTIFICATION

Owner:
Storage/archive/audit, PH1.L session lifecycle, adapter committed turn bridge, PH1.M archive references, protected fail-closed audit owners.

Purpose:
Every committed turn belongs to an immutable session/archive record, with black-box proof history sufficient for replay, audit, recall, debugging, and failure classification.

What already exists:
conversation_ledger, audit events, recent archive digests, session/archive surfaces, storage foundations, protected fail-closed foundations.

What must be built:
Product-complete sealed session archive, black-box ledger packet/event model, turn/session correlation proof, route metadata, TTS text refs, transcript refs, memory events, provider/tool metadata, correction events, protected fail-closed attempts, append-only replay/audit/debug proof, and archive search readiness.

What must not be duplicated:
No separate archive outside storage/PH1.M. No Desktop archive fabrication. No mutable old-turn rewrite path. No audit ledger outside existing storage/audit owners.

Acceptance criteria:
Every committed voice and typed turn has session_id, thread_key, turn_id, correlation_id, timestamp, modality, user transcript, Selene response, route, transcript refs, TTS refs, and audit/proof refs.
Old turns are append-only.
Corrections are new events.
Forget/delete are governed events.
Protected fail-closed attempts are recorded safely.
Provider/tool metadata is recorded without leaking secrets.
Archive can be searched later.
Black-box ledger can explain proof/failure history.

Preservation gates:
Storage/archive/audit are authoritative; Desktop renders only.

Deferred items:
Session search UI/API is Stage 12.

STAGE 8 - Real recent recall voice smoke and indexing certification

BUILD:
SELENE_REAL_RECENT_RECALL_VOICE_SMOKE_AND_INDEXING_CERTIFICATION

Owner:
PH1.M recent archive, storage/archive digest rows, adapter runtime bridge, Desktop capture/playback/transport only.

Purpose:
Prove recent recall works through the real current Desktop voice path, not just typed tests.

What already exists:
PH1.M recent archive contracts, thread digest upsert, adapter live-turn indexing, recall query detection, matching/scoring, answer composition, no-match tests, no active-context pollution tests.

What must be built:
The final real Desktop voice smoke from current HEAD: voice seed capture, archive indexing, voice recall, no-match, no active-context pollution, active follow-up preservation, new-session no-leak, clipped-fragment safety, and noise suppression.

What must not be duplicated:
No typed adapter-only substitute. No voice button substitute for wake if the smoke requires wake. No Desktop memory logic. No second memory system. No adapter shortcut recall lane.

Acceptance criteria:
Five voice seed turns are captured, committed, sent, answered, and indexable.
At least four of five recall prompts return correct seeded content.
Project Zebra Lantern no-matches cleanly.
Recall answers do not become PH1.X active context.
Active weather/time follow-ups still work after recall.
New-session no-leak still holds.
Noise/cough does not create a committed prompt.

Preservation gates:
PH1.M owns recall. PH1.X owns active context. 72-hour recall is the fast recent tier only.

Deferred items:
Trace/eval and deeper-search offer are Stage 9.

STAGE 9 - Recent recall trace/eval and deeper-search offer

BUILD:
SELENE_RECENT_RECALL_TRACE_EVAL_AND_DEEPER_SEARCH_OFFER

Owner:
PH1.M recent recall, storage/archive/search, adapter recall API, Desktop render only.

Purpose:
Make recent recall explain why it matched, why it no-matched, and when it should offer deeper archive search.

What already exists:
Recent recall matching/scoring, no-match behavior, digest references, adapter route surfaces.

What must be built:
Recall trace/evidence fields, match reason, matched snippet or digest reference, timestamp/window, confidence or match class, wrong-match evaluation, no-match reason, and "search older saved sessions?" escalation offer.

What must not be duplicated:
No Desktop recall explanation logic. No separate evaluator outside PH1.M/search owners. No hallucinated no-match recovery.

Acceptance criteria:
Recall output can show provenance/session/turn reference where available.
Trace explains match reason without leaking secrets.
Wrong confident single-match cases are detected or downgraded.
No-match says it does not see the topic in recent recall.
No-match can offer deeper archive search without pretending it already searched all memory.

Preservation gates:
Recent recall is not full archive recall. Deeper search is Stage 10.

Deferred items:
Full topic/session-bundle archive recall is Stage 10.

STAGE 10 - Full topic/session-bundle archive recall

BUILD:
SELENE_TOPIC_BUNDLE_RECALL_AND_FULL_SESSION_ARCHIVE_SEARCH_FOUNDATION

Owner:
PH1.M, storage/archive/search, PH1.CONTEXT where bounded context is needed, adapter recall API, Desktop render only.

Purpose:
Move beyond the 72-hour fast tier. Search all eligible saved sessions and compile related discussions into topic/session bundles.

What already exists:
Conversation ledger, recent archive digests, PH1.M memory foundations, archive/recent slice surfaces, bounded context concepts.

What must be built:
Topic grouping, session bundle builder, topic tags, decision extraction, open-task extraction, related-turn retrieval, preference extraction, rejected-option extraction, summary composition, confidence/reranking, full archive fallback, clarification when multiple matching topics exist, and no-record answer.

What must not be duplicated:
No second archive system. No Desktop archive synthesis. No loading all raw history into PH1.X active context.

Acceptance criteria:
Older related sessions can be found beyond recent recall.
Selene compiles what was discussed, decided, left open, and rejected.
Multiple matching topics trigger clarification.
No record returns a truthful no-record answer.
Archive recall does not become active pending context.

Preservation gates:
PH1.M owns memory/archive recall. PH1.X owns current active conversation.

Deferred items:
User-selected recall window is Stage 11.

STAGE 11 - User-selected recall window

BUILD:
SELENE_USER_SELECTED_SESSION_RECALL_WINDOW

Owner:
PH1.M recall policy/scope, storage/archive search, adapter settings/API, Desktop render/control only.

Purpose:
Let the user choose how much saved history Selene can use for recall/search.

What already exists:
Recent recall fast tier and archive surfaces exist partially. Some UI surfaces show recent/archived slices.

What must be built:
Recall scopes for current session only, last 3 sessions, last 10 sessions, last 24 hours, last 72 hours, all saved sessions, custom date range, and custom topic/project. Enforce max open/session scope rules.

What must not be duplicated:
No Desktop-owned recall filtering. No PH1.X memory window. No local archive browser with independent meaning.

Acceptance criteria:
User can select recall scope.
Selected scope affects recall/search only.
Selected history does not become active PH1.X pending context.
Scope is auditable and bounded.

Preservation gates:
PH1.M/search owner controls recall visibility; Desktop renders controls only.

Deferred items:
Permanent memory UX is Stage 16.

STAGE 12 - Session search UI/API

BUILD:
SELENE_SESSION_SEARCH_AND_RECALL_UI_API

Owner:
Storage/archive/search, PH1.M recall/search meaning, adapter API, Desktop render only.

Purpose:
Let the user find old discussions by date, topic, project, person/entity, conversation phrase, decision, open task, source/tool used, protected-action attempt, file/build reference, or semantic meaning.

What already exists:
Conversation ledger, recent rows, archived recent slice surfaces, partial session visibility surfaces.

What must be built:
Search API, result ranking, result snippets, matching reasons, open/restore affordance through lawful session paths, and UI rendering for search results.

What must not be duplicated:
No Desktop search engine. No local provider/source authority. No session reopen shortcut outside PH1.L.

Acceptance criteria:
Search finds relevant saved sessions.
Search does not mutate archive.
Results show summary, snippets, timestamps, match reasons, and confidence/match class where available.
Open/restore uses lawful session lifecycle.

Preservation gates:
Archive/search meaning remains runtime-owned.

Deferred items:
Advanced project memory is Stage 13.

STAGE 13 - Medium-term project memory

BUILD:
SELENE_MEDIUM_TERM_PROJECT_MEMORY_FOUNDATION

Owner:
PH1.M, PH1.LEARN/PH1.KNOW as governed assist, storage/archive, PH1.CONTEXT for bounded context packaging.

Purpose:
Track decisions, blockers, build direction, unresolved plans, and project continuity for roughly 30-90 days without automatically promoting everything to permanent memory.

What already exists:
PH1.M memory foundations, memory ledger/current patterns, learning/knowledge assist owners, recent recall, archive references.

What must be built:
Project decision capture, open task retention, blocker continuity, resolved task closure, expiry/refresh rules, and explicit separation from permanent memory.

What must not be duplicated:
No project memory outside PH1.M/governed learning owners. No Desktop project-state cache. No automatic permanent promotion.

Acceptance criteria:
Project decisions and blockers can be recalled across sessions.
Open tasks remain visible until resolved.
Resolved tasks close cleanly.
Medium-term items do not become permanent memory without governance.

Preservation gates:
Memory remains identity/scope governed and audit-visible.

Deferred items:
Permanent governed memory UX is Stage 16.

STAGE 14 - Semantic repair and intent reranking

BUILD:
SELENE_SEMANTIC_REPAIR_AND_INTENT_RERANKING_FOUNDATION

Owner:
PH1.C, PH1.LANG, PH1.SRL, PH1.N, PH1.X, PH1.M context/recall as bounded evidence, PH1.E for tool fit.

Purpose:
Make Selene handle bad spelling, broken phrasing, misheard speech, messy voice transcripts, and ambiguous intent without guessing protected actions.

What already exists:
Transcript quality gates, language/semantic repair foundations, PH1.SRL, PH1.N, PH1.C, false transcript gates, active context, recent recall, benchmark fixtures.

What must be built:
Transcript candidate collection, CandidateSetBuilder, DeterministicCandidateRanker, CriticalTokenDetector, confidence gate, intent reranker, ambiguity resolver, one-question clarification, correction-pair handoff, and runtime wiring.

What must not be duplicated:
No Desktop semantic repair. No separate intent router. No provider/model shortcut that bypasses PH1.C/SRL/N/X. No protected-action guessing.

Acceptance criteria:
Selene builds multiple candidate meanings.
Candidates are ranked using active context, memory, tool fit, language, and risk.
Selene answers only when confidence is sufficient.
Selene asks one clarification when uncertain.
Protected actions fail closed when ambiguous.

Preservation gates:
PH1.X remains one next conversational directive owner. PH1.M recall remains evidence, not active intent.

Deferred items:
Correction-pair learning is Stage 15.

STAGE 15 - Correction-pair learning and vocabulary packs

BUILD:
SELENE_CORRECTION_PAIR_AND_VOCAB_LEARNING_FOUNDATION

Owner:
PH1.FEEDBACK, PH1.LEARN, PH1.KNOW, PH1.PRON, PH1.C/SRL/N integration, PH1.M governed memory where applicable.

Purpose:
When the user corrects Selene, Selene can improve future hearing and understanding under governance.

What already exists:
Feedback/learn/knowledge owners, vocab/pronunciation assist rows, memory and learning architecture docs, partial correction foundations.

What must be built:
CorrectionPairEmitter, vocab pack updates, name/pronunciation correction, domain terms, consent/governance, forget/update correction, and handoff into semantic repair.

What must not be duplicated:
No uncontrolled Desktop learning. No silent permanent memory write. No provider/model mutation outside governed artifact paths.

Acceptance criteria:
Correction pairs improve future transcript/semantic repair.
User can inspect/update/forget correction pairs where governed.
No sensitive learning without consent and identity scope.
Tenant/user isolation is preserved.

Preservation gates:
Learning remains governed and auditable.

Deferred items:
Permanent governed memory UX is Stage 16.

STAGE 16 - Permanent governed memory UX

BUILD:
SELENE_PERMANENT_GOVERNED_MEMORY_UX

Owner:
PH1.M, PH1.POLICY/GOV/ACCESS where required, storage, adapter API, Desktop render/control only.

Purpose:
Let the user save, review, edit, promote, and forget stable governed memory.

What already exists:
PH1.M memory operations and memory governance foundations.

What must be built:
Remember this, what do you remember, forget that, update that memory, promote to long-term memory, inspect memory, and governed UX/API flows.

What must not be duplicated:
No Desktop memory store. No silent permanent memory write. No permanent promotion from recent recall without governance.

Acceptance criteria:
User can inspect and control permanent memory.
Consent, identity, and policy gates are preserved.
Permanent memory writes are explicit, scoped, and auditable.
Forget/update create governed events.

Preservation gates:
PH1.M remains the memory authority.

Deferred items:
Broader personality/emotional guidance work unless separately authorized.

STAGE 17 - Barge-in, interruption, and natural voice loop

BUILD:
SELENE_BARGE_IN_INTERRUPT_AND_NATURAL_VOICE_LOOP

Owner:
PH1.K for audio/interrupt signals, PH1.X for interruption decision/continuity, PH1.TTS for cancel/playback safety, Desktop for capture/playback transport only.

Purpose:
Make voice conversation feel natural: stop speaking, pause, resume, interrupt, cancel answer, avoid self-echo, handle typed interruption, handle spoken correction, and continue naturally.

What already exists:
PH1.K interrupt candidates, PH1.X interruption continuity tests, PH1.TTS playback/cancel safety foundations, output-interaction boundaries.

What must be built:
Real Desktop barge-in loop, live TTS interruption, spoken correction handling, typed interruption handling, resume behavior, and no self-transcript proof.

What must not be duplicated:
No Desktop interruption decision logic. No local semantic shortcut. No second TTS controller that bypasses PH1.TTS safety.

Acceptance criteria:
User can interrupt Selene.
Selene stops speaking cleanly.
Next user turn is captured.
Interrupted output cannot be treated as delivered if it was not delivered.
No self-echo or stale output creates a user turn.

Preservation gates:
PH1.X owns conversation continuity. PH1.TTS/runtime owns playback/cancel policy.

Deferred items:
Advanced voice personality/tone tuning unless separately authorized.

STAGE 18 - Protected simulation and business execution workflows

BUILD:
SELENE_PROTECTED_SIMULATION_BUSINESS_EXECUTION_WORKFLOWS

Owner:
SimulationExecutor, PH1.LAW, PH1.OS, PH1.X, ACCESS/POLICY/GOV, storage/audit/work-order owners, adapter/Desktop render only.

Purpose:
Move from assistant to governed business runtime for protected workflows such as payroll, salary changes, leave approval, inventory update, customer record changes, financial actions, POS, and business operations.

What already exists:
Protected fail-closed foundation, simulation executor, authority/access/policy/governance paths, work-order/audit foundations.

What must be built:
Concrete governed business workflows with simulation, authority, audit, idempotency, approval, replay safety, and fail-closed product UX.

What must not be duplicated:
No protected execution outside simulation. No Desktop approval shortcut. No provider/tool mutation without authority. No public chat route completing protected actions.

Acceptance criteria:
Public answers remain probabilistic/read-only where allowed.
Protected actions require simulation, authority, audit, and explicit completion proof.
No simulation means no protected execution.
Ambiguous protected requests clarify or fail closed.

Preservation gates:
Protected execution stays deterministic and simulation-gated.

Deferred items:
Any real external business connector mutation until its connector-specific authority, simulation, and audit gates are certified.

RECOMMENDED BUILD ORDER FROM CURRENT REPO TRUTH

1. Prove current Desktop app, wake, listening, re-arm, and hidden-screen conversation in one clean real-app smoke.
2. Complete the background availability icon/status full state model.
3. Complete the local wake privacy boundary.
4. Build voice-controlled screen show/hide.
5. Prove voice and typed input parity.
6. Complete safe logical session boundary and new-session control proof.
7. Certify immutable session archive and black-box ledger.
8. Pass real recent recall voice smoke and indexing certification.
9. Add recent recall trace/eval and no-match/deeper-search offer.
10. Build full topic/session-bundle archive recall.
11. Build user-selected recall window.
12. Build session search UI/API.
13. Build medium-term project memory.
14. Build semantic repair and intent reranking.
15. Build correction-pair/vocab learning.
16. Build permanent governed memory UX.
17. Build barge-in/natural interruption.
18. Build protected simulation/business workflows.

WHY DESKTOP COMES FIRST

If the real app cannot reliably prove the current bundle, wake, listen, capture, speak, re-arm, hide/show, and avoid stale app/adapter contamination, higher-level memory and semantic work will feel broken even when runtime tests pass.

WHY 72-HOUR RECALL IS NOT THE FINAL MEMORY PRODUCT

The 72-hour window is the fast recent recall tier. It is useful for recent discussions and smoke tests, but it is not the full memory system. The deeper memory target is full topic/session-bundle archive recall across saved sessions.

WHY ACTIVE CONTEXT AND RECALL STAY SEPARATE

PH1.X active context answers what Selene is talking about right now. PH1.M recall retrieves what was said before. A recalled mention of Sydney must not become a pending weather/time slot. A current follow-up like "and London" must stay PH1.X active context, not PH1.M recall.

EXPLICITLY DEFERRED / LATER CAPABILITIES

These are not next unless JD reprioritizes them, but they are not removed:

- advanced proactive UI behavior beyond show/hide and availability state
- advanced voice personality/tone tuning beyond OpenAI-first TTS correctness and natural interruption
- broad emotional guidance/persona work beyond governed memory and correction learning
- live external business connector mutation before connector-specific simulation, authority, audit, and replay gates are certified
- user-selected recall UX polish beyond the runtime-safe scope controls
- full semantic repair expansions that require topic/session-bundle archive context before the archive layer is reliable

MASTER ACCEPTANCE CRITERIA

This master plan is complete only when Selene can:

- wake from background
- acknowledge readiness
- listen while the screen is hidden
- answer by voice
- write the conversation into the timeline
- show/hide the screen by voice
- allow voice and typing through the same runtime path
- create safe internal session boundaries
- prevent stale context leaks
- seal immutable sessions and black-box proof events
- search recent and old sessions
- compile related topic bundles
- let the user select recall scope
- recall project context
- repair bad spelling and broken phrases
- learn corrections under governance
- handle permanent memory safely
- allow interruption/barge-in
- execute protected business actions only through simulation plus authority

HARD PRODUCT RULE

Selene is always available.
The screen is optional.
The conversation is continuous.
Sessions are internal and immutable.
Memory is searchable.
Recall is not active context.
Desktop is not the brain.
Protected execution is simulation-gated.

This corrected rewritten plan is ready for JD review and remains uncommitted until JD approves.
