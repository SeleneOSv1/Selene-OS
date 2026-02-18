-- Builder Selene Phase 13-C persistence tables.
-- Append-only approval-state + release-stage truth.

CREATE TABLE IF NOT EXISTS builder_approval_states (
    approval_row_id BIGINT PRIMARY KEY,
    schema_version BIGINT NOT NULL DEFAULT 1,
    approval_state_id TEXT NOT NULL UNIQUE,
    proposal_id TEXT NOT NULL REFERENCES builder_patch_proposals(proposal_id),
    change_class TEXT NOT NULL,
    required_approvals_total BIGINT NOT NULL,
    approvals_granted BIGINT NOT NULL,
    tech_approved BOOLEAN NOT NULL,
    product_security_approved BOOLEAN NOT NULL,
    status TEXT NOT NULL,
    reason_code BIGINT NOT NULL,
    recorded_at BIGINT NOT NULL,
    resolved_at BIGINT,
    idempotency_key TEXT,
    CHECK (change_class IN ('CLASS_A', 'CLASS_B', 'CLASS_C')),
    CHECK (status IN ('PENDING', 'APPROVED', 'REJECTED')),
    CHECK (required_approvals_total >= 0 AND required_approvals_total <= 2),
    CHECK (approvals_granted >= 0 AND approvals_granted <= required_approvals_total),
    CHECK (reason_code > 0),
    CHECK (resolved_at IS NULL OR resolved_at >= recorded_at),
    CHECK (
        (change_class = 'CLASS_A' AND required_approvals_total = 0 AND tech_approved = FALSE AND product_security_approved = FALSE)
        OR (change_class = 'CLASS_B' AND required_approvals_total = 1 AND product_security_approved = FALSE)
        OR (change_class = 'CLASS_C' AND required_approvals_total = 2)
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_builder_approval_states_idempotency
    ON builder_approval_states(proposal_id, idempotency_key)
    WHERE idempotency_key IS NOT NULL;

CREATE TABLE IF NOT EXISTS builder_release_states (
    release_row_id BIGINT PRIMARY KEY,
    schema_version BIGINT NOT NULL DEFAULT 1,
    release_state_id TEXT NOT NULL UNIQUE,
    proposal_id TEXT NOT NULL REFERENCES builder_patch_proposals(proposal_id),
    stage TEXT NOT NULL,
    stage_rollout_pct BIGINT NOT NULL,
    status TEXT NOT NULL,
    rollback_hook TEXT NOT NULL,
    rollback_ready BOOLEAN NOT NULL,
    reason_code BIGINT NOT NULL,
    recorded_at BIGINT NOT NULL,
    idempotency_key TEXT,
    CHECK (
        stage IN ('STAGING', 'CANARY', 'RAMP_25', 'RAMP_50', 'PRODUCTION', 'ROLLED_BACK')
    ),
    CHECK (status IN ('PENDING', 'ACTIVE', 'BLOCKED', 'COMPLETED', 'REVERTED')),
    CHECK (stage_rollout_pct IN (0, 5, 25, 50, 100)),
    CHECK (reason_code > 0),
    CHECK (
        (stage = 'STAGING' AND stage_rollout_pct = 0)
        OR (stage = 'CANARY' AND stage_rollout_pct = 5)
        OR (stage = 'RAMP_25' AND stage_rollout_pct = 25)
        OR (stage = 'RAMP_50' AND stage_rollout_pct = 50)
        OR (stage = 'PRODUCTION' AND stage_rollout_pct = 100)
        OR (stage = 'ROLLED_BACK' AND stage_rollout_pct = 0)
    ),
    CHECK ((stage = 'PRODUCTION') = (status = 'COMPLETED')),
    CHECK ((stage = 'ROLLED_BACK') = (status = 'REVERTED'))
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_builder_release_states_idempotency
    ON builder_release_states(proposal_id, idempotency_key)
    WHERE idempotency_key IS NOT NULL;
