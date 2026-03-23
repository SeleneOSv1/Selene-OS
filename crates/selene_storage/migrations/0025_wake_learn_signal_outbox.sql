-- Wake learning signal ledger + outbox linkage.
-- Deterministic write-before-enqueue contract for wake feedback loops.

CREATE TABLE IF NOT EXISTS wake_learn_signals (
  wake_learn_signal_row_id BIGSERIAL PRIMARY KEY,
  schema_version INTEGER NOT NULL,
  created_at_ns BIGINT NOT NULL,
  signal_id TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  wake_window_id TEXT NOT NULL,
  event_type TEXT NOT NULL,
  device_id TEXT NOT NULL,
  session_id TEXT,
  trigger TEXT NOT NULL,
  model_version TEXT,
  score_bp INTEGER,
  threshold_bp INTEGER,
  reason_code INTEGER,
  snr_db_milli INTEGER,
  vad_coverage_bp INTEGER,
  timestamp_ms BIGINT NOT NULL,
  outbox_receipt_ref TEXT NOT NULL,
  outbox_sync_job_id TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_wake_learn_signal_device_signal
  ON wake_learn_signals(device_id, signal_id);

CREATE UNIQUE INDEX IF NOT EXISTS ux_wake_learn_signal_receipt
  ON wake_learn_signals(outbox_receipt_ref);
