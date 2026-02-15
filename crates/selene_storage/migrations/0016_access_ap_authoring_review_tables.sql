-- ACCESS AP authoring review persistence tables.
-- Step-5 storage lock for ACCESS_SCHEMA_MANAGE authoring review flow.

CREATE TABLE IF NOT EXISTS access_ap_authoring_review_ledger (
    review_event_id BIGINT PRIMARY KEY,
    tenant_id TEXT,
    ap_scope TEXT NOT NULL,
    scope_key TEXT NOT NULL,
    access_profile_id TEXT NOT NULL,
    schema_version_id TEXT NOT NULL,
    event_kind TEXT NOT NULL,
    review_channel TEXT,
    confirmation_state TEXT NOT NULL,
    reason_code BIGINT NOT NULL,
    created_by_user_id TEXT NOT NULL REFERENCES identities(user_id),
    created_at BIGINT NOT NULL,
    idempotency_key TEXT NOT NULL,
    CHECK (ap_scope IN ('GLOBAL', 'TENANT')),
    CHECK (
        (ap_scope = 'GLOBAL' AND tenant_id IS NULL)
        OR (ap_scope = 'TENANT' AND tenant_id IS NOT NULL)
    ),
    CHECK (event_kind IN ('REVIEW_CHANNEL_COMMIT', 'CONFIRMATION_COMMIT')),
    CHECK (review_channel IS NULL OR review_channel IN ('PHONE_DESKTOP', 'READ_OUT_LOUD')),
    CHECK (
        confirmation_state IN (
            'NEEDS_CHANNEL_CHOICE',
            'REVIEW_IN_PROGRESS',
            'PENDING_ACTIVATION_CONFIRMATION',
            'CONFIRMED_FOR_ACTIVATION',
            'DECLINED'
        )
    )
);

ALTER TABLE access_ap_schemas_ledger
    ADD COLUMN IF NOT EXISTS activation_review_event_id BIGINT;

ALTER TABLE access_ap_schemas_ledger
    ADD COLUMN IF NOT EXISTS activation_rule_action_count BIGINT;

ALTER TABLE access_ap_schemas_ledger
    ADD COLUMN IF NOT EXISTS activation_rule_action_set_ref TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS ux_access_ap_authoring_review_channel_idem
    ON access_ap_authoring_review_ledger(
        scope_key,
        access_profile_id,
        schema_version_id,
        review_channel,
        idempotency_key
    )
    WHERE review_channel IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS ux_access_ap_authoring_confirm_idem
    ON access_ap_authoring_review_ledger(
        scope_key,
        access_profile_id,
        schema_version_id,
        confirmation_state,
        idempotency_key
    );

CREATE INDEX IF NOT EXISTS ix_access_ap_authoring_review_scope_profile_version
    ON access_ap_authoring_review_ledger(scope_key, access_profile_id, schema_version_id, review_event_id);

CREATE INDEX IF NOT EXISTS ix_access_ap_schemas_activation_review_event
    ON access_ap_schemas_ledger(activation_review_event_id)
    WHERE activation_review_event_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS access_ap_authoring_review_current (
    scope_key TEXT NOT NULL,
    access_profile_id TEXT NOT NULL,
    schema_version_id TEXT NOT NULL,
    latest_review_event_id BIGINT NOT NULL REFERENCES access_ap_authoring_review_ledger(review_event_id),
    review_channel TEXT NOT NULL,
    confirmation_state TEXT NOT NULL,
    updated_at BIGINT NOT NULL,
    reason_code BIGINT NOT NULL,
    PRIMARY KEY (scope_key, access_profile_id, schema_version_id),
    CHECK (review_channel IN ('PHONE_DESKTOP', 'READ_OUT_LOUD')),
    CHECK (
        confirmation_state IN (
            'NEEDS_CHANNEL_CHOICE',
            'REVIEW_IN_PROGRESS',
            'PENDING_ACTIVATION_CONFIRMATION',
            'CONFIRMED_FOR_ACTIVATION',
            'DECLINED'
        )
    )
);

CREATE TABLE IF NOT EXISTS access_ap_rule_review_actions_ledger (
    review_action_row_id BIGINT PRIMARY KEY,
    tenant_id TEXT,
    ap_scope TEXT NOT NULL,
    scope_key TEXT NOT NULL,
    access_profile_id TEXT NOT NULL,
    schema_version_id TEXT NOT NULL,
    rule_action TEXT NOT NULL,
    suggested_rule_ref TEXT,
    capability_id TEXT,
    constraint_ref TEXT,
    escalation_policy_ref TEXT,
    reason_code BIGINT NOT NULL,
    created_by_user_id TEXT NOT NULL REFERENCES identities(user_id),
    created_at BIGINT NOT NULL,
    idempotency_key TEXT NOT NULL,
    CHECK (ap_scope IN ('GLOBAL', 'TENANT')),
    CHECK (
        (ap_scope = 'GLOBAL' AND tenant_id IS NULL)
        OR (ap_scope = 'TENANT' AND tenant_id IS NOT NULL)
    ),
    CHECK (rule_action IN ('AGREE', 'DISAGREE', 'EDIT', 'DELETE', 'DISABLE', 'ADD_CUSTOM_RULE')),
    CHECK (
        (rule_action = 'ADD_CUSTOM_RULE' AND suggested_rule_ref IS NULL AND capability_id IS NOT NULL)
        OR (rule_action IN ('AGREE', 'DISAGREE', 'DELETE', 'DISABLE') AND suggested_rule_ref IS NOT NULL)
        OR (rule_action = 'EDIT' AND suggested_rule_ref IS NOT NULL AND capability_id IS NOT NULL)
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_access_ap_rule_review_action_idem
    ON access_ap_rule_review_actions_ledger(
        scope_key,
        access_profile_id,
        schema_version_id,
        rule_action,
        coalesce(suggested_rule_ref, ''),
        idempotency_key
    );

CREATE INDEX IF NOT EXISTS ix_access_ap_rule_review_action_scope_profile_version
    ON access_ap_rule_review_actions_ledger(scope_key, access_profile_id, schema_version_id, review_action_row_id);
