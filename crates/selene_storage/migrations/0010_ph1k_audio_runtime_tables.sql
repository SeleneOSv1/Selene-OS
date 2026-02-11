-- PH1.K voice runtime I/O persistence tables.
-- Scope: append-only runtime event ledger + rebuildable current projection.

CREATE TABLE IF NOT EXISTS audio_runtime_events (
    event_id BIGSERIAL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    device_id TEXT NOT NULL REFERENCES devices(device_id),
    session_id BIGINT REFERENCES sessions(session_id),
    event_kind TEXT NOT NULL,
    processed_stream_id TEXT,
    raw_stream_id TEXT,
    pre_roll_buffer_id BIGINT,
    selected_mic TEXT,
    selected_speaker TEXT,
    device_health TEXT,
    jitter_ms DOUBLE PRECISION,
    drift_ppm DOUBLE PRECISION,
    buffer_depth_ms DOUBLE PRECISION,
    underruns BIGINT,
    overruns BIGINT,
    phrase_id INTEGER,
    phrase_text TEXT,
    reason_code BIGINT,
    tts_playback_active BOOLEAN,
    capture_degraded BOOLEAN,
    aec_unstable BOOLEAN,
    device_changed BOOLEAN,
    stream_gap_detected BOOLEAN,
    created_at BIGINT NOT NULL,
    idempotency_key TEXT NOT NULL,
    CHECK (event_kind IN (
        'STREAM_REFS',
        'VAD_EVENT',
        'DEVICE_STATE',
        'TIMING_STATS',
        'INTERRUPT_CANDIDATE',
        'DEGRADATION_FLAGS',
        'TTS_PLAYBACK_ACTIVE'
    )),
    CHECK (device_health IS NULL OR device_health IN ('HEALTHY', 'DEGRADED', 'FAILED'))
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_audio_runtime_events_dedupe
    ON audio_runtime_events(tenant_id, device_id, event_kind, idempotency_key);

CREATE INDEX IF NOT EXISTS ix_audio_runtime_events_tenant_device_time
    ON audio_runtime_events(tenant_id, device_id, created_at);

CREATE TABLE IF NOT EXISTS audio_runtime_current (
    tenant_id TEXT NOT NULL,
    device_id TEXT NOT NULL REFERENCES devices(device_id),
    session_id BIGINT REFERENCES sessions(session_id),
    last_event_id BIGINT NOT NULL REFERENCES audio_runtime_events(event_id),
    processed_stream_id TEXT,
    raw_stream_id TEXT,
    pre_roll_buffer_id BIGINT,
    selected_mic TEXT,
    selected_speaker TEXT,
    device_health TEXT,
    jitter_ms_milli BIGINT,
    drift_ppm_milli BIGINT,
    buffer_depth_ms_milli BIGINT,
    underruns BIGINT,
    overruns BIGINT,
    tts_playback_active BOOLEAN NOT NULL DEFAULT FALSE,
    capture_degraded BOOLEAN NOT NULL DEFAULT FALSE,
    aec_unstable BOOLEAN NOT NULL DEFAULT FALSE,
    device_changed BOOLEAN NOT NULL DEFAULT FALSE,
    stream_gap_detected BOOLEAN NOT NULL DEFAULT FALSE,
    last_interrupt_phrase TEXT,
    last_interrupt_reason_code BIGINT,
    updated_at BIGINT NOT NULL,
    PRIMARY KEY (tenant_id, device_id),
    CHECK (device_health IS NULL OR device_health IN ('HEALTHY', 'DEGRADED', 'FAILED'))
);
