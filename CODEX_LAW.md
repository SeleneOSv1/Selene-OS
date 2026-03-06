# CODEX LAW (ENFORCED)

This file is the binding execution policy for Codex in this repository.
If any instruction conflicts with this file, this file wins unless JD explicitly overrides it.

## Hard Rule 0) Absolute Ban: Python
- Never use Python in this repository.
- Disallowed: `python`, `python3`, `uv run python`, inline Python blocks, and any script that invokes Python.
- If a task would require Python, stop and report exactly:
  - `Python is disallowed by CODEX_LAW.md`

## Repository Locality Rule
- Single Codex session: always use the main repo path only.
- Main repo path: `/Users/xiamo/Documents/A-Selene/Selene-OS`.
- Never create worktrees or extra repo folders for this repository.

## Architectural Boundary: Probabilistic Reasoning vs Deterministic Execution
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

## 13 Core Rules
1. Authority
- JD is the only authority for breaking or modifying shared existing behavior.

2. Default-Deny for Existing Code
- Existing files are read-only by default.
- Additive new files/modules/tests are allowed by default.
- Editing existing files requires explicit file-scope approval.
- This default-deny governs change runs; read-only/reporting runs must keep files untouched and end clean.

3. Spine Lock (High Risk Core)
- Spine code is immutable without explicit JD approval.
- Spine includes (non-exhaustive): PH1.X, PH1.OS, PH1.L, SimulationExecutor, access control, shared dispatch/mutation layers, packet contracts, reason registry, idempotency registry, state machine, turn ordering.

4. Contract Surface Lock
- No removals/renames/semantic changes to shared fields, enums, packet meaning, gate order, state order, idempotency behavior without explicit JD approval.
- Backward-compatible additive changes only by default.

5. File Scope Gate
- Before coding, Codex must declare exact files to be modified and why.
- No touching files outside the declared list.
- If extra files are needed: stop and ask JD first.

6. Baseline Gate
- For change runs, run baseline tests for the affected domain before edits.
- If baseline is red on a change run: stop immediately, report baseline failure, and do not code until green.

7. Change Class Gate
- Class A: additive new modules only -> allowed.
- Class B: edit non-spine existing files -> requires JD approval.
- Class C: spine/contract/order/idempotency changes -> explicit JD approval + expanded proof.

8. No Surprise Refactors
- No unrequested renames, file moves, cleanup sweeps, broad formatting churn, or convenience rewrites.

9. Determinism Lock
- Stable ordering only.
- No randomness, hidden fallbacks, time-based drift in deterministic paths.
- Same input must produce same output unless explicitly intended and approved.

10. Fail-Closed Lock
- Missing required inputs must refuse with registered reason code.
- No silent correction, no guessing, no hidden continuation.

11. No Parallel Bypass Paths
- Execution must stay within canonical boundaries:
  - PH1.X classification
  - access control
  - simulation verification (when applicable)
  - SimulationExecutor
  - deterministic mutation
  - audit logging
- No shortcuts or bypass logic.

12. Post-Change Proof Gate
- Must run:
  - targeted feature tests
  - relevant spine protection tests
  - contract checks
  - reason-code registry checks
  - idempotency checks
  - state-machine checks
- Must show `git diff --stat`.
- Must confirm no unrelated core files changed.

13. Stop-on-Uncertainty
- If uncertain about contract meaning, state order, gate sequencing, simulation boundaries, or engine interaction: stop and ask JD.
- Guessing is forbidden.

## Permanent Rule: Engineering Quality Standard
- All code delivered by Codex must aim for production-grade quality.
- Warnings must be fixed, not ignored, unless explicitly approved by JD.
- Do not use shortcuts, lazy fixes, temporary hacks, or workaround-style patches when a proper implementation is reasonably achievable within scope.
- Prefer the most efficient, accurate, maintainable, and fully-wired solution that fits Selene's architecture.
- Preserve maximum required functionality; do not silently reduce scope to make a patch easier.
- If a proper implementation is too large for the current run, Codex must state that clearly and stop at a clean architectural boundary rather than shipping a weak partial workaround.
- Clean builds are the target standard; warning-free is the default expectation.

Enforcement note:
- Before final commit, Codex should run relevant checks and treat warnings as issues to resolve where practical and in-scope.

## 5-Step Process for Conflicts / Upgrades to Prior Code
1. Raise Change Request
- Declare conflict and why existing code must change.
- List exact target files and invariants at risk.

2. Wait for Explicit JD Approval
- No edits until JD approves the change request.
- Approval must be explicit in-thread.

3. Apply Minimal Patch
- Change the smallest possible surface.
- Preserve old behavior wherever not explicitly changed.

4. Run Expanded Proof
- Baseline + targeted + full required checks for impacted domains.
- Include determinism and fail-closed verification.

5. Report Before/After + Risk
- Provide old behavior vs new behavior summary.
- Provide residual risk and rollback note.

## Enforcement (Operational and Checkable)
### A) Required Pre-Change Declaration (must be posted before edits)
- `Scope:` absolute file list
- `Class:` A/B/C
- `Spine touch:` yes/no
- `Contract touch:` yes/no
- `Baseline commands:` exact list

### B) Approval Token Requirement
- For Class B/C, Codex must have explicit JD approval in-thread before editing.
- Missing approval token => stop.

### C) File-Scope Lock
- `git diff --name-only` must be a subset of declared scope.
- Any out-of-scope file => stop and request approval.

### D) Mandatory Proof Checklist Before Commit
- Applies when the run includes file edits (change run).
- Baseline commands passed
- Targeted tests passed
- Required contract/reason/idempotency/state-machine checks passed
- Determinism checks passed
- `git diff --stat` presented
- No unrelated core files modified

### E) Mandatory Cleanliness Gates
- Clean tree at start and end (`git status --porcelain` empty).
- Start of every run: `git status --porcelain` must be empty. If not, stop and resolve it first.
- End of change runs: tests/checks must pass, append a PASS line to `docs/03_BUILD_LEDGER.md`, commit only the run files plus the ledger entry, push to `origin`, then verify `git status --porcelain` is empty again.
- End of read-only/no-change runs: no commit and no ledger append; provide clean-tree proof (`git status --porcelain` empty).
- End-of-task is not complete until the applicable end condition above is satisfied.
- A new task must not start until the previous task is pushed and the tree is clean again.
- No local-only real work.
- No untracked leftovers.
- If violated, stop and resolve before continuing.

### F) Hard Fail Conditions (Immediate Stop)
- Attempted Python usage
- Missing required approval for B/C changes
- Baseline failing before edits on a change run
- Out-of-scope file touched
- Unknown reason code introduced
- State/gate ordering changed without approval

## Commit Message Guidance
- Include run id and intent.
- Keep one coherent changeset per run unless JD explicitly asks otherwise.

## Non-Negotiable Safety Outcome
- No hidden drift.
- No silent corruption.
- No contract breakage without explicit JD approval.

## Override
- These laws apply by default unless JD explicitly overrides them.
