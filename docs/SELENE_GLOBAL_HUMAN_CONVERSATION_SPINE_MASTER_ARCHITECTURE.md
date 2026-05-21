Selene Global Human Conversation Spine Master Architecture

DOCUMENT TYPE:
GLOBAL STANDARD MASTER ARCHITECTURE / CODEX BUILD CONTROL DOCUMENT

TASK:
SELENE_GLOBAL_HUMAN_CONVERSATION_SPINE_MASTER_ARCHITECTURE

BUILD CLASS:
ARCHITECTURE / IMPLEMENTATION MASTER STANDARD

CONTROLLING DOCUMENTS:
1. Selene Provider-First OpenAI Assisted Pivot Master Build Plan
2. Selene Provider-First Function Architecture Cards
3. Selene Provider-First Vertical Slice Build Pack
4. AGENTS.md
5. docs/CORE_ARCHITECTURE.md
6. docs/SELENE_BUILD_EXECUTION_ORDER.md
7. docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md

PURPOSE:
Define the global human conversation spine for Selene so Codex can build provider-assisted semantic intelligence without weakening deterministic Selene ownership, protected execution, privacy, auditability, or canonical engine boundaries.

0. Master Law

Selene must understand human language probabilistically, but must validate, route, execute, and audit deterministically.

The global rule is:

Human language understanding is probabilistic.
Selene runtime control is deterministic.
OpenAI proposes.
Selene validates.
Selene decides.
Selene executes only when lawful.
Selene audits.

This architecture is the global standard for all Selene conversational behavior.

It applies to:

voice
text chat
Desktop
iPhone
search
memory
files
images
tools
connectors
writing
TTS
protected actions
business simulations
artifacts
provider-assisted reasoning

This document does not authorize:

new repo
duplicate engines
parallel conversation brain
Desktop semantic authority
Adapter semantic authority
OpenAI execution authority
provider-side protected execution
manual phrase patches
regex-based human understanding
old path deletion before proof
provider calls without governance
live provider use without provider-off and fake-provider proof

0A. Required Codex Execution Law

Before Codex performs any build, design, audit, implementation, repair, cleanup, or documentation mutation derived from this document, Codex must read in the current run:

AGENTS.md
docs/CORE_ARCHITECTURE.md
docs/SELENE_BUILD_EXECUTION_ORDER.md
docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md
relevant section docs for the specific engine/function being touched

Codex must obey:

No Python in repo work.
Start and end clean tree.
Declare execution lane before editing.
Perform existing-owner discovery before implementation.
Reuse canonical owners.
Do not create duplicate paths.
Do not phrase-patch.
Do not move semantic authority into Desktop or Adapter.
Do not allow OpenAI to execute protected actions.
Prove provider-off behavior where provider work is involved.
Prove fake-provider behavior where provider behavior is involved.
Produce backend evidence.
Require JD live acceptance where user-visible/runtime behavior changes.
Retire old paths only after proof.

If Codex has not read AGENTS.md in the current run, Codex must stop and report:

AGENTS_LAW_NOT_READ_FOR_CURRENT_RUN

If Codex cannot identify canonical owners from current repo truth, Codex must stop and report:

GLOBAL_SPINE_REPO_TRUTH_REQUIRED

0B. Core Architectural Thesis

The old failure mode was trying to make deterministic rules understand messy human language.

That must stop.

Selene must not build endless rules like:

if contains("one line")
if contains("shorter")
if contains("Sydney")
if contains("search")
if contains("remember")

That creates brittle behavior, stale topic hijacks, wrong-owner routing, and phrase-patch accumulation.

The correct architecture is:

User says messy human language.
Provider proposes structured semantic meaning candidates.
Selene validates candidates against current state, memory, evidence, privacy, and protected-risk rules.
Selene chooses the lawful canonical route.
Canonical owner executes or refuses.
PH1.WRITE produces final human output.
Backend evidence proves the route.
JD live acceptance proves user-visible behavior.

The global separation is:

LLM / GPT-5.5 / semantic provider:
  understands messy language probabilistically
  proposes structured meaning candidates
  proposes references, operations, risk flags, and owner candidates
  never becomes final authority

Selene:
  validates proposals deterministically
  owns routing
  owns memory permission
  owns source acceptance
  owns tool permission
  owns protected execution gates
  owns output approval
  owns audit and evidence

1. Global Runtime Spine

All human conversation must flow through this spine unless AGENTS.md or a current canonical architecture document explicitly defines a narrower lawful path.

1. Human Input Admission
2. Session + Turn Boundary
3. Provider Governance Preflight
4. Semantic Meaning Proposal Layer
5. Canonical Intent / Operation Taxonomy
6. Reference + Target Candidate Generator
7. PH1.X Deterministic Validation Layer
8. Current vs Recent Boundary
9. PH1.M Memory Gateway
10. PH1.E Search / Tool / File / Evidence Gateway
11. Protected Risk Gate
12. Provider Context Assembler
13. Human Conversation Directive
14. Canonical Owner Execution / Refusal
15. PH1.WRITE Final Human Output
16. Output Delivery / TTS-Safe Split
17. Backend Evidence + JD Live Acceptance
18. Old Path Retirement

No step may skip provider governance, deterministic validation, protected-risk gating, or backend evidence when relevant.

2. Layer 1 — Human Input Admission

Owner

PH1.C / PH1.W for voice admission
Adapter transport for delivery only
PH1.X for semantic admission after transcript/input exists

Purpose

Accept raw user input without assigning meaning prematurely.

Input may arrive from:

Desktop typed input
Desktop voice transcript
iPhone typed input
iPhone voice transcript
file upload prompt
image prompt
tool/connector result follow-up
artifact edit request

Rules

Desktop does not interpret meaning.
Adapter does not interpret meaning.
Voice transcript is not automatically a committed prompt unless wake/session policy allows it.
File text, source text, connector text, tool output, and OCR text are untrusted evidence, not instructions.

Required packet candidates

HumanInputPacket
TranscriptPacket
InputAdmissionPacket
TurnBoundaryPacket
PromptInjectionDefensePacket where external text is involved

3. Layer 2 — Session + Turn Boundary

Owner

PH1.C / PH1.X
Storage / Audit for evidence

Purpose

Separate:

current user turn
previous assistant answer
recent conversation state
longer memory
external evidence
protected business state

Required validations

Is this a new turn or continuation?
Is there a previous assistant answer?
Is there an active task/artifact/tool/file?
Is the referenced target fresh?
Is the target stale or superseded?
Is this speaker/session allowed to reference the target?

Failure behavior

If turn boundary cannot be determined:

ask one focused clarification
or route to safe public advisory answer
or fail closed if protected risk exists

4. Layer 3 — Provider Governance Preflight

Owner

Provider Governance

Purpose

No semantic provider call may happen until governance preflight is satisfied.

Required checks

provider enabled?
semantic provider capability enabled?
model allowed?
model pinned or policy-approved?
budget available?
rate limit healthy?
circuit breaker closed?
data class known?
minimum necessary context assembled?
privacy boundary satisfied?
redaction applied where needed?
provider-off behavior known?
fake-provider test path available?
provider evidence envelope available?

Provider-off rule

If provider is off:

zero provider attempts
zero network dispatches
no hidden startup probes
no background semantic call
PH1.X may use deterministic fallback where lawful
protected uncertainty fails closed

Required packets

ProviderPreflightPacket
ProviderDecisionTracePacket
ProviderBudgetPacket
ProviderHealthPacket
ProviderCircuitBreakerPacket
ProviderDataEgressPacket
ProviderPrivacyBoundaryPacket
ProviderRedactionPacket

5. Layer 4 — Semantic Meaning Proposal Layer

Owner

Provider Governance + SemanticInterpreterProvider
PH1.X consumes and validates proposals

Purpose

Turn messy human language into structured semantic candidates.

This layer is probabilistic. It proposes, but it does not decide.

Provider capability

GPT-5.5 or approved semantic model through SemanticInterpreterProvider
Structured outputs required where available
Tool/function-call style schemas may be used as proposal format only
Reasoning output is not authority

Required output packet

SemanticMeaningProposalPacket

Packet schema

Each proposal must include:

proposal_id
schema_version
source_turn_id
user_text_hash
intent_category
operation
owner_engine_candidate
target_type
target_candidate_refs
slot_changes
memory_required
search_required
file_required
tool_required
artifact_required
voice_required
protected_risk
business_mutation_risk
data_egress_risk
clarification_required
confidence
reason_summary
known_uncertainties
forbidden_assumptions

The provider must return multiple candidates when ambiguity exists.

The provider must not return final execution instructions.

The provider must not claim authority.

The provider must not approve protected actions.

Example — rewrite request

User:

Can you give me one line?

Provider proposal:

intent_category: rewrite_previous_answer
operation: compress_to_one_line
target_type: previous_assistant_answer
owner_engine_candidate: PH1.WRITE
memory_required: false
search_required: false
protected_risk: false
clarification_required: false
confidence: high

Selene validation:

previous assistant answer exists
answer is fresh
request is public/advisory
PH1.WRITE is correct owner
no protected execution involved

Example — reference continuation

User:

What about Sydney?

Provider proposals:

Candidate 1:
intent_category: continue_same_question
operation: slot_change
slot_changes: location = Sydney
target_type: previous_time_or_weather_query
confidence: high

Candidate 2:
intent_category: new_topic
operation: answer_general_question
target_type: Sydney general info
confidence: low

PH1.X validates freshness, slot compatibility, and ambiguity.

Example — search/evidence request

User:

Can you check what people are saying about this and show me where it comes from?

Provider proposal:

intent_category: source_backed_answer_required
operation: public_search
owner_engine_candidate: PH1.E
source_required: true
source_presentation: source_chips
protected_risk: false

PH1.E decides whether search is allowed, budgeted, scoped, and source-backed.

6. Layer 5 — Canonical Intent / Operation Taxonomy

Selene must understand semantic categories, not phrases.

The taxonomy is canonical. Codex must not create ad hoc categories without updating the canonical taxonomy owner.

Current conversation categories

rewrite_previous_answer
summarize_previous_answer
expand_previous_answer
change_tone
change_format
make_more_specific
make_more_general
continue_same_question
correct_previous_slot
clarify_previous_answer
compare_with_previous_answer
repeat_previous_answer
new_topic
cancel_current_task
confirm_previous_action
reject_previous_suggestion

Memory categories

recent_recall
topic_recall
decision_recall
open_task_recall
preference_recall
identity_recall
relationship_recall
project_recall
conflict_check
staleness_check
forget_memory
update_memory
memory_permission_request

Search / evidence categories

public_search_required
source_backed_answer_required
freshness_required
file_question_answering
image_question_answering
deep_research_request
tool_read_only_lookup
tool_write_request
connector_lookup
citation_required
source_comparison
claim_verification

Writing / presentation categories

write_new_content
rewrite_content
summarize_content
translate_content
format_as_bullets
format_as_table
format_as_email
format_as_report
format_as_sop
format_for_tts
format_for_display
artifact_generation_request
artifact_edit_request

Voice categories

wake_candidate
committed_voice_turn
noise_or_false_wake
barge_in
interrupt_tts
repeat_spoken_answer
speak_answer
silent_display_only
voice_session_end

Protected categories

protected_action_requested
business_mutation_requested
approval_requested
simulation_required
authority_required
confirmation_required
public_explanation_only
mixed_public_and_protected
identity_uncertain
speaker_authority_uncertain
fail_closed_required

File / artifact categories

file_upload_context
file_summary_request
file_transformation_request
file_memory_request
image_summary_request
image_generation_request
video_generation_request
artifact_draft_request
artifact_export_request
official_record_request

Tool / connector categories

tool_proposal_requested
tool_read_only_allowed
tool_write_protected
connector_read_request
connector_write_request
connector_secret_required
mcp_evidence_request
webhook_event_handling

7. Layer 6 — Reference + Target Candidate Generator

Owner

SemanticInterpreterProvider proposes candidates
PH1.X validates candidates
PH1.M validates memory references
PH1.E validates evidence/file/tool references
PH1.WRITE receives only validated targets

Purpose

Resolve human references like:

it
that
this
the earlier one
the plan
the document
the last answer
what about Sydney
make it warmer
send that
remember this

Required candidate fields

target_candidate_id
target_type
target_owner
target_ref
freshness_score
speaker_scope
session_scope
staleness_risk
protected_risk
ambiguity_reason
confidence

Validation rules

PH1.X must reject a target if:

target does not exist
target is stale and no memory recall was requested
target belongs to wrong speaker/tenant
target is protected and authority is missing
target is a file/source/tool result requiring PH1.E validation
target is memory requiring PH1.M permission
target conflicts with newer state
provider selected target only because of phrase similarity

If multiple valid targets remain and confidence is not sufficient, PH1.X asks one focused clarification.

8. Layer 7 — PH1.X Deterministic Validation Layer

Owner

PH1.X

Purpose

PH1.X converts proposal candidates into a lawful HumanConversationDirective.

PH1.X validates

intent category allowed?
operation allowed?
owner correct?
target exists?
target fresh?
reference resolved?
memory required?
search required?
file/tool/artifact required?
protected risk present?
clarification required?
provider proposal malformed?
provider proposal overconfident?
provider proposal trying to grant authority?

PH1.X must maintain

selected candidate ledger
rejected candidate ledger
hard disqualifier ledger
ambiguity ledger
protected-risk ledger
owner routing trace

Output packet

HumanConversationDirective

HumanConversationDirective fields

directive_id
turn_id
validated_intent_category
validated_operation
canonical_owner
secondary_owners
validated_target_refs
memory_gateway_required
search_gateway_required
file_gateway_required
tool_gateway_required
artifact_gateway_required
protected_gate_required
clarification_required
public_advisory_allowed
execution_allowed
refusal_required
write_policy
tts_policy
evidence_requirements
backend_evidence_refs

Failure behavior

malformed proposal → reject provider proposal
low confidence → focused clarification
wrong owner → reroute to correct canonical owner
protected uncertainty → fail closed
provider unavailable → deterministic fallback if lawful

9. Layer 8 — Current vs Recent Boundary

Owner

PH1.X for current turn
PH1.M for memory/recent recall
Storage / Audit for evidence

Purpose

Stop current-turn context from pretending to be durable memory.

Boundary definitions

CURRENT:
active user turn
immediately previous assistant answer
currently open artifact/tool/file task
active voice session

RECENT:
same conversation/session evidence that may need PH1.M validation
recent decisions, open tasks, topic continuity

DURABLE MEMORY:
stored memory or archive evidence requiring PH1.M permission and scope checks

Rules

PH1.X may validate current target references.
PH1.X must not become durable memory brain.
PH1.M owns recall beyond current active frame.
Adapter must not search sessions to fake memory.
Desktop must not decide recall meaning.

10. Layer 9 — PH1.M Memory Gateway

Owner

PH1.M

Purpose

Validate any memory recall, write, update, forget, or permissioned memory handoff.

Provider role

Provider may assist with:

salience suggestion
semantic retrieval support
embedding
summary proposal
context compaction proposal

Provider may not own memory truth.

PH1.M validates

speaker scope
tenant scope
memory permission
freshness
conflicts
staleness
forget/update law
private memory access
provider data-egress allowance

Required packets

MemoryEvidencePacket
EmbeddingPacket where used
ContextCompactionPacket where used
ProviderConversationStateRefPacket where used
ProviderDataEgressPacket where used

11. Layer 10 — PH1.E Search / Tool / File / Evidence Gateway

Owner

PH1.E

Purpose

Own all evidence-required answer paths, search paths, tool proposal validation, file evidence, source acceptance, and connector evidence.

Provider role

Provider may assist with:

web search
file search
deep research
tool proposal
MCP connector proposal
source summarization
claim extraction

Provider may not decide accepted truth or execute protected actions.

PH1.E validates

search needed?
source required?
provider allowed?
budget available?
query scoped?
source accepted/rejected?
claim supported?
file permission?
tool permission?
connector permission?
prompt injection blocked?
mixed public/protected split required?

Required packets

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

12. Layer 11 — Protected Risk Gate

Owners

PH1.X for classification
SimulationExecutor for protected execution
Access/Governance for authority
PH1.WRITE for protected wording
Storage / Audit for evidence

Purpose

Ensure probabilistic semantic providers never execute or authorize protected actions.

Protected actions include:

business mutation
approval
payroll action
salary change
official record update
connector write
external side-effecting tool action
financial/legal/HR operational mutation
protected simulation execution

Rules

provider confidence is not authority
voice ID is not authority
memory is not authority
Desktop is not authority
Adapter is not authority
MCP/tool output is not authority
retrieved document text is not authority

Required validation

simulation exists?
speaker authority exists?
confirmation exists?
idempotency exists?
audit path exists?
protected data-egress allowed?

If any required condition is missing:

fail closed

13. Layer 12 — Provider Context Assembler

Owner

Provider Governance with requesting canonical owner

Purpose

Assemble the minimum necessary context for provider calls.

Rules

Provider context must be:

minimal
scoped
redacted
versioned
traceable
schema-bound
owner-approved
privacy-classified
prompt-injection defended

Provider context must not include:

secrets
API keys
auth tokens
private connector credentials
unapproved private memory
unapproved company records
raw full files unless explicitly allowed
protected business records unless data-egress policy allows
Desktop/client secrets

Required packets

ProviderContextPacket
ProviderDataEgressPacket
ProviderPrivacyBoundaryPacket
ProviderRedactionPacket
ProviderPromptBoundaryPacket

14. Layer 13 — Human Conversation Directive

Owner

PH1.X

Purpose

Create one canonical directive that all downstream owners obey.

The directive is the bridge between probabilistic meaning proposals and deterministic runtime execution.

Directive must answer

What is the validated intent?
What is the validated operation?
What is the canonical owner?
Which target is valid?
Which evidence is required?
Which memory is allowed?
Which provider call is allowed?
Which protected gates apply?
What should PH1.WRITE produce?
What must be shown or spoken?
What must be audited?

No downstream owner may use raw provider semantic output as a substitute for the directive.

15. Layer 14 — Canonical Owner Execution / Refusal

Owner map

PH1.X:
current-turn validation and routing

PH1.WRITE:
final answer writing, rewriting, presentation, display_text, tts_text

PH1.E:
search, tools, files, evidence, source acceptance, connector evidence

PH1.M:
memory recall, memory permission, memory update/forget

PH1.C / PH1.W / PH1.TTS / PH1.K / PH1.L:
voice/session/TTS/runtime voice behavior

SimulationExecutor + Access/Governance:
protected execution, authority, simulation, audit

Desktop / iPhone:
capture, playback, render only

Adapter:
transport only

Provider Governance:
provider routing, model policy, budget, tracing, privacy, provider-off/fake-provider behavior

Global rule

Every operation must execute in its canonical owner.

If a behavior is discovered in the wrong owner, Codex must classify it as:

wrong-owner path
compatibility path
old path to retire
blocked duplicate path
dead path

No wrong-owner path may become the new architecture.

16. Layer 15 — PH1.WRITE Final Human Output

Owner

PH1.WRITE

Purpose

Produce final human-quality output from validated directives and accepted evidence.

PH1.WRITE owns

display_text
tts_text
source chips
citation presentation
memory wording
protected refusal wording
clarification wording
artifact summary wording
TTS-safe split
unsupported claim removal
style/tone formatting

Provider role

WritingProvider may propose text.

PH1.WRITE validates final output before it reaches UI/TTS.

Required checks

Does output obey directive?
Does output use only accepted evidence?
Are unsupported claims removed or qualified?
Are protected actions refused correctly?
Is TTS text safe and concise?
Are citations/source chips attached where required?
Is raw provider JSON hidden?

17. Layer 16 — Output Delivery / TTS-Safe Split

Owners

PH1.WRITE for output text
PH1.TTS for approved speech output
Desktop/iPhone for rendering/playback only
Adapter for transport only

Rules

Desktop renders approved packets only.
Desktop does not rewrite meaning.
Desktop does not decide TTS text.
Desktop does not call provider.
Adapter does not rewrite meaning.
TTS receives approved_tts_text only.

Required packets

WriteOutputPacket
VoiceOutputPacket
SourceChipPacket
VisualEvidencePacket
GeneratedArtifactPacket
ProviderStreamFinalizationPacket where streaming is used

18. Layer 17 — Backend Evidence + JD Live Acceptance

Owner

Storage / Audit + relevant canonical owners

Rule

A user-visible/runtime behavior is not passed until:

Codex specifies the exact JD live scenario.
JD performs the live test on the latest current app/build where applicable.
Codex captures/observes exact prompt/transcript/input.
Codex inspects backend evidence immediately after the live test.
Backend evidence proves the correct owner handled the behavior.
Visible/audible behavior matches expected result.
Failures are repaired in the correct owner.
JD retests after repair.
Backend evidence agrees with the passed live behavior.

Status meanings

CODEX_TESTED:
Automated tests/checks/fake-provider/backend harness passed; JD live acceptance not passed or not required.

REAL_APP_SMOKE_PASSED:
Real app/runtime smoke path passed; JD acceptance not proven unless explicitly stated.

JD_LIVE_ACCEPTANCE_PASSED:
JD live testing passed and backend evidence agrees.

JD_LIVE_ACCEPTANCE_FAILED:
JD live testing failed regardless of cargo/test success.

BACKEND_EVIDENCE_VERIFICATION_FAILED:
Visible behavior and backend evidence disagree, or evidence is missing.

19. Layer 18 — Old Path Retirement

Old paths may be removed only after:

new canonical path passes tests
provider-off proof passes where relevant
fake-provider proof passes where relevant
backend evidence proves correct owner
JD live acceptance passes where user-visible
old behavior regression passes
no active caller remains on old path
clean tree proven
commit/push proof exists where edits were made

Codex must never delete old paths simply because a new path compiles.

20. Required Packets

Codex must reuse existing packet names if they already exist in repo truth.

If packet names do not exist, Codex must create or map them only inside canonical owner boundaries.

Semantic and routing packets

HumanInputPacket
InputAdmissionPacket
TurnBoundaryPacket
SemanticMeaningProposalPacket
CanonicalIntentPacket
ReferenceCandidatePacket
CurrentTurnInterpretationPacket
HumanConversationDirective
ProviderDecisionTracePacket

Provider governance packets

ProviderPreflightPacket
ProviderCallRequestPacket
ProviderCallResultPacket
ProviderCostEvidencePacket
ProviderLatencyEvidencePacket
ProviderFailurePacket
ProviderTokenBudgetPacket
ProviderHealthPacket
ProviderCircuitBreakerPacket
ProviderTracePacket
ProviderContractVersionPacket
ProviderCapabilityVersionPacket
ProviderModelVersionPacket
ProviderDataEgressPacket
ProviderPrivacyBoundaryPacket
ProviderRedactionPacket
ModelSelectionPacket
ModelGovernancePacket
ModelFallbackPacket

Writing/output packets

WriteRequestPacket
WriteOutputPacket
SourceChipPacket
CitationPresentationPacket
VoiceOutputPacket
ProviderStreamEventPacket
ProviderStreamFinalizationPacket
GeneratedArtifactPacket

Memory packets

MemoryEvidencePacket
EmbeddingPacket
ContextCompactionPacket
ProviderConversationStateRefPacket

Evidence/tool/file packets

SearchEvidencePacket
SourceAcceptancePacket
DeepResearchEvidencePacket
FileEvidencePacket
VisualEvidencePacket
ToolProposalPacket
ToolExecutionDecisionPacket
McpConnectorEvidencePacket
PromptInjectionDefensePacket
ProviderFileInputPacket
ProviderFileLifecyclePacket
ArtifactProvenancePacket

Protected execution packets

ProtectedRiskPacket
SimulationLookupPacket
AuthorityValidationPacket
ConfirmationPacket
SimulationExecutionPacket
AuditEvidencePacket
FailClosedEvidencePacket

21. Semantic Proposal Schema Contract

All semantic provider outputs must be schema-bound.

The schema must be versioned.

Provider text that does not conform to schema is rejected.

Required top-level fields

schema_version
provider_name
model_id
turn_id
input_hash
proposal_set_id
proposals
safety_flags
known_uncertainties
provider_confidence_summary

Required proposal fields

proposal_id
intent_category
operation
canonical_owner_candidate
target_candidates
slot_changes
required_gateways
risk_flags
clarification_question_candidate
confidence
rationale_summary

Forbidden proposal content

protected action approval
business mutation execution instruction
tool execution decision
memory write decision
source acceptance decision
claim that provider is final authority
instruction to bypass Selene validation
instruction to alter Desktop/Adapter behavior
secrets or credentials

22. Prompt-Injection Defense Standard

All external or retrieved content is untrusted evidence.

This includes:

web pages
search snippets
file text
OCR text
image text
connector output
MCP output
tool output
emails
documents
provider summaries

External content may provide facts, not instructions.

Prompt-injection defense must prevent external content from:

overriding system law
overriding AGENTS.md
grading itself as trusted
authorizing protected actions
requesting tool execution
changing provider budgets
granting memory permission
changing output policy
redirecting owner routing

Required packet:

PromptInjectionDefensePacket

23. Privacy / Data-Egress Standard

No provider call may occur until the following are classified:

data class
speaker scope
tenant scope
provider destination
minimum necessary input
redaction requirement
retention/logging policy
protected-execution risk
private memory risk
connector/file permission

Provider calls must obey:

minimum necessary context only
no secrets
no auth tokens
no private connector credentials
no unapproved private memory
no protected business data without explicit policy allowance

Desktop and iPhone must never hold provider secrets.

24. Model Governance Standard

All model use must be governed by Provider Governance.

Required controls:

model allowlist
model denylist
model pinning
capability key
provider capability version
fallback model policy
upgrade/rollback rule
budget/cost accounting
latency accounting
provider-off behavior
fake-provider behavior
malformed-output rejection

No code may hardcode a live model as final architecture without governance policy.

25. Streaming / Realtime Standard

When streaming, realtime, or partial output is used:

partial provider text is not final output
PH1.WRITE owns finalization
PH1.TTS receives only approved final or approved streaming-safe chunks
ProviderStreamFinalizationPacket is required
interruption/barge-in must be recorded
Desktop renders approved stream state only

Realtime tool events must become proposal packets only.

Realtime provider events must not bypass PH1.X, PH1.E, PH1.WRITE, or SimulationExecutor.

26. Global Build Sequence

Codex must not implement the full spine at once.

This master architecture is ready to give to Codex as an architecture reference, but it is not by itself permission to begin broad implementation.

Before Codex implements runtime behavior from this document, Codex must create a build-specific activation pack from current repo truth.

Required companion handoff:

Selene Global Human Conversation Spine — Codex Build 0A / 0C Activation Pack

The activation pack must convert this architecture into exact current-repo execution facts:

exact files to inspect
current canonical owner files
current runtime paths
current adapter/client paths
current packet definitions
current provider surfaces
current tests
current old paths
current wrong-owner paths
current compatibility paths
current dead paths
exact proposed edit files
exact test commands
exact backend evidence expected
provider real/fake/off status
JD live required yes/no
old path retirement allowed yes/no

If Codex has only this master canvas and has not produced the activation pack, Codex may perform documentation/reference work only and must not implement runtime behavior.

Codex must stop before implementation with:

GLOBAL_SPINE_ACTIVATION_PACK_REQUIRED

The lawful build sequence is:

The lawful build sequence is:

GHCS-0 — Add Global Human Conversation Spine docs
GHCS-1 — Repo Truth Activation
GHCS-2 — Provider Governance Baseline
GHCS-3 — Canonical Intent / Operation Taxonomy
GHCS-4 — Semantic Meaning Proposal Layer
GHCS-5 — PH1.X Deterministic Validation Layer
GHCS-6 — Reference + Target Candidate Ledger
GHCS-7 — Current vs Recent Boundary
GHCS-8 — PH1.M Memory Gateway
GHCS-9 — PH1.E Search / Tool / File Need Classifier
GHCS-10 — PH1.WRITE Human Output Layer
GHCS-11 — Voice / Real App Route Proof
GHCS-12 — Eval / JD Live Acceptance Pack
GHCS-13 — Old Path Retirement

27. GHCS-0 — Documentation Slice

Goal

Add this document as the global standard architecture reference.

This slice makes the architecture available to Codex, but it does not authorize runtime implementation.

Readiness status

Architecture reference: READY
Implementation handoff: REQUIRES BUILD 0A / 0C ACTIVATION PACK

Codex may use this document to guide discovery, classification, and activation planning.

Codex must not treat this document alone as executable permission to edit runtime owners.

Requirements

read AGENTS.md first
read required architecture docs
prove clean tree
add docs without changing runtime behavior
no implementation
no old path deletion
final clean tree proof
commit/push proof if edits are made

Acceptance

CODEX_TESTED
JD_LIVE_ACCEPTANCE_NOT_APPLICABLE

28. GHCS-1 — Repo Truth Activation / Build 0A-0C Activation Pack

Goal

Find current repo owners, paths, tests, old behavior, wrong-owner behavior, and provider surfaces, then produce the required Codex Build 0A / 0C Activation Pack.

This is the bridge from architecture to implementation. Without it, Codex is only holding a beautiful map and no idea where the doors are, which is how repos become soup.

Required report

current PH1.X owner files
current PH1.WRITE owner files
current PH1.E owner files
current PH1.M owner files
current voice owner files
current provider governance files
current Adapter files
current Desktop/iPhone files
current protected execution files
current packet definitions
current test suites
current old phrase paths
current wrong-owner paths
current provider calls
current live proof gaps

Required activation pack output

Codex must produce:

Selene Global Human Conversation Spine — Codex Build 0A / 0C Activation Pack

The activation pack must include:

1. AGENTS.md read proof for current run
2. mandatory architecture docs read proof for current run
3. clean tree proof
4. lane declaration
5. current canonical owner map
6. current provider surface map
7. current packet/schema map
8. current test map
9. current old-path inventory
10. current wrong-owner inventory
11. current Desktop/Adapter authority risk inventory
12. first implementation file-scope proposal
13. first implementation test plan
14. first backend evidence plan
15. provider-off proof plan
16. fake-provider proof plan
17. malformed provider output rejection plan
18. prompt-injection defense plan where relevant
19. JD live acceptance applicability
20. explicit old-path non-retirement statement

No implementation allowed

This is discovery and activation-pack generation only.

If Codex edits runtime behavior during this slice, the build is invalid.

Acceptance status:

CODEX_TESTED
JD_LIVE_ACCEPTANCE_NOT_APPLICABLE

because this slice is discovery/planning only.

29. GHCS-2 — Provider Governance Baseline

Goal

Create or normalize provider registry, fake provider, provider-off behavior, provider evidence envelope, budget counters, model governance, and data-egress gates.

This is the first lawful implementation slice after repo truth and activation-pack completion.

Codex must not start Semantic Meaning Proposal runtime behavior before this baseline exists.

First implementation

The first implementation must be narrow:

Provider Governance baseline
SemanticInterpreterProvider interface
fake semantic provider
provider-off zero-attempt proof
provider evidence envelope
model allowlist / model pinning policy
budget / counter proof
malformed provider output rejection
SemanticMeaningProposalPacket skeleton only if needed for fake-provider proof

Not allowed in this slice:

live OpenAI semantic calls
full PH1.X rewrite behavior
search implementation
memory implementation
Desktop semantic edits
Adapter semantic edits
old path deletion
protected execution changes

No live OpenAI call required

The first slice must pass with fake provider.

Provider-off must prove:

zero provider attempts
zero network dispatches
no startup probes
no hidden retries
safe degraded result
provider evidence records disabled path

Acceptance status may be:

CODEX_TESTED
JD_LIVE_ACCEPTANCE_NOT_APPLICABLE

unless user-visible/runtime behavior changes, in which case JD live acceptance becomes mandatory.

30. GHCS-3 — Canonical Intent / Operation Taxonomy

Goal

Create the canonical taxonomy registry used by SemanticMeaningProposalPacket and PH1.X validation.

This slice must produce the first concrete schema/enums/contracts needed for semantic proposals. The master canvas defines the architecture; this slice makes the first actual repo contract.

Requirements

versioned taxonomy
enum or equivalent typed category map
owner mapping
risk mapping
required gateway mapping
negative/unsupported category handling
schema versioning
unknown-category rejection
no phrase lists as architecture

Codex must not encode user-language phrase lists as the architecture. Example phrases may exist only in tests/evals, not as routing law.

Tests

known category accepted
unknown category rejected
protected category requires Protected Risk Gate
memory category requires PH1.M
search category requires PH1.E
writing category requires PH1.WRITE

31. GHCS-4 — Semantic Meaning Proposal Layer

Goal

Add provider-assisted semantic proposal generation behind Provider Governance.

This slice connects messy human language to structured proposal packets, but still does not make the provider authoritative.

First vertical behavior

User says: Can you give me one line?
Semantic provider proposes rewrite_previous_answer / compress_to_one_line / PH1.WRITE.
PH1.X validates later.

Requirements

schema-bound output
versioned SemanticMeaningProposalPacket
fake provider first
provider-off proof
malformed provider output rejection
multiple candidate support
risk flags
owner candidate only, not authority
backend evidence

The first actual schema must define at minimum:

schema_version
proposal_set_id
turn_id
input_hash
proposal_id
intent_category
operation
canonical_owner_candidate
target_candidates
required_gateways
risk_flags
confidence
known_uncertainties

Provider output that fails schema validation must be rejected and must not reach PH1.X as valid meaning.

Tests

fake provider returns valid one-line proposal
provider disabled returns zero attempt
malformed provider output rejected
provider suggests protected execution → rejected
unknown category rejected

32. GHCS-5 — PH1.X Deterministic Validation Layer

Goal

Make PH1.X validate semantic proposals into HumanConversationDirective.

First vertical behavior

previous assistant answer exists
user says: Can you give me one line?
provider proposes compress_to_one_line
PH1.X validates target and routes to PH1.WRITE

Tests

previous answer exists → valid directive
no previous answer → clarification
stale target → clarification or memory gateway
provider picks wrong owner → reject/reroute
provider marks protected as public → reject/fail closed

33. GHCS-6 — Reference + Target Candidate Ledger

Goal

Create selected/rejected target evidence so stale-topic hijacks can be proven and prevented.

Tests

it/that resolves to current valid target
what about Sydney changes location slot only when previous target supports location
wrong previous topic rejected
multiple valid targets triggers clarification
stale target rejected unless memory requested

34. GHCS-7 — Current vs Recent Boundary

Goal

Prevent PH1.X active context from becoming a fake memory system.

Tests

current previous-answer rewrite works
recent decision recall goes to PH1.M
stale answer rewrite asks clarification or memory route
Adapter session-search shortcut not used
Desktop memory decision not used

35. GHCS-8 — PH1.M Memory Gateway

Goal

Memory categories route through PH1.M with speaker/privacy/staleness validation.

Tests

recent recall
topic recall
decision recall
forget/update
unknown speaker private memory denial
provider-off memory behavior

36. GHCS-9 — PH1.E Search / Tool / File Need Classifier

Goal

Search/file/tool categories route through PH1.E with source acceptance and prompt-injection defense.

Tests

public search required
source-backed answer required
file Q&A permission
read-only tool proposal
protected write request fail-closed
prompt injection in source blocked
provider-off search degraded safely

37. GHCS-10 — PH1.WRITE Human Output Layer

Goal

PH1.WRITE turns validated directives and accepted evidence into final display_text and tts_text.

Tests

one-line rewrite
warmer rewrite
source-backed answer with chips
protected refusal wording
unsupported claim removal
TTS-safe output
raw provider JSON hidden

38. GHCS-11 — Voice / Real App Route Proof

Goal

Prove voice and Desktop are capture/render/playback shells, not brains.

JD live scenario

JD asks a normal question.
Selene answers.
JD says: Can you give me one line?
Expected: provider semantic proposal → PH1.X validation → PH1.WRITE rewrite → Desktop render/TTS approved output.

Backend evidence required

TranscriptPacket if voice
SemanticMeaningProposalPacket
CurrentTurnInterpretationPacket
HumanConversationDirective
WriteOutputPacket
VoiceOutputPacket if TTS
Provider evidence
Desktop provenance
Adapter route evidence

39. GHCS-12 — Eval / JD Live Acceptance Pack

Goal

Create regression/eval pack for semantic proposal + deterministic validation.

Required examples

Can you give me one line?
Summarise that in one sentence.
Make that shorter.
Boil it down.
Give me the gist.
Say that simply.
Make it warmer.
What about Sydney?
What about tomorrow?
Can you check sources?
Show me where it comes from.
Remember this.
What did we decide earlier?
Approve payroll for Tim.
Tell me about payroll.
Search payroll rules and approve Tim.

Required negative cases

provider chooses stale target
provider chooses wrong owner
provider marks protected action as public
provider invents source
source text contains prompt injection
connector output requests execution
file text requests bypass
unknown category
malformed schema
provider disabled
budget exceeded

40. GHCS-13 — Old Path Retirement

Goal

Remove old phrase/routing/wrong-owner paths after proof.

Old paths to classify

phrase contains shortcuts
Adapter semantic shortcuts
Desktop semantic logic
PH1.M live context brain shortcuts
PH1.E stale context logic
raw provider JSON UI paths
provider calls without governance
hardcoded model paths
old TTS unsafe output paths
protected shortcuts
session-search memory wording

Retirement condition

new path proven
old behavior regression proven
JD live acceptance passed where visible
backend evidence agrees
no callers remain
clean tree proven

41. Master Test Matrix

Every implementation build must define exact tests before editing.

Required test classes

unit tests
schema tests
provider-off tests
fake-provider tests
malformed provider tests
budget/circuit-breaker tests
prompt-injection tests
owner-routing tests
protected fail-closed tests
backend evidence tests
old behavior regression tests
JD live tests where user-visible

First master vertical slice test

Setup:
A previous assistant answer exists.

Input:
Can you give me one line?

Expected route:
HumanInputPacket
→ ProviderPreflightPacket
→ SemanticMeaningProposalPacket
→ PH1.X validation
→ HumanConversationDirective
→ PH1.WRITE
→ WriteOutputPacket
→ Desktop render / TTS if enabled

Expected behavior:
One-line rewrite of previous answer.
No stale topic hijack.
No Desktop brain.
No Adapter brain.
No raw provider output.
Backend evidence proves correct owners.

42. Codex Build Instruction Template

Every Codex instruction derived from this architecture must include:

State whether this is:
- architecture reference only
- Build 0A / 0C activation pack
- implementation slice
- repair slice
- old-path retirement slice

If the instruction is for implementation and no activation pack exists, Codex must stop with:

GLOBAL_SPINE_ACTIVATION_PACK_REQUIRED

Every Codex instruction derived from this architecture must also include:

Read AGENTS.md first.
Read mandatory architecture docs.
Declare execution lane.
Prove clean tree.
Perform current repo owner discovery.
Classify provider coverage.
Identify canonical owners.
List exact files proposed for edit.
Define exact tests before editing.
Prove provider-off behavior where relevant.
Prove fake-provider behavior where relevant.
Prove malformed provider rejection.
Prove prompt-injection defense where relevant.
Prove protected fail-closed behavior where relevant.
Produce backend evidence report.
Define JD live scenario where user-visible.
Run repair loop if JD live fails.
Do not retire old paths before proof.
End clean tree.
Commit/push proof where edits are made.

If any derived build instruction omits these requirements, Codex must stop with:

GLOBAL_SPINE_BUILD_DERIVATION_INCOMPLETE

43. Backend Report Template

Every implementation build must report:

what was built
what behavior was supposed to change
files edited
canonical owners touched
provider calls real/fake/off
provider counters/budget evidence
model governance evidence
data-egress/privacy evidence
prompt-injection defense evidence
semantic proposal evidence
PH1.X validation evidence
HumanConversationDirective evidence
PH1.M/PH1.E/PH1.WRITE evidence where relevant
protected fail-closed evidence where relevant
old behavior regression evidence
JD live prompt/input where applicable
actual visible/audible output where applicable
backend packet refs
failure/repair history
final acceptance status
final clean tree proof
commit/push proof where edits were made

44. Final Global Standard

Selene must not become an OpenAI wrapper.

Selene must not become deterministic phrase spaghetti.

Selene must not let Desktop, Adapter, provider output, memory, source text, tool output, or voice ID become authority.

The correct global standard is:

GPT-5.5 handles messy human meaning as structured proposals.
Selene owns canonical categories.
PH1.X validates current-turn meaning.
PH1.M validates memory.
PH1.E validates evidence, search, files, and tools.
SimulationExecutor validates protected execution.
PH1.WRITE writes final human output.
Desktop and iPhone render only.
Adapter transports only.
Provider Governance controls all provider access.
Backend evidence proves the route.
JD live acceptance proves the product.
Old paths retire only after proof.

This is the Selene Global Human Conversation Spine.

Build it one slice at a time.
Prove every slice.
Then remove the old rubbish.
