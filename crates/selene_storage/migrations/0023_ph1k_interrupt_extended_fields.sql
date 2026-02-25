-- PH1.K round-2 Step 11 storage extension.
-- Scope: append-only extension for richer interrupt-candidate metrics/confidence payload,
-- while preserving existing idempotency and append-only invariants.

ALTER TABLE audio_runtime_events
    ADD COLUMN IF NOT EXISTS trigger_phrase_id INTEGER,
    ADD COLUMN IF NOT EXISTS trigger_locale TEXT,
    ADD COLUMN IF NOT EXISTS candidate_confidence_band TEXT,
    ADD COLUMN IF NOT EXISTS vad_decision_confidence_band TEXT,
    ADD COLUMN IF NOT EXISTS risk_context_class TEXT,
    ADD COLUMN IF NOT EXISTS quality_metrics_snr_db DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS quality_metrics_clipping_ratio DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS quality_metrics_echo_delay_ms DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS quality_metrics_packet_loss_pct DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS quality_metrics_double_talk_score DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS quality_metrics_erle_db DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS timing_markers_window_start_ns BIGINT,
    ADD COLUMN IF NOT EXISTS timing_markers_window_end_ns BIGINT,
    ADD COLUMN IF NOT EXISTS speech_window_metrics_voiced_window_ms INTEGER,
    ADD COLUMN IF NOT EXISTS subject_relation_confidence_bundle_lexical_confidence DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS subject_relation_confidence_bundle_vad_confidence DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS subject_relation_confidence_bundle_speech_likeness DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS subject_relation_confidence_bundle_echo_safe_confidence DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS subject_relation_confidence_bundle_nearfield_confidence DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS subject_relation_confidence_bundle_combined_confidence DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS interrupt_policy_profile_id TEXT,
    ADD COLUMN IF NOT EXISTS interrupt_tenant_profile_id TEXT,
    ADD COLUMN IF NOT EXISTS interrupt_locale_tag TEXT,
    ADD COLUMN IF NOT EXISTS adaptive_device_route TEXT,
    ADD COLUMN IF NOT EXISTS adaptive_noise_class TEXT,
    ADD COLUMN IF NOT EXISTS adaptive_capture_to_handoff_latency_ms INTEGER,
    ADD COLUMN IF NOT EXISTS adaptive_timing_jitter_ms DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS adaptive_timing_drift_ppm DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS adaptive_device_reliability_score DOUBLE PRECISION;

ALTER TABLE audio_runtime_current
    ADD COLUMN IF NOT EXISTS last_interrupt_trigger_phrase_id INTEGER,
    ADD COLUMN IF NOT EXISTS last_interrupt_trigger_locale TEXT,
    ADD COLUMN IF NOT EXISTS last_interrupt_candidate_confidence_band TEXT,
    ADD COLUMN IF NOT EXISTS last_interrupt_vad_decision_confidence_band TEXT,
    ADD COLUMN IF NOT EXISTS last_interrupt_risk_context_class TEXT,
    ADD COLUMN IF NOT EXISTS last_interrupt_policy_profile_id TEXT,
    ADD COLUMN IF NOT EXISTS last_interrupt_tenant_profile_id TEXT,
    ADD COLUMN IF NOT EXISTS last_interrupt_locale_tag TEXT,
    ADD COLUMN IF NOT EXISTS last_interrupt_adaptive_device_route TEXT,
    ADD COLUMN IF NOT EXISTS last_interrupt_adaptive_noise_class TEXT,
    ADD COLUMN IF NOT EXISTS last_interrupt_adaptive_capture_to_handoff_latency_ms INTEGER,
    ADD COLUMN IF NOT EXISTS last_interrupt_snr_db_milli BIGINT,
    ADD COLUMN IF NOT EXISTS last_interrupt_clipping_ratio_milli BIGINT,
    ADD COLUMN IF NOT EXISTS last_interrupt_echo_delay_ms_milli BIGINT,
    ADD COLUMN IF NOT EXISTS last_interrupt_packet_loss_pct_milli BIGINT,
    ADD COLUMN IF NOT EXISTS last_interrupt_double_talk_score_milli BIGINT,
    ADD COLUMN IF NOT EXISTS last_interrupt_erle_db_milli BIGINT;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'chk_audio_runtime_events_candidate_confidence_band'
    ) THEN
        ALTER TABLE audio_runtime_events
            ADD CONSTRAINT chk_audio_runtime_events_candidate_confidence_band
                CHECK (candidate_confidence_band IS NULL OR candidate_confidence_band IN ('HIGH','MEDIUM','LOW'));
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'chk_audio_runtime_events_vad_decision_confidence_band'
    ) THEN
        ALTER TABLE audio_runtime_events
            ADD CONSTRAINT chk_audio_runtime_events_vad_decision_confidence_band
                CHECK (vad_decision_confidence_band IS NULL OR vad_decision_confidence_band IN ('HIGH','MEDIUM','LOW'));
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'chk_audio_runtime_events_risk_context_class'
    ) THEN
        ALTER TABLE audio_runtime_events
            ADD CONSTRAINT chk_audio_runtime_events_risk_context_class
                CHECK (risk_context_class IS NULL OR risk_context_class IN ('LOW','GUARDED','HIGH'));
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'chk_audio_runtime_current_candidate_confidence_band'
    ) THEN
        ALTER TABLE audio_runtime_current
            ADD CONSTRAINT chk_audio_runtime_current_candidate_confidence_band
                CHECK (last_interrupt_candidate_confidence_band IS NULL OR last_interrupt_candidate_confidence_band IN ('HIGH','MEDIUM','LOW'));
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'chk_audio_runtime_current_vad_decision_confidence_band'
    ) THEN
        ALTER TABLE audio_runtime_current
            ADD CONSTRAINT chk_audio_runtime_current_vad_decision_confidence_band
                CHECK (last_interrupt_vad_decision_confidence_band IS NULL OR last_interrupt_vad_decision_confidence_band IN ('HIGH','MEDIUM','LOW'));
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'chk_audio_runtime_current_risk_context_class'
    ) THEN
        ALTER TABLE audio_runtime_current
            ADD CONSTRAINT chk_audio_runtime_current_risk_context_class
                CHECK (last_interrupt_risk_context_class IS NULL OR last_interrupt_risk_context_class IN ('LOW','GUARDED','HIGH'));
    END IF;
END $$;
