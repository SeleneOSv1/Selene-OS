-- Wake artifact pull/apply/rollback activation pointers (foundation).
-- State machine: Staged -> Active -> RolledBack.

CREATE TABLE IF NOT EXISTS wake_artifact_apply_ledger (
  apply_event_id BIGSERIAL PRIMARY KEY,
  schema_version INTEGER NOT NULL,
  created_at_ns BIGINT NOT NULL,
  device_id TEXT NOT NULL,
  artifact_version INTEGER NOT NULL,
  package_hash TEXT NOT NULL,
  payload_ref TEXT NOT NULL,
  local_cache_ref TEXT,
  state TEXT NOT NULL,
  activated_at_ns BIGINT,
  rollback_reason_code INTEGER,
  idempotency_key TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_wake_artifact_apply_idempotency
  ON wake_artifact_apply_ledger(device_id, artifact_version, state, idempotency_key);

CREATE TABLE IF NOT EXISTS wake_artifact_apply_current (
  device_id TEXT PRIMARY KEY,
  schema_version INTEGER NOT NULL,
  staged_artifact_version INTEGER,
  active_artifact_version INTEGER,
  last_known_good_artifact_version INTEGER,
  activated_at_ns BIGINT,
  rollback_reason_code INTEGER
);

CREATE TABLE IF NOT EXISTS wake_artifact_blocked_versions (
  device_id TEXT NOT NULL,
  artifact_version INTEGER NOT NULL,
  rollback_reason_code INTEGER NOT NULL,
  PRIMARY KEY (device_id, artifact_version)
);
