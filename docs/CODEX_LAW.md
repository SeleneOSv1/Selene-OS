# CODEX LAW (Selene)

Purpose: Non-negotiable operating rules for Codex work in this repo.

## Rule: Worktree + Branch per Chat

Every new Codex chat must use a NEW worktree folder + NEW branch. Never reuse an old worktree.

At the start of the chat, do this:

git fetch origin

create worktree from origin/main:
git worktree add ../selene-wt-<task_name> origin/main

cd ../selene-wt-<task_name>

create branch:
git checkout -b codex/<task_name>-<yyyy-mm-dd>

show:
git branch --show-current
git status --short (must be empty)

During the chat:

Only work inside this worktree folder.

If git status becomes dirty, that's fine during work, but you must keep it contained to this folder/branch.

End of the chat (must do all):

run tests (the run's test commands)

update docs/03_BUILD_LEDGER.md with PASS line

git add only the run files + ledger

git commit -m "<message>"

git push -u origin codex/<task_name>-<yyyy-mm-dd>

git status --short (must be empty)

Retiring old sessions:

Before starting a new chat, do not touch old worktrees.

The new chat always creates a new worktree from origin/main, so it automatically starts clean and current.

If you need old work, pull it by checking out the pushed branch (never by reusing the old dirty folder).

How the new session knows which worktree to use
It doesn't "guess." The rule is: new session ALWAYS creates its own worktree folder and branch, and prints the folder path + branch name at the top. That's how we track it.

## Override
- These laws apply by default unless JD explicitly overrides them.
