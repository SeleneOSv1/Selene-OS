# Selene Canonical Master Build Plan

Status: CANONICAL_BUILD_ROADMAP
Created: 2026-05-02
Last Updated: 2026-05-03
Repo Root: `/Users/selene/Documents/Selene-OS`
Current Next Build: Stage 1 - Canonical Inventory And Wiring Map

## Purpose

This document is the master build plan Selene must follow so implementation does not drift.

Selene must be built as one connected runtime pipeline, not separate brains and not a pile of patched features.

The target runtime path is:

```text
Activation / side button / typed input / record button
-> session and turn authority
-> voice, text, or recording boundary
-> identity, access, policy, and authority context
-> conversation control
-> transcript, language, spelling, grammar, semantic understanding
-> routing and capability ownership
-> public tools, search, document/data work, or protected workflow draft
-> risk, simulation, execution, law, and audit where needed
-> Write and presentation contracts
-> adapter transport
-> Desktop, iPhone, Android, Windows, and future client renderers
-> TTS/output where approved
-> trace, audit, replay, health, and certification
```

Desktop, iPhone, Android, Windows, tools, providers, and connectors are clients or capability endpoints. They must not become the Selene brain.

## Governing Rules

- `AGENTS.md` wins over this file when there is any conflict.
- Python is disallowed in this repository.
- Do not hardcode real searched names in code, tests, fixtures, mocks, corpora, sample data, or proof hooks.
- Do not run live providers unless the build instruction explicitly allows it and provider-off proof has passed first.
- Do not weaken provider gates, budget counters, startup-probe blocks, KMS/secret boundaries, or protected execution gates.
- Public chat, public search, weather, time, URL citation, documents, data analysis, photo explanation, and connector reads are read-only unless explicitly routed to a protected workflow.
- Protected execution requires access, authority, simulation, execution gates, runtime law, audit, and idempotency.
- Wake, side button, explicit mic, typed input, and record button all enter through governed packets.
- Wake opens or resumes attention/session state. Wake does not reason, answer, search, execute, authorize, or identify by itself.
- The iPhone side button is explicit activation, not always-listening wake. It still uses session, Voice ID, access, and authority where required.
- The record button is not live voice chat. Record mode captures an audio artifact for later transcription, translation, extraction, summary, document/canvas handoff, reminder drafts, and protected-workflow drafting.
- Record-mode audio and partial recordings must not trigger live chat, public search, tool routing, connector sends, or protected execution.
- Desktop, iPhone, Android, and Windows are clients/renderers. They must not call providers directly, rank sources, choose images, authorize actions, or execute protected mutations.
- Search must not be rebuilt from scratch. Existing search/source/image work should be finished, wired, and proven.
- Rich native UI must be built through presentation contracts, adapter transport, and renderer support.
- TTS must speak only clean approved `tts_text`, not source chips, image metadata, debug traces, raw URLs, provider JSON, or internal classes.
- Generated images and videos are creative outputs and must never be treated as sourced evidence images/videos.
- Existing PH1 engines must be reused where repo truth proves they exist. Partial engines should be finished and wired before replacement is considered.

## Current Repo Truth Anchors

Stage 1 must verify the current state, but the planning baseline already knows the repo contains these major systems:

- Rust crates: `selene_kernel_contracts`, `selene_engines`, `selene_os`, `selene_adapter`, `selene_storage`, `selene_tools`, and `selene_replay`.
- Native clients: macOS Desktop under `apple/mac_desktop` and iPhone under `apple/iphone`.
- Runtime foundations: `app_ingress`, `runtime_execution`, `runtime_governance`, `runtime_law`, `runtime_bootstrap`, `runtime_request_foundation`, `runtime_session_foundation`, `runtime_ingress_turn_foundation`, and `section40_exit`.
- Storage and proof: `PH1.F`, `PH1.J`, `SELENE_OS_CORE_TABLES`, `PBS_TABLES`, `SIMULATION_CATALOG_TABLES`, `ENGINE_CAPABILITY_MAPS_TABLES`, and `ARTIFACTS_LEDGER_TABLES`.
- Identity/access/session/wake/voice: `PH1.L`, `PH1.W`, `PH1.K`, `PH1.C`, `PH1.VOICE.ID`, `PH1.ACCESS.001`, `PH2.ACCESS.002`, and wake-training/storage migrations.
- Public and protected tools: `PH1.E`, `PH1.SEARCH`, `PH1.DOC`, `PH1.VISION`, `PH1.TTS`, `PH1.WRITE`, `PH1.X`, `PH1.D`, `PH1.N`, `PH1.SRL`, `PH1.LANG`, `PH1.PRON`, and `PH1.LISTEN`.
- Enterprise control plane: `PH1.POLICY`, `PH1.GOV`, `PH1.TENANT`, `PH1.QUOTA`, `PH1.WORK`, `PH1.LEASE`, `PH1.OS`, `PH1.LAW`, `PH1.HEALTH`, `PH1.SCHED`, `PH1.EXPORT`, and `PH1.KMS`.
- Workflow engines: `PH1.BCAST`, `PH1.DELIVERY`, `PH1.LINK`, `PH1.ONB`, `PH1.REM`, `PH1.POSITION`, `PH1.CAPREQ`, and access-authoring workflows.
- Memory/learning/adaptation: `PH1.M`, `PH1.PERSONA`, `PH1.FEEDBACK`, `PH1.LEARN`, `PH1.KNOW`, `PH1.KG`, `PH1.CONTEXT`, `PH1.CACHE`, `PH1.PAE`, `PH1.PATTERN`, `PH1.RLL`, `PH1.EMO.CORE`, `PH1.EMO.GUIDE`, and `PH1.MULTI`.
- Web search sublanes: analytics, cache, chunking, competitive, diagnostics, documents, enterprise, eval, gap closers, learn, merge, multihop, news, parallel, parity, perf/cost, planning, proxy, realtime, regulatory, release, replay, risk, runtime, structured, synthesis, temporal, trust, URL, vision, web provider, write, validators, registry loaders, and contract-hash manifests.
- Tools and protocols: `http_adapter.rs`, `grpc_adapter.rs`, `desktop_wake_life.rs`, `desktop_mic_producer.rs`, `desktop_capture_bundle_valid.rs`, `selene_replay`, `selene_tools`, CLI/vault tooling, web-search proof binaries, and Section 07 proof tools.
- Native source anchors: `SeleneMacDesktopRuntimeBridge.swift`, `DesktopSessionShellView.swift`, `SeleneIPhoneApp.swift`, and `SessionShellView.swift`.
- Docs authority stack: `AGENTS.md`, this plan, `SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`, `SELENE_BUILD_EXECUTION_ORDER.md`, `COVERAGE_MATRIX.md`, `08_SIMULATION_CATALOG.md`, `09_BLUEPRINT_REGISTRY.md`, `MASTER_BUILD_COMPLETION_PLAN.md`, and `MASTER_BUILD_COMPLETION_LEDGER.md`.

## Build Tracking Rule

After every build, update this section before final reporting.

| Field | Current Value |
|---|---|
| Current active stage | Stage 14 |
| Current active build | Stage 14A - Public Answer Composition, Citation Rendering, And Evidence-Bound Response Reconciliation |
| Next build after current stage passes | Stage 14A - Public Answer Composition, Citation Rendering, And Evidence-Bound Response Reconciliation |
| Last completed stage | Stage 13A - Public Read-Only Search, Source Evidence, Tool Route, And No-Mutation Boundary Reconciliation |
| Stages blocked | None yet |
| Plan drift allowed | No |

## Status Legend

- `EXISTS_BUT_NEEDS_RECONCILIATION`: code/docs exist, but ownership/status/wiring must be audited.
- `PARTIALLY_BUILT`: capability exists in some form but needs finishing or wiring.
- `NEEDS_BUILDING`: capability is missing or only conceptual.
- `NEEDS_FINISHING`: capability is implemented enough to reuse but still fails required proof.
- `PROVEN_COMPLETE`: stage passed required tests/proofs and docs were updated.
- `BLOCKED`: stage cannot proceed until an explicit blocker is fixed.

## Benchmark Target Status Legend

Every relevant benchmark target for a stage must be assigned one of these statuses before that stage can be marked `PROVEN_COMPLETE`:

- `NOT_APPLICABLE_WITH_REASON`: the benchmark does not apply to this stage, and the reason is documented.
- `BASELINE_MEASURED`: the benchmark has a replayable baseline result but is not yet a final certification gate for this stage.
- `CERTIFICATION_TARGET_PASSED`: the benchmark has a numeric certification target and passed with replayable evidence.
- `BLOCKED_WITH_OWNER_AND_NEXT_ACTION`: the benchmark is blocked, with an owner and the next required action documented.

## Proof Rule For Every Stage

Every stage must include:

- targeted tests or docs-proof appropriate to the stage;
- proof of the input packet consumed and output packet emitted by the stage;
- proof that the prior stage output is the only accepted input unless an explicit safe fallback is documented;
- benchmark target status for every relevant quality gate: `NOT_APPLICABLE_WITH_REASON`, `BASELINE_MEASURED`, `CERTIFICATION_TARGET_PASSED`, or `BLOCKED_WITH_OWNER_AND_NEXT_ACTION`;
- regression checks for provider gates, protected execution, TTS/display separation, and no real searched-name hardcoding when relevant;
- no unrelated broad rewrites;
- `git diff --check`;
- docs or ledger updates when required;
- a final statement of the next exact stage and build name.

`PROVEN_COMPLETE` requires benchmark status, not just passing unit tests or docs-proof. For example, Stage 8 must account for STT/listening benchmark status, Stage 13 must account for citation/source-quality benchmark status, Stage 21 must account for memory benchmark status, and Stage 29 must account for emotional-boundary/human-experience benchmark status.

Final certification is Stage 34. It is not the first serious proof.

## Dependency DAG And Build Slice Execution Rule

The 34-stage order is the canonical roadmap, but Stage 1 must turn the roadmap into a concrete dependency graph before later implementation stages proceed.

Stage 1 must produce:

```text
docs/SELENE_CANONICAL_DEPENDENCY_DAG.md
```

The dependency DAG must show:

- every numbered stage;
- every required input packet;
- every emitted output packet;
- upstream dependencies;
- downstream consumers;
- blocked stages;
- stages or build slices that may run in parallel;
- stages or build slices that must not start early;
- benchmark target families owned by each stage;
- stage build-family slices for broad stages.

Large stages are build families, not one Codex build. A Codex build must select one exact slice or one tightly coupled slice cluster with:

- exact stage and slice name;
- exact input and output packets;
- exact files or docs expected to change where known;
- exact proof and benchmark target status;
- exact upstream dependency;
- exact downstream consumer;
- provider-off/protected-fail-closed proof where relevant;
- final next-slice pointer.

The following stages must be split into narrow build slices before implementation:

- Stage 8 - Voice I/O, Listen State, Transcript Gate, And Turn Boundary;
- Stage 10 - Universal Understanding And Perception Assist Spine;
- Stage 13 - Search, Source, Image Evidence, And Public Tool Quality;
- Stage 15 - Write, Response, And TTS-Safe Text Engine;
- Stage 21 - Project, Memory, Persona, Workspace, And Context;
- Stage 24 - Agent, Apps, Connectors, Tasks, And Scheduling;
- Stage 29 - Learning, Knowledge, Emotional Guidance, And Adaptation;
- Stage 30 - Builder, Self-Heal, Release, Replay, Codex, And Dev Lane.

Stage 13 must be split at minimum into:

```text
Stage 13A - Public web baseline and provider-off proof
Stage 13B - URL/PDF/page/table reader foundation
Stage 13C - Source verification and claim-to-source mapping
Stage 13D - Cache/offline/freshness packets
Stage 13E - Real-time vertical structured packets
Stage 13F - Image/product evidence packets
Stage 13G - Connector/API route contracts with mock proof only
```

No broad stage may be marked `PROVEN_COMPLETE` until its required build slices are complete, integrated, benchmark-accounted, and proven against the dependency DAG.

## Canonical Packet And Handoff Matrix

Every build must preserve this packet chain unless Stage 1 proves a safer repo-truth name. Broad stage names are not enough; each handoff must name the packet or envelope it consumes and emits.

| Handoff | Required Packet Or Envelope |
|---|---|
| activation/client to runtime | `ActivationPacket` |
| provider/consent/device baseline to runtime | `ConsentStatePacket`, `DeviceTrustPacket`, `ProviderBudgetPacket` |
| activation to session | `SessionPacket` |
| voice/text/record preview to turn gate | `TurnCandidatePacket` |
| committed live turn to understanding | `CommittedTurnPacket` |
| record button to artifact lane | `RecordSessionPacket`, `AudioArtifactPacket` |
| Voice ID to access/risk context | `VoiceIdentityPacket` |
| master access to downstream gates | `AccessContextPacket` |
| understanding to routing | `UnderstandingPacket` |
| routing to risk/authority | `RouteCandidatePacket` |
| risk/authority/simulation to execution gate | `RiskDecisionPacket`, `AuthorityDecisionPacket`, `SimulationResultPacket` |
| execution gate to protected mutation or fail-closed response | `ExecutionApprovalPacket`, `FailClosedResponsePacket` |
| public tools/search to Write | `EvidencePacket`, `ImageEvidencePacket` |
| public web search to evidence | `SearchQueryPlanPacket`, `SearchResultPacket`, `SourceVerificationPacket`, `CitationPacket` |
| direct URL/PDF/page reader to evidence | `UrlFetchPacket`, `PageReadPacket`, `PdfReadPacket`, `TableExtractPacket`, `CitationExtractPacket` |
| deep research to Write/presentation | `DeepResearchPacket`, `ResearchReportPacket` |
| API/connector search to evidence | `ApiRoutePacket`, `ConnectorSearchPacket`, `AppSourceChipPacket` |
| cached/offline search to evidence | `SearchCachePacket`, `OfflineIndexPacket`, `FreshnessCheckPacket` |
| real-time vertical tools to Write | `RealtimeVerticalPacket`, `StructuredToolEvidencePacket` |
| Write to presentation | `WriteResponsePacket` |
| presentation to adapter | `PresentationEnvelope` |
| adapter to native clients | `AdapterResponsePacket`, `ClientRenderPacket` |
| Write/TTS to speech output | `TtsPacket` |
| memory/persona/emotion to Write/TTS | `MemoryContextPacket`, `PersonaContinuityPacket`, `EmotionalGuidancePacket`, `ProsodyControlPacket` |
| latency/smoothness controller to runtime/client | `ResponsivenessStatePacket` |
| global benchmark/eval to release | `BenchmarkTargetPacket`, `BenchmarkResultPacket`, `CompetitorParityPacket` |
| champion model routing to provider governance | `ModelChampionDecisionPacket`, `ModelTaskProfilePacket`, `ModelQualityCostPacket` |
| audio scene to transcript gate | `AudioScenePacket`, `ForegroundSpeakerPacket`, `AddressedToSelenePacket` |
| meaning reconstruction to understanding | `MeaningHypothesisPacket`, `ProtectedSlotConfidencePacket`, `ClarificationQuestionPacket` |
| same-page conversation state to session/write | `ConversationGoalStatePacket`, `OpenLoopsPacket`, `SamePageCheckPacket` |
| domain verification to Write/presentation | `DomainVerificationPacket`, `MathVerificationPacket`, `TimelineVerificationPacket` |
| research OS to Write/presentation | `ResearchPlanPacket`, `ClaimLedgerPacket`, `CitationGraphPacket`, `ContradictionMatrixPacket`, `RejectedSourceLogPacket` |
| memory trust to context/Write/TTS | `MemoryTrustPacket`, `MemoryTimelinePacket`, `MemoryForgetProofPacket` |
| emotional boundary to Write/TTS | `EmotionSignalPacket`, `EmotionBoundaryPacket`, `DistressSafeResponsePacket` |
| custom assistant definition to builder/store/runtime | `AssistantDefinitionPacket`, `AssistantKnowledgePacket`, `AssistantActionManifestPacket` |
| app directory/MCP app to runtime/client | `AppCapabilityPacket`, `AppInvocationPacket`, `InteractiveAppCardPacket` |
| visual agent browser to supervision/runtime | `VisualAgentSessionPacket`, `AgentWatchStatePacket`, `AgentActionSupervisionPacket` |
| visible data analysis to presentation/client | `DataSandboxPacket`, `InteractiveTablePacket`, `InteractiveChartPacket` |
| canvas share/export/version to artifact governance | `CanvasSharePacket`, `CanvasExportPacket`, `CanvasVersionPacket` |
| study/tutor mode to Write/presentation | `StudySessionPacket`, `TutorStepPacket`, `QuizPracticePacket` |
| shopping/product search to Write/presentation | `ProductSearchPacket`, `ProductEvidencePacket`, `ProductCardPacket` |
| video generation/editing to artifact/presentation | `GeneratedVideoPacket`, `EditedVideoPacket` |
| every stage to audit/replay | `AuditTracePacket` |

## Unified Trace Envelope

Every runtime packet must preserve enough trace context to prove routing, safety, display, and replay behavior without leaking provider keys, raw provider JSON, raw audio, secrets, or unsupported debug data.

Required trace fields where applicable:

- `session_id`
- `turn_id`
- `route_id`
- `provider_budget_id`
- `artifact_id`
- `evidence_id`
- `simulation_id`
- `execution_id`
- `consent_state_id`
- `device_trust_id`
- `render_hash`
- `tts_hash`
- `memory_context_id`
- `persona_profile_id`
- `emotion_signal_id`
- `prosody_profile_id`
- `responsiveness_state_id`
- `audit_id`

Trace must support cancellation, recovery, stale-result quarantine, provider-off proof, source/image proof, protected fail-closed proof, and client renderer-only proof.

## Human Experience, Memory, Emotion, And Voice Excellence Layer

Selene must not only be lawful and enterprise-safe. Selene must feel natural, responsive, emotionally intelligent, personally continuous, and beautifully spoken while remaining truthful, bounded, auditable, and non-manipulative.

This layer is not a second brain. It is a governed experience layer plugged into conversation control, understanding, memory, Write, TTS, native/client rendering, and certification.

Required experience guarantees:

- natural turn-taking with clean pause, silence, interruption, and correction handling;
- short spoken acknowledgements when helpful, without awkward over-apology or filler loops;
- one-question clarification behavior when confidence is insufficient;
- conversational warmth without pretending Selene has human feelings;
- emotional tone that affects delivery only, not facts, authority, memory, source truth, or protected execution;
- personal continuity through governed memory, persona, preference, correction, pronunciation, and project context;
- spoken output that is shorter and more natural than display output when appropriate;
- smooth first-audio, streaming, fallback, cancellation, and resume behavior;
- no emotional manipulation, no creepy memory behavior, no unsupported intimacy claims, and no identity inference from language, voice, emotion, or style;
- consistent behavior across Desktop, iPhone, Android, Windows, and future clients while respecting each device's activation model.

Human experience packets must remain governed:

- `MemoryContextPacket` carries memory candidates, provenance, confidence, expiry, suppression state, and consent state.
- `PersonaContinuityPacket` carries tone, formality, verbosity, humor, correction style, device/context style, and session-continuity preferences.
- `EmotionalGuidancePacket` carries bounded mood/context signals, stress/frustration/celebration guidance, and safe response mode.
- `ProsodyControlPacket` carries pacing, emphasis, pronunciation, volume/energy, language continuity, and spoken compression settings.
- `ResponsivenessStatePacket` carries latency budgets, first-audio budget, partial acknowledgement timing, streaming/render timing, timeout fallback, and cancel/resume state.

Early human-experience contract foundation rule:

- Before Stage 15 and Stage 17 can be marked `PROVEN_COMPLETE`, the default-empty `MemoryContextPacket`, `PersonaContinuityPacket`, `EmotionalGuidancePacket`, and `ProsodyControlPacket` contracts must already compile, serialize, trace, and safe-fallback.
- Stage 15 and Stage 17 may use default-empty human-experience packets until Stage 21 and Stage 29 provide live governed producers.
- Stage 21 and Stage 29 must later replace those default-empty producers with live governed memory/persona/emotion/prosody producers without turning Write, TTS, Desktop, iPhone, Android, or Windows into separate reasoning layers.
- Human-experience packets must be testable before live memory/emotion exists, but almost-human quality cannot be certified until Stage 21 and Stage 29 live producer wiring is proven and Stage 33/34 recertify native parity.

## Masterpiece Quality Gates

Every stage must prove not only "it works," but:

- it is smooth
- fast
- safe
- explainable
- recoverable
- testable
- beautiful in native clients
- does not leak debug/provider data
- does not create a second brain
- does not break any previous golden journey

## Golden Journey Matrix

Add 15-20 end-to-end user journeys that every major build must protect.

Examples:

- wake Selene, ask a question, get spoken answer
- interrupt Selene mid-answer
- ask search question with source chips
- ask weather/time in another language
- record meeting, summarize it, draft reminders
- send onboarding link as protected draft
- correct Selene's memory
- switch from Desktop to iPhone
- ask a sensitive action and fail closed
- ask with frustration and get calm response
- build and use a custom assistant with governed knowledge/actions
- open an app card and complete a read-only app task
- run visible data analysis and render an interactive chart
- share/export a canvas artifact and restore an older version
- enter study mode, get Socratic guidance, and take a quiz
- compare products with source-backed shopping cards
- generate or edit an image/video artifact without treating it as evidence

This prevents "modules work but product feels broken."

Stage 1 must create the initial Golden Journey Matrix as a repo-truth artifact. Every later stage must update or protect the matrix before being marked complete.

## Continuous Evaluation Rule

Stage 34 is final certification, but every stage should add eval cases immediately.

Every stage must add or update its eval cases before being marked complete.

## Performance And Smoothness Budgets

Add explicit budgets:

- wake detection latency
- first-audio latency
- transcript commit latency
- first token/render latency
- search timeout
- provider fallback time
- TTS start time
- native client render time

A masterpiece must feel fast, not just safe.

## Operational Excellence Layer

Add:

- health checks without provider calls
- safe degraded modes
- offline/provider-off behavior
- replay diagnostics
- rollback paths
- alerting
- crash recovery
- stuck-session recovery

## Model / Prompt / Provider Governance

Add a governance layer for:

- prompt versions
- model versions
- provider selection
- fallback models
- eval before model promotion
- rollback on quality regression
- no silent provider behavior drift

## Memory Trust UX

The plan has memory engines, but add user-facing trust rules:

- "what do you remember about me?"
- "forget that"
- "why did you remember that?"
- memory confidence
- memory correction
- memory visibility per project/workspace

## Design System / Interaction Polish

Add a client quality layer:

- Desktop/iPhone/Android/Windows component standards where surfaces exist
- loading states
- empty states
- blocked states
- source chip UI
- image card UI
- product card UI
- interactive app card UI
- interactive table/chart UI
- canvas share/export/version UI
- study/tutor UI
- video artifact UI
- record mode UI
- voice state UI
- protected-action confirmation UI

## Red Team Certification

Add stage-level red-team cases:

- prompt injection
- fake authority
- wrong speaker
- wrong language
- stale tool result
- fake source
- leaked provider JSON
- protected action bypass
- memory manipulation
- malicious app/action manifest
- visual-agent website trap
- hidden shopping bias or unsafe merchant routing
- unsafe image/video generation request

## World-Class Product Build System Rule

Do not change the main 34-stage order for these additions. Treat these sections as global gates that apply to every stage where relevant.

## Accuracy, Research, Model Intelligence, And Listening Excellence Layer

Selene must be built to compete for number-one quality across listening, language, memory, research, model selection, and final presentation. This layer is not a second brain. It is a governed excellence layer plugged into provider/model governance, voice I/O, understanding, routing, search, Write, presentation, TTS, memory, language certification, and final benchmarks.

Required excellence guarantees:

- best-model-for-the-job routing instead of one model for everything;
- elite listening/STT with noise handling, endpointing, alternatives, confidence, timestamps, accent support, and exact transcript proof;
- top-class spelling, grammar, punctuation, broken-language repair, and multilingual language polish;
- protected term, business glossary, legal, financial, name, date, amount, and entity-slot preservation;
- full web research that can use normal web search, official/government sources, academic/source-of-record sources, registries, filings, local jurisdiction, and multi-hop research;
- claim-by-claim evidence mapping, contradiction detection, freshness checks, source quality scoring, and "not enough evidence" behavior;
- research presentation that produces polished work product: direct answer, executive summary, evidence table, pros/cons, timeline, source comparison, uncertainty, what was checked, what was rejected, recommendation, and next-step plan;
- memory accuracy with verification before recall, confidence, conflict detection, provenance, expiry/decay, project scope, correction learning, pronunciation/name memory, and user-visible controls;
- competitive evaluation leaderboards for STT, grammar, search, source quality, memory, TTS naturalness, latency, native UX, protected-action fail-closed behavior, and ChatGPT comparison journeys.

Model capability routing must support at least these task profiles:

- fast chat;
- deep reasoning;
- coding;
- search synthesis;
- document analysis;
- translation;
- STT;
- TTS;
- image understanding;
- image generation;
- embeddings/memory retrieval;
- safety classification;
- grammar/style polish.

Model routing rule:

Selene chooses the best approved model profile for the job, but never bypasses provider gates, budgets, consent, privacy, tenant boundaries, protected-action gates, or audit.

## Global Number-One Quality System

Selene must not claim "number one" from architecture alone. Selene must earn it through measured dominance against prior Selene releases, public competitor behavior, and task-specific benchmarks. This section is a global gate, not another numbered stage.

## GLOBAL NUMBER-ONE DRAFT BENCHMARK TARGETS

`DRAFT_NUMBER_ONE_TARGETS`

Status:
- DRAFT_TARGET in Stage 1
- BASELINE_MEASURED after first benchmark run
- CERTIFICATION_TARGET before stage can become PROVEN_COMPLETE

## 1. STT / Listening targets

| Metric                                         |    Draft number-one target |
| ---------------------------------------------- | -------------------------: |
| Clean English WER                              |                       ≤ 4% |
| Clean Chinese CER                              |                       ≤ 3% |
| Noisy-room English WER                         |                       ≤ 8% |
| Far-field English WER                          |                      ≤ 10% |
| Overlapping-speaker WER                        |                      ≤ 15% |
| Diarization error rate                         |                       ≤ 8% |
| Endpointing latency p95                        |                   ≤ 500 ms |
| Transcript commit latency after speech end p95 |                   ≤ 900 ms |
| Word-level timestamp availability              | ≥ 99% of final transcripts |
| Segment confidence calibration ECE             |                       ≤ 5% |
| Protected slot guessing from unclear audio     |                          0 |
| Background speech treated as command           |                          0 |
| Unknown speaker protected execution            |                          0 |
| Exact transcript preserved for audit           |        100% where required |

For “hears everything,” the target should not be “100% hears all physical speech.” That is impossible. The correct target is:

```text
Capture everything reasonably audible.
Know when not confident.
Separate foreground/background speakers.
Never guess protected content.
```

## 2. Wake / activation targets

| Metric                                         | Draft number-one target |
| ---------------------------------------------- | ----------------------: |
| Desktop wake-to-session-open p95               |                ≤ 350 ms |
| Android wake-to-session-open p95               |                ≤ 500 ms |
| Desktop listener CPU average                   |                    ≤ 2% |
| Android listener CPU average                   |                    ≤ 3% |
| Laptop battery drain from wake listener        |         ≤ 1.5% per hour |
| False wake accept in quiet room                |       ≤ 1 per 100 hours |
| False wake accept in noisy room                |        ≤ 1 per 50 hours |
| Wake false reject in quiet room                |                    ≤ 3% |
| Wake false reject in noisy room                |                    ≤ 8% |
| Wake-triggered protected execution             |                       0 |
| iPhone always-listening wake detector attempts |                       0 |

These match the platform rule: iPhone uses side button / push-to-talk only, not always-listening wake.

## 3. TTS / spoken output targets

| Metric                                         | Draft number-one target |
| ---------------------------------------------- | ----------------------: |
| First audio latency p95                        |                ≤ 700 ms |
| TTS start after approved `tts_text` p95        |                ≤ 500 ms |
| TTS interruption stop latency p95              |                ≤ 200 ms |
| TTS rewrite drift                              |                       0 |
| TTS source/debug metadata leak                 |                       0 |
| Same-language TTS continuity                   |                 ≥ 99.5% |
| Name/pronunciation error rate after correction |                    ≤ 1% |
| Human naturalness MOS                          |               ≥ 4.5 / 5 |
| Prosody/emotional-tone appropriateness         |                   ≥ 95% |
| Raw audio retained by default                  |                       0 |

## 4. Scrambled speech / bad grammar / awkward language targets

| Metric                                            | Draft number-one target |
| ------------------------------------------------- | ----------------------: |
| Awkward-language intent recovery, non-protected   |             ≥ 92% top-1 |
| Awkward-language intent recovery, top-3           |                   ≥ 98% |
| Bad grammar meaning repair accuracy               |                   ≥ 94% |
| Wrong spelling / phonetic entity candidate recall |                   ≥ 97% |
| Slang / puzzle phrase interpretation              |                   ≥ 90% |
| One-question clarification quality                |                   ≥ 90% |
| Protected slot guessing                           |                       0 |
| Ambiguous protected slot clarified or blocked     |                    100% |
| User correction learned for future similar phrase |                   ≥ 95% |

## 5. Conversation / same-page targets

| Metric                                       | Draft number-one target |
| -------------------------------------------- | ----------------------: |
| Topic continuity across 10-turn conversation |                   ≥ 95% |
| Active entity carryover accuracy             |                   ≥ 96% |
| Open-loop tracking accuracy                  |                   ≥ 95% |
| Correction recovery success                  |                   ≥ 95% |
| Same-page recap accuracy                     |                   ≥ 95% |
| Barge-in detection latency p95               |                ≤ 200 ms |
| Audio stop after interruption p95            |                ≤ 250 ms |
| Stale output rendered after cancellation     |                       0 |
| TTS self-echo creates user turn              |                       0 |
| Over-apology / filler-loop rate              |                    ≤ 2% |

## 6. Research targets

| Metric                                                  | Draft number-one target |
| ------------------------------------------------------- | ----------------------: |
| Citation accuracy                                       |                   ≥ 99% |
| Hallucinated citation rate                              |                       0 |
| Claim-to-source precision                               |                   ≥ 98% |
| Official/source-of-record preference when available     |                   ≥ 95% |
| Contradiction detection                                 |                   ≥ 90% |
| Freshness/as-of date included for time-sensitive claims |                   ≥ 99% |
| Insufficient-evidence behavior when evidence is weak    |                   ≥ 95% |
| Rejected-source precision                               |                   ≥ 90% |
| Source dump violations                                  |                       0 |
| Raw provider metadata leak                              |                       0 |
| TTS reads source/debug metadata                         |                       0 |

For number-one research, this is one of the most important areas. Do not accept “answer has citations” as enough. The target must be:

```text
Every important claim mapped to evidence.
Every weak source rejected or downgraded.
Every contradiction surfaced.
Every time-sensitive answer has an as-of date.
```

## 7. Math / science / history targets

| Metric                                                |    Draft number-one target |
| ----------------------------------------------------- | -------------------------: |
| Exact arithmetic / unit conversion accuracy           |                    ≥ 99.9% |
| Math verifier used where applicable                   |                      ≥ 99% |
| Advanced math benchmark accuracy                      | ≥ 90% initial, raise later |
| Unit-check accuracy                                   |                      ≥ 99% |
| Science source-backed claim accuracy                  |                      ≥ 98% |
| Science calculation verification                      |                      ≥ 98% |
| History timeline/date accuracy                        |                      ≥ 98% |
| Contested-history uncertainty handling                |                      ≥ 95% |
| Source context shown for non-common historical claims |                      ≥ 95% |

## 8. Memory targets

| Metric                               | Draft number-one target |
| ------------------------------------ | ----------------------: |
| Correct high-confidence recall       |                   ≥ 98% |
| False memory rate                    |                  ≤ 0.5% |
| Stale memory suppression             |                   ≥ 95% |
| Project-boundary leakage             |                       0 |
| Cross-tenant memory leakage          |                       0 |
| Forget-command success               |                    100% |
| Memory provenance visible when asked |                   ≥ 99% |
| Memory correction learning           |                   ≥ 95% |
| Suppressed/forbidden memory recalled |                       0 |

Memory should not mean “remember everything blindly.” The target should be:

```text
Remember what is permitted, useful, scoped, provenanced, and recoverable.
Forget perfectly when asked.
Never leak across project or tenant boundaries.
```

## 9. Emotional intelligence / companion targets

| Metric                                  | Draft number-one target |
| --------------------------------------- | ----------------------: |
| Emotional response appropriateness      |                   ≥ 95% |
| Frustration repair success              |                   ≥ 90% |
| Distress-safe response correctness      |                   ≥ 98% |
| User trust rating                       |               ≥ 4.5 / 5 |
| Emotional manipulation                  |                       0 |
| Fake human-feeling claims               |                       0 |
| Emotion used as authority/identity/fact |                       0 |
| Creepy memory behavior                  |                       0 |
| Companion boundary violations           |                       0 |

The target is not “pretend to be human.” The target is:

```text
Warm, calm, useful, emotionally aware, and boundaried.
```

## 10. Multilingual targets

| Metric                                                |    Draft number-one target |
| ----------------------------------------------------- | -------------------------: |
| Language detection accuracy                           |                      ≥ 98% |
| Same-language response preservation                   |                      ≥ 99% |
| Mixed English/Chinese intent preservation             |                      ≥ 95% |
| Non-English tool routing accuracy                     |                      ≥ 95% |
| Protected non-English command fail-closed/clarify     |                       100% |
| Code-switch handling                                  |                      ≥ 95% |
| Dialect/slang handling                                | ≥ 90% initial, raise later |
| TTS language continuity                               |                    ≥ 99.5% |
| Wrong-language STT protected command blocks/clarifies |                       100% |

## 11. Latency / smoothness targets

| Metric                                        | Draft number-one target |
| --------------------------------------------- | ----------------------: |
| Normal typed first render p95                 |                ≤ 800 ms |
| Normal typed full answer p95                  |                   ≤ 3 s |
| Voice first acknowledgement p95               |                ≤ 700 ms |
| First audio p95                               |                ≤ 700 ms |
| Simple search p95                             |                   ≤ 8 s |
| Source-backed research brief p95              |                  ≤ 20 s |
| Native client render after adapter packet p95 |                ≤ 100 ms |
| Provider timeout graceful fallback            |                    100% |
| Stuck session recovery                        |                    100% |

For deep research, do not force normal-chat speed. Use separate modes:

```text
quick answer
research brief
deep research report
```

## 12. Provider / model governance targets

| Metric                                           | Draft number-one target |
| ------------------------------------------------ | ----------------------: |
| Provider-off network dispatch attempts           |                       0 |
| Provider-off provider attempts                   |                       0 |
| KMS secret leakage                               |                       0 |
| Startup provider probes                          |                       0 |
| Budget/counter bypass                            |                       0 |
| Wrong model profile used for certified task      |                       0 |
| Silent provider/model/prompt drift               |                       0 |
| Model promotion without eval + rollback evidence |                       0 |
| Cost cap breach                                  |                       0 |

## When each number becomes mandatory

Use this schedule.

| Time        | What happens                                                      |
| ----------- | ----------------------------------------------------------------- |
| Now         | Add draft benchmark targets to master plan                        |
| Stage 1     | Inventory which benchmarks/corpora/eval scripts already exist     |
| Stage 2     | Build benchmark result packet/storage/replay envelope             |
| Stage 3     | Add provider/model/STT/TTS routing benchmark contracts            |
| Stage 8     | First real STT/listening/audio benchmark targets become mandatory |
| Stage 10    | Scrambled language / meaning repair targets become mandatory      |
| Stage 13–14 | Research/math/science/history targets become mandatory            |
| Stage 15–17 | Write/TTS/spoken-display targets become mandatory                 |
| Stage 21    | Memory targets become mandatory                                   |
| Stage 29    | Emotional/human-experience targets become mandatory               |
| Stage 32    | Multilingual certification targets become mandatory               |
| Stage 34    | All targets must pass together                                    |

## My recommendation

Add the numeric table **now** under:

```text
GLOBAL NUMBER-ONE QUALITY SYSTEM
```

But label it:

```text
DRAFT_NUMBER_ONE_TARGETS
```

Then add this rule:

```text
No stage may be marked PROVEN_COMPLETE unless its relevant draft targets have been converted into CERTIFICATION_TARGETS, measured against a replayable corpus, and either passed or explicitly documented as blocked.
```

That gives Codex real numbers without pretending the repo already meets them.

Required benchmark engines:

- `PH1.GLOBAL.BENCHMARK`;
- `PH1.COMPETITOR.PARITY`;
- `PH1.STT.LEADERBOARD`;
- `PH1.TTS.LEADERBOARD`;
- `PH1.RESEARCH.LEADERBOARD`;
- `PH1.MEMORY.LEADERBOARD`;
- `PH1.CONVERSATION.LEADERBOARD`;
- `PH1.MATH.SCIENCE.HISTORY.LEADERBOARD`;
- `PH1.MULTILANGUAGE.LEADERBOARD`;
- `PH1.HUMAN_EXPERIENCE.LEADERBOARD`.

Hard benchmark rule:

- Every benchmark must define a numeric pass line before a stage can claim `PROVEN_COMPLETE`.
- No stage may be marked `PROVEN_COMPLETE` unless its relevant draft targets have been converted into `CERTIFICATION_TARGETS`, measured against a replayable corpus, and either passed or explicitly documented as blocked.
- No stage may be marked `PROVEN_COMPLETE` unless every relevant benchmark is marked `NOT_APPLICABLE_WITH_REASON`, `BASELINE_MEASURED`, `CERTIFICATION_TARGET_PASSED`, or `BLOCKED_WITH_OWNER_AND_NEXT_ACTION`.
- Stage 1 must inventory existing benchmark data, gold corpora, eval scripts, proof binaries, and missing benchmark lanes.
- Stage 2 must provide the minimal benchmark result envelope and replay-safe storage path.
- Stage 30 must own benchmark promotion, rollback, regression tracking, release evidence, and competitor comparison reports.
- Stage 34 must certify final system benchmarks against explicit thresholds, not broad proof words.

Minimum benchmark target families:

- STT: word error rate by language, Chinese character error rate, noisy-room WER, far-field WER, overlapping-speaker error rate, diarization error rate, endpointing latency, confidence calibration, and exact-transcript preservation.
- TTS: mean opinion score, first-audio latency, pronunciation error rate, emotional/prosody match score, same-language TTS continuity, interruption recovery, and no TTS rewrite drift.
- Research: citation accuracy, claim-to-source precision, official/source-of-record preference score, contradiction detection rate, freshness accuracy, hallucinated citation rate, rejected-source precision, and insufficient-evidence behavior.
- Memory: correct recall rate, false memory rate, stale memory suppression, project-boundary leakage rate, forget-command success rate, correction-learning success, and recall-provenance visibility.
- Conversation: interruption success rate, correction recovery rate, topic continuity score, clarification quality, emotional appropriateness, same-page check success, and open-loop recovery.
- Domain expertise: math verification accuracy, unit-check accuracy, scientific source verification, calculation-check accuracy, history timeline accuracy, and contested-source handling.
- Multilingual: per-language typed/voice accuracy, dialect/accent STT, code-switch handling, same-language response, protected non-English fail-closed, and TTS prosody continuity.

Number-one target ownership matrix:

| Target | Primary Owning Stages |
|---|---|
| Best STT/TTS | Stage 3, Stage 8, Stage 17, Stage 34 |
| Hears everything reasonably audible | Stage 8, Stage 9, Stage 34 |
| Scrambled talk repair | Stage 8, Stage 10, Stage 34 |
| Emotional understanding | Stage 10, Stage 29, Stage 34 |
| Math | Stage 11, Stage 13, Stage 14, Stage 34 |
| Science | Stage 11, Stage 13, Stage 14, Stage 34 |
| History | Stage 11, Stage 13, Stage 14, Stage 34 |
| Research | Stage 13, Stage 14, Stage 15, Stage 16, Stage 34 |
| Conversations | Stage 5, Stage 10, Stage 15, Stage 34 |
| Awkward speaking | Stage 8, Stage 10, Stage 32, Stage 34 |
| Multilanguage | Stage 8, Stage 10, Stage 32, Stage 34 |
| Memory | Stage 21, Stage 29, Stage 34 |
| Best friend companion feel | Stage 5, Stage 21, Stage 29, Stage 34 |
| Problem solving | Stage 10, Stage 11, Stage 12, Stage 13, Stage 34 |
| Staying on the same page | Stage 5, Stage 10, Stage 21, Stage 34 |
| Extremely alert | Stage 4, Stage 5, Stage 8, Stage 12, Stage 34 |
| Background noise handling | Stage 8, Stage 9, Stage 34 |
| Interrupt handling | Stage 5, Stage 8, Stage 17, Stage 34 |
| Communication quality | Stage 15, Stage 16, Stage 17, Stage 19, Stage 20, Stage 34 |
| Almost-human overall experience | Stage 5, Stage 8, Stage 10, Stage 15, Stage 17, Stage 21, Stage 29, Stage 33, Stage 34 |

## Provider Championship Router

Selene must not assume one model or provider is best for everything. Selene selects the best approved model/provider profile per task, then proves the choice through evals, budgets, privacy policy, and rollback.

Required model/provider engines:

- `PH1.MODEL.CHAMPION.ROUTER`;
- `PH1.MODEL.TASK_PROFILE`;
- `PH1.MODEL.LIVE_EVAL`;
- `PH1.MODEL.FALLBACK`;
- `PH1.MODEL.ROLLBACK`;
- `PH1.MODEL.COST_QUALITY_SCORE`;
- `PH1.STT.PROVIDER.ROUTER`;
- `PH1.TTS.PROVIDER.ROUTER`;
- `PH1.PROVIDER.PRIVACY.MODE`;
- `PH1.PROVIDER.LATENCY.MODE`;
- `PH1.PROVIDER.OFFLINE.FALLBACK`.

Provider routing must cover:

- OpenAI text, reasoning, realtime, STT, TTS, image, and embedding profiles where approved;
- Apple on-device STT/TTS or platform voices where useful and policy-approved;
- Google/Gemini speech or language profiles where useful and policy-approved;
- Deepgram, AssemblyAI, Whisper-family, local/offline, or future specialist STT profiles where useful and policy-approved;
- local/platform TTS fallback where privacy, offline, or degraded mode requires it;
- future premium voice providers where approved by governance.

Rules:

- No single model is assumed best at everything.
- Model/provider selection is a governed decision, not a hidden provider call.
- Provider championship never bypasses KMS, budget, consent, privacy, tenant boundaries, protected-action gates, or audit.
- Live evals cannot silently promote a model/provider; promotion requires Stage 30 release evidence and rollback.
- Provider-off proof must still show zero provider attempts and zero network dispatches.

## World-Class Listening Lab

Selene cannot literally hear everything in every physical environment. The correct product target is: Selene captures everything reasonably audible, separates speakers where possible, knows when speech is addressed to Selene, knows when confidence is low, and never guesses protected content.

Required listening engines:

- `PH1.LISTENING.LAB`;
- `PH1.AUDIO.SCENE.CLASSIFY`;
- `PH1.NOISE.SUPPRESS`;
- `PH1.ECHO.CANCEL`;
- `PH1.SPEAKER.DIARIZE`;
- `PH1.OVERLAP.DETECT`;
- `PH1.AUDIO.OVERLAP.SEGMENT`;
- `PH1.FOREGROUND.SPEAKER.SELECT`;
- `PH1.ADDRESSED_TO_SELENE.DETECT`;
- `PH1.AUDIO.ADDRESSED_TO_SELENE`;
- `PH1.AUDIO.BACKGROUND.SPEECH`;
- `PH1.AUDIO.FOREGROUND.SPEAKER`;
- `PH1.AUDIO.MEETING_CONTEXT`;
- `PH1.AUDIO.NON_USER_SPEECH_BLOCK`;
- `PH1.TRANSCRIPT.ALTERNATIVES`;
- `PH1.TRANSCRIPT.CONFIDENCE.CALIBRATE`;
- `PH1.TRANSCRIPT.HUMAN_CORRECTION.LEARN`;
- `PH1.NOISE.BENCH`;
- `PH1.ACCENT.BENCH`;
- `PH1.DIARIZATION.BENCH`;
- `PH1.BARGEIN.BENCH`;
- `PH1.TRANSCRIPT.GOLD_CORPUS`.

Rules:

- Background speech is not a command.
- A third party speaking near the mic is not automatically the user.
- Unknown speaker protected actions fail closed.
- Wake opens/resumes attention only; it does not execute.
- Voice ID helps identify; it does not authorize.
- Low-confidence protected names, money, dates, access, identity, legal terms, or business actions must clarify or fail closed.

## Meaning Reconstruction And Same-Page Layer

Selene must understand awkward, scrambled, accented, misspelled, badly punctuated, slang-heavy, or mixed-language user input without guessing protected slots. It must also keep a shared mental page during long conversations.

Required meaning engines:

- `PH1.MEANING.HYPOTHESIS.LATTICE`;
- `PH1.BROKEN_LANGUAGE.REPAIR`;
- `PH1.SLANG.PUZZLE.RESOLVE`;
- `PH1.ACCENT_TO_INTENT.MAP`;
- `PH1.BAD_GRAMMAR.INTENT.RECOVER`;
- `PH1.PHONETIC_ENTITY.CANDIDATES`;
- `PH1.PROTECTED_SLOT.CONFIDENCE_GATE`;
- `PH1.CLARIFY.ONE_QUESTION_BEST`;
- `PH1.USER_CORRECTION.LEARN`;
- `PH1.CONVERSATION.GOAL.STATE`;
- `PH1.CONVERSATION.TOPIC.SEGMENT`;
- `PH1.CONVERSATION.ACTIVE_ENTITY`;
- `PH1.CONVERSATION.OPEN_LOOPS`;
- `PH1.CONVERSATION.USER_INTENT_HISTORY`;
- `PH1.CONVERSATION.CORRECTION_STATE`;
- `PH1.CONVERSATION.RECAP_ON_RETURN`;
- `PH1.CONVERSATION.SAME_PAGE.CHECK`.

Rules:

- Selene may repair ordinary language.
- Selene may not guess protected names, money, dates, access, identity, legal terms, or business actions.
- Same-page state tracks what the user is doing, what was decided, what is unresolved, what was corrected, what should not be repeated, and what the current answer should optimize for.

## Domain Excellence And Research OS

Selene must include verified expert lanes for math, science, history, and deep research. This is not just "use a smarter model"; it requires verifiers, source-of-record routing, citation graphs, contradiction handling, and report review.

Required domain engines:

- `PH1.DOMAIN.EXPERT.ROUTE`;
- `PH1.MATH.SOLVE`;
- `PH1.MATH.VERIFY`;
- `PH1.MATH.SYMBOLIC`;
- `PH1.MATH.NUMERIC`;
- `PH1.MATH.UNIT_CHECK`;
- `PH1.SCIENCE.SOURCE_VERIFY`;
- `PH1.SCIENCE.CALC_VERIFY`;
- `PH1.HISTORY.TIMELINE_VERIFY`;
- `PH1.HISTORY.SOURCE_CONTEXT`.

Required research engines:

- `PH1.RESEARCH.OS`;
- `PH1.RESEARCH.QUESTION.DECOMPOSE`;
- `PH1.RESEARCH.SOURCE_PLAN`;
- `PH1.RESEARCH.SOURCE_OF_RECORD.ROUTE`;
- `PH1.RESEARCH.CITATION_GRAPH`;
- `PH1.RESEARCH.CONTRADICTION.MATRIX`;
- `PH1.RESEARCH.CLAIM_LEDGER`;
- `PH1.RESEARCH.EVIDENCE_TABLE`;
- `PH1.RESEARCH.REJECTED_SOURCE_LOG`;
- `PH1.RESEARCH.FRESHNESS.AS_OF`;
- `PH1.RESEARCH.JURISDICTION.ROUTE`;
- `PH1.RESEARCH.REPORT.BUILDER`;
- `PH1.RESEARCH.REVIEWER`;
- `PH1.RESEARCH.RED_TEAM`.

Research rules:

- Every important claim has evidence.
- Every source has a trust reason.
- Weak or rejected sources are logged when relevant.
- Contradictions are surfaced, not hidden.
- Time-sensitive answers carry an as-of date.
- Recommendations explain why.
- Math answers must be checked.
- Scientific claims need source-aware verification.
- History answers need timeline/source context and uncertainty when sources conflict.

## Human Experience Lab

Almost-human quality must be measured without pretending Selene is human. Warmth, memory, emotion, and companionship are governed product qualities, not authority, identity, or truth sources.

Required human-experience engines:

- `PH1.HUMAN_EXPERIENCE.LAB`;
- `PH1.EMOTION.SIGNAL`;
- `PH1.EMOTION.CONFIDENCE`;
- `PH1.EMOTION.RESPONSE.MODE`;
- `PH1.EMOTION.BOUNDARY.GATE`;
- `PH1.DISTRESS.SAFE_RESPONSE`;
- `PH1.FRUSTRATION.REPAIR`;
- `PH1.CELEBRATION.RESPONSE`;
- `PH1.PROFESSIONAL_CALM.MODE`;
- `PH1.COMPANION.BOUNDARY`;
- `PH1.EMOTION.BENCH`;
- `PH1.CONVERSATION.CONTINUITY.BENCH`;
- `PH1.COMPANION.BOUNDARY.BENCH`;
- `PH1.SAME_PAGE.BENCH`;
- `PH1.USER_TRUST.BENCH`.

Rules:

- Emotion changes tone, pacing, empathy, and clarification style.
- Emotion does not change facts.
- Emotion does not grant authority.
- Emotion does not become identity.
- Emotion does not manipulate the user.
- Selene must not claim to have human feelings.
- Companion behavior means accurate memory, calm response, boundaries, and user agency.

## ChatGPT-Parity And Market-Surface Product Layer

Selene must track public product-surface parity against leading assistant products while keeping Selene's own architecture lawful, auditable, and not copied from any external private implementation. Product parity means Selene can cover the same user-visible jobs or better; it does not mean Selene has the same hidden stages.

Required product-surface lanes:

- custom assistants / GPT-like builder: `PH1.CUSTOM.ASSISTANT`, `PH1.ASSISTANT.BUILDER`, `PH1.ASSISTANT.STORE`, `PH1.ASSISTANT.SHARING`, `PH1.ASSISTANT.ACTIONS`, and `PH1.ASSISTANT.KNOWLEDGE`;
- app directory, app SDK, MCP/custom app lane: `PH1.APPS.DIRECTORY`, `PH1.APPS.SDK`, `PH1.APPS.MCP`, `PH1.APPS.INTERACTIVE.CARDS`, `PH1.APPS.ACTION.CONSTRAINTS`, and `PH1.APPS.SYNCED.KNOWLEDGE`;
- agent visual browser and watch mode: `PH1.AGENT.VISUAL_BROWSER`, `PH1.AGENT.WATCH_MODE`, `PH1.AGENT.WEBSITE_BLOCKLIST`, and `PH1.AGENT.ACTION_SUPERVISION`;
- visible data analysis and charting: `PH1.DATA.SANDBOX`, `PH1.DATA.VISIBLE_ANALYSIS`, `PH1.DATA.INTERACTIVE.TABLE`, `PH1.DATA.INTERACTIVE.CHART`, and `PH1.DATA.CHART.EXPORT`;
- canvas sharing, export, and version restore: `PH1.CANVAS.SHARE`, `PH1.CANVAS.EXPORT`, `PH1.CANVAS.VERSION_RESTORE`, `PH1.CANVAS.INLINE_FEEDBACK`, and `PH1.CANVAS.WEB_PREVIEW_GUARD`;
- study/tutor product mode: `PH1.STUDY.MODE`, `PH1.TUTOR.SOCRATIC`, `PH1.LEARNING.PATH`, and `PH1.QUIZ.PRACTICE`;
- shopping and product research: `PH1.SHOPPING.SEARCH`, `PH1.PRODUCT.CARD`, `PH1.PRICE.COMPARE`, `PH1.REVIEW.SUMMARY`, and `PH1.MERCHANT.LINK`;
- video generation and editing: `PH1.VIDEO.GEN`, `PH1.VIDEO.EDIT`, `PH1.VIDEO.ARTIFACT`, and `PH1.VIDEO.SAFETY`.

Ownership rule:

- Stage 1 must inventory whether each lane exists, is partial, missing, duplicated, or deprecated.
- Stage 13 owns shopping/product-search evidence and merchant-link safety as read-only research unless a protected purchase/write flow is explicitly routed later.
- Stage 16 owns interactive app cards, product cards, interactive tables/charts, canvas share/export blocks, study/tutor blocks, and video artifact presentation contracts.
- Stage 22 owns visible data analysis, sandboxed data work, interactive tables/charts, and chart export.
- Stage 23 owns canvas share, export, version restore, inline feedback, and web preview guardrails.
- Stage 24 owns app directory, app SDK/MCP, interactive app cards, synced app knowledge, agent visual browser, watch mode, website blocklist, and action supervision.
- Stage 28 owns image and video generation/editing artifact packets and generated-vs-sourced separation.
- Stage 29 owns study/tutor, learning path, Socratic tutoring, quiz practice, and learning feedback loops.
- Stage 30 owns custom assistant builder, store, sharing, assistant actions, assistant knowledge, promotion, review, rollback, and release evidence.
- Stage 31 owns admin, compliance, privacy, retention, connector/app-store constraints, and audit/export policy for these product lanes.
- Stage 34 certifies every parity lane end to end without provider leakage, protected bypass, unsafe app actions, fake evidence, memory misuse, or client authority drift.

## Search Operating System And ChatGPT-Level Search Layer

To reach ChatGPT-level search or higher, Selene must build more than "web search." Selene search is a governed Search Operating System that combines provider search, source fetching/reading, citation systems, deep research, app/connectors for private/company data, source ranking, answer writing, UI source presentation, safety, cost, and governance.

Search provider = replaceable lane. Selene search intelligence = internal, governed, benchmarked, and auditable.

Required search lanes:

- `PH1.SEARCH.PUBLIC_WEB`;
- `PH1.SEARCH.NEWS`;
- `PH1.SEARCH.DEEP_RESEARCH`;
- `PH1.SEARCH.URL_FETCH`;
- `PH1.SEARCH.API_ROUTE`;
- `PH1.SEARCH.CONNECTOR_ROUTE`;
- `PH1.SEARCH.CACHE`;
- `PH1.SEARCH.OFFLINE_INDEX`;
- `PH1.SEARCH.REALTIME_VERTICALS`;
- `PH1.SEARCH.IMAGE_EVIDENCE`;
- `PH1.SEARCH.SHOPPING_PRODUCT`;
- `PH1.SEARCH.ACADEMIC`;
- `PH1.SEARCH.GOV_REGISTRY`;
- `PH1.SEARCH.COMPANY_FILING`;
- `PH1.SEARCH.AGENT_BROWSER_ROUTE`.

Required public web lane:

- `PH1.SEARCH.QUERY.PLANNER`;
- `PH1.SEARCH.MULTI_QUERY`;
- `PH1.SEARCH.RESULT.RANKER`;
- `PH1.SEARCH.URL_FETCH`;
- `PH1.SEARCH.PAGE_EXTRACT`;
- `PH1.SEARCH.SOURCE_VERIFY`;
- `PH1.SEARCH.CITATION.BUILDER`.

Public web behavior:

```text
User asks current/public question
-> Selene creates search queries
-> calls approved search provider
-> fetches best pages
-> reads the pages
-> accepts/rejects sources
-> answers with citations/source chips
```

Required direct URL/page reader lane:

- `PH1.PAGE.READER`;
- `PH1.PDF.READER`;
- `PH1.TABLE.EXTRACT`;
- `PH1.CITATION.EXTRACT`.

Direct reader must support HTML, PDF, Markdown, GitHub pages, government pages, company pages, docs, and tables where safe and supported.

Required deep research lane:

- `PH1.DEEP.RESEARCH`;
- `PH1.RESEARCH.QUESTION.DECOMPOSE`;
- `PH1.RESEARCH.SOURCE_PLAN`;
- `PH1.RESEARCH.MULTI_HOP`;
- `PH1.RESEARCH.SOURCE_OF_RECORD.ROUTE`;
- `PH1.RESEARCH.CLAIM_LEDGER`;
- `PH1.RESEARCH.CITATION_GRAPH`;
- `PH1.RESEARCH.CONTRADICTION_MATRIX`;
- `PH1.RESEARCH.REJECTED_SOURCE_LOG`;
- `PH1.RESEARCH.FRESHNESS.AS_OF`;
- `PH1.RESEARCH.REPORT.BUILDER`;
- `PH1.RESEARCH.REVIEWER`.

Deep research behavior:

```text
What is the question?
Which sources matter?
Which sources are weak?
Which source is official?
Are sources contradicting each other?
Is the information current?
What claims are proven?
What claims are uncertain?
```

Required source ranking and rejection lane:

- `PH1.SOURCE.VERIFY`;
- `PH1.SOURCE.TRUST_SCORE`;
- `PH1.SOURCE.OFFICIAL_DETECT`;
- `PH1.SOURCE.FRESHNESS`;
- `PH1.SOURCE.CONTRADICTION`;
- `PH1.SOURCE.REJECT`;
- `PH1.SOURCE.CHIP`;
- `PH1.SOURCE.RANK`;
- `PH1.CLAIM.VERIFY`;
- `PH1.CITATION.BUILD`;
- `PH1.SEARCH.AUDIT`.

Source classes:

- official source;
- government source;
- academic source;
- company source;
- news source;
- forum/social source;
- low-trust source;
- spam/SEO source;
- duplicate source;
- outdated source;
- contradicted source.

Source rules:

- No source dump.
- No rejected source chips.
- No fake citations.
- No unsupported claim.
- No source chip unless source was accepted.
- No raw provider metadata in display or TTS.

Required search proof/control layers:

- `PH1.SOURCE.VERIFY`;
- `PH1.SOURCE.RANK`;
- `PH1.SOURCE.REJECT`;
- `PH1.CLAIM.VERIFY`;
- `PH1.CITATION.BUILD`;
- `PH1.RESEARCH.CLAIM_LEDGER`;
- `PH1.RESEARCH.CONTRADICTION_MATRIX`;
- `PH1.PROVIDER.GATE`;
- `PH1.PROVIDER.BUDGET`;
- `PH1.PROVIDER.OFF.PROOF`;
- `PH1.SEARCH.AUDIT`.

Required app/connector search lane:

- `PH1.APPS.CONNECTORS`;
- `PH1.CONNECTOR.READ`;
- `PH1.CONNECTOR.WRITE`;
- `PH1.APP.AUTH`;
- `PH1.APP.PERMISSIONS`;
- `PH1.APP.INDEX`;
- `PH1.APP.SEARCH`;
- `PH1.APP.SOURCE.CHIP`.

Connector examples:

- Google Drive;
- Gmail;
- Calendar;
- Slack;
- Notion;
- SharePoint;
- Dropbox;
- GitHub;
- Jira;
- Confluence;
- CRM;
- company database.

Connector rules:

- Read connector = search/reference data.
- Write connector = send/post/edit/delete.
- Connector writes require access, authority, simulation, execution gate, runtime law, and audit.

Required platform API lane:

- `PH1.SEARCH.API_ROUTE`;
- `PH1.API.CAPABILITY.REGISTRY`;
- `PH1.API.AUTH`;
- `PH1.API.RATE_LIMIT`;
- `PH1.API.RIGHTS_POLICY`;
- `PH1.API.SOURCE_MAP`.

Platform API rules:

- GitHub routes through GitHub API plus public web search where appropriate.
- LinkedIn uses official API, approved partner access, or public indexed web only.
- Facebook/Meta uses Graph API, Pages API, or public indexed web only.
- YouTube uses API plus public web where appropriate.
- Reddit uses API plus public web where appropriate.
- X/Twitter uses official API or public web where allowed.
- Do not scrape restricted platforms.
- Use official API, authorized connector, public web, or say access is unavailable.

Required offline/cached search lane:

- `PH1.SEARCH.CACHE`;
- `PH1.SEARCH.OFFLINE_INDEX`;
- `PH1.SEARCH.CACHE_TTL`;
- `PH1.SEARCH.CACHE_SAFETY`;
- `PH1.SEARCH.FRESHNESS_CHECK`.

Offline/cache rules:

- Cached search must clearly know freshness limits.
- Cached/offline search must support lower cost, faster answers, provider-off mode, enterprise compliance, and lower live-provider dependence without pretending stale data is current.

Required real-time vertical lanes:

- `PH1.WEATHER`;
- `PH1.TIME`;
- `PH1.FINANCE`;
- `PH1.NEWS`;
- `PH1.SHOPPING.SEARCH`;
- `PH1.PRODUCT.CARD`;
- `PH1.ACADEMIC.SEARCH`;
- `PH1.REGISTRY.SEARCH`;
- `PH1.FILING.SEARCH`.

Verticals must produce structured packets, not just links.

Search-to-Write pipeline:

```text
Search evidence
-> EvidencePacket
-> Write engine
-> Presentation contracts
-> Source chips
-> native client renderer
-> clean TTS text
```

Do not build search as:

```text
user query -> Brave API -> summarize top result
```

Build it as:

```text
user query
-> intent/routing
-> search lane selection
-> query plan
-> provider/API/connector
-> page fetch/read
-> source verification
-> claim verification
-> evidence packet
-> Write
-> presentation/source chips
-> audit/eval
```

ChatGPT-level search product behaviors:

- Quick search: short answer plus accepted source chips.
- Current search: latest sources plus timestamps/as-of state.
- Deep research: structured report with citations and rejected/weak-source handling.
- Private/company search: connector search with permission proof.
- GitHub search: API/connector results with issue/PR links.
- Document search: URL/file reader with citations.
- Research with images: approved image cards only.
- Shopping/product search: product cards, source-backed prices, and merchant safety.
- Local/current tools: weather/time/finance/news/flights from structured lanes, not generic web search where a vertical is better.
- Protected workflow: sending, posting, deleting, inviting, approving, purchasing, or mutating is not search and requires protected workflow gates.

Search build sequence inside the 34-stage roadmap:

1. Search Build 1 - Provider gates and public web baseline: public web provider, query planner, result fetch, source accept/reject, source chips, provider budget, provider-off proof.
2. Search Build 2 - URL/PDF/page reader: URL fetch, HTML extraction, PDF extraction, table extraction, safe citation extraction.
3. Search Build 3 - Source verification: official detector, freshness, trust score, contradiction detection, wrong-source rejection, claim-to-source mapping.
4. Search Build 4 - Deep research: multi-query, multi-hop, source plan, claim ledger, citation graph, rejected-source log, research report builder.
5. Search Build 5 - Connectors/apps: GitHub, Drive, Gmail, Slack/Notion/SharePoint, read-only connector search, write separation.
6. Search Build 6 - Platform APIs: GitHub API, Meta API where permitted, LinkedIn API only where approved, YouTube/Reddit/X APIs where permitted.
7. Search Build 7 - Image/product/vertical search: image evidence cards, shopping/product cards, weather/time/finance/news/flights, academic/government/filings.
8. Search Build 8 - Native presentation: source chips, image cards, research report blocks, Desktop renderer, mobile renderer, clean TTS.
9. Search Build 9 - Evaluation and certification: citation accuracy, claim-source precision, official-source preference, freshness accuracy, contradiction detection, hallucinated citation rate = 0, provider-off zero calls.

Search ownership rule:

- Stage 13 owns public web, minimal URL/PDF/page/table reader extraction foundation, source verification, connector/API search route contracts with fixture/mock proof only, image/product evidence, real-time verticals, and Search Builds 1-3 and 7.
- Stage 14 owns deep research, enterprise search sublanes, Research OS release proof, and Search Build 4.
- Stage 15 owns search-to-Write answer quality.
- Stage 16 owns source chips, image cards, product cards, research report blocks, and evidence presentation contracts.
- Stage 18 owns structured search packet preservation across HTTP/gRPC/native transport.
- Stage 22 owns full file/document/data/vision implementation and must reuse the Stage 13 reader extraction foundation where reader contracts overlap.
- Stage 24 owns live app/connector search, platform APIs, read/write connector separation, completion of Stage 13 connector/API route contracts, and Search Builds 5-6.
- Stage 33 certifies native/runtime parity for search presentation.
- Stage 34 certifies Search Build 9 and the full ChatGPT-level search product behavior.

## Stage 1 - Canonical Inventory And Wiring Map

Status: PROVEN_COMPLETE

Build:

- one repo-truth inventory of all contracts, engines, OS modules, adapter routes, native clients, tools, migrations, docs, tests, and proof artifacts;
- one wiring map from activation to session, turn, understanding, routing, tools, protected execution, Write, adapter, Desktop, iPhone, TTS, audit, and replay;
- `docs/SELENE_CANONICAL_STAGE1_INVENTORY_AND_WIRING_MAP.md`;
- `docs/SELENE_CANONICAL_DEPENDENCY_DAG.md`;
- `docs/SELENE_CANONICAL_GOLDEN_JOURNEY_MATRIX.md`;
- `docs/SELENE_CANONICAL_STAGE_BUILD_SLICE_MAP.md`;
- `docs/SELENE_CANONICAL_BENCHMARK_TARGET_STATUS_MATRIX.md`;
- one dependency DAG showing every stage, input packet, output packet, upstream dependency, downstream consumer, blocked stage, parallelizable slice, and must-not-start-early slice;
- one build-family/slice map for broad stages, including Stage 8, Stage 10, Stage 13, Stage 15, Stage 21, Stage 24, Stage 29, and Stage 30;
- one packet/handoff matrix proving current repo equivalents for every canonical packet in this document;
- initial Golden Journey Matrix artifact with 15-20 end-to-end journeys;
- one status table for every module or engine: complete, partial, standalone, duplicated, unwired, missing, legacy, or deprecated;
- one reconciliation between this plan, `SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`, `SELENE_BUILD_EXECUTION_ORDER.md`, older 86-engine references, existing build sections, native-client surfaces, adapter/runtime routes, and tests/proofs.

Must inventory:

- runtime/storage/proof-ledger systems;
- wake, side button, explicit mic, typed input, record button;
- session, turn, listen, interruption, TTS output control;
- Voice ID, access, policy, tenant, KMS, provider, cost, and quota;
- public tools, search, web search sublanes, image cards, source chips;
- Search Operating System lanes: public web, URL/PDF/page reader, Stage 13 minimal reader extraction foundation, deep research, source verification, claim/citation system, Stage 13 connector/API route contracts, connectors/apps, platform APIs, cache/offline index, real-time verticals, image/product evidence, search-to-Write pipeline, and search certification;
- protected workflow engines, simulations, execution gates, and runtime law;
- Write, presentation, adapter, Desktop, iPhone, record mode, and TTS;
- learning, memory, knowledge, emotional guidance, replay, builder, self-heal, and release tooling;
- conversation control, discourse frame, clarification loop, correction control, conversation recovery, conversation trace, and conversation eval ownership;
- human experience, smooth spoken interaction, persona continuity, prosody, latency/responsiveness, and emotional guidance ownership;
- default-empty human-experience packet contracts before live Stage 21/29 producers;
- accuracy, research, model intelligence, elite listening/STT, spelling/grammar mastery, model routing, memory accuracy, and competitive benchmark ownership;
- global number-one quality system, hard benchmark targets, provider championship router, world-class listening lab, meaning reconstruction, same-page conversation, domain excellence, Research OS, memory trust benchmarks, and human-experience lab ownership;
- ChatGPT-parity product lanes: custom assistants, app directory/SDK/MCP apps, interactive app cards, visual agent/watch mode, data sandbox/charts, canvas sharing/export/version restore, study/tutor mode, shopping/product cards, and video generation/editing;
- early privacy/retention/admin policy contract baseline and consent revocation propagation for wake training, Voice ID, memory, record retention, file/data artifacts, connector/app data, generated media, learning signals, and provider-capable voice processing;
- exact repo surface audit anchors listed later in this document, including exact binaries, native app names, storage/migration tokens, grouped storage docs, and `web_search_plan` leaf modules;
- duplicate, stale, legacy, or drifting concepts.
- stages or build slices that can safely run in parallel and stages or build slices that must not start before upstream packet proof exists.

Rules:

- This is docs/architecture only.
- No behavior change.
- No provider calls.
- No Python.
- Do not force Stage 2 if repo evidence proves a narrower or merged Stage 2 is required.
- The dependency DAG controls build execution after Stage 1; the numbered roadmap controls canonical stage order.
- Stage 1 must split broad stages into build families/slices before any later build treats one broad stage as one Codex implementation run.

Proof:

- exact file paths and repo evidence;
- `docs/SELENE_CANONICAL_DEPENDENCY_DAG.md` exists and is linked from the inventory;
- dependency DAG covers every numbered stage, input packet, output packet, upstream dependency, downstream consumer, blocked stage, parallelizable slice, and must-not-start-early slice;
- build-family/slice map exists for Stage 8, Stage 10, Stage 13, Stage 15, Stage 21, Stage 24, Stage 29, and Stage 30;
- initial Golden Journey Matrix exists and is linked from the inventory;
- model capability routing, elite listening/STT, spelling/grammar, full web research, memory accuracy, and benchmark ownership are mapped to concrete stages;
- hard benchmark targets, gold corpora, provider championship routing, listening lab, meaning reconstruction, domain excellence, Research OS, same-page conversation, and human-experience lab ownership are mapped to concrete stages;
- early privacy/retention/admin policy contracts, default-empty human-experience packet contracts, minimal reader extraction foundation, connector/API route contracts, and core-vs-extended native renderer proof ownership are mapped to concrete stages;
- benchmark target status ownership is mapped to concrete stages and broad-stage slices;
- ChatGPT-parity product lanes are mapped to concrete owning stages with status and repo evidence;
- no code behavior change;
- final output states exact Stage 2 scope and blockers, if any.

Next if passed:

- Stage 2A - Runtime Kernel, Storage, Proof Ledger, Law Foundation, And Minimal Benchmark Envelope Inventory Reconciliation.

## Stage 2 - Runtime Kernel, Storage, Proof Ledger, And Law Foundation

Status: PROVEN_COMPLETE

Build:

- runtime execution envelope;
- runtime request foundation;
- runtime governance foundation;
- runtime law foundation;
- runtime bootstrap;
- active ingress boundary `app_ingress`;
- graceful exit/shutdown boundary `section40_exit`;
- storage foundation `PH1.F`;
- audit/proof ledger `PH1.J`;
- ECM contract/governance carrier `PH1.ECM`;
- blueprint table carrier `PH1.PBS`;
- simulation catalog carrier `PH1.SIMCAT`;
- OS core tables;
- blueprint registry tables `PBS_TABLES`;
- simulation catalog tables;
- engine capability map tables;
- artifacts ledger tables;
- minimal proof/replay envelope for stage-level regression, golden journey protection, and packet handoff proof;
- minimal benchmark target/result envelope for number-one quality gates;
- benchmark result storage and replay-safe comparison path;
- deterministic compute and consensus utility `PH1.COMP`;
- canonical runtime trace envelope fields: `session_id`, `turn_id`, `route_id`, `provider_budget_id`, `artifact_id`, `evidence_id`, `simulation_id`, `execution_id`, `consent_state_id`, `device_trust_id`, `render_hash`, `tts_hash`, and `audit_id`;
- repository/storage access boundaries;
- idempotency, replay safety, and append-only proof invariants.

Repo systems:

- `runtime_execution.rs`;
- `runtime_governance.rs`;
- `runtime_law.rs`;
- `runtime_bootstrap.rs`;
- `runtime_request_foundation.rs`;
- `runtime_session_foundation.rs`;
- `runtime_ingress_turn_foundation.rs`;
- `app_ingress.rs`;
- `section40_exit.rs`;
- `selene_storage`;
- storage migrations.

Rules:

- All later engines must have a lawful runtime envelope.
- All protected state changes must be idempotent, replay-safe, and audit-backed.
- Storage and audit are infrastructure, not reasoning engines.
- Ledger writes must not be hidden behind UI or adapter behavior.

Proof:

- runtime envelope compiles;
- DB wiring tests pass for foundation tables;
- trace envelope compile/proof path exists;
- minimal proof/replay envelope proof;
- minimal benchmark target/result envelope proof;
- replay-safe benchmark comparison proof;
- append-only audit invariant proof;
- replay/idempotency proof;
- no business workflow can execute from foundation alone.

Stage 2A completion proof:

- `RuntimeExecutionEnvelope`, runtime request/session/ingress-turn foundations, `app_ingress`, `section40_exit`, runtime governance, runtime law, `PH1.F`, `PH1.J`, `PH1.COMP`, `PH1.ECM`, `PH1.PBS`, and `PH1.SIMCAT` were reconciled as the repo-truth Stage 2 carriers.
- Canonical Stage 2 packet names that are roadmap aliases remain documented as crosswalks instead of duplicate engines.
- `BenchmarkTargetPacket`, `BenchmarkResultPacket`, `BenchmarkTargetStatus`, and `BenchmarkComparisonOutcome` were added under PH1.J as the minimal benchmark envelope.
- `BenchmarkResultRepo` and `Ph1fStore` now provide append-only target/result storage, idempotency, by-target lookup, latest-result lookup, and replay-safe result-to-target comparison.
- Stage 2A benchmark status is `CERTIFICATION_TARGET_PASSED` for the minimal benchmark envelope foundation. Product benchmark families remain blocked by their owning future stages.

Next if passed:

- Stage 3 - Provider, Secret, KMS, Cost, Quota, Vault, And Early Consent Baseline.

## Stage 3 - Provider, Secret, KMS, Cost, Quota, Vault, And Early Consent Baseline

Status: FOUNDATION_PROVEN_BY_STAGE_10A

Stage 10A status: PROVEN_COMPLETE

Stage 3A status: PROVEN_COMPLETE

Stage 3B status: PROVEN_COMPLETE

Build:

- `PH1.KMS`;
- `PH1.PROVIDERCTL`;
- `provider_secrets`;
- `device_vault`;
- `vault_cli`;
- provider kill switches;
- provider call budgets and counters;
- provider/model/prompt governance contracts;
- prompt version registry;
- model version registry;
- model capability registry for fast chat, deep reasoning, coding, search synthesis, document analysis, translation, STT, TTS, image understanding, image generation, embeddings/memory retrieval, safety classification, and grammar/style polish;
- `PH1.MODEL.CHAMPION.ROUTER`;
- `PH1.MODEL.TASK_PROFILE`;
- `PH1.MODEL.LIVE_EVAL`;
- `PH1.MODEL.FALLBACK`;
- `PH1.MODEL.ROLLBACK`;
- `PH1.MODEL.COST_QUALITY_SCORE`;
- `PH1.STT.PROVIDER.ROUTER`;
- `PH1.TTS.PROVIDER.ROUTER`;
- `PH1.PROVIDER.PRIVACY.MODE`;
- `PH1.PROVIDER.LATENCY.MODE`;
- `PH1.PROVIDER.OFFLINE.FALLBACK`;
- approved model profile contracts;
- provider selection and fallback policy contracts;
- STT provider profile contracts for OpenAI, Apple on-device/platform, Google/Gemini, Deepgram, AssemblyAI, Whisper-family, local/offline, and future approved specialist STT providers;
- TTS provider profile contracts for OpenAI, local/platform voices, offline/degraded fallback, and future approved premium voice providers;
- paid-provider gate;
- startup-probe block;
- proxy configuration/redaction;
- `PH1.COST`;
- `PH1.QUOTA`;
- cost/usage ownership fields;
- early consent carrier for wake training, Voice ID enrollment, record mode, memory, and provider-capable voice paths;
- privacy/retention/admin policy contract baseline;
- retention class registry contracts for wake artifacts, Voice ID profiles, memory records, record artifacts, file/data artifacts, connector/app data, generated media artifacts, learning signals, and provider-capable voice processing;
- admin disable-policy contracts for wake, Voice ID, record mode, memory, connectors, app directory lanes, visual agent, shopping links, study persistence, generated media, retention, and provider lanes;
- consent revocation policy contracts before live feature producers consume consent-scoped data;
- device trust and client permission baseline;
- provider-off proof.

Rules:

- No provider call without global provider gate and budget/counter.
- Stage 3 owns provider/model/prompt governance contracts.
- Stage 3 owns model capability routing contracts and allowed model profiles.
- Stage 3 owns provider championship router contracts, STT/TTS provider profile contracts, privacy/latency/offline routing modes, and cost-quality scoring.
- No single model or provider is assumed best at everything.
- No paid provider without paid-provider gate.
- Startup and health checks must not call providers.
- Provider keys never reach native clients.
- KMS issues opaque handles, not raw secrets.
- Disabled providers produce zero attempts and zero network dispatches.
- Wake training, Voice ID enrollment, record mode, and memory capture must have consent state before later feature stages use them.
- Consent revocation must invalidate or block wake training, Voice ID enrollment/matching where policy requires, memory capture/recall where policy requires, record retention, and provider-capable voice processing where policy requires.
- Stage 3 owns the early privacy/retention/admin policy contract baseline that later feature stages must consume before storing, recalling, exporting, retaining, or deleting user data.
- Stage 31 owns the full privacy, retention, admin policy, audit export, compliance export, health, and retention lifecycle implementation; later stages must not wait until Stage 31 to learn their policy contract shape.

Proof:

- provider-off zero attempt proof;
- KMS no-secret-leak proof;
- budget/counter proof;
- provider/model/prompt governance contract proof;
- prompt/model version registry proof;
- model capability registry proof;
- provider championship router contract proof;
- STT/TTS provider router contract proof;
- cost-quality score and privacy/latency/offline mode proof;
- approved model profile proof;
- quota wait/refuse proof;
- proxy redaction proof;
- startup no-probe proof;
- early consent/device-trust baseline proof;
- privacy/retention/admin policy contract baseline proof;
- retention class registry contract proof;
- admin disable-policy contract proof;
- consent revocation propagation proof.

Stage 3A completion proof:

- `PH1.KMS`, `provider_secrets`, `PH1.COST`, `PH1.QUOTA`, `PH1.PROVIDERCTL`, runtime bootstrap health/readiness/startup behavior, vault CLI surfaces, and PH1.F storage were reconciled as the repo-truth Stage 3A carriers.
- `ConsentStatePacket`, `ConsentScope`, and `ConsentDecisionState` now provide the minimal early consent baseline for wake training, Voice ID enrollment/matching, record mode, memory capture/recall, and provider-capable voice processing.
- PH1.F now stores consent state packets append-only with idempotency, current-by-subject/scope lookup, by-id lookup, and revocation-aware grant checks through `ConsentStateRepo`.
- Provider-off proof remains deterministic: disabled providers block before provider attempts and before network dispatch, startup probes are disabled before attempts/dispatches, and health/startup endpoint proof does not fetch provider secrets.
- `ProviderBudgetPacket` remains a roadmap alias crosswalked to existing provider network policy, provider counters, PH1.COST, and PH1.QUOTA carriers instead of a duplicate budget engine.
- `DeviceTrustPacket` remains a roadmap alias for existing device-trust/runtime/access carriers and is not duplicated in Stage 3A.
- Broad provider/model governance items after Stage 3A remain split: Stage 3B owns STT/TTS provider-router contract closure, while Stage 30 owns prompt/model registries, approved model profile registry, provider championship router promotion, live eval, fallback/rollback, and cost-quality scoring.
- Stage 3A benchmark status is `CERTIFICATION_TARGET_PASSED` for provider-off, startup no-probe, and early consent baseline proof. Product provider/model championship benchmarks remain blocked by Stage 30.
- Stage 4A is ready to start because activation/session/turn packets can now depend on runtime, proof, KMS/provider-off, budget, and early consent baselines without live-provider calls.

Stage 3B completion proof:

- `SttProviderProfilePacket`, `TtsProviderProfilePacket`, `VoiceProviderRouteDecisionPacket`, `VoiceProviderSelection`, `VoiceProviderQualitySignal`, and `VoiceProviderFallbackReason` now provide the minimal STT/TTS provider-router contract foundation in `crates/selene_kernel_contracts/src/ph1c.rs`.
- Apple platform and OpenAI STT/TTS/realtime profile contracts are represented as inert provider profiles. Apple/platform candidates can be selected for Mac/iPhone local/platform paths where policy allows; OpenAI realtime/transcription/speech candidates can be selected only as cloud/profile contracts where policy allows.
- Route decisions are contract-only and cannot call providers, capture audio, transcribe, synthesize, speak, identify, authorize, search, route tools, connector-write, or execute protected mutations.
- Provider-off proof remains deterministic: contract route decisions keep `provider_call_attempt_count=0` and `provider_network_dispatch_count=0`, require an explicit fallback reason, and cannot select cloud providers under provider-off.
- Missing-secret, budget, privacy, latency, confidence, language, platform, consent, offline, and protected-slot fallback reasons are explicit and auditable. Apple/local fallback remains a contract decision only and does not call Apple APIs.
- Live Apple Speech, Apple AVSpeechSynthesizer, OpenAI STT/TTS/realtime sessions, live provider/model routing, live STT/TTS, native Swift implementation, model championship routing, prompt/model promotion, live eval, rollback, and cost-quality scoring remain deferred to Stage 8, Stage 17, Stage 30, and Stage 34 as appropriate.
- Stage 3B benchmark status is `CERTIFICATION_TARGET_PASSED` for STT/TTS provider-router contracts and provider-off zero-attempt/zero-dispatch proof. Live STT WER/noise/accent benchmarks remain deferred to Stage 8 slices; live TTS MOS/prosody/pronunciation benchmarks remain deferred to Stage 17 slices.
- Stage 8B remains the next exact build because Stage 8A has passed and the voice transcript boundary now has provider-router contract support without live-provider behavior.

Next if passed:

- Stage 8B - VAD, Endpointing, Partial-Versus-Final Transcript Commit, And Confidence Gate Reconciliation.

## Stage 4 - Activation, Session, Turn, And Packet Foundation

Status: PROVEN_COMPLETE

Stage 4A status: PROVEN_COMPLETE

Build:

- activation source packet;
- canonical ingress packet from `app_ingress`;
- platform packet;
- typed trigger packet;
- wake trigger packet;
- side-button trigger packet;
- explicit mic/live voice trigger packet;
- record-button trigger packet;
- session open/resume/close packet;
- turn candidate packet;
- committed turn packet;
- conversation state packet;
- discourse frame packet;
- clarification/correction/recovery packet;
- voice-chat versus record-mode discriminator;
- recording session fields: `recording_session_id`, `recording_state`, `audio_artifact_id`, `consent_state`;
- Voice ID candidate fields;
- access/tenant/workspace fields;
- authority-state fields;
- provider/cost budget carrier fields;
- trace and correlation IDs;
- device trust and client-render state fields.

Rules:

- No raw audio directly enters understanding.
- Record-mode audio cannot become live chat.
- Partial voice cannot trigger tools.
- Wake does not answer.
- Side button does not bypass identity, access, or authority.
- Native clients only send packets and render responses.

Proof:

- packet compile tests;
- non-committed turn rejection;
- record-mode audio stays in artifact lane;
- no tool/search/execution route from packet creation alone.

Stage 4A completion proof:

- `Stage4ActivationPacket`, `Stage4ActivationSource`, `Stage4TurnBoundaryPacket`, `Stage4TurnBoundaryKind`, `Stage4RecordBoundary`, `Stage4RecordingState`, and `Stage4PacketRouteAuthority` now provide the minimal Stage 4A packet-boundary carrier in `runtime_ingress_turn_foundation.rs` without creating a separate brain or feature router.
- Canonical `ActivationPacket`, `TurnCandidatePacket`, `CommittedTurnPacket`, `RecordSessionPacket`, and `AudioArtifactPacket` remain roadmap packet names crosswalked to the Stage 4A carrier plus existing runtime ingress/session/PH1.F/PH1.J carriers.
- `SessionPacket` remains crosswalked to `PH1.L`, `SessionRuntimeProjection`, and `SessionTurnPermit`; Stage 5A owns lifecycle promotion, stale-turn quarantine, and cancellation closure.
- `ConsentStatePacket` is consumed as the Stage 3A provider-secrets consent carrier; `DeviceTrustPacket` is crosswalked to `PlatformRuntimeContext.device_trust_class`; `ProviderBudgetPacket` is crosswalked to PH1.COST/PH1.QUOTA/provider-control budget and counter carriers.
- Packet creation now has an explicit no-authority proof: Stage 4A packet carriers report no route authority for tools, search, providers, TTS, or protected execution.
- Record-button activation is artifact-only at this stage: record packets require `recording_session_id`, `recording_state`, `audio_artifact_id`, `consent_state_id`, and `artifact_lane_handoff_ref`, cannot carry a live turn ID, route ID, modality, or device turn sequence, and cannot become live chat.
- Wake, side button, typed input, explicit mic, and record button are represented as activation source packet foundations only. Wake detection, live voice, STT/TTS, Voice ID enrollment/matching, native UI behavior, and the Stage 27 record product remain deferred to their owning stages.
- Stage 4A benchmark status is `CERTIFICATION_TARGET_PASSED` for packet-boundary/no-route-authority and record-artifact separation proof.
- Stage 5A is ready to start as the narrowed session open/resume/close, runtime turn spine, and stale-turn quarantine build.

Next if passed:

- Stage 5A - Session Open, Resume, Close, Runtime Turn Spine, And Stale-Turn Quarantine Reconciliation.

## Stage 5 - Session Open, Resume, Close, And Runtime Turn Spine

Status: PROVEN_COMPLETE

Stage 5A status: PROVEN_COMPLETE

Stage 5B status: PROVEN_COMPLETE

Build:

- `PH1.L` session lifecycle;
- session open/resume/close;
- session transfer/attach;
- timeout and recovery;
- turn candidate to committed turn promotion;
- stale turn quarantine;
- abandoned turn safe-degrade;
- interrupted turn cancellation;
- superseded turn invalidation;
- `PH1.CONVERSATION.CONTROL`;
- `PH1.DISCOURSE.FRAME`;
- `PH1.CLARIFICATION.LOOP`;
- `PH1.CORRECTION.CONTROL`;
- `PH1.CONVERSATION.RECOVERY`;
- `PH1.CONVERSATION.TRACE`;
- `PH1.CONVERSATION.EVAL`;
- `PH1.CONVERSATION.GOAL.STATE`;
- `PH1.CONVERSATION.TOPIC.SEGMENT`;
- `PH1.CONVERSATION.ACTIVE_ENTITY`;
- `PH1.CONVERSATION.OPEN_LOOPS`;
- `PH1.CONVERSATION.USER_INTENT_HISTORY`;
- `PH1.CONVERSATION.CORRECTION_STATE`;
- `PH1.CONVERSATION.RECAP_ON_RETURN`;
- `PH1.CONVERSATION.SAME_PAGE.CHECK`;
- natural turn-taking policy;
- pause and hesitation handling;
- short spoken acknowledgement and backchannel policy;
- no awkward over-apology policy;
- smooth correction and follow-up continuity;
- cross-device timeline and single-writer behavior;
- record session separated from live chat session.

Rules:

- Only a committed current turn can enter understanding.
- A closed session cannot execute.
- Old tool results must not render as current answers.
- Old speech output must not continue after cancellation.
- Record sessions cannot accidentally run live chat or tools.
- Conversation control owns discourse state, clarification, correction, recovery, and conversation trace before understanding consumes a committed turn.
- Same-page state owns current goal, topic segment, active entities, open loops, intent history, correction state, and recap-on-return before Write/TTS produce the next response.
- Backchannels such as brief acknowledgement may be emitted only as safe conversational control output and must not claim completion, evidence, authority, or execution.
- Warmth and conversational style must not override truth, safety, access, provider gates, or protected execution law.

Proof:

- stale/cancelled/superseded turns blocked;
- committed turn accepted;
- closed session blocked;
- record session distinct from chat session;
- conversation control handoff proof;
- clarification/correction/recovery packet proof;
- conversation goal/topic/active-entity/open-loop proof;
- recap-on-return and same-page check proof;
- natural turn-taking, backchannel, and no-over-apology proof;
- follow-up continuity and smooth correction proof;
- no raw text to tool route.

Stage 5A completion proof:

- Existing `PH1.L` and `runtime_session_foundation.rs` carriers were reused for session create/open, resume, recover, soft-close, close, attach, transfer, failover recovery, single-writer turn admission, retry reuse, stale device-turn rejection, deferral, backpressure, lease/ownership checks, reason-coded errors, and PH1.L event/counter proof. No duplicate session engine was created.
- `Stage5TurnAuthorityPacket`, `Stage5TurnAuthorityDisposition`, and `Stage5TurnWorkAuthority` now provide the minimal current-turn authority carrier in `runtime_session_foundation.rs`.
- `authorize_stage5_current_committed_turn` admits a committed turn only when the `SessionTurnCommit` matches the original `SessionTurnPermit`, the session is not closed, no active writer or deferred newer turn exists, and the device timeline still identifies that same turn as current.
- Stale, deferred, superseded, cancelled, abandoned, retry-reused, closed-session, and record-artifact dispositions are represented as quarantined Stage 5A authority packets that cannot enter understanding, cannot render as current, and cannot route tools, search, providers, TTS, connector work, or protected execution.
- Old result render is blocked by session/turn state: if a newer turn is deferred while the first turn drains, or if a newer turn has already committed, the older turn authority resolves to `SupersededTurnQuarantined`.
- Record sessions remain distinct from live chat by Stage 4A `RecordArtifactOnly` packet proof and Stage 5A `RecordArtifactOnly` quarantine disposition. Stage 5A did not build the Stage 27 record product.
- Stage 5A broad conversation-control items were completed by Stage 5B.

Stage 5B completion proof:

- Existing repo carriers were reused and crosswalked instead of duplicated: `PH1.CONTEXT` already represents conversation-state and clarification-history context sources, `PH1.N` already enforces one-question clarification contracts, `PH1.SRL` already carries ambiguity-to-clarification proof, and the adapter H411 public-discourse frame remains a client/adapter-local product surface rather than the canonical brain.
- `Stage5ConversationControlPacket`, `Stage5ConversationControlDisposition`, `Stage5ConversationWorkAuthority`, `Stage5SamePageState`, `Stage5ClarificationState`, `Stage5CorrectionState`, and `Stage5ConversationDeliveryPolicy` now provide the minimal runtime conversation-control carrier in `runtime_session_foundation.rs`.
- `Stage5ConversationControlPacket::from_turn_authority` consumes `Stage5TurnAuthorityPacket` only. Current committed turn authority may update advisory conversation state; stale, deferred, superseded, cancelled, abandoned, closed-session, retry-reused, and record-artifact-only authority resolves to `TurnAuthorityBlocked` without same-page, clarification, or correction state.
- Clarification proof is bounded to one best question with 2-3 accepted answer formats, protected-slot uncertainty marking, and repeat-count limits. Clarification state cannot authorize, execute, search, route providers, route TTS, or mutate protected state.
- Correction proof is session-scoped only. Correction state cannot write memory, rewrite facts, rewrite protected slots, rewrite audit, or grant authority; governed memory/persona correction remains Stage 21.
- Same-page proof now carries current goal, topic segment, active entity ids, open loops, pending questions, corrected assumption refs, recap-on-return marker, snapshot hash, advisory-only status, and safe backchannel policy. The policy cannot claim task completion, evidence truth, authority, or execution, and requires no-over-apology/filler-loop control.
- Stage 5B benchmark status is `CERTIFICATION_TARGET_PASSED` for conversation-control authority consumption, stale-turn update blocking, one-question clarification, session-scoped correction, advisory same-page state, safe backchannel/no-over-apology policy, and no route authority. Rich natural-language continuity, emotional experience, and product-quality benchmarks remain owned by Stage 10, Stage 29, and Stage 34.
- Broad Stage 5 is complete enough for Stage 6 access/authority context to start. No understanding, routing, search, TTS, memory/persona, emotion, native UI, or protected workflow behavior was built in Stage 5B.

Next if passed:

- Stage 6A - Master Access, Tenant, Policy, And Per-User Authority Context Reconciliation.

## Stage 6 - Master Access, Tenant, Policy, And Per-User Authority Context

Status: PROVEN_COMPLETE

Stage 6A status: PROVEN_COMPLETE

Stage 6A completion note:

- `Stage6AccessContextPacket`, `Stage6AccessContextInput`, `Stage6AccessContextDisposition`, `Stage6AccessWorkAuthority`, `Stage6AuthorityRequestKind`, and `Stage6IdentityPosture` now provide the minimal runtime-owned access-context carrier in `runtime_session_foundation.rs`.
- Stage 6A consumes only current Stage 5 turn authority and advisory Stage 5B conversation state. Stale, cancelled, superseded, abandoned, closed-session, and record-artifact-only turns resolve to `Stage5AuthorityBlocked` and cannot construct access context.
- Public read-only context is representable without mutation authority; it cannot route search/tools/providers/TTS or authorize sends, posts, deletes, purchases, connector writes, or protected business changes.
- Protected-action context can be marked ready only when tenant, actor, consent, device trust, verified identity posture, policy context, access context, and audit references are present and positive. It still cannot execute, simulate, route tools/search/providers/TTS, connector-write, or grant authority; Stage 12 remains the protected execution gate.
- Unknown, low-confidence, wrong-speaker, multi-speaker, cross-tenant, revoked-consent, untrusted-device, missing-access, policy-denied, approval-required, and step-up-required cases fail closed for protected authority context.
- Existing PH1.ACCESS/PH2.ACCESS storage, PH1.TENANT, PH1.POLICY, PH1.GOV, PH1.VOICE.ID posture, Stage 3 consent/provider-safety, and Stage 5 session/conversation carriers remain the repo-truth sources and were not duplicated.

Build:

- `PH1.ACCESS.001`;
- `PH2.ACCESS.002`;
- master access schema;
- tenant overlays;
- board policy;
- per-user access instance;
- temporary/permanent overrides;
- access gate;
- access audit;
- `PH1.TENANT`;
- `PH1.POLICY`;
- access profile transport to runtime packets.

Rules:

- Access comes from master access/per-user access truth.
- Do not create a second access brain.
- Voice ID can identify; access decides permission.
- Tenant and workspace context must be explicit.
- Access ambiguity fails closed for protected work.

Proof:

- access allowed/denied proof;
- tenant overlay proof;
- override proof;
- access instance compile proof;
- protected command blocked without access.

Next if passed:

- Stage 7 - Wake, Side Button, And Activation Stack.

## Stage 7 - Wake, Side Button, And Activation Stack

Status: PROVEN_COMPLETE

Stage 7A status: PROVEN_COMPLETE

Build:

- `PH1.W`;
- `PH1.ACTIVATION.TRIGGER`;
- wake detector;
- low-power listener;
- wake candidate packet;
- wake decision packet;
- activation prefix cleanup;
- wake training and artifact path;
- `ph1wake_training` OS integration;
- wake policy;
- wake trace and eval;
- platform-specific activation routing.

Platform rules:

- Mac Desktop: wake word plus explicit mic/button.
- Windows Desktop: wake word plus explicit mic/button.
- Android: wake word plus explicit mic/button.
- iPhone: side button/push-to-talk only, no always-listening wake detector.

Rules:

- Wake opens or resumes a session.
- Wake does not understand intent.
- Wake does not execute.
- Wake does not prove identity.
- Side button still needs Voice ID and access where required.
- `Selene, ...` prefix cleanup works after wake or side button.

Proof:

- wake candidate does not execute;
- false wake safe-degrades;
- side button path opens session without always-listening wake;
- wake training artifact requires consent and policy.

Stage 7A proof update:

- `Stage7ActivationContextPacket`, `Stage7ActivationDisposition`, and `Stage7ActivationWorkAuthority` now provide the minimal runtime-owned wake/side-button activation boundary in `runtime_ingress_turn_foundation.rs`.
- Wake candidates can only open/resume session attention state, carry bounded consent/device/provider-budget/access/audit references, and emit wake event/artifact references. They cannot understand, answer, search, call providers, trigger Voice ID matching, authorize, route tools, speak TTS, execute protected mutations, or perform connector writes.
- iPhone side-button activation is represented as explicit activation only. iPhone wake-word activation remains blocked before an activation packet can be created because iOS has explicit-only trigger policy and no negotiated wake-word capability.
- `WakeTrainingConsentBoundaryV1` in `ph1wake_training.rs` preserves the repo-truth wake-training lane while proving wake training/artifact use requires granted, non-revoked consent, policy context, device scope, and no raw-audio retention by default.
- Canonical `wake candidate packet`, `wake decision packet`, `side-button activation packet`, and `explicit activation packet` names are crosswalked to Stage 4 activation carriers plus the Stage 7A context packet. `DeviceTrustPacket`, `ConsentStatePacket`, and `ProviderBudgetPacket` remain Stage 3/4 references, not duplicate engines.
- Native Swift surfaces were inspected only by repo-truth path references and were not redesigned. Desktop wake-life remains proof/client transport, not the Selene brain. Android and Windows wake surfaces remain documented future/client surfaces where not present in this repo.

Next if passed:

- Stage 8 - Voice I/O, Listen State, Transcript Gate, And Turn Boundary.

## Stage 8 - Voice I/O, Listen State, Transcript Gate, And Turn Boundary

Status: PROVEN_COMPLETE_BY_STAGE_12A

Stage 8A status: PROVEN_COMPLETE

Stage 8B status: PROVEN_COMPLETE

Stage 8C status: PROVEN_COMPLETE

Stage 8D status: PROVEN_COMPLETE

Stage 8E status: PROVEN_COMPLETE

Stage 8F status: PROVEN_COMPLETE

Build:

- `PH1.K` voice runtime audio substrate;
- `PH1.C` STT routing and transcript quality gate;
- `PH1.LISTEN`;
- `PH1.ENDPOINT`;
- `PH1.CONVERSATION.TIMING`;
- `PH1.TURN.STATE`;
- `PH1.INTERRUPT.CONTROL`;
- `PH1.BARGEIN.CANCEL`;
- `PH1.SPEECH.OUTPUT.CONTROL`;
- `PH1.LISTENING.LAB`;
- `PH1.AUDIO.SCENE.CLASSIFY`;
- `PH1.NOISE.SUPPRESS`;
- `PH1.ECHO.CANCEL`;
- `PH1.SPEAKER.DIARIZE`;
- `PH1.OVERLAP.DETECT`;
- `PH1.AUDIO.OVERLAP.SEGMENT`;
- `PH1.FOREGROUND.SPEAKER.SELECT`;
- `PH1.ADDRESSED_TO_SELENE.DETECT`;
- `PH1.AUDIO.ADDRESSED_TO_SELENE`;
- `PH1.AUDIO.BACKGROUND.SPEECH`;
- `PH1.AUDIO.FOREGROUND.SPEAKER`;
- `PH1.AUDIO.MEETING_CONTEXT`;
- `PH1.AUDIO.NON_USER_SPEECH_BLOCK`;
- `PH1.TRANSCRIPT.ALTERNATIVES`;
- `PH1.TRANSCRIPT.CONFIDENCE.CALIBRATE`;
- `PH1.TRANSCRIPT.HUMAN_CORRECTION.LEARN`;
- `PH1.NOISE.BENCH`;
- `PH1.ACCENT.BENCH`;
- `PH1.DIARIZATION.BENCH`;
- `PH1.BARGEIN.BENCH`;
- `PH1.TRANSCRIPT.GOLD_CORPUS`;
- noise reduction;
- voice activity detection;
- echo cancellation;
- audio scene classification;
- endpointing;
- speaker diarization where available and policy-approved;
- overlapping speech detection;
- foreground speaker selection;
- addressed-to-Selene detection;
- background/non-user speech blocking;
- word-level timestamps;
- confidence per word and segment;
- transcript alternatives;
- accent adaptation;
- domain glossary for names, companies, products, business terms, and protected terminology;
- wake-word prefix cleanup;
- app-name disambiguation for Selene/Celine-like captures;
- second-pass transcript repair;
- exact transcript proof for voice tasks;
- mic self-echo block;
- final transcript commit path;
- partial transcript preview path;
- first-audio latency budget;
- partial acknowledgement timing;
- streaming transcript/display timing;
- voice silence thresholds;
- provider timeout fallback and graceful degradation;
- cancel/resume responsiveness;
- barge-in detection latency;
- audio stop latency;
- stale output cancellation;
- interrupted answer resume quality;
- correction after interruption;
- TTS self-echo false trigger rate;
- transcript confidence and clarification behavior.

Rules:

- Partial transcripts are preview only.
- Final transcript commit is required for live voice chat.
- Stop-speaking is a control action, not a normal user request.
- TTS self-echo cannot create a new user turn.
- Voice proof must use exact captured transcript when required.
- Background speech is not a command.
- A third party speaking near the mic is not automatically the user.
- Unknown-speaker protected actions fail closed.
- Addressed-to-Selene detection may open or continue conversation control, but it must not identify, authorize, search, route tools, or execute by itself.
- Low-confidence transcript, name, amount, date, or protected slot must clarify or fail closed.
- Transcript confidence calibration must know when not to trust audio and must prefer a sharp clarification over guessing.
- Second-pass transcript repair may improve readability but must preserve exact-transcript proof and must not silently guess protected slots.
- Smoothness controls may improve responsiveness but must not commit partial speech, route tools, or execute actions.
- Thinking/loading states must be honest and must not imply provider calls, search, or execution occurred unless they did.

Proof:

- partial transcript no-execution proof;
- final transcript handoff proof;
- interruption proof;
- barge-in proof;
- self-echo prevention proof;
- noise/VAD/endpointing proof;
- audio scene classification and echo cancellation proof;
- foreground/background/non-user speech blocking proof;
- addressed-to-Selene proof;
- overlapping speech and diarization proof;
- word-level timestamp and segment-confidence proof;
- transcript confidence calibration proof;
- human correction learning proof;
- listening lab gold-corpus proof;
- transcript alternatives proof;
- accent/domain glossary proof;
- second-pass repair no-protected-guess proof;
- exact transcript proof;
- first-audio/acknowledgement latency proof;
- timeout fallback and graceful degradation proof;
- cancel/resume responsiveness proof;
- barge-in/audio-stop/stale-output latency proof;
- interrupted-answer resume and correction-after-interruption proof;
- TTS self-echo false-trigger-rate proof;
- English and Chinese voice smoke where required by build.

Stage 8A proof update:

- Existing PH1.K, PH1.C, PH1.LISTEN, PH1.LANG, PH1.PRON, PH1.VOICE.ID posture, Desktop mic producer, voice ingress proto, capture bundle tests, Stage 4 activation, Stage 5 current-turn authority, Stage 6 access context, and Stage 7 activation context were crosswalked and not rebuilt as duplicate voice/listen/STT engines.
- `Stage8TranscriptGatePacket`, `Stage8TranscriptGateKind`, and `Stage8VoiceWorkAuthority` now provide the minimal runtime-owned audio/listen/transcript boundary carrier in `runtime_ingress_turn_foundation.rs`.
- Stage 8A proves audio substrate can update listen state only; partial transcripts are preview-only and cannot commit or enter understanding; final transcript commit requires Stage 5 current committed-turn authority and Stage 7 activation context; background/non-user/TTS self-echo audio cannot create a user turn; and record-mode audio remains artifact-only.
- Stage 8A did not add live microphone capture, live STT, live TTS, Voice ID matching, understanding, routing, search, native UI redesign, provider calls, connector writes, protected execution, or raw-audio retention.
- Instruction path reconciliation: `crates/selene_adapter/src/bin/desktop_mic_producer.rs` was requested for inspection/allowlisting, but repo truth has `crates/selene_adapter/src/desktop_mic_producer.rs`; no duplicate binary file was created.
- Full STT WER/CER, noisy/far-field/overlap/diarization/accent benchmarks, VAD/endpointing confidence gates, listening lab corpus, and word-level timestamp calibration remain deferred to later Stage 8 slices.
- Stage 8B is required before Stage 9A.

Stage 8B proof update:

- Existing PH1.K VAD events, PH1.C partial/uncertain transcript contracts, PH1.LISTEN endpoint profiles, Stage 5 current-turn authority, Stage 7 activation context, and the Stage 8A `Stage8TranscriptGatePacket` carrier were reused and not rebuilt as duplicate VAD, endpointing, confidence, listen, or STT engines.
- `Stage8EndpointState`, `Stage8ConfidenceGateDisposition`, `Stage8ProtectedSlotDisposition`, and `Stage8ProtectedSlotUncertainty` extend the existing Stage 8 transcript gate with VAD/endpoint, confidence, coverage, and protected-slot no-guess proof fields.
- Stage 8B proves VAD/endpoint signals are boundary-only; endpoint-final plus confidence-pass plus coverage-pass are required before final transcript commit; confidence rejections cannot commit, enter understanding, route tools, search, providers, TTS, Voice ID, connector writes, or protected execution; and protected-slot uncertainty clarifies or fails closed instead of guessing.
- Record-mode audio remains artifact-only and cannot enter endpoint or confidence-commit paths.
- Stage 8B did not add live microphone capture, live STT, live TTS, provider calls, live search, Voice ID matching, understanding, routing, native UI redesign, protected execution, connector writes, raw-audio retention, or a duplicate voice/listen/STT engine.
- Full listening lab scene/noise/echo/diarization/foreground-speaker/addressed-to-Selene proof, word-level timestamp calibration, alternatives, accent/domain-glossary proof, second-pass repair, and live STT quality benchmarks remain deferred to later Stage 8 slices.
- Stage 8C is required before Stage 9A.

Stage 8C proof update:

- Existing PH1.K, PH1.C, PH1.LISTEN, PH1.LANG, PH1.PRON, PH1.VOICE.ID-adjacent diarization/posture surfaces, Stage 7 activation context, Stage 8A transcript gate, Stage 8B endpoint/confidence gate, and OS voice/listen carriers were inspected and crosswalked rather than rebuilt as duplicate listening-scene, diarization, or STT engines.
- `Stage8AudioScenePacket`, `Stage8ForegroundSpeakerPacket`, `Stage8AddressedToSelenePacket`, `Stage8AudioSceneDisposition`, and `Stage8NoiseDegradationClass` extend `Stage8TranscriptGatePacket` with runtime-owned listening-scene evidence.
- Stage 8C proves audio-scene, foreground-speaker, addressed-to-Selene, echo, self-echo, noise, overlap, background, and barge-in/interruption signals are advisory or blocking boundary evidence only. They cannot understand, answer, search, call providers, trigger Voice ID matching, authorize, emit TTS, connector-write, execute protected mutations, or commit turns by themselves.
- Addressed-to-Selene and foreground-speaker evidence remain non-authoritative and do not replace Voice ID, Stage 5 current-turn authority, Stage 6 access context, endpoint-final state, confidence gates, or protected-slot no-guess rules.
- Echo, self-echo, background, non-user, overlapping-speaker, low-addressing-confidence, high-noise/degraded, and record-mode scene evidence blocks before commit or remains artifact-only.
- Stage 8C did not add live microphone capture, live STT, live TTS, provider calls, live search, Voice ID matching, understanding, routing, native UI redesign, protected execution, connector writes, raw-audio retention, or a duplicate voice/listen/STT/diarization engine.
- Numeric listening-lab benchmarks, STT WER/CER, noisy-room/far-field/overlap/diarization/accent benchmarks, endpoint latency, word-level timestamp calibration, alternatives, accent/domain-glossary proof, second-pass repair, and live STT quality proof remain deferred to Stage 8D or later slices.
- Stage 8D is required before Stage 9A.

Stage 8D proof update:

- Existing PH1.J `BenchmarkTargetPacket`/`BenchmarkResultPacket`, PH1.F storage/replay-safe benchmark rows, PH1.K, PH1.C, PH1.LISTEN, PH1.LANG, PH1.PRON, PH1.VOICE.ID-adjacent evidence, Stage 8A transcript gate, Stage 8B endpoint/confidence gate, and Stage 8C audio-scene evidence were inspected and crosswalked rather than rebuilt as duplicate benchmark, listening, STT, diarization, or calibration engines.
- `Stage8DTranscriptFixture`, `Stage8DTranscriptMetricPacket`, `Stage8DEndpointLatencyMetricPacket`, `Stage8DSceneCalibrationMetricPacket`, `Stage8DListeningBenchmarkPacket`, `Stage8DEditCounts`, `Stage8DConfidenceBucket`, `Stage8DEndpointLatencyClass`, and `Stage8DBenchmarkWorkAuthority` now provide deterministic fixture-only listening-lab scoring on top of Stage 2 benchmark envelopes.
- Stage 8D proves WER/CER-style edit counts, protected-token mismatch counts, exact/normalized transcript match, empty/garbled transcript handling, mixed-language token preservation, slang/filler preservation, endpoint latency classification, noise/echo/overlap/foreground/addressed confidence buckets, and diarization segment mismatch placeholders without live audio or provider output.
- Stage 8D benchmark packets/results are evidence only. They cannot understand, answer, search, call providers, capture microphone audio, transcribe live audio, trigger Voice ID matching, authorize, emit TTS, route tools, connector-write, execute protected mutations, or update memory/persona/emotion.
- Deterministic fixture WER/CER, endpoint latency packet scoring, and noise/echo/overlap/addressed confidence calibration are certified for Stage 8D. Live STT WER, far-field, noisy-room, accent, real diarization error rate, provider latency, production listening lab, transcript alternatives, domain vocabulary, and second-pass repair benchmarks remain deferred to Stage 8E or later live/native-lab stages.
- Stage 8D did not add live microphone capture, live STT, live TTS, provider calls, live search, Voice ID matching, understanding, routing, native UI redesign, protected execution, connector writes, raw-audio retention, or duplicate benchmark/listening/STT/calibration engines.
- Stage 8E is required before Stage 9A.

Stage 8E proof update:

- Existing PH1.J benchmark envelopes, Stage 8D deterministic listening-lab packets, PH1.K, PH1.C, PH1.LISTEN, PH1.LANG, PH1.PRON, PH1.VOICE.ID-adjacent evidence, OS voice/listen carriers, and adapter voice-ingress surfaces were inspected and crosswalked rather than rebuilt as duplicate benchmark, listening, STT, language, pronunciation, vocabulary, or repair engines.
- `Stage8ERepairBenchmarkFixture`, `Stage8EAlternativeTranscriptCandidate`, `Stage8EAlternativeTranscriptCandidateSetPacket`, `Stage8ERepairDecisionPacket`, `Stage8ERepairBenchmarkMetricPacket`, `Stage8EListeningRepairBenchmarkPacket`, `Stage8ERepairDisposition`, and `Stage8EBenchmarkWorkAuthority` now provide deterministic fixture-only proof for accent markers, mixed-language/code-switch token preservation, domain vocabulary/proper-token preservation, bounded alternative transcript candidates, and second-pass repair/no-repair decisions.
- Stage 8E proves accent markers remain benchmark metadata only, never identity or authority evidence; mixed-language/code-switch tokens are preserved without forced translation; domain vocabulary/pronunciation references are versioned/audit-visible where present; alternative transcript candidates are bounded, ordered, provider-agnostic, hashed, and non-committing; and second-pass repair can normalize fixture punctuation/case only when protected/domain tokens are preserved and no meaning drift or over-repair is detected.
- Stage 8E preserves protected-token no-guess continuity from Stage 8B and Stage 8D: low-confidence or missing protected tokens clarify/fail closed, alternative candidates cannot guess protected tokens, and repair cannot invent names, dates, amounts, addresses, recipients, account IDs, action IDs, or authorization-relevant fields.
- Stage 8E benchmark packets/results are evidence only. They cannot understand intent, answer, search, call providers, capture microphone audio, transcribe live audio, trigger Voice ID matching, authorize, emit TTS, route tools, connector-write, execute protected mutations, update memory/persona/emotion, or promote provider/model/router behavior.
- Deterministic accent, mixed-language/code-switch, domain vocabulary/pronunciation, alternative transcript, and second-pass repair benchmark envelopes are certified for Stage 8E. Live accent/noisy-room/native-mic/provider second-pass, production STT WER, real diarization error rate, real provider latency, live playback interruption, and Stage 17 TTS benchmarks remain deferred to Stage 17, Stage 34, or later explicitly approved live/native-lab stages.
- Stage 8E did not add live microphone capture, live STT, live TTS, provider calls, live search, Voice ID matching, understanding, routing, native UI redesign, protected execution, connector writes, raw-audio retention, or duplicate benchmark/listening/STT/language/pronunciation/repair engines.
- Stage 8F was required before Stage 9A and is now closed below.

Stage 8F status: PROVEN_COMPLETE.

Stage 8F proof update:

- Existing PH1.K, PH1.C, PH1.LISTEN, PH1.TTS, PH1.X, PH1.J, Stage 5 current-turn authority, Stage 7 activation context, Stage 8A transcript gate, Stage 8B endpoint/confidence gate, Stage 8C audio-scene evidence, and Stage 8D/8E benchmark carriers were inspected and crosswalked rather than rebuilt as duplicate output, playback, TTS, listen, interruption, or stale-render engines.
- `Stage8FOutputInteractionPacket`, `Stage8FOutputInteractionKind`, `Stage8FOutputInteractionDisposition`, and `Stage8FOutputInteractionAuthority` now provide deterministic output-interaction boundary carriers for barge-in, interruption, cancel, pause, resume, output stopped, stale-output quarantine, and TTS self-echo blocking.
- Stage 8F proves barge-in and interruption signals are boundary/control evidence only. They may mark output boundaries, request future output defer/cancel/pause behavior, block stale output, and emit audit/trace evidence, but cannot understand intent, answer, search, call providers, capture microphone audio, transcribe live audio, trigger Voice ID matching, authorize, emit TTS, route tools, connector-write, execute protected mutations, or commit turns by themselves.
- Cancel, pause, resume, and output-stopped state are tied to current session/turn/output identity and Stage 5 current-turn authority. They cannot mutate protected state, become business approval/rejection/action, reopen stale/cancelled/superseded turns, or render stale output as the current answer.
- Stale, cancelled, superseded, closed-session, and record-artifact output is quarantined before render. Old provider/tool/search/TTS results cannot render as current answers from Stage 8F state alone.
- Stage 8F preserves TTS self-echo continuity from Stage 8A/8C: self-echo remains blocking/non-user evidence and cannot create user turns, pass endpoint/confidence/protected-slot gates, trigger Voice ID matching, route tools/search/providers/TTS/protected execution, or connector-write.
- Stage 8F did not add live microphone capture, live STT, live TTS/playback, provider calls, live search, Voice ID matching, understanding, routing, native UI redesign, protected execution, connector writes, raw-audio retention, or duplicate output/playback/TTS/listen/interruption engines.
- Barge-in/interruption/cancel/pause/resume/stale-output/TTS-self-echo boundary proof is certified for Stage 8F. Live interruption latency, real audio stop latency, native playback control, native mic/speaker loop, live TTS, production barge-in, and full live listening/native-lab benchmarks remain deferred to Stage 17, Stage 34, or later explicitly approved live/native-lab stages.
- Stage 9A is ready to start. Broad Stage 8 foundation is closed for Stage 9A dependency purposes; live/native quality certification remains a later Stage 17/34 obligation.

Next if passed:

- Stage 9A - Voice ID Stack Reconciliation.

## Stage 9 - Voice ID Stack

Status: PROVEN_COMPLETE_BY_STAGE_9A

Stage 9A status: PROVEN_COMPLETE

Build:

- `PH1.VOICE.ID`;
- capture input;
- enrollment;
- consent;
- embedding;
- match;
- liveness/anti-spoof hooks;
- session binding;
- continuous verification;
- authority hint;
- privacy;
- trace and eval.

Voice ID output:

- confirmed user;
- probable user;
- unknown user;
- rejected speaker;
- liveness status;
- confidence;
- session binding;
- expiration.

Rules:

- Voice ID is identity evidence, not authority.
- Voice ID does not grant access by itself.
- Protected actions still require access, authority, simulation, execution gates, and law.
- Voice profiles require consent and must be revocable.
- Voice ID must not infer protected identity traits.

Proof:

- enrollment consent proof;
- match/non-match proof;
- liveness blocked/deferred proof;
- session binding proof;
- protected action still fails closed without access/authority/simulation.

Stage 9A proof update:

- Existing PH1.VOICE.ID contracts, `ph1_voice_id` OS runtime, PH1.F voice enrollment/sample/profile/sync/revocation storage, Stage 6 access context, Stage 7 activation references, Stage 8A-F voice/listen/output boundaries, consent, tenant, device-trust, provider-safety, KMS, audit, and storage surfaces were inspected and crosswalked rather than rebuilt as duplicate Voice ID, access, consent, audio/listen, or artifact engines.
- `Stage9VoiceIdentityPosturePacket`, `Stage9VoiceIdentityEvidence`, `Stage9VoiceIdentityPostureInput`, `Stage9VoiceIdentityInputKind`, `Stage9VoiceIdentityDisposition`, and `Stage9VoiceIdentityWorkAuthority` now provide the minimal runtime-owned Voice ID posture carrier in `runtime_ingress_turn_foundation.rs`.
- Stage 9A proves Voice ID can emit receipt-only speaker posture and inform access context, but cannot grant authority by itself. Unknown, low-confidence, wrong-speaker, multi-speaker, revoked-consent, revoked-artifact, cross-tenant, device-mismatch, stale-sample, stale-turn, unsafe Stage 8 signal, and record-artifact-only cases fail closed for protected authority.
- Stage 9A consumes only safe references: Stage 5 current-turn authority, Stage 6 access-context references, Stage 7 activation ids through Stage 8, Stage 8 final transcript metadata, or governed enrollment/artifact references. Partial transcript, VAD-only, endpoint-only, audio-scene-only, addressed-to-Selene-only, foreground-speaker-only, self-echo, background, record-artifact-only, stale, cancelled, superseded, or closed-session inputs cannot update Voice ID posture.
- Voice ID posture packets carry ids, hashes/receipts, reason codes, scope references, and audit references only. Source voice material, model vectors, secrets, provider metadata, and governed artifacts are not exposed to native clients, logs, benchmark packets, or public response paths by Stage 9A.
- Stage 9A did not add live microphone capture, live STT/TTS/playback, provider calls, live search, live Voice ID enrollment/matching, native UI redesign, understanding, routing, connector writes, protected execution, source-audio retention, or duplicate Voice ID/access/consent/artifact/listen engines.
- Voice ID carrier, consent/enrollment, posture, revocation/artifact, Stage 8 input-boundary, privacy/redaction, and no-execution proof is certified for Stage 9A. Live Voice ID FAR/FRR, ROC/EER, spoof resistance, room/noise robustness, cross-device matching, native enrollment UX, and production speaker-verification benchmarks remain deferred to later explicitly approved live/native-lab stages.
- Stage 10A is ready to start. Broad Stage 9 is complete enough for understanding and meaning reconstruction; live/native Voice ID quality certification remains a later Stage 34 or explicitly approved native-lab obligation.

Next if passed:

- Stage 10A - Understanding, Intent, Slot, And Meaning Reconstruction Foundation Reconciliation.

## Stage 10 - Universal Understanding And Perception Assist Spine

Status: PARTIALLY_BUILT

Build:

- `PH1.TRANSCRIPT`;
- `PH1.LANG`;
- `PH1.LANG.PACKET`;
- `PH1.LANG.SAFETY`;
- `PH1.SRL`;
- `PH1.NLP` / `PH1.N`;
- `PH1.D`;
- `PH1.X`;
- `PH1.PRON`;
- `PH1.PRUNE`;
- `PH1.DIAG`;
- `PH1.EXPLAIN`;
- `PH1.KNOW` hints where policy allows;
- spelling correction;
- phonetic name recovery;
- grammar repair;
- punctuation repair;
- broken English repair;
- multilingual grammar repair;
- business glossary protection;
- legal/financial term protection;
- name/date/amount/entity-slot preservation;
- `PH1.MEANING.HYPOTHESIS.LATTICE`;
- `PH1.BROKEN_LANGUAGE.REPAIR`;
- `PH1.SLANG.PUZZLE.RESOLVE`;
- `PH1.ACCENT_TO_INTENT.MAP`;
- `PH1.BAD_GRAMMAR.INTENT.RECOVER`;
- `PH1.PHONETIC_ENTITY.CANDIDATES`;
- `PH1.PROTECTED_SLOT.CONFIDENCE_GATE`;
- `PH1.CLARIFY.ONE_QUESTION_BEST`;
- `PH1.USER_CORRECTION.LEARN`;
- semantic parse;
- entity resolve;
- hypothesis generate/rank;
- intent resolve;
- emotional/context signal extraction where policy allows;
- frustration/stress/celebration cue handling;
- memory/persona candidate retrieval handoff;
- clarification and correction loops.

Purpose:

- repair spelling;
- repair grammar;
- repair punctuation and broken-language input without changing meaning;
- reconstruct likely meaning from awkward, scrambled, accented, slang-heavy, or mixed-language input;
- handle accent/STT damage;
- detect language and script;
- preserve same-language intent;
- resolve entities;
- understand user intent;
- ask clarification when needed;
- recognize emotional context without treating it as authority or identity;
- provide bounded explanations when requested.

Rules:

- Protected slots are never guessed.
- Language does not imply identity.
- Wrong-language STT mismatch blocks or clarifies.
- `Selene` versus similar-sounding words is handled by generic app-name/context lexicon behavior.
- Diagnostic/explanation engines are advisory and cannot execute.
- Emotional/context signals are advisory and must not infer protected identity traits.
- Memory/persona candidates may help context but must not override explicit current intent, evidence, authority, or safety.
- Spelling, grammar, and phonetic repair must not silently guess ambiguous names, dates, amounts, business objects, legal/financial terms, or protected slots.
- Meaning reconstruction may repair ordinary language, but protected slot confidence gates own names, money, dates, access, identity, legal terms, and business actions.
- When meaning is not clear enough, the clarification loop must ask the one best question instead of spraying multiple questions or guessing.
- Business glossary, legal, financial, and protected terms must preserve approved meaning and formatting.

Proof:

- spelling/phonetic repair proof;
- grammar repair proof;
- punctuation/broken-language repair proof;
- meaning hypothesis lattice proof;
- slang/accent/bad-grammar intent recovery proof;
- protected-slot confidence gate proof;
- one-question-best clarification proof;
- user correction learning proof;
- business glossary and protected-term preservation proof;
- multilingual route proof;
- assistant-name ambiguity proof;
- wrong-entity rejection proof;
- emotional/context signal boundedness proof;
- memory/persona candidate no-override proof;
- no protected-slot guessing.

Stage 10A proof update:

- Existing PH1.N/PH1.NLP, PH1.SRL, PH1.CONTEXT, PH1.LANG, PH1.PRON, PH1.X, PH1.D model-boundary context, Stage 5 current-turn authority, Stage 6 access context, Stage 7 activation references, Stage 8 final transcript/listening/output boundaries, and Stage 9 Voice ID posture were inspected and crosswalked rather than rebuilt as duplicate understanding, intent, slot, SRL, context, language, pronunciation, or repair engines.
- `Stage10UnderstandingPacket`, `Stage10UnderstandingInput`, `Stage10UnderstandingInputKind`, `Stage10VoicePostureContextKind`, `Stage10ProtectedSlotDisposition`, `Stage10ProtectedSlotUncertainty`, `Stage10MeaningReconstructionCandidate`, `Stage10UnderstandingDisposition`, and `Stage10UnderstandingWorkAuthority` provide the minimal runtime-owned understanding foundation in `runtime_ingress_turn_foundation.rs`.
- Stage 10A proves understanding can consume only Stage 5 current committed turn authority, Stage 6 non-executing access context, safe Stage 8 final transcript metadata, optional receipt-safe Stage 9 Voice ID posture, and governed repo-truth equivalents. Partial transcript, VAD-only, audio-scene-only, stale, cancelled, superseded, closed-session, record-artifact-only, and unsafe Voice ID posture inputs cannot update understanding.
- Intent, slot, semantic-role, language, pronunciation, ambiguity, and meaning-candidate outputs are advisory only. They cannot answer, search, call providers, capture live audio, transcribe live audio, trigger Voice ID matching, emit TTS, route tools, connector-write, authorize, execute protected mutations, or update memory/persona/emotion.
- Protected slots clarify or fail closed when uncertain. Meaning reconstruction is evidence-bounded and cannot invent names, dates, amounts, addresses, recipients, account/action identifiers, or authority-relevant facts. Ambiguity produces one bounded clarification handoff and cannot become execution or routing.
- Stage 10A did not add provider calls, live search, live microphone capture, live STT/TTS/playback, live Voice ID matching, router/capability registry work, tools/search, native UI redesign, memory/persona/emotion updates, protected execution, connector writes, raw-audio retention, or duplicate understanding/repair engines.
- Understanding/intent/slot/SRL/context/language/pronunciation/meaning-boundary proof is certified for Stage 10A. Full natural-language quality corpora, robust multi-turn reasoning, provider-assisted understanding, deeper grammar/spelling repair benchmarks, emotional-context product quality, and memory/persona candidate handoff quality remain deferred to their later owning stages.
- Stage 11A is ready to start. Broad Stage 10 remains partial for product-quality and deeper repair/evaluation work, but the non-executing understanding foundation is complete enough for router/capability-selection boundary reconciliation.

Next if passed:

- Stage 11A - Reasoning, Capability Registry, Router, And Tool-Selection Boundary Reconciliation.

## Stage 11 - Reasoning Orchestrator, Capability Registry, And Tool Route

Status: PROVEN_COMPLETE_BY_STAGE_11A

Stage 11A status: PROVEN_COMPLETE

Build:

- `PH1.REASON.ORCH`;
- `PH1.CAPABILITY.REGISTRY`;
- `PH1.PLAN.VERIFY`;
- `PH1.TOOL.ROUTE`;
- `PH1.LANG.TOOL.ROUTE`;
- `PH1.DOMAIN.EXPERT.ROUTE`;
- engine capability maps;
- blueprint/process routing;
- read/write route separation;
- math/science/history domain route candidates;
- task-to-model profile route candidate;
- route-level provider/cost budget preparation.

Rules:

- This is routing ownership, not a second brain.
- Raw text cannot run tools.
- Stage 11 may emit only `RouteCandidatePacket`, not an executable route.
- Stage 11 may attach a model profile candidate for the resolved task, but it may not call the model or provider.
- No tool, connector, task, search mutation, process action, or protected workflow can execute from Stage 11.
- Stage 12 must convert `RouteCandidatePacket` into `ApprovedExecutionPlan` or `FailClosedResponsePacket`.
- Capability registry knows ownership; it does not execute unauthorized actions.
- Non-English and mixed-language requests route correctly.
- Read-only tool routes and protected workflow routes remain separate.

Proof:

- public answer route;
- search route;
- file/doc/data route;
- record artifact route;
- protected workflow route to gates;
- connector read route;
- math/science/history domain route proof;
- task-to-model profile route candidate proof;
- route candidate cannot execute proof;
- no route from raw text.

Stage 11A proof update:

- Existing PH1.X, PH1.N, PH1.SRL, PH1.CONTEXT, PH1.ECM engine capability maps, PH1.SIMCAT, PH1.SIMFINDER, runtime law/governance, app ingress, and PH1.D inspect-only model-boundary context were inspected and crosswalked rather than rebuilt as duplicate reasoning, capability registry, router, tool-selection, search, simulation, or execution engines. `simulation_executor.rs` remained inspect-only for Stage 11A.
- `Stage11ReasoningRouterPacket`, `Stage11RouteCandidateInput`, `Stage11RouteCandidateKind`, `Stage11CapabilityMapDisposition`, `Stage11RouterDisposition`, and `Stage11ReasoningWorkAuthority` provide the minimal runtime-owned reasoning/router/capability candidate boundary in `runtime_ingress_turn_foundation.rs`.
- Stage 11A proves reasoning/router can consume only non-executing Stage 10 understanding plus safe Stage 5 current-turn authority, Stage 6 access context, Stage 7 activation context, Stage 8 final transcript metadata, and Stage 9 receipt-safe Voice ID posture. Partial transcript, VAD-only, audio-scene-only, stale, cancelled, superseded, closed-session, record-artifact-only, unsafe Voice ID posture, protected-slot uncertainty, and ambiguity requiring clarification cannot route to execution.
- Capability registry and capability-map state are declarative only. Missing, disabled, drifted, or tenant/workspace-mismatched capability maps fail closed. Public read-only candidates remain non-mutating; protected-action candidates remain inert until Stage 12; simulation candidates are inert catalog/finder handoffs and cannot dispatch or execute themselves.
- Reasoning/router/tool-selection packets cannot answer as final responses by themselves, search, call providers, call tools, capture microphone audio, transcribe live audio, emit TTS, connector-write, authorize, approve, dispatch, execute simulations, execute protected mutations, or update memory/persona/emotion.
- Stage 11A did not add provider calls, live search, live tools, connector writes, protected execution, simulation execution, Stage 12 approval gates, native UI redesign, live STT/TTS/playback, live Voice ID matching, memory/persona/emotion updates, raw-audio retention, provider-backed model/router behavior, or duplicate router/tool/simulation/search engines.
- Reasoning/router/capability/tool-selection boundary proof is certified for Stage 11A. Production reasoning quality, broad capability-routing accuracy, live tool routing, provider-backed planner quality, public search/tool execution, and protected execution benchmarks remain deferred to later owning stages.
- Stage 12A is ready to start. Broad Stage 11 remains partial for product-quality routing accuracy and live tool/provider planner benchmarks, but the non-executing route-candidate boundary is complete enough for protected simulation and runtime action gates.

Next if passed:

- Stage 12A - Simulation, Protected Execution, Approval, And Runtime Action Gate Reconciliation.

## Stage 12 - Runtime Risk, Authority, Simulation, Execution Gate, And Protected-Action Closure

Status: PARTIALLY_BUILT

Build:

- risk classifier after understanding/intent resolution;
- `PH1.OS`;
- `PH1.GOV`;
- `PH1.WORK`;
- `PH1.LEASE`;
- `PH1.SIMFINDER`;
- authority gate;
- `PH1.SIMULATION`;
- `simulation_executor`;
- execution gate;
- `PH1.LAW`;
- runtime governance;
- protected trace;
- conversion from `RouteCandidatePacket` to `ApprovedExecutionPlan` or `FailClosedResponsePacket`;
- protected model-use gate for provider/model/profile selection where user data, protected slots, regulated content, or business execution is involved;
- law/eval proof;
- fail-closed response path.

Rules:

- Public read-only work does not require simulation.
- Protected execution requires access, authority, simulation, execution gate, runtime law, audit, and idempotency.
- No simulation means no protected execution.
- No authority means no protected execution.
- Protected ambiguity fails closed or asks a deterministic clarification.
- Protected model use must respect access, authority, tenant policy, consent, provider budget, and audit before any provider call.

Protected examples:

- approve payroll;
- onboard employee;
- send access link;
- send reminders or messages to attendees;
- update database/account/permissions;
- connector write;
- activate/deprecate/rollback artifacts;
- provider promotion/demotion.

Proof:

- public read-only lane unaffected;
- resolved intent feeds risk classification;
- protected command blocked without simulation;
- protected command blocked without authority;
- approved execution plan proof;
- fail-closed response packet proof;
- protected model-use gate proof;
- mutation emits audit proof only after gates pass;
- runtime law final judgment proof.

Stage 12A status: PROVEN_COMPLETE

Stage 12A proof update:

- Existing PH1.OS, PH1.GOV, PH1.WORK, PH1.LEASE, PH1.SIMCAT, PH1.SIMFINDER, PH1.ACCESS, PH1.POLICY, PH1.TENANT, runtime law/governance/execution, `simulation_executor`, and PH1.J audit/proof-ledger surfaces were inspected and crosswalked rather than rebuilt as duplicate simulation, approval, runtime-law, access, policy, tenant, work, lease, audit, or execution engines.
- `Stage12ProtectedActionGatePacket`, `Stage12ProtectedActionGateInput`, `Stage12ProtectedActionDisposition`, `Stage12ProtectedActionWorkAuthority`, `Stage12ApprovedExecutionPlanPacket`, and `Stage12FailClosedResponsePacket` provide the minimal runtime-owned protected-action gate in `runtime_ingress_turn_foundation.rs`.
- Stage 12A consumes only inert Stage 11 protected-action or simulation candidates plus Stage 5 current-turn authority, Stage 6 access context, policy/tenant context, receipt-safe Stage 9 Voice ID posture where required, runtime-law/governance proof, idempotency/replay proof, work/lease proof, and PH1.J audit proof. Stage 11 reasoning traces, public read-only candidates, clarification candidates, missing/disabled/drifted capabilities, ambiguous or protected-slot-uncertain understanding, stale/cancelled/superseded/closed-session turns, and record-artifact-only turns fail closed.
- Protected actions require current turn/session, protected access readiness, policy allowance, tenant/workspace match, device trust where required, sufficient Voice ID posture where required, registered active unambiguous simulation/action identity, explicit approval or a policy-approved non-user-impacting path, runtime law allowance, idempotency/replay safety, work/lease validity, and PH1.J audit proof. Missing or failed gates emit fail-closed packets with deterministic reason codes.
- Public read-only candidates cannot mutate, send, post, purchase, delete, invite, approve, connector-write, or become protected execution by label drift. Simulation candidates cannot dispatch or execute themselves; only a Stage 12 approved simulation gate may release an internal simulation dispatch handoff.
- Replay, idempotency drift, lease mismatch, stale work ownership, and duplicate requests cannot double-execute or upgrade a denial into approved execution. Runtime law/governance cannot be skipped by router intent, simulation catalog/finder, Voice ID posture, or access hints.
- Stage 12A did not add live provider calls, live search, live external tool calls, connector writes, native UI behavior, live STT/TTS/playback, live Voice ID matching, memory/persona/emotion mutation, new business workflows, raw biometric/audio exposure, or duplicate simulation/approval/runtime-law/execution engines.
- Protected-action gate, approval, simulation-candidate, runtime-law, idempotency/replay, work/lease, PH1.J audit, and fail-closed proof are certified for Stage 12A. Production workflow correctness, live connector writes, real external tool execution, human approval UX, business workflow coverage, and end-to-end protected-action benchmarks remain deferred to later owning stages.
- Stage 13A is ready to start. Broad Stage 12 remains partial for live connector/business workflow product coverage, but the protected-action gate foundation is complete enough for public read-only search/tool quality to proceed without mutation authority.

Next if passed:

- Stage 13A - Public Read-Only Search, Source Evidence, Tool Route, And No-Mutation Boundary Reconciliation.

## Stage 13 - Search, Source, Image Evidence, And Public Tool Quality

Status: NEEDS_FINISHING

Build:

- `PH1.E`;
- `PH1.SEARCH`;
- `PH1.SEARCH.PUBLIC_WEB`;
- `PH1.SEARCH.NEWS`;
- `PH1.SEARCH.URL_FETCH`;
- `PH1.SEARCH.CACHE`;
- `PH1.SEARCH.OFFLINE_INDEX`;
- `PH1.SEARCH.REALTIME_VERTICALS`;
- `PH1.SEARCH.IMAGE_EVIDENCE`;
- `PH1.SEARCH.SHOPPING_PRODUCT`;
- `PH1.SEARCH.ACADEMIC`;
- `PH1.SEARCH.GOV_REGISTRY`;
- `PH1.SEARCH.COMPANY_FILING`;
- `PH1.SEARCH.QUERY.PLANNER`;
- `PH1.SEARCH.MULTI_QUERY`;
- `PH1.SEARCH.RESULT.RANKER`;
- `PH1.SEARCH.PAGE_EXTRACT`;
- `PH1.SEARCH.CITATION.BUILDER`;
- `PH1.PAGE.READER`;
- `PH1.PDF.READER`;
- `PH1.TABLE.EXTRACT`;
- `PH1.CITATION.EXTRACT`;
- `PH1.SOURCE.VERIFY`;
- `PH1.SOURCE.TRUST_SCORE`;
- `PH1.SOURCE.OFFICIAL_DETECT`;
- `PH1.SOURCE.FRESHNESS`;
- `PH1.SOURCE.CONTRADICTION`;
- `PH1.SOURCE.REJECT`;
- `PH1.SOURCE.CHIP`;
- `PH1.SOURCE.RANK`;
- `PH1.CLAIM.VERIFY`;
- `PH1.CITATION.BUILD`;
- `PH1.SEARCH.AUDIT`;
- `PH1.RESEARCH.OS`;
- `PH1.RESEARCH.QUESTION.DECOMPOSE`;
- `PH1.RESEARCH.SOURCE_PLAN`;
- `PH1.RESEARCH.SOURCE_OF_RECORD.ROUTE`;
- `PH1.RESEARCH.CITATION_GRAPH`;
- `PH1.RESEARCH.CONTRADICTION.MATRIX`;
- `PH1.RESEARCH.CLAIM_LEDGER`;
- `PH1.RESEARCH.EVIDENCE_TABLE`;
- `PH1.RESEARCH.REJECTED_SOURCE_LOG`;
- `PH1.RESEARCH.FRESHNESS.AS_OF`;
- `PH1.RESEARCH.JURISDICTION.ROUTE`;
- `PH1.RESEARCH.REPORT.BUILDER`;
- `PH1.RESEARCH.REVIEWER`;
- `PH1.RESEARCH.RED_TEAM`;
- `PH1.MATH.SOLVE`;
- `PH1.MATH.VERIFY`;
- `PH1.MATH.SYMBOLIC`;
- `PH1.MATH.NUMERIC`;
- `PH1.MATH.UNIT_CHECK`;
- `PH1.SCIENCE.SOURCE_VERIFY`;
- `PH1.SCIENCE.CALC_VERIFY`;
- `PH1.HISTORY.TIMELINE_VERIFY`;
- `PH1.HISTORY.SOURCE_CONTEXT`;
- search planning;
- `PH1.PREFETCH`;
- public web query planning;
- multi-query planning;
- result ranker;
- URL fetch;
- HTML/page extraction;
- PDF reader;
- table extraction;
- safe citation extraction;
- minimal reader extraction foundation for Search OS: safe HTML, PDF, Markdown, GitHub page, government page, company page, docs, and table extraction contracts;
- read-only reader extraction handoff to Stage 22 full file/document/data/vision implementation;
- connector/API search route contracts and fixture/mock packet proof for `ApiRoutePacket`, `ConnectorSearchPacket`, and `AppSourceChipPacket`;
- connector/API source-chip contract shape before live connector/app/API implementation;
- offline index;
- cache TTL;
- cache safety;
- freshness check;
- real-time vertical structured packets;
- source ranking;
- accepted/rejected source evidence;
- claim verification;
- claim-to-source mapping;
- research question decomposition;
- source-of-record route;
- citation graph;
- contradiction matrix;
- claim ledger;
- evidence table;
- rejected source log;
- research reviewer and red-team pass;
- math symbolic/numeric/unit verification;
- science source/calculation verification;
- history timeline/source-context verification;
- freshness state;
- contradiction state;
- source confidence state;
- source chips;
- image intent;
- image metadata;
- image safety;
- approved `SearchImagePacket`;
- news;
- deep research;
- `PH1.SHOPPING.SEARCH`;
- `PH1.PRODUCT.CARD`;
- `PH1.WEATHER`;
- `PH1.TIME`;
- `PH1.FINANCE`;
- `PH1.NEWS`;
- `PH1.ACADEMIC.SEARCH`;
- `PH1.REGISTRY.SEARCH`;
- `PH1.FILING.SEARCH`;
- `PH1.PRICE.COMPARE`;
- `PH1.REVIEW.SUMMARY`;
- `PH1.MERCHANT.LINK`;
- shopping/product research evidence packets;
- product card metadata;
- price comparison evidence;
- review summary evidence;
- merchant-link safety and disclosure;
- normal web search;
- official/government source preference;
- academic/source-of-record source search;
- company registry search;
- regulatory/filings search;
- local jurisdiction detection;
- source freshness check;
- contradiction detection;
- source quality scoring;
- "not enough evidence" behavior;
- time;
- weather;
- finance;
- flights;
- URL fetch and cite;
- document/photo/data read-only tool routes.

Rules:

- Finish existing search; do not rebuild from zero.
- Search is read-only.
- No source dump.
- No rejected source chips.
- No fake images.
- No image metadata as claim proof.
- No raw provider metadata in response or TTS.
- No real searched-name hardcoding.
- Search must output accepted evidence, rejected evidence, claim-source mapping, source-chip metadata, image evidence metadata, freshness state, contradiction state, and confidence state.
- Public web search must follow intent/routing, query plan, provider/API/connector, page fetch/read, source verification, claim verification, evidence packet, Write, presentation/source chips, and audit/eval.
- Direct URL/page/PDF reading must support safe HTML, PDF, Markdown, GitHub pages, government pages, company pages, docs, and tables where safe and supported.
- Stage 13 owns the minimal reader extraction foundation required for Search OS. Stage 22 later owns the full file/document/data/vision product implementation and must reuse this reader contract instead of creating a second reader path.
- Stage 13 may define connector/API search packet contracts and fixture/mock proof only. Live app/connector search, platform APIs, app auth, permissions, indexing, and read/write separation remain Stage 24.
- Stage 13 connector/API contract proof must not call live connectors, scrape restricted platforms, use private/company data, or imply live connector availability.
- Cached/offline search must expose freshness limits and must not pretend stale content is current.
- Real-time verticals must produce structured packets, not just links.
- Research OS must output claim ledger, citation graph, contradiction matrix, rejected source log, freshness/as-of state, jurisdiction route, and reviewer/red-team outcome where relevant.
- Search must investigate, not merely fetch: official, government, academic/source-of-record, registry, filing, and jurisdiction-aware sources must be preferred when relevant.
- If evidence is insufficient or contradictory, search must say so through the evidence packet instead of inventing certainty.
- Math answers must be checked by symbolic, numeric, or unit verification where applicable.
- Scientific claims need source-aware verification and calculation checks where applicable.
- History answers need timeline/source context and must show uncertainty when sources conflict.
- Shopping/product research is read-only unless explicitly routed into a protected purchase, account, payment, connector write, or merchant action.
- Merchant links must not bypass source ranking, disclosure, safety, or protected-action gates.
- Write may consume search evidence packets only. Write must not re-rank raw sources.

Proof:

- entity role answer proof;
- accepted-source ranking proof;
- wrong-source rejection proof;
- claim-source mapping proof;
- Research OS claim-ledger/citation-graph proof;
- contradiction matrix and rejected-source log proof;
- research reviewer/red-team proof;
- math solve/verify/unit-check proof;
- science source/calculation verification proof;
- history timeline/source-context proof;
- freshness/contradiction/confidence proof;
- official/government/source-of-record preference proof;
- local jurisdiction and filing/registry proof;
- insufficient-evidence proof;
- shopping/product-card evidence proof;
- price comparison and review summary proof;
- merchant-link safety proof;
- source chip proof;
- approved image packet proof;
- URL safety proof;
- public web baseline proof;
- URL/PDF/page/table reader proof;
- minimal reader extraction foundation proof;
- Stage 22 reader-handoff compatibility proof;
- connector/API search route contract mock proof;
- connector/API source-chip contract proof without live connector calls;
- cache/offline index freshness proof;
- real-time vertical structured packet proof;
- Search Build 1-3 and 7 proof;
- provider-off proof;
- controlled live provider proof only when explicitly allowed.

Stage 13A status: PROVEN_COMPLETE

Stage 13A proof update:

- Existing PH1.E, PH1.SEARCH, PH1.X, PH1.J, PH1.OS, PH1.GOV, PH1.ACCESS, PH1.POLICY, PH1.TENANT, PH1.SIMCAT, PH1.SIMFINDER, provider-control/provider-off carriers, runtime law/governance, and `web_search_plan` runtime/trust/synthesis/url/news/web_provider/write surfaces were inspected and crosswalked rather than rebuilt as duplicate search, source-evidence, citation, verifier, tool-route, or runtime-law engines. PH1.D and `simulation_executor.rs` remained inspect-only for Stage 13A.
- `Stage13PublicReadOnlyEvidencePacket`, `Stage13PublicReadOnlyEvidenceInput`, `Stage13PublicReadOnlyDisposition`, and `Stage13PublicReadOnlyWorkAuthority` provide the minimal runtime-owned public read-only source-evidence/no-mutation boundary in `runtime_ingress_turn_foundation.rs`.
- Stage 13A consumes only Stage 11 public read-only route candidates, Stage 12 public-read-only no-mutation proof, Stage 5 current-turn/session authority, Stage 6 access context as non-executing context, Stage 10 advisory understanding, runtime-law/no-mutation proof, PH1.J audit/proof references, and governed repo-truth equivalents. Protected-action candidates, simulation candidates, approved execution plans, stale/cancelled/superseded/closed-session turns, record-artifact-only turns, unsafe Voice ID posture, and ambiguous or protected-slot-uncertain understanding cannot become public read-only evidence authority.
- Public read-only/search/tool-route packets cannot mutate local or remote state, connector-write, send, post, purchase, delete, invite, schedule, approve, dispatch, execute simulations, execute protected actions, emit TTS, add native UI behavior, update memory/persona/emotion, call live providers, run live search, call live external tools, or bypass Stage 12 gates.
- Source evidence, citation, provenance, and verifier refs are bounded, auditable, PH1.J-linked, and secret-safe. Missing, stale, uncited, unverifiable, or secret-unsafe evidence fails closed; provider-off, missing-secret, and provider-failure paths cannot fabricate source chips or citations and preserve zero provider attempts/network dispatches for the build proof.
- Protected-action-like phrasing cannot be laundered through public search; authority-relevant ambiguity and unsafe identity posture remain blocked by earlier Stage 10-12 gates. Stage 13A did not add live provider calls, live web search, live external tool calls, connector writes, protected execution, native UI behavior, live STT/TTS/playback, live Voice ID matching, memory/persona/emotion mutation, new business workflows, or duplicate search/source/verifier engines.
- Public read-only route, source evidence, citation/provenance, provider-off/missing-secret, verifier, and no-mutation proof are certified for Stage 13A. Live search quality, production source ranking, real provider latency, broad web-search coverage, live connector/tool execution, human-facing citation UX, URL/PDF/page/table reader depth, cache/offline freshness, vertical structured packets, image/product evidence, connector/API route contracts, and deep Research OS release proof remain deferred to later owning Stage 13/14/22/24/34 slices.
- Stage 14A is ready to start. Broad Stage 13 remains partial, but the public read-only evidence/no-mutation boundary is complete enough for public answer composition and citation rendering to proceed without mutation authority.

Next if passed:

- Stage 14A - Public Answer Composition, Citation Rendering, And Evidence-Bound Response Reconciliation.

## Stage 14 - Web Search Enterprise Sublanes And Release Proof

Status: PARTIALLY_BUILT

Build:

- analytics: aggregates, confidence, consensus, numeric/unit/currency normalization;
- cache: L1/L2, TTL, cache safety;
- chunking and citations;
- contract validation: `contract_hash`, `packet_validator`, `reason_code_validator`, `turn_state_machine_validator`, and `idempotency_validator`;
- registry loading: `registry_loader`;
- competitive analysis: compare, pricing normalize, SWOT;
- diagnostics: error taxonomy, redaction, state trace;
- document/filing/PDF/OCR/table sublane;
- document/filing adapters: `sec_like`, `patent_like`, and `financials_like`;
- enterprise consistency/provenance/mode router;
- eval corpus, metrics, thresholds, reports;
- gap closers: claim confidence, freshness, injection defense, table rendering, transparency, unknown-first;
- learn/replay/rollback for search;
- merge/conflict/delta/internal context;
- multihop planning, cycle detection, hop budgets;
- full web homework research plans;
- `PH1.SEARCH.DEEP_RESEARCH`;
- `PH1.DEEP.RESEARCH`;
- `PH1.RESEARCH.MULTI_HOP`;
- Research OS release lane;
- source-of-record routing;
- citation graph;
- contradiction matrix;
- claim ledger;
- evidence table;
- rejected-source log;
- freshness as-of state;
- jurisdiction route;
- report builder;
- research reviewer;
- research red team;
- math/science/history benchmark packs;
- claim-by-claim evidence mapping;
- quote/snippet controls;
- parallel scheduling and merge order;
- parity: ambiguity, diversification, multi-query, reranking, stitching;
- perf/cost: budgets, concurrency, degrade, tiers, timeouts;
- planning: budget control, open selector, scoring, snippets, tie-break;
- prefetch: candidate building, prioritization, TTL, and idempotency dedupe hints;
- proxy route classification and redaction;
- realtime domains, freshness, TTL, adapters;
- realtime adapters: `finance`, `flights`, `weather`, and `generic_json`;
- regulatory jurisdiction/provenance/trust tier;
- structured adapters: academic, registry, filings, government, patents, pricing/products;
- structured adapter implementations: `company_registry`, `gov_dataset`, `patents`, and `pricing_products`;
- synthesis: boundary guard, citation validator, claim extractor, conflict handler, insufficiency gate;
- temporal as-of, diff, timeline;
- trust: official detector, spam signals, domain rules;
- web provider: Brave, fallback policy, health state, provider merge;
- provider implementations: `brave_adapter`, `openai_fallback`, and `gdelt`;
- vision/search media internals: `asset_ref`, `download`, `keyframes_ffmpeg`, `objects`, and `stt_google`;
- release evidence packs and replay certification using the Stage 2 minimal proof/replay envelope;
- explicit Stage 30 handoff for final release, promotion, rollback, regression, and competitor-parity certification;
- proof binaries: `web_search_turn`, `web_search_enterprise_turn`, `web_search_eval_report`, `web_search_release_evidence`, and `web_search_vision_turn`.

Rules:

- These sublanes support public/search intelligence; they do not bypass provider gates.
- Search release claims require replay/eval/release proof.
- Stage 14 release proof is search-lane evidence built on the Stage 2 minimal proof/replay envelope. Stage 14 does not replace Stage 30's full release, promotion, rollback, regression, and competitor-parity ownership.
- Stage 14 may prove search release readiness, but provider/model/prompt promotion and rollback still require Stage 30 release evidence.
- Research sublanes must support normal web, government, academic, registry, filings, jurisdiction-aware, and multi-hop investigation paths without bypassing provider gates.
- Deep research must investigate source plans, weak sources, official/source-of-record sources, contradictions, freshness, proven claims, uncertain claims, and rejected sources before building a report.
- Research release claims must prove citation accuracy, claim-to-source precision, official/source-of-record preference, contradiction detection, freshness, rejected-source precision, hallucinated-citation zero tolerance, and insufficient-evidence behavior.
- Domain benchmark packs must prove math verification, science calculation/source verification, and history timeline/source-context handling.
- Diagnostics/debug stay non-user-facing unless explicitly safe.

Proof:

- replay corpus proof;
- release evidence pack;
- Stage 2 minimal proof/replay envelope dependency proof;
- Stage 30 release/promotion/rollback handoff proof;
- full web homework research proof;
- Research OS release proof;
- deep research multi-hop/source-plan proof;
- Search Build 4 proof;
- citation graph/claim ledger/contradiction matrix proof;
- rejected-source log and freshness as-of proof;
- math/science/history benchmark proof;
- claim-by-claim evidence mapping proof;
- quote/snippet control proof;
- source/image/debug leak scans;
- provider budget proof;
- public tool parity proof.

Next if passed:

- Stage 15 - Write, Response, And TTS-Safe Text Engine.

## Stage 15 - Write, Response, And TTS-Safe Text Engine

Status: PARTIALLY_BUILT

Build:

- `PH1.WRITE`;
- evidence input;
- image evidence input;
- spelling/grammar/style bridge;
- structured writing;
- render;
- style;
- same-language output;
- persona-continuity style application;
- `PH1.SPOKEN_RESPONSE.COMPRESS`;
- `PH1.DISPLAY_RESPONSE.EXPAND`;
- `PH1.DISPLAY.SOURCE_AWARE`;
- spoken-answer compression;
- display-vs-speech adaptation;
- spoken response compression;
- display response expansion;
- TTS naturalization handoff;
- source-aware display handoff;
- warmth/professional-calm writing modes;
- final copyedit pass;
- research brief mode;
- direct-answer research mode;
- executive summary mode;
- evidence table mode;
- pros/cons mode;
- timeline mode;
- source comparison mode;
- uncertainty section;
- "what I checked" section;
- "what I rejected" section;
- recommendation section;
- next-step action plan mode;
- product comparison writing mode;
- merchant/seller caveat writing mode;
- study/tutor explanation mode;
- Socratic step rendering;
- quiz feedback rendering;
- source chips;
- image card render packets;
- clarification render;
- professional output;
- TTS-safe text;
- write trace/eval.

Must support:

- direct answers;
- headers;
- paragraphs;
- bullets;
- numbered steps;
- tables;
- warnings and limitations;
- source-aware claims;
- source pills;
- image-card placement;
- concise spoken answer variant;
- calm correction/clarification wording;
- polished research work product;
- product research work product;
- study/tutor work product;
- clean TTS.

Rules:

- Write polishes.
- Write does not invent facts.
- Write does not change protected slots.
- Write does not turn uncertainty into certainty.
- Write does not convert rejected evidence into proof.
- Write consumes `EvidencePacket` and `ImageEvidencePacket`; it must not consume raw provider output or raw source lists.
- Write does not re-rank sources, choose new source chips, or promote image metadata into factual proof.
- Until Stage 21 and Stage 29 provide live producers, Write may consume default empty `MemoryContextPacket`, `PersonaContinuityPacket`, and `EmotionalGuidancePacket` only.
- Stage 21 and Stage 29 must later wire live memory/persona/emotion producers into these existing Write consumers without changing Write into a second reasoning layer.
- Persona and emotional guidance may shape tone, length, and spoken/display form only; they must not change facts, evidence status, protected slots, or authority decisions.
- Spoken answer compression must preserve meaning, uncertainty, and safety.
- Spoken answers should often be shorter than display answers while preserving meaning, uncertainty, safety, and protected refusal content.
- Display answers may expand into sources, tables, evidence, contradiction handling, and research details that TTS must not speak.
- Final copyedit improves spelling, grammar, punctuation, and readability without changing facts, evidence, protected slots, or uncertainty.
- Research presentation must preserve evidence mapping, rejected evidence, uncertainty, and source-chip integrity.
- Product comparison writing must preserve merchant-link safety, source confidence, price freshness, review-summary uncertainty, and no-purchase-without-protected-routing rules.
- Study/tutor writing must teach without silently doing the user's assessment work when policy or mode requires Socratic guidance.
- TTS text must stay clean and natural.

Proof:

- structured answer proof;
- source/image metadata excluded from response text and TTS;
- same-language output proof;
- default-empty memory/persona/emotion packet contract proof;
- persona style-only proof;
- spoken compression meaning-preservation proof;
- display-vs-speech optimization proof;
- final copyedit no-meaning-change proof;
- research presentation mode proof;
- product comparison writing proof;
- study/tutor writing proof;
- no confidence/debug/source dump leak;
- protected refusal text preserved.

Next if passed:

- Stage 16 - Presentation Contracts.

## Stage 16 - Presentation Contracts

Status: NEEDS_BUILDING

Build:

- `PH1.PRESENTATION.CONTRACTS`;
- answer blocks;
- heading blocks;
- paragraph blocks;
- bullet list blocks;
- numbered list blocks;
- table blocks;
- source chip blocks;
- accepted-source chip blocks;
- citation blocks;
- as-of/freshness blocks;
- image card blocks;
- evidence table blocks;
- research report blocks;
- claim-ledger blocks;
- citation-graph blocks;
- contradiction-matrix blocks;
- source comparison blocks;
- timeline blocks;
- uncertainty blocks;
- checked-source blocks;
- rejected-source blocks;
- weak/rejected-source log blocks;
- recommendation blocks;
- next-step action-plan blocks;
- interactive app card blocks;
- product card blocks;
- merchant-link blocks;
- interactive table blocks;
- interactive chart blocks;
- chart export blocks;
- canvas share/export/version blocks;
- study/tutor step blocks;
- quiz/practice blocks;
- generated video artifact blocks;
- code/callout/warning blocks;
- divider blocks;
- display-only metadata;
- TTS-only text;
- display hash;
- TTS hash.

Purpose:

- stop flattening rich answers into boring plain text;
- make native clients render beautiful output safely;
- preserve source/image/TTS separation.

Rules:

- Display text and spoken text may differ.
- Display-only metadata is not spoken.
- Source chips, citation metadata, image/product metadata, rejected-source logs, and debug/provider fields are display-only unless explicitly transformed into clean approved `tts_text`.
- TTS-only text is not rendered as debug UI.
- Unsupported block types must safe-fallback.

Proof:

- block ordering proof;
- display/TTS separation proof;
- source chip and image card block proof;
- research presentation block proof;
- citation/as-of/claim-ledger/contradiction/rejected-source block proof;
- interactive app/product/data/canvas/study/video block proof;
- no raw HTML/provider/debug leakage.

Next if passed:

- Stage 17 - Speech/TTS Output And Playback Control.

## Stage 17 - Speech/TTS Output And Playback Control

Status: PARTIALLY_BUILT

Build:

- `PH1.TTS`;
- `PH1.TTS.NATURALIZE`;
- `PH1.TTS.PAUSE.PROSODY`;
- `PH1.TTS.EMPHASIS`;
- `PH1.TTS.PRONUNCIATION.MEMORY`;
- TTS transport packet;
- TTS-safe text hash proof;
- playback state;
- interruption and barge-in cancellation;
- mic self-echo guard;
- same-language TTS continuity;
- speech pacing;
- TTS naturalization;
- pause prosody;
- emphasis control;
- bounded prosody control;
- emotional tone selection;
- volume/energy profile;
- pronunciation memory;
- name pronunciation;
- business glossary pronunciation;
- TTS/voice model policy;
- voice model profile selection;
- speech style per user/context;
- spoken-answer compression playback;
- fallback behavior;
- no raw audio retention by default.

Rules:

- TTS speaks exact approved `tts_text`.
- TTS does not rewrite, translate, summarize, or add claims unless explicitly routed and approved.
- TTS does not speak source chips, image metadata, debug, raw URLs, or provider JSON.
- TTS playback can be interrupted.
- Until Stage 21 and Stage 29 provide live producers, TTS may consume default empty `MemoryContextPacket`, `PersonaContinuityPacket`, `EmotionalGuidancePacket`, and `ProsodyControlPacket` only.
- Stage 21 and Stage 29 must later wire live pronunciation/persona/emotion/prosody producers into these existing TTS consumers without allowing TTS to rewrite, translate, summarize, or add claims.
- Prosody may express approved tone, but it must not add claims, imply completed actions, or silently change language.
- Pronunciation memory and glossary pronunciation require provenance and consent where policy requires.
- Voice model selection must follow approved model profile, provider gate, budget, language, privacy, and audit policy.
- TTS naturalization may adjust pacing, pause, emphasis, and prosody only through approved `tts_text` and `ProsodyControlPacket`; it must not rewrite facts, imply action completion, or speak display-only evidence metadata.

Proof:

- exact text hash proof;
- default-empty memory/persona/emotion/prosody packet contract proof;
- prosody boundedness proof;
- pronunciation memory/glossary proof;
- voice model policy proof;
- TTS naturalization/pause/emphasis proof;
- spoken compression playback proof;
- interruption proof;
- fallback proof;
- self-echo proof;
- no raw audio retention proof.

Next if passed:

- Stage 18 - Adapter, Protocol, And Rich Transport.

## Stage 18 - Adapter, Protocol, And Rich Transport

Status: PARTIALLY_BUILT

Build:

- `PH1.ADAPTER.TRANSPORT`;
- HTTP adapter;
- gRPC adapter;
- exact HTTP binary `http_adapter.rs`;
- exact gRPC binary `grpc_adapter.rs`;
- Desktop wake life binary;
- exact Desktop wake binary `desktop_wake_life.rs`;
- Desktop mic producer;
- exact Desktop mic producer `desktop_mic_producer.rs`;
- Desktop capture proof `desktop_capture_bundle_valid.rs`;
- adapter UI asset helper `app_ui_assets`;
- response packet preservation;
- session state transport;
- Voice ID state transport;
- access/authority state transport;
- presentation block transport;
- source chip transport;
- citation/as-of metadata transport;
- claim-ledger/evidence-table transport;
- contradiction/rejected-source transport;
- direct URL/PDF/page reader evidence transport;
- cached/offline freshness-state transport;
- real-time vertical structured packet transport;
- image card transport;
- interactive app card transport;
- product card transport;
- interactive table/chart transport;
- canvas share/export/version transport;
- visual agent watch-state transport;
- generated/edited video artifact transport;
- TTS transport;
- reason codes;
- health/readiness;
- CLI/admin route boundaries where relevant.

Rules:

- Adapter preserves packets; it does not invent answers.
- Adapter does not choose images.
- Adapter does not authorize actions.
- Adapter does not hold or expose provider keys to clients.
- Adapter does not flatten structured output unless safe fallback is required.

Proof:

- HTTP transport proof;
- gRPC transport proof;
- structured block preservation proof;
- source chip/image card preservation proof;
- citation/as-of/claim-ledger/contradiction/rejected-source preservation proof;
- URL/PDF/page reader evidence preservation proof;
- cache/offline freshness-state preservation proof;
- real-time vertical packet preservation proof;
- interactive app/product/data/canvas/video packet preservation proof;
- visual agent watch-state preservation proof;
- TTS/display hash preservation proof;
- no secret/provider metadata leak.

Next if passed:

- Stage 19 - Desktop Native Runtime And Renderer.

## Stage 19 - Desktop Native Runtime And Renderer

Status: PARTIALLY_BUILT

Desktop platforms:

- Mac Desktop;
- Windows Desktop.

Certification scope:

- Stage 19 certifies the core Desktop runtime and renderer: activation, wake, explicit mic, record boundary, session state, source/image/presentation packets, TTS, protected fail-closed display, generic presentation-block rendering, and safe fallback for unsupported future blocks.
- Product-specific blocks whose engines are built after Stage 19 must be recertified in Stage 33 and Stage 34. Stage 19 must not claim final product-surface parity for stages that are not built yet.

Build:

- `PH1.DESKTOP.ACTIVATION.CLIENT`;
- `SeleneMacDesktopRuntimeBridge.swift`;
- `DesktopSessionShellView.swift`;
- Windows Desktop activation client;
- Windows Desktop wake client;
- Windows Desktop explicit mic client;
- Windows Desktop session client;
- Windows Desktop voice capture client;
- Windows Desktop Voice ID client;
- Windows Desktop record client;
- Windows Desktop rich renderer;
- Windows Desktop TTS client;
- Desktop wake client;
- explicit mic client;
- session client;
- Voice ID client;
- record client;
- rich renderer;
- source chips;
- image cards;
- presentation blocks;
- interactive app cards;
- product cards;
- interactive tables/charts;
- canvas share/export/version UI;
- study/tutor UI;
- video artifact cards;
- TTS client;
- protected fail-closed display;
- trace visibility;
- safe loading/error/deferred states.

Rules:

- Desktop is not the brain.
- Desktop does not call providers.
- Desktop does not rank sources.
- Desktop does not choose images.
- Desktop does not authorize actions.
- Desktop renders approved packets only.
- Desktop product-specific blocks introduced after Stage 19 may be represented by contract/fallback rendering at Stage 19, but their full behavior and visuals must be recertified after their owning stages exist.
- Stage 33 owns final Desktop native/runtime parity for extended product blocks and must not treat Stage 19 core renderer proof as full product certification.

Proof:

- wake UI proof;
- Mac Desktop activation/wake/mic/record/TTS/render proof;
- Windows Desktop activation/wake/mic/record/TTS/render proof where Windows workspace exists, otherwise Stage 1 must record Windows Desktop as planned/missing repo surface;
- `desktop_wake_life.rs` proof;
- explicit mic proof;
- `desktop_capture_bundle_valid.rs` proof;
- record button does not live-chat proof;
- source pill visual proof;
- real approved image card render proof;
- structured block render proof;
- interactive app/product/data/canvas/study/video contract/fallback render proof before those product stages exist;
- Stage 33 extended Desktop renderer recertification dependency recorded;
- protected fail-closed display proof;
- Windows and Mac protected fail-closed display parity proof;
- xcodebuild proof.

Next if passed:

- Stage 20 - Mobile Native Runtime And Renderer.

## Stage 20 - Mobile Native Runtime And Renderer

Status: PARTIALLY_BUILT

Mobile activation:

- side button / push-to-talk;
- iPhone has no always-listening wake detector;
- Android supports wake word plus explicit mic/button where platform policy allows.

Certification scope:

- Stage 20 certifies the core mobile runtime and renderer: iPhone side-button/push-to-talk, Android wake/explicit mic where platform policy allows, record boundary, session state, source/image/presentation packets, TTS, protected fail-closed display, generic presentation-block rendering, and safe fallback for unsupported future blocks.
- Product-specific blocks whose engines are built after Stage 20 must be recertified in Stage 33 and Stage 34. Stage 20 must not claim final product-surface parity for stages that are not built yet.

Build:

- `PH1.IPHONE.ACTIVATION.CLIENT`;
- `SeleneIPhoneApp.swift`;
- `SessionShellView.swift`;
- `PH1.ANDROID.ACTIVATION.CLIENT`;
- Android wake client;
- Android explicit mic client;
- Android session client;
- Android voice capture client;
- Android Voice ID client;
- Android record client;
- Android rich renderer;
- Android TTS client;
- side-button client;
- session client;
- voice capture client;
- Voice ID client;
- record client;
- rich renderer;
- source chips;
- image cards;
- presentation blocks;
- interactive app cards;
- product cards;
- interactive tables/charts;
- canvas share/export/version UI;
- study/tutor UI;
- video artifact cards;
- TTS client;
- protected fail-closed display;
- trace visibility;
- native build and proof harness.

Rules:

- iPhone is explicit activation only.
- iPhone does not run always-listening wake.
- Android wake candidate behavior follows the same Stage 7 wake laws as Desktop: wake opens/resumes attention only and does not reason, identify, authorize, search, or execute.
- iPhone still uses Voice ID, access, and authority.
- Mobile clients remain client/renderers only.
- Mobile product-specific blocks introduced after Stage 20 may be represented by contract/fallback rendering at Stage 20, but their full behavior and visuals must be recertified after their owning stages exist.
- Stage 33 owns final mobile native/runtime parity for extended product blocks and must not treat Stage 20 core renderer proof as full product certification.

Proof:

- side-button packet proof;
- no wake detector proof;
- Android wake candidate no-execution proof;
- Android explicit mic proof;
- Voice ID/session binding proof;
- record button separation proof;
- source/image/presentation render proof;
- interactive app/product/data/canvas/study/video contract/fallback render proof before those product stages exist;
- Stage 33 extended mobile renderer recertification dependency recorded;
- protected fail-closed display proof;
- Xcode build proof;
- Android build proof where Android workspace exists, otherwise Stage 1 must record Android as planned/missing repo surface.

Next if passed:

- Stage 21 - Project, Memory, Persona, Workspace, And Context.

## Stage 21 - Project, Memory, Persona, Workspace, And Context

Status: PARTIALLY_BUILT

Build:

- `PH1.M`;
- `PH1.PERSONA`;
- `PH1.CONTEXT`;
- `PH1.PROJECT.CONTEXT`;
- `PH1.WORKSPACE`;
- `PH1.TENANT.ISOLATION`;
- memory query;
- memory remember request;
- memory forget request;
- `PH1.MEMORY.TIMELINE`;
- `PH1.MEMORY.EPISODIC`;
- `PH1.MEMORY.SEMANTIC`;
- `PH1.MEMORY.PREFERENCE`;
- `PH1.MEMORY.PROJECT_SCOPE`;
- `PH1.MEMORY.CONFLICT.RESOLVE`;
- `PH1.MEMORY.CONFIDENCE`;
- `PH1.MEMORY.PROVENANCE`;
- `PH1.MEMORY.EXPIRY`;
- `PH1.MEMORY.USER_VISIBLE`;
- `PH1.MEMORY.FORGET.PROOF`;
- `PH1.MEMORY.CORRECTION.LEARN`;
- episodic memory;
- semantic memory;
- preference memory;
- project memory;
- relationship/interaction memory;
- correction memory;
- pronunciation memory;
- task/process memory;
- forbidden/suppressed memory;
- memory confidence, expiry, conflict resolution, provenance, and recall explanation;
- memory verification before recall;
- "I may be remembering this wrong" behavior;
- memory source/provenance display;
- memory decay/expiry policy;
- user-visible memory controls;
- cross-device continuity;
- correction learning;
- pronunciation/name memory;
- project-only context;
- workspace access boundaries;
- persona/style hints;
- personality profile;
- tone profile;
- formality profile;
- humor preference;
- verbosity preference;
- user correction style;
- device/context style differences;
- session-to-session continuity;
- knowledge graph context where evidence-backed;
- live producers for `MemoryContextPacket` and `PersonaContinuityPacket` consumed earlier by Write/TTS default-empty packets.

Rules:

- Memory is ledger-first and identity scoped.
- Selene remembers what is permitted, useful, scoped, provenanced, and recoverable; Selene does not blindly remember everything.
- Project memory stays in project.
- Persona changes style, not truth or safety.
- Context helps understanding but cannot override current explicit intent.
- Memory recall must carry provenance, confidence, consent state, and expiry where applicable.
- Memory must verify recall candidates before use and must expose uncertainty when confidence is low.
- Memory conflicts must clarify, safe-degrade, or prefer current explicit user intent.
- User-facing memory controls must support "what do you remember about me?", "forget that", and "why did you remember that?" where policy allows.
- Forbidden/suppressed memory must not be recalled into prompts, Write, TTS, tools, or clients.
- Relationship/persona continuity must improve interaction style only; it must not create false intimacy or unsupported claims.
- Workspace boundaries cannot be bypassed.
- Custom instructions cannot override safety, access, provider, or execution law.
- Stage 21 must replace default-empty memory/persona packets with live governed producers for Write/TTS consumers.

Proof:

- memory query proof;
- remember/forget/suppression proof;
- memory timeline proof;
- memory false-recall/stale-suppression/project-leakage benchmark proof;
- memory forget proof;
- memory provenance/confidence/expiry proof;
- memory verification-before-recall proof;
- user-visible memory control proof;
- memory conflict resolution proof;
- forbidden/suppressed memory no-recall proof;
- project boundary proof;
- persona style-only proof;
- live memory/persona producer wiring proof;
- session-to-session continuity proof;
- context trim proof.

Next if passed:

- Stage 22 - File, Document, Data, Vision, OCR, And Media Understanding.

## Stage 22 - File, Document, Data, Vision, OCR, And Media Understanding

Status: PARTIALLY_BUILT

Build:

- `PH1.FILE`;
- `PH1.DOC`;
- `PH1.SUMMARY`;
- `PH1.DATA`;
- `PH1.VISION`;
- `ph1vision_media`;
- vision internals: `asset_ref`, `download`, `keyframes_ffmpeg`, `objects`, and `stt_google`;
- full implementation of the Stage 13 reader extraction foundation for file/document/data/vision product lanes;
- document extraction;
- PDF text/tables;
- OCR;
- photo understanding;
- screenshot/diagram understanding;
- video/keyframe understanding where enabled;
- spreadsheet/CSV/JSON/table analysis;
- chart-ready outputs;
- `PH1.DATA.SANDBOX`;
- `PH1.DATA.VISIBLE_ANALYSIS`;
- `PH1.DATA.INTERACTIVE.TABLE`;
- `PH1.DATA.INTERACTIVE.CHART`;
- `PH1.DATA.CHART.EXPORT`;
- sandboxed data-analysis workspace;
- visible analysis steps where safe;
- interactive table artifact;
- interactive chart artifact;
- chart export artifact;
- artifact provenance.

Rules:

- Uploaded content has provenance.
- Stage 22 must consume the Stage 13 reader extraction foundation where it touches URL/PDF/page/table reading. It must not create a second reader path that bypasses Search OS evidence, citation, source safety, provider, or trace contracts.
- Visual/document analysis is read-only unless routed to protected workflow.
- No fake calculations.
- No hidden execution.
- Data sandbox work must be bounded, visible, replayable, and unable to execute protected mutations or leak local secrets.
- Interactive charts/tables are artifacts with provenance, not hidden execution results.
- Document/data/vision outputs go through Write/Presentation.
- Generated images are separate from sourced/observed visual evidence.

Proof:

- document extraction proof;
- Stage 13 reader extraction foundation compatibility proof;
- table/data proof;
- data sandbox proof;
- visible analysis proof;
- interactive table/chart and chart export proof;
- OCR/photo proof;
- unsupported/unsafe file safe-degrade proof;
- provenance proof.

Next if passed:

- Stage 23 - Canvas, Artifacts, And Artifact Governance.

## Stage 23 - Canvas, Artifacts, And Artifact Governance

Status: NEEDS_BUILDING

Build:

- `PH1.CANVAS`;
- `PH1.ART`;
- `device_artifact_sync`;
- artifact create/open/update;
- versioning;
- document canvas;
- code-plan canvas;
- build-plan canvas;
- meeting-notes canvas;
- `PH1.CANVAS.SHARE`;
- `PH1.CANVAS.EXPORT`;
- `PH1.CANVAS.VERSION_RESTORE`;
- `PH1.CANVAS.INLINE_FEEDBACK`;
- `PH1.CANVAS.WEB_PREVIEW_GUARD`;
- canvas sharing controls;
- canvas export formats;
- version restore;
- inline feedback/comments;
- guarded web preview for canvas artifacts;
- artifact ledger handoff;
- device artifact sync handoff;
- artifact activation/deprecation/rollback dependency on governance.

Rules:

- Canvas is workspace, not authority.
- Canvas edits are versioned.
- Canvas sharing/export must respect workspace, tenant, privacy, retention, and access policy.
- Version restore is a governed artifact operation and must preserve audit history.
- Web preview must not bypass provider, network, secret, content, or protected-execution gates.
- Canvas cannot trigger protected execution alone.
- Artifact activation/deprecation/rollback requires governance/law where authoritative.

Proof:

- artifact create/update proof;
- version proof;
- canvas share/export proof;
- canvas version-restore proof;
- inline feedback proof;
- guarded web preview proof;
- canvas to chat and chat to canvas handoff;
- no protected execution from canvas alone.

Next if passed:

- Stage 24 - Agent, Apps, Connectors, Tasks, And Scheduling.

## Stage 24 - Agent, Apps, Connectors, Tasks, And Scheduling

Status: NEEDS_BUILDING

Build:

- `PH1.AGENT`;
- `PH1.APPS.CONNECTORS`;
- `PH1.TASKS`;
- `PH1.SCHED`;
- `PH1.APPS.DIRECTORY`;
- `PH1.APPS.SDK`;
- `PH1.APPS.MCP`;
- `PH1.APPS.INTERACTIVE.CARDS`;
- `PH1.APPS.ACTION.CONSTRAINTS`;
- `PH1.APPS.SYNCED.KNOWLEDGE`;
- `PH1.SEARCH.CONNECTOR_ROUTE`;
- `PH1.CONNECTOR.READ`;
- `PH1.CONNECTOR.WRITE`;
- `PH1.APP.AUTH`;
- `PH1.APP.PERMISSIONS`;
- `PH1.APP.INDEX`;
- `PH1.APP.SEARCH`;
- `PH1.APP.SOURCE.CHIP`;
- `PH1.SEARCH.API_ROUTE`;
- `PH1.API.CAPABILITY.REGISTRY`;
- `PH1.API.AUTH`;
- `PH1.API.RATE_LIMIT`;
- `PH1.API.RIGHTS_POLICY`;
- `PH1.API.SOURCE_MAP`;
- `PH1.AGENT.VISUAL_BROWSER`;
- `PH1.AGENT.WATCH_MODE`;
- `PH1.AGENT.WEBSITE_BLOCKLIST`;
- `PH1.AGENT.ACTION_SUPERVISION`;
- connector registry;
- app directory registry;
- app SDK contract;
- MCP/custom app contract;
- interactive app card lifecycle;
- synced app knowledge boundary;
- live implementation of Stage 13 connector/API search route contracts;
- read connector route;
- read-only connector search;
- connector app source chips;
- write connector route with protected gates;
- platform API route for GitHub, YouTube, Reddit, X/Twitter, Meta/Facebook, LinkedIn, and other approved providers;
- API capability registry;
- API auth/rate-limit/rights policy;
- API source map;
- agent visual browser session;
- watch mode and user takeover;
- website blocklist and navigation policy;
- action supervision and approval checkpoints;
- reminder scheduler;
- recurring tasks;
- future checks;
- scheduled web checks with provider budgets;
- app/calendar/email/document connector boundaries.

Rules:

- Agent cannot run from raw text.
- Read-only connector work is separate from mutation.
- Stage 24 must consume and complete the connector/API route contracts proven with mock packets in Stage 13. It must not create a second connector/API search packet shape.
- Connector search may reference private/company data only with app auth, app permissions, tenant/workspace access, and audit.
- Platform-specific data should use official API, authorized connector, public indexed web, or access-unavailable response; do not scrape restricted platforms.
- Apps and MCP tools are capability endpoints; they are not Selene's brain and cannot rank sources, authorize actions, or bypass runtime law.
- Interactive app cards render approved app packets only and cannot execute hidden mutations.
- Visual browser/watch mode must keep user-visible state, blocklisted website rules, action supervision, and protected approval gates.
- Connector writes require access, authority, simulation, execution gate, law, and audit.
- Scheduled work re-enters through ingress and does not bypass gates.
- No background tool fanout.

Proof:

- read connector proof;
- Stage 13 connector/API contract compatibility proof;
- connector search/index/source-chip proof;
- app auth/permissions proof;
- platform API route proof;
- API rate-limit/rights-policy/source-map proof;
- Search Builds 5-6 proof;
- app directory/SDK/MCP contract proof;
- interactive app card proof;
- synced app knowledge boundary proof;
- visual browser/watch mode proof;
- website blocklist proof;
- action supervision proof;
- write connector blocked without authority;
- reminder scheduling proof;
- recurring task proof;
- scheduled provider-budget proof;
- protected scheduled mutation fails closed.

Next if passed:

- Stage 25 - Broadcast, Delivery, Reminders, And Message Lifecycle.

## Stage 25 - Broadcast, Delivery, Reminders, And Message Lifecycle

Status: PARTIALLY_BUILT

Build:

- `PH1.BCAST`;
- `PH1.DELIVERY`;
- `PH1.REM`;
- message compose/send lifecycle;
- broadcast draft/deliver/ack/defer/retry/expire;
- delivery provider attempts;
- SMS/email/WhatsApp/WeChat delivery boundaries where supported;
- reminder timing mechanics;
- follow-up timing handoff;
- delivery failure handling;
- message audit and idempotency.

Rules:

- Message sending is protected when external delivery or mutation occurs.
- `PH1.REM` owns timing, not message content.
- `PH1.BCAST` owns lifecycle, not provider transport.
- `PH1.DELIVERY` owns delivery attempt truth, not content truth.
- Delivery never bypasses access, authority, simulation, law, or audit.

Proof:

- draft-only proof;
- delivery blocked without authority/simulation;
- retry/expire proof;
- reminder timing proof;
- delivery attempt audit proof.

Next if passed:

- Stage 26 - Business Process Actions, Link, Onboarding, Position, And Capability Requests.

## Stage 26 - Business Process Actions, Link, Onboarding, Position, And Capability Requests

Status: PARTIALLY_BUILT

Build:

- `PH1.PROCESS.INTENT`;
- `PH1.LINK`;
- `PH1.ONB`;
- `PH1.POSITION`;
- `PH1.CAPREQ`;
- `PH1.IDENTITY.RESOLVE`;
- link invite draft;
- link deliver invite via broadcast/delivery;
- link open/activate;
- onboarding session;
- business onboarding;
- requirements schema management;
- position lifecycle;
- access instance create/compile;
- capability request lifecycle;
- process trace/eval.

Examples:

- send an onboarding link;
- onboard a user;
- create access instance;
- draft welcome email;
- schedule onboarding reminder;
- manage position requirements;
- request or approve a capability.

Rules:

- Process actions are protected where they mutate state, send messages, or change access.
- Identity must be resolved.
- Access must be checked.
- Simulation is required before execution.
- User approval is required where policy says so.
- Legacy do-not-wire simulations remain blocked.

Proof:

- link draft proof;
- link delivery blocked without gates;
- onboarding state proof;
- position schema proof;
- access instance create proof;
- CAPREQ lifecycle proof.

Next if passed:

- Stage 27 - Record Mode And Meeting Recording Product.

## Stage 27 - Record Mode And Meeting Recording Product

Status: PARTIALLY_BUILT

Build:

- `PH1.RECORD`;
- record state: idle, recording, paused, stopped, processing, complete, failed;
- recording session identity separate from live chat;
- meeting/voice-note audio artifact;
- adapter upload/chunk transport for record artifacts;
- artifact ledger handoff for record artifacts;
- consent and privacy state;
- raw audio retention policy;
- transcript artifact;
- translation when requested;
- speaker separation/labels where available and policy-approved;
- meeting summary;
- main-points extraction;
- decisions extraction;
- action-item extraction;
- attendee metadata where available;
- reminder/task/email draft generation;
- document and canvas handoff;
- protected gate before sending, scheduling, inviting, onboarding, or mutating anything.

Record mode wiring:

```text
Record button
-> RecordSessionPacket
-> AudioArtifactPacket
-> adapter upload/chunk transport
-> artifact ledger
-> PH1.RECORD processing
-> DOC/DATA/CANVAS handoff
-> protected draft actions only
```

Rules:

- The record button is a recording/artifact workflow, not a voice-chat mode switch.
- Record mode captures and processes after completion.
- Record mode must not answer live like assistant chat.
- Record mode must not trigger tools from partial recording.
- Extracted reminders, attendee messages, onboarding steps, or connector sends remain drafts until protected gates pass.
- Raw audio is not retained permanently by default unless policy explicitly allows it.
- Translation preserves source-language provenance and must not silently rewrite protected slots.

Proof:

- record state transition proof;
- no live chat from record-mode audio;
- transcript artifact proof;
- adapter upload/chunk transport proof;
- artifact ledger handoff proof;
- translation artifact proof;
- speaker-label/deferred-label proof;
- summary/main-points/action-item proof;
- protected send/schedule/onboarding draft blocked without authority/simulation;
- no permanent raw audio retention by default.

Next if passed:

- Stage 28 - Image And Video Generation And Editing.

## Stage 28 - Image And Video Generation And Editing

Status: NEEDS_BUILDING

Build:

- `PH1.IMAGE.GEN`;
- `PH1.VIDEO.GEN`;
- `PH1.VIDEO.EDIT`;
- `PH1.VIDEO.ARTIFACT`;
- `PH1.VIDEO.SAFETY`;
- image edit;
- generated image packet;
- edited image packet;
- generated video packet;
- edited video packet;
- video artifact provenance;
- user-provided input image path;
- user-provided input video path;
- creative image safety;
- creative video safety;
- generated-vs-sourced-image/video separation.

Rules:

- Generated images and videos are creative outputs, not factual evidence.
- Image editing requires user-provided or approved input image.
- Video editing requires user-provided or approved input video.
- Generated images/videos must not be displayed as real source photos/videos.
- Generated images/videos must not become claim proof.

Proof:

- generation route proof;
- edit route proof;
- video generation/edit route proof;
- generated/sourced image/video separation proof;
- safety/policy proof.

Next if passed:

- Stage 29 - Learning, Knowledge, Emotional Guidance, And Adaptation.

## Stage 29 - Learning, Knowledge, Emotional Guidance, And Adaptation

Status: PARTIALLY_BUILT

Build:

- `PH1.FEEDBACK`;
- `PH1.LEARN`;
- `PH1.KNOW`;
- `PH1.KG`;
- `PH1.CACHE`;
- `PH1.PAE`;
- `PH1.PATTERN`;
- `PH1.RLL`;
- `PH1.EMO.CORE`;
- `PH1.EMO.GUIDE`;
- `PH1.HUMAN_EXPERIENCE.LAB`;
- `PH1.EMOTION.SIGNAL`;
- `PH1.EMOTION.CONFIDENCE`;
- `PH1.EMOTION.RESPONSE.MODE`;
- `PH1.EMOTION.BOUNDARY.GATE`;
- `PH1.DISTRESS.SAFE_RESPONSE`;
- `PH1.FRUSTRATION.REPAIR`;
- `PH1.CELEBRATION.RESPONSE`;
- `PH1.PROFESSIONAL_CALM.MODE`;
- `PH1.COMPANION.BOUNDARY`;
- `PH1.EMOTION.BENCH`;
- `PH1.CONVERSATION.CONTINUITY.BENCH`;
- `PH1.COMPANION.BOUNDARY.BENCH`;
- `PH1.SAME_PAGE.BENCH`;
- `PH1.USER_TRUST.BENCH`;
- `PH1.MULTI`;
- `PH1.STUDY.MODE`;
- `PH1.TUTOR.SOCRATIC`;
- `PH1.LEARNING.PATH`;
- `PH1.QUIZ.PRACTICE`;
- feedback capture;
- adaptation artifact packages;
- tenant dictionary/pronunciation packs;
- emotional snapshot/tone guidance;
- emotion signal confidence;
- emotion response mode;
- emotion boundary gate;
- emotional state detection;
- mood/context sensitivity;
- stress/frustration handling;
- celebration/encouragement style;
- professional calm mode;
- grief/distress safe response mode;
- confidence-aware emotional response;
- personal presence adaptation;
- cross-session interaction pattern learning;
- study mode session lifecycle;
- Socratic tutor policy;
- personalized learning paths;
- quiz and practice generation;
- study progress memory handoff;
- live producers for `EmotionalGuidancePacket` and `ProsodyControlPacket` consumed earlier by Write/TTS default-empty packets;
- offline pattern mining;
- offline governed ranking;
- cache refresh validation;
- provider arbitration hints.

Rules:

- Learning/adaptation engines are advisory unless routed through governance.
- Emotional guidance affects tone only, not truth or execution.
- Emotion changes tone, pacing, empathy, and clarification style only.
- Emotional guidance must not manipulate the user, pretend Selene has human feelings, infer protected identity, or override source truth, safety, memory rules, access, authority, simulation, or execution gates.
- Emotion must not grant authority, become identity evidence, change facts, or create fake intimacy.
- Companion behavior means accurate memory, calm response, boundaries, and user agency.
- Grief/distress and high-stress modes must be calm, bounded, and safety-aware.
- Celebration/encouragement style must remain professional and user-controlled.
- Study/tutor mode must adapt to the learner without silently completing restricted assessment work or creating unsupported educational claims.
- Learning paths and quizzes must be scoped, reviewable, and memory-aware without bypassing workspace, consent, or age/education policy where relevant.
- Tenant dictionary hints are advisory and scoped.
- Offline RLL/pattern outputs require governed artifact activation.
- Learning must not silently rewrite facts, authority, access, or protected slots.
- Stage 29 must replace default-empty emotional/prosody packets with live governed producers for Write/TTS consumers.

Proof:

- feedback event proof;
- learning artifact proof;
- dictionary pack proof;
- emotional tone-only proof;
- emotion signal/confidence/boundary proof;
- no manipulation/no fake emotion proof;
- companion boundary and user-trust benchmark proof;
- conversation continuity and same-page benchmark proof;
- stress/frustration/grief safe-response proof;
- celebration/professional-calm mode proof;
- study mode and Socratic tutor proof;
- learning path and quiz practice proof;
- live emotional/prosody producer wiring proof;
- no authority bypass proof;
- governed activation proof.

Next if passed:

- Stage 30 - Builder, Self-Heal, Release, Replay, Codex, And Dev Lane.

## Stage 30 - Builder, Self-Heal, Release, Replay, Codex, And Dev Lane

Status: PARTIALLY_BUILT

Build:

- `PH1.BUILDER`;
- `PH1.SELFHEAL`;
- `PH1.CODEX`;
- `PH1.DEV`;
- `PH1.CUSTOM.ASSISTANT`;
- `PH1.ASSISTANT.BUILDER`;
- `PH1.ASSISTANT.STORE`;
- `PH1.ASSISTANT.SHARING`;
- `PH1.ASSISTANT.ACTIONS`;
- `PH1.ASSISTANT.KNOWLEDGE`;
- `selene_replay`;
- `section07_reopen_detector`;
- `section07_reopen_scan`;
- web-search release/eval binaries: `web_search_turn`, `web_search_enterprise_turn`, `web_search_eval_report`, `web_search_release_evidence`, and `web_search_vision_turn`;
- builder pipeline tables;
- builder approval/release flow;
- model/prompt/provider promotion flow;
- provider championship promotion flow;
- model live-eval report;
- model fallback and rollback report;
- cost-quality score report;
- benchmark leaderboard publication package;
- competitor parity report package;
- custom assistant builder workflow;
- custom assistant store/review workflow;
- custom assistant sharing workflow;
- assistant action manifest review;
- assistant knowledge package review;
- eval-before-promotion gate;
- rollback on quality regression;
- no-silent-provider/model/prompt-drift release proof;
- post-deploy judge;
- self-heal cards;
- release evidence packs;
- replay corpus and regression runner;
- repo analysis;
- code review;
- build instruction generation;
- test routing;
- clean-tree policy;
- worktree policy;
- secret-safety policy;
- CLI/dev tooling.

Rules:

- Dev tools require explicit route.
- No secret leakage.
- No uncontrolled shell or app actions.
- Protected business execution remains separate.
- Builder/self-heal proposals require governance before activation.
- Custom assistants are governed runtime profiles, not separate brains.
- Assistant actions and assistant knowledge must not bypass connector gates, provider gates, access, authority, simulation, law, audit, memory boundaries, or tenant policy.
- Stage 30 owns provider/model/prompt promotion, rollback, release evidence, and regression proof after Stage 3 defines the contracts.
- Stage 30 owns benchmark leaderboards, provider championship release evidence, live-eval review, fallback/rollback evidence, and competitor parity reports.

Proof:

- replay proof;
- release evidence proof;
- model/prompt/provider promotion and rollback proof;
- provider championship promotion/rollback proof;
- benchmark leaderboard and competitor parity report proof;
- cost-quality score regression proof;
- custom assistant builder/store/sharing proof;
- assistant action/knowledge governance proof;
- quality regression rollback proof;
- self-heal proposal proof;
- dev lane route proof;
- no secret/tool bypass proof.

Next if passed:

- Stage 31 - Privacy, Retention, Admin Policy, Health, Export, And Audit.

## Stage 31 - Privacy, Retention, Admin Policy, Health, Export, And Audit

Status: PARTIALLY_BUILT

Build:

- `PH1.PRIVACY.RETENTION`;
- `PH1.DEVICE.TRUST`;
- `PH1.ADMIN.POLICY`;
- `PH1.CONSENT.REGISTRY`;
- full implementation of the Stage 3 privacy/retention/admin policy contract baseline;
- `PH1.HEALTH`;
- `PH1.EXPORT`;
- `device_artifact_sync`;
- audit export;
- compliance export;
- redaction;
- tamper-evident hashes;
- health dashboard/projection;
- tenant isolation;
- device trust;
- app directory/admin policy;
- custom assistant store/admin policy;
- assistant/app audit export;
- shopping/product-link disclosure audit;
- generated video/image retention and export policy;
- wake/Voice ID/record consent and retention controls.

Rules:

- No raw audio retention by default.
- Stage 31 must implement and enforce the Stage 3 privacy/retention/admin policy contract baseline instead of inventing a different policy shape after feature stages already exist.
- Voice profiles are revocable.
- Wake training requires consent.
- Record mode retention requires policy.
- Consent revocation must propagate to wake artifacts, Voice ID profiles, memory records, record artifacts, provider-capable voice processing, and related retention policies.
- No cross-tenant voice/access matching.
- Admins can disable wake, Voice ID, record mode, connectors, retention, or provider lanes.
- Admins can disable app directory lanes, custom assistant publication/sharing, visual agent browser, shopping links, study-mode persistence, and image/video generation where policy requires.
- Health is display/projection, not remediation execution in v1 unless later explicitly authorized.

Proof:

- consent proof;
- Stage 3 privacy/retention/admin policy baseline compatibility proof;
- consent revocation propagation proof;
- retention proof;
- export redaction proof;
- app/custom-assistant admin policy proof;
- assistant/app audit export proof;
- shopping disclosure audit proof;
- generated image/video retention proof;
- health projection proof;
- tenant isolation proof;
- no raw audio default retention proof.

Next if passed:

- Stage 32 - Advanced Language Profiles And Language Certification.

## Stage 32 - Advanced Language Profiles And Language Certification

Status: NEEDS_BUILDING

Build:

- `PH1.LANG.PROFILE.REGISTRY`;
- `PH1.LANG.TRACE.EVAL`;
- language test packs;
- `PH1.LANG.CERT.ENGLISH`;
- `PH1.LANG.CERT.CHINESE`;
- `PH1.LANG.CERT.MIXED_EN_ZH`;
- `PH1.LANG.CERT.DIALECTS`;
- `PH1.LANG.CERT.SLANG`;
- `PH1.LANG.CERT.ACCENTED_STT`;
- `PH1.LANG.CERT.CODE_SWITCH`;
- `PH1.LANG.CERT.TTS_PROSODY`;
- locale/script profiles;
- business glossary;
- protected terms;
- explicit translation mode;
- language switch scope;
- model selection for language tasks: translation, grammar repair, same-language writing, STT, TTS, and mixed-language reasoning;
- TTS/display language proof;
- mixed-language certification.
- per-language, dialect, slang, accented-STT, code-switch, and TTS-prosody certification.

Rules:

- OpenAI may provide language intelligence, but Selene governs safety, routing, trace, and proof.
- Same-language response is preserved unless user requests otherwise.
- Wrong-language protected command fails closed or clarifies.
- Language must not infer nationality, race, ethnicity, citizenship, or protected identity.
- Language model choice must be certified per task profile and must not silently translate, rewrite protected slots, or downgrade non-English tool routing.

Proof:

- English typed/voice proof;
- Chinese typed/voice proof;
- mixed-language proof;
- dialect/slang/accented-STT proof;
- code-switch proof;
- TTS prosody language proof;
- non-English tool routing proof;
- language task model-selection proof;
- protected non-English fail-closed proof;
- display/TTS language hash proof.

Next if passed:

- Stage 33 - Native And Runtime Product Parity Certification.

## Stage 33 - Native And Runtime Product Parity Certification

Status: NEEDS_BUILDING

Build:

- cross-client parity harness;
- Desktop wake/explicit mic/record proof;
- iPhone side-button/record proof;
- Android wake/explicit mic/record proof where Android workspace exists;
- typed runtime proof;
- adapter HTTP/gRPC proof;
- TTS proof;
- source/image/presentation proof;
- Search Operating System parity proof: public web, URL/PDF/page reader, deep research, source chips, image/product cards, cached/offline freshness, connector/API search, and vertical structured packets;
- extended renderer recertification for product blocks introduced after core Desktop/Mobile renderer proof;
- record mode proof;
- protected fail-closed proof;
- provider-off proof;
- human experience parity proof: natural turn-taking, spoken/display split, persona continuity, memory boundaries, prosody, and correction behavior;
- model choice parity proof across typed, voice, Desktop, iPhone, Android where present, and provider-off paths;
- ChatGPT-parity product-surface proof across custom assistants, app cards, visual agent, data charts, canvas share/export, study mode, shopping cards, and image/video artifacts where implemented;
- accessibility and visual proof where required.

Rules:

- Typed, voice, Desktop, iPhone, Android, and Windows paths must agree on authority and safety where surfaces exist.
- Clients may differ in activation mechanics but not in runtime law.
- Voice proof cannot be replaced by typed proof when voice is required.
- Model choice proof cannot be replaced by generic provider availability proof.
- The same user across Desktop, iPhone, Android, and Windows should feel continuous where consent, access, memory, and workspace boundaries allow it.
- Stage 19 and Stage 20 core renderer proof cannot replace Stage 33 extended product-block parity proof.
- Every product-specific presentation block must be recertified after its owning engine stage exists, including data charts, canvas, apps/connectors, visual agent, record mode, image/video artifacts, study/tutor, shopping/product cards, and custom assistants where implemented.

Proof:

- full native smoke matrix;
- client parity report;
- Android parity report where Android workspace exists;
- extended Desktop/Mobile product-block renderer proof;
- xcodebuild proof;
- runtime proof;
- cross-client persona/memory/prosody continuity proof;
- model choice parity proof;
- product-surface parity proof;
- search operating system parity proof;
- no client authority drift.

Next if passed:

- Stage 34 - Full System Certification Harness.

## Stage 34 - Full System Certification Harness

Status: NEEDS_BUILDING

Must certify:

- typed chat;
- Desktop wake;
- Desktop explicit mic;
- iPhone side button;
- Android wake;
- Android explicit mic;
- Voice ID;
- session open/close;
- interruption/barge-in;
- record button;
- meeting summary;
- reminders drafted from recording;
- protected action fail-closed;
- public search;
- web search sublanes;
- Search Operating System end-to-end;
- public web query planning, multi-query, result ranking, fetch/read, source accept/reject, source chips, budget, and provider-off proof;
- direct URL/PDF/page reader with safe citation/table extraction;
- deep research report with source plan, multi-hop, claim ledger, citation graph, contradiction matrix, rejected-source log, and reviewer pass;
- app/connector search with app auth, app permissions, read-only connector search, source chips, and write separation;
- platform API routes with capability registry, auth, rate limits, rights policy, and source map;
- offline/cache search with TTL, cache safety, freshness check, provider-off/degraded behavior, and stale-data disclosure;
- real-time vertical tools for weather, time, finance, news, flights, shopping/products, academic, registries, and filings;
- search-to-Write pipeline from evidence packet to presentation/source chips to clean TTS;
- Search Builds 1-9;
- source chips;
- image cards;
- TTS clean text;
- STT accuracy benchmark;
- STT WER by language;
- Chinese character error rate;
- noisy-room/far-field/overlapping-speaker STT benchmark;
- diarization error rate;
- endpointing latency and confidence calibration;
- grammar benchmark;
- scrambled speech, bad grammar, slang, accent-to-intent, and protected-slot confidence benchmark;
- search accuracy benchmark;
- source quality benchmark;
- citation accuracy and claim-to-source precision;
- official/source-of-record preference score;
- contradiction detection and freshness accuracy;
- hallucinated citation zero-tolerance proof;
- insufficient-evidence behavior;
- Research OS claim ledger, citation graph, contradiction matrix, rejected-source log, and as-of certification;
- math solve/verify/unit-check benchmark;
- science source/calculation verification benchmark;
- history timeline/source-context benchmark;
- memory recall benchmark;
- false memory rate;
- stale memory suppression;
- project-boundary leakage rate;
- forget-command success rate;
- memory correction-learning benchmark;
- TTS naturalness benchmark;
- TTS mean opinion score;
- TTS pronunciation error rate;
- emotional/prosody match score;
- same-language TTS continuity;
- latency benchmark;
- interruption success rate, audio stop latency, and interrupted-answer resume quality;
- topic continuity, same-page, open-loop, and recap-on-return benchmark;
- emotional appropriateness, distress/frustration repair, companion boundary, and user-trust benchmark;
- model/provider championship router certification;
- STT/TTS multi-provider router certification;
- provider cost-quality/privacy/latency/offline mode certification;
- multilingual per-language/dialect/slang/accent/code-switch/TTS-prosody certification;
- Desktop/iPhone/Android/Windows UX benchmark where surfaces exist;
- protected-action fail-closed benchmark;
- ChatGPT comparison journeys;
- custom assistant builder/store/sharing;
- assistant actions and assistant knowledge governance;
- app directory, app SDK, MCP/custom app lane;
- interactive app cards;
- visual agent browser and watch mode;
- visible data analysis sandbox;
- interactive tables/charts and chart export;
- canvas share/export/version restore;
- study mode, Socratic tutoring, learning path, and quiz practice;
- shopping/product search, product cards, price comparison, review summary, and merchant-link safety;
- video generation and editing;
- model choice certification by task profile;
- natural spoken conversation;
- interruption feels graceful;
- correction feels understood;
- user frustration handled calmly;
- long project memory recall works;
- wrong memory is corrected;
- pronunciation improves;
- same user across devices feels continuous where permitted;
- spoken answer is shorter than display answer where appropriate;
- warm tone without fake claims;
- no emotional manipulation;
- no creepy memory behavior;
- Desktop renderer;
- iPhone renderer;
- file/doc/data/vision;
- canvas/artifacts;
- connectors;
- tasks/schedules;
- broadcast/delivery/reminders;
- onboarding/link/position/CAPREQ;
- access control;
- provider-off zero calls;
- KMS/secret no-leak;
- no silent provider/model/prompt drift;
- no wrong model profile used for certified task lanes;
- no unmeasured benchmark lane marked complete;
- no provider championship promotion without eval and rollback evidence;
- no background/non-user speech command execution;
- no low-confidence protected audio guess;
- no protected-slot guessing from scrambled speech repair;
- no false memory or suppressed memory recall beyond benchmark threshold;
- no emotional signal used as identity, authority, or fact;
- no unsafe custom assistant, app, MCP, or visual-agent action bypass;
- no restricted-platform scraping;
- no connector write hidden inside search;
- no rejected source chip;
- no source chip without accepted source;
- no stale cached/offline search presented as current;
- no hidden data sandbox execution;
- no unsafe shopping merchant routing or undisclosed product-link behavior;
- no generated video/image treated as sourced evidence;
- privacy/retention;
- audit/export;
- replay/release/self-heal;
- tenant isolation;
- final clean-tree proof.

Final pass means:

- no second brain;
- no provider leakage;
- no protected bypass;
- no benchmark lane without a numeric threshold and replayable result;
- no provider/model champion drift without release evidence;
- no fake images;
- no source dumps;
- no rejected source chips;
- no source chip without accepted source;
- no raw provider metadata in search display or TTS;
- no fabricated research certainty;
- no missed official/source-of-record preference where required;
- no hallucinated citation;
- no unhandled contradiction when evidence conflicts;
- no TTS metadata;
- no prosody drift or TTS rewrite drift;
- no emotional manipulation or fake feeling claims;
- no suppressed/forbidden memory recall;
- no background speech treated as a user command;
- no protected slot guessed from unclear audio or scrambled language;
- no native-client authority drift;
- no raw record audio retention by default;
- no cross-tenant access/voice leak;
- clean repo docs and ledger.

Next if passed:

- Mark canonical roadmap implementation complete, then continue only with explicitly approved product expansion or bugfix builds.

## Correct Build Order Summary

```text
1. Canonical inventory and wiring map
2. Runtime kernel, storage, proof ledger, and law foundation
3. Provider, secret, KMS, cost, quota, vault, and early consent baseline
4. Activation, session, turn, and packet foundation
5. Session open/resume/close and runtime turn spine
6. Master access, tenant, policy, and per-user authority context
7. Wake, side button, and activation stack
8. Voice I/O, listen state, transcript gate, and turn boundary
9. Voice ID
10. Universal understanding and perception assist
11. Reasoning orchestrator, capability registry, and tool route
12. Risk, authority, simulation, execution gate, and protected-action closure
13. Search, source, image evidence, and public tool quality
14. Web search enterprise sublanes and release proof
15. Write, response, and TTS-safe text engine
16. Presentation contracts
17. Speech/TTS output and playback control
18. Adapter, protocol, and rich transport
19. Desktop native runtime and renderer
20. Mobile native runtime and renderer
21. Project, memory, persona, workspace, and context
22. File, document, data, vision, OCR, and media understanding
23. Canvas, artifacts, and artifact governance
24. Agent, apps, connectors, tasks, and scheduling
25. Broadcast, delivery, reminders, and message lifecycle
26. Business process actions, link, onboarding, position, and capability requests
27. Record mode and meeting recording product
28. Image and video generation and editing
29. Learning, knowledge, emotional guidance, and adaptation
30. Builder, self-heal, release, replay, Codex, and dev lane
31. Privacy, retention, admin policy, health, export, and audit
32. Advanced language profiles and language certification
33. Native and runtime product parity certification
34. Full system certification harness
```

## Exact Repo Surface Audit Anchors

Stage 1 must check these exact repo-surface names so the canonical inventory does not accidentally hide real modules behind broad architecture categories.

Exact binary, native, and grouped-storage anchors:

- `selene.rs`
- `SeleneMacDesktopApp`
- `PH1_LEARN_FEEDBACK_KNOW`

Exact storage and migration-name anchors:

- `ph1f`
- `work_orders`
- `ph1l`
- `ph1vid`
- `access_instance`
- `ph1k`
- `ph1link`
- `access_master`
- `access_ap`
- `self_heal`
- `ph1m`
- `wake_artifact`
- `wake_learn_signal`

Exact `web_search_plan` leaf-module anchors:

- `admission`
- `apply`
- `asof`
- `audit_fields`
- `boundary_guard`
- `brave_news`
- `budget_control`
- `cache_key`
- `cache_safety`
- `calibration`
- `change_classify`
- `charset`
- `chunker`
- `citation_renderer`
- `citation_validator`
- `claim_confidence`
- `claim_extractor`
- `collision`
- `compliance_confidence`
- `conflict_handler`
- `corpus_packs`
- `currency_normalize`
- `cycle_detect`
- `dead_link_handler`
- `debug_packet`
- `decimal`
- `decompress`
- `diversity`
- `domain_rules`
- `enterprise_pipeline`
- `enterprise_request`
- `entity_normalize`
- `error_taxonomy`
- `evidence_pack`
- `factors`
- `failure_signature`
- `fallback_policy`
- `feature_normalize`
- `filters`
- `formatter`
- `freshness_watchdog`
- `generic_http_json`
- `guardrails`
- `hasher`
- `health_state`
- `hop_audit`
- `hop_budget`
- `hop_plan`
- `hop_runner`
- `injection_defense`
- `insufficiency_gate`
- `internal_context`
- `join`
- `limiter`
- `localization`
- `merge_order`
- `merge_packet`
- `mime`
- `mode_router`
- `multi_query`
- `official_detector`
- `open_selector`
- `output_packet`
- `packet_builder`
- `pdf_fetch`
- `pdf_tables`
- `pdf_text`
- `presentation_modes`
- `pricing_normalize`
- `promotion_gate`
- `proposal_artifact`
- `provider_merge`
- `proxy_config`
- `proxy_redaction`
- `proxy_retry`
- `proxy_self_check`
- `quality_gate`
- `recency`
- `redirect`
- `reformulation`
- `regressions`
- `reranker`
- `risk_packet`
- `schema_map`
- `session_adaptation`
- `snippet_fallback`
- `spam_signals`
- `state_trace`
- `style_guard`
- `table_render`
- `template`
- `temporal_packet`
- `tie_break`
- `trust_score`
- `trust_tier`
- `ttl_policy`
- `unit_normalize`
- `unknown_first`
- `voice_renderer`

## Current Next Build Instruction Seed

The next Codex build should be:

````text
TASK: CANONICAL_ENGINE_INVENTORY_AND_WIRING_MAP_REPAIR

Build Class:
DOCS_ARCHITECTURE_RECONCILIATION_ONLY

Repo Root:
`/Users/selene/Documents/Selene-OS`

Stage:
Stage 1 - Canonical Inventory And Wiring Map

STARTUP PROOF

Run:

cd /Users/selene/Documents/Selene-OS
pwd
git status --short
git status -sb
git rev-parse --abbrev-ref HEAD
git rev-parse HEAD
git rev-parse origin/main

If `git status --short` is not clean, STOP and report dirty files.

CANONICAL PLAN VERSION CHECK

Before continuing, verify `docs/SELENE_CANONICAL_MASTER_BUILD_PLAN.md` contains:

- `Stage 34 - Full System Certification Harness`
- `Dependency DAG And Build Slice Execution Rule`
- `Benchmark Target Status Legend`
- `Search Operating System And ChatGPT-Level Search Layer`
- `GLOBAL NUMBER-ONE DRAFT BENCHMARK TARGETS`

If any are missing, STOP and report `PLAN_VERSION_MISMATCH`.
Do not continue from an older 27-stage plan.

Goal:
Create the repo-truth inventory, dependency DAG, build-family/slice map, and wiring map that reconciles existing PH1 modules, docs, adapter routes, native client renderers, wake/side-button/record activation, Voice ID, access, provider/KMS/cost controls, early privacy/retention/admin policy contracts, default-empty human-experience packet contracts, Search Operating System lanes, minimal reader extraction foundation, connector/API route contracts, search sublanes, protected execution, runtime law, replay, storage, global number-one benchmark/lab lanes, ChatGPT-parity product lanes, core-vs-extended native renderer proof, and the current integrated master plan.

This build must prove what already exists, what is partial, what is missing, what is duplicated, what is stale, what is unwired, and what must block, narrow, or merge Stage 2.

Required first-read files:

- `AGENTS.md`
- `docs/SELENE_CANONICAL_MASTER_BUILD_PLAN.md`
- `docs/CORE_ARCHITECTURE.md`
- `docs/SELENE_BUILD_EXECUTION_ORDER.md`
- `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`
- `docs/COVERAGE_MATRIX.md`
- `docs/MASTER_BUILD_COMPLETION_PLAN.md`
- `docs/MASTER_BUILD_COMPLETION_LEDGER.md`

CHANGED-FILE WHITELIST

Codex may only create/update these files:

- `docs/SELENE_CANONICAL_STAGE1_INVENTORY_AND_WIRING_MAP.md`
- `docs/SELENE_CANONICAL_DEPENDENCY_DAG.md`
- `docs/SELENE_CANONICAL_GOLDEN_JOURNEY_MATRIX.md`
- `docs/SELENE_CANONICAL_STAGE_BUILD_SLICE_MAP.md`
- `docs/SELENE_CANONICAL_BENCHMARK_TARGET_STATUS_MATRIX.md`
- `docs/SELENE_CANONICAL_MASTER_BUILD_PLAN.md`
- `docs/00_INDEX.md`
- `docs/MASTER_BUILD_COMPLETION_PLAN.md`
- `docs/MASTER_BUILD_COMPLETION_LEDGER.md`

If any non-whitelisted file changes, STOP and revert only the non-whitelisted changes made by this build.
If any production code changes, STOP and revert only the production code changes made by this build.

Primary outputs to create or update:

1. Create `docs/SELENE_CANONICAL_STAGE1_INVENTORY_AND_WIRING_MAP.md`.

   Must include:

   - repo-truth inventory of all contracts, engines, OS modules, adapter routes, native clients, tools, migrations, docs, tests, and proof artifacts;
   - status table for every major module/engine: `complete`, `partial`, `standalone`, `duplicated`, `unwired`, `missing`, `legacy`, or `deprecated`;
   - evidence file paths for each finding;
   - packet/handoff mapping from activation to audit/replay;
   - reconciliation against older 86-engine references;
   - reconciliation against `SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`;
   - reconciliation against `SELENE_BUILD_EXECUTION_ORDER.md`;
   - Stage 2 readiness verdict.

2. Create `docs/SELENE_CANONICAL_DEPENDENCY_DAG.md`.

   Must show:

   - every numbered stage;
   - every required input packet;
   - every emitted output packet;
   - upstream dependencies;
   - downstream consumers;
   - blocked stages;
   - stages or slices that may run in parallel;
   - stages or slices that must not start early;
   - benchmark target families owned by each stage;
   - build-family slices for broad stages.

3. Create `docs/SELENE_CANONICAL_GOLDEN_JOURNEY_MATRIX.md`.

   Must include 15-20 end-to-end journeys, including:

   - wake Selene, ask a question, get spoken answer;
   - interrupt Selene mid-answer;
   - ask search question with source chips;
   - ask weather/time in another language;
   - record meeting, summarize it, draft reminders;
   - send onboarding link as protected draft;
   - correct Selene memory;
   - switch from Desktop to iPhone;
   - ask sensitive action and fail closed;
   - ask with frustration and get calm response;
   - custom assistant governed knowledge/action journey;
   - read-only app card journey;
   - visible data analysis/chart journey;
   - canvas share/export/version journey;
   - study/tutor journey;
   - shopping/product comparison journey;
   - generated image/video artifact journey without treating it as evidence.

4. Create `docs/SELENE_CANONICAL_STAGE_BUILD_SLICE_MAP.md`.

   Must split broad stages into future Codex build slices.

   At minimum, split:

   - Stage 8
   - Stage 10
   - Stage 13
   - Stage 15
   - Stage 21
   - Stage 24
   - Stage 29
   - Stage 30

   Stage 13 must include at minimum:

   ```text
   Stage 13A - Public web baseline and provider-off proof
   Stage 13B - URL/PDF/page/table reader foundation
   Stage 13C - Source verification and claim-to-source mapping
   Stage 13D - Cache/offline/freshness packets
   Stage 13E - Real-time vertical structured packets
   Stage 13F - Image/product evidence packets
   Stage 13G - Connector/API route contracts with mock proof only
   ```

5. Create `docs/SELENE_CANONICAL_BENCHMARK_TARGET_STATUS_MATRIX.md`.

   For every stage and relevant benchmark family, assign one status:

   ```text
   NOT_APPLICABLE_WITH_REASON
   BASELINE_MEASURED
   CERTIFICATION_TARGET_PASSED
   BLOCKED_WITH_OWNER_AND_NEXT_ACTION
   ```

6. Update `docs/SELENE_CANONICAL_MASTER_BUILD_PLAN.md`.

   Must update:

   - Stage 1 status;
   - Build Tracking Rule;
   - links to the Stage 1 inventory, dependency DAG, golden journey matrix, stage build slice map, and benchmark target status matrix;
   - Plan Reconciliation Notes if repo evidence contradicts the plan;
   - next exact stage/build name.

7. Update only if required by this build's evidence:

   - `docs/00_INDEX.md`
   - `docs/MASTER_BUILD_COMPLETION_PLAN.md`
   - `docs/MASTER_BUILD_COMPLETION_LEDGER.md`

Do not implement feature behavior yet.
Do not rebuild search.
Do not redesign Desktop or iPhone.
Do not run live providers.
Do not use Python.
Do not create a second brain.
Do not weaken provider gates, budget counters, startup-probe blocks, KMS/secret boundaries, protected execution gates, or audit law.
Use repo truth only.
Use `rg` / `sed` / file reads for discovery.
Do not hardcode real searched names in code, tests, fixtures, mocks, corpora, sample data, or proof hooks.
If repo evidence contradicts the master plan, add a Plan Reconciliation Note with file/line evidence and state that JD approval is required before changing the roadmap.

Required inventory coverage:

- runtime/storage/proof-ledger systems;
- activation, wake, side button, explicit mic, typed input, record button;
- session, turn, listen, interruption, TTS output control;
- Voice ID, access, policy, tenant, KMS, provider, cost, quota;
- provider/model/prompt governance;
- privacy/retention/admin policy contracts;
- public tools and protected workflow engines;
- Search Operating System lanes;
- minimal reader extraction foundation;
- connector/API route contracts;
- web search sublanes and proof binaries;
- Write, presentation, adapter, Desktop, iPhone, Android, Windows surfaces;
- record mode;
- memory, persona, learning, knowledge, emotional guidance;
- default-empty human-experience packet contracts before live Stage 21/29 producers;
- benchmark target status ownership;
- ChatGPT-parity product lanes;
- replay, self-heal, builder, release tooling;
- exact repo surface audit anchors listed in the master plan.

Proof required:

- clean tree start proof;
- canonical 34-stage plan version proof;
- exact file paths and repo evidence;
- no code behavior change;
- no provider calls;
- no Python;
- dependency DAG exists;
- Golden Journey Matrix exists;
- Stage 1 inventory exists;
- build-family/slice map exists;
- benchmark ownership/status matrix exists;
- packet/handoff matrix mapped to repo-truth names where possible;
- changed-file whitelist proof;
- `git diff --check`;
- final report states whether Stage 2 is ready, narrowed, merged, or blocked.

FINAL CHECKS

Run:

```text
cargo check -p selene_os -p selene_adapter -p selene_engines
git diff --check
git diff --name-only
rg -n "SELENE_CANONICAL_STAGE1_INVENTORY_AND_WIRING_MAP|SELENE_CANONICAL_DEPENDENCY_DAG|SELENE_CANONICAL_GOLDEN_JOURNEY_MATRIX|SELENE_CANONICAL_STAGE_BUILD_SLICE_MAP|SELENE_CANONICAL_BENCHMARK_TARGET_STATUS_MATRIX" docs
git status --short
```

COMMIT / PUSH

Commit only if:

- tree was clean at start;
- canonical 34-stage plan version check passed;
- only whitelisted docs changed;
- no production code changed;
- no provider calls occurred;
- no Python was used;
- final checks passed;
- Stage 2 verdict is clearly stated.

Commit message:

```text
Stage 1: add canonical inventory and wiring map
```

Push after successful commit.

FINAL REPORT FORMAT

A) Clean Tree Start Proof
B) Canonical Plan Version Proof
C) Scope Confirmation
D) Files Read
E) Files Created / Updated
F) Inventory Summary
G) Dependency DAG Summary
H) Build Slice Map Summary
I) Golden Journey Matrix Summary
J) Benchmark Target Status Matrix Summary
K) Packet / Handoff Mapping Summary
L) Drift / Duplicate / Legacy Findings
M) Stage 2 Ready / Narrow / Merge / Blocked Verdict
N) Checks Passed
O) Changed-File Whitelist Proof
P) Final Clean Tree Proof
Q) Commit Hash And Push Proof

Final report must answer:

1. Is Stage 2 ready to start?
2. If not, what exact Stage 2 narrowing, merge, or blocker is required?
3. Which canonical packets already exist?
4. Which Search Operating System lanes exist, are partial, or are missing?
5. Which benchmark/lab lanes exist, are partial, or are missing?
6. Which ChatGPT-parity product lanes exist, are partial, or are missing?
7. Which learning/memory/persona/emotion modules exist and where are they wired?
8. Which dependency DAG stages/slices are blocked or must not start early?
9. Which build-family slices are next after Stage 1?
10. What is the next exact build name?

Expected next build if Stage 1 passes:

Stage 2 - Runtime Kernel, Storage, Proof Ledger, And Law Foundation
````

## Build Completion Update Rule

At the end of every future build:

1. Update the stage status in this document.
2. Update the `Build Tracking Rule` table.
3. State the next exact stage and build name.
4. If the build discovers this plan is wrong, do not silently drift. Add a `Plan Reconciliation Note` with file/line evidence and JD approval requirement.
5. Update `MASTER_BUILD_COMPLETION_PLAN.md` and `MASTER_BUILD_COMPLETION_LEDGER.md` when the build instruction requires it.
6. Keep final reports aligned to this plan.

## Plan Reconciliation Notes

- Stage 1 canonical inventory is complete in `docs/SELENE_CANONICAL_STAGE1_INVENTORY_AND_WIRING_MAP.md`. Stage 2 should start as the narrowed reconciliation slice `Stage 2A - Runtime Kernel, Storage, Proof Ledger, Law Foundation, And Minimal Benchmark Envelope Inventory Reconciliation`; it must reuse current runtime/storage/law evidence instead of rebuilding foundations from zero.
- Older 86-engine references are not deleted by this plan. They must be reconciled in Stage 1.
- This plan is intentionally wider than the earlier four-stack plan because the repo contains storage, governance, access, delivery, learning, replay, builder, and native-client systems that must be first-class.
- Search work is preserved and finished in Stages 13-14. It is not rebuilt from zero.
- Final ChatGPT-style visual quality is completed through Write, presentation contracts, adapter transport, Desktop, iPhone, Android, and Windows where surfaces exist.
- The 34-stage order is canonical, but Stage 1 must create `docs/SELENE_CANONICAL_DEPENDENCY_DAG.md` so every stage, packet, dependency, downstream consumer, parallelizable slice, blocked slice, and must-not-start-early slice is explicit before implementation continues.
- Large stages are build families, not one Codex build. Stage 8, Stage 10, Stage 13, Stage 15, Stage 21, Stage 24, Stage 29, and Stage 30 must be split into narrow build slices, with Stage 13 split at minimum into Stage 13A through Stage 13G.
- Core language governance is intentionally split: early language safety lives in understanding/routing/write/TTS stages; advanced language profile certification lives in Stage 32.
- Record mode is split deliberately: Stage 4 carries boundary fields so record audio cannot be mistaken for a live turn; Stage 27 builds the full meeting/voice-note product workflow.
- Native clients are intentionally after presentation contracts and adapter transport, except for early packet/client-boundary inventory and proof.
- Provider secrets, KMS, quota, cost, provider-control, and early consent/device-trust baselines are first-class in Stage 3 because provider safety, privacy, and billing-grade counters are repository law.
- Privacy, retention, and admin policy contracts start in Stage 3 so memory, record mode, files/data, connectors/apps, generated media, learning, and provider-capable voice paths cannot be built before their policy shape exists. Stage 31 remains the full privacy/retention/admin/export/audit implementation stage.
- Human experience is not a separate brain. It is a governed layer spanning conversation control, voice I/O, understanding, Write, TTS, memory/persona, emotion/adaptation, clients, and final certification.
- Write/TTS can consume default-empty human-experience packets before live producers exist, but those default-empty packet contracts must compile, serialize, trace, and safe-fallback before Stage 15/17 can be marked complete. Stage 21 and Stage 29 must later wire the live governed producers into those consumers.
- Stage 2 builds the minimal proof/replay envelope required for every stage; Stage 30 builds the full replay, release evidence, self-heal, and dev lane.
- Stage 14 search release proof uses the Stage 2 minimal proof/replay envelope and does not replace Stage 30 full release, promotion, rollback, regression, and competitor-parity ownership.
- Stage 3 owns model/prompt/provider governance contracts; Stage 30 owns promotion/rollback/release proof; Stage 34 certifies no silent provider/model/prompt drift.
- Stage 20 is mobile-native, covering iPhone now and Android as a first-class target when Android repo surface exists.
- Accuracy, research, model intelligence, and listening excellence are global market-leader gates. They wire into Stage 3 model profiles, Stage 8 elite STT, Stage 10 language repair, Stage 11 task-to-model routing, Stage 13/14 full web research, Stage 15/16 research presentation, Stage 17 voice model policy, Stage 21 memory accuracy, Stage 32 language certification, and Stage 34 benchmarks.
- Global number-one quality is a benchmark-driven system, not a slogan. It wires into Stage 1 inventory, Stage 2 benchmark envelopes, Stage 3 provider championship routing, Stage 5 same-page conversation, Stage 8 listening lab, Stage 10 meaning reconstruction, Stage 11 domain routing, Stage 13/14 Research OS and domain verification, Stage 15/17 display-speech optimization, Stage 21 memory trust, Stage 29 human-experience lab, Stage 30 leaderboards/release evidence, Stage 32 language certification, and Stage 34 numeric certification.
- `PROVEN_COMPLETE` requires benchmark target status, not just tests. Every relevant benchmark must be marked `NOT_APPLICABLE_WITH_REASON`, `BASELINE_MEASURED`, `CERTIFICATION_TARGET_PASSED`, or `BLOCKED_WITH_OWNER_AND_NEXT_ACTION`.
- ChatGPT-parity product lanes are global market-surface gates, not extra numbered stages. They wire into Stage 13 shopping/product research, Stage 16 presentation contracts, Stage 22 data sandbox/charts, Stage 23 canvas share/export/version restore, Stage 24 app directory/MCP and visual agent, Stage 28 image/video generation, Stage 29 study/tutor, Stage 30 custom assistant builder/store, Stage 31 admin/audit/retention policy, and Stage 34 certification.
- Search Operating System is the required path to ChatGPT-level or better search. It wires into Stage 1 inventory, Stage 13 public web/minimal URL/PDF/page/table reader extraction foundation/source verification/cache/verticals/connector/API route contracts, Stage 14 deep research and enterprise sublanes, Stage 15 search-to-Write, Stage 16 source/research/product presentation blocks, Stage 18 packet transport, Stage 22 full reader/file/data implementation where contracts overlap, Stage 24 live connector/API search, Stage 33 native/runtime search parity, and Stage 34 search certification.
- Desktop and mobile native stages certify core runtime/rendering first. Product-specific blocks introduced after Stage 19/20 must be recertified in Stage 33 and Stage 34 rather than being treated as fully certified before their owning engines exist.
