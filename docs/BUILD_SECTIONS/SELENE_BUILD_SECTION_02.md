Selene Build Section 02

Session Engine (PH1.L)

Purpose

Implement the Selene Session Engine responsible for managing the lifecycle, identity context, and execution container used for all Selene interactions. This engine is the operational realization of SELENE_CORE_CONTRACT_01_SESSION.

The Session Engine ensures that every interaction with Selene occurs within a cloud-governed session context and that session state remains deterministic, auditable, and consistent across devices.

Implements

SELENE_CORE_CONTRACT_01_SESSION

This contract defines the canonical session behavior for Selene.

Core Responsibilities

The Session Engine must implement the following behaviors so that Selene remains a session-first, cloud-authoritative runtime.

Session Authority Boundary

The Session Engine must preserve the core Selene law that sessions are owned by the cloud runtime, not by client devices.

Responsibilities include:

ensuring all authoritative session state exists in the cloud runtime

preventing clients from inventing or mutating session state locally

ensuring devices only open, resume, observe, and synchronize with sessions

preserving the session as the primary execution container for runtime behavior

This ensures Selene remains session-first and cloud-authoritative from the beginning.

Session Resolution Authority

The Session Engine must be the only runtime subsystem allowed to:

create session containers

assign session_id values

assign turn_id values

change session lifecycle state

resolve session reopen versus new-session decisions

close, suspend, or recover sessions

This prevents parallel session authority paths from appearing elsewhere in the runtime.

Session as Execution Container

The Session Engine must treat a session as the canonical execution container for Selene interactions.

A session coordinates:

conversation lifecycle

turn sequencing

identity scope

access policy

memory eligibility

execution ordering

cross-device continuity

A session may reference these governed domains, but it must not itself absorb their business logic. The Session Engine remains execution-agnostic and lifecycle-focused.

The Session Engine must implement the following behaviors:

Session States

Support the canonical session lifecycle states:

Closed

Open

Active

SoftClosed

Suspended

The engine must enforce deterministic state transitions as defined by the Session Contract.

Session Identifiers

The engine must generate and manage canonical identifiers:

session_id

turn_id

These identifiers must be globally unique and must be exposed to all downstream execution and response layers.

session_id identifies the authoritative conversation container.

turn_id identifies a single execution unit within the session.

Lifecycle Transitions

The engine must enforce the canonical transition rules:

Closed → Open

Open → Active

Active → Active

Active → SoftClosed

SoftClosed → Active

SoftClosed → Closed

Any state → Suspended

The engine must also support the Open bypass rule where Closed → Active may occur when the first turn is executed immediately after session creation.

The default architectural target for inactivity handling is approximately 30 seconds before transition toward SoftClosed, while remaining policy-configurable.

Suspension and Recovery Conditions

The Session Engine must support controlled suspension when runtime conditions make normal session progression unsafe.

Example suspension causes include:

degraded capture conditions

policy intervention

runtime protection behavior

session-owner uncertainty during recovery

A suspended session must remain explicitly visible as suspended until it is either safely recovered or closed.

Cross-Device Attachment

Multiple devices belonging to the same identity may attach to the same active session.

The Session Engine must support:

session discovery

session attach

session state retrieval

cross-device session synchronization

canonical session continuity across device changes

Devices must never mutate session state directly. All mutations must occur through the Session Engine.

This ensures that a user may move between devices without losing the authoritative conversation container.

Session Resume Policy

The Session Engine must distinguish between:

resume of an existing recoverable session

creation of a new session container

Responsibilities include:

reusing the canonical session when the recoverable window remains valid

creating a new session when recovery is no longer lawful or safe

making the resume-versus-new decision inside the cloud runtime only

This keeps session continuity explicit rather than inferred by clients.

Single-Writer Execution Rule

The Session Engine must enforce that only one execution path may mutate session state per turn.

This includes:

serializing concurrent turn submissions

ensuring deterministic turn ordering

preventing duplicate execution through idempotency enforcement

maintaining authoritative turn sequencing

Device Timeline Tracking

To ensure deterministic multi-device ordering, the Session Engine must maintain a device timeline model for every active session.

Each participating device must provide the following fields in the Runtime Execution Envelope:

device_id

device_turn_sequence

The Session Engine must maintain a device timeline map:

device_timeline_map
{
  device_id -> highest_seen_sequence
}

Runtime behavior must follow these rules:

New Turn

If device_turn_sequence is greater than the stored sequence for that device, the request represents a new turn and may proceed through the execution pipeline.

Retry

If device_turn_sequence equals the stored sequence, the request is a retry and the cached result must be returned.

Stale Message

If device_turn_sequence is less than the stored sequence, the request is stale and must be rejected.

Responsibilities include:

maintaining per-device monotonic turn counters

preventing stale device turns from mutating session state

allowing safe retries across network interruptions

preserving device-specific execution ordering

supporting deterministic multi-device debugging and replay

This mechanism ensures that Selene sessions remain safe and deterministic even when multiple devices interact with the same session concurrently.

Session Exposure

Every session-bound response must expose:

session_id

turn_id

session_state

This guarantees correct client synchronization, deterministic replay, audit traceability, and explicit client visibility of session state.

The exposed session identifiers act as the synchronization anchor for all participating devices and downstream runtime layers.

Session Partitioning Model

The Session Engine must support horizontal scaling through deterministic session partitioning.

Responsibilities include:

session_id hash partitioning

routing sessions to an owning runtime node

preserving session locality for active execution

supporting session migration when required

ensuring that a single partition owns session mutation authority

Session Lease Model

To guarantee the single-writer rule in distributed environments, the Session Engine must implement a lease model.

Responsibilities include:

assigning an owning node for a session

lease duration management

lease renewal during active interaction

lease transfer when execution moves to another node

recovery from stale or expired leases

This prevents split-brain session mutation across runtime nodes.

Session Ownership Transfer Protocol

The Session Engine must support a deterministic ownership handoff protocol when a session moves between runtime nodes.

Responsibilities include:

pre-transfer validation

draining in-flight turn execution before transfer

lease handoff acknowledgement

post-transfer verification

safe rejection of partial or ambiguous transfers

This ensures session ownership never becomes duplicated or uncertain during migration.

Session Snapshot and Recovery

The Session Engine must support snapshotting and recovery so that runtime failures or restarts do not lose session state.

Responsibilities include:

session snapshot persistence

crash recovery

restart recovery

rebuilding session state from snapshot data

snapshot compaction to limit storage growth

Session Event Stream

Session state transitions must produce structured events describing lifecycle changes.

Typical events include:

SessionOpened

SessionActivated

TurnStarted

TurnCompleted

SessionSoftClosed

SessionClosed

SessionSuspended

The event stream provides auditability, debugging insight, and potential replay capability.

Session Security Audit Events

The Session Engine must also emit security-relevant events when session integrity or access safety is challenged.

Typical events include:

SessionLeaseConflictDetected

StaleDeviceTurnRejected

UnauthorizedAttachRejected

SessionRecoveryTriggered

These events improve incident response and security monitoring.

Session Time Governance

The Session Engine must enforce time-based lifecycle policies.

Responsibilities include:

inactivity timers

maximum session duration limits

soft close timing policies

background expiry workers

session TTL enforcement

These policies prevent runaway sessions and ensure resource stability.

Session Observability Hooks

The Session Engine must emit structured metrics and telemetry describing session behavior.

Example metrics include:

active_sessions

session_duration

turns_per_session

session_reopens

suspended_sessions

cross_device_attaches

These metrics allow operational monitoring of session health and runtime behavior.

Session Consistency Levels

The Session Engine must support explicit consistency levels so operators and downstream systems know the current session safety posture.

Example levels include:

STRICT

LEASED_DISTRIBUTED

DEGRADED_RECOVERY

These consistency levels make it clear when a session is running under normal conditions versus controlled recovery conditions.

Session Access Classes

The Session Engine must support controlled attach roles for devices joining a session.

Example access classes include:

PRIMARY_INTERACTOR

SECONDARY_VIEWER

LIMITED_ATTACH

RECOVERY_ATTACH

These classes allow the system to distinguish between devices that may actively submit turns and devices that may only observe or recover session state.

Session Causality Chain

The Session Engine must track causal relationships between turns and session events.

Responsibilities include:

parent_turn_id references

causal_predecessor tracking

session_event_correlation_id generation

This allows the runtime to reconstruct not only order, but also why one event or turn followed another.

Session Conflict Resolution Policy

The Session Engine must define explicit resolution rules for concurrent or conflicting session interactions.

Responsibilities include:

simultaneous turn submission resolution

stale attach rejection

conflicting resume attempt handling

duplicate device-claim handling

These rules remove ambiguity during concurrency spikes and network instability.

Session Integrity Checks

The Session Engine must validate its own internal consistency continuously.

Responsibilities include:

invalid lifecycle transition detection

missing session-owner detection

impossible device timeline detection

corrupted snapshot-state detection

Execution must fail safe if session integrity is compromised.

Session Certification Targets

The Session Engine must expose certification goals that can later be validated by the Runtime Governance Layer.

Example certification targets include:

session-first compliance

single-writer compliance

lease safety compliance

device timeline ordering compliance

snapshot recovery compliance

These targets make session correctness measurable rather than assumed.

Session Backpressure Awareness

The Session Engine must detect and respond when an individual session becomes overloaded.

Responsibilities include:

maximum pending turn thresholds per session

overflow handling policy

session throttling or defer behavior

temporary rejection of excessive turn bursts

This prevents a single broken or abusive client from destabilizing a session.



Restrictions

During Build Section 02 the following subsystems must NOT yet be implemented:

memory engine

identity verification

simulation execution

authority enforcement

learning systems

artifact generation

These components will integrate later but must not be embedded inside the Session Engine.

Completion Criteria

Build Section 02 is complete when:

Session containers can be created and closed.

Canonical identifiers are generated.

Session lifecycle transitions are enforced.

Multiple devices can attach to the same session.

Concurrent requests are serialized.

Deterministic turn ordering is maintained.

Device timeline tracking is enforced.

Session ownership transfer works safely.

Session snapshots support crash and restart recovery.

Session consistency levels are exposed.

Session access classes are enforced.

Session conflict resolution rules are defined.

Session integrity checks are active.

Session certification targets are defined.

Session backpressure protections exist.

Session identifiers are exposed to the runtime response layer.

The Session Engine must remain execution-agnostic and must never contain business logic, simulation logic, authorization logic, or memory logic. It acts purely as the cloud-authoritative session lifecycle authority.
