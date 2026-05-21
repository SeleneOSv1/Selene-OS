Selene Identity + Access + Authority Spine Master Architecture

DOCUMENT TYPE:
GLOBAL STANDARD MASTER ARCHITECTURE / CODEX BUILD CONTROL DOCUMENT

TASK:
SELENE_IDENTITY_ACCESS_AUTHORITY_SPINE_MASTER_ARCHITECTURE

BUILD CLASS:
ARCHITECTURE / IMPLEMENTATION MASTER STANDARD

CONTROLLING DOCUMENTS:
1. AGENTS.md
2. Selene Provider-First OpenAI Assisted Pivot Master Build Plan
3. Selene Provider-First Function Architecture Cards
4. Selene Provider-First Vertical Slice Build Pack
5. Selene Global Human Conversation Spine Master Architecture
6. docs/CORE_ARCHITECTURE.md
7. docs/SELENE_BUILD_EXECUTION_ORDER.md
8. docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md

PURPOSE:
Define the Identity + Access + Authority Spine for Selene so Codex can connect Wake, Session, Voice ID, user access, memory scope, tool scope, protected authority, simulation gating, and audit without creating duplicate engines, wrong-owner shortcuts, Desktop authority, Adapter authority, provider authority, or phrase-patch security behavior.

0. Master Law

Selene must separate activation, identity, access, authority, protected execution, memory scope, tool scope, and semantic meaning.

The global law is:

Wake decides whether Selene should listen.
Session decides where the turn belongs.
Voice ID produces speaker evidence.
Access resolves what the speaker may see or use.
Authority decides whether the speaker may perform the requested action now.
Simulation proves whether a protected process can execute.
GPT-5.5 proposes what the user appears to want.
Selene validates, routes, executes only when lawful, and audits.

These must never be collapsed into one another.

Forbidden equivalences:

Wake ≠ identity
Voice ID ≠ access
Access ≠ authority
Authority ≠ simulation
Simulation ≠ semantic understanding
GPT-5.5 ≠ permission system
Desktop ≠ identity system
Adapter ≠ authority system
Memory ≠ identity proof
Provider confidence ≠ authority

This spine is a permission and safety spine. It interlocks with the Global Human Conversation Spine, which is the semantic meaning spine.

Together:

Global Human Conversation Spine answers:
What does the user mean?

Identity + Access + Authority Spine answers:
Who might be speaking, what are they allowed to access, and are they allowed to do this action now?

0A. Codex Execution Law

Before Codex performs any build, design, audit, implementation, repair, cleanup, or documentation mutation derived from this document, Codex must read in the current run:

AGENTS.md
docs/CORE_ARCHITECTURE.md
docs/SELENE_BUILD_EXECUTION_ORDER.md
docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md
Selene Global Human Conversation Spine Master Architecture
relevant section docs for Wake, Session, Voice ID, Access, Authority, Simulation, Memory, Tools, Search, Desktop, Adapter, and PH1.X when touched

Codex must obey AGENTS.md authority order.

Codex must declare lane before editing:

current project phase
selected lane
simulation required yes/no
authority required yes/no
state mutation allowed yes/no
protected execution allowed yes/no
provider degradation allowed yes/no
normal answer allowed yes/no
fail-closed required yes/no and for which part

If Codex has not read AGENTS.md in the current run, Codex must stop and report:

AGENTS_LAW_NOT_READ_FOR_CURRENT_RUN

If Codex cannot identify existing canonical owners from current repo truth, Codex must stop and report:

IAA_REPO_TRUTH_REQUIRED

If Codex tries to implement this spine without first producing repo-truth activation for current Wake, Session, Voice ID, Access, Authority, Simulation, Desktop, Adapter, PH1.X, PH1.M, and PH1.E surfaces, Codex must stop and report:

IAA_ACTIVATION_PACK_REQUIRED

0B. Core Thesis

Wake alone is not enough.

Voice ID alone is not enough.

Access alone is not enough.

Authority alone is not enough.

A protected action cannot execute merely because a voice was heard, a speaker was guessed, a user record exists, or GPT-5.5 understood the sentence.

Correct security posture:

Unknown or weak identity → public-safe only.
Unresolved access → private memory denied.
Unresolved authority → protected execution denied.
Missing simulation → protected execution denied.
Provider uncertainty → never grants permission.
Desktop/Adapter uncertainty → never grants permission.

The default for protected behavior is:

protected_action_allowed = false
private_memory_allowed = false
company_memory_allowed = false
tool_write_allowed = false
business_mutation_allowed = false

Permissions must be added by deterministic proof, not inferred by conversational confidence.

1. Interlocked Runtime Flow

All voice or identity-relevant human interaction must flow through this spine when applicable:

1. Wake / Activation Boundary
2. Session Open or Resume
3. Transcript Admission
4. Voice ID Evidence Capture
5. Liveness / Anti-Replay Evidence
6. Speaker Identity Evidence Packet
7. Access Instance Resolver
8. Access Scope Packet
9. Memory Scope Gate
10. Tool / Search / File Scope Gate
11. Semantic Meaning Proposal Layer
12. PH1.X Deterministic Validation
13. Risk Classification
14. Authority Gate
15. Simulation Gate
16. Canonical Owner Execution / Refusal
17. PH1.WRITE Final Human Output
18. Desktop / Adapter No-Authority Delivery
19. Protected Execution Audit
20. JD Live Acceptance + Backend Evidence
21. Old Path Cleanup

The semantic spine and permission spine meet at:

PH1.X
PH1.M
PH1.E
SimulationExecutor
PH1.WRITE
Storage / Audit

2. Owner Map

Codex must discover current repo owners before implementation. The logical ownership model is:

Wake / Activation:
  Wake engine / PH1.W / PH1.L as current repo truth defines

Session:
  PH1.L / PH1.C / session runtime as current repo truth defines

Transcript Admission:
  PH1.C plus Adapter transport only

Voice ID:
  Voice ID / speaker identity evidence engine as current repo truth defines

Liveness:
  Voice ID / anti-replay / capture evidence owner as current repo truth defines

Access Resolver:
  Access / user / tenant / role / permission owner as current repo truth defines

Memory Scope:
  PH1.M

Tool/Search/File Scope:
  PH1.E

Semantic Meaning:
  SemanticInterpreterProvider through Provider Governance; PH1.X validates

Current-Turn Validation:
  PH1.X

Protected Risk:
  PH1.X plus Protected Risk Gate

Authority:
  Access/Governance/Authority owner as current repo truth defines

Simulation:
  SimulationExecutor

Final Output:
  PH1.WRITE

Desktop / iPhone:
  capture, playback, transport, render only

Adapter:
  transport only

Audit:
  Storage / Audit owner as current repo truth defines

No owner may absorb another owner’s authority for convenience.

3. What GPT-5.5 Does in This Spine

GPT-5.5 is used only as a probabilistic semantic interpreter and optional writing assistant behind Provider Governance.

GPT-5.5 may propose:

what the user appears to want
intent category
operation
target candidates
risk class suggestion
whether memory/search/tool/protected gates may be needed
clarification question candidate
owner candidate

GPT-5.5 must not decide:

who the speaker is
whether Voice ID is confirmed
whether liveness passed
whether the speaker is JD
what private memory the speaker may access
what company data the speaker may access
what tools the speaker may use
whether authority exists
whether protected execution may run
whether simulation exists
whether audit can be skipped
whether Desktop or Adapter may bypass gates

GPT-5.5 may receive limited, sanitized identity/access context only as policy context, for example:

speaker_status: unknown / probable / confirmed
access_scope_summary: public_safe_only / private_memory_allowed / company_scope_allowed
protected_execution_policy: authority_gate_required
memory_policy: memory_gateway_required

GPT-5.5 must not receive:

raw voiceprints
identity secrets
full permission databases
private access tokens
connector secrets
raw authority credentials
internal audit secrets

4. Layer 1 — Wake / Activation Boundary

Purpose

Wake decides whether Selene should start or continue listening.

Owner

Wake / PH1.W / PH1.L owner per repo truth

Wake may decide

wake accepted
wake rejected
wake uncertain
barge-in candidate
session continuation candidate
noise / false wake

Wake must not decide

speaker identity
access permission
authority
memory scope
tool scope
protected action approval
semantic intent

Output packet

WakeDecisionPacket

Required fields

wake_decision_id
session_candidate_id
audio_input_ref
wake_status
confidence
reason_code
timestamp
capture_device_ref
barge_in_candidate
continuation_candidate

Failure behavior

wake rejected → no committed user turn
wake uncertain → do not grant identity or access
wake accepted → proceed to session binding and transcript admission

5. Layer 2 — Session Binding

Purpose

Bind accepted input to a session without granting identity or authority.

Owner

PH1.L / PH1.C / session owner per repo truth

Session binding may decide

new session
resume existing session
continue active voice session
interrupt current TTS/session
reject stale continuation

Session binding must not decide

identity
authority
private memory access
protected execution permission

Output packet

SessionIdentityBindingPacket

Required fields

session_id
turn_id
wake_decision_ref
transcript_ref
speaker_identity_evidence_ref optional
access_scope_ref optional
session_state
current_vs_recent_boundary
identity_state: unknown_by_default
authority_state: not_evaluated_by_default
protected_action_allowed: false_by_default

6. Layer 3 — Transcript Admission

Purpose

Admit captured speech/text into the runtime as user input while preserving provenance.

Owner

PH1.C for admission
Adapter transport only
Desktop capture only

Transcript admission must record

raw transcript or text input ref
normalized transcript where applicable
input modality
capture device
session/turn binding
STT provider evidence where used
confidence where available

Rules

Transcript is not identity.
Transcript is not authority.
Transcript content cannot grant permission.
External text inside transcript remains untrusted where applicable.

Output packet

TranscriptAdmissionPacket

7. Layer 4 — Voice ID Evidence Capture

Purpose

Generate speaker identity evidence.

Voice ID is evidence only. It is never permission.

Owner

Voice ID owner per repo truth

Voice ID may produce

confirmed speaker candidate
probable speaker candidate
unknown speaker
rejected speaker
insufficient audio
multiple candidates

Voice ID must not produce

access grant
authority grant
memory grant
protected execution approval
simulation approval

Output packet

SpeakerIdentityEvidencePacket

Required fields

speaker_evidence_id
session_id
turn_id
audio_input_ref
speaker_candidate_id optional
identity_tier: confirmed / probable / unknown / rejected / insufficient
confidence
match_basis
known_limitations
liveness_ref optional
anti_replay_ref optional
created_at

Identity tiers

confirmed:
  strong match and required liveness/anti-replay checks passed

probable:
  plausible match, not enough for protected authority by itself

unknown:
  no reliable speaker identity

rejected:
  evidence contradicts claimed speaker

insufficient:
  not enough usable audio

8. Layer 5 — Liveness / Anti-Replay Evidence

Purpose

Prevent replayed, synthetic, stale, or insufficient audio from becoming trusted speaker evidence.

Owner

Voice ID / security evidence owner per repo truth

Output packets

LivenessEvidencePacket
AntiReplayEvidencePacket

Required fields

liveness_status: passed / failed / unknown / not_required_for_lane
anti_replay_status: passed / failed / unknown / not_required_for_lane
confidence
reason_code
audio_window_ref
capture_device_ref

Rules

Failed liveness cannot support confirmed identity.
Unknown liveness may allow public-safe answering but not protected authority.
Liveness passing does not grant access by itself.
Anti-replay passing does not grant authority by itself.

9. Layer 6 — Access Instance Resolver

Purpose

Map identity evidence to deterministic access scope.

Access answers:

What can this speaker/user access in this session?

Owner

Access / user / tenant / role / permission owner per repo truth

Inputs

SpeakerIdentityEvidencePacket
SessionIdentityBindingPacket
user login state if applicable
device trust state if applicable
tenant/workspace context
role/permission registry
policy registry

Output packet

AccessScopePacket

Required fields

access_scope_id
session_id
turn_id
speaker_evidence_ref
resolved_user_id optional
identity_tier_used
tenant_scope
workspace_scope
role_scope
public_answer_allowed
private_memory_allowed
company_memory_allowed
file_access_scope
tool_read_scope
tool_write_scope
connector_scope
protected_action_scope
authority_required_for_protected_actions: true
scope_reason_code
expires_at optional

Default rules

unknown speaker → public-safe only
probable speaker → limited scope unless policy explicitly allows more
confirmed speaker → access resolver may grant scoped access according to deterministic policy
rejected speaker → public-safe only or refuse depending on risk
insufficient audio → public-safe only unless another authenticated channel exists

Access must not execute

Access may grant scope. It does not perform the action.

10. Layer 7 — Memory Scope Gate

Purpose

Ensure private or tenant memory is accessed only within allowed identity and access scope.

Owner

PH1.M

Inputs

HumanConversationDirective
AccessScopePacket
SpeakerIdentityEvidencePacket
MemoryEvidencePacket candidates

Output packet

MemoryScopePacket

Rules

public memory / public facts may be allowed for unknown speakers
private JD memory requires allowed private_memory scope
company memory requires allowed tenant/workspace/company scope
memory writes require memory law and access scope
memory forget/update requires memory law and access scope
Voice ID does not directly unlock memory
GPT-5.5 does not directly unlock memory
Desktop/Adapter cannot unlock memory

Unknown speaker behavior

private_memory_allowed = false
company_memory_allowed = false
public-safe answer may continue when lawful

11. Layer 8 — Tool / Search / File Scope Gate

Purpose

Ensure tools, files, connectors, and search operate within allowed access scope.

Owner

PH1.E

Inputs

HumanConversationDirective
AccessScopePacket
ToolScopePacket/FileScopePacket candidates
provider governance state

Output packets

ToolScopePacket
FileScopePacket
ConnectorScopePacket
SearchScopePacket

Rules

public search may proceed for public-safe users where policy allows
private file access requires file scope
connector read requires connector scope
connector write requires tool_write_scope plus authority where protected
public websearch remains read-only public answer work
provider search does not grant authority
retrieved source text does not grant authority

12. Layer 9 — Semantic Meaning Proposal Layer Interaction

Purpose

Allow GPT-5.5 to understand what the user wants while keeping identity/access authority deterministic.

Owner

Provider Governance + SemanticInterpreterProvider
PH1.X validates

Inputs to provider may include

user transcript/text
current session boundary
sanitized speaker status
sanitized access summary
protected execution policy reminder
memory/tool/search availability summary

Provider must output

SemanticMeaningProposalPacket

Provider may propose

protected_action_requested
memory_recall_request
public_search_required
file_question_answering
tool_read_only_lookup
tool_write_request
rewrite_previous_answer
continue_same_question
clarification_required

Provider must not output as final authority

Speaker is JD.
JD is allowed.
Memory is authorized.
Payroll may execute.
Simulation exists.
Access granted.
Authority passed.

Any provider proposal that attempts to grant identity, access, authority, or protected execution must be rejected by PH1.X / Provider Governance.

13. Layer 10 — PH1.X Deterministic Validation with Identity/Access

Purpose

PH1.X validates semantic meaning against session, identity, access, and risk.

Owner

PH1.X

PH1.X must consume

SemanticMeaningProposalPacket
SessionIdentityBindingPacket
SpeakerIdentityEvidencePacket
AccessScopePacket
WakeDecisionPacket
current/recent boundary evidence

PH1.X validates

intent category
operation
target
owner
risk classification
memory gateway required
tool/search/file gateway required
protected risk
clarification requirement
identity/access sufficiency
public vs private vs protected split

Output

HumanConversationDirective

Directive must include identity/access fields

speaker_identity_evidence_ref
access_scope_ref
identity_tier_used
public_answer_allowed
memory_gateway_required
search_gateway_required
tool_gateway_required
protected_gate_required
authority_gate_required
simulation_gate_required
protected_action_allowed: false unless authority + simulation pass

14. Layer 11 — Risk Classification

Purpose

Classify whether the request is public/advisory, private/memory, tool/file/connector scoped, protected, or mixed.

Owners

PH1.X primary
PH1.M for memory scope
PH1.E for tool/search/file scope
Protected Risk Gate for protected execution

Categories

PUBLIC_SAFE
PRIVATE_MEMORY
TENANT_DATA
TOOL_READ
TOOL_WRITE
CONNECTOR_READ
CONNECTOR_WRITE
PROTECTED_ACTION
MIXED_PUBLIC_AND_PROTECTED
IDENTITY_UNCERTAIN
AUTHORITY_UNCERTAIN

Rules

public part may proceed when allowed
private part requires access scope
protected part requires authority + simulation
mixed requests must split public/advisory from protected execution
unknown identity does not block harmless public answers
unknown identity blocks private memory and protected execution

15. Layer 12 — Authority Gate

Purpose

Determine whether the resolved user is allowed to perform the requested protected action now.

Owner

Authority / Access Governance owner per repo truth

Inputs

HumanConversationDirective
AccessScopePacket
SpeakerIdentityEvidencePacket
ProtectedRiskPacket
requested action
requested target
tenant/workspace scope
current policy registry
confirmation state where required

Output packet

AuthorityDecisionPacket

Required fields

authority_decision_id
session_id
turn_id
user_id optional
action
target_ref optional
authority_state: granted / denied / insufficient / not_required
reason_code
identity_tier_used
access_scope_ref
confirmation_required
confirmation_state
audit_required

Rules

Voice ID cannot grant authority.
Access scope cannot execute actions.
GPT-5.5 cannot grant authority.
Desktop cannot grant authority.
Adapter cannot grant authority.
Authority is action-specific and time-specific.
Authority must fail closed when uncertain.

16. Layer 13 — Simulation Gate

Purpose

Ensure protected execution happens only through approved deterministic simulations/processes.

Owner

SimulationExecutor

Inputs

AuthorityDecisionPacket
HumanConversationDirective
ProtectedRiskPacket
SimulationLookupPacket
idempotency evidence
audit path

Output packet

SimulationGateDecisionPacket

Rules

no simulation → no protected execution
authority denied → no protected execution
identity uncertain → no protected execution
missing idempotency → no protected execution
missing audit path → no protected execution
provider output cannot satisfy simulation
public advisory explanation may still proceed where lawful

17. Layer 14 — Protected Execution Audit

Purpose

Record identity, access, authority, simulation, and execution evidence for protected or denied protected actions.

Owner

Storage / Audit

Required audit evidence

WakeDecisionPacket
SessionIdentityBindingPacket
TranscriptAdmissionPacket
SpeakerIdentityEvidencePacket
LivenessEvidencePacket where applicable
AccessScopePacket
SemanticMeaningProposalPacket
HumanConversationDirective
ProtectedRiskPacket
AuthorityDecisionPacket
SimulationGateDecisionPacket
SimulationExecutionPacket if executed
FailClosedEvidencePacket if denied
WriteOutputPacket

Rule

Denied protected actions must be audited too.

Fail-closed is a security event, not invisible nothingness.

18. Desktop / Adapter No-Authority Boundary

Desktop and Adapter are not identity, access, authority, memory, semantic, provider, or protected-execution owners.

Desktop may

capture audio
show wake state
show listening state
send transcript/input
render approved output
play approved TTS
show source chips/images/artifacts approved by runtime

Desktop must not

decide speaker identity
decide access
decide authority
decide protected execution
decide semantic meaning
call provider directly
read private memory directly
choose search/tool route
approve payroll/salary/leave/database mutation

Adapter may

transport packets
bridge runtime communication
preserve provenance
report health/provenance

Adapter must not

become identity brain
become access brain
become memory brain
become PH1.X
become PH1.E
become protected execution shortcut

19. Example Flows

Example A — Unknown voice asks public question

User says:

What is the weather today?

Expected flow:

Wake accepted
Session opened/resumed
Voice ID unknown
Access public-safe only
GPT-5.5 proposes public weather/time/search answer as needed
PH1.X validates public-safe request
PH1.E may use read-only public provider where allowed
PH1.WRITE answers
Desktop renders/speaks
Audit records public-safe route

Allowed result:

Answer public weather if location is available or ask location.

Example B — Unknown voice asks private memory question

User says:

What did I tell you yesterday about my business plan?

Expected flow:

Wake accepted
Voice ID unknown
Access public-safe only
GPT-5.5 proposes memory_recall_request
PH1.X marks memory gateway required
PH1.M denies private memory due to access scope
PH1.WRITE explains verification is required
Audit records denied memory access

Allowed result:

I cannot access private memory until I can verify who is speaking.

Example C — Confirmed JD asks private memory question

User says:

What did I tell you yesterday about my business plan?

Expected flow:

Wake accepted
Voice ID confirmed/probable according to policy
Access grants JD private memory scope
GPT-5.5 proposes memory_recall_request
PH1.X routes to PH1.M
PH1.M validates memory scope
PH1.WRITE answers from allowed memory evidence
Audit records memory access

Allowed result:

Answer from scoped memory evidence only.

Example D — Unknown voice asks protected action

User says:

Approve payroll for Tim.

Expected flow:

Wake accepted
Voice ID unknown
Access public-safe only
GPT-5.5 proposes protected_action_requested / approve_payroll
PH1.X marks protected risk
Authority Gate denies identity/access insufficiency
SimulationExecutor does not execute
PH1.WRITE explains fail-closed result
Audit records denied protected action

Allowed result:

I cannot approve payroll without verified authority and an approved payroll simulation.

Example E — JD asks protected action without simulation

User says:

Approve payroll for Tim.

Expected flow:

Wake accepted
Voice ID confirmed/probable according to policy
Access resolves JD scope
GPT-5.5 proposes protected_action_requested
PH1.X marks protected risk
Authority may be sufficient or require confirmation
Simulation lookup missing/not approved
SimulationGateDecision denies execution
PH1.WRITE explains simulation is missing
Audit records fail-closed

Allowed result:

I cannot execute payroll because the approved payroll simulation is not available.

Example F — Mixed request

User says:

Search salary trends and increase Tim's salary.

Expected flow:

GPT-5.5 proposes mixed_public_and_protected
PH1.X splits request
PH1.E may handle public salary trend research
Authority + Simulation gate protected salary increase
Protected increase fails closed unless authority and simulation pass
PH1.WRITE presents public research plus protected denial

Allowed result:

Public research may proceed.
Salary increase must not execute without authority + simulation.

20. Required Packets

Codex must reuse existing packet names if repo truth already defines them.

If names differ, Codex must map this logical architecture to current repo packets and report the mapping.

Identity/access packets

WakeDecisionPacket
SessionIdentityBindingPacket
TranscriptAdmissionPacket
SpeakerIdentityEvidencePacket
LivenessEvidencePacket
AntiReplayEvidencePacket
AccessScopePacket
MemoryScopePacket
ToolScopePacket
FileScopePacket
ConnectorScopePacket
SearchScopePacket
ProtectedRiskPacket
AuthorityDecisionPacket
SimulationGateDecisionPacket
ProtectedExecutionAuditPacket
FailClosedEvidencePacket

Required default fields carried across session/turn

speaker_id_candidate
user_id_candidate
identity_tier
liveness_status
access_instance_id
tenant_scope
workspace_scope
role_scope
memory_scope
tool_scope
authority_state
protected_action_allowed = false by default
private_memory_allowed = false by default
company_memory_allowed = false by default
tool_write_allowed = false by default

21. Build Sequence

Codex must not implement the full spine at once.

This document is an architecture reference. Runtime implementation requires repo-truth activation.

Required activation pack:

Selene Identity + Access + Authority Spine — Codex VIA-0 / VIA-1 Activation Pack

The lawful build sequence is:

VIA-0 — Add Identity + Access + Authority Spine docs
VIA-1 — Repo Truth Audit / Activation Pack
VIA-2 — Session Identity Packet Baseline
VIA-3 — Voice ID Evidence-Only Contract
VIA-4 — Access Resolver Baseline
VIA-5 — Memory Scope Integration
VIA-6 — Tool/Search/File Scope Integration
VIA-7 — PH1.X Identity/Access Validation Integration
VIA-8 — Authority Gate Baseline
VIA-9 — Simulation Gate Integration
VIA-10 — Audit Evidence Pack
VIA-11 — Desktop/Adapter No-Authority Proof
VIA-12 — JD Live Acceptance Pack
VIA-13 — Old Path Cleanup

22. VIA-0 — Documentation Slice

Goal

Add this architecture document and link it to the four-document provider-first architecture set.

Requirements

read AGENTS.md first
read required architecture docs
prove clean tree
add docs only
no runtime edits
no old path deletion
link this doc to GHCS and function cards
final clean tree proof
commit/push proof if edits are made

Acceptance

CODEX_TESTED
JD_LIVE_ACCEPTANCE_NOT_APPLICABLE

23. VIA-1 — Repo Truth Audit / Activation Pack

Goal

Map current repo truth before implementation.

Required discovery

current Wake owner files
current Session owner files
current Voice ID owner files
current liveness/anti-replay surfaces
current Access/user/role/permission owner files
current Authority owner files
current SimulationExecutor files
current PH1.X files
current PH1.M files
current PH1.E files
current PH1.WRITE files
current Desktop files touching wake/voice/session/render
current Adapter files touching voice/session/runtime transport
current packet/contract files
current audit/storage files
current tests/smokes
current old identity/access/authority shortcuts
current Desktop identity/access decision surfaces
current Adapter identity/access decision surfaces
current memory access without speaker scope
current protected execution shortcuts

Required activation pack output

Selene Identity + Access + Authority Spine — Codex VIA-0 / VIA-1 Activation Pack

Activation pack must include:

1. AGENTS.md read proof
2. mandatory docs read proof
3. clean tree proof
4. lane declaration
5. current owner map
6. current packet/schema map
7. current live runtime path map
8. current test map
9. current old-path inventory
10. current wrong-owner inventory
11. current Desktop/Adapter authority risk inventory
12. first implementation file-scope proposal
13. first implementation test plan
14. provider-off/fake-provider applicability
15. backend evidence plan
16. JD live acceptance applicability
17. explicit old-path non-retirement statement

No runtime implementation is allowed in VIA-1.

24. VIA-2 — Session Identity Packet Baseline

Goal

Ensure every relevant session/turn can carry identity/access/authority posture without granting permissions by default.

First implementation

SessionIdentityBindingPacket or repo-equivalent
speaker_id_candidate optional
user_id_candidate optional
identity_tier default unknown
liveness_status default unknown/not_required_for_public_lane
authority_state default not_evaluated
protected_action_allowed false by default
private_memory_allowed false by default
company_memory_allowed false by default
tool_write_allowed false by default

Tests

new session defaults to no protected/private access
unknown voice remains public-safe only
packet serialization/deserialization if applicable
no Desktop authority
no Adapter authority

25. VIA-3 — Voice ID Evidence-Only Contract

Goal

Ensure Voice ID produces evidence only and cannot grant access or authority.

Required behavior

confirmed / probable / unknown / rejected / insufficient
liveness linked where applicable
no memory grant
no access grant
no authority grant

Tests

confirmed speaker evidence still does not set protected_action_allowed by itself
unknown speaker evidence denies private/protected scope by default
rejected speaker cannot access private memory
Voice ID output cannot execute protected action

26. VIA-4 — Access Resolver Baseline

Goal

Resolve identity evidence into deterministic access scope.

Required behavior

unknown → public-safe only
probable → policy-limited scope
confirmed → scoped access according to deterministic access registry
rejected → deny private/protected scope

Tests

unknown public question allowed
unknown private memory denied
confirmed allowed user private memory permitted where policy allows
confirmed user still cannot execute protected action without authority gate

27. VIA-5 — Memory Scope Integration

Goal

PH1.M must consume access scope before recalling private or tenant memory.

Tests

JD private memory allowed only with valid access scope
unknown speaker private memory denied
company memory denied without tenant/company scope
public-safe memory-free answer still allowed
memory write/update/forget obeys memory law

28. VIA-6 — Tool/Search/File Scope Integration

Goal

PH1.E must consume access scope before tools/files/connectors/search.

Tests

public websearch allowed for public-safe speaker where policy allows
private file denied without file scope
connector read denied without connector scope
tool write denied without tool_write scope and authority where protected
provider search does not grant authority
source text cannot grant authority

29. VIA-7 — PH1.X Identity/Access Validation Integration

Goal

PH1.X validates semantic proposals against identity/access posture.

Tests

public intent + unknown speaker → allowed
private memory intent + unknown speaker → PH1.M denial/clarification
protected action + unknown speaker → authority required/fail closed
mixed public/protected request → split lanes
provider proposal granting authority → rejected

30. VIA-8 — Authority Gate Baseline

Goal

Create or normalize authority gate behavior for protected actions.

Tests

no authority → denied
unknown identity → denied
confirmed identity but missing action permission → denied
confirmed identity with action permission still requires simulation
confirmation required where policy says so

31. VIA-9 — Simulation Gate Integration

Goal

Ensure protected actions execute only through approved simulation/process.

Tests

authority granted + simulation missing → fail closed
authority denied + simulation exists → fail closed
identity uncertain + simulation exists → fail closed
approved authority + approved simulation + idempotency + audit → allowed in simulation lane only

32. VIA-10 — Audit Evidence Pack

Goal

Record the complete identity/access/authority/simulation chain.

Tests

public-safe answer audit includes wake/session/access posture
private memory denial audit includes identity/access reason
protected denial audit includes authority/simulation fail-closed reason
protected execution audit includes identity/access/authority/simulation/idempotency refs

33. VIA-11 — Desktop/Adapter No-Authority Proof

Goal

Prove Desktop and Adapter remain capture/render/transport only.

Tests

Desktop cannot set identity_tier
Desktop cannot set access scope
Desktop cannot set authority_state
Adapter cannot set identity_tier
Adapter cannot set access scope
Adapter cannot set authority_state
runtime ignores any unauthorized client authority field

34. VIA-12 — JD Live Acceptance Pack

Required live tests

JD asks public question → allowed
Unknown speaker asks public question → allowed where policy permits
JD asks private memory question → allowed only with valid identity/access scope
Unknown speaker asks private memory question → denied
Unknown speaker asks approve payroll → fail closed
JD asks approve payroll without simulation → fail closed
Mixed request: search salary trends and increase Tim's salary → public part allowed, protected part fail closed

Required backend evidence

WakeDecisionPacket
SessionIdentityBindingPacket
SpeakerIdentityEvidencePacket
AccessScopePacket
HumanConversationDirective
MemoryScopePacket where memory used/denied
ToolScopePacket/SearchScopePacket where tools/search used
AuthorityDecisionPacket where protected action requested
SimulationGateDecisionPacket where protected action requested
WriteOutputPacket
Audit evidence refs
Desktop/Adapter provenance proof

35. VIA-13 — Old Path Cleanup

Goal

Remove or quarantine old identity/access/authority shortcuts after canonical path proof.

Cleanup targets

Voice ID authority shortcuts
Wake identity shortcuts
Session authority shortcuts
Desktop identity decisions
Desktop access decisions
Desktop authority decisions
Adapter identity decisions
Adapter access decisions
Adapter authority decisions
memory access without speaker/access scope
PH1.X protected action shortcuts
PH1.E tool/write shortcuts
provider-granted access/authority
protected execution without authority + simulation

Retirement condition

canonical path proven
old behavior regression proven
JD live acceptance passed where visible
backend evidence agrees
no active caller remains
clean tree proven

36. Master Test Matrix

Every implementation slice must define exact tests before editing.

Required classes:

unit tests
packet/schema tests
owner-routing tests
unknown-speaker tests
confirmed-speaker tests
rejected-speaker tests
memory-scope tests
tool/file/search-scope tests
protected fail-closed tests
simulation-gate tests
audit evidence tests
Desktop no-authority tests
Adapter no-authority tests
JD live tests where user-visible
old-path regression tests

Minimum scenario matrix:

unknown speaker + public question = allow public-safe answer
unknown speaker + private memory = deny private memory
unknown speaker + protected action = fail closed
confirmed speaker + private memory = allow only scoped memory
confirmed speaker + protected action + no simulation = fail closed
confirmed speaker + protected action + no authority = fail closed
mixed public/protected = split lanes
provider claims authority = reject provider claim
Desktop sends authority field = ignore/reject
Adapter sends identity/access override = ignore/reject

37. Codex Build Instruction Template

Every Codex instruction derived from this architecture must include:

State whether this is:
- architecture reference only
- VIA-0 / VIA-1 activation pack
- implementation slice
- repair slice
- old-path retirement slice

If the instruction is for implementation and no VIA activation pack exists, Codex must stop with:

IAA_ACTIVATION_PACK_REQUIRED

Every implementation instruction must include:

Read AGENTS.md first.
Read mandatory architecture docs.
Declare lane.
Prove clean tree.
Perform current owner discovery.
Identify canonical owners.
List exact files proposed for edit.
Define exact tests before editing.
Prove no Desktop authority.
Prove no Adapter authority.
Prove unknown speaker public/private/protected behavior.
Prove Voice ID evidence-only behavior where touched.
Prove access scope behavior where touched.
Prove authority fail-closed behavior where touched.
Prove simulation fail-closed behavior where touched.
Produce backend evidence report.
Define JD live scenario where user-visible.
Do not retire old paths before proof.
End clean tree.
Commit/push proof where edits are made.

If any derived build instruction omits these requirements, Codex must stop with:

IAA_BUILD_DERIVATION_INCOMPLETE

38. Backend Report Template

Every implementation final report must include:

what was built
lane declaration
files edited
canonical owners touched
Wake behavior changed yes/no
Session behavior changed yes/no
Voice ID behavior changed yes/no
Access behavior changed yes/no
Authority behavior changed yes/no
Simulation behavior changed yes/no
Desktop touched yes/no
Adapter touched yes/no
no Desktop authority proof
no Adapter authority proof
unknown speaker behavior proof
confirmed speaker behavior proof where relevant
private memory scope proof where relevant
tool/search/file scope proof where relevant
protected fail-closed proof where relevant
simulation gate proof where relevant
audit evidence refs
old behavior regression evidence
JD live prompt/input where applicable
actual visible/audible output where applicable
backend packet refs
failure/repair history
final acceptance status
final clean tree proof
commit/push proof where edits were made

39. Final Global Standard

Selene’s human interface requires two interlocking spines:

Global Human Conversation Spine:
  probabilistic semantic understanding + deterministic owner routing

Identity + Access + Authority Spine:
  deterministic identity evidence + access scope + authority + simulation gating

The final law is:

Wake listens.
Voice ID evidences.
Access scopes.
GPT-5.5 understands.
PH1.X validates.
PH1.M protects memory.
PH1.E protects tools/search/files.
Authority gates protected actions.
SimulationExecutor executes only approved protected processes.
PH1.WRITE explains.
Desktop renders.
Adapter transports.
Audit remembers.

Build this one slice at a time.
Prove every slice.
Never let convenience become authority.
