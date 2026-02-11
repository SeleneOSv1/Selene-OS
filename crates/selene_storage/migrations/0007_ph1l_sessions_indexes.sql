-- PH1.L session lifecycle DB wiring indexes (design-lock row 8)
-- Scope: optimize deterministic PH1.L reads on existing os_core.sessions table.

CREATE INDEX IF NOT EXISTS ix_sessions_user_state_last_activity
    ON sessions(user_id, session_state, last_activity_at DESC);

CREATE INDEX IF NOT EXISTS ix_sessions_device_state_last_activity
    ON sessions(device_id, session_state, last_activity_at DESC);

CREATE INDEX IF NOT EXISTS ix_sessions_state_last_activity
    ON sessions(session_state, last_activity_at DESC);
