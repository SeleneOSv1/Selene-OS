Selene Function Stack Architecture — Intent and Enterprise Stack Map

DOCUMENT TYPE:
GLOBAL STANDARD MASTER ARCHITECTURE / CODEX REPO-TRUTH PREPARATION SCRIPT

TASK:
SELENE_FUNCTION_STACK_ARCHITECTURE_INTENT_AND_STACK_MAP

BUILD CLASS:
ARCHITECTURE / FUNCTION STACK MASTER MAP / CODEX REPO-TRUTH INPUT

CONTROLLING DOCUMENTS:
1. AGENTS.md
2. Selene Provider-First OpenAI Assisted Pivot Master Build Plan
3. Selene Provider-First Function Architecture Cards
4. Selene Provider-First Vertical Slice Build Pack
5. Selene Global Human Conversation Spine Master Architecture
6. Selene Identity + Access + Authority Spine Master Architecture
7. docs/CORE_ARCHITECTURE.md
8. docs/SELENE_BUILD_EXECUTION_ORDER.md
9. docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md

PURPOSE:
Define the first complete intent and function-stack architecture map for Selene so Codex can perform repo-truth discovery and then design the first overall architecture + build plan that connects OpenAI capabilities to Selene-owned enterprise-grade stacks without turning OpenAI, Desktop, Adapter, or any helper path into runtime authority.

0. Master Intent

Selene is not a chatbot wrapper.

Selene is an enterprise-grade human interface and runtime orchestration system that uses OpenAI as a probabilistic intelligence layer behind Selene-owned provider interfaces, validation layers, function stacks, presentation layers, access gates, and audit evidence.

The master intent is:

Every user interaction should flow through a global human interface.
GPT-5.5 or approved OpenAI models help Selene understand what the user wants.
Selene validates the request against identity, access, memory scope, evidence rules, provider governance, protected-risk rules, and canonical owner routing.
Each major capability is handled by its own enterprise function stack.
OpenAI may assist inside a stack.
Selene owns the stack, decisions, validation, presentation, and audit.

This is the key architectural leap:

OpenAI provides probabilistic capability.
Selene provides enterprise-grade function stacks.

Each function stack must be designed as:

intent → routing → provider role → Selene validation → evidence → presentation → audit → tests → JD live proof where visible

1. Core Distinction

There are two different problems.

Problem A — Understanding what the user wants

This is the semantic problem.

Owner model:

GPT-5.5 / SemanticInterpreterProvider proposes.
PH1.X validates.

Questions:

What is the user asking?
Is this a rewrite, search, memory recall, tool request, file request, image request, protected action, or mixed request?
What target is the user referring to?
What owner should handle it?
Is clarification needed?

Problem B — Executing the right enterprise function safely and beautifully

This is the stack problem.

Owner model:

Each canonical Selene stack owns its own decisions, evidence, validation, presentation, and audit.

Questions:

Which stack owns this function?
Which OpenAI capability may assist?
Which Selene rules validate the result?
Which evidence is required?
Which format should be shown?
Which TTS output should be spoken?
Which tests prove it?
Which backend packets prove it?

This document defines Problem B.

2. Global Stack Pattern

Every Selene function stack must follow the same enterprise pattern.

1. Intent Contract
2. Admission and Scope Gate
3. Owner Routing
4. Provider Capability Selection
5. Provider Governance Preflight
6. Stack-Specific Planning
7. Execution / Retrieval / Generation
8. Stack-Specific Validation
9. Evidence Packet Creation
10. Claim / Safety / Permission Verification
11. Presentation Planning
12. PH1.WRITE Final Output
13. TTS-Safe Output
14. UI Metadata / Source / Image / Artifact Packet
15. Audit / Trace / Cost Evidence
16. Provider-Off Behavior
17. Fake-Provider Behavior
18. Tests / Evals / JD Live Acceptance
19. Old Path Cleanup

No stack may skip:

canonical owner discovery
provider governance
identity/access scope where applicable
protected-risk classification
backend evidence
presentation validation
JD live acceptance where user-visible

3. Global OpenAI Capability Surfaces

Codex must treat these as capability surfaces, not authority surfaces.

OpenAI may provide:

semantic interpretation
structured outputs
function/tool-call style proposals
reasoning assistance
text generation
code generation
writing transformation
summarization
translation
language detection
web search assistance
file search / retrieval assistance
image understanding
image generation
video generation where approved
speech-to-text
text-to-speech
realtime voice interaction
embeddings
conversation state support
context compaction support
deep research assistance
tool planning
connector/MCP proposal support
evals / graders / optimization assistance

OpenAI must not own:

identity
access
authority
protected execution
source acceptance
memory permission
tool permission
file permission
billing policy
provider budget authority
Desktop behavior
Adapter behavior
final audit truth
official business mutation

The rule:

OpenAI proposes, drafts, extracts, classifies, summarizes, translates, generates, or reasons.
Selene validates, permits, routes, accepts, rejects, formats, executes only when lawful, and audits.

4. Function Stack Inventory

Selene must define enterprise stacks for at least these capabilities:

A. Global Human Interface / Semantic Intent Stack
B. Web Search + Source Evidence Stack
C. Image-Backed Search + Visual Presentation Stack
D. Deep Research Stack
E. Writing + Transformation Stack
F. Presentation + TTS-Safe Output Stack
G. Memory + Recall + Preference Stack
H. File Question Answering + Document Stack
I. Tool / Connector / MCP Stack
J. Protected Action + Simulation Stack
K. Identity + Access + Authority Stack
L. Voice / Wake / Session / Realtime Stack
M. Translation + Language Adaptation Stack
N. Summarization + Compression Stack
O. Artifact / Document / Slide / Spreadsheet Stack
P. Image Generation / Editing Stack
Q. Video Generation Stack
R. Data Analysis + Report Drafting Stack
S. Code / Developer Assistance Stack
T. Cost / Provider Governance / Observability Stack
U. Evaluation / Regression / JD Live Acceptance Stack

Codex must map current repo truth for each stack before implementation.

5. Stack A — Global Human Interface / Semantic Intent Stack

Purpose

Understand what the user wants from messy human language.

Canonical owners

SemanticInterpreterProvider through Provider Governance
PH1.X for deterministic validation and routing
PH1.WRITE for final clarification wording

OpenAI role

classify intent
propose operation
propose targets
propose owner candidate
propose risk flags
propose clarification question
return schema-bound SemanticMeaningProposalPacket

Selene role

validate schema
validate category
validate current/recent target
validate identity/access implications
validate protected risk
select canonical owner
create HumanConversationDirective
reject malformed or authority-granting provider output

Required stack components

SemanticMeaningProposalPacket
CanonicalIntentTaxonomy
ReferenceCandidateLedger
HumanConversationDirective
ProviderDecisionTracePacket
ClarificationPolicy
AmbiguityThresholdPolicy

Required proof

provider-off zero attempt
fake-provider valid proposal
malformed provider rejection
wrong-owner proposal rejection/reroute
protected action proposal routed to protected gate
unseen paraphrase tests
negative unrelated-input tests

6. Stack B — Web Search + Source Evidence Stack

Purpose

Answer public, source-backed, current, or evidence-required questions with accepted sources, links, citations/source chips, and claim verification.

Canonical owners

PH1.E for search, source acceptance, page evidence, claim verification
Provider Governance for provider routing/costs
PH1.WRITE for final human answer and source presentation
Desktop/iPhone render approved source chips only

OpenAI role

query planning assistance
search provider capability where approved
source summarization
claim extraction
comparison assistance
answer draft assistance

Selene role

decide whether search is needed
select provider through provider router
apply budget and kill switch
execute search only when allowed
rank sources
accept/reject sources
fetch/read pages only when URL-fetch policy allows
extract evidence chunks
verify claims against accepted evidence
build source chips
remove unsupported claims
produce audit/cost evidence

Required stack components

SearchNeedClassifier
QueryPlanner
ProviderRouter
ProviderBudgetGate
SearchExecutionPacket
SourceCandidatePacket
SourceAcceptancePacket
PageFetchPolicy
EvidenceChunkPacket
ClaimExtractionPacket
ClaimVerificationPacket
SourceChipPacket
SearchAnswerPacket
SearchAuditPacket

Presentation requirements

short direct answer first
accepted source chips
no raw source dumps
no raw provider JSON
no rejected sources in normal UI
no long URLs in response_text
clean tts_text without source metadata
uncertainty phrased naturally when evidence is mixed

Provider-off behavior

no provider attempts
no network dispatches
safe degraded answer or clarification
no hidden fallback provider

Required proof

public search allowed without simulation
protected execution not allowed through search
source accepted/rejected evidence
claim unsupported removal
source chips created from accepted sources only
TTS clean
provider counters correct
fake-provider tests
live-provider tests opt-in only

7. Stack C — Image-Backed Search + Visual Presentation Stack

Purpose

Present relevant approved images or photos with public/entity search results when available and safe.

Canonical owners

PH1.E for image metadata, relevance, source-page validation, safety
PH1.WRITE for text + visual presentation packet
Desktop/iPhone render approved image cards only

OpenAI role

image relevance assistance
visual description assistance
image search provider capability where approved
multimodal interpretation where allowed

Selene role

decide whether images are useful
fetch/use approved image metadata only
validate source page
validate relevance
validate safety
reject raw image URL click targets
build image cards
fallback to text + source chips if images are not approved

Required stack components

ImageNeedClassifier
SearchImagePacket
ImageRelevancePacket
ImageSafetyPacket
ImageSourcePagePacket
ImageDisplayCardPacket
VisualEvidencePacket

Required proof

image cards use approved metadata only
no fabricated real images
source-page links safe
raw image URLs not normal click targets
irrelevant image rejected
unsafe image rejected
fallback works
Desktop does not choose images

8. Stack D — Deep Research Stack

Purpose

Perform high-effort, multi-source research when explicitly requested or lawfully escalated.

Canonical owners

PH1.E for research plan, source evidence, claim verification
Provider Governance for cost/budget/caps
PH1.WRITE for final report

OpenAI role

deep research assistance
research planning
source synthesis
claim extraction
outline drafting

Selene role

decide deep research eligibility
require explicit user intent or approved escalation
apply cost cap
select providers
accept/reject sources
verify claims
track contradictions
produce structured evidence report

Required stack components

DeepResearchIntentPacket
ResearchPlanPacket
ResearchBudgetPacket
ResearchSourceLedger
ClaimVerificationLedger
ContradictionLedger
DeepResearchReportPacket

Required proof

deep research off by default unless requested/approved
budget cap enforced
source ledger present
claim verification present
unsupported claims removed
normal tests do not call live providers

9. Stack E — Writing + Transformation Stack

Purpose

Write, rewrite, format, transform, adapt tone, compress, expand, and restructure content while respecting target, audience, evidence, language, and safety.

Canonical owners

PH1.WRITE for final output
PH1.X for request validation and target resolution
PH1.M/PH1.E if memory/evidence is required
Provider Governance for writing provider calls

OpenAI role

draft text
rewrite text
style transformation
summarization/compression
expansion
format conversion
language adaptation

Selene role

resolve target
validate allowed transformation
preserve evidence boundaries
apply style policy
apply language policy
remove unsupported claims
separate display_text and tts_text
hide raw provider output

Required stack components

WritingIntentPacket
TargetResolutionPacket
WritingStylePolicy
AudiencePolicy
FormatPolicy
LanguagePolicy
WritingDraftPacket
WritingValidationPacket
WriteOutputPacket
TtsOutputPacket

Style/presentation capabilities

one-line answer
bullet points
paragraphs
table
executive summary
email
report
SOP
legal-style plain summary
friendly tone
formal tone
concise tone
technical tone
TTS-safe spoken version

Required proof

rewrite previous answer
one-line compression
style change with same facts
format change with same facts
unsupported claim not introduced
TTS-safe split
raw provider JSON hidden

10. Stack F — Presentation + TTS-Safe Output Stack

Purpose

Decide how answers should be shown, spoken, structured, and visually supported.

Canonical owners

PH1.WRITE for presentation and wording
PH1.TTS for approved speech output
Desktop/iPhone render/play approved packets only

OpenAI role

propose natural wording
propose concise spoken version
adapt language/tone

Selene role

choose display structure
choose source chip display
choose image/card/artifact display
separate response_text/display_text/tts_text
prevent source metadata in speech
apply user preferences where allowed
apply accessibility constraints

Required stack components

PresentationDirective
DisplayTextPacket
TtsTextPacket
SourceChipPresentationPacket
VisualPresentationPacket
ArtifactPresentationPacket
AccessibilityPolicy
UserPreferencePresentationPacket

Required proof

short answer when requested
detailed answer when requested
bullet point presentation
paragraph presentation
TTS does not read source metadata
Desktop renders only approved packets

11. Stack G — Memory + Recall + Preference Stack

Purpose

Recall, update, forget, and apply memory safely with identity/access scope.

Canonical owners

PH1.M for memory
Identity + Access + Authority Spine for memory scope
PH1.X for routing
PH1.WRITE for final wording

OpenAI role

salience suggestion
summary proposal
semantic retrieval assistance
embedding support
context compaction proposal

Selene role

decide memory permission
scope memory by identity/tenant/session
retrieve evidence
check staleness/conflicts
apply forget/update law
apply presentation preferences where allowed

Required stack components

MemoryIntentPacket
MemoryScopePacket
MemoryEvidencePacket
MemoryRecallPacket
MemoryConflictPacket
MemoryWriteProposalPacket
MemoryUpdatePacket
MemoryForgetPacket
PreferencePacket
ContextCompactionPacket

Required proof

unknown speaker denied private memory
confirmed user scoped memory allowed
memory conflict surfaced
forget/update permission enforced
presentation preference applies in-session
persistent preference follows memory law

12. Stack H — File Question Answering + Document Stack

Purpose

Answer questions about user-provided files, summarize files, transform file content, extract facts, and generate derived documents while preserving file scope and evidence.

Canonical owners

PH1.E for file evidence and source acceptance
PH1.M only if file memory is requested/allowed
PH1.WRITE for final answer/document wording
Artifact/export owners where files are created

OpenAI role

file understanding
summary drafting
entity extraction
question answering assistance
format transformation

Selene role

validate file permission
track file provenance
extract accepted evidence
block prompt injection inside files
verify claims against file evidence
separate file facts from user instructions
build artifact/document outputs through approved artifact owners

Required stack components

FileAdmissionPacket
FileEvidencePacket
FileScopePacket
FileQuestionPacket
FileAnswerEvidencePacket
FilePromptInjectionDefensePacket
DerivedArtifactPacket
ArtifactProvenancePacket

Required proof

file text cannot override AGENTS/system law
file answer cites accepted file evidence where required
private file denied without access
summary does not invent unsupported facts
artifact provenance recorded

13. Stack I — Tool / Connector / MCP Stack

Purpose

Plan, validate, and execute read-only or write-capable tools/connectors through Selene-owned permission and protected-execution gates.

Canonical owners

PH1.E for tool/connector evidence and routing
Access/Authority for permission
SimulationExecutor for protected/write actions where required
Provider Governance for MCP/tool provider policy

OpenAI role

tool proposal
parameter extraction proposal
connector lookup proposal
workflow explanation

Selene role

validate tool permission
validate connector scope
validate parameters
separate read-only vs write/protected actions
apply authority and simulation gates
execute only through canonical tool owner
record audit

Required stack components

ToolNeedPacket
ToolProposalPacket
ToolScopePacket
ToolParameterValidationPacket
ToolExecutionDecisionPacket
ConnectorScopePacket
McpConnectorEvidencePacket
ProtectedToolActionPacket

Required proof

read-only public lookup allowed where scoped
tool write denied without authority
connector read denied without scope
provider tool proposal cannot execute directly
prompt injection from connector output blocked

14. Stack J — Protected Action + Simulation Stack

Purpose

Execute protected business actions only when identity, access, authority, simulation, confirmation, idempotency, and audit all pass.

Canonical owners

PH1.X for protected classification
Access/Authority for authority
SimulationExecutor for protected execution
Storage/Audit for evidence
PH1.WRITE for refusal/success wording

OpenAI role

understand that the user requested a protected action
extract proposed action/target as a proposal
explain refusal or required next step in plain language

Selene role

classify protected action
validate authority
lookup approved simulation
require confirmation where needed
ensure idempotency
audit execution or denial
fail closed on uncertainty

Required stack components

ProtectedRiskPacket
AuthorityDecisionPacket
SimulationLookupPacket
ConfirmationPacket
IdempotencyPacket
SimulationGateDecisionPacket
SimulationExecutionPacket
FailClosedEvidencePacket
ProtectedAuditPacket

Required proof

unknown speaker protected action fails closed
confirmed user without authority fails closed
authority without simulation fails closed
simulation without authority fails closed
provider cannot approve action
public explanation still allowed for protected topic

15. Stack K — Identity + Access + Authority Stack

Purpose

Determine who might be speaking, what scope they receive, and whether they may perform an action now.

Canonical owners

Wake/PH1.L/PH1.C for activation/session/transcript
Voice ID for identity evidence
Access resolver for scope
Authority owner for action permission
SimulationExecutor for protected execution

OpenAI role

none for identity proof
none for access grant
none for authority grant
semantic risk classification only through GHCS

Selene role

wake/listen/session
voice evidence
liveness evidence
access scope
memory scope
tool scope
authority decision
protected fail-closed

Required stack components

WakeDecisionPacket
SessionIdentityBindingPacket
TranscriptAdmissionPacket
SpeakerIdentityEvidencePacket
LivenessEvidencePacket
AccessScopePacket
AuthorityDecisionPacket

Required proof

wake does not identify
Voice ID does not grant authority
unknown speaker public-safe only
unknown speaker private memory denied
unknown speaker protected action fail closed
Desktop/Adapter no-authority proof

16. Stack L — Voice / Wake / Session / Realtime Stack

Purpose

Provide natural voice interaction while preserving session boundaries, identity evidence, TTS safety, interruption behavior, and runtime ownership.

Canonical owners

PH1.W / Wake owner for activation
PH1.L / PH1.C for session and transcript admission
Voice ID owner for identity evidence
PH1.TTS for approved speech
Desktop capture/playback/render only
Adapter transport only

OpenAI role

STT / realtime transcription where approved
realtime voice assistance where approved
TTS generation where approved
semantic proposals through governed provider interfaces

Selene role

control wake/session state
admit transcripts
bind session/turn
separate transcript from identity
manage barge-in/interruption
approve TTS text before speech
prove current app/runtime path

Required stack components

WakeDecisionPacket
TranscriptPacket
VoiceSessionPacket
BargeInPacket
RealtimeProviderPacket
VoiceOutputPacket
TtsTextPacket
DesktopProvenancePacket
AdapterProvenancePacket

Required proof

voice transcript captured
session bound correctly
TTS speaks approved_tts_text only
barge-in behavior recorded
Desktop does not decide meaning
Adapter does not decide meaning
JD live voice acceptance where visible

17. Stack M — Translation + Language Adaptation Stack

Purpose

Translate, answer in the user’s language, preserve meaning, adapt style, and avoid accidental language drift.

Canonical owners

PH1.X for intent/routing
PH1.WRITE for language output
Provider Governance for language provider calls

OpenAI role

language detection
translation
style adaptation
multilingual rewrite

Selene role

decide requested language
preserve factual evidence
separate translation from new claims
apply user preference where memory law allows
ensure TTS language compatibility where applicable

Required stack components

LanguageIntentPacket
LanguageDetectionPacket
TranslationRequestPacket
TranslationValidationPacket
LanguagePreferencePacket

Required proof

answer in requested language
translation preserves facts
source-backed claims not changed by translation
TTS language output safe

18. Stack N — Summarization + Compression Stack

Purpose

Summarize prior answers, files, sources, conversations, reports, or long evidence into concise or structured output.

Canonical owners

PH1.X for target resolution
PH1.E for evidence/file/source summarization where evidence is involved
PH1.M for memory summaries where memory is involved
PH1.WRITE for final summary

OpenAI role

summarization
compression
outline generation
key point extraction

Selene role

validate target
validate scope
preserve evidence boundaries
avoid unsupported conclusions
format according to user request

Required stack components

SummaryIntentPacket
SummaryTargetPacket
CompressionPolicy
SummaryDraftPacket
SummaryValidationPacket

Required proof

one-line summary
bullet summary
detailed summary
file summary uses file evidence
source summary uses accepted sources
private memory summary denied without scope

19. Stack O — Artifact / Document / Slide / Spreadsheet Stack

Purpose

Create, edit, transform, export, or present durable artifacts such as docs, slides, spreadsheets, reports, and build documents.

Canonical owners

Artifact/document generation owner per repo/tooling truth
PH1.WRITE for content
PH1.E for evidence-backed artifacts
PH1.M only if memory is used
Access/Authority where private/protected data is involved

OpenAI role

draft artifact content
transform content
structure documents
summarize source material

Selene role

validate artifact request
select artifact type
preserve provenance
validate data source
apply formatting rules
export through approved tool path
avoid private/protected leakage

Required stack components

ArtifactIntentPacket
ArtifactTypePacket
ArtifactSourceEvidencePacket
ArtifactDraftPacket
ArtifactValidationPacket
ArtifactExportPacket
ArtifactProvenancePacket

Required proof

artifact created with correct type
source provenance retained
private data scope enforced
no unsupported claims introduced
export proof where applicable

20. Stack P — Image Generation / Editing Stack

Purpose

Generate or edit images safely, accurately, and with provenance.

Canonical owners

Image generation provider interface through Provider Governance
PH1.X for request validation
PH1.WRITE for final presentation wording
Artifact/media owner for generated image output

OpenAI role

image generation
image editing
style transfer where allowed
visual prompt interpretation

Selene role

validate user request
validate likeness/self-reference requirements
apply safety rules
preserve provenance
present output as generated media
separate generated images from real source images

Required stack components

ImageGenerationIntentPacket
ImagePromptPacket
ImageSafetyPacket
ImageGenerationRequestPacket
GeneratedImagePacket
ImageProvenancePacket

Required proof

generated image labeled/provenanced
unsafe request blocked
real source image not fabricated
self-likeness rules followed where applicable

21. Stack Q — Video Generation Stack

Purpose

Generate or transform video where approved.

Canonical owners

Video generation provider interface through Provider Governance
PH1.X for request validation
Artifact/media owner for output

OpenAI role

video generation/transformation where capability is enabled
storyboard/prompt assistance

Selene role

validate request
apply safety/provenance
control cost/caps
store/present output through approved media path

Required stack components

VideoGenerationIntentPacket
VideoPromptPacket
VideoSafetyPacket
GeneratedVideoPacket
VideoProvenancePacket

Required proof

provider-off behavior
fake-provider behavior
safety block
provenance attached
cost cap enforced

22. Stack R — Data Analysis + Report Drafting Stack

Purpose

Analyze user-provided numbers, public data, files, or advisory datasets and draft reports without claiming official protected execution unless simulation authority exists.

Canonical owners

PH1.E for data/file evidence
PH1.WRITE for advisory report
SimulationExecutor only for official protected reports when authorized

OpenAI role

interpret data
summarize trends
draft report narrative
suggest charts/tables

Selene role

distinguish advisory vs official execution
validate data source
calculate where deterministic calculation is required
verify claims against data
label draft/advisory status
route official business execution to simulation gate

Required stack components

DataAnalysisIntentPacket
DatasetEvidencePacket
CalculationPacket
ReportDraftPacket
AdvisoryStatusPacket
OfficialExecutionRiskPacket

Required proof

advisory draft allowed from user-provided data
official report from company systems requires protected execution
unsupported numeric claims removed
draft does not claim filed/submitted/approved

23. Stack S — Code / Developer Assistance Stack

Purpose

Help with code explanation, generation, refactoring advice, test planning, and Codex instruction drafting without violating AGENTS law or repo authority.

Canonical owners

PH1.WRITE for explanation/instructions
Codex obeys AGENTS.md for repo work
Provider Governance for code-generation provider calls

OpenAI role

code reasoning
patch suggestion
test design
instruction drafting
architecture comparison

Selene role

apply AGENTS law
avoid unauthorized code edits
require repo truth
require correct owner
require tests and proof
separate advice from execution

Required stack components

CodeHelpIntentPacket
RepoTruthRequirementPacket
CodexInstructionPacket
CorrectOwnerProofRequirement
TestPlanPacket

Required proof

Codex instruction includes AGENTS read
lane declaration
clean tree
file scope
tests
backend/live proof when needed
no phrase patches
no duplicate owners

24. Stack T — Cost / Provider Governance / Observability Stack

Purpose

Control all provider usage, budget, cost, kill switches, model policy, capability routing, tracing, and provider health.

Canonical owners

Provider Governance
Storage/Audit for usage evidence
PH1.E/PH1.WRITE/etc as requesting owners only

OpenAI role

provide capability when governance permits
return structured outputs/tool results/generation results

Selene role

provider registry
model allowlist/denylist
capability routing
budget checks
kill switches
pre-network counters
network dispatch counters
latency/cost evidence
provider-off behavior
fake-provider behavior
circuit breakers
privacy/data-egress controls

Required stack components

ProviderRegistry
ModelGovernancePacket
ProviderPreflightPacket
ProviderBudgetPacket
ProviderCallCounterPacket
ProviderNetworkDispatchPacket
ProviderCostEvidencePacket
ProviderHealthPacket
ProviderCircuitBreakerPacket
ProviderDataEgressPacket
ProviderTracePacket

Required proof

provider disabled = zero attempts
provider disabled = zero network dispatches
fake provider non-billable
normal tests no live paid providers
budget exceeded safe-degrades
startup does not call providers

25. Stack U — Evaluation / Regression / JD Live Acceptance Stack

Purpose

Prove that Selene works on real routes, not only in unit tests.

Canonical owners

Testing/eval harness owners per repo truth
Storage/Audit for backend evidence
Desktop/Adapter provenance for live app tests
JD live acceptance for user-visible validation

OpenAI role

eval generation assistance
grader assistance where approved
failure summarization assistance

Selene role

exact test commands
non-vacuous test proof
real-path smoke
voice-first smoke where practical
backend evidence verification
JD live acceptance
repair loop
old path regression

Required stack components

EvalCasePacket
RegressionMatrix
JDLiveScenarioPacket
BackendEvidenceVerificationPacket
RealPathSmokePacket
FailureRepairPacket
AcceptanceStatusPacket

Required proof

unit tests passed
real route exercised
backend evidence agrees
Desktop current app proven where relevant
JD live accepted where visible
no stale/duplicate path won

26. Cross-Stack Routing Law

PH1.X must route to function stacks by canonical intent category and validated scope, not by phrase patches.

Examples:

"Can you check what people are saying?" → Web Search + Source Evidence Stack
"Give me one line." → Writing + Transformation Stack / Summarization Stack
"What did I say yesterday?" → Memory Stack
"Approve payroll." → Protected Action + Simulation Stack
"What about Sydney?" → Semantic Intent Stack + target/slot validation
"Show me pictures." → Image-Backed Search + Visual Presentation Stack
"Turn this into a slide deck." → Artifact Stack
"Translate this into Chinese." → Translation Stack

The production system must not implement these as exact phrase checks.

These examples belong in tests/evals, not in routing law.

27. Cross-Stack Presentation Law

PH1.WRITE is the final human output boundary.

Every stack must hand PH1.WRITE:

validated intent
accepted evidence
allowed sources
scope/access limits
risk status
presentation directive
style/language preference where allowed
TTS policy
UI metadata packets

PH1.WRITE must output:

display_text or response_text
tts_text where speech is enabled
source_chips where source-backed
image_cards where approved
artifact links/cards where approved
clarification text where needed
refusal text where required

PH1.WRITE must not:

invent facts
invent sources
upgrade weak evidence
ignore claim verification
read source metadata in TTS
show raw provider JSON
hide protected denial
bypass identity/access/simulation gates

28. Cross-Stack Decision-Making Law

Every function stack must define its decision surface.

For each stack, Codex must identify:

what decisions the stack owns
what decisions OpenAI may only propose
what decisions PH1.X owns
what decisions PH1.E owns
what decisions PH1.M owns
what decisions PH1.WRITE owns
what decisions Access/Authority owns
what decisions SimulationExecutor owns
what decisions Desktop/Adapter must never own

No stack may leave decisions implicit inside provider text.

All major decisions must produce evidence packets or trace entries.

29. Cross-Stack Proof Law

Every stack must prove:

provider-off behavior
fake-provider behavior
malformed provider result handling where provider involved
identity/access scope where private/protected data involved
protected fail-closed where protected actions involved
claim/evidence verification where factual claims involved
presentation separation where user-visible
TTS safety where spoken
Desktop/Adapter no-authority where clients touched
backend evidence
JD live acceptance where visible
old path cleanup only after proof

Unit tests are gates.

Real-path proof and JD live acceptance are product acceptance when visible.

30. Codex Repo-Truth Assignment

Before designing the full implementation build plan, Codex must perform repo truth across all canvases and this stack map.

Codex must read:

AGENTS.md
all architecture canvases supplied by JD
all required repo architecture docs
current owner inventories
current build execution order docs
current authoritative engine inventory

Codex must produce:

Selene Overall Architecture Repo-Truth Report

The report must include:

1. current canonical owners by stack
2. existing files for each stack
3. current packet/schema surfaces
4. current provider surfaces
5. current Desktop/Adapter surfaces
6. current tests/smokes/evals
7. current old paths
8. current wrong-owner paths
9. current duplicate-risk paths
10. current missing stacks
11. current stack readiness rating
12. first safe implementation sequence
13. docs that need surgical synchronization
14. activation packs required before implementation
15. exact stop conditions

No implementation is allowed during repo-truth reporting unless JD explicitly gives implementation scope.

31. Codex Build Plan Assignment After Repo Truth

After repo truth, Codex must design:

Selene Overall Function Stack Build Plan

The build plan must be slice-based and must not implement everything at once.

The plan must identify:

first foundation slice
provider governance dependency
semantic proposal dependency
function stack priority order
identity/access dependency order
presentation dependency order
tests for each slice
backend evidence for each slice
JD live scenarios
old path retirement gates

The likely high-level sequence is:

1. Docs architecture linking / synchronization
2. Repo truth activation
3. Provider Governance baseline
4. Semantic Meaning Proposal baseline
5. PH1.X deterministic validation baseline
6. PH1.WRITE presentation baseline
7. Web Search + Source Evidence baseline
8. Presentation + Source Chips baseline
9. Identity + Access + Authority baseline
10. Memory scope baseline
11. Tool/File scope baseline
12. Voice/live route proof
13. Evals/JD live acceptance pack
14. Old path cleanup

Codex must adjust this sequence to current repo truth.

32. Final Standard

The future Selene architecture is:

A global human interface
+ probabilistic semantic intelligence
+ deterministic identity/access/authority
+ enterprise function stacks
+ Selene-owned validation
+ Selene-owned presentation
+ Selene-owned audit
+ provider governance
+ JD live proof

The system should feel natural to the user, but internally it must be structured, scoped, validated, auditable, and owner-correct.

OpenAI brings the raw intelligence.

Selene brings the enterprise operating system around that intelligence.

That is the standard.
