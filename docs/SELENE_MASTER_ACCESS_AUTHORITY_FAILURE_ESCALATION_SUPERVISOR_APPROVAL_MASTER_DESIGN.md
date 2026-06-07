# Selene Master Access — Authority Failure Escalation + Supervisor Approval Master Design

DOCUMENT TYPE:
MASTER ACCESS DESIGN PATCH / AUTHORITY FAILURE ESCALATION LAW

STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

## 0. Authority And Scope

AGENTS.md controls execution.

This is a docs-only master design.

No runtime code was changed.

This document does not authorize implementation.

This document extends Master Access Governance with authority-failure escalation behavior.

Existing Master Access repo-truth extraction remains factual base.

Future implementation requires repo-truth activation, approved file scope, tests, backend evidence, Access/Governance proof, PH1.BCAST/DELIVERY proof, PH1.WRITE proof, simulation proof, audit proof, and JD live acceptance.

## 1. Executive Target

When a requester lacks authority, Selene should not immediately end the process unless policy says escalation is forbidden.

Selene should first check whether supervisor/approver escalation is allowed.

If escalation is allowed, the requester does not need to be told "I'm checking with your supervisor" as the default wording.

Selene may show a neutral "I'm checking whether this can proceed" only if the interaction requires a visible pending state.

Otherwise:

- escalation happens quietly,
- supervisor/approval authority receives the request through BCAST/DELIVERY,
- approved requests continue naturally,
- denied requests are explained politely to requester.

Core law:

Authority failure does not equal immediate refusal.
Authority failure creates an escalation opportunity where company policy allows.
Execution still fails closed until authority is approved.

## 2. Correct Owner Split

Access/Governance owns:

- authority failure detection
- escalation eligibility
- approver/supervisor resolution
- approval scope
- approval token
- one-time / temporary / permanent authority decision
- denial reason
- audit requirement

PH1.BCAST / PH1.DELIVERY owns:

- approval request delivery
- supervisor notification
- reminder/follow-up delivery if required
- delivery receipts

PH1.REM owns:

- reminder timing for pending approval if required

PH1.WRITE owns:

- supervisor approval wording
- requester denial wording
- requester approved-continuation wording if any
- neutral pending wording if required

PH1.D / GPT-5.5 may help:

- draft polite wording
- summarize context for supervisor
- explain denial in human language

PH1.D / GPT-5.5 must not:

- decide authority
- choose approver
- grant permission
- execute action
- bypass Access
- bypass BCAST/DELIVERY
- bypass PH1.WRITE

The original domain engine owns the original action:

- ONB owns onboarding session flow
- LINK owns link creation
- Payroll owns payroll
- Roster owns roster
- Access owns authority approval
- Delivery owns messages

## 3. Silent Escalation UX Law

Default behavior when requester lacks authority and escalation is allowed:

1. Do not immediately tell requester:
   "I'll check with your supervisor now."

2. Create escalation request internally.

3. Deliver supervisor approval request through PH1.BCAST / PH1.DELIVERY.

4. If supervisor approves quickly:
   Continue original flow naturally as though the authority gate passed through approved escalation.

5. If supervisor denies:
   Tell requester:
   "You don't currently have permission to complete this request. You can request this authority through the proper approval path and try again later."

6. If supervisor does not respond within policy time:
   Tell requester:
   "This request needs approval before it can continue. It has not been approved yet."

7. If policy requires transparency:
   Selene may use a neutral pending statement:
   "I'm checking whether this request can proceed."

Do not expose unnecessary internal authority details to requester unless policy allows.

Do not shame requester.

No production wording like "above your pay grade" even if JD is spiritually correct and hilarious.

## 4. Escalation Eligibility

Access must determine whether escalation is allowed.

Escalation may be allowed for:

- onboarding link creation
- sending invite/link
- role/access request
- temporary access request
- roster change request
- task reassignment request
- payroll draft preparation request
- document access request
- low/medium protected business action where policy allows approval

Escalation may be forbidden or stricter for:

- payroll commit
- bank/payment execution
- termination execution
- salary change
- high executive access
- confidential data export
- legal/HR notice sending
- emergency override
- access grant to highly privileged role

If escalation is forbidden:

- fail closed
- PH1.WRITE explains requester lacks permission
- audit denial

## 5. Approver / Supervisor Resolution

Access must resolve approver in order:

1. explicit company approval policy for action,
2. requester's direct supervisor,
3. department manager,
4. role owner,
5. HR/Payroll/Finance/Admin owner depending action,
6. tenant owner / company owner / board approval where required.

If no approver can be resolved:

- fail closed
- create audit event
- PH1.WRITE explains that approval route is missing

Do not let the original action engine guess the approver.

## 6. Approval Types

Supervisor/approver options must include:

- approve one time,
- approve temporary authority,
- approve permanent authority,
- deny,
- request more context,
- redirect to another approver.

Definitions:

One-time:
Authority applies only to this exact action and scope.

Temporary:
Authority applies for a bounded period, action class, tenant/company/workspace scope.

Permanent:
Authority updates requester's access template or per-user access instance through Access/Governance approval process. This must not be silently applied.

Deny:
Original action remains blocked.

Request more context:
Selene collects missing context from requester or relevant engine.

Redirect:
Approval request is transferred to another valid approver.

## 7. Required Logical Packets

These are future logical packets. Codex must later map these to repo equivalents. This document does not claim they currently exist.

AuthorityFailurePacket:

- request_id
- actor_user_id
- action
- target_entity
- tenant_id
- company_id
- workspace_id
- required_permission
- current_permission_status
- denial_reason_code
- escalation_allowed
- escalation_policy_ref
- audit_ref

AuthorityEscalationRequestPacket:

- escalation_request_id
- original_request_id
- requester_user_id
- approver_user_id
- action
- action_scope
- target_entity
- requested_outcome
- risk_level
- context_summary_ref
- delivery_required
- response_options
- expires_at
- audit_ref

AuthorityEscalationDeliveryPacket:

- delivery_id
- escalation_request_id
- approver_user_id
- channel
- message_ref
- delivery_status
- delivery_receipt_ref

AuthorityEscalationDecisionPacket:

- escalation_request_id
- approver_user_id
- decision
- decision_scope
- temporary_authority_expires_at
- permanent_authority_requested
- reason
- confirmation_ref
- audit_ref

AuthorityApprovalTokenPacket:

- approval_token_id
- escalation_request_id
- requester_user_id
- approver_user_id
- action
- action_scope
- token_type: one_time / temporary / permanent_pending_access_update
- expires_at
- consumed_at
- audit_ref

RequesterDenialExplanationPacket:

- original_request_id
- requester_user_id
- denial_reason_code
- user_safe_message_ref
- next_step_options
- audit_ref

## 8. BCAST / DELIVERY Protocol Requirement

All supervisor/approver messages must go through PH1.BCAST / PH1.DELIVERY.

Access must not send messages directly.

Required delivery behavior:

- create delivery request,
- use approved channel policy,
- use PH1.WRITE-approved message,
- capture delivery receipt where provider supports it,
- support reminder/follow-up via PH1.REM where policy allows,
- record delivery evidence.

BCAST/DELIVERY does not decide approval.

BCAST/DELIVERY does not own authority.

BCAST/DELIVERY only delivers the approval request and responses.

## 9. PH1.WRITE Wording Rules

PH1.WRITE owns final wording.

Supervisor message example:
"JD is requesting permission to send an employee onboarding link to Tom Richards for the Warehouse Supervisor role. JD does not currently have this authority. Do you want to approve this one time, give temporary authority, grant ongoing authority, deny it, or ask for more context?"

Requester denial example:
"You don't currently have permission to complete this request. You can request this authority through the proper approval path and try again later."

Pending timeout example:
"This request still needs approval before it can continue. It has not been approved yet."

Approved path:
If approval is received, Selene should continue the original flow naturally.
Do not overexplain internal authority mechanics unless policy/user asks.

## 10. Onboarding Example

User says:
"Selene, onboard Tom as Warehouse Supervisor and send him the link by SMS."

Flow:

PH1.D proposes onboarding invite intent.
PH1.N extracts Tom / Warehouse Supervisor / SMS.
PH1.X marks protected business setup.
Access checks JD authority.
JD lacks employee_onboarding.create.
Access checks escalation policy.
Escalation allowed.
Access resolves JD's supervisor.
PH1.WRITE drafts supervisor request.
PH1.BCAST / PH1.DELIVERY delivers approval request.
Supervisor approves one-time.
Access creates one-time approval token.
Original ONB/LINK flow resumes.
Selene asks sender for any missing fields and sends link after confirmation.

Requester is not told "I'm checking with your supervisor now" by default.

If supervisor denies:
Selene tells requester they do not currently have permission and can request authority through the proper path.

## 11. Simulation Rules

All future simulations must obey this section.

When a simulation includes an access failure:

- it must call Access/Governance escalation policy,
- it must not invent local escalation behavior,
- it must not send supervisor messages directly,
- it must route messages through PH1.BCAST / PH1.DELIVERY,
- it must use PH1.WRITE for wording,
- it must respect approval token scope,
- it must audit every authority failure and approval result.

Simulations are not allowed to create new access rules.

Simulations must consume the rules from:

- Master Access Governance,
- this Authority Failure Escalation design,
- BCAST/DELIVERY protocol,
- PH1.WRITE wording boundary,
- PH1.X request-risk validation.

## 12. Audit Requirements

Audit must record:

- original requester
- original action
- required permission
- authority failure
- escalation eligibility decision
- approver resolved
- delivery request created
- delivery result
- approver response
- approval token created if approved
- token consumed if used
- denial if denied
- original action resumed or blocked
- final outcome

No approval without audit.

No authority token without audit.

No silent permanent authority changes.

## 13. Failure Branches

A. No supervisor found:
fail closed, explain approval route missing.

B. Supervisor does not respond:
pending until timeout; requester receives safe pending/approval-needed message.

C. Supervisor denies:
request blocked; requester informed politely.

D. Supervisor approves one-time:
original action resumes for exact scope only.

E. Supervisor grants temporary:
Access creates bounded temporary authority.

F. Supervisor grants permanent:
Access creates permanent-authority request; it must follow Master Access template/per-user access update rules and cannot be silently granted unless policy explicitly allows and required approvals pass.

G. Delivery fails:
Access cannot assume approval. PH1.BCAST/DELIVERY reports delivery failure. PH1.REM or alternate delivery path may be used if policy allows.

H. Request context changes:
approval token invalid if scope no longer matches original request.

## 14. What Must Not Happen

- no point-blank refusal before escalation check where policy allows escalation
- no action execution without authority
- no supervisor message sent directly by Access
- no BCAST/DELIVERY approval decision
- no PH1.D/GPT-5.5 authority grant
- no local simulation-specific access rules
- no permanent authority silently granted
- no approval token reused outside scope
- no requester shaming in production wording
- no bypass of PH1.WRITE
- no bypass of PH1.X
- no bypass of audit
- no implementation from this document alone

## 15. Required Future Build Slices

1. Access Authority Failure Escalation Activation Pack
2. Escalation Eligibility Policy
3. Approver/Supervisor Resolver
4. AuthorityFailurePacket Contract
5. AuthorityEscalationRequestPacket Contract
6. BCAST/DELIVERY Approval Request Handoff
7. PH1.WRITE Approval/Denial Wording Boundary
8. One-Time Approval Token
9. Temporary Authority Token
10. Permanent Authority Request Flow
11. Denial/Timeout Failure Branch
12. Audit Evidence Pack
13. Simulation Compliance Test Pack
14. JD Live Authority Escalation Acceptance

## 16. Final Architecture Sentence

"Selene Master Access does not treat every authority failure as a blunt refusal: where company policy allows, Access/Governance quietly creates a governed supervisor/approver escalation, PH1.BCAST/DELIVERY delivers the request, PH1.WRITE controls the wording, the approver may grant one-time, temporary, or permanent-scoped authority, and the original action may resume only when the approved authority token, simulation, confirmation, and audit gates all pass."

## Commerce Stack 77-84 Authority Escalation Handoff

Commerce authority failures for pricing override, POS override, brand approval, B2B approval, dispatch hold release, address change after dispatch, high-value dispatch release, refund approval, outside-policy exception, seller/supplier enforcement, customer enforcement, provider payout release, or provider payout reversal must create scoped approval requests with owner, recipient, deadline, delivery method, confirmation, evidence, reminder, escalation, closure, and audit.
