-- Artifacts ledger + tool cache tables.

CREATE TABLE IF NOT EXISTS artifacts_ledger (
    artifact_id BIGSERIAL PRIMARY KEY,
    scope_type TEXT NOT NULL,
    scope_id TEXT NOT NULL,
    artifact_type TEXT NOT NULL,
    artifact_version INTEGER NOT NULL,
    package_hash TEXT NOT NULL,
    payload_ref TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    created_by TEXT NOT NULL,
    provenance_ref TEXT NOT NULL,
    status TEXT NOT NULL,
    idempotency_key TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_artifacts_ledger_scope_type_scope_id_type_version
    ON artifacts_ledger(scope_type, scope_id, artifact_type, artifact_version);

CREATE UNIQUE INDEX IF NOT EXISTS ux_artifacts_ledger_idempotency
    ON artifacts_ledger(scope_type, scope_id, artifact_type, artifact_version, idempotency_key)
    WHERE idempotency_key IS NOT NULL;

CREATE INDEX IF NOT EXISTS ix_artifacts_ledger_scope_type_scope_id_type_artifact_id
    ON artifacts_ledger(scope_type, scope_id, artifact_type, artifact_id);

CREATE TABLE IF NOT EXISTS tool_cache (
    cache_id BIGSERIAL PRIMARY KEY,
    tool_name TEXT NOT NULL,
    query_hash TEXT NOT NULL,
    locale TEXT NOT NULL,
    result_payload JSONB NOT NULL,
    expires_at BIGINT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_tool_cache_tool_query_locale
    ON tool_cache(tool_name, query_hash, locale);

CREATE INDEX IF NOT EXISTS ix_tool_cache_expires_at
    ON tool_cache(expires_at);
