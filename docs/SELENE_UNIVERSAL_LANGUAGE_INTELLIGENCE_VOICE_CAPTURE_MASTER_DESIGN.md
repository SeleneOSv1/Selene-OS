Selene Universal Language Intelligence + Voice Capture Master Design

DOCUMENT TYPE:
DEDICATED MASTER DESIGN / LANGUAGE + VOICE + STT + TTS + TRANSCRIPT UNDERSTANDING ARCHITECTURE

TASK:
SELENE_UNIVERSAL_LANGUAGE_INTELLIGENCE_VOICE_CAPTURE_MASTER_DESIGN

BUILD CLASS:
ARCHITECTURE / LANGUAGE INTELLIGENCE / VOICE CAPTURE / OPENAI-ASSISTED HUMAN UNDERSTANDING STACK

CONTROLLING DOCUMENTS:
1. AGENTS.md
2. Selene Master Architecture Build Set
3. Selene Final Overall Architecture Build Plan
4. Selene Overall Repo-Truth Activation Pack
5. Selene Global Human Conversation Spine Master Architecture
6. Selene Global Request Decision Lattice + 5-Lane Business Risk View Master Design
7. Selene Identity + Access + Authority Spine Master Architecture
8. Selene PH1.WRITE — Structured Writing + Human Presentation Master Design
9. Selene PH1.M Human Memory Core Master Design
10. Selene Search Intelligence Lane — Revised Enterprise Websearch Master Design
11. Selene Emotional Intelligence + Relationship Presence Stack
12. Conversational Experience + Quick Assist Stack

PURPOSE:
Define Selene’s universal language intelligence and voice capture stack so Selene can understand multilingual, accented, broken, messy, misspelled, phonetic, slang-heavy, rambling, mixed-language, multi-speaker, and badly captured human input while preserving Selene-owned validation, protected execution law, identity/access scope, PH1.X routing, PH1.WRITE final presentation, and audit.

0. Master Standard

Selene must become globally capable at understanding human input.

Humans do not speak like clean API requests.

They ramble.

They mix languages.

They misspell names.

They speak with accents.

They use slang.

They trail off.

They restart sentences.

They confuse names.

They ask business questions casually.

They say protected things vaguely.

Selene must still understand them as well as possible.

The target is:

OpenAI STT / realtime / language reasoning
+ Selene transcript quality gates
+ source-language truth
+ broken-language normalization
+ spelling / phonetic / entity repair
+ speaker-language binding
+ turn-boundary protection
+ PH1.X request decision lattice
+ PH1.WRITE final human response
+ PH1.TTS exact approved speech
+ protected execution fail-closed law

Selene should feel like she can understand almost any human mess thrown at her.

But she must not guess dangerously.

The global law is:

Understand generously.
Clarify when needed.
Fail closed for protected risk.
Never let language repair become authority.

1. Core Rule

Language understanding must be fixed at the source, not patched at the final formatter.

Correct flow:

voice/audio or typed input
→ capture / transcript admission
→ transcript confidence and mismatch gate
→ source-language detection
→ spelling / phonetic / entity repair candidates
→ broken-language and intent normalization
→ SemanticInterpreterProvider / GPT-5.5 meaning proposal where applicable
→ PH1.X Request Decision Lattice validation
→ answer-language decision
→ stack routing
→ PH1.WRITE final answer
→ PH1.TTS exact approved speech
→ Desktop/iPhone render or play only
→ audit evidence

PH1.WRITE can make the final answer elegant.

But PH1.WRITE must not be the only reason the answer language, entity, or intent is correct.

2. OpenAI Role vs Selene Role

OpenAI provides powerful capability surfaces.

OpenAI may provide:

STT / transcription
realtime voice transcription
TTS
speech-to-speech where approved
speech translation where approved
language reasoning
semantic interpretation
phonetic and spelling candidate suggestions
entity disambiguation assistance
broken-language interpretation
multilingual answer drafting
TTS-safe wording

OpenAI must not own:

final transcript truth
answer-language policy
identity
access
authority
protected execution
memory permission
entity truth
business routing
simulation
final PH1.WRITE output
Desktop behavior
Adapter behavior
audit truth

The rule:

OpenAI helps Selene hear, understand, reason, and speak.
Selene owns transcript gates, language truth, routing, safety, permission, final output, and proof.

3. Target Product Standard

Selene must eventually handle:

English
Chinese
mixed English/Chinese
future supported languages
accented English
Chinese-accented English
broken English
bad grammar
slang
misspellings
phonetic mistakes
badly captured names
company/person/place/product entity confusion
multi-speaker conversations
meeting recordings
live assistant voice turns
half-captured turns
interrupted turns
noisy speech
user self-corrections
rambling input

Selene must not simply “try her best” in unsafe ways.

If transcript, language, speaker, entity, or intent is uncertain, Selene must:

answer with safe best effort for harmless public cases
ask short clarification when needed
retry capture when transcript is incomplete
safe-degrade when provider/capture is unavailable
fail closed for protected execution risk

4. Engine Ownership Map

4.1 PH1.C — Capture / Transcript Quality / STT Gate

PH1.C owns:

audio capture admission
transcript quality
partial vs final transcript safety
no-speech detection
low-confidence transcript handling
STT mismatch handling
wrong-language transcript gating
transcript confidence/status
provider-neutral transcript metadata
voice-originated turn evidence

PH1.C does not own:

final answer language
business execution
speaker identity authority
semantic routing
memory recall
protected action permission

4.2 PH1.LANG — Language Truth + Answer-Language Decision

PH1.LANG owns:

input_language
normalized_language
output_language
dominant_language
requested_language
topic_language / target_language
mixed_language flag
explicit language request
answer-language reason
per-turn language packet

PH1.LANG must ensure:

captured Chinese text → Chinese source language → Chinese answer unless user asks otherwise
captured English text → English source language → English answer unless user asks otherwise
mixed text → dominant/requested language answer
wrong-language transcript → STT mismatch or clarification, not fake understanding

PH1.LANG must reset per committed turn unless an explicit approved language directive or session rule applies.

4.3 PH1.N — Broken-Language / NLP / Intent Normalization

PH1.N owns:

broken sentence cleanup
bad grammar understanding
slang normalization
rambling input compression
intent normalization
preserving captured text separately from normalized intent
distinguishing topic language from answer language

Example:

User: I need you to teach me how to speak Chinese.
input_language = English
topic_language = Chinese
output_language = English

Selene must answer in English unless the user explicitly asks for Chinese output.

4.4 Spelling / Phonetic / Accent Repair Layer

This layer is required for world-class language unravelling.

It owns:

misspelled words
misheard words
similar-sounding terms
accent-driven mistakes
phonetic repair
company names
people names
places
product names
technical terms
business vocabulary
Selene project terms

This must not be a phrase-patch system.

It must use:

candidate generation
context scoring
entity evidence
phonetic similarity
language model assistance
rejection ledger
ambiguity ledger
clarification when confidence is low

Example:

STT hears: Tumbling organic wine
Likely candidate: Tamburlaine Organic Wines

Selene should select the candidate only when context/evidence supports it.

4.5 Entity Resolver

Entity Resolver owns:

company names
people names
places
products
business terms
known user/project entities
public entity disambiguation
private scoped entities where access allows

It must distinguish:

Selene = possible person/entity or Selene emotional presentation context
Selene = assistant identity
Tamburlaine = company/entity
Tumbling = ordinary word or bad transcript candidate
PH1.X = architecture engine, not random speech noise

4.6 PH1.X — Orchestration / Routing / Safety Boundary

PH1.X owns:

current-turn routing
Request Decision Lattice validation
public vs private vs protected lane separation
clarification decisions
safe handoff between PH1.C, PH1.LANG, PH1.N, PH1.E, PH1.M, PH1.WRITE, and protected gates
preventing stale session language from poisoning current turn

PH1.X consumes language/session packets.

PH1.X must not invent hidden language state.

4.7 PH1.WRITE — Final Answer Shaping

PH1.WRITE owns:

final answer shape
headers / paragraphs / bullets / tables
language-consistent presentation
Selene tone
Quick Assist wording
human clarity
TTS-safe final text

PH1.WRITE must not be the source of language truth.

PH1.WRITE presents validated truth.

4.8 PH1.TTS — Spoken Output

PH1.TTS owns:

speaking exact final answer text
OpenAI TTS provider lane usage
TTS input hash / display text hash where supported
no translation during TTS
no rewrite during TTS
self-echo gating coordination

PH1.TTS must not choose answer language independently.

4.9 PH1.VOICE.ID — Speaker Evidence, Not Authority

PH1.VOICE.ID may help with:

speaker evidence
speaker turn continuity
speaker-language history where allowed
liveness / anti-replay signals
speaker-specific adaptation where approved

PH1.VOICE.ID must not:

grant authority
execute protected actions
override current-turn language
infer protected attributes
turn one speaker’s language preference into another speaker’s output

4.10 Desktop / iPhone

Clients own:

microphone capture
UI state
recording controls
live assistant controls
playback
rendering approved output

Clients must not own:

semantic understanding
language policy
speaker-language preference
protected execution
business routing
memory relevance
provider calls

4.11 Adapter

Adapter owns:

provider wiring
runtime bridge
packet forwarding
trace/proof metadata transport

Adapter must not become the brain.

5. OpenAI STT / TTS / Realtime Provider Direction

Selene’s strategic voice path is OpenAI-based and provider-interface based.

Rules:

OpenAI STT / Realtime is the target speech-to-text direction.
OpenAI TTS is the target spoken-output direction.
Apple/platform-native STT/TTS must not be the target architecture.
Do not build new Apple STT/TTS capability.
Do not rely on Apple STT/TTS for multilingual proof.
Cross-platform Selene voice must not depend on Apple-only STT/TTS APIs.

Desktop boundary:

Do not delete the macOS Desktop app.
Do not remove Swift UI, Desktop shell, Desktop runtime bridge, macOS build files, or Apple Desktop platform support unless JD separately authorizes it.
The removal/deprecation scope is Apple STT/TTS voice-provider dependency only.

Provider architecture rule:

OpenAI is the first active provider lane.
OpenAI is not a hardcoded permanent dependency.
STT/TTS provider selection must sit behind provider lanes/interfaces.
Future approved providers must be addable without rewriting PH1.C, PH1.LANG, PH1.X, protected execution, or Desktop UI logic.

6. Live Assistant Mode vs Recording / Meeting Capture Mode

Selene has two different voice modes.

They must not be mixed.

6.1 Live Assistant Mode

Live Assistant Mode is “talk to Selene now.”

Behavior:

user speaks a prompt
Selene commits one final transcript
runtime processes the prompt immediately
Selene displays one assistant answer
OpenAI TTS autoplays the exact final answer
self-echo gate prevents Selene hearing herself

Used for:

questions
commands
clarifications
short live conversation
Quick Assist
wake/session interaction

6.2 Recording / Meeting Capture Mode

Recording mode is not live assistant mode.

Behavior:

user starts recording
recording continues over time
user stops recording manually
Selene keeps the full recording/transcript
Selene does not answer every sentence live
the completed transcript becomes an artifact/context item
user can later ask for extraction, summary, to-dos, decisions, documents

Examples after a meeting:

Extract all to-dos from this meeting.
Write a meeting summary.
List the decisions made.
Create follow-up emails.
Turn this into a document.

Protected execution rule:

Meeting transcript says: pay Tom tomorrow
→ to-do candidate only
→ no payment execution
→ later protected action request still requires authority + simulation

7. TTS Autoplay + Self-Echo Gate

Voice-originated turns should speak automatically where voice mode allows.

Required behavior:

assistant final answer text displayed once
OpenAI TTS automatically plays exact final assistant answer text
no duplicate assistant bubbles from TTS
no visible replay button required for normal operation
manual replay remains optional secondary function

Self-echo rule:

While Selene/OpenAI TTS is playing, microphone/STT capture must be paused, gated, or ignored.
Selene must not transcribe her own reply.
Selene must not create a new user turn from her own TTS audio.
Voice capture resumes after playback completes.

This is separate from full barge-in/interrupt handling.

Barge-in should be built later after turn ownership is stable.

8. Selene Name Canonicalization

Selene’s assistant identity must be canonical.

Required behavior:

If user addresses the assistant and STT captures Seline, Selina, Celene, or similar assistant-name homophones, normalize it to Selene in assistant-address contexts.

Do not globally rewrite unrelated real people with similar names.

Examples:

Are you working yet, Selene?
→ assistant-addressed Selene intent

What is your name?
→ I’m Selene.

Marina is my colleague.
→ keep Marina

This is narrow assistant identity canonicalization.

It is not the full spelling/phonetic engine.

9. Topic Language vs Output Language

Selene must distinguish:

language the user used
language being discussed
language requested for answer
language Selene should output

Required fields:

input_language
output_language
topic_language / target_language
explicit_language_request
language_reason

Examples:

I need you to teach me how to speak Chinese.
input_language = English
topic_language = Chinese
output_language = English

Correct:

answer in English
include Chinese examples if useful

Another example:

请教我中文。
input_language = Chinese
topic_language = Chinese
output_language = Chinese

Another example:

Teach me Chinese, answer in Chinese.
input_language = English
topic_language = Chinese
output_language = Chinese

Another example:

Can you explain 这个功能 in simple English?
input_language = mixed
output_language = English
preserve Chinese phrase

10. Per-Turn Language Reset

Every committed turn must receive a fresh language decision.

Rules:

previous turn output_language must not automatically carry into next turn
English after Chinese must answer English
Chinese after English must answer Chinese
session language history may exist but must not override current turn unless explicit approved rule applies
language continuity may apply only to true ambiguous follow-ups

Example bug to prevent:

Turn 1: 现在东京几点？
Selene output_language = zh

Turn 2: What time is it in Sydney?
Correct output_language = en
Bug output_language = zh

11. Turn Boundary / Half-Capture Safety

Selene must not silently answer half-captured voice turns as if complete.

Problem example:

User says:
现在东京几点？ Also what about Sydney?

Committed transcript only contains:
现在东京几点？

Selene cannot answer only Tokyo as if the whole prompt was captured.

Required behavior:

partial transcripts remain preview-only
final committed transcript is the only runtime entry
if partial text indicates more content than final text, clarify or retry
if final transcript ends with continuation cues, clarify rather than answer half-turn
VAD/silence tuning must be config/policy driven, not hardcoded patches

Continuation cue examples:

and
also
what about
then
plus
another thing

These examples belong in evals and policy, not hardcoded phrase patches as primary architecture.

12. Broken English / Slang / Bad Grammar Handling

Selene must understand messy human language.

Examples:

me need pay thing Tom tomorrow
can you make boss report from meeting
what time Tokyo now please
I no understand this thing make simple
hook me up with a summary
shoot him the email
what’s the damage on payroll
sort Tom out

Required behavior:

preserve original captured text
normalize intent into candidates
infer only when confidence is sufficient
ask clarification when required slots are unclear
never fill protected action slots from fuzzy language
fail closed or clarify for protected fuzzy commands

Harmless public clear intent:

answer normally

Unclear intent:

ask a short clarification

Protected fuzzy command:

fail closed or ask for protected-action clarification
no execution

13. Spelling / Phonetic / Accent Repair

Selene must include a dedicated spelling/phonetic/entity repair layer.

This layer handles:

bad spelling
bad STT spelling
accent-driven mistakes
similar-sounding words
company names
people names
place names
product names
technical terms
business domain terms
Selene architecture vocabulary

Required approach:

candidate generation
phonetic matching
context scoring
entity evidence
memory/entity graph where allowed
source evidence where needed
rejection ledger
ambiguity ledger
clarification when confidence is low

Examples:

Tamburlaine → not Tumbling
Seline → Selene when assistant-addressed
PH1.X → not “p h one ex”
NetSuite → not “net sweet”

Do not implement as isolated phrase patches.

14. Accent Handling

Accent handling spans multiple layers.

Owners:

OpenAI STT provider lane → raw transcription quality
PH1.C → confidence/mismatch gate
PH1.LANG → language detection
Spelling/Phonetic layer → misheard word correction
PH1.N → broken grammar/intent cleanup
PH1.VOICE.ID → speaker evidence and speaker-language binding where allowed

Accent handling must not infer:

nationality
ethnicity
race
protected attributes

Accent handling improves understanding.

It does not create identity or authority.

15. Multi-Speaker / Speaker-Language Binding

Speaker-language binding must be part of the current language/voice build plan.

This does not mean Voice ID grants authority.

It means Selene can hold multilingual conversations with multiple speakers.

Target behavior

Speaker A speaks English.
Selene answers Speaker A in English.

Speaker B speaks Chinese.
Selene answers Speaker B in Chinese.

Speaker A speaks again later.
Selene preserves Speaker A’s turn context and language.

Speaker B speaks again later.
Selene preserves Speaker B’s turn context and language.

Safe rule before persistent speaker preferences

Until persistent speaker-language preferences are formally approved:

current turn language wins

Examples:

English current turn → English answer
Chinese current turn → Chinese answer
mixed current turn → dominant/requested language
unclear speaker identity → current-turn language wins or clarify

Required speaker-language packet

speaker_id or temporary_speaker_label
turn_id
session_id
captured_language
output_language
language_reason
speaker_language_preference if available
current_turn_language
explicit_user_language_request
confidence
source_modality
voice_id_confidence if available
speaker_known / speaker_unknown / speaker_uncertain

Voice ID boundary

PH1.VOICE.ID may identify or suggest who is speaking.

It must not:

override current-turn language accidentally
execute protected actions
imply authority
infer protected attributes
copy one speaker’s language preference to another speaker

Required two-speaker proof

Speaker A: What time is it in Tokyo?
Selene: answers English.

Speaker B: 现在东京几点？
Selene: answers Chinese.

Speaker A: What about Sydney?
Selene: answers English, resolving follow-up to Speaker A’s context.

Speaker B: 悉尼呢？
Selene: answers Chinese, resolving follow-up to Speaker B’s context.

If speaker identity is uncertain, Selene must not guess.

Use current-turn language or ask short clarification.

16. Protected Execution Safety

Language unravelling must never weaken protected execution.

Rules:

public language repair requires no simulation
protected actions require simulation + authority + confirmation + audit
fuzzy transcript cannot fill protected slots
broken English cannot trigger execution
meeting transcript to-dos are not execution
business/public question classification is separate from language repair

Examples:

What is the average salary at SpaceX?
→ public information question

Increase Tim’s salary.
→ protected action; simulation lookup required

Meeting transcript: Pay Tom tomorrow.
→ to-do candidate only, no execution

17. Required Packets / Evidence

Codex must reuse repo packet names where they exist and map these architecture names to equivalents.

Required logical packets:

TranscriptQualityPacket
FinalTranscriptPacket
PartialTranscriptPacket
LanguageDecisionPacket
SpeakerLanguagePacket
BrokenLanguageNormalizationPacket
PhoneticCandidatePacket
EntityCorrectionCandidatePacket
EntityResolutionPacket
TurnBoundaryPacket
HalfCaptureSafetyPacket
TtsPlaybackPacket
SelfEchoGatePacket
LiveAssistantModePacket
MeetingCaptureArtifactPacket
LanguageUnravellingEvidencePacket

Each language/voice decision must be traceable.

18. Required Proof Categories

Codex must prove the language stack through typed, voice-like, and live voice tests.

Required proof areas:

Chinese typed input
Chinese voice input
English typed input
English voice input
mixed English/Chinese
English after Chinese
Chinese after English
explicit answer-language directive
topic language vs output language
STT mismatch
wrong-language transcript gating
partial vs final transcript safety
TTS exact text
self-echo gate
Selene name canonicalization
broken English clear intent
broken English unclear intent
slang normalization
phonetic/entity candidate repair
accented English proof
multi-speaker speaker-language binding
meeting recording artifact path
protected fail-closed preservation

Do not claim spoken-language capability without live proof.

19. World-Class Language Evaluation Corpus

Selene needs a repeatable evaluation corpus for:

human English voice
human Chinese voice
mixed English/Chinese
Chinese-accented English
fast speech
slow speech
noisy speech
broken English
slang
bad grammar
badly spelled typed input
misheard company/person/place names
multi-speaker multilingual conversations
meeting recordings
half-captured turns
self-correction turns
protected fuzzy commands

No capability should be claimed without passing real evals.

20. Domain Lexicon + Entity Memory

Selene needs governed lexicons for:

Selene project terms
company names
customer names
product names
technical terms
wine/business terms
user-approved private entities
public entities
protected business vocabulary

This must be governed and scoped.

It must not become uncontrolled memory or privacy leakage.

Potential owners:

Entity Resolver
PH1.M for governed memory-backed entities
PH1.E for public entity evidence
Access/Governance for private company entities

21. Build Order

Do not build everything in one pass.

Build 0 — Language + Voice Repo-Truth Activation Pack

Before coding, Codex must inspect:

PH1.C
PH1.LANG
PH1.N
PH1.X
PH1.WRITE
PH1.TTS
PH1.VOICE.ID
Desktop voice capture
Adapter STT/TTS routes
OpenAI provider governance
Apple STT/TTS remnants
meeting/recording surfaces
language tests
echo/interrupt/barge-in surfaces
entity/spelling/correction paths

No implementation.

Build 1 — OpenAI STT/TTS Provider Lane Baseline

Purpose:

OpenAI STT/TTS through provider interfaces
Apple STT/TTS deprecation map
provider-off/fake-provider proof
no startup probes
TTS exact text proof

Build 2 — Transcript Quality + Turn Boundary Gate

Purpose:

partial vs final transcript safety
half-capture detection
low-confidence handling
wrong-language transcript mismatch gate

Build 3 — Per-Turn Language Decision + Topic/Output Language Split

Purpose:

input_language
output_language
topic_language
explicit language request
per-turn reset
English after Chinese
Chinese after English

Build 4 — Selene Name Canonicalization

Purpose:

assistant-addressed Selene homophones normalize to Selene
unrelated people/entities with similar names remain untouched

Build 5 — Broken English / Slang / Bad Grammar Normalization

Purpose:

normalize messy input into intent candidates
preserve original captured text
clarify protected ambiguity

Build 6 — Spelling / Phonetic / Entity Repair Design

Purpose:

candidate generation
context scoring
entity evidence
rejection ledger
ambiguity ledger
clarification rules

Build 7 — Spelling / Phonetic / Entity Repair Implementation

Purpose:

general repair engine
no phrase patches
domain lexicon integration
entity resolver integration

Build 8 — Speaker-Language Binding

Purpose:

per-speaker turn ownership
speaker language history
current-turn language wins
multi-speaker language proof
Voice ID evidence-only boundary

Build 9 — TTS Autoplay + Self-Echo Gate

Purpose:

autoplay exact final answer
pause/gate mic during TTS
no self-transcription
resume capture after playback

Build 10 — Meeting Recording Artifact Pipeline

Purpose:

recording mode separate from live assistant mode
long transcript capture
meeting artifact
summary/to-do/decision extraction
protected to-do not execution

Build 11 — Language Evaluation + JD Live Certification

Purpose:

full eval corpus
voice and typed tests
multi-language/multi-speaker proof
protected fail-closed preservation

22. Immediate Repair Priority

The immediate repair path should prioritize:

turn boundary safety
response language reset
topic_language vs output_language
explicit language directives
Selene name canonicalization
TTS exact text and self-echo preservation

These should be built before broad spelling/phonetic implementation.

23. What Codex Must Not Do

Codex must not:

hardcode isolated phrases as production intelligence
patch language only at formatting
let Desktop or Adapter become the language brain
weaken protected execution
use Apple STT/TTS as target architecture
claim spoken-language capability without live proof
claim spelling/phonetic capability before dedicated layer is designed and tested
infer protected attributes from accent or language
let Voice ID grant authority
let meeting transcript tasks execute protected actions
create a duplicate language/router system

24. Success Standard

Selene becomes world-class at language unravelling when:

OpenAI STT captures speech accurately through governed provider lanes
PH1.C detects transcript quality and mismatch
PH1.LANG correctly separates input/output/topic language
PH1.N normalizes broken language and slang
spelling/phonetic layer repairs likely misheard entities
entity resolver disambiguates names and terms
PH1.X validates safe routing through the Request Decision Lattice
PH1.WRITE answers naturally in the right language
PH1.TTS speaks exact approved text
PH1.VOICE.ID supports speaker-language binding without authority drift
Desktop/iPhone render/play only
Adapter transports only
protected execution remains fail-closed

25. Final Law

Selene should understand humans generously.

Selene should answer naturally.

Selene should clarify briefly when needed.

Selene should fail closed when protected risk exists.

Selene should never build language intelligence from phrase patches.

The final architecture is:

OpenAI hears and reasons.
Selene validates transcript, language, entity, intent, risk, owner, and presentation.
PH1.WRITE speaks like a human.
PH1.TTS says exactly what was approved.
Protected execution remains deterministic.

That is how Selene becomes the understanding queen across languages, accents, messy speech, spelling mistakes, slang, and human chaos without becoming unsafe or robotic.
