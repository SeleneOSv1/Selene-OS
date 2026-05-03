# Selene Canonical Golden Journey Matrix

Status: STAGE1_INITIAL_GOLDEN_JOURNEY_MATRIX
Date: 2026-05-03

These journeys are the first canonical regression set. Later stages must preserve them or document a stage-specific reason they are not yet applicable.

| ID | Journey | Primary Packets | Owning Stages | Stage 1 Status |
|---|---|---|---|---|
| GJ-01 | Wake Selene on Desktop, ask a public question, receive a spoken answer. | ActivationPacket, SessionPacket, CommittedTurnPacket, UnderstandingPacket, EvidencePacket, WriteResponsePacket, TtsPacket | 4, 5, 7, 8, 10, 13, 15, 17, 19, 34 | DRAFT |
| GJ-02 | Interrupt Selene mid-answer and get clean cancellation/resume behavior. | ConversationGoalStatePacket, ResponsivenessStatePacket, TtsPacket, AuditTracePacket | 5, 8, 17, 34 | DRAFT |
| GJ-03 | Ask a current search question and get accepted source chips only. | SearchQueryPlanPacket, SearchResultPacket, SourceVerificationPacket, CitationPacket, PresentationEnvelope | 13, 15, 16, 18, 33, 34 | DRAFT |
| GJ-04 | Ask weather or time in another language and get a structured vertical answer. | UnderstandingPacket, RealtimeVerticalPacket, StructuredToolEvidencePacket, WriteResponsePacket | 10, 13, 15, 32, 34 | DRAFT |
| GJ-05 | Record a meeting, summarize it, and draft reminders without sending. | RecordSessionPacket, AudioArtifactPacket, EvidencePacket, WriteResponsePacket, FailClosedResponsePacket | 4, 8, 22, 25, 27, 34 | DRAFT |
| GJ-06 | Draft an onboarding link but block delivery until protected gates pass. | RouteCandidatePacket, RiskDecisionPacket, SimulationResultPacket, ExecutionApprovalPacket | 6, 12, 25, 26, 34 | DRAFT |
| GJ-07 | Ask Selene what she remembers, correct one item, and forget another. | MemoryContextPacket, MemoryTrustPacket, MemoryForgetProofPacket | 21, 31, 34 | DRAFT |
| GJ-08 | Switch from Desktop to iPhone without client authority drift. | SessionPacket, AdapterResponsePacket, ClientRenderPacket, AuditTracePacket | 5, 18, 19, 20, 33, 34 | DRAFT |
| GJ-09 | Ask for a sensitive action from an unknown speaker and fail closed. | VoiceIdentityPacket, AccessContextPacket, AuthorityDecisionPacket, FailClosedResponsePacket | 6, 8, 9, 12, 34 | DRAFT |
| GJ-10 | Ask while frustrated and receive calm, bounded repair behavior. | EmotionSignalPacket, EmotionBoundaryPacket, EmotionalGuidancePacket, WriteResponsePacket | 10, 15, 21, 29, 34 | DRAFT |
| GJ-11 | Build and use a governed custom assistant with scoped knowledge/actions. | AssistantDefinitionPacket, AssistantKnowledgePacket, AssistantActionManifestPacket | 24, 30, 31, 34 | DRAFT |
| GJ-12 | Open an interactive app card and complete a read-only app task. | AppCapabilityPacket, AppInvocationPacket, InteractiveAppCardPacket | 16, 18, 24, 33, 34 | DRAFT |
| GJ-13 | Run visible data analysis and render an interactive chart. | DataSandboxPacket, InteractiveTablePacket, InteractiveChartPacket | 16, 22, 33, 34 | DRAFT |
| GJ-14 | Share/export a canvas artifact and restore an older version. | CanvasSharePacket, CanvasExportPacket, CanvasVersionPacket | 16, 23, 31, 34 | DRAFT |
| GJ-15 | Enter study mode, get Socratic guidance, and take a quiz. | StudySessionPacket, TutorStepPacket, QuizPracticePacket | 15, 16, 21, 29, 34 | DRAFT |
| GJ-16 | Compare products with source-backed shopping cards and no hidden purchase. | ProductSearchPacket, ProductEvidencePacket, ProductCardPacket, MerchantLinkPacket | 13, 16, 24, 31, 34 | DRAFT |
| GJ-17 | Generate or edit an image/video artifact without treating it as evidence. | GeneratedVideoPacket, EditedVideoPacket, ImageEvidencePacket, CanvasVersionPacket | 23, 28, 31, 34 | DRAFT |
| GJ-18 | Read a PDF/URL, extract table evidence, and cite claims safely. | UrlFetchPacket, PageReadPacket, PdfReadPacket, TableExtractPacket, CitationExtractPacket | 13, 15, 16, 22, 34 | DRAFT |
| GJ-19 | Ask a scrambled protected command and receive one clarification or fail-closed. | MeaningHypothesisPacket, ProtectedSlotConfidencePacket, ClarificationQuestionPacket | 8, 10, 12, 34 | DRAFT |
| GJ-20 | Deep research report with claim ledger, contradictions, rejected sources, and as-of date. | DeepResearchPacket, ClaimLedgerPacket, CitationGraphPacket, ContradictionMatrixPacket, RejectedSourceLogPacket | 13, 14, 15, 16, 34 | DRAFT |

## Golden Journey Protection Rule

Every future major build must either:

- add a proof case for any affected journey;
- mark the journey `NOT_APPLICABLE_WITH_REASON` for that build; or
- mark it `BLOCKED_WITH_OWNER_AND_NEXT_ACTION`.

No build may silently weaken provider-off behavior, protected fail-closed behavior, display/TTS separation, source-chip integrity, memory boundaries, or native-client renderer-only law for these journeys.

