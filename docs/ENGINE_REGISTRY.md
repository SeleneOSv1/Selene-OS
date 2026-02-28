# Engine Registry

## OS Execution Law

### Execution Boundary Law — Probabilistic Reasoning, Deterministic Execution
Selene must follow this architectural boundary:

Probabilistic Layer (Allowed):
- Language generation
- Reasoning
- Summarization
- Research
- Data analysis
- Document/photo explanation
- Connector read-only queries
- Tone/personality shaping

These may be model-driven and non-deterministic.

Deterministic Boundary (Mandatory):
- Intent -> dispatch classification
- Access control decisions
- Simulation execution
- State mutation
- Ledger writes
- Artifact activation
- Provider promotion/demotion
- Onboarding progression
- Message sending
- Any irreversible action

All execution must:
- Pass Access checks
- Require ACTIVE simulation IDs (when applicable)
- Be idempotent
- Be replay-safe
- Be auditable
- Fail closed on any inconsistency

Language may be probabilistic.
Execution must never be probabilistic.
