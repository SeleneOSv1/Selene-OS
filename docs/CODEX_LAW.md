# CODEX LAW (Selene)

Purpose: Non-negotiable operating rules for Codex work in this repo.

## Rule 1: Clean Git Tree + Push Always (no exceptions)
- Before doing anything: run `git status --short`. If it is not empty, stop and fix first by committing the real work to a branch and pushing it.
- After doing anything: run tests, append the PASS verification line to the build ledger, commit only the run's files + ledger, push to origin, then run `git status --short` again and it must be empty.
- No local-only real work.
- No untracked files left in the repo (commit them if real work, otherwise move them out or add to `.gitignore`).

## Override
- These laws apply by default unless JD explicitly overrides them.
