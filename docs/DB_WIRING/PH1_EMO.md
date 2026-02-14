# PH1.EMO DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.EMO`
- `purpose`: Persist deterministic emotional profile/tone guidance lifecycle with strict tone-only boundaries (never facts, never authority, never execution decisions).
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

### `preferences_ledger` (PH1.EMO namespace)
- `truth_type`: `LEDGER`
- `primary key`: `event_id`
- invariants:
  - PH1.EMO may write only bounded `emo.*` keys:
    - `emo.personality_type`
    - `emo.personality_lock_status`
    - `emo.voice_style_profile`
    - `emo.privacy_state`
    - `emo.snapshot_ref`
  - append-only enforcement applies
  - idempotent dedupe on `(correlation_id, idempotency_key)`

### `preferences_current` (PH1.EMO projection namespace)
- `truth_type`: `CURRENT`
- `primary key`: `(tenant_id, user_id, key)`
- invariants:
  - deterministic rebuild from `preferences_ledger` events for `emo.*` keys
  - tone-only values only; no factual business authority fields

### `artifacts_ledger`
- `truth_type`: `LEDGER`
- `primary key`: `artifact_id`
- invariants:
  - onboarding emotional snapshot artifacts are stored as bounded refs only
  - no raw transcript dumps
  - idempotent dedupe on `(scope_type, scope_id, artifact_type, artifact_version, idempotency_key)`

### `os_core.audit_events`
- `truth_type`: `LEDGER`
- `primary key`: `event_id`
- invariants:
  - PH1.EMO emits reason-coded audit proof rows for all commit paths
  - append-only and bounded payload discipline

## 3) Reads (dependencies)

- identity/session scope checks:
  - reads: `identities`, `devices`, `sessions`
  - required checks:
    - `device_id` must belong to `user_id`
    - `session_id` (if provided) must match `(user_id, device_id)`
- emotional context source:
  - reads: `preferences_current` (`emo.*` keys), `memory_current` (signal references only)
- tenant isolation:
  - one deterministic tenant binding per emotional write path

## 4) Writes (outputs)

All PH1.EMO side effects are simulation-gated (`No Simulation -> No Execution`).

### `EMO_SIM_001` Classify Profile
- writes: `preferences_ledger`, `preferences_current`, `audit_events`
- idempotency: `(user_id + session_id + idempotency_key)`

### `EMO_SIM_002` Re-evaluate Profile
- writes: `preferences_ledger`, `preferences_current`, `audit_events`
- idempotency: `(user_id + idempotency_key)`

### `EMO_SIM_003` Apply Privacy Command
- writes: `preferences_ledger`, `preferences_current`, `audit_events`
- idempotency: `(user_id + privacy_command + idempotency_key)`

### `EMO_SIM_004` Emit Tone Guidance (Draft)
- writes: none (output-only draft path)
- hard rule: emitted tone guidance is non-authoritative and cannot alter facts/actions

### `EMO_SIM_005` Snapshot Capture
- writes: `artifacts_ledger`, `preferences_ledger`, `preferences_current`, `audit_events`
- idempotency: `(onboarding_session_id + idempotency_key)`

### `EMO_SIM_006` Emit Audit Event
- writes: `audit_events`
- idempotency: `(user_id + session_id + idempotency_key)`

## 5) Relations & Keys

- `audit_events.user_id -> identities.user_id`
- `audit_events.device_id -> devices.device_id`
- `audit_events.session_id -> sessions.session_id` (optional)
- PH1.EMO must not mutate non-`emo.*` preference keys.

## 6) Hard Rules (Tone-Only)

- PH1.EMO outputs may influence tone/pacing/empathy only.
- PH1.EMO must never inject or mutate factual claims, permissions, or execution decisions.
- PH1.EMO must never bypass Access, Simulation, or orchestration controls.

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-EMO-DB-01` tenant isolation enforced for emotional writes
- `AT-EMO-DB-02` append-only enforcement for emotional ledger paths
- `AT-EMO-DB-03` idempotency dedupe works for profile/privacy/snapshot writes
- `AT-EMO-DB-04` tone-only boundary enforced (no factual/authority fields in `emo.*`)
- `AT-EMO-DB-05` `EMO_SIM_004` remains output-only with zero DB side effects

Implementation references:
- design-only in this phase
