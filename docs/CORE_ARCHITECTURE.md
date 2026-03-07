Selene Client Runtime Architecture — Universal Device Session Model

Core rule

Selene must operate through client applications running on supported device classes.

These device classes include:

iPhone

Android phone

Tablet

Desktop

Each device runs a Selene client application.

These clients are the only user-facing terminals for speaking to Selene, hearing Selene, and interacting with Selene.

The client is not the authority layer.
The cloud remains the authority layer.

Universal client principle

All Selene clients must follow the same core model.

No matter whether the device is:

iPhone

Android

Tablet

Desktop

The purpose of the client is the same:

capture user input

open or resume a cloud session

send turns to Selene cloud

receive responses from Selene cloud

render output

play audio output

maintain lightweight local assist state

sync approved updates from cloud

queue and retry operations safely

The cloud session is the real session.
The device is only the terminal into that session.

Universal session rule

All meaningful Selene work must come from an open or resumed session.

That includes:

simulation-based processing

web search

link generation

link delivery

onboarding actions

message handling

future tools and workflows

Builder and repair workflows

The session is not owned by the phone, tablet, or desktop.
The session is cloud-based.

The device may:

open a session

resume a session

reflect session state

cache session hints for speed

But the device must never invent session truth.

All future functions, simulations, and tools must be born from session context.

Session is the central runtime container through which Selene brings together its engines, memory, access checks, tools, and execution paths.

Trigger policy by platform

All clients follow the same session-first model, but trigger entry differs by platform.

iPhone:

EXPLICIT only

side button or approved explicit action opens or resumes session

no wake word

Android:

wake word allowed

explicit entry allowed

wake or explicit opens or resumes session

Tablet:

wake word allowed

explicit entry allowed

wake or explicit opens or resumes session

Desktop:

wake word allowed

explicit entry allowed

wake or explicit opens or resumes session

This trigger difference must not change the cloud session model.
It only changes how the client enters the session.

Client responsibilities shared across all platforms

Every Selene client should own the same core responsibilities:

audio capture

audio preprocessing

voice entry handling

session entry or resume request

chat or thread rendering

audio playback

local outbox and retry management

lightweight local cache

safe synchronization with cloud

receipt and device-state submission

safe apply of approved cloud updates

The exact trigger UX may vary by platform, but the core client responsibilities stay the same.

Client interaction capabilities

Selene clients are the primary interaction layer between the user and Selene.

Beyond voice interaction, the client applications must support rich interaction capabilities equal to or stronger than modern intelligent applications.

These capabilities include:

visual result rendering such as tables, charts, and structured data visualizations

ability to upload PDF files

ability to upload images

ability to upload documents and media for cloud processing

ability to generate and display printable documents such as PDF files

ability to send documents to local printers connected to the device

ability to open and control device hardware features such as camera and microphone

ability to capture images for tasks such as scanning invoices, receipts, or documents

ability to upload captured media to Selene cloud for processing

ability to display structured outputs including reports, dashboards, and formatted documents

ability to support interactive workflows such as approving actions, confirming tasks, or selecting options

These capabilities must remain platform-native but functionally equivalent across device classes.

For example:

an invoice may be captured using the camera

Selene processes the image in the cloud

the result may be rendered as a table or chart

a PDF summary may be generated

the user may print the document through the device printer

All6. Cloud responsibilities shared across all platforms

The cloud is authoritative for:

identity truth

access control

NLP understanding

LLM reasoning

simulation matching

simulation execution

session truth

onboarding truth

link generation

link delivery

learning evaluation

profile promotion

artifact authority

audit logging

Selene Builder services must also operate from this cloud-authoritative layer.

Builder responsibilities include:

diagnosing what does not work

producing fix paths for broken system behavior

supporting repair and upgrade workflows under the same session-first architecture

No device should override cloud truth.

Client-side local assist scopen

profile promotion

artifact authority

audit logging

No device should override cloud truth.

Client-side local assist scope

The client may keep local data only for speed, continuity, and UX quality.

This includes:

voice-recognition assist data

wake personalization where supported

recent thread cache

session resume hints

audio pipeline state

pending outbox items

secure auth tokens

push token state

device capability state

approved local profile versions

This local state is never authoritative.

Same capability rule across device classes

Selene should provide the same functional capability shape across client classes.

That means:

a user can speak to Selene from iPhone, Android, Tablet, or Desktop

a user can open or resume a cloud session from any supported client

a user can continue workflows from that session regardless of client class

a user can request cloud actions from that session regardless of entry device

The main difference between platforms is trigger method, not system capability.

Canonical runtime flow

The universal runtime flow is:

trigger entry on client

client captures audio or text

client submits turn to cloud

cloud validates ingress security and request integrity

cloud resolves or opens session context

cloud handles voice identity and onboarding gate requirements

cloud applies memory, access, understanding, and execution rules

cloud returns result

client renders and optionally speaks result

client records sync and operation outcome state

This flow must be treated as the canonical Selene client runtime model.

Multi-platform design consequence

The iPhone app, Android app, Tablet app, and Desktop app should not be designed as separate product concepts.

They are platform-specific expressions of the same client architecture.

Therefore:

there should be one parent client-runtime architecture

platform documents should inherit from that parent model

iPhone-specific differences should be limited mainly to trigger and platform constraints

Android/Desktop/Tablet differences should be limited mainly to wake support and platform-specific hardware behavior

What must be aligned next

The system should be updated so that:

Tablet becomes a formal platform in contracts and design

platform-aware trigger policy is enforced centrally

iPhone remains explicit-only

Android, Tablet, and Desktop support wake plus explicit

all client architecture documents follow the same parent session model

all future client builds preserve cloud-authoritative session truth

Client ingress contract (cross-platform)

All Selene clients must interact with the cloud through the same ingress contract family.

These ingress routes are canonical across device classes:

/v1/invite/click

/v1/onboarding/continue

/v1/voice/turn

All client implementations must respect the same request semantics, security headers, and idempotency rules defined by the server.

Clients must never bypass or simulate these flows locally.

The ingress contract must remain cross-platform consistent so that iPhone, Android, Tablet, and Desktop clients behave identically at the protocol layer.

Universal session payload contract

All clients must consume the same canonical session payload model.

The server must provide consistent identifiers and state semantics so that every client understands session lifecycle in the same way.

The session payload should include canonical identifiers such as:

session_id

turn_id

session_state

These identifiers must behave consistently across all device classes.

Clients may cache hints for UI continuity but must treat server responses as the source of truth.

Device trigger policy vs session model

Trigger entry is platform-specific but the session lifecycle is universal.

Platform trigger rules:

iPhone – explicit entry only

Android – wake word or explicit

Tablet – wake word or explicit

Desktop – wake word or explicit

These trigger differences must never fork the cloud session model.

Regardless of how a session is opened, the session lifecycle and processing path must remain identical.

Cross-platform parity rule

All Selene clients must implement the same functional capability set.

Clients must support:

voice entry

session open and resume

turn submission

response rendering

cloud synchronization

operation retry and deduplication

inter-device switching across the same user journey

A user must be able to begin interaction on one supported device and continue smoothly on anot16. Sync, outbox, and deduplication rule

All Selene clients must implement a deterministic synchronization model.

Each client must include:

a durable outbox

operation journaling

idempotent resend behavior

cloud acknowledgement before completion

conflict-safe retry logic

inter-device synchronization that preserves one consistent user history and session view

If device state and cloud state diverge, the cloud state always wins.

When a user switches between devices, synchronization must preserve correctness with:

no missing information

no duplicated operations

no conflicting session history

no broken continuity between devices

Learning and update loopand deduplication rule

All Selene clients must implement a deterministic synchronization model.

Each client must include:

a durable outbox

operation journaling

idempotent resend behavior

cloud acknowledgement before completion

conflict-safe retry logic

If device state and cloud state diverge, the cloud state always wins.

Learning and update loop

All clients may participate in the Selene learning and update loop.

Clients may:

upload approved learning artifacts

receive approved profile or configuration updates

stage updates locally

apply updates safely

rollback updates if required

The cloud remains responsible for evaluating learning signals and promoting official updates.

Cloud authority boundary

The cloud is the authoritative source for all critical system state.

No client may override cloud truth for:

identity

access control

session state

onboarding state

simulation execution

learning promotion

artifact authority

Clients may cache assist data only for speed and UX continuity.

Platform inventory

The Selene client ecosystem includes the following platform classes:

iPhone

Android phone

Tablet

Desktop

Tablet devices follow the Android-style wake policy unless otherwise defined by platform constraints.

All platform classes must adhere to the same session model and client-runtime architecture.

Scope boundaries for this architecture

This parent architecture defines the universal client runtime model but does not define:

exact UI layouts

screen-level UX behavior

public administrative APIs

invite generation administration endpoints

local model packaging formats

These decisions belong in platform-specific documents or later design stages.

Immediate next step

Use this document as the governing parent architecture for all Selene clients.

Platform-specific documents (iPhone, Android, Tablet, Desktop) must inherit from this model and only specify platform differences where required by OS or hardware constraints.

Voice enrollment authority contract

Voice enrollment is a mandatory onboarding stage for all Selene users.

Before onboarding can complete:

voice enrollment must reach a locked state

voice enrollment artifacts must be synchronized with the cloud

voice recognition profiles must be created and stored under the user's identity scope

Clients participate only as capture terminals during enrollment.

The authoritative enrollment process is executed and validated by the cloud.

Once locked, the enrolled voice profile becomes part of the identity recognition system used during future sessions.

Platform divergence matrix

Some onboarding steps differ by platform.

The canonical onboarding order is:

invite or open

onboarding session start

required platform receipts

missing fields, terms, primary-device confirmation, and sender verification if required

voice enrollment

wake enrollment where platform requires it

personality lock

access-related onboarding steps

complete

ready

Voice enrollment is required for all platforms.

Wake enrollment behavior differs:

iPhone – wake enrollment disabled

Android – wake enrollment required

Tablet – wake enrollment required unless a later platform contract says otherwise

Desktop – wake enrollment required

Platform receipts also differ.

iPhone requires ios_side_button_configured

Android/Desktop require wake configuration receipts

Despite these differences, the overall onboarding progression and cloud authority remain the same.

Personality lock semantics

During onboarding Selene performs a personality classification step.

The classification categories include:

Passive

Domineering

Undetermined

This classification is stored as an authoritative onboarding record in the cloud.

Personality influences response tone only in the current implemented system.

Personality must never influence:

access control

simulation execution

security checks

system authority

Current implemented behavior is tone and style guidance only.

A stronger permanent opposite-response contract is not yet the implemented system rule.

The Personality engine therefore requires additional future25. Session lifecycle contract

Selene sessions are cloud-owned objects with defined lifecycle states.

The canonical session states include:

Closed

Open

Active

SoftClosed

Suspended

Session transitions are controlled by the server.

Typical lifecycle:

trigger entry opens Closed → Active

continued interaction keeps session Active

inactivity moves Active → SoftClosed

extended inactivity moves SoftClosed → Closed

system conditions may move session → Suspended

A session should remain open while the speaker is still engaged with the topic.

The inactivity period must be configurable, for example 30 seconds or another configured duration, but the governing principle does not change.

The principle is:

once a session opens, it remains open until the user stops speaking for the configured inactivity period

Before a session fully closes, Selene should confirm whether the user has finished with the topic where the interaction model requires it.

Clients must treat the server lifecycle as the source of truth.

Current repo-grounded reopen behavior is effectively scoped by actor plus device.

True cross-device shared session continuity is still a future alignment target rather than the current fully implemented public contract.

Client session identifier exposureice.

True cross-device shared session continuity is still a future alignment target rather than the current fully implemented public contract.

Client session identifier exposure

The client runtime model assumes explicit session identifiers are available.

Canonical identifiers should include:

session_id

turn_id

session_state

These identifiers allow consistent session continuity across clients.

Clients must not infer or invent session identifiers locally.

Memory authority and identity scope

Selene memory is cloud-authoritative and identity-scoped.

Memory records are stored by user identity rather than session.

Sessions may reference memory but do not own it.

Authoritative memory read and write requires confirmed voice identity.

Without confirmed identity, the system must not enter authoritative user-memory scope.

Memory access is filtered by:

identity scope

memory sensitivity policy

confidence level

permission requirements

Sensitive memory requires additional permission checks before use.

Memory persistence model

Selene memory uses a ledger + materialized current-state model.

Memory events are append-only in the ledger.

The current view reflects the latest active memory state.

Memory events may include:

Stored

Updated

Forgotten

Selene memory should be understood in temperature bands.

Hot memory:

retained for 72 hours

Medium memory:

retained for 30 days

Cold memory:

retained indefinitely unless policy or deletion rules state otherwise

Memory provenance may include session identifiers but storage is keyed by user identity.

When a session closes, the session lifecycle may end while cloud memory persists according to memory policy.

Any remaining device-side cache remains assist-only and non-authoritative.

Every active session should be able to access allowed memory under the memory engine rules.

Local assist cache vs cloud truth

Clients may keep lightweight local caches for performance.

Examples include:

recent thread history

session resume hints

audio pipeline state

voice recognition assist data

approved profile versions

operation outbox state

Local caches are never authoritative.

If cloud and device state diverge, cloud state always overrides the device.

Learning and update pipeline

Selene devices participate in a bidirectional learning and update loop.

Upload direction:

clients may upload learning artifacts or telemetry

cloud evaluates validity

cloud decides promotion or rejection

Download direction:

cloud distributes approved updates

clients verify and stage updates

clients apply updates safely

clients confirm or rollback updates

Device-side update application must follow a strict sequence:

download

verify

stage

apply

confirm

rollback if required

The cloud remains responsible for promotion and governance of all learned behavior.

Offline, reconnect, and deduplication model

All clients must implement deterministic synchronization behavior.

Each client must include:

an operation outbox

an operation journal

retry state management

idempotent resend behavior

cloud acknowledgement before completion

Reconnect sequence:

refresh authentication

refresh session truth

flush pending operations

upload queued artifacts

pull approved updates

apply updates safely

refresh UI state

Duplicate execution must be prevented using stable idempotency identities.

Link generation and delivery model

Link generation and delivery are cloud-owned operations.

Typical flow:

user requests link from an active session

cloud verifies identity and permissions

cloud executes link-generation simulation

cloud optionally executes link-delivery workflow

cloud records audit and proof state

client renders result

Clients must never generate authoritative links locally.

This architecture does not define a public client-facing invite generate or send API.

Current repo-grounded behavior uses internal cloud simulation and delivery paths for authoritative generation and sending.

Device vs cloud responsibility boundary

Devices may perform:

audio capture

audio preprocessing

local caching

retry and sync operations

playback and rendering

assist functions

camera capture

document upload

printer access through local device interfaces

Cloud must perform:

identity verification

access control

session lifecycle authority

onboarding authority

simulation execution

memory authority

learning governance

artifact authority

audit logging

black-box compliance and proof capture for authoritative system events

Cross-platform capability parity sync operations

playback and rendering

assist functions

Cloud must perform:

identity verification

access control

session lifecycle authority

onboarding authority

simulation execution

memory authority

learning governance

artifact authority

audit logging

Cross-platform capability parity

Across iPhone, Android, Tablet, and Desktop the following must remain identical:

cloud authority

session-first runtime model

ingress contract family

onboarding flow

voice enrollment authority

memory authority

sync and retry model

response rendering flow

Allowed platform differences are limited to trigger mechanisms, hardware behavior, and operating system constraints.

Retention, purge, and delete lifecycle

Retention, purge, and delete behavior must be defined across:

wake-related artifacts

memory records

device-local assist cache

operation journals and outbox state

session-bound compliance and proof records

Where retention and deletion execution is not yet fully implemented, the gap must remain explicit and must not be silently assumed complete.

Known architectural gaps

Current system gaps include:

explicit session identifiers not yet exposed in some client responses

true cross-device shared session continuity not fully implemented

Android wake runtime parity still incomplete

tablet platform modeling still emerging

personality behavior currently limited to tone influence

retention and purge lifecycle policies still evolving

native app implementation remains outside this repo in several areas

These gaps should be resolved progressively while preserving the core architecture defined in this document.

Governing architecture rule

This document defines the universal runtime architecture for all Selene clients.

All platform implementations must comply with these rules.

Platform-specific documents may extend but must never contradict this parent architecture.

Engine ownership of current gaps

The following engines or subsystems currently require additional work or alignment. Listing engine ownership clarifies where future engineering effort must focus.

PH1.W — Wake Engine

Android wake runtime parity is still incomplete.

Wake artifact retention, purge, and delete lifecycle is not fully implemented.

Reject-reason coverage and wake runtime parity across platforms require further completion.

PH1.K — Voice Runtime I/O

Android microphone runtime parity with the desktop path is still incomplete.

Capture bundle trust and attestation boundaries still require additional hardening.

PH1.L — Session Lifecycle Engine

Session lifecycle logic is implemented but current reopen scope is effectively actor plus device.

True cross-device shared session continuity remains a future architectural target.

Public session identifier exposure is still incomplete in some API responses.

Session close confirmation behavior and inter-device smooth continuity require additional design and implementation work.

PH1.VOICE.ID — Voice Identity Engine

Voice enrollment and identity assertion are implemented in onboarding.

However, the native device-side capture contract and enrollment UX still require full implementation in client applications.

Device-side enrollment flow must be aligned with the cloud enrollment authority contract.

PH1.EMO / PH1.EMO.CORE — Personality Engine

Personality classification and persona lock exist.

Current implementation affects tone and response style only.

The stronger “react opposite permanently” behavior is not currently implemented.

This engine requires additional design and build work before a deeper long-term personality model can be considered complete.

PH1.M — Memory Engine

Core memory authority and identity-scoped storage are implemented.

However, full retention, purge, and delete lifecycle policies are not yet complete.

Hot, medium, and cold memory behavior must be finalized and enforced consistently.

Memory governance rules must be finalized alongside wake and artifact lifecycle policies.

PH1.OS — Platform Orchestration Layer

Platform-aware trigger policy needs stronger centralized enforcement.

Tablet platform modeling is still emerging and must be fully integrated into the platform model.

Cross-device switching and universal session-first enforcement across all future functions must be kept consistent at the orchestration layer.

PH1.F — Persistence Foundation

The persistence layer is largely complete but carries unfinished lifecycle responsibilities for session artifacts, memory retention, wake artifacts, and compliance records.

Additional lifecycle enforcement and purge workers may be required.

PH1.J — Audit and Proof Layer

Black-box compliance and future proof capture must be clearly bound into the authoritative runtime flow.

Session-bound audit, proof, and compliance capture require stronger explicit architectural treatment.

Simulation and Link Execution Path

Link generation and delivery are implemented through simulation dispatch and broadcast execution.

Public client-facing generation APIs are not currently defined in this architecture and remain a product-layer decision.

BCAST engine remains the authoritative delivery path for link sending under the current design.

Build-phase transition rule

The architecture must be corrected first before further feature expansion.

The first implementation run should therefore focus on aligning the current system to this target architecture.

After the architecture correction run, future work should proceed in structured build runs.

Those runs should include:

architecture correction where current wiring is wrong or incomplete

engine-specific improvement runs where more functionality is required

platform implementation runs for native clients

session and synchronization hardening runs

memory, audit, and compliance completion runs

Codex should later help propose how many implementation runs are required, but all such runs must remain aligned with this parent architecture and must preserve the session-first system law.
