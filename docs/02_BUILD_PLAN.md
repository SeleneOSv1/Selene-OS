# Selene OS Build Plan

## Section P0: Execution-Grade Layer (must be built first)
- Freeze kernel contracts + validators: goal: establish immutable runtime envelopes and strict validation; required artifacts: `selene_kernel_contracts` types, validators, version policy; acceptance bar: all runtime boundaries reject unknown/invalid envelopes deterministically.
- Enforce mediation (no direct engine calls): goal: make Selene OS the only orchestrator; required artifacts: orchestrator dispatch boundary, enforcement tests, static call graph checks; acceptance bar: engine-to-engine dispatch attempts are blocked and test-proven.
- Idempotency + outbox + retries + dedupe: goal: guarantee no duplicate side effects across retries/restarts; required artifacts: idempotency key spec, outbox schema, retry scheduler, dedupe constraints; acceptance bar: repeated requests with same key produce one logical effect.
- DB role hardening + break-glass: goal: prevent manual mutation of protected ledgers; required artifacts: Postgres role model, grants matrix, audited break-glass runbook; acceptance bar: forbidden writes are blocked by role policy and break-glass usage is auditable.
- Observability + replay tooling: goal: reconstruct every decision path by correlation ID; required artifacts: canonical audit schema, correlation propagation, replay CLI; acceptance bar: deterministic replay output for the same correlation ID is identical across runs.
- Benchmark + chaos harness: goal: validate reliability under adverse conditions; required artifacts: benchmark corpus, fault injection scenarios, pass/fail thresholds; acceptance bar: SLO and failure-recovery criteria pass under defined chaos profiles.

## Section P1: Reliability
- Runtime resilience hardening: goal: stable operation through intermittent provider/device faults; required artifacts: fallback policies, timeout budgets, bounded retries; acceptance bar: known transient failures recover without unsafe execution.
- Contract evolution safety: goal: support controlled schema upgrades; required artifacts: N/N-1 compatibility rules, migration tests, deprecation policy; acceptance bar: old supported clients continue passing contracts without drift.
- Clarification determinism: goal: guarantee consistent missing-field question selection; required artifacts: blocking-field priority rules, test fixtures; acceptance bar: same inputs always produce same clarify question.
- Tool provenance integrity: goal: keep read-only results traceable and conflict-aware; required artifacts: provenance metadata schema, conflict flags, freshness policy; acceptance bar: tool outputs always include structured provenance and conflict handling.

## Section P2: World-Class
- Latency and quality envelopes: goal: enforce p50/p95/p99 interaction budgets; required artifacts: SLO definitions, budget guards, perf dashboards; acceptance bar: production workloads stay within agreed latency/error envelopes.
- Cost guardrails: goal: keep per-user/day spend predictable; required artifacts: budget policy, per-turn accounting, throttling rules; acceptance bar: budget overruns are prevented or fail closed deterministically.
- Multi-tenant hard isolation: goal: ensure tenant-safe execution and storage boundaries; required artifacts: tenant propagation contract, isolation tests, policy gates; acceptance bar: cross-tenant access attempts fail by default with audit proof.
- Governance controls: goal: provide rapid safe-mode and kill-switch operations; required artifacts: governance simulations, operator controls, incident playbooks; acceptance bar: incident controls execute deterministically and are fully auditable.
