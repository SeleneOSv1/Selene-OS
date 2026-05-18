# Stage 8.5A PH1.X Algorithmic Audit, Patchwork Cleanup Plan

Task: `SELENE_STAGE_8_5A_PH1X_ALGORITHMIC_AUDIT_PATCHWORK_CLEANUP_AND_BUILD_PLAN`
Date: 2026-05-18
Repo: `/Users/selene/Documents/Selene-OS`
Baseline required: Stage 8 Fresh Memory Real Voice Proof `e36e323d9730daedde4613322870181851f524da`

## 1. Executive Conclusion

Stage 8.5 is not complete. The repo has a real PH1.X foundation for active context, and the Stage 8.5 design is now present in docs, but the live current-user-turn understanding spine is still partial.

The current repo truth is:

- PH1.X owns the canonical contract direction through `ActiveContextPacket`, `HumanConversationDirective`, `ThreadState`, `LastTurnContext`, and partial Stage 8.5 OS helpers.
- PH1.X already has many required active-frame fields, including planning, writing, clarification, correction, topic stack, confidence, ambiguity, protected risk, and speaker continuity fields.
- PH1.X OS currently has a partial Stage 8.5 helper for active planning, writing artifacts, tool-choice clarification, correction, and protected confirmation continuation.
- Adapter still has active wrong-owner semantic compatibility surfaces: `deterministic_active_context_followup_query`, `deterministic_weather_context_followup_query`, H380/H411/H412 public discourse and protected/payroll routing helpers.
- PH1.M Stage 8 fresh memory is active but narrow. It can support time fresh continuation from durable evidence, but it does not yet provide the requested `FreshMemoryCapsule` or general recent-context capsule.
- No safe dead local phrase patch was removed in Stage 8.5A. The risky paths found are either active compatibility or canonical-but-incomplete PH1.X partial implementation. Removing them before a replacement algorithm is wired would break existing behavior.

Readiness statement:

`READY_FOR_STAGE_8_5B`

Stage 8.5B should be a contract/frame completion slice. It should not try to remove adapter compatibility yet.

## 2. Current Repo Baseline

Start proof:

- Branch: `main`
- Start HEAD: `55d46904ecae1f3bfbd4dc5bffeedc464f795b8a`
- `origin/main`: `55d46904ecae1f3bfbd4dc5bffeedc464f795b8a`
- Tree at start: clean
- Stage 8 ancestor proof: `STAGE8_ANCESTOR_OK`

Required lane declaration:

- Current project phase: `PROBABILISTIC_FOUNDATION_BUILD`
- Selected lane: `PROBABILISTIC_PUBLIC_ANSWER / internal architecture audit and cleanup`
- Simulation required: no
- Authority required: no
- Runtime behavior mutation: no, except approved dead local cleanup
- Protected execution allowed: no
- Provider degradation allowed: yes, no provider calls expected
- Fail-closed preservation: required

## 3. Required Reviews Completed

Files and documents reviewed:

- `AGENTS.md`, including section `5.10 Universal Algorithmic Implementation / No Patchwork Law`
- `docs/SELENE_ALWAYS_AVAILABLE_VOICE_CONTINUOUS_CHAT_SESSION_MEMORY_MASTER_PLAN.md`
- `docs/reports/STAGE_8_FRESH_MEMORY_REAL_VOICE_PROOF.md`
- `docs/STAGE_8_5_PH1X_CURRENT_USER_TURN_UNIVERSAL_UNDERSTANDING_ENGINE.md`
- `docs/reports/STAGE_8_5_PH1X_UNIVERSAL_ACTIVE_CONTEXT_REPAIR.md`

Relevant repo law applied:

- JD examples are test cases, not the production algorithm.
- No phrase patches, exact string hacks, toy shortcuts, wrong-owner helper hacks, Desktop semantic fallback, or adapter-owned context brain may be introduced.
- Existing old paths must be classified and removed only when the canonical owner replacement is wired and tests prove preserved behavior.

## 4. PH1.X Current State

Canonical PH1.X owner files:

- `crates/selene_kernel_contracts/src/ph1x.rs`
- `crates/selene_os/src/ph1x.rs`
- `crates/selene_engines/src/ph1x.rs`

Related transport/runtime files:

- `crates/selene_adapter/src/lib.rs`
- `crates/selene_os/src/app_ingress.rs`

Related owner boundaries:

- PH1.M: `crates/selene_kernel_contracts/src/ph1m.rs`, `crates/selene_engines/src/ph1m.rs`
- PH1.E: `crates/selene_engines/src/ph1e.rs`
- PH1.WRITE: `crates/selene_engines/src/ph1write.rs`
- Voice ID: `crates/selene_kernel_contracts/src/ph1_voice_id.rs`, `crates/selene_engines/src/ph1_voice_id.rs`, `crates/selene_os/src/ph1_voice_id.rs`
- Desktop: not touched, not an owner

### 4.1 ActiveContextPacket

`crates/selene_kernel_contracts/src/ph1x.rs` already defines `ActiveContextPacket` with these Stage 6.6 and Stage 8.5-relevant fields:

- `active_topic`
- `active_intent`
- `interaction_posture`
- `conversation_rhythm`
- `continuation_type`
- `reference_target`
- `entity_focus`
- `tool_family`
- `writing_artifact`
- `pending_slots`
- `correction_target`
- `topic_stack`
- `response_shape`
- `confidence`
- `ambiguity_level`
- `protected_risk`
- `memory_handoff_needed`
- `suggested_next_engine`
- `user_goal`
- `current_plan`
- `open_question`
- `unresolved_decision`
- `prior_options_presented`
- `selected_option`
- `rejected_option`
- `comparison_set`
- `constraints`
- `user_preference_in_turn`
- `expected_answer_type`
- `last_clarification_question`
- `clarification_answer_target`
- `discourse_state`
- `topic_depth`
- `returnable_topic`
- `interruption_state`
- `speaker_continuity`
- `confidence_reason`
- `why_not_continue_reason`
- `evidence_refs`

Missing or not yet clearly represented in PH1.X contract:

- `raw_user_turn_ref`
- `normalized_user_turn_ref`
- `modality`
- `last_answer_type`
- `why_continue_reason`
- `selected_candidate`
- `rejected_candidates`
- per-candidate scoring fields
- hard-disqualifier proof fields
- explicit context decay tier
- explicit multi-thread conversation state contract
- explicit open-loop packet
- `CompositeHumanConversationDirective`

Stage 8.5B should decide whether these are added directly to `ActiveContextPacket`, represented as nested proof structs, or mapped to existing refs.

### 4.2 HumanConversationDirective

`crates/selene_kernel_contracts/src/ph1x.rs` defines canonical variants:

- `ContinueCurrentTopic`
- `ModifyPreviousOutput`
- `CorrectPreviousOutput`
- `AnswerNewQuestion`
- `AskClarification`
- `HandOffToMemory`
- `RouteToTool`
- `RouteToWrite`
- `FailClosedProtected`
- `WaitOrNoAction`

Current limitation:

- `From<&Ph1xDirective> for HumanConversationDirective` still maps broad runtime directives into coarse directives. It does not yet expose the full Stage 8.5 candidate/scoring decision chain.
- No composite directive exists yet for split public/protected multi-intent turns.

### 4.3 ThreadState and LastTurnContext

`ThreadState` and `LastTurnContext` are active PH1.X state foundations. Relevant fields include:

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

`LastTurnContext` includes route class and answer/tool/provider hints. This is useful but is not yet the requested generalized candidate target model.

### 4.4 Current PH1.X Runtime/OS Behavior

`crates/selene_os/src/ph1x.rs` contains the partial Stage 8.5 functions:

- `ph1x_universal_active_context_followup_query`
- `ph1x_update_universal_active_context_after_turn`

Current behavior already supports partial:

- planning frame pinning
- activity/constraint extraction
- writing artifact tracking
- tool choice clarification for time/weather
- weather/time correction
- protected confirmation fail-closed continuity
- topic-switch suppression for obvious new questions

Current limitation:

- The implementation is not yet a full candidate generation plus scoring plus hard-disqualifier model.
- Some production reference resolution still uses exact prefix surfaces and compact vocab arrays.
- Output to Adapter is still a rewritten query string, not a fully surfaced `HumanConversationDirective` with candidate proof fields.

## 5. How Much Stage 8.5 Is Already Built

| Stage 8.5 requirement | Exists / partial / missing | Current owner path | Evidence from repo | Risk | Build slice needed |
|---|---:|---|---|---|---|
| Current-user-turn pipeline | Partial | `crates/selene_os/src/ph1x.rs`, `crates/selene_adapter/src/lib.rs` | PH1.X helper exists; Adapter still coordinates much of the turn path | Wrong-owner fallback remains active | 8.5B, 8.5C |
| Input source validation | Exists outside PH1.X | PH1.C/Adapter/OS | PH1.C and ingress gate committed turns | PH1.X must consume, not duplicate | No PH1.X rewrite in 8.5A |
| Speaker posture input | Partial | Voice ID, OS, PH1.X contract | `speaker_continuity` field exists; Voice ID evidence path exists | Policy use is not fully tied into scoring | 8.5B, 8.5C |
| Language normalization | Partial | PH1.X/Adapter/PH1.M | Tokenizers exist in several places | Duplicated normalization surfaces | 8.5C |
| Interaction posture detection | Partial | PH1.X contract/OS, Adapter H380 | PH1.X fields exist; H380 has active packet | Adapter still does meaning | 8.5C, 8.5E |
| Active frame load | Partial | PH1.X | ThreadState refs and Stage8_5 subject refs exist | Not multi-thread or scored | 8.5B, 8.5C |
| Reference resolution | Partial | PH1.X OS and Adapter | Reference term vocab and prefix extraction exist | Phrase-style surfaces remain | 8.5C |
| Continuation type decision | Partial | PH1.X OS, Adapter fallback | PH1.X helper before adapter deterministic fallback | Still string rewrite based | 8.5C, 8.5D |
| Confidence/ambiguity scoring | Partial | PH1.X contract, Adapter H380 | fields exist; H380 has scores | No universal scoring in PH1.X | 8.5C |
| HumanConversationDirective output | Partial | PH1.X contract | canonical enum exists | Runtime still mostly `Ph1xDirective` and query rewrite | 8.5B, 8.5C |
| Correct owner routing | Partial | Adapter/PH1.X/PH1.E/PH1.WRITE | PH1.X is imported into Adapter | Adapter still semantic bridge | 8.5D, 8.5G |
| Scenario family A tool continuation | Partial | PH1.X OS, Adapter | time/weather helper works for some forms | too narrow and partly adapter-owned | 8.5C, 8.5D |
| Scenario family B planning continuation | Partial | PH1.X OS | planning frame helper exists | not candidate/scored; weak real proof | 8.5D |
| Scenario family C writing continuation | Partial | PH1.X OS, PH1.WRITE | writing artifact helper exists | PH1.WRITE handoff not canonical enough | 8.5D |
| Scenario family D clarification answer | Partial | PH1.X OS | tool-choice target exists | time/weather-specific | 8.5E |
| Scenario family E correction | Partial | PH1.X OS, Adapter | correction helper exists | not generalized beyond tool family | 8.5E |
| Scenario family F topic switch | Partial | PH1.X OS | new-topic detector exists | no hard-disqualifier proof | 8.5C, 8.5E |
| Scenario family G return to topic | Partial | ThreadState | interrupted/return refs exist | no multi-thread selector | 8.5C |
| Scenario family H memory after sleep | Partial | PH1.M and Adapter | Stage 8 time fresh memory works | no FreshMemoryCapsule, PH1.X decision split unclear | 8.5F |
| Scenario family I vague fragment | Partial | PH1.M/PH1.X | some fragment handling exists | no universal ambiguity scoring | 8.5C |
| Scenario family J protected continuation | Partial | Adapter and PH1.X refs | protected prefix refs; adapter payroll helpers | wrong-owner active compatibility | 8.5E |
| Scenario family K public/protected distinction | Partial | Adapter H380/H411 | public payroll/protected payroll helpers exist | wrong-owner, phrasey | 8.5E |
| Scenario family L multi-intent | Partial | Adapter H380 | H380 can detect multi-intent | no composite directive | 8.5B, 8.5E |
| Scenario family M speaker change | Partial | Voice ID/PH1.X | speaker continuity field exists | not tied into disqualifiers | 8.5C |
| Scenario family N emotional/frustrated | Missing/partial | Adapter H380 | some correction/provenance markers exist | no PH1.X posture algorithm | 8.5C, 8.5E |
| Scenario family O thinking out loud | Missing | none clear | no PH1.X proof | may over-route | 8.5C |
| Scenario family P no-action rhythm | Partial | PH1.X/Adapter | wait/no-action directive exists | no rhythm classifier | 8.5C |
| Scenario family Q file/image context | Missing/partial | PH1.VISION/Adapter | multimodal refs exist elsewhere | no PH1.X artifact selector | Later 8.5 split or Stage 9+ |
| Scenario family R search/web context | Missing/partial | PH1.E/Adapter | PH1.E evidence exists | no PH1.X search-context continuity | Later 8.5 split |
| Scenario family S formatting/presentation | Partial | PH1.WRITE | PH1.WRITE exists | no PH1.X response-shape algorithm | 8.5D |
| Scenario family T broken/accent speech | Partial | PH1.C/PH1.N/PH1.SRL | language and SRL contracts exist | PH1.X not wired to universal roles | 8.5C |
| Active frame fields | Partial | PH1.X contract | many fields already exist | missing candidate/proof fields | 8.5B |
| Candidate generation | Missing | none clear in PH1.X | no `ContextCandidate` model | phrase-style shortcut risk | 8.5C |
| Scoring model | Missing/partial | Adapter H380 only | H380 has confidence; PH1.X lacks universal score | wrong owner | 8.5C |
| Hard disqualifiers | Missing/partial | Adapter/protected gates | protected fail closed exists | not PH1.X candidate gate | 8.5C, 8.5E |
| FreshMemoryCapsule | Missing | PH1.M docs only | no symbol found in crates | Stage 8.5F needed | 8.5F |
| Context decay | Partial | PH1.M resume windows, PH1.L session | no PH1.X decay tier | stale hijack risk | 8.5F |
| Open loop tracking | Partial | ThreadState pending/open fields | fields exist, no generalized open-loop model | weak clarification answers | 8.5B, 8.5E |
| Multi-thread conversation state | Missing/partial | ThreadState refs only | no ConversationThread type | return-to-topic weak | 8.5C |
| Intention vs literal separation | Missing/partial | H380 packet in Adapter | adapter has literal/normalized/intent fields | wrong owner | 8.5C |
| Mutation tests | Partial | PH1.X tests have some paraphrases | not full CI gate | phrase patches can slip | 8.5G |
| Phrase-patch CI gate | Missing/partial | AGENTS law only | no code CI gate in repo | manual scan required | 8.5G |

## 6. Phrase-Patch and Shortcut Classification

No active production path was removed in Stage 8.5A.

| Path | Symbol/function/test | Phrase/symbol type | Classification | Production reachable | Current behavior | Remove now? | Replacement needed |
|---|---|---|---|---:|---:|---:|---|
| `crates/selene_os/src/ph1x.rs` | `STAGE8_5_*` vocabulary arrays | planning, activity, selection, timing, artifact, reference terms | `DOMAIN_VOCABULARY_OK` with limitations | yes | yes | no | convert to structured feature extraction plus candidate scoring in 8.5C |
| `crates/selene_os/src/ph1x.rs` | `ph1x_universal_active_context_followup_query` | partial PH1.X algorithm | `RETAINED_COMPATIBILITY_PATH` | yes | yes | no | replace string rewrite with directive/candidate proof in 8.5C/D |
| `crates/selene_os/src/ph1x.rs` | `ph1x_stage8_5_entity_fragment` | prefix surfaces like same/what-about/and/also | `RETAINED_COMPATIBILITY_PATH` | yes | yes | no | generalized reference resolver in 8.5C |
| `crates/selene_os/src/ph1x.rs` | `ph1x_stage8_5_tool_choice_from_response` | tool-choice parse from answer text | `RETAINED_COMPATIBILITY_PATH` | yes | yes | no | explicit open clarification target packet in 8.5B/E |
| `crates/selene_os/src/ph1x.rs` | Stage8_5 tests | exact JD examples and paraphrases | `TEST_FIXTURE_OK` | no | yes | no | keep until replaced/expanded by mutation suite |
| `crates/selene_adapter/src/lib.rs` | `deterministic_active_context_followup_query` | active context shortcut | `RETAINED_COMPATIBILITY_PATH` and `WRONG_OWNER_SURFACE` | yes | yes | no | PH1.X candidate/scoring replacement in 8.5C/G |
| `crates/selene_adapter/src/lib.rs` | `deterministic_weather_context_followup_query` | weather/time context shortcut, hardcoded city examples | `RETAINED_COMPATIBILITY_PATH` and `WRONG_OWNER_SURFACE` | yes | yes | no | PH1.X tool-continuity selector in 8.5D/G |
| `crates/selene_adapter/src/lib.rs` | H380 packet helpers | contains/starts_with exact public discourse markers | `RETAINED_COMPATIBILITY_PATH` and `WRONG_OWNER_SURFACE` | yes | yes | no | PH1.X current-turn algorithm plus owner routing in 8.5C/E |
| `crates/selene_adapter/src/lib.rs` | `payroll_public_business_knowledge_intent` | payroll public/protected split | `RETAINED_COMPATIBILITY_PATH` | yes | yes | no | protected-risk domain classifier in PH1.X/protected owner in 8.5E |
| `crates/selene_adapter/src/lib.rs` | `payroll_governed_business_intent` | payroll protected verbs | `RETAINED_COMPATIBILITY_PATH` | yes | yes | no | protected-risk classifier and composite directive in 8.5E |
| `crates/selene_adapter/src/lib.rs` | H411/H412 discourse helpers | direct public answers and follow-up discourse | `WRONG_OWNER_SURFACE` retained | yes | yes | no | PH1.X/PH1.WRITE split after current-turn algorithm exists |
| `crates/selene_engines/src/ph1m.rs` | `fresh_memory_followup_location` | exact prefix list for fresh time follow-up | `RETAINED_COMPATIBILITY_PATH` | yes | yes | no | PH1.M `FreshMemoryCapsule` plus PH1.X decision owner in 8.5F |
| `crates/selene_engines/src/ph1m.rs` | `resolve_fresh_memory_continuation` | time-only fresh continuation | `RETAINED_COMPATIBILITY_PATH` | yes | yes | no | generalized capsule evidence, not final directive, in 8.5F |
| `docs/**` | Stage reports/design docs | exact JD examples | `REPORT_OK` | no | no | no | keep |
| `crates/**/tests` | tests and fixtures | exact JD examples | `TEST_FIXTURE_OK` | no | no | no | expand with unseen paraphrases in 8.5G |

Potentially dangerous stale surfaces:

- Adapter deterministic context functions are still reachable after PH1.X helper misses.
- Adapter H380/H411/H412 still contains semantic meaning and public/protected routing behavior.
- PH1.M fresh continuation currently decides a narrow time continuation. Stage 8.5 target says PH1.M should provide evidence while PH1.X decides live continuation.

These are not safe to remove in Stage 8.5A because they are live compatibility behavior and no full replacement algorithm is wired yet.

## 7. Safe Cleanup Performed

No cleanup was performed.

Reason:

- No phrase patch was proven dead, unreachable, local, and safely obsolete.
- The visible patchwork either preserves current working behavior or belongs to a partial canonical PH1.X foundation that later slices must replace carefully.
- Removing active adapter fallbacks before a PH1.X candidate/scoring replacement would likely regress Stage 8 and current live behavior.

## 8. Retained Compatibility Paths

`RETAINED_COMPATIBILITY_PATH`:

- `crates/selene_os/src/ph1x.rs::ph1x_universal_active_context_followup_query`
  - Owner: PH1.X OS layer
  - Why retained: current partial Stage 8.5 behavior depends on it.
  - Retirement condition: replaced by PH1.X candidate generation/scoring/directive emission with equivalent or better tests.

- `crates/selene_adapter/src/lib.rs::deterministic_active_context_followup_query`
  - Owner today: Adapter
  - Why retained: still reachable fallback when PH1.X helper misses.
  - Retirement condition: PH1.X handles active follow-up through candidate/scoring and full adapter tests prove no regressions.

- `crates/selene_adapter/src/lib.rs::deterministic_weather_context_followup_query`
  - Owner today: Adapter
  - Why retained: still supports current weather/time follow-up behavior.
  - Retirement condition: PH1.X owns tool-continuity context and weather/time tests pass through PH1.X evidence.

- `crates/selene_adapter/src/lib.rs` H380/H411/H412 helpers
  - Owner today: Adapter
  - Why retained: active public discourse, protected fail-closed, and follow-up behavior use them.
  - Retirement condition: PH1.X/PH1.WRITE/protected owner split replaces behavior with packet proof.

- `crates/selene_engines/src/ph1m.rs::resolve_fresh_memory_continuation`
  - Owner: PH1.M
  - Why retained: Stage 8 fresh memory proof depends on it.
  - Retirement condition: PH1.M emits `FreshMemoryCapsule` evidence and PH1.X owns live continuation decision.

## 9. Wrong-Owner / Stale-Dangerous Paths

Wrong-owner or stale-dangerous paths found:

- Adapter deterministic active context fallback:
  - `crates/selene_adapter/src/lib.rs::deterministic_active_context_followup_query`
  - Risk: Adapter can continue acting as semantic context brain after PH1.X misses.
  - Future action: replace after Stage 8.5C/D and remove in Stage 8.5G when tests prove parity.

- Adapter deterministic weather context fallback:
  - `crates/selene_adapter/src/lib.rs::deterministic_weather_context_followup_query`
  - Risk: time/weather-only patches hide PH1.X gaps and do not generalize to planning/writing.
  - Future action: replace with PH1.X tool-continuity scoring.

- Adapter H380/H411/H412:
  - Risk: public discourse, payroll, and reference behavior remain close to string-driven route code.
  - Future action: split into PH1.X current-turn understanding, PH1.WRITE language, PH1.E tool/search, and protected owner gates.

- PH1.M fresh time continuation:
  - Risk: PH1.M currently produces a narrow continuation result instead of a general recent-context capsule.
  - Future action: PH1.M emits evidence; PH1.X selects continue/clarify/switch.

## 10. Owner Map for Related Files

| File | Likely future Stage 8.5 change | Why | Expected slice | Risk | JD approval needed |
|---|---:|---|---|---|---:|
| `crates/selene_kernel_contracts/src/ph1x.rs` | yes | add missing frame/candidate/scoring/proof fields or wrappers | 8.5B | contract churn | yes, via slice task |
| `crates/selene_engines/src/ph1x.rs` | maybe | engine-level algorithm may move here if repo wants engines owner over OS helper | 8.5C | owner placement | yes |
| `crates/selene_os/src/ph1x.rs` | yes | current partial Stage 8.5 helper lives here | 8.5C-E | high behavioral blast radius | yes |
| `crates/selene_os/src/app_ingress.rs` | maybe | may need to pass PH1.L/session/speaker evidence into PH1.X frame | 8.5F | runtime ingress coupling | maybe |
| `crates/selene_adapter/src/lib.rs` | yes later | bridge PH1.X packets, then retire fallbacks | 8.5D-G | current live behavior depends on it | yes |
| `crates/selene_engines/src/ph1m.rs` | yes | FreshMemoryCapsule/evidence handoff | 8.5F | Stage 8 regression risk | yes |
| `crates/selene_engines/src/ph1e.rs` | no/low | PH1.E should receive resolved tool context only | 8.5D maybe tests | do not make PH1.E context owner | maybe |
| `crates/selene_engines/src/ph1write.rs` | maybe | PH1.WRITE may need better route inputs for rewriting/formatting | 8.5D | avoid PH1.X becoming final writer | maybe |
| Storage/evidence files | maybe | evidence refs for new PH1.X proof packets | 8.5B/G | schema expansion risk | ask before schema |
| Desktop files | no | Desktop renders only | none | touching Desktop would violate owner law | yes, only if transport bug proven |

## 11. Recommended Stage 8.5 Slice Plan

One-shot Stage 8.5 is unsafe. The work crosses contracts, PH1.X state, adapter bridge, PH1.M after-sleep evidence, PH1.WRITE handoff, and live Desktop proof. Use slices.

### Stage 8.5B - ActiveContextPacket Frame Field Completion

Purpose:

- Complete or wrap missing active-frame and proof fields.
- Add contract-level candidate/scoring/hard-disqualifier evidence shape if needed.
- Decide where `raw_user_turn_ref`, `normalized_user_turn_ref`, `modality`, `last_answer_type`, `why_continue_reason`, candidate fields, decay tier, and composite directive live.

Expected files:

- `crates/selene_kernel_contracts/src/ph1x.rs`
- Possible tests in `crates/selene_kernel_contracts`

Old patch paths replaced:

- None yet. This is contract foundation.

Tests:

- contract serialization/defaults/validation
- no duplicate canonical PH1.X packet
- phrase-patch scan

JD live tests:

- Not required for contract-only slice, but final Stage 8.5G must live-test.

Pass gate:

- New fields/proof structs compile, no Desktop/Adapter ownership change.

### Stage 8.5C - Candidate Generation, Scoring, and Hard Disqualifiers

Purpose:

- Build PH1.X current-user-turn candidate generation.
- Score active frame, latest answer, writing artifact, tool result, plan, clarification, topic stack, FreshMemory evidence if provided, and no-context fallback.
- Add hard disqualifiers for protected risk, topic switch, stale context, speaker/private mismatch, rejected evidence, closed topic, and wrong artifact type.

Expected files:

- `crates/selene_os/src/ph1x.rs` or `crates/selene_engines/src/ph1x.rs`, depending owner split
- PH1.X tests

Old patch paths replaced:

- Internal PH1.X prefix-only reference surfaces should become scoring features.

Tests:

- original examples and unseen paraphrases
- negative hijack tests
- protected fail-closed candidate rejection

JD live tests:

- Small current-app proof for time follow-up, topic switch, planning follow-up if runtime behavior changes.

Pass gate:

- PH1.X evidence shows selected/rejected candidates and confidence reasons.

### Stage 8.5D - Tool, Planning, Writing Continuation

Purpose:

- Route resolved tool continuations to PH1.E.
- Route writing artifact modifications to PH1.WRITE.
- Preserve planning constraints, comparison set, prior options, and open decisions.

Expected files:

- `crates/selene_os/src/ph1x.rs`
- `crates/selene_adapter/src/lib.rs` bridge only
- maybe `crates/selene_engines/src/ph1write.rs`

Old patch paths replaced:

- Adapter deterministic time/weather fallbacks start becoming redundant but should not be removed until 8.5G.

Tests:

- time/tool paraphrases
- Japan/Canada/New Zealand planning paraphrases
- story/draft rewrite paraphrases

JD live tests:

- time active context
- Japan planning
- writing artifact modification

Pass gate:

- PH1.E/PH1.WRITE act only after PH1.X emits resolved directive/proof.

### Stage 8.5E - Clarification, Correction, Topic Switch, Protected Gates

Purpose:

- Generalize clarification target tracking.
- Generalize correction target handling.
- Prevent old context from stealing new questions.
- Split public/protected multi-intent turns and fail closed.

Expected files:

- `crates/selene_os/src/ph1x.rs`
- protected owner bridge if required
- adapter bridge only

Old patch paths replaced:

- Adapter payroll/protected phrase helpers should become candidates for later retirement.

Tests:

- Brisbane/Melbourne/the-time clarification
- not weather/time correction
- name/joke topic switch
- organize payroll/yes-do-it protected continuation
- tell me about payroll public distinction

JD live tests:

- clarification and protected proof.

Pass gate:

- PH1.X can understand protected continuation but cannot execute it.

### Stage 8.5F - PH1.M FreshMemoryCapsule and PH1.L Decay Integration

Purpose:

- Add PH1.M recent-context evidence capsule after live context expires.
- Let PH1.X own final continue/clarify/switch decision.
- Derive HOT/WARM/COLD/NO context tiers from PH1.L plus PH1.M evidence.

Expected files:

- `crates/selene_kernel_contracts/src/ph1m.rs`
- `crates/selene_engines/src/ph1m.rs`
- `crates/selene_os/src/ph1x.rs`
- adapter bridge

Old patch paths replaced:

- PH1.M time-only fresh continuation should become retained legacy or move behind capsule path.

Tests:

- New York sleep/wake Sydney
- Japan sleep/wake which city
- draft sleep/wake make warmer
- adapter restart durability still works

JD live tests:

- fresh memory after sleep, including non-time continuation.

Pass gate:

- Stage 8 does not regress, and PH1.X owns continuation decision.

### Stage 8.5G - Full Live Proof, Mutation Suite, and Compatibility Retirement

Purpose:

- Run full live matrix.
- Add mutation/paraphrase test suite.
- Remove adapter wrong-owner fallbacks only when safe.

Expected files:

- tests
- report
- adapter cleanup only if replacement is wired

Old patch paths replaced:

- `deterministic_active_context_followup_query`
- `deterministic_weather_context_followup_query`
- selected H380/H411/H412 surfaces where covered by canonical owners

Tests:

- full package tests plus targeted PH1.X/adapter regressions.

JD live tests:

- all scenarios in Section 12.

Pass gate:

- no exact phrase patches in production logic
- unseen paraphrases pass
- final clean tree and pushed report.

## 12. Full Live Test Plan

Before every JD live test after any code change:

1. Rebuild the latest app from current HEAD.
2. Close stale Selene app instances.
3. Launch the newly built app.
4. Prove exact app bundle path.
5. Prove current repo HEAD.
6. Prove app PID.
7. Prove adapter PID.
8. Prove one app instance.
9. Prove one managed adapter/port owner.
10. Prove health/provenance endpoint where available.

Backend evidence required for every scenario:

- exact captured transcript or typed text
- response_text
- tts_text if voice
- turn_id
- correlation_id
- thread_key/internal segment/session ref
- modality
- PH1.C status for voice
- PH1.X ActiveContextPacket fields
- HumanConversationDirective
- candidate/score/ambiguity/confidence proof once built
- reference target if applicable
- tool family if applicable
- writing artifact if applicable
- correction target if applicable
- topic-switch reason if applicable
- PH1.M evidence only after sleep/wake or memory need
- PH1.E route if applicable
- PH1.WRITE route if applicable
- protected risk and no-execution proof if applicable
- Voice ID speaker evidence for voice
- Stage 7 durable evidence row/ref

### Final Stage 8.5G Live Scenarios

1. Time active context:
   - JD: "What time is it in New York?"
   - JD: "What about Sydney?"
   - Pass: Sydney time, PH1.X continuation, PH1.E time route, no PH1.M needed inside active window.

2. Time after sleep/wake:
   - JD: "What time is it in New York?"
   - Wait for sleep.
   - JD: "Selene"
   - JD: "What about Sydney?"
   - Pass: PH1.M evidence if needed, PH1.X restored context, PH1.E time route, no mechanical wording.

3. Clarification target:
   - JD: "What time is it in Brisbane?"
   - JD: "And Melbourne"
   - If asked weather/time, JD: "The time"
   - Pass: Melbourne time; no repeated "what do you mean by the time?"

4. Japan planning, city:
   - JD: "I'm interested in Japan and doing some skiing and visiting great Japanese restaurants."
   - JD: "Which city do you suggest?"
   - Pass: stays in Japan and combines ski/food constraints.

5. Japan planning, areas:
   - Same setup.
   - JD: "Which areas do you suggest?"
   - Pass: stays in Japan and recommends Japanese areas.

6. Planning unseen paraphrases:
   - Same setup.
   - JD: "Where would you base the trip?"
   - JD: "What option makes most sense?"
   - Pass: same planning frame, not phrase-specific.

7. Writing artifact:
   - JD types: "Write me a short story about a locked factory."
   - JD types: "Make it shorter."
   - JD types: "Tighten it."
   - Pass: modifies previous story, no clarification needed.

8. Draft/message artifact:
   - JD types: "Draft a message to Mark saying I'll come back next week."
   - JD types: "Make it warmer."
   - JD types: "Shorten the draft."
   - JD types: "Add that I'll confirm timing soon."
   - Pass: keeps Mark and next-week meaning.

9. Correction:
   - JD: "What is the weather in Sydney?"
   - JD: "Not weather, time."
   - Pass: answers Sydney time, correction target is previous weather route.

10. Topic switch from time:
    - JD: "What time is it in New York?"
    - JD: "What is your name?"
    - Pass: answers identity/name; no time route.

11. Topic switch from planning:
    - JD: Japan planning setup.
    - JD: "Tell me a joke."
    - Pass: joke/new topic; Japan not hijacking.

12. Protected continuation:
    - JD: "Organize payroll for Tim."
    - JD: "Yes, do it."
    - Pass: protected fail closed, no execution, memory/Voice ID do not authorize.

13. Public payroll distinction:
    - JD: "Tell me about payroll."
    - Pass: public/business knowledge answer, no workflow execution.

14. Noise/cough:
    - JD wakes Selene and coughs or makes safe noise.
    - Pass: no committed user turn, no memory update, no answer, rejected evidence if captured.

15. Speaker changed/unknown speaker:
    - If a second speaker is available, run changed-speaker follow-up.
    - If unavailable, report `SECOND_SPEAKER_LIVE_TEST_NOT_AVAILABLE` and use Voice ID evidence tests.
    - Pass: no private memory leak; Voice ID evidence-only.

## 13. Anything Else Before Build

No blocker was found for Stage 8.5B.

Known prerequisites and cautions:

- Keep Stage 8.5B contract-focused.
- Do not remove adapter fallbacks in Stage 8.5B.
- Do not move semantic meaning into Desktop.
- Avoid storage/schema changes unless a later slice proves Stage 7 evidence cannot reference the PH1.X proof fields.
- Future slices must include phrase-patch scans and old shortcut classification before commit.

## 14. Validation and Tests Run

Commands required for this audit:

- `rg -n "#\\[test\\]|tokio::test|ph1x|active.*context|continuation|reference|writing.*artifact|topic.*switch|correction|ambiguity|what about|which city|the time|Japan|Melbourne|Brisbane|make it shorter|protected.*fail|fresh.*memory|MemoryContinuationDecision" crates --glob '!target/**'`
- `cargo check`
- `cargo test -p selene_engines ph1x -- --test-threads=1`
- `cargo test -p selene_adapter active_session_context -- --test-threads=1`
- `cargo test -p selene_adapter recent_archive_recall_does_not_pollute_active_context_after_answer -- --test-threads=1`
- `git diff --check`

Results:

- Test discovery command ran. It found existing PH1.X, PH1.M, adapter active-session-context, protected, fresh-memory, and Stage 8.5 helper tests. The discovery output was intentionally treated as inventory, not proof by itself.
- `cargo check`: passed.
- `cargo test -p selene_engines ph1x -- --test-threads=1`: passed, 56 tests executed, 0 failed, 0 ignored, 621 filtered out.
- `cargo test -p selene_adapter active_session_context -- --test-threads=1`: passed, 11 matching adapter tests executed, 0 failed, 0 ignored. Several adapter binary/test targets ran zero filtered tests; these are not counted as proof.
- `cargo test -p selene_adapter recent_archive_recall_does_not_pollute_active_context_after_answer -- --test-threads=1`: passed, 1 matching adapter test executed, 0 failed, 0 ignored. Several adapter binary/test targets ran zero filtered tests; these are not counted as proof.
- `git diff --check`: passed.

## 15. Final Readiness Statement

`READY_FOR_STAGE_8_5B`

Stage 8.5B should proceed as a narrow PH1.X contract/frame proof build. The active patchwork discovered here should be retained until the Stage 8.5C-G algorithmic replacements are wired and proven.
