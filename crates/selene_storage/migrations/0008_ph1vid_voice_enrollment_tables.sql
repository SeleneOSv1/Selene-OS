-- PH1.VOICE.ID enrollment + profile persistence tables.
-- NOTE: `onboarding_session_id` FK is enforced in PH1.F storage wiring for this slice.
-- A physical DB FK is added when PH1.ONB tables are locked in sequence.

CREATE TABLE IF NOT EXISTS voice_enrollment_sessions (
    voice_enrollment_session_id TEXT PRIMARY KEY,
    onboarding_session_id TEXT NOT NULL,
    device_id TEXT NOT NULL REFERENCES devices(device_id),
    voice_profile_id TEXT,
    voice_enroll_status TEXT NOT NULL,
    lock_after_consecutive_passes SMALLINT NOT NULL,
    consecutive_passes SMALLINT NOT NULL,
    attempt_count SMALLINT NOT NULL,
    max_total_attempts SMALLINT NOT NULL,
    max_session_enroll_time_ms INTEGER NOT NULL,
    reason_code BIGINT,
    deferred_until BIGINT,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    CHECK (voice_enroll_status IN ('IN_PROGRESS', 'LOCKED', 'PENDING', 'DECLINED')),
    CHECK (lock_after_consecutive_passes BETWEEN 2 AND 5),
    CHECK (max_total_attempts BETWEEN 5 AND 20),
    CHECK (attempt_count >= 0),
    CHECK (consecutive_passes >= 0),
    CHECK (max_session_enroll_time_ms BETWEEN 60000 AND 300000)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_voice_enrollment_sessions_onb_device
    ON voice_enrollment_sessions(onboarding_session_id, device_id);

CREATE INDEX IF NOT EXISTS ix_voice_enrollment_sessions_device_status
    ON voice_enrollment_sessions(device_id, voice_enroll_status, updated_at);

CREATE TABLE IF NOT EXISTS voice_enrollment_samples (
    sample_id BIGSERIAL PRIMARY KEY,
    voice_enrollment_session_id TEXT NOT NULL REFERENCES voice_enrollment_sessions(voice_enrollment_session_id),
    sample_seq SMALLINT NOT NULL,
    created_at BIGINT NOT NULL,
    attempt_index SMALLINT NOT NULL,
    audio_sample_ref TEXT NOT NULL,
    result TEXT NOT NULL,
    reason_code BIGINT,
    idempotency_key TEXT NOT NULL,
    CHECK (sample_seq > 0),
    CHECK (attempt_index > 0),
    CHECK (result IN ('PASS', 'FAIL'))
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_voice_enrollment_samples_session_idempotency
    ON voice_enrollment_samples(voice_enrollment_session_id, idempotency_key);

CREATE INDEX IF NOT EXISTS ix_voice_enrollment_samples_session_seq
    ON voice_enrollment_samples(voice_enrollment_session_id, sample_seq);

CREATE TABLE IF NOT EXISTS voice_profiles (
    voice_profile_id TEXT PRIMARY KEY,
    onboarding_session_id TEXT NOT NULL,
    device_id TEXT NOT NULL REFERENCES devices(device_id),
    created_at BIGINT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_voice_profiles_onb_device
    ON voice_profiles(onboarding_session_id, device_id);

CREATE TABLE IF NOT EXISTS voice_profile_bindings (
    onboarding_session_id TEXT NOT NULL,
    device_id TEXT NOT NULL REFERENCES devices(device_id),
    voice_profile_id TEXT NOT NULL REFERENCES voice_profiles(voice_profile_id),
    created_at BIGINT NOT NULL,
    PRIMARY KEY (onboarding_session_id, device_id)
);
