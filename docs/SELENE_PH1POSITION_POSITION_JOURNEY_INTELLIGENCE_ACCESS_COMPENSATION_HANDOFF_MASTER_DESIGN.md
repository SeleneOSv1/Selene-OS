# Selene PH1.POSITION — Position Journey Intelligence + Access & Compensation Handoff Master Design

DOCUMENT STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

## 0. Authority and Scope

AGENTS.md controls execution.

This is docs-only master design.

No runtime code was changed.

This document does not authorize implementation.

The PH1.POSITION repo-truth extraction remains the factual base:

- `docs/SELENE_PH1POSITION_POSITION_ENGINE_REPO_TRUTH_FUNCTIONALITY_EXTRACTION_MASTER_DESIGN.md`

This document defines future Position Journey Intelligence architecture pending Grand Architecture Reconciliation.

Future implementation requires repo-truth activation, approved file scope, tests, backend evidence, PH1.D provider-off/fake-provider proof, PH1.WRITE validation proof, Access/Governance proof, Payroll/HR/Finance/Scheduler owner proof where applicable, audit proof, and JD live acceptance.

## 1. Executive Target

PH1.POSITION is Selene's governed job/position truth boundary.

Position Journey Intelligence is the human-guided layer around it.

The target is:

- standard position library for common industries;
- guided voice/type position creation;
- dynamic position requirements;
- position hierarchy and reporting line;
- contractor vs employee distinction;
- access-template co-authoring;
- onboarding requirement handoff;
- compensation/payroll/finance handoff;
- scheduler/roster handoff;
- job description drafting;
- market salary benchmark handoff;
- cost-to-company preview handoff;
- contractor billing handoff;
- batch position/opening creation;
- conditional offer handoff;
- PH1.D/GPT-5.5/PH1.N/PH1.X/PH1.WRITE human guidance;
- deterministic validation, confirmation, simulation, and audit.

Product law:

Selene may help design the position probabilistically.

Selene must activate, update, retire, and connect positions deterministically.

## 2. Current Repo-Truth Foundation

The PH1.POSITION extraction establishes:

- PH1.POSITION already supports tenant/company-scoped position lifecycle.
- PH1.POSITION supports Draft, Active, Suspended, Retired.
- PH1.POSITION supports versioned position requirement schemas.
- PH1.POSITION can feed ONB required fields and gates.
- PH1.POSITION can signal requirement backfill through CurrentAndNew.
- PH1.POSITION has permission_profile_ref and compensation_band_ref references.
- PH1.POSITION has schedule type reference.
- PH1.POSITION is worth keeping.
- PH1.POSITION is not a complete global HR/position system yet.

Current weaknesses:

- no full position hierarchy/reporting line;
- no department hierarchy;
- no standard position library;
- no contractor-specific full setup;
- no workspace scope;
- no full position-to-access template co-authoring;
- no full position-to-payroll/HR integration;
- no full position-to-finance/budget integration;
- no scheduler/roster integration beyond schedule type;
- no full job description creation flow;
- no salary benchmark or cost-to-company handoff;
- no conditional offer flow;
- no batch opening/headcount flow;
- no PH1.WRITE position wording boundary;
- no PH1.D/GPT-5.5 position proposal path;
- no JD live position journey proof.

This document upgrades the human journey around that foundation without replacing the repo-truth owner.

## 3. Master Owner Split

PH1.POSITION owns:

- position title;
- job family;
- department;
- reporting line;
- position hierarchy references;
- position lifecycle;
- position requirement schemas;
- position-required fields;
- position-linked setup context;
- position-to-onboarding requirement handoff;
- references to access template, compensation band, schedule type, and other owner refs.

Access/Governance owns:

- permissions;
- access templates;
- field access;
- spend/approval limits;
- protected action access;
- per-user access instances.

PH1.ONB owns:

- human onboarding session;
- missing-field collection;
- receiver/sender setup flow;
- onboarding readiness;
- access handoff after requirements pass.

Payroll/HR owns:

- salary truth;
- pay rate;
- payroll rules;
- employment status;
- benefits;
- tax/payroll identifiers;
- termination/resignation/retirement truth.

Compensation Engine owns:

- salary package logic;
- allowances;
- commissions;
- bonuses;
- overtime;
- compensation benchmarks;
- compensation recommendation logic.

Finance/Budget owns:

- budget;
- profitability;
- cost-to-company impact;
- margin/profit thresholds;
- spend approval financial truth.

Scheduler/Roster owns:

- shifts;
- roster groups;
- availability;
- workload;
- leave/off-shift truth.

Contract/Supplier/AP owners own:

- contractor agreements;
- invoices;
- contractor payment method;
- contractor billing status.

PH1.REM owns:

- reminders for contract expiry, position review, onboarding follow-up, approval follow-up.

PH1.BCAST / PH1.DELIVERY owns:

- notifications and outbound communication.

PH1.WRITE owns:

- final user-facing position guidance;
- job descriptions;
- explanations;
- confirmations;
- denials.

PH1.D / GPT-5.5 proposes:

- position descriptions;
- likely requirements;
- likely access suggestions;
- compensation explanation drafts;
- job description drafts.

Required law:

Position creates the job card.

Access gives the keyring.

Onboarding sets up the person.

Payroll pays.

Finance checks budget.

Scheduler/Roster manages time.

Selene makes it feel like one conversation.

## 4. Standard Position Library + Industry Packs

Selene should include substantial default position libraries for common businesses and industries so companies are not starting from a blank screen.

Starter universal roles:

- Owner
- Chairman
- CEO
- Director
- General Manager
- Operations Manager
- Office Manager
- Administrator
- Assistant
- Department Manager
- Team Leader
- Employee
- Trainee
- Apprentice
- Intern
- Casual Worker
- Contractor

Finance / Payroll roles:

- CFO
- Finance Manager
- Accountant
- Bookkeeper
- Payroll Officer
- Accounts Payable Officer
- Accounts Receivable Officer
- Financial Controller
- Auditor
- External Accountant

HR / Compliance roles:

- HR Manager
- HR Officer
- Recruiter
- Training Coordinator
- Compliance Officer
- Safety Officer
- Workplace Health and Safety Manager

Retail roles:

- Store Manager
- Assistant Store Manager
- Cashier
- Sales Assistant
- Stock Controller
- Retail Supervisor
- Customer Service Representative

Warehouse / Logistics roles:

- Warehouse Manager
- Warehouse Supervisor
- Warehouse Worker
- Picker/Packer
- Forklift Driver
- Delivery Driver
- Inventory Controller
- Dispatch Coordinator
- Logistics Manager
- Fleet Coordinator

Hospitality roles:

- Restaurant Manager
- Chef
- Sous Chef
- Kitchen Hand
- Waiter
- Barista
- Bartender
- Cleaner
- Housekeeper
- Front Desk
- Event Coordinator

Sales / Marketing roles:

- Sales Manager
- Sales Representative
- Account Manager
- Business Development Manager
- Marketing Manager
- Campaign Coordinator
- Social Media Manager
- Customer Success Manager

IT / Systems roles:

- System Administrator
- IT Support Technician
- Security Administrator
- Data Analyst
- Developer
- Product Manager
- Technical Lead

Customer / Supplier-facing roles:

- Customer Support Agent
- Customer Account Manager
- Supplier Manager
- Procurement Officer
- Purchasing Manager

Contractor roles:

- External Contractor
- Consultant
- Trade Contractor
- Freelance Designer
- Agency Staff
- Temporary Worker
- Project Contractor
- Hourly Contractor
- Lump-Sum Contractor
- Site Contractor

Rules:

- starter roles are templates, not prison walls;
- authorized users can add more positions;
- authorized users can customize templates;
- authorized users can retire or supersede positions;
- country/industry/company-size/workspace overlays can modify required fields;
- role names must not grant access by themselves;
- access templates must remain Access/Governance-owned.

## 5. Position Type Matrix

| Position Type | Likely Owner Dependencies | Onboarding Requirements | Access Risk | Payroll / Finance / Scheduler Relevance | Proof / Document Requirements | Contract / Payment Difference |
| --- | --- | --- | --- | --- | --- | --- |
| Employee position | PH1.POSITION, PH1.ONB, Access, Payroll/HR, Scheduler/Roster | Identity, terms, employee fields, position requirements | Varies by role | Payroll, roster, benefits, leave | Identity, payroll, compliance docs where lawful | Employment relationship. |
| Casual employee position | PH1.POSITION, Payroll/HR, Scheduler/Roster | Availability, roster group, pay setup | Moderate | Variable shifts, casual rates, payroll | Identity, tax/payroll docs | Casual employment rules. |
| Contractor position | PH1.POSITION, Contract/Supplier/AP, Finance/AP, Access | Contractor details, contract refs, compliance docs | Moderate to high for system access | Invoices, billing, project budgets | Insurance, certifications, contract proof | Contract/payment mode differs from payroll. |
| Executive position | PH1.POSITION, Access, Authority, Finance/Budget, HR | Strong identity, approvals, confidentiality | Very high | Salary, board approval, finance visibility | Executive approvals, confidentiality proof | High-access employment. |
| Manager position | PH1.POSITION, Access, Scheduler/Roster | Reporting line, staff scope, approvals | High | Rosters, budgets, approval limits | Training/certification where needed | Employment or contractor depending type. |
| Warehouse position | PH1.POSITION, Scheduler/Roster, Access | Location, safety, equipment, certifications | Moderate | Shift pattern, roster group, inventory impact | Safety/forklift proof where required | Employment/contractor based on setup. |
| Retail position | PH1.POSITION, Scheduler/Roster, Access | Store, roster, POS training | Moderate | Roster, store payroll, sales operations | Identity, training proof | Usually employee/casual. |
| Finance position | PH1.POSITION, Access, Finance/Budget, Payroll/HR | Finance controls, confidentiality | Very high | Finance/payroll visibility | Confidentiality, step-up where required | Employment/contractor based on setup. |
| HR position | PH1.POSITION, Access, Payroll/HR | HR policy, employee data access | Very high | HR/payroll records | Confidentiality, HR training | Employment/contractor based on setup. |
| Admin/system position | PH1.POSITION, Access, Device/Human Presence | System scope and step-up requirements | Very high | Depends on permissions | Device proof, admin approval | Usually employment or managed contractor. |
| Customer-facing role | PH1.POSITION, Access, BCAST/DELIVERY, Customer owner | Customer data rules, communication consent | Moderate to high | Sales/support metrics | Training, communication policies | Employment/contractor based on setup. |
| Supplier-facing role | PH1.POSITION, Access, Supplier/AP owner | Supplier data rules, procurement scope | High | AP/procurement impact | Procurement training, vendor controls | Employment/contractor based on setup. |
| Workspace role | PH1.POSITION, Workspace/Governance, Access | Workspace-specific fields | Depends on workspace | Workspace schedules/budgets | Workspace policy proof | Scoped to workspace. |
| Department role | PH1.POSITION, Department/Governance, Access | Department, reporting line | Depends on department | Department budgets/rosters | Department-specific proofs | Scoped to department. |
| Company/tenant role | PH1.POSITION, Tenant/Governance, Access | Tenant/company fields | High for company-wide access | Company-wide budgets/payroll | Tenant approvals | Scoped to tenant/company. |
| Temporary/casual role | PH1.POSITION, Payroll/HR, Scheduler/Roster | Term, availability, temporary access | Moderate | Temporary payroll/roster | Identity, contract/terms | Time-bounded relationship. |
| Trainee/apprentice role | PH1.POSITION, Payroll/HR, Training owner | Training plan, supervisor, restrictions | Moderate | Training pay/roster | Training docs | Employment/apprenticeship rules. |
| Custom position | PH1.POSITION plus selected owners | Dynamically defined | Must be reviewed | Depends on chosen fields | Depends on requirements | Depends on employee/contractor type. |
| Retired/superseded position | PH1.POSITION, Audit, Access, ONB | No future onboarding unless policy allows | Historical only | Historical impact | Audit retained | Historical relationship retained. |

## 6. Guided Position Creation Flow

Selene must support natural-language and guided position creation.

Example:

User says:

`Create a Logistics Manager for the Shenzhen warehouse, working 9 AM to 5 PM Monday to Friday.`

Selene extracts:

- title = Logistics Manager;
- location = Shenzhen warehouse;
- schedule = Monday-Friday 9-5;
- department = logistics;
- schedule type = full-time;
- breaks = ask or reference local/company policy;
- country/region = China/Shenzhen if context supports, otherwise clarify.

If input is partial:

`Create a position.`

Selene asks:

- What is the title of this role?
- Which department/team is it for?
- Is it full-time, part-time, casual, contract, or shift-based?
- Where will it be based?
- Who does this role report to?
- Does this role manage staff?
- Does this role need certifications, licences, tools, languages, or equipment?
- Should I suggest a standard access template?
- Should I show estimated compensation/cost impact?

Required future flow:

User request
-> PH1.D/GPT-5.5 proposes position meaning
-> PH1.N extracts candidates
-> PH1.X validates position journey and risk
-> PH1.WRITE asks clarifications
-> PH1.POSITION creates draft
-> Access/Payroll/Finance/Scheduler owners are consulted only through correct handoffs
-> authorized user confirms
-> position activates only through deterministic PH1.POSITION and required gates.

The human experience should feel like one guided conversation. The system execution must remain owner-separated.

## 7. Additional Role Requirements

Selene should ask for position-specific requirements where relevant:

- certifications;
- licences;
- training;
- safety requirements;
- language requirements;
- equipment;
- travel requirements;
- availability;
- physical site requirements where lawful;
- document proof;
- compliance modules;
- required software systems;
- required customer/supplier access;
- roster/schedule expectations.

Anti-discrimination rule:

- do not casually ask for gender, age, or protected traits;
- only lawful eligibility constraints may be captured;
- jurisdiction-specific HR/legal governance is required;
- PH1.WRITE must phrase this carefully;
- PH1.D/GPT-5.5 may propose questions, but HR/legal/access validation must prevent discriminatory fields.

Examples:

- A warehouse role may require forklift certification.
- A finance role may require confidentiality training and elevated audit.
- A customer-support role may require approved communication channel training.
- A contractor role may require insurance proof and contract scope.

## 8. Position Requirement Schema + Dynamic Fields

Future Position Journey Intelligence should use the current PH1.POSITION schema foundation to support richer dynamic requirements.

Requirements may include:

- forklift licence number;
- safety certification;
- superannuation member number;
- tax number;
- health number;
- bank details;
- emergency contact;
- payroll group;
- roster group;
- commission plan;
- language requirements;
- availability;
- equipment issue;
- identity documents;
- contractor insurance certificate.

Field requirements must support:

- field id;
- field name;
- field type;
- sensitivity;
- exposure rule;
- evidence mode;
- sender prefill allowed;
- receiver input allowed;
- validation owner;
- access-to-view;
- access-to-edit;
- retention policy;
- stale/expiry rule;
- country/region applicability;
- industry applicability;
- company-size applicability;
- tenant/workspace applicability;
- position applicability.

Dynamic field scope options:

- one-time for one person/session;
- permanent for one position;
- tenant/company overlay;
- workspace overlay;
- country/region overlay;
- industry overlay;
- global/default template.

Rules:

- sensitive fields require access/privacy classification;
- payroll/health/legal/identity fields require stronger governance;
- PH1.POSITION may coordinate requirement shape, but Payroll/HR/Compliance/Access owners validate their own truth;
- existing onboarding sessions must not be silently broken by new requirements;
- completed valid fields must not be re-requested unless stale, expired, or invalid.

## 9. Country / Industry / Company-Size Overlays

Selene must support overlays for:

- country;
- region/state/province;
- industry;
- company size;
- tenant/company;
- workspace;
- position family.

Examples:

- Australian employee roles may require superannuation details.
- Singapore employee roles may require CPF-related setup.
- Warehouse roles may require safety/forklift certification.
- Food manufacturing roles may require food safety certification.
- Finance roles may require confidentiality and step-up access.
- Small companies may use simpler approval paths.
- Large companies may require HR/Finance/Board approval.

Rules:

- PH1.POSITION may store/apply requirement overlays;
- Payroll/Compliance/Search owners validate changing legal/tax rules;
- PH1.E/Search may verify current public/legal data where needed;
- GPT-5.5 may explain, not decide;
- no legal/tax/labor law claim should be treated as truth unless source-backed or owner-approved.

## 10. Position Hierarchy + Reporting Line

PH1.POSITION or the canonical org owner must model:

- job family;
- department;
- reporting line;
- manager position;
- supervisor position;
- seniority;
- org chart relation;
- who manages whom;
- who may approve which position changes.

Example hierarchy:

CEO
-> Operations Manager
-> Warehouse Manager
-> Warehouse Supervisor
-> Warehouse Worker

Rules:

- hierarchy belongs to PH1.POSITION or canonical org owner;
- access hierarchy belongs to Access/Governance;
- position hierarchy may suggest access candidates, but does not grant access;
- reporting line may drive approval routing, but Access/Governance validates authority;
- role seniority must not become permission without explicit access template proof.

## 11. Position Lifecycle: Create, Update, Suspend, Retire, Supersede, Delete

Current repo-truth states include:

- Draft;
- Active;
- Suspended;
- Retired.

Future design may add:

- PendingReview;
- Superseded;
- Archived.

Position removal rules:

- draft position with no usage may be cancelled/deleted if policy allows;
- active or historically used position should be retired or superseded, not erased;
- retired position remains for audit and historical records;
- superseded position maps to replacement position where policy allows;
- existing employees/users are not automatically moved without confirmation and authority.

Example:

Old role = Junior Stock Clerk.

New role = Inventory Assistant.

Selene asks:

`Do you want to retire Junior Stock Clerk for future hires only, migrate current users to Inventory Assistant, or keep both active?`

Any lifecycle transition must preview downstream impacts for onboarding, access, payroll/HR, finance/budget, scheduler/roster, and current users where applicable.

## 12. Position-To-Access Template Co-Authoring

Desired full cycle:

User creates position.

Selene suggests access template.

Authorized person reviews permissions.

Access/Governance stores approved access template.

Future onboarding uses position + access template.

Example:

Create Warehouse Manager.

PH1.POSITION:

- department = warehouse;
- reports to = Operations Manager;
- requires forklift certification;
- requires safety training;
- uses roster group.

Access/Governance:

- can view warehouse roster;
- can approve stock adjustments up to limit;
- can view inventory reports;
- cannot view payroll;
- cannot grant access.

Rules:

- PH1.POSITION may suggest access template candidates;
- Access/Governance validates and stores permissions;
- no access is granted from position name alone;
- role strings must not become production access authority;
- template changes follow Master Access propagation rules;
- position-to-access co-authoring must preserve separate owners.

## 13. Position-To-Onboarding Handoff

Position requirements feed onboarding.

Flow:

Position active
-> requirement schema active
-> onboarding link created for that position
-> PH1.ONB pins schema/version
-> sender prefill collected
-> receiver provides required fields
-> document/voice/device/consent gates applied
-> Access creates per-user access after requirements pass.

Rules:

- PH1.ONB consumes position requirements;
- PH1.ONB does not invent position requirements;
- PH1.POSITION does not onboard the person;
- PH1.WRITE explains fields;
- PH1.REM handles follow-up if user postpones;
- PH1.ONB must not restart valid progress when a user resumes;
- PH1.ONB must not ask again for completed valid fields.

## 14. Position-To-Payroll / HR / Compensation Boundary

Useful compensation ideas must be modernized into the current owner split.

Position may coordinate:

- compensation band ref;
- expected employment type;
- required payroll setup fields;
- salary package draft context;
- benefits/allowance requirement references;
- commission/bonus applicability;
- overtime applicability;
- contractor payment mode reference.

Position must not own:

- actual salary truth;
- tax calculation;
- payroll disbursement;
- benefit calculation;
- contractor payment release;
- final compensation approval.

Compensation Engine owns:

- base salary proposal;
- allowance logic;
- commission/bonus logic;
- overtime logic;
- compensation package construction;
- market salary benchmark interpretation.

Payroll owns:

- payroll rules;
- deductions;
- contributions;
- payslips;
- final payment scheduling.

HR owns:

- employment terms;
- probation;
- employment status;
- contract/offer terms.

Finance/Budget owns:

- cost-to-company forecast;
- budget availability;
- margin/profitability impact;
- company financial risk.

Access/Governance owns:

- who may view/change/approve compensation fields.

## 15. Salary Benchmark + Cost-To-Company Preview

During position setup, Selene may offer:

- market salary benchmark;
- recommended salary range;
- base salary estimate;
- allowances;
- benefits;
- employer contributions;
- net/take-home estimate;
- cost-to-company estimate;
- budget/profitability impact;
- margin threshold warning.

Owner split:

- PH1.E/Search or approved HR data provider verifies current benchmark data;
- Compensation Engine interprets compensation package;
- Payroll/Compliance validates tax/contribution rules;
- Finance/Budget validates cost and budget impact;
- PH1.WRITE explains in human language;
- PH1.POSITION stores only approved refs/context, not payroll truth.

Example:

`A Logistics Supervisor in Kuala Lumpur usually benchmarks around this range. The estimated cost-to-company is RM 7,460. Would you like the breakdown or summary?`

No salary/tax/legal claim may be invented by GPT-5.5.

Where freshness matters, Selene must verify through approved public or private sources and expose uncertainty honestly.

## 16. Compensation Override + Approval Escalation

If a user proposes salary above benchmark or outside margin/budget, Selene must:

- capture proposed amount;
- capture reason/rationale;
- compare benchmark vs proposal;
- check budget/profitability thresholds;
- determine approval route;
- escalate to authorized approver(s);
- log who proposed and who approved;
- record timestamps;
- preserve benchmark and final values.

Example:

`You entered ¥16,500 for a role benchmarked at ¥10,800. Why is this salary higher?`

Approval paths may include:

- HR Manager;
- Finance Manager;
- CFO;
- CEO;
- Chairman;
- Board;
- dual AP;
- N-of-M;
- company-defined approval matrix.

Position coordinates the position setup.

Compensation/Finance/Access owners decide salary approval.

PH1.POSITION does not approve pay.

## 17. Country / Industry Compliance + Allowance Localization

Selene may help identify:

- country contributions;
- social insurance;
- CPF/superannuation/retirement fund;
- medical/insurance obligations;
- housing/meal/travel allowances;
- country tax bands;
- industry awards/agreements;
- employee vs contractor treatment;
- employer vs employee contribution split.

Owner split:

- Payroll/Compliance owns rule truth;
- PH1.E/Search verifies current public law where required;
- Compensation Engine computes package;
- PH1.WRITE explains;
- PH1.POSITION stores references/requirements.

No legal/payroll/tax rule should be final unless source-backed or owner-approved.

If a rule depends on country, state, province, industry, company size, agreement, award, union, or contract type, Selene must clarify scope before presenting it as actionable setup truth.

## 18. Contractor Position + Billing Mode Handoff

Contractors must be included but treated differently from employees.

Contractor setup fields may include:

- contractor type;
- contractor company/legal name;
- ABN/business registration or local equivalent;
- project/work package;
- internal manager/owner;
- start date;
- end date;
- billing type: hourly or lump sum;
- rate;
- estimated hours;
- payment schedule;
- deliverables;
- scope;
- insurance certificates;
- compliance documents;
- site access needs;
- geolocation requirement if lawful and consented;
- renewal reminder.

Billing modes:

- hourly;
- lump sum;
- milestone;
- retainer;
- project-based.

Hourly contractor logic:

- estimated hours;
- hour logging;
- progress summaries;
- overrun alerts at configurable threshold such as 80% or 90%;
- manager review;
- payment approval handoff.

Lump sum contractor logic:

- fixed fee;
- milestone/deadline;
- no hour tracking unless enabled;
- completion approval.

Owner split:

- PH1.POSITION defines contractor role/category and requirements;
- Contract/Supplier/AP owner owns contract truth;
- Finance/AP owns invoices/payments;
- Scheduler/Roster or Task owner owns time/work logs where applicable;
- PH1.REM owns renewal/expiry/overrun reminders;
- PH1.BCAST/DELIVERY owns notifications;
- Access owns limited contractor permissions.

Example:

`Add contractor Michael Youssef, freelance designer, project-based, Dubai.`

Selene asks:

`Is this hourly or lump sum?`

If hourly:

`What hourly rate and estimated hours should I use?`

Selene may suggest market range through approved data, but Compensation/Finance/AP validate.

## 19. Contractor Time / Progress / Overrun Handoff

Supported future journeys:

- clock-in / clock-out by voice or app;
- weekly progress summary;
- hours logged vs estimated;
- task delivered vs time spent;
- overrun alert;
- manager approval for additional hours;
- payment release request;
- contract expiry reminder.

Owner split:

- Scheduler/Task owns time/work logs;
- Finance/AP owns payment;
- Contract owner owns contract scope;
- PH1.REM owns reminders;
- PH1.POSITION owns contractor role requirements only.

Geolocation:

- may be used only when contract/policy/consent allows;
- Device/Human Presence owner provides evidence;
- PH1.POSITION does not track location.

PH1.POSITION may hold references that a contractor role requires site evidence, but the evidence itself remains with the correct device, presence, document, task, or contract owner.

## 20. Job Description Drafting

Future job description flow:

Position facts
-> PH1.D/GPT-5.5 drafts job description
-> PH1.WRITE finalizes
-> HR/Recruitment owner publishes or stores recruitment copy where applicable
-> PH1.POSITION stores refs, not recruitment publication truth.

Selene should say:

`Here is a draft description based on the role details. Would you like to review or edit anything?`

Job descriptions should include:

- title;
- department;
- reporting line;
- duties;
- required skills;
- certifications;
- schedule;
- location;
- employment type;
- lawful eligibility constraints;
- access-sensitive responsibilities only if appropriate;
- compensation summary only if policy allows.

PH1.WRITE must avoid discriminatory wording and must not turn draft recruitment text into active hiring or contract truth.

## 21. Batch Position / Opening Creation

Batch setup must distinguish a position template from openings/headcount.

User says:

`Create 5 warehouse assistant roles in Brisbane, all full-time, same hours, same pay.`

Selene must clarify:

`Should these be five openings under one Warehouse Assistant position template, or five separate position records?`

Preferred model:

- Position template = Warehouse Assistant;
- Headcount/openings = 5;
- HR/Recruitment owner tracks vacancies/headcount;
- Finance/Budget gets combined cost forecast;
- Scheduler/Roster gets workforce planning refs;
- Payroll/Compensation handles pay refs.

PH1.POSITION must not confuse number of openings with separate job definitions.

If the user intends five distinct jobs with different reporting lines, locations, or compensation refs, Selene should create separate draft position proposals after confirmation.

## 22. Conditional Offer Handoff

Conditional offer behavior belongs to offer/HR owners, not PH1.POSITION.

Example:

`Offer $2,900/month to John, but only if he agrees to six-month probation and weekend availability.`

Owner split:

- PH1.POSITION provides role context;
- HR/Offer owner owns offer terms;
- Payroll/Compensation owns pay package;
- Scheduler/Roster owns weekend availability implications;
- Access owns future access template;
- PH1.ONB handles onboarding after acceptance;
- PH1.REM can track response deadline;
- PH1.BCAST/DELIVERY can send offer communication.

PH1.POSITION must not own employment contract truth.

PH1.WRITE should explain what is being prepared, what is only a draft, what needs approval, and what would happen if the candidate accepts.

## 23. Final Position Activation Routing

When a position is finalized/activated, Selene may route structured refs to:

- PH1.ONB for onboarding requirements;
- Access/Governance for access-template linkage;
- Payroll/HR for employment/pay setup requirements;
- Compensation for package benchmarks/refs;
- Finance/Budget for cost forecast;
- Scheduler/Roster for scheduling/roster requirements;
- PH1.REM for review/expiry/reminder needs;
- PH1.BCAST/DELIVERY for notifications;
- Document/Artifact owners for proof requirements.

Rules:

- PH1.POSITION emits refs and context;
- each owner stores its own truth;
- no owner is bypassed;
- no payroll/finance/access/scheduler mutation occurs from PH1.POSITION alone;
- activation requires confirmation, authority, and audit where policy requires.

The final activation output should be an owner-scoped handoff bundle, not an uncontrolled all-in-one mutation.

## 24. Position Update / Retire / Supersede / Remove

Supported future lifecycle changes:

- update position metadata;
- update requirements schema;
- activate new schema version;
- retire position;
- suspend position;
- supersede position;
- cancel/delete safe draft only.

Rules:

- draft with no usage may be deleted/cancelled where policy allows;
- active/historically used positions must be retired/superseded, not erased;
- current users are not automatically moved without explicit confirmation;
- Access template propagation rules apply separately;
- Onboarding backfill is required if active requirements change for current users;
- Payroll/HR/Scheduler/Access impacts must be previewed.

Example:

`Retire Junior Stock Clerk and replace with Inventory Assistant.`

Selene must preview:

- affected current users;
- onboarding requirement changes;
- access template impact;
- payroll/roster impact;
- required approvals.

PH1.POSITION records the position lifecycle decision. Each downstream owner applies only its own approved change.

## 25. PH1.D / GPT-5.5 + PH1.N Position Proposal Path

OpenAI/GPT-5.5 may help propose:

- role title;
- department;
- likely job family;
- likely required fields;
- likely certifications;
- likely job description;
- likely access template candidates;
- likely compensation benchmark context;
- clarification questions;
- batch/conditional intent.

PH1.N extracts:

- position title;
- department;
- location;
- schedule;
- employment type;
- contractor type;
- access needs;
- payroll/compensation needs;
- roster/scheduler needs;
- missing fields;
- risk hints.

PH1.X validates:

- position creation/update lane;
- access-changing risk;
- compensation risk;
- protected action gates;
- required confirmation;
- stack owners.

PH1.WRITE explains.

OpenAI/GPT-5.5 must not:

- activate positions;
- grant access;
- approve compensation;
- create payroll;
- publish job ads;
- mutate rosters;
- create legal obligations.

## 26. PH1.WRITE Position Guidance Boundary

PH1.WRITE owns final wording for:

- position creation questions;
- missing information prompts;
- job description drafts;
- salary benchmark explanations;
- compensation override prompts;
- contractor setup questions;
- conditional offer explanations;
- position activation confirmation;
- position retirement/supersede explanations;
- access-template review wording;
- handoff summaries.

No raw reason-code dumping to users.

PH1.WRITE should translate deterministic owner results into clear human language without inventing authority, approval, legal status, salary certainty, tax truth, or execution results.

## 27. Security / Privacy / Governance Standards

Required standards:

- only authorized users can create/update/retire positions;
- sensitive fields require access/privacy classification;
- compensation/payroll fields require Access/Payroll/HR governance;
- access-template suggestions require Access review;
- contractor geolocation/time tracking requires consent and policy;
- lawful eligibility rules only;
- no discriminatory field capture;
- legal/tax/compliance rules require source-backed or owner-approved truth;
- all position changes are audited;
- activation and retirement require confirmation;
- downstream impacts must be previewed.

Governance-sensitive fields include:

- salary bands;
- payroll setup requirements;
- tax identifiers;
- health identifiers;
- bank/payment details;
- HR notes;
- disciplinary or suitability fields;
- supplier/contractor banking refs;
- access templates;
- approval limits;
- protected action roles.

Selene must deny, clarify, or escalate where authority, privacy classification, legal source, or owner proof is missing.

## 28. Required Logical Packets

Future logical packets:

- PositionJourneyRequestPacket
- PositionIntentCandidatePacket
- PositionTypeResolutionPacket
- StandardPositionTemplatePacket
- IndustryPositionPackPacket
- PositionRequirementSchemaPacket
- PositionDynamicFieldPacket
- PositionOverlayPacket
- PositionHierarchyPacket
- PositionReportingLinePacket
- PositionAccessTemplateProposalPacket
- PositionOnboardingHandoffPacket
- PositionCompensationPreviewRequestPacket
- PositionSalaryBenchmarkPacket
- PositionCostToCompanyPacket
- CompensationOverrideEscalationPacket
- ContractorPositionSetupPacket
- ContractorBillingModePacket
- ContractorOverrunAlertPacket
- JobDescriptionDraftPacket
- BatchPositionCreationPacket
- ConditionalOfferHandoffPacket
- PositionFinalizationRoutingPacket
- PositionRetireSupersedePacket
- PositionAuditEvidencePacket

Codex must later map these to repo equivalents.

Do not claim they currently exist.

## 29. Example End-To-End Flows

Example A - Warehouse Manager:

User:

`Create a Warehouse Manager role.`

Flow:

Selene guides department/location/reporting line/certifications
-> suggests access template
-> offers compensation preview
-> PH1.POSITION stores job truth
-> Access stores permissions
-> ONB uses requirements.

Example B - Contractor:

User:

`Add contractor Michael Youssef, freelance designer, project-based, Dubai.`

Flow:

contractor type resolved
-> hourly vs lump sum clarified
-> contract owner/AP/Finance handoff
-> access limited
-> expiry reminder scheduled.

Example C - Batch openings:

User:

`Create 5 warehouse assistant roles in Brisbane.`

Flow:

Selene clarifies one shared position with 5 openings vs separate position records
-> cost forecast routed to Finance
-> headcount/opening truth routed to HR/Recruitment.

Example D - Conditional offer:

User:

`Offer John $2,900 if he agrees to six-month probation and weekends.`

Flow:

HR/Offer owner handles offer
-> Compensation validates pay
-> Scheduler/Roster validates weekend availability impact
-> PH1.REM tracks response.

Example E - Salary override:

User proposes salary above benchmark.

Flow:

Selene captures reason
-> Finance/Compensation checks threshold
-> Access/Governance routes dual approval if required
-> PH1.WRITE explains.

Example F - Retire position:

User retires Junior Stock Clerk.

Flow:

Selene previews current users, access impact, ONB impact, payroll/roster impact
-> authorized person confirms
-> PH1.POSITION retires/supersedes
-> downstream owners handle their truth.

## 30. What Must Not Happen

- no copying old compensation design as-is;
- no old invalid product/persona names;
- no PH1.POSITION owning payroll truth;
- no PH1.POSITION owning Finance/Budget truth;
- no PH1.POSITION owning Access permissions;
- no PH1.POSITION owning Scheduler/Roster truth;
- no PH1.POSITION owning contractor payments;
- no GPT-5.5 directly activating positions;
- no salary benchmark claim without source/owner proof;
- no tax/labor/compliance claim without source/owner proof;
- no gender/age/protected-trait preference without lawful governance;
- no contractor geolocation tracking without consent/policy;
- no active position deletion where historical usage exists;
- no access granted from position name alone;
- no payroll/finance/scheduler mutation from PH1.POSITION alone;
- no implementation from this document alone.

## 31. Required Upgrade List

1. Standard Position Library + Industry Packs
2. Guided Position Creation Flow
3. Contractor vs Employee Distinction
4. Position Type Matrix
5. Position Hierarchy + Reporting Line
6. Department + Job Family Registry
7. Dynamic Position Requirement Fields
8. Country / Region / Industry / Company-Size Overlays
9. Position-To-Onboarding Handoff
10. Position-To-Access Template Co-Authoring
11. Position-To-Payroll/HR Boundary
12. Position-To-Compensation Boundary
13. Position-To-Finance/Budget Boundary
14. Position-To-Scheduler/Roster Boundary
15. Salary Benchmark + Cost-To-Company Handoff
16. Compensation Override + Approval Escalation
17. Contractor Billing Mode Handoff
18. Contractor Time / Progress / Overrun Handoff
19. Job Description Drafting
20. Batch Position / Opening Creation
21. Conditional Offer Handoff
22. Final Position Activation Routing
23. Position Retire / Supersede / Remove Flow
24. PH1.D + PH1.N Position Proposal Shell
25. PH1.X Position Route/Risk Validation
26. PH1.WRITE Position Guidance Boundary
27. Position Audit Evidence Pack
28. JD Live Position Acceptance Pack

## 32. Required Implementation Slices

These are required later after Grand Architecture Reconciliation and repo-truth activation.

This docs-only task does not implement them.

1. Position Journey Intelligence Activation Pack
2. Standard Position Library + Industry Packs
3. Position Type Matrix
4. Position Hierarchy / Reporting Line
5. Department / Job Family Registry
6. Position Requirement Schema Expansion
7. Dynamic Field / Requirement Overlay Governance
8. Country / Region / Industry / Company-Size Overlay Proof
9. Contractor Position Flow
10. Contractor Billing Mode Handoff
11. Position-To-Onboarding Requirement Handoff
12. Position-To-Access Template Co-Authoring
13. Position-To-Payroll/HR Boundary Proof
14. Position-To-Compensation Boundary Proof
15. Position-To-Finance/Budget Boundary Proof
16. Position-To-Scheduler/Roster Boundary Proof
17. Salary Benchmark / Cost-To-Company Handoff
18. Compensation Override / Approval Escalation
19. Contractor Time / Progress / Overrun Handoff
20. Job Description Drafting Through PH1.WRITE
21. Batch Position / Opening Creation
22. Conditional Offer Handoff
23. Position Retire / Supersede / Remove Flow
24. PH1.D + PH1.N Position Proposal Shell
25. PH1.X Position Route/Risk Validation
26. PH1.WRITE Position Guidance Boundary
27. Position Audit Evidence Pack
28. Desktop/iPhone Render-Only Position Proof
29. Adapter Transport-Only Position Proof
30. JD Live Position Acceptance Pack

## 33. Grand Architecture Reconciliation Note

This document must later be reconciled into:

- PH1.POSITION extraction
- PH1.ONB Journey
- Master Access Governance + Per-User Access Journey
- Payroll/HR future owner
- Compensation future owner
- Finance/Budget future owner
- Scheduler/Roster future owner
- PH1.D Proposal Gateway
- PH1.N Meaning Unravelling
- PH1.X Request Decision Lattice
- PH1.WRITE Human Presentation
- PH1.REM Reminder Journey
- PH1.BCAST / PH1.DELIVERY
- Tenant / Workspace Governance
- Document / Artifact / Media proof stacks
- Desktop/iPhone render-only proof
- Adapter transport-only proof
- Old Compatibility Path Retirement

Reconciliation must preserve the repo-truth fact that PH1.POSITION owns position lifecycle and requirement schemas while preventing PH1.POSITION from absorbing access, payroll, compensation, finance, scheduler, roster, contract, reminder, delivery, or writing ownership.

## 34. Final Architecture Sentence

“PH1.POSITION defines Selene’s job and position truth; Position Journey Intelligence lets Selene guide users through standard roles, custom roles, contractor roles, requirements, job descriptions, access-template co-authoring, compensation previews, payroll/HR/finance/scheduler handoffs, and position lifecycle changes while keeping Access, Onboarding, Payroll/HR, Compensation, Finance/Budget, Scheduler/Roster, Reminder, Broadcast/Delivery, PH1.D, PH1.N, PH1.X, and PH1.WRITE as separate canonical owners.”
