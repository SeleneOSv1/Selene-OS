-- Access master-schema registry + board voting tables.
-- Contract alignment target: docs/04_KERNEL_CONTRACTS.md (KC.26).

ALTER TABLE access_instances ADD COLUMN compiled_global_profile_id TEXT;
ALTER TABLE access_instances ADD COLUMN compiled_global_profile_version TEXT;
ALTER TABLE access_instances ADD COLUMN compiled_tenant_profile_id TEXT;
ALTER TABLE access_instances ADD COLUMN compiled_tenant_profile_version TEXT;
ALTER TABLE access_instances ADD COLUMN compiled_overlay_set_ref TEXT;
ALTER TABLE access_instances ADD COLUMN compiled_position_id TEXT;

CREATE TABLE IF NOT EXISTS access_ap_schemas_ledger (
    event_id BIGSERIAL PRIMARY KEY,
    tenant_id TEXT,
    scope_key TEXT NOT NULL,
    access_profile_id TEXT NOT NULL,
    schema_version_id TEXT NOT NULL,
    ap_scope TEXT NOT NULL,
    event_action TEXT NOT NULL,
    lifecycle_state TEXT NOT NULL,
    profile_payload_json JSONB NOT NULL,
    reason_code BIGINT NOT NULL,
    created_by_user_id TEXT NOT NULL REFERENCES identities(user_id),
    created_at BIGINT NOT NULL,
    idempotency_key TEXT NOT NULL,
    CHECK (ap_scope IN ('GLOBAL', 'TENANT')),
    CHECK (event_action IN ('CREATE_DRAFT', 'UPDATE_DRAFT', 'ACTIVATE', 'RETIRE')),
    CHECK (lifecycle_state IN ('DRAFT', 'ACTIVE', 'RETIRED')),
    CHECK (
        (ap_scope = 'GLOBAL' AND tenant_id IS NULL)
        OR (ap_scope = 'TENANT' AND tenant_id IS NOT NULL)
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_access_ap_schemas_ledger_scope_profile_version_action_idem
    ON access_ap_schemas_ledger(scope_key, access_profile_id, schema_version_id, event_action, idempotency_key);

CREATE INDEX IF NOT EXISTS ix_access_ap_schemas_ledger_scope_profile_event
    ON access_ap_schemas_ledger(scope_key, access_profile_id, event_id);

CREATE TABLE IF NOT EXISTS access_ap_schemas_current (
    scope_key TEXT NOT NULL,
    access_profile_id TEXT NOT NULL,
    active_schema_version_id TEXT NOT NULL,
    active_event_id BIGINT NOT NULL REFERENCES access_ap_schemas_ledger(event_id),
    updated_at BIGINT NOT NULL,
    reason_code BIGINT NOT NULL,
    PRIMARY KEY (scope_key, access_profile_id)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_access_ap_schemas_current_scope_profile
    ON access_ap_schemas_current(scope_key, access_profile_id);

CREATE TABLE IF NOT EXISTS access_ap_overlay_ledger (
    event_id BIGSERIAL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    overlay_id TEXT NOT NULL,
    overlay_version_id TEXT NOT NULL,
    event_action TEXT NOT NULL,
    lifecycle_state TEXT NOT NULL,
    overlay_ops_json JSONB NOT NULL,
    reason_code BIGINT NOT NULL,
    created_by_user_id TEXT NOT NULL REFERENCES identities(user_id),
    created_at BIGINT NOT NULL,
    idempotency_key TEXT NOT NULL,
    CHECK (event_action IN ('CREATE_DRAFT', 'UPDATE_DRAFT', 'ACTIVATE', 'RETIRE')),
    CHECK (lifecycle_state IN ('DRAFT', 'ACTIVE', 'RETIRED'))
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_access_ap_overlay_ledger_tenant_overlay_version_action_idem
    ON access_ap_overlay_ledger(tenant_id, overlay_id, overlay_version_id, event_action, idempotency_key);

CREATE INDEX IF NOT EXISTS ix_access_ap_overlay_ledger_tenant_overlay_event
    ON access_ap_overlay_ledger(tenant_id, overlay_id, event_id);

CREATE TABLE IF NOT EXISTS access_ap_overlay_current (
    tenant_id TEXT NOT NULL,
    overlay_id TEXT NOT NULL,
    active_overlay_version_id TEXT NOT NULL,
    active_event_id BIGINT NOT NULL REFERENCES access_ap_overlay_ledger(event_id),
    updated_at BIGINT NOT NULL,
    reason_code BIGINT NOT NULL,
    PRIMARY KEY (tenant_id, overlay_id)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_access_ap_overlay_current_tenant_overlay
    ON access_ap_overlay_current(tenant_id, overlay_id);

CREATE TABLE IF NOT EXISTS access_board_policy_ledger (
    event_id BIGSERIAL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    board_policy_id TEXT NOT NULL,
    policy_version_id TEXT NOT NULL,
    event_action TEXT NOT NULL,
    lifecycle_state TEXT NOT NULL,
    policy_payload_json JSONB NOT NULL,
    reason_code BIGINT NOT NULL,
    created_by_user_id TEXT NOT NULL REFERENCES identities(user_id),
    created_at BIGINT NOT NULL,
    idempotency_key TEXT NOT NULL,
    CHECK (event_action IN ('CREATE_DRAFT', 'UPDATE_DRAFT', 'ACTIVATE', 'RETIRE')),
    CHECK (lifecycle_state IN ('DRAFT', 'ACTIVE', 'RETIRED'))
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_access_board_policy_ledger_tenant_policy_version_action_idem
    ON access_board_policy_ledger(tenant_id, board_policy_id, policy_version_id, event_action, idempotency_key);

CREATE INDEX IF NOT EXISTS ix_access_board_policy_ledger_tenant_policy_event
    ON access_board_policy_ledger(tenant_id, board_policy_id, event_id);

CREATE TABLE IF NOT EXISTS access_board_policy_current (
    tenant_id TEXT NOT NULL,
    board_policy_id TEXT NOT NULL,
    active_policy_version_id TEXT NOT NULL,
    active_event_id BIGINT NOT NULL REFERENCES access_board_policy_ledger(event_id),
    updated_at BIGINT NOT NULL,
    reason_code BIGINT NOT NULL,
    PRIMARY KEY (tenant_id, board_policy_id)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_access_board_policy_current_tenant_policy
    ON access_board_policy_current(tenant_id, board_policy_id);

CREATE TABLE IF NOT EXISTS access_board_votes_ledger (
    vote_row_id BIGSERIAL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    escalation_case_id TEXT NOT NULL,
    board_policy_id TEXT NOT NULL,
    voter_user_id TEXT NOT NULL REFERENCES identities(user_id),
    vote_value TEXT NOT NULL,
    reason_code BIGINT NOT NULL,
    created_at BIGINT NOT NULL,
    idempotency_key TEXT NOT NULL,
    CHECK (vote_value IN ('APPROVE', 'REJECT')),
    FOREIGN KEY (tenant_id, board_policy_id)
        REFERENCES access_board_policy_current(tenant_id, board_policy_id)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_access_board_votes_tenant_case_voter_idem
    ON access_board_votes_ledger(tenant_id, escalation_case_id, voter_user_id, idempotency_key);

CREATE INDEX IF NOT EXISTS ix_access_board_votes_tenant_case
    ON access_board_votes_ledger(tenant_id, escalation_case_id);
