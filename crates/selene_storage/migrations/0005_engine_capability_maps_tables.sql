-- Engine Capability Maps tables (`engine_capability_maps` ledger + current projection).

CREATE TABLE IF NOT EXISTS engine_capability_maps (
    engine_capability_map_event_id BIGSERIAL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    engine_id TEXT NOT NULL,
    capability_id TEXT NOT NULL,
    capability_map_version INTEGER NOT NULL,
    map_status TEXT NOT NULL,
    owning_domain TEXT NOT NULL,
    capability_name TEXT NOT NULL,
    allowed_callers TEXT NOT NULL,
    side_effects_mode TEXT NOT NULL,
    reads_tables_hash TEXT NOT NULL,
    writes_tables_hash TEXT NOT NULL,
    reason_code BIGINT NOT NULL,
    created_at BIGINT NOT NULL,
    idempotency_key TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_engine_capability_maps_idempotency
    ON engine_capability_maps(tenant_id, engine_id, capability_id, capability_map_version, idempotency_key)
    WHERE idempotency_key IS NOT NULL;

CREATE INDEX IF NOT EXISTS ix_engine_capability_maps_tenant_engine_cap_event
    ON engine_capability_maps(tenant_id, engine_id, capability_id, engine_capability_map_event_id);

CREATE TABLE IF NOT EXISTS engine_capability_maps_current (
    tenant_id TEXT NOT NULL,
    engine_id TEXT NOT NULL,
    capability_id TEXT NOT NULL,
    capability_map_version INTEGER NOT NULL,
    map_status TEXT NOT NULL,
    owning_domain TEXT NOT NULL,
    capability_name TEXT NOT NULL,
    allowed_callers TEXT NOT NULL,
    side_effects_mode TEXT NOT NULL,
    source_event_id BIGINT NOT NULL REFERENCES engine_capability_maps(engine_capability_map_event_id),
    updated_at BIGINT NOT NULL,
    PRIMARY KEY (tenant_id, engine_id, capability_id)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_engine_capability_maps_current_tenant_engine_cap_version
    ON engine_capability_maps_current(tenant_id, engine_id, capability_id, capability_map_version);
