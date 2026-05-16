# Stage 6.7 PH1.VOICE.ID Speaker Evidence Audit

## 1. Executive Conclusion

Readiness statement: READY_FOR_STAGE_7_SPEAKER_EVIDENCE.

Repo truth shows a real PH1.VOICE.ID owner already exists. Voice ID is represented by canonical kernel contracts, a fail-closed engine, OS live runtime wiring, runtime-envelope transport, governance adoption, PH1.X/PH1.M identity gates, enrollment storage, and Desktop proof-only surfaces.

Stage 7 can include speaker evidence now by storing nullable immutable speaker-evidence refs and compact per-turn identity posture fields. Stage 7 should not build a new Voice ID system, should not treat Voice ID as authority, and should not require every turn to have a known speaker. Unknown speaker must remain a valid, safe posture.

The main Stage 7 gap is storage/archive/ledger shape: the current conversation ledger stores `user_id`, `device_id`, `source`, and text, but not explicit per-turn `speaker_id`, `voice_identity_assertion_ref`, confidence, liveness, identity posture, same-speaker continuity, or capture attestation refs.

Recommendation: Option 1 - Stage 7 can include speaker evidence now using existing PH1.VOICE.ID contracts. The Stage 7 storage rows should include explicit nullable speaker evidence refs for later backfill and must align with the existing PH1.VOICE.ID and Stage 9 voice identity evidence/posture shapes instead of inventing a parallel speaker identity contract.

## 2. Current Repo Baseline

- Branch: `main`
- Start HEAD: `b723b39a85e9fc9730e1b86738e3e21fa30df4d7`
- Start `origin/main`: `b723b39a85e9fc9730e1b86738e3e21fa30df4d7`
- Stage 6.6 ancestor proof: `STAGE6_6_ANCESTOR_OK`
- Start tree: clean
- Scope: no code changes, no behavior changes, report-only audit

Reviewed required sources:

- `/Users/selene/Documents/Selene-OS/AGENTS.md`
- `/Users/selene/Documents/Selene-OS/docs/SELENE_ALWAYS_AVAILABLE_VOICE_CONTINUOUS_CHAT_SESSION_MEMORY_MASTER_PLAN.md`
- `/Users/selene/Documents/Selene-OS/docs/reports/STAGE_6_6_PH1X_PH1M_CONTRACT_FOUNDATION.md`
- `/Users/selene/Documents/Selene-OS/docs/CORE_ARCHITECTURE.md`
- `/Users/selene/Documents/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md`
- `/Users/selene/Documents/Selene-OS/docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`

## 3. PH1.VOICE.ID Owner Truth

| Area | Owner path | Repo truth |
| --- | --- | --- |
| Canonical contracts | `/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1_voice_id.rs` | Owns `Ph1VoiceIdRequest`, `Ph1VoiceIdResponse`, `SpeakerAssertionOk`, `SpeakerAssertionUnknown`, `VoiceIdentityV2`, `IdentityTierV2`, `VoiceIdDecision`, `SpeakerId`, `UserId`, `IdentityConfidence`, `VoiceEmbeddingCaptureRef`, liveness and enrollment contracts. |
| Voice ID engine | `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1_voice_id.rs` | Owns speaker matching, fail-closed unknown decisions, spoof/no-speech/multi-speaker/echo/stale-wake guards, thresholds, reauth, speaker-change detection, and reason codes. |
| OS live runtime | `/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1_voice_id.rs` | Owns live assertion orchestration, embedding gate profiles, tenant threshold packs, audit/KPI/signal emission, and proof that speaker identity alone cannot authorize protected actions. |
| Runtime envelope transport | `/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs` | `RuntimeExecutionEnvelope` carries `voice_identity_assertion: Option<Ph1VoiceIdResponse>` read-only for runtime adoption. |
| OS forwarded voice bundle | `/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1os.rs` | `OsVoiceLiveForwardBundle` carries `voice_identity_assertion` and attaches it to the runtime execution envelope. |
| Runtime governance | `/Users/selene/Documents/Selene-OS/crates/selene_os/src/runtime_governance.rs` | Derives/adopts `IdentityExecutionState` from `Ph1VoiceIdResponse` and keeps unknown/re-auth/spoof/no-speech/multi-speaker as restricted/degraded posture, not authority. |
| App ingress | `/Users/selene/Documents/Selene-OS/crates/selene_os/src/app_ingress.rs` | Requires canonical forwarded voice identity assertions for voice agent input packets and gates memory/context collection by confirmed identity state. |
| Adapter bridge | `/Users/selene/Documents/Selene-OS/crates/selene_adapter/src/lib.rs` | Builds Voice ID request/observation from Desktop/app signals and PH1.K bundle, passes them into `AppVoiceIngressRequest`, and uses Voice ID posture for named wake greeting only. |
| Adapter proto | `/Users/selene/Documents/Selene-OS/crates/selene_adapter/proto/voice_ingress.proto` | Current gRPC voice turn request does not carry full speaker evidence fields. |
| Storage enrollment/profile | `/Users/selene/Documents/Selene-OS/crates/selene_storage/migrations/0008_ph1vid_voice_enrollment_tables.sql` and `/Users/selene/Documents/Selene-OS/crates/selene_storage/src/ph1f.rs` | Stores voice enrollment sessions, samples, profiles, bindings, local voice cache, artifact refs, and profile/person links. |
| Conversation ledger | `/Users/selene/Documents/Selene-OS/crates/selene_storage/migrations/0001_ph1f_foundation.sql` | Stores turn/session/user/device/source/text metadata, but not explicit per-turn Voice ID evidence refs. |
| Desktop proof-only surfaces | `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopLiveVoiceE2EProofView.swift` and `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopControlledWakeModeView.swift` | Proof/control UI only. Desktop is not the identity owner. |

## 4. Current Speaker Evidence Truth

| Evidence | Exists now | Current evidence |
| --- | --- | --- |
| `speaker_id` | Yes, but not per-turn ledger | `SpeakerAssertionOk.speaker_id`, `VoiceIdentityV2.speaker_id`, `IdentityRecord.speaker_id`, Stage 9 evidence packet. |
| Speaker name/label | Partial | `SpeakerLabel`, Desktop proof UI speaker name, person profile preferred name. Not a canonical per-turn ledger field. |
| `voice_profile_id` | Partial | Stored in voice profile tables and proof tooling; not always present in runtime assertion. |
| Known/unknown speaker state | Yes | `Ph1VoiceIdResponse::SpeakerAssertionOk` vs `SpeakerAssertionUnknown`; `VoiceIdentityV2.identity_tier_v2`. |
| Confidence score | Yes | `IdentityConfidence`, `score_bp`, `margin_to_next_bp`, Stage 9 confidence fields. |
| Identity posture | Yes | `IdentityTierV2`, `VoiceIdDecision`, OS/governance `IdentityExecutionState`, Stage 9 posture packet. |
| Same speaker as previous turn | Partial | Engine detects session speaker changes, but Stage 7 needs explicit per-turn `same_speaker_as_previous`. |
| Speaker changed flag | Partial | Engine reason code and session fingerprint change detection exist; Stage 7 needs immutable per-turn field. |
| Voice ID evidence ref | Partial | Stage 9 has `speaker_assertion_id`/`voice_identity_id`; runtime envelope carries assertion but Stage 7 ledger ref is missing. |
| Capture attestation ref | Partial | Adapter request includes `attestation_ref`; Stage 7 ledger needs a nullable immutable ref. |
| Processed audio / embedding refs | Yes as refs | `processed_audio_stream_ref`, `VoiceEmbeddingCaptureRef`; raw audio/embedding storage is guarded separately. |
| Anti-spoof/liveness | Yes | `SpoofLivenessStatus`, spoof/no-speech/multi-speaker reason codes and engine guards. |
| Identity drift/staleness | Partial | Reauth/stale wake/low margin/unknown reason codes exist; Stage 7 should store compact staleness/drift posture. |
| Identity scope/privacy scope | Partial | Tenant/platform/channel/device/user context exists; PH1.F has privacy scope; Stage 7 needs explicit identity/privacy scope per turn. |
| Access posture | Partial | Governance can attach `IdentityExecutionState`; Stage 7 needs a ref/summary field. |

## 5. Turn Integration Truth

Voice turns:

- Adapter builds `voice_id_request` and `voice_id_observation` for voice turns in `/Users/selene/Documents/Selene-OS/crates/selene_adapter/src/lib.rs`.
- OS live runtime returns `Ph1VoiceIdResponse` and `OsVoiceLiveForwardBundle` carries it.
- Runtime execution envelope can carry `voice_identity_assertion`.
- App ingress requires canonical forwarded voice identity assertion for forwarded voice agent input packets.
- PH1.X and PH1.M can see identity context/assertions through OS/runtime request paths.
- Conversation ledger currently does not store full per-turn Voice ID evidence refs.

Typed turns:

- Typed turns should use actor/user/session identity where available, with nullable Voice ID fields.
- Typed turns should not require voice identity.
- If the user identity is known through app/session/auth, Stage 7 should store that as identity scope, not fake voice evidence.

PH1.X:

- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1x.rs` and `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1x.rs` already treat voice identity as context only.
- Voice personalization is gated to confirmed/known voice identity.
- Unknown or uncertain speaker can trigger prompt posture; it does not grant authority.

PH1.M:

- `/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/ph1m.rs` carries `speaker_assertion: Ph1VoiceIdResponse` through recent recall, digest upsert, resume selection, hint bundle, and related memory contracts.
- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1m.rs` rejects unknown speaker for store/propose style memory operations.
- PH1.M is already designed to scope memory by identity evidence, but Stage 7 must store enough per-turn evidence for later replay/recall.

PH1.ACCESS/GOV:

- Runtime governance uses Voice ID as one input to identity execution posture.
- Existing tests prove speaker identity alone cannot authorize protected actions.
- Stage 7 must preserve this boundary by storing `authority_use = evidence_only` or equivalent posture.

Unknown speaker:

- Unknown speaker is explicitly represented and fail-closed.
- Unknown speaker is not an error in itself; it is safe evidence.

Incorrect authority:

- No current owner treats Voice ID alone as protected-action authority.
- Existing proof tests lock this behavior.

## 6. PH1.X Implications

PH1.X needs Stage 7 to preserve these speaker evidence fields so live context can later behave humanly and safely:

- Current speaker posture: known, probable, unknown, guest, or blocked.
- Current speaker id and user id when known.
- Same speaker as previous turn.
- Speaker changed since prior turn.
- Confidence and reason code.
- Whether private context/memory can be used.
- Whether active context should continue, reset, or clarify on speaker change.
- Voice evidence refs for audit and replay.
- Identity prompt status when speaker is unknown or low confidence.

PH1.X must not:

- Resolve identity from Desktop labels.
- Treat device trust as speaker identity.
- Treat Voice ID as authority.
- Keep active context across speaker change without confidence/clarification policy.

## 7. PH1.M Implications

PH1.M needs Stage 7 to preserve speaker evidence for:

- Speaker-scoped memory.
- User-scoped memory.
- Shared/tenant/project memory.
- Guest or unknown speaker restrictions.
- Private memory exposure rules.
- Memory recall allowed/blocked reason.
- Trust and source type for remembered evidence.
- Identity confidence and staleness posture.
- Fresh memory handoff across sleep/wake without leaking to the wrong speaker.

PH1.M must not:

- Use raw Desktop speaker names as identity proof.
- Store memory for unknown speaker as private user memory.
- Let Voice ID bypass protected execution.
- Treat old session text as belonging to the current speaker without evidence.

## 8. Stage 7 Speaker Evidence Requirements

Stage 7 should store nullable speaker evidence refs for every turn and richer values when available:

1. `turn_id`
2. `internal_segment_id` / `session_id`
3. `thread_key`
4. `correlation_id`
5. `modality` (`voice`, `typed`, etc.)
6. `actor_user_id` / app user id when known
7. `speaker_id`
8. `speaker_label` or display name only when policy allows
9. `voice_profile_id`
10. `voice_id_confidence`
11. `score_bp`
12. `margin_to_next_bp`
13. `voice_id_decision`
14. `identity_tier_v2`
15. `identity_posture`
16. known/unknown/guest classification
17. `same_speaker_as_previous`
18. `speaker_changed`
19. `voice_identity_assertion_ref`
20. `speaker_assertion_id` or equivalent immutable assertion ref
21. `voice_id_evidence_ref`
22. `processed_audio_stream_ref`
23. `voice_embedding_capture_ref`
24. `capture_attestation_ref`
25. anti-spoof/liveness status refs
26. multi-speaker evidence/ref
27. identity drift/staleness/refusal reason
28. `identity_scope`
29. `privacy_scope`
30. `access_posture` / `identity_execution_state_ref`
31. memory recall allowed/blocked flag
32. memory recall allowed/blocked reason
33. protected execution identity posture: evidence-only
34. audit/proof refs
35. raw-audio/embedding storage flags confirming no raw biometric payload is written into turn history

Typed-turn rule:

- Typed turns should store app/session actor identity and nullable Voice ID refs.
- A typed turn should not fabricate a `speaker_id` unless the repo has a lawful app identity binding for that user.

Voice-turn rule:

- Voice turns should store the canonical `Ph1VoiceIdResponse` ref/snapshot fields when available.
- Unknown speaker should be stored as unknown evidence, not dropped.

## 9. Gap Table

| Required speaker evidence | Exists now | Owner path | Current evidence | Stage 7 gap | Recommended next build | Risk if skipped |
| --- | --- | --- | --- | --- | --- | --- |
| Canonical Voice ID assertion | Yes | `crates/selene_kernel_contracts/src/ph1_voice_id.rs` | `Ph1VoiceIdResponse` | Store immutable assertion ref per turn | Stage 7 storage row/ref | Future memory cannot prove who spoke. |
| Runtime assertion transport | Yes | `crates/selene_kernel_contracts/src/runtime_execution.rs` | `voice_identity_assertion` in envelope | Persist ref in archive/ledger | Stage 7 | Runtime truth lost at storage boundary. |
| Speaker id | Partial | `ph1_voice_id.rs`, `ph1f.rs` | `SpeakerAssertionOk.speaker_id`, `IdentityRecord.speaker_id` | Add per-turn nullable field/ref | Stage 7 | PH1.M cannot scope memory by speaker. |
| Speaker name/label | Partial | `ph1_voice_id.rs`, Desktop proof, person profile storage | `SpeakerLabel`, proof UI, person profile | Policy-bound display label ref | Stage 7 or later UI | UI/memory may show wrong person or leak name. |
| Known/unknown posture | Yes | `ph1_voice_id.rs`, `ph1_voice_id.rs` engine | Ok vs Unknown, `IdentityTierV2` | Store per-turn posture | Stage 7 | Unknown speaker may be treated as missing data. |
| Confidence/margin | Yes | `ph1_voice_id.rs` | `confidence`, `score_bp`, `margin_to_next_bp` | Store compact per-turn fields | Stage 7 | Later recall cannot judge trust. |
| Liveness/spoof posture | Yes | `ph1_voice_id.rs`, engine | `SpoofLivenessStatus`, reason codes | Store ref/status per turn | Stage 7 | Spoof-risk memory could be reused unsafely. |
| Same speaker previous | Partial | engine session lock/change detection | speaker fingerprint/session checks | Add immutable per-turn boolean | Stage 7 | PH1.X may continue across speaker swap. |
| Speaker changed | Partial | engine reason codes/session checks | speaker change detection | Add immutable per-turn boolean/reason | Stage 7 | PH1.X cannot reset/clarify at boundary. |
| Capture attestation | Partial | adapter/live proof | `attestation_ref`, capture refs | Store nullable ref | Stage 7 | Audit cannot prove capture provenance. |
| Processed audio/embedding refs | Yes as refs | `ph1_voice_id.rs` | `processed_audio_stream_ref`, `VoiceEmbeddingCaptureRef` | Store refs only, no raw payload | Stage 7 | Biometric evidence chain breaks or raw payload risk grows. |
| Conversation ledger integration | No explicit per-turn speaker evidence | `crates/selene_storage/migrations/0001_ph1f_foundation.sql` | `user_id`, `device_id`, `source` only | Add speaker evidence refs/fields | Stage 7 | Immutable history is not identity-aware. |
| PH1.M speaker assertion use | Yes | `crates/selene_kernel_contracts/src/ph1m.rs` | `speaker_assertion` on recall/digest/resume/hint requests | Stage 7 must preserve evidence refs feeding PH1.M later | Stage 7 | Memory scoping cannot be replayed. |
| PH1.X speaker context | Partial | `crates/selene_os/src/ph1x.rs`, `crates/selene_engines/src/ph1x.rs` | confirmed voice identity gates personalization | Stage 7 must provide refs for later active context evidence | Stage 7 | Active context may continue without identity evidence. |
| Governance posture | Yes | `crates/selene_os/src/runtime_governance.rs` | `IdentityExecutionState` from Voice ID | Store access posture/ref | Stage 7 | Protected fail-closed audit loses identity posture. |
| Stage 9 voice evidence shape | Yes | `crates/selene_os/src/runtime_ingress_turn_foundation.rs` | `Stage9VoiceIdentityEvidence`, `Stage9VoiceIdentityPosturePacket` | Align Stage 7 fields with this shape | Stage 7 | Duplicate later-stage identity evidence contract. |

## 10. Duplicate / Stale Path Report

No code was deleted. The following paths must be treated carefully:

| Path | Status | Reason |
| --- | --- | --- |
| `/Users/selene/Documents/Selene-OS/crates/selene_adapter/src/lib.rs` `ph1m_actor_recent_recall_assertion` | RETAINED_COMPATIBILITY_PATH | Adapter creates a synthetic actor assertion for recent PH1.M recall. It is active compatibility, not canonical Voice ID evidence. Stage 7 should not make this the speaker identity brain. |
| `/Users/selene/Documents/Selene-OS/crates/selene_adapter/src/lib.rs` wake Voice ID greeting posture | RETAINED_COMPATIBILITY_PATH | Used only for named greeting gating. It must remain display/posture-only and not become identity authority. |
| `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopLiveVoiceE2EProofView.swift` | RETAINED_PROOF_SURFACE | Proof UI can show speaker/profile/posture, but Desktop must not own identity meaning. |
| `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopControlledWakeModeView.swift` | RETAINED_PROOF_SURFACE | Controlled proof UI only. |
| `/Users/selene/Documents/Selene-OS/apple/iphone/SeleneIPhone/SessionShellView.swift` display `speaker` strings | UI_LABEL_ONLY | These are not Voice ID evidence and must not be used as identity proof. |
| `/Users/selene/Documents/Selene-OS/crates/selene_os/src/runtime_ingress_turn_foundation.rs` Stage 9 voice identity packets | ACTIVE_FUTURE_ALIGNMENT_PATH | Not duplicate owner. Stage 7 should align with these fields rather than invent a competing shape. |
| `/Users/selene/Documents/Selene-OS/crates/selene_storage/migrations/0001_ph1f_foundation.sql` `identities.speaker_id` | COARSE_IDENTITY_ROW | Useful but not per-turn speaker evidence. Stage 7 still needs turn-level refs. |

Potential stale or risky patterns:

- Adapter-local synthetic memory assertions should stay compatibility-only until PH1.M receives stored canonical Stage 7 speaker evidence.
- Desktop/iPhone UI speaker labels must remain presentation labels only.
- Device trust must never imply speaker identity.
- Voice ID must remain evidence/posture only and not authority.

## 11. Recommended Next Build

Recommended option: Option 1 - Stage 7 can include speaker evidence now using existing PH1.VOICE.ID contracts.

Stage 7 should:

- Store nullable Voice ID evidence refs and compact posture fields on immutable turn/internal segment rows.
- Reuse `Ph1VoiceIdResponse` as the canonical assertion source.
- Align field names with existing Stage 9 `Stage9VoiceIdentityEvidence` and `Stage9VoiceIdentityPosturePacket`.
- Store unknown speaker evidence, not just known speaker evidence.
- Store typed turns with actor/session identity and nullable Voice ID fields.
- Preserve `authority_use = evidence_only` or equivalent protected-action posture.
- Avoid new Desktop or adapter identity ownership.

Stage 6.8 Voice ID contract/wiring build required: No, not for Stage 7 speaker-evidence readiness. A later wiring build may be useful if Stage 7 wants live per-turn attachment everywhere immediately, but the existing contracts are sufficient for Stage 7 to reserve and store speaker evidence refs now.

## 12. Readiness Statement

READY_FOR_STAGE_7_SPEAKER_EVIDENCE

Exact reason:

- Canonical PH1.VOICE.ID contracts already exist.
- Fail-closed Voice ID engine already exists.
- Runtime envelope can carry Voice ID assertions.
- OS live runtime and governance already consume assertions.
- PH1.X and PH1.M already have identity-aware paths.
- Storage has voice enrollment/profile tables.
- Stage 7 can add immutable per-turn speaker evidence refs without inventing a new Voice ID system.

Remaining Stage 7 work:

- Add per-turn speaker evidence refs/fields to Stage 7 immutable history.
- Attach typed actor identity separately from Voice ID.
- Preserve unknown speaker as safe evidence.
- Preserve access/governance posture as evidence only.
- Ensure PH1.M and PH1.X can later read speaker-scoped evidence without Desktop or adapter shortcut ownership.

## 13. Validation Summary

Test discovery:

- Ran `rg -n "#\\[test\\]|tokio::test|voice_id|voice id|speaker|identity|known speaker|unknown speaker|voice profile|ph1_voice|identity posture" crates apple --glob '!target/**'`.
- Discovery found PH1.VOICE.ID contract tests, engine tests, OS live gate tests, adapter posture tests, storage Voice ID DB wiring tests, Stage 34P authority-boundary tests, and Stage 9 voice identity posture/evidence tests.

Targeted validation:

- `cargo test -p selene_kernel_contracts ph1_voice_id -- --test-threads=1`
  - Result: passed, 17 tests.
- `cargo test -p selene_engines ph1_voice_id -- --test-threads=1`
  - Result: passed, 21 tests.
- `cargo test -p selene_os ph1_voice_id -- --test-threads=1`
  - Result: passed, 56 tests.
- `cargo test -p selene_adapter voice_id -- --test-threads=1`
  - Result: passed, 15 tests.
- `cargo test -p selene_storage ph1_voice_id -- --test-threads=1`
  - Result: command completed, but this filter matched no storage tests; this is recorded rather than counted as storage proof.
- `cargo test -p selene_storage --test db_wiring_ph1vid_tables -- --test-threads=1`
  - Result: passed, 18 tests.
- `cargo check`
  - Result: passed.

No runtime behavior was changed.
No code files were changed.
