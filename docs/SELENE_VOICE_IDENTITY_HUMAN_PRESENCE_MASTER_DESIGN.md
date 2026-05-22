Selene Voice Identity + Human Presence Master Design

DOCUMENT TYPE:
DEDICATED MASTER DESIGN / VOICE IDENTITY + HUMAN PRESENCE ARCHITECTURE

TASK:
SELENE_VOICE_IDENTITY_HUMAN_PRESENCE_MASTER_DESIGN

BUILD CLASS:
ARCHITECTURE / VOICE IDENTITY / HUMAN PRESENCE / ACCESS POSTURE / OPENAI-ASSISTED VOICE EXPERIENCE

STATUS:
MASTER DESIGN FOR FUTURE CODEX GRAND ARCHITECTURE RECONCILIATION

CONTROLLING DOCUMENTS:
1. AGENTS.md
2. Selene Master Architecture Build Set
3. Selene Final Overall Architecture Build Plan
4. Selene Overall Repo-Truth Activation Pack
5. Selene Identity + Access + Authority Spine Master Architecture
6. Selene Global Human Conversation Spine Master Architecture
7. Selene Global Request Decision Lattice + 5-Lane Business Risk View Master Design
8. Selene Universal Language Intelligence + Voice Capture Master Design
9. Selene Full Duplex and Barge-In Enterprise Voice Architecture
10. Selene PH1.WRITE — Structured Writing + Human Presentation Master Design
11. Selene PH1.M Human Memory Core Master Design
12. Celine Persona + Emotional Presentation Stack
13. Conversational Experience + Quick Assist Stack

PURPOSE:
Define Selene’s modern Voice Identity + Human Presence architecture so Selene can recognize, track, distinguish, and reason about speakers naturally while preserving strict boundaries between wake, session, transcript capture, speaker evidence, access, authority, memory scope, protected execution, OpenAI assistance, final response wording, and audit.

0. Master Law

Voice ID is not identity authority.

Voice ID is not access.

Voice ID is not permission.

Voice ID is not protected execution.

Voice ID is speaker evidence and human presence posture.

The global law is:

Wake decides whether Selene should listen.
Session decides where the turn belongs.
Capture decides what audio/transcript evidence exists.
Voice ID decides who might be speaking and whether the same speaker appears to continue.
Access decides what the resolved user may access.
Authority decides whether that user may perform the requested action now.
SimulationExecutor executes protected actions only when simulation, authority, confirmation, and audit pass.
GPT-5.5 helps Selene speak naturally and understand messy language.
PH1.X validates request meaning and risk.
PH1.WRITE produces final user-facing output.

Voice ID may produce evidence.

Voice ID must never grant authority.

1. Executive Target

Selene must feel natural when people speak to her.

She should be able to handle:

known speakers
unknown speakers
probable speakers
weak identity confidence
speaker changes
interruptions
multi-speaker rooms
cross-device sessions
new-device recovery
soft public conversation
protected business execution attempts
voice enrollment
natural identity acquisition
speaker-language continuity

But Selene must never confuse conversational familiarity with business authority.

The target system is:

human-like voice presence
+ soft conversational continuity
+ deterministic access and authority gates
+ protected fail-closed law
+ OpenAI-assisted natural wording
+ Selene-owned identity evidence and audit

2. Correct Architecture Split

The old architecture risk was making PH1.VOICE.ID too broad.

The modern split is:

PH1.W:
  wake / activation only

PH1.L:
  session lifecycle, turn ownership, active session, soft close, full close

PH1.C:
  audio capture, transcript quality, STT confidence, capture provenance

PH1.VOICE.ID:
  speaker evidence, speaker continuity, voice profile matching, liveness/replay posture, human presence

PH1.LANG:
  current-turn language, speaker-language history, language continuity

Access / Governance:
  user identity resolution, tenant scope, role scope, memory/tool/company data access

Authority:
  action-specific protected authority

SimulationExecutor:
  protected execution only

PH1.X:
  user meaning, request lattice, protected-risk classification, routing

PH1.M:
  governed memory access, private memory scope, preference memory

PH1.E:
  tools, search, files, external evidence

PH1.WRITE:
  final wording, Celine tone, identity-safe response, protected denial explanation

PH1.TTS:
  approved spoken output

Desktop / iPhone:
  capture, playback, render only

Adapter:
  transport and provenance only

No layer may absorb another layer’s authority for convenience.

3. OpenAI Role vs Selene Role

OpenAI may assist with:

STT / transcription
Realtime voice events
TTS
natural identity prompts
unknown-speaker conversation wording
multilingual/accented speech handling
broken speech interpretation
soft re-auth wording
voice enrollment phrase suggestions
Celine-style human phrasing
Quick Assist guidance
speaker-language help where allowed

OpenAI must not:

approve speaker identity
bind a user account
assign access
approve authority
execute protected actions
mutate business state
bypass access policy
bypass simulation
write final protected status
own audit truth

Provider output is evidence and language assistance.

Selene owns identity posture, scope, authority, final wording, and proof.

4. Core Responsibilities

4.1 PH1.VOICE.ID Owns

speaker candidate evidence
known / unknown / probable / rejected speaker posture
same-speaker continuity
speaker-change evidence
background speaker evidence
speaker overlap evidence
voice sample quality evidence
liveness / replay posture where available
local voice profile match evidence
cloud voice profile match evidence
local-cloud reconciliation evidence
provisional speaker acquisition evidence
voice enrollment evidence
speaker-language evidence handoff
identity uncertainty reason codes
voice identity audit evidence

4.2 PH1.VOICE.ID Does Not Own

wake acceptance
session lifecycle
final transcript truth
language output policy
semantic intent
memory permission
private data access
tool permission
authority
simulation
protected execution
business mutation
final response wording
Desktop/iPhone behavior
Adapter routing

4.3 Voice ID Output Is Always Evidence

PH1.VOICE.ID outputs posture such as:

unknown speaker
probable JD
confirmed candidate evidence
speaker changed
same speaker likely
background speaker likely
insufficient evidence
replay/liveness risk
local/cloud mismatch

It does not output:

JD is authorized.
JD may access payroll.
JD may approve this.
Execute action.

5. Human-Like Identity Experience

Selene should not sound like a prison scanner when public conversation is safe.

5.1 Soft Continuity Mode

When the speaker is weakly recognized or unknown and the request is public-safe, Selene may use soft continuity.

Soft continuity means:

continue natural public conversation
avoid using a name too early
collect more voice evidence if policy allows
avoid private memory and protected execution
avoid company data access
ask for name only when useful

Example:

Low confidence:
Good morning. What can I help with?

Confidence improves:
Got it, JD — I can help with that.

Confidence remains weak:
I don’t think we’ve connected this voice yet. What should I call you?

5.2 Unknown Speaker Public Flow

Unknown speaker asking a public question:

Wake accepted
Session opened
Voice ID unknown
Access public-safe only
PH1.X validates public answer
PH1.WRITE answers naturally

Example:

Unknown speaker: What is gross margin?
Selene: Gross margin is revenue minus cost of goods sold, shown as a percentage of revenue.

No identity block required.

5.3 Unknown Speaker Private / Protected Flow

Unknown speaker asking for private memory or company data:

Voice ID unknown
Access denied for private scope
PH1.WRITE explains respectfully

Unknown speaker asking for protected action:

Voice ID unknown
Access/Authority insufficient
Simulation not executed
Protected fail-closed
Audit recorded

Example:

Unknown speaker: Approve payroll.
Selene: I can’t approve payroll without verified authority and an approved payroll simulation.

6. Voice Identity Posture States

Voice identity should use posture states instead of pretending every voice is either known or unknown.

Required posture states:

NO_VOICE_EVIDENCE
INSUFFICIENT_AUDIO
UNKNOWN_SPEAKER
POSSIBLE_KNOWN_SPEAKER
PROBABLE_KNOWN_SPEAKER
CONFIRMED_SPEAKER_EVIDENCE
REJECTED_SPEAKER
SPEAKER_CHANGED
BACKGROUND_SPEAKER
AMBIGUOUS_OVERLAP
LIVENESS_UNKNOWN
LIVENESS_PASSED
LIVENESS_FAILED
LOCAL_CLOUD_CONFLICT
PROVISIONAL_SPEAKER

These states feed Access and PH1.X.

They do not grant access by themselves.

7. Public vs Private vs Protected Identity Modes

7.1 Public / Casual Mode

Allowed:

soft continuity
public conversation
public websearch
public explanation
public weather/time
non-mutating drafting
name discovery where appropriate
provisional non-authoritative speaker evidence
Celine natural wording

Not allowed:

private memory recall
company data read
protected business execution
access change
authority assertion

7.2 Private Read Mode

Examples:

What did I tell you yesterday?
Show Tim’s salary.
What was our gross margin last month?

Requires:

identity/access scope
tenant/workspace scope where company data involved
memory/privacy gate where memory involved
audit

May not require simulation unless state is changed.

7.3 Protected Execution Mode

Examples:

Approve payroll.
Increase Tim’s salary.
Refund this customer.
Grant access.
Send salary file.

Requires:

protected-risk classification
identity/access
authority
required slots
confirmation where required
simulation match
audit

No simulation means no execution.

Voice confidence alone is never enough.

8. Continuous Human Presence Tracking

Voice identity is not a one-time wake check.

Selene must track human presence throughout a session.

PH1.VOICE.ID should monitor:

initial speaker estimate
active speaker continuity
confidence drift
speaker interruption
speaker replacement
background speaker bleed
device/microphone change
noisy environment degradation
liveness/replay risk
session risk changes

If continuity breaks:

downgrade identity posture
suspend private/protected access if needed
notify Access/Governance
notify PH1.X
preserve public conversation only if safe
fail closed for protected work

9. Speaker Change Handling

Speaker change is evidence that the human context changed.

Speaker change must not automatically close the conversation, but it must update risk.

Public conversation

Unknown or changed speaker may continue if the request is public-safe.

Private conversation

If private memory or company data is involved, changed speaker must trigger access revalidation.

Protected execution

If protected execution is pending, speaker change must freeze or fail closed until identity and authority are revalidated.

Example:

Selene is discussing payroll.
New voice: Approve it now.

Expected:
No execution.
Identity/authority revalidation required.

10. Local + Cloud Voice Identity Architecture

Selene should support fast local evidence and resilient cloud identity.

10.1 Local Device Identity Cache

Local device identity cache may provide:

fast speaker estimate
recent user continuity
low-latency local confidence
offline/degraded posture where allowed

Local cache must be encrypted and scoped.

It must not become authority.

10.2 Selene Cloud Voice Identity Layer

Cloud identity may store or reference:

master voice profile references
encrypted voice profile refs
multi-device continuity evidence
recovery identity state
enrollment history
confidence history
audit refs

Cloud identity also does not grant authority by itself.

10.3 Local + Cloud Reconciliation

Possible outputs:

LOCAL_MATCH
CLOUD_MATCH
LOCAL_CLOUD_CONFLICT
NO_MATCH
REQUIRE_MORE_EVIDENCE

Example:

Local device: unknown
Cloud: probable JD
Session: soft continuity
Selene continues public conversation and gathers more evidence
Protected/private access remains gated

11. Provisional Speaker and Natural Acquisition

Selene should support natural identity acquisition without awkward enrollment rituals.

But acquisition must be governed.

11.1 Provisional Speaker

A provisional speaker is not a business identity.

It may contain:

provisional_speaker_id
display_name_candidate
voice_sample_refs
confidence_history
source_device_id
acquisition_status
consent/policy status
audit refs

11.2 Natural Name Discovery

For public-safe unknown speakers, Celine/PH1.WRITE may ask naturally:

I don’t think we’ve connected this voice yet. What should I call you?

or:

Let me get your name so I know how to address you next time.

OpenAI may draft the wording.

Selene decides whether acquisition is allowed.

11.3 Voice Sample Prompting

If more clean samples are needed, OpenAI may generate natural prompts.

Example:

Before I connect this voice, say a few short lines naturally. You can just say what you’d like Selene to help you with.

Deterministic voice identity owner decides whether samples are valid.

12. Voice Enrollment and Onboarding

Voice enrollment must connect to the Onboarding / Invite / Link / Enrollment stack.

Enrollment must support:

new speaker discovery
explicit user consent where required
voice sample collection
sample quality check
local/cloud profile association
identity proof requirements where needed
recovery on new device
revocation / re-enrollment

Enrollment must not:

grants protected authority automatically
bypass Access/Governance
bind voice to account without required policy
store raw audio casually
create unscoped identity records

13. Speaker-Language Binding

Voice identity can assist language continuity, but current turn language still wins unless approved preference applies.

Speaker-language data may include:

speaker_id or temporary label
current_turn_language
output_language
language_reason
speaker_language_history
explicit language preference
confidence
voice_id_confidence
speaker_known / unknown / uncertain

Rules:

Voice ID may help distinguish which speaker is speaking.
Voice ID must not force a previous speaker’s language onto a new speaker.
Current-turn language wins before persistent preference approval.
Unknown speaker uses current-turn language or short clarification.

Example:

Speaker A asks in English.
Selene answers English.

Speaker B asks in Chinese.
Selene answers Chinese.

Speaker A says: What about Sydney?
Selene answers English using Speaker A context if speaker continuity is proven.

14. Relationship To Full Duplex / Barge-In

Voice identity feeds full-duplex safety.

During barge-in, PH1.VOICE.ID may provide:

primary speaker candidate
background speaker candidate
same speaker likelihood
speaker changed signal
unknown speaker posture
speaker authority uncertainty

PH1.K owns barge-in control.

PH1.X owns interruption meaning.

PH1.VOICE.ID does not decide why interruption matters.

Example:

Selene is speaking.
Unknown speaker interrupts: Send the salary report.

PH1.K: valid interruption.
PH1.VOICE.ID: unknown speaker.
PH1.X: protected/private request.
Access/Authority: denied.
PH1.WRITE: respectful fail-closed explanation.

15. Relationship To PH1.M Human Memory

Memory access depends on identity/access scope.

PH1.VOICE.ID may help establish speaker evidence.

PH1.M decides memory use only after access scope allows it.

Rules:

unknown speaker → no private memory
probable speaker → policy-limited memory, if allowed
confirmed/access-scoped speaker → PH1.M may recall permitted memory
speaker changed → memory scope must be revalidated

Voice ID must not retrieve memory.

Voice ID must not decide memory relevance.

16. Relationship To Celine Persona + Quick Assist

Celine and Quick Assist make identity interaction feel natural.

They may help with:

unknown speaker prompts
soft continuity wording
re-auth wording
voice enrollment guidance
identity uncertainty explanations
session recovery wording
speaker-change explanations

They must not:

grant access
soften protected denial into approval
pretend identity is stronger than evidence
expose security details unnecessarily

Tone guidance:

public unknown speaker: warm and casual
private access denial: respectful and clear
protected fail-closed: serious and precise
onboarding/enrollment: friendly and guided
identity uncertainty: calm, not awkward

17. Data Model / Logical Packets

Codex must reuse current repo symbols where they exist and map these logical packets to current equivalents during activation.

17.1 VoiceIdentityEvidencePacket

voice_identity_evidence_id
session_id
turn_id
audio_ref
speaker_candidate_id optional
speaker_candidate_confidence
identity_posture
liveness_status
local_match_status
cloud_match_status
speaker_continuity_status
background_speaker_probability
same_speaker_probability
known_limitations
created_at
evidence_refs

17.2 SpeakerContinuityPacket

session_id
previous_speaker_ref
current_speaker_candidate
continuity_state
confidence
speaker_changed
background_speaker_detected
overlap_detected
risk_change
protected_pending_implication
evidence_refs

17.3 VoicePresenceSessionPacket

session_id
voice_presence_state
active_speaker_posture
identity_strength
access_scope_ref optional
language_posture_ref optional
session_state_ref
last_voice_evidence_ref
risk_flags
expires_at

17.4 LocalCloudVoiceReconcilePacket

session_id
local_match
cloud_match
conflict_status
selected_evidence_posture
requires_more_evidence
reason_code
evidence_refs

17.5 ProvisionalSpeakerPacket

provisional_speaker_id
session_id
display_name_candidate
voice_sample_refs
sample_quality
consent_status
acquisition_status
source_device_id
audit_refs

17.6 VoiceIdentityAccessHandoffPacket

session_id
turn_id
speaker_evidence_ref
identity_posture
resolved_user_candidate optional
access_resolution_required
private_memory_allowed_candidate false by default
company_data_allowed_candidate false by default
protected_action_allowed_candidate false by default
reason_code

17.7 ProtectedVoiceIdentityRiskPacket

session_id
turn_id
speaker_evidence_ref
protected_action_candidate
speaker_identity_confidence
speaker_authority_confidence
speaker_changed
unknown_speaker
liveness_risk
required_reauth
fail_closed_required
reason_code

17.8 VoiceIdentityAuditPacket

audit_id
session_id
turn_id
voice_identity_evidence_refs
access_handoff_refs
speaker_change_refs
protected_risk_refs
final_identity_posture
private_access_granted
protected_execution_allowed
protected_execution_blocked_reason
timestamp

18. State Model

18.1 Session Voice Presence States

NO_ACTIVE_VOICE_SESSION
WAKE_ACCEPTED_SESSION_PENDING
SESSION_OPEN_SPEAKER_UNKNOWN
SESSION_OPEN_SOFT_CONTINUITY
SESSION_OPEN_PROBABLE_SPEAKER
SESSION_OPEN_ACCESS_SCOPED
SESSION_OPEN_PROTECTED_REAUTH_REQUIRED
SESSION_SPEAKER_CHANGED
SESSION_SUSPENDED
SESSION_CLOSED

18.2 Identity Posture States

UNKNOWN
SOFT_CONTINUITY
PROBABLE
ACCESS_SCOPED
VERIFIED_FOR_PRIVATE_READ
REAUTH_REQUIRED_FOR_PROTECTED
REJECTED
SUSPENDED

18.3 Speaker Continuity States

SAME_SPEAKER
POSSIBLE_CHANGE
SPEAKER_CHANGED
AMBIGUOUS_OVERLAP
BACKGROUND_SPEAKER
INSUFFICIENT_EVIDENCE

These states are runtime posture states.

They are not all protected-execution simulations.

Protected execution remains simulation-gated.

19. Protected Execution Safety

Voice identity must strengthen protected execution safety, not weaken it.

Protected execution must fail closed if:

speaker unknown
speaker changed
speaker uncertain
liveness failed or unknown where required
required identity confidence missing
access scope missing
authority missing
simulation missing
confirmation missing
required slots missing

Voice ID must not fill protected slots from fuzzy speech.

Voice ID must not treat natural familiarity as authority.

Example:

Probable JD says: Approve payroll.

Still required:
access scope
authority
confirmation
simulation
audit

20. Security / Privacy / Compliance Rules

Rules:

raw audio is not persisted by default
voice samples are sensitive identity data
voice profiles must be encrypted/scoped
provider outputs are evidence only
voice identity decisions must be auditable
speaker-change events must suspend protected pending actions where relevant
local cache must be encrypted
cloud profile refs must be scoped
unknown speakers must not access private memory/company data
voice identity must not infer protected attributes

Voice may be used as evidence.

Voice must not become unchecked surveillance.

21. Error Categories

Required error / posture categories:

NO_SIMULATION_FOUND
PROVIDER_SERVICE_FAILURE
AMBIGUOUS_IDENTITY
INSUFFICIENT_VOICE_EVIDENCE
LOCAL_CLOUD_CONFLICT
SPEAKER_CHANGED_DURING_PRIVATE_CONTEXT
SPEAKER_CHANGED_DURING_PROTECTED_ACTION
SESSION_INTEGRITY_VIOLATION
LIVENESS_RISK
ACCESS_SCOPE_MISSING
AUTHORITY_MISSING
UNKNOWN_SPEAKER_PRIVATE_REQUEST
UNKNOWN_SPEAKER_PROTECTED_REQUEST

Each error should have:

user-safe wording
backend reason code
audit evidence
owner assignment
next lawful action

22. Required Proof Categories

Codex must eventually prove:

known speaker public conversation
unknown speaker public conversation
unknown speaker private memory denial
unknown speaker company data denial
unknown speaker protected fail-closed
probable speaker public soft continuity
speaker change during public conversation
speaker change during private context
speaker change during protected pending state
local match
cloud match
local-cloud conflict
provisional speaker acquisition
voice enrollment handoff
speaker-language binding
same speaker continuity
background speaker rejection
liveness/replay risk handling
Celine natural unknown-speaker wording
PH1.WRITE protected denial wording
Desktop render-only
Adapter transport-only

No live voice identity behavior is accepted without backend evidence.

23. Build Strategy

Do not build everything at once.

Build 0 — Voice Identity Repo-Truth Activation Pack

Map existing:

PH1.VOICE.ID contracts
voice enrollment tables
speaker evidence symbols
Voice ID engine behavior
session handoff surfaces
Access/Governance integration
Desktop/iPhone voice identity surfaces
Adapter voice surfaces
OpenAI STT/TTS/realtime inputs
old identity shortcuts
tests/evals

No implementation.

Build 1 — Voice Identity Evidence Contract Mapping

Map or create repo-equivalent:

VoiceIdentityEvidencePacket
SpeakerContinuityPacket
VoiceIdentityAccessHandoffPacket
VoiceIdentityAuditPacket

No authority grant.

Build 2 — Evidence-Only Voice ID Proof

Prove Voice ID can emit speaker posture without granting access or authority.

Build 3 — Soft Continuity + Unknown Speaker Public Flow

Allow safe public conversation without awkward identity failure.

Build 4 — Access Handoff For Private Memory / Company Data

Voice ID hands evidence to Access.

Access decides scope.

PH1.M/PH1.E/private data owners enforce scope.

Build 5 — Speaker Change / Continuity Tracking

Track same speaker, changed speaker, background speaker, ambiguous overlap.

Protected pending state freezes where needed.

Build 6 — Local + Cloud Reconciliation

Support local cache + cloud profile evidence without auto-authority.

Build 7 — Provisional Speaker + Onboarding Handoff

Support natural acquisition and voice enrollment through approved onboarding/access flow.

Build 8 — Speaker-Language Binding

Connect to Universal Language Intelligence and PH1.LANG.

Current-turn language still wins unless approved preference applies.

Build 9 — Protected Voice Identity Fail-Closed Matrix

Prove unknown/probable/changed speaker cannot execute protected actions without Access + Authority + Simulation.

Build 10 — JD Live Voice Identity Proof

Real app proof:

known speaker
unknown speaker
speaker change
private denial
protected fail-closed
Celine natural wording
backend evidence agreement

24. What Codex Must Not Do

Codex must not:

merge Wake and Voice ID into one brain
make PH1.VOICE.ID own session lifecycle
make PH1.VOICE.ID own access permissions
make PH1.VOICE.ID own authority
make PH1.VOICE.ID execute protected actions
let OpenAI approve identity
let OpenAI bind accounts
let Desktop decide identity
let Adapter decide identity
create a second identity system
store raw audio casually
infer protected attributes from voice/accent/language
use Voice ID as a shortcut for business execution

25. Success Standard

The Voice Identity + Human Presence system is successful when Selene can:

recognize known speakers naturally
handle unknown speakers without awkward public-flow failure
track whether the same person is still speaking
detect speaker changes safely
handle background speakers
use local and cloud voice identity evidence
support new-device recovery
support provisional speaker acquisition
support voice enrollment through onboarding
support speaker-language continuity
allow public conversation for unknown speakers where policy permits
deny private memory/company data without access
fail closed for protected execution when identity/access/authority is insufficient
use OpenAI to speak naturally without letting OpenAI decide authority
produce backend evidence and audit for every identity-sensitive turn

26. Final Architecture Sentence

Selene Voice Identity is the human presence layer.

It helps Selene understand who may be speaking and whether that speaker appears to continue across the session.

It does not grant power.

The final architecture is:

Wake opens the door.
Session owns the room.
Capture hears the speech.
Voice ID recognizes the presence.
Access decides the scope.
Authority decides the action.
Simulation executes only when lawful.
PH1.X understands the request.
PH1.WRITE speaks like a human.

That is how Selene becomes natural with people and safe with power.
