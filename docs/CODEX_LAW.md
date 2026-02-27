# Codex Law (Selene)

Purpose: Non-negotiable operating rules for Codex work in this repo.

## Rule 1: Worktree + Branch per Chat (Mandatory)
- Every new Codex chat must use a new worktree folder and a new branch. Never reuse an old worktree.
- At chat start:
```bash
git fetch origin
git worktree add ../selene-wt-<task_name> origin/main
cd ../selene-wt-<task_name>
git checkout -b codex/<task_name>-<yyyy-mm-dd>
git branch --show-current
git status --short
```
- `git status --short` must be empty at start.
- During the chat, work only inside that worktree folder.
- End of chat (mandatory):
```bash
# run the task's test commands
# update docs/03_BUILD_LEDGER.md with PASS line
git add <run_files> docs/03_BUILD_LEDGER.md
git commit -m "<message>"
git push -u origin codex/<task_name>-<yyyy-mm-dd>
git status --short
```
- End state must be clean (`git status --short` empty).
- New sessions always start from a fresh worktree from `origin/main`; do not resume work by reusing old dirty folders.

## Rule 2: Clean Git Tree + Push Always (No Exceptions)
- Before doing anything: run `git status --short`. If it is not empty, stop and fix first by committing real work to a branch and pushing it.
- After doing anything: run tests, append PASS verification to `docs/03_BUILD_LEDGER.md`, commit only run files + ledger, push, then run `git status --short` again and it must be empty.
- No local-only real work.
- No untracked files left in the repo (commit real work, otherwise move out or add to `.gitignore`).

## Override
- These laws apply by default unless JD explicitly overrides them.
