-- PH1.ACCESS.001 + PH2.ACCESS.002 per-user access truth tables.
-- PH1.ACCESS.001 is gate/read-only; PH2.ACCESS.002 owns writes.

CREATE TABLE IF NOT EXISTS access_instances (
    access_instance_id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    user_id TEXT NOT NULL REFERENCES identities(user_id),
    role_template_id TEXT NOT NULL,
    effective_access_mode TEXT NOT NULL,
    baseline_permissions_json JSONB NOT NULL,
    identity_verified BOOLEAN NOT NULL,
    verification_level TEXT NOT NULL,
    device_trust_level TEXT NOT NULL,
    lifecycle_state TEXT NOT NULL,
    policy_snapshot_ref TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    idempotency_key TEXT,
    CHECK (effective_access_mode IN ('R', 'W', 'A', 'X')),
    CHECK (verification_level IN ('NONE', 'PASSCODE_TIME', 'BIOMETRIC', 'STEP_UP')),
    CHECK (device_trust_level IN ('DTL1', 'DTL2', 'DTL3', 'DTL4')),
    CHECK (lifecycle_state IN ('RESTRICTED', 'ACTIVE', 'SUSPENDED'))
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_access_instances_tenant_user
    ON access_instances(tenant_id, user_id);

CREATE UNIQUE INDEX IF NOT EXISTS ux_access_instances_tenant_instance
    ON access_instances(tenant_id, access_instance_id);

CREATE UNIQUE INDEX IF NOT EXISTS ux_access_instances_tenant_user_idempotency
    ON access_instances(tenant_id, user_id, idempotency_key)
    WHERE idempotency_key IS NOT NULL;

CREATE TABLE IF NOT EXISTS access_overrides (
    override_id TEXT PRIMARY KEY,
    access_instance_id TEXT NOT NULL REFERENCES access_instances(access_instance_id),
    tenant_id TEXT NOT NULL,
    override_type TEXT NOT NULL,
    scope_json JSONB NOT NULL,
    status TEXT NOT NULL,
    approved_by_user_id TEXT NOT NULL REFERENCES identities(user_id),
    approved_via_simulation_id TEXT NOT NULL,
    reason_code BIGINT NOT NULL,
    starts_at BIGINT NOT NULL,
    expires_at BIGINT,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    idempotency_key TEXT NOT NULL,
    CHECK (override_type IN ('ONE_SHOT', 'TEMPORARY', 'PERMANENT', 'REVOKE')),
    CHECK (status IN ('ACTIVE', 'EXPIRED', 'REVOKED')),
    CHECK (expires_at IS NULL OR expires_at > starts_at)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_access_overrides_tenant_override
    ON access_overrides(tenant_id, override_id);

CREATE UNIQUE INDEX IF NOT EXISTS ux_access_overrides_tenant_instance_idempotency
    ON access_overrides(tenant_id, access_instance_id, idempotency_key);

CREATE INDEX IF NOT EXISTS ix_access_overrides_instance_status_window
    ON access_overrides(access_instance_id, status, starts_at, expires_at);

-- Contract guard: block duplicate ACTIVE scope rows per instance.
CREATE UNIQUE INDEX IF NOT EXISTS ux_access_overrides_active_scope
    ON access_overrides(access_instance_id, scope_json)
    WHERE status = 'ACTIVE';
