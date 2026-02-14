-- PH1.LINK / KC.16 onboarding draft + token + dedupe baseline tables.
-- NOTE: runtime wiring remains in-memory for current MVP slices; this migration
-- locks authoritative SQL contracts for DB ownership alignment.

CREATE TABLE IF NOT EXISTS onboarding_drafts (
    draft_id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    invitee_type TEXT NOT NULL,
    schema_version_id TEXT,
    creator_user_id TEXT NOT NULL REFERENCES identities(user_id),
    draft_payload_json JSONB NOT NULL,
    missing_required_fields_json JSONB NOT NULL,
    status TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    committed_entity_id TEXT,
    idempotency_key TEXT,
    CHECK (invitee_type IN ('COMPANY','CUSTOMER','EMPLOYEE','FAMILY_MEMBER','FRIEND','ASSOCIATE')),
    -- Kernel rule: SQL CHECK enforces status enum membership only; monotonic transition enforcement is runtime-owned by PH1.F/PH1.LINK.
    CHECK (status IN ('DRAFT_CREATED', 'DRAFT_READY', 'COMMITTED', 'REVOKED', 'EXPIRED')),
    -- Kernel rule: schema_version_id is required for EMPLOYEE and COMPANY drafts.
    CHECK (
        (invitee_type IN ('EMPLOYEE','COMPANY') AND schema_version_id IS NOT NULL)
        OR (invitee_type NOT IN ('EMPLOYEE','COMPANY'))
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_onboarding_drafts_tenant_draft
    ON onboarding_drafts(tenant_id, draft_id);

CREATE UNIQUE INDEX IF NOT EXISTS ux_onboarding_drafts_idempotency
    ON onboarding_drafts(tenant_id, idempotency_key)
    WHERE idempotency_key IS NOT NULL;

CREATE TABLE IF NOT EXISTS onboarding_link_tokens (
    token_id TEXT PRIMARY KEY,
    draft_id TEXT NOT NULL REFERENCES onboarding_drafts(draft_id),
    tenant_id TEXT NOT NULL,
    token_signature TEXT NOT NULL,
    expires_at BIGINT NOT NULL,
    status TEXT NOT NULL,
    bound_device_fingerprint_hash TEXT,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    consumed_at BIGINT,
    revoked_at BIGINT,
    CHECK (
        status IN (
            'DRAFT_CREATED',
            'SENT',
            'OPENED',
            'ACTIVATED',
            'CONSUMED',
            'REVOKED',
            'EXPIRED',
            'BLOCKED'
        )
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_onboarding_link_tokens_token_tenant
    ON onboarding_link_tokens(token_id, tenant_id);

CREATE INDEX IF NOT EXISTS ix_onboarding_link_tokens_draft_status
    ON onboarding_link_tokens(draft_id, status);

CREATE TABLE IF NOT EXISTS onboarding_draft_write_dedupe (
    dedupe_id BIGSERIAL PRIMARY KEY,
    scope_type TEXT NOT NULL,
    scope_id TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    write_hash TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    CHECK (scope_type IN ('LINK', 'ONB'))
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_onboarding_draft_write_dedupe_scope_key
    ON onboarding_draft_write_dedupe(scope_type, scope_id, idempotency_key);
