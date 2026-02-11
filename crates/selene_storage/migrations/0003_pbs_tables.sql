-- PBS tables: process blueprints (append-only events) + blueprint registry (current projection).

CREATE TABLE IF NOT EXISTS process_blueprints (
    process_blueprint_event_id BIGSERIAL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    process_id TEXT NOT NULL,
    blueprint_version INTEGER NOT NULL,
    intent_type TEXT NOT NULL,
    status TEXT NOT NULL,
    ordered_step_count INTEGER NOT NULL,
    confirmation_step_count INTEGER NOT NULL,
    simulation_requirements_hash TEXT NOT NULL,
    reason_code BIGINT NOT NULL,
    created_at BIGINT NOT NULL,
    idempotency_key TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_process_blueprints_idempotency
    ON process_blueprints(tenant_id, process_id, blueprint_version, idempotency_key)
    WHERE idempotency_key IS NOT NULL;

CREATE INDEX IF NOT EXISTS ix_process_blueprints_tenant_intent_event
    ON process_blueprints(tenant_id, intent_type, process_blueprint_event_id);

CREATE TABLE IF NOT EXISTS blueprint_registry (
    tenant_id TEXT NOT NULL,
    intent_type TEXT NOT NULL,
    process_id TEXT NOT NULL,
    blueprint_version INTEGER NOT NULL,
    status TEXT NOT NULL,
    source_event_id BIGINT NOT NULL REFERENCES process_blueprints(process_blueprint_event_id),
    updated_at BIGINT NOT NULL,
    PRIMARY KEY (tenant_id, intent_type)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_blueprint_registry_tenant_process_version
    ON blueprint_registry(tenant_id, process_id, blueprint_version);
