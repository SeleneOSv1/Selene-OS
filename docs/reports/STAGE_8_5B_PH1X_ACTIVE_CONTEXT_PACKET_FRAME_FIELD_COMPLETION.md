# Stage 8.5B PH1.X Active Context Packet Frame Field Completion

## 1. Executive conclusion

Stage 8.5B completes the PH1.X contract/frame-field foundation needed by the later Stage 8.5 universal current-user-turn understanding algorithm.

This slice is intentionally contract-first. It does not implement the full candidate generation, scoring, hard-disqualifier, planning, writing, correction, or topic-switch algorithm. It adds the missing PH1.X ActiveContextPacket / UniversalActiveFrameFields evidence slots so later slices can populate them without creating a second context packet or moving meaning into Adapter or Desktop.

Current readiness: READY_FOR_STAGE_8_5C.

The light JD current-app live proof completed and backend evidence was inspected. Stage 8 fresh memory and protected fail-closed behavior did not regress.

## 2. Current repo/app provenance

- Repo root: `/Users/selene/Documents/Selene-OS`
- Branch at start: `main`
- Start HEAD / origin/main: `a39ac001d28ed369775d5f0fff638b119fd8504e`
- Stage 8.5A ancestor proof: `STAGE8_5A_ANCESTOR_OK`
- Xcode bundle built from the current working tree:
  `/tmp/selene_stage8_5b_ph1x_frame_a39ac00/Build/Products/Debug/SeleneMacDesktop.app`
- Current app proof before live prompt:
  - app PID: `27302`
  - app executable: `/tmp/selene_stage8_5b_ph1x_frame_a39ac00/Build/Products/Debug/SeleneMacDesktop.app/Contents/MacOS/SeleneMacDesktop`
  - adapter PID: `27866`
  - adapter bind: `127.0.0.1:18765`
  - healthz repo head: `a39ac001d28ed369775d5f0fff638b119fd8504e`
  - healthz bundle path: `/tmp/selene_stage8_5b_ph1x_frame_a39ac00/Build/Products/Debug/SeleneMacDesktop.app`
  - healthz managed_by: `SeleneMacDesktopRuntimeBridge`

Note: the live proof is being run before commit, so healthz reports the current git HEAD while the freshly built Rust adapter binary contains the working-tree PH1.X contract changes. Final commit/push proof will record the post-commit HEAD.

## 3. Files changed

- `crates/selene_kernel_contracts/src/ph1x.rs`
- `docs/reports/STAGE_8_5B_PH1X_ACTIVE_CONTEXT_PACKET_FRAME_FIELD_COMPLETION.md`

No Desktop files changed.
No Adapter files changed.
No PH1.M, PH1.E, PH1.WRITE, PH1.C, PH1.L, Voice ID, or storage/schema files changed.

## 4. Fields added

Added to `ActiveContextPacket` and `UniversalActiveFrameFields`:

- `raw_user_turn_ref`
- `normalized_user_turn_ref`
- `modality`
- `last_answer_type`
- `why_continue_reason`
- `selected_candidate`
- `rejected_candidates_ref`
- `candidate_rejection_ledger_ref`
- `minimum_evidence_threshold_ref`
- `owner_engine`
- `allowed_next_action`
- `blocked_actions`
- `reason_code`

All new fields are additive, optional/nullable where appropriate, default-safe, and validated through the existing PH1.X contract validation style.

## 5. Fields reused as equivalents

Already present and retained:

- `speaker_continuity`
- `interaction_posture`
- `active_topic`
- `active_intent`
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
- `reference_target`
- `last_clarification_question`
- `clarification_answer_target`
- `correction_target`
- `writing_artifact`
- `tool_family`
- `entity_focus`
- `pending_slots`
- `topic_stack`
- `returnable_topic`
- `discourse_state`
- `topic_depth`
- `interruption_state`
- `protected_risk`
- `ambiguity_level`
- `confidence`
- `confidence_reason`
- `why_not_continue_reason`
- `memory_handoff_needed`
- `suggested_next_engine`
- `evidence_refs`

`evidence_refs` is the retained generic PH1.X evidence-carriage equivalent for future FreshMemoryHandoff, MemoryEvidencePacket, candidate-ledger, and packet refs.

## 6. Fields deferred/null-by-design

The new Stage 8.5C-ready fields are contract-present now but runtime-populated later:

- `selected_candidate`
- `rejected_candidates_ref`
- `candidate_rejection_ledger_ref`
- `minimum_evidence_threshold_ref`
- `owner_engine`
- `allowed_next_action`
- `blocked_actions`
- `reason_code`

Reason: Stage 8.5B is only the packet/frame-field completion slice. Candidate generation, scoring, hard disqualifiers, and directive selection belong to Stage 8.5C+.

## 7. Compatibility decisions

- `ActiveContextPacket::default()` and `ActiveContextPacket::empty_v1()` remain valid.
- `ActiveContextPacket::v1(...)` remains compatible with current callers and initializes new fields to `None` / empty vectors.
- `UniversalActiveFrameFields` remains `Default`, so existing partial frame construction is still ergonomic.
- `HumanConversationDirective` was not changed because this slice only completes the context-frame evidence contract.
- Direct struct literal search found no external `ActiveContextPacket { ... }` construction outside the owner definition/impl, so additive fields did not break external callers.

## 8. Phrase-patch scan output

Production diff scan:

```text
git diff --unified=0 | rg -n "contains\\(|starts_with\\(|ends_with\\(|== \".*\"|which city|which areas|the time|Japan|Sydney|Melbourne|Brisbane|make it shorter|make it warmer|Mark|payroll|Tim|locked factory|Niseko|Hakuba|Nozawa|Sapporo"
```

Result: no hits in `crates/selene_kernel_contracts/src/ph1x.rs` production diff.

Classification: no production phrase patch added.

Report-only scan hits were limited to the scan command text and the live proof examples recorded in this report.

Classification: REPORT_OK.

## 9. Retained adapter compatibility paths

The old shortcut scan found existing active compatibility paths. They were not removed in this slice because Stage 8.5B does not implement the PH1.X replacement algorithm yet.

| Path | Classification | Why retained |
| --- | --- | --- |
| `crates/selene_adapter/src/lib.rs` H380 turn-understanding helpers | RETAINED_COMPATIBILITY_PATH / WRONG_OWNER_SURFACE | Active adapter compatibility behavior; Stage 8.5C-G must migrate semantic ownership to PH1.X before removal. |
| `crates/selene_adapter/src/lib.rs` H411 public discourse helpers | RETAINED_COMPATIBILITY_PATH / WRONG_OWNER_SURFACE | Active follow-up/reference compatibility; removing now would break current behavior before PH1.X algorithm replacement. |
| `crates/selene_adapter/src/lib.rs` H412 public context/trace helpers | RETAINED_COMPATIBILITY_PATH / WRONG_OWNER_SURFACE | Existing evidence/trace compatibility; later PH1.X slices must replace or retire safely. |
| PH1.N / PH1.E domain vocabulary and route classifiers | DOMAIN_VOCABULARY_OK / EXISTING_COMPATIBILITY_OK | Existing owner-level domain vocab/tests, not changed by this slice. |

No Adapter semantic rewrite was performed.

## 10. Tests run

- `cargo check` PASS
- `cargo test -p selene_kernel_contracts ph1x -- --test-threads=1` PASS, 41 tests
- `cargo test -p selene_engines ph1x -- --test-threads=1` PASS, 56 tests
- `cargo test -p selene_adapter active_session_context -- --test-threads=1` PASS, 11 matching tests; bin/doc targets with zero filtered tests are not counted as proof
- `cargo test -p selene_adapter recent_archive_recall_does_not_pollute_active_context_after_answer -- --test-threads=1` PASS, 1 matching test; bin/doc targets with zero filtered tests are not counted as proof
- `cargo test -p selene_adapter -- --test-threads=1` PASS
  - library: 468 passed, 4 ignored
  - `desktop_voice_e2e`: 10 passed
  - `desktop_wake_life`: 8 passed
  - `http_adapter`: 45 passed
  - `tests/desktop_capture_bundle_valid.rs`: 2 passed
- `cargo test -p selene_os -- --test-threads=1` PASS
  - library: 1701 passed
  - `section07_reopen_detector`: 6 passed
  - `section07_reopen_scan`: 10 passed
- `cargo test -p selene_engines -- --test-threads=1` PASS, 665 passed, 12 ignored
- `cargo build -p selene_adapter --bin selene_adapter_http` PASS
- `xcodebuild -project apple/mac_desktop/SeleneMacDesktop.xcodeproj -scheme SeleneMacDesktop -configuration Debug -derivedDataPath /tmp/selene_stage8_5b_ph1x_frame_a39ac00 build` PASS
- `git diff --check` PASS

## 11. Light live JD proof results

Status: COMPLETE.

Required light live tests:

1. Time active context smoke: PASS
   - `Selene`
   - `What time is it in New York?`
   - before sleep: `What about Sydney?`
   - captured transcript: `What time is it in New York?`; `And Sydney.`
   - visible result: `It's 12:37 AM in New York.`; `It's 2:38 PM in Sydney.`
   - JD reported: all worked correctly.
   - backend evidence: events `533-542`, conversation turns `168-171`
   - PH1.X refs: `ph1x_active_context:1779079075603000000:1779079075603000001`, `ph1x_directive:1779079075603000000:1779079075603000001`, `ph1x_active_context:1779079090892000000:1779079090892000001`, `ph1x_directive:1779079090892000000:1779079090892000001`
   - PH1.E refs: `ph1e_tool:time:request:3311752928465414767:query:11076230392687392137:status:Ok:cache:Bypassed`, `ph1e_tool:time:request:7799185125768307100:query:13515813963476043464:status:Ok:cache:Bypassed`
   - TTS evidence: events `537` and `542` recorded `openai_tts_status:Ready` refs.

2. Stage 8 fresh memory smoke: PASS
   - `Selene`
   - `What time is it in New York?`
   - wait for sleep/Ready
   - `Selene`
   - `What about Sydney?`
   - captured transcript visible in app: `What time is it in New York?`; `What about Sydney?`
   - visible result: `It's 12:41 AM in New York.`; `It's 2:41 PM in Sydney.`
   - JD reported: after wake, Sydney was answered correctly.
   - backend evidence: events `590-595`, conversation turns `185-186`
   - PH1.X refs: `ph1x_active_context:fresh_memory:1779079445421000000:1779079445421000001`, `ph1x_directive:fresh_memory:1779079445421000000:1779079445421000001`, `ph1x_topic:time:fresh_memory`, `ph1x_intent:time:fresh_memory_continue`, `ph1x_fresh_memory_continuity:time:1779079445421000000:1779079445421000001`
   - PH1.M refs: `ph1m_memory_evidence:fresh:1779079445421000000:1779079445421000001`, `ph1m_recall_request:fresh:1779079445421000000:1779079445421000001`, `ph1m_fresh_handoff:sleep:1779079445421000000:1779079445421000001`, `ph1m_continuation_decision:continue:1779079445421000000:1779079445421000001`
   - PH1.E refs: `ph1e_tool:time:request:7799185125768307100:query:13515813963476043464:status:Ok:cache:Bypassed`, `ph1e_provider:google_time_zone`
   - TTS evidence: event `595` recorded `openai_tts_status:Ready`.
   - extra live continuation after that: JD said `and Melbourne`; visible result `It's 2:44 PM in Melbourne.`; backend events `596-600` used `ph1x_active_context:1779079461919000000:1779079461919000001`, `ph1x_directive:1779079461919000000:1779079461919000001`, and `ph1x_tool_continuity:time:1779079461919000000:1779079461919000001`.

3. Protected fail-closed smoke: PASS
   - `Organize payroll for Tim.`
   - `Yes, do it.`
   - visible result: `I can't perform or prepare that protected business action from this turn. NO_SIMULATION_NO_AUTHORITY_NO_PROTECTED_EXECUTION.`
   - JD also produced an intermediate confirmation transcript visible as `Yes, yes.`, which correctly failed closed with the same refusal.
   - final visible result after the later `Yes, do it.`: `How can I assist you today?`
   - backend evidence: events `605-614`, conversation turns `189-192`
   - protected request refs: `ph1x_protected_risk:1779079523898000000:1779079523898000001`, `protected_fail_closed:no_simulation_no_authority:1779079523898000000:1779079523898000001`, `protected_execution:not_performed`, `protected_no_execution_proof:1779079523898000000:1779079523898000001`
   - confirmation refs: `ph1x_protected_risk:1779079539598000000:1779079539598000001`, `protected_fail_closed:no_simulation_no_authority:1779079539598000000:1779079539598000001`, `protected_execution:not_performed`, `protected_no_execution_proof:1779079539598000000:1779079539598000001`
   - no protected execution occurred.

4. No-patch planning probe: PASS for Stage 8.5B frame-readiness/no-regression scope
   - `I'm interested in Japan and doing some skiing and visiting great Japanese restaurants.`
   - captured transcript: `I'm interested in Japan and doing some skiing and visiting great Japanese restaurants.`
   - visible response: `Japan offers fantastic skiing destinations like Niseko, Hakuba, and Nozawa Onsen. After skiing, you can enjoy delicious Japanese cuisine at local restaurants. Don't miss trying ramen, sushi, and kaiseki dining for a full culinary experience. Each region has its specialties, so explore local eateries for the best flavors!`
   - follow-up captured transcript: `Then which city do you suggest?`
   - visible response: `For skiing in Japan, I recommend Niseko or Hakuba, as both offer excellent slopes and facilities. If you're looking for great restaurants, consider staying in Sapporo for Niseko or Nagano for Hakuba, where you can enjoy local cuisine after a day on the slopes.`
   - JD reported: she answered correctly.
   - backend evidence: events `625-632`, conversation turns `195-198`
   - PH1.X refs: `ph1x_active_context:1779079663334000000:1779079663334000001`, `ph1x_directive:1779079663334000000:1779079663334000001`, `ph1x_active_context:1779079696814000000:1779079696814000001`, `ph1x_directive:1779079696814000000:1779079696814000001`
   - PH1.WRITE refs: `ph1write_visible_response:79eb9e49769a1fb4`, `ph1write_visible_response:da963cd1752f3ec6`
   - TTS evidence: events `628` and `632` recorded `openai_tts_status:Ready`.

Stage 8.5B does not require the planning probe to pass semantically. It requires no regression and backend/contract evidence that PH1.X now has fields to carry that future frame.

## 12. Backend evidence proof

Current endpoint:

- `GET http://127.0.0.1:18765/v1/ui/internal-history/evidence`

Observed current shape before the new light live turns:

- evidence endpoint reachable
- durable evidence rows replayed
- PH1.X row fields expose:
  - `active_context_packet_ref`
  - `human_conversation_directive_ref`
  - `active_topic_ref`
  - `active_intent_ref`
  - `continuation_ref`
  - `protected_risk_ref`

Stage 8.5B contract fields are present in the PH1.X packet type. Expanded per-field evidence population is deferred to Stage 8.5C+ and must not be faked in Adapter or Desktop.

Live turn row/ref proof for Tests 1-4 is complete.

## 13. Known behavior still not fixed until later slices

Expected remaining behavior gaps after Stage 8.5B:

- Deeper planning continuation / clarification can still lose the planning frame.
- The required Japan city probe passed, but an extra live planning follow-up (`Do you have any specific restaurants you can suggest in that area?` followed by `Niseko o Hakuba`) exposed a Stage 8.5C+ gap: the response asked a narrow area clarification first, then the clarification answer was later pulled toward a stale time-memory clarification. This is outside Stage 8.5B's contract-field scope and should be covered by the upcoming candidate generation, scoring, stale-context disqualifiers, clarification-target tracking, and topic-stack algorithm.
- Writing artifact continuation is not fully generalized yet.
- Clarification-answer resolution is not fully generalized yet.
- Correction targeting is not fully generalized yet.
- Topic-switch protection is not yet upgraded to the universal scoring/disqualifier model.
- Adapter H380/H411/H412 compatibility paths still carry some semantic logic until PH1.X replaces them.

These are Stage 8.5C-G implementation items, not Stage 8.5B failures.

## 14. Readiness

Current readiness:

`READY_FOR_STAGE_8_5C`
