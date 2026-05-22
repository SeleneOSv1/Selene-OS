Selene PH1.N — Universal NLP Intelligence + Meaning Unravelling Master Design

DOCUMENT TYPE:
DEDICATED MASTER DESIGN / PH1.N NLP + MEANING UNRAVELLING ARCHITECTURE

TASK:
SELENE_PH1N_UNIVERSAL_NLP_INTELLIGENCE_MEANING_UNRAVELLING_MASTER_DESIGN

BUILD CLASS:
ARCHITECTURE / NLP INTELLIGENCE / TEXT NORMALIZATION / INTENT CANDIDATE GENERATION / ENTITY + SLOT UNDERSTANDING / OPENAI-ASSISTED HUMAN MEANING STACK

STATUS:
MASTER DESIGN FOR FUTURE CODEX GRAND ARCHITECTURE RECONCILIATION

CONTROLLING DOCUMENTS:
1. AGENTS.md
2. Selene Master Architecture Build Set
3. Selene Final Overall Architecture Build Plan
4. Selene Overall Repo-Truth Activation Pack
5. Selene Global Human Conversation Spine Master Architecture
6. Selene Global Request Decision Lattice + 5-Lane Business Risk View Master Design
7. Selene Universal Language Intelligence + Voice Capture Master Design
8. Selene PH1.WRITE — Structured Writing + Human Presentation Master Design
9. Selene PH1.M Human Memory Core Master Design
10. Selene Search Intelligence Lane — Revised Enterprise Websearch Master Design
11. Selene Identity + Access + Authority Spine Master Architecture
12. Selene Voice Identity + Human Presence Master Design
13. Selene Full Duplex and Barge-In Enterprise Voice Architecture
14. Celine Persona + Emotional Presentation Stack
15. Conversational Experience + Quick Assist Stack

PURPOSE:
Define PH1.N as Selene’s universal NLP and meaning-unravelling layer: the layer that turns messy captured text, broken grammar, slang, spelling errors, phonetic mistakes, rambling user language, mixed-language fragments, business shorthand, domain vocabulary, entity ambiguity, and partial references into structured meaning candidates for PH1.X to validate through the Request Decision Lattice.

0. Master Law

PH1.N is not the final brain.

PH1.N is the meaning unraveller.

The global law is:

OpenAI / GPT-5.5 helps understand messy human language.
PH1.N turns messy language into structured candidates.
PH1.X validates the actual request decision.
Stack owners execute or refuse through their own gates.
PH1.WRITE explains the result naturally.

PH1.N may propose.

PH1.N must not decide authority.

PH1.N must not execute tools.

PH1.N must not approve protected actions.

PH1.N must not become a phrase-patch router.

1. Why PH1.N Is Separate From Universal Language Intelligence

Universal Language Intelligence owns the capture/language side:

voice/audio capture
transcript quality
source language
output language
topic language
STT mismatch
speaker-language binding
TTS exact speech

PH1.N owns the meaning-normalization side:

what did this messy utterance probably mean?
what are the possible intents?
what are the entities?
what are the slots?
what is ambiguous?
is there protected risk?
what clarification would help?
what should PH1.X validate?

Simple split:

PH1.C / PH1.LANG:
  What did the user say, and in what language?

PH1.N:
  What might the user mean?

PH1.X:
  What validated route is lawful?

PH1.N sits between language capture and request routing.

2. Target Product Standard

Selene should understand human mess at a world-class level.

Selene must handle:

broken English
bad grammar
slang
rambling input
half-finished thoughts
self-corrections
wrong spelling
bad punctuation
mixed language
accent-driven transcript mistakes
phonetic mistakes
misheard names
company/person/place/product ambiguity
business shorthand
technical terms
Selene project vocabulary
private company terms where access allows
public entity confusion
casual commands
unclear protected requests

Selene must not pretend every messy utterance is clear.

Correct behavior:

understand generously for harmless public conversation
ask short clarification when needed
preserve original text
track candidates and rejected candidates
fail closed for protected risk
never execute from fuzzy meaning

3. OpenAI Role vs Selene Role

3.1 OpenAI / GPT-5.5 may assist with

messy language understanding
broken grammar interpretation
slang interpretation
semantic role extraction
intent candidates
entity candidates
slot candidates
phonetic candidate suggestions
clarification wording
normalization suggestions
translation-aware meaning help
business-language interpretation
TTS/display-friendly restatement

3.2 OpenAI / GPT-5.5 must not own

final lane decision
private data access
memory permission
tool permission
authority
simulation
protected execution
entity truth
source truth
company database truth
final PH1.WRITE output
audit truth

Provider intelligence is a proposal source.

Selene validation is the product architecture.

4. PH1.N Position In The Global Runtime

Correct runtime flow:

User voice/text/file/OCR/meeting input
→ PH1.C transcript quality / text admission
→ PH1.LANG source/output/topic language decision
→ PH1.N NLP normalization and candidate generation
→ SemanticInterpreterProvider / GPT-5.5 proposal where applicable
→ PH1.X Request Decision Lattice validation
→ stack owner gates:
   PH1.E search/tool/file
   PH1.M memory
   Access private data gate
   Authority + Simulation protected gate
   PH1.WRITE presentation
→ PH1.WRITE final answer
→ PH1.TTS / Desktop / Adapter output
→ audit evidence

PH1.N may run before or together with SemanticInterpreterProvider depending on stack design.

It must never bypass PH1.X.

5. PH1.N Core Responsibilities

PH1.N owns candidate-level language understanding.

Responsibilities:

normalize messy utterances
preserve original text
identify intent candidates
identify action candidates
identify entity candidates
identify slot candidates
identify time/date/amount candidates
identify references and pronouns
identify slang meaning candidates
identify business shorthand candidates
identify protected-risk hints
identify ambiguity
produce clarification candidates
produce confidence and reason codes
produce candidate/rejection ledgers
pass structured output to PH1.X

PH1.N must support both typed and transcribed input.

6. PH1.N Non-Responsibilities

PH1.N must not:

make final route decisions
execute tools
perform search
read private memory
read company data
approve access
grant authority
execute protected business actions
write final answer
own Desktop UI
own Adapter behavior
store durable memory
silently correct protected required slots
hardcode real names into production logic

PH1.N is a meaning preparation layer.

PH1.X is the route validation layer.

7. PH1.N Internal Modules

PH1.N should be designed as coordinated modules, not one giant function wearing a false mustache.

7.1 PH1.N.INPUT_NORMALIZER

Purpose:

clean surface-level noise without changing meaning
preserve original text
normalize whitespace/punctuation where safe
preserve raw transcript refs

Rules:

do not alter protected slots silently
do not replace names without candidate evidence
do not erase uncertainty

7.2 PH1.N.BROKEN_LANGUAGE_UNRAVELLER

Purpose:

interpret broken grammar
repair word order for candidate meaning
understand low-fluency English
understand concise/rushed commands

Example:

me need boss report from meeting
→ possible intent: create report from meeting transcript
→ output: candidate, not final action

7.3 PH1.N.SLANG_AND_COLLOQUIAL_ENGINE

Purpose:

map slang and casual phrases into candidate intents

Examples:

hook me up with a summary
→ summarize request

shoot him the email
→ possible draft/send email request
→ send is side-effecting; PH1.X must validate

what’s the damage on payroll
→ possible payroll cost question

sort Tom out
→ ambiguous; maybe payroll, roster, issue, payment, task

7.4 PH1.N.SEMANTIC_ROLE_ENGINE

Purpose:

Extract roles:

actor
action
target
object
recipient
amount
time
location
source
constraint
requested output

Example:

Send the invoice to Mark tomorrow.
action: send
object: invoice
recipient: Mark
time: tomorrow
action_effect: external send / possibly protected

7.5 PH1.N.INTENT_CANDIDATE_ENGINE

Purpose:

Produce multiple possible intent candidates when language is ambiguous.

Example:

Sort out Tim’s pay.
Candidate 1: explain payroll issue
Candidate 2: check Tim payroll data
Candidate 3: adjust Tim pay
Candidate 4: approve payment

PH1.X decides lane and gates.

7.6 PH1.N.ENTITY_CANDIDATE_ENGINE

Purpose:

Generate and rank entity candidates.

Entities include:

people
companies
places
products
customer names
supplier names
project names
Selene architecture terms
business systems
technical systems

It must distinguish:

Selene assistant
Celine persona
real person named Celine
Tamburlaine company
Tumbling as ordinary word
PH1.X engine
PH one X as bad transcript

7.7 PH1.N.PHONETIC_AND_SPELLING_ENGINE

Purpose:

Repair misspellings and misheard terms as candidates.

Approach:

phonetic similarity
edit distance
context scoring
domain lexicon
memory/entity graph where allowed
public source evidence where needed
rejection ledger
clarification if low confidence

Examples:

Tumbling organic wine → Tamburlaine Organic Wines candidate
net sweet → NetSuite candidate
p h one ex → PH1.X candidate
Celine → Selene only in assistant-address context

7.8 PH1.N.DOMAIN_LEXICON_ENGINE

Purpose:

Use governed domain vocabulary.

Domains:

Selene architecture
business operations
wine industry
payroll
accounting
HR
inventory
customer service
software/API terms
company-specific terms where access allows

Rules:

lexicon is governed and scoped
private company lexicon requires access rules
no uncontrolled memory leakage
no hardcoded real search names in tests/code

7.9 PH1.N.REFERENCE_RESOLUTION_HINT_ENGINE

Purpose:

Produce reference candidates for:

it
that
this
the earlier one
the second point
the previous answer
Sydney
him
her
the report
the file
the payroll thing

PH1.N may propose references.

PH1.X / PH1.M / PH1.E validate final target.

7.10 PH1.N.SLOT_CANDIDATE_ENGINE

Purpose:

Extract required fields for downstream validation.

Slots:

person
amount
date/time
company
location
file
customer
supplier
invoice
payroll period
product
language
format
recipient

Protected slot rule:

PH1.N may propose a slot.
Protected execution cannot use uncertain slots without confirmation.

7.11 PH1.N.AMBIGUITY_ENGINE

Purpose:

Detect uncertainty.

Ambiguity types:

intent ambiguity
entity ambiguity
slot ambiguity
reference ambiguity
language ambiguity
scope ambiguity
protected-risk ambiguity
speaker ambiguity handoff

Output:

ambiguity reason
candidate list
confidence
clarification needed yes/no
safe default

7.12 PH1.N.PROTECTED_RISK_HINT_ENGINE

Purpose:

Flag possible protected action or private data read.

Examples:

pay Tom
approve payroll
send salary file
change roster
give him access
refund this customer
show Tim salary

PH1.N produces risk hints only.

PH1.X / Authority / Simulation validate actual protected route.

7.13 PH1.N.CLARIFICATION_CANDIDATE_ENGINE

Purpose:

Generate short useful clarification questions.

Bad:

Intent confidence below threshold. Please restate.

Good:

Do you want me to check Tim’s pay details, or change something about his pay?

Clarification wording goes through PH1.WRITE / Quick Assist.

7.14 PH1.N.NORMALIZED_MEANING_SUMMARY_ENGINE

Purpose:

Produce a safe internal normalized meaning summary.

Example:

Original: me need boss report from meeting
Normalized summary: User likely wants a report drafted from a meeting transcript.

This summary is internal and not authority.

8. Required PH1.N Packets

Codex must reuse current repo names where they exist and map these logical packets to repo equivalents during activation.

8.1 NlpInputPacket

turn_id
session_id
source_modality
raw_text
transcript_ref optional
language_packet_ref optional
speaker_posture_ref optional
source_context_refs
input_origin

8.2 NormalizedUtterancePacket

original_text
normalized_text_candidates
normalization_confidence
preserved_terms
changed_terms
uncertain_terms
language_notes
protected_field_change_blocked

8.3 IntentCandidatePacket

intent_candidate_id
intent_type
operation
confidence
reason
required_gates_hint
protected_risk_hint
owner_candidate

8.4 EntityCandidatePacket

entity_candidate_id
raw_mention
normalized_entity_candidate
entity_type
confidence
source_of_candidate:
  transcript
  lexicon
  memory
  public_evidence
  phonetic_repair
  provider_suggestion
rejected_candidates
clarification_needed

8.5 SlotCandidatePacket

slot_name
raw_value
normalized_value_candidate
confidence
protected_slot
confirmation_required
source_ref

8.6 ReferenceCandidatePacket

raw_reference
candidate_targets
confidence
freshness_hint
requires_ph1x_validation
requires_ph1m_validation
requires_ph1e_validation

8.7 AmbiguityPacket

ambiguity_type
candidate_refs
reason
confidence
clarification_required
protected_uncertainty
safe_default

8.8 ProtectedRiskHintPacket

risk_hint_id
possible_protected_action
possible_private_read
possible_external_side_effect
possible_business_mutation
required_gate_hints
confidence
reason

8.9 ClarificationCandidatePacket

clarification_type
question_candidate
options
short_form
reason
owner_to_validate

8.10 NlpEvidenceTracePacket

turn_id
input_hash
normalization_refs
intent_candidate_refs
entity_candidate_refs
slot_candidate_refs
ambiguity_refs
protected_risk_refs
clarification_refs
provider_refs
rejection_refs

9. Interaction With OpenAI / GPT-5.5

PH1.N should use OpenAI through Provider Governance where useful.

OpenAI-assisted NLP calls must be:

schema-bound where possible
provider-off safe
fake-provider testable
malformed-output rejectable
privacy/data-egress governed
model-policy governed
audit traced

OpenAI may return:

intent candidates
entity candidates
slot candidates
normalized paraphrase
slang interpretation
clarification question suggestion
protected-risk hint
language nuance

OpenAI output must be validated by Selene.

If provider is off:

PH1.N may use deterministic safe minimal normalization
or ask clarification
or route public simple cases safely

No hidden provider call.

10. Protected Execution Safety

PH1.N must be generous for understanding but strict for execution.

Rules:

Fuzzy language can create candidates.
Fuzzy language cannot execute protected actions.
Uncertain entity can create candidates.
Uncertain entity cannot fill protected required slots.
Slang can suggest protected risk.
Slang cannot approve protected execution.
Broken English can be understood.
Broken English cannot bypass confirmation.

Examples:

me need pay thing Tom tomorrow
→ protected risk hint
→ clarification / fail closed
→ no execution

sort Tim out
→ ambiguous
→ ask whether user means check, explain, draft, or change

shoot him the email
→ possible external send
→ draft may be allowed
→ send requires appropriate gate

11. Relation To Request Decision Lattice

PH1.N feeds the Global Request Decision Lattice.

PH1.N provides:

intent candidates
data-scope hints
freshness hints
action-effect hints
risk hints
required-gate hints
presentation hints
clarification hints

PH1.X validates:

final lane
required gates
canonical owner
split/clarification
protected fail-closed
presentation mode

PH1.N is upstream candidate generation.

PH1.X is downstream validation.

12. Relation To PH1.WRITE

PH1.N helps PH1.WRITE by producing clean meaning packets.

PH1.WRITE should receive:

validated directive from PH1.X
safe normalized meaning summary
clarification candidates when needed
entity/slot uncertainty notes
language/presentation hints

PH1.WRITE should not invent normalization after the fact.

PH1.WRITE may phrase the clarification naturally.

Example:

PH1.N: ambiguous whether “sort Tim out” means check pay or change pay.
PH1.X: clarification required.
PH1.WRITE: Do you want me to check Tim’s pay details, or change something about his pay?

13. Relation To Universal Language Intelligence

Universal Language Intelligence handles:

source language
output language
topic language
STT mismatch
per-turn language reset
speaker-language binding
voice capture quality

PH1.N handles:

meaning normalization
broken grammar
slang
spelling/phonetic candidates
entity candidates
slot candidates
protected-risk hints

They must work together.

Example:

Input: Can you explain 这个功能 in simple English?
PH1.LANG: input mixed, output English
PH1.N: intent explain, target = Chinese phrase/function
PH1.X: route public explainer
PH1.WRITE: answer in English, preserve Chinese phrase

14. Relation To PH1.M Human Memory

PH1.N may use memory-backed entity and topic hints only through PH1.M and access law.

Rules:

PH1.N does not search memory directly.
PH1.M owns memory evidence.
PH1.N may consume allowed memory/entity hints.
Unknown speaker cannot use private memory entity hints.
Private company lexicon requires access scope.

Example:

User says: that Codex law thing
PH1.M may provide allowed project memory context
PH1.N proposes entity/topic candidate
PH1.X validates target

15. Relation To PH1.E Search Intelligence

PH1.N may help PH1.E by producing:

requested entity
claim type
role target
query intent
freshness hint
public/protected boundary
search-needed hint

PH1.E owns actual search planning, provider calls, source acceptance, and claim verification.

PH1.N must not accept sources.

16. Relation To Celine + Quick Assist

Celine and Quick Assist may use PH1.N outputs to sound human.

Examples:

PH1.N detects user is confused.
Quick Assist explains next step.

PH1.N detects harsh/rushed wording but harmless intent.
Celine responds calmly and directly.

PH1.N detects profanity inside control command.
Barge-in/PH1.K handles control; PH1.WRITE avoids moralizing.

Celine must not turn personality into authority.

17. Evaluation Matrix

PH1.N requires a strong eval matrix.

17.1 Broken English

me need boss report from meeting
what time Tokyo now please
I no understand this make simple

17.2 Slang

hook me up with a summary
shoot him the email
what’s the damage on payroll
sort Tim out

17.3 Misspellings / Phonetic

Tumbling organic wine
net sweet
p h one ex
Celine vs Selene

17.4 Ambiguous business requests

fix payroll
sort the roster
handle the customer issue
send that thing to Mark

17.5 Protected fuzzy commands

pay Tom tomorrow
increase Tim thing
approve it
yes do it

17.6 Mixed-language inputs

Explain 这个功能 in simple English
帮我 summarize this meeting
Teach me Chinese in English

17.7 Reference resolution

what about Sydney?
make the second one shorter
send that
continue from there

17.8 Negative tests

Do not infer protected action from vague phrase.
Do not select wrong entity from phonetic guess.
Do not use private memory without access.
Do not hardcode real names.
Do not route by keyword contains.

18. Build Strategy

Do not build PH1.N in one pass.

Build 0 — PH1.N Repo-Truth Activation Pack

Map current:

PH1.N files if present
PH1.C / PH1.LANG interaction
PH1.X lane logic
spelling/correction paths
entity resolver paths
memory/entity hints
search entity paths
Adapter phrase shortcuts
Desktop/iPhone risks
tests/evals

No implementation.

Build 1 — PH1.N Packet / Repo Equivalent Map

Create or map:

NlpInputPacket
NormalizedUtterancePacket
IntentCandidatePacket
EntityCandidatePacket
SlotCandidatePacket
AmbiguityPacket
ProtectedRiskHintPacket
NlpEvidenceTracePacket

Build 2 — Provider-Governed NLP Proposal Shell

Use GPT-5.5 / NLPProvider through Provider Governance.

Proof:

provider-off
fake-provider
malformed rejection
no raw provider authority

Build 3 — Broken Language + Slang Candidate Generation

Generate intent candidates from messy input.

No protected execution.

Build 4 — Entity / Spelling / Phonetic Candidate Engine

General candidate engine.

No phrase patches.

Build 5 — Slot + Reference Candidate Engine

Extract candidates and references.

PH1.X validates.

Build 6 — Ambiguity + Clarification Engine

Produce clarification candidates.

PH1.WRITE phrases them.

Build 7 — Protected Risk Hint Engine

Detect private/protected risk hints.

PH1.X / Authority / Simulation validate.

Build 8 — PH1.X Request Lattice Integration

Pass candidates into RequestDecisionPacket.

Build 9 — PH1.N Eval Certification

Run full eval corpus with unseen paraphrases, synthetic entities, multilingual cases, protected fail-closed, and phrase-patch scans.

19. What Codex Must Not Do

Codex must not:

create phrase-patch intelligence
add keyword contains routing
hardcode real searched names
hardcode customer/person/company/product names
turn PH1.N into final router
let PH1.N execute tools
let PH1.N read private memory directly
let PH1.N approve protected actions
let PH1.N fill protected slots from guesses
put NLP logic in Desktop
put NLP logic in Adapter
create duplicate PH1.X routing brain
claim world-class language unravelling without evals

20. Success Standard

PH1.N is successful when Selene can:

understand messy user language generously
preserve original text
produce multiple meaning candidates
rank candidates with confidence
identify entities and slots without hardcoding
detect ambiguity
ask short useful clarifications
flag protected risk without executing
feed PH1.X Request Decision Lattice cleanly
support PH1.WRITE human response
avoid phrase patches
pass unseen paraphrase evals
preserve protected fail-closed law

21. Final Architecture Sentence

PH1.N is Selene’s meaning unraveller.

It turns human chaos into structured candidates.

It does not decide power.

The final architecture is:

OpenAI helps interpret the mess.
PH1.N structures the possible meanings.
PH1.X validates what is lawful.
Stack owners do their governed work.
PH1.WRITE explains it like a human.

That is how Selene becomes capable of understanding broken English, bad grammar, slang, spelling mistakes, phonetic chaos, rambling users, and business ambiguity without becoming a deterministic phrase swamp.
