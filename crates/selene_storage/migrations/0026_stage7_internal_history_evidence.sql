-- Stage 7 immutable internal history and human-memory evidence ledger.
-- Scope: nullable evidence refs for committed/rejected turns, speaker posture,
-- PH1.X/PH1.M packet refs, tool/source/presentation/multimodal/protected/timing refs.
-- Runtime currently uses the in-memory PH1.F store; this migration keeps DB wiring aligned.

CREATE TABLE IF NOT EXISTS internal_history_evidence_ledger (
    internal_history_event_id BIGSERIAL PRIMARY KEY,
    created_at BIGINT NOT NULL,
    event_kind TEXT NOT NULL,
    conversation_turn_id BIGINT REFERENCES conversation_ledger(conversation_turn_id),
    correlation_id NUMERIC(39,0) NOT NULL,
    turn_id BIGINT NOT NULL,
    session_id BIGINT REFERENCES sessions(session_id),
    thread_key TEXT,
    role TEXT,
    source TEXT,
    modality TEXT NOT NULL,

    user_id TEXT REFERENCES identities(user_id),
    actor_id TEXT,
    device_id TEXT REFERENCES devices(device_id),
    speaker_id TEXT,
    speaker_label TEXT,
    voice_profile_id TEXT,
    identity_posture TEXT NOT NULL,
    voice_id_confidence_bp INTEGER,
    voice_id_score_bp INTEGER,
    voice_id_margin_bp INTEGER,
    same_speaker_as_previous BOOLEAN,
    speaker_changed BOOLEAN,
    voice_identity_assertion_ref TEXT,
    liveness_ref TEXT,
    capture_attestation_ref TEXT,
    typed_actor_identity_ref TEXT,
    privacy_scope_ref TEXT,
    memory_scope_ref TEXT,
    access_posture_ref TEXT,

    input_evidence JSONB NOT NULL,
    response_evidence JSONB NOT NULL,
    ph1x_evidence JSONB NOT NULL,
    ph1m_evidence JSONB NOT NULL,
    tool_provider_refs JSONB NOT NULL,
    source_refs JSONB NOT NULL,
    presentation_refs JSONB NOT NULL,
    multimodal_refs JSONB NOT NULL,
    correction_refs JSONB NOT NULL,
    decision_task_refs JSONB NOT NULL,
    privacy_retention_refs JSONB NOT NULL,
    protected_execution_refs JSONB NOT NULL,
    timing_refs JSONB NOT NULL,
    device_surface_provenance_refs JSONB NOT NULL,
    audit_refs JSONB NOT NULL,
    replay_integrity_refs JSONB NOT NULL,
    idempotency_key TEXT,
    UNIQUE (correlation_id, idempotency_key)
);

CREATE INDEX IF NOT EXISTS ix_internal_history_evidence_turn
    ON internal_history_evidence_ledger(conversation_turn_id);

CREATE INDEX IF NOT EXISTS ix_internal_history_evidence_session_turn
    ON internal_history_evidence_ledger(session_id, turn_id);

CREATE INDEX IF NOT EXISTS ix_internal_history_evidence_speaker
    ON internal_history_evidence_ledger(speaker_id, created_at);
