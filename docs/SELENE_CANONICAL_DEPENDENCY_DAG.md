# Selene Canonical Dependency DAG

Status: STAGE1_DEPENDENCY_CONTROL_ARTIFACT
Date: 2026-05-03

This graph converts the 34-stage roadmap into build-control dependencies. The numbered order remains canonical. Parallel work is allowed only inside documented slices when the upstream packet proof already exists and write scopes do not overlap.

## Global Must-Not-Start-Early Rules

- No provider/model/STT/TTS routing implementation before Stage 3 provider, KMS, budget, consent, and provider-off proof.
- No live voice understanding before Stage 8 final transcript commit proof.
- No protected mutation before Stage 12 risk, authority, simulation, execution gate, law, audit, and idempotency proof.
- No Write consumer may read raw provider output; it must consume evidence or presentation-safe packets.
- No native client may call providers, rank sources, choose images, authorize actions, or execute protected mutations.
- No benchmark lane may be marked complete without one of the benchmark statuses defined in the master plan.

## Stage DAG

| Stage | Required Input | Emits | Upstream Dependencies | Downstream Consumers | Parallel / Blocked Notes |
|---|---|---|---|---|---|
| 1 Canonical Inventory And Wiring Map | repo files and docs authority stack | inventory docs, dependency DAG, slice map, benchmark matrix | clean repo and canonical 34-stage plan | all later stages | PROVEN_COMPLETE before Stage 2. |
| 2 Runtime Kernel, Storage, Proof Ledger, And Law Foundation | Stage 1 docs | runtime/proof/replay/benchmark envelopes, trace/audit crosswalk | Stage 1 | Stages 3-34 | PROVEN_COMPLETE by narrowed Stage 2A; must stay complete before provider/feature work. |
| 3 Provider, Secret, KMS, Cost, Quota, Vault, And Early Consent | Stage 2 envelopes | provider/model/prompt governance, budget, consent, KMS, provider-off proof | Stages 1-2 | Stages 7-8, 11-17, 24, 28, 30, 34 | Stage 3A PROVEN_COMPLETE for provider-off, startup no-probe, KMS/cost/quota crosswalk, and early consent baseline. Stage 3B PROVEN_COMPLETE for inert STT/TTS provider profile and route-decision contracts. Broad model/champion routing, live eval, promotion/rollback, and cost-quality scoring remain deferred to Stage 30. |
| 4 Activation, Session, Turn, And Packet Foundation | runtime envelope, consent/device/provider budget packets | activation, turn candidate, committed-turn boundary fields | Stages 2-3 | Stages 5, 7, 8, 20, 27 | Stage 4A PROVEN_COMPLETE for packet-boundary/no-route-authority and record-artifact-only separation; wake/live voice/record product remain owned by later stages. |
| 5 Session Open, Resume, Close, And Runtime Turn Spine | activation/session/turn packets | current committed turn, conversation goal/open-loop state | Stage 4 | Stages 6, 8-12, 15, 21, 29, 34 | PROVEN_COMPLETE by Stage 5A current-turn authority and Stage 5B conversation-control/same-page reconciliation. Only current Stage 5A authority can update advisory conversation state; stale/record/closed turns are blocked. Stage 6A may start. |
| 6 Master Access, Tenant, Policy, And Authority Context | session/current turn, identity hints | access context, tenant/workspace policy | Stages 3-5 | Stages 9, 12, 21, 24-27, 31 | PROVEN_COMPLETE by Stage 6A. `Stage6AccessContextPacket` consumes current Stage 5 authority, keeps public read-only non-mutating, keeps protected context non-executing, and fails closed for unknown/wrong speaker, cross-tenant, revoked consent, untrusted device, missing access, and policy denial. Stage 7A may start. |
| 7 Wake, Side Button, And Activation Stack | activation packet foundation and consent baseline | wake/side-button activation packets | Stages 3-6 | Stage 8, native clients, Stage 34 | PROVEN_COMPLETE by Stage 7A. `Stage7ActivationContextPacket` keeps wake/side-button activation session/attention-only, blocks iPhone always-listening wake, preserves wake training consent boundary, and cannot understand, answer, search, call providers, identify, authorize, speak, route tools, connector-write, or execute. Stage 8A may start. |
| 8 Voice I/O, Listen State, Transcript Gate | wake/explicit mic/live voice packets | audio scene, final transcript, interruption state | Stages 3-7 | Stages 9-10, 17, 34 | Stage 8A, Stage 8B, Stage 8C, Stage 8D, Stage 8E, and Stage 8F PROVEN_COMPLETE for audio/listen/transcript-gate, VAD/endpoint, partial/final, confidence/coverage, protected-slot no-guess, listening-scene advisory/blocking foundation, deterministic numeric listening-lab benchmark envelopes, deterministic accent/mixed-language/domain-vocabulary/alternative/second-pass repair benchmark envelopes, and output-interaction boundaries. `Stage8TranscriptGatePacket` consumes Stage 7 activation and Stage 5 current-turn authority, keeps partial transcripts preview-only, requires endpoint-final plus confidence/coverage pass for final commit, blocks confidence/protected-slot rejects before understanding, keeps record-mode audio artifact-only, carries `Stage8AudioScenePacket` evidence without identity/authority/execution power, and now has PH1.J-backed Stage 8D/8E fixture-only WER/CER/latency/calibration/repair scoring plus `Stage8FOutputInteractionPacket` stale-output/self-echo boundary proof. Stage 8 carriers cannot call providers, capture live audio, understand, identify, authorize, route, speak, connector-write, mutate, or promote providers. Stage 3B provider-router contracts are available for later Stage 8/17 live integration. Stage 9A consumed the safe Stage 8 references; live/native listening and playback quality remain deferred to Stage 17/34 or later explicit live/native-lab stages. |
| 9 Voice ID Stack | final voice capture and consent | voice identity packet | Stages 6-8 | Stages 10-12, 21, 31, 34 | PROVEN_COMPLETE by Stage 9A for identity-posture foundation. `Stage9VoiceIdentityPosturePacket` crosswalks PH1.VOICE.ID response evidence, governed enrollment/artifact references, Stage 5 current-turn authority, Stage 6 access context, and safe Stage 8 final-transcript metadata. Voice ID is receipt-only speaker posture and can inform access context, but cannot authorize by itself, execute, search, call providers, capture live audio, transcribe live audio, route, speak, connector-write, or mutate. Unknown, low-confidence, wrong-speaker, multi-speaker, revoked-consent, revoked-artifact, cross-tenant, device-mismatch, stale-sample, unsafe Stage 8 signal, stale-turn, and record-artifact-only cases fail closed. Stage 10A may start; live/native Voice ID quality remains deferred to Stage 34 or later explicit native-lab stages. |
| 10 Universal Understanding And Perception Assist | committed turn, transcript, voice identity/access context | understanding packet, meaning hypotheses, clarification packet | Stages 5-9 | Stage 11, Stage 15, Stage 21, Stage 29, Stage 32 | Broad stage; split into language/meaning slices. |
| 11 Reasoning Orchestrator, Capability Registry, Tool Route | understanding packet | route candidate packet, task/model profile candidate | Stages 3, 10 | Stage 12, Stage 13, Stage 22, Stage 24 | Cannot execute. |
| 12 Runtime Risk, Authority, Simulation, Execution Gate | route candidate, access, policy, risk context | approved execution plan or fail-closed packet | Stages 6, 11 | Stages 13, 24-27, 30, 34 | Protected actions require full gate chain. |
| 13 Search, Source, Image Evidence, Public Tool Quality | approved read-only route or public tool route | evidence, source, citation, image/product/vertical packets | Stages 3, 11-12 | Stages 14-16, 18, 22, 24, 34 | Broad stage; Search Builds 1-3 and 7 only. |
| 14 Web Search Enterprise Sublanes And Release Proof | Stage 13 evidence foundations | deep research packet, research report, release proof | Stage 13 | Stages 15-16, 30, 34 | Deep research and enterprise release proof. |
| 15 Write, Response, And TTS-Safe Text Engine | evidence/image/research packets plus empty/live experience packets | write response packet, `tts_text` | Stages 10, 13-14 | Stages 16-17 | Broad stage; no fact invention or source reranking. |
| 16 Presentation Contracts | write response packet | presentation envelope | Stage 15 | Stages 18-20, 33-34 | Must precede rich native renderer work. |
| 17 Speech/TTS Output And Playback Control | approved `tts_text`, prosody packet | TTS packet/playback state | Stages 3, 8, 15-16 | Stages 18-20, 29, 34 | TTS cannot rewrite. |
| 18 Adapter, Protocol, And Rich Transport | presentation envelope, TTS packet, state packets | adapter response, client render packet | Stages 16-17 | Stages 19-20, 33 | Adapter preserves packets only. |
| 19 Desktop Native Runtime And Renderer | adapter response/client render packet | Desktop render proof | Stage 18 | Stage 33, Stage 34 | Mac current; Windows planned/missing. |
| 20 Mobile Native Runtime And Renderer | adapter response/client render packet | iPhone/Android render proof | Stage 18 | Stage 33, Stage 34 | iPhone current; Android planned/missing. |
| 21 Project, Memory, Persona, Workspace, Context | session, access, context, write/TTS consumers | memory trust/context and persona packets | Stages 5-6, 10, 15, 17 | Stages 29, 31, 34 | Broad stage; no cross-project leakage. |
| 22 File, Document, Data, Vision, OCR, Media Understanding | read-only route, artifact inputs, Stage 13 reader foundation | document/data/vision evidence artifacts | Stages 11-13, 16 | Stages 23, 34 | Data sandbox cannot mutate or leak secrets. |
| 23 Canvas, Artifacts, Artifact Governance | artifact/data/write outputs | canvas/artifact share/export/version packets | Stages 16, 22 | Stages 24, 31, 34 | Canvas is workspace, not authority. |
| 24 Agent, Apps, Connectors, Tasks, Scheduling | route candidates, access, app auth, Stage 13 route contracts | connector/API/app/agent packets | Stages 6, 12-13, 16, 18, 23 | Stages 25-26, 31, 34 | Live connector/API search owned here. |
| 25 Broadcast, Delivery, Reminders, Message Lifecycle | approved protected plan or draft | delivery/reminder lifecycle packets | Stages 12, 24 | Stage 26, 27, 34 | External send is protected. |
| 26 Business Process Actions, Link, Onboarding, Position, CAPREQ | access/authority/simulation, delivery workflow | process action packets | Stages 6, 12, 24-25 | Stage 34 | Mutations require protected gates. |
| 27 Record Mode And Meeting Recording Product | record packets, artifact ledger, voice processing policy | transcript/summary/action draft artifacts | Stages 4, 8, 12, 16, 22, 25-26 | Stage 34 | Record mode is not live chat. |
| 28 Image And Video Generation And Editing | creative route, artifact/policy gates | generated/edited image/video packets | Stages 3, 12, 16, 22-23 | Stages 31, 34 | Creative output is not evidence. |
| 29 Learning, Knowledge, Emotional Guidance, Adaptation | feedback/memory/persona/session packets | emotional guidance, prosody, learning artifacts | Stages 10, 15, 17, 21 | Stages 30-34 | Broad stage; emotion tone-only. |
| 30 Builder, Self-Heal, Release, Replay, Codex, Dev Lane | proof/replay, benchmark results, provider contracts | release evidence, promotion/rollback, leaderboard reports | Stages 2-3, 13-14, 29 | Stages 31, 33-34 | Broad stage; no silent provider drift. |
| 31 Privacy, Retention, Admin Policy, Health, Export, Audit | all consent/artifact/app/provider/memory lanes | retention/export/admin audit proof | Stages 3, 21, 23-24, 28-30 | Stages 33-34 | Health is display/projection unless authorized. |
| 32 Advanced Language Profiles And Certification | language/voice/write/TTS/model routes | language certification packs | Stages 8, 10, 15, 17, 30 | Stages 33-34 | Language cannot infer protected identity. |
| 33 Native And Runtime Product Parity Certification | clients, adapter, runtime, product lanes | parity report | Stages 18-20, 24, 30-32 | Stage 34 | Certifies parity where surfaces exist. |
| 34 Full System Certification Harness | all packets/proofs/benchmarks | final certification result | Stages 1-33 | future approved expansion | Blocks completion if any numeric lane lacks replayable result. |

## Parallelizable Slice Families

The following can be prepared in parallel after Stage 2 and Stage 3 contracts exist, as long as write scopes do not overlap:

- Stage 8 listening-lab corpora docs and Stage 10 meaning-repair corpora docs.
- Stage 13 URL/page reader foundation and Stage 13 source verification fixtures, after provider-off proof.
- Stage 16 presentation block schemas and Stage 17 TTS playback controls, after Stage 15 output packet shape is stable.
- Stage 19 Mac renderer and Stage 20 iPhone renderer, after Stage 18 transport packet shape is stable.
- Stage 21 memory trust UI docs and Stage 29 human-experience benchmark docs, after Stage 15/17 consumer contracts exist.

## Must-Not-Start-Early Slice Families

- Stage 13 live provider work must not start before Stage 3A provider-off proof and explicit live-provider allowance.
- Stage 24 connector writes must not start before Stage 12 protected closure.
- Stage 28 generated media provider calls must not start before Stage 3 provider gates and Stage 31 retention policy.
- Stage 30 provider/model promotion cannot start before Stage 3 contracts and relevant benchmark result envelopes.
- Stage 33/34 cannot certify product parity for lanes whose owning stages are not built.
