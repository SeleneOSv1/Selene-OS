Selene Provider-First OpenAI Assisted Pivot Master Build Plan

TASK:
SELENE_PROVIDER_FIRST_OPENAI_ASSISTED_PIVOT_MASTER_BUILD_PLAN

BUILD CLASS:
ARCHITECTURE / IMPLEMENTATION MASTER PLAN

CORE DIRECTION:
Move Selene toward OpenAI-assisted probabilistic intelligence behind Selene-owned provider interfaces, while keeping deterministic enterprise control inside existing canonical Selene engines.

---

# Master Architecture Build Set

This document is part of the Selene Master Architecture Build Set. Read it together with the [full architecture index](SELENE_MASTER_ARCHITECTURE_BUILD_SET.md). AGENTS.md remains controlling execution law.

# Related Provider-First Reference Documents

These documents must be read together as one provider-first reference set:

1. [Selene Provider-First OpenAI Assisted Pivot Master Build Plan](SELENE_PROVIDER_FIRST_OPENAI_ASSISTED_PIVOT_MASTER_BUILD_PLAN.md)
   Controlling architecture / law / direction.

2. [Selene Provider-First Function Architecture Cards](SELENE_PROVIDER_FIRST_FUNCTION_ARCHITECTURE_CARDS.md)
   Function-by-function design reference.

3. [Selene Provider-First Vertical Slice Build Pack](SELENE_PROVIDER_FIRST_VERTICAL_SLICE_BUILD_PACK.md)
   First executable slice per function.

4. [Selene OpenAI Model Routing Policy](SELENE_OPENAI_MODEL_ROUTING_POLICY.md)
   JD-controlled model routing policy. Codex must not choose, change, upgrade, downgrade, replace, fallback, or cost-optimize OpenAI models without explicit JD approval.

The model routing policy controls OpenAI model choices until JD explicitly changes it.

Codex must not treat any one of these documents as standalone implementation approval.
Implementation instructions must be derived from the full reference set plus AGENTS.md.

0. Executive Target

Selene’s target architecture is:

OpenAI provides probabilistic intelligence.
Selene owns deterministic control, evidence, memory, identity, authority, audit, and protected execution.

This must not become:

Selene = OpenAI wrapper
OpenAI = business authority
Desktop = second brain
Adapter = semantic brain
PH1.X2 / PH1.M2 / PH1.WRITE2
manual phrase-patch system

The correct architecture is:

Provider-first pivot
inside existing canonical engines
old paths removed only after proof
future provider replaceable

OpenAI becomes the first high-quality provider for language, voice, writing, search, vision, embeddings, and evaluation. Selene remains the product, the runtime, the governance system, the memory authority, the identity authority, the evidence system, and the protected-execution system.

1. Core Architecture Rule

Every external AI capability must sit behind a Selene-owned provider interface.

OpenAI Provider
    ↓
Selene Provider Interface
    ↓
Selene Canonical Packet
    ↓
Existing Canonical Engine
    ↓
Selene Runtime Decision / Evidence / UI / TTS / Audit

OpenAI responses must never directly control PH1.X, PH1.M, PH1.WRITE, PH1.E, protected execution, Desktop, database mutation, memory writes, or authority decisions.

Selene engines must consume Selene packets, not raw OpenAI responses.

Correct:

OpenAI structured output
→ SemanticInterpreterProvider
→ CurrentTurnInterpretationPacket
→ PH1.X validates and routes

Wrong:

OpenAI says route = payroll_approval
→ Adapter executes payroll

2. Provider-First Ownership Model

OpenAI may provide

probabilistic language understanding
messy speech transcription
speech repair suggestions
realtime voice substrate
text-to-speech audio
writing/rewrite/summarisation
structured JSON proposals
web search assistance
deep research assistance
file search assistance
vision/image understanding
image generation/editing
embeddings
moderation signals
eval/judging support
code-interpreter-style advisory analysis
MCP/connector access proposals

Selene must own

wake boundary
runtime truth
turn admission
PH1.X routing validation
PH1.M memory permission and evidence
Voice ID speaker scope
PH1.E tool/search policy and source acceptance
PH1.WRITE final answer policy and display/TTS contract
PH1.L session/sleep boundary
PH1.K interruption control
storage/audit/black-box evidence
provider budget/cost governance
protected simulation execution
authority validation
business state mutation
company data mutation
final acceptance tests

Non-negotiable principle

OpenAI proposes.
Selene validates.
Selene decides.
Selene executes only when lawful.
Selene audits.

3. Canonical-Owner Rehabilitation Rule

This pivot must happen inside existing canonical engines.

Do not create:

PH1.X2
PH1.M2
PH1.WRITE2
NewSearchBrain
NewVoiceBrain
NewDesktopBrain
OpenAIExecutionEngine
parallel repo
parallel runtime
parallel adapter brain

Instead:

PH1.X keeps ownership of live turn/context/routing.
PH1.M keeps ownership of governed human memory.
PH1.WRITE keeps ownership of final human presentation.
PH1.E keeps ownership of tool/search/evidence routing.
PH1.C keeps ownership of transcript admission and STT evidence.
PH1.TTS keeps ownership of approved voice output evidence.
PH1.K keeps ownership of interruption/barge-in control.
PH1.W keeps ownership of wake.
Desktop/iPhone remain capture/playback/render shells.
Adapter remains transport bridge.

The correct replacement pattern is:

inside canonical owner:
    add clean provider-backed internal module
    migrate useful working logic
    prove live
    remove old duplicate/rubbish path
    end with one canonical path

Example:

PH1.X old active-context shortcuts
→ PH1.X provider-assisted interpretation packet
→ PH1.X deterministic candidate validator
→ live proof
→ remove old adapter/time/weather phrase shortcuts
→ one clean PH1.X

4. AGENTS Law Alignment

Every Codex implementation instruction derived from this plan must obey the active Selene AGENTS law.

Mandatory Codex rules to preserve:

No Python in repo work.
Shell-only inspection.
Start and end clean tree.
Fresh fetch before remote equality claims.
Mandatory first-read files before major work.
Existing files read-only by default.
Explicit file-scope approval before editing existing files.
Spine/contract/shared behavior requires explicit JD approval.
Declare execution lane before editing.
Reuse existing owner before creating anything new.
No duplicate implementation.
Correct owner repair only.
No nearest-layer patching.
No Desktop semantic authority.
No Adapter semantic brain.
No phrase patches.
No exact JD prompt hardcoding.
No real searched-name hardcoding in code/tests/fixtures.
Provider calls require kill switch, budget, counters, evidence.
Normal tests use fake providers.
Live provider calls require explicit opt-in and cap.
Voice-first smoke where practical.
JD live acceptance required for user-visible voice/UI/context/memory/search/protected behavior.
Backend evidence must match visible behavior.
No build is complete until final tree is clean.

This plan does not override AGENTS law. It supplies the architecture direction for future implementation builds.

5. Execution Lane Model

Every build must declare one of these lanes.

Lane A — Probabilistic Public / Advisory Intelligence

Applies to:

normal chat
writing
summarisation
translation
public search
public deep research
file Q&A over user-provided files
image understanding
spreadsheet/report drafting
advisory analysis
public weather/time/news answers

Rules:

simulation not required
authority not required
state mutation not allowed
protected execution not allowed
provider degradation allowed
normal answer allowed

Lane B — Deterministic Protected Execution

Applies to:

payroll execution
salary change
leave approval
database write
inventory update
customer record mutation
official company record action
bank/payment/accounting execution
access control change

Rules:

simulation required
authority required
audit required
deterministic workflow required
fail closed if missing simulation or authority
OpenAI cannot execute

Lane C — Mixed Request

Example:

Search salary trends and increase Tim’s salary.

Correct split:

salary trend research → public/advisory lane
salary increase → protected execution lane

Never block the public/advisory part merely because the protected part fails closed. Never allow the protected part merely because the advisory part is allowed.

6. Provider Interface Inventory

Codex must first discover existing provider/control surfaces in repo truth. Only then may it add or extend contracts inside approved scope.

Provider interfaces must be Selene-owned and provider-agnostic.

Required provider interfaces

ModelReasoningProvider
SemanticInterpreterProvider
WritingProvider
SpeechToTextProvider
RealtimeTranscriptionProvider
TextToSpeechProvider
RealtimeVoiceProvider
RealtimeTransportProvider
VoiceActivityDetectionPolicy
LiveTranslationProvider
RealtimeSessionControlProvider
RealtimeServerSideControlProvider
RealtimeCostPolicy
VisionProvider
ImageGenerationProvider
SearchProvider
DeepResearchProvider
FileSearchProvider
FileLifecycleProvider
ProviderFileInputPolicy
EmbeddingProvider
VectorStoreProvider
RetrievalProvider
ModerationProvider
EvalProvider
ToolProposalProvider
ToolSearchProvider
McpConnectorProvider
CodeInterpreterProvider
ResponsesOrchestrationProvider
ResponsesWebSocketProvider
VideoGenerationProvider
ComputerUseProvider
AgentPlatformIntegrationProvider
ChatGptAppsDistributionProvider
ChatGptConnectorSurface
BatchProcessingProvider
BackgroundJobProvider
ContextCompactionProvider
PromptCachingPolicy
FineTuningOptimizationProvider
PromptOptimizationProvider
GraderProvider
ReasoningControlProvider
ProviderConversationStatePolicy
ProviderStreamingTransport
ProviderTokenBudgetPolicy
UsageTelemetryProvider
ProviderCredentialGovernance
ProviderWebhookReceiver
ProviderProcessingPriorityPolicy
PredictedOutputPolicy
TokenCountingProvider
CitationFormattingProvider
SkillProvider
SkillPackageProvider
DeveloperToolProvider
SafetyPolicyReferenceSurface
ProviderAdminTelemetryProvider
EnterpriseWorkspaceReferenceSurface
ManualPromptTestingSurface
CodexDevelopmentAssistantSurface
ProviderDataEgressPolicy
ProviderPrivacyBoundaryPolicy
PromptInjectionDefensePolicy
ProviderTraceProvider
ProviderHealthProvider
ProviderCircuitBreakerPolicy
GeneratedArtifactGovernanceProvider
ModelGovernanceProvider
ProviderContractVersioningPolicy

These provider interfaces are the normalized coverage inventory for the master plan. Codex must treat this list, Section 8A, Section 8B, Phase 1 capability examples, and Section 28 build order as one synchronized provider-coverage set.

These are interfaces, not new brains.

Each provider must return Selene-owned packets only.

7. Canonical Packet Contracts

The provider layer must convert provider-native responses into canonical Selene packets.

Required packet families:

ProviderCallRequestPacket
ProviderCallResultPacket
ProviderCostEvidencePacket
ProviderLatencyEvidencePacket
ProviderFailurePacket
ProviderTokenBudgetPacket
ProviderHealthPacket
ProviderCircuitBreakerPacket
ProviderTracePacket
ProviderDecisionTracePacket
ProviderContractVersionPacket
ProviderCapabilityVersionPacket
ProviderModelVersionPacket
ProviderDataEgressPacket
ProviderPrivacyBoundaryPacket
ProviderRedactionPacket
PromptInjectionDefensePacket
ModelSelectionPacket
ModelGovernancePacket
ModelFallbackPacket
TranscriptPacket
RealtimeTranscriptionPacket
RealtimeVoiceEventPacket
RealtimeTransportEvidencePacket
VoiceActivityDetectionPacket
LiveTranslationPacket
RealtimeSessionControlPacket
RealtimeServerSideControlPacket
RealtimeCostPacket
VoiceOutputPacket
InterruptionPacket
CurrentTurnInterpretationPacket
HumanConversationDirective
WriteRequestPacket
WriteOutputPacket
SearchEvidencePacket
SourceAcceptancePacket
DeepResearchEvidencePacket
FileEvidencePacket
ProviderFileInputPacket
ProviderFileLifecyclePacket
VisualEvidencePacket
ImageGenerationPacket
GeneratedArtifactPacket
ArtifactProvenancePacket
EmbeddingPacket
MemoryEvidencePacket
ModerationSignalPacket
EvalResultPacket
ToolProposalPacket
ToolExecutionDecisionPacket
McpConnectorEvidencePacket
CodeInterpreterEvidencePacket
ProviderSkillEvidencePacket
DeveloperToolEvidencePacket
ProviderConversationStateRefPacket
ProviderStreamEventPacket
ProviderStreamFinalizationPacket
SourceChipPacket
CitationPresentationPacket
ProviderAdminTelemetryPacket
ProviderUsageCostPacket
SafetyPolicyReferencePacket
ProviderWebhookEventPacket
AnalysisArtifactPacket
VectorStoreEvidencePacket
RetrievalEvidencePacket
VideoGenerationPacket
ComputerUseEvidencePacket
BatchJobEvidencePacket
BackgroundJobEvidencePacket
ContextCompactionPacket
FineTuningOptimizationEvidencePacket
PromptOptimizationEvidencePacket
GraderResultPacket
ProviderCredentialEvidencePacket
ProviderProcessingPriorityPacket
PredictedOutputEvidencePacket
TokenCountingEvidencePacket
JDLiveTestTracePacket
BackendEvidenceVerificationPacket

These packet families are the normalized packet inventory for provider-first implementation. If a future provider surface needs a new packet, Codex must update this section, Section 8A/8B classification, and the capability registry together rather than creating an isolated packet path.

No raw OpenAI JSON may leak into:

response_text
tts_text
Desktop normal UI
public trace
memory records
protected execution packets

Raw provider metadata may exist only in bounded debug/evidence storage if allowed by repo law and privacy rules.

8. Provider Governance Layer

Before using OpenAI broadly, Selene needs provider governance that applies to every provider.

Required controls:

global provider kill switch
provider-specific enable flag
paid-provider enable flag
per-turn budget packet
per-session budget packet
per-tenant budget packet
pre-network call counter
network dispatch counter
latency evidence
failure reason
retry cap
route cap
model selection policy
provider degradation policy
provider-off safe result
fake provider support for tests
live provider opt-in flags
no startup provider probes
no background provider calls without explicit task
no provider call before lawful wake when voice path is pre-wake

Provider governance must be provider-agnostic.

Today:

OpenAIProvider

Future:

LocalModelProvider
SeleneModelProvider
AnthropicProvider
GoogleProvider
CustomEnterpriseProvider

No engine should care which provider fulfilled the packet.

8A. OpenAI Service Coverage Classification Layer

This master build plan must include every OpenAI service surface from the Selene OpenAI Services Integration Map and classify each service by architectural role.

These categories are mandatory coverage labels for future Codex implementation instructions and service-gap reviews:

CORE_RUNTIME
VOICE_RUNTIME
WRITE_PRESENTATION
SEARCH_EVIDENCE
MEMORY_RETRIEVAL
ADVISORY_ANALYSIS
EVAL_OPTIMIZATION
COST_SCALE
DISTRIBUTION_SURFACE
ENTERPRISE_ADMIN
FUTURE_OPTIONAL

The classification is not decorative. It tells Codex where each service belongs, what canonical Selene owner must consume it, and what must not happen.

8A.1 Category definitions

CORE_RUNTIME

Core runtime services are used for main assistant intelligence, structured interpretation, reasoning, tool proposal, orchestration, and provider-backed decision support. They must feed Selene-owned packets and must not become authority.

VOICE_RUNTIME

Voice runtime services support STT, TTS, realtime voice, duplex conversation, interruption, and audio events. Selene still owns wake, transcript admission, TTS approval, interruption control, and session boundary.

WRITE_PRESENTATION

Writing and presentation services support rewriting, summarisation, tone, structure, formatting, translation, document drafting, email/message drafting, and final answer shaping. PH1.WRITE remains the owner.

SEARCH_EVIDENCE

Search/evidence services support public search, deep research, source discovery, source comparison, citations, file search, and evidence retrieval. PH1.E remains the owner of search policy, source acceptance, claim verification, source chips, and provider budget.

MEMORY_RETRIEVAL

Memory/retrieval services support embeddings, vector retrieval, file retrieval, semantic recall, topic clustering, and long-context compression. PH1.M remains the owner of memory permission, scope, freshness, conflict, and recall wording.

ADVISORY_ANALYSIS

Advisory analysis services support data analysis, spreadsheets, charts, report drafts, document transformation, and advisory business summaries. These services must not execute official accounting, payroll, banking, or protected business mutation.

EVAL_OPTIMIZATION

Evaluation/optimization services support test corpora, graders, prompt optimization, behavior scoring, regression checks, and model/output quality measurement. They do not replace backend evidence or JD live acceptance.

COST_SCALE

Cost/scale services support batch processing, background jobs, prompt caching, flex/priority processing, usage tracking, cost routing, resumability, and long-running workflows. They must be governed by provider budget/counter/evidence rules.

DISTRIBUTION_SURFACE

Distribution surfaces allow Selene or Selene connectors to appear inside external product surfaces such as ChatGPT apps, MCP connectors, Apps SDK, ChatGPT developer mode, or future app directories. They are not core Selene runtime authority.

ENTERPRISE_ADMIN

Enterprise/admin services support workspace controls, organization controls, API key management, SSO, admin roles, usage dashboards, compliance posture, and tenant/customer deployment design. They inform Selene’s enterprise model but do not replace Selene governance.

FUTURE_OPTIONAL

Future optional services may be useful later, such as video generation, advanced computer use, fine-tuning variants, agent-builder UX, or non-core media workflows. They must not distract from the core provider-first runtime pivot.

8A.2 Required OpenAI service coverage map

Every service below must either have a provider interface, a future provider placeholder, or an explicit deferred classification.

Core language / reasoning models
Category: CORE_RUNTIME
Selene owner: PH1.X / PH1.WRITE / PH1.E through provider packets
Provider surface: ModelReasoningProvider / SemanticInterpreterProvider / WritingProvider
Rule: model output proposes; Selene validates and decides.

Responses API
Category: CORE_RUNTIME
Selene owner: provider orchestration layer / PH1.X / PH1.WRITE / PH1.E
Provider surface: ResponsesOrchestrationProvider
Rule: Responses may become primary OpenAI transport, but canonical engines must consume Selene packets only.

Structured Outputs / JSON schema
Category: CORE_RUNTIME
Selene owner: PH1.X / PH1.WRITE / PH1.E validators
Provider surface: SemanticInterpreterProvider / WritingProvider / ToolProposalProvider
Rule: structured output is a proposal format, not execution authority.

Reasoning controls / reasoning effort / reasoning summaries / encrypted reasoning handling
Category: CORE_RUNTIME / COST_SCALE
Selene owner: provider router and governance
Provider surface: ReasoningControlProvider or provider policy fields
Rule: reasoning effort is a routing/cost setting, not a protected-action authority signal.

Function calling / tool calling
Category: CORE_RUNTIME
Selene owner: PH1.E / PH1.X / SimulationExecutor for protected actions
Provider surface: ToolProposalProvider
Rule: OpenAI may propose tool calls; Selene decides and executes only lawful tools.

Tool Search
Category: CORE_RUNTIME / SEARCH_EVIDENCE
Selene owner: PH1.E / tool registry / simulation catalog boundary
Provider surface: ToolSearchProvider
Rule: tool search may discover candidate tools, but Selene owns tool truth, permission, schemas, and execution.

Realtime API / voice agents
Category: VOICE_RUNTIME
Selene owner: PH1.W / PH1.C / PH1.K / PH1.TTS / PH1.L
Provider surface: RealtimeVoiceProvider
Rule: provider supplies realtime audio events; Selene owns wake, interruption, admission, session, and protected gates.

Speech-to-Text
Category: VOICE_RUNTIME
Selene owner: PH1.C
Provider surface: SpeechToTextProvider
Rule: provider transcribes; PH1.C decides transcript commit/reject.

Text-to-Speech
Category: VOICE_RUNTIME
Selene owner: PH1.TTS / Desktop playback evidence
Provider surface: TextToSpeechProvider
Rule: provider generates audio only for approved_tts_text.

Responses WebSocket Mode
Category: CORE_RUNTIME / COST_SCALE
Selene owner: provider orchestration layer / adapter transport boundary
Provider surface: ResponsesWebSocketProvider or transport mode under ResponsesOrchestrationProvider
Rule: useful for persistent non-voice workflows; must not bypass canonical packets or provider evidence.

Writing assistance / advanced text generation / rewriting
Category: WRITE_PRESENTATION
Selene owner: PH1.WRITE
Provider surface: WritingProvider
Rule: OpenAI writes; PH1.WRITE validates facts, sources, memory wording, protected wording, display_text, and tts_text.

ChatGPT Canvas-style collaborative writing concepts
Category: WRITE_PRESENTATION / DISTRIBUTION_SURFACE
Selene owner: PH1.WRITE / Desktop renderer
Provider surface: WritingProvider plus artifact/edit packet support
Rule: canvas-style iteration may inspire Selene artifacts, but Desktop must not become writing brain.

Prompt engineering / structured writing style control
Category: WRITE_PRESENTATION / CORE_RUNTIME
Selene owner: PH1.WRITE / provider prompt governance
Provider surface: WritingProvider
Rule: style control must be data-driven and validated; no phrase-patch behavior.

Files API
Category: SEARCH_EVIDENCE / MEMORY_RETRIEVAL / ADVISORY_ANALYSIS
Selene owner: PH1.E / PH1.M / Storage
Provider surface: FileLifecycleProvider
Rule: file upload/storage lifecycle is separate from FileSearchProvider; Selene owns permissions, scope, retention, and evidence.

Built-in File Search tool
Category: SEARCH_EVIDENCE / MEMORY_RETRIEVAL
Selene owner: PH1.E / PH1.M
Provider surface: FileSearchProvider
Rule: retrieves relevant chunks; Selene owns permission, claim verification, and memory use.

Vector Stores / Retrieval
Category: MEMORY_RETRIEVAL / SEARCH_EVIDENCE
Selene owner: PH1.M / Storage / PH1.E when used for evidence
Provider surface: VectorStoreProvider / RetrievalProvider
Rule: vector retrieval supports recall/evidence but does not own memory truth.

Embeddings API
Category: MEMORY_RETRIEVAL
Selene owner: PH1.M
Provider surface: EmbeddingProvider
Rule: embeddings support retrieval; PH1.M owns memory object creation and scope.

Web Search
Category: SEARCH_EVIDENCE
Selene owner: PH1.E / PH1.WRITE source presentation
Provider surface: SearchProvider
Rule: public read-only provider; accepted/rejected source discipline and claim verification remain Selene-owned.

Deep Research
Category: SEARCH_EVIDENCE / ADVISORY_ANALYSIS / COST_SCALE
Selene owner: PH1.E / PH1.WRITE / provider governance
Provider surface: DeepResearchProvider
Rule: explicit intent or approved escalation, capped, auditable, advisory by default, never protected execution.

Vision API / image understanding
Category: SEARCH_EVIDENCE / ADVISORY_ANALYSIS / MEMORY_RETRIEVAL
Selene owner: PH1.E / PH1.M / PH1.WRITE depending on use
Provider surface: VisionProvider
Rule: vision output is evidence/proposal, not guaranteed truth or identity authority.

Image Generation / editing
Category: WRITE_PRESENTATION / FUTURE_OPTIONAL
Selene owner: PH1.WRITE / media asset governance / Desktop renderer
Provider surface: ImageGenerationProvider
Rule: generated images are creative assets, not real evidence.

Sora / Video Generation
Category: FUTURE_OPTIONAL / WRITE_PRESENTATION
Selene owner: marketing/media module later, PH1.WRITE for prompts/copy, asset governance
Provider surface: VideoGenerationProvider
Rule: useful for marketing, training, onboarding, and social assets later; not core runtime.

Computer Use
Category: FUTURE_OPTIONAL / ADVISORY_ANALYSIS / DISTRIBUTION_SURFACE
Selene owner: PH1.E / SimulationExecutor for protected workflows
Provider surface: ComputerUseProvider
Rule: high risk; must not control business systems without simulation, authority, and audit.

Agents platform / Agents SDK / Agent Builder / ChatKit
Category: DISTRIBUTION_SURFACE / FUTURE_OPTIONAL
Selene owner: Selene product boundary / provider orchestration / UI strategy
Provider surface: AgentPlatformIntegrationProvider if ever used
Rule: may inspire distribution or UI, but must not create an OpenAI-run parallel Selene brain.

Remote MCP + OpenAI-maintained connectors + Secure MCP Tunnel
Category: DISTRIBUTION_SURFACE / SEARCH_EVIDENCE / CORE_RUNTIME
Selene owner: PH1.E / connector governance / Access/Gov / SimulationExecutor for writes
Provider surface: McpConnectorProvider
Rule: connector retrieval/proposal allowed where permissioned; protected writes require simulation + authority.

ChatGPT Connectors / Developer Mode
Category: DISTRIBUTION_SURFACE / ENTERPRISE_ADMIN
Selene owner: connector strategy / PH1.E / Access/Gov
Provider surface: ChatGptConnectorSurface or McpConnectorProvider
Rule: useful for testing and distribution; not a replacement for Selene runtime authority.

ChatGPT Apps SDK / Apps Directory
Category: DISTRIBUTION_SURFACE
Selene owner: product distribution strategy / external app boundary
Provider surface: ChatGptAppsDistributionProvider if ever used
Rule: optional way to expose Selene or Selene tools inside ChatGPT; not core runtime.

Code Interpreter / sandboxed data analysis
Category: ADVISORY_ANALYSIS
Selene owner: advisory analysis lane / PH1.WRITE summary / Storage evidence
Provider surface: CodeInterpreterProvider
Rule: allowed for advisory spreadsheet/report/chart/file analysis; forbidden for Codex repo work and protected official execution.

Batch API
Category: COST_SCALE / EVAL_OPTIMIZATION / ADVISORY_ANALYSIS
Selene owner: provider governance / offline jobs / eval harness / PH1.M consolidation
Provider surface: BatchProcessingProvider
Rule: useful for offline evals, archive summarisation, dataset generation, and analytics; not interactive authority.

Background Mode / resumable long-running work
Category: COST_SCALE / ADVISORY_ANALYSIS / SEARCH_EVIDENCE
Selene owner: provider governance / job status ledger / Storage
Provider surface: BackgroundJobProvider
Rule: useful for deep research and large reports; Selene owns status, audit, cancellation, and final presentation.

Prompt Caching
Category: COST_SCALE
Selene owner: provider governance / prompt policy
Provider surface: PromptCachingPolicy under provider router
Rule: cost/latency optimization only; must not alter semantics or authority.

Context Compaction
Category: COST_SCALE / MEMORY_RETRIEVAL / CORE_RUNTIME
Selene owner: PH1.X / PH1.M / provider governance
Provider surface: ContextCompactionProvider
Rule: may reduce long-context cost, but must not replace PH1.M governed memory or PH1.X active frame truth.

Fine-tuning / SFT / DPO / RFT / Vision Fine-Tuning
Category: EVAL_OPTIMIZATION / FUTURE_OPTIONAL
Selene owner: model optimization strategy / eval harness
Provider surface: FineTuningOptimizationProvider
Rule: optional later; retrieval, vocab packs, correction pairs, and evals come first.

OpenAI Evals
Category: EVAL_OPTIMIZATION
Selene owner: eval harness / acceptance matrix
Provider surface: EvalProvider
Rule: evals support testing but do not replace real-path smoke, backend evidence, or JD live acceptance.

Prompt Optimizer / Datasets / Graders
Category: EVAL_OPTIMIZATION
Selene owner: eval harness / prompt governance
Provider surface: PromptOptimizationProvider / GraderProvider
Rule: supports improvement of prompts and tests; cannot approve protected actions.

Moderation API
Category: CORE_RUNTIME / ENTERPRISE_ADMIN
Selene owner: policy engine / PH1.WRITE refusal wording / Access/Gov for escalation
Provider surface: ModerationProvider
Rule: moderation is a safety signal, not final authority.

Usage Dashboard / Cost and Usage APIs
Category: COST_SCALE / ENTERPRISE_ADMIN
Selene owner: provider budget/cost governance / billing telemetry
Provider surface: UsageTelemetryProvider
Rule: monitor cost and usage; provider spend must be auditable by actor, tenant, module, route, and capability where available.

Authentication / API Keys / Org Controls
Category: ENTERPRISE_ADMIN
Selene owner: secrets governance / deployment ops / tenant admin model
Provider surface: ProviderCredentialGovernance
Rule: secrets must remain server-side; Desktop/iPhone must never hold provider secrets.

ChatGPT Business / Enterprise workspace features
Category: ENTERPRISE_ADMIN / DISTRIBUTION_SURFACE
Selene owner: enterprise deployment strategy / tenant model inspiration
Provider surface: EnterpriseWorkspaceReferenceSurface
Rule: benchmark/inspiration for admin, SSO, workspace, and connector controls; not Selene runtime authority.

Playground / prompt testing tools
Category: EVAL_OPTIMIZATION / FUTURE_OPTIONAL
Selene owner: prompt development workflow
Provider surface: ManualPromptTestingSurface
Rule: useful for prototyping only; repo truth and automated/live proof remain authoritative.

Codex as OpenAI service
Category: EVAL_OPTIMIZATION / ADVISORY_ANALYSIS / FUTURE_OPTIONAL
Selene owner: development workflow only
Provider surface: CodexDevelopmentAssistantSurface
Rule: Codex assists implementation but must obey AGENTS law and must not be treated as Selene runtime.

Webhooks
Category: COST_SCALE / DISTRIBUTION_SURFACE / ENTERPRISE_ADMIN
Selene owner: job/event gateway / audit / connector governance
Provider surface: ProviderWebhookReceiver
Rule: webhook events must be authenticated, bounded, idempotent, audited, and must not execute protected actions without simulation authority.

Priority / Flex processing
Category: COST_SCALE
Selene owner: provider routing/cost policy
Provider surface: ProviderProcessingPriorityPolicy
Rule: processing tier is cost/latency policy only; it must not affect correctness, authority, or audit requirements.

8A.3 Coverage enforcement rule

Every future OpenAI/provider-related Codex instruction must include a coverage classification for the service being touched:

service name:
coverage category:
canonical owner:
provider interface:
canonical packet:
allowed lane:
forbidden lane:
provider governance required:
protected execution impact:
old path retirement condition:

If Codex cannot classify the service, it must stop before editing and report:

OPENAI_SERVICE_COVERAGE_CLASSIFICATION_REQUIRED

8B. OpenAI Coverage Hardening Appendix

This appendix closes the remaining OpenAI service coverage gaps without changing the architecture.

The core architecture remains:

provider-first pivot
inside existing canonical engines
old paths removed only after proof
no duplicate engines
no parallel brains
no phrase patches
Selene stays in control

These services must be represented as provider surfaces, policy surfaces, transport modes, development-only surfaces, or deferred future surfaces. They must not become new engines or parallel brains.

8B.1 Skills

Skills
Category: CORE_RUNTIME / EVAL_OPTIMIZATION / FUTURE_OPTIONAL
Selene owner: provider prompt/tool governance / eval harness where applicable
Provider surface: SkillProvider or SkillPackageProvider
Canonical packet: ProviderSkillEvidencePacket or ProviderCallResultPacket extension
Allowed lane: public/advisory or eval only unless explicitly governed otherwise
Forbidden lane: protected execution authority
Rule: Skills may package reusable provider instructions/capabilities, but must not bypass Selene packets, PH1.X validation, PH1.E tool policy, PH1.WRITE validation, provider governance, or protected simulation gates.

Skills are not Selene engines. They are provider-side capability packages or instruction bundles. Codex must not treat a Skill as authority to execute business actions.

8B.2 Shell / Local Shell / Apply Patch

Shell / Local Shell / Apply Patch
Category: FUTURE_OPTIONAL / EVAL_OPTIMIZATION
Selene owner: development workflow only
Provider surface: DeveloperToolProvider
Canonical packet: DeveloperToolEvidencePacket if ever represented
Allowed lane: Codex/development support only
Forbidden lane: Selene runtime, protected execution, business mutation, Desktop runtime, Adapter runtime
Rule: These are developer/Codex tool surfaces only. They must never become Selene runtime tools, never execute protected business actions, never bypass AGENTS law, and never create repo edits outside approved Codex scope.

Important distinction:

Codex/development shell tooling may exist in the OpenAI ecosystem.
Selene product runtime must not gain arbitrary shell execution.

AGENTS law remains controlling for Codex repository work:

No Python in repo work.
Shell-only inspection rules apply.
Existing file edits require approval.
Clean tree required.

8B.3 Predicted Outputs

Predicted Outputs
Category: COST_SCALE / WRITE_PRESENTATION
Selene owner: provider routing/cost policy / PH1.WRITE
Provider surface: PredictedOutputPolicy
Canonical packet: ProviderCallRequestPacket policy field / ProviderCallResultPacket evidence field
Allowed lane: public/advisory writing, repeated output, low-latency drafting where safe
Forbidden lane: protected execution decision, memory truth, audit truth, authority validation
Rule: Predicted outputs are latency/cost optimization only. They must not alter facts, protected gates, memory truth, PH1.X routing truth, PH1.E source verification, PH1.WRITE final validation, or audit requirements.

PH1.WRITE must still validate final output before display/TTS.

8B.4 Conversation State

Conversation State
Category: CORE_RUNTIME / MEMORY_RETRIEVAL / COST_SCALE
Selene owner: PH1.X / PH1.M / provider governance
Provider surface: ProviderConversationStatePolicy
Canonical packet: ProviderConversationStateRefPacket
Allowed lane: public/advisory continuity support
Forbidden lane: replacing PH1.X active frame, replacing PH1.M governed memory, protected execution authority
Rule: Provider conversation state may support continuity and reduce prompt cost, but must not replace PH1.X active frame truth or PH1.M governed memory. Selene must be able to reconstruct state from Selene evidence, not provider memory alone.

Provider conversation state is a convenience layer. It is not the system of record.

8B.5 Streaming

Streaming
Category: CORE_RUNTIME / VOICE_RUNTIME / WRITE_PRESENTATION / COST_SCALE
Selene owner: Adapter transport / PH1.WRITE / PH1.TTS / PH1.K / Desktop renderer
Provider surface: ProviderStreamingTransport
Canonical packet: ProviderStreamEventPacket
Allowed lane: public/advisory output, voice output, writing progress, search progress where safe
Forbidden lane: protected execution commit, unverified tool result rendering, stale result rendering
Rule: Streaming is transport/display optimization only. Partial output must not commit protected actions, render stale tool results as final, bypass PH1.WRITE validation, or bypass TTS approval.

Streaming output must be distinguishable from final accepted output.

Required states:

streaming_started
partial_output_received
validation_pending
final_output_accepted
final_output_rejected
stream_cancelled

8B.6 Counting Tokens

Counting Tokens
Category: COST_SCALE
Selene owner: provider governance / budget router
Provider surface: TokenCountingProvider or ProviderTokenBudgetPolicy
Canonical packet: ProviderTokenBudgetPacket
Allowed lane: all provider lanes for budgeting and routing
Forbidden lane: semantic authority, protected authority, memory truth, source verification
Rule: Token counting is used for budget, context limits, route selection, and cost prediction. It must not become semantic authority or evidence of factual correctness.

Token counting should support:

pre-call estimate
post-call actual usage
per-turn budget
per-session budget
per-tenant budget
provider route decision
cost evidence

8B.7 Citation Formatting

Citation Formatting
Category: SEARCH_EVIDENCE / WRITE_PRESENTATION
Selene owner: PH1.E / PH1.WRITE / presentation contract
Provider surface: CitationFormattingProvider or PH1.WRITE citation formatter policy
Canonical packet: SourceChipPacket / CitationPresentationPacket
Allowed lane: public search, file Q&A, deep research, advisory reports with evidence
Forbidden lane: invented citations, unsupported claim presentation, protected execution proof substitution
Rule: Citation formatting may help presentation, but PH1.E still owns accepted sources and claim verification. PH1.WRITE must not invent citations, upgrade weak evidence, hide uncertainty, or display rejected sources as accepted.

Citation formatting must support:

small source chips
safe source-page links
accepted sources only
no raw source dump
no rejected source exposure in normal UI
TTS-safe citation omission or short spoken source summary where appropriate

8B.8 Realtime Transport Modes: WebRTC / WebSocket / SIP

Realtime WebRTC / WebSocket / SIP
Category: VOICE_RUNTIME / DISTRIBUTION_SURFACE / COST_SCALE
Selene owner: PH1.W / PH1.C / PH1.K / PH1.TTS / PH1.L / Adapter
Provider surface: RealtimeTransportProvider
Canonical packet: RealtimeTransportEvidencePacket / RealtimeVoiceEventPacket
Allowed lane: voice runtime, explicit voice sessions, possible future telephony gateway
Forbidden lane: bypassing wake, transcript admission, interruption policy, TTS approval, or protected execution gates
Rule: Transport mode is replaceable. Selene owns wake, admission, session boundary, interruption, TTS approval, evidence, and protected gates regardless of whether the transport is WebRTC, WebSocket, SIP, or another future protocol.

Transport-specific use:

WebRTC → browser/mobile low-latency voice sessions
WebSocket → server/app controlled persistent voice or non-voice realtime sessions
SIP → future phone/telephony integration if explicitly approved

No transport mode may become business authority.

8B.9 OpenAI Admin / Audit / Usage / Cost APIs

OpenAI Admin / Audit / Usage / Cost APIs
Category: ENTERPRISE_ADMIN / COST_SCALE
Selene owner: provider governance / billing telemetry / audit mirror / deployment ops
Provider surface: ProviderAdminTelemetryProvider
Canonical packet: ProviderAdminTelemetryPacket / ProviderUsageCostPacket
Allowed lane: admin telemetry, spend monitoring, provider audit reconciliation
Forbidden lane: replacing Selene audit ledger, protected action proof, authority approval
Rule: OpenAI admin telemetry may inform cost and provider-audit reporting, but Selene’s own audit ledger remains authoritative for Selene actions.

Required governance mapping:

provider account
organization/project
actor/user when available
tenant/company/private-user scope
service/module
capability
route
operation type
cost owner
billable class
usage amount
blocked/non-billable status

OpenAI usage telemetry is provider evidence, not Selene business execution evidence.

8B.10 Safety / Cybersecurity / Under-18 Guidance

OpenAI Safety / Cybersecurity / Under-18 Guidance
Category: ENTERPRISE_ADMIN / CORE_RUNTIME
Selene owner: policy governance / Access-Gov / PH1.WRITE refusal wording / compliance strategy
Provider surface: SafetyPolicyReferenceSurface
Canonical packet: SafetyPolicyReferencePacket if encoded
Allowed lane: policy design, public safety classification support, customer deployment guidance
Forbidden lane: replacing Selene policy, replacing authority checks, bypassing protected gates
Rule: OpenAI safety guidance can inform Selene policy design, but Selene policy, business rules, authority, identity scope, tenant rules, and protected execution gates remain authoritative.

This guidance may affect:

public abuse prevention
customer chat safety
cybersecurity request handling
minor/under-18 deployment rules
enterprise safety posture
moderation escalation wording

But it must not weaken:

No Simulation → No Execution
protected fail-closed behavior
authority validation
audit requirements
speaker/tenant memory boundaries

8B.11 Coverage hardening enforcement

Every future service-gap review must check both:

8A. OpenAI Service Coverage Classification Layer
8B. OpenAI Coverage Hardening Appendix

If a newly discovered OpenAI surface is not covered, Codex must not silently proceed. It must classify the surface first as one of:

CORE_RUNTIME
VOICE_RUNTIME
WRITE_PRESENTATION
SEARCH_EVIDENCE
MEMORY_RETRIEVAL
ADVISORY_ANALYSIS
EVAL_OPTIMIZATION
COST_SCALE
DISTRIBUTION_SURFACE
ENTERPRISE_ADMIN
FUTURE_OPTIONAL

Then Codex must identify:

service name
canonical Selene owner
provider surface
canonical packet
allowed lane
forbidden lane
provider governance required
protected execution impact
old path retirement condition
whether it is runtime, development-only, distribution-only, admin-only, or future-optional

If classification is unclear, Codex must stop with:

OPENAI_COVERAGE_HARDENING_REQUIRED

8C. Provider Coverage Normalization and Deferred-Service Control

This section is the final consistency layer for the master build plan.

The provider-first architecture is correct, but Codex must not treat Section 6, Section 7, Section 8A, Section 8B, Phase 1 capability examples, and Section 28 build order as separate lists. They are one linked coverage system.

8C.1 Provider coverage normalization rule

Before any implementation from this master plan, Codex must run a provider coverage normalization pass.

Purpose:

Synchronize Section 6 provider interfaces,
Section 7 canonical packet contracts,
Section 8A/8B service coverage,
Phase 1 capability examples,
and Section 28 build order.

Codex must verify that every provider/service surface has:

service name
coverage category
canonical owner
provider interface
canonical packet
capability registry name
allowed lane
forbidden lane
provider governance requirement
protected execution impact
old path retirement condition
implementation status

Implementation status must be one of:

INITIAL_IMPLEMENTATION_ALLOWED
COVERED_FOR_ARCHITECTURE
NOT_INITIAL_IMPLEMENTATION
BUILD_ONLY_AFTER_JD_APPROVAL
DEFERRED_UNTIL_PROVIDER_GOVERNANCE_EXISTS
DEFERRED_UNTIL_CORE_RUNTIME_STABLE
DEFERRED_UNTIL_DISTRIBUTION_STRATEGY_APPROVED

If Codex finds a provider surface in one section but not the others, it must not proceed with implementation until the coverage set is normalized.

Stop code:

PROVIDER_COVERAGE_NORMALIZATION_REQUIRED

8C.2 Deferred / Do Not Build Yet labels

Some OpenAI services must be covered by architecture but not built during the first pivot implementation.

These services must be marked:

COVERED_FOR_ARCHITECTURE
NOT_INITIAL_IMPLEMENTATION
BUILD_ONLY_AFTER_JD_APPROVAL

This applies to:

Sora / video generation
Computer Use
ChatGPT Apps SDK
ChatGPT Connectors / Developer Mode
Shell / Local Shell / Apply Patch
Fine-tuning variants
Agent Builder / ChatKit
Priority/Flex processing
Under-18 deployment guidance
Skills if runtime use is proposed instead of eval/tool-governance use
SIP telephony transport
Provider webhooks that could trigger external actions

Rule:

Covered in the master plan does not mean approved for immediate build.

Codex must not implement these surfaces merely because they appear in the plan.

Each deferred service requires a later explicit JD-approved build instruction with:

exact service surface
why it is needed now
canonical owner
provider interface
packet contract
risk class
allowed lane
forbidden lane
provider governance proof
protected execution proof
old path retirement condition if applicable

8C.3 Initial implementation allowed surfaces

The initial implementation path should focus only on provider infrastructure and the first practical vertical slices.

Initial implementation allowed surfaces:

Provider registry/governance/evidence foundation
Fake provider harness
ModelReasoningProvider
SemanticInterpreterProvider
WritingProvider
SpeechToTextProvider
TextToSpeechProvider
RealtimeVoiceProvider only after STT/TTS/wake proof
SearchProvider
FileSearchProvider
DeepResearchProvider only after PH1.E provider governance proof
EmbeddingProvider
VisionProvider only after file/image evidence policy proof
EvalProvider
ModerationProvider
TokenCountingProvider
PromptCachingPolicy
ProviderAdminTelemetryProvider only for cost/usage telemetry after governance proof

Everything else is covered for architecture but deferred unless JD explicitly approves.

8C.4 Internal consistency acceptance rule

The master plan is not Codex-ready for a specific implementation run until the instruction derived from it proves:

service coverage map matches provider interface inventory
provider interface inventory matches packet inventory
packet inventory matches capability registry
capability registry matches build order
build order marks deferred surfaces clearly
no future-optional surface is accidentally selected as first implementation

If a build instruction skips this consistency proof, Codex must stop with:

PROVIDER_PLAN_INTERNAL_CONSISTENCY_PROOF_REQUIRED

8D. Provider Contract Versioning

Provider-first architecture requires strict versioning. Codex must not change provider interfaces, packet meanings, capability names, or provider model behavior silently.

Every provider surface must define:

ProviderInterfaceVersion
PacketSchemaVersion
ProviderCapabilityVersion
ProviderModelVersion
ProviderPromptVersion where prompts are part of the provider call
ProviderPolicyVersion where governance policy affects behavior

Required versioning rules:

1. Provider interfaces must be additive by default.
2. Packet field removals, renames, semantic changes, or ordering changes require explicit JD approval.
3. Backward-compatible fields must be optional or safely defaulted.
4. Deprecated provider surfaces must remain only with a retirement condition.
5. Provider capability versions must be recorded in provider evidence.
6. Model version and provider route must be recorded in every ProviderCallResultPacket.
7. Migration from one provider version to another must include fake-provider tests and provider-off tests.
8. Old version retirement requires proof that no canonical engine still consumes it.

Stop conditions:

PROVIDER_CONTRACT_VERSIONING_REQUIRED
PACKET_SCHEMA_VERSION_MISSING
PROVIDER_CAPABILITY_VERSION_MISSING
PROVIDER_VERSION_MIGRATION_APPROVAL_REQUIRED

Codex final reports for provider-contract work must include:

old provider interface version
new provider interface version
old packet schema version
new packet schema version
migration compatibility proof
retired fields/surfaces if any
consumers tested
provider-off proof
fake provider proof

8E. Model Governance and Model Pinning

JD-controlled concrete model choices are defined in [Selene OpenAI Model Routing Policy](SELENE_OPENAI_MODEL_ROUTING_POLICY.md). Codex must follow that policy until JD explicitly changes it. This master plan defines model governance; the model routing policy defines the approved OpenAI model IDs and routing choices.

Selene must not call arbitrary OpenAI models by convenience. Model use must be governed, pinned, routed, and auditable.

Required model governance:

model allowlist
model denylist
model version pinning
model fallback order
model capability matrix
model upgrade approval
model rollback rule
per-lane model selection
cheap / normal / high-reasoning route
voice model route
writing model route
search/deep-research model route
vision model route
embedding model route
eval/grader model route

Every model route must declare:

capability
allowed lane
forbidden lane
cost class
latency class
privacy class
reasoning effort if applicable
structured output support yes/no
streaming support yes/no
tool support yes/no
file/image/audio support yes/no
fallback model
provider-off behavior

Rules:

1. A model cannot be selected only because it is newest.
2. A model upgrade is a behavior change if it can affect routing, writing, source use, memory, or protected classification.
3. Model upgrades require targeted regression tests.
4. Protected execution cannot depend on provider confidence alone.
5. Model fallback must not bypass provider governance, privacy, or budget rules.
6. Reasoning effort is a cost/latency/quality setting, not authority.

Stop conditions:

MODEL_GOVERNANCE_REQUIRED
MODEL_NOT_IN_ALLOWLIST
MODEL_FALLBACK_UNGOVERNED
MODEL_UPGRADE_APPROVAL_REQUIRED

8F. Provider Data-Egress and Privacy Boundary

Before any provider call, Selene must decide what data is allowed to leave Selene.

Required data-egress classification:

public data
user-provided text
user-uploaded file content
private user memory
speaker identity evidence
company internal data
tenant/business data
protected HR/payroll/accounting data
customer records
inventory/POS records
financial records
raw audio
raw image/video
connector data
search evidence
memory evidence

Required provider egress decision:

data class
speaker/tenant scope
provider destination
capability requested
minimum necessary input
redaction required yes/no
redaction performed yes/no
retention mode
logging mode
raw data allowed yes/no
hash-only required yes/no
user consent required yes/no
company/admin approval required yes/no
protected execution risk yes/no

Rules:

1. Only minimum necessary context may be sent to a provider.
2. Private memory must not be sent unless the lane, scope, and user context allow it.
3. Protected company data must not be sent unless the provider route is explicitly allowed for that data class.
4. Uploaded files must have file-input permission before provider submission.
5. Raw audio/image/video must have modality-specific egress permission.
6. Provider retention/logging mode must be known or the call must degrade/stop depending on data class.
7. Secrets, API keys, credentials, auth tokens, and private connector credentials must never be sent as model context.
8. Desktop/iPhone must never hold provider secrets.
9. Provider egress evidence must be stored.

Stop conditions:

PROVIDER_DATA_EGRESS_CLASSIFICATION_REQUIRED
PROVIDER_PRIVACY_BOUNDARY_REQUIRED
PROVIDER_REDACTION_REQUIRED
PROVIDER_RETENTION_MODE_UNPROVEN

8G. Prompt Injection and Tool Output Defense

Provider-first architecture makes retrieved content more dangerous. Web pages, files, connector outputs, MCP responses, tool outputs, and user uploads are untrusted by default.

Untrusted input sources:

web search results
web pages
deep research sources
file search chunks
uploaded documents
MCP connector output
email/calendar/document connector content
code interpreter outputs
image OCR text
vision descriptions
tool result text
provider-generated summaries

Rules:

1. Evidence text is data, not instruction.
2. Retrieved content cannot modify Selene system instructions.
3. Retrieved content cannot grant authority.
4. Retrieved content cannot trigger protected execution.
5. Retrieved content cannot override AGENTS law, simulation law, PH1.X, PH1.E, PH1.M, or PH1.WRITE rules.
6. Tool output cannot request a second tool call unless PH1.E validates it independently.
7. File content cannot tell Selene to ignore policies.
8. Web content cannot tell Selene to reveal secrets, call tools, approve actions, or bypass gates.
9. MCP connector output cannot become authority to write business data.
10. Search/source evidence must be stripped or isolated from instruction-like control text before interpretation.

Required defense packets:

PromptInjectionDefensePacket
UntrustedEvidencePacket
InstructionLikeContentDetectedPacket
ToolOutputTrustPacket
EvidenceSanitizationPacket

Protected hard stop:

PROMPT_INJECTION_DEFENSE_REQUIRED
UNTRUSTED_TOOL_OUTPUT_REJECTED
EVIDENCE_INSTRUCTION_OVERRIDE_BLOCKED

This section must be implemented before live File Search, Web Search, Deep Research, MCP connector, or Code Interpreter provider work is considered complete.

8H. Observability, Tracing, and Debug Evidence

Provider-first systems must be traceable. Every provider proposal and every Selene validation/rejection must be visible in backend evidence.

Required trace families:

ProviderTracePacket
ProviderDecisionTracePacket
PH1.X provider-proposal trace
PH1.X candidate validation trace
PH1.X rejection ledger trace
PH1.WRITE validation trace
PH1.E evidence/source trace
PH1.M memory-use trace
PH1.C transcript admission trace
PH1.TTS output trace
PH1.K interruption trace
provider latency trace
provider failure trace
provider-cost trace
streaming trace
provider-off trace
JD live test trace
backend evidence verification trace

Every trace must answer:

what input arrived
which provider was considered
whether provider was enabled
whether provider was called
what budget/counter was used
what provider returned
what Selene packet was created
which canonical engine consumed it
what was accepted
what was rejected
why rejected
what reached response_text
what reached tts_text
what reached Desktop
what was stored
what was deliberately not stored

Rules:

1. No provider-first build is accepted without backend evidence.
2. User-visible behavior must match backend evidence.
3. Provider raw output must not leak into normal UI.
4. Trace/debug data must be bounded and privacy-safe.
5. JD live test evidence must record exact prompt, transcript, visible result, audible result, backend route, and owner decision.

Stop conditions:

PROVIDER_TRACE_MISSING
BACKEND_EVIDENCE_VERIFICATION_FAILED
VISIBLE_BEHAVIOR_BACKEND_TRACE_MISMATCH

8I. Rate Limits, Circuit Breakers, and Provider Health

Provider governance must handle failure safely. Provider outages must not create retry storms, stale answers, or protected-execution bypasses.

Required provider health controls:

rate limit handling
429 handling
timeout handling
provider circuit breaker
provider degraded state
provider cooldown
retry cap
fallback route
no retry storm
per-provider health status
provider outage reporting
provider recovery proof

Rules:

1. Provider calls must have timeout limits.
2. Retry count must be bounded.
3. Rate-limit responses must not cause uncontrolled retries.
4. Provider outage must degrade public/advisory answers where safe.
5. Provider outage must never weaken protected fail-closed behavior.
6. Fallback providers must pass the same egress, budget, privacy, and capability checks.
7. Provider health checks must not call external providers at startup unless explicitly approved.
8. Circuit breaker state must be observable.

Required packets:

ProviderHealthPacket
ProviderCircuitBreakerPacket
ProviderRateLimitPacket
ProviderRetryDecisionPacket
ProviderFallbackDecisionPacket

Stop conditions:

PROVIDER_HEALTH_POLICY_REQUIRED
PROVIDER_RETRY_STORM_RISK
PROVIDER_FALLBACK_UNGOVERNED

8J. Realtime VAD, Turn Boundary, and Live Translation

Selene’s voice problems require explicit realtime turn-boundary governance. Realtime voice is not only audio streaming; it must be governed by Selene-owned admission and session rules.

Required provider surfaces:

RealtimeTranscriptionProvider
VoiceActivityDetectionPolicy
LiveTranslationProvider
RealtimeSessionControlProvider
RealtimeServerSideControlProvider
RealtimeCostPolicy

Required packets:

RealtimeTranscriptionPacket
VoiceActivityDetectionPacket
LiveTranslationPacket
RealtimeSessionControlPacket
RealtimeServerSideControlPacket
RealtimeCostPacket

Rules:

1. VAD may detect speech boundaries, but PH1.C/PH1.L decide transcript admission and turn commit.
2. Live translation is advisory unless explicitly requested or language policy chooses it.
3. Language must reset per turn unless PH1.LANG/PH1.X evidence supports continuity.
4. Realtime server-side controls must not bypass Selene wake, privacy, interruption, or protected gates.
5. Realtime tool calls must become ToolProposalPackets, not direct execution.
6. Realtime cost must be governed by provider budget and session policy.
7. Partial transcripts are not committed turns until Selene admits them.

Live proof must cover:

missed speech
partial transcript
committed transcript
wrong language carryover
barge-in
self-echo
cough/noise
sleep/wake boundary

Stop conditions:

REALTIME_TURN_BOUNDARY_UNGOVERNED
REALTIME_PARTIAL_TRANSCRIPT_COMMITTED_UNLAWFULLY
REALTIME_LANGUAGE_CARRYOVER_REGRESSION

8K. File Inputs Policy

Files API, file input, file search, vector stores, and retrieval are separate concepts. Codex must not collapse them into one path.

Definitions:

Uploaded file = file provided by user or system.
Provider file input = file or file content sent to a provider model.
File lifecycle = upload, retention, deletion, metadata, storage.
File search = retrieval over indexed file content.
Vector store = retrieval index/storage for semantic search.
File evidence = bounded chunk or extracted fact used in an answer.

Required policy:

ProviderFileInputPolicy
ProviderFileInputPacket
file input permission
file input retention
file input redaction
file input scope
file input evidence
file deletion/expiry policy
file search permission
vector-store permission

Rules:

1. File upload does not automatically permit provider submission.
2. File submission does not automatically permit memory storage.
3. File search retrieval does not automatically prove a claim.
4. Full file content must not leak into response_text or tts_text.
5. File chunks must be bounded.
6. Tenant/speaker scope must be enforced.
7. Protected company files require data-egress approval before provider use.
8. File-derived memory requires PH1.M permission.

Stop conditions:

PROVIDER_FILE_INPUT_POLICY_REQUIRED
FILE_PERMISSION_SCOPE_UNPROVEN
FILE_SEARCH_MEMORY_BOUNDARY_UNPROVEN

8L. Streaming Finalization Law

Streaming output is not final output.

Required states:

streaming_started
partial_output_received
partial_output_display_allowed
validation_pending
final_output_accepted
final_output_rejected
stream_cancelled
stream_interrupted
stream_replaced

Rules:

1. Partial stream is not final answer.
2. Partial stream cannot enter memory.
3. Partial stream cannot trigger protected execution.
4. Partial stream cannot be spoken as protected confirmation.
5. Partial stream cannot be audited as final decision.
6. Partial stream cannot be used as source-verified claim until PH1.E/PH1.WRITE validation accepts it.
7. final_output_accepted is required before memory/write/tool/audit finalization.
8. If stream is interrupted, PH1.K/PH1.X must decide whether to resume, abandon, or replace.
9. Desktop must visually distinguish partial/progress output from final accepted answer where applicable.

Required packet:

ProviderStreamFinalizationPacket

Stop conditions:

STREAMING_FINALIZATION_REQUIRED
PARTIAL_STREAM_COMMITTED_UNLAWFULLY
STREAMED_PROTECTED_CONFIRMATION_BLOCKED

8M. Provider-Generated Artifact Governance

Providers may generate artifacts. Generated artifacts need provenance, permission, and status before display, download, memory, or business use.

Artifact sources:

Code Interpreter outputs
image generation outputs
video generation outputs
file transformations
reports
charts
spreadsheets
summaries
rewritten documents
marketing assets
proposal drafts

Required packet:

GeneratedArtifactPacket:
- artifact_id
- artifact_type
- source_input_refs
- provider_evidence_ref
- generation_prompt_hash
- output_hash
- provenance
- advisory_only yes/no
- display_allowed yes/no
- download_allowed yes/no
- memory_allowed yes/no
- retention_policy
- deletion_policy
- approval_status
- official_record_status

Rules:

1. Generated artifact is not automatically an official business record.
2. Generated artifact cannot become official payroll, accounting, legal, HR, inventory, or customer record unless simulation approves.
3. Generated image/video is creative output, not real-world evidence.
4. Generated report is advisory unless deterministic workflow converts it into an official record.
5. Artifact memory storage requires PH1.M or storage policy permission.
6. Desktop renders/downloads only approved artifacts.
7. Artifact provenance must be preserved.

Stop conditions:

GENERATED_ARTIFACT_GOVERNANCE_REQUIRED
GENERATED_ARTIFACT_OFFICIAL_RECORD_BLOCKED
ARTIFACT_PROVENANCE_MISSING

8N. Standard Codex Build Instruction Template

Every Codex build instruction derived from this master plan must use this template or explicitly state why a narrower template is sufficient.

TASK:
<exact task name>

BUILD CLASS:
IMPLEMENTATION unless JD explicitly says docs-only.

REPO ROOT:
/Users/selene/Documents/Selene-OS

LANE DECLARATION:
current project phase:
selected lane:
simulation required: yes/no
authority required: yes/no
state mutation allowed: yes/no
protected execution allowed: yes/no
provider degradation allowed: yes/no
normal answer allowed: yes/no
fail-closed required: yes/no and for which part

CANONICAL OWNER:
primary owner engine:
secondary owner engines:
forbidden owners:

OPENAI / PROVIDER COVERAGE CLASSIFICATION:
service name:
coverage category:
provider interface:
canonical packet:
capability key:
implementation status:
allowed lane:
forbidden lane:
provider governance required:
protected execution impact:
old path retirement condition:

FILE SCOPE:
files expected to change:
why each file must change:
spine/contract touch yes/no:
explicit JD approval required yes/no:

FIRST-READ / REPO TRUTH:
read docs/CORE_ARCHITECTURE.md
read docs/SELENE_BUILD_EXECUTION_ORDER.md
read docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md
read relevant section docs
prove current HEAD
prove origin/main after fresh fetch
prove clean tree

EXISTING OWNER DISCOVERY:
search exact symbols
search related symbols
identify existing owner files
identify current runtime path
identify current tests
classify existing path:
reuse/extend/remove/stop decision:

PROVIDER GOVERNANCE:
provider calls allowed in normal tests yes/no
fake provider required yes/no
live provider opt-in required yes/no
budget/counter proof required yes/no
data-egress classification required yes/no
privacy/redaction required yes/no
prompt-injection defense required yes/no
rate-limit/circuit-breaker proof required yes/no

CONTRACT / VERSIONING:
provider interface version:
packet schema version:
capability version:
model version/pin:
backward compatibility:
retirement condition:

IMPLEMENTATION REQUIREMENTS:
what behavior changes:
what behavior must not change:
old paths to classify:
old paths allowed to remove only after proof:
old paths retained temporarily:

TESTS REQUIRED:
baseline tests before edit:
targeted tests after edit:
exact tests must be verified nonzero:
provider-off tests:
fake-provider tests:
negative/provider-malformed tests:
protected fail-closed tests:
phrase-patch scan:

SMOKE / LIVE PROOF:
voice-first smoke required yes/no:
typed app smoke fallback allowed yes/no and why:
JD live acceptance required yes/no:
backend evidence verification required yes/no:
current Desktop app provenance required yes/no:

FINAL REPORT REQUIRED:
changed files
existing capability reuse proof
correct owner proof
algorithmic generality proof if relevant
provider governance proof
privacy/egress proof
prompt-injection defense proof if relevant
model governance proof if relevant
versioning proof if relevant
real vs mocked/harnessed proof
old paths classified
old paths removed
retained compatibility paths and retirement condition
smoke result
JD live result if applicable
backend evidence refs
commit hash
push result
final clean tree proof
next lawful build

If a Codex instruction omits this template for a provider-first implementation run, Codex must stop with:

STANDARD_PROVIDER_BUILD_TEMPLATE_REQUIRED

8O. Completion Criteria for This Master Build Plan

This master plan is complete only when all hardening layers are present and synchronized:

1. Internal consistency cleanup complete.
2. Provider contract versioning defined.
3. Model governance / model pinning defined.
4. Provider data-egress and privacy boundary defined.
5. Prompt injection / tool output defense defined.
6. Observability / tracing defined.
7. Rate limits / circuit breakers / provider health defined.
8. Realtime VAD / turn boundary / live translation defined.
9. File inputs policy defined.
10. Streaming finalization law defined.
11. Provider-generated artifact governance defined.
12. Standard Codex build instruction template defined.

Codex must not treat this master plan as implementation approval by itself. It is the controlling architecture and instruction framework. Each implementation slice still requires repo-truth discovery, lane declaration, file-scope approval, baseline proof, tests, live smoke where applicable, backend evidence, commit/push, and final clean tree.

9. Phase 0 — Repo Truth, Baseline, and Architecture Freeze

Objective

Before changing runtime behavior, Codex must prove current repo truth and freeze a known-good baseline.

Build 0A — Repo truth and baseline proof

Codex must run a read-only repo-truth audit.

Required proof:

current HEAD
current origin/main
clean tree
current AGENTS.md loaded
mandatory first-read files read
current provider/search/voice/PH1.X/PH1.WRITE/PH1.M/adapter/Desktop owners discovered
existing OpenAI integration surfaces discovered
existing provider gates discovered
current test suites identified
current live app proof requirements listed

Codex must not rely on memory for exact file paths, test names, or hashes.

Build 0B — Known-good baseline tag/record

Codex must create or record a known-good pivot baseline only with JD approval.

Suggested baseline label:

known-good-provider-first-pivot-baseline

Baseline proof must include:

protected fail-closed still works
public chat still works
current voice path status
current TTS path status
current PH1.X status
current PH1.WRITE status
current PH1.M status
current PH1.E/search status
current Desktop app provenance status

If any baseline behavior is broken, Codex must report it before pivot work begins.

Build 0C — Provider Coverage Normalization

Before implementation begins, Codex must normalize the provider coverage set.

Required normalization:

Section 6 provider interfaces are complete.
Section 7 canonical packet contracts are complete.
Section 8A service coverage map is complete.
Section 8B coverage-hardening appendix is complete.
Phase 1 capability examples are complete.
Section 28 build order includes normalization before implementation.
Deferred surfaces are clearly marked.
Initial implementation surfaces are clearly marked.

Codex must produce a coverage table with:

service name
category
provider interface
packet
capability key
implementation status
first allowable phase
JD approval required yes/no

If the table cannot be completed from current repo truth and this master plan, Codex must stop with:

PROVIDER_COVERAGE_NORMALIZATION_REQUIRED

10. Phase 1 — Provider Foundation Inside Existing Runtime

Objective

Add provider abstraction without changing product behavior yet.

Build 1A — Existing provider owner discovery

Codex must search for existing:

provider config
OpenAI clients
Brave/search provider gates
budget counters
model routing
STT routes
TTS routes
Realtime routes
embedding usage
file/vector search usage
vision/image paths
eval/test harnesses

If an existing owner exists, Codex must reuse or extend it.

If a duplicate path is about to be created, stop with:

EXISTING_OWNER_REUSE_REQUIRED

Build 1B — Provider registry and capability map

Add or extend the canonical provider registry.

Provider registry must answer:

which providers exist
which capabilities each provider supports
which provider is enabled
which provider is paid/live/fake/local
which provider can be called in current lane
which provider is allowed for tests
which fallback order applies

Capability examples:

stt
realtime_stt
tts
realtime_voice
semantic_interpretation
writing
web_search
deep_research
file_search
vision
image_generation
embedding
moderation
eval
mcp_connector
code_interpreter
responses_orchestration
responses_websocket
file_lifecycle
vector_store
retrieval
video_generation
computer_use
agents_platform
chatgpt_apps
batch_processing
background_job
context_compaction
fine_tuning
prompt_optimization
grader
usage_telemetry
credential_governance
webhook_receiver
processing_priority
predicted_outputs
token_counting
citation_formatting
realtime_transport
skills
developer_tools
safety_policy_reference
admin_telemetry
provider_contract_versioning
model_governance
model_pinning
data_egress
privacy_boundary
provider_redaction
prompt_injection_defense
provider_trace
observability_trace
provider_health
circuit_breaker
provider_retry_policy
provider_fallback_policy
realtime_transcription
voice_activity_detection
live_translation
realtime_session_control
realtime_server_side_control
realtime_cost
file_input
generated_artifact
artifact_governance
stream_finalization
standard_provider_build_template
coverage_normalization
deferred_service_control

The capability registry must stay synchronized with the provider interface inventory and canonical packet inventory. Codex must not add a new OpenAI/provider capability in one place while leaving the others stale.

Build 1C — Provider evidence envelope

Every provider call must produce evidence:

provider_id
capability
lane
request_id
session_id when available
actor/user when available
model id
input hash
output hash
start/end time
latency
budget packet ref
call counter
network dispatch counter
failure reason if any
redaction status
raw output retained? yes/no and why

Provider evidence must never become authority for protected execution.

Build 1D — Fake provider test framework

Before live OpenAI proof, Codex must prove the provider layer with fake providers.

Fake provider must support:

successful structured interpretation
provider disabled
provider timeout
provider malformed output
provider unsupported capability
provider budget exceeded
provider degraded answer

Normal tests must not call live OpenAI.

11. Phase 2 — Wake, STT, and TTS Provider Spine

Objective

Move voice input/output toward provider-backed architecture while keeping wake/session authority in Selene.

Canonical owners

PH1.W owns wake acceptance.
PH1.C owns transcript admission and STT evidence.
PH1.TTS owns approved TTS output evidence.
PH1.L owns session/sleep boundary.
PH1.K owns interruption policy.
Adapter transports.
Desktop captures, plays, and renders only.

Build 2A — Wake boundary cleanup

Requirements:

wake word is not committed as user prompt
pre-wake audio is privacy bounded
OpenAI STT is not called before lawful wake unless explicitly approved by wake architecture
one canonical wake path
no stale wake/listening loops
no Desktop semantic wake authority

Old wake surfaces must be classified:

CURRENT_ACTIVE_REQUIRED
MIGRATE_TO_CANONICAL_OWNER
DEAD_UNREACHABLE
STALE_DANGEROUS
LEGACY_COMPATIBILITY_REQUIRED
REPO_TRUTH_CONFLICT

Build 2B — SpeechToTextProvider vertical slice

OpenAI may transcribe audio. Selene owns the transcript packet.

Required packet:

TranscriptPacket:
- transcript_text
- normalized_text
- language
- confidence when available
- audio_ref
- speaker_evidence_ref when available
- provider_evidence_ref
- noise_gate_result
- self_echo_result
- commit_allowed
- rejection_reason

PH1.C must decide whether transcript commits to runtime.

Build 2C — TextToSpeechProvider vertical slice

OpenAI may generate audio. Selene owns approved spoken text and playback evidence.

Required packet:

VoiceOutputPacket:
- approved_tts_text
- approved_tts_text_hash
- provider_audio_ref
- provider_evidence_ref
- voice_id / voice_style if allowed
- playback_start
- playback_end
- playback_failure_reason
- spoken_text_hash

Desktop may only play approved audio and report playback evidence.

Build 2D — Re-arm and session proof

Live proof must show:

wake accepted
question captured
answer produced
OpenAI/Selene TTS spoken if enabled
TTS completion observed
listening re-arms
sleep boundary works
wake after sleep works
no duplicate app
no duplicate adapter
backend evidence matches UI

Old path retirement after proof

Remove or retire:

obsolete Apple STT remnants
obsolete Apple/native TTS fallback if replaced and approved
stale duplicate listening loops
stale transcript commit routes
Desktop semantic wake shortcuts
old TTS status paths
adapter voice shortcuts

Only remove after proof and within approved file scope.

12. Phase 3 — Realtime Voice / Duplex / Barge-In Provider

Objective

Use a provider-backed realtime voice substrate without giving the provider authority over Selene conversation state or protected execution.

Build 3A — RealtimeVoiceProvider contract

The provider may emit events such as:

speech_started
speech_stopped
transcript_delta
transcript_final
model_audio_started
model_audio_stopped
response_cancelled
audio_truncated
interruption_detected
tool_call_requested
session_error

Selene must map these to:

RealtimeVoiceEventPacket
InterruptionPacket
TranscriptPacket
VoiceOutputPacket
ProviderCallResultPacket

Build 3B — Selene interruption control

Selene decides:

is this real user speech?
is this cough/noise?
is this self-echo?
what answer was interrupted?
did the user correct the answer?
does “make it shorter” refer to current output?
does “no, I meant Sydney” correct a prior slot?
is the new speech protected?
should TTS cancel?
should the old answer resume or be abandoned?

Canonical owners:

PH1.K interruption
PH1.C audio validation
PH1.X meaning/reference resolution
PH1.TTS output authority
PH1.L session boundary
Storage evidence

Live tests

Selene gives long answer → JD interrupts: stop, make it shorter
Selene speaks → JD coughs
Selene speaks → JD says: no, I meant Sydney
Selene speaks → JD says: approve payroll for Tim
Selene speaks → typed interruption arrives

Pass only if:

visible behavior correct
TTS control correct
backend evidence correct
protected request fails closed
old working voice paths still work
no Desktop semantic authority added

13. Phase 4 — OpenAI-Assisted Current Turn Understanding Inside PH1.X

Objective

Use OpenAI structured interpretation as a proposal layer, while PH1.X remains the deterministic validator/router.

Build 4A — SemanticInterpreterProvider

Input:

current user turn
recent assistant answer summary
active frame
topic stack
writing artifact refs
tool continuation refs
clarification target
correction target
speaker posture
memory evidence refs allowed for this turn
protected-risk hints
language packet
provider budget packet

Output:

CurrentTurnInterpretationPacket:
- raw_user_intent_summary
- interaction_posture
- requested_operation
- candidate_targets
- reference_target
- entities
- slots
- temporal_refs
- language
- protected_risk
- likely_owner
- confidence
- ambiguity
- clarification_needed
- clarification_question_candidate
- prohibited_or_uncertain_fields
- provider_evidence_ref

OpenAI proposes this packet. PH1.X does not blindly trust it.

Build 4B — PH1.X deterministic validation layer

PH1.X must perform:

candidate generation
candidate scoring
hard disqualifiers
rejection ledger
owner validation
fresh/stale context check
protected-risk validation
clarification target tracking
correction target tracking
writing artifact target validation
tool continuation validation
memory handoff validation
HumanConversationDirective creation

PH1.X output:

HumanConversationDirective:
- owner_engine
- allowed_actions
- blocked_actions
- selected_candidate_ref
- rejected_candidate_refs
- confidence
- ambiguity
- reason_code
- next_action
- protected_fail_closed_status when applicable

Build 4C — Negative control tests

PH1.X must reject bad provider proposals:

provider says salary change is public chat → reject
provider chooses stale topic → reject
provider invents memory → reject
provider chooses wrong artifact → reject
provider gives high confidence but missing evidence → reject
provider routes to Desktop → reject
provider routes to Adapter semantic shortcut → reject

Build 4D — Retire old phrase-patch paths

Only after PH1.X provider-assisted validator passes live proof, remove:

deterministic_active_context shortcuts
deterministic_weather_context shortcuts
adapter semantic shortcuts
exact phrase continuation patches
time/weather one-off logic outside owner
writing one-off phrase patches

Required phrase-patch scan must classify every hit.

Live tests

Japan skiing/restaurants → which city / which area / where would you base the trip
restaurant answer → make it shorter / one line
message to Mark → make it warmer
New York time → Sydney
New York time → what is your name
weather Sydney → not weather, time
organize payroll for Tim → protected fail-closed
approve payroll for Tim → protected fail-closed

Pass requires JD live behavior and backend evidence agreement.

14. Phase 5 — OpenAI-Assisted PH1.WRITE Inside Canonical PH1.WRITE

Objective

Use OpenAI writing quality while PH1.WRITE owns final answer policy, structure, evidence discipline, memory wording, source chips, TTS-safe output, and protected fail-closed wording.

Build 5A — WritingProvider

WritingProvider may produce:

headers
paragraphs
bullets
tables
short answer
long answer
one-line answer
warmer tone
professional tone
legal-friendly summary
sales copy
SOP draft
CRM message
email draft
meeting summary
proposal draft
translation
multilingual business communication

Build 5B — PH1.WRITE control packet

Input:

WriteRequestPacket:
- directive_ref
- requested_operation
- target_artifact_ref
- source_evidence_refs
- memory_evidence_refs
- tool_evidence_refs
- audience
- tone
- length
- language
- formatting requirements
- protected wording requirements
- source chip refs
- tts constraints

Output:

WriteOutputPacket:
- title
- summary
- sections
- paragraphs
- bullets
- tables
- warnings
- display_text
- tts_text
- source_chip_refs
- image_card_refs
- memory_style
- unsupported_claims_removed
- provider_evidence_ref

Build 5C — PH1.WRITE validator

PH1.WRITE must validate:

no unsupported factual claims
no invented source/citation
no invented memory
no source dump
no raw provider JSON
no protected execution claim
same-language where appropriate
TTS-safe version available
Desktop formatting not required for meaning

Build 5D — Retire old formatter paths

Remove only after proof:

Desktop formatting brain
Adapter formatting shortcut
plain robotic formatter where superseded
source dump wording
archive/session wording
TTS unsafe long text path

Live tests

make it shorter
give me one line
make it warmer
write with headers
put it in bullets
explain briefly
write beautiful detailed answer
search answer with source chips
remembered topic answer
protected fail-closed answer

15. Phase 6 — Search, Deep Research, File Search, and Evidence Inside PH1.E

Objective

Use OpenAI search/research/file retrieval as providers, while PH1.E owns search planning, tool policy, source acceptance, claim verification, presentation metadata, and provider cost governance.

Build 6A — SearchProvider

Provider may perform:

quick public search
source retrieval
source summarisation
query expansion suggestion

PH1.E owns:

search needed?
query plan
provider selection
budget
accepted/rejected source separation
claim-to-source verification
source chips
safe degradation
no raw dumps

Build 6B — DeepResearchProvider

Deep research must be explicitly gated.

Allowed lane:

public/advisory research

Not allowed:

protected execution
business state mutation
official company filing/posting/approval

Deep research requires:

explicit user intent or approved escalation
background/status handling
budget cap
provider evidence
source evidence
final synthesis through PH1.WRITE

Build 6C — FileSearchProvider

File search can retrieve from uploaded/company/user documents, but Selene owns permission and evidence.

Required:

file permission check
tenant/speaker scope
retrieved chunk refs
source file refs
chunk evidence
claim verification where factual claims are made
no raw full-document dump

Build 6D — Old search path cleanup

Remove or retire after proof:

raw source dumps
uncontrolled provider routes
duplicate provider callers
provider shortcuts without budget/counters
PH1.E stale context logic
source chip bypasses
wrong-source acceptance paths

Live tests

What time is it in Sydney?
What about Melbourne?
Search public news about a synthetic topic.
Show sources.
Provider-off test.
Wrong-source rejection.
File Q&A over uploaded document.
Deep research request with explicit cap.
Protected mixed request with search + payroll action.

16. Phase 7 — Vision, Image Evidence, and Image Generation

Objective

Use OpenAI vision/image capabilities as providers while Selene owns visual evidence, file privacy, image display approval, and memory permission.

Build 7A — VisionProvider

OpenAI may analyze images.

Selene must convert result into:

VisualEvidencePacket:
- image_ref
- extracted_text
- visual_summary
- detected_objects
- confidence/uncertainty
- limitations
- provider_evidence_ref
- source/provenance ref
- safe_for_memory yes/no
- safe_for_display yes/no

Selene must not treat vision output as guaranteed truth.

Build 7B — Image evidence rules

Vision output may support:

receipt parsing draft
onboarding doc extraction draft
product recognition draft
visual context summary
image-based writing prompt

Vision output must not directly execute:

identity approval
payroll
legal decisions
medical interpretation
protected business mutation

Build 7C — ImageGenerationProvider

OpenAI may generate/edit images for:

marketing assets
menus
wine descriptions visual drafts
posters
business onboarding visuals
social media drafts

Selene must own:

prompt policy
brand constraints
approval status
asset provenance
display/download metadata
no fake real-world evidence

Build 7D — Desktop rendering

Desktop renders approved image cards only.

Desktop must not:

choose images
rank images
call providers
invent captions
turn images into factual proof

17. Phase 8 — Embeddings and PH1.M Human Memory

Objective

Use embeddings and summarisation as support tools, while PH1.M remains the governed human memory brain.

Build 8A — EmbeddingProvider

OpenAI may generate vectors.

PH1.M owns:

memory object creation
memory permission
speaker scope
tenant scope
privacy gate
freshness
trust
conflict/staleness
forget/update
retrieval ranking policy

Embedding packet:

EmbeddingPacket:
- text_hash
- vector_ref
- model/provider evidence
- scope
- created_at
- allowed_use

Build 8B — Memory encoding assistance

OpenAI may suggest:

summary
salience
possible topic label
possible open task
possible preference
possible decision
possible correction

PH1.M decides:

whether to remember
what to remember
who it belongs to
privacy class
fresh/topic/deep tier
how to update/forget later

Build 8C — Topic graph

OpenAI can help label and cluster topics. PH1.M owns the graph.

Required topic graph entities:

project
decision
preference
open task
person/company relation
document/artifact
conversation thread
staleness/conflict marker

Build 8D — Deep recall

Deep recall may use:

embeddings
file search
memory graph
summarisation provider

But final recall must be:

PH1.M evidence-backed
PH1.WRITE natural language
speaker/tenant scoped
privacy-gated
not archive/session wording

Live tests

What did we decide earlier?
Continue the Japan trip.
What did we say yesterday about PH1.X?
That was old, use the newer plan.
Forget that.
What do you remember about Desktop?
Unknown speaker asks private-memory follow-up.

18. Phase 9 — Voice ID and Speaker Scope

Objective

Keep speaker identity Selene-owned. OpenAI voice/STT may help with audio handling, but Voice ID is not outsourced as business authority.

Build 9A — Speaker evidence packets

Every turn should carry:

speaker_id if known
known/unknown/guest posture
voice confidence
speaker_changed
same_speaker_as_previous
typed actor identity separate from voice identity
voice evidence ref
memory scope
protected authority scope

Build 9B — Speaker-scoped memory and permissions

PH1.M must distinguish:

JD private memory
guest memory
shared conversation memory
company/workspace memory
public-only guest lane

Voice ID is evidence only.

It must never equal:

authority approval
simulation approval
payment approval
protected execution permission

Live tests

JD speaks.
Unknown speaker asks normal public question.
Unknown speaker asks private-memory question.
Recognized speaker continues fresh topic.
Recognized voice attempts protected action.
Typed actor attempts voice-only path.

19. Phase 10 — Tool Calling, Tool Search, MCP, and Connectors

Objective

Use OpenAI tool/function/MCP capability to propose tool use, not to execute protected actions.

Build 10A — ToolProposalProvider

OpenAI may produce:

ToolProposalPacket:
- proposed_tool_family
- proposed_tool_name
- proposed_arguments
- confidence
- reason
- provider_evidence_ref

Selene must decide:

is tool allowed?
is it read-only public?
is it company/private data?
is it protected execution?
is simulation required?
is authority required?
should arguments be corrected/rejected?

Build 10B — ToolSearchProvider

Tool search may help with large tool catalogs.

Selene owns:

tool registry truth
tool permission
tool argument schema
tool execution decision
simulation catalog boundary

Build 10C — MCP connector boundary

MCP/connectors may retrieve or propose interactions with:

Google Workspace
Dropbox/Drive
NetSuite
CRM
POS
inventory
email/calendar/docs
custom Selene business gateway

Rules:

read-only retrieval may be public/advisory or permission-gated depending on data
protected writes require simulation + authority
MCP must not bypass PH1.E / Access / SimulationExecutor
connector evidence must be logged
secrets must stay server-side
Desktop must never hold connector secrets

Live tests

Read-only connector lookup.
Draft an email from retrieved info.
Attempt to send email → protected/external action gate.
Search company document.
Attempt inventory update → simulation fail-closed.

20. Phase 11 — Code Interpreter / Data Analysis Advisory Lane

Objective

Add sandboxed analysis as a product capability without violating Codex repo law or protected execution law.

Important distinction:

Codex repository work: Python is forbidden by AGENTS law.
Selene product advisory analysis: sandboxed provider analysis may be allowed if governed.

Build 11A — CodeInterpreterProvider

Allowed for:

spreadsheet analysis
chart generation
advisory P&L draft
file transformation
business report drafts
data cleanup suggestions
forecast drafts

Not allowed for:

repo editing by Codex
protected official accounting execution
posting invoices
payroll approval
database mutation
bank/payment operations
authority-gated company writes

Build 11B — Advisory artifact packet

Output:

AnalysisArtifactPacket:
- input_file_refs
- generated_file_refs
- calculations_summary
- assumptions
- limitations
- advisory_only flag
- provider_evidence_ref
- PH1.WRITE summary

Live tests

Analyze uploaded numbers and draft summary.
Generate advisory chart.
Explain assumptions.
Try to submit official report → protected fail-closed.

21. Phase 12 — Evals, Prompt Optimization, and Regression Harness

Objective

Move Selene away from manual-only testing by adding provider-assisted evals while preserving JD live acceptance as final product proof.

Build 12A — EvalProvider

OpenAI may help judge:

PH1.X interpretation quality
PH1.WRITE output quality
speech transcript repair quality
search answer grounding
source-chip correctness
memory answer naturalness
protected classification safety

Selene owns:

eval corpus
golden expectations
pass/fail gates
backend evidence checks
JD live acceptance result
regression tracking

Build 12B — Eval corpus

Must include:

bad English
accented/misheard transcripts
Chinglish/mixed language
short follow-ups
artifact editing
writing style changes
protected requests
mixed requests
memory recall
speaker changes
search/source answers
file prompts
image prompts
barge-in prompts

Build 12C — Grader and prompt optimizer boundary

OpenAI grader/prompt optimizer may improve prompts and scoring, but must not become runtime authority.

No eval result alone can approve protected execution.

Required reports

Every implementation final report must still distinguish:

CODEX_TESTED
REAL_APP_SMOKE_PASSED
JD_LIVE_ACCEPTANCE_PASSED
PENDING_JD_LIVE_ACCEPTANCE
JD_LIVE_ACCEPTANCE_FAILED
BACKEND_EVIDENCE_VERIFICATION_FAILED

22. Phase 13 — Moderation and Safety Signals

Objective

Use moderation/safety models as advisory signals while Selene owns final policy and lane decisions.

Build 13A — ModerationProvider

Use for:

public-facing abuse prevention
customer chat safety
image/text content screening
unsafe request detection
support workflow safety

Selene owns:

policy mapping
business policy
protected execution gate
escalation decision
audit record
final refusal/redirect wording through PH1.WRITE

Moderation output is a signal, not execution authority.

23. Phase 14 — Provider Swap Certification

Objective

Prove Selene can replace OpenAI later without rebuilding engines.

Build 14A — Fake provider parity suite

For every provider interface, fake providers must prove:

success
failure
timeout
malformed output
provider disabled
budget exceeded
unsupported capability
partial degraded response

Build 14B — Second-provider simulation

Even before a real second provider exists, Codex must prove engines depend only on canonical packets.

Required scan:

PH1.X does not import OpenAI types
PH1.M does not import OpenAI types
PH1.WRITE does not import OpenAI types
PH1.E does not import OpenAI types except through provider abstraction if repo owner says so
Desktop does not import OpenAI provider types
Adapter does not become provider brain

Build 14C — Provider-off product behavior

With OpenAI disabled:

public chat safe-degrades
protected execution still fails closed correctly
no startup provider probes
no provider call attempts
no network dispatches
voice degraded state shown accurately
search degraded state shown accurately

24. Phase 15 — Old Path Retirement Program

Objective

Remove old rubbish only after the new provider-backed canonical path is proven.

Retirement process for every old path

For each old path, Codex must classify:

CURRENT_ACTIVE_REQUIRED
RETAINED_COMPATIBILITY_PATH with retirement condition
MIGRATE_TO_CANONICAL_OWNER
DEAD_UNREACHABLE
STALE_DANGEROUS
WRONG_OWNER_SURFACE
REPO_TRUTH_CONFLICT

Then act:

CURRENT_ACTIVE_REQUIRED → keep
RETAINED_COMPATIBILITY_PATH → keep with removal condition
MIGRATE_TO_CANONICAL_OWNER → move only with approval
DEAD_UNREACHABLE → remove inside approved scope
STALE_DANGEROUS → remove/block/escalate
WRONG_OWNER_SURFACE → move/remove/escalate
REPO_TRUTH_CONFLICT → stop for JD decision

Domains to clean

wake/listening loops
STT/TTS remnants
adapter semantic shortcuts
PH1.X phrase patches
weather/time special cases
Desktop formatting brain
Desktop identity/meaning shortcuts
adapter memory shortcuts
old PH1.M recall routes
raw source dump paths
uncontrolled provider routes
old image/source-card paths
obsolete tests/fixtures
stale status strings

Required proof before deletion

new canonical path proven
old accepted behavior re-proven
targeted tests pass
live smoke passes where user-visible
backend evidence matches visible behavior
phrase-patch scan clean or classified
legacy cleanup proof included
final tree clean

25. Phase 16 — Protected Business Simulation Workflows

Objective

Once the provider-assisted public/conversation/writing/search/memory spine is stable, protected business workflows can be built through deterministic simulations.

OpenAI role

OpenAI may assist with:

intent interpretation proposal
clarification wording
document summarisation
advisory analysis
draft reports
tool proposal

OpenAI must not:

approve payroll
increase salary
submit leave approval
write official database state
change customer records
execute inventory updates
send official external messages without approved workflow
become authority
bypass simulation

Selene protected path

PH1.X protected-action classification
Access/Governance
Simulation catalog lookup
SimulationExecutor
confirmation
authority validation
deterministic process execution
audit ledger
PH1.WRITE final wording
Desktop render only

First protected simulation candidates

payroll preparation simulation
payroll approval fail-closed
leave request
rostering update
salary change
inventory update
customer record update
invoice posting
external message sending

Live tests

Tell me about payroll. → public/advisory answer
Prepare payroll. → protected/simulation gate
Approve payroll for Tim. → fail closed unless simulation + authority
Search salary trends and increase Tim’s salary. → mixed split
Yes, do it. → only executes if prior protected confirmation and authority exist

26. Phase 17 — Desktop and iPhone Boundary Preservation

Objective

Keep clients thin while making them capable of rendering richer provider-assisted outputs.

Desktop/iPhone may own:

microphone capture
push-to-talk / wake UI capture
TTS playback
playback completion evidence
basic preview state
transport to adapter
rendering accepted runtime output
source chips
image cards
file cards
memory UI panels
status display

Desktop/iPhone must not own:

semantic intent
slot filling
PH1.X routing
PH1.M memory decisions
PH1.E search/tool decisions
provider calls
provider secrets
protected execution
authority decisions
Voice ID final identity
wake acceptance final authority

Required Desktop proof

Before any JD live smoke:

current HEAD proven
fresh app built
stale app instances closed
one Desktop app active
one adapter/runtime owner active
bundle path recorded
health/provenance endpoint checked
latest app used

No Desktop proof counts without current-app provenance.

27. First Vertical Slice Recommendation

Do not start with the whole system at once.

Start with one narrow provider-first vertical slice:

User: “Can you give me one line?”

Expected flow:

User turn
→ PH1.C transcript/typed admission
→ SemanticInterpreterProvider proposes:
      requested_operation = one_line
      target = previous_answer
      likely_owner = PH1.WRITE
→ PH1.X validates target and rejects stale/wrong targets
→ PH1.X emits HumanConversationDirective
→ PH1.WRITE sends bounded WriteRequestPacket to WritingProvider
→ WritingProvider returns WriteOutputPacket
→ PH1.WRITE validates output
→ Adapter transports
→ Desktop renders
→ PH1.TTS speaks tts_text if voice mode
→ Storage files evidence

First slice must prove:

provider abstraction works
PH1.X still decides
PH1.WRITE still owns presentation
Desktop does not format/decide
Adapter does not become brain
old answer target is correct
backend evidence agrees with UI
provider-off degradation works

Then expand to:

make it warmer
where would you base the trip
what about Sydney
payroll Tim organize
search with sources
image/file prompt
voice wake → question → answer → re-arm
barge-in correction
memory recall

28. Recommended Build Order

0. Repo truth and known-good baseline.
1. Provider coverage normalization across service map, provider interfaces, packet inventory, capability registry, deferred surfaces, and build order.
2. Provider contract versioning, model governance, data-egress/privacy boundary, prompt-injection defense, observability/tracing, provider health/circuit breaker, file-input policy, streaming finalization, artifact governance, and standard Codex template confirmation.
3. Provider registry/governance/evidence foundation.
4. Fake provider test harness.
5. SpeechToTextProvider + TextToSpeechProvider inside current voice spine.
6. Wake/re-arm/sleep recertification.
7. SemanticInterpreterProvider vertical slice.
8. PH1.X deterministic validator around provider interpretation.
9. PH1.WRITE provider bridge and output validator.
10. Retire proven-obsolete PH1.X/adapter/Desktop phrase/format shortcuts.
11. SearchProvider / FileSearchProvider / DeepResearchProvider inside PH1.E.
12. VisionProvider / ImageGenerationProvider inside visual evidence path.
13. EmbeddingProvider and PH1.M memory encoding/retrieval support.
14. RealtimeVoiceProvider and PH1.K interruption/barge-in.
15. ToolProposalProvider / ToolSearchProvider / MCP connector boundary.
16. CodeInterpreterProvider for advisory analysis only.
17. EvalProvider and regression corpus.
18. ModerationProvider safety signals.
19. Provider swap certification.
20. Old path retirement program by domain.
21. Protected business simulations.
22. Desktop/iPhone rich rendering final pass.

The first implementation build after repo truth must not skip provider coverage normalization. Codex must prove the normalized coverage set before adding provider code.

This order keeps risk controlled: provider foundation first, one vertical slice, then engine-by-engine rehabilitation.

29. Acceptance Matrix

No build from this plan is accepted unless it proves the relevant acceptance class.

Provider foundation acceptance

provider calls gated
provider disabled means zero attempts/dispatches
fake provider tests pass
budget/counter evidence recorded
raw provider output not leaked
engines consume canonical packets only

Voice acceptance

wake lawful
transcript accurate enough or rejected
TTS approved text only
TTS completion evidence
re-arm works
sleep/wake works
JD live voice passes when available

PH1.X acceptance

provider proposal captured
PH1.X candidate validation proven
rejection ledger present
unseen paraphrases pass
negative hijack tests pass
protected fail-closed preserved
old phrase shortcuts removed or justified

PH1.WRITE acceptance

beautiful structured output
same-language support
source/memory/protected wording rules obeyed
TTS-safe text produced
no invented facts/memory/sources
Desktop does not rewrite meaning

PH1.E acceptance

provider route gated
accepted/rejected sources separated
claim verification preserved
source chips safe
provider-off safe degrade
no raw dumps
protected mixed requests split

PH1.M acceptance

memory permission controlled by PH1.M
speaker scope enforced
fresh/topic/deep recall works
staleness/conflict handled
forget/update works
no session-search wording
no adapter/Desktop memory brain

Protected execution acceptance

simulation required
authority required
audit required
confirmation required
OpenAI proposal cannot execute
mixed requests split correctly
No Simulation → No Execution preserved

30. Final Completion Report Required From Codex

Every implementation build derived from this plan must report:

Task name
Lane declaration
OpenAI/provider coverage classification proof
Coverage normalization proof
Standard template compliance proof
Deferred-service check
Current HEAD
Origin/main posture from fresh fetch
Clean tree start/end
First-read files read
Existing capability reuse proof
Correct owner map
Files changed and why
Baseline tests before edit
Targeted tests after edit
Exact tests verified nonzero
Provider calls real/fake/off
Provider counters/budget proof
Provider contract versioning proof
Model governance / model pinning proof
Data-egress/privacy proof
Provider redaction proof where applicable
Prompt-injection/tool-output defense proof where applicable
Provider trace / observability proof
Provider health / rate-limit / circuit-breaker proof
Provider fallback / retry policy proof
File-input policy proof where files are involved
Streaming finalization proof where streaming is involved
Artifact governance proof where artifacts are produced
Realtime VAD / turn-boundary proof where realtime voice is involved
What was real vs mocked/harnessed
Live smoke path used
JD live result if applicable
Backend evidence refs
Old paths classified
Old paths removed
Retained compatibility paths and retirement condition
Phrase-patch scan result
Desktop authority proof if Desktop touched
Adapter no-brain proof if Adapter touched
Protected fail-closed proof
Git diff stat
Commit hash if committed
Push result
Final clean tree proof
What remains unproven
Next lawful build

31. Final Architecture Outcome

At the end of this pivot, Selene should have:

one provider-governed OpenAI integration layer
one clean PH1.X conversation/context/routing validator
one clean PH1.WRITE writing and presentation owner
one clean PH1.C transcript admission owner
one clean PH1.TTS output owner
one clean PH1.W wake owner
one clean PH1.K interruption owner
one clean PH1.M human memory brain
one clean PH1.E tool/search/evidence owner
one clean Voice ID speaker evidence path
one clean Adapter transport bridge
one clean Desktop/iPhone renderer shell
one clean protected simulation execution path

OpenAI gives Selene:

ChatGPT-like language
high-quality voice/STT/TTS
structured interpretation proposals
beautiful writing
search/research assistance
file and image understanding
embeddings
evals

Selene gives OpenAI-powered intelligence the enterprise layer it does not provide by itself:

deterministic orchestration
simulation-first execution
authority boundaries
identity scope
memory governance
evidence discipline
auditability
protected fail-closed behavior
company-safe workflows
provider replaceability

Final rule:

Build provider-first.
Stay inside existing canonical engines.
Remove old paths only after proof.
Never create parallel brains.
Never patch phrases manually.
Keep Selene in control.
