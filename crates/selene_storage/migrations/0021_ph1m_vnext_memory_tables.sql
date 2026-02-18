-- PH1.M vNext persistence tables.
-- Scope: memory suppression/thread/emotional/graph/metrics/retention persistence.
-- Note: this migration extends PH1.M DB wiring beyond legacy memory_ledger/memory_current.

CREATE TABLE IF NOT EXISTS memory_atoms_ledger (
    memory_atom_event_id BIGSERIAL PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES identities(user_id),
    atom_key TEXT NOT NULL,
    event_kind TEXT NOT NULL,
    atom_value JSONB,
    confidence TEXT NOT NULL,
    sensitivity_flag TEXT NOT NULL,
    use_policy TEXT NOT NULL,
    reason_code BIGINT NOT NULL,
    t_event BIGINT NOT NULL,
    expires_at BIGINT,
    idempotency_key TEXT NOT NULL,
    CHECK (char_length(trim(atom_key)) > 0),
    CHECK (event_kind IN ('ATOM_STORED', 'ATOM_UPDATED', 'ATOM_FORGOTTEN')),
    CHECK (char_length(trim(idempotency_key)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_memory_atoms_ledger_user_idempotency
    ON memory_atoms_ledger(user_id, idempotency_key);

CREATE INDEX IF NOT EXISTS ix_memory_atoms_ledger_user_time
    ON memory_atoms_ledger(user_id, t_event);

CREATE TABLE IF NOT EXISTS memory_atoms_current (
    user_id TEXT NOT NULL REFERENCES identities(user_id),
    atom_key TEXT NOT NULL,
    atom_value JSONB,
    confidence TEXT NOT NULL,
    sensitivity_flag TEXT NOT NULL,
    use_policy TEXT NOT NULL,
    last_seen_at BIGINT NOT NULL,
    expires_at BIGINT,
    active BOOLEAN NOT NULL,
    provenance JSONB NOT NULL,
    PRIMARY KEY (user_id, atom_key),
    CHECK (char_length(trim(atom_key)) > 0)
);

CREATE TABLE IF NOT EXISTS memory_suppression_rules (
    user_id TEXT NOT NULL REFERENCES identities(user_id),
    target_type TEXT NOT NULL,
    target_id TEXT NOT NULL,
    rule_kind TEXT NOT NULL,
    active BOOLEAN NOT NULL,
    reason_code BIGINT NOT NULL,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    PRIMARY KEY (user_id, target_type, target_id, rule_kind),
    CHECK (target_type IN ('THREAD_ID', 'WORK_ORDER_ID', 'TOPIC_KEY')),
    CHECK (rule_kind IN ('DO_NOT_MENTION', 'DO_NOT_REPEAT', 'DO_NOT_STORE')),
    CHECK (char_length(trim(target_id)) > 0)
);

CREATE INDEX IF NOT EXISTS ix_memory_suppression_rules_user_updated
    ON memory_suppression_rules(user_id, updated_at);

CREATE TABLE IF NOT EXISTS emotional_threads_ledger (
    emotional_thread_event_id BIGSERIAL PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES identities(user_id),
    thread_key TEXT NOT NULL,
    tone_tags JSONB NOT NULL,
    summary TEXT,
    updated_at BIGINT NOT NULL,
    reason_code BIGINT NOT NULL,
    idempotency_key TEXT NOT NULL,
    CHECK (char_length(trim(thread_key)) > 0),
    CHECK (char_length(trim(idempotency_key)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_emotional_threads_ledger_user_idempotency
    ON emotional_threads_ledger(user_id, idempotency_key);

CREATE TABLE IF NOT EXISTS emotional_threads_current (
    user_id TEXT NOT NULL REFERENCES identities(user_id),
    thread_key TEXT NOT NULL,
    tone_tags JSONB NOT NULL,
    summary TEXT,
    updated_at BIGINT NOT NULL,
    PRIMARY KEY (user_id, thread_key),
    CHECK (char_length(trim(thread_key)) > 0)
);

CREATE TABLE IF NOT EXISTS memory_metrics_ledger (
    memory_metric_event_id BIGSERIAL PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES identities(user_id),
    payload JSONB NOT NULL,
    reason_code BIGINT NOT NULL,
    created_at BIGINT NOT NULL,
    idempotency_key TEXT NOT NULL,
    CHECK (char_length(trim(idempotency_key)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_memory_metrics_ledger_user_idempotency
    ON memory_metrics_ledger(user_id, idempotency_key);

CREATE TABLE IF NOT EXISTS memory_threads_ledger (
    memory_thread_event_id BIGSERIAL PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES identities(user_id),
    thread_id TEXT NOT NULL,
    thread_title TEXT NOT NULL,
    summary_bullets JSONB NOT NULL,
    pinned BOOLEAN NOT NULL,
    unresolved BOOLEAN NOT NULL,
    last_updated_at BIGINT NOT NULL,
    use_count BIGINT NOT NULL,
    memory_retention_mode TEXT NOT NULL,
    event_kind TEXT NOT NULL,
    reason_code BIGINT NOT NULL,
    idempotency_key TEXT NOT NULL,
    CHECK (char_length(trim(thread_id)) > 0),
    CHECK (char_length(trim(thread_title)) > 0),
    CHECK (memory_retention_mode IN ('DEFAULT', 'REMEMBER_EVERYTHING')),
    CHECK (event_kind IN ('THREAD_DIGEST_UPSERT', 'THREAD_RESOLVED', 'THREAD_FORGOTTEN')),
    CHECK (char_length(trim(idempotency_key)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_memory_threads_ledger_user_idempotency
    ON memory_threads_ledger(user_id, idempotency_key);

CREATE TABLE IF NOT EXISTS memory_threads_current (
    user_id TEXT NOT NULL REFERENCES identities(user_id),
    thread_id TEXT NOT NULL,
    thread_title TEXT NOT NULL,
    summary_bullets JSONB NOT NULL,
    pinned BOOLEAN NOT NULL,
    unresolved BOOLEAN NOT NULL,
    unresolved_deadline_at BIGINT,
    last_used_at BIGINT NOT NULL,
    last_updated_at BIGINT NOT NULL,
    use_count BIGINT NOT NULL,
    memory_retention_mode TEXT NOT NULL,
    PRIMARY KEY (user_id, thread_id),
    CHECK (char_length(trim(thread_id)) > 0),
    CHECK (char_length(trim(thread_title)) > 0),
    CHECK (memory_retention_mode IN ('DEFAULT', 'REMEMBER_EVERYTHING'))
);

CREATE INDEX IF NOT EXISTS ix_memory_threads_current_user_last_updated
    ON memory_threads_current(user_id, last_updated_at);

CREATE TABLE IF NOT EXISTS memory_thread_refs (
    user_id TEXT NOT NULL REFERENCES identities(user_id),
    thread_id TEXT NOT NULL,
    conversation_turn_id BIGINT NOT NULL REFERENCES conversation_ledger(conversation_turn_id),
    created_at BIGINT NOT NULL,
    PRIMARY KEY (user_id, thread_id, conversation_turn_id),
    CHECK (char_length(trim(thread_id)) > 0)
);

CREATE TABLE IF NOT EXISTS memory_graph_nodes (
    user_id TEXT NOT NULL REFERENCES identities(user_id),
    node_id TEXT NOT NULL,
    node_kind TEXT NOT NULL,
    confidence TEXT NOT NULL,
    last_used_at BIGINT NOT NULL,
    use_count BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    PRIMARY KEY (user_id, node_id),
    CHECK (char_length(trim(node_id)) > 0),
    CHECK (node_kind IN ('ENTITY', 'PROJECT', 'VENDOR', 'DECISION', 'THREAD'))
);

CREATE TABLE IF NOT EXISTS memory_graph_edges (
    user_id TEXT NOT NULL REFERENCES identities(user_id),
    edge_id TEXT NOT NULL,
    from_node_id TEXT NOT NULL,
    to_node_id TEXT NOT NULL,
    edge_kind TEXT NOT NULL,
    confidence TEXT NOT NULL,
    last_used_at BIGINT NOT NULL,
    use_count BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    PRIMARY KEY (user_id, edge_id),
    CHECK (char_length(trim(edge_id)) > 0),
    CHECK (char_length(trim(from_node_id)) > 0),
    CHECK (char_length(trim(to_node_id)) > 0),
    CHECK (edge_kind IN ('MENTIONED_WITH', 'DEPENDS_ON', 'DECIDED_IN', 'BLOCKED_BY'))
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_memory_graph_edges_deterministic
    ON memory_graph_edges(user_id, from_node_id, to_node_id, edge_kind);

CREATE TABLE IF NOT EXISTS memory_archive_index (
    user_id TEXT NOT NULL REFERENCES identities(user_id),
    archive_ref_id TEXT NOT NULL,
    thread_id TEXT,
    conversation_turn_id BIGINT REFERENCES conversation_ledger(conversation_turn_id),
    rank_score BIGINT,
    updated_at BIGINT NOT NULL,
    PRIMARY KEY (user_id, archive_ref_id),
    CHECK (char_length(trim(archive_ref_id)) > 0)
);

CREATE TABLE IF NOT EXISTS memory_retention_preferences (
    user_id TEXT PRIMARY KEY REFERENCES identities(user_id),
    memory_retention_mode TEXT NOT NULL,
    updated_at BIGINT NOT NULL,
    reason_code BIGINT NOT NULL,
    idempotency_key TEXT,
    CHECK (memory_retention_mode IN ('DEFAULT', 'REMEMBER_EVERYTHING'))
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_memory_retention_preferences_user_idempotency
    ON memory_retention_preferences(user_id, idempotency_key)
    WHERE idempotency_key IS NOT NULL;
