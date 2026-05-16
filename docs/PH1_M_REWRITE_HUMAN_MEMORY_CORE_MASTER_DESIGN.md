PH1.M Human Memory Core — Master Design

1. Core principle

Selene should not “search sessions” like software.

Selene should remember like a person.

Internally, Selene can use:

sealed records

indexes

recall packets

audit refs

topic graphs

memory scoring

freshness scoring

memory trust levels

privacy gates

evidence packets

But externally, Selene should behave like:

I remember.
We spoke about that.
Earlier today we were working on...
Yesterday we decided...
A while back we discussed...
Let’s continue from there.

PH1.M must not be a search engine over old sessions.

PH1.M must become Selene’s governed human memory brain.

The memory lifecycle is:

notice → encode → consolidate → connect → recall → continue → update → forget/decay

The current design direction is right, but it must evolve from recall-only into a complete memory life-cycle system.

2. One central memory brain

There must be one memory authority:

PH1.M Memory Core

No other engine should build its own memory.

Not Desktop.Not PH1.X.Not PH1.E.Not Adapter.Not PH1.WRITE.

They all call PH1.M.

PH1.M may internally use many storage layers, indexes, topics, summaries, and audit records, but every remembered-context decision must pass through one governed PH1.M interface.

PH1.M internal modules

1. PH1.M.RECALL_ORCHESTRATOR

This is the central gateway.

Every memory request goes through it.

It decides:

is this fresh memory?

today memory?

recent memory?

topic memory?

deep memory?

permanent governed memory?

should Selene continue automatically?

should Selene ask a clarification?

should Selene ignore memory and answer normally?

should Selene dig deeper?

should Selene avoid memory because it may be stale, private, ambiguous, or unsafe?

This prevents scattered memory logic.

2. PH1.M.ENCODING_ENGINE

Selene must not remember everything equally.

Every conversation turn should be judged:

Was this important?
Was it a decision?
Was it a preference?
Was it an open task?
Was it a correction?
Was it emotional/frustrated?
Was it a project direction?
Was it temporary?
Was it private/sensitive?

Purpose:

turns raw conversation into meaningful memory candidates

This creates memory quality.

Without encoding, PH1.M becomes search over old text, not human memory.

3. PH1.M.SALIENCE_ENGINE

Humans remember important things better.

Selene needs salience scoring:

high importance
medium importance
low importance
temporary
discard/no memory

High salience examples:

“Remember this”

project decisions

Codex laws

user preferences

unresolved blockers

customer commitments

corrections

repeated topics

Low salience examples:

casual filler

noise

temporary wording

one-off jokes

failed transcript fragments

4. PH1.M.CONSOLIDATION_ENGINE

Human memory changes after the moment passes.

Selene should consolidate memory after:

conversation ends
Selene sleeps
topic changes
day ends
task completes
important correction happens

It should convert raw turns into:

topic summaries
decisions
open tasks
preferences
project state
relationship context
rejected options
next likely step

This is critical. Without consolidation, Selene will always need to dig through raw history.

5. PH1.M.FRESH_MEMORY

This is the “human just-now memory.”

Example:

User asks:

What time is it in New York?

Selene answers.

30 seconds pass.

Selene sleeps.

User wakes Selene and says:

What about Sydney?

Selene should understand this naturally.

That is fresh memory, not old archive search.

Fresh memory should cover:

just now

before sleep

earlier in the same active flow

today’s most recent topic

last unresolved question

last tool intent

last important entity

last writing artifact

last user correction

This is the most important human-feeling layer.

6. PH1.M.DAY_MEMORY

Humans remember today differently from last month.

Selene should have natural day memory:

this morning

earlier today

this afternoon

tonight

yesterday

the other day

past few days

This allows:

What did we decide this morning?
What were we fixing yesterday?
What did Codex say earlier?

Selene should answer naturally, not with “archive search result.”

7. PH1.M.TOPIC_MEMORY

This is where Selene remembers by topic.

Examples:

Japan trip

Tamburlaine

PH1.M memory

Stage 6 sleep/wake

Desktop thin client

Codex smoke testing

OpenAI TTS

wake privacy

Topic memory should know:

what the topic is

what was decided

what is still open

what changed over time

which older discussions matter

which details were rejected

what the next step likely is

This is where Selene becomes powerful.

8. PH1.M.TOPIC_GRAPH

Topic memory should not just be text search.

Selene should build a living topic graph.

Example:

Japan trip
 ├── Tokyo
 ├── Kyoto
 ├── hotels
 ├── food
 ├── transport
 └── unresolved: hotel shortlist

Another example:

Selene Desktop
 ├── wake
 ├── TTS
 ├── PH1.X
 ├── PH1.M
 ├── Codex laws
 └── current blocker

This makes memory feel like understanding, not searching.

9. PH1.M.DEEP_RECALL

This handles older memory.

Not instant like fresh memory.

If user asks:

What did we say months ago about Tamburlaine?

Selene may need to “think back.”

User-facing response should feel natural:

I remember the main direction. We were talking about a pilot, with business operations first and wine-making workflows later. Let me pull the older details together.

Internally PH1.M searches deeper history.

Externally Selene still speaks like a person.

10. PH1.M.PERMANENT_GOVERNED_MEMORY

This stores stable long-term facts.

Examples:

JD wants direct answers.

Desktop is not the brain.

Protected execution requires simulation + authority.

Codex must not use Python.

Old code must be removed when replaced.

Selene memory should feel human, not like session search.

This memory must support:

save

review

update

forget

audit

consent / policy

sensitive-data safeguards

11. PH1.M.CONTINUATION_GATE

This decides whether Selene should continue from memory automatically.

Inputs:

user wording

freshness

topic match

confidence

risk

ambiguity

active PH1.X state

remembered evidence

Outputs:

CONTINUE_AUTOMATICALLY
ASK_CLARIFICATION
ANSWER_NORMALLY
NO_MEMORY_MATCH

Important rule:

Fresh obvious memory can continue automatically.

Older ambiguous memory should ask naturally.

Example:

Continue the Japan trip plan.

Selene continues automatically.

Example:

Japan.

Selene may ask which Japan topic.

12. PH1.M.MEMORY_POSTURE_ENGINE

Selene must know why the user is invoking memory.

The user may be:

continuing
asking what was said
asking to restart
testing memory
changing topic
using memory as comparison
asking for a decision we made
asking for old details

This prevents over-memory.

Example:

User: I want to plan France.

Selene may use the Japan trip structure as helpful context, but she should not hijack the conversation into Japan unless useful.

13. PH1.M.FRESHNESS_GRADIENT

Selene should rank memories by human-like freshness.

Internal freshness order:

just now
earlier this conversation
before sleep
earlier today
yesterday
past few days
recent weeks
older topic memory
permanent governed memory

Selene should not expose this as technical tiers.

She should speak naturally:

I remember that from earlier.
We spoke about that yesterday.
A while back, we discussed...

14. PH1.M.MEMORY_EVIDENCE_PACKET

Every memory use should produce a packet.

Example:

MemoryEvidencePacket:
- memory_type: fresh / today / topic / deep / permanent
- topic_label: Japan trip
- age_label: earlier today
- confidence: high
- evidence_refs: [...]
- continuation_allowed: true
- clarification_needed: false
- user_facing_summary: We were planning Tokyo and Kyoto...
- active_context_allowed: true
- user_facing_recall_style: I remember / We spoke about / Earlier today / Yesterday / A while back
- trust_level: verbatim user instruction / Codex report / inferred summary / assistant suggestion / unverified idea / outdated note
- privacy_status: allowed / restricted / sensitive / requires identity / unavailable

This gives other engines memory safely without letting them invent memory.

15. PH1.M.CONFLICT_AND_STALENESS_CHECKER

Human memory can be wrong or outdated. Selene must handle that.

It should detect:

old decision replaced by newer decision

contradictory memories

unresolved topic

stale project status

low confidence

missing proof

user changed preference

topic was abandoned

issue was resolved

Codex result superseded old plan

Memory records should support:

superseded_by
current_status
still_valid
resolved_at
conflicts_with
confidence_reason

Selene should say naturally:

We talked about that before, but I think the later decision changed it.

16. PH1.M.MEMORY_TRUST_ENGINE

Not all memory is equally trustworthy.

Selene needs trust levels:

verbatim user instruction
Codex report
inferred summary
assistant suggestion
unverified idea
old/outdated note

A direct user instruction should rank higher than an inferred assistant summary.

17. PH1.M.MEMORY_PRIVACY_GATE

Before storing or recalling, PH1.M should ask:

Is this sensitive?
Can this be remembered?
Can this be shown now?
Can this be used with this speaker?
Can this be used in this workspace?

This matters later for Voice ID, business users, and protected work.

18. PH1.M.MEMORY_USE_POLICY

Selene should not always use memory.

Rules:

Use memory when it clearly helps.
Do not use memory when user asks a fresh unrelated question.
Do not over-explain remembered context.
Do not expose old private context unless relevant.
Clarify if multiple remembered topics match.
Continue automatically only when intent is clear.

This keeps Selene human rather than intrusive.

19. PH1.M.NO_RECORD_HANDLER

If Selene does not remember, she should be honest.

Not:

No archive result found.

Better:

I don’t remember us discussing that before.

Or:

I don’t see that in what we’ve covered.

20. PH1.M.HUMAN_MEMORY_EVAL_MATRIX

PH1.M needs a large evaluation matrix.

Positive tests:

New York → sleep → wake → what about Sydney
Continue Japan trip
What did we decide yesterday?
What were we fixing this morning?
What did Mark say?
Do you remember what I said about Desktop?
I changed my mind about that.
Forget that.
That is old, use the newer plan.

Negative tests:

Sydney
Hotels
Mark
Continue
What about that?

These should not blindly pull memory if ambiguous.

How engines use PH1.M

PH1.X

PH1.X owns the live conversation.

It asks PH1.M only when memory may help.

Example:

User: What about Sydney?

PH1.X asks PH1.M:

Is this continuing the fresh topic?

PH1.M answers:

Yes, likely continuing New York time question.

Then PH1.X continues naturally.

PH1.L

PH1.L owns sleep/wake boundary.

When Selene sleeps, PH1.L tells PH1.M:

Preserve fresh continuation memory.
Clear active capture state.
Wake remains ready.

PH1.L should not make the user think about “closing sessions.”

User-facing:

Selene sleeps.
User wakes her.
Selene remembers.

PH1.E

PH1.E owns time/weather/tools.

PH1.E should not carry memory itself.

It receives context packets.

Example:

“What about Sydney?” = time follow-up

Then PH1.E performs the time answer.

PH1.WRITE

PH1.WRITE turns memory into human language.

It should avoid robotic wording.

It should write:

I remember. We were planning Tokyo and Kyoto.

Not:

I found a session.

PH1.M should pass PH1.WRITE recall style, evidence, confidence, age label, and topic summary so PH1.WRITE can sound human without inventing memory.

Desktop

Desktop renders only.

It can show:

remembered topic list

old conversation view

current conversation

“what are you trying to remember?” search bar

Desktop must not search memory or decide relevance.

Human memory behavior rules

Rule 1 — Fresh memory should feel instant

If the topic was just discussed, Selene should not behave like she forgot.

Example:

New York time
30 seconds pass
wake
what about Sydney

Selene should continue naturally.

Rule 2 — Today memory should feel easy

If it was earlier today, Selene should recall quickly.

Earlier today we were working on Desktop voice timing.

Rule 3 — Older topic memory should feel like remembering by subject

We spoke about Tamburlaine before.

Selene should retrieve the topic, not make the user manage sessions.

Rule 4 — Deep memory can take longer

Older memories may require deeper recall.

Selene can say:

Let me think back.

But she should still avoid machine language.

Rule 5 — Old memory must not silently pollute current context

Remembering is not the same as continuing.

Selene can continue automatically only when intent is clear.

Rule 6 — Memory must be selective

Selene should notice what matters, ignore noise, and avoid remembering random low-value details forever.

Rule 7 — Memory must be updateable

Newer, stronger, or more direct memory can supersede older memory.

Rule 8 — Memory must be human-facing

Selene should speak like she remembers, not like she queried storage.

Final upgraded architecture

PH1.M Human Memory Core
  ├── Recall Orchestrator
  ├── Encoding Engine
  ├── Salience Engine
  ├── Consolidation Engine
  ├── Fresh Memory
  ├── Day Memory
  ├── Topic Memory
  ├── Topic Graph
  ├── Deep Recall
  ├── Permanent Governed Memory
  ├── Continuation Gate
  ├── Memory Posture Engine
  ├── Freshness Gradient
  ├── Conflict + Staleness Checker
  ├── Memory Trust Engine
  ├── Memory Privacy Gate
  ├── No-Record Handler
  ├── Memory Evidence Packet
  ├── Memory Use Policy
  └── Human Memory Eval Matrix

What makes it human

The old design says:

Selene can recall memory.

The upgraded system says:

Selene notices what matters.
Selene consolidates it.
Selene understands topic continuity.
Selene recalls by freshness and meaning.
Selene knows when not to use memory.
Selene speaks like she remembers.
Selene updates old memories when newer ones replace them.

That is the difference between search and human memory.

Build plan for PH1.M

Build 0 — PH1.M Repo Truth And Gap Audit

Before coding, Codex must inspect:

existing PH1.M contracts

current recent recall

storage digest rows

adapter recall route

memory ledger

archive surfaces

PH1.X memory interactions

Desktop memory UI if any

duplicate recall paths

current memory tests

existing storage schema

existing provenance/audit records

Output:

What exists
What works
What is duplicated
What is stale
What matches the new design
What is missing
What should be preserved
What should be removed
What must not be rebuilt

No coding yet.

Build 1 — PH1.M Memory Core Contracts

Add central types:

MemoryRecallRequest

MemoryEvidencePacket

MemoryContinuationDecision

MemoryAgeLabel

MemoryConfidence

MemoryScope

MemorySourceRef

MemoryUserFacingSummary

MemorySalience

MemoryTrustLevel

MemoryPrivacyStatus

MemoryFreshness

MemoryPosture

MemoryConflictStatus

This gives all engines one shared memory language.

Build 2 — PH1.M Recall Orchestrator

Build the central gateway.

All recall calls go through it.

It routes to:

fresh memory

today memory

topic memory

deep recall

permanent governed memory

Build 3 — Encoding + Salience + Consolidation

Build the memory intake pipeline.

It should:

classify important turns

detect decisions

detect preferences

detect open tasks

detect corrections

ignore low-value noise

consolidate raw turns into meaningful memory candidates

Build 4 — Fresh Memory Continuation

This is the first human-feel proof.

Required proof:

New York → sleep → wake → what about Sydney

Expected:

Selene continues naturally.

No stale context leak.

No robotic “new session” behavior.

Build 5 — Topic Memory + Topic Graph

Build topic clustering and continuity.

Examples:

Japan trip

Tamburlaine

PH1.M memory

Desktop thin client

Selene should continue clear topics automatically and clarify ambiguous ones.

Build 6 — Natural Memory Language via PH1.WRITE

PH1.WRITE uses memory packets to speak naturally.

No “session search.”

No “archive result.”

Use human recall language.

Build 7 — Deep Recall

Older memories.

Month-old, year-old, project-history recall.

Slower allowed.

Still human-facing.

Build 8 — Trust, Privacy, Conflict, And Staleness

Build:

memory trust levels

privacy gate

conflict detection

superseded memory handling

stale memory warning

forget/update safety

Build 9 — Natural Memory UI

Simple search bar only.

Placeholder:

What are you trying to remember?

Results should feel like remembered topics, not database records.

Build 10 — Human Memory Eval Matrix

Build the full test matrix:

fresh memory continuation

today memory

yesterday memory

topic memory

deep recall

conflict/staleness

no-record honesty

negative ambiguity cases

privacy/sensitive recall gates

no PH1.X active context pollution

What Codex must not do

Codex must not:

rewrite PH1.M blindly

create PH1.M.FRESH disconnected from existing PH1.M

put memory into Desktop

create adapter shortcut memory

let PH1.X become memory

let PH1.E carry stale memory

expose “sessions” to the user

load all history into active context

make memory bypass protected execution

create a second recall system

treat memory as simple text search

skip repo truth inventory

skip provenance/audit handling

Final architecture rule

Selene has one governed memory brain: PH1.M.

PH1.X owns the live moment.
PH1.L owns sleep and wake boundaries.
PH1.E owns tools.
PH1.WRITE owns human language.
Desktop renders only.
Storage files everything immutably.

All remembered context flows through PH1.M memory packets.

Do not rewrite PH1.M from scratch.

Do this:

PH1.M repo truth inventory
→ central memory contracts
→ recall orchestrator
→ encoding / salience / consolidation
→ fresh memory proof
→ topic memory and topic graph
→ human language via PH1.WRITE
→ deep recall
→ trust / privacy / conflict
→ natural memory UI
→ human memory eval matrix

That is how Selene becomes human-like without becoming a pile of disconnected memory hacks.

The key sentence:

PH1.M must not be a search engine over old sessions. It must be Selene’s governed human memory brain.
