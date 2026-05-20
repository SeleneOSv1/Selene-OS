Selene Provider-First Function Architecture Cards

DOCUMENT TYPE:
FUNCTION ARCHITECTURE CARD PACK

CONTROLLING MASTER PLAN:
Selene Provider-First OpenAI Assisted Pivot Master Build Plan

PURPOSE:
Define each major Selene function as a provider-first architecture card so Codex can implement one clean vertical slice at a time inside existing canonical engines.

---

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

0. Controlling Rule

This document does not replace the master build plan.

It exists to translate the master plan into practical function-level architecture cards.

The controlling architecture remains:

One global provider-first architecture.
One architecture card per major function.
One vertical slice per function.
Live proof and backend evidence.
Remove old paths only after proof.
Repeat until every canonical engine is clean.

Non-negotiable rule:

OpenAI proposes.
Selene validates.
Selene decides.
Selene executes only when lawful.
Selene audits.

This document must never authorize:

new repo
duplicate engines
parallel brains
manual phrase patches
Desktop semantic authority
Adapter semantic authority
OpenAI protected-execution authority
old path deletion before proof

0A. AGENTS Law Mandatory Execution Rule

Codex must read AGENTS.md before any build, design, repo audit, implementation, repair, cleanup, or Codex instruction derived from this document.

AGENTS.md is controlling execution law unless JD explicitly overrides it in-thread.

No architecture card, build instruction, provider-first design, cleanup plan, or implementation slice may override AGENTS.md.

Before any Codex build or major design/repo task derived from this document, Codex must read:

AGENTS.md
docs/CORE_ARCHITECTURE.md
docs/SELENE_BUILD_EXECUTION_ORDER.md
docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md
relevant section docs for the specific engine/function

Codex must also obey:

No Python in repo work.
Start and end clean tree.
Declare execution lane before editing.
Existing owner discovery before new implementation.
Reuse canonical owners.
No duplicate paths.
No phrase patches.
No Desktop/Adapter brain.
Provider-off proof where provider work is involved.
Fake-provider proof where relevant.
Backend evidence proof.
JD live proof where user-visible.
Old path retirement only after proof.

If Codex has not read AGENTS.md in the current run, it must stop before building and report:

AGENTS_LAW_NOT_READ_FOR_CURRENT_RUN

0B. Codex Build Derivation Rule

Every Codex build instruction derived from any card in this document must include:

read AGENTS.md first
read mandatory first-read architecture docs
lane declaration
clean-tree proof
fresh remote truth where required
existing owner discovery
provider coverage classification
canonical owner map
file-scope approval
build-specific test plan
provider governance proof where relevant
model governance proof where relevant
data-egress/privacy proof where relevant
prompt-injection defense proof where relevant
backend evidence proof
backend report requirement
JD live acceptance where user-visible
JD live repair loop where user-visible
provider-off proof where relevant
fake-provider proof where relevant
old path classification
old behavior regression proof
old path retirement only after proof
final acceptance status
final clean tree proof
commit/push proof where edits are made

If a Codex instruction is derived from this document but does not include these requirements, Codex must stop with:

FUNCTION_CARD_BUILD_DERIVATION_INCOMPLETE

0C. Function Card Activation Gate

These function cards are architecture references until Codex performs repo-truth discovery for the specific card being activated.

Codex must not treat any card as an executable implementation instruction until it creates a card-specific activation addendum from current repo truth.

For each card activation, Codex must fill in:

current repo owner files
current runtime path
current adapter path
current Desktop/iPhone path if relevant
current provider surfaces already present
current tests
current live proof status
current old paths to classify
current dead paths
current wrong-owner paths
current retained compatibility paths
exact files proposed for edit
exact tests to run
provider calls allowed yes/no
fake provider required yes/no
provider-off proof required yes/no
JD live acceptance required yes/no
backend evidence refs expected
old-path retirement condition

If Codex cannot fill the activation addendum from current repo truth, it must stop before implementation and report:

FUNCTION_CARD_ACTIVATION_REPO_TRUTH_REQUIRED

A card may be used for implementation only after:

AGENTS.md read in current run
mandatory architecture docs read in current run
clean tree proven
existing owner discovery complete
provider coverage classification complete
file-scope approval clear
baseline tests identified
live proof requirement identified
backend evidence requirement identified

0D. Final Readiness Boundary

This canvas is Codex-ready as a function architecture reference for:

Build 0A / 0C repo-truth discovery
provider coverage normalization
function-card activation addenda
first provider governance implementation slice

This canvas is not permission to:

implement all cards at once
skip AGENTS.md
skip repo-truth discovery
skip existing-owner reuse proof
create duplicate engines
create parallel provider brains
edit Desktop/Adapter semantics
use live OpenAI providers without governance
remove old paths before proof
start protected simulations without explicit JD approval

The next lawful Codex step is:

Read AGENTS.md.
Read required architecture docs.
Prove clean tree.
Run Build 0A / 0C repo-truth and provider coverage normalization.
Then activate Card 1 — Provider Governance only.

0E. Build-Specific Test Plan and JD Live Acceptance Supremacy Rule

Every build derived from this document must define exactly what must be tested before the build starts.

Building something without proving that it works is wasted time and energy.

Cargo, unit tests, mocked tests, fake-provider tests, endpoint tests, xcodebuild, and backend harness tests are mandatory safety gates, but they are not final product acceptance for user-visible/runtime behavior.

For every build that affects user-visible behavior, voice, Desktop UI, TTS, PH1.X, PH1.M, PH1.WRITE, PH1.E, search, provider behavior, memory, routing, context, protected classification, protected execution, files, images, tools, or artifacts, the build is not considered passed until:

1. Codex specifies the exact live scenario JD must test.
2. JD performs the live test on the latest current app/build where applicable.
3. Codex observes or captures the exact prompt/transcript/input.
4. Codex inspects backend evidence immediately after the live test.
5. Backend evidence proves the correct owner handled the behavior.
6. Visible/audible behavior matches the expected result.
7. Any failure is repaired in the correct owner.
8. JD retests the failed scenario after repair.
9. The scenario passes live.
10. Backend evidence agrees with the passed live behavior.

If cargo passes but JD live testing fails, the build status is:

JD_LIVE_ACCEPTANCE_FAILED

Codex must not call the build complete.

Codex must repair the correct owner, rerun the failed JD live scenario, prove backend evidence, and only then report acceptance.

Build status meanings:

CODEX_TESTED:
Cargo/tests/checks/fake-provider/backend harness passed, but JD live acceptance has not passed or was not required.

REAL_APP_SMOKE_PASSED:
Codex proved a real app/runtime smoke path, but JD has not accepted the behavior unless explicitly stated.

JD_LIVE_ACCEPTANCE_PASSED:
JD live testing passed and backend evidence agrees with the live behavior.

JD_LIVE_ACCEPTANCE_FAILED:
JD live testing failed, regardless of cargo/test success.

BACKEND_EVIDENCE_VERIFICATION_FAILED:
Visible behavior and backend evidence do not agree, or backend evidence is missing.

Every build instruction must include a build-specific test plan:

what was built
what exact behavior must be proven
which owner must prove it
which old behavior must still work
which negative cases must fail safely
which provider-off case must be tested
which fake-provider case must be tested if relevant
which backend packet/evidence must exist
which JD live prompt/input must be used
what visible/audible result JD should see/hear
what backend evidence Codex must inspect
what status counts as pass
what status counts as fail
what repair loop applies if JD live test fails

Codex must provide a backend report for every implementation build explaining how the build passed what it was supposed to build.

The backend report must include:

exact tested scenario
exact JD live prompt/input where applicable
exact captured transcript/input where applicable
expected result
actual result
owner engine that handled it
backend packet/evidence refs
provider calls real/fake/off
provider counters/budget where relevant
protected fail-closed proof where relevant
old behavior regression proof
failure/repair history if any
final acceptance status

A build may skip JD live testing only when it is genuinely non-user-visible, docs-only, or governance-only with no runtime behavior change. In that case Codex must explicitly report:

JD_LIVE_ACCEPTANCE_NOT_APPLICABLE

and explain why no live behavior exists to test.

Final rule:

Cargo is a safety gate.
Backend evidence is proof of route.
JD live testing is product acceptance for user-visible/runtime builds.
A user-visible/runtime build is not passed until JD live behavior and backend evidence agree.

1. Standard Architecture Card Template

Each function card must use this structure:

Function name
Coverage category
Implementation status
Canonical Selene owner
Secondary Selene owners
Forbidden owners
AGENTS law requirements
OpenAI capability used
Selene-owned control
Provider interface
Canonical packet
Hot / warm / cold path
Contract versioning requirement
Model governance requirement
Privacy/data-egress boundary
Prompt-injection risks
Observability / tracing requirement
Rate limit / circuit breaker requirement
File-input policy requirement
Streaming finalization requirement
Artifact governance requirement
Deferred-service status
Failure behavior
Provider-off behavior
Old paths to classify/remove
First vertical slice
Build-specific test plan
Tests
JD live proof
Backend evidence proof
Backend report requirement
Repair loop if JD live testing fails
Retirement condition for old paths

Coverage category must use one or more of:

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

Implementation status must use one of:

INITIAL_IMPLEMENTATION_ALLOWED
COVERED_FOR_ARCHITECTURE
NOT_INITIAL_IMPLEMENTATION
BUILD_ONLY_AFTER_JD_APPROVAL
DEFERRED_UNTIL_PROVIDER_GOVERNANCE_EXISTS
DEFERRED_UNTIL_CORE_RUNTIME_STABLE
DEFERRED_UNTIL_DISTRIBUTION_STRATEGY_APPROVED

Every Codex build derived from a card must still obey the master plan and AGENTS law.

1A. Global Hardening Fields For All Cards

Every card must explicitly answer these hardening fields before becoming an implementation instruction:

AGENTS law requirements:
Codex must read AGENTS.md and mandatory architecture docs in the current run before building.

Lane declaration:
Public/advisory, deterministic protected execution, or mixed request.

File-scope approval:
Whether existing files, spine files, contract files, Desktop, Adapter, or protected execution files are touched.

Existing owner discovery:
Exact current owners, runtime paths, tests, and old paths must be found before implementation.

Provider coverage classification:
Coverage category, provider interface, canonical packet, capability key, and implementation status.

Provider governance required:
Kill switch, budget/counter, provider-off, fake provider, provider evidence.

Model governance required:
Model allowlist, model pinning, fallback, upgrade/rollback rule where provider model is used.

Data-egress/privacy required:
Minimum necessary context, data class, redaction, retention/logging, speaker/tenant scope.

Prompt-injection defense required:
For user text, source text, file text, connector output, tool output, OCR, and provider summaries.

Observability/tracing required:
Provider trace, owner decision trace, backend evidence, visible behavior agreement.

Rate limit/circuit breaker required:
For live provider calls, retries, fallback, degraded state, and provider health.

File-input policy required:
Whenever uploaded files, file search, file lifecycle, vector stores, or provider file submission are involved.

Streaming finalization required:
Whenever streaming, realtime, partial output, or progressive display is involved.

Artifact governance required:
Whenever generated reports, charts, files, images, videos, or transformed documents are produced.

Deferred-service status:
Whether the card/surface is initial build, architecture-only, deferred, or JD-approval-only.

2. Architecture Cards To Build First

Recommended order:

1. Provider Governance Architecture Card
2. Voice Runtime Architecture Card
3. PH1.X Current Turn Understanding Card
4. PH1.WRITE Writing / Presentation Card
5. PH1.E Search / Tools / Evidence Card
6. PH1.M Memory / Recall Card
7. Vision / Files / Artifact Card
8. Tool / MCP / Connector Card
9. Eval / Regression / Optimization Card
10. Desktop / iPhone Boundary Card
11. Protected Simulation Execution Card

Card 1 — Provider Governance Architecture Card

Function name

Provider Governance

Coverage category

CORE_RUNTIME
COST_SCALE
ENTERPRISE_ADMIN
EVAL_OPTIMIZATION

Implementation status

INITIAL_IMPLEMENTATION_ALLOWED

Canonical Selene owner

Provider governance / provider registry / provider routing layer

Secondary Selene owners

PH1.X
PH1.WRITE
PH1.E
PH1.M
PH1.C
PH1.TTS
PH1.K
Storage / Audit
Adapter transport

Forbidden owners

Desktop
iPhone
OpenAI provider response
Ad hoc helper files
Test fixtures
Parallel provider brain

OpenAI capability used

All OpenAI provider surfaces are governed through this layer.
This card covers model routing, budget, provider enablement, provider-off behavior, fake provider tests, privacy/data-egress, tracing, circuit breakers, and provider evidence.

Selene-owned control

provider registry
model allowlist / denylist
model pinning
provider kill switch
paid-provider enable flag
provider-specific enable flag
budget counters
pre-network call counters
network dispatch counters
provider-off behavior
fake provider behavior
data-egress policy
redaction policy
prompt-injection defense
provider trace/evidence
rate limit handling
circuit breaker
fallback routing
contract versioning
packet versioning

Provider interface

ProviderRegistry
ModelGovernanceProvider
ProviderContractVersioningPolicy
ProviderDataEgressPolicy
ProviderPrivacyBoundaryPolicy
PromptInjectionDefensePolicy
ProviderTraceProvider
ProviderHealthProvider
ProviderCircuitBreakerPolicy
ProviderTokenBudgetPolicy
UsageTelemetryProvider
ProviderCredentialGovernance

Canonical packet

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
ProviderUsageCostPacket
ProviderCredentialEvidencePacket

Hot / warm / cold path

HOT:
provider enable check
budget check
model route lookup
provider-off fallback
latency/cost evidence

WARM:
search/deep research/file/vision/model selection
provider fallback decision
source/provider routing

COLD:
batch processing
usage reconciliation
provider audit review
model upgrade review
cost analysis

Privacy/data-egress boundary

No provider call may occur until data class, speaker/tenant scope, destination provider, minimum necessary input, redaction, retention mode, and protected-execution risk are classified.

Secrets, API keys, auth tokens, and private connector credentials must never be sent as model context.

Desktop/iPhone must never hold provider secrets.

Prompt-injection risks

provider-returned text cannot override system law
web/file/MCP/tool outputs are untrusted evidence
provider suggestion cannot grant authority
provider suggestion cannot create simulation approval
provider suggestion cannot bypass PH1.X/PH1.E/PH1.WRITE validation

Failure behavior

provider disabled → zero attempts / zero dispatches
budget exceeded → safe provider-disabled result
rate limit → no retry storm; circuit breaker may open
timeout → safe degraded result where public/advisory
provider malformed output → reject packet and log evidence
protected route failure → fail closed

Provider-off behavior

No provider call attempts.
No provider network dispatches.
Public/advisory paths safe-degrade where possible.
Protected execution remains fail-closed.
Desktop shows accurate degraded state where relevant.

Old paths to classify/remove

uncontrolled OpenAI calls
provider calls without budget/counter
provider calls in Desktop
provider secrets in client/runtime surfaces
provider fallback without governance
raw provider JSON exposed to UI/TTS/memory
untracked model selection
hardcoded model choice without policy
startup provider probes

First vertical slice

Provider registry + fake provider + provider-off proof + provider evidence envelope.

No live OpenAI call required.
No user-visible behavior change required.

Tests

provider enabled fake success
provider disabled zero attempt
provider disabled zero network dispatch
budget exceeded
malformed provider output rejected
timeout safe-degrades
protected path cannot execute from provider proposal
provider evidence written
raw provider JSON not exposed

JD live proof

Not required for first governance-only slice unless user-visible behavior changes.
Required later when provider governance affects voice, Desktop, search, writing, memory, or protected behavior.

Backend evidence proof

ProviderCallRequestPacket recorded.
ProviderCallResultPacket recorded.
ProviderCostEvidencePacket recorded.
Provider disabled path proves zero attempt/dispatch.
Provider failure path records failure reason.

Retirement condition for old paths

Old provider paths may be removed only after new provider registry/evidence path is proven, all existing active callers are migrated, provider-off tests pass, and no old caller remains reachable.

Card 2 — Voice Runtime Architecture Card

Function name

Voice Runtime

Coverage category

VOICE_RUNTIME
CORE_RUNTIME
COST_SCALE

Implementation status

INITIAL_IMPLEMENTATION_ALLOWED after Provider Governance baseline exists

Canonical Selene owner

PH1.W
PH1.C
PH1.TTS
PH1.K
PH1.L

Secondary Selene owners

Adapter
Desktop / iPhone shell
Storage / Audit
Voice ID evidence path
PH1.X for committed turn meaning

Forbidden owners

Desktop semantic logic
Adapter semantic logic
OpenAI direct runtime control
provider-side protected execution

OpenAI capability used

Speech-to-Text
Text-to-Speech
Realtime API
Realtime transcription
Voice Activity Detection
Realtime WebRTC / WebSocket / SIP later
Live translation later

Selene-owned control

wake acceptance
pre-wake privacy boundary
transcript admission
turn commit / reject
noise rejection
self-echo rejection
TTS approval
interruption control
session/sleep boundary
provider-off degraded state
backend voice evidence

Provider interface

SpeechToTextProvider
TextToSpeechProvider
RealtimeVoiceProvider
RealtimeTranscriptionProvider
RealtimeTransportProvider
VoiceActivityDetectionPolicy
LiveTranslationProvider
RealtimeSessionControlProvider
RealtimeServerSideControlProvider
RealtimeCostPolicy

Canonical packet

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
ProviderCallResultPacket
ProviderStreamFinalizationPacket

Hot / warm / cold path

HOT:
wake → STT → transcript admission → PH1.X → PH1.WRITE → TTS → re-arm

WARM:
barge-in
interrupt correction
voice language repair
provider retry/degraded voice

COLD:
voice quality evaluation
voice transcript repair datasets
voice model route optimization

Privacy/data-egress boundary

No OpenAI STT before lawful wake unless an explicitly approved wake architecture says otherwise.
Raw audio must be bounded by wake/session policy.
TTS may only receive approved_tts_text.
Voice ID evidence is not authority.

Prompt-injection risks

transcript text can contain malicious instructions
realtime tool calls must become ToolProposalPackets only
provider audio/session events cannot bypass PH1.X/PH1.E/SimulationExecutor

Failure behavior

STT failure → no committed turn unless fallback/admission allows
TTS failure → show accurate state; do not fake spoken output
Realtime failure → degrade to non-realtime STT/TTS if policy allows
VAD uncertainty → ask/retry rather than commit bad turn
protected request through voice → fail closed unless simulation + authority

Provider-off behavior

voice unavailable/degraded state shown accurately
no OpenAI STT/TTS dispatches
public typed path may still work
protected execution still fail-closed

Old paths to classify/remove

old Apple STT remnants
obsolete Apple/native TTS fallback if replaced and approved
stale duplicate listening loops
Desktop transcript decision logic
adapter voice shortcuts
old TTS status paths
pre-wake provider calls

First vertical slice

Wake → STT transcript → answer → approved TTS → playback completion → re-arm.

Tests

wake word not committed as prompt
cough/noise before wake rejected
transcript admitted only after lawful wake
TTS receives approved text only
TTS completion evidence recorded
provider-off voice degraded state
protected voice request fail-closed

JD live proof

JD says: Selene, what time is it in Sydney?
Expected: transcript captured, answer displayed, TTS spoken if enabled, re-arm works, backend evidence matches.

Backend evidence proof

wake evidence
TranscriptPacket
ProviderCallResultPacket
HumanConversationDirective
WriteOutputPacket
VoiceOutputPacket
playback completion evidence
re-arm/session evidence

Retirement condition for old paths

Old voice paths removed only after new voice spine passes live app proof, provider-off proof, protected fail-closed proof, and old accepted behavior is re-proven.

Card 3 — PH1.X Current Turn Understanding Card

Function name

Current Turn Understanding

Coverage category

CORE_RUNTIME
WRITE_PRESENTATION
MEMORY_RETRIEVAL
ADVISORY_ANALYSIS

Implementation status

INITIAL_IMPLEMENTATION_ALLOWED after Provider Governance and fake-provider proof exist

Canonical Selene owner

PH1.X

Secondary Selene owners

PH1.C
PH1.M
PH1.WRITE
PH1.E
PH1.L
Voice ID evidence path
Storage / Audit

Forbidden owners

Desktop
Adapter
OpenAI provider directly
PH1.M as live context brain
PH1.E as general context brain

OpenAI capability used

Structured Outputs
reasoning models
semantic interpretation proposal
reference resolution proposal
clarification proposal
intent/operation proposal

Selene-owned control

active frame
candidate generation
candidate scoring
hard disqualifiers
rejection ledger
protected-risk validation
reference target validation
clarification target validation
correction target validation
HumanConversationDirective

Provider interface

SemanticInterpreterProvider
ModelReasoningProvider
ReasoningControlProvider
ProviderConversationStatePolicy

Canonical packet

CurrentTurnInterpretationPacket
HumanConversationDirective
ProviderDecisionTracePacket
ProviderConversationStateRefPacket
PromptInjectionDefensePacket

Hot / warm / cold path

HOT:
short follow-ups
reference resolution
current turn routing
protected risk classification

WARM:
ambiguous topic switches
artifact edit requests
tool continuation
memory handoff

COLD:
PH1.X evals
prompt optimization
semantic regression corpus

Privacy/data-egress boundary

Send only compact active frame and allowed evidence refs.
Do not send private memory unless PH1.M permits.
Do not send protected company data unless explicitly allowed by data-egress policy.

Prompt-injection risks

user can ask provider to ignore Selene rules
retrieved evidence can contain instructions
provider proposal can be overconfident
provider can suggest wrong owner
provider can classify protected request as public

Failure behavior

provider unavailable → PH1.X uses deterministic fallback if available or asks clarification
provider malformed output → reject and log
low confidence → ask one focused clarification
protected uncertainty → fail closed

Provider-off behavior

PH1.X must still preserve protected fail-closed.
Public/simple route may use existing deterministic fallback if available.
No provider call attempt when disabled.

Old paths to classify/remove

deterministic_active_context shortcuts
deterministic_weather_context shortcuts
adapter semantic shortcuts
phrase patches
exact prompt branches
time/weather special cases outside owner

First vertical slice

User: “Can you give me one line?”
Provider proposes one_line operation targeting previous answer.
PH1.X validates target.
PH1.X routes to PH1.WRITE.

Tests

make it one line
make it shorter
make it warmer
what about Sydney
what is your name after time question
payroll protected request
provider says salary change is public → reject
provider chooses stale topic → reject

JD live proof

JD asks a normal answer, then says: Can you give me one line?
Expected: PH1.X targets previous answer, routes to PH1.WRITE, no stale topic hijack.

Backend evidence proof

CurrentTurnInterpretationPacket
selected/rejected candidates
HumanConversationDirective
PH1.X rejection ledger
protected fail-closed evidence where relevant

Retirement condition for old paths

Old PH1.X/Adapter shortcuts removed only after provider-assisted PH1.X validation passes unseen paraphrases, negative hijack tests, JD live proof, and backend evidence proof.

Card 4 — PH1.WRITE Writing / Presentation Card

Function name

Writing / Presentation

Coverage category

WRITE_PRESENTATION
CORE_RUNTIME
SEARCH_EVIDENCE
MEMORY_RETRIEVAL

Implementation status

INITIAL_IMPLEMENTATION_ALLOWED after PH1.X provider-assisted directive slice exists

Canonical Selene owner

PH1.WRITE

Secondary Selene owners

PH1.X
PH1.E
PH1.M
PH1.TTS
Desktop renderer
Storage / Audit

Forbidden owners

Desktop formatting brain
Adapter formatting shortcut
OpenAI final authority
raw provider output as final answer

OpenAI capability used

advanced writing
rewriting
summarisation
translation
tone control
structured formatting
emails/messages
reports
SOPs
proposals

Selene-owned control

answer policy
source discipline
memory wording
protected fail-closed wording
display_text
tts_text
source chips
image cards
artifact references
unsupported claim removal

Provider interface

WritingProvider
ModelReasoningProvider
PredictedOutputPolicy
CitationFormattingProvider
ProviderStreamingTransport

Canonical packet

WriteRequestPacket
WriteOutputPacket
SourceChipPacket
CitationPresentationPacket
ProviderStreamEventPacket
ProviderStreamFinalizationPacket
GeneratedArtifactPacket

Hot / warm / cold path

HOT:
short answer
one-line rewrite
TTS-safe response

WARM:
headers/bullets/tables
source-backed synthesis
memory-aware wording

COLD:
long reports
SOPs
proposals
large document transformations

Privacy/data-egress boundary

WritingProvider receives only allowed source/memory/tool evidence refs.
Private memory requires PH1.M permission.
Protected company data requires data-egress approval.

Prompt-injection risks

source text may instruct answer style or policy bypass
file content may ask model to ignore rules
provider may invent facts or citations
provider may overstate official execution

Failure behavior

provider unavailable → fallback concise writer if available or degraded answer
unsupported facts → remove or qualify
source mismatch → do not cite
protected wording uncertainty → fail closed phrase

Provider-off behavior

safe deterministic/plain fallback where available
no provider attempt/dispatch
no invented style claims
protected wording remains fail-closed

Old paths to classify/remove

plain robotic formatter if superseded
Desktop formatting logic
Adapter formatting logic
source dump wording
archive/session wording
TTS unsafe long text path

First vertical slice

PH1.X routes “give me one line” to PH1.WRITE.
WritingProvider returns one-line rewrite.
PH1.WRITE validates and emits display_text / tts_text.

Tests

make it shorter
one line
make it warmer
headers
bullets
search answer with source chips
protected fail-closed wording
provider invents source → reject

JD live proof

JD says: make it warmer / give me one line.
Expected: clean output, correct target, no Desktop formatting brain, backend WriteOutputPacket present.

Backend evidence proof

WriteRequestPacket
WriteOutputPacket
unsupported claims removed status
source/memory refs used
provider evidence
PH1.WRITE validation trace

Retirement condition for old paths

Old formatters removed after PH1.WRITE provider path passes target tests, source tests, protected wording tests, Desktop render proof, TTS-safe proof, and backend evidence proof.

Card 5 — PH1.E Search / Tools / Evidence Card

Function name

Search / Tools / Evidence

Coverage category

SEARCH_EVIDENCE
CORE_RUNTIME
ADVISORY_ANALYSIS
COST_SCALE

Implementation status

DEFERRED_UNTIL_PROVIDER_GOVERNANCE_EXISTS

Canonical Selene owner

PH1.E

Secondary Selene owners

PH1.X
PH1.WRITE
Storage / Audit
Provider governance
Access/Gov for permissioned data
SimulationExecutor for protected actions

Forbidden owners

Desktop provider calls
Adapter provider routing brain
OpenAI accepted-source authority
raw search provider output as final answer

OpenAI capability used

web search
deep research
file search
tool calling proposal
tool search
MCP connector proposal
source summarisation

Selene-owned control

search needed decision
query plan
provider selection
budget cap
accepted/rejected sources
claim verification
source chips
tool permission
mixed request split
protected execution boundary

Provider interface

SearchProvider
DeepResearchProvider
FileSearchProvider
ToolProposalProvider
ToolSearchProvider
McpConnectorProvider
CitationFormattingProvider
PromptInjectionDefensePolicy

Canonical packet

SearchEvidencePacket
SourceAcceptancePacket
DeepResearchEvidencePacket
FileEvidencePacket
ToolProposalPacket
ToolExecutionDecisionPacket
McpConnectorEvidencePacket
SourceChipPacket
CitationPresentationPacket
PromptInjectionDefensePacket

Hot / warm / cold path

HOT:
time/weather/basic read-only utilities where already owned
quick public search when necessary

WARM:
normal web search
file Q&A
tool proposal
connector read-only retrieval

COLD:
deep research
large source synthesis
background research job

Privacy/data-egress boundary

Public search can use public query.
Private/company/connector data requires scope and permission.
File search requires file permission.
Deep research requires explicit user intent or approved escalation.

Prompt-injection risks

web pages contain malicious instructions
file chunks contain prompt injection
connector outputs contain instructions
source text tries to trigger tools/protected actions

Failure behavior

provider off → safe degraded answer
source conflict → state uncertainty or best available source-backed answer
no accepted source → no factual claim
protected action mixed with search → public answer may proceed, protected part fails closed

Provider-off behavior

zero provider attempt/dispatch
cached/local answer only if allowed
safe no-search degraded response
protected fail-closed preserved

Old paths to classify/remove

raw source dumps
uncontrolled Brave/GDELT/OpenAI calls
provider shortcuts without budget
wrong-source acceptance
source chip bypass
PH1.E stale context logic
Adapter tool routing shortcuts

First vertical slice

PH1.X determines search needed.
PH1.E calls fake SearchProvider.
PH1.E accepts one source and rejects one source.
PH1.WRITE presents answer with source chip.

Tests

provider-off search
wrong-source rejection
accepted source chip
file Q&A permission
deep research requires cap
mixed salary search + salary change split
prompt injection in source blocked

JD live proof

JD asks a public search question and asks “show sources.”
Expected: clean answer, source chips, no raw dumps, backend PH1.E evidence.

Backend evidence proof

SearchEvidencePacket
SourceAcceptancePacket
claim verification
source chips
provider counters
PH1.E decision trace

Retirement condition for old paths

Old search/provider paths removed only after PH1.E provider path proves provider-off, source acceptance, claim verification, source chips, mixed-request split, and protected fail-closed.

Card 6 — PH1.M Memory / Recall Card

Function name

Memory / Recall

Coverage category

MEMORY_RETRIEVAL
CORE_RUNTIME
WRITE_PRESENTATION

Implementation status

DEFERRED_UNTIL_CORE_RUNTIME_STABLE

Canonical Selene owner

PH1.M

Secondary Selene owners

Storage / Archive / Audit
PH1.X
PH1.WRITE
Voice ID speaker evidence
Provider governance

Forbidden owners

Desktop memory brain
Adapter memory shortcuts
OpenAI provider memory as source of truth
PH1.X as durable memory brain

OpenAI capability used

summarisation
salience suggestion
topic labeling
embeddings
semantic retrieval
context compaction support

Selene-owned control

memory permission
what to remember
who owns memory
speaker scope
tenant scope
privacy class
fresh/topic/deep tier
forget/update
conflict/staleness
recall wording rules

Provider interface

EmbeddingProvider
ContextCompactionProvider
ModelReasoningProvider
FileSearchProvider where memory uses documents
ProviderConversationStatePolicy as non-authoritative support

Canonical packet

EmbeddingPacket
MemoryEvidencePacket
ContextCompactionPacket
ProviderConversationStateRefPacket
ProviderDataEgressPacket

Hot / warm / cold path

HOT:
recent active-context recall only when needed
speaker scope check

WARM:
today/yesterday/topic recall
memory handoff to PH1.X/PH1.WRITE

COLD:
deep recall
memory consolidation
archive summarisation
topic graph maintenance

Privacy/data-egress boundary

Private memory cannot be sent to provider unless scope and permission allow.
Unknown speaker cannot access JD private memory.
Provider compaction cannot replace PH1.M memory truth.

Prompt-injection risks

stored text may contain old malicious instructions
file-derived memory may contain injection
provider summaries may distort decisions
retrieved memory cannot grant authority

Failure behavior

memory unavailable → say not enough evidence or proceed without memory
conflicting memory → prefer newer/stronger evidence or ask
unknown speaker → public-safe only
forget request → follow memory law

Provider-off behavior

deterministic/local recall if available
no embedding/provider summarisation attempt
memory permissions still enforced

Old paths to classify/remove

adapter memory shortcuts
session-search wording
old 72-hour-only assumptions
unscoped memory paths
duplicate recall paths
dead memory helpers

First vertical slice

PH1.M uses existing evidence to answer “What did we decide earlier?” without session-search wording.
Provider assistance may be fake/local first.

Tests

fresh recall
yesterday recall
topic recall
old plan superseded by newer plan
forget/update
unknown speaker private-memory denial
provider-off recall behavior

JD live proof

JD asks: What did we decide about the provider-first plan?
Expected: natural memory answer with evidence-backed recall, no archive/session wording.

Backend evidence proof

MemoryEvidencePacket
speaker scope
source session/evidence refs
PH1.M decision trace
PH1.WRITE output

Retirement condition for old paths

Old memory paths removed only after PH1.M proves fresh/topic/deep recall, privacy scope, speaker scope, forget/update, and no adapter/Desktop memory brain remains.

Card 7 — Vision / Files / Artifact Card

Function name

Vision / Files / Artifacts

Coverage category

SEARCH_EVIDENCE
MEMORY_RETRIEVAL
ADVISORY_ANALYSIS
WRITE_PRESENTATION
FUTURE_OPTIONAL

Implementation status

DEFERRED_UNTIL_PROVIDER_GOVERNANCE_EXISTS
BUILD_ONLY_AFTER_JD_APPROVAL for image/video generation and official-record workflows

Canonical Selene owner

PH1.E for evidence
PH1.M for memory permission
PH1.WRITE for output/artifact wording
Storage for file/artifact lifecycle
Desktop for rendering only

Secondary Selene owners

Provider governance
Access/Gov for company/private files
SimulationExecutor for official records

Forbidden owners

OpenAI as truth authority
Desktop choosing/ranking images
generated artifact as official record without simulation
file upload automatically becoming memory/provider input

OpenAI capability used

Vision API
Files API
File Search
Image Generation
Video Generation later
Code Interpreter advisory artifact generation

Selene-owned control

file permission
file lifecycle
file input policy
visual evidence
artifact provenance
artifact approval
display permission
download permission
memory permission
official-record boundary

Provider interface

VisionProvider
ImageGenerationProvider
VideoGenerationProvider
FileLifecycleProvider
ProviderFileInputPolicy
FileSearchProvider
CodeInterpreterProvider
GeneratedArtifactGovernanceProvider

Canonical packet

VisualEvidencePacket
ImageGenerationPacket
VideoGenerationPacket
ProviderFileInputPacket
ProviderFileLifecyclePacket
FileEvidencePacket
GeneratedArtifactPacket
ArtifactProvenancePacket
AnalysisArtifactPacket

Hot / warm / cold path

HOT:
small image/file summary if already available

WARM:
OCR/image understanding
file Q&A
artifact draft

COLD:
large document transformation
chart/report generation
image/video generation
batch file processing

Privacy/data-egress boundary

file upload does not equal provider permission
file provider input requires scope and redaction
image/video raw data requires modality permission
generated artifacts require retention/deletion policy

Prompt-injection risks

document text may contain malicious instructions
OCR text may contain hidden prompts
generated files may contain unsafe or false claims
file search chunks are untrusted evidence

Failure behavior

file permission missing → ask/deny
generated artifact unsupported → advisory only
vision uncertainty → state limitation
official-record request → protected simulation gate

Provider-off behavior

file storage may still work
provider image/file understanding unavailable/degraded
no provider file submission
no artifact generation

Old paths to classify/remove

file upload treated as memory automatically
raw full file dumped to UI/TTS
image cards without provenance
generated assets treated as real evidence
Desktop image selection logic

First vertical slice

User uploads file/image.
Selene creates bounded FileEvidencePacket or VisualEvidencePacket.
PH1.WRITE summarizes with limitations.
No memory write unless permitted.

Tests

file upload permission
file input blocked without permission
prompt injection inside file ignored
image summary with limitations
generated artifact advisory-only
official record attempt fail-closed

JD live proof

JD uploads a document/image and asks for a short summary.
Expected: bounded evidence, clean answer, no raw dump, backend file/visual evidence.

Backend evidence proof

ProviderFileInputPacket
FileEvidencePacket or VisualEvidencePacket
PromptInjectionDefensePacket
GeneratedArtifactPacket if artifact produced
PH1.WRITE output trace

Retirement condition for old paths

Old file/image/artifact paths removed after new file-input, evidence, artifact governance, Desktop render, and privacy tests pass.

Card 8 — Tool / MCP / Connector Card

Function name

Tool / MCP / Connector

Coverage category

CORE_RUNTIME
SEARCH_EVIDENCE
DISTRIBUTION_SURFACE
ENTERPRISE_ADMIN

Implementation status

DEFERRED_UNTIL_PROVIDER_GOVERNANCE_EXISTS
BUILD_ONLY_AFTER_JD_APPROVAL for connector writes or protected tool actions

Canonical Selene owner

PH1.E
Access/Governance
SimulationExecutor for protected actions
Storage / Audit

Secondary Selene owners

PH1.X
PH1.WRITE
Provider governance
Connector governance

Forbidden owners

OpenAI direct tool execution
Desktop tool routing
Adapter tool brain
MCP output as authority

OpenAI capability used

function/tool calling
tool search
remote MCP
OpenAI-maintained connectors
ChatGPT connector developer mode later

Selene-owned control

tool registry truth
tool permission
tool argument schema
tool execution decision
read-only vs protected classification
simulation requirement
authority requirement
audit
connector secret governance

Provider interface

ToolProposalProvider
ToolSearchProvider
McpConnectorProvider
ChatGptConnectorSurface
ProviderCredentialGovernance
PromptInjectionDefensePolicy

Canonical packet

ToolProposalPacket
ToolExecutionDecisionPacket
McpConnectorEvidencePacket
ProviderCredentialEvidencePacket
PromptInjectionDefensePacket
ProviderWebhookEventPacket

Hot / warm / cold path

HOT:
simple read-only tool proposal

WARM:
connector read-only retrieval
file/doc/email/calendar lookup

COLD:
large workflow proposal
protected workflow after simulation design
enterprise connector rollout

Privacy/data-egress boundary

connector data must be scoped
secrets stay server-side
private/company data requires permission
MCP connector output is untrusted evidence

Prompt-injection risks

connector output may request tool calls
email/doc content may contain malicious instructions
MCP response may try to bypass Selene gates

Failure behavior

tool not found → ask/deny
tool proposal malformed → reject
protected tool request → simulation/authority gate
connector unavailable → degraded answer

Provider-off behavior

no provider tool proposal
Selene local tool registry may still work if allowed
protected fail-closed preserved

Old paths to classify/remove

adapter tool routing shortcuts
uncontrolled tool calls
connector secrets in client
provider-driven tool execution
old direct integrations without evidence

First vertical slice

Provider proposes a read-only tool.
PH1.E validates allowed tool and arguments.
Tool executes read-only.
PH1.WRITE summarizes result.

Tests

read-only allowed
protected write denied
malformed args rejected
prompt injection in connector output blocked
provider-off tool proposal
simulation missing fail-closed

JD live proof

JD asks for a read-only lookup or draft from retrieved info.
Expected: retrieval allowed only if scoped, no protected write, backend tool decision evidence.

Backend evidence proof

ToolProposalPacket
ToolExecutionDecisionPacket
McpConnectorEvidencePacket
Access/Gov result
SimulationExecutor gate if protected

Retirement condition for old paths

Old tool/connector paths removed after PH1.E tool decision path proves read-only, protected fail-closed, prompt-injection defense, connector evidence, and secret isolation.

Card 9 — Eval / Regression / Optimization Card

Function name

Eval / Regression / Optimization

Coverage category

EVAL_OPTIMIZATION
COST_SCALE
CORE_RUNTIME

Implementation status

INITIAL_IMPLEMENTATION_ALLOWED for fake/local eval harness
BUILD_ONLY_AFTER_JD_APPROVAL for live provider-assisted eval/fine-tuning work

Canonical Selene owner

Eval harness / acceptance matrix / Storage evidence

Secondary Selene owners

PH1.X
PH1.WRITE
PH1.E
PH1.M
PH1.C
PH1.TTS
PH1.K
Provider governance

Forbidden owners

OpenAI eval as final acceptance authority
unit tests as replacement for live proof
mock-only proof as product proof

OpenAI capability used

OpenAI Evals
Prompt Optimizer
Graders
model-assisted evaluation
synthetic test generation
fine-tuning later

Selene-owned control

golden expectations
regression corpus
acceptance thresholds
backend evidence verification
JD live acceptance
protected fail-closed gates
what is real vs mocked disclosure

Provider interface

EvalProvider
PromptOptimizationProvider
GraderProvider
FineTuningOptimizationProvider
ManualPromptTestingSurface
CodexDevelopmentAssistantSurface

Canonical packet

EvalResultPacket
PromptOptimizationEvidencePacket
GraderResultPacket
FineTuningOptimizationEvidencePacket
JDLiveTestTracePacket
BackendEvidenceVerificationPacket

Hot / warm / cold path

HOT:
per-build targeted regression

WARM:
function-card eval suites
search/source/writing quality checks

COLD:
large eval corpus
prompt optimization
fine-tuning datasets
batch regression runs

Privacy/data-egress boundary

eval data must use synthetic examples where required
real customer/company/search names must not be hardcoded in code/tests/fixtures
private data must not enter eval provider unless approved and scoped

Prompt-injection risks

eval examples may include adversarial source/tool/file content
eval grader output cannot override Selene acceptance law

Failure behavior

eval failure blocks completion where relevant
JD live failure beats passing evals
backend evidence mismatch blocks completion

Provider-off behavior

local tests still run
provider-assisted eval skipped/degraded if disabled
no live provider calls in normal tests

Old paths to classify/remove

vacuous tests
zero-test pass commands
mock-only product proof
old reports claiming unproven live behavior

First vertical slice

Create provider-first eval corpus skeleton for first vertical slice: “give me one line.”
Prove positive, paraphrase, and negative hijack cases.

Tests

unseen paraphrase
negative hijack
provider malformed output
protected fail-closed
Desktop no-brain proof
Adapter no-brain proof

JD live proof

JD live acceptance remains final product-facing validation for user-visible changes.

Backend evidence proof

EvalResultPacket
BackendEvidenceVerificationPacket
JDLiveTestTracePacket where live tested

Retirement condition for old paths

Old eval/proof language removed after new eval harness and final report checklist are in place and no unproven acceptance claims remain.

Card 10 — Desktop / iPhone Boundary Card

Function name

Desktop / iPhone Boundary

Coverage category

DISTRIBUTION_SURFACE
VOICE_RUNTIME
WRITE_PRESENTATION
SEARCH_EVIDENCE

Implementation status

DEFERRED_UNTIL_CORE_RUNTIME_STABLE
INITIAL_IMPLEMENTATION_ALLOWED only for proof/rendering changes explicitly required by a current slice

Canonical Selene owner

Desktop and iPhone client shells

Secondary Selene owners

Adapter transport
PH1.W / PH1.C / PH1.TTS / PH1.K / PH1.L
PH1.WRITE presentation packets
PH1.E source/image/file packets
Storage evidence

Forbidden owners

Desktop semantic intent
Desktop memory decisions
Desktop tool routing
Desktop provider calls
Desktop provider secrets
Desktop protected execution
Adapter semantic brain

OpenAI capability used

None directly in clients.
Clients render/transport provider-backed Selene packets only.

Selene-owned control

capture
playback
transport
rendering accepted runtime output
source chips
image cards
file cards
artifact cards
status display
current-app provenance proof

Provider interface

No direct provider interface in Desktop/iPhone.
Adapter transports provider-backed packets from runtime only.

Canonical packet

WriteOutputPacket
VoiceOutputPacket
SourceChipPacket
VisualEvidencePacket
GeneratedArtifactPacket
RealtimeVoiceEventPacket
ProviderStreamEventPacket only as approved runtime presentation state

Hot / warm / cold path

HOT:
mic capture
typed input
render answer
play TTS
show status

WARM:
source chips
image/file/artifact cards
memory UI
streaming progress

COLD:
history/search UI
settings/admin surfaces

Privacy/data-egress boundary

clients never hold provider secrets
clients do not call OpenAI directly
clients do not decide data-egress
clients only send captured input to approved adapter/runtime path

Prompt-injection risks

rendered content cannot become instructions
source/file/image cards cannot trigger execution by display alone

Failure behavior

stale app instance → proof invalid
multiple app/adapter owners → proof invalid
TTS playback failure → report accurate state
transport failure → degraded UI state

Provider-off behavior

show accurate degraded state
no client provider call
runtime remains source of truth

Old paths to classify/remove

Desktop semantic logic
Desktop memory/search/tool routing
Desktop TTS provider decision
Desktop identity decisions
stale app paths
old banner/status helpers if superseded
provider secrets in client

First vertical slice

Desktop renders WriteOutputPacket from PH1.WRITE and plays approved VoiceOutputPacket only.

Tests

one app instance
one adapter/runtime owner
current HEAD provenance
Desktop does not call provider
Desktop does not decide meaning
TTS playback evidence
source/image card render only approved packets

JD live proof

JD tests latest app only after Codex proves current app provenance.

Backend evidence proof

current HEAD
bundle path
process count
adapter owner
health/provenance
runtime packet refs
visible UI result

Retirement condition for old paths

Old Desktop/client paths removed after runtime-owned packet rendering and current-app live proof pass, with no semantic/client authority added.

Card 11 — Protected Simulation Execution Card

Function name

Protected Simulation Execution

Coverage category

CORE_RUNTIME
ADVISORY_ANALYSIS
ENTERPRISE_ADMIN

Implementation status

BUILD_ONLY_AFTER_JD_APPROVAL
NOT_INITIAL_IMPLEMENTATION for the provider-first pivot

Canonical Selene owner

SimulationExecutor
Access/Governance
protected business engines
Storage / Audit

Secondary Selene owners

PH1.X for protected-action classification
PH1.WRITE for refusal/confirmation wording
PH1.E for tool/evidence handoff
Voice ID as speaker evidence only

Forbidden owners

OpenAI execution authority
provider tool call direct execution
Desktop protected action
Adapter protected action
Voice ID as authority
memory as authority

OpenAI capability used

intent proposal
clarification wording
document summarisation
advisory analysis
tool proposal

Selene-owned control

simulation match
authority validation
confirmation
idempotency
deterministic process execution
audit ledger
fail-closed reason code
business mutation

Provider interface

SemanticInterpreterProvider as proposal only
ToolProposalProvider as proposal only
WritingProvider for wording only
CodeInterpreterProvider advisory only

Canonical packet

CurrentTurnInterpretationPacket
HumanConversationDirective
ToolProposalPacket
ToolExecutionDecisionPacket
SimulationExecutionPacket where repo owner defines it
AuditEvidencePacket where repo owner defines it
WriteOutputPacket

Hot / warm / cold path

HOT:
protected request classification and fail-closed

WARM:
clarification / confirmation flow
simulation lookup
access validation

COLD:
new simulation design
business workflow expansion
policy/audit review

Privacy/data-egress boundary

protected business data requires explicit data-egress approval before provider use
provider confidence cannot be authority
private/customer/company records must remain governed

Prompt-injection risks

user/provider/source says approve action
retrieved business document says bypass approval
tool output requests mutation
provider suggests execution path

Failure behavior

missing simulation → fail closed
missing authority → fail closed
missing confirmation → ask/stop
uncertain identity → fail closed
provider suggests action → no execution

Provider-off behavior

protected fail-closed still works
public explanation of payroll/rules may still work if allowed
no protected mutation

Old paths to classify/remove

adapter payroll classification helpers if wrong-owner
any protected shortcut path
business mutation outside SimulationExecutor
Voice ID authority shortcut
Desktop/Adapter protected action path

First vertical slice

User asks: Tell me about payroll. → public/advisory answer.
User asks: Approve payroll for Tim. → protected fail-closed unless simulation + authority exist.

Tests

public payroll explanation allowed
approve payroll fail-closed
salary change fail-closed
mixed search + salary change split
provider proposal cannot execute
Voice ID not authority

JD live proof

JD says: Organize payroll for Tim.
Expected: correct protected classification and fail-closed unless approved simulation/authority path exists.

Backend evidence proof

PH1.X protected classification
Simulation lookup result
Authority result
Audit/fail-closed evidence
PH1.WRITE protected wording

Retirement condition for old paths

Old protected/business shortcuts removed after canonical protected path proves public/protected split, simulation requirement, authority requirement, audit, and fail-closed behavior.

12. First Design Work Order

Do not implement all cards at once.

Before Codex writes or executes Build 0A / 0C, Codex must read AGENTS.md and the required architecture docs in the current run, prove clean tree, and perform existing-owner discovery.

Recommended next design/build sequence:

1. Confirm this architecture-card canvas.
2. Codex must read AGENTS.md before any build/design/repo work.
3. Codex must read docs/CORE_ARCHITECTURE.md.
4. Codex must read docs/SELENE_BUILD_EXECUTION_ORDER.md.
5. Codex must read docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md.
6. Codex must read relevant section docs for the specific engine/function.
7. Codex must prove clean tree before any new work.
8. Write Codex Build 0A / 0C instruction from the master plan.
9. Codex adds/uses docs master plan and discovers repo truth.
10. Codex performs provider coverage normalization.
11. Codex reports actual existing owners and current provider surfaces.
12. Update these cards from repo truth if needed.
13. Start implementation with Provider Governance Card only.
14. First implementation: provider registry + fake provider + evidence envelope.
15. First behavior vertical slice: “Can you give me one line?”

If Codex has not read AGENTS.md in the current run, the next lawful step is not implementation. The next lawful step is:

READ_AGENTS_AND_REQUIRED_ARCHITECTURE_DOCS_FIRST

13. Final Rule

Do not build OpenAI into Selene as a wrapper.
Do not create new duplicate engines.
Do not let Desktop or Adapter become brains.
Do not manually patch phrases.
Do not delete old paths before proof.

Build provider-first.
Keep canonical owners.
Use Selene packets.
Prove live.
Then remove old rubbish.
