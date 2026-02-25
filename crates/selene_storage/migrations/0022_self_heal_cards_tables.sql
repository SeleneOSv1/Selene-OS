-- 14.7.6 self-healing card chain persistence tables.
-- Scope: append-only rows + idempotency for FailureEvent, ProblemCard, FixCard, PromotionDecision.

CREATE TABLE IF NOT EXISTS self_heal_failure_events (
    row_id BIGSERIAL PRIMARY KEY,
    failure_id TEXT NOT NULL,
    tenant_id TEXT NOT NULL,
    correlation_id TEXT NOT NULL,
    turn_id BIGINT NOT NULL,
    idempotency_key TEXT NOT NULL,
    payload_json JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (failure_id),
    UNIQUE (failure_id, idempotency_key),
    CHECK (char_length(trim(failure_id)) > 0),
    CHECK (char_length(trim(tenant_id)) > 0),
    CHECK (char_length(trim(correlation_id)) > 0),
    CHECK (turn_id > 0),
    CHECK (char_length(trim(idempotency_key)) > 0)
);

CREATE INDEX IF NOT EXISTS ix_self_heal_failure_events_tenant
    ON self_heal_failure_events(tenant_id);

CREATE TABLE IF NOT EXISTS self_heal_problem_cards (
    row_id BIGSERIAL PRIMARY KEY,
    problem_id TEXT NOT NULL,
    tenant_id TEXT NOT NULL,
    latest_failure_id TEXT NOT NULL REFERENCES self_heal_failure_events(failure_id),
    idempotency_key TEXT NOT NULL,
    payload_json JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (problem_id),
    UNIQUE (problem_id, idempotency_key),
    CHECK (char_length(trim(problem_id)) > 0),
    CHECK (char_length(trim(tenant_id)) > 0),
    CHECK (char_length(trim(latest_failure_id)) > 0),
    CHECK (char_length(trim(idempotency_key)) > 0)
);

CREATE INDEX IF NOT EXISTS ix_self_heal_problem_cards_tenant
    ON self_heal_problem_cards(tenant_id);

CREATE TABLE IF NOT EXISTS self_heal_fix_cards (
    row_id BIGSERIAL PRIMARY KEY,
    fix_id TEXT NOT NULL,
    problem_id TEXT NOT NULL REFERENCES self_heal_problem_cards(problem_id),
    idempotency_key TEXT NOT NULL,
    payload_json JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (fix_id),
    UNIQUE (fix_id, idempotency_key),
    CHECK (char_length(trim(fix_id)) > 0),
    CHECK (char_length(trim(problem_id)) > 0),
    CHECK (char_length(trim(idempotency_key)) > 0)
);

CREATE TABLE IF NOT EXISTS self_heal_promotion_decisions (
    row_id BIGSERIAL PRIMARY KEY,
    decision_id TEXT NOT NULL,
    tenant_id TEXT NOT NULL,
    fix_id TEXT NOT NULL REFERENCES self_heal_fix_cards(fix_id),
    idempotency_key TEXT NOT NULL,
    payload_json JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (decision_id),
    UNIQUE (decision_id, idempotency_key),
    CHECK (char_length(trim(decision_id)) > 0),
    CHECK (char_length(trim(tenant_id)) > 0),
    CHECK (char_length(trim(fix_id)) > 0),
    CHECK (char_length(trim(idempotency_key)) > 0)
);

CREATE INDEX IF NOT EXISTS ix_self_heal_promotion_decisions_tenant
    ON self_heal_promotion_decisions(tenant_id);
