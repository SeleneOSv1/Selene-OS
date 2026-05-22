Selene Global Request Decision Lattice + 5-Lane Business Risk View Master Design

DOCUMENT TYPE:
DEDICATED MASTER DESIGN / GLOBAL REQUEST ROUTING ARCHITECTURE

TASK:
SELENE_GLOBAL_REQUEST_DECISION_LATTICE_MASTER_DESIGN

BUILD CLASS:
ARCHITECTURE / REQUEST ROUTING / PROBABILISTIC-PLUS-DETERMINISTIC DECISION SYSTEM

CONTROLLING DOCUMENTS:
1. AGENTS.md
2. Selene Master Architecture Build Set
3. Selene Final Overall Architecture Build Plan
4. Selene Overall Repo-Truth Activation Pack
5. Selene Global Human Conversation Spine Master Architecture
6. Selene Identity + Access + Authority Spine Master Architecture
7. Selene Function Stack Architecture — Intent and Enterprise Stack Map
8. Selene PH1.M Human Memory Core Master Design
9. Selene Search Intelligence Lane — Revised Enterprise Websearch Master Design
10. Selene PH1.WRITE — Structured Writing + Human Presentation Master Design
11. Selene Emotional Intelligence + Relationship Presence Stack
12. Conversational Experience + Quick Assist Stack

PURPOSE:
Define Selene’s global request decision system so every user request is classified by meaning, data scope, freshness, action effect, risk, required gates, and presentation mode before routing to the correct Selene stack.

This system upgrades the simple 5-lane business split into a world-class multidimensional decision lattice that supports normal chat, public business knowledge, websearch, private company reads, memory, files, tools, protected execution, mixed requests, Selene emotional presentation, Quick Assist, and PH1.WRITE final presentation.

0. Master Standard

Selene must not route human requests using phrase lists, keyword shortcuts, or scattered deterministic helpers.

Selene must use a global probabilistic-plus-deterministic request decision system:

GPT-5.5 / SemanticInterpreterProvider proposes what the user means.
PH1.X validates the decision lattice.
Selene stack owners handle their own gates.
PH1.WRITE presents the answer naturally.
Protected execution runs only through authority + simulation.

The user should experience Selene as smooth, human, helpful, and clear.

Internally, Selene must produce structured routing truth.

The system is:

probabilistic-first for human meaning and communication
structured/deterministic for gates, evidence, access, authority, simulation, and audit

1. Why The 5-Lane Model Is Useful But Not Enough

The 5-lane business model is correct as a user/business-readable summary:

1. Normal public question
2. Public business knowledge question
3. Private company read question
4. Protected business execution
5. Ambiguous or mixed request

But real requests can contain multiple dimensions at once.

Example:

Check payroll and approve it.

This is not one lane.

It contains:

private company read
+ protected execution
+ possible authority requirement
+ possible simulation requirement
+ PH1.WRITE explanation requirement

So Selene needs the 5-lane view plus a deeper request lattice.

The 5 lanes are the readable surface.

The lattice is the actual routing brain.

2. The Selene Global Request Decision Lattice

The lattice classifies every request across these dimensions:

A. Intent Type
B. Data Scope
C. Freshness Need
D. Action Effect
E. Risk Level
F. Required Gates
G. Stack Owner
H. Presentation Mode
I. Split / Clarification Requirement
J. Audit Requirement

Each request may have one or more subrequests.

Each subrequest receives its own lattice classification.

3. Target Runtime Flow

Correct flow:

User input
→ Human Input Admission
→ SemanticInterpreterProvider / GPT-5.5 proposes meaning candidates
→ PH1.X validates the Global Request Decision Lattice
→ PH1.X creates one or more RequestDecisionPackets
→ PH1.X splits mixed requests where needed
→ each subrequest routes to the correct stack:
   - normal public answer
   - public business knowledge
   - websearch / source evidence
   - private company read
   - memory recall
   - file/document analysis
   - tool/connector read
   - protected business execution
   - Quick Assist / clarification
→ stack owner performs required gates
→ PH1.WRITE creates final human output
→ Adapter transports
→ Desktop/iPhone render or speak only
→ audit records decisions and evidence

The lattice must happen before stack execution.

Stack owners may validate further, but they must not independently invent user meaning from raw language unless their owner contract explicitly allows it.

4. Lattice Dimension A — Intent Type

Intent type describes what the user appears to want.

Canonical examples:

explain
answer_public_question
answer_public_business_knowledge
search_current_public_information
verify_claim
read_private_company_data
recall_memory
analyze_file
use_tool_read_only
write_or_rewrite
summarize
translate
create_artifact
generate_image
generate_video
send_message
approve_or_reject
create_record
update_record
delete_record
pay_or_refund
change_access
execute_business_process
clarify
confirm
quick_assist

OpenAI / GPT-5.5 may propose the intent type.

PH1.X validates it.

5. Lattice Dimension B — Data Scope

Data scope determines what kind of information is involved.

PUBLIC_GENERAL
PUBLIC_BUSINESS_KNOWLEDGE
CURRENT_EXTERNAL_PUBLIC
USER_PRIVATE_MEMORY
USER_UPLOADED_FILE
COMPANY_PRIVATE_DATA
TENANT_DATA
AUTHORITY_GATED_DATA
PROTECTED_BUSINESS_RECORD
UNKNOWN_SCOPE

Examples:

What is gross margin?
→ PUBLIC_BUSINESS_KNOWLEDGE

What was our gross margin last month?
→ COMPANY_PRIVATE_DATA

What did I tell you yesterday?
→ USER_PRIVATE_MEMORY

What does this uploaded file say?
→ USER_UPLOADED_FILE

Data scope decides whether Selene needs:

no gate
websearch/source evidence
memory gate
file gate
private data gate
tenant gate
authority gate

6. Lattice Dimension C — Freshness Need

Freshness determines whether stable knowledge is enough or current evidence is required.

STABLE_KNOWLEDGE
CURRENT_PUBLIC_WEB
RECENT_NEWS
LIVE_COMPANY_SYSTEM
HISTORICAL_PRIVATE_DATA
FRESH_MEMORY
DEEP_MEMORY
UNKNOWN_FRESHNESS

Examples:

How does payroll usually work?
→ STABLE_KNOWLEDGE

What are the latest payroll tax rules?
→ CURRENT_PUBLIC_WEB

How many staff are on leave today?
→ LIVE_COMPANY_SYSTEM

What did we decide this morning?
→ FRESH_MEMORY / DAY_MEMORY

Search is not automatic.

Search is used when freshness/current/source-backed evidence is needed.

7. Lattice Dimension D — Action Effect

Action effect determines whether the request changes anything.

NO_MUTATION
READ_ONLY
DRAFT_ONLY
DISPLAY_ONLY
EXTERNAL_MESSAGE_DRAFT
EXTERNAL_MESSAGE_SEND
PRIVATE_DATA_READ
BUSINESS_MUTATION
AUTHORITY_MUTATION
IRREVERSIBLE_OR_HIGH_RISK_ACTION
UNKNOWN_EFFECT

Examples:

Explain commission structures.
→ NO_MUTATION

Show Tim’s salary.
→ PRIVATE_DATA_READ

Draft an invoice email.
→ DRAFT_ONLY

Send the invoice.
→ EXTERNAL_MESSAGE_SEND

Increase Tim’s salary.
→ BUSINESS_MUTATION

This dimension is critical.

Public language may be probabilistic.

Business mutation must be deterministic and simulation-gated.

8. Lattice Dimension E — Risk Level

Risk level summarizes what can go wrong.

LOW_PUBLIC
PUBLIC_SOURCE_BACKED
PRIVATE_READ
SENSITIVE_PRIVATE_READ
PROTECTED_ACTION
MIXED_PUBLIC_AND_PROTECTED
AMBIGUOUS_PROTECTED_RISK
UNKNOWN_RISK

Rules:

LOW_PUBLIC → normal answer allowed
PUBLIC_SOURCE_BACKED → websearch/evidence if needed
PRIVATE_READ → identity + permission + tenant + audit
PROTECTED_ACTION → authority + confirmation + simulation + audit
MIXED → split request
AMBIGUOUS_PROTECTED_RISK → clarify or fail closed

9. Lattice Dimension F — Required Gates

Required gates are derived from the previous dimensions.

Possible gates:

NO_GATE
WEBSEARCH_GATE
SOURCE_EVIDENCE_GATE
MEMORY_SCOPE_GATE
FILE_SCOPE_GATE
PRIVATE_DATA_GATE
TENANT_SCOPE_GATE
TOOL_READ_GATE
TOOL_WRITE_GATE
AUTHORITY_GATE
CONFIRMATION_GATE
SIMULATION_GATE
AUDIT_GATE
CLARIFICATION_GATE
PROVIDER_GOVERNANCE_GATE

Examples:

What is inventory turnover?
→ NO_GATE

What are current ATO payroll rules?
→ WEBSEARCH_GATE + SOURCE_EVIDENCE_GATE + PROVIDER_GOVERNANCE_GATE

Show Tim’s current salary.
→ PRIVATE_DATA_GATE + TENANT_SCOPE_GATE + AUDIT_GATE

Increase Tim’s salary.
→ AUTHORITY_GATE + CONFIRMATION_GATE + SIMULATION_GATE + AUDIT_GATE

10. Lattice Dimension G — Stack Owner

The lattice routes to canonical Selene owners.

Examples:

normal public answer → PH1.WRITE / GPT-5.5 public answer path
public business knowledge → PH1.WRITE / GPT-5.5 public answer path
current public information → PH1.E Search Intelligence Lane + PH1.WRITE
private company read → private data/tool owner + Access/Governance + PH1.WRITE
memory recall → PH1.M + PH1.WRITE
file analysis → PH1.E / PH1.DOC + PH1.WRITE
tool read → PH1.E
protected execution → Authority + SimulationExecutor + PH1.WRITE
quick clarification → Quick Assist + PH1.WRITE
persona/tone → Selene Emotional Intelligence + PH1.WRITE

Desktop and Adapter are never owners of meaning, access, search, memory, protected execution, or final writing policy.

11. Lattice Dimension H — Presentation Mode

The lattice must also decide how the answer should be presented.

Presentation modes include:

DIRECT_ANSWER
PUBLIC_BUSINESS_EXPLAINER
SOURCE_BACKED_SEARCH_ANSWER
PRIVATE_DATA_SUMMARY
MEMORY_RECALL_ANSWER
FILE_ANALYSIS_ANSWER
TOOL_RESULT_ANSWER
PROTECTED_DENIAL
PROTECTED_SUCCESS_SUMMARY
MIXED_SPLIT_RESPONSE
QUICK_ASSIST_GUIDANCE
CLARIFICATION_QUESTION
BUSINESS_WRITING
CODEX_INSTRUCTION
COMPARISON
RICH_RENDERER_CARD
TTS_SAFE_SUMMARY

PH1.WRITE owns final presentation.

Selene Emotional Intelligence and Quick Assist may shape tone where lawful.

12. Lattice Dimension I — Split / Clarification Requirement

Some requests must be split.

Examples:

Check payroll and approve it.
→ split into private read + protected execution

Search salary trends and increase Tim’s salary.
→ split into public search + protected execution

Fix the customer issue.
→ ambiguous; ask clarification

Split outputs:

Subrequest A: public/private/read-only part
Subrequest B: protected/mutation part

Clarification is required when:

intent is too vague
protected risk exists and confidence is low
required target is ambiguous
private data scope is unclear
multiple actions are possible

Clarification should be short and human.

PH1.WRITE / Quick Assist should avoid robotic phrasing.

13. Lattice Dimension J — Audit Requirement

Audit requirement is derived from risk and data scope.

NO_AUDIT_REQUIRED
LIGHT_TRACE
SOURCE_TRACE
PRIVATE_READ_AUDIT
MEMORY_ACCESS_AUDIT
TOOL_EXECUTION_AUDIT
PROTECTED_FAIL_CLOSED_AUDIT
PROTECTED_EXECUTION_AUDIT

Examples:

What is gross margin?
→ LIGHT_TRACE optional

What was our gross margin last month?
→ PRIVATE_READ_AUDIT

Approve payroll.
→ PROTECTED_EXECUTION_AUDIT or PROTECTED_FAIL_CLOSED_AUDIT

14. The 5-Lane Business Risk View

The 5-lane view is the simplified business-facing summary of the lattice.

Lane 1 — Normal Public Question

Examples:

What is gross margin?
What is inventory turnover?
Explain accrual accounting.

Behavior:

GPT-5.5 / PH1.WRITE answer
no simulation
no private data gate
no websearch unless freshness/source requested

Lane 2 — Public Business Knowledge Question

Examples:

How do commission structures work?
What is the best way to manage staff rosters?
How should a small business think about cash flow?

Behavior:

normal probabilistic public answer
business topic does not automatically mean protected execution
search only if current/source-backed info is needed

Lane 3 — Private Company Read Question

Examples:

What was our gross margin last month?
How many staff are on leave today?
Show Tim’s current salary.

Behavior:

identity required
permission required
tenant/workspace scope required
private data read allowed only through governed system
audit required
no write simulation required unless data is changed

This lane is the most important addition to avoid confusion between normal chat and protected execution.

Lane 4 — Protected Business Execution

Examples:

Approve payroll.
Increase Tim’s salary.
Refund this customer.
Create a new supplier.
Change the roster.
Send the invoice.
Grant access.
Delete this record.

Behavior:

required fields
identity/access
authority
confirmation
simulation match
execution
audit

No simulation means no execution.

Lane 5 — Ambiguous or Mixed Request

Examples:

Sort out Tim’s pay.
Fix the customer issue.
Do the roster.
Check payroll and approve it.

Behavior:

split if possible
clarify if needed
public/read part may proceed if allowed
protected part fails closed unless authority + simulation pass

15. Example Decisions

Example 1 — Normal public question

User:

What is gross margin?

Lattice:

intent: explain
scope: PUBLIC_BUSINESS_KNOWLEDGE
freshness: STABLE_KNOWLEDGE
effect: NO_MUTATION
risk: LOW_PUBLIC
gates: NO_GATE
owner: PH1.WRITE / GPT-5.5 public answer
presentation: PUBLIC_BUSINESS_EXPLAINER

Output:

Gross margin is revenue minus cost of goods sold, shown as a percentage of revenue.

Example 2 — Current public search

User:

What are the current payroll tax rules in Singapore?

Lattice:

intent: search_current_public_information
scope: CURRENT_EXTERNAL_PUBLIC
freshness: CURRENT_PUBLIC_WEB
effect: READ_ONLY
risk: PUBLIC_SOURCE_BACKED
gates: WEBSEARCH_GATE + SOURCE_EVIDENCE_GATE + PROVIDER_GOVERNANCE_GATE
owner: PH1.E Search Intelligence + PH1.WRITE
presentation: SOURCE_BACKED_SEARCH_ANSWER

Output:

Best available answer with source chips.

Example 3 — Private company read

User:

Show Tim’s current salary.

Lattice:

intent: read_private_company_data
scope: COMPANY_PRIVATE_DATA
effect: PRIVATE_DATA_READ
risk: SENSITIVE_PRIVATE_READ
gates: IDENTITY + PERMISSION + TENANT_SCOPE + AUDIT
owner: private data owner + Access/Governance + PH1.WRITE
presentation: PRIVATE_DATA_SUMMARY or access denial

Output if allowed:

Tim’s current salary is shown in the payroll record as...

Output if denied:

I can’t show salary details without the right access for this workspace.

Example 4 — Protected execution

User:

Increase Tim’s salary.

Lattice:

intent: update_record / protected business execution
scope: PROTECTED_BUSINESS_RECORD
effect: BUSINESS_MUTATION
risk: PROTECTED_ACTION
gates: AUTHORITY + CONFIRMATION + SIMULATION + AUDIT
owner: Authority + SimulationExecutor + PH1.WRITE
presentation: PROTECTED_DENIAL or PROTECTED_SUCCESS_SUMMARY

Output if no simulation:

I can’t make that change without an approved salary-change simulation and verified authority.

Example 5 — Mixed request

User:

Check payroll and approve it.

Lattice split:

Subrequest A:
intent: private company read
risk: PRIVATE_READ
required gates: identity + permission + audit

Subrequest B:
intent: approve payroll
risk: PROTECTED_ACTION
required gates: authority + confirmation + simulation + audit

Output:

I can check payroll if you have access. Approval is protected and requires verified authority plus an approved payroll simulation.

16. RequestDecisionPacket

The lattice should produce a structured packet.

RequestDecisionPacket fields

request_decision_id
turn_id
source_input_ref
semantic_proposal_refs
primary_lane
subrequests
intent_type
data_scope
freshness_need
action_effect
risk_level
required_gates
canonical_owner
secondary_owners
presentation_mode
identity_required
permission_required
tenant_scope_required
memory_required
websearch_required
private_data_required
tool_required
file_required
authority_required
confirmation_required
simulation_required
audit_required
clarification_required
split_required
protected_uncertainty_fail_closed
provider_governance_required
reason_codes
confidence
known_uncertainties

Subrequest fields

subrequest_id
parent_request_decision_id
intent_type
data_scope
action_effect
risk_level
required_gates
canonical_owner
allowed_to_proceed
blocked_reason
presentation_instruction

17. How This Fits Current Architecture

Fits GHCS

GHCS provides semantic proposal and PH1.X validation.

The Decision Lattice becomes the structured output of that validation.

Fits Identity + Access + Authority Spine

The lattice decides when identity, access, tenant scope, authority, confirmation, and simulation are needed.

Fits PH1.E Search Intelligence

The lattice decides when search is needed and which search lane should handle it.

Fits PH1.M Human Memory Core

The lattice decides when memory is needed and whether fresh, day, topic, deep, or permanent governed memory might apply.

Fits PH1.WRITE

The lattice tells PH1.WRITE which presentation mode is needed.

Fits Quick Assist + Selene

The lattice routes confusion, ambiguity, reassurance, and guided help into Quick Assist / Selene tone where lawful.

Fits Protected Execution

The lattice prevents protected actions from sneaking through normal public answer paths.

18. Upgrade Required In Current Architecture

This design should be added as a dedicated master architecture document.

It should later be reconciled into:

Selene Final Overall Architecture Build Plan
Selene Overall Repo-Truth Activation Pack
PH1.X activation plan
PH1.WRITE activation plan
PH1.E Search Intelligence activation plan
Identity + Access + Authority activation plan
PH1.M Human Memory activation plan
Enterprise Operations activation plan
Old Compatibility Path Retirement plan

Do not rewrite all plans immediately after adding this document.

Add the document now.

Mark it pending Grand Architecture Reconciliation.

Later, during Grand Architecture Reconciliation, Codex must integrate it into phases/slices.

19. Build Strategy

Do not build everything at once.

Recommended future build sequence:

Build 0 — Decision Lattice Repo-Truth Activation Pack

Map:

current PH1.X lane logic
current protected-risk classifier
current private data/access paths
current search decision paths
current memory routing paths
current tool/file routing paths
current Adapter shortcuts
current Desktop/iPhone risks
current business execution paths
current tests/evals

Build 1 — RequestDecisionPacket / Repo Equivalent

Create or map the central packet.

No behavior changes beyond packet proof unless approved.

Build 2 — Semantic Proposal To Lattice Mapping

Convert GPT-5.5 / SemanticInterpreterProvider proposals into validated lattice decisions.

Build 3 — 5-Lane Business Risk View

Implement the human/business summary lanes:

normal public
public business knowledge
private company read
protected execution
mixed/ambiguous

Build 4 — Mixed Request Splitter

Split:

public explanation + protected action
private read + protected execution
search + protected execution
memory recall + action request

Build 5 — Private Company Read Gate

Add explicit read-only private business data lane.

Must require:

identity
permission
tenant scope
audit

No simulation unless mutation is requested.

Build 6 — Protected Execution Fail-Closed Integration

Ensure protected execution always requires:

authority
confirmation
simulation
audit

Build 7 — PH1.WRITE Presentation Modes From Lattice

Map lattice presentation modes into PH1.WRITE output modes.

Build 8 — Eval Matrix And JD Live Proof

Test all lanes and mixed cases.

20. Hard Rules

No phrase patches.
No keyword routing as architecture.
No Desktop semantic routing.
No Adapter semantic routing.
No private data through normal chat.
No protected execution through public answer.
No company data read without identity/access/tenant/audit.
No simulation required for harmless public business knowledge.
No overblocking public business questions.
No executing mixed requests without splitting.
No protected uncertainty proceeding.

21. Success Standard

Selene’s request routing is successful when:

normal questions feel instant and natural
public business questions are not overblocked
current public facts route to search
private company reads require access and audit
protected actions require authority + simulation
mixed requests are split cleanly
ambiguous risky requests ask short clarifications or fail closed
PH1.WRITE explains outcomes naturally
Desktop/iPhone only render
Adapter only transports
old deterministic shortcuts retire after proof

22. Final Design Summary

The simple 5-lane model is valuable.

The Decision Lattice makes it world-class.

The correct architecture is:

GPT-5.5 proposes messy human meaning.
PH1.X validates the Global Request Decision Lattice.
The lattice produces lane, gates, owner, split, and presentation mode.
Stack owners perform their governed work.
PH1.WRITE explains the result naturally.
Protected execution only happens through authority + simulation.

This is how Selene avoids deterministic phrase rubbish while still protecting business systems with deterministic gates.

It makes Selene smooth for humans and safe for enterprise work.
