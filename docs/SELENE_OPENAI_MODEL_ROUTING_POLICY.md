# Selene OpenAI Model Routing Policy

```text
DOCUMENT TYPE:
JD-CONTROLLED MODEL ROUTING POLICY

CONTROLLING PURPOSE:
Define which OpenAI models Selene should use for each provider-first capability until JD explicitly changes the policy.

STATUS:
AUTHORITATIVE MODEL POLICY FOR OPENAI ROUTING

IMPORTANT:
Codex must not choose, change, upgrade, downgrade, replace, fallback, or cost-optimize OpenAI models without explicit JD approval.
```

---

# 0. Controlling Rule

JD controls model selection.

Codex does not control model selection.

Codex may wire the approved policy, test the approved policy, and report whether a model route works.

Codex must not silently change the policy.

```text
JD chooses the model policy.
Codex implements the model policy.
Codex does not choose alternative models.
Codex does not swap models for cost.
Codex does not upgrade models because newer appears available.
Codex does not downgrade models because cheaper appears available.
Codex does not fallback silently.
```

Any OpenAI model change is a behavior change and requires explicit JD approval.

---

# 1. Starting Strategy

JD’s starting strategy is quality-first:

```text
Use the best/latest/highest-quality capable OpenAI model first.
Prioritize reliability, quality, accuracy, and speed.
Do not optimize for cost at the beginning.
Cost reduction comes later only after Selene behavior is live-proven.
```

The first goal is to make Selene outstanding.

The second goal, later, is to reduce cost safely after the architecture is working and live-proven.

---

# 2. Model Governance Rule

Every model route must declare:

```text
Selene function
canonical owner
provider interface
model ID
reason for model choice
fallback model
provider-off behavior
allowed lane
forbidden lane
JD approval required for changes
tests required after model change
backend evidence required
JD live proof required where user-visible
```

Codex must not add or edit provider model routing without this information.

If model routing is missing, Codex must stop with:

```text
JD_MODEL_ROUTING_POLICY_REQUIRED
```

If Codex wants to change a model, it must stop with:

```text
JD_MODEL_CHANGE_APPROVAL_REQUIRED
```

If an approved model is unavailable, deprecated, inaccessible, or fails provider validation, Codex must stop and report:

```text
APPROVED_MODEL_UNAVAILABLE_JD_DECISION_REQUIRED
```

Codex must not choose the replacement.

---

# 3. Initial OpenAI Model Routing Policy

This is the initial quality-first model route.

Cost optimization is deferred.

## 3.1 Default Core Intelligence

```text
Selene function:
general reasoning, difficult language understanding, complex planning, analysis, business-quality answers

Canonical owner:
Provider Governance / PH1.X / PH1.WRITE / PH1.E depending on route

Provider interface:
ModelReasoningProvider

Approved model:
gpt-5.5

Reason:
Use the flagship/highest-quality model first for complex reasoning, professional work, long context, structured interpretation, and high-quality output.

Fallback:
none without JD approval

Provider-off behavior:
safe degrade or ask/stop depending on lane

JD approval required for changes:
yes
```

## 3.2 PH1.X Semantic Interpretation

```text
Selene function:
current turn understanding, intent proposal, operation proposal, reference proposal, ambiguity hints, protected-risk proposal

Canonical owner:
PH1.X

Provider interface:
SemanticInterpreterProvider

Approved model:
gpt-5.5

Required output:
CurrentTurnInterpretationPacket or current repo equivalent

Reason:
PH1.X is high-risk because it affects routing, context, protected classification, and wrong-owner prevention. Start with best available quality.

Fallback:
none without JD approval

Provider-off behavior:
PH1.X deterministic fallback or clarification if available; protected uncertainty fails closed

JD approval required for changes:
yes
```

## 3.3 PH1.WRITE Writing / Presentation

```text
Selene function:
beautiful writing, rewriting, summarisation, translation, tone control, headers, bullets, reports, SOPs, emails, proposals, display_text, tts_text

Canonical owner:
PH1.WRITE

Provider interface:
WritingProvider

Approved model:
gpt-5.5

Required output:
WriteOutputPacket or current repo equivalent

Reason:
Writing quality is one of Selene’s product differentiators. Start with best available writing/reasoning quality.

Fallback:
none without JD approval

Provider-off behavior:
safe deterministic/plain fallback if available; otherwise degrade honestly

JD approval required for changes:
yes
```

## 3.4 PH1.E Search Synthesis / Web Search Answering

```text
Selene function:
public web search synthesis, source-backed answer generation, source comparison, citation-aware answer drafting

Canonical owner:
PH1.E + PH1.WRITE

Provider interface:
SearchProvider / ModelReasoningProvider / WritingProvider

Approved model:
gpt-5.5 with web search tool where permitted by provider governance

Required output:
SearchEvidencePacket + SourceAcceptancePacket + WriteOutputPacket or current repo equivalents

Reason:
Search answers must be accurate, source-backed, and clean. Start with best core model plus governed search tool.

Fallback:
none without JD approval

Provider-off behavior:
safe no-search degraded answer or cached/local result if allowed

JD approval required for changes:
yes
```

## 3.5 Deep Research

```text
Selene function:
long-form research, market research, legal/compliance research, supplier research, competitor analysis, board-level report drafts

Canonical owner:
PH1.E + PH1.WRITE + Provider Governance

Provider interface:
DeepResearchProvider

Approved primary model:
o3-deep-research if available and approved in current OpenAI account

Approved fallback model:
o4-mini-deep-research only if JD explicitly approves fallback for cost/latency

Reason:
Start with highest-quality deep research for important research jobs. Use cheaper deep research only later or by explicit JD approval.

Provider-off behavior:
do not run deep research; explain provider unavailable/degraded

JD approval required for changes:
yes

Important:
If OpenAI docs/API mark the selected deep research model unavailable, deprecated, or inaccessible, Codex must not choose another model. Codex must stop and request JD decision.
```

## 3.6 Speech-to-Text / Normal Transcription

```text
Selene function:
audio file transcription, bounded utterance transcription, non-realtime STT

Canonical owner:
PH1.C

Provider interface:
SpeechToTextProvider

Approved primary model:
gpt-4o-transcribe

Approved secondary model:
gpt-4o-transcribe-diarize when diarization/speaker labeling is specifically needed

Approved cheaper fallback:
gpt-4o-mini-transcribe only with JD approval

Required output:
TranscriptPacket or current repo equivalent

Reason:
Accuracy first. Diarization only when speaker separation is needed. Cheaper mini route is not default.

Provider-off behavior:
voice/STT unavailable or degraded state; no transcript committed unless lawful fallback exists

JD approval required for changes:
yes
```

## 3.7 Realtime Voice Agent

```text
Selene function:
low-latency spoken conversation, realtime voice assistant, voice-agent workflows, tool-proposal-capable realtime session

Canonical owner:
PH1.W / PH1.C / PH1.K / PH1.TTS / PH1.L

Provider interface:
RealtimeVoiceProvider / RealtimeTransportProvider

Approved primary model:
gpt-realtime-2

Reason:
Use the state-of-the-art realtime reasoning voice model for highest-quality realtime voice behavior.

Recommended initial reasoning effort:
low, unless JD approves higher for complex voice tasks

Fallback:
none without JD approval

Provider-off behavior:
voice realtime unavailable/degraded; do not silently switch to cheaper realtime model

JD approval required for changes:
yes
```

## 3.8 Realtime Transcription

```text
Selene function:
low-latency transcript deltas from live audio

Canonical owner:
PH1.C / PH1.L

Provider interface:
RealtimeTranscriptionProvider

Approved model:
gpt-realtime-whisper

Reason:
Use the dedicated streaming speech-to-text realtime transcription model for transcript deltas and low-latency voice UX.

Fallback:
none without JD approval

Provider-off behavior:
realtime transcription unavailable/degraded; no partial transcript becomes committed turn unless PH1.C admits it

JD approval required for changes:
yes
```

## 3.9 Realtime Live Translation

```text
Selene function:
speech-to-speech live translation when explicitly needed

Canonical owner:
PH1.C / PH1.LANG / PH1.WRITE / PH1.TTS

Provider interface:
LiveTranslationProvider

Approved model:
gpt-realtime-translate

Reason:
Dedicated realtime translation model for live multilingual audio experiences.

Implementation status:
deferred until JD explicitly approves live translation work

Fallback:
none without JD approval

Provider-off behavior:
translation unavailable/degraded

JD approval required for changes:
yes
```

## 3.10 Text-to-Speech

```text
Selene function:
spoken output from approved_tts_text

Canonical owner:
PH1.TTS

Provider interface:
TextToSpeechProvider

Approved model:
gpt-4o-mini-tts

Required input:
approved_tts_text only

Required output:
VoiceOutputPacket or current repo equivalent

Reason:
OpenAI’s current fast, reliable TTS model suitable for natural spoken output with style instructions.

Fallback:
none without JD approval

Provider-off behavior:
TTS unavailable/degraded; do not fake spoken output

JD approval required for changes:
yes
```

## 3.11 Vision / Image Understanding

```text
Selene function:
image understanding, OCR-style visual reasoning, screenshot/document visual interpretation, product/receipt/image summary

Canonical owner:
PH1.E / PH1.M / PH1.WRITE depending on use

Provider interface:
VisionProvider

Approved model:
gpt-5.5

Required output:
VisualEvidencePacket or current repo equivalent

Reason:
Use flagship multimodal intelligence first for highest quality image/file understanding.

Fallback:
none without JD approval

Provider-off behavior:
vision unavailable/degraded; do not invent visual evidence

JD approval required for changes:
yes
```

## 3.12 Embeddings / Memory Retrieval

```text
Selene function:
semantic retrieval, memory retrieval, topic matching, archive lookup, company knowledge retrieval

Canonical owner:
PH1.M / Storage

Provider interface:
EmbeddingProvider

Approved primary model:
text-embedding-3-large

Approved future cost route:
text-embedding-3-small only after JD approves cost optimization

Required output:
EmbeddingPacket or current repo equivalent

Reason:
Use most capable embedding model first for retrieval quality.

Fallback:
none without JD approval

Provider-off behavior:
embedding unavailable/degraded; local deterministic recall only if available

JD approval required for changes:
yes
```

## 3.13 Moderation / Safety Signal

```text
Selene function:
text/image moderation signal, public-facing safety, customer chat safety, unsafe request detection

Canonical owner:
Policy governance / Access-Gov / PH1.WRITE refusal wording

Provider interface:
ModerationProvider

Approved model:
omni-moderation-latest

Reason:
Use latest moderation model as advisory safety signal.

Fallback:
none without JD approval

Provider-off behavior:
moderation unavailable/degraded; Selene policy still applies

JD approval required for changes:
yes
```

## 3.14 Code Interpreter / Advisory Analysis

```text
Selene function:
spreadsheet analysis, chart generation, advisory P&L draft, file transformation, business report drafts

Canonical owner:
Advisory analysis lane / PH1.WRITE / Storage evidence

Provider interface:
CodeInterpreterProvider

Approved model route:
gpt-5.5 with code interpreter tool where explicitly approved

Implementation status:
BUILD_ONLY_AFTER_JD_APPROVAL

Reason:
Powerful advisory analysis capability, but not part of first implementation and never allowed for Codex repo work.

Forbidden:
repo editing by Codex
protected official accounting execution
posting invoices
payroll approval
database mutation
bank/payment operations

JD approval required for changes:
yes
```

## 3.15 Image Generation

```text
Selene function:
marketing assets, menus, posters, product visuals, business onboarding visuals, social media drafts

Canonical owner:
PH1.WRITE / media asset governance / Desktop renderer

Provider interface:
ImageGenerationProvider

Approved model:
current official OpenAI image generation model only after JD explicitly approves image generation work

Implementation status:
BUILD_ONLY_AFTER_JD_APPROVAL

Reason:
Useful later, not part of first provider-first implementation.

Provider-off behavior:
image generation unavailable/degraded

JD approval required for changes:
yes
```

## 3.16 Video Generation

```text
Selene function:
marketing videos, training videos, onboarding videos, social assets

Canonical owner:
future marketing/media module / PH1.WRITE / artifact governance

Provider interface:
VideoGenerationProvider

Approved model:
Sora / current official OpenAI video generation model only after JD explicitly approves video generation work

Implementation status:
BUILD_ONLY_AFTER_JD_APPROVAL

Reason:
Future optional media workflow, not part of initial provider-first pivot.

Provider-off behavior:
video generation unavailable/degraded

JD approval required for changes:
yes
```

---

# 4. Cost Optimization Rule

Cost optimization is explicitly deferred.

Codex must not choose cheaper models unless JD approves.

The initial policy is:

```text
quality first
accuracy first
speed second
cost later
```

Later, after Selene is live-proven, JD may approve a cost optimization pass.

Possible future cost routes include:

```text
gpt-5.4-mini for lower-cost general reasoning
gpt-5.4-nano for very low-cost simple classification
gpt-4o-mini-transcribe for cheaper transcription
text-embedding-3-small for cheaper retrieval
gpt-realtime-mini for cheaper realtime sessions
o4-mini-deep-research for cheaper deep research
```

These are not default routes.

They are future options only.

---

# 5. Model Change Approval Rule

Any model change requires explicit JD approval.

This includes:

```text
switching model IDs
changing fallback models
using mini/nano models for cost
using newer aliases
using older snapshots
changing reasoning effort defaults
changing realtime model
changing transcription model
changing TTS model
changing embedding model
changing moderation model
changing search/deep-research model
```

Codex must not infer approval.

If model change is needed, Codex must stop with:

```text
JD_MODEL_CHANGE_APPROVAL_REQUIRED
```

---

# 6. Implementation Rule

Codex may implement only the model policy that exists in this file.

Codex must not invent model routing from memory.

Codex must not copy model choices from examples, docs, comments, or tests unless this policy authorizes them.

Provider governance implementation must treat this file as the source of truth for OpenAI model routing.

Every implementation build that touches providers or model routing must report:

```text
model route used
model ID
canonical owner
provider interface
why that model is allowed
whether JD approval exists
provider-off behavior
fallback route
tests passed
backend evidence
```

---

# 7. Final Rule

```text
JD owns model choice.
Codex implements model choice.
Codex does not optimize for cost until JD approves.
Codex does not silently change models.
Selene uses best/latest/highest-quality OpenAI models first.
Provider governance enforces the model policy.
```
