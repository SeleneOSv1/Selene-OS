SELENE AGENTS LAW (AUTO-LOADED)

This file is the active auto-loaded execution law for Codex in this repository.

If any instruction conflicts with this file, this file wins unless JD explicitly overrides it in-thread.

0. Authority Order

Codex must apply this authority order:

JD explicit in-thread instructions.

This AGENTS.md.

Repository architecture law and docs.

Task-specific prompt scope.

JD may explicitly override this file in-thread. Without an explicit JD override, this file controls Codex behavior.

AGENTS.md is the only active repository law file. CODEX_LAW.md has been retired and must not be recreated unless JD explicitly authorizes it.

1. Absolute Tool and Repository Rules

1.1 Absolute Rule 0 — No Python

Never use Python in this repository.

Disallowed:

python

python3

uv run python

inline Python

any script that invokes Python

If a task would require Python, stop and report exactly:

Python is disallowed by AGENTS.md

1.2 Shell-Only Inspection Rule

Allowed inspection commands:

git

rg

grep

sed

awk

cat

head

tail

find

ls

nl

diff

printf

test

cut

sort

uniq

wc

Forbidden:

python

python3

perl

node

php

lua

any scripted workaround that bypasses this law

Exception:

ruby is allowed only for bounded read-only extraction or line-order verification when a task instruction explicitly requires Ruby.

ruby is not allowed for editing, code generation, or scripted workarounds.

If Codex violates this shell-only rule:

stop immediately;

report the violation;

discard that line of investigation;

restart with shell-only inspection.

1.3 Repository Locality Rule

Work only in the checked-out main Selene repository that contains this AGENTS.md.

If any prompt or older machine-specific instruction names a different absolute path, current repo truth wins.

Never create worktrees.

Never create extra repo folders.

Never move work into side repositories.

1.4 Network Stability Rule

ChatGPT is locked to Singapore 01.

Codex must not change Clash, macOS proxy settings, git config, ssh config, shell proxy variables, or network settings unless JD explicitly approves it first.

If connectivity fails, Codex must stop and report the failure instead of attempting fixes.

2. Project Phase and Execution Lanes

2.1 Current Project Phase

Selene is currently in PROBABILISTIC_FOUNDATION_BUILD mode unless JD explicitly authorizes SIMULATION_EXECUTION_BUILD.

During PROBABILISTIC_FOUNDATION_BUILD, Selene’s normal assistant capabilities are probabilistic/advisory by default.

This includes:

normal chat

public Q&A

public web search

public news/research

time and weather answers

explanations

summaries

translations

language matching

drafting

analysis

advisory reports

report templates

provider-degraded public answers

STT interpretation

TTS answer delivery

non-mutating public/research output

These do not require simulation approval.

Protected business execution remains deterministic and simulation-gated.

Protected business execution includes:

payroll execution

salary changes

leave approval

access control changes

database writes

business state mutation

official company operations

customer/company record changes

protected financial, HR, POS, inventory, or authority-gated actions

No Simulation -> No Execution means no protected/business execution. It does not mean no public answer.

Selene may think, explain, search, summarize, translate, draft, analyze, and answer probabilistically.

Selene may only execute protected business operations deterministically through approved simulations.

2.2 Lane 1 — PROBABILISTIC_PUBLIC_ANSWER

This is Selene’s default lane during PROBABILISTIC_FOUNDATION_BUILD.

This lane applies to:

normal conversation

public Q&A

public web search

public news/research

time and weather answers

public explanations

summaries

translations

language matching

drafting

analysis

advisory reports

report templates

provider-degraded public answers

STT interpretation

TTS answer delivery

non-mutating public/research output

Rules:

simulation is not required;

authority approval is not required;

LLM/probabilistic reasoning is allowed;

public web/search providers are allowed;

read-only public tools are allowed;

degraded provider metadata is allowed;

if one public provider fails, Selene may still answer using available public/advisory sources;

normal answers must not be blocked only because no simulation exists;

normal public research must not be blocked only because a secondary provider such as GDELT is degraded;

this lane must not mutate business data;

this lane must not execute protected business actions;

this lane must not claim official deterministic execution.

Examples:

“Tell me a joke.”

“What is GDELT?”

“Search latest public climate news.”

“What is the weather today? Keep it short.”

“Translate this into Chinese.”

“Summarize this document.”

“Prepare a profit and loss report template.”

“Analyze these numbers and draft a P&L summary.”

“Explain payroll rules generally.”

2.3 Lane 2 — DETERMINISTIC_PROTECTED_EXECUTION

This lane applies only when Selene is asked to execute a protected business operation or mutate authority-gated business state.

This lane applies to:

payroll execution

salary changes

leave approval

access control changes

database writes

business state mutation

official company operations

customer/company record changes

protected financial, HR, POS, inventory, or authority-gated actions

Rules:

simulation is required;

authority validation is required;

audit is required;

deterministic workflow is required;

fail closed if simulation is missing;

fail closed if authority is missing;

no guessing;

no improvising;

no protected execution through the public answer lane.

Examples:

“Approve payroll for Tim.”

“Increase Tim’s salary.”

“Submit leave approval.”

“Update inventory.”

“Post this invoice.”

“Change customer account status.”

“Run the official monthly P&L from company accounting records.”

2.4 Lane 3 — MIXED_REQUEST

Some user requests contain both a public/advisory part and a protected execution part.

Example:

“Search salary trends and increase Tim’s salary.”

Correct behavior:

salary-trend research is handled in PROBABILISTIC_PUBLIC_ANSWER;

salary increase is handled in DETERMINISTIC_PROTECTED_EXECUTION;

the public answer may proceed;

the protected execution must fail closed unless an approved simulation and authority exist.

Wrong behavior:

blocking the whole public answer because one part is protected;

executing the protected action because the public part is allowed.

2.5 Intent and Execution Clarification

Public conversational intent understanding is probabilistic.

Protected execution intent dispatch is deterministic.

Answering is not protected execution unless it performs a protected business action, mutates state, sends an external real-world message, or uses authority-gated company systems.

Language may be probabilistic.

Public answers may be probabilistic.

Protected execution must never be probabilistic.

2.6 Data Authority Rule

The probabilistic public-answer lane may use:

public data

user-provided text or files

uploaded documents

non-authoritative examples

advisory calculations

draft reports

public provider results

user-supplied numbers or documents for advisory analysis

The deterministic protected-execution lane is required when Selene uses or mutates:

official company databases

payroll records

accounting systems

customer records

inventory or POS systems

HR systems

bank/payment systems

authority-gated business data

Examples:

“Draft a P&L template.”Lane: PROBABILISTIC_PUBLIC_ANSWER

“Analyze these uploaded numbers and draft a P&L summary.”Lane: PROBABILISTIC_PUBLIC_ANSWER

“Run the official company P&L from accounting records.”Lane: DETERMINISTIC_PROTECTED_EXECUTION

2.7 Draft vs Official Execution Rule

Selene may probabilistically draft, explain, summarize, estimate, format, or analyze.

Selene may not claim something is official, posted, approved, submitted, filed, paid, updated, changed, executed, or completed unless an approved deterministic simulation performed that action with required authority and audit.

Examples:

“Draft a report.”Lane: PROBABILISTIC_PUBLIC_ANSWER

“Submit the report to directors.”Lane: DETERMINISTIC_PROTECTED_EXECUTION

“Explain payroll rules.”Lane: PROBABILISTIC_PUBLIC_ANSWER

“Approve payroll.”Lane: DETERMINISTIC_PROTECTED_EXECUTION

## Selene Conversation-to-Action Guardrail — All Engines

This rule applies to every Selene document, engine, workflow, simulation, packet description, and future implementation.

Selene must not rely on fixed phrase matching for human, customer, company, supplier, employee, recipient, or board-member language.

Natural language understanding belongs to GPT-5.5 with Selene context support:
- GPT-5.5 interprets natural speech and typed text.
- GPT-5.5 repairs messy spelling, unclear speech, partial phrases, and human wording.
- PH1.X resolves live context such as “that one,” “send it to mum,” “same as before,” “the blue one,” “continue,” “that invoice,” or “the supplier from yesterday.”
- PH1.M and relevant Customer, Company, Supplier, Finance, Product, Inventory, Order, or Relationship memory resolve durable context such as preferences, habits, company relationships, usual payment methods, addresses, orders, suppliers, accounts, and prior actions.

Natural language interpretation is not execution authority.

Before any action, the relevant deterministic Selene engine must verify the required truth, permissions, policy, and evidence.

Examples:
- E-Commerce verifies product, variant, price, stock, payment permission, address, delivery, return, warranty, and audit.
- Accounting verifies ledger rules, posting authority, tax treatment, period status, evidence, and audit.
- AP verifies supplier, invoice, PO, receiving, credit notes, holds, bank safety, and payment readiness.
- Supplier Payment verifies AP readiness, bank safety, cashflow, authority, payment rail, settlement, and audit.
- Inventory verifies product identity, accepted stock, location, batch, expiry, reservation, and movement truth.
- Governance verifies identity, role authority, quorum, voting rights, policy, and audit.

Protected actions require the appropriate step-up verification or authority, including passkey, biometric/device verification, secret passcode, role authority, approval route, board vote, or other configured control.

GPT-5.5 may explain, draft, summarize, compare, recommend, translate, and speak naturally.

GPT-5.5 must not approve, pay, post, change records, alter stock, change supplier bank details, execute protected actions, invent facts, override policy, bypass authority, or replace audit evidence.

Selene must reply naturally in the user’s active mode:
- typed input normally receives typed response
- voice input normally receives voice response
- visual display may be used when helpful

Everything important must be auditable.

Every new Selene engine document must include or explicitly reference this Conversation-to-Action Guardrail.

## Selene Human / External Action Orchestration Law — All Engines

This rule applies to every Selene document, engine, workflow, simulation, packet description, and future implementation.

Selene must not use vague standalone phrases such as “notify user,” “notify Accounts,” “tell supplier,” “remind receiver,” “escalate,” “send message,” or “inform manager” unless the document defines the action orchestration path.

Any workflow requiring a human, supplier, customer, courier, approver, receiver, AP user, manager, employee, external party, or system operator must create or reference an action orchestration record.

Every human or external action must define:
- action type
- owner
- recipient
- backup owner where needed
- authority / access requirement
- schedule / due time
- delivery method
- required confirmation
- required evidence
- reminder rule
- escalation path
- closure condition
- audit reference

Action types include:
- informational only
- acknowledgement required
- action required
- approval required
- correction required
- scheduled operational task
- critical exception
- external party response required

The engine that detects the need for the action owns the action requirement, but must hand off execution to the correct Selene engines.

Required orchestration engines:
- Access / Authority verifies whether the person or party is allowed to act.
- Task / Human Workload assigns work to the correct responsible person or team.
- Scheduler / Rosters verifies availability, timing, workload, location, and due dates.
- Broadcast / Delivery sends the message, request, approval, correction, or confirmation demand.
- Reminder follows up until the action is completed, rejected, expired, or escalated.
- Audit records proof of delivery, acknowledgement, confirmation, action, escalation, and closure.

When a person lacks permission, Selene must follow Master Access and Per-User Access rules. Where policy allows, Selene should quietly route the access or approval request to the correct authority instead of hard-denying immediately. If authority approves, Selene proceeds. If authority rejects, Selene informs the requester politely.

For supplier, courier, customer, or external-party corrections, Selene must not merely notify. Selene must request the required correction, require confirmation, track response, remind if overdue, escalate if ignored, and keep AP / Procurement / Receiving / affected engines protected until the correction closes.

Examples:
- “Notify Accounts” must become an AP action with owner, delivery, confirmation requirement, due time, reminder, escalation, and audit.
- “Tell supplier they delivered short” must become a supplier correction workflow requiring corrected invoice, credit note, replacement, refund, or dispute response.
- “Receiver must prepare freezer space” must become a scheduled receiving-readiness task with receiver owner, backup owner, due time, confirmation, reminder, escalation, and audit.
- “Escalate delivery delay” must define who receives escalation, by when, what decision is needed, and what happens if no one responds.

Every new Selene engine document must include or explicitly reference this Human / External Action Orchestration Law whenever it mentions notifications, reminders, approvals, escalations, corrections, scheduled tasks, confirmations, or human/external actions.

2.8 Tool Retrieval and Provider Degradation Rule

Calling a read-only public tool/provider for time, weather, news, web search, translation, public research, or public facts is not protected simulation execution.

The factual retrieval may be deterministic or tool-based.

The answer presentation remains probabilistic and adaptive.

Provider failure must degrade the answer, not convert the request into protected execution failure.

Rules:

read-only public provider calls do not require simulation;

provider failure may produce a degraded public answer;

provider failure must not trigger protected governance failure by itself;

provider results must not be treated as authority to mutate business data;

public tool retrieval must not claim official business execution.

Examples:

Time lookup = read-only provider/tool retrieval plus adaptive answer presentation.

Weather lookup = read-only provider/tool retrieval plus adaptive answer presentation.

Public news search = read-only provider/tool retrieval plus probabilistic summary/citation behavior.

GDELT unavailable = corroboration degraded, not public answer blocked.

2.9 Probabilistic Presentation and Preference Adaptation Rule

Read-only public information may come from tools, providers, or public data sources, but how Selene understands, explains, summarizes, formats, and presents that information belongs to the probabilistic public-answer lane.

This applies to:

time answers

weather answers

public news summaries

public web/search answers

public research answers

explanations

report drafts

language matching

short or long answer preference

user-requested tone, structure, or format

remembered presentation preferences where memory law allows

Rules:

Selene must listen to how the user asks for the answer.

If the user says “keep it short,” Selene should give a short answer.

If the user asks for detail, Selene may give more detail.

If the user asks in Chinese, Selene should answer in Chinese unless safety, policy, or explicit context requires otherwise.

If the user prefers weather in a certain format, Selene should remember and reuse that preference where memory law allows.

If the user prefers time in 12-hour or 24-hour format, Selene should follow that preference where available.

Provider/tool output supplies facts; Selene’s explanation and presentation are probabilistic and adaptive.

This does not require simulation.

This must not mutate business data.

This must not execute protected business actions.

This must not claim official deterministic execution.

Examples:

User: “What’s the weather today? Keep it short.”Correct: short weather answer.Wrong: fixed long deterministic weather report.

User: “Explain the news to me and keep it short.”Correct: short public news summary.Wrong: rigid fixed report format.

User: “From now on, give me weather in short bullet points.”Correct: remember this as a presentation preference if memory law allows.

User: “What time is it? Use 24-hour time.”Correct: answer using the requested time format where available.

Important distinction:

fetching time, weather, public news, or search data may use a deterministic tool/provider call for factual retrieval;

presenting the answer remains probabilistic and user-adaptive;

protected/business execution remains deterministic and simulation-gated.

2.10 Preference Memory Rule

Selene may adapt to user presentation preferences in-session.

Persistent memory of preferences requires the approved memory layer and memory law.

Examples:

“Keep this answer short.”Apply now in the current answer.

“From now on, give weather in short bullet points.”Store only if memory law allows.

“Use 24-hour time for me.”Store only if memory law allows.

Rules:

presentation preferences do not create protected execution authority;

presentation preferences do not convert advisory output into official execution;

in-session adaptation is allowed in the probabilistic lane;

persistent preference storage must follow memory law.

2.11 Simulation Transition Authorization Rule

A probabilistic capability does not become deterministic merely because Codex thinks it should.

A process moves into deterministic protected execution only when JD explicitly authorizes simulation-building work for that exact process and the repo contains or is being given an approved simulation/catalog entry for that process.

Until then, Selene may advise, draft, explain, summarize, calculate, format, analyze, and research, but must not execute protected business operations.

Rules:

Codex must not silently convert probabilistic features into deterministic simulations.

Codex must not add simulation gates to public/advisory behavior unless JD explicitly authorizes simulation work.

Codex must not claim a process is deterministic because the topic sounds business-related.

Official execution requires explicit simulation authorization.

Examples:

“Prepare a P&L report” remains probabilistic/advisory until JD authorizes the official P&L simulation.

“Run official payroll” remains blocked until JD authorizes and wires the payroll simulation.

“Explain inventory turnover” remains probabilistic.

“Update inventory count” is deterministic protected execution.

2.12 Mandatory Codex Lane Declaration

Every Codex build must declare its lane before editing.

Required declaration:

LANE DECLARATION:
current project phase: PROBABILISTIC_FOUNDATION_BUILD or SIMULATION_EXECUTION_BUILD
selected lane: PROBABILISTIC_PUBLIC_ANSWER / DETERMINISTIC_PROTECTED_EXECUTION / MIXED_REQUEST
simulation required: yes/no
authority required: yes/no
state mutation allowed: yes/no
protected execution allowed: yes/no
provider degradation allowed: yes/no
normal answer allowed: yes/no
fail-closed required: yes/no and for which part

Codex must stop before editing if it cannot classify the lane.

2.13 Standing Stop Rules for Lane Misclassification

Stop if Codex:

applies deterministic simulation gates to ordinary public chat/search/research without JD explicitly authorizing deterministic execution work;

blocks harmless public answers because no simulation exists;

blocks harmless public answers because an unrelated provider degraded;

weakens protected/business fail-closed behavior;

mixes public answer and protected execution without separating them;

allows protected execution through the probabilistic lane;

treats provider failure as protected execution failure;

claims deterministic execution for advisory/probabilistic output;

converts a probabilistic capability into deterministic simulation work without JD authorization.

3. Build Operating Mode

3.1 Solo Shipping Mode

Selene is currently a solo-build project operated by JD with Codex assistance.

Execution law must optimize for shipping working product quickly without lowering engineering quality.

Default operating rule:

implementation-first is the default;

in this workflow, build always means implementation run;

next build means next implementation;

docs-only reconciliation is not a build unless JD explicitly says to do docs-only work;

publication-only work is not a build unless JD explicitly says to do publication-only work;

docs-only runs are exception-mode, not default;

Codex must prefer the maximum safe, accurate, provable code-changing progress that can be completed in one run when a safe implementation seam exists;

Codex should batch adjacent implementation slices when they share the same files, subsystem, invariant family, and proof pack, and when batching does not reduce safety, accuracy, or verification clarity;

Codex should fall back to the next smallest safe slice only when batching would increase semantic risk, widen approval boundaries, or weaken proof quality;

Codex must not create publication-only, frontier-only, or boundary-restatement runs unless JD explicitly asks for docs-only work.

Required behavior in Solo Shipping Mode:

first priority is shipping working behavior;

second priority is proving that behavior with targeted tests;

third priority is updating only the minimum docs needed to keep repo truth accurate.

Docs are tail work after code by default, not the main output.

Investigation limits:

Codex must time-box investigation and repo-audit work.

If a safe implementation seam is visible, Codex must move to implementation instead of continuing analysis churn.

If multiple adjacent safe implementation slices are visible, Codex should batch them into one implementation run instead of artificially splitting them into smaller runs.

If no safe implementation seam is visible after a bounded inspection pass, Codex must stop and report the blocker in plain language.

Codex must not manufacture a docs-only build just because a code choice is not yet perfect.

Quality floor:

Solo Shipping Mode does not relax determinism, contract safety, idempotency, auditability, file-scope approval, or test proof requirements.

Speed is gained by removing ceremony, not by skipping engineering discipline.

Clarification:

Implementation-first does not override explicit approval gates, file-scope gates, clean-tree gates, spine locks, contract locks, or baseline-test gates.

3.2 Build Selection Rule

For self-authored next-build instructions, Codex must default to an implementation build.

Every self-authored build instruction must:

treat build as implementation;

declare or assume Build Class: IMPLEMENTATION;

name the exact behavior to change;

name the exact files expected to change;

name the exact tests that will prove the change.

Every self-authored build instruction that pins HEAD, origin/main, or a prerequisite landed-build hash must source that exact hash from same-task git output and must not reuse remembered prior assistant text.

Every self-authored build instruction that cites preserved prior landed builds must include explicit same-task git verification commands for each cited prior landed build hash. Pinning current HEAD alone is insufficient.

Every self-authored build instruction that runs xcodebuild must clean the relevant repo-local build artifact tree before the first clean-tree check and again after each xcodebuild before final cleanliness verification.

Codex should prefer the largest safe, accurate, and provable in-scope implementation batch over a smaller batch when both are lawful and the larger batch stays within the same approval surface.

Codex must not:

default to one-slice-only execution when multiple adjacent slices can be implemented and proven together safely;

answer a next build request with a docs-first, docs-only, publication-only, frontier-only, or authority-catch-up-only run;

author a docs-only next build when a safe implementation build or safe implementation batch is available;

author a docs-only next build while any safe in-scope implementation slice or batch remains.

If no safe implementation slice is exposed by current repo truth, Codex must stop and report no lawful implementation build exposed by current repo truth instead of substituting docs work.

3.3 Anti-Ceremony Rule

Codex must not:

split one real implementation outcome into multiple paperwork-only runs;

create separate publication steps for truths that can be recorded as part of the implementation run;

keep refining frontier language when the product has not changed.

3.4 Docs Tail Rule

Default is no doc change.

Docs may be updated only as a minimal tail on the same implementation run.

Docs may be touched only when the implementation materially narrows current repo truth and the wording can be updated without overclaiming.

If JD does not explicitly ask for docs reconciliation, Codex must never select docs work as the main output of a build.

3.5 Next-Build Stop Rule

If the candidate next run is docs-only, publication-only, or master-doc reconciliation-only, it is not a lawful answer to next build.

If code truth is ahead of docs and a safe implementation slice still exists, Codex must select the implementation slice and defer docs drift.

If no safe implementation slice remains, Codex must report that directly and must not convert the absence of a code winner into a docs-selection default.

If a candidate implementation slice depends on overturning a currently preserved contrast, preserved row meaning, or preserved semantic boundary, it is not a lawful automatic next build.

Guarded candidates may be written only as conditional options, not as the default next build, unless current repo truth already exposes them as safe without overturning preserved truth.

If the only remaining candidate is guarded by a possible contradiction with currently preserved truth, Codex must stop and report no lawful implementation build exposed by current repo truth.

3.6 Mandatory First-Read Files for Major Work

Before any architecture work, design work, or code-editing run, Codex must read:

docs/CORE_ARCHITECTURE.md

docs/SELENE_BUILD_EXECUTION_ORDER.md

docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md

If the task concerns a specific build section, Codex must also read the relevant section file before proposing or changing anything.

3.7 Task-State Resolution, Recovery, and Silent Execution Protocol

Before any substantial work, Codex must classify the task into exactly one of:

FRESH_AUTHORING

PARTIAL_AUTHORING_RECOVERY

EDIT_EXISTING_FILE

READ_ONLY_AUDIT

PUBLICATION_RECOVERY

Use FRESH_AUTHORING only when:

the target file does not exist;

the target file is not already untracked;

there is no partial-authoring state for the target.

If the target file exists and is the only relevant modified or untracked file for the task, Codex must switch to PARTIAL_AUTHORING_RECOVERY instead of stopping.

If the target file already exists and the task is verification or approval review, Codex must switch to READ_ONLY_AUDIT instead of using a fresh-authoring flow.

Target existence is a stop condition only for FRESH_AUTHORING.

It is not a stop condition for:

PARTIAL_AUTHORING_RECOVERY

EDIT_EXISTING_FILE

READ_ONLY_AUDIT

PUBLICATION_RECOVERY

3.8 Single-Preflight Rule

Run preflight once only per task.

Do not loop preflight.

Do not repeat target-file existence checks over and over.

Do not repeat remote-reachability checks over and over.

If repo state changes during the task, do one reconciliation pass only.

3.9 Silent Execution Rule

Do not emit repeated progress narration.

Do not emit repeated “I’m about to...” messages.

After preflight, complete the work and return one final completion pack.

If blocked, stop once with one concrete blocker.

Clarification:

This does not remove the Plan State Tracking requirement. Codex must track plan state accurately and report it at resume, blocker, or final handoff without progress spam.

3.10 Repo-Truth Precedence Rule

If task instructions conflict with current repo truth, Codex must follow current repo truth plus these task-state rules and report the mode switch once in the final output.

3.11 Plan State Tracking and Next-Action Continuity

Whenever Codex is executing a build plan, test plan, repair plan, audit plan, staged run, or any ordered task sequence, Codex must maintain an accurate live plan-state report.

For each phase, subtest, command, smoke run, repair, commit, push, interruption, or context switch, Codex must track:

the plan name;

the current run / phase / test identifier;

the exact prompt or command used;

the input modality when relevant, such as controlled smoke voice, JD manual voice, typed diagnostic, native/manual UI, cargo command, native build command, or browser/UI route;

the result: pass, failed, repaired-and-pass, blocked, skipped-with-reason, or not yet run;

the evidence produced;

the commit hash if a repair was committed;

the clean-tree and remote equality posture;

the exact next lawful run / phase / test / action.

Codex must not rely on memory alone for ordered plan continuation.

Before resuming after an interruption, side question, user-requested change, repair, context switch, or another build thread, Codex must reconstruct and state the current plan state from repo/report truth:

last completed test;

last failed test if any;

last repair commit if any;

current clean-tree/remote posture;

exact next action.

Codex must not drift, skip ahead, repeat the wrong test, forget a passed test, forget an unresolved failure, or continue from a stale next action.

If Codex cannot reconstruct the current plan state honestly, Codex must stop and report:

PLAN_STATE_TRACKING_GAP_FOUND

Codex may not continue the plan until the plan-state report is reconciled.

3.12 Single-Review Selection and Implementation Override

For any unchanged same-frontier, same-subsystem, or same-carrier-family next-move question, Codex may perform at most one read-only winner-selection review.

If that first review exposes one lawful executable implementation winner, further review of that same frontier is forbidden until repo truth changes or JD explicitly requests another review.

After that first lawful winner-selection review, Codex must switch to implementation mode instead of continuing review churn.

Implementation-mode consequence:

if JD asked for the next build, Codex must author or execute the implementation run for the selected winner;

if JD asked for review only, Codex must report the single selected winner and stop.

Codex must not convert a completed winner-selection review into another review of the same frontier.

Priority override after the first review:

non-visibility executable winner with a direct live source seam;

non-visibility executable winner with a broader but still lawful live seam;

visibility winner that is required to unlock a blocked executable seam;

additional same-family visibility refinement.

Same-family anti-loop rule:

Codex must not keep slicing one already-live interrupt or visibility family into further read-only micro-surfaces when a lawful non-visibility executable winner is already exposed by current repo truth.

Once bounded dedicated visibility exists for that family, a non-visibility executable winner outranks any further same-family visibility refinement unless JD explicitly requests the visibility refinement.

Codex must treat repeated visibility-only refinement of an already-auditable family as review-loop behavior, not as the default next move.

A second review is lawful only if:

repo truth changed after the first review;

JD explicitly requested another review;

the first review proved no lawful implementation build exposed by current repo truth;

the first review found a real contradiction, approval-boundary conflict, or unresolved tie that prevents safe implementation selection.

Tie handling after one review:

if one target clearly outranks the others, Codex must select it and move to implementation;

if multiple adjacent implementation winners remain lawful, Codex should batch them when safe under existing implementation-first law;

if a genuine tie still remains after applying the priority rules, Codex must produce one short decision memo for JD approval.

Repeated review is never a substitute for implementation once a lawful executable winner has already been identified.

4. Approval, Scope, and Change Gates

4.1 Existing Code Default-Deny Rule

Existing files are read-only by default.

New additive files/modules/tests are allowed by default only if task scope permits.

Editing existing files requires explicit file-scope approval from JD.

If approval is unclear, stop and ask.

4.2 File Scope Gate

Before any edit run, Codex must declare:

exact files to be changed;

why each file must change.

No touching files outside declared scope.

If more files are needed, stop and ask JD first.

For any change run:

git diff --name-only must be a subset of declared scope;

any out-of-scope file means stop and request approval.

4.3 Change Class Gate

Class A:

additive new modules only;

allowed if within task scope.

Class B:

edit non-spine existing files;

requires JD approval.

Class C:

touch spine / contracts / ordering / idempotency / shared behavior;

requires explicit JD approval plus expanded proof.

For Class B or Class C edits, Codex must have explicit JD approval in-thread before editing.

Missing approval token means stop.

4.4 Spine Lock

The following high-risk core areas are immutable without explicit JD approval:

PH1.X

PH1.OS

PH1.L

SimulationExecutor

shared dispatch and mutation layers

packet contracts

reason registry

idempotency registry

state machine

turn ordering

access control

Any touch to spine code requires explicit JD approval.

4.5 Contract Surface Lock

No removals, renames, semantic changes, or ordering changes to shared:

fields

enums

packet meaning

gate order

state order

idempotency behavior

unless JD explicitly approves it.

Backward-compatible additive changes only by default.

4.6 Clean Tree Rule

Start of every run:

git status --short must be empty;

if not empty, stop and report it;

repo-local build artifact trees created by xcodebuild are part of the worktree and must be removed before this cleanliness check;

for Apple app work, relevant cleanup targets include apple/mac_desktop/build and apple/iphone/build when the task touches those targets.

End of every run:

git status --short must be empty again;

repo-local build artifact trees created by xcodebuild must be removed before final cleanliness verification.

No new task may begin on a dirty tree.

No task is complete until the tree is clean again.

4.7 Dirty-Tree Refinement Exception

This is the only narrow exception to the Clean Tree Rule.

A non-empty git status --short must not cause an automatic stop when:

the task has already started;

the only changed or untracked file is the task target file.

Automatic stop is still required when:

unrelated files have changed;

the repo is wrong;

the branch is wrong;

remote reachability is required and unavailable.

The Dirty-Tree Refinement exception applies only after a task has already started.

4.8 Baseline Gate

For change runs:

run baseline tests for the affected domain before edits;

if baseline is red, stop immediately and report;

do not code until JD decides.

4.9 Fresh Remote Truth Rule

Remote branch state, publication state, and HEAD == origin/main may be reported only from a successful fetch performed in the same run.

If that fetch fails:

remote publication state must be reported as UNVERIFIED;

Codex must not claim remote equality from stale local refs.

If the fetch succeeds, Codex may report:

local HEAD;

remote origin/main;

whether they are equal.

5. Existing Capability Reuse, No Duplicates, and Historical Cleanup

5.1 Non-Negotiable Core Protection Rule

Codex must not override, replace, delete, break, or drift previously built working code unless JD explicitly authorizes that change.

This includes:

removing working logic;

replacing old behavior with new behavior;

silently changing contract meaning;

deleting existing runtime paths;

broad rewrites of existing systems;

cleanup that changes behavior;

convenience rewrites that erase prior implementation work.

Default rule:

existing built code is protected;

existing behavior stays unless JD explicitly approves change;

additive work is preferred over destructive replacement;

if a prior implementation conflicts with a new idea, stop and ask JD before changing it.

Codex must not keep rebuilding the same area by deleting prior work and replacing it repeatedly.

If a change would alter previous behavior, Codex must stop and get JD approval first.

5.2 Non-Destructive Implementation Rule

When JD provides architecture, design, or build instructions, Codex must not:

delete previously defined functionality;

replace existing behavior with reduced scope;

silently simplify a design;

reinterpret instructions in a way that removes capabilities.

Codex must:

preserve all existing functionality;

extend functionality by adding new components;

keep previously implemented behavior intact unless JD explicitly authorizes removal.

If a change would remove or replace existing functionality, stop and request explicit JD authorization.

5.3 Conflict / Upgrade Workflow for Prior Code

If existing code must change:

raise change request;

require explicit JD approval before edits;

apply minimal patch;

run expanded proof;

report before/after behavior and residual risk.

5.4 Existing Capability Reuse and No Duplicate Implementation Law

Before Codex creates any new implementation, process, runtime owner, adapter path, Desktop lifecycle path, session/listening loop, state machine, router, provider path, memory path, authority path, tool path, or bridge, Codex must first search current repo truth for an existing or similar capability.

Required discovery before implementation:

search exact and related symbols;

identify current owner files;

identify current runtime path;

identify current tests;

identify current ledger/proof status;

classify whether the existing path is active, partial, dead, wrong-owner, or legacy-compatible;

explain why reuse, extension, or repair is not sufficient before creating anything new.

Default rule:

Reuse or repair the existing owner first.

Creating a new parallel implementation is forbidden unless same-task repo truth proves no existing owner/path can lawfully support the task, and JD explicitly approves the new owner/path.

Hard stop:

If Codex finds an existing similar implementation, owner, loop, process, route, state machine, or provider path, Codex must stop before creating a duplicate and report:

EXISTING_OWNER_REUSE_REQUIRED

Codex must include:

existing owner file/path;

existing symbols;

why it appears related;

whether it should be reused, repaired, or extended;

what approval is needed if a new parallel path is still proposed.

Forbidden without explicit JD approval:

duplicate app process owner;

duplicate managed adapter launcher;

duplicate wake/listening loop;

duplicate session lifecycle owner;

duplicate runtime bridge;

duplicate PH1.X active-context path;

duplicate PH1.M memory/recall path;

duplicate PH1.E/tool-routing path;

duplicate provider path;

duplicate authority/simulation path;

duplicate Desktop semantic workaround;

replacing existing working code with a new path because it is easier;

creating a new fallback path to hide a failure in the original path.

Every implementation final report must include an Existing Capability Reuse Proof:

searches run;

existing owners found;

reused/extended owner;

confirmation no duplicate owner/path/process was created;

if a new path was created, exact JD approval and repo evidence proving no existing owner was sufficient.

5.5 Desktop Boundary and Current-App Provenance Law

Desktop must reuse the existing Desktop lifecycle, capture, playback, transport, render, and runtime bridge owners.

Desktop may fix:

app singleton behavior;

adapter process lifecycle;

microphone capture;

listening/re-arm state;

TTS playback/completion;

transcript transport;

rendering accepted runtime output.

Desktop must not create or duplicate:

semantic intent logic;

slot filling;

memory recall/write/propose;

PH1.X active context decisions;

PH1.E tool routing;

provider calls;

protected execution;

authority decisions.

Before any Desktop app proof, live voice proof, manual smoke, JD live smoke, xcodebuild proof, or native UI test, Codex must prove that the app being opened is the current app built from the current repo HEAD.

No live Desktop/JD smoke proof counts unless Codex first closes stale app instances, proves one Desktop app and one managed adapter/runtime owner, shows the exact launched bundle path, and proves `/healthz` or the repo-equivalent health/provenance endpoint reports the current repo/head provenance.

Codex must not test an old app bundle, stale DerivedData app, old /Applications copy, duplicate Desktop process, or duplicate managed adapter process.

Required before Desktop smoke:

Prove current repo:

pwd

git rev-parse HEAD

git status --short

Prove Desktop build source:

exact xcodebuild command used;

exact app bundle path launched;

whether app came from DerivedData, repo build output, or installed Applications path.

Prove no stale app is running:

list Selene Desktop processes;

close/kill stale duplicate Selene Desktop app instances where safe;

ensure only one current Selene Desktop app instance is active.

Prove no stale managed adapter is running:

check local adapter process/port owner;

do not launch a second adapter if one is already active;

attach to existing correct adapter only if ownership/version is proven;

otherwise stop and report.

Prove runtime/app provenance:

query `/healthz` or the repo-equivalent health/provenance endpoint;

record current repo HEAD from that health/provenance result when exposed;

record adapter process/port owner;

prove exactly one Desktop app instance and exactly one adapter/runtime owner before JD is asked to run live smoke.

After any Desktop code change:

rebuild the Desktop app;

close the old running app;

launch the newly built app;

prove the launched app path matches the fresh build;

restart/reopen before live smoke.

Hard stop conditions:

If Codex cannot prove the app under test is the current build, stop and report: STALE_DESKTOP_APP_INSTANCE_UNPROVEN

If more than one Desktop app instance is active, stop and report: MULTIPLE_DESKTOP_APP_INSTANCES_FOUND

If more than one managed adapter/runtime owner is active, stop and report: MULTIPLE_ADAPTER_RUNTIME_OWNERS_FOUND

If the app path points to an old bundle or unknown bundle, stop and report: WRONG_DESKTOP_APP_BUNDLE_UNDER_TEST

If `/healthz` or the repo-equivalent provenance endpoint cannot prove current repo/head provenance, stop and report: DESKTOP_HEALTHZ_PROVENANCE_UNPROVEN

Every Desktop/live-app proof final report must include:

current HEAD;

xcodebuild result if Desktop changed;

exact app bundle path launched;

process count before and after launch;

adapter process/port owner proof;

health/provenance endpoint proof, including repo/head provenance when exposed;

confirmation stale app instances were closed;

confirmation stale adapter instances were not used;

confirmation live smoke was run against the current build.

Codex must not claim real Desktop proof unless the current app binary and single-instance runtime ownership are proven.

5.6 Clean Replacement / No Dead Legacy Path Law

When Codex fixes, replaces, reroutes, or repairs behavior, Codex must not simply add a new path and leave the old broken, duplicate, disconnected, or obsolete path behind.

Before final testing and before commit, Codex must search for old code, stale helpers, duplicate owners, unused UI paths, unused assets, stale tests, obsolete feature branches, old routes, old status strings, old providers, old bridge paths, and disconnected replacement logic related to the changed function.

Codex must remove obsolete code/assets/tests when they are no longer part of the current product behavior and removal is inside approved scope.

Codex must not leave:

duplicate owners;

unreachable old code;

stale UI paths;

unused assets;

obsolete tests;

dead helper functions;

old routing paths;

obsolete feature flags;

disconnected replacement logic;

shadow implementations;

stale app/runtime/adapter paths.

Exception:

Old code may remain only when it is still required for migration, backward compatibility, active tests, API contract stability, audit/legal proof, rollback safety, documented feature flags, or an actively used runtime path.

If any old path remains, Codex must prove it is still referenced and explain why it remains.

Every implementation build must include a Legacy Cleanup Proof section in the final report:

searches performed for old strings/functions/routes/assets/tests;

obsolete paths removed;

retained old paths justified;

proof no duplicate owner remains;

proof tests still pass;

proof final tree is clean.

This law applies only inside the approved build scope. Codex must not perform broad unrelated cleanup outside the task scope.

5.7 Mandatory Dead Surface Cleanup Gate

Codex must not leave stale, dead, obsolete, or rubbish code surfaces behind when they are directly related to the current repair.

Before commit, Codex must identify obsolete symbols, helpers, fields, constructors, compatibility shims, preview paths, fallback paths, and owner paths created by or made obsolete by the repair.

Codex must run rg -n for each identified symbol/path before deciding its fate.

Each stale-surface candidate must be classified as exactly one of:

STILL_ACTIVE_REQUIRED

DEAD_LOCAL_SURFACE

WRONG_OWNER_SURFACE

LEGACY_COMPATIBILITY_REQUIRED

DEAD_LOCAL_SURFACE code must be removed immediately when removal stays inside the approved file scope.

WRONG_OWNER_SURFACE behavior must not be kept in the wrong layer. Move it to the owning module only if that owner is already inside the approved scope. If the owner is outside scope, stop and report exactly:

STALE_SURFACE_OWNER_SCOPE_REQUIRED

LEGACY_COMPATIBILITY_REQUIRED may remain only when repo truth proves an active dependency still needs it. The final report must explain what depends on it and what future cleanup should remove it.

If directly related stale-surface cleanup requires broader unrelated files or systems, stop and report exactly:

DEDICATED_DEAD_SURFACE_CLEANUP_REQUIRED

Codex must fix warnings introduced by the current repair before commit.

Desktop-specific dead-surface rule:

Desktop must not retain stale decision surfaces for:

final transcript validity;

barge-in acceptance;

echo/user intent;

turn commit/reject authority;

TTS cancel decision;

session/listening policy;

identity;

memory;

provider/protected execution.

Desktop may keep only:

capture;

playback;

transport;

preview mechanics;

runtime bridge;

rendering accepted runtime output;

obeying runtime control.

Before commit, Codex must prove Desktop did not gain decision authority.

Every implementation final report must list:

stale symbols found;

classifications;

removed symbols;

kept symbols with reasons;

rg -n proof for removed symbols;

Desktop authority proof when Desktop is touched;

warning status;

tests still passing.

5.8 Historical Architecture Decay / Dead Code Purge Law

Selene has accumulated historical code from earlier architecture phases. Some old code may no longer match current canonical architecture, especially around PH1.M memory, PH1.X context, Voice ID, Desktop authority, routing, providers, simulations, and protected execution.

Codex must not assume old code is safe merely because it compiles.

When JD requests cleanup, audit, architecture reconciliation, memory rewrite, voice rewrite, routing repair, or owner clarification, Codex must classify historical code surfaces as:

CURRENT_ACTIVE_REQUIRED

Used by the current runtime path.

Matches current architecture.

Covered by current tests/proof.

Keep.

MIGRATE_TO_CANONICAL_OWNER

Contains useful logic but belongs inside a different canonical owner.

Must not remain in the wrong layer.

Move only with explicit JD approval and approved file scope.

DEAD_UNREACHABLE

Not referenced by runtime, tests, registry, routes, feature flags, contracts, or active docs.

Remove when inside approved cleanup scope.

STALE_DANGEROUS

Conflicts with current architecture or creates a possible second brain, duplicate owner, bypass path, stale memory path, stale Voice ID path, stale router, stale provider path, stale Desktop authority path, or stale protected-execution path.

Must be removed, blocked, or escalated before further wiring continues.

LEGACY_COMPATIBILITY_REQUIRED

May remain only when repo truth proves it is needed for active compatibility, migration, rollback, audit/legal proof, feature flag support, or existing external contract stability.

Codex must state exactly what depends on it and what future condition allows removal.

REPO_TRUTH_CONFLICT

Code, docs, ledger, tests, or architecture inventory disagree about whether the surface is active, deprecated, or canonical.

Codex must stop and report exact file/path evidence before editing.

Historical cleanup requirements:

Before building new memory, context, Voice ID, routing, provider, Desktop, or protected-execution behavior, Codex must search for historical surfaces in that same domain.

Codex must produce a Historical Surface Map:

old symbols/routes/files found;

owner layer;

whether compiled;

whether referenced;

whether tested;

whether registered;

whether reachable from runtime;

whether it conflicts with current architecture;

classification from the list above;

keep/migrate/delete recommendation.

Codex must not create a new canonical path while an old conflicting path remains reachable.

Codex must not leave two active owners for the same responsibility.

Canonical owner examples:

PH1.M owns governed human memory and recall.

PH1.X owns live turn/context/routing posture.

Voice ID / identity engines own speaker identity evidence.

PH1.E / provider/search engines own tools and public provider routing.

SimulationExecutor and authority layers own protected execution.

Desktop and iPhone are clients only: capture, playback, transport, render, and obey runtime output.

Desktop, Adapter, PH1.X, PH1.E, or helper files must not become hidden memory brains.

A cleanup run may delete old code only when:

JD explicitly approves cleanup scope;

repo truth proves the path is dead, stale, wrong-owner, or superseded;

tests prove the canonical path still works;

final report proves no duplicate owner remains.

If cleanup requires touching broader files than approved, Codex must stop and report:

HISTORICAL_CLEANUP_SCOPE_APPROVAL_REQUIRED

Every historical cleanup final report must include:

historical surfaces found;

classifications;

deleted paths;

retained paths with dependency proof;

migrated paths with owner proof;

tests passed;

runtime/smoke proof where applicable;

clean tree proof.

5.9 Correct Owner Repair and Regression Preservation Law

Codex must not fix a failure in the nearest visible file, UI layer, adapter layer, bridge layer, helper, or test harness unless that layer is proven to be the canonical owner of the broken behavior.

Before any repair, Codex must identify the responsible owner layer from repo truth.

Codex must produce a Correct Owner Map before editing:

observed failure;

expected behavior;

suspected owner layer;

exact owner file/path;

exact symbols involved;

upstream caller;

downstream callee;

tests covering the owner;

old working paths that may be affected;

why the fix belongs in the selected owner;

why the fix does not belong in Desktop, Adapter, helper code, tests, or another convenience layer.

Codex must not patch around a broken owner by adding logic to:

Desktop;

iPhone;

Adapter;

runtime bridge;

UI rendering layer;

test fixture;

fallback path;

helper shim;

duplicated route;

new parallel owner;

unless repo truth proves that layer is the canonical owner and JD explicitly approves the file scope.

Nearest-layer patching is forbidden.

A repair must be made at the lowest correct canonical owner that owns the behavior.

If the correct owner is outside the approved file scope, Codex must stop and report:

CORRECT_OWNER_SCOPE_APPROVAL_REQUIRED

Before changing a repair area, Codex must identify preserved working paths in the same domain.

Codex must run or name the baseline tests/smoke paths that prove those preserved paths currently work before editing.

After the repair, Codex must prove:

the original failure is fixed;

preserved old working paths still work;

no old working path was replaced or silently bypassed;

no new duplicate owner was created;

no convenience-layer workaround was introduced;

no protected path was weakened;

no public/advisory path was blocked incorrectly;

no Desktop/client semantic authority was added.

If a previously working path fails after the repair, Codex must stop and report:

REGRESSION_AGAINST_PRESERVED_WORKING_PATH

Codex must not commit until the regression is repaired or JD explicitly decides how to proceed.

Every repair final report must include a Correct Owner Repair Proof:

chosen owner;

rejected wrong-owner locations;

files changed;

why each changed file owns the behavior;

preserved paths tested before edit;

preserved paths tested after edit;

regression result;

proof no duplicate owner/path was created.

5.10 Universal Algorithmic Implementation / No Patchwork Law

Codex must always implement real architecture algorithms, real owner wiring, real state machines, real scoring, real packet flow, and real reusable logic.

Codex must never substitute phrase patches, example-specific string checks, toy fixes, shortcut branches, or wrong-owner helper hacks for proper implementation.

This law applies across every engine, runtime surface, client surface, bridge, storage layer, provider layer, and protected-execution layer.

This includes, but is not limited to:

PH1.X

PH1.M

PH1.C

PH1.L

PH1.E

PH1.WRITE

PH1.TTS

PH1.VOICE.ID

PH1.LANG / PH1.SRL / PH1.N

Adapter

Desktop

Storage / Archive / Audit

Search / Provider

Protected execution / SimulationExecutor

JD examples are evidence of a broken pattern.

JD examples are not the production algorithm.

Forbidden production work includes:

exact phrase branching;

prompt-specific if/else chains;

string contains fixes for one example;

one-city / one-person / one-topic hardcoding;

one-off customer/company/person/product logic;

hidden adapter shortcut meaning;

Desktop semantic fallback;

helper-shim semantic workarounds;

duplicate owner paths;

narrow route patches that only make the tested prompt pass;

fixture-driven production behavior;

fake tests that assert the patch instead of the product behavior;

storing semantic meaning in the wrong owner;

leaving old phrase patches active after a canonical owner exists.

Forbidden production examples include but are not limited to:

contains("which city")

contains("which areas")

contains("the time")

contains("Sydney")

contains("Melbourne")

contains("Japan")

contains("make it shorter")

contains("make it warmer")

contains("prepare payroll")

contains("organize payroll")

exact real person/company/customer/product names

any exact phrase JD used in live testing as the behavior trigger

Allowed uses of exact phrases:

tests;

fixtures;

proof reports;

comments explaining a scenario;

user-facing copy;

canonical enum names;

reason codes;

governed domain vocabulary tables owned by the correct engine.

A governed domain vocabulary is allowed only when it is:

data-driven;

reusable;

owned by the correct engine;

tested with unseen alternatives;

not hidden in scattered string checks;

not used as a substitute for the required algorithm.

Correct production implementation must use generalized algorithms appropriate to the owner, such as:

semantic role classification;

intent/posture classification;

active context frames;

topic stacks;

reference resolution;

slot/entity frames;

candidate generation;

candidate scoring;

hard disqualifiers;

ambiguity scoring;

confidence thresholds;

correction targeting;

clarification target tracking;

writing artifact state;

tool continuity state;

memory evidence packets;

speaker/identity evidence packets;

privacy/scope gates;

protected-risk classification;

source/evidence verification;

state machines;

owner-specific routing packets;

deterministic protected-execution gates;

reusable decision functions.

Every build touching language understanding, current user turns, context, memory, recall, search, providers, writing, speech, Voice ID, protected classification, tools, routing, Desktop behavior, Adapter behavior, or storage evidence must include an Algorithmic Generality Proof.

The Algorithmic Generality Proof must state:

1. The canonical owner of the behavior.

2. The old patch/shortcut paths found.

3. Whether each old path was removed, migrated, retained as compatibility, or deferred.

4. The generalized algorithm or state machine used.

5. The input features used.

6. The output packet/directive produced.

7. Why the solution works beyond the exact JD example.

8. Positive tests using the original failing examples.

9. Positive tests using unseen paraphrases or substituted entities.

10. Negative tests proving unrelated inputs are not hijacked.

11. Proof Desktop did not gain semantic authority.

12. Proof Adapter did not become the context/memory/tool brain.

13. Proof no duplicate owner/path was created.

14. Proof protected fail-closed behavior was preserved.

For every relevant build, Codex must run a phrase-patch scan before commit.

Minimum scan:

git diff --unified=0 | rg -n "contains\(|starts_with\(|ends_with\(|== \".*\"|which city|which areas|the time|make it shorter|make it warmer|Japan|Sydney|Melbourne|Brisbane|payroll|Tim|Mark|locked factory|Niseko|Hakuba|Nozawa|Sapporo"

Every hit must be classified as exactly one of:

TEST_FIXTURE_OK

REPORT_OK

COMMENT_ONLY_OK

USER_FACING_COPY_OK

CANONICAL_REASON_CODE_OK

DOMAIN_VOCABULARY_OK

EXISTING_COMPATIBILITY_OK

RETAINED_COMPATIBILITY_PATH

DEAD_LOCAL_SURFACE

WRONG_OWNER_SURFACE

PRODUCTION_PHRASE_PATCH_NOT_ALLOWED

If any hit is classified as PRODUCTION_PHRASE_PATCH_NOT_ALLOWED, Codex must remove the patch and implement the correct owner algorithm before commit.

Codex must also search the affected owner files for old shortcut paths:

rg -n "contains\(|starts_with\(|ends_with\(|== \".*\"|shortcut|fallback|deterministic_active_context|deterministic_weather_context|weather context|time context|H380|H411|H412" <affected files>

Old paths must be classified as:

CURRENT_ACTIVE_REQUIRED

MIGRATE_TO_CANONICAL_OWNER

DEAD_UNREACHABLE

STALE_DANGEROUS

LEGACY_COMPATIBILITY_REQUIRED

REPO_TRUTH_CONFLICT

Codex must remove old patchwork when all of these are true:

it is inside approved scope;

repo truth proves it is dead, stale, wrong-owner, or superseded;

canonical owner replacement is wired;

tests prove preserved behavior still works;

final report proves no duplicate owner remains.

Codex must not delete old code blindly.

If old patchwork cannot be safely removed inside the current scope, Codex must mark it as retained compatibility and report the future removal condition.

If cleanup requires broader scope, Codex must stop and report:

ALGORITHMIC_CLEANUP_SCOPE_APPROVAL_REQUIRED

Hard stop conditions:

If Codex fixes a behavior by hardcoding the exact tested phrase, stop.

If Codex cannot explain the generalized algorithm, stop.

If Codex puts semantic meaning into Desktop, stop.

If Codex makes Adapter the context, memory, tool, provider, Voice ID, or protected-execution brain, stop.

If Codex creates a second route instead of repairing the canonical owner, stop.

If unseen paraphrases fail, the build is not complete.

If old phrase patches remain reachable without compatibility justification, the build is not complete.

Final rule:

Codex must build real architecture, real code, real owner wiring, and real algorithms.

Codex must not build toys with phrases.

5.11 Real Proof and Code Hygiene Law

Cargo tests, cargo check, unit tests, mocked tests, and fixture tests are mandatory safety gates, but they are not product acceptance proof.

Cargo proves only that code compiles and that selected test cases passed.

Cargo does not prove:

the real Desktop app works;

the real runtime path used the code;

the real adapter carrier preserved state;

the correct engine owned the behavior;

the visible user behavior worked;

a stale or duplicate path did not win;

previously working behavior still works.

Core rule:

Cargo is a gate.

Live/runtime proof is acceptance.

Codex must never claim user-visible behavior, engine routing, Desktop behavior, voice behavior, memory behavior, writing behavior, provider behavior, protected/public split, or active-context behavior is working only because cargo passed.

For any user-visible or engine-routing change, Codex must provide real-path proof showing:

1. user input entered the real runtime path;

2. correct owner engine handled it;

3. expected carrier/input field existed;

4. correct output was produced;

5. output reached the real app/user-visible surface;

6. backend evidence agrees with visible behavior;

7. no stale, duplicate, fallback, mock, or test-only path won.

If live/user-visible behavior fails, cargo passing does not matter. The build failed.

Codex must not commit or call the build complete when cargo passes but real behavior fails.

This section reinforces Sections 7.15 and 7.16. It does not weaken their JD live testing, backend evidence, or current-app provenance requirements.

Failed Patch Cleanup / No Layered Rubbish

When a code change, repair, or build attempt fails targeted proof, live proof, JD live acceptance, or backend evidence verification, all code introduced by that failed attempt is untrusted by default.

Codex must not keep layering new fixes on top of failed work in progress.

Before attempting another fix, Codex must inspect and classify the current diff.

Required commands:

git status --short

git diff --stat

git diff

Every changed hunk must be classified as:

KEEP:

proven necessary by backend evidence;

directly tied to the correct owner repair;

not duplicate logic;

not fallback workaround;

not Desktop/Adapter semantic drift;

not phrase-patch behavior;

not dead or temporary diagnostic logic.

REMOVE:

attempted fix that did not change the failed behavior;

code added only because a previous test failed;

workaround outside the correct owner;

duplicate route, bridge, carrier, classifier, formatter, or fallback;

temporary debug logic not required for final behavior;

phrase-specific or example-specific branch;

stale path made obsolete by the next repair.

UNKNOWN:

not proven necessary;

not clearly tied to the accepted owner path;

kept only because Codex is unsure.

UNKNOWN must be treated as REMOVE unless JD explicitly approves keeping it.

A failed patch must be removed, reverted, or surgically deleted before the next repair attempt unless Codex proves, with backend evidence, that a specific hunk is still required.

Codex must not commit failed work in progress.

Codex must not hide failed work in progress behind passing unit tests if live/user-visible behavior still fails.

Codex must not build a second fix on top of a first failed fix unless the first fix has been proven necessary and documented.

If the repo has accumulated several failed attempts and Codex cannot clearly prove which hunks are valid, Codex must stop and recommend returning to the last known-good commit/baseline.

Core rule:

Failed code does not get to stay just because it was written.

If it did not help pass the real proof, remove it before building again.

Engine Capability Reality / No Dead Functionality

Codex must not treat existing code as valid merely because it exists.

Before building, extending, or repairing any engine, Codex must classify relevant engine code paths by real runtime status:

LIVE_ACTIVE:

executed by the real runtime path;

proven by backend evidence and live/user-visible proof.

LIVE_PROVEN_KEEP:

already proven working on the real runtime/live path;

must not be deleted, bypassed, weakened, or casually rewritten.

LIVE_PROVEN_EXTEND:

already proven working, but current task requires narrow extension;

Codex may add only through the smallest owner-correct seam.

LIVE_PROVEN_REPLACE:

proven working but explicitly approved for replacement;

replacement must preserve all old accepted behavior before old path is removed.

WIRED_UNPROVEN:

reachable in code but not live-proven.

DORMANT_APPROVED:

intentionally present but disabled behind an explicit documented gate or future-build flag.

TEST_ONLY:

used only in tests/fixtures and impossible to enter from production/runtime paths.

DEAD_REMOVE:

unreachable, unused, obsolete, or not connected to the canonical runtime path.

DUPLICATE_REMOVE:

overlaps with another canonical owner/path.

STALE_REMOVE:

replaced by newer logic or no longer aligned with current architecture.

FAILED_WIP_REMOVE:

introduced during failed repair attempts and not proven necessary.

UNKNOWN_REMOVE:

Codex cannot prove why it exists.

Rules:

1. Code is not real capability unless it is wired, reachable, owner-correct, tested, and proven on the real path.

2. Unused code must not remain in production merely because it compiles.

3. Failed, stale, duplicate, or unreachable functionality must be removed before adding more functionality.

4. Codex must not patch dead, duplicate, stale, or test-only code as if it were canonical owner code.

5. If Codex cannot prove a function is used by the live runtime path, treat it as untrusted.

6. Mock/unit tests alone do not prove capability is real.

7. Desktop-visible behavior must agree with backend evidence before behavior can be called working.

8. If an engine contains unused or partially wired capability, Codex must wire and prove it, remove it, or mark it DORMANT_APPROVED with JD approval.

Core rule:

A feature that exists only in code but is not wired, proven, and used by the real runtime is not a feature. It is repo noise.

Proven Working Code Preservation / Safe Extension

Codex must not delete, rewrite, bypass, or weaken code that is already proven working unless the task explicitly requires replacing it and the replacement is proven to preserve old behavior.

When Codex needs to add behavior to a path that already works, Codex must follow this sequence:

1. Baseline proof first.

Reproduce the existing working behavior before edits, capture backend evidence, capture visible/live result, and record the exact owner path.

2. Minimal seam extension.

Add new behavior only at the correct owner seam. Do not create a parallel route, bypass the proven path, move ownership to Desktop or Adapter, add phrase patches, or change unrelated working behavior.

3. Preserve old behavior.

Rerun the old accepted live scenario. It must still pass. If old behavior breaks, the patch failed.

4. Prove new behavior.

Run the new target scenario. Backend evidence and visible behavior must agree.

5. Failed patch cleanup.

If new behavior fails, remove the new failed patch, return to the previously proven baseline, and do not layer a second fix on top of the failed first fix.

6. Final report must state:

what was already proven before the edit;

what was changed;

why the changed owner was correct;

what old behavior was re-proven;

what new behavior was proven;

what failed/unused code was deleted;

what code was protected from deletion.

Core rule:

Unproven code should be deleted.

Proven working code should be protected.

Codex must not confuse these two.

6. Search, Provider, Evidence, and Presentation Law

6.1 No Real Search-Name Hardcoding

Codex must never fix search, spelling, entity understanding, source ranking, or direct-answer failures by hardcoding a real searched name into code.

Real searched names include customer names, company names, person names, product names, supplier names, competitor names, and one-off query examples.

Real searched names are forbidden in all code files, including production code, tests, fixtures, mocks, corpora, sample data, and proof hooks.

Tests must use synthetic fake entities.

If a real query fails, Codex must repair the generic capability:

spelling;

phonetic matching;

entity disambiguation;

query planning;

source ranking;

page reading;

claim verification;

answer formatting.

Any real searched-name hardcoding in code is a build blocker.

Docs and ledger may mention historical real examples only to describe what happened, not to drive runtime behavior.

6.2 Search Provider Cost, Kill-Switch, and Billing-Grade Usage Control

No search provider call may happen unless the global provider kill switch allows it.

No paid provider call may happen unless paid providers are explicitly enabled.

No provider call may happen without a ProviderCallBudgetPacket or approved equivalent.

Every provider call must increment a pre-network call counter before network execution.

Disabled providers must result in zero provider call attempts and zero provider network dispatches.

Startup and health checks must not call external providers.

Normal tests must not call paid providers.

Live provider tests must be ignored by default and require explicit env opt-in.

Provider-off must return a safe disabled result, not call a fallback provider.

Public websearch remains read-only public.

Protected execution remains simulation/authority gated.

Usage/cost events must identify account layer, tenant/company/private-user ownership, actor user, service/module, provider, route, operation type, cost owner, billing scope, and billable class where runtime context is available.

Blocked provider usage must be non-billable / zero provider cost.

Fake provider test usage must be non-billable and must not be marked as live provider cost.

Stage 2 captures usage/cost facts for future billing but must not implement final pricing, invoicing, subscription plans, tax/GST, payment collection, or commercial contract enforcement.

6.3 Safe Page Fetch and Evidence Extraction

Page fetch/read is a provider/network operation and must be gated by the global provider/url-fetch policy.

URL fetch must default OFF unless explicitly enabled by safe config or test harness.

URL fetch must block private, internal, local, metadata-service, credential-bearing, and unsafe-scheme URLs before network dispatch.

If DNS/private-address validation cannot be safely implemented or proven for live external fetches, live external URL fetch must remain disabled.

URL fetch must enforce timeout, redirect, byte, content-type, and extraction limits.

Evidence excerpts are capped at 500 characters unless a repo constant deliberately lowers the limit.

Evidence chunks are capped at 5 per source unless a repo constant deliberately lowers the limit.

Trace preview is capped at 2,000 characters unless a repo constant deliberately lowers the limit.

Raw full pages and full extracted text must not enter response_text, TTS, normal UI, or public trace.

URL fetch must not:

execute JavaScript;

use browser automation;

send secrets/cookies/internal auth headers;

bypass paywalls/access controls;

bypass Stage 2 budget/counter/usage metadata.

Blocked URL fetch must be non-billable / zero provider cost.

Fake/local fixture fetch must use in-memory fixture transport or repo fixture injection, not localhost HTTP as a fake external page.

Fake/local fixture usage must be test-only and non-billable.

Extracted evidence may support answers, but extracted evidence does not automatically become accepted proof. Stage 1 accepted/rejected source rules still apply.

No real searched names may be used in page-read tests.

OpenAI Realtime STT and OpenAI TTS must not be blocked by URL-fetch/search-provider gating.

6.4 Claim-to-Source Verification

Selene must not present factual websearch claims unless they are supported by accepted evidence.

Source mentions entity does not prove the requested claim.

Entity-only evidence is insufficient for role, numeric, date, current-status, ownership, legal, pricing, comparison, list, or ranking claims.

Claim verification may use accepted sources and accepted evidence chunks only.

Rejected, wrong-entity, weak, entity-only, mention-only, and trace-only evidence must not support final answer claims.

Contradictory evidence must be detected and resolved by source hierarchy/freshness or surfaced as uncertainty.

Unsupported claims must be removed or safe-degraded before response_text or TTS output.

Confidence must be evidence-derived, not guessed, provider-rank-derived, or inflated because multiple weak sources repeat the same unsupported claim.

PH1.WRITE/current formatter must not invent facts, sources, citations, or confidence.

Claim verification must not call external LLMs, web providers, search providers, page fetchers, verifier APIs, retries, fallbacks, or probes.

Claim verification itself must not be marked as live provider usage or billable provider cost.

Fake fixture verification must be non-billable and test-only.

Provider-off and URL-fetch-off counters must remain zero for provider/fetch attempts and network dispatches during claim verification.

Tests must use synthetic fake entities only and must prove real answer-gate behavior, not only helper functions, enum constants, or unused packets.

6.5 Websearch Presentation and Source Chips

Websearch final presentation must use PH1.WRITE or the approved canonical presentation boundary where available.

Websearch answers must be short, precise, and clean by default.

Accepted sources must be displayed as small clickable source chips or approved equivalent metadata.

response_text must not contain raw source dumps, provider JSON, debug trace, rejected source details, or long raw URLs.

tts_text must be the clean spoken answer only.

Rejected sources must remain in trace/debug only.

Source chips must link only to safe approved source-page URLs.

PH1.WRITE/current formatter must not invent facts, invent sources, invent citations, hide uncertainty, upgrade weak evidence, or override claim verification.

PH1.WRITE/current formatter must preserve claim verification classes from Stage 4.

Image display is not Stage 5 and must not be faked.

Tests must use synthetic fake entities only.

Public websearch remains read-only public answer work and does not require simulation authority.

Protected execution remains simulation and authority gated.

6.6 Safe Image-Backed Search Presentation

Image-backed search answers may display images only from approved image metadata paths.

Images must be relevant to the requested entity/query.

Image display must pass safety, URL, source-page, display-policy, and relevance checks.

Image cards must link to safe source pages, not raw image URLs.

Raw image URLs must not be used as normal click targets.

Images must not be fabricated or presented as real source images unless they come from approved source metadata.

Image metadata must not be treated as claim verification unless supported by accepted evidence.

If image display is not approved, Selene must degrade to text plus source chips.

Tests must use synthetic fake entities and fixture/local images only.

No live paid provider calls are allowed in normal tests.

No remote image loading is allowed when image fetch is disabled.

Public search remains read-only.

Protected execution remains simulation and authority gated.

6.7 Visual Search Presentation and Live Voice Proof

Entity, company, person, product, winery, brand, organization, and public-entity search results must display relevant approved photos or images when available.

Image cards must come from approved SearchImagePacket data and must not be fabricated.

Placeholder-only image cards must not be presented as real photos.

Small source pills must be compact, clickable, rounded, gray/shadow style, and visually close to JD’s uploaded reference.

Source pills must use accepted sources only and must not expose raw URLs, unix_ms, provider JSON, rejected sources, or trace/debug.

Desktop must render approved packets only and must not call Brave, choose images, hold provider secrets, rank sources, or transform provider metadata into facts.

TTS must speak clean answer text only, not source or image metadata.

Real Desktop voice proof must record the exact captured transcript and cannot be replaced by typed proof.

Protected execution must remain simulation and authority gated.

6.8 Controlled Brave Re-Enable and Low-Volume Billing Proof

Brave must remain OFF by default and may only be enabled by explicit controlled configuration.

Paid providers must remain OFF unless explicitly enabled.

No startup provider probes or background provider calls are allowed.

No live Brave call may happen without the canonical provider budget/counter gate.

Every live Brave attempt must increment a pre-network call counter before dispatch.

Every live Brave network dispatch must increment a network dispatch counter.

Disabled providers must produce zero provider call attempts and zero network dispatches.

DeepResearch, News, URL fetch, image provider calls, fallback providers, and provider fanout must remain OFF during basic Brave proof unless separately approved and capped.

Normal tests must not call Brave.

Live Brave proof tests must be skipped unless explicitly opted in.

Brave usage must be intentionally small, capped, and auditable.

Public websearch remains read-only public answer work and does not require simulation approval.

Protected execution remains simulation and authority gated.

6.9 Provider Lane Routing and Cheap-First Search

Search providers must be selected through the provider router, not scattered uncontrolled direct calls.

No-search remains default for prompts that do not need current, external, or source-backed information.

Cache must be checked before provider calls where policy allows, and valid cache hits must produce zero provider attempts and zero provider network dispatches.

Cheap/default providers must be preferred for normal public search when available and allowed.

News/current-event providers must be preferred for news/current-event queries when available and allowed.

Premium providers, including Brave, must be fallback-only unless explicitly selected by policy and budget.

Provider fanout must be OFF by default.

DeepResearch must remain approval/cap gated.

Every provider call must pass the global kill switch, provider-specific enable flag, budget gate, call counter, retry cap, and route cap.

Missing provider secrets must safe-degrade and must not break startup or normal tests.

Normal tests must use fake providers.

Live provider tests must be ignored by default and require explicit env opt-in.

Provider routing must never bypass Stage 1-6 source verification, presentation, image safety, TTS separation, or protected fail-closed law.

Stage 8 must extend the existing approved Selene search/provider/control surfaces.

Stage 8 must not create a parallel provider system, parallel search planner, or parallel websearch stack unless repo truth proves no existing approved surface can safely support Stage 8.

Prefer wiring through existing PH1.N, PH1.SEARCH, PH1.E, PH1.X, app_ingress, web_search_plan, and provider-control surfaces.

Desktop must never become provider-authoritative and must never hold or use provider secrets.

Public websearch remains read-only and must not require simulation authority.

Protected execution remains simulation/authority gated and fail-closed.

6.10 Search Evaluation, Corroboration, and Voice Certification

Selene search quality must be evaluated with synthetic fixtures and approved live smoke only.

Normal tests must not call live providers.

Real public live prompts may appear only in proof reports, never in code, tests, fixtures, or proof hooks.

Search quality must be scored on:

accuracy;

source support;

wrong-source rejection;

freshness;

directness;

source display;

cost;

latency;

voice behavior.

Multi-provider corroboration must remain gated, capped, and OFF by default unless explicitly enabled.

Deep Research must require explicit user intent or approved escalation and cost cap.

Source agreement scoring must support claim verification and must not override it.

PH1.WRITE must not make unsupported claims sound certain.

Real Desktop voice testing must record:

captured transcript;

normalized intent;

route;

query plan;

sources;

answer;

TTS text;

provider counts.

Public search is read-only and does not require simulation authority.

Protected execution remains simulation and authority gated.

6.11 End-to-End Search Certification and Root-Cause Repair

Search failures must be fixed at the responsible engine or runtime layer.

No company-specific or real searched-name fixes are allowed.

Real public names may appear only in manual proof reports, never in code, tests, fixtures, or proof hooks.

Voice proof must show the exact captured transcript and may not be replaced by typed proof.

English and Chinese voice smoke are required where technically possible.

Brave live proof may run only in controlled capped mode with JD-approved opt-in, a fresh configured key, and explicit caps.

If the configured Brave key is missing or invalid, stop and ask JD for a new key. Never create, print, or infer a key.

The total live Brave budget applies across first proof, live series, English voice, Chinese voice, multilingual reset, and Deep Research control combined.

Normal tests must not call live providers.

If a live test fails, Codex must report and fix the responsible code path, not suppress the test.

Protected execution remains simulation and authority gated.

No surface patching, pretend fixing, or docs-only masking of broken runtime code is allowed.

6.12 Best Available Public Search Answer

Public websearch must not stop at a generic conflicting-evidence response as the normal final answer.

Public websearch must return the best available source-backed answer whenever usable accepted evidence exists.

Internal source scoring, agreement scoring, freshness scoring, confidence scoring, contradiction status, and answer classes may be used for ranking and trace.

User-facing response_text and tts_text must not show confidence labels by default, raw score values, internal answer class names, rejected source details, raw provider payloads, or debug trace.

Selene must answer directly first, then expose small accepted source chips through the approved presentation metadata.

When evidence is mixed, Selene must phrase the answer naturally using the clearest available source-backed result and mention disagreement only when it helps avoid overclaiming.

When exact requested information is absent, Selene should say that plainly and return the closest useful source-backed result when one exists.

Selene must not fabricate facts, sources, citations, roles, dates, prices, images, people, or confidence.

Protected and business execution must never use best-available guessing.

7. Testing, Proof, and Reporting

7.1 Mandatory Voice-First Smoke Test Law

Every implementation build, behavior-changing edit, UI/app change, provider/runtime change, adapter change, contract change, or user-visible feature change must include an end-to-end smoke test before final handoff.

Voice smoke is preferred in all cases.

Codex must attempt a real app voice/microphone smoke whenever the current environment, permissions, device routing, and task scope make it practical.

If voice smoke cannot be run, Codex must run a typed app smoke test through the real app UI when practical and must report the exact voice-smoke blocker.

If neither voice nor typed app smoke is practical, Codex must run the closest authoritative text/runtime smoke against the real runtime, adapter, CLI, or endpoint that exercises the changed behavior and must report why app-level smoke was unavailable.

The smoke test must exercise the intended behavior of the build, not only app launch or /healthz.

When the build touches protected, business, access, safety, provider, image, citation, voice, TTS, search, or execution behavior, the smoke pack must also prove the relevant preserved fail-closed or no-overclaim behavior.

A smoke test failure that reflects product behavior, regression, route failure, UI failure, provider wiring failure, or broken end-to-end execution is not acceptable final state.

Codex must fix the failure, rebuild if needed, and rerun the smoke until it passes.

Codex may hand off with a smoke blocker only when the blocker is environmental rather than product behavior, such as missing microphone permission, unavailable audio device, unavailable app automation, network outage, or an external provider outage.

In that case, Codex must run the strongest available fallback smoke and state the blocker plainly.

Unit tests, compile checks, and xcodebuild do not replace smoke testing. They are required proof where applicable, but final completion still requires voice-first smoke or the documented fallback path above.

Final reports for build runs must include:

smoke path used;

exact prompt or command class;

observed result;

whether voice was used;

fallback reason if voice was not used.

7.2 Mandatory Proof Before Commit

For any change run, Codex must provide:

baseline commands passed;

targeted tests passed;

required contract / reason / idempotency / state-machine checks passed where applicable;

determinism checks passed where applicable;

git diff --stat;

no unrelated core files modified.

7.3 Exact Test Execution Verification Rule

If Codex cites an exact test command as proof in a self-authored build instruction, audit, corrective follow-up, or final completion report, Codex must verify both the exact runnable test name and the fact that a nonzero test count actually executed.

Required verification:

verify the exact runnable test name from current repo truth in the same task with a runner-native listing command such as cargo test ... -- --list when the exact test path could be ambiguous;

after execution, verify that the test run executed a nonzero test count, such as running 1 test;

a result with 0 passed; 0 failed or N filtered out does not count as successful exact-test verification;

if the exact test name in task instructions conflicts with current repo truth, Codex must follow current repo truth, report the mismatch once, and update the self-authored verification command to the exact runnable name;

if an exact test fails, Codex must distinguish between target-slice regressions and inherited prerequisite or contract mismatches before concluding that the task itself failed.

Forbidden:

treating an exact test command that ran zero tests as a successful verification;

reusing remembered exact test names without same-task confirmation when the repo can list them directly;

blaming the current target slice for an exact-test failure before checking whether the failure is caused by an inherited prerequisite, stale verification clause, or conflicting upstream repo truth.

7.4 No Vacuous Test Pass Rule

A test, lint, or check only counts as proof if both conditions are true:

the named target is explicitly proved to exist in current repo truth;

the command output proves that at least one matching test or check actually executed.

Passing with zero matched tests or zero executed checks does not count as proof.

When the tool supports exact targeting, Codex should use exact targeting.

If execution count cannot be proved from output, Codex must report the result as unproven.

7.5 Real-Path Proof Rule

For any change run, Codex must state in the final proof pack:

the exact command run;

the exact module, file, function, or runtime path exercised;

what was real;

what was mocked, stubbed, simulated, in-memory, or fixture-backed;

what remains unproven.

Codex must not present narrow harness proof as broader production proof.

7.6 Harness Disclosure Rule

If proof depends on a harness, fixture, fake clock, in-memory store, mock, stub, replay harness, or simulation layer, Codex must name that dependency explicitly in the final report.

Codex must also state the proof scope honestly:

what the harness proves;

what it does not prove;

whether the exercised path is real wiring, partial wiring, or harness-only.

7.7 Build-Report Hash and Ownership Verification Rule

Before any final completion report, self-authored build instruction, audit conclusion, or corrective follow-up that names:

a landed build hash;

a pinned HEAD == ... value;

a prior or prerequisite build hash;

an exact symbol carrier;

an exact file or layer ownership claim;

Codex must verify that claim from current repo truth in the same task.

Required verification:

current landed commit hash: git rev-parse HEAD;

prior or prerequisite build hash: direct git rev-parse, git log, or exact ancestor query from the current repo in the same task;

every cited prior landed build hash in a self-authored build instruction: exact same-task git query that maps the cited build identifier to the current repo hash, such as exact git log --format=%H --grep='^H253:' -n 1 or an exact ancestor query;

exact symbol or state ownership: direct rg or equivalent against the exact named files in the same task;

if a symbol exists in both bridge and shell or across multiple layers, Codex must report the split ownership exactly rather than collapsing it into one carrier.

Forbidden:

reusing remembered full hashes from earlier assistant messages;

inferring exact ownership from summary memory when the repo can be queried directly;

collapsing bridge-owned canonical fields and shell-owned derived state into one carrier when repo truth shows a split;

relying on current HEAD == ... alone while citing preserved prior landed builds without verifying their exact landed hashes in the same task;

stating an exact hash or ownership claim as definitive without same-task verification.

If verification cannot be completed:

stop and report the claim as unverified;

do not pin a self-authored build instruction or final report to that claim.

7.8 Verification Clause Specificity Rule

If a self-authored verification clause is intended to prove that the current task did not add a forbidden symbol, behavior, or route, Codex must scope that clause to the current task slice rather than blindly scanning preserved whole-file history when same-task repo truth shows that one or more forbidden symbols already exist outside the current edit.

Required verification discipline:

if a forbidden symbol check targets edited files that already contain preserved historical symbols, Codex must use a diff-scoped check such as exact git diff --unified=0 -- <task files> | rg ... or an equally narrow task-slice verification;

if the task adds only bounded slices inside larger files, Codex must pair positive slice-presence checks with narrow forbidden diff checks rather than whole-file negative greps;

if preserved symbols already exist in the target file, Codex must say so explicitly and must not present a broad whole-file grep failure as if it were caused by the current task.

Forbidden:

using a whole-file negative grep as the decisive verification clause when same-task repo truth already shows preserved conflicting symbols in that file outside the task slice;

treating inherited symbols in unchanged lines as evidence that the current task violated scope;

writing a self-authored verification clause that cannot distinguish pre-existing repo truth from the current diff when the distinction is discoverable.

7.9 Subjective-Gate Ban

Task gates, stop conditions, and completion claims must not rely on subjective phrases such as:

strongly implies

appears

seems

as if

looks like

Codex must use:

exact searchable text;

exact command output;

exact file-and-line evidence;

exact counted hits;

exact named tests/checks.

If a gate cannot be expressed objectively, Codex must stop and report the ambiguity instead of guessing.

7.10 Authority Conflict Stop Rule

If current repo artifacts that should describe the same truth conflict with each other, Codex must stop and report the exact conflict with file-and-line evidence.

Examples:

code vs plan doc;

master plan vs phase plan;

ledger vs completion claim;

test name in prompt vs test name in repo.

Codex must not silently synthesize a new truth when repo authorities disagree.

Codex must report:

the conflicting files;

the exact conflicting lines;

which authority order applies if it is already resolved by existing law;

whether work is blocked or can continue under existing authority order.

7.11 Completion Discipline

At the end of each task, Codex must explicitly state whether the target was:

authored;

recovered;

audited;

committed/pushed;

freeze-ready or not.

7.12 Strict No-Op Rule

NO_OP is allowed only when the existing artifact already matches the full intended truth or behavior for the task.

NO_OP is not allowed merely because:

a file already exists;

a heading already exists;

a partial implementation already exists;

a similarly named test already exists;

some of the requested truth is already present.

If full no-op equivalence is not proved from current repo truth, Codex must stop and report exact repo state instead of claiming NO_OP.

7.13 No Hidden Mode Switching

If a task begins as authoring but repo truth shows the target already exists, Codex must explicitly switch to recovery or audit mode instead of silently half-following the original authoring prompt.

7.14 No Progress-Spam in Compacted Sessions

If context compaction happens, Codex must resume from current repo truth, not from the original fresh-authoring assumption.

7.15 JD Live Acceptance, Evidence Capture, and Repair Loop Law

Codex’s own tests are necessary but not sufficient when the changed behavior is user-visible, voice-driven, Desktop-visible, TTS-visible, provider-backed, memory/context-related, routing-related, or protected-execution-related.

Codex must not claim final completion merely because unit tests, cargo tests, xcodebuild, mocked tests, fixture tests, typed endpoint tests, or limited smoke tests passed.

Required proof order for user-visible behavior:

Static/build proof.

Targeted automated tests.

Codex-run real-path smoke test where practical.

Codex-run real app voice smoke where practical.

JD live acceptance test when JD is available or when the build is intended for JD voice/UI validation.

Codex must run its own real app/real voice test before handing the build to JD whenever the environment makes that practical.

JD live acceptance is the final product-facing validation for voice/UI behavior.

If JD’s live test fails, Codex must treat the build as not complete, even if all previous tests passed.

On JD live failure, Codex must stop normal completion reporting and report exactly:

JD_LIVE_ACCEPTANCE_FAILED

Codex must then collect or ask for the minimum evidence needed to repair the failure:

exact JD spoken prompt or typed prompt;

exact captured transcript;

expected behavior;

actual behavior;

visible UI state;

response_text;

tts_text;

TTS playback state;

route selected;

runtime state;

adapter request/response where available;

provider calls and counters where relevant;

memory/context packets where relevant;

session/listening state where relevant;

error logs;

timestamp/window of failure;

whether the failure is reproducible.

Codex must classify the failure owner before editing:

Desktop lifecycle/capture/playback/transport/render;

adapter/runtime bridge;

PH1.X active context / turn control;

PH1.M memory / recall / continuation;

PH1.E tool or provider route;

PH1.WRITE presentation;

PH1.L session boundary;

Voice ID / identity evidence;

STT capture/transcription;

TTS generation/playback;

provider/network;

protected execution / simulation / authority;

unknown/unproven.

Codex must not patch the nearest visible symptom.

Codex must repair the canonical owner proven by the evidence, following Section 5.9 Correct Owner Repair and Regression Preservation Law.

Codex must not hide a JD live failure behind:

passing unit tests;

passing mocked tests;

passing typed endpoint tests;

app launch success;

/healthz success;

docs update;

“works in harness”;

“could not reproduce” without evidence;

disabling the failing path;

adding a fallback that bypasses the real owner.

After repairing a JD live failure, Codex must rerun:

the failed JD scenario or closest reproducible real-path equivalent;

Codex real app/voice smoke where practical;

preserved working paths in the same domain;

targeted regression tests;

required build/check commands.

Codex must prove:

the JD-observed failure is fixed;

the original intended build behavior still works;

previously working behavior was not broken;

no new duplicate owner/path was created;

no Desktop/client semantic authority was added;

no provider/protected execution gates were weakened;

final tree is clean.

If JD is not available for live acceptance, Codex may report:

PENDING_JD_LIVE_ACCEPTANCE

In that case, Codex may say the build is handoff-ready, but must not call the user-facing behavior fully accepted by JD.

Final report must clearly distinguish:

CODEX_TESTED

REAL_APP_SMOKE_PASSED

JD_LIVE_ACCEPTANCE_PASSED

PENDING_JD_LIVE_ACCEPTANCE

JD_LIVE_ACCEPTANCE_FAILED

A build involving voice, Desktop UI, TTS, continuous conversation, memory/context, search presentation, provider behavior, or protected execution is not fully complete until either:

JD live acceptance passes; or

JD explicitly waives live acceptance for that build.

7.16 Interactive JD Live Test and Backend Evidence Verification Law

This law strengthens Section 7.15 for interactive JD live testing and backend evidence verification. It must not be treated as a substitute for Section 7.15; both laws apply.

For any build involving voice, Desktop UI, TTS, PH1.X, PH1.M, Voice ID, memory, context, routing, protected classification, search presentation, or user-visible behavior, Codex must use interactive JD live testing when JD is available.

Codex must not ask JD to “just test generally.”

Codex must guide JD one step at a time.

For every live test, Codex must:

tell JD exactly what to say or type;

tell JD when to wait;

confirm the latest app was rebuilt from current HEAD;

prove one current app instance and one adapter/runtime owner;

capture the exact transcript or typed input;

capture the visible result;

capture whether TTS was heard when expected;

inspect backend evidence immediately after the prompt;

confirm the correct owner made the decision;

confirm the correct packet/directive/evidence landed;

mark the test PASS or FAIL;

if failed, identify root owner and repair there;

rebuild/restart latest app after any fix;

rerun the failed live test;

commit only after JD live acceptance and backend evidence both pass.

Cargo tests, unit tests, mocked tests, fixture tests, xcodebuild, /healthz, and app launch proof do not replace JD live testing when the behavior is user-visible.

A live test only counts as passed when all three are true:

JD confirms the visible/audible behavior is correct;

Codex confirms backend evidence matches the expected route;

the latest current app/current HEAD was used.

For PH1.X / PH1.M / Voice ID / context builds, Codex must also prove:

ActiveContextPacket or equivalent evidence;

HumanConversationDirective or equivalent directive;

selected candidate and rejected candidates where applicable;

confidence / ambiguity / reason code where applicable;

PH1.M memory evidence only when memory is actually used;

Voice ID evidence is speaker evidence only, not authority;

Desktop did not decide meaning;

Adapter did not become the context or memory brain;

protected fail-closed behavior remains preserved.

If JD reports that the visible behavior did not work, Codex must report:

JD_LIVE_ACCEPTANCE_FAILED

Codex must not claim completion until the failure is repaired and retested.

If backend evidence is missing, incomplete, in the wrong owner, or inconsistent with the visible behavior, Codex must report:

BACKEND_EVIDENCE_VERIFICATION_FAILED

Codex must not claim completion until the evidence path is repaired and retested.

Every final report must include:

exact JD prompt;

exact captured transcript or typed input;

visible result;

audible result if voice/TTS expected;

backend packet/evidence refs;

owner decision proof;

root cause if repaired;

latest app/current HEAD proof;

JD live acceptance result;

final clean tree proof.

Final rule:

A Selene build is not truly accepted because Codex says tests passed.

A Selene build is accepted only when JD live behavior and backend evidence agree.

8. Determinism, Fail-Closed, and Engineering Quality

8.1 Determinism Lock

Determinism is required for protected execution paths.

Protected execution paths include:

simulation execution;

access control;

authority validation;

business state mutation;

ledger writes;

artifact activation;

onboarding progression;

provider promotion or demotion;

external message sending with real-world effect;

irreversible actions.

Rules for deterministic protected execution:

stable ordering only;

no randomness;

no hidden fallbacks;

no time-based drift unless explicitly part of the approved deterministic process;

same input must produce same output unless explicitly approved;

required simulation and authority checks must pass;

fail closed on inconsistency.

This lock does not make ordinary public answers deterministic.

Normal chat, public search, weather/time presentation, summaries, translations, advisory analysis, and report drafting remain probabilistic/adaptive unless JD explicitly authorizes a deterministic simulation for that exact process.

8.2 Fail-Closed Lock

Protected execution missing required inputs must refuse with a registered reason code.

Protected execution must not continue when required simulation, authority, state, or audit inputs are missing.

No protected execution may use:

silent correction;

guessing;

hidden continuation;

best-effort mutation.

Public/advisory answers are different.

For public/advisory answers, missing information should usually result in:

a clarification question;

a partial answer with stated limits;

a degraded answer;

a safe refusal only if required by safety, privacy, or policy.

Examples:

Missing weather location = ask which city or use available context where lawful.

Missing payroll authority = fail closed.

GDELT unavailable = public answer may continue with degraded corroboration note.

Missing simulation for salary change = fail closed for the salary change only.

8.3 No Parallel Bypass Paths

Protected execution must stay inside canonical deterministic boundaries:

PH1.X protected-action classification;

access control;

simulation verification when applicable;

SimulationExecutor;

deterministic mutation;

audit logging.

No shortcut or bypass protected-execution paths are allowed.

This rule applies to protected/business execution.

It does not require normal public chat, public search, weather, time, translation, summaries, or advisory reports to pass through SimulationExecutor.

Public-answer paths may use probabilistic reasoning and read-only public providers, but must not mutate business state or execute protected actions.

8.4 Stop-on-Uncertainty Rule

If uncertain about:

contract meaning;

state order;

gate sequencing;

simulation boundaries;

engine interaction;

whether old code may be changed;

stop and ask JD.

Guessing is forbidden.

8.5 Engineering Quality Standard

All code delivered by Codex must aim for production-grade quality.

Rules:

warnings must be fixed, not ignored, unless explicitly approved by JD;

no shortcuts;

no lazy fixes;

no temporary hacks;

no weak workaround patch if proper implementation is achievable;

preserve full required functionality;

do not silently reduce scope to make a patch easier;

if the proper implementation is too large, stop at a clean architectural boundary and report it rather than shipping a weak partial.

Clean builds are the target.

Warning-free is the default expectation.

8.6 No Surprise Refactors

Codex must not perform unrequested:

renames;

file moves;

cleanup sweeps;

formatting churn;

convenience rewrites;

broad rewrites;

“while I’m here” changes.

Clarification:

Directly related stale-surface cleanup and correct-owner repair required by Sections 5.6, 5.7, 5.8, and 5.9 is allowed only inside approved scope and with proof. It is not permission for broad unrelated cleanup sweeps.

8.7 Hard Fail Conditions

Immediate stop on:

attempted Python usage;

missing required JD approval;

baseline failure before edits on a change run;

out-of-scope file touched;

unknown reason code introduced;

state or gate ordering changed without approval;

previous working code being overridden without JD approval.

9. Commit, Push, and Final Cleanliness

9.1 Mandatory Cleanliness Gates

For change runs:

tests/checks must pass;

if ledger update is required by repo law, do it;

commit only relevant files;

push to origin;

prove clean tree again.

For read-only runs:

no commit;

no edits;

prove clean tree at end.

No local-only real work.

No leftover untracked junk.

No unfinished hidden changes.

9.2 Commit Discipline

One coherent changeset per run unless JD explicitly says otherwise.

Commit message should reflect run intent.

Do not mix unrelated workstreams.

9.3 Final Safety Outcome

Selene must not suffer:

hidden drift;

silent corruption;

repeated replacement of previously built work;

contract breakage without JD approval;

messy dirty-tree iteration loops.

10. Final Override

These laws apply by default unless JD explicitly overrides them in-thread.
