-- Builder Selene Phase 13-D persistence table.
-- Append-only post-deploy judge results with deterministic rollback/accept decisions.

CREATE TABLE IF NOT EXISTS builder_post_deploy_judge_results (
    judge_row_id BIGINT PRIMARY KEY,
    schema_version BIGINT NOT NULL DEFAULT 1,
    judge_result_id TEXT NOT NULL UNIQUE,
    proposal_id TEXT NOT NULL REFERENCES builder_patch_proposals(proposal_id),
    release_state_id TEXT NOT NULL REFERENCES builder_release_states(release_state_id),
    before_latency_p95_ms BIGINT NOT NULL,
    before_latency_p99_ms BIGINT NOT NULL,
    before_fail_closed_rate_bp BIGINT NOT NULL,
    before_critical_reason_spike_bp BIGINT NOT NULL,
    before_observation_window_minutes BIGINT NOT NULL,
    after_latency_p95_ms BIGINT NOT NULL,
    after_latency_p99_ms BIGINT NOT NULL,
    after_fail_closed_rate_bp BIGINT NOT NULL,
    after_critical_reason_spike_bp BIGINT NOT NULL,
    after_observation_window_minutes BIGINT NOT NULL,
    action TEXT NOT NULL,
    reason_code BIGINT NOT NULL,
    recorded_at BIGINT NOT NULL,
    idempotency_key TEXT,
    CHECK (action IN ('ACCEPT', 'REVERT')),
    CHECK (reason_code > 0),
    CHECK (before_latency_p95_ms > 0 AND before_latency_p99_ms > 0),
    CHECK (after_latency_p95_ms > 0 AND after_latency_p99_ms > 0),
    CHECK (before_fail_closed_rate_bp >= 0 AND before_fail_closed_rate_bp <= 10000),
    CHECK (after_fail_closed_rate_bp >= 0 AND after_fail_closed_rate_bp <= 10000),
    CHECK (before_critical_reason_spike_bp >= -10000 AND before_critical_reason_spike_bp <= 10000),
    CHECK (after_critical_reason_spike_bp >= -10000 AND after_critical_reason_spike_bp <= 10000),
    CHECK (before_observation_window_minutes >= 1 AND before_observation_window_minutes <= 1440),
    CHECK (after_observation_window_minutes >= 1 AND after_observation_window_minutes <= 1440)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_builder_post_deploy_judge_results_idempotency
    ON builder_post_deploy_judge_results(proposal_id, idempotency_key)
    WHERE idempotency_key IS NOT NULL;
