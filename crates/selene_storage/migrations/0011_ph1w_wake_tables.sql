-- PH1.W wake enrollment/runtime persistence tables.
-- NOTE: `onboarding_session_id` FK is enforced in PH1.F storage wiring for this slice.
-- A physical DB FK is added when PH1.ONB tables are locked in sequence.

CREATE TABLE IF NOT EXISTS wake_enrollment_sessions (
    wake_enrollment_session_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES identities(user_id),
    device_id TEXT NOT NULL REFERENCES devices(device_id),
    onboarding_session_id TEXT,
    wake_enroll_status TEXT NOT NULL,
    pass_target SMALLINT NOT NULL,
    pass_count SMALLINT NOT NULL,
    attempt_count SMALLINT NOT NULL,
    max_attempts SMALLINT NOT NULL,
    enrollment_timeout_ms INTEGER NOT NULL,
    reason_code BIGINT,
    wake_profile_id TEXT,
    deferred_until BIGINT,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    completed_at BIGINT,
    CHECK (wake_enroll_status IN ('IN_PROGRESS', 'PENDING', 'COMPLETE', 'DECLINED')),
    CHECK (pass_target BETWEEN 3 AND 8),
    CHECK (max_attempts BETWEEN 8 AND 20),
    CHECK (pass_count >= 0),
    CHECK (attempt_count >= 0),
    CHECK (attempt_count <= max_attempts),
    CHECK (enrollment_timeout_ms BETWEEN 180000 AND 600000)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_wake_enrollment_sessions_active_user_device
    ON wake_enrollment_sessions(user_id, device_id)
    WHERE wake_enroll_status = 'IN_PROGRESS';

CREATE INDEX IF NOT EXISTS ix_wake_enrollment_sessions_device_status_updated
    ON wake_enrollment_sessions(device_id, wake_enroll_status, updated_at);

CREATE TABLE IF NOT EXISTS wake_enrollment_samples (
    sample_id BIGSERIAL PRIMARY KEY,
    wake_enrollment_session_id TEXT NOT NULL REFERENCES wake_enrollment_sessions(wake_enrollment_session_id),
    sample_seq SMALLINT NOT NULL,
    captured_at BIGINT NOT NULL,
    sample_duration_ms SMALLINT NOT NULL,
    vad_coverage DOUBLE PRECISION NOT NULL,
    snr_db DOUBLE PRECISION NOT NULL,
    clipping_pct DOUBLE PRECISION NOT NULL,
    rms_dbfs DOUBLE PRECISION NOT NULL,
    noise_floor_dbfs DOUBLE PRECISION NOT NULL,
    peak_dbfs DOUBLE PRECISION NOT NULL,
    overlap_ratio DOUBLE PRECISION NOT NULL,
    result TEXT NOT NULL,
    reason_code BIGINT,
    idempotency_key TEXT NOT NULL,
    CHECK (sample_seq > 0),
    CHECK (sample_duration_ms BETWEEN 500 AND 2200),
    CHECK (result IN ('PASS', 'FAIL'))
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_wake_enrollment_samples_session_idempotency
    ON wake_enrollment_samples(wake_enrollment_session_id, idempotency_key);

CREATE UNIQUE INDEX IF NOT EXISTS ux_wake_enrollment_samples_session_seq
    ON wake_enrollment_samples(wake_enrollment_session_id, sample_seq);

CREATE TABLE IF NOT EXISTS wake_runtime_events (
    wake_event_id TEXT PRIMARY KEY,
    session_id BIGINT REFERENCES sessions(session_id),
    user_id TEXT REFERENCES identities(user_id),
    device_id TEXT NOT NULL REFERENCES devices(device_id),
    created_at BIGINT NOT NULL,
    accepted BOOLEAN NOT NULL,
    reason_code BIGINT NOT NULL,
    wake_profile_id TEXT,
    tts_active_at_trigger BOOLEAN NOT NULL,
    media_playback_active_at_trigger BOOLEAN NOT NULL,
    suppression_reason_code BIGINT,
    idempotency_key TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_wake_runtime_events_device_idempotency
    ON wake_runtime_events(device_id, idempotency_key);

CREATE INDEX IF NOT EXISTS ix_wake_runtime_events_device_created
    ON wake_runtime_events(device_id, created_at);

CREATE TABLE IF NOT EXISTS wake_profile_bindings (
    user_id TEXT NOT NULL REFERENCES identities(user_id),
    device_id TEXT NOT NULL REFERENCES devices(device_id),
    wake_profile_id TEXT NOT NULL,
    artifact_version TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    PRIMARY KEY (user_id, device_id, wake_profile_id),
    CHECK (char_length(trim(wake_profile_id)) > 0),
    CHECK (char_length(trim(artifact_version)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_wake_profile_bindings_active_user_device
    ON wake_profile_bindings(user_id, device_id)
    WHERE active = TRUE;

