# CODEX LAW

Purpose: Non-negotiable operating rules for Codex work in this repo.

## Rule: Worktree + Branch per Chat
- Single Codex session: always use main repo path only. Never create worktrees.
- Main repo path: `/Users/xiamo/Documents/A-Selene/Selene-OS`.
- Do not create extra repo folders.

## Rule: Clean Tree + Push Discipline
- Start of every run: `git status --short` must be empty. If not, stop and fix first.
- End of every run: tests PASS, append a PASS line to `docs/03_BUILD_LEDGER.md`, commit only the run files + ledger, push to origin, then `git status --short` must be empty again.
- No local-only real work.
- No untracked files left in the repo. Commit real work, otherwise ignore/move it.

## Rule: Probabilistic Reasoning vs Deterministic Execution (Non-Negotiable)
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

## Override
- These laws apply by default unless JD explicitly overrides them.
