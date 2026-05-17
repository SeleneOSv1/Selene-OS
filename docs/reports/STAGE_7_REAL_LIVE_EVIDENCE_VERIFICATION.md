# Stage 7 Real Live Evidence Verification

## 1. Executive Conclusion

Stage 7 live filing was verified through the current Desktop app and the adapter evidence endpoint. Voice, typed, time/tool, TTS, protected fail-closed, PH1.X evidence refs, PH1.M nullable evidence slots, and speaker/typed identity separation all landed in the Stage 7 internal-history evidence shape.

Root-cause repairs were required and were made in the adapter evidence bridge:

- exposed the real internal-history evidence report at `/v1/ui/internal-history/evidence`
- filed Desktop OpenAI TTS ready/fail-closed evidence from the TTS bridge
- filed richer runtime response, PH1.E/tool, protected fail-closed, PH1.X, PH1.M, speaker, typed actor, and TTS evidence refs
- fixed duplicate committed voice-user evidence by deduping the adapter final transcript bridge against the PH1.C accepted transcript commit

The live app behavior was good. JD confirmed the answers were correct, the final cough/noise did not create a visible user message, and Selene went to sleep after the idle close.

Readiness:

NOT_READY_FOR_STAGE_8_FRESH_MEMORY

Exact blocker: Stage 7 evidence is append-only and well-shaped in the live adapter process, and the DB migration exists, but the current runtime evidence endpoint is still backed by the in-memory PH1.F store. After the adapter restarted during proof, prior live evidence was no longer visible through the endpoint. Stage 8 fresh memory should not depend on this until the internal-history evidence ledger is wired to durable/recoverable storage or the Stage 8 proof explicitly stays within one adapter lifetime.

## 2. Current Repo And App Provenance

- Repo root: `/Users/selene/Documents/Selene-OS`
- Branch: `main`
- Baseline before verification: `ae966bff0362a9d8944a4a58c736479071035369`
- Fresh Desktop build bundle used: `/tmp/selene_stage7_live_evidence_ae966bf/Build/Products/Debug/SeleneMacDesktop.app`
- Live app process observed: `SeleneMacDesktop` PID `4758`
- Live adapter port: `127.0.0.1:18765`
- Adapter process observed after restart: PID `6715`
- Evidence inspection route added and used: `GET /v1/ui/internal-history/evidence`

Storage repo truth discovered:

- Contract and in-memory owner: `crates/selene_storage/src/ph1f.rs`
- Repo trait surface: `crates/selene_storage/src/repo.rs`
- DB migration present: `crates/selene_storage/migrations/0026_stage7_internal_history_evidence.sql`
- Migration note says runtime currently uses the in-memory PH1.F store and keeps DB wiring aligned.

## 3. JD Live Script Actually Run

JD ran real Desktop voice and typed prompts while Codex inspected transcript/evidence snapshots:

- Voice wake and time: `Selene` then `What's the time in Sydney?`
- Voice follow-up: `What about in Melbourne?`
- Voice identity/name: `What is your name?`
- Typed time: `what time is in shanghai`
- Typed follow-up: `and what about suzhou`
- Typed protected: `approve payroll for tim`
- Voice protected: `Approve payroll for Tim.`
- Voice normal questions after wake/re-arm
- JD-reported cough/noise: did not create a visible user message

Supplemental controlled reject proof used the existing safe voice-turn path with transcript text `cough` to produce a deterministic rejected evidence row without relying on a flaky microphone artifact.

## 4. Visible And Audible Results

- Voice normal turns displayed answers and played OpenAI/runtime TTS.
- Typed normal turns displayed answers and did not fabricate voice evidence.
- Protected typed request failed closed and did not execute.
- Protected voice request did not execute; it stayed behind identity/authority handling.
- Cough/noise did not produce a visible committed user message or Selene answer.
- After the last run, Selene entered sleep/idle-close state as expected.

JD clarified one repeated prompt was caused by re-arm timing and should not be counted as a Stage 7 ledger failure.

## 5. Evidence Row Proof

Primary live evidence snapshot:

- `/tmp/selene_stage7_evidence_18765_after_cough.json`
- total events: `71`
- committed turns: `48`
- lifecycle boundaries: `17`
- protected fail-closed events: `1`
- rejected input events: `1`
- tool evidence events: `4`
- voice modality rows: `14`
- typed modality rows: `4`
- system modality rows: `53`

Important extracted proof files:

- `/tmp/selene_stage7_key_evidence_rows.txt`
- `/tmp/selene_stage7_ref_markers.txt`
- `/tmp/selene_stage7_user_turn_index.txt`
- `/tmp/selene_stage7_controlled_noise_evidence.txt`

### Voice Normal Turn

Evidence row examples:

- event `2`: `CommittedTurn`, role `User`, source `VoiceTranscript`, modality `Voice`, correlation `1778987065295000000`, turn `1778987065295000001`
- event `4`: Selene response evidence for the same correlation/turn, response hash `f11652c6620b5458`
- event `5`: `ToolEvidence`, PH1.E time route, provider `google_time_zone`
- event `6`: lifecycle TTS evidence, `tts_provider:"openai_first_runtime"`, `tts_status:"Ready"`, `has_audio_generation_ref:true`

Voice evidence landed as voice, not typed. Speaker evidence was nullable/evidence-only: `identity_posture:"Unknown"`, no authority grant.

### Typed Normal Turn

Typed rows in the live evidence report show:

- source `TypedText`
- modality `Typed`
- `typed_actor_identity_ref:"typed_actor:tenant_a:user_ingress_test"`
- `has_voice_identity_evidence:false`
- `has_typed_actor_identity:true`

This proves typed turns did not fabricate Voice ID evidence.

### Voice ID / Speaker Evidence

Voice rows carried speaker slots with unknown/nullable posture when the speaker was not recognized:

- `speaker_id:null`
- `voice_profile_id:null`
- `identity_posture:"Unknown"`
- `voice_identity_assertion_ref:null`
- `has_voice_identity_evidence:false`

Voice ID remained evidence-only and did not grant protected authority.

### Noise / Cough Rejection

Controlled safe cough/noise adapter proof:

- response: `outcome:"IGNORED"`
- reason: `voice transcript rejected as unsafe noise before runtime entry`
- transcript after reject: `messages:[]`
- evidence event `1`: `RejectedInput`
- modality `Voice`
- PH1.C status `Rejected`
- rejected reason `reason_code:1124073494`
- response hashes null
- TTS `NotRequested`
- PH1.M memory candidate status `BlockedRejectedTranscript`
- no committed conversation turn

JD's live cough also produced no visible message, which matches the controlled evidence proof.

### Protected Fail-Closed

Evidence event:

- event `52`: `ProtectedFailClosed`
- conversation turn `23`
- source `TypedText`
- modality `Typed`
- PH1.X protected risk ref `ph1x_protected_risk:1778987422829000000:1778987422829000001`
- PH1.M memory candidate status `BlockedProtected`
- protected refs:
  - `protected_fail_closed:no_simulation_no_authority:1778987422829000000:1778987422829000001`
  - `protected_execution:not_performed`
- audit ref:
  - `protected_no_execution_proof:1778987422829000000:1778987422829000001`

No business action executed.

### PH1.E Tool / Time Evidence

Evidence events:

- event `5`: Sydney time route
- event `10`: Melbourne time follow-up route
- event `39`: Shanghai typed time route
- event `43`: Suzhou typed follow-up route

Tool evidence contained:

- `ph1e_tool:time:...status:Ok`
- `ph1e_provider:google_time_zone` or `ph1e_provider:google_geocode_google_time_zone`
- deterministic source ref `ph1e_source:7d13e02d77d38777:Deterministic PH1.E source`
- PH1.X topic/intent refs such as `ph1x_topic:time` and `ph1x_intent:time`

### PH1.TTS / Spoken Output Evidence

TTS proof rows show:

- `tts_provider:"openai_first_runtime"`
- `tts_status:"Ready"` for Desktop OpenAI TTS generated audio
- `tts_status:"Requested"` on the runtime response row
- `approved_tts_text_hash` equals `spoken_text_hash`
- `spoken_matches_final_answer:true`
- replay refs such as `openai_tts_request:...`
- model ref such as `openai_tts_model:tts-1:voice:nova`

No Apple/native TTS fallback was introduced.

### PH1.X Evidence

Runtime response rows now expose:

- `active_context_packet_ref`
- `human_conversation_directive_ref`
- `active_topic_ref`
- `active_intent_ref`
- `continuation_ref`
- `protected_risk_ref` for protected attempts

Example:

- `ph1x_active_context:1778987065295000000:1778987065295000001`
- `ph1x_directive:1778987065295000000:1778987065295000001`
- `ph1x_topic:time`
- `ph1x_intent:time`

### PH1.M Evidence

Stage 7 rows include PH1.M slots:

- `memory_evidence_packet_ref`
- `memory_recall_request_ref`
- `fresh_memory_handoff_ref`
- `memory_continuation_decision_ref`
- `memory_no_match_ref`
- `memory_candidate_status`

For these live turns, the packet refs are null by design unless memory was used. Candidate statuses correctly block rejected/protected inputs and leave normal non-memory turns as `NotApplicable`.

## 6. Words Captured

The transcript snapshots prove successful capture for the main live prompts, including:

- `What's the time in Sydney?`
- `What about in Melbourne?`
- `what time is in shanghai`
- `and what about suzhou`
- `approve payroll for tim`
- `Approve payroll for Tim.`

JD reported one repeated voice phrase came from re-arm timing rather than a Stage 7 evidence failure. That repeated prompt was ignored as requested.

## 7. Root-Cause Repairs Made

### Repair 1: Missing Live Evidence Inspection Route

Owner: adapter transport/reporting surface.

Changed:

- `crates/selene_adapter/src/bin/http_adapter.rs`
- `crates/selene_adapter/src/lib.rs`

Added `/v1/ui/internal-history/evidence`, returning a redacted Stage 7 evidence report from the runtime PH1.F store. Desktop does not own or classify the evidence.

### Repair 2: Missing OpenAI TTS Evidence From Desktop TTS Bridge

Owner: adapter TTS evidence bridge.

Changed:

- `crates/selene_adapter/src/bin/http_adapter.rs`
- `crates/selene_adapter/src/lib.rs`

The OpenAI TTS route now records `Ready` evidence on successful audio generation and `FailedClosed` evidence on provider failure. The evidence includes model, voice, answer text hash, audio hash/byte length where available, and replay/timing refs.

### Repair 3: Response, Tool, Protected, Speaker, PH1.X, PH1.M Evidence Enrichment

Owner: adapter committed-turn and evidence bridge.

Changed:

- `crates/selene_adapter/src/lib.rs`

The adapter now files Stage 7 evidence for response text, approved TTS text, PH1.E time/tool route, PH1.X live context refs, PH1.M nullable slots, protected fail-closed refs, speaker posture, typed actor identity, and replay refs.

### Repair 4: Duplicate Voice User Evidence

Owner: adapter committed-turn bridge.

Changed:

- `crates/selene_adapter/src/lib.rs`

Root cause:

PH1.C accepted transcript commit already appended the voice user turn, then the adapter final transcript bridge could append the same user text again under a different idempotency key.

Fix:

`append_transcript_final_conversation_turn` now dedupes by correlation id, turn id, role, source, and text hash before appending. The targeted Stage 7 adapter test asserts the voice user turn is filed once.

## 8. Remaining Gap

The live proof found a real Stage 7 durability gap:

- the migration for `internal_history_evidence_ledger` exists
- the storage contract and in-memory append-only PH1.F ledger exist
- the live endpoint exposes the adapter process's in-memory PH1.F ledger
- after the adapter restarted, previous live evidence was no longer visible through the endpoint

This does not invalidate the evidence shape or the live filing inside the current process, but it does block a strong Stage 8 fresh-memory claim if Stage 8 expects evidence to survive adapter restart.

Recommended next owner-local work:

- wire `internal_history_evidence_ledger` through durable storage/replay, or
- explicitly constrain Stage 8 fresh-memory proof to one adapter lifetime and schedule durable replay as the next storage hardening build

## 9. Tests Run

Discovery:

- `rg -n "#\\[test\\]|tokio::test|stage7|internal.*history|history.*evidence|conversation_ledger|speaker.*evidence|voice_identity|modality|tts_text|rejected.*transcript|noise|protected.*fail|tool.*evidence|memory.*evidence|ActiveContextPacket|MemoryEvidencePacket" crates --glob '!target/**'`

Build and tests:

- `cargo fmt -p selene_adapter`
- `cargo check`
- `cargo test -p selene_kernel_contracts -- --test-threads=1`
- `cargo test -p selene_storage -- --test-threads=1`
- `cargo test -p selene_adapter -- --test-threads=1`
- `cargo test -p selene_os -- --test-threads=1`
- `cargo test -p selene_engines -- --test-threads=1`

Targeted Stage 7 adapter tests included:

- `stage7_adapter_committed_turn_bridge_files_internal_history_evidence`
- `stage7_adapter_redacted_evidence_report_exposes_live_ledger_rows`

## 10. Stage 7 Acceptance

Accepted:

- real voice turns filed
- real typed turns filed
- typed turns do not fabricate Voice ID
- speaker/Voice ID slots are nullable/evidence-only
- protected fail-closed evidence filed
- PH1.E time/tool evidence filed
- TTS/openai spoken-output evidence filed
- PH1.X evidence refs filed
- PH1.M evidence slots present and correctly nullable
- rejected cough/noise path files rejected evidence and blocks memory candidacy
- Desktop remained render/transport only

Not accepted for Stage 8 yet:

- restart-durable internal history evidence replay is not wired to the live endpoint

Readiness:

NOT_READY_FOR_STAGE_8_FRESH_MEMORY
