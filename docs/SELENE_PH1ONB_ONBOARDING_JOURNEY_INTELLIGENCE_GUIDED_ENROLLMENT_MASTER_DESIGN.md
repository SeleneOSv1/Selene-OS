# Selene PH1.ONB — Onboarding Journey Intelligence + Guided Enrollment Master Design

DOCUMENT TYPE:
DEDICATED MASTER DESIGN / PH1.ONB HUMAN ONBOARDING JOURNEY ARCHITECTURE

TASK:
SELENE_PH1ONB_ONBOARDING_JOURNEY_INTELLIGENCE_GUIDED_ENROLLMENT_MASTER_DESIGN

BUILD CLASS:
ARCHITECTURE / ONBOARDING INTELLIGENCE / GUIDED ENROLLMENT / ROLE REQUIREMENTS / ACCESS HANDOFF / OPENAI-ASSISTED HUMAN SETUP STACK

STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

SOURCE FOUNDATION:
Selene PH1.ONB Onboarding + Enrollment — Repo-Truth Functionality Extraction Master Design

PURPOSE:
Define the future human onboarding journey layer around current PH1.ONB repo-truth mechanics so Selene can guide different users, employees, customers, suppliers, contractors, friends, workspace members, and enterprise roles through a simple, human, OpenAI-assisted onboarding experience while preserving deterministic requirements, missing-field tracking, access policy, role templates, voice/device/document consent, reminders, audit, and protected execution boundaries.

## 0. Authority And Scope

AGENTS.md controls execution.

This is a docs-only master design.

No runtime code was changed.

This document does not authorize implementation.

The PH1.ONB repo-truth extraction remains the factual base.

Future implementation requires repo-truth activation, approved file scope, tests, backend evidence, PH1.D provider-off/fake-provider proof, PH1.WRITE validation proof, Access/Governance proof, and JD live acceptance.

## 1. Current Repo-Truth Foundation

Current repo truth shows PH1.ONB is worth keeping.

PH1.ONB currently supports invited onboarding after PH1.LINK activation, deterministic onboarding sessions, missing required fields, terms, employee photo/sender verification gates, primary device proof, voice/wake/persona setup prerequisites, access instance creation, completion, link consumption, and requirement backfill progress.

The current owner split is correct:

PH1.LINK = invite/link lifecycle and activation
PH1.ONB = onboarding session and setup progression
PH1.VOICE.ID = voice enrollment evidence
PH1.W = wake setup evidence
PH1.PERSONA / PH1.EMO = Selene tone/persona setup evidence
Access/Governance = roles, permissions, templates, scope
Authority + Simulation = protected execution
PH1.BCAST / PH1.DELIVERY / PH1.REM = notifications, delivery, reminders
PH1.WRITE = user-facing onboarding language
PH1.D / GPT-5.5 = governed probabilistic assistance
Desktop/iPhone = render and submit bounded actions only
Adapter = transport only

The upgrade must not replace PH1.ONB.

The upgrade must make PH1.ONB human-guided, enterprise-flexible, and deeply connected to role/position/access requirements.

## 2. Executive Target

Selene onboarding must feel simple to the human while remaining exact inside the system.

The target future experience:

Sender creates onboarding link
→ Selene asks sender for required prefill details based on onboarding type and role
→ receiver opens link
→ PH1.LINK validates and activates
→ PH1.ONB starts onboarding
→ Selene introduces herself and explains the journey
→ Selene asks only what is missing
→ Selene never asks twice for completed information
→ Selene explains every step in human language
→ Selene uses GPT-5.5 for guidance, clarification, comfort, translation, and troubleshooting
→ deterministic gates validate fields, consent, voice/device/document proof, access, role, salary/start date, and required policy
→ reminders handle postponement
→ access is created only through Access/Governance templates
→ onboarding completes only when all requirements are satisfied

Product standard:

probabilistic communication
+ deterministic requirements
+ governed memory of completed steps
+ role-specific setup
+ access-template handoff
+ reminders and recovery
+ audit

## 3. Probabilistic + Deterministic Operating Model

Onboarding is both human and strict.

### 3.1 Probabilistic layer

Use PH1.D / GPT-5.5 through Provider Governance for:

explaining onboarding steps
answering "why do I need this?"
helping confused users
understanding messy replies
rewriting questions clearly
multilingual guidance
summarizing remaining steps
comforting frustrated users
troubleshooting link/device/voice problems
explaining role/access implications
writing sender notifications
creating human-friendly reminders

### 3.2 Deterministic layer

Use deterministic owners for:

activated link validity
onboarding session state
required fields
missing fields
terms acceptance
consent
identity/device/voice proof
photo/document proof refs
role/access template selection
salary/start date/department/country/region fields
tenant/workspace scope
permission checks
access instance creation
reminder scheduling
completion readiness
audit

The law:

GPT-5.5 helps humans understand.
Selene deterministic owners decide what is complete, allowed, and auditable.

## 4. Human Onboarding Journey Flow

Standard future flow:

Sender request
→ PH1.D / GPT-5.5 meaning proposal
→ PH1.N extracts onboarding candidates
→ PH1.X validates onboarding lane and risk
→ onboarding type resolved
→ position/role/access template resolved
→ required field schema selected
→ sender prefill collected
→ confirmation before link send
→ PH1.LINK creates invite link
→ PH1.BCAST / PH1.DELIVERY sends link
→ receiver opens link
→ PH1.LINK validates and activates
→ PH1.ONB starts session
→ Selene introduction
→ required fields collected
→ consent/terms/device/voice/document steps
→ Access/Governance creates scoped access
→ completion readiness checklist
→ onboarding complete
→ link consumed

This flow must handle interruption, postponement, recovery, wrong role, expired link, and incomplete sessions.

## 5. Onboarding Type Matrix

Selene must support different onboarding types with different field requirements.

Do not treat all onboarding like one generic form.

### 5.1 Casual / personal connection user

Use cases:

friend
family member
personal connection
basic Selene user

Likely required fields:

full name or display name
phone/email or contact method
preferred language
timezone
consent to terms
basic profile preference
optional voice enrollment
optional device setup

Usually not required:

salary
department
employee number
tax/payroll fields
company access template
HR documents

Risk:

low unless private memory/company access is requested

### 5.2 Customer onboarding

Likely required fields:

customer name
contact person
phone/email
company name if business customer
billing/shipping address where needed
preferred communication channel
customer segment/type
consent/opt-in
privacy notices
account manager if relevant

Possible additional fields:

customer portal access
payment terms
tax/VAT/GST details
support SLA
contract refs

Risk:

moderate; customer data and external communication rules apply

### 5.3 Supplier onboarding

Likely required fields:

supplier legal name
contact person
email/phone
country/region
business registration number
tax details
bank/payment details where required
supplier category
compliance documents
insurance/certifications if needed
contract refs
approval owner

Risk:

high if payment/bank/compliance data is involved

### 5.4 Contractor onboarding

Likely required fields:

name
contact
contractor type
start date
end date or contract term
department/project
manager
access requirements
documents/certifications
country/region
rate or payment terms where allowed

Risk:

moderate/high depending system access and payment terms

### 5.5 Employee onboarding

Likely required sender-prefill fields:

legal/preferred name if known
role/position
department
manager
employment type
start date
work location
country/region
salary or compensation package where permitted
payroll group
roster/schedule group
access template
workspace/tenant assignment
required devices
required training/compliance modules

Likely receiver-provided fields:

personal contact details
address if required
emergency contact where lawful
bank/payroll details where lawful
tax details where lawful
identity documents where required
consents
voice/device enrollment
language preference

Risk:

high; HR/payroll/access fields require strict governance

### 5.6 Executive / high-access onboarding

Examples:

CEO
CFO
Director
Admin
HR lead
Finance controller
System administrator

Additional requirements:

elevated access template
approval/escalation path
dual-control where required
stronger identity proof
stronger device proof
mandatory voice/device enrollment if policy requires
confidentiality terms
audit flags
restricted delivery mode

Risk:

very high

No silent correction of CEO/CFO/Admin roles.

### 5.7 Workspace / tenant member onboarding

Required fields depend on:

tenant
workspace
team
role
access level
member type
region
workspace policy

Workspace onboarding is currently not fully proven in repo truth and should be marked future design until owner discovery.

## Universal Onboarding Targets

Selene onboarding is universal.

PH1.ONB may coordinate onboarding journeys for different setup targets, including:

- human/person onboarding
- employee onboarding
- contractor onboarding
- customer onboarding
- supplier onboarding
- casual/personal user onboarding
- company/entity onboarding
- tenant onboarding
- workspace onboarding
- device onboarding
- voice enrollment onboarding
- access/role onboarding
- document/proof onboarding

PH1.ONB owns the journey state, missing-field progression, resume continuity, guidance handoff, and completion checklist.

PH1.ONB must not own every target's truth.

Correct owner split:

- company/entity truth belongs to Company/Tenant/Workspace/Governance owners
- human identity truth belongs to Identity / Access / Voice Identity owners
- access permission truth belongs to Access/Governance
- position requirements belong to PH1.POSITION or the canonical position/role owner
- payroll/salary truth belongs to Payroll/HR owners
- schedule/roster truth belongs to Scheduler/Roster owners
- voice evidence belongs to PH1.VOICE.ID
- device proof belongs to Device/Human Presence owner
- document/media proof belongs to Document/Artifact/Media owners

Rule:

Onboarding is universal.
Truth ownership is specialized.

Example company onboarding:

User says:
"Set up ABC Wines in Selene."

Selene may guide:

- company legal name
- trading name
- industry
- company size
- country/region
- tax/GST/VAT status
- main admin
- modules needed
- payroll/roster/inventory/sales requirements
- workspace/tenant setup
- initial roles and access templates

But PH1.ONB coordinates the journey only.
Company/Tenant/Workspace/Governance owners store and govern the company truth.

## 6. Dynamic Requirements Engine

Selene needs a dynamic requirements system.

Not one static onboarding form.

Requirements must be generated from:

onboarding type
invitee type
company/tenant
workspace
industry
company size
country/region
position/role
department
access template
payroll/salary requirements
schedule/roster requirements
compliance requirements
voice/device policy
sender prefill
receiver responses

The future dynamic schema should be owned by the correct canonical owners:

PH1.POSITION = position requirements / job role fields
Access/Governance = access templates / permission groups
PH1.ONB = onboarding session state and missing-field progression
PH1.M = allowed user preferences and continuity where needed
PH1.WRITE = human explanations

PH1.ONB should consume schemas and required-field lists.

PH1.ONB should not invent company policy.

## 7. Position Requirements And Database Setup

Selene must support position-based onboarding requirements.

Example:

Retail cashier
→ start date
→ store location
→ payroll group
→ roster group
→ POS access
→ basic training

Warehouse worker
→ start date
→ warehouse location
→ safety training
→ shift pattern
→ equipment certification
→ inventory access

Sales manager
→ start date
→ department
→ sales region
→ CRM access
→ commission plan
→ customer segment access

CFO
→ start date
→ executive approval
→ finance system access
→ payroll visibility rules
→ confidential document step-up
→ dual authorization requirements

Potential database model:

PositionRequirementSchema
PositionRequirementField
OnboardingRequirementSet
OnboardingRequiredField
AccessTemplateMapping
IndustryRequirementOverlay
CountryRegionRequirementOverlay
CompanySizeRequirementOverlay
WorkspaceRequirementOverlay

Each field should support:

field_id
field_name
field_type
required_for
source_owner
sender_prefill_allowed
receiver_input_allowed
sensitive_level
access_required_to_view
access_required_to_edit
validation_rule
confirmation_required
can_be_reused_on_re-onboarding
expires_or_stale_after

Selene must ask only for fields that are missing, relevant, and allowed for that user/session.

## Dynamic Field Extension + One-Time Requirement Override

Selene must allow onboarding requirements to evolve safely.

Users may ask Selene to add new fields, for example:

"Add superannuation member number to employee onboarding."
"Ask Tom for his health number."
"All Australian employees need a superannuation fund and member number."
"Warehouse supervisors must provide forklift licence number."
"This company requires emergency contact for all staff."

Selene must not automatically mutate schema or database permanently from a casual sentence.

Selene must first clarify the scope:

- one-time field for this onboarding session only
- employee-type field
- position-level permanent field
- company/tenant-level permanent field
- workspace-level permanent field
- country/region-level field
- industry-level field
- global/default field

Example clarification:

"Should I add superannuation member number just for Tom, for all employees at this company, for Australian employees, or permanently to the employee onboarding schema?"

Field extension types:

1. One-time onboarding field
Used for one person/session only.
Does not alter global schema.

2. Position requirement field
Applies to a role/position.
Example:
Warehouse Supervisor requires forklift licence number.

3. Company/tenant requirement field
Applies to one company or tenant.
Example:
ABC Wines requires superannuation member number for every employee.

4. Country/region requirement field
Applies by jurisdiction.
Example:
Australia employee onboarding requires superannuation fund and member number.

5. Industry requirement overlay
Applies by industry.
Example:
Food manufacturing requires safety certification.

6. Global default field
Applies broadly across an onboarding type.
Requires stronger governance.

Dynamic field governance must include:

- requester identity
- requested field name
- field description
- field type
- field scope
- onboarding type
- position/role if applicable
- company/tenant/workspace if applicable
- country/region if applicable
- industry if applicable
- sensitivity level
- who can view
- who can edit
- whether sender can prefill
- whether receiver must provide
- validation rule
- retention policy
- confirmation requirement
- audit refs
- approval owner
- schema version impact

Suggested logical contracts:

- OnboardingFieldExtensionRequestPacket
- OnboardingFieldScopeDecisionPacket
- OnboardingOneTimeFieldPacket
- PositionRequirementFieldProposalPacket
- TenantRequirementOverlayPacket
- CountryRegionRequirementOverlayPacket
- IndustryRequirementOverlayPacket
- GlobalOnboardingRequirementProposalPacket
- OnboardingFieldSchemaVersionPacket
- OnboardingFieldApprovalPacket

Rules:

- one-time fields may be session-scoped after confirmation and permission
- permanent fields require schema governance
- sensitive fields require access/privacy classification
- payroll/health/legal/identity fields require stronger controls
- field changes must be schema-versioned
- existing onboarding sessions must not be silently broken
- completed valid fields must not be asked again
- stale/expired/invalid fields may be re-requested with explanation
- PH1.WRITE explains the decision clearly
- PH1.D/GPT-5.5 may help interpret the user's field request
- Access/Governance decides whether the requester can change requirements
- PH1.ONB consumes the resulting requirement schema

Example:

User:
"Add superannuation member number to employee list permanently."

Selene should resolve:

- field = superannuation member number
- onboarding type = employee
- likely country = Australia if context supports it, otherwise clarify
- permanence = permanent
- scope = needs clarification
- sensitivity = payroll/benefits sensitive
- governance = approval required

Selene asks:

"Should this apply to all employees, only Australian employees, only this company, or only this position?"

Only after confirmation and authority can the schema-change request proceed.

## 8. Sender Prefill Before Link Generation

Many onboarding fields should be collected from the original sender before the link is sent.

Example sender request:

"Onboard Tom as warehouse supervisor."

Selene should resolve and ask:

What is Tom's start date?
Which warehouse?
Who is his manager?
Is he full-time or casual?
Which roster group should he join?
What access template should he receive?
Should I send this by SMS?

For employee onboarding, likely sender-prefill fields include:

role/position
employment type
department
manager
start date
location
salary/pay package where allowed
payroll group
schedule/roster group
workspace/tenant
access template
required equipment
required training

Selene must confirm before sending:

Just confirming: do you want me to send Tom an employee onboarding link for Warehouse Supervisor, starting 3 June, assigned to the North Warehouse roster group, by SMS?

This should also be reflected in PH1.LINK Journey Intelligence later.

## 9. Receiver Field Collection

The receiver should only be asked for fields they need to provide.

Examples:

preferred name
contact method
language preference
consent/terms
identity verification
voice enrollment
device confirmation
bank/payroll/tax details where lawful
emergency contact where lawful
personal address where lawful

Selene should explain each field in plain language:

I need your start-date confirmation because it controls your first roster and payroll setup.

If the user is confused, Selene uses PH1.D/GPT-5.5 + PH1.WRITE to explain.

If the user gives messy replies, PH1.N extracts candidates.

If the reply is uncertain for a protected/sensitive field, Selene clarifies.

## 10. Selene Introduction And Step-By-Step Guidance

At onboarding start, Selene should introduce herself and the journey.

Example:

Hi, I'm Selene. I'll help you finish onboarding step by step. I'll only ask for what is needed, and I won't ask again for anything you've already completed.

For each step, Selene should explain:

what this step is
why it matters
what the user needs to do
whether it is optional or required
what happens next

Examples:

Wake/Desktop:

On Desktop, wake setup helps Selene know when you want to talk to her. If you also use iPhone, the side button setup may be used for explicit activation.

Voice:

Voice enrollment helps Selene recognize your voice as evidence during future sessions. It does not give you authority by itself.

Device:

Device confirmation links this onboarding session to the device you're using, so someone else cannot finish setup from a random device.

Access:

Access setup controls which parts of Selene you can use. It follows your company's role and permission rules.

## 11. Re-Onboarding And Resume Memory

Selene must never forget completed onboarding steps.

If onboarding is postponed, interrupted, or resumed, Selene must know:

completed fields
remaining fields
skipped fields
blocked fields
pending verification
expired proof
stale proof
last successful step
next required step

Rule:

Never ask again for a completed valid field.
Only ask for missing, stale, invalid, or policy-required updates.

Example:

Welcome back. You already accepted terms and confirmed this device. We still need voice enrollment and your payroll details.

This continuity belongs primarily to PH1.ONB session state, with PH1.M used only where durable user memory/preference law allows.

## 12. Postpone / Reminder Flow

Users may be busy.

If receiver says:

not now
later
tonight
tomorrow
remind me after work

Selene must use PH1.REM / PH1.BCAST / PH1.DELIVERY where applicable.

Flow:

user postpones onboarding
→ PH1.N interprets time if messy
→ PH1.X validates reminder route
→ PH1.REM schedules reminder
→ PH1.BCAST / PH1.DELIVERY delivers follow-up if needed
→ PH1.ONB resumes exact remaining step

Ambiguous time requires clarification:

Do you mean tomorrow morning around 9 AM?

Selene must not lose progress.

## 13. Expired Link Notification To Sender

If a receiver opens an expired invite link, Selene should help both sides.

Future flow:

receiver opens expired link
→ PH1.LINK marks/returns expired
→ receiver gets safe explanation
→ original sender is notified where policy allows
→ sender can resend a new link or cancel

Sender options:

resend new link
cancel onboarding
change role/details before resending
choose delivery method

PH1.LINK owns link expiry/recovery.

PH1.BCAST / PH1.DELIVERY owns notification.

PH1.WRITE explains.

PH1.ONB must not start from expired link.

## 14. Wrong Link / Wrong Role / Wrong Recipient Correction

Selene must support correction journeys.

Example:

"I sent Tom CFO but meant CEO."

Selene must inspect:

link status
activation status
onboarding status
access status
role/access template status

Rules:

not activated → revoke/reissue correct link if authorized
activated but incomplete → PH1.ONB + Access decide correction path
completed/access granted → Access/Governance role-change simulation required
wrong recipient → revoke/reissue and audit
wrong salary/start date/department → correction depends on current state and authority

No silent CEO/CFO correction.

No hidden access changes.

## 15. Access / Role / Permission Handoff

Onboarding must create the correct per-user access only through Access/Governance.

Future flow:

position/role resolved
→ access template selected
→ sender confirms
→ onboarding prerequisites complete
→ Access/Governance validates template and scope
→ per-user access instance created
→ audit

PH1.ONB may coordinate readiness.

PH1.ONB must not define access policy.

Access must consider:

tenant
workspace
role
department
position
country/region
employment type
company size/industry
manager approval
executive approval
high-risk systems

Examples:

Cashier → POS access only
Warehouse worker → inventory access + safety docs
HR manager → HR records access + confidentiality controls
CFO → finance/payroll visibility + elevated audit

## 16. Salary / Payroll / Schedule / Roster Integration

Employee onboarding often requires HR and operational setup.

Fields may include:

salary or pay rate
currency
pay frequency
employment type
start date
probation period
department
manager
country/region
work location
roster group
shift pattern
schedule availability
payroll group
commission plan
benefits eligibility

These fields are sensitive.

Rules:

PH1.ONB collects or coordinates fields
Payroll/HR/Scheduler/Roster owners own their systems
Access/Governance controls who can provide/edit/view fields
PH1.WRITE explains each field
PH1.REM can schedule missing-field follow-up

Future Scheduler/Roster engine integration:

Onboarding may pass approved start date, location, role, roster group, and availability to Scheduler/Roster only through its canonical owner and simulation.

No roster/schedule mutation from PH1.ONB alone.

## 17. Voice / Device / Document / Consent Handoff

Voice enrollment

Follow the Voice Identity + Human Presence architecture.

Rules:

voice enrollment is evidence only
voice does not grant authority
consent is required where policy says so
voice receipt may be required before completion
Selene explains what voice enrollment does and does not do

Device enrollment

Device setup may include:

primary device confirmation
platform setup receipt
wake setup
side button / explicit activation setup for iPhone where applicable
desktop wake setup where applicable

Document / photo proof

Photo/document evidence may be required for employee, contractor, supplier, executive, or compliance onboarding.

Rules:

raw artifact owner must be explicit
PH1.ONB stores refs, not raw proof ownership unless repo truth proves it
sensitive documents require access/privacy controls
PH1.WRITE explains upload/view requirements

Consent model

Consent should cover:

terms
privacy
voice enrollment
device binding
communications
document/photo proof
marketing/customer/supplier communication where relevant

## 18. Onboarding Status Assistant

Selene must be able to answer:

What step am I on?
What do I still need to do?
Why is onboarding blocked?
Did Tom finish onboarding?
What is missing for Sarah?
Can I resend the invite?
Can I change the role?
What failed?
What happens next?

Owner map:

PH1.ONB = onboarding session status
PH1.LINK = link status
PH1.BCAST / PH1.DELIVERY = notification/delivery status
PH1.REM = reminder status
Access/Governance = access status
Voice ID = voice enrollment status
PH1.WRITE = explanation

Status answers must be access-scoped.

## 19. Onboarding Troubleshooting Assistant

Selene should troubleshoot:

expired link
invalid link
wrong device
wrong tenant
missing fields
terms declined
voice enrollment failed
wake setup blocked
device confirmation failed
photo/document upload failed
sender verification rejected
access instance failed
wrong role
wrong recipient
onboarding postponed
reminder not received
client/adapter route issue

Selene should answer in human language:

The link expired before Tom completed onboarding. I can ask the sender to resend a new link or cancel this onboarding.

But all corrections must route to canonical owners.

## 20. Onboarding Completion Readiness Checklist

Before completion, Selene must verify readiness.

Checklist may include:

link activated
session active
tenant/workspace valid
required fields complete
terms accepted
sender prefill confirmed
receiver fields complete
primary device confirmed
voice enrollment locked if required
wake setup receipt if required
persona/tone setup locked if required
doc/photo proof complete if required
access template selected
access instance created
salary/start date/role/department fields complete if required
reminders/follow-up cleared or scheduled
all audit refs present

If something is missing, Selene should explain the exact missing item and next step.

## 21. Required Logical Packets

Future logical packets to map later:

OnboardingJourneyRequestPacket
OnboardingTypeResolutionPacket
OnboardingRequirementSchemaPacket
OnboardingRequiredFieldPacket
SenderPrefillPacket
ReceiverFieldSubmissionPacket
OnboardingGuidanceProposalPacket
OnboardingStatusSummaryPacket
OnboardingTroubleshootingPacket
OnboardingCorrectionPacket
OnboardingReminderHandoffPacket
OnboardingCompletionReadinessPacket
OnboardingAccessTemplateHandoffPacket
OnboardingConsentPacket
OnboardingDocumentProofPacket
OnboardingVoiceDeviceHandoffPacket
ExpiredLinkSenderNotificationPacket
OnboardingFieldExtensionRequestPacket
OnboardingFieldScopeDecisionPacket
OnboardingOneTimeFieldPacket
PositionRequirementFieldProposalPacket
TenantRequirementOverlayPacket
CountryRegionRequirementOverlayPacket
IndustryRequirementOverlayPacket
GlobalOnboardingRequirementProposalPacket
OnboardingFieldSchemaVersionPacket
OnboardingFieldApprovalPacket

Codex must later map these to repo equivalents.

Do not claim they currently exist.

## 22. Example End-To-End Flows

Example A — casual user

Sender: Send my friend Tom a Selene invite.
Selene: asks for phone/SMS confirmation.
PH1.LINK creates link.
PH1.DELIVERY sends SMS.
Tom opens link.
Selene asks only basic profile/terms/device/optional voice setup.

Example B — employee

Sender: Onboard Tim as warehouse supervisor.
Selene: asks sender for start date, location, manager, employment type, roster group, access template.
Sender confirms.
Link is sent.
Tim opens link.
Selene introduces onboarding, asks receiver fields, explains voice/device/doc steps, and completes only after all gates pass.

Example C — executive

Sender: Onboard Tom as CFO.
Selene: treats role as high access.
Requires access template, salary/payroll confidentiality, approval if policy says so, serious confirmation, stronger proof, and audit.

Example D — expired link

Receiver opens expired link.
Selene explains safely.
Original sender is notified.
Sender chooses resend, update details, or cancel.

Example E — postponed onboarding

Receiver: Not now, remind me tomorrow.
Selene clarifies time if needed.
PH1.REM schedules follow-up.
On resume, Selene continues from the exact remaining step.

Example F — wrong role

Sender: I sent CFO but meant CEO.
Selene checks lifecycle state.
If not activated, revoke/reissue.
If completed, route to Access/Governance role-change simulation.

Example G — one-time field

Sender: Ask Tom for his forklift licence number just this time.
Selene resolves a one-time onboarding field, confirms scope, validates requester permission, classifies sensitivity, and adds it only to Tom's onboarding session.
No permanent schema is changed.

Example H — permanent requirement request

Sender: Warehouse supervisors must provide forklift licence number.
Selene resolves a position-level requirement proposal, confirms scope, routes to PH1.POSITION and governance approval, and does not mutate active onboarding schemas until approved and versioned.

## 23. What Must Not Happen

Codex must not allow:

no onboarding from invalid/non-activated link
no static one-size-fits-all onboarding form
no asking again for completed valid fields
no access grant by PH1.ONB policy guessing
no role/template selection from fuzzy language without confirmation
no salary/payroll/HR fields exposed without access
no workspace onboarding invented without owner proof
no voice enrollment treated as authority
no Desktop/iPhone onboarding authority
no Adapter onboarding authority
no PH1.ONB sending reminders/messages directly
no PH1.ONB owning PH1.WRITE guidance
no hardcoded robotic onboarding UX as final experience
no permanent field/database change from casual user wording alone
no schema mutation without scope confirmation
no sensitive field collection without access/privacy classification
no health/payroll/legal field exposed without governance
no global onboarding field added when user intended one-time field
no one-time field silently promoted to permanent
no existing onboarding session broken by new field schema
no PH1.ONB owning company/legal/payroll truth
no PH1.ONB owning Position truth
no PH1.ONB owning Access policy
no implementation from this document alone

## 24. Recommended Future Build Slices

PH1.ONB Journey Intelligence Activation Pack

Universal Onboarding Target Model

Onboarding Type Matrix + Requirement Schema Map

Dynamic Field Extension Governance

One-Time Field Override Flow

Position Requirement Field Proposal Flow

Tenant/Company Requirement Overlay

Country/Region/Industry Requirement Overlay

Schema Versioning + Migration Plan

Field Sensitivity / Access / Retention Classification

Requirement Change Approval Flow

Position Requirements / Database Schema Integration

Sender Prefill Before Link Generation

PH1.WRITE Onboarding Guidance Boundary

PH1.D / GPT-5.5 Onboarding Help Proposal Shell

PH1.N Messy Reply Understanding

PH1.X Onboarding Request / Risk Routing

Access Template / Role Permission Handoff

Workspace / Tenant Scope Integration

Voice / Device / Consent Guided Setup

Document / Photo Proof Admission Boundary

Salary / Start Date / Department / Region Field Handling

Scheduler / Roster Future Handoff Boundary

Onboarding Status Assistant

Onboarding Troubleshooting Assistant

Wrong Role / Wrong Link / Wrong Recipient Correction

Expired Link Sender Notification

Reminder / Follow-Up Handoff Through PH1.REM / PH1.BCAST / PH1.DELIVERY

Completion Readiness Checklist

Onboarding Audit Evidence Pack

Desktop/iPhone Render-Only Proof

Adapter Transport-Only Proof

JD Live Onboarding Acceptance Pack

## 25. Grand Architecture Reconciliation Note

This document must later be reconciled into:

PH1.ONB extraction
PH1.LINK extraction and Link Journey Intelligence
PH1.D Proposal Gateway
PH1.N Meaning Unravelling
PH1.X Request Decision Lattice
PH1.WRITE Human Presentation
PH1.M Human Memory
PH1.VOICE.ID Human Presence
PH1.W Wake
PH1.BCAST / PH1.DELIVERY / PH1.REM
Access/Governance
Master Access Template / Role / Permission stack
PH1.POSITION / Position Requirements
Tenant / Workspace Governance
Scheduler / Roster / Workload future stacks
Document / Artifact / Media proof stacks
Universal onboarding target model
Dynamic requirement schemas
Field extension governance
Company/Tenant/Workspace entity setup
Payroll/HR field ownership
Country/region/industry overlays
Schema versioning and migration
Access/privacy classification for fields
Desktop/iPhone render-only proof
Adapter transport-only proof
Old Compatibility Path Retirement

Grand Reconciliation must preserve current repo-truth ownership and must not create duplicate onboarding, access, role, voice, reminder, writing, or routing brains.

## 26. Final Architecture Sentence

PH1.ONB provides Selene's governed onboarding-session mechanics; Onboarding Journey Intelligence turns onboarding into a human-guided, OpenAI-assisted, role-aware, requirement-driven, memory-continuous, reminder-backed, access-governed enrollment experience where Selene explains every step, asks only what is missing, remembers what is complete, and completes onboarding only when the correct deterministic owners prove every required gate.
