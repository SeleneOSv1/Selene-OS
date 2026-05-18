# Stage 8.5C PH1.X Candidate Generation, Scoring, Rejection Ledger

## 1. Executive Conclusion

Stage 8.5C is complete for its intended slice: PH1.X now has a reusable candidate generation, scoring, hard-disqualifier, rejection-ledger, minimum-threshold, owner-output contract, and Stage 7 evidence-ref path.

This slice does not claim final Stage 8.5D user-visible polish. JD live testing proved the candidate foundation working across time, planning, protected, noise, and writing-artifact cases, and also exposed remaining 8.5D behavior gaps:

- specific pricing follow-ups can still lose the selected hotel/restaurant entity;
- some person/pronoun follow-ups can ask for clarification despite the previous answer naming the person;
- writing artifact rewrite behavior is now improved, but final answer shaping belongs in Stage 8.5D.

Readiness statement:

READY_FOR_STAGE_8_5D

## 2. Current Repo/App Provenance

- Repo root: `/Users/selene/Documents/Selene-OS`
- Branch: `main`
- Stage 8.5C build baseline before commit: `093d2648794ce6da2ad85c2c969461fb33b03450`
- Baseline equals `origin/main` at the start of this slice.
- Fresh Desktop build used for live testing:
  `/tmp/selene_stage8_5c_ph1x_candidates_093d264_writingfix_172221/Build/Products/Debug/SeleneMacDesktop.app`
- Current-app live proof during JD testing:
  - app PID: `64285`
  - app executable: `/private/tmp/selene_stage8_5c_ph1x_candidates_093d264_writingfix_172221/Build/Products/Debug/SeleneMacDesktop.app/Contents/MacOS/SeleneMacDesktop`
  - adapter PID: `64306`
  - adapter parent PID: `64285`
  - adapter command: `target/debug/selene_adapter_http`
  - adapter listener during proof: `127.0.0.1:18765`
  - `/healthz` repo head during live proof: `093d2648794ce6da2ad85c2c969461fb33b03450`

Note: after the extended live session, the runtime stopped and `/healthz` was no longer reachable. That is recorded as runtime session end, not as proof loss; the evidence rows and test outputs below were captured while the current app was alive.

## 3. Files Changed

- `crates/selene_kernel_contracts/src/ph1x.rs`
- `crates/selene_kernel_contracts/src/ph1f.rs`
- `crates/selene_os/src/ph1x.rs`
- `crates/selene_adapter/src/lib.rs`
- `docs/reports/STAGE_8_5C_PH1X_CANDIDATE_GENERATION_SCORING_REJECTION_LEDGER.md`

No Desktop files changed.

## 4. Algorithmic Generality Proof

Canonical owner: PH1.X.

General algorithm:

1. Normalize the current user turn into reusable Stage 8.5 features: reference posture, correction posture, artifact modification posture, planning/tool/protected/new-topic posture, entity fragment, constraints, and discourse hints.
2. Generate candidate context targets from the active frame, latest Selene answer, active tool result, active writing artifact, active plan, open clarification, correction target, topic stack, returnable topic, PH1.M fresh-memory handoff, protected fail-closed target, and no-context fallback.
3. Score every candidate with reusable factors: semantic fit, task fit, entity fit, artifact fit, tool family fit, open-slot fit, recency, speaker continuity, topic stack, discourse fit, clarification fit, correction fit, privacy scope, risk penalty, ambiguity penalty, and stale-context penalty.
4. Apply hard disqualifiers before selecting a directive: protected action without authority/simulation, rejected/noise evidence, stale context, explicit topic switch, wrong artifact type, closed topic, unsupported source evidence, and unsafe privacy/speaker scope.
5. Select the highest safe candidate above the minimum evidence threshold; otherwise ask clarification, answer as new topic, no-match safely, or fail closed.
6. Emit `HumanConversationDirective`, `owner_engine`, `allowed_next_action`, `blocked_actions`, `reason_code`, `selected_candidate`, `rejected_candidates_ref`, `candidate_rejection_ledger_ref`, `minimum_evidence_threshold_ref`, and durable evidence refs.

Why this is not a phrase patch:

- Production behavior does not branch on exact JD prompts such as "which city", "the time", a specific city, a specific person, or a story title.
- Exact prompt strings appear only in regression tests and this proof report.
- Production matching uses generalized token/vocabulary groups for reference posture, artifact posture, protected posture, planning posture, and tool/action posture. These are reusable PH1.X-owned vocabularies, not one-off example branches.
- Adapter only calls PH1.X, appends PH1.X decision refs into Stage 7 evidence, and retains old compatibility fallback paths until later slices can safely retire them.
- Desktop was not touched and did not gain semantic authority.

Positive examples:

- time continuation: New York -> Sydney;
- time after sleep/wake: New York -> sleep/wake -> Sydney;
- planning continuation: Japan/ski/restaurants -> city/base/hotel;
- writing artifact continuation: donkey story -> shorter version -> one-line version;
- protected continuation: payroll request -> yes/do-it fail-closed;
- noise rejection: rejected voice rows never become PH1.X context.

Negative examples:

- name question after time context is not routed as a time request;
- protected confirmation does not authorize execution;
- rejected noise is blocked from committed turn and memory/context update.

## 5. Candidate Generation Design

New PH1.X contract types:

- `Ph1xContextCandidateKind`
- `Ph1xCandidateRejectionReasonCode`
- `Ph1xCandidateScoreFactors`
- `Ph1xContextCandidate`
- `Ph1xCandidateRejection`
- `Ph1xCandidateRejectionLedger`
- `Ph1xOwnerOutputContract`

Runtime owner:

- `ph1x_stage8_5c_candidate_decision(...)`
- `ph1x_stage8_5c_generate_candidates(...)`
- `ph1x_stage8_5c_select_candidate_index(...)`
- `ph1x_stage8_5c_rejections(...)`
- `ph1x_stage8_5c_active_context_packet(...)`

Candidate sources implemented:

- latest Selene answer;
- active tool result;
- active writing artifact;
- active plan;
- open clarification;
- correction target;
- topic stack / returnable topic;
- fresh-memory handoff when present;
- protected fail-closed target;
- new-topic fallback.

## 6. Scoring Model

Each candidate carries `Ph1xCandidateScoreFactors`:

- `semantic_fit`
- `task_fit`
- `entity_fit`
- `artifact_fit`
- `tool_family_fit`
- `open_slot_fit`
- `recency_score`
- `speaker_continuity_score`
- `topic_stack_score`
- `discourse_fit`
- `clarification_fit`
- `correction_fit`
- `privacy_scope_fit`
- `risk_penalty`
- `ambiguity_penalty`
- `stale_context_penalty`

The score is converted into Stage 8.5C threshold behavior:

- high confidence: continue/modify/correct/route;
- medium confidence: ask clarification when needed;
- weak confidence: answer as new topic or no-match safely;
- protected risk: fail closed;
- rejected/noise evidence: blocked from context and memory;
- stale or wrong-owner evidence: rejected or clarified.

## 7. Hard Disqualifiers

Implemented disqualifier codes include:

- `HardDisqualifierProtectedRisk`
- `HardDisqualifierSpeakerPrivacyMismatch`
- `HardDisqualifierRejectedEvidence`
- `HardDisqualifierStaleContext`
- `HardDisqualifierExplicitTopicSwitch`
- `HardDisqualifierWrongArtifactType`
- `HardDisqualifierClosedTopic`
- `HardDisqualifierUnsupportedEvidence`

Protected risk always blocks protected execution unless the separate protected simulation/authority/confirmation/audit path exists. This slice does not implement protected execution.

## 8. Minimum Evidence Threshold

Threshold evidence is carried through:

- `minimum_evidence_threshold_ref`: `ph1x_threshold:stage8_5c:high_8500_medium_6500`
- candidate score;
- ambiguity level;
- confidence reason;
- why-continue / why-not-continue reason.

Threshold decisions are reflected in both `ActiveContextPacket` and the Stage 7 evidence refs.

## 9. Candidate Rejection Ledger Proof

Every Stage 8.5C decision creates or can expose:

- selected candidate;
- rejected candidates;
- rejection reason code;
- rejection reason text;
- hard disqualifier where applied;
- owner engine;
- protected risk;
- ambiguity level;
- confidence;
- confidence reason;
- why continue / why not continue;
- evidence refs.

Representative live evidence:

- event `1162`: selected `ph1x_candidate:active_writing_artifact`, rejected ledger `ph1x_candidate_ledger:970745d38c20a94a`, threshold `ph1x_threshold:stage8_5c:high_8500_medium_6500`, owner output `ph1x_owner_output:970745d38c20a94a`, directive `ph1x_directive:1779096425973000000:1779096425973000001`.
- event `1165`: selected `ph1x_candidate:active_writing_artifact`, rejected ledger `ph1x_candidate_ledger:e941f199f0a118b2`, threshold `ph1x_threshold:stage8_5c:high_8500_medium_6500`, owner output `ph1x_owner_output:e941f199f0a118b2`.
- event `1185`: selected `ph1x_candidate:new_topic_fallback`, rejected ledger `ph1x_candidate_ledger:4069901e5aa5a409`, threshold and owner refs present, PH1.E weather route present.

## 10. Owner Output Contract Proof

`Ph1xOwnerOutputContract` records:

- selected directive;
- owner engine;
- allowed next action;
- blocked actions;
- reason code;
- evidence refs;
- selected candidate;
- rejected candidates ref;
- confidence;
- ambiguity level;
- protected risk.

Adapter behavior:

- asks PH1.X for the Stage 8.5C decision;
- uses PH1.X rewrite only when PH1.X selected one;
- appends PH1.X candidate/rejection/owner refs into Stage 7 evidence;
- keeps old deterministic fallback only as retained compatibility until Stage 8.5D+ can replace it safely.

## 11. Durable Stage 7 Evidence Proof

`LiveContextEvidenceRefs` now carries:

- `selected_candidate_ref`
- `rejected_candidates_ref`
- `candidate_rejection_ledger_ref`
- `minimum_evidence_threshold_ref`
- `owner_output_contract_ref`

Stage 7 evidence rows include those refs in `decision_task_refs` and replay/integrity refs where applicable.

Rejected/noise rows remain separated:

- rejected voice rows observed around events `1171`, `1173`, `1176`, `1180`, `1188`, and `1191`;
- memory status: `BlockedRejectedTranscript`;
- no PH1.X candidate refs;
- no committed user turn;
- no memory candidate.

## 12. Regression Replay Pack Results

1. Time continuation:
   - prompt: `What time is it in New York?` -> `What about Sydney?`
   - visible result: New York time, then Sydney time.
   - JD acceptance: PASS for typed and voice.
   - backend: PH1.X candidate/directive refs filed; PH1.E time route filed; Stage 7 evidence durable.

2. Topic switch:
   - prompt: `What time is it in New York?` -> `What is your name?`
   - visible result: `I'm Selene.`
   - JD acceptance: PASS on repeat and typed path.
   - note: one voice turn phrased `And what's your name?` hit a transient runtime bridge failure and returned retry wording; the repeated direct prompt worked. This is not counted as a PH1.X semantic failure.

3. Planning candidate:
   - prompt: Japan + skiing + restaurants -> city recommendation.
   - visible result: stayed inside Japan and recommended Niseko/Hakuba/Sapporo/Nagano style options.
   - JD acceptance: PASS for city/base/hotel follow-ups.
   - backend: PH1.X planning/active-frame refs and candidate refs present.

4. Planning paraphrase:
   - prompt: `Where would you base the trip?`
   - visible result: recommended basing the trip in Niseko/Hokkaido with skiing and restaurant rationale.
   - JD acceptance: PASS.

5. Writing artifact candidate:
   - prompt: donkey story -> `give me a shorter version` -> `shorter version` -> `one line short`
   - visible result: shortened the same story and then produced a one-line version.
   - JD acceptance: PASS after the final repair.
   - earlier lighthouse/locked-factory attempts exposed the bug and are recorded as repaired.

6. Protected disqualifier:
   - prompt: `Organize payroll for Tim.` -> `Yes, do it.`
   - visible result: protected fail-closed, no execution.
   - backend: protected risk/no-execution refs filed.
   - JD acceptance: PASS for no protected execution.

7. Stage 8 fresh memory regression:
   - prompt: `What time is it in New York?` -> sleep/wake -> `What about Sydney?`
   - visible result: Sydney time after wake.
   - JD acceptance: PASS.
   - backend: PH1.M fresh-memory handoff + PH1.X continuation + PH1.E time route refs filed.

8. Noise/cough preservation:
   - safe noise/cough artifacts remained rejected.
   - visible result: no committed user message, no Selene reply, no TTS.
   - backend: rejected rows remained `BlockedRejectedTranscript`; no PH1.X candidate refs.

## 13. JD Live Test Results

JD-provided live proof included both voice and typed paths.

Passed live behavior:

- New York -> Sydney time continuation by voice and text.
- New York -> sleep/wake -> Sydney fresh-memory continuation.
- Japan planning -> city recommendation.
- Japan planning -> hotel/base continuation.
- Switzerland / Whistler planning -> hotels and location continuation.
- Writing artifact shortening after the final repair: donkey story -> shorter -> shorter again -> one-line.
- Protected payroll request did not execute.
- Noise/cough remained blocked.

Observed Stage 8.5D deferrals:

- room/restaurant pricing can still answer generically instead of binding the specific previous hotel/restaurant.
- person/pronoun follow-up about the Australian prime minister can ask which prime minister despite the prior answer naming Anthony Albanese.
- some writing-response wording still needs PH1.WRITE polish so the user never sees internal instruction-like language.

## 14. Backend Evidence Refs For Live Replays

Key evidence refs captured:

- writing artifact follow-up:
  - event `1162`, selected `ph1x_candidate:active_writing_artifact`
  - ledger `ph1x_candidate_ledger:970745d38c20a94a`
  - owner output `ph1x_owner_output:970745d38c20a94a`
  - visible response ref `ph1write_visible_response:18c2c7f5b1d2c577`
- writing artifact second follow-up:
  - event `1165`, selected `ph1x_candidate:active_writing_artifact`
  - ledger `ph1x_candidate_ledger:e941f199f0a118b2`
  - owner output `ph1x_owner_output:e941f199f0a118b2`
  - visible response ref `ph1write_visible_response:c84dcf7ec3b80168`
- weather/new-topic proof:
  - event `1185`, selected `ph1x_candidate:new_topic_fallback`
  - ledger `ph1x_candidate_ledger:4069901e5aa5a409`
  - PH1.E route refs present.
- rejected/noise proof:
  - events `1171`, `1173`, `1176`, `1180`, `1188`, `1191`
  - rejected evidence filed, no committed turn, no PH1.X context update.

## 15. Phrase-Patch Scan Output

Diff scan:

```text
git diff --unified=0 | rg -n "contains\\(|starts_with\\(|ends_with\\(|== \".*\"|which city|which areas|the time|Japan|Sydney|Melbourne|Brisbane|make it shorter|make it warmer|Mark|payroll|Tim|locked factory|Niseko|Hakuba|Nozawa|Sapporo"
```

Classifications:

- `starts_with("ph1x_owner_output:")`, `starts_with("ph1x:frame")`: CANONICAL_REASON_CODE_OK / evidence-ref prefix handling.
- reusable PH1.X token-list membership such as artifact modifiers and definite-reference targets: DOMAIN_VOCABULARY_OK.
- exact prompt strings in tests, including time/planning/writing/protected examples: TEST_FIXTURE_OK.
- exact prompt strings in this report: REPORT_OK.
- no production hit was classified as PRODUCTION_PHRASE_PATCH_NOT_ALLOWED.

Old shortcut scan:

```text
rg -n "contains\\(|starts_with\\(|ends_with\\(|== \".*\"|shortcut|fallback|deterministic_active_context|deterministic_weather_context|weather context|time context|H380|H411|H412" crates/selene_engines crates/selene_os crates/selene_adapter crates/selene_kernel_contracts --glob '!target/**'
```

Classifications:

- existing PH1.N / PH1.E / PH1.WRITE / PH1.SEARCH vocab and provider fallback logic: DOMAIN_VOCABULARY_OK or EXISTING_COMPATIBILITY_OK.
- adapter deterministic active-context fallback paths: RETAINED_COMPATIBILITY_PATH until Stage 8.5D+ proves PH1.X replacement complete.
- Stage 8.5C PH1.X reusable vocab/scoring helpers: DOMAIN_VOCABULARY_OK.
- tests/fixtures/assertions: TEST_FIXTURE_OK.

## 16. Retained Compatibility Paths

Retained by design:

- adapter deterministic active-context fallback after PH1.X candidate decision;
- older adapter evidence/report compatibility fields;
- PH1.N time/weather and PH1.E tool/domain vocabularies;
- PH1.WRITE presentation fallback logic.

Reason retained: removing active compatibility before Stage 8.5D behavior polish would risk regressions. PH1.X now owns the candidate decision path, and later slices can replace compatibility paths safely with proof.

## 17. Known Behavior Deferred To Stage 8.5D

USER_VISIBLE_BEHAVIOR_DEFERRED_TO_STAGE_8_5D

Deferred work:

- final polished planning/writing behavior;
- selected-option/entity carry-forward for specific hotels, restaurants, costs, and named people;
- stronger answer shaping so PH1.WRITE rewrites the selected artifact directly and never echoes internal instruction text;
- richer candidate payload materialization, not just refs, for UI/backend inspection;
- retirement of retained adapter compatibility paths once replacement is proven.

These are visible behavior improvements on top of the Stage 8.5C candidate/rejection/owner proof foundation.

## 18. Tests Run

- `cargo fmt` PASS
- `cargo check` PASS
- `cargo test -p selene_kernel_contracts ph1x -- --test-threads=1` PASS, 42 tests
- `cargo test -p selene_engines ph1x -- --test-threads=1` PASS, 56 tests
- `cargo test -p selene_adapter active_session_context -- --test-threads=1` PASS, 11 tests
- `cargo test -p selene_adapter recent_archive_recall_does_not_pollute_active_context_after_answer -- --test-threads=1` PASS, 1 test
- `cargo test -p selene_adapter stage8_5c_adapter_persists_public_answer_writing_frame -- --test-threads=1` PASS, 1 test
- `cargo test -p selene_os stage8_5c -- --test-threads=1` PASS, 12 tests
- `cargo test -p selene_adapter -- --test-threads=1` PASS, 470 passed, 4 ignored
- `cargo test -p selene_os -- --test-threads=1` PASS, 1713 passed
- `cargo test -p selene_engines -- --test-threads=1` PASS, 665 passed, 12 ignored
- `git diff --check` PASS

Commands that ran zero filtered tests during targeted discovery were not counted as proof.

## 19. Readiness Statement

READY_FOR_STAGE_8_5D

Stage 8.5D should now focus on user-visible planning/writing/tool-follow-up polish using the Stage 8.5C PH1.X candidate and rejection foundation, especially:

- selected entity carry-forward;
- specific hotel/restaurant/person reference binding;
- final PH1.WRITE rewrite behavior;
- removal/migration of retained adapter compatibility paths when PH1.X replacement is proven.
