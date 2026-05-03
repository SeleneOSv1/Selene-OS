# Selene Canonical Stage Build Slice Map

Status: STAGE1_BUILD_FAMILY_CONTROL_ARTIFACT
Date: 2026-05-03

Large stages are build families. A future Codex build must select one exact slice or one tightly coupled slice cluster.

## Universal Slice Rules

- State exact input and output packets.
- State exact docs/files expected to change where known.
- Keep provider-off proof unless live provider work is explicitly allowed.
- Do not combine unrelated runtime, UI, provider, and benchmark work in one slice.
- Update benchmark target status before marking the slice complete.

## Stage 3 - Provider, Secret, KMS, Cost, Quota, Vault, Consent, And Provider Routing

| Slice | Focus | Input | Output | Proof |
|---|---|---|---|---|
| 3A | Provider safety, KMS/secret, cost/quota, provider-off, startup no-probe, early consent | Stage2 proof/benchmark envelope | ConsentStatePacket, provider-off proof, KMS/cost/quota crosswalk | PROVEN_COMPLETE: disabled providers produce zero attempts/dispatches; startup/health make no provider probes; secrets stay behind KMS/vault; early consent is revocation-aware. |
| 3B | STT/TTS provider router contracts, Apple/OpenAI/local profiles, fallback reasons | Stage3A provider safety plus Stage8A transcript boundary when present | SttProviderProfilePacket, TtsProviderProfilePacket, VoiceProviderRouteDecisionPacket, VoiceProviderFallbackReason | PROVEN_COMPLETE: route decisions are inert, provider-off zero-attempt/zero-dispatch, Apple/OpenAI/local profiles are contracts only, and no live STT/TTS/mic/native/provider behavior is added. |

## Stage 5 - Session Open, Resume, Close, Runtime Turn Spine

| Slice | Focus | Input | Output | Proof |
|---|---|---|---|---|
| 5A | Session lifecycle/current-turn authority/stale-turn quarantine | Stage4TurnBoundaryPacket, SessionPacket | Stage5TurnAuthorityPacket | PROVEN_COMPLETE: only current committed turns enter understanding/render as current; stale/superseded/cancelled/abandoned/record/closed dispositions quarantine without route authority. |
| 5B | Conversation control, clarification, correction, recovery, same-page state | Stage5TurnAuthorityPacket | ConversationGoalStatePacket, OpenLoopsPacket, SamePageCheckPacket | PROVEN_COMPLETE: only current Stage 5A authority can update advisory conversation state; stale/record/closed turns are blocked; one-question clarification, session-scoped correction, safe backchannel, and no provider/tool/protected execution proof. |

## Stage 8 - Voice I/O, Listen State, Transcript Gate

| Slice | Focus | Input | Output | Proof |
|---|---|---|---|---|
| 8A | Audio substrate and listen-state reconciliation | Stage7ActivationContextPacket, Stage5TurnAuthorityPacket where final commit is requested | Stage8TranscriptGatePacket, partial transcript preview, final transcript commit boundary | PROVEN_COMPLETE: audio/listen/partial transcript state cannot execute/search/speak/call providers/identify/authorize/route tools/connector-write/protected-mutate; final transcript commit requires Stage 5 current-turn authority; background/self-echo/non-user audio is blocked; record audio remains artifact-only. |
| 8B | VAD/endpointing/partial-vs-final transcript and confidence gate | Stage8TranscriptGatePacket, Stage7ActivationContextPacket, Stage5TurnAuthorityPacket | Stage8EndpointState, Stage8ConfidenceGateDisposition, Stage8ProtectedSlotDisposition | PROVEN_COMPLETE: VAD/endpoint signals are boundary-only; final transcript commit requires endpoint-final plus confidence/coverage pass; low-confidence protected slots clarify or fail closed; record audio remains artifact-only; no live mic/STT/TTS/provider/search/protected execution behavior. |
| 8C | Listening lab scene/noise/echo/diarization | Stage8TranscriptGatePacket, Stage8EndpointState, Stage8ConfidenceGateDisposition | Stage8AudioScenePacket, Stage8ForegroundSpeakerPacket, Stage8AddressedToSelenePacket | PROVEN_COMPLETE: scene, foreground, addressed, echo, noise, overlap, and barge-in signals are advisory or blocking-only; they cannot identify, authorize, route, execute, call providers, speak TTS, or commit turns by themselves; record audio remains artifact-only. |
| 8D | Listening lab numeric benchmarks, STT WER, noise, diarization, endpoint latency, and calibration | BenchmarkTargetPacket, Stage8AudioScenePacket | BenchmarkResultPacket, listening-lab target status | WER/CER/noise/diarization/latency status assigned |
| 8E | Barge-in, interruption, cancel/resume | TtsPacket, ResponsivenessStatePacket | cancellation and resume state | stale output blocked |
| 8F | STT benchmark corpus and target status | BenchmarkTargetPacket | BenchmarkResultPacket | WER/CER/latency status assigned |

## Stage 10 - Universal Understanding And Perception Assist

| Slice | Focus | Input | Output | Proof |
|---|---|---|---|---|
| 10A | Spelling, grammar, punctuation repair | CommittedTurnPacket | UnderstandingPacket | no protected slot rewrite |
| 10B | Meaning hypothesis lattice and slang/scrambled repair | CommittedTurnPacket | MeaningHypothesisPacket | protected slot confidence gate |
| 10C | Multilingual/script/code-switch understanding | CommittedTurnPacket | UnderstandingPacket | same-language intent proof |
| 10D | Emotion/context signal extraction | UnderstandingPacket | EmotionSignalPacket | signal is advisory only |
| 10E | One-question clarification and correction learning | ProtectedSlotConfidencePacket | ClarificationQuestionPacket | one best question proof |

## Stage 13 - Search, Source, Image Evidence, Public Tool Quality

| Slice | Focus | Input | Output | Proof |
|---|---|---|---|---|
| 13A | Public web baseline and provider-off proof | RouteCandidatePacket | SearchQueryPlanPacket, SearchResultPacket | provider-off zero attempts |
| 13B | URL/PDF/page/table reader foundation | UrlFetchPacket | PageReadPacket, PdfReadPacket, TableExtractPacket | safe extraction and citation extraction |
| 13C | Source verification and claim mapping | SearchResultPacket, PageReadPacket | SourceVerificationPacket, CitationPacket | wrong-source rejection |
| 13D | Cache/offline/freshness packets | SearchResultPacket | SearchCachePacket, OfflineIndexPacket, FreshnessCheckPacket | stale data disclosure |
| 13E | Real-time vertical structured packets | RouteCandidatePacket | RealtimeVerticalPacket, StructuredToolEvidencePacket | structured packet, not raw link |
| 13F | Image/product evidence packets | RouteCandidatePacket | ImageEvidencePacket, ProductEvidencePacket, ProductCardPacket | generated-vs-sourced separation |
| 13G | Connector/API route contracts with mock proof only | RouteCandidatePacket | ApiRoutePacket, ConnectorSearchPacket | no live connector/API mutation |

## Stage 15 - Write, Response, And TTS-Safe Text

| Slice | Focus | Input | Output | Proof |
|---|---|---|---|---|
| 15A | Evidence-to-answer core | EvidencePacket | WriteResponsePacket | no fact invention |
| 15B | Display vs spoken split | WriteResponsePacket | display text and `tts_text` | TTS-safe text proof |
| 15C | Research report modes | ResearchReportPacket | WriteResponsePacket | evidence/rejected-source preservation |
| 15D | Product/study modes | ProductEvidencePacket, StudySessionPacket | WriteResponsePacket | merchant/study boundaries |
| 15E | Persona/emotion default-empty consumers | empty Memory/Persona/Emotion packets | WriteResponsePacket | style-only proof |

## Stage 21 - Project, Memory, Persona, Workspace, Context

| Slice | Focus | Input | Output | Proof |
|---|---|---|---|---|
| 21A | Memory timeline/provenance/confidence | SessionPacket | MemoryTimelinePacket, MemoryTrustPacket | provenance visibility |
| 21B | Remember/forget/suppression | MemoryTrustPacket | MemoryForgetProofPacket | forget success proof |
| 21C | Project/workspace/tenant boundary | AccessContextPacket | MemoryContextPacket | no project/tenant leakage |
| 21D | Persona continuity live producer | PersonaContinuityPacket | Write/TTS consumer input | style-only proof |
| 21E | Memory benchmark target status | BenchmarkTargetPacket | BenchmarkResultPacket | false-memory/stale/leak status |

## Stage 24 - Agent, Apps, Connectors, Tasks, Scheduling

| Slice | Focus | Input | Output | Proof |
|---|---|---|---|---|
| 24A | App directory/SDK/MCP contracts | AppCapabilityPacket | AppInvocationPacket | endpoint not brain |
| 24B | Read connector search | ConnectorSearchPacket | AppSourceChipPacket | app auth/permission proof |
| 24C | Platform API route | ApiRoutePacket | StructuredToolEvidencePacket | rights/rate-limit/source-map proof |
| 24D | Connector write separation | RouteCandidatePacket | FailClosedResponsePacket or ExecutionApprovalPacket | protected gate proof |
| 24E | Visual browser/watch mode | VisualAgentSessionPacket | AgentWatchStatePacket | action supervision proof |
| 24F | Scheduling | Task/Schedule route | scheduled work packet | re-enters ingress |

## Stage 29 - Learning, Knowledge, Emotional Guidance, Adaptation

| Slice | Focus | Input | Output | Proof |
|---|---|---|---|---|
| 29A | Feedback and learning artifacts | feedback event | learning artifact | governed activation proof |
| 29B | Emotional guidance live producer | EmotionSignalPacket | EmotionalGuidancePacket | tone-only proof |
| 29C | Prosody live producer | EmotionalGuidancePacket | ProsodyControlPacket | no TTS rewrite |
| 29D | Human experience lab benchmarks | BenchmarkTargetPacket | BenchmarkResultPacket | trust/boundary target status |
| 29E | Study/tutor mode | StudySessionPacket | TutorStepPacket, QuizPracticePacket | Socratic policy proof |

## Stage 30 - Builder, Self-Heal, Release, Replay, Codex, Dev Lane

| Slice | Focus | Input | Output | Proof |
|---|---|---|---|---|
| 30A | Replay corpus/regression runner | AuditTracePacket | replay result | deterministic replay proof |
| 30B | Model/prompt/provider promotion | BenchmarkResultPacket | promotion/rollback report | no silent drift |
| 30C | Provider championship release evidence | ModelQualityCostPacket | release evidence pack | eval and rollback proof |
| 30D | Benchmark leaderboard package | BenchmarkResultPacket | leaderboard report | prior-release comparison |
| 30E | Custom assistant builder/store | AssistantDefinitionPacket | assistant release artifact | governance proof |
| 30F | Self-heal/dev lane | dev route | proposal artifact | no uncontrolled shell/tool bypass |

## Next Slice After Stage 8C

```text
Stage 8D - Listening Lab Numeric Benchmarks, STT WER, Noise, Diarization, Endpoint Latency, And Calibration Reconciliation
```

Stage 8A is PROVEN_COMPLETE. It added the minimal runtime-owned `Stage8TranscriptGatePacket` carrier, preserved PH1.K, PH1.C, PH1.LISTEN, Stage 7 activation, Stage 5 current-turn authority, and adapter voice surfaces as repo truth, and did not build live mic capture, live STT/TTS, Voice ID matching, understanding, routing, search, native UI redesign, protected execution, or provider/model routing.

Stage 3B is also PROVEN_COMPLETE as a Stage 3 contract slice after Stage 8A. It added inert STT/TTS provider profile and route-decision contracts for Apple/OpenAI/local fallback policy without changing the Stage 8 next-build order.

Stage 8B is PROVEN_COMPLETE. It extends the existing Stage 8A transcript gate with VAD/endpoint, confidence/coverage, and protected-slot no-guess dispositions while preserving no-live-provider/no-live-mic/no-execution scope.

Stage 8C is PROVEN_COMPLETE. It extends the existing Stage 8A/8B transcript gate with listening-scene, foreground-speaker, addressed-to-Selene, echo, noise, overlap, and barge-in boundary evidence while preserving advisory/no-execution scope. Stage 8D remains the next exact build for numeric listening-lab benchmarks and calibration.
