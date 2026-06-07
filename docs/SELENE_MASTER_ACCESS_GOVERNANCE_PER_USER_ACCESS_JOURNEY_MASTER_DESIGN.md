# Selene Master Access Governance + Per-User Access Journey Master Design

Document status:

- MASTER_DESIGN
- NOT_RUNTIME_IMPLEMENTATION
- PENDING_GRAND_ARCHITECTURE_RECONCILIATION

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only master design.

No runtime code was changed.

This document does not authorize implementation.

The Master Access repo-truth extraction remains the factual base:

- `docs/SELENE_MASTER_ACCESS_ENGINE_REPO_TRUTH_FUNCTIONALITY_EXTRACTION_MASTER_DESIGN.md`

This document defines future Master Access Governance + Per-User Access Journey architecture pending Grand Architecture Reconciliation.

Future implementation requires repo-truth activation, approved file scope, tests, backend evidence, PH1.D provider-off/fake-provider proof, PH1.WRITE validation proof, Access/Governance proof, audit proof, and JD live acceptance.

This document is an upgrade design. It does not replace current repo truth, implement PH1.ACCESS or PH2.ACCESS, change schemas, add migrations, retire old paths, mutate runtime state, or authorize access changes from documentation alone.

## 1. Executive Target

Selene must have a global-grade access system that can:

- define master access templates,
- define role and permission matrices,
- define tenant, workspace, company, and entity scopes,
- generate per-user access instances when a user is onboarded into a scoped role,
- support temporary, permanent, one-shot, revoked, suspended, restricted, retired, and reactivated access,
- support access changes caused by promotion, demotion, leave, sickness, termination, resignation, retirement, return-to-work, or reactivation,
- support personal Selene access for life after company access is removed where policy allows,
- support Position-to-Access co-authoring when positions are created,
- use GPT-5.5, PH1.D, PH1.N, PH1.X, and PH1.WRITE for human-friendly access requests and explanations,
- keep access mutation deterministic, governed, confirmed, simulated, and audited.

The key product law:

Selene may understand the human request probabilistically.

Selene must change access deterministically.

The human should be able to say "Promote Tom to manager" or "Give Sarah payroll access for two hours" and experience one natural Selene journey. Internally, that journey must separate meaning proposal, route validation, access authority, role/template truth, financial truth, approval, simulation, confirmation, and audit.

## 2. Current Repo-Truth Foundation

Current repo truth shows that Master Access is worth keeping.

The extracted foundation is:

- PH1.ACCESS currently owns master/schema/gate decision behavior.
- PH2.ACCESS currently owns per-user access instances and override writes.
- Storage/migrations contain access instances, overrides, AP schemas, overlays, board policies, votes, authoring review, and compile lineage.
- SimulationExecutor enforces some access gates before protected paths.
- Access instance storage, override records, and AP schema records already provide a strong base for enterprise access.
- Current Access is not yet a complete enterprise-grade role hierarchy engine.

Current weaknesses that this future design must address:

- `role_template_id` is mostly a string.
- Some role logic uses string matching such as owner/admin.
- Workspace scope is missing as a first-class access dimension.
- Company/entity scope is partial.
- Field-level permissions are weak.
- Private data read gate is partial.
- Protected execution gate integration is partial.
- Approval threshold resolver is partial.
- Revocation, expiration, suspension, retirement, and reactivation lifecycle is incomplete.
- Termination, resignation, retirement, and personal-access fallback flow is missing.
- PH1.WRITE access-denial wording is missing or underdefined.
- PH1.D/PH1.N access proposal path is missing.
- Adapter wrong-owner semantic classification remains a risk.

This document keeps the current PH1.ACCESS/PH2.ACCESS split and expands it into a complete access governance journey.

## 3. Master Access vs Per-User Access

Master Access owns:

- access templates,
- role templates,
- permission templates,
- role hierarchy,
- tenant, workspace, company, and entity scopes,
- field-level permissions,
- private data read gates,
- protected action access gates,
- approval and escalation policies,
- override policy,
- revocation, suspension, expiration, retirement, and reactivation policy,
- access audit rules.

Per-user Access owns:

- the individual access instance for a specific user,
- the user's current role/template assignment,
- active overrides,
- temporary access,
- one-shot access,
- permanent access,
- revoked, suspended, restricted, expired, or retired state,
- compiled lineage from master templates,
- user-specific access lifecycle.

Required rule:

Every onboarded user/person who needs scoped company/workspace access must receive a per-user access instance.

That per-user access instance belongs only to that user.

It must not be shared.

It must not be inferred from link text, voice identity, device identity, or position name alone.

Example:

Master template:

Warehouse Manager can view warehouse roster and inventory reports, but cannot view payroll.

Per-user instance:

Tom in tenant ABC Wines has Warehouse Manager access.

Access gate:

Tom can view warehouse roster.

Tom cannot view payroll.

## 4. Access Owner Split

PH1.POSITION owns:

- position title,
- job family,
- department,
- hierarchy,
- reporting line,
- required fields,
- training/certifications,
- position requirements,
- salary band reference where HR/Payroll owns truth.

Access/Governance owns:

- what that position can see,
- what that position can do,
- approval limits,
- private data access,
- payroll, finance, and HR permissions,
- role/access templates,
- field-level permissions,
- protected action access requirements.

PH1.ONB owns:

- onboarding session,
- missing field collection,
- human-guided setup,
- readiness checklist,
- access handoff.

Access/Governance creates the per-user access instance.

Rule:

Position describes the job.

Access grants the permissions.

Onboarding coordinates the human setup.

Finance/Budget owns money truth.

Payroll/HR owns employment/payroll truth.

Scheduler/Roster owns work availability, shifts, and roster truth.

Voice ID and Device/Human Presence provide evidence only.

Desktop/iPhone render and submit bounded user input only.

Adapter transports only.

GPT/OpenAI proposes meaning only.

## 5. Position Creation + Access Template Co-Authoring Journey

Future user experience should feel unified:

User says:

"Create a Warehouse Manager position."

Selene guides one full journey:

1. understand requested position,
2. define position details,
3. propose required fields,
4. propose access template,
5. authorized person reviews access,
6. confirm permissions,
7. PH1.POSITION stores position truth,
8. Access/Governance stores approved access template,
9. future onboarding uses both.

Behind the scenes, PH1.POSITION and Access/Governance remain separate owners.

Selene may suggest likely access using GPT-5.5 through PH1.D, but Access/Governance validates and stores.

Example:

Position engine:

- Warehouse Manager
- department = warehouse
- reports to = Operations Manager
- requires forklift certification
- requires safety training
- uses roster group

Access engine:

- can view warehouse roster,
- can approve stock adjustments up to an approved limit,
- can view inventory reports,
- cannot view payroll,
- cannot grant access.

Future onboarding:

Onboard Tom as Warehouse Manager

-> PH1.ONB loads position requirements.

-> Access creates Tom's per-user access from the approved Warehouse Manager template.

## 6. Per-User Access Creation During Onboarding

Whenever a person is onboarded into a scoped role, a per-user access instance must be generated.

Future flow:

Onboard Tom as Warehouse Supervisor

-> PH1.ONB collects required fields.

-> PH1.POSITION defines requirements.

-> Access template is selected.

-> Access/Governance validates template and scope.

-> Per-user access instance is created.

-> Tom receives actual scoped access.

-> Audit records access creation.

PH1.ONB may coordinate readiness.

PH1.ONB must not define access policy.

PH1.ONB must not guess permissions.

PH1.ONB must not grant company access from role wording alone.

Access/Governance owns the access policy and access instance.

This is the key access/onboarding law:

Onboarding prepares the person.

Access gives the keys.

Position defines the job.

## 7. Access Lifecycle Types

Selene must support the full access lifecycle:

- OneShot access,
- Temporary access,
- Permanent access,
- Revoke access,
- Suspended access,
- Expired access,
- Restricted access,
- Active access,
- Retired access,
- Reactivated access,
- Personal/casual access fallback.

Existing repo terms map to part of this model:

- `AccessOverrideType::OneShot`
- `AccessOverrideType::Temporary`
- `AccessOverrideType::Permanent`
- `AccessOverrideType::Revoke`
- `AccessLifecycleState::Restricted`
- `AccessLifecycleState::Active`
- `AccessLifecycleState::Suspended`

Required behavior:

OneShot:

Access for one specific action, one bounded report view, or one bounded event.

Temporary:

Access for a bounded time window.

Permanent:

Access persists until changed, revoked, retired, suspended, or superseded.

Revoke:

Access is removed or blocked.

Suspended:

Access is paused because the user is suspended, on leave, unavailable, under review, or restricted by policy.

Expired:

Temporary access window has ended.

Restricted:

User remains known but sensitive actions or private data reads are blocked or require step-up.

Retired:

Company/workspace access is retired because employment or relationship ended.

Reactivated:

Company/workspace access is re-created or resumed after validation against current templates and current employment/position truth.

Personal/casual fallback:

User may continue using Selene for general/personal use if allowed, but company access is removed.

No half-built dead wood:

If the current repo has partial override states, the future build plan must include the missing stacks required to make each lifecycle type fully functional.

## 8. Access Changes: Temporary, Permanent, Revoke, Suspend

Temporary access:

"Give Sarah payroll access for two hours."

-> PH1.D/PH1.N understand.

-> PH1.X marks access mutation.

-> Access/Governance checks requester authority.

-> Confirmation is required.

-> Access creates temporary override.

-> Audit records the grant and expiry.

-> Access expires automatically.

Permanent access:

"Make Tom a Manager permanently."

-> Position update may occur.

-> Access template changes.

-> Access updates Tom's per-user instance.

-> Approval may be required.

-> Audit records old/new lineage.

One-shot access:

"Allow Lisa to approve this one stock adjustment."

-> Bounded one-action access.

-> Expires after use or timeout.

-> Does not update the role template.

Revoke access:

"Remove Tim's supplier bank access."

-> Access verifies requester authority.

-> Revoke override or access update is created.

-> Audit records what changed and why.

Suspend access:

"Tom is suspended from work."

-> Company/workspace access becomes suspended or restricted.

-> Personal Selene access may remain.

-> Audit records the suspension and policy reason.

## 9. Leave / Holiday / Sick / Off-Work Access Posture

Access should handle non-working status through policy, not one-size-fits-all removal.

Events include:

- annual leave,
- sick leave,
- unpaid leave,
- suspension,
- maternity/paternity leave,
- long service leave,
- off-shift,
- outside rostered hours,
- unavailable,
- no longer actively working today.

Access behavior should be policy-driven.

Options:

1. No access change.
2. Risky-action restriction.
3. Step-up required.
4. Supervisor notification.
5. Temporary hold.
6. Emergency exception.

Recommended default:

Do not automatically remove all access for normal holidays or sick leave.

Apply policy by role, action, risk, company settings, and jurisdiction.

Examples:

Tom on annual leave:

- can still view general company announcements,
- cannot approve payroll unless policy explicitly allows,
- if he attempts a high-risk action, Selene explains and may notify a supervisor where policy allows.

Sarah on sick leave:

- may still use personal Selene,
- company actions may be limited by policy.

## Master Template Change Propagation

Master Access Templates define the approved role/permission model.

Per-User Access Instances are compiled from those templates for specific users.

When an authorized person changes a Master Access Template, Selene must ask how to apply the change:

- future users only,
- all existing users using this template,
- selected existing users only,
- draft only, do not apply yet,
- create new template version and require review.

Example:

"Manager template now gets inventory approval up to $2,000."

Selene must ask:

"Should this apply only to future Managers, all current Managers, or selected Managers?"

If applying to existing users, Selene must:

- preview affected users,
- preview permission changes,
- identify overrides that may conflict,
- preserve existing user-specific overrides unless policy says otherwise,
- require authority/approval where needed,
- create new template version,
- recompile affected per-user access instances,
- audit every affected change,
- allow rollback/revert where policy permits.

No master template change may silently mutate all users without explicit scope confirmation.

Propagation behavior:

- Future users only means the active template version changes for new per-user access instances, while current compiled instances remain unchanged unless policy later migrates them.
- All existing users means every current per-user instance compiled from the old template must be previewed, reconciled against overrides, recompiled, audited, and optionally rollback-capable.
- Selected existing users means the authorized requester chooses scoped users or groups after a safe preview.
- Draft only means the template proposal remains inactive and cannot affect live access.
- New template version means lineage must preserve old and new versions, who approved them, when they become active, and which per-user instances were compiled from each version.

Bulk propagation must be treated as a high-blast-radius access mutation. It requires explicit scope confirmation, simulation/capability proof, approval where policy requires, and audit.

## Per-User Access Instance Meaning

Per-user access is not a simple on/off flag.

A Per-User Access Instance includes:

- the user,
- tenant/company/workspace scope,
- current role/template assignment,
- compiled permissions,
- active overrides,
- one-shot grants,
- temporary grants,
- permanent grants,
- revoked/suspended/restricted state,
- field-level permissions,
- spend/approval limits where applicable,
- audit lineage.

Example:

Tom is a Clerk.

Clerk template does not allow company-profit access.

Authorized manager allows Tom to view company profit one time.

This must become:

- OneShot override,
- user = Tom,
- action/resource = view company profit,
- scope = specific report or financial summary,
- expiry = after one view/action or fixed time,
- audit = required.

This does not change the Clerk template.

This does not give all Clerks profit access.

This does not permanently upgrade Tom.

Per-user overrides must preserve their intent:

- a one-shot override remains one-shot,
- a temporary override expires,
- a permanent override must be explicitly approved,
- a revoke override must not be erased by template propagation unless policy and confirmation allow it,
- a suspended/restricted state must not be bypassed by role-template updates.

## Step-Up Verification For Sensitive Access

Some access requires step-up verification.

Access/Governance owns the requirement:

- face verification required,
- fingerprint verification required,
- passcode required,
- secure device confirmation required,
- manager/admin approval required,
- dual approval required.

Identity, Device, and Human Presence owners provide the evidence.

Access must not perform biometric verification itself.

Access consumes verified proof.

For users without biometric-capable devices, Selene must support fallback options where policy allows:

- secure passcode,
- approved device confirmation,
- admin approval,
- manager approval,
- alternative step-up flow.

Examples:

"Show payroll report."

-> requires step-up.

"Approve $50,000 payment."

-> may require step-up plus second approver.

"View confidential HR file."

-> may require biometric/passcode and audit.

Step-up proof must be fresh enough, scoped to the action, tied to the session/user/device where policy requires, and audited. A stored voice profile, device id, or provider confidence score cannot stand in for verified step-up unless the canonical evidence owner has produced an approved proof packet.

## Leave / Holiday / Sick / Off-Work Access Posture

Access behavior must respond to work status.

Events may include:

- annual leave,
- sick leave,
- unpaid leave,
- suspension,
- maternity/paternity leave,
- long service leave,
- off-shift,
- outside rostered hours,
- unavailable,
- no longer actively working today.

Access must be policy-driven.

Possible policies:

1. No change: user keeps normal access.
2. Risky-action restriction: user can read allowed information but cannot perform risky writes/approvals while on leave.
3. Step-up required: sensitive actions require extra verification or manager approval.
4. Supervisor notification: if user attempts restricted action while on leave/off-shift, supervisor/admin may be notified where policy allows.
5. Temporary hold: certain company/workspace permissions are paused until return date.
6. Emergency exception: emergency/break-glass policy may allow limited action with strong audit.

Recommended default:

Do not automatically remove all access for normal holidays or sick leave.

Apply policy by role, action, risk, company settings, and jurisdiction.

Example:

Tom is on annual leave and tries to approve payroll.

Selene checks leave/scheduler/roster truth.

Access policy may deny, require step-up, or notify supervisor.

PH1.WRITE explains naturally.

Work-status truth belongs to HR/Payroll, Scheduler/Roster, or the canonical employment owner. Access consumes that truth for access posture; it does not invent leave status.

## 10. Termination / Resignation / Retirement / Revert To Personal Access

Termination/resignation/retirement is required core architecture, not optional future hand-waving.

Flow:

Termination/resignation/retirement event

-> HR/Payroll owns employment truth.

-> Position owner updates role/status.

-> Scheduler/Roster removes active work scheduling.

-> Access/Governance revokes, retires, or suspends company/workspace access.

-> PH1.M/private/company memory scope is rechecked.

-> Personal Selene access remains if policy allows.

-> Company private data access is removed.

-> Audit records the transition.

-> Optional offboarding notification routes through PH1.BCAST/PH1.DELIVERY.

Required rule:

Selene is intended to support lifetime general/personal use where allowed.

Leaving a company should not necessarily delete the person from Selene.

It should remove or retire company/tenant/workspace access while preserving personal/casual Selene access if permitted.

Example:

Tom resigns from ABC Wines.

-> Tom loses ABC Wines company access.

-> Tom cannot see ABC Wines payroll, staff, customers, supplier, finance, or internal documents.

-> Tom may still use Selene personally if allowed.

-> Tom's company memory scope is detached, locked, archived, or restricted according to policy.

-> Access audit records the transition.

## Termination / Resignation / Retirement / Personal Selene Fallback

Termination, resignation, and retirement must trigger access transition.

This is not optional future hand-waving.

Flow:

Employment status changes:

termination / resignation / retirement

-> HR/Payroll owner records employment truth.

-> Position owner updates employment/position state.

-> Scheduler/Roster removes active work scheduling.

-> Access/Governance retires/revokes/suspends company/workspace access.

-> PH1.M/private/company memory scope is rechecked.

-> Personal Selene access remains where policy allows.

-> Audit records the transition.

Rule:

Selene may support lifetime general/personal use.

Leaving a company must remove company/tenant/workspace access, not necessarily delete the person from Selene.

Example:

Tom resigns from ABC Wines.

-> Tom loses ABC Wines company access.

-> Tom cannot see ABC Wines payroll, staff, customers, suppliers, finance, or internal documents.

-> Tom may still use Selene personally if allowed.

-> Tom's company memory scope is detached, locked, archived, or restricted according to policy.

-> Access audit records the transition.

Company offboarding and personal Selene continuity must be distinct. A termination event may remove company scopes, revoke workspace tokens, pause business delivery, and archive company memory eligibility without deleting the person's personal identity, personal memory, or public-safe Selene access where policy allows.

## Rehire / Return-To-Work / Reactivation

If a person returns, Selene must not blindly restore old access.

Flow:

Tom returns to ABC Wines.

-> old access history is preserved.

-> current role/position is resolved.

-> current access template is selected.

-> authorized person confirms reactivation.

-> Access creates or reactivates per-user access from current approved template.

-> previous overrides are reviewed, not automatically restored.

-> audit records the reactivation.

Rules:

- historical records remain for audit,
- old company access is not restored without validation,
- current templates win unless policy says otherwise,
- personal Selene access may have continued separately,
- company memory visibility is restored only according to current access,
- revoked, suspended, or risky historical overrides must not silently reappear,
- previous manager/admin approvals must be revalidated if policy requires.

Rehire is a new access decision using historical evidence, not a blind rollback.

## 11. Promotion / Demotion / Position Change Flow

User says:

"Promote Tom to manager and give him the usual manager access."

Probabilistic layer:

PH1.D / GPT-5.5 understands messy intent.

PH1.N extracts:

- user/person = Tom,
- event = promotion,
- new position = manager,
- access change = manager template,
- protected/access-changing risk.

PH1.X validates:

- access-changing request,
- role/position update,
- permission required,
- simulation required,
- confirmation required.

Deterministic layer:

PH1.POSITION updates position only if authorized.

Access/Governance validates the Manager access template.

Access updates Tom's per-user access instance.

Audit records old and new access lineage.

PH1.WRITE explains naturally.

User hears:

"Tom is now set as Manager, and his access has been updated to the approved Manager template."

No need to expose all the plumbing to the user.

No promotion, demotion, or position change may grant access from title alone.

## 12. Master Role Template Registry

Selene needs a proper enterprise role/template registry to replace string-guessing.

Current risk:

`role_template_id` is mostly a string.

Some code uses string contains checks such as owner/admin.

Future registry must include:

- role_template_id,
- role_name,
- role_family,
- position linkage,
- tenant/company/workspace scope,
- permissions list,
- forbidden permissions,
- approval requirements,
- field-level access,
- private data access,
- protected action access,
- spend and approval limits,
- maximum authority level,
- delegation rules,
- review cadence,
- status: draft/active/retired,
- schema version,
- audit refs.

Rules:

No role name alone grants permission.

No string matching owner/admin as authority.

Role templates must be explicit, versioned, reviewed, and auditable.

## 13. Role Hierarchy And Position Hierarchy

Position hierarchy belongs to PH1.POSITION.

Access hierarchy belongs to Access/Governance.

They must link but not merge.

Position hierarchy example:

CEO

-> Operations Manager

-> Warehouse Manager

-> Warehouse Supervisor

-> Warehouse Worker

Access hierarchy example:

basic read

-> write

-> approve

-> execute

-> admin

-> owner

-> emergency/break-glass

A high position may imply a candidate access template, but Access/Governance must validate it.

Example:

CFO position suggests finance/payroll access.

Access/Governance decides exact finance/payroll permissions, approval limits, spend limits, confidential access, and dual-approval requirements.

## 14. Tenant / Workspace / Company / Entity Scope

Access must support:

- tenant scope,
- company/entity scope,
- workspace scope,
- department scope,
- region/country scope,
- project scope,
- role scope,
- resource scope,
- field scope.

Current repo:

- tenant scope exists,
- workspace scope is missing,
- company/entity scope is partial.

Future rule:

A user may have access in one workspace but not another.

A user may have payroll access in one company but not another.

A user may belong to multiple tenants/workspaces with separate per-user access instances or scoped access records.

Tenant/company/workspace access must be explicit. A person's access in ABC Wines must never imply access in another company, tenant, workspace, department, project, roster, finance system, supplier record, customer file, or payroll scope.

## 15. Field-Level Permission Model

Sensitive fields include:

- salary,
- payroll,
- bank details,
- tax numbers,
- health numbers,
- superannuation/member numbers,
- HR notes,
- disciplinary records,
- customer financials,
- supplier bank details,
- legal documents,
- confidential reports.

Access must decide:

- who can view,
- who can edit,
- who can approve,
- who can export,
- who can send,
- who can delegate,
- who can revoke.

Examples:

Warehouse Manager can view roster, not salary.

HR Manager can view employee HR records, not approve payroll unless template says so.

Payroll Officer can edit pay details, but not grant access.

CFO can view finance/payroll reports with elevated audit.

Field access must be resource-scoped and purpose-aware where policy requires. Field-level access must not be inferred from general role title or broad company membership.

## 16. Private Data Read Gate

Private data read examples:

- "Show Tim's salary."
- "What was our gross margin last month?"
- "Show supplier bank details."
- "Show customer invoices."

Flow:

PH1.D/PH1.N may understand the request.

PH1.X classifies private company data read.

Access/Governance checks user, tenant, workspace, resource, field, and purpose.

Source owner retrieves data only if allowed.

PH1.WRITE explains or denies.

No private read from normal chat.

No private read because voice matched.

No private read from role name alone.

No private read without audit.

No private read may bypass the source owner. Access answers whether the user may read. The source owner answers what the source truth is.

## 17. Protected Execution Gate

Protected execution examples:

- approve payroll,
- increase salary,
- refund customer,
- create supplier,
- grant access,
- revoke access,
- send confidential file,
- change roster,
- update employee role.

Flow:

PH1.D/PH1.N may understand.

PH1.X marks protected action.

Access validates scope.

Authority validates action permission.

SimulationExecutor validates active simulation.

Confirmation is required where policy says.

Audit is required.

Execution occurs only by canonical owner.

Access alone is not execution authority.

An access check may be necessary for protected execution, but it is never sufficient by itself. Protected execution must still pass authority, simulation, confirmation, source-owner, and audit gates.

## 18. PH1.D / GPT-5.5 + PH1.N Access Proposal Path

Users may make voice/type natural access requests:

- "Give Tom manager access."
- "Let Sarah see payroll for today."
- "Remove Tim from supplier banking."
- "Promote Lisa to warehouse supervisor."
- "Put Jack back to normal user."
- "Tom has resigned, remove company access."

PH1.D/GPT-5.5 may propose:

- person,
- role,
- access template,
- scope,
- time limit,
- action,
- risk,
- missing fields,
- clarification question.

PH1.N extracts candidates.

PH1.X validates lane/risk.

Access/Governance makes deterministic decision.

PH1.WRITE explains.

OpenAI/GPT-5.5 must never grant access.

OpenAI/GPT-5.5 must never silently choose propagation scope for template changes, silently classify someone as owner/admin, or convert temporary/one-shot access into permanent access.

## 19. PH1.WRITE Access Explanation Boundary

PH1.WRITE owns final user-facing access wording.

Access emits:

- decision,
- reason code,
- required gate,
- missing approval,
- escalation path,
- audit ref.

PH1.WRITE says:

- "Tom does not have payroll access."
- "You can grant temporary access for two hours, but this requires manager approval."
- "Sarah's company access has been suspended while she is on leave."
- "Tom's ABC Wines access has been removed, but his personal Selene access remains."

No raw access reason-code dumping unless developer mode says so.

PH1.WRITE must not soften denial into implied approval, hide that approval is required, or phrase a pending proposal as if it has already changed access.

## 20. Approval / Escalation Workflow

Access changes may require approval.

Approval types:

- single approver,
- manager approval,
- HR approval,
- owner/admin approval,
- board quorum,
- N-of-M,
- unanimous board,
- dual control,
- emergency/break-glass.

Needed for:

- high access roles,
- salary/payroll/finance,
- HR data,
- customer/supplier bank data,
- permanent access changes,
- revocation of critical access,
- temporary elevation,
- emergency access,
- bulk template propagation,
- spend authority above threshold.

Access must support approval case lifecycle:

requested

-> pending approval

-> approved / rejected / expired

-> applied or denied

-> audit

Current repo has board policies/votes but threshold resolution is partial.

Future build must complete it.

## Budget / Spend / Approval Authority

Access must participate in spend and approval authority, but Finance/Budget owns financial truth.

Access/Governance owns:

- who may approve spending,
- approval limits,
- category limits,
- department limits,
- project limits,
- period limits,
- dual/multi-approver rules,
- board/CEO/CFO/chairman escalation,
- reimbursement authority,
- salary/payroll approval authority.

Finance/Budget owns:

- actual budgets,
- actual spend,
- remaining balances,
- salary calculations,
- reimbursement calculations,
- finance records,
- budget overrun truth.

Examples:

- Manager can approve reimbursements up to $500.
- Department stationary budget is $500 per quarter.
- CFO can approve finance payments up to $50,000.
- Payments above $100,000 require CEO + CFO.
- Some payments require 2 approvers or 3 approvers.
- If company budget is exceeded, Selene routes to CEO/chairman/board approval depending company policy.

Access answers:

"Is this person allowed to approve this amount/category?"

Finance/Budget answers:

"Is the money/budget available and what is the financial impact?"

SimulationExecutor executes only when Access, Authority, Finance/Budget, confirmation, and simulation gates pass.

Budget/spend authority must include multi-approver/board approval support where policy requires. A role title such as Manager, CFO, or CEO cannot grant spend authority without an approved template and active per-user access instance.

## 21. Access Lifecycle Assistant

Selene must help answer:

- What access does Tom have?
- Why can't Sarah see payroll?
- Who approved this access?
- When does this temporary access expire?
- Who has CFO-level access?
- Remove Tim's company access.
- Put Tom back to casual user.
- Did Lisa's access update after promotion?
- What changed in access last week?
- Which users will be affected if I update the Manager template?
- Does Tom have any one-shot or temporary overrides?
- Can this access change be rolled back?

Owner map:

- Access = access truth.
- Position = job/position truth.
- HR/Payroll = employment state.
- Scheduler/Roster = leave/off-shift truth.
- Finance/Budget = money truth.
- PH1.WRITE = human explanation.
- PH1.D/PH1.N = messy query understanding.
- PH1.X = route/risk validation.

Access lifecycle answers must be access-scoped and audit-aware.

## 22. Relationship With Onboarding

Whenever onboarding creates a scoped user/person, Access must generate a per-user access instance after requirements pass.

Correct flow:

Onboard Tom as Warehouse Supervisor

-> ONB collects required fields.

-> Position defines requirements.

-> Access template selected.

-> Access/Governance validates template.

-> Per-user access instance created.

-> Tom gets actual scoped access.

-> Audit records access creation.

Onboarding prepares the person.

Access gives the keys.

Position defines the job.

ONB must not bypass Access policy.

ONB must not reuse another person's access instance.

ONB must not infer access from invite link text.

ONB must not preserve company access after termination unless Access/Governance explicitly allows it.

## 23. Relationship With Position Engine

Creating a position should feel like one complete Selene flow, but owners remain split.

Position creation journey:

create position

-> define job requirements

-> suggest access template

-> authorized person reviews permissions

-> Access/Governance stores approved access template

-> future onboarding uses position + access template together.

This document must not implement Position.

PH1.POSITION extraction/design is required next.

Position-to-Access co-authoring must never merge the truth owners. Position stores job truth. Access stores permission truth.

## 24. Relationship With Link / Reminder / Broadcast

PH1.LINK may carry invite/access context but must not grant access.

PH1.REM may remind about access review, expiry, return-to-work, reactivation, or review deadlines but must not decide access.

PH1.BCAST/DELIVERY may notify about access changes but must not grant access.

Access remains the permission owner.

Examples:

Expired access reminder -> PH1.REM timing.

Supervisor notification -> PH1.BCAST/DELIVERY.

Access revoke -> Access/Governance.

Access review due -> PH1.REM can schedule, PH1.WRITE can explain, Access decides and records.

## 25. Security / Privacy / Audit Standards

Required standards:

- least privilege by default,
- deny by default where scope is unclear,
- explicit templates over role-name guessing,
- field-level access for sensitive data,
- workspace/tenant/company scope,
- access audit for every grant, update, revoke, suspend, override, private read, protected execution precheck, bulk template propagation, recompile, rollback, reactivation, and termination transition,
- immutable lineage for who changed what and why,
- no access by voice alone,
- no access by link alone,
- no access by GPT-5.5 proposal alone.

Bulk template changes must audit:

- old template version,
- new template version,
- propagation scope,
- affected user list,
- skipped user list,
- conflicting overrides,
- approvers,
- simulation/capability refs,
- recompile result,
- rollback token/ref where policy permits.

## 26. Required Logical Packets

Future logical packets:

- MasterAccessTemplatePacket
- RoleTemplatePacket
- PermissionMatrixPacket
- FieldPermissionPacket
- PerUserAccessInstancePacket
- AccessLifecycleStatePacket
- AccessOverrideRequestPacket
- AccessTemporaryGrantPacket
- AccessPermanentGrantPacket
- AccessOneShotGrantPacket
- AccessRevokePacket
- AccessSuspendPacket
- AccessRetirePacket
- PersonalAccessFallbackPacket
- LeaveAccessPosturePacket
- TerminationAccessTransitionPacket
- RehireAccessReactivationPacket
- MasterTemplateChangePropagationPacket
- PerUserAccessRecompilePacket
- PositionAccessTemplateProposalPacket
- AccessApprovalCasePacket
- AccessDecisionPacket
- PrivateDataReadGatePacket
- ProtectedExecutionAccessGatePacket
- StepUpVerificationRequirementPacket
- SpendApprovalAuthorityPacket
- MultiApproverDecisionPacket
- AccessAuditEvidencePacket
- AccessExplanationPacket

Codex must later map these to repo equivalents.

Do not claim these currently exist.

## 27. Example End-To-End Flows

Example A - onboarding creates per-user access:

Onboard Tom as Warehouse Supervisor

-> ONB collects fields.

-> Position requirements loaded.

-> Access template selected.

-> Access validates.

-> Tom's per-user access instance created.

Example B - promotion:

"Promote Tom to Manager."

-> Position updates Tom's position if authorized.

-> Access updates Tom's per-user access to Manager template.

-> Audit records lineage.

-> PH1.WRITE explains.

Example C - temporary access:

"Give Sarah payroll access for two hours."

-> access mutation.

-> authority check.

-> approval if required.

-> temporary override.

-> automatic expiry.

Example D - resignation:

"Tom resigned."

-> HR/employment truth updated by HR owner.

-> Access retires Tom's company access.

-> personal Selene access remains if allowed.

-> audit.

Example E - leave/off-shift attempt:

Tom is on annual leave and tries to approve payroll.

-> leave/schedule owner says Tom is on leave.

-> Access policy requires denial/step-up/supervisor notification.

-> PH1.WRITE explains.

Example F - private read:

"Show Tim's salary."

-> PH1.X marks private HR data read.

-> Access checks field-level salary permission.

-> source owner returns or denies.

-> PH1.WRITE explains.

Example G - revoke:

"Remove Tim's supplier bank access."

-> Access verifies requester authority.

-> revoke override/access update.

-> audit.

Example H - master template propagation:

"Manager template now gets inventory approval up to $2,000."

-> Selene asks future Managers, all current Managers, selected Managers, draft only, or new version review.

-> affected users and conflicting overrides are previewed.

-> approved scope is recompiled.

-> audit records every affected change.

Example I - one-shot profit/report access:

Tom is a Clerk.

Authorized manager allows Tom to view company profit one time.

-> Access creates a OneShot override for Tom only.

-> action/resource = view company profit report.

-> expiry = after one view or fixed time.

-> Clerk template remains unchanged.

Example J - rehire/reactivation:

Tom returns to ABC Wines.

-> historical access remains for audit.

-> current role and template are selected.

-> old overrides are reviewed, not restored automatically.

-> Access creates or reactivates per-user access from current approved template.

Example K - budget authority:

"Approve this $120,000 payment."

-> Access checks spend authority and multi-approver policy.

-> Finance/Budget checks budget truth and financial impact.

-> Authority, confirmation, and SimulationExecutor gates pass or deny.

-> Execution owner performs the action only if lawful.

## 28. What Must Not Happen

- no access granted from role name alone,
- no access granted from position alone,
- no access granted from onboarding without Access/Governance,
- no access granted from link activation,
- no access granted from Voice ID,
- no access granted from GPT-5.5/OpenAI,
- no Desktop/iPhone access decision,
- no Adapter access decision,
- no role string contains owner/admin as production authority,
- no sensitive fields exposed without field-level access,
- no company data retained after termination unless policy allows,
- no personal Selene access deleted merely because company relationship ended,
- no temporary access without expiry,
- no permanent access without approval where policy requires,
- no revoke/suspend/retire without audit,
- no protected execution from access check alone,
- no implementation from this document alone.

## What Must Not Happen Additions

- no master template change may silently update existing users without explicit scope confirmation,
- no per-user override may silently become a template change,
- no template change may erase user-specific overrides without policy and confirmation,
- no one-shot access may become permanent access,
- no temporary access may remain active without expiry,
- no terminated employee may retain company access unless explicit policy permits,
- no returning employee may receive old access without current validation,
- no personal Selene access may be deleted merely because company access ended,
- no leave/off-shift status may be ignored for high-risk actions where policy applies,
- no budget/spend approval may execute from Access alone,
- no budget overrun approval may bypass Finance/Budget and required approvers,
- no biometric/passcode requirement may be faked by Access without verified proof,
- no role/position name may grant spend authority without an approved template.

## 29. Required Upgrade List

1. Master role/template registry
2. Proper role hierarchy
3. Workspace scope
4. Company/entity access scope
5. Field-level permissions
6. Private data read gate
7. Protected execution gate integration
8. Access denial/explanation through PH1.WRITE
9. PH1.D/GPT-5.5 + PH1.N role/access candidate proposal
10. Approval/escalation workflow
11. Revocation/expiration/suspension journey
12. Termination/resignation/retirement access flow
13. Position-to-access co-authoring flow
14. Per-user access lifecycle assistant
15. JD live acceptance proof

Also required:

- OneShot / Temporary / Permanent / Revoke full-stack completion,
- leave/holiday/sick/off-work access posture,
- personal Selene access fallback after company access removal,
- promotion/demotion/position change access flow,
- access audit proof,
- workspace/resource/field migration plan,
- Adapter wrong-owner retirement ledger,
- master template propagation to existing per-user access,
- per-user access recompile and migration,
- step-up verification mapping,
- budget/spend/approval authority matrix,
- multi-approver/board approval support,
- access override conflict preservation,
- access rollback/revert proof.

## Required Implementation Slices

These are not optional future dreams.

These are the required implementation slices that must be built after Grand Architecture Reconciliation and repo-truth activation.

This docs-only task does not implement them, but the final build must include them.

1. Master Access Governance Activation Pack
2. Per-User Access Instance Lifecycle
3. Master Role / Access Template Registry
4. Role Hierarchy + Position Linkage
5. Permission Matrix
6. Field-Level Permission Model
7. Tenant / Workspace / Company Scope
8. Position-to-Access Co-Authoring Flow
9. Onboarding Per-User Access Creation Proof
10. Temporary / OneShot / Permanent / Revoke Overrides
11. Suspended / Restricted / Retired Access States
12. Leave / Holiday / Sick / Off-Work Access Posture
13. Termination / Resignation / Retirement Personal Fallback Flow
14. Promotion / Demotion / Position Change Flow
15. Private Data Read Gate
16. Protected Execution Access Gate
17. PH1.D + PH1.N Access Proposal Shell
18. PH1.X Access Route/Risk Validation
19. PH1.WRITE Access Explanation Boundary
20. Approval / Escalation / Board Vote Resolver
21. Access Audit Evidence Pack
22. Desktop/iPhone Render-Only Access Proof
23. Adapter Transport-Only Access Proof
24. JD Live Access Acceptance Pack
25. Master Template Change Propagation
26. Per-User Access Recompile / Migration
27. OneShot Access Override Proof
28. Temporary Access Override Proof
29. Permanent Access Change Proof
30. Revoke / Suspend / Retire Access Proof
31. Leave / Holiday / Sick / Off-Work Access Posture
32. Termination / Resignation / Retirement Flow
33. Personal Selene Fallback Flow
34. Rehire / Return-To-Work Reactivation Flow
35. Step-Up Verification Requirement Mapping
36. Budget / Spend / Approval Authority Matrix
37. Multi-Approver / Board Approval Resolver
38. Access Override Conflict Preservation
39. Access Rollback / Revert Proof
40. Access Audit For Bulk Template Updates

## 31. Grand Architecture Reconciliation Note

This document must later be reconciled into:

- Master Access repo-truth extraction,
- Identity + Access + Authority Spine,
- PH1.ONB Onboarding Journey,
- PH1.POSITION future extraction/design,
- PH1.LINK Link Journey,
- PH1.D Proposal Gateway,
- PH1.N Meaning Unravelling,
- PH1.X Request Decision Lattice,
- PH1.WRITE Human Presentation,
- PH1.M Human Memory,
- PH1.VOICE.ID Human Presence,
- PH1.REM Reminder Journey,
- PH1.BCAST / PH1.DELIVERY,
- Payroll/HR future owners,
- Scheduler/Roster future owners,
- Finance/Budget future owners,
- Tenant / Workspace Governance,
- Step-up Identity/Device/Human Presence proof,
- Access template propagation and recompile proof,
- Multi-approver/board approval resolver,
- Desktop/iPhone render-only proof,
- Adapter transport-only proof,
- Old Compatibility Path Retirement.

Grand Architecture Reconciliation must preserve the current repo-truth split: PH1.ACCESS owns access gate/schema decisions, PH2.ACCESS owns per-user access instance/override truth, and no other stack may silently become Access.

## Final Architecture Sentence Update

Master Access templates define the company's role, permission, field, spend, approval, and scope rules; Per-User Access compiles those rules into an individual user's live access instance; template changes require explicit propagation choices; overrides may be one-shot, temporary, permanent, revoked, or suspended; employment lifecycle events retire or reactivate company access while preserving personal Selene access where allowed; Position defines the job, Onboarding prepares the person, Access gives the keys, Finance/Budget owns money truth, and every access change may be understood through Selene's probabilistic intelligence but must be applied only through deterministic Access/Governance validation, simulation, confirmation, and audit.

## Commerce Stack 77-84 Authority Handoff

Access/Governance must provide authority checks for commerce pricing override, POS override, brand approval, B2B approval, dispatch hold release, address change after dispatch, high-value dispatch release, refund approval, outside-policy exception, seller/supplier enforcement, customer enforcement, provider payout release, and provider payout reversal.

GPT-5.5 may explain or draft the request, but deterministic Access/Governance owns permission truth, step-up requirements, approval route, token scope, expiry, and audit.
