Selene Client Runtime Architecture — Universal Device Session Model

Core Architecture Rule

Selene operates through client applications running on supported device classes. These client applications are the only user-facing terminals through which users interact with Selene — speaking to Selene, hearing responses, viewing results, and participating in system workflows.

Supported device classes include:

iPhoneAndroid phoneTablet (target platform class)Desktop

Each device runs a Selene client application that connects to the Selene cloud runtime.

The foundational architectural law is:

The client is never the authority layer.

All authoritative system decisions and persistent system state exist exclusively in the Selene cloud runtime.

The Selene cloud runtime is therefore responsible for the following authoritative domains:

identity verificationaccess control and authorizationsession lifecycle ownershipsimulation selection and executionmemory storage and governancelearning evaluation and promotionartifact approval and activationaudit, proof, and compliance records

Client devices function strictly as interaction terminals and assist layers. Clients may capture user input, render responses, maintain temporary assist caches, and synchronize with the cloud runtime, but clients must never override, fabricate, or finalize authoritative system truth.

Every meaningful interaction with Selene must ultimately resolve to a cloud-controlled session context. All execution paths within the system must originate from this session context and execute within the boundaries of the cloud-authoritative runtime.

This rule establishes the fundamental separation between:

interaction surfaces (client devices)andauthority layers (Selene cloud runtime)







Universal client principle

All Selene clients must operate under a single universal runtime interaction model regardless of device class. The client application functions as a platform‑specific interaction layer that connects the user to the same cloud‑authoritative Selene system.

Supported device classes include:

iPhone

Android phone

Tablet (target platform class)

Desktop

Although hardware capabilities, operating system policies, and interaction methods may differ between platforms, the logical behavior of every Selene client must remain identical.

The universal client runtime model requires that every client is capable of:

capturing user input (voice, text, files, images, camera input)

opening a new cloud session or resuming an existing session

submitting turns to the Selene cloud runtime

receiving authoritative responses from the cloud

rendering structured outputs such as text, charts, tables, documents, and dashboards

playing audio responses when required

safely synchronizing with the cloud runtime

queuing operations locally when offline

retrying operations deterministically using idempotent request identities

applying approved updates delivered by the cloud runtime

The governing architectural rule is:

The cloud session is the real session.

The device is only the terminal into that session.

Client devices may improve the user experience through local caching, temporary assist models, and short‑lived device state. However, all such state remains subordinate to the authoritative cloud state.

No Selene client may introduce a different logical execution model, bypass the canonical session‑first architecture, or perform system authority decisions locally.

All clients must therefore behave as interchangeable terminals attached to the same cloud‑authoritative Selene runtime.





Universal session rule

Selene is a session‑first system. Every meaningful interaction with Selene must originate from, and execute within, an open or resumed cloud session.

A session is the authoritative runtime container that coordinates the behavior of all Selene system engines, including identity verification, access authorization, memory retrieval, simulation execution, learning signals, and audit capture.

The architectural law is:

All system execution must occur inside a session context.

No operation that affects system behavior, memory, simulation execution, artifact generation, or workflow progression may execute outside the boundaries of a valid session.

All meaningful Selene work must execute within session context, including:

simulation‑based processing

web search and information retrieval

link generation and link delivery workflows

onboarding progression

message handling and conversation turns

builder diagnosis and repair workflows

future tools, automations, and integrations

The session is never owned by the device. Sessions are created, managed, and governed exclusively by the Selene cloud runtime.

Client devices act only as terminals connected to the session.

Client devices may:

open a new session

resume an existing session

observe session state

cache temporary session hints for performance

Client devices must never:

create authoritative session state

mutate session lifecycle rules

invent session identifiers

execute logic outside the session container

introduce alternative execution paths

This rule guarantees that all system activity is coordinated through the same deterministic runtime environment.

System extension rule

Any new Selene capability introduced in the future must comply with the session‑first architecture. Every new function, simulation, or workflow must originate from session context and execute within the session lifecycle governed by the cloud runtime.

Session Decomposition Model

A Selene session coordinates several conceptual layers that together define how interaction, execution, and memory operate during a conversation lifecycle. Separating these layers clarifies the responsibilities of the session container and prevents confusion between temporary interaction state and long‑term system state.

A session orchestrates — but does not own — identity, artifacts, or persistent memory. Instead it provides the runtime container where these elements are accessed in a controlled and deterministic way.

Conversation Layer

The conversation layer governs the user interaction lifecycle. It determines when a conversation begins, when it is actively processing turns, when it becomes idle, and when it is closed or suspended.

This layer is responsible for:

conversation state

turn sequencing

interaction timing and inactivity behavior

cross‑device session visibility

The conversation layer defines how the user experiences the interaction with Selene.

Execution Layer

The execution layer represents the runtime environment in which simulations, tools, and workflows are executed during each turn.

This layer coordinates:

simulation dispatch

tool execution

workflow orchestration

policy validation

execution ordering

Only one execution path may mutate session state per turn. The cloud runtime serializes concurrent requests to maintain deterministic system behavior.

Memory Layer

The memory layer defines how identity‑scoped knowledge is accessed during a session.

Memory itself is not owned by the session. Instead, the session references eligible memory from the cloud memory subsystem when identity scope has been verified.

This layer governs:

memory candidate retrieval

policy eligibility filtering

context injection into execution

Because memory is identity‑scoped rather than session‑scoped, memory survives session termination and may be reused in future sessions once identity is confirmed.

Session orchestration role

The session container coordinates these layers but does not directly own:

identity records

artifact storage

long‑term memory persistence

Those responsibilities belong to dedicated cloud subsystems.

By separating conversation, execution, and memory layers, Selene ensures that interaction state, execution control, and knowledge persistence remain cleanly isolated while still cooperating within the session runtime container.

Cross‑Device Session Attachment

Selene allows multiple client devices belonging to the same user identity to attach to the same active cloud session. This capability enables seamless cross‑device interaction while preserving a single authoritative execution history.

However, even when multiple devices are connected, session authority remains centralized in the Selene cloud runtime. Devices may observe and interact with the session, but they must never independently control session state.

Cross‑device interaction principle

The governing rule is:

Multiple devices may observe a session, but only the cloud runtime may mutate session state.

This guarantees deterministic execution and prevents conflicting system behavior.

Cross‑device participation model

When multiple devices attach to the same session:

multiple devices may observe the same session state

multiple devices may submit turns

all turns must be serialized by the cloud runtime

execution ordering must remain deterministic

Clients must treat server responses as authoritative for session state.

Single‑writer execution rule

Even when multiple devices are active, the system must enforce a single authoritative execution path per turn.

The runtime must guarantee:

one execution writer per turn

deterministic serialization of concurrent requests

idempotent processing of retry submissions

stable turn ordering across devices

If two devices submit requests simultaneously, the server must serialize them using canonical execution ordering.

Session authority model

The Selene cloud runtime is the only authority allowed to:

create session identifiers

assign turn identifiers

change session lifecycle state

resolve execution ordering

close, suspend, or reopen sessions

Devices may only synchronize with these authoritative values.

Cross‑device synchronization behavior

When a second device attaches to an existing session, the device must:

retrieve the authoritative session state from the cloud runtime

synchronize using the canonical session_id

adopt the authoritative turn ordering

reconcile any local caches or operation journals

Local device hints must always defer to the cloud runtime.

Concurrent device safety

If multiple devices are connected to the same identity:

all execution must still pass through the canonical runtime pipeline

concurrent operations must be deduplicated using idempotency identities

conflicting local device state must be discarded

cloud state must always override device state

This guarantees that Selene behaves as a single deterministic system regardless of how many devices are attached.

Device switching example

A typical cross‑device interaction may occur as follows:

user begins interaction on an iPhone

session_id is created by the cloud runtime

user later opens the Selene client on a Desktop device

desktop client retrieves the same session_id

both devices observe the same session state

all new turns are serialized by the cloud runtime

This interaction model ensures continuity while preserving cloud authority.

Trigger policy by platform

Selene uses a platform‑aware trigger policy that determines how a device may enter or resume a session. Trigger policy affects only how a session begins, not the internal session lifecycle or execution model.

All trigger mechanisms must ultimately converge into the same session‑first runtime pipeline defined in this architecture.

Platform Trigger Rules

iPhone

EXPLICIT only.

Session entry is triggered through an explicit user action such as:

side‑button interaction

approved explicit UI interaction

system shortcut or OS‑approved trigger

Wake word is intentionally disabled on iPhone due to platform and UX constraints.

Android

Both wake word and explicit entry are supported.

Wake word detection may run locally on device and initiate session entry once validated.

Explicit triggers such as UI interaction must also open or resume a session.

Desktop

Both wake word and explicit entry are supported.

Desktop environments may run persistent wake listeners or allow explicit keyboard/UI entry.

Tablet

Target behavior mirrors Android‑style policy unless restricted by operating system rules.

Tablet devices may support both wake and explicit triggers when platform capabilities permit.

Platform Trigger Enforcement

Platform trigger policy must be enforced centrally within PH1.OS (Platform Orchestration Layer) rather than only at adapter boundaries.

PH1.OS is responsible for validating:

allowed trigger types for the platform

device capability compatibility

platform‑specific trigger restrictions

All trigger validation must occur before session execution proceeds.

Trigger Convergence Rule

Regardless of the trigger method used, the runtime must converge to the same canonical path:

trigger → capture → ingress validation → session resolve/open → execution pipeline

Trigger differences must never create divergent runtime logic paths.

Client responsibilities shared across all platforms

Every Selene client must implement a consistent set of runtime responsibilities that allow the device to function as a reliable interaction terminal for the Selene cloud system. These responsibilities define the minimum operational behavior that every platform implementation (iPhone, Android, Tablet, Desktop) must support.

The governing rule is:

Client devices provide interaction, capture, and reliability. The cloud runtime performs all authoritative decision‑making and system execution.

Client responsibilities therefore exist only to facilitate communication between the user and the cloud‑authoritative Selene runtime.

Input capture responsibilities

Clients must capture user input across all supported modalities, including:

voice capture through the device microphone

text input through user interface components

file upload through device file selection

image upload

camera capture for photographing or scanning documents

All captured input must be packaged into a valid turn request and submitted to the cloud session runtime through the canonical ingress contract.

Session interaction responsibilities

Clients must be able to:

open a new cloud session

resume an existing session

submit turns into the active session

receive authoritative responses from the cloud runtime

reflect session lifecycle state accurately

Client devices must never modify session lifecycle state locally. The cloud runtime remains the single authority for session state.

Rendering responsibilities

Clients must support rendering structured results returned by the cloud, including:

text responses

tables and structured datasets

charts and visualizations

PDF or document outputs

interactive UI responses

Clients may also perform text‑to‑speech playback when audio responses are returned.

Local reliability responsibilities

Clients must implement deterministic reliability behavior to protect interaction continuity. This includes:

local operation outbox for pending operations

operation journaling

safe retry logic for failed requests

idempotent resend behavior

cloud acknowledgement before marking operations complete

These mechanisms ensure that interactions remain reliable during poor connectivity, application restart, or device switching.

Synchronization responsibilities

Clients must synchronize correctly with the cloud runtime by:

refreshing session state when reconnecting

flushing pending operations

applying approved configuration or profile updates

ensuring no duplicate or missing operations occur

Synchronization must maintain a single consistent session history across devices.

Device integration responsibilities

Clients may integrate with local hardware interfaces to support user workflows including:

camera access

microphone capture

printer integration

file selection dialogs

media upload interfaces

These integrations exist only to facilitate user interaction and data capture. All processing, interpretation, and decision logic remains cloud‑authoritative.

Authority constraint

Clients must not perform or simulate any of the following locally:

identity verification

access authorization

simulation execution

session lifecycle mutation

artifact activation

memory authority decisions

These responsibilities always belong to the Selene cloud runtime.

Client interaction capabilities

Selene clients provide the full user interaction surface for the system. While the cloud performs all authoritative processing and decision‑making, the client is responsible for presenting results, capturing inputs, and enabling rich interaction workflows with the user.

Clients must support a broad set of interaction capabilities comparable to or exceeding modern intelligent assistant applications.

Core interaction modalities

Clients must support the following primary modalities:

voice interaction

text interaction

file upload

image upload

camera capture

structured visualization output

document generation

printer dispatch

These modalities must all enter the same session‑first execution pipeline described earlier in this architecture.

Structured result rendering

Clients must be able to render structured outputs returned from the cloud, including:

text responses

tables and structured datasets

charts and visualizations

formatted reports

PDF documents

interactive response components

Rendering logic must remain presentation‑only. The cloud determines all computation, meaning, and structure of results.

Document and media workflows

Clients must allow the user to work with documents and media directly from the interface, including:

uploading PDF files

uploading images

uploading documents

capturing images through the device camera

scanning invoices or receipts

submitting captured media for cloud processing

receiving generated reports or documents from the cloud

printing documents through the device printer

These workflows allow Selene to integrate with real‑world tasks such as accounting, reporting, document analysis, and administrative workflows.

Hardware interface responsibilities

Clients provide the hardware integration layer that enables Selene to interact with device capabilities, including:

camera access

microphone access

local printer access

file system browsing

media upload interfaces

These integrations are strictly limited to enabling user interaction and data capture.

Interaction workflow examples

Typical interaction flows may include:

user captures an invoice using the camera

client uploads the image to the cloud

cloud processes the invoice and extracts structured data

client renders the result as a table or chart

cloud generates a PDF summary

client displays the document and allows the user to print it

All computation and interpretation remain cloud‑controlled.

Interaction authority constraint

Clients must never perform or simulate any of the following locally:

identity verification

access authorization

simulation execution

memory authority decisions

artifact activation

session lifecycle mutation

All such actions must be executed within the cloud runtime under the session‑first architecture.

The client therefore acts strictly as the interaction and hardware interface layer between the user and the Selene cloud runtime.

Cloud responsibilities shared across all platforms

The Selene cloud runtime is the authoritative execution environment for the entire system. While client devices provide interaction surfaces and reliability mechanisms, all critical system authority, decision logic, and state mutation occur exclusively within the cloud runtime.

The governing rule is:

The cloud runtime decides, executes, and records system truth. Client devices only capture, display, and synchronize.

Authoritative cloud domains

The cloud runtime is responsible for all authoritative domains of the Selene system, including:

identity verification

access control and authorization

session lifecycle management

onboarding progression and completion state

natural language understanding and reasoning

simulation discovery and execution

memory storage, retrieval, and governance

learning signal evaluation and promotion

artifact creation, validation, and activation

link generation and controlled delivery

audit logging and proof capture

policy enforcement across runtime behavior

These domains define the core intelligence and authority layer of Selene.

Execution responsibilities

All meaningful system execution must occur inside the cloud runtime and within the canonical session pipeline. This includes:

processing user turns

interpreting uploaded files or images

running simulations or workflows

generating structured results or reports

producing artifacts such as documents or links

capturing audit and proof records

Clients must never perform these operations independently.

Learning governance responsibilities

The cloud runtime also governs the system learning process. This includes:

collecting candidate learning signals

filtering and evaluating signals

running policy and safety validation

producing approved learning artifacts

distributing governed updates to clients

Client devices may supply signals, but only the cloud runtime may promote learning outcomes into system behavior.

Builder and repair responsibilities

Selene Builder services also operate inside the cloud authority layer. These services support system maintenance and evolution by:

diagnosing incorrect runtime behavior

identifying architectural or implementation gaps

producing recommended repair paths

supporting controlled system upgrades

All repair and upgrade workflows must still follow the session‑first architecture and canonical execution pipeline.

Authority protection rule

Because the cloud runtime is the single source of system truth:

client devices must never override cloud decisions

client state must always reconcile with cloud responses

local device assumptions must never mutate authoritative records

This protection ensures that Selene behaves as a single deterministic system regardless of device behavior.

Client-side local assist scope

Selene client devices may maintain limited local assist state in order to improve responsiveness, interaction continuity, and reliability under unstable network conditions. This state exists only to accelerate user interaction and must never become an authoritative source of system truth.

The governing rule is:

Local assist state may improve performance and resilience, but it must always remain replaceable by authoritative cloud state.

Purpose of local assist state

Local assist state exists to support several interaction and reliability functions, including:

reducing perceived latency when displaying recent conversation results

restoring UI context after application restart

maintaining deterministic retry capability during temporary connectivity loss

supporting local audio routing and voice interaction pipelines

allowing fast resume of the most recent session context

This state improves usability but must never influence authoritative system behavior.

Allowed categories of assist state

Clients may locally store the following categories of assist data:

recent conversation display cache

session resume hints

voice‑interaction assist embeddings

audio pipeline configuration and routing state

local device capability flags

operation outbox entries

operation retry journal

approved local assist profile versions

push notification tokens

temporary UI interaction state

These categories must always be reconstructible from cloud truth or safely discardable without affecting system correctness.

Operation outbox and retry behavior

Clients must maintain a deterministic operation outbox containing operations that have been submitted locally but not yet acknowledged by the cloud runtime.

Each outbox entry must record:

operation identifier

idempotency key

operation timestamp

retry counter

acknowledgement state

associated session identifier when available

The outbox ensures that temporary network interruption cannot result in lost or duplicated operations.

Local cache constraints

Local assist caches must obey the following strict constraints:

no client‑side authority over identity

no client‑side authority over access control

no client‑side authority over simulation execution

no client‑side mutation of session lifecycle state

no local artifact activation

If local state conflicts with cloud state, the cloud state must always override device state.

Security boundary

Client‑side assist data must be treated as untrusted until validated by the cloud runtime. Local state must never be relied upon for authorization decisions, identity validation, or security enforcement.

Recoverability rule

Local assist state must be designed so that:

it can be safely discarded at any time

it can be reconstructed from cloud state

it cannot corrupt authoritative system records

This guarantees that device resets, application reinstalls, or device switching cannot damage the authoritative Selene system state.

Same capability rule across device classes

Selene must provide a consistent functional capability model across all supported client platforms. Regardless of whether the user interacts from iPhone, Android, Tablet, or Desktop, the core experience, system behavior, and cloud interaction model must remain identical.

This rule ensures that Selene behaves as one unified system, not as separate platform-specific products.

Capability Parity Principle

Every Selene client must support the same core system capabilities, including:

voice interaction

text interaction

file and document upload

image upload

camera capture workflows

structured result rendering (tables, charts, reports)

PDF generation and viewing

printer workflows where supported by the device

session open and resume

turn submission and response handling

cloud synchronization and deterministic retry

learning artifact upload

approved update application

These capabilities must operate through the same session-first runtime pipeline regardless of device class.

Cross‑Device Continuation

A user must be able to begin interaction on one device and continue seamlessly on another without losing context or workflow continuity.

Examples include:

starting a conversation on iPhone and continuing on Desktop

uploading a document on Desktop and reviewing results on a phone

capturing an invoice image on a mobile device and generating reports later on Desktop

moving between phone, tablet, and desktop while maintaining the same session history

All cross-device transitions must preserve:

session continuity

turn ordering

memory context

operation history

system state consistency

Platform Constraint Exception Rule

Differences between device platforms are permitted only when they are required by operating system or hardware constraints.

Examples include:

iPhone explicit-only trigger policy

hardware-specific camera or printer interfaces

platform UI control differences

These constraints must not change the logical behavior of Selene itself.

Capability Evolution Rule

When new capabilities are added to Selene, they must be evaluated against this parity rule.

New functionality should be implemented in a way that preserves cross-platform parity whenever possible.

If a capability cannot be supported on a specific platform due to technical limitations, the limitation must be explicitly documented and must not alter the core runtime architecture.

Canonical runtime flow

The Selene runtime follows a single canonical execution pipeline for every interaction regardless of device type, trigger method, modality, or feature surface. This pipeline guarantees that all system behavior remains deterministic, auditable, and governed by the same architectural rules.

The governing rule is:

Every interaction must pass through the same ordered execution gates before any system action is permitted.

No feature, simulation, workflow, or tool may bypass this canonical pipeline.

Canonical execution pipeline

Every interaction must follow the ordered runtime flow below:

client trigger entry (wake or explicit trigger)

client captures input (voice, text, file, image, or camera capture)

client packages a turn request

client submits the request through the canonical ingress contract

cloud ingress validation

platform trigger policy validation

session resolution or creation

identity verification

onboarding eligibility validation

memory eligibility evaluation

access authorization

simulation or tool eligibility validation

execution of the authorized operation

audit and proof capture

response payload assembly

client synchronization and rendering

This pipeline ensures that every action performed by Selene is governed by the same deterministic runtime process.

Execution gate order

The runtime must enforce the following gate sequence strictly and in order:

ingress validation

platform trigger validation

session resolve or open

identity verification gate

onboarding eligibility gate

memory eligibility gate

access authorization gate

simulation or tool eligibility gate

execution

audit and proof capture

response assembly

client synchronization outcome

If any gate fails, execution must stop and the system must fail closed.

Turn ownership model

Each execution unit inside a session is defined as a turn. A turn represents a single authoritative processing cycle within the cloud runtime.

Every turn must contain the following canonical attributes:

session_id

turn_id

originating device identifier

originating modality (voice, text, file, image, camera)

identity confidence state

idempotency identity

interruptibility state

execution timestamp

completion state (completed, superseded, aborted)

These attributes guarantee deterministic replay, correct audit tracing, and reliable cross‑device synchronization.

Deterministic execution rule

All execution decisions must originate from the cloud runtime. Clients must never determine execution outcomes locally.

This ensures:

consistent execution ordering

replay-safe system history

complete audit traceability

correct cross-device behavior

Interaction example

A typical interaction may follow this pipeline:

user captures an invoice image using a mobile device

client submits the image as part of a session turn

cloud validates session and identity

cloud evaluates access and simulation eligibility

cloud executes invoice processing logic

cloud extracts structured data

cloud generates report or visualization output

client renders the result to the user

client optionally prints or exports the generated document

Throughout the process, the cloud runtime remains the single authority for interpretation, execution, and result generation.

Multi-platform design consequence

Selene must be designed and implemented as one unified system whose behavior is consistent across all supported client platforms.

The iPhone, Android, Tablet, and Desktop applications are not independent products. They are platform-specific interfaces to the same cloud-controlled runtime architecture.

The system must therefore obey the following architectural consequences:

Single Architecture Rule

There is exactly one client-runtime architecture for Selene.

All platform implementations must inherit from this architecture.

Platform implementations may differ only where required by:

operating system restrictions

hardware capabilities

user interaction constraints

These differences must never alter Selene's logical runtime model.

Platform Implementation Rule

Each client platform must implement the same architectural capabilities:

session-first interaction model

canonical ingress contract

deterministic turn submission

cloud-authoritative decision flow

structured result rendering

local reliability and synchronization logic

approved update application

The implementation may vary technically by platform but must remain functionally equivalent.

Device Role Clarification

Every Selene client device acts as a runtime terminal attached to the cloud system.

Devices provide:

interaction interface

hardware access

local assist caching

synchronization reliability

Devices must not provide:

identity authority

access control decisions

simulation execution

memory authority

artifact activation

session lifecycle authority

These responsibilities belong exclusively to the Selene cloud runtime.

Cross‑Device User Journey

A user must be able to move between devices without breaking the system model.

Examples include:

beginning a conversation on iPhone and continuing on Desktop

uploading documents on Desktop and reviewing results on a phone

capturing an invoice image on mobile and generating reports later on Desktop

switching between devices while maintaining session continuity

The system must preserve:

session state

turn ordering

memory scope

operation history

execution integrity

across all devices.

Cross‑Device Concurrency Principle

When multiple devices attach to the same user context:

multiple devices may observe session state

only one execution path may mutate session state per turn

concurrent requests must be serialized by the cloud runtime

idempotency identities must prevent duplicate execution

This guarantees deterministic behavior regardless of the number of active devices.

Architectural Integrity Rule

Platform-specific features must never create alternate logic paths in the Selene runtime.

All capabilities must ultimately execute through the same:

session lifecycle

execution gate order

identity and access validation

simulation execution model

If a capability cannot be supported on a specific platform due to technical limitations, that limitation must be documented explicitly without altering the core architecture.

What must be aligned next

This section defines the architectural alignment roadmap required to bring the current Selene implementation into full compliance with the universal client runtime architecture defined in this document.

The objective of this alignment phase is not feature expansion, but architectural correction and stabilization so that all future development proceeds on a consistent and deterministic system foundation.

Alignment principle

The governing rule is:

Architecture integrity must be achieved before feature expansion.

If runtime behavior diverges from the architecture defined in this document, the implementation must be corrected before additional system capabilities are introduced.

Primary alignment objectives

The system must be updated so that:

Tablet becomes a formally supported platform class across runtime contracts, policy enforcement, and client capability definitions.

Platform-aware trigger policy is enforced centrally through PH1.OS rather than being implemented only at adapter boundaries.

The iPhone platform remains explicit-only for trigger entry according to platform constraints.

Android, Tablet, and Desktop platforms support both wake and explicit entry mechanisms.

All client implementations inherit from the same universal client runtime architecture without introducing alternate runtime logic paths.

All future client platforms preserve the cloud-authoritative session model.

All simulations, tools, workflows, and system capabilities execute only within the session-first execution pipeline.

Architectural correction scope

The following architectural areas must be aligned before further feature expansion:

session lifecycle behavior and identifier exposure

platform trigger enforcement through PH1.OS

client ingress contract consistency

canonical session identifier exposure across all client-visible responses

capture-bundle trust and attestation validation

artifact authenticity and trust-root verification

cross-device session continuity behavior

wake runtime parity across supported platforms

retention, purge, and deletion lifecycle completion

These corrections ensure that Selene operates as a single coherent distributed system rather than a collection of partially aligned subsystems.

Implementation discipline

All architecture corrections must follow the following implementation rules:

no architectural drift from the session-first system law

no new features that bypass canonical runtime execution gates

no platform-specific execution logic that diverges from the shared pipeline

all runtime behavior must remain cloud-authoritative

all system actions must remain deterministic and auditable

Outcome of the alignment phase

Once the alignment phase is complete:

all client platforms will operate under the same runtime contract

session behavior will be deterministic and cross-device safe

all system engines will execute through the same canonical runtime pipeline

future development can proceed through structured build phases without requiring further architectural correction

This alignment phase therefore forms the operational bridge between architecture definition and controlled system expansion.

Client ingress contract (cross-platform)

All Selene clients must interact with the cloud exclusively through a canonical ingress contract family. These ingress routes represent the only supported entry points from client devices into the Selene runtime.

Canonical ingress routes:

/v1/invite/click

/v1/onboarding/continue

/v1/voice/turn

These routes must behave identically across all supported client platforms (iPhone, Android, Tablet, Desktop). Clients must never bypass these endpoints or simulate internal system behavior locally.

Ingress security model

Every ingress request must include the required security envelope. The cloud must fail closed if any required component is invalid or missing.

Required request elements include:

Authorization bearer token

X-Request-Id (unique request correlation identifier)

X-Nonce (replay protection token)

X-Timestamp-Ms (client timestamp)

X-Idempotency-Key (operation deduplication identity)

Server runtime must enforce:

nonce replay protection

timestamp window validation

subject-to-device binding

idempotent request behavior

rate and quota controls

Idempotency and replay protection

Every client request must be uniquely identifiable so that retry logic cannot cause duplicate system execution.

Clients must generate and maintain:

stable idempotency keys

operation journal entries

retry counters

operation timestamps

If the same operation is submitted more than once, the server must recognize the idempotency identity and return the same deterministic result rather than executing the action again.

Request and response structure

Each ingress route must expose deterministic request and response semantics.

Requests must contain:

platform identifier

device identity

actor identity

session context (if available)

payload relevant to the operation

Responses must contain:

execution outcome

structured result payload

synchronization state updates

canonical identifiers including session_id, turn_id, and session_state where applicable

These identifiers allow clients to maintain correct session continuity and turn ordering across devices.

Error handling contract

Ingress endpoints must return deterministic error classifications.

Errors must indicate one of the following categories:

authentication failure

authorization failure

session lifecycle conflict

invalid request payload

policy violation

execution failure

system retry condition

Clients must treat these responses deterministically and update their local operation journals accordingly.

Versioning and forward compatibility

Ingress contracts must support version evolution without breaking existing clients.

The system must maintain:

stable endpoint semantics

clear version upgrade paths

backward compatibility guarantees where possible

Any contract change that affects request or response structure must be governed through versioned rollout policy.

Universal session payload contract

All Selene clients must consume a single canonical session payload model returned by the Selene cloud runtime. This payload provides the authoritative state required for clients to maintain deterministic session continuity, correct turn ordering, and safe cross‑device synchronization.

The governing rule is:

The cloud runtime defines the session state. Clients only observe and synchronize with it.

Canonical identifiers

Every response produced within an active or resumed session must expose the following canonical identifiers:

session_id

turn_id

session_state

These identifiers represent the authoritative runtime state of the conversation container and execution pipeline.

Identifier semantics

session_id

A globally unique identifier representing the cloud conversation container. The session_id persists for the duration of the session lifecycle and enables multiple devices belonging to the same identity to attach to the same interaction context.

turn_id

A unique identifier representing a single execution unit within the session. Each client interaction that triggers runtime execution creates a new turn_id. Turn identifiers guarantee deterministic ordering of execution history.

session_state

A server‑controlled lifecycle indicator describing the runtime state of the session. Valid states include:

Closed

Open

Active

SoftClosed

Suspended

Clients must treat these lifecycle values as authoritative and must never infer or fabricate them locally.

Session payload structure

A valid session payload returned from the cloud runtime should include the following conceptual components:

canonical session identifiers

execution outcome information

structured result payload

synchronization state updates

execution metadata

memory context signals when applicable

This payload structure ensures every Selene client platform interprets runtime state in a consistent way.

Cross‑device session continuity

Canonical identifiers enable safe cross‑device continuation of a conversation.

When a user changes devices:

clients must attach to the existing session using the authoritative session_id

turn ordering must follow the canonical turn_id sequence

session_state must reflect the authoritative lifecycle

Devices must not create new sessions unless the cloud runtime explicitly returns a new session_id.

Client interpretation rules

Clients must follow the following interpretation rules when handling session payloads:

never fabricate identifiers locally

never mutate server‑controlled lifecycle states

always reconcile local session hints with server responses

update local operation journals using canonical identifiers

synchronize UI state with authoritative runtime results

These rules ensure that session behavior remains deterministic across devices.

Versioning and forward compatibility

Session payload contracts must support forward‑compatible evolution.

Future schema changes must:

preserve canonical identifiers

maintain deterministic semantics

allow older clients to safely ignore unsupported fields

All session payload evolution must therefore follow a versioned contract policy managed by the cloud runtime.

Device trigger policy vs session model

Trigger policy is platform-specific, but the session model is universal.

This distinction is critical to the Selene architecture.

Different platforms may use different mechanisms to enter a session, but once a session is opened or resumed, every platform must operate under the same cloud-controlled session lifecycle, execution gates, memory rules, and authority boundaries.

Platform-specific trigger entry

The current trigger policy by platform is:

iPhone – explicit entry only

Android – wake word or explicit entry

Tablet – wake word or explicit entry where supported by platform contract

Desktop – wake word or explicit entry

These differences only affect how the client requests session entry.

Universal session model

Once a session is opened or resumed, the following must be identical across all platforms:

session lifecycle state model

session identifiers and turn identifiers

identity gating rules

onboarding gate behavior

memory eligibility and retrieval rules

access authorization rules

simulation and tool execution path

audit and proof capture

response assembly

synchronization outcome rules

In other words:

Trigger is allowed to vary.

Session behavior is not allowed to vary.

Architectural non-fork rule

Platform-specific trigger behavior must never create a different runtime logic system.

A wake-triggered Android turn and an explicit-triggered iPhone turn must converge into the same canonical cloud execution path once ingress validation and session resolution begin.

No platform may use trigger policy as a reason to:

change session state rules

change memory behavior

change access behavior

change simulation dispatch behavior

change audit or proof behavior

Enforcement principle

The relationship between trigger policy and session model must be enforced centrally by the cloud runtime.

This means:

client platforms may differ in entry method

PH1.OS must validate allowed trigger policy by platform

the canonical session runtime must remain identical after trigger acceptance

This section therefore establishes the hard rule that Selene is one session system with multiple entry methods, not multiple runtime systems.











Cross-platform parity rule

Selene must maintain strict functional parity across all supported client platforms so that the system behaves as one unified distributed runtime rather than a collection of platform‑specific applications.

Supported platform classes include:

iPhone

Android phone

Tablet (target platform class)

Desktop

While user interface design and hardware integrations may differ between these platforms, the logical behavior of the Selene system must remain identical.

Core parity principle

Across all client platforms the following architectural properties must remain consistent:

cloud‑authoritative execution model

session‑first runtime architecture

canonical ingress contract

session lifecycle semantics

turn ownership and deterministic ordering

identity verification and authorization rules

memory authority and eligibility filtering

synchronization and retry guarantees

learning and update governance

audit and proof capture requirements

link generation and delivery model

These invariants ensure that Selene behaves as a single coherent system regardless of the device used to access it.

Interaction capability parity

Clients must provide equivalent interaction capabilities wherever technically feasible, including:

voice interaction

text interaction

file and document upload

image upload

camera capture workflows

structured visualization output (tables, charts, dashboards)

PDF generation and viewing

printer workflows where supported

session open and resume

turn submission and response rendering

cloud synchronization and deterministic retry behavior

Differences in presentation or device APIs are allowed, but these must not change the underlying system logic or execution pipeline.

Cross‑device continuation rule

A user must be able to move between devices without breaking conversation continuity or execution context.

Examples include:

starting a conversation on iPhone and continuing on Desktop

uploading documents on Desktop and reviewing results on a phone

capturing an invoice image on mobile and generating reports later on Desktop

switching between phone, tablet, and desktop while maintaining the same session history

Cross‑device continuation must preserve:

session identifiers

turn ordering

memory context

operation history

execution integrity

Platform constraint exceptions

Certain behavioral differences are permitted when required by operating system or hardware constraints.

Examples include:

iPhone explicit‑only trigger entry

Android persistent wake listeners

platform‑specific camera and printer APIs

platform‑specific UI interaction patterns

These differences affect only interaction mechanics and must never alter the Selene runtime architecture or execution semantics.

Capability evolution rule

Whenever a new capability is introduced into Selene:

all supported platforms must be evaluated for compatibility

capability parity should be maintained wherever technically feasible

platform limitations must be explicitly documented

no platform may introduce alternate runtime logic paths

This rule guarantees that Selene evolves as a single coherent multi‑device system.

Sync, outbox, and deduplication rule

Selene clients must implement a deterministic distributed synchronization model that guarantees system correctness across unreliable networks, device switching, application restarts, and concurrent client activity.

The purpose of this model is to ensure that Selene behaves as a single coherent cloud system rather than multiple partially synchronized devices.

Core synchronization rule

The governing rule is:

The cloud runtime is always authoritative.

Client devices may temporarily queue operations and maintain assist state, but all final system state must originate from the cloud runtime.

Durable outbox requirement

Every client must maintain a durable operation outbox for actions submitted locally but not yet acknowledged by the cloud runtime.

The outbox must persist across:

application restarts

device restarts

network outages

intermittent connectivity

Each outbox record must contain the following minimum fields:

operation_id

idempotency_key

request payload reference

submission timestamp

retry counter

acknowledgement state

associated session_id

associated turn_id when available

Operations remain in the outbox until the cloud runtime returns authoritative acknowledgement.

Operation journal

In addition to the outbox, clients must maintain a local operation journal that records the lifecycle of every submitted action. The journal must record:

operation creation

submission attempts

retry attempts

server acknowledgement

final execution result

This journal guarantees that operations can be safely retried or reconciled after interruption.

Idempotent execution guarantee

Every operation must carry a stable idempotency identity generated by the client.

The cloud runtime must treat this identity as the authoritative deduplication key.

If an operation is submitted multiple times due to retries or reconnect behavior, the cloud runtime must return the same deterministic result instead of executing the action again.

This rule guarantees that retry behavior never produces duplicate system effects.

Cross-device synchronization

When the same user interacts with Selene from multiple devices:

clients must reconcile their local journals with the authoritative cloud execution history

clients must attach to the canonical session state

clients must adopt server-provided turn ordering

conflicting local state must be discarded

This ensures that all devices converge to the same authoritative execution timeline.

Device switching rule

Users may switch devices during a session. When this occurs:

new devices must fetch authoritative session state

local pending operations must be reconciled with server history

session_id and turn ordering must remain canonical

local caches must reconcile with server state

The cloud runtime remains the single writer of authoritative session state.

Reconnect procedure

When connectivity to the cloud runtime is restored, the client must execute the following deterministic sequence:

refresh authentication credentials

request authoritative session state

reconcile local operation journal with cloud execution history

resubmit unacknowledged operations using the same idempotency keys

clear acknowledged operations from the outbox

pull approved configuration or profile updates

update the UI to reflect authoritative system state

Conflict resolution rule

If any conflict occurs between local device state and cloud runtime state:

cloud state always overrides device state.

Clients must discard conflicting local state and synchronize with the authoritative cloud result.

Reliability guarantees

The synchronization system must guarantee correctness under the following conditions:

network interruption

intermittent connectivity

application restart

device restart

multi-device interaction

concurrent client activity

This rule ensures Selene remains a deterministic distributed system rather than a collection of loosely synchronized clients.

Learning and update loop

Selene operates a governed bidirectional learning and update loop between client devices and the cloud runtime. The purpose of this loop is to continuously improve system capability, maintain compatibility across devices, and safely distribute approved behavioral improvements while preserving deterministic execution and architectural integrity.

The governing rule is:

Learning is cloud‑governed. Devices may provide signals and apply approved updates, but they must never independently change system behavior.

Learning signal collection

Client devices and runtime subsystems may generate learning signals that are transmitted to the cloud learning governance layer. These signals may include:

usage telemetry

voice interaction performance metrics

error patterns

workflow success or failure indicators

environmental interaction signals

user correction events

These signals represent candidate learning inputs. They must never directly modify system behavior before passing through the governed evaluation pipeline.

Learning evaluation

The Selene cloud runtime evaluates incoming learning signals through a controlled evaluation process. This process may include:

quality filtering

confidence scoring

policy and safety validation

simulation or sandbox testing

statistical validation

Only signals that pass the evaluation process may influence future system improvements.

Learning artifact creation

Approved learning outcomes are packaged into governed learning artifacts. These artifacts represent controlled improvements such as:

voice recognition improvements

interaction model tuning

device capability adjustments

runtime configuration improvements

behavioral guidance parameters

All artifacts must be versioned, signed, and stored under artifact governance before distribution.

Update distribution

Once approved, artifacts may be distributed from the cloud runtime to client devices. Distribution may include:

voice‑recognition assist updates

client configuration updates

device compatibility adjustments

model support updates

behavioral tuning parameters

Client devices must treat all distributed artifacts as cloud‑authoritative updates.

Safe update application sequence

Client devices must apply updates using the following deterministic procedure:

download update artifact

verify authenticity and integrity

stage update locally

apply update

confirm successful activation

rollback automatically if validation fails

This sequence guarantees that faulty updates cannot corrupt the client runtime environment.

Rollback safety

If an update fails validation or causes instability, the client must revert to the last known valid configuration without affecting active sessions.

Rollback procedures must preserve:

session continuity

operation journal state

outbox reliability guarantees

local assist caches required for synchronization

Governance boundary

Learning promotion and system behavior modification remain exclusively cloud‑governed processes.

Client devices must never independently modify:

simulation behavior

access authorization rules

session lifecycle rules

artifact activation

memory governance

All behavioral evolution must originate from the cloud governance layer and propagate through the controlled update pipeline described above.

Cloud authority boundary

The Selene cloud runtime is the single authoritative source of truth for all critical system state, execution decisions, and compliance records. Client devices interact with the system, but they never become the authority for identity, execution outcomes, memory truth, or artifact activation.

The governing rule is:

All authoritative state transitions must originate in the cloud runtime.

No client device may override, fabricate, or independently finalize system truth.

Authoritative cloud domains

The cloud runtime is authoritative for the following system domains:

identity verification

access control and authorization

session lifecycle state

onboarding state and completion

simulation selection and execution

memory storage, retrieval, and governance

learning evaluation and promotion

artifact approval and activation

audit, proof, and compliance records

policy enforcement across runtime behavior

Any state in these domains must be treated as valid only when returned or confirmed by the cloud runtime.

Device limitation rule

Client devices may assist with capture, caching, rendering, local reliability mechanisms, and hardware integration. However, they must never become authoritative for:

identity truth

access decisions

session lifecycle state

simulation execution outcomes

memory authority

artifact approval or activation

audit or compliance records

Devices may hold temporary local assist state only for speed, continuity, and resilience.

Trust boundary model

The system must treat client devices as untrusted environments. All client‑originated inputs must be validated by the cloud runtime before they influence authoritative system behavior.

Trusted only after cloud validation:

session identifiers and lifecycle state

identity assertions

access and policy decisions

memory records

artifact approval and activation

audit and proof records

Untrusted until verified:

client capture metadata

client device claims

local cached permissions

local session hints

uploaded artifacts before verification

local timestamps

Verification‑before‑authority rule

Before any client‑originated input can influence authoritative runtime behavior, it must pass the required verification path.

Examples include:

capture‑bundle attestation before voice or wake runtime reliance

artifact authenticity and trust‑root verification before activation

identity verification before entering user memory scope

policy validation before execution or update application

Cloud‑wins conflict rule

If local client state and cloud runtime state diverge:

cloud state always wins.

Clients must discard conflicting local state and synchronize with the authoritative cloud result.

Authority boundary principle

This boundary is a core Selene architectural law:

Clients are interaction terminals and assist layers.

The cloud runtime is the authority layer, execution engine, and source of system truth.

Platform inventory

The Selene client ecosystem is designed around a defined set of platform classes. Each platform class represents a runtime environment capable of hosting a Selene client application that connects to the same cloud‑authoritative system.

The currently recognized platform classes are:

iPhone

Android phone

Tablet

Desktop

These platforms differ in hardware capabilities and operating system constraints, but all must conform to the same session‑first runtime architecture defined in this document.

Platform role definition

Each platform class represents a runtime terminal connected to the Selene cloud system. The role of these terminals is to provide:

user interaction interfaces

hardware integration (camera, microphone, printer, file access)

session entry and continuation

structured output rendering

local reliability and synchronization

Platform devices must never act as authoritative runtime decision engines.

Platform capability baseline

Every platform must support the minimum Selene client capability baseline:

session entry and resume

turn submission

structured result rendering

cloud synchronization

deterministic retry and idempotent request handling

approved update application

local reliability mechanisms

The exact user interface or hardware integrations may vary, but the logical system behavior must remain identical.

Platform capability differences

Certain platform behaviors may differ due to operating system policies or hardware design.

Examples include:

iPhone uses explicit session entry rather than wake word

Android devices may support always‑listening wake engines

Desktop devices may run persistent background listeners

Tablets may inherit Android‑style wake policies depending on platform implementation

These differences affect only the entry mechanism or hardware interaction layer.

They must never alter the core Selene runtime model.

Tablet platform status

Tablet is currently treated as a target platform class.

Tablet support must eventually include:

platform enumeration in contracts

runtime trigger policy support

client capability parity

synchronization and session behavior identical to other clients

Until tablet support is fully implemented, it remains a TARGET platform class within the architecture.

Platform evolution rule

Future Selene platform classes may be introduced (for example new device categories).

Any new platform must satisfy the following rule:

The new platform must integrate with the same session‑first architecture and canonical ingress contract without introducing alternate runtime logic paths.

This ensures Selene continues to operate as one unified distributed system across all client platforms.













Scope boundaries for this architecture

This document defines the parent runtime architecture governing how Selene clients interact with the Selene cloud runtime. It establishes the system laws, architectural contracts, and authority boundaries that all Selene implementations must follow.

The purpose of this section is to clearly separate architectural law from implementation detail so that the architecture remains stable even as platform implementations evolve.

Architectural scope

This architecture governs the following core system domains:

client runtime interaction model

session‑first execution law

cross‑platform capability parity

canonical ingress contract

session lifecycle behavior

identity and onboarding authority

memory authority and retention model

synchronization and retry guarantees

learning and update governance

artifact lifecycle and activation authority

cloud authority boundary

cross‑device continuation behavior

These domains represent the non‑negotiable architectural rules that define how Selene must behave as a distributed intelligence system.

Out‑of‑scope implementation details

The following topics are intentionally excluded from this architecture document because they belong in platform‑specific or subsystem design documents:

exact UI layouts

screen‑level user experience flows

platform‑specific interaction controls

native hardware integration APIs

internal model packaging formats

client build pipelines and deployment tooling

administrative or operational interface design

These areas may vary between implementations but must always remain compliant with the architectural rules defined in this document.

Extension rule

Platform‑specific architecture documents (for example iPhone, Android, Tablet, or Desktop client specifications) may extend this parent architecture when additional detail is required by operating system behavior or hardware capability.

However, such extensions must never:

contradict the session‑first system law

bypass the canonical execution pipeline

weaken the cloud authority boundary

introduce alternate runtime logic paths

change identity, access, memory, or artifact authority rules

All extensions must remain strictly compatible with the architectural constraints defined here.

Architectural compliance principle

All Selene subsystems, engines, platform implementations, and future system extensions must be evaluated against this document for architectural compliance.

If a proposed subsystem or feature conflicts with the architectural constraints defined here, the architecture must be corrected first before implementation proceeds.

Purpose of the scope boundary

Defining explicit architectural scope ensures that:

core system laws remain stable over time

implementation layers can evolve safely

new features cannot accidentally violate architectural invariants

Selene continues to operate as a single coherent distributed intelligence system across all devices and environments.

Immediate next step

This architecture document must now transition from definition into controlled implementation alignment.

The next step for Selene development is to treat this document as the governing architecture baseline and perform a structured architecture‑correction phase before expanding system features.

Architecture alignment phase

The first engineering phase must focus on aligning the current runtime with the architecture defined in this document.

This phase must include:

exposing the canonical session identifiers in client responses

centralizing platform trigger policy enforcement in PH1.OS

introducing capture‑bundle attestation before runtime execution

implementing artifact authenticity and trust‑root verification

completing Android microphone and wake runtime parity

implementing retention, purge, and deletion workers

preparing cross‑device session continuity support

formalizing tablet as a supported runtime platform

During this phase no new product features should be introduced that bypass or alter the session‑first system law.

Implementation discipline

All implementation work during the alignment phase must follow these rules:

all runtime behavior must execute through the canonical execution pipeline

all changes must preserve cloud‑authoritative state ownership

no platform‑specific runtime forks may be introduced

all changes must maintain deterministic replay and audit capability

all changes must maintain compatibility with the canonical ingress contract

Architecture validation

Before proceeding to feature expansion, the system must validate that:

all client platforms operate under the same runtime contract

session identifiers and lifecycle semantics are consistent

synchronization logic behaves deterministically

device switching does not break session continuity

memory governance remains identity‑scoped and cloud authoritative

If any subsystem violates the architectural constraints defined in this document, the architecture must be corrected before further development continues.

Transition to structured build runs

Once architecture alignment is verified, development may proceed into structured build runs.

These runs should focus on:

completing remaining engine gaps

implementing platform client applications

strengthening synchronization and cross‑device behavior

completing memory and audit lifecycle enforcement

expanding system capability while preserving architecture integrity

This section therefore marks the transition point between architecture definition and controlled system implementation.

Voice enrollment authority contract

Voice enrollment is a mandatory identity‑establishment stage in the Selene onboarding lifecycle. It produces the authoritative voice identity artifact used by the PH1.VOICE.ID engine to verify the speaker during future sessions.

Voice enrollment must be cloud‑authoritative and must complete successfully before onboarding may reach a fully ready state.

Purpose of voice enrollment

Voice enrollment establishes the following identity guarantees:

binding between the user account and a verified speaker identity

creation of voice identity artifacts used by the PH1.VOICE.ID engine

future session speaker verification capability

secure entry into identity‑scoped memory and workflow execution

Without a locked voice identity artifact, Selene must not allow the user to enter authoritative identity scope.

Authority rule

Voice enrollment authority belongs exclusively to the Selene cloud runtime.

Client devices act only as capture terminals during the enrollment process.

The cloud runtime is responsible for:

validating enrollment samples

building the voice identity artifact

storing identity artifacts

verifying artifact synchronization

transitioning enrollment status to the Locked state

Client devices must never finalize enrollment or activate identity artifacts locally.

Enrollment completion requirements

Before onboarding can transition to a ready state, the following conditions must be satisfied:

voice enrollment state must be Locked

enrollment artifacts must be synchronized to the cloud runtime

the identity profile must exist within PH1.VOICE.ID identity scope

a verifiable artifact receipt must be produced by the cloud runtime

If any of these requirements are missing, onboarding must fail closed and remain incomplete.

Client capture role

Client devices support enrollment by performing the following tasks:

capturing voice samples through the microphone

submitting captured samples to the cloud enrollment service

displaying enrollment progress feedback to the user

Client devices must never:

store authoritative identity artifacts

decide enrollment completion

activate voice identity artifacts

Identity activation

Once enrollment reaches the Locked state:

the voice identity artifact becomes active in PH1.VOICE.ID

future sessions may perform speaker verification using this artifact

identity‑scoped memory access becomes permitted

session interactions may enter authoritative user scope

Security and artifact integrity

Voice identity artifacts must follow strict governance rules:

artifacts must be cryptographically verifiable

artifacts must be stored in identity‑scoped cloud storage

activation must occur only after successful artifact synchronization

artifacts must obey retention and lifecycle governance rules

Cross‑device implications

Once a voice identity artifact is locked:

any device belonging to the user identity may submit voice turns

the cloud runtime performs speaker verification centrally

client devices never perform authoritative identity decisions

This ensures voice identity remains portable across devices while remaining cloud‑authoritative.

Failure behavior

If enrollment cannot be validated:

onboarding must remain incomplete

identity‑scoped memory and workflows must remain disabled

the user must repeat the enrollment process

The system must always fail closed rather than allowing partial identity establishment.

Platform divergence matrix

Selene supports multiple device classes whose hardware and operating‑system constraints differ. The Platform Divergence Matrix defines how these differences are handled without violating the universal client runtime architecture.

The guiding rule is:

Platform differences may affect interaction mechanics, but must never change the session‑first execution model or cloud‑authoritative system behavior.

Canonical onboarding progression

All platforms must follow the same authoritative onboarding sequence:

invite or open

onboarding session start

required platform receipts

missing fields, terms acceptance, primary device confirmation, and sender verification when required

voice enrollment

wake enrollment where platform requires it

personality lock

access‑related onboarding steps

complete

ready

This order must remain deterministic and cloud‑controlled.

Platform‑specific differences

Some onboarding and runtime behaviors differ by platform due to OS constraints.

Voice enrollment is required on all platforms.

Wake enrollment behavior differs as follows:

iPhone — wake enrollment disabled (explicit trigger model only)

Android — wake enrollment required

Tablet — wake enrollment required unless constrained by future platform contract

Desktop — wake enrollment required

Platform receipts

Different platforms must provide different hardware capability receipts during onboarding.

Examples include:

iPhone requires ios_side_button_configured

Android devices require wake configuration receipts

Desktop devices require wake runtime readiness receipts

Tablet receipts follow Android‑style behavior unless platform restrictions apply.

These receipts allow the cloud runtime to verify that device capabilities are correctly configured before onboarding completion.

Platform runtime behavior

Even when trigger mechanisms differ, all platforms must converge to the same runtime pipeline after session entry.

Once a session begins, all platforms must share identical behavior for:

session lifecycle state

turn sequencing

memory access

simulation execution

audit and proof capture

synchronization logic

No platform may alter the execution pipeline or introduce alternative runtime semantics.

Platform evolution policy

New device classes may be added in the future. When this occurs:

platform capabilities must be formally enumerated in the platform contract

trigger policy must be defined in PH1.OS

client runtime must remain compatible with the canonical ingress contract

session‑first architecture must remain unchanged

This ensures Selene can expand to new device environments while preserving a single unified runtime architecture.

Personality lock semantics

The Personality Engine defines how Selene adapts communication style to individual users while maintaining strict separation between conversational presentation and system authority. Personality classification occurs during onboarding and becomes part of the user’s identity profile stored in the Selene cloud runtime.

Personality classification categories

The current architecture recognizes three personality classifications:

Passive

Domineering

Undetermined

The resulting classification is stored as an identity‑scoped attribute and referenced by the PH1.EMO and PH1.PERSONA engines whenever a session is initialized.

Purpose of personality classification

Personality classification exists solely to improve interaction quality. It allows Selene to adapt how information is communicated without changing how the system reasons, authorizes, or executes tasks.

Examples of personality‑driven presentation adjustments include:

response tone

conversation pacing

level of explanation detail

guidance or directive strength

instruction clarity and structure

These adjustments affect presentation only and must never alter runtime authority or system decisions.

Authority separation rule

Personality classification must never influence any authoritative runtime behavior.

The following domains remain completely independent from personality:

identity verification

access authorization

simulation eligibility

execution gate order

security validation

session lifecycle control

memory authority

artifact activation

policy enforcement

All authoritative decisions must continue to follow the deterministic execution pipeline defined earlier in this architecture.

Operational scope

In the current system design personality influences conversational tone only.

Example behaviors include:

more structured and directive responses for Passive users

more collaborative conversational tone for Domineering users

balanced neutral tone for Undetermined classifications

These adjustments operate strictly at the presentation layer and do not modify system behavior.

Identity‑scoped personality artifact

Personality classification is stored as an identity artifact governed by the cloud runtime. This artifact must follow the same governance model as other identity artifacts, including:

secure cloud storage

retrieval during session initialization

policy‑controlled lifecycle management

eligibility for refinement through the governed learning pipeline

Cross‑device consistency

Personality is identity‑scoped rather than device‑scoped. This ensures that:

all devices attached to the same identity share the same personality context

communication style remains consistent when switching devices

client devices never independently determine personality classification

Future evolution boundary

Future system phases may expand personality behavior to include more advanced adaptive interaction strategies. However, such expansion must remain strictly limited to communication style and must never influence deterministic system authority.

Failure and fallback behavior

If personality classification fails or remains undetermined:

Selene must default to a neutral communication style

system execution must remain unaffected

classification may be retried later through the governed learning process

This design guarantees that personality improves communication quality while preserving Selene’s core architectural law: deterministic authority remains separate from probabilistic interaction style.

Session lifecycle contract

Selene sessions are cloud‑owned runtime containers that govern the lifecycle of a user interaction with the system. Sessions coordinate conversation flow, execution context, identity scope, and synchronization across devices while ensuring deterministic execution and auditability.

The session lifecycle is fully controlled by the Selene cloud runtime and must never be altered by client devices.

The governing rule is:

Session lifecycle authority belongs exclusively to the cloud runtime.

Client devices may observe session state and request interaction, but they must never mutate lifecycle state locally.

Canonical session states

The Selene runtime defines the following deterministic session lifecycle states:

Closed

Open

Active

SoftClosed

Suspended

Each state represents a specific runtime condition governed by server policy.

Closed

No active interaction context exists for the user. A new trigger or explicit request must create a new session before execution may occur.

Open

A session container has been created but the user has not yet submitted an executable turn.

Active

The user is actively interacting with Selene and turns are being processed through the canonical runtime pipeline.

SoftClosed

Interaction has paused due to inactivity but the session context remains recoverable for a limited time window.

Suspended

The session is temporarily paused due to runtime conditions such as degraded audio capture, policy intervention, or system protection behavior.

Session state transitions

Session transitions must follow deterministic server‑controlled rules:

Closed → Active

A trigger or explicit interaction opens a new session and begins processing.

Active → Active

Continued interaction keeps the session active while new turns are submitted.

Active → SoftClosed

User inactivity exceeds the configured inactivity threshold.

SoftClosed → Active

User resumes interaction within the recoverable window.

SoftClosed → Closed

Extended inactivity expires the session container.

Any state → Suspended

The system may temporarily suspend the session due to policy or runtime protection conditions.

Inactivity policy

Sessions remain Active while the user continues interacting with Selene.

If the user stops speaking or interacting, the cloud runtime begins an inactivity timer.

The default architectural target is:

approximately 30 seconds of inactivity before transitioning toward SoftClosed.

The exact duration must remain configurable through policy controls.

Before final closure the runtime may optionally confirm whether the user intends to end the interaction when contextual signals suggest the conversation may still continue.

Cross‑device attachment behavior

Multiple client devices may attach to the same user identity and observe the same session.

However, session mutation remains single‑writer authoritative within the cloud runtime.

This means:

multiple devices may observe session state

multiple devices may submit turns

only one execution path may mutate session state per turn

concurrent submissions must be serialized by the cloud runtime

idempotency identities prevent duplicate execution

Device switching

Users may switch devices during an active session.

When this occurs:

new devices must retrieve the authoritative session state

the existing session_id must remain canonical

turn ordering must remain deterministic

local device caches must reconcile with cloud state

Current implementation limitation

Current runtime behavior resolves session continuity primarily by actor plus device scope.

True shared cross‑device session continuity remains a target architecture capability and is not yet fully implemented in the current runtime contract.

Session authority principle

The Selene cloud runtime is the only authority allowed to:

create session identifiers

change session lifecycle state

resolve session conflicts

close, suspend, or reopen sessions

Client devices must only observe and synchronize with the authoritative session lifecycle.

Client session identifier exposure

Selene clients must operate using explicit canonical session identifiers returned by the cloud runtime. These identifiers form the synchronization backbone that allows multiple devices, retries, and distributed execution to remain deterministic and auditable.

The governing rule is:

Clients must never infer or fabricate session identifiers. All identifiers originate from the cloud runtime.

Canonical identifiers

The Selene runtime must expose the following identifiers in client‑visible responses whenever a session context exists:

session_id

turn_id

session_state

These identifiers are generated exclusively by the cloud runtime and represent authoritative execution context.

Identifier roles

session_id

Represents the authoritative conversation container in the cloud runtime. A session may span multiple turns and may be observed or attached to by multiple devices belonging to the same identity.

turn_id

Represents a single execution unit inside the session lifecycle. Each processed interaction that triggers system execution must produce a unique turn identifier.

session_state

Represents the lifecycle state of the session as defined by the Session Lifecycle Contract. Valid states include Closed, Open, Active, SoftClosed, and Suspended.

Exposure requirement

Every server response generated from a session‑bound execution must expose canonical identifiers whenever applicable. This guarantees that:

clients maintain deterministic turn ordering

operation journals can reconcile execution history

cross‑device session continuation is possible

audit and proof records can reference stable identifiers

Client handling rules

Clients must treat canonical identifiers as read‑only authoritative state.

Clients must:

store identifiers in the local operation journal

attach identifiers when retrying operations

use session_id when attaching to an existing session

use turn_id to reconcile execution results

Clients must never:

invent session identifiers

reuse turn identifiers across operations

mutate server‑provided session_state values

Cross‑device continuation behavior

Canonical identifiers allow multiple client devices to safely participate in the same conversation lifecycle.

When switching devices:

the new client must attach using the server‑provided session_id

turn ordering must follow the canonical turn_id sequence

local state must reconcile with server state

any divergent local execution history must be discarded

Implementation gap acknowledgement

Current runtime behavior does not yet expose canonical session identifiers consistently in all client‑visible responses. Closing this gap is a priority architectural correction so that all clients operate under an explicit session contract.

Memory authority and identity scope

Selene memory is a cloud‑authoritative, identity‑scoped knowledge system that allows the runtime to retain and reuse information across sessions while maintaining strict security, consent, and policy controls.

Memory is never owned by the device and is not owned by the session. Sessions may reference memory during execution, but the authoritative records exist only in the cloud memory subsystem.

Memory authority principle

The Selene cloud runtime is the only authority allowed to:

create memory records

update memory records

remove or tombstone memory records

evaluate memory eligibility for retrieval

apply memory policy rules

Client devices must never:

store authoritative memory records

modify memory records

activate memory state

infer memory truth

Devices may only cache temporary assist hints that can always be replaced by cloud memory truth.

Identity‑scoped memory access

All memory records are scoped to a verified user identity.

Authoritative memory access requires:

confirmed voice identity or authenticated user identity

successful identity scope validation

policy eligibility checks

Without confirmed identity, the system must not enter authoritative user memory scope.

This prevents memory leakage between users or ambiguous identity conditions.

Memory eligibility filters

Before memory can be retrieved during a session, the cloud runtime must apply the following filters:

identity scope validation

memory sensitivity policy

confidence scoring

permission requirements

retention policy validation

Only memory entries that pass all filters may be returned to the execution context.

Memory types

Selene memory may include several categories of information such as:

user preferences

contact or workflow references

task‑related historical context

learned interaction patterns

system‑generated knowledge artifacts

Each memory record must include provenance metadata describing:

originating session

originating identity

confidence score

sensitivity classification

retention class

Memory write rules

New memory entries generated during a session must follow the deterministic write path:

candidate generation during execution

policy evaluation

append to memory ledger

materialize in current memory view

Only after the ledger entry is accepted may the memory be considered active.

Security and privacy guarantees

Memory records must follow strict security rules:

memory must never be visible outside its identity scope

sensitive memory must require additional authorization before use

memory must be auditable and traceable through provenance metadata

memory access must always be policy‑validated

Cross‑session behavior

Because memory is identity‑scoped rather than session‑scoped:

memory survives session termination

future sessions may retrieve memory when identity is confirmed

multiple devices may reference the same identity memory safely

This ensures continuity of knowledge across sessions without compromising security boundaries.

Memory persistence model

Selene memory persistence follows a ledger‑first architecture designed to guarantee determinism, traceability, and policy‑governed lifecycle management. In this model, the authoritative history of all memory mutations is recorded before the system exposes memory to runtime execution.

The governing rule is:

All memory state must be derivable from an append‑only ledger of memory events.

The runtime memory view is therefore a materialized projection derived from this ledger rather than an independently mutable data store.

Ledger‑first persistence

All memory operations must first produce an event written to the memory ledger. Only after the ledger entry is accepted may the resulting memory state appear in the active runtime memory view.

The ledger records the authoritative history of memory changes and enables:

reconstruction of memory state

audit verification

policy enforcement

learning signal analysis

historical replay for debugging and validation

Ledger event types

The memory ledger records lifecycle events that describe how memory evolves over time. Typical event classes include:

Stored

Updated

Forgotten

PolicyExpired

Each ledger entry must include provenance metadata such as:

identity scope

originating session_id

originating turn_id

timestamp

confidence score

sensitivity classification

retention class

This metadata ensures that every piece of stored knowledge can be traced back to its origin and evaluated against governance policy.

Materialized memory view

The system maintains a materialized memory view that represents the current set of active memory entries eligible for retrieval by the runtime.

This view is derived from the ledger and must always be recomputable from ledger history. The view is therefore a performance optimization rather than the authoritative store.

Runtime systems interact with this view during execution, while the ledger remains the canonical source of truth.

Memory temperature model

Selene memory operates under a temperature‑based retention strategy to balance performance, relevance, and policy compliance.

Hot memory

Retained for approximately 72 hours.

Optimized for short‑term conversational continuity and recent context reuse.

Medium memory

Retained for approximately 30 days.

Represents short‑term contextual knowledge and recurring workflow references.

Cold memory

Retained indefinitely unless removed by policy, explicit deletion, or lifecycle rules.

Represents durable long‑term knowledge and user identity context.

Retention durations must remain configurable through policy controls so that governance requirements can evolve without architectural changes.

Session interaction with memory

Sessions reference memory but do not own it.

During execution:

eligible memory candidates are retrieved from the materialized view

candidates are filtered by identity scope and policy rules

approved memory context is injected into the execution pipeline

If new knowledge emerges during execution, the system may produce memory candidates that must pass the ledger write path before becoming part of the active memory view.

Lifecycle management

Memory persistence must support explicit lifecycle operations including:

retention expiration

policy‑driven deletion

explicit user removal

privacy or regulatory compliance requests

Every lifecycle action must generate a ledger event so that memory state transitions remain auditable.

Device interaction rule

Client devices must never store authoritative memory records.

Devices may maintain temporary assist data such as:

recent conversation hints

UI rendering context

voice recognition assist data

These caches must always be replaceable by the authoritative cloud memory state.

Determinism and recoverability

Because the runtime memory view is derived from the ledger:

memory state can be reconstructed deterministically

system recovery can replay ledger history

audit verification can inspect the entire lifecycle of any memory entry

This persistence model ensures Selene memory remains consistent, recoverable, and policy‑compliant across sessions, devices, and system upgrades.

Local assist cache vs cloud truth

Selene client devices may maintain limited local assist caches to improve responsiveness, interaction continuity, and resilience during temporary connectivity issues. These caches exist purely as performance optimizations and must never become authoritative sources of system state.

The governing rule is:

Local cache improves speed. Cloud runtime defines truth.

Purpose of local assist caches

Local assist caches exist to support a small set of device‑level behaviors that improve user experience without influencing system authority. These include:

fast rendering of recent conversation history

restoring UI state after application restart

maintaining session resume hints

preserving temporary audio routing state

storing local wake or voice assist metadata

tracking pending operations in the retry outbox

maintaining transient UI interaction state

These functions allow the client to remain responsive while authoritative state is retrieved from the cloud runtime.

Allowed cache categories

Client devices may maintain assist data in the following categories:

recent conversation rendering cache

recent turn display state

voice interaction assist metadata

audio routing and playback configuration

local device capability flags

pending operation outbox entries

retry journal metadata

approved local assist profile versions

push notification tokens

transient UI interaction state

All cached data must remain reconstructible from authoritative cloud state.

Cache invalidation rule

Local caches must be refreshed or invalidated whenever authoritative cloud state changes. This includes situations such as:

session state transitions

new turn execution results

operation reconciliation events

client reconnect after network interruption

configuration or profile updates

Clients must reconcile cached information with server responses before presenting system state to the user.

Conflict resolution rule

If any difference exists between local cache data and cloud runtime state:

cloud state always overrides local cache.

The client must discard conflicting cache entries and synchronize with the authoritative cloud response.

Security boundary

Local caches must never be trusted for any authoritative system decision. Cached data must not be used to determine:

identity verification

access authorization

simulation eligibility

memory authority

artifact activation

session lifecycle state

All such decisions must always be verified by the cloud runtime.

Recoverability guarantee

Local assist caches must be designed so they can be safely cleared at any time without damaging system correctness. The client must be able to:

reconstruct UI state from server responses

rebuild execution history from operation journals

replay pending operations from the durable outbox

resynchronize with the authoritative cloud runtime

This guarantee ensures that Selene remains a deterministic distributed system even if a device cache is lost, corrupted, or reset.

Learning and update pipeline

Selene operates a governed learning and update pipeline that allows the system to improve capabilities while preserving deterministic behavior, security, and auditability. The pipeline is intentionally centralized in the cloud runtime so that learning signals cannot directly alter system behavior without evaluation and approval.

Learning signal collection

Client devices and runtime subsystems may generate learning signals that are sent to the cloud learning system. These signals may include:

interaction telemetry

speech recognition accuracy metrics

workflow success or failure signals

error patterns

environmental interaction context

user correction events

These signals are treated as candidate learning inputs, not authoritative knowledge.

Learning evaluation

The Selene cloud runtime evaluates candidate learning signals using the governed learning system. Evaluation may include:

signal validation

confidence scoring

policy compliance checks

simulation replay or sandbox validation

safety analysis

Only signals that pass the evaluation pipeline may influence future system behavior.

Learning artifact creation

Approved learning outcomes are packaged as learning artifacts. These artifacts represent controlled changes such as:

voice recognition improvements

interaction model adjustments

device capability tuning

system configuration refinements

behavioral guidance parameters

Learning artifacts must be versioned, signed, and stored under artifact governance rules before distribution.

Update distribution

Once approved, artifacts may be distributed to client devices as updates. Distribution must follow a controlled rollout process that may include:

version targeting

device capability checks

platform compatibility validation

progressive rollout policy

rollback safeguards

Client update application

Client devices must apply updates using a deterministic sequence:

download

verify authenticity and integrity

stage update locally

apply update

confirm successful activation

rollback if validation fails

This ensures that updates cannot corrupt the runtime environment or interrupt active sessions.

Feedback loop

After updates are applied, devices may generate additional signals indicating:

update success

runtime performance changes

unexpected behavior

These signals return to the learning pipeline to continuously improve system performance.

Governance rule

Learning and behavior modification are exclusively cloud-governed processes.

Client devices must never independently modify:

simulation logic

access rules

session lifecycle behavior

memory governance

artifact activation

All behavioral evolution must pass through the governed learning pipeline described above.





Offline, reconnect, and deduplication model

Selene must behave as a deterministic distributed system even when client devices experience network interruption, device restarts, application crashes, or cross‑device switching. The offline, reconnect, and deduplication model guarantees that no user action is lost, duplicated, or executed inconsistently.

The governing rule is:

No client action is considered complete until the cloud runtime acknowledges it.

Offline behavior

When a client temporarily loses connectivity to the Selene cloud runtime:

new user actions must be written to the durable operation outbox

operations must receive a stable idempotency identity

the UI must reflect pending or offline state

no operation may be marked successful locally

Clients may continue capturing user input, but authoritative execution remains pending until connectivity returns.

Durable operation storage

All pending actions must be stored in the durable outbox described in the synchronization model. This storage must survive:

application restart

device restart

intermittent connectivity

Each operation entry must contain:

operation_id

idempotency_key

associated session_id

associated turn_id (when available)

submission timestamp

retry counter

acknowledgement status

This information allows safe retry and reconciliation when the client reconnects.

Reconnect reconciliation

When connectivity is restored the client must execute a deterministic reconciliation sequence:

refresh authentication credentials

retrieve authoritative session state from the cloud

compare the local operation journal with server execution history

resubmit any unacknowledged operations using the same idempotency identities

remove acknowledged operations from the outbox

pull approved configuration or update artifacts

refresh the UI to reflect authoritative system state

This procedure guarantees that the client and cloud converge to the same execution timeline.

Idempotent execution guarantee

All client operations must include a stable idempotency identity.

The cloud runtime must treat this identity as the deduplication key. If an operation is submitted multiple times due to retries or reconnect behavior, the cloud runtime must:

recognize the idempotency identity

return the previously computed result

prevent duplicate execution

This ensures that unstable connectivity cannot produce duplicated system actions.

Cross‑device reconciliation

When the same user interacts with Selene from multiple devices:

clients must reconcile their local journals with the authoritative cloud execution history

clients must attach to the canonical session_id

clients must adopt server‑provided turn ordering

any divergent local state must be discarded

Clients must never attempt to merge divergent execution histories locally.

Conflict resolution rule

If any conflict occurs between device state and cloud runtime state:

cloud state always overrides device state.

Clients must discard conflicting local entries and synchronize with the authoritative cloud result.

Reliability guarantee

This model ensures that Selene maintains deterministic execution under the following conditions:

network interruption

intermittent connectivity

application restart

device restart

multi‑device interaction

concurrent client activity

Through this mechanism Selene behaves as one consistent distributed system rather than a collection of loosely synchronized clients.

Link generation and delivery model

Link generation and delivery are cloud-owned, session-bound operations. They must execute only within the canonical Selene runtime pipeline and must never be performed as uncontrolled client-side behavior.

The governing rule is:

A link may only be generated or delivered from an authorized session context after identity, access, and execution gates have passed.

Session-bound origin rule

All link workflows must begin from an active or resumed session.

This means:

a user request to create or send a link must originate inside session context

the request must pass through the canonical execution gate order

the cloud runtime must determine whether the action is authorized

No link workflow may bypass the session-first architecture.

Canonical link workflow

The standard runtime flow for link generation and delivery is:

user requests link generation from an active session

cloud verifies identity

cloud verifies access and policy eligibility

cloud resolves the correct simulation or workflow path

cloud generates the authoritative link artifact

cloud optionally executes link delivery through the approved delivery engine

cloud records audit and proof state

client renders the result to the user

This ensures link workflows remain deterministic, auditable, and policy-governed.

Cloud authority rule

Clients must never:

generate authoritative links locally

sign or approve invite links locally

decide delivery eligibility locally

record final delivery truth locally

All such actions belong exclusively to the cloud runtime.

Delivery path rule

Under the current architecture, link delivery is aligned to the BCAST engine path.

This means:

link generation is performed through the authorized cloud simulation path

delivery is executed through the broadcast and delivery path

audit and proof state are recorded in cloud systems

BCAST therefore remains the authoritative delivery path for sending links under the current system design.

Public API boundary

This parent architecture does not define a public client-facing invite generate or send API.

Current authoritative behavior assumes:

link generation and sending occur through internal cloud simulation and delivery paths

client applications request outcomes through session interactions

If a future public administrative API is introduced, it must still comply with the same identity, access, session, audit, and proof rules defined in this architecture.

Link artifact governance

Generated links are artifacts and must obey artifact lifecycle rules, including:

cloud-owned creation

policy-governed activation

audit linkage to session_id and turn_id

retention and deletion policy compliance

proof capture for generation and delivery events

Failure behavior

If any required gate fails during link generation or delivery, the system must fail closed.

Examples include:

identity not confirmed

access denied

simulation path unavailable

delivery policy violation

artifact generation failure

In these cases:

no authoritative link may be issued

no delivery may be recorded as successful

the client must receive the authoritative failure outcome from the cloud runtime

This ensures that link workflows remain secure, deterministic, and fully aligned with the Selene session-first architecture.

Device vs cloud responsibility boundary

Selene operates as a cloud‑authoritative distributed intelligence system. Client devices provide interaction surfaces and reliability support, while the Selene cloud runtime performs all authoritative reasoning, execution, and state mutation.

The governing rule is:

Devices capture and present. The cloud decides and executes.

This boundary guarantees that Selene behaves as a single deterministic system regardless of device behavior, device trust level, or network conditions.

Interaction domain (device responsibilities)

Client devices exist to interact with the user and the physical environment. Devices are responsible for:

voice capture through the microphone

audio preprocessing and routing

wake detection or explicit trigger handling

text input capture

file upload and document selection

camera capture and media upload

printer access through local device interfaces

rendering system responses (text, charts, dashboards, documents)

audio playback

UI interaction flows

local assist caching

operation journaling and retry outbox

synchronization with the cloud runtime

These responsibilities allow the system to interact with users and the physical environment but do not grant the device any authority over system truth.

Authority domain (cloud responsibilities)

All authoritative Selene behavior must occur in the cloud runtime. This includes:

identity verification and speaker recognition

access authorization and policy validation

session lifecycle control

onboarding progression and completion

simulation discovery and execution

workflow orchestration

memory storage, retrieval, and governance

artifact creation, verification, and activation

learning signal evaluation and promotion

audit logging and proof capture

execution ordering and turn management

cross‑device session coordination

Only the cloud runtime may finalize or mutate these states.

Hardware integration boundary

Devices expose local hardware capabilities but must treat captured data as untrusted until validated by the cloud runtime.

Typical hardware integrations include:

camera

microphone

file system

printer interfaces

notification systems

All captured artifacts must pass through the canonical runtime execution pipeline before influencing any system state.

Trust boundary model

Client devices must always be treated as untrusted environments. The cloud runtime must validate all device‑originated inputs before they affect authoritative system behavior.

Examples of required validation include:

capture‑bundle attestation before voice or wake runtime reliance

artifact authenticity verification before activation

identity verification before memory access

policy validation before simulation execution

Cloud‑wins rule

If device state and cloud runtime state diverge, the cloud runtime always prevails.

Clients must:

reconcile local state with server responses

discard conflicting cached values

adopt authoritative session and execution state

Deterministic system guarantee

By enforcing this boundary Selene guarantees:

all authoritative decisions originate in the cloud runtime

client devices remain interchangeable interaction terminals

system state remains consistent across devices

execution remains deterministic and auditable

This separation ensures Selene functions as one unified distributed intelligence system rather than multiple partially authoritative clients.

Cross-platform capability parity

Selene must maintain strict functional and behavioral parity across all supported client platforms. The system is designed as a single distributed intelligence runtime accessed through multiple device terminals rather than separate platform-specific products.

Supported platform classes currently include:

iPhone

Android phone

Tablet (target platform class)

Desktop

Although devices differ in hardware capabilities and operating system constraints, the logical behavior of the Selene runtime must remain identical across platforms.

The governing rule is:

Platform differences may affect interaction mechanics, but they must never alter Selene runtime logic.

Runtime invariants

Across all platforms the following architectural properties must remain identical:

cloud‑authoritative execution model

session‑first runtime architecture

canonical ingress contract

session lifecycle semantics

turn ownership and deterministic ordering

identity verification and authorization rules

memory authority and eligibility filtering

synchronization, retry, and deduplication guarantees

learning and update governance

audit and proof capture model

link generation and delivery model

These invariants ensure Selene behaves as one coherent system regardless of which device the user interacts from.

Interaction capability baseline

Each client platform must support a common interaction capability baseline wherever technically feasible. These capabilities include:

voice interaction

text interaction

file and document upload

image upload

camera capture workflows

structured visualization rendering (tables, charts, dashboards)

PDF document generation and viewing

printer workflows when supported by the device

session open and resume

turn submission and response rendering

cloud synchronization

deterministic retry behavior

Devices may implement these capabilities using platform‑specific UI or hardware APIs, but the logical system behavior must remain identical.

Cross‑device continuation guarantee

Users must be able to move between devices without breaking system continuity.

Examples include:

starting a conversation on iPhone and continuing on Desktop

uploading a document on Desktop and reviewing results on mobile

capturing an invoice image on mobile and generating reports later on Desktop

switching between devices while maintaining the same session history

Cross‑device transitions must preserve:

session identifiers

turn ordering

memory context

operation history

execution integrity

Platform constraint exception rule

Certain behavioral differences are permitted when required by operating system restrictions or hardware limitations.

Examples include:

iPhone explicit‑only trigger policy

Android persistent wake listeners

platform‑specific camera APIs

platform‑specific printer interfaces

platform‑specific UI interaction patterns

These differences affect only interaction mechanics and must never alter Selene runtime architecture, execution gates, or authority boundaries.

Capability evolution discipline

When new Selene capabilities are introduced:

all supported platforms must be evaluated for compatibility

capability parity must be maintained whenever technically feasible

platform limitations must be explicitly documented

no platform may introduce alternate runtime logic paths

This rule guarantees Selene evolves as a unified multi‑device system rather than fragmented platform implementations.

Retention, purge, and delete lifecycle

Selene must enforce a governed lifecycle for all retained data so that storage, expiration, purge, and deletion behavior remain deterministic, auditable, and compliant with security and regulatory requirements.

The governing rule is:

No retained data may outlive its policy without justification, and no deletion may occur without an auditable lifecycle record.

Lifecycle governance scope

Retention governance applies to every data class that participates in Selene runtime behavior, including:

wake and voice artifacts

voice enrollment artifacts

identity artifacts

memory records

session audit and proof records

generated user artifacts (documents, reports, links)

learning artifacts and profile updates

client reliability state such as operation journals and retry outbox entries

Each of these classes must have an explicit lifecycle policy enforced by the Selene cloud runtime.

Lifecycle policy model

For every governed data class the system must define the following lifecycle properties:

retention owner

retention basis (policy, regulation, or operational requirement)

retention duration or condition

purge trigger

delete trigger

verification receipt for purge or deletion

policy override path where permitted

These attributes guarantee that lifecycle behavior remains explicit, deterministic, and verifiable.

Retention classes

Selene data may fall into multiple retention classes depending on sensitivity and operational importance. Example classes include:

short‑lived assist state such as retry journals and UI caches

medium‑lived operational records used for synchronization and diagnostics

long‑lived identity or knowledge records such as user memory

compliance‑bound audit records retained under regulated policy

Retention duration must remain cloud‑authoritative and configurable through lifecycle policy controls.

Lifecycle execution model

Retention, purge, and deletion operations must be executed by governed cloud lifecycle workers rather than client devices.

Lifecycle workers are responsible for:

evaluating retention eligibility

applying lifecycle policy validation

executing purge or tombstone operations

generating lifecycle proof records

producing verification receipts when required

Client devices must never perform authoritative deletion of cloud‑owned data.

Audit and proof requirements

Every lifecycle action must generate a verifiable audit record. Proof records should include:

data class affected

identity or scope reference

associated session_id where applicable

reason for purge or deletion

timestamp of lifecycle action

system actor or worker identifier

verification receipt or proof artifact

These records ensure lifecycle transitions remain traceable and reviewable.

Client‑side lifecycle behavior

Client devices may hold temporary local assist data such as:

operation outbox entries

retry journals

recent conversation rendering cache

audio and UI interaction state

Deletion of these local caches must never be interpreted as deletion of authoritative cloud‑owned records.

Architectural gap discipline

If lifecycle workers, purge logic, or verification receipts are not yet implemented for a specific data class, the gap must remain explicitly documented in the Known Architectural Gaps section.

No lifecycle capability may be considered complete until governed workers and verification paths exist.

Known architectural gaps

This section records the architectural gaps that currently exist between the approved Selene runtime architecture and the present implementation state. The purpose of this section is to maintain explicit architectural visibility so that incomplete system areas cannot be mistaken for completed behavior.

The governing rule is:

A gap remains open until code, runtime behavior, tests, and documentation all confirm closure.

Architectural transparency principle

Architectural gaps must remain visible until they are fully resolved. Partial implementations, temporary workarounds, or undocumented assumptions must never be treated as architectural completion.

Every listed gap must therefore include:

clear description of the missing capability

owning engine or subsystem

implementation priority classification

expected architectural outcome once resolved

proof requirements for closure

Core architectural gaps

The following system areas currently remain incomplete relative to the approved architecture:

canonical session identifiers are not yet exposed consistently across all client-visible responses

true cross-device shared session continuity is not yet fully implemented

Android microphone and wake runtime parity remains incomplete relative to Desktop behavior

Tablet remains a TARGET platform class and is not yet fully integrated into runtime contracts

personality behavior remains tone-scoped without long-term adaptive behavioral balancing

retention, purge, and delete lifecycle enforcement is incomplete for several governed data classes

native client application implementations exist outside the primary runtime repository

These gaps must be resolved without introducing alternate runtime logic paths or violating the session-first architecture.

Gap-handling discipline

All architectural gaps must follow a strict handling discipline:

gaps must remain explicitly documented

gaps must have a clearly identifiable owning engine

gaps must be assigned an implementation priority

gaps must remain visible until verified closure exists

Temporary mitigation strategies must never be interpreted as architectural completion.

Priority-classed architectural gaps

P0 — Canonical session contract exposure

The /v1/voice/turn response must consistently expose canonical identifiers required for session-first behavior.

Required identifiers:

session_id

turn_id

session_state

This ensures every client follows an explicit session contract rather than inferring continuity indirectly.

P0 — Platform-trigger policy enforcement in PH1.OS

Platform trigger policy must be enforced centrally by PH1.OS instead of relying solely on adapter-level validation.

This ensures that platform rules such as iPhone explicit-only entry and non-iPhone wake behavior become true orchestration policies.

P0 — Capture-bundle attestation

Client-provided capture metadata must not be trusted without verification.

Capture-bundle attestation must be implemented before PH1.K and PH1.W rely on device-provided capture information.

P0 — Artifact authenticity and trust-root verification

Artifact verification must extend beyond hash validation.

Artifact activation must include authenticity signatures and trust-root verification before runtime acceptance.

P1 — Android microphone and wake runtime parity

Android runtime behavior must reach parity with the Desktop wake and microphone runtime path.

P1 — Wake lifecycle workers

Retention, purge, and delete lifecycle workers must be implemented for wake artifacts together with verification receipts.

P1 — Cross-device session continuity

Current reopen logic resolves sessions primarily by actor plus device scope. True shared session continuity across device classes remains incomplete.

P2 — Tablet runtime contract integration

Tablet must be formally integrated into platform enums, trigger policy enforcement, and runtime contract validation.

Gap closure rule

A gap may only be considered closed when:

an implementation exists in the owning subsystem

runtime behavior matches the architecture defined in this document

verification tests demonstrate correct behavior

documentation and system contracts reflect the resolved state

Until these conditions are satisfied, the gap must remain listed in this section and within the relevant engine ownership documentation.

Governing architecture rule

This document is the parent runtime architecture governing how all Selene clients, sessions, and cloud execution paths must behave. It defines the non‑negotiable architectural laws that ensure Selene remains a deterministic, cloud‑authoritative distributed intelligence system.

The governing principle is:

Architecture defines the laws of the system. Implementations must conform to those laws.

Architectural authority

This document establishes the mandatory system laws governing:

client‑runtime interaction

session‑first execution

cloud authority over system state

cross‑platform capability parity

identity‑scoped memory governance

synchronization and recovery behavior

artifact lifecycle governance

audit and proof capture requirements

All Selene platform implementations, runtime engines, system simulations, and future feature expansions must operate within these constraints.

Parent architecture role

This document sits above all platform‑specific and subsystem documentation. It acts as the parent architectural specification for the Selene client runtime model.

Platform or subsystem documents may extend this architecture to describe:

native UI flows

platform‑specific interaction controls

hardware integration details

operating‑system constraints

implementation patterns

However, such documents must never contradict or weaken the architectural rules defined here.

Non‑negotiable constraints

No subsystem, platform implementation, or future capability may:

violate the session‑first system law

bypass the canonical runtime execution pipeline

introduce alternative authority paths outside the cloud runtime

mutate identity, access, memory, or artifact state locally on client devices

introduce platform‑specific runtime logic forks

If any proposed implementation conflicts with these constraints, the implementation must be revised before development proceeds.

Deterministic system boundary

Selene maintains a strict separation between deterministic system authority and probabilistic reasoning.

Deterministic domains include:

execution gating

identity verification

access authorization

session lifecycle mutation

artifact activation

memory authority

Probabilistic domains (such as language understanding or ranking models) may assist decision‑making but must never become the authority for system state mutation.

Compliance requirement

All engineering work must be evaluated against this architecture for compliance. Before implementing a feature, engineers must confirm that the feature aligns with the rules defined in this document.

If a proposal conflicts with the architecture:

the architecture must be corrected intentionally

or the proposal must be revised

Implementation convenience must never override architectural law.

Architecture stability rule

This document is intended to remain stable over time so that Selene development proceeds on a predictable foundation. Changes to this architecture must therefore be deliberate, reviewed, and justified at the system design level.

No subsystem team, platform team, or implementation effort may silently diverge from the architectural rules defined here.

Engine ownership of current gaps

This section assigns clear ownership of all currently known architectural gaps to the engines responsible for resolving them. The purpose of this mapping is to ensure that no architectural gap exists without an accountable subsystem responsible for its closure.

The governing rule is:

Every architectural gap must have a clearly defined owning engine responsible for design, implementation, testing, and validation of the resolution.

Engine ownership provides the following guarantees:

engineering accountability for unresolved architecture areas

clear implementation responsibility during build runs

traceable alignment between architecture and subsystem implementation

preventing gaps from remaining unresolved due to ambiguous ownership

Each engine listed below must maintain implementation plans and verification tests proving that the gap has been resolved according to the architectural rules defined in this document.

PH1.W — Wake Engine

Responsibilities:

wake detection

wake artifact lifecycle management

wake runtime behavior across supported platforms

Current gaps:

Android wake runtime behavior is not yet fully aligned with Desktop wake behavior.

Wake artifact lifecycle workers (retention, purge, delete) are not fully implemented.

Wake rejection reasoning and runtime diagnostics require expansion.

Wake decision logic must integrate with capture‑bundle attestation once implemented.

PH1.K — Voice Runtime I/O

Responsibilities:

audio capture

audio preprocessing

capture‑bundle generation

voice pipeline integration with PH1.VOICE.ID

Current gaps:

Android microphone runtime parity is incomplete relative to Desktop.

Capture‑bundle attestation must be implemented so client‑provided capture metadata cannot be trusted without verification.

Audio pipeline resilience and runtime diagnostics require additional hardening.

PH1.L — Session Lifecycle Engine

Responsibilities:

session creation

session lifecycle state transitions

turn sequencing

cross‑device session coordination

Current gaps:

Session reopening logic currently resolves primarily using actor + device scope.

True shared cross‑device session continuity is not yet implemented.

Session inactivity confirmation and close behavior require refinement.

Canonical identifier exposure (session_id, turn_id, session_state) remains incomplete in some runtime responses.

PH1.VOICE.ID — Voice Identity Engine

Responsibilities:

voice enrollment

speaker recognition

identity assertion during runtime execution

identity‑scoped memory access gating

Current gaps:

Native client enrollment flows must be implemented in platform applications.

Voice identity artifact validation and lifecycle tooling require further development.

Cross‑device identity continuity must remain strictly cloud‑authoritative.

PH1.EMO / PH1.EMO.CORE — Personality Engine

Responsibilities:

personality classification

communication style guidance

personality artifact storage

Current gaps:

Current implementation affects tone only.

Long‑term adaptive personality balancing models remain unimplemented.

Advanced personality behavior requires further architectural design.

PH1.M — Memory Engine

Responsibilities:

identity‑scoped memory storage

memory retrieval and eligibility filtering

memory ledger persistence

memory lifecycle governance

Current gaps:

Retention and purge lifecycle enforcement is incomplete for some memory classes.

Hot / medium / cold retention policy enforcement requires finalization.

Memory governance tooling must align with artifact lifecycle governance.

PH1.OS — Platform Orchestration Layer

Responsibilities:

platform capability registry

platform trigger policy enforcement

client platform compatibility validation

Current gaps:

Platform trigger policy must be enforced centrally in PH1.OS instead of adapter‑level guards.

Tablet platform modeling must be fully integrated into runtime policy enforcement.

Cross‑device session attachment behavior must remain consistent across runtime paths.

PH1.F — Persistence Foundation

Responsibilities:

persistent storage systems

session state storage

memory ledger storage

artifact persistence

Current gaps:

Lifecycle workers for purge and retention enforcement require further implementation.

Lifecycle proof verification tooling must be expanded.

PH1.J — Audit and Proof Layer

Responsibilities:

audit logging

black‑box proof capture

compliance record generation

Current gaps:

Proof capture must integrate tightly with the canonical runtime execution pipeline.

Session‑bound audit guarantees require strengthening.

Artifact authenticity verification receipts must integrate with the audit system.

Simulation and Link Execution Path

Responsibilities:

simulation dispatch

workflow orchestration

link generation

link delivery through BCAST

Current gaps:

Public client‑facing link generation APIs remain undefined.

Simulation dispatch governance must continue enforcing session‑first execution.

Ownership discipline

For every engine listed above:

an implementation plan must exist

verification tests must confirm architectural compliance

runtime behavior must match the architecture defined in this document

documentation must be updated when the gap is resolved

A gap is considered closed only when implementation, runtime behavior, tests, and documentation all confirm alignment with the parent architecture.

Build-phase transition rule

This section defines how the Selene system transitions from architectural specification into controlled engineering execution. The purpose of the build‑phase transition rule is to ensure that implementation work proceeds in disciplined phases that preserve the architectural laws defined in this document.

The governing rule is:

Architecture alignment must occur before feature expansion.

No system capability may be introduced if the architectural contract it depends on has not been implemented and verified.

Architecture‑first development model

Selene development follows an architecture‑first execution model. In this model:

architecture defines system laws

engines implement those laws

platform clients consume those implementations

feature expansion occurs only after architectural alignment

This ordering prevents architectural drift and ensures that every subsystem evolves within the deterministic runtime model defined earlier in this document.

Phase 0 — Architecture alignment

The first engineering phase must align the current runtime implementation with the architectural rules defined in this document.

Phase‑0 objectives include:

exposing canonical session identifiers in /v1/voice/turn responses

centralizing trigger policy enforcement in PH1.OS

implementing capture‑bundle attestation for client‑originated media

implementing artifact authenticity and trust‑root verification

ensuring the canonical execution gate order is enforced by the runtime

These corrections ensure the runtime obeys the session‑first execution law and the cloud‑authority boundary.

Phase 1 — Runtime parity and lifecycle completion

After architecture alignment, the next phase must complete runtime parity and lifecycle enforcement across engines and platforms.

Phase‑1 objectives include:

Android microphone and wake runtime parity with Desktop

wake artifact retention, purge, and delete lifecycle workers

cross‑device shared session continuity

memory retention and lifecycle enforcement

audit and proof capture integration with execution gates

These improvements ensure Selene behaves as a consistent distributed runtime across devices and sessions.

Phase 2 — Platform expansion

Once runtime parity is verified, the architecture supports controlled platform expansion.

Phase‑2 objectives include:

Tablet runtime contract integration

Tablet trigger policy enforcement through PH1.OS

Tablet client capability parity with other platforms

formal platform enumeration across runtime contracts

This phase ensures new platforms integrate without altering the core architecture.

Structured build runs

After architectural phases are complete, engineering work may proceed in structured build runs. Each run must focus on a defined subsystem scope.

Typical run categories include:

engine capability completion runs

platform client implementation runs

cross‑device synchronization hardening runs

memory governance completion runs

audit and compliance hardening runs

Each run must produce verifiable system improvements before the next run begins.

Verification requirements

A build run may be considered complete only when the following conditions are satisfied:

an implementation exists in the owning subsystem

runtime execution behavior matches the architecture defined in this document

automated tests verify correct behavior

documentation and system contracts reflect the implemented change

Without these conditions, the run must remain open.

Architecture preservation rule

Every build run must preserve the following architectural invariants:

session‑first execution

cloud‑authoritative system state

canonical ingress contract

identity‑scoped memory governance

deterministic synchronization and replay

artifact lifecycle governance

If any build activity violates these invariants, the architecture must be corrected before further development proceeds.

Engineering coordination rule

Codex and engineering teams may propose implementation sequences, but those sequences must remain subordinate to the architectural laws defined in this document.

Architecture defines what must be true.

Implementation plans define how it becomes true.

Completion condition

The architecture alignment phase is considered complete when:

all P0 and P1 architectural gaps listed in this document are closed

all engines execute through the canonical runtime pipeline

all client platforms operate under the same session‑first architecture

Once these conditions are satisfied, Selene may safely expand functionality without risking architectural drift.