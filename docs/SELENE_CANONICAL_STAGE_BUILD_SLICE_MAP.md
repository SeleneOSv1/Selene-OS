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

## Stage 5 - Session Open, Resume, Close, Runtime Turn Spine

| Slice | Focus | Input | Output | Proof |
|---|---|---|---|---|
| 5A | Session lifecycle/current-turn authority/stale-turn quarantine | Stage4TurnBoundaryPacket, SessionPacket | Stage5TurnAuthorityPacket | PROVEN_COMPLETE: only current committed turns enter understanding/render as current; stale/superseded/cancelled/abandoned/record/closed dispositions quarantine without route authority. |
| 5B | Conversation control, clarification, correction, recovery, same-page state | Stage5TurnAuthorityPacket | ConversationGoalStatePacket, OpenLoopsPacket, SamePageCheckPacket | PROVEN_COMPLETE: only current Stage 5A authority can update advisory conversation state; stale/record/closed turns are blocked; one-question clarification, session-scoped correction, safe backchannel, and no provider/tool/protected execution proof. |

## Stage 8 - Voice I/O, Listen State, Transcript Gate

| Slice | Focus | Input | Output | Proof |
|---|---|---|---|---|
| 8A | Audio substrate and listen-state reconciliation | ActivationPacket, SessionPacket | AudioScenePacket, ResponsivenessStatePacket | no execution from audio substrate |
| 8B | VAD/endpointing/partial-vs-final transcript | AudioScenePacket | TurnCandidatePacket, CommittedTurnPacket | partial transcript no-execution, final transcript commit |
| 8C | Listening lab scene/noise/echo/diarization | AudioScenePacket | ForegroundSpeakerPacket, AddressedToSelenePacket | background/non-user speech block |
| 8D | Transcript confidence, alternatives, protected slot gate | CommittedTurnPacket | ProtectedSlotConfidencePacket, ClarificationQuestionPacket | no protected audio guessing |
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

## Next Slice After Stage 6A

```text
Stage 7A - Wake, Side Button, And Activation Stack Reconciliation
```

Stage 6A is PROVEN_COMPLETE. It added the minimal runtime-owned `Stage6AccessContextPacket` carrier, preserved PH1.ACCESS/PH2.ACCESS, PH1.TENANT, PH1.POLICY, PH1.GOV, PH1.VOICE.ID, Stage 3 consent/provider-safety, and Stage 5 current-turn/conversation carriers as repo truth, and did not build protected execution, search, voice, wake, native UI, or provider/model routing.
