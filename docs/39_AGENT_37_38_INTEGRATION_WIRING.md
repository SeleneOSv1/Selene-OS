# 39_AGENT_37_38_INTEGRATION_WIRING

Status: report-only, repo-grounded, no runtime code changes.

Scope:
- How `37_AGENT_SIM_FINDER_CORE_BUILD_PLAN.md` and `38_AGENT_EXECUTION_CORE_BUILD_PLAN.md` wire into current Selene runtime for identity/session/trigger/thread/memory/persona boundaries.
- Current wiring and gaps are both listed explicitly.

## 1) 37 -> 38 interface anchor

Canonical design interface (docs):
- `docs/37_AGENT_SIM_FINDER_CORE_BUILD_PLAN.md` defines terminal packets:
  - `SimulationMatchPacket`
  - `ClarifyPacket`
  - `RefusePacket`
  - `MissingSimulationPacket`
- `docs/38_AGENT_EXECUTION_CORE_BUILD_PLAN.md` defines relay contract:
  - consume Finder packet as-is
  - dispatch only through `SimulationExecutor`
  - no re-ranking/rewrite of simulation candidates
  - `MissingSimulationPacket` must route to Dev Intake and stop dispatch

Current runtime anchor points for that contract:
- PH1.X request/decision boundary:
  - `crates/selene_os/src/app_ingress.rs`
  - `AppServerIngressRuntime::build_ph1x_request_for_forwarded_voice`
  - `build_ph1x_request_from_voice_forward`
- PH1.X execution boundary:
  - `crates/selene_os/src/app_ingress.rs`
  - `AppServerIngressRuntime::run_voice_turn_end_to_end`
- Simulation dispatch boundary:
  - `crates/selene_os/src/simulation_executor.rs`
  - `SimulationExecutor::execute_ph1x_dispatch_simulation_candidate`

## 2) VOICE.ID identity persistence across turns

Runtime call path:
1. Adapter builds VOICE.ID request per turn:
   - `crates/selene_adapter/src/lib.rs`
   - `run_voice_turn_internal`
   - `build_voice_id_request_from_ph1k_bundle`
2. OS runs identity assertion with signals:
   - `crates/selene_os/src/ph1os.rs`
   - `Ph1OsVoiceLiveRuntime::run_turn`
3. VOICE.ID execution and signal emission:
   - `crates/selene_os/src/ph1_voice_id.rs`
   - `Ph1VoiceIdLiveRuntime::run_identity_assertion_with_signal_emission`

Identity persistence source:
- Enrolled voice profiles are persisted and reused:
  - `crates/selene_storage/src/ph1f.rs`
  - `ph1vid_enroll_complete_commit` (writes `voice_profiles`)
  - `ph1vid_voice_profile_rows` (read path)
- Each live turn resolves enrolled speakers from stored profiles:
  - `crates/selene_os/src/app_ingress.rs`
  - `locked_enrolled_speakers_from_store`

Identity assertion object passed forward:
- `Ph1VoiceIdResponse` with:
  - `SpeakerAssertionOk` (`speaker_id`, optional `user_id`, `identity_v2`, diarization)
  - `SpeakerAssertionUnknown` (`candidate_user_id`, `reason_code`, `identity_v2`)
- Contract:
  - `crates/selene_kernel_contracts/src/ph1_voice_id.rs`
  - `SpeakerAssertionOk`, `SpeakerAssertionUnknown`, `Ph1VoiceIdResponse`

## 3) Session lifecycle (PH1.L) + open/resume behavior

PH1.L deterministic lifecycle exists:
- Runtime:
  - `crates/selene_os/src/ph1l.rs`
  - `Ph1lRuntime::step`
  - `Ph1lRuntime::on_wake`
  - `Ph1lRuntime::update_pending_question`
- Contract:
  - `crates/selene_kernel_contracts/src/ph1l.rs`
  - `SessionSnapshot`, `SessionId`
- Storage projection/guards:
  - `crates/selene_storage/src/ph1f.rs`
  - `upsert_session_lifecycle`
  - `is_allowed_session_transition`
  - plus `sessions` table in `crates/selene_storage/migrations/0001_ph1f_foundation.sql`

Current ingress wiring status (important):
- Adapter currently sets session state directly to active in voice-turn build input:
  - `crates/selene_adapter/src/lib.rs`
  - `run_voice_turn_internal` sets `AppVoicePh1xBuildInput.session_state = SessionState::Active`
- Adapter VOICE.ID request currently uses a synthetic `SessionSnapshot` with `session_id = Some(SessionId(1))`:
  - `crates/selene_adapter/src/lib.rs`
  - `build_voice_id_request_from_ph1k_bundle`

Conclusion:
- PH1.L lifecycle engine is implemented.
- App voice ingress path is currently not using PH1.L runtime state progression end-to-end; it passes active snapshot values directly.

## 4) Wake / side-button triggers and session opening

Trigger representation in runtime code:
- `crates/selene_os/src/ph1os.rs`
- `OsVoiceTrigger::{WakeWord, Explicit}`
- `wake_stage_required()` controls whether `PH1.W` is required.

Adapter/API trigger input:
- `crates/selene_adapter/src/lib.rs`
- `parse_trigger` accepts only `EXPLICIT|WAKE_WORD`.

Side-button mapping:
- Side-button/app-open is modeled as `EXPLICIT` in policy/docs:
  - `docs/ECM/PH1_W.md`
  - `docs/DB_WIRING/PH1_W.md`
  - iOS default explicit-trigger-only policy references side-button/app-open

Session-open semantics:
- In PH1.L runtime, accepted wake transitions `Closed -> Active` and `SoftClosed -> Active`:
  - `crates/selene_os/src/ph1l.rs`
  - `on_wake`
- In current app adapter path, explicit and wake both feed active snapshots; PH1.L runtime step is not currently in that hot path.

## 5) Thread continuity and `thread_key`

Adapter continuity behavior:
- `crates/selene_adapter/src/lib.rs`
- `resolve_adapter_thread_key`:
  - sanitizes, token-safe, max 96 chars, default `"default"`
- `load_ph1x_thread_state`:
  - loads current thread state by `(user_id, thread_key)`
- `persist_ph1x_thread_state`:
  - upserts thread state after PH1.X response

Storage truth for thread continuity:
- `crates/selene_storage/src/ph1f.rs`
- `validate_ph1x_thread_key`
- `ph1x_thread_state_upsert_commit`
- `ph1x_thread_state_ledger_rows`
- `ph1x_thread_state_current_row`

Continuity effect:
- `thread_key` provides deterministic per-user thread lanes with append-only history and current projection.

## 6) Memory behavior across sessions (non-negotiable)

Entry gate:
- Memory candidates are only collected when identity is confirmed:
  - `crates/selene_os/src/app_ingress.rs`
  - `build_ph1x_request_for_forwarded_voice`
  - branch `if forwarded.identity_confirmed()`

Memory retrieval call path:
- `crates/selene_os/src/simulation_executor.rs`
- `collect_context_memory_candidates_for_voice_turn`
- runs PH1.M context bundle + recall, returns `Vec<MemoryCandidate>`

PH1.X consumption:
- `crates/selene_kernel_contracts/src/ph1x.rs`
- `Ph1xRequest.memory_candidates`
- `crates/selene_os/src/ph1x.rs`
- uses candidates for bounded personalization and memory permission gating (`identity_allows_personalization`, `preferred_name`, sensitive-memory confirm)

Persistence across sessions:
- PH1.M tables are user-scoped DB truth (not per-turn ephemeral):
  - `crates/selene_storage/migrations/0021_ph1m_vnext_memory_tables.sql`
  - examples: `memory_atoms_ledger/current`, `memory_threads_ledger/current`, `emotional_threads_ledger/current`
- Memory provenance can carry session linkage:
  - `crates/selene_kernel_contracts/src/ph1m.rs`
  - `MemoryProvenance { session_id, transcript_hash }`

## 7) Emotional/personality lock: tone influence vs execution authority

Onboarding lock enforcement:
- `crates/selene_os/src/app_ingress.rs`
- `run_onboarding_emo_persona_lock`
- requires:
  - `tone_only == true`
  - `no_meaning_drift == true`
  - `no_execution_authority == true`
- commits persona lock:
  - `store.ph1persona_profile_commit`
  - `store.ph1onb_emo_persona_lock_commit`

Contract-level hard guard:
- `crates/selene_kernel_contracts/src/ph1emoguide.rs`
- `crates/selene_kernel_contracts/src/ph1persona.rs`
- validation enforces `tone_only` and no execution authority invariants

Execution isolation evidence:
- PH1.X dispatch decision path is intent/confirm/access/simulation based:
  - `crates/selene_os/src/ph1x.rs`
  - `decide_from_intent`, `out_dispatch_simulation_candidate`
- Dispatch execution remains in simulation executor:
  - `crates/selene_os/src/app_ingress.rs`
  - `run_voice_turn_end_to_end`
  - `crates/selene_os/src/simulation_executor.rs`
  - `execute_ph1x_dispatch_simulation_candidate`
- `Ph1xRequest` has no persona-style-profile field:
  - `crates/selene_kernel_contracts/src/ph1x.rs`
  - confirms tone hints are not execution selectors

Conclusion:
- Personality/emo lock is governance + tone-safety constrained.
- Execution authority remains outside persona/emo hints.

## 8) Exact data objects passed in 37/38 integration path

| Object | Producer | Consumer | Key Fields |
|---|---|---|---|
| `identity assertion` (`Ph1VoiceIdResponse`) | `Ph1OsVoiceLiveRuntime::run_turn` | `build_ph1x_request_from_voice_forward` via `IdentityContext::Voice` | `identity_v2`, `user_id/candidate_user_id`, `reason_code`, diarization |
| `session_id` (`SessionSnapshot.session_id`) | `build_voice_id_request_from_ph1k_bundle` (current adapter path) and PH1.L runtime contracts | VOICE.ID request, PH1.M provenance, session lifecycle tables | `SessionId`, `session_state`, `next_allowed_actions` |
| `thread_key` (`String`) | `resolve_adapter_thread_key` | thread-state load/persist in adapter + PH1.F | sanitized token-safe key, max 96, default `default` |
| `memory candidates` (`Vec<MemoryCandidate>`) | `collect_context_memory_candidates_for_voice_turn` | PH1.X request handling | `memory_key`, `memory_value`, `confidence`, `sensitivity_flag`, `use_policy`, `provenance` |
| `persona hints` (lock artifacts) | onboarding emo/persona lock flow | stored in PH1.PERSONA audit payload; not in PH1.X dispatch request | `style_profile_ref`, `delivery_policy_ref`, `preferences_snapshot_ref`; tone-only/no-exec invariants |

## 9) End-to-end example: "Selene order pizza"

This section is split into current repo reality vs 37/38 target wiring to stay strictly non-guessing.

### 9.1 Current repo behavior for exact phrase "Selene order pizza"

Ingress and identity path:
1. Trigger parsed (`EXPLICIT` or `WAKE_WORD`) in adapter:
   - `crates/selene_adapter/src/lib.rs`
   - `parse_trigger`
2. VOICE.ID request built and executed:
   - `build_voice_id_request_from_ph1k_bundle`
   - `Ph1OsVoiceLiveRuntime::run_turn`
3. PH1.X request built with:
   - `IdentityContext::Voice(voice_identity_assertion)`
   - `memory_candidates` only if identity confirmed
   - `thread_state` loaded by `(user_id, thread_key)`

Intent matching fact:
- Current PH1.NLP rule path only explicitly maps book-table wording (`"book a table"`, `"book table"`):
  - `crates/selene_engines/src/ph1n.rs`
  - detector branch in `detect_intents`
- No explicit `"pizza"` matcher is present in that rule set.

Result:
- Exact phrase `"order pizza"` is not repo-proven as a simulation match in current detector.
- It will route through clarify/fail-closed behavior unless upstream wording/context resolves to an existing intent.

Memory + personality effects in this current path:
- Memory can influence response/clarify only when identity confirmed and candidate rules apply.
- Persona/emo lock constrains tone behavior and remains non-execution authority.
- Simulation execution path is unchanged: PH1.X -> SimulationExecutor only.

### 9.2 37+38 target behavior for the same phrase (design target)

When 37 Finder is implemented as specified:
1. Finder candidate pipeline (synonyms + tenant vocab + gold mappings + context memory) can map `"order pizza"` to an existing simulation family.
2. Finder emits top-1 `SimulationMatchPacket` (or one `ClarifyPacket` if fields missing).
3. 38 Execution Core consumes packet unchanged and enforces:
   - confirm (if required)
   - per-user access
   - ACTIVE simulation
   - idempotent dispatch through `SimulationExecutor`

Non-negotiable behavior in that target flow:
- Memory/personality can change wording/tone of clarify/response.
- Memory/personality cannot alter simulation authority path.
- If no simulation exists after ACTIVE->DRAFT checks, Execution Core must route to Dev Intake (no dispatch).

## 10) Wiring verdict (current state)

- VOICE.ID persistence across turns: `Wired` (voice profiles persisted; per-turn assertion emitted).
- Session lifecycle PH1.L: `Implemented but partially integrated` (runtime exists; adapter voice ingress currently uses active snapshots directly).
- Wake/explicit trigger routing: `Wired` (`WakeWord`/`Explicit`; side-button represented by `Explicit` policy).
- Thread continuity via `thread_key`: `Wired` (sanitized key + DB truth ledger/current).
- Memory across sessions: `Wired with identity gating` (confirmed-identity fetch path + PH1.M user-scoped persistence).
- Persona/emo lock and tone-only boundaries: `Wired for governance`; execution isolation `enforced` (no persona field in dispatch authority path).
