# SELENE AGENTS LAW (AUTO-LOADED)

This file is the active auto-loaded execution law for Codex in this repository.
If any instruction conflicts with this file, this file wins unless JD explicitly overrides it.

## Absolute Rule 0) No Python
- Never use Python in this repository.
- Disallowed: `python`, `python3`, `uv run python`, inline Python, and any script that invokes Python.
- If a task would require Python, stop and report exactly:
  - `Python is disallowed by AGENTS.md`

## Repository Locality Rule
- Work only in the checked-out main Selene repository that contains this `AGENTS.md`.
- If any prompt or older machine-specific instruction names a different absolute path, current repo truth wins.
- Never create worktrees.
- Never create extra repo folders.
- Never move work into side repositories.

## Auto-Loaded Authority Order
1. JD explicit in-thread instructions
2. This `AGENTS.md`
3. Repository architecture law and docs
4. Task-specific prompt scope

## Solo Shipping Mode
Selene is currently a solo-build project operated by JD with Codex assistance.
Execution law must optimize for shipping working product quickly without lowering engineering quality.

Default operating rule:
- implementation-first is the default
- in this workflow, `build` always means `implementation run`
- `next build` means `next implementation`
- docs-only reconciliation is not a build unless JD explicitly says to do docs-only work
- publication-only work is not a build unless JD explicitly says to do publication-only work
- docs-only runs are exception-mode, not default
- Codex must prefer the maximum safe, accurate, provable code-changing progress that can be completed in one run when a safe implementation seam exists
- Codex should batch adjacent implementation slices when they share the same files, subsystem, invariant family, and proof pack, and when batching does not reduce safety, accuracy, or verification clarity
- Codex should fall back to the next smallest safe slice only when batching would increase semantic risk, widen approval boundaries, or weaken proof quality
- Codex must not create publication-only, frontier-only, or boundary-restatement runs unless JD explicitly asks for docs-only work

Required behavior in Solo Shipping Mode:
- first priority is shipping working behavior
- second priority is proving that behavior with targeted tests
- third priority is updating only the minimum docs needed to keep repo truth accurate
- docs are tail work after code by default, not the main output

Investigation limits:
- Codex must time-box investigation and repo-audit work
- if a safe implementation seam is visible, Codex must move to implementation instead of continuing analysis churn
- if multiple adjacent safe implementation slices are visible, Codex should batch them into one implementation run instead of artificially splitting them into smaller runs
- if no safe implementation seam is visible after a bounded inspection pass, Codex must stop and report the blocker in plain language
- Codex must not manufacture a docs-only build just because a code choice is not yet perfect

Docs-only exception rule:
- docs-only work requires explicit JD request
- publication-only work requires explicit JD request
- if JD does not explicitly request docs-only execution, Codex must assume the intended outcome is code progress
- if JD asks for the next build, Codex must not answer with docs-only or publication-only work
- master-doc drift alone is never enough to select the next build

Build selection rule:
- for self-authored next-build instructions, Codex must default to an implementation build
- every self-authored build instruction must treat `build` as `implementation`
- every self-authored next-build instruction must declare or assume `Build Class: IMPLEMENTATION`
- every self-authored build instruction that pins `HEAD`, `origin/main`, or a prerequisite landed-build hash must source that exact hash from same-task git output and must not reuse remembered prior assistant text
- every self-authored build instruction that cites preserved prior landed builds must include explicit same-task git verification commands for each cited prior landed build hash; pinning current `HEAD` alone is insufficient
- every self-authored build instruction that runs `xcodebuild` must clean the relevant repo-local build artifact tree before the first clean-tree check and again after each `xcodebuild` before final cleanliness verification
- the selected build must name the exact behavior to change, the exact files expected to change, and the exact tests that will prove the change
- Codex should prefer the largest safe, accurate, and provable in-scope implementation batch over a smaller batch when both are lawful and the larger batch stays within the same approval surface
- Codex must not default to one-slice-only execution when multiple adjacent slices can be implemented and proven together safely
- Codex must not answer a `next build` request with a docs-first, docs-only, publication-only, frontier-only, or authority-catch-up-only run
- Codex must not author a docs-only next build when a safe implementation build or safe implementation batch is available
- Codex must not author a docs-only next build while any safe in-scope implementation slice or batch remains
- if no safe implementation slice is exposed by current repo truth, Codex must stop and report `no lawful implementation build exposed by current repo truth` instead of substituting docs work

Anti-ceremony rule:
- do not split one real implementation outcome into multiple paperwork-only runs
- do not create separate publication steps for truths that can be recorded as part of the implementation run
- do not keep refining frontier language when the product has not changed

Quality floor:
- Solo Shipping Mode does not relax determinism, contract safety, idempotency, auditability, file-scope approval, or test proof requirements
- speed is gained by removing ceremony, not by skipping engineering discipline

Docs tail rule:
- default is no doc change
- docs may be updated only as a minimal tail on the same implementation run
- docs may be touched only when the implementation materially narrows current repo truth and the wording can be updated without overclaiming
- if JD does not explicitly ask for docs reconciliation, Codex must never select docs work as the main output of a build

Next-build stop rule:
- if the candidate next run is docs-only, publication-only, or master-doc reconciliation-only, it is not a lawful answer to `next build`
- if code truth is ahead of docs and a safe implementation slice still exists, Codex must select the implementation slice and defer docs drift
- if no safe implementation slice remains, Codex must report that directly and must not convert the absence of a code winner into a docs-selection default
- if a candidate implementation slice depends on overturning a currently preserved contrast, preserved row meaning, or preserved semantic boundary, it is not a lawful automatic next build
- guarded candidates may be written only as conditional options, not as the default next build, unless current repo truth already exposes them as safe without overturning preserved truth
- if the only remaining candidate is guarded by a possible contradiction with currently preserved truth, Codex must stop and report `no lawful implementation build exposed by current repo truth`

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

For any change run:
- `git diff --name-only` must be a subset of declared scope
- any out-of-scope file => stop and request approval

## Clean Tree Rule
Start of every run:
- `git status --short` must be empty
- if not empty: stop and report it
- the only narrow exception is the `Dirty-Tree Refinement` rule below, and it applies only within the same already-started task when the only changed or untracked file is the task target file
- repo-local build artifact trees created by `xcodebuild` are part of the worktree and must be removed before this cleanliness check
- for Apple app work, relevant cleanup targets include `apple/mac_desktop/build` and `apple/iphone/build` when the task touches those targets

End of every run:
- `git status --short` must be empty again
- repo-local build artifact trees created by `xcodebuild` must be removed before the final cleanliness check

No new task may begin on a dirty tree.
The `Dirty-Tree Refinement` exception applies only after a task has already started.
No task is complete until the tree is clean again.

## Network Stability Rule
- `🤖 ChatGPT` is locked to `🇸🇬 新加坡 01`.
- Codex must not change Clash, macOS proxy settings, git config, ssh config, shell proxy variables, or network settings unless JD explicitly approves it first.
- If connectivity fails, Codex must stop and report the failure instead of attempting fixes.

## Task-State Resolution, Recovery, and Silent Execution Protocol
Before any substantial work, Codex must classify the task into exactly one of:
- `FRESH_AUTHORING`
- `PARTIAL_AUTHORING_RECOVERY`
- `EDIT_EXISTING_FILE`
- `READ_ONLY_AUDIT`
- `PUBLICATION_RECOVERY`

### Fresh Authoring Rule
Use `FRESH_AUTHORING` only when:
- the target file does not exist
- the target file is not already untracked
- there is no partial-authoring state for the target

### Partial Recovery Rule
If the target file exists and is the only relevant modified or untracked file for the task, Codex must switch to `PARTIAL_AUTHORING_RECOVERY` instead of stopping.

### Read-Only Audit Rule
If the target file already exists and the task is verification or approval review, Codex must switch to `READ_ONLY_AUDIT` instead of using a fresh-authoring flow.

### Existing-Target Rule
Target existence is a stop condition only for `FRESH_AUTHORING`.
It is not a stop condition for:
- `PARTIAL_AUTHORING_RECOVERY`
- `EDIT_EXISTING_FILE`
- `READ_ONLY_AUDIT`
- `PUBLICATION_RECOVERY`

### Dirty-Tree Refinement
This is the only narrow exception to the `Clean Tree Rule`.
A non-empty `git status --short` must not cause an automatic stop when:
- the task has already started
- the only changed or untracked file is the task target file

Automatic stop is still required when:
- unrelated files have changed
- the repo is wrong
- the branch is wrong
- remote reachability is required and unavailable

### Single-Preflight Rule
- Run preflight once only per task.
- Do not loop preflight.
- Do not repeat target-file existence checks over and over.
- Do not repeat remote-reachability checks over and over.
- If repo state changes during the task, do one reconciliation pass only.

### Silent Execution Rule
- Do not emit repeated progress narration.
- Do not emit repeated "I'm about to..." messages.
- After preflight, complete the work and return one final completion pack.
- If blocked, stop once with one concrete blocker.

### Repo-Truth Precedence Rule
If task instructions conflict with current repo truth, Codex must follow current repo truth plus these task-state rules and report the mode switch once in the final output.

### Build-Report Hash and Ownership Verification Rule
Before any final completion report, self-authored build instruction, audit conclusion, or corrective follow-up that names:
- a landed build hash
- a pinned `HEAD == ...` value
- a prior or prerequisite build hash
- an exact symbol carrier
- an exact file or layer ownership claim

Codex must verify that claim from current repo truth in the same task.

Required verification:
- current landed commit hash: `git rev-parse HEAD`
- prior or prerequisite build hash: direct `git rev-parse`, `git log`, or exact ancestor query from the current repo in the same task
- every cited prior landed build hash in a self-authored build instruction: exact same-task git query that maps the cited build identifier to the current repo hash, such as exact `git log --format=%H --grep='^H253:' -n 1` or an exact ancestor query
- exact symbol or state ownership: direct `rg` or equivalent against the exact named files in the same task
- if a symbol exists in both bridge and shell or across multiple layers, Codex must report the split ownership exactly rather than collapsing it into one carrier

Forbidden:
- reusing remembered full hashes from earlier assistant messages
- inferring exact ownership from summary memory when the repo can be queried directly
- collapsing bridge-owned canonical fields and shell-owned derived state into one carrier when repo truth shows a split
- relying on current `HEAD == ...` alone while citing preserved prior landed builds without verifying their exact landed hashes in the same task
- stating an exact hash or ownership claim as definitive without same-task verification

If verification cannot be completed:
- stop and report the claim as `unverified`
- do not pin a self-authored build instruction or final report to that claim

### Verification Clause Specificity Rule
If a self-authored verification clause is intended to prove that the current task did not add a forbidden symbol, behavior, or route, Codex must scope that clause to the current task slice rather than blindly scanning preserved whole-file history when same-task repo truth shows that one or more forbidden symbols already exist outside the current edit.

Required verification discipline:
- if a forbidden symbol check targets edited files that already contain preserved historical symbols, Codex must use a diff-scoped check such as exact `git diff --unified=0 -- <task files> | rg ...` or an equally narrow task-slice verification
- if the task adds only bounded slices inside larger files, Codex must pair positive slice-presence checks with narrow forbidden diff checks rather than whole-file negative greps
- if preserved symbols already exist in the target file, Codex must say so explicitly and must not present a broad whole-file grep failure as if it were caused by the current task

Forbidden:
- using a whole-file negative grep as the decisive verification clause when same-task repo truth already shows preserved conflicting symbols in that file outside the task slice
- treating inherited symbols in unchanged lines as evidence that the current task violated scope
- writing a self-authored verification clause that cannot distinguish pre-existing repo truth from the current diff when the distinction is discoverable

### Exact Test Execution Verification Rule
If Codex cites an exact test command as proof in a self-authored build instruction, audit, corrective follow-up, or final completion report, Codex must verify both the exact runnable test name and the fact that a nonzero test count actually executed.

Required verification:
- verify the exact runnable test name from current repo truth in the same task with a runner-native listing command such as `cargo test ... -- --list` when the exact test path could be ambiguous
- after execution, verify that the test run executed a nonzero test count such as `running 1 test`; a result with `0 passed; 0 failed` or `N filtered out` does not count as successful exact-test verification
- if the exact test name in task instructions conflicts with current repo truth, Codex must follow current repo truth, report the mismatch once, and update the self-authored verification command to the exact runnable name
- if an exact test fails, Codex must distinguish between target-slice regressions and inherited prerequisite or contract mismatches before concluding that the task itself failed

Forbidden:
- treating an exact test command that ran zero tests as a successful verification
- reusing remembered exact test names without same-task confirmation when the repo can list them directly
- blaming the current target slice for an exact-test failure before checking whether the failure is caused by an inherited prerequisite, stale verification clause, or conflicting upstream repo truth

### Authority Conflict Stop Rule
If current repo artifacts that should describe the same truth conflict with each other, Codex must stop and report the exact conflict with file-and-line evidence.

Examples:
- code vs plan doc
- master plan vs phase plan
- ledger vs completion claim
- test name in prompt vs test name in repo

Codex must not silently synthesize a new truth when repo authorities disagree.
Codex must report:
- the conflicting files
- the exact conflicting lines
- which authority order applies if it is already resolved by existing law
- whether work is blocked or can continue under existing authority order

### Completion Discipline
At the end of each task, Codex must explicitly state whether the target was:
- authored
- recovered
- audited
- committed/pushed
- freeze-ready or not

### Strict No-Op Rule
`NO_OP` is allowed only when the existing artifact already matches the full intended truth or behavior for the task.

`NO_OP` is not allowed merely because:
- a file already exists
- a heading already exists
- a partial implementation already exists
- a similarly named test already exists
- some of the requested truth is already present

If full no-op equivalence is not proved from current repo truth, Codex must stop and report exact repo state instead of claiming `NO_OP`.

### No Hidden Mode Switching
If a task begins as authoring but repo truth shows the target already exists, Codex must explicitly switch to recovery or audit mode instead of silently half-following the original authoring prompt.

### No Progress-Spam in Compacted Sessions
If context compaction happens, Codex must resume from current repo truth, not from the original fresh-authoring assumption.

### Deadlock Prevention and Next-Move Override
If the same Section/item receives two consecutive read-only deadlock, `NOT_EXPLICIT`, or no-new-truth audits without any repo-truth change:
- further audits of that same frontier are forbidden until repo truth changes
- Codex must switch from repeat-audit mode to next-move selection mode

Next-target precedence:
- highest priority: `PARTIAL` item with an explicit live code carrier and a direct unimplemented, ignored, or dormant seam in current source
- next priority: `PARTIAL` item with a broader live seam
- lowest priority: `NOT_EXPLICIT` item

A `NOT_EXPLICIT` item cannot outrank a live `PARTIAL` item with a direct source seam.

If one target clearly outranks the others:
- Codex must author the next build plan for that target instead of running another deadlock audit

If a genuine tie remains after applying the precedence rule:
- Codex must produce one short decision memo for JD approval
- Codex must not continue looping audits

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
- `nl`
- `diff`
- `printf`
- `test`
- `cut`
- `sort`
- `uniq`
- `wc`

Forbidden:
- `python`
- `python3`
- `perl`
- `node`
- `php`
- `lua`
- any scripted workaround that bypasses this law

Exception:
- ruby only for bounded read-only extraction or line-order verification when a task instruction explicitly requires Ruby
- ruby is not allowed for editing, code generation, or scripted workarounds

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

## Approval Token Requirement
- For Class B or Class C edits, Codex must have explicit JD approval in-thread before editing.
- Missing approval token => stop.

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

## Conflict / Upgrade Workflow for Prior Code
If existing code must change:
- raise change request
- require explicit JD approval before edits
- apply minimal patch
- run expanded proof
- report before/after behavior and residual risk

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
- warnings must be fixed, not ignored, unless explicitly approved by JD
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
- baseline commands passed
- targeted tests passed
- required contract / reason / idempotency / state-machine checks passed where applicable
- determinism checks passed where applicable
- `git diff --stat`
- no unrelated core files modified

## Subjective-Gate Ban
Task gates, stop conditions, and completion claims must not rely on subjective phrases such as:
- `strongly implies`
- `appears`
- `seems`
- `as if`
- `looks like`

Codex must use:
- exact searchable text
- exact command output
- exact file-and-line evidence
- exact counted hits
- exact named tests/checks

If a gate cannot be expressed objectively, Codex must stop and report the ambiguity instead of guessing.

## No Vacuous Test Pass Rule
A test, lint, or check only counts as proof if both conditions are true:
1. the named target is explicitly proved to exist in current repo truth
2. the command output proves that at least one matching test or check actually executed

Passing with zero matched tests or zero executed checks does not count as proof.

When the tool supports exact targeting, Codex should use exact targeting.
If execution count cannot be proved from output, Codex must report the result as unproven.

## Real-Path Proof Rule
For any change run, Codex must state in the final proof pack:
- the exact command run
- the exact module, file, function, or runtime path exercised
- what was real
- what was mocked, stubbed, simulated, in-memory, or fixture-backed
- what remains unproven

Codex must not present narrow harness proof as broader production proof.

## Harness Disclosure Rule
If proof depends on a harness, fixture, fake clock, in-memory store, mock, stub, replay harness, or simulation layer, Codex must name that dependency explicitly in the final report.

Codex must also state the proof scope honestly:
- what the harness proves
- what it does not prove
- whether the exercised path is real wiring, partial wiring, or harness-only

## Fresh Remote Truth Rule
Remote branch state, publication state, and `HEAD == origin/main` may be reported only from a successful fetch performed in the same run.

If that fetch fails:
- remote publication state must be reported as `UNVERIFIED`
- Codex must not claim remote equality from stale local refs

If the fetch succeeds, Codex may report:
- local `HEAD`
- remote `origin/main`
- whether they are equal

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
This AGENTS.md is the only active repository law file.
CODEX_LAW.md has been retired and must not be recreated unless JD explicitly authorizes it.

## Override
These laws apply by default unless JD explicitly overrides them in-thread.
