PH1.X Human Conversation Core — Master Design

1. Core principle

PH1.X owns the live moment.

It answers:

What are we talking about right now?
What does this short follow-up refer to?
Is the user continuing, correcting, changing topic, or starting fresh?
What context can safely carry forward?
Why did the user say it this way?
Is the user asking, instructing, correcting, testing, joking, thinking out loud, frustrated, or continuing?
Should Selene answer, wait, clarify, remember, help, route, refuse, or stay quiet?

PH1.X is not memory.PH1.X is active attention.

PH1.M remembers what happened before.PH1.X understands what is happening now.

The core upgrade is this:

PH1.X must become Selene’s universal live-context and human-interaction engine, not a time/weather follow-up patch.

2. The target behavior

Selene should understand live conversation like a human.

Examples:

User: What time is it in New York?
User: What about Sydney?

PH1.X understands:

same question
same tool type
new city

Another example:

User: Write me a short story.
User: Make it darker.
User: Now make it shorter.

PH1.X understands:

same writing task
modify previous answer
change tone
reduce length

Another example:

User: Draft a message to Mark.
User: Make it warmer.
User: Add that I’ll come back next week.

PH1.X understands:

same draft
style change
content addition
recipient still Mark

This must work across all topics, not only time/weather.

3. PH1.X must not become a collection of patches

Wrong direction:

time follow-up logic
weather follow-up logic
email follow-up logic
story follow-up logic
travel follow-up logic

That becomes messy.

Correct direction:

One universal active context engine
→ one live context packet
→ one human conversation directive
→ all engines use it

PH1.X should understand the pattern of conversation, not just specific topics.

4. PH1.X internal modules

4.1 Active Conversation Frame

This stores the live topic.

It tracks:

current topic
current task
current intent
last user request
last Selene answer type
last entities
last location/person/company/project
last tool used
last writing artifact
pending clarification
open user goal
current language
current modality
current interaction posture
current response shape

This is the “what are we doing right now?” layer.

4.2 Topic Stack

Humans can temporarily shift and come back.

Example:

User: Plan Japan.
User: Wait, what time is it in Sydney?
User: Okay, back to Japan.

PH1.X needs a topic stack:

primary topic: Japan trip
temporary topic: Sydney time
returnable topic: Japan trip

This prevents Selene from losing the main conversation.

4.3 Interaction Posture Engine

PH1.X must detect the user’s conversational posture.

It must distinguish:

asking a question
giving an instruction
correcting Selene
thinking out loud
venting or showing frustration
testing Selene
continuing a topic
changing topic
asking for memory
asking for action
making a joke
speaking casually without requiring action

This prevents Selene from responding like a robot to every utterance.

Selene should understand not only the words, but why the user is saying them in that way.

4.4 Conversation Rhythm Engine

Human conversation has rhythm.

PH1.X must decide when Selene should:

answer directly
pause
acknowledge briefly
ask one short clarification
continue without overexplaining
stop talking
wait for the next thought
keep the answer short
expand into structured explanation

This engine keeps Selene from over-answering, interrupting the user’s flow, or sounding mechanical.

4.5 Reference Resolver

This resolves words like:

it
that
this
same
again
there
him
her
they
the other one
do it
continue
make it shorter
go back
the first one
the one from earlier
what we said before

Example:

User: Write a report.
User: Make it shorter.

PH1.X resolves:

"it" = previous report

4.6 Continuation Gate

This decides whether the user is continuing the current topic.

Outputs:

CONTINUE_CURRENT_TOPIC
MODIFY_PREVIOUS_OUTPUT
CORRECT_PREVIOUS_OUTPUT
ANSWER_NEW_TOPIC
ASK_CLARIFICATION
HAND_OFF_TO_MEMORY
NO_ACTION_REQUIRED

Examples:

"What about Sydney?" → continue
"Make it warmer" → modify
"What is your name?" → new topic
"Japan" → clarify or memory handoff depending freshness

4.7 Slot + Entity Frame

For structured tasks, PH1.X tracks missing slots.

Example:

User: Book a meeting with Mark.

Missing:

date
time
duration
location

If user then says:

Tomorrow at 3.

PH1.X knows that fills the meeting slot.

This applies to:

time
weather
emails
meetings
travel
reports
tasks
products
people
companies
locations
business workflows
planning tasks
writing tasks

4.8 Writing Artifact State

PH1.X must track live writing artifacts.

Examples:

draft email
rewrite answer
short story
report
contract note
Codex instruction
business reply
proposal
message to a person
summary
plan

If user says:

Make it more professional.
Add one paragraph.
Remove the last line.

PH1.X knows which artifact is being changed.

PH1.WRITE handles the writing quality.PH1.X tracks the live artifact being edited.

4.9 Tool Continuity State

PH1.X tracks the live tool pattern.

Examples:

time in New York → what about Sydney
weather in Spain → Barcelona
search Tamburlaine → what about NetSuite
find a file → open that one
summarize this report → make it shorter

PH1.X should produce a context packet for PH1.E:

same tool family
same question type
new entity/location
carry previous constraints

PH1.E performs the tool work.PH1.X supplies live meaning.

4.10 Correction Controller

Humans correct themselves constantly.

Examples:

No, I meant Sydney.
Not weather, time.
Actually make it shorter.
Use Mark, not Michael.
That’s not what I meant.
Not that one.

PH1.X must detect:

correction target
old value
new value
whether to re-answer
whether to update current context
whether to preserve previous artifact
whether to ask one clarification

4.11 Topic Switch Detector

PH1.X must know when not to continue.

Example:

User: What time is it in New York?
User: What is your name?

This is not a time follow-up.

PH1.X should not let time/weather context steal normal questions.

This is exactly the bug you saw before.

4.12 Ambiguity / Clarification Engine

If the user says something too vague:

Sydney.
That one.
Do it.
Close it.

PH1.X should decide:

high confidence → continue
medium confidence → ask short clarification
low confidence → normal answer or no-match

Selene should ask the smallest useful clarification in normal language.

Example:

Do you mean the time question, or something else about Sydney?

This engine must avoid robotic clarification language.

4.13 Social Response Control

PH1.X must decide the shape of Selene’s next response before PH1.WRITE writes it.

Possible response shapes:

tiny acknowledgement
direct answer
one-question clarification
structured answer
memory recall
rewrite / modification
action confirmation
safe refusal
protected fail-closed
warning
wait / no response

Human conversation does not always need a full paragraph.

PH1.X should decide the appropriate response posture, and PH1.WRITE should turn it into beautiful language.

4.14 Risk + Protected Boundary Gate

If the follow-up might trigger protected execution, PH1.X must not guess.

Example:

User: Increase Tim’s salary.
User: Yes, do it.

PH1.X can understand the context, but protected execution still requires simulation + authority.

So PH1.X can identify intent, but cannot execute protected work.

4.15 PH1.M Handoff Port

PH1.X asks PH1.M only when live context is not enough.

Example:

30 seconds pass
Selene sleeps
User wakes
User: What about Sydney?

PH1.X asks PH1.M:

Is this continuing a fresh remembered topic?

PH1.M returns:

Likely continuing New York time question.

Then PH1.X resumes naturally.

PH1.X must not overuse PH1.M.

Live context first.Fresh memory second.Deep memory only when needed.

5. PH1.X Context Packet and Human Conversation Directive

PH1.X should produce one standard packet used by other engines.

Example:

ActiveContextPacket:
- active_topic
- active_intent
- interaction_posture
- conversation_rhythm
- continuation_type
- reference_target
- entity_focus
- tool_family
- writing_artifact
- pending_slots
- correction_target
- topic_stack
- response_shape
- confidence
- ambiguity_level
- protected_risk
- memory_handoff_needed
- suggested_next_engine

PH1.X should also output a single next-step directive.

HumanConversationDirective:
- continue current topic
- modify previous output
- correct previous output
- answer new question
- ask clarification
- hand off to PH1.M
- route to PH1.E
- route to PH1.WRITE
- fail closed for protected action
- wait / no action

This prevents every engine from inventing its own context logic and gives Selene one coherent live-conversation direction.

6. How PH1.X works with other engines

PH1.C

PH1.C decides if the transcript is valid user speech.

PH1.X only receives clean user input.

PH1.X

PH1.X decides live context.

It answers:

What does this mean in the current conversation?

PH1.M

PH1.M supplies remembered context when the active moment has passed.

PH1.X should not become memory.

PH1.E

PH1.E performs tools.

PH1.X tells PH1.E what the live context means.

PH1.WRITE

PH1.WRITE turns PH1.X context and runtime result into beautiful human language.

Desktop

Desktop only renders.

Desktop must never resolve “it,” “that,” “same,” or “continue.”

7. Human-like behavior rules

Rule 1 — Continue when obvious

What time is it in New York?
What about Sydney?

Continue.

Rule 2 — Modify when obvious

Write a message.
Make it warmer.

Modify previous output.

Rule 3 — Ask when ambiguous

Sydney.

Ask if context is unclear.

Rule 4 — Switch when clearly new

What time is it in New York?
What is your name?

Switch topic.

Rule 5 — Never let stale context steal normal questions

Old time/weather/place context must not hijack:

what is your name
write me a story
tell me a joke
explain Selene

Rule 6 — Do not over-memory

PH1.X should not pull old memory unless needed.

Live context first.Fresh memory second.Deep memory only when the user asks or the topic clearly requires it.

Rule 7 — Match the user’s interaction posture

Selene should react differently when the user is asking, correcting, testing, thinking out loud, frustrated, joking, or continuing.

PH1.X must decide the posture before PH1.WRITE writes the response.

Rule 8 — Keep human rhythm

Selene should not over-answer every utterance.

Sometimes the correct response is a direct answer.Sometimes it is one short clarification.Sometimes it is a tiny acknowledgement.Sometimes it is to wait.

Rule 9 — Ask the smallest useful clarification

Clarifications should be short, natural, and specific.

Selene should not expose mechanical ambiguity labels or routing language.

Rule 10 — Preserve returnable topics

If the user temporarily changes topic and then says “back to that” or “back to Japan,” PH1.X should restore the returnable topic when confidence is high.

8. PH1.X build plan

Build 0 — Repo Truth Inventory

Codex must inspect current PH1.X before coding.

Find:

current active context logic
time/weather follow-up patches
adapter context gates
PH1.E tool follow-up logic
PH1.M interactions
tests for topic switch / no-leak
duplicate context paths

No coding first.

Build 1 — Active Context Packet

Create or repair one central ActiveContextPacket.

All live context flows through it.

Build 2 — Universal Continuation Gate + Interaction Posture

Build one continuation decision engine and one interaction posture classifier.

Outputs:

continue
modify
correct
switch
clarify
memory_handoff
wait / no action

Postures:

question
instruction
correction
thinking_out_loud
frustration
testing
joking
continuation
topic_switch
memory_request
action_request

Build 3 — Reference Resolver + Conversation Rhythm

Resolve:

it
that
same
again
continue
make it shorter
do the same for Mark
what about Sydney
back to that
the first one
the other one

Also decide rhythm:

direct answer
short acknowledgement
one-question clarification
structured answer
wait / no action

Build 4 — Tool + Writing Continuity

Prove PH1.X works across:

time
weather
writing
email
story
planning
search
review
Codex instruction

Build 5 — Correction + Topic Switch + Social Response Control

Prove:

No, I meant...
Actually...
Not weather, time.
What is your name?
Tell me a joke.
That’s not what I meant.
Back to Japan.

Do not allow old context to steal new topics.

Also prove response shape selection:

tiny acknowledgement
direct answer
clarification
structured answer
memory handoff
protected fail-closed

Build 6 — PH1.M Fresh Memory Bridge

Prove:

New York → sleep → wake → what about Sydney

PH1.M provides memory evidence.PH1.X continues naturally.

Build 7 — Human Conversation Eval Matrix

Build a large test set:

short follow-ups
pronouns
corrections
topic switches
tool follow-ups
writing modifications
ambiguous fragments
protected-risk commands
memory handoffs
interaction posture shifts
conversation rhythm decisions
return-to-topic flows
normal questions after tool context
broken English follow-ups
frustrated user turns
thinking-out-loud turns

9. What Codex must not do

Codex must not:

add another time/weather patch
put context logic in Desktop
make PH1.E carry stale context
make PH1.M own active context
create adapter shortcut meaning
hardcode only a few phrases
weaken protected execution
let old context hijack normal chat

10. Final PH1.X rule

PH1.X is Selene’s live attention system.

It understands what the user means right now.

It handles continuation, reference, correction, modification, topic switch, ambiguity, interaction posture, conversation rhythm, response shape, and active task state.

PH1.M remembers.
PH1.X attends.
PH1.E acts.
PH1.WRITE explains.
Desktop displays.

That is the architecture that makes Selene feel human in live conversation.
