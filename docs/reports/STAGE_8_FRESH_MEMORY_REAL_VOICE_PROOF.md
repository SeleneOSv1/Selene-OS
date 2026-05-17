# Stage 8 Fresh Memory Real Voice Proof

## 1. Executive conclusion

READY_FOR_STAGE_9_FRESH_MEMORY_TRACE_EVAL

Stage 8 is implemented and proven:

- Fresh memory continues a just-now time question after Selene sleeps and wakes again.
- The continuation works from durable Stage 7 evidence, including after adapter restart.
- Active context before sleep still works through PH1.X without needing PH1.M fresh memory.
- Normal unrelated questions are not stolen by old time context.
- Vague place fragments do not blindly force a time answer.
- Noise/cough does not become a committed turn or memory update.
- Protected payroll workflow requests fail closed.
- Voice ID/speaker fields remain evidence-only and nullable.
- Desktop was not changed and does not own memory.

## 2. Current repo/app provenance

- Repo root: `/Users/selene/Documents/Selene-OS`
- Baseline prerequisite: Stage 7.1 durable evidence replay commit `6610e0e3eca592627507fe92cf7aea95a229a8e6`
- Start branch: `main`
- Start tree: clean before Stage 8 edits
- Fresh Xcode build result: `BUILD SUCCEEDED`
- Fresh Xcode build path: `/tmp/selene_stage8_fresh_memory_6610e0e_final/Build/Products/Debug/SeleneMacDesktop.app`
- Live bundle path used for controlled endpoint proof: `/tmp/selene_stage8_fresh_memory_6610e0e_latest/Build/Products/Debug/SeleneMacDesktop.app`
- Live app PID during controlled proof: `62001`
- Adapter pre-restart PID during controlled proof: `67935`
- Adapter post-restart PID during controlled proof: `68179`
- Health endpoint repo head during controlled proof: `6610e0e3eca592627507fe92cf7aea95a229a8e6`
- Health endpoint adapter owner: `SeleneMacDesktopRuntimeBridge`

The app binary was built from the Stage 8 working tree before the final report commit. The report file does not affect the Desktop binary.

## 3. Files changed

- `crates/selene_engines/src/ph1m.rs`
- `crates/selene_adapter/src/lib.rs`
- `crates/selene_kernel_contracts/src/ph1f.rs`
- `crates/selene_storage/src/ph1f.rs`
- `docs/reports/STAGE_8_FRESH_MEMORY_REAL_VOICE_PROOF.md`

No Desktop files were changed.

## 4. Root causes fixed

### Fresh memory after sleep/wake

Root cause:
PH1.M had canonical contracts from Stage 6.6, and Stage 7.1 had durable evidence, but there was no runtime PH1.M fresh-memory continuation resolver wired into the post-sleep turn path.

Owner fix:
PH1.M now owns the fresh-memory continuation decision. The adapter only builds the request from durable evidence and transports the PH1.M result.

Implemented:

- `FreshMemoryPriorTurnEvidence`
- `FreshMemoryContinuationRequest`
- `FreshMemoryContinuationResolution`
- `Ph1mRuntime::fresh_memory_continuation`
- PH1.M decisions for continue, clarify, answer normally, and no-match

### Adapter restart fresh-memory failure

Root cause:
Stage 7.1 replayed internal-history evidence after adapter restart, but the fresh-memory bridge still depended on the in-process `conversation_ledger()` projection for prior user text/tool context. After restart, evidence rows were durable, but the conversation turn projection was not restored.

Owner fix:
Storage/PH1.F and the adapter persistence bridge now persist and restore `ConversationTurnRecord` rows alongside internal-history evidence records.

Implemented:

- `ConversationTurnRecord` serde compatibility
- `restore_conversation_turn_record`
- `replace_conversation_turn_records_from_replay`
- adapter persistence of `conversation_turn_records`
- replay/merge validation for durable conversation turn projection

### Protected payroll lane

Root cause:
Payroll action verbs such as prepare/organize could pass through identity or normal-chat lanes instead of the protected business workflow lane.

Owner fix:
Adapter routing/protected-lane classification now separates public payroll knowledge, private payroll reads, and protected payroll execution.

Implemented:

- Public knowledge: `Tell me about payroll`, `How do I prepare payroll?`
- Private read: `Check/show/view/report/review payroll`
- Protected execution: `Prepare/organize/run/process/approve/submit/finalize/pay/adjust/calculate/correct/change/update/create/draft payroll`

## 5. JD live test script actually run

JD ran the live Desktop app and reported these real-life results:

- Voice fresh memory: New York time before sleep, wake after sleep, Sydney follow-up. Result: worked and remembered correctly.
- Typed fresh memory: New York time before sleep, Sydney follow-up after sleep. Result: worked and remembered correctly.
- Normal identity question: Selene answered, "I'm Selene."
- Vague/non-continuation Sydney case: Selene gave a normal Sydney information answer, not a stale time answer.
- Payroll protected request after fix: `Organize payroll for Tim.` failed closed.
- Public payroll knowledge after fix: `Tell me about payroll.` produced a normal public explanation.
- Cough/noise after wake: no visible user message, no answer, no TTS.

JD also reported one repeated/missed utterance during live testing caused by listener re-arm timing. That was not treated as a fresh-memory failure because the accepted New York/Sydney voice and typed scenarios both worked correctly afterward.

## 6. Visible/audible result for each live test

| Live test | JD result | Pass/fail |
| --- | --- | --- |
| Active/fresh time continuation voice | Remembered and acted correctly | PASS |
| Fresh time continuation typed | Remembered and acted correctly | PASS |
| Normal name question | `I'm Selene.` | PASS |
| Vague Sydney/non-continuation | Normal Sydney answer, no time hijack | PASS |
| Protected payroll action | Fail-closed protected response | PASS |
| Public payroll knowledge | Public explanation | PASS |
| Cough/noise after wake | No transcript bubble, no answer | PASS |

## 7. Exact response_text and tts_text for memory turns

Controlled durable proof produced:

- Prior turn user text: `What time is it in New York?`
- Prior Selene response: `It's 6:57 AM in New York.`
- Post-sleep user text: `What about Sydney?`
- Fresh-memory response_text: `It's 8:59 PM in Sydney.`
- Fresh-memory tts_text: `It's 8:59 PM in Sydney.`

The fresh-memory response did not include any forbidden mechanical wording:

- no `session`
- no `archive`
- no `retrieval`
- no `search result`
- no `memory packet`
- no `evidence row`
- no `I found a record`

## 8. Backend evidence proof for every live/control test

The strongest backend proof used the real adapter endpoint and Stage 7 durable evidence store:

- Store path: `/tmp/selene_stage8_endpoint_replay_1779015326.jsonl`
- Endpoint inspected: `/v1/ui/internal-history/evidence?limit=200`
- Adapter restart was performed between the New York/sleep evidence and the Sydney follow-up.

### New York prior turn

- Correlation/turn: `800000100001`
- Session: `1`
- Modality: voice
- User text: `What time is it in New York?`
- Response: `It's 6:57 AM in New York.`
- Evidence categories present: committed turn, Selene output, PH1.E time/tool evidence, TTS/requested spoken evidence, PH1.X slots/refs nullable as designed.

### Sleep boundary

- Correlation/turn: `800000100002`
- Response: `SESSION_IDLE_CLOSED`
- Reason: `ph1l_closed_after_30s_no_valid_engagement`
- Evidence event: `LifecycleBoundary`
- Fresh memory handoff ref: `ph1m_fresh_handoff_boundary:sleep:800000100002:800000100002`

### Sydney post-restart continuation

- Correlation/turn: `800000100003`
- User text: `What about Sydney?`
- Response/tts_text: `It's 8:59 PM in Sydney.`
- Evidence after Sydney: 10 internal-history events total
- Event 6: `CommittedTurn`, conversation turn id `3`, user voice
- Event 8: `CommittedTurn`, conversation turn id `4`, Selene output, TTS `Requested`
- Event 9: `ToolEvidence`, time route
- Event 10: `MemoryEvidence`

PH1.X refs:

- `ph1x_active_context:fresh_memory:800000100003:800000100003`
- `ph1x_directive:fresh_memory:800000100003:800000100003`
- `ph1x_fresh_memory_continuity:time:800000100003:800000100003`

PH1.M refs:

- `ph1m_memory_evidence:fresh:800000100003:800000100003`
- `ph1m_recall_request:fresh:800000100003:800000100003`
- `ph1m_fresh_handoff:sleep:800000100003:800000100003`
- `ph1m_continuation_decision:continue:800000100003:800000100003`

PH1.E refs:

- `ph1e_tool:time:request:7799185125768307100:query:13515813963476043464:status:Ok:cache:Bypassed`
- `ph1e_provider:google_time_zone`

## 9. New York -> sleep -> wake -> Sydney proof

PASS.

Proof chain:

1. New York voice time turn was filed durably.
2. Selene answered and TTS evidence was filed.
3. PH1.L idle close filed a sleep/lifecycle boundary after the approved idle window.
4. Adapter was restarted.
5. Durable evidence endpoint still returned the New York turn and sleep boundary.
6. Post-restart Sydney follow-up was resolved by PH1.M fresh memory.
7. PH1.X created fresh active context and continuation directive refs.
8. PH1.E routed the resolved time query.
9. Selene answered Sydney time naturally.
10. PH1.M memory evidence was filed for the continuation.

## 10. Adapter restart durable fresh memory proof

PASS.

Before restart:

- Adapter PID: `67935`
- Evidence endpoint returned 5 events after New York and sleep boundary.

Restart:

- Adapter PID changed from `67935` to `68179`
- Same store path was used.
- Health endpoint was reachable after restart.

After restart:

- Evidence endpoint still returned the pre-restart New York and sleep boundary evidence.
- Sydney follow-up worked.
- Fresh memory used durable evidence, not old adapter memory.

The Stage 8 durability bug was fixed by restoring conversation turn records from persisted storage. No Desktop workaround was added.

## 11. Speaker evidence proof

PASS.

Stage 8 does not make Voice ID an authority source. Voice speaker evidence remains nullable/evidence-only, and typed turns do not fabricate Voice ID evidence.

Verified behavior:

- Voice turns carry speaker evidence slots or nullable posture as designed.
- Typed turns carry typed actor identity separately from voice identity.
- Payroll protected execution was not authorized by Voice ID or fresh memory.
- Speaker evidence is available for later PH1.M/PH1.X scoping without becoming authority.

Second-speaker live testing was not available in this run.

## 12. Vague/ambiguous fragment proof

PASS.

Automated PH1.M proof:

- Prior context: New York time.
- New utterance: `Sydney`
- Decision: clarify/no blind continuation.
- Clarification text: `Do you mean the time question, or something else about Sydney?`

Live user observation:

- A Sydney-only/non-continuation case produced a normal Sydney information answer rather than a stale time answer.

Either outcome is acceptable for Stage 8 because the forbidden failure is blindly routing `Sydney` as stale time context with low confidence.

## 13. Normal question not stolen proof

PASS.

Automated PH1.M proof:

- Prior context: New York time.
- New utterance: `What is your name?`
- Decision: answer normally, not continue time.

Live user observation:

- Selene answered: `I'm Selene.`

## 14. Noise/protected fail-closed proof

PASS.

Noise/cough:

- JD woke Selene and coughed.
- JD observed no visible user message and no Selene reply.
- The system remained ready/slept normally afterward.
- The cough did not update fresh memory.

Protected payroll:

- Request: `Organize payroll for Tim.`
- Response: `I can't perform or prepare that protected business action from this turn. NO_SIMULATION_NO_AUTHORITY_NO_PROTECTED_EXECUTION.`
- No business action executed.
- Public knowledge remained separate:
  - Request: `Tell me about payroll.`
  - Response: normal public payroll explanation.

## 15. Tests run

Targeted tests:

- `cargo test -p selene_adapter h412_payroll_lane_detection_separates_public_read_and_protected_execution -- --test-threads=1`
- `cargo test -p selene_adapter stage8_fresh_memory -- --test-threads=1`
- `cargo test -p selene_storage at_f_stage8_conversation_turn_records_restore_after_store_reload -- --test-threads=1`

Full required validation:

- `cargo check`
- `cargo test -p selene_kernel_contracts -- --test-threads=1`
- `cargo test -p selene_storage -- --test-threads=1`
- `cargo test -p selene_adapter -- --test-threads=1`
- `cargo test -p selene_os -- --test-threads=1`
- `cargo test -p selene_engines -- --test-threads=1`
- `git diff --check`

Results:

- `cargo check`: PASS
- `selene_kernel_contracts`: 395 tests passed
- `selene_storage`: PASS
- `selene_adapter`: 471 tests passed, 4 ignored live-secret tests
- `selene_os`: 1682 tests passed
- `selene_engines`: 665 tests passed, 12 ignored
- `git diff --check`: PASS
- Xcode fresh build: PASS

## 16. Carried-forward Stage 1-7 proof

PASS.

Carried-forward behaviors verified by live report, targeted tests, and full package suites:

- Wake works.
- Voice TTS works.
- Typed input works.
- 30-second idle sleep still works.
- Fresh memory uses sleep boundary and durable evidence.
- No OpenAI STT before lawful wake was changed.
- Cough/noise/self-echo remain blocked from committed turns and memory.
- Protected fail-closed still works.
- PH1.E time route evidence still files.
- Stage 7 internal-history evidence remains durable/replayable.
- Desktop remains render/capture/playback only.

## 17. Readiness

READY_FOR_STAGE_9_FRESH_MEMORY_TRACE_EVAL

Stage 8 now has the required product behavior and evidence proof:

- Live voice and typed fresh memory worked for JD.
- Controlled backend proof showed New York -> sleep -> adapter restart -> wake -> Sydney works using durable evidence.
- PH1.M owns fresh remembered context.
- PH1.X owns live continuation context.
- PH1.L owns the sleep boundary.
- PH1.E owns the time answer.
- Storage/PH1.F owns durable evidence.
- Adapter transports and bridges.
- Desktop does not own memory.
