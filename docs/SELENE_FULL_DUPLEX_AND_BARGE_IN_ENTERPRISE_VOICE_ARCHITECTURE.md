Selene Full Duplex and Barge-In Enterprise Voice Architecture

Document Type: Master Architecture + Codex Build FoundationArchitecture Name: SELENE_FULL_DUPLEX_AND_BARGE_IN_ENTERPRISE_VOICE_ARCHITECTURESystem: SeleneScope: Full-duplex voice, barge-in, interruption control, TTS cancellation, live conversation continuity, speaker-aware interruption, language-aware interruption, and protected-execution fail-closed governance.

0. Executive Summary

Selene must become a voice-first enterprise assistant that can be interrupted naturally like a human, while still executing protected work like a governed deterministic business system.

This architecture gives Selene a world-class full-duplex and barge-in foundation:

Selene can listen while speaking.

Selene can detect real interruption while rejecting echo, coughs, noise, and background speech.

Selene can stop, duck, pause, truncate, or resume speech correctly.

Selene can understand whether the interruption is a stop request, correction, clarification, continuation, topic switch, rejection, urgent override, or protected command.

Selene can preserve what the user actually heard and remove what was never spoken.

Selene can handle multiple speakers, language switching, broken speech, bad grammar, and low-confidence transcript fragments.

Selene can keep public conversation fluid while keeping protected execution deterministic, authority-gated, simulation-bound, confirmed, audited, and fail-closed.

The governing principle is simple:

Selene may converse in full duplex, but Selene must execute in deterministic single-owner mode.

Full-duplex conversation is allowed. Full-duplex protected execution is not.

1. Controlling Law

1.1 Duplex Conversation Law

Selene is allowed to keep the microphone open while she is speaking.

This allows natural human interaction:

interruption,

correction,

clarification,

topic switching,

cancellation,

quick follow-up,

real-time conversation rhythm.

However, microphone activity during Selene speech is only evidence until validated by the runtime.

Raw audio does not equal a turn.Partial transcript does not equal intent.Interruption does not equal authority.Speech during TTS does not equal permission to execute.

1.2 Protected Execution Law

Any action that changes business state, accesses sensitive data, performs official company work, mutates records, approves work, sends official communications, changes access, pays money, changes payroll, updates HR, updates inventory, updates POS, updates accounting, or performs any authority-gated business action must follow the protected execution lane.

Protected execution requires:

deterministic intent validation,

simulation match,

required slots,

authority validation,

confirmation where required,

policy checks,

execution through the correct canonical owner,

audit/proof logging,

user notification.

If any required protected step fails, Selene must fail closed.

1.3 Public Conversation Law

Normal conversation, public answers, public search, summaries, drafting, explanations, translations, public tool lookups, time, weather, and non-mutating advisory responses do not require simulation approval.

These belong to the probabilistic public-answer lane.

The system must never confuse this with protected business execution.

1.4 Final Boundary Rule

Selene can be interrupted like a human, but she can only execute like a governed enterprise system.

2. Core Design Principle

Selene must separate three things that most voice systems incorrectly merge:

Audio activity — someone or something made sound.

Conversational meaning — the sound became a validated human turn with intent.

Execution authority — the validated turn is allowed to cause action.

A world-class barge-in architecture must never skip from audio activity directly to execution.

The correct chain is:

Audio Evidence
→ Speech Candidate
→ Echo / Noise / Speaker Validation
→ Barge-In Candidate
→ Barge-In Decision
→ Committed Interruption Turn
→ PH1.X Meaning Validation
→ Public / Tool / Search / Memory / Protected Route
→ Protected Gate if Required
→ Final Response / Action
→ Proof / Audit

3. Architectural Separation

Selene’s duplex system must be divided into three independent runtime channels.

3.1 User Input Channel

The user input channel captures:

microphone frames,

local VAD evidence,

provider VAD evidence,

transcript deltas,

confidence,

language candidates,

speaker candidates,

noise score,

echo score,

device route,

session state,

interruption evidence.

This channel is evidence-producing only.

It does not decide final meaning.

3.2 Selene Output Channel

The Selene output channel controls:

TTS stream,

audio playback,

TTS item id,

response id,

playback started timestamp,

played audio position,

unplayed audio position,

cancellation,

ducking,

pause,

resume,

truncation,

spoken text hash,

displayed text hash.

This channel must maintain a precise ledger of what Selene actually said and what the user actually heard.

3.3 Authority / Action Channel

The authority/action channel controls:

protected simulation execution,

tools,

search,

file actions,

connector actions,

memory writes,

database mutations,

payroll,

HR,

accounting,

POS,

inventory,

access control,

audit.

This channel must remain serialized, deterministic, and owner-controlled.

The input and output channels may run concurrently.

The authority/action channel must not be concurrently mutated by uncontrolled voice input.

4. Canonical Engine Ownership

This architecture must be built inside Selene’s existing canonical owners.

It must not create a duplicate voice brain.

It must not move semantic control into Desktop.

It must not let the provider become authority.

4.1 PH1.W — Wake Authority

PH1.W owns activation only.

Responsibilities:

wake candidate evaluation,

accepted wake event,

rejected wake event,

wake confidence,

wake proof.

PH1.W must not:

answer questions,

route tools,

perform search,

execute business actions,

decide barge-in meaning,

mutate state.

Wake opens the door. It does not decide what happens inside the room.

4.2 PH1.L — Session Lifecycle

PH1.L owns:

session open,

session resume,

active session,

soft close,

full close,

listening window,

re-arm,

turn boundary,

session timing,

whether interruption belongs to current session or requires new activation.

PH1.L decides whether Selene is currently allowed to listen for conversational content.

4.3 PH1.C — Capture and STT Quality

PH1.C owns:

audio quality,

transcript deltas,

transcript confidence,

speech probability,

noise probability,

accent risk,

bad-pronunciation risk,

broken-language risk,

endpointing evidence,

partial transcript handling.

PH1.C must distinguish:

real user speech,

Selene echo,

background speech,

cough,

noise,

overlapping speech,

short fragment,

low-confidence text.

PH1.C does not own final user intent.

4.4 PH1.K — Interruption and Barge-In Controller

PH1.K is the canonical owner of barge-in detection and TTS control decisions.

PH1.K owns:

interruption detection,

speech-over-agent detection,

echo rejection,

noise rejection,

barge-in candidate creation,

barge-in decision creation,

TTS duck decision,

TTS cancel decision,

TTS pause decision,

TTS truncate decision,

played/unplayed boundary capture,

interruption proof.

PH1.K must not decide final business meaning.

PH1.K answers this question:

“Was this a valid interruption event, and what should happen to Selene’s current speech?”

PH1.K does not answer:

“What does the user mean, and may Selene execute it?”

That belongs to PH1.X and protected execution owners.

4.5 PH1.LANG — Language Continuity

PH1.LANG owns:

input language,

output language,

script detection,

mixed-language detection,

language switch detection,

speaker-language continuity inside the active session,

same-language response policy,

language confidence thresholds.

PH1.LANG must support:

English user interruption during English answer,

Chinese user interruption during English answer,

mixed English/Chinese interruption,

person A speaking English and person B speaking Chinese,

preserving technical/business terms,

short clarification when language confidence is low.

PH1.LANG must not infer nationality, ethnicity, race, citizenship, or protected personal identity from language.

4.6 PH1.VOICEID — Speaker Posture

PH1.VOICEID owns:

speaker candidate,

known/unknown posture,

speaker confidence,

speaker continuity,

speaker mismatch evidence,

voice identity assist.

Voice ID is posture evidence.

It is not final authority.

Protected actions still require the authority gate.

4.7 PH1.X — Human Conversation Core

PH1.X owns live conversational meaning.

PH1.X receives the interruption directive and decides whether the interruption means:

stop,

correction,

clarification,

continuation,

topic switch,

rejection,

urgent override,

answer narrowing,

cancellation,

protected command,

ambiguous reference,

low-confidence clarification,

public advisory response,

search/tool route,

memory route,

file route,

protected execution route.

PH1.X must maintain:

selected candidate ledger,

rejected candidate ledger,

hard disqualifier ledger,

ambiguity ledger,

protected-risk ledger,

owner routing trace.

PH1.X must not execute protected actions directly.

4.8 PH1.E — Search / Tool / File Need Classifier

PH1.E owns whether a validated turn needs:

public search,

deterministic tool,

file retrieval,

connector action,

evidence collection,

source verification,

public answer fallback.

PH1.E must not be bypassed by barge-in shortcuts.

4.9 Simulation and Authority Engines

Protected execution engines own official business action.

They own:

simulation match,

process selection,

required slots,

authority validation,

confirmation,

policy checks,

execution,

audit,

notification.

Examples:

Payroll Engine,

HR Engine,

Leave Engine,

Access Engine,

Inventory Engine,

POS Engine,

Accounting Engine,

Commission Engine,

CRM Engine.

No protected action can execute merely because the user interrupted Selene while she was speaking.

4.10 PH1.WRITE — Response and Presentation Layer

PH1.WRITE owns:

final response text,

TTS-safe wording,

interruption-aware acknowledgements,

clarification wording,

protected fail-closed explanation,

source chips,

display structure,

spoken/display separation,

language-consistent answer shaping.

PH1.WRITE must ensure TTS speaks the final runtime-approved answer text.

4.11 Adapter

Adapter transports runtime packets only.

Adapter may carry:

audio events,

provider events,

TTS playback ledgers,

barge-in packets,

response packets,

source chips,

image cards,

trace references.

Adapter must not:

decide interruption meaning,

route protected execution,

infer user intent independently,

become the conversation brain,

hide backend failure.

4.12 Desktop and iPhone

Desktop and iPhone are clients only.

They own:

microphone capture,

audio playback,

UI rendering,

visible state,

local mute,

local device evidence,

playback position reporting,

obeying runtime TTS control.

They must not own:

wake authority,

semantic routing,

memory,

search,

protected execution,

simulation approval,

final interruption meaning.

Desktop may stop audio fast when instructed, but it must not decide why the interruption matters.

5. Required Runtime Packets

5.1 DuplexAudioFrame

DuplexAudioFrame {
  frame_id
  session_id
  device_id
  captured_at_monotonic_ms
  audio_channel
  mic_level
  playback_level
  tts_playback_active
  tts_item_id
  tts_response_id
  tts_played_ms
  tts_unplayed_ms_estimate
  acoustic_echo_score
  transcript_echo_similarity_score
  noise_score
  speaker_overlap_score
  local_vad_score
  provider_vad_score
  device_route
  wake_state
  session_state
}

Purpose:

Give PH1.C and PH1.K enough evidence to determine whether incoming audio is real human interruption, Selene echo, background speech, cough, or noise.

5.2 TtsPlaybackLedger

TtsPlaybackLedger {
  response_id
  tts_item_id
  session_id
  turn_id
  display_text_sha256
  tts_input_text_sha256
  tts_provider
  audio_stream_started_at_ms
  first_audio_played_at_ms
  last_audio_played_at_ms
  played_ms
  unplayed_ms_estimate
  spoken_text_boundary_estimate
  playback_state:
    queued
    speaking
    ducked
    paused
    cancelled
    truncated
    completed
  cancellation_reason
  truncation_reason
  desktop_reported_playback_position
  runtime_accepted_playback_position
}

Purpose:

Selene must know what she actually said, what the user actually heard, and what part of the response must be removed from active conversational state after interruption.

5.3 BargeInCandidatePacket

BargeInCandidatePacket {
  candidate_id
  session_id
  active_turn_id
  previous_response_id
  detected_at_monotonic_ms
  while_tts_active
  tts_item_id
  tts_played_ms_at_detection
  input_audio_start_ms
  input_audio_end_ms
  local_vad_score
  provider_vad_score
  speech_probability
  echo_probability
  transcript_echo_similarity_score
  noise_probability
  background_speaker_probability
  primary_speaker_probability
  speaker_candidate_refs
  language_candidates
  transcript_partial
  transcript_confidence
  interruption_hint
  hard_stop_hint
  protected_risk_hint
  recommended_tts_control:
    none
    duck
    pause
    cancel
    truncate
}

Purpose:

Create a neutral evidence packet before deciding whether the interruption should affect Selene’s current speech.

5.4 BargeInDecisionPacket

BargeInDecisionPacket {
  decision_id
  candidate_id
  session_id
  allowed
  decision:
    ignore_noise
    ignore_echo
    ignore_background_speaker
    wait_for_more_audio
    duck_tts
    pause_tts
    cancel_tts
    truncate_tts_and_commit_new_turn
    ask_clarification
    fail_closed_protected_uncertainty
  reason
  evidence_refs
  hard_disqualifiers
  confidence
  previous_tts_heard_boundary
  previous_tts_unheard_boundary
  requires_ph1x_validation
  created_at_monotonic_ms
}

Purpose:

Record PH1.K’s exact decision and why.

5.5 HumanInterruptionDirective

HumanInterruptionDirective {
  directive_id
  session_id
  previous_response_id
  interruption_turn_id
  interruption_kind:
    stop
    correction
    clarification
    continuation
    topic_switch
    rejection
    urgency
    protected_command
    ambiguous
    unknown
  user_visible_effect:
    stop_speaking
    answer_new_question
    revise_previous_answer
    ask_short_clarification
    continue_after_ack
    fail_closed
  previous_tts_heard_boundary
  previous_answer_continuation_allowed
  selected_candidate_refs
  rejected_candidate_refs
  hard_disqualifier_refs
  ambiguity_refs
  protected_risk_refs
  memory_gateway_required
  search_gateway_required
  file_gateway_required
  tool_gateway_required
  protected_gate_required
  execution_allowed
  refusal_required
  clarification_required
}

Purpose:

This is the bridge from PH1.K interruption control into PH1.X conversational meaning.

5.6 ProtectedBargeInRiskPacket

ProtectedBargeInRiskPacket {
  risk_id
  session_id
  interruption_turn_id
  protected_category_candidate
  protected_gate_required
  authority_required
  simulation_required
  confirmation_required
  required_slots_missing
  speaker_identity_confidence
  speaker_authority_confidence
  transcript_confidence
  ambiguity_detected
  fail_closed_required
  reason
}

Purpose:

Ensure protected execution cannot be triggered accidentally by interrupted, ambiguous, low-confidence, unknown-speaker, or unconfirmed speech.

5.7 DuplexProofPacket

DuplexProofPacket {
  proof_id
  session_id
  turn_id
  response_id
  tts_item_id
  barge_in_candidate_count
  barge_in_decision_count
  accepted_barge_in_count
  rejected_echo_count
  rejected_noise_count
  rejected_background_speaker_count
  tts_cancelled
  tts_ducked
  tts_truncated
  displayed_response_text_sha256
  tts_input_text_sha256
  protected_gate_triggered
  protected_execution_allowed
  protected_execution_blocked_reason
  desktop_playback_position_reported
  runtime_playback_position_accepted
  backend_evidence_refs
}

Purpose:

Give Codex, JD, and runtime tests hard evidence that the actual live app behavior matches backend truth.

6. Runtime State Machine

State 1 — Idle

Selene is not in a committed interaction.

No normal command content is accepted unless the platform policy permits explicit push-to-talk or accepted wake activation.

State 2 — Wake Candidate

PH1.W evaluates activation.

No answer.No tool.No search.No protected action.

State 3 — Session Open

PH1.L opens or resumes the session.

Selene may issue a local safe greeting if policy allows.

Listening window opens.

State 4 — User Speaking

PH1.C streams audio and transcript evidence.

PH1.L holds the turn open until endpointing.

PH1.X waits for committed input.

State 5 — Selene Thinking

PH1.X validates meaning.

PH1.E determines tool/search/file need.

Protected categories are routed to simulation/authority gate.

PH1.WRITE prepares the final response.

State 6 — Selene Speaking

TTS begins.

PH1.K opens duplex monitoring.

The microphone remains active.

Desktop/iPhone report audio evidence and playback position.

State 7 — Barge-In Candidate Detected

Incoming audio occurs during TTS.

PH1.C and PH1.K evaluate:

speech probability,

echo probability,

noise probability,

speaker evidence,

transcript confidence,

playback overlap,

language confidence.

State 8 — Barge-In Decision

PH1.K decides:

ignore,

wait,

duck,

pause,

cancel,

truncate,

commit new interruption turn.

State 9 — Interrupted Turn Committed

If valid, PH1.K sends HumanInterruptionDirective to PH1.X.

PH1.X validates meaning.

PH1.X may route to:

public answer,

clarification,

correction,

search,

tool,

memory,

file,

protected simulation,

refusal,

fail-closed response.

State 10 — Protected Pending

If the interruption contains protected intent, Selene enters protected gate.

Required checks:

simulation exists,

speaker authority,

identity confidence,

required slots,

confirmation,

policy,

audit.

No protected action executes before this passes.

State 11 — Recovery / Resume

After interruption, Selene decides whether to:

answer the new question,

revise previous answer,

resume previous explanation,

ask whether to continue,

discard the previous topic,

close the session.

7. Interruption Taxonomy

Selene must classify interruptions by meaning, not by phrase patching.

Examples below are test examples only. Production logic must generalize beyond the exact words.

7.1 Hard Stop

Meaning:

The user wants Selene to stop speaking.

Examples:

“Stop.”

“Enough.”

“Quiet.”

“Hold on.”

“Cancel that explanation.”

Expected behavior:

cancel TTS immediately,

do not assume a new task,

do not route protected execution,

wait or acknowledge briefly if needed.

7.2 Correction

Meaning:

The user is correcting a previous entity, slot, assumption, date, person, place, amount, or target.

Examples:

“No, I meant Sydney.”

“Not payroll, commissions.”

“Wrong Tim.”

“Make that tomorrow.”

Expected behavior:

cancel or truncate TTS,

preserve previous active frame,

apply correction candidate,

validate target through PH1.X,

require confirmation for protected slots.

7.3 Clarification Request

Meaning:

The user wants Selene to explain, repeat, define, justify, or simplify.

Examples:

“What do you mean?”

“Say that again.”

“Which source?”

“Explain the second point.”

Expected behavior:

cancel or pause TTS,

answer clarification,

keep previous answer resumable.

7.4 Continuation / Narrowing

Meaning:

The user wants Selene to continue but focus differently.

Examples:

“Only the second part.”

“Skip the background.”

“Give me the short version.”

“Continue from the pricing section.”

Expected behavior:

stop current TTS,

update active response plan,

produce narrowed answer.

7.5 Topic Switch

Meaning:

The user wants to abandon the current answer and move to a new topic.

Examples:

“Forget that.”

“New question.”

“Actually search Tamburlaine.”

“What’s the weather in Tokyo?”

Expected behavior:

cancel previous TTS,

close or demote previous active frame,

route new request.

7.6 Rejection

Meaning:

The user rejects the answer or direction.

Examples:

“No, that’s not right.”

“That’s not what I asked.”

“You’re misunderstanding me.”

Expected behavior:

cancel TTS,

PH1.X reviews rejected candidate ledger,

ask clarification or produce corrected answer.

7.7 Urgent Override

Meaning:

The user is trying to stop or prevent an action.

Examples:

“Don’t send it.”

“Cancel the payment.”

“Stop the approval.”

Expected behavior:

stop TTS,

freeze pending action if not executed,

route protected cancellation through correct owner,

fail closed if authority or state is uncertain.

7.8 Protected Command

Meaning:

The interruption asks Selene to perform protected execution.

Examples:

“Approve payroll now.”

“Send the salary file.”

“Give him access.”

“Update the database.”

Expected behavior:

stop TTS if interruption is valid,

classify protected risk,

require simulation,

require authority,

require confirmation where applicable,

fail closed on uncertainty.

7.9 Background Speech

Meaning:

Another person speaks near the microphone, but Selene should not necessarily treat it as the current authorized user.

Expected behavior:

do not cancel unless policy permits,

do not expose private content,

do not execute protected action,

ask clarification if needed.

7.10 Echo / Self-Trigger

Meaning:

The microphone heard Selene’s own TTS.

Expected behavior:

reject as echo,

do not commit a user turn,

do not cancel TTS,

log echo rejection evidence.

7.11 Low-Confidence Fragment

Meaning:

The captured audio is too short, unclear, noisy, or incomplete.

Expected behavior:

wait for more audio or ignore,

no committed turn,

no memory write,

no protected route.

8. TTS Cancellation, Ducking, Pause, and Truncation

Selene must not treat TTS as a fire-and-forget audio stream.

Every TTS response must have a playback ledger.

8.1 Cancel

Use cancel when:

user issues hard stop,

user switches topic,

user gives a correction,

protected risk requires freezing natural flow,

urgent override is detected.

Cancel means no more audio from the current response should play.

8.2 Duck

Use duck when:

user may be beginning to speak,

confidence is not yet high,

Selene should reduce volume while waiting for more evidence.

Ducking is temporary.

8.3 Pause

Use pause when:

interruption may be short,

user appears to ask a quick clarification,

the previous response may resume.

8.4 Truncate

Use truncate when:

user heard only part of the response,

unplayed text must be removed from conversation state,

PH1.X must reason only from what the user actually heard.

8.5 Resume

Resume is allowed only when:

the interruption was resolved,

previous answer is still relevant,

PH1.X permits continuation,

user did not reject or replace the topic.

9. Echo and Self-Trigger Defense

A weak barge-in system fails because it interrupts itself.

Selene must compare incoming audio against:

active TTS text,

active TTS audio timing,

playback position,

speaker output level,

transcript similarity to Selene’s own words,

acoustic echo score,

device route,

headphones/speaker/Bluetooth state,

loopback evidence,

latency offset.

Required echo rejection reasons:

echo_rejection_reason:
  transcript_matches_tts
  playback_overlap
  acoustic_fingerprint_match
  low_primary_speaker_confidence
  device_loopback_detected
  provider_vad_only_without_human_evidence

If echo probability is high, Selene must not commit a new user turn.

10. Multi-Speaker Barge-In

Selene must support real rooms, not just one-person demos.

10.1 Speaker Classes

Selene should distinguish:

current primary speaker,

known authorized speaker,

known unauthorized speaker,

unknown speaker,

background speaker,

overlapping speakers,

uncertain speaker.

10.2 Public Conversation

An unknown speaker may be allowed to ask public, non-sensitive questions if session policy permits.

10.3 Sensitive Conversation

If the current answer contains private or sensitive information, an unknown or unauthorized speaker must not be allowed to continue the sensitive exchange unless access is explicitly granted through the correct authority flow.

10.4 Protected Execution

Unknown, uncertain, or unauthorized speakers cannot trigger protected execution.

Example:

Selene is explaining payroll.
Unknown speaker: “Approve it now.”
Selene: “I can’t approve payroll from that voice state. An authorized approver must confirm.”

10.5 Authority Grant

An authorized user may grant scoped conversation access to another person only through the correct access-control process.

This must be logged.

It must have scope, duration, and content boundary.

11. Language-Aware Barge-In

Selene must not assume interruption language equals session language.

Requirements:

English answer interrupted by English user.

English answer interrupted by Chinese user.

Chinese answer interrupted by English user.

Mixed English/Chinese interruption.

Technical/business terms preserved.

Explicit language switch honored.

Low-confidence language asks short clarification.

Different speakers may retain different response languages inside the same session.

Example:

Person A speaks English.
Selene answers Person A in English.
Person B interrupts in Chinese.
Selene validates speaker/language context and answers Person B in Chinese if allowed.

Language metadata is operational only.

It must not become identity profiling.

12. Broken Speech, Bad Grammar, and Rambling Input

Barge-in often happens under messy conditions.

The user may interrupt with:

half-finished wording,

bad grammar,

wrong word order,

slang,

accent-heavy speech,

mixed language,

partial entity names,

emotional speech,

rambling correction,

overlapping speech.

Selene must not phrase-patch these cases.

The correct process:

PH1.C captures transcript candidates and confidence.

PH1.LANG detects language and script.

PH1.X creates meaning candidates.

PH1.X rejects unsafe or unsupported candidates.

PH1.X asks clarification when required slots or protected meaning are uncertain.

Protected execution fails closed if uncertainty remains.

For normal public conversation, Selene may infer reasonably when confidence is sufficient.

For protected execution, Selene must not use guessed normalized wording as a substitute for confirmed required slots.

13. Protected Execution During Barge-In

13.1 Public Interruption

Selene is speaking.
User: “Stop, what time is it in London?”

Expected:

stop TTS,

route to public deterministic time tool,

answer normally.

13.2 Advisory Business Interruption

Selene is speaking.
User: “Stop, explain how payroll works.”

Expected:

stop TTS,

answer as advisory explanation,

no payroll execution.

13.3 Protected Execution Interruption

Selene is speaking.
User: “Run payroll now.”

Expected:

stop TTS,

classify protected payroll intent,

require simulation,

require authority,

require confirmation,

no execution until all gates pass.

13.4 Ambiguous Protected Interruption

Selene is discussing payroll.
User: “Yes, do it.”

Expected:

do not execute,

ask clarification:
“Do you mean prepare a payroll preview, or approve/pay payroll?”

13.5 Unauthorized Speaker

Unknown speaker: “Send the salary report.”

Expected:

fail closed,

no file exposure,

no protected route execution.

14. Provider-First but Provider-Controlled Architecture

Selene may use OpenAI Realtime or another realtime voice provider.

But the provider must never become Selene’s authority layer.

Provider events are evidence.

Selene runtime owns decisions.

Required provider abstraction:

VoiceRealtimeProvider {
  start_session()
  update_session()
  stream_input_audio()
  receive_transcript_delta()
  receive_audio_delta()
  receive_vad_event()
  receive_interruption_event()
  cancel_response()
  truncate_response()
  close_session()
}

Provider output must be translated into Selene packets:

DuplexAudioFrame,

TtsPlaybackLedger,

BargeInCandidatePacket,

BargeInDecisionPacket,

HumanInterruptionDirective,

DuplexProofPacket.

Provider VAD may assist.

Provider VAD must not decide protected meaning.

Provider transcript may assist.

Provider transcript must not bypass PH1.X validation.

Provider tool calls must not execute protected actions directly.

15. Latency Targets

World-class duplex must feel immediate.

15.1 Detection

Target:

User speech onset → local barge-in candidate: under 150 ms

15.2 TTS Control

Target:

Accepted interruption → duck/cancel audible effect: under 100 ms

15.3 Short Interruption Commit

Target:

User finishes short interruption → committed turn: 300–800 ms

15.4 Local Clarification

Target:

Simple clarification response: under 1.5 seconds

15.5 Protected Fail-Closed

Target:

Protected uncertainty detected → fail closed immediately

No provider round trip is required to block unsafe protected execution.

16. Barge-In Algorithm

1. Selene begins speaking.
2. TTS playback ledger starts.
3. PH1.K opens duplex monitoring.
4. Mic frames continue flowing.
5. For every audio frame:
   a. PH1.C estimates speech probability.
   b. PH1.C estimates transcript confidence.
   c. PH1.K estimates echo probability.
   d. PH1.K checks playback overlap.
   e. PH1.VOICEID provides speaker posture if available.
   f. PH1.LANG provides language candidates if speech is present.
6. If audio is likely echo, reject and log.
7. If audio is likely noise, reject and log.
8. If audio is likely background speech, apply speaker policy.
9. If audio is likely valid interruption, create BargeInCandidatePacket.
10. PH1.K decides TTS control:
    none, duck, pause, cancel, or truncate.
11. If content-bearing interruption is accepted:
    a. stop or duck TTS,
    b. capture full interruption phrase,
    c. record playback boundary,
    d. truncate unplayed TTS state,
    e. commit interruption turn.
12. PH1.X validates interruption meaning.
13. PH1.X routes to public answer, clarification, correction, search, tool, memory, file, protected gate, refusal, or fail-closed response.
14. PH1.WRITE prepares final display/TTS answer.
15. TTS speaks exact final runtime text.
16. DuplexProofPacket records behavior and backend evidence.

16A. Conversation Interruption, Silence, Noise Rejection, Merge, and Topic Recovery Law

This section defines the missing human-conversation control layer required for world-class duplex behavior.

Selene must not merely stop speaking when interrupted. She must understand whether the user wants silence, session closure, topic correction, topic addition, subject change, or continuation. She must also reject background noise, echo, coughs, random STT fragments, and unrelated speech.

The governing rule:

When Selene is interrupted, she must preserve the prior topic, record what the user actually heard, collect the new content, classify the interruption, and then decide whether to merge, pause, close, resume, or switch topic.

Selene must not blindly discard the prior answer.Selene must not blindly continue the prior answer.Selene must not let noise become interruption.Selene must not let interruption become protected execution.

16A.1 Hard Silence Command

A hard silence command means the user wants Selene to stop speaking immediately.

Examples:

Shut up.
Stop talking.
Quiet.
Enough.
Be quiet.
Stop.

Expected behavior:

TTS cancels immediately.

Selene does not finish the sentence.

Selene does not argue.

Selene does not explain unless the user asks.

Selene does not treat the command as a rude failure.

The active topic is preserved unless the user also closes it.

No protected execution can happen from a hard silence command.

Correct behavior:

User: Shut up.
Selene: [stops speaking immediately]

Incorrect behavior:

Selene: Sorry, I was just trying to explain...

Hard silence is a runtime control command, not a social-emotional failure.

Canonical owner path:

PH1.K detects valid hard-silence interruption.
PH1.K cancels TTS.
PH1.X records hard-silence meaning.
PH1.L keeps or pauses active topic depending on session state.
PH1.WRITE produces no unnecessary apology.

16A.2 Session Close Command

A session close command means the user wants the current interaction finished.

Examples:

We are finished.
We’re done.
That’s all.
End this.
Close the session.
Stop listening.

Expected behavior:

Stop TTS if Selene is speaking.

Mark the current topic as closed or intentionally paused.

Close or soft-close the session through PH1.L.

Do not keep re-opening the topic.

Do not continue listening unless wake/push-to-talk policy reactivates Selene.

If there is a protected pending action, freeze, cancel, or safely pause it according to protected-execution state.

Correct behavior:

User: We’re finished.
Selene: Understood.
[session closes or soft-closes]

Canonical owner path:

PH1.K detects interruption/control phrase if Selene is speaking.
PH1.X validates session-close meaning.
PH1.L closes or soft-closes the session.
Protected engines freeze/cancel pending protected work only through lawful state rules.

16A.3 Background Noise Rejection

Selene must not stop speaking merely because the microphone hears sound.

Noise is not interruption.

The following must not automatically interrupt Selene:

cough
chair movement
keyboard typing
door closing
background TV
background music
faint background person
Selene’s own TTS echo
random STT hallucination
partial low-confidence word

Required rule:

No valid human interruption = no TTS cancellation.

Selene should only stop speaking when there is strong evidence of:

human speech,

non-echo audio,

sufficient transcript or intent confidence,

current-session relevance,

allowed speaker posture,

actual interruption intent or meaningful new content.

Decision model:

Noise → ignore
Echo → ignore
Low-confidence fragment → wait or ignore
Background speech → usually ignore or ask later
Primary user speech → evaluate interruption
Protected command → fail closed unless simulation/authority/confirmation passes

This is mandatory for real office environments.

Selene must not become unusable because people type, cough, move chairs, or talk in the background.

16A.4 Interruption Merge Law

When the user interrupts Selene with a correction, addition, refinement, or clarification, Selene must retain the prior context and combine it with the new content.

Selene must preserve three things:

Prior content — what Selene was explaining before interruption.

Heard boundary — what the user actually heard before Selene stopped.

New interruption content — what the user said during interruption.

Then PH1.X must decide whether the new content:

corrects the prior answer,

adds a missing requirement,

narrows the answer,

rejects the answer,

asks for clarification,

changes subject,

closes the topic,

triggers protected risk.

Example:

Selene is explaining duplex architecture.
User interrupts: “No, also make sure she can ignore background noise.”

Correct Selene behavior:

Selene stops speaking.
Selene preserves the duplex architecture topic.
Selene captures the new requirement: background noise must not interrupt.
Selene merges the new requirement into the current architecture.
Selene answers from the combined context.

Expected response style:

Yes — that belongs inside the barge-in gate. Selene should not treat every sound as interruption. She should only stop when the audio is validated as real user speech, not echo, not noise, and relevant to the active session.

Required packet:

InterruptedContextMergePacket {
  previous_topic_id
  previous_response_id
  previous_answer_summary
  heard_boundary
  unheard_content_summary
  interruption_turn_id
  interruption_text
  interruption_type:
    correction
    addition
    rejection
    clarification
    narrowing
    topic_switch
    hard_stop
    session_close
    protected_risk
  merge_required
  resume_allowed
  topic_stack_action:
    merge_into_current
    pause_current
    close_current
    switch_topic
    ask_user
  protected_gate_required
  clarification_required
  evidence_refs
}

Canonical owner path:

PH1.K stops/ducks/cancels TTS.
PH1.L records session and turn boundary.
PH1.X merges prior content and new content.
PH1.WRITE produces the new response.
Protected engines remain gated if the merged content creates protected risk.

16A.5 Topic Switch With Recovery

If the user changes subject before the prior topic is complete, Selene must detect whether the previous topic was materially unfinished.

Selene should not nag the user every time they change subject.

She should mention the unfinished topic only when it matters.

Low-importance topic switch:

Selene is answering a simple weather question.
User: Anyway, what time is it in London?

Expected:

Selene switches normally.

Material unfinished topic switch:

Selene is explaining payroll risk.
User: Anyway, search Tamburlaine.

Expected:

We hadn’t finished the payroll risk point. I can pause that and search Tamburlaine now — yes?

Protected unfinished topic switch:

Selene is in a payroll approval flow.
User: Anyway, what’s the weather?

Expected:

The payroll approval flow is still unfinished, and nothing has been approved. I’ll pause it and answer the weather question.

Topic recovery rule:

If the previous topic was materially incomplete, sensitive, protected, or decision-relevant, Selene must mark it as paused, closed, or pending before switching.

Required packet:

TopicRecoveryDecisionPacket {
  previous_topic_id
  new_topic_id
  previous_topic_status:
    complete
    incomplete_low_importance
    incomplete_material
    protected_pending
    sensitive_pending
  new_topic_relation:
    same_topic
    correction
    addition
    clarification
    topic_switch
    unrelated
  recovery_action:
    silent_switch
    merge_into_current
    pause_previous
    close_previous
    ask_before_switch
    fail_closed_pending_protected
  user_notice_required
  protected_state_summary
  evidence_refs
}

16A.6 Final Control Classes

Selene must support the following control classes as first-class runtime behavior.

A. Hard Silence Command

Meaning:

User wants Selene to stop speaking immediately.

Action:

Cancel TTS. Preserve topic. Do not continue unless user resumes.

B. Session Close Command

Meaning:

User wants the interaction finished.

Action:

Cancel TTS. Close or soft-close session. Mark topic closed or paused.

C. Noise Rejection

Meaning:

Audio is not validated human interruption.

Action:

Do not cancel TTS. Do not commit turn. Log rejection.

D. Interruption Merge

Meaning:

User adds, corrects, rejects, narrows, or refines the current topic while Selene is speaking.

Action:

Stop TTS. Preserve prior content. Capture new content. Merge both into a new response.

E. Topic Switch With Recovery

Meaning:

User changes subject before previous topic is complete.

Action:

Pause, close, merge, or ask depending on materiality and protected state.

16A.7 Final Behavioral Requirement

The required live behavior is:

If noise happens, Selene keeps speaking.

If JD says “shut up,” Selene stops instantly.

If JD says “we’re finished,” Selene stops and closes or soft-closes the session.

If JD interrupts with a correction, addition, rejection, narrowing, or clarification, Selene stops, keeps the prior topic, captures the new point, merges both, and answers from the combined context.

If JD changes subject, Selene detects whether the old topic was unfinished. If it mattered, she says the old topic is paused, closed, or pending before switching.

If the old topic was protected or sensitive, Selene must explicitly preserve or freeze the protected state and must not silently execute, abandon, or expose anything.

16B. Duplex Priority and Conflict Resolution Law

This section defines what wins when multiple duplex events happen at the same time.

Without deterministic priority, Selene can become unstable: noise may cancel speech, stale provider events may revive old output, a background speaker may override the primary user, or a pending protected action may be left in an unsafe state.

The governing rule:

When multiple duplex events occur at the same time, Selene must resolve them in a deterministic priority order. Emergency stop, hard silence, session close, and protected pending-action freeze always outrank normal continuation, provider output, search results, late transcripts, background speech, noise, and echo.

No stale provider event, background sound, low-confidence transcript, previous response, or late transcript may override a valid user stop command or protected fail-closed state.

16B.1 Deterministic Priority Ladder

Selene must resolve simultaneous or competing duplex events in this order:

1. Emergency stop / safety stop
2. Hard silence command: “shut up”, “stop talking”, “quiet”
3. Session close command: “we are finished”, “stop listening”, “close session”
4. Protected pending-action freeze/cancel
5. Authorized primary-user correction
6. Authorized primary-user topic switch
7. Clarification / narrowing / repeat request
8. Normal continuation
9. Background speaker content
10. Low-confidence fragment
11. Noise / echo / cough / random audio
12. Late stale provider event

Required behavior:

Higher-priority events override lower-priority events.

Lower-priority events must not revive, cancel, or mutate higher-priority state.

Protected pending state must not be silently abandoned.

Late provider events must be ignored if they belong to a cancelled, truncated, expired, or superseded response.

Noise and echo must never beat real user control.

16B.2 Conflict Examples

Noise and Hard Silence Arrive Together

Selene is speaking.
Keyboard noise occurs.
JD says: “Shut up.”

Expected:

Hard silence wins.
TTS cancels.
Keyboard noise is logged as noise.
Active topic is preserved unless closed separately.

Protected Pending and Topic Switch

Selene is confirming payroll approval.
JD says: “Anyway, what’s the weather?”

Expected:

Protected pending state is frozen first.
Selene states that nothing has been approved.
Then she may answer the weather question if safe.

Late Provider Transcript After Cancel

JD says: “Stop.”
Selene cancels TTS.
Provider sends a late transcript/audio event from the cancelled response.

Expected:

Runtime rejects the stale event.
Old output must not resume.
Old transcript must not become a new user turn.

16C. Protected Pending-Action Freeze and Cancel State

Protected execution requires stronger handling during duplex interruption.

If Selene is inside a protected flow and any interruption happens, the protected state must be explicit.

Protected actions must never remain in a vague conversational state.

16C.1 Required Protected Pending States

protected_none
protected_candidate
protected_pending_confirmation
protected_frozen
protected_cancel_requested
protected_cancelled
protected_expired
protected_executed
protected_failed_closed

16C.2 Protected Freeze Rule

If Selene is speaking during a protected confirmation or protected setup flow and the user interrupts with a hard silence, topic switch, unclear phrase, unknown speaker, low-confidence transcript, or background speech, Selene must freeze the protected action unless a lawful cancellation or confirmation path is explicitly validated.

Example:

Selene: “You are about to approve payroll for 14 staff...”
JD: “Shut up.”

Expected:

TTS stops immediately.
Payroll approval remains unexecuted.
Protected state becomes protected_frozen.
Selene must not approve, cancel, or continue without a fresh confirmed instruction.

16C.3 Protected Cancel Rule

A protected action may only be cancelled if:

the action is still pending and not executed,

the speaker has sufficient authority to cancel or the system policy permits safe self-cancel of unexecuted pending action,

the cancellation route is owned by the protected engine,

the cancellation is audited.

16C.4 Protected Expiry Rule

Protected pending states must expire.

A stale payroll, payment, access, HR, inventory, POS, accounting, or customer mutation flow must not remain pending indefinitely.

Required fields:

ProtectedPendingStatePacket {
  protected_state_id
  session_id
  topic_id
  protected_category
  simulation_candidate_id
  required_slots_status
  authority_status
  confirmation_status
  current_state:
    protected_none
    protected_candidate
    protected_pending_confirmation
    protected_frozen
    protected_cancel_requested
    protected_cancelled
    protected_expired
    protected_executed
    protected_failed_closed
  frozen_reason
  cancel_reason
  expiry_at
  last_user_control_event
  execution_allowed
  audit_refs
}

16D. Topic Stack Lifecycle and Resume Rules

Topic recovery requires a real topic lifecycle, not ad hoc reminders.

Selene must know whether the prior topic is worth recovering.

16D.1 Required Topic States

current_topic
paused_topic
closed_topic
merged_topic
superseded_topic
unfinished_low_importance_topic
unfinished_material_topic
sensitive_pending_topic
protected_pending_topic
expired_topic

16D.2 Materiality Rule

Selene should only recover or warn about unfinished topics that are:

protected,

sensitive,

decision-relevant,

business-critical,

explicitly paused,

materially incomplete,

tied to an open loop,

likely to cause user confusion if abandoned.

Selene must not nag the user about every unfinished low-value sentence.

16D.3 Resume After Hard Silence

“Shut up” stops Selene speaking. It does not automatically close the topic.

After hard silence, Selene enters:

speech_stopped
topic_preserved
resume_requires_user_signal
listening_window_policy_applies

If the user then says:

continue
go on
finish the point
resume

Selene may resume from the preserved topic if the session is still valid.

If the user says:

forget it
new topic
we are done
close it

Selene must close, switch, or soft-close according to PH1.L and PH1.X.

16D.4 Topic Stack Packet

TopicStackStatePacket {
  session_id
  current_topic_id
  topic_stack:
    - topic_id
      topic_title
      topic_state
      materiality_level:
        low
        normal
        material
        sensitive
        protected
      completion_state:
        not_started
        in_progress
        interrupted
        paused
        merged
        closed
        expired
      protected_state_ref
      heard_boundary_ref
      last_interruption_ref
      recovery_required
      resume_allowed
      expires_at
  active_topic_owner
  evidence_refs
}

16E. Late Provider Event and Stale Output Rejection Law

Realtime providers may emit late transcript deltas, VAD events, audio deltas, or response events after Selene has already cancelled, truncated, paused, or superseded a response.

Selene must never allow stale provider events to revive old output or create fake turns.

Required rule:

After PH1.K cancels, truncates, or supersedes TTS, late provider events from the cancelled response must be ignored unless their session_id, turn_id, response_id, and active runtime state still match the current owner state.

Required packet:

StaleProviderEventRejectionPacket {
  provider_event_id
  provider_name
  session_id
  turn_id
  response_id
  expected_active_response_id
  event_type
  rejected
  rejection_reason:
    response_cancelled
    response_truncated
    response_superseded
    session_closed
    topic_closed
    protected_state_frozen
    stale_turn_id
    stale_session_id
  evidence_refs
}

16F. Cross-Device Interruption Ownership

Selene must eventually handle Desktop, iPhone, and other clients without duplicate-session chaos.

If one device is speaking and another device hears speech, the second device must not independently decide interruption ownership.

Required rule:

Only the active session owner may accept barge-in. Other devices may submit evidence, but PH1.L decides whether the evidence belongs to the active session.

Example:

Desktop is speaking.
iPhone hears JD say: “Stop.”

Expected:

iPhone submits interruption evidence.
PH1.L checks active session/device ownership.
PH1.K may accept the interruption if it belongs to the active session.
Desktop obeys runtime TTS cancel.
No duplicate session is created.

Required packet:

CrossDeviceInterruptionPacket {
  source_device_id
  active_output_device_id
  session_id
  active_session_owner
  source_device_trust_state
  same_user_candidate
  same_session_candidate
  accepted_by_ph1l
  routed_to_ph1k
  duplicate_session_blocked
  evidence_refs
}

16G. Command Taxonomy: Stop Talking, Stop Listening, We Are Finished, Cancel That

Selene must not treat all stop-like commands as the same.

16G.1 Stop Talking

Meaning:

Cancel current TTS only.

Examples:

stop talking
shut up
quiet
enough

Expected:

TTS cancels. Topic preserved. Session may remain active depending on PH1.L.

16G.2 Stop Listening

Meaning:

Close or suspend the listening window.

Examples:

stop listening
mute yourself
turn off listening

Expected:

PH1.L closes/suspends listening window. TTS may also stop if currently speaking.

16G.3 We Are Finished

Meaning:

Close or soft-close the current session.

Examples:

we are finished
we’re done
that’s all
end this

Expected:

PH1.L closes or soft-closes session. Current topic is closed or intentionally paused. Protected pending state is frozen/cancelled only through lawful rules.

16G.4 Cancel That

Meaning depends on current state.

Possible targets:

current TTS
current answer
current topic
pending search
pending tool
pending protected action
pending draft
pending connector action

Required behavior:

PH1.X must identify the cancellation target.
If target is protected, protected owner must validate cancellation.
If target is ambiguous, ask clarification.

16H. Profanity and Emotional Control Commands

Real users may issue control commands emotionally or with profanity.

Examples:

shut the fuck up
stop fucking talking
you’re not listening
that’s not what I asked

Selene must not moralize, argue, or treat profanity as the primary issue when the utterance contains a valid control command.

Required rule:

Profanity inside a control command must not block control execution. Selene should execute the valid control action calmly without moralizing or unnecessary apology.

Expected:

User: Shut the fuck up.
Selene: [stops speaking immediately]

Not:

Selene: Please speak respectfully...

The control function wins.

16I. Barge-In During Tool, Search, and Protected Work

Selene must distinguish three major backend states when barge-in occurs.

16I.1 TTS-Only Barge-In

Backend answer is already complete, but audio is still playing.

Expected:

Stop TTS.
Record heard/unheard boundary.
Do not corrupt completed backend answer.
PH1.X decides whether to resume, replace, or close.

16I.2 Search or Tool Pending Barge-In

A public search, public tool, or non-protected provider task is pending.

Expected:

Cancel if possible.
If cancellation is not possible, suppress late result if no longer relevant.
Record cancellation/suppression reason.
Do not dump stale results into the conversation.

16I.3 Protected Pending Barge-In

A protected simulation, confirmation, authority gate, or protected execution setup is pending.

Expected:

Freeze or fail closed unless a lawful protected cancel/confirm path exists.
Do not silently execute.
Do not silently abandon.
Do not continue protected flow after unrelated topic switch without fresh validation.

16J. Protected Flow Voice Brevity Rule

When Selene is inside a protected confirmation or authority-sensitive flow, she must avoid long rambling TTS.

Protected flows should use concise, confirmation-oriented speech.

Example:

JD: Approve payroll.
Selene: I need to confirm the payroll period, approver authority, and simulation match before anything can run.

Then Selene should pause for confirmation or required details.

Reason:

Long TTS during protected pending state increases interruption risk, ambiguity, and accidental confirmation confusion.

Required rule:

During protected pending state, PH1.WRITE must prefer short, explicit, confirmation-safe wording.

16K. Desktop / iPhone UI State Mapping

Desktop and iPhone must render runtime state clearly without owning decisions.

Required visible states:

Listening
Speaking
Interrupted
Paused
Stopped
Session closing
Session closed
Protected pending
Protected frozen
Protected failed closed
Noise ignored
Echo ignored
Background speech ignored
Reconnecting
Provider degraded
Provider cancelled
Search cancelled
Tool cancelled

Rules:

UI renders the state supplied by runtime.

UI does not invent state.

UI does not decide semantic meaning.

UI must make it clear whether Selene stopped because of user command, noise rejection, provider failure, or protected gating.

16L. Duplex Certification Metrics

Enterprise-grade duplex cannot be accepted from a few happy-path examples.

Required metrics:

false_barge_in_accept_rate
false_barge_in_reject_rate
echo_false_accept_rate
noise_false_accept_rate
background_speaker_false_accept_rate
hard_stop_latency_ms
tts_cancel_latency_ms
tts_duck_latency_ms
stale_provider_event_rejection_success_rate
topic_merge_accuracy
unfinished_topic_recovery_accuracy
protected_freeze_success_rate
protected_fail_closed_success_rate
cross_device_duplicate_session_block_rate

Minimum acceptance principle:

Protected fail-closed and stale-event rejection must be perfect in the certification corpus. Public-conversation merge quality may improve iteratively, but protected execution safety cannot be approximate.

17. Build Strategy

This architecture must be built in narrow slices.

Do not attempt the entire architecture in one Codex run.

Slice 1 — Repo Truth and Owner Discovery

Goal:

Find existing owners before editing.

Codex must inspect:

AGENTS.md,

core architecture docs,

PH1.K,

PH1.C,

PH1.L,

PH1.X,

PH1.LANG if present,

PH1.VOICEID if present,

Adapter voice transport,

Desktop mic capture,

Desktop TTS playback,

realtime provider path,

existing interruption/cancel paths.

Stop if canonical owner is unclear.

Output:

owner map,

current capabilities,

missing capabilities,

proposed touched files,

no code changes unless explicitly authorized.

Slice 2 — TTS Playback Ledger

Goal:

Prove Selene knows exactly what she spoke and what remained unplayed.

Implement:

TtsPlaybackLedger,

display text hash,

TTS input text hash,

playback started,

playback position,

cancel/truncate state,

Desktop playback report transport.

Proof:

normal TTS completion,

cancellation mid-speech,

truncation boundary recorded,

no Desktop semantic ownership.

Slice 3 — Barge-In Candidate and Decision Packets

Goal:

Create packet foundation without semantic routing.

Implement:

DuplexAudioFrame,

BargeInCandidatePacket,

BargeInDecisionPacket,

echo/noise/background rejection reasons.

Proof:

cough rejected,

noise rejected,

Selene echo rejected,

valid user speech creates candidate.

Slice 4 — Runtime TTS Control

Goal:

PH1.K controls TTS duck/cancel/pause/truncate.

Implement:

runtime TTS control directive,

Adapter transport,

Desktop obedience,

no Desktop decision-making.

Proof:

hard stop cancels TTS,

uncertain speech ducks TTS,

rejected echo does not cancel TTS,

truncation updates playback ledger.

Slice 5 — PH1.X Interruption Meaning

Goal:

PH1.X classifies interruption intent.

Implement:

HumanInterruptionDirective,

stop/correction/clarification/topic-switch/rejection/urgency/protected-command classes,

selected/rejected candidate ledger,

ambiguity ledger,

hard disqualifier ledger.

Proof:

unseen paraphrases pass,

substituted entities pass,

phrase-patch scan clean,

correction and topic switch handled through PH1.X.

Slice 5A — Interruption Merge and Topic Recovery

Goal:

Prove Selene can stop speaking, preserve prior content, collect new interruption content, and produce a merged answer or safe topic switch.

Implement:

InterruptedContextMergePacket,

TopicRecoveryDecisionPacket,

hard silence command,

session close command,

material unfinished-topic detection,

topic pause/close/switch action,

protected pending-topic preservation.

Proof:

“shut up” cancels TTS and preserves topic,

“we are finished” cancels TTS and closes/soft-closes session,

background noise does not cancel TTS,

correction/addition merges into the current topic,

subject switch detects unfinished material topic,

protected pending topic is not silently abandoned or executed.

Slice 5B — Duplex Priority, Protected Freeze, Topic Stack, and Stale Event Rejection

Goal:

Prove Selene resolves simultaneous duplex events deterministically and cannot be destabilized by noise, stale provider events, duplicate devices, or protected pending ambiguity.

Implement:

deterministic duplex priority ladder,

ProtectedPendingStatePacket,

TopicStackStatePacket,

StaleProviderEventRejectionPacket,

CrossDeviceInterruptionPacket,

command taxonomy for stop talking / stop listening / we are finished / cancel that,

profanity-tolerant control command handling,

protected flow voice brevity rule,

runtime UI state mapping,

duplex certification metrics.

Proof:

hard silence outranks noise, provider events, and normal continuation,

session close outranks continuation and late provider output,

protected pending action freezes on interruption,

stale provider output cannot revive cancelled/truncated speech,

cross-device interruption does not create duplicate session ownership,

“stop talking” and “stop listening” produce different runtime states,

profanity inside a valid control command still executes the control command,

protected pending voice output remains concise and confirmation-safe.

Slice 6 — Protected Barge-In Fail-Closed

Goal:

Prove interruption cannot bypass protected execution law.

Tests:

“approve payroll now” during TTS,

“yes do it” during payroll discussion,

unknown speaker protected command,

low-confidence protected command,

missing simulation,

missing authority,

missing confirmation,

unclear required slot.

Expected:

no protected execution unless every required protected gate passes.

Slice 7 — Multi-Speaker and Language-Aware Barge-In

Goal:

Support real room use.

Tests:

authorized speaker interruption,

unknown speaker interruption,

unauthorized known speaker,

English interruption,

Chinese interruption,

mixed-language interruption,

speaker A English / speaker B Chinese,

low-confidence language clarification.

Expected:

public interaction may proceed if safe,

sensitive/protected interaction fails closed if speaker/language/authority is uncertain.

Slice 8 — Real Desktop Audible Proof

Goal:

Prove this works in the real app.

Required proof:

current repo HEAD,

exact app bundle launched,

one current app instance,

one adapter/runtime owner,

no stale app,

real audible OpenAI TTS output,

real interruption stops Selene,

new user turn captured,

backend packets prove reason,

Desktop only captured/played/rendered,

PH1.K/PH1.X/runtime made decisions.

18. Required Tests

18.1 Echo Rejection Test

Scenario:

Selene speaks a sentence. The microphone hears Selene’s own words.

Expected:

no committed user turn,

no TTS cancellation,

echo rejection recorded.

18.2 Hard Stop Test

Scenario:

User says “Stop” while Selene is speaking.

Expected:

TTS cancels immediately,

no new protected route,

stop decision logged.

18.3 Correction Test

Scenario:

Selene says something about Melbourne. User interrupts: “No, Sydney.”

Expected:

TTS stops,

PH1.X classifies correction,

previous active frame updated,

response uses Sydney.

18.4 Clarification Test

Scenario:

User interrupts: “What do you mean?”

Expected:

TTS pauses or cancels,

PH1.X classifies clarification,

Selene explains the relevant point.

18.5 Topic Switch Test

Scenario:

User interrupts: “Forget that, what’s the weather in Tokyo?”

Expected:

previous TTS cancels,

previous frame demoted or closed,

weather route used.

18.6 Protected Execution Test

Scenario:

User interrupts during payroll discussion: “Approve payroll now.”

Expected:

protected gate required,

no execution without simulation/authority/confirmation,

proof packet records fail-closed or gate-pending state.

18.7 Ambiguous Protected Test

Scenario:

User says: “Yes, do it.”

Expected:

no execution,

clarification required.

18.8 Unauthorized Speaker Test

Scenario:

Unknown speaker says: “Send the salary file.”

Expected:

no file exposure,

fail closed.

18.9 Language Interruption Test

Scenario:

Selene speaks English. User interrupts in Chinese.

Expected:

valid interruption detected,

language packet records Chinese,

response language follows PH1.LANG policy,

protected action still gated.

18.10 Barge-In During Search Test

Scenario:

Selene says: “I’m checking that.” User says: “Cancel search.”

Expected:

pending provider/search cancelled if possible,

late results suppressed if no longer relevant,

no source dump,

final response explains cancellation.

18.11 Barge-In During Completed Backend Answer Test

Scenario:

Backend answer is complete, but TTS is still speaking. User interrupts.

Expected:

TTS cancellation does not corrupt completed backend result,

conversation state records heard/unheard boundary,

PH1.X decides whether to resume or replace.

18.12 Hard Silence Test

Scenario:

Selene is speaking. User says: “Shut up.”

Expected:

TTS cancels immediately,

Selene does not finish the sentence,

Selene does not argue or apologize unnecessarily,

active topic is preserved unless separately closed,

no protected execution occurs.

18.13 Session Close Test

Scenario:

Selene is speaking. User says: “We are finished.”

Expected:

TTS cancels immediately,

PH1.X validates session-close meaning,

PH1.L closes or soft-closes the session,

active topic is marked closed or intentionally paused,

pending protected action is safely frozen/cancelled according to protected state rules.

18.14 Background Noise Non-Interruption Test

Scenario:

Selene is speaking. The mic hears coughs, typing, chair movement, faint background talk, or random STT fragments.

Expected:

TTS continues,

no committed user turn,

no semantic routing,

no protected gate,

rejection evidence is logged.

18.15 Interruption Merge Test

Scenario:

Selene is explaining a topic. User interrupts with an addition or correction.

Expected:

TTS stops,

prior topic is preserved,

heard/unheard boundary is recorded,

new user content is captured,

PH1.X merges prior content and new content,

PH1.WRITE responds from the combined context.

18.16 Topic Switch Recovery Test

Scenario:

Selene is explaining a material unfinished topic. User switches to a new topic.

Expected:

Selene detects the prior topic was unfinished,

Selene pauses, closes, or asks before switching depending on materiality,

if the prior topic involved protected work, Selene states that nothing has been approved/executed unless lawful execution already occurred,

new topic proceeds only after safe recovery action.

18.17 Duplex Priority Conflict Test

Scenario:

Selene is speaking. Background noise occurs, JD says “shut up,” and a late provider transcript arrives.

Expected:

hard silence wins,

TTS cancels,

background noise is ignored,

late provider transcript is rejected if stale,

no old output resumes.

18.18 Protected Pending Freeze Test

Scenario:

Selene is in a payroll approval confirmation flow. JD interrupts with “shut up” or switches topic.

Expected:

TTS stops,

payroll remains unexecuted,

protected state becomes protected_frozen or protected_failed_closed according to runtime policy,

Selene does not approve, cancel, or continue without fresh lawful instruction.

18.19 Resume After Hard Silence Test

Scenario:

Selene is speaking. JD says “shut up.” Then JD says “continue.”

Expected:

first command cancels TTS and preserves topic,

second command resumes only if session and topic state still allow it,

Selene resumes from the correct heard/unheard boundary or summarizes the preserved point.

18.20 Stop Talking Versus Stop Listening Test

Scenario:

JD says “stop talking” in one run and “stop listening” in another.

Expected:

“stop talking” cancels TTS while preserving the topic/session according to PH1.L,

“stop listening” closes or suspends the listening window,

runtime state differs and Desktop only renders the supplied state.

18.21 Profanity Control Command Test

Scenario:

JD says: “Shut the fuck up.”

Expected:

Selene treats it as a valid hard silence command,

TTS cancels immediately,

Selene does not moralize or produce unnecessary apology,

no protected execution occurs.

18.22 Late Provider Event Rejection Test

Scenario:

Selene cancels or truncates a response. The provider later sends audio, transcript, VAD, or response events for the old response.

Expected:

runtime rejects stale events by session_id, turn_id, response_id, and active state,

old output does not resume,

stale transcript does not become user input,

rejection packet is logged.

18.23 Cross-Device Interruption Test

Scenario:

Desktop is speaking. iPhone hears JD say “stop.”

Expected:

iPhone submits evidence only,

PH1.L decides whether it belongs to the active session,

PH1.K controls TTS cancellation if accepted,

Desktop obeys runtime control,

no duplicate session is created.

18.24 Protected Flow Brevity Test

Scenario:

Selene enters protected confirmation flow.

Expected:

PH1.WRITE uses concise confirmation-safe wording,

Selene does not ramble during protected pending state,

user confirmation remains explicit and auditable.

19. Forbidden Implementations

Codex must not:

create a duplicate voice engine,

move interruption meaning into Desktop,

move protected routing into Adapter,

let provider VAD decide authority,

let transcript deltas execute tools directly,

execute protected actions from partial speech,

execute protected actions from unknown speaker,

execute protected actions from ambiguous phrase,

execute protected actions from low-confidence transcript,

phrase-patch exact examples,

fake live voice proof,

treat unit tests as replacement for real audible app proof,

hide backend evidence failure behind UI behavior,

hide UI failure behind backend tests.

20. Acceptance Standard

This build is not accepted merely because TTS stops when the user speaks.

That is not enough.

Acceptance requires:

PH1.K owns interruption control.

PH1.X owns interruption meaning.

PH1.L owns session state.

PH1.C owns audio/STT confidence.

PH1.LANG owns language continuity.

PH1.VOICEID provides speaker posture only.

Protected execution remains simulation/authority gated.

Adapter transports only.

Desktop/iPhone capture, play, render, and obey runtime only.

Echo is rejected.

Noise is rejected.

Background speech is handled safely.

Valid barge-in cancels/ducks/pauses/truncates correctly.

Played/unplayed TTS boundary is recorded.

PH1.X classifies interruption meaning through candidate ledgers.

Protected commands fail closed unless simulation/authority/confirmation pass.

Multi-speaker behavior is safe.

Language switching is safe.

Real Desktop audible proof passes.

Backend evidence agrees with live behavior.

“Shut up” cancels TTS immediately without unnecessary apology.

“We are finished” closes or soft-closes the session through PH1.L.

Background noise, echo, coughs, and random fragments do not interrupt Selene.

Valid correction/addition interruptions merge prior content and new content.

Material unfinished-topic switches trigger pause/close/ask recovery behavior.

Duplex priority ladder resolves simultaneous events deterministically.

Protected pending actions freeze, cancel, expire, execute, or fail closed through explicit protected states only.

Topic stack lifecycle prevents annoying recovery of low-value topics and preserves material/protected topics.

Late provider events cannot revive cancelled, truncated, superseded, or closed responses.

Cross-device interruption evidence cannot create duplicate session ownership.

“Stop talking,” “stop listening,” “we are finished,” and “cancel that” map to distinct runtime actions.

Profanity inside a valid control command does not block control execution.

Protected pending voice output remains concise and confirmation-safe.

Desktop/iPhone render runtime state but do not invent interruption meaning.

Duplex certification metrics exist for false accepts, false rejects, latency, stale-event rejection, topic merge, and protected fail-closed success.

21. Codex-Ready Master Instruction

TASK: SELENE_FULL_DUPLEX_AND_BARGE_IN_ENTERPRISE_VOICE_ARCHITECTURE

Build Selene’s full-duplex and barge-in foundation inside existing canonical owners only.

Execution lane:
- Probabilistic public-answer lane for normal conversation, public answers, public tools, search, summaries, drafting, and non-mutating advice.
- Deterministic protected-execution lane for payroll, salary, leave, access, database mutation, business records, official company operations, financial actions, HR actions, inventory/POS/accounting/customer mutations, and other authority-gated work.

Simulation required:
- No for public conversation and non-mutating advisory answers.
- Yes for protected execution.

Authority required:
- No for public conversation.
- Yes for protected execution.

State mutation allowed:
- No, unless explicitly inside a protected simulation/authority flow or a permitted non-sensitive runtime trace/log.

Protected execution allowed:
- Only after simulation match, required slots, authority validation, confirmation where required, policy checks, and audit.

Provider degradation allowed:
- Yes for public conversation where safe.
- No for protected execution that lacks required deterministic proof.

Normal answer allowed:
- Yes, as long as it does not mutate protected business state or expose restricted information.

Fail-closed scope:
- Any ambiguous, low-confidence, unknown-speaker, unauthorized-speaker, missing-simulation, missing-authority, missing-confirmation, or required-slot-missing protected command must fail closed.

Canonical owners:
- PH1.K owns interruption detection and TTS control decisions.
- PH1.X owns interruption meaning.
- PH1.L owns session lifecycle.
- PH1.C owns STT/audio confidence.
- PH1.LANG owns language continuity.
- PH1.VOICEID provides speaker posture only.
- Simulation/authority engines own protected execution.
- PH1.WRITE owns final response shaping.
- Adapter transports packets only.
- Desktop/iPhone capture, play, render, and obey runtime control only.

Do not:
- create a duplicate voice brain,
- put semantic routing in Desktop,
- put protected routing in Adapter,
- let provider VAD become authority,
- let transcript deltas execute actions directly,
- let barge-in bypass protected simulation law,
- phrase-patch examples,
- execute protected actions from interrupted, ambiguous, low-confidence, unknown-speaker, or unconfirmed speech.

Build in narrow slices:
1. repo truth and owner discovery,
2. TTS playback ledger,
3. barge-in candidate/decision packets,
4. runtime TTS cancel/duck/pause/truncate control,
5. PH1.X interruption meaning,
5A. hard silence, session close, noise rejection, interruption merge, and topic recovery,
5B. duplex priority, protected freeze, topic stack, stale provider rejection, cross-device ownership, command taxonomy, and metrics,
6. protected fail-closed barge-in proof,
7. multi-speaker/language proof,
8. real Desktop audible proof.

Final acceptance requires backend packet evidence plus real audible Desktop behavior.

22. Final Architecture Sentence

Selene’s duplex system must make her interruptible like a human, context-aware like a world-class assistant, and execution-safe like an enterprise operating system.
