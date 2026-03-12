# SELENE AGENTS LAW (AUTO-LOADED)

This file is the active auto-loaded execution law for Codex in this repository.
If any instruction conflicts with this file, this file wins unless JD explicitly overrides it.

## Absolute Rule 0) No Python
- Never use Python in this repository.
- Disallowed: `python`, `python3`, `uv run python`, inline Python, and any script that invokes Python.
- If a task would require Python, stop and report exactly:
  - `Python is disallowed by AGENTS.md`

## Repository Locality Rule
- Work only in the main Selene repository:
  - `/Users/xiamo/Documents/A-Selene/Selene-OS`
- Never create worktrees.
- Never create extra repo folders.
- Never move work into side repositories.

## Auto-Loaded Authority Order
1. JD explicit in-thread instructions
2. This `AGENTS.md`
3. Repository architecture law and docs
4. Task-specific prompt scope

## Mandatory First-Read Files for Major Work
Before any architecture work, design work, or code-editing run, Codex must read:
- `docs/CORE_ARCHITECTURE.md`
- `docs/SELENE_BUILD_EXECUTION_ORDER.md`
- `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`

If the task concerns a specific build section, Codex must also read the relevant section file before proposing or changing anything.

## Architectural Boundary: Probabilistic vs Deterministic
Selene must follow this boundary:

Probabilistic layer:
- language generation
- reasoning
- summarization
- research
- explanation
- read-only analysis

Deterministic layer:
- intent dispatch
- access control
- simulation execution
- state mutation
- ledger writes
- artifact activation
- onboarding progression
- provider promotion or demotion
- message sending
- any irreversible action

All deterministic execution must:
- pass access checks
- require ACTIVE simulation IDs when applicable
- be idempotent
- be replay-safe
- be auditable
- fail closed on inconsistency

Language may be probabilistic.
Execution must never be probabilistic.

## Non-Negotiable Core Protection Rule
Most important law:

Codex must not override, replace, delete, break, or drift previously built working code unless JD explicitly authorizes that change.

This includes:
- removing working logic
- replacing old behavior with new behavior
- silently changing contract meaning
- deleting existing runtime paths
- broad rewrites of existing systems
- “cleanup” that changes behavior
- convenience rewrites that erase prior implementation work

Default rule:
- existing built code is protected
- existing behavior stays unless JD explicitly approves change
- additive work is preferred over destructive replacement
- if a prior implementation conflicts with a new idea, stop and ask JD before changing it

Codex must not keep rebuilding the same area by deleting prior work and replacing it repeatedly.
If a change would alter previous behavior, Codex must stop and get JD approval first.

## Existing Code Default-Deny Rule
- Existing files are read-only by default.
- New additive files/modules/tests are allowed by default only if task scope permits.
- Editing existing files requires explicit file-scope approval from JD.
- If approval is unclear: stop and ask.

## Spine Lock
The following high-risk core areas are immutable without explicit JD approval:
- PH1.X
- PH1.OS
- PH1.L
- SimulationExecutor
- shared dispatch and mutation layers
- packet contracts
- reason registry
- idempotency registry
- state machine
- turn ordering
- access control

Any touch to spine code requires explicit JD approval.

## Contract Surface Lock
No removals, renames, semantic changes, or ordering changes to shared:
- fields
- enums
- packet meaning
- gate order
- state order
- idempotency behavior

unless JD explicitly approves it.

Backward-compatible additive changes only by default.

## File Scope Gate
Before any edit run, Codex must declare:
- exact files to be changed
- why each file must change

No touching files outside declared scope.
If more files are needed, stop and ask JD first.

## Clean Tree Rule
Start of every run:
- `git status --short` must be empty
- if not empty: stop and report it

End of every run:
- `git status --short` must be empty again

No new task may begin on a dirty tree.
No task is complete until the tree is clean again.

## Network Stability Rule
- `🤖 ChatGPT` is locked to `🇸🇬 新加坡 01`.
- Codex must not change Clash, macOS proxy settings, git config, ssh config, shell proxy variables, or network settings unless JD explicitly approves it first.
- If connectivity fails, Codex must stop and report the failure instead of attempting fixes.

## Shell-Only Inspection Rule
Allowed inspection commands:
- `git`
- `rg`
- `grep`
- `sed`
- `awk`
- `cat`
- `head`
- `tail`
- `find`
- `ls`
- `cut`
- `sort`
- `uniq`
- `wc`

Forbidden:
- `python`
- `python3`
- `perl`
- `ruby`
- `node`
- `php`
- `lua`
- any scripted workaround that bypasses this law

If Codex violates this shell-only rule:
- stop immediately
- report the violation
- discard that line of investigation
- restart with shell-only inspection

## Baseline Gate
For change runs:
- run baseline tests for the affected domain before edits
- if baseline is red: stop immediately and report
- do not code until JD decides

## Change Class Gate
Class A:
- additive new modules only
- allowed if within task scope

Class B:
- edit non-spine existing files
- requires JD approval

Class C:
- touch spine / contracts / ordering / idempotency / shared behavior
- requires explicit JD approval plus expanded proof

## No Surprise Refactors
No unrequested:
- renames
- file moves
- cleanup sweeps
- formatting churn
- convenience rewrites
- broad rewrites
- “while I’m here” changes

## Determinism Lock
- stable ordering only
- no randomness
- no hidden fallbacks
- no time-based drift in deterministic paths
- same input must produce same output unless explicitly approved

## Fail-Closed Lock
Missing required inputs must refuse with registered reason code.
No:
- silent correction
- guessing
- hidden continuation
- best-effort mutation

## Non-Destructive Implementation Rule

When JD provides architecture, design, or build instructions:

Codex must not:
- delete previously defined functionality
- replace existing behavior with reduced scope
- silently simplify a design
- reinterpret instructions in a way that removes capabilities

Codex must:
- preserve all existing functionality
- extend functionality by adding new components
- keep previously implemented behavior intact unless JD explicitly authorizes removal

If a change would remove or replace existing functionality:

STOP and request explicit JD authorization.

## No Parallel Bypass Paths
Execution must stay inside canonical boundaries:
- PH1.X classification
- access control
- simulation verification when applicable
- SimulationExecutor
- deterministic mutation
- audit logging

No shortcut or bypass execution paths.

## Stop-on-Uncertainty Rule
If uncertain about:
- contract meaning
- state order
- gate sequencing
- simulation boundaries
- engine interaction
- whether old code may be changed

stop and ask JD.
Guessing is forbidden.

## Engineering Quality Standard
All code delivered by Codex must aim for production-grade quality.

Rules:
- warnings must be fixed, not ignored
- no shortcuts
- no lazy fixes
- no temporary hacks
- no weak workaround patch if proper implementation is achievable
- preserve full required functionality
- do not silently reduce scope to make a patch easier
- if the proper implementation is too large, stop at a clean architectural boundary and report it rather than shipping a weak partial

Clean builds are the target.
Warning-free is the default expectation.

## Mandatory Proof Before Commit
For any change run, Codex must provide:
- baseline commands and results
- targeted tests and results
- relevant contract / reason / idempotency / state-machine checks
- determinism checks where applicable
- `git diff --stat`
- proof that no unrelated files changed

## Mandatory Cleanliness Gates
Change runs:
- tests/checks must pass
- if ledger update is required by repo law, do it
- commit only relevant files
- push to origin
- prove clean tree again

Read-only runs:
- no commit
- no edits
- prove clean tree at end

No local-only real work.
No leftover untracked junk.
No unfinished hidden changes.

## Hard Fail Conditions
Immediate stop on:
- attempted Python usage
- missing required JD approval
- baseline failure before edits on a change run
- out-of-scope file touched
- unknown reason code introduced
- state or gate ordering changed without approval
- previous working code being overridden without JD approval

## Commit Discipline
- one coherent changeset per run unless JD explicitly says otherwise
- commit message should reflect run intent
- do not mix unrelated workstreams

## Final Safety Outcome
Selene must not suffer:
- hidden drift
- silent corruption
- repeated replacement of previously built work
- contract breakage without JD approval
- messy dirty-tree iteration loops

## AGENTS.md Status Rule
This `AGENTS.md` is the active Codex law file for this repository.
`CODEX_LAW.md` is reference-only documentation unless JD says otherwise.

## Override
These laws apply by default unless JD explicitly overrides them in-thread.
