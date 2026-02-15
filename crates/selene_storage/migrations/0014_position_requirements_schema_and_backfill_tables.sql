-- Position requirements schema + onboarding backfill baseline tables.
-- Contract alignment target: docs/04_KERNEL_CONTRACTS.md (KC.25).

CREATE TABLE IF NOT EXISTS position_requirements_schema_ledger (
    schema_event_id BIGSERIAL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    company_id TEXT NOT NULL,
    position_id TEXT NOT NULL,
    schema_version_id TEXT NOT NULL,
    action TEXT NOT NULL,
    selector_snapshot_json JSONB NOT NULL,
    field_specs_json JSONB NOT NULL,
    overlay_ops_json JSONB,
    change_reason TEXT,
    apply_scope TEXT,
    reason_code BIGINT NOT NULL,
    actor_user_id TEXT NOT NULL REFERENCES identities(user_id),
    created_at BIGINT NOT NULL,
    idempotency_key TEXT,
    CHECK (
        action IN (
            'CREATE_DRAFT',
            'UPDATE_COMMIT',
            'ACTIVATE_COMMIT',
            'RETIRE_COMMIT'
        )
    ),
    CHECK (
        change_reason IS NULL
        OR (length(trim(change_reason)) > 0 AND length(change_reason) <= 256)
    ),
    CHECK (
        (action = 'UPDATE_COMMIT' AND change_reason IS NOT NULL)
        OR (action <> 'UPDATE_COMMIT' AND change_reason IS NULL)
    ),
    CHECK (
        (action = 'ACTIVATE_COMMIT' AND apply_scope IN ('NEW_HIRES_ONLY', 'CURRENT_AND_NEW'))
        OR (action <> 'ACTIVATE_COMMIT' AND apply_scope IS NULL)
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_position_requirements_schema_ledger_scope_idempotency
    ON position_requirements_schema_ledger(
        tenant_id,
        position_id,
        schema_version_id,
        action,
        idempotency_key
    )
    WHERE idempotency_key IS NOT NULL;

CREATE INDEX IF NOT EXISTS ix_position_requirements_schema_ledger_tenant_position_event
    ON position_requirements_schema_ledger(tenant_id, position_id, schema_event_id);

CREATE TABLE IF NOT EXISTS position_requirements_schema_current (
    tenant_id TEXT NOT NULL,
    company_id TEXT NOT NULL,
    position_id TEXT NOT NULL,
    active_schema_version_id TEXT NOT NULL,
    active_selector_snapshot_json JSONB NOT NULL,
    active_field_specs_json JSONB NOT NULL,
    source_event_id BIGINT NOT NULL REFERENCES position_requirements_schema_ledger(schema_event_id),
    updated_at BIGINT NOT NULL,
    last_reason_code BIGINT NOT NULL,
    PRIMARY KEY (tenant_id, position_id)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_position_requirements_schema_current_source
    ON position_requirements_schema_current(tenant_id, position_id, source_event_id);

CREATE TABLE IF NOT EXISTS onboarding_requirement_backfill_campaigns (
    campaign_id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    company_id TEXT NOT NULL,
    position_id TEXT NOT NULL,
    schema_version_id TEXT NOT NULL,
    rollout_scope TEXT NOT NULL,
    state TEXT NOT NULL,
    created_by_user_id TEXT NOT NULL REFERENCES identities(user_id),
    reason_code BIGINT NOT NULL,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    completed_at BIGINT,
    idempotency_key TEXT,
    CHECK (rollout_scope IN ('NEW_HIRES_ONLY', 'CURRENT_AND_NEW')),
    CHECK (state IN ('DRAFT_CREATED', 'RUNNING', 'COMPLETED', 'CANCELED'))
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_onboarding_requirement_backfill_campaigns_tenant_campaign
    ON onboarding_requirement_backfill_campaigns(tenant_id, campaign_id);

CREATE UNIQUE INDEX IF NOT EXISTS ux_onboarding_requirement_backfill_campaigns_scope_idempotency
    ON onboarding_requirement_backfill_campaigns(
        tenant_id,
        position_id,
        schema_version_id,
        idempotency_key
    )
    WHERE idempotency_key IS NOT NULL;

CREATE TABLE IF NOT EXISTS onboarding_requirement_backfill_targets (
    target_row_id BIGSERIAL PRIMARY KEY,
    campaign_id TEXT NOT NULL REFERENCES onboarding_requirement_backfill_campaigns(campaign_id),
    tenant_id TEXT NOT NULL,
    user_id TEXT NOT NULL REFERENCES identities(user_id),
    status TEXT NOT NULL,
    missing_fields_json JSONB NOT NULL,
    last_reason_code BIGINT NOT NULL,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    completed_at BIGINT,
    idempotency_key TEXT,
    CHECK (
        status IN (
            'PENDING',
            'REQUESTED',
            'REMINDED',
            'COMPLETED',
            'EXEMPTED',
            'FAILED'
        )
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_onboarding_requirement_backfill_targets_campaign_user
    ON onboarding_requirement_backfill_targets(campaign_id, user_id);

CREATE UNIQUE INDEX IF NOT EXISTS ux_onboarding_requirement_backfill_targets_tenant_target
    ON onboarding_requirement_backfill_targets(tenant_id, target_row_id);

CREATE UNIQUE INDEX IF NOT EXISTS ux_onboarding_requirement_backfill_targets_campaign_user_idem
    ON onboarding_requirement_backfill_targets(campaign_id, user_id, idempotency_key)
    WHERE idempotency_key IS NOT NULL;
