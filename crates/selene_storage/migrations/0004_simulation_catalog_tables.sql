-- Simulation Catalog tables (`simulation_catalog` ledger + current projection).

CREATE TABLE IF NOT EXISTS simulation_catalog (
    simulation_catalog_event_id BIGSERIAL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    simulation_id TEXT NOT NULL,
    simulation_version INTEGER NOT NULL,
    simulation_type TEXT NOT NULL,
    status TEXT NOT NULL,
    owning_domain TEXT NOT NULL,
    reads_tables_hash TEXT NOT NULL,
    writes_tables_hash TEXT NOT NULL,
    reason_code BIGINT NOT NULL,
    created_at BIGINT NOT NULL,
    idempotency_key TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_simulation_catalog_idempotency
    ON simulation_catalog(tenant_id, simulation_id, simulation_version, idempotency_key)
    WHERE idempotency_key IS NOT NULL;

CREATE INDEX IF NOT EXISTS ix_simulation_catalog_tenant_simulation_event
    ON simulation_catalog(tenant_id, simulation_id, simulation_catalog_event_id);

CREATE TABLE IF NOT EXISTS simulation_catalog_current (
    tenant_id TEXT NOT NULL,
    simulation_id TEXT NOT NULL,
    simulation_version INTEGER NOT NULL,
    simulation_type TEXT NOT NULL,
    status TEXT NOT NULL,
    owning_domain TEXT NOT NULL,
    source_event_id BIGINT NOT NULL REFERENCES simulation_catalog(simulation_catalog_event_id),
    updated_at BIGINT NOT NULL,
    PRIMARY KEY (tenant_id, simulation_id)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_simulation_catalog_current_tenant_simulation_version
    ON simulation_catalog_current(tenant_id, simulation_id, simulation_version);
