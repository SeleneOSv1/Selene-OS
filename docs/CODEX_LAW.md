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

## Override
- These laws apply by default unless JD explicitly overrides them.
