-- Selene OS core WorkOrder persistence tables (design-lock row 3).

CREATE TABLE IF NOT EXISTS work_order_ledger (
    work_order_event_id BIGSERIAL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    work_order_id TEXT NOT NULL,
    correlation_id NUMERIC(39,0) NOT NULL,
    turn_id BIGINT NOT NULL,
    work_order_status TEXT NOT NULL,
    reason_code BIGINT NOT NULL,
    step_id TEXT,
    step_input_hash TEXT,
    lease_owner_id TEXT,
    lease_token_hash TEXT,
    lease_expires_at BIGINT,
    created_at BIGINT NOT NULL,
    idempotency_key TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_work_order_ledger_tenant_work_order_event
    ON work_order_ledger(tenant_id, work_order_id, work_order_event_id);

CREATE UNIQUE INDEX IF NOT EXISTS ux_work_order_ledger_idempotency
    ON work_order_ledger(tenant_id, work_order_id, idempotency_key)
    WHERE idempotency_key IS NOT NULL;

CREATE TABLE IF NOT EXISTS work_orders_current (
    work_order_id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    correlation_id NUMERIC(39,0) NOT NULL,
    turn_id BIGINT NOT NULL,
    work_order_status TEXT NOT NULL,
    last_event_id BIGINT NOT NULL REFERENCES work_order_ledger(work_order_event_id),
    last_reason_code BIGINT NOT NULL,
    last_updated_at BIGINT NOT NULL,
    step_id TEXT,
    step_input_hash TEXT,
    lease_owner_id TEXT,
    lease_token_hash TEXT,
    lease_expires_at BIGINT
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_work_orders_current_tenant_work_order
    ON work_orders_current(tenant_id, work_order_id);

CREATE UNIQUE INDEX IF NOT EXISTS ux_work_orders_current_tenant_correlation
    ON work_orders_current(tenant_id, correlation_id);

CREATE TABLE IF NOT EXISTS work_order_step_attempts (
    step_attempt_id BIGSERIAL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    work_order_id TEXT NOT NULL REFERENCES work_orders_current(work_order_id),
    step_id TEXT NOT NULL,
    attempt_index INTEGER NOT NULL,
    attempt_status TEXT NOT NULL,
    reason_code BIGINT,
    started_at BIGINT NOT NULL,
    finished_at BIGINT,
    idempotency_key TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_work_order_step_attempts_attempt_index
    ON work_order_step_attempts(tenant_id, work_order_id, step_id, attempt_index);

CREATE UNIQUE INDEX IF NOT EXISTS ux_work_order_step_attempts_idempotency
    ON work_order_step_attempts(tenant_id, work_order_id, step_id, idempotency_key);

CREATE TABLE IF NOT EXISTS work_order_leases (
    lease_id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    work_order_id TEXT NOT NULL REFERENCES work_orders_current(work_order_id),
    lease_owner_id TEXT NOT NULL,
    lease_token TEXT NOT NULL,
    lease_state TEXT NOT NULL,
    lease_expires_at BIGINT NOT NULL,
    acquired_at BIGINT NOT NULL,
    released_at BIGINT,
    idempotency_key TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_work_order_leases_tenant_lease_id
    ON work_order_leases(tenant_id, lease_id);

CREATE UNIQUE INDEX IF NOT EXISTS ux_work_order_leases_tenant_lease_token
    ON work_order_leases(tenant_id, lease_token);

CREATE UNIQUE INDEX IF NOT EXISTS ux_work_order_leases_tenant_work_order_idempotency
    ON work_order_leases(tenant_id, work_order_id, idempotency_key)
    WHERE idempotency_key IS NOT NULL;
