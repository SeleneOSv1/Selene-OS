-- Builder Selene Phase 13-A persistence tables.
-- Append-only proposal/run/result truth with deterministic idempotency keys.

CREATE TABLE IF NOT EXISTS builder_patch_proposals (
    proposal_row_id BIGINT PRIMARY KEY,
    schema_version BIGINT NOT NULL DEFAULT 1,
    proposal_id TEXT NOT NULL UNIQUE,
    created_at BIGINT NOT NULL,
    source_signal_window_start_at BIGINT NOT NULL,
    source_signal_window_end_at BIGINT NOT NULL,
    source_signal_count BIGINT NOT NULL,
    source_signal_hash TEXT NOT NULL,
    target_files_json TEXT NOT NULL,
    change_class TEXT NOT NULL,
    risk_score_bp BIGINT NOT NULL,
    expected_effect_json TEXT NOT NULL,
    validation_plan TEXT NOT NULL,
    rollback_plan TEXT NOT NULL,
    status TEXT NOT NULL,
    idempotency_key TEXT,
    CHECK (source_signal_window_end_at >= source_signal_window_start_at),
    CHECK (source_signal_count > 0),
    CHECK (change_class IN ('CLASS_A', 'CLASS_B', 'CLASS_C')),
    CHECK (risk_score_bp >= 0 AND risk_score_bp <= 10000),
    CHECK (status IN ('DRAFT', 'VALIDATED', 'APPROVED', 'RELEASED', 'REVERTED'))
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_builder_patch_proposals_idempotency
    ON builder_patch_proposals(source_signal_hash, idempotency_key)
    WHERE idempotency_key IS NOT NULL;

CREATE TABLE IF NOT EXISTS builder_validation_runs (
    run_row_id BIGINT PRIMARY KEY,
    schema_version BIGINT NOT NULL DEFAULT 1,
    run_id TEXT NOT NULL UNIQUE,
    proposal_id TEXT NOT NULL REFERENCES builder_patch_proposals(proposal_id),
    started_at BIGINT NOT NULL,
    finished_at BIGINT,
    status TEXT NOT NULL,
    gate_count_expected BIGINT NOT NULL,
    gate_count_recorded BIGINT NOT NULL,
    idempotency_key TEXT,
    CHECK (status IN ('RUNNING', 'PASSED', 'FAILED')),
    CHECK (gate_count_expected >= 1 AND gate_count_expected <= 10),
    CHECK (gate_count_recorded >= 0 AND gate_count_recorded <= gate_count_expected),
    CHECK (finished_at IS NULL OR finished_at >= started_at)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_builder_validation_runs_idempotency
    ON builder_validation_runs(proposal_id, idempotency_key)
    WHERE idempotency_key IS NOT NULL;

CREATE TABLE IF NOT EXISTS builder_validation_gate_results (
    gate_result_row_id BIGINT PRIMARY KEY,
    schema_version BIGINT NOT NULL DEFAULT 1,
    run_id TEXT NOT NULL REFERENCES builder_validation_runs(run_id),
    proposal_id TEXT NOT NULL REFERENCES builder_patch_proposals(proposal_id),
    gate_id TEXT NOT NULL,
    passed BOOLEAN NOT NULL,
    recorded_at BIGINT NOT NULL,
    reason_code BIGINT NOT NULL,
    detail TEXT NOT NULL,
    idempotency_key TEXT,
    CHECK (
        gate_id IN (
            'BLD-G1',
            'BLD-G2',
            'BLD-G3',
            'BLD-G4',
            'BLD-G5',
            'BLD-G6',
            'BLD-G7',
            'BLD-G8',
            'BLD-G9',
            'BLD-G10'
        )
    ),
    CHECK (reason_code > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_builder_validation_gate_results_run_gate
    ON builder_validation_gate_results(run_id, gate_id);

CREATE UNIQUE INDEX IF NOT EXISTS ux_builder_validation_gate_results_idempotency
    ON builder_validation_gate_results(run_id, gate_id, idempotency_key)
    WHERE idempotency_key IS NOT NULL;
