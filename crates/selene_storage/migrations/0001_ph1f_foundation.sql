-- PH1.F foundation migration (design-lock baseline)
-- Scope: identities/devices/sessions + memory/conversation/audit ledgers.
-- Note: this file is a schema baseline artifact for DB wiring lock; runtime currently uses in-memory store.

CREATE TABLE IF NOT EXISTS identities (
    user_id TEXT PRIMARY KEY,
    speaker_id TEXT,
    primary_language TEXT,
    created_at BIGINT NOT NULL,
    status TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS devices (
    device_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES identities(user_id),
    device_type TEXT NOT NULL,
    last_seen_at BIGINT NOT NULL,
    audio_profile_ref TEXT
);

CREATE TABLE IF NOT EXISTS sessions (
    session_id BIGINT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES identities(user_id),
    device_id TEXT NOT NULL REFERENCES devices(device_id),
    session_state TEXT NOT NULL,
    opened_at BIGINT NOT NULL,
    last_activity_at BIGINT NOT NULL,
    closed_at BIGINT
);

CREATE TABLE IF NOT EXISTS memory_ledger (
    ledger_id BIGSERIAL PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES identities(user_id),
    event JSONB NOT NULL,
    idempotency_key TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_memory_ledger_user_idempotency
    ON memory_ledger(user_id, idempotency_key)
    WHERE idempotency_key IS NOT NULL;

CREATE TABLE IF NOT EXISTS memory_current (
    user_id TEXT NOT NULL REFERENCES identities(user_id),
    memory_key TEXT NOT NULL,
    memory_value JSONB,
    confidence TEXT NOT NULL,
    sensitivity_flag TEXT NOT NULL,
    last_seen_at BIGINT NOT NULL,
    active BOOLEAN NOT NULL,
    use_policy TEXT NOT NULL,
    expires_at BIGINT,
    provenance JSONB NOT NULL,
    PRIMARY KEY (user_id, memory_key)
);

CREATE TABLE IF NOT EXISTS conversation_ledger (
    conversation_turn_id BIGSERIAL PRIMARY KEY,
    correlation_id NUMERIC(39,0) NOT NULL,
    turn_id BIGINT NOT NULL,
    session_id BIGINT REFERENCES sessions(session_id),
    user_id TEXT NOT NULL REFERENCES identities(user_id),
    device_id TEXT REFERENCES devices(device_id),
    role TEXT NOT NULL,
    source TEXT NOT NULL,
    text TEXT NOT NULL,
    text_hash TEXT NOT NULL,
    privacy_scope TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    idempotency_key TEXT,
    tombstone_of_conversation_turn_id BIGINT,
    tombstone_reason_code BIGINT,
    UNIQUE (correlation_id, turn_id)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_conversation_corr_idempotency
    ON conversation_ledger(correlation_id, idempotency_key)
    WHERE idempotency_key IS NOT NULL;

CREATE TABLE IF NOT EXISTS audit_events (
    event_id BIGSERIAL PRIMARY KEY,
    created_at BIGINT NOT NULL,
    tenant_id TEXT,
    work_order_id TEXT,
    session_id BIGINT,
    user_id TEXT,
    device_id TEXT,
    engine TEXT NOT NULL,
    event_type TEXT NOT NULL,
    reason_code BIGINT NOT NULL,
    severity TEXT NOT NULL,
    correlation_id NUMERIC(39,0) NOT NULL,
    turn_id BIGINT NOT NULL,
    payload_min JSONB NOT NULL,
    evidence_ref JSONB,
    idempotency_key TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_audit_corr_idempotency
    ON audit_events(correlation_id, idempotency_key)
    WHERE idempotency_key IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS ux_audit_tenant_work_order_idempotency
    ON audit_events(tenant_id, work_order_id, idempotency_key)
    WHERE tenant_id IS NOT NULL
      AND work_order_id IS NOT NULL
      AND idempotency_key IS NOT NULL;
