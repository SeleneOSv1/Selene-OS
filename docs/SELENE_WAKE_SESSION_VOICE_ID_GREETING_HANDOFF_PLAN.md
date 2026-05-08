# Selene Wake -> Session -> Voice ID -> Greeting Handoff Plan

Status: MASTER_SYSTEM_CHANGE_PLAN
Repo Root: `/Users/selene/Documents/Selene-OS`

This document captures JD's controlling target design for Selene activation, wake/session handoff, Voice ID posture, greeting acknowledgement, PersonProfile handling, local/cloud recognition posture, guest interaction, and progressive voice learning.

This is part of the Selene master system. It is not a standalone build, not a new numbered stage, and not a production deployment authorization. Implementation must proceed through controlled slices that preserve Selene's existing authority, provider, privacy, audit, client-renderer, and protected-execution boundaries.

## Verbatim JD Design

Selene Wake → Session → Voice ID → Greeting Handoff Plan

Status

Planning draft v0.5. This is the current controlling design for wake acknowledgement, device context, first-time speaker handling, random guest interaction, PersonProfile creation, progressive Voice ID learning, and greeting-library policy.

Core Target Flow

Activation trigger is accepted
→ session opens/resumes
→ activation audio / wake audio / pre-roll is passed to Voice ID where available
→ device context is attached as a signal
→ Voice ID attempts speaker recognition using local device cache first
→ cloud Voice ID fallback runs if local recognition is missing, weak, or unavailable
→ if local and cloud both fail, speaker is treated as new / unknown
→ greeting policy chooses a safe acknowledgement
→ TTS speaks the greeting
→ listening window stays open for the user’s command
→ Voice ID continues improving speaker recognition from clean speech

Activation trigger depends on platform:

Mac / Windows / Android:
wake word such as “Selene” may be the activation trigger.

iPhone:
side button / push-to-talk is the activation trigger.
The side button becomes the iPhone equivalent of wake.
After side-button activation, the same session → Voice ID → greeting → listening flow begins.

The architecture after activation is the same across platforms.

Core Separation Law

Activation trigger = wake word, side button, push-to-talk, keyboard, or approved explicit trigger
Wake = activation only on wake-capable platforms
Side button = iPhone activation equivalent to wake
Session = interaction container
Device context = location / device trust signal
Voice ID = speaker identity posture signal
PersonProfile = governed person record
VoiceProfile = voice recognition model linked to a person or candidate
Greeting policy = acknowledgement selector
TTS = clean spoken acknowledgement
Listening window = command capture
Memory = governed personal context
Authority = separate protected gate

Wake and iPhone side-button activation must not become the brain.

Wake / activation trigger is responsible only for:

detect activation
open/resume session
hand off activation context

Wake must not directly do:

LLM calls
search
tools
provider calls
TTS policy
protected execution
authority decisions
business actions
memory decisions

The new behavior belongs in an activation-to-session handoff layer after wake or side-button activation and session open.

Platform Activation Rule

Selene must separate activation method from the rest of the voice flow.

Activation methods may differ by platform, but the downstream runtime flow should stay consistent.

Mac / Windows / Android:
wake word accepted
→ session opens/resumes
→ wake/pre-roll audio can be used for Voice ID
→ greeting policy runs
→ listening window starts

iPhone:
side button / push-to-talk accepted
→ session opens/resumes
→ captured activation audio can be used for Voice ID where available
→ greeting policy runs
→ listening window starts

For iPhone, the side button is treated as the activation equivalent of wake.

The iPhone does not need always-listening wake detection for this flow. Once the side button starts the session, Selene continues through the same Voice ID, greeting, TTS, and listening pipeline.

Device Context Rule

Selene should use device context as a second signal, but not as proof of who is speaking.

Example:

Device = JD’s iPhone
Voice = JD
Result = very high confidence JD

Different example:

Device = JD’s iPhone
Voice = unknown person
Result = trusted device, unknown speaker

In that situation Selene must not say:

“Hi JD.”

Correct behavior:

“Hi, I’m Selene. I don’t recognize your voice yet. What should I call you?”

The rule is:

Device confirms where the request came from.
Voice ID confirms who is speaking.
Authority decides what they can do.

Device trust may raise confidence, but it must not override a mismatched or unknown voice.

Local Device / Cloud Voice ID Sync

Selene should use a local-first, cloud-fallback model for wake and Voice ID.

The clean recognition path is:

Local first
→ cloud fallback
→ if no local match and no cloud match
→ treat as new / unknown speaker

Important correction:

No local match + cloud unavailable
≠ new person

No local match + cloud unavailable
= not recognized on this device right now

Only this should mean new / unknown:

No local match
+ cloud checked
+ no cloud match
= new / unknown speaker

Local / Cloud Architecture

Recommended architecture:

Device stores fast local recognition cache.
Cloud stores durable authoritative PersonProfile / VoiceProfile.
Devices sync approved voice updates back to cloud.
Cloud pushes updated recognition packs back to approved devices.

This gives Selene:

speed
smoothness
cross-device recognition
lost-device recovery
central governance

Local-First Wake Rule

Wake word detection should run locally on the device.

Reason:

Wake must be instant.
Wake must work offline.
Wake must not wait for cloud.
Wake must not stream everything to cloud before activation.

For Mac / Windows / Android:

Local wake detector hears “Selene”
→ activation accepted
→ session opens

For iPhone:

Side button / push-to-talk
→ activation accepted
→ session opens

Wake word data should be mostly local because wake is only activation. It should not identify the person.

Best rule:

Wake detects that Selene was called.
Voice ID determines who called her.

Local-First Voice ID With Cloud Fallback

Voice ID should check local recognition first, then cloud if needed.

Runtime flow:

1. User activates Selene.
2. Device checks local VoiceProfile cache.
3. If local high-confidence match:
   - use that identity posture immediately.
4. If local low-confidence or no match:
   - send approved voice evidence / embedding to cloud.
5. Cloud checks global PersonProfile / VoiceProfile records.
6. If cloud matches:
   - return matched speaker posture.
   - update local device cache.
7. If cloud does not match:
   - create UnknownSpeakerCandidate / SpeakerProfileCandidate.

Example:

Mark speaks to Selene on JD’s phone.
JD’s phone has never heard Mark before.
Local match fails.
Cloud checks and finds Mark’s VoiceProfile from another device.
Selene says:
“I think this is Mark.”

Local Device Storage Rule

Local device storage is a cache, not the authority.

The phone or desktop should store enough information to make Selene fast and smooth, but the cloud should remain the durable authority.

Recommended split:

Local device:
- wake model
- recent VoiceProfile cache
- device trust record
- recent speaker embeddings
- recent greeting / history state
- offline-safe session state

Cloud:
- PersonProfile
- VoiceProfile master record
- VoiceProfile version history
- onboarding / consent status
- access policy
- memory scope
- audit trail
- device list
- revocation state

The device is for speed.

The cloud is for continuity, lost-device recovery, cross-device recognition, and governance.

Lost Device Rule

Because the cloud is authoritative, lost-device handling should work like this:

Device is reported lost
→ cloud revokes device trust
→ local recognition cache becomes invalid
→ future sync is blocked
→ replacement device downloads approved fresh recognition pack

This means the cloud is not just a backup. It is the durable authority.

Local Storage Contents

Locally store:

encrypted voice embeddings
not raw audio by default

recognition model cache
not full unrestricted biometric history

profile IDs
not full personal memory

device trust tokens
not authority by themselves

recent session state
only if allowed by retention policy

Do not store permanent raw microphone audio locally unless there is a specific governed reason.

Safe default:

Raw audio is temporary.
Embeddings / features are retained.
Profile updates are versioned.

Cloud Update Loop

Every clean conversation should improve recognition when policy allows.

Cloud update loop:

Clean speech sample captured
→ quality checked
→ bad samples rejected
→ local profile confidence updated
→ cloud receives approved voice evidence
→ cloud updates master VoiceProfile
→ cloud pushes updated compact recognition pack to devices

This allows Selene to become better at recognizing JD and other speakers over time.

Bad samples must still be rejected:

TV audio
two people speaking
Selene’s own TTS echo
unclear audio
spoof risk
low-quality mic
background speech

Device ↔ Cloud Sync Timing Policy

Selene should not sync every minute by default.

Constant syncing would waste battery, increase cloud cost, increase privacy exposure, and create noisy system behaviour.

The correct model is:

Event-driven sync
+ lightweight periodic checks as backup
+ immediate cloud fallback only when needed

Recommended rule:

Do not sync constantly.
Sync when something important changes.
Use periodic checks only as backup.

Best Device ↔ Cloud Sync Model

1. On App Start / Device Unlock

The device should perform a lightweight cloud check for:

device trust status
revocation status
latest PersonProfile version
latest VoiceProfile pack version
policy changes
lost-device blocks

This check should be lightweight and version-based.

2. On Selene Activation

When the user says “Selene” or presses the iPhone side button / push-to-talk trigger:

wake/session starts instantly from local device
cloud sync runs in parallel if internet exists
wake must not wait for cloud
side-button activation must not wait for cloud

The user experience must stay fast.

3. When Local Voice ID Fails

This is the most important sync moment.

local no match / weak match
→ immediate cloud fallback
→ cloud checks master PersonProfile / VoiceProfile records

This should happen right away because it decides whether the speaker is known somewhere else.

4. After a Conversation Ends

This is when the device should upload approved voice-learning updates.

session ends
→ approved clean speech samples / embeddings are batched
→ bad samples are rejected
→ cloud updates master VoiceProfile
→ cloud prepares updated recognition pack

Do not upload every few seconds while the person is talking unless there is a strong governed reason.

5. When Cloud Has Important Changes

Cloud should push updates to devices when needed.

Immediate push for:

lost device
revoked device
revoked user
blocked profile
access policy change
VoiceProfile security issue
authority change

Normal push for:

new VoiceProfile version
new PersonProfile link
new greeting library version
new device trust update

Sync Timing Recommendation

Use this timing model:

Critical security update:
immediate push

Activation/session start:
quick cloud version check in parallel

Local Voice ID fail:
immediate cloud fallback

After session:
upload approved voice updates within 30–120 seconds

While app is active:
lightweight version check every 10–15 minutes

Background:
use OS push notifications, not constant polling

When charging + Wi‑Fi:
larger recognition pack refresh

Offline:
queue updates locally and sync when online

iPhone Sync Rule

Do not expect iPhone to sync every minute in the background. iOS background behaviour will limit that anyway.

Better iPhone rule:

sync on app open
sync on side-button activation
sync after session
sync from push notification
sync when network returns
sync larger packs when charging / Wi‑Fi

Version-Based Sync Design

Use version numbers for all major syncable identity and recognition objects.

Example:

Local VoiceProfile pack version: 42
Cloud VoiceProfile pack version: 44

Device sees cloud is newer
→ downloads delta update
→ local cache becomes version 44

This avoids unnecessary syncing.

Versioned objects should include:

PersonProfile version
VoiceProfile version
VoiceProfile recognition pack version
device trust version
greeting library version
access policy version
revocation state version

Final Sync Policy

Every minute by default: no.

Event-driven sync: yes.

Immediate cloud fallback when local recognition fails: yes.

After-session upload for voice learning: yes.

Push critical updates instantly: yes.

Periodic lightweight check every 10–15 minutes while active: yes.

Large model/profile updates only when needed, preferably Wi‑Fi/charging: yes.

Best sync law:

Local recognition keeps Selene fast.
Cloud fallback keeps Selene accurate.
Cloud push keeps Selene safe.
Periodic sync is only backup, not the main method.

Local / Cloud Guest Speaker Flow

For a random speaker:

Local no match
→ cloud no match
→ new UnknownSpeakerCandidate
→ Selene asks name
→ creates SpeakerProfileCandidate
→ public-safe access only

If the person later onboards:

Onboarding completed
→ cloud checks existing candidates
→ links matching voice history
→ promotes candidate to PersonProfile

This means Selene can recognize someone from the beginning of onboarding if she has heard that person before.

Local / Cloud Security Rule

Device trust and voice recognition are separate.

JD’s phone + JD’s voice = high confidence JD.

JD’s phone + unknown voice = unknown speaker on trusted device.

Unknown device + JD’s voice = JD voice on untrusted device.

The third case is important. Selene may know it is JD, but the device is still not trusted.

For protected actions:

Voice ID alone is not enough.
Device trust alone is not enough.
Both together may still require stronger auth.

Local / Cloud Final Recommendation

Wake:
local-first.

Voice ID:
local cache first,
cloud fallback second,
new speaker only if both fail.

PersonProfile:
cloud-authoritative.

VoiceProfile:
cloud-authoritative master,
local encrypted cache for speed.

Device:
trust signal, not identity proof.

Guest speaker:
allowed public-safe conversation,
candidate profile only,
no personal/protected access.

Lost device:
cloud revokes device and rebuilds cache on replacement.

Final local / cloud law:

Local makes Selene fast.
Cloud makes Selene continuous.
Device helps trust the session.
Voice ID identifies the speaker.
PersonProfile remembers the person.
Authority remains separate.

Identity Situation Classes

Selene must treat these situations differently.

1. Known Voice on Known / Trusted Device

Device = known / trusted
Voice = recognized high confidence
Result = known person posture

Allowed behavior:

Greet by name if natural.
Use the existing PersonProfile.
Continue improving VoiceProfile from clean speech.
Apply access policy for any personal or protected request.

Example:

“Hi JD, I’m here.”

2. Unknown Voice on Known / Trusted Device

Device = JD’s phone or computer
Voice = unknown or mismatched
Result = trusted device, unknown speaker posture

Allowed behavior:

Do not assume the speaker is JD.
Use generic or first-time speaker greeting.
Ask for the person’s name after enough clean speech.
Create a candidate profile if policy allows.
Public-safe access only until verified.

3. Known Voice on New / Unknown Device

Device = unknown or new
Voice = recognized high confidence
Result = known speaker on new device

Allowed behavior:

Recognize the person by Voice ID.
Treat the device as lower trust until device policy confirms it.
Allow only the access level permitted by voice + policy.
Require stronger confirmation for sensitive or protected requests.

The VoiceProfile should be global to the person, not locked to one device.

4. Random / Non-Onboarded Speaker

Selene should allow random people to speak with her in a public-safe guest lane.

Allowed:

public-safe conversation
general questions
basic help
public web/search if enabled
asking what Selene can do

Not allowed:

personal memory access
business data access
email/calendar/customer/payroll/accounting access
protected execution
state mutation
private user data
company data
simulation execution

Example allowed:

Random person:
“Selene, what can you do?”

Allowed.

Example blocked:

Random person:
“Selene, read JD’s messages.”

Blocked.

Safe response:

“I can’t access personal information until I know who I’m speaking with and the right authority is confirmed.”

Profile Model

Selene should remember people long-term, but the system must separate candidate identity, verified person identity, voice biometrics, access, memory, and authority.

Recommended structures:

UnknownSpeakerCandidate
→ someone Selene has heard but not identified

SpeakerProfileCandidate
→ someone who gave a name but is not fully verified

PersonProfile
→ governed long-term person record

VoiceProfile
→ voice recognition model linked to the person

VoiceProfileCandidate
→ provisional voice evidence linked to a candidate

SessionSpeakerPosture
→ runtime status for the current session

PersonProfile Law

PersonProfile is the official long-term governed person structure.

It should link separate systems:

PersonProfile
→ person_profile_id
→ name / preferred name
→ known aliases
→ VoiceProfile reference
→ onboarding consent reference if onboarded
→ memory scope reference
→ preferences
→ access policy reference
→ device trust associations
→ audit references
→ created_at
→ last_seen_at
→ profile_status

Voice data belongs to Voice ID.

Memories belong to memory governance.

Authority belongs to access policy and execution governance.

Audit belongs to audit.

The PersonProfile connects these systems, but does not turn Voice ID into memory or authority by itself.

Candidate Profile Flow

When Selene hears someone for the first time, she should not treat the person as invisible.

If the speaker is unknown, the first posture is:

UNKNOWN_SPEAKER_CANDIDATE

After the person gives a name, the posture becomes:

SPEAKER_PROFILE_CANDIDATE

Example:

Random person speaks on JD’s phone.
Selene does not know the voice.
Person says: “I’m Mark.”
Selene creates:
- SpeakerProfileCandidate: Mark
- VoiceProfileCandidate: Mark voice evidence
- Authority: none
- Access: public-safe only

Next time that person speaks, even on another device, Selene should try to match the voice and may say:

“I think this may be Mark. Is that right?”

Once confirmed through onboarding, verification, admin policy, or another approved identity process, the candidate can be promoted to a full PersonProfile.

Durable Recognition Rule

Product goal:

Selene should not forget people casually.
Selene should become better at recognizing people over time.
Selene should recognize a person across devices.

Governed system rule:

Selene keeps a durable governed person record unless deleted, revoked, expired by policy, or legally required to remove it.

Do not define the system as uncontrolled permanent biometric storage.

Correct operating model:

For onboarded users:
persistent PersonProfile + continuously improving VoiceProfile.

For random non-onboarded speakers:
quarantined UnknownSpeakerCandidate / SpeakerProfileCandidate with no authority, no personal access, and no business access.

After onboarding or verification:
candidate can be promoted and linked to a full PersonProfile.

First-Time Speaker Introduction

When Selene hears a speaker for the first time, she should introduce herself and ask for the person’s name.

Example:

“Hi, I’m Selene. I don’t recognize your voice yet. What should I call you?”

A spoken name is a label, not verified identity.

Correct rule:

spoken name = claimed label
Voice ID + policy + verification = identity confidence
Authority = separate gate

If the claim cannot be verified, Selene keeps the speaker in candidate or pending posture.

Timing Rule: When Selene Should Ask Who Is Speaking

Selene should not conclude identity from the wake word alone.

Recommended timing:

Wake word only:
Do not conclude identity yet.

After first clean command:
Try Voice ID again.

After 6–8 seconds of clean speech:
If still unknown, ask for name.

Hard cap:
2 turns or 15 seconds before asking.

Example public-safe flow:

User: “Selene.”
Selene: “Hi, I’m listening.”

User: “Can you help me with something?”
Selene has enough speech now.
If still unknown:
“I don’t recognize your voice yet. What should I call you?”

For personal, private, or protected requests, Selene should ask immediately.

Example:

User: “Selene, read JD’s email.”
Selene: “I need to know who I’m speaking with before I can help with that.”

Onboarding and Guest Mode Boundary

There are two different cases.

Full Selene User

Every person who will fully use Selene should come through onboarding.

Onboarding should handle:

permission to use Selene
microphone access
voice interaction
Voice ID enrollment
continuous Voice ID improvement
identity/profile creation
memory scope
privacy and retention policy
audit logging
revocation / opt-out rules

Random Guest Speaker

Selene may still allow a random non-onboarded person to speak with her in public-safe guest mode.

Guest mode is not full Selene operation.

Guest mode can create only a quarantined candidate trail if policy allows.

Guest mode cannot grant:

personal access
business access
protected execution
memory access
authority
simulation execution

If the guest later completes onboarding, Selene should try to link the existing candidate voice evidence to the new onboarded PersonProfile.

Progressive Voice ID Learning

Continuous Voice ID learning should be part of Selene’s operating system.

For every spoken interaction, Selene should attempt to improve recognition if policy allows.

Core loop:

capture clean speech sample
score sample quality
reject bad samples
compare against existing VoiceProfiles
update confidence if match is strong
add approved voice evidence
improve model over time

Selene should gradually become more certain by collecting varied samples across:

different days
different devices
close microphone distance
far microphone distance
quiet rooms
noisy rooms
normal speech
fast speech
tired speech
short commands
longer conversation
English
Chinese
mixed language
accented speech

The goal is:

near-certain recognition under tested conditions

Not:

absolute 100% identity certainty

No identity system should claim literal 100% certainty.

Voice Sample Rejection Rules

Bad samples must be rejected and must not improve a VoiceProfile.

Reject samples such as:

two people speaking over each other
TV or speaker playback
Selene’s own TTS echo
background speech
low-quality microphone capture
suspected spoofing
unclear voice
voice mixed with another speaker
samples outside allowed policy

Voice ID Confidence Maturity

Voice ID should mature over time.

Example maturity levels:

UNKNOWN_SPEAKER_CANDIDATE:
Heard but not identified.

SPEAKER_PROFILE_CANDIDATE:
Name claimed, limited voice evidence, not verified.

NEW_PROFILE:
Name known, very limited voice data.

LEARNING_PROFILE:
Several accepted samples, improving recognition.

STABLE_PROFILE:
Strong recognition across normal conditions.

HIGH_CONFIDENCE_PROFILE:
Strong recognition across varied conditions and devices.

REVIEW_REQUIRED:
Conflicting samples, low confidence, suspected spoof, or profile drift.

Greeting Library Policy

Selene should use local library greetings first by default.

Reason:

Wake must be fast.
Wake must be reliable.
Wake must not wait for internet.
Wake must not depend on OpenAI, Brave, or any provider.
Wake must not create provider cost just to acknowledge activation.

Default runtime behavior:

Activation accepted
→ local approved greeting chosen
→ TTS speaks immediately
→ listening continues

Example local greetings:

“Hi JD, I’m here.”
“Ready when you are.”
“I’m listening.”

LLM Use for Greetings

LLM should not be in the critical wake path.

LLM may be used only after the session is already alive, or in a background/library-improvement process, when provider policy allows.

Allowed LLM uses:

creating new approved greeting phrases
contextual resume wording
natural variation after interruption
recovery after internet/provider outage
conversation continuation summaries

Example after internet returns:

“We’re back online. You were asking about the supplier report.”

Fallback if LLM/provider is unavailable:

“We’re back. I’m ready.”

Even for contextual resume, Selene should not depend on LLM. Local fallback must always exist.

Greeting Library Size and Growth Rules

Selene must not dump unlimited greetings into the library.

Recommended caps:

Core local library:
50–100 approved greetings

Per major language:
50–100 approved greetings

Per context category:
10–25 approved greetings

Greeting categories:

wake greeting
known user greeting
unknown user greeting
low-confidence identity greeting
resume after interruption
resume after internet outage
return from sleep
handoff to listening
protected-action boundary response

LLM generation rule:

Only generate new phrases when:
- repetition is detected
- library category is below target size
- provider use is allowed
- phrase passes validation
- phrase is not too similar to existing phrases

Once a category is full:

stop adding
rotate existing phrases
occasionally replace weak phrases after review

Quality beats quantity.

Greeting Validation Rules

Every accepted greeting must be:

short
warm
natural
varied
safe
non-committal
not a task answer
not a fake confirmation
not protected-action wording
not debug text
not source/citation text
not provider text

Authority Rule

Voice ID may help Selene know who is speaking.

Voice ID must not automatically grant protected authority.

Access tiers:

Public chat:
identity not required

Public-safe guest chat:
identity not required, but no private/personal/business/protected access

Light personal read-only:
high-confidence Voice ID may be enough if policy allows

Sensitive read-only:
Voice ID + policy + audit, possibly stronger confirmation

Payments / payroll / contracts / customer mutation:
Voice ID is not enough
requires stronger authentication plus simulation, authority, and audit

Protected execution:
never allowed from wake or greeting alone

Protected execution must remain governed by:

identity posture
→ access policy
→ authority gate
→ simulation match
→ execution gate
→ audit

Target Runtime Flow

1. User activates Selene.
   - On Mac / Windows / Android, the user may say “Selene”.
   - On iPhone, the user may press the side button / push-to-talk trigger.
2. Activation is accepted.
3. Session opens or resumes.
4. Activation/session handoff receives available activation audio, device context, and session context.
5. Voice ID tries local recognition from available activation / wake audio.
6. If local recognition is high confidence, Selene may use that speaker posture immediately.
7. If local recognition is weak, missing, or unavailable, cloud Voice ID fallback may check the cloud-authoritative PersonProfile / VoiceProfile records.
8. If cloud recognition succeeds, Selene updates the local recognition cache and may use the matched speaker posture.
9. If local and cloud both fail, Selene treats the speaker as new / unknown.
10. Device context is used as a confidence signal, not identity proof.
11. If recognized confidently, Selene may greet by name.
12. If not recognized, Selene uses a generic or first-time-speaker greeting.
13. TTS speaks clean greeting text.
14. Listening window stays open.
15. User continues speaking.
16. Voice ID keeps trying to identify from continuing speech.
17. If identity becomes confident, session speaker posture updates.
18. Selene may use the user’s name naturally.
19. If identity remains unknown after enough clean speech, Selene asks what to call the person.
20. Candidate profile is created if policy allows.
21. Public-safe guest access remains available for non-onboarded speakers.
22. Personal, business, and protected access remain blocked until identity, onboarding, policy, authority, simulation, and audit requirements pass.
23. Clean speech samples continue improving local cache and cloud-authoritative VoiceProfile or VoiceProfileCandidate when policy allows.
24. Approved voice-learning updates are batched after the session and uploaded to cloud within the governed sync window.
25. Cloud pushes critical security, revocation, and policy changes immediately.

Final Operating Law

Activation starts the session.
On iPhone, side button / push-to-talk is the activation equivalent of wake.
Wake detection is local-first.
Voice ID uses local cache first and cloud fallback second.
A speaker is new only when both local and cloud recognition fail.
Device helps locate and contextualize the session.
Device storage is a speed cache, not the authority.
Cloud PersonProfile / VoiceProfile records are durable authority.
Device ↔ cloud sync is event-driven, not every minute by default.
Cloud fallback runs immediately when local Voice ID fails.
Approved voice-learning updates upload after session.
Critical security and revocation updates push immediately.
Voice ID helps identify the speaker.
PersonProfile remembers the person.
VoiceProfile improves over time.
Onboarding grants full-user operating consent.
Guest mode allows public-safe conversation only.
Authority remains separate.
Unknown speakers get public-safe access only.
Selene never treats voice alone as protected authority.
Wake never becomes the brain.
Side-button activation never becomes the brain.

## Controlled Implementation Slices

The verbatim target above is part of the Selene master system, but implementation must be sliced so each change can be proven without weakening existing Stage 1-34 guarantees.

## Implementation Normalization Notes

These notes are the implementation-facing normalization of the verbatim design above. They remove repeated planning language, state the intended change against current repo behavior, and define what Codex should treat as controlling during build work.

### Change Against Current Wake Behavior

This plan is a deliberate master-system behavior change after activation acceptance.

Current certified wake behavior is:

```text
activation accepted
-> session opens/resumes
-> wake job finishes silently
```

New target behavior is:

```text
activation accepted
-> session opens/resumes
-> activation-to-session handoff runs
-> greeting policy chooses a local safe acknowledgement
-> TTS speaks the acknowledgement when audible policy allows
-> listening remains open
```

This supersedes silent post-wake handoff behavior only after session open. Wake detection itself remains activation-only. Wake still must not reason, answer, search, call providers, route tools, authorize, access memory, or execute protected work.

### Normalized Recognition And Sync Law

Use this single law when implementing identity behavior:

```text
local recognition first
-> cloud fallback only when local recognition is weak, missing, or unavailable
-> new / unknown speaker only when local and cloud both fail
```

Cloud unavailable does not mean the speaker is new. It means the speaker is not recognized on this device right now.

Device trust is not speaker identity. Voice ID is speaker posture, not protected authority. PersonProfile and VoiceProfile are durable governed identity structures. Local device storage is a speed cache. Cloud records are durable authority once that architecture is built.

### Slice Ordering Rule

Slice 1 must not attempt cloud Voice ID fallback, durable PersonProfile authority, cross-device recognition packs, lost-device revocation, or full sync. Slice 1 is only the immediate activation greeting handoff.

Cloud fallback, sync, PersonProfile authority, local encrypted recognition cache, guest candidate promotion, and lost-device behavior belong to later slices after the profile model and governing policy are explicitly wired.

### Audible Output Policy

TTS should speak the greeting immediately only when audible output is allowed by device, privacy, Do Not Disturb, session, and user policy.

If audible output is not allowed, Selene should still produce clean approved greeting text for the governed output channel, but must not play audio.

### First-Time Name Memory

At this stage, first-time guest introduction can remain simple.

If a public-safe guest tells Selene their name, Selene may remember that claimed name as a lightweight session/candidate label so the conversation feels natural. That spoken name is not verified identity, not onboarding completion, not personal access, and not authority. Onboarding, consent, retention, and durable identity governance can be handled in the later PersonProfile slices.

### Name Use Throttle

Known-speaker greetings may use `{name}`, but runtime should not use the person's name in every acknowledgement or every turn. The implementation should rotate between named and non-named phrases so Selene sounds natural rather than repetitive.

### Slice 1 - Activation Greeting Handoff

Build only the immediate handoff behavior:

- activation opens/resumes a session;
- activation handoff chooses a safe local greeting;
- TTS speaks clean greeting text;
- listening remains open;
- no provider call, search, tool routing, protected execution, connector write, or business mutation occurs;
- Desktop wake and iPhone explicit activation enter equivalent downstream handoff semantics where platform support exists.

### Slice 2 - Early Voice ID Posture

Wire activation audio / wake audio / pre-roll into Voice ID where repo surfaces safely support it:

- known high-confidence speaker may be greeted by name;
- unknown or insufficient speaker evidence receives a generic greeting;
- low-confidence and mismatched speaker posture never becomes authority;
- Voice ID remains a posture signal only.

### Slice 3 - Continuing-Speech Identification

Use continuing clean speech after the greeting to improve the current session speaker posture:

- retry Voice ID after the first clean command;
- ask who is speaking only after enough clean speech, a personal/protected request, or the configured hard cap;
- update the session speaker posture only after evidence and policy support it.

### Slice 4 - Greeting Library Governance

Add the governed greeting library:

- local-first rotating greetings;
- anti-repetition rules;
- category caps;
- phrase validation;
- optional LLM phrase harvesting only outside the critical wake path and only when provider policy allows;
- provider-off operation uses the local library only.

### Slice 5 - Unknown Speaker and Guest Lane

Implement public-safe unknown speaker behavior:

- UnknownSpeakerCandidate posture;
- SpeakerProfileCandidate after a claimed name;
- public-safe guest conversation only;
- no personal, business, memory, protected, connector, or authority access;
- spoken identity claims never self-verify.

### Slice 6 - PersonProfile Linkage

Introduce or reconcile the governed person profile structure:

- PersonProfile links name, aliases, VoiceProfile refs, onboarding consent, memory scope, preferences, access policy, device associations, and audit refs;
- voice data remains owned by Voice ID;
- memories remain owned by memory governance;
- authority remains owned by access and execution governance.

### Slice 7 - Local Voice Cache and Cloud Fallback

Implement local-first recognition and cloud fallback only after the profile model is governed:

- local encrypted VoiceProfile cache for speed;
- cloud-authoritative PersonProfile / VoiceProfile master records;
- new / unknown only after local and cloud both fail;
- cloud unavailable is classified as not recognized on this device right now, not as a new person;
- no raw audio persistence by default.

### Slice 8 - Device Sync, Revocation, and Lost Device

Implement versioned device/cloud sync:

- event-driven sync;
- immediate push for critical revocation and policy updates;
- after-session upload of approved voice-learning updates;
- lightweight active-session checks;
- no constant every-minute sync by default;
- lost devices revoke trust and invalidate local recognition cache.

### Slice 9 - Cross-Platform Activation Parity

Reconcile platform-specific activation:

- Mac / Windows / Android wake word activation where supported;
- iPhone side button / push-to-talk as activation equivalent;
- downstream session, Voice ID, greeting, TTS, listening, authority, and audit semantics remain consistent;
- native clients remain renderers and must not become the brain.

## Initial Approved Greeting Seed Library

The greeting library must be seeded before runtime use. Wake acknowledgement must never depend on an LLM or live provider call.

Initial seed ownership:

- Codex drafts the first safe seed set.
- JD reviews and approves tone before runtime use.
- Runtime may use only approved local greetings.
- LLM-generated greetings may later supplement the library only when provider policy allows and phrase validation passes.
- When a category reaches its cap, Selene rotates approved phrases instead of generating more by default.

Seed status for this section: `JD_REVIEW_REQUIRED_BEFORE_RUNTIME_USE`.

### Generic Wake Greetings

Use when activation is accepted and speaker identity is unknown, unavailable, or not needed.

1. "Hi, I'm here."
2. "I'm listening."
3. "Ready when you are."
4. "Hi, what can I do for you?"
5. "I'm here. What would you like to do?"
6. "Go ahead, I'm listening."
7. "I'm ready."
8. "Hi, I'm with you."
9. "What can I help with?"
10. "I'm here and ready."
11. "Hi, tell me what you need."
12. "Ready. What's next?"
13. "I'm awake. How can I help?"
14. "Hi, I'm ready to listen."
15. "I'm here. Go ahead."
16. "What would you like?"
17. "Hi, I'm ready when you are."
18. "I'm listening now."
19. "Ready for you."
20. "Hi, what are we doing?"

### Known Speaker Greetings

Use only when Voice ID posture confidently supports the speaker name.

1. "Hi {name}, I'm here."
2. "Ready when you are, {name}."
3. "I'm listening, {name}."
4. "Hi {name}, what can I do for you?"
5. "I'm here with you, {name}."
6. "Go ahead, {name}."
7. "I'm ready, {name}."
8. "What do you need, {name}?"
9. "Hi {name}, I'm ready to help."
10. "I'm with you, {name}."
11. "Ready for you, {name}."
12. "Hi {name}, tell me what you need."
13. "I'm listening now, {name}."
14. "What would you like to do, {name}?"
15. "Hi {name}, I'm awake."
16. "I'm here, {name}. What's next?"
17. "Ready, {name}. What can I help with?"
18. "Hi {name}, go ahead."
19. "I'm ready when you are, {name}."
20. "What are we working on, {name}?"

### Unknown Speaker Greetings

Use when Voice ID does not identify the speaker but public-safe guest interaction is allowed.

1. "Hi, I'm Selene. I don't recognize your voice yet."
2. "Hi, I'm here. I may need your name in a moment."
3. "I'm listening. I don't know who I'm speaking with yet."
4. "Hi, I'm Selene. What should I call you?"
5. "I'm here. I may need to confirm who you are."
6. "Hi, I don't recognize your voice yet. What can I call you?"
7. "I'm listening. Tell me what I should call you."
8. "Hi, I'm ready. I may need a little more voice to recognize you."
9. "I'm here. I don't have a confident speaker match yet."
10. "Hi, I can help with public-safe things while I learn who is speaking."

### Low-Confidence Identity Greetings

Use when Voice ID has a possible but weak speaker match.

1. "Hi, I'm here. I may need to confirm who I'm speaking with."
2. "I'm listening. I don't have a strong identity match yet."
3. "Ready when you are. I may ask you to confirm your name."
4. "Hi, I think I may know you, but I need more confidence."
5. "I'm here. I may need a little more voice to recognize you safely."
6. "Go ahead. I'll keep identity-sensitive things locked until I'm sure."
7. "I'm listening. Personal actions may need confirmation."
8. "Hi, I don't have a high-confidence match yet."
9. "I'm ready. I may need to verify before anything personal."
10. "I'm here. Let's keep going while I confirm who is speaking."

### Handoff To Listening Greetings

Use when activation acknowledgement should clearly hand into command capture.

1. "I'm listening. Go ahead."
2. "Ready. Tell me what you need."
3. "I'm here. What would you like me to help with?"
4. "Go ahead. I'm listening now."
5. "Ready for your next words."
6. "I'm listening for your request."
7. "You have my attention."
8. "I'm ready for the next step."
9. "Tell me what you need."
10. "I'm ready to listen."

### Protected Boundary Responses

Use when a request touches personal, sensitive, or protected scope and identity or authority is not sufficient.

1. "I need to confirm who I'm speaking with before I can help with that."
2. "I can't access personal information until identity and authority are confirmed."
3. "That needs stronger confirmation before anything can happen."
4. "I can discuss it, but I can't execute that without the required authority."
5. "Protected actions need identity, policy, simulation, and audit first."
6. "I can't make that change from voice alone."
7. "I need the proper approval path before doing anything protected."
8. "I can help prepare it, but I can't execute it casually."
9. "That request is locked until the right authority is proven."
10. "I won't run protected work from wake or greeting alone."

## Build Law

Every implementation slice must prove:

- no provider call is required for wake acknowledgement;
- no protected execution can start from wake, side-button activation, greeting, or Voice ID alone;
- voice identity is not authority;
- device trust is not identity proof;
- Desktop, iPhone, Android, and Windows remain clients/renderers;
- TTS speaks only clean approved greeting or response text;
- no raw audio is committed;
- no secrets are introduced;
- no post-34 numbered stage is created or authorized;
- no new build is authorized by this document alone.
