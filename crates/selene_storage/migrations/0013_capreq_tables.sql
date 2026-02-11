-- Capability Request tables (`capreq_ledger` ledger + current projection).

CREATE TABLE IF NOT EXISTS capreq_ledger (
    capreq_event_id BIGSERIAL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    capreq_id TEXT NOT NULL,
    requester_user_id TEXT NOT NULL,
    action TEXT NOT NULL,
    status TEXT NOT NULL,
    reason_code BIGINT NOT NULL,
    payload_hash TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    idempotency_key TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_capreq_ledger_idempotency
    ON capreq_ledger(tenant_id, capreq_id, idempotency_key)
    WHERE idempotency_key IS NOT NULL;

CREATE INDEX IF NOT EXISTS ix_capreq_ledger_tenant_capreq_event
    ON capreq_ledger(tenant_id, capreq_id, capreq_event_id);

CREATE TABLE IF NOT EXISTS capreq_current (
    tenant_id TEXT NOT NULL,
    capreq_id TEXT NOT NULL,
    requester_user_id TEXT NOT NULL,
    status TEXT NOT NULL,
    last_action TEXT NOT NULL,
    payload_hash TEXT NOT NULL,
    source_event_id BIGINT NOT NULL REFERENCES capreq_ledger(capreq_event_id),
    updated_at BIGINT NOT NULL,
    last_reason_code BIGINT NOT NULL,
    PRIMARY KEY (tenant_id, capreq_id)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_capreq_current_tenant_capreq_source_event
    ON capreq_current(tenant_id, capreq_id, source_event_id);
