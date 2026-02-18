-- Builder Selene Phase 13-E learning bridge fields.
-- Adds explicit learning-driven metadata to builder_patch_proposals.

ALTER TABLE IF EXISTS builder_patch_proposals
    ADD COLUMN IF NOT EXISTS learning_report_id TEXT,
    ADD COLUMN IF NOT EXISTS learning_source_engines_json TEXT,
    ADD COLUMN IF NOT EXISTS learning_signal_count BIGINT,
    ADD COLUMN IF NOT EXISTS learning_evidence_refs_json TEXT;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'chk_builder_learning_signal_count_non_negative'
    ) THEN
        ALTER TABLE builder_patch_proposals
            ADD CONSTRAINT chk_builder_learning_signal_count_non_negative
            CHECK (learning_signal_count IS NULL OR learning_signal_count >= 0);
    END IF;
END $$;
